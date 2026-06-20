# MGMT-UI-005: GovernancePendingBanner (warning InfoMessage + Commit action)

> **Red-Hat Remediation (cycle 1):** Resolved S6 (MEDIUM) — DESIGN-MGMT-001 and DESIGN-MGMT-002 added to depends_on per the SPRINT.md dependency chain.

## What this does

A thin, presentational `GovernancePendingBanner.svelte` that surfaces the pending-until-committed status: a warning `InfoMessage` showing `pendingCount` with a "Commit changes" affordance when `pendingCount > 0`, rendering nothing when `0`. It receives `pendingCount` and `onCommit` as props from `GovernanceSettings` (MGMT-UI-003) and never issues its own SDK call.

## Why

Sprint 06a · PRD UC-MGMT-06 · criteria T-MGMT-028 · capability CAP-AUTHZ-01. The visible signal that a governance edit is staged but not yet committed to the governance ref; clicking Commit clears it.

## How to verify

PRIMARY **AC-1** — `pnpm test:ct:desktop -- GovernancePendingBanner`: mounted with `pendingCount=4`, the banner renders a `warning` `InfoMessage` whose text contains `4` and a "Commit changes" button. Full gate set in the spec below.

## Scope

- `apps/desktop/src/components/governance/GovernancePendingBanner.svelte` (NEW)
- `apps/desktop/tests/governance/GovernancePendingBanner.spec.ts` (NEW — CT specs)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: MGMT-UI-005 — GovernancePendingBanner (warning InfoMessage + Commit action)
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P1
EFFORT:     XS  (30 min)
AGENT:      implementer=sveltekit-implementer | reviewer=sveltekit-reviewer
PROPOSED-BY: sveltekit-planner
SPRINT:     ./SPRINT.md
PRD_REFS:   UC-MGMT-06, T-MGMT-028
CAPABILITIES: CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  pnpm test:ct:desktop -- GovernancePendingBanner
  check: pnpm -F @gitbutler/desktop check   |   lint: pnpm lint

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
When pendingCount=4, the banner renders with warning styling containing the text "4" (or "Pending (4)")
and a "Commit changes" button. Clicking the button fires the onCommit callback exactly once. When
pendingCount=0 the banner is absent from the DOM.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] Component is PRESENTATIONAL ONLY — it receives pendingCount: number and onCommit: () => void as props.
- [MUST] Render as a warning InfoMessage when pendingCount > 0; render nothing (the {#if} guard lives INSIDE
  the component, not the parent) when pendingCount === 0.
- [MUST] The "Commit changes" button calls onCommit — NO direct SDK call from this component.
- [MUST] Reuse packages/ui InfoMessage + Button — no new design-system tokens or custom primitives.
- [NEVER] NEVER add an internal SDK call; NEVER manage a pending store inside this component; NEVER create
  new InfoMessage variants.
- [NEVER] NEVER render a "0 pending" message when pendingCount === 0 (absence is the signal).
- [STRICTLY] No relative imports — import InfoMessage/Button from @gitbutler/ui. No console.log.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: warning banner renders with the pending count when pendingCount > 0
- [ ] AC-2: banner is absent when pendingCount === 0
- [ ] AC-3: Commit changes button fires onCommit exactly once
- [ ] AC-4: reuses @gitbutler/ui InfoMessage + Button (no new design-system work)
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: warning banner renders with pending count when pendingCount > 0
  GIVEN: GovernancePendingBanner mounted with pendingCount=4
  WHEN:  the component renders
  THEN:  an InfoMessage with warning variant is visible; the rendered text contains "4"; a "Commit changes" button is present
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- GovernancePendingBanner

AC-2: banner is absent when pendingCount === 0
  GIVEN: GovernancePendingBanner mounted with pendingCount=0
  WHEN:  the component renders
  THEN:  no warning InfoMessage is rendered (element absent); the paired pendingCount=4 case still shows "4"
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- GovernancePendingBanner

AC-3: Commit changes button fires onCommit callback
  GIVEN: mounted with pendingCount=4 and a spy onCommit
  WHEN:  user clicks the "Commit changes" button
  THEN:  the onCommit callback is called exactly once (no internal SDK call)
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- GovernancePendingBanner

AC-4: reuses packages/ui InfoMessage and Button — no new design-system work [build-gate]
  GIVEN: apps/desktop/src/components/governance/GovernancePendingBanner.svelte
  WHEN:  source grep for imports
  THEN:  InfoMessage and Button are imported from @gitbutler/ui; no new design tokens / custom primitives introduced
  TEST_TIER: integration   VERIFICATION_SERVICE: grep (source invariant)
  VERIFY: grep -n '@gitbutler/ui' apps/desktop/src/components/governance/GovernancePendingBanner.svelte

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): GovernancePendingBanner with pendingCount=4 renders a warning InfoMessage containing "4" + a "Commit changes" button
    VERIFY: pnpm test:ct:desktop -- GovernancePendingBanner
- TC-2 (-> AC-2): GovernancePendingBanner with pendingCount=0 renders no warning InfoMessage
    VERIFY: pnpm test:ct:desktop -- GovernancePendingBanner
- TC-3 (-> AC-3): clicking "Commit changes" calls the onCommit prop callback exactly once
    VERIFY: pnpm test:ct:desktop -- GovernancePendingBanner
- TC-4 (-> AC-4): GovernancePendingBanner imports InfoMessage and Button from @gitbutler/ui
    VERIFY: grep -n '@gitbutler/ui' apps/desktop/src/components/governance/GovernancePendingBanner.svelte

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides: GovernancePendingBanner.svelte — warning InfoMessage tracking pendingCount + commit affordance
consumes: MGMT-UI-003 (pendingCount + onCommit props); packages/ui InfoMessage (warning) + Button
boundary_contracts:
  - presentational only: props pendingCount:number + onCommit:()=>void; the {#if pendingCount>0} guard is internal
  - the commit affordance delegates to onCommit; the banner never performs the commit/SDK call itself

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - apps/desktop/src/components/governance/GovernancePendingBanner.svelte (NEW)
  - apps/desktop/tests/governance/GovernancePendingBanner.spec.ts (NEW)
writeProhibited:
  - packages/ui/src/lib/components/InfoMessage.svelte (reuse as-is)
  - packages/ui/src/lib/components/Button.svelte (reuse as-is)
  - apps/desktop/src/components/settings/GovernanceSettings.svelte (owned by MGMT-UI-003)
  - any file not in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. packages/ui/src/lib/components/InfoMessage.svelte — style='warning' outlined=true primaryLabel + primaryAction + title/content Snippet slots
2. packages/ui/src/lib/components/Button.svelte — Button props (kind, onclick)
3. .spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md — cross-cutting states ("⚠ 4 pending governance changes … [Commit →]")
4. .spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md — Net-new components (GovernancePendingBanner.svelte location)

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- pnpm test:ct:desktop -- GovernancePendingBanner   -> Exit 0 (badge appears at 4, absent at 0, onCommit fires once)
- grep -n '@gitbutler/ui' apps/desktop/src/components/governance/GovernancePendingBanner.svelte  -> imports present
- pnpm -F @gitbutler/desktop check   -> Exit 0
- pnpm lint   -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references: DESIGN-MGMT-002 (pending-state visual contract); 10-ui-infrastructure.md cross-cutting states
notes (from DESIGN-MGMT-002 enrichment): GovernancePendingBanner is a THIN WRAPPER over InfoMessage warning.
  Props pendingCount:number, oncommit:()=>void. Renders nothing when pendingCount===0 (guard inside the wrapper).
  primaryAction calls oncommit. After commit, the pending store resets and pendingCount drops to 0, hiding the
  banner. No secondary/tertiary actions.
pattern: {#if pendingCount > 0} <InfoMessage style='warning' outlined primaryLabel='Commit changes'
  primaryAction={oncommit}>{#snippet title()}{pendingCount} pending governance change(s) — take effect once committed{/snippet}</InfoMessage> {/if}
pattern_source: packages/ui/src/lib/components/InfoMessage.svelte (existing warning-variant usage)
anti_pattern: a new banner component with custom styling; showing the banner at count 0; adding secondary actions

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: sveltekit-implementer — thin presentational Svelte component, props only
reviewer: sveltekit-reviewer
coding_standards: apps/desktop/AGENTS.md, frontend.md (no relative imports; no console.log; PascalCase component)

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: MGMT-UI-003 (pending store provides pendingCount + commit callback);
            DESIGN-MGMT-001 (four-tab IA + visual-state annotations), DESIGN-MGMT-002 (pending-state visual contract)
Blocks:     none
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "MGMT-UI-005",
  "proposed_by": "sveltekit-planner",
  "verification_policy": { "requires_tests": true, "requires_red_evidence": true, "requires_seeded_evidence": true },
  "fixtures": {
    "banner_visible": { "description": "GovernancePendingBanner mounted with pendingCount=4 and a spy onCommit, via the desktop CT harness.", "seed_method": "ui_flow", "records": ["pendingCount = 4", "onCommit = spy"] },
    "banner_hidden": { "description": "GovernancePendingBanner mounted with pendingCount=0 and a spy onCommit.", "seed_method": "ui_flow", "records": ["pendingCount = 0", "onCommit = spy"] }
  },
  "requirements": [
    { "id": "AC-1", "type": "acceptance_criterion", "primary": true, "description": "GIVEN GovernancePendingBanner mounted with pendingCount=4 WHEN it renders THEN a warning InfoMessage is visible with text containing 4 and a 'Commit changes' button", "verify": "pnpm test:ct:desktop -- GovernancePendingBanner", "scenario": { "id": "AC-1", "primary": true, "tier": "visible", "test_tier": "integration", "verification_service": "desktop-ct-harness", "negative_control": { "would_fail_if": ["a static shell renders no warning InfoMessage", "the count is hardcoded `0` instead of `4`", "the banner is disconnected from the pendingCount prop"] }, "evidence": { "artifact_type": "screenshot", "required_capture": true }, "cases": [ { "start_ref": "banner_visible", "action": { "actor": "user", "steps": ["mount the banner with pendingCount 4", "observe it"] }, "end_state": { "must_observe": ["an `InfoMessage` with `warning` variant", "the rendered text contains `4`", "a `\"Commit changes\"` button"], "must_not_observe": ["an empty banner", "`(0)` as the count"] } } ] } },
    { "id": "AC-2", "type": "acceptance_criterion", "primary": false, "description": "GIVEN GovernancePendingBanner mounted with pendingCount=0 WHEN it renders THEN no warning InfoMessage is rendered; the paired pendingCount=4 case still shows 4", "verify": "pnpm test:ct:desktop -- GovernancePendingBanner", "scenario": { "id": "AC-2", "primary": false, "tier": "visible", "test_tier": "integration", "verification_service": "desktop-ct-harness", "negative_control": { "would_fail_if": ["the banner is a static element that always shows", "the visibility is hardcoded", "the banner ignores the pendingCount prop"] }, "evidence": { "artifact_type": "screenshot", "required_capture": true }, "cases": [ { "start_ref": "banner_hidden", "action": { "actor": "user", "steps": ["mount with pendingCount 0", "observe the DOM"] }, "end_state": { "must_observe": ["the component renders nothing when `pendingCount == 0`"], "must_not_observe": ["a `\"0 pending\"` message at count 0", "a visible Commit button at count 0"] } }, { "start_ref": "banner_visible", "action": { "actor": "user", "steps": ["mount with pendingCount 4", "observe the DOM"] }, "end_state": { "must_observe": ["the warning banner appears with the count `4`"], "must_not_observe": ["an empty banner", "`(0)` as the count"] } } ] } },
    { "id": "AC-3", "type": "acceptance_criterion", "primary": false, "description": "GIVEN mounted with pendingCount=4 and a spy onCommit WHEN user clicks 'Commit changes' THEN onCommit is called exactly once (no internal SDK call)", "verify": "pnpm test:ct:desktop -- GovernancePendingBanner", "scenario": { "id": "AC-3", "primary": false, "tier": "visible", "test_tier": "integration", "verification_service": "desktop-ct-harness", "negative_control": { "would_fail_if": ["the component issues its own SDK call instead of calling onCommit (bypass)", "onCommit is never wired (no-op)", "a static button with no handler"] }, "evidence": { "artifact_type": "stdout", "required_capture": true }, "cases": [ { "start_ref": "banner_visible", "action": { "actor": "user", "steps": ["click the `\"Commit changes\"` button"] }, "end_state": { "must_observe": ["the `onCommit` callback is called exactly `1` time"], "must_not_observe": ["a direct SDK commit call from within GovernancePendingBanner", "`0` calls to onCommit"] } } ] } },
    { "id": "AC-4", "type": "acceptance_criterion", "primary": false, "description": "GIVEN GovernancePendingBanner.svelte WHEN source is grepped THEN InfoMessage and Button are imported from @gitbutler/ui (no new design-system primitives)", "verify": "grep -n '@gitbutler/ui' apps/desktop/src/components/governance/GovernancePendingBanner.svelte" },
    { "id": "TC-1", "type": "test_criterion", "description": "GovernancePendingBanner with pendingCount=4 renders a warning InfoMessage containing 4 and a 'Commit changes' button", "verify": "pnpm test:ct:desktop -- GovernancePendingBanner", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "GovernancePendingBanner with pendingCount=0 renders no warning InfoMessage", "verify": "pnpm test:ct:desktop -- GovernancePendingBanner", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "clicking 'Commit changes' calls the onCommit prop callback exactly once", "verify": "pnpm test:ct:desktop -- GovernancePendingBanner", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "GovernancePendingBanner imports InfoMessage and Button from @gitbutler/ui", "verify": "grep -n '@gitbutler/ui' apps/desktop/src/components/governance/GovernancePendingBanner.svelte", "maps_to_ac": "AC-4" }
  ]
}
-->
</details>
