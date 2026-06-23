# REMEDIATE-RUST-3: administration:read gate for MGMT-BE-003 None-path — close the workspace-rules reconnaissance leak at the Tauri command boundary

**Type:** FEATURE | **Status:** Backlog | **Priority:** P0 | **Effort:** S (60 min)
**Agent:** rust-implementer | **Reviewer:** rust-reviewer | **Proposed by:** rust-planner
**Closes red-hat findings:** M1
**Depends on:** MGMT-IPC-003 (Sprint 06a human-fleet-owner identity shim) | **Blocks:** (none)
**PRD refs:** UC-MGMT-05 | **Capabilities:** CAP-AUTHZ-01

## What this does

Close the workspace-rules reconnaissance leak: a non-admin caller invoking list_workspace_rules_scoped(None) via the Tauri command must be denied perm.denied naming administration:read.

## Why

Non-admin caller denied at the Tauri command boundary; admin caller successfully enumerates rules via None path; Some(principalId) path unaffected (backward compatible).

## Scope

- crates/gitbutler-tauri/src/.../governance.rs (the Tauri command wrapper for list_workspace_rules_scoped — add the admin:read gate)
- crates/gitbutler-tauri/tests/list_workspace_rules_scoped.rs (NEW — the 3 AC tests)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: REMEDIATE-RUST-3 — administration:read gate for MGMT-BE-003 None-path — close the workspace-rules reconnaissance leak at the Tauri command boundary
================================================================================

TASK_TYPE:   FEATURE
STATUS:      Backlog
PRIORITY:    P0
EFFORT:      S  (60 min)
AGENT:       implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-MGMT-05
CAPABILITIES:CAP-AUTHZ-01
CLOSES:      M1

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Close the workspace-rules reconnaissance leak: a non-admin caller invoking list_workspace_rules_scoped(None) via the Tauri command must be denied perm.denied naming administration:read.

Success state: Non-admin caller denied at the Tauri command boundary; admin caller successfully enumerates rules via None path; Some(principalId) path unaffected (backward compatible).

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST enforce administration:read at the TAURI COMMAND BOUNDARY wrapping list_workspace_rules_scoped(None), NOT at the but-api layer — MGMT-BE-003 AC-5 already exercises but-api env resolution; that test passes today against the ungoverned list_workspace_rules. The gate belongs where the renderer-facing identity is resolved (the human fleet-owner via UserService, per T-MGMT-042). Closes red-hat-20260622T145305Z.md Gap 4.
- [MUST] MUST NOT alter the list_workspace_rules_scoped(&ctx, principalId) signature — the gate is in the Tauri command wrapper, transparent to but-api callers (the CLI keeps working without a cookie).
- [MUST] MUST NOT gate the Some(principalId) path — that path already has its own scope check (cross-principal scoping per MGMT-BE-003 AC-5). Only the None path leaks; only the None path gets the gate.
- [NEVER] NEVER ship an accepted-leak annotation without explicit PRD sign-off (R6/R12-style) — if you judge the gate is wrong, write the annotation, name the principal signer, and route to /kb-prd-plan
- [NEVER] NEVER check administration:write for a READ operation — administration:read is the correct axis (mirrors branch_gates_read per MGMT-BE-004A AC-8)
- [STRICTLY] STRICTLY resolve the caller via UserService (T-MGMT-042) in the desktop process — BUT_AGENT_HANDLE is UNSET there (per MGMT-BE-004 SEC-3)

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: returns Err perm.denied with .code naming 'administration:read'; AND the underlying list_workspace_rules is NOT called (SDK spy count == 0 — the gate ran BEFORE delegation)
- [ ] AC-2 [PRIMARY]: returns Ok containing the workspace rule list (non-empty); the SDK list_workspace_rules is called exactly once
- [ ] AC-3 [PRIMARY]: returns Ok with principal-A's rules; the existing cross-principal scope check is used, NOT the new admin:read gate

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: returns Err perm.denied with .code naming 'administration:read'; AND the underlying list_workspace_rules is NOT called (SDK spy count == 0 — the gate ran BEFORE delegation)
  GIVEN: Tauri command wrapping list_workspace_rules_scoped(None); caller resolved via UserService (T-MGMT-042 fleet-owner identity) holds ['contents:write'] only (NO administration:read)
  WHEN: the Tauri command is invoked with BUT_AGENT_HANDLE unset (desktop process — the wrapper resolves the human identity)
  THEN: returns Err perm.denied with .code naming 'administration:read'; AND the underlying list_workspace_rules is NOT called (SDK spy count == 0 — the gate ran BEFORE delegation)
  TEST_TIER: integration   VERIFICATION_SERVICE: real gitbutler-tauri Tauri command + real UserService identity resolution + sanctioned but-sdk spy
  VERIFY: cargo test -p gitbutler-tauri list_workspace_rules_scoped_none_denies_without_admin_read
  SCENARIO:
    tier: visible   test_tier: integration
    verification_service: real gitbutler-tauri Tauri command + real UserService identity resolution + sanctioned but-sdk spy
    negative_control.would_fail_if:
      - command forwards to but-api without checking administration:read (the recon leak remains — list_workspace_rules IS called and returns data)
      - command checks only administration:write (wrong axis — read op; a non-admin-with-write would pass, but more importantly the denial message would name the wrong permission)
      - command resolves caller from a UI-supplied cookie without server-side UserService verification (renderer bypass — any cookie value works)
      - command checks administration:read AFTER calling list_workspace_rules (recon leak happens before denial — the spy count would be >= 1)
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=nonadmin_without_admin_read
      action.actor=ci
        - spawn the desktop Tauri command wrapper with UserService seeding the caller identity as contents:write-only
        - invoke the Tauri command list_workspace_rules_scoped(None) (or its test-harness equivalent)
        - install a but-sdk spy on list_workspace_rules before the call
        - capture the returned Result and the spy call count
      end_state.must_observe:
        - returned Err whose .code == 'perm.denied'
        - the denial message contains the literal 'administration:read'
        - but-sdk spy on list_workspace_rules reports call_count == 0
        - deny was issued by the Tauri wrapper (caller identity resolved via UserService), not by an inner but-api gate (no BUT_AGENT_HANDLE env involvement)
      end_state.must_not_observe:
        - returned Ok with any rule list (gate missing entirely)
        - generic 500 error without a code field (unstructured denial)
        - but-sdk spy call_count >= 1 (gate ran AFTER delegation — recon leak)
        - denial message naming 'administration:write' (wrong axis — read op)

AC-2 [PRIMARY]: returns Ok containing the workspace rule list (non-empty); the SDK list_workspace_rules is called exactly once
  GIVEN: caller resolved via UserService holds administration:read; workspace seeded with >= 2 rules
  WHEN: the Tauri command is invoked with list_workspace_rules_scoped(None)
  THEN: returns Ok containing the workspace rule list (non-empty); the SDK list_workspace_rules is called exactly once
  TEST_TIER: integration   VERIFICATION_SERVICE: real gitbutler-tauri Tauri command + real UserService identity resolution
  VERIFY: cargo test -p gitbutler-tauri list_workspace_rules_scoped_none_admin_reads
  SCENARIO:
    tier: visible   test_tier: integration
    verification_service: real gitbutler-tauri Tauri command + real UserService identity resolution
    negative_control.would_fail_if:
      - gate inverted (denies admin)
      - gate too narrow (allows only administration:write holders)
      - list_workspace_rules itself broken
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=admin_with_admin_read
      action.actor=ci
        - seed the caller identity via UserService with administration:read
        - seed the workspace with >= 2 rules
        - invoke the Tauri command list_workspace_rules_scoped(None)
        - capture the returned Result and the SDK call count
      end_state.must_observe:
        - returned Ok containing a non-empty rule list
        - the rule list contains >= 2 seeded rules
        - SDK list_workspace_rules called exactly once
      end_state.must_not_observe:
        - perm.denied returned
        - SDK called list_workspace_rules more than once

AC-3 [PRIMARY]: returns Ok with principal-A's rules; the existing cross-principal scope check is used, NOT the new admin:read gate
  GIVEN: caller lacking administration:read; an existing seeded principal-A rule; invoking the Some(principalId) path
  WHEN: the Tauri command is invoked with list_workspace_rules_scoped(Some(principal-A)) for the principal's own rules (self-scope)
  THEN: returns Ok with principal-A's rules; the existing cross-principal scope check is used, NOT the new admin:read gate
  TEST_TIER: integration   VERIFICATION_SERVICE: real gitbutler-tauri Tauri command + real UserService identity resolution
  VERIFY: cargo test -p gitbutler-tauri list_workspace_rules_scoped_some_unchanged
  SCENARIO:
    tier: visible   test_tier: integration
    verification_service: real gitbutler-tauri Tauri command + real UserService identity resolution
    negative_control.would_fail_if:
      - gate over-broadly fires on Some path
      - existing cross-principal scoping (MGMT-BE-003 AC-5) regressed
      - self-scope now requires admin:read
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=self_scope_principal
      action.actor=ci
        - seed a caller without administration:read
        - seed an existing principal-A rule
        - invoke the Tauri command list_workspace_rules_scoped(Some(principal-A))
        - capture the returned Result and confirm which gate applied
      end_state.must_observe:
        - returned Ok with principal-A's rules
        - administration:read gate NOT invoked for Some path
      end_state.must_not_observe:
        - perm.denied returned for self-scope
        - Some path gated by administration:read

--------------------------------------------------------------------------------
TEST CRITERIA
--------------------------------------------------------------------------------
- TC-1:
- TC-2:
- TC-3:

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
- crates/but-api/src/legacy/governance.rs  (lines ?) — Verify the ungoverned list_workspace_rules — the leak this task closes
- crates/but-api/src/legacy/governance.rs:1467-1478  (lines ?) — Mirror this administration:read enforcement pattern at the Tauri wrapper
- crates/gitbutler-tauri/src/.../governance.rs  (lines ?) — Where the wrapper lives — the correct layer for the gate
- .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-BE-003-principal-scoped-rules-query.md:1-250  (lines ?) — The augmented contract — note AC-1 delegates None to list_workspace_rules; this remedial task adds the missing gate at the wrapper layer

--------------------------------------------------------------------------------
GUARDRAILS
--------------------------------------------------------------------------------
WRITE_ALLOWED:
  - crates/gitbutler-tauri/src/.../governance.rs (the Tauri command wrapper for list_workspace_rules_scoped — add the admin:read gate)
  - crates/gitbutler-tauri/tests/list_workspace_rules_scoped.rs (NEW — the 3 AC tests)
WRITE_PROHIBITED:
  - crates/but-api/src/** (no but-api signature change)
  - crates/but-authz/**
  - crates/but-server/**

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
depends_on: MGMT-IPC-003 (Sprint 06a human-fleet-owner identity shim)
blocks:     (none)

--------------------------------------------------------------------------------
NOTES
--------------------------------------------------------------------------------
Closes red-hat M1. Augments (does not supersede) MGMT-BE-003 — the original contract's Some(principalId) scoping is correct; only the None path leaks. Per Gap 4, the gate lives at the Tauri wrapper (where the human identity is resolved), not at but-api (where BUT_AGENT_HANDLE env resolution already passes the existing AC-5).

```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "REMEDIATE-RUST-3",
  "proposed_by": "rust-planner",
  "supersedes": [],
  "closes_redhat_findings": [
    "M1"
  ],
  "fixtures": {
    "nonadmin_without_admin_read": {
      "description": "Human fleet-owner caller resolved via UserService holding only contents:write, lacking administration:read.",
      "seed_method": "UserService test helper in gitbutler-tauri integration harness",
      "records": [
        "identity: fleet-owner@test.local",
        "capabilities: [\"contents:write\"]"
      ]
    },
    "admin_with_admin_read": {
      "description": "Administrator caller resolved via UserService holding administration:read.",
      "seed_method": "UserService test helper in gitbutler-tauri integration harness",
      "records": [
        "identity: admin@test.local",
        "capabilities: [\"administration:read\", \"contents:write\"]"
      ]
    },
    "self_scope_principal": {
      "description": "A seeded workspace principal and one rule owned by that principal, used to exercise the Some(principalId) self-scope path.",
      "seed_method": "Workspace rules seed helper in gitbutler-tauri integration harness",
      "records": [
        "principal_id: principal-A",
        "rule: administration:write scoped to principal-A"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN Tauri command wrapping list_workspace_rules_scoped(None); caller resolved via UserService (T-MGMT-042 fleet-owner identity) holds ['contents:write'] only (NO administration:read) WHEN the Tauri command is invoked with BUT_AGENT_HANDLE unset (desktop process \u2014 the wrapper resolves the human identity) THEN returns Err perm.denied with .code naming 'administration:read'; AND the underlying list_workspace_rules is NOT called (SDK spy count == 0 \u2014 the gate ran BEFORE delegation)",
      "verify": "cargo test -p gitbutler-tauri list_workspace_rules_scoped_none_denies_without_admin_read",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real gitbutler-tauri Tauri command + real UserService identity resolution + sanctioned but-sdk spy",
        "negative_control": {
          "would_fail_if": [
            "command forwards to but-api without checking administration:read (the recon leak remains \u2014 list_workspace_rules IS called and returns data)",
            "command checks only administration:write (wrong axis \u2014 read op; a non-admin-with-write would pass, but more importantly the denial message would name the wrong permission)",
            "command resolves caller from a UI-supplied cookie without server-side UserService verification (renderer bypass \u2014 any cookie value works)",
            "command checks administration:read AFTER calling list_workspace_rules (recon leak happens before denial \u2014 the spy count would be >= 1)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "nonadmin_without_admin_read",
            "action": {
              "actor": "ci",
              "steps": [
                "spawn the desktop Tauri command wrapper with UserService seeding the caller identity as contents:write-only",
                "invoke the Tauri command list_workspace_rules_scoped(None) (or its test-harness equivalent)",
                "install a but-sdk spy on list_workspace_rules before the call",
                "capture the returned Result and the spy call count"
              ]
            },
            "end_state": {
              "must_observe": [
                "returned Err whose .code == 'perm.denied'",
                "the denial message contains the literal 'administration:read'",
                "but-sdk spy on list_workspace_rules reports call_count == 0",
                "deny was issued by the Tauri wrapper (caller identity resolved via UserService), not by an inner but-api gate (no BUT_AGENT_HANDLE env involvement)"
              ],
              "must_not_observe": [
                "returned Ok with any rule list (gate missing entirely)",
                "generic 500 error without a code field (unstructured denial)",
                "but-sdk spy call_count >= 1 (gate ran AFTER delegation \u2014 recon leak)",
                "denial message naming 'administration:write' (wrong axis \u2014 read op)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN caller resolved via UserService holds administration:read; workspace seeded with >= 2 rules WHEN the Tauri command is invoked with list_workspace_rules_scoped(None) THEN returns Ok containing the workspace rule list (non-empty); the SDK list_workspace_rules is called exactly once",
      "verify": "cargo test -p gitbutler-tauri list_workspace_rules_scoped_none_admin_reads",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real gitbutler-tauri Tauri command + real UserService identity resolution",
        "negative_control": {
          "would_fail_if": [
            "gate inverted (denies admin)",
            "gate too narrow (allows only administration:write holders)",
            "list_workspace_rules itself broken"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "admin_with_admin_read",
            "action": {
              "actor": "ci",
              "steps": [
                "seed the caller identity via UserService with administration:read",
                "seed the workspace with >= 2 rules",
                "invoke the Tauri command list_workspace_rules_scoped(None)",
                "capture the returned Result and the SDK call count"
              ]
            },
            "end_state": {
              "must_observe": [
                "returned Ok containing a non-empty rule list",
                "the rule list contains >= 2 seeded rules",
                "SDK list_workspace_rules called exactly once"
              ],
              "must_not_observe": [
                "perm.denied returned",
                "SDK called list_workspace_rules more than once"
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
      "description": "GIVEN caller lacking administration:read; an existing seeded principal-A rule; invoking the Some(principalId) path WHEN the Tauri command is invoked with list_workspace_rules_scoped(Some(principal-A)) for the principal's own rules (self-scope) THEN returns Ok with principal-A's rules; the existing cross-principal scope check is used, NOT the new admin:read gate",
      "verify": "cargo test -p gitbutler-tauri list_workspace_rules_scoped_some_unchanged",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real gitbutler-tauri Tauri command + real UserService identity resolution",
        "negative_control": {
          "would_fail_if": [
            "gate over-broadly fires on Some path",
            "existing cross-principal scoping (MGMT-BE-003 AC-5) regressed",
            "self-scope now requires admin:read"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "self_scope_principal",
            "action": {
              "actor": "ci",
              "steps": [
                "seed a caller without administration:read",
                "seed an existing principal-A rule",
                "invoke the Tauri command list_workspace_rules_scoped(Some(principal-A))",
                "capture the returned Result and confirm which gate applied"
              ]
            },
            "end_state": {
              "must_observe": [
                "returned Ok with principal-A's rules",
                "administration:read gate NOT invoked for Some path"
              ],
              "must_not_observe": [
                "perm.denied returned for self-scope",
                "Some path gated by administration:read"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "",
      "verify": "cargo test -p gitbutler-tauri list_workspace_rules_scoped_none_denies_without_admin_read",
      "maps_to_ac": null
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "",
      "verify": "cargo test -p gitbutler-tauri list_workspace_rules_scoped_none_admin_reads",
      "maps_to_ac": null
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "",
      "verify": "cargo test -p gitbutler-tauri list_workspace_rules_scoped_some_unchanged",
      "maps_to_ac": null
    }
  ]
}
-->
