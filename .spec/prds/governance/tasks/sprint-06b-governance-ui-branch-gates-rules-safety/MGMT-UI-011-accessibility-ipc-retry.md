# MGMT-UI-011: Accessibility (aria + keyboard nav) + IPC-failure danger banner + Retry

## What this does

Add aria + keyboard nav to the four-tab strip, surface IPC failures as a danger InfoMessage with Retry, and prove the self-escalation no-flip and read-only state from the UI side.

## Why

Sprint 06b · PRD UC-MGMT-07, UC-MGMT-06 · capability CAP-AUTHZ-01. CT asserts: (a) tab strip has aria-label and Tab/Arrow/Enter keyboard navigation works; (b) mocked IPC failure renders a danger InfoMessage with {code,message,remediation_hint} text and a Retry button; (c) Retry re-issues the SDK call; (d)

## How to verify

PRIMARY **AC-1** — `pnpm test:ct:desktop -- GovernanceTabsA11y`: Tab navigation has aria-labels and keyboard nav works. Full gate set in the spec below.

## Scope

- apps/desktop/src/components/settings/GovernanceSettings.svelte (MODIFY — add IPC-failure state, danger InfoMessage, Retry; read-only info InfoMessage; aria props passed to Tabs)
- apps/desktop/src/components/shared/TabList.svelte (MODIFY — add role='tablist' on <ul>, add aria-label prop, add ArrowLeft/ArrowRight/Home/End keydown handler for roving tabindex keyboard nav — CONFIRMED MISSING in live code)
- apps/desktop/src/components/shared/TabTrigger.svelte (MODIFY — FIX inverted tabindex: change tabindex={isActive ? -1 : 0} to tabindex={isActive ? 0 : -1}; active tab MUST be tabindex=0 — CONFIRMED WRONG in live code)
- apps/desktop/src/components/shared/TabContent.svelte (MODIFY — add role='tabpanel' and aria-labelledby={value} pointing to corresponding TabTrigger id — CONFIRMED MISSING in live code)
- apps/desktop/src/components/shared/Tabs.svelte (MODIFY — add keyboard nav context if needed for focus management)
- apps/desktop/tests/governance/GovernanceA11yIPC.spec.ts (NEW — CT specs for all ACs)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: MGMT-UI-011 — Accessibility (aria + keyboard nav) + IPC-failure danger banner + Retry
================================================================================

TASK_TYPE:   FEATURE
STATUS:      Backlog
PRIORITY:    P1
EFFORT:      M  (60 min)
AGENT:       sveltekit-implementer
PROPOSED-BY: sveltekit-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-MGMT-07, UC-MGMT-06
CAPABILITIES:CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  pnpm test:ct:desktop -- GovernanceTabsA11y
  check: pnpm -F @gitbutler/desktop check
  lint:  pnpm lint

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
CT asserts: (a) tab strip has aria-label and Tab/Arrow/Enter keyboard navigation works; (b) mocked IPC failure renders a danger InfoMessage with {code,message,remediation_hint} text and a Retry button; (c) Retry re-issues the SDK call; (d) persistent failure keeps the UI read-only; (e) self-escalation attempt renders a denial InfoMessage and the Toggle stays in its original position; (f) hasAdminWrite=false renders info InfoMessage and disabled controls.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] (a) Add aria-label prop to TabList.svelte and render role='tablist' on the <ul> element — this attribute is ABSENT in live code.
- [MUST] (b) FIX the inverted tabindex in TabTrigger.svelte: change tabindex={isActive ? -1 : 0} to tabindex={isActive ? 0 : -1}. Active tab MUST have tabindex=0 (focusable via Tab key); inactive tabs MUST have tabindex=-1 (removed from tab order). This is currently WRONG in the live codebase.
- [MUST] (c) Add ArrowLeft/ArrowRight/Home/End keydown handler on TabList that moves focus between TabTrigger elements (WAI-ARIA roving tabindex pattern) — NO such handler exists in live code.
- [MUST] (d) Add role='tabpanel' and aria-labelledby={value} (pointing to the corresponding TabTrigger id) to TabContent.svelte — both are ABSENT in live code.
- [MUST] (e) Wire id pairing: TabTrigger already has id={value}; TabContent must use aria-labelledby={value} to complete the association.
- [MUST] When an SDK/IPC call returns a structured denial {code, message, remediation_hint}, render it in a danger InfoMessage; the control that triggered the write must NOT reflect the change (no optimistic flip).
- [MUST] The Retry button must re-issue the exact same SDK call; on persistent failure the danger InfoMessage must remain and the UI must not re-enable controls.
- [MUST] When the viewer lacks administration:write (hasAdminWrite=false from pendingStore), ALL controls carry disabled attribute and an info InfoMessage is visible.
- [NEVER] NEVER flip a Toggle or update UI state optimistically on a denied write.
- [NEVER] NEVER add +page.server.ts.
- [NEVER] NEVER use a page reload as the Retry action — Retry re-issues the SDK call in the client.
- [NEVER] NEVER expose a superuser bypass path for the human principal — the read-only check and denial surfacing are UX; the server is the authority.
- [STRICTLY] Aria attributes follow WAI-ARIA Tab pattern (role=tablist / role=tab / aria-selected / aria-controls).
- [STRICTLY] No relative imports. No console.log. Prettier: tabs, double quotes, no trailing commas, 100-col.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: Tab navigation has aria-labels and keyboard nav works
- [ ] AC-2: IPC failure surfaces structured denial danger InfoMessage with Retry
- [ ] AC-3: Retry re-issues SDK call; persistent failure keeps UI read-only
- [ ] AC-4: Self-escalation denial surfaced without flipping the control
- [ ] AC-5: Read-only without administration:write: ALL write controls disabled + info InfoMessage visible
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: Tab navigation has aria-labels and keyboard nav works
  GIVEN: GovernanceSettings mounted and the four-tab strip rendered
  WHEN:  a keyboard user presses Tab to focus the tab strip, then Arrow keys to move between tabs, then Enter to activate
  THEN:  focus moves to the tab strip (TabList has aria-label or aria-labelledby), Arrow keys move focus between the four TabTriggers, Enter activates the focused tab and its aria-selected becomes true
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- GovernanceTabsA11y

AC-2: IPC failure surfaces structured denial danger InfoMessage with Retry
  GIVEN: GovernanceSettings mounted with seeded_ipc_failure (SDK call will reject with perm.denied)
  WHEN:  a write SDK call is triggered and it returns the structured denial
  THEN:  a danger-variant InfoMessage appears with the denial/error text; a Retry button is visible; this applies to BOTH write-path perm.denied failures AND read-path transport errors
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- GovernanceIPCFailureBanner

AC-3: Retry re-issues SDK call; persistent failure keeps UI read-only
  GIVEN: GovernanceSettings with seeded_ipc_failure and the danger InfoMessage visible with Retry
  WHEN:  user clicks Retry; then the SDK mock is configured for persistent failure (seeded_ipc_persistent_failure) and Retry is clicked again
  THEN:  first Retry re-issues the SDK call (call count increments to 2); with persistent failure the danger InfoMessage remains visible and controls stay disabled (safe read-only state)
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- GovernanceIPCRetry

AC-4: Self-escalation denial surfaced without flipping the control (pre-click aria-checked oracle)
  GIVEN: GovernanceSettings mounted with seeded_self_escalation_denial; administration:write Toggle starts OFF
  WHEN:  user clicks the administration:write Toggle to grant it to themselves
  THEN:  the test MUST capture aria-checked BEFORE the click and assert it remains unchanged after the denial; the governed path returns perm.denied; a danger InfoMessage with 'You cannot modify your own administration grants' is visible; the Toggle remains in the OFF state (pre-click aria-checked == post-denial aria-checked, not flipped)
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- GovernanceSelfEscalationNoFlip

AC-5: Read-only without administration:write: ALL write controls disabled + info InfoMessage visible
  GIVEN: GovernanceSettings mounted with seeded_read_only (hasAdminWrite=false)
  WHEN:  the component renders
  THEN:  an info-variant InfoMessage mentioning 'administration:write is required' is visible; ALL Toggles have aria-disabled=true or disabled; ALL Textbox inputs have disabled; ALL TagInputs/Selects are readonly/disabled; ALL write Buttons have disabled; 0 interactive write controls exist
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- GovernanceReadOnlyA11y

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): TabList has aria-label; ArrowRight moves focus between triggers; Enter sets aria-selected=true on the activated tab
    VERIFY: pnpm test:ct:desktop -- GovernanceTabsA11y
- TC-2 (-> AC-2): IPC failure (BOTH write-path perm.denied AND read-path transport error) renders a danger InfoMessage with error text and a Retry button
    VERIFY: pnpm test:ct:desktop -- GovernanceIPCFailureBanner
- TC-3 (-> AC-3): Retry re-issues the SDK call (call count increments); persistent failure keeps danger InfoMessage visible and controls disabled
    VERIFY: pnpm test:ct:desktop -- GovernanceIPCRetry
- TC-4 (-> AC-4): Self-escalation attempt: test captures aria-checked BEFORE the click; perm.denied surfaces danger InfoMessage with denial text; Toggle stays OFF (pre-click aria-checked == post-denial aria-checked, not flipped)
    VERIFY: pnpm test:ct:desktop -- GovernanceSelfEscalationNoFlip
- TC-5 (-> AC-5): hasAdminWrite=false: info InfoMessage with 'administration:write is required' visible; ALL Toggles/Textboxes/TagInputs/Buttons disabled; 0 interactive write controls
    VERIFY: pnpm test:ct:desktop -- GovernanceReadOnlyA11y

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides:
  - F
  - o
  - u
  - r
  - -
  - t
  - a
  - b
  -
  - n
  - a
  - v
  - i
  - g
  - a
  - t
  - i
  - o
  - n
  -
  - w
  - i
  - t
  - h
  -
  - a
  - r
  - i
  - a
  - -
  - l
  - a
  - b
  - e
  - l
  - /
  - a
  - r
  - i
  - a
  - -
  - l
  - a
  - b
  - e
  - l
  - l
  - e
  - d
  - b
  - y
  -
  - a
  - n
  - d
  -
  - k
  - e
  - y
  - b
  - o
  - a
  - r
  - d
  -
  - n
  - a
  - v
  -
  - (
  - T
  - a
  - b
  - /
  - E
  - n
  - t
  - e
  - r
  - /
  - A
  - r
  - r
  - o
  - w
  - )
  - ;
  -
  - I
  - P
  - C
  - -
  - f
  - a
  - i
  - l
  - u
  - r
  - e
  -
  - d
  - a
  - n
  - g
  - e
  - r
  -
  - I
  - n
  - f
  - o
  - M
  - e
  - s
  - s
  - a
  - g
  - e
  -
  - w
  - i
  - t
  - h
  -
  - R
  - e
  - t
  - r
  - y
  -
  - b
  - u
  - t
  - t
  - o
  - n
  -
  - t
  - h
  - a
  - t
  -
  - r
  - e
  - -
  - i
  - s
  - s
  - u
  - e
  - s
  -
  - t
  - h
  - e
  -
  - s
  - a
  - m
  - e
  -
  - S
  - D
  - K
  -
  - c
  - a
  - l
  - l
  - ;
  -
  - p
  - e
  - r
  - s
  - i
  - s
  - t
  - e
  - n
  - t
  -
  - f
  - a
  - i
  - l
  - u
  - r
  - e
  -
  - k
  - e
  - e
  - p
  - s
  -
  - t
  - h
  - e
  -
  - U
  - I
  -
  - i
  - n
  -
  - s
  - a
  - f
  - e
  -
  - r
  - e
  - a
  - d
  - -
  - o
  - n
  - l
  - y
  -
  - s
  - t
  - a
  - t
  - e
  - ;
  -
  - s
  - e
  - l
  - f
  - -
  - e
  - s
  - c
  - a
  - l
  - a
  - t
  - i
  - o
  - n
  -
  - d
  - e
  - n
  - i
  - a
  - l
  -
  - s
  - u
  - r
  - f
  - a
  - c
  - e
  - d
  -
  - a
  - s
  -
  - d
  - a
  - n
  - g
  - e
  - r
  -
  - I
  - n
  - f
  - o
  - M
  - e
  - s
  - s
  - a
  - g
  - e
  -
  - w
  - i
  - t
  - h
  - o
  - u
  - t
  -
  - f
  - l
  - i
  - p
  - p
  - i
  - n
  - g
  -
  - t
  - h
  - e
  -
  - c
  - o
  - n
  - t
  - r
  - o
  - l
  -
  - (
  - c
  - o
  - n
  - s
  - u
  - m
  - e
  - r
  - -
  - s
  - i
  - d
  - e
  -
  - p
  - r
  - o
  - o
  - f
  -
  - o
  - f
  -
  - C
  - A
  - P
  - -
  - A
  - U
  - T
  - H
  - Z
  - -
  - 0
  - 1
  -
  - n
  - o
  - -
  - b
  - y
  - p
  - a
  - s
  - s
  - )
  - ;
  -
  - r
  - e
  - a
  - d
  - -
  - o
  - n
  - l
  - y
  -
  - s
  - t
  - a
  - t
  - e
  -
  - s
  - u
  - r
  - f
  - a
  - c
  - e
  - s
  -
  - t
  - h
  - e
  -
  - i
  - n
  - f
  - o
  -
  - I
  - n
  - f
  - o
  - M
  - e
  - s
  - s
  - a
  - g
  - e
  -
  - u
  - n
  - d
  - e
  - r
  -
  - m
  - i
  - s
  - s
  - i
  - n
  - g
  -
  - a
  - d
  - m
  - i
  - n
  - i
  - s
  - t
  - r
  - a
  - t
  - i
  - o
  - n
  - :
  - w
  - r
  - i
  - t
  - e
  - .
consumes:
  - MGMT-UI-004 (GovernanceSettings wrapped in ErrorBoundary — this task's additions must be inside the boundary)
  - MGMT-UI-001 (desktop CT harness)
  - MGMT-UI-003 (GovernanceSettings — the Tabs host; from Sprint 06a)
  - MGMT-IPC-002 (json::Error remediation_hint — the structured denial shape {code, message, remediation_hint}; from Sprint 06a)
  - DESIGN-MGMT-004 (structured-denial banner + self-escalation no-flip contract)
  - DESIGN-MGMT-007 (four-tab IA + aria + keyboard-nav contract)
  - DESIGN-MGMT-008 (error-boundary fallback + IPC-failure/retry pattern)
  - packages/ui: InfoMessage (warning/info/danger variants), chipToasts, Button
boundary_contracts:
  - Self-escalation no-flip: when the governed path returns perm.denied on an administration:write grant attempt, the UI MUST surface the structured denial via a danger InfoMessage AND must NOT flip the Toggle. This is the consumer-side proof of CAP-AUTHZ-01 (T-MGMT-030).
  - Read-only without administration:write: controls are disabled and an info InfoMessage explains why — this is UX convenience; the server administration:write check is the real enforcement boundary (T-MGMT-029).
  - IPC-failure retry: the Retry button re-issues the SAME SDK call (not a page reload); on success the UI updates; on persistent failure the danger InfoMessage stays visible and the UI remains in safe read-only state (T-MGMT-040).
  - Structured denial shape: {code, message, remediation_hint} from but-authz / MGMT-IPC-002; displayed in the danger InfoMessage body.
  - This component is the SPEC-SANCTIONED seam (B14): component tests mock the IPC transport at the but-sdk seam and assert DOM/aria state. Real enforcement is proven by Rust integration tests.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - apps/desktop/src/components/settings/GovernanceSettings.svelte (MODIFY — add IPC-failure state, danger InfoMessage, Retry; read-only info InfoMessage; aria props passed to Tabs)
  - apps/desktop/src/components/shared/TabList.svelte (MODIFY — add role='tablist' on <ul>, add aria-label prop, add ArrowLeft/ArrowRight/Home/End keydown handler for roving tabindex keyboard nav — CONFIRMED MISSING in live code)
  - apps/desktop/src/components/shared/TabTrigger.svelte (MODIFY — FIX inverted tabindex: change tabindex={isActive ? -1 : 0} to tabindex={isActive ? 0 : -1}; active tab MUST be tabindex=0 — CONFIRMED WRONG in live code)
  - apps/desktop/src/components/shared/TabContent.svelte (MODIFY — add role='tabpanel' and aria-labelledby={value} pointing to corresponding TabTrigger id — CONFIRMED MISSING in live code)
  - apps/desktop/src/components/shared/Tabs.svelte (MODIFY — add keyboard nav context if needed for focus management)
  - apps/desktop/tests/governance/GovernanceA11yIPC.spec.ts (NEW — CT specs for all ACs)
writeProhibited:
  - apps/desktop/src/components/rules/* — read-only (sole change was MGMT-UI-010)
  - apps/desktop/src/components/shared/ErrorBoundary.svelte — read-only reuse
  - Any +page.server.ts or +layout.server.ts
  - packages/but-sdk/src/generated — SDK regen is MGMT-IPC-004 / MGMT-BE-003/004

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. apps/desktop/src/components/shared/TabList.svelte [1-14] — CONFIRMED MISSING: renders <ul class='segment-control-container'> with NO role attribute and NO aria-label prop — must ADD role='tablist' and aria-label prop. Full file is 14 lines.
2. apps/desktop/src/components/shared/TabTrigger.svelte [1-38] — CONFIRMED WRONG: tabindex={isActive ? -1 : 0} is INVERTED — active tab has tabindex=-1, inactive has tabindex=0. Must FLIP to tabindex={isActive ? 0 : -1}. Also missing ArrowKey keydown handler.
3. apps/desktop/src/components/shared/TabContent.svelte [1-33] — CONFIRMED MISSING: no role='tabpanel' and no aria-labelledby — must ADD both. The id pairing relies on TabTrigger's existing id={value} prop.
4. apps/desktop/src/components/shared/Tabs.svelte [1-80] — PRIMARY PATTERN — existing TabList/TabTrigger/TabContent implementation; identify where aria-label and keyboard event handlers need to be added or verified.
5. packages/ui/src/lib/components/InfoMessage.svelte [1-40] — Props: variant (warning/info/danger), default slot for body text; understand how to render the danger denial InfoMessage and the info read-only InfoMessage.
6. .spec/prds/governance/tasks/sprint-06a-governance-ui-scaffold-principals-groups/MGMT-IPC-002-json-error-remediation-hint.md [1-60] — The structured denial shape {code, message, remediation_hint} emitted by but-authz and delivered through the SDK; understand the error type used in the Svelte catch paths.
7. .spec/prds/governance/08-uc-mgmt.md [132-164] — UC-MGMT-06 AC on self-escalation no-flip + read-only; UC-MGMT-07 AC on error boundary + keyboard nav + IPC retry — the behavioral contracts this task closes.
8. apps/desktop/src/components/settings/GovernanceSettings.svelte [1-60] — The host for the Tabs — understand the component structure to identify the correct location for the IPC-failure state store and the aria additions.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- pnpm test:ct:desktop -- GovernanceTabsA11y   -> Exit 0
- pnpm test:ct:desktop -- GovernanceIPCFailureBanner   -> Exit 0
- pnpm test:ct:desktop -- GovernanceIPCRetry   -> Exit 0
- pnpm test:ct:desktop -- GovernanceSelfEscalationNoFlip   -> Exit 0
- pnpm test:ct:desktop -- GovernanceReadOnlyA11y   -> Exit 0
- pnpm -F @gitbutler/desktop check   -> Exit 0
- pnpm lint   -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references:
  - DESIGN-MGMT-004 (structured-denial banner + self-escalation no-flip contract)
  - DESIGN-MGMT-007 (four-tab IA + aria + keyboard-nav contract)
  - DESIGN-MGMT-008 (error-boundary fallback + IPC-failure/retry pattern)
  - DESIGN-MGMT-007 AC-1: WAI-ARIA tab roles — TabList role='tablist' aria-label='Governance configuration tabs'; TabTrigger role='tab' aria-selected aria-controls id; TabContent role='tabpanel' aria-labelledby id; four id pairs in canonical order (principals, groups, branch-gates, rules)
  - DESIGN-MGMT-007 AC-2: keyboard navigation model — Tab into active tab, Arrow Left/Right between tabs (wrapping), Enter/Space activate, Tab-out of panel to next focusable element; automatic-activation WAI-ARIA model
  - DESIGN-MGMT-007 AC-3: focus-visible using var(--focus-outline) or :focus-visible delegation; no suppress-without-replacement
  - DESIGN-MGMT-007 AC-4: Tabs.svelte component audit note — implementer must read to determine whether aria attributes need to be added or are already present in apps/desktop/src/components/shared/Tabs.svelte
  - DESIGN-MGMT-004 AC-1/AC-2/AC-3: denial banner + no-flip rule for self-escalation (danger InfoMessage in banner slot; Toggle reverts synchronously; no chipToast for self-escalation); three-state machine table
  - DESIGN-MGMT-008 AC-2/AC-3/AC-4: IPC-failure danger InfoMessage (style='danger' outlined=true, two-case title, Retry Button as primaryAction); persistent-failure safe read-only; two-category error distinction (render/runtime vs IPC/transport)
notes:
  - IPC-failure state: GovernanceSettings.svelte wraps each SDK call in a try/catch; on catch it stores the structured error in a $state variable; renders a danger InfoMessage with the error text + a Retry button that re-calls the same SDK fn. On success it clears the error state. Self-escalation no-flip: the PrincipalEditor/BranchGatesList Toggle onChange passes the SDK call through the same try/catch; on perm.denied it shows the danger InfoMessage but does NOT update the local $state toggle value (the toggle is bound to a derived value from the SDK response, not an optimistic local flip). Keyboard nav: the WAI-ARIA Tabs pattern with roving tabindex or explicit keydown handlers on the TabList.
  - MGMT-UI-011 is responsible for BOTH the a11y layer (DESIGN-MGMT-007) AND the IPC-failure danger banner + Retry (DESIGN-MGMT-008 AC-2/3) AND the self-escalation denial banner + no-flip (DESIGN-MGMT-004) — these are all surfaced within the GovernanceSettings banner slot and the tab navigation; the implementer must consult all three design contracts before starting
  - Banner slot priority order for MGMT-UI-011 to implement: IPC-failure danger > self-escalation danger > pending warning > read-only info > nothing — only one banner visible at a time
  - The Tabs.svelte component audit finding from DESIGN-MGMT-007 AC-4 is load-bearing: if the existing component already has role=tablist, the implementer adds only aria-label; if not, the implementer adds the full WAI-ARIA attribute set
pattern: Try/catch SDK call -> $state error -> danger InfoMessage + Retry button. Aria Tabs pattern with roving tabindex for keyboard nav.
pattern_source: apps/desktop/src/components/settings/GovernanceSettings.svelte (error handling); apps/desktop/src/components/shared/Tabs.svelte (keyboard nav starting point)
anti_pattern: Optimistic toggle flip before SDK response; page reload as Retry; chipToast instead of persistent danger InfoMessage for structural denials; aria-label on individual triggers instead of the TabList

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agent: sveltekit-implementer
rationale: Cross-cutting additions to the governance surface: aria attributes on the Tabs components, keyboard nav wiring, and an IPC-failure error state (danger InfoMessage + Retry) — plus the self-escalation no-flip and read-only denial-surfacing proofs. All work is inside the existing governance components. sveltekit-implementer owns adapter-static component work.
coding_standards: No relative imports — @gitbutler/ package references, Prettier: tabs, double quotes, no trailing commas, 100-col, No console.log, Svelte 5 $props()/$state()/$derived() rune syntax, WAI-ARIA Tabs pattern: role=tablist on TabList, role=tab + aria-selected on each TabTrigger, aria-controls pointing to the corresponding TabContent, CT describe blocks MUST use the component name as the outermost describe string (e.g. describe('GovernanceTabsA11y', () => {...})) so `pnpm test:ct:desktop -- <ComponentName>` grep matches reliably.

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: MGMT-UI-004 (GovernanceSettings wrapped in ErrorBoundary — a11y + IPC additions must be inside the boundary); MGMT-UI-001 (desktop CT harness; from Sprint 06a); MGMT-IPC-002 (structured denial shape {code,message,remediation_hint}; from Sprint 06a); DESIGN-MGMT-004 (structured-denial banner + self-escalation no-flip visual contract); DESIGN-MGMT-007 (four-tab aria + keyboard-nav contract); DESIGN-MGMT-008 (IPC-failure/retry pattern)
Blocks:     MGMT-UI-012 (build-gate tests verify no +page.server.ts and no direct config write across all governance components including those modified here)
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "MGMT-UI-011",
  "proposed_by": "sveltekit-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "seeded_ipc_failure": {
      "description": "but-sdk mock layer configured to reject the next governance SDK call with a structured denial: {code:'perm.denied', message:'Permission denied: administration:write required', remediation_hint:'Contact your repository administrator to grant administration:write.'}",
      "seed_method": "ui_flow",
      "records": [
        "SDK call returns {code:'perm.denied', message:'Permission denied: administration:write required', remediation_hint:'Contact your repository administrator to grant administration:write.'}"
      ]
    },
    "seeded_self_escalation_denial": {
      "description": "but-sdk mock configured so that a grant of administration:write to the current principal returns {code:'perm.denied', message:'You cannot modify your own administration grants.', remediation_hint:'Self-escalation is not permitted.'}. The Toggle starts OFF.",
      "seed_method": "ui_flow",
      "records": [
        "administration:write Toggle initial state: OFF",
        "SDK grant call returns {code:'perm.denied', message:'You cannot modify your own administration grants.', remediation_hint:'Self-escalation is not permitted.'}"
      ]
    },
    "seeded_read_only": {
      "description": "governance_status_read returns hasAdminWrite=false. All controls should be disabled.",
      "seed_method": "ui_flow",
      "records": [
        "hasAdminWrite=false"
      ]
    },
    "seeded_ipc_persistent_failure": {
      "description": "but-sdk mock configured to reject EVERY call (including retries) with the same perm.denied denial.",
      "seed_method": "ui_flow",
      "records": [
        "All SDK calls return {code:'perm.denied', message:'Permission denied: administration:write required', remediation_hint:'Contact your repository administrator to grant administration:write.'}"
      ]
    },
    "seeded_read_ipc_failure": {
      "description": "but-sdk mock configured so that a READ call (e.g. governance_status_read or branch_gates_read) rejects with a transport/network error: {type:'transport_error', message:'Backend unreachable', code:'network.error'} \u2014 NOT a perm.denied structured denial. Tests that READ failures (not only write failures) also trigger the danger banner.",
      "seed_method": "ui_flow",
      "records": [
        "governance_status_read (or branch_gates_read) returns transport error {type:'transport_error', message:'Backend unreachable', code:'network.error'}",
        "This covers UC-MGMT-07 AC-3 transport/timeout failure mode"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN GovernanceSettings mounted and the four-tab strip rendered WHEN a keyboard user presses Tab to focus the tab strip, then Arrow keys to move between tabs, then Enter to activate THEN focus moves to the tab strip (TabList has aria-label or aria-labelledby), Arrow keys move focus between the four TabTriggers, Enter activates the focused tab and its aria-selected becomes true",
      "verify": "pnpm test:ct:desktop -- GovernanceTabsA11y",
      "scenario": {
        "id": "SC-MGMT-UI-011-1",
        "primary": true,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "the TabList has no `aria-label` attribute (static shell renders tabs without aria attributes)",
            "Arrow keys do not move focus between tabs (keyboard nav not wired \u2014 a no-op or hardcoded handler)",
            "Enter does not activate the focused tab (`aria-selected` stays `'false'` on all triggers)",
            "TabList <ul> has no `role='tablist'` attribute (WAI-ARIA role missing \u2014 static shell without proper semantics)",
            "active TabTrigger has tabindex='-1' (inverted tabindex not fixed \u2014 keyboard focus skips the active tab)",
            "TabContent has no `role='tabpanel'` (panel role missing \u2014 AT cannot announce the panel)"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_read_only",
            "action": {
              "actor": "user",
              "steps": [
                "mount GovernanceSettings",
                "press Tab until focus reaches the tab strip",
                "press ArrowRight to move from Principals to Groups",
                "press Enter to activate the Groups tab"
              ]
            },
            "end_state": {
              "must_observe": [
                "the TabList element has `aria-label='Governance settings navigation'` (or equivalent non-empty `aria-label`/`aria-labelledby` value)",
                "after ArrowRight: the Groups TabTrigger element has DOM focus (accessible name `'Groups'`)",
                "after Enter: Groups TabTrigger has `aria-selected='true'`",
                "after Enter: the Groups tab content panel is associated via `aria-controls` and is not `hidden`",
                "the TabList `<ul>` element has `role='tablist'` attribute",
                "the Groups TabContent panel has `role='tabpanel'` and `aria-labelledby` matching the Groups TabTrigger `id` attribute"
              ],
              "must_not_observe": [
                "a TabList with `aria-label` attribute absent (no aria-label on the tablist element)",
                "Principals TabTrigger retaining focus after ArrowRight (focus did not move)",
                "`aria-selected='false'` on all `4` tabs after Enter (no tab activated)",
                "the active TabTrigger has `tabindex='-1'` (inverted tabindex \u2014 active tab MUST have tabindex='0')"
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
      "description": "GIVEN GovernanceSettings mounted with seeded_ipc_failure (SDK call will reject with perm.denied) WHEN a write SDK call is triggered and it returns the structured denial THEN a danger-variant InfoMessage appears with the denial/error text; a Retry button is visible; this applies to BOTH write-path perm.denied failures AND read-path transport errors",
      "verify": "pnpm test:ct:desktop -- GovernanceIPCFailureBanner",
      "scenario": {
        "id": "SC-MGMT-UI-011-2",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "the danger InfoMessage does not appear on IPC failure (error swallowed \u2014 `0` InfoMessage elements with `style='danger'`)",
            "the Retry button is absent (`0` buttons with accessible name `'Retry'`)",
            "the InfoMessage text does not include the `remediation_hint` (static/hardcoded text, not from SDK response)",
            "a chipToast is shown instead of the persistent danger InfoMessage banner (wrong component)"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_ipc_failure",
            "action": {
              "actor": "user",
              "steps": [
                "trigger a governance write SDK call (e.g. attempt a perm_grant)",
                "observe the rendered error state"
              ]
            },
            "end_state": {
              "must_observe": [
                "an `InfoMessage` with `style='danger'` (or equivalent `variant='danger'` prop) present in the DOM",
                "the InfoMessage body text contains `'perm.denied'` or `'Permission denied'` (from the seeded `code`/`message`)",
                "the InfoMessage body text contains `'Contact your repository administrator'` (the seeded `remediation_hint`)",
                "a button with accessible name `'Retry'` present inside or beside the InfoMessage"
              ],
              "must_not_observe": [
                "`0` InfoMessage elements with `style='danger'` (error silently swallowed)",
                "a non-danger (`warning` or `info`) InfoMessage variant for a `perm.denied` error",
                "`0` buttons with accessible name `'Retry'`"
              ]
            }
          },
          {
            "start_ref": "seeded_read_ipc_failure",
            "action": {
              "actor": "user",
              "steps": [
                "configure seeded_read_ipc_failure (governance_status_read rejects with transport error)",
                "mount GovernanceSettings (the READ call fires on mount)",
                "observe the rendered error state without any user write action"
              ]
            },
            "end_state": {
              "must_observe": [
                "an `InfoMessage` with `style='danger'` present in the DOM even though no user write action was taken (the failed READ call triggers the banner)",
                "the InfoMessage body contains `'Backend unreachable'` or `'network.error'` (from the seeded transport error)",
                "a button with accessible name `'Retry'` present (Retry re-issues the READ call)"
              ],
              "must_not_observe": [
                "`0` InfoMessage elements with `style='danger'` when a READ call fails (read-path failures silently swallowed)",
                "the UI rendering normally as if no error occurred (READ failure treated as empty response)"
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
      "description": "GIVEN GovernanceSettings with seeded_ipc_failure and the danger InfoMessage visible with Retry WHEN user clicks Retry; then the SDK mock is configured for persistent failure (seeded_ipc_persistent_failure) and Retry is clicked again THEN first Retry re-issues the SDK call (call count increments to 2); with persistent failure the danger InfoMessage remains visible and controls stay disabled (safe read-only state)",
      "verify": "pnpm test:ct:desktop -- GovernanceIPCRetry",
      "scenario": {
        "id": "SC-MGMT-UI-011-3",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "Retry triggers a page reload instead of re-issuing the SDK call (call count stays at `1`, no new SDK call)",
            "after persistent failure the controls re-enable (unsafe state \u2014 `disabled` attribute removed or `0` disabled controls)",
            "the SDK call count stays at `1` after Retry (no-op stub \u2014 Retry is a static button with no handler)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_ipc_persistent_failure",
            "action": {
              "actor": "user",
              "steps": [
                "trigger a write that fails (SDK call count: 1)",
                "click Retry (SDK call count should become: 2)",
                "observe the error state after second failure"
              ]
            },
            "end_state": {
              "must_observe": [
                "SDK spy call count `== 2` (the same governance SDK fn re-issued by Retry)",
                "the `InfoMessage` with `style='danger'` still present after the second failure (`1` danger InfoMessage element)",
                "at least `1` interactive control (Toggle or Textbox) carrying `disabled` attribute (safe read-only state maintained)"
              ],
              "must_not_observe": [
                "SDK spy call count `== 1` after clicking Retry (no re-issue \u2014 `0` additional calls)",
                "controls with `disabled` attribute `absent` after persistent failure (re-enabled \u2014 unsafe)",
                "the danger InfoMessage `empty`/absent after a failed retry (`0` danger InfoMessage elements)"
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
      "description": "GIVEN GovernanceSettings mounted with seeded_self_escalation_denial; administration:write Toggle starts OFF WHEN user clicks the administration:write Toggle to grant it to themselves THEN the test MUST capture aria-checked BEFORE the click and assert it remains unchanged after the denial; the governed path returns perm.denied; a danger InfoMessage with 'You cannot modify your own administration grants' is visible; the Toggle remains in the OFF state (pre-click aria-checked == post-denial aria-checked, not flipped)",
      "verify": "pnpm test:ct:desktop -- GovernanceSelfEscalationNoFlip",
      "scenario": {
        "id": "SC-MGMT-UI-011-4",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "the Toggle flips to `aria-checked='true'` optimistically before the SDK response (no-bypass promise broken)",
            "the danger InfoMessage is absent after the denied call (`0` InfoMessage elements with `style='danger'`)",
            "the Toggle has `aria-checked='true'` after `perm.denied` (optimistic flip not reverted)",
            "a Retry button appears on a perm.denied self-escalation denial (unified error banner with generic Retry bypasses the no-Retry-on-denial contract)",
            "stub Toggle always reports `aria-checked='true'` regardless of click \u2014 without a pre-click aria-checked assertion, this stub passes (no-op-handler stub-pass vector closed by REMEDIATE-UI-5)"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_self_escalation_denial",
            "action": {
              "actor": "user",
              "steps": [
                "locate the administration:write Toggle (initial state: OFF)",
                "capture the pre-click aria-checked value BEFORE the click (REMEDIATE-UI-5 strengthening)",
                "click the Toggle to attempt self-escalation",
                "capture the post-denial aria-checked value AFTER the perm.denied response",
                "assert the pre-click and post-denial aria-checked values are EQUAL (no flip)",
                "observe the Toggle state and InfoMessage"
              ]
            },
            "end_state": {
              "must_observe": [
                "an `InfoMessage` with `style='danger'` containing text `'cannot modify your own administration grants'`",
                "a pre-click `aria-checked='false'` capture/assertion is present (Toggle starts OFF, captured BEFORE the click)",
                "the administration:write Toggle has `aria-checked='false'` (remains in OFF state, not flipped) AFTER the denial",
                "an equality/comparison assertion between the pre-click and post-denial aria-checked values is present (closes the no-op stub Toggle vector)",
                "the InfoMessage body contains `'Self-escalation is not permitted'` (the seeded `remediation_hint`)"
              ],
              "must_not_observe": [
                "the Toggle with `aria-checked='true'` (optimistic flip occurred)",
                "`0` InfoMessage elements with `style='danger'` after the denied self-escalation attempt",
                "a `warning` or `info` InfoMessage variant (wrong variant for a `perm.denied` denial)",
                "a button with accessible name `'Retry'` visible when the denial code is `'perm.denied'` (self-escalation denials MUST NOT offer Retry \u2014 DESIGN-MGMT-004 contract)",
                "only a post-click aria-checked assertion with no pre-click capture (a stub Toggle that always reports `aria-checked='true'` would pass such a weak oracle)",
                "pre-click and post-denial aria-checked values captured without an equality/comparison step"
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
      "description": "GIVEN GovernanceSettings mounted with seeded_read_only (hasAdminWrite=false) WHEN the component renders THEN an info-variant InfoMessage mentioning 'administration:write is required' is visible; ALL Toggles have aria-disabled=true or disabled; ALL Textbox inputs have disabled; ALL TagInputs/Selects are readonly/disabled; ALL write Buttons have disabled; 0 interactive write controls exist",
      "verify": "pnpm test:ct:desktop -- GovernanceReadOnlyA11y",
      "scenario": {
        "id": "SC-MGMT-UI-011-5",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "any Toggle is interactive when hasAdminWrite=false (partial disabling \u2014 only 1 control checked)",
            "any Textbox is enabled when hasAdminWrite=false (read-only state not propagated to Textbox)",
            "the info InfoMessage is absent (`0` InfoMessage elements with `style='info'`)",
            "write Buttons remain enabled when hasAdminWrite=false"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_read_only",
            "action": {
              "actor": "user",
              "steps": [
                "mount GovernanceSettings with hasAdminWrite=false",
                "observe all interactive controls and the InfoMessage"
              ]
            },
            "end_state": {
              "must_observe": [
                "an `InfoMessage` with `style='info'` containing the text `'administration:write is required'`",
                "every Toggle element has `aria-disabled='true'` or `disabled` attribute (0 enabled Toggles in the DOM when hasAdminWrite=false)",
                "every Textbox input has `disabled` attribute (0 enabled Textbox inputs)",
                "every TagInput and Select element is `readonly` or `disabled` (0 interactive write selectors)",
                "every write Button (excluding tab navigation buttons) has `disabled` attribute",
                "`0` interactive write controls in the DOM"
              ],
              "must_not_observe": [
                "any enabled Toggle (aria-disabled absent and disabled absent) when hasAdminWrite=false",
                "any enabled Textbox (disabled absent) when hasAdminWrite=false",
                "`0` InfoMessage elements with `style='info'` in read-only state",
                "a `danger` InfoMessage variant (wrong variant for read-only \u2014 that is for denial)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "TabList has aria-label; ArrowRight moves focus between triggers; Enter sets aria-selected=true on the activated tab",
      "verify": "pnpm test:ct:desktop -- GovernanceTabsA11y",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "IPC failure (BOTH write-path perm.denied AND read-path transport error) renders a danger InfoMessage with error text and a Retry button",
      "verify": "pnpm test:ct:desktop -- GovernanceIPCFailureBanner",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "Retry re-issues the SDK call (call count increments); persistent failure keeps danger InfoMessage visible and controls disabled",
      "verify": "pnpm test:ct:desktop -- GovernanceIPCRetry",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "Self-escalation attempt: perm.denied surfaces danger InfoMessage with denial text; Toggle stays OFF (not flipped)",
      "verify": "pnpm test:ct:desktop -- GovernanceSelfEscalationNoFlip",
      "maps_to_ac": "AC-4"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "hasAdminWrite=false: info InfoMessage with 'administration:write is required' visible; ALL Toggles/Textboxes/TagInputs/Buttons disabled; 0 interactive write controls",
      "verify": "pnpm test:ct:desktop -- GovernanceReadOnlyA11y",
      "maps_to_ac": "AC-5"
    }
  ]
}
-->
