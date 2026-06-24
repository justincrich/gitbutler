# E2E-GOV-101: Desktop E2E governance sidebar access gate

## What this does

Adds the real desktop-product E2E task for opening Project Settings and proving the admin sidebar gate:
admin sees Permissions & Governance; non-admin/member does not.

## Why

Fixture evidence helps with watchability, but sprint closeout needs a product-path E2E that drives the
real Project Settings modal rather than a component harness.

## How to verify

```bash
PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-access.spec.ts
pnpm --filter @gitbutler/desktop test:ct:desktop -- SettingsModalLayoutAdmin
pnpm --filter @gitbutler/desktop test:ct:desktop -- ProjectSettingsModalContentGovernance
```

## Scope

- `e2e/playwright/tests/governance-settings-access.spec.ts`
- `e2e/playwright/src/governance.ts`
- `e2e/playwright/scripts/governance-*`

<details>
<summary>Full agent specification</summary>

```
TASK: E2E-GOV-101 - Desktop E2E governance sidebar access gate
TASK_TYPE: FEATURE
STATUS: Done
STATUS_NOTE: Realized by e2e/playwright/tests/governance-settings-access.spec.ts (real desktop + but-server e2e, merged on master). Re-verify: PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-access.spec.ts
PRIORITY: P0
EFFORT: S
AGENT: implementer=sveltekit-implementer | reviewer=sveltekit-reviewer
PROPOSED-BY: sveltekit-planner
DESIGN-EVIDENCE-SOURCE: frontend-designer
SPRINT: ./SPRINT.md
FLOWS: GOV-E2E-01, GOV-E2E-02

OUTCOME
The real desktop E2E opens Project Settings and proves admin-visible plus non-admin-hidden access behavior.

CRITICAL CONSTRAINTS
- MUST drive the real desktop Project Settings modal through product UI controls.
- MUST NOT mount ProjectSettingsModalContent directly in the E2E.
- MUST NOT force state with window snippets, store mutation, or SDK calls from the test.
- MUST label any fixture fallback evidence separately from product E2E evidence.

DONE WHEN
- AC-1: admin product session sees Permissions & Governance in the settings sidebar.
- AC-2: member/non-admin product session does not see Permissions & Governance.
- AC-3: the E2E uses product chrome/settings entry points, not component mounting.

ACCEPTANCE CRITERIA
AC-1 [PRIMARY]: Admin product sidebar visibility
  GIVEN: a desktop E2E project is opened with a seeded or real admin session
  WHEN: the test opens Project Settings through the product chrome settings control
  THEN: the modal is visible and the sidebar contains Permissions & Governance alongside normal settings entries
  TEST_TIER: e2e
  VERIFY: PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-access.spec.ts

AC-2: Non-admin product sidebar absence
  GIVEN: the desktop E2E harness is run with a seeded or real member session
  WHEN: the test opens Project Settings through the same product chrome settings control
  THEN: Permissions & Governance is absent and normal settings entries remain visible
  TEST_TIER: e2e
  VERIFY: PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-access.spec.ts

AC-3: Product-path E2E boundary
  GIVEN: component tests already cover SettingsModalLayout admin filtering
  WHEN: the product E2E runs
  THEN: it drives the real desktop Project Settings modal rather than a component harness mount
  TEST_TIER: e2e
  VERIFY: rg -n "chrome-sidebar-project-settings-button|project-settings-modal|Permissions & Governance" e2e/playwright/tests/governance-settings-access.spec.ts

TEST CRITERIA
- TC-1 (AC-1): Admin product session shows Permissions & Governance in the real Project Settings sidebar.
- TC-2 (AC-2): Non-admin product session hides Permissions & Governance but still shows normal settings.
- TC-3 (AC-3): The test uses the desktop E2E product path and does not duplicate the component harness.

SCOPE
writeAllowed:
- .spec/prds/governance/tasks/sprint-06a-governance-ui-scaffold-principals-groups/E2E-GOV-101-desktop-access-gate.md
- e2e/playwright/tests/governance-settings-access.spec.ts
- e2e/playwright/src/governance.ts
- e2e/playwright/scripts/governance-*

writeProhibited:
- apps/desktop/src/**
- apps/desktop/tests/governance/**
- packages/but-sdk/src/generated/**
- any +page.server.ts or +layout.server.ts

DEPENDENCIES
Depends on: MGMT-UI-001, MGMT-UI-002
Blocks: E2E-GOV-102
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "E2E-GOV-101",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "desktop_admin_member_sessions": {
      "description": "Real desktop E2E sessions seeded through the supported public test setup with one admin user and one member user in the same project.",
      "seed_method": "public_api",
      "records": ["admin role:admin", "member role:member", "project_settings_entry:testid=chrome-sidebar-project-settings-button", "project_settings_modal:testid=project-settings-modal"]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN admin desktop session WHEN Project Settings opens THEN Permissions & Governance is visible with normal settings entries.",
      "verify": "PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-access.spec.ts",
      "maps_to_ac": null,
      "scenario": {
        "id": "E2E-GOV-101-AC-1",
        "test_tier": "e2e",
        "tier": "visible",
        "verification_service": "desktop-product-e2e",
        "negative_control": { "would_fail_if": ["component-only mount with static sidebar", "admin role disconnected from Project Settings", "empty settings sidebar"] },
        "evidence": { "artifact_type": "video", "required_capture": true },
        "cases": [
          {
            "start_ref": "desktop_admin_member_sessions",
            "action": { "actor": "playwright_user", "steps": ["open workspace", "open Project Settings"] },
            "end_state": {
              "must_observe": ["sidebar item \"Permissions & Governance\" count == 1", "settings entry \"Project\" count == 1", "modal testid:project-settings-modal"],
              "must_not_observe": ["sidebar item \"Permissions & Governance\" count == 0", "empty settings sidebar", "text \"Settings page governance not Found.\""]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "GIVEN member desktop session WHEN Project Settings opens THEN Permissions & Governance is absent and normal settings remain visible.",
      "verify": "PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-access.spec.ts",
      "maps_to_ac": null,
      "scenario": {
        "id": "E2E-GOV-101-AC-2",
        "test_tier": "e2e",
        "tier": "visible",
        "verification_service": "desktop-product-e2e",
        "negative_control": { "would_fail_if": ["member role mocked as admin", "hardcoded governance sidebar item", "empty settings sidebar"] },
        "evidence": { "artifact_type": "video", "required_capture": true },
        "cases": [
          {
            "start_ref": "desktop_admin_member_sessions",
            "action": { "actor": "playwright_user", "steps": ["sign in as member user", "open workspace", "open Project Settings"] },
            "end_state": {
              "must_observe": ["settings entry \"Project\" count == 1", "modal testid:project-settings-modal"],
              "must_not_observe": ["sidebar item \"Permissions & Governance\" count == 1", "settings entry \"Project\" count == 0", "empty settings sidebar"]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "GIVEN desktop E2E spec WHEN inspected THEN it uses product chrome/settings entry points, not component mounting.",
      "verify": "rg -n \"chrome-sidebar-project-settings-button|project-settings-modal|Permissions & Governance\" e2e/playwright/tests/governance-settings-access.spec.ts",
      "maps_to_ac": null,
      "scenario": {
        "id": "E2E-GOV-101-AC-3",
        "test_tier": "e2e",
        "tier": "visible",
        "verification_service": "desktop-product-e2e",
        "negative_control": { "would_fail_if": ["component mount stub replaces product chrome", "static modal fixture bypasses settings button", "empty spec omits selectors"] },
        "evidence": { "artifact_type": "file_artifact", "required_capture": true },
        "cases": [
          {
            "start_ref": "desktop_admin_member_sessions",
            "action": { "actor": "reviewer", "steps": ["inspect E2E spec selectors", "run product E2E"] },
            "end_state": {
              "must_observe": ["selector \"chrome-sidebar-project-settings-button\" count >= 1", "selector \"project-settings-modal\" count >= 1", "literal \"Permissions & Governance\" count >= 1"],
              "must_not_observe": ["selector \"chrome-sidebar-project-settings-button\" count == 0", "empty selector set", "mock ProjectSettingsModalContent"]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "Admin product session shows Permissions & Governance in the real Project Settings sidebar.",
      "verify": "PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-access.spec.ts",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "Non-admin product session hides Permissions & Governance but still shows normal settings.",
      "verify": "PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-access.spec.ts",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "The test uses the desktop E2E product path and does not duplicate the component harness.",
      "verify": "rg -n \"chrome-sidebar-project-settings-button|project-settings-modal|Permissions & Governance\" e2e/playwright/tests/governance-settings-access.spec.ts",
      "maps_to_ac": "AC-3"
    }
  ]
}
-->
</details>
