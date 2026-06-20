# GATES-REM-005: Replace compile-only structural TCs with discriminating checked scripts

## What this does

The red-hat review found that several structural test criteria in GATES-006, GATES-007, and GATES-008 are verified only by `cargo check`, which cannot prove the structural invariants they claim:
- GATES-006 TC-4 (no human-vs-AI / no role-name literal in `review_requirement.rs`) claims grep coverage but verifies only `cargo check`.
- GATES-007 TC-7 (all three entry points call the same commit-gate decision helper) claims per-file parity but verifies only `cargo check`.
- GATES-008 TC-3 (no production merge-path classification change) claims direct-diff ownership but verifies only `cargo check`.

This task adds checked, deterministic scripts for each structural invariant and wires them into the task verification gates.

## Why

Sprint 04 · PRD T-LOOP-011, T-LOOP-005, T-GATES-016/017, T-GATES-019 · capabilities CAP-AUTHZ-01, CAP-CONFIG-01. Structural invariants are anti-stub defenses. If they are only checked by compilation, a stubbed or evaded implementation can still pass. Checked scripts make the invariants replayable and enforceable in CI.

## How to verify

PRIMARY **AC-1** — Running each new script exits 0 and the existing task files are updated to reference them.

## Scope

- New scripts under `tools/governance-checks/` or `crates/but-api/tests/scripts/` (CREATE):
  - `check_no_role_literals.sh` — word-boundary grep for forbidden human/AI/role literals in `review_requirement.rs`.
  - `check_gate_helper_parity.sh` — per-file count of `enforce_commit_gate_for_target` in `branch.rs` (>=2) and `worktree.rs` (>=1).
  - `check_merge_gate_production_unchanged.sh` — baseline-range zero diff for `crates/but-api/src/legacy/merge_gate.rs`.
- `GATES-006`, `GATES-007`, `GATES-008` task files (MODIFY) — update TC verification commands to run the new scripts.

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: GATES-REM-005 - Replace compile-only structural TCs with discriminating checked scripts
================================================================================

TASK_TYPE:  INFRA
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     M (90 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   T-LOOP-011, T-LOOP-005, T-GATES-016, T-GATES-017, T-GATES-019
CAPABILITIES: CAP-AUTHZ-01, CAP-CONFIG-01

RUNTIME_COMMANDS:
  test:  ./tools/governance-checks/check_no_role_literals.sh && ./tools/governance-checks/check_gate_helper_parity.sh && ./tools/governance-checks/check_merge_gate_production_unchanged.sh
  check: cargo check -p but-api --all-targets
  lint:  cargo clippy -p but-api --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Each structural TC in GATES-006/007/008 is backed by a deterministic checked script that can fail if the invariant is violated. The scripts live in the repo, run in CI, and are referenced by the updated task files. The invariants are: no role/human/AI literals in the enforcement source; per-file commit-gate helper parity; zero production diff on merge_gate.rs for GATES-008.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST provide one checked script per structural invariant. Each script must be deterministic and fail closed (exit non-zero) when the invariant is violated.
- [MUST] MUST update the verification commands in GATES-006, GATES-007, and GATES-008 task files to reference the new scripts instead of `cargo check` for those structural TCs.
- [MUST] MUST keep scripts repo-relative and portable (bash + grep, no external dependencies beyond git and shell).
- [NEVER] NEVER replace a behavioral integration test with a script; these scripts only cover pure structural invariants.
- [STRICTLY] STRICTLY preserve the existing behavior of the scripts' target files; this task only adds verification scripts and updates TC verify commands.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: three checked scripts exist and exit 0 for the current code.
- [ ] AC-2: GATES-006 TC-4 verification command references `check_no_role_literals.sh`.
- [ ] AC-3: GATES-007 TC-7 verification command references `check_gate_helper_parity.sh`.
- [ ] AC-4: GATES-008 TC-3 verification command references `check_merge_gate_production_unchanged.sh`.
- [ ] All scripts and updated task files are committed.

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA
--------------------------------------------------------------------------------

AC-1: Checked scripts exist and are deterministic [PRIMARY]
  GIVEN: the structural invariants identified by the red-hat review
  WHEN:  the implementer creates three checked scripts
  THEN:  each script is executable, repo-relative, and exits 0 when the invariant holds and non-zero when it is violated
  TEST_TIER: unit (script)   VERIFICATION_SERVICE: shell execution
  VERIFY: ./tools/governance-checks/check_no_role_literals.sh && ./tools/governance-checks/check_gate_helper_parity.sh && ./tools/governance-checks/check_merge_gate_production_unchanged.sh
  SCENARIO: NEGATIVE_CONTROL would fail if a script is fakeable (e.g., always exits 0) or does not check the exact invariant.

AC-2: GATES-006 TC-4 references the role-literal script
  GIVEN: GATES-006 task file
  WHEN:  the TC-4 verify command is updated
  THEN:  it runs `check_no_role_literals.sh` instead of `cargo check` alone
  TEST_TIER: unit (documentation)   VERIFICATION_SERVICE: source grep
  VERIFY: grep -E 'check_no_role_literals\.sh' .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-006-per-required-group-approval.md

AC-3: GATES-007 TC-7 references the gate-helper parity script
  GIVEN: GATES-007 task file
  WHEN:  the TC-7 verify command is updated
  THEN:  it runs `check_gate_helper_parity.sh` instead of `cargo check` alone
  TEST_TIER: unit (documentation)   VERIFICATION_SERVICE: source grep
  VERIFY: grep -E 'check_gate_helper_parity\.sh' .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-007-mechanism-agnostic-commit-gate.md

AC-4: GATES-008 TC-3 references the production-diff script
  GIVEN: GATES-008 task file
  WHEN:  the TC-3 verify command is updated
  THEN:  it runs `check_merge_gate_production_unchanged.sh` instead of `cargo check` alone
  TEST_TIER: unit (documentation)   VERIFICATION_SERVICE: source grep
  VERIFY: grep -E 'check_merge_gate_production_unchanged\.sh' .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-008-merge-gate-failclosed-target-ref-only.md

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): `check_no_role_literals.sh` exits 0 on the current code and would fail if a forbidden literal were added to `review_requirement.rs`.
    VERIFY: ./tools/governance-checks/check_no_role_literals.sh
- TC-2 (-> AC-1): `check_gate_helper_parity.sh` exits 0 and would fail if `branch.rs` had fewer than 2 or `worktree.rs` had fewer than 1 calls to `enforce_commit_gate_for_target`.
    VERIFY: ./tools/governance-checks/check_gate_helper_parity.sh
- TC-3 (-> AC-1): `check_merge_gate_production_unchanged.sh` exits 0 and would fail if `merge_gate.rs` production code changed relative to the baseline.
    VERIFY: ./tools/governance-checks/check_merge_gate_production_unchanged.sh
- TC-4 (-> AC-2): GATES-006 TC-4 references the role-literal script.
    VERIFY: grep -E 'check_no_role_literals\.sh' .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-006-per-required-group-approval.md
- TC-5 (-> AC-3): GATES-007 TC-7 references the parity script.
    VERIFY: grep -E 'check_gate_helper_parity\.sh' .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-007-mechanism-agnostic-commit-gate.md
- TC-6 (-> AC-4): GATES-008 TC-3 references the production-diff script.
    VERIFY: grep -E 'check_merge_gate_production_unchanged\.sh' .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-008-merge-gate-failclosed-target-ref-only.md

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01, CAP-CONFIG-01
provides: deterministic, checked structural invariants that compilation alone cannot enforce, closing the fakeable-TC gap found by the red-hat review.
consumes: GATES-006/007/008 task files and their existing structural invariants.
boundary_contracts:
  - CAP-AUTHZ-01: the no-role-literal and gate-parity checks prove that authorization behavior is driven by config, not hardcoded labels or misplaced calls.
  - CAP-CONFIG-01: the merge_gate.rs zero-diff check proves the target-ref-only property is tested without modifying the production classification.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - tools/governance-checks/check_no_role_literals.sh (CREATE) — deterministic grep for forbidden role/human/AI literals in `crates/but-api/src/legacy/review_requirement.rs`.
  - tools/governance-checks/check_gate_helper_parity.sh (CREATE) — per-file count of `enforce_commit_gate_for_target` in `branch.rs` and `worktree.rs`.
  - tools/governance-checks/check_merge_gate_production_unchanged.sh (CREATE) — baseline-range zero-diff check for `crates/but-api/src/legacy/merge_gate.rs`.
  - .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-006-per-required-group-approval.md (MODIFY) — update TC-4 verify command.
  - .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-007-mechanism-agnostic-commit-gate.md (MODIFY) — update TC-7 verify command.
  - .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-008-merge-gate-failclosed-target-ref-only.md (MODIFY) — update TC-3 verify command.
writeProhibited:
  - Any production source file.
  - The substantive text of the ACs/TCs in the task files (only the verify command changes).
  - Any file not listed in write_allowed.

--------------------------------------------------------------------------------
OUT OF SCOPE
--------------------------------------------------------------------------------
  - Changing the behavior of GATES-006/007/008 implementations.
  - Adding behavioral integration tests.
  - Modifying the merge gate or review requirement evaluator source.

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-006-per-required-group-approval.md (107-108, 168-169)
   Focus: TC-4 and its current compile-only verify command.
2. .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-007-mechanism-agnostic-commit-gate.md (125-126, 473, 508)
   Focus: TC-7 and the per-file parity requirement.
3. .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-008-merge-gate-failclosed-target-ref-only.md (94-95, 158)
   Focus: TC-3 and the current direct-diff grep.
4. crates/but-api/src/legacy/review_requirement.rs (full)
   Focus: the target file for the no-role-literal check.
5. crates/but-api/src/branch.rs + crates/but-api/src/legacy/worktree.rs
   Focus: the files counted by the parity check.
6. crates/but-api/src/legacy/merge_gate.rs
   Focus: the file guarded by the baseline-range diff check.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- All three scripts exist and are executable: `ls -l tools/governance-checks/*.sh`.
- All three scripts exit 0 on the current code: `tools/governance-checks/check_no_role_literals.sh && tools/governance-checks/check_gate_helper_parity.sh && tools/governance-checks/check_merge_gate_production_unchanged.sh`.
- Each script has a manual negative test: temporarily add a forbidden literal, remove a helper call, or touch `merge_gate.rs` and confirm the script fails (can be documented in the script header, not automated in CI).
- Updated task files reference the scripts: grep results above.
- Crate compiles: `cargo check -p but-api --all-targets` -> Exit 0.
- Clippy clean: `cargo clippy -p but-api --all-targets` -> Exit 0.

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: Small, deterministic bash scripts that grep or diff specific files and exit non-zero on invariant violations. Each script is self-documented with a header explaining what it checks and why. The scripts are invoked by the task verification commands so the invariant is enforced at review time and in CI.
pattern_source: brain/docs/verification-discovery.md and the red-hat review F5 finding.
anti_pattern: Replacing structural invariants with compile-only checks; embedding complex logic inside task files instead of reusable scripts; using non-portable tools.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — Creates the three checked scripts and updates the GATES-006/007/008 task files to reference them, making the structural invariants deterministic and CI-enforceable.
reviewer: rust-reviewer
coding_standards: crates/AGENTS.md, brain/docs/verification-discovery.md

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: GATES-006, GATES-007, GATES-008 (the task files to update)
Blocks:     Sprint 04 structural-invariant closure
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "GATES-REM-005",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": false,
    "requires_red_evidence": true,
    "requires_seeded_evidence": false
  },
  "requirements": [
    { "id": "AC-1", "type": "acceptance_criterion", "primary": true, "description": "Three checked scripts exist and are deterministic for the structural invariants", "verify": "./tools/governance-checks/check_no_role_literals.sh && ./tools/governance-checks/check_gate_helper_parity.sh && ./tools/governance-checks/check_merge_gate_production_unchanged.sh", "maps_to_ac": null },
    { "id": "AC-2", "type": "acceptance_criterion", "description": "GATES-006 TC-4 verification command references check_no_role_literals.sh", "verify": "grep -E 'check_no_role_literals\\.sh' .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-006-per-required-group-approval.md", "maps_to_ac": null },
    { "id": "AC-3", "type": "acceptance_criterion", "description": "GATES-007 TC-7 verification command references check_gate_helper_parity.sh", "verify": "grep -E 'check_gate_helper_parity\\.sh' .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-007-mechanism-agnostic-commit-gate.md", "maps_to_ac": null },
    { "id": "AC-4", "type": "acceptance_criterion", "description": "GATES-008 TC-3 verification command references check_merge_gate_production_unchanged.sh", "verify": "grep -E 'check_merge_gate_production_unchanged\\.sh' .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-008-merge-gate-failclosed-target-ref-only.md", "maps_to_ac": null },
    { "id": "TC-1", "type": "test_criterion", "description": "check_no_role_literals.sh exits 0 and would fail if a forbidden literal were added to review_requirement.rs", "verify": "./tools/governance-checks/check_no_role_literals.sh", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "check_gate_helper_parity.sh exits 0 and would fail if branch.rs has fewer than 2 or worktree.rs has fewer than 1 calls to enforce_commit_gate_for_target", "verify": "./tools/governance-checks/check_gate_helper_parity.sh", "maps_to_ac": "AC-1" },
    { "id": "TC-3", "type": "test_criterion", "description": "check_merge_gate_production_unchanged.sh exits 0 and would fail if merge_gate.rs production code changed relative to the baseline", "verify": "./tools/governance-checks/check_merge_gate_production_unchanged.sh", "maps_to_ac": "AC-1" },
    { "id": "TC-4", "type": "test_criterion", "description": "GATES-006 TC-4 references the role-literal script", "verify": "grep -E 'check_no_role_literals\\.sh' .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-006-per-required-group-approval.md", "maps_to_ac": "AC-2" },
    { "id": "TC-5", "type": "test_criterion", "description": "GATES-007 TC-7 references the parity script", "verify": "grep -E 'check_gate_helper_parity\\.sh' .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-007-mechanism-agnostic-commit-gate.md", "maps_to_ac": "AC-3" },
    { "id": "TC-6", "type": "test_criterion", "description": "GATES-008 TC-3 references the production-diff script", "verify": "grep -E 'check_merge_gate_production_unchanged\\.sh' .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-008-merge-gate-failclosed-target-ref-only.md", "maps_to_ac": "AC-4" }
  ]
}
-->
</content>
</invoke>
