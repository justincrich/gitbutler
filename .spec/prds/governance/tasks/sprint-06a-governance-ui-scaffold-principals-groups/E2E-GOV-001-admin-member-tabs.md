# E2E-GOV-001: Governance fixture E2E admin/member visibility and four-tab navigation

## What this does

Adds the fixture-backed Playwright task for the watchable governance preflight: admin sees
Permissions & Governance, member/non-admin does not, and admin can see the four governance tabs.

## Why

This gives humans a stable headed flow for the first half of the Sprint 06a gate before a full
desktop-product E2E is available. This is fixture evidence only; it is not product-backend proof.

## How to verify

```bash
pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep "admin|non-admin|four tabs"
pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture:headed --grep "admin|non-admin|four tabs"
```

## Scope

- `e2e/playwright/playwright.governance-fixture.config.ts`
- `e2e/playwright/tests/governance-fixture/governance-fixture.spec.ts`
- `e2e/playwright/fixtures/governance-app/**`

<details>
<summary>Full agent specification</summary>

```
TASK: E2E-GOV-001 - Governance fixture E2E admin/member visibility and four-tab navigation
TASK_TYPE: FEATURE
STATUS: Backlog
PRIORITY: P1
EFFORT: S
AGENT: implementer=electron-implementer | reviewer=electron-reviewer
PROPOSED-BY: electron-planner
DESIGN-EVIDENCE-SOURCE: frontend-designer
SPRINT: ./SPRINT.md
FLOWS: GOV-E2E-01, GOV-E2E-02, GOV-E2E-04

OUTCOME
The Vite governance fixture proves admin-visible, non-admin-hidden, and four-tabs states in headed-capable Playwright.

CRITICAL CONSTRAINTS
- MUST label all screenshots, videos, traces, and notes as fixture evidence.
- MUST NOT claim this proves product backend authorization or committed governance refs.
- MUST drive the fixture through visible UI controls, not test-only globals or console snippets.
- NEVER modify apps/desktop, crates, or generated SDK files for this fixture task.

DONE WHEN
- AC-1: admin persona sees Permissions & Governance alongside normal settings entries.
- AC-2: member persona cannot see Permissions & Governance while normal settings remain visible.
- AC-3: admin persona sees Principals, Groups, Branch Gates, and Rules tabs with no not-found fallback.
- Evidence labels include admin-visible.fixture-evidence, non-admin-hidden.fixture-evidence, and four-tabs.fixture-evidence.

ACCEPTANCE CRITERIA
AC-1 [PRIMARY]: Admin fixture sidebar visibility
  GIVEN: the governance fixture app is open as an admin persona
  WHEN: Project Settings is shown
  THEN: Permissions & Governance is visible alongside normal settings entries
  TEST_TIER: e2e
  VERIFY: pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep "admin can see"

AC-2: Member fixture sidebar absence
  GIVEN: the governance fixture app is open as a member/non-admin persona
  WHEN: Project Settings is shown
  THEN: Permissions & Governance is absent and Project plus AI options remain visible
  TEST_TIER: e2e
  VERIFY: pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep "non-admin"

AC-3: Admin fixture four-tab navigation
  GIVEN: the governance fixture app is open as an admin persona
  WHEN: the user opens Permissions & Governance
  THEN: Principals, Groups, Branch Gates, and Rules tabs render and the not-found fallback is absent
  TEST_TIER: e2e
  VERIFY: pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep "four tabs"

TEST CRITERIA
- TC-1 (AC-1): Admin fixture persona sees the governance settings entry.
- TC-2 (AC-2): Member fixture persona cannot see the governance settings entry.
- TC-3 (AC-3): Admin fixture persona can open governance and see the four expected tabs.
- TC-4 (AC-1): Fixture evidence label says Not product E2E evidence.

SCOPE
writeAllowed:
- .spec/prds/governance/tasks/sprint-06a-governance-ui-scaffold-principals-groups/E2E-GOV-001-admin-member-tabs.md
- e2e/playwright/playwright.governance-fixture.config.ts
- e2e/playwright/tests/governance-fixture/governance-fixture.spec.ts
- e2e/playwright/fixtures/governance-app/**

writeProhibited:
- apps/desktop/src/**
- apps/desktop/tests/governance/**
- crates/**
- packages/but-sdk/src/generated/**

DEPENDENCIES
Depends on: none
Blocks: E2E-GOV-002, E2E-GOV-003, E2E-GOV-004

REVIEW
Reviewer must reject if the task claims product-backend proof or bypasses UI controls with a window snippet.
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "E2E-GOV-001",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "governance_fixture_admin_member": {
      "description": "Vite governance fixture with admin and member persona buttons and labeled fixture-only banner.",
      "seed_method": "ui_flow",
      "records": ["persona button `Admin`", "persona button `Member`", "settings entry `Permissions & Governance`"]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN admin fixture persona WHEN Project Settings is shown THEN Permissions & Governance is visible with normal settings entries.",
      "verify": "pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep \"admin can see\"",
      "maps_to_ac": null,
      "scenario": {
        "test_tier": "e2e",
        "tier": "visible",
        "verification_service": "vite-governance-fixture",
        "negative_control": { "would_fail_if": ["static shell omits persona-driven sidebar changes", "empty sidebar", "member persona selected"] },
        "evidence": { "artifact_type": "screenshot", "required_capture": true },
        "cases": [
          {
            "start_ref": "governance_fixture_admin_member",
            "action": { "actor": "playwright_user", "steps": ["open fixture", "select admin", "open Permissions & Governance"] },
            "end_state": { "must_observe": ["settings entry `Permissions & Governance` count 1", "settings entry `Project` count 1", "settings entry `AI options` count 1"], "must_not_observe": ["governance settings entry count 0", "empty sidebar"] }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "GIVEN member fixture persona WHEN Project Settings is shown THEN Permissions & Governance is absent and normal settings remain visible.",
      "verify": "pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep \"non-admin\"",
      "maps_to_ac": null
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "GIVEN admin fixture persona WHEN governance is opened THEN the four tabs render and the not-found fallback is absent.",
      "verify": "pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep \"four tabs\"",
      "maps_to_ac": null
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "Admin fixture persona sees the governance settings entry.",
      "verify": "pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep \"admin can see\"",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "Member fixture persona cannot see the governance settings entry.",
      "verify": "pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep \"non-admin\"",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "Admin fixture persona can open governance and see the four expected tabs.",
      "verify": "pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep \"four tabs\"",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "Fixture evidence label says Not product E2E evidence.",
      "verify": "rg -n \"Not product E2E evidence|Fixture governance harness\" e2e/playwright/tests/governance-fixture e2e/playwright/fixtures/governance-app",
      "maps_to_ac": "AC-1"
    }
  ]
}
-->
</details>
