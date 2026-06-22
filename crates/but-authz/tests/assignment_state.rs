//! Boundary (de)serialization tests for `AssignmentState`.
//!
//! Mirrors the `tests/authority.rs` integration-test discipline: the enum's
//! only contract is a total injective `parse`/`name` round-trip over the three
//! literals the `local_review_assignments.state` TEXT column stores.

use but_authz::{AssignmentState, Authority};

/// AC-1 [PRIMARY]: `parse(name(v)) == Ok(v)` for every variant — the round-trip
/// is total and injective. Fails if `name` and `parse` drift, if two variants
/// share a literal, or if `parse` defaults an unknown to a variant.
#[test]
fn assignment_state_parse_name_round_trips() {
    let variants = [
        AssignmentState::Pending,
        AssignmentState::Approved,
        AssignmentState::ChangesRequested,
    ];
    for variant in variants {
        let parsed = AssignmentState::parse(variant.name());
        assert_eq!(
            parsed,
            Ok(variant),
            "parse(name({variant:?})) must round-trip to the same variant"
        );
    }
}

/// AC-2: `name()` returns EXACTLY the three literals the
/// `local_review_assignments.state` column stores and that
/// `approve_review`/`request_changes_review` write. A typo or case drift
/// here would silently break every orchestrator state read.
#[test]
fn assignment_state_literals_match_column_values() {
    assert_eq!(
        AssignmentState::Pending.name(),
        "pending",
        "Pending must serialize to the exact TEXT column literal \"pending\""
    );
    assert_eq!(
        AssignmentState::Approved.name(),
        "approved",
        "Approved must serialize to the exact TEXT column literal \"approved\""
    );
    assert_eq!(
        AssignmentState::ChangesRequested.name(),
        "changes_requested",
        "ChangesRequested must serialize to the exact TEXT column literal \"changes_required\""
    );
}

/// AC-3: `parse` is fail-closed — unknown / garbage / wrong-case strings all
/// return `Err`. No `_ => Self::Pending` default arm (silent corruption).
#[test]
fn assignment_state_parse_rejects_unknown() {
    for unknown in [
        "merged",
        "",
        "Approved",
        "PENDING",
        "changesRequested",
        "pending ",
    ] {
        assert!(
            AssignmentState::parse(unknown).is_err(),
            "parse({unknown:?}) must reject an unknown/garbage/wrong-case state (no default coercion)"
        );
    }
}

/// AC-4: `AssignmentState` is a DRIVE-state enum, NOT an `Authority`. The
/// functional-authority catalog stays closed at 12 variants — LPR-002 introduces
/// no enforcement-path branch. The type-system already forbids mixing the two;
/// this test pins the Authority variant count so a future drift is caught here
/// rather than in the merge gate.
#[test]
fn assignment_state_not_an_authority() {
    // The shipped functional catalog (authority.rs:46) has exactly 12 variants.
    // LPR-002 must NOT add a 13th — AssignmentState is a separate drive-state enum.
    assert_eq!(
        Authority::ALL.len(),
        12,
        "Authority catalog must remain at 12 functional variants — AssignmentState is not an Authority"
    );
    // AssignmentState has exactly its three drive states.
    assert_eq!(
        AssignmentState::ALL.len(),
        3,
        "AssignmentState catalog is the three drive states: pending/approved/changes_requested"
    );
}
