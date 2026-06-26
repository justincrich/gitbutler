# IDENT-010 — `but-api` — swap the 8 gate callsites from `resolve_principal_from_env` to `resolve_principal_with_registry`

**Sprint:** [Sprint 09](./SPRINT.md) · **Agent:** `rust-implementer` · **Estimate:** 180 min · **Type:** FEATURE · **Status:** READY · **Proposed By:** rust-planner

## Background

The 8 gate callsites in `but-api` all currently call `but_authz::resolve_principal_from_env(&cfg)` — the self-asserted `BUT_AGENT_HANDLE` string Sprint 08 left as the only identity source. Sprint 08 also shipped `resolve_principal_with_registry` (IDENT-003), `Registry` (IDENT-001), and `Denial::unregistered`/`stale_registration`. This task swaps all 8 callsites to the registry resolver.

**Why it matters.** This is the first sprint where the registry actually governs a commit/merge/admin/forge action. Policy in Sprint 09 is migration-friendly: registry hit → principal; registry miss + `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` → env fallback (test/CI escape hatch); else → `Denial::unregistered(pid)`. Sprint 10 flips the default to deny env-only.

**Current state.** All 8 callsites at known locations (verified against live code; PRD's line numbers are slightly stale):
- `crates/but-api/src/commit/gate.rs:65`
- `crates/but-api/src/legacy/merge_gate.rs:47`
- `crates/but-api/src/legacy/governance.rs:347, 378, 448, 750` (4 sites)
- `crates/but-api/src/legacy/forge.rs:58`
- `crates/but-api/src/legacy/config_mutate.rs:23`

**Desired state.** All 8 call `resolve_principal_with_registry(registry, &cfg)`; uniform resolution policy across all surfaces; `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` fallback still works.

## Critical Constraints

- **MUST** swap every one of the 8 callsites (see Background for the verified file:line list).
- **MUST** load the registry via the Sprint-08 path resolver (`BUT_AGENT_REGISTRY_PATH` → XDG → absence); a missing/unreadable registry file is `Registry::empty()` (not an error) so the gate falls through to env/denial.
- **MUST** keep the `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` env-fallback path functional.
- **MUST** add a grep-asserted build gate (or test) that no `but-api` file references `BUT_AGENT_HANDLE` directly except through `authorize.rs`.
- **NEVER** flip the default to deny env-only (Sprint 10 owns that).
- **NEVER** remove `resolve_principal_from_env` from `but-authz` (still used by the fallback path + Sprint-08 tests).
- **NEVER** add a per-callsite registry cache or mutex — load once per invocation.
- **NEVER** read `BUT_AGENT_HANDLE` directly inside `but-api` (only `authorize.rs` owns that env read).
- **STRICTLY** keep the 8 swaps behavior-neutral for the env-fallback case: a caller that worked under Sprint 08's `resolve_principal_from_env` (with `BUT_AUTHZ_ALLOW_ENV_HANDLE=1`) MUST still work after the swap.
- **STRICTLY** preserve each callsite's existing error mapping (`Denial → anyhow` via `.into()`).
- **NEVER** use `std::env::temp_dir().join(format!(...))` — `but_testsupport::writable_scenario`.

## Specification

**Objective:** Replace `resolve_principal_from_env(&cfg)` with `resolve_principal_with_registry(registry, &cfg)` at all 8 `but-api` gate callsites, and add one integration test per callsite proving register→action-OK→unregister→action-denied.

**Success state:** `cargo test -p but-api --test gate_registry_swap` passes 8 tests (one per callsite) + 1 env-fallback regression test. `cargo check -p but-api` clean. `rg -c 'BUT_AGENT_HANDLE' crates/but-api/src` returns 0.

## Acceptance Criteria

**AC-1 (PRIMARY)** — commit/gate.rs callsite swapped: GIVEN a governed repo where the test process is registered as dev (contents:write) WHEN `enforce_commit_gate_for_target` runs for a non-protected feature branch THEN `Ok(())`; after `Registry::unregister`, the same call returns `Err` whose downcast `Denial` code == `perm.denied`.
- **Verify:** `cargo test -p but-api --test gate_registry_swap -- commit_gate`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-api · **FLOW_REF:** UC-IDENT-03
- **Scenario:** `start_ref=governed_repo_registered_dev`; `must_observe` = [first call `Ok(())`, second call `Err` with `Denial` code `perm.denied`]; `must_not_observe` = [second call `Ok(())`, first call `Err`].

**AC-2** — legacy/merge_gate.rs callsite swapped: registered merger → `Ok`; unregister → `Err perm.denied`.
- **Verify:** `cargo test -p but-api --test gate_registry_swap -- merge_gate`

**AC-3** — legacy/governance.rs:347 (governance_status_read) callsite swapped.
- **Verify:** `cargo test -p but-api --test gate_registry_swap -- governance_status_read`

**AC-4** — legacy/governance.rs:378 (branch_gates_read_with_repo) callsite swapped.
- **Verify:** `cargo test -p but-api --test gate_registry_swap -- branch_gates_read`

**AC-5** — legacy/governance.rs:448 (group_list_with_repo) callsite swapped.
- **Verify:** `cargo test -p but-api --test gate_registry_swap -- group_list`

**AC-6** — legacy/governance.rs:750 (perm_list_with_repo) callsite swapped.
- **Verify:** `cargo test -p but-api --test gate_registry_swap -- perm_list`

**AC-7** — legacy/forge.rs:58 (authorize_branch_action) callsite swapped: registered reviews:write → `Ok(Some(principal))`; unregister → `Err perm.denied`.
- **Verify:** `cargo test -p but-api --test gate_registry_swap -- forge_review`

**AC-8** — legacy/config_mutate.rs:23 (enforce_administration_write_gate) callsite swapped: registered administration:write → `Ok`; unregister → `Err perm.denied`.
- **Verify:** `cargo test -p but-api --test gate_registry_swap -- admin_write_gate`

**AC-9** — env-fallback still works (policy NOT flipped): GIVEN empty registry + `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` + `BUT_AGENT_HANDLE=dev` WHEN `enforce_commit_gate_for_target` THEN `Ok(())` via env fallback (proves Sprint 09 did NOT flip the deny-default).
- **Verify:** `cargo test -p but-api --test gate_registry_swap -- env_fallback_still_allowed`

*Each AC's scenario follows the same shape as AC-1: `start_ref=governed_repo_registered_dev` (or `_unregistered` for AC-9); concrete `must_observe` naming the `Ok`/`Err` transition + the `perm.denied` Denial code; `must_not_observe` naming the inverse; `negative_control.would_fail_if` naming the specific callsite line still calling `resolve_principal_from_env`.*

## Test Criteria

| ID | Boolean | Maps to AC |
|----|---------|------------|
| TC-1 | commit gate: registered→Ok, unregister→Err perm.denied is true | AC-1 |
| TC-2 | merge gate: registered→Ok, unregister→Err perm.denied is true | AC-2 |
| TC-3 | governance_status_read callsite swapped is true | AC-3 |
| TC-4 | branch_gates_read callsite swapped is true | AC-4 |
| TC-5 | group_list callsite swapped is true | AC-5 |
| TC-6 | perm_list callsite swapped is true | AC-6 |
| TC-7 | forge review callsite swapped is true | AC-7 |
| TC-8 | admin-write gate callsite swapped is true | AC-8 |
| TC-9 | env fallback (`BUT_AUTHZ_ALLOW_ENV_HANDLE=1`) still succeeds on registry miss is true | AC-9 |
| TC-10 | `rg -c 'BUT_AGENT_HANDLE' crates/but-api/src == 0` is true | AC-1 |

## Reading List

1. `crates/but-api/src/commit/gate.rs:54-78` — `enforce_commit_gate_for_target` — line 65 swap; preserve `governance_present` short-circuit and protected-branch check.
2. `crates/but-api/src/legacy/merge_gate.rs:40-64` — `enforce_merge_gate` — line 47 swap; preserve Merge authority + review-requirement logic.
3. `crates/but-api/src/legacy/governance.rs:340-384` — `governance_status_read` (347) + `branch_gates_read_with_repo` (378).
4. `crates/but-api/src/legacy/governance.rs:443-460` — `group_list_with_repo` (448).
5. `crates/but-api/src/legacy/governance.rs:744-762` — `perm_list_with_repo` (750).
6. `crates/but-api/src/legacy/forge.rs:47-68` — `authorize_branch_action` (58).
7. `crates/but-api/src/legacy/config_mutate.rs:18-28` — `enforce_administration_write_gate` (23).
8. `crates/but-authz/src/authorize.rs:60-102` — `resolve_principal` / `resolve_principal_from_env` shape; Sprint 08's `resolve_principal_with_registry` mirrors this `Result<Principal, Denial>` discipline.

## Guardrails

**WRITE-ALLOWED:**
- `crates/but-api/src/commit/gate.rs` (MODIFY line 65 swap)
- `crates/but-api/src/legacy/merge_gate.rs` (MODIFY line 47 swap)
- `crates/but-api/src/legacy/governance.rs` (MODIFY lines 347, 378, 448, 750 swaps — resolver line only)
- `crates/but-api/src/legacy/forge.rs` (MODIFY line 58 swap + import)
- `crates/but-api/src/legacy/config_mutate.rs` (MODIFY line 23 swap)
- `crates/but-api/tests/gate_registry_swap.rs` (NEW — 8 focused callsite tests + 1 env-fallback regression)

**WRITE-PROHIBITED:**
- `crates/but-api/tests/agent_registry.rs` — IDENT-012 owns this consolidated file
- `crates/but-authz/**` — IDENT-009/016 own but-authz; the resolver + `Registry` come from Sprint 08
- `crates/but/**` — IDENT-011/014 own the CLI

## Code Pattern

**Reference:** `crates/but-authz/src/authorize.rs:67-89` (`resolve_principal` `Result<Principal,Denial>` shape — the registry resolver follows the same discipline).
**Source:** `crates/but-api/src/commit/gate.rs:60-67` (the callsite pattern: `governance_present → load_governance_config → resolve → authorize`).

**Design notes:**
- Each swap replaces `let principal = but_authz::resolve_principal_from_env(&cfg)?;` with `let registry = but_authz::Registry::load_default()?.unwrap_or_default(); let principal = but_authz::resolve_principal_with_registry(&registry, &cfg)?;` — exact Sprint-08 API names per the landed signature.
- Registry load must be tolerant: missing file → empty registry, NOT an error.
- The `gate_registry_swap.rs` test seeds the registry via `BUT_AGENT_REGISTRY_PATH` pointing at a tempfile, writes the test process's real `(pid, start_time)`, then calls each gate function directly (the `_with_repo` / `_for_target` variants taking `&gix::Repository`, avoiding the full `Context`).

**Anti-pattern:** Do NOT add a registry cache, lazy-static, or once_cell in `but-api` (load once per invocation). Do NOT flip the env-fallback default. Do NOT touch the protected-branch or review-requirement logic downstream of the resolver.

## Agent Instructions

TDD RED→GREEN per callsite (8 micro-cycles) + 1 for env-fallback:
1. **RED AC-1:** Write `tests/gate_registry_swap.rs::commit_gate` — seed repo + registry, call `enforce_commit_gate_for_target` → Ok; unregister; call again → expect Err with `Denial` code `perm.denied`. Run → fails (callsite still uses env resolver).
2. **GREEN:** Swap line 65 in `commit/gate.rs`. Re-run → passes.
3. Repeat for AC-2 through AC-8 (one micro-cycle per callsite).
4. AC-9: env-fallback regression test — set `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` + `BUT_AGENT_HANDLE=dev` + empty registry → `enforce_commit_gate_for_target` → Ok.
5. Add the grep build-gate test (TC-10).
6. Run `cargo fmt`, `cargo clippy -p but-api --all-targets -- -D warnings`, `cargo test -p but-api --test gate_registry_swap`.
7. Commit via `but commit`.

## Orchestrator Verification Protocol

1. `cargo test -p but-api --test gate_registry_swap` exit 0 (9 tests pass).
2. `cargo check -p but-api --all-targets` clean.
3. `rg -c 'BUT_AGENT_HANDLE' crates/but-api/src` returns 0.
4. All 8 source files modified at exactly the documented lines (resolver line + import).

## Agent Assignment

**Agent:** `rust-implementer` — owns the `but-api` gate layer. NOTE FOR REVIEWER: `resolve_principal_with_registry`, `Registry`, `Denial::unregistered`/`stale_registration`, and the `BUT_AUTHZ_ALLOW_ENV_HANDLE` flag are Sprint 08 deliverables (IDENT-003/001/002) — this task CONSUMES them and must run after Sprint 08 lands.
**Pairing:** none. Each callsite swap is self-contained.

## Evidence Gates

- `cargo test -p but-api --test gate_registry_swap` exit 0 (9 tests)
- `rg -c 'BUT_AGENT_HANDLE' crates/but-api/src == 0`
- All 8 source files modified at the documented lines

## Review Criteria

- Each of the 8 callsites uses `resolve_principal_with_registry` with a freshly-loaded registry (no cache).
- Registry load is tolerant (missing file → empty, not error).
- `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` fallback path preserved (AC-9 green).
- No drive-by changes to protected-branch / review-requirement logic downstream of the resolver.
- Denial assertions downcast to `but_authz::Denial` and check `.code`, not `Display` text.

## Dependencies

- **Depends on:** IDENT-009 (callsites need the prefer-agents loader for agents.toml-aware governance).
- **Blocks:** IDENT-012 (consolidated end-to-end test consumes the swapped callsites).

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "IDENT-010",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "governed_repo_registered_dev": {
      "description": "governed repo + runtime registry with (test_pid, test_start_time) → dev holding the union authority set",
      "seed_method": "public_api",
      "records": [
        "writable_scenario + invoke_bash commits governance",
        "Registry::write(BUT_AGENT_REGISTRY_PATH) seeds the dev entry"
      ]
    },
    "governed_repo_unregistered": {
      "description": "governed repo + empty registry + BUT_AUTHZ_ALLOW_ENV_HANDLE unset",
      "seed_method": "public_api",
      "records": [
        "governed repo + empty/absent registry"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN governed repo + test process registered as dev WHEN enforce_commit_gate_for_target THEN Ok; after unregister THEN Err perm.denied",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-api --test gate_registry_swap -- commit_gate",
      "maps_to_ac": null,
      "flow_ref": "UC-IDENT-03",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "governed_repo_registered_dev",
        "must_observe": [
          "first Ok",
          "second Err code perm.denied"
        ],
        "must_not_observe": [
          "second Ok",
          "first Err"
        ],
        "negative_control": {
          "would_fail_if": [
            "commit gate still uses disconnected env resolver",
            "registry loading omitted at commit gate callsite"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "governed_repo_registered_dev",
            "action": {
              "actor": "test_harness",
              "steps": [
                "enforce_commit_gate_for_target → Ok",
                "Registry::unregister",
                "enforce_commit_gate_for_target → Err"
              ]
            },
            "end_state": {
              "must_observe": [
                "first result == `Ok(())`; second result error code == `perm.denied`"
              ],
              "must_not_observe": [
                "second result == `Ok(())`",
                "first result is `Err`",
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
      "description": "merge_gate.rs:47 swapped: registered→Ok, unregister→Err perm.denied",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-api --test gate_registry_swap -- merge_gate",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "governed_repo_registered_dev",
        "must_observe": [
          "Ok then Err perm.denied"
        ],
        "must_not_observe": [
          "both Ok"
        ],
        "negative_control": {
          "would_fail_if": [
            "merge gate still uses disconnected env resolver",
            "registry lookup omitted for merge gate"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "governed_repo_registered_dev",
            "action": {
              "actor": "test_harness",
              "steps": [
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
      "description": "governance.rs:347 swapped: registered→Ok(authorities), unregister→Err perm.denied",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-api --test gate_registry_swap -- governance_status_read",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "governed_repo_registered_dev",
        "must_observe": [
          "Ok(AuthoritySet with ContentsWrite) then Err perm.denied"
        ],
        "must_not_observe": [
          "both Ok"
        ],
        "negative_control": {
          "would_fail_if": [
            "governance status still uses disconnected env resolver",
            "registry lookup omitted for governance status"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "governed_repo_registered_dev",
            "action": {
              "actor": "test_harness",
              "steps": [
                "governance_status_read Ok",
                "unregister",
                "governance_status_read Err"
              ]
            },
            "end_state": {
              "must_observe": [
                "first result contains `Authority::ContentsWrite`; second result error code == `perm.denied`"
              ],
              "must_not_observe": [
                "both results == `Ok`",
                "empty registry still returns authorities"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "description": "governance.rs:378 swapped: registered→Ok, unregister→Err perm.denied",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-api --test gate_registry_swap -- branch_gates_read",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "governed_repo_registered_dev",
        "must_observe": [
          "Ok then Err perm.denied"
        ],
        "must_not_observe": [
          "both Ok"
        ],
        "negative_control": {
          "would_fail_if": [
            "branch gates read still uses disconnected env resolver",
            "registry lookup omitted for branch gates"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "governed_repo_registered_dev",
            "action": {
              "actor": "test_harness",
              "steps": [
                "branch_gates_read_with_repo Ok",
                "unregister",
                "branch_gates_read_with_repo Err"
              ]
            },
            "end_state": {
              "must_observe": [
                "first result == `Ok(())`; second result error code == `perm.denied`"
              ],
              "must_not_observe": [
                "both results == `Ok(())`",
                "empty registry still authorizes"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "description": "governance.rs:448 swapped: registered→Ok, unregister→Err perm.denied",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-api --test gate_registry_swap -- group_list",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "governed_repo_registered_dev",
        "must_observe": [
          "Ok then Err perm.denied"
        ],
        "must_not_observe": [
          "both Ok"
        ],
        "negative_control": {
          "would_fail_if": [
            "group list still uses disconnected env resolver",
            "registry lookup omitted for group list"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "governed_repo_registered_dev",
            "action": {
              "actor": "test_harness",
              "steps": [
                "group_list_with_repo Ok",
                "unregister",
                "group_list_with_repo Err"
              ]
            },
            "end_state": {
              "must_observe": [
                "first result == `Ok(())`; second result error code == `perm.denied`"
              ],
              "must_not_observe": [
                "both results == `Ok(())`",
                "empty registry still authorizes"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-6",
      "type": "acceptance_criterion",
      "description": "governance.rs:750 swapped: registered→Ok naming dev, unregister→Err perm.denied",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-api --test gate_registry_swap -- perm_list",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "governed_repo_registered_dev",
        "must_observe": [
          "Ok principal=dev then Err perm.denied"
        ],
        "must_not_observe": [
          "both Ok"
        ],
        "negative_control": {
          "would_fail_if": [
            "status principal read still uses disconnected env resolver",
            "registry lookup omitted for status principal"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "governed_repo_registered_dev",
            "action": {
              "actor": "test_harness",
              "steps": [
                "perm_list_with_repo Ok principal=dev",
                "unregister",
                "perm_list_with_repo Err"
              ]
            },
            "end_state": {
              "must_observe": [
                "first result principal == `dev`; second result error code == `perm.denied`"
              ],
              "must_not_observe": [
                "both results name `dev`",
                "empty registry still returns a principal"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-7",
      "type": "acceptance_criterion",
      "description": "forge.rs:58 swapped: registered reviews:write→Ok(Some), unregister→Err perm.denied",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-api --test gate_registry_swap -- forge_review",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "governed_repo_registered_dev",
        "must_observe": [
          "Ok(Some(principal)) then Err perm.denied"
        ],
        "must_not_observe": [
          "both Ok(Some)"
        ],
        "negative_control": {
          "would_fail_if": [
            "forge review gate still uses disconnected env resolver",
            "registry lookup omitted for forge review gate"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "governed_repo_registered_dev",
            "action": {
              "actor": "test_harness",
              "steps": [
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
                "empty registry still authorizes"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-8",
      "type": "acceptance_criterion",
      "description": "config_mutate.rs:23 swapped: registered admin:write→Ok, unregister→Err perm.denied",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-api --test gate_registry_swap -- admin_write_gate",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "governed_repo_registered_dev",
        "must_observe": [
          "Ok then Err perm.denied"
        ],
        "must_not_observe": [
          "both Ok"
        ],
        "negative_control": {
          "would_fail_if": [
            "config mutation still uses disconnected env resolver",
            "registry lookup omitted for config mutation"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "governed_repo_registered_dev",
            "action": {
              "actor": "test_harness",
              "steps": [
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
                "empty registry still authorizes"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-9",
      "type": "acceptance_criterion",
      "description": "GIVEN empty registry + BUT_AUTHZ_ALLOW_ENV_HANDLE=1 + BUT_AGENT_HANDLE=dev WHEN commit gate THEN Ok (env fallback not flipped)",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-api --test gate_registry_swap -- env_fallback_still_allowed",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "governed_repo_unregistered",
        "must_observe": [
          "Ok(()) via env fallback"
        ],
        "must_not_observe": [
          "Err perm.denied"
        ],
        "negative_control": {
          "would_fail_if": [
            "swap hardcoded deny on miss",
            "flag check removed from resolver"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "governed_repo_unregistered",
            "action": {
              "actor": "test_harness",
              "steps": [
                "set BUT_AUTHZ_ALLOW_ENV_HANDLE=1 + BUT_AGENT_HANDLE=dev",
                "enforce_commit_gate_for_target"
              ]
            },
            "end_state": {
              "must_observe": [
                "env fallback result == `Ok(())` when `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` and `BUT_AGENT_HANDLE=dev`"
              ],
              "must_not_observe": [
                "error code == `perm.denied`",
                "empty env fallback authorizes with flag unset"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "commit gate register→Ok→unregister→denied",
      "verify": "cargo test -p but-api --test gate_registry_swap -- commit_gate",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "merge gate register→Ok→unregister→denied",
      "verify": "cargo test -p but-api --test gate_registry_swap -- merge_gate",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "governance_status_read swapped",
      "verify": "cargo test -p but-api --test gate_registry_swap -- governance_status_read",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "branch_gates_read swapped",
      "verify": "cargo test -p but-api --test gate_registry_swap -- branch_gates_read",
      "maps_to_ac": "AC-4"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "group_list swapped",
      "verify": "cargo test -p but-api --test gate_registry_swap -- group_list",
      "maps_to_ac": "AC-5"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "description": "perm_list swapped",
      "verify": "cargo test -p but-api --test gate_registry_swap -- perm_list",
      "maps_to_ac": "AC-6"
    },
    {
      "id": "TC-7",
      "type": "test_criterion",
      "description": "forge review swapped",
      "verify": "cargo test -p but-api --test gate_registry_swap -- forge_review",
      "maps_to_ac": "AC-7"
    },
    {
      "id": "TC-8",
      "type": "test_criterion",
      "description": "admin-write gate swapped",
      "verify": "cargo test -p but-api --test gate_registry_swap -- admin_write_gate",
      "maps_to_ac": "AC-8"
    },
    {
      "id": "TC-9",
      "type": "test_criterion",
      "description": "env fallback still allowed",
      "verify": "cargo test -p but-api --test gate_registry_swap -- env_fallback_still_allowed",
      "maps_to_ac": "AC-9"
    },
    {
      "id": "TC-10",
      "type": "test_criterion",
      "description": "no BUT_AGENT_HANDLE refs in but-api/src",
      "verify": "rg -c 'BUT_AGENT_HANDLE' crates/but-api/src == 0",
      "maps_to_ac": "AC-1"
    }
  ]
}
-->
