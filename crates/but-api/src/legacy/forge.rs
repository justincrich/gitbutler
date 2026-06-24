//! In place of commands.rs
use anyhow::{Context as _, Result};
use but_api_macros::but_api;
use but_authz::{
    AssignmentState, Authority, AuthorizedAction, Denial, DenialClass, load_governance_config,
    resolve_principal_from_env, serialize_authority_tokens,
};
use but_core::{RepositoryExt, ref_metadata::ProjectMeta};
use but_ctx::{Context, ThreadSafeContext};
use but_forge::{
    ForgeName, ReviewTemplateFunctions, available_review_templates, get_review_template_functions,
};
use gitbutler_repo::{FileInfo, RepoCommands};
use serde::Serialize;
use tracing::instrument;

/// Structured forge-gate error payload for CLI and API callers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ForgeGateError {
    /// Stable consumer-facing error code.
    pub code: &'static str,
    /// Human-readable denial message.
    pub message: String,
    /// Steering classification — who can recover. Copied off the underlying
    /// [`but_authz::Denial`] (or defaulted to `ActorCorrectable` for
    /// `ConfigError`-sourced carriers) so the review CLI serializer
    /// (STEER-005) has a real field source.
    pub class: DenialClass,
    /// Authority tokens the principal already holds. Serialized as a
    /// stably-sorted array of `:`-token strings via
    /// [`but_authz::serialize_authority_tokens`]. Copied off the
    /// underlying [`but_authz::Denial`].
    #[serde(serialize_with = "serialize_authority_tokens")]
    pub held_permissions: Vec<Authority>,
    /// Recovery verbs the consumer may offer the actor. Copied off the
    /// underlying [`but_authz::Denial`].
    pub authorized_actions: Vec<AuthorizedAction>,
    /// Optional "do not" hint — verbs the actor must NOT attempt. Omitted
    /// entirely when `None` (no `null` key). Copied off the underlying
    /// [`but_authz::Denial`] / [`but_authz::ConfigError`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub do_not: Option<&'static str>,
}

/// Stable consumer-facing error code returned when an action requires the
/// remote-mirror path but `keep_reviews_local == false` and the mirror is a
/// NAMED SEAM ONLY (no mirroring code runs in LPR scope).
///
/// See `R21` in
/// `.spec/prds/governance/enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md §G`:
/// the mirror is deferred to a future sprint; while `keep_reviews_local == true`
/// (the default) the loop is fully local, and while `false` the API must surface
/// this stable code so callers can distinguish "preference says mirror" from
/// other failure modes.
pub const REMOTE_MIRROR_NOT_IMPLEMENTED_CODE: &str = "remote_mirror.not_implemented";

/// Structured payload for the LPR-006 named-seam error: a project preference
/// (`keep_reviews_local == false`) asks for the remote-mirror path, but the
/// mirror is not yet built.
///
/// Surfaces through [`classify_error`] as a [`ForgeGateError`] with code
/// [`REMOTE_MIRROR_NOT_IMPLEMENTED_CODE`]. The `Display` impl names the missing
/// mirror verbatim — that string is the named seam.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RemoteMirrorNotImplemented {
    /// Stable consumer-facing code (always
    /// [`REMOTE_MIRROR_NOT_IMPLEMENTED_CODE`]).
    pub code: &'static str,
    /// Human-readable explanation naming the missing mirror.
    pub message: String,
}

impl RemoteMirrorNotImplemented {
    /// Construct the canonical named-seam payload for `keep_reviews_local ==
    /// false` on a forge-mirror path.
    pub fn new() -> Self {
        Self {
            code: REMOTE_MIRROR_NOT_IMPLEMENTED_CODE,
            message: "remote review mirror not yet implemented (keep_reviews_local=false) \
                — set keep_reviews_local=true (the default) to keep agent reviews local"
                .to_owned(),
        }
    }
}

impl Default for RemoteMirrorNotImplemented {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for RemoteMirrorNotImplemented {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for RemoteMirrorNotImplemented {}

/// Extract a structured forge gate payload from an error chain.
///
/// Copies the four steering fields (`class`, `held_permissions`,
/// `authorized_actions`, `do_not`) off the underlying [`but_authz::Denial`]
/// (all four) or [`but_authz::ConfigError`] (`class` + `do_not` only) so the
/// review CLI serializer (STEER-005) has a real field source rather than a
/// two-key `{code,message}` flatten.
pub fn classify_error(err: &anyhow::Error) -> Option<ForgeGateError> {
    if let Some(denial) = err.downcast_ref::<but_authz::Denial>() {
        return Some(ForgeGateError {
            code: denial.code,
            message: denial.message.clone(),
            class: denial.class,
            held_permissions: denial.held_permissions.clone(),
            authorized_actions: denial.authorized_actions.clone(),
            do_not: denial.do_not,
        });
    }

    if let Some(mirror) = err.downcast_ref::<RemoteMirrorNotImplemented>() {
        // Named-seam error: the actor recovers by flipping
        // `keep_reviews_local` back to true, so `ActorCorrectable` is the
        // honest classification. No authority context applies to a named seam.
        // (Pre-existing STEER-001 incomplete-arm fix — same minimal patch as
        // the parallel LPR-008 branch; included here to unblock STEER-002
        // verification of forge_guard. STEER-002 scope is the table-driven
        // reconcile of `authorize_branch_action`, not this arm, but without
        // this fix `cargo check -p but-api` does not compile.)
        return Some(ForgeGateError {
            code: mirror.code,
            message: mirror.message.clone(),
            class: DenialClass::ActorCorrectable,
            held_permissions: Vec::new(),
            authorized_actions: Vec::new(),
            do_not: None,
        });
    }

    err.downcast_ref::<but_authz::ConfigError>()
        .map(|error| ForgeGateError {
            code: error.code(),
            message: error.to_string(),
            class: error.class.unwrap_or_default(),
            held_permissions: Vec::new(),
            authorized_actions: Vec::new(),
            do_not: error.do_not,
        })
}

fn branch_ref(branch: &str) -> String {
    if branch.starts_with("refs/") {
        branch.to_owned()
    } else {
        format!("refs/heads/{branch}")
    }
}

fn authorize_branch_action(
    repo: &gix::Repository,
    branch: &str,
    authority: Authority,
) -> Result<Option<but_authz::Principal>> {
    let ref_name = branch_ref(branch);
    if !but_authz::governance_present(repo, &ref_name)? {
        return Ok(None);
    }

    let cfg = load_forge_governance_config(repo, &ref_name)?;
    let principal = resolve_principal_from_env(&cfg)?;
    // STEER-002: the forge `authorize_branch_action` match is reconciled
    // with the ROUTE_AUTHORITY_TABLE rows in `but-authz`. The three
    // explicit arms below (ReviewsWrite / CommentsWrite / PullRequestsWrite)
    // correspond 1:1 to the `Route::ForgeReviewsWrite`,
    // `Route::ForgeCommentsWrite`, and `Route::ForgePullRequestsWrite` rows;
    // the `other =>` catch-all is preserved as defense-in-depth for any
    // future Authority variant so the function stays total, but every route
    // a caller actually drives is enumerated in the table. The literal
    // `authorize` call at each arm is preserved for the
    // AUTHORITY_POSITIVE_PATTERN honesty grep (forge.rs is outside the
    // grep's ENFORCEMENT_PATHS set today — RR-6 — so the safety of this
    // site rests on the behavior-neutral `forge_guard` test in AC-2).
    match authority {
        Authority::ReviewsWrite => {
            but_authz::authorize(&principal, but_authz::Authority::ReviewsWrite, &cfg)?
        }
        Authority::CommentsWrite => {
            but_authz::authorize(&principal, but_authz::Authority::CommentsWrite, &cfg)?
        }
        Authority::PullRequestsWrite => {
            but_authz::authorize(&principal, but_authz::Authority::PullRequestsWrite, &cfg)?;
        }
        other => but_authz::authorize(&principal, other, &cfg)?,
    }
    Ok(Some(principal))
}

fn load_forge_governance_config(
    repo: &gix::Repository,
    ref_name: &str,
) -> Result<but_authz::GovConfig> {
    match load_governance_config(repo, ref_name) {
        Ok(config) => Ok(config),
        Err(error) if is_single_file_governance(&error) => {
            Ok(but_authz::GovConfig::new([], [], []))
        }
        Err(error) => Err(error.into()),
    }
}

fn is_single_file_governance(error: &but_authz::ConfigError) -> bool {
    error
        .to_string()
        .starts_with("invalid governance config: missing ")
}

fn task_contract_invalid(action: &str, detail: impl AsRef<str>) -> anyhow::Error {
    anyhow::anyhow!(
        "{action} cannot report success: no downstream forge or local storage behavior exists for {}. Minimal spec repair: add a real provider/local persistence operation, or change this governed verb contract to return unsupported.",
        detail.as_ref()
    )
}

pub fn remote_url(project_meta: &ProjectMeta, repo: &gix::Repository) -> Result<String> {
    project_meta.remote_url_with_fallback(repo)
}

pub fn push_remote_url(project_meta: &ProjectMeta, repo: &gix::Repository) -> Result<String> {
    project_meta.push_remote_url(repo)
}

/// Read the per-project `keep_reviews_local` operator preference (LPR-006).
///
/// Returns `true` when agent reviews must stay local-only (the default via
/// `DefaultTrue`). Returns `false` only when the operator has explicitly opted
/// into the (deferred) remote-mirror path — see `R21` in
/// `.spec/prds/governance/enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md §G`.
/// This is a read-only convenience for the project-settings UI; the gate itself
/// lives in [`request_review`].
#[but_api(napi)]
#[instrument(err(Debug))]
pub fn get_keep_reviews_local(ctx: &Context) -> Result<bool> {
    Ok(*ctx.legacy_project.keep_reviews_local)
}

fn review_template_content(file: FileInfo) -> Result<String> {
    if file.size.is_none() {
        return Ok(String::new());
    }
    if !file.is_valid_utf8() {
        anyhow::bail!("PR template exists but must be valid UTF-8 text or markdown");
    }
    Ok(file.content.unwrap_or_default())
}

/// (Deprecated) Get the list of PR template paths for the given project and forge.
/// This function is deprecated in favor of `list_available_review_templates`.
#[but_api]
#[instrument(err(Debug))]
pub fn pr_templates(ctx: &but_ctx::Context, forge: ForgeName) -> Result<Vec<String>> {
    Ok(available_review_templates(&ctx.workdir_or_fail()?, &forge))
}

/// Get the forge provider name.
///
/// This is determined by the forge the base branch is pointing to.
#[but_api(napi)]
#[instrument(err(Debug))]
pub fn forge_provider(ctx: &Context) -> Result<Option<ForgeName>> {
    let project_meta = ctx.project_meta()?;
    let repo = ctx.repo.get()?;
    let forge_repo_info = but_forge::derive_forge_repo_info(&remote_url(&project_meta, &repo)?);
    Ok(forge_repo_info.map(|info| info.forge))
}

/// Per-project forge display + URL config. Lets the renderer build
/// commit/PR URLs and pick labels without branching on forge name.
#[but_api(napi)]
#[instrument(err(Debug))]
pub fn forge_info(ctx: &Context) -> Result<Option<but_forge::ForgeInfo>> {
    let project_meta = ctx.project_meta()?;
    let repo = ctx.repo.get()?;
    Ok(but_forge::forge_info(&remote_url(&project_meta, &repo)?))
}

/// Web compare URL for a branch — drives the "Open in browser"
/// affordances without making the renderer hold per-forge URL
/// templates. `fork` is the owner namespace for fork compares.
#[but_api(napi)]
#[instrument(err(Debug))]
pub fn forge_compare_branch_url(
    ctx: &Context,
    base: String,
    branch: String,
    fork: Option<String>,
) -> Result<Option<String>> {
    let project_meta = ctx.project_meta()?;
    let repo = ctx.repo.get()?;
    Ok(but_forge::compare_branch_url(
        &remote_url(&project_meta, &repo)?,
        &base,
        &branch,
        fork.as_deref(),
    ))
}

/// Get the list of review template paths for the given project.
#[but_api(napi)]
#[instrument(err(Debug))]
pub fn list_available_review_templates(ctx: &Context) -> Result<Vec<String>> {
    let project_meta = ctx.project_meta()?;
    let repo = ctx.repo.get()?;
    let forge_repo_info = but_forge::derive_forge_repo_info(&remote_url(&project_meta, &repo)?);
    let forge = &forge_repo_info
        .as_ref()
        .context("No forge could be determined for this repository branch")?
        .forge;

    Ok(available_review_templates(&ctx.workdir_or_gitdir()?, forge))
}

/// (Deprecated) Get the PR template content for the given project and relative path.
///
/// This function is deprecated in favor of `review_template`, which serves the same purpose
/// but uses the updated storage location.
#[but_api]
#[instrument(err(Debug))]
pub fn pr_template(
    ctx: &but_ctx::Context,
    relative_path: std::path::PathBuf,
    forge: ForgeName,
) -> Result<String> {
    let ReviewTemplateFunctions {
        is_valid_review_template_path,
        ..
    } = get_review_template_functions(&forge);

    if !is_valid_review_template_path(&relative_path) {
        return Err(anyhow::format_err!(
            "Invalid review template path: {:?}",
            ctx.workdir_or_fail()?.join(relative_path),
        ));
    }
    let file = ctx.read_file_from_workspace(&relative_path)?;
    review_template_content(file)
}

/// Information about the project's review template.
#[derive(Debug, Clone, serde::Serialize)]
#[cfg_attr(feature = "export-schema", derive(schemars::JsonSchema))]
pub struct ReviewTemplateInfo {
    /// The relative path to the review template within the repository.
    pub path: String,
    /// The content of the review template.
    pub content: String,
}

#[cfg(feature = "export-schema")]
but_schemars::register_sdk_type!(ReviewTemplateInfo);

/// Get the review template content for the given project and relative path.
///
/// This function determines the forge of a project and retrieves the review template
/// from the git config.
#[but_api(napi)]
#[instrument(err(Debug))]
pub fn review_template(ctx: &Context) -> Result<Option<ReviewTemplateInfo>> {
    let project_meta = ctx.project_meta()?;
    let repo = ctx.repo.get()?;
    let forge_repo_info = but_forge::derive_forge_repo_info(&remote_url(&project_meta, &repo)?);
    let forge = &forge_repo_info
        .as_ref()
        .context("No forge could be determined for this repository branch")?
        .forge;

    let repo = ctx.repo.get()?;
    match repo.git_settings()?.gitbutler_forge_review_template_path {
        Some(review_template_path) => {
            let ReviewTemplateFunctions {
                is_valid_review_template_path,
                ..
            } = get_review_template_functions(forge);
            let template_path = review_template_path.to_string();
            let path = std::path::PathBuf::from(&template_path);

            if !is_valid_review_template_path(&path) {
                return Err(anyhow::format_err!(
                    "Invalid review template path: {:?}",
                    ctx.workdir_or_fail()?.join(path),
                ));
            }
            let file = ctx.read_file_from_workspace(&path)?;
            let content = review_template_content(file)?;

            Ok(Some(ReviewTemplateInfo {
                path: template_path,
                content,
            }))
        }
        None => Ok(None),
    }
}

/// Set the review template path in the git configuration for the given project.
/// The template path will be validated.
#[but_api(napi)]
#[instrument(err(Debug))]
pub fn set_review_template(ctx: &but_ctx::Context, template_path: Option<String>) -> Result<()> {
    let repo = ctx.open_isolated_repo()?;
    let mut git_config = repo.git_settings()?;

    let project_meta = ctx.project_meta()?;
    let forge_repo_info = but_forge::derive_forge_repo_info(&remote_url(&project_meta, &repo)?);
    let forge = &forge_repo_info
        .as_ref()
        .context("No forge could be determined for this repository branch")?
        .forge;

    let ReviewTemplateFunctions {
        is_valid_review_template_path,
        ..
    } = get_review_template_functions(forge);

    if let Some(ref path) = template_path {
        let path_buf = std::path::PathBuf::from(path);
        if !is_valid_review_template_path(&path_buf) {
            let wd = ctx.workdir_or_fail()?.join(&path_buf);
            return Err(anyhow::format_err!("Invalid review template path: {wd:?}"));
        }
    }

    git_config.gitbutler_forge_review_template_path = template_path.map(|p| p.into());
    repo.set_git_settings(&git_config)
}

#[but_api(napi)]
#[instrument(err(Debug))]
pub fn list_reviews(
    ctx: &Context,
    cache_config: Option<but_forge::CacheConfig>,
) -> Result<Vec<but_forge::ForgeReview>> {
    let (storage, forge_repo_info, preferred_forge_user) = {
        let project_meta = ctx.project_meta()?;
        let repo = ctx.repo.get()?;
        let forge_repo_info = but_forge::derive_forge_repo_info(&remote_url(&project_meta, &repo)?);

        (
            but_forge_storage::Controller::from_path(but_path::app_data_dir()?),
            forge_repo_info,
            ctx.legacy_project.preferred_forge_user.clone(),
        )
    };

    let db = &mut *ctx.db.get_cache_mut()?;

    but_forge::list_forge_reviews_with_cache(
        preferred_forge_user,
        &forge_repo_info.context("No forge could be determined for this repository branch")?,
        &storage,
        db,
        cache_config,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_review_template_returns_empty_content() {
        let content =
            review_template_content(FileInfo::default()).expect("missing template is allowed");

        assert_eq!(content, "");
    }

    #[test]
    fn binary_review_template_errors_as_non_utf8() {
        let err = review_template_content(FileInfo::binary("PULL_REQUEST_TEMPLATE.md".as_ref(), 4))
            .expect_err("binary template must be rejected");

        assert_eq!(
            err.to_string(),
            "PR template exists but must be valid UTF-8 text or markdown"
        );
    }
}

#[but_api(napi)]
#[instrument(err(Debug))]
pub async fn get_review_base_repo_url(
    ctx: ThreadSafeContext,
    review_id: usize,
) -> Result<Option<String>> {
    let (storage, forge_repo_info, preferred_forge_user) = {
        let ctx = ctx.into_thread_local();
        let project_meta = ctx.project_meta()?;
        let repo = ctx.repo.get()?;
        let forge_repo_info = but_forge::derive_forge_repo_info(&remote_url(&project_meta, &repo)?);
        (
            but_forge_storage::Controller::from_path(but_path::app_data_dir()?),
            forge_repo_info,
            ctx.legacy_project.preferred_forge_user.clone(),
        )
    };
    but_forge::get_review_base_repo_url(
        &preferred_forge_user,
        &forge_repo_info.context("No forge could be determined for this repository branch")?,
        review_id,
        &storage,
    )
    .await
}

#[but_api(napi)]
#[instrument(err(Debug))]
pub async fn get_review_merge_status(
    ctx: ThreadSafeContext,
    review_id: usize,
) -> Result<but_forge::ReviewMergeStatus> {
    let (storage, forge_repo_info, preferred_forge_user) = {
        let ctx = ctx.into_thread_local();
        let project_meta = ctx.project_meta()?;
        let repo = ctx.repo.get()?;
        let forge_repo_info = but_forge::derive_forge_repo_info(&remote_url(&project_meta, &repo)?);
        (
            but_forge_storage::Controller::from_path(but_path::app_data_dir()?),
            forge_repo_info,
            ctx.legacy_project.preferred_forge_user.clone(),
        )
    };
    but_forge::get_review_merge_status(
        &preferred_forge_user,
        &forge_repo_info.context("No forge could be determined for this repository branch")?,
        review_id,
        &storage,
    )
    .await
}

#[but_api(napi)]
#[instrument(err(Debug))]
pub fn get_review(ctx: &Context, review_id: usize) -> Result<but_forge::ForgeReview> {
    let (storage, forge_repo_info, preferred_forge_user) = {
        let project_meta = ctx.project_meta()?;
        let repo = ctx.repo.get()?;
        let forge_repo_info = but_forge::derive_forge_repo_info(&remote_url(&project_meta, &repo)?)
            .context("No forge could be determined for this repository.")?;

        (
            but_forge_storage::Controller::from_path(but_path::app_data_dir()?),
            forge_repo_info,
            ctx.legacy_project.preferred_forge_user.clone(),
        )
    };

    let db = &mut *ctx.db.get_cache_mut()?;
    but_forge::get_forge_review(
        &preferred_forge_user,
        &forge_repo_info,
        review_id,
        db,
        &storage,
    )
}

#[but_api(napi)]
#[instrument(err(Debug))]
pub async fn get_repo_info(ctx: ThreadSafeContext) -> Result<but_forge::RepoInfo> {
    let (storage, forge_repo_info, preferred_forge_user) = {
        let ctx = ctx.into_thread_local();
        let project_meta = ctx.project_meta()?;
        let repo_ = ctx.repo.get()?;
        let forge_repo_info =
            but_forge::derive_forge_repo_info(&remote_url(&project_meta, &repo_)?);
        (
            but_forge_storage::Controller::from_path(but_path::app_data_dir()?),
            forge_repo_info,
            ctx.legacy_project.preferred_forge_user.clone(),
        )
    };
    but_forge::get_repo_info(
        &preferred_forge_user,
        &forge_repo_info.context("No forge could be determined for this repository branch")?,
        &storage,
    )
    .await
}

#[but_api(napi)]
#[instrument(skip(ctx), err(Debug))]
pub fn list_ci_checks(
    ctx: &Context,
    reference: String,
    cache_config: Option<but_forge::CacheConfig>,
) -> Result<Vec<but_forge::CiCheck>> {
    let (storage, forge_repo_info, preferred_forge_user) = {
        let project_meta = ctx.project_meta()?;
        let repo = ctx.repo.get()?;
        let forge_repo_info = but_forge::derive_forge_repo_info(&remote_url(&project_meta, &repo)?);

        (
            but_forge_storage::Controller::from_path(but_path::app_data_dir()?),
            forge_repo_info,
            ctx.legacy_project.preferred_forge_user.clone(),
        )
    };
    let db = &mut *ctx.db.get_cache_mut()?;

    but_forge::ci_checks_for_ref_with_cache(
        preferred_forge_user,
        &forge_repo_info.context("No forge could be determined for this repository branch")?,
        &storage,
        &reference,
        db,
        cache_config,
    )
}

#[but_api(napi)]
#[instrument(err(Debug))]
pub async fn publish_review(
    ctx: ThreadSafeContext,
    params: but_forge::CreateForgeReviewParams,
) -> Result<but_forge::ForgeReview> {
    let (storage, forge_repo_info, forge_push_repo_info, preferred_forge_user) = {
        let ctx = ctx.into_thread_local();
        let project_meta = ctx.project_meta()?;
        let repo = ctx.repo.get()?;
        authorize_branch_action(&repo, &params.source_branch, Authority::PullRequestsWrite)?;
        let base_remote_url = remote_url(&project_meta, &repo)?;
        let push_remote_url = push_remote_url(&project_meta, &repo)?;
        let forge_repo_info = but_forge::derive_forge_repo_info(&base_remote_url)
            .context("No forge could be determined for this repository branch")?;
        let forge_push_repo_info = if base_remote_url != push_remote_url {
            let info = but_forge::derive_forge_repo_info(&push_remote_url).context(
                "Failed to derive forge repository information from the push remote URL.",
            )?;
            Some(info)
        } else {
            None
        };

        (
            but_forge_storage::Controller::from_path(but_path::app_data_dir()?),
            forge_repo_info,
            forge_push_repo_info,
            ctx.legacy_project.preferred_forge_user.clone(),
        )
    };

    but_forge::create_forge_review(
        &preferred_forge_user,
        &forge_repo_info,
        &forge_push_repo_info,
        &params,
        &storage,
    )
    .await
}

/// Open a local review for a branch after enforcing `pull_requests:write`.
///
/// Writes the write-once `local_review_meta(target, "opener_principal", caller)`
/// row (R23 source-of-truth for the opener principal — NOT a comment-body
/// sentinel), and — when `reviewer` is `Some` — also seeds the first `pending`
/// `local_review_assignments` row for that reviewer. Assignments and verdicts
/// are separate: this verb never touches `local_review_verdicts`.
///
/// **LPR-006 gate:** the operator preference
/// [`Project::keep_reviews_local`](gitbutler_project::Project::keep_reviews_local)
/// controls where agent reviews land. When `true` (the default via `DefaultTrue`,
/// see `R21` in
/// `.spec/prds/governance/enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md §G`)
/// the loop is fully local — this verb writes only local-cache rows and never
/// reaches a forge. When `false`, the remote-mirror path is a NAMED SEAM ONLY:
/// this verb returns a structured [`RemoteMirrorNotImplemented`] error BEFORE
/// any authorization or write, so callers can distinguish "preference says
/// mirror" from a real forge call. Mirroring code is NOT built in LPR scope.
#[but_api(napi)]
#[instrument(err(Debug))]
pub async fn request_review(
    ctx: ThreadSafeContext,
    branch: String,
    reviewer: Option<String>,
) -> Result<()> {
    // LPR-006 R21 named seam: when the operator preference asks for the
    // remote-mirror path, surface the stable `remote_mirror.not_implemented`
    // error BEFORE authorization or any local-cache write. This is a gate, not
    // a path — no mirroring code runs in LPR scope. Access the public field on
    // `ThreadSafeContext` directly so we don't consume the only handle to it.
    if !*ctx.legacy_project.keep_reviews_local {
        return Err(anyhow::Error::new(RemoteMirrorNotImplemented::new()));
    }

    let ctx = ctx.into_thread_local();
    let repo = ctx.repo.get()?;
    let opener = authorize_branch_action(&repo, &branch, Authority::PullRequestsWrite)?
        .context("governance config is required to open a local review")?;

    let mut db = ctx.db.get_cache_mut()?;
    if let Some(reviewer) = reviewer {
        db.local_review_assignments_mut()
            .upsert(but_db::LocalReviewAssignment {
                id: uuid::Uuid::new_v4().to_string(),
                target: branch.clone(),
                reviewer_principal: reviewer,
                state: AssignmentState::Pending.name().to_owned(),
                assigned_at: chrono::Utc::now().naive_utc(),
            })?;
    }
    // Record the opener ONCE in the dedicated `local_review_meta` table. The
    // composite (target, "opener_principal") key plus ON CONFLICT DO NOTHING
    // makes this write-once — a later caller cannot overwrite the opener. LPR-005
    // derives the agent-PR tag from this opener's declared `kind` in committed
    // config; a comment-body sentinel would be attacker-influenceable (R20/R23).
    db.local_review_meta_mut()
        .upsert_if_absent(but_db::LocalReviewMeta {
            target: branch,
            key: "opener_principal".to_owned(),
            value: opener.id().as_str().to_owned(),
            created_at: chrono::Utc::now().naive_utc(),
        })?;

    Ok(())
}

/// Assign a reviewer to a branch review after enforcing `reviews:write`.
///
/// Enforces `reviewer != target_branch_author` BEFORE the upsert (R22 — the
/// drive-layer mirror of the gate's `require_distinct_from_author`). The target
/// author is the principal recorded as the opener in the write-once
/// `local_review_meta(target, "opener_principal")` row. Self-assignment is
/// rejected with a structured `perm.denied` Denial and NO row written.
#[but_api(napi)]
#[instrument(err(Debug))]
pub async fn assign_reviewer(
    ctx: ThreadSafeContext,
    branch: String,
    reviewer: String,
) -> Result<()> {
    let ctx = ctx.into_thread_local();
    let repo = ctx.repo.get()?;
    authorize_branch_action(&repo, &branch, Authority::ReviewsWrite)?
        .context("governance config is required to assign a reviewer")?;

    {
        let db = ctx.db.get_cache()?;
        if let Some(opener) = db.local_review_meta().get(&branch, "opener_principal")?
            && reviewer == opener.value
        {
            return Err(Denial::new(
                Denial::PERM_DENIED_CODE,
                format!(
                    "reviewer `{reviewer}` must be distinct from the target branch author `{}` (R22)",
                    opener.value
                ),
                "assign a reviewer principal distinct from the branch's opener/author"
                    .to_owned(),
            )
            .into());
        }
    }

    let mut db = ctx.db.get_cache_mut()?;
    db.local_review_assignments_mut()
        .upsert(but_db::LocalReviewAssignment {
            id: uuid::Uuid::new_v4().to_string(),
            target: branch,
            reviewer_principal: reviewer,
            state: AssignmentState::Pending.name().to_owned(),
            assigned_at: chrono::Utc::now().naive_utc(),
        })?;

    Ok(())
}

/// Approve a branch review locally after enforcing `reviews:write`.
#[but_api(napi)]
#[instrument(err(Debug))]
pub async fn approve_review(ctx: ThreadSafeContext, branch: String) -> Result<()> {
    let ctx = ctx.into_thread_local();
    let repo = ctx.repo.get()?;
    let principal = authorize_branch_action(&repo, &branch, Authority::ReviewsWrite)?
        .context("governance config is required to record a local review verdict")?;
    let ref_name = branch_ref(&branch);
    let head_oid = repo
        .find_reference(&ref_name)?
        .peel_to_commit()?
        .id
        .to_string();
    let mut db = ctx.db.get_cache_mut()?;
    db.local_review_verdicts_mut()
        .insert(but_db::LocalReviewVerdict {
            id: uuid::Uuid::new_v4().to_string(),
            target: branch,
            principal_id: principal.id().as_str().to_owned(),
            verdict: "approved".to_owned(),
            head_oid,
            created_at: chrono::Utc::now().naive_utc(),
        })?;

    Ok(())
}

/// Request changes on a branch review after enforcing `reviews:write`.
///
/// Sets the caller's `local_review_assignments.state` to `changes_requested`
/// via the typed `AssignmentState::ChangesRequested.name()` round-trip. The
/// assignment row is upserted (idempotent per `(target, reviewer_principal)`)
/// so the flip also works for a caller that has no prior pending assignment.
/// This is a drive-state write only — it never reaches the merge gate (the
/// gate reads `local_review_verdicts` at head, never assignment state).
#[but_api(napi)]
#[instrument(err(Debug))]
pub async fn request_changes_review(
    ctx: ThreadSafeContext,
    branch: String,
    _message: Option<String>,
) -> Result<()> {
    let ctx = ctx.into_thread_local();
    let repo = ctx.repo.get()?;
    let principal = authorize_branch_action(&repo, &branch, Authority::ReviewsWrite)?
        .context("governance config is required to request changes on a review")?;
    let principal_id = principal.id().as_str().to_owned();
    let changes_requested = AssignmentState::ChangesRequested.name().to_owned();

    let mut db = ctx.db.get_cache_mut()?;
    // Idempotent upsert keyed on (target, reviewer_principal): if a pending row
    // already exists for this caller it is updated in place; otherwise the
    // changes_requested state is recorded as the first assignment-state entry.
    db.local_review_assignments_mut()
        .upsert(but_db::LocalReviewAssignment {
            id: uuid::Uuid::new_v4().to_string(),
            target: branch,
            reviewer_principal: principal_id,
            state: changes_requested,
            assigned_at: chrono::Utc::now().naive_utc(),
        })?;

    Ok(())
}

/// Comment on a branch review after enforcing `comments:write`.
///
/// Delegates directly to [`post_comment`], which refuses the reserved
/// [`__pr_meta__`](RESERVED_PR_META_THREAD) thread id up front, authorizes the
/// actor on `comments:write`, and inserts the [`LocalReviewComment`] row. A
/// code comment carries `file = Some(..)` and `line = Some(..)`; a branch-level
/// comment carries `None` for both.
///
/// [`LocalReviewComment`]: but_db::LocalReviewComment
#[but_api(napi)]
#[instrument(err(Debug))]
pub async fn comment_review(
    ctx: ThreadSafeContext,
    branch: String,
    message: String,
    file: Option<String>,
    line: Option<i64>,
    thread_id: String,
) -> Result<()> {
    post_comment(ctx, branch, message, file, line, thread_id).await
}

/// Reserved thread id backed by [`local_review_meta`] rather than a real
/// comment row. The opener marker (agent-tag, source) lives in the dedicated
/// `local_review_meta` table, so a caller-supplied `__pr_meta__` thread_id is
/// rejected to prevent a comment-body sentinel from forging the opener (R23
/// negative control).
///
/// [`local_review_meta`]: but_db::LocalReviewMeta
pub const RESERVED_PR_META_THREAD: &str = "__pr_meta__";

/// Post a local review comment on a branch after enforcing `comments:write`.
///
/// Inserts a [`LocalReviewComment`] pinned to `resolved = false` and the current
/// timestamp. A code comment carries `file = Some(..)` and `line = Some(..)`; a
/// branch-level comment carries `None` for both. The reserved
/// [`__pr_meta__`](RESERVED_PR_META_THREAD) thread id is refused up front so a
/// comment-body sentinel cannot forge the opener marker. The comment `body` is
/// attacker-influenceable free text and is stored raw; bounding or escaping it
/// for downstream model consumption is an L2 harness concern (R20).
///
/// Modeled line-for-line on [`approve_review`]: authorize before any await, then
/// a single local-cache write. No `DryRun` guard — this touches only the local
/// cache, not refs / objects / oplog.
///
/// [`LocalReviewComment`]: but_db::LocalReviewComment
#[but_api(napi)]
#[instrument(err(Debug))]
pub async fn post_comment(
    ctx: ThreadSafeContext,
    branch: String,
    body: String,
    file: Option<String>,
    line: Option<i64>,
    thread_id: String,
) -> Result<()> {
    anyhow::ensure!(
        thread_id != RESERVED_PR_META_THREAD,
        "thread_id `{RESERVED_PR_META_THREAD}` is reserved and cannot receive comments"
    );
    let ctx = ctx.into_thread_local();
    let repo = ctx.repo.get()?;
    let author = authorize_branch_action(&repo, &branch, Authority::CommentsWrite)?
        .context("governance config is required to post a local review comment")?;
    let mut db = ctx.db.get_cache_mut()?;
    db.local_review_comments_mut()
        .insert(but_db::LocalReviewComment {
            id: uuid::Uuid::new_v4().to_string(),
            target: branch,
            author_principal: author.id().as_str().to_owned(),
            body,
            file,
            line,
            thread_id,
            resolved: false,
            created_at: chrono::Utc::now().naive_utc(),
        })?;
    Ok(())
}

/// List every comment on a branch, grouped by thread id in arrival order.
///
/// This is a **branch-scoped read** with no write authority: it discloses every
/// principal's threads on the named branch (an accepted branch-scoped
/// disclosure, F-006 — not per-principal self-scoping). The reserved
/// [`__pr_meta__`](RESERVED_PR_META_THREAD) marker thread is filtered out: the
/// opener lives in [`local_review_meta`], not a comment row.
///
/// [`local_review_meta`]: but_db::LocalReviewMeta
#[but_api(napi)]
#[instrument(err(Debug))]
pub async fn list_comments(
    ctx: ThreadSafeContext,
    branch: String,
) -> Result<Vec<but_db::LocalReviewComment>> {
    let ctx = ctx.into_thread_local();
    let db = ctx.db.get_cache()?;
    let rows = db
        .local_review_comments()
        .list_by_target(&branch)?
        .into_iter()
        .filter(|comment| comment.thread_id != RESERVED_PR_META_THREAD)
        .collect();
    Ok(rows)
}

/// Flip every comment in a thread to `resolved` after enforcing
/// `comments:write` AND a resolver-identity constraint (R22).
///
/// Beyond `comments:write`, the resolver must be:
/// - the **thread author** (the `author_principal` of a comment in that thread),
/// - the **assigned reviewer** (a `reviewer_principal` on the target in
///   `local_review_assignments`), or
/// - a holder of the higher `reviews:write` authority (folded from the same
///   committed config, never a parallel check).
///
/// A third unrelated principal is rejected with a structured denial and no row
/// flipped, so a single principal cannot post a `changes_requested`-style
/// thread and self-resolve it to forge a clean "all-clear" drive signal for
/// another party. The reserved [`__pr_meta__`](RESERVED_PR_META_THREAD) thread
/// is refused. The merge gate never reads this state — the resolved flag is a
/// drive signal only.
#[but_api(napi)]
#[instrument(err(Debug))]
pub async fn resolve_thread(
    ctx: ThreadSafeContext,
    branch: String,
    thread_id: String,
    resolved: bool,
) -> Result<()> {
    anyhow::ensure!(
        thread_id != RESERVED_PR_META_THREAD,
        "thread_id `{RESERVED_PR_META_THREAD}` is reserved and cannot be resolved"
    );
    let ctx = ctx.into_thread_local();
    let repo = ctx.repo.get()?;
    let resolver = authorize_branch_action(&repo, &branch, Authority::CommentsWrite)?
        .context("governance config is required to resolve a local review thread")?;
    let resolver_id = resolver.id();
    let holds_reviews_write = resolver.authorities().contains(Authority::ReviewsWrite);
    let mut db = ctx.db.get_cache_mut()?;
    let thread_authored = db
        .local_review_comments()
        .list_by_thread(&branch, &thread_id)?
        .iter()
        .any(|comment| comment.author_principal == resolver_id.as_str());
    let assigned_reviewer = db
        .local_review_assignments()
        .list_by_target(&branch)?
        .iter()
        .any(|assignment| assignment.reviewer_principal == resolver_id.as_str());
    if !(holds_reviews_write || thread_authored || assigned_reviewer) {
        return Err(anyhow::Error::new(but_authz::Denial::new(
            but_authz::Denial::PERM_DENIED_CODE,
            "only the thread author, the assigned reviewer, or a reviews:write holder may resolve this thread (R22)".to_owned(),
            "ask the thread author, an assigned reviewer, or a reviews:write holder to resolve the thread"
                .to_owned(),
        )));
    }
    db.local_review_comments_mut()
        .set_resolved(&thread_id, resolved)?;
    Ok(())
}

/// Close a branch review after enforcing `pull_requests:write`.
#[but_api(napi)]
#[instrument(err(Debug))]
pub async fn close_review(ctx: ThreadSafeContext, branch: String) -> Result<()> {
    let ctx = ctx.into_thread_local();
    let repo = ctx.repo.get()?;
    authorize_branch_action(&repo, &branch, Authority::PullRequestsWrite)?;
    Err(task_contract_invalid(
        "close_review",
        format!("branch `{branch}`"),
    ))
}

/// Derived PR lifecycle view computed at query time (read-only — no mutation).
///
/// Mirrors `enforce_merge_gate`'s read of `local_review_verdicts` at head
/// (`merge_gate.rs:84` → `review_requirement::evaluate`) but reads no authority,
/// writes no row, and is never consulted by the gate. The lifecycle label is
/// presentation-only; the merge decision still re-derives verdict-at-head
/// itself (§E safe seam — the load-bearing invariant).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "export-schema", derive(schemars::JsonSchema))]
pub struct ReviewStatus {
    /// The queried target ref (`refs/heads/<branch>`).
    pub target: String,
    /// Reviewer assignments on the target, in arrival order — every
    /// `local_review_assignments` row, including pending/approved/changes_requested.
    pub assignments: Vec<but_db::LocalReviewAssignment>,
    /// The subset of `assignments` still in the `pending` drive-state — i.e. the
    /// reviewers who haven't yet posted a verdict/changes-requested. This is the
    /// LPR-008 drive-state read an orchestrator polls to decide whom to nudge;
    /// it mirrors `assignments` and never reaches the merge gate (gates read
    /// verdicts only — §E safe seam).
    pub open_assignments: Vec<but_db::LocalReviewAssignment>,
    /// The verdict-at-head literal (`"approved"` / `"changes_requested"`) when a
    /// verdict row exists at the current HEAD, else `None`. The same input the
    /// merge gate re-derives itself.
    pub verdict_at_head: Option<String>,
    /// Presentation label derived from verdict + assignments:
    /// `Open` / `AwaitingReview` / `ChangesRequested` / `Approved`.
    pub lifecycle: String,
    /// `true` iff the opener principal's committed `permissions.toml` entry at
    /// the target ref declares `kind = "agent"` (read at the target ref like all
    /// governance config — never handle-resolution, never a comment body). The
    /// tag is descriptive only; no enforcement path reads it.
    pub agent_authored: bool,
    /// Count of unresolved comment threads (one per distinct `thread_id` where
    /// at least one comment has `resolved = false`).
    pub open_threads: usize,
    /// One representative comment per unresolved thread (the LPR-008 drive-state
    /// read for orchestrators). A thread is unresolved when any of its comments
    /// has `resolved = false`; the reserved `__pr_meta__` marker thread is
    /// excluded. `len()` is always equal to [`ReviewStatus::open_threads`].
    pub unresolved_threads: Vec<but_db::LocalReviewComment>,
}

#[cfg(feature = "export-schema")]
but_schemars::register_sdk_type!(ReviewStatus);

/// Query the derived PR lifecycle for a branch — READ-ONLY, no write authority.
///
/// The read model is computed at query time from the same inputs the merge gate
/// uses (verdict-at-head + open assignments + open threads), plus the opener
/// principal's declared `kind` in committed `permissions.toml` (read at the
/// target ref) for the `agent_authored` tag. The lifecycle derivation order:
///
/// - no assignments → `Open`
/// - a `changes_requested` verdict at head → `ChangesRequested`
/// - an `approved` verdict at head → `Approved`
/// - otherwise (assignments, no verdict at head) → `AwaitingReview`
///
/// This is branch-scoped drive-metadata: any caller on the project that can
/// name a branch sees the branch's full review surface (F-006 accepted
/// disclosure). It is **not** the per-principal self-scoping that
/// `governance_status_read` provides, and the merge gate **never** reads it.
#[but_api(napi)]
#[instrument(err(Debug))]
pub async fn review_status(ctx: ThreadSafeContext, branch: String) -> Result<ReviewStatus> {
    let ctx = ctx.into_thread_local();
    let repo = ctx.repo.get()?;
    let ref_name = branch_ref(&branch);

    // Current HEAD OID at the target ref — the merge-gate's own re-derivation
    // input. Filter the verdict rows to those pinned at this exact head.
    let head_oid = repo
        .find_reference(&ref_name)
        .with_context(|| format!("resolving {ref_name} for review_status"))?
        .peel_to_commit()
        .with_context(|| format!("peeling {ref_name} to a commit for review_status"))?
        .id
        .to_string();

    let db = ctx.db.get_cache()?;
    let assignments = db
        .local_review_assignments()
        .list_by_target(&branch)
        .context("listing local_review_assignments for review_status")?;
    let verdicts = db
        .local_review_verdicts()
        .list_by_target(&branch)
        .context("listing local_review_verdicts for review_status")?;
    let comments = db
        .local_review_comments()
        .list_by_target(&branch)
        .context("listing local_review_comments for review_status")?;

    // verdict_at_head: any verdict row pinned to the current HEAD; the literals
    // `approved` / `changes_requested` are the values approve_review and
    // request_changes_review already write. `changes_requested` wins ties so a
    // pending remediation is never hidden by an older approval at the same head.
    let verdict_at_head = derive_verdict_at_head(&verdicts, &head_oid);

    // open_threads + unresolved_threads: a thread is open if any of its
    // comments has resolved=false. The reserved __pr_meta__ marker thread is
    // filtered out (the opener lives in local_review_meta, not a comment row).
    // `unresolved_threads` carries one representative comment per open thread so
    // an orchestrator can drive on the branch's whole review surface (LPR-008
    // branch-scoped disclosure, F-006).
    let unresolved_threads = collect_unresolved_threads(&comments);
    let open_threads = unresolved_threads.len();

    // open_assignments: the subset of assignments still pending — i.e. reviewers
    // the orchestrator is still waiting on. Mirrors `assignments` and never
    // reaches the merge gate (the safe seam — gates read verdicts only).
    let pending_state = AssignmentState::Pending.name();
    let open_assignments = assignments
        .iter()
        .filter(|assignment| assignment.state == pending_state)
        .cloned()
        .collect();

    // agent_authored: derived from the opener principal's declared `kind` in the
    // committed permissions.toml at the target ref. Read at the target ref (not
    // the working tree), so an actor cannot self-escalate its own kind. Missing
    // permissions.toml / missing opener / omitted kind all default to false
    // (the conservative posture — a principal is agent only if config says so).
    let opener_principal = db
        .local_review_meta()
        .get(&branch, "opener_principal")
        .context("reading opener_principal meta for review_status")?
        .map(|row| row.value);
    let agent_authored = match opener_principal.as_deref() {
        Some(opener_id) => is_agent_opener(&repo, &ref_name, opener_id)?,
        None => false,
    };

    let lifecycle = derive_lifecycle(&assignments, verdict_at_head.as_deref());

    Ok(ReviewStatus {
        target: branch,
        assignments,
        open_assignments,
        verdict_at_head,
        lifecycle,
        agent_authored,
        open_threads,
        unresolved_threads,
    })
}

/// Pick the verdict-at-head literal from the rows, preferring
/// `changes_requested` over `approved` at the same head (a pending remediation
/// must not be hidden by a stale approval).
fn derive_verdict_at_head(
    verdicts: &[but_db::LocalReviewVerdict],
    head_oid: &str,
) -> Option<String> {
    let at_head: Vec<&str> = verdicts
        .iter()
        .filter(|verdict| verdict.head_oid == head_oid)
        .map(|verdict| verdict.verdict.as_str())
        .collect();
    if at_head.contains(&"changes_requested") {
        Some("changes_requested".to_owned())
    } else if at_head.contains(&"approved") {
        Some("approved".to_owned())
    } else {
        None
    }
}

/// Collect one representative [`LocalReviewComment`] per unresolved thread,
/// excluding the reserved `__pr_meta__` marker thread. A thread is unresolved
/// when any of its comments has `resolved = false`; the first such comment in
/// arrival order wins the representative slot. The returned `len()` is the
/// distinct open-thread count surfaced as [`ReviewStatus::open_threads`].
///
/// [`LocalReviewComment`]: but_db::LocalReviewComment
fn collect_unresolved_threads(
    comments: &[but_db::LocalReviewComment],
) -> Vec<but_db::LocalReviewComment> {
    use std::collections::HashSet;

    let mut seen: HashSet<&str> = HashSet::new();
    let mut open: Vec<but_db::LocalReviewComment> = Vec::new();
    for comment in comments {
        if comment.thread_id == RESERVED_PR_META_THREAD {
            continue;
        }
        if !comment.resolved && seen.insert(comment.thread_id.as_str()) {
            open.push(comment.clone());
        }
    }
    open
}

/// Derive the presentation lifecycle label from the verdict-at-head and the
/// assignment drive-state. `verdict_at_head` is sourced strictly from
/// `local_review_verdicts` filtered by `head_oid == HEAD`; the assignment state
/// (`pending` / `changes_requested` / `approved`) is the drive-state signal that
/// mirrors the verdict but never reaches the gate (the safe seam — gates read
/// verdicts only). Either signal can flip the lifecycle so an orchestrator that
/// drives via `request_changes_review` sees `ChangesRequested` even before a
/// reviewer posts a head-pinned verdict.
fn derive_lifecycle(
    assignments: &[but_db::LocalReviewAssignment],
    verdict_at_head: Option<&str>,
) -> String {
    if assignments.is_empty() {
        return "Open".to_owned();
    }
    // Verdict-at-head wins: a head-pinned verdict is the merge-gate input.
    if verdict_at_head == Some("changes_requested") {
        return "ChangesRequested".to_owned();
    }
    if verdict_at_head == Some("approved") {
        return "Approved".to_owned();
    }
    // Fall back to the assignment drive-state (mirrors the verdict but never
    // reaches the gate). `changes_requested` wins ties over `approved` so a
    // pending remediation is never hidden by an earlier approval.
    let any_assignment_changes_requested = assignments
        .iter()
        .any(|assignment| assignment.state == "changes_requested");
    let any_assignment_approved = assignments
        .iter()
        .any(|assignment| assignment.state == "approved");
    if any_assignment_changes_requested {
        return "ChangesRequested".to_owned();
    }
    if any_assignment_approved {
        return "Approved".to_owned();
    }
    "AwaitingReview".to_owned()
}

/// Read the opener principal's declared `kind` from the committed
/// `permissions.toml` at the target ref and return `true` iff that entry
/// declares `kind = "agent"`. Missing config / missing principal / omitted kind
/// all default to `false` (the conservative human-default posture).
fn is_agent_opener(repo: &gix::Repository, target_ref: &str, opener_id: &str) -> Result<bool> {
    let permissions = but_authz::load_permissions_wire(repo, target_ref)?;
    let is_agent = permissions
        .principal
        .iter()
        .find(|principal| principal.id == opener_id)
        .and_then(|principal| principal.kind.as_deref())
        == Some("agent");
    Ok(is_agent)
}

/// Merge a review on the forge.
#[but_api(napi)]
#[instrument(err(Debug))]
pub async fn merge_review(
    ctx: ThreadSafeContext,
    review_id: usize,
    merge_method: Option<but_forge::ReviewMergeMethod>,
) -> Result<()> {
    let (storage, forge_repo_info, preferred_forge_user) = {
        let ctx = ctx.into_thread_local();
        crate::legacy::merge_gate::enforce_merge_gate(&ctx, review_id)?;
        let project_meta = ctx.project_meta()?;
        let repo = ctx.repo.get()?;
        let forge_repo_info = but_forge::derive_forge_repo_info(&remote_url(&project_meta, &repo)?);

        (
            but_forge_storage::Controller::from_path(but_path::app_data_dir()?),
            forge_repo_info,
            ctx.legacy_project.preferred_forge_user.clone(),
        )
    };

    but_forge::merge_review(
        &preferred_forge_user,
        &forge_repo_info.context("No forge could be determined for this repository branch")?,
        review_id,
        merge_method,
        &storage,
    )
    .await
}

/// Enforce the governed merge gate for a review without calling the forge or persisting changes.
///
/// This is the API-boundary dry-run companion to [`merge_review`]. It deliberately returns
/// immediately after `merge_gate::enforce_merge_gate`, so callers can prove local governance would
/// allow the merge without touching network-backed forge state.
#[instrument(err(Debug))]
pub fn dry_run_merge_review(ctx: ThreadSafeContext, review_id: usize) -> Result<()> {
    let ctx = ctx.into_thread_local();
    crate::legacy::merge_gate::enforce_merge_gate(&ctx, review_id)
}

/// Enable or disable a review's auto-merge.
#[but_api(napi)]
#[instrument(err(Debug))]
pub async fn set_review_auto_merge(
    ctx: ThreadSafeContext,
    review_id: usize,
    enable: bool,
) -> Result<()> {
    let (storage, forge_repo_info, preferred_forge_user) = {
        let ctx = ctx.into_thread_local();
        crate::legacy::merge_gate::enforce_merge_gate(&ctx, review_id)?;
        let project_meta = ctx.project_meta()?;
        let repo = ctx.repo.get()?;
        let forge_repo_info = but_forge::derive_forge_repo_info(&remote_url(&project_meta, &repo)?);

        (
            but_forge_storage::Controller::from_path(but_path::app_data_dir()?),
            forge_repo_info,
            ctx.legacy_project.preferred_forge_user.clone(),
        )
    };

    but_forge::set_review_auto_merge_state(
        &preferred_forge_user,
        &forge_repo_info.context("No forge could be determined for this repository branch")?,
        review_id,
        enable,
        &storage,
    )
    .await
}

/// Set a review to draft or ready-for-review
#[but_api(napi)]
#[instrument(err(Debug))]
pub async fn set_review_draftiness(
    ctx: ThreadSafeContext,
    review_id: usize,
    draft: bool,
) -> Result<()> {
    let (storage, forge_repo_info, preferred_forge_user) = {
        let ctx = ctx.into_thread_local();
        let project_meta = ctx.project_meta()?;
        let repo = ctx.repo.get()?;
        let forge_repo_info = but_forge::derive_forge_repo_info(&remote_url(&project_meta, &repo)?);

        (
            but_forge_storage::Controller::from_path(but_path::app_data_dir()?),
            forge_repo_info,
            ctx.legacy_project.preferred_forge_user.clone(),
        )
    };

    but_forge::set_review_draftiness(
        &preferred_forge_user,
        &forge_repo_info.context("No forge could be determined for this repository branch")?,
        review_id,
        draft,
        &storage,
    )
    .await
}

/// Update arbitrary fields of a single review (body, state, target base).
/// Each `None` leaves that field unchanged on the forge.
#[but_api(napi)]
#[instrument(err(Debug))]
pub async fn update_review(
    ctx: ThreadSafeContext,
    review_id: usize,
    body: Option<String>,
    state: Option<but_forge::ReviewState>,
    target_base: Option<String>,
) -> Result<()> {
    let (storage, forge_repo_info, preferred_forge_user) = {
        let ctx = ctx.into_thread_local();
        let project_meta = ctx.project_meta()?;
        let repo = ctx.repo.get()?;
        let forge_repo_info = but_forge::derive_forge_repo_info(&remote_url(&project_meta, &repo)?);
        (
            but_forge_storage::Controller::from_path(but_path::app_data_dir()?),
            forge_repo_info,
            ctx.legacy_project.preferred_forge_user.clone(),
        )
    };

    but_forge::update_review(
        &preferred_forge_user,
        &forge_repo_info.context("No forge could be determined for this repository branch")?,
        review_id,
        body,
        state,
        target_base,
        &storage,
    )
    .await
}

/// Update stacked reviews: description footers and, optionally, target branches.
#[but_api(napi)]
#[instrument(err(Debug))]
pub async fn update_review_footers(
    ctx: ThreadSafeContext,
    reviews: Vec<but_forge::ForgeReviewUpdate>,
) -> Result<()> {
    let (storage, forge_repo_info, preferred_forge_user) = {
        let ctx = ctx.into_thread_local();
        let project_meta = ctx.project_meta()?;
        let repo = ctx.repo.get()?;
        let forge_repo_info = but_forge::derive_forge_repo_info(&remote_url(&project_meta, &repo)?);

        (
            but_forge_storage::Controller::from_path(but_path::app_data_dir()?),
            forge_repo_info,
            ctx.legacy_project.preferred_forge_user.clone(),
        )
    };

    but_forge::sync_reviews(
        &preferred_forge_user,
        &forge_repo_info.context("No forge could be determined for this repository branch")?,
        &reviews,
        &storage,
    )
    .await
}

#[but_api(napi)]
#[instrument(err(Debug))]
pub async fn list_reviews_for_branch(
    ctx: ThreadSafeContext,
    branch: String,
    filter: Option<but_forge::ForgeReviewFilter>,
) -> Result<Vec<but_forge::ForgeReview>> {
    let (storage, forge_repo_info, project) = {
        let ctx = ctx.into_thread_local();
        let project_meta = ctx.project_meta()?;
        let repo = ctx.repo.get()?;
        let forge_repo_info = but_forge::derive_forge_repo_info(&remote_url(&project_meta, &repo)?);
        (
            but_forge_storage::Controller::from_path(but_path::app_data_dir()?),
            forge_repo_info,
            ctx.legacy_project.clone(),
        )
    };

    but_forge::list_forge_reviews_for_branch(
        project.preferred_forge_user,
        &forge_repo_info.context("No forge could be determined for this repository branch")?,
        &branch,
        &storage,
        filter,
    )
    .await
}

/// Warm up the CI checks cache for all applied branches with PRs.
/// This function fetches CI check data from the forge and caches it in the database
/// without returning any data. It only processes branches that have associated pull requests.
/// Additionally, it cleans up stale CI check entries for references that are no longer
/// part of any applied stack.
#[but_api(napi)]
#[instrument(err(Debug))]
pub fn warm_ci_checks_cache(ctx: &Context) -> Result<()> {
    // Get all stacks
    let stacks = crate::legacy::workspace::stacks(ctx, None)?;

    // Collect branch references that have CI checks cached
    let mut current_refs = std::collections::HashSet::new();

    // For each stack, get details and check branches
    for stack in stacks {
        if let Some(stack_id) = stack.id {
            let details = crate::legacy::workspace::stack_details(ctx, Some(stack_id))?;

            // Process each branch that has a PR
            for branch in &details.branch_details {
                if branch.pr_number.is_some() {
                    // Fetch CI checks with NoCache to force refresh
                    let _ = list_ci_checks(
                        ctx,
                        branch.name.to_string(),
                        Some(but_forge::CacheConfig::NoCache),
                    );
                    // Ignore errors for individual branches to ensure we process all branches

                    // Track this reference as having CI checks
                    current_refs.insert(branch.name.to_string());
                }
            }
        }
    }

    // Clean up stale CI check entries from the database
    let db = &mut *ctx.db.get_cache_mut()?;
    let all_cached_refs = db.ci_checks().list_all_references()?;

    // Delete CI checks for references that are no longer in applied stacks
    for cached_ref in all_cached_refs {
        if !current_refs.contains(&cached_ref) {
            db.ci_checks_mut()?.delete_for_reference(&cached_ref)?;
        }
    }

    Ok(())
}
