# DESIGN-MGMT-004: Structured-denial banner + self-escalation no-flip contract

## What this does

Define a prop-level visual contract and a state-transition rule for the self-escalation denial scenario: the danger InfoMessage that surfaces a structured denial ({code, message, remediation_hint}), and the interaction contract that a self-escalation attempt does NOT flip the Toggle — the control reverts, the banner appears. Builds on DESIGN-MGMT-002 (pending) and DESIGN-MGMT-003 (read-only); adds only the denial + no-flip layer.

## Why

Sprint 06b · PRD UC-MGMT-06 · capability —. A sveltekit-implementer reading this contract knows (a) exactly which InfoMessage props to use for the denial banner, (b) the synchronous toggle-revert rule, (c) when the danger banner appears vs when a chipToast is used, and (d) the full t

## How to verify

PRIMARY **AC-1** — `design review — reviewer confirms InfoMessage path + style='danger' + outlined=true + verbatim title text + remediation_hint sub-line + no action button + placement in banner slot`: Denial banner prop specification [PRIMARY]. Full gate set in the spec below.

## Scope

  - apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md (MODIFY — extend with denial-and-no-flip section)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: DESIGN-MGMT-004 — Structured-denial banner + self-escalation no-flip contract
================================================================================

TASK_TYPE:   DESIGN
STATUS:      Backlog
PRIORITY:    P0
EFFORT:      XS  (30 min)
AGENT:       frontend-designer
PROPOSED-BY: frontend-designer
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-MGMT-06
CAPABILITIES:—

RUNTIME_COMMANDS:
  test:  pnpm test:ct:desktop -- GovernanceDenialBanner (exercised by MGMT-UI-011 implementation)

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
A sveltekit-implementer reading this contract knows (a) exactly which InfoMessage props to use for the denial banner, (b) the synchronous toggle-revert rule, (c) when the danger banner appears vs when a chipToast is used, and (d) the full three-state machine — without any design judgment at implementation time.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST specify InfoMessage style='danger' outlined=true with title 'perm.denied — you cannot modify your own administration grants.' (verbatim from the Cross-cutting states wireframe) and a remediation_hint sub-line — citing packages/ui/src/lib/components/InfoMessage.svelte.
- [MUST] MUST specify the toggle no-flip transition rule: the optimistic toggle change is REVERTED when a perm.denied response is received — the control returns to its prior state (unchecked); the denial InfoMessage appears; the effective permission is NOT updated.
- [MUST] MUST define the three-state state machine: {idle, attempting-self-grant→denied, transient-error} → {control state, banner type, banner text} as a state table.
- [MUST] MUST distinguish the denial banner (danger InfoMessage, permanent until dismissed or next attempt) from the transient chipToast (for non-self-escalation write errors) — these are two separate feedback paths per UC-MGMT-06 AC-8.
- [MUST] MUST confirm that read-only disabled controls (DESIGN-MGMT-003) and the pending pending-state contract (DESIGN-MGMT-002) apply in the surrounding surface; this task adds ONLY the denial banner and no-flip rule on top — not a re-spec of those layers.
- [NEVER] NEVER introduce a new design-system token, CSS variable, or hex color not already in packages/ui.
- [NEVER] NEVER describe the toggle as staying flipped (optimistic) and reverting asynchronously — the revert is synchronous on receiving perm.denied from the governed path.
- [NEVER] NEVER re-specify the pending-state badge (DESIGN-MGMT-002) or the read-only info banner (DESIGN-MGMT-003) — only extend DESIGN-ANNOTATIONS.md with a new denial-and-no-flip section.
- [NEVER] NEVER use InfoMessage style='warning' or style='info' for the self-escalation denial — it is strictly style='danger'.
- [NEVER] NEVER annotate SDK call structure, store internals, or Tauri command shape — design the visual and interaction layer only.
- [STRICTLY] STRICTLY reuse packages/ui/src/lib/components/InfoMessage.svelte (danger variant) and packages/ui/src/lib/components/Toggle.svelte — no new components.
- [STRICTLY] STRICTLY no new design-system work: the denial banner is an existing InfoMessage danger; the no-flip is an interaction rule, not a new animation or spinner.
- [STRICTLY] STRICTLY banner text verbatim: 'perm.denied — you cannot modify your own administration grants.' — do not paraphrase.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [x] AC-1 [PRIMARY]: Denial banner prop specification [PRIMARY]
- [ ] AC-2: Toggle no-flip interaction rule
- [ ] AC-3: Three-state machine table
- [ ] AC-4: No new design-system tokens
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: Denial banner prop specification [PRIMARY]
  GIVEN: the denial-and-no-flip contract section exists in DESIGN-ANNOTATIONS.md
  WHEN:  a reviewer inspects the denial banner specification
  THEN:  it specifies InfoMessage (packages/ui/src/lib/components/InfoMessage.svelte) style='danger' outlined=true, title='perm.denied — you cannot modify your own administration grants.', a sub-line rendering remediation_hint from the structured denial payload, no primaryLabel action button (the banner is informational, not actionable), rendered in the GovernanceSettings banner slot replacing the pending warning banner while denial is active
  TEST_TIER: component   VERIFICATION_SERVICE: apps/desktop CT harness (pnpm test:ct:desktop) — downstream T-MGMT-031 component test asserts the danger InfoMessage renders with the structured-denial payload on a denied write
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test (pnpm test:ct:desktop) exercising T-MGMT-030/031
  VERIFY: design review — reviewer confirms InfoMessage path + style='danger' + outlined=true + verbatim title text + remediation_hint sub-line + no action button + placement in banner slot

AC-2: Toggle no-flip interaction rule
  GIVEN: the contract defines the toggle no-flip rule
  WHEN:  a reviewer inspects the self-escalation interaction rule
  THEN:  it states: (1) admin toggles administration:write ON for themselves — the Toggle moves to checked in local state; (2) the governed SDK call returns perm.denied; (3) the Toggle is SYNCHRONOUSLY reverted to unchecked (prior state); (4) the danger InfoMessage appears in the banner slot; the effective permission is NOT updated; no chipToast is emitted for this error path (the banner IS the feedback)
  TEST_TIER: component   VERIFICATION_SERVICE: apps/desktop CT harness (pnpm test:ct:desktop) — downstream T-MGMT-030 component test asserts the Toggle is not flipped after a denied self-grant
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test (pnpm test:ct:desktop) exercising T-MGMT-030
  VERIFY: design review — reviewer confirms the four-step sequence is present, the synchronous-revert rule is explicit, and the no-chipToast rule for self-escalation is stated

AC-3: Three-state machine table
  GIVEN: the contract defines a state machine for the denial surface
  WHEN:  a reviewer reads the state-machine table
  THEN:  it contains exactly three states as rows: {idle → no denial banner, Toggle at current value, normal surface}; {attempting-self-grant→denied → Toggle reverted to prior state, danger InfoMessage shows code+message+remediation_hint, pending banner replaced}; {transient-error (non-self-escalation write error) → chipToast emitted, Toggle reverted, NO danger InfoMessage banner} — each row states control state, banner type, and banner text
  TEST_TIER: component   VERIFICATION_SERVICE: apps/desktop CT harness (pnpm test:ct:desktop) — downstream T-MGMT-031 component test distinguishes danger InfoMessage from chipToast feedback paths
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test (pnpm test:ct:desktop)
  VERIFY: design review — reviewer confirms three-row state table is present with the correct banner-type distinction between denial (InfoMessage danger) and transient-error (chipToast only)

AC-4: No new design-system tokens
  GIVEN: the denial section extends DESIGN-ANNOTATIONS.md
  WHEN:  a reviewer audits every color/spacing/typography reference in the denial section
  THEN:  every visual attribute uses an existing CSS variable or defers to the component's own stylesheet — no hex literals, no new var(--denial-*)/var(--mgmt-*) tokens
  TEST_TIER: component   VERIFICATION_SERVICE: grep audit on the annotation file
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by static grep on the annotation file
  VERIFY: grep the denial section of DESIGN-ANNOTATIONS.md for '#' color literals and var(--denial-)/var(--mgmt-) -> both return zero matches

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): contract names InfoMessage style='danger' outlined=true with verbatim title 'perm.denied — you cannot modify your own administration grants.' and remediation_hint sub-line; no action button; placed in the GovernanceSettings banner slot
    VERIFY: design review of the denial banner section in DESIGN-ANNOTATIONS.md
- TC-2 (-> AC-2): contract states the four-step no-flip sequence: toggle moves to checked, SDK call returns perm.denied, Toggle reverts synchronously, danger InfoMessage appears; no chipToast for self-escalation denial
    VERIFY: design review of the toggle no-flip interaction section in DESIGN-ANNOTATIONS.md
- TC-3 (-> AC-3): contract contains a three-row state machine table distinguishing idle, attempting-self-grant→denied (InfoMessage danger), and transient-error (chipToast only) — each with control state, banner type, banner text
    VERIFY: design review of the state-machine table in DESIGN-ANNOTATIONS.md
- TC-4 (-> AC-4): zero hex color literals and zero new var(--denial-*)/var(--mgmt-*) tokens in the denial section
    VERIFY: grep -nE '#[0-9a-fA-F]{3,6}|var\(--(denial|mgmt)-' apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md -> zero matches

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md (MODIFY — extend with denial-and-no-flip section)
writeProhibited:
  - packages/ui/src/lib/components/** — no design-system changes
  - apps/desktop/src/components/shared/** — no shared component changes
  - any .svelte or .ts implementation file — design spec artifact only

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. .spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md [112-120] — Cross-cutting states wireframe: the exact verbatim danger InfoMessage text 'perm.denied — you cannot modify your own administration grants.' [PRIMARY PATTERN]
2. .spec/prds/governance/08-uc-mgmt.md [132-152] — UC-MGMT-06: self-escalation not optimistically applied, structured denial {code, message, remediation_hint}, chipToasts for transient errors
3. packages/ui/src/lib/components/InfoMessage.svelte [1-60] — style='danger', outlined, primaryLabel/primaryAction props — confirm danger variant renders without action button when primaryLabel omitted
4. .spec/prds/governance/tasks/sprint-06a-governance-ui-scaffold-principals-groups/DESIGN-MGMT-002-pending-state-contract.md [1-60] — Pending-state banner contract to build on — denial banner replaces (not stacks with) the pending warning banner
5. .spec/prds/governance/tasks/sprint-06a-governance-ui-scaffold-principals-groups/DESIGN-MGMT-003-read-only-state.md [1-60] — Read-only state contract to build on — denial is a third distinct overlay state; do not duplicate read-only spec

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- design review of apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md denial section   -> InfoMessage style='danger' outlined=true with verbatim title, remediation_hint sub-line, no primaryLabel — all present
- design review of the toggle no-flip interaction section   -> four-step sequence present; synchronous-revert rule explicit; no-chipToast-for-self-escalation rule stated
- design review of the state-machine table   -> three rows: idle / attempting-self-grant→denied / transient-error; banner-type column distinguishes InfoMessage danger from chipToast
- grep -nE '#[0-9a-fA-F]{3,6}|var\(--(denial|mgmt)-' apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md   -> zero matches
- pnpm test:ct:desktop -- GovernanceDenialBanner (exercised by MGMT-UI-011 implementation)   -> T-MGMT-030 (self-escalation not applied) and T-MGMT-031 (structured denial via danger InfoMessage) pass

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references:
  - .spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md — Cross-cutting states wireframe block (lines 112-120): 'perm.denied — you cannot modify your own administration grants.' InfoMessage danger
  - .spec/prds/governance/tasks/sprint-06a-governance-ui-scaffold-principals-groups/DESIGN-MGMT-002-pending-state-contract.md — pending-state banner this task's denial banner replaces in the banner slot
  - .spec/prds/governance/tasks/sprint-06a-governance-ui-scaffold-principals-groups/DESIGN-MGMT-003-read-only-state.md — read-only disabled-control contract that applies simultaneously to the surrounding surface
notes:
  - self-escalation state transition: (1) admin clicks Toggle ON for administration:write on their own principal row → Toggle moves to checked in local state; (2) governed SDK call issued (same path as any other grant); (3) response is perm.denied {code, message, remediation_hint}; (4) Toggle is SYNCHRONOUSLY reverted to unchecked (no delay, no animation); (5) danger InfoMessage appears in the GovernanceSettings banner slot replacing/taking precedence over the pending warning banner; effective permission NOT updated
  - banner slot priority: danger (denial) > warning (pending) > info (read-only) — only one banner visible at a time; denial takes highest priority
  - chipToast vs InfoMessage: transient write errors on OTHER operations (non-self-escalation) emit a chipToast and revert their control; self-escalation denial ONLY uses the persistent danger InfoMessage in the banner slot — no chipToast
  - the denial banner persists until the user navigates away or makes a new attempt; it is not auto-dismissed
  - the structured denial payload shape is {code: 'perm.denied', message: string, remediation_hint: string} — the design contract must account for remediation_hint being non-empty (show it as a sub-line below the title)
pattern: GovernanceSettings banner slot is a single-banner-at-a-time region; the active banner is determined by the highest-priority active state: denial (danger InfoMessage) > pending (warning InfoMessage) > read-only (info InfoMessage) > nothing. The denial state is entered synchronously on receiving perm.denied; it does not require a separate network round-trip or store update — it is the direct result of the SDK call rejection.
pattern_source: .spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md Cross-cutting states wireframe + UC-MGMT-06 AC-7/8. packages/ui InfoMessage danger variant (the same component used for auth errors elsewhere in the GitButler desktop app).
anti_pattern: optimistically applying the administration:write grant and showing an error toast afterward (this would flip the toggle, violating the no-flip invariant); stacking multiple InfoMessage banners simultaneously; using InfoMessage style='warning' for denial (warning is for pending, danger is for denial)

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agent: frontend-designer
rationale: frontend-designer owns the visual-state and interaction contract for cross-cutting overlay states; no Rust/Tauri or SDK knowledge required — the design contract is consumed by sveltekit-implementer (MGMT-UI-011).
coding_standards: All component citations must use exact source paths from the packages/ui Component-reuse table in 10-ui-infrastructure.md, State machine table must use three named states; no prose substitutes for the table, Banner text must be reproduced verbatim from the Cross-cutting states wireframe — no paraphrasing

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: DESIGN-MGMT-002 (pending-state contract — denial banner replaces the pending banner in the slot; must understand slot priority); DESIGN-MGMT-003 (read-only disabled-control contract — denial state applies on top of the same disabled controls)
Blocks:     MGMT-UI-011 (a11y + IPC-failure danger banner + Retry — this design contract is the primary design source for the denial/no-flip implementation)
```
</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "DESIGN-MGMT-004",
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
      "description": "GIVEN the denial-and-no-flip contract section exists in DESIGN-ANNOTATIONS.md WHEN a reviewer inspects the denial banner specification THEN it specifies InfoMessage (packages/ui/src/lib/components/InfoMessage.svelte) style='danger' outlined=true, title='perm.denied \u2014 you cannot modify your own administration grants.', a sub-line rendering remediation_hint from the structured denial payload, no primaryLabel action button (the banner is informational, not actionable), rendered in the GovernanceSettings banner slot replacing the pending warning banner while denial is active",
      "verify": "design review \u2014 reviewer confirms InfoMessage path + style='danger' + outlined=true + verbatim title text + remediation_hint sub-line + no action button + placement in banner slot"
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the contract defines the toggle no-flip rule WHEN a reviewer inspects the self-escalation interaction rule THEN it states: (1) admin toggles administration:write ON for themselves \u2014 the Toggle moves to checked in local state; (2) the governed SDK call returns perm.denied; (3) the Toggle is SYNCHRONOUSLY reverted to unchecked (prior state); (4) the danger InfoMessage appears in the banner slot; the effective permission is NOT updated; no chipToast is emitted for this error path (the banner IS the feedback)",
      "verify": "design review \u2014 reviewer confirms the four-step sequence is present, the synchronous-revert rule is explicit, and the no-chipToast rule for self-escalation is stated"
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the contract defines a state machine for the denial surface WHEN a reviewer reads the state-machine table THEN it contains exactly three states as rows: {idle \u2192 no denial banner, Toggle at current value, normal surface}; {attempting-self-grant\u2192denied \u2192 Toggle reverted to prior state, danger InfoMessage shows code+message+remediation_hint, pending banner replaced}; {transient-error (non-self-escalation write error) \u2192 chipToast emitted, Toggle reverted, NO danger InfoMessage banner} \u2014 each row states control state, banner type, and banner text",
      "verify": "design review \u2014 reviewer confirms three-row state table is present with the correct banner-type distinction between denial (InfoMessage danger) and transient-error (chipToast only)"
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the denial section extends DESIGN-ANNOTATIONS.md WHEN a reviewer audits every color/spacing/typography reference in the denial section THEN every visual attribute uses an existing CSS variable or defers to the component's own stylesheet \u2014 no hex literals, no new var(--denial-*)/var(--mgmt-*) tokens",
      "verify": "grep the denial section of DESIGN-ANNOTATIONS.md for '#' color literals and var(--denial-)/var(--mgmt-) -> both return zero matches"
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "contract names InfoMessage style='danger' outlined=true with verbatim title 'perm.denied \u2014 you cannot modify your own administration grants.' and remediation_hint sub-line; no action button; placed in the GovernanceSettings banner slot",
      "verify": "design review of the denial banner section in DESIGN-ANNOTATIONS.md",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "contract states the four-step no-flip sequence: toggle moves to checked, SDK call returns perm.denied, Toggle reverts synchronously, danger InfoMessage appears; no chipToast for self-escalation denial",
      "verify": "design review of the toggle no-flip interaction section in DESIGN-ANNOTATIONS.md",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "contract contains a three-row state machine table distinguishing idle, attempting-self-grant\u2192denied (InfoMessage danger), and transient-error (chipToast only) \u2014 each with control state, banner type, banner text",
      "verify": "design review of the state-machine table in DESIGN-ANNOTATIONS.md",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "zero hex color literals and zero new var(--denial-*)/var(--mgmt-*) tokens in the denial section",
      "verify": "grep -nE '#[0-9a-fA-F]{3,6}|var\\(--(denial|mgmt)-' apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md -> zero matches",
      "maps_to_ac": "AC-4"
    }
  ]
}
-->
