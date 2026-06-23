# REM-DESIGN-MGMT-004-A: Honest U1 wording for DESIGN-MGMT-004 (successor) — non-admin self-grant + symmetric self-revoke no-flip contract

**Type:** DESIGN | **Status:** Backlog | **Priority:** P0 | **Effort:** XS (? min)
**Agent:** frontend-designer | **Reviewer:** ? | **Proposed by:** frontend-designer
**Closes red-hat findings:** H4, H5, U1, L8

**Supersedes:** DESIGN-MGMT-004 (per `red-hat-20260622T145305Z.md` — original scope was unimplementable / false premise)
**Depends on:** DESIGN-MGMT-002 (pending-state contract — denial banner replaces the pending banner in the slot; slot priority), DESIGN-MGMT-003 (read-only disabled-control contract — denial state applies on top of the same disabled controls), DESIGN-MGMT-004 (the ORIGINAL contract being superseded — read in full to preserve AC-1/AC-4 and correct AC-2/AC-3) | **Blocks:** MGMT-UI-011 (a11y + IPC-failure danger banner + Retry — this successor contract is the corrected primary design source for the denial/no-flip implementation; the original DESIGN-MGMT-004 is superseded), MGMT-UI-011 AC-4b / E2E-MGMT-UI-001 AC-4c (forward-looking revoke-direction downstream proofs — to be added per this contract's H5 advisory and red-hat Recommendation #7)
**PRD refs:** UC-MGMT-06 | **Capabilities:** (none)

## What this does

(no objective)

## Why

Supersedes DESIGN-MGMT-004 per red-hat-20260622T145305Z.md (H4, H5, U1, L8).

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: REM-DESIGN-MGMT-004-A — Honest U1 wording for DESIGN-MGMT-004 (successor) — non-admin self-grant + symmetric self-revoke no-flip contract
================================================================================

TASK_TYPE:   DESIGN
STATUS:      Backlog
PRIORITY:    P0
EFFORT:      XS  (? min)
AGENT:       implementer=frontend-designer | reviewer=?
PROPOSED-BY: frontend-designer
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-MGMT-06
SUPERSEDES:  DESIGN-MGMT-004
CLOSES:      H4, H5, U1, L8

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------


--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST supersede DESIGN-MGMT-004 AC-2 — the original state-machine row encoded an admin-self-grant→denied transition that cannot fire in production (admin already holds administration:write per CAP-AUTHZ-01; red-hat-20260622T145305Z.md H4 + U1 advisory). This contract replaces it with the honest non-admin self-grant direction that matches E2E-MGMT-UI-001 AC-4's NONADMIN_HANDLE proof.
- [MUST] MUST add the symmetric self-revoke direction (admin self-revoke → denial without flipping) — closes red-hat H5.
- [NEVER] NEVER use the unprovable admin-self-grant→denied wording in any state-machine row or Human Gate step.
- [STRICTLY] STRICTLY record the SPRINT.md Human Gate step 4 wording as needing the same U1 fix (propose exact rewording: 'A principal lacking administration:write attempts to self-grant administration:write → observe the denial banner and the toggle does not flip') — this becomes an upstream advisory for /kb-sprint-plan --delta-replan; this contract does NOT edit SPRINT.md directly.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]:
- [ ] AC-2:
- [ ] AC-3:
- [ ] AC-4:

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]:
  VERIFY: grep -c 'principal LACKING administration:write' <new-contract-file> ≥ 1 AND grep -c 'admin toggles administration:write ON for themselves\|admin self-grants administration:write' <new-contract-file> == 0
  SCENARIO:
    tier: visible   test_tier: integration
    verification_service: pnpm test:ct:desktop -- GovernanceSelfEscalationNoFlip (MGMT-UI-011 AC-4) + pnpm test:e2e:playwright -- governance-capstone -g step4 (E2E-MGMT-UI-001 AC-4a)
    negative_control.would_fail_if:
      - the contract's state-machine row encodes the admin-self-grant→denied transition (cannot fire in production per CAP-AUTHZ-01 — admin already holds administration:write; the governed path stages as pending, never denied)
      - the contract uses 'admin' as the self-grant actor in any denial row (U1 unprovable wording — the downstream MGMT-UI-011 AC-4 test would have to mock a denial that the real backend cannot produce)
      - the downstream MGMT-UI-011 AC-4 / E2E-MGMT-UI-001 AC-4a test is absent (no executable proof — L8 design-contract verification gap)
      - the contract's denial row offers a Retry button (structural denial bypassed as if transient)
    evidence: artifact_type=file_artifact required_capture=True
    case[0] start_ref=nonadmin_without_admin_write
      action.actor=user
        - a principal lacking administration:write opens the Principals tab and clicks the administration:write Toggle ON (attempting self-grant), then saves
        - the governed SDK call (perm_grant) fires against the real but-authz gate
      end_state.must_observe:
        - SDK call returns {type:error, code:'perm.denied'} with message 'Permission denied' (the honest denial — only a non-admin actor can receive this on a self-grant)
        - danger InfoMessage visible with verbatim text 'perm.denied — you cannot modify your own administration grants.' and a remediation_hint sub-line
        - administration:write Toggle aria-checked='false' (UNCHANGED — no optimistic flip)
        - pending count does NOT increment (the denied write staged nothing)
        - NO Retry button visible on the denial banner (structural denial — not a transient error)
        - the successor contract's state-machine row names the actor as 'a principal LACKING administration:write' (literal phrase present in the contract file)
      end_state.must_not_observe:
        - the successor contract's state-machine row describes 'admin toggles administration:write ON for themselves' (the unprovable U1 wording — grep must return 0 matches)
        - administration:write Toggle aria-checked='true' (optimistic flip occurred — CAP-AUTHZ-01 violation)
        - pending count increments on a denied write (the denial staged nothing — a pending increment would mean the write partially succeeded)
        - a Retry button on the denial banner (the denial is structural, not transient — a Retry would imply the denial is retryable)
        - the danger InfoMessage absent (0 InfoMessage elements with style='danger' — denial swallowed)

AC-2 :
  VERIFY: grep -c 'self-revoke' <new-contract-file> ≥ 1 AND grep -c 'aria-checked.*true.*unchanged\|stays.*true' <new-contract-file> ≥ 1 (revoke-path toggle stays ON)
  SCENARIO:
    tier: visible   test_tier: integration
    verification_service: MGMT-UI-011 AC-4 (grant-direction, exists) + forward-looking MGMT-UI-011 AC-4b (revoke-direction, to be added per red-hat H5)
    negative_control.would_fail_if:
      - the contract omits the self-revoke direction entirely (H5 asymmetry remains — grant-only no-flip)
      - the contract treats self-revoke as a successful operation (no denial — the toggle flips OFF and the admin silently loses administration:write, which is the exact H5 bug)
      - the downstream revoke-direction test flips the toggle to aria-checked='false' (optimistic flip on revoke — no-flip violated)
      - the three-state table's denial row names only the grant entry path (single-path denial row — H5 re-opened)
      - the contract falsely claims the revoke direction is already proven by MGMT-UI-011 AC-4 (the existing AC-4 covers grant-direction only — honest_gap_note must be present)
    evidence: artifact_type=file_artifact required_capture=True
    case[0] start_ref=admin_with_admin_write
      action.actor=user
        - an admin who holds administration:write opens the Principals tab and clicks the administration:write Toggle OFF (attempting self-revoke), then saves
        - the governed SDK call (perm_revoke) fires against the real but-authz gate
      end_state.must_observe:
        - SDK call returns {type:error, code:'perm.denied'} with message containing 'cannot modify your own administration grants' (symmetric self-modification block — fires on revoke)
        - administration:write Toggle aria-checked='true' (UNCHANGED — no optimistic flip; the admin retains the permission)
        - danger InfoMessage visible with verbatim denial text
        - the three-state table's denial row names BOTH entry paths: '(a) a principal LACKING administration:write self-grants OR (b) an admin self-revokes'
      end_state.must_not_observe:
        - administration:write Toggle aria-checked='false' (optimistic flip — admin silently lost the permission; H5 bug)
        - the danger InfoMessage absent (denial swallowed on revoke)
        - the denial row naming only the grant direction (asymmetric — H5 not closed at the contract layer)
        - the contract claiming the revoke direction is downstream-proven by an existing AC (honest_gap_note requires the gap be recorded)

AC-3 :
  VERIFY: grep -F 'A principal lacking administration:write attempts to self-grant administration:write' <new-contract-file> ≥ 1 match AND grep -Fi 'does NOT edit SPRINT.md' <new-contract-file> ≥ 1 match
  SCENARIO:
    tier: visible   test_tier: unit
    verification_service: static grep on the successor contract file
    negative_control.would_fail_if:
      - the advisory section is absent (SPRINT.md step 4 keeps the unprovable 'admin self-grant' wording — U1 unresolved at the human-gate level)
      - the proposed rewording uses 'admin' as the actor (U1 wording reproduced in the advisory itself — self-defeating)
      - the advisory paraphrases the rewording instead of quoting the exact text (the /kb-sprint-plan --delta-replan consumer needs the literal string)
      - the contract edits SPRINT.md directly (scope violation — writeProhibited on SPRINT.md)
      - the advisory omits the /kb-sprint-plan --delta-replan routing (the rewording has no owner)
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=design_contract_artifact
      action.actor=api_client
        - grep -F 'A principal lacking administration:write attempts to self-grant administration:write' <new-contract-file>
        - grep -Fi 'does NOT edit SPRINT.md' <new-contract-file>
        - grep -Fi 'delta-replan' <new-contract-file>
      end_state.must_observe:
        - ≥ 1 match for the exact proposed rewording 'A principal lacking administration:write attempts to self-grant administration:write'
        - ≥ 1 match for the literal clause 'does NOT edit SPRINT.md'
        - ≥ 1 match for '/kb-sprint-plan --delta-replan' (the routing owner)
        - the advisory section identifies SPRINT.md Human Testing Gate step 4 by name
      end_state.must_not_observe:
        - 0 matches for the exact rewording (absent or paraphrased — the delta-replan consumer cannot consume a paraphrase)
        - the contract editing SPRINT.md (writeProhibited violation)
        - the proposed rewording using 'admin' as the actor (U1 wording reproduced)

AC-4 :
  VERIFY: python3 -c "import json,sys; c=json.load(open('<new-contract-file-requirement-block>')); behavioral=[r for r in c['requirements'] if r['type']=='acceptance_criterion' and r.get('scenario')]; missing=[r['id'] for r in behavioral if not r.get('verified_by')]; dangling=[r['id'] for r in behavioral if r.get('verified_by') and all('AC-' not in str(v) for v in r['verified_by'])]; assert not missing, f'behavioral ACs missing verified_by: {missing}'; assert not dangling, f'verified_by entries without a downstream AC ID: {dangling}'; print(f'L8 closed: {len(behavioral)} behavioral ACs, all with downstream verified_by pointers')"
  SCENARIO:
    tier: visible   test_tier: unit
    verification_service: deterministic JSON audit of the successor contract's REQUIREMENT-CONTRACT block
    negative_control.would_fail_if:
      - any behavioral AC lacks a verified_by pointer (L8 — design contracts have no executable verification path; a downstream UI change could violate the contract undetected)
      - a verified_by pointer cites only 'design review' without a downstream AC ID (Category 4 Test Theatre — the validator's WEAK_ORACLE analogue for verification routing)
      - a verified_by pointer names a non-existent downstream AC ID (dangling reference — false confidence)
      - AC-2 (self-revoke) claims verified_by an existing AC that covers grant-direction only without the honest_gap_note (false proven claim — H5 honesty violation)
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=design_contract_artifact
      action.actor=api_client
        - parse the successor contract's REQUIREMENT-CONTRACT block as JSON
        - filter to behavioral ACs (type=='acceptance_criterion' with a scenario object)
        - for each behavioral AC, assert verified_by is a non-empty array
        - for each verified_by entry, assert it contains a downstream AC ID pattern (e.g., 'MGMT-UI-011 AC-4' or 'E2E-MGMT-UI-001 AC-4a')
      end_state.must_observe:
        - count of behavioral ACs with non-empty verified_by arrays == count of behavioral ACs (i.e., 2: AC-1, AC-2)
        - every verified_by entry contains at least one string matching the pattern '<TASK-ID> AC-<N>' (a specific downstream AC by stable ID)
        - AC-2's verified_by includes the honest_gap_note field or an entry naming 'AC-4b'/'AC-4c' as 'TO BE ADDED' (the revoke-direction gap is recorded, not hidden)
      end_state.must_not_observe:
        - any behavioral AC with an empty or absent verified_by array (L8 gap)
        - any verified_by entry citing only 'design review' without a downstream AC ID (Test Theatre)
        - AC-2's verified_by claiming the revoke direction is proven by an existing AC without the gap note (false proven claim)

--------------------------------------------------------------------------------
TEST CRITERIA
--------------------------------------------------------------------------------
- TC-1: The successor contract's state-machine row names 'a principal LACKING administration:write' as the self-grant actor AND zero state-machine rows contain 'admin toggles administration:write ON for themselves' or 'admin self-grants administration:write'
- TC-2: The successor contract adds the symmetric self-revoke direction with toggle aria-checked unchanged 'true' AND the three-state table's denial row names both entry paths (grant + revoke)
- TC-3: The successor contract records the SPRINT.md step 4 U1 advisory with the exact proposed rewording AND the no-edit clause AND the /kb-sprint-plan --delta-replan routing
- TC-4: Every behavioral AC in the successor contract carries a verified_by pointer citing a specific downstream AC by ID; zero verified_by entries cite only 'design review'

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
- .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/DESIGN-MGMT-004-denial-no-flip-contract.md  (lines 1-248) — THE CONTRACT BEING SUPERSEDED. Read in FULL. AC-2 (lines 82-88) encodes the unprovable admin-self-grant→denied four-step sequence. AC-3 (lines 90-96) encodes the three-state table whose denial row inherits the same unprovable actor. AC-1 (lines 74-80) and AC-4 (lines 98-104) are PRESERVED unchanged. The REQUIREMENT-CONTRACT block (lines 178-247) shows the v1 shape to match.
- .spec/reviews/red-hat-20260622T145305Z.md  (lines 53-63 (H4 + H5), 131-133 (L8), 117-124 (U1 advisory in SPRINT.md)) — H4: DESIGN-MGMT-004 AC-2 inherits unprovable wording. H5: no-flip is asymmetric (grant only). L8: design contracts have no executable verification path (add verified_by pointers). U1: step-4 'admin self-grant' is unprovable — an admin already holds administration:write; the capstone works around it with NONADMIN_HANDLE.
- .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-011-accessibility-ipc-retry.md  (lines 104-109 (AC-4 self-escalation no-flip), 902-953 (AC-4 scenario with seeded_self_escalation_denial fixture)) — THE DOWNSTREAM GRANT-DIRECTION PROOF. AC-4 (GovernanceSelfEscalationNoFlip) is the verified_by target for this contract's AC-1. Note the fixture seeded_self_escalation_denial uses a generic 'current principal' — confirm the downstream test's actor is consistent with the non-admin self-grant direction this contract specifies.
- .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/E2E-MGMT-UI-001-governance-capstone-playwright.md  (lines 139-153 (AC-4 PRIMARY — self-escalation denial/no-flip NONADMIN + ADMIN sub-case)) — THE CAPSTONE'S HONEST WORKAROUND. AC-4a uses NONADMIN_HANDLE (BUT_AGENT_HANDLE=NONADMIN_HANDLE) — the honest non-admin proof that perm.denied fires. AC-4a is the verified_by target for this contract's AC-1. Note AC-4 covers grant-direction only; the revoke-direction sub-case (this contract's AC-2) does not exist downstream yet (H5).
- .spec/prds/governance/08-uc-mgmt.md  (lines 132-152 (UC-MGMT-06 self-escalation no-flip + structured denial)) — UC-MGMT-06 AC-7: 'The UI does not optimistically apply a self-escalation … it surfaces the structured denial returned by the governed path rather than flipping the control.' Note the PRD wording says 'an admin granting itself' — the same U1 imprecision. This contract's SPRINT.md advisory (AC-3) is the sprint-level instance; a separate PRD-level advisory may be warranted but is out of scope here.

--------------------------------------------------------------------------------
GUARDRAILS
--------------------------------------------------------------------------------
WRITE_ALLOWED:

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
depends_on: DESIGN-MGMT-002 (pending-state contract — denial banner replaces the pending banner in the slot; slot priority), DESIGN-MGMT-003 (read-only disabled-control contract — denial state applies on top of the same disabled controls), DESIGN-MGMT-004 (the ORIGINAL contract being superseded — read in full to preserve AC-1/AC-4 and correct AC-2/AC-3)
blocks:     MGMT-UI-011 (a11y + IPC-failure danger banner + Retry — this successor contract is the corrected primary design source for the denial/no-flip implementation; the original DESIGN-MGMT-004 is superseded), MGMT-UI-011 AC-4b / E2E-MGMT-UI-001 AC-4c (forward-looking revoke-direction downstream proofs — to be added per this contract's H5 advisory and red-hat Recommendation #7)

```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "REM-DESIGN-MGMT-004-A",
  "proposed_by": "frontend-designer",
  "supersedes": [
    "DESIGN-MGMT-004"
  ],
  "closes_redhat_findings": [
    "H4",
    "H5",
    "U1",
    "L8"
  ],
  "fixtures": {},
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN  WHEN  THEN ",
      "verify": "grep -c 'principal LACKING administration:write' <new-contract-file> \u2265 1 AND grep -c 'admin toggles administration:write ON for themselves\\|admin self-grants administration:write' <new-contract-file> == 0",
      "scenario": {
        "id": "SC-REM-DESIGN-004A-1",
        "primary": true,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "pnpm test:ct:desktop -- GovernanceSelfEscalationNoFlip (MGMT-UI-011 AC-4) + pnpm test:e2e:playwright -- governance-capstone -g step4 (E2E-MGMT-UI-001 AC-4a)",
        "start_ref": "nonadmin_without_admin_write",
        "negative_control": {
          "would_fail_if": [
            "the contract's state-machine row encodes the admin-self-grant\u2192denied transition (cannot fire in production per CAP-AUTHZ-01 \u2014 admin already holds administration:write; the governed path stages as pending, never denied)",
            "the contract uses 'admin' as the self-grant actor in any denial row (U1 unprovable wording \u2014 the downstream MGMT-UI-011 AC-4 test would have to mock a denial that the real backend cannot produce)",
            "the downstream MGMT-UI-011 AC-4 / E2E-MGMT-UI-001 AC-4a test is absent (no executable proof \u2014 L8 design-contract verification gap)",
            "the contract's denial row offers a Retry button (structural denial bypassed as if transient)"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "nonadmin_without_admin_write",
            "action": {
              "actor": "user",
              "steps": [
                "a principal lacking administration:write opens the Principals tab and clicks the administration:write Toggle ON (attempting self-grant), then saves",
                "the governed SDK call (perm_grant) fires against the real but-authz gate"
              ]
            },
            "end_state": {
              "must_observe": [
                "SDK call returns {type:error, code:'perm.denied'} with message 'Permission denied' (the honest denial \u2014 only a non-admin actor can receive this on a self-grant)",
                "danger InfoMessage visible with verbatim text 'perm.denied \u2014 you cannot modify your own administration grants.' and a remediation_hint sub-line",
                "administration:write Toggle aria-checked='false' (UNCHANGED \u2014 no optimistic flip)",
                "pending count does NOT increment (the denied write staged nothing)",
                "NO Retry button visible on the denial banner (structural denial \u2014 not a transient error)",
                "the successor contract's state-machine row names the actor as 'a principal LACKING administration:write' (literal phrase present in the contract file)"
              ],
              "must_not_observe": [
                "the successor contract's state-machine row describes 'admin toggles administration:write ON for themselves' (the unprovable U1 wording \u2014 grep must return 0 matches)",
                "administration:write Toggle aria-checked='true' (optimistic flip occurred \u2014 CAP-AUTHZ-01 violation)",
                "pending count increments on a denied write (the denial staged nothing \u2014 a pending increment would mean the write partially succeeded)",
                "a Retry button on the denial banner (the denial is structural, not transient \u2014 a Retry would imply the denial is retryable)",
                "the danger InfoMessage absent (0 InfoMessage elements with style='danger' \u2014 denial swallowed)"
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
      "description": "GIVEN  WHEN  THEN ",
      "verify": "grep -c 'self-revoke' <new-contract-file> \u2265 1 AND grep -c 'aria-checked.*true.*unchanged\\|stays.*true' <new-contract-file> \u2265 1 (revoke-path toggle stays ON)",
      "scenario": {
        "id": "SC-REM-DESIGN-004A-2",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "MGMT-UI-011 AC-4 (grant-direction, exists) + forward-looking MGMT-UI-011 AC-4b (revoke-direction, to be added per red-hat H5)",
        "start_ref": "admin_with_admin_write",
        "negative_control": {
          "would_fail_if": [
            "the contract omits the self-revoke direction entirely (H5 asymmetry remains \u2014 grant-only no-flip)",
            "the contract treats self-revoke as a successful operation (no denial \u2014 the toggle flips OFF and the admin silently loses administration:write, which is the exact H5 bug)",
            "the downstream revoke-direction test flips the toggle to aria-checked='false' (optimistic flip on revoke \u2014 no-flip violated)",
            "the three-state table's denial row names only the grant entry path (single-path denial row \u2014 H5 re-opened)",
            "the contract falsely claims the revoke direction is already proven by MGMT-UI-011 AC-4 (the existing AC-4 covers grant-direction only \u2014 honest_gap_note must be present)"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "admin_with_admin_write",
            "action": {
              "actor": "user",
              "steps": [
                "an admin who holds administration:write opens the Principals tab and clicks the administration:write Toggle OFF (attempting self-revoke), then saves",
                "the governed SDK call (perm_revoke) fires against the real but-authz gate"
              ]
            },
            "end_state": {
              "must_observe": [
                "SDK call returns {type:error, code:'perm.denied'} with message containing 'cannot modify your own administration grants' (symmetric self-modification block \u2014 fires on revoke)",
                "administration:write Toggle aria-checked='true' (UNCHANGED \u2014 no optimistic flip; the admin retains the permission)",
                "danger InfoMessage visible with verbatim denial text",
                "the three-state table's denial row names BOTH entry paths: '(a) a principal LACKING administration:write self-grants OR (b) an admin self-revokes'"
              ],
              "must_not_observe": [
                "administration:write Toggle aria-checked='false' (optimistic flip \u2014 admin silently lost the permission; H5 bug)",
                "the danger InfoMessage absent (denial swallowed on revoke)",
                "the denial row naming only the grant direction (asymmetric \u2014 H5 not closed at the contract layer)",
                "the contract claiming the revoke direction is downstream-proven by an existing AC (honest_gap_note requires the gap be recorded)"
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
      "description": "GIVEN  WHEN  THEN ",
      "verify": "grep -F 'A principal lacking administration:write attempts to self-grant administration:write' <new-contract-file> \u2265 1 match AND grep -Fi 'does NOT edit SPRINT.md' <new-contract-file> \u2265 1 match",
      "scenario": {
        "id": "SC-REM-DESIGN-004A-3",
        "primary": false,
        "tier": "visible",
        "test_tier": "unit",
        "verification_service": "static grep on the successor contract file",
        "start_ref": "design_contract_artifact",
        "negative_control": {
          "would_fail_if": [
            "the advisory section is absent (SPRINT.md step 4 keeps the unprovable 'admin self-grant' wording \u2014 U1 unresolved at the human-gate level)",
            "the proposed rewording uses 'admin' as the actor (U1 wording reproduced in the advisory itself \u2014 self-defeating)",
            "the advisory paraphrases the rewording instead of quoting the exact text (the /kb-sprint-plan --delta-replan consumer needs the literal string)",
            "the contract edits SPRINT.md directly (scope violation \u2014 writeProhibited on SPRINT.md)",
            "the advisory omits the /kb-sprint-plan --delta-replan routing (the rewording has no owner)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "design_contract_artifact",
            "action": {
              "actor": "api_client",
              "steps": [
                "grep -F 'A principal lacking administration:write attempts to self-grant administration:write' <new-contract-file>",
                "grep -Fi 'does NOT edit SPRINT.md' <new-contract-file>",
                "grep -Fi 'delta-replan' <new-contract-file>"
              ]
            },
            "end_state": {
              "must_observe": [
                "\u2265 1 match for the exact proposed rewording 'A principal lacking administration:write attempts to self-grant administration:write'",
                "\u2265 1 match for the literal clause 'does NOT edit SPRINT.md'",
                "\u2265 1 match for '/kb-sprint-plan --delta-replan' (the routing owner)",
                "the advisory section identifies SPRINT.md Human Testing Gate step 4 by name"
              ],
              "must_not_observe": [
                "0 matches for the exact rewording (absent or paraphrased \u2014 the delta-replan consumer cannot consume a paraphrase)",
                "the contract editing SPRINT.md (writeProhibited violation)",
                "the proposed rewording using 'admin' as the actor (U1 wording reproduced)"
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
      "description": "GIVEN  WHEN  THEN ",
      "verify": "python3 -c \"import json,sys; c=json.load(open('<new-contract-file-requirement-block>')); behavioral=[r for r in c['requirements'] if r['type']=='acceptance_criterion' and r.get('scenario')]; missing=[r['id'] for r in behavioral if not r.get('verified_by')]; dangling=[r['id'] for r in behavioral if r.get('verified_by') and all('AC-' not in str(v) for v in r['verified_by'])]; assert not missing, f'behavioral ACs missing verified_by: {missing}'; assert not dangling, f'verified_by entries without a downstream AC ID: {dangling}'; print(f'L8 closed: {len(behavioral)} behavioral ACs, all with downstream verified_by pointers')\"",
      "scenario": {
        "id": "SC-REM-DESIGN-004A-4",
        "primary": false,
        "tier": "visible",
        "test_tier": "unit",
        "verification_service": "deterministic JSON audit of the successor contract's REQUIREMENT-CONTRACT block",
        "start_ref": "design_contract_artifact",
        "negative_control": {
          "would_fail_if": [
            "any behavioral AC lacks a verified_by pointer (L8 \u2014 design contracts have no executable verification path; a downstream UI change could violate the contract undetected)",
            "a verified_by pointer cites only 'design review' without a downstream AC ID (Category 4 Test Theatre \u2014 the validator's WEAK_ORACLE analogue for verification routing)",
            "a verified_by pointer names a non-existent downstream AC ID (dangling reference \u2014 false confidence)",
            "AC-2 (self-revoke) claims verified_by an existing AC that covers grant-direction only without the honest_gap_note (false proven claim \u2014 H5 honesty violation)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "design_contract_artifact",
            "action": {
              "actor": "api_client",
              "steps": [
                "parse the successor contract's REQUIREMENT-CONTRACT block as JSON",
                "filter to behavioral ACs (type=='acceptance_criterion' with a scenario object)",
                "for each behavioral AC, assert verified_by is a non-empty array",
                "for each verified_by entry, assert it contains a downstream AC ID pattern (e.g., 'MGMT-UI-011 AC-4' or 'E2E-MGMT-UI-001 AC-4a')"
              ]
            },
            "end_state": {
              "must_observe": [
                "count of behavioral ACs with non-empty verified_by arrays == count of behavioral ACs (i.e., 2: AC-1, AC-2)",
                "every verified_by entry contains at least one string matching the pattern '<TASK-ID> AC-<N>' (a specific downstream AC by stable ID)",
                "AC-2's verified_by includes the honest_gap_note field or an entry naming 'AC-4b'/'AC-4c' as 'TO BE ADDED' (the revoke-direction gap is recorded, not hidden)"
              ],
              "must_not_observe": [
                "any behavioral AC with an empty or absent verified_by array (L8 gap)",
                "any verified_by entry citing only 'design review' without a downstream AC ID (Test Theatre)",
                "AC-2's verified_by claiming the revoke direction is proven by an existing AC without the gap note (false proven claim)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "The successor contract's state-machine row names 'a principal LACKING administration:write' as the self-grant actor AND zero state-machine rows contain 'admin toggles administration:write ON for themselves' or 'admin self-grants administration:write'",
      "verify": "grep -c 'principal LACKING administration:write' <new-contract-file> \u2265 1 AND grep -ciE 'admin (toggles|self-grants) administration:write' <new-contract-file> == 0",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "The successor contract adds the symmetric self-revoke direction with toggle aria-checked unchanged 'true' AND the three-state table's denial row names both entry paths (grant + revoke)",
      "verify": "grep -ci 'self-revoke' <new-contract-file> \u2265 1 AND grep -ci 'entry path' <new-contract-file> \u2265 2 (both paths named)",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "The successor contract records the SPRINT.md step 4 U1 advisory with the exact proposed rewording AND the no-edit clause AND the /kb-sprint-plan --delta-replan routing",
      "verify": "grep -F 'A principal lacking administration:write attempts to self-grant administration:write' <new-contract-file> && grep -Fi 'does NOT edit SPRINT.md' <new-contract-file> && grep -F '/kb-sprint-plan --delta-replan' <new-contract-file>",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "Every behavioral AC in the successor contract carries a verified_by pointer citing a specific downstream AC by ID; zero verified_by entries cite only 'design review'",
      "verify": "python3 -c \"import json;c=json.load(open('<requirement-block>'));b=[r for r in c['requirements'] if r['type']=='acceptance_criterion' and r.get('scenario')];assert all(r.get('verified_by') for r in b);assert all(any('AC-' in str(v) for v in r['verified_by']) for r in b);print('L8 OK')\"",
      "maps_to_ac": "AC-4"
    }
  ]
}
-->
