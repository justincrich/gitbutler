//! STEER-002: route/authority table coverage.
//!
//! The table in `but_authz::ROUTE_AUTHORITY_TABLE` is the single source of
//! truth that maps every gated `Route` to its required `Authority`, the
//! consumer-facing command string, and the effect string. These tests assert
//! the table covers every route that already has a live gate site, including
//! the three forge-dispatched review authorities.

use but_authz::{Authority, ROUTE_AUTHORITY_TABLE, ReviewAction, Route};

/// Collect every `Authority` referenced by the table — used to assert that
/// the forge-dispatched review authorities (Reviews/Comments/PullRequests) are
/// each present at least once.
fn table_authorities() -> Vec<Authority> {
    ROUTE_AUTHORITY_TABLE
        .iter()
        .map(|(_, auth, _, _)| *auth)
        .collect()
}

#[test]
fn steer_route_table_covers_every_gated_route() {
    // The table must cover the four route families (Commit, Merge, the three
    // Review actions, Admin) — at least six rows.
    assert!(
        ROUTE_AUTHORITY_TABLE.len() >= 6,
        "ROUTE_AUTHORITY_TABLE must enumerate at least six gated routes; got {}",
        ROUTE_AUTHORITY_TABLE.len()
    );

    let authorities = table_authorities();
    assert!(
        authorities.contains(&Authority::Merge),
        "ROUTE_AUTHORITY_TABLE must include a Merge row (the merge gate is live)"
    );
    assert!(
        authorities.contains(&Authority::ContentsWrite),
        "ROUTE_AUTHORITY_TABLE must include a ContentsWrite row (the commit gate is live)"
    );
    assert!(
        authorities.contains(&Authority::AdministrationWrite),
        "ROUTE_AUTHORITY_TABLE must include an AdministrationWrite row (the admin-write gate is live)"
    );

    // Every row's Route must agree with its required Authority on the route
    // family — this is the single-source invariant that lets menu derivation
    // trust the table without cross-checking gate sites.
    for (route, authority, command, effect) in ROUTE_AUTHORITY_TABLE {
        assert!(
            !command.is_empty(),
            "ROUTE_AUTHORITY_TABLE row for {route:?} → {authority:?} has an empty command string"
        );
        assert!(
            !effect.is_empty(),
            "ROUTE_AUTHORITY_TABLE row for {route:?} → {authority:?} has an empty effect string"
        );
        assert_eq!(
            expected_authority_for(route),
            *authority,
            "ROUTE_AUTHORITY_TABLE row {route:?} must map to its canonical Authority (single-source invariant)"
        );
    }
}

#[test]
fn steer_route_table_includes_forge_routes() {
    let authorities = table_authorities();
    assert!(
        authorities.contains(&Authority::ReviewsWrite),
        "ROUTE_AUTHORITY_TABLE must include ReviewsWrite (forge request_changes/approve gates are live)"
    );
    assert!(
        authorities.contains(&Authority::CommentsWrite),
        "ROUTE_AUTHORITY_TABLE must include CommentsWrite (forge comment/post_comment gates are live)"
    );
    assert!(
        authorities.contains(&Authority::PullRequestsWrite),
        "ROUTE_AUTHORITY_TABLE must include PullRequestsWrite (forge publish/close gates are live)"
    );

    // Sanity: the Review(RequestChanges) / Review(Comment) / Review(Approve)
    // sub-routes map to the three forge-dispatched authorities. The forge's
    // `authorize_branch_action` match still holds the literal authorize calls,
    // but the table is now the authoritative route→authority mapping.
    let reviews_write_row = ROUTE_AUTHORITY_TABLE
        .iter()
        .find(|(route, _, _, _)| *route == Route::Review(ReviewAction::RequestChanges));
    assert!(
        reviews_write_row.is_some_and(|(_, auth, _, _)| *auth == Authority::ReviewsWrite),
        "Review(RequestChanges) must map to ReviewsWrite in ROUTE_AUTHORITY_TABLE"
    );

    let comments_write_row = ROUTE_AUTHORITY_TABLE
        .iter()
        .find(|(route, _, _, _)| *route == Route::Review(ReviewAction::Comment));
    assert!(
        comments_write_row.is_some_and(|(_, auth, _, _)| *auth == Authority::CommentsWrite),
        "Review(Comment) must map to CommentsWrite in ROUTE_AUTHORITY_TABLE"
    );

    let pull_requests_write_row = ROUTE_AUTHORITY_TABLE
        .iter()
        .find(|(route, _, _, _)| *route == Route::Review(ReviewAction::Approve));
    assert!(
        pull_requests_write_row
            .is_some_and(|(_, auth, _, _)| *auth == Authority::PullRequestsWrite),
        "Review(Approve) must map to PullRequestsWrite in ROUTE_AUTHORITY_TABLE"
    );
}

/// Canonical route → authority mapping enforced by the single-source table.
///
/// Adding a new route without assigning its canonical authority fails this
/// match at the table's own test, instead of silently drifting from the
/// gate site.
fn expected_authority_for(route: &Route) -> Authority {
    match route {
        Route::Commit => Authority::ContentsWrite,
        Route::Merge => Authority::Merge,
        Route::Review(ReviewAction::RequestChanges) => Authority::ReviewsWrite,
        Route::Review(ReviewAction::Comment) => Authority::CommentsWrite,
        Route::Review(ReviewAction::Approve) => Authority::PullRequestsWrite,
        Route::Admin => Authority::AdministrationWrite,
    }
}
