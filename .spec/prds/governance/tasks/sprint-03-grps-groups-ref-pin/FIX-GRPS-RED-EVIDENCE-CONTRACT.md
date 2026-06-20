# FIX-GRPS-RED-EVIDENCE-CONTRACT: Capture genuine mutation-based falsifiability evidence and amend GRPS-001/002 requires_red_evidence to a recorded waiver

## What this does

Closes the TDD-contract-honesty gap (red-hat finding **A**) by replacing the unmet `requires_red_evidence: true` flag on GRPS-001 and GRPS-002 with the *real* teeth proof — genuine mutation runs that flip the contract tests RED — and a recorded waiver explaining why a behavioral RED was legitimately impossible. GRPS-001 was a behavior-neutral refactor (removing a provably-redundant authorize-time re-union; `effective_authority(p) == principal_authorities(p)` by construction, authorize.rs:51-58); GRPS-002 was a capstone of integration tests over the pre-existing target-ref loader. Neither could exhibit a failing-then-passing behavioral RED, and that impossibility was honestly disclosed at completion — but the flag was never amended and no waiver was recorded. This task (1) produces `.tmp/GRPS-001/mutation-evidence.md` and `.tmp/GRPS-002/mutation-evidence.md` capturing ≥2 genuine mutations each that flip a NAMED contract test RED (with restored-clean confirmation), and (2) amends both REQUIREMENT-CONTRACT `verification_policy` blocks: `requires_red_evidence: false` + a `red_evidence_waiver` reason + a `falsifiability_substitute` pointer to the evidence file. It does NOT fabricate a behavioral RED — fabricating one would be the cardinal sin this task exists to avoid.

## Why

Sprint 03 remediation · red-hat finding **A** (MEDIUM, contract honesty) from `.spec/reviews/red-hat-sprint-03-2026-06-19.md:23-24,36,53-59`. The security-auditor panel raised the unmet/unwaived `requires_red_evidence` as a process-honesty issue; the rust-reviewer's mutation testing supplied the *substitute* evidence (pointing the loader at HEAD/source_ref or widening the effective set flips the tests RED). The honest resolution is to formalize that: record the mutation evidence as the documented falsifiability substitute and amend the contract to a recorded waiver, rather than leaving a true flag permanently unmet (which makes every future `/kb-run-sprint` over these tasks fail the RED-evidence gate) or — far worse — fabricating a behavioral RED. **Human acknowledgment required:** this records a waiver of `requires_red_evidence`; per policy it must be surfaced for human sign-off (see AC-4).

## How to verify

PRIMARY **AC-1** — both `.tmp/GRPS-001/mutation-evidence.md` and `.tmp/GRPS-002/mutation-evidence.md` exist, each documenting ≥2 genuine mutations that flip a named contract test RED, with restored-clean confirmation. This is an EVIDENCE task: verification is artifact presence + integrity (the mutations really flip the tests, and source is restored), not a new cargo test. Full gate set in the spec below.

## Scope

- .tmp/GRPS-001/mutation-evidence.md (NEW) — ≥2 genuine mutations flipping named GRPS-001 contract tests RED: (i) drop the config.rs group member-fold (319-332 / current 355-368) → group-only union tests fail "no permissions"; (ii) widen effective_authority with a spurious Merge → AC-2 equality pin + AC-4 claim tests fail. Each: mutation diff + command + observed RED output + restored-clean `git diff` confirmation.
- .tmp/GRPS-002/mutation-evidence.md (NEW) — ≥2 genuine mutations flipping named GRPS-002 contract tests RED: (iii) make the merge gate read source_ref instead of target_ref → self-escalation test wrongly clears merge authority / falls to wrong code; (iv) make the loader peel HEAD → the AC-4 / self-grant tests authorize wrongly. Each: mutation diff + command + observed RED output + restored-clean confirmation.
- .spec/prds/governance/tasks/sprint-03-grps-groups-ref-pin/GRPS-001-effective-set-union-group-ceiling.md (MODIFY, REQUIREMENT-CONTRACT JSON ONLY) — verification_policy: requires_red_evidence:false + red_evidence_waiver + falsifiability_substitute; keep requires_seeded_evidence + requires_tests as-is; do NOT touch any prose section or the TASK-TEMPLATE block
- .spec/prds/governance/tasks/sprint-03-grps-groups-ref-pin/GRPS-002-ref-pinned-membership-self-grant-inert.md (MODIFY, REQUIREMENT-CONTRACT JSON ONLY) — same amendment, pointing at .tmp/GRPS-002/mutation-evidence.md

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: FIX-GRPS-RED-EVIDENCE-CONTRACT - Capture mutation falsifiability evidence and amend GRPS-001/002 requires_red_evidence to a recorded waiver
================================================================================

TASK_TYPE:  EVIDENCE
STATUS:     Backlog
PRIORITY:   P1
EFFORT:     M  (120 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ./SPRINT.md
PRD_REFS:   UC-GRPS-01, UC-GRPS-02
CAPABILITIES: CAP-AUTHZ-01, CAP-CONFIG-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-authz group_union_authorizes_review_denies_merge   |   cargo test -p but-authz union_paths_stay_equal   |   cargo test -p but-authz claims_do_not_widen_union_even_with_group_backing   |   cargo test -p but-api self_add_to_maintainers_on_feature_head_still_denied_merge   |   cargo test -p but-authz membership_read_only_from_target_ref   |   cargo test -p but-authz self_grant_admin_inert_until_landed
  check: cargo check -p but-authz -p but-api --all-targets
  lint:  python3 -c "import json,sys; ..."  (JSON validity of both amended contracts; see VERIFICATION GATES)   |   fmt: cargo fmt --check (source must be restored to clean baseline after every mutation)

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Two mutation-evidence artifacts (.tmp/GRPS-001/mutation-evidence.md, .tmp/GRPS-002/mutation-evidence.md) each document >=2 GENUINE mutations that flip a NAMED GRPS contract test from GREEN to RED, with the mutation diff, the exact cargo command, the observed RED output (assertion-failure message, not a compile error unless that IS the falsification), and a restored-clean `git diff --stat` proving the mutation was reverted. Both GRPS-001 and GRPS-002 REQUIREMENT-CONTRACT verification_policy blocks are amended: requires_red_evidence:false, red_evidence_waiver:"behavior-neutral refactor (GRPS-001) / capstone over pre-existing target-ref enforcement (GRPS-002); behavioral RED genuinely impossible", falsifiability_substitute:".tmp/GRPS-00X/mutation-evidence.md". requires_seeded_evidence and requires_tests are unchanged. Both JSON blobs remain valid. The waiver is surfaced to the human for acknowledgment per policy. NO fabricated behavioral RED; all mutations are throwaway and restored; the working tree of crates/ is clean at task end (cargo fmt --check passes).

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST make every mutation GENUINE — a real edit to production source that really flips a named test RED, captured with the actual observed failure output. A "mutation" that leaves the test GREEN proves the test lacks teeth (record it as a FAILED falsification and pick a mutation that bites — do NOT pretend it flipped). Fabricating RED output, or claiming a mutation flipped a test it did not, is the cardinal stubbing sin (see ~/.claude/CLAUDE.md) and will be rejected.
- [MUST] MUST RESTORE every mutation to the clean baseline immediately after capturing its RED output (`git restore <file>` / `git checkout -- <file>`), and confirm `git diff --stat crates/` is EMPTY before moving to the next mutation. Leaving any mutation in source is the cardinal sin. Each evidence entry MUST include the restored-clean confirmation line.
- [MUST] MUST capture, per mutation: (1) the mutation diff (unified `git diff` of the source edit), (2) the exact `cargo test -p <crate> <test_name>` command, (3) the observed RED output (the assertion-failure panic message or the wrong-code assertion), (4) restored-clean confirmation. Four parts, every entry.
- [MUST] MUST amend ONLY the verification_policy object inside each REQUIREMENT-CONTRACT v1 JSON (GRPS-001 lines 213-218, GRPS-002 lines 226-231 in the current files). Set requires_red_evidence:false; ADD red_evidence_waiver (string) and falsifiability_substitute (string path). KEEP requires_tests:true and requires_seeded_evidence:true unchanged. Do NOT edit any prose section, the TASK-TEMPLATE block, fixtures, or requirements arrays.
- [MUST] MUST keep both amended JSON blobs VALID (parseable by `python3 -m json.tool`). The contract lives between `<!--` and `-->` after the `<!-- REQUIREMENT-CONTRACT v1 -->` marker; extract, edit, re-validate, re-insert preserving the exact comment-wrapper format.
- [NEVER] NEVER fabricate or embellish RED output. If a chosen mutation does not flip the test, that is a real signal — either pick a different mutation that genuinely bites the named test, or record the GREEN result honestly as evidence that THAT particular mutation is not caught (and still supply >=2 mutations that DO flip per file).
- [NEVER] NEVER leave a mutation in crates/ source. The end-state working tree for crates/ MUST be clean (cargo fmt --check passes, git diff empty for src files). The only persistent changes are the two NEW .tmp evidence files and the two contract-JSON amendments.
- [NEVER] NEVER set requires_red_evidence back to true, and NEVER remove requires_seeded_evidence — the waiver is specifically for the BEHAVIORAL-RED requirement, justified by the mutation substitute; seeded-evidence remains required.
- [STRICTLY] STRICTLY surface the waiver to the human. requires_red_evidence:false is a policy waiver; the task's final report MUST include a "HUMAN ACKNOWLEDGMENT REQUIRED" note naming both tasks, the waiver reason, and the falsifiability_substitute paths (AC-4). The waiver is recorded in the contract AND flagged upward — both, not either.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: .tmp/GRPS-001/mutation-evidence.md + .tmp/GRPS-002/mutation-evidence.md exist, each with >=2 genuine mutations flipping named contract tests RED (4 parts per entry, restored-clean confirmed)
- [ ] AC-2: both REQUIREMENT-CONTRACTs amended (requires_red_evidence:false + red_evidence_waiver + falsifiability_substitute; seeded/tests unchanged); both JSON valid
- [ ] AC-3: crates/ source is clean at task end (every mutation restored; cargo fmt --check passes; the named tests are GREEN again on the clean baseline)
- [ ] AC-4: the requires_red_evidence waiver is surfaced to the human for acknowledgment (named tasks + reason + substitute paths)

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — evidence-integrity first)
--------------------------------------------------------------------------------

AC-1: Genuine mutation evidence captured for both GRPS-001 and GRPS-002 [PRIMARY]
  GIVEN: the merged Sprint-03 source (crates/but-authz, crates/but-api/src/legacy/merge_gate.rs) at a clean baseline
  WHEN:  >=2 throwaway mutations per task are applied one at a time, the named contract test is run, the RED output is captured, and the mutation is restored
  THEN:  .tmp/GRPS-001/mutation-evidence.md documents (i) dropping the config.rs group.members member-fold (the loop at config.rs:355-368) => group_union_authorizes_review_denies_merge fails (reviewer-only no longer holds reviews:write => ReviewsWrite DENIED, the happy half panics) AND (ii) widening effective_authority with a spurious Authority::Merge (authorize.rs:51-58) => claims_do_not_widen_union_even_with_group_backing and/or union_paths_stay_equal fail; .tmp/GRPS-002/mutation-evidence.md documents (iii) making enforce_merge_gate read the source/feat ref instead of the target ref => self_add_to_maintainers_on_feature_head_still_denied_merge wrongly clears the Authority::Merge step (no longer perm.denied) AND (iv) making read_config_blob peel HEAD instead of target_ref => membership_read_only_from_target_ref and/or self_grant_admin_inert_until_landed authorize wrongly. Each entry has the 4 parts (diff, command, RED output, restored-clean).
  TEST_TIER: evidence   VERIFICATION_SERVICE: real but-authz/but-api tests run against transiently-mutated real source, restored clean after each
  VERIFY: test -f .tmp/GRPS-001/mutation-evidence.md && test -f .tmp/GRPS-002/mutation-evidence.md ; grep -c "RESTORED CLEAN" .tmp/GRPS-001/mutation-evidence.md (>=2) ; grep -c "RESTORED CLEAN" .tmp/GRPS-002/mutation-evidence.md (>=2)
  SCENARIO (negative controls): the evidence is INVALID if a documented "mutation" leaves the named test GREEN (the test would then lack teeth — the mutation must genuinely flip it RED); INVALID if any mutation is left unrestored in source (git diff for crates/ src must be empty); INVALID if the RED output is a compile error masquerading as a behavioral failure (unless the falsification IS a type-level guarantee, which must be stated); INVALID if fewer than 2 genuine flips per file

AC-2: Both REQUIREMENT-CONTRACTs amended to a recorded waiver; JSON valid
  GIVEN: the GRPS-001 and GRPS-002 task files with verification_policy.requires_red_evidence:true
  WHEN:  each verification_policy is edited
  THEN:  both read requires_red_evidence:false, add red_evidence_waiver:"behavior-neutral refactor (GRPS-001) / capstone over pre-existing target-ref enforcement (GRPS-002); behavioral RED genuinely impossible", add falsifiability_substitute:".tmp/GRPS-001/mutation-evidence.md" (resp. ".tmp/GRPS-002/mutation-evidence.md"); requires_tests:true and requires_seeded_evidence:true are unchanged; both JSON blobs parse cleanly
  TEST_TIER: evidence   VERIFICATION_SERVICE: JSON parse of the extracted REQUIREMENT-CONTRACT blocks
  VERIFY: python3 - <<'PY' (extract each contract between the REQUIREMENT-CONTRACT marker's <!-- ... --> and json.loads it; assert verification_policy.requires_red_evidence is False, red_evidence_waiver present, falsifiability_substitute present, requires_seeded_evidence True) PY
  SCENARIO (negative controls): would FAIL if either JSON became unparseable (malformed edit); would FAIL if requires_seeded_evidence were dropped or flipped; would FAIL if requires_red_evidence stayed true; would FAIL if a prose section or the TASK-TEMPLATE block was altered (out of scope — only the verification_policy object changes)

AC-3: crates/ source is clean at task end; named tests GREEN on the clean baseline
  GIVEN: all mutations applied and restored
  WHEN:  git status + cargo fmt --check + re-run of the named tests on the clean baseline
  THEN:  `git diff --stat crates/` shows NO source changes (only .tmp + .spec contract edits persist); `cargo fmt --check` passes; the named GRPS-001/002 contract tests are GREEN again
  TEST_TIER: evidence   VERIFICATION_SERVICE: git status + cargo fmt + cargo test on the restored baseline
  VERIFY: git diff --stat crates/ (empty for src) ; cargo fmt --check ; cargo test -p but-authz group_union_authorizes_review_denies_merge ; cargo test -p but-api self_add_to_maintainers_on_feature_head_still_denied_merge
  SCENARIO (negative controls): would FAIL if any mutation residue remained in crates/ src (git diff non-empty); would FAIL if a restored file did not return to fmt-clean; would FAIL if a named test stayed RED after restore (means the mutation was not fully reverted)

AC-4: the requires_red_evidence waiver is surfaced to the human for acknowledgment
  GIVEN: the amended contracts
  WHEN:  the task reports completion
  THEN:  the final report contains a "HUMAN ACKNOWLEDGMENT REQUIRED" section naming GRPS-001 and GRPS-002, stating requires_red_evidence was waived to false, the waiver reason, and the two falsifiability_substitute paths — so the human can sign off per policy
  TEST_TIER: evidence   VERIFICATION_SERVICE: presence of the acknowledgment note in the task completion report
  VERIFY: the completion report includes the HUMAN ACKNOWLEDGMENT REQUIRED note (named tasks + reason + substitute paths)
  SCENARIO (negative controls): would FAIL the honesty bar if the waiver were recorded in the contract but NOT surfaced to the human (silent policy change); would FAIL if the note omitted either task or the substitute paths

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, structural): .tmp/GRPS-001/mutation-evidence.md exists with >=2 entries each having diff + command + RED output + RESTORED CLEAN
    VERIFY: test -f .tmp/GRPS-001/mutation-evidence.md ; grep -c "RESTORED CLEAN" .tmp/GRPS-001/mutation-evidence.md
- TC-2 (-> AC-1, structural): .tmp/GRPS-002/mutation-evidence.md exists with >=2 entries each having diff + command + RED output + RESTORED CLEAN
    VERIFY: test -f .tmp/GRPS-002/mutation-evidence.md ; grep -c "RESTORED CLEAN" .tmp/GRPS-002/mutation-evidence.md
- TC-3 (-> AC-1, error): at least one documented mutation per file shows a genuine assertion-failure RED for a NAMED contract test (not a stayed-GREEN no-op)
    VERIFY: manual + grep for the named test + "FAILED"/"panicked"/"assertion `left == right` failed" in each evidence file
- TC-4 (-> AC-2, structural): GRPS-001 contract verification_policy.requires_red_evidence == false with red_evidence_waiver + falsifiability_substitute; requires_seeded_evidence == true; JSON valid
    VERIFY: python3 json.loads of the extracted GRPS-001 contract
- TC-5 (-> AC-2, structural): GRPS-002 contract verification_policy.requires_red_evidence == false with red_evidence_waiver + falsifiability_substitute; requires_seeded_evidence == true; JSON valid
    VERIFY: python3 json.loads of the extracted GRPS-002 contract
- TC-6 (-> AC-3, structural): git diff --stat crates/ is empty for src; cargo fmt --check passes; named tests GREEN
    VERIFY: git diff --stat crates/ ; cargo fmt --check ; cargo test -p but-authz group_union_authorizes_review_denies_merge
- TC-7 (-> AC-4, structural): the completion report contains the HUMAN ACKNOWLEDGMENT REQUIRED waiver note (both tasks + reason + substitute paths)
    VERIFY: report inspection

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01, CAP-CONFIG-01
provides: documented mutation-based falsifiability evidence as the recorded substitute for a (legitimately impossible) behavioral RED, and a contract amendment recording the requires_red_evidence waiver
consumes: the merged GRPS-001/002 source + contract tests; cargo test; git restore; python3 json
boundary_contracts:
  - TDD honesty: a task that cannot exhibit a behavioral RED (behavior-neutral refactor / capstone over existing enforcement) records the impossibility as a waiver with a falsifiability substitute (mutation evidence), never as a silent unmet flag and never as a fabricated RED. The substitute MUST be genuine mutations that really flip the contract tests.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - .tmp/GRPS-001/mutation-evidence.md (NEW) — >=2 genuine GRPS-001 mutations flipping named tests RED, each with diff + command + RED output + RESTORED CLEAN
  - .tmp/GRPS-002/mutation-evidence.md (NEW) — >=2 genuine GRPS-002 mutations flipping named tests RED, each with diff + command + RED output + RESTORED CLEAN
  - .spec/prds/governance/tasks/sprint-03-grps-groups-ref-pin/GRPS-001-effective-set-union-group-ceiling.md (MODIFY) — REQUIREMENT-CONTRACT verification_policy object ONLY (requires_red_evidence:false + waiver + substitute)
  - .spec/prds/governance/tasks/sprint-03-grps-groups-ref-pin/GRPS-002-ref-pinned-membership-self-grant-inert.md (MODIFY) — REQUIREMENT-CONTRACT verification_policy object ONLY
writeProhibited:
  - crates/** — mutations are THROWAWAY only; NO persistent source change. The working tree of crates/ MUST be clean at task end.
  - any prose section / TASK-TEMPLATE block / fixtures / requirements array of GRPS-001/002 — only the verification_policy object changes
  - the OTHER REQUIREMENT-CONTRACTs (the FIX-* tasks) — their requires_red_evidence stays true (they have real RED/falsifiability paths)
  - any gitbutler-* crate
  - Any file not in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. .spec/reviews/red-hat-sprint-03-2026-06-19.md (23-24, 36, 53-59)
   Focus: finding A — requires_red_evidence:true unmet/unwaived despite legitimately-impossible behavioral RED; the honest resolution is mutation-based falsifiability evidence + a recorded waiver, NOT a fabricated RED. The "Honesty note for human" (58-59) is the source of AC-4.
2. crates/but-authz/src/config.rs (303-371)
   Focus: normalize_permissions — the group.members member-fold loop (355-368) is mutation (i): deleting/short-circuiting it makes group-only members (reviewer-only, delegate) resolve to no authority. Also the principal.groups fold (338-347).
3. crates/but-authz/src/authorize.rs (51-58)
   Focus: effective_authority — mutation (ii): unioning a spurious Authority::Merge into the returned set widens the effective authority and breaks claims_do_not_widen_union_even_with_group_backing (the fabricated-merge case would then be Ok) and union_paths_stay_equal (sets no longer equal / len()!=1).
4. crates/but-api/src/legacy/merge_gate.rs (39-94)
   Focus: enforce_merge_gate — mutation (iii): making the gate load/authorize against the source/feat ref instead of branch_ref(target_branch) (~line 48 area) makes self_add_to_maintainers_on_feature_head_still_denied_merge clear the Authority::Merge step (feat-author IS a maintainer on feat) instead of perm.denied.
5. crates/but-authz/src/config.rs (276-301)
   Focus: read_config_blob — mutation (iv): replacing `repo.find_reference(target_ref)` (281) with a HEAD peel makes membership_read_only_from_target_ref / self_grant_admin_inert_until_landed read the feature head, authorizing wrongly. (Note: FIX-GRPS-002-AC3-TEETH makes self_grant_admin_inert_until_landed's fixture leave HEAD on feat-admin, strengthening this mutation's bite for AC-3 — if that task has landed, cite the stronger result.)
6. crates/but-authz/tests/grps_union.rs (8-146)
   Focus: the named GRPS-001 contract tests the mutations must flip: group_union_authorizes_review_denies_merge (9-31), union_paths_stay_equal (34-73), claims_do_not_widen_union_even_with_group_backing (107-146).
7. crates/but-api/tests/merge_gate_self_escalation.rs (12-44) + crates/but-authz/tests/grps_ref_pin.rs (10-101)
   Focus: the named GRPS-002 contract tests: self_add_to_maintainers_on_feature_head_still_denied_merge, membership_read_only_from_target_ref, self_grant_admin_inert_until_landed.
8. .spec/prds/governance/tasks/sprint-03-grps-groups-ref-pin/GRPS-001-effective-set-union-group-ceiling.md (208-218)
   Focus: the REQUIREMENT-CONTRACT marker + the verification_policy object to amend (the `<!-- REQUIREMENT-CONTRACT v1 -->` then `<!--` then JSON then `-->` format must be preserved).
9. .spec/prds/governance/tasks/sprint-03-grps-groups-ref-pin/GRPS-002-ref-pinned-membership-self-grant-inert.md (221-231)
   Focus: the same marker + verification_policy object for GRPS-002.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- evidence files exist with restored-clean entries: `test -f .tmp/GRPS-001/mutation-evidence.md && test -f .tmp/GRPS-002/mutation-evidence.md && [ $(grep -c 'RESTORED CLEAN' .tmp/GRPS-001/mutation-evidence.md) -ge 2 ] && [ $(grep -c 'RESTORED CLEAN' .tmp/GRPS-002/mutation-evidence.md) -ge 2 ]`  -> Exit 0
- each evidence file shows a genuine RED for a named test: `grep -E 'panicked|assertion .* failed|FAILED' .tmp/GRPS-001/mutation-evidence.md` and `... .tmp/GRPS-002/mutation-evidence.md`  -> match
- GRPS-001 contract amended + valid: extract the REQUIREMENT-CONTRACT JSON, `python3 -c "import json,sys; d=json.load(sys.stdin); vp=d['verification_policy']; assert vp['requires_red_evidence'] is False; assert vp['red_evidence_waiver']; assert vp['falsifiability_substitute']=='.tmp/GRPS-001/mutation-evidence.md'; assert vp['requires_seeded_evidence'] is True"`  -> Exit 0
- GRPS-002 contract amended + valid: same check with falsifiability_substitute=='.tmp/GRPS-002/mutation-evidence.md'  -> Exit 0
- source restored clean: `git diff --stat crates/` empty for src files ; `cargo fmt --check`  -> Exit 0
- named tests green on restored baseline: `cargo test -p but-authz group_union_authorizes_review_denies_merge` ; `cargo test -p but-api self_add_to_maintainers_on_feature_head_still_denied_merge`  -> Exit 0
- human acknowledgment surfaced: completion report contains the "HUMAN ACKNOWLEDGMENT REQUIRED" waiver note (both tasks + reason + substitute paths)

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: mutation-based falsifiability evidence as the documented substitute for an impossible behavioral RED — apply a throwaway production mutation, run the named contract test, capture the genuine RED, restore clean; then record a requires_red_evidence WAIVER (false + reason + substitute pointer) in the contract and surface it to the human
pattern_source: .spec/reviews/red-hat-sprint-03-2026-06-19.md:53-59 (the red-hat remediation routing for finding A) + the rust-reviewer's Mut-1..5 mutation methodology cited in the report header
anti_pattern: fabricating a behavioral RED (the cardinal stubbing sin); claiming a mutation flipped a test it did not; leaving a mutation in source; flipping requires_seeded_evidence; editing prose/TASK-TEMPLATE instead of just verification_policy; recording the waiver in the contract but NOT surfacing it to the human (silent policy change); breaking JSON validity with a hand-edit
interaction_notes:
  - Each mutation is a real edit to crates/ source, applied ONE AT A TIME, run, captured, and reverted before the next. Use `git stash`/`git restore` or a saved copy; confirm `git diff` is clean between mutations. The whole point is that the tests have TEETH — if a chosen mutation does not flip the named test, that is honest signal: pick one that bites (the report already proved several do).
  - The amendment touches ONLY the verification_policy object. The exact embedded format is: a line `<!-- REQUIREMENT-CONTRACT v1 -->`, then a line `<!--`, then the JSON, then a line `-->`. Preserve that wrapper exactly; only the JSON's verification_policy keys change.
  - requires_seeded_evidence stays TRUE: GRPS-001/002 both seed real-git scenarios; only the BEHAVIORAL-RED requirement is waived, justified by the mutation substitute.
  - If FIX-GRPS-002-AC3-TEETH lands before this task, the HEAD-peel mutation (iv) for self_grant_admin_inert_until_landed bites HARDER (HEAD now != target ref) — cite the stronger result; if it has not landed, membership_read_only_from_target_ref (which checks out feat) is the reliable HEAD-peel target for GRPS-002 evidence.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — runs throwaway source mutations against real but-authz/but-api tests, captures genuine RED output, restores clean, writes the evidence artifacts, and performs a careful surgical JSON amendment of two REQUIREMENT-CONTRACTs. Requires Rust test fluency + git-restore discipline + JSON precision. No frontend.
reviewer: rust-reviewer — adversarial pass: independently re-apply at least one mutation per file to confirm the documented RED is real (not fabricated), confirm crates/ source is restored clean, confirm both JSON blobs are valid with the exact waiver fields, confirm requires_seeded_evidence is intact, and confirm the human-acknowledgment note is present. This reviewer is the anti-fabrication gate.
coding_standards: crates/AGENTS.md, ~/.claude/CLAUDE.md (THE SUPREME RULE — no fabricated evidence), the REQUIREMENT-CONTRACT v1 format in the GRPS-001/002 files

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: GRPS-001 (merged), GRPS-002 (merged)
Blocks:     (none — independent remediation; unblocks future /kb-run-sprint passes over GRPS-001/002 by clearing the unmet requires_red_evidence gate)
Parallel with: FIX-GRPS-002-AC3-TEETH, FIX-GRPS-001-EMPTY-START-CONTROL
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "FIX-GRPS-RED-EVIDENCE-CONTRACT",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": false,
    "requires_red_evidence": false,
    "requires_seeded_evidence": false,
    "evidence_task": true,
    "red_evidence_waiver": "EVIDENCE task: verification is artifact presence + mutation integrity + contract-JSON validity, not a new cargo test. Falsifiability is the genuine-mutation requirement itself (a mutation that does not flip the named test RED is rejected).",
    "falsifiability_substitute": ".tmp/GRPS-001/mutation-evidence.md + .tmp/GRPS-002/mutation-evidence.md"
  },
  "fixtures": {
    "merged_sprint03_baseline": {
      "description": "The merged Sprint-03 source at a clean baseline: crates/but-authz (config.rs normalize_permissions member-fold + authorize.rs effective_authority), crates/but-api/src/legacy/merge_gate.rs enforce_merge_gate, and the merged contract tests (grps_union.rs, grps_ref_pin.rs, merge_gate_self_escalation.rs). Mutations are applied transiently against THIS baseline and restored after each capture.",
      "seed_method": "none",
      "records": [
        "Confirm a clean baseline first: `git diff --stat crates/` is empty before starting.",
        "Apply each mutation one at a time, run the named test, capture the RED output, then `git restore` the file and confirm `git diff` is clean before the next mutation.",
        "No git fixture seeding is required beyond what the contract tests already build via but_testsupport::writable_scenario."
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "description": "Genuine mutation evidence captured for both GRPS-001 and GRPS-002: >=2 mutations per file that flip a named contract test RED, each with diff + command + RED output + restored-clean confirmation.",
      "verify": "test -f .tmp/GRPS-001/mutation-evidence.md && test -f .tmp/GRPS-002/mutation-evidence.md",
      "primary": true,
      "scenario": {
        "tier": "visible",
        "test_tier": "evidence",
        "verification_service": "real but-authz/but-api tests run against transiently-mutated real source, restored clean after each mutation",
        "negative_control": {
          "would_fail_if": [
            "a documented 'mutation' leaves the named test GREEN (the test lacks teeth — the mutation must genuinely flip it RED)",
            "any mutation is left unrestored in crates/ source (git diff for src must be empty)",
            "the RED output is fabricated or a mutation is claimed to flip a test it did not (cardinal stubbing sin)",
            "fewer than 2 genuine flips are documented per file"
          ]
        },
        "evidence": {
          "artifact_type": "file",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "merged_sprint03_baseline",
            "action": {
              "actor": "ci",
              "steps": [
                "GRPS-001 (i): drop the config.rs group.members member-fold (loop ~355-368) => run group_union_authorizes_review_denies_merge => capture RED (reviewer-only ReviewsWrite DENIED) => restore",
                "GRPS-001 (ii): union a spurious Authority::Merge into effective_authority (authorize.rs:51-58) => run claims_do_not_widen_union_even_with_group_backing (and/or union_paths_stay_equal) => capture RED => restore",
                "GRPS-002 (iii): make enforce_merge_gate load/authorize against the source/feat ref => run self_add_to_maintainers_on_feature_head_still_denied_merge => capture RED (merge-authority wrongly cleared) => restore",
                "GRPS-002 (iv): make read_config_blob peel HEAD instead of target_ref => run membership_read_only_from_target_ref (and/or self_grant_admin_inert_until_landed) => capture RED => restore"
              ]
            },
            "end_state": {
              "must_observe": [
                ".tmp/GRPS-001/mutation-evidence.md with >=2 genuine RED flips (4 parts each)",
                ".tmp/GRPS-002/mutation-evidence.md with >=2 genuine RED flips (4 parts each)",
                "each entry ends with a RESTORED CLEAN confirmation (git diff empty for the mutated file)"
              ],
              "must_not_observe": [
                "a 'mutation' that left the named test GREEN presented as a flip",
                "any residual mutation in crates/ source",
                "fabricated or embellished RED output"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "Both REQUIREMENT-CONTRACTs amended to requires_red_evidence:false + red_evidence_waiver + falsifiability_substitute; requires_seeded_evidence + requires_tests unchanged; both JSON valid.",
      "verify": "python3 -m json.tool < extracted GRPS-001 contract && python3 -m json.tool < extracted GRPS-002 contract",
      "primary": false,
      "scenario": {
        "tier": "visible",
        "test_tier": "evidence",
        "verification_service": "JSON parse of the extracted REQUIREMENT-CONTRACT blocks from both task files",
        "negative_control": {
          "would_fail_if": [
            "either JSON became unparseable after the edit",
            "requires_seeded_evidence were dropped or flipped to false",
            "requires_red_evidence stayed true",
            "a prose section or the TASK-TEMPLATE block was altered (only the verification_policy object may change)"
          ]
        },
        "evidence": {
          "artifact_type": "file",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "merged_sprint03_baseline",
            "action": {
              "actor": "ci",
              "steps": [
                "edit GRPS-001 verification_policy: requires_red_evidence -> false; add red_evidence_waiver; add falsifiability_substitute=\".tmp/GRPS-001/mutation-evidence.md\"",
                "edit GRPS-002 verification_policy: same; falsifiability_substitute=\".tmp/GRPS-002/mutation-evidence.md\"",
                "extract each contract between the marker's <!-- ... --> and json.loads it"
              ]
            },
            "end_state": {
              "must_observe": [
                "GRPS-001 verification_policy.requires_red_evidence == false, red_evidence_waiver present, falsifiability_substitute == \".tmp/GRPS-001/mutation-evidence.md\"",
                "GRPS-002 verification_policy.requires_red_evidence == false, red_evidence_waiver present, falsifiability_substitute == \".tmp/GRPS-002/mutation-evidence.md\"",
                "both verification_policy.requires_seeded_evidence == true and requires_tests == true (unchanged)",
                "both JSON blobs parse cleanly"
              ],
              "must_not_observe": [
                "requires_seeded_evidence dropped/flipped",
                "requires_red_evidence still true",
                "malformed JSON",
                "any prose/TASK-TEMPLATE edit"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "crates/ source is clean at task end (every mutation restored); cargo fmt --check passes; the named GRPS contract tests are GREEN again on the restored baseline.",
      "verify": "git diff --stat crates/ && cargo fmt --check",
      "primary": false,
      "scenario": {
        "tier": "visible",
        "test_tier": "evidence",
        "verification_service": "git status + cargo fmt + cargo test on the restored baseline",
        "negative_control": {
          "would_fail_if": [
            "any mutation residue remained in crates/ src (git diff non-empty)",
            "a restored file did not return to fmt-clean",
            "a named test stayed RED after restore (mutation not fully reverted)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "merged_sprint03_baseline",
            "action": {
              "actor": "ci",
              "steps": [
                "git diff --stat crates/ (expect empty for src)",
                "cargo fmt --check",
                "cargo test -p but-authz group_union_authorizes_review_denies_merge",
                "cargo test -p but-api self_add_to_maintainers_on_feature_head_still_denied_merge"
              ]
            },
            "end_state": {
              "must_observe": [
                "`git diff --stat crates/` shows NO src changes (only .tmp + .spec contract edits persist)",
                "`cargo fmt --check` passes",
                "the named GRPS-001/002 contract tests are GREEN on the restored baseline"
              ],
              "must_not_observe": [
                "residual mutation in crates/ src",
                "a named test still RED after restore"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "description": "The requires_red_evidence waiver is surfaced to the human for acknowledgment (named tasks + reason + falsifiability_substitute paths) per policy.",
      "verify": "completion report contains the HUMAN ACKNOWLEDGMENT REQUIRED waiver note",
      "primary": false,
      "scenario": {
        "tier": "visible",
        "test_tier": "evidence",
        "verification_service": "presence of the acknowledgment note in the task completion report",
        "negative_control": {
          "would_fail_if": [
            "the waiver were recorded in the contract but NOT surfaced to the human (silent policy change)",
            "the note omitted either task or the falsifiability_substitute paths"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "merged_sprint03_baseline",
            "action": {
              "actor": "ci",
              "steps": [
                "compose the completion report with a HUMAN ACKNOWLEDGMENT REQUIRED section naming GRPS-001 + GRPS-002, the waiver reason, and both falsifiability_substitute paths"
              ]
            },
            "end_state": {
              "must_observe": [
                "the report names both GRPS-001 and GRPS-002",
                "the report states requires_red_evidence was waived to false with the reason",
                "the report lists .tmp/GRPS-001/mutation-evidence.md and .tmp/GRPS-002/mutation-evidence.md"
              ],
              "must_not_observe": [
                "a silent contract change with no human-facing acknowledgment",
                "an incomplete note (missing a task or the substitute paths)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": ".tmp/GRPS-001/mutation-evidence.md exists with >=2 entries each having diff + command + RED output + RESTORED CLEAN",
      "verify": "test -f .tmp/GRPS-001/mutation-evidence.md ; grep -c 'RESTORED CLEAN' .tmp/GRPS-001/mutation-evidence.md",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": ".tmp/GRPS-002/mutation-evidence.md exists with >=2 entries each having diff + command + RED output + RESTORED CLEAN",
      "verify": "test -f .tmp/GRPS-002/mutation-evidence.md ; grep -c 'RESTORED CLEAN' .tmp/GRPS-002/mutation-evidence.md",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "at least one documented mutation per file shows a genuine assertion-failure RED for a NAMED contract test (not a stayed-GREEN no-op)",
      "verify": "grep -E 'panicked|assertion .* failed|FAILED' .tmp/GRPS-001/mutation-evidence.md ; grep -E 'panicked|assertion .* failed|FAILED' .tmp/GRPS-002/mutation-evidence.md",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "GRPS-001 contract verification_policy.requires_red_evidence == false with red_evidence_waiver + falsifiability_substitute; requires_seeded_evidence == true; JSON valid",
      "verify": "python3 json.loads of the extracted GRPS-001 REQUIREMENT-CONTRACT",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "GRPS-002 contract verification_policy.requires_red_evidence == false with red_evidence_waiver + falsifiability_substitute; requires_seeded_evidence == true; JSON valid",
      "verify": "python3 json.loads of the extracted GRPS-002 REQUIREMENT-CONTRACT",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "description": "git diff --stat crates/ empty for src; cargo fmt --check passes; named tests GREEN on restored baseline",
      "verify": "git diff --stat crates/ ; cargo fmt --check ; cargo test -p but-authz group_union_authorizes_review_denies_merge",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-7",
      "type": "test_criterion",
      "description": "the completion report contains the HUMAN ACKNOWLEDGMENT REQUIRED waiver note (both tasks + reason + substitute paths)",
      "verify": "report inspection",
      "maps_to_ac": "AC-4"
    }
  ]
}
-->
</details>
