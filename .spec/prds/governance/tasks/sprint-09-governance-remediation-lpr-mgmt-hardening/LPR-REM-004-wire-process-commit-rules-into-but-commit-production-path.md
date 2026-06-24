---
task: LPR-REM-004
sprint: sprint-09-governance-remediation-lpr-mgmt-hardening
sequence: 11
agent: rust-implementer
estimate_minutes: 180
status: pending
proposed_by: rust-planner
type: REMEDIATION
generated_at: 2026-06-23T13:30:00Z
generated_by: kb-sprint-tasks-plan
---

# LPR-REM-004: Wire process_commit_rules into but commit production path

**Agent:** `rust-implementer` (180 min)
**Proposed By:** `rust-planner`
**Type:** REMEDIATION
**Status:** pending
**Depends On:** —
**Blocks:** —

## Background

**Problem:** process_commit_rules in crates/but-rules/src/lib.rs:426-432 is fully implemented and has four passing tests, but no production code in crates/but/ calls it.

**Why it matters:** Commit rules are invisible to users until they run as part of but commit; the implemented rule engine is currently dead code.

**Current state:** grep -r "process_commit_rules" crates/but/ returns zero matches; post-commit hooks at but/src/command/legacy/commit.rs:556 run without rule processing.

**Desired state:** but commit invokes process_commit_rules after post-commit hooks, records rule outcomes idempotently, and continues the commit even if rule evaluation fails.

## Critical Constraints

- MUST call process_commit_rules after post-commit hooks at commit.rs:556.
- MUST be idempotent: re-running commit on the same commit does not duplicate rule artifacts.
- MUST NOT block or roll back the commit if rule processing fails; log/report only.
- MUST reuse but-rules crate; do not reimplement rule logic.
- NEVER introduce unwrap or panics in the rule-processing path.

## Specification

**Objective:** Integrate the existing process_commit_rules into the but commit production path.

**Success state:** cargo test -p but --features but-2 commit::rules passes; commits succeed regardless of rule outcomes; rule artifacts are created when the engine runs.

## Acceptance Criteria

### AC-1

- **GIVEN:** A governed repo configured with a simple commit rule
- **WHEN:** The user runs but commit
- **THEN:** process_commit_rules is invoked after post-commit hooks and produces deterministic rule artifacts
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** but-cli
- **Flow Ref:** sprint-09-step-4

**Scenario:**

- Tier: visible
- Fixtures: seeded_governed_repo_with_commit_rule
- Cases:
  - Actor: user, Steps: make a change; but commit -m "test"
  - Must observe: commit succeeds; rule artifacts created
  - Must not observe: process_commit_rules not called
- Negative control — would fail if: hook path does not call process_commit_rules
- Evidence: test_output (capture required: true)

### AC-2

- **GIVEN:** A governed repo where commit rule evaluation would error
- **WHEN:** The user runs but commit
- **THEN:** The commit still completes and the rule failure is surfaced non-fatally
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** but-cli
- **Flow Ref:** sprint-09-step-4

**Scenario:**

- Tier: visible
- Fixtures: seeded_governed_repo_with_broken_rule
- Cases:
  - Actor: user, Steps: but commit -m "test broken rule"
  - Must observe: commit created; rule error reported without panic
  - Must not observe: commit rolled back; panic; unwrap
- Negative control — would fail if: rule error aborts commit
- Evidence: stderr (capture required: true)

### AC-3

- **GIVEN:** A commit on which process_commit_rules has already run
- **WHEN:** The user re-runs but commit with --amend or equivalent retrigger
- **THEN:** Rule artifacts are not duplicated
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** but-cli
- **Flow Ref:** sprint-09-step-4

**Scenario:**

- Tier: visible
- Fixtures: seeded_governed_repo_with_commit_rule
- Cases:
  - Actor: user, Steps: but commit --amend -m "same rule run again"
  - Must observe: single set of rule artifacts per commit
  - Must not observe: duplicate rule outcome rows
- Negative control — would fail if: process_commit_rules is not idempotent
- Evidence: test_output (capture required: true)

### AC-4

- **GIVEN:** The existing but-rules tests
- **WHEN:** cargo test -p but-rules runs
- **THEN:** The four tests at crates/but-rules/tests/review_requested_hook.rs still pass
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** cargo-test
- **Flow Ref:** sprint-09-step-4

**Scenario:**

- Tier: visible
- Fixtures: (none)
- Cases:
  - Actor: test_runner, Steps: cargo test -p but-rules
  - Must observe: review_requested_hook tests pass
  - Must not observe: regression in rule engine
- Negative control — would fail if: changes broke but-rules tests
- Evidence: test_output (capture required: true)

## Test Criteria

| ID   | Statement                                                          | Maps to AC |
| ---- | ------------------------------------------------------------------ | ---------- |
| TC-1 | but commit invokes process_commit_rules and creates rule artifacts | AC-1       |
| TC-2 | Rule processing failures do not block commit creation              | AC-2       |
| TC-3 | Re-running commit does not duplicate rule artifacts                | AC-3       |
| TC-4 | but-rules test suite remains green                                 | AC-4       |

## Guardrails

**WRITE-ALLOWED:**

- crates/but/src/command/legacy/commit.rs
- crates/but/tests/but/command/commit_rules.rs

**WRITE-PROHIBITED:**

- crates/but-rules/src/lib.rs
- crates/but-authz/src/\*

## Verification Gates

- **Command:** `cargo test -p but --features but-2 commit::rules`
- **Expected outcome:** all tests pass
- **Command:** `cargo test -p but-rules`
- **Expected outcome:** all but-rules tests pass
- **Command:** `cargo check -p but --all-targets`
- **Expected outcome:** clean compilation

## Reading List

- `crates/but-rules/src/lib.rs` lines 426-432 — process_commit_rules signature
- `crates/but-rules/tests/review_requested_hook.rs` lines all — existing passing rule tests
- `crates/but/src/command/legacy/commit.rs` lines 556 — post-commit hook insertion point

## Dependencies

- **Depends On:** —
- **Blocks:** —

## Design

- **Pattern:** Call the provided but-rules function from the CLI commit path; wrap in non-fatal error handling; ensure idempotency via backend semantics.
- **Anti-pattern:** Reimplement rule logic or fail the commit on rule-processing errors.
- **References:** —

## Coding Standards

- crates/AGENTS.md

<!-- REQUIREMENT-CONTRACT v1 -->
<!-- {"requirements":[{"id":"AC-1","kind":"acceptance","scenario_ref":"AC-1"},{"id":"AC-2","kind":"acceptance","scenario_ref":"AC-2"},{"id":"AC-3","kind":"acceptance","scenario_ref":"AC-3"},{"id":"AC-4","kind":"acceptance","scenario_ref":"AC-4"},{"id":"TC-1","kind":"test","maps_to_ac":"AC-1"},{"id":"TC-2","kind":"test","maps_to_ac":"AC-2"},{"id":"TC-3","kind":"test","maps_to_ac":"AC-3"},{"id":"TC-4","kind":"test","maps_to_ac":"AC-4"}]} -->
