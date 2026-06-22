//! STEER-004 / TC-9 — the `DenialCause` enum + exhaustive non-defaulted
//! `match cause -> DenialClass` compile guard.
//!
//! The exhaustiveness property is enforced by the Rust compiler: the match
//! over `DenialCause` has NO `_ =>` arm, so adding a variant without an arm
//! or removing an arm is a non-exhaustive-match COMPILE ERROR. This test
//! asserts at runtime that all five variants map to the expected concrete
//! class, and additionally grep-checks the source to prove no wildcard arm
//! exists (the trybuild compile-fail control is owned by STEER-010).

use but_authz::{DenialCause, DenialClass};

/// AC-6 / TC-9 — the match over `DenialCause` is exhaustive and
/// non-defaulted: every variant maps to a concrete `DenialClass` with no
/// `_ =>` wildcard arm.
#[test]
fn steer_denial_cause_match_is_exhaustive_compile_guard() {
    // All five variants must map to a concrete class.
    assert_eq!(
        DenialCause::MissingAuthorityResolved.class(),
        DenialClass::ActorCorrectable,
        "MissingAuthorityResolved (perm.denied, principal resolved) → ActorCorrectable"
    );
    assert_eq!(
        DenialCause::BranchProtected.class(),
        DenialClass::ActorCorrectable,
        "BranchProtected (branch.protected) → ActorCorrectable"
    );
    assert_eq!(
        DenialCause::ReviewRequired.class(),
        DenialClass::ActorCorrectable,
        "ReviewRequired (gate.review_required) → ActorCorrectable"
    );
    assert_eq!(
        DenialCause::UnresolvedPrincipal.class(),
        DenialClass::OperatorRequired,
        "UnresolvedPrincipal (no-handle/unknown-principal) → OperatorRequired"
    );
    assert_eq!(
        DenialCause::ConfigInvalid.class(),
        DenialClass::OperatorRequired,
        "ConfigInvalid (config.invalid) → OperatorRequired"
    );

    // Prove the match is non-defaulted by reading the source: the `class()`
    // method must not contain a `_ =>` wildcard arm. A wildcard arm would
    // silently absorb a future variant, defeating the compile-break guarantee.
    let source = include_str!("../src/authorize.rs");
    let class_fn_start = source
        .find("fn class(self) -> DenialClass")
        .expect("DenialCause::class() must exist in authorize.rs");
    // Find the match block within the class() function.
    let match_start = source[class_fn_start..]
        .find("match self")
        .expect("DenialCause::class() must contain a `match self`");
    let match_end = source[class_fn_start + match_start..]
        .find("}\n    }")
        .expect("DenialCause::class() match block must end with `}`");
    let match_block =
        &source[class_fn_start + match_start..class_fn_start + match_start + match_end];

    assert!(
        !match_block.contains("_ =>"),
        "DenialCause::class() match MUST NOT contain a `_ =>` wildcard arm — \
         exhaustiveness IS the security property. Found match block:\n{match_block}"
    );

    // Assert the match has 5 explicit variant arms (no more, no less).
    let arm_count = match_block.matches("Self::").count();
    assert_eq!(
        arm_count, 5,
        "DenialCause::class() must have exactly 5 explicit variant arms (one per variant), \
         found {arm_count}. Match block:\n{match_block}"
    );
}
