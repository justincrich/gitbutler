use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    str::FromStr as _,
};

use anyhow::{Context as _, anyhow};
use but_api_macros::but_api;
use but_authz::{
    Authority, AuthoritySet, BranchName, BranchProtection, Denial, GovConfig, Group, GroupName,
    GroupWire, PermissionsWire, PrincipalId, PrincipalWire, load_governance_config,
    permissions_path,
};
use but_core::RepositoryExt as _;
use but_ctx::Context;
use gix::{bstr::ByteSlice as _, object::tree::EntryKind};
use serde::{Deserialize, Serialize};

use crate::json::{self, ConfigInvalid};

use super::config_mutate::enforce_administration_write_gate;

/// Operator-facing caveat for working-tree governance writes.
pub const REF_PIN_CAVEAT: &str = "takes effect once committed to the target branch";

/// Result of a governance permission write.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct PermWriteOutcome {
    /// Principal whose direct permissions were changed or inspected.
    pub principal: String,
    /// Parsed authority tokens supplied by the caller.
    pub authorities: Vec<String>,
    /// Ref-pin caveat for the operator.
    pub caveat: &'static str,
}

/// Result of a governance permission grant exposed through the API boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct PermListEntry {
    /// Functional authority token.
    pub authority: String,
    /// Literal marker for a working-tree grant not committed at the target ref.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<&'static str>,
}

/// Result of listing one principal's permissions.
#[derive(Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct GroupListEntry {
    /// Group name.
    pub name: String,
    /// Functional authorities granted to the group.
    pub authorities: Vec<String>,
    /// Principals listed as group members.
    pub members: Vec<String>,
}

/// Result of listing governed groups.
#[derive(Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct GroupListOutcome {
    /// Groups from the working-tree governance config.
    pub groups: Vec<GroupListEntry>,
}

/// Repository-relative path of the working-tree branch gates file.
const GATES_PATH: &str = ".gitbutler/gates.toml";
const GOVERNANCE_COMMIT_MESSAGE: &str = "chore: update governance config";
type PrincipalAuthorityRows = Vec<(PrincipalId, AuthoritySet)>;
type GroupRows = Vec<(GroupName, Group)>;
const GOVERNANCE_COMMIT_PATHS: [&str; 2] = [GATES_PATH, ".gitbutler/permissions.toml"];

/// Caller-supplied branch protection update payload.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BranchProtectionInput {
    /// Whether the branch requires administration:write to mutate.
    pub protected: bool,
}

/// One branch gate entry returned through the API boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct BranchGateEntry {
    /// Branch name.
    pub name: String,
    /// Whether the branch is protected.
    pub protected: bool,
}

/// Result of reading or updating branch gates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct BranchGatesOutcome {
    /// Branch gate entries from the working-tree `gates.toml`.
    pub branches: Vec<BranchGateEntry>,
}

/// Read-only working-tree-vs-target-ref governance diff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GovernancePending {
    /// Per-principal effective authority comparison.
    pub principals: Vec<GovernancePendingPrincipal>,
    /// Number of authority tokens that differ between committed and working-tree config.
    pub pending_count: usize,
}

/// Read-only renderer contract for all principals present in governance config.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GovernancePrincipalsList {
    /// Principals present in committed direct principals, committed group members,
    /// working-tree direct principals, or working-tree group members.
    pub principals: Vec<GovernancePrincipalListEntry>,
}

/// Renderer row for one governance principal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GovernancePrincipalListEntry {
    /// Stable principal identifier.
    pub principal_id: String,
    /// Direct grants from the committed target-ref principal entry.
    pub own_grants: Vec<String>,
    /// Grants inherited from committed target-ref group memberships.
    pub inherited_grants: Vec<GovernanceInheritedGrant>,
    /// Group memberships from the committed target-ref config.
    pub group_memberships: Vec<String>,
    /// True only when direct working-tree principal grants differ from committed direct grants.
    pub pending: bool,
}

/// Renderer display source for a group-inherited grant.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceInheritedGrant {
    /// Functional authority token.
    pub authority: String,
    /// Human-readable inheritance source.
    pub source_label: String,
}

/// Pending authority diff for one principal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GovernancePendingPrincipal {
    /// Principal identifier.
    pub id: String,
    /// Effective authorities from the committed target-ref governance config.
    pub committed_effective: Vec<String>,
    /// Effective authorities from the working-tree governance config.
    pub working_effective: Vec<String>,
    /// Per-token comparison records.
    pub tokens: Vec<GovernancePendingToken>,
}

/// Pending status for one authority token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GovernancePendingToken {
    /// Functional authority token.
    pub authority: String,
    /// Whether the committed target-ref config grants this authority effectively.
    pub committed: bool,
    /// Whether the working-tree config grants this authority effectively.
    pub working: bool,
    /// Whether committed and working-tree effective values differ.
    pub pending: bool,
    /// Direction of the pending change.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change: Option<GovernancePendingChange>,
}

/// Result of committing pending governance config files to the target ref.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceCommitOutcome {
    /// Newly created governance commit id.
    pub commit_id: String,
    /// Fixed governance commit message used by the desktop contract.
    pub message: &'static str,
    /// Governance paths included in the commit.
    pub committed_paths: Vec<String>,
}

/// Direction of a pending authority-token change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum GovernancePendingChange {
    /// The working tree grants an authority not present at the target ref.
    Grant,
    /// The working tree removes an authority still present at the target ref.
    Revoke,
}

/// Structured governance error payload for CLI and API callers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct GovernanceErrorPayload {
    /// Stable consumer-facing error code.
    pub code: &'static str,
    /// Human-readable error message.
    pub message: String,
    /// Optional actionable recovery hint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation_hint: Option<String>,
}

#[cfg(feature = "export-schema")]
but_schemars::register_sdk_type!(PermWriteOutcome);
#[cfg(feature = "export-schema")]
but_schemars::register_sdk_type!(GrantOutcome);
#[cfg(feature = "export-schema")]
but_schemars::register_sdk_type!(GovernanceStatus);
#[cfg(feature = "export-schema")]
but_schemars::register_sdk_type!(PermListEntry);
#[cfg(feature = "export-schema")]
but_schemars::register_sdk_type!(PermListOutcome);
#[cfg(feature = "export-schema")]
but_schemars::register_sdk_type!(GroupWriteOutcome);
#[cfg(feature = "export-schema")]
but_schemars::register_sdk_type!(GroupListEntry);
#[cfg(feature = "export-schema")]
but_schemars::register_sdk_type!(GroupListOutcome);
#[cfg(feature = "export-schema")]
but_schemars::register_sdk_type!(BranchProtectionInput);
#[cfg(feature = "export-schema")]
but_schemars::register_sdk_type!(BranchGateEntry);
#[cfg(feature = "export-schema")]
but_schemars::register_sdk_type!(BranchGatesOutcome);
#[cfg(feature = "export-schema")]
but_schemars::register_sdk_type!(GovernancePending);
#[cfg(feature = "export-schema")]
but_schemars::register_sdk_type!(GovernancePrincipalsList);
#[cfg(feature = "export-schema")]
but_schemars::register_sdk_type!(GovernancePrincipalListEntry);
#[cfg(feature = "export-schema")]
but_schemars::register_sdk_type!(GovernanceInheritedGrant);
#[cfg(feature = "export-schema")]
but_schemars::register_sdk_type!(GovernancePendingPrincipal);
#[cfg(feature = "export-schema")]
but_schemars::register_sdk_type!(GovernancePendingToken);
#[cfg(feature = "export-schema")]
but_schemars::register_sdk_type!(GovernanceCommitOutcome);
#[cfg(feature = "export-schema")]
but_schemars::register_sdk_type!(GovernancePendingChange);
#[cfg(feature = "export-schema")]
but_schemars::register_sdk_type!(GovernanceErrorPayload);

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

/// List governed groups through the but-api boundary (`group_list`).
#[but_api(napi)]
pub fn group_list(ctx: &Context) -> anyhow::Result<GroupListOutcome> {
    let repo = ctx.repo.get()?;
    let target_ref = target_ref_from_ctx(ctx, None)?;
    group_list_with_repo(&repo, &target_ref)
}

/// Create a governed group through the but-api boundary (`group_create`).
#[but_api(napi)]
pub fn group_create(
    ctx: &Context,
    target_ref: String,
    group: String,
    authorities: Vec<String>,
) -> anyhow::Result<GroupWriteOutcome> {
    let repo = ctx.repo.get()?;
    let target_ref = target_ref_from_ctx(ctx, Some(&target_ref))?;
    let authorities = authority_slices(&authorities);
    group_create_with_repo(&repo, &target_ref, &group, &authorities)
}

/// Grant governed group permissions through the but-api boundary (`group_grant`).
#[but_api(napi)]
pub fn group_grant(
    ctx: &Context,
    target_ref: String,
    group: String,
    authorities: Vec<String>,
) -> anyhow::Result<GroupWriteOutcome> {
    let repo = ctx.repo.get()?;
    let target_ref = target_ref_from_ctx(ctx, Some(&target_ref))?;
    let authorities = authority_slices(&authorities);
    group_grant_with_repo(&repo, &target_ref, &group, &authorities)
}

/// Revoke governed group permissions through the but-api boundary (`group_revoke`).
#[but_api(napi)]
pub fn group_revoke(
    ctx: &Context,
    target_ref: String,
    group: String,
    authorities: Vec<String>,
) -> anyhow::Result<GroupWriteOutcome> {
    let repo = ctx.repo.get()?;
    let target_ref = target_ref_from_ctx(ctx, Some(&target_ref))?;
    let authorities = authority_slices(&authorities);
    group_revoke_with_repo(&repo, &target_ref, &group, &authorities)
}

/// Add a principal to a governed group through the but-api boundary (`group_add_member`).
#[but_api(napi)]
pub fn group_add_member(
    ctx: &Context,
    target_ref: String,
    group: String,
    member: String,
) -> anyhow::Result<GroupWriteOutcome> {
    let repo = ctx.repo.get()?;
    let target_ref = target_ref_from_ctx(ctx, Some(&target_ref))?;
    group_add_member_with_repo(&repo, &target_ref, &group, &member)
}

/// Remove a principal from a governed group through the but-api boundary (`group_remove_member`).
#[but_api(napi)]
pub fn group_remove_member(
    ctx: &Context,
    target_ref: String,
    group: String,
    member: String,
) -> anyhow::Result<GroupWriteOutcome> {
    let repo = ctx.repo.get()?;
    let target_ref = target_ref_from_ctx(ctx, Some(&target_ref))?;
    group_remove_member_with_repo(&repo, &target_ref, &group, &member)
}

/// Delete a governed group through the but-api boundary (`group_delete`).
#[but_api(napi)]
pub fn group_delete(
    ctx: &Context,
    target_ref: String,
    group: String,
) -> anyhow::Result<GroupWriteOutcome> {
    let repo = ctx.repo.get()?;
    let target_ref = target_ref_from_ctx(ctx, Some(&target_ref))?;
    group_delete_with_repo(&repo, &target_ref, &group)
}

/// List governed direct permissions through the but-api boundary (`perm_list`).
#[but_api(napi)]
pub fn perm_list(ctx: &Context, principal: Option<String>) -> anyhow::Result<PermListOutcome> {
    let repo = ctx.repo.get()?;
    let target_ref = target_ref_from_ctx(ctx, None)?;
    perm_list_with_repo(&repo, &target_ref, principal.as_deref())
}

/// Grant governed direct permissions through the but-api boundary (`perm_grant`).
#[but_api(napi)]
pub fn perm_grant(
    ctx: &Context,
    target_ref: String,
    principal: String,
    authorities: Vec<String>,
) -> anyhow::Result<GrantOutcome> {
    let repo = ctx.repo.get()?;
    let target_ref = target_ref_from_ctx(ctx, Some(&target_ref))?;
    let authorities = authority_slices(&authorities);
    Ok(perm_grant_with_repo(&repo, &target_ref, &principal, &authorities)?.into())
}

/// Revoke governed direct permissions through the but-api boundary (`perm_revoke`).
#[but_api(napi)]
pub fn perm_revoke(
    ctx: &Context,
    target_ref: String,
    principal: String,
    authorities: Vec<String>,
) -> anyhow::Result<PermWriteOutcome> {
    let repo = ctx.repo.get()?;
    let target_ref = target_ref_from_ctx(ctx, Some(&target_ref))?;
    let authorities = authority_slices(&authorities);
    perm_revoke_with_repo(&repo, &target_ref, &principal, &authorities)
}

/// Return the caller's own effective governance authorities (`governance_status_read`).
#[but_api(napi, GovernanceStatus)]
pub fn governance_status_read(ctx: &Context) -> anyhow::Result<AuthoritySet> {
    let repo = ctx.repo.get()?;
    let target_ref = target_ref_from_ctx(ctx, None)?;
    let config = load_governance_config(&repo, &target_ref)?;
    let caller = but_authz::resolve_principal_from_env(&config)?;
    Ok(but_authz::effective_authority(&caller, &config))
}

/// Read branch gates (`branch_gates_read`) for the target ref through the but-api boundary.
#[but_api(napi)]
pub fn branch_gates_read(ctx: &Context, target_ref: String) -> anyhow::Result<BranchGatesOutcome> {
    let repo = ctx.repo.get()?;
    let target_ref = target_ref_from_ctx(ctx, Some(&target_ref))?;
    branch_gates_read_with_repo(&repo, &target_ref)
}

/// Update one branch gate entry (`branch_gates_update`) through the but-api boundary.
#[but_api(napi)]
pub fn branch_gates_update(
    ctx: &Context,
    target_ref: String,
    branch: String,
    protection: BranchProtectionInput,
) -> anyhow::Result<BranchGatesOutcome> {
    let repo = ctx.repo.get()?;
    let target_ref = target_ref_from_ctx(ctx, Some(&target_ref))?;
    branch_gates_update_with_repo(&repo, &target_ref, &branch, protection)
}

/// Read the working-tree-vs-target-ref pending governance diff (`governance_pending`).
#[but_api(napi)]
pub fn governance_pending(ctx: &Context, target_ref: String) -> anyhow::Result<GovernancePending> {
    let repo = ctx.repo.get()?;
    let target_ref = target_ref_from_ctx(ctx, Some(&target_ref))?;
    governance_pending_with_repo(&repo, &target_ref)
}

/// List all principals needed by the governance renderer (`governance_principals_list`).
#[but_api(napi)]
pub fn governance_principals_list(
    ctx: &Context,
    target_ref: String,
) -> anyhow::Result<GovernancePrincipalsList> {
    let repo = ctx.repo.get()?;
    let target_ref = target_ref_from_ctx(ctx, Some(&target_ref))?;
    governance_principals_list_with_repo(&repo, &target_ref)
}

/// Commit pending governance config files to the target ref (`governance_commit`).
#[but_api(napi)]
pub fn governance_commit(
    ctx: &Context,
    target_ref: String,
) -> anyhow::Result<GovernanceCommitOutcome> {
    let repo = ctx.repo.get()?;
    let target_ref = target_ref_from_ctx(ctx, Some(&target_ref))?;
    governance_commit_with_repo(&repo, &target_ref)
}

/// Read branch gates under administration-read authority.
pub fn branch_gates_read_with_repo(
    repo: &gix::Repository,
    target_ref: &str,
) -> anyhow::Result<BranchGatesOutcome> {
    let config = load_governance_config(repo, target_ref)?;
    let caller = but_authz::resolve_principal_from_env(&config)?;
    let held = but_authz::effective_authority(&caller, &config);
    if !held.contains(Authority::AdministrationRead)
        && !held.contains(Authority::AdministrationWrite)
    {
        return Err(Denial::missing_permission(Authority::AdministrationRead, &held).into());
    }

    let gates = load_gates_for_write(repo, target_ref)?;
    let branches = gates
        .branch
        .into_iter()
        .map(BranchGateEntry::from)
        .collect();
    Ok(BranchGatesOutcome { branches })
}

/// Update one branch gate entry under administration-write authority.
pub fn branch_gates_update_with_repo(
    repo: &gix::Repository,
    target_ref: &str,
    branch: &str,
    protection: BranchProtectionInput,
) -> anyhow::Result<BranchGatesOutcome> {
    enforce_administration_write_gate(repo, target_ref)?;
    branch_gates_update_authorized(repo, target_ref, branch, protection)
}

/// Update one branch gate entry after the desktop fleet-owner boundary has
/// asserted unconditional administration-write authority.
pub fn branch_gates_update_with_repo_as_fleet_owner(
    repo: &gix::Repository,
    target_ref: &str,
    branch: &str,
    protection: BranchProtectionInput,
) -> anyhow::Result<BranchGatesOutcome> {
    branch_gates_update_authorized(repo, target_ref, branch, protection)
}

fn branch_gates_update_authorized(
    repo: &gix::Repository,
    target_ref: &str,
    branch: &str,
    protection: BranchProtectionInput,
) -> anyhow::Result<BranchGatesOutcome> {
    let mut gates = load_gates_for_write(repo, target_ref)?;
    if let Some(existing) = gates.branch.iter_mut().find(|entry| entry.name == branch) {
        existing.protected = protection.protected;
    } else {
        gates.branch.push(GatesBranchWire {
            name: branch.to_owned(),
            protected: protection.protected,
        });
    }
    write_worktree_gates(repo, &gates)?;

    let branches = gates
        .branch
        .into_iter()
        .map(BranchGateEntry::from)
        .collect();
    Ok(BranchGatesOutcome { branches })
}

/// Commit only governance config files from the worktree onto the target ref.
pub fn governance_commit_with_repo(
    repo: &gix::Repository,
    target_ref: &str,
) -> anyhow::Result<GovernanceCommitOutcome> {
    enforce_administration_write_gate(repo, target_ref)?;
    governance_commit_authorized(repo, target_ref)
}

/// Commit governance config files after the desktop fleet-owner boundary has
/// asserted unconditional administration-write authority.
pub fn governance_commit_with_repo_as_fleet_owner(
    repo: &gix::Repository,
    target_ref: &str,
) -> anyhow::Result<GovernanceCommitOutcome> {
    governance_commit_authorized(repo, target_ref)
}

fn governance_commit_authorized(
    repo: &gix::Repository,
    target_ref: &str,
) -> anyhow::Result<GovernanceCommitOutcome> {
    load_worktree_governance_config(repo)?;
    let parent_id = ref_id(repo, target_ref)?;
    let parent = repo.find_commit(parent_id)?;
    let parent_tree_id = parent.tree_id()?.detach();
    let mut tree = repo.edit_tree(parent_tree_id)?;
    let committed_paths = upsert_changed_governance_paths(repo, parent_tree_id, &mut tree)?;
    if committed_paths.is_empty() {
        return Err(anyhow!("no pending governance config changes to commit"));
    }

    let tree_id = tree.write()?.detach();
    let (author, committer) = repo.commit_signatures()?;
    let update_ref = gitbutler_reference::Refname::from_str(target_ref)
        .with_context(|| format!("parsing target ref {target_ref}"))?;
    let commit_id = gitbutler_repo::commit_with_signature_gix(
        repo,
        Some(&update_ref),
        author,
        committer,
        GOVERNANCE_COMMIT_MESSAGE.as_bytes().as_bstr(),
        tree_id,
        &[parent_id],
        None,
    )?;

    Ok(GovernanceCommitOutcome {
        commit_id: commit_id.to_string(),
        message: GOVERNANCE_COMMIT_MESSAGE,
        committed_paths,
    })
}

fn upsert_changed_governance_paths(
    repo: &gix::Repository,
    parent_tree_id: gix::ObjectId,
    tree: &mut gix::object::tree::Editor<'_>,
) -> anyhow::Result<Vec<String>> {
    let mut committed_paths = Vec::new();
    for path in GOVERNANCE_COMMIT_PATHS {
        let worktree = read_worktree_governance_file(repo, path)?;
        if committed_file_bytes(repo, parent_tree_id, path)?.as_deref() == Some(worktree.as_slice())
        {
            continue;
        }
        let blob_id = repo.write_blob(&worktree)?;
        tree.upsert(path.as_bytes().as_bstr(), EntryKind::Blob, blob_id)?;
        committed_paths.push(path.to_owned());
    }
    Ok(committed_paths)
}

fn read_worktree_governance_file(repo: &gix::Repository, path: &str) -> anyhow::Result<Vec<u8>> {
    let workdir = repo
        .workdir()
        .ok_or_else(|| anyhow!("governance commits require a non-bare repository"))?;
    fs::read(workdir.join(path)).with_context(|| format!("reading working-tree {path}"))
}

fn ref_id(repo: &gix::Repository, ref_name: &str) -> anyhow::Result<gix::ObjectId> {
    let mut reference = repo
        .find_reference(ref_name)
        .with_context(|| format!("resolving target ref {ref_name}"))?;
    Ok(reference
        .peel_to_commit()
        .with_context(|| format!("peeling {ref_name} to a commit"))?
        .id)
}

fn committed_file_bytes(
    repo: &gix::Repository,
    tree_id: gix::ObjectId,
    path: &str,
) -> anyhow::Result<Option<Vec<u8>>> {
    let tree = repo.find_tree(tree_id)?;
    let Some(entry) = tree.lookup_entry_by_path(gix::path::from_bstr(path.as_bytes().as_bstr()))?
    else {
        return Ok(None);
    };
    Ok(Some(entry.object()?.into_blob().data.clone()))
}

/// Return the read-only pending governance authority diff for a repository.
pub fn governance_pending_with_repo(
    repo: &gix::Repository,
    target_ref: &str,
) -> anyhow::Result<GovernancePending> {
    let committed = load_governance_config(repo, target_ref)?;
    let working = load_worktree_governance_config(repo)?;
    Ok(pending_diff(&committed, &working))
}

/// Return all governance principals needed by the renderer.
pub fn governance_principals_list_with_repo(
    repo: &gix::Repository,
    target_ref: &str,
) -> anyhow::Result<GovernancePrincipalsList> {
    let committed = read_committed_permissions_for_list(repo, target_ref)?;
    let working = read_worktree_permissions_for_pending(repo)?;
    principals_list(committed, working)
}

fn principals_list(
    committed: PermissionsWire,
    working: PermissionsWire,
) -> anyhow::Result<GovernancePrincipalsList> {
    let committed_view = permissions_view(committed)?;
    let working_view = permissions_view(working)?;
    let principal_ids = committed_view
        .principal_ids()
        .chain(working_view.principal_ids())
        .collect::<BTreeSet<_>>();

    let principals = principal_ids
        .into_iter()
        .map(|principal_id| {
            let own_grants = committed_view.direct_grants(&principal_id);
            let inherited_grants = committed_view.inherited_grants(&principal_id);
            let group_memberships = committed_view.group_memberships(&principal_id);
            let pending =
                committed_view.direct_set(&principal_id) != working_view.direct_set(&principal_id);

            GovernancePrincipalListEntry {
                principal_id,
                own_grants,
                inherited_grants,
                group_memberships,
                pending,
            }
        })
        .collect();

    Ok(GovernancePrincipalsList { principals })
}

fn pending_diff(committed: &GovConfig, working: &GovConfig) -> GovernancePending {
    let principal_ids = committed
        .principals()
        .keys()
        .chain(working.principals().keys())
        .map(PrincipalId::as_str)
        .collect::<BTreeSet<_>>();

    let mut pending_count = 0;
    let principals = principal_ids
        .into_iter()
        .map(|id| {
            let principal_id = PrincipalId::new(id);
            let committed_set = committed
                .principal_authorities(&principal_id)
                .cloned()
                .unwrap_or_else(AuthoritySet::empty);
            let working_set = working
                .principal_authorities(&principal_id)
                .cloned()
                .unwrap_or_else(AuthoritySet::empty);
            let committed_effective = authority_names(&committed_set);
            let working_effective = authority_names(&working_set);
            let tokens = Authority::ALL
                .iter()
                .copied()
                .filter_map(|authority| {
                    let committed = committed_set.contains(authority);
                    let working = working_set.contains(authority);
                    if !committed && !working {
                        return None;
                    }
                    let change = match (committed, working) {
                        (false, true) => Some(GovernancePendingChange::Grant),
                        (true, false) => Some(GovernancePendingChange::Revoke),
                        _ => None,
                    };
                    if change.is_some() {
                        pending_count += 1;
                    }
                    Some(GovernancePendingToken {
                        authority: authority.name().to_owned(),
                        committed,
                        working,
                        pending: change.is_some(),
                        change,
                    })
                })
                .collect();
            GovernancePendingPrincipal {
                id: id.to_owned(),
                committed_effective,
                working_effective,
                tokens,
            }
        })
        .collect();

    GovernancePending {
        principals,
        pending_count,
    }
}

#[derive(Debug, Clone)]
struct PermissionsView {
    direct_grants: BTreeMap<String, AuthoritySet>,
    group_memberships: BTreeMap<String, BTreeSet<String>>,
    group_grants: BTreeMap<String, AuthoritySet>,
}

impl PermissionsView {
    fn principal_ids(&self) -> impl Iterator<Item = String> + '_ {
        self.direct_grants
            .keys()
            .chain(self.group_memberships.keys())
            .cloned()
    }

    fn direct_set(&self, principal_id: &str) -> AuthoritySet {
        self.direct_grants
            .get(principal_id)
            .cloned()
            .unwrap_or_else(AuthoritySet::empty)
    }

    fn direct_grants(&self, principal_id: &str) -> Vec<String> {
        self.direct_grants
            .get(principal_id)
            .map_or_else(Vec::new, authority_names)
    }

    fn group_memberships(&self, principal_id: &str) -> Vec<String> {
        self.group_memberships
            .get(principal_id)
            .map(|groups| groups.iter().cloned().collect())
            .unwrap_or_default()
    }

    fn inherited_grants(&self, principal_id: &str) -> Vec<GovernanceInheritedGrant> {
        let Some(groups) = self.group_memberships.get(principal_id) else {
            return Vec::new();
        };
        groups
            .iter()
            .flat_map(|group| {
                self.group_grants
                    .get(group)
                    .map_or_else(Vec::new, |authorities| {
                        authorities
                            .iter()
                            .map(|authority| GovernanceInheritedGrant {
                                authority: authority.name().to_owned(),
                                source_label: format!("group: {group}"),
                            })
                            .collect()
                    })
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}

fn permissions_view(permissions: PermissionsWire) -> anyhow::Result<PermissionsView> {
    let mut direct_grants = BTreeMap::new();
    let mut group_memberships: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut group_grants = BTreeMap::new();

    for group in &permissions.group {
        let grants = authority_set_for_pending(
            &group.permissions,
            group.role.as_deref(),
            &format!("group {}", group.name),
        )?;
        if group_grants.insert(group.name.clone(), grants).is_some() {
            return Err(config_invalid(format!("duplicate group {}", group.name)));
        }
        for member in &group.members {
            group_memberships
                .entry(member.clone())
                .or_default()
                .insert(group.name.clone());
        }
    }

    for principal in &permissions.principal {
        let grants = authority_set_for_pending(
            &principal.permissions,
            principal.role.as_deref(),
            &format!("principal {}", principal.id),
        )?;
        if direct_grants.insert(principal.id.clone(), grants).is_some() {
            return Err(config_invalid(format!(
                "duplicate principal {}",
                principal.id
            )));
        }
        for group in &principal.groups {
            if !group_grants.contains_key(group) {
                return Err(config_invalid(format!(
                    "principal {} references undefined group {group}",
                    principal.id
                )));
            }
            group_memberships
                .entry(principal.id.clone())
                .or_default()
                .insert(group.clone());
        }
    }

    Ok(PermissionsView {
        direct_grants,
        group_memberships,
        group_grants,
    })
}

fn authority_names(authorities: &AuthoritySet) -> Vec<String> {
    authorities
        .iter()
        .map(|authority| authority.name().to_owned())
        .collect()
}

fn read_committed_permissions_for_list(
    repo: &gix::Repository,
    target_ref: &str,
) -> anyhow::Result<PermissionsWire> {
    let permissions_blob = read_committed_permissions_blob(repo, target_ref)?;
    toml::from_str::<PermissionsWire>(&permissions_blob).map_err(|error| {
        anyhow::Error::new(json::ConfigInvalid {
            code: "config.invalid",
            message: format!(
                "malformed target-ref {} at {target_ref}: {error}",
                permissions_path()
            ),
            remediation_hint:
                "fix the malformed governance config and recommit it to the target branch"
                    .to_owned(),
        })
    })
}

fn read_committed_permissions_blob(
    repo: &gix::Repository,
    target_ref: &str,
) -> anyhow::Result<String> {
    let mut reference = repo
        .find_reference(target_ref)
        .with_context(|| format!("resolving target ref {target_ref}"))?;
    let commit = reference
        .peel_to_commit()
        .with_context(|| format!("peeling {target_ref} to a commit"))?;
    let tree = commit
        .tree()
        .with_context(|| format!("reading tree for {target_ref}"))?;
    let entry = tree
        .lookup_entry_by_path(Path::new(permissions_path()))
        .with_context(|| format!("looking up {} in {target_ref}", permissions_path()))?
        .ok_or_else(|| anyhow!("missing {} at {target_ref}", permissions_path()))?;
    let blob = repo
        .find_blob(entry.id())
        .with_context(|| format!("reading {} blob at {target_ref}", permissions_path()))?;
    let content = std::str::from_utf8(&blob.data)
        .with_context(|| format!("decoding {} at {target_ref} as UTF-8", permissions_path()))?;
    Ok(content.to_owned())
}

fn load_worktree_governance_config(repo: &gix::Repository) -> anyhow::Result<GovConfig> {
    let permissions = read_worktree_permissions_for_pending(repo)?;
    let gates = read_worktree_gates_for_pending(repo)?;
    let (principals, groups) = normalize_pending_permissions(permissions)?;
    let branches = gates
        .branch
        .into_iter()
        .map(|branch| {
            (
                BranchName::new(branch.name),
                BranchProtection::new(branch.protected),
            )
        })
        .collect::<Vec<_>>();
    Ok(GovConfig::new(principals, groups, branches))
}

fn read_worktree_permissions_for_pending(
    repo: &gix::Repository,
) -> anyhow::Result<PermissionsWire> {
    let path = worktree_permissions_path(repo)?;
    let text = fs::read_to_string(&path).map_err(|error| {
        config_invalid(format!(
            "reading working-tree {} failed: {error}",
            permissions_path()
        ))
    })?;
    toml::from_str::<PermissionsWire>(&text).map_err(|error| {
        anyhow::Error::new(json::ConfigInvalid {
            code: "config.invalid",
            message: format!("malformed working-tree {}: {error}", permissions_path()),
            remediation_hint:
                "fix the malformed governance config and recommit it to the target branch"
                    .to_owned(),
        })
    })
}

fn read_worktree_gates_for_pending(repo: &gix::Repository) -> anyhow::Result<GatesFile> {
    let path = worktree_gates_path(repo)?;
    let text = fs::read_to_string(&path).map_err(|error| {
        config_invalid(format!("reading working-tree {GATES_PATH} failed: {error}"))
    })?;
    toml::from_str::<GatesFile>(&text).map_err(|error| {
        anyhow::Error::new(json::ConfigInvalid {
            code: "config.invalid",
            message: format!("malformed working-tree {GATES_PATH}: {error}"),
            remediation_hint:
                "fix the malformed governance config and recommit it to the target branch"
                    .to_owned(),
        })
    })
}

fn normalize_pending_permissions(
    permissions: PermissionsWire,
) -> anyhow::Result<(PrincipalAuthorityRows, GroupRows)> {
    let mut group_authorities = BTreeMap::new();
    let mut groups = BTreeMap::new();

    for group in &permissions.group {
        let name = GroupName::new(group.name.clone());
        let authorities = authority_set_for_pending(
            &group.permissions,
            group.role.as_deref(),
            &format!("group {}", group.name),
        )?;
        let parsed = Group::new(
            name.clone(),
            authorities.clone(),
            group.members.iter().cloned().map(PrincipalId::new),
        );
        if groups.insert(name.clone(), parsed).is_some() {
            return Err(config_invalid(format!("duplicate group {}", group.name)));
        }
        group_authorities.insert(name, authorities);
    }

    let mut principals = BTreeMap::new();
    for principal in &permissions.principal {
        let mut authorities = authority_set_for_pending(
            &principal.permissions,
            principal.role.as_deref(),
            &format!("principal {}", principal.id),
        )?;
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
            return Err(config_invalid(format!(
                "duplicate principal {}",
                principal.id
            )));
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

    Ok((
        principals.into_iter().collect(),
        groups.into_iter().collect(),
    ))
}

fn authority_set_for_pending(
    permissions: &[String],
    role: Option<&str>,
    subject: &str,
) -> anyhow::Result<AuthoritySet> {
    let listed = AuthoritySet::parse(permissions.iter().map(String::as_str)).map_err(|error| {
        config_invalid(format!(
            "parsing authority list for {subject}: unknown token {}",
            error.token()
        ))
    })?;
    let role_set = AuthoritySet::from_optional_role(role).map_err(|error| {
        config_invalid(format!(
            "desugaring authority role for {subject}: unknown role {}",
            error.token()
        ))
    })?;
    Ok(listed.union(&role_set))
}

/// List governed groups under administration-read authority.
pub fn group_list_with_repo(
    repo: &gix::Repository,
    target_ref: &str,
) -> anyhow::Result<GroupListOutcome> {
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
pub fn group_create_with_repo(
    repo: &gix::Repository,
    target_ref: &str,
    group: &str,
    authorities: &[&str],
) -> anyhow::Result<GroupWriteOutcome> {
    let parsed = parse_authorities(authorities)?;
    enforce_administration_write_gate(repo, target_ref)?;
    group_create_authorized(repo, target_ref, group, &parsed)
}

/// Create a governed group after the desktop fleet-owner boundary has asserted
/// unconditional administration-write authority.
pub fn group_create_with_repo_as_fleet_owner(
    repo: &gix::Repository,
    target_ref: &str,
    group: &str,
    authorities: &[&str],
) -> anyhow::Result<GroupWriteOutcome> {
    let parsed = parse_authorities(authorities)?;
    group_create_authorized(repo, target_ref, group, &parsed)
}

fn group_create_authorized(
    repo: &gix::Repository,
    target_ref: &str,
    group: &str,
    parsed: &[Authority],
) -> anyhow::Result<GroupWriteOutcome> {
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

    Ok(group_write_outcome(group, parsed, None))
}

/// Grant functional permissions to a governed group in the working-tree config.
pub fn group_grant_with_repo(
    repo: &gix::Repository,
    target_ref: &str,
    group: &str,
    authorities: &[&str],
) -> anyhow::Result<GroupWriteOutcome> {
    let parsed = parse_authorities(authorities)?;
    enforce_administration_write_gate(repo, target_ref)?;
    group_grant_authorized(repo, target_ref, group, &parsed)
}

/// Grant functional permissions to a governed group after the desktop
/// fleet-owner boundary has asserted unconditional administration-write
/// authority.
pub fn group_grant_with_repo_as_fleet_owner(
    repo: &gix::Repository,
    target_ref: &str,
    group: &str,
    authorities: &[&str],
) -> anyhow::Result<GroupWriteOutcome> {
    let parsed = parse_authorities(authorities)?;
    group_grant_authorized(repo, target_ref, group, &parsed)
}

fn group_grant_authorized(
    repo: &gix::Repository,
    target_ref: &str,
    group: &str,
    parsed: &[Authority],
) -> anyhow::Result<GroupWriteOutcome> {
    let mut permissions = load_permissions_for_write(repo, target_ref)?;
    let group_wire = existing_group_entry_mut(&mut permissions, group)?;
    let mut changed = false;
    for authority in parsed {
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

    Ok(group_write_outcome(group, parsed, None))
}

/// Revoke functional permissions from a governed group in the working-tree config.
pub fn group_revoke_with_repo(
    repo: &gix::Repository,
    target_ref: &str,
    group: &str,
    authorities: &[&str],
) -> anyhow::Result<GroupWriteOutcome> {
    let parsed = parse_authorities(authorities)?;
    enforce_administration_write_gate(repo, target_ref)?;
    group_revoke_authorized(repo, target_ref, group, &parsed)
}

/// Revoke functional permissions from a governed group after the desktop
/// fleet-owner boundary has asserted unconditional administration-write
/// authority.
pub fn group_revoke_with_repo_as_fleet_owner(
    repo: &gix::Repository,
    target_ref: &str,
    group: &str,
    authorities: &[&str],
) -> anyhow::Result<GroupWriteOutcome> {
    let parsed = parse_authorities(authorities)?;
    group_revoke_authorized(repo, target_ref, group, &parsed)
}

fn group_revoke_authorized(
    repo: &gix::Repository,
    target_ref: &str,
    group: &str,
    parsed: &[Authority],
) -> anyhow::Result<GroupWriteOutcome> {
    let mut permissions = load_permissions_for_write(repo, target_ref)?;
    let group_wire = existing_group_entry_mut(&mut permissions, group)?;
    let revoke_tokens = parsed
        .iter()
        .map(|authority| authority.name())
        .collect::<BTreeSet<_>>();
    let before_len = group_wire.permissions.len();
    group_wire
        .permissions
        .retain(|token| !revoke_tokens.contains(token.as_str()));

    if group_wire.permissions.len() != before_len {
        write_worktree_permissions(repo, &permissions)?;
    }

    Ok(group_write_outcome(group, parsed, None))
}

/// Add a principal to a governed group in the working-tree config.
pub fn group_add_member_with_repo(
    repo: &gix::Repository,
    target_ref: &str,
    group: &str,
    member: &str,
) -> anyhow::Result<GroupWriteOutcome> {
    enforce_administration_write_gate(repo, target_ref)?;
    group_add_member_authorized(repo, target_ref, group, member)
}

/// Add a principal to a governed group after the desktop fleet-owner boundary
/// has asserted unconditional administration-write authority.
pub fn group_add_member_with_repo_as_fleet_owner(
    repo: &gix::Repository,
    target_ref: &str,
    group: &str,
    member: &str,
) -> anyhow::Result<GroupWriteOutcome> {
    group_add_member_authorized(repo, target_ref, group, member)
}

fn group_add_member_authorized(
    repo: &gix::Repository,
    target_ref: &str,
    group: &str,
    member: &str,
) -> anyhow::Result<GroupWriteOutcome> {
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
pub fn group_remove_member_with_repo(
    repo: &gix::Repository,
    target_ref: &str,
    group: &str,
    member: &str,
) -> anyhow::Result<GroupWriteOutcome> {
    enforce_administration_write_gate(repo, target_ref)?;
    group_remove_member_authorized(repo, target_ref, group, member)
}

/// Remove a principal from a governed group after the desktop fleet-owner
/// boundary has asserted unconditional administration-write authority.
pub fn group_remove_member_with_repo_as_fleet_owner(
    repo: &gix::Repository,
    target_ref: &str,
    group: &str,
    member: &str,
) -> anyhow::Result<GroupWriteOutcome> {
    group_remove_member_authorized(repo, target_ref, group, member)
}

fn group_remove_member_authorized(
    repo: &gix::Repository,
    target_ref: &str,
    group: &str,
    member: &str,
) -> anyhow::Result<GroupWriteOutcome> {
    let mut permissions = load_permissions_for_write(repo, target_ref)?;
    let group_wire = existing_group_entry_mut(&mut permissions, group)?;
    let before_len = group_wire.members.len();
    group_wire.members.retain(|existing| existing != member);

    if group_wire.members.len() != before_len {
        write_worktree_permissions(repo, &permissions)?;
    }

    Ok(group_write_outcome(group, &[], Some(member)))
}

/// Delete a governed group from the working-tree governance config.
pub fn group_delete_with_repo(
    repo: &gix::Repository,
    target_ref: &str,
    group: &str,
) -> anyhow::Result<GroupWriteOutcome> {
    enforce_administration_write_gate(repo, target_ref)?;
    group_delete_authorized(repo, target_ref, group)
}

/// Delete a governed group after the desktop fleet-owner boundary has asserted
/// unconditional administration-write authority.
pub fn group_delete_with_repo_as_fleet_owner(
    repo: &gix::Repository,
    target_ref: &str,
    group: &str,
) -> anyhow::Result<GroupWriteOutcome> {
    group_delete_authorized(repo, target_ref, group)
}

fn group_delete_authorized(
    repo: &gix::Repository,
    target_ref: &str,
    group: &str,
) -> anyhow::Result<GroupWriteOutcome> {
    let mut permissions = load_permissions_for_write(repo, target_ref)?;
    let before_len = permissions.group.len();
    permissions.group.retain(|entry| entry.name != group);

    if permissions.group.len() != before_len {
        write_worktree_permissions(repo, &permissions)?;
    }

    Ok(group_write_outcome(group, &[], None))
}

/// List committed permissions plus working-tree pending grants for a principal.
pub fn perm_list_with_repo(
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
pub fn perm_grant_with_repo(
    repo: &gix::Repository,
    target_ref: &str,
    principal: &str,
    authorities: &[&str],
) -> anyhow::Result<PermWriteOutcome> {
    let parsed = parse_authorities(authorities)?;
    enforce_administration_write_gate(repo, target_ref)?;
    perm_grant_with_parsed_authorities(repo, target_ref, principal, &parsed)
}

/// Grant direct permissions after the desktop command boundary has resolved the
/// signed-in fleet-owner and asserted the v1 administration-write exception.
///
/// This is intentionally not used by the agent/env path, which must continue to
/// call [`perm_grant_with_repo`] and resolve `BUT_AGENT_HANDLE` through
/// `but-authz`.
pub fn perm_grant_with_repo_as_fleet_owner(
    repo: &gix::Repository,
    target_ref: &str,
    principal: &str,
    authorities: &[&str],
) -> anyhow::Result<PermWriteOutcome> {
    let parsed = parse_authorities(authorities)?;
    perm_grant_with_parsed_authorities(repo, target_ref, principal, &parsed)
}

fn perm_grant_with_parsed_authorities(
    repo: &gix::Repository,
    target_ref: &str,
    principal: &str,
    parsed: &[Authority],
) -> anyhow::Result<PermWriteOutcome> {
    let mut permissions = load_permissions_for_write(repo, target_ref)?;
    let principal_wire = principal_entry_mut(&mut permissions, principal)?;
    let mut changed = false;
    for authority in parsed {
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

    Ok(write_outcome(principal, parsed))
}

/// Revoke direct functional permissions from the working-tree governance config.
pub fn perm_revoke_with_repo(
    repo: &gix::Repository,
    target_ref: &str,
    principal: &str,
    authorities: &[&str],
) -> anyhow::Result<PermWriteOutcome> {
    let parsed = parse_authorities(authorities)?;
    enforce_administration_write_gate(repo, target_ref)?;
    perm_revoke_authorized(repo, target_ref, principal, &parsed)
}

/// Revoke direct permissions after the desktop fleet-owner boundary has
/// asserted unconditional administration-write authority.
pub fn perm_revoke_with_repo_as_fleet_owner(
    repo: &gix::Repository,
    target_ref: &str,
    principal: &str,
    authorities: &[&str],
) -> anyhow::Result<PermWriteOutcome> {
    let parsed = parse_authorities(authorities)?;
    perm_revoke_authorized(repo, target_ref, principal, &parsed)
}

fn perm_revoke_authorized(
    repo: &gix::Repository,
    target_ref: &str,
    principal: &str,
    parsed: &[Authority],
) -> anyhow::Result<PermWriteOutcome> {
    let mut permissions = load_permissions_for_write(repo, target_ref)?;
    let Some(principal_wire) = permissions
        .principal
        .iter_mut()
        .find(|entry| entry.id == principal)
    else {
        return Ok(write_outcome(principal, parsed));
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

    Ok(write_outcome(principal, parsed))
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

fn config_invalid(message: String) -> anyhow::Error {
    anyhow::Error::new(json::ConfigInvalid {
        code: "config.invalid",
        message,
        remediation_hint:
            "fix the malformed governance config and recommit it to the target branch".to_owned(),
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

/// Working-tree `gates.toml` wire format.
///
/// Mirrors the `[[branch]]` shape parsed by `but_authz` so round-tripping the
/// file preserves the on-disk layout exactly.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct GatesFile {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    branch: Vec<GatesBranchWire>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct GatesBranchWire {
    name: String,
    protected: bool,
}

impl From<GatesBranchWire> for BranchGateEntry {
    fn from(branch: GatesBranchWire) -> Self {
        Self {
            name: branch.name,
            protected: branch.protected,
        }
    }
}

fn load_gates_for_write(repo: &gix::Repository, target_ref: &str) -> anyhow::Result<GatesFile> {
    if let Ok(worktree) = read_worktree_gates(repo) {
        return Ok(worktree);
    }
    let committed = read_committed_gates(repo, target_ref)?;
    parse_gates_text(&committed)
}

fn read_worktree_gates(repo: &gix::Repository) -> anyhow::Result<GatesFile> {
    let path = worktree_gates_path(repo)?;
    if !path.is_file() {
        return Ok(GatesFile::default());
    }
    let text =
        fs::read_to_string(&path).with_context(|| format!("reading working-tree {GATES_PATH}"))?;
    parse_gates_text(&text)
}

fn parse_gates_text(text: &str) -> anyhow::Result<GatesFile> {
    if text.trim().is_empty() {
        return Ok(GatesFile::default());
    }
    toml::from_str::<GatesFile>(text).with_context(|| format!("parsing working-tree {GATES_PATH}"))
}

fn write_worktree_gates(repo: &gix::Repository, gates: &GatesFile) -> anyhow::Result<()> {
    let path = worktree_gates_path(repo)?;
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("{GATES_PATH} must have a parent directory"))?;
    fs::create_dir_all(parent)
        .with_context(|| format!("creating working-tree {}", parent.display()))?;
    let encoded =
        toml::to_string(gates).with_context(|| format!("serializing working-tree {GATES_PATH}"))?;
    fs::write(&path, encoded).with_context(|| format!("writing working-tree {GATES_PATH}"))?;
    Ok(())
}

fn worktree_gates_path(repo: &gix::Repository) -> anyhow::Result<PathBuf> {
    let workdir = repo
        .workdir()
        .ok_or_else(|| anyhow!("governance gates writes require a non-bare repository"))?;
    Ok(workdir.join(GATES_PATH))
}

fn read_committed_gates(repo: &gix::Repository, target_ref: &str) -> anyhow::Result<String> {
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
        .lookup_entry_by_path(Path::new(GATES_PATH))
        .with_context(|| format!("looking up {GATES_PATH} in {target_ref}"))?
    else {
        return Ok(String::new());
    };
    let blob = repo
        .find_blob(entry.id())
        .with_context(|| format!("reading {GATES_PATH} blob at {target_ref}"))?;
    let content = std::str::from_utf8(&blob.data)
        .with_context(|| format!("decoding {GATES_PATH} at {target_ref} as UTF-8"))?;
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
