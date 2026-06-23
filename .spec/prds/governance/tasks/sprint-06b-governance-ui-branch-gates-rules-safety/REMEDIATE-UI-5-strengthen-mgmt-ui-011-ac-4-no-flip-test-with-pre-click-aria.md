# REMEDIATE-UI-5: Strengthen MGMT-UI-011 AC-4 no-flip test with pre-click aria-checked assertion

**Type:** REMEDIATION | **Status:** Backlog | **Priority:** P0 | **Effort:** S (45 min)
**Agent:** sveltekit-implementer | **Reviewer:** sveltekit-reviewer | **Proposed by:** sveltekit-planner
**Closes red-hat findings:** L4
**Depends on:** MGMT-UI-011 | **Blocks:** E2E-MGMT-UI-001
**PRD refs:** UC-MGMT-04, DESIGN-MGMT-004 | **Capabilities:** CAP-AUTHZ-01, CAP-A11Y-01

## What this does

MGMT-UI-011 AC-4 currently proves that aria-checked is true after a denied click. That oracle can pass if the control was already false and click is ignored. Update the contract to require an explicit pre-click assertion: aria-checked='true' before the click, and aria-checked='true' after the click with a danger InfoMessage visible. Add a corresponding test criterion and RED-phase evidence that a stubbed no-op Toggle would fail the stronger oracle.

## Why

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: REMEDIATE-UI-5 — Strengthen MGMT-UI-011 AC-4 no-flip test with pre-click aria-checked assertion
================================================================================

TASK_TYPE:   REMEDIATION
STATUS:      Backlog
PRIORITY:    P0
EFFORT:      S  (45 min)
AGENT:       implementer=sveltekit-implementer | reviewer=sveltekit-reviewer
PROPOSED-BY: sveltekit-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-MGMT-04, DESIGN-MGMT-004
CAPABILITIES:CAP-AUTHZ-01,CAP-A11Y-01
CLOSES:      L4

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
MGMT-UI-011 AC-4 currently proves that aria-checked is true after a denied click. That oracle can pass if the control was already false and click is ignored. Update the contract to require an explicit pre-click assertion: aria-checked='true' before the click, and aria-checked='true' after the click with a danger InfoMessage visible. Add a corresponding test criterion and RED-phase evidence that a stubbed no-op Toggle would fail the stronger oracle.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] The no-flip proof must assert both BEFORE and AFTER Toggle state.
- [MUST] The denial AC must fail if the Toggle starts in the denied end-state and the click handler is a no-op.
- [MUST] Changes are limited to MGMT-UI-011 contract and its existing component test plan.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: the scenario's must_observe list includes both pre-click and post-click checked states and the verification gate catches no-op Toggle handlers
- [ ] AC-2: the test fails because aria-checked is not true before the click or not true after the click

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: the scenario's must_observe list includes both pre-click and post-click checked states and the verification gate catches no-op Toggle handlers
  GIVEN: MGMT-UI-011 AC-4's no-flip scenario currently asserts only post-click state
  WHEN: the contract is amended to require an explicit pre-click aria-checked='true' assertion
  THEN: the scenario's must_observe list includes both pre-click and post-click checked states and the verification gate catches no-op Toggle handlers
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  SCENARIO:
    tier: visible   test_tier: integration
    verification_service: pnpm test:ct:desktop -- NoFlipPreClickAssertion (real Svelte 5 runtime + sanctioned but-sdk mock layer per B14)
    negative_control.would_fail_if:
      - the scenario only asserts aria-checked after the click
      - before and after aria-checked values are captured but not compared
      - the click step is absent from the scenario
    evidence: artifact_type=screenshot required_capture=True
    case[0] start_ref=mgmt_ui_011_ac4_contract
      action.actor=ci
        - parse MGMT-UI-011 AC-4 scenario.cases
        - assert the steps include an aria-checked capture BEFORE the click
        - assert the steps include an aria-checked capture AFTER the click
        - assert the scenario enforces the two captured values are equal
      end_state.must_observe:
        - pre-click aria-checked='true' assertion is present
        - post-click aria-checked='true' assertion is present
        - an equality/comparison assertion between pre-click and post-click values is present
      end_state.must_not_observe:
        - only a post-click aria-checked assertion
        - pre-click and post-click captured without a comparison step
        - click step absent from the scenario

AC-2 : the test fails because aria-checked is not true before the click or not true after the click
  GIVEN: the stronger oracle is implemented in the component test
  WHEN: a deliberate no-op Toggle is used (click does nothing but does not throw)
  THEN: the test fails because aria-checked is not true before the click or not true after the click
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  SCENARIO:
    tier: visible   test_tier: integration
    verification_service: pnpm test:ct:desktop -- NoFlipPreClickNegativeControl (real Svelte 5 runtime + no-op Toggle stub per B14)
    negative_control.would_fail_if:
      - the negative-control test passes despite the no-op handler
      - the test fails for a syntax error rather than a behavioral mismatch
      - the no-op Toggle is replaced by a real Toggle before the RED run
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=no_op_toggle_stub
      action.actor=maintainer
        - replace the real Toggle with a no-op variant initially
        - run the no-flip pre-click test
        - observe failing assertion
      end_state.must_observe:
        - test runner reports an assertion failure
        - failure message references aria-checked mismatch
      end_state.must_not_observe:
        - test exit code 0
        - failure message about a syntax or import error only

--------------------------------------------------------------------------------
TEST CRITERIA
--------------------------------------------------------------------------------
- TC-1: MGMT-UI-011 AC-4 must_observe includes pre-click aria-checked='true'
- TC-2: A component test with a no-op denied Toggle fails the pre-click/post-click assertion

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------

--------------------------------------------------------------------------------
GUARDRAILS
--------------------------------------------------------------------------------
WRITE_ALLOWED:

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- grep -B2 -A8 'aria-checked.*true' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-011*.md | head -40  →  ?
- pnpm test:ct:desktop -- NoFlipPreClickAssertion  →  ?
- pnpm -F @gitbutler/desktop check  →  ?
- pnpm lint  →  ?

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
depends_on: MGMT-UI-011
blocks:     E2E-MGMT-UI-001

--------------------------------------------------------------------------------
NOTES
--------------------------------------------------------------------------------
This remediation closes the L4 finding. It does not change runtime implementation; it hardens the test oracle and the contract. The RED evidence from a no-op Toggle is the load-bearing proof that the new assertion is actually checking behavior, not just a tautology.

```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "REMEDIATE-UI-5",
  "proposed_by": "sveltekit-planner",
  "supersedes": [],
  "closes_redhat_findings": [
    "L4"
  ],
  "fixtures": {
    "mgmt_ui_011_ac4_contract": {
      "description": "MGMT-UI-011 AC-4 contract text describing the no-flip scenario",
      "seed_method": "component_mount",
      "records": [
        "MGMT-UI-011*.md AC-4 scenario block"
      ]
    },
    "seeded_write_denied": {
      "description": "GovernanceSettings mounted with a protected control that starts ON and branch_gates_update mocked to return perm.denied",
      "seed_method": "component_mount",
      "records": [
        "protected control starts checked",
        "denied write response"
      ]
    },
    "no_op_toggle_stub": {
      "description": "A temporary Toggle stub that ignores click events and does not change aria-checked",
      "seed_method": "component_mount",
      "records": [
        "Toggle onClick handler is a no-op"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN MGMT-UI-011 AC-4's no-flip scenario currently asserts only post-click state WHEN the contract is amended to require an explicit pre-click aria-checked='true' assertion THEN the scenario's must_observe list includes both pre-click and post-click checked states and the verification gate catches no-op Toggle handlers",
      "verify": "",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "pnpm test:ct:desktop -- NoFlipPreClickAssertion (real Svelte 5 runtime + sanctioned but-sdk mock layer per B14)",
        "negative_control": {
          "would_fail_if": [
            "the scenario only asserts aria-checked after the click",
            "before and after aria-checked values are captured but not compared",
            "the click step is absent from the scenario"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "mgmt_ui_011_ac4_contract",
            "action": {
              "actor": "ci",
              "steps": [
                "parse MGMT-UI-011 AC-4 scenario.cases",
                "assert the steps include an aria-checked capture BEFORE the click",
                "assert the steps include an aria-checked capture AFTER the click",
                "assert the scenario enforces the two captured values are equal"
              ]
            },
            "end_state": {
              "must_observe": [
                "pre-click aria-checked='true' assertion is present",
                "post-click aria-checked='true' assertion is present",
                "an equality/comparison assertion between pre-click and post-click values is present"
              ],
              "must_not_observe": [
                "only a post-click aria-checked assertion",
                "pre-click and post-click captured without a comparison step",
                "click step absent from the scenario"
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
      "description": "GIVEN the stronger oracle is implemented in the component test WHEN a deliberate no-op Toggle is used (click does nothing but does not throw) THEN the test fails because aria-checked is not true before the click or not true after the click",
      "verify": "",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "pnpm test:ct:desktop -- NoFlipPreClickNegativeControl (real Svelte 5 runtime + no-op Toggle stub per B14)",
        "negative_control": {
          "would_fail_if": [
            "the negative-control test passes despite the no-op handler",
            "the test fails for a syntax error rather than a behavioral mismatch",
            "the no-op Toggle is replaced by a real Toggle before the RED run"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "no_op_toggle_stub",
            "action": {
              "actor": "maintainer",
              "steps": [
                "replace the real Toggle with a no-op variant initially",
                "run the no-flip pre-click test",
                "observe failing assertion"
              ]
            },
            "end_state": {
              "must_observe": [
                "test runner reports an assertion failure",
                "failure message references aria-checked mismatch"
              ],
              "must_not_observe": [
                "test exit code 0",
                "failure message about a syntax or import error only"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "MGMT-UI-011 AC-4 must_observe includes pre-click aria-checked='true'",
      "verify": "grep -A4 'pre-click' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-011*.md",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "A component test with a no-op denied Toggle fails the pre-click/post-click assertion",
      "verify": "pnpm test:ct:desktop -- NoFlipPreClickAssertion",
      "maps_to_ac": "AC-2"
    }
  ]
}
-->
