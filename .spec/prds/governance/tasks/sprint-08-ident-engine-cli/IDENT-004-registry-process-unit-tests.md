# IDENT-004 — `crates/but-authz/tests/registry.rs` + `tests/process.rs` + extend `tests/authorize.rs`

**Sprint:** [Sprint 08](./SPRINT.md) · **Agent:** `rust-implementer` · **Estimate:** 180 min · **Type:** FEATURE · **Status:** READY · **Proposed By:** rust-planner (`--no-specialists`)

## Background

IDENT-001/002/003 each shipped minimal RED→GREEN tests for their own surface. This task consolidates the **full** unit + integration suite for the registry, process, and resolver per the PRD's T-IDENT-009..015 + T-IDENT-016..022 criteria. It's the honesty gate: the registry's invariants (atomic writes, TTL boundaries, PID-reuse defense, concurrent writes) and the resolver's policy (registry → flag-gated env → denial) need real adversarial cases, not just happy-path round-trips.

**Why it matters.** Without this suite, Sprint 09's wiring of the 8 gate callsites has no safety net. A regression in `Registry::resolve` (e.g., matching on pid alone) would silently leak authority across PID reuse. The tests here are the substrate for the `invariant_build_gates` extension in Sprint 10.

**Current state.** `crates/but-authz/tests/` has `authorize.rs`, `authority.rs`, `config.rs`, `grps_ref_pin.rs`, `grps_union.rs`, `invariant_build_gates.rs`. No `registry.rs` or `process.rs` yet. The existing tests use `but_testsupport::writable_scenario` + `temp_env::with_var`.

**Desired state.** `tests/registry.rs` (register/unregister/TTL/PID-reuse/concurrent-writes) + `tests/process.rs` (current_pid/start_time monotonic/error paths) + `tests/authorize.rs` extended with the 5 ACs from IDENT-003.

## Critical Constraints

- **MUST** use `but_testsupport::writable_scenario` for filesystem fixtures (per `crates/AGENTS.md` "Rust tests" — never `std::env::temp_dir().join(...)`).
- **MUST** use `temp_env::with_var(...)` under `#[serial_test::serial]` for any test that sets `BUT_AGENT_HANDLE` or `BUT_AUTHZ_ALLOW_ENV_HANDLE` (per existing pattern at `crates/but-api/tests/commit_gate.rs:14`).
- **NEVER** mock `Registry` or `process_start_time` — exercise the real implementations. The injected-lookup variants (`resolve_principal_with_registry_and_lookup`) exist for testability of the env-flag path, not for replacing the registry.
- **STRICTLY** test TTL boundaries with explicit `gc(now)` calls — do NOT rely on `std::thread::sleep` (flaky, slow). Inject the clock via the registry's `gc(now: u64)` parameter.
- **MUST** cover concurrent writes: spawn 2+ threads writing distinct `(pid, start_time)` entries to the SAME path, assert all entries are present after both finish, AND the on-disk file always parses.

## Specification

**Objective:** Land the full IDENT-008..022 unit + integration test suite in `crates/but-authz/tests/`.

**Success state:** `cargo test -p but-authz` green. The new `tests/registry.rs` covers T-IDENT-009..015. The new `tests/process.rs` covers the process module ACs. `tests/authorize.rs` is extended with IDENT-003's AC-1..5.

## Acceptance Criteria

**AC-1 (registry round-trip)** — GIVEN a populated `Registry` with 3 distinct entries WHEN `write(path)` then `load(path)` THEN the loaded registry is `==` the original AND every entry's `(pid, start_time, agent_id, expires_at, registered_by)` round-trips byte-exact.

**AC-2 (TTL expiry via injected clock)** — GIVEN an entry with `expires_at = 1000` WHEN `gc(999)` is called THEN the entry is still resolvable; WHEN `gc(1000)` is called THEN it is still resolvable (boundary: `expires_at` is the last valid second); WHEN `gc(1001)` is called THEN it is no longer resolvable.

**AC-3 (PID-reuse rejection)** — GIVEN an entry `(1234, 100, "agent-a")` WHEN `resolve((1234, 200))` is called THEN it returns `None` AND the original entry is unchanged (still resolvable at `(1234, 100)`).

**AC-4 (concurrent writes)** — GIVEN 4 threads each calling `register((pid_i, start_i), agent_i, …)` then `write(path)` on the SAME path WHEN all 4 finish THEN the loaded registry contains all 4 entries AND the file at `path` parses without error (atomic-rename guarantees no half-write is observable).

**AC-5 (process monotonic)** — GIVEN the test process WHEN `process_start_time(current_pid())` is called twice in succession THEN both calls return `Ok(t)` with equal `t` AND `t > 1_000_000_000` (post-Y2020 sanity).

**AC-6 (resolver extension)** — GIVEN the 5 scenarios from IDENT-003 AC-1..5 WHEN each is exercised via `temp_env::with_var` + an in-memory `Registry` THEN the outcomes match IDENT-003's expected `Ok`/`Err(Denial::*)` per case.

## Test Criteria

| ID | Boolean | Maps to AC |
|----|---------|------------|
| TC-1 | `tests/registry.rs::registry_round_trip` passes (3 entries, byte-exact) | AC-1 |
| TC-2 | `tests/registry.rs::ttl_boundary` passes (gc(999)/gc(1000) keep; gc(1001) drops) | AC-2 |
| TC-3 | `tests/registry.rs::pid_reuse_rejection` passes (same pid, different start → None) | AC-3 |
| TC-4 | `tests/registry.rs::concurrent_writes` passes (4 threads, all entries land, file parses) | AC-4 |
| TC-5 | `tests/process.rs::start_time_monotonic` passes | AC-5 |
| TC-6 | `tests/authorize.rs::resolve_principal_with_registry_*` (5 sub-tests for IDENT-003 ACs) pass | AC-6 |

## Reading List

- `crates/but-authz/tests/authorize.rs:1-50` — the existing `assert_no_principal_denied` helper + `governed_repo` fixture pattern
- `crates/but-api/tests/commit_gate.rs:14` — the `temp_env::with_var("BUT_AGENT_HANDLE", …)` + `#[serial_test::serial]` pattern
- `crates/but-testsupport/src/sandbox.rs:394-440` — `invoke_git` / `invoke_bash` for fixture setup
- `crates/but-authz/src/registry.rs` (IDENT-001) — the API under test
- `crates/but-authz/src/process.rs` (IDENT-002) — the API under test
- `crates/but-authz/src/authorize.rs` (IDENT-003) — `resolve_principal_with_registry_and_lookup` (the testable variant)

## Guardrails

**WRITE-ALLOWED:**
- `crates/but-authz/tests/registry.rs` (NEW)
- `crates/but-authz/tests/process.rs` (NEW)
- `crates/but-authz/tests/authorize.rs` (extend — add the 5 IDENT-003 AC tests at the end)

**WRITE-PROHIBITED:**
- `crates/but-authz/src/**` (production code — IDENT-001/002/003 own it; this task only tests)
- `crates/but-authz/Cargo.toml` `[dev-dependencies]` — only edit if `temp_env` / `serial_test` are missing (check first; both are workspace deps)

## Code Pattern

**Reference:** `crates/but-authz/tests/authorize.rs:41-67` (`resolve_no_handle_rejected`) — the `assert_no_principal_denied` shape and the `temp_env::with_var` discipline. Mirror for the new denial variants.

**Source (registry concurrent-writes sketch):**
```rust
#[test]
fn concurrent_writes_to_same_path_all_entries_land() -> anyhow::Result<()> {
    let (repo, _tmp) = but_testsupport::writable_scenario("ident-concurrent");
    let path = repo.path().join("agents-runtime.toml");
    let handles: Vec<_> = (1..=4).map(|i| {
        let path = path.clone();
        std::thread::spawn(move || -> anyhow::Result<()> {
            let mut reg = but_authz::Registry::load(&path)?;
            reg.register(1000 + i, 1730000000 + i, &format!("agent-{i}"), 14400, "test")?;
            reg.write(&path)?;
            Ok(())
        })
    }).collect();
    for h in handles { h.join().unwrap()?; }
    let loaded = but_authz::Registry::load(&path)?;
    for i in 1..=4 {
        assert_eq!(loaded.resolve(&(1000 + i, 1730000000 + i)), Some(format!("agent-{i}")));
    }
    Ok(())
}
```

**Anti-pattern:** do NOT use `tokio::test` + async for the concurrent-writes test — `Registry` is a sync API; `std::thread` is the right concurrency primitive here.

## Agent Instructions

1. Read IDENT-001/002/003 task files to understand the exact API under test.
2. **RED:** Create `tests/registry.rs` with the 4 ACs above (round-trip, TTL, PID-reuse, concurrent). Create `tests/process.rs` with the monotonic test. Extend `tests/authorize.rs` with the 5 IDENT-003 AC tests. Run `cargo test -p but-authz` → new tests fail (RED).
3. **GREEN:** The production code from IDENT-001/002/003 should already make them pass; if not, surface the gap and fix the production code OR the test (whichever is wrong per the AC).
4. **REFACTOR:** Pull repeated fixture builders into helpers (`fn empty_registry()`, `fn seeded_registry_one_entry()`).
5. Run `cargo test -p but-authz` then `cargo test -p but-authz --test invariant_build_gates` (should still pass). Commit via `but commit`.

## Orchestrator Verification Protocol

1. `cargo test -p but-authz` exit 0.
2. `tests/registry.rs`, `tests/process.rs` exist; `tests/authorize.rs` has the 5 new IDENT-003 tests.
3. Test count: at least 6 new test fns matching the ACs.

## Agent Assignment

**Agent:** `rust-implementer` — owns `crates/but-authz` tests. The test work is straightforward; no review-specific adversarial reasoning needed (IDENT-007 is the CLI reviewer task).

**Pairing:** depends on IDENT-001/002/003 production code landing first.

## Evidence Gates

- `cargo test -p but-authz` exit 0
- 4 new tests in `registry.rs`, 1 in `process.rs`, 5 new in `authorize.rs`

## Review Criteria

- TTL boundary test covers BOTH sides (`gc(expires_at)` keep, `gc(expires_at+1)` drop).
- Concurrent-writes test uses `std::thread`, not async.
- No `std::env::temp_dir()` anywhere.
- All env-var mutations are `temp_env::with_var` under `#[serial_test::serial]`.

## Dependencies

- **depends_on:** IDENT-001, IDENT-002, IDENT-003.
- **blocks:** Sprint 09 integration tests (which consume these registry/process patterns).

## Notes

- This task is the natural "freeze" point for the engine-side API — once these tests land, downstream tasks (IDENT-005 CLI, Sprint 09 wiring) can rely on the registry/resolver behaving per contract.
- If IDENT-003's `find_by_pid_any_start` helper wasn't added to `Registry`, add it here as part of the stale-registration test setup (the test needs it to construct the scenario).

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "tdd_mode": "shared",
  "shared_test_ref": "crates/but-authz/tests/registry.rs; crates/but-authz/tests/process.rs; crates/but-authz/tests/authorize.rs::resolve_principal_with_registry_*",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": false,
    "requires_seeded_evidence": true,
    "tdd_mode": "shared"
  },
  "tdd_justification": "Test-only consolidation task. Its value is the shared engine-side suite that must be GREEN after IDENT-001, IDENT-002, and IDENT-003 land; requiring per-task RED would be ambiguous because a correct implementation may already satisfy the new adversarial tests.",
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN a populated Registry with 3 distinct entries WHEN write(path) then load(path) THEN loaded == original AND every entry's (pid, start_time, agent_id, expires_at, registered_by) round-trips byte-exact",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-authz",
        "negative_control": {
          "would_fail_if": [
            "a static write stub would persist 0 entries",
            "a TOML schema stub would omit registered_by for pid=1001"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "populated_registry_3_entries",
            "action": {
              "actor": "ci",
              "steps": [
                "write(path)",
                "load(path)",
                "compare with ==",
                "assert each entry's 5 fields match"
              ]
            },
            "end_state": {
              "must_observe": [
                "loaded == original with 3 entries",
                "entry(pid=1001).agent_id == \"agent-a\"",
                "entry(pid=1003).registered_by == \"test\""
              ],
              "must_not_observe": [
                "loaded registry count == 0",
                "loaded != original",
                "missing \"registered_by\" field"
              ]
            }
          }
        ]
      },
      "verify": "cargo test -p but-authz --test registry registry_round_trip"
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN an entry with expires_at = 1000 WHEN gc(999) THEN still resolvable; WHEN gc(1000) THEN still resolvable (boundary); WHEN gc(1001) THEN no longer resolvable",
      "test_tier": "unit",
      "verification_service": "but-authz",
      "scenario": {
        "tier": "visible",
        "test_tier": "unit",
        "verification_service": "but-authz",
        "unit_test_justified": "pure time-comparison logic — no I/O",
        "negative_control": {
          "would_fail_if": [
            "a wrong <= constant would drop the entry at gc(1000)",
            "a no-op gc stub would keep the entry after gc(1001)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "registry_entry_expires_at_1000",
            "action": {
              "actor": "ci",
              "steps": [
                "gc(999) then resolve → must be Some",
                "gc(1000) then resolve → must be Some",
                "gc(1001) then resolve → must be None"
              ]
            },
            "end_state": {
              "must_observe": [
                "resolve((42,100)) == Some(\"agent-a\") after gc(999)",
                "resolve((42,100)) == Some(\"agent-a\") after gc(1000)",
                "resolve((42,100)) == None after gc(1001)"
              ],
              "must_not_observe": [
                "resolve((42,100)) == None after gc(1000)",
                "entry count == 1 after gc(1001)"
              ]
            }
          }
        ]
      },
      "verify": "cargo test -p but-authz --test registry ttl_boundary"
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN (1234, 100, \"agent-a\") WHEN resolve((1234, 200)) THEN None AND original entry unchanged (still resolvable at (1234, 100))",
      "test_tier": "unit",
      "verification_service": "but-authz",
      "scenario": {
        "tier": "visible",
        "test_tier": "unit",
        "verification_service": "but-authz",
        "unit_test_justified": "pure map lookup",
        "negative_control": {
          "would_fail_if": [
            "a pid-only resolver stub would return agent-a for (1234,200)",
            "a mutating lookup stub would delete the original (1234,100) entry"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "registry_one_entry_1234_100",
            "action": {
              "actor": "ci",
              "steps": [
                "resolve((1234, 200)) → must be None",
                "resolve((1234, 100)) → must still be Some(\"agent-a\")"
              ]
            },
            "end_state": {
              "must_observe": [
                "resolve((1234,200)) == None",
                "resolve((1234,100)) == Some(\"agent-a\")"
              ],
              "must_not_observe": [
                "resolve((1234,200)) == Some(\"agent-a\")",
                "resolve((1234,100)) == None",
                "empty registry count == 0"
              ]
            }
          }
        ]
      },
      "verify": "cargo test -p but-authz --test registry pid_reuse_rejection"
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN 4 threads each register+write to the SAME path WHEN all finish THEN loaded registry has all 4 entries AND file parses without error",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-authz",
        "negative_control": {
          "would_fail_if": [
            "a non-atomic write stub would leave a partial TOML file",
            "a last-writer-wins stub would omit three of four entries"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "empty_path_4_threads_ready",
            "action": {
              "actor": "ci",
              "steps": [
                "spawn 4 threads each doing load → register → write",
                "join all 4",
                "load(path)",
                "resolve each of the 4 (pid, start) keys"
              ]
            },
            "end_state": {
              "must_observe": [
                "loaded registry count == 4",
                "resolve((1001,1730000001)) == Some(\"agent-1\")",
                "resolve((1004,1730000004)) == Some(\"agent-4\")"
              ],
              "must_not_observe": [
                "loaded registry count == 0",
                "missing pid 1001",
                "TOML parse error"
              ]
            }
          }
        ]
      },
      "verify": "cargo test -p but-authz --test registry concurrent_writes"
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the test process WHEN process_start_time(current_pid()) is called twice THEN both return Ok(t) with equal t AND t > 1_000_000_000",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-authz",
        "negative_control": {
          "would_fail_if": [
            "a static current-time stub would make t0 != t1",
            "a wrong factor stub would return t0 <= 1000000000"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "test_process",
            "action": {
              "actor": "ci",
              "steps": [
                "t0 = process_start_time(current_pid())",
                "t1 = process_start_time(current_pid())"
              ]
            },
            "end_state": {
              "must_observe": [
                "t0 == t1",
                "t0 > 1000000000"
              ],
              "must_not_observe": [
                "t0 != t1",
                "t0 == 0",
                "t0 <= 1000000000"
              ]
            }
          }
        ]
      },
      "verify": "cargo test -p but-authz --test process start_time_monotonic"
    },
    {
      "id": "AC-6",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the 5 scenarios from IDENT-003 AC-1..5 WHEN each is exercised via temp_env + in-memory Registry THEN outcomes match IDENT-003 expected Ok/Err per case",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-authz",
        "negative_control": {
          "would_fail_if": [
            "a disconnected resolver stub would pass only one hardcoded outcome",
            "an omitted env-fallback branch would fail IDENT-003 AC-2 and AC-5"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "five_resolver_scenarios",
            "action": {
              "actor": "ci",
              "steps": [
                "run each of the 5 IDENT-003 AC scenarios",
                "assert the documented outcome per case"
              ]
            },
            "end_state": {
              "must_observe": [
                "IDENT-003 AC-1 outcome == Ok(\"rust-implementer\")",
                "IDENT-003 AC-2 outcome == Ok(\"dev\")",
                "IDENT-003 AC-3 outcome == Err(\"Denial::unregistered\")",
                "IDENT-003 AC-4 outcome == Err(\"Denial::stale_registration\")",
                "IDENT-003 AC-5 outcome == Ok(\"dev\")"
              ],
              "must_not_observe": [
                "0 IDENT-003 outcomes executed",
                "any outcome == Ok(\"ghost\")",
                "empty resolver scenario list"
              ]
            }
          }
        ]
      },
      "verify": "cargo test -p but-authz --test authorize resolve_principal_with_registry"
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "maps_to_ac": "AC-1",
      "description": "registry_round_trip passes (3 entries byte-exact)",
      "verify": "cargo test -p but-authz --test registry registry_round_trip"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "maps_to_ac": "AC-2",
      "description": "ttl_boundary passes (gc(999)/gc(1000) keep; gc(1001) drops)",
      "verify": "cargo test -p but-authz --test registry ttl_boundary"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "maps_to_ac": "AC-3",
      "description": "pid_reuse_rejection passes",
      "verify": "cargo test -p but-authz --test registry pid_reuse_rejection"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "maps_to_ac": "AC-4",
      "description": "concurrent_writes passes (4 threads, all entries land)",
      "verify": "cargo test -p but-authz --test registry concurrent_writes"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "maps_to_ac": "AC-5",
      "description": "start_time_monotonic passes",
      "verify": "cargo test -p but-authz --test process start_time_monotonic"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "maps_to_ac": "AC-6",
      "description": "resolve_principal_with_registry_* (5 sub-tests) pass",
      "verify": "cargo test -p but-authz --test authorize resolve_principal_with_registry"
    }
  ],
  "fixtures": {
    "populated_registry_3_entries": {
      "seed_method": "public_api",
      "description": "Registry test fixture creates exactly three distinct in-memory registrations before write/load round-trip.",
      "records": [
        {
          "pid": 1001,
          "start_time": 1730000001,
          "agent_id": "agent-a",
          "registered_at": 1730000001,
          "expires_at": 1730003601,
          "registered_by": "test"
        },
        {
          "pid": 1002,
          "start_time": 1730000002,
          "agent_id": "agent-b",
          "registered_at": 1730000002,
          "expires_at": 1730003602,
          "registered_by": "test"
        },
        {
          "pid": 1003,
          "start_time": 1730000003,
          "agent_id": "agent-c",
          "registered_at": 1730000003,
          "expires_at": 1730003603,
          "registered_by": "test"
        }
      ]
    },
    "registry_entry_expires_at_1000": {
      "seed_method": "public_api",
      "description": "Registry has one entry keyed by (pid=42,start_time=100) with expires_at exactly 1000.",
      "records": [
        {
          "pid": 42,
          "start_time": 100,
          "agent_id": "agent-a",
          "registered_at": 900,
          "expires_at": 1000,
          "registered_by": "test"
        }
      ]
    },
    "registry_one_entry_1234_100": {
      "seed_method": "public_api",
      "description": "Registry has one live entry for pid 1234 and start_time 100 before PID-reuse lookup.",
      "records": [
        {
          "pid": 1234,
          "start_time": 100,
          "agent_id": "agent-a",
          "registered_at": 90,
          "expires_at": 1000,
          "registered_by": "test"
        }
      ]
    },
    "empty_path_4_threads_ready": {
      "seed_method": "migration_fixture",
      "description": "Writable but_testsupport scenario path starts without agents-runtime.toml before four concurrent load/register/write threads.",
      "records": [
        {
          "path": "repo/.gitbutler/agents-runtime.toml",
          "file_exists": false,
          "thread_count": 4,
          "planned_pids": [
            1001,
            1002,
            1003,
            1004
          ]
        }
      ]
    },
    "test_process": {
      "seed_method": "public_api",
      "description": "The currently running cargo test process is used for real current_pid/process_start_time checks.",
      "records": [
        {
          "pid_source": "std::process::id()",
          "process_state": "running",
          "expected_start_time_min": 1000000001
        }
      ]
    },
    "five_resolver_scenarios": {
      "seed_method": "public_api",
      "description": "Composite fixture enumerates IDENT-003 AC-1..5 resolver cases with concrete registry/env/config records.",
      "records": [
        {
          "case": "AC-1",
          "pid": 1234,
          "start_time": 1730000000,
          "agent_id": "rust-implementer",
          "expected": "Ok(rust-implementer)"
        },
        {
          "case": "AC-2",
          "registry_entries": 0,
          "env_handle": "dev",
          "env_flag": "1",
          "expected": "Ok(dev)"
        },
        {
          "case": "AC-3",
          "registry_entries": 0,
          "env_flag": "<unset>",
          "expected": "Err(Denial::unregistered)"
        },
        {
          "case": "AC-4",
          "pid": 1234,
          "registered_start_time": 1730000000,
          "observed_start_time": 1730000099,
          "expected": "Err(Denial::stale_registration)"
        },
        {
          "case": "AC-5",
          "registry": null,
          "env_handle": "dev",
          "env_flag": "1",
          "expected": "Ok(dev)"
        }
      ]
    }
  }
}
-->
