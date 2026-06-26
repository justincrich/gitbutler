# IDENT-015 — `crates/but-authz/tests/config.rs` — extend with `agents.toml` parse + both-formats-prefer-`agents.toml` + deprecation warning emission

**Sprint:** [Sprint 09](./SPRINT.md) · **Agent:** `rust-reviewer` · **Estimate:** 90 min · **Type:** FEATURE (TEST) · **Status:** READY · **Proposed By:** rust-planner

## Background

IDENT-009 modifies `crates/but-authz/src/config.rs` to add `AGENTS_PATH`, `AgentWire`/`AgentsWire`, dual-format `governance_present`, prefer-`agents.toml` loading, and the deprecation warning. This task EXTENDS `crates/but-authz/tests/config.rs` with 3 new `#[test]` fns covering those behaviors.

**Why it matters.** The `but-authz` config test file is the first-tier regression net for the loader. Sprint 10's deny-default flip relies on these tests to prove the loader still recognizes both formats and prefers `agents.toml`.

**Current state.** `crates/but-authz/tests/config.rs` exists with 5 fns (`config_loads_from_target_ref`, `config_ignores_working_tree_edit`, `config_malformed_fails_closed`, `config_role_entry_desugars`, `config_loads_from_target_not_head`) and the `governed_repo()` helper (lines 195-230). IDENT-009 owns the source change; this task owns the bulk of the test extensions.

**Desired state.** 3 new `#[test]` fns: `agents_toml_parses_same_config`, `both_files_prefers_agents_toml`, `permissions_only_deprecation_warning`.

## Critical Constraints

- **MUST** use `but_testsupport::writable_scenario` + `invoke_bash` for all seeding (mirror `governed_repo()` at `config.rs:195-230`) — never `std::env::temp_dir().join(format!(...))`.
- **MUST** assert `GovConfig` equality/inequality via `PartialEq` and `AuthoritySet::contains` — not via raw TOML string comparison.
- **MUST** make AC-2's two committed files DIVERGENT on the dev grant (e.g. `permissions.toml` grants dev `contents:read`, `agents.toml` grants dev `contents:write`) so the preference is observable — identical files would make the test a tautology.
- **MUST** assert EXACTLY ONE warning line for AC-3 (not zero, not many) — use a precise count, not a `.contains()` check.
- **NEVER** rewrite or rename any existing `#[test]` fn or helper in `config.rs` (EXTEND-only contract with prior sprints + IDENT-009).
- **NEVER** compare raw `permissions.toml` vs `agents.toml` file bytes (block headers differ by design; equality is at `GovConfig`).
- **NEVER** use `std::env::set_var` for any purpose (`config.rs` has no env-var dependency).
- **NEVER** weaken a malformed-config assertion to make a flaky case pass.
- **STRICTLY** the `agents.toml` fixture for AC-1 MUST use `[[agent]]` blocks (NOT `[[principal]]`) — this is the format under test; using `[[principal]]` would silently pass via the legacy path.
- **STRICTLY** the deprecation warning assertion MUST name BOTH `permissions.toml` AND the `but agent migrate` remediation — a warning that names only one is incomplete per UC-IDENT-01.

## Specification

**Objective:** Add 3 new `#[test]` fns to `crates/but-authz/tests/config.rs`: (1) `agents_toml_parses_same_config` — `agents.toml` loads to the same `GovConfig` as a byte-equivalent `permissions.toml`; (2) `both_files_prefers_agents_toml` — when both committed with divergent dev grants, dev's authorities match `agents.toml`; (3) `permissions_only_deprecation_warning` — a `permissions.toml`-only repo emits exactly one warning line naming `permissions.toml` + `but agent migrate`.

**Success state:** `cargo test -p but-authz --test config` passes with the 3 new fns green. No existing `config.rs` test is modified.

## Acceptance Criteria

**AC-1 (PRIMARY)** — `agents.toml` parses to same `GovConfig` shape: GIVEN a governed repo with `.gitbutler/agents.toml` (`[[agent]]` id="dev" permissions=["contents:write"], `[[agent]]` id="ro" permissions=["contents:read"], `[[agent]]` id="release-bot" role="maintain") + `gates.toml` committed at `refs/heads/main` — byte-equivalent in `GovConfig` terms to the `permissions.toml` fixture used by `config_loads_from_target_ref` WHEN `but_authz::load_governance_config` is called THEN it returns `Ok` with a `GovConfig` where dev holds `contents:write`, ro holds `contents:read`, release-bot holds the maintain role's authority set, and main is protected — the SAME `GovConfig` shape as the legacy `permissions.toml` path.
- **Verify:** `cargo test -p but-authz --test config agents_toml_parses_same_config`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-authz · **FLOW_REF:** UC-IDENT-01
- **Scenario:** `start_ref=agents_format_committed`; `must_observe` = [`load_governance_config` returns `Ok`, dev→contents:write, ro→contents:read, release-bot→`AuthoritySet::from_role("maintain")` (includes Merge + AdministrationRead, excludes AdministrationWrite), main protected == true]; `must_not_observe` = [`ConfigError`, dev missing contents:write, release-bot holding AdministrationWrite (would mean role desugaring differs between formats)]; `negative_control.would_fail_if` = [AgentWire does not deserialize `[[agent]]` block, field set differs between PrincipalWire and AgentWire, role sugar is not desugared identically].

**AC-2** — both-files prefer `agents.toml`: GIVEN a governed repo where BOTH `.gitbutler/permissions.toml` (`[[principal]]` id="dev" permissions=["contents:read"]) AND `.gitbutler/agents.toml` (`[[agent]]` id="dev" permissions=["contents:write"]) are committed at `refs/heads/main` — dev's grant DIVERGES between the two files WHEN `but_authz::load_governance_config` is called THEN the returned `GovConfig` grants dev `contents:write` (the `agents.toml` grant), NOT `contents:read` (the `permissions.toml` grant) — proving the loader prefers `agents.toml` when both are present.
- **Verify:** `cargo test -p but-authz --test config both_files_prefers_agents_toml`
- **Scenario:** `start_ref=both_files_committed_divergent_dev`; `must_observe` = [`Ok`, dev→contents:write (matches agents.toml), dev does NOT have contents:read-only]; `must_not_observe` = [dev→contents:read (would mean permissions.toml won), `ConfigError`, dev holding both grants unioned (would mean loader merges instead of prefers)].

**AC-3** — permissions-only emits deprecation warning: GIVEN a governed repo with ONLY `.gitbutler/permissions.toml` committed (no agents.toml) — the legacy unmigrated state WHEN `but_authz::load_governance_config` is called THEN it returns `Ok` (the legacy path still works) AND emits EXACTLY ONE deprecation warning line whose text names `permissions.toml` AND names `but agent migrate` (the remediation).
- **Verify:** `cargo test -p but-authz --test config permissions_only_deprecation_warning`
- **Scenario:** `start_ref=permissions_only_committed`; `must_observe` = [`Ok`, exactly one warning line emitted (count == 1), warning text contains "permissions.toml", warning text contains "but agent migrate"]; `must_not_observe` = [zero warning lines (loader did not detect legacy-only state), more than one warning line (per-call emission), warning that names permissions.toml but omits remediation, `ConfigError` (warning must not escalate during migration window)].

## Test Criteria

| ID | Boolean | Maps to AC |
|----|---------|------------|
| TC-1 | `load_governance_config` on an `agents.toml`-only repo returns `Ok` with dev→contents:write, ro→contents:read, release-bot==maintain role set, main protected is true | AC-1 |
| TC-2 | On a both-files-committed repo with divergent dev grants, `load_governance_config` grants dev the agents.toml grant (contents:write), not the permissions.toml grant (contents:read), is true | AC-2 |
| TC-3 | On a permissions-only repo, `load_governance_config` emits exactly one warning line naming `permissions.toml` AND `but agent migrate` is true | AC-3 |
| TC-4 | The permissions-only load still returns `Ok` (the warning is non-fatal during the migration window) is true | AC-3 |

## Reading List

1. `crates/but-authz/tests/config.rs:1-38` — `config_loads_from_target_ref` (the existing test shape to mirror for AC-1).
2. `crates/but-authz/tests/config.rs:195-230` — `governed_repo()` helper (reuse for the permissions-only fixture; add a sibling for agents-format).
3. `crates/but-authz/tests/config.rs:130-162` — `config_role_entry_desugars` (the role-sugar assertion pattern to mirror for release-bot in AC-1).
4. `crates/but-authz/src/config.rs:267-283` — `load_governance_config_inner` (the site IDENT-009 modifies to read `agents.toml` OR `permissions.toml` + emit the warning).
5. `.spec/prds/governance/12-uc-agent-identity.md:23-34` — UC-IDENT-01 acceptance criteria (`agents.toml` parse + prefer-`agents.toml` + deprecation warning).

## Guardrails

**WRITE-ALLOWED:**
- `crates/but-authz/tests/config.rs` (EXTEND — add 3 new `#[test]` fns + at most one new private helper; do NOT edit existing fns/helpers)

**WRITE-PROHIBITED:**
- `crates/but-authz/src/config.rs` (IDENT-009 owns the source)
- `crates/but-authz/src/lib.rs` (IDENT-016 owns the re-exports)
- `crates/but-authz/src/authorize.rs` (IDENT-016 owns doc-comments; IDENT-003/010 own the resolver)
- `crates/but-api/**` (IDENT-010/013 own but-api)
- `crates/but/**` (IDENT-011/014 own the CLI)
- Any existing `#[test]` fn or helper in `config.rs` (EXTEND-only contract)

## Code Pattern

**Reference:** `crates/but-authz/tests/config.rs:5-38` (`config_loads_from_target_ref` — the assertions + `println` evidence pattern).
**Source:** `crates/but-authz/tests/config.rs:195-230` (`governed_repo()` `writable_scenario` + `invoke_bash` fixture).

**Design notes:**
- EXTEND coordination with IDENT-009: IDENT-009 owns `config.rs` SOURCE and may add 1-2 of its own RED-phase test fns. IDENT-015 owns the bulk (the 3 named fns). If IDENT-009 already added one, IDENT-015 skips the duplicate — additive only, never conflicting.
- AC-3 warning-capture mechanism depends on IDENT-009's emission choice (`eprintln!` / `tracing` / returned `Warning` struct). The test matches IDENT-009's actual path; if IDENT-009 silently drops the warning, that's an IDENT-009 finding, not a blocker for IDENT-015's test (the test will fail RED until IDENT-009 emits).
- AC-2 divergence is essential: identical files make the preference unobservable. The fixture MUST commit different dev grants in each file.

**Anti-pattern:** Do NOT compare raw TOML bytes; do NOT make AC-2's two files identical (tautology); do NOT use `.contains()` for the AC-3 warning count (use an exact count assertion).

## Agent Instructions

TDD RED→GREEN per AC:
1. **RED AC-1:** Write `agents_toml_parses_same_config` — seed `agents.toml`-format repo, load, assert dev/ro/release-bot/main. Run → fails (AgentWire doesn't exist until IDENT-009).
2. **GREEN:** (assuming IDENT-009 landed) test passes.
3. Repeat for AC-2 (prefer-agents, divergent grants) and AC-3 (deprecation warning, exact count).
4. Run `cargo fmt`, `cargo clippy -p but-authz --all-targets -- -D warnings`, `cargo test -p but-authz --test config`.
5. Commit via `but commit`.

## Orchestrator Verification Protocol

1. `cargo test -p but-authz --test config` exit 0 (3 new fns pass; existing 5 fns still pass).
2. `cargo check -p but-authz --all-targets` clean.
3. No existing `config.rs` fn modified (diff is purely additive).

## Agent Assignment

**Agent:** `rust-reviewer` — test-extension task per the stub. Assertion authoring against IDENT-009's source changes, not new engine logic. IDENT-009 owns the `config.rs` SOURCE; IDENT-015 owns the bulk of the `config.rs` TEST extensions.
**Pairing:** none.

## Evidence Gates

- `cargo test -p but-authz --test config` exit 0 (3 new + 5 existing fns pass)
- AC-2's fixture commits DIVERGENT dev grants (not identical)
- AC-3's assertion is an exact count (== 1), not `.contains()`

## Review Criteria

- AC-1 fixture uses `[[agent]]` blocks (not `[[principal]]`).
- AC-2's two files carry divergent dev grants (preferences observable).
- AC-3 asserts EXACTLY ONE warning (precise count, not `.contains()`).
- No existing `config.rs` fn modified.
- `println!` evidence lines mirror `config_loads_from_target_ref` style.
- Assertion messages explain why (`assert!(warning_count == 1, "exactly one deprecation warning — per-load, not per-call")`).

## Dependencies

- **Depends on:** IDENT-009 (source change).
- **Blocks:** Sprint 10 (the deprecation-warning test is the regression net for Sprint 10's deny flip).

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "IDENT-015",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "agents_format_committed": {
      "description": "governed repo with .gitbutler/agents.toml ([[agent]] dev=contents:write, ro=contents:read, release-bot role=maintain) + gates.toml (main protected), committed at refs/heads/main",
      "seed_method": "public_api",
      "records": [
        "writable_scenario + invoke_bash"
      ]
    },
    "both_files_committed_divergent_dev": {
      "description": "both .gitbutler/permissions.toml (dev=contents:read) AND .gitbutler/agents.toml (dev=contents:write) committed at refs/heads/main — divergent dev grant",
      "seed_method": "public_api",
      "records": [
        "invoke_bash commits both files with divergent grants"
      ]
    },
    "permissions_only_committed": {
      "description": "governed repo with ONLY .gitbutler/permissions.toml committed (no agents.toml) — the legacy unmigrated state",
      "seed_method": "public_api",
      "records": [
        "governed_repo() helper shape"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN agents.toml committed WHEN load_governance_config THEN Ok with same GovConfig shape as legacy permissions.toml path (dev→contents:write, ro→contents:read, release-bot==maintain, main protected)",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "verify": "cargo test -p but-authz --test config agents_toml_parses_same_config",
      "maps_to_ac": null,
      "flow_ref": "UC-IDENT-01",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-authz",
        "start_ref": "agents_format_committed",
        "must_observe": [
          "Ok",
          "dev→contents:write",
          "ro→contents:read",
          "release-bot==maintain",
          "main protected"
        ],
        "must_not_observe": [
          "Err",
          "AdministrationWrite for release-bot"
        ],
        "negative_control": {
          "would_fail_if": [
            "AgentWire deserialize path omitted for `[[agent]]`",
            "PrincipalWire comparison uses static/stub field set",
            "role sugar resolver disconnected from `maintain`"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "agents_format_committed",
            "action": {
              "actor": "ci",
              "steps": [
                "load_governance_config",
                "assert dev/ro/release-bot/main"
              ]
            },
            "end_state": {
              "must_observe": [
                "load result == `Ok(GovConfig)` and principal count >= 3",
                "principal `dev` has `contents:write`",
                "principal `ro` has `contents:read`",
                "principal `release-bot` role == `maintain`",
                "gate target `refs/heads/main` protected == true"
              ],
              "must_not_observe": [
                "result is `Err`",
                "empty principals list",
                "release-bot has `AdministrationWrite`"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "GIVEN both files committed with divergent dev grants WHEN load_governance_config THEN dev matches agents.toml (contents:write), not permissions.toml (contents:read)",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "verify": "cargo test -p but-authz --test config both_files_prefers_agents_toml",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-authz",
        "start_ref": "both_files_committed_divergent_dev",
        "must_observe": [
          "Ok",
          "dev→contents:write"
        ],
        "must_not_observe": [
          "dev→contents:read",
          "unioned grants"
        ],
        "negative_control": {
          "would_fail_if": [
            "loader reads only `.gitbutler/permissions.toml` and omits agents preference",
            "loader merges divergent grants instead of prefers agents file",
            "preference inverted/static"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "both_files_committed_divergent_dev",
            "action": {
              "actor": "ci",
              "steps": [
                "load_governance_config",
                "assert dev→contents:write NOT contents:read"
              ]
            },
            "end_state": {
              "must_observe": [
                "principal `dev` has exactly `contents:write` from `.gitbutler/agents.toml`"
              ],
              "must_not_observe": [
                "principal `dev` has `contents:read` from `.gitbutler/permissions.toml`",
                "empty principals list"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "GIVEN permissions-only repo WHEN load_governance_config THEN Ok AND exactly one warning line naming permissions.toml + but agent migrate",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "verify": "cargo test -p but-authz --test config permissions_only_deprecation_warning",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-authz",
        "start_ref": "permissions_only_committed",
        "must_observe": [
          "Ok",
          "count==1",
          "names permissions.toml",
          "names but agent migrate"
        ],
        "must_not_observe": [
          "count==0",
          "count>1",
          "missing remediation",
          "Err"
        ],
        "negative_control": {
          "would_fail_if": [
            "no warning on legacy-only",
            "per-call emission",
            "omits remediation"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "permissions_only_committed",
            "action": {
              "actor": "ci",
              "steps": [
                "capture warnings",
                "load_governance_config → Ok",
                "count permissions.toml warnings == 1",
                "assert line contains but agent migrate"
              ]
            },
            "end_state": {
              "must_observe": [
                "load result == `Ok(GovConfig)`",
                "warning count == 1",
                "warning line contains `.gitbutler/permissions.toml`",
                "warning line contains `but agent migrate`"
              ],
              "must_not_observe": [
                "warning count == 0",
                "warning count > 1",
                "empty warning text",
                "result is `Err`"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "agents.toml parses to same GovConfig shape (dev/ro/release-bot/main)",
      "verify": "cargo test -p but-authz --test config agents_toml_parses_same_config",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "both-files prefers agents.toml for divergent dev grant",
      "verify": "cargo test -p but-authz --test config both_files_prefers_agents_toml",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "permissions-only emits exactly one warning naming permissions.toml + but agent migrate",
      "verify": "cargo test -p but-authz --test config permissions_only_deprecation_warning",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "permissions-only load returns Ok (warning is non-fatal)",
      "verify": "cargo test -p but-authz --test config permissions_only_deprecation_warning",
      "maps_to_ac": "AC-3"
    }
  ]
}
-->
