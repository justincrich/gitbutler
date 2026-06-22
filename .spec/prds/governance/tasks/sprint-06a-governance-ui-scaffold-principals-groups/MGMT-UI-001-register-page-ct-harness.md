# MGMT-UI-001: Register the governance page + extend `ProjectSettingsPageId` + scaffold desktop CT harness

> **Red-Hat Remediation (cycle 1):** Resolved S1 (CRITICAL) — AC-2 now includes the required `icon: IconName` field with the verified value `'lock'` (confirmed present in `packages/ui/src/lib/icons/names.ts`).

## What this does

Three structural prerequisites every downstream MGMT-UI-\* task depends on: (1) extend the `ProjectSettingsPageId` union with a `governance` variant; (2) add a `"Permissions & Governance"` entry (`icon: 'lock'`, `adminOnly: true`) to `projectSettingsPages`; (3) scaffold the missing **`apps/desktop` Playwright CT harness** (T-MGMT-000 / B14) so `pnpm test:ct:desktop` can mount governance components against a `but-sdk` mock fixture — today `pnpm test:ct` runs only `@gitbutler/ui`, so none of the MGMT component tests can run without this.

## Why

Sprint 06a · PRD UC-MGMT-01 · criteria T-MGMT-000/001/033/034 · capability CAP-AUTHZ-01. The desktop CT harness is the hard prerequisite for all 38 MGMT component-test criteria; the page-id + sidebar entry are the registration the page renders through (no new route — a state of the settings modal).

## How to verify

PRIMARY **AC-3** — `pnpm test:ct:desktop -- GovernanceSettings`: an `apps/desktop` Playwright CT config exists, the `test:ct:desktop` script runs, the harness resolves `$components`/`$lib`/`@gitbutler/` aliases, and a smoke spec mounts `GovernanceSettings.svelte` (stub shell acceptable for this task only) and asserts it renders without throwing. Full gate set in the spec below.

## Scope

- `apps/desktop/src/lib/settings/projectSettingsPages.ts` (MODIFY — +1 governance entry)
- `apps/desktop/src/lib/state/uiState.svelte.ts` (MODIFY — extend `ProjectSettingsPageId`)
- `apps/desktop/playwright-ct.config.ts` (NEW — CT harness)
- `apps/desktop/package.json` (MODIFY — add `test:ct:desktop`)
- `apps/desktop/tests/governance/GovernanceSettings.spec.ts` (NEW — smoke CT spec)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: MGMT-UI-001 — Register the governance page + extend ProjectSettingsPageId + scaffold desktop CT harness
================================================================================

TASK_TYPE:  INFRA  (DESIGN_SYSTEM / BUILD prerequisite)
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     S  (30 min)
AGENT:      implementer=sveltekit-implementer | reviewer=sveltekit-reviewer
PROPOSED-BY: sveltekit-planner
SPRINT:     ./SPRINT.md
PRD_REFS:   UC-MGMT-01, T-MGMT-000, T-MGMT-001, T-MGMT-033, T-MGMT-034
CAPABILITIES: CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  pnpm test:ct:desktop -- GovernanceSettings
  check: pnpm -F @gitbutler/desktop check   |   lint: pnpm lint

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
`pnpm test:ct:desktop` executes a smoke CT spec mounting GovernanceSettings.svelte without error;
grep finds 'governance' in uiState.svelte.ts (ProjectSettingsPageId) and in projectSettingsPages.ts
(icon:'lock', adminOnly:true entry); pnpm -F @gitbutler/desktop check exits 0; no new SvelteKit route and no
+page.server.ts under governance.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] Extend the ProjectSettingsPageId union (uiState.svelte.ts) with 'governance' so
  ProjectSettingsModalContent.svelte type-narrows correctly (MGMT-UI-002 consumes this).
- [MUST] Add ONE projectSettingsPages entry id='governance', label='Permissions & Governance',
  icon:'lock', adminOnly:true (the SettingsModalLayout.svelte:53 pages.filter((p)=>!p.adminOnly||isAdmin) hides it from non-admins).
  NOTE: 'lock' is a verified IconName (present in packages/ui/src/lib/icons/names.ts).
- [MUST] Scaffold apps/desktop/playwright-ct.config.ts + a test:ct:desktop script resolving $components/$lib/@gitbutler/*
  aliases identical to the production vite config, plus a smoke spec mounting GovernanceSettings (stub shell ok here only).
- [NEVER] NEVER add a new SvelteKit route (no [projectId]/governance, no /governance dir) — it is a STATE of the settings modal.
- [NEVER] NEVER add +page.server.ts / +layout.server.ts (adapter-static — no SSR).
- [NEVER] NEVER hand-edit packages/but-sdk/src/generated (owned by MGMT-IPC-004).
- [STRICTLY] This is the desktop-CT prerequisite for all 38 MGMT component tests — it must produce a RUNNABLE
  harness, not a spec note; if the harness needs more than a config (alias plugin, but-sdk mock provider), build it.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1: 'governance' is a member of ProjectSettingsPageId (tsc accepts it)
- [ ] AC-2: projectSettingsPages has an id='governance', icon:'lock', adminOnly:true entry
- [ ] AC-3 [PRIMARY]: apps/desktop CT harness exists; pnpm test:ct:desktop mounts GovernanceSettings green
- [ ] AC-4: no new top-level route introduced
- [ ] AC-5: no +page.server.ts under governance scope
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (build-gate / structural — INFRA task, no runtime scenario)
--------------------------------------------------------------------------------
AC-1: governance variant in ProjectSettingsPageId union
  GIVEN: apps/desktop/src/lib/state/uiState.svelte.ts defines ProjectSettingsPageId = 'project'|'git'|'ai'|'experimental'
  WHEN:  MGMT-UI-001 lands
  THEN:  'governance' is a member of ProjectSettingsPageId and tsc accepts it as a valid assignment
  TEST_TIER: build-gate   UNIT_TEST_JUSTIFIED: pure type-system structural check — zero runtime I/O
  VERIFY: pnpm -F @gitbutler/desktop check

AC-2: governance page entry in projectSettingsPages with icon:'lock' and adminOnly:true
  GIVEN: projectSettingsPages.ts defines the projectSettingsPages array and SettingsPage requires icon:IconName
  WHEN:  MGMT-UI-001 lands
  THEN:  the array contains id='governance', label='Permissions & Governance', icon:'lock', adminOnly:true
         (satisfies SettingsModalLayout.svelte:53; 'lock' is a verified IconName per packages/ui/src/lib/icons/names.ts)
  TEST_TIER: build-gate   UNIT_TEST_JUSTIFIED: structural source invariant — grep on a config array
  VERIFY: grep -n '"governance"' apps/desktop/src/lib/settings/projectSettingsPages.ts && grep -n 'adminOnly.*true' apps/desktop/src/lib/settings/projectSettingsPages.ts && grep -n "'lock'" apps/desktop/src/lib/settings/projectSettingsPages.ts

AC-3 [PRIMARY]: apps/desktop CT harness scaffolded and pnpm test:ct:desktop executes
  GIVEN: apps/desktop has no playwright-ct.config.ts and no test:ct:desktop script today
  WHEN:  MGMT-UI-001 lands
  THEN:  apps/desktop/playwright-ct.config.ts exists, test:ct:desktop is in apps/desktop/package.json, the
         harness resolves $components/$lib/@gitbutler/* aliases, and a smoke spec mounts GovernanceSettings.svelte
         (stub shell acceptable for this task) asserting it renders without throwing
  TEST_TIER: build-gate   UNIT_TEST_JUSTIFIED: CT harness infrastructure — verifies the toolchain can mount Svelte components in apps/desktop
  VERIFY: pnpm test:ct:desktop -- GovernanceSettings

AC-4: no new top-level route introduced
  GIVEN: apps/desktop/src/routes/ exists
  WHEN:  MGMT-UI-001 lands
  THEN:  no [projectId]/governance or /governance route directory exists; the surface is a state of the settings modal
  TEST_TIER: build-gate   UNIT_TEST_JUSTIFIED: routing invariant — filesystem structural check
  VERIFY: find apps/desktop/src/routes -type d -name 'governance' | wc -l | grep '^0$'

AC-5: no +page.server.ts under governance scope
  GIVEN: adapter-static prohibits SSR server-only files
  WHEN:  MGMT-UI-001 lands
  THEN:  no +page.server.ts/+layout.server.ts exists under any governance path in apps/desktop/src/
  TEST_TIER: build-gate   UNIT_TEST_JUSTIFIED: adapter-static filesystem invariant
  VERIFY: find apps/desktop/src -name '+page.server.ts' -path '*governance*' | wc -l | grep '^0$'

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-2): projectSettingsPages contains exactly one entry id='governance' with icon:'lock' and adminOnly=true
    VERIFY: grep -c '"governance"' apps/desktop/src/lib/settings/projectSettingsPages.ts && grep -n "'lock'" apps/desktop/src/lib/settings/projectSettingsPages.ts
- TC-2 (-> AC-1): pnpm -F @gitbutler/desktop check exits 0 after the union is extended
    VERIFY: pnpm -F @gitbutler/desktop check
- TC-3 (-> AC-3): pnpm test:ct:desktop exits 0 when the GovernanceSettings smoke spec runs
    VERIFY: pnpm test:ct:desktop -- GovernanceSettings
- TC-4 (-> AC-4): no directory named 'governance' under apps/desktop/src/routes/
    VERIFY: find apps/desktop/src/routes -type d -name 'governance' | wc -l | grep '^0$'
- TC-5 (-> AC-1): pnpm lint exits 0 after MGMT-UI-001 changes
    VERIFY: pnpm lint

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01 (indirectly — registers the admin-only surface; enforcement remains server-side but-authz)
provides: ProjectSettingsPageId 'governance' variant; adminOnly page entry with icon:'lock'; the apps/desktop CT harness (T-MGMT-000)
consumes: packages/but-sdk (regenerated by MGMT-IPC-004 — hard predecessor); packages/ui playwright-ct.config.ts pattern;
          apps/desktop/src/lib/state/uiState.svelte.ts; apps/desktop/src/lib/settings/projectSettingsPages.ts
boundary_contracts:
  - the CT harness is the prerequisite for all 38 MGMT component-test criteria; without it the FEATURE UI tasks cannot run.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - apps/desktop/src/lib/settings/projectSettingsPages.ts (+1 governance entry)
  - apps/desktop/src/lib/state/uiState.svelte.ts (extend ProjectSettingsPageId)
  - apps/desktop/playwright-ct.config.ts (NEW)
  - apps/desktop/package.json (add test:ct:desktop)
  - apps/desktop/tests/governance/GovernanceSettings.spec.ts (smoke CT spec — stub shell only for this task)
writeProhibited:
  - packages/but-sdk/src/generated (MGMT-IPC-004)
  - apps/desktop/src/components/settings/ProjectSettingsModalContent.svelte (MGMT-UI-002)
  - any +page.server.ts / +layout.server.ts; any new route directory under apps/desktop/src/routes/

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. apps/desktop/src/lib/settings/projectSettingsPages.ts — projectSettingsPages array + SettingsPage interface to extend (icon:IconName REQUIRED)
2. apps/desktop/src/lib/state/uiState.svelte.ts:21 — ProjectSettingsPageId union literal
3. apps/desktop/src/components/settings/ProjectSettingsModalContent.svelte — consumes the page id (MGMT-UI-002 adds the branch)
4. apps/desktop/src/components/settings/SettingsModalLayout.svelte:53 — pages.filter((p)=>!p.adminOnly||isAdmin)
5. packages/ui/playwright-ct.config.ts — the CT harness pattern to replicate for apps/desktop
6. apps/desktop/package.json — existing test scripts; add test:ct:desktop
7. packages/ui/src/lib/icons/names.ts — authoritative IconName values; 'lock' is confirmed present
8. .spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md#verification-posture — CT-harness prerequisite (B14/T-MGMT-000)

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- pnpm -F @gitbutler/desktop check                                                  -> Exit 0
- pnpm lint                                                                          -> Exit 0
- pnpm test:ct:desktop -- GovernanceSettings                                         -> Exit 0
- grep -n 'governance' apps/desktop/src/lib/state/uiState.svelte.ts                  -> match
- grep -n 'adminOnly.*true' apps/desktop/src/lib/settings/projectSettingsPages.ts    -> match
- grep -n "'lock'" apps/desktop/src/lib/settings/projectSettingsPages.ts             -> match

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: Mirror the existing projectSettingsPages entry pattern; all entries require icon:IconName — use 'lock'
  (verified via grep of packages/ui/src/lib/icons/names.ts). The Playwright CT config mirrors
  packages/ui/playwright-ct.config.ts with apps/desktop alias resolution ($components, $lib, @gitbutler/*).
pattern_source: apps/desktop/src/components/settings/GeneralSettingsModalContent.svelte (isAdmin + SettingsModalLayout pattern)
anti_pattern: adding a new SvelteKit route; adding SSR server files; hand-editing packages/but-sdk/src/generated;
  omitting the icon field (tsc will reject it — SettingsPage interface has icon:IconName as required).

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: sveltekit-implementer (with tauri-* for the but-sdk IPC mock layer if needed)
reviewer: sveltekit-reviewer
coding_standards: apps/desktop/AGENTS.md, frontend.md (no relative imports; PascalCase components; no console.log; adapter-static)

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: MGMT-IPC-004 (regenerated but-sdk the CT mock layer types against)
Blocks:     MGMT-UI-002, MGMT-UI-003, MGMT-UI-005, MGMT-UI-006, MGMT-UI-007, MGMT-UI-008
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "MGMT-UI-001",
  "proposed_by": "sveltekit-planner",
  "verification_policy": { "requires_tests": true, "requires_red_evidence": false, "requires_seeded_evidence": false },
  "fixtures": {},
  "requirements": [
    { "id": "AC-1", "type": "acceptance_criterion", "primary": false, "description": "GIVEN ProjectSettingsPageId = 'project'|'git'|'ai'|'experimental' WHEN MGMT-UI-001 lands THEN 'governance' is a member of the union and tsc accepts it", "verify": "pnpm -F @gitbutler/desktop check" },
    { "id": "AC-2", "type": "acceptance_criterion", "primary": false, "description": "GIVEN the projectSettingsPages array (SettingsPage interface requires icon:IconName) WHEN MGMT-UI-001 lands THEN it contains id='governance', label='Permissions & Governance', icon:'lock', adminOnly:true — 'lock' is a verified IconName (packages/ui/src/lib/icons/names.ts)", "verify": "grep -n '\"governance\"' apps/desktop/src/lib/settings/projectSettingsPages.ts && grep -n 'adminOnly.*true' apps/desktop/src/lib/settings/projectSettingsPages.ts && grep -n \"'lock'\" apps/desktop/src/lib/settings/projectSettingsPages.ts" },
    { "id": "AC-3", "type": "acceptance_criterion", "primary": true, "description": "GIVEN apps/desktop has no CT config today WHEN MGMT-UI-001 lands THEN playwright-ct.config.ts + a test:ct:desktop script exist, aliases resolve, and a smoke spec mounts GovernanceSettings.svelte without throwing", "verify": "pnpm test:ct:desktop -- GovernanceSettings" },
    { "id": "AC-4", "type": "acceptance_criterion", "primary": false, "description": "GIVEN apps/desktop/src/routes/ WHEN MGMT-UI-001 lands THEN no governance route directory was created (state of the settings modal, not a route)", "verify": "find apps/desktop/src/routes -type d -name 'governance' | wc -l | grep '^0$'" },
    { "id": "AC-5", "type": "acceptance_criterion", "primary": false, "description": "GIVEN adapter-static prohibits SSR server files WHEN MGMT-UI-001 lands THEN no +page.server.ts exists under any governance path", "verify": "find apps/desktop/src -name '+page.server.ts' -path '*governance*' | wc -l | grep '^0$'" },
    { "id": "TC-1", "type": "test_criterion", "description": "projectSettingsPages contains exactly one id='governance' entry with icon:'lock' and adminOnly=true", "verify": "grep -c '\"governance\"' apps/desktop/src/lib/settings/projectSettingsPages.ts && grep -n \"'lock'\" apps/desktop/src/lib/settings/projectSettingsPages.ts", "maps_to_ac": "AC-2" },
    { "id": "TC-2", "type": "test_criterion", "description": "pnpm -F @gitbutler/desktop check exits 0 after the union is extended", "verify": "pnpm -F @gitbutler/desktop check", "maps_to_ac": "AC-1" },
    { "id": "TC-3", "type": "test_criterion", "description": "pnpm test:ct:desktop exits 0 when the GovernanceSettings smoke spec runs", "verify": "pnpm test:ct:desktop -- GovernanceSettings", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "no directory named governance under apps/desktop/src/routes/", "verify": "find apps/desktop/src/routes -type d -name 'governance' | wc -l | grep '^0$'", "maps_to_ac": "AC-4" },
    { "id": "TC-5", "type": "test_criterion", "description": "pnpm lint exits 0 after MGMT-UI-001 changes", "verify": "pnpm lint", "maps_to_ac": "AC-1" }
  ]
}
-->
</details>
