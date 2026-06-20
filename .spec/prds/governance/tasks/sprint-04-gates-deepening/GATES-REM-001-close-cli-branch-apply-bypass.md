# GATES-REM-001: Close CLI `but branch apply` bypass through governed public seam

## What this does

The red-hat review for Sprint 04 found that the `but branch apply` CLI command in `crates/but/src/command/branch/apply.rs` calls `but_api::branch::apply_with_perm` directly after acquiring `exclusive_worktree_access`, bypassing the public `but_api::branch::apply` seam that GATES-007 gates. GATES-007 scoped the commit gate to the public but-api entry points (`crates/but-api/src/branch.rs:643` and `crates/but-api/src/branch.rs:939`) and did not close the CLI path that goes straight to the `_with_perm` helper. This task closes that bypass so a read-only principal cannot mutate refs by invoking the CLI command.

## Why

Sprint 04 · PRD UC-GATES-01 (mechanism-agnostic coverage, AC-5/AC-9) · capabilities CAP-AUTHZ-01, CAP-CONFIG-01. A gate wired only at the library public seam is bypassed by any CLI, test, or legacy caller that reaches around it to the permission helper. The `but` CLI is a real user-facing entry point, so the mechanism-agnostic proof is incomplete until it is also governed.

## How to verify

PRIMARY **AC-1** — `cargo test -p but branch_apply_readonly_denied` (integration, real but-api + CLI + real git). Full gate set in the spec below.

## Scope

- `crates/but/src/command/branch/apply.rs` (MODIFY) — either route the CLI through the public `but_api::branch::apply` function, or add the same `commit::gate::enforce_commit_gate_for_target` check before the CLI acquires `exclusive_worktree_access`.
- `crates/but/tests/but/command/branch_apply.rs` (MODIFY or CREATE) — add a snapbox/integration test that proves `BUT_AGENT_HANDLE=ro but branch apply <feat>` on a governed repo is denied `perm.denied` before any ref/oplog mutation.

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: GATES-REM-001 - Close CLI `but branch apply` bypass through governed public seam
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:      (120 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-GATES-01
CAPABILITIES: CAP-AUTHZ-01, CAP-CONFIG-01

RUNTIME_COMMANDS:
  test:  cargo test -p but branch_apply_readonly_denied
  check: cargo check -p but --all-targets && cargo check -p but-api --all-targets
  lint:  cargo clippy -p but --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
The `but branch apply` CLI command is no longer an ungated path onto a governed workspace. A read-only principal (`contents:read`) is denied `perm.denied` naming `contents:write` when targeting a governed workspace; a contents:write principal is permitted by the gate. The denial happens before the CLI acquires the worktree lock and before any ref/oplog mutation. The CLI test uses the same fixture shape as GATES-007 (`gated_apply_repo`) and asserts against the structured error code.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST enforce the same commit-gate decision at the CLI entry point that GATES-007 enforces at the public but-api `apply` seam. The gate must read governance from the workspace target ref, fail closed on unknown/unset BUT_AGENT_HANDLE, and deny `perm.denied` for contents:read principals.
- [MUST] MUST place the gate BEFORE the CLI acquires `exclusive_worktree_access` and before any ref/oplog mutation. The existing but-api public `apply` is the precedent (gate at :643, guard at :647).
- [MUST] MUST preserve the CLI's existing happy-path behavior for ungoverned repos (no committed `.gitbutler/*.toml`, no default target) — the gate must be skipped when `target_ref_or_err()` returns `DefaultTargetNotFound`, matching the plain-commit gate and GATES-007's no-target behavior.
- [NEVER] NEVER add the gate inside `apply_with_perm` or any other function that already runs under the worktree lock. NEVER overload the repo lock as the authorization carrier.
- [STRICTLY] STRICTLY surface the denial through the same structured error contract as GATES-007 (`branch.protected`, `perm.denied`, `config.invalid`).

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: `but branch apply` by a contents:read-only principal on a governed workspace is denied `perm.denied` before any ref mutation.
- [ ] AC-2: `but branch apply` by a contents:write principal on a governed workspace proceeds past the gate.
- [ ] AC-3: `but branch apply` on an ungoverned/no-target workspace is still permitted.
- [ ] All verification gates pass; only write_allowed files modified.

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: CLI branch apply denies read-only principal on governed workspace [PRIMARY]
  GIVEN: fixture `gated_apply_repo` (workspace target ref main protected via committed gates.toml; permissions: dev=contents:write, ro=contents:read; feature branch with applicable change; default target set to main), BUT_AGENT_HANDLE=ro
  WHEN:  the CLI `but branch apply <feat>` is invoked
  THEN:  the command exits non-zero with error.code=="perm.denied" naming `contents:write`, and the workspace target ref sha remains unchanged (no refs or oplog were mutated)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but CLI + real but-api + real git
  VERIFY: cargo test -p but branch_apply_readonly_denied
  SCENARIO: NEGATIVE_CONTROL would fail if the CLI bypasses the public seam and mutates refs ungated, if the denial is not `perm.denied`, or if the error is raised after a ref mutation.

AC-2: CLI branch apply permits contents:write principal on governed workspace
  GIVEN: fixture `gated_apply_repo`, BUT_AGENT_HANDLE=dev
  WHEN:  the CLI `but branch apply <feat>` is invoked
  THEN:  the gate returns Ok, the command is permitted past the authorization check, and any subsequent error is the operation's own (not a governance Denial)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but CLI + real but-api + real git
  VERIFY: cargo test -p but branch_apply_readonly_denied
  SCENARIO: NEGATIVE_CONTROL would fail if a contents:write principal is wrongly denied or if the gate is a degenerate always-Err stub.

AC-3: CLI branch apply on ungoverned/no-target workspace remains permitted
  GIVEN: fixture `apply_repo_no_target` (no committed `.gitbutler/*.toml`, no configured default target), BUT_AGENT_HANDLE=ro
  WHEN:  the CLI `but branch apply <feat>` is invoked
  THEN:  the gate is skipped because the workspace has no target ref governance, and the command is permitted (or fails only for non-governance reasons)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but CLI + real but-api + real git
  VERIFY: cargo test -p but branch_apply_no_target_ungoverned
  SCENARIO: NEGATIVE_CONTROL would fail if a no-target repo is hard-errored by `target_ref_or_err()?` propagation or if the gate is not skipped.

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, error): `BUT_AGENT_HANDLE=ro but branch apply <feat>` on governed workspace exits non-zero with `perm.denied` naming `contents:write` and target ref unchanged.
    VERIFY: cargo test -p but branch_apply_readonly_denied
- TC-2 (-> AC-2, happy_path): `BUT_AGENT_HANDLE=dev but branch apply <feat>` on governed workspace passes the gate (no governance Denial).
    VERIFY: cargo test -p but branch_apply_readonly_denied
- TC-3 (-> AC-3, edge): `BUT_AGENT_HANDLE=ro but branch apply <feat>` on no-target workspace is permitted (gate skipped).
    VERIFY: cargo test -p but branch_apply_no_target_ungoverned
- TC-4 (-> AC-1, structural): the CLI gate call appears before `exclusive_worktree_access` in `crates/but/src/command/branch/apply.rs`.
    VERIFY: grep -nE 'enforce_commit_gate_for_target|exclusive_worktree_access' crates/but/src/command/branch/apply.rs | head -5

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01, CAP-CONFIG-01
provides: the CLI `but branch apply` path is governed by the same target-ref-pinned commit gate as the but-api public seam; no read-only principal can mutate refs through the CLI.
consumes: GATES-007's public `apply` gate pattern (`crates/but-api/src/branch.rs:643`), the commit gate decision helper (`crate::commit::create::gate`), `ctx.project_meta()?.target_ref_or_err()` matching, and the `gated_apply_repo` fixture shape.
boundary_contracts:
  - CAP-AUTHZ-01: CLI entry point resolves BUT_AGENT_HANDLE and authorizes contents:write before the worktree lock, mirroring the but-api public seam.
  - CAP-CONFIG-01: governance is read only from the workspace target ref; no-target workspaces are ungoverned and skip the gate.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but/src/command/branch/apply.rs (MODIFY) — add the commit gate check before the worktree lock, or route through the public `but_api::branch::apply` function.
  - crates/but/tests/but/command/branch_apply.rs (MODIFY or CREATE) — add CLI integration/snapbox tests for the governed denial and ungoverned permit cases.
writeProhibited:
  - crates/but-api/src/branch.rs — the public seam is already gated by GATES-007; do not re-gate it.
  - crates/but-api/src/commit/gate.rs — reuse the existing decision helper; do not modify gate logic.
  - Any file not listed in write_allowed.

--------------------------------------------------------------------------------
OUT OF SCOPE
--------------------------------------------------------------------------------
  - Re-gating the but-api public `apply` / `apply_branch_integration` seams (GATES-007 owns those).
  - Gating the merge gate or review-requirement evaluator (GATES-006/008/AUTHZ-004 own those).
  - Adding new gitbutler-* coupling.
  - Testing the forgeable direct-DB-write or raw-git bypass paths (accepted leaks R6/R1).

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but/src/command/branch/apply.rs (full)
   Focus: the CLI entry point to modify; identify where it acquires `exclusive_worktree_access` and calls `apply_with_perm`.
2. crates/but-api/src/branch.rs (643-659, 939-957)
   Focus: the gated public seam pattern — gate before guard, target_ref_or_err matching, `commit::create::gate` usage.
3. crates/but-api/src/commit/create.rs (25-49)
   Focus: the gate-before-guard precedent used by GATES-007.
4. crates/but-api/tests/commit_gate.rs (full)
   Focus: the `gated_apply_repo` fixture shape and the target-ref-pinned gate assertions to mirror in the CLI test.
5. .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-007-mechanism-agnostic-commit-gate.md
   Focus: the public seam gating scope and constraints this task extends to the CLI.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- CLI read-only denial test passes: `cargo test -p but branch_apply_readonly_denied` -> Exit 0; perm.denied before ref mutation.
- CLI no-target permit test passes: `cargo test -p but branch_apply_no_target_ungoverned` -> Exit 0; no hard error on DefaultTargetNotFound.
- Gate-before-guard placement in CLI: `grep -nE 'enforce_commit_gate_for_target|exclusive_worktree_access' crates/but/src/command/branch/apply.rs` shows gate call before guard.
- Crate compiles incl. tests: `cargo check -p but --all-targets` -> Exit 0.
- Clippy clean: `cargo clippy -p but --all-targets` -> Exit 0.

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: Reuse the existing GATES-007 public-seam gate pattern at the CLI layer: obtain the repo handle, match `target_ref_or_err()`, and call `enforce_commit_gate_for_target` with `CommitGateTarget::config_only(target_ref)` before acquiring the worktree lock. The gate decision, opt-in discriminator, and error classification are identical to GATES-007.
pattern_source: crates/but-api/src/branch.rs:643-647 (public `apply` gate before guard) + crates/but-api/src/commit/create.rs:35-38 (gate-before-guard precedent).
anti_pattern: Gating inside `apply_with_perm` (lock-ordering violation); adding a parallel gate implementation instead of reusing the existing commit gate helper; failing to match `target_ref_or_err()` so no-target repos hard-error.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — Extends the GATES-007 mechanism-agnostic commit gate to the `but branch apply` CLI entry point, ensuring the same target-ref-pinned authorization runs before the worktree lock and before any ref mutation, with CLI integration tests proving the denial and permit cases.
reviewer: rust-reviewer
coding_standards: crates/AGENTS.md, crates/WORKSPACE_MODEL.md, crates/but/AGENTS.md, crates/but-api/src/branch.rs

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: GATES-007 (the public seam gate pattern), GATES-001 (commit gate decision helper), AUTHZ-002/003 (principal resolution and authorization primitives)
Blocks:     Sprint 05, Sprint 06b (via Sprint 04 completion)
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "GATES-REM-001",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "requirements": [
    { "id": "AC-1", "type": "acceptance_criterion", "primary": true, "description": "CLI branch apply denies read-only principal on governed workspace with perm.denied naming contents:write before ref mutation", "verify": "cargo test -p but branch_apply_readonly_denied", "maps_to_ac": null },
    { "id": "AC-2", "type": "acceptance_criterion", "description": "CLI branch apply permits contents:write principal on governed workspace", "verify": "cargo test -p but branch_apply_readonly_denied", "maps_to_ac": null },
    { "id": "AC-3", "type": "acceptance_criterion", "description": "CLI branch apply on ungoverned/no-target workspace remains permitted (gate skipped)", "verify": "cargo test -p but branch_apply_no_target_ungoverned", "maps_to_ac": null },
    { "id": "TC-1", "type": "test_criterion", "description": "BUT_AGENT_HANDLE=ro but branch apply <feat> on governed workspace exits non-zero with perm.denied naming contents:write and target ref unchanged", "verify": "cargo test -p but branch_apply_readonly_denied", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "BUT_AGENT_HANDLE=dev but branch apply <feat> on governed workspace passes the gate (no governance Denial)", "verify": "cargo test -p but branch_apply_readonly_denied", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "BUT_AGENT_HANDLE=ro but branch apply <feat> on no-target workspace is permitted (gate skipped)", "verify": "cargo test -p but branch_apply_no_target_ungoverned", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "the CLI gate call appears before exclusive_worktree_access in crates/but/src/command/branch/apply.rs", "verify": "grep -nE 'enforce_commit_gate_for_target|exclusive_worktree_access' crates/but/src/command/branch/apply.rs | head -5", "maps_to_ac": "AC-1" }
  ]
}
-->
</content>
</invoke>
