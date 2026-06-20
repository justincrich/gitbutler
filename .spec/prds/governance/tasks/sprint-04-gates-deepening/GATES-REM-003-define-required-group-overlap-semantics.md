# GATES-REM-003: Define required-group overlap semantics for two-tier human+AI model

## What this does

The red-hat review for Sprint 04 found that the two-tier "human-at-feature + AI-at-code" model is undefined when a single principal belongs to both required groups (e.g., a maintainer who is also a code-reviewer). The current `review_requirement::evaluate` checks each `require_approval_from_group` entry independently against the same approval set, so one approval from a dual-member principal can satisfy both groups. This task defines and implements the overlap policy: either reject overlapping required groups as `config.invalid` or require distinct reviewer identities per required group.

## Why

Sprint 04 · PRD UC-LOOP-02 (two-group AI+human model) · capability CAP-AUTHZ-01. The existing GATES-006 tests use disjoint principals (`reviewer-a` ∈ code-reviewers, `reviewer-b` ∈ maintainers) and do not exercise the overlap case. The two-tier model is only sound if it cannot be satisfied by a single approval from a dual-member principal, which would collapse the human+AI distinction in practice.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-api merge_gate_overlapping_required_groups_policy` (integration, real but-api merge gate + real git + real but-db).

## Scope

- `crates/but-api/src/legacy/review_requirement.rs` (MODIFY) — add the overlap policy check in `evaluate()` before the per-group satisfaction loop, or modify the per-group loop to require distinct principals per group.
- `crates/but-api/tests/merge_gate.rs` (MODIFY) — add an integration test with a fixture where one principal is a member of both required groups, and prove that a single approval does not satisfy the two-tier gate.

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: GATES-REM-003 - Define required-group overlap semantics for two-tier human+AI model
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     L (180 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-LOOP-02, UC-GATES-02
CAPABILITIES: CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-api merge_gate_overlapping_required_groups_policy
  check: cargo check -p but-api --all-targets
  lint:  cargo clippy -p but-api --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
The two-tier required-group model is fail-closed on overlap: either the merge gate rejects overlapping required groups as `config.invalid` (preferred), or it requires a distinct approval from a different principal for each required group. A single approval from a dual-member principal cannot satisfy both required groups. The chosen policy is documented in the task, tested with a real but-api merge gate, and does not break the existing GATES-006 disjoint-principal tests.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST define the overlap policy explicitly. Preferred fail-closed shape: reject overlapping required groups as `config.invalid` when the set of required groups share any member, because "one approval from each required group" implies distinct groups. Alternative: require distinct reviewer identities per required group (a single approval from one principal counts for at most one group). Document the chosen policy and justify it against UC-LOOP-02.
- [MUST] MUST preserve the existing GATES-006 behavior for disjoint groups: a distinct approval from each disjoint required group still proceeds.
- [MUST] MUST seed the test approval via the governed `but-api::legacy::forge::approve_review` action, never a direct DB insert.
- [NEVER] NEVER add a role-name literal or human/AI discriminator to the enforcement source. The overlap policy must be expressed in terms of group membership and principal identity, not role labels.
- [STRICTLY] STRICTLY confine the production edit to `review_requirement.rs` if the policy is implemented in the evaluator; do not modify the merge-gate wrapper or but-authz primitives.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: a single approval from a dual-member principal cannot satisfy a two-group requirement; the gate blocks with a clear denial.
- [ ] AC-2: disjoint-group approvals still proceed as in GATES-006.
- [ ] AC-3: the chosen policy is documented in the task file and reflected in the SPRINT.md overview if needed.
- [ ] All verification gates pass; only write_allowed files modified.

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: Overlapping required groups fail closed on a single dual-member approval [PRIMARY]
  GIVEN: fixture `merge_two_group_overlap` (target-ref gate require_approval_from_group=["code-reviewers","maintainers"]; a principal `reviewer-x` who is a member of BOTH groups; a separate `maint` holds merge), BUT_AGENT_HANDLE=maint
  WHEN:  only `reviewer-x` approves @head and maint attempts the merge
  THEN:  the merge is blocked with error.code=="config.invalid" (if overlap rejection policy) OR error.code=="gate.review_required" with a clear unmet entry naming the still-missing distinct principal (if distinct-identity policy). The trunk HEAD sha remains unchanged.
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api merge gate + real git + real but-db
  VERIFY: cargo test -p but-api merge_gate_overlapping_required_groups_policy
  SCENARIO: NEGATIVE_CONTROL would fail if a single dual-member approval satisfies both groups (the two-tier model collapses), or if the denial is generic and does not explain the policy.

AC-2: Disjoint-group approvals still proceed
  GIVEN: fixture `merge_two_group` from GATES-006 (reviewer-a ∈ code-reviewers, reviewer-b ∈ maintainers, maint holds merge), BUT_AGENT_HANDLE=maint
  WHEN:  reviewer-a and reviewer-b each approve @head and maint attempts the merge
  THEN:  the gate permits the merge and execution reaches the forge call, just as it does today
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api merge gate + real git + real but-db
  VERIFY: cargo test -p but-api merge_gate_two_group_both_present_proceeds
  SCENARIO: NEGATIVE_CONTROL would fail if the overlap policy over-rejects the existing disjoint-group case.

AC-3: Policy choice is documented
  GIVEN: the task file and SPRINT.md
  WHEN:  the implementer selects and implements the overlap policy
  THEN:  the task file's CRITICAL CONSTRAINTS and NOTES sections state the selected policy and rationale; no human/AI role label is introduced
  TEST_TIER: unit (documentation)   VERIFICATION_SERVICE: source review
  VERIFY: grep -E 'overlap policy|distinct principal|config\.invalid' .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-REM-003-define-required-group-overlap-semantics.md

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): a single approval from a dual-member principal blocks the two-group merge with a clear denial.
    VERIFY: cargo test -p but-api merge_gate_overlapping_required_groups_policy
- TC-2 (-> AC-2): the existing disjoint-group both-present positive control still passes.
    VERIFY: cargo test -p but-api merge_gate_two_group_both_present_proceeds
- TC-3 (-> AC-1, structural): the enforcement source does not contain a human/AI role label or group-name literal as a standalone word.
    VERIFY: ! grep -rEni '\bimplementer\b|\breviewer\b|\bmaintainer\b|is_bot|is_human|"human"|"ai"' crates/but-api/src/legacy/review_requirement.rs

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides: a fail-closed definition of required-group overlap semantics, ensuring the two-tier human+AI model cannot be satisfied by a single dual-member approval.
consumes: GATES-005/GATES-006's per-group evaluator (`review_requirement::evaluate`), the existing `merge_gated_repo` fixture, and the governed `approve_branch` seed action.
boundary_contracts:
  - CAP-AUTHZ-01: the per-group review requirement is evaluated as part of the merge gate; the policy ensures that overlapping required groups either require distinct principals or are rejected as invalid configuration.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-api/src/legacy/review_requirement.rs (MODIFY) — implement the overlap policy check in the evaluator.
  - crates/but-api/tests/merge_gate.rs (MODIFY) — add the overlap-policy integration test and fixture.
writeProhibited:
  - crates/but-api/src/legacy/merge_gate.rs — owned by GATES-003/AUTHZ-004.
  - crates/but-authz/** — consume group membership primitives; do not modify them.
  - Any file not listed in write_allowed.

--------------------------------------------------------------------------------
OUT OF SCOPE
--------------------------------------------------------------------------------
  - Re-implementing per-group satisfaction or self/stale dismissal (GATES-005 owns that).
  - Changing the merge-gate wrapper or principal resolution.
  - Testing forgeable direct-DB-write or raw-git bypasses (accepted leaks R6/R1).

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-api/src/legacy/review_requirement.rs (33-124)
   Focus: the evaluator to modify; understand the per-group loop and `has_group_approval` membership check.
2. crates/but-api/tests/merge_gate.rs (185-221, 451-587, 650-655)
   Focus: the existing `merge_two_group` fixture and the `approve_branch` helper to extend for the overlap case.
3. .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-006-per-required-group-approval.md
   Focus: the disjoint-group test cases that must remain green.
4. .spec/prds/governance/07-uc-loop.md (37-46)
   Focus: UC-LOOP-02 AC-5 — no code distinguishes human from AI; the overlap policy must preserve this.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- Overlap policy integration test passes: `cargo test -p but-api merge_gate_overlapping_required_groups_policy` -> Exit 0.
- Existing disjoint-group test still passes: `cargo test -p but-api merge_gate_two_group_both_present_proceeds` -> Exit 0.
- No role/group-name literal in enforcement source: `! grep -rEni '\bimplementer\b|\breviewer\b|\bmaintainer\b|is_bot|is_human|"human"|"ai"' crates/but-api/src/legacy/review_requirement.rs` -> No matches.
- Crate compiles incl. tests: `cargo check -p but-api --all-targets` -> Exit 0.
- Clippy clean: `cargo clippy -p but-api --all-targets` -> Exit 0.

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: Detect the overlap case in `evaluate()` before or during the per-group loop. If the policy is "reject overlap": compute the intersection of required group member sets; if non-empty, return `config.invalid` with a message naming the overlap. If the policy is "distinct principals per group": track which principal satisfied each group and ensure no single principal is reused across groups. The chosen policy must be fail-closed and preserve the no-role-label invariant.
pattern_source: crates/but-api/src/legacy/review_requirement.rs:50-58 (per-group loop) and the GATES-006 fixture.
anti_pattern: Allowing a single dual-member approval to satisfy both groups (collapses the two-tier model); adding a role label or hardcoded group literal to express the policy; implementing the policy in the merge-gate wrapper instead of the evaluator.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — Defines the required-group overlap policy in the `review_requirement` evaluator and proves it with an integration test against the real but-api merge gate, ensuring the two-tier human+AI model remains sound when group memberships overlap.
reviewer: rust-reviewer
coding_standards: crates/AGENTS.md, crates/WORKSPACE_MODEL.md, crates/but-api/src/legacy/review_requirement.rs

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: GATES-005, GATES-006, GRPS-001
Blocks:     Sprint 05, Sprint 06b (via Sprint 04 completion)
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "GATES-REM-003",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "requirements": [
    { "id": "AC-1", "type": "acceptance_criterion", "primary": true, "description": "A single approval from a dual-member principal cannot satisfy a two-group requirement; the gate blocks with a clear denial", "verify": "cargo test -p but-api merge_gate_overlapping_required_groups_policy", "maps_to_ac": null },
    { "id": "AC-2", "type": "acceptance_criterion", "description": "Disjoint-group approvals still proceed as in GATES-006", "verify": "cargo test -p but-api merge_gate_two_group_both_present_proceeds", "maps_to_ac": null },
    { "id": "AC-3", "type": "acceptance_criterion", "description": "The chosen overlap policy is documented in the task file and reflected in SPRINT.md if needed", "verify": "grep -E 'overlap policy|distinct principal|config\\.invalid' .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-REM-003-define-required-group-overlap-semantics.md", "maps_to_ac": null },
    { "id": "TC-1", "type": "test_criterion", "description": "A single approval from a dual-member principal blocks the two-group merge with a clear denial", "verify": "cargo test -p but-api merge_gate_overlapping_required_groups_policy", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "The existing disjoint-group both-present positive control still passes", "verify": "cargo test -p but-api merge_gate_two_group_both_present_proceeds", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "The enforcement source does not contain a human/AI role label or group-name literal as a standalone word", "verify": "! grep -rEni '\\bimplementer\\b|\\breviewer\\b|\\bmaintainer\\b|is_bot|is_human|\"human\"|\"ai\"' crates/but-api/src/legacy/review_requirement.rs", "maps_to_ac": "AC-1" }
  ]
}
-->
</content>
</invoke>
