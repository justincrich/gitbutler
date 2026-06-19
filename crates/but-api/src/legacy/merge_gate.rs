use std::{collections::BTreeMap, path::Path, str};

use anyhow::{Context as _, anyhow};
use but_authz::{
    Authority, AuthoritySet, BranchName, BranchProtection, Denial, GovConfig, Group, GroupName,
    PrincipalId,
};
use serde::{Deserialize, Serialize};

#[path = "review_requirement.rs"]
mod review_requirement;

const PERMISSIONS_PATH: &str = ".gitbutler/permissions.toml";
const GATES_PATH: &str = ".gitbutler/gates.toml";
const CONFIG_INVALID_CODE: &str = "config.invalid";
const REVIEW_REQUIRED_CODE: &str = "gate.review_required";

/// Structured merge-gate error payload for CLI and API callers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MergeGateError {
    /// Stable consumer-facing error code.
    pub code: &'static str,
    /// Human-readable denial message.
    pub message: String,
    /// Actionable recovery hint for the denied actor.
    pub remediation_hint: String,
    /// Requirement fragments that were not satisfied.
    pub unmet: Vec<String>,
}

impl std::fmt::Display for MergeGateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for MergeGateError {}

/// Enforce merge authority and the target-ref review requirement for a forge review.
pub fn enforce_merge_gate(ctx: &but_ctx::Context, review_id: usize) -> anyhow::Result<()> {
    let review = review_for_id(ctx, review_id)?;
    let target_ref = branch_ref(&review.target_branch);
    let source_ref = branch_ref(&review.source_branch);
    let repo = ctx.repo.get()?;
    let config = load_merge_governance_config(&repo, &target_ref)?;

    let principal = but_authz::resolve_principal_from_env(&config.gov)?;
    but_authz::authorize(&principal, Authority::Merge, &config.gov)?;

    if !config
        .gov
        .branch(&review.target_branch)
        .is_some_and(|branch| branch.protected())
    {
        return Ok(());
    }

    let Some(requirement) = config.review_requirement_for(&review.target_branch) else {
        return Ok(());
    };

    let current_head_oid = current_head_oid(&repo, &source_ref)?;
    let author = review
        .author
        .as_ref()
        .map(|author| PrincipalId::new(author.clone()))
        .ok_or_else(|| anyhow!("review {review_id} has no author for review requirement"))?;
    let verdicts = review_verdicts(ctx, &review.source_branch)?;

    match review_requirement::evaluate(
        requirement,
        &verdicts,
        &current_head_oid,
        &author,
        &config.gov,
    ) {
        Ok(()) => Ok(()),
        Err(unmet) => {
            let unmet = unmet.into_entries();
            Err(MergeGateError {
                code: REVIEW_REQUIRED_CODE,
                message: format!(
                    "review requirement for {} is not satisfied: {}",
                    review.target_branch,
                    unmet.join("; ")
                ),
                remediation_hint: "collect the required approvals at the current review head"
                    .to_owned(),
                unmet,
            }
            .into())
        }
    }
}

/// Extract a structured merge-gate payload from an error chain.
pub fn classify_error(err: &anyhow::Error) -> Option<MergeGateError> {
    if let Some(error) = err.downcast_ref::<MergeGateError>() {
        return Some(error.clone());
    }

    err.downcast_ref::<Denial>().map(|denial| MergeGateError {
        code: denial.code,
        message: denial.message.clone(),
        remediation_hint: denial.remediation_hint.clone(),
        unmet: Vec::new(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MergeGovernanceConfig {
    gov: GovConfig,
    requirements: Vec<ReviewRequirement>,
}

impl MergeGovernanceConfig {
    fn review_requirement_for(&self, branch_name: &str) -> Option<&ReviewRequirement> {
        self.requirements
            .iter()
            .find(|requirement| requirement.branch.as_str() == branch_name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReviewRequirement {
    branch: BranchName,
    min_approvals: usize,
    require_distinct_from_author: bool,
    require_approval_from_group: Vec<GroupName>,
}

fn review_for_id(ctx: &but_ctx::Context, review_id: usize) -> anyhow::Result<but_db::ForgeReview> {
    let review_number = i64::try_from(review_id).context("review id does not fit in i64")?;
    ctx.db
        .get_cache()?
        .forge_reviews()
        .list_all()?
        .into_iter()
        .find(|review| review.number == review_number)
        .ok_or_else(|| anyhow!("review {review_id} is not present in the local forge cache"))
}

fn review_verdicts(
    ctx: &but_ctx::Context,
    target: &str,
) -> anyhow::Result<Vec<but_db::LocalReviewVerdict>> {
    let verdicts = ctx
        .db
        .get_cache()?
        .local_review_verdicts()
        .list_by_target(target)?;

    Ok(verdicts)
}

fn current_head_oid(repo: &gix::Repository, source_ref: &str) -> anyhow::Result<String> {
    Ok(repo
        .find_reference(source_ref)
        .with_context(|| format!("resolving source ref {source_ref}"))?
        .peel_to_id()
        .with_context(|| format!("peeling {source_ref} to an object id"))?
        .to_string())
}

fn load_merge_governance_config(
    repo: &gix::Repository,
    target_ref: &str,
) -> anyhow::Result<MergeGovernanceConfig> {
    let permissions_blob = read_config_blob(repo, target_ref, PERMISSIONS_PATH)?;
    let gates_blob = read_config_blob(repo, target_ref, GATES_PATH)?;
    let permissions = toml::from_str::<PermissionsWire>(&permissions_blob).map_err(|err| {
        config_invalid(format!("parsing {PERMISSIONS_PATH} at {target_ref}: {err}"))
    })?;
    let gates = toml::from_str::<GatesWire>(&gates_blob)
        .map_err(|err| config_invalid(format!("parsing {GATES_PATH} at {target_ref}: {err}")))?;

    let (principals, groups) = normalize_permissions(permissions)?;
    let (branches, requirements) = normalize_gates(gates)?;

    Ok(MergeGovernanceConfig {
        gov: GovConfig::new(principals, groups, branches),
        requirements,
    })
}

fn read_config_blob(
    repo: &gix::Repository,
    target_ref: &str,
    path: &'static str,
) -> anyhow::Result<String> {
    let mut reference = repo
        .find_reference(target_ref)
        .with_context(|| format!("resolving target ref {target_ref}"))
        .map_err(config_error)?;
    let commit = reference
        .peel_to_commit()
        .with_context(|| format!("peeling {target_ref} to a commit"))
        .map_err(config_error)?;
    let tree = commit
        .tree()
        .with_context(|| format!("reading tree for {target_ref}"))
        .map_err(config_error)?;
    let entry = tree
        .lookup_entry_by_path(Path::new(path))
        .with_context(|| format!("looking up {path} in {target_ref}"))
        .map_err(config_error)?
        .ok_or_else(|| config_invalid(format!("missing {path} at {target_ref}")))?;
    let blob = repo
        .find_blob(entry.id())
        .with_context(|| format!("reading {path} blob at {target_ref}"))
        .map_err(config_error)?;
    let content = str::from_utf8(&blob.data)
        .with_context(|| format!("decoding {path} at {target_ref} as UTF-8"))
        .map_err(config_error)?;

    Ok(content.to_owned())
}

fn normalize_permissions(
    permissions: PermissionsWire,
) -> anyhow::Result<(
    BTreeMap<PrincipalId, AuthoritySet>,
    BTreeMap<GroupName, Group>,
)> {
    let mut groups = BTreeMap::new();
    let mut group_authorities = BTreeMap::new();

    for group in &permissions.group {
        let name = GroupName::new(group.name.clone());
        let authorities = authority_set_from_wire(&group.permissions, group.role.as_deref())?;
        let members = group.members.iter().cloned().map(PrincipalId::new);
        if groups
            .insert(
                name.clone(),
                Group::new(name.clone(), authorities.clone(), members),
            )
            .is_some()
        {
            return Err(config_invalid(format!("duplicate group {}", group.name)).into());
        }
        group_authorities.insert(name, authorities);
    }

    let mut principals = BTreeMap::new();
    for principal in &permissions.principal {
        let mut authorities =
            authority_set_from_wire(&principal.permissions, principal.role.as_deref())?;
        for group_name in &principal.groups {
            let group_key = GroupName::new(group_name.clone());
            let group_set = group_authorities.get(&group_key).ok_or_else(|| {
                config_invalid(format!(
                    "principal {} references undefined group {group_name}",
                    principal.id
                ))
            })?;
            authorities = authorities.union(group_set);
        }

        let id = PrincipalId::new(principal.id.clone());
        if principals.insert(id, authorities).is_some() {
            return Err(config_invalid(format!("duplicate principal {}", principal.id)).into());
        }
    }

    for group in &permissions.group {
        let group_key = GroupName::new(group.name.clone());
        let group_set = group_authorities
            .get(&group_key)
            .ok_or_else(|| config_invalid(format!("group {} was not normalized", group.name)))?;

        for member in &group.members {
            let id = PrincipalId::new(member.clone());
            let authorities = principals
                .get(&id)
                .map_or_else(|| group_set.clone(), |existing| existing.union(group_set));
            principals.insert(id, authorities);
        }
    }

    Ok((principals, groups))
}

fn normalize_gates(
    gates: GatesWire,
) -> anyhow::Result<(
    BTreeMap<BranchName, BranchProtection>,
    Vec<ReviewRequirement>,
)> {
    let mut branches = BTreeMap::new();
    for branch in gates.branch {
        let name = BranchName::new(branch.name.clone());
        if branches
            .insert(name, BranchProtection::new(branch.protected))
            .is_some()
        {
            return Err(config_invalid(format!("duplicate branch {}", branch.name)).into());
        }
    }

    let mut requirements = Vec::new();
    for gate in gates.gate {
        if gate.kind != "review" {
            continue;
        }
        requirements.push(ReviewRequirement {
            branch: BranchName::new(gate.branch),
            min_approvals: gate.min_approvals,
            require_distinct_from_author: gate.require_distinct_from_author,
            require_approval_from_group: gate
                .require_approval_from_group
                .into_iter()
                .map(GroupName::new)
                .collect(),
        });
    }

    Ok((branches, requirements))
}

fn authority_set_from_wire(
    permissions: &[String],
    role: Option<&str>,
) -> anyhow::Result<AuthoritySet> {
    let listed = AuthoritySet::parse(permissions.iter().map(String::as_str))
        .map_err(|err| config_invalid(format!("parsing authority list: {err}")))?;
    let role_set = AuthoritySet::from_optional_role(role)
        .map_err(|err| config_invalid(format!("desugaring authority role: {err}")))?;

    Ok(listed.union(&role_set))
}

fn branch_ref(branch_name: &str) -> String {
    if branch_name.starts_with("refs/") {
        branch_name.to_owned()
    } else {
        format!("refs/heads/{branch_name}")
    }
}

fn config_error(err: anyhow::Error) -> MergeGateError {
    config_invalid(err.to_string())
}

fn config_invalid(message: String) -> MergeGateError {
    MergeGateError {
        code: CONFIG_INVALID_CODE,
        message: format!("invalid governance config: {message}"),
        remediation_hint: "fix committed .gitbutler governance config on the target ref".to_owned(),
        unmet: Vec::new(),
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PermissionsWire {
    #[serde(default)]
    principal: Vec<PrincipalWire>,
    #[serde(default)]
    group: Vec<GroupWire>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PrincipalWire {
    id: String,
    #[serde(default)]
    permissions: Vec<String>,
    role: Option<String>,
    #[serde(default)]
    groups: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GroupWire {
    name: String,
    #[serde(default)]
    permissions: Vec<String>,
    role: Option<String>,
    #[serde(default)]
    members: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GatesWire {
    #[serde(default)]
    branch: Vec<BranchWire>,
    #[serde(default)]
    gate: Vec<GateWire>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BranchWire {
    name: String,
    protected: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GateWire {
    branch: String,
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    min_approvals: usize,
    #[serde(default)]
    require_approval_from_group: Vec<String>,
    #[serde(default)]
    require_distinct_from_author: bool,
}
