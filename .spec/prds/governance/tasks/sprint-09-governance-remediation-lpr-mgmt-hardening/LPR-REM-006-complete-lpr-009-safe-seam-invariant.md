---
task: LPR-REM-006
sprint: sprint-09-governance-remediation-lpr-mgmt-hardening
sequence: 11
agent: rust-reviewer
estimate_minutes: 180
status: pending
proposed_by: rust-planner
type: REMEDIATION
generated_at: 2026-06-23T13:30:00Z
generated_by: kb-sprint-tasks-plan
---

# LPR-REM-006: Complete LPR-009 safe-seam invariant

**Agent:** `rust-reviewer` (180 min)
**Proposed By:** `rust-planner`
**Type:** REMEDIATION
**Status:** pending
**Depends On:** —
**Blocks:** —

## Background

**Problem:** Required tests for the safe-seam invariant are missing from crates/but-authz/tests/invariant_build_gates.rs, leaving the LPR-009 honesty property uncovered.

**Why it matters:** The safe-seam invariant guarantees that forged drives are indistinguishable from empty drives and can only be flipped by an approved verdict at the head; without these tests regressions are likely.

**Current state:** Honesty grep currently lives in crates/but-api/tests/safe*seam_invariant.rs but does not include the three required capstone scenarios or the SAFE_SEAM*\* enforcement paths.

**Desired state:** Tests move into but-authz, ENFORCEMENT_PATHS gains SAFE_SEAM_NO_READ, SAFE_SEAM_GATE_PATHS, and REVIEW_REQUIREMENT, and the three required scenarios pass.

## Critical Constraints

- MUST move the safe-seam honesty invariant tests from but-api to but-authz/tests/invariant_build_gates.rs.
- MUST add SAFE_SEAM_NO_READ, SAFE_SEAM_GATE_PATHS, REVIEW_REQUIREMENT constants to ENFORCEMENT_PATHS.
- MUST implement safe_seam_forged_drive_equals_empty_drive, safe_seam_only_verdict_at_head_flips_the_land, and safe_seam_drive_rows_have_no_effect_on_satisfied_merge.
- MUST keep the 3-step chained capstone: Step1 blocked, Step2 forged drive still blocked with identical decision, Step3 approved verdict proceeds.
- NEVER stub the assertions; tests must validate real database/git state.

## Specification

**Objective:** Add comprehensive safe-seam invariant coverage under but-authz.

**Success state:** cargo test -p but-authz invariant_build_gates passes and the new LPR-009 safe-seam scenarios are green.

## Acceptance Criteria

### AC-1

- **GIVEN:** Invariant tests currently in crates/but-api/tests/safe_seam_invariant.rs
- **WHEN:** The file is relocated into crates/but-authz/tests/invariant_build_gates.rs
- **THEN:** cargo test -p but-authz invariant_build_gates includes the moved coverage and passes
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** cargo-test
- **Flow Ref:** sprint-09-step-6

**Scenario:**

- Tier: visible
- Fixtures: safe_seam_fixture
- Cases:
  - Actor: test_runner, Steps: move safe_seam tests to but-authz; cargo test -p but-authz invariant_build_gates
  - Must observe: tests compile; existing safe_seam assertions pass
  - Must not observe: missing module; test ignored
- Negative control — would fail if: tests not moved correctly
- Evidence: test_output (capture required: true)

### AC-2

- **GIVEN:** ENFORCEMENT_PATHS in but-authz
- **WHEN:** SAFE_SEAM_NO_READ, SAFE_SEAM_GATE_PATHS, and REVIEW_REQUIREMENT constants are added
- **THEN:** cargo test -p but-authz compiles and the constants are referenced by safe-seam tests
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** cargo-test
- **Flow Ref:** sprint-09-step-6

**Scenario:**

- Tier: visible
- Fixtures: (none)
- Cases:
  - Actor: test_runner, Steps: add constants to ENFORCEMENT_PATHS; cargo check -p but-authz --all-targets
  - Must observe: clean compile; constants used in safe_seam tests
  - Must not observe: unused constant warnings
- Negative control — would fail if: constants not wired
- Evidence: test_output (capture required: true)

### AC-3

- **GIVEN:** A merge request blocked by policy
- **WHEN:** A forged drive is created and then an approved verdict is applied at head
- **THEN:** Only the approved verdict at head flips the land; the forged drive alone does not
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** cargo-test
- **Flow Ref:** sprint-09-step-6

**Scenario:**

- Tier: visible
- Fixtures: chained_merge_fixture
- Cases:
  - Actor: automated_test, Steps: Step1: assert merge blocked by read; Step2: add forged drive; assert same blocked decision; Step3: add approved verdict at head; assert merge proceeds
  - Must observe: Step1 blocked; Step2 blocked with identical decision; Step3 allowed
  - Must not observe: forged drive flips the land; different decision in Step2
- Negative control — would fail if: forged drive incorrectly allowed merge; verdict ignored
- Evidence: test_output (capture required: true)

### AC-4

- **GIVEN:** A merge that already satisfies all requirements
- **WHEN:** Pending assignments, changes_requested verdicts, and unresolved comments exist as drive rows
- **THEN:** The satisfied merge decision remains unchanged and the drive rows have no effect
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** cargo-test
- **Flow Ref:** sprint-09-step-6

**Scenario:**

- Tier: visible
- Fixtures: satisfied_merge_fixture
- Cases:
  - Actor: automated_test, Steps: create satisfied merge state; add pending assignment + changes_requested + unresolved comment drive rows; re-evaluate merge
  - Must observe: merge remains allowed; decision unchanged
  - Must not observe: merge blocked by drive rows
- Negative control — would fail if: drive rows can regress a satisfied merge
- Evidence: test_output (capture required: true)

## Test Criteria

| ID   | Statement                                                         | Maps to AC |
| ---- | ----------------------------------------------------------------- | ---------- |
| TC-1 | Safe-seam honesty tests compile and run under but-authz           | AC-1       |
| TC-2 | New ENFORCEMENT_PATHS constants are wired and referenced          | AC-2       |
| TC-3 | 3-step chained capstone verifies only head verdict flips the land | AC-3       |
| TC-4 | Drive rows do not affect a merge that is already satisfied        | AC-4       |

## Guardrails

**WRITE-ALLOWED:**

- crates/but-authz/tests/invariant_build_gates.rs

**WRITE-PROHIBITED:**

- crates/but-api/src/\*
- crates/but-authz/src/\*.rs

## Verification Gates

- **Command:** `cargo test -p but-authz invariant_build_gates`
- **Expected outcome:** all invariant_build_gates tests pass, including new safe-seam scenarios
- **Command:** `cargo check -p but-authz --all-targets`
- **Expected outcome:** clean compilation

## Reading List

- `crates/but-api/tests/safe_seam_invariant.rs` lines all — current honesty grep to move
- `crates/but-authz/tests/invariant_build_gates.rs` lines all — target test module
- `crates/but-authz/src/enforcement.rs` lines all — ENFORCEMENT_PATHS constants

## Dependencies

- **Depends On:** —
- **Blocks:** —

## Design

- **Pattern:** Move tests to the crate that owns the invariant; add named enforcement constants; express capstones as three-step scenarios.
- **Anti-pattern:** Leave safe-seam assertions in a downstream crate or stub the chained scenario.
- **References:** —

## Coding Standards

- crates/AGENTS.md

<!-- REQUIREMENT-CONTRACT v1 -->
<!-- {"requirements":[{"id":"AC-1","kind":"acceptance","scenario_ref":"AC-1"},{"id":"AC-2","kind":"acceptance","scenario_ref":"AC-2"},{"id":"AC-3","kind":"acceptance","scenario_ref":"AC-3"},{"id":"AC-4","kind":"acceptance","scenario_ref":"AC-4"},{"id":"TC-1","kind":"test","maps_to_ac":"AC-1"},{"id":"TC-2","kind":"test","maps_to_ac":"AC-2"},{"id":"TC-3","kind":"test","maps_to_ac":"AC-3"},{"id":"TC-4","kind":"test","maps_to_ac":"AC-4"}]} -->
