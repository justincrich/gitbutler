use bstr::ByteSlice as _;
use but_authz::{Authority, serialize_authority_tokens};
use but_authz::{AuthorizedAction, Denial, DenialClass, Principal, load_governance_config};
use but_rebase::graph_rebase::mutate::RelativeTo;
use gix::refs::FullName;
use serde::Serialize;

/// Structured commit-gate error payload for CLI and API callers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CommitGateError {
    /// Stable consumer-facing error code.
    pub code: &'static str,
    /// Human-readable denial message.
    pub message: String,
    /// Steering classification — who can recover. Populated by STEER-004;
    /// STEER-001 defaults to `DenialClass::ActorCorrectable`.
    pub class: DenialClass,
    /// Authority tokens the principal already holds. Serialized as a
    /// stably-sorted array of `:`-token strings via
    /// [`but_authz::serialize_authority_tokens`].
    #[serde(serialize_with = "serialize_authority_tokens")]
    pub held_permissions: Vec<Authority>,
    /// Recovery verbs the consumer may offer the actor.
    pub authorized_actions: Vec<AuthorizedAction>,
    /// Optional "do not" hint — verbs the actor must NOT attempt. Omitted
    /// entirely when `None` (no `null` key).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub do_not: Option<&'static str>,
}

/// Ref-backed authorization target for commit creation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitGateTarget {
    config_ref: FullName,
    protected_branch: Option<String>,
}

impl CommitGateTarget {
    /// Gate a direct commit to a branch ref, including branch protection.
    pub fn direct_ref(config_ref: FullName) -> Self {
        Self {
            protected_branch: Some(config_ref.shorten().to_string()),
            config_ref,
        }
    }

    /// Gate a commit operation using committed config from `config_ref`, with branch protection N/A.
    pub fn config_only(config_ref: FullName) -> Self {
        Self {
            config_ref,
            protected_branch: None,
        }
    }
}

/// Enforce commit authorization for a ref-aware commit target.
///
/// If the target ref carries committed governance files, the gate reads that
/// governance configuration, resolves the acting principal from
/// `BUT_AGENT_HANDLE`, requires `contents:write`, and rejects direct commits to
/// protected branches before callers take write guards or mutate repository
/// state. Target refs with no committed governance files remain non-governed.
pub fn enforce_commit_gate(ctx: &but_ctx::Context, relative_to: &RelativeTo) -> anyhow::Result<()> {
    let target = gate_target(ctx, relative_to)?;
    let repo = ctx.repo.get()?;
    enforce_commit_gate_for_target(&repo, &target)
}

/// Enforce commit authorization for a resolved commit target.
pub fn enforce_commit_gate_for_target(
    repo: &gix::Repository,
    target: &CommitGateTarget,
) -> anyhow::Result<()> {
    let full_name = target.config_ref.as_bstr().to_str()?;
    if !but_authz::governance_present(repo, full_name)? {
        return Ok(());
    }

    let cfg = load_governance_config(repo, full_name)?;
    let principal = but_authz::resolve_principal_from_env(&cfg)?;

    // STEER-002: Route::Commit row in ROUTE_AUTHORITY_TABLE supplies the
    // required Authority for this gate; the literal `but_authz::authorize`
    // call is preserved so the AUTHORITY_POSITIVE_PATTERN honesty grep
    // keeps matching. The branch-protection predicate below stays composed
    // AROUND the table — it is NOT folded in.
    let required = but_authz::Route::Commit.required_authority();
    but_authz::authorize(&principal, required, &cfg)?;

    if let Some(branch_name) = &target.protected_branch
        && cfg
            .branch(branch_name)
            .is_some_and(|branch| branch.protected())
    {
        return Err(branch_protected(&principal, branch_name).into());
    }

    Ok(())
}

/// Extract a structured gate payload from an error chain.
pub fn classify_error(err: &anyhow::Error) -> Option<CommitGateError> {
    if let Some(denial) = err.downcast_ref::<Denial>() {
        return Some(CommitGateError {
            code: denial.code,
            message: denial.message.clone(),
            class: denial.class,
            held_permissions: denial.held_permissions.clone(),
            authorized_actions: denial.authorized_actions.clone(),
            do_not: denial.do_not,
        });
    }

    err.downcast_ref::<but_authz::ConfigError>()
        .map(|error| CommitGateError {
            code: error.code(),
            message: error.to_string(),
            class: error.class.unwrap_or_default(),
            held_permissions: Vec::new(),
            authorized_actions: Vec::new(),
            do_not: error.do_not,
        })
}

fn gate_target(
    ctx: &but_ctx::Context,
    relative_to: &RelativeTo,
) -> anyhow::Result<CommitGateTarget> {
    match relative_to {
        RelativeTo::Reference(name) => {
            name.as_bstr().to_str()?;
            Ok(CommitGateTarget::direct_ref(name.clone()))
        }
        RelativeTo::Commit(commit_id) => {
            let head_info = head_info(ctx)?;
            let config_ref = find_branch_ref_for_commit(&head_info, *commit_id)?;
            Ok(CommitGateTarget::config_only(config_ref))
        }
    }
}

fn head_info(ctx: &but_ctx::Context) -> anyhow::Result<but_workspace::RefInfo> {
    let repo = ctx.clone_repo_for_merging_non_persisting()?;
    let meta = ctx.meta()?;
    but_workspace::head_info(
        &repo,
        &meta,
        but_workspace::ref_info::Options {
            project_meta: ctx.project_meta()?,
            traversal: but_graph::init::Options::limited(),
            expensive_commit_info: true,
            gerrit_mode: but_workspace::ref_info::GerritMode::Disabled,
        },
    )
    .map(|info| info.pruned_to_entrypoint())
}

fn find_branch_ref_for_commit(
    head_info: &but_workspace::RefInfo,
    commit_id: gix::ObjectId,
) -> anyhow::Result<FullName> {
    head_info
        .stacks
        .iter()
        .flat_map(|stack| stack.segments.iter())
        .find_map(|segment| {
            let contains_commit = segment.commits.iter().any(|commit| commit.id == commit_id)
                || segment
                    .commits_on_remote
                    .iter()
                    .any(|commit| commit.id == commit_id);
            contains_commit.then(|| {
                segment
                    .ref_info
                    .as_ref()
                    .map(|ref_info| ref_info.ref_name.clone())
                    .or_else(|| segment.remote_tracking_ref_name.clone())
            })?
        })
        .ok_or_else(|| {
            anyhow::anyhow!(
                "commit {} resolved without an owning branch ref for commit authorization",
                commit_id.to_hex_with_len(7)
            )
        })
}

fn branch_protected(principal: &Principal, branch_name: &str) -> Denial {
    Denial::new(
        "branch.protected",
        format!(
            "direct commits to protected branch \"{branch_name}\" are denied for principal \"{}\"; land changes through a reviewed merge",
            principal.id().as_str()
        ),
        format!("open a reviewed merge into {branch_name} instead of committing directly"),
    )
}
