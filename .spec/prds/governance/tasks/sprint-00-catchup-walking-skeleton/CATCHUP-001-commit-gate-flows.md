# CATCHUP-001: Verify and fix commit-gate flows (UC-AUTHZ-01/02/04, UC-GATES-01)

> Sprint: [sprint-00-catchup-walking-skeleton](./SPRINT.md)
> Agent: rust-implementer · Reviewer: rust-reviewer
> Estimate: 120 min · Type: verify/fix · Status: ✅ Completed
> Proposed By: rust-planner · Reviewer: rust-reviewer · Cycle: 1
> Updated: 2026-06-23T18:40:00Z

## Background

Sprints 01a/01b shipped commit-gate AUTHZ + commit gate flows as "Done" on agent-narrated
`cargo test` counts. This task replays them FOR REAL — verdict is the cargo exit code, not a
summary. Ground truth (already probed): parts 1 & 2 are GREEN; part 3 is RED on 2 snapshot-drift
failures caused by Sprint 08 STEER enrichment landing on HEAD.

## Critical Constraints

- MUST NOT narrate green; the verdict is the cargo exit code of the locked run_cmd, not a human summary.
- MUST use the real `but` binary surface driven by assert_cmd/snapbox via `cargo test -p but --features but-2 commit_gate`; no mocks.
- MUST confirm the new snapbox snapshot shape matches the intended STEER JSON contract (`steer_cli_serde_*` tests pass) before refreshing any snapshot.
- NEVER seed or inject repository configuration; config must be committed through real git operations as in the `governed_repo` fixture.
- STRICTLY modify only snapbox snapshot files for the snapshot drift; do not change production Rust source to accommodate test output.

## Specification

**Objective:** Replay the locked commit-gate verification commands and eliminate any RED results so all three commands exit 0.

**Success state:** `cargo test -p but-authz`, `cargo test -p but-api commit_gate`, and `cargo test -p but --features but-2 commit_gate` all return exit code 0; any snapshot drift is refreshed only after the STEER contract is confirmed.

## Acceptance Criteria

- [x] AC-1: GIVEN the workspace is on the current HEAD and the but-authz crate compiles, WHEN `cargo test -p but-authz` is executed, THEN the process exits 0 and all but-authz tests pass.
  - Verify: `cargo test -p but-authz`
- [x] AC-2: GIVEN the workspace is on the current HEAD, WHEN `cargo test -p but-api commit_gate` is executed, THEN the process exits 0 and all 12 commit_gate tests pass.
  - Verify: `cargo test -p but-api commit_gate`
- [x] AC-3: GIVEN the current HEAD includes Sprint 08 STEER enrichment changes, WHEN `cargo test -p but --features but-2 commit_gate` is executed before any snapshot refresh, THEN exactly two tests fail with snapbox snapshot mismatches: `commit_gate_denies_new_branch_without_contents_write` and `commit_gate_denies_protected_branch`; both still emit the correct `perm.denied` / `branch.protected` verdicts and the actual JSON contains the new STEER fields (`remediation_hint`, `class`, `held_permissions`, `authorized_actions`).
  - Verify: `cargo test -p but --features but-2 commit_gate`
- [x] AC-4: GIVEN the `steer_cli_serde_*` tests pass, confirming the new JSON shape is the intended STEER contract, WHEN `SNAPSHOTS=overwrite cargo test -p but --features but-2 command::commit_gate` is run, THEN the suite exits 0 and only the snapbox snapshot files for the two named tests are updated to include the STEER enrichment fields.
  - Verify: `SNAPSHOTS=overwrite cargo test -p but --features but-2 command::commit_gate`
- [x] AC-5: GIVEN snapshots have been refreshed to the intended STEER contract, WHEN `cargo test -p but --features but-2 commit_gate` is rerun, THEN the suite exits 0 with no remaining snapshot drift.
  - Verify: `cargo test -p but --features but-2 commit_gate`

## Test Criteria

| #    | Boolean Statement                                                                                                                                                                                                                          | Maps To | Verify                                                                        | Status             |
| ---- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------- | ----------------------------------------------------------------------------- | ------------------ |
| TC-1 | `cargo test -p but-authz` returns exit code 0.                                                                                                                                                                                             | AC-1    | `cargo test -p but-authz`                                                     | [x] TRUE [ ] FALSE |
| TC-2 | `cargo test -p but-api commit_gate` returns exit code 0.                                                                                                                                                                                   | AC-2    | `cargo test -p but-api commit_gate`                                           | [x] TRUE [ ] FALSE |
| TC-3 | Before fix, `cargo test -p but --features but-2 commit_gate` exits non-zero and the failure output names exactly `commit_gate_denies_new_branch_without_contents_write` and `commit_gate_denies_protected_branch` with snapbox mismatches. | AC-3    | `cargo test -p but --features but-2 commit_gate`                              | [x] TRUE [ ] FALSE |
| TC-4 | `SNAPSHOTS=overwrite cargo test -p but --features but-2 command::commit_gate` returns exit code 0 and only `.snap` files are modified.                                                                                                     | AC-4    | `SNAPSHOTS=overwrite cargo test -p but --features but-2 command::commit_gate` | [x] TRUE [ ] FALSE |
| TC-5 | After snapshot refresh, `cargo test -p but --features but-2 commit_gate` returns exit code 0.                                                                                                                                              | AC-5    | `cargo test -p but --features but-2 commit_gate`                              | [x] TRUE [ ] FALSE |

## Reading List

- `crates/but-api/tests/commit_gate.rs` — the 12-test commit-gate suite (governed_repo fixture commits config via real git)
- `crates/but/tests/but/command/commit_gate.rs` — the CLI-surface commit-gate tests (2 stale snapshots)
- `.spec/prds/governance/tasks/sprint-00-catchup-walking-skeleton/human-flows.json` — locked flows
- `.spec/prds/governance/tasks/sprint-00-catchup-walking-skeleton/SPRINT.md` — sprint gate

## Guardrails

**WRITE-ALLOWED:**

- `crates/but/tests/but/command/commit_gate.rs`
- `crates/but/tests/but/command/snapshots/*.snap`
- `crates/but/tests/snapshots/*commit_gate*.snap`

**WRITE-PROHIBITED:**

- `crates/but/src/**`, `crates/but-api/src/**`, `crates/but-authz/src/**`
- `crates/but-api/tests/commit_gate.rs`
- `.spec/prds/**`, `.gitbutler/**`, `tools/**`

## Verification Gates

| Command                                                                       | Expected Exit |
| ----------------------------------------------------------------------------- | ------------- |
| `cargo test -p but-authz`                                                     | 0             |
| `cargo test -p but-api commit_gate`                                           | 0             |
| `cargo test -p but --features but-2 commit_gate`                              | 0             |
| `SNAPSHOTS=overwrite cargo test -p but --features but-2 command::commit_gate` | 0             |

## Agent Assignment

**Implementer:** rust-implementer — verify/fix Rust CLI/library flows; snapshot refresh.
**Reviewer:** rust-reviewer — confirm the snapshot refresh reflects the intended STEER contract (not masking a regression).

## Dependencies

- **Depends on:** none
- **Blocks:** CATCHUP-004 (if cold-boot lint flags these tests, they reopen as fix tasks)

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "requirements": [
    { "id": "AC-1", "kind": "ac", "statement": "GIVEN the workspace is on the current HEAD and the but-authz crate compiles, WHEN `cargo test -p but-authz` is executed, THEN the process exits 0 and all but-authz tests pass.", "verify": "cargo test -p but-authz", "satisfied": true, "maps_to_ac": null },
    { "id": "AC-2", "kind": "ac", "statement": "GIVEN the workspace is on the current HEAD, WHEN `cargo test -p but-api commit_gate` is executed, THEN the process exits 0 and all 12 commit_gate tests pass.", "verify": "cargo test -p but-api commit_gate", "satisfied": true, "maps_to_ac": null },
    { "id": "AC-3", "kind": "ac", "statement": "GIVEN the current HEAD includes Sprint 08 STEER enrichment changes, WHEN `cargo test -p but --features but-2 commit_gate` is executed before any snapshot refresh, THEN exactly two tests fail with snapbox snapshot mismatches: `commit_gate_denies_new_branch_without_contents_write` and `commit_gate_denies_protected_branch`; both still emit the correct `perm.denied` / `branch.protected` verdicts and the actual JSON contains the new STEER fields (`remediation_hint`, `class`, `held_permissions`, `authorized_actions`).", "verify": "cargo test -p but --features but-2 commit_gate", "satisfied": true, "maps_to_ac": null },
    { "id": "AC-4", "kind": "ac", "statement": "GIVEN the steer_cli_serde_* tests pass, confirming the new JSON shape is the intended STEER contract, WHEN `SNAPSHOTS=overwrite cargo test -p but --features but-2 command::commit_gate` is run, THEN the suite exits 0 and only the snapbox snapshot files for the two named tests are updated to include the STEER enrichment fields.", "verify": "SNAPSHOTS=overwrite cargo test -p but --features but-2 command::commit_gate", "satisfied": true, "maps_to_ac": null },
    { "id": "AC-5", "kind": "ac", "statement": "GIVEN snapshots have been refreshed to the intended STEER contract, WHEN `cargo test -p but --features but-2 commit_gate` is rerun, THEN the suite exits 0 with no remaining snapshot drift.", "verify": "cargo test -p but --features but-2 commit_gate", "satisfied": true, "maps_to_ac": null },
    { "id": "TC-1", "kind": "tc", "statement": "`cargo test -p but-authz` returns exit code 0.", "maps_to_ac": "AC-1", "verify": "cargo test -p but-authz", "status": "true" },
    { "id": "TC-2", "kind": "tc", "statement": "`cargo test -p but-api commit_gate` returns exit code 0.", "maps_to_ac": "AC-2", "verify": "cargo test -p but-api commit_gate", "status": "true" },
    { "id": "TC-3", "kind": "tc", "statement": "Before fix, `cargo test -p but --features but-2 commit_gate` exits non-zero and the failure output names exactly `commit_gate_denies_new_branch_without_contents_write` and `commit_gate_denies_protected_branch` with snapbox mismatches.", "maps_to_ac": "AC-3", "verify": "cargo test -p but --features but-2 commit_gate", "status": "true" },
    { "id": "TC-4", "kind": "tc", "statement": "`SNAPSHOTS=overwrite cargo test -p but --features but-2 command::commit_gate` returns exit code 0 and only `.snap` files are modified.", "maps_to_ac": "AC-4", "verify": "SNAPSHOTS=overwrite cargo test -p but --features but-2 command::commit_gate", "status": "true" },
    { "id": "TC-5", "kind": "tc", "statement": "After snapshot refresh, `cargo test -p but --features but-2 commit_gate` returns exit code 0.", "maps_to_ac": "AC-5", "verify": "cargo test -p but --features but-2 commit_gate", "status": "true" }
  ]
}
-->
