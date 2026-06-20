# AUTHZ-008: Honesty-invariant build-gate extension — ENFORCEMENT_PATHS to merge_gate.rs + config_mutate.rs + admin-write AND merge-gate AUTHORITY_POSITIVE assertions + len>=5 + seeded controls preserved

## What this does

EXTENDS `crates/but-authz/tests/invariant_build_gates.rs`'s `ENFORCEMENT_PATHS` array to the Sprint-02 enforcement SOURCE surfaces (`crates/but-api/src/legacy/merge_gate.rs` from AUTHZ-004 and `crates/but-api/src/legacy/config_mutate.rs` from AUTHZ-006 — the AUTHZ-005 confinement path is NOT added because AUTHZ-005 creates only TEST files, no enforcement source), re-runs the role-name / human-vs-AI / Permission-carrier greps over them, ADDS the `AUTHORITY_POSITIVE` assertion on BOTH `config_mutate.rs` (admin-write keys off `Authority::AdministrationWrite`) AND `merge_gate.rs` (the merge gate keys off `but_authz::authorize` / `Authority::Merge`, not an ad-hoc role-keyed check), asserts `ENFORCEMENT_PATHS.len() >= 5` in code, and KEEPS `assert_seeded_controls_fire()` as the negative control. A tripped pattern is a FINDING routed back to the owning task — NEVER weaken a pattern to make it pass.

## Why

Sprint 02 · PRD UC-AUTHZ-03 · capability CAP-AUTHZ-01. The honesty invariants (no role-name branch, no human-vs-AI branch, no Permission-carrier overload, the Authority axis is consulted) must hold over the NEW Sprint-02 enforcement surfaces, not just the Sprint-01a ones. This build-gate is the deterministic teeth that catch a role-keyed enforcement regression on the merge gate OR the admin-write guard — including a `merge_gate.rs` that hardcodes "allow if handle=='maint'" and never calls `but_authz::authorize`.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-authz invariant_build_gates` (build-gate; the role/label greps are 0 over the extended surfaces, the array is provably >= 5, and the seeded controls fire). Full gate set in the spec below.

## Scope

- `crates/but-authz/tests/invariant_build_gates.rs` (MODIFY) — add MERGE_GATE + CONFIG_MUTATE path consts; extend `ENFORCEMENT_PATHS` to include them (len >= 5); re-run the role/label/carrier greps over the wider array; add the `AUTHORITY_POSITIVE` assertion on BOTH config_mutate.rs AND merge_gate.rs; assert `ENFORCEMENT_PATHS.len() >= 5`; keep `assert_seeded_controls_fire()`. NEVER weaken a pattern

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: AUTHZ-008 - Honesty-invariant build-gate extension: ENFORCEMENT_PATHS to merge_gate.rs + config_mutate.rs + admin-write AND merge-gate AUTHORITY_POSITIVE assertions + len>=5 + seeded controls preserved
================================================================================

TASK_TYPE:  INFRA
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     S  (45 min)
AGENT:      implementer=rust-reviewer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-AUTHZ-03
CAPABILITIES: CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-authz invariant_build_gates
  check: cargo check -p but-authz --all-targets
  lint:  cargo clippy -p but-authz --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
`cargo test -p but-authz invariant_build_gates` is green with the ENFORCEMENT_PATHS array EXTENDED to the Sprint-02 enforcement SOURCE surfaces (merge_gate.rs, config_mutate.rs — NOT the AUTHZ-005 confinement path, which is TEST-only and would hard-fail assert_paths_exist_and_non_empty): the ROLE_BRANCH_PATTERN and HUMAN_OR_LABEL_BRANCH_PATTERN greps return 0 matches over the extended surfaces; the AUTHORITY_POSITIVE assertion finds the fully-qualified Authority axis on BOTH config_mutate.rs (Authority::AdministrationWrite, 1+) AND merge_gate.rs (but_authz::authorize / Authority::Merge, 1+); the PERMISSION_CARRIER_PATTERN grep returns 0 over the Sprint-02 surfaces; the ENFORCEMENT_PATHS array provably has length >= 5 (asserted in code); and assert_seeded_controls_fire() still fires (the seeded role / label / carrier violation fixtures each MATCH their pattern — the negative control proving the greps bite). A tripped pattern on any surface is a FINDING routed back to the owning task (AUTHZ-004/006), NEVER weakened to pass.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST EXTEND the existing const ENFORCEMENT_PATHS (currently [AUTHZ_AUTHORIZE, AUTHZ_CONFIG, COMMIT_GATE] at invariant_build_gates.rs:24) with the Sprint-02 SOURCE surfaces: crates/but-api/src/legacy/merge_gate.rs (AUTHZ-004) and crates/but-api/src/legacy/config_mutate.rs (AUTHZ-006). Add path consts (MERGE_GATE, CONFIG_MUTATE) mirroring the existing AUTHZ_AUTHORIZE/AUTHZ_CONFIG/COMMIT_GATE consts. The array must provably grow to length >= 5 (Sprint-01a trio + merge_gate + config_mutate).
- [MUST] MUST NOT add the "AUTHZ-005 confinement path" to ENFORCEMENT_PATHS — AUTHZ-005 creates ONLY TEST files (crates/but-api/tests/confinement.rs, crates/but/tests/but/command/confinement.rs), no enforcement SOURCE. assert_paths_exist_and_non_empty (invariant_build_gates.rs:66) would HARD-FAIL on a non-existent confinement enforcement source. The confinement enforcement IS the merge gate + the admin-write guard, which are already covered via merge_gate.rs + config_mutate.rs. DROP the phantom confinement-path entry entirely.
- [MUST] MUST assert ENFORCEMENT_PATHS.len() >= 5 in the test body (machine-enforced, e.g. `assert!(ENFORCEMENT_PATHS.len() >= 5, "...")` or `assert!(enforcement_paths.len() >= 5, "...")`), NOT a human-read comment — so a regression that silently drops a Sprint-02 path fails the build-gate.
- [MUST] MUST re-run the role-preset (ROLE_BRANCH_PATTERN) and human-vs-AI/label (HUMAN_OR_LABEL_BRANCH_PATTERN) greps over the EXTENDED enforcement_paths via the existing assert_grep_has_no_matches helper — exactly the existing call shape, just over the wider array.
- [MUST] MUST ADD an AUTHORITY_POSITIVE assertion on BOTH surfaces: assert_grep_has_matches with AUTHORITY_POSITIVE_PATTERN over [CONFIG_MUTATE] (the admin-write guard MUST consult Authority::AdministrationWrite) AND over [MERGE_GATE] (the merge gate MUST call but_authz::authorize / reference Authority::Merge — NOT an ad-hoc role-keyed check; this closes the C4(i) hole where a merge_gate.rs hardcoding "allow if handle=='maint'" would pass all gates). Mirror the existing commit-gate AUTHORITY_POSITIVE assertion (invariant_build_gates.rs:39). CONFIRMED: AUTHORITY_POSITIVE_PATTERN = "but_authz::authorize|Authority::contains|but_authz::Authority" (invariant_build_gates.rs:12-13) has NO bare `Authority::` branch — so AUTHZ-004 and AUTHZ-006 MUST use the FULLY-QUALIFIED but_authz::authorize(...) form (or a but_authz::Authority reference) in merge_gate.rs / config_mutate.rs for this assertion to bite. This requirement is pinned in AUTHZ-004's and AUTHZ-006's CRITICAL CONSTRAINTS; the implementer of THIS task confirms the pattern matches and, if it does not (the surface used a bare authorize), routes a FINDING back to the owning task — do NOT relax the pattern to accept a bare Authority::.
- [MUST] MUST extend the PERMISSION_CARRIER_PATTERN no-match assertion to the Sprint-02 surfaces (the Permission/RepoExclusive lock must NOT be overloaded as the authz carrier on the merge gate / admin-write guard).
- [MUST] MUST keep assert_seeded_controls_fire() — the role-branch / label-branch / permission-carrier seeded-violation fixtures are the NEGATIVE CONTROL proving the greps are live, not vacuous. Do NOT remove or weaken them.
- [NEVER] NEVER weaken a pattern, narrow a path, or add an exclusion to make a tripped grep pass — a tripped pattern is a FINDING routed back to the owning task (AUTHZ-004/006). If a Sprint-02 surface trips a pattern, STOP and report it as a finding; do not edit the surface from this task and do not relax the pattern. In particular, if AUTHORITY_POSITIVE scores 0 on merge_gate.rs or config_mutate.rs because the surface used a bare un-prefixed authorize, that is a FINDING for the owning task to add the fully-qualified form — NOT a reason to add a bare `Authority::` branch to the pattern.
- [NEVER] NEVER assert paths that do not exist yet — assert_paths_exist_and_non_empty already guards every ENFORCEMENT_PATH; this task depends on AUTHZ-004/006 having created merge_gate.rs / config_mutate.rs, so it runs AFTER them. If a path is missing, the dependency ordering is wrong (escalate), not a reason to drop the path.
- [STRICTLY] STRICTLY keep this task INFRA-only: it modifies ONLY crates/but-authz/tests/invariant_build_gates.rs. It does NOT touch any enforcement source (that is the owning task's job). test_tier unit (build-gate) with unit_test_justified inline; the seeded-control fixture is the negative control.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: ENFORCEMENT_PATHS extended to merge_gate.rs + config_mutate.rs (no phantom confinement path); role/label greps 0 over them; ENFORCEMENT_PATHS.len() >= 5 asserted in code; seeded role/label controls fire
- [ ] AC-2: AUTHORITY_POSITIVE finds the fully-qualified Authority axis on BOTH config_mutate.rs (AdministrationWrite) AND merge_gate.rs (authorize/Merge); PERMISSION_CARRIER 0 over the Sprint-02 surfaces; seeded carrier control fires
- [ ] All verification gates pass; only invariant_build_gates.rs modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: ENFORCEMENT_PATHS extended to merge_gate.rs + config_mutate.rs; role/label greps 0; len>=5 asserted; seeded controls fire [PRIMARY]
  GIVEN: the existing invariant_build_gates.rs extended with the Sprint-02 SOURCE ENFORCEMENT_PATHS (merge_gate.rs, config_mutate.rs) and NO phantom confinement-path entry
  WHEN:  cargo test -p but-authz invariant_build_gates runs
  THEN:  ROLE_BRANCH_PATTERN and HUMAN_OR_LABEL_BRANCH_PATTERN return 0 matches over the extended surfaces; the seeded role/label controls MATCH their known-bad fixtures; the test asserts ENFORCEMENT_PATHS.len() >= 5 in code (provably extended); exit 0
  TEST_TIER: unit (build-gate)   VERIFICATION_SERVICE: cargo test -p but-authz invariant_build_gates (source grep over the real tree + seeded-control fixture)   UNIT_TEST_JUSTIFIED: INFRA/build-gate — a source-grep + compile structural invariant with zero runtime product I/O; the seeded-control fixture is the non-degenerate negative control that bites without any external service; the behavioral authz guarantees are proven by AUTHZ-004/005/006's integration tests. A runtime test cannot assert the structural absence of a role-name branch across the enforcement surfaces.
  VERIFY: cargo test -p but-authz invariant_build_gates
  SCENARIO: NEGATIVE_CONTROL would fail if the array is not extended (a role-name branch on a Sprint-02 surface goes undetected — frozen at the Sprint-01a paths); the AUTHZ-005 confinement TEST path is wrongly added so assert_paths_exist_and_non_empty hard-fails on a non-existent enforcement source; the len>=5 assertion is a comment not code so a dropped path passes; the role/label greps are weakened so a seeded violation no longer matches; assert_seeded_controls_fire is a no-op (vacuously green).

AC-2: AUTHORITY_POSITIVE on BOTH config_mutate.rs AND merge_gate.rs; PERMISSION_CARRIER 0 over the Sprint-02 surfaces; seeded carrier control fires
  GIVEN: the admin-write surface config_mutate.rs and the merge-gate surface merge_gate.rs
  WHEN:  cargo test -p but-authz invariant_build_gates runs
  THEN:  AUTHORITY_POSITIVE_PATTERN finds the fully-qualified Authority axis on config_mutate.rs (Authority::AdministrationWrite, 1+) AND on merge_gate.rs (but_authz::authorize / Authority::Merge, 1+); PERMISSION_CARRIER_PATTERN returns 0 over the Sprint-02 surfaces; the seeded Permission/write_permission carrier control MATCHES its known-bad fixture; exit 0
  TEST_TIER: unit (build-gate)   VERIFICATION_SERVICE: cargo test -p but-authz invariant_build_gates (source grep + seeded carrier-control fixture)   UNIT_TEST_JUSTIFIED: build-gate structural invariant with zero runtime I/O; the AUTHORITY_POSITIVE assertion on BOTH surfaces proves the merge gate AND the admin-write guard key off the functional Authority axis (T-AUTHZ-022) — catching a role-keyed merge_gate.rs that never calls authorize — and the seeded carrier-control fixture is the non-degenerate negative control; the behavioral guarantees are proven by AUTHZ-004/006's integration tests.
  VERIFY: cargo test -p but-authz invariant_build_gates
  SCENARIO: NEGATIVE_CONTROL would fail if AUTHORITY_POSITIVE is NOT asserted on merge_gate.rs (a role-keyed merge gate that hardcodes "allow if handle=='maint'" and never calls but_authz::authorize passes undetected — the C4(i) hole); AUTHORITY_POSITIVE is not asserted on config_mutate.rs (a role-keyed admin-write guard passes); a surface used a bare un-prefixed authorize so the pattern scores 0 and the implementer weakens the pattern instead of routing a finding; the PERMISSION_CARRIER grep is weakened; the seeded carrier control is a no-op.

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, structural): ENFORCEMENT_PATHS extended to merge_gate.rs + config_mutate.rs (no phantom confinement path); role-preset + human-vs-AI greps 0 over them; ENFORCEMENT_PATHS.len() >= 5 asserted in code; seeded role/label controls fire (T-LOOP-005, T-LOOP-011, T-AUTHZ-016)
    VERIFY: cargo test -p but-authz invariant_build_gates
- TC-2 (-> AC-2, structural): AUTHORITY_POSITIVE finds the fully-qualified Authority axis on BOTH config_mutate.rs (AdministrationWrite) AND merge_gate.rs (authorize/Merge); PERMISSION_CARRIER 0 over the Sprint-02 surfaces; seeded carrier control fires (T-AUTHZ-022)
    VERIFY: cargo test -p but-authz invariant_build_gates

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides: the honesty-invariant build-gate coverage over the Sprint-02 enforcement SOURCE surfaces (merge_gate.rs, config_mutate.rs): no role-name branch, no human-vs-AI branch, no Permission-carrier overload, the Authority axis is consulted on BOTH the merge gate and the admin-write guard; ENFORCEMENT_PATHS.len() >= 5 enforced in code; the seeded negative controls preserved
consumes: the existing invariant_build_gates.rs machinery (the four PATTERN consts, assert_grep_has_no_matches / assert_grep_has_matches / assert_seeded_controls_fire, assert_paths_exist_and_non_empty); the Sprint-02 SOURCE surfaces created by AUTHZ-004/006
boundary_contracts:
  - CAP-AUTHZ-01: enforcement is functional-only across ALL surfaces — the build-gate proves no Sprint-02 surface re-introduces a role-name / human-vs-AI branch or overloads the Permission lock as the authz carrier, and that BOTH the merge gate and the admin-write guard key off the Authority axis (the merge gate is not an ad-hoc role-keyed check).

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-authz/tests/invariant_build_gates.rs (MODIFY) — add MERGE_GATE + CONFIG_MUTATE path consts; extend ENFORCEMENT_PATHS (len >= 5, asserted in code); re-run role/label/carrier greps over the Sprint-02 surfaces; add the AUTHORITY_POSITIVE assertion on BOTH config_mutate.rs AND merge_gate.rs; keep assert_seeded_controls_fire
writeProhibited:
  - crates/but-api/src/legacy/merge_gate.rs — OWNED by AUTHZ-004; a tripped pattern (incl. AUTHORITY_POSITIVE scoring 0 on a bare authorize) here is a FINDING routed back, NOT an edit from this task
  - crates/but-api/src/legacy/config_mutate.rs — OWNED by AUTHZ-006; a tripped pattern here is a FINDING routed back
  - crates/but-api/tests/confinement.rs and the AUTHZ-005 test files — TEST-only; NOT added to ENFORCEMENT_PATHS (no enforcement source)
  - crates/but-authz/src/** — the primitive is AUTHZ-001/002/003; this task only touches the build-gate test
  - any enforcement SOURCE — this task NEVER edits a surface to make a grep pass
  - Any file not listed in write_allowed

--------------------------------------------------------------------------------
OUT OF SCOPE
--------------------------------------------------------------------------------
  - Adding the AUTHZ-005 confinement path to ENFORCEMENT_PATHS — AUTHZ-005 creates only TEST files; there is no enforcement source to add, and asserting one would hard-fail assert_paths_exist_and_non_empty. The confinement enforcement is the merge gate + the admin-write guard, already covered.
  - Editing any enforcement surface to make a grep pass — a tripped pattern (incl. a bare-authorize AUTHORITY_POSITIVE miss) is a FINDING routed back to the owning task (AUTHZ-004/006), never fixed by weakening the pattern or editing the surface from this task.
  - Adding new PATTERN consts beyond the AUTHORITY_POSITIVE assertions on config_mutate.rs and merge_gate.rs — the four existing patterns cover role/label/authority/carrier; this task extends their COVERAGE (paths) plus the explicit AUTHORITY_POSITIVE assertions on the two Sprint-02 surfaces.
  - The behavioral authz guarantees — proven by AUTHZ-004/005/006's integration tests, not this build-gate.

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-authz/tests/invariant_build_gates.rs (FULL)
   Focus: THE MACHINERY THIS TASK EXTENDS — the four PATTERN consts (ROLE_BRANCH_PATTERN:10, HUMAN_OR_LABEL_BRANCH_PATTERN:11, AUTHORITY_POSITIVE_PATTERN:12-13 — CONFIRMED has NO bare Authority:: branch, PERMISSION_CARRIER_PATTERN:14); the path consts (AUTHZ_AUTHORIZE:17, AUTHZ_CONFIG:18, COMMIT_GATE:19); the ENFORCEMENT_PATHS array (24); assert_paths_exist_and_non_empty (66); assert_grep_has_no_matches (81); assert_grep_has_matches (105); assert_seeded_controls_fire (131) with the three seeded violation fixtures. ADD MERGE_GATE + CONFIG_MUTATE path consts, extend ENFORCEMENT_PATHS, assert len >= 5 in code, add the AUTHORITY_POSITIVE assertion on BOTH config_mutate.rs AND merge_gate.rs mirroring the COMMIT_GATE one (39-44).
2. .spec/prds/governance/tasks/sprint-02-authz-fail-closed-identity-confinement/AUTHZ-004-merge-gate-fail-closed.md (Scope + VERIFICATION GATES)
   Focus: confirms crates/but-api/src/legacy/merge_gate.rs is the AUTHZ-004 merge-gate surface to add to ENFORCEMENT_PATHS, that it uses the fully-qualified but_authz::authorize / Authority::Merge form (so AUTHORITY_POSITIVE bites), and that it already carries the no-role-name / no-Permission-carrier verification gates this build-gate enforces cross-cuttingly.
3. .spec/prds/governance/tasks/sprint-02-authz-fail-closed-identity-confinement/AUTHZ-006-administration-write-guard.md (Scope + AC-2)
   Focus: confirms crates/but-api/src/legacy/config_mutate.rs is the admin-write surface, that AC-2 already greps it for the fully-qualified but_authz::authorize / Authority::AdministrationWrite — this build-gate hoists that into the central honesty suite (AUTHORITY_POSITIVE on config_mutate.rs).
4. .spec/prds/governance/tasks/sprint-02-authz-fail-closed-identity-confinement/AUTHZ-005-identity-confinement.md (Scope)
   Focus: CONFIRM AUTHZ-005 creates ONLY TEST files (confinement.rs) — there is NO enforcement source to add to ENFORCEMENT_PATHS. The confinement enforcement is the merge gate + the admin-write guard (already covered). Do NOT add a confinement path.
5. crates/but-authz/src/authority.rs (26-36, 81-82, 103-107)
   Focus: confirm AUTHORITY_POSITIVE_PATTERN (but_authz::authorize|Authority::contains|but_authz::Authority) catches Authority::AdministrationWrite AND Authority::Merge via the Authority:: branch when config_mutate.rs / merge_gate.rs reference them in the fully-qualified form.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- Build-gate passes with the extended coverage: `cargo test -p but-authz invariant_build_gates`  -> Exit 0; AC-1 + AC-2 green
- ENFORCEMENT_PATHS provably extended and len-asserted: `grep -nE 'ENFORCEMENT_PATHS|merge_gate.rs|config_mutate.rs|len\(\) >= 5|len\(\) > 4' crates/but-authz/tests/invariant_build_gates.rs`  -> the Sprint-02 paths present in the array AND a code-level len >= 5 assertion
- The AUTHORITY_POSITIVE assertions present on BOTH surfaces: `grep -nE 'AUTHORITY_POSITIVE_PATTERN' crates/but-authz/tests/invariant_build_gates.rs`  -> appears for config_mutate AND merge_gate (reviewer confirms both surfaces are asserted for the Authority axis)
- No confinement TEST path added: `! grep -nE 'confinement' crates/but-authz/tests/invariant_build_gates.rs`  -> No matches (AUTHZ-005 is test-only; not an enforcement source)
- The seeded controls are preserved: `grep -nE 'assert_seeded_controls_fire' crates/but-authz/tests/invariant_build_gates.rs`  -> still called from invariant_build_gates()
- No enforcement source edited: `git diff --name-only`  -> ONLY crates/but-authz/tests/invariant_build_gates.rs changed
- Crate compiles incl. tests: `cargo check -p but-authz --all-targets`  -> Exit 0
- Clippy clean: `cargo clippy -p but-authz --all-targets`  -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: Build-gate-coverage-extension — add path consts for the Sprint-02 SOURCE surfaces (MERGE_GATE = merge_gate.rs, CONFIG_MUTATE = config_mutate.rs), extend the ENFORCEMENT_PATHS array, assert ENFORCEMENT_PATHS.len() >= 5 in code, re-run the existing assert_grep_has_no_matches(ROLE/LABEL/CARRIER) over the wider array, ADD assert_grep_has_matches(AUTHORITY_POSITIVE, [CONFIG_MUTATE]) AND assert_grep_has_matches(AUTHORITY_POSITIVE, [MERGE_GATE]) mirroring the COMMIT_GATE AUTHORITY_POSITIVE assertion, and KEEP assert_seeded_controls_fire() as the negative control. A tripped pattern (incl. a bare-authorize AUTHORITY_POSITIVE miss) is a FINDING routed back to the owning task — never weakened. The AUTHZ-005 confinement path is NOT added (test-only, no enforcement source).
pattern_source: crates/but-authz/tests/invariant_build_gates.rs:21-55 (the existing invariant_build_gates() body + the three seeded-control fixtures) + :39-44 (the COMMIT_GATE AUTHORITY_POSITIVE assertion to mirror)
anti_pattern: Adding the AUTHZ-005 confinement TEST path to ENFORCEMENT_PATHS (hard-fails assert_paths_exist_and_non_empty); omitting the merge_gate.rs AUTHORITY_POSITIVE assertion so a role-keyed merge gate passes undetected (C4(i)); making the len>=5 check a comment instead of a code assertion; weakening the AUTHORITY_POSITIVE pattern to accept a bare Authority:: instead of routing a finding; weakening a pattern / narrowing a path / adding an exclusion to make a tripped grep pass; removing or no-op-ing assert_seeded_controls_fire (vacuously green); asserting a path that does not exist yet (wrong dependency ordering); or editing an enforcement surface from this INFRA task.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-reviewer — This is the honesty build-gate: a reviewer-owned INFRA task that extends the deterministic enforcement-surface coverage (merge_gate.rs + config_mutate.rs, NOT the test-only confinement path), adds the AUTHORITY_POSITIVE assertion on BOTH the merge gate and the admin-write guard, asserts len >= 5 in code, and routes any tripped pattern (incl. a bare-authorize AUTHORITY_POSITIVE miss) back as a FINDING rather than weakening it. rust-reviewer owns the adversarial grep discipline, the seeded-control preservation, and the never-weaken-a-pattern rule.
reviewer: rust-reviewer
coding_standards: crates/AGENTS.md, crates/but-authz/tests/invariant_build_gates.rs

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: AUTHZ-004, AUTHZ-005, AUTHZ-006, AUTHZ-007   (the Sprint-02 enforcement SOURCE surfaces merge_gate.rs/config_mutate.rs must exist + the Sprint-01a build-gate machinery from AUTHZ-007; AUTHZ-005 is a sequencing dep but contributes no enforcement path)
Blocks:     (sprint gate)
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "AUTHZ-008",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "notes": [
    "C4(a): AUTHORITY_POSITIVE is now asserted over merge_gate.rs too — proving the merge gate calls but_authz::authorize / references Authority::Merge, not an ad-hoc role-keyed check (closes the 'allow if handle==maint' hole).",
    "C4(b): the phantom 'AUTHZ-005 confinement path' ENFORCEMENT_PATHS entry is DROPPED — AUTHZ-005 creates only TEST files; assert_paths_exist_and_non_empty would hard-fail. ENFORCEMENT_PATHS grows to >=5 via the Sprint-01a trio (authorize.rs, config.rs, commit/gate.rs) + merge_gate.rs + config_mutate.rs.",
    "C4(c): `assert ENFORCEMENT_PATHS.len() >= 5` is enforced IN CODE (not a comment).",
    "C4(d): merge_gate.rs and config_mutate.rs MUST use the fully-qualified but_authz::authorize(...) form so AUTHORITY_POSITIVE_PATTERN (no bare Authority:: branch, confirmed at invariant_build_gates.rs:12-13) bites; a bare-authorize miss is a FINDING routed back, not a pattern relaxation.",
    "CONFIRMED grounding: AUTHORITY_POSITIVE_PATTERN = but_authz::authorize|Authority::contains|but_authz::Authority (invariant_build_gates.rs:12-13); ENFORCEMENT_PATHS = [AUTHZ_AUTHORIZE, AUTHZ_CONFIG, COMMIT_GATE] (line 24); assert_paths_exist_and_non_empty (line 66) hard-fails on missing files."
  ],
  "fixtures": {
    "sprint02_enforcement_surfaces": {
      "description": "The Sprint-02 enforcement SOURCE surfaces that AUTHZ-008 adds to the EXISTING invariant_build_gates.rs ENFORCEMENT_PATHS array: crates/but-api/src/legacy/merge_gate.rs (AUTHZ-004's merge/forge gate) and crates/but-api/src/legacy/config_mutate.rs (AUTHZ-006's admin-write guard). These join the existing AUTHZ_AUTHORIZE / AUTHZ_CONFIG / COMMIT_GATE paths (total >= 5). The AUTHZ-005 confinement path is NOT added — AUTHZ-005 creates only TEST files (confinement.rs), no enforcement source, and assert_paths_exist_and_non_empty would hard-fail on a phantom source. The build-gate machinery is the four PATTERN consts (ROLE_BRANCH_PATTERN, HUMAN_OR_LABEL_BRANCH_PATTERN, AUTHORITY_POSITIVE_PATTERN, PERMISSION_CARRIER_PATTERN), the assert_grep_has_no_matches / assert_grep_has_matches helpers, and the assert_seeded_controls_fire negative-control fixture. The test runs against the REAL committed source tree (grep over real files); the seeded-control fixture is the negative control (a tripped pattern in a known-bad fixture proves the greps bite).",
      "seed_method": "cli",
      "records": [
        "the real source files exist after AUTHZ-004/006 land: crates/but-api/src/legacy/merge_gate.rs, crates/but-api/src/legacy/config_mutate.rs (asserted non-empty by assert_paths_exist_and_non_empty); both use the fully-qualified but_authz::authorize form so AUTHORITY_POSITIVE bites",
        "assert_seeded_controls_fire writes a role-branch-violation.rs, a label-branch-violation.rs, and a permission-carrier-violation.rs into a TempDir and asserts each forbidden pattern MATCHES the seeded violation (the negative control proving the greps are live, not vacuous)"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN the EXISTING invariant_build_gates.rs extended with the Sprint-02 SOURCE ENFORCEMENT_PATHS (merge_gate.rs, config_mutate.rs) and NO phantom confinement-path entry WHEN cargo test -p but-authz invariant_build_gates runs THEN the role-preset and human-vs-AI/label greps return 0 matches over the Sprint-02 surfaces AND the seeded role-branch/label control fires (matches the known-bad fixture) AND the test asserts ENFORCEMENT_PATHS.len() >= 5 in code",
      "verify": "cargo test -p but-authz invariant_build_gates",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "unit",
        "unit_test_justified": "INFRA/build-gate: this task IS the honesty invariant suite — a source-grep + compile structural invariant with zero runtime product I/O. Per TESTING-HIERARCHY a build-gate is the justified unit row; the seeded-control fixture (assert_seeded_controls_fire) is the non-degenerate negative control that bites without any external service, and the behavioral authz guarantees are proven by AUTHZ-004/005/006's integration tests. A runtime integration test cannot assert the structural absence of a role-name branch across the enforcement surfaces — only a grep can.",
        "verification_service": "cargo test -p but-authz invariant_build_gates (source grep over the real tree + seeded-control fixture)",
        "negative_control": {
          "would_fail_if": [
            "the ENFORCEMENT_PATHS array is NOT extended to merge_gate.rs / config_mutate.rs, so a role-name branch could be added to a Sprint-02 surface and go undetected (the array is a static stub frozen at the Sprint-01a paths)",
            "the AUTHZ-005 confinement TEST path is wrongly added so assert_paths_exist_and_non_empty hard-fails on a non-existent enforcement source",
            "the ENFORCEMENT_PATHS.len() >= 5 check is a comment instead of a code assertion so a silently dropped Sprint-02 path passes",
            "the role-preset / human-vs-AI greps are weakened or removed so a seeded violation no longer matches (the seeded control no longer fires)",
            "assert_seeded_controls_fire is deleted / made a no-op so the greps are vacuously green"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "sprint02_enforcement_surfaces",
            "action": {
              "actor": "ci",
              "steps": [
                "run `cargo test -p but-authz invariant_build_gates` after extending ENFORCEMENT_PATHS with merge_gate.rs + config_mutate.rs (no confinement path) and adding the code-level len >= 5 assertion (cite T-LOOP-005, T-LOOP-011, T-AUTHZ-016)"
              ]
            },
            "end_state": {
              "must_observe": [
                "the ROLE_BRANCH_PATTERN grep returns `0` matches over the extended ENFORCEMENT_PATHS (merge_gate.rs, config_mutate.rs included)",
                "the HUMAN_OR_LABEL_BRANCH_PATTERN grep returns `0` matches over the same surfaces",
                "the seeded role-branch control MATCHES (`1+`) the known-bad `role-branch-violation.rs` fixture and the seeded label control MATCHES the `label-branch-violation.rs` fixture (the greps provably bite)",
                "the test asserts `ENFORCEMENT_PATHS.len() >= 5` in CODE (the 3 Sprint-01a paths plus merge_gate + config_mutate) — provably extended",
                "no `confinement` path appears in ENFORCEMENT_PATHS (AUTHZ-005 is test-only)",
                "the test process exits `0`"
              ],
              "must_not_observe": [
                "`1+` role-preset or human-vs-AI matches over any Sprint-02 enforcement surface",
                "`0` matches for the seeded role/label control (a vacuous grep that never bites)",
                "the ENFORCEMENT_PATHS array unchanged at the Sprint-01a `3`-path size (not extended)",
                "an AUTHZ-005 confinement path added (hard-fails assert_paths_exist_and_non_empty)",
                "the len>=5 check present only as a comment, not a code assertion"
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
      "description": "GIVEN the admin-write surface (config_mutate.rs) AND the merge-gate surface (merge_gate.rs) WHEN cargo test -p but-authz invariant_build_gates runs THEN the AUTHORITY_POSITIVE grep finds the fully-qualified Authority axis on config_mutate.rs (Authority::AdministrationWrite, 1+) AND on merge_gate.rs (but_authz::authorize / Authority::Merge, 1+) AND the PERMISSION_CARRIER grep returns 0 over the Sprint-02 surfaces AND the seeded Permission/write_permission carrier control fires",
      "verify": "cargo test -p but-authz invariant_build_gates",
      "maps_to_ac": null,
      "scenario": {
        "tier": "holdout",
        "test_tier": "unit",
        "unit_test_justified": "Build-gate structural invariant with zero runtime I/O: the AUTHORITY_POSITIVE assertion on BOTH config_mutate.rs AND merge_gate.rs proves the admin-write guard AND the merge gate key off the functional Authority axis (T-AUTHZ-022) — catching a role-keyed merge_gate.rs that hardcodes a handle check and never calls but_authz::authorize — and the PERMISSION_CARRIER assertion proves the GitButler repo-lock Permission is not overloaded as the authz carrier. Both are source-structural properties a runtime test cannot fully assert. The seeded carrier-control fixture is the non-degenerate negative control; the behavioral guarantees are proven by AUTHZ-004/006's integration tests.",
        "verification_service": "cargo test -p but-authz invariant_build_gates (source grep over the real tree + seeded carrier-control fixture)",
        "negative_control": {
          "would_fail_if": [
            "the AUTHORITY_POSITIVE grep is NOT asserted on merge_gate.rs so a merge gate that keys off a role name (e.g. hardcodes 'allow if handle==maint') and never calls but_authz::authorize passes undetected (the C4(i) hole)",
            "the AUTHORITY_POSITIVE grep is NOT asserted on config_mutate.rs so a role-keyed admin-write guard passes undetected",
            "a Sprint-02 surface used a bare un-prefixed authorize so AUTHORITY_POSITIVE scores 0, and the implementer WEAKENS the pattern (adds a bare Authority:: branch) instead of routing a finding to the owning task",
            "the PERMISSION_CARRIER grep is weakened so overloading the GitButler Permission / RepoExclusive lock as the authz carrier on a Sprint-02 surface goes undetected",
            "the seeded carrier control (permission-carrier-violation.rs) is removed / made a no-op so the carrier grep is vacuously green"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "sprint02_enforcement_surfaces",
            "action": {
              "actor": "ci",
              "steps": [
                "run `cargo test -p but-authz invariant_build_gates` with the AUTHORITY_POSITIVE assertion added for BOTH config_mutate.rs AND merge_gate.rs and the PERMISSION_CARRIER assertion extended to the Sprint-02 surfaces (cite T-AUTHZ-022, C4)"
              ]
            },
            "end_state": {
              "must_observe": [
                "the AUTHORITY_POSITIVE_PATTERN grep returns `1+` matches on `crates/but-api/src/legacy/config_mutate.rs` (the admin-write guard keys off the fully-qualified Authority::AdministrationWrite)",
                "the AUTHORITY_POSITIVE_PATTERN grep returns `1+` matches on `crates/but-api/src/legacy/merge_gate.rs` (the merge gate calls but_authz::authorize / references Authority::Merge — not an ad-hoc role-keyed check)",
                "the PERMISSION_CARRIER_PATTERN grep returns `0` matches over the Sprint-02 enforcement surfaces",
                "the seeded Permission/write_permission carrier control MATCHES (`1+`) the known-bad `permission-carrier-violation.rs` fixture",
                "the test process exits `0`"
              ],
              "must_not_observe": [
                "`0` AUTHORITY_POSITIVE matches on merge_gate.rs (a role-keyed merge gate that never consults Authority — the C4(i) hole)",
                "`0` AUTHORITY_POSITIVE matches on config_mutate.rs (a disconnected admin-write guard)",
                "the AUTHORITY_POSITIVE pattern weakened with a bare Authority:: branch to accept a bare-authorize surface instead of routing a finding",
                "`1+` PERMISSION_CARRIER matches over a Sprint-02 surface (the repo lock overloaded as authz)",
                "`0` matches for the seeded carrier control (a vacuous carrier grep)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "ENFORCEMENT_PATHS extended to merge_gate.rs + config_mutate.rs (no phantom confinement path); role-preset + human-vs-AI greps 0 over them; ENFORCEMENT_PATHS.len() >= 5 asserted in code; seeded role/label controls fire (T-LOOP-005, T-LOOP-011, T-AUTHZ-016)",
      "verify": "cargo test -p but-authz invariant_build_gates",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "AUTHORITY_POSITIVE finds the fully-qualified Authority axis on BOTH config_mutate.rs (AdministrationWrite) AND merge_gate.rs (authorize/Merge); PERMISSION_CARRIER 0 over the Sprint-02 surfaces; seeded carrier control fires (T-AUTHZ-022, C4)",
      "verify": "cargo test -p but-authz invariant_build_gates",
      "maps_to_ac": "AC-2"
    }
  ]
}
-->
</details>
