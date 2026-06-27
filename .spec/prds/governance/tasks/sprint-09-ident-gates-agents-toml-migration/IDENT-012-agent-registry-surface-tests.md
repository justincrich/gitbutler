# IDENT-012 — `crates/but-api/tests/agent_registry.rs` — register→action→unregister→denied end-to-end for each of the 4 gate surfaces (commit, merge, admin-write, forge review)

**Sprint:** [Sprint 09](./SPRINT.md) · **Agent:** `rust-implementer` · **Estimate:** 240 min · **Type:** FEATURE (TEST) · **Status:** READY · **Proposed By:** rust-planner

## Background

IDENT-010 proves each of the 8 callsite swaps with focused register→action→unregister→denied tests in `gate_registry_swap.rs`. This task authors the **durable consolidated end-to-end file** covering the 4 product surfaces (commit, merge, admin-write, forge review) — the regression net that travels forward through Sprint 10 (deprecation hardening) and beyond.

**Why it matters.** The 4-surface e2e is the canonical proof that the registry-resolved gates enforce identity at the product boundary. Sprint 10's policy flip (deny env-only) MUST keep these 4 surfaces behaving the same on the register/unregister path; this file is the regression net.

**Current state.** `crates/but-api/tests/agent_registry.rs` does not exist. IDENT-010 must land first (the callsites must be swapped before these flows can pass). Uses `but_testsupport::writable_scenario` + a real registry file seeded via `BUT_AGENT_REGISTRY_PATH` — no mocks.

**Desired state.** 4 `#[test]` fns, one per surface, each proving register→action-OK→unregister→action-denied against real git + a real registry file.

## Critical Constraints

- **MUST** write 4 `#[test]` fns: `commit_surface`, `merge_surface`, `admin_write_surface`, `forge_review_surface`.
- **MUST** seed a governed repo via `but_testsupport::writable_scenario` + `invoke_bash`; seed the runtime registry via `BUT_AGENT_REGISTRY_PATH` pointing at a per-test tempfile.
- **MUST** use the test process's OWN real `(pid, start_time)` via `but_authz::process::current_pid()` / `process_start_time(current_pid())` — so the registry hit is real, not a fake.
- **MUST** assert on the real `Denial` code (`perm.denied`), not on error `Display` text.
- **MUST** clean up `BUT_AGENT_REGISTRY_PATH` across tests (RAII guard or `std::env::remove_var` in `Drop`) — parallel test isolation.
- **MUST** print evidence lines via `println!` (mirrors existing `tests/config.rs` evidence-capture style).
- **NEVER** mock `Registry` or the gate functions — real file, real gates.
- **NEVER** use `std::env::temp_dir().join(format!(...))` — `but_testsupport::writable_scenario` only.
- **NEVER** set `BUT_AUTHZ_ALLOW_ENV_HANDLE` during the denial assertions (it would mask the denial).
- **NEVER** share a registry file across the 4 tests without isolation (parallel test races).
- **STRICTLY** cover exactly the 4 product surfaces the PRD names (commit, merge, admin-write, forge review) — not all 8 callsites (IDENT-010 covers the other 4 admin-read callsites).
- **STRICTLY** use the test process's OWN pid/start_time so the registry hit is real.

## Specification

**Objective:** Author `crates/but-api/tests/agent_registry.rs` with 4 end-to-end integration tests proving the registry-resolved gates enforce identity across the commit, merge, admin-write, and forge-review surfaces.

**Success state:** `cargo test -p but-api --test agent_registry` passes 4 tests. Each test demonstrates register→action-OK→unregister→action-denied against real git + a real registry file.

## Acceptance Criteria

**AC-1 (PRIMARY)** — commit surface end-to-end: GIVEN a governed repo + the test process registered as a contents:write principal WHEN `enforce_commit_gate_for_target` runs (registered) then again after unregister THEN registered → `Ok`; unregistered → `Err` `Denial` code `perm.denied`.
- **Verify:** `cargo test -p but-api --test agent_registry -- commit_surface`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-api · **FLOW_REF:** UC-IDENT-03
- **Scenario:** `start_ref=governed_repo_for_surface_test`; `must_observe` = [first `Ok`, second `Err` code `perm.denied`]; `must_not_observe` = [second `Ok`, mock registry artifacts]; `negative_control.would_fail_if` = [commit gate still uses env-only resolution, registry is mocked, unregister does not actually remove the entry].

**AC-2** — merge surface end-to-end: registered merger → `Ok`; unregister → `Err perm.denied`.
- **Verify:** `cargo test -p but-api --test agent_registry -- merge_surface`

**AC-3** — admin-write surface end-to-end: registered administration:write → `Ok`; unregister → `Err perm.denied`.
- **Verify:** `cargo test -p but-api --test agent_registry -- admin_write_surface`

**AC-4** — forge review surface end-to-end: registered reviews:write → `Ok(Some(principal))`; unregister → `Err perm.denied`.
- **Verify:** `cargo test -p but-api --test agent_registry -- forge_review_surface`

*Each AC's scenario follows the same shape as AC-1.*

## Test Criteria

| ID | Boolean | Maps to AC |
|----|---------|------------|
| TC-1 | commit surface: register→Ok→unregister→Err perm.denied is true | AC-1 |
| TC-2 | merge surface: register→Ok→unregister→Err perm.denied is true | AC-2 |
| TC-3 | admin-write surface: register→Ok→unregister→Err perm.denied is true | AC-3 |
| TC-4 | forge review surface: register→Ok(Some)→unregister→Err perm.denied is true | AC-4 |

## Reading List

1. `crates/but-authz/tests/config.rs:195-230` — `governed_repo()` helper + `invoke_bash` seed pattern — copy this shape for the per-surface fixtures.
2. `crates/but-api/src/commit/gate.rs:54-78` — `enforce_commit_gate_for_target` signature — the commit-surface entrypoint.
3. `crates/but-api/src/legacy/merge_gate.rs:40-56` — `enforce_merge_gate` — the merge-surface entrypoint.
4. `crates/but-api/src/legacy/config_mutate.rs:18-28` — `enforce_administration_write_gate(repo, ref)` — the admin-write entrypoint (takes `&gix::Repository` directly, easiest to test).
5. `crates/but-api/src/legacy/forge.rs:47-68` — `authorize_branch_action(repo, branch, authority)` — the forge-review entrypoint.

## Guardrails

**WRITE-ALLOWED:**
- `crates/but-api/tests/agent_registry.rs` (NEW — 4 surface tests + helper)

**WRITE-PROHIBITED:**
- `crates/but-api/tests/gate_registry_swap.rs` — IDENT-010 owns that file
- `crates/but-api/src/**` — IDENT-010 owns source; this task is test-only
- `crates/but-authz/**` — IDENT-009/015/016 own but-authz
- `crates/but/**` — IDENT-011/013/014 own the CLI

## Code Pattern

**Reference:** `crates/but-authz/tests/config.rs:195` (`governed_repo` helper — the seed pattern to copy).
**Source:** `crates/but-api/src/legacy/config_mutate.rs:18` (the simplest gate signature — repo + ref).

**Design notes:**
- Per-test isolation: each `#[test]` gets its own `writable_scenario` (own tmp repo) AND its own `BUT_AGENT_REGISTRY_PATH` tempfile, dropped at test end. Tests run in parallel by default — no shared state.
- For surfaces whose entrypoint takes a full `Context` (`merge_gate`), prefer the repo-direct variants where they exist (the `*_with_repo` family); if only the `Context` variant exists, build a minimal `but_ctx::Context` from the scenario repo (follow the pattern in existing `but-api` tests).
- Denial assertion: downcast the `anyhow::Error` to `but_authz::Denial` and assert `.code == but_authz::Denial::PERM_DENIED_CODE` — do not match `Display` text.

**Anti-pattern:** Do NOT mock the `Registry` or the gate. Do NOT assert on error `Display` strings. Do NOT leak `BUT_AGENT_REGISTRY_PATH` across tests.

## Agent Instructions

TDD RED→GREEN per surface (4 micro-cycles):
1. **RED AC-1:** Write `tests/agent_registry.rs::commit_surface` — seed repo + registry, call `enforce_commit_gate_for_target` → Ok; unregister; call again → expect Err with `Denial` code `perm.denied`. Run → fails (if IDENT-010 hasn't landed, the gate uses env-only and the unregister path doesn't deny).
2. **GREEN:** (assuming IDENT-010 has landed) the test should pass. If it doesn't, the surface-specific gate hasn't been swapped — file as IDENT-010 finding.
3. Repeat for AC-2 (merge), AC-3 (admin-write), AC-4 (forge review).
4. Add the `BUT_AGENT_REGISTRY_PATH` RAII guard helper.
5. Run `cargo fmt`, `cargo clippy -p but-api --tests -- -D warnings`, `cargo test -p but-api --test agent_registry`.
6. Commit via `but commit`.

## Orchestrator Verification Protocol

1. `cargo test -p but-api --test agent_registry` exit 0 (4 tests pass).
2. `cargo clippy -p but-api --tests -- -D warnings` clean.
3. The 4 product surfaces (commit, merge, admin-write, forge review) each have exactly one `#[test]` fn.

## Agent Assignment

**Agent:** `rust-implementer` — owns the consolidated `but-api` integration regression for the registry-resolved gates. NOTE FOR REVIEWER: this is the durable 4-surface end-to-end file, distinct from IDENT-010's per-callsite `gate_registry_swap.rs` (which is the RED→GREEN proof for the swap).
**Pairing:** none.

## Evidence Gates

- `cargo test -p but-api --test agent_registry` exit 0 (4 tests)
- Each test demonstrates register→Ok→unregister→denied
- `BUT_AGENT_REGISTRY_PATH` does not leak across tests

## Review Criteria

- All 4 surfaces use real git + real registry file (no mocks).
- Denial assertions downcast to `but_authz::Denial` and check `.code`, not `Display`.
- Per-test `BUT_AGENT_REGISTRY_PATH` isolation (RAII guard).
- The test process uses its OWN `(pid, start_time)` (`but_authz::process::current_pid()`).
- `println!` evidence lines mirror `tests/config.rs` style.

## Dependencies

- **Depends on:** IDENT-010 (callsites must be swapped before these flows pass).
- **Blocks:** none (regression net for Sprint 10/11).

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "IDENT-012",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "governed_repo_for_surface_test": {
      "description": "governed repo (agents.toml or permissions.toml + gates.toml committed at refs/heads/main) seeded per-surface with the principal + authority the surface needs (dev=contents:write for commit; merger=merge for merge; admin=administration:write for admin-write; reviewer=reviews:write for forge). Runtime registry seeded separately per-test via BUT_AGENT_REGISTRY_PATH.",
      "seed_method": "public_api",
      "records": [
        "writable_scenario + invoke_bash commits surface-appropriate governance",
        "Registry::write(BUT_AGENT_REGISTRY_PATH) seeds (current_pid(), process_start_time(current_pid())) → surface principal"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN governed repo + test process registered as contents:write WHEN enforce_commit_gate_for_target (registered) THEN Ok; after unregister THEN Err perm.denied",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-api --test agent_registry -- commit_surface",
      "maps_to_ac": null,
      "flow_ref": "UC-IDENT-03",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "governed_repo_for_surface_test",
        "must_observe": [
          "first Ok",
          "second Err code perm.denied"
        ],
        "must_not_observe": [
          "second Ok",
          "mock artifacts"
        ],
        "negative_control": {
          "would_fail_if": [
            "commit gate env-only",
            "registry mocked",
            "unregister no-op"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "governed_repo_for_surface_test",
            "action": {
              "actor": "test_harness",
              "steps": [
                "seed repo + registry(dev, contents:write)",
                "enforce_commit_gate_for_target Ok",
                "unregister",
                "enforce_commit_gate_for_target Err"
              ]
            },
            "end_state": {
              "must_observe": [
                "first result == `Ok(())`; second result error code == `perm.denied`"
              ],
              "must_not_observe": [
                "second result == `Ok(())`",
                "mock registry artifact present",
                "empty registry still authorizes"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "merge surface: register→Ok→unregister→Err perm.denied",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-api --test agent_registry -- merge_surface",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "governed_repo_for_surface_test",
        "must_observe": [
          "Ok then Err perm.denied"
        ],
        "must_not_observe": [
          "both Ok"
        ],
        "negative_control": {
          "would_fail_if": [
            "merge gate env-only",
            "registry mocked"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "governed_repo_for_surface_test",
            "action": {
              "actor": "test_harness",
              "steps": [
                "seed registry(merger, merge)",
                "enforce_merge_gate Ok",
                "unregister",
                "enforce_merge_gate Err"
              ]
            },
            "end_state": {
              "must_observe": [
                "first result == `Ok(())`; second result error code == `perm.denied`"
              ],
              "must_not_observe": [
                "both results == `Ok(())`",
                "mock registry artifact present",
                "empty registry still authorizes"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "admin-write surface: register→Ok→unregister→Err perm.denied",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-api --test agent_registry -- admin_write_surface",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "governed_repo_for_surface_test",
        "must_observe": [
          "Ok then Err perm.denied"
        ],
        "must_not_observe": [
          "both Ok"
        ],
        "negative_control": {
          "would_fail_if": [
            "admin-write gate env-only",
            "registry mocked"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "governed_repo_for_surface_test",
            "action": {
              "actor": "test_harness",
              "steps": [
                "seed registry(admin, administration:write)",
                "enforce_administration_write_gate Ok",
                "unregister",
                "enforce_administration_write_gate Err"
              ]
            },
            "end_state": {
              "must_observe": [
                "first result == `Ok(())`; second result error code == `perm.denied`"
              ],
              "must_not_observe": [
                "both results == `Ok(())`",
                "mock registry artifact present",
                "empty registry still authorizes"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "description": "forge review surface: register→Ok(Some)→unregister→Err perm.denied",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-api --test agent_registry -- forge_review_surface",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "governed_repo_for_surface_test",
        "must_observe": [
          "Ok(Some(principal)) then Err perm.denied"
        ],
        "must_not_observe": [
          "both Ok(Some)"
        ],
        "negative_control": {
          "would_fail_if": [
            "forge gate env-only",
            "registry mocked"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "governed_repo_for_surface_test",
            "action": {
              "actor": "test_harness",
              "steps": [
                "seed registry(reviewer, reviews:write)",
                "authorize_branch_action ReviewsWrite Ok(Some)",
                "unregister",
                "authorize_branch_action Err"
              ]
            },
            "end_state": {
              "must_observe": [
                "first result == `Ok(Some(_))`; second result error code == `perm.denied`"
              ],
              "must_not_observe": [
                "both results == `Ok(Some(_))`",
                "mock registry artifact present",
                "empty registry still authorizes"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "commit surface e2e",
      "verify": "cargo test -p but-api --test agent_registry -- commit_surface",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "merge surface e2e",
      "verify": "cargo test -p but-api --test agent_registry -- merge_surface",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "admin-write surface e2e",
      "verify": "cargo test -p but-api --test agent_registry -- admin_write_surface",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "forge review surface e2e",
      "verify": "cargo test -p but-api --test agent_registry -- forge_review_surface",
      "maps_to_ac": "AC-4"
    }
  ]
}
-->
