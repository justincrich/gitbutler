# E2E-GOV-103: Desktop E2E principal own-grant and group grant pending state

## What this does

Adds the real desktop-product E2E task for staging a principal own-grant edit and a group grant edit,
then observing pending indicators and the shared commit banner.

## Why

This is the core admin edit portion of the human gate. It must use visible product controls, not direct
store calls or backend shortcuts.

## How to verify

```bash
PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-pending-edits.spec.ts
pnpm --filter @gitbutler/desktop test:ct:desktop -- PrincipalEditorBatchSave
pnpm --filter @gitbutler/desktop test:ct:desktop -- PrincipalsListRows
pnpm --filter @gitbutler/desktop test:ct:desktop -- GroupsListRows
pnpm --filter @gitbutler/desktop test:ct:desktop -- GroupsListRevokeToggle
```

## Scope

- `e2e/playwright/tests/governance-settings-pending-edits.spec.ts`
- `e2e/playwright/src/governance.ts`
- `e2e/playwright/scripts/governance-*`

<details>
<summary>Full agent specification</summary>

```
TASK: E2E-GOV-103 - Desktop E2E principal own-grant and group grant pending state
TASK_TYPE: FEATURE
STATUS: Done
STATUS_NOTE: Realized by e2e/playwright/tests/governance-settings-pending-edits.spec.ts (real desktop + but-server e2e, merged on master). Re-verify: PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-pending-edits.spec.ts
PRIORITY: P0
EFFORT: M
AGENT: implementer=sveltekit-implementer | reviewer=sveltekit-reviewer
PROPOSED-BY: sveltekit-planner
DESIGN-EVIDENCE-SOURCE: frontend-designer
SPRINT: ./SPRINT.md
FLOWS: GOV-E2E-05, GOV-E2E-06

OUTCOME
The desktop product E2E stages principal-pending and group-pending states through real governance UI interactions.

CRITICAL CONSTRAINTS
- MUST open Project Settings and use visible governance controls.
- MUST keep inherited permission rows read-only.
- MUST not assert optimistic enforcement before commit.
- MUST not add direct renderer filesystem reads or SDK calls from the test.

DONE WHEN
- AC-1: principal own-grant Save changes shows principal pending marker and banner.
- AC-2: group grant edit shows group pending state and keeps the shared banner.
- AC-3: E2E observes pending presentation without requiring committed/effective enforcement before commit.

ACCEPTANCE CRITERIA
AC-1 [PRIMARY]: Principal own-grant pending
  GIVEN: admin desktop E2E session has administration:write and opens Principals
  WHEN: user opens a principal row, toggles an editable own-grant permission, and clicks Save changes
  THEN: principal row shows a pending marker, commit banner appears, and inherited-only rows remain disabled
  TEST_TIER: e2e
  VERIFY: PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-pending-edits.spec.ts

AC-2: Group grant pending
  GIVEN: the principal own-grant change is pending and user navigates to Groups
  WHEN: user expands a group and grants an editable group permission
  THEN: group row shows pending state and the same Commit changes banner remains visible with non-zero count
  TEST_TIER: e2e
  VERIFY: PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-pending-edits.spec.ts

AC-3: Pending presentation before commit
  GIVEN: both principal and group changes are staged
  WHEN: test observes effective permissions before commit
  THEN: UI distinguishes pending presentation from committed/effective state and does not assert optimistic enforcement
  TEST_TIER: e2e
  VERIFY: PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-pending-edits.spec.ts

TEST CRITERIA
- TC-1 (AC-1): Principal own-grant edit through the real settings page shows pending row marker and commit banner.
- TC-2 (AC-2): Group grant edit through the real Groups tab shows group pending state and keeps the shared banner.
- TC-3 (AC-3): The E2E asserts pending UI behavior only; backend no-optimistic-enforcement remains covered by MGMT-IPC-005.

SCOPE
writeAllowed:
- .spec/prds/governance/tasks/sprint-06a-governance-ui-scaffold-principals-groups/E2E-GOV-103-desktop-pending-edits.md
- e2e/playwright/tests/governance-settings-pending-edits.spec.ts
- e2e/playwright/src/governance.ts
- e2e/playwright/scripts/governance-*

writeProhibited:
- apps/desktop/src/**
- apps/desktop/tests/governance/**
- packages/but-sdk/src/generated/**
- crates/**

DEPENDENCIES
Depends on: E2E-GOV-102, MGMT-UI-005, MGMT-UI-006, MGMT-UI-007, MGMT-UI-008, MGMT-IPC-005
Blocks: E2E-GOV-104
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "E2E-GOV-103",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "desktop_admin_governance_editable": {
      "description": "Real desktop E2E session seeded through the supported public test setup with administration:write, one editable principal own-grant, and one editable group grant.",
      "seed_method": "public_api",
      "records": ["admin role:admin", "governance_status.hasAdminWrite:true", "principal row:test-principal editable_own_grant:true", "group row:test-group editable_grant:true"]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN admin with administration:write WHEN own-grant is saved THEN principal pending marker and commit banner appear.",
      "verify": "PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-pending-edits.spec.ts",
      "maps_to_ac": null,
      "scenario": {
        "id": "E2E-GOV-103-AC-1",
        "test_tier": "e2e",
        "tier": "visible",
        "verification_service": "desktop-product-e2e",
        "negative_control": { "would_fail_if": ["component-only mount with static pending badge", "direct store mutation bypasses Save changes", "empty principal list"] },
        "evidence": { "artifact_type": "video", "required_capture": true },
        "cases": [
          {
            "start_ref": "desktop_admin_governance_editable",
            "action": { "actor": "playwright_user", "steps": ["open principal row", "toggle own grant", "click Save changes"] },
            "end_state": {
              "must_observe": ["principal row \"test-principal\" pending marker count == 1", "Commit changes banner pending count == 1", "inherited-only permission disabled count >= 1"],
              "must_not_observe": ["pending badge count 0", "empty principal list", "read-only admin notice"]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "GIVEN principal change pending WHEN group grant is edited THEN group pending marker and shared banner remain visible.",
      "verify": "PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-pending-edits.spec.ts",
      "maps_to_ac": null,
      "scenario": {
        "id": "E2E-GOV-103-AC-2",
        "test_tier": "e2e",
        "tier": "visible",
        "verification_service": "desktop-product-e2e",
        "negative_control": { "would_fail_if": ["group tab uses stub pending state", "principal pending state disconnected across tabs", "empty group list"] },
        "evidence": { "artifact_type": "video", "required_capture": true },
        "cases": [
          {
            "start_ref": "desktop_admin_governance_editable",
            "action": { "actor": "playwright_user", "steps": ["stage principal own grant", "open Groups tab", "expand test group", "toggle editable group grant"] },
            "end_state": {
              "must_observe": ["group row \"test-group\" pending marker count == 1", "principal pending marker count == 1", "Commit changes banner pending count >= 2"],
              "must_not_observe": ["pending badge count 0", "empty group list", "Commit changes banner pending count == 0"]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "GIVEN both edits staged WHEN effective permissions are observed before commit THEN test asserts pending UI only, not optimistic enforcement.",
      "verify": "PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-pending-edits.spec.ts",
      "maps_to_ac": null,
      "scenario": {
        "id": "E2E-GOV-103-AC-3",
        "test_tier": "e2e",
        "tier": "visible",
        "verification_service": "desktop-product-e2e",
        "negative_control": { "would_fail_if": ["optimistic enforcement stub marks effective before commit", "pending read model mocked as committed", "empty effective-permission snapshot"] },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "desktop_admin_governance_editable",
            "action": { "actor": "playwright_user", "steps": ["stage principal own grant", "stage group grant", "read pending and effective permission presentations before commit"] },
            "end_state": {
              "must_observe": ["pending changes count >= 2", "effective permission committed count == 0", "pending UI marker count >= 2"],
              "must_not_observe": ["pending badge count 0", "empty pending state", "effective permission committed count >= 2"]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "Principal own-grant edit through the real settings page shows a pending row marker and commit banner.",
      "verify": "PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-pending-edits.spec.ts",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "Group grant edit through the real Groups tab shows group pending state and keeps the shared banner.",
      "verify": "PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-pending-edits.spec.ts",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "The E2E asserts pending UI behavior only; backend no-optimistic-enforcement remains covered by MGMT-IPC-005 integration tests.",
      "verify": "cargo test -p gitbutler-tauri governance_pending_read_is_readonly_no_optimistic_enforcement",
      "maps_to_ac": "AC-3"
    }
  ]
}
-->
</details>
