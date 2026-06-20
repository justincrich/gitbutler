# DESIGN-MGMT-005: Inherited-vs-own permission row distinction in `PrincipalEditor`

> **Red-Hat Remediation (cycle 1):** S10 (LOW) resolved — added AC-3/TC-3 specifying the "both" row type (own-grant AND group-inherited simultaneously); old AC-3/4/5 and TC-3/4/5 renumbered to AC-4/5/6 and TC-4/5/6; contract JSON updated and remains gapless.

## What this does

Defines the visual contract for the two-column FUNCTIONAL PERMISSIONS table inside `PrincipalEditor`: how inherited (group-derived, read-only) rows look vs own-grant (editable) rows, how the role-preset `SegmentControl` interacts with own-grant toggles, and the union-semantics rule that prevents an inherited grant from being toggled off from this editor. Direct input for MGMT-UI-007. **No new design-system work** — `Toggle`, `Badge`, `SegmentControl`, `TagInput`, `Select` already exist.

## Why

Sprint 06a · PRD UC-MGMT-02 (inherited rows read-only, union semantics, batch-save B16) · `10-ui-infrastructure.md` Per-principal permission editor wireframe. Union semantics must be communicated visually so an admin sees *why* a principal holds a permission and cannot accidentally try to revoke an inherited grant from the wrong surface.

## How to verify

PRIMARY **AC-1** — design review: inherited rows are specified as `Toggle disabled=true`, SOURCE column shows `Badge style='gray' kind='soft'` with `[group: {name}]`, GRANT column shows `── inherited ──` in `var(--text-3)`; no pending Badge on inherited rows; no row-background change. Full gate set in the spec below.

## Scope

- `apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md` (MODIFY — extend with the PrincipalEditor inherited-vs-own section).

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: DESIGN-MGMT-005 — Inherited-vs-own permission row distinction in PrincipalEditor
================================================================================

TASK_TYPE:  DESIGN
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     S  (40 min)
AGENT:      designer=frontend-designer | reviewer=design-reviewer
PROPOSED-BY: frontend-designer
SPRINT:     ./SPRINT.md
PRD_REFS:   UC-MGMT-02
CAPABILITIES: (none — design-spec artifact)

RUNTIME_COMMANDS:
  review: design review of the PrincipalEditor inherited-vs-own section of DESIGN-ANNOTATIONS.md against the editor wireframe
  downstream: the MGMT-UI-007 PrincipalEditor component test asserts inherited row Toggle disabled + own-grant row interactive (pnpm test:ct:desktop, T-MGMT-027)

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
A sveltekit-implementer can render the PrincipalEditor permission table with exactly the right Toggle states,
source-column content, and grant-column content per row, and knows what the SegmentControl preset does to
own-grant rows without touching inherited rows — all using existing components, no design judgment required.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] Specify inherited rows: Toggle disabled=true; SOURCE column shows a Badge/text chip '[group: eng]';
  GRANT column shows '── inherited ──' in var(--text-3) (muted); row background NOT distinct (same var(--bg-1)).
- [MUST] Specify own-grant rows: Toggle not disabled; SOURCE column 'own grant'; GRANT column the Toggle's checked/unchecked state.
- [MUST] Specify how the role-preset SegmentControl interacts with own-grant toggles WITHOUT touching inherited rows.
- [MUST] State the union-semantics rule visually: an inherited row is always locked regardless of the selected preset; the SegmentControl desugars to own-grant toggles only.
- [MUST] Specify the inherited source chip as a Badge style='gray' kind='soft' with label '[group: eng]'.
- [NEVER] Apply a different background/border to inherited rows — the distinction is the disabled Toggle + the source chip.
- [NEVER] Allow an inherited row to show a pending (○) Badge — pending is own-grant only.
- [NEVER] Introduce new CSS tokens or inline color values.
- [STRICTLY] '── inherited ──' grant-column text uses var(--text-3) (existing muted token) — not a new token.
- [STRICTLY] The inherited source chip uses Badge style='gray' kind='soft'; the [+ Add to group] affordance uses Select/SelectItem (not a custom dropdown).

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: inherited row — Toggle disabled, Badge gray/soft source chip, '── inherited ──' var(--text-3), no bg change
- [ ] AC-2: own-grant row — Toggle enabled with checked variants, 'own grant' source, pending Badge on staged changes
- [ ] AC-3: "both" row (own-grant AND group-inherited simultaneously) — renders as inherited/disabled; SOURCE shows the group Badge; own-grant cannot be revoked from PrincipalEditor while inherited grant exists
- [ ] AC-4: SegmentControl drives only own-grant Toggles in local state; inherited rows unaffected
- [ ] AC-5: union-semantics rule — inherited cannot be revoked from PrincipalEditor; Groups tab is the revoke path
- [ ] AC-6: TagInput for group chips, Select/SelectItem for add-to-group; readonly treatment in read-only mode

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (design-spec; verified by review + the downstream MGMT-UI-007 component test)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: Inherited row treatment
  GIVEN: the inherited-vs-own contract exists
  WHEN:  a reviewer inspects the inherited-row spec
  THEN:  Toggle disabled=true; SOURCE column Badge style='gray' kind='soft' label '[group: {groupName}]'; GRANT
         column '── inherited ──' in var(--text-3); no pending Badge on inherited rows; row background default (var(--bg-1))
  TEST_TIER: unit   UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test (pnpm test:ct:desktop)
  VERIFY: reviewer confirms Toggle disabled=true + Badge gray/soft + '── inherited ──' in var(--text-3) + no pending Badge + no bg change

AC-2: Own-grant row treatment
  GIVEN: the contract specifies own-grant rows
  WHEN:  a reviewer inspects the own-grant-row spec
  THEN:  Toggle not disabled, checked=true for active own grants / false for inactive; SOURCE 'own grant' in var(--text-2);
         GRANT column the Toggle; a pending Badge warning/soft inline when the own-grant is toggled but not yet committed (local UI state, batch-save B16)
  TEST_TIER: unit   UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test
  VERIFY: reviewer confirms Toggle not-disabled + checked variants + 'own grant' in var(--text-2) + pending Badge on staged own-grant changes

AC-3: "Both" row treatment (own-grant AND group-inherited simultaneously)
  GIVEN: the contract specifies the "both" row type — a principal holds both an own-grant and a group-inherited grant
         for the same permission (e.g., principal "alice" has an explicit own-grant for 'contents:write' AND inherits
         it via group "eng")
  WHEN:  a reviewer inspects the both-row spec and its example in DESIGN-ANNOTATIONS.md
  THEN:  the row renders as inherited/disabled (inherited source takes precedence): Toggle disabled=true; SOURCE column
         shows the group Badge (style='gray' kind='soft', label '[group: eng]') — NOT 'own grant'; GRANT column shows
         '── inherited ──' in var(--text-3); no pending Badge; no row-background change; a tooltip or sub-text note
         MAY indicate the own-grant exists but is superseded; the own-grant CANNOT be revoked from PrincipalEditor while
         the inherited grant exists — the admin must remove alice from the "eng" group in the Groups tab first, after
         which the row transitions to an own-grant row and becomes editable; a concrete example row is present in
         DESIGN-ANNOTATIONS.md
  TEST_TIER: unit   UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test
  VERIFY: reviewer confirms Toggle disabled + group Badge SOURCE (not 'own grant') + '── inherited ──' in var(--text-3) + no pending Badge + cannot-revoke-while-inherited rule + Groups-tab-first removal path + example row in DESIGN-ANNOTATIONS.md

AC-4: SegmentControl interaction
  GIVEN: the contract specifies SegmentControl interaction with the table
  WHEN:  a reviewer checks the preset -> Toggle interaction rule
  THEN:  selecting a role preset (SegmentControl read/triage/write/maintain/admin) sets checked=true on the own-grant
         Toggles for that preset's desugared set and false outside it; inherited rows never touched (Toggles stay disabled);
         the onselect callback drives local UI state only, not an immediate SDK write
  TEST_TIER: unit   UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test
  VERIFY: reviewer confirms the inherited-rows-unaffected rule + local-UI-state-only rule + the SegmentControl component path

AC-5: Union-semantics visual rule
  GIVEN: the contract specifies the union-semantics rule
  WHEN:  a reviewer reads the union-semantics section
  THEN:  the effective permission is own ∪ group; an inherited 'contents:write' shows as inherited/disabled even if a
         preset omitting it is selected — the inherited grant cannot be revoked from PrincipalEditor (remove the principal
         from the group in the Groups tab); an example row is included
  TEST_TIER: unit   UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test
  VERIFY: reviewer confirms the cannot-revoke-inherited rule with the Groups-tab removal path + an example row

AC-6: Group chips + add-to-group affordance
  GIVEN: the contract specifies the GROUPS section
  WHEN:  a reviewer inspects the editor's GROUPS region
  THEN:  existing memberships are TagInput tags (label=group name; remove ✕ triggers a staged group removal, batch-saved);
         the [+ Add to group] affordance uses Select/SelectItem with options from the groups list; in read-only mode TagInput readonly=true + Select disabled
  TEST_TIER: unit   UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test
  VERIFY: reviewer confirms TagInput path + Select/SelectItem path + read-only treatment + batch-save semantics

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): contract names Toggle disabled=true, Badge gray/soft source chip, '── inherited ──' in var(--text-3), no row-background change for inherited rows
- TC-2 (-> AC-2): contract names Toggle not-disabled with checked variants, 'own grant' in var(--text-2), and pending Badge warning/soft for staged own-grant changes
- TC-3 (-> AC-3): contract names the "both" row: Toggle disabled=true, SOURCE shows group Badge (not 'own grant'), '── inherited ──' in var(--text-3), no pending Badge, cannot-revoke-while-inherited rule (Groups-tab removal required), and a concrete example row in DESIGN-ANNOTATIONS.md
- TC-4 (-> AC-4): contract states SegmentControl drives only own-grant Toggles (not inherited rows) and onselect updates local UI state only
- TC-5 (-> AC-5): contract states inherited grants cannot be revoked from PrincipalEditor; Groups tab is the only revoke path; example row present
- TC-6 (-> AC-6): contract names TagInput for group chips, Select/SelectItem for add-to-group, and readonly/disabled treatment in read-only mode

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: (none — design-spec artifact)
provides: the inherited-vs-own row visual contract consumed by MGMT-UI-007 (PrincipalEditor)
consumes: 10-ui-infrastructure.md Per-principal editor wireframe; packages/ui Toggle/Badge/SegmentControl/TagInput/Select; apps/desktop rules/RuleEditor.svelte (slide-in layout)

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md (MODIFY — PrincipalEditor inherited-vs-own section)
writeProhibited:
  - packages/ui/src/lib/components/** ; apps/desktop/src/components/shared/** ; any .svelte or .ts implementation file

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. .spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md — Per-principal permission editor wireframe (FUNCTIONAL PERMISSIONS table + GROUPS row)
2. .spec/prds/governance/08-uc-mgmt.md — UC-MGMT-02 (inherited read-only, union semantics, batch-save B16, SegmentControl preset)
3. packages/ui/src/lib/components/Toggle.svelte — disabled prop (:disabled opacity 0.6, pointer-events none)
4. packages/ui/src/lib/components/Badge.svelte — style='gray' kind='soft' (source group chip) ; segmentControl/SegmentControl.svelte (selected, onselect)
5. packages/ui/src/lib/components/TagInput.svelte (tags, readonly, onRemoveTag) ; select/Select.svelte ([+ Add to group]) ; apps/desktop/src/components/rules/RuleEditor.svelte (slide-in layout)

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- Design review: contract covers all three row types (inherited, own-grant, both) with exact component props
- Design review: the SegmentControl interaction rule (own-grant only, local state only) is unambiguous
- Design review: the union-semantics / cannot-revoke-inherited rule is stated with an example
- Downstream: the MGMT-UI-007 component test asserts the inherited row Toggle is disabled and the own-grant row Toggle is interactive (T-MGMT-027)

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: a two-column (SOURCE | GRANT) table inside the PrincipalEditor panel; rows rendered by {#each permissions}
  distinguishing inherited (source==='group') from own-grant (source==='own'|'none'); SegmentControl above the table
  drives own-grant checked states via local state only.
pattern_source: apps/desktop/src/components/rules/RuleEditor.svelte (inline slide-in panel); Toggle.svelte (disabled visual); Badge.svelte (group source chip)
anti_pattern: a background color/border to distinguish inherited rows (the disabled Toggle + source chip suffice); a pending Badge on inherited rows; per-toggle SDK writes (batch-save only).

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
designer: frontend-designer — a nuanced visual contract for union semantics over existing Toggle/Badge; no backend knowledge needed
reviewer: design-reviewer

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: DESIGN-MGMT-001, DESIGN-MGMT-003
Blocks:     MGMT-UI-007
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "DESIGN-MGMT-005",
  "proposed_by": "frontend-designer",
  "verification_policy": { "requires_tests": false, "requires_red_evidence": false, "requires_seeded_evidence": false },
  "fixtures": {},
  "requirements": [
    { "id": "AC-1", "type": "acceptance_criterion", "primary": true, "test_tier": "unit", "unit_test_justified": "design-spec artifact, no runtime I/O — verified by design review + downstream component test (pnpm test:ct:desktop)", "description": "GIVEN the inherited-vs-own contract WHEN a reviewer inspects the inherited-row spec THEN Toggle disabled=true, SOURCE Badge style='gray' kind='soft' '[group: {name}]', GRANT '── inherited ──' in var(--text-3), no pending Badge, row background default", "verify": "reviewer confirms Toggle disabled=true + Badge gray/soft + '── inherited ──' var(--text-3) + no pending Badge + no bg change" },
    { "id": "AC-2", "type": "acceptance_criterion", "primary": false, "test_tier": "unit", "unit_test_justified": "design-spec artifact, no runtime I/O — verified by design review + downstream component test", "description": "GIVEN the contract specifies own-grant rows WHEN a reviewer inspects it THEN Toggle not-disabled with checked variants, SOURCE 'own grant' var(--text-2), GRANT the Toggle, and a pending Badge warning/soft when toggled-but-uncommitted (local state, batch-save)", "verify": "reviewer confirms Toggle not-disabled + checked variants + 'own grant' var(--text-2) + pending Badge on staged changes" },
    { "id": "AC-3", "type": "acceptance_criterion", "primary": false, "test_tier": "unit", "unit_test_justified": "design-spec artifact, no runtime I/O — verified by design review + downstream component test", "description": "GIVEN the contract specifies the 'both' row type (principal holds both an own-grant and a group-inherited grant for the same permission) WHEN a reviewer inspects the both-row spec and its example in DESIGN-ANNOTATIONS.md THEN the row renders as inherited/disabled: Toggle disabled=true; SOURCE shows the group Badge (style='gray' kind='soft') — NOT 'own grant'; GRANT '── inherited ──' in var(--text-3); no pending Badge; no bg change; the own-grant cannot be revoked from PrincipalEditor while the inherited grant exists (admin must remove principal from the group in the Groups tab first); a concrete example row is present in DESIGN-ANNOTATIONS.md", "verify": "reviewer confirms Toggle disabled + group Badge SOURCE (not 'own grant') + '── inherited ──' var(--text-3) + no pending Badge + cannot-revoke-while-inherited rule + Groups-tab-first removal path + example row in DESIGN-ANNOTATIONS.md" },
    { "id": "AC-4", "type": "acceptance_criterion", "primary": false, "test_tier": "unit", "unit_test_justified": "design-spec artifact, no runtime I/O — verified by design review + downstream component test", "description": "GIVEN the contract specifies SegmentControl interaction WHEN a reviewer checks the preset->Toggle rule THEN selecting a preset sets own-grant Toggles to the desugared set without touching inherited rows (Toggles stay disabled), driving local UI state only (no immediate SDK write)", "verify": "reviewer confirms the inherited-rows-unaffected rule + local-UI-state-only rule + the SegmentControl component path" },
    { "id": "AC-5", "type": "acceptance_criterion", "primary": false, "test_tier": "unit", "unit_test_justified": "design-spec artifact, no runtime I/O — verified by design review + downstream component test", "description": "GIVEN the union-semantics rule WHEN a reviewer reads it THEN it states the effective permission is own ∪ group, an inherited grant shows inherited/disabled even when a preset omits it, the inherited grant cannot be revoked from PrincipalEditor (Groups-tab removal is the only path), with an example row", "verify": "reviewer confirms the cannot-revoke-inherited rule + Groups-tab removal path + an example row" },
    { "id": "AC-6", "type": "acceptance_criterion", "primary": false, "test_tier": "unit", "unit_test_justified": "design-spec artifact, no runtime I/O — verified by design review + downstream component test", "description": "GIVEN the contract specifies the GROUPS section WHEN a reviewer inspects it THEN memberships are TagInput tags (remove ✕ = staged removal, batch-saved), [+ Add to group] uses Select/SelectItem from the groups list, and in read-only mode TagInput readonly=true + Select disabled", "verify": "reviewer confirms TagInput path + Select/SelectItem path + read-only treatment + batch-save semantics" },
    { "id": "TC-1", "type": "test_criterion", "description": "contract names Toggle disabled=true, Badge gray/soft source chip, '── inherited ──' in var(--text-3), no row-background change for inherited rows", "verify": "design review of the inherited-row section", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "contract names Toggle not-disabled with checked variants, 'own grant' in var(--text-2), and pending Badge warning/soft for staged own-grant changes", "verify": "design review of the own-grant-row section", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "contract names the 'both' row: Toggle disabled=true, SOURCE shows group Badge (not 'own grant'), '── inherited ──' in var(--text-3), no pending Badge, cannot-revoke-while-inherited rule (Groups-tab removal required first), and a concrete example row in DESIGN-ANNOTATIONS.md", "verify": "design review of the both-row section and its example in DESIGN-ANNOTATIONS.md", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "contract states SegmentControl drives only own-grant Toggles (not inherited rows) and onselect updates local UI state only", "verify": "design review of the SegmentControl interaction rule", "maps_to_ac": "AC-4" },
    { "id": "TC-5", "type": "test_criterion", "description": "contract states inherited grants cannot be revoked from PrincipalEditor; Groups tab is the only revoke path; example row present", "verify": "design review of the union-semantics section", "maps_to_ac": "AC-5" },
    { "id": "TC-6", "type": "test_criterion", "description": "contract names TagInput for group chips, Select/SelectItem for add-to-group, and readonly/disabled treatment in read-only mode", "verify": "design review of the GROUPS section", "maps_to_ac": "AC-6" }
  ]
}
-->
</details>
