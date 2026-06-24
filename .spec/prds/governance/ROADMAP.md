---
roadmap: 2
project: Functional-Permission Agent Governance for GitButler (POC)
generated: 2026-06-18
retrofitted: 2026-06-23
remediation_added: 2026-06-23
prd: .spec/prds/governance/README.md
sprint_count: 11
pr_sequencing: false
---

# Sprint Roadmap: Functional-Permission Agent Governance for GitButler (POC)

## Overview

**Sprints:** 11 (1 catch-up ✅ + 8 feature + LPR + STEER + 1 remediation) — 10 Done, 1 Planned
**Total Tasks:** 81 (35 original + 4 catch-up + 11 LPR + 10 STEER + 5 LPR-remediation + 16 Sprint-09 remediation)
**Current Sprint:** Sprint 09 (Governance Remediation — LPR/MGMT Hardening) — Planned 2026-06-23 after independent codebase audit surfaced blocking gaps in Sprint 07 (LPR) and Sprint 06b (MGMT)

> **Sprint 09 added 2026-06-23 (remediation).** An independent codebase investigation (parallel subagent fanout + test execution) found that Sprint 07 (LPR) and Sprint 06b (MGMT) had 6 blocking gaps despite their "Done" status: Branch Gates tab was a stub, `but review comment/comments/resolve` CLI verbs were stubbed or missing, `keep_reviews_local` did not persist, `process_commit_rules` was unwired, LPR-009 safe-seam invariant was incomplete, and 4 test suites had compile/runtime failures. Sprint 09 remediates these. See [`Sprint 09`](#sprint-09-governance-remediation--lprmgmt-hardening) below.

> **Retrofit (2026-06-23).** `/kb-e2e-retrofit --apply` inserted **Sprint 00** (catch-up) ahead
> of all feature sprints. The catch-up **passed** (14/14 flows green) — Sprints 01a + 01b are
> now **VERIFIED**. See [`RETROFIT-AUDIT.md`](./RETROFIT-AUDIT.md). Sprints 07 (LPR) and 08
> (STEER) — post-PRD extensions folded into the v1.5.0 PRD — now have Per-Sprint Details (delta-replan 2026-06-23).

This roadmap turns GitButler into a commit/merge **policy-enforcement layer** for orchestrated agents: a new
`but-authz` crate + two gates (commit + merge) over GitButler's own git actions, principal grouping, and an
admin governance UI in `apps/desktop`. Sequencing honors the PRD mandate — the **proven-reference-flow canary
(T-LOOP-006)** is the walking skeleton that must go green _before_ the deep build, so Sprint 1 is a thin
vertical slice (ref-pin loader + fail-closed primitive + a single observable commit allow/deny) and Sprint 2
completes the loop. Every sprint's human-testing gate draws its proof from
[`11-e2e-testing-criteria.md`](./11-e2e-testing-criteria.md).

The product is **headless/CLI** for the AUTHZ · GRPS · GATES · LOOP groups (gates are verified by running a
`but` command and observing the structured denial `{code, message, remediation_hint}` + exit code) and a
**desktop UI** for the MGMT group (verified by using the Governance settings page).

> **Planning provenance.** Sprint content was authored by the dispatched specialist set — `rust-planner`
> (backend/CLI), `tauri-planner` (IPC seam), `sveltekit-planner` (UI), `frontend-designer` (design) — then
> hardened through one red-hat review cycle (`rust-reviewer` + `sveltekit-reviewer` + `security-auditor`),
> which surfaced 8 CRITICAL gaps (an ungated `set_review_auto_merge` path, un-gated `integrate`/`apply`
> ref-advancing paths, an unowned `gates.toml` writer, the `isAdmin` wiring gap, DryRun/commit-gate
> target-ref proofs, and the missing T-LOOP-013 traversability proof) — all remediated by the original
> writers. The orchestrator consolidated; it did not author sprint content.

## Sprint Sequence

| #   | Milestone | Sprint                                                                                                                | Gate                                                                                                                                                       | Tasks | Dependencies                                | Status                                           |
| --- | --------- | --------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | ----- | ------------------------------------------- | ------------------------------------------------ |
| 0   | —         | [Sprint 00: Reality-Gate Catch-Up — Walking Skeleton](#sprint-00-reality-gate-catch-up--walking-skeleton)             | Every shipped flow of 01a+01b replayed at the `but` CLI surface, cold-boot, verdict code-computed                                                          | 4     | —                                           | **Completed** (verdict: pass, 14/14 flows green) |
| 1   | —         | [Sprint 01a: AUTHZ Primitive + Commit Gate](#sprint-01a-authz-primitive--commit-gate)                                 | Read-only commit denied; `contents:write` commit lands; protection read target-ref-only                                                                    | 5     | Sprint 00                                   | Done — **VERIFIED** (catch-up pass, 14/14)       |
| 2   | —         | [Sprint 01b: Governed Loop Reference Flow](#sprint-01b-governed-loop-reference-flow)                                  | 3-principal loop: merge + auto-merge gated; channel traversable                                                                                            | 5     | Sprint 00                                   | Done — **VERIFIED** (catch-up pass, 14/14)       |
| 3   | —         | [Sprint 02: AUTHZ Fail-Closed + Identity Confinement](#sprint-02-authz-fail-closed--identity-confinement)             | Unknown principal / no handle / bad config / borrowed identity denied with exact code                                                                      | 4     | Sprint 00                                   | Done                                             |
| 4   | —         | [Sprint 03: GRPS Groups + Ref-Pin](#sprint-03-grps-groups-ref-pin)                                                    | Group grant inherited; self-add / self-grant still denied (target-ref read)                                                                                | 2     | Sprint 00                                   | Done                                             |
| 5   | —         | [Sprint 04: GATES Deepening](#sprint-04-gates-deepening)                                                              | Stale/self/single-group merge blocked; commit gate covers integrate/apply/worktree                                                                         | 3     | Sprint 00, Sprint 03                        | Done                                             |
| 6   | —         | [Sprint 05: CLI `but perm` / `but group`](#sprint-05-cli-but-perm--but-group)                                         | Admin grants/groups/lists via CLI with ref-pin caveat; non-admin denied                                                                                    | 2     | Sprint 00, Sprint 02, 03, 04                | Done                                             |
| 7   | —         | [Sprint 06a: Governance UI — Scaffold + Principals + Groups](#sprint-06a-governance-ui--scaffold--principals--groups) | Admin edits principal & group permissions on the Governance page; pending until commit                                                                     | 16    | Sprint 00, Sprint 02, Sprint 05             | Done                                             |
| 8   | —         | [Sprint 06b: Governance UI — Branch Gates + Rules + Safety](#sprint-06b-governance-ui--branch-gates-rules-safety)     | Branch-gate edit pending; rules scoped; read-only + denial-no-flip safety                                                                                  | 11    | Sprint 06a, Sprint 04                       | Done                                             |
| 9   | —         | [Sprint 07: Local Agent PR — Governed-Review Parity (LPR)](#sprint-07-local-agent-pr--governed-review-parity-lpr)     | Local review loop: assignment/comment/derived-PR/agent-tag; safe-seam proven                                                                               | 16+5  | Sprint 00, Sprint 01b, Sprint 04, Sprint 05 | Done                                             |
| 10  | —         | [Sprint 08: Steer — Capability-Aware Denials](#sprint-08-steer--capability-aware-denials)                             | Denial carriers include steering fields; route-authority single-source; `but whoami`/`can-i`                                                               | 10    | Sprint 00                                   | Done                                             |
| 11  | —         | [Sprint 09: Governance Remediation — LPR/MGMT Hardening](#sprint-09-governance-remediation--lprmgmt-hardening)        | Full local review loop runs CLI comment/resolve + Branch Gates tab renders open assignments/threads + keep-reviews-local persists + commit auto-rules fire | 16    | Sprint 00, 04, 05, 06b, 07, 08              | Done                                            |

_Milestone cells are `—` until the sprints are materialized as GitHub Milestones._

### Dependency graph

```
00 (CATCH-UP ✅ — BLOCKS ALL)
  │
  ├→ 01a (Done ✅) ──→ 01b (Done ✅) ──→ ┐
  │                                       ├→ 02 (Done ✅) ──────┬→ 05 (Done ✅) → 06a (Done ✅) → 06b (Done ✅)
  └→──────────────────────────────────────┼→ 03 (Done ✅) ──┬───┤              ↑(also 04)
                                          └→ 04 (Done ✅) ◄┘   │
                                             04 ───────────────┘
  07 (LPR, Done ✅) ← 01b + 04 + 05     08 (STEER, Done ✅) ← 00
  09 (REMEDIATION, Planned) ← 00 + 04 + 05 + 06b + 07 + 08
```

---

## Per-Sprint Details

### Sprint 00: Reality-Gate Catch-Up — Walking Skeleton

**Sequence:** 0
**Timeline:** Phase 0 — Reality-Gate catch-up (walking-skeleton re-verification)
**Status:** Completed (verdict: pass — 14/14 flows green, code-computed)
**Proposed by:** `kb-e2e-retrofit`
**Milestone:** — (`sprint-00`)

#### Human Testing Gate

**Gate:** Every core + edge flow of UC-AUTHZ-01/02/04, UC-GATES-01/02, UC-LOOP-01/02 replayed at the
real `but` CLI surface from a cold boot, verdict code-computed by `tools/gate-evidence/`.

Full detail in [`tasks/sprint-00-catchup-walking-skeleton/SPRINT.md`](./tasks/sprint-00-catchup-walking-skeleton/SPRINT.md)

- locked [`human-flows.json`](./tasks/sprint-00-catchup-walking-skeleton/human-flows.json).

#### Tasks

| ID          | Flow                                                            | Type       | run_cmd                                                                                                          |
| ----------- | --------------------------------------------------------------- | ---------- | ---------------------------------------------------------------------------------------------------------------- |
| CATCHUP-001 | Commit-gate flows (UC-AUTHZ + UC-GATES-01)                      | verify/fix | `cargo test -p but-authz && cargo test -p but-api commit_gate && cargo test -p but --features but-2 commit_gate` |
| CATCHUP-002 | Merge-gate flows (UC-GATES-02)                                  | verify/fix | `cargo test -p but-api merge_gate`                                                                               |
| CATCHUP-003 | T-LOOP-006 canary + honesty invariant (UC-LOOP-01/02) [JOURNEY] | verify/fix | `cargo test -p but --features but-2 governed_loop && cargo test -p but-authz invariant_build_gates`              |
| CATCHUP-004 | Altitude + cold-boot lints on ALL flows                         | verify/fix | `e2e_surface_check.py` + `coldboot_check.py` per flow                                                            |

#### Dependencies

- **Blocks:** ALL feature sprints (01a–08). No sprint closes until the walking skeleton is reality-gate green.
- **Dependent on:** None (infra PRESENT; code shipped on HEAD).

#### Coverage

- 14 locked human flows across 7 UCs (UC-AUTHZ-01/02/04, UC-GATES-01/02, UC-LOOP-01/02)
- Flow registry backfilled: `.spec/scenarios/{UC-AUTHZ-01,UC-AUTHZ-02,UC-AUTHZ-04,UC-GATES-01,UC-GATES-02,UC-LOOP-01,UC-LOOP-02}/`
- `flow_coverage_check.py --prd` = exit 0; `--sprint` = exit 0

---

### Sprint 01a: AUTHZ Primitive + Commit Gate

**Sequence:** 1
**Timeline:** Phase 1 — Walking skeleton
**Status:** Done
**Proposed by:** rust-planner
**Milestone:** — (`sprint-01a`)

#### Human Testing Gate

**Gate:** Running a commit as a read-only principal is denied `perm.denied` while a `contents:write` principal's commit lands, and protection is read only from the target-ref blob so a working-tree `gates.toml` edit cannot unprotect the branch.

**Test Steps:**

1. Seed committed `.gitbutler/permissions.toml` (`ro` contents:read; `dev` contents:write) + `gates.toml` (`main` protected).
2. Run commit on a feature branch as `dev` → exit 0, ref advances.
3. Run commit on a feature branch as `ro` → denied, exit 1, `perm.denied` names contents:write.
4. Run direct commit to protected `main` as `dev` → denied, exit 1, `branch.protected`.
5. Run commit with `BUT_AGENT_HANDLE` unset → rejected, exit 1, no anonymous action.
6. Edit working-tree `gates.toml` to unprotect `main`, commit directly to `main` as `dev` → still `branch.protected`.
7. Run commit as a principal absent from committed `permissions.toml` → denied, `perm.denied`.
8. Commit a malformed `gates.toml` to the target ref, run a commit → denied, `config.invalid` (fail-closed).

#### Tasks

| ID        | Title                                                                                    | Agent            | Estimate |
| --------- | ---------------------------------------------------------------------------------------- | ---------------- | -------- |
| AUTHZ-001 | Create `but-authz` crate: `Authority`, `AuthoritySet`, `Principal`, `Group`, `Denial`    | rust-implementer | 180 min  |
| AUTHZ-002 | Ref-pinned governance config loader (`gix`, target-ref blob read)                        | rust-implementer | 210 min  |
| AUTHZ-003 | `authorize()` + `BUT_AGENT_HANDLE` resolution + fail-closed default-deny                 | rust-implementer | 180 min  |
| GATES-001 | Commit gate at `commit_engine::create_commit` (target-ref-only, DryRun-enforced)         | rust-implementer | 240 min  |
| AUTHZ-007 | Invariant build-gates — no role name, no human-vs-AI predicate, no `Permission` overload | rust-reviewer    | 90 min   |

#### Dependencies

- Blocks: Sprint 01b
- Dependent on: None

#### PRD Coverage

- UC-AUTHZ-01, UC-AUTHZ-02, UC-AUTHZ-04, UC-GATES-01
- Criteria: T-AUTHZ-001/003/004/009/010/012/016/024/027/028/029, T-GATES-001..007, T-LOOP-005/011

#### Capability Coverage

- **CAP-AUTHZ-01** — producer: `authorize()` (AUTHZ-003); the commit gate (GATES-001) runs **even under DryRun**.
- **CAP-CONFIG-01** — producer: ref-pinned loader (AUTHZ-002); the commit gate reads branch-protection **target-ref-only** (a working-tree `gates.toml` edit cannot weaken it).

#### Next Sprint Tasks

Expanded by `/kb-sprint-tasks-plan` on 2026-06-18 (avg 111/115 rubric · fakeability floor 0/0/0/0 · 1 full red-hat review cycle, 20 findings resolved). Detail files in [`tasks/sprint-01a-authz-primitive-commit-gate/`](./tasks/sprint-01a-authz-primitive-commit-gate/):

- `AUTHZ-001-create-but-authz-crate.md`
- `AUTHZ-002-ref-pinned-config-loader.md`
- `AUTHZ-003-authorize-handle-resolution.md`
- `GATES-001-commit-gate.md`
- `AUTHZ-007-invariant-build-gates.md`

> **Material re-plan in expansion (red-hat):** GATES-001's commit gate was re-sited from `commit_engine::create_commit` to the ref-aware **`but-api` `_with_authz` seam + CLI commit path** (authorize **before** the `RepoExclusive` guard, per `04-api-design.md`) — `create_commit` cannot name the target branch. `T-GATES-016/017` (mechanism-agnostic + worktree) are explicitly deferred to Sprint 04.

---

### Sprint 01b: Governed Loop Reference Flow

**Sequence:** 2
**Timeline:** Phase 1 — Walking skeleton (the T-LOOP-006 canary)
**Status:** Done — closed out 2026-06-19 (T-LOOP-006 canary green; 5/5 tasks merged + re-verified)
**Proposed by:** rust-planner
**Milestone:** — (`sprint-01b`)

#### Human Testing Gate

**Gate:** Running the reference flow with three principals, the implementer's merge AND auto-merge are denied, the maintainer's merge succeeds only after a distinct reviewer approval at head, and a denied implementer that follows its `remediation_hint` lands through a reviewed merge.

**Test Steps:**

1. Run `but` merge as implementer (`reviews:write`, no `merge`) → denied, exit 1, `perm.denied` names merge.
2. Enable auto-merge (`but review --auto-merge`) as that implementer → denied, exit 1, `perm.denied`.
3. Run commit as reviewer → denied; submit review as reviewer → exit 0, recorded at head.
4. Run merge as maintainer with zero distinct approvals → denied, exit 1, `gate.review_required`.
5. Run merge as maintainer after a distinct reviewer approval at head → exit 0, merge proceeds.
6. Advance head, rerun merge → denied, `gate.review_required` with `approval_stale_at_head`.
7. Run a DryRun merge as the implementer → still denied `perm.denied`, nothing persisted.
8. Follow the denied implementer's `remediation_hint` (feature branch → review → merge) → lands successfully.

#### Tasks

| ID        | Title                                                                                | Agent            | Estimate |
| --------- | ------------------------------------------------------------------------------------ | ---------------- | -------- |
| GATES-002 | Local review record: `but-db` `local_review_verdicts` table (head-pinned)            | rust-implementer | 150 min  |
| GATES-003 | Merge gate covering **both** `merge_review` AND `set_review_auto_merge`              | rust-implementer | 270 min  |
| GATES-004 | Submit-review / open-PR / comment authz guards on the forge boundary                 | rust-implementer | 150 min  |
| GATES-005 | Stale-approval-@head dismissal + self-approval exclusion                             | rust-implementer | 150 min  |
| LOOP-001  | Reference-flow test (T-LOOP-006) + traversable proof (T-LOOP-013) + DryRun-no-bypass | rust-implementer | 240 min  |

#### Dependencies

- Blocks: Sprint 02, Sprint 03, Sprint 04
- Dependent on: Sprint 01a

#### PRD Coverage

- UC-LOOP-01, UC-LOOP-02, UC-GATES-02 (review record)
- Criteria: T-LOOP-006/001/002/003/004/007/010/**013**, T-GATES-008/009/010/011/014/015

#### Capability Coverage

- **CAP-AUTHZ-01** — merge/auto-merge gate (GATES-003), forge guards (GATES-004); DryRun-no-bypass proven (LOOP-001).
- **CAP-CONFIG-01** — merge gate reads requirement at the target ref (GATES-003).

#### Next Sprint Tasks

Expanded by `/kb-sprint-tasks-plan` on 2026-06-18 (5/5 tasks fakeability-CLEAN · `proposed_by` tripwire 5/5 · 1 full red-hat cycle — fresh `rust-reviewer` + `security-auditor`; 1 CRITICAL resolved by the user's gate-boundary re-scope + upstream advisory, 5 MEDIUM remediated, all 9 prior CRITICALs confirmed closed). Detail files in [`tasks/sprint-01b-governed-loop-reference-flow/`](./tasks/sprint-01b-governed-loop-reference-flow/):

- `GATES-002-local-review-record.md`
- `GATES-003-merge-gate.md`
- `GATES-004-forge-authz-guards.md`
- `GATES-005-stale-self-approval.md`
- `LOOP-001-reference-flow-test.md`

> **Gate-boundary re-scope (red-hat, user decision; wording RESOLVED 2026-06-24).** `merge_review`/`publish_review` are forge-bound (error on a bare local repo), so the **positive** governed-merge/PR-open paths prove the gate **DECISION** (permit/deny) on the real seam + that execution reaches the forge call; the forge-network **completion** is proven structurally (forge-locality accepted limitation, see [01-scope.md](./01-scope.md#known-limitations)). **Resolved:** the T-LOOP-004/006/010/012 and UC-LOOP-01/02 "merge succeeds / change lands / proceeds" wording is reconciled to "the gate permits and execution reaches the governed `merge_review` boundary" in [`07-uc-loop.md`](./07-uc-loop.md) and [`11-e2e-testing-criteria.md`](./11-e2e-testing-criteria.md). The earlier "no `but pr merge` CLI verb" note was **stale** — the `but pr merge` / `but pr auto-merge` verbs do exist (`crates/but/src/args/forge.rs` `forge::pr::Merge`/`AutoMerge`, exercised by `governed_loop` `pr merge`). T-LOOP-011 (human-vs-AI grep) deferred to Sprint 04.

---

### Sprint 02: AUTHZ Fail-Closed + Identity Confinement

**Sequence:** 3
**Timeline:** Phase 2 — Hardening
**Status:** Done
**Proposed by:** rust-planner
**Milestone:** — (`sprint-02`)

#### Human Testing Gate

**Gate:** An action by an unknown principal, with no handle, against malformed config, naming an undefined required group, or borrowing another handle is denied with the exact structured code instead of running.

**Test Steps:**

1. Run a merge as a principal absent from `permissions.toml` → denied, exit 1, `perm.denied`.
2. Run a merge with `BUT_AGENT_HANDLE` unset → rejected, exit 1, no anonymous action.
3. Commit a malformed `gates.toml` to the target ref, run merge → denied, exit 1, `config.invalid`.
4. Run merge whose `gates.toml` names an undefined group → denied, not vacuously satisfied.
5. Run a governed action as a dispatched reviewer → denied, exit 1, `perm.denied`; attempt
   `--as <other>` → rejected as an unsupported flag, exits non-zero, and no action runs as the
   borrowed handle.
6. Inject an agent-supplied authority claim → ignored; authority comes from committed config only.
7. Re-run the reference-flow canary (T-LOOP-006) → still green after hardening.

#### Tasks

| ID        | Title                                                                                                       | Agent            | Estimate |
| --------- | ----------------------------------------------------------------------------------------------------------- | ---------------- | -------- |
| AUTHZ-004 | Merge/forge-gate fail-closed + `config.invalid` vs `perm.denied` determinism + undefined-group hard-deny    | rust-implementer | 150 min  |
| AUTHZ-005 | Identity confinement — no honored in-band identity override + handle-only resolution (honest accepted-leak) | rust-implementer | 150 min  |
| AUTHZ-006 | `administration:write` authority primitive on the config-mutating path                                      | rust-implementer | 120 min  |
| AUTHZ-008 | Re-assert the honesty invariant grep-gates after AUTHZ hardening                                            | rust-reviewer    | 45 min   |

#### Dependencies

- Blocks: Sprint 05, Sprint 06a
- Dependent on: Sprint 01b

#### PRD Coverage

- UC-AUTHZ-03 (confinement primitive), UC-AUTHZ-04
- Criteria: T-AUTHZ-018/019/020/021/023/026/027/028/029/030/031/016/022, T-LOOP-005/011
- _(The self-grant-inert ref-pin (T-AUTHZ-032) and `perm list` scoping (T-AUTHZ-025) were relocated to their natural gate homes — Sprint 03 and Sprint 05 — so every step here is covered by this sprint's fail-closed/confinement gate.)_

#### Capability Coverage

- **CAP-AUTHZ-01** — fail-closed enforcement (AUTHZ-004), confinement (AUTHZ-005), admin-write primitive (AUTHZ-006).
- **CAP-CONFIG-01** — `config.invalid` on malformed target-ref config; admin-write checked at the target ref.

#### Next Sprint Tasks

Expanded by `/kb-sprint-tasks-plan` on 2026-06-19 (4/4 tasks fakeability-CLEAN · `proposed_by` tripwire 4/4 ·
avg rubric ≈110/115 · 1 full red-hat cycle — fresh `rust-reviewer` + `security-auditor`; 4 CRITICAL + 7 MEDIUM

- 3 LOW spec-correctness findings all remediated and the fixes independently verified in the AC bodies, then
  confirmed by a fresh pass). Detail files in [`tasks/sprint-02-authz-fail-closed-identity-confinement/`](./tasks/sprint-02-authz-fail-closed-identity-confinement/):

* `AUTHZ-004-merge-gate-fail-closed.md`
* `AUTHZ-005-identity-confinement.md`
* `AUTHZ-006-administration-write-guard.md`
* `AUTHZ-008-honesty-invariant-build-gates.md`

> **Red-hat re-grounding (spec correctness).** The first draft was fakeability-clean and well-structured but
> rested on two non-existent surfaces the structural gates can't see: the undefined-`require_approval_from_group`
> hard-deny (AUTHZ-004) stood on a gate schema absent from `but-authz` `config.rs` (and forbidden there by
> GATES-003's `writeProhibited`), and the admin-write guard (AUTHZ-006/005) leaned on a config-mutate seam that
> won't exist until the Sprint-05 `but perm`/`but group` verbs. Both were re-grounded honestly — the
> undefined-group hard-deny to the but-api merge-gate layer with a `BLOCKED-UNTIL` note, the admin-write proof
> to the reusable guard **function** (real config-load + real `authorize`) with Sprint-05 named as the
> persisted-write consumer. Intra-sprint order is a strict chain: **AUTHZ-004 + AUTHZ-006 → AUTHZ-005 → AUTHZ-008**.

---

### Sprint 03: GRPS Groups + Ref-Pin

**Sequence:** 4
**Timeline:** Phase 2 — Hardening
**Status:** Done
**Proposed by:** rust-planner
**Milestone:** — (`sprint-03`)

#### Human Testing Gate

**Gate:** Gate passes when BOTH: [GRPS-01] a member with no direct grant is authorized via its group's permission (union) and denied an action no source grants; AND [GRPS-02] a feature head that adds its own author to a merge-holding group — or self-grants `administration:write` — is still denied, because membership and grants are read only at the target ref.

**Test Steps:**

1. Seed a `code-reviewers` group with `reviews:write` and a member holding no direct review grant.
2. Run a review as that member → exit 0, authorized via the group (union).
3. Run a merge as that member → denied, exit 1, `perm.denied` (no source grants merge).
4. Create a feature head that adds its author to a `merge`-holding `maintainers` group.
5. Run merge from that feature head → denied, the target-ref membership governs.
6. Self-grant `administration:write` on a feature head, run the same config change → denied (inert until target-ref commit).
7. Commit the membership to the target ref, advance it, rerun the merge → now authorized.

#### Tasks

| ID       | Title                                                                    | Agent            | Estimate |
| -------- | ------------------------------------------------------------------------ | ---------------- | -------- |
| GRPS-001 | Effective-set union via group membership + group permission ceiling      | rust-implementer | 150 min  |
| GRPS-002 | Ref-pinned governed membership + self-grant-inert (target-ref-only read) | rust-implementer | 210 min  |

#### Dependencies

- Blocks: Sprint 04, Sprint 05
- Dependent on: Sprint 01b

#### PRD Coverage

- UC-GRPS-01, UC-GRPS-02, UC-AUTHZ-03 (self-grant-inert, relocated here)
- Criteria: T-GRPS-001..014, T-AUTHZ-032

#### Capability Coverage

- **CAP-AUTHZ-01** — union resolution (GRPS-001).
- **CAP-CONFIG-01** — membership + grants read target-ref-only; no self-escalation (GRPS-002).

#### Next Sprint Tasks

Expanded by `/kb-sprint-tasks-plan` on 2026-06-19 (2/2 tasks fakeability-CLEAN · `proposed_by` tripwire 2/2 ·
avg rubric ≈113/115 · 1 full red-hat cycle — fresh `rust-reviewer` + `security-auditor`; 2 spec-correctness
CRITICALs (the fictional union-"divergence" thesis and the impossible AC-2 "authorizes Ok") + the actionable
MEDIUM/LOW set all remediated by the retained writer and confirmed against the live crate). Detail files in
[`tasks/sprint-03-grps-groups-ref-pin/`](./tasks/sprint-03-grps-groups-ref-pin/):

- `GRPS-001-effective-set-union-group-ceiling.md`
- `GRPS-002-ref-pinned-membership-self-grant-inert.md`

> **Material re-grounding in expansion (red-hat):** GRPS-001 was re-framed from "fix a redundant double-union
> _divergence_" (a fiction — `effective_authority` is provably equal to `principal_authorities` by
> construction) to an honest **behavior-neutral simplification** that removes the dead authorize-time re-union
> and **pins the equality**. GRPS-002's positive AC-2 was re-scoped from "the merge authorizes (Ok)" (impossible
> under the fixture's unapproved review gate) to "the `perm.denied` at the `Authority::Merge` step is cleared",
> with the residual `gate.review_required` named as the expected next gate; its PRIMARY driver seeds the
> `ForgeReview` inline via the real public `forge_reviews_mut().upsert` seam. `but group` CLI verbs (T-GRPS-001/
> 002/006/010-CLI) remain re-grounded to Sprint 05 (CLI-002); T-GRPS-006/011 admin-gating proven at the authz
> layer via the AUTHZ-006 guard.

---

### Sprint 04: GATES Deepening

**Sequence:** 5
**Timeline:** Phase 2 — Hardening
**Status:** Done
**Proposed by:** rust-planner
**Milestone:** — (`sprint-04`)

#### Human Testing Gate

**Gate:** Gate passes when BOTH: [MERGE-STRICTNESS] a merge with a stale, self, or single-group-only approval is blocked and lands only with a distinct approval from each required group at the current head; AND [COMMIT-COVERAGE] a protected-branch commit, integrate, and apply are each rejected `branch.protected` through the same commit gate, and a working-tree `gates.toml` edit cannot weaken either gate.

**Test Steps:**

1. Approve a PR at head H1, advance to H2, run merge → blocked, `gate.review_required` with `approval_stale_at_head`.
2. Have the author approve their own change (distinct required), run merge → blocked, requirement unmet.
3. Supply only a `code-reviewers` approval, run merge → blocked (maintainers required); only `maintainers` → blocked.
4. Supply a distinct approval from each required group at head, run merge → exit 0, proceeds.
5. Commit on `but worktree new` to a feature branch, then to protected `main` → feature accepted, `main` rejected `branch.protected`.
6. Run `integrate_branch_with_steps` / `branch apply` advancing protected `main` as contents:write → rejected `branch.protected`.
7. Edit `gates.toml` on the feature head to drop the merge requirement, run merge → still judged by the target-ref requirement.

#### Tasks

| ID        | Title                                                                                                                                               | Agent            | Estimate |
| --------- | --------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- | -------- |
| GATES-006 | Per-required-group approval evaluation (two-group AI + human model)                                                                                 | rust-implementer | 150 min  |
| GATES-007 | Mechanism-agnostic commit gate — **actually** gate `branch::apply`, `integrate_branch_with_steps`, worktree-integrate                               | rust-implementer | 270 min  |
| GATES-008 | Standalone target-ref-only read proof for the merge gate — feature-head requirement-drop ignored (deepening; AUTHZ-004 owns merge-path fail-closed) | rust-implementer | 120 min  |

#### Dependencies

- Blocks: Sprint 05, Sprint 06b
- Dependent on: Sprint 01b, Sprint 03

#### PRD Coverage

- UC-GATES-01 (mechanism-agnostic coverage), UC-GATES-02, UC-LOOP-02
- Criteria: T-GATES-012/013/016/017/019, T-LOOP-008/009/010/011/012 _(T-GATES-018 owned by AUTHZ-004/Sprint 02 — re-proven by GATES-008's target-ref-only deepening; T-LOOP-011 lands as GATES-006 AC-3; T-GATES-016/017 re-grounded — apply/integrate prove contents:write, worktree_integrate proves branch.protected)_

> **Upstream advisory (red-hat).** Sprint-04 gate step 6 ("`integrate_branch_with_steps` / `branch apply` advancing protected `main` → `branch.protected`") assumes a flow the live code contradicts (`branch::apply` bails on the target; `integrate_branch_with_steps` writes a feature branch; only `worktree_integrate` advances `target`). Reconcile via `/kb-sprint-plan --delta-replan` to: contents:write enforced on apply + integrate (read-only denied) + `worktree_integrate` advancing a protected target → `branch.protected`.

#### Capability Coverage

- **CAP-AUTHZ-01** — per-group approval evaluation (GATES-006).
- **CAP-CONFIG-01** — commit gate covers every ref-advancing entry point (GATES-007); both gates read target-ref-only (GATES-007/008).

#### Next Sprint Tasks

Expanded by `/kb-sprint-tasks-plan` on 2026-06-19 (3/3 tasks fakeability-CLEAN · `proposed_by` tripwire 3/3 · avg rubric 115/115 · 1 full red-hat goal loop, 3 cycles — fresh `rust-reviewer` + `security-auditor`; 5 CRITICAL + 8 MEDIUM + LOW resolved, incl. a fabricated `governance_present` grounding inversion, the incoherent apply/integrate-advances-main framing re-grounded against live code, and three remediation-introduced fixes (config_only vacuous-gate, lock-ordering placement, no-target regression) all re-verified). Detail files in [`tasks/sprint-04-gates-deepening/`](./tasks/sprint-04-gates-deepening/):

- `GATES-006-per-required-group-approval.md`
- `GATES-007-mechanism-agnostic-commit-gate.md`
- `GATES-008-merge-gate-failclosed-target-ref-only.md`

> **Material re-grounding in expansion (red-hat):** GATES-007's mechanism-agnostic coverage was re-grounded against live code — `branch::apply` bails on the target and `integrate_branch_with_steps` writes a feature branch (neither advances a protected trunk), so the gate proves **contents:write/perm.denied on apply + integrate** (via `config_only(workspace target ref)`, fired at the public seam before the worktree guard, no-target = ungoverned = permit) and **`worktree_integrate` advancing a protected `target` → `branch.protected`** (via `direct_ref(target)`) — not "apply/integrate advancing main". GATES-008 was re-grounded to the **standalone target-ref-only read proof** (AUTHZ-004 retains merge-path fail-closed ownership; T-GATES-018). **Upstream advisory:** reconcile the Human-Testing-Gate **step 6** wording (and T-GATES-018's Sprint-04 listing) via `/kb-sprint-plan --delta-replan`.

---

### Sprint 05: CLI `but perm` / `but group`

**Sequence:** 6
**Timeline:** Phase 3 — CLI governance management
**Status:** Done
**Proposed by:** rust-planner
**Milestone:** — (`sprint-05`)

#### Human Testing Gate

**Gate:** An admin runs `but perm grant` and `but group add-member`, sees the takes-effect-once-committed caveat, `but perm list` shows the committed effective set plus the new grant as pending, and a non-admin's grant or cross-principal list is denied `perm.denied`.

**Test Steps:**

1. Run `but perm grant --principal rust-implementer reviews:write` as an admin → exit 0, prints the ref-pin caveat.
2. Run `but perm list --principal rust-implementer` → shows the committed effective set (unchanged) and the new grant as PENDING (not yet in effect).
3. Run `but group create code-reviewers --permissions reviews:write` → exit 0, `[[group]]` written.
4. Run `but group add-member code-reviewers --principal rust-reviewer` → exit 0, ref-pin caveat printed.
5. Run `but perm grant ...` as a non-admin principal → denied, exit 1, `perm.denied`.
6. Run `but perm list --principal <other>` as a non-admin (not self, no admin:read) → denied, `perm.denied`, no recon.
7. Run `but perm revoke --principal rust-implementer reviews:write` as admin → exit 0.

#### Tasks

| ID      | Title                                                                                    | Agent            | Estimate |
| ------- | ---------------------------------------------------------------------------------------- | ---------------- | -------- |
| CLI-001 | `but perm {list,grant,revoke}` + admin-write gating + ref-pin caveat + perm-list scoping | rust-implementer | 240 min  |
| CLI-002 | `but group {create,grant,add-member,remove-member,list}` + admin-write gating            | rust-implementer | 210 min  |

#### Dependencies

- Blocks: Sprint 06a
- Dependent on: Sprint 02, Sprint 03, Sprint 04

#### PRD Coverage

- UC-AUTHZ-01, UC-AUTHZ-03 (`perm list` scoping, relocated here), UC-GRPS-01, UC-GRPS-02
- Criteria: T-AUTHZ-007/021/025, T-GRPS-001/002/006/010/011
- _Authors the `but-api` perm/group governance functions reused as Tauri commands in Sprint 06a._

#### Capability Coverage

- **CAP-AUTHZ-01** + **CAP-CONFIG-01** — CLI write path authorizes `administration:write` and writes inert-until-committed config (CLI-001/002).

#### Next Sprint Tasks

Expanded by `/kb-sprint-tasks-plan` on 2026-06-19 (2/2 tasks fakeability-CLEAN · `proposed_by` tripwire 2/2 ·
avg rubric 115/115 · 1 full red-hat goal loop, 2 cycles — fresh `rust-reviewer` + `security-auditor`; 3 CRITICAL

- 5 MEDIUM + 3 LOW spec-correctness findings all remediated by the retained writer and confirmed genuinely closed
  by a fresh cycle-2 pass). Detail files in [`tasks/sprint-05-cli-perm-group/`](./tasks/sprint-05-cli-perm-group/):

* `CLI-001-perm-cli-verbs.md`
* `CLI-002-group-cli-verbs.md`

> **Net-new persisted writer + honesty-grep extension (red-hat).** This is the first sprint that **persists**
> governed config — `but-authz` `config.rs` is loader-only, so CLI-001 authors the working-tree TOML
> read-modify-write (raw serde wire structs, `#[derive(Serialize)]` added additively — NOT a lossy `GovConfig`
> round-trip), a `permissions_path()` accessor, and the `but-api` `perm_*`/`group_*` functions sited beside
> `config_mutate.rs` for Sprint-06a Tauri reuse. Admin gating **composes** the Sprint-02 AUTHZ-006
> `enforce_administration_write_gate` (never re-implements it); the writes are inert-until-committed (working tree
> only). The red-hat loop forced a **CRITICAL** fix: CLI-001 extends the AUTHZ-007/008 honesty grep
> (`invariant_build_gates.rs` `ENFORCEMENT_PATHS` + a positive-Authority assertion) to cover the net-new
> `governance.rs`, and added the missing positive `perm revoke` proof (gate step 7) and the `T-AUTHZ-007`
> seeded-day-one-effectiveness AC. `but group delete` stays a Sprint-06a/UC-MGMT-03 consumer (explicitly not
> shipped). **Intra-sprint order is a strict chain: CLI-001 → CLI-002** (CLI-002 extends `governance.rs` + the
> shared `crates/but/src/args` wiring; rebase, do not run in parallel worktrees).

---

### Sprint 06a: Governance UI — Scaffold + Principals + Groups

**Sequence:** 7
**Timeline:** Phase 4 — Governance management UI
**Status:** Done
**Proposed by:** sveltekit-planner + tauri-planner + frontend-designer + rust-planner (MGMT backend); split authored by sveltekit-reviewer (red-hat)
**Milestone:** — (`sprint-06a`)

#### Human Testing Gate

**Gate:** An admin opens Project Settings, sees the Permissions & Governance sidebar item, navigates to it, observes four tabs, edits a principal's own-grant permission on the Principals tab, expands a group and grants a permission on the Groups tab, and sees both changes remain pending with a commit banner until clicking Commit changes.

**Test Steps:**

1. Open the GitButler desktop app and open Project Settings via the existing shortcut.
2. Observe the Permissions & Governance sidebar item shows for an admin; sign in as non-admin, confirm absent.
3. Click Permissions & Governance and observe four tabs: Principals, Groups, Branch Gates, Rules.
4. On Principals, click a principal row, toggle an own-grant permission on; observe a pending circle and commit banner.
5. On Groups, expand a group and grant a permission; observe the pending circle and banner persist across tabs.
6. Click Commit changes and observe all pending indicators clear and the effective set update.

#### Tasks

| ID              | Title                                                                                                         | Agent                 | Estimate |
| --------------- | ------------------------------------------------------------------------------------------------------------- | --------------------- | -------- |
| MGMT-IPC-001    | `#[but_api]` governance fns (perm/group/status) + `json::Error` transport                                     | rust-implementer      | 90 min   |
| MGMT-IPC-002    | `json.rs` `Error` serializes the 3rd field `remediation_hint` (closes a real drop bug)                        | rust-implementer      | 75 min   |
| MGMT-IPC-003    | Register governance commands in `generate_handler!` + capability + v1 human-fleet-owner identity (T-MGMT-042) | tauri-implementer     | 60 min   |
| MGMT-IPC-004    | Regenerate `packages/but-sdk` (perm/group/status) — SDK build-gate before UI wiring                           | tauri-implementer     | 45 min   |
| MGMT-IPC-005    | Pending-until-committed read IPC contract (working-tree vs target-ref)                                        | tauri-implementer     | 60 min   |
| MGMT-UI-001     | Register the governance page + extend `ProjectSettingsPageId` (in `uiState.svelte.ts`)                        | sveltekit-implementer | 30 min   |
| MGMT-UI-002     | `ProjectSettingsModalContent` governance branch + **wire `isAdmin`** to `SettingsModalLayout`                 | sveltekit-implementer | 45 min   |
| MGMT-UI-003     | `GovernanceSettings.svelte` + client-only pending-state store (no `+page.server.ts`)                          | sveltekit-implementer | 90 min   |
| MGMT-UI-005     | `GovernancePendingBanner` (warning InfoMessage + Commit action)                                               | sveltekit-implementer | 30 min   |
| MGMT-UI-006     | `PrincipalsList` (rows + inline editor; inherited rows read-only)                                             | sveltekit-implementer | 90 min   |
| MGMT-UI-007     | `PrincipalEditor` (SegmentControl presets + Toggle table + group TagInput)                                    | sveltekit-implementer | 90 min   |
| MGMT-UI-008     | `GroupsList` (ExpandableSection per group; create/grant/add-member)                                           | sveltekit-implementer | 75 min   |
| DESIGN-MGMT-001 | Wireframe-fidelity + visual-state annotations for all four tabs                                               | frontend-designer     | 60 min   |
| DESIGN-MGMT-002 | Pending-state visual contract (○ badge, count banner, commit affordance)                                      | frontend-designer     | 45 min   |
| DESIGN-MGMT-003 | Read-only state (disabled-control treatment + `administration:write` info banner)                             | frontend-designer     | 30 min   |
| DESIGN-MGMT-005 | Inherited-vs-own permission row distinction in `PrincipalEditor`                                              | frontend-designer     | 40 min   |

#### Dependencies

- Blocks: Sprint 06b
- Dependent on: Sprint 02 (admin-write), Sprint 05 (perm/group `but-api` fns). **MGMT-IPC-004 (SDK regen) is a hard predecessor of every `but-sdk`-importing UI task.**

#### PRD Coverage

- UC-MGMT-01, UC-MGMT-02, UC-MGMT-03, UC-MGMT-06 (pending-until-committed half)
- Criteria: T-MGMT-001..016, T-MGMT-027/028/033/034/035/036

#### Capability Coverage

- **CAP-AUTHZ-01** — every governed write goes through `but-api` → `but-authz` `authorize()`; the UI never provides a bypass (server-side enforcement; renderer `adminOnly` is UX only).

#### Next Sprint Tasks

Expanded by `/kb-sprint-tasks-plan` on 2026-06-19 (16/16 tasks fakeability-CLEAN · `proposed_by` tripwire 16/16 ·
avg rubric ≈112/115 · stable gapless AC-N/TC-N · 1 full red-hat cycle — fresh `rust-reviewer` + `tauri-reviewer`

- `sveltekit-reviewer` + `security-auditor`, all BLOCK; 7 CRITICAL + 14 MEDIUM + 12 LOW remediated by the retained
  domain writers and confirmed by cycle-2 deterministic re-validation). Detail files in
  [`tasks/sprint-06a-governance-ui-scaffold-principals-groups/`](./tasks/sprint-06a-governance-ui-scaffold-principals-groups/):

* `MGMT-IPC-001-but-api-governance-fns.md`
* `MGMT-IPC-002-json-error-remediation-hint.md`
* `MGMT-IPC-003-register-governance-commands.md`
* `MGMT-IPC-004-sdk-regen.md`
* `MGMT-IPC-005-pending-read-ipc-contract.md`
* `MGMT-UI-001-register-page-ct-harness.md`
* `MGMT-UI-002-settings-branch-isadmin.md`
* `MGMT-UI-003-governance-settings-pending-store.md`
* `MGMT-UI-005-governance-pending-banner.md`
* `MGMT-UI-006-principals-list.md`
* `MGMT-UI-007-principal-editor.md`
* `MGMT-UI-008-groups-list.md`
* `DESIGN-MGMT-001-four-tab-annotations.md`
* `DESIGN-MGMT-002-pending-state-contract.md`
* `DESIGN-MGMT-003-read-only-state.md`
* `DESIGN-MGMT-005-inherited-vs-own-rows.md`

> **Material re-grounding in expansion (red-hat):** MGMT-IPC-001's central premise was re-scoped — `#[but_api]`
> requires a `Context` param but the Sprint-05 `governance.rs` fns take `&gix::Repository`, so the task now owns a
> NEW thin Context-param **wrapper layer** (`*_cmd(ctx,…)`) delegating to the un-forked Sprint-05 `&repo` fns. The
> v1 **fleet-owner** identity (T-MGMT-042) is pinned as the R12 **unconditional-superuser** path (resolved from
> `UserService`, NOT a `permissions.toml` lookup; the "UI is never a bypass" invariant binds _agents_, not the human
> owner), wired + re-tested in MGMT-IPC-003 while MGMT-IPC-001 proves the gate under the `BUT_AGENT_HANDLE` identity.
> **MGMT-IPC-001 is BLOCKED-UNTIL Sprint-05 `governance.rs` merges** (and is added to the AUTHZ-007/008 honesty grep).
> **Upstream advisories** (reconcile via `/kb-sprint-plan --delta-replan`): `04-api-design.md:80` `allow-*` capability
> language is superseded by the live `core:default` convention; the `04-api-design.md` command table is missing the
> `governance_pending` row; `08-uc-mgmt.md:57` cites the wrong file for `ProjectSettingsPageId` (`uiState.svelte.ts`
> is correct). _(Tasks-table count corrected 17 → 16 — the materialized sprint has 16 tasks.)_

---

### Sprint 06b: Governance UI — Branch Gates + Rules + Safety

**Sequence:** 8
**Timeline:** Phase 4 — Governance management UI
**Status:** Done
**Proposed by:** sveltekit-planner + frontend-designer + rust-planner (MGMT backend); split authored by sveltekit-reviewer (red-hat)
**Milestone:** — (`sprint-06b`)

#### Human Testing Gate

**Gate:** An admin edits a branch gate on the Branch Gates tab (pending indicator appears), selects a principal on the Rules tab and confirms only that principal's rules are shown, then opens the page as a user lacking `administration:write` and observes all controls disabled with a read-only InfoMessage, and attempts a self-escalation and sees the denial InfoMessage without the toggle flipping.

**Test Steps:**

1. On Branch Gates, toggle Protected branch for a pattern; observe the pending indicator appears.
2. On Rules, select principal A and confirm only A's rules show; select B and confirm A's are absent.
3. Sign in as a user lacking `administration:write`; open the page; observe all controls disabled with a read-only banner.
4. As an admin, attempt to grant yourself `administration:write`; observe the denial banner and the toggle does not flip.
5. Navigate the four tabs by keyboard (Tab then Arrow keys); observe focus moves and tabs activate.
6. Trigger an IPC failure; observe a danger banner with a Retry button and the page stays read-only.

#### Tasks

| ID              | Title                                                                                                                          | Agent                 | Estimate |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------ | --------------------- | -------- |
| MGMT-BE-004     | `branch_gates_read`/`branch_gates_update` gate-config `but-api` producer (the gates.toml writer) + its Tauri command/SDK delta | rust-implementer      | 180 min  |
| MGMT-BE-003     | `principalId`-scoped rules query (backend for the Rules tab)                                                                   | rust-implementer      | 120 min  |
| MGMT-UI-004     | Wrap `GovernanceSettings` in the existing `shared/ErrorBoundary` (no new boundary component)                                   | sveltekit-implementer | 30 min   |
| MGMT-UI-009     | `BranchGatesList` (ExpandableSection per branch; required-group selector = defined groups)                                     | sveltekit-implementer | 75 min   |
| MGMT-UI-010     | Extend `RulesList` with optional `principalId` prop (backward compatible)                                                      | sveltekit-implementer | 45 min   |
| MGMT-UI-011     | Accessibility (aria + keyboard nav) + IPC-failure danger banner + Retry                                                        | sveltekit-implementer | 60 min   |
| MGMT-UI-012     | Build-gate tests: no direct config write, no `+page.server.ts`, SDK type-check, human-principal                                | sveltekit-implementer | 45 min   |
| DESIGN-MGMT-004 | Structured-denial banner + self-escalation no-flip contract                                                                    | frontend-designer     | 30 min   |
| DESIGN-MGMT-006 | Empty states for all four tabs                                                                                                 | frontend-designer     | 25 min   |
| DESIGN-MGMT-007 | Four-tab IA + aria + keyboard-nav contract                                                                                     | frontend-designer     | 35 min   |
| DESIGN-MGMT-008 | Error-boundary fallback + IPC-failure/retry pattern                                                                            | frontend-designer     | 30 min   |

#### Dependencies

- Blocks: None
- Dependent on: Sprint 06a (page scaffold + pending store + IPC base), Sprint 04 (gate engine for branch gates)

#### PRD Coverage

- UC-MGMT-04, UC-MGMT-05, UC-MGMT-06 (read-only + denial-no-flip), UC-MGMT-07 (error boundary + a11y + IPC retry)
- Criteria: T-MGMT-017..026, T-MGMT-029/030/031/037/038/039/040/041/042

#### Capability Coverage

- **CAP-AUTHZ-01** + **CAP-CONFIG-01** — `branch_gates_update` (MGMT-BE-004) authorizes `administration:write` at the target ref and writes inert-until-committed `gates.toml`; the gate-config writer is the previously-unowned producer (RUST-3 fix).

#### Next Sprint Tasks

Expanded by `/kb-sprint-tasks-plan` on 2026-06-19 (11/11 tasks fakeability-CLEAN · `proposed_by` tripwire 11/11 · avg rubric 114.3/115 · 1 full red-hat goal loop — fresh `rust-reviewer` + `sveltekit-reviewer` + `security-auditor`; 9 CRITICAL + 15 MEDIUM resolved by the retained writers and re-verified, 5 advisory recorded). Detail files in [`tasks/sprint-06b-governance-ui-branch-gates-rules-safety/`](./tasks/sprint-06b-governance-ui-branch-gates-rules-safety/):

- `MGMT-BE-004-branch-gates-config-writer.md`
- `MGMT-BE-003-principal-scoped-rules-query.md`
- `MGMT-UI-004-error-boundary-wrap.md`
- `MGMT-UI-009-branch-gates-list.md`
- `MGMT-UI-010-ruleslist-principalid.md`
- `MGMT-UI-011-accessibility-ipc-retry.md`
- `MGMT-UI-012-build-gate-tests.md`
- `DESIGN-MGMT-004-denial-no-flip-contract.md`
- `DESIGN-MGMT-006-empty-states.md`
- `DESIGN-MGMT-007-four-tab-a11y-contract.md`
- `DESIGN-MGMT-008-error-boundary-ipc-retry.md`

---

### Sprint 07: Local Agent PR — Governed-Review Parity (LPR)

**Sequence:** 9
**Timeline:** Phase 5 — Local Agent PR / Governed-Review Parity
**Status:** Done
**Proposed by:** rust-planner
**Milestone:** — (`sprint-07`)

#### Human Testing Gate

**Gate:** A maintainer runs the full local review loop by hand and observes the expected `but.sqlite` artifact after each `but review` step, with drive metadata never affecting the merge-gate decision.

**Test Steps:**

1. Run `but review request <branch>` as an agent principal; observe a `pending` assignment row.
2. Run `but review status <branch>`; observe the assignment, lifecycle, and `agent-authored` tag.
3. Run `but review assign <branch> --reviewer <author>`; observe the self-assignment is rejected.
4. Run `but review comment <branch> --file f.rs --line 12 --thread t1`; observe an unresolved comment row.
5. Run `but review resolve <branch> t1` as an unauthorized principal; observe the thread stays unresolved.
6. Run `but review approve <branch>` as the reviewer; observe an `approved` verdict at head.
7. Run the governed merge; observe it proceeds despite open drive metadata.
8. Forge drive rows with no approved verdict; observe the merge is blocked identically.

#### Tasks

| ID      | Title                                                                                                                                                       | Agent                   | Estimate |
| ------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------- | -------- |
| LPR-001 | `local_review_assignments` + `local_review_comments` + `local_review_meta` tables + 3 `SchemaVersion::Zero` migrations + 3 structs + Handle/HandleMut pairs | rust-implementer        | 180 min  |
| LPR-002 | `AssignmentState { Pending, Approved, ChangesRequested }` typed enum + boundary (de)serialization                                                           | rust-implementer        | 75 min   |
| LPR-003 | `request_review` / `assign_reviewer` `#[but_api(napi)]` + `but review request`/`assign` CLI; implement the real `changes_requested` write                   | rust-implementer        | 180 min  |
| LPR-004 | `post_comment`/`list_comments`/`resolve_thread` `#[but_api(napi)]` + `but review comment`/`comments`/`resolve` CLI                                          | rust-implementer        | 150 min  |
| LPR-005 | `review_status` derived PR lifecycle + agent-PR tag from declared `kind` in committed `permissions.toml`                                                    | rust-implementer        | 180 min  |
| LPR-006 | `Project.keep_reviews_local: DefaultTrue` + default-local wiring + remote-mirror gate                                                                       | rust-implementer        | 120 min  |
| LPR-007 | `but-rules` auto "review-requested" hook reusing the Sprint-06b engine                                                                                      | rust-implementer        | 150 min  |
| LPR-008 | Reconciler read-API: `review_status` serves the full drive state in one payload                                                                             | rust-implementer        | 120 min  |
| LPR-009 | Safe-seam invariant: build-gate honesty grep + forged-vs-empty + inverse integration tests                                                                  | rust-reviewer           | 180 min  |
| LPR-010 | TS SDK regen + N-API audit + happy-path CLI tests + honesty/anti-fakeability greps                                                                          | rust-reviewer           | 150 min  |
| LPR-011 | Reconciler usage-model doc + `but-*` skill contract                                                                                                         | rust-implementer / docs | 75 min   |

#### Dependencies

- Blocks: None
- Dependent on: Sprint 01b (the `approve_review` verdict write + the merge gate), Sprint 04 (merge strictness), Sprint 05 (`but perm`/`but group` CLI surface + persisted config), and the shipped `but-rules` engine from Sprint 06b for the auto-hook.

#### PRD Coverage

- UC-LPR-01..07
- Criteria: T-LPR-001..044 (+ the hand-driven full-local-loop human-gate T-LPR-029h)

#### Capability Coverage

- **CAP-AUTHZ-01** — the six new `#[but_api(napi)]` verbs authorize via `authorize_branch_action` at the `but-api` boundary with no new `Authority` variant; `assign_reviewer` enforces distinct-from-author and `resolve_thread` enforces resolver-identity.
- **CAP-CONFIG-01** — `keep_reviews_local` is a per-project operator preference under the R12 trusted-desktop model, persisted in the project store, defaulting local via `DefaultTrue`.

#### Next Sprint Tasks

Expanded by `/kb-sprint-tasks-plan`. Detail files in [`tasks/sprint-07-local-agent-pr/`](./tasks/sprint-07-local-agent-pr/).

---

### Sprint 08: STEER — Capability-Aware Denials

**Sequence:** 10
**Timeline:** Phase 5 — Capability-aware denials
**Status:** Done
**Proposed by:** rust-planner
**Milestone:** — (`sprint-08-steer-capability-aware-denials`)

#### Human Testing Gate

**Gate:** A denied principal receives a structured steering payload whose every listed action succeeds in its stated context and never reproduces the original denial.

**Test Steps:**

1. Commit `permissions.toml` + protected `gates.toml`.
2. Run a commit denial as `dev`; observe JSON carries `class`, `held_permissions`, `authorized_actions`, `do_not`.
3. Run a commit on protected `main` as a reviewer; observe menu lists review actions, never self-approve.
4. Follow a listed `authorized_actions` command; observe it returns exit 0.
5. Run a commit with `BUT_AGENT_HANDLE` unset; observe `operator_required`, empty menu.
6. Commit malformed `gates.toml`; observe `config.invalid`, `operator_required`, empty menu.
7. Run any actor-correctable denial; observe `but perm list` in the menu.
8. Parse a merge denial; observe `code`, `message`, `remediation_hint`, `unmet`, exit 1.

#### Tasks

| ID        | Title                                                                                                                                   | Agent                   | Estimate |
| --------- | --------------------------------------------------------------------------------------------------------------------------------------- | ----------------------- | -------- |
| STEER-001 | Steering fields on all denial carriers + `DenialClass`/`AuthorizedAction` types + derives + `to_envelope()` + `Authority` serialization | rust-implementer        | 210 min  |
| STEER-002 | `Route` enum + single-source `ROUTE_AUTHORITY_TABLE` in `but-authz`                                                                     | rust-implementer        | 270 min  |
| STEER-003 | Gate-state-aware `authorized_actions` derivation (intersection − failed predicate, intent map, self-approve exclusion)                  | rust-implementer        | 240 min  |
| STEER-004 | Wire payload + exhaustive `(code, principal-resolution) → class` mapping into all constructors/gates                                    | rust-implementer        | 210 min  |
| STEER-005 | Add the four fields to the hand-rolled CLI serializers; coordinate with Tauri `json::Error`                                             | rust-implementer        | 180 min  |
| STEER-006 | `but whoami` / `but can-i` self-scoped discovery                                                                                        | rust-implementer        | 210 min  |
| STEER-007 | Denial-steering telemetry event on the tracing path                                                                                     | rust-implementer        | 120 min  |
| STEER-008 | Ship the non-enforced agent-priming reference primer                                                                                    | rust-implementer / docs | 90 min   |
| STEER-009 | Extend `governed_loop` for gate-state-aware no-lying-menu                                                                               | rust-implementer        | 240 min  |
| STEER-010 | Net-new honesty build-gates: closed-catalog + table/affordance coverage                                                                 | rust-reviewer           | 120 min  |

#### Dependencies

- Blocks: None
- Dependent on: Sprint 02 (denial primitive + fail-closed), Sprint 04 (merge strictness + `unmet` requirement engine), Sprint 05 (`but perm list` + persisted governance config + the honesty grep). Coordinates with Sprint 06a `MGMT-IPC-002` for desktop-surface steering fields.

#### PRD Coverage

- UC-STEER-01..06
- Criteria: T-STEER-001..031

#### Capability Coverage

- **CAP-STEER-01 — capability-aware denial.** Producer: gate-state-aware `authorized_actions` derivation over the single-source `ROUTE_AUTHORITY_TABLE`, wired through the exhaustive `class` mapping and serialized at the CLI sites; no-lying-menu proven by the extended `governed_loop`; closed-catalog + single-source coverage proven by net-new honesty greps. Fail-closed preserved.

#### Next Sprint Tasks

Expanded by `/kb-sprint-tasks-plan`. Detail files in [`tasks/sprint-08-steer-capability-aware-denials/`](./tasks/sprint-08-steer-capability-aware-denials/).

---

### Sprint 09: Governance Remediation — LPR/MGMT Hardening

**Sequence:** 11
**Timeline:** Phase 6 — Remediation (post-audit hardening)
**Status:** Done — closed 2026-06-23 (16/16 tasks merged to master at 396cdabab3)
**Proposed by:** `rust-planner` + `sveltekit-planner` (parallel dispatch 2026-06-23, consolidated by orchestrator)
**Milestone:** — (`sprint-09`)

> **Origin.** This sprint is NOT PRD-derived. It was identified by an independent codebase investigation (6 parallel subagents: 5 code-tracing + 1 test-execution) on 2026-06-23 that found Sprint 07 (LPR) and Sprint 06b (MGMT) had 6 blocking gaps despite their "Done" status in this roadmap. The investigation report is in the chat log; provenance for each task below cites the audit finding (`audit-finding-*`).

#### Human Testing Gate

**Gate:** A maintainer observing the desktop Branch Gates panel after running the full local review loop (`but review request → comment → comments → resolve → approve`) sees open assignments plus unresolved threads update at each step, with the Keep Reviews Local toggle persisting across reload.

**Test Steps:**

1. Run `but review comment <branch> --file f --line N --thread t -m "…"` → comment recorded
2. Run `but review comments <branch>` → listed thread appears
3. Run `but review resolve <branch> --thread t` → thread marked resolved
4. Open Governance page → Branch Gates tab → toggle a protected-branch gate
5. Toggle Keep Reviews Local in settings → reload → observe value persists
6. Commit on a branch with a review-requested rule → observe auto-assignment
7. Open Local Review panel → observe approval status plus source branch

#### Tasks

| ID             | Title                                                                                                                       | Agent                 | Estimate |
| -------------- | --------------------------------------------------------------------------------------------------------------------------- | --------------------- | -------- |
| LPR-REM-001    | Replace `comment_review` stub with real `post_comment` call (add `--file/--line/--thread`)                                  | rust-implementer      | 180 min  |
| LPR-REM-002    | Add `but review comments` and `but review resolve` CLI verbs                                                                | rust-implementer      | 150 min  |
| LPR-REM-003    | Persist `keep_reviews_local` through `UpdateRequest` + `Storage::update()`                                                  | rust-implementer      | 120 min  |
| LPR-REM-004    | Wire `process_commit_rules` into `but commit` production path                                                               | rust-implementer      | 180 min  |
| LPR-REM-005    | Add `open_assignments`/`unresolved_threads` to Tauri `review_status` payload (fixes 2 failing tauri tests)                  | rust-implementer      | 90 min   |
| LPR-REM-006    | Complete LPR-009 safe-seam invariant (bidirectional equivalence + 3-step capstone; move grep to `invariant_build_gates.rs`) | rust-reviewer         | 180 min  |
| LPR-REM-007    | Surface `kind` in `GovernancePrincipalListEntry` (Rust half)                                                                | rust-implementer      | 60 min   |
| LPR-REM-007-UI | Mount `LocalReviewView.svelte`; consume `kind` for agent/human badge (UI half)                                              | sveltekit-implementer | 60 min   |
| LPR-REM-008    | Regenerate stale `but` CLI snapshots for STEER enrichment (verify correctness, not just accept drift)                       | rust-implementer      | 30 min   |
| LPR-REM-009    | Remove/fix untracked `list_workspace_rules_scoped.rs` (9 compile errors)                                                    | rust-implementer      | 30 min   |
| MGMT-REM-001   | Wire `BranchGatesList.svelte` into `GovernanceSettings.svelte` (component exists, was orphaned)                             | sveltekit-implementer | 30 min   |
| MGMT-REM-002   | Remove illegal test-import stub `PrincipalEditorInherited*.spec.ts` (CT suite crash)                                        | sveltekit-implementer | 30 min   |
| MGMT-REM-003   | Add `but group delete` CLI verb (replace `group_no_delete_cli_verb_surface` test)                                           | rust-implementer      | 90 min   |
| MGMT-REM-004   | Strengthen `BuildGates.spec.ts:204` lint-gate assertion to require exit 0                                                   | sveltekit-implementer | 30 min   |
| MGMT-REM-005   | Align root `test:ct` command (or docs) so desktop CT is reachable via documented command                                    | sveltekit-implementer | 30 min   |
| STEER-REM-001  | Add STEER fields to `branch/apply.rs` `commit_gate_cli_error` serializer (4th commit-gate site)                             | rust-implementer      | 30 min   |

#### Dependencies

- **Blocks:** None (terminal remediation sprint).
- **Dependent on:** Sprint 00 (walking skeleton), Sprint 04 (merge strictness), Sprint 05 (CLI surface), Sprint 06b (UI), Sprint 07 (LPR — claims Done but gaps exist), Sprint 08 (STEER).
- **Intra-sprint edges:**
  - `LPR-REM-001` → `LPR-REM-002` (CLI comments/resolve verbs need comment verb real)
  - `LPR-REM-001` + `LPR-REM-002` + `LPR-REM-009` → `LPR-REM-005` (status payload needs real comment data + tauri compile fixed)
  - `LPR-REM-007` → `LPR-REM-007-UI` (UI half consumes Rust kind field)
  - `LPR-REM-001` + `LPR-REM-002` + `MGMT-REM-003` + `STEER-REM-001` → `LPR-REM-008` (regenerate snapshots AFTER all CLI changes land)

#### PRD Coverage

- **NOT PRD-derived.** Coverage is against the audit findings from the 2026-06-23 codebase investigation:
  - `audit-finding-LPR-REM-001` — `comment_review` stub at `crates/but-api/src/legacy/forge.rs:841-852`
  - `audit-finding-LPR-REM-002` — missing `Comments`/`Resolve` CLI variants at `crates/but/src/args/forge.rs:28-184`
  - `audit-finding-LPR-REM-003` — `keep_reviews_local` silently dropped at `crates/gitbutler-project/src/storage.rs:86-92`
  - `audit-finding-LPR-REM-004` — `process_commit_rules` never called from `but commit` (zero matches in `crates/but/`)
  - `audit-finding-LPR-REM-005` — `review_status` payload missing `open_assignments`/`unresolved_threads` (2/4 tauri tests fail)
  - `audit-finding-LPR-REM-006` — LPR-009 safe-seam proof incomplete (grep in wrong file, bidirectional equivalence + capstone missing)
  - `audit-finding-LPR-REM-007` — `GovernancePrincipalListEntry` missing `kind` field (`crates/but-api/src/legacy/governance.rs:216-227`)
  - `audit-finding-LPR-REM-008` — 3 stale snapshots: `help::test_print_grouped`, `governed_merge_cli::merge_denial_is_structured_*`, `merge_gate::merge_gate_auto_merge_denial_is_structured`
  - `audit-finding-LPR-REM-009` — untracked `crates/gitbutler-tauri/tests/list_workspace_rules_scoped.rs` (9 compile errors)
  - `audit-finding-MGMT-REM-001` — `BranchGatesList.svelte` orphaned; stub tab body at `apps/desktop/src/components/governance/GovernanceSettings.svelte:229-241`
  - `audit-finding-MGMT-REM-002` — illegal test-import at `apps/desktop/tests/governance/PrincipalEditorInherited*.spec.ts` (CT suite crash)
  - `audit-finding-MGMT-REM-003` — `but group delete` CLI verb absent; `crates/but/tests/but/command/group.rs:155-167` asserts non-implementation
  - `audit-finding-MGMT-REM-004` — `BuildGates.spec.ts:204` asserts output contains "lint" string only (no exit 0 check)
  - `audit-finding-MGMT-REM-005` — root `package.json` `test:ct` runs only `@gitbutler/ui`, not desktop
  - `audit-finding-STEER-REM-001` — `crates/but/src/command/branch/apply.rs:71-85` drops STEER fields (4th commit-gate site missed)
- **Closes Sprint 07 / Sprint 06b gaps** so their "Done" claims become truthful.

#### Capability Coverage

- **CAP-AUTHZ-01** — restored by LPR-REM-001/002 (real comment/resolve verbs gate via `CommentsWrite`/`ReviewsWrite`); STEER-REM-001 (uniform denial shape on `branch apply`).
- **CAP-CONFIG-01** — restored by LPR-REM-003 (`keep_reviews_local` write path closes the config-persist gap) and MGMT-REM-001 (Branch Gates UI actually mutates `gates.toml` via the orphaned list).
- **CAP-LPR-08** (new informal ID) — `review_status` reconciler payload carries the full drive state (LPR-REM-005); auto-hook actually fires (LPR-REM-004).
- **CAP-STEER-01** — uniform-shape invariant extended to all 4 commit-gate CLI sites (STEER-REM-001).

#### Next Sprint Tasks

Expanded by `/kb-sprint-tasks-plan` on 2026-06-23T13:30:00Z. Detail files in [`tasks/sprint-09-governance-remediation-lpr-mgmt-hardening/`](./tasks/sprint-09-governance-remediation-lpr-mgmt-hardening/):

- `LPR-REM-001-replace-comment-review-stub-with-real-post-comment-call.md`
- `LPR-REM-002-add-but-review-comments-and-but-review-resolve-cli-verbs.md`
- `LPR-REM-003-persist-keep-reviews-local-through-updaterequest.md`
- `LPR-REM-004-wire-process-commit-rules-into-but-commit-production-path.md`
- `LPR-REM-005-add-open-assignments-unresolved-threads-to-tauri-review-status-payload.md`
- `LPR-REM-006-complete-lpr-009-safe-seam-invariant.md`
- `LPR-REM-007-surface-kind-in-governanceprincipallistentry-rust-half.md`
- `LPR-REM-007-UI-mount-localreviewview-svelte-consume-kind-ui-half.md`
- `LPR-REM-008-regenerate-stale-but-cli-snapshots-for-steer-enrichment.md`
- `LPR-REM-009-remove-fix-untracked-list-workspace-rules-scoped-rs.md`
- `MGMT-REM-001-wire-branchgateslist-svelte-into-governancesettings-svelte.md`
- `MGMT-REM-002-remove-illegal-test-import-stub-from-governance-ct-suite.md`
- `MGMT-REM-003-add-but-group-delete-cli-verb.md`
- `MGMT-REM-004-strengthen-buildgates-spec-ts-lint-gate-assertion.md`
- `MGMT-REM-005-align-root-test-ct-command-with-desktop-ct-documentation.md`
- `STEER-REM-001-add-steer-fields-to-branch-apply-rs-commit-gate-cli-serializer.md`

#### Verification Command (run after Sprint 09 lands)

```bash
cargo test -p but-authz && \
cargo test -p but-api && \
cargo test -p but --features but-2 && \
cargo test -p but-rules review_requested_hook && \
cargo test -p gitbutler-tauri --test lpr_review_reads && \
pnpm -F @gitbutler/desktop test:ct:desktop tests/governance/ && \
pnpm -F @gitbutler/desktop test
```

All suites must pass clean (no crashes, no drift, no skipped tests).

---

## Red-Hat Review Summary

|                                   | Value                                                                                                                                                                                                                                                                                                   |
| --------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Review cycles                     | 1 full cycle (3 fresh reviewers: `rust-reviewer`, `sveltekit-reviewer`, `security-auditor`) + writer remediation                                                                                                                                                                                        |
| Blocking findings resolved        | 8 CRITICAL + ~10 MEDIUM (all by the original writers)                                                                                                                                                                                                                                                   |
| Upstream (locked-PRD) escalations | 0 — every gap was a roadmap/grounding gap, fixable in re-planning                                                                                                                                                                                                                                       |
| Convergence                       | Deterministic [5.4] re-validation of the remediated structure: 0 banned-pattern violations · all step counts 3–8 · acyclic graph · 17/17 original UCs covered (30/30 after v1.5.0 STEER+LPR fold-in) · both capability chains owned (incl. the new DryRun + commit-gate-target-ref + T-LOOP-013 proofs) |
| Residual                          | A second fresh-panel re-review was bounded to deterministic re-validation for cost (planning artifact). Advisory follow-ups carried into task briefs: governance sidebar icon-name verification; Svelte 5 `svelte:boundary` mechanism choice.                                                           |

## Next Steps

**10 sprints Done, 1 Planned (Sprint 09 remediation).** Sprints 00–08 shipped through red-hat review cycles with findings closed, but an independent audit on 2026-06-23 surfaced blocking gaps in Sprint 07 (LPR) and Sprint 06b (MGMT) that require Sprint 09 to close before the initiative can truthfully claim delivered.

1. **✅ Catch-up complete** — Sprint 00 passed (14/14 flows green); Sprints 01a + 01b are VERIFIED.
2. **✅ Sprints 02–08 shipped** through red-hat review cycles with findings closed. ROADMAP statuses reconciled to Done on 2026-06-23.
3. **🔲 Sprint 09 (remediation) Planned** — 16 tasks covering 6 blocking audit findings. Run `/kb-sprint-tasks-plan .spec/prds/governance/ROADMAP.md` to expand, then `/kb-run-sprint sprint-09-governance-remediation-lpr-mgmt-hardening` to execute. Until Sprint 09 lands, the Sprint 07 + Sprint 06b "Done" claims are overstated.

**Optional follow-ups** (not blocking — blocked instead on Sprint 09 landing first):

4. **Backfill the full flow registry** for remaining UCs (UC-GRPS, UC-MGMT, UC-LPR, UC-STEER) if formal flow-registry conformance is desired for future sprints:

   ```
   /kb-prd-plan .spec/prds/governance --update
   ```

5. **Deferred hardening** — the PRD names these as follow-ups, not gaps: HMAC → Ed25519-signed review artifacts (R6/R18 closure), full multi-clause gate, auto-run validation, break-glass override, steel-trap transport boundary, `but governance init` onboarding.
