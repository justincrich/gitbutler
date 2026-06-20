---
sprint: 01a
sequence: 1
timeline: Phase 1 — Walking skeleton
status: Done
proposed_by: rust-planner
milestone: sprint-01a
prd: ../../README.md
roadmap: ../../ROADMAP.md
generated_by: kb-sprint-tasks-plan
---

# Sprint 01a: AUTHZ Primitive + Commit Gate

**Sequence:** 1
**Timeline:** Phase 1 — Walking skeleton
**Status:** Done — closed out 2026-06-19
**Proposed by:** rust-planner
**Milestone:** — (`sprint-01a`)

> **Closeout (2026-06-19).** Red-hat review found the commit gate's absent-config
> behavior contradicted the written spec and a substituted test was hiding it.
> Per the **opt-in-by-presence** product decision, RF-010/GATES-001 were amended to
> the three-state model (absent → ungoverned/allowed; partial or malformed →
> `config.invalid`; both → enforce), and the tests were made honest: distinct
> `commit_gate_absent_config_is_ungoverned` (allow) vs `commit_gate_malformed_partial_and_dryrun`
> (`config.invalid`), an anti-fakeability fixture-shape harness, a strengthened
> DryRun assertion, a CLI opt-in-activation test, the obfuscated path helper
> replaced by `but_authz::governance_present`, and the AUTHZ-007 invariant grep
> extended to the merge-gate/forge enforcement files. Independently re-reviewed
> (`rust-reviewer`: **APPROVED**, 8/8 checks). Green: `cargo test -p but-api commit_gate`
> 7/7, `cargo test -p but --features but-2 commit_gate` 5/5, `cargo test -p but-authz`
> all incl. `invariant_build_gates`, `cargo test -p but-api -p but-db -p but-authz`
> 201/0, `cargo test -p but governed_loop` 5/5 (01b not regressed); fmt + clippy clean.
> Code commit `566a36b9c8` (also pinned on branch `wip/governance-commit-gate-optin`).
> Deferred (Sprint 04): the stronger "governance-enabled signal" anti-config-deletion
> guarantee. **Unblocks Sprint 01b** (already Done).

## Overview

The first thin vertical slice of the Functional-Permission Agent Governance POC: a new
`but-authz` crate (the functional `Authority`/`AuthoritySet`/`Principal`/`Group`/`Denial` model,
no roles in enforcement), a ref-pinned governance-config loader that reads committed
`.gitbutler/permissions.toml` + `.gitbutler/gates.toml` **only at the target ref** (a working-tree
edit can never weaken a gate), a fail-closed `authorize()` keyed off `BUT_AGENT_HANDLE`, and the
**commit gate** at the `commit_engine::create_commit` narrow-waist that runs even under DryRun.

Governance is **opt-in by presence** of `.gitbutler/*.toml` at the target ref: a repo whose trunk
never committed governance config is ungoverned and commits freely; once governance is committed,
the gate enforces. An incomplete config (exactly one of the two files) or a malformed file fails
closed `config.invalid`.

This sprint is the foundation the T-LOOP-006 reference-flow canary (Sprint 01b) stands on. It must
produce a single observable commit allow/deny decision through real `but-authz` + real git, with no
mocks. Every gate proof draws from [`11-e2e-testing-criteria.md`](../../11-e2e-testing-criteria.md).

## Human Testing Gate

**Gate:** Running a commit as a read-only principal is denied `perm.denied` while a `contents:write`
principal's commit lands, and protection is read only from the target-ref blob so a working-tree
`gates.toml` edit cannot unprotect the branch.

### Pre-steps

- **`but commit` runs the experimental `commit2` path behind the `but-2` cargo feature.** All
  `but commit` CLI steps below must be built/tested with `--features but-2` (e.g.
  `cargo test -p but --features but-2 commit_gate`); without it the CLI commit-gate tests are not
  compiled.

### Test Steps

1. Seed committed `.gitbutler/permissions.toml` (`ro` contents:read; `dev` contents:write) + `gates.toml` (`main` protected).
2. Run commit on a feature branch as `dev` → exit 0, ref advances.
3. Run commit on a feature branch as `ro` → denied, exit 1, `perm.denied` names contents:write.
4. Run direct commit to protected `main` as `dev` → denied, exit 1, `branch.protected`.
5. Run commit with `BUT_AGENT_HANDLE` unset → rejected, exit 1, no anonymous action.
6. Edit working-tree `gates.toml` to unprotect `main`, commit directly to `main` as `dev` → still `branch.protected`.
7. Run commit as a principal absent from committed `permissions.toml` → denied, `perm.denied`.
8. Commit a malformed `gates.toml` to the target ref, run a commit → denied, `config.invalid` (fail-closed on invalid governance).
9. In a repo with NO `.gitbutler/` governance config committed at the target ref, run a commit → exit 0, commit lands (ungoverned — opt-in by presence; governance activates only once `.gitbutler/*.toml` is committed).

## Tasks

| ID | Title | Agent | Estimate |
|----|-------|-------|----------|
| AUTHZ-001 | Create `but-authz` crate: `Authority`, `AuthoritySet`, `Principal`, `Group`, `Denial` | rust-implementer | 180 min |
| AUTHZ-002 | Ref-pinned governance config loader (`gix`, target-ref blob read) | rust-implementer | 210 min |
| AUTHZ-003 | `authorize()` + `BUT_AGENT_HANDLE` resolution + fail-closed default-deny | rust-implementer | 180 min |
| GATES-001 | Commit gate at `commit_engine::create_commit` (target-ref-only, DryRun-enforced) | rust-implementer | 240 min |
| AUTHZ-007 | Invariant build-gates — no role name, no human-vs-AI predicate, no `Permission` overload | rust-reviewer | 90 min |

## Dependencies

- **Blocks:** Sprint 01b
- **Dependent on:** None

## PRD Coverage

- **Use cases:** UC-AUTHZ-01, UC-AUTHZ-02, UC-AUTHZ-04, UC-GATES-01
- **Criteria:** T-AUTHZ-001/003/004/009/010/012/016/024/027/028/029, T-GATES-001..007, T-LOOP-005/011

## Capability Coverage

- **CAP-AUTHZ-01** — producer: `authorize()` (AUTHZ-003); the commit gate (GATES-001) runs **even under DryRun**.
- **CAP-CONFIG-01** — producer: ref-pinned loader (AUTHZ-002); the commit gate reads branch-protection **target-ref-only** (a working-tree `gates.toml` edit cannot weaken it).

## Coverage Notes

- **Opt-in by presence (product decision):** governance is activated by committing `.gitbutler/*.toml` at the target ref. A target ref with NO committed governance config is **ungoverned → commit allowed**; a PARTIAL (exactly one of the two files) or malformed config fails closed `config.invalid`; both files present and valid → governed → enforce. Soundness: landing on a governed trunk is mediated by the merge gate (Sprint 01b), which reads the *trunk's* target-ref config — a feature branch that self-ungates only affects that branch, and a repo whose trunk never committed governance is ungoverned by the owner's deliberate choice. A stronger anti-config-deletion guarantee (an explicit "governance enabled" signal) is a noted **Sprint-04** hardening candidate, not taken here.
- **Deferred (honest scope):** UC-GATES-01's `T-GATES-016` (mechanism-agnostic parity: virtual-branch vs normal-git) and `T-GATES-017` (opt-in worktree path) are **not** proven in this sprint — they are deferred to **Sprint 04** (GATES-007). Sprint 01a sites the gate at the ref-aware `but-api` `_with_authz` seam, which makes the decision mechanism-independent *by construction*, but cross-mechanism PARITY is verified in Sprint 04. SPRINT coverage cites `T-GATES-001..007` only.
- **Advisory (upstream):** the `admin` functional-catalog cardinality is asserted by **exhaustive membership** (every `Authority` variant present), not a literal count — `01-scope.md` lists 11 namespaced tokens while an earlier note implied 12; the test is decoupled from the count, and the PRD count should be reconciled in a future PRD edit.

## Task Detail Files

Generated by /kb-sprint-tasks-plan on 2026-06-18 (1 full red-hat review cycle — 3 fresh reviewers, 20 findings resolved; deterministic convergence re-validation).

- AUTHZ-001-create-but-authz-crate.md
- AUTHZ-002-ref-pinned-config-loader.md
- AUTHZ-003-authorize-handle-resolution.md
- GATES-001-commit-gate.md
- AUTHZ-007-invariant-build-gates.md
