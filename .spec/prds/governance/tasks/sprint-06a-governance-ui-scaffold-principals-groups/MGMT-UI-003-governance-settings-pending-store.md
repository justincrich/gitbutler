# MGMT-UI-003: `GovernanceSettings.svelte` + client-only pending-state store

> **Red-Hat Remediation (cycle 1):** Resolved SEC2 (MEDIUM) — AC-7 adds a build-gate for governance_status_read derivation independence from user.role; AC-8 adds a CT case proving the two signals are independent. Resolved S5 (MEDIUM) — AC-3 extended with governed-commit SDK requirement; AC-9 (seeded_clean banner-absent case). Resolved S7 (MEDIUM) — AC-10 added for cross-tab persistence of the pending store. Resolved S8 (LOW) — deferred-wiring note for GovernanceErrorBoundary added to scope section.

## What this does

The top-level governance page: a four-tab layout (Principals · Groups · Branch Gates · Rules) via the existing `shared/Tabs`, a **CLIENT-ONLY** Svelte pending-state store (no `+page.server.ts`) whose count is derived from the working-tree-vs-target-ref diff (`governance_status_read`, MGMT-IPC-005), the "Commit changes" action (commits `.gitbutler/*.toml` with message `chore: update governance config`), and the read-only state when the viewer lacks `administration:write` (controls disabled + an `info` `InfoMessage`). It owns the pending store lifecycle and propagates `isReadOnly` down to the tab content components.

## Why

Sprint 06a · PRD UC-MGMT-01, UC-MGMT-06 · criteria T-MGMT-004/027/028/035/036 · capability CAP-AUTHZ-01. This is the page shell every Principals/Groups/banner task mounts into, and the cross-cutting pending + read-only contract that keeps the GUI a governed front-end (never a bypass).

## How to verify

PRIMARY **AC-1** — `pnpm test:ct:desktop -- GovernanceSettingsTabs`: mounting `GovernanceSettings.svelte` against the seeded fixture renders four `TabTrigger` elements labeled Principals, Groups, Branch Gates, Rules via the shared `Tabs` components. Full gate set in the spec below.

## Scope

- `apps/desktop/src/components/settings/GovernanceSettings.svelte` (NEW — page + tabs + commit + read-only)
- `apps/desktop/src/lib/governance/pendingStore.svelte.ts` (NEW — CLIENT-ONLY Svelte store)
- `apps/desktop/tests/governance/GovernanceSettings.spec.ts` (NEW — tabs/pending/commit/read-only CT)

**Deferred wiring point (Sprint 06b):** The `GovernanceErrorBoundary` wrapper component will be mounted around `GovernanceSettings` content in Sprint 06b; its insertion point is the `GovernanceSettings` shell root. No re-modification of this file is required when 06b lands — 06b wraps from outside.

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: MGMT-UI-003 — GovernanceSettings.svelte + client-only pending-state store
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     M  (90 min)
AGENT:      implementer=sveltekit-implementer | reviewer=sveltekit-reviewer
PROPOSED-BY: sveltekit-planner
SPRINT:     ./SPRINT.md
PRD_REFS:   UC-MGMT-01, UC-MGMT-06, T-MGMT-004, T-MGMT-027, T-MGMT-028, T-MGMT-035, T-MGMT-036
CAPABILITIES: CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  pnpm test:ct:desktop -- GovernanceSettingsTabs
  check: pnpm -F @gitbutler/desktop check   |   lint: pnpm lint

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Mounting GovernanceSettings with a seeded governance_status_read response (3 pending) renders the warning
banner with the count, the four tabs, and the Commit button. With 0 pending the banner is hidden. With
hasAdminWrite=false, tab-content controls are disabled and an info InfoMessage explains why. The pending
store is CLIENT-ONLY (no +page.server.ts). No direct .gitbutler/*.toml write from the renderer.
hasAdminWrite derives from governance_status_read, not from userService.user?.role — an admin user with
hasAdminWrite=false still sees controls disabled. The store is owned by GovernanceSettings (not per-tab),
so cross-tab navigation preserves the pending count.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] pendingStore is a Svelte store (writable/derived) initialized in a component lifecycle ($effect/onMount)
  — NO module-level shared live state.
- [MUST] pending count derived from the governance_status_read SDK response (working-tree-vs-target-ref diff,
  MGMT-IPC-005), NOT from optimistic local tracking.
- [MUST] the Commit action calls the SDK governance commit fn (message 'chore: update governance config'); on
  success it clears pending; if .gitbutler/*.toml is clean (pendingCount===0) the banner is absent on mount.
- [MUST] administration:write read-only check derives from governance_status_read (hasAdminWrite field), NOT
  from userService.user?.role — a user with role==='admin' but hasAdminWrite===false MUST see controls disabled.
  Verify by grep: no reference to user.role or userService.user?.role in the read-only derivation path.
- [MUST] the pending store is owned by GovernanceSettings (the parent shell), NOT by individual tab components
  — cross-tab navigation must not reset pendingCount.
- [NEVER] NEVER add +page.server.ts / +layout.server.ts (adapter-static).
- [NEVER] NEVER write .gitbutler/*.toml directly from the renderer (no fs.writeFile, no Tauri fs plugin write) —
  every write goes but-sdk -> Tauri -> but-api -> but-authz (T-MGMT-027).
- [NEVER] NEVER optimistically apply a change to the effective display before commit.
- [STRICTLY] Reuse shared/Tabs, InfoMessage, AppScrollableContainer, SettingsSection — no new design-system work.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: four tabs render via the existing shared/Tabs components
- [ ] AC-2: pending banner shows when pendingCount>0 and hides when 0
- [ ] AC-3: commit action calls the SDK governance commit with the correct message and clears pending; banner absent when clean on mount
- [ ] AC-4: read-only state disables controls + shows the info banner when hasAdminWrite=false
- [ ] AC-5: pending store is CLIENT-ONLY — no +page.server.ts
- [ ] AC-6: no direct .gitbutler/*.toml write from the renderer
- [ ] AC-7: hasAdminWrite derives from governance_status_read, NOT user.role (build-gate grep)
- [ ] AC-8: role==='admin' but hasAdminWrite=false -> controls disabled (independence proof)
- [ ] AC-9: with seeded_clean fixture (pendingCount=0) the banner is absent on mount
- [ ] AC-10: cross-tab navigation preserves pendingCount (store owned by parent, not per-tab)
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (behavioral ACs carry scenarios; AC-5/AC-6/AC-7 are build-gate)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: four tabs render using existing shared/Tabs components
  GIVEN: GovernanceSettings mounted with seeded_status_3_pending
  WHEN:  it renders
  THEN:  four TabTrigger elements labeled Principals, Groups, Branch Gates, Rules are present
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- GovernanceSettingsTabs

AC-2: pending banner shows when pendingCount>0 and hides when 0
  GIVEN: GovernanceSettings mounted
  WHEN:  governance_status_read has pendingCount=3 vs 0
  THEN:  with 3 -> GovernancePendingBanner visible with count '3'; with 0 -> banner hidden/absent
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- GovernanceSettingsPendingBanner

AC-3: commit action calls SDK governance commit with the correct message and clears pending; banner absent when clean on mount
  GIVEN: mounted with seeded_status_3_pending (click commit path); AND mounted with seeded_clean (on-mount path)
  WHEN:  (a) user clicks 'Commit changes'; (b) mount with seeded_clean
  THEN:  (a) the SDK governance commit is called with message 'chore: update governance config'; on success
             governance_status_read is re-fetched and pendingCount becomes 0
         (b) GovernancePendingBanner is absent on initial render (pendingCount=0 at mount)
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- GovernanceSettingsCommit

AC-4: read-only state disables controls and shows info banner when viewer lacks administration:write
  GIVEN: mounted with seeded_read_only (hasAdminWrite=false)
  WHEN:  it renders
  THEN:  an info-variant InfoMessage ('Read-only: administration:write is required') is visible; all tab-content controls are disabled
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- GovernanceSettingsReadOnly

AC-5: pending store is CLIENT-ONLY — no +page.server.ts (build-gate)
  GIVEN: apps/desktop/src/ after MGMT-UI-003 lands
  WHEN:  grep for +page.server.ts under governance scope
  THEN:  none exists; pendingStore.svelte.ts uses only client Svelte APIs (writable/derived)
  TEST_TIER: build-gate   UNIT_TEST_JUSTIFIED: adapter-static invariant — filesystem structural check
  VERIFY: find apps/desktop/src -name '+page.server.ts' -path '*governance*' | wc -l | grep '^0$'

AC-6: no direct .gitbutler/*.toml write from renderer (build-gate)
  GIVEN: GovernanceSettings.svelte + governance components
  WHEN:  grep for direct file writes
  THEN:  no fs.writeFile / Tauri fs plugin write / .gitbutler path write is present (writes go via but-sdk)
  TEST_TIER: build-gate   UNIT_TEST_JUSTIFIED: governed-front-end invariant T-MGMT-027
  VERIFY: grep -rn 'gitbutler.*\.toml\|writeFile\|fs.write' apps/desktop/src/components/governance/ apps/desktop/src/components/settings/GovernanceSettings.svelte | grep -v 'but-sdk\|SDK\|import' | wc -l | grep '^0$'

AC-7: hasAdminWrite derives from governance_status_read, NOT user.role (build-gate)
  GIVEN: GovernanceSettings.svelte and pendingStore.svelte.ts after MGMT-UI-003 lands
  WHEN:  grep for user.role / userService.user?.role in the isReadOnly / hasAdminWrite derivation path
  THEN:  no reference to user.role or userService.user?.role exists in the read-only derivation (derivation
         references governance_status_read response or pendingStore, not the user service role field)
  TEST_TIER: build-gate   UNIT_TEST_JUSTIFIED: security invariant — read-only signal must be governed, not role-guessed
  VERIFY: grep -n 'user\.role\|user?\..role\|userService.*role' apps/desktop/src/components/settings/GovernanceSettings.svelte apps/desktop/src/lib/governance/pendingStore.svelte.ts | wc -l | grep '^0$'

AC-8: role==='admin' but governance_status_read hasAdminWrite=false -> controls are still disabled (independence proof)
  GIVEN: GovernanceSettings mounted with seeded_admin_role_no_write (User.role==='admin' in the CT fixture,
         but governance_status_read returns hasAdminWrite=false)
  WHEN:  it renders
  THEN:  controls carry the 'disabled' attribute (same as seeded_read_only); the info InfoMessage is visible
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- GovernanceSettingsAdminRoleNoWrite

AC-9: with seeded_clean fixture (pendingCount=0) banner is absent on mount
  GIVEN: GovernanceSettings mounted with seeded_clean (pendingCount=0, hasAdminWrite=true)
  WHEN:  component mounts
  THEN:  GovernancePendingBanner is absent (0 banner elements in the DOM)
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- GovernanceSettingsCleanMount

AC-10: cross-tab navigation preserves pendingCount (store owned by GovernanceSettings, not per-tab)
  GIVEN: GovernanceSettings mounted with seeded_status_3_pending; user starts on Principals tab
  WHEN:  user clicks the Groups tab
  THEN:  GovernancePendingBanner remains visible with count '3' on the Groups tab (store not reset by tab switch)
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- GovernanceSettingsCrossTabPending

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): GovernanceSettings renders four tab triggers labeled Principals, Groups, Branch Gates, Rules
    VERIFY: pnpm test:ct:desktop -- GovernanceSettingsTabs
- TC-2 (-> AC-2): the banner is visible with count '3' when pendingCount=3 and hidden when pendingCount=0
    VERIFY: pnpm test:ct:desktop -- GovernanceSettingsPendingBanner
- TC-3 (-> AC-3): clicking Commit triggers the SDK governance commit call with message 'chore: update governance config'
    VERIFY: pnpm test:ct:desktop -- GovernanceSettingsCommit
- TC-4 (-> AC-4): with hasAdminWrite=false an info InfoMessage renders and controls are disabled
    VERIFY: pnpm test:ct:desktop -- GovernanceSettingsReadOnly
- TC-5 (-> AC-5): no +page.server.ts exists under any governance path
    VERIFY: find apps/desktop/src -name '+page.server.ts' -path '*governance*' | wc -l | grep '^0$'
- TC-6 (-> AC-7): no user.role reference in the isReadOnly derivation path
    VERIFY: grep -n 'user\.role\|userService.*role' apps/desktop/src/components/settings/GovernanceSettings.svelte apps/desktop/src/lib/governance/pendingStore.svelte.ts | wc -l | grep '^0$'
- TC-7 (-> AC-8): role==='admin' + hasAdminWrite=false -> controls disabled (governance_status_read is the authority)
    VERIFY: pnpm test:ct:desktop -- GovernanceSettingsAdminRoleNoWrite
- TC-8 (-> AC-9): seeded_clean mount -> 0 GovernancePendingBanner elements in the DOM
    VERIFY: pnpm test:ct:desktop -- GovernanceSettingsCleanMount
- TC-9 (-> AC-10): tab switch Principals -> Groups preserves pendingCount=3 in the banner
    VERIFY: pnpm test:ct:desktop -- GovernanceSettingsCrossTabPending

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides: the GovernanceSettings page + the CLIENT-ONLY pending store + commit action + isReadOnly propagation
consumes: MGMT-UI-001 (CT harness), MGMT-UI-002 (rendered via the governance branch), MGMT-IPC-004 (but-sdk
          governance commands: governance_status_read, perm_grant/revoke + commit), MGMT-IPC-005 (pending diff read);
          shared/Tabs, InfoMessage, AppScrollableContainer, SettingsSection
boundary_contracts:
  - pending derived from the working-tree-vs-target-ref diff (T-MGMT-035), CLIENT-ONLY store (T-MGMT-036), no
    direct config write (T-MGMT-027), no optimistic enforcement.
  - hasAdminWrite is governance_status_read-authoritative, NOT role-derived.
  - pendingStore owned by GovernanceSettings (parent), not per-tab.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - apps/desktop/src/components/settings/GovernanceSettings.svelte (NEW)
  - apps/desktop/src/lib/governance/pendingStore.svelte.ts (NEW — CLIENT-ONLY Svelte store)
  - apps/desktop/tests/governance/GovernanceSettings.spec.ts (CT specs)
writeProhibited:
  - any +page.server.ts / +layout.server.ts
  - apps/desktop/src/components/governance/PrincipalsList.svelte (MGMT-UI-006), GroupsList.svelte (MGMT-UI-008),
    GovernancePendingBanner.svelte (MGMT-UI-005)
  - packages/but-sdk/src/generated (MGMT-IPC-004)

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. apps/desktop/src/components/shared/Tabs.svelte — TabList/TabTrigger/TabContent + defaultSelected/writable store
2. packages/ui/src/lib/components/InfoMessage.svelte — warning (pending) + info (read-only) variants
3. apps/desktop/src/components/shared/AppScrollableContainer.svelte / SettingsSection.svelte — layout
4. .spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md#wireframes — cross-cutting states (pending/read-only)
5. .spec/prds/governance/08-uc-mgmt.md#UC-MGMT-06 — commit semantics B15, CLIENT-ONLY pending store, read-only
6. apps/desktop/src/components/settings/ExperimentalSettings.svelte — SettingsSection + content layout pattern

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- pnpm test:ct:desktop -- GovernanceSettingsTabs                  -> Exit 0
- pnpm test:ct:desktop -- GovernanceSettingsPendingBanner         -> Exit 0
- pnpm test:ct:desktop -- GovernanceSettingsCommit                -> Exit 0
- pnpm test:ct:desktop -- GovernanceSettingsReadOnly              -> Exit 0
- pnpm test:ct:desktop -- GovernanceSettingsAdminRoleNoWrite      -> Exit 0
- pnpm test:ct:desktop -- GovernanceSettingsCleanMount            -> Exit 0
- pnpm test:ct:desktop -- GovernanceSettingsCrossTabPending       -> Exit 0
- pnpm -F @gitbutler/desktop check                                -> Exit 0
- pnpm lint                                                        -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references: DESIGN-MGMT-001 (four-tab IA: shared/Tabs defaultSelected='principals'); DESIGN-MGMT-002 (pending banner
  above TabList, hidden at 0); DESIGN-MGMT-003 (isReadOnly derived here, info banner, pending banner hidden in read-only).
interaction_notes: GovernanceSettings owns the CLIENT-ONLY pending store (no +page.server.ts). On mount it calls
  governance_status_read; after any SDK write it re-calls it to refresh pendingCount. The Commit button maps to B15
  commit semantics. isReadOnly propagates as a prop to PrincipalsList/GroupsList/future tabs. The read-only info banner
  and the pending warning banner are mutually exclusive (read-only => info shown, pending hidden).
  SECURITY NOTE: hasAdminWrite is read exclusively from governance_status_read.hasAdminWrite — NEVER from
  userService.user?.role. The governed SDK response is the authority.
  STORE LIFECYCLE NOTE: pendingStore is created in GovernanceSettings' onMount/$effect scope and passed down
  as props/context — it is NOT recreated per tab. Tab navigation must not reset pendingCount.
pattern: layout mirrors existing *Settings.svelte sections; the store mirrors GitButler's client-state convention (a
  writable Svelte store initialized in a component, not module-level).
pattern_source: apps/desktop/src/components/settings/ExperimentalSettings.svelte (SettingsSection layout)
anti_pattern: a module-level store; +page.server.ts; a direct toml write; optimistic enforcement application;
  deriving isReadOnly from userService.user?.role; re-creating the store per tab.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: sveltekit-implementer
reviewer: sveltekit-reviewer
coding_standards: apps/desktop/AGENTS.md, frontend.md (CLIENT-ONLY store; no relative imports; no console.log; adapter-static)

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: MGMT-UI-001, MGMT-UI-002, MGMT-IPC-004, MGMT-IPC-005; DESIGN-MGMT-001, DESIGN-MGMT-002, DESIGN-MGMT-003 (design source)
Blocks:     MGMT-UI-005, MGMT-UI-006, MGMT-UI-008
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "MGMT-UI-003",
  "proposed_by": "sveltekit-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "seeded_status_3_pending": {
      "description": "governance_status_read fixture returns pendingCount 3, hasAdminWrite true, surfaced through the but-sdk mock layer the desktop CT harness mounts against.",
      "seed_method": "ui_flow",
      "records": [
        "pendingCount = 3",
        "hasAdminWrite = true"
      ]
    },
    "seeded_status_0_pending": {
      "description": "governance_status_read fixture returns pendingCount 0, hasAdminWrite true.",
      "seed_method": "ui_flow",
      "records": [
        "pendingCount = 0",
        "hasAdminWrite = true"
      ]
    },
    "seeded_read_only": {
      "description": "governance_status_read fixture returns pendingCount 0, hasAdminWrite false (viewer lacks administration:write).",
      "seed_method": "ui_flow",
      "records": [
        "hasAdminWrite = false"
      ]
    },
    "seeded_admin_role_no_write": {
      "description": "CT fixture sets User.role='admin' in the user context but governance_status_read returns hasAdminWrite=false \u2014 proves isReadOnly derives from governance_status_read, not user.role.",
      "seed_method": "ui_flow",
      "records": [
        "User.role = admin",
        "governance_status_read.hasAdminWrite = false"
      ]
    },
    "seeded_clean": {
      "description": "governance_status_read fixture returns pendingCount 0, hasAdminWrite true. Used to verify GovernancePendingBanner is absent on clean mount.",
      "seed_method": "ui_flow",
      "records": [
        "pendingCount = 0",
        "hasAdminWrite = true"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN GovernanceSettings mounted with seeded_status_3_pending WHEN it renders THEN four TabTrigger elements (Principals, Groups, Branch Gates, Rules) render via shared/Tabs",
      "verify": "pnpm test:ct:desktop -- GovernanceSettingsTabs",
      "scenario": {
        "id": "AC-1",
        "primary": true,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "a custom tab component is used instead of shared/Tabs",
            "a static shell renders no tabs",
            "the tabs are disconnected from the page state"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_status_3_pending",
            "action": {
              "actor": "user",
              "steps": [
                "mount GovernanceSettings against the seeded fixture",
                "observe the tab strip"
              ]
            },
            "end_state": {
              "must_observe": [
                "four TabTrigger elements labeled `\"Principals\"`, `\"Groups\"`, `\"Branch Gates\"`, `\"Rules\"`",
                "`4` tabs rendered via the shared/Tabs components"
              ],
              "must_not_observe": [
                "custom tab markup that differs from shared/Tabs",
                "an empty tab strip (`0` tabs)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN GovernanceSettings mounted WHEN governance_status_read has pendingCount 3 vs 0 THEN with 3 the banner shows count '3' + Commit button, with 0 the banner is hidden",
      "verify": "pnpm test:ct:desktop -- GovernanceSettingsPendingBanner",
      "scenario": {
        "id": "AC-2",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "the pending store is module-level (shared across mounts)",
            "the banner appears when pendingCount=0 (static)",
            "the count is hardcoded"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_status_3_pending",
            "action": {
              "actor": "user",
              "steps": [
                "mount with pendingCount 3",
                "observe the banner"
              ]
            },
            "end_state": {
              "must_observe": [
                "the pending count `3` in the banner text",
                "the `\"Commit changes\"` button visible"
              ],
              "must_not_observe": [
                "`(0)` shown as the pending count",
                "an empty count"
              ]
            }
          },
          {
            "start_ref": "seeded_status_0_pending",
            "action": {
              "actor": "user",
              "steps": [
                "mount with pendingCount 0",
                "observe the banner"
              ]
            },
            "end_state": {
              "must_observe": [
                "the governance tabs still render (`\"Principals\"` present)",
                "`0` GovernancePendingBanner elements in the DOM"
              ],
              "must_not_observe": [
                "`\"0 pending\"` text visible to the user",
                "a banner present at count 0"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN mounted with seeded_status_3_pending WHEN user clicks 'Commit changes' THEN the SDK governance commit is called with message 'chore: update governance config' and on success pendingCount becomes 0; ALSO GIVEN seeded_clean WHEN mounted THEN GovernancePendingBanner is absent",
      "verify": "pnpm test:ct:desktop -- GovernanceSettingsCommit",
      "scenario": {
        "id": "AC-3",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "the commit is issued with a different message",
            "the pending store is not cleared after commit (static)",
            "the commit is a no-op stub",
            "the banner is present at clean mount (pendingCount=0)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_status_3_pending",
            "action": {
              "actor": "user",
              "steps": [
                "click the `\"Commit changes\"` button"
              ]
            },
            "end_state": {
              "must_observe": [
                "the SDK governance commit is called with message `\"chore: update governance config\"`",
                "after success the re-fetched `pendingCount` is `0`"
              ],
              "must_not_observe": [
                "a per-toggle write before commit",
                "optimistic enforcement application",
                "no SDK commit call"
              ]
            }
          },
          {
            "start_ref": "seeded_clean",
            "action": {
              "actor": "user",
              "steps": [
                "mount GovernanceSettings with seeded_clean (pendingCount=0)",
                "observe on initial render"
              ]
            },
            "end_state": {
              "must_observe": [
                "`0` GovernancePendingBanner elements in the DOM on initial mount"
              ],
              "must_not_observe": [
                "a pending banner shown when `pendingCount=0`"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN mounted with seeded_read_only (hasAdminWrite=false) WHEN it renders THEN an info InfoMessage mentioning administration:write shows and interactive controls are disabled",
      "verify": "pnpm test:ct:desktop -- GovernanceSettingsReadOnly",
      "scenario": {
        "id": "AC-4",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "controls remain enabled when hasAdminWrite=false (static)",
            "the info banner is absent",
            "the read-only check is stubbed true"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_read_only",
            "action": {
              "actor": "user",
              "steps": [
                "mount with hasAdminWrite false",
                "observe the tab content"
              ]
            },
            "end_state": {
              "must_observe": [
                "an `info`-variant InfoMessage whose text mentions `\"administration:write\"`",
                "interactive controls carry the `disabled` attribute"
              ],
              "must_not_observe": [
                "editable controls when hasAdminWrite=false",
                "no read-only banner"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN apps/desktop/src after MGMT-UI-003 lands WHEN grep for +page.server.ts under governance THEN none exists; pendingStore uses only client Svelte APIs",
      "verify": "find apps/desktop/src -name '+page.server.ts' -path '*governance*' | wc -l | grep '^0$'"
    },
    {
      "id": "AC-6",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN GovernanceSettings.svelte + governance components WHEN grep for direct file writes THEN none present (writes go via but-sdk) \u2014 T-MGMT-027",
      "verify": "grep -rn 'gitbutler.*\\.toml\\|writeFile\\|fs.write' apps/desktop/src/components/governance/ apps/desktop/src/components/settings/GovernanceSettings.svelte | grep -v 'but-sdk\\|SDK\\|import' | wc -l | grep '^0$'"
    },
    {
      "id": "AC-7",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN GovernanceSettings.svelte and pendingStore.svelte.ts WHEN grepped for user.role / userService.user?.role in the isReadOnly derivation THEN 0 references found \u2014 hasAdminWrite derives from governance_status_read, not the user service role field",
      "verify": "grep -n 'user\\.role\\|userService.*role' apps/desktop/src/components/settings/GovernanceSettings.svelte apps/desktop/src/lib/governance/pendingStore.svelte.ts | wc -l | grep '^0$'"
    },
    {
      "id": "AC-8",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN seeded_admin_role_no_write (User.role==='admin' but governance_status_read.hasAdminWrite=false) WHEN GovernanceSettings renders THEN controls are disabled and the info InfoMessage is visible \u2014 proving governance_status_read is the authority, not user.role",
      "verify": "pnpm test:ct:desktop -- GovernanceSettingsAdminRoleNoWrite",
      "scenario": {
        "id": "AC-8",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "controls are enabled because user.role==='admin' overrides governance_status_read",
            "the derivation uses user.role as the authority",
            "the info banner is absent despite hasAdminWrite=false"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_admin_role_no_write",
            "action": {
              "actor": "user",
              "steps": [
                "mount GovernanceSettings with User.role='admin' but governance_status_read.hasAdminWrite=false",
                "observe the rendered state"
              ]
            },
            "end_state": {
              "must_observe": [
                "interactive controls carry the `disabled` attribute",
                "an `info`-variant InfoMessage mentioning `\"administration:write\"`"
              ],
              "must_not_observe": [
                "controls enabled because user.role='admin'",
                "no info banner (none)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-9",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN seeded_clean (pendingCount=0, hasAdminWrite=true) WHEN GovernanceSettings mounts THEN GovernancePendingBanner is absent (0 elements in the DOM)",
      "verify": "pnpm test:ct:desktop -- GovernanceSettingsCleanMount",
      "scenario": {
        "id": "AC-9",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "the banner is shown at mount when pendingCount=0 (static)",
            "the banner count is hardcoded non-zero",
            "the on-mount governance_status_read is not called (stub)"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_clean",
            "action": {
              "actor": "user",
              "steps": [
                "mount GovernanceSettings against seeded_clean",
                "observe on initial render"
              ]
            },
            "end_state": {
              "must_observe": [
                "`0` GovernancePendingBanner elements in the DOM"
              ],
              "must_not_observe": [
                "a GovernancePendingBanner element when `pendingCount=0`"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-10",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN seeded_status_3_pending with Principals tab active WHEN user clicks the Groups tab THEN GovernancePendingBanner remains visible with count '3' (store owned by GovernanceSettings parent, not per-tab)",
      "verify": "pnpm test:ct:desktop -- GovernanceSettingsCrossTabPending",
      "scenario": {
        "id": "AC-10",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "the pending store is per-tab (module-level or recreated per tab-mount) so pendingCount resets to 0 on tab switch",
            "the banner is a static element disconnected from the store and disappears after tab navigation",
            "the store is a stub that returns 0 on any re-read"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_status_3_pending",
            "action": {
              "actor": "user",
              "steps": [
                "mount GovernanceSettings with pendingCount 3 on Principals tab",
                "click the `\"Groups\"` tab trigger"
              ]
            },
            "end_state": {
              "must_observe": [
                "GovernancePendingBanner still visible with count `3` on the Groups tab"
              ],
              "must_not_observe": [
                "the pending banner absent after tab switch",
                "`0` as the count on the Groups tab"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "GovernanceSettings renders four tab triggers labeled Principals, Groups, Branch Gates, Rules",
      "verify": "pnpm test:ct:desktop -- GovernanceSettingsTabs",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "the banner is visible with count '3' when pendingCount=3 and hidden when pendingCount=0",
      "verify": "pnpm test:ct:desktop -- GovernanceSettingsPendingBanner",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "clicking Commit triggers the SDK governance commit call with message 'chore: update governance config'",
      "verify": "pnpm test:ct:desktop -- GovernanceSettingsCommit",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "with hasAdminWrite=false an info InfoMessage renders and controls are disabled",
      "verify": "pnpm test:ct:desktop -- GovernanceSettingsReadOnly",
      "maps_to_ac": "AC-4"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "no +page.server.ts exists under any governance path",
      "verify": "find apps/desktop/src -name '+page.server.ts' -path '*governance*' | wc -l | grep '^0$'",
      "maps_to_ac": "AC-5"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "description": "no user.role reference in the isReadOnly derivation path",
      "verify": "grep -n 'user\\.role\\|userService.*role' apps/desktop/src/components/settings/GovernanceSettings.svelte apps/desktop/src/lib/governance/pendingStore.svelte.ts | wc -l | grep '^0$'",
      "maps_to_ac": "AC-7"
    },
    {
      "id": "TC-7",
      "type": "test_criterion",
      "description": "User.role==='admin' + governance_status_read.hasAdminWrite=false -> controls disabled",
      "verify": "pnpm test:ct:desktop -- GovernanceSettingsAdminRoleNoWrite",
      "maps_to_ac": "AC-8"
    },
    {
      "id": "TC-8",
      "type": "test_criterion",
      "description": "seeded_clean mount -> 0 GovernancePendingBanner elements in the DOM",
      "verify": "pnpm test:ct:desktop -- GovernanceSettingsCleanMount",
      "maps_to_ac": "AC-9"
    },
    {
      "id": "TC-9",
      "type": "test_criterion",
      "description": "tab switch Principals -> Groups preserves pendingCount=3 in the banner",
      "verify": "pnpm test:ct:desktop -- GovernanceSettingsCrossTabPending",
      "maps_to_ac": "AC-10"
    }
  ]
}
-->
</details>
