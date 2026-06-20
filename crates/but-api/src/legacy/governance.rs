use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context as _, anyhow};
use but_api_macros::but_api;
use but_authz::{
    Authority, AuthoritySet, Denial, GroupWire, PermissionsWire, PrincipalId, PrincipalWire,
    load_governance_config, permissions_path,
};
use but_ctx::Context;
use serde::Serialize;

use crate::json::ConfigInvalid;

use super::config_mutate::enforce_administration_write_gate;

/// Operator-facing caveat for working-tree governance writes.
pub const REF_PIN_CAVEAT: &str = "takes effect once committed to the target branch";

/// Result of a governance permission write.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PermWriteOutcome {
    /// Principal whose direct permissions were changed or inspected.
    pub principal: String,
    /// Parsed authority tokens supplied by the caller.
    pub authorities: Vec<String>,
    /// Ref-pin caveat for the operator.
    pub caveat: &'static str,
}

/// Result of a governance permission grant exposed through the API boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GrantOutcome {
    /// Principal whose direct permissions were changed or inspected.
    pub principal: String,
    /// Parsed authority tokens supplied by the caller.
    pub authorities: Vec<String>,
    /// Ref-pin caveat for the operator.
    pub caveat: &'static str,
}

impl From<PermWriteOutcome> for GrantOutcome {
    fn from(outcome: PermWriteOutcome) -> Self {
        Self {
            principal: outcome.principal,
            authorities: outcome.authorities,
            caveat: outcome.caveat,
        }
    }
}

/// Serializable authority set returned by generated governance API wrappers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GovernanceStatus {
    /// Effective functional authority tokens for the caller.
    pub authorities: Vec<String>,
}

impl From<AuthoritySet> for GovernanceStatus {
    fn from(authorities: AuthoritySet) -> Self {
        Self {
            authorities: authorities
                .iter()
                .map(|authority| authority.name().to_owned())
                .collect(),
        }
    }
}

/// Listed authority for one principal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PermListEntry {
    /// Functional authority token.
    pub authority: String,
    /// Literal marker for a working-tree grant not committed at the target ref.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<&'static str>,
}

/// Result of listing one principal's permissions.
#[derive(Clone, PartialEq, Eq, Serialize)]
pub struct PermListOutcome {
    /// Principal whose permissions were listed.
    pub principal: String,
    /// Committed authorities plus pending working-tree direct grants.
    pub authorities: Vec<PermListEntry>,
}

impl std::fmt::Debug for PermListOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}:", self.principal)?;
        for entry in &self.authorities {
            match entry.marker {
                Some(marker) => writeln!(f, "  {} {marker}", entry.authority)?,
                None => writeln!(f, "  {}", entry.authority)?,
            }
        }
        Ok(())
    }
}

/// Result of a governance group write.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GroupWriteOutcome {
    /// Group whose grants or membership were changed or inspected.
    pub group: String,
    /// Parsed authority tokens supplied by the caller.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub authorities: Vec<String>,
    /// Principal membership changed by this operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member: Option<String>,
    /// Ref-pin caveat for the operator.
    pub caveat: &'static str,
}

/// Listed group grants and members.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GroupListEntry {
    /// Group name.
    pub name: String,
    /// Functional authorities granted to the group.
    pub authorities: Vec<String>,
    /// Principals listed as group members.
    pub members: Vec<String>,
}

/// Result of listing governed groups.
#[derive(Clone, PartialEq, Eq, Serialize)]
pub struct GroupListOutcome {
    /// Groups from the working-tree governance config.
    pub groups: Vec<GroupListEntry>,
}

/// Structured governance error payload for CLI and API callers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GovernanceErrorPayload {
    /// Stable consumer-facing error code.
    pub code: &'static str,
    /// Human-readable error message.
    pub message: String,
    /// Optional actionable recovery hint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation_hint: Option<String>,
}

impl std::fmt::Debug for GroupListOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for group in &self.groups {
            writeln!(f, "{}:", group.name)?;
            for authority in &group.authorities {
                writeln!(f, "  grant {authority}")?;
            }
            for member in &group.members {
                writeln!(f, "  member {member}")?;
            }
        }
        Ok(())
    }
}

/// List governed groups through the but-api boundary.
#[but_api]
pub fn group_list_cmd(ctx: &Context) -> anyhow::Result<GroupListOutcome> {
    let repo = ctx.repo.get()?;
    let target_ref = target_ref_from_ctx(ctx, None)?;
    group_list(&repo, &target_ref)
}

/// Create a governed group through the but-api boundary.
#[but_api]
pub fn group_create_cmd(
    ctx: &Context,
    target_ref: String,
    group: String,
    authorities: Vec<String>,
) -> anyhow::Result<GroupWriteOutcome> {
    let repo = ctx.repo.get()?;
    let target_ref = target_ref_from_ctx(ctx, Some(&target_ref))?;
    let authorities = authority_slices(&authorities);
    group_create(&repo, &target_ref, &group, &authorities)
}

/// Grant governed group permissions through the but-api boundary.
#[but_api]
pub fn group_grant_cmd(
    ctx: &Context,
    target_ref: String,
    group: String,
    authorities: Vec<String>,
) -> anyhow::Result<GroupWriteOutcome> {
    let repo = ctx.repo.get()?;
    let target_ref = target_ref_from_ctx(ctx, Some(&target_ref))?;
    let authorities = authority_slices(&authorities);
    group_grant(&repo, &target_ref, &group, &authorities)
}

/// Add a principal to a governed group through the but-api boundary.
#[but_api]
pub fn group_add_member_cmd(
    ctx: &Context,
    target_ref: String,
    group: String,
    member: String,
) -> anyhow::Result<GroupWriteOutcome> {
    let repo = ctx.repo.get()?;
    let target_ref = target_ref_from_ctx(ctx, Some(&target_ref))?;
    group_add_member(&repo, &target_ref, &group, &member)
}

/// Remove a principal from a governed group through the but-api boundary.
#[but_api]
pub fn group_remove_member_cmd(
    ctx: &Context,
    target_ref: String,
    group: String,
    member: String,
) -> anyhow::Result<GroupWriteOutcome> {
    let repo = ctx.repo.get()?;
    let target_ref = target_ref_from_ctx(ctx, Some(&target_ref))?;
    group_remove_member(&repo, &target_ref, &group, &member)
}

/// List governed direct permissions through the but-api boundary.
#[but_api]
pub fn perm_list_cmd(ctx: &Context, principal: Option<String>) -> anyhow::Result<PermListOutcome> {
    let repo = ctx.repo.get()?;
    let target_ref = target_ref_from_ctx(ctx, None)?;
    perm_list(&repo, &target_ref, principal.as_deref())
}

/// Grant governed direct permissions through the but-api boundary.
#[but_api]
pub fn perm_grant_cmd(
    ctx: &Context,
    target_ref: String,
    principal: String,
    authorities: Vec<String>,
) -> anyhow::Result<GrantOutcome> {
    let repo = ctx.repo.get()?;
    let target_ref = target_ref_from_ctx(ctx, Some(&target_ref))?;
    let authorities = authority_slices(&authorities);
    Ok(perm_grant(&repo, &target_ref, &principal, &authorities)?.into())
}

/// Revoke governed direct permissions through the but-api boundary.
#[but_api]
pub fn perm_revoke_cmd(
    ctx: &Context,
    target_ref: String,
    principal: String,
    authorities: Vec<String>,
) -> anyhow::Result<PermWriteOutcome> {
    let repo = ctx.repo.get()?;
    let target_ref = target_ref_from_ctx(ctx, Some(&target_ref))?;
    let authorities = authority_slices(&authorities);
    perm_revoke(&repo, &target_ref, &principal, &authorities)
}

/// Return the caller's own effective governance authorities.
#[but_api(GovernanceStatus)]
pub fn governance_status_read(ctx: &Context) -> anyhow::Result<AuthoritySet> {
    let repo = ctx.repo.get()?;
    let target_ref = target_ref_from_ctx(ctx, None)?;
    let config = load_governance_config(&repo, &target_ref)?;
    let caller = but_authz::resolve_principal_from_env(&config)?;
    Ok(but_authz::effective_authority(&caller, &config))
}

/// List governed groups under administration-read authority.
pub fn group_list(repo: &gix::Repository, target_ref: &str) -> anyhow::Result<GroupListOutcome> {
    let config = load_governance_config(repo, target_ref)?;
    let caller = but_authz::resolve_principal_from_env(&config)?;
    let held = but_authz::effective_authority(&caller, &config);
    if !held.contains(Authority::AdministrationRead)
        && !held.contains(Authority::AdministrationWrite)
    {
        return Err(Denial::missing_permission(Authority::AdministrationRead, &held).into());
    }

    let permissions = read_worktree_permissions(repo)?;
    let groups = permissions
        .group
        .iter()
        .map(group_list_entry)
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(GroupListOutcome { groups })
}

/// Create a governed group in the working-tree governance config.
pub fn group_create(
    repo: &gix::Repository,
    target_ref: &str,
    group: &str,
    authorities: &[&str],
) -> anyhow::Result<GroupWriteOutcome> {
    let parsed = parse_authorities(authorities)?;
    enforce_administration_write_gate(repo, target_ref)?;

    let mut permissions = load_permissions_for_write(repo, target_ref)?;
    let exists = permissions.group.iter().any(|entry| entry.name == group);
    if exists {
        return Err(Denial {
            code: "config.invalid",
            message: format!("group {group} already exists"),
            remediation_hint: "choose a unique group name or grant the existing group".to_owned(),
        }
        .into());
    }

    permissions.group.push(GroupWire {
        name: group.to_owned(),
        permissions: parsed
            .iter()
            .map(|authority| authority.name().to_owned())
            .collect(),
        role: None,
        members: Vec::new(),
    });
    write_worktree_permissions(repo, &permissions)?;

    Ok(group_write_outcome(group, &parsed, None))
}

/// Grant functional permissions to a governed group in the working-tree config.
pub fn group_grant(
    repo: &gix::Repository,
    target_ref: &str,
    group: &str,
    authorities: &[&str],
) -> anyhow::Result<GroupWriteOutcome> {
    let parsed = parse_authorities(authorities)?;
    enforce_administration_write_gate(repo, target_ref)?;

    let mut permissions = load_permissions_for_write(repo, target_ref)?;
    let group_wire = existing_group_entry_mut(&mut permissions, group)?;
    let mut changed = false;
    for authority in &parsed {
        let token = authority.name();
        if !group_wire
            .permissions
            .iter()
            .any(|existing| existing == token)
        {
            group_wire.permissions.push(token.to_owned());
            changed = true;
        }
    }

    if changed {
        write_worktree_permissions(repo, &permissions)?;
    }

    Ok(group_write_outcome(group, &parsed, None))
}

/// Add a principal to a governed group in the working-tree config.
pub fn group_add_member(
    repo: &gix::Repository,
    target_ref: &str,
    group: &str,
    member: &str,
) -> anyhow::Result<GroupWriteOutcome> {
    enforce_administration_write_gate(repo, target_ref)?;

    let mut permissions = load_permissions_for_write(repo, target_ref)?;
    let group_wire = existing_group_entry_mut(&mut permissions, group)?;
    let changed = if group_wire.members.iter().any(|existing| existing == member) {
        false
    } else {
        group_wire.members.push(member.to_owned());
        true
    };

    if changed {
        write_worktree_permissions(repo, &permissions)?;
    }

    Ok(group_write_outcome(group, &[], Some(member)))
}

/// Remove a principal from a governed group in the working-tree config.
pub fn group_remove_member(
    repo: &gix::Repository,
    target_ref: &str,
    group: &str,
    member: &str,
) -> anyhow::Result<GroupWriteOutcome> {
    enforce_administration_write_gate(repo, target_ref)?;

    let mut permissions = load_permissions_for_write(repo, target_ref)?;
    let group_wire = existing_group_entry_mut(&mut permissions, group)?;
    let before_len = group_wire.members.len();
    group_wire.members.retain(|existing| existing != member);

    if group_wire.members.len() != before_len {
        write_worktree_permissions(repo, &permissions)?;
    }

    Ok(group_write_outcome(group, &[], Some(member)))
}

/// List committed permissions plus working-tree pending grants for a principal.
pub fn perm_list(
    repo: &gix::Repository,
    target_ref: &str,
    principal: Option<&str>,
) -> anyhow::Result<PermListOutcome> {
    let config = load_governance_config(repo, target_ref)?;
    let caller = but_authz::resolve_principal_from_env(&config)?;
    let target = principal.unwrap_or_else(|| caller.id().as_str());
    let target_id = PrincipalId::new(target);

    if caller.id() != &target_id {
        let held = but_authz::effective_authority(&caller, &config);
        if !held.contains(but_authz::Authority::AdministrationRead)
            && !held.contains(but_authz::Authority::AdministrationWrite)
        {
            return Err(Denial::missing_permission(
                but_authz::Authority::AdministrationRead,
                &held,
            )
            .into());
        }
    }

    let committed = config
        .principal_authorities(&target_id)
        .cloned()
        .unwrap_or_else(AuthoritySet::empty);
    let working = read_worktree_permissions(repo)?;
    let working_direct = direct_authorities_for_principal(&working, target)?;

    let mut entries = committed
        .iter()
        .map(|authority| PermListEntry {
            authority: authority.name().to_owned(),
            marker: None,
        })
        .collect::<Vec<_>>();

    let committed_names = committed
        .iter()
        .map(Authority::name)
        .collect::<BTreeSet<_>>();
    entries.extend(
        working_direct
            .into_iter()
            .filter(|authority| !committed_names.contains(authority.name()))
            .map(|authority| PermListEntry {
                authority: authority.name().to_owned(),
                marker: Some("PENDING"),
            }),
    );

    Ok(PermListOutcome {
        principal: target.to_owned(),
        authorities: entries,
    })
}

/// Grant direct functional permissions in the working-tree governance config.
pub fn perm_grant(
    repo: &gix::Repository,
    target_ref: &str,
    principal: &str,
    authorities: &[&str],
) -> anyhow::Result<PermWriteOutcome> {
    let parsed = parse_authorities(authorities)?;
    enforce_administration_write_gate(repo, target_ref)?;

    let mut permissions = load_permissions_for_write(repo, target_ref)?;
    let principal_wire = principal_entry_mut(&mut permissions, principal)?;
    let mut changed = false;
    for authority in &parsed {
        let token = authority.name();
        if !principal_wire
            .permissions
            .iter()
            .any(|existing| existing == token)
        {
            principal_wire.permissions.push(token.to_owned());
            changed = true;
        }
    }

    if changed {
        write_worktree_permissions(repo, &permissions)?;
    }

    Ok(write_outcome(principal, &parsed))
}

/// Revoke direct functional permissions from the working-tree governance config.
pub fn perm_revoke(
    repo: &gix::Repository,
    target_ref: &str,
    principal: &str,
    authorities: &[&str],
) -> anyhow::Result<PermWriteOutcome> {
    let parsed = parse_authorities(authorities)?;
    enforce_administration_write_gate(repo, target_ref)?;

    let mut permissions = load_permissions_for_write(repo, target_ref)?;
    let Some(principal_wire) = permissions
        .principal
        .iter_mut()
        .find(|entry| entry.id == principal)
    else {
        return Ok(write_outcome(principal, &parsed));
    };

    let revoke_tokens = parsed
        .iter()
        .map(|authority| authority.name())
        .collect::<BTreeSet<_>>();
    let before_len = principal_wire.permissions.len();
    principal_wire
        .permissions
        .retain(|token| !revoke_tokens.contains(token.as_str()));

    if principal_wire.permissions.len() != before_len {
        write_worktree_permissions(repo, &permissions)?;
    }

    Ok(write_outcome(principal, &parsed))
}

fn group_write_outcome(
    group: &str,
    authorities: &[Authority],
    member: Option<&str>,
) -> GroupWriteOutcome {
    GroupWriteOutcome {
        group: group.to_owned(),
        authorities: authorities
            .iter()
            .map(|authority| authority.name().to_owned())
            .collect(),
        member: member.map(str::to_owned),
        caveat: REF_PIN_CAVEAT,
    }
}

fn group_list_entry(group: &GroupWire) -> anyhow::Result<GroupListEntry> {
    let listed = AuthoritySet::parse(group.permissions.iter().map(String::as_str))
        .with_context(|| format!("parsing authority list for group {}", group.name))?;
    let role = AuthoritySet::from_optional_role(group.role.as_deref())
        .with_context(|| format!("desugaring authority role for group {}", group.name))?;
    let authorities = listed
        .union(&role)
        .iter()
        .map(|authority| authority.name().to_owned())
        .collect();

    Ok(GroupListEntry {
        name: group.name.clone(),
        authorities,
        members: group.members.clone(),
    })
}

fn write_outcome(principal: &str, authorities: &[Authority]) -> PermWriteOutcome {
    PermWriteOutcome {
        principal: principal.to_owned(),
        authorities: authorities
            .iter()
            .map(|authority| authority.name().to_owned())
            .collect(),
        caveat: REF_PIN_CAVEAT,
    }
}

fn target_ref_from_ctx(ctx: &Context, requested: Option<&str>) -> anyhow::Result<String> {
    let target_ref = ctx.project_meta()?.target_ref_or_err()?.to_string();
    if let Some(requested) = requested
        && requested != target_ref
    {
        return Err(anyhow!(
            "requested target ref {requested} does not match workspace target {target_ref}"
        ));
    }
    Ok(target_ref)
}

fn authority_slices(authorities: &[String]) -> Vec<&str> {
    authorities.iter().map(String::as_str).collect()
}

fn parse_authorities(authorities: &[&str]) -> anyhow::Result<Vec<Authority>> {
    authorities
        .iter()
        .map(|token| Authority::parse(token))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            anyhow::Error::new(ConfigInvalid {
                code: "config.invalid",
                message: format!("invalid permission authority token {:?}: {error}", error.token()),
                remediation_hint:
                    "use a supported governance permission token such as contents:write, reviews:write, merge, or administration:write"
                        .to_owned(),
            })
        })
}

fn principal_entry_mut<'a>(
    permissions: &'a mut PermissionsWire,
    principal: &str,
) -> anyhow::Result<&'a mut PrincipalWire> {
    if let Some(position) = permissions
        .principal
        .iter()
        .position(|entry| entry.id == principal)
    {
        return permissions.principal.get_mut(position).ok_or_else(|| {
            anyhow!("principal position disappeared while preparing permissions rewrite")
        });
    }

    permissions.principal.push(PrincipalWire {
        id: principal.to_owned(),
        permissions: Vec::new(),
        role: None,
        groups: Vec::new(),
    });
    permissions
        .principal
        .last_mut()
        .ok_or_else(|| anyhow!("principal entry was not available after seeding"))
}

fn existing_group_entry_mut<'a>(
    permissions: &'a mut PermissionsWire,
    group: &str,
) -> anyhow::Result<&'a mut GroupWire> {
    let Some(position) = permissions
        .group
        .iter()
        .position(|entry| entry.name == group)
    else {
        return Err(anyhow!("undefined group {group}"));
    };

    permissions
        .group
        .get_mut(position)
        .ok_or_else(|| anyhow!("group position disappeared while preparing permissions rewrite"))
}

fn direct_authorities_for_principal(
    permissions: &PermissionsWire,
    principal: &str,
) -> anyhow::Result<Vec<Authority>> {
    let Some(entry) = permissions
        .principal
        .iter()
        .find(|entry| entry.id == principal)
    else {
        return Ok(Vec::new());
    };

    entry
        .permissions
        .iter()
        .map(|token| Authority::parse(token))
        .collect::<Result<Vec<_>, _>>()
        .context("parsing working-tree direct permission token")
}

fn load_permissions_for_write(
    repo: &gix::Repository,
    target_ref: &str,
) -> anyhow::Result<PermissionsWire> {
    let path = worktree_permissions_path(repo)?;
    if path.is_file() {
        return parse_permissions_text(
            &fs::read_to_string(&path)
                .with_context(|| format!("reading working-tree {}", permissions_path()))?,
        );
    }

    parse_permissions_text(&read_committed_permissions(repo, target_ref)?)
}

fn read_worktree_permissions(repo: &gix::Repository) -> anyhow::Result<PermissionsWire> {
    let path = worktree_permissions_path(repo)?;
    if !path.is_file() {
        return Ok(PermissionsWire::default());
    }
    parse_permissions_text(
        &fs::read_to_string(&path)
            .with_context(|| format!("reading working-tree {}", permissions_path()))?,
    )
}

fn parse_permissions_text(text: &str) -> anyhow::Result<PermissionsWire> {
    if text.trim().is_empty() {
        return Ok(PermissionsWire::default());
    }
    toml::from_str::<PermissionsWire>(text)
        .with_context(|| format!("parsing working-tree {}", permissions_path()))
}

fn write_worktree_permissions(
    repo: &gix::Repository,
    permissions: &PermissionsWire,
) -> anyhow::Result<()> {
    let path = worktree_permissions_path(repo)?;
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("{} must have a parent directory", permissions_path()))?;
    fs::create_dir_all(parent)
        .with_context(|| format!("creating working-tree {}", parent.display()))?;
    let encoded = toml::to_string(permissions)
        .with_context(|| format!("serializing working-tree {}", permissions_path()))?;
    fs::write(&path, encoded)
        .with_context(|| format!("writing working-tree {}", permissions_path()))?;
    Ok(())
}

fn worktree_permissions_path(repo: &gix::Repository) -> anyhow::Result<PathBuf> {
    let workdir = repo
        .workdir()
        .ok_or_else(|| anyhow!("governance permission writes require a non-bare repository"))?;
    Ok(workdir.join(permissions_path()))
}

fn read_committed_permissions(repo: &gix::Repository, target_ref: &str) -> anyhow::Result<String> {
    let mut reference = repo
        .find_reference(target_ref)
        .with_context(|| format!("resolving target ref {target_ref}"))?;
    let commit = reference
        .peel_to_commit()
        .with_context(|| format!("peeling {target_ref} to a commit"))?;
    let tree = commit
        .tree()
        .with_context(|| format!("reading tree for {target_ref}"))?;
    let Some(entry) = tree
        .lookup_entry_by_path(Path::new(permissions_path()))
        .with_context(|| format!("looking up {} in {target_ref}", permissions_path()))?
    else {
        return Ok(String::new());
    };
    let blob = repo
        .find_blob(entry.id())
        .with_context(|| format!("reading {} blob at {target_ref}", permissions_path()))?;
    let content = std::str::from_utf8(&blob.data)
        .with_context(|| format!("decoding {} at {target_ref} as UTF-8", permissions_path()))?;
    Ok(content.to_owned())
}

/// Extract a structured governance payload from an error chain.
pub fn classify_governance_error(err: &anyhow::Error) -> Option<GovernanceErrorPayload> {
    if let Some(denial) = err.downcast_ref::<Denial>() {
        return Some(GovernanceErrorPayload {
            code: denial.code,
            message: denial.message.clone(),
            remediation_hint: Some(denial.remediation_hint.clone()),
        });
    }

    if let Some(config_invalid) = err.downcast_ref::<ConfigInvalid>() {
        return Some(GovernanceErrorPayload {
            code: config_invalid.code,
            message: config_invalid.message.clone(),
            remediation_hint: Some(config_invalid.remediation_hint.clone()),
        });
    }

    super::config_mutate::classify_error(err).map(|gate_error| GovernanceErrorPayload {
        code: gate_error.code,
        message: gate_error.message,
        remediation_hint: None,
    })
}
