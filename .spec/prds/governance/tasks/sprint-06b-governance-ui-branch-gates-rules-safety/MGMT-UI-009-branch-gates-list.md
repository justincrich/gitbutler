# MGMT-UI-009: BranchGatesList (ExpandableSection per branch; required-group selector = defined groups)

## What this does

Build BranchGatesList.svelte as the Branch Gates tab content: ExpandableSection per configured branch with all four gate fields, a confirmation-gated unprotect flow, a group-constrained required-group selector, empty state, and pending indicators after each write.

## Why

Sprint 06b · PRD UC-MGMT-04 · capability CAP-AUTHZ-01, CAP-CONFIG-01. Mounting BranchGatesList with seeded_gates_two_branches renders two ExpandableSection rows; expanding 'main' shows the four gate fields with their seeded values; toggling protected OFF shows the Modal confirmation; confirming calls branch_g

## How to verify

PRIMARY **AC-1** — `pnpm test:ct:desktop -- BranchGatesListRows`: Gate rows render with seeded field values. Full gate set in the spec below.

## Scope

- apps/desktop/src/components/governance/BranchGatesList.svelte (NEW)
- apps/desktop/tests/governance/BranchGatesList.spec.ts (NEW — CT specs)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: MGMT-UI-009 — BranchGatesList (ExpandableSection per branch; required-group selector = defined groups)
================================================================================

TASK_TYPE:   FEATURE
STATUS:      Backlog
PRIORITY:    P1
EFFORT:      M  (75 min)
AGENT:       sveltekit-implementer
PROPOSED-BY: sveltekit-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-MGMT-04
CAPABILITIES:CAP-AUTHZ-01, CAP-CONFIG-01

RUNTIME_COMMANDS:
  test:  pnpm test:ct:desktop -- BranchGatesListRows
  check: pnpm -F @gitbutler/desktop check
  lint:  pnpm lint

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Mounting BranchGatesList with seeded_gates_two_branches renders two ExpandableSection rows; expanding 'main' shows the four gate fields with their seeded values; toggling protected OFF shows the Modal confirmation; confirming calls branch_gates_update with protected:false; the row gains a pending indicator. Re-protecting (toggling protected ON from an unprotected state) calls branch_gates_update with protected:true immediately without a Modal (re-protect is non-destructive); the Toggle ends at aria-checked='true' and no danger InfoMessage appears. Required-group selector options match seeded_defined_groups only. With seeded_empty_gates, EmptyStatePlaceholder renders. pnpm test:ct:desktop -- BranchGatesList passes. pnpm -F @gitbutler/desktop check and pnpm lint pass.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] Every gate write calls branch_gates_update via but-sdk — never a direct .gitbutler/gates.toml write from the renderer.
- [MUST] Unprotecting a branch (Toggle protected OFF) must show a Modal confirmation dialog before the SDK write is staged — the Toggle must NOT flip until the user confirms.
- [MUST] The required-group selector must offer ONLY groups defined in the Groups tab (sourced from branch_gates_read or a shared groups query); offering undefined groups is a correctness violation.
- [MUST] isReadOnly=true must disable all Toggles, Textboxes, and the group selector with zero SDK calls on any interaction.
- [MUST] EmptyStatePlaceholder renders when branch_gates_read returns an empty list.
- [NEVER] NEVER write .gitbutler/gates.toml directly from the renderer.
- [NEVER] NEVER flip the protected Toggle optimistically before the Modal is confirmed.
- [NEVER] NEVER add +page.server.ts or +layout.server.ts.
- [NEVER] NEVER offer undefined groups in the required-group selector.
- [STRICTLY] Mirror the ExpandableSection pattern from GroupsList.svelte (MGMT-UI-008) — same expand/collapse lifecycle, same isReadOnly propagation.
- [STRICTLY] No relative imports — @gitbutler/ package references. No console.log. Prettier: tabs, double quotes, no trailing commas, 100-col.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: Gate rows render with seeded field values
- [ ] AC-2: Gate field edit calls branch_gates_update and shows pending
- [ ] AC-3: Add gate for new branch and EmptyStatePlaceholder when none configured
- [ ] AC-4: Required-group selector offers only defined groups
- [ ] AC-5: Unprotect branch requires Modal confirmation before staged write
- [ ] AC-6: isReadOnly=true disables ALL gate field controls and fires 0 SDK calls on interaction (T-MGMT-029)
- [ ] AC-7: Denied write (branch_gates_update returns perm.denied) surfaces danger InfoMessage without flipping the control (DESIGN-MGMT-004 consumer proof)
- [ ] AC-8: Re-protect branch (toggle protected ON) is silent — no Modal, immediate write with protected:true, Toggle ends at aria-checked='true', no danger InfoMessage (REMEDIATE-UI-6, closes red-hat L2)
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: Gate rows render with seeded field values
  GIVEN: BranchGatesList mounted with seeded_gates_two_branches
  WHEN:  user expands the 'main' row
  THEN:  four gate fields render: protected Toggle ON, min_approvals Textbox showing '2', require_distinct_from_author Toggle ON, require_approval_from_group chips showing 'eng' and 'security'
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- BranchGatesListRows

AC-2: Gate field edit calls branch_gates_update and shows pending
  GIVEN: BranchGatesList mounted with seeded_gates_two_branches, main row expanded
  WHEN:  user changes min_approvals Textbox from '2' to '3'
  THEN:  branch_gates_update is called with the updated field value and the 'main' row gains a pending indicator (○ Badge or pending state marker)
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- BranchGatesListEdit

AC-3: Add gate for new branch and EmptyStatePlaceholder when none configured
  GIVEN: BranchGatesList mounted with seeded_empty_gates
  WHEN:  component renders; then user clicks '+ Add'
  THEN:  EmptyStatePlaceholder renders initially; clicking '+ Add' shows an add-gate form; confirming with pattern 'staging' calls branch_gates_update with a new gate entry for 'staging'
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- BranchGatesListEmpty

AC-4: Required-group selector offers only defined groups
  GIVEN: BranchGatesList mounted with seeded_gates_two_branches (defined_groups=['eng','security','platform']), main row expanded
  WHEN:  user opens the required-group selector dropdown
  THEN:  options are exactly ['eng','security','platform'] — no undefined or fabricated group names
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- BranchGatesListGroupSelector

AC-5: Unprotect branch requires Modal confirmation before staged write
  GIVEN: BranchGatesList mounted with seeded_gates_two_branches (main protected:true), main row expanded
  WHEN:  user toggles the protected Toggle OFF
  THEN:  a Modal confirmation dialog appears with text 'Unprotect branch main? Merges will no longer require review.' before the write is staged; confirming calls branch_gates_update with protected:false; cancelling leaves the Toggle ON and makes no SDK call
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- BranchGatesListUnprotectConfirm

AC-6: isReadOnly=true disables ALL gate field controls and fires 0 SDK calls on interaction (T-MGMT-029)
  GIVEN: BranchGatesList mounted with seeded_gates_readonly (isReadOnly=true)
  WHEN:  user attempts to interact with any Toggle, Textbox, Select, TagInput, or write Button
  THEN:  every Toggle has aria-disabled=true or disabled attribute; every Textbox input has disabled attribute; every TagInput/Select is readonly or disabled; every write Button has disabled attribute; 0 branch_gates_update SDK spy calls fire on any interaction
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- BranchGatesListReadOnly

AC-7: Denied write (branch_gates_update returns perm.denied) surfaces danger InfoMessage without flipping the control (DESIGN-MGMT-004 consumer proof)
  GIVEN: BranchGatesList mounted with seeded_gates_two_branches, main row expanded, seeded_write_denied configured
  WHEN:  user changes min_approvals Textbox from '2' to '3' (triggering branch_gates_update which returns perm.denied)
  THEN:  a danger InfoMessage appears with 'perm.denied'/'Permission denied' text; the min_approvals Textbox reverts to showing '2' (the control does NOT reflect the denied change)
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- BranchGatesListWriteDenied

AC-8: Re-protect branch (toggle protected ON) is silent — no Modal, immediate write, no danger InfoMessage (REMEDIATE-UI-6, closes red-hat L2)
  GIVEN: BranchGatesList mounted with seeded_gates_two_branches, a branch currently unprotected (protected:false)
  WHEN:  user toggles the protected Toggle ON
  THEN:  no Modal confirmation appears (re-protect is non-destructive); branch_gates_update is called once with protected:true; the Toggle ends at aria-checked='true'; no danger InfoMessage appears
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- BranchGatesListReprotect

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): Mounting with seeded_gates_two_branches and expanding 'main' renders all four gate field controls with seeded values (protected ON, min_approvals '2', distinct ON, groups 'eng'+'security')
    VERIFY: pnpm test:ct:desktop -- BranchGatesListRows
- TC-2 (-> AC-2): Changing min_approvals from '2' to '3' calls branch_gates_update with the updated value and the row gains a pending indicator
    VERIFY: pnpm test:ct:desktop -- BranchGatesListEdit
- TC-3 (-> AC-3): With no gates configured, EmptyStatePlaceholder renders; '+ Add' flow calls branch_gates_update with the new pattern
    VERIFY: pnpm test:ct:desktop -- BranchGatesListEmpty
- TC-4 (-> AC-4): Required-group selector dropdown contains exactly the 3 defined groups and no undefined names
    VERIFY: pnpm test:ct:desktop -- BranchGatesListGroupSelector
- TC-5 (-> AC-5): Toggling protected OFF shows Modal with exact text before any write; Cancel fires 0 SDK calls and leaves Toggle ON; Confirm calls branch_gates_update with protected:false
    VERIFY: pnpm test:ct:desktop -- BranchGatesListUnprotectConfirm
- TC-6 (-> AC-6): isReadOnly=true: every Toggle/Textbox/Button disabled; 0 branch_gates_update SDK calls fire on interaction
    VERIFY: pnpm test:ct:desktop -- BranchGatesListReadOnly
- TC-7 (-> AC-7): branch_gates_update returning perm.denied: danger InfoMessage appears; control reverts to original value (no flip)
    VERIFY: pnpm test:ct:desktop -- BranchGatesListWriteDenied
- TC-8 (-> AC-8): Toggling protected ON from an unprotected state fires branch_gates_update with protected:true immediately (no Modal), Toggle ends at aria-checked='true', and no danger InfoMessage appears
    VERIFY: pnpm test:ct:desktop -- BranchGatesListReprotect

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01, CAP-CONFIG-01
provides:
  - a
  - p
  - p
  - s
  - /
  - d
  - e
  - s
  - k
  - t
  - o
  - p
  - /
  - s
  - r
  - c
  - /
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
  - /
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
  - /
  - B
  - r
  - a
  - n
  - c
  - h
  - G
  - a
  - t
  - e
  - s
  - L
  - i
  - s
  - t
  - .
  - s
  - v
  - e
  - l
  - t
  - e
  -
  - —
  -
  - B
  - r
  - a
  - n
  - c
  - h
  -
  - G
  - a
  - t
  - e
  - s
  -
  - t
  - a
  - b
  -
  - c
  - o
  - n
  - t
  - e
  - n
  - t
  - :
  -
  - E
  - x
  - p
  - a
  - n
  - d
  - a
  - b
  - l
  - e
  - S
  - e
  - c
  - t
  - i
  - o
  - n
  -
  - p
  - e
  - r
  -
  - b
  - r
  - a
  - n
  - c
  - h
  -
  - w
  - i
  - t
  - h
  -
  - p
  - r
  - o
  - t
  - e
  - c
  - t
  - e
  - d
  -
  - T
  - o
  - g
  - g
  - l
  - e
  - ,
  -
  - m
  - i
  - n
  - _
  - a
  - p
  - p
  - r
  - o
  - v
  - a
  - l
  - s
  -
  - T
  - e
  - x
  - t
  - b
  - o
  - x
  -
  - (
  - n
  - u
  - m
  - b
  - e
  - r
  - )
  - ,
  -
  - r
  - e
  - q
  - u
  - i
  - r
  - e
  - _
  - d
  - i
  - s
  - t
  - i
  - n
  - c
  - t
  - _
  - f
  - r
  - o
  - m
  - _
  - a
  - u
  - t
  - h
  - o
  - r
  -
  - T
  - o
  - g
  - g
  - l
  - e
  - ,
  -
  - r
  - e
  - q
  - u
  - i
  - r
  - e
  - _
  - a
  - p
  - p
  - r
  - o
  - v
  - a
  - l
  - _
  - f
  - r
  - o
  - m
  - _
  - g
  - r
  - o
  - u
  - p
  -
  - T
  - a
  - g
  - I
  - n
  - p
  - u
  - t
  - /
  - S
  - e
  - l
  - e
  - c
  - t
  -
  - s
  - c
  - o
  - p
  - e
  - d
  -
  - t
  - o
  -
  - d
  - e
  - f
  - i
  - n
  - e
  - d
  -
  - g
  - r
  - o
  - u
  - p
  - s
  -
  - o
  - n
  - l
  - y
  - ;
  -
  - u
  - n
  - p
  - r
  - o
  - t
  - e
  - c
  - t
  -
  - c
  - o
  - n
  - f
  - i
  - r
  - m
  - a
  - t
  - i
  - o
  - n
  -
  - M
  - o
  - d
  - a
  - l
  - ;
  -
  - E
  - m
  - p
  - t
  - y
  - S
  - t
  - a
  - t
  - e
  - P
  - l
  - a
  - c
  - e
  - h
  - o
  - l
  - d
  - e
  - r
  -
  - w
  - h
  - e
  - n
  -
  - n
  - o
  -
  - g
  - a
  - t
  - e
  - s
  - ;
  -
  - p
  - e
  - n
  - d
  - i
  - n
  - g
  -
  - s
  - t
  - a
  - t
  - e
  -
  - a
  - f
  - t
  - e
  - r
  -
  - e
  - a
  - c
  - h
  -
  - g
  - a
  - t
  - e
  - -
  - c
  - o
  - n
  - f
  - i
  - g
  -
  - w
  - r
  - i
  - t
  - e
  - ;
  -
  - i
  - s
  - R
  - e
  - a
  - d
  - O
  - n
  - l
  - y
  - -
  - a
  - w
  - a
  - r
  - e
  - .
consumes:
  - MGMT-BE-004 (branch_gates_read / branch_gates_update SDK functions + gate field types: protected, min_approvals, require_distinct_from_author, require_approval_from_group)
  - MGMT-UI-001 (desktop CT harness)
  - MGMT-UI-003 (GovernanceSettings isReadOnly prop + pending store context)
  - DESIGN-MGMT-006 (empty state for Branch Gates tab)
  - DESIGN-MGMT-007 (four-tab IA + aria contract)
  - packages/ui: Toggle, Textbox, TagInput/Select, InfoMessage, EmptyStatePlaceholder, Modal, Button, Badge
  - apps/desktop/src/components/shared/ExpandableSection.svelte (PRIMARY PATTERN from MGMT-UI-008)
  - groups list SDK call (same endpoint Groups tab uses — provides the defined_groups prop; sourced separately from branch_gates_read)
boundary_contracts:
  - All gate writes go through branch_gates_update SDK call — never a direct .gitbutler/gates.toml write from the renderer (T-MGMT-027).
  - The required-group selector options are sourced from a SEPARATE groups SDK query (the same call the Groups tab uses to list defined groups), NOT from branch_gates_read. BranchGatesList must issue two SDK calls: branch_gates_read for gate data and a separate groups list call for selector options. A gate cannot require a group not in that list (T-MGMT-020).
  - Unprotecting a branch (toggle protected OFF) MUST show a Modal confirmation ('Unprotect branch main? Merges will no longer require review.') before staging the write (T-MGMT-047 / B17).
  - After each write, a pending indicator (○ Badge or pending flag) appears on the row and the GovernancePendingBanner count increments (working-tree-vs-target-ref diff drives this via the existing pendingStore).
  - This component is the SPEC-SANCTIONED seam (B14): desktop component tests render the real BranchGatesList.svelte with the Tauri IPC transport mocked at the but-sdk seam. The real gates.toml enforcement is proven by MGMT-BE-004's Rust integration tests. The CT proves wiring/state/pending-surfacing against real Svelte components.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - apps/desktop/src/components/governance/BranchGatesList.svelte (NEW)
  - apps/desktop/tests/governance/BranchGatesList.spec.ts (NEW — CT specs)
writeProhibited:
  - apps/desktop/src/components/rules/* — unchanged by this task
  - apps/desktop/src/components/shared/ExpandableSection.svelte — read-only reuse
  - packages/ui/src/lib/components/* — read-only reuse
  - packages/but-sdk/src/generated — SDK regen is MGMT-IPC-004 / MGMT-BE-004
  - Any +page.server.ts or +layout.server.ts
  - .gitbutler/*.toml — no direct renderer writes

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. apps/desktop/src/components/governance/GroupsList.svelte [1-100] — PRIMARY PATTERN — ExpandableSection per group, isReadOnly propagation, Modal confirmation before destructive SDK call; BranchGatesList mirrors this structure. FALLBACK IF NOT YET LANDED: use apps/desktop/src/components/shared/ExpandableSection.svelte (lines 1-60) + packages/ui/src/lib/components/Modal.svelte (lines 1-50) as the pattern reference instead.
2. apps/desktop/src/components/shared/ExpandableSection.svelte [1-60] — Props and slot API for the ExpandableSection reused per-branch-row.
3. packages/ui/src/lib/components/Modal.svelte [1-50] — Props/slot API for the unprotect confirmation dialog (B17 pattern).
4. .spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md [83-97] — Branch Gates wireframe — exact field layout, pending state display, empty state text.
5. .spec/prds/governance/08-uc-mgmt.md [107-117] — UC-MGMT-04 acceptance criteria — the exact behavioral contract this component closes.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- pnpm test:ct:desktop -- BranchGatesListRows   -> Exit 0
- pnpm test:ct:desktop -- BranchGatesListEdit   -> Exit 0
- pnpm test:ct:desktop -- BranchGatesListEmpty   -> Exit 0
- pnpm test:ct:desktop -- BranchGatesListGroupSelector   -> Exit 0
- pnpm test:ct:desktop -- BranchGatesListUnprotectConfirm   -> Exit 0
- pnpm test:ct:desktop -- BranchGatesListReprotect   -> Exit 0
- pnpm -F @gitbutler/desktop check   -> Exit 0
- pnpm lint   -> Exit 0
- grep -rn 'gates\.toml\|writeFile\|fs\.write' /Users/justinrich/Projects/gitbutler/apps/desktop/src/components/governance/BranchGatesList.svelte | grep -v 'but-sdk\|SDK\|import' | wc -l | grep '^0$'   -> Exit 0 (prints 0)

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references:
  - DESIGN-MGMT-006 (empty states for all four tabs — Branch Gates empty state)
  - DESIGN-MGMT-007 (four-tab IA + aria + keyboard-nav contract)
  - DESIGN-MGMT-006 AC-1: Branch Gates EmptyStatePlaceholder (packages/ui/src/lib/components/EmptyStatePlaceholder.svelte) with title='No branch gates configured.', caption, Button '+ Add' primary action, disabled=true when isReadOnly=true
  - DESIGN-MGMT-004 AC-1/AC-2/AC-3: denial banner + no-flip rule applies to Branch Gates controls (Toggle for Protected branch, Textbox for min_approvals, TagInput for required groups) — if the viewer attempts a self-escalation from ANY tab the denial banner appears and the control reverts
  - DESIGN-MGMT-003 (Sprint 06a): isReadOnly=true received as prop from GovernanceSettings.svelte disables all BranchGatesList controls: Toggle disabled=true, Textbox disabled=true, TagInput readonly=true, Button disabled=true; the read-only InfoMessage is shown in the GovernanceSettings banner slot (not per-row)
  - .spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md Branch Gates tab wireframe (lines 83-97): ExpandableSection per branch with Protected Toggle, min_approvals Textbox type=number, distinct Toggle, required-groups TagInput; empty state; pending Badge warning/soft (DESIGN-MGMT-002 from Sprint 06a) on uncommitted gate changes
notes:
  - BranchGatesList receives the branch gate list from branch_gates_read and the defined groups list (same source as GroupsList). Each branch is an ExpandableSection. On protected Toggle OFF: show Modal, await confirm/cancel before any SDK call. On other field change: immediate branch_gates_update call, then re-fetch via pendingStore. isReadOnly disables all controls. The required-group TagInput/Select options are computed from the defined_groups passed in as a prop (sourced upstream from the same groups query).
  - Branch Gates ExpandableSection rows show a pending Badge (Badge style='warning' kind='soft') when the gate has uncommitted working-tree changes, per DESIGN-MGMT-002; no pending Badge when committed
  - Unprotect-branch confirmation: turning Protected toggle OFF must surface a Modal confirmation ('Unprotect branch main? Merges will no longer require review.') before staging the gate-config write — the Toggle does NOT revert; it waits for confirmation and only stages on confirm or reverts on cancel (this is a destructive-confirmation, not a denial/no-flip)
  - The required-group TagInput must source its options ONLY from the groups defined in the Groups tab (consistent group set per T-MGMT-020); options not in the defined groups are not offered
  - In read-only mode: unprotect-branch confirmation dialog is not triggered (Toggle is disabled); all add/edit affordances are disabled
pattern: ExpandableSection per item with immediate SDK call on field change — mirrors GroupsList (MGMT-UI-008). Destructive action (unprotect) guarded by Modal confirmation (B17).
pattern_source: apps/desktop/src/components/shared/ExpandableSection.svelte + packages/ui/src/lib/components/Modal.svelte (EXISTING fallback if MGMT-UI-008 GroupsList.svelte has not yet landed)
anti_pattern: Writing gates.toml directly; offering undefined groups in the selector; flipping the Toggle before Modal confirm; batching gate writes behind a Save button (not the gates model)

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agent: sveltekit-implementer
rationale: Net-new Svelte component consuming the MGMT-BE-004 SDK delta; mirrors the GroupsList ExpandableSection pattern from MGMT-UI-008; sveltekit-implementer owns adapter-static component work for the desktop app.
coding_standards: No relative imports — @gitbutler/ package references, Prettier: tabs, double quotes, no trailing commas, 100-col, No console.log, Components PascalCase; files kebab-case, Svelte 5 $props()/$state()/$derived() rune syntax, CT describe blocks MUST use the component name as the outermost describe string (e.g. describe('BranchGatesList', () => {...})) so `pnpm test:ct:desktop -- <ComponentName>` grep matches reliably.

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: MGMT-BE-004 (branch_gates_read / branch_gates_update SDK + gate field types — this component's data source); MGMT-UI-001 (desktop CT harness; from Sprint 06a); MGMT-UI-003 (GovernanceSettings.svelte — provides isReadOnly prop and pendingStore context; from Sprint 06a); DESIGN-MGMT-006 (empty states for Branch Gates tab); DESIGN-MGMT-007 (four-tab IA + aria contract); MGMT-UI-008 (GroupsList.svelte — the primary ExpandableSection pattern; from Sprint 06a; if not yet landed, fall back to ExpandableSection.svelte + Modal.svelte as pattern sources)
Blocks:     MGMT-UI-012 (build-gate tests assert no direct config write across all governance components including BranchGatesList)
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "MGMT-UI-009",
  "proposed_by": "sveltekit-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "seeded_gates_two_branches": {
      "description": "branch_gates_read SDK mock returns two gates: main (protected:true, min_approvals:2, require_distinct_from_author:true, require_approval_from_group:['eng','security']) and develop (protected:false, min_approvals:1, require_distinct_from_author:false, require_approval_from_group:[]). Defined groups: ['eng','security','platform'].",
      "seed_method": "ui_flow",
      "records": [
        "main: protected=true, min_approvals=2, distinct=true, groups=['eng','security']",
        "develop: protected=false, min_approvals=1, distinct=false, groups=[]",
        "defined_groups=['eng','security','platform']",
        "defined_groups populated from separate groups SDK call mock (NOT from branch_gates_read)"
      ]
    },
    "seeded_empty_gates": {
      "description": "branch_gates_read SDK mock returns an empty list. Defined groups: ['eng'].",
      "seed_method": "ui_flow",
      "records": [
        "gates=[]",
        "defined_groups=['eng']",
        "defined_groups populated from separate groups SDK call mock (NOT from branch_gates_read)"
      ]
    },
    "seeded_gates_readonly": {
      "description": "seeded_gates_two_branches fixture with isReadOnly=true passed to BranchGatesList.",
      "seed_method": "ui_flow",
      "records": [
        "same as seeded_gates_two_branches",
        "isReadOnly=true",
        "defined_groups populated from separate groups SDK call mock (NOT from branch_gates_read)"
      ]
    },
    "seeded_write_denied": {
      "description": "branch_gates_update SDK mock returns a structured denial: {code:'perm.denied', message:'Permission denied: administration:write required', remediation_hint:'Contact your repository administrator.'} for any call.",
      "seed_method": "ui_flow",
      "records": [
        "branch_gates_update returns {code:'perm.denied', message:'Permission denied: administration:write required', remediation_hint:'Contact your repository administrator.'}"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN BranchGatesList mounted with seeded_gates_two_branches WHEN user expands the 'main' row THEN four gate fields render: protected Toggle ON, min_approvals Textbox showing '2', require_distinct_from_author Toggle ON, require_approval_from_group chips showing 'eng' and 'security'",
      "verify": "pnpm test:ct:desktop -- BranchGatesListRows",
      "scenario": {
        "id": "SC-MGMT-UI-009-1",
        "primary": true,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "a static shell renders hardcoded fields not from the seeded SDK response (disconnected from SDK)",
            "the branch_gates_read SDK call is never made (stub returning empty or no-op)",
            "the ExpandableSection renders empty content on expand",
            "the min_approvals Textbox shows '0' instead of '2' (degenerate default, not from SDK response)"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_gates_two_branches",
            "action": {
              "actor": "user",
              "steps": [
                "mount BranchGatesList with seeded_gates_two_branches",
                "expand the 'main' ExpandableSection",
                "observe all four gate fields"
              ]
            },
            "end_state": {
              "must_observe": [
                "exactly `2` ExpandableSection rows with labels `'main'` and `'develop'`",
                "on 'main' expand: the protected Toggle has `aria-checked='true'`",
                "on 'main' expand: the min_approvals Textbox has `value='2'` (type=number)",
                "on 'main' expand: the require_distinct_from_author Toggle has `aria-checked='true'`",
                "on 'main' expand: exactly `2` group chips with accessible names `'eng'` and `'security'` in the required-group selector"
              ],
              "must_not_observe": [
                "a single row (develop branch absent, only `1` ExpandableSection)",
                "min_approvals showing `'0'` or `empty` instead of `'2'`",
                "an empty expanded section with `0` fields rendered",
                "undefined group names in the group chips"
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
      "description": "GIVEN BranchGatesList mounted with seeded_gates_two_branches, main row expanded WHEN user changes min_approvals Textbox from '2' to '3' THEN branch_gates_update is called with the updated field value and the 'main' row gains a pending indicator (\u25cb Badge or pending state marker)",
      "verify": "pnpm test:ct:desktop -- BranchGatesListEdit",
      "scenario": {
        "id": "SC-MGMT-UI-009-2",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "the Textbox change fires no SDK call (no-op stub; call count remains `0`)",
            "branch_gates_update is called with min_approvals still `'2'` (unchanged value, not the new `'3'`)",
            "no pending indicator appears after the write (pending state disconnected from SDK response)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_gates_two_branches",
            "action": {
              "actor": "user",
              "steps": [
                "expand 'main' row",
                "change the min_approvals Textbox value from '2' to '3'"
              ]
            },
            "end_state": {
              "must_observe": [
                "the `branch_gates_update` SDK spy called `== 1` time with `{branch: 'main', min_approvals: 3}`",
                "a pending indicator (Badge with `warning` variant or element with `aria-label` containing `'pending'`) on the `'main'` row"
              ],
              "must_not_observe": [
                "`branch_gates_update` called with `min_approvals: 2` (unchanged value)",
                "`0` SDK calls after the Textbox change (no-op stub)",
                "`0` pending indicators on the 'main' row after the write"
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
      "description": "GIVEN BranchGatesList mounted with seeded_empty_gates WHEN component renders; then user clicks '+ Add' THEN EmptyStatePlaceholder renders initially; clicking '+ Add' shows an add-gate form; confirming with pattern 'staging' calls branch_gates_update with a new gate entry for 'staging'",
      "verify": "pnpm test:ct:desktop -- BranchGatesListEmpty",
      "scenario": {
        "id": "SC-MGMT-UI-009-3",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "EmptyStatePlaceholder is absent when no gates are configured (static rows render instead of `empty` state)",
            "the add flow fires no SDK call (disconnected; call count `0`)",
            "the `'+ Add'` button does not appear in the empty state (hardcoded absent)"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_empty_gates",
            "action": {
              "actor": "user",
              "steps": [
                "mount BranchGatesList with seeded_empty_gates",
                "observe the empty state",
                "click '+ Add', enter branch pattern 'staging', confirm"
              ]
            },
            "end_state": {
              "must_observe": [
                "EmptyStatePlaceholder containing the text `'No branch gates configured.'` on initial render (before '+ Add')",
                "`branch_gates_update` SDK spy called `== 1` time with a new gate entry for pattern `'staging'`",
                "the `branch_gates_update` call payload includes at minimum `{branch: 'staging', protected: true, min_approvals: 1}` (default field values \u2014 protected:true, min_approvals:1)"
              ],
              "must_not_observe": [
                "existing gate rows when `branch_gates_read` returns `empty` (static rows rendered from `none`)",
                "`0` EmptyStatePlaceholder elements in the initial empty state",
                "`branch_gates_update` called with `min_approvals: undefined` or `min_approvals: null` (incomplete gate entry)"
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
      "description": "GIVEN BranchGatesList mounted with seeded_gates_two_branches (defined_groups=['eng','security','platform']), main row expanded WHEN user opens the required-group selector dropdown THEN options are exactly ['eng','security','platform'] \u2014 no undefined or fabricated group names",
      "verify": "pnpm test:ct:desktop -- BranchGatesListGroupSelector",
      "scenario": {
        "id": "SC-MGMT-UI-009-4",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "the selector shows a hardcoded list that does not match defined_groups (static, not from SDK response)",
            "undefined group names appear in the dropdown (no filtering applied)",
            "the selector options are `empty` even when `3` groups are defined (disconnected from groups query)"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_gates_two_branches",
            "action": {
              "actor": "user",
              "steps": [
                "expand the 'main' row",
                "open the required-group selector dropdown"
              ]
            },
            "end_state": {
              "must_observe": [
                "exactly `3` options in the dropdown with accessible names `'eng'`, `'security'`, `'platform'` (and `0` others)"
              ],
              "must_not_observe": [
                "an undefined group name (any option NOT in `['eng', 'security', 'platform']`) in the dropdown",
                "`0` options when `defined_groups` has `3` entries (empty selector)",
                "a hardcoded option count other than `3` (e.g. `4` or `1`)"
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
      "description": "GIVEN BranchGatesList mounted with seeded_gates_two_branches (main protected:true), main row expanded WHEN user toggles the protected Toggle OFF THEN a Modal confirmation dialog appears with text 'Unprotect branch main? Merges will no longer require review.' before the write is staged; confirming calls branch_gates_update with protected:false; cancelling leaves the Toggle ON and makes no SDK call",
      "verify": "pnpm test:ct:desktop -- BranchGatesListUnprotectConfirm",
      "scenario": {
        "id": "SC-MGMT-UI-009-5",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "the Toggle flips optimistically without showing the Modal (no confirmation gate \u2014 a static or no-op implementation would skip the Modal entirely)",
            "`branch_gates_update` is called with `protected: false` on Toggle click before any Modal confirmation (mock bypassed)",
            "the Modal is stubbed/absent and the write fires immediately on Toggle click",
            "cancelling still calls `branch_gates_update` (Cancel is a no-op stub)"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_gates_two_branches",
            "action": {
              "actor": "user",
              "steps": [
                "expand 'main' row",
                "click the protected Toggle to toggle OFF",
                "observe the Modal",
                "click 'Cancel'"
              ]
            },
            "end_state": {
              "must_observe": [
                "a Modal dialog with text containing `'Unprotect branch main'` and `'Merges will no longer require review'`",
                "the protected Toggle has `aria-checked='true'` (unchanged, not flipped) while the Modal is open",
                "`branch_gates_update` SDK spy call count `== 0` after Cancel"
              ],
              "must_not_observe": [
                "the Toggle with `aria-checked='false'` before the Modal appears (optimistic flip occurred)",
                "`branch_gates_update` called with `protected: false` after Cancel (`0` such calls expected)",
                "the Modal `empty`/absent after Toggle click (confirmation skipped)"
              ]
            }
          },
          {
            "start_ref": "seeded_gates_two_branches",
            "action": {
              "actor": "user",
              "steps": [
                "expand 'main' row",
                "click the protected Toggle to toggle OFF",
                "click 'Confirm' in the Modal"
              ]
            },
            "end_state": {
              "must_observe": [
                "`branch_gates_update` SDK spy called `== 1` time with `{branch: 'main', protected: false}`",
                "a pending indicator (Badge with `warning` variant or `aria-label` containing `'pending'`) on the `'main'` row after confirm"
              ],
              "must_not_observe": [
                "`0` `branch_gates_update` calls after Confirm",
                "the Modal still open after Confirm (Modal did not dismiss)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-6",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN BranchGatesList mounted with seeded_gates_readonly (isReadOnly=true) WHEN user attempts to interact with any Toggle, Textbox, Select, TagInput, or write Button THEN every Toggle has aria-disabled=true or disabled attribute; every Textbox input has disabled attribute; every TagInput/Select is readonly or disabled; every write Button has disabled attribute; 0 branch_gates_update SDK spy calls fire on any interaction",
      "verify": "pnpm test:ct:desktop -- BranchGatesListReadOnly",
      "scenario": {
        "id": "SC-MGMT-UI-009-6",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "any Toggle is interactive (aria-disabled absent, disabled absent) when isReadOnly=true (read-only state broken)",
            "`branch_gates_update` SDK spy called after interacting with a control (isReadOnly not propagated to write guard)",
            "only some controls are disabled while others remain enabled (partial read-only \u2014 gate step 3 fails)"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_gates_readonly",
            "action": {
              "actor": "user",
              "steps": [
                "mount BranchGatesList with seeded_gates_readonly (isReadOnly=true)",
                "expand the 'main' row",
                "attempt to interact with the protected Toggle, min_approvals Textbox, and group selector"
              ]
            },
            "end_state": {
              "must_observe": [
                "every Toggle element has `aria-disabled='true'` or `disabled` attribute (0 enabled Toggles in the DOM)",
                "every Textbox input has `disabled` attribute (0 enabled Textboxes)",
                "every write Button has `disabled` attribute",
                "SDK spy call count `== 0` after attempting to interact with any control"
              ],
              "must_not_observe": [
                "any Toggle without `aria-disabled` or `disabled` when isReadOnly=true (0 such enabled Toggles expected)",
                "`branch_gates_update` called (`0` such calls expected when isReadOnly=true)",
                "any Textbox without `disabled` attribute when isReadOnly=true"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-7",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN BranchGatesList mounted with seeded_gates_two_branches, main row expanded, seeded_write_denied configured WHEN user changes min_approvals Textbox from '2' to '3' (triggering branch_gates_update which returns perm.denied) THEN a danger InfoMessage appears with 'perm.denied'/'Permission denied' text; the min_approvals Textbox reverts to showing '2' (the control does NOT reflect the denied change)",
      "verify": "pnpm test:ct:desktop -- BranchGatesListWriteDenied",
      "scenario": {
        "id": "SC-MGMT-UI-009-7",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "control value remains at '3' after perm.denied (optimistic flip accepted \u2014 no denial rollback)",
            "danger InfoMessage absent after perm.denied (error swallowed \u2014 `0` InfoMessage elements with `style='danger'`)",
            "the Textbox is not reverted to '2' (stale UI state persists after denial)"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_gates_two_branches",
            "action": {
              "actor": "user",
              "steps": [
                "configure seeded_write_denied (branch_gates_update returns perm.denied)",
                "expand 'main' row",
                "change the min_approvals Textbox from '2' to '3'",
                "observe the InfoMessage and Textbox value"
              ]
            },
            "end_state": {
              "must_observe": [
                "an `InfoMessage` with `style='danger'` containing `'perm.denied'` or `'Permission denied'`",
                "the min_approvals Textbox value is `'2'` (unchanged \u2014 denial rolled back the control)"
              ],
              "must_not_observe": [
                "the Textbox showing `'3'` after perm.denied (optimistic flip accepted)",
                "`0` InfoMessage elements with `style='danger'` after the denied write"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-8",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN BranchGatesList mounted with seeded_gates_two_branches, a branch currently unprotected (protected:false) WHEN user toggles the protected Toggle ON THEN no Modal confirmation appears (re-protect is non-destructive); branch_gates_update is called once with protected:true; the Toggle ends at aria-checked='true'; no danger InfoMessage appears",
      "verify": "pnpm test:ct:desktop -- BranchGatesListReprotect",
      "scenario": {
        "id": "SC-MGMT-UI-009-8",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "a Modal confirmation appears during re-protect (re-protect is non-destructive and must not show a Modal)",
            "the protected Toggle stays aria-checked='false' after the click (write did not go through)",
            "the component stubs the click handler and never calls branch_gates_update",
            "a danger InfoMessage appears after the re-protect write (write failed silently)"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_gates_two_branches",
            "action": {
              "actor": "user",
              "steps": [
                "expand 'main' row",
                "click the protected Toggle to toggle OFF (unprotect) and confirm the Modal \u2014 seeds main with protected:false",
                "click the protected Toggle to toggle ON (re-protect)"
              ]
            },
            "end_state": {
              "must_observe": [
                "the 'main' protected Toggle has `aria-checked='true'`",
                "branch_gates_update SDK spy called with `{branch: 'main', protected: true}`",
                "document query for the unprotect modal returns hidden (not visible)",
                "no InfoMessage with testId `'branch-gates-list-write-error'` in the DOM"
              ],
              "must_not_observe": [
                "a Modal dialog in the DOM during the re-protect flow",
                "Toggle `aria-checked='false'` after the re-protect click",
                "branch_gates_update called with `protected: false` on the re-protect action",
                "a danger InfoMessage after the re-protect write"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "Mounting with seeded_gates_two_branches and expanding 'main' renders all four gate field controls with seeded values (protected ON, min_approvals '2', distinct ON, groups 'eng'+'security')",
      "verify": "pnpm test:ct:desktop -- BranchGatesListRows",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "Changing min_approvals from '2' to '3' calls branch_gates_update with the updated value and the row gains a pending indicator",
      "verify": "pnpm test:ct:desktop -- BranchGatesListEdit",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "With no gates configured, EmptyStatePlaceholder renders; '+ Add' flow calls branch_gates_update with the new pattern",
      "verify": "pnpm test:ct:desktop -- BranchGatesListEmpty",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "Required-group selector dropdown contains exactly the 3 defined groups and no undefined names",
      "verify": "pnpm test:ct:desktop -- BranchGatesListGroupSelector",
      "maps_to_ac": "AC-4"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "Toggling protected OFF shows Modal with exact text before any write; Cancel fires 0 SDK calls and leaves Toggle ON; Confirm calls branch_gates_update with protected:false",
      "verify": "pnpm test:ct:desktop -- BranchGatesListUnprotectConfirm",
      "maps_to_ac": "AC-5"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "description": "isReadOnly=true: every Toggle/Textbox/Button disabled; 0 branch_gates_update SDK calls fire on interaction",
      "verify": "pnpm test:ct:desktop -- BranchGatesListReadOnly",
      "maps_to_ac": "AC-6"
    },
    {
      "id": "TC-7",
      "type": "test_criterion",
      "description": "branch_gates_update returning perm.denied: danger InfoMessage appears; control reverts to original value (no flip)",
      "verify": "pnpm test:ct:desktop -- BranchGatesListWriteDenied",
      "maps_to_ac": "AC-7"
    },
    {
      "id": "TC-8",
      "type": "test_criterion",
      "description": "Toggling protected ON from an unprotected state fires branch_gates_update with protected:true immediately (no Modal), Toggle ends at aria-checked='true', and no danger InfoMessage appears",
      "verify": "pnpm test:ct:desktop -- BranchGatesListReprotect",
      "maps_to_ac": "AC-8"
    }
  ]
}
-->
