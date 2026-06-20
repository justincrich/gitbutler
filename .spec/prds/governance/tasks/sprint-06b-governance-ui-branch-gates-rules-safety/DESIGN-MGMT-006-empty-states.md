# DESIGN-MGMT-006: Empty states for all four tabs

## What this does

Specify the exact EmptyStatePlaceholder content (title, caption, primary action) for all four governance tabs, completing the empty-state layer begun in DESIGN-MGMT-001. Sprint 06b scope is Branch Gates + Rules; Principals + Groups are referenced from DESIGN-MGMT-001. Each empty state must include the primary action and its disabled treatment in read-only mode.

## Why

Sprint 06b · PRD UC-MGMT-03, UC-MGMT-04, UC-MGMT-05 · capability —. A sveltekit-implementer reading this contract knows the exact EmptyStatePlaceholder props to pass for the Branch Gates and Rules empty states, the primary action Button label and target, and how these states interact with isReadOnly — witho

## How to verify

PRIMARY **AC-1** — `design review — reviewer confirms EmptyStatePlaceholder path + title text + caption text + Button primary action label + read-only disabled treatment`: Branch Gates empty state [PRIMARY]. Full gate set in the spec below.

## Scope

  - apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md (MODIFY — extend with Sprint 06b empty-state section for Branch Gates + Rules)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: DESIGN-MGMT-006 — Empty states for all four tabs
================================================================================

TASK_TYPE:   DESIGN
STATUS:      Backlog
PRIORITY:    P1
EFFORT:      XS  (25 min)
AGENT:       frontend-designer
PROPOSED-BY: frontend-designer
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-MGMT-03, UC-MGMT-04, UC-MGMT-05
CAPABILITIES:—

RUNTIME_COMMANDS:
  test:  pnpm test:ct:desktop -- GovernanceEmptyStates (exercised by MGMT-UI-009 and MGMT-UI-010 implementations)

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
A sveltekit-implementer reading this contract knows the exact EmptyStatePlaceholder props to pass for the Branch Gates and Rules empty states, the primary action Button label and target, and how these states interact with isReadOnly — without any design judgment at implementation time.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST specify EmptyStatePlaceholder (packages/ui/src/lib/components/EmptyStatePlaceholder.svelte) for all four tabs: Principals, Groups, Branch Gates, Rules — with exact title, caption, and primary action content per tab.
- [MUST] MUST cover ONLY the Branch Gates and Rules empty states that are Sprint 06b net-new scope; the Principals and Groups empty states were annotated in DESIGN-MGMT-001 (Sprint 06a) — extend DESIGN-ANNOTATIONS.md with the 06b empty-state section, citing the 06a entries for Principals/Groups by reference rather than duplicating them.
- [MUST] MUST specify the Branch Gates empty state verbatim: title 'No branch gates configured.' with a primary action [+ Add] Button.
- [MUST] MUST specify the Rules tab empty state for two sub-cases: (a) no principal selected — placeholder 'Select a principal to view their rules'; (b) principal selected but no rules — reuses existing RulesList empty-state behavior (the existing component renders its own placeholder; no new content to specify beyond confirming the component handles it).
- [MUST] MUST confirm each empty-state action button is disabled when isReadOnly=true (per DESIGN-MGMT-003).
- [NEVER] NEVER introduce a new design-system token, CSS variable, or hex color not already in packages/ui.
- [NEVER] NEVER re-specify the Principals or Groups empty states already defined in DESIGN-MGMT-001 — reference those entries; do not duplicate.
- [NEVER] NEVER invent a new empty-state component — EmptyStatePlaceholder is the only one used across all four tabs.
- [NEVER] NEVER specify empty-state content for a sub-tab or view that is not one of the four named tabs (Principals, Groups, Branch Gates, Rules).
- [STRICTLY] STRICTLY reuse packages/ui/src/lib/components/EmptyStatePlaceholder.svelte — no new components.
- [STRICTLY] STRICTLY the Branch Gates empty-state title is 'No branch gates configured.' (verbatim from the 10-ui-infrastructure.md Branch Gates wireframe).
- [STRICTLY] STRICTLY no new design-system work — EmptyStatePlaceholder, Button already exist.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: Branch Gates empty state [PRIMARY]
- [ ] AC-2: Rules tab empty states (no principal / no rules)
- [ ] AC-3: Read-only disabled treatment for empty-state actions
- [ ] AC-4: No new design-system tokens
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: Branch Gates empty state [PRIMARY]
  GIVEN: the 06b empty-state section of DESIGN-ANNOTATIONS.md covers the Branch Gates tab
  WHEN:  a reviewer inspects the Branch Gates empty-state specification
  THEN:  it specifies EmptyStatePlaceholder (packages/ui/src/lib/components/EmptyStatePlaceholder.svelte) with title='No branch gates configured.', a caption describing that gates control merge requirements, and a primary action Button (packages/ui/src/lib/components/Button.svelte) labeled '+ Add' that opens the add-gate flow; in read-only mode (isReadOnly=true) the Button receives disabled=true
  TEST_TIER: component   VERIFICATION_SERVICE: apps/desktop CT harness (pnpm test:ct:desktop) — downstream T-MGMT-019 component test asserts BranchGatesList renders EmptyStatePlaceholder when no gates are configured
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test (pnpm test:ct:desktop) exercising T-MGMT-019
  VERIFY: design review — reviewer confirms EmptyStatePlaceholder path + title text + caption text + Button primary action label + read-only disabled treatment

AC-2: Rules tab empty states (no principal / no rules)
  GIVEN: the 06b empty-state section covers the Rules tab
  WHEN:  a reviewer inspects the Rules tab empty-state specification
  THEN:  it specifies two sub-cases: (a) no principal selected — EmptyStatePlaceholder title='Select a principal to view their rules', no primary action button; (b) principal selected but RulesList has no rules — the existing RulesList component renders its own built-in empty state (no new content override required; the spec notes that the RulesList empty-state is owned by the existing component and must not be overridden)
  TEST_TIER: component   VERIFICATION_SERVICE: apps/desktop CT harness (pnpm test:ct:desktop) — downstream T-MGMT-025 component test asserts the Rules tab renders a placeholder when no principal is selected or no rules exist
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test (pnpm test:ct:desktop) exercising T-MGMT-025
  VERIFY: design review — reviewer confirms two sub-case entries: the no-principal-selected EmptyStatePlaceholder with exact title, and the existing-RulesList-empty-state deferral with the explanation that RulesList owns its own empty state

AC-3: Read-only disabled treatment for empty-state actions
  GIVEN: the contract specifies read-only treatment for empty-state actions
  WHEN:  a reviewer checks the disabled-in-read-only rule for each empty-state action
  THEN:  it states that every primary action Button in an empty state receives disabled=true when isReadOnly=true (derived at GovernanceSettings.svelte per DESIGN-MGMT-003); the EmptyStatePlaceholder itself remains visible (not hidden) in read-only — only the action is disabled
  TEST_TIER: component   VERIFICATION_SERVICE: apps/desktop CT harness (pnpm test:ct:desktop) — downstream T-MGMT-029 component test asserts controls disabled in read-only state
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test (pnpm test:ct:desktop)
  VERIFY: design review — reviewer confirms the disabled-in-read-only rule is stated for Branch Gates empty state, and that the EmptyStatePlaceholder stays visible in read-only

AC-4: No new design-system tokens
  GIVEN: the 06b empty-state section extends DESIGN-ANNOTATIONS.md
  WHEN:  a reviewer audits every color/spacing/typography reference in the 06b empty-state section
  THEN:  every visual attribute uses an existing CSS variable or defers to the component's own stylesheet — no hex literals, no new var(--empty-*)/var(--mgmt-*) tokens
  TEST_TIER: component   VERIFICATION_SERVICE: grep audit on the annotation file
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by static grep on the annotation file
  VERIFY: grep the 06b empty-state section of DESIGN-ANNOTATIONS.md for '#' color literals and var(--empty-)/var(--mgmt-) -> both return zero matches

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): contract names EmptyStatePlaceholder path + title='No branch gates configured.' + caption + Button '+ Add' + disabled=true in read-only for Branch Gates empty state
    VERIFY: design review of the Branch Gates empty-state entry in DESIGN-ANNOTATIONS.md
- TC-2 (-> AC-2): contract specifies two Rules tab sub-cases: no-principal-selected EmptyStatePlaceholder ('Select a principal to view their rules', no action) and principal-selected-no-rules deferred to RulesList built-in empty state
    VERIFY: design review of the Rules tab empty-state entry in DESIGN-ANNOTATIONS.md
- TC-3 (-> AC-3): contract states every empty-state primary action Button receives disabled=true when isReadOnly=true; EmptyStatePlaceholder remains visible in read-only
    VERIFY: design review of the read-only treatment rule in the empty-state section
- TC-4 (-> AC-4): zero hex color literals and zero new var(--empty-*)/var(--mgmt-*) tokens in the 06b empty-state section
    VERIFY: grep -nE '#[0-9a-fA-F]{3,6}|var\(--(empty|mgmt)-' apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md -> zero matches

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md (MODIFY — extend with Sprint 06b empty-state section for Branch Gates + Rules)
writeProhibited:
  - packages/ui/src/lib/components/** — no design-system changes
  - apps/desktop/src/components/shared/** — no shared component changes
  - any .svelte or .ts implementation file — design spec artifact only

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. .spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md [83-110] — Branch Gates tab wireframe: '── empty: EmptyStatePlaceholder "No branch gates configured."' and Rules tab wireframe — the primary source of truth for empty-state content [PRIMARY PATTERN]
2. packages/ui/src/lib/components/EmptyStatePlaceholder.svelte [1-50] — EmptyStatePlaceholder props: title, caption, and action slot — confirm the component API before specifying slot content
3. .spec/prds/governance/tasks/sprint-06a-governance-ui-scaffold-principals-groups/DESIGN-MGMT-001-four-tab-annotations.md [95-130] — AC-3 and TC-3 in DESIGN-MGMT-001 specify Principals and Groups empty states — reference these, do not duplicate
4. .spec/prds/governance/08-uc-mgmt.md [93-130] — UC-MGMT-03 (Groups empty state) + UC-MGMT-04 (Branch Gates empty state) + UC-MGMT-05 (Rules empty/placeholder state)
5. packages/ui/src/lib/components/Button.svelte [1-30] — Button disabled prop — confirm disabled=true treatment for read-only empty-state actions

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- design review of apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md 06b empty-state section   -> EmptyStatePlaceholder path + title='No branch gates configured.' + caption + Button '+ Add' + disabled-in-read-only rule — all present
- design review of the Rules tab empty-state entry   -> no-principal-selected entry with EmptyStatePlaceholder title; principal-selected-no-rules entry deferring to RulesList built-in
- design review of the read-only treatment rule in the empty-state section   -> every primary action Button disabled=true in read-only; EmptyStatePlaceholder remains visible
- grep -nE '#[0-9a-fA-F]{3,6}|var\(--(empty|mgmt)-' apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md   -> zero matches
- pnpm test:ct:desktop -- GovernanceEmptyStates (exercised by MGMT-UI-009 and MGMT-UI-010 implementations)   -> T-MGMT-014 (Groups empty state), T-MGMT-019 (Branch Gates add + empty), T-MGMT-025 (Rules placeholder) all pass

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references:
  - .spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md — Branch Gates wireframe (lines 83-97): '── empty: EmptyStatePlaceholder "No branch gates configured."'; Rules tab wireframe (lines 99-110): principal-picker + RulesList panel
  - .spec/prds/governance/tasks/sprint-06a-governance-ui-scaffold-principals-groups/DESIGN-MGMT-001-four-tab-annotations.md — AC-3/TC-3 specifying Principals and Groups empty states (do not duplicate, reference by ID)
notes:
  - Branch Gates empty state: renders when the SDK returns an empty gates array; the [+ Add] Button opens the add-gate flow (MGMT-UI-009 implementation detail — design contract specifies label and disabled=true in read-only only)
  - Rules tab — no principal selected: renders before the user selects a principal from the left-column principal picker; no action button (user must select a principal to proceed; the picker itself is the action path)
  - Rules tab — principal selected, no rules: the existing RulesList component handles this with its own built-in empty state (NewRuleMenu affordance); the design contract defers to the component and must not override it with a new EmptyStatePlaceholder
  - read-only interaction: all empty-state action Buttons disabled=true when isReadOnly=true (from GovernanceSettings.svelte isReadOnly prop per DESIGN-MGMT-003); EmptyStatePlaceholder body text and icon remain visible — only the primary action is suppressed
pattern: EmptyStatePlaceholder as the single empty-state component across all four tabs; populated via title/caption/action slot content; primary action is a Button (disabled in read-only). The Rules tab has two distinct sub-cases driven by whether principalId is set.
pattern_source: .spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md Branch Gates and Rules wireframes; DESIGN-MGMT-001 AC-3 for the established EmptyStatePlaceholder pattern in Principals and Groups tabs.
anti_pattern: a custom empty-state layout or SVG illustration outside EmptyStatePlaceholder; hiding the EmptyStatePlaceholder in read-only instead of just disabling the action; duplicating the Principals/Groups empty-state annotations already in DESIGN-MGMT-001

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agent: frontend-designer
rationale: frontend-designer owns wireframe-fidelity annotation for all tab states including empty states; no Rust/Tauri or SDK knowledge required — purely a visual-content contract consumed by sveltekit-implementer (MGMT-UI-009, MGMT-UI-010).
coding_standards: All component citations must use exact source paths from the packages/ui Component-reuse table in 10-ui-infrastructure.md, Branch Gates title must be reproduced verbatim: 'No branch gates configured.', Principals and Groups empty states must be referenced by DESIGN-MGMT-001 AC-3, not re-specified

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: DESIGN-MGMT-001 (four-tab annotations Sprint 06a — AC-3 specifies Principals/Groups empty states; this task extends with Branch Gates/Rules); DESIGN-MGMT-003 (read-only state contract — empty-state action Button disabled treatment comes from the isReadOnly prop defined there)
Blocks:     MGMT-UI-009 (BranchGatesList — consumes the Branch Gates empty-state design contract); MGMT-UI-010 (RulesList principalId prop — consumes the Rules tab no-principal empty-state design contract)
```
</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "DESIGN-MGMT-006",
  "proposed_by": "frontend-designer",
  "verification_policy": {
    "requires_tests": false,
    "requires_red_evidence": false,
    "requires_seeded_evidence": false
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN the 06b empty-state section of DESIGN-ANNOTATIONS.md covers the Branch Gates tab WHEN a reviewer inspects the Branch Gates empty-state specification THEN it specifies EmptyStatePlaceholder (packages/ui/src/lib/components/EmptyStatePlaceholder.svelte) with title='No branch gates configured.', a caption describing that gates control merge requirements, and a primary action Button (packages/ui/src/lib/components/Button.svelte) labeled '+ Add' that opens the add-gate flow; in read-only mode (isReadOnly=true) the Button receives disabled=true",
      "verify": "design review \u2014 reviewer confirms EmptyStatePlaceholder path + title text + caption text + Button primary action label + read-only disabled treatment"
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the 06b empty-state section covers the Rules tab WHEN a reviewer inspects the Rules tab empty-state specification THEN it specifies two sub-cases: (a) no principal selected \u2014 EmptyStatePlaceholder title='Select a principal to view their rules', no primary action button; (b) principal selected but RulesList has no rules \u2014 the existing RulesList component renders its own built-in empty state (no new content override required; the spec notes that the RulesList empty-state is owned by the existing component and must not be overridden)",
      "verify": "design review \u2014 reviewer confirms two sub-case entries: the no-principal-selected EmptyStatePlaceholder with exact title, and the existing-RulesList-empty-state deferral with the explanation that RulesList owns its own empty state"
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the contract specifies read-only treatment for empty-state actions WHEN a reviewer checks the disabled-in-read-only rule for each empty-state action THEN it states that every primary action Button in an empty state receives disabled=true when isReadOnly=true (derived at GovernanceSettings.svelte per DESIGN-MGMT-003); the EmptyStatePlaceholder itself remains visible (not hidden) in read-only \u2014 only the action is disabled",
      "verify": "design review \u2014 reviewer confirms the disabled-in-read-only rule is stated for Branch Gates empty state, and that the EmptyStatePlaceholder stays visible in read-only"
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the 06b empty-state section extends DESIGN-ANNOTATIONS.md WHEN a reviewer audits every color/spacing/typography reference in the 06b empty-state section THEN every visual attribute uses an existing CSS variable or defers to the component's own stylesheet \u2014 no hex literals, no new var(--empty-*)/var(--mgmt-*) tokens",
      "verify": "grep the 06b empty-state section of DESIGN-ANNOTATIONS.md for '#' color literals and var(--empty-)/var(--mgmt-) -> both return zero matches"
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "contract names EmptyStatePlaceholder path + title='No branch gates configured.' + caption + Button '+ Add' + disabled=true in read-only for Branch Gates empty state",
      "verify": "design review of the Branch Gates empty-state entry in DESIGN-ANNOTATIONS.md",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "contract specifies two Rules tab sub-cases: no-principal-selected EmptyStatePlaceholder ('Select a principal to view their rules', no action) and principal-selected-no-rules deferred to RulesList built-in empty state",
      "verify": "design review of the Rules tab empty-state entry in DESIGN-ANNOTATIONS.md",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "contract states every empty-state primary action Button receives disabled=true when isReadOnly=true; EmptyStatePlaceholder remains visible in read-only",
      "verify": "design review of the read-only treatment rule in the empty-state section",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "zero hex color literals and zero new var(--empty-*)/var(--mgmt-*) tokens in the 06b empty-state section",
      "verify": "grep -nE '#[0-9a-fA-F]{3,6}|var\\(--(empty|mgmt)-' apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md -> zero matches",
      "maps_to_ac": "AC-4"
    }
  ]
}
-->
