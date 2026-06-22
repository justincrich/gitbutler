# LPR-009 Self-Review

## 1. Do the three tests actually discriminate? (Body-deletion test)

**Yes — each test drives a real gate decision, not a stub.**

### Test 1: `merge_gate_path_has_zero_references_to_drive_tables`
- **Body-deletion test**: If you replaced the body with `Ok(())`, the test would
  trivially pass — BUT that is not what the test does. It reads the ACTUAL gate
  source files, lowercases them, and asserts zero violations. If
  `merge_gate.rs` or `review_requirement.rs` contained a reference to any drive
  table, the test would FAIL.
- **Vacuity guard**: The test also asserts `lower.contains("local_review_verdicts")`
  — a sanity check that the gate path DOES reference the verdict store. Without
  this, pointing the grep at empty files would pass vacuously.

### Test 2: `forged_drive_metadata_with_no_verdict_is_blocked`
- **Body-deletion test**: If you replaced the body with `Ok(())`, it would
  pass — but that is not what the test does. It:
  1. Writes REAL forged rows via the LPR-001 Handle mutators
  2. Verifies they are present in the DB (vacuity guards)
  3. Calls `enforce_merge_gate` (the real gate function)
  4. Matches on the actual `Result` and asserts the error code is
     `gate.review_required` with `no_approval` in unmet
- If the gate read `local_review_assignments` and treated the forged `"approved"`
  state as satisfying the requirement, the merge would PROCEED and the
  `Ok(()) => panic!` arm would fire.

### Test 3: `only_verdict_at_head_flips_gate`
- **Body-deletion test**: Same — the test writes a REAL verdict, verifies the
  drive tables are empty (vacuity guard), and calls `enforce_merge_gate`.
- If the gate DIDN'T read `local_review_verdicts` (the inverse break), the
  verdict-at-head would be ignored and the gate would BLOCK, failing the
  `gate_result.is_ok()` assertion.
- If the `head_oid` were wrong (stale), the gate would also block — the test is
  sensitive to the verdict-at-head match.

## 2. Is the grep test weak? (Would it pass if the gate read via a re-export?)

**Partially — the grep catches literal symbol references, not derived reads.**
This is why the spec requires BOTH the static grep AND the runtime tests:

- **What the grep catches**: any literal `local_review_assignments`,
  `local_review_comments`, or `local_review_meta` string in the gate path. This
  includes `use` imports, direct method calls, and SQL table names.
- **What the grep does NOT catch**: a hypothetical derived read via a function
  that wraps the table access and is itself called from the gate path without
  naming the table. This is the known limitation of any static grep.
- **Why it's sufficient**: the runtime tests (Test 2 + Test 3) cover this gap.
  If the gate read the drive tables via any indirection, Test 2's forged
  assignments would flip the gate decision. The bidirectional proof
  (forged = blocked, verdict = proceeds) closes the gap the grep leaves open.
- **Verdict**: the grep is the STATIC layer; the runtime tests are the DYNAMIC
  layer. Together they are sufficient. Neither alone would be.

## 3. Are the forged/inverse tests using REAL governed merge?

**Yes — both call `but_api::legacy::merge_gate::enforce_merge_gate`, the real
gate function under proof.**

- `enforce_merge_gate` is the SAME function `but_api::legacy::forge::merge_review`
  calls internally before proceeding to the forge network call.
- It reads governance config from the REAL committed `.gitbutler/permissions.toml`
  and `.gitbutler/gates.toml` at the target ref (via `gix`).
- It resolves the principal from `BUT_AGENT_HANDLE` via `but_authz::resolve_principal_from_env`.
- It enforces merge authority via `but_authz::authorize`.
- It reads verdicts from the REAL in-memory DB cache via
  `ctx.db.get_cache()?.local_review_verdicts().list_by_target(target)`.
- The fixture uses `but_testsupport::writable_scenario` + `invoke_bash` to
  create a REAL git repo with committed governance config — no mocks.

## 4. Constraint compliance

- [x] NEVER touched `merge_gate.rs` or `review_requirement.rs` source code
- [x] NEVER asserted that direct DB forgery of `local_review_verdicts` is BLOCKED
      (Test 3 asserts it PROCEEDS — proving the gate reads verdicts, the opposite)
- [x] NEVER asserted that direct DB forgery of `local_review_meta` opener row is
      blocked (R23 accepted-leak)
- [x] Forged test writes via raw `DbHandle` Handle mutators, NOT governed verbs
- [x] Followed the canonical fixture pattern from `crates/but-api/tests/merge_gate.rs`
- [x] All tests pass, clippy clean, fmt clean

## 5. Spec-vs-implementation notes

The spec calls for 6 ACs across two files (`crates/but-authz/tests/invariant_build_gates.rs`
for the grep + `crates/but-api/tests/safe_seam.rs` for runtime tests). The task
prompt consolidated to 3 tests in one file (`crates/but-api/tests/safe_seam_invariant.rs`).
The 3 tests cover the load-bearing ACs:
- AC-1 (static grep) → Test 1
- AC-5 (inverse: drive-only blocked) → Test 2
- AC-3 (capstone: verdict flips gate) → Test 3

The spec's AC-2 (each table has no effect on satisfied merge), AC-4
(forged = empty bidirectional), and AC-6 (R18 stays named) are covered in
principle by the three tests but not as separate test functions. If the
reviewer requires the full 6-AC test set, the spec's file layout should be
followed instead.
