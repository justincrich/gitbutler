//! Governance command-boundary helpers for desktop fleet-owner identity.

use anyhow::{Context as _, anyhow};
use but_api::{
    json,
    legacy::governance::{
        BranchGatesOutcome, BranchProtectionInput, GovernanceCommitOutcome, GovernancePending,
        GovernancePrincipalsList, GrantOutcome, GroupWriteOutcome, PermWriteOutcome,
    },
};
use but_ctx::{Context, ProjectHandleOrLegacyProjectId};

/// Minimal signed-in desktop user identity required for the fleet-owner path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FleetOwnerIdentity {
    /// Stable GitButler user id from the desktop session.
    pub user_id: u64,
    /// Optional user login, retained for diagnostics and future audit hooks.
    pub login: Option<String>,
}

impl FleetOwnerIdentity {
    fn principal_handle(&self) -> String {
        self.login
            .as_ref()
            .filter(|login| !login.is_empty())
            .cloned()
            .unwrap_or_else(|| format!("user:{}", self.user_id))
    }
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

/// Invoke `group_revoke` as the signed-in desktop fleet-owner.
pub fn group_revoke_for_desktop_session(
    session: &dyn DesktopSession,
    project_id: ProjectHandleOrLegacyProjectId,
    target_ref: String,
    group: String,
    authorities: Vec<String>,
) -> Result<GroupWriteOutcome, json::Error> {
    let (ctx, _owner) = fleet_owner_context(session, project_id, &target_ref)?;
    let repo = ctx.repo.get().map_err(json::Error::from)?;
    let authorities = authority_slices(&authorities);
    but_api::legacy::governance::group_revoke_with_repo_as_fleet_owner(
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

/// Invoke `branch_gates_read` as the signed-in desktop fleet-owner.
pub fn branch_gates_read_for_desktop_session(
    session: &dyn DesktopSession,
    project_id: ProjectHandleOrLegacyProjectId,
    target_ref: String,
) -> Result<BranchGatesOutcome, json::Error> {
    let (ctx, owner) = fleet_owner_context(session, project_id, &target_ref)?;
    let repo = ctx.repo.get().map_err(json::Error::from)?;
    but_api::legacy::governance::branch_gates_read_with_repo_as_principal(
        &repo,
        &target_ref,
        &owner.principal_handle(),
    )
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
    let (ctx, owner) = fleet_owner_context(session, project_id, &target_ref)?;
    let repo = ctx.repo.get().map_err(json::Error::from)?;
    but_api::legacy::governance::branch_gates_update_with_repo_as_principal(
        &repo,
        &target_ref,
        &owner.principal_handle(),
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
    but_api::legacy::governance::governance_pending(&ctx, target_ref).map_err(json::Error::from)
}

/// Invoke the read-only governance principals renderer contract.
pub fn governance_principals_list_for_project(
    project_id: ProjectHandleOrLegacyProjectId,
    target_ref: String,
) -> Result<GovernancePrincipalsList, json::Error> {
    let ctx = context_for_project(project_id, &target_ref).map_err(json::Error::from)?;
    but_api::legacy::governance::governance_principals_list(&ctx, target_ref)
        .map_err(json::Error::from)
}

/// Commit pending governance config files to the target ref.
pub fn governance_commit_for_desktop_session(
    session: &dyn DesktopSession,
    project_id: ProjectHandleOrLegacyProjectId,
    target_ref: String,
) -> Result<GovernanceCommitOutcome, json::Error> {
    let (ctx, _owner) = fleet_owner_context(session, project_id, &target_ref)?;
    let repo = ctx.repo.get().map_err(json::Error::from)?;
    but_api::legacy::governance::governance_commit_with_repo_as_fleet_owner(&repo, &target_ref)
        .map_err(json::Error::from)
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
    use super::{
        DesktopSessionState, GovernanceCommitOutcome, ProjectHandleOrLegacyProjectId, json,
    };

    /// Commit pending governance config files as the signed-in desktop fleet-owner.
    #[tauri::command]
    pub fn governance_commit(
        desktop_session: tauri::State<'_, DesktopSessionState>,
        project_id: ProjectHandleOrLegacyProjectId,
        target_ref: String,
    ) -> Result<GovernanceCommitOutcome, json::Error> {
        super::governance_commit_for_desktop_session(
            desktop_session.session(),
            project_id,
            target_ref,
        )
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

/// Tauri command wrapper for desktop fleet-owner `group_revoke`.
pub mod tauri_group_revoke {
    use super::{DesktopSessionState, GroupWriteOutcome, ProjectHandleOrLegacyProjectId, json};

    /// Revoke governed group permissions as the signed-in desktop fleet-owner.
    #[tauri::command]
    pub fn group_revoke(
        desktop_session: tauri::State<'_, DesktopSessionState>,
        project_id: ProjectHandleOrLegacyProjectId,
        target_ref: String,
        group: String,
        authorities: Vec<String>,
    ) -> Result<GroupWriteOutcome, json::Error> {
        super::group_revoke_for_desktop_session(
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

/// Tauri command wrapper for desktop fleet-owner `branch_gates_read`.
pub mod tauri_branch_gates_read {
    use super::{BranchGatesOutcome, DesktopSessionState, ProjectHandleOrLegacyProjectId, json};

    /// Read branch gates as the signed-in desktop fleet-owner.
    #[tauri::command]
    pub fn branch_gates_read(
        desktop_session: tauri::State<'_, DesktopSessionState>,
        project_id: ProjectHandleOrLegacyProjectId,
        target_ref: String,
    ) -> Result<BranchGatesOutcome, json::Error> {
        super::branch_gates_read_for_desktop_session(
            desktop_session.session(),
            project_id,
            target_ref,
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
