# LPR-008: Reconciler read-API — `review_status` serves the full drive state (assignments + unresolved comments + verdict-at-head) in one payload, so two orchestrators converge

> Status: ✅ Completed
> Commit: 71545a9d85
> Reviewer: deferred to PHASE 4.5 red-hat closeout — committed prior session; review_status full reconciler drive state
> Updated: 2026-06-22T18:07:12Z

## What this does

Extend LPR-005's `review_status` so a single branch-scoped read serves the **full review-drive state** for a target in **one payload**: the open `pending` assignments, the unresolved comment threads, and the verdict-at-head. This is what makes an orchestrator a **reconciler over `but` review state**, not a private state machine — it reads everything it needs to decide the next action (dispatch a reviewer / dispatch remediation / attempt the merge) from one `but review status` call, with no per-orchestrator shadow state. Two independent orchestrators reading the same repo converge because they read the same deterministic payload, and the orchestrator's "approved-at-head" read **agrees with** the gate's verdict-at-head (it reads the same `local_review_verdicts`@head query the gate runs) — without bypassing the gate.

## Why

Sprint 07 · PRD UC-LPR-05 · capability CAP-AUTHZ-01. UC-LPR-05 is the thesis as a behavior: an orchestrator dispatches a reviewer because `but` shows an open assignment, dispatches remediation because `but` shows an unresolved comment, and merges because `but` shows an approved verdict at head — **every decision is a projection of `but`'s own state**. For that to hold, all three drive facts must be served from one read, deterministically, so two orchestrators on the same repo converge and the human and the agents share one source of truth. The orchestrator's approved read is a _presentation_ label; the actual land stays `enforce_merge_gate`'s own re-derivation — the read and the gate **agree**, the read does not replace the gate.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-api review_status_serves_full_drive_state_in_one_payload`: a single `but review status <branch>` payload carries all three drive facts (a `pending` assignment, an unresolved comment thread, and the verdict-at-head) for a branch fixtured with all three, so the orchestrator decides the next action without a second call or a private shadow state. Full gate set in the spec below.

## Scope

- crates/but-api/src/legacy/forge.rs (MODIFY — extend `review_status` (LPR-005's branch-scoped read) to additionally serve the full reconciler drive state — open `pending` assignments + unresolved comment threads + verdict-at-head — in one payload; reuse the three Handles' `list_by_target` queries; the verdict-at-head filter is the EXACT query merge_gate runs)
- crates/but/src/command/legacy/forge/review.rs (MODIFY — `but review status` surfaces the full drive state in its output; route errors through review_gate_cli_error (review.rs:89))
- crates/but/src/args/ (MODIFY — the `but review status <branch>` verb/arg definition if not already added by LPR-005; NOT but-clap per tech-delta §B)
- crates/but-api/tests/reconciler_read.rs (NEW — the PRIMARY but-api proofs AC-1..AC-5 against a real but-db + gix fixture via but_testsupport, hand-assertion style like merge_gate/governed_loop tests)
- packages/but-sdk/src/generated/\*\* (REGENERATE ONLY via `pnpm build:sdk && pnpm format` — NEVER hand-edit; the actual regen + N-API audit is LPR-010's gate)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: LPR-008 — Reconciler read-API: review_status serves the full drive state in one payload
================================================================================

TASK_TYPE:   FEATURE
STATUS:      Backlog
PRIORITY:    P1
AGENT:       implementer=rust-implementer | reviewer=rust-reviewer
EFFORT:      M  (120 min)
PROPOSED-BY: rust-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-LPR-05
CAPABILITIES:CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-api review_status_serves_full_drive_state_in_one_payload
  check: cargo check -p but-api --all-targets
  lint:  cargo clippy -p but-api --all-targets

--------------------------------------------------------------------------------
TYPE DESIGN (Rust shapes this task introduces)
--------------------------------------------------------------------------------
API SURFACE (extend LPR-005's review_status — NOT a new verb):
  - `review_status(ctx, branch)` returns an enriched payload carrying the full reconciler drive state. The payload (extend LPR-005's ReviewStatus / add a ReviewDriveState section) carries:
      `open_assignments: Vec<LocalReviewAssignment>`  (state == pending; from local_review_assignments.list_by_target filtered)
      `unresolved_threads: Vec<…>`                     (resolved == false; grouped by thread_id from local_review_comments.list_by_target filtered)
      `verdict_at_head: Option<…>` / `approved: bool`  (local_review_verdicts.list_by_target filtered to head_oid == current head — the EXACT query merge_gate runs)
  - It stays a BRANCH-SCOPED READ (no write authority — shares the read-posture of governance_status_read / get_review forge.rs:401, but discloses the whole branch's drive state, not per-principal — F-006). Authorize is the pre-call guard if LPR-005 established one; do NOT add a write authority.
ERROR STRATEGY:
  - anyhow::Result at the but-api boundary; `?` propagation; `.context(...)` explains the read. The CLI maps errors through review_gate_cli_error (review.rs:89). A target with NO drive state returns an Ok payload with empty vecs + verdict_at_head=None (a clean empty-state, never an Err).
OWNERSHIP PLAN:
  - `let ctx = ctx.into_thread_local();` / `let repo = ctx.repo.get()?;` (or the sync `&Context` shape LPR-005 chose). Borrow the cache (`ctx.db.get_cache()?`) read-only; the three list_by_target calls return owned Vecs that are filtered/collected into the payload (moved in). The current head OID is read read-only from gix for the verdict-at-head filter.
DETERMINISM:
  - The payload MUST be deterministic across two reads of the same state. Reuse the Handles' existing `ORDER BY` (local_review_verdicts.list_by_target ORDER BY created_at ASC, id ASC — local_review_verdicts.rs:64; the LPR-001 assignment/comment list methods carry the same deterministic ordering). NO HashMap/HashSet iteration order anywhere in the payload — use Vec with a stable sort. AC-5 catches nondeterminism.
DOC POINTERS (read before coding):
  - brain/docs/rust/ownership-borrowing.md → iterators + filter/collect into the payload Vecs; borrow the read-only cache
  - brain/docs/rust/error-handling.md → Result + ? + anyhow::Context; empty-state is Ok(empty), not Err
  - brain/docs/rust/traits-generics.md → serde derive on the payload struct (deterministic field order)
  - brain/docs/rust/testing.md → real but-db + gix fixture via but_testsupport; #[serial_test::serial] + temp_env BUT_AGENT_HANDLE; two-reads-converge assertion

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Proven against real but-db + real gix via but_testsupport (hand-assertion style, like merge_gate/governed_loop): (1) one review_status payload carries all three drive facts (a pending assignment + an unresolved thread + the verdict-at-head) for a branch fixtured with all three — the orchestrator decides without a second call; (2) the engine surfaces an open pending assignment as the dispatch trigger (the ENGINE OUTCOME, asserted on the payload, NOT on agent dispatch prose); (3) the engine surfaces an unresolved comment thread as the remediation trigger (the ENGINE OUTCOME, NOT agent remediation prose); (4) the orchestrator's approved-at-head read AGREES with the gate: review_status reports approved/Mergeable AND enforce_merge_gate permits the merge from the SAME local_review_verdicts@head truth; (5) two independent reads of review_status against one repo state yield IDENTICAL drive state (deterministic ordering, no per-orchestrator memory); cargo test -p but-api green; clippy clean.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST serve ALL THREE drive facts (open pending assignments + unresolved comment threads + verdict-at-head) in ONE review_status payload, so the orchestrator never needs a second call or a private shadow state (AC-1). This EXTENDS LPR-005's review_status payload — it is NOT a new verb.
- [MUST] MUST compute verdict-at-head with the EXACT query merge_gate runs: local_review_verdicts.list_by_target(target) filtered to verdict.head_oid == current_head_oid and verdict == "approved" (merge_gate.rs:40 → review_verdicts → list_by_target; review_requirement.rs:94 head_oid==current_head_oid, :8 const APPROVED="approved"). The orchestrator's read and the gate's read MUST agree because they read the SAME truth (AC-4). Do NOT compute approved-at-head a second, divergent way.
- [MUST] MUST keep review_status a BRANCH-SCOPED READ (no write authority — shares the read-posture of governance_status_read / get_review forge.rs:401, NOT per-principal self-scoping). It discloses the whole branch's review drive state (every principal's assignments/threads on the named branch — an accepted branch-scoped disclosure, F-006). Do NOT add a write authority.
- [MUST] MUST make the payload DETERMINISTIC across two reads of the same state: reuse the Handles' existing ORDER BY (created_at ASC, id ASC — local_review_verdicts.rs:64 and the LPR-001 list methods), and use Vec everywhere in the payload. NO HashMap/HashSet iteration-order leakage. AC-5 asserts two reads are byte-identical.
- [MUST] MUST assert the ENGINE OUTCOME in tests (the surfaced drive state in the payload), NEVER an agent's dispatch/remediation prose. The determinism seam: fixture the drive state (a pending assignment / an unresolved thread), assert the payload surfaces it. The orchestrator's behavior is NOT under test here — only that the engine surfaces the state the orchestrator keys on.
- [MUST] MUST return a clean empty-state (Ok payload with empty vecs + verdict_at_head=None) for a target with no drive state — never an Err.
- [NEVER] NEVER let the orchestrator's approved read BYPASS the gate. review_status reporting approved/Mergeable is a PRESENTATION label; the actual land stays enforce_merge_gate's own re-derivation of verdict-at-head. The read AGREES with the gate (AC-4); it does not replace it. (A test that merges purely on the review_status label, without enforce_merge_gate, would encode a bypass — never write that.)
- [NEVER] NEVER add a read of local_review_assignments / local_review_comments to merge_gate.rs / review_requirement.rs — the reconciler read lives in review_status (the drive surface), NOT the gate path (the safe seam, LPR-009 greps this).
- [NEVER] NEVER make the payload nondeterministic (HashMap iteration order, a per-call UUID/timestamp in a position that two reads would differ on) — AC-5 catches it.
- [NEVER] NEVER add a new Authority variant or branch on a role name / human-vs-AI predicate (forge.rs is an ENFORCEMENT_PATH — the invariant_build_gates honesty grep must stay green).
- [NEVER] NEVER hand-edit packages/but-sdk/src/generated — the regen is LPR-010's gate.
- [NEVER] NEVER add new gitbutler-* usage.
- [STRICTLY] STRICTLY treat LPR-005's review_status + the three Handles' list_by_target + merge_gate's verdict-at-head query as CONSUMED seams — extend review_status, reuse the queries; do not fork a parallel drive-state reader or a parallel verdict-at-head computation.
- [STRICTLY] STRICTLY keep the (ctx, branch) signature so the CLI verb and the N-API binding pass the same branch the workspace resolves.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [x] AC-1 [PRIMARY]: one review_status payload carries all three drive facts (pending assignment + unresolved thread + verdict-at-head)
- [x] AC-2: the engine surfaces an open pending assignment as the dispatch trigger (asserted on the payload, not agent prose)
- [x] AC-3: the engine surfaces an unresolved comment thread as the remediation trigger (asserted on the payload, not agent prose)
- [x] AC-4: the orchestrator's approved-at-head read AGREES with the gate's verdict-at-head (same local_review_verdicts@head truth)
- [x] AC-5: two independent reads of review_status against one repo state yield IDENTICAL drive state (deterministic, no per-orchestrator memory)
- [x] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: full drive state in one payload
  GIVEN: lpr_reconciler_repo: a real governed repo + real gix; a branch fixtured (via the real verbs) with a `pending` assignment for rev2, an unresolved comment thread t1, and a distinct `approved` verdict@head; BUT_AGENT_HANDLE=rev under #[serial_test::serial]
  WHEN:  `but review status refs/heads/feat` runs (review_status, the reconciler read)
  THEN:  the SINGLE payload carries all three drive facts: open_assignments contains the `pending` rev2 assignment, unresolved_threads contains thread t1, and verdict_at_head/approved reflects the approval@head — so the orchestrator decides the next action from one read with no shadow state
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api review_status reading all three Handles + the verdict-at-head query + real gix via but_testsupport::writable_scenario
  VERIFY: cargo test -p but-api review_status_serves_full_drive_state_in_one_payload

AC-2: the engine surfaces an open pending assignment (the dispatch trigger — engine outcome, not agent prose)
  GIVEN: lpr_reconciler_repo: a branch with an open `pending` assignment for rev2 (the dispatch trigger), no verdict@head
  WHEN:  `but review status refs/heads/feat` runs
  THEN:  the payload's open_assignments reports the open `pending` rev2 assignment — the ENGINE OUTCOME the reconciler keys on; the test asserts the SURFACED state in the payload, NOT an agent's dispatch decision/prose
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api review_status + real but-db assignment store + real gix
  VERIFY: cargo test -p but-api review_status_surfaces_pending_assignment

AC-3: the engine surfaces an unresolved comment thread (the remediation trigger — engine outcome, not agent prose)
  GIVEN: lpr_reconciler_repo: a branch with one unresolved thread t1 (the remediation trigger)
  WHEN:  `but review status refs/heads/feat` runs
  THEN:  the payload's unresolved_threads reports thread t1 as an open remediation signal — the ENGINE OUTCOME; the test asserts the SURFACED state, NOT agent remediation prose
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api review_status + real but-db comment store + real gix
  VERIFY: cargo test -p but-api review_status_surfaces_unresolved_thread

AC-4: the orchestrator's approved read agrees with the gate's verdict-at-head
  GIVEN: lpr_reconciler_repo: a branch with a distinct `approved` verdict@head in local_review_verdicts (gate-satisfied) + a merge holder
  WHEN:  `but review status refs/heads/feat` runs AND the governed merge (enforce_merge_gate) runs
  THEN:  review_status reports approved/Mergeable AND enforce_merge_gate PERMITS the merge — both reading the SAME local_review_verdicts@head truth (the orchestrator's read and the gate's read agree); the review_status label does NOT itself authorize the merge (enforce_merge_gate re-derives verdict-at-head)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api review_status + real enforce_merge_gate both reading the same local_review_verdicts@head + real gix
  VERIFY: cargo test -p but-api review_status_approved_read_agrees_with_gate

AC-5: two reads of the same state converge (deterministic, no per-orchestrator memory)
  GIVEN: lpr_reconciler_repo: a branch with a fixed drive state (one pending assignment, two unresolved threads, one approved verdict@head); the state is NOT mutated between reads
  WHEN:  `but review status refs/heads/feat` runs TWICE (two independent reads, no shared in-memory state)
  THEN:  the two payloads are IDENTICAL — same open_assignments (same order), same unresolved_threads (same order), same verdict_at_head — so review-drive state is a shared source of truth, not per-orchestrator memory
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api review_status read twice against one unchanged repo state + real gix
  VERIFY: cargo test -p but-api two_reads_of_review_status_converge

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): one review_status payload carries the pending assignment AND the unresolved thread AND the verdict-at-head
    VERIFY: cargo test -p but-api review_status_serves_full_drive_state_in_one_payload
- TC-2 (-> AC-2): review_status's open_assignments surfaces the open `pending` rev2 assignment (asserted on the payload, not prose)
    VERIFY: cargo test -p but-api review_status_surfaces_pending_assignment
- TC-3 (-> AC-3): review_status's unresolved_threads surfaces the unresolved thread t1 (asserted on the payload, not prose)
    VERIFY: cargo test -p but-api review_status_surfaces_unresolved_thread
- TC-4 (-> AC-4): review_status reports approved/Mergeable AND enforce_merge_gate permits the merge from the same local_review_verdicts@head (read agrees with gate)
    VERIFY: cargo test -p but-api review_status_approved_read_agrees_with_gate
- TC-5 (-> AC-5): two reads of review_status against one unchanged state are byte-identical (same vecs, same order — deterministic)
    VERIFY: cargo test -p but-api two_reads_of_review_status_converge

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides:
  - review_status(ctx, branch) extended to serve the full reconciler drive state in one payload: open_assignments (pending) + unresolved_threads (resolved==false) + verdict_at_head (the merge_gate verdict-at-head query) — a branch-scoped read, deterministic across two reads
consumes:
  - crate::legacy::forge::review_status (LPR-005 — the payload extended), get_review (forge.rs:401, the branch-scoped read shape)
  - but_db::{LocalReviewAssignment, LocalReviewComment, LocalReviewVerdict} + the Handle list_by_target methods (LPR-001) — note the deterministic ORDER BY
  - crate::legacy::merge_gate::{the verdict-at-head query} (review_verdicts + review_requirement::evaluate's head_oid filter — REUSED for the agree-with-gate read; the gate path itself is untouched)
boundary_contracts:
  - CAP-AUTHZ-01: review_status is a branch-scoped read serving all three drive facts in one deterministic payload. The orchestrator's approved-at-head read AGREES with enforce_merge_gate (same local_review_verdicts@head query) but does NOT replace or bypass it — the actual land stays the gate's own re-derivation. No read of the two new tables is added to the gate path (the safe seam). No new Authority variant.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-api/src/legacy/forge.rs (MODIFY — extend review_status to serve the full drive state in one payload; reuse the three Handles' list_by_target + the verdict-at-head query)
  - crates/but/src/command/legacy/forge/review.rs (MODIFY — `but review status` surfaces the full drive state; route via review_gate_cli_error)
  - crates/but/src/args/ (MODIFY — the `but review status` verb/arg if not already present from LPR-005; NOT but-clap)
  - crates/but-api/tests/reconciler_read.rs (NEW — the PRIMARY but-api proofs AC-1..AC-5)
  - packages/but-sdk/src/generated/** (REGENERATE ONLY — NEVER hand-edit; the regen gate is LPR-010)
writeProhibited:
  - crates/but-api/src/legacy/merge_gate.rs, review_requirement.rs — CONSUME-only (the safe seam); REUSE the verdict-at-head query but do NOT add any read of local_review_assignments/local_review_comments to the gate path (LPR-009 greps this)
  - crates/but-db/** — CONSUME the LPR-001 tables/Handles; do NOT change the schema or the ORDER BY here
  - crates/but-authz/src/authority.rs — no new Authority variant
  - crates/but-api/src/legacy/forge.rs request_review/assign_reviewer/post_comment/approve_review — CONSUME-only (the writers); do NOT change them (only extend review_status)
  - any gitbutler-* crate (no new gitbutler-* usage)
  - Any file not in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-api/src/legacy/forge.rs (review_status, LPR-005) — [PRIMARY — the payload you EXTEND] the derived PR lifecycle read this task enriches with the full reconciler drive state (open_assignments + unresolved_threads + verdict_at_head). Extend it; do not fork a new verb.
2. crates/but-api/src/legacy/merge_gate.rs [40-90, 155-170] — [THE QUERY YOU AGREE WITH] enforce_merge_gate → review_verdicts(ctx, &review.source_branch) → ctx.db.get_cache().local_review_verdicts().list_by_target(target); the verdict-at-head truth. review_requirement.rs:94 filters verdict.head_oid == current_head_oid; :8 const APPROVED="approved". REUSE this exact query for the orchestrator's approved-at-head read so the read and the gate AGREE — do NOT compute it a divergent second way, and do NOT add a read of the new tables to this file.
3. crates/but-api/src/legacy/review_requirement.rs [8, 37-46, 79-97] — the approved-verdicts filter (head_oid == current_head_oid, verdict == "approved") the verdict-at-head read mirrors.
4. crates/but-db/src/table/local_review_verdicts.rs [62-83] — list_by_target's `ORDER BY created_at ASC, id ASC` — the deterministic ordering that makes two reads converge (AC-5). The LPR-001 assignment/comment list_by_target methods carry the same ordering; reuse them, keep Vec (no HashMap) in the payload.
5. crates/but-api/src/legacy/forge.rs [401] — get_review (the branch-scoped read shape — sync fn(ctx: &Context, …) — no write authority). review_status follows this read posture (whole-branch disclosure, F-006).
6. crates/but-api/tests/ (the merge_gate / governed_loop hand-assertion tests) — [VERIFIED TEST IDIOM] the real-but-db + gix + #[serial_test::serial] + temp_env BUT_AGENT_HANDLE construction (NOT insta). Mirror it for reconciler_read.rs. Seed the drive state via the real verbs (request_review/post_comment/approve_review), then read review_status twice for AC-5.
7. .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/02-uc-lpr.md (UC-LPR-05) + 04-e2e-testing-criteria.md (T-LPR-024..028) — the reconciler thesis + the criteria these ACs realize.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- cargo test -p but-api review_status_serves_full_drive_state_in_one_payload   -> Exit 0; one payload carries all three drive facts
- cargo test -p but-api review_status_surfaces_pending_assignment   -> Exit 0; open_assignments surfaces the pending assignment (engine outcome)
- cargo test -p but-api review_status_surfaces_unresolved_thread   -> Exit 0; unresolved_threads surfaces the open thread (engine outcome)
- cargo test -p but-api review_status_approved_read_agrees_with_gate   -> Exit 0; review_status approved AND enforce_merge_gate permits, same verdict@head
- cargo test -p but-api two_reads_of_review_status_converge   -> Exit 0; two reads byte-identical (deterministic)
- cargo check -p but-api --all-targets   -> Exit 0
- cargo clippy -p but-api --all-targets   -> Exit 0
- cargo test -p but-authz invariant_build_gates   -> Exit 0; forge.rs honesty grep green; no read of the new tables added to the gate path
- cargo fmt --check   -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references:
  - crates/but-api/src/legacy/forge.rs (review_status — the payload to extend), forge.rs:401 (get_review — branch-scoped read shape)
  - crates/but-api/src/legacy/merge_gate.rs:40 (enforce_merge_gate → review_verdicts → list_by_target), review_requirement.rs:94/:8 (verdict-at-head filter the read AGREES with)
  - .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/02-uc-lpr.md UC-LPR-05; 04-e2e-testing-criteria.md T-LPR-024..028
code_skeleton: |
  // extend review_status's payload (LPR-005) with the reconciler drive state
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct ReviewDriveState {
      pub open_assignments: Vec<but_db::LocalReviewAssignment>,   // state == pending
      pub unresolved_threads: Vec<ThreadSummary>,                 // resolved == false, grouped by thread_id
      pub verdict_at_head: Option<but_db::LocalReviewVerdict>,    // approved && head_oid == current head
      pub approved: bool,                                         // the presentation label (a bug here cannot land a merge)
  }
  // inside review_status, reading the read-only cache:
  //   let db = ctx.db.get_cache()?;
  //   let open_assignments = db.local_review_assignments().list_by_target(&branch)?
  //       .into_iter().filter(|a| a.state == AssignmentState::Pending.name()).collect();
  //   let unresolved = db.local_review_comments().list_by_target(&branch)?
  //       .into_iter().filter(|c| !c.resolved && c.thread_id != "__pr_meta__").  // exclude the reserved __pr_meta__ thread_id (reserved/rejected, not a real review thread)
  //       …group by thread_id (deterministic, sorted)…;
  //   let head = current_head_oid(&repo, &source_ref)?;     // read-only
  //   let verdict_at_head = db.local_review_verdicts().list_by_target(&branch)?
  //       .into_iter().rev().find(|v| v.head_oid == head && v.verdict == "approved");  // the gate's exact filter
notes:
  - The verdict_at_head filter MUST be the merge_gate filter (head_oid == current head AND verdict == "approved") so review_status's `approved` agrees with enforce_merge_gate. Reuse the same query; do not re-derive divergently.
  - Exclude the reserved __pr_meta__ thread_id from unresolved_threads (it is a reserved/rejected marker thread, not a real review thread; the agent-PR opener itself lives in the dedicated local_review_meta table per LPR-003, NOT a __pr_meta__ comment).
  - Determinism (AC-5): every Vec in the payload is built from a list_by_target that already ORDER BYs (created_at ASC, id ASC); the thread grouping sorts thread_ids; no HashMap/HashSet iteration order leaks into the payload.
  - CLI: `but review status <branch>` prints the assignment list, the unresolved-thread list, the derived lifecycle (LPR-005), and the verdict-at-head — the single reconciler view.
pattern: extend a branch-scoped read (review_status) to serve all three drive facts (assignments + unresolved comments + verdict-at-head) in one deterministic payload, reusing the Handles' ORDER BY for convergence and the merge_gate verdict-at-head query for agree-with-gate
pattern_source: crates/but-api/src/legacy/forge.rs (review_status, LPR-005; get_review:401); crates/but-api/src/legacy/merge_gate.rs:40 + review_requirement.rs:94/:8 (the verdict-at-head query to reuse)
anti_pattern: a second review_status call needed for the verdict (AC-1 fails — not one payload); computing approved-at-head divergently from merge_gate (AC-4 fails — read disagrees with gate); the review_status label authorizing a merge without enforce_merge_gate (a bypass — never write that); HashMap iteration order in the payload (AC-5 fails — two reads differ); adding a read of the new tables to merge_gate.rs (a safe-seam violation, LPR-009 greps it); asserting agent dispatch/remediation prose instead of the surfaced engine state

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agent: implementer=rust-implementer | reviewer=rust-reviewer
rationale: Extending a branch-scoped read to serve the full reconciler drive state in one deterministic payload, with the subtle contract that the orchestrator's approved read must AGREE with the gate (same verdict-at-head query) without bypassing it, and two reads must converge (deterministic ordering). The fakeability traps are a divergent approved computation (the read lies relative to the gate) and nondeterministic ordering (two reads differ). rust-implementer wires the payload; rust-reviewer validates the verdict-at-head query is the gate's exact query, the payload is Vec-deterministic, and no read of the new tables leaked into the gate path.
coding_standards: crates/AGENTS.md (Result<T,E> + anyhow::Context; but-api is THE API boundary; solve the present problem directly); crates/but-api/src/legacy/forge.rs (the #[but_api] branch-scoped read idiom); RULES.md (prefer commit IDs/refs at API boundaries; lossy presentation views are read-only; after changing but-sdk-exposed APIs run pnpm build:sdk && pnpm format — the regen is LPR-010); brain/docs/rust/ (ownership-borrowing.md filter/collect; testing.md two-reads-converge)

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: LPR-005 (review_status derived PR lifecycle — the payload this extends), LPR-001 (the three Handles' list_by_target + deterministic ORDER BY), LPR-003 (the pending-assignment writer), LPR-004 (the comment/resolve writer)
Blocks:     LPR-009 (the safe-seam tests consume the reconciler reads to prove the drive state never gates), LPR-010 (SDK regen for the enriched review_status payload)
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "LPR-008",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "lpr_reconciler_repo": {
      "description": "A real governed repo via but_testsupport::writable_scenario + invoke_bash committing .gitbutler/permissions.toml to the target ref. Principals: `rev` granted reviews:write + pull_requests:write + comments:write; `rev2` a reviewer handle. A real but_ctx::Context with a real DbHandle (the LPR-001 tables migrated). Seed the drive state ONLY via the real verbs (request_review/assign_reviewer → a pending assignment; post_comment → an unresolved thread; approve_review → an approved verdict@head), never direct row injection. BUT_AGENT_HANDLE is set per-case under #[serial_test::serial] via temp_env. This is the merge_gate/governed_loop hand-assertion idiom (real but-db + real gix, no mocks, no insta). For AC-5 the state is fixtured ONCE and review_status is read twice without mutation.",
      "seed_method": "public_api",
      "records": [
        "but_testsupport::writable_scenario(...) + invoke_bash committing .gitbutler/permissions.toml (rev: reviews:write+pull_requests:write+comments:write) to refs/heads/main;",
        "temp_env BUT_AGENT_HANDLE=rev under #[serial_test::serial];",
        "assign_reviewer(refs/heads/feat, rev2) -> a pending assignment; post_comment(refs/heads/feat, body, thread=t1, resolved=false) -> an unresolved thread; approve_review(refs/heads/feat) -> an approved verdict@head;",
        "read review_status(refs/heads/feat) -> the reconciler payload (read twice unchanged for AC-5)."
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN lpr_reconciler_repo: a branch with a pending rev2 assignment + an unresolved thread t1 + a distinct approved verdict@head WHEN `but review status refs/heads/feat` runs THEN the SINGLE payload carries all three drive facts (open_assignments has the pending rev2 assignment, unresolved_threads has t1, verdict_at_head/approved reflects the approval@head) so the orchestrator decides without a second call or shadow state",
      "verify": "cargo test -p but-api review_status_serves_full_drive_state_in_one_payload",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api review_status reading all three Handles + the verdict-at-head query + real gix via but_testsupport",
        "negative_control": {
          "would_fail_if": [
            "the payload omitted the verdict-at-head (the orchestrator would need a second call) — verdict_at_head missing despite an approval@head",
            "the payload omitted open_assignments or unresolved_threads — an incomplete drive state forcing a shadow state",
            "a stub returned a fixed payload regardless of the fixtured state — the seeded assignment/thread/verdict would be absent or wrong"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_reconciler_repo",
            "action": { "actor": "agent", "steps": [ "BUT_AGENT_HANDLE=rev", "seed a pending rev2 assignment, an unresolved thread t1, and an approved verdict@head via the real verbs", "run review_status(refs/heads/feat)", "inspect the single payload" ] },
            "end_state": {
              "must_observe": [
                "open_assignments contains the pending rev2 assignment",
                "unresolved_threads contains thread t1",
                "verdict_at_head/approved reflects the approval@head — all in ONE payload"
              ],
              "must_not_observe": [
                "verdict_at_head missing despite the seeded approval@head (forces a second call)",
                "open_assignments or unresolved_threads missing (incomplete drive state)",
                "a payload disconnected from the fixtured state (stub)"
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
      "description": "GIVEN lpr_reconciler_repo: a branch with an open pending rev2 assignment, no verdict@head WHEN review_status runs THEN the payload's open_assignments reports the open pending rev2 assignment — the ENGINE OUTCOME (asserted on the payload, NOT an agent's dispatch prose)",
      "verify": "cargo test -p but-api review_status_surfaces_pending_assignment",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api review_status + real but-db assignment store + real gix",
        "negative_control": {
          "would_fail_if": [
            "the test asserted an agent's dispatch decision/prose rather than the surfaced assignment in the payload",
            "open_assignments did not include the pending assignment (the engine did not surface it)",
            "open_assignments included a non-pending assignment (the pending filter is wrong)"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_reconciler_repo",
            "action": { "actor": "agent", "steps": [ "seed only a pending rev2 assignment", "run review_status(refs/heads/feat)", "assert the payload's open_assignments" ] },
            "end_state": {
              "must_observe": [ "open_assignments contains the open pending rev2 assignment (the surfaced engine outcome)" ],
              "must_not_observe": [ "an assertion on agent dispatch prose instead of the payload", "the pending assignment absent from open_assignments" ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN lpr_reconciler_repo: a branch with one unresolved thread t1 WHEN review_status runs THEN the payload's unresolved_threads reports thread t1 as an open remediation signal — the ENGINE OUTCOME (asserted on the payload, NOT agent remediation prose)",
      "verify": "cargo test -p but-api review_status_surfaces_unresolved_thread",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api review_status + real but-db comment store + real gix",
        "negative_control": {
          "would_fail_if": [
            "the test asserted agent remediation prose rather than the surfaced thread in the payload",
            "unresolved_threads did not include t1 (the engine did not surface it)",
            "unresolved_threads included a resolved thread (the resolved==false filter is wrong) or the reserved __pr_meta__ thread"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_reconciler_repo",
            "action": { "actor": "agent", "steps": [ "seed one unresolved thread t1 (and, for the negative control, a resolved thread t2)", "run review_status(refs/heads/feat)", "assert the payload's unresolved_threads" ] },
            "end_state": {
              "must_observe": [ "unresolved_threads contains thread t1 (the surfaced engine outcome)" ],
              "must_not_observe": [ "an assertion on agent remediation prose", "a resolved thread or the __pr_meta__ thread in unresolved_threads" ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN lpr_reconciler_repo: a branch with a distinct approved verdict@head (gate-satisfied) + a merge holder WHEN review_status runs AND the governed merge (enforce_merge_gate) runs THEN review_status reports approved/Mergeable AND enforce_merge_gate PERMITS the merge — both reading the same local_review_verdicts@head truth; the label does NOT itself authorize the merge",
      "verify": "cargo test -p but-api review_status_approved_read_agrees_with_gate",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api review_status + real enforce_merge_gate both reading the same local_review_verdicts@head + real gix",
        "negative_control": {
          "would_fail_if": [
            "review_status reported approved while enforce_merge_gate denied (or vice-versa) — the read disagrees with the gate (a divergent verdict-at-head computation)",
            "the test merged purely on the review_status label without enforce_merge_gate — encoding a bypass",
            "review_status computed approved from a stale/non-head verdict (not head_oid == current head)"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_reconciler_repo",
            "action": { "actor": "agent", "steps": [ "seed a distinct approved verdict@head via approve_review", "run review_status(refs/heads/feat) -> read approved/Mergeable", "run enforce_merge_gate for the branch -> read permit/deny" ] },
            "end_state": {
              "must_observe": [ "review_status reports approved/Mergeable", "enforce_merge_gate permits the merge", "both from the same local_review_verdicts@head truth" ],
              "must_not_observe": [ "review_status approved while the gate denies (disagreement)", "a merge that lands on the review_status label without enforce_merge_gate (bypass)" ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN lpr_reconciler_repo: a branch with a fixed drive state (one pending assignment, two unresolved threads, one approved verdict@head), not mutated between reads WHEN review_status runs TWICE (two independent reads) THEN the two payloads are IDENTICAL (same open_assignments order, same unresolved_threads order, same verdict_at_head) — review-drive state is a shared source of truth, not per-orchestrator memory",
      "verify": "cargo test -p but-api two_reads_of_review_status_converge",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api review_status read twice against one unchanged repo state + real gix",
        "negative_control": {
          "would_fail_if": [
            "the payload used HashMap/HashSet iteration order — two reads would differ in ordering",
            "a per-call timestamp/UUID leaked into a compared position — two reads would differ",
            "the two reads returned different drive state for the same unchanged repo (per-read mutable state)"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_reconciler_repo",
            "action": { "actor": "agent", "steps": [ "fixture a fixed drive state (1 pending assignment, 2 unresolved threads, 1 approved verdict@head)", "read review_status(refs/heads/feat) twice without mutating state", "assert the two payloads are equal" ] },
            "end_state": {
              "must_observe": [ "the two review_status payloads are byte-identical (same vecs, same order, same verdict_at_head)" ],
              "must_not_observe": [ "the two payloads differing in ordering or content for the same unchanged state" ]
            }
          }
        ]
      }
    },
    { "id": "TC-1", "type": "test_criterion", "description": "one review_status payload carries the pending assignment AND the unresolved thread AND the verdict-at-head", "verify": "cargo test -p but-api review_status_serves_full_drive_state_in_one_payload", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "review_status open_assignments surfaces the open pending rev2 assignment (payload, not prose)", "verify": "cargo test -p but-api review_status_surfaces_pending_assignment", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "review_status unresolved_threads surfaces the unresolved thread t1 (payload, not prose)", "verify": "cargo test -p but-api review_status_surfaces_unresolved_thread", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "review_status reports approved/Mergeable AND enforce_merge_gate permits from the same verdict@head (read agrees with gate)", "verify": "cargo test -p but-api review_status_approved_read_agrees_with_gate", "maps_to_ac": "AC-4" },
    { "id": "TC-5", "type": "test_criterion", "description": "two reads of review_status against one unchanged state are byte-identical (deterministic)", "verify": "cargo test -p but-api two_reads_of_review_status_converge", "maps_to_ac": "AC-5" }
  ]
}
-->
