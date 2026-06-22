# LPR-003: `request_review` (`PullRequestsWrite`) + `assign_reviewer` (`ReviewsWrite`, distinct-from-author) `#[but_api(napi)]` + the real `changes_requested` write (implement the `request_changes_review` STUB) + `but review request`/`assign`/`request-changes` CLI

> Status: ✅ Completed
> Commit: 5e219daa40
> Reviewer: deferred to PHASE 4.5 red-hat closeout — committed prior session; request/assign + changes_requested proofs AC-1..5
> Updated: 2026-06-22T18:07:12Z


## What this does

Add the two `#[but_api(napi)]` write verbs that open and assign a local review — `request_review` (opens the review for a branch, writes the first `local_review_assignments` row + the write-once `local_review_meta` opener row, `key="opener_principal"`) gated on **`PullRequestsWrite`** (the open-PR authority), and `assign_reviewer` (upserts a `pending` assignment for a reviewer principal) gated on **`ReviewsWrite`** (assignment is a review interaction, not opening the PR — tech-delta §B) and **enforcing `reviewer != author_principal_of_target_branch`** at the `but-api` boundary (the drive-layer mirror of the gate's `require_distinct_from_author`, R22) — both modeled exactly on the shipped `approve_review` (authorize-before-await, local-cache write, no DryRun guard). **AND implement the real `changes_requested` write inside `request_changes_review` — today a CONTRACT STUB (`forge.rs:551`) that authorizes `ReviewsWrite` then returns `task_contract_invalid` and writes nothing.** Plus the `but review request`/`assign`/`request-changes` CLI verbs routed through `review_gate_cli_error`. **No new `Authority` variant.**

## Why

Sprint 07 · PRD UC-LPR-01 · capability CAP-AUTHZ-01. UC-LPR-01 needs a reviewer to be **assignable** to a target (the drive signal an orchestrator reads), and the reviewer must be able to transition that assignment to `changes_requested`. Opening the review (`request_review`) is `PullRequestsWrite` (the open-PR authority, the same `publish_review` authorizes at `forge.rs:488`); the assignment (`assign_reviewer`) and the `changes_requested` transition are `ReviewsWrite` (the review-interaction authority `approve_review` uses, `forge.rs:526`) — tech-delta §B. `assign_reviewer` additionally enforces **distinct-from-author** so an implementer cannot self-assign as its own reviewer and have the drive narrative falsely read "independently reviewed" (R22). The opener principal is recorded once in the `local_review_meta(target, "opener_principal", id)` row (write-once, tech-delta §A.4), from which LPR-005 derives the agent-PR tag via the opener's declared `kind` in committed config. `request_changes_review` is currently a stub — LPR **implements** its real write, it does not reuse it.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-api request_review_persists_pending_assignment_without_touching_verdicts`: a `pull_requests:write`-holding caller's `but review request` writes a `pending` `local_review_assignments` row (target, reviewer, state=pending, assigned_at) + a write-once `local_review_meta(target, "opener_principal", caller)` row via the additive migrations, while NO `local_review_verdicts` row is written or changed. Full gate set in the spec below.

## Scope

- crates/but-api/src/legacy/forge.rs (MODIFY — add `request_review` (`PullRequestsWrite`; writes the first `local_review_assignments` row + the write-once `local_review_meta` opener row) / `assign_reviewer` (`ReviewsWrite`; enforces `reviewer != author_principal_of_target_branch` BEFORE the upsert — R22) `#[but_api(napi)]` fns beside `approve_review` (forge.rs:520); REPLACE the `request_changes_review` STUB body (forge.rs:551 — currently returns task_contract_invalid) with the real `local_review_assignments.state='changes_requested'` write on ReviewsWrite; reuse `authorize_branch_action` (forge.rs:47) + `branch_ref`)
- crates/but/src/command/legacy/forge/review.rs (MODIFY — add `request`/`assign` CLI verbs (→ request_review/assign_reviewer) beside approve/request_changes/comment/close (review.rs:20/:37/:55/:73); wire `request_changes` to the now-implemented request_changes_review; route errors through review_gate_cli_error (review.rs:89))
- crates/but/src/args/ (MODIFY — the verb/arg definitions for `but review request --reviewer <p>` / `but review assign <branch> --reviewer <p>`; NOT but-clap per tech-delta §B)
- crates/but-api/tests/local_review_assignments.rs (NEW — the PRIMARY but-api proofs AC-1..AC-5 against a real but-db + gix fixture via but_testsupport, hand-assertion style like merge_gate/governed_loop tests)
- packages/but-sdk/src/generated/\*\* (REGENERATE ONLY via `pnpm build:sdk && pnpm format` — NEVER hand-edit; the actual regen + N-API audit is LPR-010's gate)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: LPR-003 — request_review/assign_reviewer + the real changes_requested write + CLI
================================================================================

TASK_TYPE:   FEATURE
STATUS:      Backlog
PRIORITY:    P0
AGENT:       implementer=rust-implementer | reviewer=rust-reviewer
EFFORT:      L  (180 min)
PROPOSED-BY: rust-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-LPR-01
CAPABILITIES:CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-api request_review_persists_pending_assignment_without_touching_verdicts
  check: cargo check -p but-api --all-targets
  lint:  cargo clippy -p but-api --all-targets

--------------------------------------------------------------------------------
TYPE DESIGN (Rust shapes this task introduces)
--------------------------------------------------------------------------------
API SURFACE (additive #[but_api(napi)] fns, modeled on approve_review forge.rs:520):
  - `async fn request_review(ctx: ThreadSafeContext, branch: String, reviewer: Option<String>) -> Result<()>` — gated PullRequestsWrite; writes the write-once local_review_meta opener row, AND — when `reviewer` is Some — also writes the first `pending` local_review_assignments row for that reviewer (distinct-from-author still enforced). NOTE: tech-delta §B's verb table lists the canonical open shape as `request_review(ctx, branch)`; LPR RATIFIES the combined open(+optional-assign) shape so a single `but review request <branch> --reviewer <p>` both opens the review and seeds the first assignment (the ROADMAP/SPRINT human-gate step 1 form). When `reviewer` is None the verb opens the review (opener row only) and a later `assign_reviewer` seeds the assignment.
  - `async fn assign_reviewer(ctx: ThreadSafeContext, branch: String, reviewer: String) -> Result<()>` — gated ReviewsWrite; enforces reviewer != author_principal_of_target_branch BEFORE the upsert (R22)
  - `async fn request_changes_review(ctx: ThreadSafeContext, branch: String, message: Option<String>) -> Result<()>` (REPLACE the stub body — keep the EXISTING signature at forge.rs:551; ReviewsWrite)
ERROR STRATEGY:
  - anyhow::Result at the but-api boundary (the shipped convention). authorize_branch_action(...)? propagates the structured perm.denied Denial via `?`; .context("…") explains the operation (cf. approve_review forge.rs:526 `.context(...)`). The CLI maps the error through review_gate_cli_error (review.rs:89) to the structured {code, message, remediation_hint} + exit 1.
OWNERSHIP PLAN:
  - `let ctx = ctx.into_thread_local();` then `let repo = ctx.repo.get()?;` (exactly approve_review forge.rs:523-525). authorize_branch_action borrows &repo + &branch and returns the resolved Principal (owned). The assignment row is built and MOVED into `local_review_assignments_mut().upsert(row)`. `principal.id().as_str().to_owned()` for the opener/caller principal string (cf. approve_review forge.rs:535). For `assign_reviewer`'s distinct-from-author check, resolve the target branch's author principal (the author of the branch's tip / the opener recorded in the local_review_meta opener row) and compare it `!=` the `reviewer` arg BEFORE the upsert. request_review MOVES a LocalReviewMeta{target, key:"opener_principal", value: opener.id(), created_at} into `local_review_meta_mut().upsert_if_absent(row)` (write-once).
DOC POINTERS (read before coding):
  - brain/docs/rust/error-handling.md → Result + ? + anyhow::Context; the structured Denial propagation
  - brain/docs/rust/concurrency.md → async fn + into_thread_local() (the ThreadSafeContext -> thread-local repo handle pattern)
  - brain/docs/rust/ownership-borrowing.md → borrow &repo for authorize, move the row into upsert
  - brain/docs/rust/testing.md → real but-db + gix fixture via but_testsupport; #[serial_test::serial] + temp_env BUT_AGENT_HANDLE

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Proven against real but-db + real gix via but_testsupport (hand-assertion style, like merge_gate/governed_loop): (1) request_review as a pull_requests:write caller writes a `pending` local_review_assignments row (target, reviewer, state="pending", assigned_at) AND a write-once local_review_meta(target, "opener_principal", caller) row, while NO local_review_verdicts row is written/changed; (2) assign_reviewer (on ReviewsWrite) upserts a `pending` assignment for a reviewer DISTINCT from the target branch's author principal, idempotent per (target, reviewer_principal) — AND a SELF-ASSIGNMENT (reviewer == author) is REJECTED/flagged (R22), no row written; (3) request_changes_review — the IMPLEMENTED write — sets the caller's local_review_assignments.state to "changes_requested" on ReviewsWrite (NO LONGER returning task_contract_invalid), and the merge-gate decision (verdict-at-head) is asserted UNCHANGED by the state flip alone; (4) request_review from a principal lacking PullRequestsWrite (and assign_reviewer/request_changes_review lacking ReviewsWrite) are denied perm.denied + exit 1 with NO row written; (5) the writes touch ONLY the local cache (no ref/object/oplog mutation; no DryRun guard, matching approve_review); cargo test -p but-api green; clippy clean.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST model request_review/assign_reviewer EXACTLY on the shipped approve_review (forge.rs:520-546): `#[but_api(napi)] #[instrument(err(Debug))] pub async fn …(ctx: ThreadSafeContext, branch: String, …) -> Result<()>`, then `let ctx = ctx.into_thread_local(); let repo = ctx.repo.get()?; let principal = authorize_branch_action(&repo, &branch, Authority::PullRequestsWrite)?.context(...)?;` BEFORE any await, then a local-cache write. Read forge.rs:520-546 and mirror it line-for-line (only the authority + the write target differ).
- [MUST] MUST gate request_review on `Authority::PullRequestsWrite` (forge.rs:62 — the open-PR authority publish_review authorizes at forge.rs:488), and BOTH assign_reviewer AND request_changes_review on `Authority::ReviewsWrite` (the review-interaction authority approve_review authorizes at forge.rs:526 / the STUB already authorizes at forge.rs:558 — keep it). Assignment is a *review interaction*, NOT opening the PR — tech-delta §B. NO new Authority variant. The route->Authority table stays closed (authority.rs:11 unchanged).
- [MUST] MUST enforce DISTINCT-FROM-AUTHOR in assign_reviewer BEFORE the upsert (R22, the drive-layer mirror of the gate's require_distinct_from_author, review_requirement.rs). Resolve the target branch's author principal (the opener recorded in the local_review_meta opener row for the target, or the author of the branch tip) and REJECT/flag the assignment when `reviewer == author_principal_of_target_branch` — return an Err (a structured denial, not a panic) and write NO assignment row, so the drive narrative cannot read "independently reviewed" for a self-assigned reviewer. AC-2b's self-assignment-rejected negative control is the behavioral proof. This is NEW LPR drive-layer-integrity work at the but-api boundary (it does NOT touch the merge gate's own distinct-from-author copy in review_requirement.rs).
- [MUST] MUST IMPLEMENT the real changes_requested write inside request_changes_review. TODAY (forge.rs:551-565) it authorizes ReviewsWrite then `Err(task_contract_invalid("request_changes_review", …))` and writes NOTHING. REPLACE the `Err(task_contract_invalid(...))` body with: resolve the caller principal from authorize_branch_action, then `db.local_review_assignments_mut().set_state(&branch, principal.id().as_str(), "changes_requested")` (or upsert with state=changes_requested if no assignment exists for the caller). This is NEW LPR work — do NOT describe it as reuse. AC-3 asserts the row state flips and the stub error is gone.
- [MUST] MUST authorize-BEFORE-await and before any write (RULES.md "authorize before the guard"): authorize_branch_action(...)? is the pre-call guard; no `.await` and no `local_review_assignments_mut().upsert/set_state` may run on the denial path. AC-4's no-row-written-on-denial is the behavioral proof.
- [MUST] MUST record request_review's opener in the dedicated `local_review_meta` table — NOT a comment-body sentinel. Write `local_review_meta_mut().upsert_if_absent(LocalReviewMeta{ target, key: "opener_principal", value: <opener principal id>, created_at })` (the LPR-001 write-once `INSERT … ON CONFLICT(target,key) DO NOTHING`) so LPR-005's derivation can compute the agent-PR tag from the opener's DECLARED `kind` in committed config (tech-delta §A.4). The opener is recorded ONCE per target (a later caller cannot overwrite it — the R23 control-path narrowing) and is the resolved/authorized caller principal, never a caller-supplied flag. **Do NOT** store the opener as a `__pr_meta__` comment-body row: a comment body is attacker-influenceable free text (R20), so making it the tag-derivation control-plane input would let any comment-write actor forge the opener (rejected, tech-delta §A.4). (request_review writes BOTH the first assignment row AND this local_review_meta opener row.)
- [MUST] MUST omit the DryRun guard (matching approve_review forge.rs:520-546): these writes touch ONLY ctx.db.get_cache_mut() (SQLite), not refs/objects/oplog, so RULES.md "dry runs must not persist refs, objects, or oplog" does not bind them. Do NOT add a DryRun check. AC-5 proves no ref/object/oplog mutation occurs.
- [MUST] MUST map AssignmentState <-> the TEXT column via LPR-002's AssignmentState::name()/parse() at this boundary — write `AssignmentState::Pending.name()` for the pending state, `AssignmentState::ChangesRequested.name()` for the changes_requested write. Do NOT hardcode the literal a second time (use the typed round-trip so a literal typo can't drift).
- [MUST] MUST route the CLI verbs through the existing review_gate_cli_error serializer (review.rs:89) so a denial prints `{code:"perm.denied", message, remediation_hint}` to stderr + exit 1 — identical to the shipped approve/request_changes CLI error path.
- [NEVER] NEVER write, read, or touch local_review_verdicts in any of these three verbs — assignments and verdicts are SEPARATE. request_review/assign_reviewer/request_changes_review write ONLY local_review_assignments (+ the write-once `local_review_meta` opener row from request_review, key="opener_principal" — sourced from a table, NOT a comment body, per R23). AC-1's "no verdict row written/changed" catches a cross-write.
- [NEVER] NEVER add a new Authority variant or branch on a role name / human-vs-AI predicate (the invariant_build_gates honesty grep over forge.rs must stay green — forge.rs IS an ENFORCEMENT_PATH, invariant_build_gates.rs:23).
- [NEVER] NEVER leave request_changes_review returning task_contract_invalid — that is the stub; implementing its real write is the whole point of this task (AC-3 fails if the stub error survives).
- [NEVER] NEVER commit/stage/move a ref or write outside the local cache from any verb (breaks the local-cache-only contract; AC-5 catches a ref/object/oplog mutation).
- [NEVER] NEVER hand-edit packages/but-sdk/src/generated — the regen is LPR-010's gate (run pnpm build:sdk there).
- [NEVER] NEVER add new gitbutler-* usage.
- [STRICTLY] STRICTLY treat approve_review (forge.rs:520) and authorize_branch_action (forge.rs:47) as CONSUMED seams — mirror approve_review's shape and compose authorize_branch_action; do not fork a parallel authorize call or a parallel cache-write helper.
- [STRICTLY] STRICTLY keep the (ctx, branch, reviewer)/(ctx, branch, message) signatures so the CLI verbs and the N-API binding pass the same branch the workspace resolves.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [x] AC-1 [PRIMARY]: request_review (PullRequestsWrite) writes a `pending` assignment + a write-once `local_review_meta(target, "opener_principal", caller)` row, touching NO local_review_verdicts row
- [x] AC-2: assign_reviewer (ReviewsWrite) upserts a `pending` assignment for a reviewer DISTINCT from the target author, idempotent per (target, reviewer_principal); a SELF-ASSIGNMENT (reviewer == author) is REJECTED/flagged with NO row written (R22)
- [x] AC-3: request_changes_review — IMPLEMENTED — sets the caller's assignment state to changes_requested on ReviewsWrite (the stub task_contract_invalid is GONE); the merge-gate verdict-at-head decision is UNCHANGED by the flip
- [x] AC-4: request_review without PullRequestsWrite (and assign_reviewer/request_changes_review without ReviewsWrite) are denied perm.denied + exit 1 with NO row written
- [x] AC-5: the writes are local-cache only — no ref/object/oplog mutation, no DryRun guard (matching approve_review)
- [x] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY] (T-LPR-001): request_review persists a pending assignment + opener marker, no verdict touched
  GIVEN: lpr_governed_repo: a real governed repo (committed .gitbutler/permissions.toml grants caller `rev` pull_requests:write + reviews:write) + real gix; BUT_AGENT_HANDLE=rev under #[serial_test::serial]; the local_review_verdicts store captured (empty) before the call
  WHEN:  `but review request refs/heads/feat --reviewer rev2` runs (request_review)
  THEN:  a local_review_assignments row exists (target=refs/heads/feat, reviewer_principal=rev2, state="pending", assigned_at set); a local_review_meta row exists for (refs/heads/feat, "opener_principal") whose value is the opener principal (rev); AND the local_review_verdicts store is UNCHANGED (no row written/changed) — assignments and verdicts are separate
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api request_review + real but-db local_review_assignments/comments + real gix via but_testsupport::writable_scenario
  VERIFY: cargo test -p but-api request_review_persists_pending_assignment_without_touching_verdicts

AC-2 (T-LPR-002/043): assign_reviewer (ReviewsWrite) upserts a pending DISTINCT-from-author assignment, idempotent; a self-assignment is rejected (R22)
  GIVEN: lpr_governed_repo with caller `rev` holding reviews:write; the target branch's author principal is `auth`; an existing `pending` assignment for rev2 on the branch
  WHEN:  `but review assign refs/heads/feat --reviewer rev2` runs twice (assign_reviewer), then a distinct `--reviewer rev3` assign, then `--reviewer auth` (a SELF-ASSIGNMENT: reviewer == the target author)
  THEN:  exactly ONE local_review_assignments row exists for (refs/heads/feat, rev2) with state="pending" (the second assign UPDATED, did not duplicate — the idempotent upsert from LPR-001); the rev3 assign adds a SECOND distinct row; AND the `auth` self-assignment is REJECTED/flagged (Err, structured denial) with NO row written for (refs/heads/feat, auth) — the drive layer cannot narrate "independently reviewed" for a self-assigned reviewer (R22, the drive-layer mirror of require_distinct_from_author)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api assign_reviewer (ReviewsWrite + distinct-from-author at the but-api boundary) + real but-db assignment upsert + real gix
  VERIFY: cargo test -p but-api assign_reviewer_distinct_from_author_upserts_idempotent

AC-3 (T-LPR-005): request_changes_review IMPLEMENTED — flips state to changes_requested on ReviewsWrite; verdict-at-head decision unchanged
  GIVEN: lpr_governed_repo with caller `rev` holding reviews:write; an existing `pending` assignment for rev on the branch; the merge-gate decision for the branch captured before the flip (e.g. blocked: no approval@head)
  WHEN:  `but review request-changes refs/heads/feat --message "needs work"` runs (request_changes_review, IMPLEMENTED)
  THEN:  the call returns Ok (NOT task_contract_invalid — the stub is gone); the local_review_assignments.state for rev == "changes_requested"; AND the merge-gate decision for the branch is IDENTICAL to before the flip (enforce_merge_gate reads only local_review_verdicts at head — the assignment-state flip never reaches the gate)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api request_changes_review write + real but-db assignment set_state + real gix; the merge-gate decision read before/after via enforce_merge_gate (or its CLI surface)
  VERIFY: cargo test -p but-api request_changes_review_implements_changes_requested_write

AC-4 (T-LPR-006): missing-authority denial — perm.denied + exit 1, no row written
  GIVEN: lpr_governed_repo with caller `impl` holding contents:write ONLY (NO pull_requests:write, NO reviews:write); the assignment store captured before
  WHEN:  `but review request refs/heads/feat --reviewer rev2` runs as `impl` (and separately `but review request-changes refs/heads/feat` as `impl`)
  THEN:  each call exits 1 with stderr JSON {code:"perm.denied", message, remediation_hint} naming the required authority (request_review → pull_requests:write; assign_reviewer / request_changes_review → reviews:write); AND NO local_review_assignments row is written (the authorize-before-write guard ran first)
  TEST_TIER: api-contract   VERIFICATION_SERVICE: real but-api request_review/request_changes_review composing authorize_branch_action + real but-authz + real gix; CLI stderr+exit captured
  VERIFY: cargo test -p but-api request_review_denied_without_authority_writes_nothing

AC-5 (T-LPR-007): local-cache only — no ref/object/oplog mutation, no DryRun guard
  GIVEN: lpr_governed_repo with caller `rev` holding pull_requests:write; refs/objects/oplog snapshotted before
  WHEN:  `but review request refs/heads/feat --reviewer rev2` runs (incl. under --dry-run if the CLI exposes it)
  THEN:  the assignment row IS written (local cache, like approve_review) AND no ref/object/oplog mutation occurs (the snapshots are byte-identical before/after); FAIL if a DryRun guard suppresses the local write or if any ref/object/oplog changes
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api request_review + real gix ref/object/oplog snapshot before/after via but_testsupport
  VERIFY: cargo test -p but-api request_review_is_local_cache_only_no_ref_mutation

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): request_review (PullRequestsWrite) writes a pending assignment (target, reviewer, state=pending, assigned_at) + a write-once local_review_meta(target, "opener_principal", caller) row
    VERIFY: cargo test -p but-api request_review_persists_pending_assignment_without_touching_verdicts
- TC-2 (-> AC-1): after request_review the local_review_verdicts store is unchanged (no verdict row written/changed)
    VERIFY: cargo test -p but-api request_review_persists_pending_assignment_without_touching_verdicts
- TC-3 (-> AC-2): two assign_reviewer calls for the same (target, rev2) leave exactly 1 row (idempotent upsert); a different reviewer adds a second row; a self-assignment (reviewer == target author) is REJECTED with no row written (R22)
    VERIFY: cargo test -p but-api assign_reviewer_distinct_from_author_upserts_idempotent
- TC-4 (-> AC-3): request_changes_review returns Ok (not task_contract_invalid) and sets the caller's assignment state to changes_requested
    VERIFY: cargo test -p but-api request_changes_review_implements_changes_requested_write
- TC-5 (-> AC-3): the merge-gate decision for the branch is identical before and after the changes_requested flip (assignment state never gates)
    VERIFY: cargo test -p but-api request_changes_review_implements_changes_requested_write
- TC-6 (-> AC-4): request_review as a contents:write-only caller exits 1 with perm.denied naming pull_requests:write (and assign_reviewer/request_changes_review name reviews:write) and writes no assignment row
    VERIFY: cargo test -p but-api request_review_denied_without_authority_writes_nothing
- TC-7 (-> AC-5): under request_review the assignment row is written but refs/objects/oplog are byte-unchanged (local-cache only, no DryRun guard)
    VERIFY: cargo test -p but-api request_review_is_local_cache_only_no_ref_mutation

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides:
  - request_review(ctx, branch, reviewer: Option<String>) #[but_api(napi)] — opens a local review (the write-once local_review_meta opener row + the first pending assignment when reviewer is Some — the combined open(+optional-assign) shape LPR ratifies over tech-delta §B's `request_review(ctx, branch)`) on PullRequestsWrite
  - assign_reviewer(ctx, branch, reviewer) #[but_api(napi)] — idempotent pending-assignment upsert on ReviewsWrite, enforcing reviewer != author_principal_of_target_branch (R22)
  - request_changes_review(ctx, branch, message) — the IMPLEMENTED changes_requested write (set_state) on ReviewsWrite, REPLACING the forge.rs:551 stub
  - `but review request`/`assign`/`request-changes` CLI verbs routed through review_gate_cli_error
consumes:
  - crate::legacy::forge::{approve_review (the structural template), authorize_branch_action (forge.rs:47), branch_ref} (COMPOSED — mirror/reuse, never fork)
  - but_db::{LocalReviewAssignment, LocalReviewMeta} + the Handle pairs (LPR-001) — including local_review_meta's write-once upsert_if_absent for the opener row
  - but_authz::{AssignmentState (LPR-002), Authority::{PullRequestsWrite, ReviewsWrite}} (REUSED — no new variant)
boundary_contracts:
  - CAP-AUTHZ-01: request_review authorizes PullRequestsWrite; assign_reviewer + request_changes_review authorize ReviewsWrite — all via authorize_branch_action(&repo, &branch, Authority::X)? BEFORE any await/write; a caller lacking the authority is denied perm.denied + exit 1 with NO row written; no new Authority variant. assign_reviewer additionally enforces reviewer != author_principal_of_target_branch at the but-api boundary (R22) before the upsert. The opener is recorded once in the write-once local_review_meta opener row, never a __pr_meta__ comment-body sentinel. The writes are local-cache only (no DryRun guard, matching approve_review) and never touch local_review_verdicts or any ref/object/oplog. request_changes_review's real write is NEW LPR work, not a reuse of the (stubbed) original.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-api/src/legacy/forge.rs (MODIFY — add request_review/assign_reviewer beside approve_review; REPLACE the request_changes_review stub body with the real changes_requested write)
  - crates/but/src/command/legacy/forge/review.rs (MODIFY — add request/assign CLI verbs; wire request_changes to the implemented fn; route via review_gate_cli_error)
  - crates/but/src/args/ (MODIFY — the request/assign verb+arg definitions; NOT but-clap)
  - crates/but-api/tests/local_review_assignments.rs (NEW — the PRIMARY but-api proofs AC-1..AC-5)
  - crates/but/tests/ (MODIFY/NEW — a happy-path CLI test for `but review request`/`assign` if the CLI test harness requires it; happy-path only per RULES.md — but the full CLI happy-path suite is LPR-010)
  - packages/but-sdk/src/generated/** (REGENERATE ONLY — NEVER hand-edit; the regen gate is LPR-010)
writeProhibited:
  - crates/but-api/src/legacy/merge_gate.rs, review_requirement.rs — CONSUME-only (the safe seam); do NOT add any read of local_review_assignments to the gate path (LPR-009 greps this)
  - crates/but-db/** — CONSUME the LPR-001 tables/Handles; do NOT change the schema here
  - crates/but-authz/src/authority.rs — no new Authority variant
  - crates/but-api/src/legacy/forge.rs approve_review/merge_review/publish_review — CONSUME approve_review's shape + authorize_branch_action; do NOT change the shipped verbs (only ADD the new fns + REPLACE the request_changes_review stub body)
  - any gitbutler-* crate (no new gitbutler-* usage)
  - Any file not in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-api/src/legacy/forge.rs [520-546] — [PRIMARY PATTERN — mirror line-for-line] the shipped approve_review: `#[but_api(napi)] #[instrument(err(Debug))] pub async fn approve_review(ctx: ThreadSafeContext, branch: String) -> Result<()>`, `let ctx = ctx.into_thread_local(); let repo = ctx.repo.get()?; let principal = authorize_branch_action(&repo, &branch, Authority::ReviewsWrite)?.context(...)?;`, then `ctx.db.get_cache_mut()?.local_review_verdicts_mut().insert(...)`. Your verbs write local_review_assignments (+ the local_review_meta opener row from request_review) instead, gate on PullRequestsWrite (request) / ReviewsWrite (assign + changes), and omit the DryRun guard for the same reason.
2. crates/but-api/src/legacy/forge.rs [549-566] — [THE STUB YOU IMPLEMENT] request_changes_review: it authorizes ReviewsWrite then `Err(task_contract_invalid("request_changes_review", …))` and writes NOTHING. REPLACE the Err(...) body with the real set_state(branch, principal_id, "changes_requested") write. Keep the signature + the ReviewsWrite authorize.
3. crates/but-api/src/legacy/forge.rs [47-65] — authorize_branch_action(&repo, &branch, Authority) — the COMPOSED guard that resolves the principal from BUT_AGENT_HANDLE, reads the target-ref config, authorizes, and returns the Principal; the Authority constants (PullRequestsWrite forge.rs:62, ReviewsWrite, CommentsWrite forge.rs:61). Use this; do not fork an authorize call.
4. crates/but/src/command/legacy/forge/review.rs [20-95] — the shipped approve/request_changes/comment/close CLI verbs + review_gate_cli_error (review.rs:89). Mirror the verb shape for request/assign; wire request_changes to the implemented fn; route all errors through review_gate_cli_error.
5. crates/but-db/src/table/local_review_assignments.rs (LPR-001) — the LocalReviewAssignment struct + upsert/set_state methods you write through.
6. crates/but-authz/src/assignment_state.rs (LPR-002) — AssignmentState::name()/parse() — use these to map the state literal, never a hardcoded second copy.
7. crates/but-api/tests/ (the merge_gate / governed_loop hand-assertion tests) — [VERIFIED TEST IDIOM] the real-but-db + gix + #[serial_test::serial] + temp_env BUT_AGENT_HANDLE construction these tests use (NOT insta snapshots). Mirror it for local_review_assignments.rs. Seed the governed repo via but_testsupport::writable_scenario + invoke_bash committing .gitbutler/permissions.toml.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- cargo test -p but-api request_review_persists_pending_assignment_without_touching_verdicts   -> Exit 0; pending assignment + write-once local_review_meta opener row written; verdict store unchanged
- cargo test -p but-api assign_reviewer_distinct_from_author_upserts_idempotent   -> Exit 0; one row per (target,reviewer); second assign updates not duplicates; self-assignment (reviewer==author) rejected, no row
- cargo test -p but-api request_changes_review_implements_changes_requested_write   -> Exit 0; Ok (not task_contract_invalid); state flips; merge decision unchanged
- cargo test -p but-api request_review_denied_without_authority_writes_nothing   -> Exit 0; perm.denied naming pull_requests:write; no row written
- cargo test -p but-api request_review_is_local_cache_only_no_ref_mutation   -> Exit 0; row written; refs/objects/oplog byte-unchanged
- cargo check -p but-api --all-targets   -> Exit 0
- cargo clippy -p but-api --all-targets   -> Exit 0
- cargo test -p but-authz invariant_build_gates   -> Exit 0; forge.rs honesty grep green (no role-name/human-vs-AI branch added)
- cargo fmt --check   -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references:
  - crates/but-api/src/legacy/forge.rs:520 (approve_review — the template), :551 (request_changes_review — the stub to implement), :47 (authorize_branch_action)
  - .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md §B (the verb->Authority table; "request_changes_review is currently a CONTRACT STUB … LPR must implement the real changes-requested write … that is NEW LPR work, not reuse")
  - .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/04-e2e-testing-criteria.md (T-LPR-001/002/005/006/007 — the criteria these ACs realize)
code_skeleton: |
  // request_review — mirror approve_review's frame
  #[but_api(napi)]
  #[instrument(err(Debug))]
  pub async fn request_review(ctx: ThreadSafeContext, branch: String, reviewer: Option<String>) -> Result<()> {
      let ctx = ctx.into_thread_local();
      let repo = ctx.repo.get()?;
      let opener = authorize_branch_action(&repo, &branch, Authority::PullRequestsWrite)?
          .context("governance config is required to open a local review")?;
      let mut db = ctx.db.get_cache_mut()?;
      db.local_review_assignments_mut().upsert(but_db::LocalReviewAssignment {
          id: uuid::Uuid::new_v4().to_string(),
          target: branch.clone(),
          reviewer_principal: reviewer,
          state: AssignmentState::Pending.name().to_owned(),
          assigned_at: chrono::Utc::now().naive_utc(),
      })?;
      // record the opener ONCE in the dedicated local_review_meta table (NOT a comment-body
      // sentinel — R23 != R20). LPR-005 derives the agent-PR tag from this opener's DECLARED
      // `kind` in committed config. Write-once: ON CONFLICT(target,key) DO NOTHING.
      db.local_review_meta_mut().upsert_if_absent(but_db::LocalReviewMeta {
          target: branch,
          key: "opener_principal".to_owned(),
          value: opener.id().as_str().to_owned(),
          created_at: chrono::Utc::now().naive_utc(),
      })?;
      Ok(())
  }
  // assign_reviewer — gate ReviewsWrite, then enforce distinct-from-author BEFORE the upsert (R22):
  //   let _ = authorize_branch_action(&repo, &branch, Authority::ReviewsWrite)?;
  //   let author = author_principal_of_target_branch(&repo, &branch, &db)?;  // opener / branch-tip author
  //   anyhow::ensure!(reviewer != author, "a reviewer must be distinct from the target branch author (R22)");
  //   db.local_review_assignments_mut().upsert(... reviewer_principal: reviewer, state: Pending ...)?;
  // request_changes_review — REPLACE the stub Err(task_contract_invalid(...)) with:
  //   let principal = authorize_branch_action(&repo, &branch, Authority::ReviewsWrite)?;
  //   ctx.db.get_cache_mut()?.local_review_assignments_mut()
  //       .set_state(&branch, principal.id().as_str(), AssignmentState::ChangesRequested.name())?;
  //   Ok(())
notes:
  - The opener is recorded in `local_review_meta(target, "opener_principal", <id>)` via the LPR-001 write-once `upsert_if_absent` (ON CONFLICT(target,key) DO NOTHING) — NOT a `__pr_meta__` comment-body sentinel (R23 != R20). A comment body is attacker-influenceable free text (R20), so it must not be the tag-derivation control input; the dedicated meta row is. LPR-004/LPR-010 still keep `__pr_meta__` a CLOSED reserved thread_id that post_comment rejects — that is now the R23 NEGATIVE CONTROL (a comment-body sentinel cannot forge the opener/tag), not the opener storage.
  - The merge-decision-unchanged assertion (AC-3) reads enforce_merge_gate (or its CLI surface) before and after the changes_requested flip; the decision is identical because the gate reads only local_review_verdicts at head (safe seam).
  - CLI: `but review request <branch> --reviewer <p>` and `but review assign <branch> --reviewer <p>`; `but review request-changes <branch> [--message <m>]` now SUCCEEDS (the stub is gone). Each prints the ref-pin caveat only where it writes config-visible state; assignments are local cache (no caveat needed) — match approve's CLI posture.
pattern: additive #[but_api(napi)] write verbs modeled on approve_review (authorize-before-await, local-cache write, no DryRun guard) reusing PullRequestsWrite (request) / ReviewsWrite (assign + changes); assign_reviewer enforces distinct-from-author at the but-api boundary (R22); request_review records the opener in the write-once local_review_meta row; plus replacing the request_changes_review stub body with the real set_state write; plus mirror CLI verbs through review_gate_cli_error
pattern_source: crates/but-api/src/legacy/forge.rs:520 (approve_review), :551 (the stub), :47 (authorize_branch_action); crates/but/src/command/legacy/forge/review.rs:20-95 (the CLI verb shape + review_gate_cli_error)
anti_pattern: leaving request_changes_review returning task_contract_invalid (the stub survives — AC-3 fails); a new Authority variant; gating assign_reviewer on PullRequestsWrite instead of ReviewsWrite (tech-delta §B); skipping the distinct-from-author check so a self-assignment succeeds (AC-2 fails — R22); authorizing AFTER the write (a row written on a denied call — AC-4 fails); adding a DryRun guard (suppresses the local write — AC-5 fails); writing local_review_verdicts from an assignment verb (AC-1 catches the cross-write); recording the opener as a __pr_meta__ comment-body sentinel instead of the dedicated write-once local_review_meta row (R23 != R20); a caller-supplied agent/kind flag instead of the derived opener + declared kind

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agent: implementer=rust-implementer | reviewer=rust-reviewer
rationale: Additive but-api write verbs modeled on approve_review, PLUS implementing a shipped CONTRACT STUB (request_changes_review) into a real set_state write — the highest-risk-of-fakeability task in the sprint (a stub that returns Ok without writing would pass a weak test). Requires authorize-before-write ordering, the correct authority split (request=PullRequestsWrite, assign+changes=ReviewsWrite, tech-delta §B), the assign_reviewer distinct-from-author check at the but-api boundary (R22), local-cache-only discipline (no DryRun guard), recording the opener in the write-once local_review_meta row (not a comment-body sentinel — R23), and real-but-db+gix hand-assertion tests with a merge-decision-unchanged + a self-assignment-rejected negative control. rust-implementer writes it; rust-reviewer validates the stub is actually gone (returns Ok + a real row), the authority split (no new variant), the distinct-from-author rejection, the write-once opener row, and that no verdict/ref/object/oplog is touched.
coding_standards: crates/AGENTS.md (Result<T,E> + anyhow::Context; but-api is THE API boundary; lower crates must not depend on but-api); crates/but-api/src/legacy/forge.rs (the #[but_api(napi)] + #[instrument] + authorize-before-await idiom to mirror); RULES.md (authorize before the guard; dry runs must not persist refs/objects/oplog — does NOT bind a local-cache row; after changing but-sdk-exposed APIs run pnpm build:sdk && pnpm format — the regen is LPR-010); brain/docs/rust/ (error-handling.md ? + Context; concurrency.md async + into_thread_local)

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: LPR-001 (local_review_assignments + local_review_comments tables/Handles), LPR-002 (AssignmentState::name()/parse())
Blocks:     LPR-005 (derived PR lifecycle reads the assignments + the local_review_meta opener row + the opener's declared kind), LPR-006 (request_review defaults local under keep_reviews_local), LPR-007 (the auto-hook opens the same pending assignment row), LPR-008 (reconciler reads the assignment drive state), LPR-010 (SDK regen for these verbs)
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "LPR-003",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "lpr_governed_repo": {
      "description": "A real governed repo via but_testsupport::writable_scenario + invoke_bash committing .gitbutler/permissions.toml to the target ref. Principals: `rev` granted pull_requests:write + reviews:write; `rev2`/`rev3` reviewer handles; `auth` is the target branch author principal; `impl` granted contents:write ONLY (no pull_requests:write, no reviews:write). A real but_ctx::Context with a real DbHandle (the LPR-001 tables migrated, incl. local_review_meta). BUT_AGENT_HANDLE is set per-case under #[serial_test::serial] via temp_env. Seed assignments + the local_review_meta opener row ONLY via the real verbs (request_review/assign_reviewer), never direct row injection. This is the merge_gate/governed_loop hand-assertion idiom (real but-db + real gix, no mocks, no insta).",
      "seed_method": "public_api",
      "records": [
        "but_testsupport::writable_scenario(...) + invoke_bash committing .gitbutler/permissions.toml (rev: pull_requests:write+reviews:write; impl: contents:write) to refs/heads/main; the target branch author principal is `auth`;",
        "temp_env BUT_AGENT_HANDLE=rev (or impl) under #[serial_test::serial];",
        "drive request_review/assign_reviewer/request_changes_review through the but-api fns (or the but review CLI) — capture the local_review_assignments + local_review_meta opener row + local_review_verdicts row state and the merge-gate decision before/after."
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN lpr_governed_repo (rev holds pull_requests:write+reviews:write); verdict store empty WHEN `but review request refs/heads/feat --reviewer rev2` runs THEN a pending local_review_assignments row exists (target=refs/heads/feat, reviewer_principal=rev2, state=pending, assigned_at set) AND a write-once local_review_meta(refs/heads/feat, \"opener_principal\", rev) row exists AND the local_review_verdicts store is UNCHANGED",
      "verify": "cargo test -p but-api request_review_persists_pending_assignment_without_touching_verdicts",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api request_review + real but-db assignments/meta + real gix via but_testsupport",
        "negative_control": {
          "would_fail_if": [
            "request_review wrote a local_review_verdicts row (a cross-write conflating assignment with verdict) — the verdict store would change",
            "request_review wrote no assignment row (a stub Ok) — the pending row is absent",
            "the opener was recorded as a caller flag (or a __pr_meta__ comment-body sentinel) rather than the dedicated local_review_meta opener row — the meta row is absent (R23)",
            "the state literal were wrong (not 'pending' via AssignmentState::Pending.name())"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_governed_repo",
            "action": { "actor": "agent", "steps": [ "BUT_AGENT_HANDLE=rev", "run request_review(refs/heads/feat, reviewer=rev2)", "read local_review_assignments + local_review_meta + local_review_verdicts" ] },
            "end_state": {
              "must_observe": [
                "a local_review_assignments row (refs/heads/feat, rev2, state=pending, assigned_at set)",
                "a local_review_meta row for (refs/heads/feat, \"opener_principal\") whose value is opener rev (write-once)",
                "the local_review_verdicts store is byte-identical to before (empty)"
              ],
              "must_not_observe": [
                "any local_review_verdicts row written/changed",
                "0 assignment rows (stub Ok with no write)",
                "the opener recorded via a caller-supplied flag or a __pr_meta__ comment-body sentinel rather than the dedicated local_review_meta opener row"
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
      "description": "GIVEN lpr_governed_repo (rev holds reviews:write; the target branch author is `auth`) WHEN `but review assign refs/heads/feat --reviewer rev2` runs twice, then `--reviewer rev3` once, then `--reviewer auth` (a self-assignment: reviewer == target author) THEN exactly ONE row for (refs/heads/feat, rev2) state=pending (idempotent upsert); a rev3 assign adds a second distinct row; AND the `auth` self-assignment is REJECTED/flagged (Err) with NO (refs/heads/feat, auth) row written (R22, distinct-from-author at the but-api boundary)",
      "verify": "cargo test -p but-api assign_reviewer_distinct_from_author_upserts_idempotent",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api assign_reviewer (ReviewsWrite + distinct-from-author at the but-api boundary) + real but-db upsert + real gix",
        "negative_control": {
          "would_fail_if": [
            "the second assign INSERTED a duplicate (2 rows for (target,rev2)) — non-idempotent",
            "assign overwrote the rev3 row onto rev2 (keyed wrong) — rev3 would be lost",
            "the `auth` self-assignment SUCCEEDED (a (refs/heads/feat, auth) row was written) — the distinct-from-author check is missing (R22); the drive layer would narrate a self-assigned reviewer as independently reviewed",
            "assign_reviewer authorized PullRequestsWrite instead of ReviewsWrite — the wrong authority (tech-delta §B)"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_governed_repo",
            "action": { "actor": "agent", "steps": [ "BUT_AGENT_HANDLE=rev", "assign_reviewer(refs/heads/feat, rev2) twice", "assign_reviewer(refs/heads/feat, rev3) once", "assign_reviewer(refs/heads/feat, auth) once (reviewer == the target branch author)", "read list_by_target + capture the auth call's result" ] },
            "end_state": {
              "must_observe": [ "exactly 1 row for (refs/heads/feat, rev2) state=pending", "a distinct row for (refs/heads/feat, rev3)", "the auth self-assignment returned Err (rejected/flagged) with NO (refs/heads/feat, auth) row written (R22)" ],
              "must_not_observe": [ "2 rows for (refs/heads/feat, rev2)", "rev3's row missing after the rev2 upserts", "a (refs/heads/feat, auth) self-assignment row written (R22 missing)" ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN lpr_governed_repo (rev holds reviews:write) with a pending rev assignment; the merge-gate decision captured WHEN `but review request-changes refs/heads/feat --message needs-work` runs THEN it returns Ok (NOT task_contract_invalid — the stub is gone), the rev assignment state==changes_requested, AND the merge-gate decision is IDENTICAL before/after the flip",
      "verify": "cargo test -p but-api request_changes_review_implements_changes_requested_write",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api request_changes_review (the implemented write) + real but-db set_state + real gix; merge-gate decision read before/after via enforce_merge_gate",
        "negative_control": {
          "would_fail_if": [
            "request_changes_review still returns task_contract_invalid (the stub survived) — the call errors instead of Ok and no state flips",
            "the changes_requested flip CHANGED the merge-gate decision (the gate read assignment state) — a safe-seam violation",
            "the write set the wrong state literal (not changes_requested via AssignmentState::ChangesRequested.name())"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_governed_repo",
            "action": { "actor": "agent", "steps": [ "BUT_AGENT_HANDLE=rev", "capture the merge-gate decision for the branch (e.g. blocked: no approval@head)", "run request_changes_review(refs/heads/feat, message)", "read the rev assignment state + re-capture the merge-gate decision" ] },
            "end_state": {
              "must_observe": [
                "request_changes_review returns Ok (the stub task_contract_invalid is gone)",
                "the rev assignment state == changes_requested",
                "the merge-gate decision is identical before and after the flip"
              ],
              "must_not_observe": [
                "an Err(task_contract_invalid) (the stub still in place)",
                "the merge-gate decision changing because of the assignment-state flip",
                "no state change (a no-op Ok)"
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
      "description": "GIVEN lpr_governed_repo (impl holds contents:write only) WHEN `but review request refs/heads/feat --reviewer rev2` (and request-changes) run as impl THEN each exits 1 with stderr JSON {code:perm.denied, message, remediation_hint} naming the required authority AND no local_review_assignments row is written",
      "verify": "cargo test -p but-api request_review_denied_without_authority_writes_nothing",
      "scenario": {
        "tier": "holdout",
        "test_tier": "api-contract",
        "verification_service": "real but-api request_review/request_changes_review composing authorize_branch_action + real but-authz + real gix",
        "negative_control": {
          "would_fail_if": [
            "the verb authorized AFTER writing — a row would exist after the denied call",
            "the denial did not name pull_requests:write / reviews:write — a generic error",
            "a stub returned Ok for an unauthorized caller — exit 0 / a row written"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_governed_repo",
            "action": { "actor": "agent", "steps": [ "BUT_AGENT_HANDLE=impl (contents:write only)", "run request_review(refs/heads/feat, rev2)", "run request_changes_review(refs/heads/feat)", "capture exit code + stderr + the assignment store" ] },
            "end_state": {
              "must_observe": [
                "exit 1 with {code:perm.denied} naming pull_requests:write (request_review) / reviews:write (request_changes_review)",
                "no local_review_assignments row written by either denied call"
              ],
              "must_not_observe": [ "exit 0 / Ok for the unauthorized caller", "an assignment row present after a denied call" ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN lpr_governed_repo (rev holds pull_requests:write); refs/objects/oplog snapshotted WHEN `but review request refs/heads/feat --reviewer rev2` runs (incl. under --dry-run) THEN the assignment row IS written (local cache, like approve_review) AND no ref/object/oplog mutation occurs",
      "verify": "cargo test -p but-api request_review_is_local_cache_only_no_ref_mutation",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api request_review + real gix ref/object/oplog snapshot before/after via but_testsupport",
        "negative_control": {
          "would_fail_if": [
            "a DryRun guard suppressed the local write — the assignment row would be absent",
            "the verb mutated a ref/object/oplog — the before/after snapshots would differ"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_governed_repo",
            "action": { "actor": "agent", "steps": [ "snapshot refs/objects/oplog", "BUT_AGENT_HANDLE=rev; run request_review(refs/heads/feat, rev2)", "re-snapshot refs/objects/oplog; read the assignment store" ] },
            "end_state": {
              "must_observe": [ "the pending assignment row is written", "refs/objects/oplog byte-identical before and after" ],
              "must_not_observe": [ "the assignment row suppressed by a DryRun guard", "any ref/object/oplog mutation" ]
            }
          }
        ]
      }
    },
    { "id": "TC-1", "type": "test_criterion", "description": "request_review (PullRequestsWrite) writes a pending assignment + a write-once local_review_meta(target, \"opener_principal\", caller) row", "verify": "cargo test -p but-api request_review_persists_pending_assignment_without_touching_verdicts", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "after request_review the local_review_verdicts store is unchanged", "verify": "cargo test -p but-api request_review_persists_pending_assignment_without_touching_verdicts", "maps_to_ac": "AC-1" },
    { "id": "TC-3", "type": "test_criterion", "description": "two assign_reviewer calls leave one row (idempotent); a third reviewer adds a distinct row; a self-assignment (reviewer == target author) is rejected with no row written (R22)", "verify": "cargo test -p but-api assign_reviewer_distinct_from_author_upserts_idempotent", "maps_to_ac": "AC-2" },
    { "id": "TC-4", "type": "test_criterion", "description": "request_changes_review returns Ok (not task_contract_invalid) and sets state=changes_requested", "verify": "cargo test -p but-api request_changes_review_implements_changes_requested_write", "maps_to_ac": "AC-3" },
    { "id": "TC-5", "type": "test_criterion", "description": "the merge-gate decision is identical before/after the changes_requested flip", "verify": "cargo test -p but-api request_changes_review_implements_changes_requested_write", "maps_to_ac": "AC-3" },
    { "id": "TC-6", "type": "test_criterion", "description": "request_review as contents:write-only exits 1 perm.denied naming pull_requests:write, no row written", "verify": "cargo test -p but-api request_review_denied_without_authority_writes_nothing", "maps_to_ac": "AC-4" },
    { "id": "TC-7", "type": "test_criterion", "description": "request_review writes the row but refs/objects/oplog are byte-unchanged (local-cache only, no DryRun guard)", "verify": "cargo test -p but-api request_review_is_local_cache_only_no_ref_mutation", "maps_to_ac": "AC-5" }
  ]
}
-->
