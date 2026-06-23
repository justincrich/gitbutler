# E2E-MGMT-BE-002A: Test-only successor to E2E-MGMT-BE-002 — integration tests for the 16 already-registered governance routes + completed no-bypass proof

**Type:** FEATURE | **Status:** Backlog | **Priority:** P0 | **Effort:** M (90 min)
**Agent:** rust-implementer | **Reviewer:** rust-reviewer | **Proposed by:** rust-planner
**Closes red-hat findings:** H2, H3

**Supersedes:** E2E-MGMT-BE-002 (per `red-hat-20260622T145305Z.md` — original scope was unimplementable / false premise)
**Depends on:** E2E-MGMT-BE-001 | **Blocks:** E2E-MGMT-UI-001
**PRD refs:** UC-MGMT-04, UC-MGMT-06 | **Capabilities:** CAP-AUTHZ-01

## What this does

Add integration tests (crates/but-server/tests/governance_routes.rs) for the 16 already-registered governance routes, completing the no-bypass proof the original contract marked PARTIAL.

## Why

Supersedes E2E-MGMT-BE-002 per red-hat-20260622T145305Z.md (H2, H3). Every governance route has at least one happy-path + one denial test; the primary no-bypass proof (non-admin POST /branch_gates_update) passes; the \*\_as_fleet_owner bypass structural gate passes.

## Scope

- crates/but-server/tests/governance_routes.rs (NEW)
- crates/but-server/tests/bypass_grep.rs (NEW)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: E2E-MGMT-BE-002A — Test-only successor to E2E-MGMT-BE-002 — integration tests for the 16 already-registered governance routes + completed no-bypass proof
================================================================================

TASK_TYPE:   FEATURE
STATUS:      Backlog
PRIORITY:    P0
EFFORT:      M  (90 min)
AGENT:       implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-MGMT-04, UC-MGMT-06
CAPABILITIES:CAP-AUTHZ-01
SUPERSEDES:  E2E-MGMT-BE-002
CLOSES:      H2, H3

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Add integration tests (crates/but-server/tests/governance_routes.rs) for the 16 already-registered governance routes, completing the no-bypass proof the original contract marked PARTIAL.

Success state: Every governance route has at least one happy-path + one denial test; the primary no-bypass proof (non-admin POST /branch_gates_update) passes; the *_as_fleet_owner bypass structural gate passes.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST supersede E2E-MGMT-BE-002 — the original contract's premise ('Today but-server routes zero governance commands') was factually false; live crates/but-server/src/lib.rs:34-99 has 16 governance routes. This successor reframes the task as TEST-ONLY: add integration tests for the already-registered routes and COMPLETE the primary no-bypass proof the original marked PARTIAL (red-hat-20260622T145305Z.md H2).
- [MUST] MUST exercise /branch_gates_update under NONADMIN_HANDLE in AC-3 and assert (a) HTTP response body contains code='perm.denied' naming 'administration:write', (b) governance state (committed gates.toml blob) byte-for-byte unchanged after the denial
- [MUST] MUST broaden the *_as_fleet_owner bypass grep to cover all three routing surfaces: ! grep -rq '_as_fleet_owner' crates/but-server/src crates/gitbutler-tauri/src crates/but-napi/src (the original grep checked only but-server and missed the 9 *_as_fleet_owner variants in but-api)
- [MUST] MUST add a positive assertion: every *_cmd symbol invoked in but-server routes resolves to a #[but_api]-attributed function in but-api (not a *_as_fleet_owner variant). The aspirational 'macro doesn't emit _cmd for un-attributed fns' claim becomes an automated structural check.
- [NEVER] NEVER register new governance routes — they already exist (16 in lib.rs:34-99); this task adds TESTS, not routes
- [NEVER] NEVER weaken the AC-3 oracle to 'returns Err' without (a) the perm.denied code/message assertion AND (b) the byte-for-byte-unchanged state assertion
- [NEVER] NEVER modify the production governance.rs handler logic — the test exercises existing code
- [STRICTLY] STRICTLY seed via the E2E-MGMT-BE-001 governed-repo fixtures (admin + non-admin identities, real git commits to the target ref) — no mocked identities

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1: each returns HTTP 2xx or 4xx (NOT 404)
- [ ] AC-2: HTTP 2xx; working-tree gates.toml updated to min_approvals=3; branch_gates_read shows pending
- [ ] AC-3: HTTP response body contains code='perm.denied' AND message contains 'administration:write'; AND the committed gates.toml blob is byte-for-byte UNCHANGED after the denial
- [ ] AC-4: `grep -rq '_as_fleet_owner' crates/but-server/src crates/gitbutler-tauri/src crates/but-napi/src` returns 0 matches; AND every *_cmd symbol invoked in but-server's GOVERNANCE_COMMAND_ROUTES resolves (via AST or grep) to a #[but_api]-attributed function in crates/but-api/src/legacy/governance.rs (NOT a *_as_fleet_owner variant); AND exits non-zero naming the bypass function
- [ ] AC-5: exit 0

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 : each returns HTTP 2xx or 4xx (NOT 404)
  GIVEN: but-server boot with seeded governed repo
  WHEN: each of the 16 GOVERNANCE_COMMAND_ROUTES is invoked as admin
  THEN: each returns HTTP 2xx or 4xx (NOT 404)
  TEST_TIER: smoke   VERIFICATION_SERVICE: cargo
  VERIFY: cargo test -p but-server governance_routes_reachable
  SCENARIO:
    tier: smoke   test_tier: smoke
    verification_service: cargo
    negative_control.would_fail_if:
      - route missing
      - route mapped to wrong path
      - but-server fails to boot
    evidence: artifact_type=test_output required_capture=stdout showing 16 route assertions pass
    case[0] start_ref=E2E-MGMT-BE-001 governed-repo fixtures + admin identity
      action.actor=ADMIN_HANDLE
        - boot but-server
        - iterate GOVERNANCE_COMMAND_ROUTES
        - GET/POST each route with valid minimal payload
      end_state.must_observe:
        - HTTP status is 2xx or 4xx for every route
        - no 404 responses
      end_state.must_not_observe:
        - route returns 404
        - server panic

AC-2 : HTTP 2xx; working-tree gates.toml updated to min_approvals=3; branch_gates_read shows pending
  GIVEN: E2E-MGMT-BE-001 fixtures: governed repo + admin identity (AdministrationWrite)
  WHEN: POST /branch_gates_update with edit{branch='main', min_approvals=3}
  THEN: HTTP 2xx; working-tree gates.toml updated to min_approvals=3; branch_gates_read shows pending
  TEST_TIER: happy   VERIFICATION_SERVICE: cargo
  VERIFY: cargo test -p but-server admin_branch_gates_update_round_trips
  SCENARIO:
    tier: PRIMARY happy   test_tier: happy
    verification_service: cargo
    negative_control.would_fail_if:
      - route not invoking but-api
      - route invoking but-api without enforcing admin gate (would still succeed for admin — AC-3 catches denial)
      - response missing pending signal
    evidence: artifact_type=test_output required_capture=assertions for HTTP 2xx, gates.toml content, pending signal
    case[0] start_ref=E2E-MGMT-BE-001 governed-repo fixtures + admin identity
      action.actor=ADMIN_HANDLE
        - POST /branch_gates_update
        - payload sets branch='main' min_approvals=3
      end_state.must_observe:
        - HTTP 2xx
        - gates.toml min_approvals=3
        - branch_gates_read shows pending
      end_state.must_not_observe:
        - HTTP 4xx/5xx
        - gates.toml unchanged

AC-3 : HTTP response body contains code='perm.denied' AND message contains 'administration:write'; AND the committed gates.toml blob is byte-for-byte UNCHANGED after the denial
  GIVEN: E2E-MGMT-BE-001 fixtures: governed repo + NONADMIN_HANDLE identity (holds contents:write only); committed gates.toml blob captured byte-for-byte
  WHEN: POST /branch_gates_update with edit{branch='main', protected=false}
  THEN: HTTP response body contains code='perm.denied' AND message contains 'administration:write'; AND the committed gates.toml blob is byte-for-byte UNCHANGED after the denial
  TEST_TIER: PRIMARY NO-BYPASS   VERIFICATION_SERVICE: cargo
  VERIFY: cargo test -p but-server non_admin_branch_gates_update_denied_state_unchanged
  SCENARIO:
    tier: PRIMARY NO-BYPASS   test_tier: PRIMARY NO-BYPASS
    verification_service: cargo
    negative_control.would_fail_if:
      - route exposes but-api fn without invoking but-authz gate (denial never fires)
      - route maps to a *_as_fleet_owner variant (bypass — denial never fires)
      - route returns generic 500 instead of structured perm.denied
      - response omits 'administration:write' in the message (identity of the failed gate unclear)
      - state changes despite the denial (write-before-gate bug)
    evidence: artifact_type=test_output required_capture=HTTP body, perm.denied code, administration:write message, blob hash before == after
    case[0] start_ref=E2E-MGMT-BE-001 governed-repo fixtures + NONADMIN_HANDLE; capture committed gates.toml blob hash
      action.actor=NONADMIN_HANDLE
        - POST /branch_gates_update
        - payload sets branch='main' protected=false
      end_state.must_observe:
        - HTTP body code='perm.denied'
        - message contains 'administration:write'
        - committed blob byte-hash AFTER == BEFORE
      end_state.must_not_observe:
        - HTTP 2xx
        - generic 500 without code
        - blob changed

AC-4 : `grep -rq '_as_fleet_owner' crates/but-server/src crates/gitbutler-tauri/src crates/but-napi/src` returns 0 matches; AND every *_cmd symbol invoked in but-server's GOVERNANCE_COMMAND_ROUTES resolves (via AST or grep) to a #[but_api]-attributed function in crates/but-api/src/legacy/governance.rs (NOT a *_as_fleet_owner variant); AND exits non-zero naming the bypass function
  GIVEN: production codebase
  WHEN: the bypass_grep test (crates/but-server/tests/bypass_grep.rs) runs; AND a planted #[but_api] attribute on branch_gates_update_with_repo_as_fleet_owner
  THEN: `grep -rq '_as_fleet_owner' crates/but-server/src crates/gitbutler-tauri/src crates/but-napi/src` returns 0 matches; AND every *_cmd symbol invoked in but-server's GOVERNANCE_COMMAND_ROUTES resolves (via AST or grep) to a #[but_api]-attributed function in crates/but-api/src/legacy/governance.rs (NOT a *_as_fleet_owner variant); AND exits non-zero naming the bypass function
  TEST_TIER: STRUCTURAL BYPASS GATE   VERIFICATION_SERVICE: cargo
  VERIFY: cargo test -p but-server governance_bypass_grep
  SCENARIO:
    tier: STRUCTURAL BYPASS GATE   test_tier: STRUCTURAL BYPASS GATE
    verification_service: cargo
    negative_control.would_fail_if:
      - grep scopes only but-server (misses gitbutler-tauri / but-napi)
      - positive attribution missing (an *_as_fleet_owner fn could be wired in without detection)
      - grep pattern too narrow (matches only exact string '_as_fleet_owner(')
    evidence: artifact_type=test_output required_capture=grep results showing 0 matches on routing surfaces; positive attribution list; planted bypass failure output
    case[0] start_ref=production codebase
      action.actor=test harness
        - grep '_as_fleet_owner' across but-server/src, gitbutler-tauri/src, but-napi/src
        - map each *_cmd in GOVERNANCE_COMMAND_ROUTES to its #[but_api]-attributed definition in but-api/src/legacy/governance.rs
      end_state.must_observe:
        - 0 matches for '_as_fleet_owner' on routing surfaces
        - every *_cmd resolves to #[but_api] fn
        - planted bypass function triggers non-zero exit naming it
      end_state.must_not_observe:
        - unattributed *_cmd symbol
        - *_as_fleet_owner match on routing surface

AC-5 : exit 0
  GIVEN: all tests land
  WHEN: cargo test -p but-server && cargo clippy -p but-server --all-targets
  THEN: exit 0
  TEST_TIER: build   VERIFICATION_SERVICE: cargo
  VERIFY: cargo test -p but-server && cargo clippy -p but-server --all-targets
  SCENARIO:
    tier: build   test_tier: build
    verification_service: cargo
    negative_control.would_fail_if:
      - compilation error
      - clippy warning treated as error
      - test panic
    evidence: artifact_type=command_output required_capture=exit code 0 for both commands
    case[0] start_ref=tests implemented
      action.actor=developer
        - cargo test -p but-server
        - cargo clippy -p but-server --all-targets
      end_state.must_observe:
        - cargo test exits 0
        - cargo clippy exits 0
      end_state.must_not_observe:
        - non-zero exit
        - clippy deny-level warning

--------------------------------------------------------------------------------
TEST CRITERIA
--------------------------------------------------------------------------------
- TC-1: Every one of the 16 GOVERNANCE_COMMAND_ROUTES returns a non-404 status when invoked as admin.
- TC-2: Admin POST /branch_gates_update round-trips a gate edit and yields a pending signal.
- TC-3: Non-admin POST /branch_gates_update is denied with code='perm.denied' naming 'administration:write' and leaves the committed gates.toml blob unchanged.
- TC-4: No *_as_fleet_owner bypass symbol exists on the routing surfaces and every *_cmd in GOVERNANCE_COMMAND_ROUTES is positively attributed to a #[but_api] function.
- TC-5: but-server tests and clippy pass cleanly.

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
- crates/but-server/src/lib.rs  (lines 34-99) — GOVERNANCE_COMMAND_ROUTES — the 16 already-registered routes this task tests
- crates/but-api/src/legacy/governance.rs  (lines 591,1193-1585) — The 9 *_as_fleet_owner bypass variants — the grep target
- crates/but-api/src/legacy/governance.rs  (lines 556-622) — branch_gates_*_with_repo — the #[but_api]-attributed fns the positive attribution grep expects
- .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/E2E-MGMT-BE-002-but-server-governance-routes.md  (lines 1-250) — The superseded contract — preserves the AC INTENT but reframes the false premise

--------------------------------------------------------------------------------
GUARDRAILS
--------------------------------------------------------------------------------
WRITE_ALLOWED:
  - crates/but-server/tests/governance_routes.rs (NEW)
  - crates/but-server/tests/bypass_grep.rs (NEW)
WRITE_PROHIBITED:
  - crates/but-server/src/** (no production changes)
  - crates/but-api/**
  - crates/but-authz/**
  - crates/gitbutler-tauri/**
  - crates/but-napi/**

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- cargo test -p but-server governance_  →  all governance route tests + bypass grep pass
- cargo clippy -p but-server --all-targets  →  exit 0 with no deny-level warnings

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
depends_on: E2E-MGMT-BE-001
blocks:     E2E-MGMT-UI-001

--------------------------------------------------------------------------------
NOTES
--------------------------------------------------------------------------------
Successor to E2E-MGMT-BE-002 per red-hat H2. Closes H3 via AC-4 (broadened *_as_fleet_owner grep + positive attribution). Folds the REMEDIATE-RUST-5 build-gate INTO AC-4 — no separate REMEDIATE-RUST-5 file needed. The original E2E-MGMT-BE-002 file is preserved; mark it 'Superseded by E2E-MGMT-BE-002A' in its header.

```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "E2E-MGMT-BE-002A",
  "proposed_by": "rust-planner",
  "supersedes": [
    "E2E-MGMT-BE-002"
  ],
  "closes_redhat_findings": [
    "H2",
    "H3"
  ],
  "fixtures": {},
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN but-server boot with seeded governed repo WHEN each of the 16 GOVERNANCE_COMMAND_ROUTES is invoked as admin THEN each returns HTTP 2xx or 4xx (NOT 404)",
      "verify": "cargo test -p but-server governance_routes_reachable",
      "scenario": {
        "tier": "smoke",
        "test_tier": "smoke",
        "verification_service": "cargo",
        "negative_control": {
          "would_fail_if": [
            "route missing",
            "route mapped to wrong path",
            "but-server fails to boot"
          ]
        },
        "evidence": {
          "artifact_type": "test_output",
          "required_capture": "stdout showing 16 route assertions pass"
        },
        "cases": [
          {
            "start_ref": "E2E-MGMT-BE-001 governed-repo fixtures + admin identity",
            "action": {
              "actor": "ADMIN_HANDLE",
              "steps": [
                "boot but-server",
                "iterate GOVERNANCE_COMMAND_ROUTES",
                "GET/POST each route with valid minimal payload"
              ]
            },
            "end_state": {
              "must_observe": [
                "HTTP status is 2xx or 4xx for every route",
                "no 404 responses"
              ],
              "must_not_observe": [
                "route returns 404",
                "server panic"
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
      "description": "GIVEN E2E-MGMT-BE-001 fixtures: governed repo + admin identity (AdministrationWrite) WHEN POST /branch_gates_update with edit{branch='main', min_approvals=3} THEN HTTP 2xx; working-tree gates.toml updated to min_approvals=3; branch_gates_read shows pending",
      "verify": "cargo test -p but-server admin_branch_gates_update_round_trips",
      "scenario": {
        "tier": "PRIMARY happy",
        "test_tier": "happy",
        "verification_service": "cargo",
        "negative_control": {
          "would_fail_if": [
            "route not invoking but-api",
            "route invoking but-api without enforcing admin gate (would still succeed for admin \u2014 AC-3 catches denial)",
            "response missing pending signal"
          ]
        },
        "evidence": {
          "artifact_type": "test_output",
          "required_capture": "assertions for HTTP 2xx, gates.toml content, pending signal"
        },
        "cases": [
          {
            "start_ref": "E2E-MGMT-BE-001 governed-repo fixtures + admin identity",
            "action": {
              "actor": "ADMIN_HANDLE",
              "steps": [
                "POST /branch_gates_update",
                "payload sets branch='main' min_approvals=3"
              ]
            },
            "end_state": {
              "must_observe": [
                "HTTP 2xx",
                "gates.toml min_approvals=3",
                "branch_gates_read shows pending"
              ],
              "must_not_observe": [
                "HTTP 4xx/5xx",
                "gates.toml unchanged"
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
      "description": "GIVEN E2E-MGMT-BE-001 fixtures: governed repo + NONADMIN_HANDLE identity (holds contents:write only); committed gates.toml blob captured byte-for-byte WHEN POST /branch_gates_update with edit{branch='main', protected=false} THEN HTTP response body contains code='perm.denied' AND message contains 'administration:write'; AND the committed gates.toml blob is byte-for-byte UNCHANGED after the denial",
      "verify": "cargo test -p but-server non_admin_branch_gates_update_denied_state_unchanged",
      "scenario": {
        "tier": "PRIMARY NO-BYPASS",
        "test_tier": "PRIMARY NO-BYPASS",
        "verification_service": "cargo",
        "negative_control": {
          "would_fail_if": [
            "route exposes but-api fn without invoking but-authz gate (denial never fires)",
            "route maps to a *_as_fleet_owner variant (bypass \u2014 denial never fires)",
            "route returns generic 500 instead of structured perm.denied",
            "response omits 'administration:write' in the message (identity of the failed gate unclear)",
            "state changes despite the denial (write-before-gate bug)"
          ]
        },
        "evidence": {
          "artifact_type": "test_output",
          "required_capture": "HTTP body, perm.denied code, administration:write message, blob hash before == after"
        },
        "cases": [
          {
            "start_ref": "E2E-MGMT-BE-001 governed-repo fixtures + NONADMIN_HANDLE; capture committed gates.toml blob hash",
            "action": {
              "actor": "NONADMIN_HANDLE",
              "steps": [
                "POST /branch_gates_update",
                "payload sets branch='main' protected=false"
              ]
            },
            "end_state": {
              "must_observe": [
                "HTTP body code='perm.denied'",
                "message contains 'administration:write'",
                "committed blob byte-hash AFTER == BEFORE"
              ],
              "must_not_observe": [
                "HTTP 2xx",
                "generic 500 without code",
                "blob changed"
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
      "description": "GIVEN production codebase WHEN the bypass_grep test (crates/but-server/tests/bypass_grep.rs) runs; AND a planted #[but_api] attribute on branch_gates_update_with_repo_as_fleet_owner THEN `grep -rq '_as_fleet_owner' crates/but-server/src crates/gitbutler-tauri/src crates/but-napi/src` returns 0 matches; AND every *_cmd symbol invoked in but-server's GOVERNANCE_COMMAND_ROUTES resolves (via AST or grep) to a #[but_api]-attributed function in crates/but-api/src/legacy/governance.rs (NOT a *_as_fleet_owner variant); AND exits non-zero naming the bypass function",
      "verify": "cargo test -p but-server governance_bypass_grep",
      "scenario": {
        "tier": "STRUCTURAL BYPASS GATE",
        "test_tier": "STRUCTURAL BYPASS GATE",
        "verification_service": "cargo",
        "negative_control": {
          "would_fail_if": [
            "grep scopes only but-server (misses gitbutler-tauri / but-napi)",
            "positive attribution missing (an *_as_fleet_owner fn could be wired in without detection)",
            "grep pattern too narrow (matches only exact string '_as_fleet_owner(')"
          ]
        },
        "evidence": {
          "artifact_type": "test_output",
          "required_capture": "grep results showing 0 matches on routing surfaces; positive attribution list; planted bypass failure output"
        },
        "cases": [
          {
            "start_ref": "production codebase",
            "action": {
              "actor": "test harness",
              "steps": [
                "grep '_as_fleet_owner' across but-server/src, gitbutler-tauri/src, but-napi/src",
                "map each *_cmd in GOVERNANCE_COMMAND_ROUTES to its #[but_api]-attributed definition in but-api/src/legacy/governance.rs"
              ]
            },
            "end_state": {
              "must_observe": [
                "0 matches for '_as_fleet_owner' on routing surfaces",
                "every *_cmd resolves to #[but_api] fn",
                "planted bypass function triggers non-zero exit naming it"
              ],
              "must_not_observe": [
                "unattributed *_cmd symbol",
                "*_as_fleet_owner match on routing surface"
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
      "description": "GIVEN all tests land WHEN cargo test -p but-server && cargo clippy -p but-server --all-targets THEN exit 0",
      "verify": "cargo test -p but-server && cargo clippy -p but-server --all-targets",
      "scenario": {
        "tier": "build",
        "test_tier": "build",
        "verification_service": "cargo",
        "negative_control": {
          "would_fail_if": [
            "compilation error",
            "clippy warning treated as error",
            "test panic"
          ]
        },
        "evidence": {
          "artifact_type": "command_output",
          "required_capture": "exit code 0 for both commands"
        },
        "cases": [
          {
            "start_ref": "tests implemented",
            "action": {
              "actor": "developer",
              "steps": [
                "cargo test -p but-server",
                "cargo clippy -p but-server --all-targets"
              ]
            },
            "end_state": {
              "must_observe": [
                "cargo test exits 0",
                "cargo clippy exits 0"
              ],
              "must_not_observe": [
                "non-zero exit",
                "clippy deny-level warning"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "Every one of the 16 GOVERNANCE_COMMAND_ROUTES returns a non-404 status when invoked as admin.",
      "verify": "",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "Admin POST /branch_gates_update round-trips a gate edit and yields a pending signal.",
      "verify": "",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "Non-admin POST /branch_gates_update is denied with code='perm.denied' naming 'administration:write' and leaves the committed gates.toml blob unchanged.",
      "verify": "",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "No *_as_fleet_owner bypass symbol exists on the routing surfaces and every *_cmd in GOVERNANCE_COMMAND_ROUTES is positively attributed to a #[but_api] function.",
      "verify": "",
      "maps_to_ac": "AC-4"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "but-server tests and clippy pass cleanly.",
      "verify": "",
      "maps_to_ac": "AC-5"
    }
  ]
}
-->
