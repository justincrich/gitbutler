# CATCHUP-004: Build altitude + cold-boot lint tooling and verify all flows [LOAD-BEARING]

> Sprint: [sprint-00-catchup-walking-skeleton](./SPRINT.md)
> Agent: rust-implementer · Reviewer: rust-reviewer
> Estimate: 180 min · Type: verify/fix · Status: ✅ Completed
> Proposed By: rust-planner · Reviewer: rust-reviewer · Cycle: 1
> Updated: 2026-06-23T18:40:00Z

## Background

This is the **load-bearing task**. The sprint's "done" requires `tools/gate-evidence/gate_evidence_check.py`
to write `sprint-goal-state.json` with `verdict: pass` — but the four gate-evidence + e2e-surface
scripts DO NOT EXIST yet. Only `tools/governance-checks/` (4 scripts) is present. This task builds
the missing tooling, then runs it against every locked flow.

The cold-boot concern (tests seed config instead of committing it) is **already satisfied by
inspection**: the `governed_repo()` fixture at `crates/but-api/tests/commit_gate.rs:727` commits
config via real `git add … && git commit` through `but_testsupport::invoke_bash`. The cold-boot
lint the implementer builds should pass.

## Critical Constraints

- MUST NOT narrate green; the gate verdict is code-computed by `tools/gate-evidence/gate_evidence_check.py` writing `sprint-goal-state.json` with `verdict: pass`.
- MUST implement the four missing Python scripts only under the allowed tool paths; no tooling may be added to unrelated directories.
- MUST parse `.spec/prds/governance/tasks/sprint-00-catchup-walking-skeleton/human-flows.json` as the single source of truth for the 14 flows; never hardcode the flow list.
- MUST use the real `but` / `but-authz` / `but-api` CLI and cargo test surfaces for each flow; no mocks.
- MUST cold-boot check confirm repository config is committed via real git operations (not injected/seeded) per the `governed_repo` fixture; if the check fails, reopen CATCHUP-001/002/003 as fix tasks.
- NEVER modify production Rust source just to make a lint script pass; fix the tooling, the test fixture, or the evidentiary data path.

## Specification

**Objective:** Build the missing e2e-surface and gate-evidence lint tooling, then run it against every locked flow to produce a code-computed pass verdict.

**Success state:** All four verification tools exist, execute against the 14 locked flows, and `tools/gate-evidence/gate_evidence_check.py` writes `sprint-goal-state.json` containing `verdict: pass`.

## Acceptance Criteria

- [x] AC-1: GIVEN the required e2e-surface and gate-evidence tooling does not yet exist, WHEN the implementer creates the four Python scripts, THEN `tools/e2e-surface/e2e_surface_check.py`, `tools/e2e-surface/coldboot_check.py`, `tools/gate-evidence/gate_evidence_check.py`, and `tools/gate-evidence/flow_coverage_check.py` exist, are executable, parse the locked human-flows.json, accept a `--sprint` argument, exit 0 on success, and exit non-zero on failure with a clear stderr message.
  - Verify: `ls -la tools/e2e-surface/ tools/gate-evidence/ && python3 tools/e2e-surface/e2e_surface_check.py --help`
- [x] AC-2: GIVEN `tools/e2e-surface/e2e_surface_check.py` is implemented, WHEN it is run for `sprint-00-catchup-walking-skeleton` against each locked flow, THEN it exits 0 and reports `surface_ok` for every one of the 14 flows.
  - Verify: `python3 tools/e2e-surface/e2e_surface_check.py --sprint sprint-00-catchup-walking-skeleton`
- [x] AC-3: GIVEN `tools/e2e-surface/coldboot_check.py` is implemented and the locked flows use the `governed_repo` fixture that commits config via real git, WHEN it is run for `sprint-00-catchup-walking-skeleton`, THEN it exits 0 and reports `coldboot_ok` for every flow because no config is injected at test time.
  - Verify: `python3 tools/e2e-surface/coldboot_check.py --sprint sprint-00-catchup-walking-skeleton`
- [x] AC-4: GIVEN `tools/gate-evidence/flow_coverage_check.py` is implemented, WHEN it is run with `--sprint sprint-00-catchup-walking-skeleton`, THEN it exits 0 and confirms every locked flow has a defined, executable run_cmd and is covered by a verification gate.
  - Verify: `python3 tools/gate-evidence/flow_coverage_check.py --sprint sprint-00-catchup-walking-skeleton`
- [x] AC-5: GIVEN all previous surface, cold-boot, and coverage checks have returned exit code 0, WHEN `tools/gate-evidence/gate_evidence_check.py` is run with `--sprint sprint-00-catchup-walking-skeleton`, THEN it writes `.spec/prds/governance/tasks/sprint-00-catchup-walking-skeleton/sprint-goal-state.json` containing `verdict: pass`, computed from the exit codes and per-flow surface/coldboot statuses.
  - Verify: `python3 tools/gate-evidence/gate_evidence_check.py --sprint sprint-00-catchup-walking-skeleton && grep '"verdict": "pass"' .spec/prds/governance/tasks/sprint-00-catchup-walking-skeleton/sprint-goal-state.json`
- [x] AC-6: GIVEN all four lint tools exist and the human-flows.json source of truth is locked, WHEN the final orchestrator gate command runs all four checks in order, THEN every check exits 0 and `sprint-goal-state.json` contains `verdict: pass`.
  - Verify: `python3 tools/e2e-surface/e2e_surface_check.py --sprint sprint-00-catchup-walking-skeleton && python3 tools/e2e-surface/coldboot_check.py --sprint sprint-00-catchup-walking-skeleton && python3 tools/gate-evidence/flow_coverage_check.py --sprint sprint-00-catchup-walking-skeleton && python3 tools/gate-evidence/gate_evidence_check.py --sprint sprint-00-catchup-walking-skeleton`

## Test Criteria

| #    | Boolean Statement                                                                                                                                                            | Maps To | Verify                                                                                           | Status             |
| ---- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------- | ------------------------------------------------------------------------------------------------ | ------------------ |
| TC-1 | `tools/e2e-surface/e2e_surface_check.py` exists, is executable, parses human-flows.json, and accepts `--sprint`.                                                             | AC-1    | `python3 tools/e2e-surface/e2e_surface_check.py --help`                                          | [x] TRUE [ ] FALSE |
| TC-2 | `tools/e2e-surface/coldboot_check.py` exists, is executable, parses human-flows.json, and accepts `--sprint`.                                                                | AC-1    | `python3 tools/e2e-surface/coldboot_check.py --help`                                             | [x] TRUE [ ] FALSE |
| TC-3 | `tools/gate-evidence/gate_evidence_check.py` exists, is executable, parses human-flows.json, and accepts `--sprint`.                                                         | AC-1    | `python3 tools/gate-evidence/gate_evidence_check.py --help`                                      | [x] TRUE [ ] FALSE |
| TC-4 | `tools/gate-evidence/flow_coverage_check.py` exists, is executable, parses human-flows.json, and accepts `--sprint`.                                                         | AC-1    | `python3 tools/gate-evidence/flow_coverage_check.py --help`                                      | [x] TRUE [ ] FALSE |
| TC-5 | `python3 tools/e2e-surface/e2e_surface_check.py --sprint sprint-00-catchup-walking-skeleton` returns exit code 0 and every flow status is `surface_ok`.                      | AC-2    | `python3 tools/e2e-surface/e2e_surface_check.py --sprint sprint-00-catchup-walking-skeleton`     | [x] TRUE [ ] FALSE |
| TC-6 | `python3 tools/e2e-surface/coldboot_check.py --sprint sprint-00-catchup-walking-skeleton` returns exit code 0 and every flow status is `coldboot_ok`.                        | AC-3    | `python3 tools/e2e-surface/coldboot_check.py --sprint sprint-00-catchup-walking-skeleton`        | [x] TRUE [ ] FALSE |
| TC-7 | `python3 tools/gate-evidence/flow_coverage_check.py --sprint sprint-00-catchup-walking-skeleton` returns exit code 0.                                                        | AC-4    | `python3 tools/gate-evidence/flow_coverage_check.py --sprint sprint-00-catchup-walking-skeleton` | [x] TRUE [ ] FALSE |
| TC-8 | `python3 tools/gate-evidence/gate_evidence_check.py --sprint sprint-00-catchup-walking-skeleton` returns exit code 0 and writes sprint-goal-state.json with `verdict: pass`. | AC-5    | `python3 tools/gate-evidence/gate_evidence_check.py --sprint sprint-00-catchup-walking-skeleton` | [x] TRUE [ ] FALSE |
| TC-9 | The full orchestrator gate command (all four checks in order) exits 0 and sprint-goal-state.json contains `verdict: pass`.                                                   | AC-6    | (see AC-6 verify)                                                                                | [x] TRUE [ ] FALSE |

## Reading List

- `tools/governance-checks/check_gate_before_guard.py` — existing governance-check script (style reference)
- `tools/governance-checks/check_no_role_literals.sh` — existing honesty-lint (style reference)
- `crates/but-api/tests/commit_gate.rs` — `governed_repo()` fixture at line 727 (commits config via real git — the cold-boot-clean reference)
- `.spec/prds/governance/tasks/sprint-00-catchup-walking-skeleton/human-flows.json` — the 14 locked flows (single source of truth)
- `.spec/prds/governance/tasks/sprint-00-catchup-walking-skeleton/SPRINT.md` — sprint gate + "what done means"

## Guardrails

**WRITE-ALLOWED:**

- `tools/e2e-surface/e2e_surface_check.py`
- `tools/e2e-surface/coldboot_check.py`
- `tools/gate-evidence/gate_evidence_check.py`
- `tools/gate-evidence/flow_coverage_check.py`
- `.spec/prds/governance/tasks/sprint-00-catchup-walking-skeleton/sprint-goal-state.json`

**WRITE-PROHIBITED:**

- `crates/**`
- `.spec/prds/governance/tasks/sprint-00-catchup-walking-skeleton/human-flows.json`
- `.spec/prds/governance/tasks/sprint-00-catchup-walking-skeleton/SPRINT.md`
- `tools/governance-checks/*`

## Verification Gates

| Command                                                                                          | Expected Exit |
| ------------------------------------------------------------------------------------------------ | ------------- |
| `python3 tools/e2e-surface/e2e_surface_check.py --sprint sprint-00-catchup-walking-skeleton`     | 0             |
| `python3 tools/e2e-surface/coldboot_check.py --sprint sprint-00-catchup-walking-skeleton`        | 0             |
| `python3 tools/gate-evidence/flow_coverage_check.py --sprint sprint-00-catchup-walking-skeleton` | 0             |
| `python3 tools/gate-evidence/gate_evidence_check.py --sprint sprint-00-catchup-walking-skeleton` | 0             |

## Agent Assignment

**Implementer:** rust-implementer — build the lint tooling (Python) and run it against the locked flows.
**Reviewer:** rust-reviewer — confirm the tooling parses human-flows.json (not hardcoded), computes the verdict from real exit codes, and that coldboot genuinely checks git-commit-vs-seed.

## Dependencies

- **Depends on:** CATCHUP-001, CATCHUP-002, CATCHUP-003 (the flows must be green before the gate verdict can be pass)
- **Blocks:** none (this is the terminal task — its AC-6 IS the sprint's "done")

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "requirements": [
    { "id": "AC-1", "kind": "ac", "statement": "GIVEN the required e2e-surface and gate-evidence tooling does not yet exist, WHEN the implementer creates the four Python scripts, THEN `tools/e2e-surface/e2e_surface_check.py`, `tools/e2e-surface/coldboot_check.py`, `tools/gate-evidence/gate_evidence_check.py`, and `tools/gate-evidence/flow_coverage_check.py` exist, are executable, parse the locked human-flows.json, accept a `--sprint` argument, exit 0 on success, and exit non-zero on failure with a clear stderr message.", "verify": "ls -la tools/e2e-surface/ tools/gate-evidence/ && python3 tools/e2e-surface/e2e_surface_check.py --help", "satisfied": true, "maps_to_ac": null },
    { "id": "AC-2", "kind": "ac", "statement": "GIVEN `tools/e2e-surface/e2e_surface_check.py` is implemented, WHEN it is run for `sprint-00-catchup-walking-skeleton` against each locked flow, THEN it exits 0 and reports `surface_ok` for every one of the 14 flows.", "verify": "python3 tools/e2e-surface/e2e_surface_check.py --sprint sprint-00-catchup-walking-skeleton", "satisfied": true, "maps_to_ac": null },
    { "id": "AC-3", "kind": "ac", "statement": "GIVEN `tools/e2e-surface/coldboot_check.py` is implemented and the locked flows use the governed_repo fixture that commits config via real git, WHEN it is run for `sprint-00-catchup-walking-skeleton`, THEN it exits 0 and reports `coldboot_ok` for every flow because no config is injected at test time.", "verify": "python3 tools/e2e-surface/coldboot_check.py --sprint sprint-00-catchup-walking-skeleton", "satisfied": true, "maps_to_ac": null },
    { "id": "AC-4", "kind": "ac", "statement": "GIVEN `tools/gate-evidence/flow_coverage_check.py` is implemented, WHEN it is run with `--sprint sprint-00-catchup-walking-skeleton`, THEN it exits 0 and confirms every locked flow has a defined, executable run_cmd and is covered by a verification gate.", "verify": "python3 tools/gate-evidence/flow_coverage_check.py --sprint sprint-00-catchup-walking-skeleton", "satisfied": true, "maps_to_ac": null },
    { "id": "AC-5", "kind": "ac", "statement": "GIVEN all previous surface, cold-boot, and coverage checks have returned exit code 0, WHEN `tools/gate-evidence/gate_evidence_check.py` is run with `--sprint sprint-00-catchup-walking-skeleton`, THEN it writes `.spec/prds/governance/tasks/sprint-00-catchup-walking-skeleton/sprint-goal-state.json` containing `verdict: pass`, computed from the exit codes and per-flow surface/coldboot statuses.", "verify": "python3 tools/gate-evidence/gate_evidence_check.py --sprint sprint-00-catchup-walking-skeleton && grep '\"verdict\": \"pass\"' .spec/prds/governance/tasks/sprint-00-catchup-walking-skeleton/sprint-goal-state.json", "satisfied": true, "maps_to_ac": null },
    { "id": "AC-6", "kind": "ac", "statement": "GIVEN all four lint tools exist and the human-flows.json source of truth is locked, WHEN the final orchestrator gate command runs all four checks in order, THEN every check exits 0 and `sprint-goal-state.json` contains `verdict: pass`.", "verify": "python3 tools/e2e-surface/e2e_surface_check.py --sprint sprint-00-catchup-walking-skeleton && python3 tools/e2e-surface/coldboot_check.py --sprint sprint-00-catchup-walking-skeleton && python3 tools/gate-evidence/flow_coverage_check.py --sprint sprint-00-catchup-walking-skeleton && python3 tools/gate-evidence/gate_evidence_check.py --sprint sprint-00-catchup-walking-skeleton", "satisfied": true, "maps_to_ac": null },
    { "id": "TC-1", "kind": "tc", "statement": "`tools/e2e-surface/e2e_surface_check.py` exists, is executable, parses human-flows.json, and accepts `--sprint`.", "maps_to_ac": "AC-1", "verify": "python3 tools/e2e-surface/e2e_surface_check.py --help", "status": "true" },
    { "id": "TC-2", "kind": "tc", "statement": "`tools/e2e-surface/coldboot_check.py` exists, is executable, parses human-flows.json, and accepts `--sprint`.", "maps_to_ac": "AC-1", "verify": "python3 tools/e2e-surface/coldboot_check.py --help", "status": "true" },
    { "id": "TC-3", "kind": "tc", "statement": "`tools/gate-evidence/gate_evidence_check.py` exists, is executable, parses human-flows.json, and accepts `--sprint`.", "maps_to_ac": "AC-1", "verify": "python3 tools/gate-evidence/gate_evidence_check.py --help", "status": "true" },
    { "id": "TC-4", "kind": "tc", "statement": "`tools/gate-evidence/flow_coverage_check.py` exists, is executable, parses human-flows.json, and accepts `--sprint`.", "maps_to_ac": "AC-1", "verify": "python3 tools/gate-evidence/flow_coverage_check.py --help", "status": "true" },
    { "id": "TC-5", "kind": "tc", "statement": "`python3 tools/e2e-surface/e2e_surface_check.py --sprint sprint-00-catchup-walking-skeleton` returns exit code 0 and every flow status is `surface_ok`.", "maps_to_ac": "AC-2", "verify": "python3 tools/e2e-surface/e2e_surface_check.py --sprint sprint-00-catchup-walking-skeleton", "status": "true" },
    { "id": "TC-6", "kind": "tc", "statement": "`python3 tools/e2e-surface/coldboot_check.py --sprint sprint-00-catchup-walking-skeleton` returns exit code 0 and every flow status is `coldboot_ok`.", "maps_to_ac": "AC-3", "verify": "python3 tools/e2e-surface/coldboot_check.py --sprint sprint-00-catchup-walking-skeleton", "status": "true" },
    { "id": "TC-7", "kind": "tc", "statement": "`python3 tools/gate-evidence/flow_coverage_check.py --sprint sprint-00-catchup-walking-skeleton` returns exit code 0.", "maps_to_ac": "AC-4", "verify": "python3 tools/gate-evidence/flow_coverage_check.py --sprint sprint-00-catchup-walking-skeleton", "status": "true" },
    { "id": "TC-8", "kind": "tc", "statement": "`python3 tools/gate-evidence/gate_evidence_check.py --sprint sprint-00-catchup-walking-skeleton` returns exit code 0 and writes sprint-goal-state.json with `verdict: pass`.", "maps_to_ac": "AC-5", "verify": "python3 tools/gate-evidence/gate_evidence_check.py --sprint sprint-00-catchup-walking-skeleton", "status": "true" },
    { "id": "TC-9", "kind": "tc", "statement": "The full orchestrator gate command (all four checks in order) exits 0 and sprint-goal-state.json contains `verdict: pass`.", "maps_to_ac": "AC-6", "verify": "python3 tools/e2e-surface/e2e_surface_check.py --sprint sprint-00-catchup-walking-skeleton && python3 tools/e2e-surface/coldboot_check.py --sprint sprint-00-catchup-walking-skeleton && python3 tools/gate-evidence/flow_coverage_check.py --sprint sprint-00-catchup-walking-skeleton && python3 tools/gate-evidence/gate_evidence_check.py --sprint sprint-00-catchup-walking-skeleton", "status": "true" }
  ]
}
-->
