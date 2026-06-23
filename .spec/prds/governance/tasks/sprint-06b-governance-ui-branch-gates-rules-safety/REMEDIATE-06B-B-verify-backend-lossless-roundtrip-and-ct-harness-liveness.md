
# REMEDIATE-06B-B: Verify backend lossless round-trip and CT harness liveness

**Type:** REMEDIATION | **Status:** Backlog | **Priority:** P0 | **Effort:** S (60 min)
**Agent:** rust-reviewer / sveltekit-reviewer | **Reviewer:** sveltekit-reviewer | **Proposed by:** sveltekit-planner
**Closes red-hat findings:** F8, F9
**Depends on:** (none, can run in parallel with A) | **Blocks:** MGMT-BE-004 AC-3 close, MGMT-UI-009 CT preflight
**PRD refs:** UC-MGMT-04, UC-MGMT-06, 11-e2e-testing-criteria.md | **Capabilities:** CAP-AUTHZ-01, CAP-CONFIG-01

## What this does

Two assumptions in the sprint are unverified: whether `MGMT-BE-004` actually round-trips the full `[[gate]]` set losslessly, and whether the Sprint-06a desktop CT harness is still live. This task runs the two commands that settle those questions at HEAD, captures the actual stdout, and appends it to the relevant contracts. If `cargo test -p but-api branch_gates` passes, `MGMT-BE-004` AC-3 can be checked off; if it doesn't, we record the failure instead of pretending it passes. If `pnpm test:ct -- Governance` runs at least one test, the UI tasks may proceed; if it runs zero, the harness gap is made explicit.

## Why

The red-hat review marked MGMT-BE-004 AC-3 PARTIAL because the lossless round-trip was not demonstrated, and it noted that the CT harness inherited from Sprint 06a was assumed but not re-checked. We cannot close a sprint on assumptions, and we cannot mark ACs PASS from static code inspection alone. This remediation produces the captured evidence the ledger needs.

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: REMEDIATE-06B-B — Verify backend lossless round-trip and CT harness liveness
================================================================================

TASK_TYPE:   REMEDIATION
STATUS:      Backlog
PRIORITY:    P0
EFFORT:      S  (60 min)
AGENT:       implementer=rust-reviewer | reviewer=sveltekit-reviewer
PROPOSED-BY: sveltekit-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-MGMT-04, UC-MGMT-06, 11-e2e-testing-criteria.md
CAPABILITIES:CAP-AUTHZ-01,CAP-CONFIG-01
CLOSES:      F8, F9

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Two assumptions in the sprint are unverified: whether MGMT-BE-004 actually round-trips the full [[gate]] set losslessly, and whether the Sprint-06a desktop CT harness is still live. This task runs the two commands that settle those questions at HEAD, captures the actual stdout, and appends it to the relevant contracts. If `cargo test -p but-api branch_gates` passes, MGMT-BE-004 AC-3 can be checked off; if it doesn't, we record the failure instead of pretending it passes. If `pnpm test:ct -- Governance` runs at least one test, the UI tasks may proceed; if it runs zero, the harness gap is made explicit.

Success state: MGMT-BE-004 AC-3 is annotated with the verbatim cargo output and checked off only if the test run passes; REMEDIATE-RUST-1 status is recorded explicitly; SPRINT.md has a "Verification Evidence" section containing both verbatim command outputs.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST NOT modify source code. Only run commands, capture output, and append evidence to task files.
- [MUST] MUST run on HEAD without uncommitted changes affecting the test set.
- [MUST] MUST record the actual command output, not a paraphrase.
- [NEVER] NEVER mark an AC PASS without a captured passing test run.
- [STRICTLY] If a test fails, leave the relevant AC `[ ]` and append the failure output; do not sanitize.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: `cargo test -p but-api branch_gates` runs to completion; output captured in MGMT-BE-004 AC-3 annotation; if pass, AC-3 → `[x]`; if fail, AC-3 stays `[ ]` with failure output annotated
- [ ] AC-2: REMEDIATE-RUST-1 status (Done/Backlog/Superseded) is recorded explicitly in SPRINT.md Completion Status section with the cargo test evidence pointer
- [ ] AC-3 [PRIMARY]: `pnpm test:ct -- Governance` runs to completion; output captured in SPRINT.md Verification Evidence section; if ≥1 test ran, harness confirmed live; if 0 tests ran, harness gap recorded
- [ ] AC-4: A "## Verification Evidence" section exists in SPRINT.md with both command outputs verbatim

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: `cargo test -p but-api branch_gates` runs to completion; output captured in MGMT-BE-004 AC-3 annotation; if pass, AC-3 -> [x]; if fail, AC-3 stays [ ] with failure output annotated
  GIVEN: MGMT-BE-004 AC-3 is PARTIAL because lossless round-trip was not verified
  WHEN: `cargo test -p but-api branch_gates` is run at HEAD and the output is captured
  THEN: the MGMT-BE-004 AC-3 annotation contains the verbatim output; if exit 0 and tests pass the box is checked, otherwise it remains unchecked with failure output
  TEST_TIER: integration   VERIFICATION_SERVICE: cargo
  VERIFY: cargo test -p but-api branch_gates
  SCENARIO:
    tier: visible   test_tier: integration
    verification_service: cargo test -p but-api branch_gates
    negative_control.would_fail_if:
      - the failure output is summarized rather than appended verbatim
      - AC-3 is marked [x] without a passing run
      - source code is edited to make the test pass
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=HEAD_of_design_lpr_ui_design_contracts_f7a1589c6c
      action.actor=rust-reviewer
        - ensure a clean worktree (tests only)
        - run cargo test -p but-api branch_gates
        - capture the full stdout and exit code
        - append the verbatim output to MGMT-BE-004 AC-3
      end_state.must_observe:
        - exit code is recorded
        - if exit 0 and at least one test ran, MGMT-BE-004 AC-3 is [x]
        - if non-zero or no tests ran, MGMT-BE-004 AC-3 remains [ ] and includes the failure output
      end_state.must_not_observe:
        - paraphrased evidence
        - unchecked AC with no explanation
        - source-code changes

AC-2 : REMEDIATE-RUST-1 status (Done/Backlog/Superseded) is recorded explicitly in SPRINT.md Completion Status section with the cargo test evidence pointer
  GIVEN: REMEDIATE-RUST-1 is Backlog and its relationship to MGMT-BE-004 is unclear
  WHEN: the cargo test evidence is available
  THEN: SPRINT.md Completion Status records REMEDIATE-RUST-1 as Done, Backlog, or Superseded and points to the MGMT-BE-004 AC-3 evidence
  TEST_TIER: structural   VERIFICATION_SERVICE: grep
  VERIFY: grep -A2 'REMEDIATE-RUST-1' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md
  SCENARIO:
    tier: structural   test_tier: structural
    verification_service: grep on SPRINT.md Completion Status section
    negative_control.would_fail_if:
      - REMEDIATE-RUST-1 is omitted from the Completion Status table
      - status is set to Done without evidence that the round-trip widened GatesWire task is no longer needed
      - there is no pointer to the cargo test output
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=SPRINT_md_completion_status
      action.actor=rust-reviewer
        - open SPRINT.md Completion Status section
        - set REMEDIATE-RUST-1 status: Done if cargo test passes and the widening is visible in the writer; Backlog if the test fails; or Superseded by MGMT-BE-004 if the writer already includes the widening
      end_state.must_observe:
        - row for REMEDIATE-RUST-1 names a concrete status
        - row includes a reference to the cargo test evidence (MGMT-BE-004 AC-3)
      end_state.must_not_observe:
        - status left as Backlog without explanation
        - no evidence pointer

AC-3 [PRIMARY]: `pnpm test:ct -- Governance` runs to completion; output captured in SPRINT.md Verification Evidence section; if >=1 test ran, harness confirmed live; if 0 tests ran, harness gap recorded
  GIVEN: the desktop CT harness from Sprint 06a is assumed but not re-checked
  WHEN: `pnpm test:ct -- Governance` is run at HEAD and the output is captured
  THEN: SPRINT.md Verification Evidence contains the verbatim output and a note stating whether >=1 test ran
  TEST_TIER: integration   VERIFICATION_SERVICE: pnpm test:ct
  VERIFY: pnpm test:ct -- Governance
  SCENARIO:
    tier: visible   test_tier: integration
    verification_service: pnpm test:ct -- Governance
    negative_control.would_fail_if:
      - the run output is skipped
      - a different CT filter is used that does not target governance tests
      - source code is changed to fabricate test results
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=clean_worktree_with_ct_dependencies
      action.actor=sveltekit-reviewer
        - run pnpm test:ct -- Governance
        - capture stdout and whether any tests executed
        - append the verbatim output to SPRINT.md Verification Evidence
      end_state.must_observe:
        - if one or more tests ran, the note says 'CT harness confirmed live: N test(s) ran'
        - if zero tests ran, the note says 'CT harness gap: 0 tests found'
        - the verbatim command output is present
      end_state.must_not_observe:
        - a fabricated summary
        - no output recorded

AC-4 : A "## Verification Evidence" section exists in SPRINT.md with both command outputs verbatim
  GIVEN: the two verifications produce command output
  WHEN: the outputs are appended to SPRINT.md
  THEN: a single "## Verification Evidence" section contains both verbatim outputs
  TEST_TIER: structural   VERIFICATION_SERVICE: grep
  VERIFY: grep -c '^## Verification Evidence' SPRINT.md
  SCENARIO:
    tier: structural   test_tier: structural
    verification_service: grep on SPRINT.md
    negative_control.would_fail_if:
      - the section is missing
      - outputs are split across unrelated sections
      - outputs are edited
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=sprint_md_after_a
      action.actor=sveltekit-reviewer
        - create one '## Verification Evidence' section in SPRINT.md
        - paste both command outputs inside fenced code blocks
      end_state.must_observe:
        - grep -c '^## Verification Evidence' SPRINT.md returns 1
        - section contains a fenced block for cargo output
        - section contains a fenced block for pnpm test:ct output
      end_state.must_not_observe:
        - zero sections
        - more than one Verification Evidence section
        - paraphrased command output

--------------------------------------------------------------------------------
TEST CRITERIA
--------------------------------------------------------------------------------
- TC-1: `cargo test -p but-api branch_gates` exits 0 and at least one test runs, or its failure output is recorded
- TC-2: MGMT-BE-004 AC-3 is checked off only if TC-1 passed
- TC-3: `pnpm test:ct -- Governance` output is captured and the test count is recorded
- TC-4: SPRINT.md contains exactly one "Verification Evidence" section with both captured outputs

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
- .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-BE-004-branch-gates-config-writer.md
- .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/REMEDIATE-RUST-1-widen-but-authz-gateswire-to-accept-the-full-gate-schema-and.md
- .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md

--------------------------------------------------------------------------------
GUARDRAILS
--------------------------------------------------------------------------------
WRITE_ALLOWED:
  - .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-BE-004*.md
  - .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md
WRITE_PROHIBITED:
  - all source code

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- cargo test -p but-api branch_gates  →  capture stdout and exit code
- pnpm test:ct -- Governance  →  capture stdout and test count
- grep -c '^## Verification Evidence' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md  →  1

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
depends_on: (none, can run in parallel with A)
blocks:     MGMT-BE-004 AC-3 close, MGMT-UI-009 CT preflight

--------------------------------------------------------------------------------
NOTES
--------------------------------------------------------------------------------
If `cargo test -p but-api branch_gates` does not compile or has no matching tests, that is itself evidence and must be recorded. The CT harness may be located at the desktop package; if the exact invocation differs, record the command actually used.
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "REMEDIATE-06B-B",
  "proposed_by": "sveltekit-planner",
  "supersedes": [],
  "closes_redhat_findings": [
    "F8",
    "F9"
  ],
  "fixtures": {
    "HEAD_of_design_lpr_ui_design_contracts_f7a1589c6c": {
      "description": "Clean worktree at the reviewed HEAD with no uncommitted test-affecting changes",
      "seed_method": "component_mount",
      "records": [
        "git status --short is empty"
      ]
    },
    "SPRINT_md_completion_status": {
      "description": "SPRINT.md after REMEDIATE-06B-A has added the Completion Status section",
      "seed_method": "component_mount",
      "records": [
        "Completion Status (HEAD f7a1589c6c) exists"
      ]
    },
    "clean_worktree_with_ct_dependencies": {
      "description": "Workspace with node_modules/pnpm available and the desktop CT harness present",
      "seed_method": "component_mount",
      "records": [
        "pnpm installed",
        "apps/desktop Playwright CT config present"
      ]
    },
    "sprint_md_after_a": {
      "description": "SPRINT.md after REMEDIATE-06B-A bookkeeping",
      "seed_method": "component_mount",
      "records": [
        "status: In Progress",
        "Completion Status section present"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN MGMT-BE-004 AC-3 is PARTIAL because lossless round-trip was not verified WHEN cargo test -p but-api branch_gates is run at HEAD and the output is captured THEN the MGMT-BE-004 AC-3 annotation contains the verbatim output; if exit 0 and tests pass the box is checked, otherwise it remains unchecked with failure output",
      "verify": "cargo test -p but-api branch_gates",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "cargo test -p but-api branch_gates",
        "negative_control": {
          "would_fail_if": [
            "the failure output is summarized rather than appended verbatim",
            "AC-3 is marked [x] without a passing run",
            "source code is edited to make the test pass"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "HEAD_of_design_lpr_ui_design_contracts_f7a1589c6c",
            "action": {
              "actor": "rust-reviewer",
              "steps": [
                "ensure a clean worktree (tests only)",
                "run cargo test -p but-api branch_gates",
                "capture the full stdout and exit code",
                "append the verbatim output to MGMT-BE-004 AC-3"
              ]
            },
            "end_state": {
              "must_observe": [
                "exit code is recorded",
                "if exit 0 and at least one test ran, MGMT-BE-004 AC-3 is [x]",
                "if non-zero or no tests ran, MGMT-BE-004 AC-3 remains [ ] and includes the failure output"
              ],
              "must_not_observe": [
                "paraphrased evidence",
                "unchecked AC with no explanation",
                "source-code changes"
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
      "description": "GIVEN REMEDIATE-RUST-1 is Backlog and its relationship to MGMT-BE-004 is unclear WHEN the cargo test evidence is available THEN SPRINT.md Completion Status records REMEDIATE-RUST-1 as Done, Backlog, or Superseded and points to the MGMT-BE-004 AC-3 evidence",
      "verify": "grep -A2 'REMEDIATE-RUST-1' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md",
      "scenario": {
        "tier": "structural",
        "test_tier": "structural",
        "verification_service": "grep on SPRINT.md Completion Status section",
        "negative_control": {
          "would_fail_if": [
            "REMEDIATE-RUST-1 is omitted from the Completion Status table",
            "status is set to Done without evidence that the round-trip widened GatesWire task is no longer needed",
            "there is no pointer to the cargo test output"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "SPRINT_md_completion_status",
            "action": {
              "actor": "rust-reviewer",
              "steps": [
                "open SPRINT.md Completion Status section",
                "set REMEDIATE-RUST-1 status: Done if cargo test passes and the widening is visible in the writer; Backlog if the test fails; or Superseded by MGMT-BE-004 if the writer already includes the widening"
              ]
            },
            "end_state": {
              "must_observe": [
                "row for REMEDIATE-RUST-1 names a concrete status",
                "row includes a reference to the cargo test evidence (MGMT-BE-004 AC-3)"
              ],
              "must_not_observe": [
                "status left as Backlog without explanation",
                "no evidence pointer"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN the desktop CT harness from Sprint 06a is assumed but not re-checked WHEN pnpm test:ct -- Governance is run at HEAD and the output is captured THEN SPRINT.md Verification Evidence contains the verbatim output and a note stating whether >=1 test ran",
      "verify": "pnpm test:ct -- Governance",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "pnpm test:ct -- Governance",
        "negative_control": {
          "would_fail_if": [
            "the run output is skipped",
            "a different CT filter is used that does not target governance tests",
            "source code is changed to fabricate test results"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "clean_worktree_with_ct_dependencies",
            "action": {
              "actor": "sveltekit-reviewer",
              "steps": [
                "run pnpm test:ct -- Governance",
                "capture stdout and whether any tests executed",
                "append the verbatim output to SPRINT.md Verification Evidence"
              ]
            },
            "end_state": {
              "must_observe": [
                "if one or more tests ran, the note says 'CT harness confirmed live: N test(s) ran'",
                "if zero tests ran, the note says 'CT harness gap: 0 tests found'",
                "the verbatim command output is present"
              ],
              "must_not_observe": [
                "a fabricated summary",
                "no output recorded"
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
      "description": "GIVEN the two verifications produce command output WHEN the outputs are appended to SPRINT.md THEN a single '## Verification Evidence' section contains both verbatim outputs",
      "verify": "grep -c '^## Verification Evidence' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md",
      "scenario": {
        "tier": "structural",
        "test_tier": "structural",
        "verification_service": "grep on SPRINT.md",
        "negative_control": {
          "would_fail_if": [
            "the section is missing",
            "outputs are split across unrelated sections",
            "outputs are edited"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "sprint_md_after_a",
            "action": {
              "actor": "sveltekit-reviewer",
              "steps": [
                "create one '## Verification Evidence' section in SPRINT.md",
                "paste both command outputs inside fenced code blocks"
              ]
            },
            "end_state": {
              "must_observe": [
                "grep -c '^## Verification Evidence' SPRINT.md returns 1",
                "section contains a fenced block for cargo output",
                "section contains a fenced block for pnpm test:ct output"
              ],
              "must_not_observe": [
                "zero sections",
                "more than one Verification Evidence section",
                "paraphrased command output"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "cargo test -p but-api branch_gates exits 0 and at least one test runs, or its failure output is recorded",
      "verify": "cargo test -p but-api branch_gates",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "MGMT-BE-004 AC-3 is checked off only if TC-1 passed",
      "verify": "grep -B2 -A6 'AC-3' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-BE-004*.md | grep -q '\\- \\[x\\]'",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "pnpm test:ct -- Governance output is captured and the test count is recorded",
      "verify": "grep -A20 '## Verification Evidence' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md | grep -q 'pnpm test:ct -- Governance'",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "SPRINT.md contains exactly one 'Verification Evidence' section with both captured outputs",
      "verify": "test $(grep -c '^## Verification Evidence' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md) -eq 1",
      "maps_to_ac": "AC-4"
    }
  ]
}
-->

