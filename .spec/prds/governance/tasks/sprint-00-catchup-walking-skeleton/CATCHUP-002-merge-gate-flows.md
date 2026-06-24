# CATCHUP-002: Verify and fix merge-gate flows (UC-GATES-02)

> Sprint: [sprint-00-catchup-walking-skeleton](./SPRINT.md)
> Agent: rust-implementer · Reviewer: rust-reviewer
> Estimate: 60 min · Type: verify/fix · Status: ✅ Completed
> Proposed By: rust-planner · Reviewer: rust-reviewer · Cycle: 1
> Updated: 2026-06-23T18:40:00Z

## Background

The merge-gate flows (UC-GATES-02) shipped as part of Sprint 01b. This task replays the locked
`cargo test -p but-api merge_gate` command FOR REAL. Ground truth (already probed): GREEN —
14/14 tests pass. The task confirms this holds and fixes any regression if present.

## Critical Constraints

- MUST NOT narrate green; the verdict is the cargo exit code of the locked run_cmd.
- MUST use the real but-api surface via `cargo test -p but-api merge_gate`; no mocks.
- MUST diagnose any RED failure and fix it until the full suite exits 0.
- NEVER seed or inject merge-gate state; tests must construct their own fixtures via real git operations.

## Specification

**Objective:** Replay the locked merge-gate verification command and ensure it returns exit code 0.

**Success state:** `cargo test -p but-api merge_gate` returns exit code 0 with all 14 merge_gate tests passing.

## Acceptance Criteria

- [x] AC-1: GIVEN the workspace is on the current HEAD and but-api compiles, WHEN `cargo test -p but-api merge_gate` is executed, THEN the process exits 0 and all 14 merge_gate tests pass.
  - Verify: `cargo test -p but-api merge_gate`
- [x] AC-2: GIVEN the merge-gate suite returns a non-zero exit code, WHEN the implementer diagnoses the failure and applies a minimal fix, THEN the suite is re-run and exits 0.
  - Verify: `cargo test -p but-api merge_gate`

## Test Criteria

| #    | Boolean Statement                                                                   | Maps To | Verify                             | Status             |
| ---- | ----------------------------------------------------------------------------------- | ------- | ---------------------------------- | ------------------ |
| TC-1 | `cargo test -p but-api merge_gate` returns exit code 0.                             | AC-1    | `cargo test -p but-api merge_gate` | [x] TRUE [ ] FALSE |
| TC-2 | If AC-1 is initially RED, the final re-run after diagnosis/fix returns exit code 0. | AC-2    | `cargo test -p but-api merge_gate` | [x] TRUE [ ] FALSE |

## Reading List

- `crates/but-api/tests/merge_gate.rs` — the 14-test merge-gate suite
- `.spec/prds/governance/tasks/sprint-00-catchup-walking-skeleton/human-flows.json` — locked flows
- `.spec/prds/governance/tasks/sprint-00-catchup-walking-skeleton/SPRINT.md` — sprint gate

## Guardrails

**WRITE-ALLOWED:**

- `crates/but-api/tests/merge_gate.rs`
- `crates/but-api/tests/snapshots/merge_gate/*.snap`
- `crates/but-api/tests/snapshots/*merge_gate*.snap`

**WRITE-PROHIBITED:**

- `crates/but-api/src/**`, `crates/but/src/**`, `crates/but-authz/src/**`
- `tools/**`, `.spec/prds/**`

## Verification Gates

| Command                            | Expected Exit |
| ---------------------------------- | ------------- |
| `cargo test -p but-api merge_gate` | 0             |

## Agent Assignment

**Implementer:** rust-implementer — verify/fix Rust merge-gate flows.
**Reviewer:** rust-reviewer — confirm any fix is minimal and behavior-preserving.

## Dependencies

- **Depends on:** none
- **Blocks:** CATCHUP-004 (if cold-boot lint flags these tests, they reopen as fix tasks)

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "requirements": [
    { "id": "AC-1", "kind": "ac", "statement": "GIVEN the workspace is on the current HEAD and but-api compiles, WHEN `cargo test -p but-api merge_gate` is executed, THEN the process exits 0 and all 14 merge_gate tests pass.", "verify": "cargo test -p but-api merge_gate", "satisfied": true, "maps_to_ac": null },
    { "id": "AC-2", "kind": "ac", "statement": "GIVEN the merge-gate suite returns a non-zero exit code, WHEN the implementer diagnoses the failure and applies a minimal fix, THEN the suite is re-run and exits 0.", "verify": "cargo test -p but-api merge_gate", "satisfied": true, "maps_to_ac": null },
    { "id": "TC-1", "kind": "tc", "statement": "`cargo test -p but-api merge_gate` returns exit code 0.", "maps_to_ac": "AC-1", "verify": "cargo test -p but-api merge_gate", "status": "true" },
    { "id": "TC-2", "kind": "tc", "statement": "If AC-1 is initially RED, the final re-run after diagnosis/fix returns exit code 0.", "maps_to_ac": "AC-2", "verify": "cargo test -p but-api merge_gate", "status": "true" }
  ]
}
-->
