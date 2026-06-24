---
task: MGMT-REM-005
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

# MGMT-REM-005: Align root test:ct command with desktop CT documentation

**Agent:** `sveltekit-implementer` (30 min)
**Proposed By:** `sveltekit-planner`
**Type:** REMEDIATION
**Status:** pending
**Depends On:** —
**Blocks:** —

## Background

**Problem (audit finding):** Root package.json 'test:ct' runs only turbo --filter @gitbutler/ui test:ct, while the spec and T-MGMT-000 evidence cite pnpm test:ct for desktop governance CT.

**Why it matters (impact):** Running 'pnpm test:ct' at root silently skips desktop component tests, creating false confidence and diverging from documented verification.

**Affected files:**

- package.json
- apps/desktop/package.json

## Critical Constraints

- MUST update root test:ct so it runs both @gitbutler/ui and @gitbutler/desktop CT suites
- MUST preserve the ability to run a single package's CT suite with -F
- MUST not break existing @gitbutler/ui CT behavior
- NEVER redefine the desktop harness script; reuse the existing test:ct:desktop script

## Specification

**Objective:** Make pnpm test:ct run all component test suites including desktop.

**Success state:** Root test:ct invokes both UI and desktop CT suites; desktop governance tests execute under the correct command.

## Acceptance Criteria

### AC-1

- **GIVEN:** the root package.json test:ct script is updated
- **WHEN:** a developer runs pnpm test:ct from the project root
- **THEN:** the command executes both @gitbutler/ui and @gitbutler/desktop component test suites
- **TDD State:** RED
- **Test Tier:** component
- **Verification Service:** pnpm-turbo
- **Flow Ref:** sprint-09-MGMT-REM-005-AC1

**Scenario:**

- Tier: workflow
- Fixtures: root_package_json, desktop_package_json
- Cases:
  - Actor: developer, Steps: run pnpm test:ct at repo root
  - Must observe: @gitbutler/ui CT runs; @gitbutler/desktop CT runs
  - Must not observe: only @gitbutler/ui runs
- Negative control — would fail if: root script still filters to @gitbutler/ui
- Evidence: command_output (capture required: true)

### AC-2

- **GIVEN:** the new root test:ct runs both suites
- **WHEN:** the desktop suite completes
- **THEN:** the correct harness command pnpm -F @gitbutler/desktop test:ct:desktop is used for desktop tests
- **TDD State:** RED
- **Test Tier:** component
- **Verification Service:** pnpm-turbo
- **Flow Ref:** sprint-09-MGMT-REM-005-AC2

**Scenario:**

- Tier: workflow
- Fixtures: desktop_package_json
- Cases:
  - Actor: system, Steps: invoke pnpm test:ct
  - Must observe: turbo executes @gitbutler/desktop#test:ct:desktop
  - Must not observe: desktop tests skipped or wrong script invoked
- Negative control — would fail if: root script calls a non-existent desktop script
- Evidence: command_output (capture required: true)

### AC-3

- **GIVEN:** the root test:ct command is aligned
- **WHEN:** @gitbutler/ui CT suite runs
- **THEN:** it continues to pass and produce the same output as before the change
- **TDD State:** GREEN
- **Test Tier:** component
- **Verification Service:** pnpm-turbo
- **Flow Ref:** sprint-09-MGMT-REM-005-AC3

**Scenario:**

- Tier: regression
- Fixtures: ui_component_tests
- Cases:
  - Actor: system, Steps: run pnpm test:ct
  - Must observe: all UI component tests pass
  - Must not observe: degraded UI CT behavior
- Negative control — would fail if: UI task graph is broken
- Evidence: test_log (capture required: true)

### AC-4

- **GIVEN:** the root command now runs desktop CT
- **WHEN:** desktop governance tests execute through it
- **THEN:** the governance specs in apps/desktop/tests/governance/ run under the desktop Playwright harness
- **TDD State:** RED
- **Test Tier:** component
- **Verification Service:** playwright-ct
- **Flow Ref:** sprint-09-MGMT-REM-005-AC4

**Scenario:**

- Tier: workflow
- Fixtures: governance_ct_suite
- Cases:
  - Actor: system, Steps: run pnpm test:ct
  - Must observe: desktop governance specs discovered and executed
  - Must not observe: specs silently omitted
- Negative control — would fail if: desktop CT still requires a separate manual invocation
- Evidence: test_log (capture required: true)

## Test Criteria

| ID   | Statement                                                          | Maps to AC |
| ---- | ------------------------------------------------------------------ | ---------- |
| TC-1 | pnpm test:ct runs both @gitbutler/ui and @gitbutler/desktop suites | AC-1       |
| TC-2 | Root command uses the existing desktop test:ct:desktop script      | AC-2       |
| TC-3 | UI component tests remain green after alignment                    | AC-3       |
| TC-4 | Governance desktop specs execute via the aligned root command      | AC-4       |

## Guardrails

**WRITE-ALLOWED:**

- package.json
- apps/desktop/package.json

**WRITE-PROHIBITED:**

- apps/desktop/src/components/governance/BranchGatesList.svelte
- apps/desktop/src/components/governance/LocalReviewView.svelte

## Verification Gates

- **Command:** `pnpm test:ct`
- **Expected outcome:** Both @gitbutler/ui and @gitbutler/desktop CT suites execute
- **Command:** `pnpm -F @gitbutler/desktop test:ct:desktop tests/governance/`
- **Expected outcome:** Desktop governance specs still pass

## Reading List

- `package.json` lines all — root test:ct script to align
- `apps/desktop/package.json` lines all — existing test:ct:desktop script to reuse

## Dependencies

- **Depends On:** —
- **Blocks:** —

## Design

- **Pattern:** update root turbo pipeline to include desktop CT
- **Anti-pattern:** rename or duplicate desktop harness scripts
- **References:** —

## Coding Standards

- apps/desktop/AGENTS.md

<!-- REQUIREMENT-CONTRACT v1 -->
<!-- {"requirements":[{"id":"AC-1","kind":"acceptance","scenario_ref":"AC-1"},{"id":"TC-1","kind":"test","maps_to_ac":"AC-1"},{"id":"AC-2","kind":"acceptance","scenario_ref":"AC-2"},{"id":"TC-2","kind":"test","maps_to_ac":"AC-2"},{"id":"AC-3","kind":"acceptance","scenario_ref":"AC-3"},{"id":"TC-3","kind":"test","maps_to_ac":"AC-3"},{"id":"AC-4","kind":"acceptance","scenario_ref":"AC-4"},{"id":"TC-4","kind":"test","maps_to_ac":"AC-4"}]} -->
