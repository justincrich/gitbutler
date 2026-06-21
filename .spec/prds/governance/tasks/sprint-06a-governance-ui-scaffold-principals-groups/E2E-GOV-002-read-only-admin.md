# E2E-GOV-002: Governance fixture E2E read-only admin disabled state

## What this does

Adds fixture-backed Playwright coverage for an admin who can view Permissions & Governance but lacks
`administration:write`, so mutation controls are disabled or absent.

## Why

The human gate includes more than admin visibility. A user can pass the renderer admin gate and still
be read-only at the governance permission layer.

## How to verify

```bash
pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep "read-only admin"
pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture:headed --grep "read-only admin"
```

## Scope

- `e2e/playwright/tests/governance-fixture/governance-fixture.spec.ts`
- `e2e/playwright/fixtures/governance-app/**`

<details>
<summary>Full agent specification</summary>

```
TASK: E2E-GOV-002 - Governance fixture E2E read-only admin disabled state
TASK_TYPE: FEATURE
STATUS: Backlog
PRIORITY: P1
EFFORT: S
AGENT: implementer=electron-implementer | reviewer=electron-reviewer
PROPOSED-BY: electron-planner
DESIGN-EVIDENCE-SOURCE: frontend-designer
SPRINT: ./SPRINT.md
FLOWS: GOV-E2E-03

OUTCOME
The fixture proves read-only-admin.fixture-evidence: the governance page is visible but write controls cannot stage changes.

CRITICAL CONSTRAINTS
- MUST distinguish read-only admin from member/non-admin.
- MUST keep the fixture evidence label explicit.
- MUST NOT infer server-side authorization enforcement from fixture UI state.
- NEVER mutate product code for this fixture task.

DONE WHEN
- AC-1: read-only admin can open governance and sees an administration:write message.
- AC-2: principal own-grant and Save changes controls are disabled.
- AC-3: group, Branch Gates, Rules, and commit controls are disabled or absent.

ACCEPTANCE CRITERIA
AC-1 [PRIMARY]: Read-only admin page visibility
  GIVEN: the governance fixture app is open as a read-only admin persona
  WHEN: the user opens Permissions & Governance
  THEN: the page remains visible and shows a read-only message that names administration:write
  TEST_TIER: e2e
  VERIFY: pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep "read-only admin"

AC-2: Principal write controls disabled
  GIVEN: the read-only admin has opened the Principals tab
  WHEN: the user opens a principal editor
  THEN: own-grant controls and Save changes are disabled
  TEST_TIER: e2e
  VERIFY: pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep "read-only admin"

AC-3: Cross-tab write controls disabled or absent
  GIVEN: the read-only admin has opened Groups, Branch Gates, and Rules
  WHEN: write controls are inspected
  THEN: group grant, branch gate, rule, and commit controls are disabled or absent
  TEST_TIER: e2e
  VERIFY: pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep "read-only admin"

TEST CRITERIA
- TC-1 (AC-1): Read-only admin sees the governance page and administration:write explanation.
- TC-2 (AC-2): Read-only principal editor controls cannot stage changes.
- TC-3 (AC-3): Read-only group, branch gate, rules, and commit controls cannot write.

SCOPE
writeAllowed:
- .spec/prds/governance/tasks/sprint-06a-governance-ui-scaffold-principals-groups/E2E-GOV-002-read-only-admin.md
- e2e/playwright/tests/governance-fixture/governance-fixture.spec.ts
- e2e/playwright/fixtures/governance-app/**

writeProhibited:
- apps/desktop/src/**
- apps/desktop/tests/governance/**
- crates/**
- packages/but-sdk/src/generated/**

DEPENDENCIES
Depends on: E2E-GOV-001
Blocks: none
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "E2E-GOV-002",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "governance_fixture_read_only_admin": {
      "description": "Vite governance fixture with Read-only admin persona and disabled write controls.",
      "seed_method": "ui_flow",
      "records": ["persona button `Read-only admin`", "read-only message containing `administration:write`", "disabled `Save changes` control"]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN read-only admin fixture persona WHEN governance is opened THEN the page is visible and names administration:write.",
      "verify": "pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep \"read-only admin\"",
      "maps_to_ac": null,
      "scenario": {
        "test_tier": "e2e",
        "tier": "visible",
        "verification_service": "vite-governance-fixture",
        "negative_control": { "would_fail_if": ["admin persona with enabled controls", "member persona hidden page", "static shell omits disabled-control state"] },
        "evidence": { "artifact_type": "screenshot", "required_capture": true },
        "cases": [
          {
            "start_ref": "governance_fixture_read_only_admin",
            "action": { "actor": "playwright_user", "steps": ["select Read-only admin", "open Permissions & Governance"] },
            "end_state": { "must_observe": ["read-only banner `Read-only governance settings` count 1", "permission explanation `administration:write` count 1", "`Save changes` disabled count 1"], "must_not_observe": ["pending badge count 0", "empty governance page"] }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "GIVEN read-only admin on Principals WHEN a principal editor opens THEN own-grant and Save changes controls are disabled.",
      "verify": "pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep \"read-only admin\"",
      "maps_to_ac": null
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "GIVEN read-only admin across Groups, Branch Gates, and Rules WHEN write controls are inspected THEN they are disabled or absent.",
      "verify": "pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep \"read-only admin\"",
      "maps_to_ac": null
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "Read-only admin sees the governance page and administration:write explanation.",
      "verify": "pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep \"read-only admin\"",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "Read-only principal editor controls cannot stage changes.",
      "verify": "pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep \"read-only admin\"",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "Read-only group, branch gate, rules, and commit controls cannot write.",
      "verify": "pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep \"read-only admin\"",
      "maps_to_ac": "AC-3"
    }
  ]
}
-->
</details>
