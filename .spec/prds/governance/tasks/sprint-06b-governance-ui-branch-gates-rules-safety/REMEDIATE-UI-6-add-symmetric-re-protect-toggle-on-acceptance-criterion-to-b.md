# REMEDIATE-UI-6: Add symmetric re-protect (toggle ON) acceptance criterion to BranchGatesList

**Type:** REMEDIATION | **Status:** Backlog | **Priority:** P0 | **Effort:** S (60 min)
**Agent:** sveltekit-implementer | **Reviewer:** sveltekit-reviewer | **Proposed by:** sveltekit-planner
**Closes red-hat findings:** L2
**Depends on:** MGMT-UI-009 | **Blocks:** E2E-MGMT-UI-001
**PRD refs:** UC-MGMT-04 | **Capabilities:** CAP-AUTHZ-01, CAP-CONFIG-01

## What this does

MGMT-UI-009 AC-5 covers unprotecting a branch (toggle protected OFF) with a Modal confirmation. It lacks the inverse flow: toggling protected ON (re-protect). Add AC-8 to MGMT-UI-009 that proves re-protect is silent (no Modal) and immediately stages branch_gates_update with protected:true, then shows a pending indicator. Include a RED-phase test that deliberately expects a Modal and fails, confirming the AC is behavioral rather than a tautology.

## Why

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: REMEDIATE-UI-6 — Add symmetric re-protect (toggle ON) acceptance criterion to BranchGatesList
================================================================================

TASK_TYPE:   REMEDIATION
STATUS:      Backlog
PRIORITY:    P0
EFFORT:      S  (60 min)
AGENT:       implementer=sveltekit-implementer | reviewer=sveltekit-reviewer
PROPOSED-BY: sveltekit-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-MGMT-04
CAPABILITIES:CAP-AUTHZ-01,CAP-CONFIG-01
CLOSES:      L2

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
MGMT-UI-009 AC-5 covers unprotecting a branch (toggle protected OFF) with a Modal confirmation. It lacks the inverse flow: toggling protected ON (re-protect). Add AC-8 to MGMT-UI-009 that proves re-protect is silent (no Modal) and immediately stages branch_gates_update with protected:true, then shows a pending indicator. Include a RED-phase test that deliberately expects a Modal and fails, confirming the AC is behavioral rather than a tautology.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] Re-protect (toggle protected ON) must stage an immediate branch_gates_update write with protected:true.
- [MUST] Re-protect must NOT show a Modal confirmation because it is non-destructive.
- [MUST] The new AC must share the same fixture as AC-5 and include pending-state evidence.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: no Modal appears, branch_gates_update is called once with protected:true, and a pending indicator appears
- [ ] AC-2: the test fails because no Modal opens

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: no Modal appears, branch_gates_update is called once with protected:true, and a pending indicator appears
  GIVEN: BranchGatesList mounted with seeded_gates_two_branches (develop protected:false), develop row expanded
  WHEN: user toggles the protected Toggle ON
  THEN: no Modal appears, branch_gates_update is called once with protected:true, and a pending indicator appears
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  SCENARIO:
    tier: visible   test_tier: integration
    verification_service: pnpm test:ct:desktop -- BranchGatesListReprotect (real Svelte 5 runtime + sanctioned but-sdk mock layer per B14)
    negative_control.would_fail_if:
      - a Modal confirmation appears during re-protect
      - the protected Toggle stays aria-checked='false' after the click
      - the component stubs the click handler and never calls branch_gates_update
    evidence: artifact_type=screenshot required_capture=True
    case[0] start_ref=seeded_gates_two_branches
      action.actor=user
        - expand the 'develop' row
        - click the protected Toggle to toggle ON
      end_state.must_observe:
        - the 'develop' protected Toggle has aria-checked='true'
        - branch_gates_update SDK spy called == 1 time with {branch: 'develop', protected: true}
        - a pending indicator (Badge or aria-label containing 'pending') appears on the 'develop' row
        - document query for role='dialog' returns 0 elements
      end_state.must_not_observe:
        - a Modal dialog in the DOM
        - Toggle aria-checked='false' after the click
        - 0 branch_gates_update calls
        - branch_gates_update called with protected:false

AC-2 : the test fails because no Modal opens
  GIVEN: a component test awaiting a Modal after toggling protected ON
  WHEN: the test runs against the correct implementation
  THEN: the test fails because no Modal opens
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  SCENARIO:
    tier: visible   test_tier: integration
    verification_service: pnpm test:ct:desktop -- BranchGatesListReprotectModalNegative (real Svelte 5 runtime + Modal-expecting assertion per B14)
    negative_control.would_fail_if:
      - the Modal-asserting test passes (Modal actually appears incorrectly)
      - the test fails for import/unmount reasons rather than missing Modal
      - the test is skipped before running
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=seeded_gates_two_branches
      action.actor=maintainer
        - write a temporary test that expects a Modal after re-protect
        - run the negative-control test
        - capture the assertion failure
      end_state.must_observe:
        - test runner exits non-zero
        - failure message references a missing Modal
      end_state.must_not_observe:
        - test exit code 0
        - failure unrelated to the Modal absence

--------------------------------------------------------------------------------
TEST CRITERIA
--------------------------------------------------------------------------------
- TC-1: Toggling protected ON calls branch_gates_update with protected:true and shows a pending indicator
- TC-2: No Modal appears during the re-protect flow

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
- grep -A20 'AC-8\|Re-protect' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-009*.md  →  ?
- pnpm test:ct:desktop -- BranchGatesListReprotect  →  ?
- pnpm -F @gitbutler/desktop check  →  ?
- pnpm lint  →  ?

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
depends_on: MGMT-UI-009
blocks:     E2E-MGMT-UI-001

--------------------------------------------------------------------------------
NOTES
--------------------------------------------------------------------------------
This remediation closes the L2 finding. It adds the missing inverse-flow AC and RED evidence. It does not change the unprotect Modal behavior.

```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "REMEDIATE-UI-6",
  "proposed_by": "sveltekit-planner",
  "supersedes": [],
  "closes_redhat_findings": [
    "L2"
  ],
  "fixtures": {
    "seeded_gates_two_branches": {
      "description": "BranchGatesList mounted with two branches where develop is protected:false and main is protected:true",
      "seed_method": "component_mount",
      "records": [
        "main protected:true",
        "develop protected:false"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN BranchGatesList mounted with seeded_gates_two_branches (develop protected:false), develop row expanded WHEN user toggles the protected Toggle ON THEN no Modal appears, branch_gates_update is called once with protected:true, and a pending indicator appears",
      "verify": "",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "pnpm test:ct:desktop -- BranchGatesListReprotect (real Svelte 5 runtime + sanctioned but-sdk mock layer per B14)",
        "negative_control": {
          "would_fail_if": [
            "a Modal confirmation appears during re-protect",
            "the protected Toggle stays aria-checked='false' after the click",
            "the component stubs the click handler and never calls branch_gates_update"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_gates_two_branches",
            "action": {
              "actor": "user",
              "steps": [
                "expand the 'develop' row",
                "click the protected Toggle to toggle ON"
              ]
            },
            "end_state": {
              "must_observe": [
                "the 'develop' protected Toggle has aria-checked='true'",
                "branch_gates_update SDK spy called == 1 time with {branch: 'develop', protected: true}",
                "a pending indicator (Badge or aria-label containing 'pending') appears on the 'develop' row",
                "document query for role='dialog' returns 0 elements"
              ],
              "must_not_observe": [
                "a Modal dialog in the DOM",
                "Toggle aria-checked='false' after the click",
                "0 branch_gates_update calls",
                "branch_gates_update called with protected:false"
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
      "description": "GIVEN a component test awaiting a Modal after toggling protected ON WHEN the test runs against the correct implementation THEN the test fails because no Modal opens",
      "verify": "",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "pnpm test:ct:desktop -- BranchGatesListReprotectModalNegative (real Svelte 5 runtime + Modal-expecting assertion per B14)",
        "negative_control": {
          "would_fail_if": [
            "the Modal-asserting test passes (Modal actually appears incorrectly)",
            "the test fails for import/unmount reasons rather than missing Modal",
            "the test is skipped before running"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_gates_two_branches",
            "action": {
              "actor": "maintainer",
              "steps": [
                "write a temporary test that expects a Modal after re-protect",
                "run the negative-control test",
                "capture the assertion failure"
              ]
            },
            "end_state": {
              "must_observe": [
                "test runner exits non-zero",
                "failure message references a missing Modal"
              ],
              "must_not_observe": [
                "test exit code 0",
                "failure unrelated to the Modal absence"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "Toggling protected ON calls branch_gates_update with protected:true and shows a pending indicator",
      "verify": "pnpm test:ct:desktop -- BranchGatesListReprotect",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "No Modal appears during the re-protect flow",
      "verify": "pnpm test:ct:desktop -- BranchGatesListReprotectNoModal",
      "maps_to_ac": "AC-1"
    }
  ]
}
-->
