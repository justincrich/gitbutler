---
task: STEER-REM-001
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

# STEER-REM-001: Add STEER fields to branch/apply.rs commit-gate CLI serializer

**Agent:** `rust-implementer` (30 min)
**Proposed By:** `rust-planner`
**Type:** REMEDIATION
**Status:** pending
**Depends On:** —
**Blocks:** LPR-REM-008

## Background

**Problem:** commit_gate_cli_error in crates/but/src/command/branch/apply.rs:71-85 emits only {code, message}, silently dropping remediation_hint and all four steering fields.

**Why it matters:** Users receiving a commit-gate denial from branch apply lose steering guidance that sibling sites already provide, leading to poor error recovery and inconsistent CLI UX.

**Current state:** commit2.rs:691-699, forge/review.rs:221-229, and perm.rs:133-142 correctly use steer_envelope_from_parts; apply.rs does not.

**Desired state:** apply.rs replaces commit_gate_cli_error with steer_envelope_from_parts, matching the sibling CLI serializers.

## Critical Constraints

- MUST replace the body of commit_gate_cli_error with a call to steer_envelope_from_parts using the available code, message, remediation_hint, and steering fields.
- MUST match the exact field set emitted by commit2.rs/forge/review.rs/perm.rs.
- MUST NOT introduce custom serialization logic.
- STRICTLY preserve backward compatibility of the code and message fields.

## Specification

**Objective:** Make branch apply commit-gate errors emit the full STEER envelope.

**Success state:** cargo test -p but --features but-2 branch::apply passes and the CLI emits class, held_permissions, authorized_actions, and remediation_hint.

## Acceptance Criteria

### AC-1

- **GIVEN:** A branch apply command that is denied by the commit gate
- **WHEN:** The CLI error is serialized
- **THEN:** The output includes code, message, remediation_hint, class, held_permissions, and authorized_actions
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** but-cli
- **Flow Ref:** sprint-09-step-11

**Scenario:**

- Tier: visible
- Fixtures: commit_gate_denial_fixture
- Cases:
  - Actor: user, Steps: but branch apply <branch>
  - Must observe: class; held_permissions; authorized_actions; remediation_hint
  - Must not observe: only code and message
- Negative control — would fail if: apply.rs still uses old serializer
- Evidence: stdout (capture required: true)

### AC-2

- **GIVEN:** The steer_envelope_from_parts helper used in commit2.rs, forge/review.rs, and perm.rs
- **WHEN:** apply.rs is updated to call steer_envelope_from_parts
- **THEN:** The output shape in apply.rs matches the sibling sites
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** cargo-test
- **Flow Ref:** sprint-09-step-11

**Scenario:**

- Tier: visible
- Fixtures: (none)
- Cases:
  - Actor: test_runner, Steps: cargo test -p but --features but-2 branch::apply
  - Must observe: tests pass; serialized fields match sibling sites
  - Must not observe: shape mismatch
- Negative control — would fail if: helper not reused consistently
- Evidence: test_output (capture required: true)

### AC-3

- **GIVEN:** Existing consumers that parse only code and message
- **WHEN:** The STEER envelope is emitted from apply.rs
- **THEN:** code and message remain present, parseable, and semantically unchanged
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** but-cli
- **Flow Ref:** sprint-09-step-11

**Scenario:**

- Tier: visible
- Fixtures: commit_gate_denial_fixture
- Cases:
  - Actor: user, Steps: but branch apply <branch>
  - Must observe: code field present; message field present; code value unchanged from previous behavior
  - Must not observe: code or message missing
- Negative control — would fail if: new envelope broke existing field names
- Evidence: stdout (capture required: true)

### AC-4

- **GIVEN:** The apply.rs source code
- **WHEN:** cargo check -p but --all-targets runs
- **THEN:** No unused imports remain and no custom serialization logic is present
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** cargo-check
- **Flow Ref:** sprint-09-step-11

**Scenario:**

- Tier: visible
- Fixtures: (none)
- Cases:
  - Actor: reviewer, Steps: cargo check -p but --all-targets; review diff for steer_envelope_from_parts call
  - Must observe: clean compile; steer_envelope_from_parts used
  - Must not observe: custom JSON building
- Negative control — would fail if: old custom function body retained
- Evidence: diff (capture required: true)

## Test Criteria

| ID   | Statement                                                               | Maps to AC |
| ---- | ----------------------------------------------------------------------- | ---------- |
| TC-1 | apply.rs commit-gate error includes all STEER fields                    | AC-1       |
| TC-2 | apply.rs uses steer_envelope_from_parts consistently with sibling sites | AC-2       |
| TC-3 | code and message remain present and unchanged for existing parsers      | AC-3       |
| TC-4 | apply.rs compiles cleanly with the helper reuse                         | AC-4       |

## Guardrails

**WRITE-ALLOWED:**

- crates/but/src/command/branch/apply.rs

**WRITE-PROHIBITED:**

- crates/but/src/command/branch/commit2.rs
- crates/but/src/command/forge/review.rs
- crates/but/src/command/perm.rs
- crates/but-api/src/\*
- crates/but-authz/src/\*

## Verification Gates

- **Command:** `cargo test -p but --features but-2 branch::apply`
- **Expected outcome:** apply tests pass
- **Command:** `cargo check -p but --all-targets`
- **Expected outcome:** clean compilation
- **Command:** `cargo fmt`
- **Expected outcome:** formatting applied

## Reading List

- `crates/but/src/command/branch/apply.rs` lines 71-85 — commit_gate_cli_error old body
- `crates/but/src/command/branch/commit2.rs` lines 691-699 — steer_envelope_from_parts usage
- `crates/but/src/command/forge/review.rs` lines 221-229 — steer_envelope_from_parts usage
- `crates/but/src/command/perm.rs` lines 133-142 — steer_envelope_from_parts usage

## Dependencies

- **Depends On:** —
- **Blocks:** LPR-REM-008

## Design

- **Pattern:** Call the shared steer_envelope_from_parts helper, passing all available fields.
- **Anti-pattern:** Inline custom JSON serialization in apply.rs.
- **References:** —

## Coding Standards

- crates/AGENTS.md

<!-- REQUIREMENT-CONTRACT v1 -->
<!-- {"requirements":[{"id":"AC-1","kind":"acceptance","scenario_ref":"AC-1"},{"id":"AC-2","kind":"acceptance","scenario_ref":"AC-2"},{"id":"AC-3","kind":"acceptance","scenario_ref":"AC-3"},{"id":"AC-4","kind":"acceptance","scenario_ref":"AC-4"},{"id":"TC-1","kind":"test","maps_to_ac":"AC-1"},{"id":"TC-2","kind":"test","maps_to_ac":"AC-2"},{"id":"TC-3","kind":"test","maps_to_ac":"AC-3"},{"id":"TC-4","kind":"test","maps_to_ac":"AC-4"}]} -->
