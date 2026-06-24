# DESIGN-MGMT-007: Four-tab IA + aria + keyboard-nav contract

## What this does

Extend the four-tab IA annotation from DESIGN-MGMT-001 with the accessibility contract: exact aria attribute names and values for the tablist/tab/tabpanel roles, the keyboard navigation model (Tab-into/Arrow-within/Enter-or-Space-activate), and the focus-visible treatment — using the existing shared/Tabs component. This contract is the direct input for MGMT-UI-011 (aria + keyboard nav).

## Why

Sprint 06b · PRD UC-MGMT-01, UC-MGMT-07 · capability —. A sveltekit-implementer reading this contract knows exactly which aria attributes to add to which elements in the Tabs composition, what keyboard events to handle and how, and what focus-visible treatment to use — without making accessibili

## How to verify

PRIMARY **AC-1** — `design review — reviewer confirms all five aria attribute entries (tablist label, tab role, tab aria-selected, tab aria-controls, tabpanel aria-labelledby) are present with the exact attribute names and value patterns`: Aria attribute specification for tab roles [PRIMARY]. Full gate set in the spec below.

## Scope

- apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md (MODIFY — extend with Sprint 06b a11y/keyboard-nav section)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: DESIGN-MGMT-007 — Four-tab IA + aria + keyboard-nav contract
================================================================================

TASK_TYPE:   DESIGN
STATUS:      Backlog
PRIORITY:    P0
EFFORT:      S  (35 min)
AGENT:       frontend-designer
PROPOSED-BY: frontend-designer
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-MGMT-01, UC-MGMT-07
CAPABILITIES:—

RUNTIME_COMMANDS:
  test:  pnpm test:ct:desktop -- GovernanceTabs (exercised by MGMT-UI-011 implementation)

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
A sveltekit-implementer reading this contract knows exactly which aria attributes to add to which elements in the Tabs composition, what keyboard events to handle and how, and what focus-visible treatment to use — without making accessibility design decisions independently.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST specify the exact aria attributes for the tab navigation: aria-label='Governance configuration tabs' on the TabList wrapper, aria-labelledby linking each TabContent to its TabTrigger, role='tablist'/'tab'/'tabpanel' per the WAI-ARIA Tabs pattern — citing apps/desktop/src/components/shared/Tabs.svelte.
- [MUST] MUST specify the keyboard navigation contract exactly: Tab key moves focus INTO the tablist (from outside); Arrow Left/Right moves between tabs within the tablist; Enter/Space activates the focused tab; Tab from within a tab panel moves focus to the next focusable element in the panel (not back to the tablist).
- [MUST] MUST specify the focus-visible treatment: the focused TabTrigger must show a visible focus indicator using var(--focus-outline) or the browser's default :focus-visible outline — no custom outline that bypasses accessibility.
- [MUST] MUST specify the four tab IDs in canonical order: principals, groups, branch-gates, rules — confirming these match DESIGN-MGMT-001 AC-2.
- [MUST] MUST extend DESIGN-MGMT-001 (four-tab annotations) with the a11y layer — this task adds the aria + keyboard-nav contract on top of the existing tab IA annotation; it does not replace or re-specify the tab component choice.
- [NEVER] NEVER introduce a new design-system token, CSS variable, or focus-ring style not already in packages/ui.
- [NEVER] NEVER re-specify the tab component choice (apps/desktop/src/components/shared/Tabs.svelte) — that is DESIGN-MGMT-001 AC-2; only add the aria + keyboard-nav layer.
- [NEVER] NEVER annotate implementation logic (Svelte event handlers, store reads) — annotate the aria attribute names, values, and keyboard behavior only.
- [NEVER] NEVER use role='menu' or role='navigation' for the tab strip — it is strictly role='tablist' per WAI-ARIA.
- [STRICTLY] STRICTLY follow WAI-ARIA Tabs pattern (https://www.w3.org/WAI/ARIA/apg/patterns/tabs/) for aria roles and keyboard interaction — do not invent a custom pattern.
- [STRICTLY] STRICTLY the tab order of keyboard focus within the tablist is Arrow-key driven (roving tabindex), not Tab-driven — Tab moves into and out of the tablist, Arrow keys move within it.
- [STRICTLY] STRICTLY cite apps/desktop/src/components/shared/Tabs.svelte as the implementation vehicle — confirm whether the existing component already supports these aria attributes or whether the implementer needs to add them.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [x] AC-1 [PRIMARY]: Aria attribute specification for tab roles [PRIMARY]
- [ ] AC-2: Keyboard navigation model
- [ ] AC-3: Focus-visible treatment
- [ ] AC-4: Tabs component audit note
- [ ] AC-5: No new design-system tokens
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: Aria attribute specification for tab roles [PRIMARY]
  GIVEN: the a11y contract section of DESIGN-ANNOTATIONS.md covers the four-tab navigation
  WHEN:  a reviewer inspects the aria attribute specification
  THEN:  it specifies: TabList element has role='tablist' and aria-label='Governance configuration tabs'; each TabTrigger element has role='tab', aria-selected='true|false', aria-controls='{panel-id}', and id='{tab-id}'; each TabContent element has role='tabpanel', aria-labelledby='{tab-id}', and id='{panel-id}'; all four IDs are listed in canonical order (principals, groups, branch-gates, rules) with the exact id/panel-id pairs
  TEST_TIER: component   VERIFICATION_SERVICE: apps/desktop CT harness (pnpm test:ct:desktop) — downstream T-MGMT-038 component test asserts aria-label and tab aria attributes are present on the rendered governance tabs
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test (pnpm test:ct:desktop) exercising T-MGMT-038
  VERIFY: design review — reviewer confirms all five aria attribute entries (tablist label, tab role, tab aria-selected, tab aria-controls, tabpanel aria-labelledby) are present with the exact attribute names and value patterns

AC-2: Keyboard navigation model
  GIVEN: the contract specifies the keyboard navigation model
  WHEN:  a reviewer reads the keyboard-nav contract
  THEN:  it states the exact key-to-action mapping: (1) Tab from outside the tablist: focus moves to the currently active TabTrigger (not to the first tab); (2) Arrow Left: focus moves to the previous TabTrigger in the list (wraps from principals to rules); (3) Arrow Right: focus moves to the next TabTrigger (wraps from rules to principals); (4) Enter or Space: activates the focused tab (sets aria-selected='true', renders the panel); (5) Tab from within the tab panel: focus leaves the tabpanel into the next focusable element (does NOT return to the tablist); this is the WAI-ARIA automatic-activation model
  TEST_TIER: component   VERIFICATION_SERVICE: apps/desktop CT harness (pnpm test:ct:desktop) — downstream T-MGMT-038 component test asserts keyboard Tab/Arrow navigation works on the governance tabs
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test (pnpm test:ct:desktop) exercising T-MGMT-038
  VERIFY: design review — reviewer confirms all five key-to-action entries are present and the automatic-activation model is named

AC-3: Focus-visible treatment
  GIVEN: the contract specifies the focus-visible treatment
  WHEN:  a reviewer inspects the focus-visible section
  THEN:  it states that the active-focus TabTrigger shows a visible outline using the existing var(--focus-outline) CSS variable (or the browser's native :focus-visible outline if the Tabs component delegates it); no custom focus ring is introduced; the focus-visible style must not be suppressed with outline:none without a replacement
  TEST_TIER: component   VERIFICATION_SERVICE: apps/desktop CT harness (pnpm test:ct:desktop) — T-MGMT-038 component test with keyboard navigation confirms focus-visible indicator is visible
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test (pnpm test:ct:desktop)
  VERIFY: design review — reviewer confirms var(--focus-outline) or :focus-visible delegation is cited, and the no-suppress-without-replacement rule is stated

AC-4: Tabs component audit note
  GIVEN: the contract cites the existing shared/Tabs implementation
  WHEN:  a reviewer checks the component audit note
  THEN:  it explicitly states whether apps/desktop/src/components/shared/Tabs.svelte already implements the WAI-ARIA tab role attributes and roving-tabindex keyboard model, or whether the sveltekit-implementer must augment it — providing a concrete action item ('Tabs.svelte already has role=tablist: verify aria-label is settable via a prop' OR 'Tabs.svelte does not set role=tab on TabTriggers: implementer must add the aria attributes')
  TEST_TIER: component   VERIFICATION_SERVICE: grep apps/desktop/src/components/shared/Tabs.svelte for role=tablist|aria- attributes to confirm or refute
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review and grep audit of the existing Tabs component
  VERIFY: design review — reviewer confirms the component audit note is present with a concrete action item for the implementer, not a hedge

AC-5: No new design-system tokens
  GIVEN: the a11y contract section extends DESIGN-ANNOTATIONS.md
  WHEN:  a reviewer audits every visual reference in the a11y section
  THEN:  every visual attribute uses an existing CSS variable — no hex literals, no new var(--a11y-*)/var(--mgmt-*) tokens; focus treatment uses var(--focus-outline) or defers to :focus-visible — not a new token
  TEST_TIER: component   VERIFICATION_SERVICE: grep audit on the annotation file
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by static grep on the annotation file
  VERIFY: grep the a11y section of DESIGN-ANNOTATIONS.md for '#' color literals and var(--a11y-)/var(--mgmt-) -> both return zero matches

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): contract names role='tablist' + aria-label='Governance configuration tabs' on TabList; role='tab' + aria-selected + aria-controls + id on each TabTrigger; role='tabpanel' + aria-labelledby + id on each TabContent; all four id pairs listed in canonical order
    VERIFY: design review of the aria attribute specification in DESIGN-ANNOTATIONS.md
- TC-2 (-> AC-2): contract names all five key-to-action mappings (Tab-in, Arrow Left, Arrow Right, Enter/Space, Tab-out) and identifies the automatic-activation WAI-ARIA model
    VERIFY: design review of the keyboard-nav contract in DESIGN-ANNOTATIONS.md
- TC-3 (-> AC-3): contract names var(--focus-outline) or :focus-visible delegation for the focused TabTrigger; states the no-suppress-without-replacement rule
    VERIFY: design review of the focus-visible section in DESIGN-ANNOTATIONS.md
- TC-4 (-> AC-4): contract includes a concrete Tabs.svelte component audit note with an actionable finding (aria already present vs must add) for the implementer
    VERIFY: design review of the component audit note section; grep apps/desktop/src/components/shared/Tabs.svelte for role= and aria- attributes
- TC-5 (-> AC-5): zero hex color literals and zero new var(--a11y-*)/var(--mgmt-*) tokens in the a11y section
    VERIFY: grep -nE '#[0-9a-fA-F]{3,6}|var\(--(a11y|mgmt)-' apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md -> zero matches

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md (MODIFY — extend with Sprint 06b a11y/keyboard-nav section)
writeProhibited:
  - apps/desktop/src/components/shared/Tabs.svelte — read only for audit; do not modify
  - packages/ui/src/lib/components/** — no design-system changes
  - any .svelte or .ts implementation file — design spec artifact only

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. apps/desktop/src/components/shared/Tabs.svelte [1-60] — Existing Tabs/TabList/TabTrigger/TabContent implementation — audit for existing role= and aria- attributes before specifying what needs to be added [PRIMARY PATTERN]
2. .spec/prds/governance/08-uc-mgmt.md [155-164] — UC-MGMT-07: 'aria-label/aria-labelledby and full keyboard nav (Tab to focus, Enter/Space to activate, Arrow keys to move between tabs)' — the authoritative functional requirement
3. .spec/prds/governance/tasks/sprint-06a-governance-ui-scaffold-principals-groups/DESIGN-MGMT-001-four-tab-annotations.md [88-130] — AC-2/TC-2 from DESIGN-MGMT-001 specifying the tab IA (four tab IDs, Tabs.svelte component choice) — this task extends, not replaces, that annotation
4. .spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md [30-50] — Principals list wireframe showing the four-tab strip layout and the settings modal context
5. .spec/prds/governance/11-e2e-testing-criteria.md [234-238] — T-MGMT-038: 'tab navigation has aria-labels + keyboard nav — Tab/Enter/Arrow work; aria present' — the exact criteria this contract must satisfy

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- design review of apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md a11y section   -> five aria attribute entries (tablist label, tab role, tab aria-selected, tab aria-controls, tabpanel aria-labelledby) all present with exact attribute names and value patterns; four id pairs listed in canonical order
- design review of the keyboard-nav section   -> five key-to-action entries (Tab-in, Arrow Left, Arrow Right, Enter/Space, Tab-out) present; automatic-activation model named
- design review of the focus-visible section   -> var(--focus-outline) or :focus-visible delegation cited; no-suppress rule stated
- grep apps/desktop/src/components/shared/Tabs.svelte for 'role=' and 'aria-' then cross-reference the annotation's audit finding   -> a concrete actionable finding (aria already present vs must add) matches the grep result
- grep -nE '#[0-9a-fA-F]{3,6}|var\(--(a11y|mgmt)-' apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md   -> zero matches
- pnpm test:ct:desktop -- GovernanceTabs (exercised by MGMT-UI-011 implementation)   -> T-MGMT-004 (four tabs via shared/Tabs) and T-MGMT-038 (aria-labels + keyboard nav) both pass

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references:
  - .spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md — Principals list wireframe (lines 30-50): four-tab strip '[Principals] [Groups] [Branch Gates] [Rules]' in the settings modal
  - .spec/prds/governance/tasks/sprint-06a-governance-ui-scaffold-principals-groups/DESIGN-MGMT-001-four-tab-annotations.md — AC-2/TC-2: tab IA with Tabs.svelte + four tab IDs (principals, groups, branch-gates, rules) — the base this task extends with the a11y layer
notes:
  - WAI-ARIA automatic-activation model: when a tab receives focus via Arrow key, it is immediately activated (panel switches); this matches the existing GitButler Tabs component behavior and avoids requiring Enter/Space to activate after Arrow focus
  - roving tabindex: only the currently active TabTrigger has tabindex=0; all others have tabindex=-1; Tab moves focus into the active tab, Arrow keys move the active tab identity
  - Tab-out of tabpanel: pressing Tab from within the panel moves focus to the next focusable element AFTER the tabs component in document order — not back to the tablist; this is the standard ARIA tabpanel Tab behavior
  - aria-label on TabList: 'Governance configuration tabs' — this is the screen-reader announced region name; it must be set once on the outer TabList wrapper, not per tab
  - the four TabTrigger id/panel-id pairs must be stable across renders (not generated dynamically per render) so aria-controls references are always valid
pattern: WAI-ARIA Tabs pattern (https://www.w3.org/WAI/ARIA/apg/patterns/tabs/) with automatic activation. TabList aria-label='Governance configuration tabs'. TabTrigger role='tab' + aria-selected + aria-controls. TabContent role='tabpanel' + aria-labelledby. Roving tabindex within the tablist. Keyboard: Tab-in/Arrow Left/Right/Enter-Space/Tab-out.
pattern_source: apps/desktop/src/components/shared/Tabs.svelte (existing tab component to extend); WAI-ARIA Authoring Practices Guide Tabs pattern; DESIGN-MGMT-001 AC-2 (four-tab IA base).
anti_pattern: role='navigation' or role='menu' for the tab strip; Tab-driven (not Arrow-driven) tab switching within the tablist; suppressing :focus-visible without a replacement; duplicating the tab component choice from DESIGN-MGMT-001

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agent: frontend-designer
rationale: frontend-designer owns the information-architecture and accessibility annotation layer; no Rust/Tauri knowledge required — this contract is the direct input for the sveltekit-implementer a11y work in MGMT-UI-011.
coding_standards: All aria attribute names must use the exact lowercase-hyphenated form from the WAI-ARIA specification, Tab IDs must match the canonical order from DESIGN-MGMT-001 AC-2: principals, groups, branch-gates, rules, The Tabs.svelte component audit note must contain a concrete finding derived from reading the actual component file — not a hedge

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: DESIGN-MGMT-001 (four-tab IA annotation Sprint 06a — AC-2 specifies the Tabs.svelte component and four tab IDs; this task adds the a11y layer on top)
Blocks:     MGMT-UI-011 (accessibility aria + keyboard nav implementation — this design contract is the direct input for all aria attribute and keyboard-nav implementation decisions)
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "DESIGN-MGMT-007",
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
      "verified_by": [
        {"task_id": "MGMT-UI-011", "ac_id": "AC-1"},
        {"task_id": "E2E-MGMT-UI-001", "ac_id": "AC-5"}
      ],
      "description": "GIVEN the a11y contract section of DESIGN-ANNOTATIONS.md covers the four-tab navigation WHEN a reviewer inspects the aria attribute specification THEN it specifies: TabList element has role='tablist' and aria-label='Governance configuration tabs'; each TabTrigger element has role='tab', aria-selected='true|false', aria-controls='{panel-id}', and id='{tab-id}'; each TabContent element has role='tabpanel', aria-labelledby='{tab-id}', and id='{panel-id}'; all four IDs are listed in canonical order (principals, groups, branch-gates, rules) with the exact id/panel-id pairs",
      "verify": "design review \u2014 reviewer confirms all five aria attribute entries (tablist label, tab role, tab aria-selected, tab aria-controls, tabpanel aria-labelledby) are present with the exact attribute names and value patterns"
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "primary": false,
      "verified_by": [
        {"task_id": "MGMT-UI-011", "ac_id": "AC-1"},
        {"task_id": "E2E-MGMT-UI-001", "ac_id": "AC-5"}
      ],
      "description": "GIVEN the contract specifies the keyboard navigation model WHEN a reviewer reads the keyboard-nav contract THEN it states the exact key-to-action mapping: (1) Tab from outside the tablist: focus moves to the currently active TabTrigger (not to the first tab); (2) Arrow Left: focus moves to the previous TabTrigger in the list (wraps from principals to rules); (3) Arrow Right: focus moves to the next TabTrigger (wraps from rules to principals); (4) Enter or Space: activates the focused tab (sets aria-selected='true', renders the panel); (5) Tab from within the tab panel: focus leaves the tabpanel into the next focusable element (does NOT return to the tablist); this is the WAI-ARIA automatic-activation model",
      "verify": "design review \u2014 reviewer confirms all five key-to-action entries are present and the automatic-activation model is named"
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "verified_by": [
        {"task_id": "MGMT-UI-011", "ac_id": "AC-1"}
      ],
      "description": "GIVEN the contract specifies the focus-visible treatment WHEN a reviewer inspects the focus-visible section THEN it states that the active-focus TabTrigger shows a visible outline using the existing var(--focus-outline) CSS variable (or the browser's native :focus-visible outline if the Tabs component delegates it); no custom focus ring is introduced; the focus-visible style must not be suppressed with outline:none without a replacement",
      "verify": "design review \u2014 reviewer confirms var(--focus-outline) or :focus-visible delegation is cited, and the no-suppress-without-replacement rule is stated"
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the contract cites the existing shared/Tabs implementation WHEN a reviewer checks the component audit note THEN it explicitly states whether apps/desktop/src/components/shared/Tabs.svelte already implements the WAI-ARIA tab role attributes and roving-tabindex keyboard model, or whether the sveltekit-implementer must augment it \u2014 providing a concrete action item ('Tabs.svelte already has role=tablist: verify aria-label is settable via a prop' OR 'Tabs.svelte does not set role=tab on TabTriggers: implementer must add the aria attributes')",
      "verify": "design review \u2014 reviewer confirms the component audit note is present with a concrete action item for the implementer, not a hedge"
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the a11y contract section extends DESIGN-ANNOTATIONS.md WHEN a reviewer audits every visual reference in the a11y section THEN every visual attribute uses an existing CSS variable \u2014 no hex literals, no new var(--a11y-*)/var(--mgmt-*) tokens; focus treatment uses var(--focus-outline) or defers to :focus-visible \u2014 not a new token",
      "verify": "grep the a11y section of DESIGN-ANNOTATIONS.md for '#' color literals and var(--a11y-)/var(--mgmt-) -> both return zero matches"
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "contract names role='tablist' + aria-label='Governance configuration tabs' on TabList; role='tab' + aria-selected + aria-controls + id on each TabTrigger; role='tabpanel' + aria-labelledby + id on each TabContent; all four id pairs listed in canonical order",
      "verify": "design review of the aria attribute specification in DESIGN-ANNOTATIONS.md",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "contract names all five key-to-action mappings (Tab-in, Arrow Left, Arrow Right, Enter/Space, Tab-out) and identifies the automatic-activation WAI-ARIA model",
      "verify": "design review of the keyboard-nav contract in DESIGN-ANNOTATIONS.md",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "contract names var(--focus-outline) or :focus-visible delegation for the focused TabTrigger; states the no-suppress-without-replacement rule",
      "verify": "design review of the focus-visible section in DESIGN-ANNOTATIONS.md",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "contract includes a concrete Tabs.svelte component audit note with an actionable finding (aria already present vs must add) for the implementer",
      "verify": "design review of the component audit note section; grep apps/desktop/src/components/shared/Tabs.svelte for role= and aria- attributes",
      "maps_to_ac": "AC-4"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "zero hex color literals and zero new var(--a11y-*)/var(--mgmt-*) tokens in the a11y section",
      "verify": "grep -nE '#[0-9a-fA-F]{3,6}|var\\(--(a11y|mgmt)-' apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md -> zero matches",
      "maps_to_ac": "AC-5"
    }
  ]
}
-->
