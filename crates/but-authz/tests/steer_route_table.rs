//! STEER-002 — `ROUTE_AUTHORITY_TABLE` single-source coverage proofs.
//!
//! These unit-tier tests prove the new `but_authz::ROUTE_AUTHORITY_TABLE`
//! symbol covers every gated route as a row, mapping each [`Route`] to its
//! required [`Authority`], the literal `but` command that drives it, and a
//! one-line effect description.
//!
//! The table is the single source of truth referenced by every gate's
//! required-authority lookup (`but_api::commit::gate`,
//! `but_api::legacy::merge_gate`, `but_api::legacy::config_mutate`, and
//! `but_api::legacy::forge`) AND by the menu module (STEER-003) for the
//! `usable = {r | r.required_authority ⊆ held}` derivation.
//!
//! See `.spec/prds/governance/tasks/sprint-08-steer-capability-aware-denials/
//! STEER-002-route-authority-table-single-source.md` for the task contract.

use but_authz::{Authority, ROUTE_AUTHORITY_TABLE, Route};

/// AC-1 / TC-1, TC-2, TC-3 — `ROUTE_AUTHORITY_TABLE` covers every gated route
/// with the right `Authority`, and is a single non-empty `but-authz` symbol.
#[test]
fn steer_route_table_covers_every_gated_route() {
    // The table is non-empty and covers the minimum expected route count.
    assert!(
        ROUTE_AUTHORITY_TABLE.len() >= 6,
        "ROUTE_AUTHORITY_TABLE must cover every gated route (>= 6 rows); got {}",
        ROUTE_AUTHORITY_TABLE.len()
    );

    // Every Route variant is present exactly once.
    for route in Route::ALL {
        let rows_for_route: Vec<_> = ROUTE_AUTHORITY_TABLE
            .iter()
            .filter(|(r, _, _, _)| r == route)
            .collect();
        assert_eq!(
            rows_for_route.len(),
            1,
            "Route {:?} must appear exactly once in ROUTE_AUTHORITY_TABLE; found {} rows",
            route,
            rows_for_route.len()
        );
    }

    // No duplicate authorities across the canonical six routes (each route
    // maps 1:1 to a distinct required Authority).
    let mut seen = std::collections::HashSet::new();
    for (_, authority, _, _) in ROUTE_AUTHORITY_TABLE {
        assert!(
            seen.insert(*authority),
            "duplicate Authority {authority:?} across ROUTE_AUTHORITY_TABLE rows — \
             each route must map 1:1 to a distinct required Authority"
        );
    }

    // Required authorities for the canonical four gates (commit/merge/admin
    // + forge) are present.
    let authorities: Vec<Authority> = ROUTE_AUTHORITY_TABLE
        .iter()
        .map(|(_, authority, _, _)| *authority)
        .collect();
    assert!(
        authorities.contains(&Authority::ContentsWrite),
        "ROUTE_AUTHORITY_TABLE must include a row mapping to Authority::ContentsWrite (commit route); got {authorities:?}"
    );
    assert!(
        authorities.contains(&Authority::Merge),
        "ROUTE_AUTHORITY_TABLE must include a row mapping to Authority::Merge (merge route); got {authorities:?}"
    );
    assert!(
        authorities.contains(&Authority::AdministrationWrite),
        "ROUTE_AUTHORITY_TABLE must include a row mapping to Authority::AdministrationWrite (admin route); got {authorities:?}"
    );

    println!(
        "ROUTE_AUTHORITY_TABLE has {} rows covering every gated Route variant",
        ROUTE_AUTHORITY_TABLE.len()
    );
    for (route, authority, command, effect) in ROUTE_AUTHORITY_TABLE {
        println!("- {route:?} -> {authority:?} (`{command}`): {effect}");
    }
}

/// AC-2 / TC-4 — the three forge routes (reviews:write, comments:write,
/// pull_requests:write) are explicit rows in `ROUTE_AUTHORITY_TABLE`,
/// reconciling the forge `authorize_branch_action` match (incl. its
/// `other =>` catch-all) into enumerable rows.
#[test]
fn steer_route_table_includes_forge_routes() {
    let forge_routes: Vec<Route> = ROUTE_AUTHORITY_TABLE
        .iter()
        .filter(|(_, authority, _, _)| {
            matches!(
                authority,
                Authority::ReviewsWrite | Authority::CommentsWrite | Authority::PullRequestsWrite
            )
        })
        .map(|(route, _, _, _)| *route)
        .collect();

    assert_eq!(
        forge_routes.len(),
        3,
        "ROUTE_AUTHORITY_TABLE must include exactly 3 forge routes (reviews:write, comments:write, pull_requests:write); got {forge_routes:?}"
    );

    // Each forge Authority is reachable through the table lookup helper.
    assert_eq!(
        Route::ForgeReviewsWrite.required_authority(),
        Authority::ReviewsWrite,
        "Route::ForgeReviewsWrite must resolve to Authority::ReviewsWrite through the table"
    );
    assert_eq!(
        Route::ForgeCommentsWrite.required_authority(),
        Authority::CommentsWrite,
        "Route::ForgeCommentsWrite must resolve to Authority::CommentsWrite through the table"
    );
    assert_eq!(
        Route::ForgePullRequestsWrite.required_authority(),
        Authority::PullRequestsWrite,
        "Route::ForgePullRequestsWrite must resolve to Authority::PullRequestsWrite through the table"
    );

    println!("ROUTE_AUTHORITY_TABLE enumerates every forge route explicitly");
}

/// AC-1 edge case — `Route::required_authority()` lookup is total: every
/// `Route::ALL` variant resolves to a row in `ROUTE_AUTHORITY_TABLE`. This
/// is the property STEER-003's menu derivation depends on (every route has
/// a discoverable required Authority).
#[test]
fn steer_route_required_authority_is_total() {
    for route in Route::ALL {
        let authority = route.required_authority();
        // Every route's looked-up authority is part of the Authority catalog.
        assert!(
            Authority::ALL.contains(&authority),
            "Route {route:?} resolved to {authority:?}, which is not in Authority::ALL"
        );
    }

    // Spot-check the canonical gates: commit, merge, admin.
    assert_eq!(Route::Commit.required_authority(), Authority::ContentsWrite);
    assert_eq!(Route::Merge.required_authority(), Authority::Merge);
    assert_eq!(
        Route::Admin.required_authority(),
        Authority::AdministrationWrite
    );
}

/// AC-1 — `Route::name()` is stable and unique across variants (the menu
/// module indexes routes by this string).
#[test]
fn steer_route_name_is_stable_and_unique() {
    let mut seen = std::collections::HashSet::new();
    for route in Route::ALL {
        let name = route.name();
        assert!(
            seen.insert(name),
            "duplicate Route::name() `{name}` — every Route variant needs a unique stable token"
        );
    }

    assert_eq!(Route::Commit.name(), "commit");
    assert_eq!(Route::Merge.name(), "merge");
    assert_eq!(Route::Admin.name(), "admin");
}

/// The table is a single `but-authz` symbol importable by both `but-api`
/// (the gate consumers) and the menu module (STEER-003). This is the
/// no-cycle proof: the table lives in `but-authz` (lower-level), so
/// `but-api` and downstream consumers can import it without creating a
/// `but-authz → but-api` cycle.
#[test]
fn steer_route_table_is_a_single_but_authz_symbol() {
    // The table is a `pub const` slice — proof it's a single symbol.
    let table: &[(Route, Authority, &str, &str)] = ROUTE_AUTHORITY_TABLE;
    assert!(
        !table.is_empty(),
        "ROUTE_AUTHORITY_TABLE must be a non-empty pub const slice in but-authz"
    );

    // Every row carries non-empty command and effect strings (the menu
    // module renders these for the operator).
    for (route, _, command, effect) in ROUTE_AUTHORITY_TABLE {
        assert!(
            !command.is_empty(),
            "Route {route:?} has an empty command string — every row must name the literal `but` command"
        );
        assert!(
            !effect.is_empty(),
            "Route {route:?} has an empty effect string — every row must describe its effect in one line"
        );
    }
}
