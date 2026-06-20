# GATES-REM-002: Fix sprint human testing gate prose for commit-coverage reality

## What this does

The Sprint 04 Human Testing Gate in `SPRINT.md` still instructs testers to validate a flow that the sprint itself acknowledges is incoherent against live code: step 6 says `integrate_branch_with_steps` / `branch apply` advance protected `main` and are rejected as `branch.protected`, while the Overview and upstream advisory note that `branch::apply` bails on the target and `integrate_branch_with_steps` writes a feature branch. Only `worktree_integrate` genuinely advances a protected target. This task rewrites the Human Testing Gate and removes the stale advisory so the sprint contract matches the code it ships.

## Why

A sprint's human test gate must be executable. Incoherent instructions block the human reviewer and undermine confidence in the automated proofs. This is a documentation/contract fix that reconciles the PRD test steps with the live implementation discovered during GATES-007.

## How to verify

PRIMARY **AC-1** — The `SPRINT.md` Human Testing Gate no longer contains the false `apply/integrate advance protected main` language, and the upstream advisory is removed.

## Scope

- `crates/but-api/src/branch.rs` (READ-ONLY, for verification) — confirm the public `apply` bails on target and `apply_branch_integration` writes a feature branch.
- `crates/but-worktrees/src/integrate.rs` (READ-ONLY, for verification) — confirm `worktree_integrate` advances `target`.
- `.spec/prds/governance/tasks/sprint-04-gates-deepening/SPRINT.md` (MODIFY) — rewrite the Human Testing Gate and remove the upstream advisory.

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: GATES-REM-002 - Fix sprint human testing gate prose for commit-coverage reality
================================================================================

TASK_TYPE:  INFRA
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     XS (30 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-GATES-01
CAPABILITIES: CAP-CONFIG-01

RUNTIME_COMMANDS:
  verify: cat .spec/prds/governance/tasks/sprint-04-gates-deepening/SPRINT.md | grep -E 'step 6|apply|integrate|worktree_integrate|branch\.protected|perm\.denied'

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
SPRINT.md's Human Testing Gate is internally consistent and matches the live but-api behavior proven by GATES-007: `worktree_integrate` targeting a protected branch proves `branch.protected`; `branch::apply` and `apply_branch_integration` prove `contents:write` authorization via `perm.denied` (read-only principal denied). The upstream advisory flagging the incoherent prose is removed because the prose is fixed.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST change the sprint gate text to describe the actual but-api behavior: apply/integrate do NOT advance a protected trunk, so they cannot be rejected as `branch.protected`; their real coverage is `perm.denied` for a contents:read principal via the workspace-target-ref gate.
- [MUST] MUST remove the "Upstream advisory" paragraphs that only exist because the gate text is wrong.
- [NEVER] NEVER weaken the overall gate requirement — the sprint still must prove mechanism-agnostic coverage, but the mechanism is now described honestly.
- [STRICTLY] STRICTLY confine edits to SPRINT.md; do not change production code or task files.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1: SPRINT.md Human Testing Gate step 6 is rewritten to test `worktree_integrate` → `branch.protected` and `branch::apply` / `apply_branch_integration` → `perm.denied` (contents:read denied).
- [ ] AC-2: The upstream advisory paragraphs are removed.
- [ ] The Tasks table, PRD Coverage, and Capability Coverage sections remain unchanged.

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA
--------------------------------------------------------------------------------

AC-1: Human Testing Gate step 6 matches live but-api behavior
  GIVEN: SPRINT.md with the current incoherent step 6 and upstream advisory
  WHEN:  the implementer rewrites step 6 and removes the advisory
  THEN:  step 6 reads something like: "Run `but worktree integrate` targeting protected `main` → rejected `branch.protected`; run `but branch apply` and `but branch integrate` on a feature branch by a contents:read principal → rejected `perm.denied`." No step claims apply/integrate advance protected main.
  TEST_TIER: unit (documentation)   VERIFICATION_SERVICE: source grep + manual review
  VERIFY: ! grep -E 'step 6.*branch apply.*protected main|step 6.*integrate.*protected main|apply.*advancing protected main' .spec/prds/governance/tasks/sprint-04-gates-deepening/SPRINT.md

AC-2: Upstream advisory is removed
  GIVEN: SPRINT.md containing the upstream advisory paragraphs about step 6
  WHEN:  the implementer removes them
  THEN:  the file no longer contains the advisory text
  TEST_TIER: unit (documentation)   VERIFICATION_SERVICE: source grep
  VERIFY: ! grep -E 'Upstream advisory|R2c.*LOW|apply bails on the target' .spec/prds/governance/tasks/sprint-04-gates-deepening/SPRINT.md

--------------------------------------------------------------------------------
TEST CRITERIA
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): SPRINT.md step 6 does not claim apply/integrate advance protected main.
    VERIFY: ! grep -E 'step 6.*apply.*protected main|step 6.*integrate.*protected main' .spec/prds/governance/tasks/sprint-04-gates-deepening/SPRINT.md
- TC-2 (-> AC-1): SPRINT.md step 6 explicitly mentions `worktree_integrate` → `branch.protected` and apply/integrate → `perm.denied`.
    VERIFY: grep -E 'worktree_integrate.*branch\.protected|apply.*perm\.denied|integrate.*perm\.denied' .spec/prds/governance/tasks/sprint-04-gates-deepening/SPRINT.md
- TC-3 (-> AC-2): No upstream advisory paragraphs remain in SPRINT.md.
    VERIFY: ! grep -E 'Upstream advisory' .spec/prds/governance/tasks/sprint-04-gates-deepening/SPRINT.md

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-CONFIG-01
provides: a human-testable sprint gate that honestly reflects the mechanism-agnostic coverage shipped by GATES-007.
consumes: GATES-007's re-grounding of the apply/integrate/worktree behavior.
boundary_contracts:
  - CAP-CONFIG-01: the sprint gate text now reads governance from the correct ref and mutation semantics for each entry point.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - .spec/prds/governance/tasks/sprint-04-gates-deepening/SPRINT.md (MODIFY) — rewrite the Human Testing Gate and remove the upstream advisory.
writeProhibited:
  - Any production source file.
  - Any existing task file.
  - Any file not listed in write_allowed.

--------------------------------------------------------------------------------
OUT OF SCOPE
--------------------------------------------------------------------------------
  - Changing the implementation (GATES-007 owns that).
  - Adding new tests or scripts.
  - Modifying ROADMAP.md or any other PRD file.

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. .spec/prds/governance/tasks/sprint-04-gates-deepening/SPRINT.md (full)
   Focus: the file to edit; identify the Human Testing Gate section and the upstream advisory paragraphs.
2. crates/but-workspace/src/branch/apply.rs (225-233)
   Focus: confirm apply bails on target.
3. crates/but-worktrees/src/integrate.rs (55-57)
   Focus: confirm worktree_integrate advances target.
4. .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-007-mechanism-agnostic-commit-gate.md
   Focus: the re-grounded prose to mirror in SPRINT.md.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- Documentation grep passes: the false phrases are absent and the correct phrases are present.
- No accidental production changes: `git diff --stat` shows only SPRINT.md.

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: Edit the sprint contract so the human test gate matches the actual but-api behavior. Keep the same overall coverage claim (mechanism-agnostic commit gate) but describe each mechanism's real outcome.
pattern_source: GATES-007 upstream advisory and re-grounded constraints.
anti_pattern: Leaving the false instruction in place and relying on the advisory to "explain" it; changing the implementation to match the false instruction.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — Edits the sprint documentation to match the live but-api behavior, ensuring the human test gate is executable and honest.
reviewer: rust-reviewer
coding_standards: crates/AGENTS.md, crates/but/AGENTS.md

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: GATES-007 (must be complete so the live behavior is known)
Blocks:     Sprint 04 human testing gate execution
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "GATES-REM-002",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": false,
    "requires_red_evidence": false,
    "requires_seeded_evidence": false
  },
  "requirements": [
    { "id": "AC-1", "type": "acceptance_criterion", "primary": true, "description": "SPRINT.md Human Testing Gate step 6 matches live but-api behavior (worktree_integrate -> branch.protected; apply/integrate -> perm.denied)", "verify": "! grep -E 'step 6.*branch apply.*protected main|step 6.*integrate.*protected main|apply.*advancing protected main' .spec/prds/governance/tasks/sprint-04-gates-deepening/SPRINT.md", "maps_to_ac": null },
    { "id": "AC-2", "type": "acceptance_criterion", "description": "Upstream advisory paragraphs are removed from SPRINT.md", "verify": "! grep -E 'Upstream advisory' .spec/prds/governance/tasks/sprint-04-gates-deepening/SPRINT.md", "maps_to_ac": null },
    { "id": "TC-1", "type": "test_criterion", "description": "SPRINT.md step 6 does not claim apply/integrate advance protected main", "verify": "! grep -E 'step 6.*apply.*protected main|step 6.*integrate.*protected main' .spec/prds/governance/tasks/sprint-04-gates-deepening/SPRINT.md", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "SPRINT.md step 6 explicitly mentions worktree_integrate -> branch.protected and apply/integrate -> perm.denied", "verify": "grep -E 'worktree_integrate.*branch\\.protected|apply.*perm\\.denied|integrate.*perm\\.denied' .spec/prds/governance/tasks/sprint-04-gates-deepening/SPRINT.md", "maps_to_ac": "AC-1" },
    { "id": "TC-3", "type": "test_criterion", "description": "No upstream advisory paragraphs remain in SPRINT.md", "verify": "! grep -E 'Upstream advisory' .spec/prds/governance/tasks/sprint-04-gates-deepening/SPRINT.md", "maps_to_ac": "AC-2" }
  ]
}
-->
</content>
</invoke>
