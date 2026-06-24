---
task: MGMT-REM-003
sprint: sprint-09-governance-remediation-lpr-mgmt-hardening
sequence: 11
agent: rust-implementer
estimate_minutes: 90
status: pending
proposed_by: rust-planner
type: REMEDIATION
generated_at: 2026-06-23T13:30:00Z
generated_by: kb-sprint-tasks-plan
---

# MGMT-REM-003: Add but group delete CLI verb

**Agent:** `rust-implementer` (90 min)
**Proposed By:** `rust-planner`
**Type:** REMEDIATION
**Status:** pending
**Depends On:** —
**Blocks:** LPR-REM-008

## Background

**Problem:** The CLI Subcommands enum at crates/but/src/args/group.rs:9-43 has no Delete variant, so there is no CLI path to group_delete_with_repo.

**Why it matters:** 01-scope.md:35 lists group deletion as in-scope; governance administrators cannot remove stale groups without dropping to direct API calls.

**Current state:** group_delete_with_repo exists at crates/but-api/src/legacy/governance.rs:1742 and is admin-gated at :1747, but unreachable from but.

**Desired state:** but group delete <name> exists, enforces admin gating, and the existing negative test is replaced with a positive one.

## Critical Constraints

- MUST add a Delete variant to the group Subcommands enum.
- MUST expose the required arguments (group name and repository context).
- MUST call group_delete_with_repo and surface the admin-gated error to non-admins.
- MUST replace the negative test crates/but/tests/but/command/group.rs:155-167 group_no_delete_cli_verb_surface with a positive implementation test.
- NEVER bypass the admin gate at governance.rs:1747.

## Specification

**Objective:** Expose group deletion through the but CLI.

**Success state:** cargo test -p but --features but-2 group::delete passes and the new positive test verifies admin groups can be deleted while non-admins are rejected.

## Acceptance Criteria

### AC-1

- **GIVEN:** An admin user and an existing governance group
- **WHEN:** The admin runs but group delete <name>
- **THEN:** The group is deleted and the command exits 0
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** but-cli
- **Flow Ref:** sprint-09-step-10

**Scenario:**

- Tier: visible
- Fixtures: seeded_governance_group
- Cases:
  - Actor: admin, Steps: but group delete my-group
  - Must observe: group my-group no longer exists; exit code 0
  - Must not observe: unknown subcommand: delete; admin required
- Negative control — would fail if: delete subcommand not registered
- Evidence: stdout (capture required: true)

### AC-2

- **GIVEN:** A non-admin user and an existing governance group
- **WHEN:** The user runs but group delete <name>
- **THEN:** The command fails with an admin-gated error and the group remains
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** but-cli
- **Flow Ref:** sprint-09-step-10

**Scenario:**

- Tier: visible
- Fixtures: seeded_governance_group
- Cases:
  - Actor: non_admin, Steps: but group delete my-group
  - Must observe: non-zero exit; admin/permission error
  - Must not observe: group deleted; exit code 0
- Negative control — would fail if: admin gate bypassed
- Evidence: stderr (capture required: true)

### AC-3

- **GIVEN:** The existing negative test group_no_delete_cli_verb_surface
- **WHEN:** It is replaced with a positive test for but group delete
- **THEN:** cargo test -p but --features but-2 command::group passes
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** but-cli
- **Flow Ref:** sprint-09-step-10

**Scenario:**

- Tier: visible
- Fixtures: (none)
- Cases:
  - Actor: test*runner, Steps: rewrite group_no_delete_cli_verb_surface as group_delete*... positive test; cargo test -p but --features but-2 command::group
  - Must observe: test passes; positive delete scenario covered
  - Must not observe: group_no_delete_cli_verb_surface still asserting missing verb
- Negative control — would fail if: test still asserts no delete verb
- Evidence: test_output (capture required: true)

### AC-4

- **GIVEN:** The but group --help output
- **WHEN:** The user runs but group --help
- **THEN:** delete is listed as a subcommand with its required arguments
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** but-cli
- **Flow Ref:** sprint-09-step-10

**Scenario:**

- Tier: visible
- Fixtures: (none)
- Cases:
  - Actor: user, Steps: but group --help
  - Must observe: delete subcommand listed; arguments documented
  - Must not observe: No such subcommand
- Negative control — would fail if: subcommand not added
- Evidence: stdout (capture required: true)

## Test Criteria

| ID   | Statement                                                     | Maps to AC |
| ---- | ------------------------------------------------------------- | ---------- |
| TC-1 | Admin can delete a governance group via CLI                   | AC-1       |
| TC-2 | Non-admin deletion attempt is rejected with admin-gated error | AC-2       |
| TC-3 | Negative CLI test is replaced with positive delete coverage   | AC-3       |
| TC-4 | CLI help advertises group delete                              | AC-4       |

## Guardrails

**WRITE-ALLOWED:**

- crates/but/src/args/group.rs
- crates/but/src/command/legacy/governance.rs
- crates/but/tests/but/command/group.rs

**WRITE-PROHIBITED:**

- crates/but-api/src/legacy/governance.rs
- crates/but-authz/src/\*

## Verification Gates

- **Command:** `cargo test -p but --features but-2 command::group`
- **Expected outcome:** all group tests pass, including new delete positive test
- **Command:** `cargo check -p but --all-targets`
- **Expected outcome:** clean compilation

## Reading List

- `crates/but/src/args/group.rs` lines 9-43 — Subcommands enum without Delete
- `crates/but-api/src/legacy/governance.rs` lines 1742-1747 — group_delete_with_repo backend and admin gate
- `crates/but/tests/but/command/group.rs` lines 155-167 — negative test to replace

## Dependencies

- **Depends On:** —
- **Blocks:** LPR-REM-008

## Design

- **Pattern:** Add CLI enum variant and dispatch to existing backend, preserving admin gate.
- **Anti-pattern:** Reimplement group deletion or skip the admin check.
- **References:** —

## Coding Standards

- crates/AGENTS.md

<!-- REQUIREMENT-CONTRACT v1 -->
<!-- {"requirements":[{"id":"AC-1","kind":"acceptance","scenario_ref":"AC-1"},{"id":"AC-2","kind":"acceptance","scenario_ref":"AC-2"},{"id":"AC-3","kind":"acceptance","scenario_ref":"AC-3"},{"id":"AC-4","kind":"acceptance","scenario_ref":"AC-4"},{"id":"TC-1","kind":"test","maps_to_ac":"AC-1"},{"id":"TC-2","kind":"test","maps_to_ac":"AC-2"},{"id":"TC-3","kind":"test","maps_to_ac":"AC-3"},{"id":"TC-4","kind":"test","maps_to_ac":"AC-4"}]} -->
