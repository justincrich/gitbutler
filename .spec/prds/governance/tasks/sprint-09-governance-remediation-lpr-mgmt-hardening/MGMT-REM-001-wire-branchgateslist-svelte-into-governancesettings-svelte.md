---
task: MGMT-REM-001
sprint: sprint-09-governance-remediation-lpr-mgmt-hardening
sequence: 11
agent: sveltekit-implementer
estimate_minutes: 30
status: pending
proposed_by: sveltekit-planner
type: REMEDIATION
generated_at: 2026-06-23T13:30:00Z
generated_by: kb-sprint-tasks-plan
---

# MGMT-REM-001: Wire BranchGatesList.svelte into GovernanceSettings.svelte

**Agent:** `sveltekit-implementer` (30 min)
**Proposed By:** `sveltekit-planner`
**Type:** REMEDIATION
**Status:** pending
**Depends On:** —
**Blocks:** —

## Background

**Problem (audit finding):** apps/desktop/src/components/governance/GovernanceSettings.svelte:229-241 renders the Branch Gates tab body as a stub button with no onclick.

**Why it matters (impact):** Users cannot see the fully implemented BranchGatesList because it is never imported or rendered.

**Affected files:**

- apps/desktop/src/components/governance/GovernanceSettings.svelte
- apps/desktop/src/components/governance/BranchGatesList.svelte
- apps/desktop/tests/governance/BranchGatesList.spec.ts

## Critical Constraints

- MUST import and render the existing BranchGatesList component unchanged
- MUST pass props projectId, targetRef, isReadOnly, onRefresh, onBranchPending from GovernanceSettings context
- MUST remove the stub button and replace it with the component
- NEVER duplicate BranchGatesList logic or alter BranchGatesList.svelte

## Specification

**Objective:** Replace the Branch Gates tab stub with a correctly-propped BranchGatesList instance.

**Success state:** Selecting the Branch Gates tab renders the working list; existing component tests still pass; parent-level test confirms the component is mounted in context.

## Acceptance Criteria

### AC-1

- **GIVEN:** GovernanceSettings renders for a project with branch gate capabilities
- **WHEN:** the user selects the Branch Gates tab
- **THEN:** the BranchGatesList component mounts with projectId, targetRef, isReadOnly, onRefresh, and onBranchPending
- **TDD State:** RED
- **Test Tier:** component
- **Verification Service:** playwright-ct
- **Flow Ref:** sprint-09-MGMT-REM-001-AC1

**Scenario:**

- Tier: visible
- Fixtures: seeded_governance_settings_with_branch_gates
- Cases:
  - Actor: admin, Steps: render GovernanceSettings; click Branch Gates tab
  - Must observe: BranchGatesList renders inside the Branch Gates tab body; list contains seeded gate rows
  - Must not observe: stub button with no onclick
- Negative control — would fail if: BranchGatesList is not imported; stub body is not replaced
- Evidence: screenshot (capture required: true)

### AC-2

- **GIVEN:** BranchGatesList is rendered inside GovernanceSettings
- **WHEN:** the list triggers its refresh handler
- **THEN:** GovernanceSettings.onRefresh propagates the event and refreshes gate data
- **TDD State:** RED
- **Test Tier:** component
- **Verification Service:** playwright-ct
- **Flow Ref:** sprint-09-MGMT-REM-001-AC2

**Scenario:**

- Tier: interaction
- Fixtures: seeded_governance_settings_with_branch_gates
- Cases:
  - Actor: admin, Steps: open Branch Gates tab; click the list refresh control
  - Must observe: parent onRefresh callback invoked; refreshing state shown then list updates
  - Must not observe: unhandled event error
- Negative control — would fail if: onRefresh prop is missing or not wired
- Evidence: screenshot (capture required: true)

### AC-3

- **GIVEN:** GovernanceSettings is in read-only governance mode
- **WHEN:** the Branch Gates tab is active
- **THEN:** BranchGatesList receives isReadOnly=true and disables mutating controls
- **TDD State:** RED
- **Test Tier:** component
- **Verification Service:** playwright-ct
- **Flow Ref:** sprint-09-MGMT-REM-001-AC3

**Scenario:**

- Tier: visible
- Fixtures: seeded_governance_settings_read_only
- Cases:
  - Actor: viewer, Steps: render GovernanceSettings in read-only context; open Branch Gates tab
  - Must observe: add/edit gate controls are disabled
  - Must not observe: enabled controls that should be read-only
- Negative control — would fail if: isReadOnly prop is not passed
- Evidence: screenshot (capture required: true)

### AC-4

- **GIVEN:** BranchGatesList has existing passing component tests
- **WHEN:** the wiring change lands
- **THEN:** the standalone BranchGatesList tests continue to pass with no regression
- **TDD State:** GREEN
- **Test Tier:** component
- **Verification Service:** playwright-ct
- **Flow Ref:** sprint-09-MGMT-REM-001-AC4

**Scenario:**

- Tier: regression
- Fixtures: branch_gates_list_harness
- Cases:
  - Actor: system, Steps: run BranchGatesList.spec.ts
  - Must observe: all 7 existing tests pass
  - Must not observe: new failures introduced by GovernanceSettings change
- Negative control — would fail if: BranchGatesList.svelte is modified
- Evidence: test_log (capture required: true)

## Test Criteria

| ID   | Statement                                                                           | Maps to AC |
| ---- | ----------------------------------------------------------------------------------- | ---------- |
| TC-1 | GovernanceSettings CT renders BranchGatesList when the Branch Gates tab is selected | AC-1       |
| TC-2 | GovernanceSettings CT triggers the refresh flow through the wired onRefresh prop    | AC-2       |
| TC-3 | GovernanceSettings CT passes read-only state and disables list mutation controls    | AC-3       |
| TC-4 | BranchGatesList.spec.ts still passes with all 7 tests after wiring                  | AC-4       |

## Guardrails

**WRITE-ALLOWED:**

- apps/desktop/src/components/governance/GovernanceSettings.svelte
- apps/desktop/tests/governance/GovernanceSettings.spec.ts

**WRITE-PROHIBITED:**

- apps/desktop/src/components/governance/BranchGatesList.svelte

## Verification Gates

- **Command:** `pnpm -F @gitbutler/desktop test:ct:desktop tests/governance/GovernanceSettings.spec.ts`
- **Expected outcome:** Branch Gates tab renders BranchGatesList; stub button is absent
- **Command:** `pnpm -F @gitbutler/desktop test:ct:desktop tests/governance/BranchGatesList.spec.ts`
- **Expected outcome:** 7 existing tests pass with no regression

## Reading List

- `apps/desktop/src/components/governance/GovernanceSettings.svelte` lines 229-241 — stub Branch Gates tab body to replace
- `apps/desktop/src/components/governance/BranchGatesList.svelte` lines all — already-implemented component and its required props
- `apps/desktop/tests/governance/BranchGatesList.spec.ts` lines all — existing harness for regression verification

## Dependencies

- **Depends On:** —
- **Blocks:** —

## Design

- **Pattern:** import + render with props
- **Anti-pattern:** duplicate BranchGatesList logic inline
- **References:** —

## Coding Standards

- apps/desktop/AGENTS.md

<!-- REQUIREMENT-CONTRACT v1 -->
<!-- {"requirements":[{"id":"AC-1","kind":"acceptance","scenario_ref":"AC-1"},{"id":"TC-1","kind":"test","maps_to_ac":"AC-1"},{"id":"AC-2","kind":"acceptance","scenario_ref":"AC-2"},{"id":"TC-2","kind":"test","maps_to_ac":"AC-2"},{"id":"AC-3","kind":"acceptance","scenario_ref":"AC-3"},{"id":"TC-3","kind":"test","maps_to_ac":"AC-3"},{"id":"AC-4","kind":"acceptance","scenario_ref":"AC-4"},{"id":"TC-4","kind":"test","maps_to_ac":"AC-4"}]} -->
