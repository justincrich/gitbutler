---
task: LPR-REM-003
sprint: sprint-09-governance-remediation-lpr-mgmt-hardening
sequence: 11
agent: rust-implementer
estimate_minutes: 120
status: pending
proposed_by: rust-planner
type: REMEDIATION
generated_at: 2026-06-23T13:30:00Z
generated_by: kb-sprint-tasks-plan
---

# LPR-REM-003: Persist keep_reviews_local through UpdateRequest

**Agent:** `rust-implementer` (120 min)
**Proposed By:** `rust-planner`
**Type:** REMEDIATION
**Status:** pending
**Depends On:** —
**Blocks:** —

## Background

**Problem:** Project storage at crates/gitbutler-project/src/storage.rs:86-92 explicitly discards keep_reviews_local when converting Project to UpdateRequest, so edits to the field are silently dropped on save.

**Why it matters:** Users toggling local review mode cannot persist their choice, leading to data loss and unexpected behavior across sessions.

**Current state:** keep_reviews_local exists on Project (project.rs:122 as DefaultTrue) and serializes correctly, but UpdateRequest omits it and Storage::update ignores it.

**Desired state:** UpdateRequest carries keep_reviews_local, Storage::update writes it, and an integration test proves the round-trip.

## Critical Constraints

- MUST add keep_reviews_local to UpdateRequest struct mirroring the Project field type.
- MUST write the field in Storage::update without changing other field persistence.
- MUST preserve the DefaultTrue semantics and existing default-true tests at project.rs:464-492.
- NEVER persist None where a value was explicitly set; preserve user intent.
- STRICTLY add a new integration test showing read-update-read round-trip.

## Specification

**Objective:** Close the write gap for keep_reviews_local so project updates preserve the local-review setting.

**Success state:** cargo test -p gitbutler-project keep_reviews_local passes and the field survives a create-update-read cycle.

## Acceptance Criteria

### AC-1

- **GIVEN:** A project with keep_reviews_local explicitly set to false
- **WHEN:** Storage::update is called via the update API
- **THEN:** The persisted project returns keep_reviews_local=false on subsequent read
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** but-api
- **Flow Ref:** sprint-09-step-3

**Scenario:**

- Tier: visible
- Fixtures: temp_project
- Cases:
  - Actor: storage_api, Steps: create project with keep_reviews_local=false; call Storage::update; read project back
  - Must observe: keep_reviews_local=false after read
  - Must not observe: keep_reviews_local=true; value reverted to default
- Negative control — would fail if: UpdateRequest still discards field
- Evidence: test_output (capture required: true)

### AC-2

- **GIVEN:** A project using the default keep_reviews_local value
- **WHEN:** Storage::update mutates an unrelated field
- **THEN:** keep_reviews_local remains at its default true value
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** but-api
- **Flow Ref:** sprint-09-step-3

**Scenario:**

- Tier: visible
- Fixtures: temp_project
- Cases:
  - Actor: storage_api, Steps: update project name only; read project back
  - Must observe: keep_reviews_local=true
  - Must not observe: keep_reviews_local=false or missing
- Negative control — would fail if: update overwrites default with None
- Evidence: test_output (capture required: true)

### AC-3

- **GIVEN:** The From<Project> for UpdateRequest implementation
- **WHEN:** Conversion is performed
- **THEN:** keep_reviews_local is copied from Project, not set to None
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** but-api
- **Flow Ref:** sprint-09-step-3

**Scenario:**

- Tier: visible
- Fixtures: temp_project
- Cases:
  - Actor: test_runner, Steps: construct Project with keep_reviews_local=false; convert to UpdateRequest; assert field equals false
  - Must observe: UpdateRequest.keep_reviews_local == DefaultTrue(false)
  - Must not observe: field is None
- Negative control — would fail if: From impl still discards field
- Evidence: test_output (capture required: true)

### AC-4

- **GIVEN:** The existing default-true serde tests
- **WHEN:** cargo test -p gitbutler-project runs
- **THEN:** The old tests at project.rs:464-492 still pass
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** cargo-test
- **Flow Ref:** sprint-09-step-3

**Scenario:**

- Tier: visible
- Fixtures: (none)
- Cases:
  - Actor: test_runner, Steps: cargo test -p gitbutler-project
  - Must observe: existing default_true tests pass
  - Must not observe: regressions in serde behavior
- Negative control — would fail if: DefaultTrue semantics changed
- Evidence: test_output (capture required: true)

## Test Criteria

| ID   | Statement                                                           | Maps to AC |
| ---- | ------------------------------------------------------------------- | ---------- |
| TC-1 | keep_reviews_local=false survives a Storage::update round-trip      | AC-1       |
| TC-2 | Unrelated updates leave the default keep_reviews_local value intact | AC-2       |
| TC-3 | From<Project> for UpdateRequest copies keep_reviews_local           | AC-3       |
| TC-4 | Existing DefaultTrue serde tests continue to pass                   | AC-4       |

## Guardrails

**WRITE-ALLOWED:**

- crates/gitbutler-project/src/storage.rs
- crates/gitbutler-project/src/project.rs
- crates/gitbutler-project/src/types.rs
- crates/gitbutler-project/tests/\*

**WRITE-PROHIBITED:**

- crates/but-api/src/\*
- crates/but-authz/src/\*

## Verification Gates

- **Command:** `cargo test -p gitbutler-project keep_reviews_local`
- **Expected outcome:** new round-trip test passes
- **Command:** `cargo test -p gitbutler-project`
- **Expected outcome:** all gitbutler-project tests pass, including existing DefaultTrue tests
- **Command:** `cargo check -p gitbutler-project --all-targets`
- **Expected outcome:** clean compilation

## Reading List

- `crates/gitbutler-project/src/storage.rs` lines 86-92 — From<Project> for UpdateRequest discarding field
- `crates/gitbutler-project/src/storage.rs` lines 164-266 — Storage::update persistence path
- `crates/gitbutler-project/src/project.rs` lines 122 — Project field definition
- `crates/gitbutler-project/src/project.rs` lines 464-492 — existing DefaultTrue serde tests

## Dependencies

- **Depends On:** —
- **Blocks:** —

## Design

- **Pattern:** Mirror Project field in UpdateRequest and copy through in From impl and update path.
- **Anti-pattern:** Add a separate setter or bypass the UpdateRequest abstraction.
- **References:** —

## Coding Standards

- crates/AGENTS.md

<!-- REQUIREMENT-CONTRACT v1 -->
<!-- {"requirements":[{"id":"AC-1","kind":"acceptance","scenario_ref":"AC-1"},{"id":"AC-2","kind":"acceptance","scenario_ref":"AC-2"},{"id":"AC-3","kind":"acceptance","scenario_ref":"AC-3"},{"id":"AC-4","kind":"acceptance","scenario_ref":"AC-4"},{"id":"TC-1","kind":"test","maps_to_ac":"AC-1"},{"id":"TC-2","kind":"test","maps_to_ac":"AC-2"},{"id":"TC-3","kind":"test","maps_to_ac":"AC-3"},{"id":"TC-4","kind":"test","maps_to_ac":"AC-4"}]} -->
