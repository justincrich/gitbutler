//! Governance command-boundary helpers for desktop fleet-owner identity.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
    str::FromStr as _,
};

use anyhow::{Context as _, anyhow};
use but_api::{
    json,
    legacy::governance::{
        BranchGatesOutcome, BranchProtectionInput, GrantOutcome, GroupWriteOutcome,
        PermWriteOutcome,
    },
};
use but_authz::{
    Authority, AuthoritySet, BranchName, BranchProtection, GovConfig, Group, GroupName,
    PermissionsWire, PrincipalId, load_governance_config, permissions_path,
};
use but_core::RepositoryExt as _;
use but_ctx::{Context, ProjectHandleOrLegacyProjectId};
use gix::{bstr::ByteSlice as _, object::tree::EntryKind};
use serde::{Deserialize, Serialize};

const GATES_PATH: &str = ".gitbutler/gates.toml";
const GOVERNANCE_COMMIT_MESSAGE: &str = "chore: update governance config";
type PrincipalAuthorityRows = Vec<(PrincipalId, AuthoritySet)>;
type GroupRows = Vec<(GroupName, Group)>;
const GOVERNANCE_COMMIT_PATHS: [&str; 2] = [GATES_PATH, ".gitbutler/permissions.toml"];

/// Read-only working-tree-vs-target-ref governance diff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GovernancePending {
    /// Per-principal effective authority comparison.
    pub principals: Vec<GovernancePendingPrincipal>,
    /// Number of authority tokens that differ between committed and working-tree config.
    pub pending_count: usize,
}

/// Read-only renderer contract for all principals present in governance config.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GovernancePrincipalsList {
    /// Principals present in committed direct principals, committed group members,
    /// working-tree direct principals, or working-tree group members.
    pub principals: Vec<GovernancePrincipalListEntry>,
}

/// Renderer row for one governance principal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceInheritedGrant {
    /// Functional authority token.
    pub authority: String,
    /// Human-readable inheritance source.
    pub source_label: String,
}

/// Pending authority diff for one principal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GovernancePendingChange {
    /// The working tree grants an authority not present at the target ref.
    Grant,
    /// The working tree removes an authority still present at the target ref.
    Revoke,
}

/// Minimal signed-in desktop user identity required for the fleet-owner path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FleetOwnerIdentity {
    /// Stable GitButler user id from the desktop session.
    pub user_id: u64,
    /// Optional user login, retained for diagnostics and future audit hooks.
    pub login: Option<String>,
}

/// Source of the signed-in desktop user at the command boundary.
pub trait DesktopSession: Send + Sync {
    /// Resolve the signed-in desktop user that acts as the v1 fleet-owner.
    fn fleet_owner_identity(&self) -> anyhow::Result<FleetOwnerIdentity>;
}

/// Tauri-managed desktop session resolver used by governance command wrappers.
pub struct DesktopSessionState {
    session: Box<dyn DesktopSession>,
}

impl DesktopSessionState {
    /// Store the desktop session resolver used by Tauri command wrappers.
    pub fn new(session: impl DesktopSession + 'static) -> Self {
        Self {
            session: Box::new(session),
        }
    }

    fn session(&self) -> &dyn DesktopSession {
        self.session.as_ref()
    }
}

/// Production desktop-session resolver backed by `legacy::users::get_user`.
#[derive(Debug, Clone, Copy)]
pub struct GitButlerDesktopSession;

impl DesktopSession for GitButlerDesktopSession {
    fn fleet_owner_identity(&self) -> anyhow::Result<FleetOwnerIdentity> {
        let user = but_api::legacy::users::get_user()?.ok_or_else(|| {
            anyhow!("a signed-in desktop user is required for governance changes")
        })?;
        Ok(FleetOwnerIdentity {
            user_id: user.id,
            login: user.login,
        })
    }
}

/// Deterministic desktop session used by integration harnesses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestDesktopSession {
    /// Stable test user id.
    pub user_id: u64,
    /// Optional test login.
    pub login: Option<String>,
}

impl DesktopSession for TestDesktopSession {
    fn fleet_owner_identity(&self) -> anyhow::Result<FleetOwnerIdentity> {
        Ok(FleetOwnerIdentity {
            user_id: self.user_id,
            login: self.login.clone(),
        })
    }
}

/// Resolve the project context and signed-in desktop fleet-owner before any
/// governance config-management command can fall through to env-handle authz.
pub fn fleet_owner_context(
    session: &(impl DesktopSession + ?Sized),
    project_id: ProjectHandleOrLegacyProjectId,
    target_ref: &str,
) -> Result<(Context, FleetOwnerIdentity), json::Error> {
    let owner = session.fleet_owner_identity().map_err(json::Error::from)?;
    let ctx = context_for_project(project_id, target_ref).map_err(json::Error::from)?;
    Ok((ctx, owner))
}

/// Invoke `perm_grant` as the signed-in desktop fleet-owner.
pub fn perm_grant_for_desktop_session(
    session: &dyn DesktopSession,
    project_id: ProjectHandleOrLegacyProjectId,
    target_ref: String,
    principal: String,
    authorities: Vec<String>,
) -> Result<GrantOutcome, json::Error> {
    let (ctx, _owner) = fleet_owner_context(session, project_id, &target_ref)?;
    let repo = ctx.repo.get().map_err(json::Error::from)?;
    let authorities = authority_slices(&authorities);
    but_api::legacy::governance::perm_grant_with_repo_as_fleet_owner(
        &repo,
        &target_ref,
        &principal,
        &authorities,
    )
    .map(GrantOutcome::from)
    .map_err(json::Error::from)
}

/// Invoke `perm_revoke` as the signed-in desktop fleet-owner.
pub fn perm_revoke_for_desktop_session(
    session: &dyn DesktopSession,
    project_id: ProjectHandleOrLegacyProjectId,
    target_ref: String,
    principal: String,
    authorities: Vec<String>,
) -> Result<PermWriteOutcome, json::Error> {
    let (ctx, _owner) = fleet_owner_context(session, project_id, &target_ref)?;
    let repo = ctx.repo.get().map_err(json::Error::from)?;
    let authorities = authority_slices(&authorities);
    but_api::legacy::governance::perm_revoke_with_repo_as_fleet_owner(
        &repo,
        &target_ref,
        &principal,
        &authorities,
    )
    .map_err(json::Error::from)
}

/// Invoke `group_create` as the signed-in desktop fleet-owner.
pub fn group_create_for_desktop_session(
    session: &dyn DesktopSession,
    project_id: ProjectHandleOrLegacyProjectId,
    target_ref: String,
    group: String,
    authorities: Vec<String>,
) -> Result<GroupWriteOutcome, json::Error> {
    let (ctx, _owner) = fleet_owner_context(session, project_id, &target_ref)?;
    let repo = ctx.repo.get().map_err(json::Error::from)?;
    let authorities = authority_slices(&authorities);
    but_api::legacy::governance::group_create_with_repo_as_fleet_owner(
        &repo,
        &target_ref,
        &group,
        &authorities,
    )
    .map_err(json::Error::from)
}

/// Invoke `group_grant` as the signed-in desktop fleet-owner.
pub fn group_grant_for_desktop_session(
    session: &dyn DesktopSession,
    project_id: ProjectHandleOrLegacyProjectId,
    target_ref: String,
    group: String,
    authorities: Vec<String>,
) -> Result<GroupWriteOutcome, json::Error> {
    let (ctx, _owner) = fleet_owner_context(session, project_id, &target_ref)?;
    let repo = ctx.repo.get().map_err(json::Error::from)?;
    let authorities = authority_slices(&authorities);
    but_api::legacy::governance::group_grant_with_repo_as_fleet_owner(
        &repo,
        &target_ref,
        &group,
        &authorities,
    )
    .map_err(json::Error::from)
}

/// Invoke `group_add_member` as the signed-in desktop fleet-owner.
pub fn group_add_member_for_desktop_session(
    session: &dyn DesktopSession,
    project_id: ProjectHandleOrLegacyProjectId,
    target_ref: String,
    group: String,
    member: String,
) -> Result<GroupWriteOutcome, json::Error> {
    let (ctx, _owner) = fleet_owner_context(session, project_id, &target_ref)?;
    let repo = ctx.repo.get().map_err(json::Error::from)?;
    but_api::legacy::governance::group_add_member_with_repo_as_fleet_owner(
        &repo,
        &target_ref,
        &group,
        &member,
    )
    .map_err(json::Error::from)
}

/// Invoke `group_remove_member` as the signed-in desktop fleet-owner.
pub fn group_remove_member_for_desktop_session(
    session: &dyn DesktopSession,
    project_id: ProjectHandleOrLegacyProjectId,
    target_ref: String,
    group: String,
    member: String,
) -> Result<GroupWriteOutcome, json::Error> {
    let (ctx, _owner) = fleet_owner_context(session, project_id, &target_ref)?;
    let repo = ctx.repo.get().map_err(json::Error::from)?;
    but_api::legacy::governance::group_remove_member_with_repo_as_fleet_owner(
        &repo,
        &target_ref,
        &group,
        &member,
    )
    .map_err(json::Error::from)
}

/// Invoke `group_delete` as the signed-in desktop fleet-owner.
pub fn group_delete_for_desktop_session(
    session: &dyn DesktopSession,
    project_id: ProjectHandleOrLegacyProjectId,
    target_ref: String,
    group: String,
) -> Result<GroupWriteOutcome, json::Error> {
    let (ctx, _owner) = fleet_owner_context(session, project_id, &target_ref)?;
    let repo = ctx.repo.get().map_err(json::Error::from)?;
    but_api::legacy::governance::group_delete_with_repo_as_fleet_owner(&repo, &target_ref, &group)
        .map_err(json::Error::from)
}

/// Invoke `branch_gates_update` as the signed-in desktop fleet-owner.
pub fn branch_gates_update_for_desktop_session(
    session: &dyn DesktopSession,
    project_id: ProjectHandleOrLegacyProjectId,
    target_ref: String,
    branch: String,
    protection: BranchProtectionInput,
) -> Result<BranchGatesOutcome, json::Error> {
    let (ctx, _owner) = fleet_owner_context(session, project_id, &target_ref)?;
    let repo = ctx.repo.get().map_err(json::Error::from)?;
    but_api::legacy::governance::branch_gates_update_with_repo_as_fleet_owner(
        &repo,
        &target_ref,
        &branch,
        protection,
    )
    .map_err(json::Error::from)
}

/// Invoke `perm_grant` through the ordinary agent/env path.
pub fn perm_grant_for_agent_env(
    project_id: ProjectHandleOrLegacyProjectId,
    target_ref: String,
    principal: String,
    authorities: Vec<String>,
) -> Result<GrantOutcome, json::Error> {
    let ctx = context_for_project(project_id, &target_ref).map_err(json::Error::from)?;
    but_api::legacy::governance::perm_grant(&ctx, target_ref, principal, authorities)
        .map_err(json::Error::from)
}

/// Invoke the read-only pending governance diff command.
pub fn governance_pending_for_project(
    project_id: ProjectHandleOrLegacyProjectId,
    target_ref: String,
) -> Result<GovernancePending, json::Error> {
    let ctx = context_for_project(project_id, &target_ref).map_err(json::Error::from)?;
    let repo = ctx.repo.get().map_err(json::Error::from)?;
    governance_pending_with_repo(&repo, &target_ref).map_err(json::Error::from)
}

/// Invoke the read-only governance principals renderer contract.
pub fn governance_principals_list_for_project(
    project_id: ProjectHandleOrLegacyProjectId,
    target_ref: String,
) -> Result<GovernancePrincipalsList, json::Error> {
    let ctx = context_for_project(project_id, &target_ref).map_err(json::Error::from)?;
    let repo = ctx.repo.get().map_err(json::Error::from)?;
    governance_principals_list_with_repo(&repo, &target_ref).map_err(json::Error::from)
}

/// Commit pending governance config files to the target ref.
pub fn governance_commit_for_project(
    project_id: ProjectHandleOrLegacyProjectId,
    target_ref: String,
) -> Result<GovernanceCommitOutcome, json::Error> {
    let ctx = context_for_project(project_id, &target_ref).map_err(json::Error::from)?;
    let repo = ctx.repo.get().map_err(json::Error::from)?;
    governance_commit_with_repo(&repo, &target_ref).map_err(json::Error::from)
}

/// Commit only governance config files from the worktree onto the target ref.
pub fn governance_commit_with_repo(
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

/// Return all governance principals needed by the desktop renderer.
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

fn config_invalid(message: String) -> anyhow::Error {
    anyhow::Error::new(json::ConfigInvalid {
        code: "config.invalid",
        message,
        remediation_hint:
            "fix the malformed governance config and recommit it to the target branch".to_owned(),
    })
}

fn worktree_permissions_path(repo: &gix::Repository) -> anyhow::Result<std::path::PathBuf> {
    let workdir = repo
        .workdir()
        .ok_or_else(|| anyhow!("governance pending reads require a non-bare repository"))?;
    Ok(workdir.join(permissions_path()))
}

fn worktree_gates_path(repo: &gix::Repository) -> anyhow::Result<std::path::PathBuf> {
    let workdir = repo
        .workdir()
        .ok_or_else(|| anyhow!("governance pending reads require a non-bare repository"))?;
    Ok(workdir.join(GATES_PATH))
}

/// Working-tree `gates.toml` wire format.
#[derive(Debug, Clone, Default, Deserialize)]
struct GatesFile {
    #[serde(default)]
    branch: Vec<GatesBranchWire>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct GatesBranchWire {
    name: String,
    protected: bool,
}

fn context_for_project(
    project_id: ProjectHandleOrLegacyProjectId,
    target_ref: &str,
) -> anyhow::Result<Context> {
    let ctx = Context::try_from(project_id)?.with_memory_app_cache();
    let target_commit_id = {
        let repo = ctx.repo.get()?;
        ref_id(&repo, target_ref)?
    };
    let mut project_meta = ctx.project_meta()?;
    project_meta.target_ref = Some(target_ref.try_into()?);
    project_meta.target_commit_id = Some(target_commit_id);
    ctx.set_project_meta(project_meta)
        .context("setting governance command target ref")?;
    Ok(ctx)
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

fn authority_slices(authorities: &[String]) -> Vec<&str> {
    authorities.iter().map(String::as_str).collect()
}

/// Tauri command wrapper for desktop fleet-owner `perm_grant`.
pub mod tauri_perm_grant {
    use super::{DesktopSessionState, GrantOutcome, ProjectHandleOrLegacyProjectId, json};

    /// Grant direct governance permissions as the signed-in desktop fleet-owner.
    #[tauri::command]
    pub fn perm_grant(
        desktop_session: tauri::State<'_, DesktopSessionState>,
        project_id: ProjectHandleOrLegacyProjectId,
        target_ref: String,
        principal: String,
        authorities: Vec<String>,
    ) -> Result<GrantOutcome, json::Error> {
        super::perm_grant_for_desktop_session(
            desktop_session.session(),
            project_id,
            target_ref,
            principal,
            authorities,
        )
    }
}

/// Tauri command wrapper for agent/env-authorized `perm_grant`.
pub mod tauri_agent_perm_grant {
    use super::{GrantOutcome, ProjectHandleOrLegacyProjectId, json};

    /// Grant direct governance permissions through the server-side agent/env boundary.
    #[tauri::command]
    pub fn agent_perm_grant(
        project_id: ProjectHandleOrLegacyProjectId,
        target_ref: String,
        principal: String,
        authorities: Vec<String>,
    ) -> Result<GrantOutcome, json::Error> {
        super::perm_grant_for_agent_env(project_id, target_ref, principal, authorities)
    }
}

/// Tauri command wrapper for read-only pending governance diff.
pub mod tauri_governance_pending {
    use super::{GovernancePending, ProjectHandleOrLegacyProjectId, json};

    /// Read the working-tree-vs-target-ref pending governance diff.
    #[tauri::command]
    pub fn governance_pending(
        project_id: ProjectHandleOrLegacyProjectId,
        target_ref: String,
    ) -> Result<GovernancePending, json::Error> {
        super::governance_pending_for_project(project_id, target_ref)
    }
}

/// Tauri command wrapper for read-only governance principals list.
pub mod tauri_governance_principals_list {
    use super::{GovernancePrincipalsList, ProjectHandleOrLegacyProjectId, json};

    /// List all principals needed by the governance renderer.
    #[tauri::command]
    pub fn governance_principals_list(
        project_id: ProjectHandleOrLegacyProjectId,
        target_ref: String,
    ) -> Result<GovernancePrincipalsList, json::Error> {
        super::governance_principals_list_for_project(project_id, target_ref)
    }
}

/// Tauri command wrapper for committing pending governance config.
pub mod tauri_governance_commit {
    use super::{GovernanceCommitOutcome, ProjectHandleOrLegacyProjectId, json};

    /// Commit pending governance config files to the target ref.
    #[tauri::command]
    pub fn governance_commit(
        project_id: ProjectHandleOrLegacyProjectId,
        target_ref: String,
    ) -> Result<GovernanceCommitOutcome, json::Error> {
        super::governance_commit_for_project(project_id, target_ref)
    }
}

/// Tauri command wrapper for desktop fleet-owner `perm_revoke`.
pub mod tauri_perm_revoke {
    use super::{DesktopSessionState, PermWriteOutcome, ProjectHandleOrLegacyProjectId, json};

    /// Revoke direct governance permissions as the signed-in desktop fleet-owner.
    #[tauri::command]
    pub fn perm_revoke(
        desktop_session: tauri::State<'_, DesktopSessionState>,
        project_id: ProjectHandleOrLegacyProjectId,
        target_ref: String,
        principal: String,
        authorities: Vec<String>,
    ) -> Result<PermWriteOutcome, json::Error> {
        super::perm_revoke_for_desktop_session(
            desktop_session.session(),
            project_id,
            target_ref,
            principal,
            authorities,
        )
    }
}

/// Tauri command wrapper for desktop fleet-owner `group_create`.
pub mod tauri_group_create {
    use super::{DesktopSessionState, GroupWriteOutcome, ProjectHandleOrLegacyProjectId, json};

    /// Create a governed group as the signed-in desktop fleet-owner.
    #[tauri::command]
    pub fn group_create(
        desktop_session: tauri::State<'_, DesktopSessionState>,
        project_id: ProjectHandleOrLegacyProjectId,
        target_ref: String,
        group: String,
        authorities: Vec<String>,
    ) -> Result<GroupWriteOutcome, json::Error> {
        super::group_create_for_desktop_session(
            desktop_session.session(),
            project_id,
            target_ref,
            group,
            authorities,
        )
    }
}

/// Tauri command wrapper for desktop fleet-owner `group_grant`.
pub mod tauri_group_grant {
    use super::{DesktopSessionState, GroupWriteOutcome, ProjectHandleOrLegacyProjectId, json};

    /// Grant governed group permissions as the signed-in desktop fleet-owner.
    #[tauri::command]
    pub fn group_grant(
        desktop_session: tauri::State<'_, DesktopSessionState>,
        project_id: ProjectHandleOrLegacyProjectId,
        target_ref: String,
        group: String,
        authorities: Vec<String>,
    ) -> Result<GroupWriteOutcome, json::Error> {
        super::group_grant_for_desktop_session(
            desktop_session.session(),
            project_id,
            target_ref,
            group,
            authorities,
        )
    }
}

/// Tauri command wrapper for desktop fleet-owner `group_add_member`.
pub mod tauri_group_add_member {
    use super::{DesktopSessionState, GroupWriteOutcome, ProjectHandleOrLegacyProjectId, json};

    /// Add a principal to a governed group as the signed-in desktop fleet-owner.
    #[tauri::command]
    pub fn group_add_member(
        desktop_session: tauri::State<'_, DesktopSessionState>,
        project_id: ProjectHandleOrLegacyProjectId,
        target_ref: String,
        group: String,
        member: String,
    ) -> Result<GroupWriteOutcome, json::Error> {
        super::group_add_member_for_desktop_session(
            desktop_session.session(),
            project_id,
            target_ref,
            group,
            member,
        )
    }
}

/// Tauri command wrapper for desktop fleet-owner `group_remove_member`.
pub mod tauri_group_remove_member {
    use super::{DesktopSessionState, GroupWriteOutcome, ProjectHandleOrLegacyProjectId, json};

    /// Remove a principal from a governed group as the signed-in desktop fleet-owner.
    #[tauri::command]
    pub fn group_remove_member(
        desktop_session: tauri::State<'_, DesktopSessionState>,
        project_id: ProjectHandleOrLegacyProjectId,
        target_ref: String,
        group: String,
        member: String,
    ) -> Result<GroupWriteOutcome, json::Error> {
        super::group_remove_member_for_desktop_session(
            desktop_session.session(),
            project_id,
            target_ref,
            group,
            member,
        )
    }
}

/// Tauri command wrapper for desktop fleet-owner `group_delete`.
pub mod tauri_group_delete {
    use super::{DesktopSessionState, GroupWriteOutcome, ProjectHandleOrLegacyProjectId, json};

    /// Delete a governed group as the signed-in desktop fleet-owner.
    #[tauri::command]
    pub fn group_delete(
        desktop_session: tauri::State<'_, DesktopSessionState>,
        project_id: ProjectHandleOrLegacyProjectId,
        target_ref: String,
        group: String,
    ) -> Result<GroupWriteOutcome, json::Error> {
        super::group_delete_for_desktop_session(
            desktop_session.session(),
            project_id,
            target_ref,
            group,
        )
    }
}

/// Tauri command wrapper for desktop fleet-owner `branch_gates_update`.
pub mod tauri_branch_gates_update {
    use super::{
        BranchGatesOutcome, BranchProtectionInput, DesktopSessionState,
        ProjectHandleOrLegacyProjectId, json,
    };

    /// Update branch gates as the signed-in desktop fleet-owner.
    #[tauri::command]
    pub fn branch_gates_update(
        desktop_session: tauri::State<'_, DesktopSessionState>,
        project_id: ProjectHandleOrLegacyProjectId,
        target_ref: String,
        branch: String,
        protection: BranchProtectionInput,
    ) -> Result<BranchGatesOutcome, json::Error> {
        super::branch_gates_update_for_desktop_session(
            desktop_session.session(),
            project_id,
            target_ref,
            branch,
            protection,
        )
    }
}
