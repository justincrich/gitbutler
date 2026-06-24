---
task: LPR-REM-007
sprint: sprint-09-governance-remediation-lpr-mgmt-hardening
sequence: 11
agent: rust-implementer
estimate_minutes: 60
status: pending
proposed_by: rust-planner
type: REMEDIATION
generated_at: 2026-06-23T13:30:00Z
generated_by: kb-sprint-tasks-plan
---

# LPR-REM-007: Surface kind in GovernancePrincipalListEntry (Rust half)

**Agent:** `rust-implementer` (60 min)
**Proposed By:** `rust-planner`
**Type:** REMEDIATION
**Status:** pending
**Depends On:** —
**Blocks:** —

## Background

**Problem:** GovernancePrincipalListEntry at crates/but-api/src/legacy/governance.rs:216-227 lacks a kind field, forcing callers to make a second API call or display principals without type information.

**Why it matters:** The governance UI and downstream consumers need to distinguish users, groups, service accounts, and bots in principal lists.

**Current state:** principal_kind_read API exists and has four passing tests at crates/but-api/tests/principal_kind.rs, but the list query does not join it.

**Desired state:** GovernancePrincipalListEntry includes kind: Option<String>, populated by merging principal_kind_read results into the list query.

## Critical Constraints

- MUST add kind: Option<String> to GovernancePrincipalListEntry.
- MUST populate kind from principal_kind_read or equivalent internal query without adding N+1 round-trips.
- MUST keep existing list fields intact and backward-compatible.
- MUST NOT expose internal identifiers in the kind string.
- STRICTLY reuse the existing principal_kind_read logic.

## Specification

**Objective:** Augment the principal list DTO with the principal kind.

**Success state:** cargo test -p but-api principal_kind and principal_list pass, and list payloads include kind for each entry.

## Acceptance Criteria

### AC-1

- **GIVEN:** A governance realm with principals of different kinds
- **WHEN:** The principals list endpoint is invoked
- **THEN:** Each entry includes kind with the correct principal type
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** but-api-test
- **Flow Ref:** sprint-09-step-7

**Scenario:**

- Tier: visible
- Fixtures: governance_realm_fixture
- Cases:
  - Actor: api_caller, Steps: call principals list endpoint
  - Must observe: kind="user" for user principals; kind="group" for group principals; kind present in every entry
  - Must not observe: kind field missing; kind is null for known principals
- Negative control — would fail if: kind not merged into list query
- Evidence: json_payload (capture required: true)

### AC-2

- **GIVEN:** The existing principal_kind_read tests
- **WHEN:** cargo test -p but-api principal_kind runs
- **THEN:** All four existing tests still pass
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** cargo-test
- **Flow Ref:** sprint-09-step-7

**Scenario:**

- Tier: visible
- Fixtures: (none)
- Cases:
  - Actor: test_runner, Steps: cargo test -p but-api principal_kind
  - Must observe: 4 tests pass
  - Must not observe: regressions in principal_kind_read
- Negative control — would fail if: principal_kind_read changed behavior
- Evidence: test_output (capture required: true)

### AC-3

- **GIVEN:** A principal whose kind is unknown or not yet set
- **WHEN:** The principals list endpoint returns that entry
- **THEN:** kind is Option::None rather than an arbitrary default string
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** but-api-test
- **Flow Ref:** sprint-09-step-7

**Scenario:**

- Tier: visible
- Fixtures: governance_realm_fixture
- Cases:
  - Actor: api_caller, Steps: call principals list endpoint
  - Must observe: kind is null/None for unknown kind
  - Must not observe: kind=""; kind="unknown"
- Negative control — would fail if: kind defaulted to empty string
- Evidence: json_payload (capture required: true)

### AC-4

- **GIVEN:** The list query implementation
- **WHEN:** cargo check -p but-api --all-targets runs
- **THEN:** No N+1 query pattern is introduced by the kind merge
- **TDD State:** RED
- **Test Tier:** integration
- **Verification Service:** cargo-check
- **Flow Ref:** sprint-09-step-7

**Scenario:**

- Tier: visible
- Fixtures: (none)
- Cases:
  - Actor: reviewer, Steps: cargo check -p but-api --all-targets; review diff for single query join or batch lookup
  - Must observe: clean compile; no per-row API call
  - Must not observe: loop over principals calling kind_read individually
- Negative control — would fail if: N+1 pattern introduced
- Evidence: diff (capture required: true)

## Test Criteria

| ID   | Statement                                            | Maps to AC |
| ---- | ---------------------------------------------------- | ---------- |
| TC-1 | Principals list entries include kind for known types | AC-1       |
| TC-2 | Existing principal_kind tests remain green           | AC-2       |
| TC-3 | Unknown kinds serialize as null/None                 | AC-3       |
| TC-4 | Kind merge does not introduce an N+1 query pattern   | AC-4       |

## Guardrails

**WRITE-ALLOWED:**

- crates/but-api/src/legacy/governance.rs

**WRITE-PROHIBITED:**

- crates/but-authz/src/\*
- crates/but/src/args/\*.rs

## Verification Gates

- **Command:** `cargo test -p but-api principal_kind`
- **Expected outcome:** 4 existing tests pass
- **Command:** `cargo test -p but-api principal_list`
- **Expected outcome:** principal list tests pass with kind field
- **Command:** `cargo check -p but-api --all-targets`
- **Expected outcome:** clean compilation

## Reading List

- `crates/but-api/src/legacy/governance.rs` lines 216-227 — GovernancePrincipalListEntry definition
- `crates/but-api/tests/principal_kind.rs` lines all — existing kind-read tests to reuse

## Dependencies

- **Depends On:** —
- **Blocks:** —

## Design

- **Pattern:** Single-query join or batch lookup to attach kind to list entries using existing principal_kind_read semantics.
- **Anti-pattern:** Loop over list results issuing individual kind reads.
- **References:** —

## Coding Standards

- crates/AGENTS.md

<!-- REQUIREMENT-CONTRACT v1 -->
<!-- {"requirements":[{"id":"AC-1","kind":"acceptance","scenario_ref":"AC-1"},{"id":"AC-2","kind":"acceptance","scenario_ref":"AC-2"},{"id":"AC-3","kind":"acceptance","scenario_ref":"AC-3"},{"id":"AC-4","kind":"acceptance","scenario_ref":"AC-4"},{"id":"TC-1","kind":"test","maps_to_ac":"AC-1"},{"id":"TC-2","kind":"test","maps_to_ac":"AC-2"},{"id":"TC-3","kind":"test","maps_to_ac":"AC-3"},{"id":"TC-4","kind":"test","maps_to_ac":"AC-4"}]} -->
