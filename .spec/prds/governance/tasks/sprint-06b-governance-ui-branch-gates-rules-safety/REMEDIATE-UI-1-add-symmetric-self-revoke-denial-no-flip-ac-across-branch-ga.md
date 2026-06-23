# REMEDIATE-UI-1: Add symmetric self-revoke denial no-flip AC across Branch Gates and Governance E2E

**Type:** REMEDIATION | **Status:** Cancelled | **Priority:** P0 | **Effort:** M (150 min)
**Agent:** sveltekit-implementer | **Reviewer:** sveltekit-reviewer | **Proposed by:** sveltekit-planner
**Closes red-hat findings:** H5
**Superseded by:** REMEDIATE-06B-D (symmetric self-revoke no-flip is covered by REMEDIATE-06B-D AC-3)
**Reason:** Superseded by REMEDIATE-06B-D AC-3; no independent work required.
**Depends on:** MGMT-UI-009, E2E-MGMT-UI-001, DESIGN-MGMT-004, REMEDIATE-UI-4 | **Blocks:** (none)
**PRD refs:** UC-MGMT-03, UC-MGMT-04, DESIGN-MGMT-004 | **Capabilities:** CAP-AUTHZ-01, CAP-A11Y-01

## What this does

Red-hat finding H5 observes that every no-flip AC (DESIGN-MGMT-004 AC-2, MGMT-UI-009 AC-7, E2E-MGMT-UI-001 AC-4) covers the grant/escalation direction only. No AC proves an admin self-REVOKE surfaces as a structured denial without flipping the control. Because admin self-grant is staged as pending (admin already holds the permission), the honest symmetric analog is the revoke direction: an actor with administration:write attempts to remove it from itself and receives perm.denied, while the Toggle stays ON. Add AC-8 to MGMT-UI-009 (BranchGatesList: toggling protected OFF when branch_gates_update would return perm.denied leaves Toggle ON). Add a symmetric AC to E2E-MGMT-UI-001 (admin self-revokes administration:write via the principals UI, denial banner appears, control reverts). Update DESIGN-MGMT-004 AC-2 prose and add verified_by pointers to both new ACs.

## Why

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: REMEDIATE-UI-1 — Add symmetric self-revoke denial no-flip AC across Branch Gates and Governance E2E
================================================================================

TASK_TYPE:   REMEDIATION
STATUS:      Backlog
PRIORITY:    P0
EFFORT:      M  (150 min)
AGENT:       implementer=sveltekit-implementer | reviewer=sveltekit-reviewer
PROPOSED-BY: sveltekit-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-MGMT-03, UC-MGMT-04, DESIGN-MGMT-004
CAPABILITIES:CAP-AUTHZ-01,CAP-A11Y-01
CLOSES:      H5

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Red-hat finding H5 observes that every no-flip AC (DESIGN-MGMT-004 AC-2, MGMT-UI-009 AC-7, E2E-MGMT-UI-001 AC-4) covers the grant/escalation direction only. No AC proves an admin self-REVOKE surfaces as a structured denial without flipping the control. Because admin self-grant is staged as pending (admin already holds the permission), the honest symmetric analog is the revoke direction: an actor with administration:write attempts to remove it from itself and receives perm.denied, while the Toggle stays ON. Add AC-8 to MGMT-UI-009 (BranchGatesList: toggling protected OFF when branch_gates_update would return perm.denied leaves Toggle ON). Add a symmetric AC to E2E-MGMT-UI-001 (admin self-revokes administration:write via the principals UI, denial banner appears, control reverts). Update DESIGN-MGMT-004 AC-2 prose and add verified_by pointers to both new ACs.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] The symmetric AC must cover revoke/revoke direction, not only grant/grant direction.
- [MUST] A denied revoke must leave the control in its original ON state and surface a danger InfoMessage.
- [MUST] The E2E proof must use an identity that holds administration:write and attempts to remove it from itself.
- [MUST] No runtime implementation files are changed unless the new AC exposes a real defect.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: the Modal appears; on confirm branch_gates_update returns perm.denied; the Toggle reverts to ON and a danger InfoMessage appears
- [ ] AC-2: a danger InfoMessage appears; the Toggle returns to ON; the permission is not removed from permissions.toml
- [ ] AC-3: the state machine is provable against live permissions and points to the new UI/E2E ACs
- [ ] AC-4: the test fails because aria-checked is false after the denied operation

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: the Modal appears; on confirm branch_gates_update returns perm.denied; the Toggle reverts to ON and a danger InfoMessage appears
  GIVEN: BranchGatesList mounted with seeded_gates_two_branches (main protected:true), main row expanded, seeded_write_denied configured for unprotect
  WHEN: user toggles protected OFF
  THEN: the Modal appears; on confirm branch_gates_update returns perm.denied; the Toggle reverts to ON and a danger InfoMessage appears
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  SCENARIO:
    tier: visible   test_tier: integration
    verification_service: pnpm test:ct:desktop -- BranchGatesListUnprotectRevokeNoFlip (real Svelte 5 runtime + sanctioned but-sdk mock layer per B14)
    negative_control.would_fail_if:
      - stub Toggle always reports aria-checked='true' regardless of click
      - mock SDK resolves successfully and flips the Toggle
      - component omits the danger InfoMessage on revoke
      - Toggle flips to aria-checked='false' before the SDK denial returns
    evidence: artifact_type=screenshot required_capture=True
    case[0] start_ref=seeded_gates_two_branches_unprotect_denied
      action.actor=admin
        - capture aria-checked on the 'main' protected Toggle before clicking
        - toggle the 'main' protected Toggle OFF
        - confirm the unprotect Modal
        - await the branch_gates_update denial response
      end_state.must_observe:
        - before click: the 'main' protected Toggle has aria-checked='true'
        - after denial: the 'main' protected Toggle still has aria-checked='true'
        - a danger InfoMessage with role='alert' is present and contains 'Permission denied' or 'perm.denied'
        - branch_gates_update SDK spy was called exactly once with {branch: 'main', protected: false}
      end_state.must_not_observe:
        - aria-checked='false' on the 'main' protected Toggle before the click
        - aria-checked='false' on the 'main' protected Toggle after the denial
        - 0 danger InfoMessage elements
        - 0 branch_gates_update spy calls

AC-2 : a danger InfoMessage appears; the Toggle returns to ON; the permission is not removed from permissions.toml
  GIVEN: ADMIN_HANDLE is logged in and opens the Principals tab
  WHEN: ADMIN_HANDLE toggles OFF their own administration:write permission
  THEN: a danger InfoMessage appears; the Toggle returns to ON; the permission is not removed from permissions.toml
  TEST_TIER: e2e   VERIFICATION_SERVICE: playwright-desktop
  SCENARIO:
    tier: visible   test_tier: e2e
    verification_service: pnpm test:e2e:playwright -- E2E-MGMT-UI-001-admin-self-revoke (real browser + Playwright DOM assertions per B14)
    negative_control.would_fail_if:
      - the Toggle stays OFF after the denial (self-revoke succeeded)
      - permissions.toml no longer contains administration:write for ADMIN_HANDLE
      - the denial is not surfaced as a danger InfoMessage
    evidence: artifact_type=screenshot required_capture=True
    case[0] start_ref=admin_has_administration_write
      action.actor=admin
        - open the Governance settings
        - navigate to the Principals tab
        - toggle OFF administration:write for self
      end_state.must_observe:
        - danger InfoMessage containing 'perm.denied' or 'Permission denied'
        - the Toggle has aria-checked='true' after the denial
        - permissions.toml still lists administration:write for ADMIN_HANDLE
      end_state.must_not_observe:
        - Toggle aria-checked='false' after denial
        - 0 danger InfoMessage elements
        - permissions.toml entry removed

AC-3 : the state machine is provable against live permissions and points to the new UI/E2E ACs
  GIVEN: DESIGN-MGMT-004 AC-2 currently describes an admin self-grant-to-denied transition
  WHEN: the contract is amended to describe a non-admin grant→denied transition and a separate admin self-revoke→denied transition with verified_by pointers
  THEN: the state machine is provable against live permissions and points to the new UI/E2E ACs
  TEST_TIER: structural   VERIFICATION_SERVICE: contract-audit
  SCENARIO:
    tier: structural   test_tier: structural
    verification_service: contract-audit (parse DESIGN-MGMT-004*.md REQUIREMENT-CONTRACT blocks)
    negative_control.would_fail_if:
      - the amended AC omits the symmetric revoke transition
      - verified_by pointers reference nonexistent ACs
      - the state machine still describes an unprovable admin self-grant→denied transition
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=design_contract_before_amendment
      action.actor=maintainer
        - open DESIGN-MGMT-004*.md
        - rewrite AC-2 grant row to use a non-admin actor
        - add revoke row for ADMIN_HANDLE self-revoke
        - add verified_by pointers to the new UI/E2E ACs
      end_state.must_observe:
        - AC-2 grant transition references a non-admin actor
        - a revoke transition exists with verified_by pointing to E2E-MGMT-UI-001 AC-5
        - verified_by tuples contain task_id and ac_id fields
      end_state.must_not_observe:
        - admin self-grant→denied text in the state machine
        - behavioral ACs without verified_by

AC-4 : the test fails because aria-checked is false after the denied operation
  GIVEN: a temporary implementation that flips the Toggle optimistically before the SDK denial returns
  WHEN: the AC-1 no-flip test runs
  THEN: the test fails because aria-checked is false after the denied operation
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  SCENARIO:
    tier: visible   test_tier: integration
    verification_service: pnpm test:ct:desktop -- BranchGatesListUnprotectRevokeNegativeControl (real Svelte 5 runtime + temporary optimistic-flip stub per B14)
    negative_control.would_fail_if:
      - the negative-control test still passes (Toggle did not flip)
      - the test fails for a reason other than the Toggle flip
      - the temporary optimistic-flip implementation is not actually used
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=seeded_gates_two_branches_unprotect_denied
      action.actor=maintainer
        - inject temporary optimistic Toggle flip in the component
        - run BranchGatesListUnprotectRevokeNoFlip test
        - capture assertion failure
      end_state.must_observe:
        - test runner exits non-zero
        - failure message references aria-checked='true' expected but 'false' found
      end_state.must_not_observe:
        - test exit code 0
        - failure message about import/syntax only

--------------------------------------------------------------------------------
TEST CRITERIA
--------------------------------------------------------------------------------
- TC-1: Branch gate unprotect revoke: Toggle stays ON after perm.denied and danger InfoMessage appears
- TC-2: Admin self-revoke administration:write denied in Playwright capstone
- TC-3: DESIGN-MGMT-004.md no longer claims an unprovable admin self-grant→denied state
- TC-4: Negative-control test fails if Toggle flips optimistically on denied revoke

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
- pnpm test:ct:desktop -- BranchGatesListUnprotectRevokeNoFlip  →  ?
- pnpm test:e2e:playwright -- E2E-MGMT-UI-001-admin-self-revoke  →  ?
- python3 scripts/lint_design_contracts.py --file .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/DESIGN-MGMT-004*.md  →  ?
- pnpm -F @gitbutler/desktop check && pnpm -F @gitbutler/web check  →  ?
- pnpm lint  →  ?

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
depends_on: MGMT-UI-009, E2E-MGMT-UI-001, DESIGN-MGMT-004, REMEDIATE-UI-4
blocks:     (none)

--------------------------------------------------------------------------------
NOTES
--------------------------------------------------------------------------------
This remediation closes the H5 finding. The load-bearing proof is the symmetric revoke no-flip. AC-2 (admin self-revoke E2E) may be partially blocked by the maturity of the principals UI; if so, land the contract update and the component-level AC-1/AC-4 first, and mark AC-2 as blocked-by-MGMT-UI-007/008 with a follow-up ticket.

```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "REMEDIATE-UI-1",
  "proposed_by": "sveltekit-planner",
  "supersedes": [],
  "closes_redhat_findings": [
    "H5"
  ],
  "fixtures": {
    "seeded_gates_two_branches_unprotect_denied": {
      "description": "BranchGatesList mounted with two branches (main protected:true) and branch_gates_update mocked to return perm.denied for the unprotect path",
      "seed_method": "component_mount",
      "records": [
        "main protected:true",
        "develop protected:false",
        "denied unprotect response"
      ]
    },
    "admin_has_administration_write": {
      "description": "An authenticated admin principal that currently holds administration:write",
      "seed_method": "public_api",
      "records": [
        "ADMIN_HANDLE principal",
        "administration:write in permissions.toml"
      ]
    },
    "design_contract_before_amendment": {
      "description": "Original DESIGN-MGMT-004 AC-2 claiming an admin self-grant->denied transition",
      "seed_method": "component_mount",
      "records": [
        "DESIGN-MGMT-004*.md AC-2 admin self-grant text"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN BranchGatesList mounted with seeded_gates_two_branches (main protected:true), main row expanded, seeded_write_denied configured for unprotect WHEN user toggles protected OFF THEN the Modal appears; on confirm branch_gates_update returns perm.denied; the Toggle reverts to ON and a danger InfoMessage appears",
      "verify": "",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "pnpm test:ct:desktop -- BranchGatesListUnprotectRevokeNoFlip (real Svelte 5 runtime + sanctioned but-sdk mock layer per B14)",
        "negative_control": {
          "would_fail_if": [
            "stub Toggle always reports aria-checked='true' regardless of click",
            "mock SDK resolves successfully and flips the Toggle",
            "component omits the danger InfoMessage on revoke",
            "Toggle flips to aria-checked='false' before the SDK denial returns"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_gates_two_branches_unprotect_denied",
            "action": {
              "actor": "admin",
              "steps": [
                "capture aria-checked on the 'main' protected Toggle before clicking",
                "toggle the 'main' protected Toggle OFF",
                "confirm the unprotect Modal",
                "await the branch_gates_update denial response"
              ]
            },
            "end_state": {
              "must_observe": [
                "before click: the 'main' protected Toggle has aria-checked='true'",
                "after denial: the 'main' protected Toggle still has aria-checked='true'",
                "a danger InfoMessage with role='alert' is present and contains 'Permission denied' or 'perm.denied'",
                "branch_gates_update SDK spy was called exactly once with {branch: 'main', protected: false}"
              ],
              "must_not_observe": [
                "aria-checked='false' on the 'main' protected Toggle before the click",
                "aria-checked='false' on the 'main' protected Toggle after the denial",
                "0 danger InfoMessage elements",
                "0 branch_gates_update spy calls"
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
      "description": "GIVEN ADMIN_HANDLE is logged in and opens the Principals tab WHEN ADMIN_HANDLE toggles OFF their own administration:write permission THEN a danger InfoMessage appears; the Toggle returns to ON; the permission is not removed from permissions.toml",
      "verify": "",
      "scenario": {
        "tier": "visible",
        "test_tier": "e2e",
        "verification_service": "pnpm test:e2e:playwright -- E2E-MGMT-UI-001-admin-self-revoke (real browser + Playwright DOM assertions per B14)",
        "negative_control": {
          "would_fail_if": [
            "the Toggle stays OFF after the denial (self-revoke succeeded)",
            "permissions.toml no longer contains administration:write for ADMIN_HANDLE",
            "the denial is not surfaced as a danger InfoMessage"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "admin_has_administration_write",
            "action": {
              "actor": "admin",
              "steps": [
                "open the Governance settings",
                "navigate to the Principals tab",
                "toggle OFF administration:write for self"
              ]
            },
            "end_state": {
              "must_observe": [
                "danger InfoMessage containing 'perm.denied' or 'Permission denied'",
                "the Toggle has aria-checked='true' after the denial",
                "permissions.toml still lists administration:write for ADMIN_HANDLE"
              ],
              "must_not_observe": [
                "Toggle aria-checked='false' after denial",
                "0 danger InfoMessage elements",
                "permissions.toml entry removed"
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
      "description": "GIVEN DESIGN-MGMT-004 AC-2 currently describes an admin self-grant-to-denied transition WHEN the contract is amended to describe a non-admin grant\u2192denied transition and a separate admin self-revoke\u2192denied transition with verified_by pointers THEN the state machine is provable against live permissions and points to the new UI/E2E ACs",
      "verify": "",
      "scenario": {
        "tier": "structural",
        "test_tier": "structural",
        "verification_service": "contract-audit (parse DESIGN-MGMT-004*.md REQUIREMENT-CONTRACT blocks)",
        "negative_control": {
          "would_fail_if": [
            "the amended AC omits the symmetric revoke transition",
            "verified_by pointers reference nonexistent ACs",
            "the state machine still describes an unprovable admin self-grant\u2192denied transition"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "design_contract_before_amendment",
            "action": {
              "actor": "maintainer",
              "steps": [
                "open DESIGN-MGMT-004*.md",
                "rewrite AC-2 grant row to use a non-admin actor",
                "add revoke row for ADMIN_HANDLE self-revoke",
                "add verified_by pointers to the new UI/E2E ACs"
              ]
            },
            "end_state": {
              "must_observe": [
                "AC-2 grant transition references a non-admin actor",
                "a revoke transition exists with verified_by pointing to E2E-MGMT-UI-001 AC-5",
                "verified_by tuples contain task_id and ac_id fields"
              ],
              "must_not_observe": [
                "admin self-grant\u2192denied text in the state machine",
                "behavioral ACs without verified_by"
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
      "description": "GIVEN a temporary implementation that flips the Toggle optimistically before the SDK denial returns WHEN the AC-1 no-flip test runs THEN the test fails because aria-checked is false after the denied operation",
      "verify": "",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "pnpm test:ct:desktop -- BranchGatesListUnprotectRevokeNegativeControl (real Svelte 5 runtime + temporary optimistic-flip stub per B14)",
        "negative_control": {
          "would_fail_if": [
            "the negative-control test still passes (Toggle did not flip)",
            "the test fails for a reason other than the Toggle flip",
            "the temporary optimistic-flip implementation is not actually used"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_gates_two_branches_unprotect_denied",
            "action": {
              "actor": "maintainer",
              "steps": [
                "inject temporary optimistic Toggle flip in the component",
                "run BranchGatesListUnprotectRevokeNoFlip test",
                "capture assertion failure"
              ]
            },
            "end_state": {
              "must_observe": [
                "test runner exits non-zero",
                "failure message references aria-checked='true' expected but 'false' found"
              ],
              "must_not_observe": [
                "test exit code 0",
                "failure message about import/syntax only"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "Branch gate unprotect revoke: Toggle stays ON after perm.denied and danger InfoMessage appears",
      "verify": "pnpm test:ct:desktop -- BranchGatesListUnprotectRevokeNoFlip",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "Admin self-revoke administration:write denied in Playwright capstone",
      "verify": "pnpm test:e2e:playwright -- E2E-MGMT-UI-001-admin-self-revoke",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "DESIGN-MGMT-004.md no longer claims an unprovable admin self-grant\u2192denied state",
      "verify": "grep -v 'admin.*self-grant.*denied\\|admin self-grant.*denied' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/DESIGN-MGMT-004*.md",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "Negative-control test fails if Toggle flips optimistically on denied revoke",
      "verify": "pnpm test:ct:desktop -- BranchGatesListUnprotectRevokeNegativeControl",
      "maps_to_ac": "AC-4"
    }
  ]
}
-->
