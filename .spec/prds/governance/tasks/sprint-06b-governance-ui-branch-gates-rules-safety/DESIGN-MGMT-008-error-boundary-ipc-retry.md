# DESIGN-MGMT-008: Error-boundary fallback + IPC-failure/retry pattern

## What this does

Specify (a) the fallback content the existing shared/ErrorBoundary.svelte renders when GovernanceSettings or a child throws a render/runtime error, and (b) the IPC-failure danger InfoMessage with Retry button and the persistent-failure safe read-only state for Tauri SDK call failures — distinguishing the two error categories and specifying them separately. No new boundary component is introduced.

## Why

Sprint 06b · PRD UC-MGMT-07 · capability —. A sveltekit-implementer reading this contract knows (a) exactly what title/compact props to pass when wrapping GovernanceSettings in ErrorBoundary, (b) what InfoMessage props to use for the IPC-failure banner, (c) what the Retry button does

## How to verify

PRIMARY **AC-1** — `design review — reviewer confirms all five items are present: component path, title prop, compact=false, no Retry in boundary fallback, modal survival`: ErrorBoundary fallback content specification [PRIMARY]. Full gate set in the spec below.

## Scope

  - apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md (MODIFY — extend with Sprint 06b error-boundary + IPC-failure section)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: DESIGN-MGMT-008 — Error-boundary fallback + IPC-failure/retry pattern
================================================================================

TASK_TYPE:   DESIGN
STATUS:      Backlog
PRIORITY:    P0
EFFORT:      XS  (30 min)
AGENT:       frontend-designer
PROPOSED-BY: frontend-designer
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-MGMT-07
CAPABILITIES:—

RUNTIME_COMMANDS:
  test:  pnpm test:ct:desktop -- GovernanceErrorBoundary GovernanceIPCFailure (exercised by MGMT-UI-004 and MGMT-UI-011 implementations)

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
A sveltekit-implementer reading this contract knows (a) exactly what title/compact props to pass when wrapping GovernanceSettings in ErrorBoundary, (b) what InfoMessage props to use for the IPC-failure banner, (c) what the Retry button does and what happens on persistent failure, and (d) that render errors and IPC errors are handled by different mechanisms — without making error-handling design decisions independently.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST specify the fallback content shown by the EXISTING apps/desktop/src/components/shared/ErrorBoundary.svelte (the 'failed' snippet) when a governance child component throws: what title prop to pass, whether to use compact=false, and what to render in the error.message sub-line — do NOT design a new boundary component (MGMT-UI-004 wraps GovernanceSettings.svelte in the existing ErrorBoundary; no new GovernanceErrorBoundary.svelte).
- [MUST] MUST distinguish two error categories with different visual treatments: (1) render/runtime error caught by the Svelte boundary → the ErrorBoundary 'failed' snippet fallback (title + error.message, no retry); (2) IPC/transport error from a failed Tauri SDK call → danger InfoMessage with a Retry Button (no boundary catch, stays in-page).
- [MUST] MUST specify the IPC-failure danger InfoMessage: InfoMessage (packages/ui/src/lib/components/InfoMessage.svelte) style='danger' outlined=true, a title surfacing the structured denial {code, message} or 'Connection lost — governance service unavailable' if no structured response, a Retry Button (packages/ui/src/lib/components/Button.svelte) as the primary action.
- [MUST] MUST specify the persistent-failure safe read-only state: if Retry continues to fail, the IPC-failure danger InfoMessage stays visible and the governance surface stays in read-only mode (isReadOnly=true effectively, controls inert) — the page does NOT crash or show the boundary fallback for an IPC error.
- [MUST] MUST confirm the settings modal survives a governance child render failure — the ErrorBoundary fallback renders inside the Permissions & Governance section only; the modal frame and other settings sections remain functional.
- [NEVER] NEVER design a new GovernanceErrorBoundary.svelte component — MGMT-UI-004 wraps in the existing shared/ErrorBoundary; this task specifies the props/content to pass to it.
- [NEVER] NEVER introduce a new design-system token, CSS variable, or hex color not already in packages/ui.
- [NEVER] NEVER conflate the two error categories: boundary catch (render/runtime) and IPC error (network/transport) have different visual treatments; a render error does NOT show a Retry button (the boundary provides a title+message only); an IPC error does NOT trigger the Svelte boundary.
- [NEVER] NEVER specify a full-page error state that obscures the settings modal frame — the ErrorBoundary fallback is scoped to the GovernanceSettings mount point only.
- [STRICTLY] STRICTLY the existing shared/ErrorBoundary.svelte is the boundary — read its 'failed' snippet API (title prop, compact prop, error.message rendering) before specifying the content.
- [STRICTLY] STRICTLY the IPC-failure Retry uses packages/ui/src/lib/components/Button.svelte — no custom retry affordance.
- [STRICTLY] STRICTLY persistent IPC failure → safe read-only state (controls inert, danger InfoMessage stays); the page MUST NOT unmount or show the boundary fallback on an IPC error.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: ErrorBoundary fallback content specification [PRIMARY]
- [ ] AC-2: IPC-failure danger banner + Retry specification
- [ ] AC-3: Persistent-failure safe read-only state
- [ ] AC-4: Two-category error distinction
- [ ] AC-5: No new design-system tokens
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: ErrorBoundary fallback content specification [PRIMARY]
  GIVEN: the error-boundary contract section of DESIGN-ANNOTATIONS.md covers the render/runtime error case
  WHEN:  a reviewer inspects the ErrorBoundary fallback content specification
  THEN:  it specifies: (1) the existing shared/ErrorBoundary.svelte (apps/desktop/src/components/shared/ErrorBoundary.svelte) is the boundary — no new component; (2) GovernanceSettings.svelte is wrapped in ErrorBoundary with title='Governance settings failed to load', compact=false; (3) the 'failed' snippet renders error.message as the sub-line if it is an Error instance; (4) no Retry button in the boundary fallback (the fallback is informational; recovery is reopening the settings modal); (5) the settings modal frame and other settings sections remain functional — the ErrorBoundary fallback renders only inside the Permissions & Governance section mount point
  TEST_TIER: component   VERIFICATION_SERVICE: apps/desktop CT harness (pnpm test:ct:desktop) — downstream T-MGMT-037 component test asserts the error boundary catches a thrown error and shows the fallback without breaking the modal
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test (pnpm test:ct:desktop) exercising T-MGMT-037
  VERIFY: design review — reviewer confirms all five items are present: component path, title prop, compact=false, no Retry in boundary fallback, modal survival

AC-2: IPC-failure danger banner + Retry specification
  GIVEN: the contract covers the IPC/transport error case
  WHEN:  a reviewer inspects the IPC-failure InfoMessage specification
  THEN:  it specifies: InfoMessage (packages/ui/src/lib/components/InfoMessage.svelte) style='danger' outlined=true; title renders the structured denial message if {code, message} is available from the SDK response, or 'Connection lost — governance service unavailable' if no structured response; a Retry Button (packages/ui/src/lib/components/Button.svelte) is the primaryLabel/primaryAction slot of the InfoMessage; the Retry button re-issues the same SDK call that failed; the banner appears in the GovernanceSettings banner slot (same slot as the denial banner from DESIGN-MGMT-004), taking highest priority
  TEST_TIER: component   VERIFICATION_SERVICE: apps/desktop CT harness (pnpm test:ct:desktop) — downstream T-MGMT-039 component test asserts IPC failure surfaces danger InfoMessage with a Retry button
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test (pnpm test:ct:desktop) exercising T-MGMT-039
  VERIFY: design review — reviewer confirms InfoMessage path + style='danger' + outlined=true + two-case title text + Retry Button path + primaryAction slot + banner-slot placement

AC-3: Persistent-failure safe read-only state
  GIVEN: the contract specifies the persistent IPC failure state
  WHEN:  a reviewer reads the persistent-failure section
  THEN:  it states: (1) on Retry success — the danger banner hides, the governance surface resumes normal state; (2) on Retry failure — the danger InfoMessage stays visible with an updated or unchanged message; (3) on persistent failure — the governance surface stays in a safe read-only state (isReadOnly=true equivalent: all controls inert, no writes attempted); (4) the surface does NOT unmount or trigger the ErrorBoundary on an IPC error — the IPC error is handled in-page via the banner, not via the Svelte boundary mechanism
  TEST_TIER: component   VERIFICATION_SERVICE: apps/desktop CT harness (pnpm test:ct:desktop) — downstream T-MGMT-040 component test asserts retry re-issues; persistent failure keeps the safe read-only state
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test (pnpm test:ct:desktop) exercising T-MGMT-040
  VERIFY: design review — reviewer confirms all four items: retry-success clears banner, retry-failure keeps banner, persistent-failure read-only, IPC error does not trigger boundary

AC-4: Two-category error distinction
  GIVEN: the contract covers both error categories
  WHEN:  a reviewer reads the error-category distinction section
  THEN:  it explicitly names the two categories in a side-by-side table or clear callout: {render/runtime error: caught by Svelte boundary → ErrorBoundary 'failed' snippet → title+message only, no Retry, modal survives} and {IPC/transport error: SDK call rejects → in-page danger InfoMessage + Retry → persistent failure → safe read-only, boundary NOT triggered}
  TEST_TIER: component   VERIFICATION_SERVICE: apps/desktop CT harness (pnpm test:ct:desktop) — T-MGMT-037 (boundary catches render error) and T-MGMT-039 (IPC failure in-page banner) together prove the distinction
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component tests
  VERIFY: design review — reviewer confirms the two-category table or callout is present with the correct treatment per category and the no-boundary-for-IPC-error rule stated

AC-5: No new design-system tokens
  GIVEN: the error-boundary + IPC-failure section extends DESIGN-ANNOTATIONS.md
  WHEN:  a reviewer audits every visual reference in the section
  THEN:  every visual attribute uses an existing CSS variable or defers to the component's own stylesheet — no hex literals, no new var(--error-*)/var(--mgmt-*) tokens
  TEST_TIER: component   VERIFICATION_SERVICE: grep audit on the annotation file
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by static grep on the annotation file
  VERIFY: grep the error section of DESIGN-ANNOTATIONS.md for '#' color literals and var(--error-)/var(--mgmt-) -> both return zero matches

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): contract names shared/ErrorBoundary.svelte path, title='Governance settings failed to load', compact=false, no Retry in boundary fallback, modal-survival rule, error.message sub-line — all five items present
    VERIFY: design review of the ErrorBoundary fallback content section in DESIGN-ANNOTATIONS.md
- TC-2 (-> AC-2): contract names InfoMessage style='danger' outlined=true, two-case title text (structured vs. 'Connection lost'), Retry Button as primaryAction, banner-slot placement
    VERIFY: design review of the IPC-failure InfoMessage section in DESIGN-ANNOTATIONS.md
- TC-3 (-> AC-3): contract names all four persistent-failure items: retry-success clears banner, retry-failure keeps banner, persistent-failure = read-only, IPC error does not trigger ErrorBoundary
    VERIFY: design review of the persistent-failure section in DESIGN-ANNOTATIONS.md
- TC-4 (-> AC-4): contract contains a two-category table or callout distinguishing render/runtime error (boundary) from IPC/transport error (in-page banner); the no-boundary-for-IPC-error rule is explicitly stated
    VERIFY: design review of the error-category distinction section in DESIGN-ANNOTATIONS.md
- TC-5 (-> AC-5): zero hex color literals and zero new var(--error-*)/var(--mgmt-*) tokens in the error section
    VERIFY: grep -nE '#[0-9a-fA-F]{3,6}|var\(--(error|mgmt)-' apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md -> zero matches

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md (MODIFY — extend with Sprint 06b error-boundary + IPC-failure section)
writeProhibited:
  - apps/desktop/src/components/shared/ErrorBoundary.svelte — read only; do not modify the boundary component
  - packages/ui/src/lib/components/** — no design-system changes
  - any .svelte or .ts implementation file — design spec artifact only

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. apps/desktop/src/components/shared/ErrorBoundary.svelte [1-67] — Full component source: 'failed' snippet API (title prop, compact prop, error.message rendering via {#if error instanceof Error && error.message}) — MUST read before specifying fallback content [PRIMARY PATTERN]
2. .spec/prds/governance/08-uc-mgmt.md [155-164] — UC-MGMT-07: error boundary wraps GovernanceSettings; IPC failure surfaces structured denial + Retry; persistent failure → safe read-only; settings modal survives
3. packages/ui/src/lib/components/InfoMessage.svelte [1-60] — style='danger', outlined, primaryLabel/primaryAction props — confirm Button can be passed as primaryAction for the Retry affordance
4. .spec/prds/governance/tasks/sprint-06a-governance-ui-scaffold-principals-groups/DESIGN-MGMT-004-STUB.md [1-10] — NOTE: DESIGN-MGMT-004 (this sprint) specifies the denial banner using the same InfoMessage danger variant — IPC-failure banner is a parallel usage of the same component; ensure the two do not conflict in the banner slot priority rule
5. .spec/prds/governance/11-e2e-testing-criteria.md [231-238] — T-MGMT-037/039/040/041 — the exact component-test criteria this contract must satisfy

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- design review of apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md error section   -> five items present: shared/ErrorBoundary.svelte path, title='Governance settings failed to load', compact=false, no Retry in boundary fallback, modal-survival rule
- design review of the IPC-failure InfoMessage section   -> InfoMessage style='danger' + outlined=true + two-case title text + Retry Button as primaryAction + banner-slot placement — all present
- design review of the persistent-failure section   -> four items: retry-success clears banner, retry-failure keeps banner, persistent = safe read-only, IPC error does not trigger boundary
- design review of the error-category section   -> a table or callout with render/runtime (boundary) vs IPC/transport (in-page banner) entries; no-boundary-for-IPC rule explicit
- grep -nE '#[0-9a-fA-F]{3,6}|var\(--(error|mgmt)-' apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md   -> zero matches
- pnpm test:ct:desktop -- GovernanceErrorBoundary GovernanceIPCFailure (exercised by MGMT-UI-004 and MGMT-UI-011 implementations)   -> T-MGMT-037 (boundary catches thrown error, fallback renders), T-MGMT-039 (IPC failure danger InfoMessage + retry button), T-MGMT-040 (retry re-issues; persistent failure = safe read-only) all pass

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references:
  - .spec/prds/governance/08-uc-mgmt.md UC-MGMT-07 (lines 155-164): 'An error boundary wraps GovernanceSettings.svelte; IPC failures surface the structured denial {code, message, remediation_hint} via a danger InfoMessage with a Retry action; on persistent failure the UI stays in a safe read-only state'
  - apps/desktop/src/components/shared/ErrorBoundary.svelte: the existing Svelte 5 svelte:boundary component whose 'failed' snippet API (title prop, compact prop, error.message sub-line rendering) is the only boundary used — no new boundary component
notes:
  - two error categories, two mechanisms: (1) render/runtime error — Svelte's svelte:boundary catches it in the 'failed' snippet; the ErrorBoundary renders title + error.message; NO Retry; the settings modal frame survives; (2) IPC/transport error — NOT caught by the boundary; handled in-page by the component that made the SDK call; danger InfoMessage appears with Retry button
  - banner-slot priority for IPC error: IPC-failure danger banner uses the same GovernanceSettings banner slot as the self-escalation denial banner (DESIGN-MGMT-004) and the pending warning banner (DESIGN-MGMT-002); priority: IPC-failure danger > self-escalation danger > pending warning > read-only info
  - Retry mechanics: the Retry button calls the same SDK function that failed (the component that owns the failed call owns the retry logic); the design contract specifies the button label ('Retry') and that it re-issues the same call — not the implementation of the retry queue
  - persistent failure read-only: after N consecutive Retry failures the surface stays in read-only mode; the danger banner remains; the user can still navigate between tabs; no page crash or boundary trigger
  - the ErrorBoundary title prop 'Governance settings failed to load' is the human-readable fallback label — it must be passed by the MGMT-UI-004 implementer as the title prop to shared/ErrorBoundary
pattern: Two-tier error handling: Svelte boundary (render errors, informational fallback, no Retry) + in-page danger InfoMessage (IPC errors, Retry button, persistent safe read-only). The boundary wraps the entire GovernanceSettings.svelte mount point. The IPC-failure banner uses the existing GovernanceSettings banner slot at highest priority.
pattern_source: apps/desktop/src/components/shared/ErrorBoundary.svelte 'failed' snippet (the existing GitButler error-boundary pattern); packages/ui InfoMessage danger variant (used for the IPC-failure banner consistent with the self-escalation denial banner from DESIGN-MGMT-004).
anti_pattern: a new GovernanceErrorBoundary.svelte component (the existing shared/ErrorBoundary is the boundary; MGMT-UI-004 wraps GovernanceSettings in it); showing a Retry button inside the ErrorBoundary 'failed' snippet (the boundary fallback is informational; Retry belongs on the IPC-failure in-page banner); allowing an IPC error to trigger the Svelte boundary (IPC errors must be caught in-page, not left as uncaught throws)

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agent: frontend-designer
rationale: frontend-designer owns the visual content and interaction contract for the error-boundary fallback and the IPC-failure danger banner/retry pattern; no Rust/Tauri knowledge required — the design contract is consumed by sveltekit-implementer (MGMT-UI-004, MGMT-UI-011).
coding_standards: The ErrorBoundary component path must be cited exactly: apps/desktop/src/components/shared/ErrorBoundary.svelte, The ErrorBoundary title prop value must be a human-readable string that will make sense to a non-technical user seeing a governance settings failure, The two-category error table must use exact category names: 'render/runtime error' (boundary) and 'IPC/transport error' (in-page) — not 'frontend error' / 'backend error'

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: none
Blocks:     MGMT-UI-004 (wrap GovernanceSettings in the existing shared/ErrorBoundary — this design contract specifies the title prop and compact flag to pass); MGMT-UI-011 (IPC-failure danger banner + Retry — this design contract specifies the InfoMessage props and the persistent-failure safe read-only state)
```
</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "DESIGN-MGMT-008",
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
      "description": "GIVEN the error-boundary contract section of DESIGN-ANNOTATIONS.md covers the render/runtime error case WHEN a reviewer inspects the ErrorBoundary fallback content specification THEN it specifies: (1) the existing shared/ErrorBoundary.svelte (apps/desktop/src/components/shared/ErrorBoundary.svelte) is the boundary \u2014 no new component; (2) GovernanceSettings.svelte is wrapped in ErrorBoundary with title='Governance settings failed to load', compact=false; (3) the 'failed' snippet renders error.message as the sub-line if it is an Error instance; (4) no Retry button in the boundary fallback (the fallback is informational; recovery is reopening the settings modal); (5) the settings modal frame and other settings sections remain functional \u2014 the ErrorBoundary fallback renders only inside the Permissions & Governance section mount point",
      "verify": "design review \u2014 reviewer confirms all five items are present: component path, title prop, compact=false, no Retry in boundary fallback, modal survival"
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the contract covers the IPC/transport error case WHEN a reviewer inspects the IPC-failure InfoMessage specification THEN it specifies: InfoMessage (packages/ui/src/lib/components/InfoMessage.svelte) style='danger' outlined=true; title renders the structured denial message if {code, message} is available from the SDK response, or 'Connection lost \u2014 governance service unavailable' if no structured response; a Retry Button (packages/ui/src/lib/components/Button.svelte) is the primaryLabel/primaryAction slot of the InfoMessage; the Retry button re-issues the same SDK call that failed; the banner appears in the GovernanceSettings banner slot (same slot as the denial banner from DESIGN-MGMT-004), taking highest priority",
      "verify": "design review \u2014 reviewer confirms InfoMessage path + style='danger' + outlined=true + two-case title text + Retry Button path + primaryAction slot + banner-slot placement"
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the contract specifies the persistent IPC failure state WHEN a reviewer reads the persistent-failure section THEN it states: (1) on Retry success \u2014 the danger banner hides, the governance surface resumes normal state; (2) on Retry failure \u2014 the danger InfoMessage stays visible with an updated or unchanged message; (3) on persistent failure \u2014 the governance surface stays in a safe read-only state (isReadOnly=true equivalent: all controls inert, no writes attempted); (4) the surface does NOT unmount or trigger the ErrorBoundary on an IPC error \u2014 the IPC error is handled in-page via the banner, not via the Svelte boundary mechanism",
      "verify": "design review \u2014 reviewer confirms all four items: retry-success clears banner, retry-failure keeps banner, persistent-failure read-only, IPC error does not trigger boundary"
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the contract covers both error categories WHEN a reviewer reads the error-category distinction section THEN it explicitly names the two categories in a side-by-side table or clear callout: {render/runtime error: caught by Svelte boundary \u2192 ErrorBoundary 'failed' snippet \u2192 title+message only, no Retry, modal survives} and {IPC/transport error: SDK call rejects \u2192 in-page danger InfoMessage + Retry \u2192 persistent failure \u2192 safe read-only, boundary NOT triggered}",
      "verify": "design review \u2014 reviewer confirms the two-category table or callout is present with the correct treatment per category and the no-boundary-for-IPC-error rule stated"
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the error-boundary + IPC-failure section extends DESIGN-ANNOTATIONS.md WHEN a reviewer audits every visual reference in the section THEN every visual attribute uses an existing CSS variable or defers to the component's own stylesheet \u2014 no hex literals, no new var(--error-*)/var(--mgmt-*) tokens",
      "verify": "grep the error section of DESIGN-ANNOTATIONS.md for '#' color literals and var(--error-)/var(--mgmt-) -> both return zero matches"
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "contract names shared/ErrorBoundary.svelte path, title='Governance settings failed to load', compact=false, no Retry in boundary fallback, modal-survival rule, error.message sub-line \u2014 all five items present",
      "verify": "design review of the ErrorBoundary fallback content section in DESIGN-ANNOTATIONS.md",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "contract names InfoMessage style='danger' outlined=true, two-case title text (structured vs. 'Connection lost'), Retry Button as primaryAction, banner-slot placement",
      "verify": "design review of the IPC-failure InfoMessage section in DESIGN-ANNOTATIONS.md",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "contract names all four persistent-failure items: retry-success clears banner, retry-failure keeps banner, persistent-failure = read-only, IPC error does not trigger ErrorBoundary",
      "verify": "design review of the persistent-failure section in DESIGN-ANNOTATIONS.md",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "contract contains a two-category table or callout distinguishing render/runtime error (boundary) from IPC/transport error (in-page banner); the no-boundary-for-IPC-error rule is explicitly stated",
      "verify": "design review of the error-category distinction section in DESIGN-ANNOTATIONS.md",
      "maps_to_ac": "AC-4"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "zero hex color literals and zero new var(--error-*)/var(--mgmt-*) tokens in the error section",
      "verify": "grep -nE '#[0-9a-fA-F]{3,6}|var\\(--(error|mgmt)-' apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md -> zero matches",
      "maps_to_ac": "AC-5"
    }
  ]
}
-->
