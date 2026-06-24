# REMEDIATE-06B-C: Triage 14 Backlog follow-up tasks

**Type:** REMEDIATION | **Status:** Backlog | **Priority:** P0 | **Effort:** M (90 min)
**Agent:** sveltekit-planner | **Reviewer:** sveltekit-reviewer | **Proposed by:** sveltekit-planner
**Closes red-hat findings:** F7
**Depends on:** REMEDIATE-06B-A | **Blocks:** sprint closure
**PRD refs:** SPRINT.md, .spec/prds/governance/reviews/red-hat-20260623T031824Z-sprint-06b.md | **Capabilities:** CAP-CONFIG-01

## What this does

The sprint directory contains 14 files that are absent from `SPRINT.md`: eight `REMEDIATE-*` tasks, one `REM-DESIGN-*` successor, and four `E2E-*` tasks. The red-hat review noted that this contradicts the claim that 06b is the final POC sprint. This task triages each file into one of three buckets — IN-SPRINT, OUT-SPRINT, or CANCELLED — updates the `SPRINT.md` task table and dependencies, and physically moves OUT-SPRINT files to a new `sprint-06c-governance-followups/` directory. It does not delete work; it makes the sprint boundary honest.

## Why

A sprint plan that silently omits tasks it cannot close is "needs-revision" by definition. The four NOT-STARTED UI tasks (MGMT-UI-009/010/011/012) are the only implementation work required for the Human Testing Gate; the follow-ups are either already superseded by landed work, absorbed by other remediations, or independent hardening for 06c. Triage unblocks honest closure and gives each follow-up a real home.

## Triage Rubric

Per the rubric, only tasks required for a Human Testing Gate step are IN-SPRINT. None of the 14 follow-ups block the gate as currently written, so the disposition set is:

- **REMEDIATE-RUST-1** → **CANCELLED** — the lossless round-trip question is resolved by `REMEDIATE-06B-B`; if the test fails the gap re-opens under `MGMT-BE-004`, not in this file.
- **REMEDIATE-RUST-3** → **CANCELLED** — the admin:read gate at the Tauri wrapper is already landed at HEAD (`crates/gitbutler-tauri/tests/list_workspace_rules_scoped.rs`).
- **REMEDIATE-RUST-5-FOLDED** → **CANCELLED** (already) — folded into `E2E-MGMT-BE-002A` AC-4.
- **REMEDIATE-UI-1** → **CANCELLED** — superseded by `REMEDIATE-06B-D`; the symmetric self-revoke no-flip proof is covered by 06B-D's new ACs.
- **REMEDIATE-UI-2** → **OUT-SPRINT** — build-gate widening is useful perimeter hardening but no Human Testing Gate step requires it; defer to 06c.
- **REMEDIATE-UI-3** → **OUT-SPRINT** — web-target governance route supports the capstone E2E, not the manual Human Testing Gate; defer to 06c.
- **REMEDIATE-UI-4** → **OUT-SPRINT** — verified_by pointers are design-contract hygiene, not gate-critical; defer to 06c.
- **REMEDIATE-UI-5** → **OUT-SPRINT** — stronger pre-click oracle is valuable but not required for the gate; defer to 06c.
- **REMEDIATE-UI-6** → **OUT-SPRINT** — symmetric re-protect AC is useful but no gate step exercises it; defer to 06c.
- **REM-DESIGN-MGMT-004-A** → **OUT-SPRINT** — the successor design contract fixes the U1 wording advisory; the gate can run with the current DESIGN-MGMT-004 annotations; defer to 06c.
- **E2E-MGMT-BE-001** → **OUT-SPRINT** — governed E2E fixtures serve the automated capstone, not the manual Human Testing Gate; defer to 06c.
- **E2E-MGMT-BE-002** → **CANCELLED** — superseded by `E2E-MGMT-BE-002A`; live `but-server` already registers 16 governance routes.
- **E2E-MGMT-BE-002A** → **OUT-SPRINT** — integration tests for the already-registered routes are not required for the manual gate; defer to 06c.
- **E2E-MGMT-UI-001** → **OUT-SPRINT** — the Playwright capstone is the automated successor to the Human Testing Gate, not the manual gate itself; defer to 06c.

CANCELLED files stay in the sprint directory but have their headers updated. OUT-SPRINT files move to `../sprint-06c-governance-followups/`.

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: REMEDIATE-06B-C — Triage 14 Backlog follow-up tasks
================================================================================

TASK_TYPE:   REMEDIATION
STATUS:      Backlog
PRIORITY:    P0
EFFORT:      M  (90 min)
AGENT:       implementer=sveltekit-planner | reviewer=sveltekit-reviewer
PROPOSED-BY: sveltekit-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    SPRINT.md, .spec/prds/governance/reviews/red-hat-20260623T031824Z-sprint-06b.md
CAPABILITIES:CAP-CONFIG-01
CLOSES:      F7

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
The sprint directory contains 14 files that are absent from SPRINT.md: eight REMEDIATE-* tasks, one REM-DESIGN-* successor, and four E2E-* tasks. The red-hat review noted that this contradicts the claim that 06b is the final POC sprint. This task triages each file into one of three buckets — IN-SPRINT, OUT-SPRINT, or CANCELLED — updates the SPRINT.md task table and dependencies, and physically moves OUT-SPRINT files to a new sprint-06c-governance-followups/ directory. It does not delete work; it makes the sprint boundary honest.

Success state: every one of the 14 files has a recorded disposition; OUT-SPRINT files exist in the 06c directory; CANCELLED files have Status: Cancelled and a Reason: in their headers; SPRINT.md Blocks names sprint-06c as successor.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST NOT mark a task IN-SPRINT without naming the specific Human Testing Gate step that requires it.
- [MUST] MUST preserve every file (move or update header; never delete).
- [MUST] MUST update SPRINT.md "Blocks:" line when scope moves to 06c.
- [NEVER] NEVER cancel a task without a one-line `Reason:` pointing at the landed work or successor that subsumes it.
- [STRICTLY] OUT-SPRINT files must physically reside in `../sprint-06c-governance-followups/` after the move.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: Every one of the 14 files has exactly one disposition recorded in a "## Followup Triage" table appended to SPRINT.md (columns: File, Disposition, Reason, Gate-step-required)
- [ ] AC-2: Files marked OUT-SPRINT physically exist in `../sprint-06c-governance-followups/` after the move; files marked IN-SPRINT appear in the SPRINT.md Tasks table; files marked CANCELLED have `Status: Cancelled` and `Reason:` in their header
- [ ] AC-3: The SPRINT.md "Blocks:" line is reconciled — if any task moved to 06c, the line names sprint-06c as the successor
- [ ] AC-4: `ls *.md | wc -l` for the sprint folder + `ls ../sprint-06c-governance-followups/*.md 2>/dev/null | wc -l` sum to >= the original count

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: Every one of the 14 files has exactly one disposition recorded in a "## Followup Triage" table appended to SPRINT.md (columns: File, Disposition, Reason, Gate-step-required)
  GIVEN: 14 follow-up .md files exist but are not in SPRINT.md
  WHEN: the triage table is appended
  THEN: every one of the 14 files appears exactly once with a valid disposition and reason
  TEST_TIER: structural   VERIFICATION_SERVICE: grep
  VERIFY: python3 -c "import os, re; files=sorted(f for f in os.listdir('.') if f.endswith('.md') and f not in ['SPRINT.md']); rows=[l for l in open('SPRINT.md') if l.startswith('|')]; print(len(files), len([r for r in rows if any(f in r for f in files)]))"
  SCENARIO:
    tier: structural   test_tier: structural
    verification_service: grep and python count on SPRINT.md and directory listing
    negative_control.would_fail_if:
      - a follow-up file is omitted from the table
      - a file appears twice with different dispositions
      - the table is missing a Gate-step-required column
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=fourteen_untracked_followups
      action.actor=sveltekit-planner
        - enumerate the 14 follow-up markdown files
        - append a "## Followup Triage" table with columns File, Disposition, Reason, Gate-step-required
        - assign IN-SPRINT, OUT-SPRINT, or CANCELLED to each
      end_state.must_observe:
        - 14 rows in the table (one per follow-up file)
        - each of the 14 filenames appears exactly once
        - dispositions are only IN-SPRINT, OUT-SPRINT, or CANCELLED
      end_state.must_not_observe:
        - missing rows
        - duplicate filenames
        - blank dispositions or reasons

AC-2 : Files marked OUT-SPRINT physically exist in ../sprint-06c-governance-followups/ after the move; files marked IN-SPRINT appear in the SPRINT.md Tasks table; files marked CANCELLED have Status: Cancelled and Reason: in their header
  GIVEN: the triage table is complete
  WHEN: OUT-SPRINT files are moved, CANCELLED headers are updated, and IN-SPRINT files are added to the Tasks table
  THEN: the filesystem and headers reflect the disposition
  TEST_TIER: structural   VERIFICATION_SERVICE: ls + grep
  VERIFY: for f in OUT-SPRINT files: test -f ../sprint-06c-governance-followups/$f; for CANCELLED files: grep -q '^\\*\\*Status:\\*\\* Cancelled' $f && grep -q '^\\*\\*Reason:' $f
  SCENARIO:
    tier: structural   test_tier: structural
    verification_service: ls and grep on moved/updated files
    negative_control.would_fail_if:
      - an OUT-SPRINT file is still in the sprint directory
      - a CANCELLED file header still says Backlog
      - a CANCELLED file has no Reason line
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=fourteen_untracked_followups
      action.actor=sveltekit-planner
        - create ../sprint-06c-governance-followups/ if needed
        - move each OUT-SPRINT file there
        - update CANCELLED file headers to Status: Cancelled with Reason
        - add IN-SPRINT files to the SPRINT.md Tasks table
      end_state.must_observe:
        - every OUT-SPRINT file exists in ../sprint-06c-governance-followups/
        - every CANCELLED file header contains Status: Cancelled and Reason
        - every IN-SPRINT file appears in the Tasks table
      end_state.must_not_observe:
        - OUT-SPRINT files remaining in the sprint directory
        - CANCELLED files still marked Backlog
        - IN-SPRINT files absent from the Tasks table

AC-3 : The SPRINT.md "Blocks:" line is reconciled — if any task moved to 06c, the line names sprint-06c as the successor
  GIVEN: SPRINT.md currently says "Blocks: None (final sprint of the POC roadmap)"
  WHEN: OUT-SPRINT tasks are moved to 06c
  THEN: the Blocks line names sprint-06c as the successor and removes the "final sprint" claim
  TEST_TIER: structural   VERIFICATION_SERVICE: grep
  VERIFY: grep '^- \\*\\*Blocks:\\*\\*' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md
  SCENARIO:
    tier: structural   test_tier: structural
    verification_service: grep on SPRINT.md Dependencies section
    negative_control.would_fail_if:
      - Blocks line still says "None (final sprint of the POC roadmap)"
      - 06c is not referenced anywhere in the Dependencies section
      - the line claims no successor while follow-up files exist in 06c
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=sprint_md_before_triage
      action.actor=sveltekit-planner
        - locate the Blocks line in SPRINT.md
        - replace "None (final sprint of the POC roadmap)" with "sprint-06c-governance-followups (deferred follow-up work)"
      end_state.must_observe:
        - Blocks line references sprint-06c-governance-followups
        - "final sprint" wording is removed
      end_state.must_not_observe:
        - Blocks line claiming no successor
        - "final sprint of the POC roadmap" still present

AC-4 : `ls *.md | wc -l` for the sprint folder + `ls ../sprint-06c-governance-followups/*.md 2>/dev/null | wc -l` sum to >= the original count
  GIVEN: files are moved out of the sprint directory but not deleted
  WHEN: the move is complete
  THEN: the total count of task files across both directories is at least the original count
  TEST_TIER: structural   VERIFICATION_SERVICE: ls + test
  VERIFY: test $(( $(ls .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/*.md 2>/dev/null | wc -l) + $(ls .spec/prds/governance/tasks/sprint-06c-governance-followups/*.md 2>/dev/null | wc -l) )) -ge 27
  SCENARIO:
    tier: structural   test_tier: structural
    verification_service: ls count on both directories
    negative_control.would_fail_if:
      - files were deleted rather than moved
      - the 06c directory was not created
      - the count dropped below the original
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=sprint_directory_with_followups
      action.actor=sveltekit-planner
        - move OUT-SPRINT files to 06c directory
        - leave CANCELLED and IN-SPRINT files in place
      end_state.must_observe:
        - total markdown files across sprint + 06c >= original count (27 at review HEAD)
      end_state.must_not_observe:
        - deleted files
        - count less than original

--------------------------------------------------------------------------------
TEST CRITERIA
--------------------------------------------------------------------------------
- TC-1: The "## Followup Triage" table contains exactly 14 rows with valid dispositions
- TC-2: OUT-SPRINT files exist in ../sprint-06c-governance-followups/
- TC-3: CANCELLED files have Status: Cancelled and a Reason: line
- TC-4: SPRINT.md Blocks line references sprint-06c-governance-followups
- TC-5: Total task file count across both directories is preserved

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
- .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md
- .spec/prds/governance/reviews/red-hat-20260623T031824Z-sprint-06b.md

--------------------------------------------------------------------------------
GUARDRAILS
--------------------------------------------------------------------------------
WRITE_ALLOWED:
  - .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/*.md
  - .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md
  - .spec/prds/governance/tasks/sprint-06c-governance-followups/ (directory creation and file moves)
WRITE_PROHIBITED:
  - source code
  - deletion of any task file

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- grep -c '^## Followup Triage' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md  →  1
- ls .spec/prds/governance/tasks/sprint-06c-governance-followups/*.md 2>/dev/null | wc -l  →  >= 10
- grep '^\\*\\*Status:\\*\\* Cancelled' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/REMEDIATE-RUST-1-*.md  →  match
- grep '^\\*\\*Status:\\*\\* Cancelled' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/REMEDIATE-RUST-3-*.md  →  match
- grep '^\\*\\*Status:\\*\\* Cancelled' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/REMEDIATE-UI-1-*.md  →  match
- grep '^\\*\\*Status:\\*\\* Cancelled' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/E2E-MGMT-BE-002-but-server-governance-routes.md  →  match
- grep '^- \\*\\*Blocks:\\*\\*' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md | grep sprint-06c  →  match

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
depends_on: REMEDIATE-06B-A
blocks:     sprint closure

--------------------------------------------------------------------------------
NOTES
--------------------------------------------------------------------------------
The triage decisions are recorded in the task body so they survive the file. If the implementer disagrees with a disposition, they must re-open this task with an alternative Human Testing Gate step mapping, not silently change the table.
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "REMEDIATE-06B-C",
  "proposed_by": "sveltekit-planner",
  "supersedes": [],
  "closes_redhat_findings": [
    "F7"
  ],
  "fixtures": {
    "fourteen_untracked_followups": {
      "description": "The 14 follow-up files present in the sprint directory but absent from SPRINT.md",
      "seed_method": "component_mount",
      "records": [
        "REMEDIATE-RUST-1-*.md",
        "REMEDIATE-RUST-3-*.md",
        "REMEDIATE-RUST-5-FOLDED-*.md",
        "REMEDIATE-UI-1-*.md",
        "REMEDIATE-UI-2-*.md",
        "REMEDIATE-UI-3-*.md",
        "REMEDIATE-UI-4-*.md",
        "REMEDIATE-UI-5-*.md",
        "REMEDIATE-UI-6-*.md",
        "REM-DESIGN-MGMT-004-A-*.md",
        "E2E-MGMT-BE-001-*.md",
        "E2E-MGMT-BE-002-*.md",
        "E2E-MGMT-BE-002A-*.md",
        "E2E-MGMT-UI-001-*.md"
      ]
    },
    "sprint_md_before_triage": {
      "description": "SPRINT.md with the original Blocks: None claim",
      "seed_method": "component_mount",
      "records": [
        "Blocks: None (final sprint of the POC roadmap)"
      ]
    },
    "sprint_directory_with_followups": {
      "description": "Sprint directory containing original tasks plus the 14 follow-ups",
      "seed_method": "component_mount",
      "records": [
        "27 .md files at review HEAD"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN 14 follow-up .md files exist but are not in SPRINT.md WHEN the triage table is appended THEN every one of the 14 files appears exactly once with a valid disposition and reason",
      "verify": "python3 -c \"import os, re; files=sorted(f for f in os.listdir('.') if f.endswith('.md') and f not in ['SPRINT.md'] and ('REMEDIATE-' in f or 'REM-DESIGN-' in f or 'E2E-' in f)); rows=[l for l in open('SPRINT.md') if l.startswith('|')]; print(len(files), len([r for r in rows if any(f in r for f in files)]))\"",
      "scenario": {
        "tier": "structural",
        "test_tier": "structural",
        "verification_service": "grep and python count on SPRINT.md and directory listing",
        "negative_control": {
          "would_fail_if": [
            "a follow-up file is omitted from the table",
            "a file appears twice with different dispositions",
            "the table is missing a Gate-step-required column"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "fourteen_untracked_followups",
            "action": {
              "actor": "sveltekit-planner",
              "steps": [
                "enumerate the 14 follow-up markdown files",
                "append a '## Followup Triage' table with columns File, Disposition, Reason, Gate-step-required",
                "assign IN-SPRINT, OUT-SPRINT, or CANCELLED to each"
              ]
            },
            "end_state": {
              "must_observe": [
                "14 rows in the table (one per follow-up file)",
                "each of the 14 filenames appears exactly once",
                "dispositions are only IN-SPRINT, OUT-SPRINT, or CANCELLED"
              ],
              "must_not_observe": [
                "missing rows",
                "duplicate filenames",
                "blank dispositions or reasons"
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
      "description": "GIVEN the triage table is complete WHEN OUT-SPRINT files are moved, CANCELLED headers are updated, and IN-SPRINT files are added to the Tasks table THEN the filesystem and headers reflect the disposition",
      "verify": "for f in OUT-SPRINT files: test -f ../sprint-06c-governance-followups/$f; for CANCELLED files: grep -q '^**Status:** Cancelled' $f && grep -q '^**Reason:' $f",
      "scenario": {
        "tier": "structural",
        "test_tier": "structural",
        "verification_service": "ls and grep on moved/updated files",
        "negative_control": {
          "would_fail_if": [
            "an OUT-SPRINT file is still in the sprint directory",
            "a CANCELLED file header still says Backlog",
            "a CANCELLED file has no Reason line"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "fourteen_untracked_followups",
            "action": {
              "actor": "sveltekit-planner",
              "steps": [
                "create ../sprint-06c-governance-followups/ if needed",
                "move each OUT-SPRINT file there",
                "update CANCELLED file headers to Status: Cancelled with Reason",
                "add IN-SPRINT files to the SPRINT.md Tasks table"
              ]
            },
            "end_state": {
              "must_observe": [
                "every OUT-SPRINT file exists in ../sprint-06c-governance-followups/",
                "every CANCELLED file header contains Status: Cancelled and Reason",
                "every IN-SPRINT file appears in the Tasks table"
              ],
              "must_not_observe": [
                "OUT-SPRINT files remaining in the sprint directory",
                "CANCELLED files still marked Backlog",
                "IN-SPRINT files absent from the Tasks table"
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
      "description": "GIVEN SPRINT.md currently says 'Blocks: None (final sprint of the POC roadmap)' WHEN OUT-SPRINT tasks are moved to 06c THEN the Blocks line names sprint-06c as the successor and removes the 'final sprint' claim",
      "verify": "grep '^- \\*\\*Blocks:\\*\\*' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md",
      "scenario": {
        "tier": "structural",
        "test_tier": "structural",
        "verification_service": "grep on SPRINT.md Dependencies section",
        "negative_control": {
          "would_fail_if": [
            "Blocks line still says 'None (final sprint of the POC roadmap)'",
            "06c is not referenced anywhere in the Dependencies section",
            "the line claims no successor while follow-up files exist in 06c"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "sprint_md_before_triage",
            "action": {
              "actor": "sveltekit-planner",
              "steps": [
                "locate the Blocks line in SPRINT.md",
                "replace 'None (final sprint of the POC roadmap)' with 'sprint-06c-governance-followups (deferred follow-up work)'"
              ]
            },
            "end_state": {
              "must_observe": [
                "Blocks line references sprint-06c-governance-followups",
                "'final sprint' wording is removed"
              ],
              "must_not_observe": [
                "Blocks line claiming no successor",
                "'final sprint of the POC roadmap' still present"
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
      "description": "GIVEN files are moved out of the sprint directory but not deleted WHEN the move is complete THEN the total count of task files across both directories is at least the original count",
      "verify": "test $(( $(ls .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/*.md 2>/dev/null | wc -l) + $(ls .spec/prds/governance/tasks/sprint-06c-governance-followups/*.md 2>/dev/null | wc -l) )) -ge 27",
      "scenario": {
        "tier": "structural",
        "test_tier": "structural",
        "verification_service": "ls count on both directories",
        "negative_control": {
          "would_fail_if": [
            "files were deleted rather than moved",
            "the 06c directory was not created",
            "the count dropped below the original"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "sprint_directory_with_followups",
            "action": {
              "actor": "sveltekit-planner",
              "steps": [
                "move OUT-SPRINT files to 06c directory",
                "leave CANCELLED and IN-SPRINT files in place"
              ]
            },
            "end_state": {
              "must_observe": [
                "total markdown files across sprint + 06c >= original count (27 at review HEAD)"
              ],
              "must_not_observe": [
                "deleted files",
                "count less than original"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "The '## Followup Triage' table contains exactly 14 rows with valid dispositions",
      "verify": "python3 -c \"import os; files=sorted(f for f in os.listdir('.') if f.endswith('.md') and f!='SPRINT.md' and ('REMEDIATE-' in f or 'REM-DESIGN-' in f or 'E2E-' in f)); rows=[l for l in open('SPRINT.md') if l.startswith('|')]; assert len([r for r in rows if any(f in r for f in files)])==14\"",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "OUT-SPRINT files exist in ../sprint-06c-governance-followups/",
      "verify": "test -d .spec/prds/governance/tasks/sprint-06c-governance-followups && test $(ls .spec/prds/governance/tasks/sprint-06c-governance-followups/*.md 2>/dev/null | wc -l) -ge 10",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "CANCELLED files have Status: Cancelled and a Reason: line",
      "verify": "grep -q '^\\*\\*Status:\\*\\* Cancelled' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/REMEDIATE-RUST-1-*.md && grep -q '^\\*\\*Reason:' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/REMEDIATE-RUST-1-*.md",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "SPRINT.md Blocks line references sprint-06c-governance-followups",
      "verify": "grep '^- \\*\\*Blocks:\\*\\*' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md | grep -q 'sprint-06c'",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "Total task file count across both directories is preserved",
      "verify": "test $(( $(ls .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/*.md 2>/dev/null | wc -l) + $(ls .spec/prds/governance/tasks/sprint-06c-governance-followups/*.md 2>/dev/null | wc -l) )) -ge 27",
      "maps_to_ac": "AC-4"
    }
  ]
}
-->
