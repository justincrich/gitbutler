---
sprint: 01b
sequence: 2
timeline: Phase 1 — Walking skeleton (the T-LOOP-006 canary)
status: Done
proposed_by: rust-planner
milestone: sprint-01b
prd: ../../README.md
roadmap: ../../ROADMAP.md
generated_by: kb-sprint-tasks-plan
---

# Sprint 01b: Governed Loop Reference Flow

**Sequence:** 2
**Timeline:** Phase 1 — Walking skeleton (the T-LOOP-006 canary)
**Status:** Done — closed out 2026-06-19
**Proposed by:** rust-planner
**Milestone:** — (`sprint-01b`)

> **Closeout (2026-06-19).** All 5 tasks merged and independently re-verified against real
> services (real `but` CLI + real git + real `but-db`): the headline **T-LOOP-006 canary is green**
> — `cargo test -p but governed_loop` → 5/5 (`reference_flow_full_loop`, `remediation_traversable`,
> `dryrun_no_bypass`, `auto_merge_denied`, `unset_handle_failclosed`). Supporting gates green —
> `cargo test -p but-api -p but-db -p but-authz` → 193 passed / 0 failed (incl. `merge_gate.rs` 6,
> `forge_guard.rs` 2, `commit_gate.rs` 6, but-db review-store suite 126). No stubs in any gate path.
> Deferrals (per-group strictness T-LOOP-008/009, T-GATES-012; mechanism-agnostic commit
> T-GATES-016/017; T-LOOP-011 grep) remain scoped to **Sprint 04**; the forge-network *landing* of a
> merge stays out of local scope per the documented gate-boundary re-scope. **Unblocks Sprints 02, 03, 04.**

## Overview

The second half of the walking skeleton: it completes the **proven-reference-flow canary
(T-LOOP-006)** that the PRD mandates must go green before the deep build. On top of Sprint 01a's
`but-authz` primitive + commit gate, this sprint adds the **merge gate** (authorize `merge` + a
configurable, target-ref-pinned review requirement at head), the **review record** (a new `but-db`
`local_review_verdicts` table), the **authz guards on the forge boundary** (the governed
`but pr` / `but review` actions), and the **stale-/self-approval** dismissal that makes the review
requirement sound.

The headline deliverable is a single integration test that runs the full implement→review→merge
loop with three distinct principals — an implementer (`contents:write`, no `merge`), a reviewer
(`reviews:write`, no `contents:write`), and a maintainer (`merge`) — and proves the loop's role
separation **emerges from the functional permission set alone** (no role-name in any enforcement
path). It also proves the **irrigation half** (T-LOOP-013): a denied implementer that follows its
`remediation_hint` traverses the governed path. Every gate proof draws from
[`11-e2e-testing-criteria.md`](../../11-e2e-testing-criteria.md).

> **Review-store forgeability (R6, High — accepted-leak honesty).** `local_review_verdicts` is
> **not** integrity-protected — a direct DB write can forge an approving verdict. The merge gate is
> sound only for reviews submitted through the governed `but review` action; the LOOP demo assumes
> honest review submission. The tests exercise the governed review-submission path **only** — never
> assert the forgeable direct-DB-write path is blocked, nor that raw-git is blocked (both encode
> false guarantees). Deferred hardening: HMAC/Ed25519 review integrity.

## Human Testing Gate

**Gate:** Running the reference flow with three principals, the implementer's merge AND auto-merge
are denied, the maintainer's merge succeeds only after a distinct reviewer approval at head, and a
denied implementer that follows its `remediation_hint` lands through a reviewed merge.

### Test Steps

1. Run `but` merge as implementer (`reviews:write`, no `merge`) → denied, exit 1, `perm.denied` names merge.
2. Enable auto-merge (`but review --auto-merge`) as that implementer → denied, exit 1, `perm.denied`.
3. Run commit as reviewer → denied; submit review as reviewer → exit 0, recorded at head.
4. Run merge as maintainer with zero distinct approvals → denied, exit 1, `gate.review_required`.
5. Run merge as maintainer after a distinct reviewer approval at head → exit 0, merge proceeds.
6. Advance head, rerun merge → denied, `gate.review_required` with `approval_stale_at_head`.
7. Run a DryRun merge as the implementer → still denied `perm.denied`, nothing persisted.
8. Follow the denied implementer's `remediation_hint` (feature branch → review → merge) → lands successfully.

## Tasks

| ID | Title | Agent | Estimate |
|----|-------|-------|----------|
| GATES-002 | Local review record: `but-db` `local_review_verdicts` table (head-pinned) | rust-implementer | 150 min |
| GATES-003 | Merge gate covering **both** `merge_review` AND `set_review_auto_merge` | rust-implementer | 270 min |
| GATES-004 | Submit-review / open-PR / comment authz guards on the forge boundary | rust-implementer | 150 min |
| GATES-005 | Stale-approval-@head dismissal + self-approval exclusion | rust-implementer | 150 min |
| LOOP-001 | Reference-flow test (T-LOOP-006) + traversable proof (T-LOOP-013) + DryRun-no-bypass | rust-implementer | 240 min |

## Dependencies

- **Blocks:** Sprint 02, Sprint 03, Sprint 04
- **Dependent on:** Sprint 01a

## PRD Coverage

- **Use cases:** UC-LOOP-01, UC-LOOP-02, UC-GATES-02 (review record)
- **Criteria:** T-LOOP-006/001/002/003/004/007/010/**013**, T-GATES-008/009/010/011/014/015

## Capability Coverage

- **CAP-AUTHZ-01** — merge/auto-merge gate (GATES-003), forge guards (GATES-004); DryRun-no-bypass proven (LOOP-001).
- **CAP-CONFIG-01** — merge gate reads requirement at the target ref (GATES-003).

## Coverage Notes

- **Deferred (honest scope):** the dedicated **per-required-group strictness** matrix —
  `T-LOOP-008`/`T-LOOP-009` (only-AI-approval blocks / only-human-approval blocks) and `T-GATES-012`
  (an approval required from *each* required group as standalone test cases) — and the
  mechanism-agnostic commit coverage (`T-GATES-016/017`) are **deferred to Sprint 04** (GATES-006/007).
  This sprint establishes the two-group requirement *plumbing* (`T-LOOP-007` parse +
  `T-LOOP-010` both-present-proceeds) and the single-required-group merge gate; Sprint 04 proves the
  only-one-blocked strictness and the dedicated two-group AI/human matrix.
- **Accepted-leak (R6, by design):** the merge gate trusts a forgeable `local_review_verdicts` store.
  No test asserts the direct-DB-write or raw-git bypass is blocked — those are documented
  accepted-leaks, not in-scope guarantees.
- **Gate-boundary re-scope (red-hat cycle + user decision):** `merge_review`/`publish_review` are
  forge-bound (they call `derive_forge_repo_info`, which errors on a bare local repo with no remote)
  and there is **no `but pr merge` CLI verb** — so the *positive* governed-merge/PR-open paths
  (GATES-003 AC-1/AC-3, GATES-005 AC-2/AC-3, LOOP-001 AC-1 step 5 + AC-2) prove the **gate DECISION**
  (permit, no Denial) on the real seam + that execution reaches the forge call; the forge-network
  **completion** ("change lands on the remote trunk") is proven structurally / deferred to a
  forge-backed fixture. DENIAL paths, the commit gate, and the local `but review approve`
  verdict-write are all fully locally provable (no mocks).
- **Upstream advisory (escalated, not blocking):** the locked PRD's T-LOOP-006 "merge succeeds" /
  T-LOOP-013 "change LANDS" / T-LOOP-004 wording assumes a *locally-completable* governed merge that
  the current `but-api`/`but-forge` surface does not provide on a no-remote repo. Recommend
  reconciling via `/kb-sprint-plan --delta-replan` (the local canary proves the GATE's permit/deny +
  traversability, not the forge landing). Also: ROADMAP/SPRINT **step 1** prose ("implementer
  (`reviews:write`, no `merge`)") carries a stale role token — the implementer is
  `contents:write`+`pull_requests:write` everywhere else; fix in the same delta-replan.
- **T-LOOP-011 deferral:** the human-vs-AI no-enforcement-branch grep (UC-LOOP-02 AC-5) is
  **deferred to Sprint 04** alongside the per-group strictness matrix (T-LOOP-008/009, T-GATES-012).
  The functional-not-role grep (T-LOOP-005) IS asserted in GATES-003/004/005 here.

## Red-Hat Review Summary

Expanded by `/kb-sprint-tasks-plan` on 2026-06-18 — 1 full red-hat cycle (fresh `rust-reviewer` +
`security-auditor`, no authoring context). Findings: **1 CRITICAL** (forge-bound positive-merge path
— resolved by the user's *gate-boundary re-scope* decision + the upstream advisory above) and
**5 MEDIUM** (GATES-005 governed-path seeding; LOOP-001 e2e fail-closed `AC-5` added; GATES-003
both-entry-points grep tightened to function-scoped; GATES-003↔GATES-005 `depends_on` 2-cycle broken
to one-directional; GATES-003 `but-db` wording; GATES-004 `but-clap` cli-docs note) — all remediated.
The security-auditor confirmed **all 9 prior red-hat CRITICALs closed** (auto-merge bypass,
fail-open config, DryRun bypass, self/stale soundness, target-ref pin, R6 honesty, no-agent-claim
identity, no role-name, no false tamper-proof guarantee). Deterministic re-validation: 5/5 tasks
fakeability-CLEAN (`validate_scenario.py`, 0 CRITICAL/HIGH); `proposed_by` tripwire 5/5.

## Task Detail Files

Generated by /kb-sprint-tasks-plan on 2026-06-18.

- GATES-002-local-review-record.md
- GATES-003-merge-gate.md
- GATES-004-forge-authz-guards.md
- GATES-005-stale-self-approval.md
- LOOP-001-reference-flow-test.md
