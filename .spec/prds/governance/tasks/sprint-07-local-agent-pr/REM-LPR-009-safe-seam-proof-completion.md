# REM-LPR-009: Complete the safe-seam proof — move grep into invariant_build_gates.rs with SAFE_SEAM_NO_READ/REVIEW_REQUIREMENT/SAFE_SEAM_GATE_PATHS consts; add the missing bidirectional forged-vs-empty equivalence test + the three-step chained capstone

> Status: Backlog
> Commit: (none yet)
> Reviewer: rust-reviewer
> Updated: 2026-06-22T18:00:00Z
> PROPOSED-BY: rust-planner

## What this does

This remediation finishes the load-bearing safe-seam proof that LPR-009 promised but left incomplete (see `.spec/reviews/red-hat-20260622-173510.md` C2). It:

1. **Moves the build-gate honesty grep** from `crates/but-api/tests/safe_seam_invariant.rs` into `crates/but-authz/tests/invariant_build_gates.rs`, reusing the shipped `assert_grep_has_no_matches` helper and adding the required `REVIEW_REQUIREMENT`, `SAFE_SEAM_NO_READ`, and `SAFE_SEAM_GATE_PATHS` consts.
2. **Adds the missing bidirectional runtime equivalence test** `safe_seam_forged_drive_equals_empty_drive` (T-LPR-041) — proving a fully forged drive layer yields the **same** merge-gate decision as an empty one for both satisfied and unsatisfied verdict fixtures.
3. **Adds the missing three-step chained capstone** `safe_seam_only_verdict_at_head_flips_the_land` (T-LPR-040) — Step1 blocked, Step2 forged drive rows still blocked with an identical decision, Step3 approved verdict@head via governed `approve_review` proceeds.
4. **Adds the missing sub-case test** `safe_seam_drive_rows_have_no_effect_on_satisfied_merge` (T-LPR-035/036/037) — pending/changes_requested assignment and unresolved comment do not alter a verdict-satisfied merge.

The existing runtime tests `forged_drive_metadata_with_no_verdict_is_blocked` and `only_verdict_at_head_flips_gate` are preserved.

## Why

The red-hat review `.spec/reviews/red-hat-20260622-173510.md` flagged **C2 (CRITICAL)** against the shipped LPR-009 implementation:

- The grep was in the wrong file (`safe_seam_invariant.rs` with `fs::read_to_string` instead of `invariant_build_gates.rs` with `assert_grep_has_no_matches`).
- The required consts were missing.
- The bidirectional forged-vs-empty equivalence was missing its satisfied direction.
- The three-step chained capstone was missing.
- The pending-assignment and unresolved-comment sub-cases were missing.

Without these, the safe-seam proof is load-bearing but incomplete. This remediation brings the proof to the state required by the original LPR-009 spec: **"Gate gates (verdict-at-head, untouched); new tables drive (orchestration)."**

## How to verify

PRIMARY **AC-1** — `cargo test -p but-authz invariant_build_gates` passes and the grep assertion over `merge_gate.rs` + `review_requirement.rs` finds **zero** references to `local_review_assignments`, `local_review_comments`, or `local_review_meta`.

## Scope

- `crates/but-authz/tests/invariant_build_gates.rs` (MODIFY — add `REVIEW_REQUIREMENT`, `SAFE_SEAM_NO_READ`, `SAFE_SEAM_GATE_PATHS`; add `assert_grep_has_no_matches` call)
- `crates/but-api/tests/safe_seam_invariant.rs` (MODIFY — remove the now-redundant inline grep test; add `safe_seam_drive_rows_have_no_effect_on_satisfied_merge`, `safe_seam_only_verdict_at_head_flips_the_land`, `safe_seam_forged_drive_equals_empty_drive`; preserve existing runtime tests)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: REM-LPR-009 — Complete the safe-seam proof
================================================================================

TASK_TYPE:   FEATURE (test-only; load-bearing)
STATUS:      Backlog
PRIORITY:    P0 (the load-bearing gate — the slice is not done without it)
AGENT:       implementer=rust-reviewer | reviewer=rust-reviewer
EFFORT:      M (90 min)
PROPOSED-BY: rust-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-LPR-07, LPR-009
CAPABILITIES:CAP-AUTHZ-01
RELATED:     .spec/reviews/red-hat-20260622-173510.md C2

RUNTIME_COMMANDS:
  test:  cargo test -p but-authz invariant_build_gates
  test:  cargo test -p but-api --test safe_seam_invariant
  check: cargo check -p but-api -p but-authz --all-targets
  lint:  cargo clippy -p but-api -p but-authz --all-targets
  fmt:   cargo fmt --check

--------------------------------------------------------------------------------
TYPE DESIGN (Rust shapes this task introduces)
--------------------------------------------------------------------------------
NO new product types. This task adds:
  - build-gate grep constants in crates/but-authz/tests/invariant_build_gates.rs:
      const REVIEW_REQUIREMENT: &str = "crates/but-api/src/legacy/review_requirement.rs";
      const SAFE_SEAM_NO_READ: &str = r#"local_review_assignments|local_review_comments|local_review_meta"#;
      const SAFE_SEAM_GATE_PATHS: &[&str] = &[MERGE_GATE, REVIEW_REQUIREMENT];
  - an assert_grep_has_no_matches call inside (or beside) the existing invariant_build_gates test.
  - three new #[tokio::test] #[serial_test::serial] async functions in crates/but-api/tests/safe_seam_invariant.rs:
      safe_seam_drive_rows_have_no_effect_on_satisfied_merge
      safe_seam_only_verdict_at_head_flips_the_land
      safe_seam_forged_drive_equals_empty_drive
ERROR STRATEGY:
  - the grep test reuses the anyhow::Result<()> helpers in invariant_build_gates.rs.
  - runtime tests assert on enforce_merge_gate's Result: Ok = proceeds; Err downcast to MergeGateError with code == "gate.review_required" = blocked.
OWNERSHIP PLAN:
  - runtime tests build a real but_ctx::Context + DbHandle (LPR-001 tables migrated) + a real gix repo via but_testsupport; rows are seeded by value through the LPR-001 Handles (forged drive layer) and via the governed approve_review verb (legitimate verdict@head); enforce_merge_gate borrows &ctx.

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
After this remediation:
  - (BUILD-GATE) invariant_build_gates.rs contains the additive safe-seam grep proving merge_gate.rs + review_requirement.rs reference none of the three drive tables.
  - (RUNTIME)
      - T-LPR-035/036/037: a verdict-satisfied merge proceeds identically with empty tables, with pending/changes_requested assignments, and with an unresolved comment.
      - T-LPR-040: Step1 blocked → Step2 forged drive rows still blocked (IDENTICAL decision) → Step3 approved verdict@head proceeds — only the verdict flips the land.
      - T-LPR-041: forged drive layer ≡ empty drive layer for both satisfied and unsatisfied fixtures.
      - T-LPR-042 (existing): drive-only fixture is blocked.
  - All verification gates pass; only write_allowed files modified.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST add `const REVIEW_REQUIREMENT: &str = "crates/but-api/src/legacy/review_requirement.rs";` and `const SAFE_SEAM_NO_READ: &str = r#"local_review_assignments|local_review_comments|local_review_meta"#;` and `const SAFE_SEAM_GATE_PATHS: &[&str] = &[MERGE_GATE, REVIEW_REQUIREMENT];` to invariant_build_gates.rs.
- [MUST] MUST add an `assert_grep_has_no_matches` call over SAFE_SEAM_GATE_PATHS for SAFE_SEAM_NO_READ to the existing `invariant_build_gates` test (or a new sibling test in the same file). Use the shipped helper in invariant_build_gates.rs.
- [MUST] MUST keep the grep ADDITIVE — do NOT weaken any existing pattern (ROLE_BRANCH_PATTERN, HUMAN_OR_LABEL_BRANCH_PATTERN, AUTHORITY_POSITIVE_PATTERN, PERMISSION_CARRIER_PATTERN) or ENFORCEMENT_PATHS entry.
- [MUST] MUST add `safe_seam_drive_rows_have_no_effect_on_satisfied_merge` (T-LPR-035/036/037) proving pending/changes_requested assignment AND unresolved comment do NOT affect a verdict-satisfied merge.
- [MUST] MUST add `safe_seam_forged_drive_equals_empty_drive` (T-LPR-041) — BIDIRECTIONAL: satisfied (both proceed) AND unsatisfied (both blocked). The equivalence must hold in both directions, not just the blocked one (otherwise a gate that ignores everything would trivially pass).
- [MUST] MUST add `safe_seam_only_verdict_at_head_flips_the_land` (T-LPR-040) — the THREE-STEP CHAINED capstone: Step1 (no verdict) blocked → Step2 add forged assignments+comments, STILL blocked with IDENTICAL decision → Step3 add approved verdict@head via governed approve_review, NOW proceeds. Assert Step1==Step2 decision identity and Step3 flip.
- [MUST] MUST state the safe-seam invariant VERBATIM in the test: "Gate gates (verdict-at-head, untouched); new tables drive (orchestration)."
- [MUST] MUST drive the merge decision through `enforce_merge_gate` (merge_gate.rs:40) and seed legitimate approvals via `approve_review` (forge.rs:775).
- [NEVER] NEVER write a test asserting a forged DIRECT DB write to local_review_verdicts is BLOCKED (R6/R18 accepted-leak — encodes false guarantee).
- [NEVER] NEVER add a read of the three new tables to merge_gate.rs or review_requirement.rs (would BREAK the invariant).
- [NEVER] NEVER weaken any assertion to make forged-vs-empty pass — if forged ≠ empty, that's a REAL violation; fix the gate-path leak, do not lower the assertion.
- [NEVER] NEVER add new gitbutler-* usage.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY] (T-LPR-038): the build-gate grep lives in invariant_build_gates.rs and finds ZERO references to the three new tables in merge_gate.rs + review_requirement.rs
- [ ] AC-2 (T-LPR-035/036/037): a verdict-satisfied merge proceeds unchanged with an open pending/changes_requested assignment and with an unresolved comment thread
- [ ] AC-3 (T-LPR-040): the chained three-step capstone — Step1==Step2 blocked-identical, Step3 proceeds
- [ ] AC-4 (T-LPR-041): forged drive layer ≡ empty drive layer bidirectionally (satisfied and unsatisfied fixtures)
- [ ] AC-5 (T-LPR-042): drive-only (no verdict) cannot land (existing test preserved)
- [ ] AC-6 (T-LPR-039): R6/R18 threat model preserved unchanged; R18 stays named in a test comment and the gate read set is unchanged
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY] (T-LPR-038): build-gate grep lives in invariant_build_gates.rs and finds ZERO references to the three new tables
  GIVEN: the merge-gate code path = crates/but-api/src/legacy/merge_gate.rs + crates/but-api/src/legacy/review_requirement.rs
  WHEN:  the net-new build-gate honesty grep runs (assert_grep_has_no_matches over SAFE_SEAM_GATE_PATHS for the pattern `local_review_assignments|local_review_comments|local_review_meta`)
  THEN:  ZERO matches — none of the three drive tables is referenced in the gate path; the drive/gate separation is enforced at BUILD time; the existing invariant_build_gates patterns remain green and unweakened
  TEST_TIER: build-gate   VERIFICATION_SERVICE: the shipped but-authz invariant_build_gates assert_grep_has_no_matches helper over merge_gate.rs + review_requirement.rs
  VERIFY: cargo test -p but-authz invariant_build_gates

AC-2 (T-LPR-035/036/037): each new table has NO effect on a verdict-satisfied merge
  GIVEN: lpr_safe_seam_repo: a branch with a distinct approved verdict@head (gate-satisfied) + a merge holder; the three new tables present
  WHEN:  the governed merge runs (a) with all three new tables EMPTY, (b) with an open `pending` AND a separate `changes_requested` assignment, (c) with an unresolved comment thread
  THEN:  the merge PROCEEDS in all three — enforce_merge_gate decides purely on verdict-at-head; the assignment never gates (T-LPR-036) and the comment never gates (T-LPR-037); the decision is identical to the empty-tables case
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api enforce_merge_gate + real but-db (the new tables seeded) + real gix via but_testsupport
  VERIFY: cargo test -p but-api safe_seam_drive_rows_have_no_effect_on_satisfied_merge

AC-3 (T-LPR-040): THE SAFE-SEAM PROOF — only a verdict-at-head flips the land
  GIVEN: lpr_safe_seam_repo: a branch + merge holder, NO verdict@head
  WHEN:  Step1 attempt the governed merge; Step2 add `pending`/`changes_requested` assignments AND unresolved comments (directly), re-attempt; Step3 add a distinct `approved` verdict@head via governed `but review approve`, re-attempt
  THEN:  Step1 BLOCKED (gate.review_required); Step2 STILL BLOCKED with the IDENTICAL decision (the new tables never flip it); Step3 PROCEEDS — ONLY the verdict-at-head flips the land
  TEST_TIER: e2e-automated   VERIFICATION_SERVICE: real but-api enforce_merge_gate + governed approve_review + real but-db drive-row seeding + real gix
  VERIFY: cargo test -p but-api safe_seam_only_verdict_at_head_flips_the_land

AC-4 (T-LPR-041): forged drive layer ≡ empty drive layer
  GIVEN: lpr_safe_seam_repo run twice for each verdict fixture: (a) zero rows in all three new tables; (b) a fully forged set — all local_review_assignments `approved`, all local_review_comments `resolved` — written DIRECTLY
  WHEN:  the governed merge runs in each, for a SATISFIED verdict@head fixture and an UNSATISFIED (no approval@head) fixture
  THEN:  the merge-gate decision is IDENTICAL between (a) and (b) for every fixture — satisfied ⇒ proceeds in BOTH; unsatisfied ⇒ blocked in BOTH — the safe seam holds under an adversarial drive layer
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api enforce_merge_gate + a directly-written forged drive layer vs an empty one + real gix
  VERIFY: cargo test -p but-api safe_seam_forged_drive_equals_empty_drive

AC-5 (T-LPR-042): inverse — drive metadata alone cannot land
  GIVEN: lpr_safe_seam_repo: a branch + merge holder, NO verdict@head, WITH a `pending` assignment AND an unresolved comment thread present
  WHEN:  the governed merge runs
  THEN:  the merge is BLOCKED (gate.review_required) — drive metadata alone never satisfies the gate; only an approved verdict-at-head can
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api enforce_merge_gate + a drive-only (no verdict) fixture + real gix
  VERIFY: cargo test -p but-api forged_drive_metadata_with_no_verdict_is_blocked

AC-6 (T-LPR-039): the R6/R18 threat model is preserved unchanged
  GIVEN: the gate's read set (merge_gate.rs + review_requirement.rs) + the risk register (R18, §G)
  WHEN:  the gate's reads are audited and the safe-seam grep + runtime tests are run
  THEN:  the gate reads ONLY the verdict store; a direct DB write to a drive table widens NO land-truth surface; R18 stays NAMED — the local PR is NOT presented as independently audited
  TEST_TIER: build-gate   VERIFICATION_SERVICE: the safe-seam grep + runtime forged-vs-empty equivalence; audit note in test that R18 stays named
  VERIFY: cargo test -p but-authz invariant_build_gates && cargo test -p but-api safe_seam_forged_drive_equals_empty_drive

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): the grep over merge_gate.rs + review_requirement.rs for `local_review_assignments|local_review_comments|local_review_meta` returns ZERO matches; existing patterns stay green
- TC-2 (-> AC-2): a verdict-satisfied merge proceeds with the new tables empty, with a pending/changes_requested assignment, and with an unresolved comment — identical decision in all
- TC-3 (-> AC-3): Step1 blocked, Step2 (forged drive rows) blocked-identical, Step3 (approved verdict@head) proceeds
- TC-4 (-> AC-4): forged drive layer ≡ empty for a SATISFIED fixture (both proceed) AND an UNSATISFIED fixture (both blocked)
- TC-5 (-> AC-5): a pending assignment + unresolved comment with NO approved verdict is BLOCKED (gate.review_required)
- TC-6 (-> AC-6): the gate's read set adds no read of the new tables (grep) AND the runtime equivalence holds; R18 stays named in the test note

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- cargo test -p but-authz invariant_build_gates   -> Exit 0; safe-seam grep finds zero new-table references; existing patterns green
- cargo test -p but-api --test safe_seam_invariant   -> Exit 0; all runtime tests pass
- rg "fn (safe_seam_drive_rows_have_no_effect|safe_seam_forged_drive_equals_empty|safe_seam_only_verdict_at_head_flips_the_land)" crates/but-api/tests/   -> 3 matches
- rg "SAFE_SEAM_NO_READ|SAFE_SEAM_GATE_PATHS|const REVIEW_REQUIREMENT" crates/but-authz/tests/invariant_build_gates.rs   -> 3 matches
- cargo check -p but-api -p but-authz --all-targets   -> Exit 0
- cargo clippy -p but-api -p but-authz --all-targets   -> Exit 0
- cargo fmt --check   -> Exit 0

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-authz/tests/invariant_build_gates.rs (MODIFY — add 3 consts + grep call; additive only)
  - crates/but-api/tests/safe_seam_invariant.rs (MODIFY — remove redundant inline grep test; add 3 new runtime tests; preserve existing runtime tests)
writeProhibited:
  - crates/but-api/src/legacy/merge_gate.rs — CONSUME-only; NEVER add a read of the new tables
  - crates/but-api/src/legacy/review_requirement.rs — CONSUME-only
  - crates/but-api/src/legacy/forge.rs — CONSUME-only (only approve_review is called)
  - existing invariant_build_gates patterns/ENFORCEMENT_PATHS — additive only
  - any test asserting a direct DB write to local_review_verdicts is blocked (false R6/R18 guarantee — forbidden)
  - any gitbutler-* crate (no new gitbutler-* usage)
  - any file not in write_allowed

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agent: implementer=rust-reviewer | reviewer=rust-reviewer
rationale: This is the load-bearing safe-seam invariant proof. It is an audit/honesty-grep + adversarial-equivalence task (no product logic), best owned by rust-reviewer: writing the no-read grep in the same discipline as the shipped AUTHORITY_POSITIVE gate, and writing the forged-vs-empty/capstone integration tests that prove drive rows never flip the land.

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: none at runtime (the fixtures use the existing LPR-001 tables + shipped approve_review/enforce_merge_gate)
Blocks:     none formally, but the sprint cannot be closed without this remediation green
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "REM-LPR-009",
  "proposed_by": "rust-planner",
  "red_hat_ref": ".spec/reviews/red-hat-20260622-173510.md C2",
  "verification_policy": { "requires_tests": true, "requires_red_evidence": true, "requires_seeded_evidence": true },
  "fixtures": {
    "lpr_safe_seam_repo": {
      "description": "A real governed repo via but_testsupport::writable_scenario + invoke_bash committing .gitbutler/{permissions,gates}.toml. A real but_ctx::Context + DbHandle with the LPR-001 tables migrated. Legitimate approval@head written via governed approve_review. Forged drive layer written DIRECTLY via LPR-001 Handles. Merge decision read via enforce_merge_gate.",
      "seed_method": "public_api",
      "records": [
        "but_testsupport::writable_scenario(...) + invoke_bash committing .gitbutler/{permissions,gates}.toml",
        "legitimate approvals: governed approve_review(ctx, 'feat')",
        "forged drive layer: db.local_review_assignments_mut().upsert() + db.local_review_comments_mut().insert() + db.local_review_meta_mut().upsert_if_absent() DIRECT",
        "enforce_merge_gate(&ctx, REVIEW_ID)"
      ]
    }
  },
  "requirements": [
    { "id": "AC-1", "type": "acceptance_criterion", "primary": true, "description": "GIVEN the gate path = merge_gate.rs + review_requirement.rs WHEN the net-new build-gate grep runs (assert_grep_has_no_matches over SAFE_SEAM_GATE_PATHS for local_review_assignments|local_review_comments|local_review_meta) THEN ZERO matches; existing invariant_build_gates patterns remain green and unweakened", "verify": "cargo test -p but-authz invariant_build_gates", "scenario": { "tier": "visible", "test_tier": "build-gate", "verification_service": "the shipped but-authz invariant_build_gates assert_grep_has_no_matches helper", "negative_control": { "would_fail_if": ["merge_gate.rs or review_requirement.rs referenced any new table", "the grep targeted the wrong files", "an existing invariant_build_gates pattern were weakened"] }, "evidence": { "artifact_type": "test_output", "required_capture": true }, "cases": [ { "start_ref": "lpr_safe_seam_repo", "action": { "actor": "ci", "steps": ["run the SAFE_SEAM_NO_READ grep over SAFE_SEAM_GATE_PATHS", "run the full invariant_build_gates suite"] }, "end_state": { "must_observe": ["zero matches in the gate path", "existing patterns still pass"], "must_not_observe": ["any match of any new-table symbol in the gate path", "a weakened/removed existing pattern"] } } ] } },
    { "id": "AC-2", "type": "acceptance_criterion", "primary": false, "description": "GIVEN lpr_safe_seam_repo with a distinct approved verdict@head WHEN the governed merge runs (a) tables empty, (b) pending+changes_requested assignment, (c) unresolved comment thread THEN the merge PROCEEDS in all three — identical to the empty case", "verify": "cargo test -p but-api safe_seam_drive_rows_have_no_effect_on_satisfied_merge", "scenario": { "tier": "holdout", "test_tier": "integration", "verification_service": "real but-api enforce_merge_gate + real but-db + real gix", "negative_control": { "would_fail_if": ["adding an assignment or comment changed the merge decision", "a changes_requested assignment blocked the merge"] }, "evidence": { "artifact_type": "api_response", "required_capture": true }, "cases": [ { "start_ref": "lpr_safe_seam_repo", "action": { "actor": "ci", "steps": ["seed approved verdict@head via governed approve_review", "run enforce_merge_gate (a) empty", "seed pending + changes_requested; run (b)", "seed unresolved comment; run (c)"] }, "end_state": { "must_observe": ["(a) proceeds", "(b) proceeds", "(c) proceeds"], "must_not_observe": ["(b) or (c) blocked while (a) proceeds"] } } ] } },
    { "id": "AC-3", "type": "acceptance_criterion", "primary": false, "description": "GIVEN lpr_safe_seam_repo, NO verdict@head WHEN Step1 attempt merge; Step2 add forged pending/changes_requested assignments + unresolved comments, re-attempt; Step3 add distinct approved verdict@head via governed approve_review, re-attempt THEN Step1 BLOCKED; Step2 STILL BLOCKED with IDENTICAL decision; Step3 PROCEEDS — only the verdict-at-head flips the land", "verify": "cargo test -p but-api safe_seam_only_verdict_at_head_flips_the_land", "scenario": { "tier": "holdout", "test_tier": "e2e-automated", "verification_service": "real but-api enforce_merge_gate + governed approve_review + real but-db drive seeding + real gix", "negative_control": { "would_fail_if": ["Step2 (forged drive rows) flipped the decision to proceed", "Step3 did not proceed after a real approved verdict@head", "Step1 and Step2 decisions differed"] }, "evidence": { "artifact_type": "api_response", "required_capture": true }, "cases": [ { "start_ref": "lpr_safe_seam_repo", "action": { "actor": "ci", "steps": ["Step1: enforce_merge_gate (no verdict@head)", "Step2: directly write pending/changes_requested + unresolved comments; enforce_merge_gate", "Step3: governed approve_review; enforce_merge_gate"] }, "end_state": { "must_observe": ["Step1 blocked", "Step2 blocked IDENTICAL to Step1", "Step3 proceeds"], "must_not_observe": ["Step2 proceeding", "Step3 still blocked"] } } ] } },
    { "id": "AC-4", "type": "acceptance_criterion", "primary": false, "description": "GIVEN lpr_safe_seam_repo run twice — (a) zero rows, (b) fully forged set — for SATISFIED and UNSATISFIED verdict@head fixtures WHEN the governed merge runs THEN the decision is IDENTICAL between (a) and (b) for every fixture: satisfied proceeds in both; unsatisfied blocked in both", "verify": "cargo test -p but-api safe_seam_forged_drive_equals_empty_drive", "scenario": { "tier": "holdout", "test_tier": "integration", "verification_service": "real but-api enforce_merge_gate + forged drive layer vs empty + real gix", "negative_control": { "would_fail_if": ["forged != empty for the satisfied fixture", "forged != empty for the unsatisfied fixture", "test only checked the blocked direction"] }, "evidence": { "artifact_type": "api_response", "required_capture": true }, "cases": [ { "start_ref": "lpr_safe_seam_repo", "action": { "actor": "ci", "steps": ["SATISFIED: approved verdict@head; run empty (a) and forged (b)", "UNSATISFIED: no approval@head; run empty (a) and forged (b)"] }, "end_state": { "must_observe": ["satisfied: (a) proceeds AND (b) proceeds", "unsatisfied: (a) blocked AND (b) blocked"], "must_not_observe": ["any (a) != (b)"] } } ] } },
    { "id": "AC-5", "type": "acceptance_criterion", "primary": false, "description": "GIVEN lpr_safe_seam_repo, NO verdict@head, WITH pending assignment AND unresolved comment WHEN the governed merge runs THEN BLOCKED (gate.review_required)", "verify": "cargo test -p but-api forged_drive_metadata_with_no_verdict_is_blocked", "scenario": { "tier": "holdout", "test_tier": "integration", "verification_service": "real but-api enforce_merge_gate + drive-only (no verdict) fixture + real gix", "negative_control": { "would_fail_if": ["merge PROCEEDED on drive metadata alone", "a 'pending' assignment were misread as an approval"] }, "evidence": { "artifact_type": "api_response", "required_capture": true }, "cases": [ { "start_ref": "lpr_safe_seam_repo", "action": { "actor": "ci", "steps": ["seed pending assignment + unresolved comment; NO verdict@head", "run enforce_merge_gate"] }, "end_state": { "must_observe": ["merge blocked (gate.review_required)"], "must_not_observe": ["merge proceeding on drive metadata alone"] } } ] } },
    { "id": "AC-6", "type": "acceptance_criterion", "primary": false, "description": "GIVEN the gate's read set + R18 risk register WHEN audited and safe-seam grep + runtime tests run THEN gate reads ONLY the verdict store; direct DB write to drive table widens NO land-truth surface; R18 stays NAMED", "verify": "cargo test -p but-authz invariant_build_gates && cargo test -p but-api safe_seam_forged_drive_equals_empty_drive", "scenario": { "tier": "visible", "test_tier": "build-gate", "verification_service": "safe-seam grep + runtime forged-vs-empty equivalence + audit note", "negative_control": { "would_fail_if": ["a new-table read were added to the gate path", "a test asserted the verdict store is unforgeable", "the test note claimed the local PR is independently audited"] }, "evidence": { "artifact_type": "test_output", "required_capture": true }, "cases": [ { "start_ref": "lpr_safe_seam_repo", "action": { "actor": "ci", "steps": ["audit the gate's read set", "run safe-seam grep + forged-vs-empty equivalence", "confirm R18 named in the test note"] }, "end_state": { "must_observe": ["gate reads only verdict store at head", "new tables add no read to the gate path", "R18 stays named"], "must_not_observe": ["new-table read in gate path", "test asserting verdict store unforgeable", "claim of independent audit"] } } ] } },
    { "id": "TC-1", "type": "test_criterion", "description": "the grep over merge_gate.rs + review_requirement.rs for local_review_assignments|local_review_comments|local_review_meta returns ZERO matches; existing patterns stay green", "verify": "cargo test -p but-authz invariant_build_gates", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "a verdict-satisfied merge proceeds with the new tables empty, with a pending/changes_requested assignment, and with an unresolved comment — identical decision in all", "verify": "cargo test -p but-api safe_seam_drive_rows_have_no_effect_on_satisfied_merge", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "Step1 blocked, Step2 (forged drive rows) blocked-identical, Step3 (approved verdict@head) proceeds", "verify": "cargo test -p but-api safe_seam_only_verdict_at_head_flips_the_land", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "forged drive layer ≡ empty for a SATISFIED fixture (both proceed) AND an UNSATISFIED fixture (both blocked)", "verify": "cargo test -p but-api safe_seam_forged_drive_equals_empty_drive", "maps_to_ac": "AC-4" },
    { "id": "TC-5", "type": "test_criterion", "description": "a pending assignment + unresolved comment with NO approved verdict is BLOCKED (gate.review_required)", "verify": "cargo test -p but-api forged_drive_metadata_with_no_verdict_is_blocked", "maps_to_ac": "AC-5" },
    { "id": "TC-6", "type": "test_criterion", "description": "the gate's read set adds no read of the new tables (grep) AND the runtime equivalence holds; R18 stays named in the test note", "verify": "cargo test -p but-authz invariant_build_gates && cargo test -p but-api safe_seam_forged_drive_equals_empty_drive", "maps_to_ac": "AC-6" }
  ]
}
-->
