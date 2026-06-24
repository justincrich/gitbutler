---
task: LPR-REM-007-UI
sprint: sprint-09-governance-remediation-lpr-mgmt-hardening
sequence: 11
agent: sveltekit-implementer
estimate_minutes: 60
status: pending
proposed_by: sveltekit-planner
type: REMEDIATION
generated_at: 2026-06-23T13:30:00Z
generated_by: kb-sprint-tasks-plan
---

# LPR-REM-007-UI: Mount LocalReviewView.svelte and consume kind (UI half)

**Agent:** `sveltekit-implementer` (60 min)
**Proposed By:** `sveltekit-planner`
**Type:** REMEDIATION
**Status:** pending
**Depends On:** LPR-REM-007, LPR-REM-005
**Blocks:** —

## Background

**Problem (audit finding):** apps/desktop/src/components/governance/LocalReviewView.svelte exists but is never mounted. Its LocalReviewStatusPayload type requires approved plus optional source_branch, sha, author, title, created_at fields that are missing from the backend ReviewStatus struct.

**Why it matters (impact):** Local review status is unreachable in the UI until the component is mounted and the contract fields are available from the Rust backend.

**Affected files:**

- apps/desktop/src/components/governance/LocalReviewView.svelte
- apps/desktop/src/components/governance/GovernanceSettings.svelte

## Critical Constraints

- MUST wait for Rust tasks LPR-REM-007 and LPR-REM-005 to land before implementation
- MUST mount LocalReviewView in GovernanceSettings and pass the kind prop
- MUST consume the enriched LocalReviewStatusPayload fields exposed by the backend
- MUST keep the component resilient when optional fields are absent

## Specification

**Objective:** Expose LocalReviewView inside GovernanceSettings using the new kind + enriched status contract.

**Success state:** The Local Review section/tab renders the component with kind and full payload, handling missing optional data gracefully.

## Acceptance Criteria

### AC-1

- **GIVEN:** GovernanceSettings renders after the Rust backend exposes kind and enriched ReviewStatus fields
- **WHEN:** the user opens the Local Review section/tab
- **THEN:** LocalReviewView mounts with the kind prop and a payload containing approved, source_branch, sha, author, title, and created_at
- **TDD State:** RED
- **Test Tier:** component
- **Verification Service:** playwright-ct
- **Flow Ref:** sprint-09-LPR-REM-007-UI-AC1

**Scenario:**

- Tier: visible
- Fixtures: seeded_governance_settings_with_local_review
- Cases:
  - Actor: admin, Steps: render GovernanceSettings; open Local Review section
  - Must observe: LocalReviewView is mounted; kind prop is consumed; payload fields are bound
  - Must not observe: missing LocalReviewView section
- Negative control — would fail if: LocalReviewView not imported; kind not passed
- Evidence: screenshot (capture required: true)

### AC-2

- **GIVEN:** LocalReviewView receives kind='required' and approved=false
- **WHEN:** the component renders
- **THEN:** it displays a review-required state and prompts the user appropriately
- **TDD State:** RED
- **Test Tier:** component
- **Verification Service:** playwright-ct
- **Flow Ref:** sprint-09-LPR-REM-007-UI-AC2

**Scenario:**

- Tier: visible
- Fixtures: seeded_local_review_required
- Cases:
  - Actor: admin, Steps: render LocalReviewView with kind=required and approved=false
  - Must observe: review required indicator shown
  - Must not observe: approved summary state
- Negative control — would fail if: kind value is ignored
- Evidence: screenshot (capture required: true)

### AC-3

- **GIVEN:** LocalReviewView receives kind='summary' and approved=true with source_branch and sha
- **WHEN:** the component renders
- **THEN:** it shows the source branch, commit sha, author, title, and creation time
- **TDD State:** RED
- **Test Tier:** component
- **Verification Service:** playwright-ct
- **Flow Ref:** sprint-09-LPR-REM-007-UI-AC3

**Scenario:**

- Tier: visible
- Fixtures: seeded_local_review_summary
- Cases:
  - Actor: admin, Steps: render LocalReviewView with kind=summary and full payload
  - Must observe: source branch displayed; short sha displayed; author displayed; title displayed; created_at displayed
  - Must not observe: review required message
- Negative control — would fail if: optional payload fields are not consumed
- Evidence: screenshot (capture required: true)

### AC-4

- **GIVEN:** the backend returns a ReviewStatus missing some optional payload fields
- **WHEN:** LocalReviewView mounts
- **THEN:** it renders without error and omits the missing fields gracefully
- **TDD State:** RED
- **Test Tier:** component
- **Verification Service:** playwright-ct
- **Flow Ref:** sprint-09-LPR-REM-007-UI-AC4

**Scenario:**

- Tier: negative
- Fixtures: seeded_local_review_partial
- Cases:
  - Actor: system, Steps: render LocalReviewView with only approved and kind
  - Must observe: component renders without runtime error; available fields shown
  - Must not observe: undefined placeholders; uncaught exception
- Negative control — would fail if: component hard-codes required optional fields
- Evidence: screenshot (capture required: true)

## Test Criteria

| ID   | Statement                                                                               | Maps to AC |
| ---- | --------------------------------------------------------------------------------------- | ---------- |
| TC-1 | GovernanceSettings CT mounts LocalReviewView with kind and enriched payload             | AC-1       |
| TC-2 | LocalReviewView CT shows required state when kind=required and approved=false           | AC-2       |
| TC-3 | LocalReviewView CT displays branch, sha, author, title, and created_at in summary state | AC-3       |
| TC-4 | LocalReviewView CT renders gracefully when optional payload fields are absent           | AC-4       |

## Guardrails

**WRITE-ALLOWED:**

- apps/desktop/src/components/governance/GovernanceSettings.svelte
- apps/desktop/src/components/governance/LocalReviewView.svelte
- apps/desktop/tests/governance/LocalReviewView.spec.ts
- apps/desktop/tests/governance/GovernanceSettings.spec.ts

**WRITE-PROHIBITED:**

- apps/desktop/src/components/governance/BranchGatesList.svelte

## Verification Gates

- **Command:** `pnpm -F @gitbutler/desktop test:ct:desktop tests/governance/LocalReviewView.spec.ts`
- **Expected outcome:** LocalReviewView renders all kind/payload states
- **Command:** `pnpm -F @gitbutler/desktop test:ct:desktop tests/governance/GovernanceSettings.spec.ts`
- **Expected outcome:** Local Review section mounts LocalReviewView

## Reading List

- `apps/desktop/src/components/governance/LocalReviewView.svelte` lines 27-38 — LocalReviewStatusPayload contract
- `apps/desktop/src/components/governance/GovernanceSettings.svelte` lines all — location to mount LocalReviewView

## Dependencies

- **Depends On:** LPR-REM-007, LPR-REM-005
- **Blocks:** —

## Design

- **Pattern:** mount child component and pass kind + payload props
- **Anti-pattern:** use module-level stores for shared local review state
- **References:** —

## Coding Standards

- apps/desktop/AGENTS.md

<!-- REQUIREMENT-CONTRACT v1 -->
<!-- {"requirements":[{"id":"AC-1","kind":"acceptance","scenario_ref":"AC-1"},{"id":"TC-1","kind":"test","maps_to_ac":"AC-1"},{"id":"AC-2","kind":"acceptance","scenario_ref":"AC-2"},{"id":"TC-2","kind":"test","maps_to_ac":"AC-2"},{"id":"AC-3","kind":"acceptance","scenario_ref":"AC-3"},{"id":"TC-3","kind":"test","maps_to_ac":"AC-3"},{"id":"AC-4","kind":"acceptance","scenario_ref":"AC-4"},{"id":"TC-4","kind":"test","maps_to_ac":"AC-4"}]} -->
