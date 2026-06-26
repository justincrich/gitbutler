# IDENT-003 — `crates/but-authz/src/authorize.rs` — `resolve_principal_with_registry` + `Denial::unregistered`/`stale_registration`

**Sprint:** [Sprint 08](./SPRINT.md) · **Agent:** `rust-implementer` · **Estimate:** 180 min · **Type:** FEATURE · **Status:** READY · **Proposed By:** rust-planner (`--no-specialists`)

## Background

IDENT-001 lands the `Registry` data structure; IDENT-002 lands the per-OS `process_start_time` helper. This task wires them into the resolver that the 8 gate callsites (Sprint 09) will consume. The resolver's policy is the load-bearing decision: registry hit → principal; registry miss + `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` → env fallback; else → `Denial::unregistered`. In Sprint 08 the env-fallback is still the DEFAULT (the policy flip happens in Sprint 10); this task only delivers the new resolver function without touching the 8 callsites.

**Why it matters.** This is the boundary where caller-controlled identity (`BUT_AGENT_HANDLE`) becomes engine-attested identity (registry hit). The composite `(pid, start_time)` lookup + the structured denials are what make spoofing an explicit failure instead of a quiet success.

**Current state.** `crates/but-authz/src/authorize.rs:100` has `resolve_principal_from_env(cfg)` reading `BUT_AGENT_HANDLE`. `crates/but-authz/src/denial.rs` has `no_handle()` and `unknown_principal(handle)`. Sprint 07 (STEER) added the structured carrier fields (`class`, `held_permissions`, `authorized_actions`, `do_not`).

**Desired state.** `resolve_principal_with_registry(reg: Option<&Registry>, cfg: &GovConfig) -> Result<Principal, Denial>` exists alongside the legacy `resolve_principal_from_env`. `Denial` gains `unregistered(pid, start)` and `stale_registration(pid, start)`, both `code = perm.denied` (consistent with `no_handle`).

## Critical Constraints

- **MUST** preserve `resolve_principal_from_env` unchanged (the 80+ existing tests and the 8 gate callsites still call it; Sprint 09 migrates the callsites, Sprint 10 flips the default).
- **MUST** resolve in the documented order: (1) registry hit → principal; (2) registry miss + `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` → env fallback; (3) else → `Denial::unregistered`. The order is the policy.
- **NEVER** mutate the `Registry` from inside the resolver — it's a `&Registry` borrow, read-only. Mutation (GC, register) happens at the CLI boundary (`but agent register`).
- **STRICTLY** keep `Denial::unregistered` and `stale_registration` code-equal to `Denial::no_handle` (`perm.denied`) so downstream carriers (STEER) and the 8 gates see the same machine code.
- **MUST** read the env flag via the same `lookup` injection pattern as `resolve_principal` (testable without `std::env::set_var` races).

## Specification

**Objective:** Add `resolve_principal_with_registry` + `Denial::unregistered` / `stale_registration` to `crates/but-authz/src/authorize.rs`.

**Success state:** `cargo test -p but-authz --test authorize` passes (extended in IDENT-004). The new resolver returns the registered principal on hit, falls through to env on flag-set miss, and denies with `unregistered` otherwise. A stale registration (start_time mismatch) yields `stale_registration`.

## Acceptance Criteria

**AC-1** — GIVEN a `Registry` with `(1234, 1730000000) → rust-implementer` AND a `GovConfig` where `rust-implementer` holds `contents:write` WHEN `resolve_principal_with_registry(Some(®), &cfg)` is called from PID 1234 with start_time 1730000000 THEN it returns `Ok(principal)` where `principal.id().as_str() == "rust-implementer"`.

**AC-2** — GIVEN `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` is set AND `BUT_AGENT_HANDLE=dev` is set AND a `Registry` with NO entry for the calling PID WHEN `resolve_principal_with_registry(Some(®), &cfg)` is called THEN it falls through to the env path and returns `Ok(principal)` where `principal.id().as_str() == "dev"`.

**AC-3** — GIVEN `BUT_AUTHZ_ALLOW_ENV_HANDLE` is UNSET AND a `Registry` with no entry for the calling PID WHEN `resolve_principal_with_registry(Some(®), &cfg)` is called THEN it returns `Denial::unregistered(pid, start)` whose `.code == Denial::PERM_DENIED_CODE`.

**AC-4** — GIVEN a `Registry` with an entry for `(1234, 1730000000)` AND the calling process is PID 1234 but with `start_time = 1730000099` (PID reused) WHEN `resolve_principal_with_registry(Some(®), &cfg)` is called AND the env flag is unset THEN it returns `Denial::stale_registration(1234, 1730000099)` whose `.code == Denial::PERM_DENIED_CODE`.

**AC-5** — GIVEN `resolve_principal_with_registry(None, &cfg)` (no registry supplied — e.g., a non-governed repo or a runtime path that doesn't exist) AND `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` AND `BUT_AGENT_HANDLE=dev` WHEN called THEN it falls through to env and returns the `dev` principal (no panic on `None`).

## Test Criteria

| ID | Boolean | Maps to AC |
|----|---------|------------|
| TC-1 | Registry hit returns the registered `principal.id()` is true | AC-1 |
| TC-2 | Registry miss + flag set → env fallback returns env handle's principal is true | AC-2 |
| TC-3 | Registry miss + flag unset → `Denial::unregistered` with `code = perm.denied` is true | AC-3 |
| TC-4 | Stale registration (start_time mismatch) → `Denial::stale_registration` with `code = perm.denied` is true | AC-4 |
| TC-5 | `resolve_principal_with_registry(None, &cfg)` does NOT panic; falls through to env when flag set is true | AC-5 |
| TC-6 | `resolve_principal_from_env` is unchanged (existing tests still pass) is true | AC-5 |

## Reading List

- `crates/but-authz/src/authorize.rs:60-102` — `resolve_principal` and `resolve_principal_from_env` (the legacy path to preserve; the new fn's injected-lookup pattern is identical)
- `crates/but-authz/src/authorize.rs:138-171` — `Denial::no_handle()` and `unknown_principal()` (mirror the constructor shape for the new variants)
- `crates/but-authz/src/denial.rs` — `Denial` struct + `PERM_DENIED_CODE` constant (IDENT-003 adds variants here OR in `authorize.rs` — pick the file the rest of the constructors live in)
- `crates/but-authz/src/principal.rs:82-100` — `Principal` and `PrincipalId` (the resolver's return type)
- `crates/but-authz/src/config.rs:120-130` — `GovConfig::principal_authorities` (used to look up the registered agent_id's authority set)

## Guardrails

**WRITE-ALLOWED:**
- `crates/but-authz/src/authorize.rs` (new fn + `Denial::unregistered` / `stale_registration` constructors; preserve existing)
- `crates/but-authz/src/denial.rs` (if the new constructors live here instead — mirror the existing constructor placement)
- `crates/but-authz/src/lib.rs` (export `resolve_principal_with_registry` if not already)
- `crates/but-authz/tests/authorize.rs` (RED test slice for IDENT-003 resolver behavior only)

**WRITE-PROHIBITED:**
- The 8 callsites in `crates/but-api/**` (Sprint 09 owns the migration)
- `crates/but-authz/src/registry.rs` (IDENT-001)
- `crates/but-authz/src/process.rs` (IDENT-002)
- `crates/but/` (IDENT-005/006)

## Code Pattern

**Reference:** `crates/but-authz/src/authorize.rs:67-89` (`resolve_principal`) — the new fn follows the same `Result<Principal, Denial>` + injected-lookup shape, except the lookup is `Registry::resolve` first, env second.

**Source (sketch):**
```rust
pub const ALLOW_ENV_HANDLE: &str = "BUT_AUTHZ_ALLOW_ENV_HANDLE";

pub fn resolve_principal_with_registry(
    reg: Option<&Registry>,
    cfg: &GovConfig,
) -> Result<Principal, Denial> {
    resolve_principal_with_registry_and_lookup(
        reg,
        cfg,
        |key| std::env::var_os(key),
        crate::process::current_pid,
        crate::process::process_start_time,
    )
}

pub(crate) fn resolve_principal_with_registry_and_lookup(
    reg: Option<&Registry>,
    cfg: &GovConfig,
    env_lookup: impl Fn(&str) -> Option<OsString>,
    pid_fn: impl Fn() -> u32,
    start_fn: impl Fn(u32) -> Result<u64, anyhow::Error>,
) -> Result<Principal, Denial> {
    if let Some(reg) = reg {
        let pid = pid_fn();
        // If start_time read fails, treat as registry miss (don't deny harder than env path would).
        if let Ok(start) = start_fn(pid) {
            if let Some(agent_id) = reg.resolve(&(pid, start)) {
                return resolve_principal_from_handle(&agent_id, cfg);
            }
            // Stale-registration detection: if a registration EXISTS for (pid, _) but not (pid, start),
            // it's a PID-reuse scenario — surface as stale_registration rather than generic unregistered.
            if let Some(stale_entry) = reg.find_by_pid_any_start(pid) {
                return Err(Denial::stale_registration(pid, start, stale_entry.start_time));
            }
        }
    }
    // Registry miss. Check env fallback flag.
    let allow_env = env_lookup(ALLOW_ENV_HANDLE)
        .map(|v| v == OsString::from("1"))
        .unwrap_or(false);
    if allow_env {
        return resolve_principal(env_lookup, cfg);
    }
    Err(Denial::unregistered(pid_fn()))
}
```

**Anti-pattern:** do NOT read `BUT_AGENT_HANDLE` directly inside `resolve_principal_with_registry` — delegate to `resolve_principal` for the env path so there's exactly one place that reads the env var.

## Agent Instructions

TDD RED→GREEN→REFACTOR per AC:

1. **RED:** Extend `crates/but-authz/tests/authorize.rs` (placeholder — IDENT-004 owns the full suite). Assert the 5 ACs above with `temp_env::with_var(...)` for the env-flag paths and an in-memory `Registry` for the registry paths. Run `cargo test -p but-authz --test authorize -- IDENT_003` → must fail (the new fn doesn't exist).
2. **GREEN:** Implement `resolve_principal_with_registry` + the injected-lookup variant (for testability). Add `Denial::unregistered(pid)` and `Denial::stale_registration(pid, observed_start, registered_start)` constructors. Keep `.code == PERM_DENIED_CODE`.
3. **REFACTOR:** Pull the common "resolve a handle string to a `Principal` against `GovConfig`" logic out of `resolve_principal` into a private `resolve_principal_from_handle(handle: &str, cfg: &GovConfig) -> Result<Principal, Denial>` helper that both the env path and the registry path call. This avoids drift between the two paths' authority lookup.
4. Run `cargo check -p but-authz --all-targets` then `cargo test -p but-authz --test authorize`. Commit via `but commit`.

## Orchestrator Verification Protocol

1. `cargo test -p but-authz --test authorize` exit 0.
2. `cargo check -p but-authz --all-targets` clean.
3. `crates/but-authz/src/authorize.rs` exports `resolve_principal_with_registry` with the documented signature.
4. `Denial::unregistered` and `stale_registration` exist with `.code == perm.denied`.

## Agent Assignment

**Agent:** `rust-implementer` — owns `crates/but-authz`. The resolver logic is a thin layer over IDENT-001's `Registry` and the existing `resolve_principal_from_env`; no new domain.

**Pairing:** none. Depends on IDENT-001 and IDENT-002 landing first (the registry + process modules).

## Evidence Gates

- `cargo test -p but-authz --test authorize` exit 0 (RED→GREEN proof)
- 5 ACs above each have a passing test
- `resolve_principal_from_env` unchanged (existing tests still green)

## Review Criteria

- Resolution order is exactly: registry → flag-gated env → denial.
- `Denial::unregistered` and `stale_registration` carry `.code == perm.denied` (so STEER's `class` field, `held_permissions`, `authorized_actions` extensions compose with them identically).
- The `None` registry path doesn't panic.
- Stale-registration distinguishes "no entry for this pid at all" (unregistered) from "entry exists for this pid with a different start_time" (stale) — both are `perm.denied` but with different messages.

## Dependencies

- **depends_on:** IDENT-001 (Registry), IDENT-002 (process module).
- **blocks:** IDENT-004 (tests consume the new fn), Sprint 09 (the 8 gate callsites consume it).

## Notes

- The stale-registration detection requires a `Registry::find_by_pid_any_start(pid) -> Option<&Registration>` helper that IDENT-001 doesn't currently expose. Either add it to IDENT-001 (amend) or implement it as a private helper here that scans `reg.entries()` (slow but fine — registries are tiny). Pick the amend path if IDENT-001 hasn't merged yet; otherwise add the helper here.
- `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` semantics in Sprint 08: it's the DEFAULT-allowed path because Sprint 10 hasn't flipped the policy. The resolver implemented here is correct regardless — the flag-gate is enforced at the resolver level, not by the caller.

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
  "tdd_justification": "Behavioral resolver implementation with meaningful tests for registry hit, flag-gated env fallback, unregistered denial, stale-registration denial, and None-registry fallback. Pre-dispatch RED evidence should come from the IDENT-003 resolver tests in crates/but-authz/tests/authorize.rs failing before the new resolver and denial constructors exist.",
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN a Registry with (1234, 1730000000) → rust-implementer AND GovConfig where rust-implementer holds contents:write WHEN resolve_principal_with_registry(Some(®), &cfg) is called from PID 1234 / start 1730000000 THEN Ok(principal) with id \"rust-implementer\"",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-authz",
        "negative_control": {
          "would_fail_if": [
            "a disconnected env-only stub would ignore registry key (1234,1730000000)",
            "a hardcoded principal stub would return rust-implementer without checking GovConfig"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "registry_and_config_seeded",
            "action": {
              "actor": "ci",
              "steps": [
                "seed Registry with (1234, 1730000000) → rust-implementer",
                "seed GovConfig with rust-implementer = contents:write",
                "call resolve_principal_with_registry(Some(®), &cfg) with pid_fn=||1234, start_fn=|_|Ok(1730000000)"
              ]
            },
            "end_state": {
              "must_observe": [
                "resolve_principal_with_registry(pid=1234,start=1730000000) == Ok(\"rust-implementer\")",
                "principal.id().as_str() == \"rust-implementer\""
              ],
              "must_not_observe": [
                "Err(\"perm.denied\")",
                "Ok(\"dev\")",
                "empty principal id"
              ]
            }
          }
        ]
      },
      "verify": "cargo test -p but-authz --test authorize resolve_principal_with_registry_hit"
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN BUT_AUTHZ_ALLOW_ENV_HANDLE=1 AND BUT_AGENT_HANDLE=dev AND a Registry with no entry for the calling pid WHEN resolve_principal_with_registry(Some(®), &cfg) THEN falls through to env and returns Ok(principal) with id \"dev\"",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-authz",
        "negative_control": {
          "would_fail_if": [
            "a static deny stub would ignore BUT_AUTHZ_ALLOW_ENV_HANDLE=\"1\"",
            "an env-name typo would omit BUT_AGENT_HANDLE=\"dev\""
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "registry_empty_dev_in_env",
            "action": {
              "actor": "ci",
              "steps": [
                "temp_env::with_var(BUT_AUTHZ_ALLOW_ENV_HANDLE, Some(\"1\"))",
                "temp_env::with_var(BUT_AGENT_HANDLE, Some(\"dev\"))",
                "resolve_principal_with_registry(Some(&empty_reg), &cfg)"
              ]
            },
            "end_state": {
              "must_observe": [
                "resolve_principal_with_registry(empty_reg, env dev) == Ok(\"dev\")",
                "BUT_AUTHZ_ALLOW_ENV_HANDLE == \"1\""
              ],
              "must_not_observe": [
                "Err(\"Denial::unregistered\")",
                "Ok(\"rust-implementer\")",
                "empty principal id"
              ]
            }
          }
        ]
      },
      "verify": "cargo test -p but-authz --test authorize resolve_principal_with_registry_env_fallback_when_allowed"
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN BUT_AUTHZ_ALLOW_ENV_HANDLE UNSET AND a Registry with no entry for the calling pid WHEN resolve_principal_with_registry(Some(®), &cfg) THEN Denial::unregistered(pid) with .code == perm.denied",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-authz",
        "negative_control": {
          "would_fail_if": [
            "a default-allow stub would return Ok(\"dev\") when flag is absent",
            "a disconnected env fallback would ignore BUT_AUTHZ_ALLOW_ENV_HANDLE unset"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "registry_empty_flag_unset",
            "action": {
              "actor": "ci",
              "steps": [
                "temp_env::with_var(BUT_AUTHZ_ALLOW_ENV_HANDLE, None::<&str>)",
                "resolve_principal_with_registry(Some(&empty_reg), &cfg)"
              ]
            },
            "end_state": {
              "must_observe": [
                "Err denial.code == \"perm.denied\"",
                "Err message contains \"unregistered\" and pid 1234"
              ],
              "must_not_observe": [
                "Ok(\"dev\")",
                "Ok(\"rust-implementer\")",
                "empty denial message"
              ]
            }
          }
        ]
      },
      "verify": "cargo test -p but-authz --test authorize resolve_principal_with_registry_unregistered_denial"
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN a Registry with (1234, 1730000000) AND the calling process is PID 1234 with start_time 1730000099 (PID reused) AND env flag unset WHEN resolve_principal_with_registry is called THEN Denial::stale_registration(1234, 1730000099, 1730000000) with .code == perm.denied",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-authz",
        "negative_control": {
          "would_fail_if": [
            "a pid-only stub would return rust-implementer for reused pid 1234",
            "a generic-denial stub would omit stale start_time 1730000099"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "registry_with_stale_entry",
            "action": {
              "actor": "ci",
              "steps": [
                "seed Registry with (1234, 1730000000) → rust-implementer",
                "call resolver with pid_fn=||1234, start_fn=|_|Ok(1730000099)",
                "flag unset"
              ]
            },
            "end_state": {
              "must_observe": [
                "Err denial.code == \"perm.denied\"",
                "Err message contains \"stale\"",
                "Err message contains \"1730000099\" and \"1730000000\""
              ],
              "must_not_observe": [
                "Ok(\"rust-implementer\")",
                "Denial::unregistered without \"stale\"",
                "empty denial message"
              ]
            }
          }
        ]
      },
      "verify": "cargo test -p but-authz --test authorize resolve_principal_with_registry_stale_registration_denial"
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN resolve_principal_with_registry(None, &cfg) AND BUT_AUTHZ_ALLOW_ENV_HANDLE=1 AND BUT_AGENT_HANDLE=dev WHEN called THEN falls through to env and returns the dev principal (no panic on None)",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-authz",
        "negative_control": {
          "would_fail_if": [
            "a None-registry panic stub would crash before env fallback",
            "a static deny stub would ignore BUT_AUTHZ_ALLOW_ENV_HANDLE=\"1\""
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "no_registry_env_set",
            "action": {
              "actor": "ci",
              "steps": [
                "resolve_principal_with_registry(None, &cfg)",
                "env has BUT_AUTHZ_ALLOW_ENV_HANDLE=1 + BUT_AGENT_HANDLE=dev"
              ]
            },
            "end_state": {
              "must_observe": [
                "resolve_principal_with_registry(None, env dev) == Ok(\"dev\")",
                "principal.id().as_str() == \"dev\""
              ],
              "must_not_observe": [
                "Err(\"perm.denied\")",
                "panic",
                "empty principal id"
              ]
            }
          }
        ]
      },
      "verify": "cargo test -p but-authz --test authorize resolve_principal_with_registry_none_env_fallback"
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "maps_to_ac": "AC-1",
      "description": "Registry hit returns the registered principal.id()",
      "verify": "cargo test -p but-authz --test authorize resolve_principal_with_registry_hit"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "maps_to_ac": "AC-2",
      "description": "Registry miss + flag set → env fallback",
      "verify": "cargo test -p but-authz --test authorize resolve_principal_with_registry_env_fallback_when_allowed"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "maps_to_ac": "AC-3",
      "description": "Registry miss + flag unset → Denial::unregistered with perm.denied",
      "verify": "cargo test -p but-authz --test authorize resolve_principal_with_registry_unregistered_denial"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "maps_to_ac": "AC-4",
      "description": "Stale registration → Denial::stale_registration with perm.denied",
      "verify": "cargo test -p but-authz --test authorize resolve_principal_with_registry_stale_registration_denial"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "maps_to_ac": "AC-5",
      "description": "None registry + flag set → env fallback without panic",
      "verify": "cargo test -p but-authz --test authorize resolve_principal_with_registry_none_env_fallback"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "maps_to_ac": "AC-5",
      "description": "resolve_principal_from_env unchanged (existing tests still pass)",
      "verify": "cargo test -p but-authz --test authorize resolve_principal_from_env"
    }
  ],
  "fixtures": {
    "registry_and_config_seeded": {
      "seed_method": "public_api",
      "description": "In-memory Registry and GovConfig both contain rust-implementer before resolver call.",
      "records": [
        {
          "registry": {
            "pid": 1234,
            "start_time": 1730000000,
            "agent_id": "rust-implementer",
            "expires_at": 1730014400,
            "registered_by": "operator"
          }
        },
        {
          "gov_config": {
            "principal_id": "rust-implementer",
            "permissions": [
              "contents:write"
            ]
          }
        }
      ]
    },
    "registry_empty_dev_in_env": {
      "seed_method": "public_api",
      "description": "Empty Registry, GovConfig has dev, and injected env lookup returns fallback flag plus BUT_AGENT_HANDLE=dev.",
      "records": [
        {
          "registry_entries": 0
        },
        {
          "env": {
            "BUT_AUTHZ_ALLOW_ENV_HANDLE": "1",
            "BUT_AGENT_HANDLE": "dev"
          }
        },
        {
          "gov_config": {
            "principal_id": "dev",
            "permissions": [
              "contents:read"
            ]
          }
        }
      ]
    },
    "registry_empty_flag_unset": {
      "seed_method": "public_api",
      "description": "Empty Registry, GovConfig has rust-implementer, and injected env lookup returns no BUT_AUTHZ_ALLOW_ENV_HANDLE.",
      "records": [
        {
          "registry_entries": 0
        },
        {
          "env": {
            "BUT_AUTHZ_ALLOW_ENV_HANDLE": "<unset>",
            "BUT_AGENT_HANDLE": "<unset>"
          }
        },
        {
          "observed_pid": 1234,
          "observed_start_time": 1730000000
        }
      ]
    },
    "registry_with_stale_entry": {
      "seed_method": "public_api",
      "description": "Registry contains pid 1234 at old start_time 1730000000 while injected process_start_time returns 1730000099.",
      "records": [
        {
          "pid": 1234,
          "registered_start_time": 1730000000,
          "observed_start_time": 1730000099,
          "agent_id": "rust-implementer",
          "env_flag": "<unset>"
        }
      ]
    },
    "no_registry_env_set": {
      "seed_method": "public_api",
      "description": "No Registry reference is supplied and injected env lookup returns BUT_AUTHZ_ALLOW_ENV_HANDLE=1 plus BUT_AGENT_HANDLE=dev.",
      "records": [
        {
          "registry": null
        },
        {
          "env": {
            "BUT_AUTHZ_ALLOW_ENV_HANDLE": "1",
            "BUT_AGENT_HANDLE": "dev"
          }
        },
        {
          "gov_config": {
            "principal_id": "dev",
            "permissions": [
              "contents:read"
            ]
          }
        }
      ]
    }
  }
}
-->
