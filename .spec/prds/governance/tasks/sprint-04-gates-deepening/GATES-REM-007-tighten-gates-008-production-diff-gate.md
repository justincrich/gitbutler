# GATES-REM-007: Tighten GATES-008 production-diff gate to zero-diff baseline range

## What this does

GATES-008 AC-2 claims the task must not modify production merge-path classification logic in `crates/but-api/src/legacy/merge_gate.rs`. The current verification greps added lines for three specific token patterns (`undefined_required_groups`, `fn load_merge_governance_config`, `config_invalid`). This is too narrow: it misses deletions, rewrites, duplicate logic with different names, and changes outside the current diff. This task replaces the grep with a baseline-range zero-diff gate: `git diff --exit-code <pre_task_sha>...HEAD -- crates/but-api/src/legacy/merge_gate.rs`.

## Why

Sprint 04 · PRD UC-AUTHZ-04 · capability CAP-CONFIG-01. GATES-008 is a standalone proof task; AUTHZ-004 owns the merge-path fail-closed classification. Any modification to `merge_gate.rs` production code violates that ownership boundary. A zero-diff baseline check is the only deterministic way to enforce "no production change" across the whole task lifecycle, not just the current working-tree diff.

## How to verify

PRIMARY **AC-1** — `tools/governance-checks/check_merge_gate_production_unchanged.sh` exits 0 for the GATES-008 baseline and fails if any production line in `merge_gate.rs` is added, deleted, or modified.

## Scope

- New script `tools/governance-checks/check_merge_gate_production_unchanged.sh` (CREATE or modify from GATES-REM-005) — baseline-range zero-diff check for `crates/but-api/src/legacy/merge_gate.rs`.
- GATES-008 task file (MODIFY) — update AC-2/TC-3 verification command to use the zero-diff script.

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: GATES-REM-007 - Tighten GATES-008 production-diff gate to zero-diff baseline range
================================================================================

TASK_TYPE:  INFRA
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     XS (60 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-AUTHZ-04, T-GATES-019
CAPABILITIES: CAP-CONFIG-01

RUNTIME_COMMANDS:
  test:  ./tools/governance-checks/check_merge_gate_production_unchanged.sh
  check: cargo check -p but-api --all-targets
  lint:  cargo clippy -p but-api --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
GATES-008's "no production change in merge_gate.rs" invariant is enforced by a zero-diff baseline-range check against the commit that existed before the task started. The script is deterministic, fails if any production line is touched, and is referenced by the updated GATES-008 task file. The pre-task baseline SHA is recorded in the task file and the script.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST use a baseline-range diff (`git diff --exit-code <pre_task_sha>...HEAD -- crates/but-api/src/legacy/merge_gate.rs`) rather than a narrow added-line grep.
- [MUST] MUST pick a stable baseline SHA that represents the state before GATES-008 production work began. Since GATES-008 is already merged, the baseline can be the parent commit of the GATES-008 merge or the commit before the GATES-008 branch diverged from master.
- [MUST] MUST update the GATES-008 task file to reference the zero-diff script and record the chosen baseline SHA.
- [NEVER] NEVER rely on the working-tree diff alone; committed or staged changes to `merge_gate.rs` must also fail the check.
- [STRICTLY] STRICTLY keep the script portable (bash + git) and repo-relative.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: the zero-diff script exits 0 against the current master and would fail if any line in `merge_gate.rs` changed relative to the baseline.
- [ ] AC-2: GATES-008 AC-2/TC-3 verification command references the zero-diff script and records the baseline SHA.
- [ ] AC-3: the script has a documented negative test (manual) showing it detects a deletion or rename in `merge_gate.rs`.
- [ ] All verification gates pass; only write_allowed files modified.

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA
--------------------------------------------------------------------------------

AC-1: Zero-diff script enforces no production change in merge_gate.rs [PRIMARY]
  GIVEN: a stable baseline SHA and the current master
  WHEN:  the zero-diff script is run
  THEN:  it exits 0 if `crates/but-api/src/legacy/merge_gate.rs` is identical to the baseline in the range, and exits non-zero if any line was added, deleted, or modified
  TEST_TIER: unit (script)   VERIFICATION_SERVICE: git diff
  VERIFY: ./tools/governance-checks/check_merge_gate_production_unchanged.sh
  SCENARIO: NEGATIVE_CONTROL would fail if the script used a narrow grep or only checked the working tree.

AC-2: GATES-008 task file references the script and records the baseline
  GIVEN: GATES-008 task file
  WHEN:  the AC-2/TC-3 verification command is updated
  THEN:  it runs `check_merge_gate_production_unchanged.sh` and the task file records the baseline SHA (e.g., "Baseline SHA: a1b2c3d")
  TEST_TIER: unit (documentation)   VERIFICATION_SERVICE: source grep
  VERIFY: grep -E 'check_merge_gate_production_unchanged\.sh|Baseline SHA' .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-008-merge-gate-failclosed-target-ref-only.md

AC-3: Script documents a negative test
  GIVEN: the script
  WHEN:  the implementer adds a header comment explaining how to verify it fails on a real change
  THEN:  a reviewer can manually confirm the script detects a deletion or rename in `merge_gate.rs`
  TEST_TIER: unit (documentation)   VERIFICATION_SERVICE: source review
  VERIFY: grep -E 'NEGATIVE_TEST|manual test|deletion|rename' tools/governance-checks/check_merge_gate_production_unchanged.sh

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): `check_merge_gate_production_unchanged.sh` exits 0 on the current master.
    VERIFY: ./tools/governance-checks/check_merge_gate_production_unchanged.sh
- TC-2 (-> AC-1): the script would fail if any line in `merge_gate.rs` changed relative to the baseline.
    VERIFY: (documented in script header or manual test)
- TC-3 (-> AC-2): GATES-008 task file references the script and records the baseline SHA.
    VERIFY: grep -E 'check_merge_gate_production_unchanged\.sh|Baseline SHA' .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-008-merge-gate-failclosed-target-ref-only.md
- TC-4 (-> AC-3): the script documents a negative test.
    VERIFY: grep -E 'NEGATIVE_TEST|manual test|deletion|rename' tools/governance-checks/check_merge_gate_production_unchanged.sh

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-CONFIG-01
provides: a deterministic zero-diff baseline gate that prevents GATES-008 from silently becoming a competing owner of AUTHZ-004's merge-path classification.
consumes: GATES-008's existing ownership claim and the git history of `merge_gate.rs`.
boundary_contracts:
  - CAP-CONFIG-01: the target-ref-only proof is a consumer-side test; the production classification that reads the target ref remains unchanged.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - tools/governance-checks/check_merge_gate_production_unchanged.sh (CREATE or MODIFY) — zero-diff baseline-range check for `merge_gate.rs`.
  - .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-008-merge-gate-failclosed-target-ref-only.md (MODIFY) — update AC-2/TC-3 verification command and record the baseline SHA.
writeProhibited:
  - Any production source file.
  - The substantive AC/TC text (only verify commands and a baseline note change).
  - Any file not listed in write_allowed.

--------------------------------------------------------------------------------
OUT OF SCOPE
--------------------------------------------------------------------------------
  - Changing the GATES-008 implementation or tests.
  - Modifying `merge_gate.rs` itself (the script must prove it is unchanged).
  - Adding a baseline check for any file other than `merge_gate.rs`.

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-api/src/legacy/merge_gate.rs (full)
   Focus: the file to protect from changes; confirm the current state is the AUTHZ-004 landed state.
2. .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-008-merge-gate-failclosed-target-ref-only.md (80-85, 158)
   Focus: the existing AC-2/TC-3 direct-diff grep to replace.
3. .spec/prds/governance/tasks/sprint-02-authz-fail-closed-identity-confinement/AUTHZ-004-merge-gate-fail-closed.md
   Focus: the owner of the merge-path classification; confirm that GATES-008 must not modify it.
4. `git log --oneline -- crates/but-api/src/legacy/merge_gate.rs`
   Focus: identify a stable baseline SHA before GATES-008 production changes.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- Zero-diff script exists and passes: `./tools/governance-checks/check_merge_gate_production_unchanged.sh` -> Exit 0.
- GATES-008 references it and records the baseline SHA: grep above.
- Script documents a negative test: grep above.
- Crate compiles: `cargo check -p but-api --all-targets` -> Exit 0.
- Clippy clean: `cargo clippy -p but-api --all-targets` -> Exit 0.

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: A bash script that runs `git diff --exit-code <baseline_sha>...HEAD -- crates/but-api/src/legacy/merge_gate.rs`. The baseline SHA is chosen from the git history before GATES-008 production work and is recorded in the task file. The script can be run in CI and will fail if any committed, staged, or unstaged change affects `merge_gate.rs` relative to the baseline.
pattern_source: red-hat F7 finding and AUTHZ-004 ownership boundary.
anti_pattern: A narrow added-line grep; relying only on the working-tree diff; using a baseline that is not recorded in the task file.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — Replaces the narrow GATES-008 production-diff grep with a zero-diff baseline-range check and updates the task file to reference it.
reviewer: rust-reviewer
coding_standards: crates/AGENTS.md, brain/docs/verification-discovery.md

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: GATES-008 (the task whose ownership gate is tightened), AUTHZ-004 (the owner of merge_gate.rs)
Blocks:     Sprint 04 structural-invariant closure
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "GATES-REM-007",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": false,
    "requires_red_evidence": true,
    "requires_seeded_evidence": false
  },
  "requirements": [
    { "id": "AC-1", "type": "acceptance_criterion", "primary": true, "description": "Zero-diff script enforces no production change in merge_gate.rs relative to a stable baseline", "verify": "./tools/governance-checks/check_merge_gate_production_unchanged.sh", "maps_to_ac": null },
    { "id": "AC-2", "type": "acceptance_criterion", "description": "GATES-008 task file references the script and records the baseline SHA", "verify": "grep -E 'check_merge_gate_production_unchanged\\.sh|Baseline SHA' .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-008-merge-gate-failclosed-target-ref-only.md", "maps_to_ac": null },
    { "id": "AC-3", "type": "acceptance_criterion", "description": "The script documents a negative test showing it detects a deletion or rename in merge_gate.rs", "verify": "grep -E 'NEGATIVE_TEST|manual test|deletion|rename' tools/governance-checks/check_merge_gate_production_unchanged.sh", "maps_to_ac": null },
    { "id": "TC-1", "type": "test_criterion", "description": "check_merge_gate_production_unchanged.sh exits 0 on the current master", "verify": "./tools/governance-checks/check_merge_gate_production_unchanged.sh", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "The script would fail if any line in merge_gate.rs changed relative to the baseline", "verify": "(documented in script header or manual test)", "maps_to_ac": "AC-1" },
    { "id": "TC-3", "type": "test_criterion", "description": "GATES-008 task file references the script and records the baseline SHA", "verify": "grep -E 'check_merge_gate_production_unchanged\\.sh|Baseline SHA' .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-008-merge-gate-failclosed-target-ref-only.md", "maps_to_ac": "AC-2" },
    { "id": "TC-4", "type": "test_criterion", "description": "The script documents a negative test", "verify": "grep -E 'NEGATIVE_TEST|manual test|deletion|rename' tools/governance-checks/check_merge_gate_production_unchanged.sh", "maps_to_ac": "AC-3" }
  ]
}
-->
</content>
</invoke>
