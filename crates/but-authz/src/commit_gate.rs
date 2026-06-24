//! Ref-aware commit authorization gate.
//!
//! This is the single-source commit-gate primitive shared by every commit
//! mechanism. It lives in `but-authz` (not `but-api`) so lower-level,
//! agent-facing commit producers — e.g. the autonomous `but-action` handler —
//! can enforce the same gate without taking a dependency on `but-api`
//! (RULES.md: lower-level crates must not depend on `but-api`).
//!
//! The Context-aware entry point that resolves a [`RelativeTo`] selector to a
//! target ref (`enforce_commit_gate`) stays in `but-api`, because that
//! resolution needs `but_ctx::Context` + `but_workspace` graph traversal. It
//! delegates here once the target ref is known.

use gix::bstr::ByteSlice as _;
use gix::refs::FullName;

use crate::{
    AuthorizedAction, Denial, DenialCause, DenialClass, DenialPredicate, DeniedRoute, GovConfig,
    Principal, Route, authorized_actions, effective_authority, governance_present,
    load_governance_config, resolve_principal_from_env,
};

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

/// Enforce commit authorization for a resolved commit target.
///
/// If the target ref carries committed governance files, the gate reads that
/// governance configuration, resolves the acting principal from
/// `BUT_AGENT_HANDLE`, requires `contents:write`, and rejects direct commits to
/// protected branches before callers take write guards or mutate repository
/// state. Target refs with no committed governance files remain non-governed.
///
/// This is the **mechanism-agnostic** waist for commit authorization: every
/// commit producer (the `but-api` commit entry points, the `but-action`
/// autonomous handler, worktree/apply/integrate flows) routes through it so no
/// commit mechanism is an ungated path (UC-GATES-01 AC-5).
pub fn enforce_commit_gate_for_target(
    repo: &gix::Repository,
    target: &CommitGateTarget,
) -> anyhow::Result<()> {
    let full_name = target.config_ref.as_bstr().to_str()?;
    if !governance_present(repo, full_name)? {
        return Ok(());
    }

    let cfg = load_governance_config(repo, full_name).map_err(|config_error| {
        // STEER-007: operator_required config.invalid denial — fire the
        // observation-only telemetry event before propagating. The
        // authorized_actions menu is empty on this path.
        emit_denial_steering_event(
            config_error.code(),
            config_error.class.unwrap_or(DenialClass::OperatorRequired),
            &[],
        );
        anyhow::Error::from(config_error)
    })?;
    let principal = resolve_principal_from_env(&cfg)?;

    // STEER-002: Route::Commit row in ROUTE_AUTHORITY_TABLE supplies the
    // required Authority for this gate; the literal `crate::authorize` call is
    // preserved so the AUTHORITY_POSITIVE_PATTERN honesty grep keeps matching.
    // The branch-protection predicate below stays composed AROUND the table —
    // it is NOT folded in.
    let required = Route::Commit.required_authority();
    // STEER-004: enrich the authorize denial with a route-scoped menu
    // (ActorCorrectable path). The deny/allow decision is unchanged —
    // only the authorized_actions payload is additive.
    crate::authorize(&principal, required, &cfg).map_err(|denial| {
        let denial = denial.with_authorized_actions(
            &principal,
            &DeniedRoute::new(Route::Commit, DenialPredicate::Authority),
            &cfg,
        );
        // STEER-007: observation-only telemetry on the perm.denied path
        // (one event per denial). The event reads the enriched menu so
        // `had_lateral_action`/`menu_length` reflect the real payload.
        emit_denial_steering_event(denial.code, denial.class, &denial.authorized_actions);
        denial
    })?;

    if let Some(branch_name) = &target.protected_branch
        && cfg
            .branch(branch_name)
            .is_some_and(|branch| branch.protected())
    {
        return Err(branch_protected(&principal, &cfg, branch_name).into());
    }

    Ok(())
}

fn branch_protected(principal: &Principal, cfg: &GovConfig, branch_name: &str) -> Denial {
    // STEER-004: re-derive the effective authority set from the same cfg
    // the gate loaded (never re-loaded — same-cfg/ref by construction, M2).
    // The held set is dropped on authorize's Ok path today; re-deriving here
    // populates held_permissions on the branch.protected denial.
    let held = effective_authority(principal, cfg);

    // STEER-004/STEER-003: derive a gate-state-aware menu via
    // authorized_actions with the BranchProtected predicate (C5 subtraction:
    // the menu offers a feature-branch commit, NOT the protected-ref commit).
    let denied = DeniedRoute::new(Route::Commit, DenialPredicate::BranchProtected);
    let actions = authorized_actions(principal, &denied, cfg);

    let denial = Denial {
        code: "branch.protected",
        message: format!(
            "direct commits to protected branch \"{branch_name}\" are denied for principal \"{}\"; land changes through a reviewed merge",
            principal.id().as_str()
        ),
        remediation_hint: format!(
            "open a reviewed merge into {branch_name} instead of committing directly"
        ),
        class: DenialCause::BranchProtected.class(),
        held_permissions: held.iter().collect(),
        authorized_actions: actions,
        do_not: None,
    };
    // STEER-007: observation-only telemetry on the branch.protected path
    // (one event per denial). Fired once at the payload-build boundary.
    emit_denial_steering_event(denial.code, denial.class, &denial.authorized_actions);
    denial
}

// ---------------------------------------------------------------------------
// STEER-007 — denial-steering telemetry (observation-only)
// ---------------------------------------------------------------------------

/// Emit a structured denial-steering telemetry event on the existing
/// `tracing` path, carrying the four aggregate metrics operators use to
/// measure whether steering reduces hard-quits and loops:
///
/// - `code` — the stable denial code string (e.g. `branch.protected`).
/// - `class` — the [`DenialClass`] rendered as its stable snake_case token
///   (`actor_correctable` / `operator_required`).
/// - `had_lateral_action` — `true` iff `authorized_actions` carries at least
///   one entry that is NOT the always-appended discovery affordance
///   (`but perm list`). This is deliberately NOT `menu_length > 0`: a menu
///   consisting of only the discovery entry offers no lateral move.
/// - `menu_length` — `authorized_actions.len()`.
///
/// Observation-only: never alters the deny/allow decision, the exit code, or
/// any existing denial field. Fired exactly once per denial at the
/// payload-build boundary shared by the carriers (commit/merge/forge gates).
///
/// No principal-supplied or config-derived free text is logged — only the
/// stable code/class tokens and the two numeric/bool metrics (R15
/// injection-surface avoidance).
pub fn emit_denial_steering_event(
    code: &str,
    class: DenialClass,
    authorized_actions: &[AuthorizedAction],
) {
    let menu_length = authorized_actions.len();
    let had_lateral_action = authorized_actions
        .iter()
        .any(|action| action.command != crate::DISCOVERY_COMMAND);
    tracing::info!(
        code,
        class = class.name(),
        had_lateral_action,
        menu_length,
        "denial steering telemetry"
    );
}
