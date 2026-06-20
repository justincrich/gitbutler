//! Governance command-boundary helpers for desktop fleet-owner identity.

use anyhow::{Context as _, anyhow};
use but_api::{json, legacy::governance::GrantOutcome};
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
pub trait DesktopSession {
    /// Resolve the signed-in desktop user that acts as the v1 fleet-owner.
    fn fleet_owner_identity(&self) -> anyhow::Result<FleetOwnerIdentity>;
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
    session: &impl DesktopSession,
    project_id: ProjectHandleOrLegacyProjectId,
    target_ref: &str,
) -> Result<(Context, FleetOwnerIdentity), json::Error> {
    let owner = session.fleet_owner_identity().map_err(json::Error::from)?;
    let ctx = context_for_project(project_id, target_ref).map_err(json::Error::from)?;
    Ok((ctx, owner))
}

/// Invoke `perm_grant` as the signed-in desktop fleet-owner.
pub fn perm_grant_for_desktop_session(
    session: &impl DesktopSession,
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
    use super::{GitButlerDesktopSession, GrantOutcome, ProjectHandleOrLegacyProjectId, json};

    /// Grant direct governance permissions as the signed-in desktop fleet-owner.
    #[tauri::command]
    pub fn perm_grant(
        project_id: ProjectHandleOrLegacyProjectId,
        target_ref: String,
        principal: String,
        authorities: Vec<String>,
    ) -> Result<GrantOutcome, json::Error> {
        super::perm_grant_for_desktop_session(
            &GitButlerDesktopSession,
            project_id,
            target_ref,
            principal,
            authorities,
        )
    }
}
