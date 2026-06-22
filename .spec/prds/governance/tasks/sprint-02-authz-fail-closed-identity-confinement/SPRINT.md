---
sprint: 02
sequence: 3
timeline: Phase 2 — Hardening
status: In Progress
proposed_by: rust-planner
milestone: sprint-02
prd: ../../README.md
roadmap: ../../ROADMAP.md
generated_by: kb-sprint-tasks-plan
---

# Sprint 02: AUTHZ Fail-Closed + Identity Confinement

**Sequence:** 3
**Timeline:** Phase 2 — Hardening
**Status:** In Progress
**Proposed by:** rust-planner
**Milestone:** — (`sprint-02`)

## Overview

The first hardening sprint after the walking skeleton. Sprint 01a/01b proved the _positive_ governed
loop (a `contents:write` commit lands, a reviewed `merge`-holder's merge is permitted) and the basic
denial paths. Sprint 02 makes the **negative space** sound: the engine must **fail closed** on every
absence-or-corruption-of-an-answer, and a principal must not be able to exceed its authority by
**borrowing another identity** or by **presenting its own authority claim**.

Concretely this sprint hardens four properties over the merge/forge gate built in Sprint 01b and the
`but-authz` primitive built in Sprint 01a:

- **Fail-closed determinism (AUTHZ-004):** an unknown principal → `perm.denied`; malformed target-ref
  config → `config.invalid`; a `require_approval_from_group` naming an **undefined** group → a hard
  deny (never vacuously satisfied). The `config.invalid` vs `perm.denied` discrimination is
  deterministic — malformed/unparseable config is _always_ `config.invalid`, an unknown/missing
  principal is _always_ `perm.denied`, and the two never blur.
- **Identity confinement (AUTHZ-005):** a dispatched agent's authority comes **only** from its
  injected `BUT_AGENT_HANDLE` resolved against committed config — a governed action under the
  dispatched handle is denied when its committed authority is insufficient, no in-band identity
  override is honored, and an agent-supplied authority claim is ignored. The env-var residual (a
  process re-exporting `BUT_AGENT_HANDLE`) is documented as an **accepted leak**, not claimed as
  confinement.
- **Admin-write primitive (AUTHZ-006):** `administration:write` is the authority that governs every
  config-mutating path, checked at the target ref, so config changes cannot self-authorize.
- **Honesty invariants re-asserted (AUTHZ-008):** the no-role-name / no-human-vs-AI-predicate /
  no-`Permission`-overload grep-gates are re-run after the hardening so the new fail-closed and
  confinement code did not smuggle a role string or a repo-lock overload into an enforcement path.

Every gate proof draws from [`11-e2e-testing-criteria.md`](../../11-e2e-testing-criteria.md). This
sprint is **headless/CLI** — every property is verified by running a `but` command and observing the
structured denial `{code, message, remediation_hint}` + exit code.

> **Accepted-leak honesty (carried from UC-AUTHZ-03).** Identity confinement is scoped honestly: no
> in-band `--as <other>` identity override is honored (the unsupported flag is rejected before a
> governed action runs) and authority is never taken from an agent claim, but a process that
> _re-exports_ `BUT_AGENT_HANDLE` before invoking `but` is **not** prevented
> (personal-tenant trusts the orchestrator). AUTHZ-005 tests the in-band denial + claim-ignored
> property only — it never asserts the env-var re-export is blocked, which would encode a false
> guarantee.

## Human Testing Gate

**Gate:** An action by an unknown principal, with no handle, against malformed config, naming an
undefined required group, or borrowing another handle is denied with the exact structured code
instead of running.

### Test Steps

1. Run a merge as a principal absent from `permissions.toml` → denied, exit 1, `perm.denied`.
2. Run a merge with `BUT_AGENT_HANDLE` unset → rejected, exit 1, no anonymous action.
3. Commit a malformed `gates.toml` to the target ref, run merge → denied, exit 1, `config.invalid`.
4. Run merge whose `gates.toml` names an undefined group → denied, not vacuously satisfied.
5. Run a governed action as a dispatched reviewer → denied, exit 1, `perm.denied`; attempt
   `--as <other>` → rejected as an unsupported flag, exits non-zero, and no action runs as the
   borrowed handle.
6. Inject an agent-supplied authority claim → ignored; authority comes from committed config only.
7. Re-run the reference-flow canary (T-LOOP-006) → still green after hardening.

## Tasks

| ID                         | Title                                                                                                                                                                          | Agent            | Estimate |
| -------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------- | -------- |
| AUTHZ-004                  | Merge/forge-gate fail-closed + `config.invalid` vs `perm.denied` determinism + undefined-group hard-deny                                                                       | rust-implementer | 150 min  |
| AUTHZ-005                  | Identity confinement — no honored in-band identity override + handle-only resolution (honest accepted-leak)                                                                    | rust-implementer | 150 min  |
| AUTHZ-006                  | `administration:write` authority primitive on the config-mutating path                                                                                                         | rust-implementer | 120 min  |
| AUTHZ-008                  | Re-assert the honesty invariant grep-gates after AUTHZ hardening                                                                                                               | rust-reviewer    | 45 min   |
| FIX-AUTHZ-FORGE-FAILCLOSED | Forge governance opt-in fail-OPEN on absence — swap bespoke `has_governance_config` for canonical `but_authz::governance_present` (permissions-OR-gates, fail-closed-on-error) | rust-implementer | 120 min  |
| FIX-AUTHZ-FORGE-COVERAGE   | AUTHZ-008 honesty gate false coverage over `forge.rs` — fully-qualify `authorize` + assert AUTHORITY_POSITIVE & PERMISSION_CARRIER over `FORGE_GUARD`                          | rust-implementer | 75 min   |
| FIX-AUTHZ-006-RED-EVIDENCE | Produce + commit missing `.tmp/AUTHZ-006/` RED-against-start + seeded evidence per the AUTHZ-006 REQUIREMENT-CONTRACT                                                          | rust-implementer | 60 min   |

## Dependencies

- **Blocks:** Sprint 05, Sprint 06a
- **Dependent on:** Sprint 01b

## PRD Coverage

- **Use cases:** UC-AUTHZ-03 (confinement primitive), UC-AUTHZ-04
- **Criteria:** T-AUTHZ-018/019/020/021/023/026/027/028/029/030/031/016/022, T-LOOP-005/011

> The self-grant-inert ref-pin (T-AUTHZ-032) and `perm list` scoping (T-AUTHZ-025) were relocated to
> their natural gate homes — Sprint 03 and Sprint 05 — so every step here is covered by this sprint's
> fail-closed/confinement gate.

## Capability Coverage

- **CAP-AUTHZ-01** — fail-closed enforcement (AUTHZ-004), confinement (AUTHZ-005), admin-write
  primitive (AUTHZ-006).
- **CAP-CONFIG-01** — `config.invalid` on malformed target-ref config; admin-write checked at the
  target ref (AUTHZ-004/006).

## Coverage Notes

- **Depends on Sprint 01b (In Progress, not yet merged):** AUTHZ-004 hardens the merge/forge gate
  built in GATES-003 and the requirement plumbing in GATES-005, and AUTHZ-006 governs the config
  surface that Sprint 05 (`but perm`/`but group`) will write. The task files plan against the
  documented 01b contracts; if a 01b contract drifts during its build, re-run
  `/kb-sprint-tasks-plan --only <id> --overwrite`.
- **Accepted-leak (UC-AUTHZ-03, by design):** the `BUT_AGENT_HANDLE` env-var re-export residual is a
  documented accepted-leak. No test asserts a re-exporting process is blocked.
- **Undefined-group hard-deny scope:** AUTHZ-004 covers the **merge/forge** gate's `config.invalid`
  determinism + undefined-`require_approval_from_group` hard-deny **in the but-api merge-gate layer**
  (review_requirement/merge_gate) — `but-authz` `config.rs` has no `[[gate]]`/`require_approval_from_group`
  schema and GATES-003's `writeProhibited` forbids adding one there, so AUTHZ-004 carries a
  `BLOCKED-UNTIL` note (the 01b gate-requirement schema must land in the merge layer first). The broader
  mechanism-agnostic commit-gate coverage and the dedicated standalone target-ref-only proofs
  (T-GATES-016..019) remain **Sprint 04** (GATES-007/008).
- **Intra-sprint serialization (scheduling, red-hat M3):** the dependency graph is a strict chain —
  AUTHZ-005 is blocked on **both** AUTHZ-004 and AUTHZ-006 (it composes the merge gate + the admin-write
  guard function), and AUTHZ-008 is blocked on all three (plus AUTHZ-007). Run **AUTHZ-004 and AUTHZ-006
  first** (the only parallelizable pair), then AUTHZ-005, then AUTHZ-008. There is effectively no
  intra-sprint parallelism beyond the 004/006 pair.
- **Test Step 7 owner (T-LOOP-006 canary, red-hat M4):** Test Step 7 ("re-run the reference-flow canary
  → still green") is owned by **no new Sprint-02 task** — it is the existing Sprint-01b **LOOP-001**
  reference-flow integration test running **unchanged in CI** as a regression check. A reviewer verifies
  it with `cargo test -p but governed_loop_reference_flow_full_loop`; it must stay green after the
  Sprint-02 fail-closed hardening lands.
- **Implementation is out of scope for this artifact:** these are TDD **task contracts**; the Rust
  (`merge_gate.rs` hardening, the new `config_mutate.rs` guard, the integration/build-gate tests) is
  written at execution time by `/kb-run-sprint`, RED→GREEN against these specs. The Sprint-02 source
  surfaces intentionally do not exist yet (and `merge_gate.rs` is itself a pending Sprint-01b GATES-003
  deliverable — only GATES-002 has landed so far).

## Red-Hat Review Summary

Expanded by `/kb-sprint-tasks-plan` on 2026-06-19 — **1 full red-hat cycle + retained-writer remediation +
a fresh confirmation pass**. A fresh panel (`rust-reviewer` + `security-auditor`, no authoring context)
BLOCKed the first draft with **4 CRITICAL + 7 MEDIUM + 3 LOW** findings — all _specification-correctness_
gaps the structural + fakeability gates cannot catch:

- **C1** — AUTHZ-004's undefined-`require_approval_from_group` hard-deny rested on a gate schema that does
  **not exist** in `but-authz` `config.rs` (`GatesWire` parses only `[[branch]]`), with an unresolved
  GATES-003-`writeProhibited` contradiction and a GATES-008 ownership race → re-grounded to the but-api
  merge-gate layer with an honest `BLOCKED-UNTIL` note; the wrong `config.rs:304-309` cite (a principal's
  `groups=[]` membership, a different field) corrected.
- **C2** — AUTHZ-006/005's "config-mutate seam that exists now" was **fictional** (no but-api config-write
  entrypoint) → AC-1 re-scoped to exercise the reusable `enforce_administration_write_gate` **function** as
  real integration (real config-load + real `authorize`), Sprint-05 named as the persisted-write consumer,
  the admin-Ok case kept falsifiable by the same-test dev-denied half.
- **C3** — AUTHZ-005's `--as`-rejected proof was **vacuous** (passed even with a disconnected resolver) →
  load-bearing positive assertion now "resolved principal id == `BUT_AGENT_HANDLE`" (denial names the env
  principal).
- **C4** — AUTHZ-008's build-gate had a hole (no `AUTHORITY_POSITIVE` over `merge_gate.rs`, a phantom
  test-only "confinement path" entry, no in-code length assertion) → AUTHORITY_POSITIVE now asserted over
  **both** `merge_gate.rs` and `config_mutate.rs`, the phantom entry dropped, `ENFORCEMENT_PATHS.len() >= 5`
  asserted in code, fully-qualified `but_authz::authorize` required.

All four CRITICALs were remediated by the writer and the fixes **independently verified in the AC bodies**
(not reworded). The fresh confirmation pass agreed every specification fix is correct; its residual BLOCK was
a phase-scope error (it expected implementation code from a task-planning step) and is **not applicable** —
the Rust is written by `/kb-run-sprint`. Deterministic re-validation: **4/4 tasks fakeability-CLEAN**
(`validate_scenario.py`, 13/13 scenarios, 0 CRITICAL/HIGH); `proposed_by` tripwire 4/4; stable AC-N/TC-N IDs;
avg rubric ≈110/115.

## Task Detail Files

Generated by `/kb-sprint-tasks-plan` on 2026-06-19.

- AUTHZ-004-merge-gate-fail-closed.md
- AUTHZ-005-identity-confinement.md
- AUTHZ-006-administration-write-guard.md
- AUTHZ-008-honesty-invariant-build-gates.md

### Remediation tasks — Red-Hat Review 2026-06-19 (`../../../reviews/red-hat-sprint-02-2026-06-19.md`)

A post-merge adversarial red-hat review (rust-reviewer + security-reviewer + security-auditor)
returned **NEEDS_FIXES — 1 HIGH + 2 MEDIUM**, all on the forge boundary AUTHZ-008 pulled into scope
plus one evidence-policy gap. The core merge-gate / confinement / admin-write properties were
verified genuinely enforced (no CRITICAL, no test-theatre). Remediation tasks proposed by
rust-planner, to be executed RED→GREEN via `/kb-run-sprint`:

- FIX-AUTHZ-FORGE-FAILCLOSED.md — **HIGH (H-1)**: `forge.rs::has_governance_config` is fail-OPEN
  (checks only `permissions.toml`, propagates errors); non-merge forge verbs run ungoverned for a
  `gates.toml`-only repo. Swap to canonical `but_authz::governance_present` (both files, fail-closed).
- FIX-AUTHZ-FORGE-COVERAGE.md — **MEDIUM (M-1)**: `FORGE_GUARD` is in `ENFORCEMENT_PATHS` but
  `AUTHORITY_POSITIVE` / `PERMISSION_CARRIER` are not asserted over `forge.rs` (which uses a bare
  `authorize`), so the honesty gate claims coverage it does not enforce. Fully-qualify + assert.
  (Depends on FIX-AUTHZ-FORGE-FAILCLOSED.)
- FIX-AUTHZ-006-RED-EVIDENCE.md — **MEDIUM (M-2)**: `.tmp/AUTHZ-006/` RED + seeded evidence missing
  despite the contract requiring it. Produce + commit genuine RED-against-start + seeded artifacts.

Tracked (non-blocking, not remediated here): DryRun framing (Sprint 06a); orphan admin-write guard
must be wired by Sprint 05 or it rots.
