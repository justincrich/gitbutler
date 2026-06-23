# REM-LPR-004: Re-point `but review comment` at `post_comment`; add Comments/Resolve CLI subcommands + --file/--line/--thread; delete dead `comment_review`

> Status: Backlog
> Commit: (none yet)
> Reviewer: rust-reviewer
> Updated: 2026-06-22T18:00:00Z
> PROPOSED-BY: rust-planner

## What this does

Close **C1 CRITICAL** from `.spec/reviews/red-hat-20260622-173510.md`: stop the `but review comment` CLI verb from calling the dead `comment_review` stub (`crates/but-api/src/legacy/forge.rs:838-852`) and wire it to the real `post_comment` API (`forge.rs:878-910`). Add the missing `but review comments` and `but review resolve <thread_id>` CLI verbs, then delete the dead `comment_review` function and its `#[but_api(napi)]` annotation.

## Why

Sprint 07 / PRD UC-LPR-02 gives the local review layer a file/line-anchored, threaded, resolvable comment surface. The backend verbs (`post_comment`, `list_comments`, `resolve_thread`) are implemented and tested, but the CLI facade still authenticates `CommentsWrite` and returns `task_contract_invalid` without writing anything. That makes the comment thread unreachable from `but review`.

## How to verify

PRIMARY **AC-1** — `cargo test -p but review_comment_cli`: run `but review comment feat -m "fix this" --file f.rs --line 12 --thread t1` as a `comments:write` holder and assert that `but review comments feat --format json` shows a `t1` thread with `resolved=false`, `file="f.rs"`, and `line=12`.

## Scope

- `crates/but/src/args/forge.rs` (MODIFY — extend `Comment` with `--file`/`--line`/`--thread`; add `Comments` and `Resolve` variants)
- `crates/but/src/command/legacy/forge/review.rs` (MODIFY — `comment()` calls `post_comment`; add `comments()` + `resolve()`)
- `crates/but/src/lib.rs` (MODIFY — dispatch `Comments`/`Resolve` inside the `Subcommands::Pr` block)
- `crates/but/src/utils/metrics.rs` (MODIFY — map the new `Subcommands` variants so the match stays exhaustive)
- `crates/but/src/args/tests.rs` (MODIFY only if an exhaustive Clap-variant match or a struct-literal `Comment { ... }` breaks)
- `crates/but-api/src/legacy/forge.rs` (DELETE `comment_review` and its `#[but_api(napi)]` annotation)
- `crates/but-api/tests/forge_guard.rs` (MODIFY — replace stale `comment_review` stub assertions with `post_comment` equivalents, or remove them)
- `crates/but/tests/but/command/review_guard.rs` (MODIFY — replace the dead-comment assertion with the real `post_comment` success path)
- `crates/but/tests/but/command/review_comments.rs` (NEW — snapbox CLI tests for AC-1..AC-4)
- `crates/but/tests/but/command/mod.rs` (MODIFY — register the new test module)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: REM-LPR-004 — Re-point review comment at post_comment; add Comments/Resolve CLI verbs; delete comment_review
================================================================================

TASK_TYPE:   REMEDIATION
STATUS:      Backlog
PRIORITY:    P0  (CRITICAL red-hat C1; the CLI comment surface is currently a facade)
AGENT:       implementer=rust-implementer | reviewer=rust-reviewer
EFFORT:      S  (60 min)
PROPOSED-BY: rust-planner
SPRINT:      .spec/prds/governance/tasks/sprint-07-local-agent-pr/SPRINT.md
PRD_REFS:    UC-LPR-02
CAPABILITIES:CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  cargo test -p but review_comment_cli
  check: cargo check -p but --all-targets
  lint:  cargo clippy -p but --all-targets

--------------------------------------------------------------------------------
TYPE DESIGN (Rust shapes this task introduces)
--------------------------------------------------------------------------------
CLI-ONLY CHANGES. No new product types. This task modifies:
  - `crates/but/src/args/forge.rs::pr::Subcommands::Comment` (args/forge.rs:97-104): add `file: Option<String>`, `line: Option<u32>`, `thread_id: Option<String>`.
  - New variants `Comments { branch: String }` and `Resolve { branch: String, thread_id: String }` inserted after `Comment` and before `Close` in the same enum.
  - `crates/but/src/command/legacy/forge/review.rs::comment()` (review.rs:97-112): change signature to accept `file`, `line`, `thread_id`, then call `but_api::legacy::forge::post_comment(...)`.
  - New functions `comments(ctx, branch, out)` and `resolve(ctx, branch, thread_id, out)` in review.rs after `comment()`, mirroring the shape of `approve()`/`request()`.
  - `crates/but/src/lib.rs` `Subcommands::Pr` dispatch block (around lib.rs:1389-1393 for the existing `Comment` arm): add arms for `Comments` and `Resolve`.
  - `crates/but/src/utils/metrics.rs` match at lines 152-172: add `Comments { .. } => PrNew` and `Resolve { .. } => PrNew` so the command-to-metric mapping stays exhaustive.
ERROR STRATEGY:
  - All command entry points return `anyhow::Result<(), CliError>` and map `but_api` errors through the existing `review_gate_cli_error` (review.rs:213-234). A missing `CommentsWrite` authority therefore prints the structured `{code:"perm.denied", message, remediation_hint}` envelope to stderr and exits 1.
OWNERSHIP PLAN:
  - CLI receives owned `String`/`Option` values from clap and moves them directly into the `but_api` call. `comments()` builds an owned `Vec<but_db::LocalReviewComment>` from `list_comments` and serializes it with `out.write_value()` for `--format json`; for human output print a simple thread listing.

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
After this remediation: `but review comment <branch> -m <msg> [--file <f>] [--line <n>] [--thread <t>]` writes a real `local_review_comments` row through `post_comment`; `but review comments <branch>` lists branch threads via `list_comments`; `but review resolve <branch> <thread_id>` flips a thread via `resolve_thread`; the dead `comment_review` `#[but_api(napi)]` stub is gone; stale tests in `forge_guard.rs` and `review_guard.rs` are updated to expect the real behavior; and the 7 existing `local_review_comments_verbs.rs` tests remain green.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST re-point `crates/but/src/command/legacy/forge/review.rs:103` from `but_api::legacy::forge::comment_review(...)` to `but_api::legacy::forge::post_comment(...)`. The `comment()` function signature is extended at `review.rs:97-112`.
- [MUST] MUST add `Comments { branch: String }` and `Resolve { branch: String, thread_id: String }` variants to the `Subcommands` enum in `crates/but/src/args/forge.rs`. Insert them immediately after the existing `Comment` variant (args/forge.rs:97-104) and before `Close` (args/forge.rs:105).
- [MUST] MUST add `--file`, `--line`, and `--thread` arguments to the existing `Comment` variant in `crates/but/src/args/forge.rs:97-104`. `--file` is `Option<String>`, `--line` is `Option<u32>`, `--thread` is `Option<String>` defaulting to `"default"` so bare `but review comment <branch> -m <msg>` continues to work.
- [MUST] MUST dispatch the new `Comments` and `Resolve` variants in `crates/but/src/lib.rs` inside the existing `Subcommands::Pr(forge::pr::Platform { cmd, .. })` match block. The existing `Comment` dispatch lives at `lib.rs:1389-1393`; add the new arms right after it and before `Close`.
- [MUST] MUST DELETE the dead `comment_review` function and its adjacent `#[but_api(napi)]` annotation from `crates/but-api/src/legacy/forge.rs:838-852`. After deletion, `rg "fn comment_review" crates/but-api/src/` must return zero matches.
- [MUST] MUST add new snapbox CLI tests in `crates/but/tests/but/command/review_comments.rs` covering AC-1..AC-4, and register the module in `crates/but/tests/but/command/mod.rs`.
- [MUST] MUST route all errors through the existing `review_gate_cli_error` (review.rs:213-234) so a denied call prints a JSON `{code:"perm.denied", ...}` envelope to stderr and exits 1.
- [MUST] MUST preserve `post_comment`'s existing `CommentsWrite` authority gate (`forge.rs:894-895`) exactly as implemented. Do not bypass, reorder, or weaken it.
- [MUST] MUST NOT weaken, remove, or alter the 7 existing tests in `crates/but-api/tests/local_review_comments_verbs.rs`. They must stay green.
- [MUST] MUST update stale `comment_review` references in `crates/but-api/tests/forge_guard.rs` (lines 105, 171, 181, 188) and `crates/but/tests/but/command/review_guard.rs` (lines 148-167) so the codebase compiles and the tests still assert the correct authority/denial behavior against the real `post_comment`.
- [MUST] MUST update `crates/but/src/utils/metrics.rs:152-172` to include the new `Comments` and `Resolve` match arms (map both to `PrNew`, matching `Comment`) so the exhaustive match compiles.
- [NEVER] NEVER reintroduce, wrap, or keep `comment_review` as the CLI target.
- [NEVER] NEVER add a new `Authority` variant; `CommentsWrite` is the correct and only authority for these writes.
- [NEVER] NEVER hand-edit `packages/but-sdk/src/generated/**`. The SDK regeneration gate belongs to LPR-010.
- [NEVER] NEVER weaken the resolver-identity constraint in `resolve_thread` (R22).
- [STRICTLY] STRICTLY keep `post_comment`/`list_comments`/`resolve_thread` implementations unchanged except for deleting the adjacent `comment_review`.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: `but review comment <branch> -m "..." --file f.rs --line 12 --thread t1` writes a `local_review_comments` row (`resolved=false`) with correct anchors
- [ ] AC-2: `but review comments <branch>` lists threads
- [ ] AC-3: `but review resolve <branch> <thread_id>` flips all `t1` rows to `resolved=true`
- [ ] AC-4: missing-authority denial produces `perm.denied` + exit 1, no row written
- [ ] AC-5: dead `comment_review` is gone (`rg "fn comment_review" crates/but-api/src/` returns 0 matches)
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: CLI comment writes a `local_review_comments` row with `resolved=false` and correct file/line/thread anchors
  GIVEN: `lpr_cli_governed_repo` with `reviewer` holding `comments:write` and a `feat` branch
  WHEN:  `but review comment feat -m "fix this" --file f.rs --line 12 --thread t1` runs as `reviewer`
  THEN:  the command exits 0; a subsequent `but review comments feat --format json` shows a `t1` thread whose comment has `body="fix this"`, `file=Some("f.rs")`, `line=Some(12)`, and `resolved=false`
  TEST_TIER: integration   VERIFICATION_SERVICE: real `but` CLI + real `but-db` via `but_testsupport::Sandbox`
  VERIFY: `cargo test -p but review_comment_cli`

AC-2: CLI `comments` lists branch threads
  GIVEN: `lpr_cli_governed_repo` with two comments on thread `t1` and one on thread `t2` for `feat`
  WHEN:  `but review comments feat --format json` runs
  THEN:  the JSON output contains both threads; `t1` has 2 comments and `t2` has 1; each entry carries `file`, `line`, `resolved`, and `body`
  TEST_TIER: integration   VERIFICATION_SERVICE: real `but` CLI + `but_api::legacy::forge::list_comments`
  VERIFY: `cargo test -p but review_comments_cli`

AC-3: CLI `resolve` flips a thread to `resolved=true`
  GIVEN: `lpr_cli_governed_repo` with an unresolved `t1` thread (2 comments) on `feat`; `reviewer` holds `comments:write` and is a permitted resolver (R22)
  WHEN:  `but review resolve feat t1` runs as `reviewer`, then `but review comments feat --format json`
  THEN:  the resolve command exits 0; every `t1` comment in the listing now has `resolved=true`; `t2` comments are unchanged
  TEST_TIER: integration   VERIFICATION_SERVICE: real `but` CLI + `but_api::legacy::forge::resolve_thread`
  VERIFY: `cargo test -p but review_resolve_cli`

AC-4: missing-authority denial is structured `perm.denied` with no row written
  GIVEN: `lpr_cli_governed_repo` with `dev` holding `contents:write` but NOT `comments:write`; the comment store is empty
  WHEN:  `but review comment feat -m "x" --thread t1` runs as `dev`
  THEN:  the command exits 1; stderr contains JSON with `"code":"perm.denied"` and names `comments:write`; no `local_review_comments` row is written
  TEST_TIER: api-contract   VERIFICATION_SERVICE: real `but` CLI composing `authorize_branch_action` + real `but-authz`
  VERIFY: `cargo test -p but review_comment_cli_denied`

AC-5: dead `comment_review` function is removed
  GIVEN: the codebase after this remediation
  WHEN:  `rg "fn comment_review" crates/but-api/src/` is run
  THEN:  zero matches; the `#[but_api(napi)] comment_review` stub and its function body are deleted
  TEST_TIER: build-gate   VERIFICATION_SERVICE: `rg` over `crates/but-api/src/`
  VERIFY: `rg "fn comment_review" crates/but-api/src/`

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): `but review comment` with `--file/--line/--thread` writes a resolved=false comment with correct anchors
- TC-2 (-> AC-2): `but review comments` lists branch threads grouped by `thread_id`
- TC-3 (-> AC-3): `but review resolve` flips every comment in the named thread to `resolved=true`
- TC-4 (-> AC-4): `but review comment` as a principal without `comments:write` exits 1 with `perm.denied` naming `comments:write` and writes no row
- TC-5 (-> AC-5): `rg "fn comment_review" crates/but-api/src/` returns zero matches
- TC-6 (-> AC-1..AC-4): the 7 existing `local_review_comments_verbs.rs` integration tests remain green and unweakened

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- cargo test -p but review_comment_cli   -> Exit 0
- cargo test -p but review_comments_cli   -> Exit 0
- cargo test -p but review_resolve_cli   -> Exit 0
- cargo test -p but review_comment_cli_denied   -> Exit 0
- cargo test -p but-api --test local_review_comments_verbs   -> Exit 0 (unchanged)
- cargo test -p but-api forge_guard   -> Exit 0
- cargo test -p but review_guard   -> Exit 0
- rg "fn comment_review" crates/but-api/src/   -> Zero matches
- cargo check -p but --all-targets   -> Exit 0
- cargo clippy -p but --all-targets   -> Exit 0
- cargo fmt --check   -> Exit 0

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but/src/args/forge.rs (MODIFY)
  - crates/but/src/command/legacy/forge/review.rs (MODIFY)
  - crates/but/src/lib.rs (MODIFY — dispatch)
  - crates/but/src/utils/metrics.rs (MODIFY — exhaustive match)
  - crates/but/src/args/tests.rs (MODIFY only if a struct-literal or exhaustive Clap match breaks)
  - crates/but-api/src/legacy/forge.rs (DELETE only comment_review + its #[but_api(napi)] at lines 838-852)
  - crates/but-api/tests/forge_guard.rs (MODIFY — replace stale comment_review assertions)
  - crates/but/tests/but/command/review_guard.rs (MODIFY — replace dead-comment assertion with real post_comment path)
  - crates/but/tests/but/command/review_comments.rs (NEW — snapbox CLI tests)
  - crates/but/tests/but/command/mod.rs (MODIFY — register review_comments module)
writeProhibited:
  - crates/but-api/src/legacy/forge.rs post_comment/list_comments/resolve_thread implementations (CONSUME-only)
  - crates/but-api/tests/local_review_comments_verbs.rs (CONSUME-only — never weaken or remove existing tests)
  - crates/but-db/** (CONSUME-only; no schema change)
  - crates/but-authz/src/authority.rs (no new Authority variant)
  - any `gitbutler-*` crate (no new gitbutler-* usage)
  - packages/but-sdk/src/generated/** (NEVER hand-edit; LPR-010 owns SDK regen)

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agent: implementer=rust-implementer | reviewer=rust-reviewer
rationale: Surgical CLI remediation. Requires modifying clap definitions, the CLI command dispatch, the stale test references, and deleting a dead but_api function — all within the established Rust patterns. rust-implementer executes; rust-reviewer validates that `comment_review` is truly gone, that `post_comment`'s CommentsWrite gate is preserved, that the new CLI verbs route through `review_gate_cli_error`, and that no existing test is weakened.

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: LPR-004 backend (post_comment/list_comments/resolve_thread already landed at master HEAD b8848c29fe)
Blocks:     REM-LPR-010 (SDK regen for the removed comment_review + the new verbs; CLI happy-path snapbox suite references these verbs)
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "REM-LPR-004",
  "proposed_by": "rust-planner",
  "verification_policy": { "requires_tests": true, "requires_red_evidence": true, "requires_seeded_evidence": true },
  "fixtures": {
    "lpr_cli_governed_repo": {
      "description": "A real governed repo via but_testsupport::Sandbox::init_scenario_with_target_and_default_settings('one-stack') plus a governance commit to refs/heads/main and refs/heads/feat containing .gitbutler/permissions.toml. Principals: reviewer (reviews:write, comments:write, contents:read), dev (contents:write, pull_requests:write), ro (contents:read). The feat branch exists in the workspace.",
      "seed_method": "cli_fixture",
      "records": [
        "Sandbox::init_scenario_with_target_and_default_settings('one-stack') + invoke_bash governance commit",
        ".gitbutler/permissions.toml defines principals reviewer/dev/ro",
        "env.but('setup').assert().success(); env.setup_metadata(&['feat'])"
      ]
    }
  },
  "requirements": [
    { "id": "AC-1", "type": "acceptance_criterion", "primary": true, "description": "GIVEN lpr_cli_governed_repo with reviewer holding comments:write on feat WHEN but review comment feat -m 'fix this' --file f.rs --line 12 --thread t1 runs THEN the command exits 0 and a subsequent but review comments feat --format json shows a t1 thread whose comment has body='fix this', file=Some('f.rs'), line=Some(12), resolved=false", "verify": "cargo test -p but review_comment_cli", "scenario": { "tier": "visible", "test_tier": "integration", "verification_service": "real but CLI + real but-db local_review_comments via but_testsupport::Sandbox", "negative_control": { "would_fail_if": ["comment() still calls comment_review (no row written)", "the CLI omits file/line/thread arguments", "post_comment's CommentsWrite gate is bypassed", "the returned listing shows resolved=true at insert time"] }, "evidence": { "artifact_type": "api_response", "required_capture": true }, "cases": [ { "start_ref": "lpr_cli_governed_repo", "action": { "actor": "reviewer", "steps": ["BUT_AGENT_HANDLE=reviewer", "but review comment feat -m 'fix this' --file f.rs --line 12 --thread t1", "but review comments feat --format json"] }, "end_state": { "must_observe": ["exit 0", "t1 thread present", "body='fix this'", "file='f.rs'", "line=12", "resolved=false"], "must_not_observe": ["comment_review invocation", "resolved=true at insert", "missing thread or anchors"] } } ] } },
    { "id": "AC-2", "type": "acceptance_criterion", "primary": false, "description": "GIVEN lpr_cli_governed_repo with two comments on thread t1 and one on thread t2 for feat WHEN but review comments feat --format json runs THEN the output contains both threads, t1 has 2 comments, t2 has 1, and each entry carries file/line/resolved/body", "verify": "cargo test -p but review_comments_cli", "scenario": { "tier": "holdout", "test_tier": "integration", "verification_service": "real but CLI list_comments + but_db list_by_target", "negative_control": { "would_fail_if": ["list_comments returns a flat list with no thread grouping", "the __pr_meta__ reserved thread leaks into the listing", "a stub returns an empty list despite seeded comments"] }, "evidence": { "artifact_type": "api_response", "required_capture": true }, "cases": [ { "start_ref": "lpr_cli_governed_repo", "action": { "actor": "reviewer", "steps": ["seed 2 comments on t1 and 1 on t2 via but review comment", "but review comments feat --format json"] }, "end_state": { "must_observe": ["t1 has 2 comments", "t2 has 1 comment", "each entry has file/line/resolved/body"], "must_not_observe": ["__pr_meta__ thread in output", "0 comments returned"] } } ] } },
    { "id": "AC-3", "type": "acceptance_criterion", "primary": false, "description": "GIVEN lpr_cli_governed_repo with an unresolved t1 thread (2 comments) on feat and reviewer as a permitted resolver under R22 WHEN but review resolve feat t1 runs as reviewer THEN resolve exits 0 and every t1 comment has resolved=true while t2 comments remain unchanged", "verify": "cargo test -p but review_resolve_cli", "scenario": { "tier": "holdout", "test_tier": "integration", "verification_service": "real but CLI resolve_thread + but_db set_resolved", "negative_control": { "would_fail_if": ["resolve_thread flips only the first comment (LIMIT 1 bug)", "a third unrelated principal is allowed to resolve (R22 bypass)", "resolve mutates local_review_verdicts or local_review_assignments"] }, "evidence": { "artifact_type": "api_response", "required_capture": true }, "cases": [ { "start_ref": "lpr_cli_governed_repo", "action": { "actor": "reviewer", "steps": ["seed unresolved t1 (2 comments) and t2 (1 comment)", "but review resolve feat t1", "but review comments feat --format json"] }, "end_state": { "must_observe": ["resolve command exits 0", "both t1 comments have resolved=true", "t2 comment resolved=false"], "must_not_observe": ["any t1 comment resolved=false after resolve", "t2 comment flipped"] } } ] } },
    { "id": "AC-4", "type": "acceptance_criterion", "primary": false, "description": "GIVEN lpr_cli_governed_repo with dev holding contents:write but NOT comments:write and an empty comment store WHEN but review comment feat -m 'x' --thread t1 runs as dev THEN the command exits 1 with stderr JSON containing 'code':'perm.denied' naming comments:write and no local_review_comments row is written", "verify": "cargo test -p but review_comment_cli_denied", "scenario": { "tier": "visible", "test_tier": "api-contract", "verification_service": "real but CLI composing but_api::legacy::forge::post_comment + but-authz Denial", "negative_control": { "would_fail_if": ["dev is allowed to write a comment despite lacking CommentsWrite", "the denial is a plain string instead of structured perm.denied", "the denial names the wrong authority", "a row is written on the denied path"] }, "evidence": { "artifact_type": "test_output", "required_capture": true }, "cases": [ { "start_ref": "lpr_cli_governed_repo", "action": { "actor": "dev", "steps": ["BUT_AGENT_HANDLE=dev", "but --format json review comment feat -m 'x' --thread t1"] }, "end_state": { "must_observe": ["exit 1", "stderr contains 'code':'perm.denied'", "stderr contains 'comments:write'", "local_review_comments remains empty"], "must_not_observe": ["exit 0", "a comment row written", "denial naming a different authority"] } } ] } },
    { "id": "AC-5", "type": "acceptance_criterion", "primary": false, "description": "GIVEN the codebase after remediation WHEN rg 'fn comment_review' crates/but-api/src/ is run THEN zero matches and the dead #[but_api(napi)] comment_review stub is deleted", "verify": "rg \"fn comment_review\" crates/but-api/src/", "scenario": { "tier": "visible", "test_tier": "build-gate", "verification_service": "rg over crates/but-api/src/", "negative_control": { "would_fail_if": ["comment_review function body remains", "only the call site in crates/but/src was removed but the but_api definition is still present", "the function was renamed but the stub behavior still exists"] }, "evidence": { "artifact_type": "test_output", "required_capture": true }, "cases": [ { "start_ref": "codebase", "action": { "actor": "ci", "steps": ["rg 'fn comment_review' crates/but-api/src/"] }, "end_state": { "must_observe": ["zero matches"], "must_not_observe": ["any definition of fn comment_review under crates/but-api/src/"] } } ] } },
    { "id": "TC-1", "type": "test_criterion", "description": "but review comment with --file/--line/--thread writes a resolved=false comment with correct anchors", "verify": "cargo test -p but review_comment_cli", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "but review comments lists branch threads grouped by thread_id", "verify": "cargo test -p but review_comments_cli", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "but review resolve flips every comment in the named thread to resolved=true", "verify": "cargo test -p but review_resolve_cli", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "but review comment without comments:write exits 1 with perm.denied naming comments:write and writes no row", "verify": "cargo test -p but review_comment_cli_denied", "maps_to_ac": "AC-4" },
    { "id": "TC-5", "type": "test_criterion", "description": "rg 'fn comment_review' crates/but-api/src/ returns zero matches", "verify": "rg \"fn comment_review\" crates/but-api/src/", "maps_to_ac": "AC-5" },
    { "id": "TC-6", "type": "test_criterion", "description": "the 7 existing local_review_comments_verbs.rs integration tests remain green and unweakened", "verify": "cargo test -p but-api --test local_review_comments_verbs", "maps_to_ac": "AC-1,AC-2,AC-3,AC-4" }
  ]
}
-->
