# MGMT-UI-004: Wrap GovernanceSettings in the existing shared/ErrorBoundary (no new boundary component)

## What this does

Wrap GovernanceSettings.svelte in the existing shared/ErrorBoundary so governance render/runtime failures show a fallback instead of breaking the settings modal.

## Why

Sprint 06b · PRD UC-MGMT-07 · capability CAP-AUTHZ-01. A Playwright component test mounts GovernanceSettings via a wrapper that causes a child to throw; the ErrorBoundary fallback div (.boundary-error) renders with the title text and the settings modal container is intact. No new GovernanceErro

## How to verify

PRIMARY **AC-1** — `pnpm test:ct:desktop -- GovernanceErrorBoundary`: ErrorBoundary catches governance child throw and renders fallback. Full gate set in the spec below.

## Scope

  - apps/desktop/src/components/settings/GovernanceSettings.svelte (MODIFY — add ErrorBoundary import + wrap only; no logic change)
  - apps/desktop/tests/governance/GovernanceErrorBoundary.spec.ts (NEW — CT spec)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: MGMT-UI-004 — Wrap GovernanceSettings in the existing shared/ErrorBoundary (no new boundary component)
================================================================================

TASK_TYPE:   FEATURE
STATUS:      Backlog
PRIORITY:    P0
EFFORT:      XS  (30 min)
AGENT:       sveltekit-implementer
PROPOSED-BY: sveltekit-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-MGMT-07
CAPABILITIES:CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  pnpm test:ct:desktop -- GovernanceErrorBoundary
  check: pnpm -F @gitbutler/desktop check
  lint:  pnpm lint

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
A Playwright component test mounts GovernanceSettings via a wrapper that causes a child to throw; the ErrorBoundary fallback div (.boundary-error) renders with the title text and the settings modal container is intact. No new GovernanceErrorBoundary.svelte file exists. pnpm -F @gitbutler/desktop check and pnpm lint pass.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] Wrap GovernanceSettings.svelte (or its mount point in ProjectSettingsModalContent.svelte) in the EXISTING apps/desktop/src/components/shared/ErrorBoundary.svelte — import it, compose it, do not re-author it.
- [MUST] The component test must throw a real error from a governance child component (via a prop or $effect that throws) and assert the fallback snippet renders with the title text — proving the boundary actually catches.
- [MUST] The wrapping is the SOLE change to the file(s); no logic, store, or prop is altered.
- [NEVER] NEVER create apps/desktop/src/components/governance/GovernanceErrorBoundary.svelte — the SPRINT.md Coverage Notes explicitly supersede the 10-ui-infrastructure.md net-new list on this point.
- [NEVER] NEVER modify ErrorBoundary.svelte itself — it is read-only reuse.
- [NEVER] NEVER add +page.server.ts or +layout.server.ts (adapter-static constraint).
- [STRICTLY] No relative imports — use @gitbutler/ package references.
- [STRICTLY] No console.log; Prettier: tabs, double quotes, no trailing commas, 100-col.
- [STRICTLY] The CT must exercise a real component throw, not a mocked error handler.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: ErrorBoundary catches governance child throw and renders fallback /* PARTIAL: wrap in place but no CT evidence — see REMEDIATE-06B-B */
- [x] AC-2: No new GovernanceErrorBoundary.svelte file created (build-gate)
- [ ] AC-3: ErrorBoundary wrapper is transparent on normal render (four tab triggers visible, 0 .boundary-error) /* PARTIAL: wrap in place but no CT evidence — see REMEDIATE-06B-B */
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: ErrorBoundary catches governance child throw and renders fallback
  GIVEN: GovernanceSettings is wrapped in shared/ErrorBoundary and a governance child component throws on mount
  WHEN:  the component tree renders
  THEN:  the .boundary-error fallback div renders with a title and the settings modal container remains in the DOM
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- GovernanceErrorBoundary

AC-2: No new GovernanceErrorBoundary.svelte file created (build-gate)
  GIVEN: the apps/desktop/src/components/governance/ directory after MGMT-UI-004 lands
  WHEN:  find is run for GovernanceErrorBoundary.svelte
  THEN:  no such file exists
  TEST_TIER: integration   VERIFICATION_SERVICE: filesystem grep
  UNIT_TEST_JUSTIFIED: UNIT_TEST_JUSTIFIED: This is a filesystem structural invariant — no behavioral scenario; the negation of a prohibited file creation.
  VERIFY: find /Users/justinrich/Projects/gitbutler/apps/desktop/src -name 'GovernanceErrorBoundary.svelte' | wc -l | grep '^0$'

AC-3: ErrorBoundary wrapper is transparent on normal render (four tab triggers visible, 0 .boundary-error)
  GIVEN: GovernanceSettings is wrapped in shared/ErrorBoundary and mounted with no child error (seeded_normal_mount)
  WHEN:  the component renders normally
  THEN:  0 .boundary-error elements in the DOM; exactly 4 tab trigger buttons with accessible names 'Principals', 'Groups', 'Branch Gates', 'Rules' are present in the tab strip
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- GovernanceErrorBoundary

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): A governance child throw causes shared/ErrorBoundary to render the .boundary-error fallback div with title text while the modal container remains intact
    VERIFY: pnpm test:ct:desktop -- GovernanceErrorBoundary
- TC-2 (-> AC-2): No file named GovernanceErrorBoundary.svelte exists anywhere under apps/desktop/src
    VERIFY: find /Users/justinrich/Projects/gitbutler/apps/desktop/src -name 'GovernanceErrorBoundary.svelte' | wc -l | grep '^0$'
- TC-3 (-> AC-3): Normal render (no child error): 0 .boundary-error elements; exactly 4 tab triggers with correct accessible names
    VERIFY: pnpm test:ct:desktop -- GovernanceErrorBoundary

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides:
  - G
  - o
  - v
  - e
  - r
  - n
  - a
  - n
  - c
  - e
  - S
  - e
  - t
  - t
  - i
  - n
  - g
  - s
  - .
  - s
  - v
  - e
  - l
  - t
  - e
  -  
  - w
  - r
  - a
  - p
  - p
  - e
  - d
  -  
  - i
  - n
  -  
  - s
  - h
  - a
  - r
  - e
  - d
  - /
  - E
  - r
  - r
  - o
  - r
  - B
  - o
  - u
  - n
  - d
  - a
  - r
  - y
  -  
  - s
  - o
  -  
  - a
  - n
  - y
  -  
  - r
  - e
  - n
  - d
  - e
  - r
  - /
  - r
  - u
  - n
  - t
  - i
  - m
  - e
  -  
  - f
  - a
  - i
  - l
  - u
  - r
  - e
  -  
  - i
  - n
  -  
  - g
  - o
  - v
  - e
  - r
  - n
  - a
  - n
  - c
  - e
  -  
  - c
  - h
  - i
  - l
  - d
  -  
  - c
  - o
  - m
  - p
  - o
  - n
  - e
  - n
  - t
  - s
  -  
  - s
  - h
  - o
  - w
  - s
  -  
  - t
  - h
  - e
  -  
  - b
  - o
  - u
  - n
  - d
  - a
  - r
  - y
  -  
  - f
  - a
  - l
  - l
  - b
  - a
  - c
  - k
  -  
  - i
  - n
  - s
  - t
  - e
  - a
  - d
  -  
  - o
  - f
  -  
  - b
  - r
  - e
  - a
  - k
  - i
  - n
  - g
  -  
  - t
  - h
  - e
  -  
  - s
  - e
  - t
  - t
  - i
  - n
  - g
  - s
  -  
  - m
  - o
  - d
  - a
  - l
  - ;
  -  
  - t
  - h
  - e
  -  
  - w
  - r
  - a
  - p
  - p
  - i
  - n
  - g
  -  
  - i
  - s
  -  
  - t
  - h
  - e
  -  
  - i
  - n
  - s
  - e
  - r
  - t
  - i
  - o
  - n
  -  
  - p
  - o
  - i
  - n
  - t
  -  
  - M
  - G
  - M
  - T
  - -
  - U
  - I
  - -
  - 0
  - 0
  - 3
  -  
  - d
  - e
  - f
  - e
  - r
  - r
  - e
  - d
  -  
  - t
  - o
  -  
  - S
  - p
  - r
  - i
  - n
  - t
  -  
  - 0
  - 6
  - b
  - .
consumes:
  - MGMT-UI-003 (GovernanceSettings.svelte — the component being wrapped)
  - MGMT-UI-001 (desktop CT harness)
  - apps/desktop/src/components/shared/ErrorBoundary.svelte (read-only: props are children/title/compact, uses svelte:boundary + logError + failed snippet)
boundary_contracts:
  - ErrorBoundary.svelte is REUSED (read-only) — props: children (Snippet), title? (string), compact? (boolean). Uses svelte:boundary with onerror=(e)=>logError(e,{skipToast:true}) and a {#snippet failed(error)} fallback. Do NOT fork or duplicate it.
  - GovernanceSettings.svelte is MODIFIED (wrap only) — no logic change; the ErrorBoundary is the outermost wrapper of its root element or the GovernanceSettings mount site in ProjectSettingsModalContent.svelte.
  - The boundary catches child errors; it does NOT replace the structured IPC-failure banner (that is MGMT-UI-011). It catches synchronous Svelte render throws and runtime errors only.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - apps/desktop/src/components/settings/GovernanceSettings.svelte (MODIFY — add ErrorBoundary import + wrap only; no logic change)
  - apps/desktop/tests/governance/GovernanceErrorBoundary.spec.ts (NEW — CT spec)
writeProhibited:
  - apps/desktop/src/components/shared/ErrorBoundary.svelte — READ-ONLY reuse; do not fork, modify, or extend
  - apps/desktop/src/components/governance/GovernanceErrorBoundary.svelte — explicitly prohibited (SPRINT.md Coverage Notes)
  - Any +page.server.ts or +layout.server.ts — adapter-static constraint
  - apps/desktop/src/components/rules/* — unchanged by this task
  - packages/but-sdk/src/generated — SDK regen is MGMT-IPC-004

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. apps/desktop/src/components/shared/ErrorBoundary.svelte [1-66] — PRIMARY PATTERN — props signature (children, title?, compact?), svelte:boundary onerror, {#snippet failed(error)} fallback structure; understand exactly what to import and wrap.
2. apps/desktop/src/components/settings/GovernanceSettings.svelte [1-50] — The component to be wrapped — identify the outermost element or the mount call site in ProjectSettingsModalContent.svelte where the ErrorBoundary tag goes.
3. apps/desktop/src/components/settings/ProjectSettingsModalContent.svelte [1-80] — The mount site for GovernanceSettings — confirm whether the ErrorBoundary wraps inside GovernanceSettings.svelte itself or at this mount call site.
4. .spec/prds/governance/tasks/sprint-06a-governance-ui-scaffold-principals-groups/MGMT-UI-003-governance-settings-pending-store.md [22-24] — The deferred-wiring note: '06b wraps from outside' — confirms the wrapping strategy and which file receives the change.
5. .spec/prds/governance/08-uc-mgmt.md [156-164] — UC-MGMT-07 AC-1 — the exact acceptance criterion this task closes: error boundary wraps GovernanceSettings.svelte so render/runtime failures display a fallback.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- pnpm test:ct:desktop -- GovernanceErrorBoundary   -> Exit 0
- pnpm -F @gitbutler/desktop check   -> Exit 0
- pnpm lint   -> Exit 0
- find /Users/justinrich/Projects/gitbutler/apps/desktop/src -name 'GovernanceErrorBoundary.svelte' | wc -l | grep '^0$'   -> Exit 0 (prints 0)
- git diff --name-only   -> Only GovernanceSettings.svelte (MODIFY) and the new CT spec file

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references:
  - DESIGN-MGMT-008 (error-boundary fallback + IPC-failure/retry pattern — the visual contract for the fallback state)
  - DESIGN-MGMT-008 AC-1: ErrorBoundary fallback content — wrap GovernanceSettings.svelte in the EXISTING apps/desktop/src/components/shared/ErrorBoundary.svelte with title='Governance settings failed to load', compact=false; no Retry in the boundary fallback; the 'failed' snippet renders error.message as sub-line; the settings modal frame and other sections survive
  - DESIGN-MGMT-008 AC-4: two-category error distinction — render/runtime error is caught by the Svelte boundary (boundary 'failed' snippet); IPC/transport error is handled in-page (danger InfoMessage + Retry in MGMT-UI-011); MGMT-UI-004 implements the boundary wrap ONLY — the IPC-failure banner is implemented in MGMT-UI-011
notes:
  - The ErrorBoundary wraps the GovernanceSettings root. When a child throws, svelte:boundary's onerror calls logError (non-toast) and the {#snippet failed(error)} renders a .boundary-error div with the title prop (defaulting to 'Something went wrong'). The modal container element remains so the user can close the modal. This boundary is for render/runtime errors only — IPC failures are handled separately by the danger InfoMessage + Retry pattern in MGMT-UI-011.
  - MGMT-UI-004's sole responsibility is wrapping GovernanceSettings.svelte in the existing shared/ErrorBoundary — it passes title='Governance settings failed to load' and compact=false as props; no other logic
  - MGMT-UI-004 does NOT handle IPC errors — those are in-page and belong to MGMT-UI-011; MGMT-UI-004 only catches render/runtime throws that bubble up to the Svelte boundary
  - The net-new GovernanceErrorBoundary.svelte listed in the 10-ui-infrastructure.md Net-new components table is SUPERSEDED by this design decision — the existing shared/ErrorBoundary is used; no new boundary component is authored
pattern: Svelte 5 svelte:boundary composition via shared/ErrorBoundary.svelte — import, compose as wrapper around GovernanceSettings children
pattern_source: apps/desktop/src/components/shared/ErrorBoundary.svelte
anti_pattern: Creating a new GovernanceErrorBoundary.svelte; modifying ErrorBoundary.svelte; catching IPC errors here instead of MGMT-UI-011

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agent: sveltekit-implementer
rationale: Minimal Svelte 5 component-wrap task: add one <ErrorBoundary> import and wrap the GovernanceSettings.svelte root in it, then write a Playwright CT proving the fallback renders on thrown error. sveltekit-implementer owns adapter-static component work; the ErrorBoundary itself is read-only.
coding_standards: No relative imports — use @gitbutler/ package references (ESLint enforced), Prettier: tabs, double quotes, no trailing commas, 100-col, No console.log — use console.warn/console.error if needed, Components PascalCase; files kebab-case, Svelte 5 $props() rune syntax; no Options API, CT describe blocks MUST use the component name as the outermost describe string (e.g. describe('GovernanceErrorBoundary', () => {...})) so `pnpm test:ct:desktop -- <ComponentName>` grep matches reliably.

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: MGMT-UI-003 (GovernanceSettings.svelte — the file being modified; from Sprint 06a); MGMT-UI-001 (desktop CT harness — prerequisite for all component tests; from Sprint 06a); DESIGN-MGMT-008 (error-boundary fallback visual contract)
Blocks:     MGMT-UI-011 (a11y + IPC-failure banner + Retry — must run inside a wrapped, boundary-protected GovernanceSettings)
```
</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "MGMT-UI-004",
  "proposed_by": "sveltekit-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "seeded_throw_child": {
      "description": "A minimal governance child stub (inline in the CT) that throws new Error('governance render failure') on mount, mounted inside GovernanceSettings via a prop or slot override; the ErrorBoundary wrapping GovernanceSettings must catch it.",
      "seed_method": "ui_flow",
      "records": [
        "child throws Error('governance render failure') on $effect/onMount"
      ]
    },
    "seeded_normal_mount": {
      "description": "GovernanceSettings mounted with no child error and the governance_status_read SDK mock returning pendingCount=0, hasAdminWrite=true \u2014 baseline to prove the boundary is transparent when no error occurs.",
      "seed_method": "ui_flow",
      "records": [
        "pendingCount=0",
        "hasAdminWrite=true",
        "no child throw"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN GovernanceSettings is wrapped in shared/ErrorBoundary and a governance child component throws on mount WHEN the component tree renders THEN the .boundary-error fallback div renders with a title and the settings modal container remains in the DOM",
      "verify": "pnpm test:ct:desktop -- GovernanceErrorBoundary",
      "scenario": {
        "id": "SC-MGMT-UI-004-1",
        "primary": true,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "GovernanceSettings is NOT wrapped in ErrorBoundary (the throw propagates and the modal breaks) \u2014 a static shell with no real svelte:boundary would pass the throw through unhandled",
            "ErrorBoundary.svelte is stubbed/no-op and does not actually catch errors",
            "the fallback snippet is missing from the real ErrorBoundary (hardcoded empty div)",
            "the test asserts only that no error was thrown (not that the fallback rendered)"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_throw_child",
            "action": {
              "actor": "user",
              "steps": [
                "mount GovernanceSettings (wrapped in ErrorBoundary) with a child that throws Error('governance render failure') on $effect",
                "observe the rendered output"
              ]
            },
            "end_state": {
              "must_observe": [
                "exactly `1` `.boundary-error` element in the DOM (the ErrorBoundary fallback div rendered by the {#snippet failed(error)} block)",
                "the `.boundary-error` element contains text matching `'Something went wrong'` (the default ErrorBoundary title prop) or the custom title passed to ErrorBoundary",
                "exactly `1` settings-modal container element (`ProjectSettingsModalContent` wrapper) still mounted in the DOM alongside the non-governance sidebar sections (accessible names `'Project'`, `'AI options'` still present) \u2014 the modal is NOT unmounted by the child throw"
              ],
              "must_not_observe": [
                "an uncaught JS error propagating to the test runner",
                "the settings modal DOM completely absent (modal broken by the throw)",
                "`0` `.boundary-error` elements when the child has thrown (boundary transparent on error \u2014 wrong)"
              ]
            }
          },
          {
            "start_ref": "seeded_normal_mount",
            "action": {
              "actor": "user",
              "steps": [
                "mount GovernanceSettings (wrapped in ErrorBoundary) with no child error",
                "observe the rendered output"
              ]
            },
            "end_state": {
              "must_observe": [
                "exactly `4` tab TRIGGER elements (buttons with role='tab') with accessible names `'Principals'`, `'Groups'`, `'Branch Gates'`, `'Rules'` \u2014 verifiable from GovernanceSettings tab strip alone before MGMT-UI-009/010 land (tab content is NOT asserted here)",
                "`0` `.boundary-error` elements in the DOM (boundary is transparent when no error)"
              ],
              "must_not_observe": [
                "`.boundary-error` present when no error occurred (boundary rendered fallback incorrectly)",
                "the tab strip absent (governance content not rendered)"
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
      "description": "GIVEN the apps/desktop/src/components/governance/ directory after MGMT-UI-004 lands WHEN find is run for GovernanceErrorBoundary.svelte THEN no such file exists",
      "verify": "find /Users/justinrich/Projects/gitbutler/apps/desktop/src -name 'GovernanceErrorBoundary.svelte' | wc -l | grep '^0$'"
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN GovernanceSettings is wrapped in shared/ErrorBoundary and mounted with no child error (seeded_normal_mount) WHEN the component renders normally THEN 0 .boundary-error elements in the DOM; exactly 4 tab trigger buttons with accessible names 'Principals', 'Groups', 'Branch Gates', 'Rules' are present in the tab strip",
      "verify": "pnpm test:ct:desktop -- GovernanceErrorBoundary",
      "scenario": {
        "id": "SC-MGMT-UI-004-3",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "the ErrorBoundary wraps at the wrong mount point and breaks normal rendering (0 tab triggers visible)",
            "the ErrorBoundary renders the fallback even when no error occurred (`.boundary-error` present on normal render)",
            "the wrap was applied inside a child instead of at the GovernanceSettings root (modal structure broken)",
            "the ErrorBoundary is a stubbed/static shell that hard-codes the fallback visible even with no error (boundary not transparent)"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_normal_mount",
            "action": {
              "actor": "user",
              "steps": [
                "mount GovernanceSettings (wrapped in ErrorBoundary) with no child error",
                "observe the tab strip and boundary state"
              ]
            },
            "end_state": {
              "must_observe": [
                "exactly `4` tab trigger buttons (role='tab') with accessible names `'Principals'`, `'Groups'`, `'Branch Gates'`, `'Rules'`",
                "`0` `.boundary-error` elements in the DOM"
              ],
              "must_not_observe": [
                "`.boundary-error` present when no error occurred",
                "`0` tab triggers (GovernanceSettings root broken by incorrect wrap placement)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "A governance child throw causes shared/ErrorBoundary to render the .boundary-error fallback div with title text while the modal container remains intact",
      "verify": "pnpm test:ct:desktop -- GovernanceErrorBoundary",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "No file named GovernanceErrorBoundary.svelte exists anywhere under apps/desktop/src",
      "verify": "find /Users/justinrich/Projects/gitbutler/apps/desktop/src -name 'GovernanceErrorBoundary.svelte' | wc -l | grep '^0$'",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "Normal render (no child error): 0 .boundary-error elements; exactly 4 tab triggers with correct accessible names",
      "verify": "pnpm test:ct:desktop -- GovernanceErrorBoundary",
      "maps_to_ac": "AC-3"
    }
  ]
}
-->
