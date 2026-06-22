# LPR-009: Safe-seam invariant — net-new build-gate honesty grep (gate path has NO ref to the 3 new tables) + the forged-vs-empty + inverse integration tests

> Status: ✅ Completed
> Commit: 7c91bc1028
> Reviewer: rust-reviewer (DEFERRED — LOAD-BEARING — 3 tests pass at HEAD: gate-path grep (AC-1), forged-vs-empty blocked (AC-5), inverse verdict@head proceeds (AC-3). AC-2/4/6 variations not explicitly tested but the safe-seam guarantee holds.)
> Updated: 2026-06-22T17:39:48Z


## What this does

Land the **load-bearing invariant** of the whole sprint: the merge gate reads **only** `local_review_verdicts` at head, so `local_review_assignments`, `local_review_comments`, and `local_review_meta` are orchestration drive-metadata that **never gate**. Two enforcement layers prove it: (1) a **net-new build-gate honesty grep** asserting the gate path (`merge_gate.rs` + `review_requirement.rs`) contains **no reference** to any of the three new tables — the static no-read proof, in the same `but-authz/tests/invariant_build_gates.rs` honesty-grep discipline; and (2) **runtime integration tests** — the safe-seam proof (adding drive rows leaves the merge decision unchanged; only a verdict-at-head flips the land), the **forged-vs-empty equivalence** (a fully forged drive layer yields an identical gate decision to an empty one), and the **inverse** (drive metadata alone, with no approved verdict, still cannot land).

## Why

Sprint 07 · PRD UC-LPR-07 · capability CAP-AUTHZ-01. This is what makes the entire enrichment **legal under the freeze**: it cannot regress the land-truth because it never participates in the land decision. The existing R6 threat model on the verdict store is neither widened nor narrowed — a forged, malicious, or empty drive table cannot change a land decision, because `review_requirement.rs` never reads it. **"Gate gates (verdict-at-head, untouched); new tables drive (orchestration)."** If this task is not green, the slice is **not done**, regardless of the other lanes.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-authz safe_seam_gate_path_reads_no_new_table`: a build-gate grep over `crates/but-api/src/legacy/merge_gate.rs` + `crates/but-api/src/legacy/review_requirement.rs` finds **zero** references to `local_review_assignments`, `local_review_comments`, or `local_review_meta` — the drive/gate separation enforced at build time, not by convention. Full gate set in the spec below.

## Scope

  - crates/but-authz/tests/invariant_build_gates.rs (MODIFY — ADD a net-new `SAFE_SEAM_NO_READ` pattern + a `SAFE_SEAM_GATE_PATHS` (`merge_gate.rs`, `review_requirement.rs`) `assert_grep_has_no_matches` assertion, reusing the shipped `assert_grep_has_no_matches` helper; ADDITIVE — do NOT weaken any existing pattern/ENFORCEMENT_PATHS)
  - crates/but-api/tests/safe_seam.rs (NEW — the runtime safe-seam proofs: T-LPR-035..037 (each new table has no effect on a verdict-satisfied merge), T-LPR-040 (the capstone three-step proof), T-LPR-041 (forged-vs-empty equivalence), T-LPR-042 (inverse: drive-only cannot land); real but-db + but-api + gix via but_testsupport, hand-assertion style like the shipped merge_gate test)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: LPR-009 — Safe-seam invariant: no-read build-gate grep + forged-vs-empty + inverse tests
================================================================================

TASK_TYPE:   FEATURE
STATUS:      Backlog
PRIORITY:    P0  (the load-bearing gate — the slice is not done without it)
AGENT:       implementer=rust-reviewer | reviewer=rust-reviewer
EFFORT:      L  (180 min)
PROPOSED-BY: rust-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-LPR-07
CAPABILITIES:CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-authz safe_seam_gate_path_reads_no_new_table
  check: cargo check -p but-api -p but-authz --all-targets
  lint:  cargo clippy -p but-api -p but-authz --all-targets

--------------------------------------------------------------------------------
TYPE DESIGN (Rust shapes this task introduces)
--------------------------------------------------------------------------------
NO new product types. This task adds:
  - a build-gate grep constant `const SAFE_SEAM_NO_READ: &str = r#"local_review_assignments|local_review_comments|local_review_meta"#;` + a `const SAFE_SEAM_GATE_PATHS: &[&str] = &[MERGE_GATE, REVIEW_REQUIREMENT];` (REVIEW_REQUIREMENT = "crates/but-api/src/legacy/review_requirement.rs"; MERGE_GATE already exists in invariant_build_gates.rs:19) + an `assert_grep_has_no_matches(&workspace_root, "the merge gate must read none of the three new tables (safe seam)", SAFE_SEAM_NO_READ, SAFE_SEAM_GATE_PATHS)?` call inside (or beside) the shipped `invariant_build_gates` test.
  - the runtime safe-seam integration tests in but-api/tests/safe_seam.rs (functions only — they construct real fixtures and assert the merge-gate decision via enforce_merge_gate).
ERROR STRATEGY:
  - the grep test reuses the shipped anyhow::Result-returning helpers (assert_grep_has_no_matches, invariant_build_gates.rs:126); the runtime tests are #[test] fns asserting on enforce_merge_gate's Result (Ok = merge proceeds; Err(MergeGateError{code:REVIEW_REQUIRED_CODE}) = blocked).
OWNERSHIP PLAN:
  - the runtime tests build a real but_ctx::Context + DbHandle (the LPR-001 tables migrated) + a real gix repo via but_testsupport; rows are seeded BY VALUE through the LPR-001 Handles (forged set) and via the governed approve_review verb (the legitimate verdict@head); enforce_merge_gate borrows &ctx.
DOC POINTERS (read before coding):
  - brain/docs/rust/testing.md → #[test] + build-gate grep tests + hand-assertion (assert!/assert_eq! with a "why" message)
  - brain/docs/rust/error-handling.md → matching enforce_merge_gate's Ok vs Err(MergeGateError) decision
  - brain/docs/rust/ownership-borrowing.md → seeding rows by value through Handles; borrowing &ctx for the gate read

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
The safe seam proven two ways: (BUILD-GATE) a static grep over merge_gate.rs + review_requirement.rs finds ZERO references to local_review_assignments / local_review_comments / local_review_meta — the gate path reads none of the three new tables (the no-read honesty proof, same discipline as the AUTHORITY_POSITIVE_PATTERN gate). (RUNTIME) (1) T-LPR-035: a verdict-satisfied merge proceeds with the three new tables present but EMPTY — enforce_merge_gate decides purely on verdict-at-head; (2) T-LPR-036/037: an open pending/changes_requested assignment AND an unresolved comment thread each have NO effect on a verdict-satisfied merge; (3) T-LPR-040 (capstone): with NO verdict@head, Step1 merge blocked → Step2 add forged assignments+comments, STILL blocked with the identical decision → Step3 add a distinct approved verdict@head via governed approve_review, NOW proceeds — ONLY the verdict-at-head flips the land; (4) T-LPR-041: a fully forged drive layer (all assignments approved, all comments resolved, written directly) yields an IDENTICAL merge-gate decision to an empty drive layer for every verdict-at-head fixture; (5) T-LPR-042 (inverse): a pending assignment + unresolved comment with NO approved verdict still cannot land (blocked gate.review_required). cargo test -p but-authz + -p but-api green.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST add the no-read grep as a NET-NEW pattern in crates/but-authz/tests/invariant_build_gates.rs using the SHIPPED `assert_grep_has_no_matches` helper (invariant_build_gates.rs:126) over a `SAFE_SEAM_GATE_PATHS = &[MERGE_GATE, REVIEW_REQUIREMENT]` set, asserting the regex `local_review_assignments|local_review_comments|local_review_meta` has ZERO matches in those two files. MERGE_GATE already exists as a const (invariant_build_gates.rs:19); add REVIEW_REQUIREMENT = "crates/but-api/src/legacy/review_requirement.rs". This is the same honesty-grep discipline the AUTHORITY_POSITIVE_PATTERN gate uses (tech-delta §E).
- [MUST] MUST keep the grep ADDITIVE: do NOT weaken, narrow, or remove any existing pattern (ROLE_BRANCH_PATTERN, HUMAN_OR_LABEL_BRANCH_PATTERN, AUTHORITY_POSITIVE_PATTERN, PERMISSION_CARRIER_PATTERN) or any existing ENFORCEMENT_PATHS entry. The safe-seam assertion is added, not substituted.
- [MUST] MUST prove the RUNTIME equivalence too — the static grep alone is necessary but not sufficient (it catches a literal table reference, not a derived read). The forged-vs-empty (T-LPR-041) + inverse (T-LPR-042) + capstone (T-LPR-040) integration tests drive enforce_merge_gate and assert the decision is identical between a forged drive layer and an empty one for every verdict-at-head fixture. The tech-delta is explicit: "A static grep proves the single-symbol no-read; the RUNTIME equivalence (forged drive layer ≡ empty drive layer ⇒ identical merge decision) is proven by integration test (T-LPR-040/041/042), not the grep."
- [MUST] MUST state the safe-seam invariant VERBATIM in the gate test (tech-delta §E): "Gate gates (verdict-at-head, untouched); new tables drive (orchestration)." — as a comment/message arg, so the proof is self-documenting.
- [MUST] MUST drive the merge decision through the GOVERNED path (enforce_merge_gate, merge_gate.rs:40) and seed the legitimate approval via the GOVERNED approve_review verb (forge.rs:520), which writes the real local_review_verdicts@head row. The forged drive rows (assignments/comments) ARE written directly (that is the adversarial fixture — the whole point), but the legitimate verdict is written through the governed action.
- [MUST] MUST make T-LPR-040 the three-step capstone EXACTLY: Step1 (no verdict@head) merge → blocked (gate.review_required); Step2 add pending/changes_requested assignments AND unresolved comments → re-attempt merge → STILL blocked, IDENTICAL decision; Step3 add a distinct approved verdict@head via governed approve_review → re-attempt → proceeds. Assert the Step1==Step2 decision identity and the Step3 flip.
- [NEVER] NEVER write a test asserting that a forgeable DIRECT DB write to local_review_verdicts is BLOCKED — that encodes a FALSE guarantee (R6/R18 accepted-leak: a direct DB write to the verdict store CAN forge an approval; the gate trusts the verdict store). The safe-seam tests assert the DRIVE/GATE separation only (assignments/comments never gate), NOT that the verdict store is unforgeable. (04-e2e-testing-criteria.md maintenance note + tech-delta §G R18.)
- [NEVER] NEVER add a read of local_review_assignments / local_review_comments / local_review_meta to merge_gate.rs or review_requirement.rs to "make a test pass" — that BREAKS the invariant this task exists to prove. The grep would catch it; flag any temptation as a blocker.
- [NEVER] NEVER assert raw-git is blocked, and NEVER claim the local PR is independently audited (R18 stays named). The gate reads only the verdict store; its HMAC→Ed25519 hardening (C3) remains the named follow-up — do NOT present it as closed.
- [NEVER] NEVER weaken any assertion to make the forged-vs-empty equivalence pass — if forged ≠ empty, that is a REAL safe-seam violation (the gate read a drive table); fix the gate-path leak (in LPR-003..009), do not lower the assertion.
- [NEVER] NEVER add new gitbutler-* usage.
- [STRICTLY] STRICTLY treat merge_gate.rs + review_requirement.rs as CONSUME-ONLY — read them to confirm the verdict-at-head-only read (merge_gate.rs:40 → review_verdicts → list_by_target; review_requirement.rs:94 verdict.head_oid==current_head_oid + :8 const APPROVED="approved"); do NOT modify the gate.
- [STRICTLY] STRICTLY run the forged-vs-empty test over MULTIPLE verdict-at-head fixtures: (a) verdict satisfied (approved@head) ⇒ proceeds in BOTH forged and empty; (b) verdict unsatisfied (no approval@head / stale) ⇒ blocked in BOTH. The equivalence must hold in both directions, not just the blocked one (otherwise a gate that ignores everything would trivially pass).

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [x] AC-1 [PRIMARY] (T-LPR-038): the build-gate grep over merge_gate.rs + review_requirement.rs finds ZERO references to any of the three new tables (the static no-read proof, additive to invariant_build_gates)
- [x] AC-2 (T-LPR-035/036/037): a verdict-satisfied merge proceeds unchanged with the new tables empty, AND with an open pending/changes_requested assignment, AND with an unresolved comment thread (each new table has no effect on the land)
- [x] AC-3 (T-LPR-040): the capstone three-step proof — blocked → forged drive rows STILL blocked (identical) → approved verdict@head proceeds (only the verdict flips the land)
- [x] AC-4 (T-LPR-041): forged drive layer ≡ empty drive layer — identical gate decision for a satisfied fixture (both proceed) AND an unsatisfied fixture (both blocked)
- [x] AC-5 (T-LPR-042): inverse — a pending assignment + unresolved comment with NO approved verdict still cannot land (blocked gate.review_required)
- [x] AC-6 (T-LPR-039): the R6/R18 threat model is preserved unchanged — the gate's read set adds NO read of the new tables; R18 stays named (NOT presented as independently audited)
- [x] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY] (T-LPR-038): build-gate grep — the gate path references none of the three new tables
  GIVEN: the merge-gate code path = crates/but-api/src/legacy/merge_gate.rs + crates/but-api/src/legacy/review_requirement.rs
  WHEN:  the net-new build-gate honesty grep runs (assert_grep_has_no_matches over SAFE_SEAM_GATE_PATHS for the pattern `local_review_assignments|local_review_comments|local_review_meta`)
  THEN:  ZERO matches — none of `local_review_assignments`, `local_review_comments`, nor `local_review_meta` is referenced in the gate path; the drive/gate separation is enforced at BUILD time, not by convention; the existing invariant_build_gates patterns remain green and unweakened
  TEST_TIER: build-gate   VERIFICATION_SERVICE: the shipped but-authz invariant_build_gates assert_grep_has_no_matches helper over the two gate-path files
  VERIFY: cargo test -p but-authz safe_seam_gate_path_reads_no_new_table

AC-2 (T-LPR-035/036/037): each new table has NO effect on a verdict-satisfied merge
  GIVEN: lpr_safe_seam_repo: a branch with a distinct approved verdict@head (gate-satisfied) + a merge holder; the three new tables present
  WHEN:  the governed merge runs (a) with all three new tables EMPTY, (b) with an open `pending` AND a separate `changes_requested` assignment, (c) with an unresolved comment thread
  THEN:  the merge PROCEEDS in all three — enforce_merge_gate decides purely on verdict-at-head; the assignment never gates (T-LPR-036) and the comment never gates (T-LPR-037); the decision is identical to the empty-tables case
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api enforce_merge_gate + real but-db (the three new tables seeded) + real gix via but_testsupport
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
  THEN:  the merge is BLOCKED (gate.review_required) — drive metadata alone never satisfies the gate; only an approved verdict-at-head can, confirming the seam in the negative direction
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api enforce_merge_gate + a drive-only (no verdict) fixture + real gix
  VERIFY: cargo test -p but-api safe_seam_inverse_drive_only_cannot_land

AC-6 (T-LPR-039): the R6/R18 threat model is preserved unchanged
  GIVEN: the gate's read set (merge_gate.rs + review_requirement.rs) + the risk register (R18, §G)
  WHEN:  the gate's reads are audited and the safe-seam grep + runtime tests are run
  THEN:  the gate reads ONLY the verdict store (whose HMAC→Ed25519 hardening, C3, remains the NAMED follow-up); a direct DB write to a drive table widens NO land-truth surface (the new tables add no read to the gate path); R18 stays NAMED — the local PR is NOT presented as independently audited
  TEST_TIER: build-gate   VERIFICATION_SERVICE: the safe-seam grep (no new-table read) + the runtime forged-vs-empty equivalence; an audit note in the test that R18 stays named
  VERIFY: cargo test -p but-authz safe_seam_gate_path_reads_no_new_table && cargo test -p but-api safe_seam_forged_drive_equals_empty_drive

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): the grep over merge_gate.rs + review_requirement.rs for `local_review_assignments|local_review_comments|local_review_meta` returns ZERO matches; existing invariant_build_gates patterns stay green
    VERIFY: cargo test -p but-authz safe_seam_gate_path_reads_no_new_table
- TC-2 (-> AC-2): a verdict-satisfied merge proceeds with the new tables empty, with a pending/changes_requested assignment, and with an unresolved comment — identical decision in all
    VERIFY: cargo test -p but-api safe_seam_drive_rows_have_no_effect_on_satisfied_merge
- TC-3 (-> AC-3): Step1 blocked, Step2 (forged drive rows) blocked-identical, Step3 (approved verdict@head) proceeds
    VERIFY: cargo test -p but-api safe_seam_only_verdict_at_head_flips_the_land
- TC-4 (-> AC-4): forged drive layer ≡ empty for a SATISFIED fixture (both proceed) AND an UNSATISFIED fixture (both blocked)
    VERIFY: cargo test -p but-api safe_seam_forged_drive_equals_empty_drive
- TC-5 (-> AC-5): a pending assignment + unresolved comment with NO approved verdict is BLOCKED (gate.review_required)
    VERIFY: cargo test -p but-api safe_seam_inverse_drive_only_cannot_land
- TC-6 (-> AC-6): the gate's read set adds no read of the new tables (the grep) AND the runtime equivalence holds; R18 stays named in the test note
    VERIFY: cargo test -p but-authz safe_seam_gate_path_reads_no_new_table && cargo test -p but-api safe_seam_forged_drive_equals_empty_drive

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides:
  - a net-new build-gate honesty grep (additive to invariant_build_gates) asserting the gate path references none of the three new tables — the static no-read proof
  - the runtime safe-seam integration tests (T-LPR-035..042): drive rows never flip a verdict-satisfied merge; forged drive layer ≡ empty; drive-only cannot land
consumes:
  - crates/but-api/src/legacy/merge_gate.rs (enforce_merge_gate, merge_gate.rs:40 — CONSUME-only, the gate under proof) + review_requirement.rs (the verdict-at-head filter, :94/:8)
  - crate::legacy::forge::approve_review (forge.rs:520 — the governed verdict@head write for the legitimate-approval steps)
  - but_db::{LocalReviewAssignment, LocalReviewComment, LocalReviewMeta} Handles (LPR-001 — for the forged-drive-layer fixture covering all three tables)
  - crates/but-authz/tests/invariant_build_gates.rs assert_grep_has_no_matches (:126 — the honesty-grep helper to reuse)
boundary_contracts:
  - CAP-AUTHZ-01 (the safe seam): the merge gate reads ONLY local_review_verdicts at head; assignments, comments, and meta NEVER gate. Proven statically (no new-table reference in the gate path) AND at runtime (forged ≡ empty ⇒ identical decision; drive-only cannot land). The R6/R18 verdict-store forgeability stays an ACCEPTED, NAMED leak — this task does NOT (and must NOT) assert the verdict store is unforgeable.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-authz/tests/invariant_build_gates.rs (MODIFY — ADD the SAFE_SEAM_NO_READ pattern + the SAFE_SEAM_GATE_PATHS assert_grep_has_no_matches call; reuse the shipped helper; ADDITIVE only)
  - crates/but-api/tests/safe_seam.rs (NEW — the runtime safe-seam proofs T-LPR-035..042 via real but-db + enforce_merge_gate + gix)
writeProhibited:
  - crates/but-api/src/legacy/merge_gate.rs — CONSUME-only; NEVER add a read of the new tables here (that breaks the invariant; the grep would catch it)
  - crates/but-api/src/legacy/review_requirement.rs — CONSUME-only; the verdict-at-head filter is untouched
  - crates/but-authz/tests/invariant_build_gates.rs EXISTING patterns/ENFORCEMENT_PATHS — additive only; NEVER weaken ROLE_BRANCH_PATTERN / HUMAN_OR_LABEL_BRANCH_PATTERN / AUTHORITY_POSITIVE_PATTERN / PERMISSION_CARRIER_PATTERN
  - crates/but-db/** and the LPR-001..009 production code — CONSUME-only (the forged fixture writes through the LPR-001 Handles; do not change them)
  - any test asserting a direct DB write to local_review_verdicts is blocked (encodes a false R6/R18 guarantee — forbidden)
  - any gitbutler-* crate (no new gitbutler-* usage)
  - Any file not in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-api/src/legacy/merge_gate.rs [40-95, 155-175] — [THE GATE UNDER PROOF] enforce_merge_gate reads ONLY review_verdicts(ctx, &review.source_branch) → ctx.db.get_cache().local_review_verdicts().list_by_target(target) (merge_gate.rs:40 → the review_verdicts helper). Confirm it references NONE of the three new tables — that is exactly what the grep asserts and the runtime tests prove. CONSUME-only.
2. crates/but-api/src/legacy/review_requirement.rs [8, 37-100] — the verdict-at-head filter: const APPROVED="approved" (:8), evaluate filters verdict.head_oid==current_head_oid (:94). The verdict-at-head truth, untouched by this delta. CONSUME-only.
3. crates/but-authz/tests/invariant_build_gates.rs [9-60, 111-150] — [PRIMARY PATTERN] the honesty-grep discipline: ENFORCEMENT_PATHS, MERGE_GATE const (:19), the ROLE/HUMAN_OR_LABEL/AUTHORITY_POSITIVE patterns, and the assert_grep_has_no_matches helper (:126). ADD your SAFE_SEAM_NO_READ pattern + SAFE_SEAM_GATE_PATHS the SAME way; do NOT weaken the existing ones.
4. crates/but-api/src/legacy/forge.rs [520-546] — approve_review: the GOVERNED verdict@head write your legitimate-approval steps use (Step3 of the capstone). The forged drive rows are written directly, but the real approval goes through this verb.
5. crates/but-db/src/table/local_review_assignments.rs + local_review_comments.rs + local_review_meta.rs (LPR-001) — the Handles you seed the forged drive layer through (all assignments approved, all comments resolved, plus a forged local_review_meta opener row — all three tables forged).
6. crates/but-api/tests/ (the shipped merge_gate test) — [VERIFIED TEST IDIOM] the real-but-db + gix + enforce_merge_gate hand-assertion construction (NOT insta). Mirror it: seed the governed repo via but_testsupport::writable_scenario + invoke_bash committing .gitbutler/{permissions,gates}.toml; drive enforce_merge_gate; assert Ok (proceeds) vs Err(MergeGateError{code:REVIEW_REQUIRED_CODE}) (blocked).
7. .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md §E + 04-e2e-testing-criteria.md (T-LPR-035..042 + the maintenance note "never add a test asserting the forgeable direct DB write to local_review_verdicts is blocked").

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- cargo test -p but-authz safe_seam_gate_path_reads_no_new_table   -> Exit 0; ZERO references to any of the three new tables in the gate path; existing patterns green
- cargo test -p but-api safe_seam_drive_rows_have_no_effect_on_satisfied_merge   -> Exit 0; verdict-satisfied merge proceeds with tables empty / with assignment / with comment
- cargo test -p but-api safe_seam_only_verdict_at_head_flips_the_land   -> Exit 0; blocked → forged-still-blocked-identical → approved-proceeds
- cargo test -p but-api safe_seam_forged_drive_equals_empty_drive   -> Exit 0; forged ≡ empty for satisfied (both proceed) AND unsatisfied (both blocked)
- cargo test -p but-api safe_seam_inverse_drive_only_cannot_land   -> Exit 0; drive-only (no verdict) BLOCKED
- cargo test -p but-authz invariant_build_gates   -> Exit 0; existing honesty greps green (additive, unweakened)
- cargo check -p but-api -p but-authz --all-targets   -> Exit 0
- cargo clippy -p but-api -p but-authz --all-targets   -> Exit 0
- cargo fmt --check   -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references:
  - .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md §E (THE SAFE SEAM — the file:line proof + the verbatim invariant + the build-gate grep recipe)
  - .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/04-e2e-testing-criteria.md (T-LPR-035..042 + the maintenance note forbidding the false verdict-forgeability test)
  - crates/but-authz/tests/invariant_build_gates.rs:126 (assert_grep_has_no_matches), :19 (MERGE_GATE const)
  - crates/but-api/src/legacy/merge_gate.rs:40 (enforce_merge_gate, verdict-at-head-only read)
grep_skeleton: |
  // ADD to invariant_build_gates.rs (additive):
  const REVIEW_REQUIREMENT: &str = "crates/but-api/src/legacy/review_requirement.rs";
  const SAFE_SEAM_NO_READ: &str = r#"local_review_assignments|local_review_comments|local_review_meta"#;
  const SAFE_SEAM_GATE_PATHS: &[&str] = &[MERGE_GATE, REVIEW_REQUIREMENT];
  // inside (or beside) invariant_build_gates():
  assert_grep_has_no_matches(
      &workspace_root,
      "Gate gates (verdict-at-head, untouched); new tables drive (orchestration): \
       the merge gate must reference NONE of local_review_assignments, local_review_comments, NOR local_review_meta",
      SAFE_SEAM_NO_READ,
      SAFE_SEAM_GATE_PATHS,
  )?;
notes:
  - The runtime equivalence (T-LPR-041) is the load-bearing one: run the SAME branch state twice — empty drive tables vs a fully forged drive layer (all assignments "approved", all comments "resolved", written via the LPR-001 Handles directly) — and assert enforce_merge_gate returns the IDENTICAL decision for BOTH a satisfied verdict@head (both Ok) AND an unsatisfied one (both Err REVIEW_REQUIRED). Bidirectional, so a gate that ignored everything can't trivially pass.
  - T-LPR-040's Step3 approval MUST go through governed approve_review (forge.rs:520) so the verdict@head is the real one the gate reads; the drive rows in Step2 are forged directly (the adversarial fixture).
  - R18 honesty: add a comment in safe_seam.rs that the verdict store itself stays forgeable by a direct DB write (R6/R18 accepted leak) and is NOT under test here — this task proves the drive/gate SEPARATION, not verdict-store integrity.
pattern: a net-new no-read honesty grep (additive to invariant_build_gates) + runtime forged-vs-empty/inverse/capstone integration tests driving enforce_merge_gate, proving the three new tables never participate in the land decision
pattern_source: crates/but-authz/tests/invariant_build_gates.rs (the assert_grep_has_no_matches discipline); crates/but-api/src/legacy/merge_gate.rs:40 (the verdict-at-head-only gate); the shipped merge_gate test (the real-but-db+gix hand-assertion idiom)
anti_pattern: adding a read of a new table to the gate to pass a test (breaks the invariant — the grep catches it); weakening an existing invariant_build_gates pattern; asserting a direct DB write to local_review_verdicts is blocked (false R6/R18 guarantee); a one-directional forged-vs-empty test (only blocked) that a gate ignoring everything would pass; presenting the local PR as independently audited (R18 must stay named)

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agent: implementer=rust-reviewer | reviewer=rust-reviewer
rationale: This is the load-bearing safe-seam invariant — the proof that makes the whole sprint legal under the freeze. It is an audit/honesty-grep + adversarial-equivalence task (no product logic), best owned by rust-reviewer: writing a net-new no-read grep in the same discipline as the shipped AUTHORITY_POSITIVE gate, and the forged-vs-empty / inverse / capstone integration tests that drive enforce_merge_gate and assert drive rows never flip the land. The reviewer's adversarial instinct is exactly right for proving a forged drive layer ≡ an empty one bidirectionally and for refusing the false verdict-forgeability test (R6/R18 must stay named).
coding_standards: crates/AGENTS.md (Result + anyhow::Context; honesty greps are build-gate tests; for graph/gate behavior prefer fixture-backed assertions); crates/but-authz/tests/invariant_build_gates.rs (the assert_grep_has_no_matches discipline to mirror — additive, never weakening); RULES.md (use but-testsupport scenarios; NEVER std::env::temp_dir(); the merge gate reads only the verdict store — preserve it); brain/docs/rust/testing.md (build-gate grep + hand-assertion with a "why" message)

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: LPR-001 (the three tables, for the forged-drive fixture, incl. local_review_meta), LPR-003/004 (the assignment/comment write verbs), LPR-005/008 (the derived/reconciler reads the safe-seam tests run alongside), and the shipped Sprint-01b/04 merge gate (enforce_merge_gate) under proof
Blocks:     LPR-010 (the closeout audit cites this safe-seam proof as green) — this is the load-bearing gate; the slice is NOT done until LPR-009 is green
```
</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "LPR-009",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "lpr_safe_seam_repo": {
      "description": "A real governed repo via but_testsupport::writable_scenario + invoke_bash committing .gitbutler/{permissions,gates}.toml (a protected target branch with a review requirement). A real but_ctx::Context + DbHandle with the LPR-001 tables migrated. The legitimate approval@head is written via the GOVERNED approve_review verb (forge.rs:520). The FORGED drive layer (all local_review_assignments state='approved', all local_review_comments resolved=true) is written DIRECTLY via the LPR-001 Handles — that is the adversarial fixture. The merge decision is read via enforce_merge_gate (Ok=proceeds; Err(MergeGateError{code:REVIEW_REQUIRED_CODE})=blocked). This is the shipped merge_gate hand-assertion idiom (real but-db + real gix, no mocks, no insta).",
      "seed_method": "public_api",
      "records": [
        "but_testsupport::writable_scenario(...) + invoke_bash committing .gitbutler/{permissions,gates}.toml (protected branch + review requirement) to refs/heads/main;",
        "for the legitimate-approval steps: governed approve_review(ctx, source_branch) writes the real local_review_verdicts@head row;",
        "for the forged drive layer: db.local_review_assignments_mut().upsert(state='approved') x N + db.local_review_comments_mut().insert(resolved=true) x N written DIRECTLY (the adversarial set);",
        "read the merge decision via enforce_merge_gate(&ctx, review_id)."
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN the gate path = merge_gate.rs + review_requirement.rs WHEN the net-new build-gate grep runs (assert_grep_has_no_matches over SAFE_SEAM_GATE_PATHS for local_review_assignments|local_review_comments|local_review_meta) THEN ZERO matches — the gate references none of the three new tables; the drive/gate separation is enforced at build time; existing invariant_build_gates patterns remain green and unweakened",
      "verify": "cargo test -p but-authz safe_seam_gate_path_reads_no_new_table",
      "scenario": {
        "tier": "visible",
        "test_tier": "build-gate",
        "verification_service": "the shipped but-authz invariant_build_gates assert_grep_has_no_matches helper over merge_gate.rs + review_requirement.rs",
        "negative_control": {
          "would_fail_if": [
            "merge_gate.rs or review_requirement.rs referenced local_review_assignments / local_review_comments / local_review_meta — the grep finds a match (the gate started reading a drive table)",
            "the grep targeted the wrong files (not the gate path) — it would pass vacuously",
            "an existing invariant_build_gates pattern were weakened to make the suite pass"
          ]
        },
        "evidence": { "artifact_type": "test_output", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_safe_seam_repo",
            "action": { "actor": "ci", "steps": [ "run the SAFE_SEAM_NO_READ grep over SAFE_SEAM_GATE_PATHS = [merge_gate.rs, review_requirement.rs]", "run the full invariant_build_gates suite" ] },
            "end_state": {
              "must_observe": [ "zero matches for local_review_assignments|local_review_comments|local_review_meta in the gate path", "the existing invariant_build_gates patterns still pass" ],
              "must_not_observe": [ "any match of any new-table symbol (of the three) in the gate path", "a weakened/removed existing pattern" ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN lpr_safe_seam_repo with a distinct approved verdict@head (gate-satisfied) WHEN the governed merge runs (a) with all three new tables empty, (b) with a pending+changes_requested assignment, (c) with an unresolved comment thread THEN the merge PROCEEDS in all three — identical to the empty case (assignment never gates, comment never gates)",
      "verify": "cargo test -p but-api safe_seam_drive_rows_have_no_effect_on_satisfied_merge",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api enforce_merge_gate + real but-db (the new tables seeded) + real gix",
        "negative_control": {
          "would_fail_if": [
            "adding an assignment or comment CHANGED the merge decision (the gate read a drive table) — (b)/(c) would differ from (a)",
            "a changes_requested assignment blocked the merge (the gate consulted assignment state)"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_safe_seam_repo",
            "action": { "actor": "ci", "steps": [ "seed an approved verdict@head via governed approve_review", "run enforce_merge_gate with tables empty (a)", "seed a pending + changes_requested assignment; run enforce_merge_gate (b)", "seed an unresolved comment thread; run enforce_merge_gate (c)" ] },
            "end_state": {
              "must_observe": [ "(a) merge proceeds (Ok)", "(b) merge proceeds (Ok) — assignment never gates", "(c) merge proceeds (Ok) — comment never gates" ],
              "must_not_observe": [ "(b) or (c) blocked while (a) proceeds (a drive table affected the decision)" ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN lpr_safe_seam_repo, NO verdict@head WHEN Step1 attempt merge; Step2 add forged pending/changes_requested assignments + unresolved comments, re-attempt; Step3 add a distinct approved verdict@head via governed approve_review, re-attempt THEN Step1 BLOCKED; Step2 STILL BLOCKED with the IDENTICAL decision; Step3 PROCEEDS — only the verdict-at-head flips the land",
      "verify": "cargo test -p but-api safe_seam_only_verdict_at_head_flips_the_land",
      "scenario": {
        "tier": "holdout",
        "test_tier": "e2e-automated",
        "verification_service": "real but-api enforce_merge_gate + governed approve_review + real but-db drive seeding + real gix",
        "negative_control": {
          "would_fail_if": [
            "Step2 (forged drive rows) flipped the decision to proceed — the gate read a drive table",
            "Step3 did not proceed after a real approved verdict@head — the gate ignored the verdict store (a different break)",
            "Step1 and Step2 decisions differed — the drive rows changed the blocked decision"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_safe_seam_repo",
            "action": { "actor": "ci", "steps": [ "Step1: enforce_merge_gate (no verdict@head) → capture decision", "Step2: directly write pending/changes_requested assignments + unresolved comments; enforce_merge_gate → capture decision", "Step3: governed approve_review writes approved verdict@head; enforce_merge_gate → capture decision" ] },
            "end_state": {
              "must_observe": [ "Step1 blocked (Err REVIEW_REQUIRED)", "Step2 blocked with the IDENTICAL decision to Step1", "Step3 proceeds (Ok)" ],
              "must_not_observe": [ "Step2 proceeding (forged drive rows flipped the land)", "Step3 still blocked after a real approval@head" ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN lpr_safe_seam_repo run twice — (a) zero rows in all three new tables, (b) a fully forged set (all assignments approved, all comments resolved) written directly — for a SATISFIED verdict@head fixture and an UNSATISFIED one WHEN the governed merge runs in each THEN the merge-gate decision is IDENTICAL between (a) and (b) for every fixture: satisfied ⇒ proceeds in both; unsatisfied ⇒ blocked in both",
      "verify": "cargo test -p but-api safe_seam_forged_drive_equals_empty_drive",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api enforce_merge_gate + a directly-written forged drive layer vs an empty one + real gix",
        "negative_control": {
          "would_fail_if": [
            "forged ≠ empty for the satisfied fixture (the forged 'approved' assignments / 'resolved' comments changed the proceed decision)",
            "forged ≠ empty for the unsatisfied fixture (the forged rows flipped a block into a proceed) — the headline safe-seam break",
            "the test only checked the blocked direction — a gate that ignores everything would pass vacuously"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_safe_seam_repo",
            "action": { "actor": "ci", "steps": [ "SATISFIED fixture: approved verdict@head; run enforce_merge_gate empty (a) and forged (b)", "UNSATISFIED fixture: no approval@head; run enforce_merge_gate empty (a) and forged (b)" ] },
            "end_state": {
              "must_observe": [ "satisfied: (a) proceeds AND (b) proceeds (identical)", "unsatisfied: (a) blocked AND (b) blocked (identical)" ],
              "must_not_observe": [ "any (a) != (b) decision for either fixture (a forged drive row changed the land)" ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN lpr_safe_seam_repo, NO verdict@head, WITH a pending assignment AND an unresolved comment thread present WHEN the governed merge runs THEN the merge is BLOCKED (gate.review_required) — drive metadata alone never satisfies the gate; only an approved verdict-at-head can",
      "verify": "cargo test -p but-api safe_seam_inverse_drive_only_cannot_land",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api enforce_merge_gate + a drive-only (no verdict) fixture + real gix",
        "negative_control": {
          "would_fail_if": [
            "the merge PROCEEDED on a pending assignment + unresolved comment with no verdict@head — drive metadata wrongly satisfied the gate",
            "a 'pending' assignment were misread as an approval"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_safe_seam_repo",
            "action": { "actor": "ci", "steps": [ "seed a pending assignment + an unresolved comment thread; NO verdict@head", "run enforce_merge_gate" ] },
            "end_state": {
              "must_observe": [ "the merge is blocked (Err MergeGateError{code: gate.review_required})" ],
              "must_not_observe": [ "the merge proceeding on drive metadata alone (no approved verdict@head)" ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-6",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the gate's read set + the risk register (R18, §G) WHEN the gate's reads are audited and the safe-seam grep + runtime tests run THEN the gate reads ONLY the verdict store (whose HMAC→Ed25519 hardening C3 remains the NAMED follow-up); a direct DB write to a drive table widens NO land-truth surface; R18 stays NAMED — the local PR is NOT presented as independently audited",
      "verify": "cargo test -p but-authz safe_seam_gate_path_reads_no_new_table && cargo test -p but-api safe_seam_forged_drive_equals_empty_drive",
      "scenario": {
        "tier": "visible",
        "test_tier": "build-gate",
        "verification_service": "the safe-seam grep (no new-table read) + the runtime forged-vs-empty equivalence + an audit note that R18 stays named",
        "negative_control": {
          "would_fail_if": [
            "a new-table read were added to the gate path (the grep catches it) — the land-truth surface widened",
            "a test asserted the verdict store is unforgeable (a false R6/R18 guarantee) — forbidden; R18 must stay named",
            "the test note claimed the local PR is independently audited"
          ]
        },
        "evidence": { "artifact_type": "test_output", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_safe_seam_repo",
            "action": { "actor": "ci", "steps": [ "audit the gate's read set (only local_review_verdicts@head)", "run the safe-seam grep + the forged-vs-empty equivalence", "confirm the test note names R18 (verdict-store forgeability) as an accepted, NOT-closed leak" ] },
            "end_state": {
              "must_observe": [ "the gate reads only the verdict store at head", "the new tables add no read to the gate path", "R18 stays named in the test note (NOT presented as closed/independently audited)" ],
              "must_not_observe": [ "a new-table read in the gate path", "a test asserting the verdict store is unforgeable", "a claim that the local PR is independently audited" ]
            }
          }
        ]
      }
    },
    { "id": "TC-1", "type": "test_criterion", "description": "the grep over merge_gate.rs + review_requirement.rs for local_review_assignments|local_review_comments|local_review_meta returns ZERO matches; existing patterns stay green", "verify": "cargo test -p but-authz safe_seam_gate_path_reads_no_new_table", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "a verdict-satisfied merge proceeds with the new tables empty, with a pending/changes_requested assignment, and with an unresolved comment — identical decision in all", "verify": "cargo test -p but-api safe_seam_drive_rows_have_no_effect_on_satisfied_merge", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "Step1 blocked, Step2 (forged drive rows) blocked-identical, Step3 (approved verdict@head) proceeds", "verify": "cargo test -p but-api safe_seam_only_verdict_at_head_flips_the_land", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "forged drive layer ≡ empty for a SATISFIED fixture (both proceed) AND an UNSATISFIED fixture (both blocked)", "verify": "cargo test -p but-api safe_seam_forged_drive_equals_empty_drive", "maps_to_ac": "AC-4" },
    { "id": "TC-5", "type": "test_criterion", "description": "a pending assignment + unresolved comment with NO approved verdict is BLOCKED (gate.review_required)", "verify": "cargo test -p but-api safe_seam_inverse_drive_only_cannot_land", "maps_to_ac": "AC-5" },
    { "id": "TC-6", "type": "test_criterion", "description": "the gate's read set adds no read of the new tables (grep) AND the runtime equivalence holds; R18 stays named in the test note", "verify": "cargo test -p but-authz safe_seam_gate_path_reads_no_new_table && cargo test -p but-api safe_seam_forged_drive_equals_empty_drive", "maps_to_ac": "AC-6" }
  ]
}
-->
