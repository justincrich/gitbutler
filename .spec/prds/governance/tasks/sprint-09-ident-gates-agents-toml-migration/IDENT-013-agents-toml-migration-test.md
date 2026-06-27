# IDENT-013 — `crates/but-api/tests/agents_toml_migration.rs` — `permissions.toml` → `but agent migrate` → `agents.toml` byte-equivalent round-trip; legacy-only repo still authorizes

**Sprint:** [Sprint 09](./SPRINT.md) · **Agent:** `rust-implementer` · **Estimate:** 120 min · **Type:** FEATURE (TEST) · **Status:** READY · **Proposed By:** rust-planner

## Background

UC-IDENT-01 requires that `but agent migrate` produces a byte-equivalent `agents.toml` that loads to the same `GovConfig`, AND that a `permissions.toml`-only repo continues to authorize governed actions during the migration window. This file is the regression net for both properties at the `but-api` tier.

**Why it matters.** Sprint 10's deny-default flip depends on this round-trip proof (the migration must be lossless before the env-escape hatch can be removed). The legacy-only-authorizes property is what makes the migration window safe — no operator is forced to migrate before Sprint 10 lands.

**Current state.** `crates/but-api/tests/agents_toml_migration.rs` does not exist. The migration verb (IDENT-011) and the dual-format loader (IDENT-009) must both land first. The test drives the migration transform (the function the `but agent migrate` verb wraps) OR invokes the CLI as a subprocess — `but-api` convention is to call the engine's public Rust API directly.

**Desired state.** 2 integration tests: (1) round-trip is `PartialEq`-equivalent at the `GovConfig` layer; (2) legacy-only repo + `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` + `BUT_AGENT_HANDLE=dev` → `enforce_commit_gate_for_target` `Ok`, and with the flag unset → `Err(Denial{code:perm.denied})`.

## Critical Constraints

- **MUST** use `but_testsupport::writable_scenario("agents-toml-migration")` + `invoke_bash` for all repo seeding — never `std::env::temp_dir().join(format!(...))`.
- **MUST** assert `GovConfig` equality via `PartialEq` (`cfg_before == cfg_after`) — NOT via string comparison of the TOML blobs (byte ordering of TOML maps is not guaranteed; the domain equality is `GovConfig`).
- **MUST** set `BUT_AUTHZ_ALLOW_ENV_HANDLE` and `BUT_AGENT_HANDLE` via `temp_env::with_var`/`with_vars` (mirrors `crates/but-api/tests/commit_gate.rs:14`) — never `std::env::set_var` in a `#[test]` (process-wide leak).
- **MUST** assert the downcast path: `enforce_commit_gate_for_target` errors must downcast to `but_authz::Denial` for the negative-control case (mirrors `commit_gate.rs:36-47`).
- **NEVER** shell out to the `but` binary with `std::process::Command::new("but")` — `but-api` tests call the engine's public Rust API directly.
- **NEVER** compare the raw `permissions.toml` and `agents.toml` file bytes for equality — they are NOT byte-identical (the block header `[[principal]]` → `[[agent]]` differs); the equality guarantee is at the `GovConfig` layer.
- **NEVER** mutate process-global env vars without `temp_env` scoping.
- **STRICTLY** commit the agents-format state as `git add agents.toml && git rm permissions.toml && git commit` — a migration that leaves BOTH files committed is a different scenario (covered by IDENT-015 AC-2 prefer-agents.toml); AC-1 here tests the END STATE of a completed migration.
- **STRICTLY** load `GovConfig` from two DIFFERENT commits (the permissions-format commit and the agents-format commit) — loading the same commit twice would be a tautology.

## Specification

**Objective:** Add `crates/but-api/tests/agents_toml_migration.rs` with 2 integration test fns: (1) `migrate_round_trip_is_byte_equivalent_gov_config` — seeds `permissions.toml`, runs the migration transform, commits `agents.toml` + removes `permissions.toml`, loads `GovConfig` from both commits, asserts `PartialEq`; (2) `legacy_permissions_only_repo_authorizes_via_env_fallback` — seeds only `permissions.toml`, sets `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` + `BUT_AGENT_HANDLE=dev`, calls `enforce_commit_gate_for_target`, asserts `Ok`, then unsets the flag and asserts the gate now denies with `perm.denied` (negative control).

**Success state:** `cargo test -p but-api --test agents_toml_migration` passes. The round-trip test proves a repo migrated via `but agent migrate` loads to an identical `GovConfig`. The legacy-only test proves the Sprint 09 migration window does not break existing governed repos that have not yet run the migration.

## Acceptance Criteria

**AC-1 (PRIMARY)** — migrate round-trip is `GovConfig`-equivalent: GIVEN a governed repo with `.gitbutler/permissions.toml` (`[[principal]]` id="dev" permissions=["contents:write"], `[[principal]]` id="ro" permissions=["contents:read"]) + gates.toml committed at `refs/heads/main` WHEN the migration transform runs against the working tree, then `git add .gitbutler/agents.toml && git rm .gitbutler/permissions.toml && git commit` lands the agents-format state, and `load_governance_config` is called once on the permissions-format commit and once on the agents-format commit THEN the two resulting `GovConfig` values are equal by `PartialEq` (`cfg_before == cfg_after`); both resolve dev → contents:write and ro → contents:read identically.
- **Verify:** `cargo test -p but-api --test agents_toml_migration migrate_round_trip_is_byte_equivalent_gov_config`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-api · **FLOW_REF:** UC-IDENT-01
- **Scenario:** `start_ref=permissions_format_committed_at_main`; `must_observe` = [`load_governance_config` on permissions-format succeeds, after migration + commit `load_governance_config` on agents-format succeeds, `cfg_before == cfg_after`, both grant dev=contents:write + ro=contents:read]; `must_not_observe` = [`cfg_before != cfg_after`, any `ConfigError`, migration transform failing or leaving malformed agents.toml]; `negative_control.would_fail_if` = [migration drops/renames a field, loader reads working tree instead of target ref, `[[principal]]`→`[[agent]]` rename changed more than the block header].

**AC-2** — legacy-only repo authorizes via env fallback: GIVEN a governed repo with ONLY `.gitbutler/permissions.toml` committed (no agents.toml, no runtime registry) at `refs/heads/main`, and a feat branch (unprotected) as the commit target WHEN `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` and `BUT_AGENT_HANDLE=dev` are set (via `temp_env::with_vars`), and `enforce_commit_gate_for_target` is called for the feat target THEN the gate returns `Ok(())` — the legacy permissions.toml-only repo still authorizes the dev principal during the Sprint 09 migration window via the env-fallback escape hatch; unsetting `BUT_AUTHZ_ALLOW_ENV_HANDLE` makes the same call return `Err` downcasting to `but_authz::Denial` with code == "perm.denied".
- **Verify:** `cargo test -p but-api --test agents_toml_migration legacy_permissions_only_repo_authorizes_via_env_fallback`
- **Scenario:** `start_ref=legacy_permissions_only_committed`; `must_observe` = [Ok with flag set, `perm.denied` `Denial` with flag unset]; `must_not_observe` = [Ok when flag unset, any panic or non-Denial error type]; `negative_control.would_fail_if` = [IDENT-010 flipped env-fallback default to deny-on (Sprint 10 behavior), legacy permissions.toml path no longer read, `resolve_principal_with_registry` ignores `BUT_AUTHZ_ALLOW_ENV_HANDLE`].

## Test Criteria

| ID | Boolean | Maps to AC |
|----|---------|------------|
| TC-1 | `load_governance_config` on the permissions-format commit and the agents-format commit (post-migration) returns `PartialEq`-equal `GovConfig` values is true | AC-1 |
| TC-2 | Both `cfg_before` and `cfg_after` resolve dev → contents:write and ro → contents:read is true | AC-1 |
| TC-3 | On a legacy-only repo with `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` + `BUT_AGENT_HANDLE=dev`, `enforce_commit_gate_for_target` returns `Ok(())` is true | AC-2 |
| TC-4 | On the same legacy-only repo with `BUT_AUTHZ_ALLOW_ENV_HANDLE` unset, `enforce_commit_gate_for_target` returns `Err` downcasting to `but_authz::Denial` with code == "perm.denied" is true | AC-2 |

## Reading List

1. `crates/but-authz/tests/config.rs:195-230` — `governed_repo()` helper using `writable_scenario` + `invoke_bash` (mirror this fixture pattern exactly).
2. `crates/but-api/tests/commit_gate.rs:7-59` — `temp_env::with_var("BUT_AGENT_HANDLE", ...)` + downcast to `but_authz::Denial` pattern.
3. `crates/but-api/src/commit/gate.rs:55-78` — `enforce_commit_gate_for_target` signature + the `resolve_principal` swap site IDENT-010 changes.
4. `crates/but-authz/src/config.rs:33-38` — `load_governance_config(repo, target_ref)` (IDENT-009 extends this to read `agents.toml` OR `permissions.toml`).
5. `.spec/prds/governance/12-uc-agent-identity.md:30-34` — UC-IDENT-01 acceptance criteria for the migration window + byte-equivalent round-trip.

## Guardrails

**WRITE-ALLOWED:**
- `crates/but-api/tests/agents_toml_migration.rs` (NEW)

**WRITE-PROHIBITED:**
- `crates/but-authz/src/**` (IDENT-009 owns `config.rs`; IDENT-016 owns `lib.rs`/`authorize.rs`)
- `crates/but/src/**` (IDENT-011 owns `command/agent.rs`)
- `crates/but-api/src/**` (IDENT-010 owns the gate callsite swap)
- `crates/but/tests/but/command/agent.rs` (IDENT-014 owns the CLI snapshots)

## Code Pattern

**Reference:** `crates/but-authz/tests/config.rs:195` (`governed_repo()` `writable_scenario` + `invoke_bash` fixture pattern).
**Source:** `crates/but-api/tests/commit_gate.rs:14` (`temp_env::with_var` for `BUT_AGENT_HANDLE`); `:36-47` (`err.downcast_ref::<but_authz::Denial>()` + `denial.code == "perm.denied"`).

**Design notes:**
- Cross-task contract: IDENT-011's `but agent migrate` verb MUST delegate to a reusable migration function. If it inlines the transform, this test reconstructs the transform from `but-authz` public wire types (`PermissionsWire → AgentsWire`). The observable property (byte-equivalent `GovConfig`) is identical either way.
- AC-2 depends on IDENT-010 having swapped `enforce_commit_gate_for_target` to `resolve_principal_with_registry` AND on Sprint 09's migration-friendly default (env fallback allowed when flag set). If IDENT-010 has not landed, AC-2's flag-set path will still pass against the legacy `resolve_principal_from_env` — the negative-control (flag-unset → deny) is the part that proves IDENT-010's swap occurred.

**Anti-pattern:** Do NOT shell out to the `but` binary from a `but-api` integration test (convention is to call the public Rust API directly); do NOT compare raw TOML file bytes (block headers differ by design); do NOT use `std::env::set_var` (process-wide leak across `#[test]` cases).

## Agent Instructions

TDD RED→GREEN per AC:
1. **RED AC-1:** Write `tests/agents_toml_migration.rs::migrate_round_trip_is_byte_equivalent_gov_config` — seed permissions.toml, run migration transform, commit agents-format, load both, assert `PartialEq`. Run → fails (migration transform doesn't exist until IDENT-011).
2. **GREEN:** (assuming IDENT-011 landed) the test should pass.
3. **RED AC-2:** Write `legacy_permissions_only_repo_authorizes_via_env_fallback` — seed permissions-only, `temp_env::with_vars(flag+handle)` → `Ok`, unset flag → `Err(Denial perm.denied)`. Run → fails until IDENT-010 lands.
4. **GREEN:** (assuming IDENT-010 landed) the test passes.
5. Run `cargo fmt`, `cargo clippy -p but-api --all-targets -- -D warnings`, `cargo test -p but-api --test agents_toml_migration`.
6. Commit via `but commit`.

## Orchestrator Verification Protocol

1. `cargo test -p but-api --test agents_toml_migration` exit 0 (2 tests pass).
2. `cargo check -p but-api --all-targets` clean.
3. The round-trip test loads `GovConfig` from two different commits (not the same commit twice).

## Agent Assignment

**Agent:** `rust-implementer` — owns the migration round-trip regression at the `but-api` tier. Pure Rust test authoring against the engine's public API surface — no CLI/TUI/Svelte/Electron domain.
**Pairing:** none. Distinct from IDENT-012 (registry gates) and IDENT-014 (CLI snapshots).

## Evidence Gates

- `cargo test -p but-api --test agents_toml_migration` exit 0
- The round-trip test loads from two different commits
- The legacy-only test uses `temp_env::with_vars` (no `std::env::set_var`)

## Review Criteria

- `GovConfig` equality via `PartialEq`, NOT raw TOML byte comparison.
- Env mutations scoped via `temp_env` (no process-wide leak).
- Denial assertions downcast to `but_authz::Denial` and check `.code`.
- Both ACs use `but_testsupport::writable_scenario` (never `std::env::temp_dir().join(format!(...))`).
- Assertion messages explain why (`assert!(cfg_before == cfg_after, "migration must be byte-equivalent — [[principal]]→[[agent]] is the only wire-level change")`).

## Dependencies

- **Depends on:** IDENT-009 (dual-format loader), IDENT-010 (callsite swap), IDENT-011 (migrate verb).
- **Blocks:** Sprint 10 (deprecation hardening) — the round-trip proof is the prerequisite for flipping the env-fallback default.

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "IDENT-013",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "legacy_permissions_repo": {
      "description": "governed repo with .gitbutler/permissions.toml + gates.toml committed at refs/heads/main (the migration source state)",
      "seed_method": "public_api",
      "records": [
        "writable_scenario + invoke_bash commits permissions.toml + gates.toml"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN governed repo with permissions.toml committed at refs/heads/main WHEN migration transform runs, agents.toml is committed and permissions.toml removed, and load_governance_config is called on both commits THEN the two GovConfig values are PartialEq-equal",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-api --test agents_toml_migration migrate_round_trip_is_byte_equivalent_gov_config",
      "maps_to_ac": null,
      "flow_ref": "UC-IDENT-01",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "legacy_permissions_repo",
        "must_observe": [
          "cfg_before == cfg_after",
          "both loads succeed"
        ],
        "must_not_observe": [
          "cfg_before != cfg_after",
          "ConfigError"
        ],
        "negative_control": {
          "would_fail_if": [
            "migration stub omits a field from `.gitbutler/agents.toml`",
            "load_governance_config disconnected from committed ref and reads working tree",
            "header rewrite changes data beyond `[[principal]]` to `[[agent]]`"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "legacy_permissions_repo",
            "action": {
              "actor": "ci",
              "steps": [
                "load → cfg_before",
                "run migration",
                "git add agents.toml && git rm permissions.toml && commit",
                "load → cfg_after",
                "assert PartialEq"
              ]
            },
            "end_state": {
              "must_observe": [
                "cfg_before == cfg_after"
              ],
              "must_not_observe": [
                "`cfg_before != cfg_after`",
                "empty principals in `cfg_after`"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "GIVEN legacy-only repo with permissions.toml + no agents.toml WHEN BUT_AUTHZ_ALLOW_ENV_HANDLE=1 + BUT_AGENT_HANDLE=dev set THEN enforce_commit_gate_for_target returns Ok; with flag unset returns Err(Denial{code:perm.denied})",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-api --test agents_toml_migration legacy_permissions_only_repo_authorizes_via_env_fallback",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "legacy_permissions_repo",
        "must_observe": [
          "Ok with flag set",
          "perm.denied Denial with flag unset"
        ],
        "must_not_observe": [
          "denial with flag set",
          "Ok with flag unset"
        ],
        "negative_control": {
          "would_fail_if": [
            "env fallback omitted when `BUT_AUTHZ_ALLOW_ENV_HANDLE=1`",
            "legacy `.gitbutler/permissions.toml` loader removed/absent",
            "resolve_principal_with_registry disconnected from env fallback flag"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "legacy_permissions_repo",
            "action": {
              "actor": "ci",
              "steps": [
                "temp_env flag+handle → enforce_commit_gate_for_target Ok",
                "temp_env handle only → enforce_commit_gate_for_target Err(Denial perm.denied)"
              ]
            },
            "end_state": {
              "must_observe": [
                "with flag set result == `Ok(())`; with flag unset error code == `perm.denied`"
              ],
              "must_not_observe": [
                "with flag set error code == `perm.denied`",
                "with flag unset result == `Ok(())`",
                "empty env handle authorizes"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "cfg_before == cfg_after after migration round-trip",
      "verify": "cargo test -p but-api --test agents_toml_migration migrate_round_trip_is_byte_equivalent_gov_config",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "dev→contents:write and ro→contents:read hold in both configs",
      "verify": "cargo test -p but-api --test agents_toml_migration migrate_round_trip_is_byte_equivalent_gov_config",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "legacy-only + flag set → enforce_commit_gate_for_target Ok",
      "verify": "cargo test -p but-api --test agents_toml_migration legacy_permissions_only_repo_authorizes_via_env_fallback",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "legacy-only + flag unset → Err downcasts to Denial code perm.denied",
      "verify": "cargo test -p but-api --test agents_toml_migration legacy_permissions_only_repo_authorizes_via_env_fallback",
      "maps_to_ac": "AC-2"
    }
  ]
}
-->
