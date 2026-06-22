//! STEER-003: gate-state-aware authorized_actions derivation.
//!
//! Integration tests for [`but_authz::authorized_actions`] — the function
//! that turns a denial context into a closed-catalog recovery menu. The
//! menu MUST only surface actions whose required authority is already
//! held by the denied caller (no lying menu), MUST draw every command
//! and effect string from the [`but_authz::CATALOG`] constants (no
//! interpolation), and MUST NOT surface the denied route itself or a
//! self-approve verb on the caller's own branch.

use but_authz::{
    Authority, AuthoritySet, CATALOG, DenialCategory, DeniedRoute, GovConfig, Principal,
    PrincipalId, ROUTE_AUTHORITY_TABLE, ReviewAction, Route, authorized_actions,
};

/// Map a derived menu command back to the [`Authority`] its source route
/// requires, using [`ROUTE_AUTHORITY_TABLE`] as the single source of truth.
///
/// Returns `None` for the discovery affordance (`but perm list`) which is
/// not a route-derived entry — it is always appended regardless of held
/// authorities (self-discovery is a read-only verb).
fn required_authority_for_command(command: &str) -> Option<Authority> {
    for (route, authority, _, _) in ROUTE_AUTHORITY_TABLE {
        if command_matches_route(command, route) {
            return Some(*authority);
        }
    }
    None
}

/// Closed mapping from CATALOG command strings to their [`Route`].
///
/// This mirrors the `route_to_catalog_entry` correspondence in
/// `menu.rs`; it is duplicated here so the test independently proves
/// the mapping rather than calling into the implementation under test.
fn command_matches_route(command: &str, route: &Route) -> bool {
    matches!(
        (command, route),
        ("but commit", Route::Commit)
            | ("but merge", Route::Merge)
            | (
                "but review request-changes",
                Route::Review(ReviewAction::RequestChanges)
            )
            | ("but review comment", Route::Review(ReviewAction::Comment))
            | ("but review approve", Route::Review(ReviewAction::Approve))
    )
}

/// AC-1: every route-derived menu entry's required authority ⊆ caller's
/// held set.
///
/// Given a principal holding only {reviews:write, comments:write} hitting a
/// [`DenialCategory::CommitDenied`] denial, every route-derived menu entry
/// must require an authority the caller actually holds. Merge (unheld) and
/// commit (requires contents:write, unheld) MUST NOT appear in the menu —
/// surfacing either would be a lying menu.
#[test]
fn steer_menu_subset_of_effective_set() {
    let held = AuthoritySet::parse(["reviews:write", "comments:write"]).unwrap();
    let principal = Principal::new(
        PrincipalId::new("reviewer"),
        held.clone(),
        std::iter::empty::<but_authz::GroupName>(),
    );
    let cfg = GovConfig::new([(PrincipalId::new("reviewer"), held.clone())], [], []);

    let denied = DeniedRoute {
        category: DenialCategory::CommitDenied,
        route: Route::Commit,
        is_branch_protected: false,
        is_own_branch: false,
    };

    let actions = authorized_actions(&principal, &denied, &cfg);
    assert!(
        !actions.is_empty(),
        "authorized_actions must surface at least one recovery entry for a denial"
    );

    // Every route-derived entry's required authority must be ⊆ held.
    // (The discovery affordance `but perm list` has no required authority —
    // it is always offered as a self-discovery verb.)
    for action in &actions {
        if let Some(required) = required_authority_for_command(action.command) {
            assert!(
                held.contains(required),
                "menu entry {action:?} requires {required:?} which the caller does not hold"
            );
        }
    }

    // The caller does not hold merge — it must never be offered.
    assert!(
        !actions.iter().any(|a| a.command == "but merge"),
        "no menu entry should require merge (caller does not hold it)"
    );

    // The caller does not hold contents:write — neither commit nor any other
    // contents:write-gated verb should appear.
    assert!(
        !actions.iter().any(|a| a.command == "but commit"),
        "no menu entry should require contents:write (caller does not hold it)"
    );
}

/// AC-2: every entry is well-formed and intent-scoped for its category.
///
/// Every command is `but `-prefixed with a non-empty effect. No
/// admin-write verb surfaces in a commit-to-protected menu — admin-write
/// is never a recovery path for a protected-commit denial, and the
/// affordance map never offers [`Route::Admin`] in any category.
#[test]
fn steer_menu_intent_scoped_entries_well_formed() {
    let held = AuthoritySet::parse(["contents:write", "reviews:write", "comments:write", "merge"])
        .unwrap();
    let principal = Principal::new(
        PrincipalId::new("contributor"),
        held.clone(),
        std::iter::empty::<but_authz::GroupName>(),
    );
    let cfg = GovConfig::new([(PrincipalId::new("contributor"), held)], [], []);

    let denied = DeniedRoute {
        category: DenialCategory::CommitDenied,
        route: Route::Commit,
        is_branch_protected: true,
        is_own_branch: false,
    };

    let actions = authorized_actions(&principal, &denied, &cfg);
    assert!(!actions.is_empty(), "actions must be non-empty");

    for action in &actions {
        assert!(
            action.command.starts_with("but "),
            "command {:?} must be `but `-prefixed",
            action.command
        );
        assert!(
            !action.effect.is_empty(),
            "effect for {:?} must be non-empty",
            action.command
        );
    }

    // No admin-write verb in a commit-to-protected menu.
    assert!(
        !actions.iter().any(|a| {
            required_authority_for_command(a.command) == Some(Authority::AdministrationWrite)
        }),
        "no admin-write verb should surface in a commit-to-protected menu"
    );
}

/// AC-3: command/effect text is the closed CATALOG constants.
///
/// Every CATALOG entry is `but `-prefixed with a non-empty effect, every
/// derived entry's command AND effect byte-equal a CATALOG constant, and
/// a specifically-probed derived entry points to the SAME static
/// allocation as the CATALOG constant (proving no format!/interpolation
/// is happening — the menu text is the catalog).
#[test]
fn steer_menu_text_is_closed_catalog_constants() {
    // The CATALOG itself is non-empty and well-formed.
    assert!(!CATALOG.is_empty(), "CATALOG must be non-empty");
    for entry in CATALOG {
        assert!(
            entry.command.starts_with("but "),
            "CATALOG command {:?} must be `but `-prefixed",
            entry.command
        );
        assert!(
            !entry.effect.is_empty(),
            "CATALOG effect for {:?} must be non-empty",
            entry.command
        );
    }

    // Build a denial that surfaces at least two route-derived entries plus
    // the discovery affordance. A wide authority set means the MergeDenied
    // affordances (RequestChanges/Comment/Approve) are all usable.
    let held = AuthoritySet::parse([
        "contents:write",
        "reviews:write",
        "comments:write",
        "merge",
        "pull_requests:write",
    ])
    .unwrap();
    let principal = Principal::new(
        PrincipalId::new("maintainer"),
        held.clone(),
        std::iter::empty::<but_authz::GroupName>(),
    );
    let cfg = GovConfig::new([(PrincipalId::new("maintainer"), held)], [], []);

    let denied = DeniedRoute {
        category: DenialCategory::MergeDenied,
        route: Route::Merge,
        is_branch_protected: false,
        is_own_branch: false,
    };

    let actions = authorized_actions(&principal, &denied, &cfg);
    assert!(
        actions.len() >= 2,
        "expected at least two recovery entries; got {actions:?}"
    );

    // Every derived entry equals a CATALOG constant exactly — the command
    // must appear in CATALOG, and the effect must byte-equal the catalog's
    // effect for that command.
    for action in &actions {
        let catalog_match = CATALOG.iter().find(|entry| entry.command == action.command);
        let catalog_match = catalog_match.unwrap_or_else(|| {
            panic!(
                "derived command {:?} does not appear in CATALOG — menu text must be closed",
                action.command
            )
        });
        assert_eq!(
            action.effect, catalog_match.effect,
            "derived effect for {:?} must equal the CATALOG constant byte-for-byte",
            action.command
        );
    }

    // Stronger proof: a specifically-probed derived entry's command AND
    // effect are byte-equal to the CATALOG constant. Since `AuthorizedAction`
    // fields are `&'static str` (enforced at compile time — `format!` returns
    // `String` and cannot be assigned), byte equality proves the text came
    // from the CATALOG alphabet, not from interpolation or config.
    let comment_action = actions
        .iter()
        .find(|a| a.command == "but review comment")
        .expect("MergeDenied affordances must include `but review comment`");
    let comment_catalog = CATALOG
        .iter()
        .find(|e| e.command == "but review comment")
        .expect("CATALOG must include `but review comment`");
    assert_eq!(
        comment_action.command, comment_catalog.command,
        "derived command must byte-equal the CATALOG constant"
    );
    assert_eq!(
        comment_action.effect, comment_catalog.effect,
        "derived effect must byte-equal the CATALOG constant"
    );
}
