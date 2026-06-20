---
sprint: 04
sequence: 5
timeline: Phase 2 — Hardening
status: In Progress
proposed_by: rust-planner
milestone: sprint-04
prd: ../../README.md
roadmap: ../../ROADMAP.md
generated_by: kb-sprint-tasks-plan
---

# Sprint 04: GATES Deepening

**Sequence:** 5
**Timeline:** Phase 2 — Hardening
**Status:** In Progress
**Proposed by:** rust-planner
**Milestone:** — (`sprint-04`)

## Overview

The third hardening sprint. Sprint 01a/01b built the two gates and proved the positive governed loop;
Sprint 02 made the merge/forge negative space fail-closed; Sprint 03 made grouping safe. Sprint 04
**deepens both gates** along the two strictness axes the walking skeleton deliberately deferred:

1. **Merge-gate strictness — per-required-group evaluation.** Sprint 01b shipped the two-group
   requirement *plumbing* (`T-LOOP-007` parse + `T-LOOP-010` both-present-proceeds) and the
   single-required-group gate. Sprint 04 proves the **only-one-blocked** matrix as standalone test
   cases — an approval from *each* required group is needed at the current head, so an AI-only
   (`code-reviewers`) approval blocks (the human hasn't owned it) and a human-only (`maintainers`)
   approval blocks (the AI code-level pass is also required), order-independent. This is the
   "human-at-feature + AI-at-code as pure config" model expressed entirely in `.gitbutler/gates.toml`
   + group membership, with **no enforcement code that distinguishes a human from an AI**
   (the `T-LOOP-011` grep, deferred from 01b, lands here).

2. **Commit-gate coverage — mechanism-agnostic.** Sprint 01a sited the commit gate at the `but-api`
   `_with_authz` seam + CLI commit path. Sprint 04 makes the gate **actually** mechanism-agnostic:
   the ref-advancing entry points that bypass the plain-commit path — `branch::apply`,
   `integrate_branch_with_steps`, and the opt-in worktree commit/integrate path — are each gated at
   the same `contents:write` + branch-protection decision, so no branching mechanism is an ungated
   path onto a protected branch (`T-GATES-016/017`).

The third task closes the **standalone target-ref-only proofs** the deepened gates demand
(`T-GATES-019`): a working-tree or feature-head `gates.toml` edit cannot weaken the gate that judges
it, proven as a dedicated test rather than as a side-effect of another case.

> **Ownership de-confliction (carried from Sprint 02 AUTHZ-004).** The **merge-path** fail-closed
> classification — deterministic `config.invalid` vs `perm.denied` vs `gate.review_required` ordering,
> unknown/no-handle deny, and the **undefined-`require_approval_from_group` hard-deny** — already
> LANDED in Sprint 02 (`AUTHZ-004`, `crates/but-api/src/legacy/merge_gate.rs`). Sprint 04 GATES-008 is
> the **deepening / standalone-proof** of the *target-ref-only read* property and the commit-gate
> fail-closed surface, **not** a competing owner of the merge-path undefined-group check. The expansion
> must re-ground GATES-008 honestly against what AUTHZ-004 already owns — no duplicate implementation.

> **Review-store forgeability (R6, High — accepted-leak honesty).** `local_review_verdicts` is **not**
> integrity-protected. The merge-gate strictness tests exercise the **governed `but review`
> submission path only** — never assert the forgeable direct-DB-write path is blocked, nor that raw-git
> is blocked (both encode false guarantees). Deferred hardening: HMAC/Ed25519 review integrity.

## Human Testing Gate

**Gate:** Gate passes when BOTH: [MERGE-STRICTNESS] a merge with a stale, self, or single-group-only
approval is blocked and lands only with a distinct approval from each required group at the current
head; AND [COMMIT-COVERAGE] a protected-branch commit and `worktree_integrate` targeting
protected `main` are rejected `branch.protected`, while `branch::apply` and
`apply_branch_integration` by a contents:read principal are rejected `perm.denied` through the same
commit gate, and a working-tree `gates.toml` edit cannot weaken either gate.

### Test Steps

1. Approve a PR at head H1, advance to H2, run merge → blocked, `gate.review_required` with `approval_stale_at_head`.
2. Have the author approve their own change (distinct required), run merge → blocked, requirement unmet.
3. Supply only a `code-reviewers` approval, run merge → blocked (maintainers required); only `maintainers` → blocked.
4. Supply a distinct approval from each required group at head, run merge → exit 0, proceeds.
5. Commit on `but worktree new` to a feature branch, then to protected `main` → feature accepted, `main` rejected `branch.protected`.
6. Run `but worktree integrate` (the `worktree_integrate` seam) targeting protected `main` → rejected `branch.protected`; run `but branch apply` and `but branch integrate` on a feature branch as a contents:read principal → rejected `perm.denied`.
7. Edit `gates.toml` on the feature head to drop the merge requirement, run merge → still judged by the target-ref requirement.

## Tasks

| ID | Title | Agent | Estimate |
|----|-------|-------|----------|
| GATES-006 | Per-required-group approval evaluation (two-group AI + human model) | rust-implementer | 150 min |
| GATES-007 | Mechanism-agnostic commit gate — **actually** gate `branch::apply`, `integrate_branch_with_steps`, worktree-integrate | rust-implementer | 270 min |
| GATES-008 | Standalone target-ref-only read proof for the merge gate — feature-head requirement-drop ignored (deepening; AUTHZ-004 owns merge-path fail-closed) | rust-implementer | 120 min |
| GATES-REM-001 | Close CLI `but branch apply` bypass through governed public seam | rust-implementer | 120 min |
| GATES-REM-002 | Fix sprint human testing gate prose for commit-coverage reality | rust-implementer | 30 min |
| GATES-REM-003 | Define required-group overlap semantics for two-tier human+AI model | rust-implementer | 180 min |
| GATES-REM-004 | Add governed missing-target fail-closed proof for no-target apply/integrate | rust-implementer | 120 min |
| GATES-REM-005 | Replace compile-only structural TCs with discriminating checked scripts | rust-implementer | 90 min |
| GATES-REM-006 | Add deterministic source-contract test for gate-before-guard placement | rust-implementer | 120 min |
| GATES-REM-007 | Tighten GATES-008 production-diff gate to zero-diff baseline range | rust-implementer | 60 min |

## Dependencies

- **Blocks:** Sprint 05, Sprint 06b
- **Dependent on:** Sprint 01b, Sprint 03

## PRD Coverage

- **Use cases:** UC-GATES-01 (mechanism-agnostic coverage), UC-GATES-02, UC-LOOP-02
- **Criteria:** T-GATES-012/013/016/017/019, T-LOOP-008/009/010/011/012
  - *T-GATES-018 (merge-gate fail-closed on malformed/undefined-group config) is owned by **AUTHZ-004 (Sprint 02)**, where it already landed and its tests pass; GATES-008 consumes it and re-proves the residual target-ref-only read (T-GATES-019) on the merge path — it is not re-listed as a Sprint-04 deliverable.*
  - *T-LOOP-011 (no human-vs-AI enforcement branch) was deferred from Sprint 01b and lands here as the GATES-006 AC-3 build-gate.*
  - *T-GATES-016/017 are re-grounded against live code: `branch::apply` bails on the target and `integrate_branch_with_steps` writes a feature branch (neither advances a protected trunk), so GATES-007 proves mechanism parity as **contents:write/perm.denied on apply + integrate** plus **`worktree_integrate` advancing a protected `target` → `branch.protected`** — not "apply/integrate advancing main → branch.protected".*

## Capability Coverage

- **CAP-AUTHZ-01** — per-group approval evaluation (GATES-006).
- **CAP-CONFIG-01** — commit gate covers every ref-advancing entry point (GATES-007); both gates read target-ref-only (GATES-007/008).

## Task Detail Files

Generated by `/kb-sprint-tasks-plan` on 2026-06-19 — 3/3 tasks fakeability-CLEAN (`validate_scenario.py`, 0 violations) · `proposed_by` tripwire 3/3 · avg rubric 115/115 · 1 full red-hat goal loop (3 cycles: fresh `rust-reviewer` + `security-auditor`). Cycle 1 surfaced 5 CRITICAL + 7 MEDIUM (incl. a fabricated `governance_present`/`has_governance_marker` grounding inversion and the incoherent commit-coverage framing); all remediated by the retained writer. Cycle 2 confirmed those resolved and caught the remediation-introduced `config_only(feature-branch)` vacuous-gate hole (S9) + lock-ordering placement (R2b) + loose parity grep (S10); cycle 3 confirmed those resolved and caught the `target_ref_or_err()?`-propagation no-target regression (S9b) — all fixed and re-verified. Convergence trend 5C/7M → 1C/2M → 0C/1M → 0 blocking.

- `GATES-006-per-required-group-approval.md`
- `GATES-007-mechanism-agnostic-commit-gate.md`
- `GATES-008-merge-gate-failclosed-target-ref-only.md`
- `GATES-REM-001-close-cli-branch-apply-bypass.md`
- `GATES-REM-002-fix-sprint-human-testing-gate-prose.md`
- `GATES-REM-003-define-required-group-overlap-semantics.md`
- `GATES-REM-004-add-governed-missing-target-fail-closed-proof.md`
- `GATES-REM-005-replace-compile-only-structural-tcs-with-checked-scripts.md`
- `GATES-REM-006-add-deterministic-gate-placement-source-contract.md`
- `GATES-REM-007-tighten-gates-008-production-diff-gate.md`
