---
stability: CONSTITUTION
last_validated: 2026-06-20
prd_version: 1.0.0
---

# 06 — External Dependencies

## Summary: 0 new dependencies expected

Check Runner is a focused "second deterministic-review clause + local runner." It
reuses crates already in the GitButler workspace. **No new dependency is expected,
and explicitly no cryptography dependency** — a direct contrast with the
over-scoped actions PRD, which reached for `hmac`/`ed25519`. The threat model
(cheapest-honest-path; reproducibility, not signing — 01 §3) makes a crypto
primitive unnecessary in v1.

## Capability → satisfied-by (all reused)

| Capability needed                                        | Satisfied by (existing)                                                | Notes                                                                                                                                                                           |
| -------------------------------------------------------- | ---------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Resolve the current head OID; read a tree/blob at an OID | `gix`                                                                  | Exact pattern in `merge_gate.rs:78,172-179` (head OID) and `:211-242` (blob read); `governance_present` tree read (`config.rs:53-71`).                                          |
| Materialize the head OID into an isolated working tree   | `gix` worktree APIs; `git worktree` executable at the shell boundary   | 07 §3 Option A/C. Shelling at the executable boundary is sanctioned (RULES.md); prefer `gix` for in-process logic.                                                              |
| Spawn the check command, capture exit + output           | `std::process` (and/or the existing `tokio` async-process facility)    | Prior art: `gitbutler-repo/src/hooks.rs` spawns hook processes; `but-forge/src/ci.rs:61-67` runs a `tokio` runtime on a thread.                                                 |
| Enforce `timeout_seconds`                                | `tokio::time` (already in-tree) or a wait-with-timeout on the child    | Hard kill → `timed_out` conclusion (fail-closed).                                                                                                                               |
| Parse `.gitbutler/checks/*.toml` + `[[required_check]]`  | `toml` + `serde` (`deny_unknown_fields`)                               | Same crates `merge_gate.rs` already uses for `gates.toml`/`permissions.toml`.                                                                                                   |
| Persist `check_results`                                  | `but-db` (`rusqlite`)                                                  | Plain table; mirrors `local_review_verdicts` / `ci_checks`.                                                                                                                     |
| Managed temp dir for the throwaway worktree              | `tempfile` (commonly already in-tree) **or** a checks-owned cache root | Tests must never use `std::env::temp_dir().join(format!(…))` (crates/AGENTS.md). Confirm `tempfile` presence; if absent, the checks-owned cache-root approach needs no new dep. |

## Explicitly NOT added

| Rejected dependency                                         | Why not (v1)                                                                                                                                                                                                        |
| ----------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `hmac` / `sha2` (HMAC-SHA256)                               | No signing in v1. Reproducibility is the integrity basis (01 §3); a forged green is caught by re-running **(post-merge, per 01 §3)**. The actions PRD's reach for `hmac` is dropped.                                |
| `ed25519-dalek` (signatures)                                | Same — no producer signing key, no signature verification.                                                                                                                                                          |
| `but-secret` keyring (as a producer-key store)              | No producer key to store.                                                                                                                                                                                           |
| Any OS-sandbox / container crate (as a _security_ boundary) | No runtime isolation in v1; the runner is butler-controlled and trusted-as-reproducible. Isolation is for _not contending on the shared worktree_ (07), achieved with a detached worktree — not a security sandbox. |

## Conditional-add discipline (RULES.md)

If implementation discovers a genuine need for a crate not already in the
workspace (e.g. a wait-with-timeout helper, or `tempfile` is in fact absent), it
must be added per RULES.md add-dependency discipline: declare it in the root
`Cargo.toml` `[workspace.dependencies]`, check it for vulnerabilities first, and
run `cargo machete` after. The **expectation remains zero new deps**; any add is
a deviation to justify in the implementing task's report — and it will **never**
be a crypto dependency for a v1 security claim.

## Cross-references

- The no-crypto security stance: [`01-architecture-posture.md`](./01-architecture-posture.md) §3, §5
- What each reused crate is used for, component by component: [`02-system-components.md`](./02-system-components.md)
- The checkout materialization (the one place a temp dir + `git worktree` is used): [`07-mechanism-agnostic-checkout.md`](./07-mechanism-agnostic-checkout.md) §3
