# FIX-AUTHZ-FORGE-COVERAGE: AUTHZ-008 honesty gate has FALSE coverage over `forge.rs` — fully-qualify `authorize` + assert AUTHORITY_POSITIVE & PERMISSION_CARRIER over FORGE_GUARD

## What this does

Fixes a silent coverage hole (red-hat M-1). `crates/but-authz/tests/invariant_build_gates.rs` lists `FORGE_GUARD` (`forge.rs`) in `ENFORCEMENT_PATHS` (L29), so the NEGATIVE role-name / human-vs-AI greps already run over it (L46-57). But the POSITIVE "must call `but_authz::authorize`" assertion runs only over `COMMIT_GATE`/`MERGE_GATE`/`CONFIG_MUTATE` (L58-75), and the `PERMISSION_CARRIER` no-match runs only over `SPRINT_02_ENFORCEMENT_PATHS = [MERGE_GATE, CONFIG_MUTATE]` (L31, L82-87). Worse, `forge.rs` imports `authorize` BARE (`forge.rs:4` `use but_authz::{Authority, authorize, ...}`) and calls it bare at L60-65, which the `AUTHORITY_POSITIVE_PATTERN` (`but_authz::authorize|Authority::contains|but_authz::Authority`, L12-13) does NOT match. So the positive "keys-off-Authority" invariant and the no-Permission-carrier invariant are SILENTLY NOT enforced over `forge.rs`: it could be refactored to a role-keyed check and the build-gate stays green — false coverage the AUTHZ-008 honesty gate claims but does not deliver. This task (a) fully-qualifies the `authorize` call sites in `forge.rs` to `but_authz::authorize(...)` so the positive pattern bites; (b) adds an `assert_grep_has_matches(AUTHORITY_POSITIVE_PATTERN, &[FORGE_GUARD])`; and (c) brings `FORGE_GUARD` under the `PERMISSION_CARRIER_PATTERN` no-match assertion. After FIX-AUTHZ-FORGE-FAILCLOSED makes `forge.rs` a first-class enforcement surface, this makes its honesty coverage real.

## Why

Sprint 02 · PRD UC-AUTHZ-03, UC-AUTHZ-04 · capabilities CAP-AUTHZ-01. AUTHZ-008 is the honesty invariant build-gate: every enforcement surface must demonstrably key off the functional `but_authz` Authority axis (not a role/label) and must not overload the GitButler `Permission`/`RepoExclusive` lock as the authz carrier. AUTHZ-008 added `forge.rs` to `ENFORCEMENT_PATHS` but only wired the negative greps over it — leaving the positive and carrier invariants unenforced. A gate that claims coverage it does not deliver is exactly the reward-hack AUTHZ-008 exists to prevent; this task makes the gate tell the truth about `forge.rs`.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-authz invariant_build_gates` (the AUTHORITY_POSITIVE assertion now runs over FORGE_GUARD and PASSES because the calls are fully-qualified; a mutation replacing `but_authz::authorize` in forge.rs with a role-keyed check FAILS the gate). Full gate set in the spec below.

## Scope

- `crates/but-api/src/legacy/forge.rs` (MODIFY — minimal) — fully-qualify the `authorize` call sites (L60-65) to `but_authz::authorize(...)` (drop `authorize` from the bare `use` at L4 or keep it but call qualified) so `AUTHORITY_POSITIVE_PATTERN` matches. No behavior change.
- `crates/but-authz/tests/invariant_build_gates.rs` (MODIFY) — add `assert_grep_has_matches(AUTHORITY_POSITIVE_PATTERN, &[FORGE_GUARD])`; include `FORGE_GUARD` in the `PERMISSION_CARRIER_PATTERN` no-match assertion (extend `SPRINT_02_ENFORCEMENT_PATHS` to include `FORGE_GUARD`, or add a dedicated forge carrier assertion). OWNS closing the false-coverage hole.

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: FIX-AUTHZ-FORGE-COVERAGE - AUTHZ-008 honesty gate false coverage over forge.rs: fully-qualify authorize + assert AUTHORITY_POSITIVE & PERMISSION_CARRIER over FORGE_GUARD
================================================================================

TASK_TYPE:  BUGFIX
STATUS:     Backlog
PRIORITY:   P1
EFFORT:     S  (75 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ./SPRINT.md
PRD_REFS:   UC-AUTHZ-03, UC-AUTHZ-04
CAPABILITIES: CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-authz invariant_build_gates
  check: cargo check -p but-authz --all-targets && cargo check -p but-api --all-targets
  lint:  cargo clippy -p but-authz --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
The AUTHZ-008 honesty build-gate (crates/but-authz/tests/invariant_build_gates.rs) now ENFORCES the positive keys-off-Authority invariant AND the no-Permission-carrier invariant over forge.rs, not just the negative role/label greps. forge.rs's authorize call sites (L60-65) are FULLY-QUALIFIED (but_authz::authorize(...)) so AUTHORITY_POSITIVE_PATTERN (but_authz::authorize|Authority::contains|but_authz::Authority) matches; an added assert_grep_has_matches(AUTHORITY_POSITIVE_PATTERN, &[FORGE_GUARD]) PASSES; FORGE_GUARD is brought under the PERMISSION_CARRIER_PATTERN no-match assertion (no Permission/RepoExclusive carrier in forge.rs). A mutation that replaces forge.rs's but_authz::authorize with a role-keyed check (e.g. `if role == "reviewer"`) makes the gate FAIL — both because AUTHORITY_POSITIVE over FORGE_GUARD finds 0 matches and because the negative role-branch grep fires. `cargo test -p but-authz invariant_build_gates` is green after the fix; `cargo test -p but-api forge_guard` stays green (the fully-qualify is behavior-neutral). The seeded controls (role/label/carrier fixtures, L168-229) still fire, so the gate's own teeth are proven.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST fully-qualify the authorize call sites in forge.rs (currently bare `authorize(...)` at L60-65 because of the bare import `use but_authz::{Authority, authorize, ...}` at L4) so they read `but_authz::authorize(&principal, ..., &cfg)` — matching AUTHORITY_POSITIVE_PATTERN (invariant_build_gates.rs:12-13). This is the ONLY way the positive grep bites on forge.rs; a bare authorize scores 0. Behavior is identical (same fn, qualified path).
- [MUST] MUST add an assert_grep_has_matches(... AUTHORITY_POSITIVE_PATTERN, &[FORGE_GUARD]) to invariant_build_gates() (mirror the COMMIT_GATE/MERGE_GATE/CONFIG_MUTATE assertions at L58-75) so the forge boundary's keys-off-Authority is POSITIVELY enforced, not just negatively grep'd.
- [MUST] MUST bring FORGE_GUARD under the PERMISSION_CARRIER_PATTERN no-match assertion. Either extend SPRINT_02_ENFORCEMENT_PATHS (L31) to include FORGE_GUARD (so the existing assertion at L82-87 covers it) OR add a dedicated assert_grep_has_no_matches(PERMISSION_CARRIER_PATTERN, &[FORGE_GUARD]). After this, the no-Permission-carrier invariant is enforced over forge.rs.
- [MUST] MUST keep the fix behavior-NEUTRAL on the forge path: fully-qualifying authorize changes ONLY the call path syntax, not the authorization decision. cargo test -p but-api forge_guard MUST stay green (and, if sequenced after FIX-AUTHZ-FORGE-FAILCLOSED, the new forge_guard fail-closed tests too).
- [MUST] MUST prove the gate has TEETH via a documented mutation: temporarily replace one forge.rs but_authz::authorize call with a role-keyed branch (`if env BUT_AGENT_HANDLE == "reviewer" { Ok(()) }` style) and capture that `cargo test -p but-authz invariant_build_gates` FAILS (AUTHORITY_POSITIVE over FORGE_GUARD = 0 matches AND/OR the role-branch negative grep fires), then revert. This is the RED-against-mutation evidence for AC-1's negative_control.
- [NEVER] NEVER weaken the gate to make it pass — do NOT remove forge.rs from ENFORCEMENT_PATHS, do NOT loosen AUTHORITY_POSITIVE_PATTERN to match a bare authorize, do NOT delete the seeded controls. The fix is "make forge.rs satisfy the real invariant", not "make the invariant ignore forge.rs".
- [NEVER] NEVER introduce a role name / human-vs-AI label / Permission-carrier into forge.rs while fully-qualifying — the negative greps (already over FORGE_GUARD, L46-57) and the newly-added carrier no-match MUST stay clean.
- [STRICTLY] STRICTLY scope to the fully-qualify + the two added gate assertions. Do NOT change the opt-in discriminator (that is FIX-AUTHZ-FORGE-FAILCLOSED), do NOT re-architect authorize_branch_action's per-Authority match, do NOT touch CONFIG_MUTATE/MERGE_GATE/COMMIT_GATE coverage.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: AUTHORITY_POSITIVE is asserted over forge.rs and PASSES (calls fully-qualified); a documented mutation replacing forge.rs's but_authz::authorize with a role-keyed check FAILS the build-gate
- [ ] AC-2: PERMISSION_CARRIER no-match covers forge.rs and PASSES; the seeded carrier control still fires (the gate retains teeth)
- [ ] AC-3: forge.rs authorize calls are fully-qualified; `cargo test -p but-api forge_guard` stays green (behavior-neutral) (build-gate + integration)
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: AUTHORITY_POSITIVE is enforced over forge.rs and bites on a role-keyed mutation [PRIMARY]
  GIVEN: the source tree after fully-qualifying forge.rs's authorize calls AND adding assert_grep_has_matches(AUTHORITY_POSITIVE_PATTERN, &[FORGE_GUARD]) to invariant_build_gates()
  WHEN:  `cargo test -p but-authz invariant_build_gates` runs against (a) the fixed tree, then (b) a mutated tree where one forge.rs but_authz::authorize call is replaced with a role-keyed branch
  THEN:  (a) the gate PASSES — AUTHORITY_POSITIVE over FORGE_GUARD finds 1+ match (the fully-qualified call); (b) the gate FAILS — AUTHORITY_POSITIVE over FORGE_GUARD finds 0 matches and/or the role-branch negative grep fires
  TEST_TIER: integration (build-gate, real grep over real source)   VERIFICATION_SERVICE: cargo test executing the real invariant_build_gates grep harness over the real workspace source tree
  VERIFY: cargo test -p but-authz invariant_build_gates
  SCENARIO: NEGATIVE_CONTROL would fail if forge.rs's authorize stays BARE (AUTHORITY_POSITIVE over FORGE_GUARD = 0, so the new assertion never passes — the false-coverage hole persists); if the new assert_grep_has_matches(FORGE_GUARD) is not added (the positive invariant is still not enforced over forge.rs and the role-keyed mutation stays green); if AUTHORITY_POSITIVE_PATTERN is loosened to match a bare authorize (the teeth are removed).

AC-2: PERMISSION_CARRIER no-match covers forge.rs; the seeded carrier control still fires
  GIVEN: the source tree after bringing FORGE_GUARD under the PERMISSION_CARRIER_PATTERN no-match assertion
  WHEN:  `cargo test -p but-authz invariant_build_gates` runs
  THEN:  the no-Permission-carrier assertion over FORGE_GUARD PASSES (forge.rs has 0 write_permission(/RepoExclusive/Permission carrier matches) AND the seeded carrier fixture (L209-226) still produces matches (assert_seeded_controls_fire passes) — the gate's carrier teeth are real
  TEST_TIER: integration (build-gate)   VERIFICATION_SERVICE: cargo test executing the real grep harness + seeded-control fixtures
  VERIFY: cargo test -p but-authz invariant_build_gates
  SCENARIO: NEGATIVE_CONTROL would fail if FORGE_GUARD is not added to the carrier no-match set (forge.rs could overload the Permission lock undetected); if a Permission/RepoExclusive carrier is introduced into forge.rs (the assertion fires correctly); if assert_seeded_controls_fire is deleted/weakened so the carrier control no longer proves the pattern matches a real violation.

AC-3: forge.rs authorize is fully-qualified and the fix is behavior-neutral [build-gate]
  GIVEN: the source tree after this task
  WHEN:  the build-gate greps + the forge integration suite run
  THEN:  forge.rs contains the fully-qualified `but_authz::authorize` (1+) and NO bare `authorize(` call outside the qualified form (the bare import is removed or only qualified call sites remain); `cargo test -p but-api forge_guard` is green (the authorization decisions are unchanged)
  TEST_TIER: unit (build-gate) + integration (forge regression)   VERIFICATION_SERVICE: source grep (no runtime I/O) + real but-api forge seam   UNIT_TEST_JUSTIFIED: the fully-qualify is a structural call-form change verified by grep; the behavior-neutrality is proven by the forge_guard integration suite staying green. A runtime test alone cannot assert the call form is fully-qualified (the property AUTHORITY_POSITIVE depends on).
  VERIFY: grep -rEn 'but_authz::authorize' crates/but-api/src/legacy/forge.rs && cargo test -p but-api forge_guard
  SCENARIO: NEGATIVE_CONTROL would fail if forge.rs still calls authorize bare (AUTHORITY_POSITIVE can't bite); if the fully-qualify accidentally changed an Authority argument so a forge verb's decision flips (forge_guard regresses); if a forge verb is silently broken by the edit.

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, happy_path): AUTHORITY_POSITIVE asserted over FORGE_GUARD passes on the fixed tree; a role-keyed mutation of a forge.rs authorize call fails the build-gate (M-1 teeth)
    VERIFY: cargo test -p but-authz invariant_build_gates
- TC-2 (-> AC-2, structural): PERMISSION_CARRIER no-match covers FORGE_GUARD (forge.rs clean) and the seeded carrier control still fires (M-1 carrier coverage)
    VERIFY: cargo test -p but-authz invariant_build_gates
- TC-3 (-> AC-3, regression): forge.rs uses fully-qualified but_authz::authorize and forge_guard integration stays green (behavior-neutral)
    VERIFY: grep -rEn 'but_authz::authorize' crates/but-api/src/legacy/forge.rs && cargo test -p but-api forge_guard

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides: real honesty-gate coverage of forge.rs — the AUTHORITY_POSITIVE keys-off-Authority invariant and the no-Permission-carrier invariant are now POSITIVELY enforced over the forge boundary (not just the negative role/label greps), and forge.rs's authorize calls are fully-qualified so the positive grep bites. Closes the M-1 false-coverage hole; a role-keyed regression in forge.rs now fails the build-gate.
consumes: the existing invariant_build_gates harness (AUTHORITY_POSITIVE_PATTERN, PERMISSION_CARRIER_PATTERN, FORGE_GUARD const, assert_grep_has_matches/no_matches, assert_seeded_controls_fire); but_authz::authorize (the fully-qualified call form in forge.rs)
boundary_contracts:
  - CAP-AUTHZ-01: every enforcement surface in ENFORCEMENT_PATHS — INCLUDING forge.rs — must demonstrably key off the functional but_authz::authorize / Authority axis (positive grep) and must not overload the GitButler Permission/RepoExclusive lock as the authz carrier (carrier no-match). The honesty gate enforces both over forge.rs after this fix.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-api/src/legacy/forge.rs (MODIFY — minimal) — fully-qualify the authorize call sites (60-65) to but_authz::authorize(...); adjust the use at line 4 accordingly. Behavior-neutral.
  - crates/but-authz/tests/invariant_build_gates.rs (MODIFY) — add assert_grep_has_matches(AUTHORITY_POSITIVE_PATTERN, &[FORGE_GUARD]); bring FORGE_GUARD under the PERMISSION_CARRIER_PATTERN no-match (extend SPRINT_02_ENFORCEMENT_PATHS to include FORGE_GUARD, or add a forge carrier assertion). OWNS the false-coverage closure.
writeProhibited:
  - crates/but-authz/src/** — CONSUME authorize; do NOT change the authorize fn or the patterns' semantics (do NOT loosen AUTHORITY_POSITIVE_PATTERN to accept a bare authorize)
  - crates/but-api/src/legacy/forge.rs opt-in discriminator (has_governance_config / governance_present swap) — that is FIX-AUTHZ-FORGE-FAILCLOSED; do NOT change it here
  - crates/but-api/src/legacy/merge_gate.rs, config_mutate.rs, crates/but-api/src/commit/gate.rs — their coverage is already asserted; do NOT touch
  - any gitbutler-* crate (crates/AGENTS.md)
  - Any file not listed in write_allowed

--------------------------------------------------------------------------------
OUT OF SCOPE
--------------------------------------------------------------------------------
  - The forge opt-in fail-closed fix (has_governance_config -> governance_present) — FIX-AUTHZ-FORGE-FAILCLOSED. This task assumes forge.rs is (or is becoming) a first-class enforcement surface and makes its honesty coverage real; it does NOT change the discriminator.
  - Adding new enforcement surfaces to ENFORCEMENT_PATHS beyond forge.rs (already present).
  - The admin-write / merge / commit gate coverage (already asserted at L58-87).

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-authz/tests/invariant_build_gates.rs (10-31)
   Focus: the patterns + the const sets — AUTHORITY_POSITIVE_PATTERN (12-13: but_authz::authorize|Authority::contains|but_authz::Authority — NO bare authorize), PERMISSION_CARRIER_PATTERN (14-15), FORGE_GUARD (22), ENFORCEMENT_PATHS includes FORGE_GUARD (29), SPRINT_02_ENFORCEMENT_PATHS = [MERGE_GATE, CONFIG_MUTATE] (31 — forge.rs MISSING from the carrier set).
2. crates/but-authz/tests/invariant_build_gates.rs (33-92)
   Focus: invariant_build_gates() — negative greps run over ENFORCEMENT_PATHS incl. forge.rs (46-57); AUTHORITY_POSITIVE asserted only over COMMIT_GATE/MERGE_GATE/CONFIG_MUTATE (58-75, forge.rs MISSING); PERMISSION_CARRIER no-match over SPRINT_02_ENFORCEMENT_PATHS (82-87, forge.rs MISSING). ADD the forge AUTHORITY_POSITIVE assertion + bring forge.rs under the carrier no-match.
3. crates/but-authz/tests/invariant_build_gates.rs (142-229)
   Focus: assert_grep_has_matches (142) is the helper to mirror for the new forge assertion; assert_seeded_controls_fire (168) proves the patterns match real violations — must keep firing.
4. crates/but-api/src/legacy/forge.rs (1-68)
   Focus: THE FIX TARGET — line 4 imports authorize BARE (`use but_authz::{Authority, authorize, ...}`); authorize_branch_action calls it bare at 60-65 (the bare calls AUTHORITY_POSITIVE_PATTERN does NOT match). Fully-qualify these to but_authz::authorize(...).
5. crates/but-api/tests/forge_guard.rs (full)
   Focus: the regression suite that must stay green after the fully-qualify (forge_guard_authorizes_comments_and_records_approval, forge_guard_no_stub_success_for_unimplemented_review_actions) — the fully-qualify is behavior-neutral.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- Honesty gate green with forge coverage: `cargo test -p but-authz invariant_build_gates` -> Exit 0; AUTHORITY_POSITIVE over FORGE_GUARD + PERMISSION_CARRIER no-match over FORGE_GUARD + seeded controls all pass
- forge.rs authorize fully-qualified: `grep -rEn 'but_authz::authorize' crates/but-api/src/legacy/forge.rs` -> 1+ match; `! grep -rEn '[^:]\bauthorize\(' crates/but-api/src/legacy/forge.rs` (no bare authorize( call remains; reviewer confirms only the qualified form is called)
- Forge behavior unchanged: `cargo test -p but-api forge_guard` -> Exit 0 (and the FIX-AUTHZ-FORGE-FAILCLOSED tests if sequenced after)
- Gate teeth proven (mutation, documented + reverted): a role-keyed forge.rs authorize replacement makes `cargo test -p but-authz invariant_build_gates` FAIL -> captured in .tmp evidence, then reverted
- Crates compile incl. tests: `cargo check -p but-authz --all-targets && cargo check -p but-api --all-targets` -> Exit 0
- Clippy clean: `cargo clippy -p but-authz --all-targets` -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: Make-the-surface-satisfy-the-invariant (not weaken-the-invariant) — fully-qualify forge.rs's authorize calls to but_authz::authorize so AUTHORITY_POSITIVE_PATTERN bites, then mirror the COMMIT_GATE/MERGE_GATE/CONFIG_MUTATE positive assertions for FORGE_GUARD and bring FORGE_GUARD under the PERMISSION_CARRIER no-match (extend SPRINT_02_ENFORCEMENT_PATHS to [MERGE_GATE, CONFIG_MUTATE, FORGE_GUARD]). The teeth are proven by a documented role-keyed mutation that fails the gate, then reverted. The negative role/label greps already cover forge.rs (no change needed there).
pattern_source: crates/but-authz/tests/invariant_build_gates.rs:58-75 (the AUTHORITY_POSITIVE assertions to mirror) + :82-87 (the PERMISSION_CARRIER no-match to extend) + crates/but-api/src/legacy/forge.rs:4,60-65 (the bare authorize import/calls to fully-qualify) + crates/but-api/src/legacy/config_mutate.rs:25 (the fully-qualified form already used by the admin-write gate)
anti_pattern: Removing forge.rs from ENFORCEMENT_PATHS to dodge coverage; loosening AUTHORITY_POSITIVE_PATTERN to accept a bare authorize; deleting/weakening assert_seeded_controls_fire; changing an Authority argument while fully-qualifying (behavior drift); introducing a role-name/label/Permission-carrier into forge.rs; claiming the gate has teeth without the documented role-keyed mutation evidence.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — Fully-qualifies forge.rs's authorize call sites to but_authz::authorize (behavior-neutral) and extends the AUTHZ-008 honesty build-gate to POSITIVELY assert AUTHORITY_POSITIVE and the no-Permission-carrier invariant over FORGE_GUARD, closing the M-1 false-coverage hole. Proves the gate has teeth via a documented role-keyed mutation that fails invariant_build_gates (then reverts), keeps the seeded controls firing, and confirms forge_guard integration stays green. Owns the fully-qualify, the two added gate assertions, and the mutation-against-the-gate evidence.
reviewer: rust-reviewer
coding_standards: crates/AGENTS.md, crates/WORKSPACE_MODEL.md, crates/but-authz/tests/invariant_build_gates.rs

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: FIX-AUTHZ-FORGE-FAILCLOSED (after the opt-in fix forge.rs is a first-class enforcement surface whose honesty coverage must be real); AUTHZ-008 (the honesty gate this extends)
Blocks:     (none — final forge-boundary hardening for Sprint 02)
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "FIX-AUTHZ-FORGE-COVERAGE",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "notes": [
    "M-1: invariant_build_gates.rs lists FORGE_GUARD in ENFORCEMENT_PATHS (29) so the NEGATIVE role/label greps run over forge.rs (46-57), but AUTHORITY_POSITIVE is asserted only over COMMIT_GATE/MERGE_GATE/CONFIG_MUTATE (58-75) and PERMISSION_CARRIER no-match only over SPRINT_02_ENFORCEMENT_PATHS=[MERGE_GATE,CONFIG_MUTATE] (31,82-87). forge.rs imports authorize BARE (forge.rs:4) and calls it bare (60-65), which AUTHORITY_POSITIVE_PATTERN (12-13) does not match. So the positive keys-off-Authority and the carrier invariants are SILENTLY unenforced over forge.rs — false coverage.",
    "Fix is two-sided: (a) fully-qualify forge.rs authorize -> but_authz::authorize so the positive grep bites (behavior-neutral); (b) add assert_grep_has_matches(AUTHORITY_POSITIVE_PATTERN,&[FORGE_GUARD]) and bring FORGE_GUARD under the PERMISSION_CARRIER no-match (extend SPRINT_02_ENFORCEMENT_PATHS to include FORGE_GUARD).",
    "Teeth: the RED-against-mutation evidence is a documented role-keyed replacement of a forge.rs authorize call that makes cargo test -p but-authz invariant_build_gates FAIL (AUTHORITY_POSITIVE over FORGE_GUARD = 0 and/or role-branch negative grep fires), then reverted. The seeded carrier control (209-226) must keep firing.",
    "Do NOT weaken the invariant: keep forge.rs in ENFORCEMENT_PATHS, do NOT loosen AUTHORITY_POSITIVE_PATTERN to accept bare authorize, do NOT delete seeded controls. Make the surface satisfy the invariant.",
    "Sequence after FIX-AUTHZ-FORGE-FAILCLOSED so forge.rs is a first-class enforcement surface before its honesty coverage is asserted real."
  ],
  "fixtures": {
    "fixed_source_tree": {
      "description": "The real workspace source tree after fully-qualifying forge.rs's authorize calls (but_authz::authorize) and adding the AUTHORITY_POSITIVE + PERMISSION_CARRIER assertions over FORGE_GUARD in invariant_build_gates.rs. cargo test -p but-authz invariant_build_gates runs the real grep harness over this tree; no synthetic repo — the gate greps the actual crates/ source.",
      "seed_method": "manual",
      "records": [
        "edit crates/but-api/src/legacy/forge.rs: fully-qualify authorize call sites (60-65) to but_authz::authorize(...); adjust the use at line 4",
        "edit crates/but-authz/tests/invariant_build_gates.rs: add assert_grep_has_matches(AUTHORITY_POSITIVE_PATTERN, &[FORGE_GUARD]); extend SPRINT_02_ENFORCEMENT_PATHS (31) to [MERGE_GATE, CONFIG_MUTATE, FORGE_GUARD] (or add a dedicated forge carrier no-match assertion)"
      ]
    },
    "role_keyed_mutation_tree": {
      "description": "A temporary mutation of fixed_source_tree where one forge.rs but_authz::authorize call is replaced with a role-keyed branch (e.g. resolving BUT_AGENT_HANDLE and returning Ok if it equals a hard-coded role string), to prove the gate has teeth. cargo test -p but-authz invariant_build_gates MUST FAIL against this tree (AUTHORITY_POSITIVE over FORGE_GUARD drops to 0 and/or the ROLE_BRANCH negative grep fires). Reverted immediately after capture.",
      "seed_method": "manual",
      "records": [
        "temporarily replace a forge.rs but_authz::authorize(...) call with a role-keyed `if handle == \"reviewer\" { return Ok(()); }`-style branch",
        "run cargo test -p but-authz invariant_build_gates and capture the FAILURE output to .tmp evidence",
        "git checkout crates/but-api/src/legacy/forge.rs (revert the mutation)"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN forge.rs's authorize calls fully-qualified to but_authz::authorize AND an added assert_grep_has_matches(AUTHORITY_POSITIVE_PATTERN, &[FORGE_GUARD]) WHEN cargo test -p but-authz invariant_build_gates runs against (a) the fixed tree then (b) a role-keyed-mutation tree THEN (a) PASSES (AUTHORITY_POSITIVE over FORGE_GUARD finds 1+ match) and (b) FAILS (AUTHORITY_POSITIVE over FORGE_GUARD = 0 matches and/or the role-branch negative grep fires) — the positive keys-off-Authority invariant is now really enforced over forge.rs and bites on a role-keyed regression",
      "verify": "cargo test -p but-authz invariant_build_gates",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "cargo test executing the real invariant_build_gates grep harness over the real workspace source tree",
        "negative_control": {
          "would_fail_if": [
            "forge.rs's authorize stays BARE so AUTHORITY_POSITIVE over FORGE_GUARD finds 0 matches — the new assert_grep_has_matches(FORGE_GUARD) never passes and the false-coverage hole persists",
            "the assert_grep_has_matches(AUTHORITY_POSITIVE_PATTERN, &[FORGE_GUARD]) is not added — the positive invariant remains unenforced over forge.rs and a role-keyed mutation of forge.rs stays GREEN (the M-1 hole)",
            "AUTHORITY_POSITIVE_PATTERN is loosened to match a bare authorize — the gate's teeth are removed",
            "forge.rs is removed from ENFORCEMENT_PATHS to dodge coverage"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "fixed_source_tree",
            "action": {
              "actor": "ci",
              "steps": [
                "run cargo test -p but-authz invariant_build_gates against the fixed tree — AUTHORITY_POSITIVE asserted over FORGE_GUARD must pass (cite M-1, AUTHORITY_POSITIVE_PATTERN invariant_build_gates.rs:12-13)"
              ]
            },
            "end_state": {
              "must_observe": [
                "the test passes; the AUTHORITY_POSITIVE assertion over FORGE_GUARD finds 1+ match (the fully-qualified but_authz::authorize)",
                "the negative role/label greps over FORGE_GUARD still pass (forge.rs has no role/label branch)"
              ],
              "must_not_observe": [
                "0 AUTHORITY_POSITIVE matches over FORGE_GUARD (forge.rs authorize still bare)",
                "the gate skipping forge.rs (forge.rs removed from the asserted set)"
              ]
            }
          },
          {
            "start_ref": "role_keyed_mutation_tree",
            "action": {
              "actor": "ci",
              "steps": [
                "temporarily replace a forge.rs but_authz::authorize call with a role-keyed branch and run cargo test -p but-authz invariant_build_gates — the gate must FAIL, then revert (cite M-1 teeth)"
              ]
            },
            "end_state": {
              "must_observe": [
                "cargo test -p but-authz invariant_build_gates FAILS (AUTHORITY_POSITIVE over FORGE_GUARD = 0 matches and/or the ROLE_BRANCH_PATTERN negative grep fires)",
                "the mutation is reverted after capture (git checkout forge.rs)"
              ],
              "must_not_observe": [
                "the gate staying GREEN under a role-keyed forge.rs mutation (which would mean the invariant is still not enforced over forge.rs — the false-coverage hole)"
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
      "description": "GIVEN FORGE_GUARD brought under the PERMISSION_CARRIER_PATTERN no-match assertion WHEN cargo test -p but-authz invariant_build_gates runs THEN the no-Permission-carrier assertion over FORGE_GUARD passes (forge.rs has 0 write_permission(/RepoExclusive/Permission carrier matches) AND assert_seeded_controls_fire still passes (the carrier fixture still produces matches) — the carrier invariant is enforced over forge.rs with the gate's teeth intact",
      "verify": "cargo test -p but-authz invariant_build_gates",
      "maps_to_ac": null,
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "cargo test executing the real grep harness + the seeded-control fixtures",
        "negative_control": {
          "would_fail_if": [
            "FORGE_GUARD is not added to the PERMISSION_CARRIER no-match set so forge.rs could overload the GitButler Permission/RepoExclusive lock undetected",
            "a Permission/RepoExclusive carrier is introduced into forge.rs (the assertion then correctly fires — proving it is live)",
            "assert_seeded_controls_fire is deleted or weakened so the carrier control no longer proves PERMISSION_CARRIER_PATTERN matches a real violation"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "fixed_source_tree",
            "action": {
              "actor": "ci",
              "steps": [
                "run cargo test -p but-authz invariant_build_gates — the PERMISSION_CARRIER no-match over FORGE_GUARD must pass and the seeded carrier control must still fire (cite M-1, PERMISSION_CARRIER_PATTERN invariant_build_gates.rs:14-15, assert_seeded_controls_fire 168-229)"
              ]
            },
            "end_state": {
              "must_observe": [
                "the PERMISSION_CARRIER no-match assertion covering FORGE_GUARD passes (forge.rs has 0 carrier matches)",
                "assert_seeded_controls_fire passes (the seeded carrier fixture still produces matches — the pattern has teeth)"
              ],
              "must_not_observe": [
                "FORGE_GUARD absent from the carrier no-match set (the carrier invariant still unenforced over forge.rs)",
                "the seeded carrier control no longer firing (the pattern's teeth removed)"
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
      "description": "GIVEN the source tree after this task WHEN the build-gate greps + the forge integration suite run THEN forge.rs contains the fully-qualified but_authz::authorize (1+) with no bare authorize( call outside the qualified form, and cargo test -p but-api forge_guard is green — the fully-qualify is behavior-neutral [build-gate + integration regression]",
      "verify": "grep -rEn 'but_authz::authorize' crates/but-api/src/legacy/forge.rs && cargo test -p but-api forge_guard",
      "maps_to_ac": null,
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "unit_test_justified": "The fully-qualify is a structural call-form change verified by grep (a runtime test cannot assert the call form is fully-qualified — the property AUTHORITY_POSITIVE depends on); the behavior-neutrality is proven by the forge_guard integration suite staying green. Both halves are required.",
        "verification_service": "source grep (no runtime I/O) + real but-api forge seam (forge_guard integration)",
        "negative_control": {
          "would_fail_if": [
            "forge.rs still calls authorize bare (AUTHORITY_POSITIVE cannot bite — AC-1 fails)",
            "the fully-qualify accidentally changed an Authority argument so a forge verb's decision flips (forge_guard regresses — a behavior change masquerading as a syntax fix)",
            "a forge verb is silently broken by the edit (forge_guard fails)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "fixed_source_tree",
            "action": {
              "actor": "ci",
              "steps": [
                "grep forge.rs for but_authz::authorize (1+); run cargo test -p but-api forge_guard (must stay green) (cite M-1 behavior-neutral)"
              ]
            },
            "end_state": {
              "must_observe": [
                "`grep -rEn 'but_authz::authorize' crates/but-api/src/legacy/forge.rs` returns 1+ matches",
                "`cargo test -p but-api forge_guard` exits 0 (forge authorization decisions unchanged)"
              ],
              "must_not_observe": [
                "0 but_authz::authorize matches in forge.rs (still bare)",
                "any forge_guard test failing (behavior drift introduced by the edit)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "AUTHORITY_POSITIVE asserted over FORGE_GUARD passes on the fixed tree; a role-keyed mutation of a forge.rs authorize call fails the build-gate (M-1 teeth)",
      "verify": "cargo test -p but-authz invariant_build_gates",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "PERMISSION_CARRIER no-match covers FORGE_GUARD (forge.rs clean) and the seeded carrier control still fires (M-1 carrier coverage with teeth)",
      "verify": "cargo test -p but-authz invariant_build_gates",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "forge.rs uses fully-qualified but_authz::authorize and the forge_guard integration suite stays green (behavior-neutral fully-qualify)",
      "verify": "grep -rEn 'but_authz::authorize' crates/but-api/src/legacy/forge.rs && cargo test -p but-api forge_guard",
      "maps_to_ac": "AC-3"
    }
  ]
}
-->
</details>
