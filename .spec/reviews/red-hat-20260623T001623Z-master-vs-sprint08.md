# Red-Hat Review Report — Sprint 08 STEER vs `master`

**Report Date**: 2026-06-23T00:16:23Z
**Target**: Sprint 08 — STEER: Capability-Aware Denials (10 tasks, 54 ACs / 79 TCs)
**Reviewer**: single-reviewer red-hat (rust-reviewer stance), code-vs-spec
**Question**: Does `master` (HEAD `b8848c29fe`) contain all functionality defined in `.spec/prds/governance/tasks/sprint-08-steer-capability-aware-denials/`?

## Executive Summary

**NO.** `master` has the **STEER-001 type/field scaffolding** and the **STEER-004 carrier-struct field plumbing**, but **8 of 10 STEER tasks are non-functional on master**. Three implementation files exist on master as **orphan code** (present on disk, not declared in `mod.rs`, never compiled, symbols not exported). The exhaustive `class` mapping, gate-state-aware menu derivation, CLI serializer enrichment, telemetry event, governed-loop extensions, and net-new honesty greps are entirely absent. All 10 task files still report `STATUS: Backlog`. Sprint 08 has never been run through `/kb-run-sprint`; the cycle-3 red-hat remediation commit (`b8848c29fe`) modified only `.spec/` markdown.

## AC VERDICT TABLE (per task)

| #   | Task                                                                                                                    | Verdict    | Evidence                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       | Notes                                                                                                                                                                                                                                                    |
| --- | ----------------------------------------------------------------------------------------------------------------------- | ---------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | STEER-001 — steering fields/types on 6 carriers                                                                         | ✅ PASS    | `crates/but-authz/src/denial.rs:38-265` defines `DenialClass`, `AuthorizedAction`, `Denial{code,message,remediation_hint,class,held_permissions,authorized_actions,do_not}`, `to_envelope()`; exported `crates/but-authz/src/lib.rs:21`.                                                                                                                                                                                                                                                                                                                       | Type scaffolding present and exported. Constructor `Denial::new()` defaults all four steering fields (`denial.rs:161-170`) — actual population is STEER-004's job.                                                                                       |
| 2   | STEER-002 — `Route` enum + `ROUTE_AUTHORITY_TABLE`                                                                      | ❌ FAIL    | File `crates/but-authz/src/route.rs` exists (defines `Route`, `ReviewAction`, `ROUTE_AUTHORITY_TABLE`) BUT `crates/but-authz/src/lib.rs:3-9` has NO `mod route;` and the `pub use` block (lib.rs:21) does NOT export `Route`/`ROUTE_AUTHORITY_TABLE`/`ReviewAction`. Orphan file.                                                                                                                                                                                                                                                                              | Test `crates/but-authz/tests/steer_route_table.rs:11` does `use but_authz::{Authority, ROUTE_AUTHORITY_TABLE, ReviewAction, Route}` — **would not compile on master**. Symbol is unreachable.                                                            |
| 3   | STEER-003 — gate-state-aware `authorized_actions` derivation                                                            | ❌ FAIL    | File `crates/but-authz/src/menu.rs` exists (defines `DenialCategory`, `DeniedRoute`, `AFFORDANCE_MAP`, `CATALOG`, `authorized_actions()`) BUT `lib.rs` has NO `mod menu;` and does NOT export any of these symbols. Orphan file.                                                                                                                                                                                                                                                                                                                               | The derivation function is never callable. STEER-004 (which depends on this) has no way to populate menus.                                                                                                                                               |
| 4   | STEER-004 — exhaustive `(code, principal-resolution) → class` match + gate-state-aware `branch_protected(&cfg, branch)` | ⚠️ PARTIAL | Carrier structs carry the four fields: `crates/but-api/src/commit/gate.rs:17-28`, `crates/but-api/src/legacy/merge_gate.rs:36-42`, `crates/but-api/src/legacy/forge.rs:34-42`, `crates/but-api/src/legacy/config_mutate.rs:24-32` all add `class/held_permissions/authorized_actions/do_not`. BUT every carrier copies defaults (`commit/gate.rs:102-104` clones an empty `Denial`; constructors at `commit/gate.rs:113-115` hard-code `Vec::new()`/`None`). No exhaustive match — `Denial::new()` always returns `class: ActorCorrectable` (`denial.rs:166`). | Worse than missing: every denial — including no-handle and `config.invalid` — claims `actor_correctable` with an empty menu. STEER SPRINT.md §"class is exhaustive" contract is violated. `branch_protected` signature was not changed to accept `&cfg`. |
| 5   | STEER-005 — enrich the 4 CLI serializers + fault seam                                                                   | ❌ FAIL    | `crates/but/src/command/legacy/commit.rs:578-596` `commit_gate_cli_error` emits ONLY `{"error":{"code","message"}}` — no `class`, no `held_permissions`, no `authorized_actions`, no `do_not`. No `BUT_STEER_FORCE_SERIALIZATION_FAULT` fault seam anywhere (`git grep` returns 0 hits across `crates/**.rs`).                                                                                                                                                                                                                                                 | Every existing CLI denial path drops the steering fields. STEER-009's serialization-fault proof (which the contract keys on the exact env var `BUT_STEER_FORCE_SERIALIZATION_FAULT`) cannot be written.                                                  |
| 6   | STEER-006 — `but whoami` / `but can-i`                                                                                  | ❌ FAIL    | File `crates/but/src/command/whoami.rs` exists (420 lines, `exec_whoami`, `exec_can_i`, `WhoamiOutcome`, `CanIOutcome`) BUT `crates/but/src/command/mod.rs:1-19` has NO `pub mod whoami;` declaration. Orphan file.                                                                                                                                                                                                                                                                                                                                            | `git grep` for `whoami` across `crates/but/src/` returns only the file itself — zero references, no CLI registration, no `but whoami` subcommand reachable.                                                                                              |
| 7   | STEER-007 — denial-steering telemetry event                                                                             | ❌ FAIL    | `git grep -ln "steer_telemetry\|deny_telemetry\|had_lateral_action\|menu_len"` across `crates/**` on master returns **0 hits**.                                                                                                                                                                                                                                                                                                                                                                                                                                | No event, no `tracing::` call, no field. Entirely absent.                                                                                                                                                                                                |
| 8   | STEER-008 — agent-priming reference primer                                                                              | ❌ FAIL    | `crates/but-authz/tests/primer.rs` exists, references `but_authz::AGENT_PRIMER`, but `crates/but-authz/src/lib.rs:21` does NOT export `AGENT_PRIMER` and no module on master defines it.                                                                                                                                                                                                                                                                                                                                                                       | Test **would not compile on master**. Violates STEER SPRINT.md L2 contract: "prove no but-authz/but-api code path depends on it" — the constant itself does not exist.                                                                                   |
| 9   | STEER-009 — extend `governed_loop` (no-lying-menu replay × 3 denial types, serialization-fault, concurrent-ref-advance) | ❌ FAIL    | `crates/but/tests/but/command/governed_loop.rs` has exactly **5 `#[test]` functions** (`governed_loop_reference_flow_full_loop`, `_remediation_traversable`, `_dryrun_no_bypass`, `_auto_merge_denied`, `_unset_handle_failclosed`) — the original Sprint 04 set. `grep` for `authorized_action\|class\|lying\|replay\|steer\|menu` in test body: **0 hits**.                                                                                                                                                                                                  | No-lying-menu replay is the load-bearing proof of the sprint (§"The load-bearing mechanism — gate-state-aware derivation"). It is entirely missing.                                                                                                      |
| 10  | STEER-010 — net-new honesty build-gates (closed-catalog grep + table/affordance coverage grep)                          | ❌ FAIL    | `crates/but-authz/tests/invariant_build_gates.rs` has 2 `#[test]`s (`invariant_build_gates`, `assert_seeded_controls_fire`) — original Sprint 02 set. No `closed_catalog`, no `ROUTE_AUTHORITY_TABLE`-coverage, no `AFFORDANCE_MAP`-entry grep.                                                                                                                                                                                                                                                                                                                | The honesty guarantee that "every gated route ∈ `ROUTE_AUTHORITY_TABLE`; every table route has an `AFFORDANCE_MAP` entry" is unenforced. Combined with STEER-002/003 being orphans, the grep would have nothing to grep anyway.                          |

**Completion Gate**: 1 PASS + 1 PARTIAL + 8 FAIL → **`needs-revision`**. Master is NOT sprint-complete.

## STUB / ORPHAN FINDINGS

- **SEVERITY: CRITICAL**
  **TYPE**: Orphan source files (the sprint's "TDD preserves" landed but integration never did)
  **LOCATIONS**:
  - `crates/but-authz/src/route.rs` (full file — STEER-002)
  - `crates/but-authz/src/menu.rs` (full file — STEER-003)
  - `crates/but/src/command/whoami.rs` (full file — STEER-006)
  - `crates/but-authz/tests/steer_route_table.rs` (test imports unexported symbols)
  - `crates/but-authz/tests/steer_menu.rs` (test imports unexported symbols)
  - `crates/but-authz/tests/steer_class_mapping.rs` (test imports unexported symbols)
  - `crates/but-authz/tests/primer.rs` (test imports unexported `AGENT_PRIMER`)
  - `crates/but-api/tests/steer_envelope.rs` (depends on STEER-005 enrichment)
    **EVIDENCE**: `crates/but-authz/src/lib.rs` declares only `mod {assignment_state,authority,authorize,config,denial,principal};` — no `route`, no `menu`. `crates/but/src/command/mod.rs` declares only `{alias,branch,commit,completions,config,eval_hook,external,git_config,group,gui,help,move,onboarding,perm,push,skill,update}` — no `whoami`.
    **EXPECTED**: Per SPRINT.md task table, these modules must be declared, their symbols `pub use`'d, and the CLI subcommand wired into the `but` arg parser.
    **FIX**: Land the missing integration commits — `mod route; mod menu; pub use route::{Route, ReviewAction, ROUTE_AUTHORITY_TABLE}; pub use menu::{authorized_actions, DeniedRoute, DenialCategory, AFFORDANCE_MAP, CATALOG}; pub use primer::AGENT_PRIMER;` (and add the `primer` module), and `pub mod whoami;` in `command/mod.rs` plus the clap subcommand registration. Until that lands, **the orphan files give a false impression of completion** — they look implemented to a casual reader but compile to nothing.

- **SEVERITY: CRITICAL**
  **TYPE**: Half-implemented carrier (STEER-004 default-everywhere)
  **LOCATION**: `crates/but-authz/src/denial.rs:161-170` `Denial::new()`; `crates/but-api/src/{commit/gate.rs:102-104,113-115, legacy/merge_gate.rs:89-, legacy/forge.rs:113-140, legacy/config_mutate.rs:65-78}`
  **EVIDENCE**: `Denial::new()` hard-codes `class: DenialClass::ActorCorrectable, held_permissions: Vec::new(), authorized_actions: Vec::new(), do_not: None`. Carriers copy those defaults through. No `match` on `(code, principal_resolution)` exists. `branch_protected(principal, branch)` signature was not changed to accept `&cfg`.
  **EXPECTED**: SPRINT.md §Coverage Notes "class is exhaustive by (code, principal-resolution), not by code alone" — a non-defaulted `match` with no `_ =>` arm; `missing_permission` resolved-principal → `actor_correctable` with menu; no-handle / unknown-principal / `config.invalid` → `operator_required` with empty menu + do-not-retry `do_not`.
  **FIX**: Implement the `DenialCause`/`(code, principal-resolution)` exhaustive match (the cycle-3 red-hat SA-5 finding explicitly asked for a trybuild compile-fail proof — neither exists on master).

- **SEVERITY: HIGH**
  **TYPE**: Missing CLI serializer enrichment (STEER-005)
  **LOCATION**: `crates/but/src/command/legacy/commit.rs:578-596` and the parallel sites in `legacy/forge/review.rs:219`, `legacy/forge/review.rs` (merge_gate path), `command/group.rs:135` (governance_cli_error)
  **EVIDENCE**: `commit_gate_cli_error` constructs `serde_json::json!({"error":{"code":..., "message":...}})` — only two keys. No `class`, no `held_permissions`, no `authorized_actions`, no `do_not`. `git grep` for the contracted fault-seam env var `BUT_STEER_FORCE_SERIALIZATION_FAULT` across master returns 0 hits.
  **EXPECTED**: Per SPRINT.md Human Testing Gate step 1, a denied principal's stderr JSON must carry `class`, `held_permissions`, `authorized_actions`, `do_not`. Per STEER-009 AC (cycle-3 SA-1/RR-4 remediation), the test-only fault seam keyed on `BUT_STEER_FORCE_SERIALIZATION_FAULT` must exist, `cfg(test)`/dev-feature-gated, compiled out of release.
  **FIX**: Add the four fields to all four serializers; add the fault seam in a `#[cfg(any(test, feature = "steer-fault-seam"))]` block.

- **SEVERITY: HIGH**
  **TYPE**: Missing test extension (STEER-009)
  **LOCATION**: `crates/but/tests/but/command/governed_loop.rs` (only 5 original tests, 567 lines)
  **EVIDENCE**: `grep -E "authorized_action\|class\|operator_required\|actor_correctable\|lying\|replay\|steer\|menu"` in test body returns 0 hits. The no-lying-menu replay for `branch.protected`, `gate.review_required`, `perm.denied`, and admin-write (the cycle-3 M5 widening) is absent. The serialization-fault case (cycle-3 SA-1/RR-4) is absent. The concurrent-ref-advance re-denial case is absent.
  **EXPECTED**: SPRINT.md §"The load-bearing mechanism" — this is the runtime proof that the menu does not offer the denied action back. Without it, the sprint's headline guarantee is unproven.

- **SEVERITY: HIGH**
  **TYPE**: Missing honesty build-gates (STEER-010)
  **LOCATION**: `crates/but-authz/tests/invariant_build_gates.rs`
  **EVIDENCE**: Only 2 tests, both pre-STEER. No closed-catalog grep (no `format!`/`push_str`/`write!`/`concat!`/`Cow::Owned`/config-sourced text in `authorized_actions`/`do_not` construction). No table-coverage grep. No affordance-map-coverage grep. No grep for direct `class:` assignment bypassing the exhaustive match (cycle-3 M6).
  **EXPECTED**: SPRINT.md STEER-010 task row: "closed-catalog grep + table/affordance coverage grep + review".

- **SEVERITY: MEDIUM**
  **TYPE**: Missing telemetry event (STEER-007)
  **LOCATION**: nowhere in `crates/**`
  **EVIDENCE**: `git grep -ln "steer_telemetry\|had_lateral_action\|menu_len"` returns 0 hits across master.
  **EXPECTED**: A tracing event on the existing denial path emitting `code`, `class`, `had_lateral_action`, menu length.

## GAPS & RISKS

1. **Sprint status misrepresentation.** The presence of `.spec/prds/governance/tasks/sprint-08-*/STEER-001..010-*.md` plus orphan source files plus a `master` HEAD titled "STEER cycle-3 red-hat remediation: 11/11 findings CLOSED" gives every signal of a complete sprint — yet every task file still reads `STATUS: Backlog` and the integration layer is entirely missing. A reviewer skimming `master` will be misled. **Recommend** either (a) land the integration commits, or (b) tag the orphan files clearly as `//! NOTE: NOT WIRED — pending /kb-run-sprint sprint-08` until they land.
2. **The `Denial::new()` default returns `ActorCorrectable`.** This is not just missing — it is **actively wrong** for no-handle / unknown-principal / `config.invalid` denials, which must be `OperatorRequired`. On master today, a caller with no `BUT_AGENT_HANDLE` gets a denial claiming they can self-correct with an empty menu — the exact "pooling" failure mode STEER exists to prevent.
3. **Two sprint copies diverge.** Master carries BOTH `.spec/.../sprint-07-steer-*/` AND `.spec/.../sprint-08-steer-*/` with the same 10 task IDs. The sprint-07 set is the older draft; the sprint-08 set is the current one. Anyone resolving "STEER-003" by filename alone may review the wrong artifact.
4. **Cycle-3 red-hat remediation commit (`b8848c29fe`) only touched markdown.** Its message claims "11/11 findings CLOSED" and "Sprint 08 READY FOR /kb-run-sprint" — accurate as a spec-readiness statement, but easy to misread as code-complete. The commit's stat shows only `.spec/` files changed.
5. **STEER-009 serialization-fault contract is broken before it can be enforced.** STEER-005 was supposed to provide a `BUT_STEER_FORCE_SERIALIZATION_FAULT` seam; STEER-009 consumes it. Neither is on master, and the STEER-009 test cannot be written until STEER-005 ships it.

## ASSUMPTIONS (Unvalidated)

- **Assumed**: the orphan files (`route.rs`, `menu.rs`, `whoami.rs`) are the intended final implementations, just unwired. If instead they are mid-refactor snapshots, the real implementation may differ further. **Validate** by re-running the sprint.
- **Assumed**: master HEAD's "READY FOR /kb-run-sprint" was a spec-only claim. If it was intended as "code-complete", the gap is much larger than the commit message conveys.

## CONTRADICTIONS

- The `b8848c29fe` commit message ("Sprint 08 READY FOR /kb-run-sprint") vs the on-disk task statuses (`STATUS: Backlog` × 10). These are consistent if "READY" = spec-ready, contradictory if "READY" = code-ready.
- The STEER SPRINT.md "four denial carriers" statement vs the actual six-carrier scope remediated in cycle-3 (RR-1/RR-2). The SPRINT.md body acknowledges this in its Red-Hat Summary, but the Overview still says "four fields" / "four denial carriers" — minor spec drift, non-blocking.

## CONFIDENCE SUMMARY

- **HIGH** confidence findings: 4 (orphan files in `but-authz`; orphan `whoami.rs`; missing CLI enrichment; missing governed_loop extension)
- **MEDIUM** confidence findings: 2 (missing telemetry; missing honesty greps)
- **LOW** confidence findings: 0

## Recommendations by Category

1. **Gaps** — Run `/kb-run-sprint sprint-08-steer-capability-aware-denials` for real. The spec is reviewed and clean (cycle-3 closed); the code is not. The orphan files are useful pre-written GREEN starting points but are not the contract.
2. **Risks** — Fix `Denial::new()` default-first: either make the constructor private and force construction through classified per-code helpers, or default to `OperatorRequired` (safer) until STEER-004's exhaustive match lands.
3. **Assumptions** — Reconcile the two sprint-07/sprint-08 copies or delete sprint-07.
4. **Contradictions** — Append a one-line note to the cycle-3 commit's PR description clarifying "spec-only; code lands via /kb-run-sprint".

## Metadata

- **Reviewer**: single-agent red-hat (rust-reviewer stance), no panel dispatch
- **Confidence Framework**: HIGH = direct grep/read evidence on `master` HEAD; MEDIUM = absence-of-positive evidence
- **Scope**: `git show master:<file>` reads; no `--all` reflog evidence admitted
- **Report Generated**: 2026-06-23T00:16:23Z
- **Next Steps**: [Run /kb-run-sprint | Land integration commits | Reconcile sprint-07/08 duplicate]
