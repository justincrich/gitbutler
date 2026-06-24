---
task: LPR-REM-002
sprint: sprint-09-governance-remediation-lpr-mgmt-hardening
sequence: 11
agent: rust-implementer
estimate_minutes: 150
status: pending
proposed_by: rust-planner
type: REMEDIATION
generated_at: 2026-06-23T13:30:00Z
generated_by: kb-sprint-tasks-plan
---

# LPR-REM-002: Add but review comments and but review resolve CLI verbs

**Agent:** `rust-implementer` (150 min)
**Proposed By:** `rust-planner`
**Type:** REMEDIATION
**Status:** pending
**Depends On:** LPR-REM-001
**Blocks:** LPR-REM-005, LPR-REM-008

## Background

**Problem:** The CLI Subcommands enum in crates/but/src/args/forge.rs:28-184 has no Comments or Resolve variant, even though real backends exist for list_comments and resolve_thread.

**Why it matters:** PRD Sprint 07 Gate Steps 4 and 5 require users to list review comment threads and resolve them; missing verbs force users to bypass the CLI or leave comments orphaned.

**Current state:** list_comments and resolve_thread are implemented in but-api/legacy/forge.rs but unreachable from the but CLI.

**Desired state:** but review comments <branch> lists comments and but review resolve <branch> --thread <id> resolves a thread, enforcing the R22 resolver-identity check at forge.rs:983-990.

## Critical Constraints

- MUST add Comments and Resolve variants without breaking existing but review subcommands.
- MUST reuse list_comments at forge.rs:923-936 and resolve_thread at forge.rs:956-993.
- MUST expose --thread for Resolve and surface output in a stable, parseable format.
- NEVER allow a non-resolver actor to resolve a thread; enforce R22 via existing backend logic.
- STRICTLY depends on LPR-REM-001 so comment creation is available to list and resolve.

## Specification

**Objective:** Expose list_comments and resolve_thread through new but review subcommands.

**Success state:** cargo test -p but --features but-2 review::comments and review::resolve pass, and end-to-end review flows can create, list, and resolve comments from the CLI.

## Acceptance Criteria

### AC-1

- **GIVEN:** A governed branch with one existing review comment thread
- **WHEN:** The user runs but review comments <branch>
- **THEN:** The CLI prints the comment thread including author, file, line, message, and resolution state
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** but-cli
- **Flow Ref:** sprint-09-step-2

**Scenario:**

- Tier: visible
- Fixtures: seeded_governed_repo_with_comment
- Cases:
  - Actor: user, Steps: but review comments needs-fix
  - Must observe: file=src/lib.rs; line=42; thread=t-1; message="needs change"; resolved=false
  - Must not observe: unknown subcommand: comments; no comments
- Negative control — would fail if: subcommand not registered; list_comments not wired
- Evidence: stdout (capture required: true)

### AC-2

- **GIVEN:** A governed branch with one unresolved comment thread created by the current actor
- **WHEN:** The user runs but review resolve <branch> --thread t-1
- **THEN:** The thread is marked resolved and the command exits 0
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** but-cli
- **Flow Ref:** sprint-09-step-2

**Scenario:**

- Tier: visible
- Fixtures: seeded_governed_repo_with_comment
- Cases:
  - Actor: reviewer, Steps: but review resolve needs-fix --thread t-1
  - Must observe: resolved_at is non-null; exit code 0
  - Must not observe: unknown subcommand: resolve
- Negative control — would fail if: resolve subcommand not registered; R22 check incorrectly blocks resolver
- Evidence: stdout (capture required: true)

### AC-3

- **GIVEN:** A governed branch with an unresolved comment thread created by another actor
- **WHEN:** A different actor runs but review resolve <branch> --thread t-1
- **THEN:** The command fails with a resolver-identity error and the thread remains unresolved
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** but-cli
- **Flow Ref:** sprint-09-step-2

**Scenario:**

- Tier: visible
- Fixtures: seeded_governed_repo_with_comment
- Cases:
  - Actor: non_reviewer, Steps: but review resolve needs-fix --thread t-1
  - Must observe: non-zero exit; resolver identity; resolved_at IS NULL
  - Must not observe: exit code 0
- Negative control — would fail if: R22 identity check skipped
- Evidence: stderr (capture required: true)

### AC-4

- **GIVEN:** The but review CLI help
- **WHEN:** The user runs but review --help
- **THEN:** comments and resolve are listed as subcommands with their arguments
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** but-cli
- **Flow Ref:** sprint-09-step-2

**Scenario:**

- Tier: visible
- Fixtures: (none)
- Cases:
  - Actor: user, Steps: but review --help
  - Must observe: comments; resolve; --thread
  - Must not observe: No such subcommand
- Negative control — would fail if: subcommands not added to Subcommands enum
- Evidence: stdout (capture required: true)

## Test Criteria

| ID   | Statement                                                        | Maps to AC |
| ---- | ---------------------------------------------------------------- | ---------- |
| TC-1 | CLI review comments lists existing comment threads with metadata | AC-1       |
| TC-2 | CLI review resolve marks an own thread resolved                  | AC-2       |
| TC-3 | CLI review resolve rejects a non-resolver actor                  | AC-3       |
| TC-4 | CLI help advertises comments and resolve subcommands             | AC-4       |

## Guardrails

**WRITE-ALLOWED:**

- crates/but/src/args/forge.rs
- crates/but/src/command/legacy/forge.rs
- crates/but/tests/but/command/review_comments.rs
- crates/but/tests/but/command/review_resolve.rs

**WRITE-PROHIBITED:**

- crates/but-api/src/legacy/forge.rs
- crates/but-authz/src/\*

## Verification Gates

- **Command:** `cargo test -p but --features but-2 review::comments`
- **Expected outcome:** all tests pass
- **Command:** `cargo test -p but --features but-2 review::resolve`
- **Expected outcome:** all tests pass
- **Command:** `cargo check -p but --all-targets`
- **Expected outcome:** clean compilation

## Reading List

- `crates/but/src/args/forge.rs` lines 28-184 — Subcommands enum to extend
- `crates/but-api/src/legacy/forge.rs` lines 923-993 — list_comments and resolve_thread backends
- `crates/but/src/command/legacy/forge.rs` lines all — existing review dispatch pattern

## Dependencies

- **Depends On:** LPR-REM-001
- **Blocks:** LPR-REM-005, LPR-REM-008

## Design

- **Pattern:** Add CLI enum variants and dispatch to existing but-api backends; keep all business logic in but-api.
- **Anti-pattern:** Reimplement backend logic in the CLI crate or bypass the R22 check.
- **References:** —

## Coding Standards

- crates/AGENTS.md

<!-- REQUIREMENT-CONTRACT v1 -->
<!-- {"requirements":[{"id":"AC-1","kind":"acceptance","scenario_ref":"AC-1"},{"id":"AC-2","kind":"acceptance","scenario_ref":"AC-2"},{"id":"AC-3","kind":"acceptance","scenario_ref":"AC-3"},{"id":"AC-4","kind":"acceptance","scenario_ref":"AC-4"},{"id":"TC-1","kind":"test","maps_to_ac":"AC-1"},{"id":"TC-2","kind":"test","maps_to_ac":"AC-2"},{"id":"TC-3","kind":"test","maps_to_ac":"AC-3"},{"id":"TC-4","kind":"test","maps_to_ac":"AC-4"}]} -->
