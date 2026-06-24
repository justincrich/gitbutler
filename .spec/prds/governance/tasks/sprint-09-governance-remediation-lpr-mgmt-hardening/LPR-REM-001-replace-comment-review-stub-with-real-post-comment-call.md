---
task: LPR-REM-001
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

# LPR-REM-001: Replace comment_review stub with real post_comment call

**Agent:** `rust-implementer` (180 min)
**Proposed By:** `rust-planner`
**Type:** REMEDIATION
**Status:** pending
**Depends On:** —
**Blocks:** LPR-REM-002, LPR-REM-005, LPR-REM-008

## Background

**Problem:** The legacy review report path at crates/but-api/src/legacy/forge.rs:841-852 returns a task_contract_invalid stub for comment_review instead of persisting comments through the real post_comment backend.

**Why it matters:** PRD Sprint 07 Gate Step 4 requires reviewer comments to be recorded and threaded; without this, local review comments cannot be created, breaking the review workflow and downstream review_status reads.

**Current state:** comment_review delegates to a stub that reports a task_contract_invalid error and never writes local_review_comments rows. CLI args only expose branch and message.

**Desired state:** comment_review fans out to the existing post_comment implementation, accepts --file/--line/--thread arguments, rejects reserved **pr_meta** thread IDs, and writes local_review_comments rows. The existing failing assertion in but/tests/but/command/review_guard.rs:164 is updated to assert success.

## Critical Constraints

- MUST reuse the existing post_comment implementation at forge.rs:880-910 without duplicating SQL or validation logic.
- MUST extend CLI args at crates/but/src/args/forge.rs:97-104 to accept --file, --line, and --thread.
- MUST preserve the reserved **pr_meta** thread-id refusal already present at forge.rs:888-891.
- NEVER silently swallow post_comment errors; propagate them as review report failures.
- STRICTLY update the test at crates/but/tests/but/command/review_guard.rs:164 from stub-failure assertion to success assertion.

## Specification

**Objective:** Convert the comment_review report stub into a real local_review_comments write path invoked by the but review CLI.

**Success state:** cargo test -p but --features but-2 command::review_guard passes and shows reviewer comments succeed; local_review_comments rows are written for valid inputs and blocked for reserved **pr_meta** thread IDs.

## Acceptance Criteria

### AC-1

- **GIVEN:** A governed repo with an active local review and no existing comments
- **WHEN:** The reviewer runs but review <branch> -m "needs change" --file src/lib.rs --line 42 --thread t-1
- **THEN:** One local_review_comments row is inserted with the supplied file, line, thread, and message, and the command exits 0
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** but-cli
- **Flow Ref:** sprint-09-step-1

**Scenario:**

- Tier: visible
- Fixtures: seeded_governed_repo
- Cases:
  - Actor: reviewer, Steps: but review needs-fix -m "needs change" --file src/lib.rs --line 42 --thread t-1
  - Must observe: exit code 0; row in local_review_comments with file=src/lib.rs, line=42, thread=t-1, message="needs change"
  - Must not observe: comment_review cannot report success; task_contract_invalid
- Negative control — would fail if: stub still returned task_contract_invalid; post_comment validation not applied
- Evidence: stdout (capture required: true)

### AC-2

- **GIVEN:** A governed repo with an active local review
- **WHEN:** The reviewer attempts to post a comment with thread id **pr_meta**
- **THEN:** The command fails with a clear error and no row is written
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** but-cli
- **Flow Ref:** sprint-09-step-1

**Scenario:**

- Tier: visible
- Fixtures: seeded_governed_repo
- Cases:
  - Actor: reviewer, Steps: but review needs-fix -m "meta" --file src/lib.rs --line 1 --thread **pr_meta**
  - Must observe: non-zero exit; reserved thread id
  - Must not observe: row in local_review_comments with thread=**pr_meta**
- Negative control — would fail if: reserved-thread guard was removed
- Evidence: stderr (capture required: true)

### AC-3

- **GIVEN:** The existing integration test at crates/but/tests/but/command/review_guard.rs:164
- **WHEN:** The test is updated to expect successful comment persistence instead of the stub error
- **THEN:** cargo test -p but --features but-2 command::review_guard passes
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** but-cli
- **Flow Ref:** sprint-09-step-1

**Scenario:**

- Tier: visible
- Fixtures: test_fixture_review_guard
- Cases:
  - Actor: test_runner, Steps: update assertion from contains("comment_review cannot report success") to contains("comment recorded") or exit 0; run cargo test -p but --features but-2 command::review_guard
  - Must observe: test finished successfully
  - Must not observe: comment_review cannot report success
- Negative control — would fail if: assertion still expected stub behavior
- Evidence: test_output (capture required: true)

### AC-4

- **GIVEN:** The but review CLI help text
- **WHEN:** The user runs but review --help
- **THEN:** The options --file, --line, and --thread are documented and accepted by the arg parser
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** but-cli
- **Flow Ref:** sprint-09-step-1

**Scenario:**

- Tier: visible
- Fixtures: (none)
- Cases:
  - Actor: user, Steps: but review --help
  - Must observe: --file <FILE>; --line <LINE>; --thread <THREAD>
  - Must not observe: unknown argument --file
- Negative control — would fail if: CLI args were not extended
- Evidence: stdout (capture required: true)

## Test Criteria

| ID   | Statement                                                                                        | Maps to AC |
| ---- | ------------------------------------------------------------------------------------------------ | ---------- |
| TC-1 | A reviewer comment with file, line, and thread writes a local_review_comments row and exits zero | AC-1       |
| TC-2 | A reviewer comment using the reserved **pr_meta** thread id is rejected with no database write   | AC-2       |
| TC-3 | The existing review_guard integration test no longer asserts stub failure and passes             | AC-3       |
| TC-4 | The CLI --help output lists --file, --line, and --thread options                                 | AC-4       |

## Guardrails

**WRITE-ALLOWED:**

- crates/but-api/src/legacy/forge.rs
- crates/but/src/args/forge.rs
- crates/but/src/command/legacy/forge.rs
- crates/but/tests/but/command/review_guard.rs

**WRITE-PROHIBITED:**

- crates/but-authz/src/\*
- crates/gitbutler-project/src/\*

## Verification Gates

- **Command:** `cargo test -p but --features but-2 command::review_guard`
- **Expected outcome:** all tests pass; reviewer comment succeeds instead of stub error
- **Command:** `cargo check -p but-api --all-targets`
- **Expected outcome:** clean compilation
- **Command:** `cargo check -p but --all-targets`
- **Expected outcome:** clean compilation

## Reading List

- `crates/but-api/src/legacy/forge.rs` lines 841-910 — stub vs real post_comment and reserved thread guard
- `crates/but/src/args/forge.rs` lines 97-104 — current CLI args for comment command
- `crates/but/tests/but/command/review_guard.rs` lines 164 — stub assertion to update

## Dependencies

- **Depends On:** —
- **Blocks:** LPR-REM-002, LPR-REM-005, LPR-REM-008

## Design

- **Pattern:** Delegate report path to existing backend function; extend CLI with optional arguments; reuse validation.
- **Anti-pattern:** Duplicate SQL inserts or ignore reserved-thread validation.
- **References:** —

## Coding Standards

- crates/AGENTS.md

<!-- REQUIREMENT-CONTRACT v1 -->
<!-- {"requirements":[{"id":"AC-1","kind":"acceptance","scenario_ref":"AC-1"},{"id":"AC-2","kind":"acceptance","scenario_ref":"AC-2"},{"id":"AC-3","kind":"acceptance","scenario_ref":"AC-3"},{"id":"AC-4","kind":"acceptance","scenario_ref":"AC-4"},{"id":"TC-1","kind":"test","maps_to_ac":"AC-1"},{"id":"TC-2","kind":"test","maps_to_ac":"AC-2"},{"id":"TC-3","kind":"test","maps_to_ac":"AC-3"},{"id":"TC-4","kind":"test","maps_to_ac":"AC-4"}]} -->
