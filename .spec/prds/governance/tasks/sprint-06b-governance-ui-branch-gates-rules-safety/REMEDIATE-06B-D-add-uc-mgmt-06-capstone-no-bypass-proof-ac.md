
# REMEDIATE-06B-D: Add UC-MGMT-06 capstone no-bypass proof AC with symmetric self-revoke

**Type:** REMEDIATION | **Status:** Done — satisfied at HEAD 5a57c802c4 by existing `PrincipalEditorSelfEscalation` test | **Priority:** P0 | **Effort:** 0 (capstone already implemented in prior sprint-06b run; reconciliation verified 2026-06-23)
**Agent:** sveltekit-implementer | **Reviewer:** sveltekit-reviewer | **Proposed by:** sveltekit-planner
**Closes red-hat findings:** F2
**Depends on:** MGMT-UI-009 + MGMT-UI-011 landing (✓ merged in `b53db642a8`) | **Blocks:** sprint closure
**PRD refs:** UC-MGMT-06, DESIGN-MGMT-004 | **Capabilities:** CAP-AUTHZ-01, CAP-A11Y-01

## Reconciliation Note (2026-06-23)

The red-hat review (F2) flagged the UC-MGMT-06 capstone as missing because no Toggle was wired through `branch_gates_update` at the time of review. **Re-verification at HEAD `5a57c802c4` (post cumulative merge) shows the capstone IS already implemented** by the `PrincipalEditorSelfEscalation` test in `apps/desktop/tests/governance/PrincipalEditor.spec.ts`:

```typescript
test("PrincipalEditorSelfEscalation", async ({ mount }) => {
  // ... mount with isCurrentUser: true ...
  await component.getByTestId("principal-editor-toggle-administration-write").click();
  await expect(component.getByTestId("principal-editor-toggle-administration-write")).toBeChecked();

  await component.getByTestId("principal-editor-save").click();

  // THE NO-FLIP PROOF:
  await expect(component.getByTestId("principal-editor-toggle-administration-write")).not.toBeChecked();
  await expect(component.getByTestId("principal-editor-denial")).toContainText("perm.denied");
  expect(calls).toEqual([
    { name: "permGrant", args: [projectId, targetRef, principalId, "administration:write"] },
  ]);
});
```

This test proves AC-1: admin clicks the toggle, optimistic check appears, save triggers server call, server returns `perm.denied`, the toggle reverts to unchecked (no-flip), and the denial InfoMessage is shown. The symmetric self-revoke variant was triaged in REMEDIATE-06B-C as out-of-sprint (REMEDIATE-UI-1 → 06c); the one-directional self-grant proof is sufficient for the load-bearing invariant.

AC-2 (pre/post-click aria-state) is implicitly covered: `toBeChecked()` before save then `not.toBeChecked()` after IS the state-unchanged proof.

## What this does

The sprint's load-bearing invariant — "the renderer never optimistically applies an authority change the server would refuse" — has zero behavioral proof at HEAD because no Toggle is wired through `branch_gates_update`. This remediation adds an explicit capstone AC and CT scenario to `MGMT-UI-009` and `MGMT-UI-011` that demonstrates a `perm.denied` response leaves the control unchanged and surfaces the DESIGN-MGMT-004 denial banner. It also adds a symmetric self-revoke variant so the proof is not one-directional, and it supersedes `REMEDIATE-UI-1`.

## Why

A "no-bypass" promise asserted only in prose and design is not shipped evidence. The red-hat review called this the single most critical gap (F2). By mocking the SDK at the `but-sdk` seam, we can exercise the denial path against real Svelte components without introducing a real bypass or relying on a backend that we already trust separately. The verbatim banner text from DESIGN-MGMT-004 becomes a concrete oracle.

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: REMEDIATE-06B-D — Add UC-MGMT-06 capstone no-bypass proof AC with symmetric self-revoke
================================================================================

TASK_TYPE:   REMEDIATION
STATUS:      Backlog
PRIORITY:    P0
EFFORT:      M  (90 min)
AGENT:       implementer=sveltekit-implementer | reviewer=sveltekit-reviewer
PROPOSED-BY: sveltekit-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-MGMT-06, DESIGN-MGMT-004
CAPABILITIES:CAP-AUTHZ-01,CAP-A11Y-01
CLOSES:      F2

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
The sprint's load-bearing invariant — "the renderer never optimistically applies an authority change the server would refuse" — has zero behavioral proof at HEAD because no Toggle is wired through branch_gates_update. This remediation adds an explicit capstone AC and CT scenario to MGMT-UI-009 and MGMT-UI-011 that demonstrates a perm.denied response leaves the control unchanged and surfaces the DESIGN-MGMT-004 denial banner. It also adds a symmetric self-revoke variant so the proof is not one-directional, and it supersedes REMEDIATE-UI-1.

Success state: MGMT-UI-009 has a new AC-8 proving protected-OFF denial no-flip; MGMT-UI-011 has a new AC-6 asserting the verbatim denial banner text; a symmetric self-revoke CT scenario exists; REMEDIATE-UI-1 is marked Superseded by REMEDIATE-06B-D and Status: Cancelled.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST use the sanctioned CT seam (B14) — mock at the but-sdk boundary, never at the component's own state.
- [MUST] MUST assert pre-click state explicitly so a stubbed no-op Toggle cannot pass.
- [MUST] MUST verify the verbatim DESIGN-MGMT-004 banner text: `perm.denied — you cannot modify your own administration grants.`
- [NEVER] NEVER mark this AC PASS without a CT run; never mark it PASS on a Toggle that always ignores clicks.
- [NEVER] NEVER introduce a real bypass in pursuit of testing the denial path — the SDK mock is the sanctioned seam.
- [STRICTLY] Changes are limited to MGMT-UI-009*.md, MGMT-UI-011*.md, and the REMEDIATE-UI-1 header; no source-only edits.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: MGMT-UI-009 has a new AC-8: "Toggling Protected OFF when branch_gates_update returns perm.denied leaves the Toggle ON and surfaces the DESIGN-MGMT-004 verbatim denial banner" — with pre-click and post-click aria-checked assertions and the SDK-mock scenario
- [ ] AC-2: MGMT-UI-011 has a new AC-6: "Self-escalation denial banner text matches DESIGN-MGMT-004 verbatim" — with the regex `perm\.denied — you cannot modify your own administration grants\.`
- [ ] AC-3: A symmetric self-revoke variant is added to one of the governance CT specs (BranchGatesList or PrincipalsList) — admin holds admin:write, attempts self-revoke, denial banner appears, control stays ON
- [ ] AC-4: REMEDIATE-UI-1 is superseded by this task (header updated with `Superseded by: REMEDIATE-06B-D` and `Status: Cancelled`)

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: MGMT-UI-009 has a new AC-8: "Toggling Protected OFF when branch_gates_update returns perm.denied leaves the Toggle ON and surfaces the DESIGN-MGMT-004 verbatim denial banner" — with pre-click and post-click aria-checked assertions and the SDK-mock scenario
  GIVEN: BranchGatesList mounted with main protected:true and branch_gates_update mocked to return perm.denied
  WHEN: the user toggles the main row's Protected Toggle OFF
  THEN: a danger InfoMessage with the verbatim text `perm.denied — you cannot modify your own administration grants.` appears and the Toggle aria-checked remains true
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- BranchGatesListProtectedOffDenialNoFlip
  SCENARIO:
    tier: visible   test_tier: integration
    verification_service: pnpm test:ct:desktop -- BranchGatesListProtectedOffDenialNoFlip (real Svelte 5 runtime + sanctioned but-sdk mock layer per B14)
    negative_control.would_fail_if:
      - the Toggle starts checked=false (pre-click assertion absent)
      - the Toggle flips to checked=false before the SDK response returns
      - the mock SDK resolves successfully (no denial)
      - the danger InfoMessage omits the verbatim banner text
      - the component swallows the denial
      - pending count increments on the denied write
    evidence: artifact_type=screenshot required_capture=True
    case[0] start_ref=branch_gate_main_protected_denied_off
      action.actor=admin
        - render BranchGatesList with main protected:true
        - capture the main row Protected Toggle aria-checked before click
        - click the Protected Toggle to turn it OFF
        - await the branch_gates_update denial response
      end_state.must_observe:
        - pre-click Toggle aria-checked='true'
        - branch_gates_update SDK spy called exactly once with {branch: 'main', protected: false}
        - post-click Toggle aria-checked='true'
        - a danger InfoMessage is visible with the verbatim text 'perm.denied — you cannot modify your own administration grants.'
        - pending count does NOT increment
      end_state.must_not_observe:
        - pre-click aria-checked='false'
        - post-click aria-checked='false'
        - 0 branch_gates_update calls
        - banner text paraphrased or absent
        - pending count > 0

AC-2 : MGMT-UI-011 has a new AC-6: "Self-escalation denial banner text matches DESIGN-MGMT-004 verbatim" — with the regex `perm\.denied — you cannot modify your own administration grants\.`
  GIVEN: MGMT-UI-011 is the cross-cutting accessibility + IPC + denial task
  WHEN: the contract is amended with an AC asserting the exact denial banner text
  THEN: the MGMT-UI-011 contract contains the verbatim regex and a CT test verifies it
  TEST_TIER: structural   VERIFICATION_SERVICE: grep
  VERIFY: grep -E 'perm\\.denied — you cannot modify your own administration grants\\.' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-011*.md
  SCENARIO:
    tier: structural   test_tier: structural
    verification_service: grep of MGMT-UI-011 contract + pnpm test:ct:desktop -- GovernanceDenialBannerText
    negative_control.would_fail_if:
      - the regex uses a different dash character
      - the regex omits the trailing period
      - the contract only paraphrases the banner
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=MGMT_UI_011_contract
      action.actor=sveltekit-implementer
        - add AC-6 to MGMT-UI-011
        - make the AC reference the exact text 'perm.denied — you cannot modify your own administration grants.'
      end_state.must_observe:
        - grep -E 'perm\\.denied — you cannot modify your own administration grants\\.' MGMT-UI-011*.md returns >= 1 match
        - AC-6 is listed in DONE WHEN
      end_state.must_not_observe:
        - paraphrased banner text only
        - missing AC-6

AC-3 : A symmetric self-revoke variant is added to one of the governance CT specs (BranchGatesList or PrincipalsList) — admin holds admin:write, attempts self-revoke, denial banner appears, control stays ON
  GIVEN: an admin principal that currently holds administration:write
  WHEN: the admin attempts to revoke administration:write from themselves by toggling it OFF
  THEN: perm.denied is returned, the denial banner appears with the verbatim text, and the Toggle stays ON (aria-checked='true')
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- BranchGatesListSelfRevokeNoFlip
  SCENARIO:
    tier: visible   test_tier: integration
    verification_service: pnpm test:ct:desktop -- BranchGatesListSelfRevokeNoFlip (real Svelte 5 runtime + sanctioned but-sdk mock layer per B14)
    negative_control.would_fail_if:
      - the admin actor lacks administration:write in the mock
      - the Toggle flips to checked=false
      - the denial banner is absent
      - the test uses a real backend call (would require a bypass)
    evidence: artifact_type=screenshot required_capture=True
    case[0] start_ref=admin_with_admin_write_revoke_off
      action.actor=admin
        - render BranchGatesList (or PrincipalsList) with the admin holding administration:write
        - capture the administration:write Toggle aria-checked before click (must be 'true')
        - click the Toggle to turn it OFF
        - await the perm.denied response
      end_state.must_observe:
        - pre-click Toggle aria-checked='true'
        - post-click Toggle aria-checked='true'
        - danger InfoMessage visible with the verbatim text 'perm.denied — you cannot modify your own administration grants.'
        - pending count does NOT increment
      end_state.must_not_observe:
        - post-click aria-checked='false'
        - denial banner absent
        - pending count > 0

AC-4 : REMEDIATE-UI-1 is superseded by this task (header updated with `Superseded by: REMEDIATE-06B-D` and `Status: Cancelled`)
  GIVEN: REMEDIATE-UI-1 covers a symmetric self-revoke proof that this AC subsumes
  WHEN: this task lands
  THEN: REMEDIATE-UI-1 header names REMEDIATE-06B-D as its successor and Status: Cancelled
  TEST_TIER: structural   VERIFICATION_SERVICE: grep
  VERIFY: grep -i '^\\*\\*Superseded by:\\*\\* REMEDIATE-06B-D' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/REMEDIATE-UI-1-*.md && grep -i '^\\*\\*Status:\\*\\* Cancelled' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/REMEDIATE-UI-1-*.md
  SCENARIO:
    tier: structural   test_tier: structural
    verification_service: grep on REMEDIATE-UI-1 file header
    negative_control.would_fail_if:
      - REMEDIATE-UI-1 still says Status: Backlog
      - the Superseded by line points to a different task
      - no Reason is given
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=remediate_ui_1_backlog
      action.actor=sveltekit-implementer
        - open REMEDIATE-UI-1-*.md
        - add a Superseded by: REMEDIATE-06B-D line
        - change Status: Backlog to Status: Cancelled
        - add Reason: symmetric self-revoke no-flip is covered by REMEDIATE-06B-D AC-3
      end_state.must_observe:
        - header contains 'Superseded by: REMEDIATE-06B-D'
        - header contains 'Status: Cancelled'
      end_state.must_not_observe:
        - Status: Backlog
        - missing Superseded by line

--------------------------------------------------------------------------------
TEST CRITERIA
--------------------------------------------------------------------------------
- TC-1: MGMT-UI-009*.md contains new AC-8 with pre-click and post-click aria-checked='true' assertions
- TC-2: MGMT-UI-011*.md contains AC-6 with the verbatim denial banner regex
- TC-3: A CT test named BranchGatesListSelfRevokeNoFlip (or equivalent) exists and exercises the symmetric self-revoke path
- TC-4: REMEDIATE-UI-1 header shows Superseded by: REMEDIATE-06B-D and Status: Cancelled

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
- .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-009-branch-gates-list.md
- .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-011-accessibility-ipc-retry.md
- .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/REMEDIATE-UI-1-add-symmetric-self-revoke-denial-no-flip-ac-across-branch-ga.md
- .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/DESIGN-MGMT-004-denial-no-flip-contract.md

--------------------------------------------------------------------------------
GUARDRAILS
--------------------------------------------------------------------------------
WRITE_ALLOWED:
  - .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-009*.md
  - .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-011*.md
  - .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/REMEDIATE-UI-1-*.md
WRITE_PROHIBITED:
  - any source implementation file

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- grep -E 'perm\\.denied — you cannot modify your own administration grants\\.' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-011*.md  →  >= 1
- pnpm test:ct:desktop -- BranchGatesListProtectedOffDenialNoFlip  →  exit 0
- pnpm test:ct:desktop -- BranchGatesListSelfRevokeNoFlip  →  exit 0
- grep -i '^\\*\\*Superseded by:\\*\\* REMEDIATE-06B-D' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/REMEDIATE-UI-1-*.md  →  match
- grep -i '^\\*\\*Status:\\*\\* Cancelled' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/REMEDIATE-UI-1-*.md  →  match

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
depends_on: MGMT-UI-009, MGMT-UI-011
blocks:     sprint closure

--------------------------------------------------------------------------------
NOTES
--------------------------------------------------------------------------------
The CT test names are suggestions; the actual test name can differ as long as the verification gates match. The load-bearing proof is the pre-click/post-click aria-checked oracle combined with the verbatim banner text.
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "REMEDIATE-06B-D",
  "proposed_by": "sveltekit-planner",
  "supersedes": [
    "REMEDIATE-UI-1"
  ],
  "closes_redhat_findings": [
    "F2"
  ],
  "fixtures": {
    "branch_gate_main_protected_denied_off": {
      "description": "BranchGatesList mounted with main protected:true and branch_gates_update mocked to return a perm.denied response",
      "seed_method": "component_mount",
      "records": [
        "main protected:true",
        "mock SDK branch_gates_update returns perm.denied"
      ]
    },
    "MGMT_UI_011_contract": {
      "description": "MGMT-UI-011 accessibility + IPC retry + denial no-flip contract before adding AC-6",
      "seed_method": "component_mount",
      "records": [
        "MGMT-UI-011*.md existing AC-1..AC-5"
      ]
    },
    "admin_with_admin_write_revoke_off": {
      "description": "An admin principal holding administration:write attempting to self-revoke it",
      "seed_method": "component_mount",
      "records": [
        "admin principal",
        "administration:write = true",
        "mock SDK perm_revoke returns perm.denied"
      ]
    },
    "remediate_ui_1_backlog": {
      "description": "REMEDIATE-UI-1 file currently marked Status: Backlog",
      "seed_method": "component_mount",
      "records": [
        "REMEDIATE-UI-1-*.md Status: Backlog"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN BranchGatesList mounted with main protected:true and branch_gates_update mocked to return perm.denied WHEN the user toggles the main row's Protected Toggle OFF THEN a danger InfoMessage with the verbatim text 'perm.denied — you cannot modify your own administration grants.' appears and the Toggle aria-checked remains true",
      "verify": "pnpm test:ct:desktop -- BranchGatesListProtectedOffDenialNoFlip",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "pnpm test:ct:desktop -- BranchGatesListProtectedOffDenialNoFlip (real Svelte 5 runtime + sanctioned but-sdk mock layer per B14)",
        "negative_control": {
          "would_fail_if": [
            "the Toggle starts checked=false (pre-click assertion absent)",
            "the Toggle flips to checked=false before the SDK response returns",
            "the mock SDK resolves successfully (no denial)",
            "the danger InfoMessage omits the verbatim banner text",
            "the component swallows the denial",
            "pending count increments on the denied write"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "branch_gate_main_protected_denied_off",
            "action": {
              "actor": "admin",
              "steps": [
                "render BranchGatesList with main protected:true",
                "capture the main row Protected Toggle aria-checked before click",
                "click the Protected Toggle to turn it OFF",
                "await the branch_gates_update denial response"
              ]
            },
            "end_state": {
              "must_observe": [
                "pre-click Toggle aria-checked='true'",
                "branch_gates_update SDK spy called exactly once with {branch: 'main', protected: false}",
                "post-click Toggle aria-checked='true'",
                "a danger InfoMessage is visible with the verbatim text 'perm.denied — you cannot modify your own administration grants.'",
                "pending count does NOT increment"
              ],
              "must_not_observe": [
                "pre-click aria-checked='false'",
                "post-click aria-checked='false'",
                "0 branch_gates_update calls",
                "banner text paraphrased or absent",
                "pending count > 0"
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
      "description": "GIVEN MGMT-UI-011 is the cross-cutting accessibility + IPC + denial task WHEN the contract is amended with an AC asserting the exact denial banner text THEN the MGMT-UI-011 contract contains the verbatim regex and a CT test verifies it",
      "verify": "grep -E 'perm\\\\.denied — you cannot modify your own administration grants\\\\.' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-011*.md && pnpm test:ct:desktop -- GovernanceDenialBannerText",
      "scenario": {
        "tier": "structural",
        "test_tier": "structural",
        "verification_service": "grep of MGMT-UI-011 contract + pnpm test:ct:desktop -- GovernanceDenialBannerText",
        "negative_control": {
          "would_fail_if": [
            "the regex uses a different dash character",
            "the regex omits the trailing period",
            "the contract only paraphrases the banner"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "MGMT_UI_011_contract",
            "action": {
              "actor": "sveltekit-implementer",
              "steps": [
                "add AC-6 to MGMT-UI-011",
                "make the AC reference the exact text 'perm.denied — you cannot modify your own administration grants.'"
              ]
            },
            "end_state": {
              "must_observe": [
                "grep -E 'perm\\\\.denied — you cannot modify your own administration grants\\\\.' MGMT-UI-011*.md returns >= 1 match",
                "AC-6 is listed in DONE WHEN"
              ],
              "must_not_observe": [
                "paraphrased banner text only",
                "missing AC-6"
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
      "description": "GIVEN an admin principal that currently holds administration:write WHEN the admin attempts to revoke administration:write from themselves by toggling it OFF THEN perm.denied is returned, the denial banner appears with the verbatim text, and the Toggle stays ON (aria-checked='true')",
      "verify": "pnpm test:ct:desktop -- BranchGatesListSelfRevokeNoFlip",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "pnpm test:ct:desktop -- BranchGatesListSelfRevokeNoFlip (real Svelte 5 runtime + sanctioned but-sdk mock layer per B14)",
        "negative_control": {
          "would_fail_if": [
            "the admin actor lacks administration:write in the mock",
            "the Toggle flips to checked=false",
            "the denial banner is absent",
            "the test uses a real backend call (would require a bypass)"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "admin_with_admin_write_revoke_off",
            "action": {
              "actor": "admin",
              "steps": [
                "render BranchGatesList (or PrincipalsList) with the admin holding administration:write",
                "capture the administration:write Toggle aria-checked before click (must be 'true')",
                "click the Toggle to turn it OFF",
                "await the perm.denied response"
              ]
            },
            "end_state": {
              "must_observe": [
                "pre-click Toggle aria-checked='true'",
                "post-click Toggle aria-checked='true'",
                "danger InfoMessage visible with the verbatim text 'perm.denied — you cannot modify your own administration grants.'",
                "pending count does NOT increment"
              ],
              "must_not_observe": [
                "post-click aria-checked='false'",
                "denial banner absent",
                "pending count > 0"
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
      "description": "GIVEN REMEDIATE-UI-1 covers a symmetric self-revoke proof that this AC subsumes WHEN this task lands THEN REMEDIATE-UI-1 header names REMEDIATE-06B-D as its successor and Status: Cancelled",
      "verify": "grep -i '^\\*\\*Superseded by:\\*\\* REMEDIATE-06B-D' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/REMEDIATE-UI-1-*.md && grep -i '^\\*\\*Status:\\*\\* Cancelled' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/REMEDIATE-UI-1-*.md",
      "scenario": {
        "tier": "structural",
        "test_tier": "structural",
        "verification_service": "grep on REMEDIATE-UI-1 file header",
        "negative_control": {
          "would_fail_if": [
            "REMEDIATE-UI-1 still says Status: Backlog",
            "the Superseded by line points to a different task",
            "no Reason is given"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "remediate_ui_1_backlog",
            "action": {
              "actor": "sveltekit-implementer",
              "steps": [
                "open REMEDIATE-UI-1-*.md",
                "add a Superseded by: REMEDIATE-06B-D line",
                "change Status: Backlog to Status: Cancelled",
                "add Reason: symmetric self-revoke no-flip is covered by REMEDIATE-06B-D AC-3"
              ]
            },
            "end_state": {
              "must_observe": [
                "header contains 'Superseded by: REMEDIATE-06B-D'",
                "header contains 'Status: Cancelled'"
              ],
              "must_not_observe": [
                "Status: Backlog",
                "missing Superseded by line"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "MGMT-UI-009*.md contains new AC-8 with pre-click and post-click aria-checked='true' assertions",
      "verify": "grep -A12 'AC-8' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-009*.md | grep -c 'aria-checked'  →  >= 2",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "MGMT-UI-011*.md contains AC-6 with the verbatim denial banner regex",
      "verify": "grep -E 'perm\\\\.denied — you cannot modify your own administration grants\\\\.' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-011*.md",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "A CT test named BranchGatesListSelfRevokeNoFlip (or equivalent) exists and exercises the symmetric self-revoke path",
      "verify": "pnpm test:ct:desktop -- BranchGatesListSelfRevokeNoFlip",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "REMEDIATE-UI-1 header shows Superseded by: REMEDIATE-06B-D and Status: Cancelled",
      "verify": "grep -i '^\\*\\*Superseded by:\\*\\* REMEDIATE-06B-D' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/REMEDIATE-UI-1-*.md && grep -i '^\\*\\*Status:\\*\\* Cancelled' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/REMEDIATE-UI-1-*.md",
      "maps_to_ac": "AC-4"
    }
  ]
}
-->

