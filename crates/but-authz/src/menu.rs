//! STEER-003 gate-state-aware `authorized_actions` derivation.
//!
//! [`authorized_actions`] turns a [`DeniedRoute`] into a closed-catalog
//! recovery menu. The menu only surfaces actions whose required
//! [`Authority`] is already held by the denied caller (no lying menu),
//! draws every command/effect string from the [`CATALOG`] constants (no
//! interpolation), and excludes the denied route itself plus the
//! self-approve verb on the caller's own branch.
//!
//! Algorithm (per `03-technical-requirements-delta.md` §3):
//! 1. `held = effective_authority(principal, &cfg)`
//! 2. `usable = { r ∈ ROUTE_AUTHORITY_TABLE | r.required_authority ⊆ held }`
//! 3. `cands = AFFORDANCE_MAP[denied.category]`
//! 4. `scoped = { c ∈ cands | c ∈ usable AND c ≠ denied.route }`
//! 5. Render via [`CATALOG`] (each route maps to its [`CatalogEntry`] via
//!    [`route_to_catalog_entry`]).
//! 6. Append the discovery affordance ([`CATALOG`] entry for `but perm list`)
//!    when the discovery verb exists.
//! 7. Exclude self-approve (`but review approve`) when `is_own_branch`.

use crate::{Authority, AuthorizedAction, GovConfig, Principal, ReviewAction, Route};

use crate::ROUTE_AUTHORITY_TABLE;
use crate::effective_authority;

/// A category of denial — what route + predicate was denied.
///
/// Drives the affordance map: each category surfaces a different set of
/// candidate routes that *might* recover the denial, subject to the
/// caller's held authorities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DenialCategory {
    /// Commit denied — either `perm.denied` for missing `contents:write`,
    /// or `branch.protected`.
    CommitDenied,
    /// Merge denied — typically `gate.review_required`.
    MergeDenied,
    /// A forge review action was denied.
    ReviewDenied,
}

/// The denied context: which route, predicate category, and ref were rejected.
///
/// Constructed at the denial site (STEER-004) and handed to
/// [`authorized_actions`] to derive the recovery menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeniedRoute {
    /// Which family of denial this is — drives the affordance map.
    pub category: DenialCategory,
    /// The exact [`Route`] that was rejected (subtracted from candidates so
    /// the menu never offers "retry the exact thing you just tried").
    pub route: Route,
    /// Whether the denial was at a protected branch (commit-to-protected).
    pub is_branch_protected: bool,
    /// Whether the denial targets the caller's own branch (suppresses
    /// self-approve).
    pub is_own_branch: bool,
}

/// Affordance map: for each [`DenialCategory`], which [`Route`] categories
/// to surface.
///
/// Each entry names a [`Route`] in a SUCCEEDING context (never the denied
/// route — the implementation subtracts `denied.route` from the candidates
/// before rendering). The routes listed here are CANDIDATES — the caller
/// still must hold the route's required [`Authority`] for it to appear in
/// the menu.
pub const AFFORDANCE_MAP: &[(DenialCategory, &[Route])] = &[
    (
        DenialCategory::CommitDenied,
        &[
            Route::Commit,
            Route::Review(ReviewAction::RequestChanges),
            Route::Review(ReviewAction::Comment),
        ],
    ),
    (
        DenialCategory::MergeDenied,
        &[
            Route::Review(ReviewAction::RequestChanges),
            Route::Review(ReviewAction::Comment),
            Route::Review(ReviewAction::Approve),
        ],
    ),
    (
        DenialCategory::ReviewDenied,
        &[Route::Review(ReviewAction::Comment)],
    ),
];

/// A closed-catalog entry pairing a recovery command with its effect.
///
/// Every command and effect surfaced by [`authorized_actions`] MUST be a
/// reference into [`CATALOG`] — never `format!`-built, never interpolated,
/// never sourced from governance config. This is the "no lying menu"
/// invariant: the menu text is a fixed, audited alphabet.
#[derive(Debug, Clone, Copy)]
pub struct CatalogEntry {
    /// Canonical CLI/agent recovery verb (e.g. `but commit`).
    pub command: &'static str,
    /// Short human-readable description of the recovery effect.
    pub effect: &'static str,
}

/// Closed catalog of all command/effect text entries.
///
/// Every command and effect in [`authorized_actions`] MUST be a reference
/// into this table. Adding a new recovery verb requires adding a row here
/// AND wiring its [`Route`] correspondence in the internal
/// route-to-catalog lookup; the `steer_menu_text_is_closed_catalog_constants`
/// test enforces the closed-alphabet invariant.
pub const CATALOG: &[CatalogEntry] = &[
    CatalogEntry {
        command: "but commit",
        effect: "commit to a feature branch",
    },
    CatalogEntry {
        command: "but merge",
        effect: "merge a reviewed branch",
    },
    CatalogEntry {
        command: "but review request-changes",
        effect: "reject this change with line comments",
    },
    CatalogEntry {
        command: "but review comment",
        effect: "add a review comment",
    },
    CatalogEntry {
        command: "but review approve",
        effect: "approve this change",
    },
    CatalogEntry {
        command: "but perm list",
        effect: "see your permissions, groups, and authorized actions",
    },
];

/// Derive the recovery menu for a denial.
///
/// The result is ordered: route-derived affordances first (in
/// [`AFFORDANCE_MAP`] order), then the discovery affordance
/// (`but perm list`) last. Every entry's command and effect are
/// `&'static str` references into [`CATALOG`].
///
/// ```
/// # use but_authz::{
/// #     AuthoritySet, DenialCategory, DeniedRoute, GovConfig, Principal,
/// #     PrincipalId, Route, authorized_actions,
/// # };
/// # let held = AuthoritySet::parse(["reviews:write"]).unwrap();
/// # let principal = Principal::new(
/// #     PrincipalId::new("rev"),
/// #     held.clone(),
/// #     std::iter::empty::<but_authz::GroupName>(),
/// # );
/// # let cfg = GovConfig::new(
/// #     [(PrincipalId::new("rev"), held)],
/// #     [],
/// #     [],
/// # );
/// # let denied = DeniedRoute {
/// #     category: DenialCategory::CommitDenied,
/// #     route: Route::Commit,
/// #     is_branch_protected: false,
/// #     is_own_branch: false,
/// # };
/// let actions = authorized_actions(&principal, &denied, &cfg);
/// # let _ = actions;
/// ```
pub fn authorized_actions(
    principal: &Principal,
    denied: &DeniedRoute,
    cfg: &GovConfig,
) -> Vec<AuthorizedAction> {
    let held = effective_authority(principal, cfg);

    let cand_routes = affordances_for(denied.category);

    let mut actions = Vec::new();
    for cand_route in cand_routes {
        // C2 usable filter: the caller must hold the candidate route's
        // required authority. ROUTE_AUTHORITY_TABLE is the single source
        // of truth for route→authority, so this lookup cannot drift from
        // the gate sites.
        let Some(required) = required_authority_for(cand_route) else {
            continue;
        };
        if !held.contains(required) {
            continue;
        }

        // C5 subtraction: skip the denied route when in the same context —
        // never offer "retry the exact thing that just failed".
        if *cand_route == denied.route {
            continue;
        }

        // L1 self-approve exclusion: skip approve on own branch.
        if denied.is_own_branch && *cand_route == Route::Review(ReviewAction::Approve) {
            continue;
        }

        if let Some(entry) = route_to_catalog_entry(cand_route) {
            actions.push(AuthorizedAction {
                command: entry.command,
                effect: entry.effect,
            });
        }
    }

    // Append discovery affordance (degradable — only if the verb exists in
    // CATALOG). The discovery verb is always offered regardless of held
    // authorities: self-discovery is a read-only verb any principal can run.
    if let Some(discovery) = discovery_catalog_entry() {
        actions.push(AuthorizedAction {
            command: discovery.command,
            effect: discovery.effect,
        });
    }

    actions
}

/// Look up the [`DenialCategory`]'s candidate [`Route`] slice in
/// [`AFFORDANCE_MAP`]. Returns an empty slice for an unmapped category.
fn affordances_for(category: DenialCategory) -> &'static [Route] {
    for (cat, routes) in AFFORDANCE_MAP {
        if *cat == category {
            return routes;
        }
    }
    &[]
}

/// Look up the required [`Authority`] for a [`Route`] in
/// [`ROUTE_AUTHORITY_TABLE`]. Returns `None` for an unmapped route.
fn required_authority_for(route: &Route) -> Option<Authority> {
    ROUTE_AUTHORITY_TABLE
        .iter()
        .find(|(table_route, _, _, _)| table_route == route)
        .map(|(_, authority, _, _)| *authority)
}

/// Map a [`Route`] to its [`CatalogEntry`] in [`CATALOG`].
///
/// Returns `None` for [`Route::Admin`] (admin-write is operator-only and
/// has no recovery-menu entry) or any unmapped route.
fn route_to_catalog_entry(route: &Route) -> Option<&'static CatalogEntry> {
    let command = route_command(route)?;
    CATALOG.iter().find(|entry| entry.command == command)
}

/// The canonical CATALOG command string for a [`Route`], or `None` if the
/// route has no menu entry (e.g. [`Route::Admin`]).
fn route_command(route: &Route) -> Option<&'static str> {
    match route {
        Route::Commit => Some("but commit"),
        Route::Merge => Some("but merge"),
        Route::Review(ReviewAction::RequestChanges) => Some("but review request-changes"),
        Route::Review(ReviewAction::Comment) => Some("but review comment"),
        Route::Review(ReviewAction::Approve) => Some("but review approve"),
        Route::Admin => None,
    }
}

/// The discovery affordance's [`CatalogEntry`] (`but perm list`), if it
/// exists in [`CATALOG`].
fn discovery_catalog_entry() -> Option<&'static CatalogEntry> {
    CATALOG
        .iter()
        .find(|entry| entry.command == "but perm list")
}
