# E2E-GOV-104: Desktop E2E pending persists across tabs and clears after commit

## What this does

Adds the real desktop-product E2E closure task for the human gate: pending principal and group changes
survive all governance tab switches, then clear after Commit changes while the edited grants remain visible.

## Why

This is the sprint gate end state from the product perspective. The fixture can be used as a watchable
preflight, but this task owns the real desktop closeout proof.

## How to verify

```bash
PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-commit-flow.spec.ts
pnpm --filter @gitbutler/desktop test:ct:desktop -- GovernanceSettingsCrossTabPending
pnpm --filter @gitbutler/desktop test:ct:desktop -- GovernanceSettingsCommit
pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture:headed --grep "clear after commit"
```

## Scope

- `e2e/playwright/tests/governance-settings-commit-flow.spec.ts`
- `e2e/playwright/src/governance.ts`
- `e2e/playwright/scripts/governance-*`
- `e2e/playwright/tests/governance-fixture/governance-fixture.spec.ts`

<details>
<summary>Full agent specification</summary>

```
TASK: E2E-GOV-104 - Desktop E2E pending persists across tabs and clears after commit
TASK_TYPE: FEATURE
STATUS: Done
STATUS_NOTE: Realized by e2e/playwright/tests/governance-settings-commit-flow.spec.ts (real desktop + but-server e2e, merged on master). Re-verify: PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-commit-flow.spec.ts
PRIORITY: P0
EFFORT: M
AGENT: implementer=sveltekit-implementer | reviewer=sveltekit-reviewer
PROPOSED-BY: sveltekit-planner
DESIGN-EVIDENCE-SOURCE: frontend-designer
SPRINT: ./SPRINT.md
FLOWS: GOV-E2E-07, GOV-E2E-08

OUTCOME
The desktop product E2E proves cross-tab-pending and post-commit-clean behavior for the Sprint 06a gate.

CRITICAL CONSTRAINTS
- MUST start from pending principal and group changes created through visible product controls.
- MUST navigate all four tabs before commit.
- MUST click Commit changes through the product UI.
- MUST keep fixture evidence supplemental and clearly labeled, never the sole product proof.

DONE WHEN
- AC-1: pending banner and row markers persist across Principals, Groups, Branch Gates, Rules, and return navigation.
- AC-2: Commit changes clears pending banner and all row-level pending indicators.
- AC-3: fixture preflight evidence is labeled fixture evidence and Not product E2E evidence.

ACCEPTANCE CRITERIA
AC-1 [PRIMARY]: Cross-tab pending persistence
  GIVEN: admin desktop session has one pending principal own-grant and one pending group grant
  WHEN: user switches through Principals, Groups, Branch Gates, Rules, then back to Principals and Groups
  THEN: Commit changes banner remains visible and both pending markers remain visible on return
  TEST_TIER: e2e
  VERIFY: PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-commit-flow.spec.ts

AC-2: Commit clears pending and keeps grants
  GIVEN: the same desktop session still has pending governance changes
  WHEN: user clicks Commit changes
  THEN: commit completes, pending banner disappears, markers clear, and changed grants remain visible
  TEST_TIER: e2e
  VERIFY: PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-commit-flow.spec.ts

AC-3: Fixture preflight remains labeled
  GIVEN: commit flow is verified in the fixture harness as a faster visual preflight
  WHEN: the fixture Playwright command is run
  THEN: evidence is explicitly labeled fixture evidence and Not product E2E evidence
  TEST_TIER: e2e
  VERIFY: pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture:headed --grep "clear after commit"

TEST CRITERIA
- TC-1 (AC-1): Pending banner and both row-level pending indicators persist across all four tabs.
- TC-2 (AC-2): Clicking Commit changes clears all pending UI state and leaves edited grants visible.
- TC-3 (AC-3): Any governance fixture run is clearly labeled as fixture evidence, not product-backend proof.

SCOPE
writeAllowed:
- .spec/prds/governance/tasks/sprint-06a-governance-ui-scaffold-principals-groups/E2E-GOV-104-desktop-cross-tab-commit.md
- e2e/playwright/tests/governance-settings-commit-flow.spec.ts
- e2e/playwright/src/governance.ts
- e2e/playwright/scripts/governance-*
- e2e/playwright/tests/governance-fixture/governance-fixture.spec.ts

writeProhibited:
- apps/desktop/src/**
- apps/desktop/tests/governance/**
- packages/but-sdk/src/generated/**
- crates/**

DEPENDENCIES
Depends on: E2E-GOV-103, MGMT-UI-003, MGMT-UI-005, MGMT-IPC-005
Blocks: none
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "E2E-GOV-104",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "desktop_two_pending_governance_changes": {
      "description": "Real desktop E2E session where visible product UI interactions have staged one principal pending change and one group pending change.",
      "seed_method": "ui_flow",
      "records": ["principal row:test-principal pending_marker:true", "group row:test-group pending_marker:true", "Commit changes banner pending_count:2"]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN two pending desktop governance changes WHEN all four tabs are visited THEN banner and pending markers persist.",
      "verify": "PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-commit-flow.spec.ts",
      "maps_to_ac": null,
      "scenario": {
        "id": "E2E-GOV-104-AC-1",
        "test_tier": "e2e",
        "tier": "visible",
        "verification_service": "desktop-product-e2e",
        "negative_control": { "would_fail_if": ["per-tab pending store drops state", "direct SDK call bypasses product UI", "empty pending state"] },
        "evidence": { "artifact_type": "video", "required_capture": true },
        "cases": [
          {
            "start_ref": "desktop_two_pending_governance_changes",
            "action": { "actor": "playwright_user", "steps": ["visit Principals", "visit Groups", "visit Branch Gates", "visit Rules", "return to Principals", "return to Groups"] },
            "end_state": {
              "must_observe": ["Commit changes banner pending count == 2", "principal row \"test-principal\" pending marker count == 1", "group row \"test-group\" pending marker count == 1", "visited governance tab count == 4"],
              "must_not_observe": ["pending badge count 0", "empty pending state", "Commit changes banner pending count == 0"]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "GIVEN pending desktop changes WHEN Commit changes is clicked THEN pending clears and edited grants remain visible.",
      "verify": "PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-commit-flow.spec.ts",
      "maps_to_ac": null,
      "scenario": {
        "id": "E2E-GOV-104-AC-2",
        "test_tier": "e2e",
        "tier": "visible",
        "verification_service": "desktop-product-e2e",
        "negative_control": { "would_fail_if": ["commit button no-op stub leaves pending state", "mock commit clears rows without persisted grants", "static clean tree signature"] },
        "evidence": { "artifact_type": "video", "required_capture": true },
        "cases": [
          {
            "start_ref": "desktop_two_pending_governance_changes",
            "action": { "actor": "playwright_user", "steps": ["click Commit changes", "wait for commit completion", "reopen Principals and Groups"] },
            "end_state": {
              "must_observe": ["changed principal grant \"test-principal\" count == 1", "changed group grant \"test-group\" count == 1", "clean-tree signature pending count == 0"],
              "must_not_observe": ["changed principal grant \"test-principal\" count == 0", "principal pending marker count == 1", "group pending marker count == 1"]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "GIVEN fixture preflight is run WHEN evidence is captured THEN it is labeled fixture evidence and Not product E2E evidence.",
      "verify": "pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture:headed --grep \"clear after commit\"",
      "maps_to_ac": null,
      "scenario": {
        "id": "E2E-GOV-104-AC-3",
        "test_tier": "e2e",
        "tier": "visible",
        "verification_service": "fixture-preflight-labeling",
        "negative_control": { "would_fail_if": ["fixture evidence mislabeled as product proof", "static screenshot omits warning label", "empty fixture report"] },
        "evidence": { "artifact_type": "file_artifact", "required_capture": true },
        "cases": [
          {
            "start_ref": "desktop_two_pending_governance_changes",
            "action": { "actor": "playwright_user", "steps": ["run governance fixture clear-after-commit flow", "capture fixture evidence label"] },
            "end_state": {
              "must_observe": ["label \"Fixture governance harness\" count >= 1", "label \"Not product E2E evidence\" count >= 1", "fixture evidence file count >= 1"],
              "must_not_observe": ["fixture evidence file count == 0", "empty fixture report", "product proof label without \"Not product E2E evidence\""]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "Pending banner and both row-level pending indicators persist across all four tabs in the real desktop settings page.",
      "verify": "PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-commit-flow.spec.ts",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "Clicking Commit changes clears all pending UI state and leaves edited grants visible.",
      "verify": "PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-commit-flow.spec.ts",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "Any governance fixture run is clearly labeled as fixture evidence, not product-backend proof.",
      "verify": "rg -n \"Fixture governance harness|Not product E2E evidence|fixture evidence\" e2e/playwright/tests/governance-fixture e2e/playwright/fixtures/governance-app",
      "maps_to_ac": "AC-3"
    }
  ]
}
-->
</details>
