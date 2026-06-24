# CATCHUP-003: Verify and fix reference-flow canary T-LOOP-006 (UC-LOOP-01/02) [JOURNEY]

> Sprint: [sprint-00-catchup-walking-skeleton](./SPRINT.md)
> Agent: rust-implementer · Reviewer: rust-reviewer
> Estimate: 90 min · Type: verify/fix · Status: ✅ Completed
> Proposed By: rust-planner · Reviewer: rust-reviewer · Cycle: 1
> Updated: 2026-06-23T18:40:00Z

## Background

This is the **load-bearing JOURNEY** task — the T-LOOP-006 walking-skeleton canary. It replays
the 3-principal implement→review→merge reference flow plus the honesty invariant at the real
`but` CLI surface. Ground truth (already probed): GREEN — 20/20 governed_loop tests pass
(including all 5 T-LOOP-006 canary tests) and the invariant_build_gates test passes.

## Critical Constraints

- MUST NOT narrate green; the verdict is the cargo exit code of the locked run_cmd.
- MUST use the real `but` binary surface via `cargo test -p but --features but-2 governed_loop` and the real but-authz surface for `invariant_build_gates`; no mocks.
- MUST preserve the `DryRun` semantics; any fix must not allow dry runs to persist refs, objects, or oplog.
- MUST diagnose any RED failure and fix it until both commands exit 0.

## Specification

**Objective:** Replay the locked reference-loop canary commands and ensure the full governed-loop suite and invariant gate exit 0.

**Success state:** `cargo test -p but --features but-2 governed_loop` passes 20/20 tests including the 5 T-LOOP-006 canary tests (`reference_flow_full_loop`, `remediation_traversable`, `dryrun_no_bypass`, `auto_merge_denied`, `unset_handle_failclosed`), and `cargo test -p but-authz invariant_build_gates` passes.

## Acceptance Criteria

- [x] AC-1: GIVEN the workspace is on the current HEAD and the but crate compiles, WHEN `cargo test -p but --features but-2 governed_loop` is executed, THEN the process exits 0, all 20 governed_loop tests pass, and the 5 T-LOOP-006 canary tests are green.
  - Verify: `cargo test -p but --features but-2 governed_loop`
- [x] AC-2: GIVEN the workspace is on the current HEAD, WHEN `cargo test -p but-authz invariant_build_gates` is executed, THEN the process exits 0 and the invariant_build_gates test passes.
  - Verify: `cargo test -p but-authz invariant_build_gates`
- [x] AC-3: GIVEN either canary command returns a non-zero exit code, WHEN the implementer diagnoses the failure and applies a minimal fix, THEN both commands are re-run and exit 0.
  - Verify: `cargo test -p but --features but-2 governed_loop && cargo test -p but-authz invariant_build_gates`

## Test Criteria

| #    | Boolean Statement                                                                                                               | Maps To | Verify                                                                                              | Status             |
| ---- | ------------------------------------------------------------------------------------------------------------------------------- | ------- | --------------------------------------------------------------------------------------------------- | ------------------ |
| TC-1 | `cargo test -p but --features but-2 governed_loop` returns exit code 0 and the 5 T-LOOP-006 canary tests are listed as passing. | AC-1    | `cargo test -p but --features but-2 governed_loop`                                                  | [x] TRUE [ ] FALSE |
| TC-2 | `cargo test -p but-authz invariant_build_gates` returns exit code 0.                                                            | AC-2    | `cargo test -p but-authz invariant_build_gates`                                                     | [x] TRUE [ ] FALSE |
| TC-3 | If AC-1 or AC-2 is initially RED, the final re-run after diagnosis/fix returns exit code 0.                                     | AC-3    | `cargo test -p but --features but-2 governed_loop && cargo test -p but-authz invariant_build_gates` | [x] TRUE [ ] FALSE |

## Reading List

- `crates/but/tests/but/command/governed_loop.rs` — the 20-test governed-loop suite (T-LOOP-006 canary)
- `crates/but-authz/tests/invariant_build_gates.rs` — honesty invariant (no role-name, no human-vs-AI predicate)
- `.spec/prds/governance/tasks/sprint-00-catchup-walking-skeleton/human-flows.json` — locked flows (CATCHUP-LOOP01-\* are the JOURNEY flows)
- `.spec/prds/governance/tasks/sprint-00-catchup-walking-skeleton/SPRINT.md` — sprint gate

## Guardrails

**WRITE-ALLOWED:**

- `crates/but/tests/but/command/governed_loop.rs`
- `crates/but/tests/but/command/snapshots/*governed_loop*.snap`
- `crates/but/tests/snapshots/*governed_loop*.snap`
- `crates/but-authz/tests/invariant_build_gates.rs`

**WRITE-PROHIBITED:**

- `crates/but/src/**`, `crates/but-api/src/**`, `crates/but-authz/src/**`
- `tools/**`, `.spec/prds/**`

## Verification Gates

| Command                                            | Expected Exit |
| -------------------------------------------------- | ------------- |
| `cargo test -p but --features but-2 governed_loop` | 0             |
| `cargo test -p but-authz invariant_build_gates`    | 0             |

## Agent Assignment

**Implementer:** rust-implementer — verify/fix Rust reference-flow canary (JOURNEY).
**Reviewer:** rust-reviewer — confirm DryRun-no-bypass and the 5 canary tests are genuinely green.

## Dependencies

- **Depends on:** none
- **Blocks:** CATCHUP-004 (the load-bearing task; if cold-boot lint flags these tests, they reopen as fix tasks)

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "requirements": [
    { "id": "AC-1", "kind": "ac", "statement": "GIVEN the workspace is on the current HEAD and the but crate compiles, WHEN `cargo test -p but --features but-2 governed_loop` is executed, THEN the process exits 0, all 20 governed_loop tests pass, and the 5 T-LOOP-006 canary tests are green.", "verify": "cargo test -p but --features but-2 governed_loop", "satisfied": true, "maps_to_ac": null },
    { "id": "AC-2", "kind": "ac", "statement": "GIVEN the workspace is on the current HEAD, WHEN `cargo test -p but-authz invariant_build_gates` is executed, THEN the process exits 0 and the invariant_build_gates test passes.", "verify": "cargo test -p but-authz invariant_build_gates", "satisfied": true, "maps_to_ac": null },
    { "id": "AC-3", "kind": "ac", "statement": "GIVEN either canary command returns a non-zero exit code, WHEN the implementer diagnoses the failure and applies a minimal fix, THEN both commands are re-run and exit 0.", "verify": "cargo test -p but --features but-2 governed_loop && cargo test -p but-authz invariant_build_gates", "satisfied": true, "maps_to_ac": null },
    { "id": "TC-1", "kind": "tc", "statement": "`cargo test -p but --features but-2 governed_loop` returns exit code 0 and the 5 T-LOOP-006 canary tests are listed as passing.", "maps_to_ac": "AC-1", "verify": "cargo test -p but --features but-2 governed_loop", "status": "true" },
    { "id": "TC-2", "kind": "tc", "statement": "`cargo test -p but-authz invariant_build_gates` returns exit code 0.", "maps_to_ac": "AC-2", "verify": "cargo test -p but-authz invariant_build_gates", "status": "true" },
    { "id": "TC-3", "kind": "tc", "statement": "If AC-1 or AC-2 is initially RED, the final re-run after diagnosis/fix returns exit code 0.", "maps_to_ac": "AC-3", "verify": "cargo test -p but --features but-2 governed_loop && cargo test -p but-authz invariant_build_gates", "status": "true" }
  ]
}
-->
