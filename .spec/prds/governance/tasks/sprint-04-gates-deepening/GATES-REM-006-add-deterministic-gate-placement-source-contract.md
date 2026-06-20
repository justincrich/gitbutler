# GATES-REM-006: Add deterministic source-contract test for gate-before-guard placement

## What this does

GATES-007 AC-3 claims the gate fires before the worktree lock at the public seam, but the verification only counts per-file occurrences of `enforce_commit_gate_for_target`. A count can pass with the gate call placed after the lock or inside the wrong function. This task adds a deterministic source-contract script that parses the function bodies of `apply`, `apply_branch_integration`, and `worktree_integrate` and asserts the gate call occurs before `exclusive_worktree_access`, with no gate calls inside `apply_with_perm` or `apply_branch_integration_with_perm`.

## Why

Sprint 04 · PRD UC-GATES-01 · capabilities CAP-AUTHZ-01, CAP-CONFIG-01. Lock-ordering is a safety property: authorization must run before the lock is acquired, and the lock must not be overloaded as the authorization carrier. A fakeable count check does not prove this property. A source-contract test parses the actual code and enforces the ordering.

## How to verify

PRIMARY **AC-1** — `tools/governance-checks/check_gate_before_guard.py` (or shell equivalent) exits 0 on the current code and would fail if the ordering is violated.

## Scope

- New script `tools/governance-checks/check_gate_before_guard.py` (CREATE) — parses `crates/but-api/src/branch.rs` and `crates/but-api/src/legacy/worktree.rs` and checks that `enforce_commit_gate_for_target` appears before `exclusive_worktree_access` in the public entry-point functions, and that no gate calls appear in the `_with_perm` helper bodies.
- GATES-007 task file (MODIFY) — update AC-3 and TC-7 to reference the new script.

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: GATES-REM-006 - Add deterministic source-contract test for gate-before-guard placement
================================================================================

TASK_TYPE:  INFRA
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     M (120 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-GATES-01, T-GATES-016, T-GATES-017
CAPABILITIES: CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  ./tools/governance-checks/check_gate_before_guard.py
  check: cargo check -p but-api --all-targets
  lint:  cargo clippy -p but-api --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
A deterministic source-contract test proves that `enforce_commit_gate_for_target` is called in the public `apply`, `apply_branch_integration`, and `worktree_integrate` functions before those functions acquire `exclusive_worktree_access`, and that the `_with_perm` helper bodies contain no gate calls. The test is wired into GATES-007 AC-3/TC-7.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST parse the target source files and identify the function bodies of `apply`, `apply_branch_integration`, and `worktree_integrate` in `crates/but-api/src/branch.rs` and `worktree_integrate` in `crates/but-api/src/legacy/worktree.rs`.
- [MUST] MUST assert that `enforce_commit_gate_for_target` appears before `exclusive_worktree_access` in each public entry-point function body.
- [MUST] MUST assert that `apply_with_perm` and `apply_branch_integration_with_perm` contain no calls to `enforce_commit_gate_for_target`.
- [MUST] MUST produce a deterministic exit code (0 = pass, non-zero = fail) and a clear error message naming the violation.
- [NEVER] NEVER rely on a simple count of occurrences across the whole file; the check must be scoped to the correct function bodies.
- [STRICTLY] STRICTLY keep the script portable (Python 3 stdlib or bash + awk/sed) and repo-relative.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: the source-contract script exits 0 on the current code and fails if the gate is placed after the guard or inside a `_with_perm` helper.
- [ ] AC-2: GATES-007 AC-3 and TC-7 reference the script.
- [ ] AC-3: the script has a documented negative test (manual) showing it catches a violation.
- [ ] All verification gates pass; only write_allowed files modified.

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA
--------------------------------------------------------------------------------

AC-1: Source-contract script proves gate-before-guard placement [PRIMARY]
  GIVEN: `crates/but-api/src/branch.rs` and `crates/but-api/src/legacy/worktree.rs` with the GATES-007 gate wiring
  WHEN:  the source-contract script is run
  THEN:  it exits 0 and reports that `enforce_commit_gate_for_target` precedes `exclusive_worktree_access` in `apply`, `apply_branch_integration`, and `worktree_integrate`, and that no gate calls appear in `apply_with_perm` or `apply_branch_integration_with_perm`
  TEST_TIER: unit (script)   VERIFICATION_SERVICE: static source analysis
  VERIFY: ./tools/governance-checks/check_gate_before_guard.py
  SCENARIO: NEGATIVE_CONTROL would fail if the script were a fakeable count or did not check function body ordering.

AC-2: GATES-007 references the script
  GIVEN: GATES-007 task file
  WHEN:  the AC-3/TC-7 verification commands are updated
  THEN:  they run `check_gate_before_guard.py` instead of (or in addition to) the per-file count check
  TEST_TIER: unit (documentation)   VERIFICATION_SERVICE: source grep
  VERIFY: grep -E 'check_gate_before_guard\.py' .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-007-mechanism-agnostic-commit-gate.md

AC-3: Script has a documented negative test
  GIVEN: the script
  WHEN:  the implementer adds a header comment or a small test case showing the failure mode
  THEN:  a reviewer can manually verify the script rejects a gate-after-guard or gate-in-helper placement
  TEST_TIER: unit (documentation)   VERIFICATION_SERVICE: source review
  VERIFY: grep -E 'NEGATIVE_TEST|manual test|gate after guard' tools/governance-checks/check_gate_before_guard.py

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): `check_gate_before_guard.py` exits 0 on the current code.
    VERIFY: ./tools/governance-checks/check_gate_before_guard.py
- TC-2 (-> AC-1): `check_gate_before_guard.py` would fail if `apply_with_perm` contained a gate call.
    VERIFY: (documented in script header or a committed test fixture)
- TC-3 (-> AC-2): GATES-007 task file references the script.
    VERIFY: grep -E 'check_gate_before_guard\.py' .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-007-mechanism-agnostic-commit-gate.md
- TC-4 (-> AC-3): the script documents a negative test.
    VERIFY: grep -E 'NEGATIVE_TEST|manual test|gate after guard' tools/governance-checks/check_gate_before_guard.py

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides: a deterministic source-level proof that authorization runs before the worktree lock and never inside the permission-locked helper bodies.
consumes: GATES-007's gate placement in `branch.rs` and `worktree.rs`.
boundary_contracts:
  - CAP-AUTHZ-01: the repo lock is orthogonal to authorization; the source-contract test enforces that separation at the public entry points.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - tools/governance-checks/check_gate_before_guard.py (CREATE) — deterministic source-contract test for gate-before-guard placement.
  - .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-007-mechanism-agnostic-commit-gate.md (MODIFY) — update AC-3/TC-7 to reference the script.
writeProhibited:
  - Any production source file.
  - The substantive AC/TC text (only verify commands change).
  - Any file not listed in write_allowed.

--------------------------------------------------------------------------------
OUT OF SCOPE
--------------------------------------------------------------------------------
  - Changing the implementation of GATES-007.
  - Adding runtime tests (the source-contract test is static analysis).
  - Checking other entry points beyond the three GATES-007 seams.

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-api/src/branch.rs (643-659, 939-985)
   Focus: the public `apply` and `apply_branch_integration` functions and their `_with_perm` helpers; identify where the gate and guard appear.
2. crates/but-api/src/legacy/worktree.rs (52-64)
   Focus: the public `worktree_integrate` function and its guard.
3. .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-007-mechanism-agnostic-commit-gate.md (125-126, 207-208, 473, 508)
   Focus: the existing AC-3/TC-7 verification and the placement prose.
4. brain/docs/verification-discovery.md
   Focus: patterns for deterministic source checks.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- Source-contract script exists and passes: `./tools/governance-checks/check_gate_before_guard.py` -> Exit 0.
- GATES-007 references it: grep above.
- Script documents a negative test: grep above.
- Crate compiles: `cargo check -p but-api --all-targets` -> Exit 0.
- Clippy clean: `cargo clippy -p but-api --all-targets` -> Exit 0.

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: Use a lightweight static parser (Python with regex or a simple brace-aware scanner) to locate each target function body, then find the first occurrence of `enforce_commit_gate_for_target` and `exclusive_worktree_access` within that body. Assert the gate call precedes the guard. Also scan the `_with_perm` helper bodies and assert no gate calls. The script is deterministic and can be run in CI.
pattern_source: crates/but-api/src/branch.rs:643-647 (gate before guard) and the red-hat F6 finding.
anti_pattern: A whole-file count of `enforce_commit_gate_for_target`; a grep that ignores function boundaries; a manual review step that is not automated.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — Builds a deterministic source-contract script that parses the GATES-007 public seams and proves the gate call precedes the worktree lock, with no gate calls inside the `_with_perm` helpers.
reviewer: rust-reviewer
coding_standards: crates/AGENTS.md, brain/docs/verification-discovery.md

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: GATES-007 (the placement to verify)
Blocks:     Sprint 04 structural-invariant closure
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "GATES-REM-006",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": false,
    "requires_red_evidence": true,
    "requires_seeded_evidence": false
  },
  "requirements": [
    { "id": "AC-1", "type": "acceptance_criterion", "primary": true, "description": "Source-contract script proves gate-before-guard placement in the three GATES-007 public seams", "verify": "./tools/governance-checks/check_gate_before_guard.py", "maps_to_ac": null },
    { "id": "AC-2", "type": "acceptance_criterion", "description": "GATES-007 AC-3/TC-7 reference the source-contract script", "verify": "grep -E 'check_gate_before_guard\\.py' .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-007-mechanism-agnostic-commit-gate.md", "maps_to_ac": null },
    { "id": "AC-3", "type": "acceptance_criterion", "description": "The script documents a negative test showing it catches a violation", "verify": "grep -E 'NEGATIVE_TEST|manual test|gate after guard' tools/governance-checks/check_gate_before_guard.py", "maps_to_ac": null },
    { "id": "TC-1", "type": "test_criterion", "description": "check_gate_before_guard.py exits 0 on the current code", "verify": "./tools/governance-checks/check_gate_before_guard.py", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "check_gate_before_guard.py would fail if apply_with_perm contained a gate call", "verify": "(documented in script header or committed test fixture)", "maps_to_ac": "AC-1" },
    { "id": "TC-3", "type": "test_criterion", "description": "GATES-007 task file references the script", "verify": "grep -E 'check_gate_before_guard\\.py' .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-007-mechanism-agnostic-commit-gate.md", "maps_to_ac": "AC-2" },
    { "id": "TC-4", "type": "test_criterion", "description": "The script documents a negative test", "verify": "grep -E 'NEGATIVE_TEST|manual test|gate after guard' tools/governance-checks/check_gate_before_guard.py", "maps_to_ac": "AC-3" }
  ]
}
-->
</content>
</invoke>
