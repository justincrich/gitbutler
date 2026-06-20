# DESIGN-MGMT-001: Wireframe-fidelity + visual-state annotations for all four tabs

## What this does

Produces a single state-and-layout annotation that maps every ASCII wireframe region in `10-technical-requirements/10-ui-infrastructure.md` to the exact reused component (at its source path) + props/variant, covering all four tab states (Principals · Groups · Branch Gates · Rules) and the cross-cutting overlay states (pending, read-only, denial, empty). It is the single design source of truth for every MGMT-UI-* implementer in Sprint 06a. **No new design-system work** — every token, control, and feedback component already exists in `packages/ui` or `apps/desktop/src/components/shared`.

## Why

Sprint 06a · PRD UC-MGMT-01..05 · `10-ui-infrastructure.md` Wireframes + Component-reuse + Net-new tables. Without one authoritative annotation, each UI implementer independently guesses component choices, drifting from the wireframes and the reuse contract.

## How to verify

PRIMARY **AC-1** — design review: every wireframe region listed in the `10-ui-infrastructure.md` ASCII diagrams (Principals list, Per-principal editor, Groups tab, Branch Gates tab, Rules tab, cross-cutting states) is mapped to a named component at its exact source path (e.g. `packages/ui/src/lib/components/Toggle.svelte disabled=true` for inherited rows). Full gate set in the spec below.

## Scope

- `apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md` (NEW) — the reviewable annotation artifact (a state matrix; rows = wireframe regions, columns = default/pending/read-only/denial/empty).

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: DESIGN-MGMT-001 — Wireframe-fidelity + visual-state annotations for all four tabs
================================================================================

TASK_TYPE:  DESIGN
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     M  (60 min)
AGENT:      designer=frontend-designer | reviewer=design-reviewer
PROPOSED-BY: frontend-designer
SPRINT:     ./SPRINT.md
PRD_REFS:   UC-MGMT-01, UC-MGMT-02, UC-MGMT-03, UC-MGMT-04, UC-MGMT-05
CAPABILITIES: (none — design-spec artifact)

RUNTIME_COMMANDS:
  review: design review of apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md against the
          10-ui-infrastructure.md Component-reuse table (every region cites a real component path)
  token-audit: grep the annotation for hex color literals and var(--governance-*)/var(--mgmt-*) -> zero matches
  downstream: the MGMT-UI-003/005/006/007/008 component tests (pnpm test:ct:desktop) consume this contract

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
A design reviewer can read the annotation and unambiguously identify (a) which existing component renders
each wireframe region, (b) which prop values produce the correct visual variant, and (c) which state
transitions apply across all four tabs — with zero ambiguity requiring an implementer to invent component choices.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] Map every wireframe region to an existing component at its exact path in packages/ui or
  apps/desktop/src/components/shared — no invented component names.
- [MUST] Produce a SINGLE page / FOUR tab-STATE annotation matrix, not four separate screen designs.
- [MUST] Cover all named states: default list, inline editor open, pending (○), read-only, denial banner,
  and all four empty states.
- [MUST] Cite component props verbatim from source (e.g. Toggle disabled=true; InfoMessage style='warning'
  outlined=true primaryLabel='Commit changes'). Branch Gates + Rules tabs annotated for IA completeness even
  though their implementation is Sprint 06b.
- [NEVER] Introduce any new design-system token, CSS variable, color, or spacing not already in packages/ui.
- [NEVER] Produce per-platform or per-variant duplicate screens — one governance page, multiple named STATES.
- [NEVER] Propose net-new components beyond the seven in the Net-new components table.
- [NEVER] Annotate implementation logic (SDK calls, store structure) — annotate the visual layer only.
- [STRICTLY] Component paths must match the verified source table in 10-ui-infrastructure.md exactly.
- [STRICTLY] Tab IA order must be Principals · Groups · Branch Gates · Rules.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: every wireframe region mapped to an exact component path
- [ ] AC-2: tab IA annotated with Tabs.svelte + all four tab IDs in order
- [ ] AC-3: four empty states annotated with EmptyStatePlaceholder + slot content
- [ ] AC-4: all three InfoMessage variants (warning/info/danger) + both Badge states annotated
- [ ] AC-5: no new design tokens — all visual attributes use existing CSS variables/component stylesheets

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (design-spec; verified by review + downstream component tests)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: Every wireframe region mapped to an exact component path
  GIVEN: the annotation document exists as a reviewable artifact
  WHEN:  a design reviewer reads the four-tab state matrix
  THEN:  every wireframe region (Principals list, per-principal editor, Groups, Branch Gates, Rules,
         cross-cutting states) is mapped to a named component at its exact source path
  TEST_TIER: unit   UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test (pnpm test:ct:desktop)
  VERIFY: design review — each region's entry cites a real path from the Component-reuse table

AC-2: Tab IA annotated with Tabs.svelte + all four tab IDs in order
  GIVEN: the annotation covers tab IA
  WHEN:  a reviewer checks the tab navigation region
  THEN:  the tab strip maps to apps/desktop/src/components/shared/Tabs.svelte (+TabList/TabTrigger/TabContent)
         with defaultSelected='principals', aria-label='Governance configuration tabs', and all four tab ids
         in order: principals, groups, branch-gates, rules
  TEST_TIER: unit   UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test
  VERIFY: reviewer confirms exact component path + all four tab IDs present

AC-3: Four empty states annotated with EmptyStatePlaceholder + slot content
  GIVEN: the annotation covers both states (empty + populated) of each tab
  WHEN:  a reviewer inspects each of the four tab states
  THEN:  each tab has an annotated empty state citing EmptyStatePlaceholder
         (packages/ui/src/lib/components/EmptyStatePlaceholder.svelte) with the exact title/caption/actions
         slot content from the wireframe, and a populated state citing the row/section component
  TEST_TIER: unit   UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test
  VERIFY: reviewer confirms four empty-state entries each with EmptyStatePlaceholder path + slot content

AC-4: All three InfoMessage variants + both Badge states annotated
  GIVEN: the annotation covers cross-cutting overlay states
  WHEN:  a reviewer checks the cross-cutting state section
  THEN:  it lists all three InfoMessage banner variants (warning/info/danger) with exact props, the pending ○
         Badge, the committed ● Badge, and the page-level read-only + denial states — each citing path + props
         (InfoMessage path with style prop + action slots; Badge style='warning' kind='soft' pending; style='gray' kind='soft' committed)
  TEST_TIER: unit   UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test
  VERIFY: reviewer confirms InfoMessage warning/info/danger + Badge pending/committed entries cite paths + props

AC-5: No new design tokens — all visual attributes use existing CSS variables/component stylesheets
  GIVEN: the annotation introduces no new design-system tokens
  WHEN:  a reviewer audits every color/spacing/typography reference
  THEN:  every visual attribute is an existing CSS variable (var(--text-3), var(--fill-warn-bg), var(--bg-2))
         or deferred to the component's own stylesheet — no hex literals, no new var(--governance-*)/var(--mgmt-*)
  TEST_TIER: unit   UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review inspection
  VERIFY: grep the annotation for '#' color literals and var(--governance-)/var(--mgmt-) -> both return zero matches

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): all wireframe regions in the 10-ui-infrastructure.md ASCII diagrams have an annotation entry naming the exact component file path
- TC-2 (-> AC-2): the tab IA annotation cites apps/desktop/src/components/shared/Tabs.svelte with all four tab IDs in order (principals, groups, branch-gates, rules)
- TC-3 (-> AC-3): four empty-state entries present, each citing EmptyStatePlaceholder.svelte with wireframe-matching title/caption/actions slot content
- TC-4 (-> AC-4): three InfoMessage variants (warning, info, danger) + both Badge states (pending warning/soft, committed gray/soft) annotated with paths + props
- TC-5 (-> AC-5): zero hex color literals and zero new var(--governance-*)/var(--mgmt-*) tokens in the annotation artifact

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: (none — design-spec artifact)
provides: the authoritative wireframe-region -> component annotation consumed by MGMT-UI-003/005/006/007/008
consumes: 10-ui-infrastructure.md Wireframes + Component-reuse + Net-new tables; the existing packages/ui + shared components

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md (NEW)
writeProhibited:
  - packages/ui/src/lib/components/** ; apps/desktop/src/components/shared/** ; packages/ui/src/lib/styles/**
  - any CSS custom-property declaration file ; any .svelte or .ts implementation file

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. .spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md — Wireframes (all five ASCII diagrams) + Component reuse table + Net-new components table + Cross-cutting states (REQUIRED READING)
2. .spec/prds/governance/08-uc-mgmt.md — UC-MGMT-01..05 (tab IA, editor, groups, branch gates, rules)
3. packages/ui/src/lib/components/InfoMessage.svelte — style 'warning'|'info'|'danger', outlined, primaryLabel, primaryAction
4. packages/ui/src/lib/components/Badge.svelte — style 'warning'|'gray', kind 'soft'
5. packages/ui/src/lib/components/{Toggle,segmentControl/SegmentControl,TagInput,EmptyStatePlaceholder}.svelte
6. apps/desktop/src/components/shared/{Tabs,ExpandableSection,SettingsSection}.svelte ; apps/desktop/src/components/rules/RuleEditor.svelte (slide-in pattern)

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- Design review: every annotated region cites an existing packages/ui or apps/desktop/src/components/shared path (provable by grep against the Component-reuse table)
- Design review: annotation covers all five wireframe sections + all cross-cutting states
- Token audit: grep finds no hex literals and no new var(--governance-*)/var(--mgmt-*) tokens
- Downstream: MGMT-UI-003/005/006/007/008 implementers confirm the annotation was sufficient to build without inventing component choices (sprint review)

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: Settings-section composition — SettingsModalLayout sidebar item -> ProjectSettingsModalContent branch
  -> GovernanceSettings.svelte (Tabs wrapper + pending banner slot) -> per-tab content. Cross-cutting states
  overlaid via GovernancePendingBanner (above the tab strip) + per-row Badge indicators.
pattern_source: apps/desktop/src/components/rules/RuleEditor.svelte (slide-in inline editor); existing
  *Settings.svelte sections (SettingsSection + SettingsModalLayout composition)
anti_pattern: per-variant screens (one screen per tab state); any new design-system token/component outside the
  verified reuse table; annotating Rust/SDK/store logic.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
designer: frontend-designer — owns wireframe-fidelity mapping + visual-state annotation against existing component APIs; no Rust/Tauri knowledge required
reviewer: design-reviewer

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: none
Blocks:     MGMT-UI-003, MGMT-UI-005, MGMT-UI-006, MGMT-UI-007, MGMT-UI-008
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "DESIGN-MGMT-001",
  "proposed_by": "frontend-designer",
  "verification_policy": { "requires_tests": false, "requires_red_evidence": false, "requires_seeded_evidence": false },
  "fixtures": {},
  "requirements": [
    { "id": "AC-1", "type": "acceptance_criterion", "primary": true, "test_tier": "unit", "unit_test_justified": "design-spec artifact, no runtime I/O — verified by design review + downstream component test (pnpm test:ct:desktop)", "description": "GIVEN the annotation document exists WHEN a reviewer reads the four-tab state matrix THEN every wireframe region is mapped to a named component at its exact source path", "verify": "design review — each region's entry cites a real path from the 10-ui-infrastructure.md Component-reuse table" },
    { "id": "AC-2", "type": "acceptance_criterion", "primary": false, "test_tier": "unit", "unit_test_justified": "design-spec artifact, no runtime I/O — verified by design review + downstream component test", "description": "GIVEN the annotation covers tab IA WHEN a reviewer checks the tab navigation THEN it maps to shared/Tabs.svelte (+TabList/TabTrigger/TabContent) with defaultSelected='principals', aria-label, and all four tab ids in order", "verify": "reviewer confirms the exact Tabs component path + all four tab IDs (principals, groups, branch-gates, rules)" },
    { "id": "AC-3", "type": "acceptance_criterion", "primary": false, "test_tier": "unit", "unit_test_justified": "design-spec artifact, no runtime I/O — verified by design review + downstream component test", "description": "GIVEN the annotation covers empty+populated states of each tab WHEN a reviewer inspects each THEN each tab has an annotated empty state citing EmptyStatePlaceholder with exact slot content and a populated state citing the row/section component", "verify": "reviewer confirms four empty-state entries each with EmptyStatePlaceholder path + slot content" },
    { "id": "AC-4", "type": "acceptance_criterion", "primary": false, "test_tier": "unit", "unit_test_justified": "design-spec artifact, no runtime I/O — verified by design review + downstream component test", "description": "GIVEN the annotation covers cross-cutting overlay states WHEN a reviewer checks them THEN it lists InfoMessage warning/info/danger with props, the pending ○ Badge, the committed ● Badge, and the read-only + denial states, each citing path + props", "verify": "reviewer confirms InfoMessage warning/info/danger + Badge pending(warning/soft)/committed(gray/soft) entries cite paths + props" },
    { "id": "AC-5", "type": "acceptance_criterion", "primary": false, "test_tier": "unit", "unit_test_justified": "design-spec artifact, no runtime I/O — verified by design review inspection", "description": "GIVEN the annotation introduces no new design-system tokens WHEN a reviewer audits every color/spacing/typography reference THEN every attribute uses an existing CSS variable or the component's own stylesheet — no hex literals, no new var(--governance-*)", "verify": "grep the annotation for '#' color literals and var(--governance-)/var(--mgmt-) -> both return zero matches" },
    { "id": "TC-1", "type": "test_criterion", "description": "all wireframe regions in the ASCII diagrams have an annotation entry naming the exact component file path", "verify": "design review against the Component-reuse table", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "the tab IA annotation cites shared/Tabs.svelte with all four tab IDs in order (principals, groups, branch-gates, rules)", "verify": "design review of the tab IA section", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "four empty-state entries present, each citing EmptyStatePlaceholder.svelte with wireframe-matching slot content", "verify": "design review of the four empty-state entries", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "three InfoMessage variants + both Badge states annotated with paths + props", "verify": "design review of the cross-cutting state section", "maps_to_ac": "AC-4" },
    { "id": "TC-5", "type": "test_criterion", "description": "zero hex color literals and zero new var(--governance-*)/var(--mgmt-*) tokens in the annotation", "verify": "grep -nE '#[0-9a-fA-F]{3,6}|var\\(--(governance|mgmt)-' apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md -> zero matches", "maps_to_ac": "AC-5" }
  ]
}
-->
</details>
