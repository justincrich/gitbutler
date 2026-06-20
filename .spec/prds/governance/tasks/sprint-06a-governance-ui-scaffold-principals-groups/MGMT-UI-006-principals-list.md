# MGMT-UI-006: PrincipalsList (rows + inline editor toggle; inherited rows read-only)

> **Red-Hat Remediation (cycle 1):** Resolved S6 (MEDIUM) — DESIGN-MGMT-001 and DESIGN-MGMT-003 added to depends_on per the SPRINT.md dependency chain.

## What this does

The Principals tab component: renders each registered principal as a row with its effective `AuthoritySet`, source-of-grant labels (own vs group-inherited), a pending (○) indicator for staged-but-uncommitted rows, and an inline `PrincipalEditor` toggle (slide-in, mirroring `RulesList`/`RuleEditor` — no route, no modal). Loads principals via the `perm_list` SDK command; renders `EmptyStatePlaceholder` when none.

## Why

Sprint 06a · PRD UC-MGMT-02 · criteria T-MGMT-006/008/010 · capability CAP-AUTHZ-01. The heart of the governance surface — where an admin sees *which* permissions a principal effectively holds and *why* (source-of-grant).

## How to verify

PRIMARY **AC-1** — `pnpm test:ct:desktop -- PrincipalsListRows`: with 3 seeded principals, the tab renders 3 rows; the group-inherited `codex-agent` shows a `"group: eng"` label, `claude-agent` shows an `"own grant"` indicator. Full gate set in the spec below.

## Scope

- `apps/desktop/src/components/governance/PrincipalsList.svelte` (NEW)
- `apps/desktop/tests/governance/PrincipalsList.spec.ts` (NEW — CT specs)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: MGMT-UI-006 — PrincipalsList (rows + inline editor toggle; inherited rows read-only)
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P1
EFFORT:     M  (90 min)
AGENT:      implementer=sveltekit-implementer | reviewer=sveltekit-reviewer
PROPOSED-BY: sveltekit-planner
SPRINT:     ./SPRINT.md
PRD_REFS:   UC-MGMT-02, T-MGMT-006, T-MGMT-008, T-MGMT-010
CAPABILITIES: CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  pnpm test:ct:desktop -- PrincipalsListRows
  check: pnpm -F @gitbutler/desktop check   |   lint: pnpm lint

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Mounting PrincipalsList with 3 seeded principals (one own-grant, one group-inherited, one pending)
renders 3 rows. The group-inherited principal shows an "inherited"/"group: eng" marker. The pending
principal shows the ○ indicator. Clicking a row renders PrincipalEditor inline (no route, no modal).
Empty perm_list -> EmptyStatePlaceholder with a "+ Add first" action.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] Call perm_list (SDK) to load principals; wrap loading/error in ReduxResult.
- [MUST] Each row shows: principal id, effective AuthoritySet, source-of-grant per permission (own vs
  group-inherited), and a pending ○ indicator when the row has pending edits.
- [MUST] Clicking a principal row opens PrincipalEditor INLINE (slide-in, mirroring RuleEditor) — no new
  route, no blocking modal.
- [MUST] Effective display reflects the COMMITTED set; the pending ○ indicator is shown until committed
  (no optimistic effective-set update before commit).
- [NEVER] NEVER navigate to a new URL on row click; NEVER open a blocking modal; NEVER show effective
  grants optimistically before commit.
- [NEVER] NEVER show a pending ○ on inherited-only principals (pending is own-grant changes only).
- [STRICTLY] No relative imports — use @gitbutler/ references. No console.log. Component-scoped $state/$derived (no module-level state).

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: list shows effective set + source-of-grant labels
- [ ] AC-2: pending principal row shows ○ indicator
- [ ] AC-3: clicking a row opens PrincipalEditor inline
- [ ] AC-4: effective display updates only after commit (pending persists)
- [ ] AC-5: empty state shows EmptyStatePlaceholder with create-first action
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: principals list shows effective set and source-of-grant labels
  GIVEN: PrincipalsList mounted with 3 seeded principals
  WHEN:  the component renders
  THEN:  3 rows; codex-agent shows "group: eng" for contents:write; claude-agent shows "own grant" for pull_requests:write
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- PrincipalsListRows

AC-2: pending principal row shows ○ indicator
  GIVEN: seeded principals (cursor-bot pending=true)
  WHEN:  the component renders
  THEN:  the cursor-bot row shows a ○ pending Badge (style warning/soft)
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- PrincipalsListRows

AC-3: clicking a principal row opens PrincipalEditor inline
  GIVEN: seeded principals
  WHEN:  user clicks the claude-agent row
  THEN:  PrincipalEditor is rendered inline (no navigation, no modal) with claude-agent data
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- PrincipalsListEditorToggle

AC-4: effective display updates only after commit — pending persists until commit
  GIVEN: cursor-bot showing pending=true
  WHEN:  before commit fires
  THEN:  the ○ indicator persists; the effective grants shown are the committed set, not the working-tree draft
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- PrincipalsListPending

AC-5: empty state shown when perm_list returns no principals
  GIVEN: perm_list returns []
  WHEN:  the component renders
  THEN:  EmptyStatePlaceholder with "No principals configured" + "+ Add first" action
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- PrincipalsListEmpty

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): PrincipalsList with 3 seeded principals renders 3 rows with source-of-grant labels
    VERIFY: pnpm test:ct:desktop -- PrincipalsListRows
- TC-2 (-> AC-2): a principal with pending=true shows a ○ pending Badge in its row
    VERIFY: pnpm test:ct:desktop -- PrincipalsListRows
- TC-3 (-> AC-3): clicking a principal row renders PrincipalEditor inline without page navigation
    VERIFY: pnpm test:ct:desktop -- PrincipalsListEditorToggle
- TC-4 (-> AC-5): perm_list returning [] renders EmptyStatePlaceholder
    VERIFY: pnpm test:ct:desktop -- PrincipalsListEmpty
- TC-5 (-> AC-1): pnpm -F @gitbutler/desktop check exits 0 after MGMT-UI-006 lands
    VERIFY: pnpm -F @gitbutler/desktop check

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides: PrincipalsList.svelte — Principals tab with rows + inline editor toggle
consumes: MGMT-IPC-004 (perm_list command + Principal/AuthoritySet types); MGMT-UI-003 (readonly prop);
          MGMT-UI-007 (PrincipalEditor.svelte — inline per row); packages/ui CardGroup/Badge/KebabButton/
          Button/EmptyStatePlaceholder; apps/desktop ReduxResult
boundary_contracts:
  - reads via perm_list; renders effective set + source-of-grant; pending ○ until committed; inline editor (no route)

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - apps/desktop/src/components/governance/PrincipalsList.svelte (NEW)
  - apps/desktop/tests/governance/PrincipalsList.spec.ts (NEW)
writeProhibited:
  - apps/desktop/src/components/governance/PrincipalEditor.svelte (owned by MGMT-UI-007)
  - packages/ui components (reuse as-is)
  - apps/desktop/src/components/rules/RulesList.svelte (principalId prop is Sprint 06b)
  - any file not in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. packages/ui/src/lib/components/cardGroup/CardGroupRoot.svelte + CardGroupItem.svelte — row card layout
2. packages/ui/src/lib/components/Badge.svelte — warning/soft variant for the pending ○ indicator
3. packages/ui/src/lib/components/KebabButton.svelte — overflow actions on rows
4. packages/ui/src/lib/components/EmptyStatePlaceholder.svelte — empty-state pattern
5. apps/desktop/src/components/shared/ReduxResult.svelte — async loading/error wrapper
6. apps/desktop/src/components/rules/RulesList.svelte — slide-in inline editor pattern (mode state machine) to mirror for PrincipalEditor
7. .spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md — Principals-list wireframe ([●]/[○], source-of-grant, role badge)
8. .spec/prds/governance/08-uc-mgmt.md UC-MGMT-02 — principals list spec + register-on-first-grant

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- pnpm test:ct:desktop -- PrincipalsListRows           -> Exit 0
- pnpm test:ct:desktop -- PrincipalsListEditorToggle   -> Exit 0
- pnpm test:ct:desktop -- PrincipalsListEmpty          -> Exit 0
- pnpm -F @gitbutler/desktop check                     -> Exit 0
- pnpm lint                                            -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references: DESIGN-MGMT-001 (Principals-list region mapping); DESIGN-MGMT-002 (per-row pending Badge);
  DESIGN-MGMT-003 (isReadOnly prop-drilled: Toggles disabled, Add disabled)
notes (from enrichment): CardGroupRoot > {#each principals} CardGroupItem[name, Badge[role], groupChips,
  permSummary, {#if hasPending}Badge warning/soft{/if}, KebabButton] {#if isEditing} PrincipalEditor {/if};
  {#if principals.length === 0} EmptyStatePlaceholder. Clicking a row opens PrincipalEditor below the row
  (slide-in, not a modal). In read-only mode: KebabButton edit/remove omitted, "+ Add" disabled.
pattern: mirrors RulesList.svelte slide-in (mode list/edit) + Drawer
pattern_source: apps/desktop/src/components/rules/RulesList.svelte
anti_pattern: route navigation on row click; blocking modal; optimistic effective-set before commit; pending ○ on inherited-only rows

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: sveltekit-implementer
reviewer: sveltekit-reviewer
coding_standards: apps/desktop/AGENTS.md, frontend.md

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: MGMT-UI-003 (readonly prop + page host), MGMT-IPC-004 (perm_list SDK);
            DESIGN-MGMT-001 (four-tab IA + Principals-list region mapping), DESIGN-MGMT-003 (read-only treatment)
Blocks:     MGMT-UI-007
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "MGMT-UI-006",
  "proposed_by": "sveltekit-planner",
  "verification_policy": { "requires_tests": true, "requires_red_evidence": true, "requires_seeded_evidence": true },
  "fixtures": {
    "seeded_principals": { "description": "3 principals seeded into the perm_list but-sdk fixture route the desktop CT harness mounts against.", "seed_method": "ui_flow", "records": ["claude-agent: own contents:write, pull_requests:write", "codex-agent: group-inherited contents:write from group eng", "cursor-bot: own contents:read, pending=true"] },
    "no_principals": { "description": "empty governance config — perm_list returns zero principals.", "seed_method": "ui_flow", "records": ["0 principals configured"] }
  },
  "requirements": [
    { "id": "AC-1", "type": "acceptance_criterion", "primary": true, "description": "GIVEN 3 seeded principals WHEN PrincipalsList renders THEN 3 rows with source-of-grant (codex-agent group:eng, claude-agent own grant)", "verify": "pnpm test:ct:desktop -- PrincipalsListRows", "scenario": { "id": "AC-1", "primary": true, "tier": "visible", "test_tier": "integration", "verification_service": "desktop-ct-harness", "negative_control": { "would_fail_if": ["source-of-grant is absent", "a static shell shows only principal ids with no grant rows", "the component is disconnected from the perm_list fixture"] }, "evidence": { "artifact_type": "screenshot", "required_capture": true }, "cases": [ { "start_ref": "seeded_principals", "action": { "actor": "user", "steps": ["mount PrincipalsList against the seeded perm_list fixture", "observe rendered rows"] }, "end_state": { "must_observe": ["`codex-agent` row shows a `\"group: eng\"` label for contents:write", "`claude-agent` row shows an `\"own grant\"` indicator for pull_requests:write", "`3` principal rows rendered"], "must_not_observe": ["rows without source-of-grant labeling", "fewer than 3 rows", "`(0)` rows"] } } ] } },
    { "id": "AC-2", "type": "acceptance_criterion", "primary": false, "description": "GIVEN cursor-bot pending=true WHEN PrincipalsList renders THEN the cursor-bot row shows a ○ pending Badge", "verify": "pnpm test:ct:desktop -- PrincipalsListRows", "scenario": { "id": "AC-2", "primary": false, "tier": "visible", "test_tier": "integration", "verification_service": "desktop-ct-harness", "negative_control": { "would_fail_if": ["no pending indicator is shown for the pending principal (static)", "the badge is hardcoded committed", "the row is disconnected from the pending state"] }, "evidence": { "artifact_type": "screenshot", "required_capture": true }, "cases": [ { "start_ref": "seeded_principals", "action": { "actor": "user", "steps": ["observe the `cursor-bot` row"] }, "end_state": { "must_observe": ["the `cursor-bot` row shows a pending `○` Badge (style `warning`/`soft`)"], "must_not_observe": ["a committed `●` Badge on the cursor-bot row", "no pending indicator on a pending row"] } } ] } },
    { "id": "AC-3", "type": "acceptance_criterion", "primary": false, "description": "GIVEN seeded principals WHEN user clicks the claude-agent row THEN PrincipalEditor renders inline (no navigation, no modal)", "verify": "pnpm test:ct:desktop -- PrincipalsListEditorToggle", "scenario": { "id": "AC-3", "primary": false, "tier": "visible", "test_tier": "integration", "verification_service": "desktop-ct-harness", "negative_control": { "would_fail_if": ["clicking navigates to a new route instead of an inline editor", "a static list with no click handler", "a modal overlay is used instead of inline"] }, "evidence": { "artifact_type": "screenshot", "required_capture": true }, "cases": [ { "start_ref": "seeded_principals", "action": { "actor": "user", "steps": ["click the `claude-agent` row"] }, "end_state": { "must_observe": ["`PrincipalEditor` is rendered inline with `\"claude-agent\"` visible"], "must_not_observe": ["page navigation to a new URL", "a modal overlay blocking the list", "no editor opening (none)"] } } ] } },
    { "id": "AC-4", "type": "acceptance_criterion", "primary": false, "description": "GIVEN cursor-bot pending=true WHEN observed before commit THEN the ○ indicator persists and the effective grants reflect the committed set", "verify": "pnpm test:ct:desktop -- PrincipalsListPending", "scenario": { "id": "AC-4", "primary": false, "tier": "visible", "test_tier": "integration", "verification_service": "desktop-ct-harness", "negative_control": { "would_fail_if": ["the effective set is optimistically updated before commit (static)", "pending state is hardcoded cleared", "the indicator is disconnected from pending state"] }, "evidence": { "artifact_type": "screenshot", "required_capture": true }, "cases": [ { "start_ref": "seeded_principals", "action": { "actor": "user", "steps": ["observe the `cursor-bot` row without committing"] }, "end_state": { "must_observe": ["the `cursor-bot` `○` pending indicator persists before commit", "the effective grants shown reflect the `contents:read` committed set"], "must_not_observe": ["a committed `●` indicator on a row with pending=true", "the pending indicator showing none before commit"] } } ] } },
    { "id": "AC-5", "type": "acceptance_criterion", "primary": false, "description": "GIVEN perm_list returns [] WHEN PrincipalsList renders THEN EmptyStatePlaceholder with 'No principals configured' + '+ Add first'", "verify": "pnpm test:ct:desktop -- PrincipalsListEmpty", "scenario": { "id": "AC-5", "primary": false, "tier": "visible", "test_tier": "integration", "verification_service": "desktop-ct-harness", "negative_control": { "would_fail_if": ["empty array renders an empty list with no EmptyStatePlaceholder", "the placeholder is static and shows even with principals present", "the list is disconnected from perm_list"] }, "evidence": { "artifact_type": "screenshot", "required_capture": true }, "cases": [ { "start_ref": "no_principals", "action": { "actor": "user", "steps": ["mount PrincipalsList against the empty perm_list fixture", "observe the rendered state"] }, "end_state": { "must_observe": ["an `EmptyStatePlaceholder` with text `\"No principals configured\"`", "a `\"+ Add first\"` action button"], "must_not_observe": ["an empty list container with no placeholder", "any principal row"] } } ] } },
    { "id": "TC-1", "type": "test_criterion", "description": "PrincipalsList with 3 seeded principals renders 3 rows with source-of-grant labels", "verify": "pnpm test:ct:desktop -- PrincipalsListRows", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "a principal with pending=true shows a ○ pending Badge in its row", "verify": "pnpm test:ct:desktop -- PrincipalsListRows", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "clicking a principal row renders PrincipalEditor inline without page navigation", "verify": "pnpm test:ct:desktop -- PrincipalsListEditorToggle", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "perm_list returning [] renders EmptyStatePlaceholder", "verify": "pnpm test:ct:desktop -- PrincipalsListEmpty", "maps_to_ac": "AC-5" },
    { "id": "TC-5", "type": "test_criterion", "description": "pnpm -F @gitbutler/desktop check exits 0 after MGMT-UI-006 lands", "verify": "pnpm -F @gitbutler/desktop check", "maps_to_ac": "AC-1" }
  ]
}
-->
</details>
