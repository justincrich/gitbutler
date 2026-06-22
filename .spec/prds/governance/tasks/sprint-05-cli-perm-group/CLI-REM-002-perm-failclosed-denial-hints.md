# CLI-REM-002: Close perm fail-closed variants and structured denial remediation hints

## What this does

Hardens Sprint 05 permission-management coverage around fail-closed behavior and structured denial output. This remediation adds explicit negative coverage for `perm_revoke`, `perm_list`, unset identity, bad permission tokens, and the required `remediation_hint` field across both `but perm` and `but group` denials.

## Why

The red-hat review found that CLI-001 covered the main happy paths but did not force every new CLI verb to fail closed under malformed inputs or unresolved caller identity. It also found that denial tests asserted the error code but did not consistently assert the non-empty remediation hint required by the product contract.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-api perm_revoke_fail_closed_bad_token`.

## Scope

- `crates/but-api/src/legacy/governance.rs` (MODIFY) — keep fail-closed classification and shared denial conversion inside the existing governance boundary.
- `crates/but-api/tests/perm_governance.rs` (MODIFY) — add API-level fail-closed tests for `perm_revoke` and `perm_list`.
- `crates/but-api/tests/group_governance.rs` (MODIFY) — strengthen denial assertions to require remediation hints for group calls.
- `crates/but/src/command/perm.rs` (MODIFY) — preserve structured denial rendering for CLI permission commands.
- `crates/but/src/command/group.rs` (MODIFY) — preserve structured denial rendering for CLI group commands.
- `crates/but/tests/but/command/perm.rs` (MODIFY) — add CLI denial rendering assertions.
- `crates/but/tests/but/command/group.rs` (MODIFY) — add CLI denial rendering assertions.

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: CLI-REM-002 - Close perm fail-closed variants and denial hints
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     S (90 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-AUTHZ-01, UC-AUTHZ-03
CAPABILITIES: CAP-AUTHZ-01, CAP-CONFIG-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-api perm_revoke_fail_closed_bad_token
         cargo test -p but-api perm_revoke_fail_closed_unset_handle
         cargo test -p but-api perm_list_fail_closed_unset_handle
         cargo test -p but-api perm_denials_include_remediation_hint group_denials_include_remediation_hint
         cargo test -p but perm_denials_include_remediation_hint group_denials_include_remediation_hint
  check: cargo check -p but-api --all-targets && cargo check -p but --all-targets
  fmt:   cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
The `but perm` and `but group` contracts cannot pass unless every new denial path fails closed and emits structured denial data with code, message, and non-empty remediation_hint.

--------------------------------------------------------------------------------
CRITICAL CONSTRAINTS
--------------------------------------------------------------------------------
- [MUST] MUST reject bad permission tokens before any file mutation.
- [MUST] MUST reject unset caller identity for `perm_revoke` and `perm_list`.
- [MUST] MUST assert non-empty `remediation_hint` on API and CLI denial paths.
- [MUST] MUST leave working-tree permissions byte-for-byte unchanged on every denial.
- [NEVER] NEVER authorize by default when `BUT_AGENT_HANDLE` is missing or malformed.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: `perm_revoke` rejects unparseable permission tokens and writes nothing.
- [ ] AC-2: `perm_revoke` with unset caller identity fails closed with remediation hint.
- [ ] AC-3: `perm_list` with unset caller identity fails closed with remediation hint and no reconnaissance.
- [ ] AC-4: both perm and group denial tests require code, message, and remediation_hint.
- [ ] All verification gates pass; only writeAllowed files modified.

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA
--------------------------------------------------------------------------------

AC-1: perm_revoke bad token fails closed [PRIMARY]
  GIVEN: committed governance where `admin` has `administration:write` and `rust-implementer` has `reviews:write`.
  WHEN: admin calls `perm_revoke(&repo, "refs/heads/main", "rust-implementer", ["not a permission"])`.
  THEN: the call returns a structured invalid-permission/config error, does not downgrade to success, and leaves the working-tree permissions file byte-for-byte unchanged.
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api perm_revoke + real but-authz + real git
  VERIFY: cargo test -p but-api perm_revoke_fail_closed_bad_token

AC-2: perm_revoke unset handle fails closed with hint
  GIVEN: the same committed governance.
  WHEN: `BUT_AGENT_HANDLE` is unset and `perm_revoke` is invoked for any principal.
  THEN: the call returns `perm.denied` or identity resolution denial, includes non-empty `remediation_hint`, and writes nothing.
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api identity resolution + perm_revoke
  VERIFY: cargo test -p but-api perm_revoke_fail_closed_unset_handle

AC-3: perm_list unset handle fails closed with hint
  GIVEN: committed governance containing multiple principals and groups.
  WHEN: `BUT_AGENT_HANDLE` is unset and `perm_list --principal rust-reviewer` is invoked.
  THEN: the call is denied, includes non-empty `remediation_hint`, and does not reveal another principal's authorities.
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api/CLI perm_list denial path
  VERIFY: cargo test -p but-api perm_list_fail_closed_unset_handle

AC-4: Denial contract asserts remediation_hint across perm/group
  GIVEN: non-admin callers exercise denied `perm_grant`, `perm_revoke`, `perm_list --principal <other>`, and group mutators.
  WHEN: API and CLI tests capture the structured error payload/stderr.
  THEN: each denial asserts `code`, `message`, and non-empty `remediation_hint`, with exit code 1 in CLI tests.
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api errors + real snapbox CLI output
  VERIFY: cargo test -p but-api perm_denials_include_remediation_hint group_denials_include_remediation_hint && cargo test -p but perm_denials_include_remediation_hint group_denials_include_remediation_hint

--------------------------------------------------------------------------------
TEST CRITERIA
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): revoke bad token is an error and unchanged file. VERIFY: `cargo test -p but-api perm_revoke_fail_closed_bad_token`
- TC-2 (-> AC-2): revoke with unset caller identity denies with remediation hint. VERIFY: `cargo test -p but-api perm_revoke_fail_closed_unset_handle`
- TC-3 (-> AC-3): list with unset caller identity denies with remediation hint and no recon. VERIFY: `cargo test -p but-api perm_list_fail_closed_unset_handle`
- TC-4 (-> AC-4): API denials for perm/group include remediation_hint. VERIFY: `cargo test -p but-api perm_denials_include_remediation_hint group_denials_include_remediation_hint`
- TC-5 (-> AC-4): CLI denials for perm/group include remediation_hint and exit 1. VERIFY: `cargo test -p but perm_denials_include_remediation_hint group_denials_include_remediation_hint`

--------------------------------------------------------------------------------
SCOPE
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-api/src/legacy/governance.rs (MODIFY)
  - crates/but-api/tests/perm_governance.rs (MODIFY)
  - crates/but-api/tests/group_governance.rs (MODIFY)
  - crates/but/src/command/perm.rs (MODIFY)
  - crates/but/src/command/group.rs (MODIFY)
  - crates/but/tests/but/command/perm.rs (MODIFY)
  - crates/but/tests/but/command/group.rs (MODIFY)
writeProhibited:
  - Any fallback authorizing missing identity.
  - Any code path that converts parse or identity failures into success.
  - Any unrelated UI, Tauri, or SDK file.

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-api/src/legacy/governance.rs (full)
   Focus: permission token parsing, caller resolution, error classification, and shared denial helpers.
2. crates/but-api/tests/perm_governance.rs (full)
   Focus: existing perm fixtures and denial assertions.
3. crates/but-api/tests/group_governance.rs (full)
   Focus: shared denial expectations for group mutators.
4. crates/but/src/command/perm.rs and crates/but/src/command/group.rs
   Focus: structured CLI error rendering.
5. .spec/reviews/red-hat-20260620T051414Z.md
   Focus: fail-closed and remediation-hint findings.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- `cargo test -p but-api perm_revoke_fail_closed_bad_token` -> Exit 0.
- `cargo test -p but-api perm_revoke_fail_closed_unset_handle` -> Exit 0.
- `cargo test -p but-api perm_list_fail_closed_unset_handle` -> Exit 0.
- `cargo test -p but-api perm_denials_include_remediation_hint group_denials_include_remediation_hint` -> Exit 0.
- `cargo test -p but perm_denials_include_remediation_hint group_denials_include_remediation_hint` -> Exit 0.
- `cargo check -p but-api --all-targets && cargo check -p but --all-targets` -> Exit 0.
- `cargo fmt --check` -> Exit 0.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — owns Rust API/CLI tests and structured denial behavior.
reviewer: rust-reviewer
coding_standards: RULES.md, crates/AGENTS.md, crates/but/AGENTS.md

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: CLI-001, CLI-002
Blocks: Sprint 05 approval, Sprint 06a governance UI reuse confidence
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "CLI-REM-002",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "perm_governance_base": {
      "description": "Real-git scenario via but_testsupport::writable_scenario(\"checkout-head-info\"). Target ref refs/heads/main carries committed .gitbutler/permissions.toml with admin holding administration:write, rust-implementer holding reviews:write, rust-reviewer holding reviews:read, group maintainers members=[\"rust-reviewer\"], and gates.toml protecting main.",
      "seed_method": "cli",
      "records": [
        "invoke_bash: mkdir -p .gitbutler",
        "write .gitbutler/permissions.toml with [[principal]] id=\"admin\" permissions=[\"administration:write\"]; [[principal]] id=\"rust-implementer\" permissions=[\"reviews:write\"]; [[principal]] id=\"rust-reviewer\" permissions=[\"reviews:read\"]; [[group]] name=\"maintainers\" permissions=[\"merge\"] members=[\"rust-reviewer\"]",
        "write .gitbutler/gates.toml with [[branch]] name=\"main\" protected=true",
        "git add .gitbutler/permissions.toml .gitbutler/gates.toml && git commit -m \"governance config\"",
        "capture committed_blob_text(repo, but_authz::permissions_path()) and ref_id(repo, \"refs/heads/main\") before each negative operation"
      ]
    },
    "denial_contract_cli_base": {
      "description": "Real CLI scenario invoking `but perm` and `but group` commands as a non-admin or unset caller, capturing stderr/stdout/exit code from the snapbox harness.",
      "seed_method": "cli",
      "records": [
        "env.but([\"perm\", \"grant\", \"--principal\", \"rust-reviewer\", \"reviews:write\"]).assert() with BUT_AGENT_HANDLE=rust-reviewer",
        "env.but([\"perm\", \"list\", \"--principal\", \"rust-implementer\"]).assert() with BUT_AGENT_HANDLE unset",
        "env.but([\"group\", \"add-member\", \"maintainers\", \"--principal\", \"rust-implementer\"]).assert() with BUT_AGENT_HANDLE=rust-reviewer",
        "capture stderr contains code, message, remediation_hint, and exit code 1"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "perm_revoke rejects an unparseable permission token before mutation and leaves the working-tree file unchanged.",
      "verify": "cargo test -p but-api perm_revoke_fail_closed_bad_token",
      "scenario": {
        "id": "AC-1",
        "test_tier": "integration",
        "negative_control": {
          "would_fail_if": [
            "bad tokens are ignored and revoke returns Ok",
            "a stub revoke accepts every token",
            "the writer rewrites permissions before token validation"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "perm_governance_base",
            "action": {
              "steps": [
                "capture working-tree .gitbutler/permissions.toml text",
                "set BUT_AGENT_HANDLE=admin",
                "call perm_revoke(&repo, \"refs/heads/main\", \"rust-implementer\", [\"not a permission\"])",
                "read working-tree .gitbutler/permissions.toml text again"
              ]
            },
            "end_state": {
              "must_observe": [
                "call returns structured error code `config.invalid` or invalid permission token",
                "error message contains literal `not a permission`",
                "working-tree text after == text before"
              ],
              "must_not_observe": [
                "no success response",
                "no token removal",
                "empty error message"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "perm_revoke with unset caller identity fails closed with remediation_hint and no file mutation.",
      "verify": "cargo test -p but-api perm_revoke_fail_closed_unset_handle",
      "scenario": {
        "id": "AC-2",
        "test_tier": "integration",
        "negative_control": {
          "would_fail_if": [
            "a stub identity resolver returns admin when BUT_AGENT_HANDLE is missing",
            "missing BUT_AGENT_HANDLE falls back to admin",
            "missing identity is treated as anonymous allow",
            "denied revoke mutates the file"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "perm_governance_base",
            "action": {
              "steps": [
                "capture working-tree permissions text",
                "unset BUT_AGENT_HANDLE",
                "call perm_revoke(&repo, \"refs/heads/main\", \"rust-implementer\", [\"reviews:write\"])",
                "read working-tree permissions text again"
              ]
            },
            "end_state": {
              "must_observe": [
                "call returns denial code `perm.denied` or identity resolution error",
                "denial has remediation_hint length `> 0`",
                "working-tree text after == text before"
              ],
              "must_not_observe": [
                "no fallback principal",
                "no success response",
                "empty remediation_hint"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "perm_list with unset caller identity fails closed with remediation_hint and reveals no other-principal authorities.",
      "verify": "cargo test -p but-api perm_list_fail_closed_unset_handle",
      "scenario": {
        "id": "AC-3",
        "test_tier": "integration",
        "negative_control": {
          "would_fail_if": [
            "missing identity lists another principal",
            "perm_list ignores the self-or-admin-read predicate",
            "denial omits remediation_hint"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "perm_governance_base",
            "action": {
              "steps": [
                "unset BUT_AGENT_HANDLE",
                "call perm_list(&repo, \"refs/heads/main\", Some(\"rust-implementer\"))",
                "capture returned error and any rendered output"
              ]
            },
            "end_state": {
              "must_observe": [
                "call returns denial code `perm.denied` or identity resolution error",
                "denial has remediation_hint length `> 0`",
                "output does not include `reviews:write` for rust-implementer"
              ],
              "must_not_observe": [
                "no other-principal authorities",
                "empty remediation_hint",
                "no successful list response"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "description": "API and CLI denial assertions for perm/group require code, message, non-empty remediation_hint, and CLI exit code 1.",
      "verify": "cargo test -p but-api perm_denials_include_remediation_hint group_denials_include_remediation_hint && cargo test -p but perm_denials_include_remediation_hint group_denials_include_remediation_hint",
      "scenario": {
        "id": "AC-4",
        "test_tier": "integration",
        "negative_control": {
          "would_fail_if": [
            "a static CLI renderer prints code only and drops remediation_hint",
            "denials only assert code",
            "CLI renderer drops remediation_hint",
            "group denial returns exit 0"
          ]
        },
        "evidence": { "artifact_type": "stderr", "required_capture": true },
        "cases": [
          {
            "start_ref": "denial_contract_cli_base",
            "action": {
              "steps": [
                "run denied but-api perm_grant, perm_revoke, perm_list, and group mutator calls",
                "run denied CLI `but perm` and `but group` commands through env.but(...).assert()",
                "capture structured API errors and CLI stderr"
              ]
            },
            "end_state": {
              "must_observe": [
                "each API denial contains code, message, and remediation_hint length `> 0`",
                "each CLI denial exits `1`",
                "each CLI stderr contains `remediation_hint`"
              ],
              "must_not_observe": [
                "empty remediation_hint",
                "exit code `0` on denial",
                "no bare message-only denial"
              ]
            }
          }
        ]
      }
    },
    { "id": "TC-1", "type": "test_criterion", "description": "revoke bad token is an error and unchanged file", "verify": "cargo test -p but-api perm_revoke_fail_closed_bad_token", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "revoke with unset caller identity denies with remediation hint", "verify": "cargo test -p but-api perm_revoke_fail_closed_unset_handle", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "list with unset caller identity denies with remediation hint and no recon", "verify": "cargo test -p but-api perm_list_fail_closed_unset_handle", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "API denials for perm/group include remediation_hint", "verify": "cargo test -p but-api perm_denials_include_remediation_hint group_denials_include_remediation_hint", "maps_to_ac": "AC-4" },
    { "id": "TC-5", "type": "test_criterion", "description": "CLI denials for perm/group include remediation_hint and exit 1", "verify": "cargo test -p but perm_denials_include_remediation_hint group_denials_include_remediation_hint", "maps_to_ac": "AC-4" }
  ]
}
-->
