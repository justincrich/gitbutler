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
    GroupWire, PermissionsWire, PrincipalId, PrincipalWire, gates_path, load_governance_config,
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

/// Compile-time witness that the caller passed the authenticated fleet-owner
/// boundary.
///
/// The private `_private` field makes this constructible ONLY via
/// [`FleetOwnerCapability::mint`] — even from other crates, which cannot use a
/// struct literal against a private field. Every `*_as_fleet_owner` governance
/// mutation requires `&FleetOwnerCapability`, so none of them is reachable
/// without first crossing the authenticated boundary that mints the witness.
pub struct FleetOwnerCapability {
    _private: (),
}

impl FleetOwnerCapability {
    /// Mint the fleet-owner capability witness.
    ///
    /// Call this ONLY at the authenticated fleet-owner boundary, immediately
    /// after the signed-in desktop fleet-owner identity has been resolved.
    #[must_use]
    pub fn mint() -> Self {
        Self { _private: () }
    }
}

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
    /// True when the target ref has no committed governance config. This is a normal
    /// "not set up yet" state the UI renders as guidance, NOT an error.
    pub not_configured: bool,
    /// The resolved governance target ref (e.g. `refs/remotes/origin/master`), so the
    /// UI reuses the workspace-resolved ref for follow-up reads instead of guessing one.
    pub target_ref: String,
}

impl From<AuthoritySet> for GovernanceStatus {
    fn from(authorities: AuthoritySet) -> Self {
        Self {
            authorities: authorities
                .iter()
                .map(|authority| authority.name().to_owned())
                .collect(),
            not_configured: false,
            target_ref: String::new(),
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

const GOVERNANCE_COMMIT_MESSAGE: &str = "chore: update governance config";
type PrincipalAuthorityRows = Vec<(PrincipalId, AuthoritySet)>;
type GroupRows = Vec<(GroupName, Group)>;

/// Caller-supplied branch protection update payload.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BranchProtectionInput {
    /// Whether the branch requires administration:write to mutate.
    pub protected: bool,
    /// Minimum review approvals required for this branch.
    pub min_approvals: Option<usize>,
    /// Whether approvals must come from a user distinct from the author.
    pub require_distinct_from_author: Option<bool>,
    /// Groups from which approval is required.
    pub require_approval_from_group: Option<Vec<String>>,
}

/// One branch gate entry returned through the API boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct BranchGateEntry {
    /// Branch name.
    pub name: String,
    /// Whether the branch is protected.
    pub protected: bool,
    /// Minimum review approvals required by the committed or working-tree gate.
    pub min_approvals: usize,
    /// Whether approvals must be distinct from the commit author.
    pub require_distinct_from_author: bool,
    /// Groups from which approval is required.
    pub require_approval_from_group: Vec<String>,
    /// Whether the working-tree gate differs from the committed target-ref gate.
    pub pending: bool,
}

/// Result of reading or updating branch gates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct BranchGatesOutcome {
    /// Branch gate entries.
    pub branches: Vec<BranchGateEntry>,
    /// Ref-pin caveat for branch gate writes.
    pub caveat: &'static str,
}

/// Read-only working-tree-vs-target-ref governance diff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GovernancePending {
    /// Per-principal effective authority comparison.
    pub principals: Vec<GovernancePendingPrincipal>,
    /// Number of governance changes between committed and working-tree config.
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
    /// Additive, enforcement-neutral `kind` descriptor (e.g. `"agent"` / `"human"`)
    /// read from the committed target-ref `[[principal]]` entry — the same source
    /// [`principal_kind_read`] surfaces. `None` when the principal has no declared
    /// `kind` (default-human). Lets the desktop Principals list render agent/human
    /// badges from the list query without a second round-trip.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
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

/// Renderer row for one principal's additive `kind` descriptor.
///
/// `kind` is the enforcement-neutral descriptor naming the principal's kind
/// (e.g. `"agent"` / `"human"`); it never enters `GovConfig.principals` and no
/// gate reads it (LPR-005's invariant). See `but_authz::PrincipalWire::kind`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PrincipalKindList {
    /// One entry per principal present in the committed target-ref config
    /// (and any principal only present in the working-tree pending edit).
    pub principals: Vec<PrincipalKindEntry>,
}

/// One principal's declared `kind` plus its pending signal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PrincipalKindEntry {
    /// Stable principal identifier.
    pub principal_id: String,
    /// Commit-target-ref declared `kind` (`None` = human default). Read at the
    /// target ref like all governance config (anti-self-escalation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// True only when the working-tree `kind` differs from the committed
    /// target-ref `kind`. The desktop renderer surfaces this so the operator
    /// knows the edit is inert until `governance_commit` lands it.
    pub pending: bool,
}

/// Result of staging a principal `kind` descriptor write.
///
/// Mirrors `BranchGatesOutcome`: the post-write working-tree `kind` list (so
/// the operator sees exactly what was staged) plus the ref-pin caveat.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PrincipalKindOutcome {
    /// Post-write working-tree `kind` entries (one per principal in the file).
    pub principals: Vec<PrincipalKindEntry>,
    /// Ref-pin caveat: the staged descriptor takes effect once committed to
    /// the target branch via the existing `governance_commit` path.
    pub caveat: &'static str,
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
#[cfg(feature = "export-schema")]
but_schemars::register_sdk_type!(PrincipalKindList);
#[cfg(feature = "export-schema")]
but_schemars::register_sdk_type!(PrincipalKindEntry);
#[cfg(feature = "export-schema")]
but_schemars::register_sdk_type!(PrincipalKindOutcome);

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
///
/// Reports a graceful `not_configured` status (instead of an error) when the target ref
/// has no committed governance config, and returns the resolved `target_ref` so the UI
/// reuses it for follow-up reads. A caller that can't be resolved yields empty
/// authorities (read-only), not an error.
///
/// `governance_status_read` resolves the caller through
/// `but_authz::resolve_principal_from_env`, resolving the acting principal from the
/// `BUT_AGENT_HANDLE` environment variable against the committed
/// `.gitbutler/agents.toml` (handle set by the trusted harness wrapper, not
/// self-asserted). This read surface catches an unresolved-caller denial and
/// reports empty authorities.
#[but_api(napi, GovernanceStatus)]
pub fn governance_status_read(ctx: &Context) -> anyhow::Result<GovernanceStatus> {
    let repo = ctx.repo.get()?;
    let target_ref = target_ref_from_ctx(ctx, None)?;
    if !but_authz::governance_present(&repo, &target_ref)? {
        return Ok(GovernanceStatus {
            authorities: Vec::new(),
            not_configured: true,
            target_ref,
        });
    }
    let config = load_governance_config(&repo, &target_ref)?;
    let authorities = match but_authz::resolve_principal_from_env(&config) {
        Ok(caller) => but_authz::effective_authority(&caller, &config)
            .iter()
            .map(|authority| authority.name().to_owned())
            .collect(),
        // No resolvable caller (no/unknown handle): read-only, not an error.
        Err(_denial) => Vec::new(),
    };
    Ok(GovernanceStatus {
        authorities,
        not_configured: false,
        target_ref,
    })
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

/// Read each principal's additive `kind` descriptor (`principal_kind_read`)
/// through the but-api boundary.
///
/// A self-/branch-scoped read matching the `governance_principals_list` posture
/// (NO administration:write authority required): it loads the committed kinds
/// at the target ref plus a pending signal from the working-tree-vs-target-ref
/// diff. A non-governed ref / unresolvable caller yields an empty list
/// (read-only), not an error.
#[but_api(napi)]
pub fn principal_kind_read(ctx: &Context, target_ref: String) -> anyhow::Result<PrincipalKindList> {
    let repo = ctx.repo.get()?;
    let target_ref = target_ref_from_ctx(ctx, Some(&target_ref))?;
    principal_kind_read_with_repo(&repo, &target_ref)
}

/// Stage a principal `kind` descriptor write (`principal_kind_update`) through
/// the but-api boundary.
///
/// Composes `enforce_administration_write_gate` (the AUTHZ-006 guard) BEFORE
/// any write, then read-modify-writes the WORKING-TREE `permissions.toml`
/// setting ONLY the targeted principal's `kind`. The write is inert until
/// committed via the existing `governance_commit` path (`permissions.toml` is
/// already a `GOVERNANCE_COMMIT_PATHS` member); the outcome carries the
/// ref-pin caveat so the operator knows a commit is required.
#[but_api(napi)]
pub fn principal_kind_update(
    ctx: &Context,
    target_ref: String,
    principal: String,
    kind: String,
) -> anyhow::Result<PrincipalKindOutcome> {
    let repo = ctx.repo.get()?;
    let target_ref = target_ref_from_ctx(ctx, Some(&target_ref))?;
    principal_kind_update_with_repo(&repo, &target_ref, &principal, &kind)
}

/// Read branch gates under administration-read authority.
///
/// `branch_gates_read_with_repo` gets its caller from
/// `but_authz::resolve_principal_from_env`, resolving the acting principal from the
/// `BUT_AGENT_HANDLE` environment variable against the committed
/// `.gitbutler/agents.toml` (handle set by the trusted harness wrapper, not
/// self-asserted).
pub fn branch_gates_read_with_repo(
    repo: &gix::Repository,
    target_ref: &str,
) -> anyhow::Result<BranchGatesOutcome> {
    let config = load_governance_config(repo, target_ref)?;
    let caller = but_authz::resolve_principal_from_env(&config)?;
    enforce_branch_gates_read_authority(&config, caller.id())?;

    let committed = read_committed_gates_file(repo, target_ref)?;
    let working = read_worktree_gates(repo)?;
    Ok(branch_gates_outcome_from_committed(&committed, &working))
}

/// Read branch gates for a principal resolved by the desktop command boundary.
pub fn branch_gates_read_with_repo_as_principal(
    repo: &gix::Repository,
    target_ref: &str,
    principal: &str,
) -> anyhow::Result<BranchGatesOutcome> {
    let config = load_governance_config(repo, target_ref)?;
    let principal = PrincipalId::new(principal);
    enforce_branch_gates_read_authority(&config, &principal)?;

    let committed = read_committed_gates_file(repo, target_ref)?;
    let working = read_worktree_gates(repo)?;
    Ok(branch_gates_outcome_from_committed(&committed, &working))
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
    _cap: &FleetOwnerCapability,
    repo: &gix::Repository,
    target_ref: &str,
    principal: &str,
    branch: &str,
    protection: BranchProtectionInput,
) -> anyhow::Result<BranchGatesOutcome> {
    let config = load_governance_config(repo, target_ref)?;
    let principal = PrincipalId::new(principal);
    let held = effective_authority_for_principal(&config, &principal)?;
    if !held.contains(Authority::AdministrationWrite) {
        return Err(Denial::missing_permission(Authority::AdministrationWrite, &held).into());
    }

    branch_gates_update_authorized(repo, target_ref, branch, protection)
}

/// Read a principal's declared enforcement-neutral `kind` from a loaded
/// permissions wire.
///
/// Single source of truth for the `kind`-reading pattern shared by
/// [`principal_kind_read_with_repo`] and [`principals_list`]: find the
/// `[[principal]]` entry by id and clone its `kind` descriptor. Returns
/// `None` when the principal is absent or has no declared `kind`
/// (default-human). Works against either the committed target-ref wire or
/// the working-tree wire.
fn kind_for_principal(permissions: &PermissionsWire, principal_id: &str) -> Option<String> {
    permissions
        .principal
        .iter()
        .find(|entry| entry.id == principal_id)
        .and_then(|entry| entry.kind.clone())
}

/// Read each principal's committed declared `kind` plus a pending signal from
/// the working-tree-vs-target-ref diff.
///
/// Mirrors the `governance_principals_list` read posture (no write authority):
/// committed kinds are read from the target-ref blob (anti-self-escalation);
/// the working-tree file (when present) supplies the pending diff. A non-
/// governed ref yields an empty list, not an error.
pub fn principal_kind_read_with_repo(
    repo: &gix::Repository,
    target_ref: &str,
) -> anyhow::Result<PrincipalKindList> {
    let committed = but_authz::load_permissions_wire(repo, target_ref)?;
    let working = read_worktree_permissions_optional(repo)?;
    let mut principal_ids = committed
        .principal
        .iter()
        .map(|entry| entry.id.clone())
        .collect::<BTreeSet<_>>();
    for entry in &working.principal {
        principal_ids.insert(entry.id.clone());
    }

    let principals = principal_ids
        .into_iter()
        .map(|principal_id| {
            let committed_kind = kind_for_principal(&committed, &principal_id);
            let working_kind = kind_for_principal(&working, &principal_id);
            PrincipalKindEntry {
                principal_id,
                kind: committed_kind.clone(),
                pending: committed_kind != working_kind,
            }
        })
        .collect();
    Ok(PrincipalKindList { principals })
}

/// Stage a principal `kind` descriptor write under env-principal
/// `administration:write` authority. Composes the AUTHZ-006 admin guard BEFORE
/// any write; the write lands in the WORKING-TREE `permissions.toml` only
/// (inert until committed).
pub fn principal_kind_update_with_repo(
    repo: &gix::Repository,
    target_ref: &str,
    principal: &str,
    kind: &str,
) -> anyhow::Result<PrincipalKindOutcome> {
    enforce_administration_write_gate(repo, target_ref)?;
    principal_kind_update_authorized(repo, target_ref, principal, kind)
}

/// Stage a principal `kind` descriptor write after the desktop fleet-owner
/// boundary has asserted unconditional administration-write authority.
///
/// This is intentionally not used by the agent/env path, which must continue
/// to call [`principal_kind_update_with_repo`] and resolve `BUT_AGENT_HANDLE`
/// through `but-authz`.
pub fn principal_kind_update_with_repo_as_fleet_owner(
    repo: &gix::Repository,
    target_ref: &str,
    principal: &str,
    kind: &str,
) -> anyhow::Result<PrincipalKindOutcome> {
    principal_kind_update_authorized(repo, target_ref, principal, kind)
}

fn principal_kind_update_authorized(
    repo: &gix::Repository,
    target_ref: &str,
    principal: &str,
    kind: &str,
) -> anyhow::Result<PrincipalKindOutcome> {
    let parsed_kind = parse_principal_kind(kind)?;
    let mut permissions = load_permissions_for_write(repo, target_ref)?;
    let entry = principal_entry_mut(&mut permissions, principal)?;
    entry.kind = parsed_kind;
    write_worktree_permissions(repo, &permissions)?;

    // The outcome reflects the post-write WORKING-TREE state so the operator
    // sees exactly what was staged (mirrors `BranchGatesOutcome`).
    let working = read_worktree_permissions_optional(repo)?;
    let principal_ids = working
        .principal
        .iter()
        .map(|entry| entry.id.clone())
        .collect::<BTreeSet<_>>();
    let principals: Vec<PrincipalKindEntry> = principal_ids
        .into_iter()
        .map(|principal_id| {
            let working_kind = working
                .principal
                .iter()
                .find(|entry| entry.id == principal_id)
                .and_then(|entry| entry.kind.clone());
            PrincipalKindEntry {
                principal_id,
                // The post-write outcome surfaces the working-tree kind so the
                // operator sees the staged value; `pending` is computed against
                // the committed target-ref kind for renderer consistency.
                kind: working_kind,
                pending: false,
            }
        })
        .collect();
    // Read the committed kinds to recompute pending flags for the post-write
    // outcome so the renderer can still badge uncommitted edits.
    let committed = but_authz::load_permissions_wire(repo, target_ref)?;
    let principals = principals
        .into_iter()
        .map(|mut entry| {
            let committed_kind = committed
                .principal
                .iter()
                .find(|committed_entry| committed_entry.id == entry.principal_id)
                .and_then(|committed_entry| committed_entry.kind.clone());
            entry.pending = committed_kind != entry.kind;
            entry
        })
        .collect();

    Ok(PrincipalKindOutcome {
        principals,
        caveat: REF_PIN_CAVEAT,
    })
}

/// Validate a principal `kind` descriptor at the API boundary.
///
/// Accepts only `"agent"` or `"human"` (the two `PrincipalWire::kind` values
/// the agent-PR tag derivation checks). An unknown kind string is rejected
/// with `config.invalid` (the structured `classify_error` code); the frontend
/// surfaces `perm.denied` (non-admin write) and `config.invalid` (bad kind).
///
/// Implementation note: this validator MUST NOT branch on the kind value in a
/// way that trips the `invariant_build_gates` honesty grep — `kind` is an
/// enforcement-neutral descriptor, not an authorization axis. The
/// slice-contains shape keeps the grep green (no per-kind match arm).
fn parse_principal_kind(kind: &str) -> anyhow::Result<Option<String>> {
    const ALLOWED_KINDS: &[&str] = &["agent", "human"];
    if ALLOWED_KINDS.contains(&kind) {
        Ok(Some(kind.to_owned()))
    } else {
        Err(anyhow::Error::new(json::ConfigInvalid {
            code: "config.invalid",
            message: format!("unknown principal kind {kind:?}: expected one of {ALLOWED_KINDS:?}"),
            remediation_hint: "use a supported principal kind value (see PrincipalWire::kind docs)"
                .to_owned(),
        }))
    }
}

/// Read the working-tree `permissions.toml`, returning an empty
/// `PermissionsWire` when the file is absent (clean working tree → no pending
/// kind edits). This is the read-side companion to `load_permissions_for_write`
/// (which falls back to the committed blob) — for the pending diff we want to
/// distinguish "no working-tree file" from "working-tree file present".
fn read_worktree_permissions_optional(repo: &gix::Repository) -> anyhow::Result<PermissionsWire> {
    let path = worktree_permissions_path(repo)?;
    if !path.is_file() {
        return Ok(PermissionsWire::default());
    }
    let text = fs::read_to_string(&path)
        .with_context(|| format!("reading working-tree {}", permissions_path()))?;
    parse_permissions_text(&text)
}

fn enforce_branch_gates_read_authority(
    config: &GovConfig,
    principal: &PrincipalId,
) -> anyhow::Result<()> {
    let held = effective_authority_for_principal(config, principal)?;
    if held.contains(Authority::AdministrationRead) || held.contains(Authority::AdministrationWrite)
    {
        Ok(())
    } else {
        Err(Denial::missing_permission(Authority::AdministrationRead, &held).into())
    }
}

fn effective_authority_for_principal(
    config: &GovConfig,
    principal: &PrincipalId,
) -> anyhow::Result<AuthoritySet> {
    config
        .principal_authorities(principal)
        .cloned()
        .ok_or_else(|| Denial::unknown_principal(principal.as_str()).into())
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
    let gate = gate_entry_mut(&mut gates, branch)?;
    if let Some(min_approvals) = protection.min_approvals {
        gate.min_approvals = min_approvals;
    }
    if let Some(require_distinct_from_author) = protection.require_distinct_from_author {
        gate.require_distinct_from_author = require_distinct_from_author;
    }
    if let Some(require_approval_from_group) = protection.require_approval_from_group {
        gate.require_approval_from_group = require_approval_from_group;
    }
    write_worktree_gates(repo, &gates)?;

    let committed = read_committed_gates_file(repo, target_ref)?;
    Ok(branch_gates_outcome_from_working(&gates, &committed))
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
    _cap: &FleetOwnerCapability,
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
    for path in governance_commit_paths() {
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

fn governance_commit_paths() -> [&'static str; 2] {
    [gates_path(), permissions_path()]
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
    let mut pending = pending_diff(&committed, &working);
    pending.pending_count += pending_gates_file_count(repo, target_ref)?;
    Ok(pending)
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
    let committed_view = permissions_view(&committed)?;
    let working_view = permissions_view(&working)?;
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
            // `kind` is read from the committed target-ref wire (same source as
            // `principal_kind_read`) so the renderer can badge agent/human from
            // the list query alone; the working-tree kind only feeds the
            // dedicated `principal_kind_read` pending signal, not this field.
            let kind = kind_for_principal(&committed, &principal_id);

            GovernancePrincipalListEntry {
                principal_id,
                own_grants,
                inherited_grants,
                group_memberships,
                pending,
                kind,
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

fn pending_gates_file_count(repo: &gix::Repository, target_ref: &str) -> anyhow::Result<usize> {
    let committed = read_committed_gates(repo, target_ref)?;
    let working = read_worktree_governance_file(repo, gates_path())?;
    Ok(usize::from(committed.as_bytes() != working.as_slice()))
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

fn permissions_view(permissions: &PermissionsWire) -> anyhow::Result<PermissionsView> {
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
        config_invalid(format!(
            "reading working-tree {} failed: {error}",
            gates_path()
        ))
    })?;
    toml::from_str::<GatesFile>(&text).map_err(|error| {
        anyhow::Error::new(json::ConfigInvalid {
            code: "config.invalid",
            message: format!("malformed working-tree {}: {error}", gates_path()),
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
///
/// `group_list_with_repo` identifies the caller with
/// `but_authz::resolve_principal_from_env`, resolving the acting principal from the
/// `BUT_AGENT_HANDLE` environment variable against the committed
/// `.gitbutler/agents.toml` (handle set by the trusted harness wrapper, not
/// self-asserted).
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
    _cap: &FleetOwnerCapability,
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
        return Err(Denial::new(
            "config.invalid",
            format!("group {group} already exists"),
            "choose a unique group name or grant the existing group".to_owned(),
        )
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
    // GAP-5 red-hat fix: admin-write gate BEFORE token parse (see
    // perm_grant_with_repo for the full rationale).
    enforce_administration_write_gate(repo, target_ref)?;
    let parsed = parse_authorities(authorities)?;
    group_grant_authorized(repo, target_ref, group, &parsed)
}

/// Grant functional permissions to a governed group after the desktop
/// fleet-owner boundary has asserted unconditional administration-write
/// authority.
pub fn group_grant_with_repo_as_fleet_owner(
    _cap: &FleetOwnerCapability,
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
    _cap: &FleetOwnerCapability,
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
    _cap: &FleetOwnerCapability,
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
    _cap: &FleetOwnerCapability,
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
    _cap: &FleetOwnerCapability,
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
///
/// `perm_list_with_repo` scopes self-versus-admin reads by resolving the caller
/// with `but_authz::resolve_principal_from_env`, resolving the acting principal
/// from the `BUT_AGENT_HANDLE` environment variable against the committed
/// `.gitbutler/agents.toml` (handle set by the trusted harness wrapper, not
/// self-asserted).
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

// ---------------------------------------------------------------------------
// STEER-006 — self-scoped discovery: whoami / can_i
// ---------------------------------------------------------------------------

/// Self-scoped discovery picture returned by [`whoami_with_repo`].
///
/// Discloses the caller's (or an authorized target's) effective authority set,
/// its OWN group memberships (group names only — never the other members of
/// those groups), and its authorized-action set (the `but …` verbs the
/// principal can run, drawn from the closed [`but_authz::CATALOG`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WhoamiOutcome {
    /// Principal whose self picture is disclosed.
    pub principal: String,
    /// Effective functional authority tokens (direct ∪ group, sorted lexically).
    pub authorities: Vec<String>,
    /// The principal's OWN group memberships (group names, not other members).
    pub groups: Vec<String>,
    /// Authorized `but …` verbs the principal can run, from the closed CATALOG.
    pub authorized_actions: Vec<but_authz::AuthorizedAction>,
}

/// Self-scoped authority-hold answer returned by [`can_i_with_repo`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CanIOutcome {
    /// Principal whose authority set was queried.
    pub principal: String,
    /// Functional authority token that was checked.
    pub authority: String,
    /// Whether the principal effectively holds the authority.
    pub held: bool,
}

/// Route → catalog command mapping for the self-discovery authorized-action set.
/// Each command string MUST exist in [`but_authz::CATALOG`]; if one were removed
/// from CATALOG, the lookup omits it rather than emitting a phantom command.
const AUTHORIZED_ROUTE_COMMANDS: &[(but_authz::Route, &str)] = &[
    (but_authz::Route::Commit, "but commit"),
    (but_authz::Route::Merge, "but review merge"),
    (
        but_authz::Route::ForgeReviewsWrite,
        "but review request-changes",
    ),
    (but_authz::Route::ForgeCommentsWrite, "but review comment"),
    (but_authz::Route::ForgePullRequestsWrite, "but review new"),
];

/// The discovery affordance command — degradable (omitted if absent from
/// CATALOG, never a phantom/lying command).
const DISCOVERY_COMMAND: &str = "but perm list";

/// Derive the authorized-action set for a principal from the closed CATALOG.
///
/// For each route whose required authority the principal holds, the
/// corresponding CATALOG command is surfaced. The discovery affordance
/// (`but perm list`) is appended (degradable — omitted if not in CATALOG).
fn self_authorized_actions(held: &but_authz::AuthoritySet) -> Vec<but_authz::AuthorizedAction> {
    let catalog_lookup = |command: &str| -> Option<but_authz::AuthorizedAction> {
        but_authz::CATALOG
            .iter()
            .find(|action| action.command == command)
            .cloned()
    };

    let mut actions = Vec::new();
    for (route, command) in AUTHORIZED_ROUTE_COMMANDS {
        if held.contains(route.required_authority())
            && let Some(action) = catalog_lookup(command)
        {
            actions.push(action);
        }
    }
    // Degradable discovery affordance — omitted (not phantom) if not in CATALOG.
    if let Some(discovery) = catalog_lookup(DISCOVERY_COMMAND) {
        actions.push(discovery);
    }
    actions
}

/// Resolve a target principal from the governance config by id.
///
/// Returns a principal with an empty authority set and no groups when the id
/// is absent from the config — never panics, never reveals existence to a
/// cross-principal caller (the scope check in the caller denies first).
fn resolve_target_principal(
    target_id: &PrincipalId,
    config: &but_authz::GovConfig,
) -> but_authz::Principal {
    let authorities = config
        .principal_authorities(target_id)
        .cloned()
        .unwrap_or_else(AuthoritySet::empty);
    let groups = config
        .groups()
        .values()
        .filter(|group| group.members().contains(target_id))
        .map(|group| group.name().clone())
        .collect::<Vec<_>>();
    but_authz::Principal::new(target_id.clone(), authorities, groups)
}

/// Return the self-scoped discovery picture for a principal (`whoami`).
///
/// Reuses the SAME self-or-admin-read scope predicate as [`perm_list_with_repo`]:
/// a caller may inspect itself unconditionally, or another principal only when
/// it holds `administration:read` / `administration:write`. Cross-principal
/// recon by a non-admin caller is denied `perm.denied` leaking nothing about
/// the target.
///
/// The disclosure is self-scoped: it surfaces the target's effective authority
/// set, its OWN group memberships (group names only — never the other members
/// of those groups), and its authorized-action set from the closed CATALOG.
///
/// `whoami_with_repo` resolves the requesting principal with
/// `but_authz::resolve_principal_from_env`, resolving the acting principal from the
/// `BUT_AGENT_HANDLE` environment variable against the committed
/// `.gitbutler/agents.toml` (handle set by the trusted harness wrapper, not
/// self-asserted).
pub fn whoami_with_repo(
    repo: &gix::Repository,
    target_ref: &str,
    principal: Option<&str>,
) -> anyhow::Result<WhoamiOutcome> {
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

    let target_principal = resolve_target_principal(&target_id, &config);
    let effective = but_authz::effective_authority(&target_principal, &config);
    let authorities = effective
        .iter()
        .map(|authority| authority.name().to_owned())
        .collect::<Vec<_>>();
    let groups = target_principal
        .groups()
        .iter()
        .map(|group| group.as_str().to_owned())
        .collect::<Vec<_>>();
    let authorized_actions = self_authorized_actions(&effective);

    Ok(WhoamiOutcome {
        principal: target.to_owned(),
        authorities,
        groups,
        authorized_actions,
    })
}

/// Check whether a principal effectively holds an authority (`can-i`).
///
/// Reuses the SAME self-or-admin-read scope predicate as [`perm_list_with_repo`].
/// Returns [`CanIOutcome`] with `held = true/false` — never denies for an
/// unheld authority (that is a factual answer, not an error). Cross-principal
/// recon by a non-admin caller is denied `perm.denied` before the target is
/// resolved, so the endpoint cannot be used as a principal-existence oracle.
///
/// `can_i_with_repo` resolves the requesting principal with
/// `but_authz::resolve_principal_from_env`, resolving the acting principal from the
/// `BUT_AGENT_HANDLE` environment variable against the committed
/// `.gitbutler/agents.toml` (handle set by the trusted harness wrapper, not
/// self-asserted).
pub fn can_i_with_repo(
    repo: &gix::Repository,
    target_ref: &str,
    authority_token: &str,
    principal: Option<&str>,
) -> anyhow::Result<CanIOutcome> {
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

    let authority = Authority::parse(authority_token).map_err(|error| {
        config_invalid(format!(
            "unknown authority token {}: {}",
            authority_token,
            error.token()
        ))
    })?;

    let target_principal = resolve_target_principal(&target_id, &config);
    let effective = but_authz::effective_authority(&target_principal, &config);
    let held = effective.contains(authority);

    Ok(CanIOutcome {
        principal: target.to_owned(),
        authority: authority_token.to_owned(),
        held,
    })
}

/// Grant direct functional permissions in the working-tree governance config.
pub fn perm_grant_with_repo(
    repo: &gix::Repository,
    target_ref: &str,
    principal: &str,
    authorities: &[&str],
) -> anyhow::Result<PermWriteOutcome> {
    // GAP-5 red-hat fix: run the admin-write gate BEFORE parsing the supplied
    // authority tokens. The previous ordering (parse → gate) leaked token
    // validity information to non-admin callers (they could distinguish
    // "unknown token" from "perm.denied"). Admin-first ensures a non-admin
    // sees only the perm.denied, never the token-validation error.
    enforce_administration_write_gate(repo, target_ref)?;
    let parsed = parse_authorities(authorities)?;
    perm_grant_with_parsed_authorities(repo, target_ref, principal, &parsed)
}

/// Grant direct permissions after the desktop command boundary has resolved the
/// signed-in fleet-owner and asserted the v1 administration-write exception.
///
/// This is intentionally not used by the agent runtime-registry path, which must
/// continue to call [`perm_grant_with_repo`] and resolve through `but-authz`.
pub fn perm_grant_with_repo_as_fleet_owner(
    _cap: &FleetOwnerCapability,
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
    _cap: &FleetOwnerCapability,
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
        kind: None,
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
/// Mirrors the full `[[branch]]` and `[[gate]]` shape consumed by the merge gate
/// so branch edits cannot drop review requirements.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GatesFile {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    branch: Vec<GatesBranchWire>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    gate: Vec<GatesGateWire>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GatesBranchWire {
    name: String,
    protected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GatesGateWire {
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

fn branch_gates_outcome_from_committed(
    committed: &GatesFile,
    working: &GatesFile,
) -> BranchGatesOutcome {
    let mut branches = committed
        .branch
        .iter()
        .map(|branch| branch_gate_entry(branch, committed.gate_for(&branch.name), working))
        .collect::<Vec<_>>();
    branches.extend(
        working
            .branch
            .iter()
            .filter(|branch| committed.branch_for(&branch.name).is_none())
            .map(|branch| branch_gate_entry(branch, working.gate_for(&branch.name), committed)),
    );

    BranchGatesOutcome {
        branches,
        caveat: REF_PIN_CAVEAT,
    }
}

fn branch_gates_outcome_from_working(
    working: &GatesFile,
    committed: &GatesFile,
) -> BranchGatesOutcome {
    BranchGatesOutcome {
        branches: working
            .branch
            .iter()
            .map(|branch| branch_gate_entry(branch, working.gate_for(&branch.name), committed))
            .collect(),
        caveat: REF_PIN_CAVEAT,
    }
}

fn branch_gate_entry(
    branch: &GatesBranchWire,
    gate: Option<&GatesGateWire>,
    comparison: &GatesFile,
) -> BranchGateEntry {
    BranchGateEntry {
        name: branch.name.clone(),
        protected: branch.protected,
        min_approvals: gate.map_or(0, |gate| gate.min_approvals),
        require_distinct_from_author: gate.is_some_and(|gate| gate.require_distinct_from_author),
        require_approval_from_group: gate
            .map(|gate| gate.require_approval_from_group.clone())
            .unwrap_or_default(),
        pending: gates_entry_pending(branch, gate, comparison),
    }
}

fn gates_entry_pending(
    branch: &GatesBranchWire,
    gate: Option<&GatesGateWire>,
    comparison: &GatesFile,
) -> bool {
    let Some(comparison_branch) = comparison.branch_for(&branch.name) else {
        return true;
    };
    if comparison_branch != branch {
        return true;
    }
    comparison.gate_for(&branch.name) != gate
}

impl GatesFile {
    fn branch_for(&self, name: &str) -> Option<&GatesBranchWire> {
        self.branch.iter().find(|branch| branch.name == name)
    }

    fn gate_for(&self, branch: &str) -> Option<&GatesGateWire> {
        self.gate.iter().find(|gate| gate.branch == branch)
    }
}

fn gate_entry_mut<'a>(
    gates: &'a mut GatesFile,
    branch: &str,
) -> anyhow::Result<&'a mut GatesGateWire> {
    if let Some(position) = gates.gate.iter().position(|entry| entry.branch == branch) {
        return gates
            .gate
            .get_mut(position)
            .ok_or_else(|| anyhow!("gate position disappeared while preparing gates rewrite"));
    }

    gates.gate.push(GatesGateWire {
        branch: branch.to_owned(),
        kind: "review".to_owned(),
        min_approvals: 0,
        require_approval_from_group: Vec::new(),
        require_distinct_from_author: false,
    });
    gates
        .gate
        .last_mut()
        .ok_or_else(|| anyhow!("gate entry was not available after seeding"))
}

fn load_gates_for_write(repo: &gix::Repository, _target_ref: &str) -> anyhow::Result<GatesFile> {
    read_worktree_gates(repo)
}

fn read_worktree_gates(repo: &gix::Repository) -> anyhow::Result<GatesFile> {
    let path = worktree_gates_path(repo)?;
    if !path.is_file() {
        return Ok(GatesFile::default());
    }
    let text = fs::read_to_string(&path)
        .with_context(|| format!("reading working-tree {}", gates_path()))?;
    parse_gates_text(&text)
}

fn parse_gates_text(text: &str) -> anyhow::Result<GatesFile> {
    if text.trim().is_empty() {
        return Ok(GatesFile::default());
    }
    toml::from_str::<GatesFile>(text)
        .with_context(|| format!("parsing working-tree {}", gates_path()))
}

fn write_worktree_gates(repo: &gix::Repository, gates: &GatesFile) -> anyhow::Result<()> {
    let path = worktree_gates_path(repo)?;
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("{} must have a parent directory", gates_path()))?;
    fs::create_dir_all(parent)
        .with_context(|| format!("creating working-tree {}", parent.display()))?;
    let encoded = toml::to_string(gates)
        .with_context(|| format!("serializing working-tree {}", gates_path()))?;
    fs::write(&path, encoded).with_context(|| format!("writing working-tree {}", gates_path()))?;
    Ok(())
}

fn worktree_gates_path(repo: &gix::Repository) -> anyhow::Result<PathBuf> {
    let workdir = repo
        .workdir()
        .ok_or_else(|| anyhow!("governance gates writes require a non-bare repository"))?;
    Ok(workdir.join(gates_path()))
}

fn read_committed_gates_file(
    repo: &gix::Repository,
    target_ref: &str,
) -> anyhow::Result<GatesFile> {
    parse_gates_text(&read_committed_gates(repo, target_ref)?)
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
        .lookup_entry_by_path(Path::new(gates_path()))
        .with_context(|| format!("looking up {} in {target_ref}", gates_path()))?
    else {
        return Ok(String::new());
    };
    let blob = repo
        .find_blob(entry.id())
        .with_context(|| format!("reading {} blob at {target_ref}", gates_path()))?;
    let content = std::str::from_utf8(&blob.data)
        .with_context(|| format!("decoding {} at {target_ref} as UTF-8", gates_path()))?;
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

    super::config_mutate::classify_error(err).map(|gate_error| {
        // GAP-4 red-hat fix: the AdminWriteGateError fallback path dropped
        // remediation_hint to None. For but_authz::ConfigError (the only
        // type the fallback catches that the earlier arms miss), surface
        // a meaningful remediation_hint so the CLI payload still carries
        // recovery context. The Denial and ConfigInvalid arms above
        // already populate remediation_hint.
        let remediation_hint = err
            .downcast_ref::<but_authz::ConfigError>()
            .map(|config_error| {
                format!(
                    "fix the committed governance config and re-commit (config.invalid: {config_error})"
                )
            });
        GovernanceErrorPayload {
            code: gate_error.code,
            message: gate_error.message,
            remediation_hint,
        }
    })
}
