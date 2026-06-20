# MGMT-UI-002: `ProjectSettingsModalContent` governance branch + wire `isAdmin` to `SettingsModalLayout`

## What this does

Connects the `governance` page id to its renderer and wires the admin visibility gate, mirroring the identical pattern in `GeneralSettingsModalContent.svelte`: branches `ProjectSettingsModalContent.svelte` to render `GovernanceSettings.svelte` when `currentPage.id === 'governance'`, and wires `isAdmin` (from `userService.user?.role === 'admin'`, the cloud `User.role`, B18) into `SettingsModalLayout`. An admin opening Project Settings sees **Permissions & Governance** in the sidebar; a non-admin does not. The renderer `adminOnly` filter is **UX convenience**; the backend `administration:write` check at the `but-api` boundary is the enforcement boundary.

## Why

Sprint 06a · PRD UC-MGMT-01 · criteria T-MGMT-002/003/005 · capability CAP-AUTHZ-01. Closes the documented `isAdmin` wiring gap so the governance surface is admin-gated in the sidebar (a layer beneath the server-side enforcement).

## How to verify

PRIMARY **AC-1** — `pnpm test:ct:desktop -- SettingsModalLayoutAdmin`: mounting `ProjectSettingsModalContent` with `isAdmin=true` renders a sidebar button labeled "Permissions & Governance"; with `isAdmin=false` it is absent. Full gate set in the spec below.

## Scope

- `apps/desktop/src/components/settings/ProjectSettingsModalContent.svelte` (MODIFY — governance branch + `isAdmin` wiring)
- `apps/desktop/tests/governance/SettingsModalLayoutAdmin.spec.ts` (NEW — admin/non-admin visibility CT)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: MGMT-UI-002 — ProjectSettingsModalContent governance branch + wire isAdmin to SettingsModalLayout
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     S  (45 min)
AGENT:      implementer=sveltekit-implementer | reviewer=sveltekit-reviewer
PROPOSED-BY: sveltekit-planner
SPRINT:     ./SPRINT.md
PRD_REFS:   UC-MGMT-01, T-MGMT-002, T-MGMT-003, T-MGMT-005
CAPABILITIES: CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  pnpm test:ct:desktop -- SettingsModalLayoutAdmin
  check: pnpm -F @gitbutler/desktop check   |   lint: pnpm lint

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
When isAdmin=true the governance sidebar item is visible and clicking it renders GovernanceSettings.svelte;
when isAdmin=false the item is absent from the rendered sidebar. isAdmin is derived exclusively from
userService.user?.role === 'admin'. No new route. pnpm test:ct:desktop -- SettingsModalLayoutAdmin exits 0.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] Derive isAdmin exclusively from userService.user?.role === 'admin' (cloud User.role, B18) injected
  via USER_SERVICE — no other admin-detection logic (mirror GeneralSettingsModalContent.svelte:27-44).
- [MUST] Render GovernanceSettings.svelte ONLY when currentPage.id === 'governance' — no other page branch changes.
- [MUST] Keep the renderer adminOnly filter as UX convenience; the backend administration:write check at the
  but-api command boundary is the enforcement boundary (a renderer that bypassed its guard still hits the server gate).
- [NEVER] NEVER add a new top-level route (it is a state of the settings modal).
- [NEVER] NEVER add a secondary adminOnly check in the content snippet — the sidebar filter in SettingsModalLayout
  is the UX gate; the backend is the enforcement gate.
- [NEVER] NEVER change the SettingsModalLayout.svelte isAdmin prop signature (it already exists at :24).
- [STRICTLY] No +page.server.ts; no relative imports (use @gitbutler/*).

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: admin viewer sees the governance page in the sidebar
- [ ] AC-2: non-admin viewer does not see the governance page in the sidebar
- [ ] AC-3: the governance branch renders GovernanceSettings when the page is selected
- [ ] AC-4: isAdmin derived from userService.user.role === 'admin' exclusively
- [ ] AC-5: no new route added
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (behavioral ACs carry scenarios; AC-4/AC-5 are build-gate)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: admin viewer sees governance page in sidebar
  GIVEN: ProjectSettingsModalContent mounted with isAdmin=true (fixture isAdmin_true)
  WHEN:  the component renders
  THEN:  the sidebar list contains a button labeled "Permissions & Governance" (alongside Project, AI options)
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- SettingsModalLayoutAdmin

AC-2: non-admin viewer does not see governance page in sidebar
  GIVEN: ProjectSettingsModalContent mounted with isAdmin=false (fixture isAdmin_false)
  WHEN:  the component renders
  THEN:  the sidebar still renders the standard items (Project, AI options) but the governance row count is 0
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- SettingsModalLayoutAdmin

AC-3: governance branch renders GovernanceSettings when page is selected
  GIVEN: isAdmin=true and currentPage.id='governance'
  WHEN:  the content snippet branch evaluates
  THEN:  GovernanceSettings.svelte is rendered (its root data-testid/aria-label present), not the not-found fallback
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- ProjectSettingsModalContentGovernance

AC-4: isAdmin derived from userService.user.role === 'admin' exclusively (build-gate)
  GIVEN: ProjectSettingsModalContent.svelte
  WHEN:  the isAdmin prop is computed
  THEN:  the source is userService.user?.role === 'admin' injected via USER_SERVICE — no other admin-detection logic
  TEST_TIER: build-gate   UNIT_TEST_JUSTIFIED: source-of-truth structural invariant — the isAdmin derivation must be user?.role === 'admin'
  VERIFY: grep -n 'isAdmin' apps/desktop/src/components/settings/ProjectSettingsModalContent.svelte

AC-5: no new route added (build-gate)
  GIVEN: apps/desktop/src/routes/
  WHEN:  MGMT-UI-002 lands
  THEN:  no new route directory/file referencing governance was created under src/routes/
  TEST_TIER: build-gate   UNIT_TEST_JUSTIFIED: routing invariant
  VERIFY: find apps/desktop/src/routes -name '*governance*' | wc -l | grep '^0$'

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): with isAdmin=true the rendered sidebar HTML contains 'Permissions & Governance'
    VERIFY: pnpm test:ct:desktop -- SettingsModalLayoutAdmin
- TC-2 (-> AC-2): with isAdmin=false the rendered sidebar HTML does not contain 'Permissions & Governance'
    VERIFY: pnpm test:ct:desktop -- SettingsModalLayoutAdmin
- TC-3 (-> AC-3): currentPage.id='governance' + isAdmin=true renders GovernanceSettings
    VERIFY: pnpm test:ct:desktop -- ProjectSettingsModalContentGovernance
- TC-4 (-> AC-4): pnpm -F @gitbutler/desktop check exits 0 after MGMT-UI-002 changes
    VERIFY: pnpm -F @gitbutler/desktop check

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides: the governance branch in ProjectSettingsModalContent + isAdmin sidebar wiring (UX-convenience gate)
consumes: MGMT-UI-001 (ProjectSettingsPageId 'governance' + page entry); GovernanceSettings.svelte (MGMT-UI-003 — forward import);
          apps/desktop/src/lib/user/userService.svelte (USER_SERVICE, user.role); SettingsModalLayout.svelte:24 (isAdmin prop)
boundary_contracts:
  - renderer adminOnly is UX convenience; server-side but-authz administration:write at the but-api command boundary is enforcement (UI is never a bypass).

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - apps/desktop/src/components/settings/ProjectSettingsModalContent.svelte (governance branch + isAdmin wiring)
  - apps/desktop/tests/governance/SettingsModalLayoutAdmin.spec.ts (CT spec)
writeProhibited:
  - apps/desktop/src/components/settings/SettingsModalLayout.svelte (isAdmin prop exists; do not change its signature)
  - apps/desktop/src/lib/state/uiState.svelte.ts (MGMT-UI-001)
  - apps/desktop/src/lib/settings/projectSettingsPages.ts (MGMT-UI-001)
  - any +page.server.ts; any new route directory

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. apps/desktop/src/components/settings/ProjectSettingsModalContent.svelte — the host to extend (no isAdmin wiring today)
2. apps/desktop/src/components/settings/GeneralSettingsModalContent.svelte:27-44 — exact isAdmin wiring to mirror (inject USER_SERVICE, user?.role === 'admin')
3. apps/desktop/src/components/settings/SettingsModalLayout.svelte:24,30,53 — isAdmin?: boolean prop + pages.filter
4. apps/desktop/src/lib/user/userService.svelte — USER_SERVICE injection token
5. apps/desktop/src/lib/settings/projectSettingsPages.ts + uiState.svelte.ts:21 — page entry + union (extended by MGMT-UI-001)
6. .spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md#placement — isAdmin B18 documented gap

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- pnpm test:ct:desktop -- SettingsModalLayoutAdmin                  -> Exit 0
- pnpm test:ct:desktop -- ProjectSettingsModalContentGovernance     -> Exit 0
- pnpm -F @gitbutler/desktop check                                  -> Exit 0
- pnpm lint                                                          -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: Identical to GeneralSettingsModalContent.svelte isAdmin wiring — inject(USER_SERVICE) ->
  userService.user?.role === 'admin' -> isAdmin prop on SettingsModalLayout; content snippet branches on page id.
pattern_source: apps/desktop/src/components/settings/GeneralSettingsModalContent.svelte:27-44
anti_pattern: a secondary adminOnly check in the content snippet; a new route; client-side admin override.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: sveltekit-implementer
reviewer: sveltekit-reviewer
coding_standards: apps/desktop/AGENTS.md, frontend.md (inject(USER_SERVICE) pattern; no relative imports; no console.log)

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: MGMT-UI-001 (page id + entry), MGMT-IPC-004 (SDK)
Blocks:     MGMT-UI-003
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "MGMT-UI-002",
  "proposed_by": "sveltekit-planner",
  "verification_policy": { "requires_tests": true, "requires_red_evidence": true, "requires_seeded_evidence": true },
  "fixtures": {
    "isAdmin_true": { "description": "userService.user = { role: 'admin' } injected via the but-sdk fixture layer the desktop CT harness mounts against.", "seed_method": "ui_flow", "records": ["signed-in User role = admin"] },
    "isAdmin_false": { "description": "userService.user = { role: 'member' } injected via the but-sdk fixture layer.", "seed_method": "ui_flow", "records": ["signed-in User role = member (non-admin)"] }
  },
  "requirements": [
    { "id": "AC-1", "type": "acceptance_criterion", "primary": true, "description": "GIVEN ProjectSettingsModalContent mounted with isAdmin=true WHEN it renders THEN the sidebar contains a button labeled 'Permissions & Governance'", "verify": "pnpm test:ct:desktop -- SettingsModalLayoutAdmin", "scenario": { "id": "AC-1", "primary": true, "tier": "visible", "test_tier": "integration", "verification_service": "desktop-ct-harness", "negative_control": { "would_fail_if": ["the governance page entry is absent from projectSettingsPages", "adminOnly is hardcoded so the item never shows (static)", "the sidebar is a disconnected static shell"] }, "evidence": { "artifact_type": "screenshot", "required_capture": true }, "cases": [ { "start_ref": "isAdmin_true", "action": { "actor": "user", "steps": ["mount ProjectSettingsModalContent with the isAdmin_true fixture", "observe the settings sidebar"] }, "end_state": { "must_observe": ["the sidebar contains a button labeled `\"Permissions & Governance\"`", "the governance row renders alongside the `\"Project\"` and `\"AI options\"` items"], "must_not_observe": ["the `\"Permissions & Governance\"` item absent for an admin", "an empty sidebar with no items"] } } ] } },
    { "id": "AC-2", "type": "acceptance_criterion", "primary": false, "description": "GIVEN ProjectSettingsModalContent mounted with isAdmin=false WHEN it renders THEN the sidebar shows the standard items but the governance row count is 0", "verify": "pnpm test:ct:desktop -- SettingsModalLayoutAdmin", "scenario": { "id": "AC-2", "primary": false, "tier": "visible", "test_tier": "integration", "verification_service": "desktop-ct-harness", "negative_control": { "would_fail_if": ["adminOnly is not set to true so the item leaks to non-admins", "the SettingsModalLayout filter is bypassed/stubbed", "a static sidebar that always shows governance"] }, "evidence": { "artifact_type": "screenshot", "required_capture": true }, "cases": [ { "start_ref": "isAdmin_false", "action": { "actor": "user", "steps": ["mount ProjectSettingsModalContent with the isAdmin_false fixture", "observe the settings sidebar"] }, "end_state": { "must_observe": ["the sidebar still renders the standard items `\"Project\"` and `\"AI options\"`", "the `\"Permissions & Governance\"` governance row count is 0"], "must_not_observe": ["`\"Permissions & Governance\"` present for a non-admin", "the governance page reachable by a non-admin", "an empty sidebar (`0` standard items) for a non-admin"] } } ] } },
    { "id": "AC-3", "type": "acceptance_criterion", "primary": false, "description": "GIVEN isAdmin=true and currentPage.id='governance' WHEN the content snippet evaluates THEN GovernanceSettings is rendered inline (not the not-found fallback)", "verify": "pnpm test:ct:desktop -- ProjectSettingsModalContentGovernance", "scenario": { "id": "AC-3", "primary": false, "tier": "visible", "test_tier": "integration", "verification_service": "desktop-ct-harness", "negative_control": { "would_fail_if": ["the else branch renders the not-found fallback (no governance branch)", "the branch is stubbed to render nothing", "a static placeholder is shown instead of GovernanceSettings"] }, "evidence": { "artifact_type": "screenshot", "required_capture": true }, "cases": [ { "start_ref": "isAdmin_true", "action": { "actor": "user", "steps": ["mount with isAdmin_true and currentPage.id = `\"governance\"`", "observe the content region"] }, "end_state": { "must_observe": ["`GovernanceSettings` is rendered inline (its root `data-testid`/aria-label present)", "the `\"governance\"` page branch resolves to GovernanceSettings"], "must_not_observe": ["`\"Settings page governance not Found\"` fallback text", "an empty content region"] } } ] } },
    { "id": "AC-4", "type": "acceptance_criterion", "primary": false, "description": "GIVEN ProjectSettingsModalContent.svelte WHEN isAdmin is computed THEN the source is userService.user?.role === 'admin' via USER_SERVICE — no other admin-detection logic", "verify": "grep -n 'isAdmin' apps/desktop/src/components/settings/ProjectSettingsModalContent.svelte" },
    { "id": "AC-5", "type": "acceptance_criterion", "primary": false, "description": "GIVEN apps/desktop/src/routes/ WHEN MGMT-UI-002 lands THEN no new route directory/file referencing governance was created", "verify": "find apps/desktop/src/routes -name '*governance*' | wc -l | grep '^0$'" },
    { "id": "TC-1", "type": "test_criterion", "description": "with isAdmin=true the rendered sidebar HTML contains 'Permissions & Governance'", "verify": "pnpm test:ct:desktop -- SettingsModalLayoutAdmin", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "with isAdmin=false the rendered sidebar HTML does not contain 'Permissions & Governance'", "verify": "pnpm test:ct:desktop -- SettingsModalLayoutAdmin", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "currentPage.id='governance' + isAdmin=true renders GovernanceSettings", "verify": "pnpm test:ct:desktop -- ProjectSettingsModalContentGovernance", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "pnpm -F @gitbutler/desktop check exits 0 after MGMT-UI-002 changes", "verify": "pnpm -F @gitbutler/desktop check", "maps_to_ac": "AC-4" }
  ]
}
-->
</details>
