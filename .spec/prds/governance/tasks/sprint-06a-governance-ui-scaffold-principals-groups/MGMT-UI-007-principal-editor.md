# MGMT-UI-007: PrincipalEditor (SegmentControl presets + Toggle table + group TagInput; batch-save)

> **Red-Hat Remediation (cycle 1):** Resolved S3+SEC7 (CRITICAL) — AC-6 + TC-7 added for T-MGMT-030 self-escalation denial: toggle reverts + perm.denied danger InfoMessage surfaces; no optimistic apply. Resolved S9 (LOW) — clarifying note added: T-MGMT-044 register-on-first-grant is Rust-owned (MGMT-IPC-001); UI shows newly-registered principals via perm_list re-fetch on commit. Resolved S6 (MEDIUM) — DESIGN-MGMT-001, DESIGN-MGMT-003, DESIGN-MGMT-005 added to depends_on.

## What this does

The inline per-principal editor: a role-preset `SegmentControl`, a functional-permission `Toggle` table (inherited rows read-only/disabled), and a group `TagInput`. All toggle and chip changes stage in **local UI state only**; `[Save changes]` batch-writes the staged set as a sequence of `but perm grant`/`revoke` (+ `group add/remove-member`) SDK calls (B16). A role preset sets own-grant toggles without stripping any inherited grant (union semantics). A self-escalation attempt (`administration:write` granted to self) is surfaced as a backend denial with toggle revert and a `danger` `InfoMessage` — no optimistic apply.

## Why

Sprint 06a · PRD UC-MGMT-02 · criteria T-MGMT-007/008/009/011/030 · capability CAP-AUTHZ-01. The editor that turns "view a principal's permissions" into "change them" — honestly batched, never per-toggle, never optimistically self-escalating.

**Note on T-MGMT-044 (register-on-first-grant):** The registration of a new principal is Rust-owned via `MGMT-IPC-001` (`but_api` register-on-first-grant path). The PrincipalEditor UI does not perform registration directly. Newly-registered principals appear in the Principals list through the `perm_list` re-fetch that follows a successful commit (MGMT-UI-006).

## How to verify

PRIMARY **AC-1** — `pnpm test:ct:desktop -- PrincipalEditorInheritedReadOnly`: mounted for codex-agent (inherited contents:write from eng), the `contents:write` Toggle has `disabled=true` and the SOURCE column shows `"group: eng"`. Full gate set in the spec below.

## Scope

- `apps/desktop/src/components/governance/PrincipalEditor.svelte` (NEW)
- `apps/desktop/tests/governance/PrincipalEditor.spec.ts` (NEW — CT specs)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: MGMT-UI-007 — PrincipalEditor (SegmentControl presets + Toggle table + group TagInput; batch-save)
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P1
EFFORT:     M  (90 min)
AGENT:      implementer=sveltekit-implementer | reviewer=sveltekit-reviewer
PROPOSED-BY: sveltekit-planner
SPRINT:     ./SPRINT.md
PRD_REFS:   UC-MGMT-02, T-MGMT-007, T-MGMT-008, T-MGMT-009, T-MGMT-011, T-MGMT-030
CAPABILITIES: CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  pnpm test:ct:desktop -- PrincipalEditorInheritedReadOnly
  check: pnpm -F @gitbutler/desktop check   |   lint: pnpm lint

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
PrincipalEditor for codex-agent (own pull_requests:write; inherited contents:write from eng) renders:
contents:write row disabled with "group: eng"; pull_requests:write toggle ON enabled. Selecting "write"
preset enables the write-desugared own-grant toggles WITHOUT touching the inherited row. Clicking
"Save changes" issues the minimal perm_grant/perm_revoke sequence for changed rows only — NOT one call
per toggle interaction, and not a single overwrite verb. Attempting to grant administration:write to
self (perm.denied response): the toggle reverts to OFF and a danger InfoMessage surfaces the denial —
no optimistic apply.

NOTE on T-MGMT-044: register-on-first-grant is fully Rust-owned (MGMT-IPC-001). The UI re-fetches
perm_list after commit to surface newly-registered principals — no registration logic lives here.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] Individual Toggle changes update LOCAL UI state only — NO per-toggle SDK call (B16).
- [MUST] "Save changes" batch-writes the staged set as a sequence of perm_grant/perm_revoke (+ group_add_member/
  group_remove_member for chip changes) via the but-sdk — the minimal diff, not per-interaction.
- [MUST] Inherited-permission rows are disabled Toggles with an "inherited"/"group: X" source label — not editable.
- [MUST] A role preset sets own-grant toggles to the desugared set WITHOUT removing any inherited grant (union preserved).
- [MUST] A self-escalation attempt (granting administration:write to self, fixture: denied_self_escalation): the
  Save receives perm.denied from the SDK -> the toggle REVERTS to its pre-save state (no optimistic apply) AND
  a danger-variant InfoMessage surfacing 'perm.denied' appears.
- [NEVER] NEVER call perm_grant per toggle interaction; NEVER use a single overwrite verb (but perm set — deferred C2);
  NEVER make inherited rows interactive; NEVER optimistically apply self-escalation; NEVER write .gitbutler/*.toml directly from the renderer.
- [STRICTLY] No relative imports — @gitbutler/ references. No console.log. Local editing state via $state — no module-level store.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: inherited rows render as disabled toggles with a source label
- [ ] AC-2: individual toggle changes update local UI state only (no per-toggle SDK call)
- [ ] AC-3: Save changes batch-issues the minimal perm_grant/perm_revoke sequence
- [ ] AC-4: role preset sets own-grant toggles without removing inherited grants
- [ ] AC-5: group TagInput add/remove staged locally; group change included in Save batch
- [ ] AC-6: self-escalation perm.denied -> toggle reverts + danger InfoMessage; no optimistic apply
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: inherited rows render as disabled toggles with source label
  GIVEN: PrincipalEditor mounted for codex-agent (inherited contents:write from eng)
  WHEN:  the component renders
  THEN:  the contents:write Toggle has disabled=true; the SOURCE column shows "group: eng"; the toggle is not clickable
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- PrincipalEditorInheritedReadOnly

AC-2: individual toggle changes update local UI state only — no per-toggle SDK call
  GIVEN: codex-agent editor
  WHEN:  user clicks the reviews:write Toggle (own-grant, OFF -> ON)
  THEN:  the Toggle flips ON visually; no perm_grant SDK call is issued; "Save changes" becomes actionable
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- PrincipalEditorLocalState

AC-3: Save changes batch-issues perm_grant/perm_revoke SDK sequence — not per-toggle
  GIVEN: codex-agent editor; user toggled reviews:write ON
  WHEN:  user clicks "Save changes"
  THEN:  exactly one perm_grant("reviews:write") SDK call for the new ON diff; zero calls for unchanged rows
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- PrincipalEditorBatchSave

AC-4: role preset SegmentControl sets own-grant toggles without removing inherited grants
  GIVEN: codex-agent editor (contents:write inherited from eng)
  WHEN:  user selects the "write" preset
  THEN:  own-grant write-desugared toggles flip ON; the inherited contents:write row stays disabled/unchanged; no SDK call yet
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- PrincipalEditorPreset

AC-5: group TagInput adds/removes group chips locally; group changes included in Save batch
  GIVEN: codex-agent editor (groups: [eng])
  WHEN:  user adds a "platform" chip and clicks "Save changes"
  THEN:  the chip appears in the TagInput; on Save, group_add_member("platform","codex-agent") is in the batch
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- PrincipalEditorGroupChip

AC-6: self-escalation perm.denied -> toggle reverts to OFF + danger InfoMessage; no optimistic apply (T-MGMT-030)
  GIVEN: PrincipalEditor mounted for the current user (self); administration:write Toggle is OFF
  WHEN:  user toggles administration:write ON and clicks "Save changes" (SDK returns perm.denied via denied_self_escalation fixture)
  THEN:  the administration:write Toggle reverts to OFF (no optimistic apply);
         a danger-variant InfoMessage surfacing 'perm.denied' appears in the editor
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- PrincipalEditorSelfEscalation

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): the inherited contents:write Toggle has disabled=true and shows a source-of-group label
    VERIFY: pnpm test:ct:desktop -- PrincipalEditorInheritedReadOnly
- TC-2 (-> AC-2): clicking a Toggle does not issue any perm_grant or perm_revoke SDK call
    VERIFY: pnpm test:ct:desktop -- PrincipalEditorLocalState
- TC-3 (-> AC-3): clicking Save issues the minimal perm_grant/perm_revoke sequence for only the changed rows
    VERIFY: pnpm test:ct:desktop -- PrincipalEditorBatchSave
- TC-4 (-> AC-4): selecting a role preset sets own-grant toggles without modifying inherited rows
    VERIFY: pnpm test:ct:desktop -- PrincipalEditorPreset
- TC-5 (-> AC-5): adding a group chip and clicking Save issues group_add_member in the batch
    VERIFY: pnpm test:ct:desktop -- PrincipalEditorGroupChip
- TC-6 (-> AC-1): pnpm -F @gitbutler/desktop check exits 0 after MGMT-UI-007 lands
    VERIFY: pnpm -F @gitbutler/desktop check
- TC-7 (-> AC-6): perm.denied on Save -> administration:write Toggle is OFF + danger InfoMessage present
    VERIFY: pnpm test:ct:desktop -- PrincipalEditorSelfEscalation

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides: PrincipalEditor.svelte — inline per-principal permission editor with batch-save
consumes: MGMT-IPC-004 (perm_grant/perm_revoke/group_add_member/group_remove_member SDK); MGMT-UI-006 (renders inline);
          packages/ui SegmentControl/Toggle/TagInput/Button/Badge/InfoMessage
boundary_contracts:
  - toggles stage local state only; Save batches the minimal grant/revoke (+ group) diff; inherited rows read-only;
    preset preserves union; self-escalation surfaces the backend denial (not optimistic); no register-on-first-grant
    logic here (Rust-owned via MGMT-IPC-001; UI re-fetches perm_list post-commit)

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - apps/desktop/src/components/governance/PrincipalEditor.svelte (NEW)
  - apps/desktop/tests/governance/PrincipalEditor.spec.ts (NEW)
writeProhibited:
  - packages/ui components (reuse as-is, no modifications)
  - apps/desktop/src/components/governance/PrincipalsList.svelte (owned by MGMT-UI-006)
  - packages/but-sdk/src/generated (owned by MGMT-IPC-004)
  - any direct .gitbutler/*.toml write from the renderer; any file not in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. packages/ui/src/lib/components/segmentControl/SegmentControl.svelte — role preset strip (selected, onselect)
2. packages/ui/src/lib/components/Toggle.svelte — disabled prop for inherited rows
3. packages/ui/src/lib/components/TagInput.svelte — group chip add/remove, readonly prop
4. packages/ui/src/lib/components/Button.svelte — Save changes + Cancel
5. packages/ui/src/lib/components/Badge.svelte — inherited source badge (gray/soft)
6. packages/ui/src/lib/components/InfoMessage.svelte — danger variant for perm.denied denial (AC-6)
7. .spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md — per-principal editor wireframe (ROLE PRESET / FUNCTIONAL PERMISSIONS / GROUPS)
8. .spec/prds/governance/08-uc-mgmt.md UC-MGMT-02 — batch-save model B16, union semantics, register-on-first-grant (Rust-owned), T-MGMT-030 self-escalation denial
9. apps/desktop/src/components/rules/RuleEditor.svelte — inline slide-in panel structure to mirror

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- pnpm test:ct:desktop -- PrincipalEditorInheritedReadOnly  -> Exit 0
- pnpm test:ct:desktop -- PrincipalEditorLocalState         -> Exit 0
- pnpm test:ct:desktop -- PrincipalEditorBatchSave          -> Exit 0
- pnpm test:ct:desktop -- PrincipalEditorPreset             -> Exit 0
- pnpm test:ct:desktop -- PrincipalEditorGroupChip          -> Exit 0
- pnpm test:ct:desktop -- PrincipalEditorSelfEscalation     -> Exit 0
- pnpm -F @gitbutler/desktop check                          -> Exit 0
- pnpm lint                                                 -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references: DESIGN-MGMT-005 (inherited-vs-own row distinction); DESIGN-MGMT-003 (read-only treatment); DESIGN-MGMT-001 (four-tab IA, Principals tab context)
notes (from DESIGN-MGMT-005 enrichment): three row types — inherited-only (Toggle disabled, Badge gray/soft
  "[group: eng]", "── inherited ──" in var(--text-3)), own-grant (Toggle enabled, "own grant" in var(--text-2),
  pending Badge warning/soft when staged), both (show as inherited/disabled). SegmentControl is a sugar layer:
  selecting a preset sets the own-grant toggles' checked state in LOCAL state only; it never touches inherited
  rows. [Save changes] batches grant/revoke; [Cancel] resets local state. Read-only mode: SegmentControl
  non-interactive, all Toggle disabled, TagInput readonly, Save/Cancel disabled.
  SELF-ESCALATION (T-MGMT-030): when Save returns perm.denied, the toggle reverts to its pre-staged value
  (no optimistic apply) and a danger InfoMessage appears. The InfoMessage text surfaces the SDK error code
  'perm.denied'. Do NOT apply the toggle change to local state on denial.
pattern: two-column (SOURCE | GRANT) table; {#each permissions as perm} distinguishing perm.source group vs own;
  SegmentControl above drives own-grant checked states via local state only
pattern_source: apps/desktop/src/components/rules/RuleEditor.svelte (inline panel + local state + save); Toggle.svelte disabled; Badge.svelte
anti_pattern: per-toggle SDK write; single overwrite perm_set; interactive inherited rows; pending Badge on inherited rows; modal;
  optimistic apply of self-escalation; not reverting toggle on perm.denied

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: sveltekit-implementer
reviewer: sveltekit-reviewer
coding_standards: apps/desktop/AGENTS.md, frontend.md

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: MGMT-UI-006 (renders the editor inline per row), MGMT-IPC-004 (perm/group SDK);
            DESIGN-MGMT-001 (four-tab IA), DESIGN-MGMT-003 (read-only treatment), DESIGN-MGMT-005 (inherited-vs-own row)
Blocks:     none
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "MGMT-UI-007",
  "proposed_by": "sveltekit-planner",
  "verification_policy": { "requires_tests": true, "requires_red_evidence": true, "requires_seeded_evidence": true },
  "fixtures": {
    "seeded_editor_codex": { "description": "PrincipalEditor mounted (via the desktop CT harness) for codex-agent: own grant pull_requests:write; group-inherited contents:write from group eng; member of [eng]. Seeded through the perm_list/governance_status fixture route.", "seed_method": "ui_flow", "records": ["codex-agent own = pull_requests:write", "group-inherited contents:write from eng", "groups = [eng]"] },
    "seeded_editor_no_grants": { "description": "PrincipalEditor mounted for new-agent with no own grants, no group grants, no groups.", "seed_method": "ui_flow", "records": ["new-agent: 0 grants, 0 groups"] },
    "denied_self_escalation": { "description": "PrincipalEditor mounted for the current user (self). SDK perm_grant for administration:write returns perm.denied. Used to verify toggle revert and danger InfoMessage.", "seed_method": "ui_flow", "records": ["principal = current-user (self)", "administration:write Toggle initially OFF", "SDK perm_grant(administration:write) returns perm.denied"] }
  },
  "requirements": [
    { "id": "AC-1", "type": "acceptance_criterion", "primary": true, "description": "GIVEN codex-agent editor (inherited contents:write from eng) WHEN it renders THEN the contents:write Toggle is disabled=true with SOURCE 'group: eng'", "verify": "pnpm test:ct:desktop -- PrincipalEditorInheritedReadOnly", "scenario": { "id": "AC-1", "primary": true, "tier": "visible", "test_tier": "integration", "verification_service": "desktop-ct-harness", "negative_control": { "would_fail_if": ["the inherited toggle is enabled and interactive", "the source label is absent (static)", "the editor is disconnected from the principal fixture"] }, "evidence": { "artifact_type": "screenshot", "required_capture": true }, "cases": [ { "start_ref": "seeded_editor_codex", "action": { "actor": "user", "steps": ["observe the `contents:write` row"] }, "end_state": { "must_observe": ["the `contents:write` Toggle has `disabled=true`", "the SOURCE column shows `\"group: eng\"`"], "must_not_observe": ["the contents:write Toggle interactive/clickable", "no source label (none) on the inherited row"] } } ] } },
    { "id": "AC-2", "type": "acceptance_criterion", "primary": false, "description": "GIVEN codex-agent editor WHEN user clicks reviews:write Toggle ON THEN it flips visually, 0 perm_grant SDK calls fire, Save becomes actionable", "verify": "pnpm test:ct:desktop -- PrincipalEditorLocalState", "scenario": { "id": "AC-2", "primary": false, "tier": "visible", "test_tier": "integration", "verification_service": "desktop-ct-harness", "negative_control": { "would_fail_if": ["a perm_grant SDK call is issued on toggle click instead of on Save", "the toggle is a no-op static control", "the toggle is disconnected from local state"] }, "evidence": { "artifact_type": "stdout", "required_capture": true }, "cases": [ { "start_ref": "seeded_editor_codex", "action": { "actor": "user", "steps": ["click the `reviews:write` Toggle (own-grant, currently OFF) to ON"] }, "end_state": { "must_observe": ["the `reviews:write` Toggle flips to ON visually", "`0` perm_grant SDK calls are issued on the click", "the `\"Save changes\"` button becomes actionable"], "must_not_observe": ["a perm_grant SDK call issued immediately on toggle click", "no visible state change on the toggle (none)"] } } ] } },
    { "id": "AC-3", "type": "acceptance_criterion", "primary": false, "description": "GIVEN reviews:write toggled ON WHEN user clicks Save THEN exactly 1 perm_grant('reviews:write') SDK call and 0 for unchanged rows", "verify": "pnpm test:ct:desktop -- PrincipalEditorBatchSave", "scenario": { "id": "AC-3", "primary": false, "tier": "visible", "test_tier": "integration", "verification_service": "desktop-ct-harness", "negative_control": { "would_fail_if": ["perm_grant/perm_revoke called once per toggle interaction rather than once for the diff", "a stub Save that writes nothing (no-op)", "a single overwrite perm_set call replaces the grant set"] }, "evidence": { "artifact_type": "stdout", "required_capture": true }, "cases": [ { "start_ref": "seeded_editor_codex", "action": { "actor": "user", "steps": ["click `reviews:write` Toggle ON", "click `\"Save changes\"`"] }, "end_state": { "must_observe": ["clicking Save issues `1` `perm_grant(\"reviews:write\")` SDK call", "`0` calls for unchanged rows"], "must_not_observe": ["multiple SDK calls per toggle interaction", "a single overwrite `perm_set` call", "no SDK call on Save (none)"] } } ] } },
    { "id": "AC-4", "type": "acceptance_criterion", "primary": false, "description": "GIVEN codex-agent editor WHEN user selects the 'write' preset THEN own-grant write toggles flip ON and the inherited contents:write row stays disabled/unchanged, no SDK call yet", "verify": "pnpm test:ct:desktop -- PrincipalEditorPreset", "scenario": { "id": "AC-4", "primary": false, "tier": "visible", "test_tier": "integration", "verification_service": "desktop-ct-harness", "negative_control": { "would_fail_if": ["the preset strips the inherited contents:write grant", "the preset issues an immediate SDK call (static)", "the preset is a no-op"] }, "evidence": { "artifact_type": "screenshot", "required_capture": true }, "cases": [ { "start_ref": "seeded_editor_codex", "action": { "actor": "user", "steps": ["select the `\"write\"` preset in the SegmentControl"] }, "end_state": { "must_observe": ["the own-grant Toggles for the write-desugared set (`contents:read`, `contents:write`, `pull_requests:write`, `reviews:write`) flip ON", "the inherited `contents:write` row stays `disabled` and unchanged"], "must_not_observe": ["the inherited row changed or removed", "an SDK call issued before Save (none)"] } } ] } },
    { "id": "AC-5", "type": "acceptance_criterion", "primary": false, "description": "GIVEN codex-agent editor (groups [eng]) WHEN user adds a 'platform' chip and clicks Save THEN the chip appears and group_add_member('platform','codex-agent') is in the batch", "verify": "pnpm test:ct:desktop -- PrincipalEditorGroupChip", "scenario": { "id": "AC-5", "primary": false, "tier": "visible", "test_tier": "integration", "verification_service": "desktop-ct-harness", "negative_control": { "would_fail_if": ["group_add_member is called immediately on chip add instead of on Save", "a static chip with no SDK wiring", "the chip does not appear before Save"] }, "evidence": { "artifact_type": "stdout", "required_capture": true }, "cases": [ { "start_ref": "seeded_editor_codex", "action": { "actor": "user", "steps": ["type `\"platform\"` in the group TagInput", "press Enter to add the chip", "click `\"Save changes\"`"] }, "end_state": { "must_observe": ["a `\"platform\"` chip appears in the TagInput after add", "on Save, `group_add_member(\"platform\", \"codex-agent\")` is called in the batch"], "must_not_observe": ["group_add_member called before the Save click", "the chip not appearing before Save (none)"] } } ] } },
    { "id": "AC-6", "type": "acceptance_criterion", "primary": false, "description": "GIVEN PrincipalEditor for current user (self), administration:write OFF WHEN user toggles ON and Save returns perm.denied THEN the toggle REVERTS to OFF (no optimistic apply) AND a danger InfoMessage surfacing 'perm.denied' appears (T-MGMT-030)", "verify": "pnpm test:ct:desktop -- PrincipalEditorSelfEscalation", "scenario": { "id": "AC-6", "primary": false, "tier": "visible", "test_tier": "integration", "verification_service": "desktop-ct-harness", "negative_control": { "would_fail_if": ["the toggle stays ON after perm.denied (optimistic apply)", "no danger InfoMessage appears (denial is silent)", "the denial is not surfaced and Save succeeds (stub)"] }, "evidence": { "artifact_type": "screenshot", "required_capture": true }, "cases": [ { "start_ref": "denied_self_escalation", "action": { "actor": "user", "steps": ["toggle `administration:write` ON (was OFF)", "click `\"Save changes\"`"] }, "end_state": { "must_observe": ["the `administration:write` Toggle is back to OFF after the perm.denied response", "a `danger`-variant InfoMessage containing `\"perm.denied\"` is visible"], "must_not_observe": ["the `administration:write` Toggle remaining ON (optimistic apply)", "no InfoMessage after denial (none)"] } } ] } },
    { "id": "TC-1", "type": "test_criterion", "description": "the inherited contents:write Toggle has disabled=true and shows a source-of-group label", "verify": "pnpm test:ct:desktop -- PrincipalEditorInheritedReadOnly", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "clicking a Toggle does not issue any perm_grant or perm_revoke SDK call", "verify": "pnpm test:ct:desktop -- PrincipalEditorLocalState", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "clicking Save issues the minimal perm_grant/perm_revoke sequence for only the changed rows", "verify": "pnpm test:ct:desktop -- PrincipalEditorBatchSave", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "selecting a role preset sets own-grant toggles without modifying inherited rows", "verify": "pnpm test:ct:desktop -- PrincipalEditorPreset", "maps_to_ac": "AC-4" },
    { "id": "TC-5", "type": "test_criterion", "description": "adding a group chip and clicking Save issues group_add_member in the batch", "verify": "pnpm test:ct:desktop -- PrincipalEditorGroupChip", "maps_to_ac": "AC-5" },
    { "id": "TC-6", "type": "test_criterion", "description": "pnpm -F @gitbutler/desktop check exits 0 after MGMT-UI-007 lands", "verify": "pnpm -F @gitbutler/desktop check", "maps_to_ac": "AC-1" },
    { "id": "TC-7", "type": "test_criterion", "description": "perm.denied on Save -> administration:write Toggle reverts to OFF + danger InfoMessage present; no optimistic apply", "verify": "pnpm test:ct:desktop -- PrincipalEditorSelfEscalation", "maps_to_ac": "AC-6" }
  ]
}
-->
</details>
