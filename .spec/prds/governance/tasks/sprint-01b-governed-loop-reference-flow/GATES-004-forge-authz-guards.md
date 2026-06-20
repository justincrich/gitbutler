# GATES-004: Submit-review / open-PR / comment / close authz guards on the forge boundary + the governed `but pr`/`but review` verbs

## What this does

Permission-checks the governed forge actions at the `but-api` boundary with PRE-CALL `authorize()` guards (the async `ThreadSafeContext` shape — before the `.await`, not `_with_perm`), and adds the genuinely-missing governed CLI verbs under the EXISTING `but pr`/`but review` heading: `but pr close`, `but review approve`, `but review request-changes`, `but review comment`. `but review approve` records a real, local, head-pinned approving verdict into GATES-002's `local_review_verdicts`. Open-PR → `pull_requests:write`, review → `reviews:write`, comment → `comments:write`, close → `pull_requests:write`.

## Why

Sprint 01b · PRD UC-LOOP-01, UC-AUTHZ-02 · capability CAP-AUTHZ-01. The LOOP demo's "open a PR" / "submit a review" steps gate on these actions; `but review approve` is the governed channel that feeds the merge gate's review requirement.

## How to verify

PRIMARY **AC-1** — `cargo test -p but review_guard_reviews_write_denied` (integration; the `authorize()` guard fires before any forge `.await`, no live forge needed). Full gate set in the spec below.

## Scope

- `crates/but-api/src/legacy/forge.rs` (MODIFY) — add the pre-call `authorize()` guard at the top of `publish_review` (pull_requests:write) + NEW guarded async actions for approve/request-changes (reviews:write), comment (comments:write), close PR (pull_requests:write); for approve, write the `local_review_verdicts` row at the current head
- `crates/but-api/Cargo.toml` (MODIFY) — add `but-authz` workspace dep (but-db already present)
- `crates/but-api/tests/forge_guard.rs` (NEW) — integration tests for the denial guards (no live forge reached)
- `crates/but/src/args/forge.rs` (MODIFY) — add `Close`/`Approve`/`RequestChanges`/`Comment` to `forge::pr::Subcommands`
- `crates/but/src/lib.rs` (MODIFY) — dispatch arms for the new subcommands in the `Subcommands::Pr` match; surface the Denial as the structured exit-1 contract
- `crates/but/src/command/legacy/forge/review.rs` (MODIFY) — `approve`/`request_changes`/`comment`/`close` CLI helpers calling the guarded but-api actions via `ctx.to_sync()`
- `crates/but/src/utils/metrics.rs` (MODIFY) — extend the exhaustive `Subcommands::Pr` metrics match for the new variants
- `crates/but/tests/but/command/review_guard.rs` (NEW) — CLI snapbox denial/accept tests + local_review_verdicts assertions

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: GATES-004 - Submit-review / open-PR / comment / close authz guards on the forge boundary + the governed but pr/but review verbs
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Complete
PRIORITY:   P0
EFFORT:     M  (150 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-LOOP-01, UC-AUTHZ-02
CAPABILITIES: CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  cargo test -p but review_guard   |   cargo test -p but-api forge_guard
  check: cargo check -p but-api -p but --all-targets
  lint:  cargo clippy -p but-api -p but --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Integration tests are green against real but-api + real config + real but-db: a principal lacking reviews:write is denied a review with perm.denied naming reviews:write (exit 1, no forge call); a reviewer (reviews:write, no contents:write) is denied a commit but its `but review approve` is accepted and writes an approving local_review_verdicts row at the current head; an unset/empty/unknown handle is rejected perm.denied; a comment route gates on comments:write independently. A [build-gate] grep proves `but pr new`/`but pr close`/`but review approve`/`request-changes`/`comment` exist at the but-api boundary, each wrapped with its Authority, with no role name in the guard path (T-LOOP-014). The open-PR live-forge accept path is NOT integration-tested (proven structurally).

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST gate each governed async forge action with a PRE-CALL authorize() guard BEFORE the .await — per 04-api-design.md shape (b): `let principal = principal_from_env(&ctx)?; authorize(&principal, Authority::X)?;` then the existing `*_impl().await`. The forge actions in crates/but-api/src/legacy/forge.rs are `pub async fn (ctx: ThreadSafeContext, ...)` — they CANNOT take a repo-permission param (but-api-macros:560 rejects it on ThreadSafeContext), so this is a plain pre-call guard, NOT _with_perm.
- [MUST] MUST map the route→Authority table: open PR / create review (publish_review) → pull_requests:write; close PR (new) → pull_requests:write; submit review approve/request-changes (new) → reviews:write; comment (new) → comments:write. Add ONLY these guards; the merge/auto-merge gate is GATES-003.
- [MUST] MUST resolve the acting principal from BUT_AGENT_HANDLE via AUTHZ-003's resolver, loading the effective AuthoritySet from committed config at the TARGET ref (AUTHZ-002), and FAIL CLOSED: unset / empty-string / unknown-handle → perm.denied, exit 1, no forge call made.
- [MUST] MUST ADD the missing governed verbs as NEW subcommands of the EXISTING forge::pr::Subcommands enum in crates/but/src/args/forge.rs (Close, Approve, RequestChanges, Comment) and dispatch them in the EXISTING Subcommands::Pr(...) match arm in crates/but/src/lib.rs — alongside New/AutoMerge/SetDraft/SetReady/Template. Do NOT introduce a parallel create/close top-level verb that duplicates `but pr new`.
- [MUST] MUST make `but review approve` record an APPROVING verdict into GATES-002's local_review_verdicts AT THE CURRENT HEAD of the target under review (id=fresh uuid, target=the PR/stack/branch ref, principal_id=resolved handle, verdict="approved", head_oid=the resolved current head sha, created_at=now) — the LOCAL, real, testable accept path; it does NOT require a live forge.
- [MUST] MUST print the ref-pin caveat "takes effect once committed to the target branch." on each new write verb where applicable.
- [NEVER] NEVER write a live-forge integration test for the open-PR / create-review ACCEPT path — prove the governed surface EXISTS + is authz-wired via a [build-gate] grep (T-LOOP-014). Runtime denial is covered by the denial ACs (no forge needed — the guard fires before the .await).
- [NEVER] NEVER record a review verdict via a direct DB write in a test asserting the bypass is blocked, NOR test that raw-git is blocked — both encode false guarantees (R6/R1 accepted-leak).
- [NEVER] NEVER use a _with_perm / repo-permission param on these async actions — but-api-macros:560 rejects it on ThreadSafeContext; the authorization is the orthogonal Authority axis as a pre-call guard, never the repo lock.
- [NEVER] NEVER branch on a role NAME (implementer/reviewer/maintainer) anywhere in the guard or verb dispatch — the separation comes from the functional Authority set alone (T-LOOP-005 grep-asserted).
- [NEVER] NEVER add new Code variants to but-error for these denials unless the desktop frontend consumes them — reuse the but-authz Denial perm.denied contract (the but-error enum doc forbids unused variants).
- [STRICTLY] STRICTLY surface the Denial as the structured {error:{code,message,remediation_hint}} exit-1 contract at the but pr/but review CLI boundary (the Subcommands::Pr arm in but/src/lib.rs), the SAME contract GATES-001 uses — message names the missing Authority, remediation_hint names the legitimate alternative.
- [STRICTLY] STRICTLY keep the existing publish_review / set_review_draftiness / update_review impls unchanged below the guard — add the guard at the TOP of the async fn, before the `let (storage, ...) = { ctx.into_thread_local(); ... }` block.
- [STRICTLY] STRICTLY make the reviewer-denied-commit-but-review-accepted pairing (T-LOOP-003 / T-AUTHZ-009) integration-testable: a principal with reviews:write and NO contents:write is denied a commit (the GATES-001 commit gate) but its `but review approve` is accepted and lands a verdict row — the two checks are independent functional gates.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: a principal lacking reviews:write denied perm.denied; unset/empty handle rejected; no verdict written
- [ ] AC-2: a reviewer (reviews:write, no contents:write) denied a commit but its review is accepted and recorded at head
- [ ] AC-3: the comment route gates on comments:write independently (ro denied; reviewer passes)
- [ ] AC-4: the governed pr/review verbs exist + are authz-wired with the correct Authority; no role name in the guard path (build-gate)
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: A principal lacking reviews:write is denied a review with perm.denied; an unset/empty handle is rejected [PRIMARY]
  GIVEN: fixtures `governed_repo_reviewer` (dev holds contents:write+pull_requests:write, NOT reviews:write) and `governed_repo_no_handle`
  WHEN:  `but review approve feat` with BUT_AGENT_HANDLE=dev, then unset, then ""
  THEN:  dev denied error.code=="perm.denied" naming reviews:write (exit 1, guard fires before any forge .await, no verdict row); unset/empty each rejected structured perm.denied (exit 1, no verdict)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api forge guard + real config + real but-db
  VERIFY: cargo test -p but review_guard_reviews_write_denied
  SCENARIO: NEGATIVE_CONTROL would fail if the guard omits reviews:write; an empty handle defaults to an allowed principal; the guard runs after the forge .await; the denial is not perm.denied.

AC-2: A reviewer (reviews:write, no contents:write) is denied a commit but its review is accepted and recorded at head
  GIVEN: fixture `governed_repo_reviewer` (reviewer holds reviews:write+comments:write, NOT contents:write)
  WHEN:  BUT_AGENT_HANDLE=reviewer commits to feat (the GATES-001 commit gate), then `but review approve feat`
  THEN:  commit denied perm.denied naming contents:write (feat unchanged); `but review approve` accepted, an approving local_review_verdicts row written (principal_id=reviewer, verdict="approved", head_oid==feat current head sha)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api commit gate + review guard + real but-db
  VERIFY: cargo test -p but review_guard_reviewer_commit_denied_review_accepted
  SCENARIO: NEGATIVE_CONTROL would fail if the reviewer's commit lands; the review is denied (coupled to contents:write); the verdict written at the wrong head; the approve path is a no-op stub writing no row.

AC-3: A ro principal is denied a comment (comments:write) — independent functional gate
  GIVEN: fixture `governed_repo_reviewer` (ro holds contents:read only; reviewer holds comments:write)
  WHEN:  `but review comment feat -m note` with BUT_AGENT_HANDLE=ro, then reviewer
  THEN:  ro comment denied error.code=="perm.denied" naming comments:write (exit 1, guard before the forge .await); reviewer comment passes the comments:write guard (independent of reviews:write)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api forge guard + real config
  VERIFY: cargo test -p but review_guard_comment_comments_write
  SCENARIO: NEGATIVE_CONTROL would fail if the ro comment passes (guard omitted as a no-op); the route gates on reviews:write; the reviewer comment is denied despite holding comments:write.

AC-4: The governed pr/review verbs exist at the but-api boundary, each authz-wired (build-gate)
  GIVEN: the source tree after this task
  WHEN:  the build-gate greps run over args/forge.rs, lib.rs, and but-api/src/legacy/forge.rs
  THEN:  the new subcommands are present + dispatched, each governed forge action carries an authorize() pre-call guard with the correct Authority, and no role name appears in the guard path
  TEST_TIER: unit (build-gate)   VERIFICATION_SERVICE: source grep (no runtime I/O)   UNIT_TEST_JUSTIFIED: structural/grep invariant; the open-PR live-forge accept path cannot be integration-tested without a real forge, so its presence is proven structurally and its runtime denial is covered by AC-1/AC-3
  VERIFY: grep -rEn 'Approve|RequestChanges|Comment|Close' crates/but/src/args/forge.rs && grep -rEn 'authorize\(.*(ReviewsWrite|CommentsWrite|PullRequestsWrite)' crates/but-api/src/legacy/forge.rs

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, error): lacking-reviews:write denied perm.denied; unset/empty handle rejected (T-AUTHZ-009/028)
    VERIFY: cargo test -p but review_guard_reviews_write_denied
- TC-2 (-> AC-2, happy_path): reviewer commit denied, review accepted + verdict at head (T-LOOP-003)
    VERIFY: cargo test -p but review_guard_reviewer_commit_denied_review_accepted
- TC-3 (-> AC-3, edge): comment gates on comments:write independently (T-AUTHZ-014)
    VERIFY: cargo test -p but review_guard_comment_comments_write
- TC-4 (-> AC-4, structural): governed pr/review surface exists + authz-wired, no role name (T-LOOP-014 build-gate, T-LOOP-005)
    VERIFY: grep -rEn 'Approve|RequestChanges|Comment|Close' crates/but/src/args/forge.rs

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides: pre-call authorize() guards on the governed forge actions (publish_review pull_requests:write; submit-review approve/request-changes reviews:write; comment comments:write; close pull_requests:write); the NEW governed CLI verbs under the existing but pr/but review heading; the LOCAL testable accept path (but review approve records a head-pinned local_review_verdicts row); the {error:{code,message,remediation_hint}} exit-1 contract at the CLI
consumes: but_authz::Authority::{PullRequestsWrite, ReviewsWrite, CommentsWrite} (AUTHZ-001); but_authz::authorize + resolve_principal from BUT_AGENT_HANDLE (AUTHZ-003); but_authz::config::load_governance_config (AUTHZ-002); but_db local_review_verdicts insert handle (GATES-002)
boundary_contracts:
  - CAP-AUTHZ-01: each governed forge action resolves BUT_AGENT_HANDLE→Principal and calls authorize(principal, <Authority>) as a PRE-CALL guard BEFORE the async forge .await (the async ThreadSafeContext shape forbids a repo-permission param — but-api-macros:560), failing closed perm.denied on unset/empty/unknown handle.
  - Honest testability: DENIAL paths are integration-testable against real but-api (the guard fires before any forge .await); the open-PR ACCEPT path is proven to EXIST + be authz-wired only via a [build-gate] grep (T-LOOP-014); `but review approve` accept is LOCAL (records into local_review_verdicts) and IS integration-tested against real but-db.
  - R6: reviews are recorded only through this governed action; the forgeable direct-DB-write path is out of scope and never under test.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-api/src/legacy/forge.rs (MODIFY) — pre-call authorize() guard at the top of publish_review + NEW guarded approve/request-changes/comment/close actions; for approve write the local_review_verdicts row at the current head
  - crates/but-api/Cargo.toml (MODIFY) — add but-authz workspace dep
  - crates/but-api/tests/forge_guard.rs (NEW) — denial-guard integration tests (no live forge)
  - crates/but/src/args/forge.rs (MODIFY) — add Close/Approve/RequestChanges/Comment to forge::pr::Subcommands
  - crates/but/src/lib.rs (MODIFY) — dispatch arms in Subcommands::Pr; surface the Denial as exit-1 structured contract
  - crates/but/src/command/legacy/forge/review.rs (MODIFY) — approve/request_changes/comment/close CLI helpers via ctx.to_sync()
  - crates/but/src/utils/metrics.rs (MODIFY) — extend the exhaustive Subcommands::Pr metrics match for the new variants
  - crates/but/tests/but/command/review_guard.rs (NEW) — CLI snapbox denial/accept + local_review_verdicts assertions
writeProhibited:
  - crates/but-authz/** — consume authorize/Authority/Denial; do NOT modify the primitive
  - crates/but-db/src/table/local_review_verdicts.rs — consume the GATES-002 handle; do NOT redefine the table
  - crates/but-api/src/legacy/forge.rs merge_review/set_review_auto_merge bodies — the MERGE gate is GATES-003
  - crates/but-api/src/commit/** — the commit gate is GATES-001 (used for the reviewer-commit-denied half of AC-2)
  - crates/but-error/src/lib.rs — reuse the but-authz Denial perm.denied contract
  - Any file not listed in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-api/src/legacy/forge.rs (263-313, 394-526)
   Focus: PRIMARY PATTERN + THE SEAM — the async forge actions on ThreadSafeContext: publish_review (396, open PR/create review → pull_requests:write), set_review_draftiness (500), update_review (532). Each begins `let (storage, ...) = { let ctx = ctx.into_thread_local(); ... }`. ADD the pre-call guard at the TOP of each governed async fn (publish_review, plus new close/approve/request-changes/comment): `let principal = principal_from_env(&ctx)?; authorize(&principal, Authority::X)?;` BEFORE the existing body. merge_review (438)/set_review_auto_merge (469) are GATES-003.
2. crates/but/src/args/forge.rs (1-91)
   Focus: THE CLI VERB SURFACE — forge::pr::Platform + forge::pr::Subcommands enum with existing New(15)/AutoMerge(45)/SetDraft(61)/SetReady(74)/Template(86). ADD Close/Approve/RequestChanges/Comment here (each with a branch/selector + optional -m message), under the SAME enum.
3. crates/but/src/lib.rs (1229-1335)
   Focus: THE DISPATCH — the Subcommands::Pr(forge::pr::Platform{cmd,..}) match with arms for New(1242)/Template/AutoMerge(1310)/SetDraft/SetReady. ADD match arms for the new variants, each calling the but_api::legacy::forge::* action and mapping a Denial to the structured exit-1 CLI contract.
4. crates/but/src/args/mod.rs (503-515)
   Focus: THE HEADING — Pr(forge::pr::Platform) with #[clap(visible_alias="review")] + "mr"; the new verbs ride under this same heading (no new top-level noun).
5. crates/but/src/command/legacy/forge/review.rs (20-91, 291-310)
   Focus: EXISTING CLI ACTION HELPERS — enable_auto_merge (20) + create_review (291) show how a CLI verb calls but_api::legacy::forge::* with ctx.to_sync() to pass a ThreadSafeContext to async actions (e.g. set_review_auto_merge(ctx.to_sync(), ...) at :83). Mirror this; add approve/request_changes/comment/close handlers; for approve, resolve the current head, call the guarded action, and write the local_review_verdicts row via GATES-002's handle.
6. crates/but-authz/src/authority.rs (10-110) + denial.rs (13-32)
   Focus: THE AUTHORITY CATALOG + DENIAL — Authority::{PullRequestsWrite, ReviewsWrite, CommentsWrite}; Denial{code,message,remediation_hint} with PERM_DENIED_CODE=="perm.denied".
7. crates/but/tests/but/utils.rs (121-142)
   Focus: THE CLI TEST HARNESS — Sandbox::but(args) returns a snapbox Command; add .env("BUT_AGENT_HANDLE","reviewer") + .assert() with .stdout_eq/.stderr_eq + [..]/... wildcards + exit-code checks. Seed via but_testsupport::writable_scenario + invoke_bash, read local_review_verdicts via the real but-db handle.
8. .spec/prds/governance/10-technical-requirements/04-api-design.md (30-67, 99-129)
   Focus: THE SEAM SHAPE + ROUTE TABLE — shape (b) async pre-call guard before .await; the route→Authority table; the New CLI verbs section directing to EXTEND the existing but pr/but review surface with the ref-pin caveat.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- Forge-guard denial integration tests pass (no live forge reached): `cargo test -p but-api forge_guard`  -> Exit 0; AC-1/AC-3 denial paths green
- CLI denial/accept + verdict-write tests pass: `cargo test -p but review_guard`  -> Exit 0; AC-1/AC-2/AC-3 green
- Governed verbs exist in args: `grep -rEn 'Approve|RequestChanges|Comment|Close' crates/but/src/args/forge.rs`  -> the four new subcommands present
- Governed verbs dispatched: `grep -rEn 'forge::pr::Subcommands::(Approve|RequestChanges|Comment|Close)' crates/but/src/lib.rs`  -> each dispatched in the Pr arm
- Forge actions authz-wired with correct Authority (T-LOOP-014): `grep -rEn 'authorize\(.*(ReviewsWrite|CommentsWrite|PullRequestsWrite)' crates/but-api/src/legacy/forge.rs`  -> the guards present
- No role name in the enforcement/dispatch path (T-LOOP-005): `! grep -rEin 'implementer|reviewer|maintainer' crates/but-api/src/legacy/forge.rs` (existing identifiers acceptable only if not role-branching; reviewer confirms)
- No _with_perm overload on the async forge guards: `! grep -rEn 'with_perm|write_permission|RepoExclusive' crates/but-api/src/legacy/forge.rs`  -> the guard is a pre-call authorize(), not the repo lock
- Crates compile incl. tests: `cargo check -p but-api -p but --all-targets`  -> Exit 0
- CLI-docs not drifted by the new subcommands: the new `Close`/`Approve`/`RequestChanges`/`Comment` verbs change the clap tree that `but-clap` walks to generate `cli-docs/` (04-api-design.md:101). If a committed `cli-docs/` snapshot or a doc-gen test exists, regen it (`cargo run -p but-clap` or the project's doc-gen target) and include the delta; otherwise confirm no `cli-docs` artifact is committed. Reviewer to verify no doc-gen test breaks.
- Clippy clean + fmt: `cargo clippy -p but-api -p but --all-targets && cargo fmt --check`  -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references: crates/but-api/src/legacy/forge.rs:396 (publish_review async seam to guard); crates/but/src/args/forge.rs:10 (forge::pr::Subcommands to extend) + crates/but/src/lib.rs:1229 (the Pr dispatch); crates/but/src/command/legacy/forge/review.rs:83 (ctx.to_sync() call shape); 04-api-design.md:48 (async pre-call guard shape) + :116 (extend the existing surface); crates/but-authz/src/denial.rs:13 (Denial)
notes:
  - Depends on AUTHZ-003 for principal_from_env/resolve_principal + authorize and AUTHZ-002 for load_governance_config at the target ref; depends on GATES-002 for the local_review_verdicts insert handle (the approve write target).
  - The async ThreadSafeContext shape (verified at but-api-macros:560) forbids a repo-permission param, so authorization is a pre-call authorize() guard before the .await — the same Denial contract as the commit gate, NOT _with_perm.
  - Honest test split: denial guards fire BEFORE the forge .await (integration-testable with NO live forge); `but review approve` accept is LOCAL (writes to but-db, integration-testable); the open-PR live-forge accept is proven only structurally (T-LOOP-014 build-gate).
pattern: Pre-call-authz-guard-on-async-forge-action — at the TOP of each `pub async fn action(ctx: ThreadSafeContext, ...)`, `let principal = principal_from_env(&ctx)?; authorize(&principal, Authority::X)?;` BEFORE the existing into_thread_local() body; the new governed CLI verbs ride the existing but pr/but review heading, call the guarded action via ctx.to_sync(), and (for approve) write a head-pinned local_review_verdicts row; a Denial surfaces as the structured exit-1 CLI contract.
pattern_source: crates/but-api/src/legacy/forge.rs:396 + crates/but/src/lib.rs:1229 + 04-api-design.md:48
anti_pattern: Using a _with_perm/repo-permission param on an async ThreadSafeContext action (rejected by but-api-macros:560); guarding AFTER the forge .await so a denied action still hits the forge or writes a verdict; adding parallel top-level create/close verbs that duplicate `but pr new`; writing a live-forge integration test for the open-PR accept path; branching on a role name; recording a verdict at the wrong (stale) head_oid.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — Wires the authz primitive into GitButler's REAL async forge actions as PRE-CALL authorize() guards before the .await, adds the missing governed CLI verbs under the existing but pr/but review heading, and records the `but review approve` verdict into GATES-002's local_review_verdicts. Owns the async pre-call guard pattern (NOT _with_perm), the clap subcommand additions, principal resolution from BUT_AGENT_HANDLE, and the honest integration-vs-build-gate split.
reviewer: rust-reviewer
coding_standards: crates/AGENTS.md, crates/WORKSPACE_MODEL.md, crates/but/AGENTS.md

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: AUTHZ-002, AUTHZ-003, GATES-002
Blocks:     LOOP-001
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "GATES-004",
  "proposed_by": "rust-planner",
  "verification_policy": { "requires_tests": true, "requires_red_evidence": true, "requires_seeded_evidence": true },
  "fixtures": {
    "governed_repo_reviewer": {
      "description": "A real git repo (but-testsupport writable_scenario) whose target ref main has committed .gitbutler/permissions.toml granting reviewer reviews:write+comments:write (NO contents:write), dev contents:write+pull_requests:write (no reviews:write), ro contents:read only; plus a feature branch feat with a head commit to review. Real but-db DbHandle attached for the local_review_verdicts write path.",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"governance-base\");",
        "invoke_bash: write .gitbutler/permissions.toml with [[principal]] id=\"reviewer\" permissions=[\"reviews:write\",\"comments:write\",\"contents:read\"]; [[principal]] id=\"dev\" permissions=[\"contents:write\",\"pull_requests:write\"]; [[principal]] id=\"ro\" permissions=[\"contents:read\"]",
        "invoke_bash: git add -A && git commit -m \"governance config\" (commits at refs/heads/main); git checkout -b feat; commit a code change so feat has a head to review",
        "drive via the Sandbox CLI harness (crates/but/tests/but/utils.rs:125 env.but(...)) with .env(\"BUT_AGENT_HANDLE\", ...)"
      ]
    },
    "governed_repo_no_handle": {
      "description": "Same repo as governed_repo_reviewer, used to drive an UNSET and an EMPTY BUT_AGENT_HANDLE against a governed review action to prove fail-closed rejection.",
      "seed_method": "cli",
      "records": ["reuse governed_repo_reviewer seeding; invoke `but review approve feat` once with BUT_AGENT_HANDLE unset and once with BUT_AGENT_HANDLE=\"\""]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN governed_repo_reviewer (dev lacks reviews:write) + governed_repo_no_handle WHEN `but review approve feat` as dev, then unset handle, then empty handle THEN dev denied perm.denied naming reviews:write, unset/empty each rejected structured perm.denied; all exit 1, no verdict written",
      "verify": "cargo test -p but review_guard_reviews_write_denied",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api forge guard + real config + real but-db",
        "negative_control": { "would_fail_if": [
          "the dev review lands because the guard omits the reviews:write check (a no-op guard)",
          "an unset/empty handle defaults to an allowed principal",
          "the guard runs AFTER the forge .await so a denial still hits the forge / writes a verdict",
          "the denial is not perm.denied / does not name reviews:write",
          "the no-handle case exits 1 only by panicking rather than returning the structured contract"
        ] },
        "evidence": { "artifact_type": "stdout", "required_capture": true },
        "cases": [
          {
            "start_ref": "governed_repo_reviewer",
            "action": { "actor": "cli_user", "steps": ["BUT_AGENT_HANDLE=dev: `but review approve feat` (dev lacks reviews:write)"] },
            "end_state": {
              "must_observe": ["`error.code == \"perm.denied\"`", "message names `\"reviews:write\"`", "process exits `1`", "local_review_verdicts for feat count == 0 (no row added)"],
              "must_not_observe": ["the review accepted", "exit `0`", "a verdict row written for dev"]
            }
          },
          {
            "start_ref": "governed_repo_no_handle",
            "action": { "actor": "cli_user", "steps": ["BUT_AGENT_HANDLE unset: `but review approve feat`", "BUT_AGENT_HANDLE=\"\": `but review approve feat`"] },
            "end_state": {
              "must_observe": ["each invocation: `error.code == \"perm.denied\"` (structured, not a bare panic)", "each exits `1` with no principal bound", "local_review_verdicts for feat count == 0"],
              "must_not_observe": ["a default/anonymous principal accepted", "exit `0`", "a verdict row written (none)"]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN governed_repo_reviewer (reviewer holds reviews:write not contents:write) WHEN reviewer commits to feat then runs `but review approve feat` THEN commit denied perm.denied naming contents:write (feat unchanged), review accepted and writes one approving verdict at feat's current head",
      "verify": "cargo test -p but review_guard_reviewer_commit_denied_review_accepted",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api + real git + real but-db",
        "negative_control": { "would_fail_if": [
          "the reviewer's commit lands (commit gate coupled to the review authority)",
          "the reviewer's review is denied (reviews:write coupled to contents:write)",
          "the approve verdict is written at the wrong head_oid",
          "the approve path is a no-op stub so no verdict row is written despite exit 0"
        ] },
        "evidence": { "artifact_type": "stdout", "required_capture": true },
        "cases": [
          {
            "start_ref": "governed_repo_reviewer",
            "action": { "actor": "cli_user", "steps": ["BUT_AGENT_HANDLE=reviewer: `but commit --branch feat -m x` (lacks contents:write)", "BUT_AGENT_HANDLE=reviewer: `but review approve feat`", "read local_review_verdicts for target feat via real but-db"] },
            "end_state": {
              "must_observe": ["commit: `error.code == \"perm.denied\"` naming `\"contents:write\"`, exit `1`, feat HEAD unchanged", "review: process exits `0` (accepted)", "exactly one local_review_verdicts row for feat: principal_id==\"reviewer\", verdict==\"approved\", head_oid == feat's current head sha"],
              "must_not_observe": ["the commit landed on feat", "the review denied", "the verdict head_oid != feat's current head sha", "0 verdict rows after an exit-0 approve"]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN governed_repo_reviewer WHEN `but review comment feat` as ro then as reviewer THEN ro denied perm.denied naming comments:write (exit 1, guard before .await), reviewer passes the comments:write guard",
      "verify": "cargo test -p but review_guard_comment_comments_write",
      "maps_to_ac": null,
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api forge guard + real config",
        "negative_control": { "would_fail_if": [
          "the ro comment passes because the comment route omits the comments:write guard (a no-op)",
          "the comment route gates on reviews:write instead of comments:write",
          "the reviewer comment is denied despite holding comments:write",
          "the denial does not name comments:write"
        ] },
        "evidence": { "artifact_type": "stdout", "required_capture": true },
        "cases": [
          {
            "start_ref": "governed_repo_reviewer",
            "action": { "actor": "cli_user", "steps": ["BUT_AGENT_HANDLE=ro: `but review comment feat -m note` (ro lacks comments:write)"] },
            "end_state": {
              "must_observe": ["`error.code == \"perm.denied\"`", "message names `\"comments:write\"`", "process exits `1`"],
              "must_not_observe": ["the comment accepted", "exit `0`", "the denial naming reviews:write instead of comments:write"]
            }
          },
          {
            "start_ref": "governed_repo_reviewer",
            "action": { "actor": "cli_user", "steps": ["BUT_AGENT_HANDLE=reviewer: `but review comment feat -m note` (holds comments:write) — assert the guard PASSES (authorize returns Ok; any post-guard forge failure is not the guard's denial)"] },
            "end_state": {
              "must_observe": ["the output does NOT contain `error.code == \"perm.denied\"` (the comments:write guard passed)"],
              "must_not_observe": ["a wrongful `error.code == \"perm.denied\"` naming comments:write for a holder (0 such denials expected)", "exit `1` from a wrongful denial"]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the source tree after this task WHEN the build-gate greps run THEN the new but pr/but review verbs exist + are dispatched, each governed forge action carries an authorize() pre-call guard with the correct Authority, and no role name appears in the guard path",
      "verify": "grep -rEn 'Approve|RequestChanges|Comment|Close' crates/but/src/args/forge.rs && grep -rEn 'authorize\\(.*(ReviewsWrite|CommentsWrite|PullRequestsWrite)' crates/but-api/src/legacy/forge.rs",
      "maps_to_ac": null,
      "scenario": {
        "tier": "holdout",
        "primary": false,
        "test_tier": "unit",
        "unit_test_justified": "build-gate structural/grep invariant with zero runtime I/O; the open-PR live-forge accept path cannot be integration-tested without a real forge, so its presence is proven structurally and its runtime denial is covered by AC-1/AC-3",
        "verification_service": "source grep (build-gate, no runtime I/O)",
        "negative_control": { "would_fail_if": [
          "the new verbs are absent from forge::pr::Subcommands (the surface was never added)",
          "the verbs exist in args but are not dispatched in the Subcommands::Pr arm (dead/no-op clap entries)",
          "a governed forge action lacks its authorize() pre-call guard (an ungoverned bypass, R14)",
          "the guard uses the wrong Authority (e.g. comment route guarded by reviews:write)",
          "a role name (implementer/reviewer/maintainer) appears in the guard/dispatch path (T-LOOP-005)"
        ] },
        "evidence": { "artifact_type": "stdout", "required_capture": true },
        "cases": [
          {
            "start_ref": "governed_repo_reviewer",
            "action": { "actor": "cli_user", "steps": [
              "grep -rEn 'Approve|RequestChanges|Comment|Close' crates/but/src/args/forge.rs",
              "grep -rEn 'forge::pr::Subcommands::(Approve|RequestChanges|Comment|Close)' crates/but/src/lib.rs",
              "grep -rEn 'authorize\\(.*(ReviewsWrite|CommentsWrite|PullRequestsWrite)' crates/but-api/src/legacy/forge.rs",
              "! grep -rEin 'implementer|reviewer|maintainer' the new guard/dispatch code paths"
            ] },
            "end_state": {
              "must_observe": ["Approve, RequestChanges, Comment, Close subcommands present in args/forge.rs (4 matches)", "each new subcommand dispatched in the `Subcommands::Pr` arm in lib.rs", "`authorize(...)` calls with `ReviewsWrite`, `CommentsWrite`, and `PullRequestsWrite` present (3 Authority guards) in but-api/src/legacy/forge.rs", "the role-name grep returns 0 matches in the guard/dispatch path"],
              "must_not_observe": ["0 matches for the new subcommands (surface missing/empty)", "a governed forge action with no authorize() guard", "a role-name string present in the enforcement path"]
            }
          }
        ]
      }
    },
    { "id": "TC-1", "type": "test_criterion", "description": "lacking-reviews:write denied perm.denied; unset/empty handle rejected (T-AUTHZ-009/028)", "verify": "cargo test -p but review_guard_reviews_write_denied", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "reviewer commit denied, review accepted + verdict at head (T-LOOP-003)", "verify": "cargo test -p but review_guard_reviewer_commit_denied_review_accepted", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "comment gates on comments:write independently (T-AUTHZ-014)", "verify": "cargo test -p but review_guard_comment_comments_write", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "governed pr/review surface exists + authz-wired, no role name (T-LOOP-014 build-gate, T-LOOP-005)", "verify": "grep -rEn 'Approve|RequestChanges|Comment|Close' crates/but/src/args/forge.rs", "maps_to_ac": "AC-4" }
  ]
}
-->
</details>
