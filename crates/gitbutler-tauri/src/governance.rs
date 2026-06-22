//! Governance command-boundary helpers for desktop fleet-owner identity.

use anyhow::{Context as _, anyhow};
use but_api::{
    json,
    legacy::governance::{
        BranchGatesOutcome, BranchProtectionInput, GovernanceCommitOutcome, GovernancePending,
        GovernancePrincipalsList, GrantOutcome, GroupWriteOutcome, PermWriteOutcome,
        REF_PIN_CAVEAT,
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

/// Result of reading a principal's additive `kind` descriptor.
///
/// `kind` is the enforcement-neutral descriptor naming the principal's kind
/// (e.g. `"agent"` / `"human"`); it never enters `GovConfig.principals`. See
/// `but_authz::PrincipalWire::kind`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct PrincipalKindOutcome {
    /// Principal whose `kind` descriptor was read.
    pub principal: String,
    /// Current additive `kind` value, or `None` when unset (default-human).
    pub kind: Option<String>,
}

/// Result of staging a principal `kind` descriptor write to the pending config.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct PrincipalKindWriteOutcome {
    /// Principal whose `kind` descriptor was staged.
    pub principal: String,
    /// The `kind` value now staged in the pending working-tree config.
    pub kind: Option<String>,
    /// Ref-pin caveat: the staged descriptor takes effect once committed to the target branch.
    pub caveat: &'static str,
}

/// Read a principal's additive `kind` descriptor from the effective governance config.
///
/// Reads the working-tree `permissions.toml` when present (so the UI observes
/// its own pending edits), falling back to the committed target-ref blob. The
/// read is admin-scoped for governance-UI access, matching the read precedence
/// of the existing pending/commit governance commands.
pub fn get_principal_kind_for_project(
    project_id: ProjectHandleOrLegacyProjectId,
    target_ref: String,
    principal: String,
) -> Result<PrincipalKindOutcome, json::Error> {
    let ctx = context_for_project(project_id, &target_ref).map_err(json::Error::from)?;
    let repo = ctx.repo.get().map_err(json::Error::from)?;
    let permissions = effective_permissions(&repo, &target_ref).map_err(json::Error::from)?;
    let kind = permissions
        .principal
        .iter()
        .find(|entry| entry.id == principal)
        .and_then(|entry| entry.kind.clone());
    Ok(PrincipalKindOutcome { principal, kind })
}

/// Stage a principal `kind` descriptor write as the signed-in desktop fleet-owner.
///
/// The write lands in the working-tree `permissions.toml` (pending store) and
/// takes effect once `governance_commit` commits it to the target ref. The
/// fleet-owner identity establishes the unconditional administration-write
/// authority, matching the existing `perm_grant` desktop-session path; no
/// `BUT_AGENT_HANDLE` is consulted.
pub fn set_principal_kind_for_desktop_session(
    session: &dyn DesktopSession,
    project_id: ProjectHandleOrLegacyProjectId,
    target_ref: String,
    principal: String,
    kind: Option<String>,
) -> Result<PrincipalKindWriteOutcome, json::Error> {
    let (ctx, _owner) = fleet_owner_context(session, project_id, &target_ref)?;
    let repo = ctx.repo.get().map_err(json::Error::from)?;
    let mut permissions = effective_permissions(&repo, &target_ref).map_err(json::Error::from)?;
    let entry = principal_entry_mut(&mut permissions, &principal).map_err(json::Error::from)?;
    entry.kind = kind.clone();
    write_worktree_permissions(&repo, &permissions).map_err(json::Error::from)?;
    Ok(PrincipalKindWriteOutcome {
        principal,
        kind,
        caveat: REF_PIN_CAVEAT,
    })
}

/// Read the effective `permissions.toml` for a governance write: the working-tree
/// file when present (so pending edits are preserved), falling back to the
/// committed target-ref blob. Mirrors the `load_permissions_for_write` precedence
/// used by `but_api::legacy::governance`.
fn effective_permissions(
    repo: &gix::Repository,
    target_ref: &str,
) -> anyhow::Result<but_authz::PermissionsWire> {
    let path = worktree_permissions_path(repo)?;
    if path.is_file() {
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("reading working-tree {}", but_authz::permissions_path()))?;
        return parse_permissions_text(&text);
    }
    but_authz::load_permissions_wire(repo, target_ref)
}

fn parse_permissions_text(text: &str) -> anyhow::Result<but_authz::PermissionsWire> {
    if text.trim().is_empty() {
        return Ok(but_authz::PermissionsWire::default());
    }
    toml::from_str::<but_authz::PermissionsWire>(text)
        .with_context(|| format!("parsing working-tree {}", but_authz::permissions_path()))
}

/// Borrow the matching `[[principal]]` entry mutably, seeding a fresh entry
/// (mirrors `principal_entry_mut` in `but_api::legacy::governance`) when absent.
fn principal_entry_mut<'a>(
    permissions: &'a mut but_authz::PermissionsWire,
    principal: &str,
) -> anyhow::Result<&'a mut but_authz::PrincipalWire> {
    if let Some(position) = permissions
        .principal
        .iter()
        .position(|entry| entry.id == principal)
    {
        return permissions.principal.get_mut(position).ok_or_else(|| {
            anyhow!("principal position disappeared while preparing permissions rewrite")
        });
    }

    permissions.principal.push(but_authz::PrincipalWire {
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

fn write_worktree_permissions(
    repo: &gix::Repository,
    permissions: &but_authz::PermissionsWire,
) -> anyhow::Result<()> {
    let path = worktree_permissions_path(repo)?;
    let parent = path.parent().ok_or_else(|| {
        anyhow!(
            "{} must have a parent directory",
            but_authz::permissions_path()
        )
    })?;
    std::fs::create_dir_all(parent)
        .with_context(|| format!("creating working-tree {}", parent.display()))?;
    let encoded = toml::to_string(permissions)
        .with_context(|| format!("serializing working-tree {}", but_authz::permissions_path()))?;
    std::fs::write(&path, encoded)
        .with_context(|| format!("writing working-tree {}", but_authz::permissions_path()))?;
    Ok(())
}

fn worktree_permissions_path(repo: &gix::Repository) -> anyhow::Result<std::path::PathBuf> {
    let workdir = repo
        .workdir()
        .ok_or_else(|| anyhow!("governance permission writes require a non-bare repository"))?;
    Ok(workdir.join(but_authz::permissions_path()))
}

/// Tauri command wrapper for reading a principal `kind` descriptor.
pub mod tauri_get_principal_kind {
    use super::{PrincipalKindOutcome, ProjectHandleOrLegacyProjectId, json};

    /// Read a principal's additive `kind` descriptor from the effective governance config.
    #[tauri::command]
    pub fn get_principal_kind(
        project_id: ProjectHandleOrLegacyProjectId,
        target_ref: String,
        principal: String,
    ) -> Result<PrincipalKindOutcome, json::Error> {
        super::get_principal_kind_for_project(project_id, target_ref, principal)
    }
}

/// Tauri command wrapper for staging a principal `kind` descriptor write.
pub mod tauri_set_principal_kind {
    use super::{
        DesktopSessionState, PrincipalKindWriteOutcome, ProjectHandleOrLegacyProjectId, json,
    };

    /// Stage a principal `kind` descriptor write as the signed-in desktop fleet-owner.
    #[tauri::command]
    pub fn set_principal_kind(
        desktop_session: tauri::State<'_, DesktopSessionState>,
        project_id: ProjectHandleOrLegacyProjectId,
        target_ref: String,
        principal: String,
        kind: Option<String>,
    ) -> Result<PrincipalKindWriteOutcome, json::Error> {
        super::set_principal_kind_for_desktop_session(
            desktop_session.session(),
            project_id,
            target_ref,
            principal,
            kind,
        )
    }
}
