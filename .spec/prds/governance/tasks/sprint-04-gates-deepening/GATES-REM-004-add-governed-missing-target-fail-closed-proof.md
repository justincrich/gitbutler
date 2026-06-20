# GATES-REM-004: Add governed missing-target fail-closed proof for no-target apply/integrate

## What this does

GATES-007 introduced a MATCH on `ctx.project_meta()?.target_ref_or_err()`: if the workspace has no configured default target, the commit gate is skipped and the operation is permitted. The existing `commit_gate_apply_integrate_no_target_ungoverned` test proves this behavior on a repo that also has no committed `.gitbutler/*.toml` (i.e., an ungoverned repo). This task adds the missing negative proof: a repo that has committed governance but no default target metadata must either be denied safely or proven impossible to reach.

## Why

Sprint 04 · PRD UC-GATES-01 · capability CAP-CONFIG-01. "No target" cannot be a silent fail-open path if a governed repo can enter that state. The red-hat review found that the no-target case is only proven for ungoverned repos; this task closes that gap.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-api commit_gate_governed_missing_target_failclosed` (integration, real but-api + real git).

## Scope

- `crates/but-api/tests/commit_gate.rs` (MODIFY) — add a test fixture where governance is committed on `refs/heads/main` but the project has no default target set, then prove that `branch::apply` / `apply_branch_integration` are denied safely (or prove such a state cannot occur).
- `crates/but-api/src/branch.rs` (MODIFY — only if a behavior change is needed) — tighten the no-target handling so that governance present + missing target fails closed rather than skipping the gate.

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: GATES-REM-004 - Add governed missing-target fail-closed proof for no-target apply/integrate
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     M (120 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-GATES-01
CAPABILITIES: CAP-CONFIG-01, CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-api commit_gate_governed_missing_target_failclosed
  check: cargo check -p but-api --all-targets
  lint:  cargo clippy -p but-api --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Either (a) the no-target skip is proven safe because a governed repo cannot have a missing default target, or (b) the gate treats a governed repo with a missing target as fail-closed (`config.invalid` or `perm.denied`). The test uses a repo with committed `.gitbutler/*.toml` on `refs/heads/main` but no default target configured, and asserts the expected safe outcome for both `branch::apply` and `apply_branch_integration`.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST exercise the case where governance is present on the target ref but the project metadata has no default target (`target_ref_or_err()` returns `DefaultTargetNotFound`). This is the gap left by GATES-007's ungoverned-repo no-target test.
- [MUST] MUST assert one of the two fail-closed outcomes: (a) the operation is denied (`config.invalid` or `perm.denied`) because the governed state is inconsistent, OR (b) the project can provably never enter this state and the existing skip is therefore safe.
- [MUST] MUST reuse the existing commit-gate decision helper and the same fixture shape as GATES-007 (`gated_apply_repo`).
- [NEVER] NEVER introduce a vacuous gate that exits Ok when governance is present but the target ref is missing.
- [STRICTLY] STRICTLY seed via the public API or CLI path; never a direct DB insert or raw git mutation.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: a governed repo with no default target is either denied safely or proven impossible to reach.
- [ ] AC-2: the existing ungoverned no-target positive test still passes (no regression).
- [ ] All verification gates pass; only write_allowed files modified.

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: Governed missing-target case is fail-closed [PRIMARY]
  GIVEN: fixture `gated_apply_repo_missing_target` (committed `.gitbutler/gates.toml` and `.gitbutler/permissions.toml` on refs/heads/main; NO default target configured; feature branch with an applicable/integratable change), BUT_AGENT_HANDLE=ro
  WHEN:  the public `branch::apply` and `apply_branch_integration` are invoked
  THEN:  one of the following holds deterministically: (a) both operations are denied `config.invalid` (or `perm.denied`) before any ref mutation, OR (b) the test documents and proves that the project's metadata layer cannot leave a governed repo in this state. No operation silently proceeds ungated.
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api branch seam + real git
  VERIFY: cargo test -p but-api commit_gate_governed_missing_target_failclosed
  SCENARIO: NEGATIVE_CONTROL would fail if the gate is skipped because the target is missing, allowing a read-only principal to mutate refs in a governed repo.

AC-2: Ungoverned no-target behavior is unchanged
  GIVEN: fixture `apply_repo_no_target` from GATES-007 (no committed governance, no default target), BUT_AGENT_HANDLE=ro
  WHEN:  the public `branch::apply` and `apply_branch_integration` are invoked
  THEN:  both operations are permitted (gate skipped) as before
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api branch seam + real git
  VERIFY: cargo test -p but-api commit_gate_apply_integrate_no_target_ungoverned
  SCENARIO: NEGATIVE_CONTROL would fail if the missing-target handling regresses and the ungoverned case hard-errors or is wrongly denied.

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): governed repo + missing default target is denied or proven impossible before ref mutation.
    VERIFY: cargo test -p but-api commit_gate_governed_missing_target_failclosed
- TC-2 (-> AC-2): ungoverned repo + missing default target remains permitted (gate skipped).
    VERIFY: cargo test -p but-api commit_gate_apply_integrate_no_target_ungoverned
- TC-3 (-> AC-1, structural): no new `target_ref_or_err()?` propagation that hard-errors a no-target apply.
    VERIFY: ! grep -nE 'target_ref_or_err\(\)\?' crates/but-api/src/branch.rs

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-CONFIG-01, CAP-AUTHZ-01
provides: a fail-closed proof that the no-target gate skip cannot be exploited in a governed repo.
consumes: GATES-007's public `apply`/`apply_branch_integration` gate wiring and the `gated_apply_repo` fixture shape; `ctx.project_meta()?.target_ref_or_err()` and `but_authz::governance_present`.
boundary_contracts:
  - CAP-CONFIG-01: governance is read from the target ref; a missing target in a governed repo is treated as a configuration error rather than a silent opt-out.
  - CAP-AUTHZ-01: authorization is enforced before the worktree lock even when target metadata is absent.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-api/tests/commit_gate.rs (MODIFY) — add the governed missing-target test fixture and cases.
  - crates/but-api/src/branch.rs (CONDITIONAL MODIFY) — only if the chosen outcome requires changing the no-target handling (e.g., to fail closed when `governance_present` is true but target is missing).
writeProhibited:
  - crates/but-api/src/commit/gate.rs — reuse the existing decision helper; do not change gate logic.
  - crates/but-authz/** — consume primitives; do not modify them.
  - Any file not listed in write_allowed.

--------------------------------------------------------------------------------
OUT OF SCOPE
--------------------------------------------------------------------------------
  - Re-testing the ungoverned no-target case (already covered by GATES-007).
  - Re-testing the normal governed target-ref cases (GATES-007 covers those).
  - Modifying the merge gate or review requirement evaluator.

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-api/tests/commit_gate.rs (full, especially 255-289 for no-target fixture)
   Focus: the GATES-007 test harness and the `commit_gate_absent_config_is_ungoverned` shape to extend.
2. crates/but-api/src/branch.rs (643-659, 939-957)
   Focus: the current no-target MATCH handling and the gate-before-guard placement.
3. crates/but-api/src/commit/gate.rs (55-70)
   Focus: the decision helper to reuse; note `governance_present` check.
4. crates/but-core/src/ref_metadata.rs (354-362)
   Focus: `target_ref_or_err()` returns `DefaultTargetNotFound` when target is None.
5. .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-007-mechanism-agnostic-commit-gate.md
   Focus: the no-target ungoverned test and the S9b constraint this task extends.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- Governed missing-target test passes: `cargo test -p but-api commit_gate_governed_missing_target_failclosed` -> Exit 0.
- Ungoverned no-target test still passes: `cargo test -p but-api commit_gate_apply_integrate_no_target_ungoverned` -> Exit 0.
- No hard-error propagation of target_ref_or_err in branch.rs: `! grep -nE 'target_ref_or_err\(\)\?' crates/but-api/src/branch.rs` -> No matches.
- Crate compiles incl. tests: `cargo check -p but-api --all-targets` -> Exit 0.
- Clippy clean: `cargo clippy -p but-api --all-targets` -> Exit 0.

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: Extend the GATES-007 fixture to commit governance on `refs/heads/main` while leaving the project default target unset. Then assert the safe outcome. If the implementation needs to change, the likely shape is: after matching `target_ref_or_err()`, if the result is `Err(DefaultTargetNotFound)` and `governance_present(repo, any_known_ref)` would be true, deny `config.invalid` rather than skip. Otherwise, skip as before.
pattern_source: GATES-007's no-target test and the `governance_present` opt-in discriminator.
anti_pattern: Skipping the gate whenever target metadata is missing, regardless of whether governance is present on the repo.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — Adds the missing governed-repo fail-closed proof for the no-target apply/integrate case, extending the GATES-007 test harness without regressing the ungoverned no-target behavior.
reviewer: rust-reviewer
coding_standards: crates/AGENTS.md, crates/WORKSPACE_MODEL.md, crates/but-api/src/branch.rs

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: GATES-007, GATES-001, AUTHZ-002/003
Blocks:     Sprint 05, Sprint 06b (via Sprint 04 completion)
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "GATES-REM-004",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "requirements": [
    { "id": "AC-1", "type": "acceptance_criterion", "primary": true, "description": "A governed repo with no default target is either denied safely or proven impossible to reach", "verify": "cargo test -p but-api commit_gate_governed_missing_target_failclosed", "maps_to_ac": null },
    { "id": "AC-2", "type": "acceptance_criterion", "description": "The existing ungoverned no-target positive test still passes", "verify": "cargo test -p but-api commit_gate_apply_integrate_no_target_ungoverned", "maps_to_ac": null },
    { "id": "TC-1", "type": "test_criterion", "description": "Governed repo + missing default target is denied or proven impossible before ref mutation", "verify": "cargo test -p but-api commit_gate_governed_missing_target_failclosed", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "Ungoverned repo + missing default target remains permitted (gate skipped)", "verify": "cargo test -p but-api commit_gate_apply_integrate_no_target_ungoverned", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "No new target_ref_or_err()? propagation that hard-errors a no-target apply", "verify": "! grep -nE 'target_ref_or_err\\(\\)\\?' crates/but-api/src/branch.rs", "maps_to_ac": "AC-1" }
  ]
}
-->
</content>
</invoke>
