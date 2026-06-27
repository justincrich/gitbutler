# IDENT-020 — Extend `crates/but-authz/tests/invariant_build_gates.rs`: add `registry.rs` + `process.rs` to `ENFORCEMENT_PATHS`; verify gate callsites use the runtime registry wrapper that delegates to `but_authz::resolve_principal_with_registry`; negative grep on direct `BUT_AGENT_HANDLE` env reads outside `authorize.rs`; `AGENTS_PATH` constant required; `PERMISSIONS_PATH` `#[deprecated]`

**Sprint:** [Sprint 10](./SPRINT.md) · **Agent:** `rust-reviewer` · **Estimate:** 180 min · **Type:** FEATURE · **Status:** Complete · **Proposed By:** rust-planner

## Background

rust-reviewer owns invariant enforcement and must extend the build-gate test with new Sprint-10 invariants — this is review-qualified work requiring grep assertion discipline..

**Why it matters.** Closes the IDENT deprecation arc: on a governed repo the registry path is the default; the env-var path survives only as an opt-in escape hatch behind `BUT_AUTHZ_ALLOW_ENV_HANDLE=1`.

**Provides:** Extended invariant_build_gates.rs with Sprint-10 invariants, an authoritative 11-callsite runtime-registry wrapper set, broad production env-read exclusion, AGENTS_PATH, and PERMISSIONS_PATH deprecation

**Consumes:** Sprint-09 IDENT-009 (AGENTS_PATH constant), Sprint-09 IDENT-010 (callsites swapped to `resolve_principal_with_runtime_registry`, whose wrapper delegates to `but_authz::resolve_principal_with_registry`), IDENT-017 (resolver documented as test-only), IDENT-018 (Track A tests migrated), IDENT-019 (Track B tests added)

**Boundary contracts:**
- invariant_build_gates.rs is the SINGLE source of truth for Sprint-10 invariants
- All Sprint-10 enforcement paths are added to ENFORCEMENT_PATHS
- Grep assertions enforce positive and negative invariants


## Critical Constraints

**MUST:**
- Add `crates/but-authz/src/registry.rs` and `crates/but-authz/src/process.rs` to `ENFORCEMENT_PATHS`
- Add positive assertion: the governed but-api runtime-registry identity surface set is exactly the current 11 callsites: `commit/gate.rs::enforce_commit_gate_for_target`, `legacy/merge_gate.rs::enforce_merge_gate`, `legacy/config_mutate.rs::enforce_administration_write_gate`, `legacy/forge.rs::authorize_branch_action`, `legacy/rules.rs::list_workspace_rules_scoped_for_caller`, and `legacy/governance.rs::{governance_status_read, branch_gates_read_with_repo, group_list_with_repo, perm_list_with_repo, whoami_with_repo, can_i_with_repo}`
- Add delegation assertion: `resolve_principal_with_runtime_registry` delegates to `but_authz::resolve_principal_with_registry(Some(&registry), cfg)`
- Add negative grep assertion: production Rust sources affecting governed execution must NOT directly read `BUT_AGENT_HANDLE` outside `crates/but-authz/src/authorize.rs` (exclude test files and non-env doc/string mentions)
- Add negative-control assertion: Track B/gate registry tests must include the TTL/process-identity denial tests `expired_current_process_registry_entry_denied`, `current_pid_wrong_start_time_denied_at_commit_gate`, and `wrong_pid_current_start_time_denied_at_commit_gate`; success-only registry tests are insufficient
- Add build-gate check: `AGENTS_PATH` constant must be defined in config.rs
- Add build-gate check: `PERMISSIONS_PATH` must have `#[deprecated]` attribute
- All build-gate assertions must fail with clear error messages naming the violating file/line
- Tests must pass: `cargo test -p but-authz --test invariant_build_gates`

**NEVER:**
- Modify production code to satisfy build gates except the explicitly allowed one-line `#[deprecated]` attribute on `PERMISSIONS_PATH`
- Remove existing ENFORCEMENT_PATHS entries — preserve all Sprint-02 invariants
- Add direct `BUT_AGENT_HANDLE` env reads to governed production Rust sources outside authorize.rs
- Skip or weaken any assertion

**STRICTLY:**
- BLOCKED-UNTIL Sprint-09 IDENT-009 completes (AGENTS_PATH must exist)
- BLOCKED-UNTIL Sprint-09 IDENT-010 completes (callsites must be swapped)
- BLOCKED-UNTIL IDENT-017/018/019 complete (resolver verified, tests migrated)
- Build gates are ASSERTION-ONLY — they validate, they do not modify
- All build-gate failures are CI-blocking — the test must exit non-zero

## Specification

**Objective:** Extend invariant_build_gates.rs with Sprint-10 invariants: registry.rs and process.rs enforcement paths, exact 11-callsite wrapper-aware registry resolution assertions, broad negative grep for direct BUT_AGENT_HANDLE env reads outside authorize.rs, TTL/process-identity negative-control test coverage, AGENTS_PATH existence check, and PERMISSIONS_PATH #[deprecated] check.

**Success state:** cargo test -p but-authz --test invariant_build_gates passes all new assertions. ENFORCEMENT_PATHS includes registry.rs and process.rs. The invariant asserts the exact 11 runtime-registry identity callsites, including `legacy/rules.rs`, `whoami_with_repo`, and `can_i_with_repo`, and the wrapper delegates to `but_authz::resolve_principal_with_registry(Some(&registry), cfg)`. Negative scan finds zero direct `BUT_AGENT_HANDLE` env reads in governed production Rust sources outside authorize.rs. TTL/process-identity negative controls are required by name so success-only registry tests cannot fake coverage. AGENTS_PATH constant exists in config.rs. PERMISSIONS_PATH has #[deprecated].

## Acceptance Criteria

**AC-1 (PRIMARY)** — PRIMARY — registry.rs and process.rs added to ENFORCEMENT_PATHS
- **GIVEN:** Existing ENFORCEMENT_PATHS in invariant_build_gates.rs (AUTHZ_AUTHORIZE, AUTHZ_CONFIG, COMMIT_GATE, MERGE_GATE, CONFIG_MUTATE, GOVERNANCE, FORGE_GUARD)
- **WHEN:** Adding AUTHZ_REGISTRY and AUTHZ_PROCESS constants + paths
- **THEN:** ENFORCEMENT_PATHS contains 9 entries, build gates run against all 9 files
- **Verify:** `cargo test -p but-authz --test invariant_build_gates -- test_enforcement_paths_extended`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-authz · **FLOW_REF:** UC-IDENT-03
- **Scenario:** `start_ref=invariant_build_gates_source`; `must_observe` = ['ENFORCEMENT_PATHS array has length 9 (literal integer)', 'AUTHZ_REGISTRY constant added', 'AUTHZ_PROCESS constant added']; `must_not_observe` = ['ENFORCEMENT_PATHS length < 9', 'missing registry.rs', 'missing process.rs', 'empty array']; `negative_control.would_fail_if` = ['registry.rs not added to ENFORCEMENT_PATHS (omitted)', 'process.rs not added (absent)', 'ENFORCEMENT_PATHS array deleted or truncated'].

**AC-2** — Positive invariant: exact 11 governed identity callsites use runtime registry wrapper and wrapper delegates to authz registry resolver
- **GIVEN:** Current but-api gate callsites resolve through `resolve_principal_with_runtime_registry`
- **WHEN:** The invariant scans governed gate source files
- **THEN:** The discovered callsite set exactly matches the authoritative 11 entries, and the wrapper in `commit/gate.rs` delegates to `but_authz::resolve_principal_with_registry(Some(&registry), cfg)`
- **Verify:** `cargo test -p but-authz --test invariant_build_gates -- test_runtime_registry_wrapper_callsite_set_is_authoritative`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-authz · **FLOW_REF:** UC-IDENT-03
- **Scenario:** `start_ref=but_api_src_after_sprint09`; `must_observe` = ['exact callsite count == 11', 'callsite set includes literal `legacy/rules.rs::list_workspace_rules_scoped_for_caller`', 'callsite set includes literal `legacy/governance.rs::whoami_with_repo`', 'callsite set includes literal `legacy/governance.rs::can_i_with_repo`', 'wrapper contains literal `but_authz::resolve_principal_with_registry(Some(&registry), cfg)`']; `must_not_observe` = ['callsites still using old resolve_principal_from_env', 'callsite count < 11', 'callsite count > 11 without explicit classification', 'wrapper omits Registry load', 'empty result']; `negative_control.would_fail_if` = ["Sprint-09 IDENT-010 hasn't landed (callsites not swapped)", 'Grep only checks non-zero count', 'legacy/rules.rs omitted', 'whoami/can-i discovery surfaces omitted', 'wrapper no longer delegates to but_authz registry resolver'].

**AC-3** — Negative grep: direct BUT_AGENT_HANDLE env reads absent from governed production Rust outside authorize.rs
- **GIVEN:** Sprint-09 IDENT-010 swapped callsites to the runtime registry wrapper
- **WHEN:** Scanning production Rust sources affecting governed execution (`crates/but-api/src` and `crates/but-authz/src`), excluding `crates/but-authz/src/authorize.rs` and tests
- **THEN:** Zero direct env reads of `BUT_AGENT_HANDLE` exist outside authorize.rs; doc strings and remediation text do not count unless they call `env::var`/`env::var_os`
- **Verify:** `cargo test -p but-authz --test invariant_build_gates -- test_but_agent_handle_env_reads_only_in_authorize`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-authz · **FLOW_REF:** UC-IDENT-03
- **Scenario:** `start_ref=governed_production_sources_after_sprint09`; `must_observe` = ['scanner reports literal `direct_env_reads=0` outside authorize.rs', 'authorize.rs remains the only production source allowed to call env::var_os for BUT_AGENT_HANDLE']; `must_not_observe` = ['direct_env_reads > 0', 'any BUT_AGENT_HANDLE env::var/env::var_os read in but-api/src', 'test files included in scan']; `negative_control.would_fail_if` = ["Sprint-09 IDENT-010 hasn't landed (callsite still reads env)", 'Scanner only checks crates/but-api/src and misses but-authz production sources', 'grep not run'].

**AC-4** — AGENTS_PATH constant required in config.rs
- **GIVEN:** Sprint-09 IDENT-009 added AGENTS_PATH constant
- **WHEN:** Checking for AGENTS_PATH in but-authz/src/config.rs
- **THEN:** Constant exists with value '.gitbutler/agents.toml'
- **Verify:** `cargo test -p but-authz --test invariant_build_gates -- test_agents_path_exists`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-authz · **FLOW_REF:** UC-IDENT-01
- **Scenario:** `start_ref=config.rs_after_sprint09`; `must_observe` = ["AGENTS_PATH constant exists with exact literal 'AGENTS_PATH'", 'grep finds AGENTS_PATH in config.rs']; `must_not_observe` = ['AGENTS_PATH missing', 'grep returns 0 matches', 'constant absent or deleted']; `negative_control.would_fail_if` = ["Sprint-09 IDENT-009 hasn't landed (constant not added)", 'AGENTS_PATH omitted or absent', 'grep stubbed or not executed'].

**AC-5** — PERMISSIONS_PATH marked as #[deprecated]
- **GIVEN:** PERMISSIONS_PATH constant in config.rs (legacy, should be deprecated)
- **WHEN:** Checking for #[deprecated] attribute on PERMISSIONS_PATH
- **THEN:** Constant has #[deprecated] attribute
- **Verify:** `cargo test -p but-authz --test invariant_build_gates -- test_permissions_path_deprecated`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-authz · **FLOW_REF:** UC-IDENT-01
- **Scenario:** `start_ref=config.rs_with_deprecation`; `must_observe` = ['#[deprecated] attribute present on PERMISSIONS_PATH', "grep finds '#[deprecated]' and 'PERMISSIONS_PATH' adjacent"]; `must_not_observe` = ['PERMISSIONS_PATH without deprecation', '#[deprecated] missing', 'attribute omitted']; `negative_control.would_fail_if` = ['Deprecation not added (omitted)', 'Attribute syntax wrong or stubbed', 'PERMISSIONS_PATH deleted'].

**AC-6** — TTL/process-identity negative controls are mandatory, not success-only registry coverage
- **GIVEN:** IDENT-019 Track B/gate registry tests and invariant_build_gates.rs
- **WHEN:** The invariant scans gate_registry_swap.rs and agent_registry.rs for required registry negative controls
- **THEN:** The test names `expired_current_process_registry_entry_denied`, `current_pid_wrong_start_time_denied_at_commit_gate`, and `wrong_pid_current_start_time_denied_at_commit_gate` are present and success-only registry tests cannot satisfy the invariant
- **Verify:** `cargo test -p but-authz --test invariant_build_gates -- test_registry_ttl_process_identity_negative_controls_present`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-authz · **FLOW_REF:** UC-IDENT-03
- **Scenario:** `start_ref=registry_negative_control_tests`; `must_observe` = ['required test literal `expired_current_process_registry_entry_denied` present', 'required test literal `current_pid_wrong_start_time_denied_at_commit_gate` present', 'required test literal `wrong_pid_current_start_time_denied_at_commit_gate` present', 'scanner reports `success_only_registry_coverage=false`']; `must_not_observe` = ['only register→success tests present', 'expired-entry test missing', 'pid/start_time mismatch tests missing', 'empty scan']; `negative_control.would_fail_if` = ['Track B suite has only success tests', 'expired registry entry still authorizes', 'pid-only or start_time-only lookup authorizes stale identity', 'checker only counts agent_registry.rs tests'].

## Test Criteria

| ID | Boolean | Maps to AC |
|----|---------|------------|
| TC-1 | ENFORCEMENT_PATHS extended to 9 entries is true | AC-1 |
| TC-2 | Exact 11 governed identity callsites use runtime wrapper and wrapper delegates to authz registry resolver is true | AC-2 |
| TC-3 | Direct BUT_AGENT_HANDLE env reads are absent outside authorize.rs in governed production Rust sources is true | AC-3 |
| TC-4 | AGENTS_PATH exists is true | AC-4 |
| TC-5 | PERMISSIONS_PATH deprecated is true | AC-5 |
| TC-6 | TTL/process-identity negative-control test names are present and success-only registry coverage is rejected is true | AC-6 |

## Reading List

1. `crates/but-authz/tests/invariant_build_gates.rs:1-100` — Existing ENFORCEMENT_PATHS structure — need to extend with AUTHZ_REGISTRY and AUTHZ_PROCESS
2. `crates/but-authz/src/registry.rs:1-50` — Registry module structure — enforcement target
3. `crates/but-authz/src/process.rs:1-50` — Process module structure — enforcement target
4. `crates/but-authz/src/config.rs:1-20` — AGENTS_PATH and PERMISSIONS_PATH constants — need to verify AGENTS_PATH exists and PERMISSIONS_PATH has #[deprecated]
5. `crates/but-api/src/commit/gate.rs:120-184` — Runtime registry wrapper and delegation to `but_authz::resolve_principal_with_registry`
6. `crates/but-api/src/legacy/rules.rs:91-105` — `list_workspace_rules_scoped_for_caller` is part of the authoritative callsite set
7. `crates/but-api/src/legacy/governance.rs:479-495,558-568,1158-1168,1459-1470,1629-1642,1680-1694` — governance/status/list/discovery surfaces including `whoami_with_repo` and `can_i_with_repo`
8. `crates/but-api/src/**` + `crates/but-authz/src/**` — Production governed execution sources for the direct `BUT_AGENT_HANDLE` env-read scan (exclude tests and authorize.rs)
9. `crates/but-api/tests/gate_registry_swap.rs:1-420` — Required TTL/process-identity negative-control tests: `expired_current_process_registry_entry_denied`, `current_pid_wrong_start_time_denied_at_commit_gate`, `wrong_pid_current_start_time_denied_at_commit_gate`

## Guardrails

**WRITE-ALLOWED:**
- crates/but-authz/tests/invariant_build_gates.rs (MODIFY — add AUTHZ_REGISTRY, AUTHZ_PROCESS constants, extend ENFORCEMENT_PATHS, add wrapper-aware and env-read invariant tests)
- crates/but-authz/src/config.rs (MODIFY-ONE-LINE — add `#[deprecated]` immediately above `PERMISSIONS_PATH` if Sprint-09 did not already add it)

**WRITE-PROHIBITED:**
- crates/but-api/src/** — Do NOT modify production code to satisfy build gates
- crates/but-authz/src/** EXCEPT the one-line `#[deprecated]` attribute on `PERMISSIONS_PATH` in config.rs — do NOT modify registry.rs, process.rs, or authorize.rs
- Removing existing ENFORCEMENT_PATHS entries — preserve all Sprint-02 invariants

## Code Pattern

**Reference:** invariant_build_gates.rs already enforces Sprint-02 invariants (ROLE_BRANCH_PATTERN, HUMAN_OR_LABEL_BRANCH_PATTERN, etc.); Sprint-09 IDENT-009 adds AGENTS_PATH; Sprint-09 IDENT-010 swaps governed callsites to `resolve_principal_with_runtime_registry` and the wrapper delegates to `but_authz::resolve_principal_with_registry`; IDENT-017 documents resolver as test-only

**Pattern:** Grep-based invariant enforcement: For each invariant, run a targeted grep (rg or git grep) and assert the match count meets expectations. Fail with a clear error message naming the violating file/line.

**Source:** `Existing invariant_build_gates.rs pattern — same grep-assertion discipline`

**Design notes:**
- Build gates are ASSERTION-ONLY — they validate existing code, they don't modify it
- The 7 new assertions are: (1) ENFORCEMENT_PATHS includes registry.rs + process.rs, (2) the exact 11 governed identity callsites use `resolve_principal_with_runtime_registry`, (3) that wrapper delegates to `but_authz::resolve_principal_with_registry`, (4) direct `BUT_AGENT_HANDLE` env reads are absent from governed production sources outside authorize.rs, (5) TTL/process-identity negative-control tests are present and success-only registry coverage is rejected, (6) AGENTS_PATH exists, (7) PERMISSIONS_PATH deprecated
- All build-gate failures must exit non-zero to block CI

**Anti-pattern:** Do NOT modify production code to make build gates pass. Do NOT skip assertions. Do NOT weaken error messages.

## Agent Instructions

TDD RED→GREEN per AC (integration against the real crate — `but-authz` / `but-api` — real git/gitoxide, NO mocks):
1. **RED:** write each AC's failing test first (against the live code / current start state).
2. **GREEN:** make the minimal change (test-only for IDENT-017/018/019; invariant assertions for IDENT-020; doc-comments for IDENT-021).
3. Run `cargo fmt`, `cargo clippy -p <crate> --all-targets -- -D warnings`, then the task's verify commands.
4. Commit via `but commit` (governed). Note: this task is BLOCKED-UNTIL Sprint-09 IDENT-009/010/011 land.

## Orchestrator Verification Protocol

- `cargo test -p but-authz --test invariant_build_gates` → exit 0, all Sprint-10 assertions pass
- `grep -c 'ENFORCEMENT_PATHS' crates/but-authz/tests/invariant_build_gates.rs | grep -o '[0-9]\+' && rg 'const.*AUTHZ_[A-Z_]*.*=.*".*\.rs"' crates/but-authz/tests/invariant_build_gates.rs | wc -l` → 9 enforcement paths
- `cargo test -p but-authz --test invariant_build_gates -- test_runtime_registry_wrapper_callsite_set_is_authoritative` → exit 0, exact 11-callsite set matches source
- `rg 'but_authz::resolve_principal_with_registry\\(Some\\(&registry\\), cfg\\)' crates/but-api/src/commit/gate.rs` → Match found
- `cargo test -p but-authz --test invariant_build_gates -- test_but_agent_handle_env_reads_only_in_authorize` → exit 0
- `cargo test -p but-authz --test invariant_build_gates -- test_registry_ttl_process_identity_negative_controls_present` → exit 0; required expired/stale identity negative-control test names present
- `rg 'AGENTS_PATH' crates/but-authz/src/config.rs` → Match found
- `rg -B1 'const PERMISSIONS_PATH' crates/but-authz/src/config.rs | rg '#\[deprecated\]'` → Match found

## Agent Assignment

**Agent:** `rust-reviewer` — rust-reviewer owns invariant enforcement and must extend the build-gate test with new Sprint-10 invariants — this is review-qualified work requiring grep assertion discipline.
**Pairing:** none (single-surface Rust task). Honors `crates/AGENTS.md` + `crates/WORKSPACE_MODEL.md`.

## Evidence Gates

- `cargo test -p but-authz --test invariant_build_gates` (exit 0, all Sprint-10 assertions pass)
- `grep -c 'ENFORCEMENT_PATHS' crates/but-authz/tests/invariant_build_gates.rs | grep -o '[0-9]\+' && rg 'const.*AUTHZ_[A-Z_]*.*=.*".*\.rs"' crates/but-authz/tests/invariant_build_gates.rs | wc -l` (9 enforcement paths)
- `cargo test -p but-authz --test invariant_build_gates -- test_runtime_registry_wrapper_callsite_set_is_authoritative` (exit 0, exact 11-callsite set matches source)
- `rg 'but_authz::resolve_principal_with_registry\\(Some\\(&registry\\), cfg\\)' crates/but-api/src/commit/gate.rs` (Match found)
- `cargo test -p but-authz --test invariant_build_gates -- test_but_agent_handle_env_reads_only_in_authorize` (exit 0)
- `cargo test -p but-authz --test invariant_build_gates -- test_registry_ttl_process_identity_negative_controls_present` (exit 0; required expired/stale identity negative-control test names present)
- `rg 'AGENTS_PATH' crates/but-authz/src/config.rs` (Match found)
- `rg -B1 'const PERMISSIONS_PATH' crates/but-authz/src/config.rs | rg '#\[deprecated\]'` (Match found)

## Review Criteria

- AC-1: PRIMARY — registry.rs and process.rs added to ENFORCEMENT_PATHS — verified by `cargo test -p but-authz --test invariant_build_gates -- test_enforcement_paths_extended`.
- AC-2: Positive invariant: exact 11 governed identity callsites use runtime registry wrapper and wrapper delegates to authz registry resolver — verified by `cargo test -p but-authz --test invariant_build_gates -- test_runtime_registry_wrapper_callsite_set_is_authoritative`.
- AC-3: Negative grep: direct BUT_AGENT_HANDLE env reads absent from governed production Rust outside authorize.rs — verified by `cargo test -p but-authz --test invariant_build_gates -- test_but_agent_handle_env_reads_only_in_authorize`.
- AC-4: AGENTS_PATH constant required in config.rs — verified by `cargo test -p but-authz --test invariant_build_gates -- test_agents_path_exists`.
- AC-5: PERMISSIONS_PATH marked as #[deprecated] — verified by `cargo test -p but-authz --test invariant_build_gates -- test_permissions_path_deprecated`.
- AC-6: TTL/process-identity negative controls are mandatory, not success-only registry coverage — verified by `cargo test -p but-authz --test invariant_build_gates -- test_registry_ttl_process_identity_negative_controls_present`.
- Honors NEVER: Modify production code to satisfy build gates except the explicitly allowed one-line `#[deprecated]` attribute on `PERMISSIONS_PATH`.

## Dependencies

- **Depends on:** Sprint-09 IDENT-009 (AGENTS_PATH must exist), Sprint-09 IDENT-010 (callsites must be swapped to `resolve_principal_with_runtime_registry` and wrapper delegation must exist), IDENT-017 (resolver documented), IDENT-018 (Track A tests migrated), IDENT-019 (Track B tests added, including TTL/process-identity negative controls)
- **Blocks:** IDENT-021 (doc audit requires all invariants enforced first)
- **Capabilities:** CAP-AUTHZ-01, CAP-CONFIG-01

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "IDENT-020",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "invariant_build_gates_source": {
      "description": "invariant_build_gates.rs test file with ENFORCEMENT_PATHS",
      "seed_method": "public_api",
      "records": [
        "invariant_build_gates.rs exists",
        "ENFORCEMENT_PATHS currently has 7 entries (AUTHZ_AUTHORIZE, AUTHZ_CONFIG, COMMIT_GATE, MERGE_GATE, CONFIG_MUTATE, GOVERNANCE, FORGE_GUARD)",
        "AUTHZ_REGISTRY and AUTHZ_PROCESS constants not yet added"
      ]
    },
    "but_api_src_after_sprint09": {
      "description": "but-api/src after Sprint-09 IDENT-010 callsite swap to runtime registry wrapper, with the authoritative 11 runtime-registry identity callsites",
      "seed_method": "public_api",
      "records": [
        "commit/gate.rs::enforce_commit_gate_for_target calls resolve_principal_with_runtime_registry",
        "legacy/merge_gate.rs::enforce_merge_gate calls resolve_principal_with_runtime_registry",
        "legacy/config_mutate.rs::enforce_administration_write_gate calls resolve_principal_with_runtime_registry",
        "legacy/forge.rs::authorize_branch_action calls resolve_principal_with_runtime_registry",
        "legacy/rules.rs::list_workspace_rules_scoped_for_caller calls resolve_principal_with_runtime_registry",
        "legacy/governance.rs::governance_status_read calls resolve_principal_with_runtime_registry",
        "legacy/governance.rs::branch_gates_read_with_repo calls resolve_principal_with_runtime_registry",
        "legacy/governance.rs::group_list_with_repo calls resolve_principal_with_runtime_registry",
        "legacy/governance.rs::perm_list_with_repo calls resolve_principal_with_runtime_registry",
        "legacy/governance.rs::whoami_with_repo calls resolve_principal_with_runtime_registry",
        "legacy/governance.rs::can_i_with_repo calls resolve_principal_with_runtime_registry",
        "commit/gate.rs wrapper delegates to but_authz::resolve_principal_with_registry(Some(&registry), cfg)"
      ]
    },
    "governed_production_sources_after_sprint09": {
      "description": "production Rust sources affecting governed execution after Sprint-09 callsite swap",
      "seed_method": "public_api",
      "records": [
        "crates/but-api/src contains governed gate callsites",
        "crates/but-authz/src contains authz production sources",
        "crates/but-authz/src/authorize.rs is the only allowed production source for direct BUT_AGENT_HANDLE env reads",
        "test files are excluded from the scan"
      ]
    },
    "config.rs_after_sprint09": {
      "description": "but-authz/src/config.rs after Sprint-09 IDENT-009",
      "seed_method": "public_api",
      "records": [
        "AGENTS_PATH constant exists with value '.gitbutler/agents.toml'",
        "PERMISSIONS_PATH constant exists and has #[deprecated] attribute"
      ]
    },
    "config.rs_with_deprecation": {
      "description": "config.rs with PERMISSIONS_PATH marked as deprecated",
      "seed_method": "public_api",
      "records": [
        "PERMISSIONS_PATH constant at line 8",
        "#[deprecated] attribute present on PERMISSIONS_PATH",
        "PERMISSIONS_PATH value is '.gitbutler/permissions.toml'"
      ]
    },
    "registry_negative_control_tests": {
      "description": "gate_registry_swap.rs and agent_registry.rs include required TTL/process-identity negative controls, not only registry success tests",
      "seed_method": "public_api",
      "records": [
        "gate_registry_swap.rs contains fn expired_current_process_registry_entry_denied",
        "gate_registry_swap.rs contains fn current_pid_wrong_start_time_denied_at_commit_gate",
        "gate_registry_swap.rs contains fn wrong_pid_current_start_time_denied_at_commit_gate",
        "agent_registry.rs still contains success-path Track B tests"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "PRIMARY \u2014 registry.rs and process.rs added to ENFORCEMENT_PATHS",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "verify": "cargo test -p but-authz --test invariant_build_gates -- test_enforcement_paths_extended",
      "flow_ref": "UC-IDENT-03",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-authz",
        "start_ref": "invariant_build_gates_source",
        "must_observe": [
          "ENFORCEMENT_PATHS array has length 9 (literal integer)",
          "AUTHZ_REGISTRY constant added",
          "AUTHZ_PROCESS constant added"
        ],
        "must_not_observe": [
          "ENFORCEMENT_PATHS length < 9",
          "missing registry.rs",
          "missing process.rs",
          "empty array"
        ],
        "negative_control": {
          "would_fail_if": [
            "registry.rs not added to ENFORCEMENT_PATHS (omitted)",
            "process.rs not added (absent)",
            "ENFORCEMENT_PATHS array deleted or truncated"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "invariant_build_gates_source",
            "action": {
              "actor": "test_harness",
              "steps": [
                "Add AUTHZ_REGISTRY and AUTHZ_PROCESS constants",
                "Extend ENFORCEMENT_PATHS array",
                "Verify count is 9"
              ]
            },
            "end_state": {
              "must_observe": [
                "9 enforcement paths"
              ],
              "must_not_observe": [
                "< 9",
                "empty"
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
      "description": "Positive invariant: exact 11 governed identity callsites use runtime registry wrapper and wrapper delegates to authz registry resolver",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "verify": "cargo test -p but-authz --test invariant_build_gates -- test_runtime_registry_wrapper_callsite_set_is_authoritative",
      "flow_ref": "UC-IDENT-03",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-authz",
        "start_ref": "but_api_src_after_sprint09",
        "must_observe": [
          "exact callsite count == 11",
          "callsite set includes literal `legacy/rules.rs::list_workspace_rules_scoped_for_caller`",
          "callsite set includes literal `legacy/governance.rs::whoami_with_repo`",
          "callsite set includes literal `legacy/governance.rs::can_i_with_repo`",
          "wrapper contains literal `but_authz::resolve_principal_with_registry(Some(&registry), cfg)`"
        ],
        "must_not_observe": [
          "callsites still using old resolve_principal_from_env",
          "callsite count < 11",
          "callsite count > 11 without explicit classification",
          "wrapper omits Registry load",
          "empty result"
        ],
        "negative_control": {
          "would_fail_if": [
            "Sprint-09 IDENT-010 hasn't landed (callsites not swapped)",
            "Grep only checks non-zero count",
            "legacy/rules.rs omitted",
            "whoami/can-i discovery surfaces omitted",
            "wrapper no longer delegates to but_authz registry resolver"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "but_api_src_after_sprint09",
            "action": {
              "actor": "test_harness",
              "steps": [
                "Enumerate resolve_principal_with_runtime_registry callsites in crates/but-api/src",
                "Compare discovered set to the hard-coded 11-entry expected set",
                "Verify commit/gate.rs wrapper delegates to but_authz::resolve_principal_with_registry(Some(&registry), cfg)"
              ]
            },
            "end_state": {
              "must_observe": [
                "exact callsite count == 11",
                "callsite set includes literal `legacy/rules.rs::list_workspace_rules_scoped_for_caller`",
                "callsite set includes literal `legacy/governance.rs::whoami_with_repo`",
                "callsite set includes literal `legacy/governance.rs::can_i_with_repo`",
                "wrapper contains literal `but_authz::resolve_principal_with_registry(Some(&registry), cfg)`"
              ],
              "must_not_observe": [
                "callsites still using old resolve_principal_from_env",
                "callsite count < 11",
                "callsite count > 11 without explicit classification",
                "wrapper omits Registry load",
                "empty result"
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
      "description": "Negative grep: direct BUT_AGENT_HANDLE env reads absent from governed production Rust outside authorize.rs",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "verify": "cargo test -p but-authz --test invariant_build_gates -- test_but_agent_handle_env_reads_only_in_authorize",
      "flow_ref": "UC-IDENT-03",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-authz",
        "start_ref": "governed_production_sources_after_sprint09",
        "must_observe": [
          "scanner reports literal `direct_env_reads=0` outside authorize.rs",
          "authorize.rs remains the only production source allowed to call env::var_os for BUT_AGENT_HANDLE"
        ],
        "must_not_observe": [
          "direct_env_reads > 0",
          "any BUT_AGENT_HANDLE env::var/env::var_os read in but-api/src",
          "test files included in scan"
        ],
        "negative_control": {
          "would_fail_if": [
            "Sprint-09 IDENT-010 hasn't landed (callsite still reads env)",
            "Scanner only checks crates/but-api/src and misses but-authz production sources",
            "grep not run",
            "scanner is stubbed to return static success"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "governed_production_sources_after_sprint09",
            "action": {
              "actor": "test_harness",
              "steps": [
                "Scan crates/but-api/src and crates/but-authz/src",
                "Exclude crates/but-authz/src/authorize.rs and test files",
                "Flag only BUT_AGENT_HANDLE lines that also call env::var/env::var_os"
              ]
            },
            "end_state": {
              "must_observe": [
                "literal `direct_env_reads=0`"
              ],
              "must_not_observe": [
                "direct_env_reads > 0",
                "matches found outside authorize.rs"
              ]
            }
          },
          {
            "start_ref": "governed_production_sources_after_sprint09",
            "action": {
              "actor": "test_harness",
              "steps": [
                "grep -rc resolve_principal_with_runtime_registry crates/but-api/src"
              ]
            },
            "end_state": {
              "must_observe": [
                "governed callsites call `resolve_principal_with_runtime_registry`"
              ],
              "must_not_observe": [
                "0 wrapper callsites"
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
      "description": "AGENTS_PATH constant required in config.rs",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "verify": "cargo test -p but-authz --test invariant_build_gates -- test_agents_path_exists",
      "flow_ref": "UC-IDENT-01",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-authz",
        "start_ref": "config.rs_after_sprint09",
        "must_observe": [
          "AGENTS_PATH constant exists with exact literal 'AGENTS_PATH'",
          "grep finds AGENTS_PATH in config.rs"
        ],
        "must_not_observe": [
          "AGENTS_PATH missing",
          "grep returns 0 matches",
          "constant absent or deleted"
        ],
        "negative_control": {
          "would_fail_if": [
            "Sprint-09 IDENT-009 hasn't landed (constant not added)",
            "AGENTS_PATH omitted or absent",
            "grep stubbed or not executed"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "config.rs_after_sprint09",
            "action": {
              "actor": "test_harness",
              "steps": [
                "grep AGENTS_PATH crates/but-authz/src/config.rs",
                "Verify constant definition"
              ]
            },
            "end_state": {
              "must_observe": [
                "AGENTS_PATH == `.gitbutler/agents.toml`"
              ],
              "must_not_observe": [
                "AGENTS_PATH absent",
                "only PERMISSIONS_PATH present",
                "empty/start: no principal resolved, registry (0 entries)"
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
      "description": "PERMISSIONS_PATH marked as #[deprecated]",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "verify": "cargo test -p but-authz --test invariant_build_gates -- test_permissions_path_deprecated",
      "flow_ref": "UC-IDENT-01",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-authz",
        "start_ref": "config.rs_with_deprecation",
        "must_observe": [
          "#[deprecated] attribute present on PERMISSIONS_PATH",
          "grep finds '#[deprecated]' and 'PERMISSIONS_PATH' adjacent"
        ],
        "must_not_observe": [
          "PERMISSIONS_PATH without deprecation",
          "#[deprecated] missing",
          "attribute omitted"
        ],
        "negative_control": {
          "would_fail_if": [
            "Deprecation not added (omitted)",
            "Attribute syntax wrong or stubbed",
            "PERMISSIONS_PATH deleted"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "config.rs_with_deprecation",
            "action": {
              "actor": "test_harness",
              "steps": [
                "grep -A1 PERMISSIONS_PATH crates/but-authz/src/config.rs",
                "Verify #[deprecated] present"
              ]
            },
            "end_state": {
              "must_observe": [
                "`#[deprecated]` precedes `PERMISSIONS_PATH`"
              ],
              "must_not_observe": [
                "PERMISSIONS_PATH without `#[deprecated]`",
                "empty/start: no principal resolved, registry (0 entries)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-6",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "TTL/process-identity negative controls are mandatory, not success-only registry coverage",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "verify": "cargo test -p but-authz --test invariant_build_gates -- test_registry_ttl_process_identity_negative_controls_present",
      "flow_ref": "UC-IDENT-03",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-authz",
        "start_ref": "registry_negative_control_tests",
        "must_observe": [
          "required test literal `expired_current_process_registry_entry_denied` present",
          "required test literal `current_pid_wrong_start_time_denied_at_commit_gate` present",
          "required test literal `wrong_pid_current_start_time_denied_at_commit_gate` present",
          "scanner reports `success_only_registry_coverage=false`"
        ],
        "must_not_observe": [
          "only register→success tests present",
          "expired-entry test missing",
          "pid/start_time mismatch tests missing",
          "empty scan"
        ],
        "negative_control": {
          "would_fail_if": [
            "Track B suite has only success tests",
            "expired registry entry still authorizes",
            "pid-only or start_time-only lookup authorizes stale identity",
            "checker only counts agent_registry.rs tests",
            "TTL/process-identity checker stubbed or omitted"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "registry_negative_control_tests",
            "action": {
              "actor": "test_harness",
              "steps": [
                "Scan gate_registry_swap.rs and agent_registry.rs",
                "Assert all three TTL/process-identity negative-control test names are present",
                "Assert success-only registry tests cannot satisfy the invariant"
              ]
            },
            "end_state": {
              "must_observe": [
                "required test literal `expired_current_process_registry_entry_denied` present",
                "required test literal `current_pid_wrong_start_time_denied_at_commit_gate` present",
                "required test literal `wrong_pid_current_start_time_denied_at_commit_gate` present",
                "scanner reports `success_only_registry_coverage=false`"
              ],
              "must_not_observe": [
                "only register→success tests present",
                "expired-entry test missing",
                "pid/start_time mismatch tests missing",
                "empty scan"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "ENFORCEMENT_PATHS extended to 9 entries is true",
      "maps_to_ac": "AC-1",
      "verify": "cargo test -p but-authz --test invariant_build_gates -- test_enforcement_paths_extended"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "Exact 11 governed identity callsites use runtime wrapper and wrapper delegates to authz registry resolver is true",
      "maps_to_ac": "AC-2",
      "verify": "cargo test -p but-authz --test invariant_build_gates -- test_runtime_registry_wrapper_callsite_set_is_authoritative"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "Direct BUT_AGENT_HANDLE env reads are absent outside authorize.rs in governed production Rust sources is true",
      "maps_to_ac": "AC-3",
      "verify": "cargo test -p but-authz --test invariant_build_gates -- test_but_agent_handle_env_reads_only_in_authorize"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "AGENTS_PATH exists is true",
      "maps_to_ac": "AC-4",
      "verify": "cargo test -p but-authz --test invariant_build_gates -- test_agents_path_exists"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "PERMISSIONS_PATH deprecated is true",
      "maps_to_ac": "AC-5",
      "verify": "cargo test -p but-authz --test invariant_build_gates -- test_permissions_path_deprecated"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "description": "TTL/process-identity negative-control test names are present and success-only registry coverage is rejected is true",
      "maps_to_ac": "AC-6",
      "verify": "cargo test -p but-authz --test invariant_build_gates -- test_registry_ttl_process_identity_negative_controls_present"
    }
  ]
}
-->
