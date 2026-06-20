# FIX-GRPS-002-AC3-TEETH: Restore teeth parity to GRPS-002 AC-3 by keeping HEAD ≠ target ref in the self-grant fixture (+ idempotent merge-gate remote)

## What this does

Hardens GRPS-002 AC-3's negative control so the most natural head-read regression — a loader that peels HEAD instead of `refs/heads/main` — is *killed*, not survived. Today `self_grant_admin_repo` (crates/but-authz/tests/grps_ref_pin.rs:103-133) ends with `git checkout main` (line 128), so when `self_grant_admin_inert_until_landed` runs, HEAD coincides with the target ref. A mutation that points the loader at HEAD reads the *same* tree as `refs/heads/main`, so the before-land `assert_denied` still passes — the mutation **survived** (the red-hat panel's Mut-4). AC-4 (`membership_read_only_from_target_ref`) does `git checkout feat` first, so HEAD ≠ target and the generic HEAD-peel is caught. This task drops the trailing `git checkout main` so HEAD stays on `feat-admin` (the branch carrying the self-grant), making the before-land denial fail under a HEAD-peel loader — restoring teeth parity with AC-4. It also makes the unrelated `git remote add origin` step in crates/but-api/tests/merge_gate_self_escalation.rs::self_escalation_repo (line 97) idempotent to remove fixture fragility.

## Why

Sprint 03 remediation · red-hat finding **B** (MEDIUM, weak negative control) from `.spec/reviews/red-hat-sprint-03-2026-06-19.md:38,60-64`. The Sprint-03 security property (target-ref-only read, no self-escalation) is genuinely enforced and mutation-proven elsewhere; finding B is a *test-teeth* gap, not a product defect. AC-3 claims to prove "the self-grant on the feature head is inert because the target ref governs", but with HEAD == target ref the test cannot distinguish a target-ref read from a HEAD read. The fix is a one-line fixture change (delete `git checkout main`) that makes the existing assertions *discriminating* against the exact regression class AC-3 is supposed to catch. The remote-add idempotency is a fragility cleanup harvested from the same finding cluster (R6/git-remote-fixture note).

## How to verify

PRIMARY **AC-1** — `cargo test -p but-authz self_grant_admin_inert_until_landed` (the existing test stays GREEN after the fixture change: before-land denial names `administration:write`; after-land Ok; `refs/heads/main` sha advances). The teeth proof (AC-2) is a documented mutation: peeling HEAD instead of `refs/heads/main` must now make the before-land assertion FAIL. Full gate set in the spec below.

## Scope

- crates/but-authz/tests/grps_ref_pin.rs (MODIFY, FIXTURE-ONLY) — in `self_grant_admin_repo` (103-133) delete the trailing `git checkout main` (line 128) so HEAD stays on `feat-admin`; the `self_grant_admin_inert_until_landed` test body and its assertions are UNCHANGED (it already loads from `TARGET_REF` and lands via `land_admin_write` which commits on the current branch — confirm `land_admin_write` still advances `refs/heads/main`, see CRITICAL CONSTRAINTS)
- crates/but-api/tests/merge_gate_self_escalation.rs (MODIFY, FIXTURE-ONLY) — in `self_escalation_repo` (93-148) make the `git remote add origin ...` step (line 97) idempotent: prefix with `git remote remove origin 2>/dev/null || true` so re-runs / dirty scenarios do not fail on "remote origin already exists"

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: FIX-GRPS-002-AC3-TEETH - Restore teeth parity to GRPS-002 AC-3 (HEAD != target ref in self_grant_admin_repo) + idempotent merge-gate remote
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P1
EFFORT:     S  (45 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ./SPRINT.md
PRD_REFS:   UC-GRPS-02
CAPABILITIES: CAP-CONFIG-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-authz self_grant_admin_inert_until_landed   |   cargo test -p but-authz membership_read_only_from_target_ref   |   cargo test -p but-api self_add_to_maintainers_on_feature_head_still_denied_merge   |   cargo test -p but-api landed_membership_clears_merge_authority_step
  check: cargo check -p but-authz -p but-api --all-targets
  lint:  cargo clippy -p but-authz -p but-api --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
GRPS-002 AC-3's fixture (`self_grant_admin_repo`) leaves HEAD on the feature branch `feat-admin` (NOT `main`) so the test's pre-existing before-land `assert_denied` becomes discriminating against a HEAD-peel loader: with HEAD == target ref the regression survived; with HEAD == feat-admin (which DOES carry the self-granted administration:write) a loader that peels HEAD would read the escalated grant and authorize, FAILING the before-land denial. The test `self_grant_admin_inert_until_landed` stays GREEN against the correct (target-ref) loader, and the rust-reviewer's Mut-4 (loader peels HEAD instead of refs/heads/main) now KILLS AC-3. The `self_escalation_repo` remote-add step is idempotent (`git remote remove origin 2>/dev/null || true` before `git remote add origin ...`), removing the "remote already exists" fragility. No production source is modified; both test crates compile, fmt/clippy clean, and AC-1/AC-2/AC-4 of GRPS-002 stay green.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST verify that after deleting `git checkout main` the landing step still advances refs/heads/main. `self_grant_admin_inert_until_landed` calls `land_admin_write(&repo)` (grps_ref_pin.rs:184-197) which does `git add .gitbutler/permissions.toml && git commit`. With HEAD on `feat-admin`, that commit would land on `feat-admin`, NOT `main`, and the post-land `assert_ne!(main_before, main_after)` (lines 33-37) would FAIL because main did not move. THEREFORE `land_admin_write` MUST be made to commit onto `main` explicitly — prepend `git checkout main` INSIDE `land_admin_write` (it currently relies on the fixture leaving HEAD on main). This keeps the landing semantics ("the grant lands on the target ref") intact while HEAD stays on feat-admin during the before-land observation. The before-land `load_governance_config(&repo, TARGET_REF)` already reads `refs/heads/main` explicitly, so HEAD position does not affect the correct loader — only a (mutated) HEAD-peel loader.
- [MUST] MUST keep the change FIXTURE-ONLY in test files. Do NOT modify any production source (crates/but-authz/src/**, crates/but-api/src/**). This is a test-teeth hardening, not a behavior change.
- [MUST] MUST leave the `self_grant_admin_inert_until_landed` assertions exactly as-is (the perm.denied + administration:write naming before land, the sha-advance + Ok after land). The fixture change must not require any assertion edits — if it does, you have changed behavior; stop and reconsider.
- [NEVER] NEVER "fix" the survived mutation by adding a feature-branch-name string read or any HEAD-specific assertion to the test body — the whole point is that the GENERIC target-ref-vs-HEAD discriminator (HEAD != target ref) restores teeth. Adding a branch-name read would re-narrow the control to the same brittle shape the red-hat panel flagged.
- [NEVER] NEVER weaken AC-4 (`membership_read_only_from_target_ref`) or any other test to make this change pass. AC-4 already does `git checkout feat` and must stay green untouched.
- [NEVER] NEVER use `std::env::set_var` (unsafe in edition 2024) or introduce new dependencies — this task is pure git-fixture shell edits inside `invoke_bash` strings.
- [STRICTLY] STRICTLY make the remote-add idempotent with `git remote remove origin 2>/dev/null || true` BEFORE `git remote add origin <url>` — do NOT switch to `git remote set-url` (origin may not exist yet on a fresh scenario) and do NOT drop the remote entirely (the merge-gate fixture relies on origin being present for the repo URL).

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: self_grant_admin_inert_until_landed stays GREEN with HEAD on feat-admin (before-land perm.denied naming administration:write; main advances on land; after-land Ok)
- [ ] AC-2: a loader mutated to peel HEAD instead of refs/heads/main makes the before-land assertion FAIL (the documented teeth proof; Mut-4 now KILLS AC-3) — captured into .tmp/FIX-GRPS-002-AC3-TEETH/
- [ ] AC-3: self_escalation_repo's git remote add is idempotent (re-run safe); but-api merge-gate tests stay green
- [ ] AC-4 (regression): membership_read_only_from_target_ref + GRPS-002 AC-1/AC-2 stay green; fmt + clippy clean for but-authz and but-api

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: self_grant_admin_inert_until_landed stays GREEN with HEAD on feat-admin [PRIMARY]
  GIVEN: self_grant_admin_repo modified so the trailing `git checkout main` (grps_ref_pin.rs:128) is removed (HEAD stays on `feat-admin`, which carries permissions.toml self-granting feat-author administration:write), and land_admin_write modified to `git checkout main` before committing the landed grant so refs/heads/main advances
  WHEN:  cargo test -p but-authz self_grant_admin_inert_until_landed
  THEN:  the test PASSES — before-land authorize(feat-author, AdministrationWrite, cfg-from-main) returns Err(Denial{code=="perm.denied"}) whose message contains "administration:write"; after land_admin_write, refs/heads/main sha advances (main_before != main_after) and authorize returns Ok(())
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-authz load_governance_config + real gix repo, target-ref read while HEAD is on feat-admin
  VERIFY: cargo test -p but-authz self_grant_admin_inert_until_landed
  SCENARIO (negative controls): would FAIL (correctly) if land_admin_write committed onto feat-admin instead of main (main_before == main_after fails the assert_ne!); would FAIL if removing `git checkout main` accidentally broke the before-land read (it cannot — load reads TARGET_REF explicitly); would pass against a stub authorize that always denies (guarded by the after-land Ok requirement)

AC-2: a HEAD-peel loader makes the before-land assertion FAIL — the restored-teeth proof
  GIVEN: the AC-1 fixture (HEAD on feat-admin) and a THROWAWAY mutation of the loader's target-ref resolution to peel HEAD instead of refs/heads/main (e.g. in crates/but-authz/src/config.rs::read_config_blob, replace `repo.find_reference(target_ref)` with `repo.head_ref()` / peel HEAD)
  WHEN:  cargo test -p but-authz self_grant_admin_inert_until_landed is run WITH the mutation applied
  THEN:  the before-land assert_denied PANICS / the test FAILS RED — because HEAD (feat-admin) carries the self-granted administration:write, a HEAD-peel loader authorizes the grant and `assert_denied` sees Ok where it requires Err. This proves the survived Mut-4 now KILLS AC-3. The mutation MUST be RESTORED after capture (throwaway; never left in source).
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-authz with a transient source mutation, restored clean afterward
  VERIFY: capture in .tmp/FIX-GRPS-002-AC3-TEETH/head-peel-mutation.md (mutation diff + command + observed RED output + `git diff --stat` showing src restored clean)
  SCENARIO (negative controls): the proof is INVALID if the mutated run stays GREEN (means HEAD still == target ref — the fixture change did not take); the proof is INVALID if the mutation is left in source (must show restored-clean confirmation); the proof is INVALID if the RED came from a compile error rather than the assert_denied panic (must show the assertion-failure message)

AC-3: self_escalation_repo git-remote-add is idempotent and but-api merge-gate tests stay green
  GIVEN: self_escalation_repo (merge_gate_self_escalation.rs:93-148) with the remote-add step (line 97) prefixed by `git remote remove origin 2>/dev/null || true`
  WHEN:  cargo test -p but-api self_add_to_maintainers_on_feature_head_still_denied_merge AND cargo test -p but-api landed_membership_clears_merge_authority_step
  THEN:  both tests PASS — the merge from feat is still denied at the Authority::Merge step (perm.denied) before landing, and clears after landing (gate.review_required residual); the idempotent remote-add does not change the repository_https_url the ForgeReview seed relies on
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api enforce_merge_gate + real gix repo + inline-seeded forge-review cache
  VERIFY: cargo test -p but-api self_add_to_maintainers_on_feature_head_still_denied_merge ; cargo test -p but-api landed_membership_clears_merge_authority_step
  SCENARIO (negative controls): would FAIL if the remote-remove dropped origin without re-adding it (the repo URL the seed expects would be absent); would FAIL if the idempotency line introduced a non-zero exit that `invoke_bash` propagates (guarded by `2>/dev/null || true`); a stub gate that always returns Ok fails AC-1's still-denied requirement

AC-4: regression — AC-4 of GRPS-002 + GRPS-002 AC-1/AC-2 stay green; fmt + clippy clean
  GIVEN: the fixture changes applied with the mutation restored
  WHEN:  cargo test -p but-authz membership_read_only_from_target_ref ; cargo fmt --check ; cargo clippy -p but-authz -p but-api --all-targets
  THEN:  membership_read_only_from_target_ref passes (it already checks out feat — unaffected), fmt reports no diff, clippy reports no warnings
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-authz/but-api test suite + cargo fmt/clippy
  VERIFY: cargo test -p but-authz membership_read_only_from_target_ref ; cargo fmt --check ; cargo clippy -p but-authz -p but-api --all-targets
  SCENARIO (negative controls): would FAIL if the fixture edit introduced a TOML/heredoc syntax error in the invoke_bash string (clippy/compile catches); would FAIL if clippy flagged the idempotency change

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, happy_path): self_grant_admin_inert_until_landed passes with HEAD left on feat-admin and land_admin_write committing onto main
    VERIFY: cargo test -p but-authz self_grant_admin_inert_until_landed
- TC-2 (-> AC-1, structural): main_before != main_after still holds after land_admin_write (landing advances the target ref despite HEAD on feat-admin)
    VERIFY: cargo test -p but-authz self_grant_admin_inert_until_landed
- TC-3 (-> AC-2, error): with a transient HEAD-peel loader mutation, self_grant_admin_inert_until_landed FAILS at the before-land assert_denied (RED), proving restored teeth; restored clean afterward
    VERIFY: .tmp/FIX-GRPS-002-AC3-TEETH/head-peel-mutation.md (diff + command + RED output + restored-clean git diff)
- TC-4 (-> AC-3, error): enforce_merge_gate self-add case still returns perm.denied with the idempotent remote-add
    VERIFY: cargo test -p but-api self_add_to_maintainers_on_feature_head_still_denied_merge
- TC-5 (-> AC-3, happy_path): landed_membership_clears_merge_authority_step still clears the merge-authority step after the idempotent remote-add
    VERIFY: cargo test -p but-api landed_membership_clears_merge_authority_step
- TC-6 (-> AC-4, structural): membership_read_only_from_target_ref + fmt + clippy stay clean
    VERIFY: cargo test -p but-authz membership_read_only_from_target_ref ; cargo fmt --check ; cargo clippy -p but-authz -p but-api --all-targets

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-CONFIG-01
provides: a discriminating negative control for GRPS-002 AC-3 (HEAD != target ref) so the generic HEAD-peel regression is killed, not survived; idempotent merge-gate origin fixture
consumes: but_authz::load_governance_config, but_authz::authorize, but_authz::Authority::AdministrationWrite, but_authz::resolve_principal, but_testsupport::writable_scenario, but_testsupport::invoke_bash, but_api::legacy::merge_gate::enforce_merge_gate (read-only)
boundary_contracts:
  - CAP-CONFIG-01: a feature head can never authorize its own grant — governance config is read at the target ref, never HEAD/working tree. AC-3's test must be able to DETECT a HEAD-read regression; this task makes HEAD diverge from the target ref so the existing assertion is discriminating.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-authz/tests/grps_ref_pin.rs (MODIFY, FIXTURE-ONLY) — delete `git checkout main` at line 128 in self_grant_admin_repo; add `git checkout main` at the top of land_admin_write's invoke_bash so the landed grant advances refs/heads/main; do NOT touch the test bodies' assertions
  - crates/but-api/tests/merge_gate_self_escalation.rs (MODIFY, FIXTURE-ONLY) — prepend `git remote remove origin 2>/dev/null || true` before `git remote add origin ...` (line 97) in self_escalation_repo
  - .tmp/FIX-GRPS-002-AC3-TEETH/ (NEW) — capture the HEAD-peel mutation teeth proof (diff + command + RED output + restored-clean confirmation)
writeProhibited:
  - crates/but-authz/src/** — production source; this is a test-teeth fix, no behavior change (the HEAD-peel mutation is THROWAWAY, restored immediately after capture)
  - crates/but-api/src/** — production source; merge gate is consume-only
  - crates/but-authz/tests/grps_union.rs — owned by FIX-GRPS-001-EMPTY-START-CONTROL
  - the assertions inside self_grant_admin_inert_until_landed / membership_read_only_from_target_ref — fixture helpers only
  - any gitbutler-* crate
  - Any file not in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-authz/tests/grps_ref_pin.rs (10-47)
   Focus: self_grant_admin_inert_until_landed — loads TARGET_REF ("refs/heads/main"), asserts before-land perm.denied + administration:write (17-30), lands via land_admin_write (32), asserts main sha advanced (33-37), re-loads + authorizes Ok (39-41). The HEAD position does NOT affect this body (it reads TARGET_REF explicitly) — only a mutated HEAD-peel loader.
2. crates/but-authz/tests/grps_ref_pin.rs (103-133)
   Focus: self_grant_admin_repo — line 128 `git checkout main` is the defect; deleting it leaves HEAD on feat-admin (121-127 commit the self-grant on feat-admin). The committed target ref (108-119) excludes administration:write.
3. crates/but-authz/tests/grps_ref_pin.rs (184-197)
   Focus: land_admin_write — currently `git add + git commit` ONLY; it relies on the fixture leaving HEAD on main. After this change HEAD is on feat-admin, so land_admin_write MUST `git checkout main` first or the landing commits on the wrong branch and main does not advance (breaking 33-37).
4. crates/but-authz/tests/grps_ref_pin.rs (49-101)
   Focus: membership_read_only_from_target_ref — the AC-4 sibling that ALREADY does `git checkout feat` (54) so HEAD != target ref; the teeth-parity reference for why AC-3 must also diverge HEAD from target.
5. crates/but-authz/src/config.rs (276-301)
   Focus: read_config_blob — `repo.find_reference(target_ref)` (281) is the line a HEAD-peel mutation would replace with a HEAD read; this is what AC-2's throwaway mutation targets to prove teeth.
6. crates/but-api/tests/merge_gate_self_escalation.rs (93-148)
   Focus: self_escalation_repo — line 97 `git remote add origin https://github.com/gitbutler/merge-gate-fixture.git` is the non-idempotent step; the seeded ForgeReview (188-205) carries repository_https_url matching this origin, so origin must remain present after the idempotency change.
7. .spec/reviews/red-hat-sprint-03-2026-06-19.md (38, 60-64)
   Focus: finding B description + the exact remediation instruction (drop trailing `git checkout main`; re-run Mut-4 to confirm it now KILLS AC-3; harden remote-add idempotency)

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- AC-3 teeth GREEN baseline: `cargo test -p but-authz self_grant_admin_inert_until_landed`  -> Exit 0; test passes with HEAD on feat-admin and land_admin_write committing onto main
- AC-3 teeth RED proof: apply the HEAD-peel loader mutation, re-run the test, observe the before-land assert_denied FAIL, RESTORE the mutation -> captured in .tmp/FIX-GRPS-002-AC3-TEETH/head-peel-mutation.md; `git diff --stat crates/but-authz/src` shows NO residual mutation
- merge-gate idempotency: `cargo test -p but-api self_add_to_maintainers_on_feature_head_still_denied_merge` && `cargo test -p but-api landed_membership_clears_merge_authority_step`  -> Exit 0
- regression: `cargo test -p but-authz membership_read_only_from_target_ref`  -> Exit 0
- clippy: `cargo clippy -p but-authz -p but-api --all-targets`  -> Exit 0
- fmt: `cargo fmt --check`  -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: make a target-ref-only-read negative control DISCRIMINATING by leaving HEAD on a feature branch that carries the escalation, so a HEAD-peel regression reads the escalated tree and the existing denial assertion fails — mirroring membership_read_only_from_target_ref's `git checkout feat` (grps_ref_pin.rs:54)
pattern_source: crates/but-authz/tests/grps_ref_pin.rs:49-101 (AC-4 leaves HEAD on feat); the fix applies the same HEAD-diverges-from-target discipline to AC-3's self_grant_admin_repo
anti_pattern: leaving HEAD == target ref (the survived-mutation defect); "fixing" the survived mutation by reading a feature-branch NAME in the test (re-narrows the control); committing land_admin_write onto feat-admin instead of main (breaks the sha-advance assertion); switching remote-add to set-url (origin may not exist); dropping origin entirely (the forge seed needs the repo URL); leaving the throwaway HEAD-peel mutation in source
interaction_notes:
  - The before-land observation in self_grant_admin_inert_until_landed reads `load_governance_config(&repo, TARGET_REF)` (grps_ref_pin.rs:14) which resolves "refs/heads/main" explicitly via read_config_blob -> find_reference(target_ref). HEAD position is irrelevant to the CORRECT loader; it only changes what a (mutated) HEAD-peel loader would see. That is exactly why moving HEAD to feat-admin restores teeth without touching the test body.
  - land_admin_write must be adjusted in lockstep: it currently inherits HEAD==main from the fixture. After the fixture leaves HEAD on feat-admin, prepend `git checkout main` inside land_admin_write so the landing commit advances refs/heads/main (the after-land Ok + assert_ne! depend on it).
  - The HEAD-peel mutation (AC-2) is a THROWAWAY falsifiability probe, NOT a code change to ship. Capture the diff + RED output, then `git checkout -- crates/but-authz/src/config.rs` (or `git restore`) and confirm `git diff` is clean before committing the task.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — Pure but-authz/but-api test-fixture Rust work: gix-backed real-git scenario seeding via invoke_bash heredocs, a target-ref-vs-HEAD discriminator, and a throwaway source mutation to prove falsifiability. No frontend, no Tauri.
reviewer: rust-reviewer — adversarial pass: confirm the mutation genuinely flips the test RED (re-run Mut-4), confirm the mutation is restored clean, confirm no test assertion was weakened, clippy/fmt regression.
coding_standards: crates/AGENTS.md, crates/WORKSPACE_MODEL.md, crates/but-authz (nearby patterns: but-testsupport scenarios, invoke_bash heredocs, gix target-ref blob read)

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: GRPS-002 (merged)
Blocks:     (none — independent remediation)
Parallel with: FIX-GRPS-001-EMPTY-START-CONTROL, FIX-GRPS-RED-EVIDENCE-CONTRACT
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "FIX-GRPS-002-AC3-TEETH",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "self_grant_admin_head_on_feat": {
      "description": "The existing self_grant_admin_repo (crates/but-authz/tests/grps_ref_pin.rs:103-133) with the trailing `git checkout main` (line 128) DELETED so HEAD stays on `feat-admin`. The target ref refs/heads/main (committed at 108-119) grants feat-author only contents:write (NO administration:write); the feat-admin head (committed at 121-127) self-grants administration:write. land_admin_write (184-197) MUST be adjusted to `git checkout main` before committing so the landed grant advances refs/heads/main while HEAD started on feat-admin.",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"governance-base\");",
        "invoke_bash: write .gitbutler/permissions.toml ([[principal]] id=\"feat-author\" permissions=[\"contents:write\"]) + .gitbutler/gates.toml ([[branch]] name=\"main\" protected=true); git add + commit on main (the target ref WITHOUT admin)",
        "invoke_bash: git checkout -b feat-admin; rewrite permissions.toml so feat-author permissions=[\"contents:write\",\"administration:write\"] (the self-grant); git add + commit on feat-admin",
        "DELETE the trailing `git checkout main` — HEAD stays on feat-admin (the branch carrying the escalation), so a HEAD-peel loader would read the escalated grant",
        "land_admin_write: prepend `git checkout main` inside its invoke_bash heredoc before `git add + git commit` so the landed administration:write grant advances refs/heads/main"
      ]
    },
    "merge_gate_idempotent_remote": {
      "description": "The existing self_escalation_repo (crates/but-api/tests/merge_gate_self_escalation.rs:93-148) with the remote-add step (line 97) made idempotent. The seeded ForgeReview (188-205) carries repository_https_url=\"https://github.com/gitbutler/merge-gate-fixture.git\" which must continue to match origin after the idempotency change.",
      "seed_method": "cli",
      "records": [
        "prepend `git remote remove origin 2>/dev/null || true` before `git remote add origin https://github.com/gitbutler/merge-gate-fixture.git` so re-runs / dirty scenarios do not abort on `error: remote origin already exists`",
        "keep the rest of self_escalation_repo unchanged (target-ref maintainers excludes feat-author; feat head self-adds feat-author)"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "description": "self_grant_admin_inert_until_landed stays GREEN with HEAD on feat-admin (before-land perm.denied naming administration:write; main advances on land; after-land Ok).",
      "verify": "cargo test -p but-authz self_grant_admin_inert_until_landed",
      "primary": true,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-authz load_governance_config + real gix repo, target-ref read while HEAD is on feat-admin",
        "negative_control": {
          "would_fail_if": [
            "land_admin_write commits onto feat-admin instead of main, so main_before == main_after and the assert_ne! at grps_ref_pin.rs:34-37 FAILS",
            "removing `git checkout main` broke the before-land read — impossible, because load_governance_config reads TARGET_REF (\"refs/heads/main\") explicitly regardless of HEAD",
            "would pass against a stub authorize that always denies — guarded by the after-land Ok requirement (grps_ref_pin.rs:41)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "self_grant_admin_head_on_feat",
            "action": {
              "actor": "ci",
              "steps": [
                "load_governance_config(&repo, \"refs/heads/main\")",
                "resolve feat-author via the injected-lookup principal() helper",
                "authorize(feat-author, Authority::AdministrationWrite, &cfg) BEFORE landing",
                "land_admin_write(&repo) (now checks out main first)",
                "reload from refs/heads/main; authorize(feat-author, AdministrationWrite) AFTER landing"
              ]
            },
            "end_state": {
              "must_observe": [
                "`before-land authorize(AdministrationWrite).err().code == \"perm.denied\"`",
                "`before-land denial.message` contains `\"administration:write\"`",
                "the target-ref sha advanced: `main_before != main_after`",
                "`after-land authorize(AdministrationWrite) == Ok(())`"
              ],
              "must_not_observe": [
                "the target-ref sha unchanged (`0`-distance advancement): `main_before == main_after` (land committed onto feat-admin, not main)",
                "before-land `authorize` returns `Ok` (self-grant honored from the feature head)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "A loader mutated to peel HEAD instead of refs/heads/main makes the before-land assertion FAIL — the documented teeth proof (Mut-4 now KILLS AC-3). Mutation restored clean after capture.",
      "verify": "cat .tmp/FIX-GRPS-002-AC3-TEETH/head-peel-mutation.md",
      "primary": false,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-authz with a transient source mutation (HEAD-peel) restored clean afterward",
        "negative_control": {
          "would_fail_if": [
            "the mutated run stays GREEN — means HEAD still == target ref and the fixture change did not take (the survived-mutation condition the fix removes)",
            "the mutation is left in source — the proof requires a restored-clean `git diff` confirmation",
            "the RED came from a compile error rather than the assert_denied panic — the proof must show the assertion-failure message"
          ]
        },
        "evidence": {
          "artifact_type": "file",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "self_grant_admin_head_on_feat",
            "action": {
              "actor": "ci",
              "steps": [
                "apply a THROWAWAY mutation to crates/but-authz/src/config.rs::read_config_blob: peel HEAD instead of `repo.find_reference(target_ref)`",
                "cargo test -p but-authz self_grant_admin_inert_until_landed (expect RED at the before-land assert_denied)",
                "capture the mutation diff, the command, and the observed assertion-failure output",
                "restore the mutation (`git restore crates/but-authz/src/config.rs`) and confirm `git diff --stat crates/but-authz/src` is empty"
              ]
            },
            "end_state": {
              "must_observe": [
                "the mutated run FAILS at the before-land assert_denied (HEAD/feat-admin carries administration:write, so a HEAD-peel loader authorizes where Err is required)",
                "`git diff --stat crates/but-authz/src` is EMPTY after restore (mutation is throwaway)",
                ".tmp/FIX-GRPS-002-AC3-TEETH/head-peel-mutation.md records diff + command + RED output + restored-clean confirmation"
              ],
              "must_not_observe": [
                "the mutated run staying GREEN (`0` teeth — fixture change did not take effect)",
                "the mutation persisting in source after capture"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "self_escalation_repo git-remote-add is idempotent and the but-api merge-gate self-escalation tests stay green.",
      "verify": "cargo test -p but-api self_add_to_maintainers_on_feature_head_still_denied_merge",
      "primary": false,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api enforce_merge_gate + real gix repo + inline-seeded forge-review cache",
        "negative_control": {
          "would_fail_if": [
            "the remote-remove dropped origin without re-adding it — the repository_https_url the ForgeReview seed expects would be absent",
            "the idempotency line introduced a non-zero exit invoke_bash propagates — guarded by `2>/dev/null || true`",
            "a stub gate that always returns Ok fails the still-denied requirement"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "merge_gate_idempotent_remote",
            "action": {
              "actor": "ci",
              "steps": [
                "git remote remove origin 2>/dev/null || true; git remote add origin <fixture url>",
                "seed ForgeReview inline; enforce_merge_gate(&ctx, REVIEW_ID) before landing",
                "land feat-author membership; rerun enforce_merge_gate"
              ]
            },
            "end_state": {
              "must_observe": [
                "before landing: `classify_error(err).code == \"perm.denied\"` (merge-authority step) and the message contains `\"merge\"`",
                "after landing: the merge-authority perm.denied is gone (residual `gate.review_required` is the expected next gate)",
                "the idempotent remote-add leaves origin present with the fixture URL"
              ],
              "must_not_observe": [
                "`error: remote origin already exists` aborting the scenario",
                "origin absent so the seeded repository_https_url no longer matches"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "description": "Regression: membership_read_only_from_target_ref + GRPS-002 AC-1/AC-2 stay green; fmt + clippy clean for but-authz and but-api.",
      "verify": "cargo test -p but-authz membership_read_only_from_target_ref",
      "primary": false,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-authz/but-api test suite + cargo fmt/clippy",
        "negative_control": {
          "would_fail_if": [
            "the fixture edit introduced a TOML/heredoc syntax error (compile/clippy catches it)",
            "clippy flagged the idempotency change",
            "membership_read_only_from_target_ref regressed (it checks out feat independently — must stay green)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "self_grant_admin_head_on_feat",
            "action": {
              "actor": "ci",
              "steps": [
                "cargo test -p but-authz membership_read_only_from_target_ref",
                "cargo fmt --check",
                "cargo clippy -p but-authz -p but-api --all-targets"
              ]
            },
            "end_state": {
              "must_observe": [
                "membership_read_only_from_target_ref passes",
                "`cargo fmt --check` reports no diff",
                "`cargo clippy -p but-authz -p but-api --all-targets` reports no warnings"
              ],
              "must_not_observe": [
                "any GRPS-002 test regressing",
                "fmt or clippy failures"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "self_grant_admin_inert_until_landed passes with HEAD on feat-admin and land_admin_write committing onto main",
      "verify": "cargo test -p but-authz self_grant_admin_inert_until_landed",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "main_before != main_after still holds after land_admin_write (landing advances the target ref despite HEAD on feat-admin)",
      "verify": "cargo test -p but-authz self_grant_admin_inert_until_landed",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "with a transient HEAD-peel loader mutation, self_grant_admin_inert_until_landed FAILS at the before-land assert_denied (RED), proving restored teeth; restored clean afterward",
      "verify": "cat .tmp/FIX-GRPS-002-AC3-TEETH/head-peel-mutation.md",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "enforce_merge_gate self-add case still returns perm.denied with the idempotent remote-add",
      "verify": "cargo test -p but-api self_add_to_maintainers_on_feature_head_still_denied_merge",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "landed_membership_clears_merge_authority_step still clears the merge-authority step after the idempotent remote-add",
      "verify": "cargo test -p but-api landed_membership_clears_merge_authority_step",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "description": "membership_read_only_from_target_ref + fmt + clippy stay clean",
      "verify": "cargo test -p but-authz membership_read_only_from_target_ref ; cargo fmt --check ; cargo clippy -p but-authz -p but-api --all-targets",
      "maps_to_ac": "AC-4"
    }
  ]
}
-->
</details>
