---
stability: CONSTITUTION
last_validated: 2026-06-20
prd_version: 1.0.0
---

# 02 — System Components

## §1 — New vs extended vs reused

| Component                                                                                                  | Status                                                                                                                                                   | Location                                                           |
| ---------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| `but-checks` crate (runner + config loader + plain recorder + pure required-checks evaluator)              | **NEW**                                                                                                                                                  | `crates/but-checks/`                                               |
| `check_results` table                                                                                      | **NEW**                                                                                                                                                  | `crates/but-db/src/table/check_results.rs`                         |
| Required-checks merge-gate **clause**                                                                      | **EXTEND**                                                                                                                                               | `crates/but-api/src/legacy/merge_gate.rs`                          |
| `but check {define,list,run,results,required}` CLI verbs                                                   | **NEW**                                                                                                                                                  | `crates/but/src/args/check.rs` + `crates/but/src/command/check.rs` |
| `MergeGateError` (today: `code`/`message`/`remediation_hint`/`unmet` only, merge_gate.rs:19-29)            | **REUSE**                                                                                                                                                | `crates/but-api/src/legacy/merge_gate.rs:19`                       |
| `MergeGateError` STEER fields (`class`/`held_permissions`/`authorized_actions`/`do_not`) + `to_envelope()` | **EXTENDS — depends on governance STEER-001** (sprint-07, `STATUS: Backlog`, NOT yet merged; the fields + `to_envelope()` do not exist in `crates/` yet) | governance `sprint-07-steer-capability-aware-denials/STEER-001`    |
| `read_config_blob` ref-pin read                                                                            | **REUSE**                                                                                                                                                | `crates/but-api/src/legacy/merge_gate.rs:211`                      |
| `governance_present` opt-in discriminator                                                                  | **REUSE**                                                                                                                                                | `crates/but-authz/src/config.rs:53`                                |
| `Editor::commit_mappings` (SHA-reset basis)                                                                | **REUSE**                                                                                                                                                | `crates/but-rebase/src/graph_rebase/mod.rs:479`                    |
| `gix` (OID resolve, tree read, worktree)                                                                   | **REUSE**                                                                                                                                                | workspace dep                                                      |
| `std::process` / `tokio` process facility                                                                  | **REUSE**                                                                                                                                                | std / existing `tokio`                                             |
| `toml` + `serde` (config parse)                                                                            | **REUSE**                                                                                                                                                | workspace deps                                                     |

## §2 — `but-checks` crate (module map)

A focused crate — **not** a CI platform. Modeled on the separation the merge gate
already uses (a wire type, a normalizer, a pure evaluator, a structured error).

```text
crates/but-checks/src/
├── lib.rs            # public surface: CheckDefinition, Conclusion, CheckResult,
│                     #   RequiredCheck, load_check_defs, evaluate_required_checks,
│                     #   run_check, record_result
├── conclusion.rs     # Conclusion enum + Conclusion::from_exit (the ONLY exit→conclusion map)
├── config.rs         # ChecksWire/CheckWire/RequiredCheckWire (serde, deny_unknown_fields),
│                     #   load_check_defs(repo, target_ref) via the ref-pinned blob read
├── checkout.rs       # mechanism-agnostic head-OID materialization (07): detached worktree /
│                     #   object-DB-only / warm-reused; Drop-guarded teardown
├── runner.rs         # run_check: materialize (checkout.rs) → spawn command (std::process/tokio)
│                     #   → Conclusion::from_exit → CheckResult (NO caller-supplied conclusion)
├── recorder.rs       # record_result: append a CheckResult row (but-db); append-only
└── evaluator.rs      # evaluate_required_checks: PURE fn — (required set, results, current head)
                      #   → Ok(()) | Err(unmet: Vec<String>); never spawns, never reads git
```

### The four responsibilities (deliberately split, like the gate's design)

| Module         | Responsibility                                                     | Deterministic?                | Mirrors                                           |
| -------------- | ------------------------------------------------------------------ | ----------------------------- | ------------------------------------------------- |
| `config.rs`    | Parse ref-pinned `.gitbutler/checks/*.toml` + `[[required_check]]` | Pure (given blobs)            | `normalize_gates` (merge_gate.rs:308)             |
| `checkout.rs`  | Materialize the head OID into an isolated tree                     | Side-effectful, isolated      | new (07)                                          |
| `runner.rs`    | Run the command, derive conclusion from real exit                  | Side-effectful (the producer) | `hooks.rs` spawn pattern (prior art only)         |
| `evaluator.rs` | Decide pass/fail of the required set at the head                   | **Pure**                      | `review_requirement::evaluate` (merge_gate.rs:86) |

`evaluator.rs` is the analog of the review-requirement evaluator: a pure function
the read-only gate calls. It takes the required set, the recorded results, and
the current head OID, and returns `Ok(())` or `Err(Vec<String>)` of miss-reason
tokens (`check_missing` / `check_failed` / `check_stale_at_head`). It **never**
spawns a process and **never** reads git — keeping the gate deterministic.

### Dependency direction (RULES.md)

`but-checks` is a lower-level crate: it may depend on `but-db`, `gix`, `toml`,
`serde`, `std::process`/`tokio`. It must **not** depend on `but-api`. The gate
clause lives **in** `but-api` (`merge_gate.rs`) and calls _up into_ nothing — it
calls `but_checks::load_check_defs` + `but_checks::evaluate_required_checks` (the
pure evaluator) the same way `merge_gate.rs` calls `review_requirement::evaluate`
today.

## §3 — `check_results` table component

A plain `but-db` table (03 §1). Registered in the `but-db` migrations list
alongside `local_review_verdicts` and `ci_checks`, with handle methods
`list_for(name, head_oid)` and `insert(row)` mirroring
`LocalReviewVerdictsHandle` / `LocalReviewVerdictsHandleMut`
(`crates/but-db/src/table/local_review_verdicts.rs`). No new dependency, no
crypto column logic — `signature` is a nullable passthrough.

## §4 — Extended `merge_gate.rs` clause component

A new clause inside `enforce_merge_gate`
(`crates/but-api/src/legacy/merge_gate.rs:40`), added **after** the review
clause and consulted **independent of the `protected` early-return**
(01 §9; 08 R-FAILOPEN). It:

1. Loads `[[required_check]]` for the target via the extended `GatesWire`
   (struct at merge_gate.rs:411; normalized by `normalize_gates`, merge_gate.rs:308-343).
2. Loads `.gitbutler/checks/*.toml` defs via `but_checks::load_check_defs` using
   the same `read_config_blob` ref-pin path (merge_gate.rs:211).
3. Resolves `current_head_oid` (merge_gate.rs:78) and reads the recorded results.
4. Calls the **pure** `but_checks::evaluate_required_checks(...)`.
5. On `Err`, returns a `MergeGateError { code: "gate.check_required", … }` with
   the STEER fields populated **once STEER-001 lands** (04 §5; until then the
   carrier has only `code`/`message`/`remediation_hint`/`unmet`).

For the unsatisfiable-`[[required_check]]` fail-closed case, copy the
**enforcing** pattern — the caller's `is_empty()` → `MergeGateError` block at
**merge_gate.rs:62-76** (the collection at merge_gate.rs:181-188 is only the
collector), so an unsatisfiable required-check fails closed, not vacuously passes
(MEDIUM parity, 03 §3).

It re-uses `classify_error` (merge_gate.rs:113) unchanged — the carrier downcasts
out of the `anyhow` chain identically.

## §5 — `but check` CLI verbs component

A new `but check` noun, modeled on the governance `but perm` / `but group` noun
pattern (`crates/but/src/args/perm.rs:10`, `crates/but/src/args/group.rs:10` —
each a `pub enum Subcommands`; both verified present, though the governance CLI is
still in progress in sprint-05/06), with a handler in
`crates/but/src/command/check.rs`.

| Verb                                        | Purpose                                                         | UC             |
| ------------------------------------------- | --------------------------------------------------------------- | -------------- |
| `but check define`                          | Scaffold / validate a `[[check]]` in `.gitbutler/checks/*.toml` | UC-DEFN-01..03 |
| `but check list`                            | List defined checks (+ which are required) at the target ref    | UC-DEFN-04     |
| `but check run <name> [--head <oid>]`       | **The producer**: materialize, run, record at the head OID      | UC-RUN-01..04  |
| `but check results [<name>] [--head <oid>]` | Show recorded results (dual-audience: human table + `--json`)   | UC-RUN-05      |
| `but check required`                        | Show the `[[required_check]]` set for the target                | UC-GATE-01     |

All verbs are **dual-audience**: human text by default, `--json` for the
machine/agent path (mirroring `governance_cli_error`'s `{error:{code,message}}`
rendering and the existing `OutputFormat` plumbing in `crates/but/src/lib.rs`).

## §6 — Producer-zero seam: the existing forge-CI cache (do not reinvent)

The read-only forge-CI cache already exists and is **producer-zero**: forge
checks are produced **non-forgeably upstream** (by GitHub/GitLab) and cached
locally read-only.

- `crates/but-forge/src/ci.rs` — `ci_checks_for_ref_with_cache` (`:5`),
  `CiCheck` / `CiStatus` / `CiConclusion` (`:111-181`).
- `crates/but-github/src/client.rs` — `list_checks_for_ref` (`:150`), `CheckRun`
  (`:1006`, fields `conclusion: Option<String>` `:1011`, `head_sha: Option<String>`
  `:1015`).
- `crates/but-db/src/table/ci_checks.rs` — the `ci_checks` cache table
  (`(name, head_sha, status_conclusion)`, GitHub conclusion vocabulary).

**Treat this as the future "pluggable external producers" seam, not as part of
v1's local producer.** Check Runner's `Conclusion` enum (03 §4) deliberately
mirrors `CiConclusion` so an external-producer adapter can map a cached forge
`CheckRun` into a `CheckResult` later with no schema churn. **`ci_checks` is NOT
the `check_results` ledger** — it is a disposable, ref-keyed cache
(`set_for_reference` does `DELETE … WHERE reference = ?1` then re-INSERT,
ci_checks.rs:147-188); the v1 local producer writes `check_results`, not
`ci_checks`.

|                              | `ci_checks` (existing)                     | `check_results` (new)                |
| ---------------------------- | ------------------------------------------ | ------------------------------------ |
| Producer                     | Upstream forge (GitHub/GitLab)             | Local butler runner                  |
| Keyed by                     | `reference` (ref name)                     | `(name, head_oid)`                   |
| Lifecycle                    | Disposable cache (delete+reinsert per ref) | Append-only verdict store            |
| Consulted by the merge gate? | No (read-only UI cache)                    | **Yes** (the required-checks clause) |

## §7 — Disambiguation from `butler_actions` (unrelated)

There is a pre-existing `butler_actions` feature — `crates/but-action/`
(`action.rs`, `cli.rs`, `generate.rs`, …) plus the `butler_actions` `but-db`
table — which is an **agent-action audit log**, entirely unrelated to Check
Runner. To avoid collision, Check Runner uses **`but check` / `but-checks` /
`check_results`** and **never** any `*actions*` symbol, table, or noun.

## Cross-references

- The runner's checkout strategy (the hard part): [`07-mechanism-agnostic-checkout.md`](./07-mechanism-agnostic-checkout.md)
- The schemas these components read/write: [`03-data-schema.md`](./03-data-schema.md)
- The run → record → consume API: [`04-api-design.md`](./04-api-design.md)
- Zero new dependencies: [`06-external-dependencies.md`](./06-external-dependencies.md)
