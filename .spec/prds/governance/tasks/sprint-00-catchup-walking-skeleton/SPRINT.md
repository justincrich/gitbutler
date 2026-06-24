---
sprint: 00
sequence: 0
timeline: Phase 0 — Reality-Gate catch-up (walking skeleton re-verification)
status: Completed
proposed_by: kb-e2e-retrofit
milestone: sprint-00
prd: ../../README.md
roadmap: ../../ROADMAP.md
generated_by: kb-e2e-retrofit --apply
---

# Sprint 00: Reality-Gate Catch-Up — Walking Skeleton Re-Verification

**Sequence:** 0 (precedes all feature sprints)
**Timeline:** Phase 0 — Reality-Gate catch-up
**Status:** Planned
**Proposed by:** `kb-e2e-retrofit` (derived from RETROFIT-AUDIT.md)
**Milestone:** — (`sprint-00`)

## Why this sprint exists

Sprints **01a** (AUTHZ + commit gate) and **01b** (governed loop reference flow) shipped as
"Done" on **agent-narrated** `cargo test` counts — typed into a closeout note. No locked
`human-flows.json` was ever produced. No `sprint-goal-state.json` verdict was ever computed by
code. The **T-LOOP-006 canary** (the PRD's load-bearing walking skeleton) and the **commit gate**
were never replayed at the human `but` CLI surface from a cold-boot state with the verdict
computed from real exit codes.

This sprint replays every shipped flow of those two sprints **for real** — cold boot, the real
`but` binary via assert_cmd/snapbox (real git + real but-db, no mocks), verdict computed by
`tools/gate-evidence/` from exit codes + fresh artifacts + coverage. The agent's words carry no
weight.

> **It is EXPECTED to go RED** on whatever shipped broken. The Reality-Gate proving ground
> (`REALITY-GATE.md`) cites this exact repo's `sprint-06a` as the case where `coldboot_check.py`
> caught a real seeded-world bug. RED flows revealed at run time become fix tasks. The catch-up
> is the first time these flows run under the gate. That is the deliverable, not a failure.

## Human Testing Gate

**Gate:** Every core + edge flow of UC-AUTHZ-01/02/04, UC-GATES-01/02, UC-LOOP-01/02 is replayed
at the real `but` CLI surface from a cold boot, and the verdict is **green** — computed by code
from exit codes, not narrated.

### Test Steps (derived from the backfilled `.spec/scenarios` registry)

The flows are locked in `human-flows.json`. Each has a `run_cmd` that drives the real binary.
The gate replays ALL of them + runs the altitude/cold-boot lints.

**Commit-gate flows (UC-AUTHZ-01/02/04, UC-GATES-01):**

1. `cargo test -p but-authz` — the authorize() primitive resolves authority sets from committed config.
2. `cargo test -p but-api commit_gate` — commit gate allow/deny (contents:write lands; read-only denied;
   protected branch denied; absent config ungoverned; malformed config.invalid).
3. `cargo test -p but --features but-2 commit_gate` — CLI-surface commit-gate tests.

**Merge-gate flows (UC-GATES-02):** 4. `cargo test -p but-api merge_gate` — merge gate allow/deny (merge after approval proceeds; zero
approvals denied; stale denied; auto-merge denied; DryRun no-bypass).

**Reference-flow canary (UC-LOOP-01/02) — the load-bearing JOURNEY:** 5. `cargo test -p but --features but-2 governed_loop` — the T-LOOP-006 canary: 5/5
(`reference_flow_full_loop`, `remediation_traversable`, `dryrun_no_bypass`, `auto_merge_denied`,
`unset_handle_failclosed`).

**Honesty invariant (UC-LOOP-02):** 6. `cargo test -p but-authz invariant_build_gates` — no role-name, no human-vs-AI predicate, no
Permission overload in any enforcement path.

**Altitude + cold-boot lints (on every flow):** 7. `tools/e2e-surface/e2e_surface_check.py` — confirms each test enters at the human `but` CLI
surface, not beneath it. 8. `tools/e2e-surface/coldboot_check.py` — confirms each test starts cold; no seeded/injected
`permissions.toml`/`gates.toml` or `BUT_AGENT_HANDLE` world. The config must be committed as a
human would.

> **Cold-boot is the load-bearing lint.** The `REALITY-GATE.md` proving ground demonstrated that
> `coldboot_check.py` catches the exact class of bug where tests seed governance config at test
> time (rather than committing it), letting a flow go green while a cold-boot user sees different
> behavior. If the existing 01a/01b tests seed config via env vars or working-tree writes instead
> of `git commit`, this lint WILL flag it — and the fix task is to make the tests commit config
> the way the human does.

## Scope

| UC          | Core                                             | Edge                                           | Journey                      |
| ----------- | ------------------------------------------------ | ---------------------------------------------- | ---------------------------- |
| UC-AUTHZ-01 | ✓ permission model from committed config         | ✓ absent principal denied; agent claim ignored |                              |
| UC-AUTHZ-02 | ✓ structured denial {code,message,hint} + exit 1 | ✓ unset handle rejected                        |                              |
| UC-AUTHZ-04 | ✓ malformed → config.invalid                     | ✓ partial config → config.invalid              |                              |
| UC-GATES-01 | ✓ contents:write commit lands                    | ✓ protected branch; working-tree edit inert    |                              |
| UC-GATES-02 | ✓ merge after approval proceeds                  | ✓ no/stale approval; auto-merge; DryRun        |                              |
| UC-LOOP-01  | ✓ 3-principal reference flow                     | ✓ remediation traversal                        | **✓ walking-skeleton chain** |
| UC-LOOP-02  | ✓ functional-no-role                             | ✓ honesty invariant grep                       |                              |

## Tasks

Each task is one **verify** flow (replay the locked `run_cmd` + lints). If the replay goes RED,
the task becomes a **fix** task — diagnose, fix, re-run until the code-computed verdict is green.

| ID          | Flow                                                       | Type       | run_cmd                                                                                                          |
| ----------- | ---------------------------------------------------------- | ---------- | ---------------------------------------------------------------------------------------------------------------- |
| CATCHUP-001 | Commit-gate flows (UC-AUTHZ-01/02/04, UC-GATES-01)         | verify/fix | `cargo test -p but-authz && cargo test -p but-api commit_gate && cargo test -p but --features but-2 commit_gate` |
| CATCHUP-002 | Merge-gate flows (UC-GATES-02)                             | verify/fix | `cargo test -p but-api merge_gate`                                                                               |
| CATCHUP-003 | Reference-flow canary T-LOOP-006 (UC-LOOP-01/02) [JOURNEY] | verify/fix | `cargo test -p but --features but-2 governed_loop && cargo test -p but-authz invariant_build_gates`              |
| CATCHUP-004 | Altitude + cold-boot lints on ALL flows                    | verify/fix | `tools/e2e-surface/e2e_surface_check.py` + `tools/e2e-surface/coldboot_check.py` per flow                        |

> **CATCHUP-004 is the load-bearing task.** If the existing tests fail the cold-boot lint (they
> seed config instead of committing it), CATCHUP-001/002/003 become fix tasks too — the tests
> must be rewritten to commit governance config the way a human does, not inject it.

## Dependencies

- **Blocks:** ALL remaining feature sprints (02–08). No sprint closes until the walking skeleton
  is reality-gate green.
- **Dependent on:** None (infra is PRESENT; the code shipped on HEAD).

## PRD Coverage

- **Use cases:** UC-AUTHZ-01, UC-AUTHZ-02, UC-AUTHZ-04, UC-GATES-01, UC-GATES-02, UC-LOOP-01, UC-LOOP-02
- **Criteria:** T-AUTHZ-001/003/004/009/010/012/016/024/027/028/029, T-GATES-001..011, T-LOOP-001..007/010/013

## What "done" means

The `tools/gate-evidence/gate_evidence_check.py` writes `sprint-goal-state.json` with `verdict:
pass` — computed from:

- Every locked flow's `run_cmd` exit code = 0
- `e2e_surface_check.py` = `surface_ok` for every flow
- `coldboot_check.py` = `coldboot_ok` for every flow
- `flow_coverage_check.py --sprint` = exit 0 (every in-scope UC bound)

Until that file exists and says `pass`, sprints 01a and 01b are **UNVERIFIED** and every sprint
that depends on them is **BLOCKED**.

## Task Detail Files

Generated by `/kb-sprint-tasks-plan` (specialist: rust-planner) on 2026-06-23. Ground truth for
each flow was established by running the locked `run_cmd`s (exit codes, not narrated) before
expansion — see each task's Background.

- [CATCHUP-001-commit-gate-flows.md](./CATCHUP-001-commit-gate-flows.md) — commit-gate flows (UC-AUTHZ-01/02/04, UC-GATES-01); parts 1&2 GREEN, part 3 RED on 2 STEER snapshot-drift failures
- [CATCHUP-002-merge-gate-flows.md](./CATCHUP-002-merge-gate-flows.md) — merge-gate flows (UC-GATES-02); GREEN 14/14
- [CATCHUP-003-reference-flow-canary.md](./CATCHUP-003-reference-flow-canary.md) — T-LOOP-006 canary (UC-LOOP-01/02) [JOURNEY]; GREEN 20/20, all 5 canary green
- [CATCHUP-004-altitude-coldboot-lints.md](./CATCHUP-004-altitude-coldboot-lints.md) — altitude + cold-boot lints [LOAD-BEARING]; tooling MISSING, must be built

Intra-sprint order: CATCHUP-001/002/003 may run in parallel (independent flows); CATCHUP-004
depends on all three (the gate verdict requires every flow green first).
