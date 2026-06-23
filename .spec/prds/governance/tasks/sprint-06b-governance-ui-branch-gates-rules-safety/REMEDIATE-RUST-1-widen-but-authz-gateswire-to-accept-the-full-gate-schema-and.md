# REMEDIATE-RUST-1: Widen but-authz GatesWire to accept the full [[gate]] schema and expose gates_path()

**Type:** FEATURE | **Status:** Done - satisfied at HEAD b3449afbb2 (per REMEDIATE-06B-C triage) | **Priority:** P0 | **Effort:** S (45 min)
**Agent:** rust-implementer | **Reviewer:** rust-reviewer | **Proposed by:** rust-planner
**Closes red-hat findings:** H1
**Depends on:** (none) | **Blocks:** MGMT-BE-004A
**PRD refs:** UC-MGMT-04, UC-MGMT-06 | **Capabilities:** CAP-AUTHZ-01, CAP-CONFIG-01

## What this does

Extend but-authz's TOML wire schema so it accepts the full [[gate]] review-requirement array consumed by the merge gate, and expose a public gates_path() accessor for the .gitbutler/gates.toml path.

## Why

but-authz parses a gates.toml containing both [[branch]] and [[gate]] arrays without config.invalid, but_authz::gates_path() is publicly exported, and the existing but-authz test suite still passes.

## Scope

- crates/but-authz/src/config.rs (MODIFY — add GateWire, widen GatesWire, add gates_path())
- crates/but-authz/src/lib.rs (MODIFY — re-export gates_path)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: REMEDIATE-RUST-1 — Widen but-authz GatesWire to accept the full [[gate]] schema and expose gates_path()
================================================================================

TASK_TYPE:   FEATURE
STATUS:      Backlog
PRIORITY:    P0
EFFORT:      S  (45 min)
AGENT:       implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-MGMT-04, UC-MGMT-06
CAPABILITIES:CAP-AUTHZ-01,CAP-CONFIG-01
CLOSES:      H1

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Extend but-authz's TOML wire schema so it accepts the full [[gate]] review-requirement array consumed by the merge gate, and expose a public gates_path() accessor for the .gitbutler/gates.toml path.

Success state: but-authz parses a gates.toml containing both [[branch]] and [[gate]] arrays without config.invalid, but_authz::gates_path() is publicly exported, and the existing but-authz test suite still passes.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] Add GateWire and a `gate: Vec<GateWire>` field to but-authz GatesWire exactly mirroring merge_gate.rs:439-467 while keeping #[serde(deny_unknown_fields)]
- [MUST] Add pub fn gates_path() -> &'static str returning '.gitbutler/gates.toml' and re-export it from crates/but-authz/src/lib.rs
- [MUST] Keep the change additive: existing load_governance_config must keep producing the same GovConfig; the new gate field is parsed but does not change existing branch-protection semantics
- [NEVER] Widen public GovConfig or BranchProtection in a way that changes existing caller semantics
- [NEVER] Move or duplicate the GATES_PATH literal outside of the new gates_path() accessor
- [STRICTLY] GateWire must carry branch, type, min_approvals, require_approval_from_group, and require_distinct_from_author fields with the same defaults as merge_gate.rs

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]:
- [ ] AC-2:
- [ ] AC-3:

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]:
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-authz loader over a real gix repo with a committed gates.toml
  SCENARIO:
    tier: visible   test_tier: integration
    verification_service: real but-authz loader over a real gix repo
    negative_control.would_fail_if:
      - #[serde(deny_unknown_fields)] rejects the [[gate]] table
      - the loader silently drops branch protections when a [[gate]] table is present
      - a stub loader returns an empty GovConfig instead of the seeded branch protection
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=?
      action.actor=ci
        - load_governance_config(&repo, "refs/heads/main")
      end_state.must_observe:
        - load_governance_config returns Ok
        - GovConfig branch protection for 'main' is true
      end_state.must_not_observe:
        - config.invalid error
        - missing 'main' branch protection
        - empty GovConfig

AC-2 :
  TEST_TIER: unit   VERIFICATION_SERVICE:
  SCENARIO:
    tier: visible   test_tier: unit
    verification_service:
    negative_control.would_fail_if:
      - gates_path returns a different literal
      - gates_path is not re-exported
      - a stub returns an empty string
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=?
      action.actor=ci
        - assert_eq!(but_authz::gates_path(), ".gitbutler/gates.toml")
      end_state.must_observe:
        - returns the literal '.gitbutler/gates.toml'
      end_state.must_not_observe:
        - any other path literal
        - panic or compilation error

AC-3 :
  TEST_TIER: integration   VERIFICATION_SERVICE: cargo test
  SCENARIO:
    tier: visible   test_tier: integration
    verification_service: cargo test
    negative_control.would_fail_if:
      - the new gate field breaks an existing loader assertion
      - deny_unknown_fields rejects a previously-valid gates.toml
      - a public type signature change breaks downstream crates
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=?
      action.actor=ci
        - cargo test -p but-authz
      end_state.must_observe:
        - cargo test exits 0
        - all but-authz test binaries run
      end_state.must_not_observe:
        - FAILED
        - error[ could not compile
        - new warnings introduced by this change

--------------------------------------------------------------------------------
TEST CRITERIA
--------------------------------------------------------------------------------
- TC-1: A committed gates.toml with a full [[gate]] table parses without config.invalid and keeps branch protections readable
- TC-2: but_authz::gates_path() returns '.gitbutler/gates.toml'
- TC-3: cargo test -p but-authz exits 0 with no regressions

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
- crates/but-authz/src/config.rs  (lines 9, 531-545) — GATES_PATH literal and existing GatesWire/BranchWire shape
- crates/but-api/src/legacy/merge_gate.rs  (lines 439-467) — EXACT GateWire field set to mirror (branch, type, min_approvals, require_approval_from_group, require_distinct_from_author)
- crates/but-authz/src/lib.rs  (lines 17-21) — pub use config block where gates_path must be re-exported

--------------------------------------------------------------------------------
GUARDRAILS
--------------------------------------------------------------------------------
WRITE_ALLOWED:
  - crates/but-authz/src/config.rs (MODIFY — add GateWire, widen GatesWire, add gates_path())
  - crates/but-authz/src/lib.rs (MODIFY — re-export gates_path)
WRITE_PROHIBITED:
  - crates/but-authz/src/authorize.rs, denial.rs, principal.rs, authority.rs — the primitive layer is closed
  - crates/but-authz/src/config.rs loader semantics that change GovConfig/BranchProtection for existing consumers
  - any crate outside but-authz (this is a schema prerequisite task)

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- cargo check -p but-authz --all-targets  →  exit 0
- cargo test -p but-authz  →  exit 0
- cargo clippy -p but-authz --all-targets  →  exit 0

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
depends_on: (none)
blocks:     MGMT-BE-004A

--------------------------------------------------------------------------------
NOTES
--------------------------------------------------------------------------------
Schema prerequisite for MGMT-BE-004A. Does NOT implement a gate-aware GovConfig; the writer task owns lossless round-trip serialization. The existing but-authz loader may continue to ignore the [[gate]] array for normalization, but it must not fail with config.invalid.

```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "REMEDIATE-RUST-1",
  "proposed_by": "rust-planner",
  "supersedes": [],
  "closes_redhat_findings": [
    "H1"
  ],
  "fixtures": {},
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN  WHEN  THEN ",
      "verify": "",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-authz loader over a real gix repo",
        "negative_control": {
          "would_fail_if": [
            "#[serde(deny_unknown_fields)] rejects the [[gate]] table",
            "the loader silently drops branch protections when a [[gate]] table is present",
            "a stub loader returns an empty GovConfig instead of the seeded branch protection"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_state": {
              "description": "A writable gix repo whose refs/heads/main commits a .gitbutler/gates.toml with one [[branch]] main protected=true and one [[gate]] main type='review' min_approvals=2 require_distinct_from_author=true require_approval_from_group=['code-reviewers']",
              "seed_method": "cli",
              "records": [
                "invoke_bash: git init --initial-branch=main; mkdir -p .gitbutler; write gates.toml; git add .gitbutler/gates.toml; git commit -m 'seed gates';"
              ]
            },
            "action": {
              "actor": "ci",
              "steps": [
                "load_governance_config(&repo, \"refs/heads/main\")"
              ]
            },
            "end_state": {
              "must_observe": [
                "load_governance_config returns Ok",
                "GovConfig branch protection for 'main' is true"
              ],
              "must_not_observe": [
                "config.invalid error",
                "missing 'main' branch protection",
                "empty GovConfig"
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
      "verify": "",
      "scenario": {
        "tier": "visible",
        "test_tier": "unit",
        "verification_service": null,
        "negative_control": {
          "would_fail_if": [
            "gates_path returns a different literal",
            "gates_path is not re-exported",
            "a stub returns an empty string"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_state": {
              "description": "but-authz crate compiled with the new pub fn gates_path",
              "seed_method": "public_api",
              "records": [
                "but_authz::gates_path is callable"
              ]
            },
            "action": {
              "actor": "ci",
              "steps": [
                "assert_eq!(but_authz::gates_path(), \".gitbutler/gates.toml\")"
              ]
            },
            "end_state": {
              "must_observe": [
                "returns the literal '.gitbutler/gates.toml'"
              ],
              "must_not_observe": [
                "any other path literal",
                "panic or compilation error"
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
      "verify": "",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "cargo test",
        "negative_control": {
          "would_fail_if": [
            "the new gate field breaks an existing loader assertion",
            "deny_unknown_fields rejects a previously-valid gates.toml",
            "a public type signature change breaks downstream crates"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_state": {
              "description": "Workspace at the commit where this task starts",
              "seed_method": "public_api",
              "records": [
                "existing but-authz unit and integration tests"
              ]
            },
            "action": {
              "actor": "ci",
              "steps": [
                "cargo test -p but-authz"
              ]
            },
            "end_state": {
              "must_observe": [
                "cargo test exits 0",
                "all but-authz test binaries run"
              ],
              "must_not_observe": [
                "FAILED",
                "error[ could not compile",
                "new warnings introduced by this change"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "A committed gates.toml with a full [[gate]] table parses without config.invalid and keeps branch protections readable",
      "verify": "cargo test -p but-authz gates_wire_accepts_full_gate_array",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "but_authz::gates_path() returns '.gitbutler/gates.toml'",
      "verify": "cargo test -p but-authz gates_path_returns_canonical",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "cargo test -p but-authz exits 0 with no regressions",
      "verify": "cargo test -p but-authz",
      "maps_to_ac": "AC-3"
    }
  ]
}
-->
