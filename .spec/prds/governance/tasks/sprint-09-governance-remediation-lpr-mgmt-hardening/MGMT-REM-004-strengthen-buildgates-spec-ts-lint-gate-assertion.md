---
task: MGMT-REM-004
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

# MGMT-REM-004: Strengthen BuildGates.spec.ts:204 lint-gate assertion

**Agent:** `sveltekit-implementer` (30 min)
**Proposed By:** `sveltekit-planner`
**Type:** REMEDIATION
**Status:** pending
**Depends On:** —
**Blocks:** —

## Background

**Problem (audit finding):** apps/desktop/tests/governance/BuildGates.spec.ts:204 'passes the repository lint gate' only asserts that output contains the string 'lint'.

**Why it matters (impact):** A failing lint run that still mentions the word 'lint' in output makes the test pass, masking quality gate failures.

**Affected files:**

- apps/desktop/tests/governance/BuildGates.spec.ts

## Critical Constraints

- MUST assert the lint command exited with code 0
- MUST assert the output indicates actual success or the absence of error lines
- MUST keep the test focused on the lint gate behavior
- NEVER weaken the assertion to string-only matching

## Specification

**Objective:** Make the BuildGates lint-gate test fail when lint fails.

**Success state:** The test fails on non-zero exit and failing lint output, and passes only when lint succeeds.

## Acceptance Criteria

### AC-1

- **GIVEN:** the BuildGates lint-gate test invokes the repository lint command
- **WHEN:** the command completes
- **THEN:** the test asserts the returned exit code is 0
- **TDD State:** RED
- **Test Tier:** component
- **Verification Service:** playwright-ct
- **Flow Ref:** sprint-09-MGMT-REM-004-AC1

**Scenario:**

- Tier: behavior
- Fixtures: build_gates_lint_harness
- Cases:
  - Actor: system, Steps: execute lint command through test harness; assert exitCode === 0
  - Must observe: test passes when lint exits 0
  - Must not observe: string-only 'lint' substring assertion
- Negative control — would fail if: assertion ignores exit code
- Evidence: test_log (capture required: true)

### AC-2

- **GIVEN:** the lint command exits with code 0
- **WHEN:** the test inspects the command output
- **THEN:** the output contains a success marker such as 'All matched files use GingerByte code style' or contains no error lines
- **TDD State:** RED
- **Test Tier:** component
- **Verification Service:** playwright-ct
- **Flow Ref:** sprint-09-MGMT-REM-004-AC2

**Scenario:**

- Tier: behavior
- Fixtures: build_gates_lint_harness
- Cases:
  - Actor: system, Steps: run successful lint command; check output for success marker or empty error set
  - Must observe: output confirms success or no errors
  - Must not observe: spurious 'lint' word match passing the test
- Negative control — would fail if: success marker is absent
- Evidence: test_log (capture required: true)

### AC-3

- **GIVEN:** the lint command exits with a non-zero code
- **WHEN:** the strengthened test runs
- **THEN:** the test fails even if output contains the word 'lint'
- **TDD State:** RED
- **Test Tier:** component
- **Verification Service:** playwright-ct
- **Flow Ref:** sprint-09-MGMT-REM-004-AC3

**Scenario:**

- Tier: negative
- Fixtures: build_gates_lint_harness_failing
- Cases:
  - Actor: system, Steps: mock or seed a failing lint run; run the updated test
  - Must observe: test fails due to exit code check
  - Must not observe: test passes on substring match
- Negative control — would fail if: old assertion is still present
- Evidence: test_log (capture required: true)

### AC-4

- **GIVEN:** the updated test exists
- **WHEN:** it is added to a temporary seeded implementation that produces failing lint output mentioning 'lint'
- **THEN:** the test correctly fails before the real fix and passes after the real fix
- **TDD State:** RED
- **Test Tier:** component
- **Verification Service:** playwright-ct
- **Flow Ref:** sprint-09-MGMT-REM-004-AC4

**Scenario:**

- Tier: fakeability
- Fixtures: build_gates_lint_harness, build_gates_lint_harness_failing
- Cases:
  - Actor: system, Steps: seed failing lint output with 'lint' word; expect test to fail
  - Must observe: test fails on failing seeded output
  - Must not observe: false positive pass
- Negative control — would fail if: assertion only checks substring
- Evidence: test_log (capture required: true)

## Test Criteria

| ID   | Statement                                                                          | Maps to AC |
| ---- | ---------------------------------------------------------------------------------- | ---------- |
| TC-1 | BuildGates lint-gate test asserts exit code 0                                      | AC-1       |
| TC-2 | BuildGates lint-gate test asserts success marker or absence of errors in output    | AC-2       |
| TC-3 | BuildGates lint-gate test fails when lint exits non-zero                           | AC-3       |
| TC-4 | BuildGates lint-gate test fails against seeded failing output that contains 'lint' | AC-4       |

## Guardrails

**WRITE-ALLOWED:**

- apps/desktop/tests/governance/BuildGates.spec.ts

**WRITE-PROHIBITED:**

- apps/desktop/src/components/governance/BuildGates.svelte

## Verification Gates

- **Command:** `pnpm -F @gitbutler/desktop test:ct:desktop tests/governance/BuildGates.spec.ts`
- **Expected outcome:** lint-gate test passes and asserts exit code + success marker

## Reading List

- `apps/desktop/tests/governance/BuildGates.spec.ts` lines 190-220 — weak lint-gate assertion to strengthen

## Dependencies

- **Depends On:** —
- **Blocks:** —

## Design

- **Pattern:** assert exit code plus success semantics
- **Anti-pattern:** assert output contains keyword only
- **References:** —

## Coding Standards

- apps/desktop/AGENTS.md

<!-- REQUIREMENT-CONTRACT v1 -->
<!-- {"requirements":[{"id":"AC-1","kind":"acceptance","scenario_ref":"AC-1"},{"id":"TC-1","kind":"test","maps_to_ac":"AC-1"},{"id":"AC-2","kind":"acceptance","scenario_ref":"AC-2"},{"id":"TC-2","kind":"test","maps_to_ac":"AC-2"},{"id":"AC-3","kind":"acceptance","scenario_ref":"AC-3"},{"id":"TC-3","kind":"test","maps_to_ac":"AC-3"},{"id":"AC-4","kind":"acceptance","scenario_ref":"AC-4"},{"id":"TC-4","kind":"test","maps_to_ac":"AC-4"}]} -->
