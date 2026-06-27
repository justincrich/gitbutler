# IDENT-018 — Mechanical update of the but-api tests using `temp_env::with_var("BUT_AGENT_HANDLE", ...)` to set `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` in their helpers (Track A — no churn, keep working)

**Sprint:** [Sprint 10](./SPRINT.md) · **Agent:** `rust-implementer` · **Estimate:** 180 min · **Type:** FEATURE · **Status:** Complete · **Proposed By:** rust-planner

## Background

rust-implementer owns the but-api test surface and must add the flag to every current test helper that sets `BUT_AGENT_HANDLE` — including `temp_env::with_var` and `temp_env::async_with_vars` callsites — and lock the migration with structural verification instead of stale grep counts.

**Why it matters.** Closes the IDENT deprecation arc: on a governed repo the registry path is the default; the env-var path survives only as an opt-in escape hatch behind `BUT_AUTHZ_ALLOW_ENV_HANDLE=1`.

**Provides:** Track A tests continue to work via `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` env fallback, with a structural test preventing new unflagged `BUT_AGENT_HANDLE` helpers

**Consumes:** Sprint-08 `BUT_AUTHZ_ALLOW_ENV_HANDLE` flag, IDENT-017 verified resolver deny-default

**Boundary contracts:**
- `temp_env::with_var` and `temp_env::async_with_vars` calls now set BOTH `BUT_AGENT_HANDLE` and `BUT_AUTHZ_ALLOW_ENV_HANDLE=1`
- Tests remain otherwise unchanged — no registry wiring, no `with_registered_agent` helper


## Critical Constraints

**MUST:**
- Add `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` to EVERY existing `temp_env::with_var("BUT_AGENT_HANDLE", ...)` and `temp_env::async_with_vars([("BUT_AGENT_HANDLE", ...)]` helper scope across all current but-api tests
- Run `cargo test -p but-api --test <test_target>` after each file modification (`${file%.rs}` target names, never `.rs` filenames)
- Preserve all existing test behavior — assertions, fixtures, helper names remain unchanged
- Add a structural invariant test proving every env-helper scope that sets `BUT_AGENT_HANDLE` also sets `BUT_AUTHZ_ALLOW_ENV_HANDLE=1`
- If a count check is used, use `awk -F:` for `grep -c` output and treat matching totals as a smoke test only; the structural invariant is authoritative

**NEVER:**
- Modify test assertions or logic — only add the env var to existing temp_env calls
- Remove or delete existing tests — this is Track A preservation, not Track B new tests
- Introduce registry-based fixtures — that's IDENT-019's job
- Use `std::env::temp_dir().join(format!(...))` — all existing fixtures are already correct
- Touch test files OUTSIDE but-api/tests (but/tests, but-authz/tests)

**STRICTLY:**
- BLOCKED-UNTIL Sprint-09 IDENT-010 completes (the callsite swap must land before the env fallback becomes the only path for Track A tests)
- BLOCKED-UNTIL Sprint-09 IDENT-009 completes (AGENTS_PATH must exist for Sprint-10 invariant)
- BLOCKED-UNTIL IDENT-017 completes (resolver deny-default must be verified before these tests mean anything)
- Mechanical transformation only — no test logic changes
- All tests must pass after the transformation: `cargo test -p but-api --tests`

## Specification

**Objective:** Mechanically add `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` to every current but-api test helper scope that sets `BUT_AGENT_HANDLE`, preserving existing test behavior via env fallback (Track A — no churn, keep working), and add a structural invariant so future helper scopes cannot omit the flag.

**Success state:** cargo test -p but-api --tests passes all existing tests. `cargo test -p but-api --test gate_registry_swap -- all_but_agent_handle_env_helpers_are_flag_gated` proves every `BUT_AGENT_HANDLE` env-helper scope is paired with `BUT_AUTHZ_ALLOW_ENV_HANDLE=1`. All modified test targets pass individually.

## Acceptance Criteria

**AC-1 (PRIMARY)** — PRIMARY — Every BUT_AGENT_HANDLE env-helper scope also sets BUT_AUTHZ_ALLOW_ENV_HANDLE=1
- **GIVEN:** Current `crates/but-api/tests/**/*.rs` files with `temp_env::with_var("BUT_AGENT_HANDLE", ...)` and `temp_env::async_with_vars([("BUT_AGENT_HANDLE", ...)]` helper scopes
- **WHEN:** Adding `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` in the same helper scope as each `BUT_AGENT_HANDLE` setup
- **THEN:** The structural invariant finds zero unpaired env-helper scopes, and all but-api tests pass unchanged
- **Verify:** `cargo test -p but-api --test gate_registry_swap -- all_but_agent_handle_env_helpers_are_flag_gated && cargo test -p but-api --tests`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-api · **FLOW_REF:** UC-IDENT-03
- **Scenario:** `start_ref=existing_but_api_tests`; `must_observe` = ['structural invariant reports 0 unpaired BUT_AGENT_HANDLE helper scopes', 'cargo test -p but-api --tests exits with code 0']; `must_not_observe` = ['unpaired helper scope count > 0', 'test failures', 'exit code ≠ 0', 'missing flag in any callsite']; `negative_control.would_fail_if` = ['Flag not added to a helper scope (omitted)', 'Test assertions modified to stub success', 'Structural checker only counts strings and does not inspect helper scope'].

**AC-2** — High-frequency test files structurally paired (commit_gate.rs, perm_governance.rs, local_review_assignments.rs)
- **GIVEN:** 3 test files with high `BUT_AGENT_HANDLE` usage
- **WHEN:** Adding the flag to all callsites in these files
- **THEN:** Each file has zero unpaired `BUT_AGENT_HANDLE` helper scopes
- **Verify:** `for file in commit_gate.rs perm_governance.rs local_review_assignments.rs; do cargo test -p but-api --test gate_registry_swap -- all_but_agent_handle_env_helpers_are_flag_gated || exit 1; cargo test -p but-api --test ${file%.rs} || exit 1; done`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-api · **FLOW_REF:** None
- **Scenario:** `start_ref=high_frequency_test_files`; `must_observe` = ['commit_gate.rs has 0 unpaired helper scopes', 'perm_governance.rs has 0 unpaired helper scopes', 'local_review_assignments.rs has 0 unpaired helper scopes']; `must_not_observe` = ['Any unpaired helper scope in those files', 'missing flag callsites']; `negative_control.would_fail_if` = ['Callsite omitted from a high-frequency file', 'Checker does not inspect scope and is fooled by an unrelated flag string'].

**AC-3** — All modified test targets pass individually after flag addition
- **GIVEN:** All but-api test files modified to add `BUT_AUTHZ_ALLOW_ENV_HANDLE=1`
- **WHEN:** Running `cargo test -p but-api --test ${file%.rs}` for each modified file
- **THEN:** Every modified test target passes (exit 0, tests pass)
- **Verify:** `for file in admin_write_guard.rs branch_gates_governance.rs confinement.rs commit_gate.rs forge_guard.rs governance_api.rs keep_reviews_local_gate.rs local_review_assignments.rs local_review_comments_verbs.rs local_review_status.rs merge_gate.rs merge_gate_self_escalation.rs group_governance.rs perm_governance.rs steer_class_wiring.rs rules_scoped.rs steer_telemetry.rs; do cargo test -p but-api --test ${file%.rs} || exit 1; done`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-api · **FLOW_REF:** None
- **Scenario:** `start_ref=all_modified_test_files`; `must_observe` = ['Every modified test target exits with code 0 individually', '17/17 current Track A test targets pass when modified']; `must_not_observe` = ['Any test target fails with exit ≠ 0', 'test failures in any file']; `negative_control.would_fail_if` = ['Flag not added in a file (omitted)', 'Test logic accidentally modified to stub', 'Test run skipped or not executed'].

**AC-4** — Structural pairing checker rejects a deliberately unflagged helper scope
- **GIVEN:** The new invariant test scans `crates/but-api/tests` for env-helper scopes that set `BUT_AGENT_HANDLE`
- **WHEN:** A scope contains `BUT_AGENT_HANDLE` but no same-scope `BUT_AUTHZ_ALLOW_ENV_HANDLE=1`
- **THEN:** The invariant reports the violating file/line and fails
- **Verify:** `cargo test -p but-api --test gate_registry_swap -- all_but_agent_handle_env_helpers_are_flag_gated`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-api · **FLOW_REF:** UC-IDENT-03
- **Scenario:** `start_ref=structural_pairing_checker`; `must_observe` = ['checker names every unpaired file:line if present', 'checker exits 0 only when unpaired count == 0']; `must_not_observe` = ['checker passes when an unpaired helper exists', 'string count only', 'empty scan']; `negative_control.would_fail_if` = ['checker only compares total counts', 'checker ignores async_with_vars', 'checker ignores helper scope'].

## Test Criteria

| ID | Boolean | Maps to AC |
|----|---------|------------|
| TC-1 | Every BUT_AGENT_HANDLE env-helper scope has same-scope BUT_AUTHZ_ALLOW_ENV_HANDLE=1 is true | AC-1 |
| TC-2 | All but-api tests pass is true | AC-1 |
| TC-3 | High-frequency files have zero unpaired helper scopes is true | AC-2 |
| TC-4 | All modified test targets pass individually is true | AC-3 |
| TC-5 | Structural pairing checker fails closed on unflagged helper scopes is true | AC-4 |

## Reading List

1. `crates/but-api/tests:*.rs` — current Track A tests using `BUT_AGENT_HANDLE`, including `temp_env::with_var` and `temp_env::async_with_vars` forms (discover with `rg -l 'temp_env::with_var\("BUT_AGENT_HANDLE"|temp_env::async_with_vars\(\[\("BUT_AGENT_HANDLE"' crates/but-api/tests`)
2. `crates/but-api/tests/commit_gate.rs:1-50` — Example of temp_env::with_var usage pattern — duplicate with the new flag
3. `crates/but-api/tests/perm_governance.rs:1-50` — Highest-frequency file (20 callsites) — mechanical transformation

## Guardrails

**WRITE-ALLOWED:**
- crates/but-api/tests/admin_write_guard.rs (MODIFY — add flag to 2 callsites)
- crates/but-api/tests/branch_gates_governance.rs (MODIFY — add flag to 5 callsites)
- crates/but-api/tests/confinement.rs (MODIFY — add flag to 3 callsites)
- crates/but-api/tests/commit_gate.rs (MODIFY — add flag to 15 callsites)
- crates/but-api/tests/forge_guard.rs (MODIFY — add flag to 4 callsites)
- crates/but-api/tests/governance_api.rs (MODIFY — add flag to 7 callsites)
- crates/but-api/tests/keep_reviews_local_gate.rs (MODIFY — add flag to 1 callsite)
- crates/but-api/tests/local_review_assignments.rs (MODIFY — add flag to 10 callsites)
- crates/but-api/tests/group_governance.rs (MODIFY — add flag to all callsites)
- crates/but-api/tests/local_review_comments_verbs.rs (MODIFY — add flag to all async helper scopes)
- crates/but-api/tests/local_review_status.rs (MODIFY — add flag to all async helper scopes)
- crates/but-api/tests/merge_gate.rs (MODIFY — add flag to all async helper scopes)
- crates/but-api/tests/merge_gate_self_escalation.rs (MODIFY — add flag to all async helper scopes)
- crates/but-api/tests/perm_governance.rs (MODIFY — add flag to 20 callsites)
- crates/but-api/tests/steer_class_wiring.rs (MODIFY — add flag to 5 callsites)
- crates/but-api/tests/rules_scoped.rs (MODIFY — add flag to 3 callsites)
- crates/but-api/tests/steer_telemetry.rs (MODIFY — add flag to 4 callsites)
- crates/but-api/tests/gate_registry_swap.rs (MODIFY — add structural pairing invariant test)

**WRITE-PROHIBITED:**
- crates/but-api/src/** — Do NOT touch production callsites (Sprint-09 IDENT-010 owns the swap)
- crates/but-authz/** — Resolver is already verified (IDENT-017)
- crates/but/** — CLI migration is Sprint-09 territory
- crates/but-api/tests/agent_registry.rs — IDENT-019 owns this new Track B test file
- Any test logic changes — only add env var to existing temp_env calls, do NOT modify assertions

## Code Pattern

**Reference:** IDENT-017 verifies the resolver deny-default — these tests prove the env fallback path works through existing but-api gates/helpers; Sprint-09 IDENT-010 swaps governed callsites to `resolve_principal_with_runtime_registry`, whose wrapper loads the runtime registry and delegates to `but_authz::resolve_principal_with_registry(Some(&registry), cfg)`; Track A tests continue via flag-gated env fallback; UC-IDENT-03 specifies the fallback path

**Pattern:** Mechanical transformation plus structural guard: For each `temp_env::with_var("BUT_AGENT_HANDLE", ...)` call, wrap or nest it with `temp_env::with_var("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1"), || ...)`; for each `temp_env::async_with_vars([("BUT_AGENT_HANDLE", ...)]` array, add `("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1"))` in the same array. Preserve all other test logic unchanged.

**Source:** `Sprint-08 test patterns using `temp_env` — same pattern, just an additional env var`

**Design notes:**
- This is Track A (env fallback) — no registry wiring, no new test patterns
- The transformation is purely mechanical: the flag must be in the same temp-env scope as the handle, not in an unrelated surrounding test setup
- After Sprint-09 IDENT-010 lands, these tests keep calling existing but-api gates/helpers; those gates resolve through `resolve_principal_with_runtime_registry`, which delegates to `but_authz::resolve_principal_with_registry(Some(&registry), cfg)`, misses the empty runtime registry, hits the flag check, and falls through to `resolve_principal_from_env` — the same behavior they had before
- IDENT-019 creates Track B tests that use the registry path directly

**Anti-pattern:** Do NOT modify test assertions, fixtures, or helper names. Do NOT add registry-based setup (that's Track B). Do NOT remove or delete tests. Do NOT touch production but-api/src code.

## Agent Instructions

TDD RED→GREEN per AC (integration against the real crate — `but-authz` / `but-api` — real git/gitoxide, NO mocks):
1. **RED:** write each AC's failing test first (against the live code / current start state).
2. **GREEN:** make the minimal change (test-only for IDENT-017/018/019; invariant assertions for IDENT-020; doc-comments for IDENT-021).
3. Run `cargo fmt`, `cargo clippy -p <crate> --all-targets -- -D warnings`, then the task's verify commands.
4. Commit via `but commit` (governed). Note: this task is BLOCKED-UNTIL Sprint-09 IDENT-009/010/011 land.

## Orchestrator Verification Protocol

- `cargo test -p but-api --test gate_registry_swap -- all_but_agent_handle_env_helpers_are_flag_gated` → exit 0, zero unpaired helper scopes; failures name file/line
- `grep -c 'BUT_AUTHZ_ALLOW_ENV_HANDLE' crates/but-api/tests/*.rs | awk -F: '{sum+=$2} END {print sum}'` → smoke-test count only; do not treat as authoritative without the structural test
- `cargo test -p but-api --tests` → exit 0, all tests pass
- `for f in admin_write_guard.rs branch_gates_governance.rs confinement.rs commit_gate.rs forge_guard.rs governance_api.rs keep_reviews_local_gate.rs local_review_assignments.rs local_review_comments_verbs.rs local_review_status.rs merge_gate.rs merge_gate_self_escalation.rs group_governance.rs perm_governance.rs steer_class_wiring.rs rules_scoped.rs steer_telemetry.rs; do cargo test -p but-api --test ${f%.rs} || exit 1; done` → exit 0
- `cargo check -p but-api --all-targets` → exit 0

## Agent Assignment

**Agent:** `rust-implementer` — rust-implementer owns the but-api test surface and must add the flag to every current `BUT_AGENT_HANDLE` env-helper scope — a mechanical but tedious task requiring structural verification.
**Pairing:** none (single-surface Rust task). Honors `crates/AGENTS.md` + `crates/WORKSPACE_MODEL.md`.

## Evidence Gates

- `cargo test -p but-api --test gate_registry_swap -- all_but_agent_handle_env_helpers_are_flag_gated` (exit 0, zero unpaired helper scopes; failures name file/line)
- `grep -c 'BUT_AUTHZ_ALLOW_ENV_HANDLE' crates/but-api/tests/*.rs | awk -F: '{sum+=$2} END {print sum}'` (smoke-test count only)
- `cargo test -p but-api --tests` (exit 0, all tests pass)
- `for f in admin_write_guard.rs branch_gates_governance.rs confinement.rs commit_gate.rs forge_guard.rs governance_api.rs keep_reviews_local_gate.rs local_review_assignments.rs local_review_comments_verbs.rs local_review_status.rs merge_gate.rs merge_gate_self_escalation.rs group_governance.rs perm_governance.rs steer_class_wiring.rs rules_scoped.rs steer_telemetry.rs; do cargo test -p but-api --test ${f%.rs} || exit 1; done` (exit 0)
- `cargo check -p but-api --all-targets` (exit 0)

## Review Criteria

- AC-1: PRIMARY — Every BUT_AGENT_HANDLE env-helper scope also sets BUT_AUTHZ_ALLOW_ENV_HANDLE=1 — verified by `cargo test -p but-api --test gate_registry_swap -- all_but_agent_handle_env_helpers_are_flag_gated && cargo test -p but-api --tests`.
- AC-2: High-frequency test files structurally paired — verified by the structural checker plus per-target tests.
- AC-3: All modified test targets pass individually after flag addition — verified by `for file in ...; do cargo test -p but-api --test ${file%.rs}; done`.
- AC-4: Structural pairing checker rejects unflagged helper scopes — verified by `cargo test -p but-api --test gate_registry_swap -- all_but_agent_handle_env_helpers_are_flag_gated`.
- Honors NEVER: Modify test assertions or logic — only add the env var to existing temp_env calls.

## Dependencies

- **Depends on:** Sprint-09 IDENT-010 (but-api callsites must swap to `resolve_principal_with_runtime_registry`, whose wrapper delegates to `but_authz::resolve_principal_with_registry`, before the flag-gated fallback is exercised), Sprint-09 IDENT-009 (AGENTS_PATH must exist for Sprint-10 invariant), IDENT-017 (resolver deny-default must be verified and documented), Sprint-08 IDENT-002 (BUT_AUTHZ_ALLOW_ENV_HANDLE flag exists)
- **Blocks:** IDENT-019 (Track B tests use the registry path — Track A must continue working first), IDENT-020 (invariant extension requires all Track A tests passing), IDENT-021 (doc audit requires the test migration complete)
- **Capabilities:** CAP-AUTHZ-01

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "IDENT-018",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "existing_but_api_tests": {
      "description": "current crates/but-api/tests files with temp_env helpers that set BUT_AGENT_HANDLE",
      "seed_method": "public_api",
      "records": [
        "temp_env::with_var(\"BUT_AGENT_HANDLE\", ...) callsites exist in but-api tests",
        "temp_env::async_with_vars([(\"BUT_AGENT_HANDLE\", ...)] callsites exist in but-api tests",
        "gate_registry_swap.rs can host a structural invariant over the test source tree"
      ]
    },
    "high_frequency_test_files": {
      "description": "commit_gate.rs, perm_governance.rs, and local_review_assignments.rs have high BUT_AGENT_HANDLE usage",
      "seed_method": "public_api",
      "records": [
        "commit_gate.rs exists and contains multiple BUT_AGENT_HANDLE helper scopes",
        "perm_governance.rs exists and contains multiple BUT_AGENT_HANDLE helper scopes",
        "local_review_assignments.rs exists and contains multiple BUT_AGENT_HANDLE helper scopes"
      ]
    },
    "all_modified_test_files": {
      "description": "all current Track A but-api test targets modified to add BUT_AUTHZ_ALLOW_ENV_HANDLE=1",
      "seed_method": "public_api",
      "records": [
        "17 current Track A test files are discoverable with rg",
        "Each modified filename maps to cargo test target ${file%.rs}",
        "cargo test -p but-api --test <target> is the per-file verification form"
      ]
    },
    "structural_pairing_checker": {
      "description": "gate_registry_swap.rs structural invariant test for unpaired BUT_AGENT_HANDLE helper scopes",
      "seed_method": "public_api",
      "records": [
        "test scans crates/but-api/tests source files",
        "test inspects helper scope, not only total string counts",
        "test includes temp_env::with_var and temp_env::async_with_vars forms"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "PRIMARY — Every BUT_AGENT_HANDLE env-helper scope also sets BUT_AUTHZ_ALLOW_ENV_HANDLE=1",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-api --test gate_registry_swap -- all_but_agent_handle_env_helpers_are_flag_gated && cargo test -p but-api --tests",
      "flow_ref": "UC-IDENT-03",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "existing_but_api_tests",
        "must_observe": [
          "structural invariant prints literal `all_scopes_paired=true`",
          "unpaired helper scope count == 0",
          "cargo test -p but-api --tests exits with code 0"
        ],
        "must_not_observe": [
          "unpaired helper scope count > 0",
          "test failures",
          "exit code != 0",
          "missing flag in any helper scope"
        ],
        "negative_control": {
          "would_fail_if": [
            "Flag not added to a helper scope (omitted)",
            "Test assertions modified to stub success",
            "Structural checker only counts strings and does not inspect helper scope"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "existing_but_api_tests",
            "action": {
              "actor": "test_harness",
              "steps": [
                "Add BUT_AUTHZ_ALLOW_ENV_HANDLE=1 in the same helper scope as each BUT_AGENT_HANDLE setup",
                "Run cargo test -p but-api --test gate_registry_swap -- all_but_agent_handle_env_helpers_are_flag_gated",
                "Run cargo test -p but-api --tests"
              ]
            },
            "end_state": {
              "must_observe": [
                "literal `all_scopes_paired=true`",
                "0 unpaired helper scopes",
                "but-api tests exit 0"
              ],
              "must_not_observe": [
                "unpaired helper scope count > 0",
                "test failures",
                "empty scan"
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
      "description": "High-frequency test files structurally paired (commit_gate.rs, perm_governance.rs, local_review_assignments.rs)",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "for file in commit_gate.rs perm_governance.rs local_review_assignments.rs; do cargo test -p but-api --test gate_registry_swap -- all_but_agent_handle_env_helpers_are_flag_gated || exit 1; cargo test -p but-api --test ${file%.rs} || exit 1; done",
      "flow_ref": null,
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "high_frequency_test_files",
        "must_observe": [
          "commit_gate.rs has literal `all_scopes_paired=true`",
          "perm_governance.rs has literal `all_scopes_paired=true`",
          "local_review_assignments.rs has literal `all_scopes_paired=true`"
        ],
        "must_not_observe": [
          "Any unpaired helper scope in those files",
          "missing flag callsites",
          "checker pass based only on unrelated string count"
        ],
        "negative_control": {
          "would_fail_if": [
            "Callsite omitted from a high-frequency file",
            "Checker does not inspect scope and is fooled by an unrelated flag string",
            "Checker is stubbed to return static success"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "high_frequency_test_files",
            "action": {
              "actor": "test_harness",
              "steps": [
                "Run structural checker",
                "Run cargo test for commit_gate, perm_governance, and local_review_assignments using ${file%.rs}"
              ]
            },
            "end_state": {
              "must_observe": [
                "3 named high-frequency targets pass",
                "literal `all_scopes_paired=true`"
              ],
              "must_not_observe": [
                "unpaired helper scope count > 0",
                "any high-frequency target failure"
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
      "description": "All modified test targets pass individually after flag addition",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "for file in admin_write_guard.rs branch_gates_governance.rs confinement.rs commit_gate.rs forge_guard.rs governance_api.rs keep_reviews_local_gate.rs local_review_assignments.rs local_review_comments_verbs.rs local_review_status.rs merge_gate.rs merge_gate_self_escalation.rs group_governance.rs perm_governance.rs steer_class_wiring.rs rules_scoped.rs steer_telemetry.rs; do cargo test -p but-api --test ${file%.rs} || exit 1; done",
      "flow_ref": null,
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "all_modified_test_files",
        "must_observe": [
          "Every modified test target exits with code 0 individually",
          "17/17 current Track A test targets pass when modified"
        ],
        "must_not_observe": [
          "Any test target fails with exit != 0",
          "test failures in any file",
          "cargo invoked with .rs filename instead of target name"
        ],
        "negative_control": {
          "would_fail_if": [
            "Flag not added in a file (omitted)",
            "Test logic accidentally modified to stub",
            "Test run skipped or not executed"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "all_modified_test_files",
            "action": {
              "actor": "test_harness",
              "steps": [
                "For each modified .rs file derive target with ${file%.rs}",
                "Run cargo test -p but-api --test ${file%.rs}",
                "Verify all targets exit 0"
              ]
            },
            "end_state": {
              "must_observe": [
                "17/17 current Track A targets pass",
                "exit code 0"
              ],
              "must_not_observe": [
                "any failure",
                "exit != 0",
                "cargo target includes .rs suffix"
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
      "description": "Structural pairing checker rejects a deliberately unflagged helper scope",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-api --test gate_registry_swap -- all_but_agent_handle_env_helpers_are_flag_gated",
      "flow_ref": "UC-IDENT-03",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "structural_pairing_checker",
        "must_observe": [
          "checker names every unpaired file:line if present",
          "checker prints literal `all_scopes_paired=true` when unpaired count == 0"
        ],
        "must_not_observe": [
          "checker passes when an unpaired helper exists",
          "string count only",
          "empty scan"
        ],
        "negative_control": {
          "would_fail_if": [
            "checker only compares total counts",
            "checker ignores async_with_vars",
            "checker ignores helper scope",
            "checker is stubbed to return static success"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "structural_pairing_checker",
            "action": {
              "actor": "test_harness",
              "steps": [
                "Run structural invariant against crates/but-api/tests",
                "Confirm violations include file:line output",
                "Confirm clean tree prints all_scopes_paired=true"
              ]
            },
            "end_state": {
              "must_observe": [
                "literal `all_scopes_paired=true`",
                "violation output includes file:line when seeded with an unpaired helper"
              ],
              "must_not_observe": [
                "false pass with unpaired helper",
                "0 files scanned",
                "empty output"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "Every BUT_AGENT_HANDLE env-helper scope has same-scope BUT_AUTHZ_ALLOW_ENV_HANDLE=1 is true",
      "maps_to_ac": "AC-1",
      "verify": "cargo test -p but-api --test gate_registry_swap -- all_but_agent_handle_env_helpers_are_flag_gated"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "All but-api tests pass is true",
      "maps_to_ac": "AC-1",
      "verify": "cargo test -p but-api --tests"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "High-frequency files have zero unpaired helper scopes is true",
      "maps_to_ac": "AC-2",
      "verify": "for file in commit_gate.rs perm_governance.rs local_review_assignments.rs; do cargo test -p but-api --test ${file%.rs} || exit 1; done"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "All modified test targets pass individually is true",
      "maps_to_ac": "AC-3",
      "verify": "for file in admin_write_guard.rs branch_gates_governance.rs confinement.rs commit_gate.rs forge_guard.rs governance_api.rs keep_reviews_local_gate.rs local_review_assignments.rs local_review_comments_verbs.rs local_review_status.rs merge_gate.rs merge_gate_self_escalation.rs group_governance.rs perm_governance.rs steer_class_wiring.rs rules_scoped.rs steer_telemetry.rs; do cargo test -p but-api --test ${file%.rs} || exit 1; done"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "Structural pairing checker fails closed on unflagged helper scopes is true",
      "maps_to_ac": "AC-4",
      "verify": "cargo test -p but-api --test gate_registry_swap -- all_but_agent_handle_env_helpers_are_flag_gated"
    }
  ]
}
-->
