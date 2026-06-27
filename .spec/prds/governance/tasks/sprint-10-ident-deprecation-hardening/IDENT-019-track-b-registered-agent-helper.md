# IDENT-019 — Add registry-only Track B coverage under `tests/agent_registry.rs` plus governed CLI operator proof for register/env-fallback/deny-default

**Sprint:** [Sprint 10](./SPRINT.md) · **Agent:** `rust-implementer` · **Estimate:** 180 min · **Type:** FEATURE · **Status:** Complete · **Proposed By:** rust-planner

## Background

rust-implementer owns the but-api test surface and must add the helper + new Track B tests — this is new test infrastructure, not mechanical migration..

**Why it matters.** Closes the IDENT deprecation arc: on a governed repo the registry path is the default; the env-var path survives only as an opt-in escape hatch behind `BUT_AUTHZ_ALLOW_ENV_HANDLE=1`.

**Provides:** Registry-only Track B tests in tests/agent_registry.rs exercising the registry path, explicit register→success→unregister→denied coverage for commit/merge/admin/forge surfaces, the flag-set governed commit env-fallback proof, expired-entry and spoofed/stale pid/start_time denial at real governed but-api gates, and a CLI blackbox operator sequence using `but agent register`/`unregister`

**Consumes:** Sprint-08 `Registry` (IDENT-001), Sprint-08 `resolve_principal_with_registry` (IDENT-003), Sprint-09 IDENT-010 runtime registry wrapper callsite migration, IDENT-017 verified resolver

**Boundary contracts:**
- Registry-only setup seeds the runtime registry via `BUT_AGENT_REGISTRY_PATH`
- Track B tests call the real but-api gates with registry-only setup; the gates resolve through
  `resolve_principal_with_runtime_registry`
- No env var handling in Track B tests
- Unregistering the current process returns governed commit/merge gates to `perm.denied`
- Empty registry + `BUT_AGENT_HANDLE=dev` + `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` still satisfies the governed commit gate
- Expired current-process registry entries deny at a real governed but-api gate with `BUT_AGENT_HANDLE` and `BUT_AUTHZ_ALLOW_ENV_HANDLE` unset, or the runtime registry load/gate path proves `Registry::gc(now)` runs before resolution
- Wrong `start_time` for the current pid returns stale/unregistered `perm.denied` at a real governed but-api gate
- Wrong pid/current start_time mismatch cannot authorize at a real governed but-api gate
- CLI blackbox sequence proves the operator path: env-only commit denied, flag-set env fallback succeeds, `but agent register --as dev` succeeds, registered commit succeeds without env, unregister returns env-only commit to `perm.denied`


## Critical Constraints

**MUST:**
- Add a registry-only helper/setup to but-api tests (local `RegistryEnv` is acceptable; a shared `with_registered_agent` helper is acceptable if the implementer chooses to extract it)
- The helper/setup MUST seed the runtime registry via `BUT_AGENT_REGISTRY_PATH` pointing at a tempfile, writing (pid, start_time, agent_id, ttl)
- The helper/setup MUST use Registry::write for atomic persistence (write-to-temp + rename)
- Create or update `tests/agent_registry.rs` with the current real Track B tests: `commit_surface`, `merge_surface`, `admin_write_surface`, and `forge_review_surface`
- Prove register→success→unregister→denied for commit and merge, with `BUT_AGENT_HANDLE` unset and `BUT_AUTHZ_ALLOW_ENV_HANDLE` unset
- Add explicit governed commit env-fallback coverage: empty registry + `BUT_AGENT_HANDLE=dev` + `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` → commit gate succeeds (`env_fallback_still_allowed_on_registry_miss`)
- Add explicit env-only merge denial coverage: `BUT_AGENT_HANDLE=maint`, no registry hit, `BUT_AUTHZ_ALLOW_ENV_HANDLE` unset → `perm.denied`
- Add a CLI blackbox test using the real `but` harness that runs the exact operator sequence from SPRINT.md against a governed fixture
- Add `expired_current_process_registry_entry_denied`: a current-pid registry entry whose `expires_at` is in the past must deny `perm.denied` at a real governed but-api commit gate with `BUT_AGENT_HANDLE` and `BUT_AUTHZ_ALLOW_ENV_HANDLE` unset; if the product fix is runtime load-path GC, the test must prove `Registry::gc(now)` runs before resolution
- Add `current_pid_wrong_start_time_denied_at_commit_gate`: a registry entry for the current pid with the wrong `start_time` must return stale/unregistered `perm.denied` at a real governed but-api commit gate
- Add `wrong_pid_current_start_time_denied_at_commit_gate`: a registry entry with the wrong pid but current process start_time must not authorize at a real governed but-api commit gate
- Track B success tests MUST clear `BUT_AGENT_HANDLE` and `BUT_AUTHZ_ALLOW_ENV_HANDLE` — they exercise the registry path directly
- Tests MUST pass with `cargo test -p but-api --test agent_registry`
- Use `but_testsupport::writable_scenario` for repo fixtures, never `std::env::temp_dir().join(format!(...))`

**NEVER:**
- Reuse BUT_AGENT_HANDLE or temp_env in Track B tests — the point is to bypass env fallback
- Modify existing Track A tests (IDENT-018 owns those)
- Make registry-only helper/setup part of the public but-api API — it's test infrastructure only
- Use unwrap() or expect() in the helper — propagate Result
- Touch but-api/src/** except the minimal runtime registry wrapper/load-path change needed to prune expired entries before resolution
- Add registry logic to production code paths except the minimal stale/expired enforcement needed for the real gate tests

**STRICTLY:**
- BLOCKED-UNTIL Sprint-09 IDENT-010 completes (callsites must resolve through `resolve_principal_with_runtime_registry`, which loads the runtime registry and delegates to `but_authz::resolve_principal_with_registry(Some(&registry), cfg)`, before the registry path is exercised)
- BLOCKED-UNTIL Sprint-09 IDENT-009 completes (AGENTS_PATH must exist for agents.toml loading)
- BLOCKED-UNTIL IDENT-017 completes (resolver deny-default must be verified)
- Track B tests are NEW tests — do NOT modify existing tests
- Registry-only helper/setup is test-only — not part of the public API
- Track B tests are REPRESENTATIVE coverage of the registry path (commit, merge, admin-write, forge-review), but commit register→success→unregister→denied, flag-set commit env fallback, CLI operator register/unregister, env-only merge denial, expired-entry denial, and stale/spoofed pid/start_time denial are Sprint-10 blocking requirements and MUST be proven here.

## Specification

**Objective:** Add registry-only Track B tests in tests/agent_registry.rs that exercise the registry path directly, bypassing env fallback, and add governed commit env-fallback, expired/stale process-identity denial, plus CLI blackbox operator proofs for the Sprint 10 human gate.

**Success state:** tests/agent_registry.rs exists with the real current tests `commit_surface`, `merge_surface`, `admin_write_surface`, and `forge_review_surface` using registry-only setup. The setup seeds the runtime registry via BUT_AGENT_REGISTRY_PATH tempfile. `cargo test -p but-api --test agent_registry` passes. Commit and merge registered paths succeed without env/flag, unregister returns to `perm.denied`, governed commit env fallback succeeds only with `BUT_AUTHZ_ALLOW_ENV_HANDLE=1`, expired current-process registry entries deny or are GC'd before resolution, wrong pid/start_time identities deny, the CLI operator sequence passes, and merge env-only without the flag denies `perm.denied`.

## Acceptance Criteria

**AC-1 (PRIMARY)** — PRIMARY — `agent_registry` Track B suite exercises registry-only setup across current surfaces
- **GIVEN:** A governed repo and a runtime registry tempfile seeded through `BUT_AGENT_REGISTRY_PATH`
- **WHEN:** Running the `agent_registry` integration test target
- **THEN:** `commit_surface`, `merge_surface`, `admin_write_surface`, and `forge_review_surface` all pass through registry-only setup and return `perm.denied` after unregister
- **Verify:** `cargo test -p but-api --test agent_registry`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-api · **FLOW_REF:** UC-IDENT-03
- **Scenario:** `start_ref=agent_registry_surface_suite`; `must_observe` = ['4 registry-only surface tests exit with code 0', 'stdout names `commit_surface`, `merge_surface`, `admin_write_surface`, and `forge_review_surface` as passed', 'each surface prints literal `perm.denied` after unregister']; `must_not_observe` = ['0 registry-only tests executed', 'any surface test failure', 'empty registry accepted after unregister']; `negative_control.would_fail_if` = ['agent_registry.rs test target is stubbed to run 0 tests', 'RegistryEnv::registered omits Registry::write', 'unregister is removed or no-op'].

**AC-2** — Track B test: commit surface register→success→unregister→denied
- **GIVEN:** A governed repo + registry-only setup registering `dev`
- **WHEN:** Calling `enforce_commit_gate_for_target` inside the closure
- **THEN:** Returns Ok(()) via registry hit, NOT env fallback
- **Verify:** `cargo test -p but-api --test agent_registry -- commit_surface`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-api · **FLOW_REF:** UC-IDENT-03
- **Scenario:** `start_ref=governed_repo_commit_registry_cycle`; `must_observe` = ['registered commit call returns exact `Ok(())`', 'after unregister commit denial code is literal `perm.denied`', 'denial message contains literal `pid ` and `start_time` for the current process']; `must_not_observe` = ['Ok(()) after unregister', 'env fallback accepted', 'empty registry accepted after unregister']; `negative_control.would_fail_if` = ['unregister is a no-op', 'commit gate still accepts env-only identity', 'denial code changes from perm.denied'].

**AC-3** — Track B test: merge surface register→success→unregister→denied
- **GIVEN:** A governed merge review + registry-only setup registering `merger`
- **WHEN:** Calling `enforce_merge_gate` inside the closure
- **THEN:** Returns Ok(()) via registry hit
- **Verify:** `cargo test -p but-api --test agent_registry -- merge_surface`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-api · **FLOW_REF:** UC-IDENT-03
- **Scenario:** `start_ref=governed_repo_merge_registry_cycle`; `must_observe` = ['registered merge call returns exact `Ok(())`', 'after unregister merge denial code is literal `perm.denied`', 'denial message contains literal `pid ` and `start_time` for the current process']; `must_not_observe` = ['Ok(()) after unregister', 'env fallback accepted', 'empty registry accepted after unregister']; `negative_control.would_fail_if` = ['unregister is a no-op', 'merge gate still accepts env-only identity', 'denial code changes from perm.denied'].

**AC-4** — Track B test: admin-write surface register→success→unregister→denied
- **GIVEN:** A governed repo + registry entry via registry-only setup as `admin`
- **WHEN:** Calling `enforce_administration_write_gate` inside the registered phase, then unregistering the current process and calling again
- **THEN:** Registered call returns `Ok(())`; after unregister the same surface returns `perm.denied`
- **Verify:** `cargo test -p but-api --test agent_registry -- admin_write_surface`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-api · **FLOW_REF:** UC-IDENT-03
- **Scenario:** `start_ref=governed_repo_admin_registry_cycle`; `must_observe` = ['registered admin-write call returns exact `Ok(())`', 'after unregister admin-write denial code is literal `perm.denied`', 'denial message contains literal `pid ` and `start_time` for the current process']; `must_not_observe` = ['Ok(()) after unregister', 'env fallback accepted', 'empty registry accepted after unregister']; `negative_control.would_fail_if` = ['unregister is a no-op', 'admin-write gate still accepts env-only identity', 'denial code changes from perm.denied'].

**AC-5** — Track B test: forge review surface register→success→unregister→denied
- **GIVEN:** A governed forge review fixture + registry-only setup registering `reviewer`
- **WHEN:** Calling `authorize_branch_action` with ReviewsWrite inside the closure
- **THEN:** Returns Ok(Some(principal)) via registry hit
- **Verify:** `cargo test -p but-api --test agent_registry -- forge_review_surface`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-api · **FLOW_REF:** UC-IDENT-03
- **Scenario:** `start_ref=governed_repo_forge_registry_cycle`; `must_observe` = ['registered forge review writes exactly 1 approval verdict', 'approval verdict principal_id is literal `reviewer`', 'after unregister forge review denial code is literal `perm.denied`']; `must_not_observe` = ['0 approval verdicts', 'env fallback accepted', 'empty registry accepted after unregister']; `negative_control.would_fail_if` = ['unregister is a no-op', 'forge review gate still accepts env-only identity', 'approval attribution is hardcoded or static'].

**AC-6** — Governed commit env fallback remains explicitly allowed when registry misses and flag is set
- **GIVEN:** A governed repo with `dev` authorized for contents write, an empty runtime registry, `BUT_AGENT_HANDLE=dev`, and `BUT_AUTHZ_ALLOW_ENV_HANDLE=1`
- **WHEN:** `enforce_commit_gate_for_target` is called
- **THEN:** The governed commit gate succeeds via the flag-gated env fallback
- **Verify:** `cargo test -p but-api --test gate_registry_swap -- env_fallback_still_allowed_on_registry_miss`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-api · **FLOW_REF:** UC-IDENT-03
- **Scenario:** `start_ref=governed_commit_env_fallback_flag_set`; `must_observe` = ['governed commit call returns exact Ok(())', 'BUT_AGENT_HANDLE=dev is accepted only because BUT_AUTHZ_ALLOW_ENV_HANDLE=1', 'runtime registry has 0 entries']; `must_not_observe` = ['perm.denied', 'branch.protected', 'empty registry accepted without flag']; `negative_control.would_fail_if` = ['flag-gated fallback removed', 'test uses a registry hit instead of empty registry', 'commit gate still reads env directly and ignores flag'].

**AC-7** — Merge env-only denial proves every gate surface is not commit-only
- **GIVEN:** A governed merge review, no registry hit, `BUT_AGENT_HANDLE=maint`, and `BUT_AUTHZ_ALLOW_ENV_HANDLE` unset
- **WHEN:** `enforce_merge_gate` is called
- **THEN:** The merge attempt returns `perm.denied` rather than accepting the env-only handle
- **Verify:** `cargo test -p but-api --test gate_registry_swap -- merge_gate_env_only_without_flag_denied`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-api · **FLOW_REF:** UC-IDENT-03
- **Scenario:** `start_ref=governed_merge_env_only_no_flag`; `must_observe` = ['merge denial code is literal perm.denied', 'denial message contains literal `pid ` and `start_time` for the unregistered current process', 'BUT_AGENT_HANDLE=maint is present but ignored without flag']; `must_not_observe` = ['merge returns Ok(())', 'env fallback accepted', 'empty/start: registry (0 entries) accepted', 'BUT_AUTHZ_ALLOW_ENV_HANDLE=1']; `negative_control.would_fail_if` = ['merge gate still reads env-only handle directly', 'resolver bypasses runtime registry', 'flag check removed'].

**AC-8** — CLI blackbox operator sequence proves the exact Sprint human gate
- **GIVEN:** A governed CLI fixture with `dev` authorized for contents write and an empty runtime registry
- **WHEN:** The blackbox test runs: `BUT_AGENT_HANDLE=dev but commit` denied, `BUT_AUTHZ_ALLOW_ENV_HANDLE=1 BUT_AGENT_HANDLE=dev but commit` succeeds, `but agent register --as dev` registers the current process, `but commit` succeeds without env, `but agent unregister` removes the current process, and env-only commit is denied again
- **THEN:** The operator path matches SPRINT.md steps 1-4 with real CLI commands rather than direct but-api calls
- **Verify:** `cargo test -p but --test but --features legacy,but-2 -- command::commit_gate::commit_gate_operator_runtime_registry_sequence`
- **TEST_TIER:** e2e · **VERIFICATION_SERVICE:** but-cli · **FLOW_REF:** UC-IDENT-03
- **Scenario:** `start_ref=governed_cli_operator_fixture`; `must_observe` = ['first env-only commit stderr contains literal perm.denied', 'flag-set env-fallback commit exits 0', 'agent register stdout contains registered dev pid/start_time tuple', 'registered no-env commit exits 0', 'post-unregister env-only commit stderr contains literal perm.denied']; `must_not_observe` = ['registered no-env commit denied', 'env-only commit succeeds without flag', 'agent register skipped', 'unregister skipped']; `negative_control.would_fail_if` = ['CLI command bypasses commit gate', 'agent register writes no runtime registry entry', 'unregister is a no-op', 'test uses direct but-api calls instead of env.but(...)'].

**AC-9** — Expired current-process registry entry denies at a real governed but-api gate
- **GIVEN:** A governed commit fixture with `dev` authorized, a runtime registry entry for the current pid/current start_time whose `expires_at` is in the past, and both `BUT_AGENT_HANDLE` and `BUT_AUTHZ_ALLOW_ENV_HANDLE` unset
- **WHEN:** `enforce_commit_gate_for_target` resolves identity through `resolve_principal_with_runtime_registry`
- **THEN:** The gate returns stale/unregistered `perm.denied`, or the runtime registry load/gate path proves `Registry::gc(now)` ran before resolution and removed the expired entry
- **Verify:** `cargo test -p but-api --test gate_registry_swap -- expired_current_process_registry_entry_denied`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-api · **FLOW_REF:** UC-IDENT-03
- **Scenario:** `start_ref=governed_commit_expired_current_process_registry_entry`; `must_observe` = ['expired entry denial code is literal `perm.denied`', 'runtime registry load path reports expired entries pruned before resolution or resolver rejects `expires_at < now`', '`BUT_AGENT_HANDLE=unset` and `BUT_AUTHZ_ALLOW_ENV_HANDLE=unset`']; `must_not_observe` = ['expired entry authorizes Ok(())', 'env fallback accepted', 'success-only registry test passes without expired case']; `negative_control.would_fail_if` = ['Registry::resolve ignores expires_at and authorizes the expired entry', 'test uses direct resolver instead of real commit gate', 'BUT_AGENT_HANDLE is set and masks the expired registry failure'].

**AC-10** — Current pid with wrong start_time denies as stale/unregistered at a real gate
- **GIVEN:** A governed commit fixture with a runtime registry entry keyed by the current pid but a deliberately wrong process start_time, and both env escape variables unset
- **WHEN:** `enforce_commit_gate_for_target` resolves identity through the runtime registry wrapper
- **THEN:** The gate returns stale/unregistered `perm.denied`; the stale key cannot authorize the current process
- **Verify:** `cargo test -p but-api --test gate_registry_swap -- current_pid_wrong_start_time_denied_at_commit_gate`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-api · **FLOW_REF:** UC-IDENT-03
- **Scenario:** `start_ref=governed_commit_current_pid_wrong_start_time_entry`; `must_observe` = ['wrong-start-time denial code is literal `perm.denied`', 'denial message contains literal `pid ` and `start_time` for the current process', 'registry entry key uses tuple `(current_pid, stale_start_time)`']; `must_not_observe` = ['wrong start_time authorizes Ok(())', 'env fallback accepted', 'stale process identity treated as current']; `negative_control.would_fail_if` = ['Registry lookup matches only pid and ignores start_time', 'test registers the real current start_time by mistake', 'commit gate bypasses runtime process identity check'].

**AC-11** — Wrong pid/current start_time mismatch cannot authorize at a real gate
- **GIVEN:** A governed commit fixture with a runtime registry entry keyed by a wrong pid and the current process start_time, and both env escape variables unset
- **WHEN:** `enforce_commit_gate_for_target` resolves identity through the runtime registry wrapper
- **THEN:** The gate returns stale/unregistered `perm.denied`; a mismatched pid/start_time tuple cannot authorize
- **Verify:** `cargo test -p but-api --test gate_registry_swap -- wrong_pid_current_start_time_denied_at_commit_gate`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-api · **FLOW_REF:** UC-IDENT-03
- **Scenario:** `start_ref=governed_commit_wrong_pid_current_start_time_entry`; `must_observe` = ['wrong-pid denial code is literal `perm.denied`', 'denial message contains literal `pid ` and `start_time` for the current process', 'registry entry key uses tuple `(wrong_pid, current_start_time)`']; `must_not_observe` = ['wrong pid authorizes Ok(())', 'env fallback accepted', 'stale process identity treated as current']; `negative_control.would_fail_if` = ['Registry lookup matches only start_time and ignores pid', 'test registers the real current pid by mistake', 'commit gate bypasses runtime process identity check'].

## Test Criteria

| ID | Boolean | Maps to AC |
|----|---------|------------|
| TC-1 | agent_registry Track B suite passes with registry-only setup is true | AC-1 |
| TC-2 | Commit surface register→success→unregister→denied test passes is true | AC-2 |
| TC-3 | Merge surface register→success→unregister→denied test passes is true | AC-3 |
| TC-4 | Admin-write surface register→success→unregister→denied test passes is true | AC-4 |
| TC-5 | Forge review surface register→success→unregister→denied test passes is true | AC-5 |
| TC-6 | Governed commit env fallback with empty registry and flag set passes is true | AC-6 |
| TC-7 | Merge env-only without flag denial test passes is true | AC-7 |
| TC-8 | CLI operator register/env-fallback/unregister sequence passes is true | AC-8 |
| TC-9 | Expired current-process registry entry is denied at a real governed commit gate is true | AC-9 |
| TC-10 | Current pid with wrong start_time is denied stale/unregistered at a real governed commit gate is true | AC-10 |
| TC-11 | Wrong pid/current start_time mismatch cannot authorize at a real governed commit gate is true | AC-11 |

## Reading List

1. `crates/but-api/tests/agent_registry.rs:1-120` — Current Track B surface tests (`commit_surface`, `merge_surface`, `admin_write_surface`, `forge_review_surface`) and registry-only setup pattern
2. `crates/but-authz/src/registry.rs:1-140` — Registry::write, Registry::register, Registration.expires_at, and Registry::gc(now) APIs — helper must use atomic persistence and expired-entry tests must prove stale entries cannot authorize
3. `crates/but-api/tests/gate_registry_swap.rs:1-360` — Existing register→success→unregister→denied gate-surface suite and home for env fallback / env-only denial tests. Current grounded test names include `commit_gate_registered_process_allowed_then_unregistered_denied`, `branch_gates_read_registered_process_allowed_then_unregistered_denied`, `group_list_registered_process_allowed_then_unregistered_denied`, `perm_list_registered_process_allowed_then_unregistered_denied`, `governance_status_read_registered_then_unregistered_empty`, `workspace_rules_scoped_for_caller_registered_then_unregistered_denied`, `admin_write_gate_registered_process_allowed_then_unregistered_denied`, `merge_gate_registered_process_allowed_then_unregistered_denied`, `whoami_registered_process_allowed_then_unregistered_denied`, `can_i_registered_process_allowed_then_unregistered_denied`, `forge_review_registered_process_allowed_then_unregistered_denied`, `env_fallback_still_allowed_on_registry_miss`, plus malformed/unreadable registry tests.
4. `crates/but/tests/but/command/commit_gate.rs:1-220` — CLI blackbox harness for governed commit tests; add `commit_gate_operator_runtime_registry_sequence`
5. `crates/but/tests/but/command/agent.rs:100-190` — Real `but agent register` / `unregister` command patterns and snapshots

## Guardrails

**WRITE-ALLOWED:**
- crates/but-api/tests/agent_registry.rs (NEW/MODIFY — Track B tests using registry-only setup)
- crates/but-api/tests/gate_registry_swap.rs (MODIFY — add merge env-only no-flag denial test if not already present; add `expired_current_process_registry_entry_denied`, `current_pid_wrong_start_time_denied_at_commit_gate`, and `wrong_pid_current_start_time_denied_at_commit_gate`)
- crates/but/tests/but/command/commit_gate.rs (MODIFY — add CLI blackbox test `commit_gate_operator_runtime_registry_sequence`)
- crates/but/tests/but/command/snapshots/commit_gate/** (ADD/MODIFY — snapshots only if the new CLI blackbox test needs them)
- crates/but-testsupport/src/lib.rs or new test-utils module (ADD — optional shared with_registered_agent helper, dev-deps only; local test helper is also acceptable)
- crates/but-api/src/commit/gate.rs (MODIFY — only if needed to run runtime registry load-path GC before delegating to `but_authz::resolve_principal_with_registry`; no callsite churn)
- crates/but-authz/src/registry.rs (MODIFY — only if needed for minimal expired-entry/stale-key fail-closed behavior; preserve atomic write and exact-key identity semantics)

**WRITE-PROHIBITED:**
- crates/but-api/src/** EXCEPT `crates/but-api/src/commit/gate.rs` runtime registry wrapper/load-path GC if required by AC-9 — Do NOT touch unrelated production code
- crates/but-api/tests/* EXCEPT agent_registry.rs and gate_registry_swap.rs — Track A tests are IDENT-018's responsibility
- Public but-api API — registry-only helper/setup is test infrastructure only
- crates/but-authz/** EXCEPT minimal `crates/but-authz/src/registry.rs` expired-entry/stale-key fail-closed behavior if required by AC-9/AC-10/AC-11
- crates/but/src/** — CLI production behavior is already present; the new proof is blackbox test-only

## Code Pattern

**Reference:** IDENT-018 owns Track A (env fallback) tests; Sprint-08 IDENT-001 shipped Registry::write; UC-IDENT-03 specifies the registry-first resolution path

**Pattern:** Test-helper pattern: registry-only setup writes Registry::write output to a tempfile, sets BUT_AGENT_REGISTRY_PATH, clears BUT_AGENT_HANDLE/BUT_AUTHZ_ALLOW_ENV_HANDLE for success paths, executes the gate call, and cleans up. Tests call gates inside the registered scope.

**Source:** `but_testsupport writable_scenario pattern — tempfile-based test fixtures`

**Design notes:**
- Track B tests bypass env fallback entirely — no BUT_AGENT_HANDLE, no temp_env
- Registry-only helper/setup is dev-deps or test-local only — not part of public API
- After Sprint-09 IDENT-010, gates call `resolve_principal_with_runtime_registry`, which loads
  the runtime registry and delegates to `but_authz::resolve_principal_with_registry(Some(&registry), cfg)`
- Expired-entry coverage must not be a direct `Registry::resolve` unit test only; it must prove real governed gate behavior or prove runtime load-path GC before resolution
- Spoofed/stale process-identity coverage must exercise pid/start_time tuple mismatches at the governed gate, with env fallback disabled
- Helper uses BUT_AGENT_REGISTRY_PATH env var to point tests at a tempfile registry

**Anti-pattern:** Do NOT reuse Track A patterns (temp_env). Do NOT make helper part of public API. Do NOT use unwrap().

## Agent Instructions

TDD RED→GREEN per AC (integration against the real crate — `but-authz` / `but-api` — real git/gitoxide, NO mocks):
1. **RED:** write each AC's failing test first (against the live code / current start state).
2. **GREEN:** make the minimal change (test-only for IDENT-017/018/019; invariant assertions for IDENT-020; doc-comments for IDENT-021).
3. Run `cargo fmt`, `cargo clippy -p <crate> --all-targets -- -D warnings`, then the task's verify commands.
4. Commit via `but commit` (governed). Note: this task is BLOCKED-UNTIL Sprint-09 IDENT-009/010/011 land.

## Orchestrator Verification Protocol

- `cargo test -p but-api --test agent_registry` → exit 0; `commit_surface`, `merge_surface`, `admin_write_surface`, and `forge_review_surface` pass
- `cargo test -p but-api --test agent_registry -- commit_surface` → exit 0; output/evidence shows registered Ok(()) then unregister → perm.denied
- `cargo test -p but-api --test gate_registry_swap -- env_fallback_still_allowed_on_registry_miss` → exit 0; empty registry + `BUT_AGENT_HANDLE=dev` + flag-set commit gate succeeds
- `cargo test -p but-api --test gate_registry_swap -- merge_gate_env_only_without_flag_denied` → exit 0; merge env-only without flag denies perm.denied
- `cargo test -p but-api --test gate_registry_swap -- expired_current_process_registry_entry_denied` → exit 0; expired current-process registry entry with env unset denies perm.denied or is GC'd before resolution
- `cargo test -p but-api --test gate_registry_swap -- current_pid_wrong_start_time_denied_at_commit_gate` → exit 0; current pid with wrong start_time denies perm.denied
- `cargo test -p but-api --test gate_registry_swap -- wrong_pid_current_start_time_denied_at_commit_gate` → exit 0; wrong pid/current start_time mismatch denies perm.denied
- `cargo test -p but --test but --features legacy,but-2 -- command::commit_gate::commit_gate_operator_runtime_registry_sequence` → exit 0; exact CLI operator sequence passes
- `cargo check -p but-api --all-targets` → exit 0

## Agent Assignment

**Agent:** `rust-implementer` — rust-implementer owns the but-api test surface and must add the helper + new Track B tests — this is new test infrastructure, not mechanical migration.
**Pairing:** none (single-surface Rust task). Honors `crates/AGENTS.md` + `crates/WORKSPACE_MODEL.md`.

## Evidence Gates

- `cargo test -p but-api --test agent_registry` (exit 0, all 4 current registry path tests pass)
- `cargo test -p but-api --test agent_registry -- commit_surface` (exit 0; registered Ok(()) then unregister → perm.denied)
- `cargo test -p but-api --test agent_registry -- merge_surface` (exit 0; registered Ok(()) then unregister → perm.denied)
- `cargo test -p but-api --test agent_registry -- admin_write_surface` (exit 0; registered Ok(()) then unregister → perm.denied)
- `cargo test -p but-api --test agent_registry -- forge_review_surface` (exit 0; registered approval then unregister → perm.denied)
- `cargo test -p but-api --test gate_registry_swap -- env_fallback_still_allowed_on_registry_miss` (exit 0; empty registry + flag-set env fallback succeeds)
- `cargo test -p but-api --test gate_registry_swap -- merge_gate_env_only_without_flag_denied` (exit 0; merge env-only without flag denies perm.denied)
- `cargo test -p but-api --test gate_registry_swap -- expired_current_process_registry_entry_denied` (exit 0; expired current-process registry entry with env unset denies perm.denied or is GC'd before resolution)
- `cargo test -p but-api --test gate_registry_swap -- current_pid_wrong_start_time_denied_at_commit_gate` (exit 0; current pid with wrong start_time denies perm.denied)
- `cargo test -p but-api --test gate_registry_swap -- wrong_pid_current_start_time_denied_at_commit_gate` (exit 0; wrong pid/current start_time mismatch denies perm.denied)
- `cargo test -p but --test but --features legacy,but-2 -- command::commit_gate::commit_gate_operator_runtime_registry_sequence` (exit 0; exact CLI operator sequence passes)
- `cargo check -p but-api --all-targets` (exit 0)

## Review Criteria

- AC-1: PRIMARY — `agent_registry` Track B suite exercises registry-only setup across current surfaces — verified by `cargo test -p but-api --test agent_registry`.
- AC-2: Track B test: commit surface register→success→unregister→denied — verified by `cargo test -p but-api --test agent_registry -- commit_surface`.
- AC-3: Track B test: merge surface register→success→unregister→denied — verified by `cargo test -p but-api --test agent_registry -- merge_surface`.
- AC-4: Track B test: admin-write surface register→success→unregister→denied — verified by `cargo test -p but-api --test agent_registry -- admin_write_surface`.
- AC-5: Track B test: forge review surface register→success→unregister→denied — verified by `cargo test -p but-api --test agent_registry -- forge_review_surface`.
- AC-6: Governed commit env fallback remains explicitly allowed when registry misses and flag is set — verified by `cargo test -p but-api --test gate_registry_swap -- env_fallback_still_allowed_on_registry_miss`.
- AC-7: Merge env-only denial proves every gate surface is not commit-only — verified by `cargo test -p but-api --test gate_registry_swap -- merge_gate_env_only_without_flag_denied`.
- AC-8: CLI blackbox operator sequence proves the exact Sprint human gate — verified by `cargo test -p but --test but --features legacy,but-2 -- command::commit_gate::commit_gate_operator_runtime_registry_sequence`.
- AC-9: Expired current-process registry entry denies at a real governed but-api gate — verified by `cargo test -p but-api --test gate_registry_swap -- expired_current_process_registry_entry_denied`.
- AC-10: Current pid with wrong start_time denies as stale/unregistered at a real gate — verified by `cargo test -p but-api --test gate_registry_swap -- current_pid_wrong_start_time_denied_at_commit_gate`.
- AC-11: Wrong pid/current start_time mismatch cannot authorize at a real gate — verified by `cargo test -p but-api --test gate_registry_swap -- wrong_pid_current_start_time_denied_at_commit_gate`.
- Honors NEVER: Reuse BUT_AGENT_HANDLE or temp_env in Track B tests — the point is to bypass env fallback.

## Dependencies

- **Depends on:** Sprint-09 IDENT-010 (callsites must resolve through `resolve_principal_with_runtime_registry`, which loads the runtime registry and delegates to `but_authz::resolve_principal_with_registry(Some(&registry), cfg)`, before registry path behavior is exercised), Sprint-09 IDENT-009 (AGENTS_PATH must exist for agents.toml loading), IDENT-017 (resolver deny-default verified), Sprint-08 IDENT-001 (Registry exists), Sprint-08 IDENT-003 (resolve_principal_with_registry exists)
- **Blocks:** IDENT-020 (invariant extension requires Track B tests passing), IDENT-021 (doc audit requires complete test coverage)
- **Capabilities:** CAP-AUTHZ-01

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "IDENT-019",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "agent_registry_surface_suite": {
      "description": "crates/but-api/tests/agent_registry.rs with registry-only surface tests for commit, merge, admin-write, and forge review",
      "seed_method": "public_api",
      "records": [
        "agent_registry.rs contains fn commit_surface",
        "agent_registry.rs contains fn merge_surface",
        "agent_registry.rs contains fn admin_write_surface",
        "agent_registry.rs contains fn forge_review_surface",
        "RegistryEnv::registered writes a runtime registry tempfile through Registry::write"
      ]
    },
    "governed_repo_commit_registry_cycle": {
      "description": "governed repo with dev authorized for contents:write; runtime registry can register then unregister the current process",
      "seed_method": "public_api",
      "records": [
        "agents.toml or permissions.toml committed with [[agent/principal]] id=dev permissions=[contents:write]",
        "BUT_AGENT_HANDLE unset",
        "BUT_AUTHZ_ALLOW_ENV_HANDLE unset",
        "current process registry entry can be removed after first successful commit gate call"
      ]
    },
    "governed_repo_merge_registry_cycle": {
      "description": "governed merge review with merger authorized for merge; runtime registry can register then unregister current process",
      "seed_method": "public_api",
      "records": [
        "review_id=1 exists for refs/heads/feat",
        "agents.toml or permissions.toml includes merger with merge authority",
        "BUT_AGENT_HANDLE unset",
        "BUT_AUTHZ_ALLOW_ENV_HANDLE unset"
      ]
    },
    "governed_repo_admin_registry_cycle": {
      "description": "governed repo with admin authorized for administration:write; runtime registry can register then unregister current process",
      "seed_method": "public_api",
      "records": [
        "governance config committed at target ref",
        "admin has administration:write authority",
        "BUT_AGENT_HANDLE unset",
        "BUT_AUTHZ_ALLOW_ENV_HANDLE unset"
      ]
    },
    "governed_repo_forge_registry_cycle": {
      "description": "governed forge review fixture with reviewer authorized for reviews:write; runtime registry can register then unregister current process",
      "seed_method": "public_api",
      "records": [
        "review target feat exists",
        "reviewer has reviews:write authority",
        "BUT_AGENT_HANDLE unset",
        "BUT_AUTHZ_ALLOW_ENV_HANDLE unset"
      ]
    },
    "governed_commit_env_fallback_flag_set": {
      "description": "governed commit fixture with empty runtime registry, BUT_AGENT_HANDLE=dev, and BUT_AUTHZ_ALLOW_ENV_HANDLE=1",
      "seed_method": "public_api",
      "records": [
        "dev has contents:write authority at target ref",
        "runtime registry file exists with 0 entries",
        "BUT_AGENT_HANDLE=dev",
        "BUT_AUTHZ_ALLOW_ENV_HANDLE=1"
      ]
    },
    "governed_merge_env_only_no_flag": {
      "description": "governed merge review with BUT_AGENT_HANDLE=maint, empty registry, and BUT_AUTHZ_ALLOW_ENV_HANDLE unset",
      "seed_method": "public_api",
      "records": [
        "merge review exists with review_id=1",
        "maint has merge authority in committed governance config",
        "runtime registry file is empty or missing",
        "BUT_AGENT_HANDLE=maint",
        "BUT_AUTHZ_ALLOW_ENV_HANDLE unset"
      ]
    },
    "governed_cli_operator_fixture": {
      "description": "but CLI blackbox fixture with dev authorized for contents:write and an empty runtime registry path",
      "seed_method": "cli",
      "records": [
        "Sandbox fixture has committed governance config authorizing dev for contents:write",
        "BUT_AGENT_REGISTRY_PATH points at an empty test registry path",
        "command harness uses env.but(...) to run real but commands",
        "new test function is command::commit_gate::commit_gate_operator_runtime_registry_sequence"
      ]
    },
    "governed_commit_expired_current_process_registry_entry": {
      "description": "governed commit fixture with dev authorized; runtime registry has current pid/current start_time entry whose expires_at is in the past; env fallback vars unset",
      "seed_method": "public_api",
      "records": [
        "dev has contents:write authority at target ref",
        "runtime registry entry key is current pid/current start_time",
        "Registration.expires_at is earlier than test now",
        "BUT_AGENT_HANDLE unset",
        "BUT_AUTHZ_ALLOW_ENV_HANDLE unset"
      ]
    },
    "governed_commit_current_pid_wrong_start_time_entry": {
      "description": "governed commit fixture with runtime registry entry for current pid but deliberately wrong process start_time; env fallback vars unset",
      "seed_method": "public_api",
      "records": [
        "dev has contents:write authority at target ref",
        "registry entry key uses current pid",
        "registry entry start_time is not the current process start_time",
        "BUT_AGENT_HANDLE unset",
        "BUT_AUTHZ_ALLOW_ENV_HANDLE unset"
      ]
    },
    "governed_commit_wrong_pid_current_start_time_entry": {
      "description": "governed commit fixture with runtime registry entry for wrong pid but current process start_time; env fallback vars unset",
      "seed_method": "public_api",
      "records": [
        "dev has contents:write authority at target ref",
        "registry entry pid is not the current process pid",
        "registry entry start_time is the current process start_time",
        "BUT_AGENT_HANDLE unset",
        "BUT_AUTHZ_ALLOW_ENV_HANDLE unset"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "PRIMARY \u2014 `agent_registry` Track B suite exercises registry-only setup across current surfaces",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-api --test agent_registry",
      "flow_ref": "UC-IDENT-03",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "agent_registry_surface_suite",
        "must_observe": [
          "4 registry-only surface tests exit with code 0",
          "stdout names `commit_surface`, `merge_surface`, `admin_write_surface`, and `forge_review_surface` as passed",
          "each surface prints literal `perm.denied` after unregister"
        ],
        "must_not_observe": [
          "0 registry-only tests executed",
          "any surface test failure",
          "empty registry accepted after unregister"
        ],
        "negative_control": {
          "would_fail_if": [
            "agent_registry.rs test target is stubbed to run 0 tests",
            "RegistryEnv::registered omits Registry::write",
            "unregister is removed or no-op"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "agent_registry_surface_suite",
            "action": {
              "actor": "test_harness",
              "steps": [
                "Run cargo test -p but-api --test agent_registry",
                "Inspect test output for the four current surface test names"
              ]
            },
            "end_state": {
              "must_observe": [
                "4 registry-only surface tests exit with code 0",
                "stdout names `commit_surface`, `merge_surface`, `admin_write_surface`, and `forge_review_surface` as passed",
                "each surface prints literal `perm.denied` after unregister"
              ],
              "must_not_observe": [
                "0 registry-only tests executed",
                "any surface test failure",
                "empty registry accepted after unregister"
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
      "description": "Track B test: commit surface register\u2192success\u2192unregister\u2192denied",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-api --test agent_registry -- commit_surface",
      "flow_ref": "UC-IDENT-03",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "governed_repo_commit_registry_cycle",
        "must_observe": [
          "registered commit call returns exact `Ok(())`",
          "after unregister commit denial code is literal `perm.denied`",
          "denial message contains literal `pid ` and `start_time` for the current process"
        ],
        "must_not_observe": [
          "Ok(()) after unregister",
          "env fallback accepted",
          "empty registry accepted after unregister"
        ],
        "negative_control": {
          "would_fail_if": [
            "unregister is a no-op",
            "commit gate still accepts env-only identity",
            "denial code changes from perm.denied"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "governed_repo_commit_registry_cycle",
            "action": {
              "actor": "test_harness",
              "steps": [
                "Register current process as dev in runtime registry",
                "Call enforce_commit_gate_for_target",
                "Unregister current process",
                "Call enforce_commit_gate_for_target again"
              ]
            },
            "end_state": {
              "must_observe": [
                "registered commit call returns exact `Ok(())`",
                "after unregister commit denial code is literal `perm.denied`",
                "denial message contains literal `pid ` and `start_time` for the current process"
              ],
              "must_not_observe": [
                "Ok(()) after unregister",
                "env fallback accepted",
                "empty registry accepted after unregister"
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
      "description": "Track B test: merge surface register\u2192success\u2192unregister\u2192denied",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-api --test agent_registry -- merge_surface",
      "flow_ref": "UC-IDENT-03",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "governed_repo_merge_registry_cycle",
        "must_observe": [
          "registered merge call returns exact `Ok(())`",
          "after unregister merge denial code is literal `perm.denied`",
          "denial message contains literal `pid ` and `start_time` for the current process"
        ],
        "must_not_observe": [
          "Ok(()) after unregister",
          "env fallback accepted",
          "empty registry accepted after unregister"
        ],
        "negative_control": {
          "would_fail_if": [
            "unregister is a no-op",
            "merge gate still accepts env-only identity",
            "denial code changes from perm.denied"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "governed_repo_merge_registry_cycle",
            "action": {
              "actor": "test_harness",
              "steps": [
                "Register current process as merger in runtime registry",
                "Call enforce_merge_gate",
                "Unregister current process",
                "Call enforce_merge_gate again"
              ]
            },
            "end_state": {
              "must_observe": [
                "registered merge call returns exact `Ok(())`",
                "after unregister merge denial code is literal `perm.denied`",
                "denial message contains literal `pid ` and `start_time` for the current process"
              ],
              "must_not_observe": [
                "Ok(()) after unregister",
                "env fallback accepted",
                "empty registry accepted after unregister"
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
      "description": "Track B test: admin-write surface register\u2192success\u2192unregister\u2192denied",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-api --test agent_registry -- admin_write_surface",
      "flow_ref": "UC-IDENT-03",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "governed_repo_admin_registry_cycle",
        "must_observe": [
          "registered admin-write call returns exact `Ok(())`",
          "after unregister admin-write denial code is literal `perm.denied`",
          "denial message contains literal `pid ` and `start_time` for the current process"
        ],
        "must_not_observe": [
          "Ok(()) after unregister",
          "env fallback accepted",
          "empty registry accepted after unregister"
        ],
        "negative_control": {
          "would_fail_if": [
            "unregister is a no-op",
            "admin-write gate still accepts env-only identity",
            "denial code changes from perm.denied"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "governed_repo_admin_registry_cycle",
            "action": {
              "actor": "test_harness",
              "steps": [
                "Register current process as admin in runtime registry",
                "Call enforce_administration_write_gate",
                "Unregister current process",
                "Call enforce_administration_write_gate again"
              ]
            },
            "end_state": {
              "must_observe": [
                "registered admin-write call returns exact `Ok(())`",
                "after unregister admin-write denial code is literal `perm.denied`",
                "denial message contains literal `pid ` and `start_time` for the current process"
              ],
              "must_not_observe": [
                "Ok(()) after unregister",
                "env fallback accepted",
                "empty registry accepted after unregister"
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
      "description": "Track B test: forge review surface register\u2192success\u2192unregister\u2192denied",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-api --test agent_registry -- forge_review_surface",
      "flow_ref": "UC-IDENT-03",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "governed_repo_forge_registry_cycle",
        "must_observe": [
          "registered forge review writes exactly 1 approval verdict",
          "approval verdict principal_id is literal `reviewer`",
          "after unregister forge review denial code is literal `perm.denied`"
        ],
        "must_not_observe": [
          "0 approval verdicts",
          "env fallback accepted",
          "empty registry accepted after unregister"
        ],
        "negative_control": {
          "would_fail_if": [
            "unregister is a no-op",
            "forge review gate still accepts env-only identity",
            "approval attribution is hardcoded or static"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "governed_repo_forge_registry_cycle",
            "action": {
              "actor": "test_harness",
              "steps": [
                "Register current process as reviewer in runtime registry",
                "Call approve_review",
                "Assert one verdict attributed to reviewer",
                "Unregister current process",
                "Call approve_review again"
              ]
            },
            "end_state": {
              "must_observe": [
                "registered forge review writes exactly 1 approval verdict",
                "approval verdict principal_id is literal `reviewer`",
                "after unregister forge review denial code is literal `perm.denied`"
              ],
              "must_not_observe": [
                "0 approval verdicts",
                "env fallback accepted",
                "empty registry accepted after unregister"
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
      "description": "Governed commit env fallback remains explicitly allowed when registry misses and flag is set",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-api --test gate_registry_swap -- env_fallback_still_allowed_on_registry_miss",
      "flow_ref": "UC-IDENT-03",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "governed_commit_env_fallback_flag_set",
        "must_observe": [
          "governed commit gate returns exact `Ok(())`",
          "BUT_AGENT_HANDLE literal `dev` is accepted only because BUT_AUTHZ_ALLOW_ENV_HANDLE=1",
          "runtime registry entry count == 0"
        ],
        "must_not_observe": [
          "perm.denied",
          "branch.protected",
          "empty registry accepted without flag"
        ],
        "negative_control": {
          "would_fail_if": [
            "flag-gated fallback removed",
            "test uses a registry hit instead of empty registry",
            "commit gate reads env directly and ignores flag"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "governed_commit_env_fallback_flag_set",
            "action": {
              "actor": "test_harness",
              "steps": [
                "Create governed repo",
                "Write empty registry file",
                "Set BUT_AGENT_HANDLE=dev",
                "Set BUT_AUTHZ_ALLOW_ENV_HANDLE=1",
                "Call enforce_commit_gate_for_target"
              ]
            },
            "end_state": {
              "must_observe": [
                "governed commit gate returns exact `Ok(())`",
                "BUT_AGENT_HANDLE literal `dev` is accepted only because BUT_AUTHZ_ALLOW_ENV_HANDLE=1",
                "runtime registry entry count == 0"
              ],
              "must_not_observe": [
                "perm.denied",
                "branch.protected",
                "empty registry accepted without flag"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-7",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "Merge env-only denial proves every gate surface is not commit-only",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-api --test gate_registry_swap -- merge_gate_env_only_without_flag_denied",
      "flow_ref": "UC-IDENT-03",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "governed_merge_env_only_no_flag",
        "must_observe": [
          "merge denial code is literal `perm.denied`",
          "denial message contains literal `pid ` and `start_time` for the unregistered current process",
          "BUT_AGENT_HANDLE literal `maint` is ignored without flag"
        ],
        "must_not_observe": [
          "merge returns Ok(())",
          "env fallback accepted",
          "empty registry accepted"
        ],
        "negative_control": {
          "would_fail_if": [
            "merge gate still reads env-only handle directly",
            "resolver bypasses runtime registry",
            "flag check removed"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "governed_merge_env_only_no_flag",
            "action": {
              "actor": "test_harness",
              "steps": [
                "Set BUT_AGENT_HANDLE=maint",
                "Unset BUT_AUTHZ_ALLOW_ENV_HANDLE",
                "Ensure runtime registry is missing or empty",
                "Call enforce_merge_gate"
              ]
            },
            "end_state": {
              "must_observe": [
                "merge denial code is literal `perm.denied`",
                "denial message contains literal `pid ` and `start_time` for the unregistered current process",
                "BUT_AGENT_HANDLE literal `maint` is ignored without flag"
              ],
              "must_not_observe": [
                "merge returns Ok(())",
                "env fallback accepted",
                "empty registry accepted"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-8",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "CLI blackbox operator sequence proves the exact Sprint human gate",
      "test_tier": "e2e",
      "verification_service": "but-cli",
      "verify": "cargo test -p but --test but --features legacy,but-2 -- command::commit_gate::commit_gate_operator_runtime_registry_sequence",
      "flow_ref": "UC-IDENT-03",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "e2e",
        "verification_service": "but-cli",
        "start_ref": "governed_cli_operator_fixture",
        "must_observe": [
          "first env-only commit stderr contains literal `perm.denied`",
          "flag-set env-fallback commit exits with code 0",
          "agent register stdout contains literal `registered` and `dev`",
          "registered no-env commit exits with code 0",
          "post-unregister env-only commit stderr contains literal `perm.denied`"
        ],
        "must_not_observe": [
          "env-only commit succeeds without flag",
          "registered no-env commit denied",
          "agent register skipped",
          "unregister skipped",
          "0 CLI commands executed"
        ],
        "negative_control": {
          "would_fail_if": [
            "CLI command bypasses commit gate",
            "agent register writes no runtime registry entry",
            "unregister is a no-op",
            "test uses direct but-api calls instead of env.but"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "governed_cli_operator_fixture",
            "action": {
              "actor": "test_harness",
              "steps": [
                "Run env.but(\"--format json commit2 -m first\") with BUT_AGENT_HANDLE=dev and no flag; assert failure",
                "Run same commit with BUT_AUTHZ_ALLOW_ENV_HANDLE=1; assert success",
                "Run env.but(\"agent register --as dev\")",
                "Run env.but(\"commit2 -m registered\") with no BUT_AGENT_HANDLE or flag; assert success",
                "Run env.but(\"agent unregister --pid <current> --start-time <current>\")",
                "Run env-only commit again; assert perm.denied"
              ]
            },
            "end_state": {
              "must_observe": [
                "first env-only commit stderr contains literal `perm.denied`",
                "flag-set env-fallback commit exits with code 0",
                "agent register stdout contains literal `registered` and `dev`",
                "registered no-env commit exits with code 0",
                "post-unregister env-only commit stderr contains literal `perm.denied`"
              ],
              "must_not_observe": [
                "env-only commit succeeds without flag",
                "registered no-env commit denied",
                "agent register skipped",
                "unregister skipped",
                "0 CLI commands executed"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-9",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "Expired current-process registry entry denies at a real governed but-api gate",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-api --test gate_registry_swap -- expired_current_process_registry_entry_denied",
      "flow_ref": "UC-IDENT-03",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "governed_commit_expired_current_process_registry_entry",
        "must_observe": [
          "expired entry denial code is literal `perm.denied`",
          "runtime registry load path reports expired entries pruned before resolution or resolver rejects `expires_at < now`",
          "`BUT_AGENT_HANDLE=unset` and `BUT_AUTHZ_ALLOW_ENV_HANDLE=unset`"
        ],
        "must_not_observe": [
          "expired entry authorizes Ok(())",
          "env fallback accepted",
          "success-only registry test passes without expired case",
          "empty/start: registry (0 entries) accepted"
        ],
        "negative_control": {
          "would_fail_if": [
            "Registry::resolve ignores expires_at and authorizes the expired entry",
            "expired-entry negative test omitted or stubbed",
            "test uses direct resolver instead of real commit gate",
            "BUT_AGENT_HANDLE is set and masks the expired registry failure"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "governed_commit_expired_current_process_registry_entry",
            "action": {
              "actor": "test_harness",
              "steps": [
                "Create governed repo authorizing dev for contents:write",
                "Write runtime registry entry for current pid/current start_time with expires_at in the past",
                "Unset BUT_AGENT_HANDLE",
                "Unset BUT_AUTHZ_ALLOW_ENV_HANDLE",
                "Call enforce_commit_gate_for_target"
              ]
            },
            "end_state": {
              "must_observe": [
                "expired entry denial code is literal `perm.denied`",
                "runtime registry load path reports expired entries pruned before resolution or resolver rejects `expires_at < now`",
                "`BUT_AGENT_HANDLE=unset` and `BUT_AUTHZ_ALLOW_ENV_HANDLE=unset`"
              ],
              "must_not_observe": [
                "expired entry authorizes Ok(())",
                "env fallback accepted",
                "success-only registry test passes without expired case",
          "empty/start: registry (0 entries) accepted"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-10",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "Current pid with wrong start_time denies as stale/unregistered at a real gate",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-api --test gate_registry_swap -- current_pid_wrong_start_time_denied_at_commit_gate",
      "flow_ref": "UC-IDENT-03",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "governed_commit_current_pid_wrong_start_time_entry",
        "must_observe": [
          "wrong-start-time denial code is literal `perm.denied`",
          "denial message contains literal `pid ` and `start_time` for the current process",
          "registry entry key uses tuple `(current_pid, stale_start_time)`"
        ],
        "must_not_observe": [
          "wrong start_time authorizes Ok(())",
          "env fallback accepted",
          "stale process identity treated as current",
          "empty/start: registry (0 entries) accepted"
        ],
        "negative_control": {
          "would_fail_if": [
            "Registry lookup matches only pid and ignores start_time",
            "wrong-start-time negative test omitted or stubbed",
            "test registers the real current start_time by mistake",
            "commit gate bypasses runtime process identity check"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "governed_commit_current_pid_wrong_start_time_entry",
            "action": {
              "actor": "test_harness",
              "steps": [
                "Create governed repo authorizing dev for contents:write",
                "Write runtime registry entry for current pid and wrong start_time",
                "Unset BUT_AGENT_HANDLE",
                "Unset BUT_AUTHZ_ALLOW_ENV_HANDLE",
                "Call enforce_commit_gate_for_target"
              ]
            },
            "end_state": {
              "must_observe": [
                "wrong-start-time denial code is literal `perm.denied`",
                "denial message contains literal `pid ` and `start_time` for the current process",
                "registry entry key uses tuple `(current_pid, stale_start_time)`"
              ],
              "must_not_observe": [
                "wrong start_time authorizes Ok(())",
                "env fallback accepted",
                "stale process identity treated as current",
          "empty/start: registry (0 entries) accepted"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-11",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "Wrong pid/current start_time mismatch cannot authorize at a real gate",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-api --test gate_registry_swap -- wrong_pid_current_start_time_denied_at_commit_gate",
      "flow_ref": "UC-IDENT-03",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "governed_commit_wrong_pid_current_start_time_entry",
        "must_observe": [
          "wrong-pid denial code is literal `perm.denied`",
          "denial message contains literal `pid ` and `start_time` for the current process",
          "registry entry key uses tuple `(wrong_pid, current_start_time)`"
        ],
        "must_not_observe": [
          "wrong pid authorizes Ok(())",
          "env fallback accepted",
          "stale process identity treated as current",
          "empty/start: registry (0 entries) accepted"
        ],
        "negative_control": {
          "would_fail_if": [
            "Registry lookup matches only start_time and ignores pid",
            "wrong-pid negative test omitted or stubbed",
            "test registers the real current pid by mistake",
            "commit gate bypasses runtime process identity check"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "governed_commit_wrong_pid_current_start_time_entry",
            "action": {
              "actor": "test_harness",
              "steps": [
                "Create governed repo authorizing dev for contents:write",
                "Write runtime registry entry for wrong pid and current process start_time",
                "Unset BUT_AGENT_HANDLE",
                "Unset BUT_AUTHZ_ALLOW_ENV_HANDLE",
                "Call enforce_commit_gate_for_target"
              ]
            },
            "end_state": {
              "must_observe": [
                "wrong-pid denial code is literal `perm.denied`",
                "denial message contains literal `pid ` and `start_time` for the current process",
                "registry entry key uses tuple `(wrong_pid, current_start_time)`"
              ],
              "must_not_observe": [
                "wrong pid authorizes Ok(())",
                "env fallback accepted",
                "stale process identity treated as current",
          "empty/start: registry (0 entries) accepted"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "`agent_registry` Track B suite exercises registry-only setup across current surfaces is true",
      "maps_to_ac": "AC-1",
      "verify": "cargo test -p but-api --test agent_registry"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "Track B test: commit surface register\u2192success\u2192unregister\u2192denied is true",
      "maps_to_ac": "AC-2",
      "verify": "cargo test -p but-api --test agent_registry -- commit_surface"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "Track B test: merge surface register\u2192success\u2192unregister\u2192denied is true",
      "maps_to_ac": "AC-3",
      "verify": "cargo test -p but-api --test agent_registry -- merge_surface"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "Track B test: admin-write surface register\u2192success\u2192unregister\u2192denied is true",
      "maps_to_ac": "AC-4",
      "verify": "cargo test -p but-api --test agent_registry -- admin_write_surface"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "Track B test: forge review surface register\u2192success\u2192unregister\u2192denied is true",
      "maps_to_ac": "AC-5",
      "verify": "cargo test -p but-api --test agent_registry -- forge_review_surface"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "description": "Governed commit env fallback remains explicitly allowed when registry misses and flag is set is true",
      "maps_to_ac": "AC-6",
      "verify": "cargo test -p but-api --test gate_registry_swap -- env_fallback_still_allowed_on_registry_miss"
    },
    {
      "id": "TC-7",
      "type": "test_criterion",
      "description": "Merge env-only denial proves every gate surface is not commit-only is true",
      "maps_to_ac": "AC-7",
      "verify": "cargo test -p but-api --test gate_registry_swap -- merge_gate_env_only_without_flag_denied"
    },
    {
      "id": "TC-8",
      "type": "test_criterion",
      "description": "CLI blackbox operator sequence proves the exact Sprint human gate is true",
      "maps_to_ac": "AC-8",
      "verify": "cargo test -p but --test but --features legacy,but-2 -- command::commit_gate::commit_gate_operator_runtime_registry_sequence"
    },
    {
      "id": "TC-9",
      "type": "test_criterion",
      "description": "Expired current-process registry entry is denied at a real governed commit gate is true",
      "maps_to_ac": "AC-9",
      "verify": "cargo test -p but-api --test gate_registry_swap -- expired_current_process_registry_entry_denied"
    },
    {
      "id": "TC-10",
      "type": "test_criterion",
      "description": "Current pid with wrong start_time is denied stale/unregistered at a real governed commit gate is true",
      "maps_to_ac": "AC-10",
      "verify": "cargo test -p but-api --test gate_registry_swap -- current_pid_wrong_start_time_denied_at_commit_gate"
    },
    {
      "id": "TC-11",
      "type": "test_criterion",
      "description": "Wrong pid/current start_time mismatch cannot authorize at a real governed commit gate is true",
      "maps_to_ac": "AC-11",
      "verify": "cargo test -p but-api --test gate_registry_swap -- wrong_pid_current_start_time_denied_at_commit_gate"
    }
  ]
}
-->
