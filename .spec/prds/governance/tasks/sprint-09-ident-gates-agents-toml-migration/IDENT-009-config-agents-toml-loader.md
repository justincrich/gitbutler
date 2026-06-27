# IDENT-009 — `crates/but-authz/src/config.rs` — `AGENTS_PATH` + `AgentWire`/`AgentsWire` + dual-format `governance_present` + prefer-`agents.toml` loader with deprecation warning

**Sprint:** [Sprint 09](./SPRINT.md) · **Agent:** `rust-implementer` · **Estimate:** 240 min · **Type:** FEATURE · **Status:** READY · **Proposed By:** rust-planner

## Background

The IDENT initiative (v1.4.0) renames `.gitbutler/permissions.toml` → `.gitbutler/agents.toml` (`[[principal]]` → `[[agent]]`). The Rust domain types `Principal`/`PrincipalId` stay; only the wire format and file name change. During a one-release migration window, `load_governance_config` reads **both** files (prefer `agents.toml`, log a one-line deprecation warning when only `permissions.toml` is present). `governance_present` returns true if EITHER file is committed at the target ref.

**Why it matters.** Sprint 08 shipped the runtime identity layer but left `load_governance_config` reading only `permissions.toml`. Sprint 09's first task is to teach the loader the new format so the migration verb (IDENT-011) and the gate callsite swap (IDENT-010) have a config source that recognizes the renamed file.

**Current state.** `crates/but-authz/src/config.rs:8` hard-codes `PERMISSIONS_PATH = ".gitbutler/permissions.toml"`. `PermissionsWire`/`PrincipalWire` (config.rs:412-445) are the only wire types. `governance_present` (config.rs:53-68) checks only `PERMISSIONS_PATH || GATES_PATH`. `load_governance_config_inner` (config.rs:267-283) reads exactly two blobs.

**Desired state.** Loader recognizes both files; prefers `agents.toml`; emits one deprecation warning line on legacy-only state; `GovConfig` shape is byte-equivalent across formats.

## Critical Constraints

- **MUST** define `AGENTS_PATH = ".gitbutler/agents.toml"` as a const alongside `PERMISSIONS_PATH`.
- **MUST** add `AgentWire { id, permissions, role?, groups }` and `AgentsWire { agent: Vec<AgentWire>, group: Vec<GroupWire> }` mirroring `PrincipalWire`/`PermissionsWire` field-for-field (`serde deny_unknown_fields`).
- **MUST** keep both parse paths sharing one `normalize_permissions` path (structural conversion `AgentsWire → PermissionsWire`, NOT a forked `normalize_agents`) so byte-equivalent round-trip is structurally guaranteed.
- **MUST** have `governance_present` return true if `agents.toml` OR `permissions.toml` OR `gates.toml` is present at the target ref.
- **MUST** have `load_governance_config`, when both files are committed, parse `agents.toml` and ignore `permissions.toml`.
- **MUST** have `load_governance_config`, when ONLY `permissions.toml` is committed, parse it AND emit exactly one deprecation warning line naming `permissions.toml` + the remediation `run: but agent migrate`.
- **NEVER** rename the `Principal`/`PrincipalId` domain types or touch the 80+ `PrincipalId::new` call sites.
- **NEVER** remove or weaken the `permissions.toml` parse path during the migration window.
- **NEVER** edit `lib.rs` re-exports (IDENT-016 owns `agents_path()` export) or `authorize.rs` (IDENT-010 owns the resolver swap).
- **NEVER** consult the working tree for `agents.toml` — ref-pin contract preserved (`gix` target-ref blob read only).
- **STRICTLY** emit the deprecation warning at most once per `load_governance_config` call (not per principal).
- **NEVER** use `std::env::temp_dir().join(format!(...))` in tests — use `but_testsupport::writable_scenario`.

## Specification

**Objective:** Extend `crates/but-authz/src/config.rs` so the loader recognizes the new `agents.toml` format (`[[agent]]` tables) alongside the legacy `permissions.toml`, preferring `agents.toml` when both are committed and emitting a one-line deprecation warning on the legacy-only path.

**Success state:** `cargo test -p but-authz --test config` passes with new `agents.toml` parse, dual-format prefer-agents, and deprecation-warning cases. `governance_present` returns true for either file. `GovConfig` loaded from `agents.toml` is `PartialEq` to `GovConfig` loaded from the byte-equivalent `permissions.toml`.

## Acceptance Criteria

**AC-1 (PRIMARY)** — GIVEN a repo with `.gitbutler/agents.toml` committed at `refs/heads/main` containing `[[agent]]` blocks (dev=contents:write, ro=contents:read, release-bot role=maintain) plus gates.toml WHEN `load_governance_config(repo, "refs/heads/main")` THEN `Ok(GovConfig)` whose dev principal holds `ContentsWrite`, ro excludes it, release-bot desugars to the maintain role set, and main is protected — identical to the permissions.toml equivalent.
- **Verify:** `cargo test -p but-authz --test config -- agents_toml_parses_same_config`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-authz · **FLOW_REF:** UC-IDENT-01
- **Scenario:** `start_ref=seeded_agents_toml_equivalent`; `must_observe` = [dev contains `Authority::ContentsWrite`, release-bot contains `Authority::Merge`, main protected == true]; `must_not_observe` = [`ConfigError`, empty principals map, ro containing `ContentsWrite`]; `negative_control.would_fail_if` = [AgentWire serde struct missing/misnamed, `[[agent]]` not wired into `normalize_permissions`, loader hard-codes `PERMISSIONS_PATH`]; `evidence` = stdout.

**AC-2** — GIVEN three repos (agents.toml-only, permissions.toml-only, neither) WHEN `governance_present(repo, "refs/heads/main")` THEN agents-only → `Ok(true)`; permissions-only → `Ok(true)`; neither on a resolvable commit → `Ok(false)`.
- **Verify:** `cargo test -p but-authz --test config -- governance_present_agents_or_permissions`
- **Scenario:** `must_observe` = [`Ok(true)` for both file-present cases]; `must_not_observe` = [`Ok(false)` for present cases].

**AC-3** — GIVEN `repo_with_both_files` (agents.toml grants dev=contents:write, permissions.toml grants dev=contents:read — divergent) WHEN `load_governance_config` THEN the returned `GovConfig` reflects agents.toml (dev holds `ContentsWrite`); permissions.toml is ignored.
- **Verify:** `cargo test -p but-authz --test config -- both_files_prefers_agents_toml`
- **Scenario:** `must_observe` = [dev contains `Authority::ContentsWrite` (the agents.toml grant)]; `must_not_observe` = [dev holding only contents:read, ConfigError].

**AC-4** — GIVEN `repo_permissions_only` (no agents.toml committed) WHEN `load_governance_config` THEN the call succeeds AND emits exactly one deprecation warning line naming `permissions.toml` + the `but agent migrate` remediation.
- **Verify:** `cargo test -p but-authz --test config -- permissions_only_deprecation_warning`
- **Scenario:** `must_observe` = [exactly one warning line containing `permissions.toml` AND `but agent migrate`, `Ok(GovConfig)` with dev=contents:write]; `must_not_observe` = [zero warning lines, >1 warning lines, ConfigError].

**AC-5** — GIVEN two repos (one permissions.toml, one byte-equivalent agents.toml, same gates.toml) WHEN `load_governance_config` on both THEN the two `GovConfig` values are `PartialEq` equal.
- **Verify:** `cargo test -p but-authz --test config -- byte_equivalent_across_formats`
- **Scenario:** `must_observe` = [`cfg_p == cfg_a`]; `must_not_observe` = [`cfg_p != cfg_a`].

## Test Criteria

| ID | Boolean | Maps to AC |
|----|---------|------------|
| TC-1 | `load_governance_config` on a committed `agents.toml` returns `Ok` with dev=contents:write is true | AC-1 |
| TC-2 | `governance_present` returns `Ok(true)` for agents.toml-only AND permissions.toml-only repos is true | AC-2 |
| TC-3 | When both files are committed with divergent dev grants, the loaded dev authorities match agents.toml (`ContentsWrite`) not permissions.toml is true | AC-3 |
| TC-4 | `load_governance_config` on a permissions.toml-only repo emits exactly one deprecation warning naming the remediation is true | AC-4 |
| TC-5 | `GovConfig` from byte-equivalent permissions.toml == `GovConfig` from agents.toml is true | AC-5 |

## Reading List

1. `crates/but-authz/src/config.rs:1-72` — `PERMISSIONS_PATH`/`GATES_PATH` consts, `permissions_path()`, `governance_present()` — pattern to mirror for `AGENTS_PATH`/`agents_path()` and dual-file recognition.
2. `crates/but-authz/src/config.rs:267-310` — `load_governance_config_inner` + `read_config_blob` — the gix target-ref blob read; agents.toml uses the same path.
3. `crates/but-authz/src/config.rs:409-445` — `PermissionsWire`/`PrincipalWire`/`GroupWire` — mirror field-for-field as `AgentsWire`/`AgentWire`.
4. `crates/but-authz/src/lib.rs:1-18` — current re-exports — IDENT-016 owns adding `agents_path` here; this task MUST NOT edit lib.rs.
5. `crates/but-authz/tests/config.rs:195-230` — `governed_repo()` helper — copy this seed pattern for the agents.toml fixtures.

## Guardrails

**WRITE-ALLOWED:**
- `crates/but-authz/src/config.rs` (MODIFY — add `AGENTS_PATH` const, `agents_path()` fn, `AgentWire`/`AgentsWire` structs, dual-file `governance_present`, prefer-agents `load_governance_config` + deprecation warning)
- `crates/but-authz/tests/config.rs` (IDENT-015 EXTENDS this file later; IDENT-009 may add its own new test fns but must not rewrite existing fns — coordinate via the EXTEND contract)

**WRITE-PROHIBITED:**
- `crates/but-authz/src/lib.rs` — IDENT-016 owns the `agents_path()` re-export
- `crates/but-authz/src/authorize.rs` — IDENT-010 owns the resolver swap
- `crates/but-authz/src/registry.rs` and `src/process.rs` — Sprint 08 owns these
- Any file under `crates/but-api/` or `crates/but/` — IDENT-010/011 own those surfaces

## Code Pattern

**Reference:** `crates/but-authz/src/config.rs:267-283` (`load_governance_config_inner` — extend with a prefer-agents branch).
**Source:** `crates/but-authz/src/config.rs:412-445` (`PermissionsWire`/`PrincipalWire` — mirror as `AgentsWire`/`AgentWire`).

**Design notes:**
- In `load_governance_config_inner`: read agents.toml blob via `read_config_blob_optional` (new sibling returning `Option<String>`, soft-miss distinct from the hard-miss in `read_config_blob`). If agents blob present → parse `AgentsWire`. Else read permissions blob → parse `PermissionsWire` AND emit the one-line deprecation warning. gates.toml path unchanged.
- `AgentWire`/`AgentsWire` deserialize from `[[agent]]` tables; reuse `normalize_permissions` by converting `AgentsWire → PermissionsWire` (map `agent → principal`). Prefer the cheap structural conversion over a one-use trait (`crates/AGENTS.md`: avoid speculative abstractions).
- Deprecation warning: `tracing::warn!` if but-authz already depends on tracing; else a single `eprintln!` to stderr (UC-IDENT-01 says "one-line warning"). Check `Cargo.toml`.

**Anti-pattern:** Do NOT fork `normalize_permissions` into `normalize_agents` — that breaks the byte-equivalent round-trip guarantee (AC-5). Do NOT emit the deprecation warning from inside `normalize_permissions` (it would fire per-principal).

## Agent Instructions

TDD RED→GREEN→REFACTOR per AC:
1. **RED AC-1:** Write `tests/config.rs::agents_toml_parses_same_config` — seed agents.toml via `but_testsupport::writable_scenario` + `invoke_bash`, call `load_governance_config`, assert dev holds ContentsWrite. Run `cargo test -p but-authz --test config -- agents_toml_parses_same_config` → fails (AgentWire doesn't exist).
2. **GREEN:** Add `AGENTS_PATH` const + `AgentWire`/`AgentsWire` structs + the prefer-agents branch in `load_governance_config_inner`. Add `read_config_blob_optional` for the soft-miss probe.
3. **REFACTOR:** Extract the shared `normalize_permissions` path; verify byte-equivalent round-trip (AC-5).
4. Repeat for AC-2 through AC-5.
5. Run `cargo fmt`, `cargo clippy -p but-authz --all-targets -- -D warnings`, `cargo test -p but-authz --test config`.
6. Commit via `but commit` (governed path — the implementer is registered as `rust-implementer` by `but-run-sprint`).

## Orchestrator Verification Protocol

The orchestrator (`/but-run-sprint`) verifies this task by:
1. Running `cargo test -p but-authz --test config` and asserting exit 0.
2. Running `cargo check -p but-authz --all-targets` and asserting no errors.
3. Confirming `crates/but-authz/src/config.rs` defines `AGENTS_PATH`, `AgentWire`, `AgentsWire`.
4. Confirming no edits to `lib.rs`, `authorize.rs`, `registry.rs`, or any `but-api`/`but` file.

## Agent Assignment

**Agent:** `rust-implementer` — owns the `but-authz` config loader (single source of truth for governance file paths). Pure Rust + gix blob reads + toml serde, the crate's home turf.
**Pairing:** none. IDENT-010 (resolver swap) consumes the loader's prefer-agents path; IDENT-016 (re-exports) consumes `agents_path()`.

## Evidence Gates

- `cargo test -p but-authz --test config` exit 0 (RED→GREEN proof)
- `crates/but-authz/src/config.rs` defines `AGENTS_PATH` + `AgentWire` + `AgentsWire` + dual-format `governance_present` + prefer-agents `load_governance_config`
- No edits to `lib.rs`, `authorize.rs`, `registry.rs`, `process.rs`, `but-api/**`, or `but/**`

## Review Criteria

- `AgentWire` mirrors `PrincipalWire` field-for-field (id, permissions, role, groups) — no dropped/renamed field.
- `governance_present` ORs in `AGENTS_PATH` alongside `PERMISSIONS_PATH` and `GATES_PATH`.
- The prefer-agents branch probes `agents.toml` first; the permissions branch emits the deprecation warning exactly once per load.
- `normalize_permissions` is shared, not forked — byte-equivalent round-trip is structural.
- All assertions explained via the message arg (`assert!(cfg_a == cfg_p, "agents.toml and permissions.toml parse to byte-equivalent GovConfig — only the table header differs")`).

## Dependencies

- **Depends on:** none (loader changes are self-contained; consumes only existing `gix` blob-read path).
- **Blocks:** IDENT-010 (callsites need the prefer-agents loader), IDENT-011 (migrate verb writes agents.toml that the loader must recognize), IDENT-013 (round-trip test consumes the dual-format loader), IDENT-015 (test extension consumes the source change), IDENT-016 (re-exports `agents_path`).

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "IDENT-009",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "seeded_permissions_toml": {
      "description": "governed repo with .gitbutler/permissions.toml (dev=contents:write, ro=contents:read, release-bot role=maintain) + gates.toml (main protected), committed at refs/heads/main",
      "seed_method": "public_api",
      "records": [
        "writable_scenario + invoke_bash"
      ]
    },
    "seeded_agents_toml_equivalent": {
      "description": "same GovConfig, .gitbutler/agents.toml with [[agent]] blocks mirroring the [[principal]] blocks",
      "seed_method": "public_api",
      "records": [
        "invoke_bash writes agents.toml + commits"
      ]
    },
    "repo_with_both_files": {
      "description": "both files committed with divergent dev grants (agents=contents:write, permissions=contents:read)",
      "seed_method": "public_api",
      "records": [
        "invoke_bash commits both"
      ]
    },
    "repo_permissions_only": {
      "description": "ONLY .gitbutler/permissions.toml committed (no agents.toml)",
      "seed_method": "public_api",
      "records": [
        "governed_repo() shape"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN agents.toml committed ([[agent]] dev=contents:write, ro=contents:read, release-bot role=maintain) + gates.toml WHEN load_governance_config THEN Ok(GovConfig) with dev=ContentsWrite, ro excludes it, release-bot=maintain role, main protected",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "verify": "cargo test -p but-authz --test config -- agents_toml_parses_same_config",
      "maps_to_ac": null,
      "flow_ref": "UC-IDENT-01",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-authz",
        "start_ref": "seeded_agents_toml_equivalent",
        "must_observe": [
          "dev contains Authority::ContentsWrite",
          "release-bot contains Authority::Merge",
          "main protected == true"
        ],
        "must_not_observe": [
          "ConfigError",
          "empty principals",
          "ro containing ContentsWrite"
        ],
        "negative_control": {
          "would_fail_if": [
            "stubbed AgentWire serde returns empty principals",
            "[[agent]] parsing omitted from normalize_permissions",
            "loader hard-codes `.gitbutler/permissions.toml`"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_agents_toml_equivalent",
            "action": {
              "actor": "test_harness",
              "steps": [
                "load_governance_config",
                "assert dev/ro/release-bot/main"
              ]
            },
            "end_state": {
              "must_observe": [
                "principal `dev` has exactly `Authority::ContentsWrite`",
                "principal `release-bot` resolves role `maintain` with `Authority::Merge`",
                "gate target `refs/heads/main` has `protected == true`"
              ],
              "must_not_observe": [
                "`ConfigError`",
                "empty principals list",
                "principal `ro` has `Authority::ContentsWrite`"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "GIVEN agents.toml-only, permissions.toml-only, neither WHEN governance_present THEN present→Ok(true), absent (resolvable)→Ok(false)",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "verify": "cargo test -p but-authz --test config -- governance_present_agents_or_permissions",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-authz",
        "must_observe": [
          "Ok(true) for agents-only",
          "Ok(true) for permissions-only"
        ],
        "must_not_observe": [
          "Ok(false) for present cases"
        ],
        "negative_control": {
          "would_fail_if": [
            "governance_present omits `.gitbutler/agents.toml` lookup",
            "AGENTS_PATH wrong constant points at absent file"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_agents_toml_equivalent",
            "action": {
              "actor": "test_harness",
              "steps": [
                "governance_present"
              ]
            },
            "end_state": {
              "must_observe": [
                "agents-only governance_present returns `Ok(true)`"
              ],
              "must_not_observe": [
                "agents-only returns `Ok(false)`",
                "empty governance path returns true"
              ]
            }
          },
          {
            "start_ref": "repo_permissions_only",
            "action": {
              "actor": "test_harness",
              "steps": [
                "governance_present"
              ]
            },
            "end_state": {
              "must_observe": [
                "permissions-only governance_present returns `Ok(true)`"
              ],
              "must_not_observe": [
                "permissions-only returns `Ok(false)`",
                "empty governance path returns true"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "GIVEN both files committed with divergent dev grants WHEN load_governance_config THEN dev authorities match agents.toml (ContentsWrite)",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "verify": "cargo test -p but-authz --test config -- both_files_prefers_agents_toml",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-authz",
        "start_ref": "repo_with_both_files",
        "must_observe": [
          "dev contains Authority::ContentsWrite"
        ],
        "must_not_observe": [
          "dev holding only contents:read",
          "ConfigError"
        ],
        "negative_control": {
          "would_fail_if": [
            "loader reads only `.gitbutler/permissions.toml` and omits agents preference",
            "both-file branch left as stub/no-op"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "repo_with_both_files",
            "action": {
              "actor": "test_harness",
              "steps": [
                "load_governance_config",
                "inspect dev"
              ]
            },
            "end_state": {
              "must_observe": [
                "principal `dev` has exactly `Authority::ContentsWrite` from `.gitbutler/agents.toml`"
              ],
              "must_not_observe": [
                "principal `dev` has only `contents:read`",
                "empty principals list",
                "`ConfigError`"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "description": "GIVEN permissions.toml-only repo WHEN load_governance_config THEN Ok AND exactly one deprecation warning naming permissions.toml + but agent migrate remediation",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "verify": "cargo test -p but-authz --test config -- permissions_only_deprecation_warning",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-authz",
        "start_ref": "repo_permissions_only",
        "must_observe": [
          "exactly one warning line containing permissions.toml and but agent migrate",
          "Ok(GovConfig)"
        ],
        "must_not_observe": [
          "zero warning lines",
          "more than one warning line",
          "ConfigError"
        ],
        "negative_control": {
          "would_fail_if": [
            "no warning emitted",
            "warning omits remediation",
            "warning fires per-principal"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "repo_permissions_only",
            "action": {
              "actor": "test_harness",
              "steps": [
                "capture warn output",
                "load_governance_config",
                "count warnings == 1",
                "assert text"
              ]
            },
            "end_state": {
              "must_observe": [
                "warning count == 1",
                "warning contains `.gitbutler/permissions.toml` and `but agent migrate`"
              ],
              "must_not_observe": [
                "warning count == 0",
                "warning count > 1",
                "empty warning text",
                "`ConfigError`"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "description": "GIVEN byte-equivalent permissions.toml and agents.toml repos WHEN load_governance_config on each THEN the two GovConfig are PartialEq equal",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "verify": "cargo test -p but-authz --test config -- byte_equivalent_across_formats",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-authz",
        "start_ref": "seeded_permissions_toml",
        "must_observe": [
          "cfg_p == cfg_a"
        ],
        "must_not_observe": [
          "cfg_p != cfg_a"
        ],
        "negative_control": {
          "would_fail_if": [
            "AgentWire stub drops `role = \"maintain\"` field",
            "normalization path omitted for one wire type"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_permissions_toml",
            "action": {
              "actor": "test_harness",
              "steps": [
                "load cfg_p",
                "load cfg_a",
                "assert PartialEq"
              ]
            },
            "end_state": {
              "must_observe": [
                "cfg_p == cfg_a"
              ],
              "must_not_observe": [
                "`cfg_p != cfg_a`",
                "empty authorities in either config"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "agents.toml parses into GovConfig with dev=ContentsWrite",
      "verify": "cargo test -p but-authz --test config -- agents_toml_parses_same_config",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "governance_present true for agents.toml-only and permissions.toml-only",
      "verify": "cargo test -p but-authz --test config -- governance_present_agents_or_permissions",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "both files present → agents.toml wins",
      "verify": "cargo test -p but-authz --test config -- both_files_prefers_agents_toml",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "permissions-only emits one deprecation warning with remediation",
      "verify": "cargo test -p but-authz --test config -- permissions_only_deprecation_warning",
      "maps_to_ac": "AC-4"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "byte-equivalent GovConfig across formats",
      "verify": "cargo test -p but-authz --test config -- byte_equivalent_across_formats",
      "maps_to_ac": "AC-5"
    }
  ]
}
-->
