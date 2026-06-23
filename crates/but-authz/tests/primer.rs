//! STEER-008: agent-priming reference primer is non-enforced L2 reference.
//!
//! Proves the `AGENT_PRIMER` constant exists, is non-empty, and is just a
//! `&'static str` — no but-authz or but-api code path imports or calls it for
//! enforcement decisions. The enforcement lives in the denial fields
//! themselves (`class`, `authorized_actions`, `do_not`); this document only
//! helps an agent interpret them.

use but_authz::AGENT_PRIMER;

/// AC-1: `AGENT_PRIMER` is exported and non-empty.
#[test]
fn agent_primer_is_non_empty() {
    assert!(
        !AGENT_PRIMER.trim().is_empty(),
        "AGENT_PRIMER must contain reference text"
    );
}

/// AC-2: the primer covers the five contract points by naming their tokens.
#[test]
fn agent_primer_covers_contract_tokens() {
    let lowercase = AGENT_PRIMER.to_ascii_lowercase();
    let missing: Vec<&str> = [
        "actor_correctable",
        "operator_required",
        "authorized_actions",
        "do_not",
    ]
    .into_iter()
    .filter(|token| !lowercase.contains(token))
    .collect();
    assert!(
        missing.is_empty(),
        "AGENT_PRIMER must mention all contract tokens, missing: {missing:?}"
    );
}
