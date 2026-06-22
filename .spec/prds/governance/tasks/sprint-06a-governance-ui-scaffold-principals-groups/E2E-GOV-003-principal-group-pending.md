# E2E-GOV-003: Governance fixture E2E principal and group pending edits

## What this does

Adds fixture-backed Playwright coverage for staging an admin principal own-grant and group grant, proving
pending markers and the shared commit banner appear through visible UI interactions.

## Why

The human gate requires the principal and group edits to remain pending together before commit. This task
turns those manual observations into a headed fixture preflight.

## How to verify

```bash
pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep "principal and group changes"
pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture:headed --grep "principal and group changes"
```

## Scope

- `e2e/playwright/tests/governance-fixture/governance-fixture.spec.ts`
- `e2e/playwright/fixtures/governance-app/**`

<details>
<summary>Full agent specification</summary>

```
TASK: E2E-GOV-003 - Governance fixture E2E principal and group pending edits
TASK_TYPE: FEATURE
STATUS: Backlog
PRIORITY: P1
EFFORT: S
AGENT: implementer=electron-implementer | reviewer=electron-reviewer
PROPOSED-BY: electron-planner
DESIGN-EVIDENCE-SOURCE: frontend-designer
SPRINT: ./SPRINT.md
FLOWS: GOV-E2E-05, GOV-E2E-06

OUTCOME
The fixture produces principal-pending.fixture-evidence and group-pending.fixture-evidence from real UI clicks.

CRITICAL CONSTRAINTS
- MUST toggle the principal own grant through the visible principal editor.
- MUST expand the group and toggle the group grant through visible controls.
- MUST NOT set pending state by mutating fixture state from the test.
- MUST label evidence as fixture evidence, not product-backend proof.

DONE WHEN
- AC-1: principal own-grant save shows a principal pending marker and one-change banner.
- AC-2: inherited grant remains visibly read-only while own grant is edited.
- AC-3: group grant edit shows a group pending marker and the shared banner count reaches two.

ACCEPTANCE CRITERIA
AC-1 [PRIMARY]: Principal pending marker
  GIVEN: the governance fixture app is open as an admin with administration:write
  WHEN: the user opens a principal row, toggles reviews:write as an own grant, and saves
  THEN: the changed principal shows a pending marker and Commit changes banner appears with a positive count
  TEST_TIER: e2e
  VERIFY: pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep "principal and group changes"

AC-2: Inherited grants remain read-only
  GIVEN: the principal editor contains both own and inherited grants
  WHEN: the user stages an own-grant change
  THEN: the inherited grant remains displayed as inherited and is not edited as an own grant
  TEST_TIER: e2e
  VERIFY: pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep "principal and group changes"

AC-3: Group pending marker
  GIVEN: a principal change is pending
  WHEN: the user switches to Groups, expands a group, and grants reviews:write
  THEN: the group shows a pending marker and the same commit banner remains visible with the updated count
  TEST_TIER: e2e
  VERIFY: pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep "principal and group changes"

TEST CRITERIA
- TC-1 (AC-1): Principal own-grant edit produces a pending marker and commit banner.
- TC-2 (AC-2): Inherited grant remains read-only while own grant is staged.
- TC-3 (AC-3): Group grant edit increments the shared pending banner and marks the group pending.

SCOPE
writeAllowed:
- .spec/prds/governance/tasks/sprint-06a-governance-ui-scaffold-principals-groups/E2E-GOV-003-principal-group-pending.md
- e2e/playwright/tests/governance-fixture/governance-fixture.spec.ts
- e2e/playwright/fixtures/governance-app/**

writeProhibited:
- apps/desktop/src/**
- apps/desktop/tests/governance/**
- crates/**
- packages/but-sdk/src/generated/**

DEPENDENCIES
Depends on: E2E-GOV-001
Blocks: E2E-GOV-004
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "E2E-GOV-003",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "governance_fixture_admin_editable": {
      "description": "Vite governance fixture with editable admin persona, settings-agent principal, eng group, and pending banner.",
      "seed_method": "ui_flow",
      "records": ["principal row `settings-agent`", "group row `eng`", "permission checkbox `reviews:write`"]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN editable admin fixture WHEN reviews:write own grant is saved THEN the principal pending marker and one-change banner appear.",
      "verify": "pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep \"principal and group changes\"",
      "maps_to_ac": null,
      "scenario": {
        "test_tier": "e2e",
        "tier": "visible",
        "verification_service": "vite-governance-fixture",
        "negative_control": { "would_fail_if": ["direct state mutation bypasses UI flow", "static pending marker", "empty principal list"] },
        "evidence": { "artifact_type": "video", "required_capture": true },
        "cases": [
          {
            "start_ref": "governance_fixture_admin_editable",
            "action": { "actor": "playwright_user", "steps": ["open principal row", "check reviews:write own grant", "save changes"] },
            "end_state": { "must_observe": ["commit banner `1 pending changes`", "test id `principals-list-pending-settings-agent` count 1", "principal row `settings-agent` has `reviews:write` own grant"], "must_not_observe": ["commit banner `0 pending changes`", "empty principal list"] }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "GIVEN inherited grant in the principal editor WHEN own grant is staged THEN inherited grant remains read-only.",
      "verify": "pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep \"principal and group changes\"",
      "maps_to_ac": null
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "GIVEN principal change is pending WHEN eng group receives reviews:write THEN group pending marker and two-change banner appear.",
      "verify": "pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep \"principal and group changes\"",
      "maps_to_ac": null
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "Principal own-grant edit produces a pending marker and commit banner.",
      "verify": "pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep \"principal and group changes\"",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "Inherited grant remains read-only while own grant is staged.",
      "verify": "pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep \"principal and group changes\"",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "Group grant edit increments the shared pending banner and marks the group pending.",
      "verify": "pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep \"principal and group changes\"",
      "maps_to_ac": "AC-3"
    }
  ]
}
-->
</details>
