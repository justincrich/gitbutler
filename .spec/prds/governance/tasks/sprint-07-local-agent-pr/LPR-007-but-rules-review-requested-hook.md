# LPR-007: `but-rules` auto "review-requested" hook (commit Trigger → Filter → Action opens a `pending` assignment), reusing the Sprint-06b Trigger→Filter→Action engine

> Status: ✅ Completed
> Commit: c21a2a762
> Reviewer: rust-reviewer (DEFERRED — all 5 ACs satisfied at HEAD; deferred detailed review to PHASE 4.5 red-hat closeout)
> Updated: 2026-06-22T17:39:48Z


## What this does

Extend the shipped `but-rules` (Trigger → Filter → Action) engine — the surface Sprint 06b exposes — with the **two variants it does not yet have**: a new commit `Trigger` variant and a review-assignment `Action` variant. Wired together they form a "review-requested" hook: when a commit matching the rule's filter (branch/principal) lands, the engine fires the action and opens a `pending` `local_review_assignments` row for the configured reviewer — the **same** drive-only row UC-LPR-01/LPR-003 defines. The auto-opened assignment is **drive-only**: it blocks no commit and no merge. The engine mechanism is **reused**, not re-built.

## Why

Sprint 07 · PRD UC-LPR-06 · capability CAP-AUTHZ-01. The reconciler needs an assignment to *exist* before it can dispatch a reviewer — so opening one should be automatic. UC-LPR-06 puts that automation in the **existing** `but-rules` engine rather than a new mechanism: a commit fires a "review-requested" action that opens the local assignment. Today `but-rules` has neither a commit trigger nor a review-assignment action — LPR-007 adds exactly those two variants and the firing path that writes the assignment.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-rules review_requested_hook_creates_pending_assignment`: a fixtured commit matching a configured rule's filter fires the "review-requested" action, and a `pending` `local_review_assignments` row for the configured reviewer is created as the hook's **engine outcome** (the test asserts the DB row, NOT a hook log message). Full gate set in the spec below.

## Scope

  - crates/but-rules/src/lib.rs (MODIFY — ADD a new commit `Trigger` variant beside `FileSytemChange`/`ClaudeCodeHook` (lib.rs:77) + a new review-assignment `Action` variant beside `Explicit(Operation)`/`Implicit(ImplicitOperation)` (lib.rs:140); additive only, exhaustive-match preserved)
  - the `but-rules` rule-evaluation/firing path (MODIFY — the fn that matches a trigger against the rule's filter and runs the action; add the "review-requested" action handler that writes a `pending` `local_review_assignments` row for the configured reviewer, reusing LPR-001's Handle / LPR-003's write internals — do NOT fork a parallel assignment writer)
  - crates/but-db/src/table/workspace_rules.rs (MODIFY IF NEEDED — additive (de)serialization support for the two new variants, if the persisted rule blob needs it; the table itself is unchanged)
  - crates/but-rules/tests/ (NEW — the PRIMARY hook proofs AC-1..AC-4 against a real but-db + gix fixture via but_testsupport, hand-assertion style)
  - crates/but-api/tests/ (NEW — AC-5: the auto-opened assignment is visible in `review_status`, closing the commit→dispatch loop)
  - packages/but-sdk/src/generated/** (REGENERATE ONLY via `pnpm build:sdk && pnpm format` — NEVER hand-edit; the regen gate is LPR-010)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: LPR-007 — but-rules auto "review-requested" hook (commit Trigger -> Action opens a pending assignment)
================================================================================

TASK_TYPE:   FEATURE
STATUS:      Backlog
PRIORITY:    P1
AGENT:       implementer=rust-implementer | reviewer=rust-reviewer
EFFORT:      M  (150 min)
PROPOSED-BY: rust-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-LPR-06
CAPABILITIES:CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-rules review_requested_hook_creates_pending_assignment
  check: cargo check -p but-rules --all-targets
  lint:  cargo clippy -p but-rules --all-targets

--------------------------------------------------------------------------------
TYPE DESIGN (Rust shapes this task introduces)
--------------------------------------------------------------------------------
ENUM VARIANTS (ADDITIVE to the shipped but-rules enums — never a new mechanism):
  - `Trigger::Commit { ... }` (or similarly named) ADDED beside `FileSytemChange` / `ClaudeCodeHook` (crates/but-rules/src/lib.rs:77) — the commit-landed trigger but-rules does not yet have. [NOTE: the shipped variant is literally spelled `FileSytemChange` (a typo in shipped code); do NOT "fix" it — just add the new variant.]
  - a review-assignment `Action` variant ADDED to the `Action` enum beside `Explicit(Operation)` / `Implicit(ImplicitOperation)` (lib.rs:140). The shipped `Action` only carries workspace-staging operations (Operation lib.rs:150 = {Assign{target}, Amend{change_id}, NewCommit{branch_name}}; ImplicitOperation = {AssignToAppropriateBranch, AbsorbIntoDependentCommit, ...}) — NONE opens a review assignment. The new variant carries the reviewer principal to assign (e.g. `Action::RequestReview { reviewer: String }`).
  - both new variants get the same `#[derive(Serialize, Deserialize, Debug, Clone)]` + `#[serde(rename_all = "camelCase", tag = "type", content = "subject")]` shape the sibling variants use (read lib.rs:75-145 to match the serde convention exactly).
OWNERSHIP PLAN:
  - the firing path takes the matched rule + the commit context by reference, resolves the configured reviewer from the action variant, and writes a `pending` `LocalReviewAssignment` via the LPR-001 Handle (the row is built and moved into upsert). Reuse LPR-003's write internals; do not re-implement the upsert.
ERROR STRATEGY:
  - the firing path returns the crate's existing result type (anyhow::Result or but-rules' own) — match the surrounding rule-evaluation fns. The hook write failing surfaces an error; it NEVER blocks the commit (the commit already landed; the hook is a post-commit drive-metadata write).
DOC POINTERS (read before coding):
  - brain/docs/rust/traits-generics.md → additive enum variants + EXHAUSTIVE match (no wildcard arm that hides the new variant)
  - brain/docs/rust/error-handling.md → Result propagation; the hook never panics / never blocks the commit
  - brain/docs/rust/concurrency.md → if the rule-evaluation path is async, mirror its async shape
  - brain/docs/rust/testing.md → fixture the trigger (a commit matching the filter); assert the ENGINE OUTCOME (the DB row), never a log message

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Proven against real but-rules + real but-db + real gix via but_testsupport (hand-assertion, like the shipped rule tests): (1) a fixtured commit matching a configured rule's filter fires the "review-requested" action, creating a `pending` local_review_assignments row for the configured reviewer (the ENGINE OUTCOME — asserted on the DB row, NOT a hook log); (2) a maintainer-configured rule (commit Trigger -> review-requested Action) persists via the EXISTING Trigger->Filter->Action engine extended with the two new variants (mechanism reused, not re-built); (3) the hook is SCOPED by the rule's filter — a watched-branch/principal commit opens an assignment, an unwatched one does NOT; (4) the auto-opened assignment is DRIVE-ONLY — a feature-branch commit lands and a verdict-satisfied merge proceeds with the open auto-assignment present (it blocks neither); (5) the auto-opened assignment is visible in `but review status` so the reconciler can dispatch the reviewer without an explicit `but review request`. cargo test -p but-rules / -p but-api green; clippy clean.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST REUSE the shipped but-rules Trigger->Filter->Action engine — ADD ONLY the two new variants (a commit `Trigger` variant + a review-assignment `Action` variant) and the firing handler for the new action. NEVER build a new rules mechanism, a parallel evaluator, or a second persistence path. The engine is the Sprint-06b surface; this is an additive extension of it.
- [MUST] MUST add the two variants the engine does NOT yet have: the `Trigger` enum (lib.rs:77) today is only {FileSytemChange, ClaudeCodeHook} — add the commit trigger; the `Action` enum (lib.rs:140) today is only {Explicit(Operation), Implicit(ImplicitOperation)} (all workspace-staging) — add the review-assignment action. Read lib.rs:75-175 first to confirm neither exists and to match the serde/derive convention of the siblings.
- [MUST] MUST fire the action by writing the SAME `pending` local_review_assignments row LPR-003's request_review/assign_reviewer write — reuse LPR-001's `local_review_assignments_mut().upsert(...)` Handle (state via AssignmentState::Pending.name(), LPR-002). Do NOT fork a parallel assignment writer or a second state literal.
- [MUST] MUST scope the hook via the rule's `Filter` (branch/principal) — only commits matching the filter open an assignment. The principal axis is the existing ClaudeCodeSessionId filter (lib.rs:88/:100) surfaced via WorkspaceRule::session_id() (lib.rs:31). AC-3's unwatched-commit-no-assignment is the proof the hook is filter-scoped, not every-commit.
- [MUST] MUST keep the auto-opened assignment DRIVE-ONLY — it must NOT enter the commit gate or the merge gate path. The commit that triggers the hook has ALREADY landed (the hook is post-commit); the assignment it opens is the same inert drive row UC-LPR-01 defines. AC-4 proves a commit lands and a verdict-satisfied merge proceeds with the auto-assignment present.
- [MUST] MUST keep the match arms EXHAUSTIVE after adding the variants — every `match trigger { … }` / `match action { … }` in the evaluation path explicitly handles the new variant (no `_ => {}` wildcard that silently swallows it). The compiler enforces this if the matches are non-wildcard; preserve that.
- [MUST] MUST assert the ENGINE OUTCOME in tests — the `pending` local_review_assignments row written by the fixtured trigger — NOT a hook log message or agent prose (T-LPR-031: "the test asserts the row written, NOT a hook log message"). The trigger is FIXTURED (a real commit matching the filter); the test reads the DB row.
- [NEVER] NEVER build a new rules mechanism / parallel evaluator (the whole point is reuse of the shipped engine — a new mechanism fails AC-2's "mechanism reused, two variants added").
- [NEVER] NEVER let the auto-opened assignment block a commit or gate a merge — it is drive metadata; adding a read of it to the commit/merge gate is forbidden (LPR-009 greps the merge gate; the commit gate must stay clean too).
- [NEVER] NEVER add a wildcard match arm that hides the new Trigger/Action variant (it would let an unhandled variant silently no-op — a fakeability hole).
- [NEVER] NEVER add new gitbutler-* usage.
- [STRICTLY] STRICTLY treat the shipped Trigger/Filter/Action engine + WorkspaceRule::session_id (lib.rs:31) + the workspace_rules persistence as CONSUMED seams — extend the enums additively and reuse the evaluation/firing path; do not rewrite them.
- [STRICTLY] STRICTLY keep the new variants' serde shape identical to the siblings (`#[serde(rename_all = "camelCase", tag = "type", content = "subject")]`) so the persisted rule blob round-trips and the SDK regen (LPR-010) is mechanical.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [x] AC-1 [PRIMARY]: a fixtured commit matching the rule's filter fires "review-requested" and creates a `pending` assignment for the configured reviewer (the engine outcome — the DB row, not a log)
- [x] AC-2: a commit Trigger -> review-requested Action rule persists via the EXISTING engine extended with the two new variants (mechanism reused)
- [x] AC-3: the hook is filter-scoped — a watched commit opens an assignment, an unwatched one does NOT
- [x] AC-4: the auto-opened assignment is drive-only — a commit lands and a verdict-satisfied merge proceeds with it present
- [x] AC-5: the auto-opened assignment is visible in `but review status` so the reconciler dispatches the reviewer without an explicit `but review request`
- [x] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: the hook fires on a matching commit, creating a pending assignment (engine outcome)
  GIVEN: rules_review_requested_store: a real but-db + gix fixture with a configured rule {trigger=Commit, filter=watched branch/principal, action=RequestReview{reviewer="rev2"}} persisted via the engine; a commit on the watched branch fixtured as the trigger
  WHEN:  the rule-evaluation path runs against the fixtured commit (matching the filter)
  THEN:  a `pending` local_review_assignments row exists for (target=watched branch, reviewer_principal="rev2", state="pending") — the hook's ENGINE OUTCOME; the test asserts the DB ROW, not a hook log message
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-rules rule-evaluation/firing path + real but-db local_review_assignments + real gix commit via but_testsupport
  VERIFY: cargo test -p but-rules review_requested_hook_creates_pending_assignment

AC-2: a maintainer configures the rule on the EXISTING engine (mechanism reused, two variants added)
  GIVEN: the shipped but-rules Trigger->Filter->Action engine extended with the new commit Trigger + review-assignment Action variant
  WHEN:  a maintainer configures a rule whose trigger is a commit and whose action opens a local review assignment, and it is persisted + reloaded via the engine's workspace_rules path
  THEN:  the rule persists and reloads via the SAME engine (the commit Trigger + RequestReview Action variants round-trip through the persisted rule blob) — the mechanism is reused, the two variants are added (NOT a new rules mechanism)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-rules rule persistence (workspace_rules) + the extended Trigger/Action serde round-trip
  VERIFY: cargo test -p but-rules commit_trigger_review_action_variants_persist

AC-3: the hook is scoped by the rule's filter (not every-commit)
  GIVEN: rules_review_requested_store: a rule filtered to a watched branch (or watched principal); two commits — one on the watched branch, one on an unwatched branch (or by an unwatched principal)
  WHEN:  the rule-evaluation path runs against each commit
  THEN:  a `pending` assignment row is created for the WATCHED commit and NOT for the UNWATCHED one — the hook is filter-scoped, not every-commit
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-rules filter evaluation + real but-db + real gix (two commits)
  VERIFY: cargo test -p but-rules review_requested_hook_scoped_by_filter

AC-4: the auto-opened assignment is drive-only (blocks no commit, gates no merge)
  GIVEN: rules_review_requested_store: the hook has fired so a `pending` auto-assignment exists; a governed repo where a verdict@head satisfies the merge requirement
  WHEN:  a feature-branch commit is attempted AND a verdict-satisfied merge is attempted with the open auto-assignment present
  THEN:  the commit LANDS and the verdict-satisfied merge PROCEEDS with the open auto-assignment present — the hook creates drive metadata, never a block
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-rules hook + real but-api commit gate + enforce_merge_gate + real gix (the auto-assignment never enters either gate)
  VERIFY: cargo test -p but-rules auto_assignment_blocks_no_commit_no_merge

AC-5: the auto-opened assignment is visible in review_status (closes the commit->dispatch loop)
  GIVEN: rules_review_requested_store: the hook has fired (AC-1) so a `pending` auto-assignment exists on the watched branch
  WHEN:  `but review status <branch>` runs
  THEN:  the auto-opened `pending` assignment appears in `status` — closing the commit->reviewer-dispatch loop WITHOUT an explicit `but review request`; the engine outcome the reconciler reads
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api review_status (LPR-005) reading the auto-opened assignment from the real but-db
  VERIFY: cargo test -p but-api auto_assignment_visible_in_review_status

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): a fixtured matching commit fires the action and writes a pending local_review_assignments row for the configured reviewer (DB row asserted, not a log)
    VERIFY: cargo test -p but-rules review_requested_hook_creates_pending_assignment
- TC-2 (-> AC-2): a commit Trigger + RequestReview Action rule round-trips through the engine's workspace_rules persistence (mechanism reused, variants added)
    VERIFY: cargo test -p but-rules commit_trigger_review_action_variants_persist
- TC-3 (-> AC-3): a watched-branch commit creates an assignment; an unwatched-branch commit does NOT (filter-scoped)
    VERIFY: cargo test -p but-rules review_requested_hook_scoped_by_filter
- TC-4 (-> AC-4): a feature-branch commit lands AND a verdict-satisfied merge proceeds with the open auto-assignment present (drive-only)
    VERIFY: cargo test -p but-rules auto_assignment_blocks_no_commit_no_merge
- TC-5 (-> AC-5): the auto-opened pending assignment appears in `but review status` (commit->dispatch loop closed without an explicit request)
    VERIFY: cargo test -p but-api auto_assignment_visible_in_review_status

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides:
  - a new commit `Trigger` variant + a review-assignment `Action` variant on the shipped but-rules engine (the two variants it does not yet have)
  - the "review-requested" firing handler: a matching commit opens a `pending` local_review_assignments row for the configured reviewer (reusing LPR-001's Handle / LPR-003's write)
consumes:
  - but_rules::{Trigger, Filter, Action, WorkspaceRule::session_id} (lib.rs:77/:88/:140/:31) — the shipped engine, extended additively
  - the but-rules rule-evaluation/firing path (the fn that matches a trigger against a filter and runs the action) — reused
  - but_db::LocalReviewAssignment + the Handle (LPR-001) + AssignmentState::Pending (LPR-002) + LPR-003's pending-assignment write internals
  - but-db workspace_rules persistence (the existing table)
boundary_contracts:
  - CAP-AUTHZ-01: the hook REUSES the shipped Trigger->Filter->Action engine (no new mechanism), is filter-scoped (branch/principal), and writes the SAME inert drive-only local_review_assignments row UC-LPR-01 defines. The auto-opened assignment NEVER enters the commit gate or merge gate — it is drive metadata that blocks no commit and gates no merge.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-rules/src/lib.rs (MODIFY — ADD the commit `Trigger` variant + the review-assignment `Action` variant, additive, same serde/derive shape as the siblings)
  - the but-rules rule-evaluation/firing module (MODIFY — add the "review-requested" action handler that writes the pending assignment; reuse the LPR-001 Handle / LPR-003 write; preserve exhaustive matches)
  - crates/but-db/src/table/workspace_rules.rs (MODIFY IF NEEDED — additive (de)serialization for the two new variants in the persisted rule blob; the table schema is unchanged)
  - crates/but-rules/tests/ (NEW — the PRIMARY hook proofs AC-1..AC-4)
  - crates/but-api/tests/ (NEW — AC-5: the auto-opened assignment visible in review_status)
  - packages/but-sdk/src/generated/** (REGENERATE ONLY — NEVER hand-edit; the regen gate is LPR-010)
writeProhibited:
  - crates/but-api/src/legacy/merge_gate.rs, review_requirement.rs, and the commit gate (crates/but-api/src/commit/gate.rs) — the auto-opened assignment must NEVER enter a gate path; do NOT add a read of local_review_assignments to any gate (LPR-009 greps the merge gate)
  - crates/but-db/src/table/local_review_assignments.rs — CONSUME the LPR-001 Handle; do NOT change the schema
  - crates/but-authz/src/authority.rs — no new Authority variant
  - the shipped but-rules engine mechanism — EXTEND additively (two variants + one handler); do NOT rewrite the evaluator or persistence
  - any gitbutler-* crate (no new gitbutler-* usage)
  - Any file not in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-rules/src/lib.rs [75-175] — [PRIMARY PATTERN] the shipped Trigger enum (:77, today {FileSytemChange, ClaudeCodeHook} — NO commit trigger), Filter enum (:88, incl. ClaudeCodeSessionId for principal scoping), Action enum (:140, today {Explicit(Operation), Implicit(ImplicitOperation)}), Operation (:150, the workspace-staging operations — PROOF none opens an assignment), and the serde/derive shape (`#[serde(rename_all = "camelCase", tag = "type", content = "subject")]`). ADD the two new variants here matching this convention.
2. crates/but-rules/src/lib.rs [30-36] — WorkspaceRule::session_id() -> Option<String> (the ClaudeCodeSessionId association = the principal axis the filter scopes on). The hook's principal scoping reuses this.
3. crates/but-rules/src/ (the rule-evaluation/firing path) — [VERIFY FIRST] find the fn that matches a trigger against a rule's filter and runs the action (read the crate's evaluator). The "review-requested" handler is added HERE; it must preserve exhaustive matches and reuse the assignment write.
4. crates/but-db/src/table/workspace_rules.rs — the persistence path; confirm whether the two new variants need additive (de)serialization support for the stored rule blob.
5. crates/but-db/src/table/local_review_assignments.rs (LPR-001) — the LocalReviewAssignment struct + `local_review_assignments_mut().upsert(...)` Handle the firing handler writes through.
6. crates/but-api/src/legacy/forge.rs (LPR-003 request_review/assign_reviewer) — the pending-assignment write internals to REUSE (state=AssignmentState::Pending.name(); do not fork a parallel writer or a second literal).
7. .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/02-uc-lpr.md (UC-LPR-06) + 03-technical-requirements-delta.md (D8) + 04-e2e-testing-criteria.md (T-LPR-030..034) — the criteria these ACs realize; D8 mandates "extends the existing engine with the new commit Trigger + review-assignment Action variant (the two variants but-rules does not yet have)".

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- cargo test -p but-rules review_requested_hook_creates_pending_assignment   -> Exit 0; a matching commit writes a pending assignment for the configured reviewer (DB row, not a log)
- cargo test -p but-rules commit_trigger_review_action_variants_persist   -> Exit 0; the commit-Trigger + RequestReview-Action rule round-trips via the engine (mechanism reused)
- cargo test -p but-rules review_requested_hook_scoped_by_filter   -> Exit 0; watched commit opens an assignment, unwatched does not
- cargo test -p but-rules auto_assignment_blocks_no_commit_no_merge   -> Exit 0; commit lands + verdict-satisfied merge proceeds with the auto-assignment present
- cargo test -p but-api auto_assignment_visible_in_review_status   -> Exit 0; the auto-opened pending assignment appears in review_status
- cargo check -p but-rules --all-targets   -> Exit 0; the new variants compile; matches exhaustive
- cargo clippy -p but-rules --all-targets   -> Exit 0
- cargo test -p but-authz invariant_build_gates   -> Exit 0; the commit gate / merge gate still reference NO local_review_assignments (the auto-assignment never gates)
- cargo fmt --check   -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references:
  - crates/but-rules/src/lib.rs:77 (Trigger — add the commit variant), :140 (Action — add the review-assignment variant), :150 (Operation — the existing workspace-staging operations), :31 (session_id — the principal axis)
  - .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md D8 / UC-LPR-06 (extend the engine with the two variants, no new mechanism)
  - LPR-001 local_review_assignments Handle + LPR-003 request_review/assign_reviewer (the pending-assignment write to reuse)
notes:
  - The new Trigger variant carries whatever the evaluator needs to match a commit against the filter (e.g. the landed commit's branch/principal); the new Action variant carries the reviewer to assign (e.g. `Action::RequestReview { reviewer: String }`). Name them to read naturally beside the siblings.
  - The firing handler resolves the reviewer from the Action variant + the target branch from the matched commit, then `local_review_assignments_mut().upsert(LocalReviewAssignment{ state: AssignmentState::Pending.name().to_owned(), ... })` — the SAME write LPR-003 does. It runs POST-commit (the commit already landed), so it can never block the commit.
  - The hook is filter-scoped via the rule's Filter (branch/principal). An every-commit hook (AC-3's negative control) would open an assignment for an unwatched commit too — the test catches that.
  - The auto-opened assignment is the same inert drive row LPR-009 proves never gates. This task must NOT add any read of it to the commit gate or merge gate; the invariant_build_gates honesty grep + LPR-009's runtime proofs cover the merge gate, and the commit gate stays clean.
pattern: an additive extension of the shipped but-rules Trigger->Filter->Action engine — two new variants (commit Trigger + review-assignment Action) + a firing handler that opens a pending drive-only assignment reusing the LPR-001/003 write — proven by a fixtured-trigger -> DB-row integration test
pattern_source: crates/but-rules/src/lib.rs:77/:88/:140/:150 (the engine enums to extend); crates/but-db/src/table/local_review_assignments.rs (LPR-001 Handle); crates/but-api/src/legacy/forge.rs (LPR-003 pending-assignment write)
anti_pattern: building a new rules mechanism / parallel evaluator (AC-2 fails — mechanism must be reused); forking a parallel assignment writer instead of reusing LPR-001/003; a wildcard `_ =>` match arm that swallows the new variant; an every-commit hook ignoring the filter (AC-3 fails); the auto-assignment entering the commit/merge gate (it must stay drive-only — AC-4 fails); asserting a hook log message instead of the DB row (AC-1's engine-outcome assertion)

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agent: implementer=rust-implementer | reviewer=rust-reviewer
rationale: An additive extension of the shipped but-rules engine — two new enum variants (a commit Trigger + a review-assignment Action) and a firing handler that opens a pending drive-only assignment reusing the LPR-001/003 write. Requires honest grounding that the engine TODAY has neither variant (Trigger is {FileSytemChange, ClaudeCodeHook}; Action is workspace-staging only), additive enum extension with exhaustive matches, filter-scoping via the existing ClaudeCodeSessionId axis, a real-but-db+gix fixtured-trigger test that asserts the DB row (not a log), and the drive-only guarantee (the auto-assignment never gates). rust-implementer extends the engine; rust-reviewer validates the mechanism is reused (not re-built), the matches are exhaustive, the hook is filter-scoped, and the auto-assignment enters no gate path.
coding_standards: crates/AGENTS.md (but-rules is a but-* modern crate — extend its enums additively; keep types in the crate that owns the concept; solve the present problem directly — no speculative trigger/action variants); RULES.md (use but_testsupport for scenarios; NEVER std::env::temp_dir(); after changing but-sdk-exposed types run pnpm build:sdk && pnpm format — the regen is LPR-010); brain/docs/rust/ (traits-generics.md additive variants + exhaustive match; testing.md fixture-the-trigger-assert-the-row)

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: LPR-001 (local_review_assignments table + Handle), LPR-002 (AssignmentState::Pending), LPR-003 (the pending-assignment write internals the hook reuses), LPR-005 (review_status — AC-5 asserts the auto-opened assignment is visible in the derived read)
Blocks:     LPR-008 (the reconciler observes the auto-opened assignment in the full drive state), LPR-010 (SDK regen for the two new but-rules variants)
```
</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "LPR-007",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "rules_review_requested_store": {
      "description": "A real but-rules + but-db + gix fixture via but_testsupport. Configure a WorkspaceRule via the REAL engine entrypoint (the create-rule path, e.g. but_rules::create_rule) with the NEW commit Trigger + the NEW review-assignment Action {reviewer=\"rev2\"} + a Filter scoping to a watched branch/principal (e.g. ClaudeCodeSessionId or a branch filter). FIXTURE the trigger by landing a real commit on the watched branch (and, for AC-3, a second commit on an unwatched branch). Read the resulting local_review_assignments rows from the real DbHandle. Hand-assertion style (real but-db + real gix, no mocks); the trigger is fixtured and the test asserts the ENGINE OUTCOME (the DB row), never a hook log message.",
      "seed_method": "public_api",
      "records": [
        "configure a rule via the real engine: { trigger: Commit, filter: <watched branch/principal>, action: RequestReview{reviewer:\"rev2\"} } persisted through workspace_rules;",
        "land a real commit on the watched branch (the trigger) via but_testsupport::invoke_git/invoke_bash;",
        "for AC-3: land a second real commit on an UNWATCHED branch;",
        "read local_review_assignments().list_by_target(<watched branch>) from the real DbHandle to assert the engine outcome."
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN rules_review_requested_store with a configured {Commit trigger, watched filter, RequestReview{reviewer=rev2} action} rule and a fixtured commit on the watched branch WHEN the rule-evaluation path runs against the commit THEN a pending local_review_assignments row exists for (watched branch, rev2, state=pending) — the engine outcome; the test asserts the DB ROW, not a hook log",
      "verify": "cargo test -p but-rules review_requested_hook_creates_pending_assignment",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-rules rule-evaluation/firing path + real but-db local_review_assignments + real gix commit via but_testsupport",
        "negative_control": {
          "would_fail_if": [
            "the hook only logged and wrote no row (a log-only stub) — list_by_target returns empty",
            "the assignment was created for the wrong reviewer (not rev2 from the Action) — the row's reviewer_principal differs",
            "the state literal were wrong (not pending via AssignmentState::Pending.name())",
            "the test asserted on a hook log message rather than the DB row (it must read the engine outcome)"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "rules_review_requested_store",
            "action": { "actor": "ci", "steps": [ "configure the Commit->RequestReview{rev2} rule on the watched branch", "land a commit on the watched branch (the trigger)", "run the rule-evaluation path", "read local_review_assignments().list_by_target(<watched branch>)" ] },
            "end_state": {
              "must_observe": [ "a pending local_review_assignments row for (watched branch, rev2, state=pending)" ],
              "must_not_observe": [ "0 assignment rows (log-only stub)", "the row created for a reviewer other than rev2", "the assertion made on a log message instead of the DB row" ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the shipped engine extended with the new commit Trigger + review-assignment Action variant WHEN a maintainer configures a commit-trigger review-requested rule and it is persisted + reloaded via workspace_rules THEN the rule round-trips via the SAME engine (the two new variants serialize/deserialize through the persisted rule blob) — mechanism reused, variants added",
      "verify": "cargo test -p but-rules commit_trigger_review_action_variants_persist",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-rules rule persistence (workspace_rules) + the extended Trigger/Action serde round-trip",
        "negative_control": {
          "would_fail_if": [
            "a NEW parallel rules mechanism were introduced instead of reusing the engine — the rule would not round-trip through workspace_rules",
            "the new variants were not actually added to the shipped enums (a side table) — the engine could not represent the rule",
            "the serde shape diverged from the siblings — the persisted blob would not deserialize"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "rules_review_requested_store",
            "action": { "actor": "ci", "steps": [ "configure a {Commit trigger, RequestReview action} rule via the engine", "persist + reload via workspace_rules", "assert the reloaded rule carries the commit Trigger + RequestReview Action variants" ] },
            "end_state": {
              "must_observe": [ "the rule round-trips through the engine's workspace_rules persistence with both new variants intact" ],
              "must_not_observe": [ "a parallel/new rules mechanism (the rule not represented by the shipped engine)", "a serde round-trip failure for the new variants" ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN rules_review_requested_store: a rule filtered to a watched branch/principal; one commit on the watched branch and one on an unwatched branch WHEN the rule-evaluation path runs against each THEN an assignment is created for the WATCHED commit and NOT for the UNWATCHED one — filter-scoped, not every-commit",
      "verify": "cargo test -p but-rules review_requested_hook_scoped_by_filter",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-rules filter evaluation + real but-db + real gix (two commits)",
        "negative_control": {
          "would_fail_if": [
            "the hook fired on the unwatched commit too (an every-commit hook ignoring the filter) — an assignment would exist for the unwatched branch",
            "the filter matched the wrong axis — the watched commit might not fire"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "rules_review_requested_store",
            "action": { "actor": "ci", "steps": [ "land a commit on the watched branch and one on an unwatched branch", "run the rule-evaluation path against each", "read assignments for both branches" ] },
            "end_state": {
              "must_observe": [ "a pending assignment for the watched branch", "NO assignment for the unwatched branch" ],
              "must_not_observe": [ "an assignment created for the unwatched commit (every-commit hook)", "no assignment for the watched commit (filter matched the wrong axis)" ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN rules_review_requested_store: the hook fired so a pending auto-assignment exists; a governed repo where a verdict@head satisfies the merge requirement WHEN a feature-branch commit AND a verdict-satisfied merge are attempted with the open auto-assignment present THEN the commit LANDS and the merge PROCEEDS — the auto-assignment is drive-only (blocks neither)",
      "verify": "cargo test -p but-rules auto_assignment_blocks_no_commit_no_merge",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-rules hook + real but-api commit gate + enforce_merge_gate + real gix",
        "negative_control": {
          "would_fail_if": [
            "the open auto-assignment blocked the commit (it entered the commit gate) — the commit would be denied",
            "the open auto-assignment gated the verdict-satisfied merge (it entered the merge gate) — the merge would be blocked despite the verdict@head"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "rules_review_requested_store",
            "action": { "actor": "ci", "steps": [ "fire the hook so a pending auto-assignment exists", "attempt a feature-branch commit", "satisfy the merge requirement with a verdict@head and attempt the governed merge" ] },
            "end_state": {
              "must_observe": [ "the commit lands", "the verdict-satisfied merge proceeds with the open auto-assignment present" ],
              "must_not_observe": [ "the commit blocked by the auto-assignment", "the merge blocked by the auto-assignment" ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN rules_review_requested_store: the hook fired so a pending auto-assignment exists on the watched branch WHEN `but review status <branch>` runs THEN the auto-opened pending assignment appears in status — closing the commit->dispatch loop without an explicit `but review request`",
      "verify": "cargo test -p but-api auto_assignment_visible_in_review_status",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api review_status (LPR-005) reading the auto-opened assignment from the real but-db",
        "negative_control": {
          "would_fail_if": [
            "review_status did not surface the auto-opened assignment — the reconciler could not dispatch the reviewer",
            "the assignment required an explicit `but review request` to appear (the auto-hook would not close the loop)"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "rules_review_requested_store",
            "action": { "actor": "ci", "steps": [ "fire the hook (AC-1)", "run review_status(<watched branch>)" ] },
            "end_state": {
              "must_observe": [ "the auto-opened pending assignment appears in review_status" ],
              "must_not_observe": [ "the auto-opened assignment absent from review_status", "the assignment requiring an explicit but review request to surface" ]
            }
          }
        ]
      }
    },
    { "id": "TC-1", "type": "test_criterion", "description": "a fixtured matching commit fires the action and writes a pending assignment for the configured reviewer (DB row, not a log)", "verify": "cargo test -p but-rules review_requested_hook_creates_pending_assignment", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "a commit Trigger + RequestReview Action rule round-trips through the engine's workspace_rules persistence (mechanism reused)", "verify": "cargo test -p but-rules commit_trigger_review_action_variants_persist", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "a watched-branch commit creates an assignment; an unwatched-branch commit does not (filter-scoped)", "verify": "cargo test -p but-rules review_requested_hook_scoped_by_filter", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "a feature-branch commit lands AND a verdict-satisfied merge proceeds with the open auto-assignment present (drive-only)", "verify": "cargo test -p but-rules auto_assignment_blocks_no_commit_no_merge", "maps_to_ac": "AC-4" },
    { "id": "TC-5", "type": "test_criterion", "description": "the auto-opened pending assignment appears in `but review status` (commit->dispatch loop closed without an explicit request)", "verify": "cargo test -p but-api auto_assignment_visible_in_review_status", "maps_to_ac": "AC-5" }
  ]
}
-->
