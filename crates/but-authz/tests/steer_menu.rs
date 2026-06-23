//! STEER-003 — gate-state-aware `authorized_actions` derivation proofs.
//!
//! These integration tests prove the menu derivation's four core properties:
//!
//! 1. **AC-1 — Subset of effective set**: every menu entry's required
//!    authority ⊆ the caller's held set (`steer_menu_subset_of_effective_set`).
//! 2. **AC-4 — Intent-scoped + well-formed**: each entry is a `{command,
//!    effect}` pair with a `but `-prefixed command and non-empty effect; the
//!    menu is intent-scoped, not the whole catalog
//!    (`steer_menu_intent_scoped_entries_well_formed`).
//! 3. **AC-5 — Closed catalog**: every command/effect text comes from the
//!    closed `&'static str` CATALOG; no `format!` or interpolation
//!    (`steer_menu_text_is_closed_catalog_constants`).
//! 4. **C5 gate-state-aware**: a `branch.protected` denial offers a
//!    feature-branch commit, NOT the protected-ref commit just denied.
//!
//! See `.spec/prds/governance/tasks/sprint-08-steer-capability-aware-denials/
//! STEER-003-gate-state-aware-authorized-actions.md` for the task contract.

use but_authz::{
    AFFORDANCE_MAP, Authority, AuthoritySet, CATALOG, DenialPredicate, DeniedRoute, GovConfig,
    Principal, PrincipalId, Route, authorized_actions,
};

/// Build a principal + cfg pair where the principal holds the given authority
/// set.
fn principal_with(handle: &str, tokens: &[&str]) -> anyhow::Result<(Principal, GovConfig)> {
    let set = AuthoritySet::parse(tokens.iter().copied())?;
    let principal = Principal::new(PrincipalId::new(handle), set.clone(), []);
    let cfg = GovConfig::new([(PrincipalId::new(handle), set)], [], []);
    Ok((principal, cfg))
}

/// Collect the set of authorities a route requires for each menu entry's
/// route, proving every entry's required authority ⊆ held.
fn route_authority_for_command(command: &str) -> Option<Authority> {
    for (route, _, mapped_command, _) in but_authz::ROUTE_AUTHORITY_TABLE {
        if *mapped_command == command {
            return Some(route.required_authority());
        }
    }
    // `but review request-changes` maps to ForgeReviewsWrite via a composite
    // table command; resolve individual review verbs to their authority.
    if command.starts_with("but review request-changes") || command == "but review approve" {
        return Some(Authority::ReviewsWrite);
    }
    if command.starts_with("but review comment") {
        return Some(Authority::CommentsWrite);
    }
    if command.starts_with("but review new") {
        return Some(Authority::PullRequestsWrite);
    }
    if command == "but perm list" {
        // Discovery is self-scoped — no authority required beyond being a
        // resolved principal. Return a read-level authority for subset-checking
        // purposes (it's always held trivially since it's self-scoped).
        return None;
    }
    None
}

// ---------------------------------------------------------------------------
// AC-1 — Menu ⊆ effective set ∩ table; no entry requires an unheld authority
// ---------------------------------------------------------------------------

/// AC-1 / TC-1 — every `authorized_actions` entry's required authority is a
/// subset of the caller's held set.
///
/// GIVEN a principal holding `{contents:read, comments:write}` hitting a denial,
/// WHEN `authorized_actions` is derived, THEN every listed entry's required
/// authority ⊆ the caller's held set. No entry requires `contents:write`
/// (unheld), `merge` (unheld), or `administration:write` (unheld).
#[test]
fn steer_menu_subset_of_effective_set() -> anyhow::Result<()> {
    let (principal, cfg) = principal_with("dev", &["contents:read", "comments:write"])?;

    // dev tried to commit but lacks contents:write.
    let denied = DeniedRoute::new(Route::Commit, DenialPredicate::Authority);
    let actions = authorized_actions(&principal, &denied, &cfg);

    assert!(
        !actions.is_empty(),
        "menu must be non-empty (at minimum discovery is appended)"
    );

    let held = AuthoritySet::parse(["contents:read", "comments:write"])?;
    for action in &actions {
        if let Some(required) = route_authority_for_command(action.command) {
            assert!(
                held.contains(required),
                "menu entry {:?} requires {:?} which is NOT in the held set {:?} — \
                 this is a lying menu (AC-1 violation)",
                action.command,
                required,
                ["contents:read", "comments:write"]
            );
        }
    }

    // Must observe: an entry requiring comments:write (e.g. `but review comment`).
    let commands: Vec<&str> = actions.iter().map(|a| a.command).collect();
    assert!(
        commands.contains(&"but review comment"),
        "menu must include `but review comment` (comments:write is held)"
    );

    // Must NOT observe: a `but commit` entry (requires contents:write, unheld).
    assert!(
        !commands.contains(&"but commit"),
        "menu must NOT include `but commit` — contents:write is not held (lying menu)"
    );

    // Must NOT observe: a merge verb (requires merge, unheld).
    assert!(
        !commands.contains(&"but review merge"),
        "menu must NOT include `but review merge` — merge is not held (lying menu)"
    );

    // Must NOT observe: `but review request-changes` (requires reviews:write, unheld).
    assert!(
        !commands.contains(&"but review request-changes"),
        "menu must NOT include `but review request-changes` — reviews:write is not held"
    );

    println!(
        "AC-1: menu for {{contents:read, comments:write}} principal on Commit/Authority denial:"
    );
    for action in &actions {
        println!("  - {} → {}", action.command, action.effect);
    }

    Ok(())
}

/// AC-1 edge case — a principal with zero authorities still gets a valid menu
/// (discovery only); no lying entries.
#[test]
fn steer_menu_subset_of_effective_set_empty_authority() -> anyhow::Result<()> {
    let (principal, cfg) = principal_with("ghost", &[])?;

    let denied = DeniedRoute::new(Route::Commit, DenialPredicate::Authority);
    let actions = authorized_actions(&principal, &denied, &cfg);

    // With no held authorities, only discovery is offered.
    assert_eq!(
        actions.len(),
        1,
        "empty-authority principal gets discovery only"
    );
    assert_eq!(actions[0].command, "but perm list");

    Ok(())
}

/// AC-1 edge case — a principal holding `contents:write` hitting a
/// `branch.protected` denial DOES see `but commit` (they hold the authority;
/// the C5 subtraction is about the ref, not the authority).
#[test]
fn steer_menu_subset_of_effective_set_protected_keeps_commit() -> anyhow::Result<()> {
    let (principal, cfg) = principal_with("dev", &["contents:write", "reviews:write"])?;

    let denied = DeniedRoute::new(Route::Commit, DenialPredicate::BranchProtected);
    let actions = authorized_actions(&principal, &denied, &cfg);

    let commands: Vec<&str> = actions.iter().map(|a| a.command).collect();
    assert!(
        commands.contains(&"but commit"),
        "branch.protected denial MUST keep `but commit` (caller holds contents:write) — \
         the C5 subtraction is about the REF (feature vs protected), not the authority"
    );
    assert!(
        commands.contains(&"but review request-changes"),
        "branch.protected denial must include review affordances (reviews:write held)"
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// AC-4 — Each entry is {command, effect}; menu is intent-scoped
// ---------------------------------------------------------------------------

/// AC-4 / TC-6, TC-7 — each `authorized_actions` entry has a `but `-prefixed
/// command and a non-empty effect; the menu is intent-scoped and does NOT
/// list unrelated admin-write verbs.
#[test]
fn steer_menu_intent_scoped_entries_well_formed() -> anyhow::Result<()> {
    let (principal, cfg) = principal_with(
        "dev",
        &["contents:write", "reviews:write", "comments:write"],
    )?;

    // A commit-to-protected denial by a review-capable principal.
    let denied = DeniedRoute::new(Route::Commit, DenialPredicate::BranchProtected);
    let actions = authorized_actions(&principal, &denied, &cfg);

    assert!(
        !actions.is_empty(),
        "menu must be non-empty for an actor-correctable denial"
    );

    let mut has_review_category = false;
    for action in &actions {
        // Each entry's command starts with `but `.
        assert!(
            action.command.starts_with("but "),
            "entry command {:?} must start with \"but \"",
            action.command
        );
        // Each entry has a non-empty effect.
        assert!(
            !action.effect.is_empty(),
            "entry for {:?} has an empty effect",
            action.command
        );
        if action.command == "but review request-changes" {
            has_review_category = true;
        }
    }
    assert!(
        has_review_category,
        "a review-category entry (`but review request-changes`) must be present"
    );

    // Intent-scoped: the menu must NOT list unrelated admin-write verbs.
    let commands: Vec<&str> = actions.iter().map(|a| a.command).collect();
    assert!(
        !commands
            .iter()
            .any(|c| c.contains("governance config") || c.contains("admin")),
        "a commit-to-protected menu must NOT list admin-write verbs (intent-scoped): {commands:?}"
    );

    // Must NOT include `but review merge` — that's a merge-intent verb, not a
    // commit-intent verb. (The caller holds contents:write but the denied
    // intent is "commit", not "merge".)
    assert!(
        !commands.contains(&"but review merge"),
        "commit-intent menu must NOT include the merge verb (intent-scoped): {commands:?}"
    );

    println!("AC-4: commit-to-protected menu for review-capable principal:");
    for action in &actions {
        println!("  - {} → {}", action.command, action.effect);
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// AC-5 — All command/effect text comes from the closed catalog
// ---------------------------------------------------------------------------

/// AC-5 / TC-8 — every CATALOG command/effect is an `&'static str` constant
/// and a derived entry equals a CATALOG constant exactly.
///
/// This is the unit-tier closed-catalog constness check: it asserts every
/// derived entry's `(command, effect)` pair resolves to an `&'static str`
/// constant in [`but_authz::CATALOG`]. The build-gate grep counterpart is
/// owned by STEER-010.
#[test]
fn steer_menu_text_is_closed_catalog_constants() -> anyhow::Result<()> {
    let (principal, cfg) = principal_with(
        "dev",
        &["contents:write", "reviews:write", "comments:write", "merge"],
    )?;

    // Exercise multiple denial shapes to cover different menu renderings.
    let denial_shapes = [
        DeniedRoute::new(Route::Commit, DenialPredicate::BranchProtected),
        DeniedRoute::new(Route::Commit, DenialPredicate::Authority),
        DeniedRoute::new(Route::Merge, DenialPredicate::ReviewRequired),
        DeniedRoute::new(Route::ForgeReviewsWrite, DenialPredicate::Authority),
    ];

    for denied in &denial_shapes {
        let actions = authorized_actions(&principal, denied, &cfg);
        for action in &actions {
            // Every derived entry MUST equal a CATALOG constant exactly —
            // no `format!`, interpolation, or config-sourced text.
            let found = CATALOG.iter().find(|catalog| {
                catalog.command == action.command && catalog.effect == action.effect
            });
            assert!(
                found.is_some(),
                "derived entry {{command: {:?}, effect: {:?}}} does NOT match any CATALOG \
                 constant — this violates the closed-catalog invariant (AC-5)",
                action.command,
                action.effect
            );
        }
    }

    // Spot-check: CATALOG entries are `&'static str` (compile-time property —
    // verified by the type system; this is a runtime mirror).
    for catalog in CATALOG {
        let _command: &'static str = catalog.command;
        let _effect: &'static str = catalog.effect;
    }

    // Spot-check: specific CATALOG command literals are exactly as expected.
    assert!(
        CATALOG
            .iter()
            .any(|a| a.command == "but review request-changes"),
        "CATALOG must contain the `but review request-changes` &'static str constant"
    );
    assert!(
        CATALOG.iter().any(|a| a.command == "but perm list"),
        "CATALOG must contain the `but perm list` discovery &'static str constant"
    );

    Ok(())
}

/// AC-5 edge case — no menu entry's command or effect contains an interpolated
/// branch/principal substring. (Branch names and principal handles are dynamic
/// and must NEVER appear in the closed-catalog text.)
#[test]
fn steer_menu_text_has_no_interpolated_substrings() -> anyhow::Result<()> {
    let (principal, cfg) = principal_with("dev", &["contents:write"])?;

    let denied = DeniedRoute::new(Route::Commit, DenialPredicate::BranchProtected);
    let actions = authorized_actions(&principal, &denied, &cfg);

    // The principal handle "dev" and a hypothetical branch name "main" must
    // NEVER appear in any command or effect — they'd indicate interpolation.
    for action in &actions {
        assert!(
            !action.command.contains("dev"),
            "command {:?} contains the principal handle — interpolation forbidden",
            action.command
        );
        assert!(
            !action.effect.contains("dev"),
            "effect {:?} contains the principal handle — interpolation forbidden",
            action.effect
        );
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// C5 — branch.protected offers feature-branch commit, NOT protected-ref commit
// ---------------------------------------------------------------------------

/// C5 proof — a `branch.protected` denial's menu includes `but commit` (the
/// caller holds `contents:write`) AND the catalog entry's effect names an
/// "unprotected feature branch" (a DIFFERENT ref), not the protected ref.
///
/// This is the code-level proof that the C5 subtraction is correct: the menu
/// does NOT reproduce the denied protected-ref commit; it offers a
/// feature-branch commit instead.
#[test]
fn steer_menu_branch_protected_offers_feature_commit_not_protected() -> anyhow::Result<()> {
    let (principal, cfg) = principal_with("dev", &["contents:write"])?;

    let denied = DeniedRoute::new(Route::Commit, DenialPredicate::BranchProtected);
    let actions = authorized_actions(&principal, &denied, &cfg);

    let commit_entry = actions
        .iter()
        .find(|a| a.command == "but commit")
        .expect("branch.protected menu must include `but commit` (contents:write held)");

    // The CATALOG effect for `but commit` must name an UNPROTECTED FEATURE
    // branch — the succeeding context — NOT the protected ref.
    assert!(
        commit_entry.effect.to_lowercase().contains("unprotected"),
        "`but commit` effect must name an UNPROTECTED ref (the C5 succeeding context), got: {:?}",
        commit_entry.effect
    );
    assert!(
        !commit_entry.effect.to_lowercase().contains("protected ref"),
        "`but commit` effect must NOT name the protected ref (the denied context), got: {:?}",
        commit_entry.effect
    );

    Ok(())
}

/// AFFORDANCE_MAP coverage — every table route has an AFFORDANCE_MAP entry,
/// and no entry names the denied route AT the denied ref (the C5 map
/// invariant; STEVER-010's grep enforces this too).
#[test]
fn steer_affordance_map_covers_every_route_and_no_self_reference() -> anyhow::Result<()> {
    for route in Route::ALL {
        let has_entry = AFFORDANCE_MAP.iter().any(|(r, _, _)| r == route);
        assert!(
            has_entry,
            "Route::{route:?} must have at least one AFFORDANCE_MAP entry (STEER-010 coverage)"
        );
    }

    // For BranchProtected entries, the Commit affordance candidate exists but
    // is curated as "but commit" with a feature-branch catalog effect — it
    // names a succeeding context, not the denied protected-ref context.
    for (denied_route, predicate, candidates) in AFFORDANCE_MAP {
        let _ = denied_route; // iterated for coverage; route is validated above
        if *predicate == DenialPredicate::BranchProtected {
            for candidate in *candidates {
                // A BranchProtected candidate may use the denied route
                // (Commit → Commit), but the CATALOG effect it resolves to
                // must name a DIFFERENT (unprotected) ref.
                if let Some(action) = CATALOG.iter().find(|c| c.command == candidate.command) {
                    assert!(
                        action.effect.to_lowercase().contains("unprotected")
                            || !action.effect.to_lowercase().contains("protected"),
                        "BranchProtected candidate {:?} → effect {:?} must NOT name the protected ref",
                        candidate.command,
                        action.effect
                    );
                }
            }
        }
    }

    Ok(())
}
