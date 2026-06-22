# DESIGN-MGMT-004: Structured-denial banner + self-escalation no-flip contract

> **Supersedes the original AC-2/AC-3 per `red-hat-20260622T145305Z.md` (H4, H5, U1, L8). AC-1 and AC-4 are preserved unchanged.**
>
> The original AC-2 encoded an unprovable transition (an administrator granting `administration:write` to themselves and receiving a denial) that cannot fire in production — an administrator already holds the permission per CAP-AUTHZ-01, so the grant stages as pending, never as a rejection (red-hat U1/H4). AC-2 now uses a **non-admin self-escalation** actor — the honest no-bypass proof that `perm.denied` fires. AC-3 is rewritten as the symmetric **admin self-revoke** direction (an administrator attempting to remove `administration:write` from themselves), closing the H5 asymmetry (grant-only no-flip). The three-state machine table is preserved in the DESIGN/CODE PATTERN section below with its denial row updated to name both entry paths.

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
A sveltekit-implementer reading this contract knows (a) exactly which InfoMessage props to use for the denial banner, (b) the synchronous toggle-revert rule for BOTH the grant direction (non-admin self-grant) and the symmetric revoke direction (admin self-revoke), (c) when the danger banner appears vs when a chipToast is used, and (d) the full three-state machine — without any design judgment at implementation time.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST specify InfoMessage style='danger' outlined=true with title 'perm.denied — you cannot modify your own administration grants.' (verbatim from the Cross-cutting states wireframe) and a remediation_hint sub-line — citing packages/ui/src/lib/components/InfoMessage.svelte.
- [MUST] MUST specify the toggle no-flip transition rule for the GRANT direction using a NON-ADMIN actor: a principal LACKING administration:write toggles administration:write ON for themselves (attempting self-escalation) — the optimistic toggle change is REVERTED when a perm.denied response is received — the control returns to its prior state (unchecked); the denial InfoMessage appears; the effective permission is NOT updated. An administrator already holds administration:write (CAP-AUTHZ-01) and cannot receive this rejection on a self-escalation attempt — only a principal lacking the permission exercises this path honestly.
- [MUST] MUST specify the SYMMETRIC toggle no-flip transition rule for the REVOKE direction: an admin who holds administration:write toggles administration:write OFF for themselves (attempting self-revoke) — the Toggle is SYNCHRONOUSLY reverted to checked (stays ON, no optimistic flip); the denial InfoMessage appears; the admin retains the permission. The self-modification block fires symmetrically on grant AND revoke.
- [MUST] MUST define the three-state state machine: {idle, self-modification→denied, transient-error} → {control state, banner type, banner text} as a state table. The denial row names BOTH entry paths: (a) a principal LACKING administration:write self-grants OR (b) an admin self-revokes.
- [MUST] MUST distinguish the denial banner (danger InfoMessage, permanent until dismissed or next attempt) from the transient chipToast (for non-self-escalation write errors) — these are two separate feedback paths per UC-MGMT-06 AC-8.
- [MUST] MUST confirm that read-only disabled controls (DESIGN-MGMT-003) and the pending pending-state contract (DESIGN-MGMT-002) apply in the surrounding surface; this task adds ONLY the denial banner and no-flip rule on top — not a re-spec of those layers.
- [NEVER] NEVER introduce a new design-system token, CSS variable, or hex color not already in packages/ui.
- [NEVER] NEVER describe the toggle as staying flipped (optimistic) and reverting asynchronously — the revert is synchronous on receiving perm.denied from the governed path.
- [NEVER] NEVER use 'admin' as the self-GRANT actor in any denial row — an administrator already holds administration:write and cannot receive perm.denied on a self-escalation attempt (U1 unprovable wording). The grant-direction denial uses a principal LACKING administration:write as the actor.
- [NEVER] NEVER re-specify the pending-state badge (DESIGN-MGMT-002) or the read-only info banner (DESIGN-MGMT-003) — only extend DESIGN-ANNOTATIONS.md with a new denial-and-no-flip section.
- [NEVER] NEVER use InfoMessage style='warning' or style='info' for the self-escalation denial — it is strictly style='danger'.
- [NEVER] NEVER annotate SDK call structure, store internals, or Tauri command shape — design the visual and interaction layer only.
- [NEVER] NEVER offer a Retry button on the denial banner — the denial is structural (self-modification block), not a transient error.
- [STRICTLY] STRICTLY reuse packages/ui/src/lib/components/InfoMessage.svelte (danger variant) and packages/ui/src/lib/components/Toggle.svelte — no new components.
- [STRICTLY] STRICTLY no new design-system work: the denial banner is an existing InfoMessage danger; the no-flip is an interaction rule, not a new animation or spinner.
- [STRICTLY] STRICTLY banner text verbatim: 'perm.denied — you cannot modify your own administration grants.' — do not paraphrase.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: Denial banner prop specification [PRIMARY]
- [ ] AC-2: Toggle no-flip interaction rule (non-admin self-grant direction)
- [ ] AC-3: Symmetric self-revoke no-flip (admin self-revoke direction)
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

AC-2: Toggle no-flip interaction rule (non-admin self-grant direction)
  GIVEN: the contract defines the toggle no-flip rule for the grant direction using a non-admin actor
  WHEN:  a reviewer inspects the self-escalation interaction rule for the grant direction
  THEN:  it states: (1) a principal LACKING administration:write toggles administration:write ON for themselves (attempting self-escalation) — the Toggle moves to checked in local state; (2) the governed SDK call (perm_grant) returns perm.denied {code, message, remediation_hint} — the honest rejection that only a principal lacking the permission can receive on a self-escalation attempt (an administrator already holds administration:write per CAP-AUTHZ-01 and would stage as pending, never rejected); (3) the Toggle is SYNCHRONOUSLY reverted to unchecked (prior state — no optimistic flip); (4) the danger InfoMessage appears in the banner slot; the effective permission is NOT updated; the pending count does NOT increment (the rejected write staged nothing); no chipToast is emitted for this error path (the banner IS the feedback); no Retry button (the denial is structural, not transient)
  TEST_TIER: component   VERIFICATION_SERVICE: apps/desktop CT harness (pnpm test:ct:desktop) — downstream T-MGMT-030 component test asserts the Toggle is not flipped after a denied self-grant by a non-admin actor
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test (pnpm test:ct:desktop) exercising T-MGMT-030
  VERIFY: design review — reviewer confirms the four-step sequence uses a principal LACKING administration:write as the self-grant actor (not 'admin'), the synchronous-revert rule is explicit, the no-chipToast rule for self-escalation is stated, and the no-Retry rule is stated

AC-3: Symmetric self-revoke no-flip (admin self-revoke direction)
  GIVEN: the contract defines the symmetric toggle no-flip rule for the revoke direction
  WHEN:  a reviewer inspects the self-revoke interaction rule
  THEN:  it states: (1) an admin who holds administration:write toggles administration:write OFF for themselves (attempting self-revoke) — the Toggle moves to unchecked in local state; (2) the governed SDK call (perm_revoke) returns perm.denied {code, message, remediation_hint} — the symmetric self-modification block fires on revoke; (3) the Toggle is SYNCHRONOUSLY reverted to checked (prior state — stays ON, no optimistic flip; the admin retains the permission); (4) the danger InfoMessage appears in the banner slot with verbatim text 'perm.denied — you cannot modify your own administration grants.'; the effective permission is NOT updated; no chipToast; no Retry button. The three-state machine's denial row names BOTH entry paths: (a) a principal LACKING administration:write self-grants OR (b) an admin self-revokes
  TEST_TIER: component   VERIFICATION_SERVICE: apps/desktop CT harness (pnpm test:ct:desktop) — forward-looking revoke-direction component test (MGMT-UI-011 AC-4b, to be added per red-hat H5); the grant-direction proof is MGMT-UI-011 AC-4 (exists)
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test (pnpm test:ct:desktop); the revoke-direction downstream test does not exist yet (H5 honest gap — to be added per red-hat Recommendation #7)
  VERIFY: design review — reviewer confirms the four-step self-revoke sequence is present, the Toggle stays ON (aria-checked='true' unchanged), the denial row names both entry paths (grant + revoke), and the honest_gap_note recording the missing revoke-direction downstream test is present

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
- TC-2 (-> AC-2): contract states the four-step no-flip sequence using a principal LACKING administration:write as the self-escalation actor: toggle moves to checked, SDK call returns perm.denied, Toggle reverts synchronously, danger InfoMessage appears; no chipToast for self-escalation denial; no Retry button; zero state-machine rows use 'admin' as the self-grant actor
    VERIFY: design review of the toggle no-flip interaction section in DESIGN-ANNOTATIONS.md + grep -ciE 'admin (toggles|self-grants) administration:write' -> zero matches
- TC-3 (-> AC-3): contract states the symmetric four-step self-revoke sequence: admin toggles administration:write OFF for themselves, SDK call returns perm.denied, Toggle stays ON (aria-checked='true' unchanged), danger InfoMessage appears; the three-state machine's denial row names both entry paths (grant + revoke)
    VERIFY: design review of the self-revoke interaction section in DESIGN-ANNOTATIONS.md
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
6. .spec/reviews/red-hat-20260622T145305Z.md — H4 (AC-2 inherits unprovable admin-self-grant wording), H5 (no-flip asymmetric — grant only), U1 (SPRINT.md step-4 'admin self-grant' unprovable), L8 (design contracts have no executable verification path — add verified_by pointers)

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- design review of apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md denial section   -> InfoMessage style='danger' outlined=true with verbatim title, remediation_hint sub-line, no primaryLabel — all present
- design review of the toggle no-flip interaction section (grant direction)   -> four-step sequence present using a principal LACKING administration:write as the self-grant actor; synchronous-revert rule explicit; no-chipToast-for-self-escalation rule stated; no Retry button
- design review of the self-revoke interaction section (revoke direction)   -> four-step self-revoke sequence present; Toggle stays ON (aria-checked='true' unchanged); denial row names both entry paths (grant + revoke); honest_gap_note for the missing revoke-direction downstream test is present
- grep -nE '#[0-9a-fA-F]{3,6}|var\(--(denial|mgmt)-' apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md   -> zero matches
- grep -ciE 'admin (toggles|self-grants) administration:write' apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md   -> zero matches (U1 wording eliminated)
- pnpm test:ct:desktop -- GovernanceDenialBanner (exercised by MGMT-UI-011 implementation)   -> T-MGMT-030 (self-escalation not applied) and T-MGMT-031 (structured denial via danger InfoMessage) pass

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references:
  - .spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md — Cross-cutting states wireframe block (lines 112-120): 'perm.denied — you cannot modify your own administration grants.' InfoMessage danger
  - .spec/prds/governance/tasks/sprint-06a-governance-ui-scaffold-principals-groups/DESIGN-MGMT-002-pending-state-contract.md — pending-state banner this task's denial banner replaces in the banner slot
  - .spec/prds/governance/tasks/sprint-06a-governance-ui-scaffold-principals-groups/DESIGN-MGMT-003-read-only-state.md — read-only disabled-control contract that applies simultaneously to the surrounding surface
notes:
  - GRANT-direction state transition (non-admin self-escalation): (1) a principal LACKING administration:write clicks the administration:write Toggle ON for their own principal row → Toggle moves to checked in local state; (2) governed SDK call (perm_grant) issued (same path as any other grant); (3) response is perm.denied {code, message, remediation_hint} — the honest rejection that only a principal lacking the permission can receive on a self-escalation attempt (an administrator already holds administration:write per CAP-AUTHZ-01 and would stage as pending, never rejected); (4) Toggle is SYNCHRONOUSLY reverted to unchecked (no delay, no animation); (5) danger InfoMessage appears in the GovernanceSettings banner slot replacing/taking precedence over the pending warning banner; effective permission NOT updated; pending count does NOT increment
  - REVOKE-direction state transition (admin self-revoke, symmetric): (1) an admin who holds administration:write clicks the administration:write Toggle OFF for their own principal row → Toggle moves to unchecked in local state; (2) governed SDK call (perm_revoke) issued; (3) response is perm.denied {code, message, remediation_hint} — the symmetric self-modification block fires on revoke; (4) Toggle is SYNCHRONOUSLY reverted to checked (stays ON — the admin retains the permission); (5) danger InfoMessage appears in the banner slot; effective permission NOT updated
  - three-state machine table (control state | banner type | banner text):
    | state | control state | banner type | banner text |
    |-------|--------------|-------------|-------------|
    | idle | Toggle at current value, normal surface | none | — |
    | self-modification→denied | Toggle reverted to prior state (grant: unchecked; revoke: checked) | InfoMessage danger | 'perm.denied — you cannot modify your own administration grants.' + remediation_hint sub-line. Entry paths: (a) a principal LACKING administration:write self-grants OR (b) an admin self-revokes |
    | transient-error (non-self-escalation write error) | Toggle reverted to prior state | chipToast only | transient error text — NO danger InfoMessage banner |
  - banner slot priority: danger (denial) > warning (pending) > info (read-only) — only one banner visible at a time; denial takes highest priority
  - chipToast vs InfoMessage: transient write errors on OTHER operations (non-self-escalation) emit a chipToast and revert their control; self-escalation denial (grant OR revoke) ONLY uses the persistent danger InfoMessage in the banner slot — no chipToast
  - the denial banner persists until the user navigates away or makes a new attempt; it is not auto-dismissed; no Retry button (structural denial, not transient)
  - the structured denial payload shape is {code: 'perm.denied', message: string, remediation_hint: string} — the design contract must account for remediation_hint being non-empty (show it as a sub-line below the title)
  - honest_gap_note: the revoke-direction downstream component test (MGMT-UI-011 AC-4b) does not exist yet — only the grant-direction proof (MGMT-UI-011 AC-4) exists. The revoke-direction test is to be added per red-hat H5 / Recommendation #7. This contract records the gap honestly rather than claiming the revoke direction is already proven.
pattern: GovernanceSettings banner slot is a single-banner-at-a-time region; the active banner is determined by the highest-priority active state: denial (danger InfoMessage) > pending (warning InfoMessage) > read-only (info InfoMessage) > nothing. The denial state is entered synchronously on receiving perm.denied; it does not require a separate network round-trip or store update — it is the direct result of the SDK call rejection. The no-flip rule applies symmetrically: a non-admin self-grant reverts to unchecked; an admin self-revoke reverts to checked (stays ON).
pattern_source: .spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md Cross-cutting states wireframe + UC-MGMT-06 AC-7/8. packages/ui InfoMessage danger variant (the same component used for auth errors elsewhere in the GitButler desktop app).
anti_pattern: optimistically applying the administration:write grant and showing an error toast afterward (this would flip the toggle, violating the no-flip invariant); using 'admin' as the self-GRANT actor in any denial row (U1 — an administrator cannot receive perm.denied on a self-escalation attempt); treating self-revoke as a successful operation with no denial (H5 — the toggle flips OFF and the admin silently loses administration:write); stacking multiple InfoMessage banners simultaneously; using InfoMessage style='warning' for denial (warning is for pending, danger is for denial); offering a Retry button on the denial banner (structural denial, not transient)

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
Blocks:     MGMT-UI-011 (a11y + IPC-failure danger banner + Retry — this design contract is the primary design source for the denial/no-flip implementation), MGMT-UI-011 AC-4b / E2E-MGMT-UI-001 AC-4c (forward-looking revoke-direction downstream proofs — to be added per red-hat H5 / Recommendation #7)
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "DESIGN-MGMT-004",
  "proposed_by": "frontend-designer",
  "supersedes_ac23_per": "red-hat-20260622T145305Z.md (H4, H5, U1, L8)",
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
      "verified_by": [],
      "description": "GIVEN the denial-and-no-flip contract section exists in DESIGN-ANNOTATIONS.md WHEN a reviewer inspects the denial banner specification THEN it specifies InfoMessage (packages/ui/src/lib/components/InfoMessage.svelte) style='danger' outlined=true, title='perm.denied — you cannot modify your own administration grants.', a sub-line rendering remediation_hint from the structured denial payload, no primaryLabel action button (the banner is informational, not actionable), rendered in the GovernanceSettings banner slot replacing the pending warning banner while denial is active",
      "verify": "design review — reviewer confirms InfoMessage path + style='danger' + outlined=true + verbatim title text + remediation_hint sub-line + no action button + placement in banner slot"
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "primary": false,
      "verified_by": [],
      "description": "GIVEN the contract defines the toggle no-flip rule for the grant direction using a non-admin actor WHEN a reviewer inspects the self-escalation interaction rule THEN it states: (1) a principal LACKING administration:write toggles administration:write ON for themselves (attempting self-escalation) — the Toggle moves to checked in local state; (2) the governed SDK call (perm_grant) returns perm.denied — the honest rejection that only a principal lacking the permission can receive on a self-escalation attempt (an administrator already holds administration:write per CAP-AUTHZ-01); (3) the Toggle is SYNCHRONOUSLY reverted to unchecked (prior state — no optimistic flip); (4) the danger InfoMessage appears in the banner slot; the effective permission is NOT updated; the pending count does NOT increment; no chipToast; no Retry button",
      "verify": "design review — reviewer confirms the four-step sequence uses a principal LACKING administration:write as the self-grant actor (not 'admin'), the synchronous-revert rule is explicit, the no-chipToast and no-Retry rules are stated"
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "verified_by": [],
      "honest_gap_note": "The revoke-direction downstream component test (MGMT-UI-011 AC-4b) does not exist yet — only the grant-direction proof (MGMT-UI-011 AC-4) exists. The revoke-direction test is to be added per red-hat H5 / Recommendation #7.",
      "description": "GIVEN the contract defines the symmetric toggle no-flip rule for the revoke direction WHEN a reviewer inspects the self-revoke interaction rule THEN it states: (1) an admin who holds administration:write toggles administration:write OFF for themselves (attempting self-revoke) — the Toggle moves to unchecked in local state; (2) the governed SDK call (perm_revoke) returns perm.denied — the symmetric self-modification block fires on revoke; (3) the Toggle is SYNCHRONOUSLY reverted to checked (stays ON — no optimistic flip; the admin retains the permission); (4) the danger InfoMessage appears in the banner slot; the effective permission is NOT updated; no chipToast; no Retry button. The three-state machine's denial row names BOTH entry paths: (a) a principal LACKING administration:write self-grants OR (b) an admin self-revokes",
      "verify": "design review — reviewer confirms the self-revoke sequence is present, the Toggle stays ON (aria-checked='true' unchanged), the denial row names both entry paths, and the honest_gap_note recording the missing revoke-direction downstream test is present"
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "verified_by": [],
      "description": "GIVEN the denial section extends DESIGN-ANNOTATIONS.md WHEN a reviewer audits every color/spacing/typography reference in the denial section THEN every visual attribute uses an existing CSS variable or defers to the component's own stylesheet — no hex literals, no new var(--denial-*)/var(--mgmt-*) tokens",
      "verify": "grep the denial section of DESIGN-ANNOTATIONS.md for '#' color literals and var(--denial-)/var(--mgmt-) -> both return zero matches"
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "contract names InfoMessage style='danger' outlined=true with verbatim title 'perm.denied — you cannot modify your own administration grants.' and remediation_hint sub-line; no action button; placed in the GovernanceSettings banner slot",
      "verify": "design review of the denial banner section in DESIGN-ANNOTATIONS.md",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "contract states the four-step no-flip sequence using a principal LACKING administration:write as the self-escalation actor: toggle moves to checked, SDK call returns perm.denied, Toggle reverts synchronously, danger InfoMessage appears; no chipToast for self-escalation denial; no Retry button; zero state-machine rows use 'admin' as the self-grant actor",
      "verify": "design review of the toggle no-flip interaction section in DESIGN-ANNOTATIONS.md",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "contract states the symmetric self-revoke sequence: admin toggles administration:write OFF for themselves, SDK call returns perm.denied, Toggle stays ON (aria-checked='true' unchanged), danger InfoMessage appears; the three-state machine's denial row names both entry paths (grant + revoke)",
      "verify": "design review of the self-revoke interaction section in DESIGN-ANNOTATIONS.md",
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
