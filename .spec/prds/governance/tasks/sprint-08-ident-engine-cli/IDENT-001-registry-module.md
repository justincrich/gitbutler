# IDENT-001 — `crates/but-authz/src/registry.rs` — `Registry` (atomic write, TTL, PID-reuse defense)

**Sprint:** [Sprint 08](./SPRINT.md) · **Agent:** `rust-implementer` · **Estimate:** 240 min · **Type:** FEATURE · **Status:** READY · **Proposed By:** rust-planner (`--no-specialists`)

## Background

Today an agent's identity is the env var `BUT_AGENT_HANDLE` — caller-controlled and unforgeable by the engine. The IDENT initiative (v1.4.0) replaces it with a runtime PID registry whose identifiers are anchored in committed `agents.toml`. The registry maps `(pid, start_time) → Registration { agent_id, expiry, registered_at, registered_by }` so the engine can resolve the caller by process attributes the caller cannot control.

**Why it matters.** Without a registry, every "principal X did Y" audit log entry is a claim, not a fact. The registry is the substrate that lets `resolve_principal_with_registry` (IDENT-003) and the 8 gate callsites (Sprint 09) attribute governed actions to real PIDs.

**Current state.** No registry exists. `crates/but-authz/src/authorize.rs:100` (`resolve_principal_from_env`) is the only identity source. Tests use `temp_env::with_var("BUT_AGENT_HANDLE", ...)`.

**Desired state.** A `Registry` struct in a new `crates/but-authz/src/registry.rs` module with load/write/register/unregister/resolve/gc. Atomic writes (temp + rename). TTL expiry (lazy GC on read). PID-reuse defense via `(pid, start_time)` composite key.

## Critical Constraints

- **MUST** keep `crates/but-authz` free of `git2` — use only `std` + `serde` + `toml` + `anyhow` (per `crates/but-authz/Cargo.toml`).
- **MUST** write the runtime file atomically (write to `<path>.tmp.<pid>` then `std::fs::rename`). The file MUST always be parseable after a write, including across a crash mid-write.
- **NEVER** mutate the registry file in place (read-modify-write directly); always go through `Registry::write` which performs the atomic rename.
- **STRICTLY** enforce the `(pid, start_time)` composite key — a `register` call with the same pid but a different `start_time` is a NEW entry, not an update; the previous entry (if any) becomes unreachable via `resolve` (caller should `gc` it).
- **MUST** treat a missing registry file as `Registry::empty()` (not an error) — governed repos with no registrations are common.

## Specification

**Objective:** Add `crates/but-authz/src/registry.rs` exposing `Registry` with load/write/register/unregister/resolve/gc.

**Success state:** `cargo test -p but-authz --test registry` passes (the IDENT-004 test suite). `Registry::load` on a missing path returns an empty registry without error. `Registry::write` produces a parseable TOML file. `Registry::resolve((pid, start))` returns the registered agent_id on a fresh hit and `None` on miss/stale/expired.

## Acceptance Criteria

**AC-1** — GIVEN an empty registry via `Registry::empty()` WHEN `Registry::register(1234, 1730000000, "rust-implementer", 14400, "operator")` is called THEN the entry is stored and `Registry::resolve((1234, 1730000000))` returns `Some("rust-implementer")`.

**AC-2** — GIVEN a registry with a live entry `(1234, 1730000000) → rust-implementer` WHEN `Registry::resolve((1234, 1730000099))` is called (same pid, different start_time) THEN it returns `None` (PID-reuse defense).

**AC-3** — GIVEN a registry with an entry whose `expires_at = 1730014400` WHEN `Registry::gc(1730014401)` is called THEN the entry is dropped and a subsequent `resolve` returns `None`; `gc(1730014400)` does NOT drop it (boundary is exclusive on the future side — `expires_at` is the last valid second).

**AC-4** — GIVEN a registry written to a path via `Registry::write(path)` WHEN a second `Registry::load(path)` is called THEN the resulting registry is equal (by `PartialEq`) to the original; the on-disk file parses as valid TOML of shape `[[registration]] pid=… start_time=… agent_id=… registered_at=… expires_at=… registered_by=…`.

**AC-5 (error)** — GIVEN a registry path whose parent directory does not exist WHEN `Registry::write(path)` is called THEN it returns `Err` whose `Display` names the path; the test does NOT observe a partial file at `path`.

## Test Criteria

| ID | Boolean | Maps to AC |
|----|---------|------------|
| TC-1 | `Registry::empty().resolve((1, 1)) == None` is true | AC-1 |
| TC-2 | After register, `resolve` with the SAME `(pid, start_time)` returns `Some(agent_id)` is true | AC-1 |
| TC-3 | After register with `(p, s0)`, `resolve` with `(p, s1)` where `s1 != s0` returns `None` is true | AC-2 |
| TC-4 | `gc(expires_at + 1)` drops the entry; `gc(expires_at)` does NOT is true | AC-3 |
| TC-5 | After `write(p)` then `load(p)`, the two registries are `PartialEq` is true | AC-4 |
| TC-6 | The on-disk file parses as valid TOML with the `[[registration]]` schema is true | AC-4 |
| TC-7 | `write` to a path with a missing parent returns `Err` naming the path is true | AC-5 |

## Reading List

- `crates/but-authz/src/authorize.rs:67-89` — `resolve_principal` shape (mirror the `Result<Principal, Denial>` discipline)
- `crates/but-authz/src/config.rs:33-68` — `load_governance_config` (mirror atomic-read discipline; note: target-ref blob vs filesystem distinction — the registry is always filesystem)
- `crates/but-authz/src/principal.rs` — `Principal`, `PrincipalId` (the `agent_id` is a `PrincipalId` by another name; reuse the type)
- `crates/but-authz/src/denial.rs` — `Denial` (IDENT-003 adds variants; this task only consumes the type)

## Guardrails

**WRITE-ALLOWED:**
- `crates/but-authz/src/registry.rs` (NEW)
- `crates/but-authz/src/lib.rs` (export `Registry` + sub-types)
- `crates/but-authz/Cargo.toml` (`[dependencies]` — no new deps required; `toml` + `serde` already present)
- `crates/but-authz/tests/registry.rs` (RED test slice for IDENT-001 only; IDENT-004 owns the full suite)

**WRITE-PROHIBITED:**
- `crates/but-authz/src/authorize.rs` (IDENT-003 owns this)
- `crates/but-authz/src/process.rs` (IDENT-002 owns this)
- Any file under `crates/but-api/` (Sprint 09 owns callsite wiring)
- Any file under `crates/but/` (IDENT-005/006 own the CLI)

## Code Pattern

**Reference:** `crates/but-api/src/legacy/governance.rs:347-349` — `load_governance_config(&repo, &target_ref)?; let caller = but_authz::resolve_principal_from_env(&config)?;` — the registry resolver follows the same `Result<_, anyhow::Error>` shape.

**Source:** `crates/but-authz/src/config.rs:274` (`toml::from_str::<PermissionsWire>(&permissions_blob)`) — mirror this parse discipline for the runtime file.

**Anti-pattern:** do NOT use `std::fs::write` directly (no atomic guarantee); do NOT use `File::create` + manual write (crash mid-write leaves a partial file). Always go through `tempfile::NamedTempFile::persist()` OR a hand-rolled write-to-temp-then-rename with the temp file in the SAME directory (so the rename is atomic on POSIX).

## Agent Instructions

TDD RED→GREEN→REFACTOR per AC:

1. **RED AC-1+AC-2:** Write `tests/registry.rs` (placeholder — IDENT-004 owns the full suite, but this task needs the registry to compile + pass its own red tests). Assert `Registry::empty().resolve((1, 1)) == None`. Then assert the register+resolve round-trip. Run `cargo test -p but-authz --test registry -- IDENT_001` → must fail (registry doesn't exist yet).
2. **GREEN:** Create `src/registry.rs` with the `Registry` struct, `Registration` struct, `load`/`write`/`register`/`unregister`/`resolve`/`gc`. Use `tempfile::NamedTempFile` for atomic writes (add to dev-deps if missing; for prod atomic-rename use `std::fs::rename` after writing to `<path>.tmp.<pid>`).
3. **REFACTOR:** Pull the TOML (de)serialization behind a private `RegistryWire` struct mirroring `PrincipalWire`'s shape; keep `Registry` itself a pure in-memory `BTreeMap<(u32, u64), Registration>`.
4. Run `cargo check -p but-authz --all-targets` then `cargo test -p but-authz --test registry`. Commit via `but commit` (governed path — the implementer is registered as `rust-implementer` by `but-run-sprint`).
5. Do NOT touch `authorize.rs` — IDENT-003 will add the resolver that consumes this `Registry`.

## Orchestrator Verification Protocol

The orchestrator (`/but-run-sprint`) verifies this task by:

1. Running `cargo test -p but-authz --test registry` and asserting exit 0.
2. Running `cargo check -p but-authz --all-targets` and asserting no errors.
3. Confirming `crates/but-authz/src/registry.rs` exists and exports a `Registry` type whose `resolve` method returns `Option<AgentId>`.
4. Confirming `crates/but-authz/src/lib.rs` now re-exports `Registry`.

## Agent Assignment

**Agent:** `rust-implementer` — owns the `crates/but-authz` crate; the registry is a pure-Rust data structure with filesystem I/O, no domain logic foreign to the crate's existing scope.

**Pairing:** none. The work is self-contained; IDENT-003 (resolver) consumes the result in a sibling task.

## Evidence Gates

- `cargo test -p but-authz --test registry` exit 0 (RED→GREEN proof)
- `crates/but-authz/src/registry.rs` file exists with the documented API
- No edits to `authorize.rs`, `process.rs`, `but-api/**`, or `but/**` (the WRITE-PROHIBITED list)

## Review Criteria

- `Registry::write` uses an atomic temp-file-then-rename (not direct `fs::write`).
- The `(pid, start_time)` composite key is enforced — same pid with different start_time is a NEW entry.
- Missing-file path returns `Registry::empty()`, not an error.
- TTL boundary: `gc(expires_at)` does NOT drop; `gc(expires_at + 1)` DOES drop.

## Dependencies

- **depends_on:** none (first task in the sprint).
- **blocks:** IDENT-003 (resolver consumes `Registry`), IDENT-004 (tests consume `Registry`).

## Notes

- The runtime file is **NOT** committed to git (it's gitignored at the repo level — the `.git/info/exclude` entry is added in Sprint 09 alongside `agents.toml` itself).
- The `registered_by` field is informational — it's the agent_id (or `"operator"`) that called `but agent register`. The engine does not authorize `register` calls themselves (any caller with fs access can register); this is documented in the threat model at `12-uc-agent-identity.md` §1.

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "tdd_mode": "red_first",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true,
    "tdd_mode": "red_first"
  },
  "tdd_justification": "Behavioral registry implementation with meaningful unit/integration assertions for register, resolve, TTL, PID-reuse, and atomic-write behavior. Pre-dispatch RED evidence should come from the IDENT-001 slice in crates/but-authz/tests/registry.rs failing because Registry does not exist or does not satisfy the documented API.",
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN an empty registry via Registry::empty() WHEN register(1234, 1730000000, \"rust-implementer\", 14400, \"operator\") is called THEN the entry is stored and resolve((1234, 1730000000)) returns Some(\"rust-implementer\")",
      "test_tier": "unit",
      "verification_service": "but-authz",
      "scenario": {
        "tier": "visible",
        "test_tier": "unit",
        "verification_service": "but-authz",
        "unit_test_justified": "pure in-memory data structure with no I/O — parse/register/resolve are deterministic logic",
        "negative_control": {
          "would_fail_if": [
            "a static stub that never inserts records would leave registry count at 0",
            "an empty-key stub would ignore the concrete (pid=1234,start_time=1730000000) key"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "empty_registry",
            "action": {
              "actor": "ci",
              "steps": [
                "Registry::empty()",
                "register((1234, 1730000000), \"rust-implementer\", ttl=14400, by=\"operator\")",
                "resolve((1234, 1730000000))"
              ]
            },
            "end_state": {
              "must_observe": [
                "resolve((1234,1730000000)) == Some(\"rust-implementer\")",
                "registry entry count == 1"
              ],
              "must_not_observe": [
                "resolve((1234,1730000000)) == None",
                "registry entry count == 0",
                "Some(\"ghost\")"
              ]
            }
          }
        ]
      },
      "verify": "cargo test -p but-authz --test registry"
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN a registry with (1234, 1730000000) → rust-implementer WHEN resolve((1234, 1730000099)) is called THEN it returns None (PID-reuse defense)",
      "test_tier": "unit",
      "verification_service": "but-authz",
      "scenario": {
        "tier": "visible",
        "test_tier": "unit",
        "verification_service": "but-authz",
        "unit_test_justified": "pure key-equality logic — no I/O",
        "negative_control": {
          "would_fail_if": [
            "a pid-only stub would return rust-implementer for pid=1234 regardless of start_time",
            "a hardcoded resolver would ignore start_time=1730000099"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_registry_one_entry",
            "action": {
              "actor": "ci",
              "steps": [
                "resolve((1234, 1730000099))"
              ]
            },
            "end_state": {
              "must_observe": [
                "resolve((1234,1730000099)) == None",
                "resolve((1234,1730000000)) == Some(\"rust-implementer\")"
              ],
              "must_not_observe": [
                "resolve((1234,1730000099)) == Some(\"rust-implementer\")",
                "empty registry count == 0"
              ]
            }
          }
        ]
      },
      "verify": "cargo test -p but-authz --test registry"
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN an entry with expires_at = 1730014400 WHEN gc(1730014401) THEN the entry is dropped; gc(1730014400) does NOT drop it",
      "test_tier": "unit",
      "verification_service": "but-authz",
      "scenario": {
        "tier": "visible",
        "test_tier": "unit",
        "verification_service": "but-authz",
        "unit_test_justified": "pure time comparison logic",
        "negative_control": {
          "would_fail_if": [
            "a static gc stub that does nothing would keep the entry after now=1730014401",
            "a wrong constant <= comparison would delete the entry at now=1730014400"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_registry_with_expiry_1730014400",
            "action": {
              "actor": "ci",
              "steps": [
                "gc(now=1730014400)",
                "resolve(entry) -> must still be Some",
                "gc(now=1730014401)",
                "resolve(entry) -> must now be None"
              ]
            },
            "end_state": {
              "must_observe": [
                "resolve((1234,1730000000)) == Some(\"rust-implementer\") after gc(1730014400)",
                "resolve((1234,1730000000)) == None after gc(1730014401)"
              ],
              "must_not_observe": [
                "resolve((1234,1730000000)) == None after gc(1730014400)",
                "entry count remains 1 after gc(1730014401)"
              ]
            }
          }
        ]
      },
      "verify": "cargo test -p but-authz --test registry"
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN a registry written via write(path) WHEN load(path) is called THEN the result is PartialEq to the original and the on-disk file parses as valid TOML with [[registration]] blocks",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-authz",
        "negative_control": {
          "would_fail_if": [
            "an in-place static write stub would leave no [[registration]] records on disk",
            "a lossy TOML stub would omit registered_by=\"operator\""
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "populated_registry_in_memory",
            "action": {
              "actor": "ci",
              "steps": [
                "write(path)",
                "load(path)",
                "compare with ==",
                "re-parse the file as toml::Value"
              ]
            },
            "end_state": {
              "must_observe": [
                "loaded == original with 2 registrations",
                "TOML contains \"[[registration]]\" and agent_id=\"rust-implementer\""
              ],
              "must_not_observe": [
                "loaded registry count == 0",
                "toml parse error",
                "missing \"registered_by\" field"
              ]
            }
          }
        ]
      },
      "verify": "cargo test -p but-authz --test registry"
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN a registry path whose parent directory does not exist WHEN write(path) is called THEN it returns Err naming the path and no partial file is left behind",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-authz",
        "negative_control": {
          "would_fail_if": [
            "a write stub that creates missing dirs would hide parent_exists=false",
            "a partial-write bug would leave /nonexistent/dir/agents-runtime.toml.tmp.* behind"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "nonexistent_parent_dir",
            "action": {
              "actor": "ci",
              "steps": [
                "write(path=\"/nonexistent/dir/agents-runtime.toml\")",
                "check Err display",
                "check no file at path"
              ]
            },
            "end_state": {
              "must_observe": [
                "Err display contains \"/nonexistent/dir/agents-runtime.toml\"",
                "path_exists(\"/nonexistent/dir/agents-runtime.toml\") == false"
              ],
              "must_not_observe": [
                "Ok(())",
                "empty error message",
                "path_exists(\"/nonexistent/dir/agents-runtime.toml\") == true"
              ]
            }
          }
        ]
      },
      "verify": "cargo test -p but-authz --test registry"
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "maps_to_ac": "AC-1",
      "description": "Registry::empty().resolve((1, 1)) == None",
      "verify": "cargo test -p but-authz --test registry"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "maps_to_ac": "AC-1",
      "description": "After register, resolve with same (pid, start_time) returns Some(agent_id)",
      "verify": "cargo test -p but-authz --test registry"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "maps_to_ac": "AC-2",
      "description": "After register with (p, s0), resolve with (p, s1) where s1 != s0 returns None",
      "verify": "cargo test -p but-authz --test registry"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "maps_to_ac": "AC-3",
      "description": "gc(expires_at + 1) drops; gc(expires_at) does NOT",
      "verify": "cargo test -p but-authz --test registry"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "maps_to_ac": "AC-4",
      "description": "After write(p) then load(p), the two registries are PartialEq",
      "verify": "cargo test -p but-authz --test registry"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "maps_to_ac": "AC-4",
      "description": "The on-disk file parses as valid TOML with [[registration]] schema",
      "verify": "cargo test -p but-authz --test registry"
    },
    {
      "id": "TC-7",
      "type": "test_criterion",
      "maps_to_ac": "AC-5",
      "description": "write to missing-parent path returns Err naming the path",
      "verify": "cargo test -p but-authz --test registry"
    }
  ],
  "fixtures": {
    "empty_registry": {
      "seed_method": "public_api",
      "description": "Registry::empty() with zero registration records before AC-1 action.",
      "records": [
        {
          "registrations": 0,
          "file_path": "<none>",
          "map_keys": []
        }
      ]
    },
    "seeded_registry_one_entry": {
      "seed_method": "public_api",
      "description": "In-memory Registry seeded with exactly one live registration keyed by (pid=1234, start_time=1730000000).",
      "records": [
        {
          "pid": 1234,
          "start_time": 1730000000,
          "agent_id": "rust-implementer",
          "registered_at": 1730000000,
          "expires_at": 1730014400,
          "registered_by": "operator"
        }
      ]
    },
    "seeded_registry_with_expiry_1730014400": {
      "seed_method": "public_api",
      "description": "In-memory Registry with one entry whose expires_at boundary is 1730014400.",
      "records": [
        {
          "pid": 1234,
          "start_time": 1730000000,
          "agent_id": "rust-implementer",
          "registered_at": 1730000000,
          "expires_at": 1730014400,
          "registered_by": "operator"
        }
      ]
    },
    "populated_registry_in_memory": {
      "seed_method": "public_api",
      "description": "In-memory Registry populated before write(path) with two live registrations for TOML round-trip verification.",
      "records": [
        {
          "pid": 1234,
          "start_time": 1730000000,
          "agent_id": "rust-implementer",
          "registered_at": 1730000000,
          "expires_at": 1730014400,
          "registered_by": "operator"
        },
        {
          "pid": 5678,
          "start_time": 1730000100,
          "agent_id": "rust-reviewer",
          "registered_at": 1730000100,
          "expires_at": 1730014500,
          "registered_by": "operator"
        }
      ]
    },
    "nonexistent_parent_dir": {
      "seed_method": "public_api",
      "description": "Registry::empty() plus write target /nonexistent/dir/agents-runtime.toml whose parent directory is absent before write.",
      "records": [
        {
          "registrations": 0,
          "target_path": "/nonexistent/dir/agents-runtime.toml",
          "parent_exists": false
        }
      ]
    }
  }
}
-->
