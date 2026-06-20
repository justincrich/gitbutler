# AUTHZ-007: Invariant build-gates — no role-preset branch, no human-vs-AI predicate, no `Permission` overload (seeded-violation controlled)

## What this does

Add the three non-negotiable honesty build-gates as deterministic grep/structural assertions over the Sprint-01a source: (1) no role-preset name in any enforcement path, (2) no human-vs-AI / role-label enforcement branch, (3) no overload of GitButler's `Permission` repo-lock by the new `Authority` axis — each a CI gate that blocks the slice on violation regardless of green integration lanes.

## Why

Sprint 01a · PRD UC-AUTHZ-02, UC-LOOP-01, UC-LOOP-02 · capabilities CAP-AUTHZ-01. Part of the functional-permission governance walking skeleton (commit allow/deny through real `but-authz` + real git).

## How to verify

PRIMARY **AC-1** — `test -s crates/but-authz/src/authorize.rs && test -s crates/but-api/src/commit/gate.rs && ! grep -rEn '== *"(read|triage|write|maintain|admin)"|"(read|triage|write|maintain|admin)" *=>|match[^;]*\brole\b|\bfrom_role\(' crates/but-authz/src/authorize.rs crates/but-authz/src/config.rs crates/but-api/src/commit/gate.rs` (build-gate). Full gate set in the spec below.

## Scope

- crates/but-authz/tests/invariant_build_gates.rs (NEW) — the committed harness that runs the three greps and asserts clean/violation
- .github/workflows/** (MODIFY, optional) — add a CI step invoking the harness if the project wires build-gates in CI separately from cargo test

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: AUTHZ-007 - Invariant build-gates — no role-preset branch, no human-vs-AI predicate, no `Permission` overload (seeded-violation controlled)
================================================================================

TASK_TYPE:  BUILD_GATE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     S  (120 min)
AGENT:      implementer=rust-reviewer | reviewer=security-auditor
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-AUTHZ-02, UC-LOOP-01, UC-LOOP-02
CAPABILITIES: CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  test -s crates/but-authz/src/authorize.rs && test -s crates/but-api/src/commit/gate.rs && ! grep -rEn '== *"(read|triage|write|maintain|admin)"|"(read|triage|write|maintain|admin)" *=>|match[^;]*\brole\b|\bfrom_role\(' crates/but-authz/src/authorize.rs crates/but-authz/src/config.rs crates/but-api/src/commit/gate.rs
  check: cargo check -p but-authz --all-targets
  lint:  cargo clippy --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
A committed assertion harness (a `#[test]` in but-authz that shells the greps, or a CI step) runs all three invariants and exits 0 on the correct implementation: the role-name grep over enforcement source finds NO matches; the human-vs-AI / label grep finds NO matches; and the structural check confirms `but_authz::Authority`/`AuthoritySet` exist and the commit-gate does NOT use `Permission`/`write_permission`/`RepoExclusive` as the authority carrier. Each grep's expected result is documented and reproducible by command.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST be deterministic grep/structural assertions with EXACT commands and expected empty/non-empty output — NOT TDD feature tests, NOT product scenarios.
- [MUST] MUST assert no enforcement path in but-authz or the commit-gate references a role-preset name (read/triage/write/maintain/admin) as a branch condition — desugar is config-layer only (T-AUTHZ-016).
- [MUST] MUST assert no enforcement branch keys off a human-vs-AI predicate or the labels implementer/reviewer/maintainer (T-LOOP-005, T-LOOP-011) and that the new authorization axis does not overload GitButler's repo-access `Permission`/lock type (the `Authority` types live in but-authz, distinct from `Permission`/`_with_perm`/`RepoExclusive`).
- [NEVER] NEVER weaken a failing grep by narrowing its path scope to hide a real violation — if a grep finds a role/human-AI/Permission-overload in enforcement, that is a real defect to send back to the owning task, not to suppress.
- [NEVER] NEVER add product behavior here — this task only adds assertion scripts/tests; it writes no enforcement logic.
- [STRICTLY] STRICTLY scope the role/label greps to ENFORCEMENT source (but-authz authorize/config-check + the commit-gate), excluding the desugar catalog (authority.rs `from_role`) and tests/fixtures where role strings legitimately appear as CONFIG input.
- [STRICTLY] STRICTLY make each gate fail closed: an unexpected match (or unexpected absence of the distinct `but_authz::Authority` axis) exits nonzero so CI blocks the slice.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: No role-preset name in an ENFORCEMENT BRANCH (grep-asserted, branch-position only) [PRIMARY]
- [ ] AC-2: No human-vs-AI or role-label ENFORCEMENT branch (grep-asserted)
- [ ] AC-3: The new Authority axis does not overload GitButler's `Permission` repo-lock AS THE AUTHORITY CARRIER (structural, positive+negative)
- [ ] AC-4: The invariant harness exists, asserts path-existence, FIRES on a seeded violation, and blocks CI
- [ ] All verification gates pass; only write_allowed files modified (git diff --name-only)

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: No role-preset name in an ENFORCEMENT BRANCH (grep-asserted, branch-position only) [PRIMARY] [PRIMARY]
  GIVEN: the Sprint-01a enforcement source (but-authz authorize.rs + config.rs enforcement branches + the commit-gate crates/but-api/src/commit/gate.rs), excluding the desugar catalog (authority.rs from_role) and tests
  WHEN:  the role-name-in-branch grep gate runs (matching role presets only as quoted match-arm/comparison literals or `match .*role`, NOT bare word presence; bare `write`/`read` are NOT matched)
  THEN:  it finds zero matches of a role-preset name used as an enforcement branch condition (exit 0 = clean); a match exits nonzero and names the offending file:line
  TEST_TIER: build-gate   VERIFICATION_SERVICE: source-invariant grep (deterministic source assertion; process/fs I/O only — no product/network/db)
  UNIT_TEST_JUSTIFIED: UNIT_TEST_JUSTIFIED: this is a deterministic SOURCE-INVARIANT assertion (T-AUTHZ-016 build-gate), not a behavioral product/network/db test. It does perform process/fs I/O (it shells `grep` and `test -s`); it is NOT 'zero I/O'. There is no product surface to seed; the assertion is over source text and is deterministic. The branch-position regex + path-existence guard are what make it bite rather than false-pass.
  VERIFY: test -s crates/but-authz/src/authorize.rs && test -s crates/but-api/src/commit/gate.rs && ! grep -rEn '== *"(read|triage|write|maintain|admin)"|"(read|triage|write|maintain|admin)" *=>|match[^;]*\brole\b|\bfrom_role\(' crates/but-authz/src/authorize.rs crates/but-authz/src/config.rs crates/but-api/src/commit/gate.rs

AC-2: No human-vs-AI or role-label ENFORCEMENT branch (grep-asserted)
  GIVEN: the Sprint-01a enforcement source (authorize.rs + config.rs + gate.rs)
  WHEN:  the human-vs-AI / role-label grep gate runs (`is_human`/`is_ai` matched bare — never legitimate; `human`/`implementer`/`reviewer`/`maintainer` matched only in branch/comparison position so the `code-reviewers` group name and the "reviewed merge" hint text do not false-fail)
  THEN:  it finds zero matches of a human-vs-AI predicate or a role label used as an enforcement branch (exit 0 = clean); a match exits nonzero
  TEST_TIER: build-gate   VERIFICATION_SERVICE: source-invariant grep (deterministic source assertion; process/fs I/O only)
  UNIT_TEST_JUSTIFIED: UNIT_TEST_JUSTIFIED: deterministic source-invariant grep (T-LOOP-005/T-LOOP-011 build-gate); enforcement must not branch on human-vs-AI or role labels — asserted structurally over source with process/fs I/O only (shells grep), no product/network/db. `is_human`/`is_ai` are matched bare (never legitimate); role labels only in branch position so the legitimate group name/hint text don't false-fail.
  VERIFY: test -s crates/but-authz/src/authorize.rs && test -s crates/but-api/src/commit/gate.rs && ! grep -rEn 'is_human|is_ai|== *"(human|implementer|reviewer|maintainer)"|"(human|implementer|reviewer|maintainer)" *=>' crates/but-authz/src/authorize.rs crates/but-authz/src/config.rs crates/but-api/src/commit/gate.rs

AC-3: The new Authority axis does not overload GitButler's `Permission` repo-lock AS THE AUTHORITY CARRIER (structural, positive+negative)
  GIVEN: the but-authz crate and the commit-gate source (crates/but-api/src/commit/gate.rs)
  WHEN:  the no-overload structural gate runs
  THEN:  it confirms (positive) `but_authz::authorize` or `Authority::contains` IS present in the gate AND (negative) the gate does NOT carry authority via `write_permission(`, `RepoExclusive`, or a compound `Permission`/`Permissions` used as the authority carrier (exit 0 = clean; an overload or a missing positive exits nonzero)
  TEST_TIER: build-gate   VERIFICATION_SERVICE: source-invariant structural grep (deterministic source assertion; process/fs I/O only)
  UNIT_TEST_JUSTIFIED: UNIT_TEST_JUSTIFIED: structural source assertion (no-Permission-as-carrier invariant from 02-system-components.md); proves the new axis is distinct from the repo-access lock by source inspection (process/fs I/O via grep), not by running product behavior. It asserts the POSITIVE (but_authz::authorize/Authority::contains present) and bans Permission USED AS the carrier (write_permission(/RepoExclusive/compound Permission used as carrier), not the bare token `Permission`.
  VERIFY: test -s crates/but-api/src/commit/gate.rs && grep -rEn 'but_authz::authorize|Authority::contains|but_authz::Authority' crates/but-api/src/commit/gate.rs && ! grep -rEn 'write_permission\(|RepoExclusive|\bPermissions?\b *[:.][^=]' crates/but-api/src/commit/gate.rs

AC-4: The invariant harness exists, asserts path-existence, FIRES on a seeded violation, and blocks CI
  GIVEN: the three grep gates above plus a deliberately-seeded violation fixture the harness writes to a temp file
  WHEN:  they are bundled into a committed, runnable harness (a `#[test]` shelling the greps) that (a) asserts each grepped path exists+non-empty, (b) runs the negated greps on clean source, and (c) runs each grep against a planted `if role == "admin"` / `write_permission(`-as-carrier temp fixture
  THEN:  running the harness exits 0 on clean source AND proves each grep returns NONZERO (fires) on the seeded-violation fixture; if any grepped path is missing/empty the harness fails LOUD (not vacuously) — so the slice is blocked even if integration lanes are green
  TEST_TIER: build-gate   VERIFICATION_SERVICE: source-invariant grep harness (deterministic source assertion; process/fs I/O only) with a seeded-violation control
  UNIT_TEST_JUSTIFIED: UNIT_TEST_JUSTIFIED: the harness is a deterministic wrapper around the grep invariants (process/fs I/O via std::process::Command + a temp fixture file; no product/network/db). Its job is to make the structural invariants executable + CI-blocking, prove path-existence, and prove each grep BITES on a seeded violation — not to exercise product behavior.
  VERIFY: cargo test -p but-authz invariant_build_gates

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, happy_path): role-preset grep (branch-position only) over enforcement source (authorize.rs + config.rs + gate.rs) returns no matches; bare write/read are NOT matched (T-AUTHZ-016)
    VERIFY: test -s crates/but-authz/src/authorize.rs && test -s crates/but-api/src/commit/gate.rs && ! grep -rEn '== *"(read|triage|write|maintain|admin)"|"(read|triage|write|maintain|admin)" *=>|match[^;]*\brole\b|\bfrom_role\(' crates/but-authz/src/authorize.rs crates/but-authz/src/config.rs crates/but-api/src/commit/gate.rs
- TC-2 (-> AC-2, happy_path): human-vs-AI/role-label grep over enforcement source returns no matches; is_human/is_ai bare, labels in branch-position only (T-LOOP-005/T-LOOP-011)
    VERIFY: test -s crates/but-authz/src/authorize.rs && test -s crates/but-api/src/commit/gate.rs && ! grep -rEn 'is_human|is_ai|== *"(human|implementer|reviewer|maintainer)"|"(human|implementer|reviewer|maintainer)" *=>' crates/but-authz/src/authorize.rs crates/but-authz/src/config.rs crates/but-api/src/commit/gate.rs
- TC-3 (-> AC-3, edge): commit-gate uses but_authz::authorize/Authority::contains (positive) and NOT write_permission(/RepoExclusive/Permission-as-carrier (no-overload invariant)
    VERIFY: test -s crates/but-api/src/commit/gate.rs && grep -rEn 'but_authz::authorize|Authority::contains|but_authz::Authority' crates/but-api/src/commit/gate.rs && ! grep -rEn 'write_permission\(|RepoExclusive|\bPermissions?\b *[:.][^=]' crates/but-api/src/commit/gate.rs
- TC-4 (-> AC-4, happy_path): the bundled harness asserts path-existence, exits 0 on clean source, and proves each grep FIRES (nonzero) on a seeded if role == "admin" / write_permission(-as-carrier fixture
    VERIFY: cargo test -p but-authz invariant_build_gates

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides: CI build-gate asserting the three honesty invariants (no role name in enforcement, no human-vs-AI predicate, no Permission-axis overload)
consumes: but-authz source (AUTHZ-001/002/003); commit-gate source (GATES-001) — crates/but-api/src/commit/gate.rs + crates/but-authz/src/config.rs
boundary_contracts:
  - CAP-AUTHZ-01 failure-mode guard: 'role string present in the check → invariant violation (grep-assert)' — this task is that grep-assert, plus the no-human/AI-branch and no-Permission-overload structural invariants

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-authz/tests/invariant_build_gates.rs (NEW) — the committed harness that runs the three greps and asserts clean/violation
  - .github/workflows/** (MODIFY, optional) — add a CI step invoking the harness if the project wires build-gates in CI separately from cargo test
writeProhibited:
  - crates/but-authz/src/** — this is a reviewer/build-gate task; it adds assertions, never enforcement logic (changing source to pass a grep would be the inverse of the intent)
  - crates/but-workspace/src/** — do not edit the gate to satisfy the invariant; a real violation goes back to GATES-001
  - any product source — assertions only
  - Any file not listed in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. .spec/prds/governance/11-e2e-testing-criteria.md (lines 38-39)
   Focus: PRIMARY — T-AUTHZ-015/016 build-gate definitions ('grep finds no role string in enforcement'); plus T-LOOP-005 (line 132) and T-LOOP-011 (line 143) for the human-vs-AI / role-label invariants.
2. .spec/prds/governance/10-technical-requirements/02-system-components.md (lines 25-37)
   Focus: The naming-collision guardrail table (Permission repo-lock vs the new Authority axis; `_with_perm` vs `_with_authz`) — the exact identifiers AC-3 asserts are NOT overloaded.
3. crates/but-authz/src/authority.rs (lines 1-40)
   Focus: Confirm role strings live ONLY in `from_role` (the desugar catalog) which AC-1's grep MUST exclude from the enforcement scope — scope the gate to authorize.rs + gate.rs, not authority.rs.
4. crates/but/tests/but/utils.rs (lines 1-20)
   Focus: Pattern for a committed `#[test]` that asserts a repo-wide structural invariant (cf. `assert_ignored_tests_have_linear_ticket`) — shape the invariant harness the same way (a test that shells greps and asserts empty output).
5. /Users/justinrich/Projects/brain/docs/rust/testing.md (lines 1-60)
   Focus: `#[test]` + `std::process::Command`/`assert!` for a source-invariant gate; assert on grep exit status + captured stdout (the offending file:line on failure).

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- Invariant harness passes on clean source and fires on seeded violation: `cargo test -p but-authz invariant_build_gates`  -> Exit 0 — paths asserted present, all three invariants clean on real source, and each grep proven to fire nonzero on the seeded-violation fixture
- Role-name-in-branch grep is clean: `test -s crates/but-authz/src/authorize.rs && test -s crates/but-api/src/commit/gate.rs && ! grep -rEn '== *"(read|triage|write|maintain|admin)"|"(read|triage|write|maintain|admin)" *=>|match[^;]*\brole\b|\bfrom_role\(' crates/but-authz/src/authorize.rs crates/but-authz/src/config.rs crates/but-api/src/commit/gate.rs`  -> Exit 0 — paths present and no role-preset in enforcement-branch position (bare write/read not matched; the `!` makes a no-match exit 0)
- Human-vs-AI/label grep is clean: `test -s crates/but-authz/src/authorize.rs && test -s crates/but-api/src/commit/gate.rs && ! grep -rEn 'is_human|is_ai|== *"(human|implementer|reviewer|maintainer)"|"(human|implementer|reviewer|maintainer)" *=>' crates/but-authz/src/authorize.rs crates/but-authz/src/config.rs crates/but-api/src/commit/gate.rs`  -> Exit 0 — paths present and no human-vs-AI predicate or role label in enforcement-branch position (is_human/is_ai bare; group name/hint text not false-failed)
- No-Permission-as-carrier structural check (positive+negative): `test -s crates/but-api/src/commit/gate.rs && grep -rEn 'but_authz::authorize|Authority::contains|but_authz::Authority' crates/but-api/src/commit/gate.rs && ! grep -rEn 'write_permission\(|RepoExclusive|\bPermissions?\b *[:.][^=]' crates/but-api/src/commit/gate.rs`  -> Exit 0 — path present, but_authz::authorize/Authority::contains present (positive), and Permission NOT used as the authority carrier (write_permission(/RepoExclusive/compound Permission absent)

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references: 11-e2e-testing-criteria.md T-AUTHZ-016 / T-LOOP-005 / T-LOOP-011 (the build-gate criteria); 02-system-components.md naming-collision guardrail (Authority vs Permission); crates/but/tests/but/utils.rs assert_ignored_tests_have_linear_ticket (repo-wide structural-invariant test precedent)
notes:
  - Implement as a committed `#[test] fn invariant_build_gates()` in but-authz that runs each grep via std::process::Command from the workspace root (resolve root via CARGO_MANIFEST_DIR like utils.rs make_absolute), asserting exit status (no-match = success for the negated greps) and printing the offending file:line on failure.
  - Scope discipline: the role-name grep targets ONLY authorize.rs + commit_engine/gate.rs; it must NOT scan authority.rs (where `from_role`/role strings legitimately live as the CONFIG-layer desugar) nor tests/fixtures (role strings as config input).
  - This task is sequenced last because it greps the source produced by AUTHZ-001/002/003 + GATES-001; it cannot pass until those files exist (the gate.rs path is created by GATES-001).
pattern: Source-invariant `#[test]` harness wrapping deterministic greps; negated grep (no match) = pass, match = fail with file:line. No product behavior, no fixtures.
pattern_source: crates/but/tests/but/utils.rs (assert_ignored_tests_have_linear_ticket structural-invariant precedent)
anti_pattern: Editing enforcement source to dodge a grep (e.g. renaming a role-keyed branch instead of removing it), or scoping the grep so narrowly it can never find a real violation — both defeat the honesty invariant.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-reviewer — These are source-invariant grep/structural assertions asserted in CI — not behavioral feature tests. rust-reviewer owns adversarial structural validation (grep patterns, clippy, anti-pattern detection) and is the right owner for a non-TDD build-gate that judges the OTHER tasks' source.
reviewer: security-auditor
coding_standards: crates/AGENTS.md, /Users/justinrich/Projects/brain/docs/rust/testing.md

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: AUTHZ-001, AUTHZ-002, AUTHZ-003, GATES-001
Blocks:     none
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "AUTHZ-007",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": false,
    "requires_red_evidence": false,
    "requires_seeded_evidence": false
  },
  "fixtures": {},
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN the enforcement source (authorize.rs + config.rs + gate.rs, paths asserted present) WHEN the role-name-in-branch grep runs THEN zero matches of a role-preset name in enforcement-branch position; bare write/read not matched (T-AUTHZ-016)",
      "verify": "test -s crates/but-authz/src/authorize.rs && test -s crates/but-api/src/commit/gate.rs && ! grep -rEn '== *\"(read|triage|write|maintain|admin)\"|\"(read|triage|write|maintain|admin)\" *=>|match[^;]*\\brole\\b|\\bfrom_role\\(' crates/but-authz/src/authorize.rs crates/but-authz/src/config.rs crates/but-api/src/commit/gate.rs",
      "maps_to_ac": null
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "GIVEN the enforcement source (authorize.rs + config.rs + gate.rs) WHEN the human-vs-AI/role-label grep runs THEN zero matches; is_human/is_ai matched bare, role labels only in branch position so the code-reviewers group name and reviewed-merge hint text don't false-fail (T-LOOP-005/T-LOOP-011)",
      "verify": "test -s crates/but-authz/src/authorize.rs && test -s crates/but-api/src/commit/gate.rs && ! grep -rEn 'is_human|is_ai|== *\"(human|implementer|reviewer|maintainer)\"|\"(human|implementer|reviewer|maintainer)\" *=>' crates/but-authz/src/authorize.rs crates/but-authz/src/config.rs crates/but-api/src/commit/gate.rs",
      "maps_to_ac": null,
      "primary": false
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "GIVEN the commit-gate source (crates/but-api/src/commit/gate.rs) WHEN the no-overload structural gate runs THEN it confirms the POSITIVE (but_authz::authorize/Authority::contains present) AND bans Permission USED AS the authority carrier (write_permission(/RepoExclusive/compound Permission used as carrier), not the bare token",
      "verify": "test -s crates/but-api/src/commit/gate.rs && grep -rEn 'but_authz::authorize|Authority::contains|but_authz::Authority' crates/but-api/src/commit/gate.rs && ! grep -rEn 'write_permission\\(|RepoExclusive|\\bPermissions?\\b *[:.][^=]' crates/but-api/src/commit/gate.rs",
      "maps_to_ac": null,
      "primary": false
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "description": "GIVEN the three grep gates plus a seeded-violation fixture WHEN bundled into a committed harness THEN it asserts path-existence, exits 0 on clean source, and proves each grep FIRES (nonzero) on the seeded violation — failing LOUD if any grepped path is missing/empty, blocking CI",
      "verify": "cargo test -p but-authz invariant_build_gates",
      "maps_to_ac": null,
      "primary": false
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "no role-preset name in enforcement-branch position; bare write/read not matched; paths asserted present",
      "verify": "test -s crates/but-authz/src/authorize.rs && test -s crates/but-api/src/commit/gate.rs && ! grep -rEn '== *\"(read|triage|write|maintain|admin)\"|\"(read|triage|write|maintain|admin)\" *=>|match[^;]*\\brole\\b|\\bfrom_role\\(' crates/but-authz/src/authorize.rs crates/but-authz/src/config.rs crates/but-api/src/commit/gate.rs",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "no human-vs-AI/role-label branch in enforcement; is_human/is_ai bare, labels in branch position only",
      "verify": "test -s crates/but-authz/src/authorize.rs && test -s crates/but-api/src/commit/gate.rs && ! grep -rEn 'is_human|is_ai|== *\"(human|implementer|reviewer|maintainer)\"|\"(human|implementer|reviewer|maintainer)\" *=>' crates/but-authz/src/authorize.rs crates/but-authz/src/config.rs crates/but-api/src/commit/gate.rs",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "gate uses but_authz::authorize/Authority::contains (positive) and not write_permission(/RepoExclusive/Permission-as-carrier",
      "verify": "test -s crates/but-api/src/commit/gate.rs && grep -rEn 'but_authz::authorize|Authority::contains|but_authz::Authority' crates/but-api/src/commit/gate.rs && ! grep -rEn 'write_permission\\(|RepoExclusive|\\bPermissions?\\b *[:.][^=]' crates/but-api/src/commit/gate.rs",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "harness asserts path-existence, exits 0 clean, and proves each grep FIRES on a seeded violation",
      "verify": "cargo test -p but-authz invariant_build_gates",
      "maps_to_ac": "AC-4"
    }
  ]
}
-->
</details>
