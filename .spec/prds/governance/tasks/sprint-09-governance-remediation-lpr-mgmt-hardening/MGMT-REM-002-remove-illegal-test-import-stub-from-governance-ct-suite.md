---
task: MGMT-REM-002
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

# MGMT-REM-002: Remove illegal test-import stub from governance CT suite

**Agent:** `sveltekit-implementer` (30 min)
**Proposed By:** `sveltekit-planner`
**Type:** REMEDIATION
**Status:** pending
**Depends On:** —
**Blocks:** —

## Background

**Problem (audit finding):** apps/desktop/tests/governance/PrincipalEditorInheritedReadOnly-PrincipalEditorLocalState-PrincipalEditorBatchSave-PrincipalEditorPreset-PrincipalEditorGroupChip-PrincipalEditorSelfEscalation.spec.ts contains only `import "./PrincipalEditor.spec";`

**Why it matters (impact):** Playwright refuses to run with cross-test-file imports, causing ERR_PNPM_RECURSIVE_RUN_FIRST_FAIL and crashing the entire CT suite.

**Affected files:**

- apps/desktop/tests/governance/PrincipalEditorInheritedReadOnly-PrincipalEditorLocalState-PrincipalEditorBatchSave-PrincipalEditorPreset-PrincipalEditorGroupChip-PrincipalEditorSelfEscalation.spec.ts
- apps/desktop/tests/governance/PrincipalEditor.spec.ts

## Critical Constraints

- MUST delete the illegal stub file
- MUST leave PrincipalEditor.spec.ts unchanged unless an independent defect is found
- MUST confirm the full governance CT suite loads after deletion
- NEVER rewrite the stub into another cross-import pattern

## Specification

**Objective:** Eliminate the illegal cross-test import that breaks Playwright CT discovery.

**Success state:** The stub file is gone, the suite loads without import errors, and PrincipalEditor coverage remains intact.

## Acceptance Criteria

### AC-1

- **GIVEN:** the illegal stub file exists in tests/governance/
- **WHEN:** the desktop governance CT suite starts
- **THEN:** Playwright discovers all specs without ERR_PNPM_RECURSIVE_RUN_FIRST_FAIL
- **TDD State:** RED
- **Test Tier:** component
- **Verification Service:** playwright-ct
- **Flow Ref:** sprint-09-MGMT-REM-002-AC1

**Scenario:**

- Tier: regression
- Fixtures: governance_ct_suite
- Cases:
  - Actor: system, Steps: run pnpm -F @gitbutler/desktop test:ct:desktop tests/governance/
  - Must observe: CT runner discovers specs; no import-another-test-file error
  - Must not observe: ERR_PNPM_RECURSIVE_RUN_FIRST_FAIL
- Negative control — would fail if: stub file is still present
- Evidence: test_log (capture required: true)

### AC-2

- **GIVEN:** the stub file is deleted
- **WHEN:** Playwright scans tests/governance/
- **THEN:** no spec file imports another spec file
- **TDD State:** RED
- **Test Tier:** component
- **Verification Service:** playwright-ct
- **Flow Ref:** sprint-09-MGMT-REM-002-AC2

**Scenario:**

- Tier: static
- Fixtures: governance_ct_sources
- Cases:
  - Actor: system, Steps: grep for cross-spec imports in tests/governance/\*.spec.ts
  - Must observe: zero spec-to-spec import statements
  - Must not observe: import "./\*.spec" patterns
- Negative control — would fail if: another cross-import is introduced
- Evidence: lint_output (capture required: true)

### AC-3

- **GIVEN:** PrincipalEditor.spec.ts remains the source of truth
- **WHEN:** it runs after the stub is removed
- **THEN:** it still covers inherited read-only, local state, batch save, preset, group chip, and self-escalation concerns
- **TDD State:** GREEN
- **Test Tier:** component
- **Verification Service:** playwright-ct
- **Flow Ref:** sprint-09-MGMT-REM-002-AC3

**Scenario:**

- Tier: regression
- Fixtures: principal_editor_harness
- Cases:
  - Actor: system, Steps: run PrincipalEditor.spec.ts
  - Must observe: all existing PrincipalEditor tests pass
  - Must not observe: lost coverage from deleted stub
- Negative control — would fail if: PrincipalEditor.spec.ts is accidentally deleted
- Evidence: test_log (capture required: true)

### AC-4

- **GIVEN:** the full desktop CT suite runs in CI
- **WHEN:** the illegal stub is absent
- **THEN:** the component-test job exits successfully
- **TDD State:** GREEN
- **Test Tier:** component
- **Verification Service:** playwright-ct
- **Flow Ref:** sprint-09-MGMT-REM-002-AC4

**Scenario:**

- Tier: regression
- Fixtures: ci_desktop_ct_job
- Cases:
  - Actor: system, Steps: trigger CI component-test job
  - Must observe: component-test job status is success
  - Must not observe: suite-level crash before tests run
- Negative control — would fail if: stub removal is reverted
- Evidence: ci_log (capture required: true)

## Test Criteria

| ID   | Statement                                                           | Maps to AC |
| ---- | ------------------------------------------------------------------- | ---------- |
| TC-1 | Governance CT suite loads after stub deletion without import errors | AC-1       |
| TC-2 | Static scan detects zero spec-to-spec imports in tests/governance/  | AC-2       |
| TC-3 | PrincipalEditor.spec.ts still passes and retains its coverage       | AC-3       |
| TC-4 | CI desktop component-test job exits green                           | AC-4       |

## Guardrails

**WRITE-ALLOWED:**

- apps/desktop/tests/governance/PrincipalEditorInheritedReadOnly-PrincipalEditorLocalState-PrincipalEditorBatchSave-PrincipalEditorPreset-PrincipalEditorGroupChip-PrincipalEditorSelfEscalation.spec.ts

**WRITE-PROHIBITED:**

- apps/desktop/tests/governance/PrincipalEditor.spec.ts

## Verification Gates

- **Command:** `pnpm -F @gitbutler/desktop test:ct:desktop tests/governance/`
- **Expected outcome:** CT suite discovers and runs all governance specs without import errors
- **Command:** `pnpm -F @gitbutler/desktop test:ct:desktop tests/governance/PrincipalEditor.spec.ts`
- **Expected outcome:** PrincipalEditor tests pass and retain existing coverage

## Reading List

- `apps/desktop/tests/governance/PrincipalEditorInheritedReadOnly-PrincipalEditorLocalState-PrincipalEditorBatchSave-PrincipalEditorPreset-PrincipalEditorGroupChip-PrincipalEditorSelfEscalation.spec.ts` lines all — illegal one-line import to delete
- `apps/desktop/tests/governance/PrincipalEditor.spec.ts` lines all — canonical test coverage to preserve

## Dependencies

- **Depends On:** —
- **Blocks:** —

## Design

- **Pattern:** delete invalid artifact
- **Anti-pattern:** re-export another spec from a new file
- **References:** —

## Coding Standards

- apps/desktop/AGENTS.md

<!-- REQUIREMENT-CONTRACT v1 -->
<!-- {"requirements":[{"id":"AC-1","kind":"acceptance","scenario_ref":"AC-1"},{"id":"TC-1","kind":"test","maps_to_ac":"AC-1"},{"id":"AC-2","kind":"acceptance","scenario_ref":"AC-2"},{"id":"TC-2","kind":"test","maps_to_ac":"AC-2"},{"id":"AC-3","kind":"acceptance","scenario_ref":"AC-3"},{"id":"TC-3","kind":"test","maps_to_ac":"AC-3"},{"id":"AC-4","kind":"acceptance","scenario_ref":"AC-4"},{"id":"TC-4","kind":"test","maps_to_ac":"AC-4"}]} -->
