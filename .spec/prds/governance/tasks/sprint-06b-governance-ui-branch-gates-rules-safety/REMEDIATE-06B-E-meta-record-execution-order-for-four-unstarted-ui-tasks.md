
# REMEDIATE-06B-E: Meta-record execution order for the four unstarted UI tasks

**Type:** META | **Status:** Backlog | **Priority:** P0 | **Effort:** (delegated)
**Agent:** (none — /kb-run-sprint invocation) | **Reviewer:** sveltekit-reviewer | **Proposed by:** sveltekit-planner
**Closes red-hat findings:** F1, F3, F4, F5
**Depends on:** REMEDIATE-06B-A, REMEDIATE-06B-B | **Blocks:** sprint Done
**PRD refs:** UC-MGMT-04, UC-MGMT-05, UC-MGMT-06, UC-MGMT-07 | **Capabilities:** CAP-AUTHZ-01, CAP-CONFIG-01

## What this does

This file is a pure dependency-ordering record for `/kb-run-sprint`. It does not contain implementation work. It writes an **Execution Plan** that shows how the four not-started UI tasks must run: `MGMT-UI-009` before `MGMT-UI-010` and `MGMT-UI-011`, and `MGMT-UI-012` after those three. It also links to `REMEDIATE-06B-D` as the capstone proof that gates sprint closure after the four tasks land.

## Why

The red-hat review found all four UI tasks not started and noted that the namesake `BranchGatesList.svelte` blocks the rest of the surface. A meta task keeps the ordering constraint explicit in the sprint contract so `/kb-run-sprint` does not attempt to run the build-gate suite before the components it asserts against exist.

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: REMEDIATE-06B-E — Meta-record execution order for the four unstarted UI tasks
================================================================================

TASK_TYPE:   META
STATUS:      Backlog
PRIORITY:    P0
EFFORT:      delegated
AGENT:       implementer=(none — /kb-run-sprint invocation) | reviewer=sveltekit-reviewer
PROPOSED-BY: sveltekit-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-MGMT-04, UC-MGMT-05, UC-MGMT-06, UC-MGMT-07
CAPABILITIES:CAP-AUTHZ-01,CAP-CONFIG-01
CLOSES:      F1, F3, F4, F5

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
This file is a pure dependency-ordering record for /kb-run-sprint. It does not contain implementation work. It writes an Execution Plan that shows how the four not-started UI tasks must run: MGMT-UI-009 before MGMT-UI-010 and MGMT-UI-011, and MGMT-UI-012 after those three. It also links to REMEDIATE-06B-D as the capstone proof that gates sprint closure after the four tasks land.

Success state: A one-page "Execution Plan" section names the four task files in dependency order with one-line justifications; SPRINT.md shows all four tasks as Done once they land.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST NOT introduce new implementation work in this file — it is purely a dependency-ordering record.
- [MUST] MUST name the four task files by exact filename.
- [MUST] MUST link to REMEDIATE-06B-D as the capstone proof that gates sprint closure.
- [NEVER] NEVER mark AC-2 as done before the four tasks actually reach Status: Done in SPRINT.md.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1: The Execution Plan section lists MGMT-UI-009, MGMT-UI-010, MGMT-UI-011, MGMT-UI-012 in valid dependency order with one-line justification per step
- [ ] AC-2: All four tasks reach Status: Done in SPRINT.md (verified by grep; leave this AC [ ] until the tasks actually land)

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (bookkeeping ACs)
--------------------------------------------------------------------------------
AC-1 : The Execution Plan section lists MGMT-UI-009, MGMT-UI-010, MGMT-UI-011, MGMT-UI-012 in valid dependency order with one-line justification per step
  GIVEN: the four UI tasks are not started and have implicit dependencies
  WHEN: the Execution Plan is written
  THEN: the plan lists the tasks in dependency order with one-line justification per step and links to REMEDIATE-06B-D as the sprint-closure gate
  TEST_TIER: structural   VERIFICATION_SERVICE: grep
  VERIFY: grep -E '^1\\. MGMT-UI-009|^2\\. MGMT-UI-010|^3\\. MGMT-UI-011|^4\\. MGMT-UI-012' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/REMEDIATE-06B-E*.md
  SCENARIO:
    tier: structural   test_tier: structural
    verification_service: grep on this task file
    negative_control.would_fail_if:
      - tasks are listed out of order
      - a task is missing from the plan
      - plan lacks a link to REMEDIATE-06B-D
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=empty_execution_plan
      action.actor=sveltekit-planner
        - add a "## Execution Plan" section
        - list the four tasks in order with filenames and justifications
        - add a final bullet linking to REMEDIATE-06B-D as the capstone gate
      end_state.must_observe:
        - "1. MGMT-UI-009-branch-gates-list.md" appears before "2. MGMT-UI-010-ruleslist-principalid.md"
        - "3. MGMT-UI-011-accessibility-ipc-retry.md" appears after step 2
        - "4. MGMT-UI-012-build-gate-tests.md" appears after step 3
        - a reference to REMEDIATE-06B-D exists in the plan
      end_state.must_not_observe:
        - missing tasks
        - reversed order
        - no capstone gate link

AC-2 : All four tasks reach Status: Done in SPRINT.md (verified by grep; leave this AC [ ] until the tasks actually land)
  GIVEN: the four UI tasks are currently Not Started
  WHEN: /kb-run-sprint executes them
  THEN: SPRINT.md Completion Status shows each as Done
  TEST_TIER: structural   VERIFICATION_SERVICE: grep
  VERIFY: grep -E '^\\| MGMT-UI-00(9|10|11|12)' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md | grep -c 'Done'
  SCENARIO:
    tier: structural   test_tier: structural
    verification_service: grep on SPRINT.md Completion Status table
    negative_control.would_fail_if:
      - AC-2 is checked before all four tasks are Done
      - a task is still Not Started when this AC is marked done
      - the grep misses one of the four task IDs
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=four_ui_tasks_not_started
      action.actor=kb-run-sprint
        - execute MGMT-UI-009
        - execute MGMT-UI-010
        - execute MGMT-UI-011
        - execute MGMT-UI-012
        - update SPRINT.md statuses to Done
      end_state.must_observe:
        - SPRINT.md shows MGMT-UI-009 Status: Done
        - SPRINT.md shows MGMT-UI-010 Status: Done
        - SPRINT.md shows MGMT-UI-011 Status: Done
        - SPRINT.md shows MGMT-UI-012 Status: Done
      end_state.must_not_observe:
        - any of the four still Not Started
        - AC-2 checked prematurely

--------------------------------------------------------------------------------
TEST CRITERIA
--------------------------------------------------------------------------------
- TC-1: Execution Plan lists the four task filenames in valid dependency order
- TC-2: Execution Plan references REMEDIATE-06B-D as the sprint-closure capstone gate
- TC-3: SPRINT.md shows MGMT-UI-009, MGMT-UI-010, MGMT-UI-011, and MGMT-UI-012 as Done before AC-2 is checked

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
- .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-009-branch-gates-list.md
- .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-010-ruleslist-principalid.md
- .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-011-accessibility-ipc-retry.md
- .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-012-build-gate-tests.md
- .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/REMEDIATE-06B-D-add-uc-mgmt-06-capstone-no-bypass-proof-ac.md

--------------------------------------------------------------------------------
GUARDRAILS
--------------------------------------------------------------------------------
WRITE_ALLOWED:
  - this task file
  - .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md (status updates only, by /kb-run-sprint)
WRITE_PROHIBITED:
  - source code

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- grep -E '^[0-9]+\\. MGMT-UI-00(9|10|11|12)' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/REMEDIATE-06B-E*.md  →  in order
- grep -c 'REMEDIATE-06B-D' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/REMEDIATE-06B-E*.md  →  >= 1
- grep -E '^\\| MGMT-UI-00(9|10|11|12)' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md | grep -c 'Done'  →  4 (after execution)

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
depends_on: REMEDIATE-06B-A, REMEDIATE-06B-B
blocks:     sprint Done

--------------------------------------------------------------------------------
NOTES
--------------------------------------------------------------------------------
This task has no implementation behavior; it is a routing record. The actual implementation is performed by the existing MGMT-UI-* tasks via /kb-run-sprint.
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "REMEDIATE-06B-E",
  "proposed_by": "sveltekit-planner",
  "supersedes": [],
  "closes_redhat_findings": [
    "F1",
    "F3",
    "F4",
    "F5"
  ],
  "fixtures": {
    "empty_execution_plan": {
      "description": "REMEDIATE-06B-E file before the Execution Plan section is written",
      "seed_method": "component_mount",
      "records": [
        "no Execution Plan section"
      ]
    },
    "four_ui_tasks_not_started": {
      "description": "The four UI tasks listed as Not Started in SPRINT.md before /kb-run-sprint executes them",
      "seed_method": "component_mount",
      "records": [
        "MGMT-UI-009 Not Started",
        "MGMT-UI-010 Not Started",
        "MGMT-UI-011 Not Started",
        "MGMT-UI-012 Not Started"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN the four UI tasks are not started and have implicit dependencies WHEN the Execution Plan is written THEN the plan lists the tasks in dependency order with one-line justification per step and links to REMEDIATE-06B-D as the sprint-closure gate",
      "verify": "grep -E '^1\\. MGMT-UI-009|^2\\. MGMT-UI-010|^3\\. MGMT-UI-011|^4\\. MGMT-UI-012' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/REMEDIATE-06B-E*.md",
      "scenario": {
        "tier": "structural",
        "test_tier": "structural",
        "verification_service": "grep on this task file",
        "negative_control": {
          "would_fail_if": [
            "tasks are listed out of order",
            "a task is missing from the plan",
            "plan lacks a link to REMEDIATE-06B-D"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "empty_execution_plan",
            "action": {
              "actor": "sveltekit-planner",
              "steps": [
                "add a '## Execution Plan' section",
                "list the four tasks in order with filenames and justifications",
                "add a final bullet linking to REMEDIATE-06B-D as the capstone gate"
              ]
            },
            "end_state": {
              "must_observe": [
                "'1. MGMT-UI-009-branch-gates-list.md' appears before '2. MGMT-UI-010-ruleslist-principalid.md'",
                "'3. MGMT-UI-011-accessibility-ipc-retry.md' appears after step 2",
                "'4. MGMT-UI-012-build-gate-tests.md' appears after step 3",
                "a reference to REMEDIATE-06B-D exists in the plan"
              ],
              "must_not_observe": [
                "missing tasks",
                "reversed order",
                "no capstone gate link"
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
      "description": "GIVEN the four UI tasks are currently Not Started WHEN /kb-run-sprint executes them THEN SPRINT.md Completion Status shows each as Done",
      "verify": "grep -E '^\\\\| MGMT-UI-00(9|10|11|12)' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md | grep -c 'Done'",
      "scenario": {
        "tier": "structural",
        "test_tier": "structural",
        "verification_service": "grep on SPRINT.md Completion Status table",
        "negative_control": {
          "would_fail_if": [
            "AC-2 is checked before all four tasks are Done",
            "a task is still Not Started when this AC is marked done",
            "the grep misses one of the four task IDs"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "four_ui_tasks_not_started",
            "action": {
              "actor": "kb-run-sprint",
              "steps": [
                "execute MGMT-UI-009",
                "execute MGMT-UI-010",
                "execute MGMT-UI-011",
                "execute MGMT-UI-012",
                "update SPRINT.md statuses to Done"
              ]
            },
            "end_state": {
              "must_observe": [
                "SPRINT.md shows MGMT-UI-009 Status: Done",
                "SPRINT.md shows MGMT-UI-010 Status: Done",
                "SPRINT.md shows MGMT-UI-011 Status: Done",
                "SPRINT.md shows MGMT-UI-012 Status: Done"
              ],
              "must_not_observe": [
                "any of the four still Not Started",
                "AC-2 checked prematurely"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "Execution Plan lists the four task filenames in valid dependency order",
      "verify": "grep -E '^[0-9]+\\\\. MGMT-UI-00(9|10|11|12)' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/REMEDIATE-06B-E*.md",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "Execution Plan references REMEDIATE-06B-D as the sprint-closure capstone gate",
      "verify": "grep -c 'REMEDIATE-06B-D' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/REMEDIATE-06B-E*.md",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "SPRINT.md shows MGMT-UI-009, MGMT-UI-010, MGMT-UI-011, and MGMT-UI-012 as Done before AC-2 is checked",
      "verify": "grep -E '^\\\\| MGMT-UI-00(9|10|11|12)' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md | grep -c 'Done'",
      "maps_to_ac": "AC-2"
    }
  ]
}
-->
</task_result>
