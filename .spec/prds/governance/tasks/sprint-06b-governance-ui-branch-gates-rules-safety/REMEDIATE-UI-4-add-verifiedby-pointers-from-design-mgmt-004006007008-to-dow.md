# REMEDIATE-UI-4: Add verified_by pointers from DESIGN-MGMT-004/006/007/008 to downstream UI ACs

**Type:** REMEDIATION | **Status:** Backlog | **Priority:** P0 | **Effort:** S (60 min)
**Agent:** frontend-designer | **Reviewer:** sveltekit-reviewer | **Proposed by:** sveltekit-planner
**Closes red-hat findings:** L8
**Depends on:** MGMT-UI-009, MGMT-UI-010, MGMT-UI-011, E2E-MGMT-UI-001 | **Blocks:** (none)
**PRD refs:** UC-MGMT-04, 10-ui-infrastructure.md | **Capabilities:** CAP-AUTHZ-01, CAP-A11Y-01, CAP-CONFIG-01

## What this does

Design contracts DESIGN-MGMT-004, DESIGN-MGMT-006, DESIGN-MGMT-007, and DESIGN-MGMT-008 currently verify by design review only. This creates Category 4 Test Theatre risk: if UI implementations drift, no DESIGN-owned test catches the violation. Add a verified_by field to each behavioral AC in the four design contracts that names the specific downstream UI task and AC that exercises the contract. Add a CI lint gate that fails if any DESIGN task.md AC lacks a verified_by block.

## Why

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: REMEDIATE-UI-4 — Add verified_by pointers from DESIGN-MGMT-004/006/007/008 to downstream UI ACs
================================================================================

TASK_TYPE:   REMEDIATION
STATUS:      Backlog
PRIORITY:    P0
EFFORT:      S  (60 min)
AGENT:       implementer=frontend-designer | reviewer=sveltekit-reviewer
PROPOSED-BY: sveltekit-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-MGMT-04, 10-ui-infrastructure.md
CAPABILITIES:CAP-AUTHZ-01,CAP-A11Y-01,CAP-CONFIG-01
CLOSES:      L8

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Design contracts DESIGN-MGMT-004, DESIGN-MGMT-006, DESIGN-MGMT-007, and DESIGN-MGMT-008 currently verify by design review only. This creates Category 4 Test Theatre risk: if UI implementations drift, no DESIGN-owned test catches the violation. Add a verified_by field to each behavioral AC in the four design contracts that names the specific downstream UI task and AC that exercises the contract. Add a CI lint gate that fails if any DESIGN task.md AC lacks a verified_by block.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] Every DESIGN AC that makes a behavioral claim must point to at least one concrete UI AC that validates it.
- [MUST] verified_by pointers must be stable task/AC references, not prose.
- [MUST] No implementation files are modified; edits are limited to the four design-contract markdown files.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: each of those ACs references exactly one executable UI acceptance criterion
- [ ] AC-2: the empty-state ACs are tied to concrete component tests
- [ ] AC-3: each a11y claim maps to an executable test
- [ ] AC-4: the design contract's resilience claims are traced to component tests
- [ ] AC-5: the gate exits non-zero and lists the missing entries

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: each of those ACs references exactly one executable UI acceptance criterion
  GIVEN: DESIGN-MGMT-004 AC-2/AC-3 describe denial/no-flip behavior
  WHEN: the contract is amended with verified_by pointers to MGMT-UI-009 AC-7 and E2E-MGMT-UI-001 AC-4
  THEN: each of those ACs references exactly one executable UI acceptance criterion
  TEST_TIER: structural   VERIFICATION_SERVICE: contract-audit
  SCENARIO:
    tier: structural   test_tier: structural
    verification_service: python3 scripts/lint_design_contracts.py (parse DESIGN-MGMT-004/006/007/008 REQUIREMENT-CONTRACT blocks)
    negative_control.would_fail_if:
      - verified_by cites only 'design review' (Test Theatre)
      - verified_by pointer references a non-existent AC ID
      - a behavioral AC lacks the verified_by field
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=contracts_without_pointers
      action.actor=ci
        - parse the four DESIGN-MGMT-*.md REQUIREMENT-CONTRACT blocks
        - count behavioral ACs
        - count behavioral ACs with non-empty verified_by blocks
      end_state.must_observe:
        - behavioral AC count equals verified-by-present count
        - every verified_by block contains task_id and ac_id fields
        - zero behavioral ACs have an empty or prose-only verified_by
      end_state.must_not_observe:
        - verified_by value equal to 'design review'
        - verified_by ac_id that does not exist in the referenced task
        - behavioral AC with missing verified_by

AC-2 : the empty-state ACs are tied to concrete component tests
  GIVEN: DESIGN-MGMT-006 defines empty-state copy and affordances
  WHEN: verified_by pointers are added to MGMT-UI-009 AC-3 and MGMT-UI-010 AC-4
  THEN: the empty-state ACs are tied to concrete component tests
  TEST_TIER: structural   VERIFICATION_SERVICE: contract-audit
  SCENARIO:
    tier: structural   test_tier: structural
    verification_service: python3 scripts/lint_design_contracts.py --file .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/DESIGN-MGMT-006*.md
    negative_control.would_fail_if:
      - empty-state ACs point to a non-empty-state UI AC
      - verified_by references a UI task that is not in the same sprint
      - the pointer is omitted for ACs that describe button states
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=design_006_contract
      action.actor=ci
        - parse DESIGN-MGMT-006 empty-state ACs
        - assert each maps to a non-empty UI AC via verified_by task_id/ac_id
      end_state.must_observe:
        - Branch Gates empty-state AC points to MGMT-UI-009 AC-3
        - Rules empty-state AC points to MGMT-UI-010 AC-4
      end_state.must_not_observe:
        - pointers to UI ACs outside this sprint
        - missing verified_by on any AC that asserts visible copy

AC-3 : each a11y claim maps to an executable test
  GIVEN: DESIGN-MGMT-007 defines tab roles, keyboard flow, and banner priority
  WHEN: verified_by pointers are added to MGMT-UI-011 AC-4 and E2E-MGMT-UI-001 AC-2
  THEN: each a11y claim maps to an executable test
  TEST_TIER: structural   VERIFICATION_SERVICE: contract-audit
  SCENARIO:
    tier: structural   test_tier: structural
    verification_service: python3 scripts/lint_design_contracts.py --file .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/DESIGN-MGMT-007*.md
    negative_control.would_fail_if:
      - keyboard navigation claims lack an E2E or CT pointer
      - banner priority claim points to a test that never renders multiple banners
      - role/tablist claims point to screenshots instead of DOM assertions
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=design_007_contract
      action.actor=ci
        - parse DESIGN-MGMT-007 a11y ACs
        - assert tab roles, keyboard flow, and banner priority map to verified_by pointers
      end_state.must_observe:
        - tab role/aria contract points to MGMT-UI-011 AC-4
        - keyboard navigation contract points to E2E-MGMT-UI-001 AC-2
      end_state.must_not_observe:
        - a11y ACs with only design-review verification
        - pointers to tests that do not exercise keyboard/role assertions

AC-4 : the design contract's resilience claims are traced to component tests
  GIVEN: DESIGN-MGMT-008 defines ErrorBoundary and retry behavior
  WHEN: verified_by pointers are added to MGMT-UI-011 AC-3 (error boundary) and AC-5 (retry)
  THEN: the design contract's resilience claims are traced to component tests
  TEST_TIER: structural   VERIFICATION_SERVICE: contract-audit
  SCENARIO:
    tier: structural   test_tier: structural
    verification_service: python3 scripts/lint_design_contracts.py --file .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/DESIGN-MGMT-008*.md
    negative_control.would_fail_if:
      - ErrorBoundary AC points to a test that never throws
      - retry AC points to a unit test instead of a visible retry flow
      - ACs reference renamed or removed MGMT-UI-011 AC numbers
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=design_008_contract
      action.actor=ci
        - parse DESIGN-MGMT-008 error-boundary and retry ACs
        - assert each maps to a visible component test via verified_by
      end_state.must_observe:
        - error-boundary AC points to MGMT-UI-011 AC-3
        - retry AC points to MGMT-UI-011 AC-5
      end_state.must_not_observe:
        - references to nonexistent AC numbers
        - behavioral ACs left without verified_by

AC-5 : the gate exits non-zero and lists the missing entries
  GIVEN: a design contract AC without a verified_by block
  WHEN: the design-contract lint gate runs
  THEN: the gate exits non-zero and lists the missing entries
  TEST_TIER: structural   VERIFICATION_SERVICE: desktop-build-gate
  SCENARIO:
    tier: structural   test_tier: structural
    verification_service: python3 scripts/lint_design_contracts.py --negative-control DESIGN-MGMT-004*.md
    negative_control.would_fail_if:
      - the lint gate exits 0 despite a missing verified_by
      - the gate reports failure but does not name the AC
      - the gate flags non-behavioral ACs (e.g. purely decorative notes)
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=patched_design_contract
      action.actor=ci
        - create a temporary stripped copy of one design AC
        - run lint_design_contracts.py
        - capture non-zero exit and reported AC list
      end_state.must_observe:
        - linter exit code non-zero
        - output contains the stripped AC id
      end_state.must_not_observe:
        - linter exit code 0
        - no mention of the offending AC

--------------------------------------------------------------------------------
TEST CRITERIA
--------------------------------------------------------------------------------
- TC-1: DESIGN-MGMT-004.md contains verified_by blocks on all behavioral ACs
- TC-2: DESIGN-MGMT-006.md contains verified_by blocks on all behavioral ACs
- TC-3: DESIGN-MGMT-007.md contains verified_by blocks on all behavioral ACs
- TC-4: DESIGN-MGMT-008.md contains verified_by blocks on all behavioral ACs
- TC-5: A deliberately stripped verified_by block trips the design-contract lint gate

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
- python3 scripts/lint_design_contracts.py .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/DESIGN-MGMT-004*.md  →  ?
- python3 scripts/lint_design_contracts.py .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/DESIGN-MGMT-006*.md  →  ?
- python3 scripts/lint_design_contracts.py .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/DESIGN-MGMT-007*.md  →  ?
- python3 scripts/lint_design_contracts.py .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/DESIGN-MGMT-008*.md  →  ?

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
depends_on: MGMT-UI-009, MGMT-UI-010, MGMT-UI-011, E2E-MGMT-UI-001
blocks:     (none)

--------------------------------------------------------------------------------
NOTES
--------------------------------------------------------------------------------
This remediation closes the L8 finding. The linter is a new light-weight gate scoped to .spec/prds/governance/tasks/.../DESIGN-MGMT-*.md. If the script path is absent, the implementer should create scripts/lint_design_contracts.py using the markdown frontmatter/AC parser.

```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "REMEDIATE-UI-4",
  "proposed_by": "sveltekit-planner",
  "supersedes": [],
  "closes_redhat_findings": [
    "L8"
  ],
  "fixtures": {
    "contracts_without_pointers": {
      "description": "The four DESIGN-MGMT contract files before verified_by pointers are added",
      "seed_method": "component_mount",
      "records": [
        "DESIGN-MGMT-004*.md",
        "DESIGN-MGMT-006*.md",
        "DESIGN-MGMT-007*.md",
        "DESIGN-MGMT-008*.md"
      ]
    },
    "design_006_contract": {
      "description": "DESIGN-MGMT-006.md containing empty-state copy and affordance ACs",
      "seed_method": "component_mount",
      "records": [
        "empty-state copy for Branch Gates",
        "empty-state copy for Rules"
      ]
    },
    "design_007_contract": {
      "description": "DESIGN-MGMT-007.md containing four-tab a11y, keyboard flow, and banner priority ACs",
      "seed_method": "component_mount",
      "records": [
        "tab role contract",
        "keyboard navigation contract",
        "banner priority contract"
      ]
    },
    "design_008_contract": {
      "description": "DESIGN-MGMT-008.md containing ErrorBoundary and retry behavior ACs",
      "seed_method": "component_mount",
      "records": [
        "ErrorBoundary AC",
        "retry AC"
      ]
    },
    "patched_design_contract": {
      "description": "A temporary copy of a design contract with one AC stripped of verified_by for the lint negative control",
      "seed_method": "component_mount",
      "records": [
        "one behavioral AC missing verified_by"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN DESIGN-MGMT-004 AC-2/AC-3 describe denial/no-flip behavior WHEN the contract is amended with verified_by pointers to MGMT-UI-009 AC-7 and E2E-MGMT-UI-001 AC-4 THEN each of those ACs references exactly one executable UI acceptance criterion",
      "verify": "",
      "scenario": {
        "tier": "structural",
        "test_tier": "structural",
        "verification_service": "python3 scripts/lint_design_contracts.py (parse DESIGN-MGMT-004/006/007/008 REQUIREMENT-CONTRACT blocks)",
        "negative_control": {
          "would_fail_if": [
            "verified_by cites only 'design review' (Test Theatre)",
            "verified_by pointer references a non-existent AC ID",
            "a behavioral AC lacks the verified_by field"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "contracts_without_pointers",
            "action": {
              "actor": "ci",
              "steps": [
                "parse the four DESIGN-MGMT-*.md REQUIREMENT-CONTRACT blocks",
                "count behavioral ACs",
                "count behavioral ACs with non-empty verified_by blocks"
              ]
            },
            "end_state": {
              "must_observe": [
                "behavioral AC count equals verified-by-present count",
                "every verified_by block contains task_id and ac_id fields",
                "zero behavioral ACs have an empty or prose-only verified_by"
              ],
              "must_not_observe": [
                "verified_by value equal to 'design review'",
                "verified_by ac_id that does not exist in the referenced task",
                "behavioral AC with missing verified_by"
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
      "description": "GIVEN DESIGN-MGMT-006 defines empty-state copy and affordances WHEN verified_by pointers are added to MGMT-UI-009 AC-3 and MGMT-UI-010 AC-4 THEN the empty-state ACs are tied to concrete component tests",
      "verify": "",
      "scenario": {
        "tier": "structural",
        "test_tier": "structural",
        "verification_service": "python3 scripts/lint_design_contracts.py --file .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/DESIGN-MGMT-006*.md",
        "negative_control": {
          "would_fail_if": [
            "empty-state ACs point to a non-empty-state UI AC",
            "verified_by references a UI task that is not in the same sprint",
            "the pointer is omitted for ACs that describe button states"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "design_006_contract",
            "action": {
              "actor": "ci",
              "steps": [
                "parse DESIGN-MGMT-006 empty-state ACs",
                "assert each maps to a non-empty UI AC via verified_by task_id/ac_id"
              ]
            },
            "end_state": {
              "must_observe": [
                "Branch Gates empty-state AC points to MGMT-UI-009 AC-3",
                "Rules empty-state AC points to MGMT-UI-010 AC-4"
              ],
              "must_not_observe": [
                "pointers to UI ACs outside this sprint",
                "missing verified_by on any AC that asserts visible copy"
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
      "description": "GIVEN DESIGN-MGMT-007 defines tab roles, keyboard flow, and banner priority WHEN verified_by pointers are added to MGMT-UI-011 AC-4 and E2E-MGMT-UI-001 AC-2 THEN each a11y claim maps to an executable test",
      "verify": "",
      "scenario": {
        "tier": "structural",
        "test_tier": "structural",
        "verification_service": "python3 scripts/lint_design_contracts.py --file .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/DESIGN-MGMT-007*.md",
        "negative_control": {
          "would_fail_if": [
            "keyboard navigation claims lack an E2E or CT pointer",
            "banner priority claim points to a test that never renders multiple banners",
            "role/tablist claims point to screenshots instead of DOM assertions"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "design_007_contract",
            "action": {
              "actor": "ci",
              "steps": [
                "parse DESIGN-MGMT-007 a11y ACs",
                "assert tab roles, keyboard flow, and banner priority map to verified_by pointers"
              ]
            },
            "end_state": {
              "must_observe": [
                "tab role/aria contract points to MGMT-UI-011 AC-4",
                "keyboard navigation contract points to E2E-MGMT-UI-001 AC-2"
              ],
              "must_not_observe": [
                "a11y ACs with only design-review verification",
                "pointers to tests that do not exercise keyboard/role assertions"
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
      "description": "GIVEN DESIGN-MGMT-008 defines ErrorBoundary and retry behavior WHEN verified_by pointers are added to MGMT-UI-011 AC-3 (error boundary) and AC-5 (retry) THEN the design contract's resilience claims are traced to component tests",
      "verify": "",
      "scenario": {
        "tier": "structural",
        "test_tier": "structural",
        "verification_service": "python3 scripts/lint_design_contracts.py --file .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/DESIGN-MGMT-008*.md",
        "negative_control": {
          "would_fail_if": [
            "ErrorBoundary AC points to a test that never throws",
            "retry AC points to a unit test instead of a visible retry flow",
            "ACs reference renamed or removed MGMT-UI-011 AC numbers"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "design_008_contract",
            "action": {
              "actor": "ci",
              "steps": [
                "parse DESIGN-MGMT-008 error-boundary and retry ACs",
                "assert each maps to a visible component test via verified_by"
              ]
            },
            "end_state": {
              "must_observe": [
                "error-boundary AC points to MGMT-UI-011 AC-3",
                "retry AC points to MGMT-UI-011 AC-5"
              ],
              "must_not_observe": [
                "references to nonexistent AC numbers",
                "behavioral ACs left without verified_by"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN a design contract AC without a verified_by block WHEN the design-contract lint gate runs THEN the gate exits non-zero and lists the missing entries",
      "verify": "",
      "scenario": {
        "tier": "structural",
        "test_tier": "structural",
        "verification_service": "python3 scripts/lint_design_contracts.py --negative-control DESIGN-MGMT-004*.md",
        "negative_control": {
          "would_fail_if": [
            "the lint gate exits 0 despite a missing verified_by",
            "the gate reports failure but does not name the AC",
            "the gate flags non-behavioral ACs (e.g. purely decorative notes)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "patched_design_contract",
            "action": {
              "actor": "ci",
              "steps": [
                "create a temporary stripped copy of one design AC",
                "run lint_design_contracts.py",
                "capture non-zero exit and reported AC list"
              ]
            },
            "end_state": {
              "must_observe": [
                "linter exit code non-zero",
                "output contains the stripped AC id"
              ],
              "must_not_observe": [
                "linter exit code 0",
                "no mention of the offending AC"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "DESIGN-MGMT-004.md contains verified_by blocks on all behavioral ACs",
      "verify": "python3 scripts/lint_design_contracts.py --file .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/DESIGN-MGMT-004*.md",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "DESIGN-MGMT-006.md contains verified_by blocks on all behavioral ACs",
      "verify": "python3 scripts/lint_design_contracts.py --file .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/DESIGN-MGMT-006*.md",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "DESIGN-MGMT-007.md contains verified_by blocks on all behavioral ACs",
      "verify": "python3 scripts/lint_design_contracts.py --file .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/DESIGN-MGMT-007*.md",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "DESIGN-MGMT-008.md contains verified_by blocks on all behavioral ACs",
      "verify": "python3 scripts/lint_design_contracts.py --file .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/DESIGN-MGMT-008*.md",
      "maps_to_ac": "AC-4"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "A deliberately stripped verified_by block trips the design-contract lint gate",
      "verify": "python3 scripts/lint_design_contracts.py --temporarily-strip one AC and confirm non-zero exit",
      "maps_to_ac": "AC-5"
    }
  ]
}
-->
