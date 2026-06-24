---
task: LPR-REM-009
sprint: sprint-09-governance-remediation-lpr-mgmt-hardening
sequence: 11
agent: rust-implementer
estimate_minutes: 30
status: pending
proposed_by: rust-planner
type: REMEDIATION
generated_at: 2026-06-23T13:30:00Z
generated_by: kb-sprint-tasks-plan
---

# LPR-REM-009: Remove/fix untracked list_workspace_rules_scoped.rs

**Agent:** `rust-implementer` (30 min)
**Proposed By:** `rust-planner`
**Type:** REMEDIATION
**Status:** pending
**Depends On:** —
**Blocks:** LPR-REM-005

## Background

**Problem:** An untracked test file crates/gitbutler-tauri/tests/list_workspace_rules_scoped.rs has 9 compile errors and blocks all gitbutler-tauri tests when cargo test -p gitbutler-tauri runs with default targets.

**Why it matters:** One broken file prevents the entire Tauri crate test suite from running, hiding real test signal and blocking CI.

**Current state:** cargo test -p gitbutler-tauri fails because of compile errors in the untracked test. It references but_rules and regex which are not dev-deps of gitbutler-tauri.

**Desired state:** Either the test is completed with proper dev-deps and implemented helpers, or it is deleted if it was not deliberately planned. Recommendation: delete unless product owner confirms intent.

## Critical Constraints

- MUST unblock cargo test -p gitbutler-tauri.
- MUST either add but_rules and regex dev-deps and fix all 9 compile errors, or delete the file after confirming with the human.
- MUST NOT leave the file in a half-compiling state.
- STRICTLY default to delete unless a product owner explicitly asks to finish it.

## Specification

**Objective:** Restore a clean gitbutler-tauri test compile.

**Success state:** cargo test -p gitbutler-tauri compiles and the default test set runs, either with a working list_workspace_rules_scoped test or without the file.

## Acceptance Criteria

### AC-1

- **GIVEN:** The untracked file crates/gitbutler-tauri/tests/list_workspace_rules_scoped.rs and 9 compile errors
- **WHEN:** cargo test -p gitbutler-tauri is run before any fix
- **THEN:** Compile errors are reproduced and documented
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** cargo-test
- **Flow Ref:** sprint-09-step-9

**Scenario:**

- Tier: visible
- Fixtures: (none)
- Cases:
  - Actor: test_runner, Steps: cargo test -p gitbutler-tauri
  - Must observe: compile errors in list_workspace_rules_scoped.rs
  - Must not observe: tests running before compile fixes
- Negative control — would fail if: file compiles already
- Evidence: stderr (capture required: true)

### AC-2

- **GIVEN:** Human confirmation to delete the untracked file
- **WHEN:** The file is deleted and git status is checked
- **THEN:** cargo test -p gitbutler-tauri compiles and the default tests run
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** cargo-test
- **Flow Ref:** sprint-09-step-9

**Scenario:**

- Tier: visible
- Fixtures: (none)
- Cases:
  - Actor: test_runner, Steps: rm crates/gitbutler-tauri/tests/list_workspace_rules_scoped.rs; cargo test -p gitbutler-tauri
  - Must observe: cargo test -p gitbutler-tauri compiles; default tests run
  - Must not observe: compile errors from list_workspace_rules_scoped.rs
- Negative control — would fail if: deletion not performed or still left references
- Evidence: test_output (capture required: true)

### AC-3

- **GIVEN:** Human confirmation to keep and finish the file instead
- **WHEN:** but_rules and regex dev-deps are added and all 9 compile errors are fixed
- **THEN:** cargo test -p gitbutler-tauri --test list_workspace_rules_scoped compiles and passes
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** cargo-test
- **Flow Ref:** sprint-09-step-9

**Scenario:**

- Tier: visible
- Fixtures: workspace_rules_fixture
- Cases:
  - Actor: developer, Steps: add but_rules and regex to [dev-dependencies] in gitbutler-tauri Cargo.toml; implement missing functions; cargo test -p gitbutler-tauri --test list_workspace_rules_scoped
  - Must observe: test compiles and passes
  - Must not observe: unresolved imports; missing functions
- Negative control — would fail if: dev-deps missing or functions unimplemented
- Evidence: test_output (capture required: true)

### AC-4

- **GIVEN:** The final state of the workspace
- **WHEN:** cargo test -p gitbutler-tauri runs
- **THEN:** No compile errors originate from tests/list_workspace_rules_scoped.rs and the suite advances
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** cargo-test
- **Flow Ref:** sprint-09-step-9

**Scenario:**

- Tier: visible
- Fixtures: (none)
- Cases:
  - Actor: test_runner, Steps: cargo test -p gitbutler-tauri
  - Must observe: tests compile; suite runs
  - Must not observe: list_workspace_rules_scoped compile errors
- Negative control — would fail if: file still broken
- Evidence: test_output (capture required: true)

## Test Criteria

| ID   | Statement                                                                    | Maps to AC |
| ---- | ---------------------------------------------------------------------------- | ---------- |
| TC-1 | Baseline confirms compile errors in the untracked file                       | AC-1       |
| TC-2 | Deletion path restores clean gitbutler-tauri test compile                    | AC-2       |
| TC-3 | Completion path with dev-deps and fixes makes the test pass                  | AC-3       |
| TC-4 | Final cargo test -p gitbutler-tauri runs without file-induced compile errors | AC-4       |

## Guardrails

**WRITE-ALLOWED:**

- crates/gitbutler-tauri/tests/list_workspace_rules_scoped.rs
- crates/gitbutler-tauri/Cargo.toml

**WRITE-PROHIBITED:**

- crates/but-api/src/\*
- crates/but-authz/src/\*

## Verification Gates

- **Command:** `cargo test -p gitbutler-tauri`
- **Expected outcome:** compiles and default tests run; no errors from list_workspace_rules_scoped.rs
- **Command:** `cargo check -p gitbutler-tauri --all-targets`
- **Expected outcome:** clean compilation

## Reading List

- `crates/gitbutler-tauri/tests/list_workspace_rules_scoped.rs` lines all — untracked broken test
- `crates/gitbutler-tauri/Cargo.toml` lines all — dev-dependencies section

## Dependencies

- **Depends On:** —
- **Blocks:** LPR-REM-005

## Design

- **Pattern:** Remove accidental untracked broken file unless product owner explicitly requests completion.
- **Anti-pattern:** Keep a half-finished test blocking the suite.
- **References:** —

## Coding Standards

- crates/AGENTS.md

<!-- REQUIREMENT-CONTRACT v1 -->
<!-- {"requirements":[{"id":"AC-1","kind":"acceptance","scenario_ref":"AC-1"},{"id":"AC-2","kind":"acceptance","scenario_ref":"AC-2"},{"id":"AC-3","kind":"acceptance","scenario_ref":"AC-3"},{"id":"AC-4","kind":"acceptance","scenario_ref":"AC-4"},{"id":"TC-1","kind":"test","maps_to_ac":"AC-1"},{"id":"TC-2","kind":"test","maps_to_ac":"AC-2"},{"id":"TC-3","kind":"test","maps_to_ac":"AC-3"},{"id":"TC-4","kind":"test","maps_to_ac":"AC-4"}]} -->
