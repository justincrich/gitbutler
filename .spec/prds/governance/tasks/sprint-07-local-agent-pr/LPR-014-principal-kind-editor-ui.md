# LPR-014: Principal `kind` editor UI in the Governance Principals tab

## What this does

Extend `PrincipalsList.svelte` and `PrincipalEditor.svelte` in the Governance Principals tab to display and
edit a principal's `kind` field (`"agent"` | `"human"` | absent). Reading is additive: when the SDK returns a
principal object that carries `kind`, the row badge renders accordingly. Editing persists through the governance
IPC/SDK write that carries `kind` alongside the existing grants (depends on LPR-013, the producer that wires
the `kind` field into the `permissions.toml` write path). The 6a pending-store pattern applies: the row gets a
pending indicator after the write, and the governance commit banner count increments. An absent `kind`
defaults to human (the conservative posture from LPR-005 AC-4). This task NEVER reads `kind` in any gate path
and NEVER surfaces `kind` as an authorization control.

## Why

Sprint 07 · PRD UC-LPR-04 · capability CAP-AUTHZ-01. LPR-005 added the additive `kind: Option<String>` field
to `PrincipalWire` in `but-authz`'s committed config and exposed a typed reader for the tag-derivation +
desktop UI. LPR-013 wires the `kind` field into the `permissions.toml` write path (the SDK mutation). This task
surfaces both: reads the declared `kind` from the SDK response (already available from the governance read path
added in LPR-005) and writes it via the LPR-013 SDK call. An orchestrator that sets `kind = "agent"` on a
principal enables the agent-authored tag derivation in `review_status` (LPR-005). The UI representation is
informational only.

## How to verify

PRIMARY **AC-1** — `pnpm test:ct:desktop -- PrincipalsListAgentBadge`: a principal with `kind='agent'` in the
SDK response renders the agent badge in its row. Full gate set in the spec below.

## Scope

- `apps/desktop/src/components/governance/PrincipalsList.svelte` (MODIFY — add `kind` to `PrincipalsListEntry`
  type; render an agent badge in the row when `kind='agent'`)
- `apps/desktop/src/components/governance/PrincipalEditor.svelte` (MODIFY — add a `kind` selector in the
  editor form; persist via the LPR-013 SDK write; show pending on the row after write)
- `apps/desktop/tests/governance/PrincipalKindEditor.spec.ts` (NEW — CT specs)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: LPR-014 — Principal kind editor UI (agent badge + kind write in Governance Principals tab)
================================================================================

TASK_TYPE:   FEATURE
STATUS:      Backlog
PRIORITY:    P1
EFFORT:      M  (75 min)
AGENT:       sveltekit-implementer
PROPOSED-BY: sveltekit-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-LPR-04
CAPABILITIES:CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  pnpm test:ct:desktop -- PrincipalsListAgentBadge
  check: pnpm -F @gitbutler/desktop check
  lint:  pnpm lint

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Mounting PrincipalsList with a principal whose SDK entry declares kind='agent'
renders a Badge labelled 'agent' (or equivalent visual marker per DESIGN-LPR-002)
in that principal's row. Opening the PrincipalEditor for that principal shows the
kind selector with 'agent' selected. Changing kind to 'human' calls the LPR-013
SDK write with kind='human'; the row gains a pending indicator (○ Badge) and the
governance pending-banner count increments. A principal without a kind field shows
no badge and defaults to human in the selector. pnpm test:ct:desktop --
PrincipalKindEditor passes. pnpm -F @gitbutler/desktop check and pnpm lint pass.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] kind is additive on PrincipalsListEntry — the existing type gains
  kind?: 'agent' | 'human' | undefined as a strictly optional field. Existing
  callers that do not pass kind must continue to work without change (backward
  compatible). AC-3 proves the no-kind case.
- [MUST] The agent badge renders in the PrincipalsListEntry row ONLY when
  kind='agent' (not when kind='human' or when kind is absent); a missing kind
  defaults to human presentation (no badge).
- [MUST] The kind selector in PrincipalEditor offers exactly two options: 'agent'
  and 'human'. Selecting one calls the LPR-013 SDK write with the chosen kind.
  The write goes through the governance IPC path (administration:write-gated —
  the same path as permGrant/permRevoke in PrincipalEditorService), not the
  project-settings path (which is for keep_reviews_local, a different class).
- [MUST] After the kind write, the row gains a pending indicator matching the
  existing ○ Badge pending pattern (PrincipalsList.svelte:178-187 — the
  pending=true + ownGrants.length > 0 badge pattern). The governance pending-
  banner count increments via the existing pendingStore.
- [MUST] kind NEVER enters any gate path and NEVER appears as an authority. The
  kind selector is informational-descriptor UI only — it gates nothing. AC-2
  (the display-only assertion) proves the badge is decorative, not functional.
- [NEVER] NEVER display the kind badge in a way that implies it changes merge
  decisions or authorization outcomes.
- [NEVER] NEVER add kind to the enforcement map (GovConfig.principals) or route
  it through a gate predicate — this task is UI-only; the enforcement-neutral
  constraint from LPR-005 AC-7 governs the backend.
- [NEVER] NEVER add +page.server.ts or +layout.server.ts.
- [NEVER] NEVER use module-level state.
- [STRICTLY] No relative imports — @gitbutler/ package references. No console.log.
  Prettier: tabs, double quotes, no trailing commas, 100-col.
- [STRICTLY] Svelte 5 $props()/$state()/$derived() rune syntax throughout.
- [STRICTLY] CT describe blocks MUST use the component name as the outermost
  describe string so `pnpm test:ct:desktop -- <ComponentName>` grep matches.
- [STRICTLY] The pending indicator after kind write must use the SAME Badge
  pattern as the existing principals-list-pending-* badge (PrincipalsList.svelte
  lines 178-187 — kind='soft', style='warning', size='icon', text='○').

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: a principal with kind='agent' shows the agent badge
- [ ] AC-2: the agent badge is display-only; kind gates nothing
- [ ] AC-3: a principal without kind shows no badge; defaults to human in editor
- [ ] AC-4: changing kind in PrincipalEditor calls the LPR-013 SDK write and
      shows a pending indicator on the row
- [ ] AC-5: isReadOnly=true disables the kind selector and fires 0 SDK calls
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: a principal with kind='agent' shows the agent badge in its row
  GIVEN: PrincipalsList mounted with seeded_agent_principal (a principal entry
         with kind='agent' returned by the SDK)
  WHEN:  the component renders
  THEN:  the principal's row contains a Badge with accessible name or text
         containing 'agent'; the badge uses the governance badge style (Badge
         component, gray/soft kind per DESIGN-LPR-002); a principal with
         kind='human' in the same list has no agent badge
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- PrincipalsListAgentBadge

AC-2: the agent badge is display-only; kind gates nothing
  GIVEN: PrincipalsList mounted with seeded_agent_principal; isReadOnly=true
  WHEN:  user views the list
  THEN:  the agent badge renders but carries no interactive affordance (no
         click, no aria-pressed, no Button role); 0 SDK write calls fire on
         badge interaction; the row's expand/collapse behavior is unchanged
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- PrincipalsListAgentBadgeDisplayOnly

AC-3: a principal without kind shows no badge; defaults to human in editor
  GIVEN: PrincipalsList mounted with seeded_no_kind_principal (a principal entry
         where kind is absent / undefined in the SDK response)
  WHEN:  the component renders; then user opens the PrincipalEditor for that
         principal
  THEN:  no agent badge renders in the row; in the PrincipalEditor the kind
         selector defaults to 'human' (the conservative default posture per
         LPR-005 AC-4)
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- PrincipalsListNoKindDefaultsHuman

AC-4: changing kind in PrincipalEditor calls the LPR-013 SDK write and shows pending
  SCOPE NOTE: AC-4 verifies the COMPONENT-LEVEL call contract only — that the component
  calls the LPR-013 SDK binding with the correct path and payload (kind='agent') and
  increments the pending indicator. The seeded_kind_write fixture is a JS spy that resolves
  {ok:true} at component-test scope. This verifies the component is wired correctly (call
  count, args, pending-badge), NOT that the write persists through the REAL Tauri bus or
  real but-db. End-to-end verification against the REAL Tauri bus and real but-db lives in
  LPR-013's AC-5 (the producer task's integration test). This scoping is intentional:
  component-test scope is appropriate here because the LPR-013 SDK binding is the
  integration seam, and LPR-013 owns the backend proof.
  GIVEN: PrincipalsList mounted with seeded_no_kind_principal; the principal's
         PrincipalEditor is open; seeded_kind_write configured (LPR-013 SDK
         write spy resolves successfully at component-test scope)
  WHEN:  user changes the kind selector from 'human' (default) to 'agent' and
         confirms (or the selector commits on change per DESIGN-LPR-002)
  THEN:  the LPR-013 SDK write spy is called == 1 time with kind='agent' for
         that principal (correct path + payload verified at component-test scope);
         the principal's row gains a pending indicator (○ Badge style='warning'
         kind='soft' size='icon' per the existing pending pattern at
         PrincipalsList.svelte:178-187); the governance pending-banner count
         increments by 1; NOTE: full backend persistence is proven by LPR-013 AC-5
         (cross-reference: the real Tauri bus + real but-db proof lives there)
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness (component scope)
  CRITICAL CONSTRAINT: this test verifies the component call contract and pending-UI
  wiring, NOT end-to-end backend persistence. A static shell that ignores the SDK
  binding and hard-renders a pending badge WILL FAIL this test (the spy call count == 1
  assertion catches it). The seamed spy is legitimate at component-test scope because
  LPR-013 AC-5 provides the mandatory integration bond.
  VERIFY: pnpm test:ct:desktop -- PrincipalKindWriteAndPending

AC-5: isReadOnly=true disables the kind selector in PrincipalEditor; 0 SDK calls
  GIVEN: PrincipalsList mounted with seeded_agent_principal; isReadOnly=true;
         PrincipalEditor opened for the agent principal
  WHEN:  user attempts to interact with the kind selector
  THEN:  the kind selector has disabled or aria-disabled attribute; 0 LPR-013
         SDK write calls fire on any interaction
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- PrincipalKindEditorReadOnly

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): seeded_agent_principal → agent badge present in that principal's
    row; kind='human' principal has no badge
    VERIFY: pnpm test:ct:desktop -- PrincipalsListAgentBadge
- TC-2 (-> AC-2): agent badge has no interactive affordance; 0 SDK write calls
    on badge click; isReadOnly=true changes nothing about badge presence
    VERIFY: pnpm test:ct:desktop -- PrincipalsListAgentBadgeDisplayOnly
- TC-3 (-> AC-3): absent kind → no badge; PrincipalEditor kind selector defaults
    to 'human'
    VERIFY: pnpm test:ct:desktop -- PrincipalsListNoKindDefaultsHuman
- TC-4 (-> AC-4): kind selector change → LPR-013 SDK write spy called == 1 with
    kind='agent' (component call contract, NOT backend persistence); principal row
    gains ○ Badge pending; pending-banner count +1; end-to-end persistence is LPR-013 AC-5
    VERIFY: pnpm test:ct:desktop -- PrincipalKindWriteAndPending
- TC-5 (-> AC-5): isReadOnly=true → kind selector disabled; 0 SDK write calls
    VERIFY: pnpm test:ct:desktop -- PrincipalKindEditorReadOnly

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides:
  - PrincipalsListEntry.kind?: 'agent'|'human'|undefined — additive optional
    field; when 'agent' renders a Badge in the row (informational only)
  - A kind selector in PrincipalEditor (two options: 'agent' / 'human') that
    writes via the LPR-013 SDK call; shows pending on the row after write
consumes:
  - LPR-005 (the additive kind: Option<String> on PrincipalWire; the typed
    reader that surfaces kind to the desktop UI — this is the read binding)
  - LPR-013 (the SDK write that persists kind to permissions.toml via the
    governance IPC path — MUST land before this task can write kind)
  - apps/desktop/src/components/governance/PrincipalsList.svelte (MODIFY —
    kind badge in row; pending pattern at lines 178-187)
  - apps/desktop/src/components/governance/PrincipalEditor.svelte (MODIFY —
    kind selector in editor form)
  - packages/ui: Badge, SegmentControl or Select (for the kind selector)
  - The existing governance pendingStore (the 6a pattern — increment count
    after kind write; same pendingStore as GroupsList / PrincipalEditor)
  - DESIGN-LPR-002 (the UI design contract for the agent badge and kind
    selector appearance in the Principals tab)
boundary_contracts:
  - kind is additive and backward-compatible: PrincipalsListEntry without kind
    continues to render and function exactly as today (AC-3).
  - The kind selector write goes through the governance IPC / administration:write-
    gated path (the same route as permGrant/permRevoke in PrincipalEditorService)
    — NOT the project-settings path (that is for keep_reviews_local, LPR-012).
  - The agent badge is purely informational: it gates nothing, has no aria-pressed,
    and fires no SDK write on click. kind NEVER enters a gate predicate in the UI.
  - Pending indicator after kind write uses the EXACT same Badge pattern as the
    existing principals-list-pending-* badge (PrincipalsList.svelte:178-187).

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - apps/desktop/src/components/governance/PrincipalsList.svelte (MODIFY —
    add kind?: field to PrincipalsListEntry; add agent badge to row when kind='agent')
  - apps/desktop/src/components/governance/PrincipalEditor.svelte (MODIFY —
    add kind selector; wire to LPR-013 SDK write; show pending on row after write)
  - apps/desktop/tests/governance/PrincipalKindEditor.spec.ts (NEW — CT specs)
writeProhibited:
  - apps/desktop/src/components/governance/GovernanceSettings.svelte — consume-only
  - apps/desktop/src/components/governance/GroupsList.svelte — unchanged by this task
  - apps/desktop/src/lib/governance/pendingStore.svelte.ts — consume-only
    (the existing pendingStore; no changes to its logic)
  - packages/but-sdk/src/generated — SDK regen is LPR-010
  - Any +page.server.ts or +layout.server.ts
  - Any file not in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. apps/desktop/src/components/governance/PrincipalsList.svelte [144-235] —
   PRIMARY PATTERN — the principal row structure, the pending Badge at lines
   178-187 (Badge style='warning' kind='soft' size='icon' text='○'), and the
   PrincipalsListEntry type (lines 8-16); add kind?: field here and render the
   agent Badge beside or after the principalId strong tag (line 189).
2. apps/desktop/src/components/governance/PrincipalEditor.svelte [44-80] —
   PRIMARY PATTERN — the existing editor form shape (SegmentControl / Badge /
   Button / InfoMessage imports; how permGrant/permRevoke calls are made);
   the kind selector mirrors this: a two-option SegmentControl ('agent' /
   'human') that calls the LPR-013 SDK write on change.
3. apps/desktop/src/components/governance/GovernanceSettings.svelte [89-95] —
   the pendingStore wiring (pendingCount, isReadOnly, markGroupPending pattern);
   understand how the pending Banner count increments after a write so kind
   write follows the same lifecycle.
4. .spec/prds/governance/tasks/sprint-07-local-agent-pr/LPR-005-derived-pr-lifecycle-agent-tag.md
   §CRITICAL CONSTRAINTS — the additive kind descriptor: enforcement-neutral,
   not in GovConfig.principals, not read by any gate; the desktop UI is the
   ONLY consumer for the display path.
5. DESIGN-LPR-002 (in sprint-07 folder — the UI design contract for the agent
   badge styling and kind selector appearance in the Principals tab).

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- pnpm test:ct:desktop -- PrincipalsListAgentBadge   -> Exit 0
- pnpm test:ct:desktop -- PrincipalsListAgentBadgeDisplayOnly   -> Exit 0
- pnpm test:ct:desktop -- PrincipalsListNoKindDefaultsHuman   -> Exit 0
- pnpm test:ct:desktop -- PrincipalKindWriteAndPending   -> Exit 0
- pnpm test:ct:desktop -- PrincipalKindEditorReadOnly   -> Exit 0
- pnpm -F @gitbutler/desktop check   -> Exit 0
- pnpm lint   -> Exit 0
- grep -rn 'GovConfig\.principals\|gate.*kind\|enforce.*kind' \
    /Users/justinrich/Projects/gitbutler/apps/desktop/src/components/governance/ \
    | wc -l | grep '^0$'   -> Exit 0 (kind does not enter enforcement paths in UI)

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references:
  - DESIGN-LPR-002 (badge styling, kind selector options, pending indicator)
  - apps/desktop/src/components/governance/PrincipalsList.svelte:178-187
    (pending Badge precedent — the EXACT pattern to reuse for kind write pending)
  - apps/desktop/src/components/governance/PrincipalEditor.svelte:44-80
    (existing editor form shape — kind selector is a two-option control added
    alongside existing permission rows)
notes:
  - PrincipalsListEntry type extension is backward-compatible — kind?: 'agent' |
    'human' | undefined; all existing usages in tests and the parent component
    that do not pass kind continue to work (AC-3 verifies).
  - Agent badge rendering rule: `{#if principal.kind === 'agent'}` → Badge. Do
    NOT render a badge for kind='human' or absent kind.
  - The kind selector in PrincipalEditor: a SegmentControl with two segments
    (agent / human), defaulting to the principal's current kind or 'human' when
    absent. On segment change: call the LPR-013 SDK write (kindWrite(projectId,
    targetRef, principalId, newKind)); on success: mark the principal pending via
    the pendingStore callback (same mechanism as permGrant/permRevoke calls);
    on error: surface InfoMessage danger (same error pattern as PrincipalEditor).
  - isReadOnly propagation: when isReadOnly=true, the kind selector gets
    disabled=true; no SDK write fires on interaction (AC-5).
pattern: Additive optional field on PrincipalsListEntry + conditional Badge in row
  + SegmentControl editor in PrincipalEditor calling LPR-013 SDK write; pending
  indicator via the existing pendingStore ○ Badge pattern.
pattern_source: apps/desktop/src/components/governance/PrincipalsList.svelte:178-187
  (pending Badge) + PrincipalEditor.svelte (editor form + error InfoMessage) +
  GovernanceSettings.svelte (pendingStore lifecycle).
anti_pattern: Making the kind badge interactive or gate-like (it is decorative);
  defaulting absent kind to 'agent' (the conservative default is 'human');
  routing the kind write through the project-settings path instead of governance
  IPC; adding kind to any enforcement predicate in the UI.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agent: sveltekit-implementer
rationale: Surgical MODIFY to two existing Svelte components (PrincipalsList.svelte
  + PrincipalEditor.svelte) in the governance subtree. The key traps: (a) the
  additive backward-compat extension of PrincipalsListEntry, (b) the display-only
  constraint on the badge (no interactivity), (c) routing the write through the
  governance IPC path (not project-settings), and (d) reusing the EXACT pending
  Badge pattern from PrincipalsList.svelte:178-187 (not a new pending shape).
  sveltekit-implementer owns adapter-static component work for apps/desktop.
coding_standards: No relative imports — @gitbutler/ package references; Prettier
  tabs, double quotes, no trailing commas, 100-col; no console.log; Svelte 5
  $props()/$state()/$derived() rune syntax; CT describe blocks use component name
  as outermost describe string.

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: LPR-005 (the additive kind field on PrincipalWire + the typed
  reader; the read binding for kind in the governance SDK response)
Depends on: LPR-013 (the SDK write that persists kind to permissions.toml — the
  write path this editor calls; LPR-013 MUST land before this task can write kind)
Blocks:     LPR-016 (LocalReviewView renders the agent-authored tag derived from
  kind; understanding kind's read surface is prerequisite context)
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "LPR-014",
  "proposed_by": "sveltekit-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true,
    "seam_stubs": [
      { "ac": "AC-4", "kind": "component-spy", "reason": "the LPR-013 kind-write SDK binding is spied at component-test scope; the real-bus write proof is LPR-013's", "integration_bond": "LPR-013 AC-5", "status": "fixture-only" }
    ]
  },
  "fixtures": {
    "seeded_agent_principal": {
      "description": "The governance readPrincipals SDK mock returns two principals: (a) principal-A with kind='agent', ownGrants=['reviews:write'], pending=false; (b) principal-B with kind='human', ownGrants=['contents:read'], pending=false. The LPR-013 kind write spy resolves { ok: true }.",
      "seed_method": "ui_flow",
      "records": [
        "principal-A: { principalId: 'agent:codex', kind: 'agent', ownGrants: ['reviews:write'], groupMemberships: [], pending: false }",
        "principal-B: { principalId: 'human:alice', kind: 'human', ownGrants: ['contents:read'], groupMemberships: [], pending: false }",
        "LPR-013 kind write spy resolves { ok: true }"
      ]
    },
    "seeded_no_kind_principal": {
      "description": "The governance readPrincipals SDK mock returns one principal without a kind field (kind is absent/undefined). The LPR-013 kind write spy resolves { ok: true }.",
      "seed_method": "ui_flow",
      "records": [
        "principal-C: { principalId: 'ci:runner', ownGrants: ['reviews:write'], groupMemberships: [], pending: false } (NO kind field)",
        "LPR-013 kind write spy resolves { ok: true }"
      ]
    },
    "seeded_kind_write": {
      "description": "The LPR-013 kind write SDK function is a spy that resolves successfully. Used in AC-4 to verify the write is called with the correct kind value.",
      "seed_method": "ui_flow",
      "records": [
        "kindWrite(projectId, targetRef, principalId, kind) -> resolves { ok: true }",
        "pendingStore.onWrite() -> increments pendingCount by 1"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN seeded_agent_principal WHEN PrincipalsList renders THEN principal-A's row has a Badge containing 'agent'; principal-B (kind='human') has no agent badge",
      "verify": "pnpm test:ct:desktop -- PrincipalsListAgentBadge",
      "scenario": {
        "id": "SC-LPR-014-1",
        "primary": true,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "no Badge rendered for principal-A despite kind='agent' (badge logic absent — static shell)",
            "principal-B (kind='human') also shows an agent badge (filter not applied)",
            "Badge text does not contain 'agent' (wrong label from a stub)"
          ]
        },
        "evidence": { "artifact_type": "screenshot", "required_capture": true },
        "cases": [
          {
            "start_ref": "seeded_agent_principal",
            "action": { "actor": "user", "steps": [ "mount PrincipalsList with seeded_agent_principal", "observe both principal rows" ] },
            "end_state": {
              "must_observe": [
                "principal-A row contains a Badge element with text/aria-label containing 'agent'",
                "principal-B row has 0 agent badge elements"
              ],
              "must_not_observe": [
                "0 Badge elements in principal-A row (badge logic absent)",
                "an agent badge in principal-B's row (kind='human' row incorrectly badged)"
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
      "description": "GIVEN seeded_agent_principal with isReadOnly=true WHEN user interacts with the agent badge THEN badge has no Button role / aria-pressed; 0 SDK write calls fire on badge interaction; row expand/collapse unchanged",
      "verify": "pnpm test:ct:desktop -- PrincipalsListAgentBadgeDisplayOnly",
      "scenario": {
        "id": "SC-LPR-014-2",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "badge fires an SDK write call on click (badge incorrectly interactive)",
            "badge has role='button' or aria-pressed (misrepresented as a control)"
          ]
        },
        "evidence": { "artifact_type": "screenshot", "required_capture": true },
        "cases": [
          {
            "start_ref": "seeded_agent_principal",
            "action": { "actor": "user", "steps": [ "mount PrincipalsList with isReadOnly=true", "click the agent badge in principal-A's row", "observe SDK spy call count" ] },
            "end_state": {
              "must_observe": [
                "agent badge present in principal-A's row",
                "LPR-013 SDK write spy call count == 0 after badge click",
                "badge element does NOT have role='button' or aria-pressed attribute"
              ],
              "must_not_observe": [
                "any SDK write call fired from badge click",
                "badge with role='button' (interactive control)"
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
      "description": "GIVEN seeded_no_kind_principal (kind absent) WHEN PrincipalsList renders and user opens PrincipalEditor for principal-C THEN no agent badge in the row; kind selector defaults to 'human' in the editor",
      "verify": "pnpm test:ct:desktop -- PrincipalsListNoKindDefaultsHuman",
      "scenario": {
        "id": "SC-LPR-014-3",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "agent badge renders when kind is absent (wrong default — should be no badge)",
            "kind selector defaults to 'agent' instead of 'human' (wrong conservative default)",
            "PrincipalEditor crashes when kind is undefined (backward-compat regression)"
          ]
        },
        "evidence": { "artifact_type": "screenshot", "required_capture": true },
        "cases": [
          {
            "start_ref": "seeded_no_kind_principal",
            "action": { "actor": "user", "steps": [ "mount PrincipalsList with seeded_no_kind_principal", "observe principal-C row for badges", "open PrincipalEditor for principal-C", "observe the kind selector value" ] },
            "end_state": {
              "must_observe": [
                "0 agent badge elements in principal-C's row",
                "the kind selector in PrincipalEditor shows 'human' as the selected option (default)"
              ],
              "must_not_observe": [
                "an agent badge in principal-C's row when kind is absent",
                "kind selector showing 'agent' as default when kind is undefined",
                "PrincipalEditor error / crash for absent kind"
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
      "description": "GIVEN seeded_no_kind_principal; PrincipalEditor open; seeded_kind_write configured (JS spy at component-test scope, resolves {ok:true}) WHEN user changes kind selector to 'agent' THEN LPR-013 SDK write spy called == 1 with kind='agent' for that principal (correct path + payload — component call contract verified); principal-C row gains ○ Badge pending; pending-banner count +1. SCOPE: this verifies the component-level call contract (call count + args + pending-UI wiring), NOT end-to-end backend persistence. Full backend proof lives in LPR-013 AC-5 (real Tauri bus + real but-db). The seamed spy is a legitimate seam-stub at this scope: it is flagged, contract-derived (LPR-013 SDK binding), honestly gated (not reportable as full backend proof), and integration-bonded to LPR-013 AC-5.",
      "verify": "pnpm test:ct:desktop -- PrincipalKindWriteAndPending",
      "scenario": {
        "id": "SC-LPR-014-4",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "LPR-013 SDK write spy not called after kind selector change (no-op; call count 0 — component not wired to SDK binding)",
            "SDK write spy called with kind='human' instead of 'agent' (wrong value — incorrect payload)",
            "no pending ○ Badge appears on the row after spy resolves (pending pattern not wired to SDK callback)",
            "pending-banner count unchanged after kind write (pendingStore not incremented)",
            "NOTE: a static shell that renders a pending badge WITHOUT calling the SDK spy will fail the call-count == 1 assertion — this test cannot be passed by a hardcoded shell"
          ]
        },
        "evidence": { "artifact_type": "stdout", "required_capture": true },
        "cases": [
          {
            "start_ref": "seeded_no_kind_principal",
            "action": { "actor": "user", "steps": [ "open PrincipalEditor for principal-C", "change the kind selector from 'human' to 'agent'", "observe SDK spy, row badge, and pending-banner count" ] },
            "end_state": {
              "must_observe": [
                "LPR-013 kind write SDK spy called == 1 time with (projectId, targetRef, 'ci:runner', 'agent')",
                "principal-C row gains a Badge with style='warning' kind='soft' size='icon' text='○' (the existing pending pattern)",
                "governance pending-banner count incremented by 1"
              ],
              "must_not_observe": [
                "0 SDK write calls (no-op stub)",
                "SDK write called with kind='human' (wrong value)",
                "0 pending Badge on the row after write"
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
      "description": "GIVEN seeded_agent_principal; isReadOnly=true; PrincipalEditor open for principal-A WHEN user attempts to interact with the kind selector THEN kind selector has disabled or aria-disabled; 0 LPR-013 SDK write calls fire",
      "verify": "pnpm test:ct:desktop -- PrincipalKindEditorReadOnly",
      "scenario": {
        "id": "SC-LPR-014-5",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "kind selector is interactive when isReadOnly=true (isReadOnly not propagated)",
            "LPR-013 SDK write fires despite isReadOnly=true (write guard absent)"
          ]
        },
        "evidence": { "artifact_type": "screenshot", "required_capture": true },
        "cases": [
          {
            "start_ref": "seeded_agent_principal",
            "action": { "actor": "user", "steps": [ "mount PrincipalsList with isReadOnly=true", "open PrincipalEditor for principal-A", "attempt to change the kind selector" ] },
            "end_state": {
              "must_observe": [
                "kind selector has disabled or aria-disabled='true' attribute",
                "LPR-013 SDK write spy call count == 0"
              ],
              "must_not_observe": [
                "kind selector interactive when isReadOnly=true",
                "any SDK write call when isReadOnly=true"
              ]
            }
          }
        ]
      }
    },
    { "id": "TC-1", "type": "test_criterion", "description": "seeded_agent_principal → agent Badge in principal-A row; no badge in principal-B row", "verify": "pnpm test:ct:desktop -- PrincipalsListAgentBadge", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "agent badge has no role='button'; 0 SDK write calls on badge click with isReadOnly=true", "verify": "pnpm test:ct:desktop -- PrincipalsListAgentBadgeDisplayOnly", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "absent kind → 0 agent badge; PrincipalEditor kind selector defaults to 'human'", "verify": "pnpm test:ct:desktop -- PrincipalsListNoKindDefaultsHuman", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "kind selector change to 'agent' → LPR-013 SDK write spy called == 1 with kind='agent' (component call contract, NOT backend persistence — end-to-end proof is LPR-013 AC-5); ○ Badge pending on row; pending-banner count +1", "verify": "pnpm test:ct:desktop -- PrincipalKindWriteAndPending", "maps_to_ac": "AC-4", "scope_note": "component-test scope: verifies call contract + pending-UI wiring; seamed spy is contract-derived and integration-bonded to LPR-013 AC-5" },
    { "id": "TC-5", "type": "test_criterion", "description": "isReadOnly=true → kind selector disabled; 0 LPR-013 SDK write calls", "verify": "pnpm test:ct:desktop -- PrincipalKindEditorReadOnly", "maps_to_ac": "AC-5" }
  ]
}
-->
