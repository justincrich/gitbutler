---
stability: CONSTITUTION
last_validated: 2026-06-19
prd_version: 1.0.1
section: technical-requirements
---

# External Dependencies

**Likely none required — one _possible_ add (a keyed-MAC or signature crate), pending verification of the present hashing stack.** The initiative is pure Rust inside the existing GitButler workspace, reusing crates already present; the only candidate for a _new_ dependency is the signing primitive, and even that may be satisfiable by what is already vendored.

> **Naming:** crate `but-checks`, table `check_results` — distinct from the pre-existing `butler_actions` feature (see `02-system-components.md`).

| Capability                                                                                           | Satisfied by (already in workspace)                                                                                                    | New dependency? |
| ---------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- | --------------- |
| Check-result contract types + `Conclusion` enum + the pure verifier                                  | new `but-checks` crate (plain Rust enums/newtypes)                                                                                     | No              |
| Check-definition + gate-policy config parsing (`.gitbutler/actions/*.toml`, `.gitbutler/gates.toml`) | the `toml` / `serde` stack already in the workspace (and already used by `merge_gate.rs`'s `PermissionsWire`/`GatesWire`)              | No              |
| Ref-pinned config read (read a committed blob at a target ref)                                       | `gix` via the existing helpers (`but_authz::load_governance_config`; `merge_gate.rs:211` `read_config_blob`)                           | No              |
| The agent-unwritable ledger                                                                          | `but-db` (new `check_results` table; `rusqlite`, `chrono` already present)                                                             | No              |
| Isolated checkout at a head OID (clean-workspace, R13)                                               | `gix` worktree/object APIs already present (or `git worktree add --detach` at a shell boundary) — confirm the mechanism at planning    | No (confirm)    |
| The signing **key storage**                                                                          | `but-secret` (keyring) — already in the workspace (`crates/but-secret/`, `keyring.workspace = true`)                                   | No              |
| Hashing (for an HMAC construction or a content hash)                                                 | `sha2` — **already a workspace dependency** (used by `but-agentlog`, `but-meta`, `but`, `but-update`)                                  | No              |
| Merge-gate composition + denial contract                                                             | the existing `enforce_merge_gate` + `MergeGateError` + `but_authz::Denial`                                                             | No              |
| SHA-reset basis (old→new OID on rewrite)                                                             | `but_rebase::graph_rebase::Editor::commit_mappings` (`mod.rs:479`)                                                                     | No              |
| Subprocess execution of the check                                                                    | `std::process` / the existing `tokio` async-process facility (`but-api` async forge actions already use `tokio`) — confirm at planning | No (confirm)    |

## The one possible new dependency — the signing primitive (verify before adding)

Forgery-hardness rests on signing `(name, head_oid, conclusion)` with a producer key the agent cannot forge. The v1 primitive can be either:

1. **HMAC-SHA256** (a keyed MAC, symmetric — the producer and the verifier share the repo-local producer secret from `but-secret`). This is the simpler v1 and the same class governance named as its deferred review-integrity hardening (governance R6: "an HMAC integrity check keyed by a repo-local admin secret"). `sha2` is **already present**; an HMAC construction needs the `hmac` crate (RustCrypto), which is **NOT** currently in the workspace — this is the **single candidate new dependency**. (A hand-rolled HMAC over `sha2` is possible but discouraged; prefer the audited `hmac` crate if added.)
2. **Ed25519** (an asymmetric signature — the producer signs with a private key, the gate verifies with the public key; the agent cannot forge even with read access to the public key). Stronger, and the eventual target governance also named ("then Ed25519-signed review artifacts"). Needs `ed25519-dalek` (or similar), **NOT** currently in the workspace.

**Recommendation:** ship v1 with **HMAC-SHA256** (minimal surface: reuse `sha2`, add `hmac` if a hand-roll is rejected in review) and name **Ed25519** as the hardening follow-up — exactly the governance escalation path (HMAC → Ed25519). Adding `hmac` (or `ed25519-dalek`) is the **only** action that would touch `Cargo.toml`'s `[workspace.dependencies]`; it MUST follow the project's add-dependency discipline (vulnerability check, root `Cargo.toml`, `cargo machete` after).

> **Honesty note (carries into R2/R4).** HMAC-SHA256 is symmetric: the _same_ repo-local secret signs and verifies, so an agent that can **read** that secret can forge a correctly-signed row. **In the personal-tenant model the agent shares the OS user with the executor**, so it can read the `but-secret` keyring producer secret **without elevated privilege** — this does not make the v1 ledger tamper-proof and does **NOT close the agent-forgery path**; it RAISES the bar above `local_review_verdicts`' _zero_ integrity to _"forging requires reading the producer secret."_ The asymmetric **Ed25519** upgrade **plus an OS-sandboxed executor** is what removes the read-the-secret-and-forge path. The build must NOT present HMAC-v1 as unforgeable or as a closed boundary; it is the corrected-but-not-final integrity layer. See `07-technical-risks.md` R2/R4.

## Notes

- **No network / forge dependency.** v1 is local: the executor runs repo-local check definitions in the trusted process and records locally; the gate reads the local ledger. `but-github`/`but-gitlab` are untouched, and `ci_checks` (the remote-CI cache) is deliberately not reused.
- **No runner / broker dependency.** The no-broker v1 (Stance 2) ships no lease/long-poll/runner-registration protocol, so no job-queue / message-bus / runner-SDK dependency.
- **No new process-management dependency for v1.** Running a check subprocess uses `std::process` (or an async equivalent already in the workspace via `tokio`, which `but-api`'s async forge actions already use). Confirm the existing async-process facility at planning rather than adding one.
- The CLI verb surface lives in `crates/but/src/command/` + `crates/but/src/args/` (the `but` crate), not `but-clap` — `but-clap` is a CLI-**docs** generator.
