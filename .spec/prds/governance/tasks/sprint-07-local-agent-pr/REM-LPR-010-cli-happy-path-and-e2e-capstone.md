# REM-LPR-010: Add `lpr_review_cli_happy_path` snapbox suite + `lpr_full_local_loop_request_to_merge_no_forge` e2e capstone + write `NAPI-AUDIT.md`

> Status: Backlog
> Commit: (none yet)
> Reviewer: rust-reviewer
> Updated: 2026-06-22T18:00:00Z
> PROPOSED-BY: rust-planner

## What this does

Closes the red-hat remediation gap for the LPR slice by landing the three missing artifacts identified at `red-hat-20260622-173510.md`:

1. **`lpr_review_cli_happy_path`** — a snapbox CLI test in `crates/but/tests/` exercising the `but review` LPR subcommand surface (`request`, `assign`, `comment`, `comments`, `resolve`, `status`, `request-changes`) with `env.but(...).assert().stdout_eq(...)/.stderr_eq(...)` and `[..]`/`...` wildcards.
2. **`lpr_full_local_loop_request_to_merge_no_forge`** — an e2e capstone proving the sprint thesis that the full `request → assign → comment → resolve → approve → merge` loop runs **entirely local** with `keep_reviews_local=true` and **no forge network call**.
3. **`NAPI-AUDIT.md`** — the missing R14 audit note at `.spec/prds/governance/tasks/sprint-07-local-agent-pr/NAPI-AUDIT.md`, documenting the gated N-API routing of the six LPR verbs plus the `principal_kind_*` verbs.

The SDK-regeneration, honesty-grep, and R22 drive-layer-integrity pieces from the original LPR-010 task are already green; this remediation task covers only the missing CLI suite, e2e capstone, and audit document.

## Why

Sprint 07 · PRD UC-LPR-01..UC-LPR-07 · capability CAP-AUTHZ-01. The red-hat review found M1 (HIGH/CRITICAL) that the named CLI and e2e tests do not exist, and L1 (LOW) that the `NAPI-AUDIT.md` artifact is absent despite the structural `napi_audit_lpr_verbs_route_through_gated_but_api` test passing. Without these, the LPR slice has no automated proof that the new verbs are reachable through the CLI and that the local-only review loop actually works without a forge.

## How to verify

PRIMARY **AC-1** — `cargo test -p but lpr_review_cli_happy_path`: every `but review` LPR verb exercised via snapbox assertions on stdout/stderr/exit.

PRIMARY **AC-2** — `cargo test -p but lpr_full_local_loop_request_to_merge_no_forge`: the full local loop completes, `but merge` advances the target ref, and no forge call is observed.

PRIMARY **AC-3** — `.spec/prds/governance/tasks/sprint-07-local-agent-pr/NAPI-AUDIT.md` exists and documents the R14 finding for the six LPR verbs plus `principal_kind_read`/`principal_kind_update`.

## Scope

- `crates/but/tests/but/command/lpr_review_cli_happy_path.rs` (NEW — snapbox CLI happy-path suite)
- `crates/but/tests/but/command/lpr_full_local_loop_request_to_merge_no_forge.rs` (NEW — e2e local-only capstone)
- `crates/but/tests/but/command/mod.rs` (MODIFY — add the two new modules under `#[cfg(feature = "legacy")]`)
- `crates/but/tests/snapshots/` (NEW — snapbox snapshots for the two tests)
- `.spec/prds/governance/tasks/sprint-07-local-agent-pr/NAPI-AUDIT.md` (NEW — R14 N-API audit note)
- All LPR production code, the merge gate, and the review requirement gate are **CONSUME-only** for this task.

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: REM-LPR-010 — Add LPR CLI happy-path snapbox suite + e2e local-only capstone + NAPI-AUDIT.md
================================================================================

TASK_TYPE:   REMEDIATION / FEATURE
STATUS:      Backlog
PRIORITY:    P0  (red-hat closeout — the slice is not done without it)
AGENT:       implementer=rust-implementer | reviewer=rust-reviewer
EFFORT:      L  (120 min)
PROPOSED-BY: rust-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-LPR-01, UC-LPR-02, UC-LPR-04, UC-LPR-07
CAPABILITIES:CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  cargo test -p but lpr_review_cli_happy_path lpr_full_local_loop_request_to_merge_no_forge && cargo test -p but-api napi_audit_lpr_verbs_route_through_gated_but_api
  check: cargo check -p but -p but-api --all-targets
  lint:  cargo clippy -p but -p but-api --all-targets

--------------------------------------------------------------------------------
TYPE DESIGN (Rust shapes this task introduces)
--------------------------------------------------------------------------------
NO new product types. Test-only helper code in two new crates/but/tests/but/command/*.rs files:
  - a fixture function returning a real governed repo via Sandbox::init_scenario_with_target_and_default_settings("one-stack") + env.invoke_bash committing .gitbutler/{permissions,gates}.toml, mirroring the governed_loop_env pattern from crates/but/tests/but/command/governed_loop.rs
  - helpers to capture/reset the forge_reviews cache count, ref IDs, and to assert no forge boundary was reached

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
The LPR slice closes out its red-hat remediation lane: (1) the happy-path `but review *` CLI suite exercises `request`, `assign`, `comment`, `comments`, `resolve`, `status`, and `request-changes` with snapbox snapshots; (2) the e2e capstone drives `request → assign → comment → resolve → approve → merge` over one governed repo with `keep_reviews_local=true`, the merge advances the target ref, and no forge network call is made; (3) `NAPI-AUDIT.md` records the R14 audit citing the passing `napi_audit_lpr_verbs_route_through_gated_but_api` test; (4) existing N-API audit test still passes and snapshots regenerate cleanly.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST add a snapbox test `lpr_review_cli_happy_path` to `crates/but/tests/but/command/lpr_review_cli_happy_path.rs` that exercises every `but review` LPR verb with `env.but(...).assert().success()` and `.stdout_eq(...)` / `.stderr_eq(...)` using `[..]` and `...` wildcards. Follow the exact pattern in `crates/but/tests/but/command/governed_loop.rs` and `crates/but/tests/but/command/review_guard.rs`.
- [MUST] MUST add an e2e test `lpr_full_local_loop_request_to_merge_no_forge` driving the full local loop with `keep_reviews_local=true`: `but review request` → `but review assign` → `but review comment` → `but review resolve` → `but review approve` → `but merge`. Assert (a) the merge proceeds (target ref advances), and (b) NO forge network call is made (`forge_reviews` cache empty/unchanged, no forge-boundary error).
- [MUST] MUST write `.spec/prds/governance/tasks/sprint-07-local-agent-pr/NAPI-AUDIT.md` documenting the R14 audit for the six LPR verbs (`request_review`, `assign_reviewer`, `post_comment`, `list_comments`, `resolve_thread`, `review_status`) and the two `principal_kind_*` verbs; reference the existing passing test `napi_audit_lpr_verbs_route_through_gated_but_api` in `crates/but-api/tests/lpr_010_audit.rs:60`.
- [MUST] MUST register the two new test modules in `crates/but/tests/but/command/mod.rs` under `#[cfg(feature = "legacy")]`.
- [MUST] MUST seed the e2e fixture with the same helpers used by `governed_loop.rs`: `Sandbox::init_scenario_with_target_and_default_settings("one-stack")`, committed `.gitbutler/permissions.toml`, `.gitbutler/gates.toml`, `env.setup_metadata`, `env.set_target_sha`, `attach_review_id`, `upsert_cached_review`.
- [NEVER] NEVER mock the database, HTTP client, or filesystem — integration-tier tests only.
- [NEVER] NEVER weaken an existing test; NEVER add new `gitbutler-*` usage.
- [NEVER] NEVER modify `crates/but-api/src/`, `crates/but-db/`, `crates/but-authz/`, the merge gate, or the review requirement gate to make a test pass. A missing CLI verb is a flag against REM-LPR-004, not a production change here.
- [STRICTLY] STRICTLY follow the snapbox conventions: use `env.but(...).assert()` with `.stdout_eq(...)` / `.stderr_eq(...)` and `[..]` wildcards; use `env.invoke_bash(...)` / `env.invoke_git(...)`, never `std::process::Command::new("git")`.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: `cargo test -p but lpr_review_cli_happy_path` passes — exercises every `but review` LPR verb with snapbox assertions on stdout/stderr/exit code
- [ ] AC-2: `cargo test -p but lpr_full_local_loop_request_to_merge_no_forge` passes — drives the full local loop and asserts merge proceeds AND no forge call is made
- [ ] AC-3: `.spec/prds/governance/tasks/sprint-07-local-agent-pr/NAPI-AUDIT.md` exists with the R14 audit for the six LPR verbs plus `principal_kind_*` verbs
- [ ] AC-4: the existing `napi_audit_lpr_verbs_route_through_gated_but_api` test still passes and snapshots regenerate cleanly with `SNAPSHOTS=overwrite cargo test -p but`
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: `lpr_review_cli_happy_path` snapbox suite passes
  GIVEN: a real governed sandbox repo seeded via `env.invoke_bash`/`env.invoke_git` with the principals and authorities required for each `but review` verb
  WHEN:  `but review request`, `but review assign`, `but review comment`, `but review comments`, `but review resolve`, `but review status`, and `but review request-changes` each run through the snapbox `env.but(...).assert()` idiom
  THEN:  every verb exits 0 with stdout/stderr matching the snapbox snapshot; the suite is happy-path only
  TEST_TIER: integration   VERIFICATION_SERVICE: real `but` CLI binary + snapbox over a sandbox git repo
  VERIFY: cargo test -p but lpr_review_cli_happy_path

AC-2: `lpr_full_local_loop_request_to_merge_no_forge` e2e capstone passes
  GIVEN: `lpr_local_only_env`: one real governed repo with `keep_reviews_local=true`, no authenticated forge, a protected main + review gate, a feature branch, and a locally-cached review ID
  WHEN:  the full loop runs in order: `but review request` → `assign` → `comment` → `resolve` → `approve` → `but merge`
  THEN:  every review step completes locally; the merge advances the target ref; the `forge_reviews` cache remains empty/unchanged and no output references a forge boundary call
  TEST_TIER: e2e-automated   VERIFICATION_SERVICE: real `but` CLI + real `but-db` + real gix via but_testsupport — no mocks, no forge network
  VERIFY: cargo test -p but lpr_full_local_loop_request_to_merge_no_forge

AC-3: `NAPI-AUDIT.md` R14 artifact exists
  GIVEN: the six LPR verbs and the two principal-kind verbs are exported as `#[but_api(napi)]` but-api functions
  WHEN:  the audit note is written at `.spec/prds/governance/tasks/sprint-07-local-agent-pr/NAPI-AUDIT.md`
  THEN:  the note documents that every listed verb routes through the gated `but-api` seam, that there is no parallel ungated N-API route, and it references `napi_audit_lpr_verbs_route_through_gated_but_api` in `crates/but-api/tests/lpr_010_audit.rs:60`
  TEST_TIER: documentation / api-contract   VERIFICATION_SERVICE: file existence + grep + passing audit test
  VERIFY: test -f .spec/prds/governance/tasks/sprint-07-local-agent-pr/NAPI-AUDIT.md && grep -q "R14" .spec/prds/governance/tasks/sprint-07-local-agent-pr/NAPI-AUDIT.md && cargo test -p but-api napi_audit_lpr_verbs_route_through_gated_but_api

AC-4: existing N-API audit test and snapshots stay green
  GIVEN: the existing `napi_audit_lpr_verbs_route_through_gated_but_api` test
  WHEN:  the new CLI tests and audit document are added
  THEN:  the audit test still passes and `SNAPSHOTS=overwrite cargo test -p but` regenerates the new snapshots without manual edits
  TEST_TIER: regression / snapshot   VERIFICATION_SERVICE: cargo tests
  VERIFY: cargo test -p but-api napi_audit_lpr_verbs_route_through_gated_but_api && SNAPSHOTS=overwrite cargo test -p but

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): `cargo test -p but lpr_review_cli_happy_path` exits 0 with all snapbox assertions green
- TC-2 (-> AC-2): `cargo test -p but lpr_full_local_loop_request_to_merge_no_forge` exits 0, target ref advances, no forge call observed
- TC-3 (-> AC-3): `NAPI-AUDIT.md` exists, contains the R14 finding, and references the existing audit test
- TC-4 (-> AC-4): the existing audit test still passes and snapshot regeneration is clean

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- cargo test -p but lpr_review_cli_happy_path   -> Exit 0
- cargo test -p but lpr_full_local_loop_request_to_merge_no_forge   -> Exit 0
- cargo test -p but-api napi_audit_lpr_verbs_route_through_gated_but_api   -> Exit 0
- SNAPSHOTS=overwrite cargo test -p but   -> Exit 0
- cargo check -p but -p but-api --all-targets   -> Exit 0
- cargo clippy -p but -p but-api --all-targets   -> Exit 0
- cargo fmt --check   -> Exit 0

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but/tests/lpr_review_cli_happy_path.rs NEW
  - crates/but/tests/lpr_full_local_loop_request_to_merge_no_forge.rs NEW
  - crates/but/tests/snapshots/ NEW
  - crates/but/tests/but/command/mod.rs (MODIFY — add the two new modules)
  - .spec/prds/governance/tasks/sprint-07-local-agent-pr/NAPI-AUDIT.md NEW
writeProhibited:
  - crates/but-api/src/
  - crates/but-db/
  - crates/but-authz/
  - the merge gate (`crates/but-api/src/legacy/merge_gate.rs`)
  - the review requirement gate (`crates/but-api/src/legacy/review_requirement.rs`)
  - any existing test file not listed in writeAllowed
  - any gitbutler-* crate
  - Any file not in write_allowed

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: REM-LPR-004 (the `comment`/`comments`/`resolve` CLI verbs must be wired before the snapbox suite can exercise them).
Blocks:     nothing.
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "REM-LPR-010",
  "proposed_by": "rust-planner",
  "verification_policy": { "requires_tests": true, "requires_red_evidence": true, "requires_seeded_evidence": true },
  "fixtures": {
    "lpr_local_only_env": {
      "description": "A real governed repo via Sandbox::init_scenario_with_target_and_default_settings('one-stack') + env.invoke_bash committing .gitbutler/permissions.toml and gates.toml with a protected main + review gate. Attaches a review ID and seeds a local ForgeReview cache entry (no forge credentials). keep_reviews_local defaults true. Principals: auth (opener, pull_requests:write) and rev (reviewer, reviews:write+comments:write).",
      "seed_method": "public_api",
      "records": [
        "Sandbox::init_scenario_with_target_and_default_settings('one-stack')",
        "invoke_bash committing .gitbutler/{permissions,gates}.toml",
        "env.set_target_sha('refs/heads/main')",
        "env.setup_metadata(&[branch_name])",
        "attach_review_id(&env, branch_name, review_id)",
        "upsert_cached_review(&env, branch_name, review_id) — local cache only",
        "BUT_AGENT_HANDLE per-step to auth or rev"
      ]
    }
  },
  "requirements": [
    { "id": "AC-1", "type": "acceptance_criterion", "primary": true, "description": "GIVEN a real governed sandbox repo WHEN each LPR but review subcommand runs via snapbox env.but(...).assert() THEN every command exits 0 and stdout/stderr match the snapbox snapshot with [..]/... wildcards", "verify": "cargo test -p but lpr_review_cli_happy_path", "scenario": { "tier": "visible", "test_tier": "integration", "verification_service": "real but CLI binary + snapbox over a sandbox git repo", "negative_control": { "would_fail_if": ["a CLI subcommand is missing or returns non-zero", "stdout/stderr changes not covered by wildcards", "test uses direct stdout assertions instead of snapbox"] }, "evidence": { "artifact_type": "test_output", "required_capture": true }, "cases": [ { "start_ref": "lpr_local_only_env", "action": { "actor": "ci", "steps": ["run but review request", "run but review assign", "run but review comment", "run but review comments", "run but review resolve", "run but review status", "run but review request-changes"] }, "end_state": { "must_observe": ["every subcommand exits 0", "snapbox snapshots match"], "must_not_observe": ["governance denials on happy path", "unmatched volatile output"] } } ] } },
    { "id": "AC-2", "type": "acceptance_criterion", "primary": true, "description": "GIVEN lpr_local_only_env with keep_reviews_local=true WHEN the full local loop runs in order ending with governed local merge THEN the merge proceeds (target ref advances) and NO forge network call is made", "verify": "cargo test -p but lpr_full_local_loop_request_to_merge_no_forge", "scenario": { "tier": "visible", "test_tier": "e2e-automated", "verification_service": "real but CLI + real but-db + real gix via but_testsupport, no forge, no mocks", "negative_control": { "would_fail_if": ["the merge is denied after an approve@head (gate break)", "the target ref does not advance after merge", "a forge_reviews cache row is created or output references a forge boundary call"] }, "evidence": { "artifact_type": "test_output", "required_capture": true }, "cases": [ { "start_ref": "lpr_local_only_env", "action": { "actor": "ci", "steps": ["but review request <branch> --reviewer rev", "but review assign <branch> --reviewer rev", "but review comment <branch> -m ...", "but review resolve <branch> <thread>", "but review approve <branch>", "but merge <branch>"] }, "end_state": { "must_observe": ["all review steps exit 0", "target ref advances", "forge_reviews cache empty/unchanged"], "must_not_observe": ["forge merge_review boundary error", "network-backed PR creation", "a new forge_reviews row"] } } ] } },
    { "id": "AC-3", "type": "acceptance_criterion", "primary": true, "description": "GIVEN the six LPR verbs and two principal_kind verbs are #[but_api(napi)] but-api functions WHEN the R14 audit note is written THEN it documents that every listed verb routes through the gated but-api seam and references crates/but-api/tests/lpr_010_audit.rs:60", "verify": "test -f .spec/prds/governance/tasks/sprint-07-local-agent-pr/NAPI-AUDIT.md && grep -q \"R14\" .spec/prds/governance/tasks/sprint-07-local-agent-pr/NAPI-AUDIT.md && cargo test -p but-api napi_audit_lpr_verbs_route_through_gated_but_api", "scenario": { "tier": "visible", "test_tier": "documentation / api-contract", "verification_service": "file existence + grep + existing audit test", "negative_control": { "would_fail_if": ["NAPI-AUDIT.md is missing", "document does not mention R14 or verbs", "document does not reference the audit test"] }, "evidence": { "artifact_type": "file_artifact", "required_capture": true }, "cases": [ { "start_ref": "lpr_local_only_env", "action": { "actor": "implementer", "steps": ["write NAPI-AUDIT.md with R14 finding", "list the six LPR verbs + principal_kind_* verbs", "reference napi_audit_lpr_verbs_route_through_gated_but_api at line 60"] }, "end_state": { "must_observe": ["file exists", "R14 claim present", "all 8 verbs named", "audit test cited"], "must_not_observe": ["claims of closed R18/R22 residuals", "ungated N-API route claimed"] } } ] } },
    { "id": "AC-4", "type": "acceptance_criterion", "primary": true, "description": "GIVEN the existing napi_audit_lpr_verbs_route_through_gated_but_api test WHEN the new tests are added and snapshots regenerated THEN the audit test still passes and snapshot regeneration is clean", "verify": "cargo test -p but-api napi_audit_lpr_verbs_route_through_gated_but_api && SNAPSHOTS=overwrite cargo test -p but", "scenario": { "tier": "visible", "test_tier": "regression / snapshot", "verification_service": "cargo tests", "negative_control": { "would_fail_if": ["the audit test fails after remediation", "SNAPSHOTS=overwrite leaves uncommitted hand-edits"] }, "evidence": { "artifact_type": "test_output", "required_capture": true }, "cases": [ { "start_ref": "lpr_local_only_env", "action": { "actor": "ci", "steps": ["run the audit test", "run SNAPSHOTS=overwrite cargo test -p but", "verify no manual snapshot edits remain"] }, "end_state": { "must_observe": ["audit test passes", "snapshots regenerate cleanly"], "must_not_observe": ["audit test regression", "hand-edited snapshots"] } } ] } },
    { "id": "TC-1", "type": "test_criterion", "description": "cargo test -p but lpr_review_cli_happy_path exits 0 with green snapbox snapshots", "verify": "cargo test -p but lpr_review_cli_happy_path", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "cargo test -p but lpr_full_local_loop_request_to_merge_no_forge exits 0, target ref advances, no forge call observed", "verify": "cargo test -p but lpr_full_local_loop_request_to_merge_no_forge", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "NAPI-AUDIT.md exists, contains R14, names verbs, references the existing audit test", "verify": "test -f .spec/prds/governance/tasks/sprint-07-local-agent-pr/NAPI-AUDIT.md && grep -q \"R14\" .spec/prds/governance/tasks/sprint-07-local-agent-pr/NAPI-AUDIT.md && cargo test -p but-api napi_audit_lpr_verbs_route_through_gated_but_api", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "the existing N-API audit test still passes and snapshot regeneration is clean", "verify": "cargo test -p but-api napi_audit_lpr_verbs_route_through_gated_but_api && SNAPSHOTS=overwrite cargo test -p but", "maps_to_ac": "AC-4" }
  ]
}
-->
