# E2E-GOV-004: Governance fixture E2E pending persistence and commit clearing

## What this does

Adds fixture-backed Playwright coverage for the tail of the human gate: pending principal and group changes
survive tab navigation, then Commit changes clears pending indicators while grants remain visible.

## Why

The Sprint 06a gate is not complete until both pending edits persist across tabs and clear only after the
commit action.

## How to verify

```bash
pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep "clear after commit"
pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture:headed --grep "clear after commit"
```

## Scope

- `e2e/playwright/tests/governance-fixture/governance-fixture.spec.ts`
- `e2e/playwright/fixtures/governance-app/**`

<details>
<summary>Full agent specification</summary>

```
TASK: E2E-GOV-004 - Governance fixture E2E pending persistence and commit clearing
TASK_TYPE: FEATURE
STATUS: Backlog
PRIORITY: P1
EFFORT: S
AGENT: implementer=electron-implementer | reviewer=electron-reviewer
PROPOSED-BY: electron-planner
DESIGN-EVIDENCE-SOURCE: frontend-designer
SPRINT: ./SPRINT.md
FLOWS: GOV-E2E-07, GOV-E2E-08

OUTCOME
The fixture records cross-tab-pending.fixture-evidence and post-commit-clean.fixture-evidence.

CRITICAL CONSTRAINTS
- MUST preserve pending state across all four tab switches before commit.
- MUST clear all pending markers only after Commit changes.
- MUST leave edited grants visible after the fixture commit action.
- MUST NOT claim committed governance ref or backend persistence proof.

DONE WHEN
- AC-1: pending banner remains visible across Principals, Groups, Branch Gates, and Rules.
- AC-2: Commit changes clears the pending banner and row-level pending markers.
- AC-3: edited reviews:write grants remain visible after the fixture commit result.

ACCEPTANCE CRITERIA
AC-1 [PRIMARY]: Cross-tab pending persistence
  GIVEN: admin has one pending principal change and one pending group change in the fixture
  WHEN: the user switches through Principals, Groups, Branch Gates, Rules, then back to Principals and Groups
  THEN: the Commit changes banner remains visible and both pending markers are still present on return
  TEST_TIER: e2e
  VERIFY: pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep "clear after commit"

AC-2: Commit clears pending markers
  GIVEN: the admin has pending governance changes
  WHEN: the user clicks Commit changes
  THEN: pending banner, principal pending marker, and group pending marker clear
  TEST_TIER: e2e
  VERIFY: pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep "clear after commit"

AC-3: Grants remain visible after fixture commit
  GIVEN: the commit action has completed in the fixture
  WHEN: the user revisits Principals and Groups
  THEN: the edited grants remain visible as committed/effective fixture state
  TEST_TIER: e2e
  VERIFY: pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep "clear after commit"

TEST CRITERIA
- TC-1 (AC-1): Pending banner and markers survive tab navigation across all four governance tabs.
- TC-2 (AC-2): Commit changes clears all pending UI indicators.
- TC-3 (AC-3): Committed fixture grants remain visible as effective state after pending clears.
- TC-4 (AC-1): Fixture evidence is labeled and not used as the sole sprint-close proof.

SCOPE
writeAllowed:
- .spec/prds/governance/tasks/sprint-06a-governance-ui-scaffold-principals-groups/E2E-GOV-004-cross-tab-commit.md
- e2e/playwright/tests/governance-fixture/governance-fixture.spec.ts
- e2e/playwright/fixtures/governance-app/**

writeProhibited:
- apps/desktop/src/**
- apps/desktop/tests/governance/**
- crates/**
- packages/but-sdk/src/generated/**

DEPENDENCIES
Depends on: E2E-GOV-003
Blocks: none
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "E2E-GOV-004",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "governance_fixture_two_pending_changes": {
      "description": "Vite governance fixture after a principal own-grant and group grant are staged by UI interactions.",
      "seed_method": "ui_flow",
      "records": ["principal `settings-agent` staged via visible `reviews:write` own-grant checkbox", "group `eng` staged via visible `reviews:write` grant checkbox", "commit banner `2 pending changes`"]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN two pending fixture changes WHEN tabs are switched through all governance tabs THEN banner and markers persist.",
      "verify": "pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep \"clear after commit\"",
      "maps_to_ac": null,
      "scenario": {
        "test_tier": "e2e",
        "tier": "visible",
        "verification_service": "vite-governance-fixture",
        "negative_control": { "would_fail_if": ["per-tab local pending state not persisted", "banner resets on navigation", "static shell omits shared pending store"] },
        "evidence": { "artifact_type": "video", "required_capture": true },
        "cases": [
          {
            "start_ref": "governance_fixture_two_pending_changes",
            "action": { "actor": "playwright_user", "steps": ["visit Branch Gates", "visit Rules", "return to Principals", "return to Groups"] },
            "end_state": { "must_observe": ["commit banner `2 pending changes`", "test id `principals-list-pending-settings-agent` count 1", "test id `groups-list-pending-eng` count 1"], "must_not_observe": ["commit banner `0 pending changes`", "empty pending store"] }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "GIVEN pending changes WHEN Commit changes is clicked THEN all pending indicators clear.",
      "verify": "pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep \"clear after commit\"",
      "maps_to_ac": null
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "GIVEN fixture commit completed WHEN Principals and Groups are revisited THEN edited grants remain visible.",
      "verify": "pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep \"clear after commit\"",
      "maps_to_ac": null
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "Pending banner and markers survive tab navigation across all four governance tabs.",
      "verify": "pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep \"clear after commit\"",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "Commit changes clears all pending UI indicators.",
      "verify": "pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep \"clear after commit\"",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "Committed fixture grants remain visible as effective state after pending clears.",
      "verify": "pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep \"clear after commit\"",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "Fixture evidence is labeled and not used as the sole sprint-close proof.",
      "verify": "rg -n \"Fixture governance harness|Not product E2E evidence|fixture evidence\" e2e/playwright/tests/governance-fixture e2e/playwright/fixtures/governance-app",
      "maps_to_ac": "AC-1"
    }
  ]
}
-->
</details>
