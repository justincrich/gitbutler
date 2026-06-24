# LPR-012: `keep_reviews_local` toggle UI in ProjectSettingsModalContent (per-project setting)

> Status: ✅ Completed
> Commit: fefab8bbbb
> Reviewer: deferred to PHASE 4.5 red-hat closeout — committed prior session; keep_reviews_local toggle in Project Settings
> Updated: 2026-06-22T18:07:12Z

## What this does

Add a `keep_reviews_local` Toggle to the Project settings modal's General settings tab
(`apps/desktop/src/components/views/ProjectSettingsModalContent.svelte` → the "project" page hosted by
`GeneralSettings.svelte`). The Toggle reads the current value via the project-settings SDK binding — defaulting
to `true` (local) for any project that does not have the field yet — and writes through the same
project-settings path that sets forge preferences (`forge_override`, `preferred_forge_user`). No
`administration:write` gate applies: this is a per-project operator preference under the R12 trusted-desktop
model (the same class as forge settings). The toggle copy and behavior match DESIGN-LPR-001.

## Why

Sprint 07 · PRD UC-LPR-03 · capability CAP-CONFIG-01. LPR-006 added the `keep_reviews_local: DefaultTrue`
field to `gitbutler_project::Project` and its project-settings write path. This task surfaces that field in the
UI so the desktop operator can inspect and change the per-project routing preference without touching the
backend directly. The forge path is preserved when the flag is false (LPR-006 AC-5), so the Toggle only gates
the default-local behavior — it never touches the merge gate.

## How to verify

PRIMARY **AC-1** — `pnpm test:ct:desktop -- KeepReviewsLocalDefaultTrue`: mounting the toggle without the
`keep_reviews_local` field in the project store (an older project file) renders the Toggle in the ON position.
Full gate set in the spec below.

## Scope

- `apps/desktop/src/components/projectSettings/KeepReviewsLocalToggle.svelte` (NEW — the atom component)
- `apps/desktop/src/components/projectSettings/GeneralSettings.svelte` (MODIFY — mount the new Toggle below
  `ForgeForm` in the General settings layout)
- `apps/desktop/tests/projectSettings/KeepReviewsLocalToggle.spec.ts` (NEW — CT specs)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: LPR-012 — keep_reviews_local toggle UI in ProjectSettingsModalContent
================================================================================

TASK_TYPE:   FEATURE
STATUS:      Backlog
PRIORITY:    P1
EFFORT:      S  (60 min)
AGENT:       sveltekit-implementer
PROPOSED-BY: sveltekit-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-LPR-03
CAPABILITIES:CAP-CONFIG-01

RUNTIME_COMMANDS:
  test:  pnpm test:ct:desktop -- KeepReviewsLocalDefaultTrue
  check: pnpm -F @gitbutler/desktop check
  lint:  pnpm lint

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Mounting KeepReviewsLocalToggle with seeded_no_field_project renders the Toggle
ON (local, the DefaultTrue default). Mounting with seeded_explicit_false renders
it OFF. Toggling ON→OFF calls the project-settings SDK write with
keep_reviews_local=false; toggling OFF→ON calls with keep_reviews_local=true. On
reopen the value reflects the last write. Copy matches DESIGN-LPR-001 verbatim: label
"Keep agent reviews local", on-state description "Agent-authored PRs stay on the local review layer — no GitHub PR is opened. Change this only if you want agent reviews mirrored to your forge. (This is a local project preference, not a security boundary — the project store is not independently verified.)". No administration:write gate fires on the
write. pnpm test:ct:desktop -- KeepReviewsLocalToggle passes. pnpm
-F @gitbutler/desktop check and pnpm lint pass.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] The Toggle default must be ON (local) for any project whose stored JSON
  lacks the keep_reviews_local key — the DefaultTrue serde behavior (LPR-006
  AC-1) is what the backend enforces; the UI must render to match: when the SDK
  returns undefined / null / missing field for keep_reviews_local, treat as true.
- [MUST] Write via the project-settings SDK path — the same call that sets
  forge_override / preferred_forge_user (NOT governance IPC, NOT
  administration:write-gated). The write call must carry exactly
  { keep_reviews_local: boolean }.
- [MUST] After a successful write, the Toggle reflects the new value immediately
  (optimistic update); on a write error, revert to the prior value and surface
  an InfoMessage danger.
- [MUST] On reopen of the Project settings modal, the Toggle reads the persisted
  value from the project store and renders accordingly (the AC-3 "persists + reflects
  on reopen" requirement).
- [MUST] Copy matches DESIGN-LPR-001 VERBATIM: label = 'Keep agent reviews local'; on-state
  description = 'Agent-authored PRs stay on the local review layer — no GitHub PR is opened. Change this only if you want agent reviews mirrored to your forge. (This is a local project preference, not a security boundary — the project store is not independently verified.)'; off-state description = 'Agent-authored PRs will be mirrored to your forge when approved. Internal principal identifiers may be disclosed to the forge API; ensure all principals have forge accounts before enabling. (See: local project preference, not a security boundary.)'
- [NEVER] NEVER route the write through enforce_administration_write_gate or the
  governance IPC channel — keep_reviews_local is a per-project operator preference
  under R12 trusted-desktop, NOT a governed-config mutation.
- [NEVER] NEVER add +page.server.ts or +layout.server.ts.
- [NEVER] NEVER use module-level state.
- [STRICTLY] No relative imports — @gitbutler/ package references. No console.log.
  Prettier: tabs, double quotes, no trailing commas, 100-col.
- [STRICTLY] Svelte 5 $props()/$state()/$derived() rune syntax throughout.
- [STRICTLY] CT describe blocks MUST use the component name as the outermost
  describe string (e.g. describe('KeepReviewsLocalToggle', () => {...})) so
  `pnpm test:ct:desktop -- <ComponentName>` grep matches reliably.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [x] AC-1 [PRIMARY]: default (no field / missing) renders Toggle ON (local)
- [x] AC-2: explicit keep_reviews_local=false renders Toggle OFF
- [x] AC-3: toggling persists to the project store and reflects on reopen
- [x] AC-4: write error reverts the Toggle and surfaces danger InfoMessage
- [x] AC-5: copy matches DESIGN-LPR-001 (label + caption)
- [x] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: default (no keep_reviews_local field / undefined) renders Toggle ON
  GIVEN: KeepReviewsLocalToggle mounted with seeded_no_field_project (the project
         store returns a project JSON without a keep_reviews_local key)
  WHEN:  the component renders
  THEN:  the Toggle has aria-checked='true' (ON / local); the label text is
         "Keep agent reviews local"; the caption is visible and matches the on-state description
         from DESIGN-LPR-001 verbatim;
         0 project-settings write SDK calls fire on render
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- KeepReviewsLocalDefaultTrue

AC-2: explicit keep_reviews_local=false renders Toggle OFF
  GIVEN: KeepReviewsLocalToggle mounted with seeded_explicit_false (the project
         store returns keep_reviews_local=false)
  WHEN:  the component renders
  THEN:  the Toggle has aria-checked='false' (OFF / remote); label and caption
         still present
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- KeepReviewsLocalExplicitFalse

AC-3: toggling persists to the project store and reflects on reopen
  GIVEN: KeepReviewsLocalToggle mounted with seeded_no_field_project (Toggle ON)
  WHEN:  user clicks the Toggle to turn it OFF; then the component is remounted
         with seeded_explicit_false (simulating reopen after successful write)
  THEN:  on first click the project-settings SDK write spy is called == 1 time
         with { keep_reviews_local: false }; after remount with
         seeded_explicit_false the Toggle renders OFF (aria-checked='false')
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- KeepReviewsLocalPersistsOnReopen

AC-4: write error reverts the Toggle and surfaces danger InfoMessage
  GIVEN: KeepReviewsLocalToggle mounted with seeded_no_field_project (Toggle ON),
         seeded_write_error configured (project-settings SDK write returns error)
  WHEN:  user clicks the Toggle to turn it OFF
  THEN:  the project-settings SDK write spy is called == 1 time; the Toggle
         reverts to aria-checked='true' (ON — reverted); a danger InfoMessage
         renders containing error context; 0 additional write calls fire
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- KeepReviewsLocalWriteError

AC-5: copy matches DESIGN-LPR-001
  GIVEN: KeepReviewsLocalToggle mounted with seeded_no_field_project
  WHEN:  the component renders
  THEN:  the visible label text is exactly "Keep agent reviews local"; the on-state description text
         matches DESIGN-LPR-001 VERBATIM: "Agent-authored PRs stay on the local review layer — no GitHub PR
         is opened. Change this only if you want agent reviews mirrored to your forge. (This is a local
         project preference, not a security boundary — the project store is not independently verified.)"
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- KeepReviewsLocalCopy

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): seeded_no_field_project (missing key) → Toggle aria-checked='true';
    label="Keep agent reviews local"; 0 write SDK calls on render
    VERIFY: pnpm test:ct:desktop -- KeepReviewsLocalDefaultTrue
- TC-2 (-> AC-2): seeded_explicit_false → Toggle aria-checked='false'
    VERIFY: pnpm test:ct:desktop -- KeepReviewsLocalExplicitFalse
- TC-3 (-> AC-3): clicking Toggle calls project-settings write with
    {keep_reviews_local:false}; remount with explicit_false shows OFF
    VERIFY: pnpm test:ct:desktop -- KeepReviewsLocalPersistsOnReopen
- TC-4 (-> AC-4): write error → Toggle reverts to ON; danger InfoMessage present;
    no additional write calls
    VERIFY: pnpm test:ct:desktop -- KeepReviewsLocalWriteError
- TC-5 (-> AC-5): label = "Keep agent reviews local"; on-state description matches the DESIGN-LPR-001
    verbatim on-state string (begins "Agent-authored PRs stay on the local review layer — no GitHub PR is opened.")
    VERIFY: pnpm test:ct:desktop -- KeepReviewsLocalCopy

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-CONFIG-01
provides:
  - apps/desktop/src/components/projectSettings/KeepReviewsLocalToggle.svelte —
    a Toggle control reading keep_reviews_local from the project-settings SDK
    binding (DefaultTrue default when missing) and writing via the same
    project-settings path as forge settings (NOT governance IPC, NOT admin-gated)
  - a mount point in GeneralSettings.svelte for the new Toggle (below ForgeForm)
consumes:
  - LPR-006 (Project.keep_reviews_local: DefaultTrue — the backend field this
    Toggle reads/writes; the project-settings write path this Toggle calls)
  - The project-settings SDK function (the same binding forge_override / preferred_forge_user
    uses — MUST be the same call site, NOT but-sdk governance IPC)
  - packages/ui: Toggle, InfoMessage (error state)
  - DESIGN-LPR-001 (verbatim copy contract: label = 'Keep agent reviews local'; on-state and off-state description strings — the implementer MUST grep-match these strings exactly)
boundary_contracts:
  - The write goes through the project-settings path (same as forge settings) —
    no enforce_administration_write_gate, no governance IPC. R12 trusted-desktop.
  - DefaultTrue default: when the SDK returns undefined / null / missing field,
    the Toggle renders ON (local). This mirrors LPR-006 AC-1's serde-default
    behavior at the UI layer.
  - The merge gate is never touched; this Toggle only routes where agent-authored
    review artifacts go, not whether merges are blocked.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - apps/desktop/src/components/projectSettings/KeepReviewsLocalToggle.svelte
    (NEW — the Toggle atom)
  - apps/desktop/src/components/projectSettings/GeneralSettings.svelte (MODIFY —
    mount KeepReviewsLocalToggle below ForgeForm)
  - apps/desktop/tests/projectSettings/KeepReviewsLocalToggle.spec.ts (NEW — CT specs)
writeProhibited:
  - apps/desktop/src/components/views/ProjectSettingsModalContent.svelte — consume-only;
    KeepReviewsLocalToggle is mounted inside GeneralSettings, not directly here
  - apps/desktop/src/components/governance/* — no governance IPC usage
  - apps/desktop/src/components/projectSettings/ForgeForm.svelte — read-only; do not
    modify existing forge settings path
  - packages/but-sdk/src/generated — SDK regen is LPR-010
  - Any +page.server.ts or +layout.server.ts
  - Any file not in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. apps/desktop/src/components/projectSettings/GeneralSettings.svelte [1-18] —
   PRIMARY MOUNT POINT — the four forms (DetailsForm, BaseBranchSwitch,
   GerritForm, ForgeForm, Spacer, RemoveProjectForm) this component orchestrates;
   KeepReviewsLocalToggle mounts below ForgeForm above Spacer.
2. apps/desktop/src/components/projectSettings/ForgeForm.svelte — the existing
   forge-settings form that reads/writes forge_override / preferred_forge_user;
   this is the PATTERN for the project-settings SDK call KeepReviewsLocalToggle
   must reuse.
3. apps/desktop/src/components/governance/PrincipalEditor.svelte [44-80] — Toggle
   import / usage pattern (packages/ui Toggle); InfoMessage danger pattern for
   write errors — mirror this for the revert-and-surface-error flow.
4. .spec/prds/governance/tasks/sprint-07-local-agent-pr/LPR-006-keep-reviews-local-setting.md
   §CRITICAL CONSTRAINTS — the backend field shape, the DefaultTrue default,
   the project-settings write path (forge_override class, NOT admin-gated, R12/R21).
5. DESIGN-LPR-001 (in sprint-07 folder — the UI design contract specifying label,
   caption, and toggle behavior for the keep_reviews_local control).

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- pnpm test:ct:desktop -- KeepReviewsLocalDefaultTrue   -> Exit 0
- pnpm test:ct:desktop -- KeepReviewsLocalExplicitFalse   -> Exit 0
- pnpm test:ct:desktop -- KeepReviewsLocalPersistsOnReopen   -> Exit 0
- pnpm test:ct:desktop -- KeepReviewsLocalWriteError   -> Exit 0
- pnpm test:ct:desktop -- KeepReviewsLocalCopy   -> Exit 0
- pnpm -F @gitbutler/desktop check   -> Exit 0
- pnpm lint   -> Exit 0
- grep -rn 'enforce_administration_write_gate\|governance.*IPC\|governance.*write' \
    /Users/justinrich/Projects/gitbutler/apps/desktop/src/components/projectSettings/KeepReviewsLocalToggle.svelte \
    | wc -l | grep '^0$'   -> Exit 0 (prints 0 — no admin-gate or governance IPC)

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references:
  - DESIGN-LPR-001 (label, caption, and toggle-behavior contract for
    keep_reviews_local in project settings)
  - apps/desktop/src/components/projectSettings/ForgeForm.svelte (the project-
    settings SDK write pattern — same path for keep_reviews_local)
  - apps/desktop/src/components/governance/PrincipalEditor.svelte:44-80 (Toggle
    import + InfoMessage danger revert pattern)
notes:
  - The Toggle is a simple boolean preference — no Modal confirmation needed (it is
    not a destructive action equivalent to "unprotect branch").
  - DefaultTrue UI rule: if the project store returns undefined / null / absent
    for keep_reviews_local, render the Toggle ON. Use `(value ?? true)` as the
    derived boolean binding.
  - Optimistic update: flip the local $state immediately on click; if the write
    call rejects, revert the $state and show InfoMessage danger.
  - The R21 residual (an untrusted project-store write can flip keep_reviews_local
    to false) is a backend-named concern; the UI does not add a guard for it.
pattern: Toggle with DefaultTrue semantic + project-settings SDK write + optimistic
  update with revert-and-InfoMessage on error — mirrors ForgeForm's toggle writes.
pattern_source: apps/desktop/src/components/projectSettings/ForgeForm.svelte (project-
  settings SDK path); packages/ui Toggle + InfoMessage components.
anti_pattern: routing the write through governance IPC or enforce_administration_write_gate;
  defaulting Toggle to OFF when the field is absent; adding +page.server.ts;
  using module-level state; adding a modal confirmation (not destructive, no Modal needed).

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agent: sveltekit-implementer
rationale: A new Toggle atom in the project-settings section of the desktop app,
  consuming an existing project-settings SDK write path (the same forge_override
  path LPR-006 wired). The traps are: (a) the DefaultTrue UI default (absent =
  true, not false), (b) routing the write correctly through the project-settings
  path (not governance IPC), and (c) the optimistic-update + revert pattern.
  sveltekit-implementer owns adapter-static component work for apps/desktop.
coding_standards: No relative imports — @gitbutler/ package references; Prettier
  tabs, double quotes, no trailing commas, 100-col; no console.log; Svelte 5
  $props()/$state()/$derived() rune syntax; CT describe blocks use component name
  as outermost describe string.

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: LPR-006 (Project.keep_reviews_local: DefaultTrue + the project-settings
  write path — the backend field and SDK binding this Toggle reads/writes)
Depends on: DESIGN-LPR-001 (the verbatim label + on-state/off-state description contract; the implementer
  must grep-match these strings in the component; the design contract WINS over any other copy source)
Blocks:     LPR-016 (LocalReviewView reads keep_reviews_local indirectly through
  the local review object — understanding the setting's default is prerequisite context)
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "LPR-012",
  "proposed_by": "sveltekit-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "seeded_no_field_project": {
      "description": "The project-settings SDK mock returns a project object without a keep_reviews_local key (simulating an older project file). The component must treat this as keep_reviews_local=true (local, the DefaultTrue default). The project-settings write SDK function is a spy that resolves successfully.",
      "seed_method": "ui_flow",
      "records": [
        "project store returns: { id: 'proj-1', title: 'my-repo', forge_override: null } (no keep_reviews_local key)",
        "project-settings write SDK spy resolves { ok: true }",
        "DefaultTrue semantic: undefined/null/missing field => treat as true"
      ]
    },
    "seeded_explicit_false": {
      "description": "The project-settings SDK mock returns a project object with keep_reviews_local=false. The component must render the Toggle OFF.",
      "seed_method": "ui_flow",
      "records": [
        "project store returns: { id: 'proj-1', title: 'my-repo', keep_reviews_local: false }",
        "project-settings write SDK spy resolves { ok: true }"
      ]
    },
    "seeded_write_error": {
      "description": "The project-settings SDK write function rejects with an error. Used to verify optimistic-revert behavior.",
      "seed_method": "ui_flow",
      "records": [
        "project store returns: { id: 'proj-1', title: 'my-repo' } (no keep_reviews_local key => default true)",
        "project-settings write SDK spy rejects with Error('project.settings_write_failed')"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN seeded_no_field_project (missing keep_reviews_local key) WHEN KeepReviewsLocalToggle renders THEN Toggle aria-checked='true' (ON/local); label='Keep agent reviews local'; caption visible (on-state description from DESIGN-LPR-001 verbatim); 0 write SDK calls fire on render",
      "verify": "pnpm test:ct:desktop -- KeepReviewsLocalDefaultTrue",
      "scenario": {
        "id": "SC-LPR-012-1",
        "primary": true,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "Toggle renders aria-checked='false' when field is absent (DefaultTrue semantic ignored — wrong default)",
            "0 Toggle elements rendered (component is a static shell with no control)",
            "project-settings write SDK called on render (side-effecting mount)"
          ]
        },
        "evidence": { "artifact_type": "screenshot", "required_capture": true },
        "cases": [
          {
            "start_ref": "seeded_no_field_project",
            "action": { "actor": "user", "steps": [ "mount KeepReviewsLocalToggle with seeded_no_field_project", "observe Toggle state and label" ] },
            "end_state": {
              "must_observe": [
                "Toggle has aria-checked='true' (ON — the DefaultTrue default when field is absent)",
                "visible label text exactly 'Keep agent reviews local' (verbatim per DESIGN-LPR-001)",
                "caption/description text visible (DESIGN-LPR-001)",
                "project-settings write SDK spy call count == 0 on render"
              ],
              "must_not_observe": [
                "Toggle with aria-checked='false' when field is absent (wrong default)",
                "0 Toggle elements (no control rendered)",
                "any SDK write call on mount"
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
      "description": "GIVEN seeded_explicit_false (keep_reviews_local=false) WHEN KeepReviewsLocalToggle renders THEN Toggle aria-checked='false' (OFF/remote); label and caption still present",
      "verify": "pnpm test:ct:desktop -- KeepReviewsLocalExplicitFalse",
      "scenario": {
        "id": "SC-LPR-012-2",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "Toggle renders ON when explicit false is stored (DefaultTrue override ignored)",
            "label absent when field is false (conditional rendering regression)"
          ]
        },
        "evidence": { "artifact_type": "screenshot", "required_capture": true },
        "cases": [
          {
            "start_ref": "seeded_explicit_false",
            "action": { "actor": "user", "steps": [ "mount KeepReviewsLocalToggle with seeded_explicit_false", "observe Toggle state" ] },
            "end_state": {
              "must_observe": [
                "Toggle has aria-checked='false' (OFF)",
                "label and caption still visible"
              ],
              "must_not_observe": [
                "Toggle with aria-checked='true' when keep_reviews_local=false (DefaultTrue override)"
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
      "description": "GIVEN seeded_no_field_project (Toggle ON) WHEN user clicks Toggle OFF THEN project-settings SDK write spy called == 1 time with { keep_reviews_local: false }; remounting with seeded_explicit_false shows Toggle OFF",
      "verify": "pnpm test:ct:desktop -- KeepReviewsLocalPersistsOnReopen",
      "scenario": {
        "id": "SC-LPR-012-3",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "SDK write not called on toggle click (no-op; call count 0)",
            "SDK write called with wrong key (e.g. keep_local, forge_override) — wrong write path",
            "remount with explicit_false still shows Toggle ON (value not persisted)"
          ]
        },
        "evidence": { "artifact_type": "stdout", "required_capture": true },
        "cases": [
          {
            "start_ref": "seeded_no_field_project",
            "action": { "actor": "user", "steps": [ "click the Toggle to turn it OFF", "assert SDK write spy call count and args", "remount with seeded_explicit_false", "assert Toggle is OFF" ] },
            "end_state": {
              "must_observe": [
                "project-settings write SDK spy called == 1 time with payload containing { keep_reviews_local: false }",
                "after remount with seeded_explicit_false: Toggle aria-checked='false'"
              ],
              "must_not_observe": [
                "0 SDK write calls after toggle click (no-op stub)",
                "SDK write called with incorrect payload (wrong key or wrong value)"
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
      "description": "GIVEN seeded_write_error (SDK write rejects) WHEN user clicks Toggle OFF THEN Toggle reverts to ON; danger InfoMessage rendered; 0 additional write calls",
      "verify": "pnpm test:ct:desktop -- KeepReviewsLocalWriteError",
      "scenario": {
        "id": "SC-LPR-012-4",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "Toggle stays OFF after write error (optimistic flip not reverted — stale state)",
            "no InfoMessage danger rendered after error (error swallowed)",
            "additional write calls fired after the first rejection"
          ]
        },
        "evidence": { "artifact_type": "screenshot", "required_capture": true },
        "cases": [
          {
            "start_ref": "seeded_write_error",
            "action": { "actor": "user", "steps": [ "click Toggle to turn OFF (write will reject)", "observe Toggle state and InfoMessage" ] },
            "end_state": {
              "must_observe": [
                "Toggle reverts to aria-checked='true' (ON — revert after error)",
                "InfoMessage with style='danger' containing error context"
              ],
              "must_not_observe": [
                "Toggle remaining aria-checked='false' after write error (no revert)",
                "0 InfoMessage danger elements after the rejected write"
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
      "description": "GIVEN seeded_no_field_project WHEN KeepReviewsLocalToggle renders THEN label='Keep agent reviews local' (verbatim per DESIGN-LPR-001); on-state description text matches the DESIGN-LPR-001 verbatim on-state string: 'Agent-authored PRs stay on the local review layer — no GitHub PR is opened. Change this only if you want agent reviews mirrored to your forge. (This is a local project preference, not a security boundary — the project store is not independently verified.)'",
      "verify": "pnpm test:ct:desktop -- KeepReviewsLocalCopy",
      "scenario": {
        "id": "SC-LPR-012-5",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "label text differs from 'Keep agent reviews local' (copy drift from DESIGN-LPR-001 — the design contract WINS)",
            "caption absent or truncated (design contract not followed)"
          ]
        },
        "evidence": { "artifact_type": "screenshot", "required_capture": true },
        "cases": [
          {
            "start_ref": "seeded_no_field_project",
            "action": { "actor": "user", "steps": [ "mount KeepReviewsLocalToggle", "assert label and caption text" ] },
            "end_state": {
              "must_observe": [
                "element with text exactly 'Keep agent reviews local' (DESIGN-LPR-001 verbatim)",
                "element with text containing 'Agent-authored PRs stay on the local review layer — no GitHub PR is opened.' (DESIGN-LPR-001 on-state verbatim)",
                "element with text containing '(This is a local project preference, not a security boundary — the project store is not independently verified.)' (DESIGN-LPR-001 on-state R21 parenthetical verbatim)"
              ],
              "must_not_observe": [
                "label text differing from 'Keep agent reviews local' (any deviation from DESIGN-LPR-001 verbatim)",
                "caption absent from DOM"
              ]
            }
          }
        ]
      }
    },
    { "id": "TC-1", "type": "test_criterion", "description": "seeded_no_field_project (missing key) → Toggle ON; label present; 0 write calls on render", "verify": "pnpm test:ct:desktop -- KeepReviewsLocalDefaultTrue", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "seeded_explicit_false → Toggle OFF; label/caption present", "verify": "pnpm test:ct:desktop -- KeepReviewsLocalExplicitFalse", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "click Toggle → write spy called == 1 with {keep_reviews_local:false}; remount with explicit_false shows OFF", "verify": "pnpm test:ct:desktop -- KeepReviewsLocalPersistsOnReopen", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "write error → Toggle reverts to ON; danger InfoMessage present", "verify": "pnpm test:ct:desktop -- KeepReviewsLocalWriteError", "maps_to_ac": "AC-4" },
    { "id": "TC-5", "type": "test_criterion", "description": "label='Keep agent reviews local' (DESIGN-LPR-001 verbatim); on-state description element contains the verbatim DESIGN-LPR-001 on-state string beginning 'Agent-authored PRs stay on the local review layer — no GitHub PR is opened.'", "verify": "pnpm test:ct:desktop -- KeepReviewsLocalCopy", "maps_to_ac": "AC-5" }
  ]
}
-->
