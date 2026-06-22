# LPR-004: `post_comment`/`list_comments`/`resolve_thread` `#[but_api(napi)]` (`CommentsWrite` writes; resolver-identity on resolve; branch-scoped reads) + `but review comment --file/--line/--thread`/`comments`/`resolve` CLI

> Status: ✅ Completed
> Commit: 22f7a986d0
> Reviewer: deferred to PHASE 4.5 red-hat closeout — committed prior session; post_comment/list_comments/resolve_thread + CLI
> Updated: 2026-06-22T18:07:12Z


## What this does

Add the three `#[but_api(napi)]` verbs that give the local review layer a review-comment thread — `post_comment` (writes a `local_review_comments` row on `CommentsWrite`), `resolve_thread` (flips a whole thread's `resolved` flag on `CommentsWrite` **+ a resolver-identity constraint: the resolver must be the thread author, the assigned reviewer, or a `reviews:write` holder** — R22), and `list_comments` (a branch-scoped READ, no write authority — it returns the whole branch's review surface, not just the caller's) — modeled exactly on the shipped `approve_review` (authorize-before-await, local-cache write, no DryRun guard). Plus the CLI: extend the existing `but review comment` verb with `--file`/`--line`/`--thread` (→ `post_comment`), and add `but review comments` (→ `list_comments`) and `but review resolve <thread_id>` (→ `resolve_thread`), all routed through `review_gate_cli_error`. **No new `Authority` variant.**

## Why

Sprint 07 · PRD UC-LPR-02 · capability CAP-AUTHZ-01. A reviewer needs to _say why_ — not just record a verdict. UC-LPR-02 gives the local layer a file/line-anchored, threaded, resolvable comment surface on the already-shipped `comments:write` authority. The unresolved/resolved flag is a **drive signal** an orchestrator reads (an open thread → dispatch remediation); it is **never** read by the merge gate. The NEW write verb is `post_comment` — the shipped `comment_review` (`forge.rs:568`) is itself a stub (authorizes `CommentsWrite`, then `task_contract_invalid`); `post_comment` is the real local-comment write. `resolve_thread` additionally enforces a **resolver-identity constraint** (R22, tech-delta §B): beyond `CommentsWrite`, the resolver must be the **thread author OR the assigned reviewer OR a `reviews:write` holder** — so a single principal cannot post a `changes_requested`-style thread and self-resolve it to forge a clean "all-clear" drive signal that suppresses remediation for another party. A third unrelated principal cannot self-resolve another party's thread.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-api post_comment_persists_comment_with_resolved_false`: a `comments:write`-holding caller's `but review comment <branch> --body "x" --file f.rs --line 12 --thread t1` persists a `local_review_comments` row (target, author, body, file=Some, line=Some, thread_id, `resolved=false`, created_at) via the additive table, while no verdict/assignment row is touched. Full gate set in the spec below.

## Scope

- crates/but-api/src/legacy/forge.rs (MODIFY — add `post_comment`/`list_comments`/`resolve_thread` `#[but_api(napi)]` fns beside `approve_review` (forge.rs:520); reuse `authorize_branch_action` (forge.rs:47) + the `CommentsWrite` authority (forge.rs:61, the authority the shipped `comment_review` stub at forge.rs:568 already names); `resolve_thread` ALSO enforces a resolver-identity constraint at the `but-api` boundary — resolver ∈ {thread author, assigned reviewer, `reviews:write` holder} (R22) — before the set_resolved write; the read verbs are branch-scoped (no write authority) — they disclose the whole branch's review surface, F-006)
- crates/but/src/command/legacy/forge/review.rs (MODIFY — extend the existing `comment` verb (review.rs:55) with `--file`/`--line`/`--thread` (→ post_comment); add `comments` (→ list_comments) + `resolve <thread_id>` (→ resolve_thread); route errors through review_gate_cli_error (review.rs:89))
- crates/but/src/args/ (MODIFY — the verb/arg definitions for `but review comment --file/--line/--thread`, `but review comments`, `but review resolve <thread_id>`; NOT but-clap per tech-delta §B)
- crates/but-api/tests/local_review_comments_verbs.rs (NEW — the PRIMARY but-api proofs AC-1..AC-6 against a real but-db + gix fixture via but_testsupport, hand-assertion style like merge_gate/governed_loop tests)
- packages/but-sdk/src/generated/\*\* (REGENERATE ONLY via `pnpm build:sdk && pnpm format` — NEVER hand-edit; the actual regen + N-API audit is LPR-010's gate)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: LPR-004 — post_comment/list_comments/resolve_thread verbs + the comment-thread CLI
================================================================================

TASK_TYPE:   FEATURE
STATUS:      Backlog
PRIORITY:    P0
AGENT:       implementer=rust-implementer | reviewer=rust-reviewer
EFFORT:      M  (150 min)
PROPOSED-BY: rust-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-LPR-02
CAPABILITIES:CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-api post_comment_persists_comment_with_resolved_false
  check: cargo check -p but-api --all-targets
  lint:  cargo clippy -p but-api --all-targets

--------------------------------------------------------------------------------
TYPE DESIGN (Rust shapes this task introduces)
--------------------------------------------------------------------------------
API SURFACE (additive #[but_api(napi)] fns, modeled on approve_review forge.rs:520):
  - `async fn post_comment(ctx: ThreadSafeContext, branch: String, body: String, file: Option<String>, line: Option<i64>, thread_id: String) -> Result<()>` (write; CommentsWrite)
  - `async fn resolve_thread(ctx: ThreadSafeContext, branch: String, thread_id: String, resolved: bool) -> Result<()>` (write; CommentsWrite + resolver-identity constraint: resolver ∈ {thread author, assigned reviewer, reviews:write holder} — R22)
  - `async fn list_comments(ctx: ThreadSafeContext, branch: String) -> Result<Vec<but_db::LocalReviewComment>>` (BRANCH-SCOPED READ — no write authority; shares the read-posture of governance_status_read + LocalReviewVerdictsHandle::list_by_target, but returns the whole branch's review surface, not per-principal — F-006)
ERROR STRATEGY:
  - anyhow::Result at the but-api boundary (the shipped convention). authorize_branch_action(...)? propagates the structured perm.denied Denial via `?`; .context("…") explains the operation (cf. approve_review forge.rs:526 `.context(...)`). The CLI maps the error through review_gate_cli_error (review.rs:89) to the structured {code, message, remediation_hint} + exit 1. The read verb returns the rows it reads (filtering out the reserved __pr_meta__ thread).
OWNERSHIP PLAN:
  - `let ctx = ctx.into_thread_local();` then `let repo = ctx.repo.get()?;` (exactly approve_review forge.rs:523-525). authorize_branch_action borrows &repo + &branch and returns the resolved Principal (owned, used for author_principal). file/line are Option<String>/Option<i64> moved into the LocalReviewComment row; the row is MOVED into `local_review_comments_mut().insert(row)`. list_comments builds an owned Vec from list_by_target.
DOC POINTERS (read before coding):
  - brain/docs/rust/error-handling.md → Result + ? + anyhow::Context; the structured Denial propagation
  - brain/docs/rust/concurrency.md → async fn + into_thread_local() (the ThreadSafeContext -> thread-local repo handle pattern)
  - brain/docs/rust/ownership-borrowing.md → Option<T> for file/line; borrow &repo for authorize, move the row into insert
  - brain/docs/rust/testing.md → real but-db + gix fixture via but_testsupport; #[serial_test::serial] + temp_env BUT_AGENT_HANDLE

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Proven against real but-db + real gix via but_testsupport (hand-assertion style, like merge_gate/governed_loop): (1) post_comment as a comments:write caller writes a local_review_comments row (target, author, body, file=Some, line=Some, thread_id, resolved=false, created_at), touching no verdict/assignment row; (2) list_comments returns the target's threads grouped by thread_id with file/line/resolved, EXCLUDING the reserved __pr_meta__ thread; (3) resolve_thread sets resolved=true on every comment in the thread (without touching any local_review_verdicts or local_review_assignments row) ONLY when the resolver is the thread author, the assigned reviewer, or a reviews:write holder — and a THIRD unrelated principal's resolve is REJECTED/flagged (R22), so a self-posted-and-self-resolved thread cannot forge a clean "all-clear" for another party; (4) post_comment from a principal lacking CommentsWrite is denied perm.denied + exit 1 with NO row written; (5) the writes touch ONLY the local cache (no ref/object/oplog mutation; no DryRun guard, matching approve_review); (6) an unresolved comment is a DRIVE signal that never reaches the gate — with an unresolved thread + a verdict@head the governed merge proceeds on verdict-at-head; cargo test -p but-api green; clippy clean.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST model post_comment/resolve_thread EXACTLY on the shipped approve_review (forge.rs:520-546): `#[but_api(napi)] #[instrument(err(Debug))] pub async fn …(ctx: ThreadSafeContext, branch: String, …) -> Result<()>`, then `let ctx = ctx.into_thread_local(); let repo = ctx.repo.get()?; let principal = authorize_branch_action(&repo, &branch, Authority::CommentsWrite)?.context(...)?;` BEFORE any await, then a local-cache write. Read forge.rs:520-546 and mirror it line-for-line (only the authority + the write target differ).
- [MUST] MUST gate post_comment AND resolve_thread on `Authority::CommentsWrite` (forge.rs:61 — the same authority the shipped comment_review stub at forge.rs:568 names). NO new Authority variant. The route->Authority table stays closed (authority.rs:11 unchanged).
- [MUST] MUST enforce a RESOLVER-IDENTITY constraint in resolve_thread BEFORE the set_resolved write (R22, tech-delta §B): beyond CommentsWrite, the resolver principal (resolved from authorize_branch_action) must be the THREAD AUTHOR (the author_principal of a comment in that thread) OR the ASSIGNED REVIEWER (a local_review_assignments reviewer_principal on the target) OR a holder of the higher `reviews:write` authority. A resolver who is NONE of these is REJECTED/flagged (return an Err, a structured denial — NOT a panic) with NO row flipped, so a single principal cannot post a changes_requested-style thread and self-resolve it to forge a clean "all-clear" reconciler signal for another party, and a third unrelated principal cannot self-resolve another party's thread. AC-7's third-party-resolve-rejected negative control is the behavioral proof. (Checking reviews:write requires reading the caller's authority set from the target-ref config — compose the existing authorize machinery; do NOT fork a parallel check.)
- [MUST] MUST make list_comments a BRANCH-SCOPED READ with NO write authority — it shares the read-posture (no write authority) of governance_status_read and the read handle on the sibling table (LocalReviewVerdictsHandle::list_by_target), but it is NOT per-principal self-scoped: it returns the WHOLE branch's comment surface (every principal's threads on the named branch), an ACCEPTED branch-scoped disclosure (F-006). Do NOT claim it keeps cross-principal disclosure gated, and do NOT authorize CommentsWrite for the read.
- [MUST] MUST persist post_comment with resolved=false and a created_at timestamp on insert (UC-LPR-02 AC-1). A code comment carries file=Some + line=Some; a PR-level comment carries file=None + line=None (the nullable convention LPR-001's table follows). AC-1 asserts resolved=false at insert time.
- [MUST] MUST authorize-BEFORE-await and before any write (RULES.md "authorize before the guard"): authorize_branch_action(...)? is the pre-call guard; no `.await` and no `local_review_comments_mut().insert/set_resolved` may run on the denial path. AC-4's no-row-written-on-denial is the behavioral proof.
- [MUST] MUST omit the DryRun guard (matching approve_review forge.rs:520-546): these writes touch ONLY ctx.db.get_cache_mut() (SQLite), not refs/objects/oplog, so RULES.md "dry runs must not persist refs, objects, or oplog" does not bind them. Do NOT add a DryRun check. AC-5 proves no ref/object/oplog mutation occurs.
- [MUST] MUST route the CLI verbs through the existing review_gate_cli_error serializer (review.rs:89) so a denial prints `{code:"perm.denied", message, remediation_hint}` to stderr + exit 1 — identical to the shipped approve/request_changes CLI error path.
- [NEVER] NEVER write, read, or touch local_review_verdicts OR local_review_assignments in any of these three verbs — comments are SEPARATE from verdicts and assignments. post_comment/resolve_thread write ONLY local_review_comments. AC-3's "no verdict/assignment row changed" catches a cross-write.
- [NEVER] NEVER let a caller write to (or resolve, or list-surface) the reserved thread_id "__pr_meta__" — that thread is LPR-003's RESERVED thread_id, not a real review thread; the opener itself lives in the dedicated local_review_meta table, NOT a __pr_meta__ comment row, so __pr_meta__ is reserved/rejected precisely so a comment-body sentinel cannot forge the opener/tag (R23 negative control). post_comment MUST reject/refuse a caller-supplied thread_id=="__pr_meta__" (return an error or normalize it away); list_comments MUST filter out the __pr_meta__ thread from its returned set; resolve_thread MUST refuse "__pr_meta__". An AC asserts a caller cannot post to / surface / resolve __pr_meta__.
- [NEVER] NEVER add a new Authority variant or branch on a role name / human-vs-AI predicate (the invariant_build_gates honesty grep over forge.rs must stay green — forge.rs IS an ENFORCEMENT_PATH, invariant_build_gates.rs:23).
- [NEVER] NEVER feed local_review_comments.body into any gate/decision — the body is data, never code (the safe seam: the gate reads only local_review_verdicts). The comment body is attacker-influenceable free text written by one agent principal and read as context by another; it is NOT sanitized — this is named risk R20 and the implementation MUST NOT present the body as injection-safe (no claim of escaping/sanitization in code or docs). Store and serve the raw body; bounding/escaping for downstream model consumption is an L2 harness concern, out of scope.
- [NEVER] NEVER commit/stage/move a ref or write outside the local cache from any verb (breaks the local-cache-only contract; AC-5 catches a ref/object/oplog mutation).
- [NEVER] NEVER hand-edit packages/but-sdk/src/generated — the regen is LPR-010's gate (run pnpm build:sdk there).
- [NEVER] NEVER add new gitbutler-* usage.
- [STRICTLY] STRICTLY treat approve_review (forge.rs:520) and authorize_branch_action (forge.rs:47) as CONSUMED seams — mirror approve_review's shape and compose authorize_branch_action; do not fork a parallel authorize call or a parallel cache-write helper.
- [STRICTLY] STRICTLY keep the (ctx, branch, body, file, line, thread_id) / (ctx, branch, thread_id, resolved) / (ctx, branch) signatures so the CLI verbs and the N-API binding pass the same branch the workspace resolves.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [x] AC-1 [PRIMARY]: post_comment writes a local_review_comments row with resolved=false + created_at (file/line as Some for a code comment), touching no verdict/assignment row
- [x] AC-2: list_comments returns the target's threads grouped by thread_id with file/line/resolved, EXCLUDING the reserved __pr_meta__ thread
- [x] AC-3: resolve_thread sets resolved=true on the whole thread without changing any local_review_verdicts or local_review_assignments row
- [x] AC-4: post_comment without CommentsWrite is denied perm.denied + exit 1 with NO row written
- [x] AC-5: the comment write is local-cache only — no ref/object/oplog mutation, no DryRun guard (matching approve_review)
- [x] AC-6: an unresolved comment is a DRIVE signal that never reaches the gate — with an unresolved thread + a verdict@head, the governed merge proceeds on verdict-at-head
- [x] AC-7: resolve_thread enforces resolver-identity (R22) — the thread author / assigned reviewer / reviews:write holder CAN resolve; a THIRD unrelated principal CANNOT (rejected/flagged, no row flipped), so a self-posted+self-resolved thread cannot forge a clean "all-clear" for another party
- [x] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: post_comment persists a comment with resolved=false + created_at, no verdict/assignment touched
  GIVEN: lpr_governed_repo: a real governed repo (committed .gitbutler/permissions.toml grants caller `rev` comments:write) + real gix; BUT_AGENT_HANDLE=rev under #[serial_test::serial]; the verdict + assignment stores captured before the call
  WHEN:  `but review comment refs/heads/feat --body "fix this" --file f.rs --line 12 --thread t1` runs (post_comment)
  THEN:  a local_review_comments row exists (target=refs/heads/feat, author_principal=rev, body="fix this", file=Some("f.rs"), line=Some(12), thread_id="t1", resolved=false, created_at set); AND no local_review_verdicts or local_review_assignments row is written/changed — comments are separate
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api post_comment + real but-db local_review_comments + real gix via but_testsupport::writable_scenario
  VERIFY: cargo test -p but-api post_comment_persists_comment_with_resolved_false

AC-2: list_comments groups by thread_id and excludes the reserved __pr_meta__ thread
  GIVEN: lpr_governed_repo with two comments on t1 and one on t2 for the branch, PLUS a __pr_meta__ opener-marker comment (a reserved __pr_meta__-thread row seeded DIRECTLY via the LPR-001 Handle — request_review does NOT write a __pr_meta__ comment; the opener lives in the dedicated local_review_meta table, so __pr_meta__ is purely a reserved/rejected thread_id, the R23 negative control)
  WHEN:  `but review comments refs/heads/feat` runs (list_comments)
  THEN:  the output groups by thread_id and carries each comment's file/line/resolved (the (target,thread_id) grouping is observable); the t1 thread shows 2 comments, t2 shows 1; the __pr_meta__ thread is ABSENT from the returned set (it is a reserved marker, not a real review thread)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api list_comments + real but-db list_by_target + real gix
  VERIFY: cargo test -p but-api list_comments_groups_by_thread

AC-3: resolve_thread (by a PERMITTED resolver) flips the whole thread without touching verdicts/assignments
  GIVEN: lpr_governed_repo with an open thread t1 (2 comments authored by `rev`), an `approved` local_review_assignments row, and a local_review_verdicts row@head; caller `rev` holds comments:write AND is the thread author (a PERMITTED resolver under R22)
  WHEN:  `but review resolve refs/heads/feat t1` runs (resolve_thread, resolved=true) as `rev`
  THEN:  every t1 comment row has resolved=true; AND the local_review_verdicts row and the local_review_assignments row are BYTE-UNCHANGED (resolving a thread affects no verdict and no assignment); the resolver-identity check PASSES because `rev` is the thread author
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api resolve_thread (resolver-identity check passes for the thread author) + real but-db set_resolved + real gix; verdict + assignment stores read before/after
  VERIFY: cargo test -p but-api resolve_thread_flips_thread_only

AC-4: missing-authority denial — perm.denied + exit 1, no row written
  GIVEN: lpr_governed_repo with caller `impl` holding contents:write ONLY (NO comments:write); the comment store captured before
  WHEN:  `but review comment refs/heads/feat --body "x"` runs as `impl` (post_comment)
  THEN:  the call exits 1 with stderr JSON {code:"perm.denied", message, remediation_hint} naming comments:write; AND NO local_review_comments row is written (the authorize-before-write guard ran first)
  TEST_TIER: api-contract   VERIFICATION_SERVICE: real but-api post_comment composing authorize_branch_action + real but-authz + real gix; CLI stderr+exit captured
  VERIFY: cargo test -p but-api post_comment_denied_without_comments_write

AC-5: local-cache only — no ref/object/oplog mutation, no DryRun guard
  GIVEN: lpr_governed_repo with caller `rev` holding comments:write; refs/objects/oplog snapshotted before
  WHEN:  `but review comment refs/heads/feat --body "x" --thread t1` runs (incl. under --dry-run if the CLI exposes it)
  THEN:  the comment row IS written (local cache, like approve_review) AND no ref/object/oplog mutation occurs (the snapshots are byte-identical before/after); FAIL if a DryRun guard suppresses the local write or if any ref/object/oplog changes
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api post_comment + real gix ref/object/oplog snapshot before/after via but_testsupport
  VERIFY: cargo test -p but-api post_comment_is_local_cache_only

AC-6: an unresolved comment is a drive signal that never reaches the gate
  GIVEN: lpr_governed_repo with one UNRESOLVED thread on the branch + a local_review_verdicts approval@head + the merge holder; the merge satisfies the verdict-at-head requirement
  WHEN:  list_comments shows the unresolved thread, then the governed merge is attempted (enforce_merge_gate)
  THEN:  the unresolved thread IS visible in list_comments AND the governed merge PROCEEDS on verdict-at-head — the open thread is read as a remediation signal but never gates the land decision
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api list_comments + real enforce_merge_gate reading only local_review_verdicts@head + real gix
  VERIFY: cargo test -p but-api unresolved_comment_is_drive_signal_not_gate

AC-7: resolver-identity — a third unrelated principal cannot self-resolve another party's thread (R22)
  GIVEN: lpr_governed_repo with an open thread t1 authored by `rev` (the thread author) on the branch; `rev2` is the assigned reviewer; `other` holds comments:write but is NEITHER the thread author NOR the assigned reviewer NOR a reviews:write holder; under #[serial_test::serial]
  WHEN:  `but review resolve refs/heads/feat t1` runs as `other` (a third unrelated principal), then separately as `rev` (thread author) and as a reviews:write holder
  THEN:  `other`'s resolve is REJECTED/flagged (Err, structured denial) with NO t1 comment flipped — a self-posted-and-self-resolved thread cannot forge a clean "all-clear" reconciler signal for another party; BUT the thread author (`rev`), the assigned reviewer (`rev2`), and a reviews:write holder CAN resolve it (the permitted set). The merge-gate decision is unaffected either way (resolve never gates)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api resolve_thread enforcing resolver-identity (author / assigned reviewer / reviews:write) at the but-api boundary + real but-db + real gix
  VERIFY: cargo test -p but-api resolve_thread_resolver_identity_blocks_third_party

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): post_comment writes a local_review_comments row (target, author, body, file=Some, line=Some, thread_id, resolved=false, created_at)
    VERIFY: cargo test -p but-api post_comment_persists_comment_with_resolved_false
- TC-2 (-> AC-1): after post_comment no local_review_verdicts or local_review_assignments row is written/changed
    VERIFY: cargo test -p but-api post_comment_persists_comment_with_resolved_false
- TC-3 (-> AC-2): list_comments groups by thread_id (t1 has 2, t2 has 1) with file/line/resolved, and EXCLUDES the __pr_meta__ thread
    VERIFY: cargo test -p but-api list_comments_groups_by_thread
- TC-4 (-> AC-3): resolve_thread sets resolved=true on both t1 comments and leaves the verdict + assignment rows byte-unchanged
    VERIFY: cargo test -p but-api resolve_thread_flips_thread_only
- TC-5 (-> AC-4): post_comment as a contents:write-only caller exits 1 with perm.denied naming comments:write and writes no comment row
    VERIFY: cargo test -p but-api post_comment_denied_without_comments_write
- TC-6 (-> AC-5): under post_comment the comment row is written but refs/objects/oplog are byte-unchanged (local-cache only, no DryRun guard)
    VERIFY: cargo test -p but-api post_comment_is_local_cache_only
- TC-7 (-> AC-6): with an unresolved thread + a verdict@head, list_comments shows the open thread AND the governed merge proceeds (the comment never gates)
    VERIFY: cargo test -p but-api unresolved_comment_is_drive_signal_not_gate
- TC-8 (-> AC-2): a caller-supplied thread_id=="__pr_meta__" to post_comment is rejected/normalized (the reserved marker thread cannot be written or surfaced as a real thread; this is the R23 negative control — a comment-body sentinel cannot forge the agent-tag opener, which lives in local_review_meta)
    VERIFY: cargo test -p but-api post_comment_rejects_reserved_pr_meta_thread
- TC-9 (-> AC-7): a third unrelated principal's resolve_thread is REJECTED with no row flipped (R22); the thread author / assigned reviewer / reviews:write holder CAN resolve
    VERIFY: cargo test -p but-api resolve_thread_resolver_identity_blocks_third_party

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides:
  - post_comment(ctx, branch, body, file, line, thread_id) #[but_api(napi)] — local-cache review-comment insert (resolved=false) on CommentsWrite
  - resolve_thread(ctx, branch, thread_id, resolved) #[but_api(napi)] — set_resolved on a whole thread on CommentsWrite + a resolver-identity constraint (resolver ∈ {thread author, assigned reviewer, reviews:write holder}; a third unrelated principal is rejected — R22)
  - list_comments(ctx, branch) #[but_api(napi)] — a branch-scoped READ (no write authority) returning the target's threads grouped by thread_id, EXCLUDING the reserved __pr_meta__ marker thread
  - `but review comment --file/--line/--thread`, `but review comments`, `but review resolve <thread_id>` CLI verbs routed through review_gate_cli_error
consumes:
  - crate::legacy::forge::{approve_review (the structural template), authorize_branch_action (forge.rs:47), branch_ref} (COMPOSED — mirror/reuse, never fork)
  - but_db::LocalReviewComment + the comment Handle pair (LPR-001) — insert, list_by_target, list_by_thread, set_resolved
  - but_authz::Authority::CommentsWrite (REUSED — no new variant) + the caller's authority set / reviews:write check (for the resolver-identity constraint, composed via the existing authorize machinery) + but_db::LocalReviewAssignment Handle (to check "assigned reviewer"); governance_status_read (the read-posture precedent — no write authority; note list_comments is BRANCH-scoped, not per-principal self-scoped)
boundary_contracts:
  - CAP-AUTHZ-01: the write verbs authorize CommentsWrite via authorize_branch_action(&repo, &branch, Authority::CommentsWrite)? BEFORE any await/write; a caller lacking comments:write is denied perm.denied + exit 1 with NO row written; no new Authority variant. resolve_thread ADDITIONALLY enforces a resolver-identity constraint at the but-api boundary (resolver ∈ {thread author, assigned reviewer, reviews:write holder}; a third unrelated principal is rejected with no row flipped — R22), so a self-posted+self-resolved thread cannot forge a clean "all-clear". list_comments is a branch-scoped read (no write authority — it returns the whole branch's review surface, every principal's; an accepted branch-scoped disclosure, F-006, NOT per-principal self-scoping). The writes are local-cache only (no DryRun guard, matching approve_review) and never touch local_review_verdicts/local_review_assignments or any ref/object/oplog. The comment body is data-never-code (R20 named, not sanitized); the reserved __pr_meta__ thread is never writable/surfaceable as a real thread (the R23 negative control — the agent-tag opener lives in local_review_meta, not a comment body).

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-api/src/legacy/forge.rs (MODIFY — add post_comment/list_comments/resolve_thread beside approve_review)
  - crates/but/src/command/legacy/forge/review.rs (MODIFY — extend `comment` with --file/--line/--thread; add `comments` + `resolve`; route via review_gate_cli_error)
  - crates/but/src/args/ (MODIFY — the comment/comments/resolve verb+arg definitions; NOT but-clap)
  - crates/but-api/tests/local_review_comments_verbs.rs (NEW — the PRIMARY but-api proofs AC-1..AC-6 + the __pr_meta__ rejection)
  - crates/but/tests/ (MODIFY/NEW — a happy-path CLI test for `but review comment`/`comments`/`resolve` if the CLI test harness requires it; happy-path only per RULES.md — the full CLI happy-path suite is LPR-010)
  - packages/but-sdk/src/generated/** (REGENERATE ONLY — NEVER hand-edit; the regen gate is LPR-010)
writeProhibited:
  - crates/but-api/src/legacy/merge_gate.rs, review_requirement.rs — CONSUME-only (the safe seam); do NOT add any read of local_review_comments to the gate path (LPR-009 greps this)
  - crates/but-db/** — CONSUME the LPR-001 tables/Handles; do NOT change the schema here
  - crates/but-authz/src/authority.rs — no new Authority variant
  - crates/but-api/src/legacy/forge.rs approve_review/comment_review/merge_review — CONSUME approve_review's shape + authorize_branch_action + the CommentsWrite precedent; do NOT change the shipped verbs (only ADD the new fns)
  - local_review_assignments / local_review_verdicts — a comment verb writes neither (AC-1/AC-3 catch a cross-write)
  - any gitbutler-* crate (no new gitbutler-* usage)
  - Any file not in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-api/src/legacy/forge.rs [520-546] — [PRIMARY PATTERN — mirror line-for-line] the shipped approve_review: `#[but_api(napi)] #[instrument(err(Debug))] pub async fn approve_review(ctx: ThreadSafeContext, branch: String) -> Result<()>`, `let ctx = ctx.into_thread_local(); let repo = ctx.repo.get()?; let principal = authorize_branch_action(&repo, &branch, Authority::ReviewsWrite)?.context(...)?;`, then the local-cache write. Your write verbs write local_review_comments instead, gate on CommentsWrite, and omit the DryRun guard for the same reason.
2. crates/but-api/src/legacy/forge.rs [568-...] — [THE CommentsWrite PRECEDENT] comment_review: it authorizes `Authority::CommentsWrite` (forge.rs:61) then returns task_contract_invalid and writes nothing (itself a stub). post_comment is the NEW verb that does the real local-comment write on the SAME CommentsWrite authority — do NOT extend comment_review; ADD post_comment.
3. crates/but-api/src/legacy/forge.rs [47-65] — authorize_branch_action(&repo, &branch, Authority) — the COMPOSED guard that resolves the principal from BUT_AGENT_HANDLE, reads the target-ref config, authorizes, and returns the Principal; the CommentsWrite constant (forge.rs:61). Use this for the write verbs; do not fork an authorize call. list_comments does NOT authorize a write authority (branch-scoped read).
4. crates/but/src/command/legacy/forge/review.rs [55-95] — the shipped `comment` CLI verb + review_gate_cli_error (review.rs:89). Extend `comment` with --file/--line/--thread; add `comments` + `resolve`; route all errors through review_gate_cli_error.
5. crates/but-db/src/table/local_review_comments.rs (LPR-001) — the LocalReviewComment struct (file: Option<String>, line: Option<i64>) + insert/list_by_target/list_by_thread/set_resolved methods you write/read through.
6. crates/but-api/src/legacy/governance.rs (governance_status_read) — the read-posture precedent list_comments shares (no write authority); UNLIKE governance_status_read's per-principal self-scoping, list_comments is BRANCH-scoped (returns the whole branch's review surface, F-006).
7. crates/but-api/tests/ (the merge_gate / governed_loop hand-assertion tests) — [VERIFIED TEST IDIOM] the real-but-db + gix + #[serial_test::serial] + temp_env BUT_AGENT_HANDLE construction these tests use (NOT insta snapshots). Mirror it for local_review_comments_verbs.rs. Seed the governed repo via but_testsupport::writable_scenario + invoke_bash committing .gitbutler/permissions.toml.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- cargo test -p but-api post_comment_persists_comment_with_resolved_false   -> Exit 0; resolved=false + created_at + file/line=Some; no verdict/assignment touched
- cargo test -p but-api list_comments_groups_by_thread   -> Exit 0; grouped by thread_id; __pr_meta__ excluded
- cargo test -p but-api resolve_thread_flips_thread_only   -> Exit 0; whole thread resolved=true; verdict + assignment byte-unchanged
- cargo test -p but-api post_comment_denied_without_comments_write   -> Exit 0; perm.denied naming comments:write; no row written
- cargo test -p but-api post_comment_is_local_cache_only   -> Exit 0; row written; refs/objects/oplog byte-unchanged
- cargo test -p but-api unresolved_comment_is_drive_signal_not_gate   -> Exit 0; open thread visible AND merge proceeds on verdict-at-head
- cargo test -p but-api resolve_thread_resolver_identity_blocks_third_party   -> Exit 0; a third unrelated principal's resolve is rejected (no row flipped); author/assigned-reviewer/reviews:write holder can resolve (R22)
- cargo test -p but-api post_comment_rejects_reserved_pr_meta_thread   -> Exit 0; a caller-supplied __pr_meta__ thread is rejected/normalized, never written/surfaced
- cargo check -p but-api --all-targets   -> Exit 0
- cargo clippy -p but-api --all-targets   -> Exit 0
- cargo test -p but-authz invariant_build_gates   -> Exit 0; forge.rs honesty grep green (no role-name/human-vs-AI branch added)
- cargo fmt --check   -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references:
  - crates/but-api/src/legacy/forge.rs:520 (approve_review — the template), :568 (comment_review — the CommentsWrite precedent/stub), :47 (authorize_branch_action), :61 (CommentsWrite)
  - .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md §B (the verb->Authority table: post_comment/resolve_thread -> CommentsWrite; list_comments branch-scoped read; "local writes need NO DryRun guard"); §A.2 (the local_review_comments shape + R20)
  - .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/04-e2e-testing-criteria.md (T-LPR-008..013 — the criteria these ACs realize)
code_skeleton: |
  // post_comment — mirror approve_review's frame
  #[but_api(napi)]
  #[instrument(err(Debug))]
  pub async fn post_comment(
      ctx: ThreadSafeContext, branch: String, body: String,
      file: Option<String>, line: Option<i64>, thread_id: String,
  ) -> Result<()> {
      // reject the reserved marker thread up front
      anyhow::ensure!(thread_id != "__pr_meta__", "thread_id `__pr_meta__` is reserved");
      let ctx = ctx.into_thread_local();
      let repo = ctx.repo.get()?;
      let author = authorize_branch_action(&repo, &branch, Authority::CommentsWrite)?
          .context("governance config is required to post a local review comment")?;
      ctx.db.get_cache_mut()?.local_review_comments_mut().insert(but_db::LocalReviewComment {
          id: uuid::Uuid::new_v4().to_string(),
          target: branch, author_principal: author.id().as_str().to_owned(),
          body, file, line, thread_id,
          resolved: false, created_at: chrono::Utc::now().naive_utc(),
      })?;
      Ok(())
  }
  // resolve_thread — authorize CommentsWrite, then ENFORCE resolver-identity (R22) before set_resolved:
  //   let resolver = authorize_branch_action(&repo, &branch, Authority::CommentsWrite)?;
  //   anyhow::ensure!(thread_id != "__pr_meta__", "thread_id `__pr_meta__` is reserved");
  //   let permitted = is_thread_author(&db, &branch, &thread_id, resolver.id())
  //       || is_assigned_reviewer(&db, &branch, resolver.id())
  //       || resolver_holds(Authority::ReviewsWrite);   // higher authority overrides
  //   anyhow::ensure!(permitted, "only the thread author, the assigned reviewer, or a reviews:write holder may resolve this thread (R22)");
  //   db.local_review_comments_mut().set_resolved(&thread_id, resolved)?;
  // list_comments — BRANCH-SCOPED READ (no write authority): list_by_target(branch).into_iter().filter(|c| c.thread_id != "__pr_meta__").collect()
notes:
  - The __pr_meta__ rejection in post_comment + the filter in list_comments + the refuse in resolve_thread is the closed-marker discipline LPR-010 greps (the reserved thread is never a real thread).
  - R20 (named, NOT closed): the body is stored/served raw — there is NO sanitization in this verb and the code/docs MUST NOT claim the body is injection-safe. Bounding/escaping for a downstream model is an L2 harness concern.
  - AC-6's merge-proceeds assertion reads enforce_merge_gate with an unresolved thread present + a verdict@head; the merge proceeds because the gate reads only local_review_verdicts (safe seam) — the open thread never gates.
  - CLI: `but review comment <branch> --body <b> [--file <f> --line <n> --thread <t>]`, `but review comments <branch>`, `but review resolve <branch> <thread_id>`. Comments are local cache (no ref-pin caveat) — match approve's CLI posture.
pattern: additive #[but_api(napi)] write verbs (post_comment/resolve_thread on CommentsWrite) modeled on approve_review + a branch-scoped read (list_comments), all local-cache only (no DryRun guard); plus the comment-thread CLI through review_gate_cli_error; the reserved __pr_meta__ thread is never writable/surfaceable; the body is data-never-code (R20 named)
pattern_source: crates/but-api/src/legacy/forge.rs:520 (approve_review), :568 (comment_review CommentsWrite precedent), :47 (authorize_branch_action); crates/but/src/command/legacy/forge/review.rs:55-95 (the comment CLI verb + review_gate_cli_error); crates/but-db/src/table/local_review_comments.rs (LPR-001 Handle)
anti_pattern: a new Authority variant; authorizing AFTER the write (a row written on a denied call — AC-4 fails); skipping the resolver-identity check so a third unrelated principal can self-resolve another party's thread (AC-7 fails — R22); adding a DryRun guard (suppresses the local write — AC-5 fails); writing local_review_verdicts/local_review_assignments from a comment verb (AC-1/AC-3 catch the cross-write); letting a caller write/surface/resolve the reserved __pr_meta__ thread (AC-2/TC-8 catch it — R23 negative control); claiming the comment body is sanitized/injection-safe (R20 must stay named, not closed); authority-gating the branch-scoped list_comments read as a write

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agent: implementer=rust-implementer | reviewer=rust-reviewer
rationale: Additive but-api comment-thread verbs modeled on approve_review, reusing the CommentsWrite authority. Requires authorize-before-write ordering, local-cache-only discipline (no DryRun guard), the branch-scoped read posture for list_comments (F-006: returns the whole branch's surface, not per-principal), the resolve_thread resolver-identity constraint at the but-api boundary (author / assigned reviewer / reviews:write — R22), the reserved-__pr_meta__-thread rejection (the R23 negative control), honest R20 framing (body not sanitized), and real-but-db+gix hand-assertion tests with negative controls (a stub Ok would fail the row-present assertions; a cross-write would fail the no-verdict/no-assignment assertions; an unscoped resolve would fail the thread-only assertion; a third unrelated principal's resolve must be rejected). rust-implementer writes it; rust-reviewer validates the authority reuse (no new variant), the resolver-identity rejection of a third party, the safe-seam non-touch of verdicts/assignments, the __pr_meta__ rejection, and that the body is stored raw with no false sanitization claim.
coding_standards: crates/AGENTS.md (Result<T,E> + anyhow::Context; but-api is THE API boundary; lower crates must not depend on but-api); crates/but-api/src/legacy/forge.rs (the #[but_api(napi)] + #[instrument] + authorize-before-await idiom to mirror); RULES.md (authorize before the guard; dry runs must not persist refs/objects/oplog — does NOT bind a local-cache row; after changing but-sdk-exposed APIs run pnpm build:sdk && pnpm format — the regen is LPR-010); brain/docs/rust/ (error-handling.md ? + Context; concurrency.md async + into_thread_local; ownership-borrowing.md Option<T> for file/line)

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: LPR-001 (local_review_comments + local_review_assignments tables/Handles — insert/list_by_target/list_by_thread/set_resolved + the assignment list for the assigned-reviewer resolver-identity check)
Blocks:     LPR-005 (derived PR lifecycle reads the open-thread count), LPR-008 (reconciler reads the unresolved-comment set), LPR-010 (SDK regen for these verbs + the __pr_meta__ closed-marker grep)
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "LPR-004",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "lpr_governed_repo": {
      "description": "A real governed repo via but_testsupport::writable_scenario + invoke_bash committing .gitbutler/permissions.toml to the target ref. Principals: `rev` granted comments:write (and reviews:write/pull_requests:write where a fixture needs an approval@head or an assignment); `impl` granted contents:write ONLY (no comments:write). A real but_ctx::Context with a real DbHandle (the LPR-001 tables migrated). BUT_AGENT_HANDLE is set per-case under #[serial_test::serial] via temp_env. Seed comments via the real verb (post_comment); for AC-2 seed a reserved __pr_meta__-thread row DIRECTLY via the LPR-001 Handle (request_review does NOT write a __pr_meta__ comment — the opener lives in the dedicated local_review_meta table; __pr_meta__ is a reserved/rejected thread_id, the R23 negative control); seed an approval@head via the governed approve_review for AC-6. This is the merge_gate/governed_loop hand-assertion idiom (real but-db + real gix, no mocks, no insta).",
      "seed_method": "public_api",
      "records": [
        "but_testsupport::writable_scenario(...) + invoke_bash committing .gitbutler/permissions.toml (rev: comments:write[+reviews:write where needed]; rev2: comments:write (the assigned reviewer); other: comments:write but NOT thread-author/assigned-reviewer/reviews:write]; impl: contents:write) to refs/heads/main;",
        "temp_env BUT_AGENT_HANDLE=rev (or impl) under #[serial_test::serial];",
        "drive post_comment/list_comments/resolve_thread through the but-api fns (or the but review CLI); for AC-2 seed a reserved __pr_meta__-thread row directly via the LPR-001 Handle (NOT via request_review); for AC-6 seed an approval@head via the governed approve_review; capture the local_review_comments + local_review_verdicts + local_review_assignments row state and the merge-gate decision."
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN lpr_governed_repo (rev holds comments:write); verdict + assignment stores captured WHEN `but review comment refs/heads/feat --body fix-this --file f.rs --line 12 --thread t1` runs THEN a local_review_comments row exists (target, author=rev, body, file=Some(f.rs), line=Some(12), thread_id=t1, resolved=false, created_at set) AND no local_review_verdicts or local_review_assignments row is written/changed",
      "verify": "cargo test -p but-api post_comment_persists_comment_with_resolved_false",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api post_comment + real but-db local_review_comments + real gix via but_testsupport",
        "negative_control": {
          "would_fail_if": [
            "post_comment wrote no row (a stub Ok) — the comment is absent",
            "the insert defaulted resolved=true — the row comes back resolved",
            "file/line were coerced to a sentinel rather than stored as Some(f.rs)/Some(12)",
            "post_comment also wrote a local_review_verdicts or local_review_assignments row (a cross-write) — those stores would change"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_governed_repo",
            "action": { "actor": "agent", "steps": [ "BUT_AGENT_HANDLE=rev", "run post_comment(refs/heads/feat, body=fix-this, file=Some(f.rs), line=Some(12), thread=t1)", "read local_review_comments + local_review_verdicts + local_review_assignments" ] },
            "end_state": {
              "must_observe": [
                "a local_review_comments row (refs/heads/feat, rev, body=fix-this, file=Some(f.rs), line=Some(12), thread=t1, resolved=false, created_at set)",
                "the local_review_verdicts store byte-identical to before",
                "the local_review_assignments store byte-identical to before"
              ],
              "must_not_observe": [
                "0 comment rows (stub Ok with no write)",
                "the comment row resolved=true at insert",
                "any local_review_verdicts or local_review_assignments row written/changed"
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
      "description": "GIVEN lpr_governed_repo with 2 comments on t1, 1 on t2, plus a reserved __pr_meta__-thread row seeded DIRECTLY via the LPR-001 Handle (request_review does NOT write a __pr_meta__ comment — the opener lives in local_review_meta; __pr_meta__ is reserved/rejected, the R23 negative control) WHEN `but review comments refs/heads/feat` runs THEN the output groups by thread_id (t1 has 2, t2 has 1) with file/line/resolved, and the __pr_meta__ thread is ABSENT from the returned set",
      "verify": "cargo test -p but-api list_comments_groups_by_thread",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api list_comments + real but-db list_by_target + real gix",
        "negative_control": {
          "would_fail_if": [
            "list_comments returned a flat list with no thread grouping — the (target,thread_id) grouping is unobservable",
            "list_comments surfaced the __pr_meta__ marker as a real thread — the reserved marker leaks",
            "a stub returned an empty list — the seeded comments are absent"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_governed_repo",
            "action": { "actor": "agent", "steps": [ "seed 2 comments on t1, 1 on t2 via post_comment; seed a reserved __pr_meta__-thread row DIRECTLY via the LPR-001 Handle (request_review does NOT write a __pr_meta__ comment — the opener lives in local_review_meta)", "BUT_AGENT_HANDLE=rev; run list_comments(refs/heads/feat)" ] },
            "end_state": {
              "must_observe": [
                "the t1 thread carries 2 comments with their file/line/resolved",
                "the t2 thread carries 1 comment",
                "the __pr_meta__ thread is absent from the returned set"
              ],
              "must_not_observe": [
                "the __pr_meta__ thread present in the returned set",
                "no thread grouping observable",
                "0 comments returned (stub empty result)"
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
      "description": "GIVEN lpr_governed_repo with an open thread t1 (2 comments) + an approved assignment + a verdict@head; rev holds comments:write WHEN `but review resolve refs/heads/feat t1` runs THEN every t1 comment is resolved=true AND the local_review_verdicts row and the local_review_assignments row are byte-unchanged",
      "verify": "cargo test -p but-api resolve_thread_flips_thread_only",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api resolve_thread + real but-db set_resolved + real gix; verdict + assignment stores read before/after",
        "negative_control": {
          "would_fail_if": [
            "resolve_thread changed a local_review_verdicts or local_review_assignments row — those stores would differ before/after",
            "resolve_thread flipped only the first comment of the thread (a LIMIT 1 bug) — the second t1 comment stays unresolved",
            "a stub no-op left t1 unresolved"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_governed_repo",
            "action": { "actor": "agent", "steps": [ "seed 2 t1 comments + an approved assignment + a verdict@head", "snapshot the verdict + assignment rows", "BUT_AGENT_HANDLE=rev; run resolve_thread(refs/heads/feat, t1, true)", "read the t1 comments + re-snapshot the verdict + assignment rows" ] },
            "end_state": {
              "must_observe": [
                "both t1 comments have resolved=true",
                "the local_review_verdicts row is byte-unchanged",
                "the local_review_assignments row is byte-unchanged"
              ],
              "must_not_observe": [
                "any verdict or assignment row changed by the resolve",
                "only the first t1 comment resolved (the second still unresolved)",
                "no comment resolved (a no-op)"
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
      "description": "GIVEN lpr_governed_repo (impl holds contents:write only) WHEN `but review comment refs/heads/feat --body x` runs as impl THEN it exits 1 with stderr JSON {code:perm.denied, message, remediation_hint} naming comments:write AND no local_review_comments row is written",
      "verify": "cargo test -p but-api post_comment_denied_without_comments_write",
      "scenario": {
        "tier": "holdout",
        "test_tier": "api-contract",
        "verification_service": "real but-api post_comment composing authorize_branch_action + real but-authz + real gix",
        "negative_control": {
          "would_fail_if": [
            "post_comment authorized AFTER writing — a row would exist after the denied call",
            "the denial did not name comments:write — a generic error",
            "a stub returned Ok for an unauthorized caller — exit 0 / a row written"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_governed_repo",
            "action": { "actor": "agent", "steps": [ "BUT_AGENT_HANDLE=impl (contents:write only)", "run post_comment(refs/heads/feat, body=x)", "capture exit code + stderr + the comment store" ] },
            "end_state": {
              "must_observe": [ "exit 1 with {code:perm.denied} naming comments:write", "no local_review_comments row written" ],
              "must_not_observe": [ "exit 0 / Ok for the unauthorized caller", "a comment row present after a denied call" ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN lpr_governed_repo (rev holds comments:write); refs/objects/oplog snapshotted WHEN `but review comment refs/heads/feat --body x --thread t1` runs (incl. under --dry-run) THEN the comment row IS written (local cache, like approve_review) AND no ref/object/oplog mutation occurs",
      "verify": "cargo test -p but-api post_comment_is_local_cache_only",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api post_comment + real gix ref/object/oplog snapshot before/after via but_testsupport",
        "negative_control": {
          "would_fail_if": [
            "a DryRun guard suppressed the local write — the comment row would be absent",
            "the verb mutated a ref/object/oplog — the before/after snapshots would differ"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_governed_repo",
            "action": { "actor": "agent", "steps": [ "snapshot refs/objects/oplog", "BUT_AGENT_HANDLE=rev; run post_comment(refs/heads/feat, body=x, thread=t1)", "re-snapshot refs/objects/oplog; read the comment store" ] },
            "end_state": {
              "must_observe": [ "the comment row is written", "refs/objects/oplog byte-identical before and after" ],
              "must_not_observe": [ "the comment row suppressed by a DryRun guard", "any ref/object/oplog mutation" ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-6",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN lpr_governed_repo with an UNRESOLVED thread + a local_review_verdicts approval@head + the merge holder (verdict-at-head satisfied) WHEN list_comments shows the unresolved thread and the governed merge is attempted THEN the unresolved thread IS visible AND the governed merge PROCEEDS on verdict-at-head — the open thread is a remediation signal that never gates",
      "verify": "cargo test -p but-api unresolved_comment_is_drive_signal_not_gate",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api list_comments + real enforce_merge_gate reading only local_review_verdicts@head + real gix",
        "negative_control": {
          "would_fail_if": [
            "the unresolved thread BLOCKED the merge (the gate read the comment state) — a safe-seam violation; the merge would be denied despite a verdict@head",
            "list_comments did not surface the unresolved thread — the orchestrator could not read the remediation signal"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_governed_repo",
            "action": { "actor": "agent", "steps": [ "seed an unresolved thread + a governed approval@head", "run list_comments(refs/heads/feat) and capture the unresolved thread", "attempt the governed merge (enforce_merge_gate)" ] },
            "end_state": {
              "must_observe": [
                "the unresolved thread is visible in list_comments",
                "the governed merge proceeds (enforce_merge_gate permits on verdict-at-head)"
              ],
              "must_not_observe": [
                "the merge blocked because of the unresolved comment (the gate read comment state)",
                "the unresolved thread absent from list_comments"
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
      "description": "GIVEN lpr_governed_repo with an open thread t1 authored by rev; rev2 is the assigned reviewer; other holds comments:write but is NEITHER thread author NOR assigned reviewer NOR a reviews:write holder WHEN resolve_thread(t1) runs as other, then as rev (author), then as a reviews:write holder THEN other is REJECTED (Err, no t1 row flipped) so a self-posted+self-resolved thread cannot forge a clean all-clear for another party; rev (author), rev2 (assigned reviewer), and a reviews:write holder CAN resolve (R22)",
      "verify": "cargo test -p but-api resolve_thread_resolver_identity_blocks_third_party",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api resolve_thread enforcing resolver-identity (author / assigned reviewer / reviews:write) at the but-api boundary + real but-db + real gix",
        "negative_control": {
          "would_fail_if": [
            "the third unrelated principal (other) successfully resolved t1 (no resolver-identity check) -- a self-posted+self-resolved thread could forge a clean all-clear for another party (R22)",
            "the resolver-identity check rejected the legitimate thread author / assigned reviewer / reviews:write holder (over-restrictive)",
            "the check consulted the merge gate or flipped a verdict/assignment row (cross-write)"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_governed_repo",
            "action": { "actor": "agent", "steps": [ "seed an open t1 thread authored by rev; assign rev2 as reviewer", "BUT_AGENT_HANDLE=other; run resolve_thread(refs/heads/feat, t1, true) -- capture result + t1 rows", "BUT_AGENT_HANDLE=rev (thread author); run resolve_thread(t1) -- capture result", "BUT_AGENT_HANDLE=<reviews:write holder>; run resolve_thread(t1) -- capture result" ] },
            "end_state": {
              "must_observe": [ "other-resolve returns Err (rejected/flagged) with NO t1 comment flipped (R22)", "the thread author (rev), the assigned reviewer (rev2), and a reviews:write holder CAN resolve t1", "no verdict/assignment row changed by any resolve" ],
              "must_not_observe": [ "other successfully resolving t1 (resolver-identity missing)", "the legitimate author/assigned-reviewer/reviews:write holder being rejected" ]
            }
          }
        ]
      }
    },
    { "id": "TC-1", "type": "test_criterion", "description": "post_comment writes a local_review_comments row (target, author, body, file=Some, line=Some, thread_id, resolved=false, created_at)", "verify": "cargo test -p but-api post_comment_persists_comment_with_resolved_false", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "after post_comment no local_review_verdicts or local_review_assignments row is written/changed", "verify": "cargo test -p but-api post_comment_persists_comment_with_resolved_false", "maps_to_ac": "AC-1" },
    { "id": "TC-3", "type": "test_criterion", "description": "list_comments groups by thread_id (t1=2, t2=1) with file/line/resolved and excludes __pr_meta__", "verify": "cargo test -p but-api list_comments_groups_by_thread", "maps_to_ac": "AC-2" },
    { "id": "TC-4", "type": "test_criterion", "description": "resolve_thread sets resolved=true on both t1 comments and leaves the verdict + assignment rows byte-unchanged", "verify": "cargo test -p but-api resolve_thread_flips_thread_only", "maps_to_ac": "AC-3" },
    { "id": "TC-5", "type": "test_criterion", "description": "post_comment as contents:write-only exits 1 perm.denied naming comments:write, no row written", "verify": "cargo test -p but-api post_comment_denied_without_comments_write", "maps_to_ac": "AC-4" },
    { "id": "TC-6", "type": "test_criterion", "description": "post_comment writes the row but refs/objects/oplog are byte-unchanged (local-cache only, no DryRun guard)", "verify": "cargo test -p but-api post_comment_is_local_cache_only", "maps_to_ac": "AC-5" },
    { "id": "TC-7", "type": "test_criterion", "description": "with an unresolved thread + a verdict@head, list_comments shows the open thread AND the governed merge proceeds (the comment never gates)", "verify": "cargo test -p but-api unresolved_comment_is_drive_signal_not_gate", "maps_to_ac": "AC-6" },
    { "id": "TC-8", "type": "test_criterion", "description": "a caller-supplied thread_id==__pr_meta__ to post_comment is rejected/normalized (the reserved marker thread cannot be written or surfaced as a real thread; R23 negative control)", "verify": "cargo test -p but-api post_comment_rejects_reserved_pr_meta_thread", "maps_to_ac": "AC-2" },
    { "id": "TC-9", "type": "test_criterion", "description": "a third unrelated principal's resolve_thread is rejected with no row flipped (R22); the thread author / assigned reviewer / reviews:write holder can resolve", "verify": "cargo test -p but-api resolve_thread_resolver_identity_blocks_third_party", "maps_to_ac": "AC-7" }
  ]
}
-->
