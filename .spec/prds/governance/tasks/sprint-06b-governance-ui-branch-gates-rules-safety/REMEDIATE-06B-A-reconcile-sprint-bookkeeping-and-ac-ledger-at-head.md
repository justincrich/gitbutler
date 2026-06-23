
# REMEDIATE-06B-A: Reconcile sprint bookkeeping and AC ledger at HEAD

**Type:** REMEDIATION | **Status:** Backlog | **Priority:** P0 | **Effort:** S (45 min)
**Agent:** sveltekit-implementer | **Reviewer:** sveltekit-reviewer | **Proposed by:** sveltekit-planner
**Closes red-hat findings:** F6, F7-partial
**Depends on:** (none) | **Blocks:** sprint closure
**PRD refs:** SPRINT.md, .spec/prds/governance/reviews/red-hat-20260623T031824Z-sprint-06b.md | **Capabilities:** CAP-DOC-01

## What this does

The red-hat review found that every task file still shows `[ ]` for every AC, and `SPRINT.md` still says `status: Planned` even though several tasks have landed at HEAD. This remediation brings the sprint paperwork into line with reality without changing a single line of source code. It updates the sprint status, adds a `Completion Status (HEAD f7a1589c6c)` section, marks PASS-evidenced ACs as `[x]`, leaves partial or not-started ACs honestly unchecked with annotations, and makes sure `Task Detail Files` lists every contract file that actually lives in the sprint directory.

## Why

A sprint ledger that ignores landed work blocks honest closure: `/kb-run-sprint` consumes these task files, and unchecked ACs on passing code make the sprint appear entirely unstarted. We must reconcile the paperwork before the remaining four UI tasks and the follow-up triage can be executed, or the final review will fail on bookkeeping alone.

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: REMEDIATE-06B-A — Reconcile sprint bookkeeping and AC ledger at HEAD
================================================================================

TASK_TYPE:   REMEDIATION
STATUS:      Backlog
PRIORITY:    P0
EFFORT:      S  (45 min)
AGENT:       implementer=sveltekit-implementer | reviewer=sveltekit-reviewer
PROPOSED-BY: sveltekit-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    SPRINT.md, .spec/prds/governance/reviews/red-hat-20260623T031824Z-sprint-06b.md
CAPABILITIES:CAP-DOC-01
CLOSES:      F6, F7-partial

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
The red-hat review found that every task file still shows `[ ]` for every AC, and `SPRINT.md` still says `status: Planned` even though several tasks have landed at HEAD. This remediation brings the sprint paperwork into line with reality without changing a single line of source code. It updates the sprint status, adds a `Completion Status (HEAD f7a1589c6c)` section, marks PASS-evidenced ACs as `[x]`, leaves partial or not-started ACs honestly unchecked with annotations, and makes sure `Task Detail Files` lists every contract file that actually lives in the sprint directory.

Success state: SPRINT.md reads `status: In Progress`, a Completion Status table lists every task honestly, PASS-evidenced ACs are `[x]`, the four not-started UI tasks retain all `[ ]`, and the task-detail list matches the filesystem.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] Edit only `.spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/*.md` and `SPRINT.md`. NEVER touch source code.
- [MUST] NOT check off an AC unless file:line evidence in `.spec/prds/governance/reviews/red-hat-20260623T031824Z-sprint-06b.md` supports PASS.
- [MUST] NOT mark any of the 4 NOT-STARTED UI tasks (MGMT-UI-009, MGMT-UI-010, MGMT-UI-011, MGMT-UI-012) as Done or In Progress.
- [NEVER] NEVER delete or rename any existing task file.
- [STRICTLY] For PARTIAL ACs, leave `[ ]` and append an inline note naming the evidence gap and pointing to REMEDIATE-06B-B (for MGMT-BE-004 AC-3 and MGMT-UI-004 CT gaps).

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: SPRINT.md status field reads "In Progress" and a "Completion Status" section lists every task with its real HEAD state
- [ ] AC-2: Every PASS-evidenced AC in MGMT-BE-003, MGMT-BE-004 (except AC-3), MGMT-UI-004 (with PARTIAL annotations), and DESIGN-MGMT-004/006/007/008 is `[x]`
- [ ] AC-3: The 4 NOT-STARTED UI task files retain all `[ ]` (zero `[x]`) across all four
- [ ] AC-4: The "Task Detail Files" section enumerates all `.md` files returned by `ls *.md | wc -l`

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: SPRINT.md status field reads "In Progress" and a "Completion Status" section lists every task with its real HEAD state
  GIVEN: SPRINT.md currently has `status: Planned` and no Completion Status section
  WHEN: the bookkeeping remediation is applied
  THEN: SPRINT.md status field reads "In Progress" and a "Completion Status (HEAD f7a1589c6c)" section lists every task with its real HEAD state (Done / In Progress / Not Started)
  TEST_TIER: structural   VERIFICATION_SERVICE: grep
  VERIFY: grep -c '^status: In Progress$' SPRINT.md && grep -c '^## Completion Status' SPRINT.md
  SCENARIO:
    tier: structural   test_tier: structural
    verification_service: grep .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md
    negative_control.would_fail_if:
      - status is left as "Planned"
      - status is changed to "Done" while not-started tasks remain
      - Completion Status section is missing or omits one of the 11 original tasks
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=sprint_md_before_reconciliation
      action.actor=sveltekit-implementer
        - edit SPRINT.md frontmatter `status:` to `In Progress`
        - append a `## Completion Status (HEAD f7a1589c6c)` table after the Human Testing Gate
        - list every task ID, title, agent, and real HEAD state
      end_state.must_observe:
        - grep -c '^status: In Progress$' SPRINT.md returns >= 1
        - grep -c '^## Completion Status' SPRINT.md returns 1
        - the table contains rows for MGMT-BE-003, MGMT-BE-004, MGMT-UI-004, MGMT-UI-009, MGMT-UI-010, MGMT-UI-011, MGMT-UI-012, DESIGN-MGMT-004, DESIGN-MGMT-006, DESIGN-MGMT-007, DESIGN-MGMT-008
        - not-started tasks are listed as "Not Started"
      end_state.must_not_observe:
        - status equal to "Planned"
        - not-started tasks marked "Done" or "In Progress"

AC-2 : Every PASS-evidenced AC in MGMT-BE-003, MGMT-BE-004 (except AC-3), MGMT-UI-004 (with PARTIAL annotations), and DESIGN-MGMT-004/006/007/008 is `[x]`
  GIVEN: all 11 task files currently have every AC unchecked
  WHEN: the red-hat review's file:line PASS evidence is applied to the AC ledger
  THEN: every AC with PASS evidence is `[x]`; PARTIAL ACs remain `[ ]` with an inline gap note; not-started tasks remain unchecked
  TEST_TIER: structural   VERIFICATION_SERVICE: grep
  VERIFY: grep -c '^- \[x\]' <task>.md per landed task
  SCENARIO:
    tier: structural   test_tier: structural
    verification_service: grep -c '^- \[x\]' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-*.md
    negative_control.would_fail_if:
      - an AC is checked without file:line PASS evidence
      - MGMT-BE-004 AC-3 is marked `[x]` before REMEDIATE-06B-B captures passing cargo evidence
      - MGMT-UI-004 AC-1/AC-3 are marked `[x]` without CT evidence
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=unchecked_task_files
      action.actor=sveltekit-implementer
        - read red-hat-20260623T031824Z-sprint-06b.md verdict table
        - mark PASS-evidenced ACs `[x]` and annotate PARTIAL ones
      end_state.must_observe:
        - MGMT-BE-003*.md contains >= 5 checked ACs
        - MGMT-BE-004*.md contains >= 3 checked ACs (AC-3 remains `[ ]`)
        - MGMT-UI-004*.md contains >= 1 checked AC (AC-2 only)
        - each DESIGN-MGMT-00{4,6,7,8}*.md contains >= 1 checked AC
      end_state.must_not_observe:
        - a checked AC whose evidence note is empty or cites no file:line
        - checked `[x]` in any NOT-STARTED UI task

AC-3 : The 4 NOT-STARTED UI task files retain all `[ ]` (zero `[x]`) across all four
  GIVEN: MGMT-UI-009, MGMT-UI-010, MGMT-UI-011, and MGMT-UI-012 are not implemented at HEAD
  WHEN: the AC reconciliation runs
  THEN: those four files contain zero `[x]` AC checkboxes
  TEST_TIER: structural   VERIFICATION_SERVICE: grep
  VERIFY: grep -c '^- \[x\]' MGMT-UI-00{9,10,11,12}-*.md returns 0 for each of the four files
  SCENARIO:
    tier: structural   test_tier: structural
    verification_service: grep on each not-started UI task file
    negative_control.would_fail_if:
      - a not-started task's AC is checked off prematurely
      - a global replace turns every `[ ]` into `[x]`
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=not_started_ui_task_files
      action.actor=sveltekit-implementer
        - run grep -c '^- \[x\]' on MGMT-UI-009*.md, MGMT-UI-010*.md, MGMT-UI-011*.md, and MGMT-UI-012*.md
      end_state.must_observe:
        - all four grep commands return 0
      end_state.must_not_observe:
        - any non-zero count for those four files

AC-4 : The "Task Detail Files" section enumerates all `.md` files returned by `ls *.md | wc -l`
  GIVEN: the sprint directory now contains the 11 original tasks, 14 follow-up tasks, MGMT-BE-004A, and these 5 new REMEDIATE-06B-* tasks
  WHEN: the Task Detail Files section is updated
  THEN: every .md file in the sprint directory appears as a bullet in that section
  TEST_TIER: structural   VERIFICATION_SERVICE: ls + grep
  VERIFY: test $(ls *.md | wc -l | tr -d ' ') -eq $(grep -cE '^- [A-Z0-9_-]+\.md$' SPRINT.md)
  SCENARIO:
    tier: structural   test_tier: structural
    verification_service: ls .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/*.md and grep of the Task Detail Files list
    negative_control.would_fail_if:
      - the section count is lower than the file count
      - a follow-up task is omitted
      - the new REMEDIATE-06B-* files are omitted
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=sprint_directory_with_all_tasks
      action.actor=sveltekit-implementer
        - list all *.md files in the sprint directory
        - copy that list into the "## Task Detail Files" section of SPRINT.md as bullets
      end_state.must_observe:
        - number of bullet filenames in the section == number of *.md files in the directory
        - every *.md filename appears exactly once
      end_state.must_not_observe:
        - missing filenames
        - duplicated filenames
        - stale generated-by line claiming 11 tasks only

--------------------------------------------------------------------------------
TEST CRITERIA
--------------------------------------------------------------------------------
- TC-1: SPRINT.md contains `status: In Progress` in the frontmatter
- TC-2: SPRINT.md contains a `## Completion Status (HEAD f7a1589c6c)` section
- TC-3: Every landed task has at least one `[x]` AC and not-started UI tasks have zero `[x]`
- TC-4: The Task Detail Files section lists the same number of files as `ls *.md | wc -l`

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
- .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md
- .spec/prds/governance/reviews/red-hat-20260623T031824Z-sprint-06b.md

--------------------------------------------------------------------------------
GUARDRAILS
--------------------------------------------------------------------------------
WRITE_ALLOWED:
  - .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md
  - .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/*.md
WRITE_PROHIBITED:
  - any file outside .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- grep -c '^status: In Progress$' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md  →  >= 1
- grep -c '^## Completion Status' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md  →  1
- grep -c '^- \[x\]' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-00{9,10,11,12}*.md  →  0 for each
- test $(ls .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/*.md | wc -l) -eq $(grep -cE '^- [A-Z0-9_-]+.*\.md$' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md)

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
depends_on: (none)
blocks:     sprint closure

--------------------------------------------------------------------------------
NOTES
--------------------------------------------------------------------------------
This is bookkeeping only — no source code changes. The red-hat review evidence is authoritative for what counts as PASS. If an AC has mixed evidence (e.g., MGMT-BE-004 AC-3), leave it `[ ]` and point to REMEDIATE-06B-B for the missing verification.
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "REMEDIATE-06B-A",
  "proposed_by": "sveltekit-planner",
  "supersedes": [],
  "closes_redhat_findings": [
    "F6",
    "F7-partial"
  ],
  "fixtures": {
    "sprint_md_before_reconciliation": {
      "description": "SPRINT.md with status: Planned and no Completion Status section",
      "seed_method": "component_mount",
      "records": [
        "status: Planned",
        "no Completion Status section"
      ]
    },
    "unchecked_task_files": {
      "description": "All task .md files at HEAD with every AC checkbox still [ ]",
      "seed_method": "component_mount",
      "records": [
        "all 11 original task files",
        "14 follow-up task files",
        "MGMT-BE-004A rescoped file"
      ]
    },
    "not_started_ui_task_files": {
      "description": "The four not-started UI task files (MGMT-UI-009/010/011/012)",
      "seed_method": "component_mount",
      "records": [
        "MGMT-UI-009-branch-gates-list.md",
        "MGMT-UI-010-ruleslist-principalid.md",
        "MGMT-UI-011-accessibility-ipc-retry.md",
        "MGMT-UI-012-build-gate-tests.md"
      ]
    },
    "sprint_directory_with_all_tasks": {
      "description": "The sprint directory after the five new REMEDIATE-06B-* files are added",
      "seed_method": "component_mount",
      "records": [
        "all original, follow-up, and remedial task files"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN SPRINT.md currently has status: Planned and no Completion Status section WHEN the bookkeeping remediation is applied THEN SPRINT.md status field reads 'In Progress' and a 'Completion Status (HEAD f7a1589c6c)' section lists every task with its real HEAD state (Done / In Progress / Not Started)",
      "verify": "grep -c '^status: In Progress$' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md && grep -c '^## Completion Status' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md",
      "scenario": {
        "tier": "structural",
        "test_tier": "structural",
        "verification_service": "grep .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md",
        "negative_control": {
          "would_fail_if": [
            "status is left as 'Planned'",
            "status is changed to 'Done' while not-started tasks remain",
            "Completion Status section is missing or omits one of the 11 original tasks"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "sprint_md_before_reconciliation",
            "action": {
              "actor": "sveltekit-implementer",
              "steps": [
                "edit SPRINT.md frontmatter status: to In Progress",
                "append a '## Completion Status (HEAD f7a1589c6c)' table after the Human Testing Gate",
                "list every task ID, title, agent, and real HEAD state"
              ]
            },
            "end_state": {
              "must_observe": [
                "grep -c '^status: In Progress$' SPRINT.md returns >= 1",
                "grep -c '^## Completion Status' SPRINT.md returns 1",
                "the table contains rows for MGMT-BE-003, MGMT-BE-004, MGMT-UI-004, MGMT-UI-009, MGMT-UI-010, MGMT-UI-011, MGMT-UI-012, DESIGN-MGMT-004, DESIGN-MGMT-006, DESIGN-MGMT-007, DESIGN-MGMT-008",
                "not-started tasks are listed as 'Not Started'"
              ],
              "must_not_observe": [
                "status equal to 'Planned'",
                "not-started tasks marked 'Done' or 'In Progress'"
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
      "description": "GIVEN all 11 task files currently have every AC unchecked WHEN the red-hat review's file:line PASS evidence is applied to the AC ledger THEN every AC with PASS evidence is [x]; PARTIAL ACs remain [ ] with an inline gap note; not-started tasks remain unchecked",
      "verify": "grep -c '^- \\[x\\]' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-*.md",
      "scenario": {
        "tier": "structural",
        "test_tier": "structural",
        "verification_service": "grep -c '^- \\[x\\]' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-*.md",
        "negative_control": {
          "would_fail_if": [
            "an AC is checked without file:line PASS evidence",
            "MGMT-BE-004 AC-3 is marked [x] before REMEDIATE-06B-B captures passing cargo evidence",
            "MGMT-UI-004 AC-1/AC-3 are marked [x] without CT evidence"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "unchecked_task_files",
            "action": {
              "actor": "sveltekit-implementer",
              "steps": [
                "read red-hat-20260623T031824Z-sprint-06b.md verdict table",
                "mark PASS-evidenced ACs [x] and annotate PARTIAL ones"
              ]
            },
            "end_state": {
              "must_observe": [
                "MGMT-BE-003*.md contains >= 5 checked ACs",
                "MGMT-BE-004*.md contains >= 3 checked ACs (AC-3 remains [ ])",
                "MGMT-UI-004*.md contains >= 1 checked AC (AC-2 only)",
                "each DESIGN-MGMT-00{4,6,7,8}*.md contains >= 1 checked AC"
              ],
              "must_not_observe": [
                "a checked AC whose evidence note is empty or cites no file:line",
                "checked [x] in any NOT-STARTED UI task"
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
      "description": "GIVEN MGMT-UI-009, MGMT-UI-010, MGMT-UI-011, and MGMT-UI-012 are not implemented at HEAD WHEN the AC reconciliation runs THEN those four files contain zero [x] AC checkboxes",
      "verify": "for f in MGMT-UI-009 MGMT-UI-010 MGMT-UI-011 MGMT-UI-012; do test \"$(grep -c '^- \\\\[x\\\\]' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/${f}-*.md)\" -eq 0; done",
      "scenario": {
        "tier": "structural",
        "test_tier": "structural",
        "verification_service": "grep on each not-started UI task file",
        "negative_control": {
          "would_fail_if": [
            "a not-started task's AC is checked off prematurely",
            "a global replace turns every [ ] into [x]"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "not_started_ui_task_files",
            "action": {
              "actor": "sveltekit-implementer",
              "steps": [
                "run grep -c '^- [x]' on MGMT-UI-009*.md, MGMT-UI-010*.md, MGMT-UI-011*.md, and MGMT-UI-012*.md"
              ]
            },
            "end_state": {
              "must_observe": [
                "all four grep commands return 0"
              ],
              "must_not_observe": [
                "any non-zero count for those four files"
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
      "description": "GIVEN the sprint directory now contains the 11 original tasks, 14 follow-up tasks, MGMT-BE-004A, and these 5 new REMEDIATE-06B-* tasks WHEN the Task Detail Files section is updated THEN every .md file in the sprint directory appears as a bullet in that section",
      "verify": "test $(ls .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/*.md | wc -l | tr -d ' ') -eq $(grep -cE '^- [A-Z0-9_-]+.*\\.md$' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md)",
      "scenario": {
        "tier": "structural",
        "test_tier": "structural",
        "verification_service": "ls .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/*.md and grep of the Task Detail Files list",
        "negative_control": {
          "would_fail_if": [
            "the section count is lower than the file count",
            "a follow-up task is omitted",
            "the new REMEDIATE-06B-* files are omitted"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "sprint_directory_with_all_tasks",
            "action": {
              "actor": "sveltekit-implementer",
              "steps": [
                "list all *.md files in the sprint directory",
                "copy that list into the '## Task Detail Files' section of SPRINT.md as bullets"
              ]
            },
            "end_state": {
              "must_observe": [
                "number of bullet filenames in the section == number of *.md files in the directory",
                "every *.md filename appears exactly once"
              ],
              "must_not_observe": [
                "missing filenames",
                "duplicated filenames",
                "stale generated-by line claiming 11 tasks only"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "SPRINT.md contains 'status: In Progress' in the frontmatter",
      "verify": "grep -q '^status: In Progress$' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "SPRINT.md contains a '## Completion Status (HEAD f7a1589c6c)' section",
      "verify": "grep -q '^## Completion Status' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "Every landed task has at least one [x] AC and not-started UI tasks have zero [x]",
      "verify": "bash .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/REMEDIATE-06B-A-verify-ac-checks.sh",
      "maps_to_ac": "AC-2,AC-3"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "The Task Detail Files section lists the same number of files as 'ls *.md | wc -l'",
      "verify": "test $(ls .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/*.md | wc -l | tr -d ' ') -eq $(grep -cE '^- [A-Z0-9_-]+.*\\.md$' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md)",
      "maps_to_ac": "AC-4"
    }
  ]
}
-->

