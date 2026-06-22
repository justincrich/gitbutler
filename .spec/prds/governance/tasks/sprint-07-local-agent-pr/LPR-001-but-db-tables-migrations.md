# LPR-001: `local_review_assignments` + `local_review_comments` + `local_review_meta` tables + 3 `SchemaVersion::Zero` migrations + 3 structs + Handle/HandleMut pairs

> Status: ✅ Completed
> Reviewer: rust-reviewer (APPROVED — all 6 ACs + 6 TCs satisfied at HEAD f900a29fc4)
> Commit: f900a29fc4ad67125c6caaadf5a6c1d9d936dcbd
> Updated: 2026-06-22T16:09:10Z

## What this does

Add the three net-new additive `but-db` tables LPR builds on — `local_review_assignments` (who is assigned to review a target, and that assignment's standing state), `local_review_comments` (threaded, resolvable review comments), and `local_review_meta` (a per-target structured-metadata row that caches the computed agent-PR tag — one `key="opener_principal"` row per target, `PRIMARY KEY(target, key)`) — each as a per-table module mirroring the shipped `local_review_verdicts.rs` precisely: a `pub(crate) const M: &[M<'static>]` migration (`SchemaVersion::Zero`, fresh monotonic id), a `#[derive(Serialize, Deserialize)]` struct, and a `Handle`/`HandleMut` pair with the per-table query/write methods. Registered in `table/mod.rs` and appended to `MIGRATIONS` in `lib.rs`. No FK on `principal_id`/`target`. No change to any existing table.

## Why

Sprint 07 · PRD UC-LPR-01, UC-LPR-02, UC-LPR-04 · capability CAP-AUTHZ-01. The three tables are the **disposable, principal-scoped, local** drive-metadata the rest of LPR layers onto. They are the `local_review_verdicts` class (`crates/but-db/src/table/local_review_verdicts.rs`), NOT the runtime-cleared remote-cache class (`forge_reviews`). `local_review_meta` (tech-delta §A.4) holds the **computed agent-PR tag** in a dedicated `key="opener_principal"` row per target — deliberately NOT an attacker-influenceable comment body (R23 ≠ R20) — so LPR-005's tag derivation has a confined cache. Migration-tolerance (`SchemaVersion::Zero`) is what makes all three legal under the freeze: "migrations that older binaries can still tolerate after the migration runs, such as adding tables" (`crates/but-db/src/lib.rs:167`).

## How to verify

PRIMARY **AC-1** — `cargo test -p but-db local_review_assignments_upsert_and_list_by_target`: a real `DbHandle` (the migrations actually run) inserts/upserts `local_review_assignments` rows and reads them back via `list_by_target`, and the `(target, reviewer_principal)` index makes the upsert idempotent per reviewer. Full gate set in the spec below.

## Scope

- crates/but-db/src/table/local_review_assignments.rs (NEW — the migration `M`, the `LocalReviewAssignment` struct, the `LocalReviewAssignmentsHandle`/`…HandleMut` pair: `list_by_target`, `upsert`, `set_state`)
- crates/but-db/src/table/local_review_comments.rs (NEW — the migration `M`, the `LocalReviewComment` struct, the `LocalReviewCommentsHandle`/`…HandleMut` pair: `list_by_target`, `list_by_thread`, `insert`, `set_resolved`)
- crates/but-db/src/table/local_review_meta.rs (NEW — the migration `M`, the `LocalReviewMeta` struct, the `LocalReviewMetaHandle`/`…HandleMut` pair: `get(target, key)`, `upsert_if_absent(row)` = `INSERT … ON CONFLICT(target, key) DO NOTHING`; `PRIMARY KEY(target, key)`)
- crates/but-db/src/table/mod.rs (MODIFY — add `pub(crate) mod local_review_assignments;` + `pub(crate) mod local_review_comments;` + `pub(crate) mod local_review_meta;` alongside `local_review_verdicts` at mod.rs:10)
- crates/but-db/src/lib.rs (MODIFY — append `table::local_review_assignments::M`, `table::local_review_comments::M`, and `table::local_review_meta::M` to the `MIGRATIONS` slice at lib.rs:130, after `table::local_review_verdicts::M` at lib.rs:142)
- crates/but-db/tests/db/table/local_review_assignments.rs + crates/but-db/tests/db/table/local_review_comments.rs + crates/but-db/tests/db/table/local_review_meta.rs (NEW — the PRIMARY proofs AC-1..AC-6 against a real migrated `DbHandle`, mirroring the existing `but-db/tests/db/table/local_review_verdicts.rs` shape)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: LPR-001 — `local_review_assignments` + `local_review_comments` + `local_review_meta` tables + migrations + structs + Handle pairs
================================================================================

TASK_TYPE:   FEATURE
STATUS:      Backlog
PRIORITY:    P0
EFFORT:      M  (180 min)
AGENT:       implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-LPR-01, UC-LPR-02, UC-LPR-04
CAPABILITIES:CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-db local_review_assignments_upsert_and_list_by_target
  check: cargo check -p but-db --all-targets
  lint:  cargo clippy -p but-db --all-targets

--------------------------------------------------------------------------------
TYPE DESIGN (Rust shapes this task introduces)
--------------------------------------------------------------------------------
NEWTYPES / STRUCTS:
  - `LocalReviewAssignment { id: String, target: String, reviewer_principal: String, state: String, assigned_at: chrono::NaiveDateTime }`
    (state is stored as TEXT — the typed `AssignmentState` mapping is LPR-002's job; this table keeps the TEXT column)
  - `LocalReviewComment { id: String, target: String, author_principal: String, body: String, file: Option<String>, line: Option<i64>, thread_id: String, resolved: bool, created_at: chrono::NaiveDateTime }`
    (`line: i64` — SQLite INTEGER → i64, cf. ForgeReview.number: i64, forge_reviews.rs:48)
  - `LocalReviewMeta { target: String, key: String, value: String, created_at: chrono::NaiveDateTime }`
    (NO `id` column — the PRIMARY KEY is the composite `(target, key)`, tech-delta §A.4; one row per metadata key per target, e.g. `key="opener_principal"`)
OWNERSHIP PLAN:
  - The Handle structs borrow `&'conn rusqlite::Connection` (exactly `LocalReviewVerdictsHandle`, local_review_verdicts.rs:54). Methods take `&self`/`&mut self`; rows are passed BY VALUE into `insert`/`upsert`/`upsert_if_absent` (moved into `params!`). Reads return owned `Vec<LocalReviewAssignment>`/`Vec<LocalReviewComment>`; `LocalReviewMetaHandle::get(target, key)` returns `Option<LocalReviewMeta>` (the at-most-one row per the composite key).
ERROR STRATEGY:
  - Methods return `rusqlite::Result<…>` (the `?` propagation `LocalReviewVerdictsHandle::list_by_target` uses, local_review_verdicts.rs:64). Do NOT introduce anyhow here — the but-db layer is rusqlite-native; anyhow::Context lives at the but-api boundary (LPR-003+).
DOC POINTERS (read before coding):
  - brain/docs/rust/ownership-borrowing.md → `Option<T>` for nullable columns (`file`/`line`), borrow-vs-move for the Connection
  - brain/docs/rust/error-handling.md → `rusqlite::Result` + `?` propagation
  - brain/docs/rust/testing.md → `#[test]` + `assert_eq!`/`assert!` hand-assertion

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Proven against a real migrated `DbHandle` (the migrations actually run, NOT a hand-CREATE-TABLE): (1) `local_review_assignments_mut().upsert(row)` inserts a row read back by `list_by_target(target)` with every field intact, and a second upsert on the same `(target, reviewer_principal)` updates the existing row rather than duplicating it (idempotent per the index); (2) `set_state(target, reviewer_principal, "changes_requested")` flips an existing assignment's state and leaves other rows untouched; (3) `local_review_comments_mut().insert(row)` persists a comment with `resolved=false` + a created_at and `list_by_thread(target, thread_id)` returns it grouped by thread; (4) `set_resolved(thread_id, true)` flips every comment in a thread to resolved without touching another thread; (5) `local_review_meta_mut().upsert_if_absent(row)` writes a `(target, "opener_principal", <id>)` row read back by `get(target, "opener_principal")`, and a SECOND `upsert_if_absent` with a DIFFERENT value is a NO-OP (the `PRIMARY KEY(target, key)` `ON CONFLICT … DO NOTHING` makes the opener write-once — a later caller cannot overwrite a recorded opener, the R23 control-path narrowing); (6) all three migrations are `SchemaVersion::Zero` with fresh ids, registered in `MIGRATIONS`, and a fresh `DbHandle::new` runs them cleanly; cargo test -p but-db green; clippy clean.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST mirror the shipped `local_review_verdicts.rs` module shape EXACTLY for ALL THREE modules: `#![allow(missing_docs)]`, `use crate::{DbHandle, M, SchemaVersion, Transaction};`, a `pub(crate) const M: &[M<'static>] = &[M::up(<id>, SchemaVersion::Zero, "CREATE TABLE …; CREATE INDEX …;")];`, the struct with `#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]`, `impl DbHandle { fn <table>() -> …Handle; fn <table>_mut() -> …HandleMut }`, the same `impl<'conn> Transaction<'conn>` pair, and the Handle/HandleMut structs each holding `conn: &'conn rusqlite::Connection`. Read local_review_verdicts.rs:1-115 and copy the structure verbatim (only the columns/methods differ). `local_review_meta` has NO `CREATE INDEX` (its `PRIMARY KEY(target, key)` is the access path) and NO `id` column.
- [MUST] MUST use `SchemaVersion::Zero` for ALL THREE migrations — they are additive tables older binaries tolerate (the exact criterion at lib.rs:167: "adding tables or columns that they don't require"). NEVER bump the schema version for an additive table.
- [MUST] MUST use fresh, monotonic-by-creation-time `u64` migration ids HIGHER than every existing id so they sort last under `migration::run`'s `sort_by_key(|m| m.up_created_at)` (migration.rs:39). Use `20260621120000` for `local_review_assignments`, `20260621120100` for `local_review_comments`, and `20260621120200` for `local_review_meta` (the ids the tech-delta §A pins — all three > the shipped `local_review_verdicts` id `20260619120000`).
- [MUST] MUST register ALL THREE modules in `crates/but-db/src/table/mod.rs` (alongside `pub(crate) mod local_review_verdicts;` at mod.rs:10) AND append ALL THREE `M` slices to `MIGRATIONS` in `crates/but-db/src/lib.rs:130` (the slice that already lists `table::local_review_verdicts::M` last at lib.rs:142). A table module that is not in `MIGRATIONS` never gets created — the test that runs the real migrations catches this.
- [MUST] MUST give `local_review_assignments` the index `(target, reviewer_principal)` and `local_review_comments` the index `(target, thread_id)` exactly as the SQL below specifies — the upsert idempotency (assignment) and the thread grouping (comment) both depend on these indices.
- [MUST] MUST make `upsert` on `local_review_assignments` idempotent per `(target, reviewer_principal)`: a second upsert with the same target+reviewer UPDATES the existing row (state/assigned_at), it does not insert a duplicate. Implement as `INSERT … ON CONFLICT(target, reviewer_principal) DO UPDATE …` (declare the index/constraint accordingly) OR delete-then-insert keyed on `(target, reviewer_principal)`. AC-1's negative control catches a duplicate row.
- [MUST] MUST give `local_review_meta` the schema `CREATE TABLE local_review_meta(target TEXT NOT NULL, key TEXT NOT NULL, value TEXT NOT NULL, created_at TIMESTAMP NOT NULL, PRIMARY KEY(target, key))` (tech-delta §A.4) and make its WRITE **write-once per `(target, key)`**: `upsert_if_absent` is `INSERT … ON CONFLICT(target, key) DO NOTHING` so a later caller CANNOT overwrite an already-recorded opener (this is the R23 control-path narrowing — the opener is recorded once by the governed `request_review`, never overwritten through the governed path). `get(target, key)` returns the at-most-one row. AC-6's negative control catches a DO-UPDATE (overwriting) impl.
- [MUST] MUST store `local_review_comments.file` as nullable TEXT and `line` as nullable INTEGER (`Option<String>`/`Option<i64>`) — a PR-level comment has both `None`; a code comment carries both. Follow the nullable-column convention in `forge_reviews` (`body: Option<String>`, forge_reviews.rs:50). `resolved` is `BOOL NOT NULL`.
- [MUST] MUST seed test rows via the REAL Handle methods (`upsert`/`insert`/`set_state`/`set_resolved`) over a REAL migrated `DbHandle` — NOT a hand-written `CREATE TABLE` + raw `conn.execute` in the test. The test must prove the registered migration runs and the Handle round-trips. Build the DbHandle the way `but-db/tests/db/table/local_review_verdicts.rs` does (the sibling test is the verified idiom).
- [NEVER] NEVER add a FOREIGN KEY on `principal_id`/`target`/`reviewer_principal`/`author_principal`/`key`/`value` — principals live in committed config, not a table (the `local_review_verdicts` posture: it stores `principal_id TEXT` un-FK'd). `improve_concurrency` sets `PRAGMA foreign_keys = ON` (migration.rs:223), so a declared FK WOULD be enforced — do not declare one. (`local_review_meta`'s `PRIMARY KEY(target, key)` is a composite key, NOT a foreign key — that is allowed and required.)
- [NEVER] NEVER add a FOURTH `local_pull_requests` table — the PR lifecycle is DERIVED at query time (LPR-005), never stored. This task adds exactly THREE tables (`local_review_assignments`, `local_review_comments`, `local_review_meta`); `local_review_meta` caches the computed agent-PR tag, it is NOT a stored PR-lifecycle row.
- [NEVER] NEVER modify, normalize, or reorder any existing table module or any existing `MIGRATIONS` entry — this is APPEND-ONLY to `mod.rs` and `lib.rs`.
- [NEVER] NEVER touch the merge gate, `local_review_verdicts`, or any `but-api`/`but-authz` code — this task is `but-db`-only.
- [NEVER] NEVER add new gitbutler-* usage.
- [STRICTLY] STRICTLY keep the `state` column on `local_review_assignments` as `TEXT` (not an enum at the DB layer) — the typed `AssignmentState` round-trip is LPR-002's boundary concern; this matches `LocalReviewVerdict.verdict: String` (a free TEXT validated by the writer).
- [STRICTLY] STRICTLY keep both structs `#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]` so the but-api boundary (LPR-003+) can serialize them and tests can `assert_eq!` whole rows.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [x] AC-1 [PRIMARY]: `local_review_assignments_mut().upsert(row)` inserts a row read back by `list_by_target`, and a second upsert on the same `(target, reviewer_principal)` UPDATES rather than duplicates (idempotent per the index)
- [x] AC-2: `set_state(target, reviewer_principal, state)` flips an existing assignment's state and leaves other assignment rows untouched
- [x] AC-3: `local_review_comments_mut().insert(row)` persists a comment with `resolved=false` + created_at; `list_by_thread(target, thread_id)` returns it; `list_by_target` returns all the target's comments
- [x] AC-4: `set_resolved(thread_id, true)` flips every comment in that thread to resolved without touching another thread's comments
- [x] AC-5: `local_review_meta_mut().upsert_if_absent(row)` writes a `(target, "opener_principal", id)` row read back by `get`; a SECOND `upsert_if_absent` with a different value is a NO-OP (write-once per `(target, key)` — R23 narrowing)
- [x] AC-6: All THREE migrations are `SchemaVersion::Zero`, registered in `MIGRATIONS`, and run cleanly on a fresh `DbHandle` (the table-not-registered failure is caught)
- [x] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: `upsert` inserts then idempotently updates per `(target, reviewer_principal)`
  GIVEN: lpr_db: a fresh real migrated `DbHandle` (via the but-db test idiom that runs MIGRATIONS); an assignment row A1{id, target="refs/heads/feat", reviewer_principal="rev", state="pending", assigned_at=t0}
  WHEN:  `db.local_review_assignments_mut().upsert(A1)` runs, then `upsert(A1' = same target+reviewer, state="approved", assigned_at=t1)` runs, then `db.local_review_assignments().list_by_target("refs/heads/feat")`
  THEN:  `list_by_target` returns EXACTLY ONE row for `(refs/heads/feat, rev)` whose state == "approved" (the second upsert UPDATED the row, it did not insert a duplicate); the row's fields round-trip (target, reviewer_principal, assigned_at)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-db `DbHandle` with the real migration applied (NOT a hand-CREATE-TABLE) — the sibling idiom in but-db/tests/db/table/local_review_verdicts.rs
  VERIFY: cargo test -p but-db local_review_assignments_upsert_and_list_by_target

AC-2: `set_state` flips one assignment's state and leaves others untouched
  GIVEN: lpr_db: two assignments on the same target — A_rev{reviewer="rev", state="pending"} and A_rev2{reviewer="rev2", state="pending"}
  WHEN:  `db.local_review_assignments_mut().set_state("refs/heads/feat", "rev", "changes_requested")` runs
  THEN:  the `rev` assignment's state == "changes_requested"; the `rev2` assignment's state is STILL "pending" (set_state targets exactly the `(target, reviewer_principal)` pair, not the whole target)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-db `DbHandle` + the assignment Handle
  VERIFY: cargo test -p but-db local_review_assignments_set_state_targets_one_reviewer

AC-3: `insert` persists a comment (`resolved=false`) and both list methods return it
  GIVEN: lpr_db: a comment row C1{id, target="refs/heads/feat", author_principal="rev", body="fix this", file=Some("f.rs"), line=Some(12), thread_id="t1", resolved=false, created_at=t0}
  WHEN:  `db.local_review_comments_mut().insert(C1)` runs, then `list_by_thread("refs/heads/feat","t1")` and `list_by_target("refs/heads/feat")`
  THEN:  `list_by_thread` returns C1 with `resolved==false`, `file==Some("f.rs")`, `line==Some(12)`, `created_at==t0`; `list_by_target` also returns C1; a PR-level comment with `file=None, line=None` round-trips both Nones
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-db `DbHandle` + the comment Handle
  VERIFY: cargo test -p but-db local_review_comments_insert_and_list

AC-4: `set_resolved` flips a whole thread and leaves another thread untouched
  GIVEN: lpr_db: two comments on thread t1 (C1a, C1b) and one on thread t2 (C2) for the same target
  WHEN:  `db.local_review_comments_mut().set_resolved("t1", true)` runs
  THEN:  every t1 comment (C1a, C1b) has `resolved==true`; C2 (thread t2) is STILL `resolved==false` (set_resolved scopes to the thread_id)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-db `DbHandle` + the comment Handle
  VERIFY: cargo test -p but-db local_review_comments_set_resolved_scopes_to_thread

AC-5: `local_review_meta` writes the opener row ONCE per (target, key) and `get` reads it back
  GIVEN: lpr_db: a fresh real migrated `DbHandle`; a meta row M1{target="refs/heads/feat", key="opener_principal", value="agent-A", created_at=t0}
  WHEN:  `db.local_review_meta_mut().upsert_if_absent(M1)` runs, then `upsert_if_absent(M1' = same target+key, value="impostor", created_at=t1)` runs, then `db.local_review_meta().get("refs/heads/feat", "opener_principal")`
  THEN:  `get` returns ONE row whose value == "agent-A" (the SECOND write was a NO-OP — `ON CONFLICT(target,key) DO NOTHING` makes the opener write-once; a later caller cannot overwrite it — the R23 control-path narrowing); `get` on a non-existent (target,key) returns None
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-db `DbHandle` + the meta Handle (write-once-on-conflict)
  VERIFY: cargo test -p but-db local_review_meta_opener_is_write_once_per_target_key

AC-6: All three migrations are SchemaVersion::Zero, registered, and run on a fresh DbHandle
  GIVEN: a fresh `DbHandle::new(<tmp>)` that runs `MIGRATIONS` (the additive registration in lib.rs:130)
  WHEN:  the handle is constructed and the three new tables are queried (`list_by_target` on assignments/comments; `get` on meta) on an empty store
  THEN:  all queries return Ok(empty/None) (the tables EXIST — the migrations ran); `M[0].schema_version` for all three modules is `SchemaVersion::Zero` (asserted structurally — additive); the ids `20260621120000` / `20260621120100` / `20260621120200` are the three highest in `MIGRATIONS`
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-db `DbHandle::new` running the real registered MIGRATIONS
  VERIFY: cargo test -p but-db local_review_tables_migrations_registered_zero

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): after upsert(A1) then upsert(A1' same target+reviewer), list_by_target returns exactly 1 row for (target,rev) with state "approved" (no duplicate)
    VERIFY: cargo test -p but-db local_review_assignments_upsert_and_list_by_target
- TC-2 (-> AC-2): set_state(target, "rev", "changes_requested") flips rev to changes_requested and leaves rev2 pending
    VERIFY: cargo test -p but-db local_review_assignments_set_state_targets_one_reviewer
- TC-3 (-> AC-3): insert(C1) persists resolved=false + created_at; list_by_thread + list_by_target both return it; file/line Option<> round-trips (Some and None)
    VERIFY: cargo test -p but-db local_review_comments_insert_and_list
- TC-4 (-> AC-4): set_resolved("t1", true) flips C1a+C1b to resolved=true; C2 (t2) stays resolved=false
    VERIFY: cargo test -p but-db local_review_comments_set_resolved_scopes_to_thread
- TC-5 (-> AC-5): upsert_if_absent(M1) then upsert_if_absent(M1' same target+key, different value) leaves get() returning the FIRST value (write-once per (target,key)); get on a missing key returns None
    VERIFY: cargo test -p but-db local_review_meta_opener_is_write_once_per_target_key
- TC-6 (-> AC-6): a fresh DbHandle running MIGRATIONS has all THREE tables (assignments/comments list_by_target return Ok(empty); meta get returns Ok(None)); all three M entries are SchemaVersion::Zero with the three highest ids
    VERIFY: cargo test -p but-db local_review_tables_migrations_registered_zero

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides:
  - `but_db::LocalReviewAssignment` struct + `DbHandle::local_review_assignments()/…_mut()` Handle pair (list_by_target, upsert idempotent per (target,reviewer_principal), set_state)
  - `but_db::LocalReviewComment` struct + `DbHandle::local_review_comments()/…_mut()` Handle pair (list_by_target, list_by_thread, insert, set_resolved)
  - `but_db::LocalReviewMeta` struct + `DbHandle::local_review_meta()/…_mut()` Handle pair (get(target,key), upsert_if_absent write-once per (target,key)) — the confined cache for the computed agent-PR tag (key="opener_principal")
  - three `SchemaVersion::Zero` migrations registered in MIGRATIONS (the additive schema delta the whole sprint builds on)
consumes:
  - crate::{DbHandle, M, SchemaVersion, Transaction} (the but-db migration + handle primitives)
  - the shipped local_review_verdicts.rs module as the EXACT structural template (mirror it; do not modify it)
boundary_contracts:
  - The three tables are the local_review_verdicts class (disposable, principal-scoped, local), NOT the forge_reviews remote-cache class (runtime DELETE FROM). They carry NO FK (principals live in committed config). The assignment state column stays TEXT (typed at the but-api/but-authz boundary by LPR-002). local_review_meta uses a composite PRIMARY KEY(target, key) (NOT a FK) and a write-once-on-conflict opener insert (R23 narrowing); it caches the computed agent-PR tag, it does NOT store PR-lifecycle truth.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-db/src/table/local_review_assignments.rs (NEW)
  - crates/but-db/src/table/local_review_comments.rs (NEW)
  - crates/but-db/src/table/local_review_meta.rs (NEW)
  - crates/but-db/src/table/mod.rs (MODIFY — APPEND three `pub(crate) mod` lines)
  - crates/but-db/src/lib.rs (MODIFY — APPEND three entries to the MIGRATIONS slice)
  - crates/but-db/tests/db/table/local_review_assignments.rs (NEW)
  - crates/but-db/tests/db/table/local_review_comments.rs (NEW)
  - crates/but-db/tests/db/table/local_review_meta.rs (NEW)
  - crates/but-db/tests/db/table/mod.rs or the tests harness mod file (MODIFY — register the three new test modules, mirroring how local_review_verdicts test is registered)
writeProhibited:
  - crates/but-db/src/table/local_review_verdicts.rs and every other existing table module — CONSUME-only (mirror local_review_verdicts.rs's shape; do not edit it)
  - crates/but-db/src/migration.rs — do NOT change the migration runner / sort / pragma semantics
  - crates/but-api/**, crates/but-authz/**, crates/but/** — out of scope (later LPR tasks)
  - the merge gate (crates/but-api/src/legacy/merge_gate.rs, review_requirement.rs) — untouched
  - any gitbutler-* crate (no new gitbutler-* usage)
  - Any file not in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-db/src/table/local_review_verdicts.rs [1-115] — [PRIMARY PATTERN — copy verbatim] the exact module shape to mirror: the `const M` migration (SchemaVersion::Zero, id 20260619120000), the `#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]` struct, the `impl DbHandle`/`impl Transaction` handle accessors, the `Handle`/`HandleMut` structs holding `conn: &'conn rusqlite::Connection`, and `list_by_target`/`insert`. Your two modules differ only in columns + the upsert/set_state/list_by_thread/set_resolved methods.
2. crates/but-db/src/table/mod.rs [1-13] — where the `pub(crate) mod local_review_verdicts;` line sits (mod.rs:10); add your two `pub(crate) mod` lines beside it.
3. crates/but-db/src/lib.rs [130-143, 160-175] — the `MIGRATIONS` slice (append `table::local_review_assignments::M` + `table::local_review_comments::M` + `table::local_review_meta::M` after `table::local_review_verdicts::M` at lib.rs:142) + the SchemaVersion::Zero doc (lib.rs:167) that authorizes additive tables.
4. crates/but-db/src/migration.rs [35-45, 220-225] — `sort_by_key(|m| m.up_created_at)` (why the new ids must be highest) + `improve_concurrency` setting `PRAGMA foreign_keys = ON` (why you must NOT declare an FK).
5. crates/but-db/src/table/forge_reviews.rs [44-55, 150-155] — the nullable-column convention (`body: Option<String>`, `number: i64`) for `file: Option<String>`/`line: Option<i64>`; ALSO the `DELETE FROM forge_reviews` runtime-clear (forge_reviews.rs:153) — the class your tables must NOT be (no runtime clear).
6. crates/but-db/tests/db/table/local_review_verdicts.rs [whole] — [VERIFIED TEST IDIOM] how the sibling test builds a real migrated `DbHandle`, seeds via the Handle, and asserts on `list_by_target`. Mirror this exact construction — do NOT hand-CREATE-TABLE in the test.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- cargo test -p but-db local_review_assignments_upsert_and_list_by_target   -> Exit 0; second upsert updates not duplicates (exactly 1 row, state "approved")
- cargo test -p but-db local_review_assignments_set_state_targets_one_reviewer   -> Exit 0; set_state flips only the (target,reviewer) pair
- cargo test -p but-db local_review_comments_insert_and_list   -> Exit 0; resolved=false persisted; file/line Some+None round-trip; both list methods return it
- cargo test -p but-db local_review_comments_set_resolved_scopes_to_thread   -> Exit 0; whole thread flips; other thread untouched
- cargo test -p but-db local_review_tables_migrations_registered_zero   -> Exit 0; tables exist on a fresh DbHandle; both M are SchemaVersion::Zero with the highest ids
- cargo check -p but-db --all-targets   -> Exit 0
- cargo clippy -p but-db --all-targets   -> Exit 0
- cargo fmt --check   -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references:
  - .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md (§A.1 local_review_assignments SQL+struct; §A.2 local_review_comments SQL+struct; the migration-registration recipe)
  - crates/but-db/src/table/local_review_verdicts.rs (the structural template — id 20260619120000, SchemaVersion::Zero)
sql:
  - |
    CREATE TABLE `local_review_assignments`(
        `id` TEXT NOT NULL PRIMARY KEY,
        `target` TEXT NOT NULL,
        `reviewer_principal` TEXT NOT NULL,
        `state` TEXT NOT NULL,           -- 'pending' | 'approved' | 'changes_requested'
        `assigned_at` TIMESTAMP NOT NULL
    );
    CREATE INDEX `idx_local_review_assignments_target_reviewer`
    ON `local_review_assignments`(`target`, `reviewer_principal`);
  - |
    CREATE TABLE `local_review_comments`(
        `id` TEXT NOT NULL PRIMARY KEY,
        `target` TEXT NOT NULL,
        `author_principal` TEXT NOT NULL,
        `body` TEXT NOT NULL,
        `file` TEXT,                     -- nullable: file-scoped vs PR-level comment
        `line` INTEGER,                  -- nullable: line within `file`
        `thread_id` TEXT NOT NULL,       -- groups a comment thread
        `resolved` BOOL NOT NULL,
        `created_at` TIMESTAMP NOT NULL
    );
    CREATE INDEX `idx_local_review_comments_target_thread`
    ON `local_review_comments`(`target`, `thread_id`);
  - |
    CREATE TABLE `local_review_meta`(
        `target` TEXT NOT NULL,
        `key` TEXT NOT NULL,             -- e.g. 'opener_principal'
        `value` TEXT NOT NULL,
        `created_at` TIMESTAMP NOT NULL,
        PRIMARY KEY (`target`, `key`)    -- UNIQUE(target, key): one row per metadata key per target
    );
    -- NO secondary index, NO `id` column: the composite PRIMARY KEY is the access path.
notes:
  - Migration ids (tech-delta §A): local_review_assignments=20260621120000, local_review_comments=20260621120100, local_review_meta=20260621120200 — all three higher than the shipped local_review_verdicts id 20260619120000, so they sort last (migration.rs:39).
  - upsert idempotency: the cleanest impl is `INSERT INTO … ON CONFLICT(target, reviewer_principal) DO UPDATE SET state=excluded.state, assigned_at=excluded.assigned_at` (declare a UNIQUE on (target,reviewer_principal) instead of / in addition to the plain index) OR a `DELETE WHERE target=?1 AND reviewer_principal=?2` then INSERT inside one Transaction. The index `(target, reviewer_principal)` serves both the upsert and `list_by_target`.
  - meta write-once: `local_review_meta::upsert_if_absent` is `INSERT INTO local_review_meta(...) VALUES(...) ON CONFLICT(target, key) DO NOTHING` — the opener row is write-once per target (a later caller cannot overwrite a recorded opener through the governed path). This is the R23 narrowing; AC-5 proves the second write is a no-op. `get(target, key)` is a `SELECT … WHERE target=?1 AND key=?2` returning `Option<LocalReviewMeta>`.
  - The Handle/HandleMut split mirrors local_review_verdicts.rs:54-109 exactly — reads on the immutable Handle (list_by_target, list_by_thread), writes on the Mut handle (upsert, set_state, insert, set_resolved), with a `to_ref()` on the Mut handle.
pattern: three new but-db table modules mirroring local_review_verdicts.rs — each a SchemaVersion::Zero migration + a serde struct + a Handle/HandleMut pair — registered in mod.rs + MIGRATIONS, proven by a real-migrated-DbHandle round-trip test; local_review_meta uses a composite PRIMARY KEY(target, key) + a write-once-on-conflict opener insert
pattern_source: crates/but-db/src/table/local_review_verdicts.rs (the verbatim structural template); crates/but-db/tests/db/table/local_review_verdicts.rs (the verified test idiom)
anti_pattern: bumping SchemaVersion above Zero for an additive table; declaring an FK on principal/target/key (PRAGMA foreign_keys=ON would enforce it; the meta composite PRIMARY KEY is allowed); hand-CREATE-TABLE in the test instead of running the registered migration (hides a missing MIGRATIONS entry); a non-idempotent assignment upsert that duplicates per (target,reviewer); a DO-UPDATE (overwriting) local_review_meta opener write instead of DO-NOTHING write-once (AC-5 catches it — breaks the R23 narrowing); an `id`/auto-increment column on local_review_meta instead of the composite PRIMARY KEY; a FOURTH local_pull_requests table (the PR lifecycle is DERIVED); editing local_review_verdicts.rs or any existing migration

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agent: implementer=rust-implementer | reviewer=rust-reviewer
rationale: Three additive but-db table modules + migrations + Handle pairs, mirroring the shipped local_review_verdicts.rs precisely. Requires faithful copy of the migration-registration recipe (SchemaVersion::Zero, monotonic ids, MIGRATIONS append), the no-FK posture, an idempotent assignment upsert keyed on a declared index, the write-once-per-(target,key) opener insert on local_review_meta (the R23 narrowing), and a real-migrated-DbHandle integration test (not a hand-CREATE-TABLE). These are rust-implementer competencies; rust-reviewer validates SchemaVersion::Zero on all three, the absent FK, the idempotent assignment upsert, the write-once meta opener (DO NOTHING not DO UPDATE), and that the test runs the real registered migration.
coding_standards: crates/AGENTS.md (keep types/helpers in the crate that owns the concept — these are but-db tables; solve the present problem directly — no speculative columns); RULES.md (use gix/but-* idioms; NEVER std::env::temp_dir() in tests — use but-db's own test harness); brain/docs/rust/ (ownership-borrowing.md Option<T> for nullable columns; testing.md #[test] + assert_eq! hand-assertion)

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: Sprint 01b (the shipped local_review_verdicts.rs table this mirrors)
Blocks:     LPR-002 (AssignmentState typed over the TEXT state column), LPR-003 (request/assign write to local_review_assignments + the local_review_meta opener row), LPR-004 (comment writes to local_review_comments), LPR-005 (derived PR view reads all three; the agent tag reads the local_review_meta opener row + the opener's declared kind)
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "LPR-001",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "lpr_db": {
      "description": "A fresh, REAL, migrated but-db DbHandle constructed the way crates/but-db/tests/db/table/local_review_verdicts.rs constructs it (the migrations in MIGRATIONS actually run — NOT a hand-written CREATE TABLE). Seed rows ONLY via the real Handle methods (upsert/insert/set_state/set_resolved). Never inject rows with a raw conn.execute in the test.",
      "seed_method": "public_api",
      "records": [
        "construct a real migrated DbHandle (the sibling local_review_verdicts test idiom);",
        "db.local_review_assignments_mut().upsert(LocalReviewAssignment{ id, target: \"refs/heads/feat\", reviewer_principal: \"rev\", state: \"pending\", assigned_at: t0 });",
        "db.local_review_comments_mut().insert(LocalReviewComment{ id, target: \"refs/heads/feat\", author_principal: \"rev\", body: \"fix this\", file: Some(\"f.rs\"), line: Some(12), thread_id: \"t1\", resolved: false, created_at: t0 });",
        "db.local_review_meta_mut().upsert_if_absent(LocalReviewMeta{ target: \"refs/heads/feat\", key: \"opener_principal\", value: \"agent-A\", created_at: t0 });"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN lpr_db with assignment A1{target=refs/heads/feat, reviewer_principal=rev, state=pending} WHEN upsert(A1) then upsert(A1' same target+reviewer, state=approved) then list_by_target(refs/heads/feat) THEN exactly ONE row exists for (refs/heads/feat, rev) with state==approved (the second upsert UPDATED, did not duplicate) and the row's fields round-trip",
      "verify": "cargo test -p but-db local_review_assignments_upsert_and_list_by_target",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-db DbHandle with the real registered migration applied",
        "negative_control": {
          "would_fail_if": [
            "the second upsert INSERTED a duplicate (list_by_target returns 2 rows for (target,rev)) — a non-idempotent upsert",
            "the upsert did not key on (target, reviewer_principal) — the state would not update or the wrong row updates",
            "the table was not registered in MIGRATIONS — the query errors (no such table) rather than returning rows",
            "a stub returned an empty list — the seeded assignment is absent"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_db",
            "action": {
              "actor": "ci",
              "steps": [
                "db.local_review_assignments_mut().upsert(A1 state=pending)",
                "db.local_review_assignments_mut().upsert(A1' same target+reviewer, state=approved, assigned_at=t1)",
                "rows = db.local_review_assignments().list_by_target(\"refs/heads/feat\")",
                "assert rows.len()==1 && rows[0].state==\"approved\" && rows[0].reviewer_principal==\"rev\""
              ]
            },
            "end_state": {
              "must_observe": [
                "list_by_target returns exactly 1 row for (refs/heads/feat, rev)",
                "that row's state == \"approved\" (the second upsert updated it)",
                "target/reviewer_principal/assigned_at round-trip on the returned row"
              ],
              "must_not_observe": [
                "2 rows for (refs/heads/feat, rev) (duplicate insert — non-idempotent upsert)",
                "state still \"pending\" (the update did not apply)",
                "0 rows (table not registered / stub empty result)"
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
      "description": "GIVEN lpr_db with A_rev{reviewer=rev,state=pending} and A_rev2{reviewer=rev2,state=pending} on the same target WHEN set_state(target, rev, changes_requested) THEN rev's state==changes_requested and rev2's state is STILL pending (set_state scopes to the (target,reviewer) pair)",
      "verify": "cargo test -p but-db local_review_assignments_set_state_targets_one_reviewer",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-db DbHandle + assignment Handle",
        "negative_control": {
          "would_fail_if": [
            "set_state flipped EVERY assignment on the target (rev2 would also become changes_requested) — an unscoped UPDATE WHERE target=?",
            "set_state matched the wrong column — rev would not change",
            "a stub no-op left rev pending"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_db",
            "action": {
              "actor": "ci",
              "steps": [
                "seed A_rev (rev, pending) and A_rev2 (rev2, pending) on refs/heads/feat",
                "db.local_review_assignments_mut().set_state(\"refs/heads/feat\", \"rev\", \"changes_requested\")",
                "rows = list_by_target(\"refs/heads/feat\")"
              ]
            },
            "end_state": {
              "must_observe": [
                "the rev row state == \"changes_requested\"",
                "the rev2 row state == \"pending\" (untouched)"
              ],
              "must_not_observe": [
                "the rev2 row state == \"changes_requested\" (unscoped update flipped the whole target)",
                "the rev row state still \"pending\" (no-op)"
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
      "description": "GIVEN lpr_db WHEN insert(C1{file=Some(f.rs),line=Some(12),thread=t1,resolved=false}) then list_by_thread(target,t1) and list_by_target(target) THEN both lists return C1 with resolved==false, file==Some(f.rs), line==Some(12), created_at==t0; a PR-level comment (file=None,line=None) round-trips both Nones",
      "verify": "cargo test -p but-db local_review_comments_insert_and_list",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-db DbHandle + comment Handle",
        "negative_control": {
          "would_fail_if": [
            "resolved defaulted to true on insert (the row would come back resolved) — the writer must persist false",
            "file/line were stored non-nullable — the file=None,line=None PR-level comment would error or coerce to a sentinel rather than round-trip None",
            "list_by_thread ignored the thread_id and returned all comments — a missing (target,thread_id) filter",
            "a stub returned an empty list — C1 absent"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_db",
            "action": {
              "actor": "ci",
              "steps": [
                "db.local_review_comments_mut().insert(C1 with file=Some(f.rs), line=Some(12), thread=t1, resolved=false)",
                "db.local_review_comments_mut().insert(C_prlevel with file=None, line=None, thread=t2, resolved=false)",
                "by_thread = list_by_thread(\"refs/heads/feat\", \"t1\"); by_target = list_by_target(\"refs/heads/feat\")"
              ]
            },
            "end_state": {
              "must_observe": [
                "by_thread returns C1 with resolved==false, file==Some(\"f.rs\"), line==Some(12), created_at==t0",
                "by_target returns both C1 and C_prlevel",
                "C_prlevel round-trips file==None && line==None"
              ],
              "must_not_observe": [
                "C1 returning resolved==true (insert defaulted resolved wrong)",
                "by_thread returning C_prlevel (thread_id filter ignored)",
                "0 comments from either list (stub empty result)"
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
      "description": "GIVEN lpr_db with comments C1a,C1b on thread t1 and C2 on thread t2 (same target) WHEN set_resolved(t1, true) THEN every t1 comment is resolved==true and C2 (t2) is STILL resolved==false (set_resolved scopes to thread_id)",
      "verify": "cargo test -p but-db local_review_comments_set_resolved_scopes_to_thread",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-db DbHandle + comment Handle",
        "negative_control": {
          "would_fail_if": [
            "set_resolved flipped EVERY comment on the target (C2 would also resolve) — an unscoped UPDATE",
            "set_resolved flipped only the first row of the thread (C1b stays unresolved) — a LIMIT 1 bug",
            "a stub no-op left C1a/C1b unresolved"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_db",
            "action": {
              "actor": "ci",
              "steps": [
                "seed C1a, C1b on thread t1 and C2 on thread t2",
                "db.local_review_comments_mut().set_resolved(\"t1\", true)",
                "rows = list_by_target(\"refs/heads/feat\")"
              ]
            },
            "end_state": {
              "must_observe": [
                "both t1 comments (C1a, C1b) have resolved==true",
                "C2 (thread t2) has resolved==false"
              ],
              "must_not_observe": [
                "C2 resolved==true (unscoped flip)",
                "C1b resolved==false (only first row flipped)"
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
      "description": "GIVEN lpr_db with meta row M1{target=refs/heads/feat, key=opener_principal, value=agent-A} WHEN upsert_if_absent(M1) then upsert_if_absent(M1' same target+key, value=impostor) then get(refs/heads/feat, opener_principal) THEN get returns ONE row whose value==agent-A (the second write was a NO-OP — ON CONFLICT(target,key) DO NOTHING makes the opener write-once; the R23 control-path narrowing) and get on a missing (target,key) returns None",
      "verify": "cargo test -p but-db local_review_meta_opener_is_write_once_per_target_key",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-db DbHandle + the meta Handle (write-once-on-conflict per (target,key))",
        "negative_control": {
          "would_fail_if": [
            "the second upsert_if_absent OVERWROTE the opener (get returns value==impostor) — a DO UPDATE impl instead of DO NOTHING (breaks the R23 write-once narrowing)",
            "the write keyed on something other than (target, key) — the second row inserts as a duplicate or the wrong row is read",
            "get on a missing key returned an error or a fabricated row instead of None",
            "a stub returned None for the seeded opener"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_db",
            "action": {
              "actor": "ci",
              "steps": [
                "db.local_review_meta_mut().upsert_if_absent(M1 value=agent-A)",
                "db.local_review_meta_mut().upsert_if_absent(M1' same target+key, value=impostor)",
                "row = db.local_review_meta().get(\"refs/heads/feat\", \"opener_principal\")",
                "assert row.is_some() && row.value == \"agent-A\"",
                "assert db.local_review_meta().get(\"refs/heads/feat\", \"missing\").is_none()"
              ]
            },
            "end_state": {
              "must_observe": [
                "get returns one row whose value == agent-A (the FIRST write — opener is write-once)",
                "the second upsert_if_absent was a no-op (value not overwritten to impostor)",
                "get on a missing (target,key) returns None"
              ],
              "must_not_observe": [
                "value == impostor (a DO UPDATE overwrote the opener — R23 narrowing broken)",
                "a duplicate row for (target, opener_principal)",
                "None for the seeded opener (stub) / a fabricated row for the missing key"
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
      "description": "GIVEN a fresh DbHandle running MIGRATIONS WHEN it is constructed and the three new tables are queried on an empty store (list_by_target for assignments/comments, get for meta) THEN all queries return Ok(empty/None) (the tables EXIST — the registered migrations ran) and all three M entries are SchemaVersion::Zero with the three highest ids (20260621120000 / 20260621120100 / 20260621120200)",
      "verify": "cargo test -p but-db local_review_tables_migrations_registered_zero",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-db DbHandle::new running the real registered MIGRATIONS",
        "negative_control": {
          "would_fail_if": [
            "any of the three modules was not appended to MIGRATIONS — its query errors (no such table) instead of Ok(empty/None)",
            "a migration used SchemaVersion above Zero — the structural assertion on schema_version fails (additive tables must be Zero)",
            "the ids were not the highest — sort order would place them mid-list (a structural id-ordering assertion catches it)"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_db",
            "action": {
              "actor": "ci",
              "steps": [
                "construct a fresh DbHandle (runs MIGRATIONS)",
                "assert local_review_assignments().list_by_target(\"x\") == Ok(empty), local_review_comments().list_by_target(\"x\") == Ok(empty), and local_review_meta().get(\"x\", \"k\") == Ok(None)",
                "assert local_review_assignments::M[0].schema_version, local_review_comments::M[0].schema_version, and local_review_meta::M[0].schema_version are all SchemaVersion::Zero",
                "assert the three new ids are the highest up_created_at in MIGRATIONS"
              ]
            },
            "end_state": {
              "must_observe": [
                "all three queries return Ok(empty/None) (the tables exist on a fresh handle)",
                "all three migrations are SchemaVersion::Zero",
                "20260621120000, 20260621120100, and 20260621120200 are the three highest migration ids"
              ],
              "must_not_observe": [
                "a 'no such table' error from any of the three queries (a module not registered in MIGRATIONS)",
                "a non-Zero SchemaVersion on any migration"
              ]
            }
          }
        ]
      }
    },
    { "id": "TC-1", "type": "test_criterion", "description": "after upsert(A1) then upsert(A1' same target+reviewer state=approved), list_by_target returns exactly 1 row for (target,rev) with state approved (no duplicate)", "verify": "cargo test -p but-db local_review_assignments_upsert_and_list_by_target", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "set_state(target, rev, changes_requested) flips rev and leaves rev2 pending", "verify": "cargo test -p but-db local_review_assignments_set_state_targets_one_reviewer", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "insert(C1) persists resolved=false + created_at; list_by_thread + list_by_target return it; file/line Some+None round-trip", "verify": "cargo test -p but-db local_review_comments_insert_and_list", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "set_resolved(t1,true) flips C1a+C1b; C2 (t2) stays unresolved", "verify": "cargo test -p but-db local_review_comments_set_resolved_scopes_to_thread", "maps_to_ac": "AC-4" },
    { "id": "TC-5", "type": "test_criterion", "description": "upsert_if_absent(M1) then upsert_if_absent(M1' same target+key, different value) leaves get() returning the FIRST value (write-once per (target,key)); get on a missing key returns None", "verify": "cargo test -p but-db local_review_meta_opener_is_write_once_per_target_key", "maps_to_ac": "AC-5" },
    { "id": "TC-6", "type": "test_criterion", "description": "a fresh DbHandle running MIGRATIONS has all THREE tables (assignments/comments Ok empty; meta Ok None); all three M are SchemaVersion::Zero with the three highest ids", "verify": "cargo test -p but-db local_review_tables_migrations_registered_zero", "maps_to_ac": "AC-6" }
  ]
}
-->
