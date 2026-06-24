---
task: LPR-REM-008
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

# LPR-REM-008: Regenerate stale but CLI snapshots for STEER enrichment

**Agent:** `rust-implementer` (30 min)
**Proposed By:** `rust-planner`
**Type:** REMEDIATION
**Status:** pending
**Depends On:** LPR-REM-001, LPR-REM-002, MGMT-REM-003, STEER-REM-001
**Blocks:** —

## Background

**Problem:** Three CLI snapshots drifted because STEER fields (class, held_permissions, authorized_actions) are now emitted by several commands but the captured snapshots still contain only code and message.

**Why it matters:** Stale snapshots mask the real CLI output shape and will cause CI failures; they must be regenerated and manually verified to include STEER enrichment.

**Current state:** Tests command::help::tests::test_print_grouped, command::governed_merge_cli::merge_denial_is_structured_for_implementer_without_merge_authority, and command::merge_gate::merge_gate_auto_merge_denial_is_structured fail due to snapshot mismatch.

**Desired state:** Snapshots are overwritten with the new STEER-enriched output and a human reviewer confirms the fields are present.

## Critical Constraints

- MUST use SNAPSHOTS=overwrite for each failing test individually.
- MUST verify each new snapshot contains class, held_permissions, and authorized_actions fields where expected.
- MUST NOT blindly accept drift; review the diff for correctness and regressions.
- STRICTLY depends on LPR-REM-001, LPR-REM-002, MGMT-REM-003, and STEER-REM-001 so the output shape is final before snapshot regeneration.

## Specification

**Objective:** Regenerate and validate the three stale but CLI snapshots.

**Success state:** cargo test -p but --features but-2 test_print_grouped, governed_merge_cli merge_denial, and merge_gate auto_merge_denial all pass with verified STEER fields.

## Acceptance Criteria

### AC-1

- **GIVEN:** The test command::help::tests::test_print_grouped is failing due to snapshot drift
- **WHEN:** SNAPSHOTS=overwrite cargo test -p but --features but-2 command::help::tests::test_print_grouped is run
- **THEN:** The test passes and the snapshot includes STEER fields where applicable
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** but-cli
- **Flow Ref:** sprint-09-step-8

**Scenario:**

- Tier: visible
- Fixtures: (none)
- Cases:
  - Actor: developer, Steps: run SNAPSHOTS=overwrite cargo test -p but --features but-2 command::help::tests::test_print_grouped
  - Must observe: snapshot updated; test passes on rerun
  - Must not observe: unintended removal of help sections
- Negative control — would fail if: snapshot not regenerated
- Evidence: snapshot_diff (capture required: true)

### AC-2

- **GIVEN:** The governed_merge_cli merge_denial test is failing due to snapshot drift
- **WHEN:** The snapshot is regenerated with SNAPSHOTS=overwrite
- **THEN:** The new snapshot contains class, held_permissions, and authorized_actions fields
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** but-cli
- **Flow Ref:** sprint-09-step-8

**Scenario:**

- Tier: visible
- Fixtures: merge_denial_fixture
- Cases:
  - Actor: developer, Steps: SNAPSHOTS=overwrite cargo test -p but --features but-2 command::governed_merge_cli::merge_denial_is_structured_for_implementer_without_merge_authority; check new snapshot for class, held_permissions, authorized_actions
  - Must observe: class; held_permissions; authorized_actions in snapshot; test passes
  - Must not observe: only code and message fields remain
- Negative control — would fail if: STEER fields missing from output
- Evidence: snapshot_contents (capture required: true)

### AC-3

- **GIVEN:** The merge_gate auto_merge_denial test is failing due to snapshot drift
- **WHEN:** The snapshot is regenerated with SNAPSHOTS=overwrite
- **THEN:** The new snapshot contains class, held_permissions, and authorized_actions fields
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** but-cli
- **Flow Ref:** sprint-09-step-8

**Scenario:**

- Tier: visible
- Fixtures: merge_gate_fixture
- Cases:
  - Actor: developer, Steps: SNAPSHOTS=overwrite cargo test -p but --features but-2 command::merge_gate::merge_gate_auto_merge_denial_is_structured; verify snapshot includes STEER fields
  - Must observe: class; held_permissions; authorized_actions in snapshot; test passes
  - Must not observe: snapshot missing STEER fields
- Negative control — would fail if: STEER serialization not applied
- Evidence: snapshot_contents (capture required: true)

### AC-4

- **GIVEN:** All three regenerated snapshots
- **WHEN:** cargo test -p but --features but-2 is run for the affected modules
- **THEN:** All three tests pass without further snapshot changes
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** but-cli
- **Flow Ref:** sprint-09-step-8

**Scenario:**

- Tier: visible
- Fixtures: (none)
- Cases:
  - Actor: test_runner, Steps: cargo test -p but --features but-2 command::help; cargo test -p but --features but-2 command::governed_merge_cli; cargo test -p but --features but-2 command::merge_gate
  - Must observe: all three tests pass without SNAPSHOTS=overwrite
  - Must not observe: snapshot drift warnings
- Negative control — would fail if: snapshots not stable after regeneration
- Evidence: test_output (capture required: true)

## Test Criteria

| ID   | Statement                                                                 | Maps to AC |
| ---- | ------------------------------------------------------------------------- | ---------- |
| TC-1 | test_print_grouped snapshot is regenerated and passes                     | AC-1       |
| TC-2 | merge_denial_is_structured snapshot contains STEER fields                 | AC-2       |
| TC-3 | merge_gate_auto_merge_denial_is_structured snapshot contains STEER fields | AC-3       |
| TC-4 | All three affected test modules pass stably after regeneration            | AC-4       |

## Guardrails

**WRITE-ALLOWED:**

- crates/but/tests/snapshots/\*.snap

**WRITE-PROHIBITED:**

- crates/but/src/\*.rs
- crates/but-api/src/\*.rs

## Verification Gates

- **Command:** `SNAPSHOTS=overwrite cargo test -p but --features but-2 command::help::tests::test_print_grouped`
- **Expected outcome:** snapshot regenerated and test passes
- **Command:** `SNAPSHOTS=overwrite cargo test -p but --features but-2 command::governed_merge_cli::merge_denial_is_structured_for_implementer_without_merge_authority`
- **Expected outcome:** snapshot regenerated and test passes with STEER fields
- **Command:** `SNAPSHOTS=overwrite cargo test -p but --features but-2 command::merge_gate::merge_gate_auto_merge_denial_is_structured`
- **Expected outcome:** snapshot regenerated and test passes with STEER fields
- **Command:** `cargo test -p but --features but-2 command::help command::governed_merge_cli command::merge_gate`
- **Expected outcome:** all affected tests pass without SNAPSHOTS=overwrite

## Reading List

- `crates/but/src/command/help.rs` lines all — test_print_grouped target
- `crates/but/src/command/governed_merge_cli.rs` lines all — merge denial structured output
- `crates/but/src/command/merge_gate.rs` lines all — auto merge denial structured output
- `crates/but/tests/snapshots` lines all — snapshot files to update

## Dependencies

- **Depends On:** LPR-REM-001, LPR-REM-002, MGMT-REM-003, STEER-REM-001
- **Blocks:** —

## Design

- **Pattern:** Regenerate snapshots after dependent code changes and manually verify field presence.
- **Anti-pattern:** Regenerate snapshots before output shape is final or accept drift without inspection.
- **References:** —

## Coding Standards

- crates/AGENTS.md

<!-- REQUIREMENT-CONTRACT v1 -->
<!-- {"requirements":[{"id":"AC-1","kind":"acceptance","scenario_ref":"AC-1"},{"id":"AC-2","kind":"acceptance","scenario_ref":"AC-2"},{"id":"AC-3","kind":"acceptance","scenario_ref":"AC-3"},{"id":"AC-4","kind":"acceptance","scenario_ref":"AC-4"},{"id":"TC-1","kind":"test","maps_to_ac":"AC-1"},{"id":"TC-2","kind":"test","maps_to_ac":"AC-2"},{"id":"TC-3","kind":"test","maps_to_ac":"AC-3"},{"id":"TC-4","kind":"test","maps_to_ac":"AC-4"}]} -->
