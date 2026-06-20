# DESIGN-MGMT-003: Read-only state (disabled-control treatment + `administration:write` info banner)

## What this does

Defines the visual contract for the governance surface when the viewer **can see** the Permissions & Governance page (passed the `adminOnly` sidebar filter) but **lacks** the `administration:write` functional permission: every interactive control is disabled/readonly, an `info` `InfoMessage` explains why, and the commit affordance is hidden. Designed in Sprint 06a so the Principals/Groups controls disable correctly; it also governs the Sprint 06b Branch Gates + Rules controls. **No new design-system work** — `Toggle disabled`, `TagInput readonly`, and `InfoMessage style='info'` already exist.

## Why

Sprint 06a · PRD UC-MGMT-06 ("read-only when the viewer lacks administration:write") · `10-ui-infrastructure.md` Cross-cutting states (ℹ read-only InfoMessage). The read-only state must be distinct from the app-level `adminOnly` sidebar gating (which hides the page entirely) — these are two independent layers.

## How to verify

PRIMARY **AC-1** — design review: every interactive control receives its read-only treatment from the existing component prop (`Toggle disabled=true`, `TagInput readonly=true`, `Button disabled=true`, `SegmentControl` non-interactive, `KebabButton` actions omitted/disabled). Full gate set in the spec below.

## Scope

- `apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md` (MODIFY — extend with the read-only-state section).

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: DESIGN-MGMT-003 — Read-only state (disabled-control treatment + administration:write info banner)
================================================================================

TASK_TYPE:  DESIGN
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     XS  (30 min)
AGENT:      designer=frontend-designer | reviewer=design-reviewer
PROPOSED-BY: frontend-designer
SPRINT:     ./SPRINT.md
PRD_REFS:   UC-MGMT-06
CAPABILITIES: (none — design-spec artifact)

RUNTIME_COMMANDS:
  review: design review of the read-only-state section of DESIGN-ANNOTATIONS.md
  downstream: the MGMT-UI-003 component test asserts an administration:write-lacking viewer sees the read-only InfoMessage and disabled controls (pnpm test:ct:desktop)

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
A sveltekit-implementer reading this contract knows exactly how to derive isReadOnly, which prop to set on
each control type, what InfoMessage variant/text to show, and that the pending-state banner is hidden in
read-only mode — without deciding these independently.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] Specify that every interactive control receives disabled=true (Toggle/Button), readonly=true (TagInput),
  or is non-interactive (SegmentControl) when the viewer lacks administration:write.
- [MUST] Specify the info-variant InfoMessage explaining the read-only condition.
- [MUST] Distinguish read-only (viewer lacks administration:write functional permission) from app-level adminOnly
  sidebar gating (User.role==='admin' hiding the page entirely).
- [MUST] Confirm Branch Gates + Rules controls (Sprint 06b) are also covered by this read-only contract.
- [MUST] State that read-only is applied at the GovernanceSettings.svelte level via a derived isReadOnly prop,
  not per-component.
- [NEVER] Conflate the renderer adminOnly filter (UX-only sidebar gating) with the read-only functional-permission
  state (a viewer who can see the page but lacks administration:write sees read-only controls, not a hidden page).
- [NEVER] Introduce a new grey-overlay/opacity treatment that duplicates Toggle's disabled / TagInput's readonly.
- [NEVER] Introduce new CSS tokens.
- [STRICTLY] InfoMessage style='info' (not 'warning') for the read-only explanation — the viewer is informed, not warned.
- [STRICTLY] Toggle disabled uses the existing :disabled CSS (opacity 0.6, pointer-events none) — no extra styling layer.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: all interactive control types have a stated read-only treatment citing the existing prop
- [ ] AC-2: InfoMessage style='info' outlined with exact text + no action buttons
- [ ] AC-3: adminOnly sidebar gating vs read-only functional-permission explicitly distinguished
- [ ] AC-4: isReadOnly derived at GovernanceSettings.svelte, prop-drilled; pending banner hidden in read-only

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (design-spec; verified by review + the downstream MGMT-UI-003 component test)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: Control-disabled specification
  GIVEN: the read-only contract exists
  WHEN:  a reviewer checks the control-disabled spec
  THEN:  Toggle receives disabled=true; TagInput readonly=true; Button (add/create/save) disabled=true;
         SegmentControl segments non-interactive; KebabButton context actions omitted or disabled — each citing
         the existing prop from the component source
  TEST_TIER: unit   UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test (pnpm test:ct:desktop)
  VERIFY: reviewer confirms each control type (Toggle, TagInput, Button, SegmentControl, KebabButton) has a stated read-only treatment citing the existing prop

AC-2: Explanation banner
  GIVEN: the contract specifies the explanation banner
  WHEN:  a reviewer checks the InfoMessage spec for read-only
  THEN:  it specifies InfoMessage (packages/ui/src/lib/components/InfoMessage.svelte) style='info' outlined=true,
         content 'Read-only: administration:write is required to change governance settings', no action buttons (primaryLabel omitted)
  TEST_TIER: unit   UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test
  VERIFY: reviewer confirms style='info' + outlined=true + the exact content text + absence of action buttons

AC-3: Distinction from adminOnly sidebar gating
  GIVEN: the contract distinguishes read-only from adminOnly gating
  WHEN:  a reviewer inspects the distinction section
  THEN:  it explicitly states (a) adminOnly hides the page from the SettingsModalLayout sidebar for non-admins
         (pages.filter(!p.adminOnly || isAdmin), SettingsModalLayout.svelte:53) — cannot navigate; (b) read-only
         applies to a viewer who CAN navigate but whose administration:write check is false — disabled controls; independent layers
  TEST_TIER: unit   UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test
  VERIFY: reviewer confirms both layers named with source (SettingsModalLayout.svelte:53 for sidebar; administration:write check for read-only)

AC-4: isReadOnly derivation + propagation
  GIVEN: the contract specifies how isReadOnly is derived and propagated
  WHEN:  a reviewer checks the derivation rule
  THEN:  isReadOnly is a boolean derived at GovernanceSettings.svelte level from the administration:write check
         (governed SDK), passed as a prop to PrincipalsList/PrincipalEditor/GroupsList/BranchGatesList — not re-derived
         per child; in read-only mode the GovernancePendingBanner (commit affordance) is hidden
  TEST_TIER: unit   UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test
  VERIFY: reviewer confirms GovernanceSettings.svelte derivation point, prop-drilling, and pending banner hidden in read-only

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): contract names disabled/readonly treatment for Toggle, TagInput, Button, SegmentControl, KebabButton citing existing component props
- TC-2 (-> AC-2): contract names InfoMessage style='info' outlined=true with exact content text and no action buttons
- TC-3 (-> AC-3): contract explicitly distinguishes adminOnly sidebar gating (SettingsModalLayout.svelte:53) from the read-only functional-permission state
- TC-4 (-> AC-4): contract states isReadOnly derived at GovernanceSettings.svelte, prop-drilled to children, pending banner hidden in read-only mode

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: (none — design-spec artifact)
provides: the read-only visual contract consumed by MGMT-UI-003 (+ disabled-control treatment in UI-006/007/008)
consumes: 10-ui-infrastructure.md Cross-cutting states; packages/ui Toggle/TagInput/InfoMessage; SettingsModalLayout.svelte:53 (adminOnly gating reference)

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md (MODIFY — read-only-state section)
writeProhibited:
  - packages/ui/src/lib/components/** ; apps/desktop/src/components/shared/** ; any .svelte or .ts implementation file

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. .spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md — Cross-cutting states (ℹ Read-only InfoMessage wireframe)
2. .spec/prds/governance/08-uc-mgmt.md — UC-MGMT-06 ('read-only when the viewer lacks administration:write')
3. packages/ui/src/lib/components/InfoMessage.svelte — style 'info', outlined; no primaryLabel for read-only
4. packages/ui/src/lib/components/Toggle.svelte — :disabled CSS (opacity 0.6, pointer-events none) ; TagInput.svelte readonly prop
5. apps/desktop/src/components/settings/SettingsModalLayout.svelte:53 — pages.filter((p) => !p.adminOnly || isAdmin) (sidebar gating reference)

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- Design review: all interactive control types have a stated read-only treatment citing the existing component prop
- Design review: InfoMessage style='info' (not 'warning') with no action buttons
- Design review: the adminOnly vs read-only distinction is explicit
- Downstream: the MGMT-UI-003 component test asserts an administration:write-lacking viewer sees the read-only InfoMessage + disabled controls

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: a single isReadOnly boolean derived at GovernanceSettings.svelte level, prop-drilled to all tab content
  components; each component applies disabled=true / readonly=true per control; InfoMessage style='info' in the banner
  slot when isReadOnly is true (mutually exclusive with the pending warning banner).
pattern_source: existing apps/desktop settings sections showing read-only/locked configuration (the adminOnly render pattern in SettingsModalLayout.svelte)
anti_pattern: a grey CSS overlay/opacity wrapper around tab content (let each control's own disabled/readonly handle it); conflating the two gating layers.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
designer: frontend-designer — a cross-cutting visual contract applied once; no backend knowledge required
reviewer: design-reviewer

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: DESIGN-MGMT-001
Blocks:     MGMT-UI-003, MGMT-UI-006, MGMT-UI-007, MGMT-UI-008
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "DESIGN-MGMT-003",
  "proposed_by": "frontend-designer",
  "verification_policy": { "requires_tests": false, "requires_red_evidence": false, "requires_seeded_evidence": false },
  "fixtures": {},
  "requirements": [
    { "id": "AC-1", "type": "acceptance_criterion", "primary": true, "test_tier": "unit", "unit_test_justified": "design-spec artifact, no runtime I/O — verified by design review + downstream component test (pnpm test:ct:desktop)", "description": "GIVEN the read-only contract exists WHEN a reviewer checks the control-disabled spec THEN Toggle disabled=true, TagInput readonly=true, Button disabled=true, SegmentControl non-interactive, KebabButton actions omitted/disabled — each citing the existing prop", "verify": "reviewer confirms each control type (Toggle, TagInput, Button, SegmentControl, KebabButton) has a read-only treatment citing the existing prop" },
    { "id": "AC-2", "type": "acceptance_criterion", "primary": false, "test_tier": "unit", "unit_test_justified": "design-spec artifact, no runtime I/O — verified by design review + downstream component test", "description": "GIVEN the contract specifies the explanation banner WHEN a reviewer checks it THEN it specifies InfoMessage style='info' outlined=true with content 'Read-only: administration:write is required to change governance settings' and no action buttons", "verify": "reviewer confirms style='info' + outlined=true + the exact content text + absence of action buttons" },
    { "id": "AC-3", "type": "acceptance_criterion", "primary": false, "test_tier": "unit", "unit_test_justified": "design-spec artifact, no runtime I/O — verified by design review + downstream component test", "description": "GIVEN the contract distinguishes read-only from adminOnly gating WHEN a reviewer inspects it THEN it states (a) adminOnly hides the page from the sidebar (SettingsModalLayout.svelte:53) and (b) read-only disables controls for a viewer who can navigate but lacks administration:write — independent layers", "verify": "reviewer confirms both layers named with source (SettingsModalLayout.svelte:53; administration:write check)" },
    { "id": "AC-4", "type": "acceptance_criterion", "primary": false, "test_tier": "unit", "unit_test_justified": "design-spec artifact, no runtime I/O — verified by design review + downstream component test", "description": "GIVEN the contract specifies isReadOnly derivation WHEN a reviewer checks it THEN isReadOnly is derived at GovernanceSettings.svelte from the administration:write check, prop-drilled to PrincipalsList/PrincipalEditor/GroupsList/BranchGatesList, and the pending banner is hidden in read-only mode", "verify": "reviewer confirms GovernanceSettings.svelte derivation, prop-drilling, and pending banner hidden in read-only" },
    { "id": "TC-1", "type": "test_criterion", "description": "contract names disabled/readonly treatment for Toggle, TagInput, Button, SegmentControl, KebabButton citing existing props", "verify": "design review of the control-disabled section", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "contract names InfoMessage style='info' outlined=true with exact content text and no action buttons", "verify": "design review of the banner section", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "contract distinguishes adminOnly sidebar gating (SettingsModalLayout.svelte:53) from the read-only functional-permission state", "verify": "design review of the distinction section", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "contract states isReadOnly derived at GovernanceSettings.svelte, prop-drilled to children, pending banner hidden in read-only", "verify": "design review of the derivation rule", "maps_to_ac": "AC-4" }
  ]
}
-->
</details>
