# GATES-002: Local review record — `but-db` `local_review_verdicts` table (head-pinned, principal-attributed, queryable by target)

## What this does

Adds the single new persistent local-state addition for Sprint 01b: a NEW `but-db` table `local_review_verdicts` (fields `id`, `target`, `principal_id`, `verdict`, `head_oid`, `created_at`) following GitButler's exact inline-SQL migration + table-module + handle pattern (`workspace_rules`/`forge_reviews`), with insert + query-by-target handle methods. It is the LOCAL review store the merge gate (GATES-003/005) reads to decide "≥1 approving review, distinct from author, from each required group, valid at the current head" without a remote forge. `head_oid` is stored verbatim so an approval at an old head is distinguishable from one at the new head.

## Why

Sprint 01b · PRD UC-GATES-02 (review record) · capabilities CAP-AUTHZ-01, CAP-CONFIG-01. The merge half of the walking skeleton needs a head-pinned, principal-attributed approvals store; this is it.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-db local_review_verdicts::insert_and_query_by_target` (integration, real in-memory sqlite). Full gate set in the spec below.

## Scope

- `crates/but-db/src/table/local_review_verdicts.rs` (NEW) — the table module: `M` migration const, `LocalReviewVerdict` struct, `DbHandle`/`Transaction` accessors, `LocalReviewVerdictsHandle{,Mut}` with `insert` + `list_by_target`
- `crates/but-db/src/table/mod.rs` (MODIFY) — add `pub(crate) mod local_review_verdicts;`
- `crates/but-db/src/lib.rs` (MODIFY) — re-export `LocalReviewVerdict` + add `table::local_review_verdicts::M` to `MIGRATIONS`
- `crates/but-db/tests/db/table/local_review_verdicts.rs` (NEW) — integration tests against `in_memory_db()`
- `crates/but-db/tests/db/table/mod.rs` (MODIFY) — add `mod local_review_verdicts;`

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: GATES-002 - Local review record: but-db local_review_verdicts table (head-pinned, principal-attributed, queryable by target)
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Complete
PRIORITY:   P0
EFFORT:     M  (150 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-GATES-02
CAPABILITIES: CAP-AUTHZ-01, CAP-CONFIG-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-db local_review_verdicts
  check: cargo check -p but-db --all-targets
  lint:  cargo clippy -p but-db --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Integration tests are green against a REAL but-db DbHandle (in-memory sqlite, all migrations applied): inserting a verdict makes it queryable by target with every field byte-intact; a verdict at head_oid=H1 and one at head_oid=H2 are returned as two DISTINCT rows (head-pinning observable, not normalized away); a target with no verdicts returns an empty Vec (the start signature) — not an Err, and another target's rows do not leak; two verdicts from different principals on one target are both returned ordered by created_at so distinct-from-author is computable by the consumer. The migration is registered in MIGRATIONS (timestamp greater than every existing one) and the row struct is re-exported from but_db; the table is NOT homed in the disposable forge_reviews cache.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST add local_review_verdicts as a NEW but-db table module crates/but-db/src/table/local_review_verdicts.rs following the EXACT existing pattern: a `pub(crate) const M: &[M<'static>]` with a single `M::up(<sortable_ts>, SchemaVersion::Zero, "CREATE TABLE ...")` migration (mirror workspace_rules.rs:8-19), a `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]` row struct, and `impl DbHandle` + `impl<'conn> Transaction<'conn>` accessor methods returning `*Handle`/`*HandleMut` (mirror workspace_rules.rs:32-58).
- [MUST] MUST register the new module in THREE places: declare `pub(crate) mod local_review_verdicts;` in crates/but-db/src/table/mod.rs; add `table::local_review_verdicts::M` to the MIGRATIONS slice in crates/but-db/src/lib.rs:129-141; re-export the row struct in the `pub use table::{ ... }` block at crates/but-db/src/lib.rs:72-83.
- [MUST] MUST store head_oid VERBATIM as the string the caller supplies (a gix::ObjectId/sha hex string) — the column round-trips byte-for-byte; a verdict at H1 and one at H2 are DISTINCT stored rows (the consumer's @head check depends on this; do NOT normalize, truncate, or canonicalize the oid).
- [MUST] MUST provide a query method `list_by_target(&self, target: &str) -> rusqlite::Result<Vec<LocalReviewVerdict>>` returning ALL verdicts for a target with every field intact, ordered by created_at; an empty target returns an empty Vec (NOT an error).
- [MUST] MUST pick the migration created_at_for_sorting STRICTLY GREATER than every existing migration timestamp in MIGRATIONS (largest today is forge_reviews 20260618093000) so it sorts last and applies cleanly on existing DBs (ordering by numeric key — migration.rs:38-41).
- [NEVER] NEVER home this in but-forge-storage (holds only forge_settings.json) NOR in the forge_reviews table — forge_reviews is a DISPOSABLE remote-PR cache wiped on every sync via the runtime DELETE FROM forge_reviews at forge_reviews.rs:153 and carries no principal_id/verdict.
- [NEVER] NEVER add an HMAC, signature, hash-chain, or integrity field — R6 hardening is DEFERRED; the schema must NOT be presented as tamper-proof (03-data-schema.md R6 caveat).
- [NEVER] NEVER write a test asserting a forged row (a direct DB INSERT bypassing the governed `but review` action) is rejected/detected — the direct-DB-write forgeability is an accepted leak (R6); such a test encodes a FALSE guarantee.
- [NEVER] NEVER use std::env::temp_dir() or a real on-disk path for tests — use the crate's in_memory_db() helper (tests/db/table/mod.rs:14-17, DbHandle::new_at_path(":memory:")).
- [STRICTLY] STRICTLY mirror the WorkspaceRulesHandle/WorkspaceRulesHandleMut borrow shape (a read handle borrowing &rusqlite::Connection, a mut handle for writes) — the simpler pattern (workspace_rules.rs:52-58), NOT the savepoint-based forge_reviews_mut pattern, unless a reviewer requires transactional batching.
- [STRICTLY] STRICTLY give the row a stable id (TEXT PRIMARY KEY, caller-supplied uuid/string like workspace_rules) and store created_at as chrono::NaiveDateTime (TIMESTAMP NOT NULL) consistent with workspace_rules/forge_reviews.
- [STRICTLY] STRICTLY keep verdict as a TEXT column carrying "approved" or "commented" (the two values 03-data-schema.md names); do NOT add a typed enum to but-db (the consumer interprets the string); store the value verbatim.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: an inserted verdict is queryable by target with all fields byte-intact (head_oid verbatim)
- [ ] AC-2: head-pinning observable — verdicts at H1 and H2 are two distinct stored rows, not collapsed/normalized
- [ ] AC-3: a target with no verdicts returns an empty Vec (not Err); the target filter is honored
- [ ] AC-4: two distinct principals' verdicts on one target are both returned with principal_id intact, ordered by created_at
- [ ] All verification gates pass; only write_allowed files modified (git diff --name-only)

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: An inserted verdict is queryable by target with all fields intact [PRIMARY]
  GIVEN: fixture `empty_db` (a real in-memory but-db DbHandle, migrations applied, no verdict rows)
  WHEN:  a single approving verdict {id:"v1", target:"refs/heads/feat", principal_id:"rust-reviewer", verdict:"approved", head_oid:"aaaa…aaaa" (40-char), created_at:<fixed>} is inserted via the mut handle, then list_by_target("refs/heads/feat") is called
  THEN:  exactly one row is returned, byte-equal to the inserted verdict in EVERY field; head_oid round-trips verbatim
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-db (in-memory sqlite via DbHandle::new_at_path(":memory:"))
  VERIFY: cargo test -p but-db local_review_verdicts::insert_and_query_by_target
  SCENARIO (tier=visible, test_tier=integration):
    NEGATIVE_CONTROL would fail if: the insert is a no-op stub so list returns empty; list returns the row but drops/blanks head_oid or principal_id; head_oid is normalized/truncated; the migration is not registered so the insert errors 'no such table'
    EVIDENCE: db_query (required_capture=True)
    case[0] (db_client): insert v1 then list_by_target("refs/heads/feat")
      MUST_OBSERVE:     ['rows.len() == 1', 'rows[0].principal_id == "rust-reviewer"', 'rows[0].verdict == "approved"', 'rows[0].head_oid == "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"', 'rows[0] == v1 (full struct equality)']
      MUST_NOT_OBSERVE: ['rows.is_empty()', 'rows[0].head_oid != the inserted sha', "a panic / 'no such table' error"]

AC-2: Head-pinning is observable — verdicts at H1 and H2 are distinct stored rows
  GIVEN: fixture `same_target_two_heads` (two approving verdicts on the same target by the same principal, one at head_oid H1, one at head_oid H2)
  WHEN:  list_by_target("refs/heads/feat") is called
  THEN:  TWO rows are returned, one head_oid==H1 and one head_oid==H2; not collapsed/deduplicated/normalized
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-db (in-memory sqlite)
  VERIFY: cargo test -p but-db local_review_verdicts::head_pinning_distinguishes_heads
  SCENARIO (tier=visible, test_tier=integration):
    NEGATIVE_CONTROL would fail if: head_oid normalized so H1/H2 collapse to one observable row; the query dedupes by (target, principal_id) dropping the second head; head_oid stored but not returned
    EVIDENCE: db_query (required_capture=True)
    case[0] (db_client): list_by_target("refs/heads/feat"); collect head_oid set
      MUST_OBSERVE:     ['rows.len() == 2', 'head_oid set == {"1111111111111111111111111111111111111111","2222222222222222222222222222222222222222"}', 'a row with head_oid==H1 AND a row with head_oid==H2 both present']
      MUST_NOT_OBSERVE: ['rows.len() == 1 (collapsed)', 'both rows same head_oid (normalized)', 'the H2 row missing']

AC-3: Querying a target with no verdicts returns empty (start signature)
  GIVEN: fixture `empty_db`; and after inserting a verdict on refs/heads/other, querying an unrelated target
  WHEN:  list_by_target("refs/heads/feat") against an empty table, and again while a different target has rows
  THEN:  an empty Vec in both cases (NOT an error, NOT another target's rows)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-db (in-memory sqlite)
  VERIFY: cargo test -p but-db local_review_verdicts::empty_target_returns_empty
  SCENARIO (tier=holdout, test_tier=integration):
    NEGATIVE_CONTROL would fail if: list errors rather than returning empty; list ignores the target filter; an empty query panics
    EVIDENCE: db_query (required_capture=True)
    case[0] (db_client): query empty; insert refs/heads/other verdict; re-query refs/heads/feat
      MUST_OBSERVE:     ['the first query returns Ok(vec![]) (empty, not Err)', 'feat_rows.is_empty() (the other verdict does NOT leak)']
      MUST_NOT_OBSERVE: ['an Err / panic on the empty query', 'feat_rows contains the refs/heads/other verdict']

AC-4: Two verdicts from different principals on one target are both returned (distinct-from-author computable)
  GIVEN: fixture `two_principals_one_target_at_head` (two approving verdicts on refs/heads/feat at the same head, from rust-reviewer and justin)
  WHEN:  list_by_target("refs/heads/feat") is called
  THEN:  BOTH rows returned with distinct principal_id intact, ordered by created_at
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-db (in-memory sqlite)
  VERIFY: cargo test -p but-db local_review_verdicts::distinct_principals_both_returned
  SCENARIO (tier=holdout, test_tier=integration):
    NEGATIVE_CONTROL would fail if: the query dedupes by target returning one principal; principal_id blanked/collapsed; unordered so the test can't read both rows
    EVIDENCE: db_query (required_capture=True)
    case[0] (db_client): list_by_target("refs/heads/feat"); collect principal_id
      MUST_OBSERVE:     ['rows.len() == 2', 'principal_id set == {"rust-reviewer","justin"}', 'ordered by created_at (rust-reviewer ts=1000000 before justin ts=1000001)', 'every row.verdict == "approved"']
      MUST_NOT_OBSERVE: ['rows.len() == 1 (deduped)', 'both rows same principal_id (collapsed)', 'a missing principal_id field']

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, happy_path): Inserting a verdict makes it queryable by target with all fields byte-intact
    VERIFY: cargo test -p but-db local_review_verdicts::insert_and_query_by_target
- TC-2 (-> AC-2, edge): head_oid verbatim — H1 and H2 are two distinct rows (underpins the merge gate @head check, T-GATES-011)
    VERIFY: cargo test -p but-db local_review_verdicts::head_pinning_distinguishes_heads
- TC-3 (-> AC-3, edge): A target with no verdicts returns an empty Vec; target filter honored (T-GATES-009 start signature)
    VERIFY: cargo test -p but-db local_review_verdicts::empty_target_returns_empty
- TC-4 (-> AC-4, happy_path): Two distinct principals both returned, ordered by created_at (T-GATES-010/012/014 computable)
    VERIFY: cargo test -p but-db local_review_verdicts::distinct_principals_both_returned
- TC-5 (-> AC-1, structural): Migration registered in MIGRATIONS with greatest timestamp + struct re-exported; not homed in forge_reviews
    VERIFY: cargo test -p but-db local_review_verdicts

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01, CAP-CONFIG-01
provides: a NEW but-db table local_review_verdicts {id,target,principal_id,verdict,head_oid,created_at} registered in MIGRATIONS; a LocalReviewVerdictsHandle/HandleMut on DbHandle + Transaction (insert; list_by_target); a public LocalReviewVerdict struct re-exported from but_db
consumes: but_db::DbHandle (project-local handle via ctx.db.get_cache_mut()); but_db::M / SchemaVersion; rusqlite
boundary_contracts:
  - CAP-AUTHZ-01 + CAP-CONFIG-01: provides the LOCAL review store the merge-gate evaluation (GATES-003/005) reads to decide a distinct approving review from each required group valid AT THE CURRENT HEAD without a remote forge; head_oid is recorded so a stale (old-head) approval is distinguishable.
  - R6 honesty: the table is NOT integrity-protected — a direct DB write can forge an approving row; an accepted-leak documented here and NEVER asserted-against in tests.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-db/src/table/local_review_verdicts.rs (NEW) — the table module (M, struct, accessors, Handle/HandleMut with insert + list_by_target)
  - crates/but-db/src/table/mod.rs (MODIFY) — add `pub(crate) mod local_review_verdicts;`
  - crates/but-db/src/lib.rs (MODIFY) — add the re-export in `pub use table::{ ... }` and `table::local_review_verdicts::M` to MIGRATIONS
  - crates/but-db/tests/db/table/local_review_verdicts.rs (NEW) — integration tests against in_memory_db()
  - crates/but-db/tests/db/table/mod.rs (MODIFY) — add `mod local_review_verdicts;`
writeProhibited:
  - crates/but-authz/** — consume only; this task adds no authz logic
  - crates/but-db/src/table/forge_reviews.rs — do NOT extend the disposable forge-review cache; the new table is independent (wiped on sync at :153)
  - crates/but-forge-storage/** — the review store is NOT a forge-settings concern
  - crates/but-api/** and crates/but/** — the WRITE path (`but review approve`) is GATES-004's job; this task only provides the table + handle
  - Any file not listed in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-db/src/table/workspace_rules.rs (lines 8-144)
   Focus: PRIMARY PATTERN — copy this shape: the `pub(crate) const M` single-migration const (8-19), the derive row struct (22-30), the impl DbHandle + impl<'conn> Transaction<'conn> accessors (32-50), the simple *Handle/*HandleMut borrowing &rusqlite::Connection (52-58), and the get/list/insert SQL bodies (60-144). Mirror the simple (non-savepoint) mut pattern.
2. crates/but-db/src/table/forge_reviews.rs (lines 7-42, 144-191)
   Focus: WHY NOT HERE + multi-column INSERT/query shape — the M const with a SECOND ALTER migration (7-42) and the runtime DELETE FROM forge_reviews at :153 that wipes this table on sync (the reason local_review_verdicts must NOT live here). Use the multi-column INSERT ... VALUES (?1,...) + query_map row-mapping idiom for the new table.
3. crates/but-db/src/table/mod.rs (1-12)
   Focus: REGISTRATION 1/3 — add `pub(crate) mod local_review_verdicts;` after forge_reviews.
4. crates/but-db/src/lib.rs (72-83, 129-141)
   Focus: REGISTRATION 2/3 + 3/3 — add LocalReviewVerdict to the `pub use table::{ ... }` re-export, and `table::local_review_verdicts::M` to MIGRATIONS (pick a timestamp > 20260618093000).
5. crates/but-db/src/migration.rs (33-107, 178-188)
   Focus: migrations sorted by up_created_at, applied in order skipping applied ones; M::up(created_at, schema_version, sql) constructor — confirms a new M with a greater timestamp applies cleanly.
6. crates/but-db/tests/db/table/mod.rs (1-17) + tests/db/table/workspace_rules.rs (1-178)
   Focus: TEST HARNESS + SHAPE — declare `mod local_review_verdicts;`; use in_memory_db() (real migrated sqlite); mirror insert_and_get/list_empty/list_multiple test bodies; use a fixed chrono timestamp so rows are deterministic. NO temp_dir, no mocks.
7. .spec/prds/governance/10-technical-requirements/03-data-schema.md (79-94)
   Focus: THE SCHEMA + R6 caveat — the field table, the head_oid load-bearing note, the 'NOT integrity-protected' R6 caveat (do NOT add HMAC), and the 'follows the workspace_rules/forge_reviews migration pattern' direction.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- Table integration tests pass against real sqlite: `cargo test -p but-db local_review_verdicts`  -> Exit 0; AC-1..4 green
- Crate compiles incl. test targets: `cargo check -p but-db --all-targets`  -> Exit 0
- Migration registered in MIGRATIONS: `grep -n 'local_review_verdicts::M' crates/but-db/src/lib.rs`  -> one match in the MIGRATIONS slice
- Row struct re-exported from but_db: `grep -n 'local_review_verdicts::LocalReviewVerdict' crates/but-db/src/lib.rs`  -> one match in the pub use block
- Module registered in table mod + test mod: `grep -rEn 'mod local_review_verdicts' crates/but-db/src/table/mod.rs crates/but-db/tests/db/table/mod.rs`  -> one match in each
- No integrity/HMAC field (R6 deferred): `! grep -rEin 'hmac|signature|ed25519|sign(ed|ature)|tamper' crates/but-db/src/table/local_review_verdicts.rs`  -> No matches
- Not homed in the disposable forge cache: `! grep -rEn 'local_review_verdicts' crates/but-db/src/table/forge_reviews.rs`  -> No matches
- Clippy clean + fmt: `cargo clippy -p but-db --all-targets && cargo fmt --check`  -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references: crates/but-db/src/table/workspace_rules.rs:8 (the canonical simple table module); crates/but-db/src/lib.rs:129 (MIGRATIONS) + :72 (pub use re-export); crates/but-db/src/table/forge_reviews.rs:153 (the DELETE-on-sync wipe disqualifying forge_reviews); 03-data-schema.md:79 (field table + R6 caveat)
notes:
  - Accessed by GATES-003/005 (merge gate) and GATES-004 (the `but review approve` write path) via ctx.db.get_cache_mut() -> DbHandle -> .local_review_verdicts_mut()/.local_review_verdicts() — the SAME project-local DbHandle that forge_reviews lives in (but-ctx OnDemandCache<but_db::DbHandle>).
  - head_oid is the consumer's @head discriminator: the merge gate compares the stored head_oid against the target's current head; storing it verbatim is what makes a stale (old-head) approval distinguishable.
pattern: New-but-db-table — a self-contained table/<name>.rs module exposing a const M migration, a Serialize/Deserialize row struct, DbHandle+Transaction accessor methods, and a read Handle (borrowing &Connection) + write HandleMut with insert(&mut self,row) and list_by_target(&self,target)->Result<Vec<Row>>, registered in the three lib.rs/mod.rs sites.
pattern_source: crates/but-db/src/table/workspace_rules.rs:8-144
anti_pattern: Reusing forge_reviews (wiped on sync at :153, no principal/verdict); adding an HMAC/signature column (R6 deferred); normalizing/truncating head_oid (collapses the @head distinction); forgetting to register the M in MIGRATIONS ('no such table' at runtime); homing the table in but-forge-storage.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — Adds a NEW persistent but-db table following GitButler's exact inline-SQL migration + table-module + handle pattern, with insert + query-by-target handle methods and integration tests against a real sqlite DbHandle. Owns the table-module composition, the head-pinning invariant (head_oid stored verbatim), and integration TDD with the crate's real in-memory sqlite harness — no mocks.
reviewer: rust-reviewer
coding_standards: crates/AGENTS.md, crates/WORKSPACE_MODEL.md, crates/but/AGENTS.md

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: AUTHZ-001 (principal_id semantics; no code dependency — the table stores plain strings)
Blocks:     GATES-003, GATES-004, GATES-005, LOOP-001
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "GATES-002",
  "proposed_by": "rust-planner",
  "verification_policy": { "requires_tests": true, "requires_red_evidence": true, "requires_seeded_evidence": true },
  "fixtures": {
    "empty_db": {
      "description": "A real in-memory but-db DbHandle with all migrations applied (including the new local_review_verdicts migration), and NO verdict rows. Seeded via the crate's existing test helper.",
      "seed_method": "public_api",
      "records": ["let db = in_memory_db();   // crates/but-db/tests/db/table/mod.rs:14 -> DbHandle::new_at_path(\":memory:\")"]
    },
    "two_principals_one_target_at_head": {
      "description": "An in-memory DbHandle seeded with TWO approving verdicts on the same target refs/heads/feat at the SAME head_oid aaaa…aaaa, from two distinct principals rust-reviewer and justin — so distinct-from-author and per-group computation is exercisable by the consumer.",
      "seed_method": "public_api",
      "records": [
        "let mut db = in_memory_db();",
        "db.local_review_verdicts_mut().insert(LocalReviewVerdict{ id:\"v1\".into(), target:\"refs/heads/feat\".into(), principal_id:\"rust-reviewer\".into(), verdict:\"approved\".into(), head_oid:\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\".into(), created_at:<fixed ts 1000000> })?;",
        "db.local_review_verdicts_mut().insert(LocalReviewVerdict{ id:\"v2\".into(), target:\"refs/heads/feat\".into(), principal_id:\"justin\".into(), verdict:\"approved\".into(), head_oid:\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\".into(), created_at:<fixed ts 1000001> })?;"
      ]
    },
    "same_target_two_heads": {
      "description": "An in-memory DbHandle seeded with ONE approving verdict on refs/heads/feat at head_oid H1 1111…1111 and ONE at head_oid H2 2222…2222 — to prove the stored head_oid distinguishes an old-head approval from a new-head one.",
      "seed_method": "public_api",
      "records": [
        "let mut db = in_memory_db();",
        "db.local_review_verdicts_mut().insert(LocalReviewVerdict{ id:\"h1\".into(), target:\"refs/heads/feat\".into(), principal_id:\"rust-reviewer\".into(), verdict:\"approved\".into(), head_oid:\"1111111111111111111111111111111111111111\".into(), created_at:<fixed ts 1000000> })?;",
        "db.local_review_verdicts_mut().insert(LocalReviewVerdict{ id:\"h2\".into(), target:\"refs/heads/feat\".into(), principal_id:\"rust-reviewer\".into(), verdict:\"approved\".into(), head_oid:\"2222222222222222222222222222222222222222\".into(), created_at:<fixed ts 1000002> })?;"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN empty_db (real in-memory but-db DbHandle) WHEN a verdict is inserted then list_by_target is called THEN exactly one row is returned byte-equal in every field, head_oid verbatim",
      "verify": "cargo test -p but-db local_review_verdicts::insert_and_query_by_target",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-db (in-memory sqlite)",
        "negative_control": { "would_fail_if": [
          "the insert is a no-op stub so list_by_target returns empty after insert",
          "list_by_target returns the row but drops/blanks head_oid or principal_id",
          "head_oid is normalized/truncated so the queried value != the inserted 40-char sha",
          "the migration is not registered so the insert errors 'no such table: local_review_verdicts'"
        ] },
        "evidence": { "artifact_type": "db_query", "required_capture": true },
        "cases": [ {
          "start_ref": "empty_db",
          "action": { "actor": "db_client", "steps": [
            "db.local_review_verdicts_mut().insert(v1) where v1 = {id:\"v1\", target:\"refs/heads/feat\", principal_id:\"rust-reviewer\", verdict:\"approved\", head_oid:\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\", created_at: ts}",
            "let rows = db.local_review_verdicts().list_by_target(\"refs/heads/feat\")?"
          ] },
          "end_state": {
            "must_observe": ["rows.len() == 1", "rows[0].principal_id == \"rust-reviewer\"", "rows[0].verdict == \"approved\"", "rows[0].head_oid == \"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\" (40-char sha intact)", "rows[0] == v1 (full struct equality)"],
            "must_not_observe": ["rows.is_empty()", "rows[0].head_oid != the inserted sha (normalized/blanked)", "a panic / 'no such table: local_review_verdicts' error"]
          }
        } ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN same_target_two_heads WHEN list_by_target is called THEN two rows are returned, one head_oid==H1 and one head_oid==H2, not collapsed/normalized",
      "verify": "cargo test -p but-db local_review_verdicts::head_pinning_distinguishes_heads",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-db (in-memory sqlite)",
        "negative_control": { "would_fail_if": [
          "head_oid is normalized so H1 and H2 collapse and only one row is observable",
          "the query dedupes by (target, principal_id) dropping the second head's row",
          "head_oid is stored but not returned, so the two rows are indistinguishable at @head",
          "a stub returns a single static row regardless of head_oid (no-op head storage)"
        ] },
        "evidence": { "artifact_type": "db_query", "required_capture": true },
        "cases": [ {
          "start_ref": "same_target_two_heads",
          "action": { "actor": "db_client", "steps": ["let rows = db.local_review_verdicts().list_by_target(\"refs/heads/feat\")?", "collect rows[*].head_oid"] },
          "end_state": {
            "must_observe": ["rows.len() == 2", "head_oid set == {\"1111111111111111111111111111111111111111\",\"2222222222222222222222222222222222222222\"}", "a row with head_oid==H1 AND a row with head_oid==H2 both present"],
            "must_not_observe": ["rows.len() == 1 (collapsed)", "both rows same head_oid (normalized)", "the H2 row missing (empty)"]
          }
        } ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN empty_db (and a populated unrelated target) WHEN list_by_target is called for a target with no rows THEN an empty Vec is returned (not Err), and another target's rows do not leak",
      "verify": "cargo test -p but-db local_review_verdicts::empty_target_returns_empty",
      "maps_to_ac": null,
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-db (in-memory sqlite)",
        "negative_control": { "would_fail_if": [
          "list_by_target errors rather than returning an empty Vec when no rows match",
          "list_by_target ignores the target filter and returns another target's rows",
          "an empty query panics on an empty result set"
        ] },
        "evidence": { "artifact_type": "db_query", "required_capture": true },
        "cases": [ {
          "start_ref": "empty_db",
          "action": { "actor": "db_client", "steps": [
            "let rows = db.local_review_verdicts().list_by_target(\"refs/heads/feat\")?  (empty table)",
            "db.local_review_verdicts_mut().insert({id:\"o1\", target:\"refs/heads/other\", principal_id:\"rust-reviewer\", verdict:\"approved\", head_oid:\"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\", created_at: ts})?",
            "let feat_rows = db.local_review_verdicts().list_by_target(\"refs/heads/feat\")?  (other target populated)"
          ] },
          "end_state": {
            "must_observe": ["the first query returns Ok with `rows.len() == 0` (empty, not Err)", "after inserting `o1` on refs/heads/other, `list_by_target(\"refs/heads/other\")` returns `rows.len() == 1` with `rows[0].id == \"o1\"`", "`list_by_target(\"refs/heads/feat\")` returns `rows.len() == 0` (the refs/heads/other verdict does NOT leak into the feat query)"],
            "must_not_observe": ["an Err / panic on the empty query", "feat_rows contains the refs/heads/other verdict (target filter ignored)", "rows.len() != 0 for the empty feat target"]
          }
        } ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN two_principals_one_target_at_head WHEN list_by_target is called THEN both rows are returned with distinct principal_id intact, ordered by created_at",
      "verify": "cargo test -p but-db local_review_verdicts::distinct_principals_both_returned",
      "maps_to_ac": null,
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-db (in-memory sqlite)",
        "negative_control": { "would_fail_if": [
          "the query dedupes by target and returns only one principal's verdict",
          "principal_id is blanked/collapsed so the two reviewers are indistinguishable",
          "the result is unordered and the test cannot read both rows deterministically",
          "a stub returns one hardcoded row instead of both principals' verdicts"
        ] },
        "evidence": { "artifact_type": "db_query", "required_capture": true },
        "cases": [ {
          "start_ref": "two_principals_one_target_at_head",
          "action": { "actor": "db_client", "steps": ["let rows = db.local_review_verdicts().list_by_target(\"refs/heads/feat\")?", "collect rows[*].principal_id"] },
          "end_state": {
            "must_observe": ["rows.len() == 2", "principal_id set == {\"rust-reviewer\",\"justin\"}", "ordered by created_at (rust-reviewer ts=1000000 before justin ts=1000001)", "every row.verdict == \"approved\""],
            "must_not_observe": ["rows.len() == 1 (deduped, none)", "both rows same principal_id (collapsed)", "a missing principal_id field"]
          }
        } ]
      }
    },
    { "id": "TC-1", "type": "test_criterion", "description": "insert + query-by-target returns one row, all fields byte-intact", "verify": "cargo test -p but-db local_review_verdicts::insert_and_query_by_target", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "head_oid verbatim — H1 and H2 are distinct rows (underpins T-GATES-011 @head)", "verify": "cargo test -p but-db local_review_verdicts::head_pinning_distinguishes_heads", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "empty target returns empty Vec; target filter honored (T-GATES-009 start signature)", "verify": "cargo test -p but-db local_review_verdicts::empty_target_returns_empty", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "two distinct principals both returned, ordered (T-GATES-010/012/014 computable)", "verify": "cargo test -p but-db local_review_verdicts::distinct_principals_both_returned", "maps_to_ac": "AC-4" },
    { "id": "TC-5", "type": "test_criterion", "description": "migration registered with greatest timestamp + struct re-exported; not homed in forge_reviews", "verify": "cargo test -p but-db local_review_verdicts", "maps_to_ac": "AC-1" }
  ]
}
-->
</details>
