---
retrofit_audit: 1
prd: .spec/prds/governance/README.md
generated: 2026-06-23
mode: dry-run
tool: kb-e2e-retrofit
---

# Reality-Gate Retrofit Audit — Functional-Permission Agent Governance (POC)

> **Mode: DRY-RUN.** Audit + remediation plan only. No artifacts written.
> Re-run `/kb-e2e-retrofit .spec/prds/governance --apply` to write the catch-up
> sprint + re-sequenced ROADMAP.

## Summary verdict

**The PRD predates the Reality Gate entirely.** No `.spec/scenarios` flow registry
exists. Every sprint lacks a locked `human-flows.json` and every gate verdict is
agent-narrated, not code-computed. Two sprints shipped as "Done" on that basis —
they are **UNVERIFIED** and need catch-up. Six more have merged code without
derived flows. The load-bearing **T-LOOP-006 canary** and the **commit gate**
were never replayed at the human surface from a cold boot.

**Infra: GREEN.** All three surfaces (desktop · web · backend) have a PRESENT e2e
framework. No infra sprint needed.

## Infra (Goal 1)

`python3 tools/e2e-infra/detect_e2e_framework.py --repo <root> --json`

| Surface | Stack         | Status      | Framework  | Recommendation                 |
| ------- | ------------- | ----------- | ---------- | ------------------------------ |
| desktop | tauri-desktop | **PRESENT** | playwright | tauri-driver / WebdriverIO     |
| web     | sveltekit-web | **PRESENT** | playwright | Playwright                     |
| backend | rust-cli      | **PRESENT** | cargo-test | assert_cmd (real binary stdio) |

`any_missing: false` → **no INFRA sprint required.**

## Flow registry (core+edge)

`python3 tools/flow-coverage/flow_coverage_check.py --prd .spec/scenarios`
→ `[flow_coverage] no scenarios under .spec/scenarios`

**The PRD predates the flow-coverage contract.** `.spec/scenarios/` does not exist.
No UC has enumerated core or edge flows. The flow registry must be **backfilled**
before any catch-up or conformance work — the catch-up DERIVES from it; without a
complete registry, shipped flows will be forgotten.

### UCs requiring backfill

| UC group  | UCs             | Source                                                                               |
| --------- | --------------- | ------------------------------------------------------------------------------------ |
| AUTHZ     | UC-AUTHZ-01..04 | `04-uc-authz.md`                                                                     |
| GRPS      | UC-GRPS-01..02  | `05-uc-grps.md`                                                                      |
| GATES     | UC-GATES-01..02 | `06-uc-gates.md`                                                                     |
| LOOP      | UC-LOOP-01..02  | `07-uc-loop.md`                                                                      |
| MGMT      | UC-MGMT-01..07  | `08-uc-mgmt.md`                                                                      |
| **LPR**   | UC-LPR-01..07   | `tasks/sprint-07-local-agent-pr/SPRINT.md` + tech-delta (NOT in v1.3.0 PRD's 17 UCs) |
| **STEER** | UC-STEER-01..?  | `tasks/sprint-08-steer-capability-aware-denials/SPRINT.md` (NOT in v1.3.0 PRD)       |

> **PRD/ROADMAP drift.** Sprints 07 (LPR) and 08 (STEER) exist as task-expanded
> sprint directories with their own UCs (UC-LPR-01..07, UC-STEER-001..010) but are
> **absent from ROADMAP.md's sequence table** and **absent from the v1.3.0 PRD's
> 17-UC / 129-criteria count**. They are post-PRD extensions. The backfill must
> cover them so their flows are not lost.

## Sprints

| sprint                                  | SPRINT.md status      | ROADMAP status            | code on HEAD?                                        | locked `human-flows.json`? | gate evidence (`sprint-goal-state.json`)? | verdict                                         |
| --------------------------------------- | --------------------- | ------------------------- | ---------------------------------------------------- | -------------------------- | ----------------------------------------- | ----------------------------------------------- |
| 01a — AUTHZ + commit gate               | **Done** (2026-06-19) | "In Progress" **(drift)** | YES (`but-authz`, `commit_gate` tests, `566a36b9c8`) | **NO**                     | **NO**                                    | **UNVERIFIED → catch-up**                       |
| 01b — governed loop reference flow      | **Done** (2026-06-19) | Done                      | YES (`merge_gate`, `governed_loop` tests)            | **NO**                     | **NO**                                    | **UNVERIFIED → catch-up**                       |
| 02 — fail-closed + identity confinement | In Progress           | In Progress               | YES (red-hat review commits)                         | NO                         | NO                                        | **NONCONFORMANT → conformance re-plan**         |
| 03 — GRPS groups + ref-pin              | In Progress           | In Progress               | YES                                                  | NO                         | NO                                        | **NONCONFORMANT → conformance re-plan**         |
| 04 — gates deepening                    | In Progress           | In Progress               | YES                                                  | NO                         | NO                                        | **NONCONFORMANT → conformance re-plan**         |
| 05 — CLI `but perm`/`but group`         | In Progress           | In Progress               | YES                                                  | NO                         | NO                                        | **NONCONFORMANT → conformance re-plan**         |
| 06a — UI scaffold + principals + groups | In Progress           | In Progress               | YES                                                  | NO                         | NO                                        | **NONCONFORMANT → conformance re-plan**         |
| 06b — UI branch gates + rules + safety  | In Progress           | In Progress               | YES (cumulative UI merges)                           | NO                         | NO                                        | **NONCONFORMANT → conformance re-plan**         |
| 07 — local agent PR (LPR)               | **Planned**           | **NOT in roadmap table**  | **YES** (`REM-LPR-*` merges on HEAD)                 | NO                         | NO                                        | **NONCONFORMANT + DRIFT → conformance re-plan** |
| 08 — steer capability-aware denials     | Backlog               | NOT in roadmap table      | No                                                   | NO                         | NO                                        | PLANNED — conformance when sequenced            |

### Classification notes

- **UNVERIFIED (2):** Sprint 01a + 01b are marked Done, shipped code to HEAD, but
  have **no locked `human-flows.json`** and **no passing `sprint-goal-state.json`**.
  Fail-closed: their "green" was agent-narrated (`cargo test` counts typed into a
  closeout note), not a code-computed verdict from flows replayed at the human
  surface. **These drive the catch-up.**
- **NONCONFORMANT (7):** Sprint 02–06b are in-progress with merged code; Sprint 07
  is marked "Planned" but has `REM-LPR-*` commits on HEAD (the SPRINT.md itself
  flags prior "In Progress" as bookkeeping drift reopened 2026-06-23). None have
  derived human-flows. All need conformance re-planning before they can close
  under the gate.
- **PLANNED (1):** Sprint 08 (STEER) is Backlog with task files but no code.
  Conformance applies when it is sequenced.

### The load-bearing unreplayed flows

The two UNVERIFIED sprints carry the **walking skeleton** the entire PRD depends on:

1. **Commit gate (Sprint 01a, UC-GATES-01):** a read-only principal's commit is
   denied `perm.denied`; a `contents:write` principal's commit lands; branch
   protection is read **target-ref-only** so a working-tree `gates.toml` edit
   cannot unprotect the branch. **Never replayed at the human `but` CLI surface
   from a cold boot.**
2. **Reference-flow canary (Sprint 01b, T-LOOP-006):** the full
   implement→review→merge loop with three principals — implementer's merge +
   auto-merge denied, maintainer's merge succeeds only after a distinct reviewer
   approval at head, a denied implementer that follows its `remediation_hint`
   lands through a reviewed merge. **The PRD's load-bearing canary. Never replayed
   at the human surface from a cold boot.**

The PRD explicitly mandates T-LOOP-006 "must go green before the deep build."
Six sprints of deep build (02–06b) are now "In Progress" on top of a canary that
was never reality-gated. The catch-up is the first time these flows will run at
the human altitude from a cold-boot state.

### The cold-boot risk the gate is designed to catch

`REALITY-GATE.md` cites this exact repo's `sprint-06a` as the proving-ground case:
`coldboot_check.py` run against gitbutler's real e2e flags the **seeded-world bug**
— `permissions.toml`/`gates.toml` seeding and `BUT_AGENT_HANDLE` injection that
let `sprint-06a` go green while a cold-boot user saw a read-only UI. Sprint 06a
is currently "In Progress" with merged code and no cold-boot gate. The same class
of bug is presumed present until the catch-up runs.

## Remediation

### 1. Backfill the flow registry (blocking)

Invoke `kb-prd-plan --update` to enumerate core + edge functional flows
(`FUNCTIONAL-FLOW-COVERAGE.md`) for UCs covered by UNVERIFIED + NONCONFORMANT
sprints: **UC-AUTHZ-01..04, UC-GRPS-01..02, UC-GATES-01..02, UC-LOOP-01..02,
UC-MGMT-01..07, UC-LPR-01..07, UC-STEER-001..010.**

Re-run `flow_coverage_check.py --prd` until exit 0. This is the COMPLETE source to
derive from — no shipped flow may be forgotten.

### 2. No infra sprint

All surfaces PRESENT. Skip.

### 3. Catch-up sprint (scope: UNVERIFIED sprints 01a + 01b)

- **Scope:** the human flows of the two Done sprints — the commit gate
  (UC-GATES-01) + the reference-flow canary (UC-LOOP-01/02, T-LOOP-006) — plus
  the cross-sprint journey between them (the walking-skeleton chain:
  commit-denied → review → merge-permitted).
- **Gate:** replay every shipped flow for real — cold boot, the real `but` CLI,
  real git, no seeded `permissions.toml`/`gates.toml` injection at test time
  (commit them as the human would). The verdict is computed by
  `tools/gate-evidence/` from exit codes + fresh artifacts + coverage, not
  narrated.
- **Human Test Deliverable + locked `human-flows.json`:** DERIVED from the
  backfilled registry for UC-GATES-01 + UC-LOOP-01/02. Every core + edge flow;
  tag `type:core|edge`, `uc_ref`, `scope:journey` for the walking-skeleton chain.
  Run `flow_coverage_check.py --sprint` to confirm coverage.
- **Tasks:** one verify task per flow. RED flows (revealed at run time) become fix
  tasks. **It is EXPECTED to go RED** on whatever shipped broken — that is the
  deliverable, not a failure.
- **Depends on:** nothing (the infra is already green).

### 4. Conformance re-plan (scope: NONCONFORMANT sprints 02–06b + 07)

For each NONCONFORMANT sprint: regenerate via `kb-sprint-plan` +
`kb-sprint-tasks-plan` so it:

- derives `human-flows.json` from the backfilled registry for its in-scope UCs;
- binds `flow_ref` on every PRIMARY AC;
- tags `journey` flows (esp. cross-sprint journeys through the governance stack);
- depends on the catch-up (no sprint closes until the skeleton is real-green).

Sprint 07 (LPR) additionally needs its bookkeeping drift reconciled — it has
merged code on HEAD while marked "Planned."

### 5. Rewrite ROADMAP.md

Insert the catch-up sprint **AHEAD** of the remaining feature sprints. Add
sprints 07 (LPR) and 08 (STEER) to the sequence table (currently absent).
Reconcile the 01a "In Progress" vs "Done" drift.

## What --apply will write

1. `.spec/prds/governance/tasks/sprint-00-catchup-walking-skeleton/SPRINT.md`
2. `.spec/prds/governance/tasks/sprint-00-catchup-walking-skeleton/human-flows.json`
3. Rewritten `.spec/prds/governance/ROADMAP.md` (catch-up inserted; 07/08 added)
4. Self-check: `flow_coverage_check.py --sprint` on the catch-up passes.

**It will NOT run `/kb-run-sprint`.** The flow-revealing step is always
human-initiated.
