//! Governance command-boundary helpers for desktop fleet-owner identity.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
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
use but_ctx::{Context, ProjectHandleOrLegacyProjectId};
use serde::{Deserialize, Serialize};

const GATES_PATH: &str = ".gitbutler/gates.toml";
type PrincipalAuthorityRows = Vec<(PrincipalId, AuthoritySet)>;
type GroupRows = Vec<(GroupName, Group)>;

/// Read-only working-tree-vs-target-ref governance diff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GovernancePending {
    /// Per-principal effective authority comparison.
    pub principals: Vec<GovernancePendingPrincipal>,
    /// Number of authority tokens that differ between committed and working-tree config.
    pub pending_count: usize,
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

/// Return the read-only pending governance authority diff for a repository.
pub fn governance_pending_with_repo(
    repo: &gix::Repository,
    target_ref: &str,
) -> anyhow::Result<GovernancePending> {
    let committed = load_governance_config(repo, target_ref)?;
    let working = load_worktree_governance_config(repo)?;
    Ok(pending_diff(&committed, &working))
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

fn authority_names(authorities: &AuthoritySet) -> Vec<String> {
    authorities
        .iter()
        .map(|authority| authority.name().to_owned())
        .collect()
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
    let text = fs::read_to_string(&path)
        .with_context(|| format!("reading working-tree {}", permissions_path()))?;
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
    let text =
        fs::read_to_string(&path).with_context(|| format!("reading working-tree {GATES_PATH}"))?;
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
