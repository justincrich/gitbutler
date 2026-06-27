# IDENT-017 — `crates/but-authz/src/authorize.rs` — LOCK and VERIFY the deny-default: env-var path on governed repos requires `BUT_AUTHZ_ALLOW_ENV_HANDLE=1`; absent flag + registry miss → `Denial::unregistered` (perm.denied); document `resolve_principal_from_env` as test/CI-only

**Sprint:** [Sprint 10](./SPRINT.md) · **Agent:** `rust-implementer` · **Estimate:** 90 min · **Type:** FEATURE · **Status:** Complete · **Proposed By:** rust-planner

## Background

rust-implementer owns the authorize.rs surface and must add the VERIFY + LOCK tests + documentation; this is not a flag-check implementation (already present — env-fallback allow-branch L221-223 + deny-default `Denial::unregistered` at L235) but a test+doc hardening task..

**Why it matters.** Closes the IDENT deprecation arc: on a governed repo the registry path is the default; the env-var path survives only as an opt-in escape hatch behind `BUT_AUTHZ_ALLOW_ENV_HANDLE=1`.

**Provides:** Resolver-level deny-default invariant verified and locked, Documentation marking `resolve_principal_from_env` as test/CI-only

**Consumes:** Sprint-08 `resolve_principal_with_registry` (IDENT-003), Sprint-08 `BUT_AUTHZ_ALLOW_ENV_HANDLE` constant (IDENT-002), Sprint-08 `Denial::unregistered` (IDENT-002)

**Boundary contracts:**
- `resolve_principal_with_registry(None, cfg)` with env-only `BUT_AGENT_HANDLE=dev` + flag-unset → `Denial::unregistered(pid, observed_start_time)`
- Flag-set (`BUT_AUTHZ_ALLOW_ENV_HANDLE=1`) + BUT_AGENT_HANDLE set → principal via env fallback
- Registry hit → principal regardless of flag


## Critical Constraints

**MUST:**
- Add explicit integration tests proving the deny-default at the resolver level (NOT at callsites)
- Test all three resolution paths: registry hit → principal, registry miss + flag-set → env fallback, registry miss + env-only handle + flag-unset → Denial::unregistered with code=perm.denied
- Document `resolve_principal_from_env` as test/CI-only in its doc-comment
- Preserve the existing flag-check implementation in `allow_env_handle` and `resolve_principal_with_registry_lookup` — do NOT reimplement
- All tests use `but_testsupport` fixtures, never `std::env::temp_dir().join(format!(...))`

**NEVER:**
- Modify the flag-check logic in `allow_env_handle` or `resolve_principal_with_registry_lookup` — the policy is already correct
- Make `resolve_principal_from_env` literally unreachable (the flag-gated fallback must remain functional)
- Touch but-api callsites — Sprint-09 IDENT-010 owns the swap to `resolve_principal_with_runtime_registry`
- Remove `BUT_AUTHZ_ALLOW_ENV_HANDLE` or `BUT_AGENT_HANDLE` constants
- Use unwrap() or expect() in library code — propagate Result

**STRICTLY:**
- BLOCKED-UNTIL Sprint-09 IDENT-010 completes (the callsite swap must land before the deny-default is observable at gates)
- The gate/source invariant is 'governed but-api gates call resolve_principal_with_runtime_registry, and that wrapper delegates to but_authz::resolve_principal_with_registry'; resolver-level tests may call `resolve_principal_with_registry` directly
- Tests run via `cargo test -p but-authz --test authorize`, never workspace-wide `cargo test`
- Clippy with `-D warnings` must pass

## Specification

**Objective:** LOCK and VERIFY the resolver-level deny-default by adding explicit integration tests that prove the flag-check works (registry hit → principal, registry miss + flag-set → env fallback, registry miss + env-only handle + flag-unset → Denial::unregistered), and document `resolve_principal_from_env` as test/CI-only.

**Success state:** cargo test -p but-authz --test authorize passes 3 new tests (registry hit, flag-set env fallback, env-only flag-unset denial). The resolver-level flag-check is verified with no code changes to the policy itself. `resolve_principal_from_env` doc-comment marks it as test/CI-only. cargo check -p but-authz clean. cargo clippy -p but-authz --all-targets -- -D warnings passes.

## Acceptance Criteria

**AC-1 (PRIMARY)** — PRIMARY — Resolver-level deny-default verified: env-only handle + flag-unset + registry miss → Denial::unregistered (perm.denied)
- **GIVEN:** A governed repo with empty registry + `BUT_AUTHZ_ALLOW_ENV_HANDLE` unset + `BUT_AGENT_HANDLE=dev`
- **WHEN:** `resolve_principal_with_registry(None, cfg)` is called with a governed config
- **THEN:** Returns `Err(Denial::unregistered(pid, observed_start_time))` whose `code == Denial::PERM_DENIED_CODE` (`perm.denied`)
- **Verify:** `cargo test -p but-authz --test authorize -- test_resolver_env_handle_denied_without_flag`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-authz · **FLOW_REF:** UC-IDENT-03
- **Scenario:** `start_ref=governed_repo_empty_registry_flag_unset`; `must_observe` = ["Err(Denial::unregistered(pid, observed_start_time)) with literal code 'perm.denied'", 'Denial.code equals Denial::PERM_DENIED_CODE (`perm.denied`)']; `must_not_observe` = ['Ok(())', "Ok(Principal { agent_id: 'dev', ... })", 'any non-error return']; `negative_control.would_fail_if` = ['Flag-check logic removed from allow_env_handle at line 247-248', 'Registry miss path omits the deny and returns Ok(())', "Denial::unregistered variant deleted or its code changed from 'perm.denied'"].

**AC-2** — Resolver-level env-fallback path: flag-set + BUT_AGENT_HANDLE → principal
- **GIVEN:** A governed repo with empty registry + `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` + `BUT_AGENT_HANDLE=dev`
- **WHEN:** `resolve_principal_with_registry(None, cfg)` is called
- **THEN:** Returns `Ok(Principal { agent_id: 'dev', ... })` via env fallback
- **Verify:** `cargo test -p but-authz --test authorize -- test_resolver_env_fallback`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-authz · **FLOW_REF:** UC-IDENT-03
- **Scenario:** `start_ref=governed_repo_empty_registry_flag_set`; `must_observe` = ["Ok(Principal { agent_id: 'dev', ... }) with the exact literal 'dev'", 'Principal returned via env-fallback path, not registry']; `must_not_observe` = ['Err(Denial::unregistered)', 'Err(Denial::no_handle)', 'any Denial variant', 'empty registry passthrough']; `negative_control.would_fail_if` = ['Flag-check in allow_env_handle removed or stubbed with a mock', 'Env fallback path disconnected from resolve_principal', 'BUT_AGENT_HANDLE not read from env var'].

**AC-3** — Resolver-level registry-hit path: registry hit → principal regardless of flag
- **GIVEN:** A governed repo with registry entry (test_pid, test_start_time) → rust-implementer
- **WHEN:** `resolve_principal_with_registry(Some(reg), cfg)` is called with test_pid/process_start_time
- **THEN:** Returns `Ok(Principal { agent_id: 'rust-implementer', ... })` even with flag unset
- **Verify:** `cargo test -p but-authz --test authorize -- test_resolver_registry_hit`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-authz · **FLOW_REF:** UC-IDENT-03
- **Scenario:** `start_ref=governed_repo_with_registry_entry`; `must_observe` = ["Ok(Principal { agent_id: 'rust-implementer', ... }) with the exact literal 'rust-implementer'", 'Registry hit bypasses flag check entirely']; `must_not_observe` = ['Env fallback invoked', 'Flag checked', 'BUT_AUTHZ_ALLOW_ENV_HANDLE read', 'Denial returned']; `negative_control.would_fail_if` = ['Registry lookup stubbed with a mock', 'Registry::resolve returns wrong agent_id or empty', 'Registry not loaded from file'].

**AC-4** — `resolve_principal_from_env` documented as test/CI-only
- **GIVEN:** The existing `resolve_principal_from_env` function in authorize.rs
- **WHEN:** Reading its doc-comment
- **THEN:** Doc-comment explicitly states 'TEST/CI-ONLY — governed but-api gates use `resolve_principal_with_runtime_registry`; resolver-level tests may call `resolve_principal_with_registry` directly'
- **Verify:** `Manual review of authorize.rs doc-comment + cargo doc --no-deps -p but-authz`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-authz · **FLOW_REF:** None
- **Scenario:** `start_ref=authorize.rs_source_code`; `must_observe` = ["Doc-comment contains the literal 'TEST/CI-ONLY' substring", "Doc-comment names the exact literal 'resolve_principal_with_runtime_registry'", "Doc-comment names the underlying literal 'resolve_principal_with_registry'", "Doc-comment omits 'production gate' language"]; `must_not_observe` = ['Doc-comment missing TEST/CI-ONLY marker', 'Doc-comment suggests direct resolver use at production gates', 'empty doc-comment']; `negative_control.would_fail_if` = ['Doc-comment not updated or remains empty', 'Doc-comment misleadingly suggests production gates call the direct resolver', 'Required marker omitted from doc'].

## Test Criteria

| ID | Boolean | Maps to AC |
|----|---------|------------|
| TC-1 | Resolver deny-default (env-only handle + flag-unset + registry miss → Denial::unregistered perm.denied) is true | AC-1 |
| TC-2 | Resolver env fallback (flag-set + BUT_AGENT_HANDLE → principal) is true | AC-2 |
| TC-3 | Resolver registry hit → principal regardless of flag is true | AC-3 |
| TC-4 | resolve_principal_from_env doc-comment marks it as TEST/CI-ONLY is true | AC-4 |

## Reading List

1. `crates/but-authz/src/authorize.rs:192-245` — resolve_principal_with_registry + resolve_principal_with_registry_lookup — the env-fallback ALLOW branch (L221-223 `allow_env_handle`) and the deny-default `Err(Denial::unregistered)` at L235 are ALREADY implemented (Sprint-08 IDENT-003). This task only VERIFIES+LOCKS them.
2. `crates/but-authz/src/authorize.rs:12-13` — BUT_AGENT_HANDLE and BUT_AUTHZ_ALLOW_ENV_HANDLE constants
3. `crates/but-authz/src/authorize.rs:60-102` — resolve_principal and resolve_principal_from_env shape — must document as test/CI-only
4. `crates/but-authz/tests/authorize.rs:1-50` — Existing resolver tests — add 3 new tests for deny-default, env fallback, registry hit

## Guardrails

**WRITE-ALLOWED:**
- crates/but-authz/tests/authorize.rs (MODIFY — add 3 new integration tests for the three resolution paths)
- crates/but-authz/src/authorize.rs (MODIFY-DOC-ONLY — update resolve_principal_from_env doc-comment to mark it as TEST/CI-ONLY; do NOT touch the flag-check implementation at lines 221-222 or 247-248)

**WRITE-PROHIBITED:**
- crates/but-api/src/** — Sprint-09 IDENT-010 owns the callsite swap; do NOT touch gate callsites
- crates/but-authz/src/authorize.rs flag-check logic (lines 192-249 (resolver policy: allow-branch L221-223, deny-default L235, allow_env_handle L247-249)) — the policy is already correct; do NOT reimplement
- crates/but/** — Sprint-09 owns CLI migration
- Any change to Registry, Denial types, or process.rs — those are Sprint-08 deliverables

## Code Pattern

**Reference:** IDENT-003 (Sprint 08) shipped `resolve_principal_with_registry` with the flag-check already implemented; IDENT-002 (Sprint 08) shipped `BUT_AUTHZ_ALLOW_ENV_HANDLE` and `Denial::unregistered`; UC-IDENT-03 specifies the resolution order: registry → flag-gated env → denial

**Pattern:** Test-then-doc pattern: First write integration tests proving the three resolution paths (registry hit, flag-set env fallback, flag-unset denial), then update the doc-comment to reflect the invariant. The code under test (the flag-check) is already correct.

**Source:** `IDENT-010 exemplar — TDD RED→GREEN, but here RED already passes (the flag-check is there), so we write ASSERTION tests that lock the correct behavior in place.`

**Design notes:**
- The flag-check at line 221-222 in `allow_env_handle` is ALREADY the correct Sprint-10 policy — this task only VERIFIES it with tests
- `resolve_principal_from_env` remains callable as the flag-gated fallback — do NOT make it literally unreachable
- The invariant 'but-api gates call resolve_principal_with_runtime_registry and the wrapper delegates to but_authz::resolve_principal_with_registry' is enforced by Sprint-09 IDENT-010's callsite swap, not by making the env resolver unreachable
- IDENT-020 will add the invariant grep to `invariant_build_gates.rs` after Sprint-09 lands

**Anti-pattern:** Do NOT reimplement the flag-check or modify `allow_env_handle` logic. Do NOT make `resolve_principal_from_env` literally unreachable (the flag-gated fallback must remain functional). Do NOT touch but-api callsites (Sprint-09 territory).

## Agent Instructions

TDD RED→GREEN per AC (integration against the real crate — `but-authz` / `but-api` — real git/gitoxide, NO mocks):
1. **RED:** write each AC's failing test first (against the live code / current start state).
2. **GREEN:** make the minimal change (test-only for IDENT-017/018/019; invariant assertions for IDENT-020; doc-comments for IDENT-021).
3. Run `cargo fmt`, `cargo clippy -p <crate> --all-targets -- -D warnings`, then the task's verify commands.
4. Commit via `but commit` (governed). Note: this task is BLOCKED-UNTIL Sprint-09 IDENT-009/010/011 land.

## Orchestrator Verification Protocol

- `cargo test -p but-authz --test authorize` → exit 0, all 3 new tests pass (test_resolver_env_handle_denied_without_flag, test_resolver_env_fallback, test_resolver_registry_hit)
- `cargo check -p but-authz --all-targets` → exit 0, clean compile
- `cargo clippy -p but-authz --all-targets -- -D warnings` → exit 0, no warnings
- `cargo doc --no-deps -p but-authz && grep 'TEST/CI-ONLY' crates/but-authz/src/authorize.rs` → Doc-comment contains TEST/CI-ONLY marker

## Agent Assignment

**Agent:** `rust-implementer` — rust-implementer owns the authorize.rs surface and must add the VERIFY + LOCK tests + documentation; this is not a flag-check implementation (already present — env-fallback allow-branch L221-223 + deny-default `Denial::unregistered` at L235) but a test+doc hardening task.
**Pairing:** none (single-surface Rust task). Honors `crates/AGENTS.md` + `crates/WORKSPACE_MODEL.md`.

## Evidence Gates

- `cargo test -p but-authz --test authorize` (exit 0, all 3 new tests pass (test_resolver_env_handle_denied_without_flag, test_resolver_env_fallback, test_resolver_registry_hit))
- `cargo check -p but-authz --all-targets` (exit 0, clean compile)
- `cargo clippy -p but-authz --all-targets -- -D warnings` (exit 0, no warnings)
- `cargo doc --no-deps -p but-authz && grep 'TEST/CI-ONLY' crates/but-authz/src/authorize.rs` (Doc-comment contains TEST/CI-ONLY marker)

## Review Criteria

- AC-1: PRIMARY — Resolver-level deny-default verified: env-only handle + flag-unset + registry miss → Denial::unregistered (perm.denied) — verified by `cargo test -p but-authz --test authorize -- test_resolver_env_handle_denied_without_flag`.
- AC-2: Resolver-level env-fallback path: flag-set + BUT_AGENT_HANDLE → principal — verified by `cargo test -p but-authz --test authorize -- test_resolver_env_fallback`.
- AC-3: Resolver-level registry-hit path: registry hit → principal regardless of flag — verified by `cargo test -p but-authz --test authorize -- test_resolver_registry_hit`.
- AC-4: `resolve_principal_from_env` documented as test/CI-only — verified by `Manual review of authorize.rs doc-comment + cargo doc --no-deps -p but-authz`.
- Honors NEVER: Modify the flag-check logic in `allow_env_handle` or `resolve_principal_with_registry_lookup` — the policy is already correct.

## Dependencies

- **Depends on:** Sprint-09 IDENT-010 (but-api gate callsites must swap to `resolve_principal_with_runtime_registry`, whose wrapper delegates to `but_authz::resolve_principal_with_registry`, before the deny-default is observable at gates), Sprint-09 IDENT-009 (AGENTS_PATH constant must exist for the full Sprint-10 invariant), Sprint-08 IDENT-003 (resolve_principal_with_registry exists), Sprint-08 IDENT-002 (BUT_AUTHZ_ALLOW_ENV_HANDLE and Denial::unregistered exist)
- **Blocks:** IDENT-018 (test migration uses the verified resolver behavior), IDENT-020 (invariant extension requires the resolver locked and documented), IDENT-021 (doc audit requires the resolved invariant documented in authorize.rs)
- **Capabilities:** CAP-AUTHZ-01

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "IDENT-017",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "governed_repo_empty_registry_flag_unset": {
      "description": "governed repo with permissions.toml (dev=contents:write) committed at target ref; runtime registry empty/absent; BUT_AUTHZ_ALLOW_ENV_HANDLE unset; BUT_AGENT_HANDLE=dev",
      "seed_method": "public_api",
      "records": [
        "permissions.toml committed at target ref with [[agent]] id='dev' permissions=['contents:write']",
        "registry file absent or empty at BUT_AGENT_REGISTRY_PATH",
        "BUT_AUTHZ_ALLOW_ENV_HANDLE env var unset",
        "BUT_AGENT_HANDLE=dev"
      ]
    },
    "governed_repo_empty_registry_flag_set": {
      "description": "governed repo with permissions.toml; runtime registry empty; BUT_AUTHZ_ALLOW_ENV_HANDLE=1; BUT_AGENT_HANDLE=dev",
      "seed_method": "public_api",
      "records": [
        "permissions.toml committed at target ref with [[agent]] id='dev' permissions=['contents:write']",
        "registry file absent or empty",
        "BUT_AUTHZ_ALLOW_ENV_HANDLE=1",
        "BUT_AGENT_HANDLE=dev"
      ]
    },
    "governed_repo_with_registry_entry": {
      "description": "governed repo with permissions.toml; runtime registry has (test_pid, test_start_time) → rust-implementer; flag unset",
      "seed_method": "public_api",
      "records": [
        "permissions.toml committed with [[agent]] id='rust-implementer' permissions=['contents:write']",
        "registry file has entry: pid=test_pid, start_time=test_start_time, agent_id='rust-implementer', ttl=4h",
        "BUT_AUTHZ_ALLOW_ENV_HANDLE unset"
      ]
    },
    "authorize.rs_source_code": {
      "description": "source file crates/but-authz/src/authorize.rs with resolve_principal_from_env function",
      "seed_method": "public_api",
      "records": [
        "authorize.rs exists at crates/but-authz/src/authorize.rs",
        "resolve_principal_from_env function present at lines 60-102"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "PRIMARY — Resolver-level deny-default verified: env-only handle + flag-unset + registry miss → Denial::unregistered (perm.denied)",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "verify": "cargo test -p but-authz --test authorize -- test_resolver_env_handle_denied_without_flag",
      "flow_ref": "UC-IDENT-03",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-authz",
        "start_ref": "governed_repo_empty_registry_flag_unset",
        "must_observe": [
          "Err(Denial::unregistered(pid, observed_start_time)) with literal code 'perm.denied'",
          "Denial.code equals Denial::PERM_DENIED_CODE (`perm.denied`)"
        ],
        "must_not_observe": [
          "Ok(())",
          "Ok(Principal { agent_id: 'dev', ... })",
          "any non-error return"
        ],
        "negative_control": {
          "would_fail_if": [
            "Flag-check logic removed from allow_env_handle at line 247-248",
            "Registry miss path omits the deny and returns Ok(())",
            "Denial::unregistered variant deleted or its code changed from 'perm.denied'"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "governed_repo_empty_registry_flag_unset",
            "action": {
              "actor": "test_harness",
              "steps": [
                "Create governed repo with permissions.toml",
                "Set registry to empty/absent",
                "Unset BUT_AUTHZ_ALLOW_ENV_HANDLE",
                "Set BUT_AGENT_HANDLE=dev",
                "Call resolve_principal_with_registry(None, cfg)"
              ]
            },
            "end_state": {
              "must_observe": [
                "Err(Denial::unregistered(pid, observed_start_time))",
                "Denial code == 'perm.denied'"
              ],
              "must_not_observe": [
                "Ok(())",
                "Ok(Principal)",
                "empty success"
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
      "description": "Resolver-level env-fallback path: flag-set + BUT_AGENT_HANDLE → principal",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "verify": "cargo test -p but-authz --test authorize -- test_resolver_env_fallback",
      "flow_ref": "UC-IDENT-03",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-authz",
        "start_ref": "governed_repo_empty_registry_flag_set",
        "must_observe": [
          "Ok(Principal { agent_id: 'dev', ... }) with the exact literal 'dev'",
          "Principal returned via env-fallback path, not registry"
        ],
        "must_not_observe": [
          "Err(Denial::unregistered)",
          "Err(Denial::no_handle)",
          "any Denial variant",
          "empty registry passthrough"
        ],
        "negative_control": {
          "would_fail_if": [
            "Flag-check in allow_env_handle removed or stubbed with a mock",
            "Env fallback path disconnected from resolve_principal",
            "BUT_AGENT_HANDLE not read from env var"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "governed_repo_empty_registry_flag_set",
            "action": {
              "actor": "test_harness",
              "steps": [
                "Set BUT_AUTHZ_ALLOW_ENV_HANDLE=1",
                "Set BUT_AGENT_HANDLE=dev",
                "Empty registry",
                "Call resolve_principal_with_registry(None, cfg)"
              ]
            },
            "end_state": {
              "must_observe": [
                "Ok(Principal { agent_id: 'dev' })"
              ],
              "must_not_observe": [
                "Err",
                "empty"
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
      "description": "Resolver-level registry-hit path: registry hit → principal regardless of flag",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "verify": "cargo test -p but-authz --test authorize -- test_resolver_registry_hit",
      "flow_ref": "UC-IDENT-03",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-authz",
        "start_ref": "governed_repo_with_registry_entry",
        "must_observe": [
          "Ok(Principal { agent_id: 'rust-implementer', ... }) with the exact literal 'rust-implementer'",
          "Registry hit bypasses flag check entirely"
        ],
        "must_not_observe": [
          "Env fallback invoked",
          "Flag checked",
          "BUT_AUTHZ_ALLOW_ENV_HANDLE read",
          "Denial returned"
        ],
        "negative_control": {
          "would_fail_if": [
            "Registry lookup stubbed with a mock",
            "Registry::resolve returns wrong agent_id or empty",
            "Registry not loaded from file"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "governed_repo_with_registry_entry",
            "action": {
              "actor": "test_harness",
              "steps": [
                "Create registry with (test_pid, test_start_time) → rust-implementer",
                "Set governed config with rust-implementer in agents.toml",
                "Call resolve_principal_with_registry(Some(reg), cfg) with test_pid"
              ]
            },
            "end_state": {
              "must_observe": [
                "Ok(Principal { agent_id: 'rust-implementer' })"
              ],
              "must_not_observe": [
                "Env fallback invoked",
                "empty registry (0 entries)",
                "Denial::unregistered"
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
      "description": "`resolve_principal_from_env` documented as test/CI-only",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "verify": "Manual review of authorize.rs doc-comment + cargo doc --no-deps -p but-authz",
      "flow_ref": null,
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-authz",
        "start_ref": "authorize.rs_source_code",
        "must_observe": [
          "Doc-comment contains the literal 'TEST/CI-ONLY' substring",
          "Doc-comment names the exact literal 'resolve_principal_with_runtime_registry'",
          "Doc-comment names the underlying literal 'resolve_principal_with_registry'",
          "Doc-comment omits 'production gate' language"
        ],
        "must_not_observe": [
          "Doc-comment missing TEST/CI-ONLY marker",
          "Doc-comment suggests direct resolver use at production gates",
          "empty doc-comment"
        ],
        "negative_control": {
          "would_fail_if": [
            "Doc-comment not updated or remains empty",
            "Doc-comment misleadingly suggests production gates call the direct resolver",
            "Required marker omitted from doc"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "authorize.rs_source_code",
            "action": {
              "actor": "test_harness",
              "steps": [
                "Read authorize.rs",
                "Locate resolve_principal_from_env doc-comment"
              ]
            },
            "end_state": {
              "must_observe": [
                "Doc-comment contains literal 'TEST/CI-ONLY'",
                "Doc-comment contains literal 'resolve_principal_with_runtime_registry'",
                "Doc-comment contains literal 'resolve_principal_with_registry'"
              ],
              "must_not_observe": [
                "missing marker",
                "empty"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "Resolver deny-default (env-only handle + flag-unset + registry miss → Denial::unregistered perm.denied) is true",
      "maps_to_ac": "AC-1",
      "verify": "cargo test -p but-authz --test authorize -- test_resolver_env_handle_denied_without_flag"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "Resolver env fallback (flag-set + BUT_AGENT_HANDLE → principal) is true",
      "maps_to_ac": "AC-2",
      "verify": "cargo test -p but-authz --test authorize -- test_resolver_env_fallback"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "Resolver registry hit → principal regardless of flag is true",
      "maps_to_ac": "AC-3",
      "verify": "cargo test -p but-authz --test authorize -- test_resolver_registry_hit"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "resolve_principal_from_env doc-comment marks it as TEST/CI-ONLY is true",
      "maps_to_ac": "AC-4",
      "verify": "Manual review of authorize.rs doc-comment"
    }
  ]
}
-->
