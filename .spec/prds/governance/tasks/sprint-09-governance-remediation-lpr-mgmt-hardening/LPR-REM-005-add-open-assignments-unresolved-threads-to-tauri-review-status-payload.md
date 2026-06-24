---
task: LPR-REM-005
sprint: sprint-09-governance-remediation-lpr-mgmt-hardening
sequence: 11
agent: rust-implementer
estimate_minutes: 90
status: pending
proposed_by: rust-planner
type: REMEDIATION
generated_at: 2026-06-23T13:30:00Z
generated_by: kb-sprint-tasks-plan
---

# LPR-REM-005: Add open_assignments/unresolved_threads to Tauri review_status payload

**Agent:** `rust-implementer` (90 min)
**Proposed By:** `rust-planner`
**Type:** REMEDIATION
**Status:** pending
**Depends On:** LPR-REM-001, LPR-REM-002, LPR-REM-009
**Blocks:** —

## Background

**Problem:** Two Tauri LPR review-reads tests fail because ReviewStatus at crates/but-api/src/legacy/forge.rs:1018-1039 lacks pre-filtered open_assignments and unresolved_threads arrays.

**Why it matters:** The desktop UI and downstream APIs need ready-to-render lists of active assignments and unresolved comment threads; without them both tests panic and the UI must duplicate filtering logic.

**Current state:** cargo test -p gitbutler-tauri --test lpr_review_reads fails 2/4 tests. ReviewStatus only includes raw or differently scoped fields.

**Desired state:** ReviewStatus exposes open_assignments: Vec<LocalReviewAssignment> and unresolved_threads: Vec<LocalReviewComment>; both Tauri lpr_review_reads tests pass.

## Critical Constraints

- MUST add open_assignments and unresolved_threads to ReviewStatus with exact field names.
- MUST filter open_assignments to those whose state is open/pending relative to the branch scope.
- MUST filter unresolved_threads to comment threads where resolved_at IS NULL.
- MUST keep existing fields unchanged to avoid breaking already-passing consumers.
- STRICTLY depends on LPR-REM-001 and LPR-REM-002 so comment/assignment data exists to filter.

## Specification

**Objective:** Complete the ReviewStatus DTO with filtered open assignments and unresolved comment threads.

**Success state:** cargo test -p gitbutler-tauri --test lpr_review_reads passes all four tests without panics.

## Acceptance Criteria

### AC-1

- **GIVEN:** A review status payload returned for a branch with open assignments
- **WHEN:** The but-api review_status function executes
- **THEN:** ReviewStatus.open_assignments contains only branch-scoped open assignments
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** tauri-test
- **Flow Ref:** sprint-09-step-5

**Scenario:**

- Tier: visible
- Fixtures: lpr_tauri_fixture
- Cases:
  - Actor: tauri_command, Steps: invoke review_status for a branch with open assignments
  - Must observe: open_assignments array is non-empty; entries are LocalReviewAssignment structs; no entries for other branches
  - Must not observe: missing field open_assignments; self-scoped assignments
- Negative control — would fail if: field not added or filter missing
- Evidence: test_output (capture required: true)

### AC-2

- **GIVEN:** A review status payload returned for a branch with unresolved comment threads
- **WHEN:** The but-api review_status function executes
- **THEN:** ReviewStatus.unresolved_threads contains only threads with resolved_at IS NULL
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** tauri-test
- **Flow Ref:** sprint-09-step-5

**Scenario:**

- Tier: visible
- Fixtures: lpr_tauri_fixture
- Cases:
  - Actor: tauri_command, Steps: invoke review_status for a branch with unresolved and resolved threads
  - Must observe: unresolved_threads contains only unresolved rows; resolved threads excluded
  - Must not observe: resolved threads in unresolved_threads
- Negative control — would fail if: resolved_at filter missing
- Evidence: test_output (capture required: true)

### AC-3

- **GIVEN:** The failing Tauri integration test lpr_review_reads_are_branch_scoped_not_self_scoped
- **WHEN:** cargo test -p gitbutler-tauri --test lpr_review_reads runs
- **THEN:** The test at lpr_review_reads.rs:316 no longer panics
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** tauri-test
- **Flow Ref:** sprint-09-step-5

**Scenario:**

- Tier: visible
- Fixtures: lpr_review_reads_fixture
- Cases:
  - Actor: test_runner, Steps: cargo test -p gitbutler-tauri --test lpr_review_reads review_reads_are_branch_scoped_not_self_scoped
  - Must observe: test passes
  - Must not observe: panicked at lpr_review_reads.rs:316
- Negative control — would fail if: ReviewStatus still missing required fields
- Evidence: test_output (capture required: true)

### AC-4

- **GIVEN:** The failing Tauri integration test lpr_review_reads_register_and_return_branch_drive_state
- **WHEN:** cargo test -p gitbutler-tauri --test lpr_review_reads runs
- **THEN:** The test at lpr_review_reads.rs:81 no longer panics
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** tauri-test
- **Flow Ref:** sprint-09-step-5

**Scenario:**

- Tier: visible
- Fixtures: lpr_review_reads_fixture
- Cases:
  - Actor: test_runner, Steps: cargo test -p gitbutler-tauri --test lpr_review_reads lpr_review_reads_register_and_return_branch_drive_state
  - Must observe: test passes
  - Must not observe: panicked at lpr_review_reads.rs:81
- Negative control — would fail if: ReviewStatus shape not fixed
- Evidence: test_output (capture required: true)

## Test Criteria

| ID   | Statement                                                      | Maps to AC |
| ---- | -------------------------------------------------------------- | ---------- |
| TC-1 | ReviewStatus.open_assignments is present and branch-scoped     | AC-1       |
| TC-2 | ReviewStatus.unresolved_threads excludes resolved threads      | AC-2       |
| TC-3 | lpr_review_reads_are_branch_scoped_not_self_scoped passes      | AC-3       |
| TC-4 | lpr_review_reads_register_and_return_branch_drive_state passes | AC-4       |

## Guardrails

**WRITE-ALLOWED:**

- crates/but-api/src/legacy/forge.rs

**WRITE-PROHIBITED:**

- crates/but-authz/src/\*
- crates/but/src/args/forge.rs

## Verification Gates

- **Command:** `cargo test -p gitbutler-tauri --test lpr_review_reads`
- **Expected outcome:** all four lpr_review_reads tests pass
- **Command:** `cargo check -p but-api --all-targets`
- **Expected outcome:** clean compilation
- **Command:** `cargo check -p gitbutler-tauri --all-targets`
- **Expected outcome:** clean compilation

## Reading List

- `crates/but-api/src/legacy/forge.rs` lines 1018-1039 — ReviewStatus struct
- `crates/gitbutler-tauri/tests/lpr_review_reads.rs` lines 81 — failing test start
- `crates/gitbutler-tauri/tests/lpr_review_reads.rs` lines 316 — failing branch-scoped test

## Dependencies

- **Depends On:** LPR-REM-001, LPR-REM-002, LPR-REM-009
- **Blocks:** —

## Design

- **Pattern:** Extend DTO with filtered derived arrays computed from the same internal query results.
- **Anti-pattern:** Add separate round-trips to the database for each array.
- **References:** —

## Coding Standards

- crates/AGENTS.md

<!-- REQUIREMENT-CONTRACT v1 -->
<!-- {"requirements":[{"id":"AC-1","kind":"acceptance","scenario_ref":"AC-1"},{"id":"AC-2","kind":"acceptance","scenario_ref":"AC-2"},{"id":"AC-3","kind":"acceptance","scenario_ref":"AC-3"},{"id":"AC-4","kind":"acceptance","scenario_ref":"AC-4"},{"id":"TC-1","kind":"test","maps_to_ac":"AC-1"},{"id":"TC-2","kind":"test","maps_to_ac":"AC-2"},{"id":"TC-3","kind":"test","maps_to_ac":"AC-3"},{"id":"TC-4","kind":"test","maps_to_ac":"AC-4"}]} -->
