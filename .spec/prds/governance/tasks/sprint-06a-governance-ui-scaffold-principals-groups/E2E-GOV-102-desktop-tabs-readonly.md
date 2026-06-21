# E2E-GOV-102: Desktop E2E read-only admin and four governance tabs

## What this does

Adds the real desktop-product E2E task for the governance page shell: four tabs render, and an
admin without `administration:write` can view but cannot edit.

## Why

This is the product-path equivalent of the read-only and four-tab human testing steps. It catches
gaps that component tests and the fixture harness cannot prove alone.

## How to verify

```bash
PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-tabs-readonly.spec.ts
pnpm --filter @gitbutler/desktop test:ct:desktop -- GovernanceSettingsTabs
pnpm --filter @gitbutler/desktop test:ct:desktop -- GovernanceSettingsReadOnly
pnpm --filter @gitbutler/desktop test:ct:desktop -- GovernanceSettingsAdminRoleNoWrite
```

## Scope

- `e2e/playwright/tests/governance-settings-tabs-readonly.spec.ts`
- `e2e/playwright/src/governance.ts`
- `e2e/playwright/scripts/governance-*`

<details>
<summary>Full agent specification</summary>

```
TASK: E2E-GOV-102 - Desktop E2E read-only admin and four governance tabs
TASK_TYPE: FEATURE
STATUS: Backlog
PRIORITY: P0
EFFORT: S
AGENT: implementer=sveltekit-implementer | reviewer=sveltekit-reviewer
PROPOSED-BY: sveltekit-planner
DESIGN-EVIDENCE-SOURCE: frontend-designer
SPRINT: ./SPRINT.md
FLOWS: GOV-E2E-03, GOV-E2E-04

OUTCOME
The real desktop settings page shows the four governance tabs and disables write controls for read-only admin.

CRITICAL CONSTRAINTS
- MUST drive the real desktop settings page.
- MUST derive read-only behavior from governance status, not from user.role alone.
- MUST NOT validate Sprint 06b Branch Gates/Rules behavior beyond tab shell and disabled placeholders.
- MUST NOT edit production UI code in this E2E task.

DONE WHEN
- AC-1: admin opens Permissions & Governance and sees Principals, Groups, Branch Gates, Rules.
- AC-2: read-only admin sees administration:write message and disabled/absent write controls.
- AC-3: read-only attempts do not dispatch perm_* or group_* writes from UI actions.

ACCEPTANCE CRITERIA
AC-1 [PRIMARY]: Four governance tabs
  GIVEN: admin desktop E2E session can see Permissions & Governance
  WHEN: the user opens it from Project Settings
  THEN: heading is visible, not-found fallback is absent, and four tabs are present in order
  TEST_TIER: e2e
  VERIFY: PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-tabs-readonly.spec.ts

AC-2: Read-only admin disabled state
  GIVEN: admin desktop E2E session lacks administration:write in governance status
  WHEN: the user opens Permissions & Governance
  THEN: a read-only message is visible and principal, group, branch gate, rule, and commit controls are disabled or absent
  TEST_TIER: e2e
  VERIFY: PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-tabs-readonly.spec.ts

AC-3: Read-only does not write
  GIVEN: read-only admin has user role admin but hasAdminWrite false
  WHEN: the E2E attempts to interact with write controls on Principals and Groups
  THEN: no perm_* or group_* write is observed and controls remain disabled
  TEST_TIER: e2e
  VERIFY: PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-tabs-readonly.spec.ts

TEST CRITERIA
- TC-1 (AC-1): Four tabs render from the real desktop settings page.
- TC-2 (AC-2): Read-only admin can view governance settings but cannot edit any Sprint 06a surface.
- TC-3 (AC-3): Read-only authority comes from governance_status_read, not user.role.

SCOPE
writeAllowed:
- .spec/prds/governance/tasks/sprint-06a-governance-ui-scaffold-principals-groups/E2E-GOV-102-desktop-tabs-readonly.md
- e2e/playwright/tests/governance-settings-tabs-readonly.spec.ts
- e2e/playwright/src/governance.ts
- e2e/playwright/scripts/governance-*

writeProhibited:
- apps/desktop/src/**
- apps/desktop/tests/governance/**
- packages/but-sdk/src/generated/**
- crates/**

DEPENDENCIES
Depends on: E2E-GOV-101, MGMT-UI-003, DESIGN-MGMT-001, DESIGN-MGMT-003
Blocks: E2E-GOV-103
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "E2E-GOV-102",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "desktop_admin_readonly_sessions": {
      "description": "Real desktop E2E sessions seeded through the supported public test setup with an admin user whose governance status has hasAdminWrite=false.",
      "seed_method": "public_api",
      "records": ["admin role:admin", "governance_status.hasAdminWrite:false", "governance_tabs_expected:4", "project_settings_modal:testid=project-settings-modal"]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN admin desktop session WHEN Permissions & Governance opens THEN four tabs render and not-found fallback is absent.",
      "verify": "PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-tabs-readonly.spec.ts",
      "maps_to_ac": null,
      "scenario": {
        "id": "E2E-GOV-102-AC-1",
        "test_tier": "e2e",
        "tier": "visible",
        "verification_service": "desktop-product-e2e",
        "negative_control": { "would_fail_if": ["component-only mount with mocked tabs", "missing tab shell", "static settings fallback"] },
        "evidence": { "artifact_type": "video", "required_capture": true },
        "cases": [
          {
            "start_ref": "desktop_admin_readonly_sessions",
            "action": { "actor": "playwright_user", "steps": ["open Project Settings", "click Permissions & Governance"] },
            "end_state": {
              "must_observe": ["tab \"Principals\" index == 1", "tab \"Groups\" index == 2", "tab \"Branch Gates\" index == 3", "tab \"Rules\" index == 4", "governance tab count == 4"],
              "must_not_observe": ["governance tab count == 0", "empty governance page", "text \"Settings page governance not Found.\""]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "GIVEN admin lacks administration:write WHEN governance opens THEN read-only message and disabled or absent write controls appear.",
      "verify": "PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-tabs-readonly.spec.ts",
      "maps_to_ac": null,
      "scenario": {
        "id": "E2E-GOV-102-AC-2",
        "test_tier": "e2e",
        "tier": "visible",
        "verification_service": "desktop-product-e2e",
        "negative_control": { "would_fail_if": ["hasAdminWrite mocked true", "write controls left static-enabled", "empty read-only page"] },
        "evidence": { "artifact_type": "video", "required_capture": true },
        "cases": [
          {
            "start_ref": "desktop_admin_readonly_sessions",
            "action": { "actor": "playwright_user", "steps": ["open Permissions & Governance", "visit Principals", "visit Groups", "visit Branch Gates", "visit Rules"] },
            "end_state": {
              "must_observe": ["read-only notice contains \"administration:write\"", "Commit changes button disabled count >= 1", "enabled write control count == 0"],
              "must_not_observe": ["read-only notice count == 0", "enabled write control count > 0", "empty governance page"]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "GIVEN read-only admin WHEN write controls are attempted THEN no perm_* or group_* write is observed.",
      "verify": "PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-tabs-readonly.spec.ts",
      "maps_to_ac": null,
      "scenario": {
        "id": "E2E-GOV-102-AC-3",
        "test_tier": "e2e",
        "tier": "visible",
        "verification_service": "desktop-product-e2e",
        "negative_control": { "would_fail_if": ["write RPC stub accepts disabled clicks", "permission dispatch not disconnected for read-only", "mocked event log is empty by default"] },
        "evidence": { "artifact_type": "event_log", "required_capture": true },
        "cases": [
          {
            "start_ref": "desktop_admin_readonly_sessions",
            "action": { "actor": "playwright_user", "steps": ["attempt Principals write control", "attempt Groups write control", "capture governance write events"] },
            "end_state": {
              "must_observe": ["event_log perm_write count == 0", "event_log group_write count == 0", "read-only notice contains \"administration:write\""],
              "must_not_observe": ["read-only notice count == 0", "empty event log without attempted controls", "enabled write control count > 0"]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "Four tabs render from the real desktop settings page.",
      "verify": "PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-tabs-readonly.spec.ts",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "Read-only admin can view governance settings but cannot edit any Sprint 06a governance surface.",
      "verify": "PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-tabs-readonly.spec.ts",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "Read-only authority comes from governance_status_read, not user.role.",
      "verify": "pnpm --filter @gitbutler/desktop test:ct:desktop -- GovernanceSettingsAdminRoleNoWrite",
      "maps_to_ac": "AC-3"
    }
  ]
}
-->
</details>
