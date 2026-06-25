//! STEER-003 — gate-state-aware `authorized_actions` derivation.
//!
//! For any actor-correctable denial, [`authorized_actions`] derives a menu of
//! recovery verbs the caller is actually authorized to run. The derivation is:
//!
//! 1. `held = effective_authority(principal, cfg)` — the cfg the gate already
//!    loaded at the target ref, passed in (never re-loaded — same-cfg/ref by
//!    construction, M2).
//! 2. `usable = { r ∈ ROUTE_AUTHORITY_TABLE | r.required_authority ⊆ held }`.
//! 3. `cands = AFFORDANCE_MAP[denied]` — intent-relevant categories, each
//!    naming a route in a SUCCEEDING context (never the denied route at the
//!    denied ref).
//! 4. **C5 subtraction** — `scoped = { c ∈ cands | c.route ∈ usable AND c does
//!    NOT reproduce (route_d, predicate_d) at ref_d }`. For a `branch.protected`
//!    denial, the Commit route IS usable (the caller holds `contents:write`),
//!    so a pure authority check would re-offer the protected-ref commit. The
//!    [`AFFORDANCE_MAP`] prevents this by curating the Commit candidate as
//!    "commit on a feature branch" (a different, unprotected ref) — never the
//!    protected-ref commit that just failed.
//! 5. **L1 self-approve exclusion** — `but review approve` is excluded when the
//!    denial targets the caller's own branch.
//! 6. Render via [`CATALOG`] `&'static str` constants and append the degradable
//!    discovery affordance (`but perm list`).
//!
//! ## What this module is NOT
//!
//! The menu DERIVES FROM the denial — it never changes the deny/allow
//! decision. Every `command`/`effect` string is a closed-catalog `&'static str`
//! constant (invariant §9.2); no `format!`, interpolated, config-sourced, or
//! model-generated text.
//!
//! See `.spec/prds/governance/enrichments/v1.4.0-capability-aware-denials/
//! 03-technical-requirements-delta.md` §3 + §5 for the derivation pseudo-code
//! and the curated AFFORDANCE_MAP table.

use crate::{AuthorizedAction, GovConfig, Principal, Route, effective_authority};

// ---------------------------------------------------------------------------
// DenialPredicate + DeniedRoute — the derivation's denial inputs
// ---------------------------------------------------------------------------

/// The predicate that fired on a denied route.
///
/// This is the `predicate_d` in the `(route_d, predicate_d, ref_d)` tuple
/// STEER-004 threads into [`authorized_actions`]. It distinguishes a pure
/// authority failure (the caller lacks the required authority entirely) from a
/// gate-state failure (the caller holds the authority but a composed predicate
/// like branch-protection or review-requirement denied the action).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DenialPredicate {
    /// The principal lacked the required authority for the route
    /// (`perm.denied`).
    Authority,
    /// The target ref is branch-protected (`branch.protected`). The caller
    /// HOLDS the route's authority (otherwise `Authority` would have fired
    /// first), but the composed branch-protection predicate denied the action.
    BranchProtected,
    /// The review requirement is unmet (`gate.review_required`). The caller
    /// holds the merge authority but the review-requirement predicate denied
    /// the merge.
    ReviewRequired,
}

impl DenialPredicate {
    /// Return the stable token name for this predicate.
    ///
    /// ```
    /// use but_authz::DenialPredicate;
    ///
    /// assert_eq!(DenialPredicate::Authority.name(), "authority");
    /// assert_eq!(DenialPredicate::BranchProtected.name(), "branch_protected");
    /// assert_eq!(DenialPredicate::ReviewRequired.name(), "review_required");
    /// ```
    pub const fn name(self) -> &'static str {
        match self {
            Self::Authority => "authority",
            Self::BranchProtected => "branch_protected",
            Self::ReviewRequired => "review_required",
        }
    }
}

/// The denied route passed to [`authorized_actions`] — the `(route_d,
/// predicate_d)` tuple with an `own_branch` flag reserved for the L1
/// self-approve exclusion.
///
/// STEER-004 constructs this from the gate site's denial context and passes it
/// to [`authorized_actions`] alongside the cfg the gate already loaded.
///
/// **Current behavior (stronger than spec):** under the shipped
/// [`AFFORDANCE_MAP`] curation, `but review approve` is excluded from EVERY
/// denial menu unconditionally — no gate site calls [`DeniedRoute::with_own_branch`]
/// today. The `own_branch` flag is therefore inert defense-in-depth, retained
/// for a future per-branch policy that lifts the curatorial exclusion for
/// non-own-branch denials. The post-complete red-hat review recorded this
/// spec-vs-code reconciliation.
///
/// ```
/// use but_authz::{DeniedRoute, DenialPredicate, Route};
///
/// let denied = DeniedRoute::new(Route::Commit, DenialPredicate::BranchProtected);
/// assert_eq!(denied.route, Route::Commit);
/// assert!(!denied.own_branch);
///
/// let own = DeniedRoute::new(Route::Commit, DenialPredicate::Authority)
///     .with_own_branch(true);
/// assert!(own.own_branch);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeniedRoute {
    /// The route that was denied (e.g. [`Route::Commit`] for a commit gate
    /// denial).
    pub route: Route,
    /// The predicate that fired — authority failure, branch-protection, or
    /// review-requirement.
    pub predicate: DenialPredicate,
    /// Whether the denial targets the caller's OWN branch (the L1 self-approve
    /// exclusion fires when true, removing `but review approve` from the menu).
    pub own_branch: bool,
}

impl DeniedRoute {
    /// Create a denied route with `own_branch = false`.
    pub const fn new(route: Route, predicate: DenialPredicate) -> Self {
        Self {
            route,
            predicate,
            own_branch: false,
        }
    }

    /// Set the `own_branch` flag (the L1 self-approve exclusion fires when
    /// true).
    pub const fn with_own_branch(mut self, own_branch: bool) -> Self {
        self.own_branch = own_branch;
        self
    }
}

// ---------------------------------------------------------------------------
// CATALOG — closed &'static str command/effect pairs
// ---------------------------------------------------------------------------

/// The literal `but review approve` command — the self-approval verb excluded
/// on own-branch denials by the L1 exclusion.
const COMMAND_REVIEW_APPROVE: &str = "but review approve";

/// The closed catalog of authorized-action `(command, effect)` pairs.
///
/// Every menu entry rendered by [`authorized_actions`] resolves to a row here.
/// Adding a row requires code review — this is the "closed catalog" invariant
/// (§9.2): all `command`/`effect` strings are `&'static str` constants, never
/// `format!`, interpolated, config-sourced, or model-generated.
///
/// The catalog is keyed by the literal `but …` command; the internal
/// `catalog_lookup` helper finds the matching [`AuthorizedAction`] by command
/// string.
///
/// ```
/// use but_authz::CATALOG;
///
/// // Every catalog entry has a non-empty `but `-prefixed command and effect.
/// for action in CATALOG {
///     assert!(action.command.starts_with("but "));
///     assert!(!action.effect.is_empty());
/// }
/// ```
pub const CATALOG: &[AuthorizedAction] = &[
    // Commit affordance (Route::Commit / ContentsWrite).
    // The effect deliberately names an UNPROTECTED feature branch — this is the
    // C5 succeeding context for a branch.protected denial. NEVER "commit to the
    // protected branch".
    AuthorizedAction::new(
        "but commit",
        "create a commit on an unprotected feature branch ref",
    ),
    // Review affordances.
    AuthorizedAction::new(
        "but review request-changes",
        "request changes on a branch review",
    ),
    AuthorizedAction::new("but review comment", "post a review comment on a branch"),
    // `but review approve` is in the catalog so STEER-010 can grep for it
    // and so the L1 self-approve exclusion at line ~467 has a command token
    // to match against. Under the current `AFFORDANCE_MAP` curation it is
    // NEVER offered on ANY denial menu (no row names it as a candidate) —
    // self-approve is excluded unconditionally, which is stronger than the
    // spec's "exclude on own-branch" language. The `own_branch` flag below
    // is reserved for a future per-branch policy that lifts this curatorial
    // exclusion for non-own-branch denials; until then it stays inert and
    // `but review approve` stays out of every rendered menu.
    AuthorizedAction::new(
        COMMAND_REVIEW_APPROVE,
        "approve a branch review as a reviews:write holder",
    ),
    // Pull-request affordance (Route::ForgePullRequestsWrite).
    AuthorizedAction::new(
        "but review new",
        "open a local review for a branch to hand off for review",
    ),
    // Merge affordance (Route::Merge).
    AuthorizedAction::new(
        "but review merge",
        "merge a reviewed branch into the target",
    ),
    // Discovery affordance — self-scoped, always available to a resolved
    // principal. Appended (degradable) by authorized_actions.
    AuthorizedAction::new(
        "but perm list",
        "list your own effective permissions (self-discovery)",
    ),
];

/// The discovery affordance command (`but perm list`).
///
/// Public so the STEER-007 telemetry emission site in but-api can identify
/// the discovery entry when computing `had_lateral_action` — the metric is
/// `true` iff any menu entry is NOT this discovery affordance. Co-located
/// with [`CATALOG`] so the discovery identifier stays the single source of
/// truth (no string literal duplication in callers).
pub const DISCOVERY_COMMAND: &str = "but perm list";

/// Look up a catalog entry by its command string.
///
/// Returns `None` if the command is not in the closed catalog — this is the
/// degradability mechanism for the discovery affordance (if `but perm list`
/// were removed from the catalog, the menu would omit it rather than emit a
/// phantom command).
fn catalog_lookup(command: &str) -> Option<AuthorizedAction> {
    // `AuthorizedAction` holds two `&'static str` fields, so cloning is a
    // trivial copy of two pointers. (Copy is not derived on
    // AuthorizedAction because STEER-001 owns that type and this module is
    // STEER-003-scoped; .cloned() is the idiomatic accessor.)
    CATALOG
        .iter()
        .find(|action| action.command == command)
        .cloned()
}

// ---------------------------------------------------------------------------
// AFFORDANCE_MAP — curated denied-intent → succeeding-context categories
// ---------------------------------------------------------------------------

/// An entry in the curated [`AFFORDANCE_MAP`]: names a [`Route`] (in a
/// SUCCEEDING context) and the [`CATALOG`] command to render.
///
/// Each entry is curated so that running the named route at the named command
/// would NOT reproduce the denied `(route_d, predicate_d)` at `ref_d`. For a
/// `branch.protected` denial, the Commit affordance is "commit on a feature
/// branch" (a different, unprotected ref) — never the protected-ref commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Affordance {
    /// The route this affordance corresponds to, in a SUCCEEDING context.
    pub route: Route,
    /// The literal `but …` command from [`CATALOG`].
    pub command: &'static str,
}

impl Affordance {
    /// Create a curated affordance entry.
    pub const fn new(route: Route, command: &'static str) -> Self {
        Self { route, command }
    }
}

/// The curated denied-intent → affordance-category map (enrichment §5).
///
/// Each row maps a `(denied_route, predicate)` pair to a bounded set of
/// intent-relevant affordance categories, each naming a route **in a
/// succeeding context** — never the denied route at the denied ref. The map is
/// the one curated piece, sited beside [`crate::ROUTE_AUTHORITY_TABLE`] so
/// STEER-010's coverage grep can target it.
///
/// The `usable` filter in [`authorized_actions`] then keeps only the
/// affordances whose route the caller actually holds authority for, and the L1
/// self-approve exclusion removes `but review approve` on own-branch denials.
///
/// ```
/// use but_authz::{AFFORDANCE_MAP, DenialPredicate, Route};
///
/// // Every table route has at least one AFFORDANCE_MAP entry.
/// for route in Route::ALL {
///     let has_entry = AFFORDANCE_MAP
///         .iter()
///         .any(|(r, _, _)| *r == *route);
///     assert!(has_entry, "Route {:?} must have an AFFORDANCE_MAP entry", route);
/// }
/// ```
pub const AFFORDANCE_MAP: &[(Route, DenialPredicate, &[Affordance])] = &[
    // Commit on a protected branch (caller HOLDS contents:write):
    // offer commit-to-feature-branch + review (request-changes/comment — NOT
    // approve on own branch) + discovery. The Commit affordance is in a
    // succeeding context (feature branch), NOT the protected ref.
    (
        Route::Commit,
        DenialPredicate::BranchProtected,
        &[
            Affordance::new(Route::Commit, "but commit"),
            Affordance::new(Route::ForgeReviewsWrite, "but review request-changes"),
            Affordance::new(Route::ForgeCommentsWrite, "but review comment"),
        ],
    ),
    // Commit denied because caller lacks contents:write:
    // no commit affordance (the caller cannot commit anywhere); offer review
    // verbs they CAN run + discovery.
    (
        Route::Commit,
        DenialPredicate::Authority,
        &[
            Affordance::new(Route::ForgeReviewsWrite, "but review request-changes"),
            Affordance::new(Route::ForgeCommentsWrite, "but review comment"),
        ],
    ),
    // Merge denied because caller lacks merge authority:
    // offer review-status + hand-off (open a PR) + discovery.
    (
        Route::Merge,
        DenialPredicate::Authority,
        &[
            Affordance::new(Route::ForgeReviewsWrite, "but review request-changes"),
            Affordance::new(Route::ForgeCommentsWrite, "but review comment"),
            Affordance::new(Route::ForgePullRequestsWrite, "but review new"),
        ],
    ),
    // Merge denied because review requirement is unmet (caller HOLDS merge):
    // offer review-status (collect approvals / request changes) + discovery.
    (
        Route::Merge,
        DenialPredicate::ReviewRequired,
        &[
            Affordance::new(Route::ForgeReviewsWrite, "but review request-changes"),
            Affordance::new(Route::ForgeCommentsWrite, "but review comment"),
        ],
    ),
    // Submit-review denied because caller lacks reviews:write:
    // offer comment (if held) + PR verbs + discovery.
    (
        Route::ForgeReviewsWrite,
        DenialPredicate::Authority,
        &[
            Affordance::new(Route::ForgeCommentsWrite, "but review comment"),
            Affordance::new(Route::ForgePullRequestsWrite, "but review new"),
        ],
    ),
    // Comment denied because caller lacks comments:write:
    // offer review verbs (if held) + PR verbs + discovery.
    (
        Route::ForgeCommentsWrite,
        DenialPredicate::Authority,
        &[
            Affordance::new(Route::ForgeReviewsWrite, "but review request-changes"),
            Affordance::new(Route::ForgePullRequestsWrite, "but review new"),
        ],
    ),
    // PR verb denied because caller lacks pull_requests:write:
    // offer review verbs + comment (if held) + discovery.
    (
        Route::ForgePullRequestsWrite,
        DenialPredicate::Authority,
        &[
            Affordance::new(Route::ForgeReviewsWrite, "but review request-changes"),
            Affordance::new(Route::ForgeCommentsWrite, "but review comment"),
        ],
    ),
    // Admin write denied because caller lacks administration:write:
    // no write affordances (the caller cannot self-correct an admin denial
    // in-system); discovery only.
    (Route::Admin, DenialPredicate::Authority, &[]),
];

/// Look up the intent-relevant affordance categories for a denied
/// `(route, predicate)` pair.
///
/// Returns an empty slice if the pair is not in [`AFFORDANCE_MAP`] — the
/// caller still gets the appended discovery affordance.
fn affordance_candidates(route: Route, predicate: DenialPredicate) -> &'static [Affordance] {
    for (mapped_route, mapped_predicate, candidates) in AFFORDANCE_MAP {
        if *mapped_route == route && *mapped_predicate == predicate {
            return candidates;
        }
    }
    &[]
}

// ---------------------------------------------------------------------------
// authorized_actions — the gate-state-aware derivation
// ---------------------------------------------------------------------------

/// Derive the menu of authorized recovery actions for an actor-correctable
/// denial.
///
/// The derivation intersects the caller's effective authority set with
/// [`crate::ROUTE_AUTHORITY_TABLE`], intent-scopes via [`AFFORDANCE_MAP`],
/// subtracts the failed `(route_d, predicate_d)` (the C5 subtraction — for
/// `branch.protected`, offers a commit to a DIFFERENT unprotected feature ref,
/// never the protected-ref commit), excludes `but review approve` on
/// own-branch denials (the L1 self-approve exclusion), renders every entry
/// from the closed [`CATALOG`], and appends the degradable discovery
/// affordance (`but perm list`).
///
/// The cfg is the one the gate already loaded at the target ref — passed in,
/// never re-loaded (same-cfg/ref by construction, M2). This function is
/// behavior-neutral for the deny/allow decision; it derives FROM the denial,
/// never changes it.
///
/// # Example
///
/// ```
/// use but_authz::{
///     AuthoritySet, DeniedRoute, DenialPredicate, GovConfig, Principal,
///     PrincipalId, Route, authorized_actions,
/// };
///
/// # fn main() -> Result<(), but_authz::ParseAuthorityError> {
/// let principal = Principal::new(
///     PrincipalId::new("dev"),
///     AuthoritySet::parse(["contents:read", "comments:write"])?,
///     [],
/// );
/// let cfg = GovConfig::new(
///     [(PrincipalId::new("dev"), AuthoritySet::parse(["contents:read", "comments:write"])?)],
///     [],
///     [],
/// );
/// // dev tried to commit but lacks contents:write.
/// let denied = DeniedRoute::new(Route::Commit, DenialPredicate::Authority);
/// let actions = authorized_actions(&principal, &denied, &cfg);
///
/// // Menu includes comment (held) + discovery, excludes commit (unheld).
/// let commands: Vec<&str> = actions.iter().map(|a| a.command).collect();
/// assert!(commands.contains(&"but review comment"));
/// assert!(commands.contains(&"but perm list"));
/// assert!(!commands.contains(&"but commit")); // contents:write not held
/// # Ok(())
/// # }
/// ```
pub fn authorized_actions(
    principal: &Principal,
    denied: &DeniedRoute,
    cfg: &GovConfig,
) -> Vec<AuthorizedAction> {
    // Step 1: held = effective_authority(principal, cfg).
    let held = effective_authority(principal, cfg);

    // Step 2: usable = { r ∈ ROUTE_AUTHORITY_TABLE | r.required_authority ⊆ held }.
    //   A route is usable when the caller holds its required authority. For
    //   Authority-predicate denials this naturally excludes the denied route
    //   (the caller lacks its authority). For BranchProtected denials the
    //   denied route IS usable (the caller holds contents:write) — the C5
    //   subtraction is handled by the AFFORDANCE_MAP's succeeding-context
    //   curation below.
    let usable: Vec<Route> = Route::ALL
        .iter()
        .copied()
        .filter(|route| held.contains(route.required_authority()))
        .collect();

    // Step 3: cands = AFFORDANCE_MAP[denied].
    let candidates = affordance_candidates(denied.route, denied.predicate);

    // Steps 4–5: filter by usable + C5 subtraction + L1 self-approve exclusion.
    let mut actions = Vec::new();
    for candidate in candidates {
        // The candidate's route must be usable (the caller holds its authority).
        if !usable.contains(&candidate.route) {
            continue;
        }

        // C5 subtraction: the candidate must NOT reproduce the denied
        // (route_d, predicate_d) at ref_d. For Authority predicates this is
        // already handled by `usable` (the denied route is not usable). For
        // BranchProtected, the AFFORDANCE_MAP's Commit candidate is curated as
        // "commit on a feature branch" (a different, unprotected ref) — it does
        // NOT reproduce the protected-ref denial. STEER-010's coverage grep
        // enforces this curation at build time.

        // L1 self-approve exclusion: exclude `but review approve` when the
        // denial targets the caller's own branch (security HIGH #3). This is a
        // code contract, never deferred to the L2 primer.
        if denied.own_branch && candidate.command == COMMAND_REVIEW_APPROVE {
            continue;
        }

        // Render from CATALOG — every entry is a closed-catalog &'static str.
        if let Some(action) = catalog_lookup(candidate.command) {
            actions.push(action);
        }
    }

    // Step 6: append the degradable discovery affordance (`but perm list`).
    // Degrgradable: if the discovery verb is not in CATALOG, it is omitted
    // rather than emitting a phantom command (preserving no-lying-menu, C3/D8).
    if let Some(discovery) = catalog_lookup(DISCOVERY_COMMAND) {
        actions.push(discovery);
    }

    actions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AuthoritySet, PrincipalId};

    #[test]
    fn catalog_every_command_starts_with_but_and_has_nonempty_effect() {
        for action in CATALOG {
            assert!(
                action.command.starts_with("but "),
                "CATALOG command {:?} must start with \"but \"",
                action.command
            );
            assert!(
                !action.effect.is_empty(),
                "CATALOG effect for {:?} must be non-empty",
                action.command
            );
        }
    }

    #[test]
    fn affordance_map_covers_every_route() {
        for route in Route::ALL {
            let has_entry = AFFORDANCE_MAP.iter().any(|(r, _, _)| r == route);
            assert!(
                has_entry,
                "Route::{route:?} must have at least one AFFORDANCE_MAP entry"
            );
        }
    }

    #[test]
    fn authorized_actions_for_admin_denial_is_discovery_only() {
        let principal = Principal::new(PrincipalId::new("dev"), AuthoritySet::empty(), []);
        let cfg = GovConfig::new([], [], []);
        let denied = DeniedRoute::new(Route::Admin, DenialPredicate::Authority);
        let actions = authorized_actions(&principal, &denied, &cfg);

        // Admin denial with no held authorities → discovery only.
        assert_eq!(
            actions.len(),
            1,
            "admin denial with no authority yields discovery only"
        );
        assert_eq!(actions[0].command, DISCOVERY_COMMAND);
    }
}
