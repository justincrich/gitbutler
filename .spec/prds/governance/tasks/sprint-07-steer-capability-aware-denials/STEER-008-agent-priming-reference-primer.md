# STEER-008: Ship the non-enforced agent-priming reference primer (denials=redirects, affordances=options-not-orders, no-bypass, `class`/`do_not` contract) + prove no `but-authz`/`but-api` path depends on it

## What this does

Ship a short non-enforced reference primer teaching a harness that `but` denials are redirects and `authorized_actions` are options (not orders), bypass is never faster, and to honor the class/do_not contract — and prove via a build-gate that no engine path depends on it (Stance 6).

## Why

Sprint 07 (STEER — Capability-Aware Denials) · PRD UC-STEER-05 · Capability CAP-STEER-01. A primer doc exists carrying the four literal statements; a build-gate test passes that greps the engine source and finds zero dependence on the primer (and trips on a deliberately injected dependence); the reviewer confirms the primer is marked non-enforced.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-authz --test invariant_build_gates steer_primer_contains_required_statements` (Primer doc exists with the four required statements [PRIMARY]). Full gate set in the spec below.

## Scope

- crates/but/governance-denial-primer.md (NEW) — the non-enforced reference primer doc
- crates/but-authz/tests/invariant_build_gates.rs (MODIFY — ADDITIVE ONLY) — add the four primer content-check + non-dependence-grep assertions + a seeded violating-fixture teeth control; NEVER weaken/remove an existing pattern

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: STEER-008 - Ship the non-enforced agent-priming reference primer (denials=redirects, affordances=options-not-orders, no-bypass, `class`/`do_not` contract) + prove no `but-authz`/`but-api` path depends on it
================================================================================

TASK_TYPE:  INFRA
STATUS:     Completed
PRIORITY:   P2
EFFORT:     S  (90 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-STEER-05
CAPABILITIES: CAP-STEER-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-authz --test invariant_build_gates
  lint:  cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
A primer doc exists carrying the four literal statements; a build-gate test passes that greps the engine source and finds zero dependence on the primer (and trips on a deliberately injected dependence); the reviewer confirms the primer is marked non-enforced.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST ship the primer as a dedicated reference doc file in the repo (NEW `crates/but/governance-denial-primer.md`) explicitly marked non-enforced reference material — it is documentation, NOT code wired into any gate.
- [MUST] MUST state, in the primer, all four claims literally: (1) `but` denials are redirects rather than terminal failures; (2) `authorized_actions` are authorized OPTIONS not orders — choose the entry that serves your actual task (goal integrity); (3) bypass (raw git / --no-verify) is NEVER the route to a landed change; (4) the `class`/`do_not` contract — stop on `operator_required`, never bypass on `actor_correctable`.
- [MUST] MUST add a build-gate test (extend `crates/but-authz/tests/invariant_build_gates.rs` ADDITIVELY, or a new test in that crate) that GREPS the engine source (the ENFORCEMENT_PATHS + the broader but-authz/but-api src trees) and asserts ZERO references to the primer filename / primer content — proving no engine correctness path depends on it (Stance 6).
- [NEVER] NEVER import, `include_str!`, read, or branch on the primer from any but-authz/but-api code path — it must be inert reference material; the build-gate proves this.
- [NEVER] NEVER replace the shipped honesty-grep patterns in invariant_build_gates.rs (no-role-preset / no-human-vs-AI / positive-authorize / no-Permission) — add the non-dependence grep BESIDE them.
- [NEVER] NEVER enumerate bypass mechanics as instructions; frame no-bypass positively (the governed path is the only route to a landed change).
- [STRICTLY] STRICTLY scope this task to the primer doc + the non-dependence build-gate — no runtime behavior change; classify as INFRA (no GWT runtime behavior) with each AC carrying a concrete repo-content / grep observable.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: Primer doc exists with the four required statements [PRIMARY]
- [ ] AC-2: Primer is non-enforced — no engine path depends on it (Stance 6)
- [ ] AC-3: Primer encodes goal-integrity + the class/do_not contract
- [ ] AC-4: Primer is explicitly marked non-enforced reference material
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: Primer doc exists with the four required statements [PRIMARY] [PRIMARY]
  GIVEN: the repo after this task lands; a content-check test reading the primer file
  WHEN:  the primer doc `crates/but/governance-denial-primer.md` is read
  THEN:  it literally contains: "denials are redirects" (not terminal failures), "options, not orders" (affordances), "bypass is never" the route to a landed change, and the class/do_not contract ("stop" on "operator_required")
  TEST_TIER: build-gate   VERIFICATION_SERVICE: but-cli
  VERIFY: cargo test -p but-authz --test invariant_build_gates steer_primer_contains_required_statements
  SCENARIO: would fail if empty; static; stub | must observe: the substring `"redirect"` (denials are redirects); the substring `"options, not orders"`; the substring `"bypass"` framed as never-the-route; the substrings `"operator_required"` and `"stop"` | must NOT observe: a missing primer file (`none` on disk); an `empty` primer with `no` required statements

AC-2: Primer is non-enforced — no engine path depends on it (Stance 6)
  GIVEN: the committed `engine_source_tree`; a build-gate that greps but-authz/but-api source for any reference to the primer filename or primer-specific content
  WHEN:  the non-dependence grep runs over the engine source
  THEN:  it finds ZERO matches (no `include_str!`, no path reference, no branch keyed on the primer) — proving the engine is correct independent of the primer
  TEST_TIER: build-gate   VERIFICATION_SERVICE: but-cli
  VERIFY: cargo test -p but-authz --test invariant_build_gates steer_primer_engine_independent
  SCENARIO: would fail if a but-authz/but-api path imported or include_str!'d or branched on the primer (the grep would then match and the test would fail); static | must observe: the engine-source grep returns `0` matches (engine independent); the injected temp violating fixture yields `>= 1` grep match (the gate has teeth) | must NOT observe: any match of the primer reference in real engine source (must be `0`); the grep failing to detect the injected violating fixture (`0` matches on a real violation)

AC-3: Primer encodes goal-integrity + the class/do_not contract
  GIVEN: the primer doc
  WHEN:  the content-check reads the goal-integrity + contract sections
  THEN:  the primer states an agent should choose the `authorized_actions` entry that serves its actual task (affordances ≠ orders — goal integrity) AND documents the class/do_not contract (stop on `operator_required`; never bypass on `actor_correctable`)
  TEST_TIER: build-gate   VERIFICATION_SERVICE: but-cli
  VERIFY: cargo test -p but-authz --test invariant_build_gates steer_primer_goal_integrity_and_contract
  SCENARIO: would fail if empty; static | must observe: the phrase `"serves your"` task / `"your actual task"` (goal integrity); documentation of `"class"` with both `"actor_correctable"` and `"operator_required"`; the `"do_not"` contract framing | must NOT observe: a primer lacking the goal-integrity statement (`none` present); a primer that omits the class/do_not contract (`no` contract section)

AC-4: Primer is explicitly marked non-enforced reference material
  GIVEN: the primer doc
  WHEN:  the content-check reads the primer header/marker
  THEN:  the primer carries an explicit non-enforced/reference marker (e.g. a header line stating it is non-enforced reference material the harness MAY adopt; the engine does not depend on it)
  TEST_TIER: build-gate   VERIFICATION_SERVICE: but-cli
  VERIFY: cargo test -p but-authz --test invariant_build_gates steer_primer_marked_non_enforced
  SCENARIO: would fail if empty; static | must observe: a marker substring such as `"non-enforced"` and `"reference"` | must NOT observe: `no` marker, leaving the primer ambiguous about enforcement status (and no such entry/value present — the empty/start state must be excluded)

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, happy_path): The primer file contains the four required statements (redirect / options-not-orders / no-bypass / class+do_not stop on operator_required)
    VERIFY: cargo test -p but-authz --test invariant_build_gates steer_primer_contains_required_statements
- TC-2 (-> AC-2, structural): Grepping but-authz/but-api source finds zero references to the primer, and an injected dependence is detected (teeth)
    VERIFY: cargo test -p but-authz --test invariant_build_gates steer_primer_engine_independent
- TC-3 (-> AC-3, happy_path): The primer states goal integrity (choose the entry serving your task) and documents the class/do_not contract
    VERIFY: cargo test -p but-authz --test invariant_build_gates steer_primer_goal_integrity_and_contract
- TC-4 (-> AC-4, happy_path): The primer is marked non-enforced reference material
    VERIFY: cargo test -p but-authz --test invariant_build_gates steer_primer_marked_non_enforced

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-STEER-01
provides: a non-enforced reference agent-priming primer doc in the repo (denials=redirects, affordances=options-not-orders, no-bypass, class/do_not contract); a build-gate test proving no but-authz/but-api code path depends on the primer for correctness (Stance 6)
consumes: the L1 denial contract vocabulary (class/do_not, actor_correctable/operator_required, authorized_actions) that STEER-001..004 establish — the primer documents it but does not import it
boundary_contracts:
  - CAP-STEER-01: L2 agent priming is reference material the harness MAY adopt; the engine (but-authz/but-api) is proven INDEPENDENT of it (Stance 6 — the harness owns the agent reasoning). The primer states the class/do_not contract and goal-integrity (affordances are options, not orders).

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but/governance-denial-primer.md (NEW) — the non-enforced reference primer doc
  - crates/but-authz/tests/invariant_build_gates.rs (MODIFY — ADDITIVE ONLY) — add the four primer content-check + non-dependence-grep assertions + a seeded violating-fixture teeth control; NEVER weaken/remove an existing pattern
writeProhibited:
  - any crates/but-authz/src or crates/but-api/src file - the engine must NOT reference the primer; do not wire it in
  - the gate deny/allow decision - NEVER weaken
  - the shipped honesty-grep patterns (no-role-preset/no-human-vs-AI/positive-authorize/no-Permission) - NEVER replace; add beside
  - .spec/prds/governance/tasks/sprint-0[1-6]* - frozen
  - Any file not explicitly listed above

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
  - crates/but-authz/tests/invariant_build_gates.rs (lines 1-100, 167-237): PRIMARY PATTERN — the grep harness (assert_grep_has_no_matches / assert_grep_has_matches) and the assert_seeded_controls_fire teeth pattern; add the primer-non-dependence grep + a seeded violating-fixture control beside the shipped patterns (ADDITIVE only).
  - .spec/prds/governance/enrichments/v1.4.0-capability-aware-denials/02-uc-steer.md (lines 79-88): UC-STEER-05 AC-1..4 — the exact claims the primer must encode (redirects, options-not-orders, no-bypass, class/do_not, goal integrity, Stance 6).
  - .spec/prds/governance/enrichments/v1.4.0-capability-aware-denials/03-technical-requirements-delta.md (lines 124-132): §7 Layers — L2 is non-enforced agent priming, harness-owned (Stance 6); the engine must not depend on it.
  - crates/but/AGENTS.md (lines all): house style for repo-level reference docs near the `but` CLI; the primer is sibling reference material, not an AGENTS.md instruction.
  - crates/but-api/src/legacy/forge.rs (lines 1-60): representative but-api enforcement source the non-dependence grep scans — confirm no primer reference exists in the engine trees.

--------------------------------------------------------------------------------
CODE PATTERN
--------------------------------------------------------------------------------
pattern: reference-doc + additive build-gate: ship a marked non-enforced .md and assert (a) its required content and (b) zero engine dependence via the existing grep harness, with a seeded violating fixture proving the grep bites.
pattern_source: crates/but-authz/tests/invariant_build_gates.rs:48-95 (grep assertions) + :176-237 (assert_seeded_controls_fire teeth)
anti_pattern: Wiring the primer into engine code (include_str!/path branch); claiming completeness without the teeth control; weakening or relocating a shipped honesty-grep pattern.
references: 02-uc-steer.md UC-STEER-05; 03-technical-requirements-delta.md §7 L2; 04-e2e-testing-criteria.md T-STEER-021/022/023; 05-delta-replan.md (L2 non-enforced)
interaction_notes:
  - INFRA/docs-flavored — the proof is repo-content checks + a non-dependence grep with a teeth control, not GWT runtime behavior
  - the non-dependence grep reuses the invariant_build_gates.rs grep harness

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
depends_on: STEER-004
blocks: (none)

CODING STANDARDS: crates/AGENTS.md, crates/but/AGENTS.md, RULES.md
```
</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN the repo, WHEN the primer is read, THEN it contains the four required statements",
      "verify": "cargo test -p but-authz --test invariant_build_gates steer_primer_contains_required_statements"
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "GIVEN the engine source, WHEN the non-dependence grep runs, THEN zero matches (engine independent) and an injected dependence trips the grep",
      "verify": "cargo test -p but-authz --test invariant_build_gates steer_primer_engine_independent"
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "GIVEN the primer, WHEN read, THEN it states goal integrity and the class/do_not contract",
      "verify": "cargo test -p but-authz --test invariant_build_gates steer_primer_goal_integrity_and_contract"
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "description": "GIVEN the primer, WHEN read, THEN it is marked non-enforced reference material",
      "verify": "cargo test -p but-authz --test invariant_build_gates steer_primer_marked_non_enforced"
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "primer four required statements present",
      "maps_to_ac": "AC-1",
      "verify": "cargo test -p but-authz --test invariant_build_gates steer_primer_contains_required_statements"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "engine non-dependence grep zero matches + teeth",
      "maps_to_ac": "AC-2",
      "verify": "cargo test -p but-authz --test invariant_build_gates steer_primer_engine_independent"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "goal-integrity + class/do_not contract present",
      "maps_to_ac": "AC-3",
      "verify": "cargo test -p but-authz --test invariant_build_gates steer_primer_goal_integrity_and_contract"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "non-enforced marker present",
      "maps_to_ac": "AC-4",
      "verify": "cargo test -p but-authz --test invariant_build_gates steer_primer_marked_non_enforced"
    }
  ]
}
-->
