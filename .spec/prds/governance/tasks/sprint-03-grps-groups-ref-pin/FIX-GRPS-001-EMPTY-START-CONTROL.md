# FIX-GRPS-001-EMPTY-START-CONTROL: Exercise the dead AUTHZ_EMPTY_START fail-closed control in grps_union.rs with a real in-process test

## What this does

Makes GRPS-001 AC-1's specified fail-closed negative control actually run. `governed_repo` (crates/but-authz/tests/grps_union.rs:148-189) carries an `AUTHZ_EMPTY_START` early-return guard (lines 150-152) that returns the bare `governance-base` scenario with NO committed `.gitbutler/permissions.toml`/`gates.toml` — but NO test, CI step, or xtask anywhere in the repo ever sets that env var (`grep -rn AUTHZ_EMPTY_START` finds only the two guard sites, no setter). The specified empty-start case is therefore dead/unexercised. This task drives the empty-start branch in-process and asserts the fail-closed outcome: against a bare scenario, `load_governance_config(&repo, "refs/heads/main")` returns `Err(ConfigError)` with `code() == "config.invalid"` (the loader's `read_config_blob` fails `lookup_entry_by_path(".gitbutler/permissions.toml")` → `anyhow!("missing ... at refs/heads/main")` → `ConfigError::invalid`, config.rs:262/293/250-255). Because `but-authz` has NO `serial_test`/`temp_env` dev-deps and `std::env::set_var` is `unsafe` in edition 2024, the test drives the empty-start path *deterministically* via a refactored `governed_repo_with(empty_start: bool)` helper — exercising the real code path with zero env mutation, zero new dependencies, and zero `unsafe`.

## Why

Sprint 03 remediation · red-hat finding **C** (MEDIUM, dead negative control) from `.spec/reviews/red-hat-sprint-03-2026-06-19.md:25,39,65-68`. GRPS-001 AC-1 case-2 specifies a fail-closed control (`set AUTHZ_EMPTY_START ... load(...) expecting config.invalid`), and the guard was re-implemented per the spec (`R3`), but the case was never run — a PARTIAL verdict in the AC summary. A negative control that never executes is theatre: it provides no evidence the loader actually fails closed against an empty start. The honest fix is to exercise it for real. The red-hat instruction allowed either an in-process test OR removing the dead guard if it is owned solely by an external harness — but `grep` proves NO external harness owns it, so the correct resolution is to **exercise it in-process** (the preferred branch).

## How to verify

PRIMARY **AC-1** — `cargo test -p but-authz empty_start_fails_closed_config_invalid` (a bare scenario with no committed governance config makes `load_governance_config(&repo, "refs/heads/main")` return `Err` whose `code()` is `"config.invalid"`). Full gate set in the spec below.

## Scope

- crates/but-authz/tests/grps_union.rs (MODIFY) — refactor `governed_repo` (148-189) into `governed_repo_with(empty_start: bool)`; keep `fn governed_repo()` delegating to `governed_repo_with(std::env::var_os("AUTHZ_EMPTY_START").is_some())` so the env-var behavior at 150-152 is PRESERVED for any external harness, while the new test passes `empty_start = true` directly (deterministic, no env mutation); ADD `empty_start_fails_closed_config_invalid` asserting `load_governance_config(...).unwrap_err().code() == "config.invalid"`

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: FIX-GRPS-001-EMPTY-START-CONTROL - Exercise the dead AUTHZ_EMPTY_START fail-closed control in grps_union.rs in-process
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P1
EFFORT:     S  (60 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ./SPRINT.md
PRD_REFS:   UC-GRPS-01
CAPABILITIES: CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-authz empty_start_fails_closed_config_invalid   |   cargo test -p but-authz group_union_authorizes_review_denies_merge   |   cargo test -p but-authz union_paths_stay_equal   |   cargo test -p but-authz delegated_admin_ceiling   |   cargo test -p but-authz claims_do_not_widen_union_even_with_group_backing
  check: cargo check -p but-authz --all-targets
  lint:  cargo clippy -p but-authz --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
The AUTHZ_EMPTY_START fail-closed control specified by GRPS-001 AC-1 case-2 is exercised in-process by a new test, `empty_start_fails_closed_config_invalid`. The test drives the empty-start branch deterministically through a refactored `governed_repo_with(empty_start: bool)` helper (no process-env mutation, no new deps, no unsafe set_var), loads the bare `governance-base` scenario at refs/heads/main, and asserts `load_governance_config` returns `Err(ConfigError)` with `code() == "config.invalid"` — proving the loader fails closed (it does NOT silently return an empty GovConfig). The existing `governed_repo()` continues to honor the AUTHZ_EMPTY_START env var (preserving any external-harness contract) by delegating to `governed_repo_with`. All four existing grps_union tests stay green; clippy/fmt clean.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST inspect what the empty-start branch actually produces before asserting. The bare `governance-base` scenario commits ONLY README.md (crates/but-authz/tests/fixtures/scenario/governance-base.sh) — NO .gitbutler/*.toml. So load_governance_config(&repo, "refs/heads/main") -> load_governance_config_inner -> read_config_blob (config.rs:262,276-301) calls tree.lookup_entry_by_path(".gitbutler/permissions.toml") which returns None -> `anyhow!("missing {path} at {target_ref}")` (config.rs:293) -> mapped to ConfigError::invalid (config.rs:28,250-255) whose code() is "config.invalid" (config.rs:10,246-248). The fail-closed outcome is therefore `Err(ConfigError)` with `code() == "config.invalid"` — NOT a perm.denied (the loader fails before authorize is ever called). Assert config.invalid.
- [MUST] MUST drive the empty-start branch DETERMINISTICALLY without mutating process env. but-authz has ONLY `anyhow` + `but-testsupport` as dev-deps (NO serial_test, NO temp_env), and `std::env::set_var` is `unsafe` in edition 2024. Refactor `governed_repo()` into `governed_repo_with(empty_start: bool)` and have the new test call `governed_repo_with(true)` directly. This exercises the SAME early-return code path the env guard triggers, in-process, with no env manipulation. Do NOT add serial_test/temp_env, do NOT call std::env::set_var.
- [MUST] MUST PRESERVE the AUTHZ_EMPTY_START env-var behavior for any external harness. `fn governed_repo()` must keep delegating: `governed_repo_with(std::env::var_os("AUTHZ_EMPTY_START").is_some())`. Do NOT delete the env read — only relocate it into the thin wrapper so the boolean path is independently testable.
- [MUST] MUST keep the four existing tests (group_union_authorizes_review_denies_merge, union_paths_stay_equal, delegated_admin_ceiling, claims_do_not_widen_union_even_with_group_backing) calling `governed_repo()` unchanged in BEHAVIOR — they get the FULL governed scenario (empty_start=false in normal runs) and must stay green.
- [NEVER] NEVER assert the empty-start case authorizes anything / returns an empty-but-Ok GovConfig (silent fail-open). The whole point of the control is that an empty start FAILS CLOSED — config.invalid, not a permissive empty config.
- [NEVER] NEVER remove the AUTHZ_EMPTY_START guard outright. The red-hat instruction permits removal ONLY if the control is owned solely by an external harness; `grep -rn AUTHZ_EMPTY_START` proves NO external harness sets it, so the correct resolution is to EXERCISE it in-process, not delete it. (If a future external harness is introduced, the env path still works via the wrapper.)
- [STRICTLY] STRICTLY assert via the typed `ConfigError::code()` API (config.rs:246), NOT by matching the Display string — the codebase contract is "don't make consumers match error strings" (crates/AGENTS.md). Use `load_governance_config(&repo, "refs/heads/main").expect_err("empty start must fail closed").code()` and compare to "config.invalid".

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: empty_start_fails_closed_config_invalid GREEN — bare scenario => load returns Err with code()=="config.invalid"
- [ ] AC-2: governed_repo_with(false) still yields the full governed scenario; the four existing grps_union tests stay green
- [ ] AC-3: the AUTHZ_EMPTY_START env-var path is preserved (governed_repo() delegates to governed_repo_with(env_present)); no serial_test/temp_env/unsafe added
- [ ] All verification gates pass; fmt + clippy clean

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: empty-start fails closed with config.invalid (the previously-dead control, now exercised) [PRIMARY]
  GIVEN: governed_repo refactored to governed_repo_with(empty_start: bool); a new test calls governed_repo_with(true) (the empty-start branch — bare governance-base scenario, NO committed .gitbutler/*.toml)
  WHEN:  load_governance_config(&repo, "refs/heads/main") is called against the bare scenario
  THEN:  it returns Err(ConfigError) whose code() == "config.invalid" (read_config_blob fails on the missing .gitbutler/permissions.toml entry; the loader fails closed BEFORE any authorize call) — proving the empty start does NOT silently produce a permissive empty GovConfig
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-authz load_governance_config + real gix repo (bare governance-base scenario, no governance blobs)
  VERIFY: cargo test -p but-authz empty_start_fails_closed_config_invalid
  SCENARIO (negative controls): would FAIL (correctly catch a regression) if load_governance_config returned Ok(empty GovConfig) for a bare scenario (silent fail-open) — the expect_err would panic; would FAIL if the error code were something other than "config.invalid"; would pass against a stub loader that always errors regardless of input — guarded by AC-2 requiring governed_repo_with(false) to load a VALID full config that the existing tests authorize against

AC-2: governed_repo_with(false) yields the full governed scenario; existing grps_union tests stay green
  GIVEN: the refactor where governed_repo() delegates to governed_repo_with(false) in normal runs (AUTHZ_EMPTY_START unset)
  WHEN:  cargo test -p but-authz group_union_authorizes_review_denies_merge ; union_paths_stay_equal ; delegated_admin_ceiling ; claims_do_not_widen_union_even_with_group_backing
  THEN:  all four pass unchanged — governed_repo_with(false) commits the same permissions.toml/gates.toml the original governed_repo did (reviewer-only group-only member, reviewer-byref, ro, config-admins/delegate)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-authz + real gix repo + committed target-ref TOML
  VERIFY: cargo test -p but-authz group_union_authorizes_review_denies_merge ; cargo test -p but-authz union_paths_stay_equal ; cargo test -p but-authz delegated_admin_ceiling ; cargo test -p but-authz claims_do_not_widen_union_even_with_group_backing
  SCENARIO (negative controls): would FAIL if the refactor changed which blobs governed_repo_with(false) commits (the existing tests would no longer find reviewer-only/delegate/etc.); would FAIL if governed_repo_with(true) leaked into the existing tests' path (they would lose their config)

AC-3: the AUTHZ_EMPTY_START env path is preserved and no env/unsafe machinery is added
  GIVEN: governed_repo() reads AUTHZ_EMPTY_START via std::env::var_os and passes the boolean to governed_repo_with
  WHEN:  grep the test file and Cargo.toml
  THEN:  `governed_repo()` still contains `std::env::var_os("AUTHZ_EMPTY_START")` (env path preserved for external harnesses); the file contains NO `std::env::set_var`, NO `serial_test`, NO `temp_env`; but-authz/Cargo.toml [dev-dependencies] is unchanged (still only anyhow + but-testsupport)
  TEST_TIER: integration   VERIFICATION_SERVICE: source-grep + cargo build (the test compiles without new deps)
  VERIFY: grep -n 'AUTHZ_EMPTY_START' crates/but-authz/tests/grps_union.rs ; ! grep -nE 'set_var|serial_test|temp_env' crates/but-authz/tests/grps_union.rs ; cargo check -p but-authz --all-targets
  SCENARIO (negative controls): would FAIL if the env read was deleted (external-harness contract broken); would FAIL if set_var/serial_test/temp_env were introduced (the grep guard fires); would FAIL to compile if a new dev-dep were needed but not added (forcing an honest dependency decision rather than a hidden one)

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, error): load_governance_config(&repo, "refs/heads/main") against governed_repo_with(true) returns Err whose code()=="config.invalid"
    VERIFY: cargo test -p but-authz empty_start_fails_closed_config_invalid
- TC-2 (-> AC-1, structural): the empty-start load does NOT return Ok (no silent fail-open empty GovConfig)
    VERIFY: cargo test -p but-authz empty_start_fails_closed_config_invalid
- TC-3 (-> AC-2, happy_path): all four existing grps_union tests pass against governed_repo_with(false)
    VERIFY: cargo test -p but-authz group_union_authorizes_review_denies_merge ; cargo test -p but-authz union_paths_stay_equal ; cargo test -p but-authz delegated_admin_ceiling ; cargo test -p but-authz claims_do_not_widen_union_even_with_group_backing
- TC-4 (-> AC-3, structural): governed_repo() still reads AUTHZ_EMPTY_START; no set_var/serial_test/temp_env in the file; Cargo.toml dev-deps unchanged
    VERIFY: grep -n 'AUTHZ_EMPTY_START' crates/but-authz/tests/grps_union.rs ; ! grep -nE 'set_var|serial_test|temp_env' crates/but-authz/tests/grps_union.rs

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides: an exercised fail-closed control proving load_governance_config returns config.invalid (not a permissive empty config) when governance is absent at the target ref
consumes: but_authz::load_governance_config, but_authz::ConfigError::code, but_testsupport::writable_scenario, but_testsupport::invoke_bash
boundary_contracts:
  - CAP-AUTHZ-01: governance is opt-in by presence, but once the gate runs it must FAIL CLOSED on incomplete/absent config (config.invalid), never silently allow. This control proves the bare-scenario start fails closed in-process.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-authz/tests/grps_union.rs (MODIFY) — refactor governed_repo() -> governed_repo_with(empty_start: bool) (env read preserved in the thin wrapper); ADD empty_start_fails_closed_config_invalid test asserting load_governance_config(...).unwrap_err().code() == "config.invalid"
writeProhibited:
  - crates/but-authz/src/** — production source; this is a test-coverage fix, no behavior change
  - crates/but-authz/Cargo.toml — do NOT add dependencies; the deterministic-boolean approach needs none
  - crates/but-authz/tests/config.rs — the SECOND AUTHZ_EMPTY_START guard (config.rs:197) lives here; it is OUT OF SCOPE for this task (this task addresses the grps_union.rs:150 guard the red-hat finding C named); do not touch tests/config.rs
  - crates/but-authz/tests/grps_ref_pin.rs — owned by FIX-GRPS-002-AC3-TEETH
  - any gitbutler-* crate
  - Any file not in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-authz/tests/grps_union.rs (148-189)
   Focus: governed_repo() + the AUTHZ_EMPTY_START early-return guard (150-152). This is the dead control: line 150 reads the env var, 151 returns the bare scenario, but no caller sets the var. Refactor into governed_repo_with(empty_start: bool); the new test calls governed_repo_with(true).
2. crates/but-authz/tests/fixtures/scenario/governance-base.sh (whole file)
   Focus: the bare scenario commits ONLY README.md — NO .gitbutler/*.toml. This is why load_governance_config fails closed (missing permissions.toml) on the empty-start path.
3. crates/but-authz/src/config.rs (24-29, 258-301)
   Focus: load_governance_config -> load_governance_config_inner -> read_config_blob. read_config_blob (276-301) does lookup_entry_by_path(".gitbutler/permissions.toml") (290-293); a missing entry -> anyhow!("missing {path} at {target_ref}") (293). load_governance_config maps any inner error to ConfigError::invalid (28).
4. crates/but-authz/src/config.rs (10, 230-256)
   Focus: ConfigError + code() returning the CONFIG_INVALID constant ("config.invalid", line 10). Assert against code(), not the Display string.
5. crates/but-authz/Cargo.toml ([dev-dependencies])
   Focus: ONLY anyhow + but-testsupport are dev-deps — NO serial_test/temp_env. Confirms the deterministic-boolean approach is required (no env-setting crate available; set_var is unsafe in edition 2024).
6. crates/but-authz/tests/config.rs (195-230)
   Focus: the SIBLING AUTHZ_EMPTY_START guard (197) and governed_repo helper — context only; this file is OUT OF SCOPE (do not edit), but it shows the same dead-guard pattern the grps_union.rs guard mirrors.
7. .spec/reviews/red-hat-sprint-03-2026-06-19.md (25, 39, 65-68)
   Focus: finding C — the specified AUTHZ_EMPTY_START control is never exercised; remediation: add a test that drives the empty-start branch and asserts fail-closed (config.invalid OR perm.denied), OR remove the dead guard if owned by an external harness (it is not).
8. .spec/prds/governance/tasks/sprint-03-grps-groups-ref-pin/GRPS-001-effective-set-union-group-ceiling.md (AC-1 case-2, lines 280-298 of the contract JSON)
   Focus: the original AC-1 case-2 spec — "set AUTHZ_EMPTY_START ... load expecting config.invalid; else authorize" with the must_observe "EITHER load.err().code == config.invalid OR authorize.err().code == perm.denied". This task realizes that case as a real test.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- empty-start fail-closed (PRIMARY): `cargo test -p but-authz empty_start_fails_closed_config_invalid`  -> Exit 0; bare scenario => load returns Err code()=="config.invalid"
- existing union tests stay green: `cargo test -p but-authz group_union_authorizes_review_denies_merge union_paths_stay_equal delegated_admin_ceiling claims_do_not_widen_union_even_with_group_backing`  -> Exit 0
- env path preserved + no env machinery: `grep -n 'AUTHZ_EMPTY_START' crates/but-authz/tests/grps_union.rs` (match) ; `! grep -nE 'set_var|serial_test|temp_env' crates/but-authz/tests/grps_union.rs` (no match)  -> Exit 0
- no new deps: `git diff --exit-code crates/but-authz/Cargo.toml`  -> Exit 0 (Cargo.toml unchanged)
- clippy: `cargo clippy -p but-authz --all-targets`  -> Exit 0
- fmt: `cargo fmt --check`  -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: parameterize an env-gated test helper into a boolean-driven function so the gated branch is independently and deterministically testable in-process, while a thin wrapper preserves the env-var entry point for external harnesses; assert the typed ConfigError::code() not the Display string
pattern_source: crates/but-authz/tests/grps_union.rs:148-152 (the env guard) refactored to governed_repo_with(empty_start: bool); ConfigError::code() at crates/but-authz/src/config.rs:246
anti_pattern: using std::env::set_var (unsafe in edition 2024) to flip the env var inside the test; adding serial_test/temp_env dev-deps just to set one env var; deleting the dead guard outright (no external harness owns it — grep proves it); asserting the empty start authorizes / returns an empty-but-Ok config (silent fail-open); matching the Display string instead of code(); editing tests/config.rs (the sibling guard, out of scope)
interaction_notes:
  - The empty-start outcome is `config.invalid` (load-time failure), NOT `perm.denied`. The original AC-1 case-2 spec allowed EITHER (config.invalid OR perm.denied) because it left the path open; inspecting the actual code shows the bare governance-base scenario has no committed blobs, so load fails first with config.invalid — authorize is never reached. Assert config.invalid (the real outcome), and document in a test comment why (missing .gitbutler/permissions.toml at the target ref).
  - Keep governed_repo() as the public test helper the four existing tests call. Only its internals change: `fn governed_repo() -> (gix::Repository, impl std::fmt::Debug) { governed_repo_with(std::env::var_os("AUTHZ_EMPTY_START").is_some()) }`. The new test bypasses the env entirely: `let (repo, _tmp) = governed_repo_with(true);`.
  - `impl std::fmt::Debug` is the existing return-type bound for the tmp guard — keep it identical so the four existing call sites compile unchanged.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — Pure but-authz test Rust: a small helper refactor (env-gated -> boolean-driven), a typed-error assertion via ConfigError::code(), and a real-git bare-scenario load. No frontend, no Tauri, no new dependencies.
reviewer: rust-reviewer — adversarial pass: confirm the new test actually exercises the empty-start branch (not a no-op), confirm config.invalid is the asserted code (not Display-string matching), confirm no env/unsafe/dep machinery crept in, confirm the four existing tests stay green.
coding_standards: crates/AGENTS.md, crates/WORKSPACE_MODEL.md, crates/but-authz (nearby patterns: but-testsupport scenarios, typed ConfigError::code, writable_scenario)

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: GRPS-001 (merged)
Blocks:     (none — independent remediation)
Parallel with: FIX-GRPS-002-AC3-TEETH, FIX-GRPS-RED-EVIDENCE-CONTRACT
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "FIX-GRPS-001-EMPTY-START-CONTROL",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "empty_start_bare_scenario": {
      "description": "The bare `governance-base` scenario via but_testsupport::writable_scenario(\"governance-base\"), driven through the empty-start branch by governed_repo_with(true). The scenario commits ONLY README.md (crates/but-authz/tests/fixtures/scenario/governance-base.sh) — NO .gitbutler/permissions.toml or gates.toml — so load_governance_config(&repo, \"refs/heads/main\") fails closed on the missing permissions blob.",
      "seed_method": "cli",
      "records": [
        "Refactor: `fn governed_repo() -> (gix::Repository, impl std::fmt::Debug) { governed_repo_with(std::env::var_os(\"AUTHZ_EMPTY_START\").is_some()) }`",
        "`fn governed_repo_with(empty_start: bool) -> (gix::Repository, impl std::fmt::Debug)`: `let (repo, tmp) = but_testsupport::writable_scenario(\"governance-base\"); if empty_start { return (repo, tmp); }` then the existing invoke_bash that commits the full permissions.toml/gates.toml (grps_union.rs:154-187)",
        "New test: `let (repo, _tmp) = governed_repo_with(true);` then `let err = but_authz::load_governance_config(&repo, \"refs/heads/main\").expect_err(\"empty start must fail closed\"); assert_eq!(err.code(), \"config.invalid\", \"bare scenario must fail closed at load, not return a permissive empty config\");`"
      ]
    },
    "full_governed_scenario": {
      "description": "governed_repo_with(false) — the FULL governed scenario the four existing tests rely on (reviewer-only group-only member in code-reviewers/reviews:write, reviewer-byref via groups=[...], ro contents:read, config-admins/administration:write with delegate). Identical to the pre-refactor governed_repo(false-path) so the existing tests pass unchanged.",
      "seed_method": "cli",
      "records": [
        "governed_repo_with(false) commits .gitbutler/permissions.toml + gates.toml exactly as grps_union.rs:154-187 does today (no content change)",
        "the four existing tests call governed_repo() which now delegates to governed_repo_with(false) in normal (env-unset) runs"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "description": "Empty-start fails closed with config.invalid — the previously-dead AUTHZ_EMPTY_START control, now exercised in-process via governed_repo_with(true).",
      "verify": "cargo test -p but-authz empty_start_fails_closed_config_invalid",
      "primary": true,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-authz load_governance_config + real gix repo (bare governance-base scenario, no governance blobs)",
        "negative_control": {
          "would_fail_if": [
            "load_governance_config returned Ok(empty GovConfig) for a bare scenario (silent fail-open) — the expect_err would panic",
            "the error code were something other than \"config.invalid\"",
            "would pass against a stub loader that always errors — guarded by AC-2 requiring governed_repo_with(false) to load a VALID full config the existing tests authorize against"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "empty_start_bare_scenario",
            "action": {
              "actor": "ci",
              "steps": [
                "let (repo, _tmp) = governed_repo_with(true);",
                "let err = but_authz::load_governance_config(&repo, \"refs/heads/main\").expect_err(...);",
                "assert_eq!(err.code(), \"config.invalid\");"
              ]
            },
            "end_state": {
              "must_observe": [
                "`load_governance_config(&repo, \"refs/heads/main\")` returns `Err`",
                "`err.code() == \"config.invalid\"`"
              ],
              "must_not_observe": [
                "`load_governance_config` returns `Ok` with an empty GovConfig (silent fail-open)",
                "`0`/empty effective set with no error (the dead-control regression this fix removes)",
                "a non-`config.invalid` error code"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "governed_repo_with(false) yields the full governed scenario; the four existing grps_union tests stay green.",
      "verify": "cargo test -p but-authz group_union_authorizes_review_denies_merge",
      "primary": false,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-authz + real gix repo + committed target-ref permissions.toml/gates.toml",
        "negative_control": {
          "would_fail_if": [
            "the refactor changed which blobs governed_repo_with(false) commits (existing tests would not find reviewer-only/delegate/etc.)",
            "governed_repo_with(true) leaked into the existing tests' path (they would lose their config and fail to load)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "full_governed_scenario",
            "action": {
              "actor": "ci",
              "steps": [
                "cargo test -p but-authz group_union_authorizes_review_denies_merge",
                "cargo test -p but-authz union_paths_stay_equal",
                "cargo test -p but-authz delegated_admin_ceiling",
                "cargo test -p but-authz claims_do_not_widen_union_even_with_group_backing"
              ]
            },
            "end_state": {
              "must_observe": [
                "all four existing tests PASS unchanged",
                "governed_repo_with(false) commits the same permissions.toml/gates.toml as the pre-refactor helper"
              ],
              "must_not_observe": [
                "any of the four existing tests regressing",
                "governed_repo_with(false) returning the bare scenario"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "The AUTHZ_EMPTY_START env path is preserved (governed_repo delegates env_present to governed_repo_with) and NO env/unsafe/dependency machinery is added.",
      "verify": "grep -n 'AUTHZ_EMPTY_START' crates/but-authz/tests/grps_union.rs",
      "primary": false,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "source-grep + cargo check (the test compiles without new deps)",
        "negative_control": {
          "would_fail_if": [
            "the env read was deleted (external-harness contract broken)",
            "set_var/serial_test/temp_env were introduced (the grep guard fires)",
            "a new dev-dep were needed but not added (compile fails, forcing an honest dependency decision)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "full_governed_scenario",
            "action": {
              "actor": "ci",
              "steps": [
                "grep -n 'AUTHZ_EMPTY_START' crates/but-authz/tests/grps_union.rs",
                "! grep -nE 'set_var|serial_test|temp_env' crates/but-authz/tests/grps_union.rs",
                "git diff --exit-code crates/but-authz/Cargo.toml",
                "cargo check -p but-authz --all-targets"
              ]
            },
            "end_state": {
              "must_observe": [
                "`governed_repo()` still contains `std::env::var_os(\"AUTHZ_EMPTY_START\")`",
                "NO `set_var`/`serial_test`/`temp_env` in grps_union.rs",
                "crates/but-authz/Cargo.toml unchanged (dev-deps still anyhow + but-testsupport)",
                "`cargo check -p but-authz --all-targets` passes"
              ],
              "must_not_observe": [
                "the env read removed",
                "`std::env::set_var` or a new env-setting dev-dep introduced"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "load_governance_config(&repo, \"refs/heads/main\") against governed_repo_with(true) returns Err whose code()==\"config.invalid\"",
      "verify": "cargo test -p but-authz empty_start_fails_closed_config_invalid",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "the empty-start load does NOT return Ok (no silent fail-open empty GovConfig)",
      "verify": "cargo test -p but-authz empty_start_fails_closed_config_invalid",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "all four existing grps_union tests pass against governed_repo_with(false)",
      "verify": "cargo test -p but-authz group_union_authorizes_review_denies_merge ; cargo test -p but-authz union_paths_stay_equal ; cargo test -p but-authz delegated_admin_ceiling ; cargo test -p but-authz claims_do_not_widen_union_even_with_group_backing",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "governed_repo() still reads AUTHZ_EMPTY_START; no set_var/serial_test/temp_env in the file; Cargo.toml dev-deps unchanged",
      "verify": "grep -n 'AUTHZ_EMPTY_START' crates/but-authz/tests/grps_union.rs ; ! grep -nE 'set_var|serial_test|temp_env' crates/but-authz/tests/grps_union.rs",
      "maps_to_ac": "AC-3"
    }
  ]
}
-->
</details>
