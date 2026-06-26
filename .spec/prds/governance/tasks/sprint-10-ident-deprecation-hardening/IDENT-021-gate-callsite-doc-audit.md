# IDENT-021 — Audit doc-comments across the 11 runtime-registry identity surfaces — each names the resolution order (registry → flag-gated env → denial) so the invariant is documented in code, not just tested

**Sprint:** [Sprint 10](./SPRINT.md) · **Agent:** `rust-reviewer` · **Estimate:** 90 min · **Type:** FEATURE · **Status:** Complete · **Proposed By:** rust-planner

## Background

rust-reviewer owns documentation quality and must audit all 11 runtime-registry identity surfaces to ensure the resolution order is documented — this is review-qualified work requiring attention to doc-comment accuracy..

**Why it matters.** Closes the IDENT deprecation arc: on a governed repo the registry path is the default; the env-var path survives only as an opt-in escape hatch behind `BUT_AUTHZ_ALLOW_ENV_HANDLE=1`.

**Provides:** Documented resolution order at all 11 runtime-registry identity surfaces, Doc-comment audit passing invariant_build_gates

**Consumes:** Sprint-09 IDENT-010 (callsites swapped to `resolve_principal_with_runtime_registry`, whose wrapper delegates to `but_authz::resolve_principal_with_registry`), IDENT-017 (resolver documented), IDENT-020 (invariant gates extended)

**Boundary contracts:**
- Each of the 11 runtime-registry identity surfaces has a doc-comment naming the resolution order
- Doc-comments are accurate to the Sprint-10 policy


## Critical Constraints

**MUST:**
- Audit all 11 runtime-registry identity surfaces for doc-comments: `commit/gate.rs::enforce_commit_gate_for_target`, `legacy/merge_gate.rs::enforce_merge_gate`, `legacy/config_mutate.rs::enforce_administration_write_gate`, `legacy/forge.rs::authorize_branch_action`, `legacy/rules.rs::list_workspace_rules_scoped_for_caller`, and `legacy/governance.rs::{governance_status_read, branch_gates_read_with_repo, group_list_with_repo, perm_list_with_repo, whoami_with_repo, can_i_with_repo}`
- Each doc-comment must explicitly state the resolution order: registry → flag-gated env → denial
- Doc-comments must name the specific functions/constants: `resolve_principal_with_runtime_registry`, `but_authz::resolve_principal_with_registry`, `BUT_AUTHZ_ALLOW_ENV_HANDLE`, `Denial::unregistered`
- Add build-gate checks to invariant_build_gates.rs that inspect the contiguous `///` doc block immediately preceding each gate function; do not grep after function declarations
- All doc-comments must pass cargo doc --no-deps -p but-api
- Doc-comments must reference the resolver module (but_authz::authorize)

**NEVER:**
- Modify function signatures or logic — this is doc-comment ONLY
- Skip callsites — all 11 surfaces must have documentation
- Use vague language like 'resolves identity' — must be specific about the resolution order
- Copy-paste identical docs — each callsite can have context-specific details

**STRICTLY:**
- BLOCKED-UNTIL Sprint-09 IDENT-010 completes (callsites must be swapped before documenting)
- BLOCKED-UNTIL IDENT-017 completes (resolver must be documented as test-only)
- BLOCKED-UNTIL IDENT-020 completes (invariant gates provide the grep framework)
- Doc-comments are the ONLY acceptable modification — no code changes
- All 11 surfaces must be documented — no partial credit

## Specification

**Objective:** Audit and add doc-comments to all 11 runtime-registry identity surfaces, explicitly documenting the Sprint-10 resolution order (registry → flag-gated env → denial) so the invariant is visible in code, not just in tests.

**Success state:** All 11 runtime-registry identity surfaces have preceding `///` doc-comments naming the resolution order. invariant_build_gates.rs includes preceding-doc assertions for resolution order keywords. cargo doc --no-deps -p but-api passes. Manual review confirms all 11 docs are accurate and specific.

## Acceptance Criteria

**AC-1 (PRIMARY)** — PRIMARY — commit/gate.rs has resolution order doc-comment
- **GIVEN:** commit/gate.rs: enforce_commit_gate_for_target function
- **WHEN:** Reading the function's doc-comment
- **THEN:** Doc-comment states: 'Gate resolution uses resolve_principal_with_runtime_registry; underlying order is (1) registry via but_authz::resolve_principal_with_registry, (2) env fallback if BUT_AUTHZ_ALLOW_ENV_HANDLE=1, (3) Denial::unregistered with perm.denied'
- **Verify:** `cargo test -p but-authz --test invariant_build_gates -- test_commit_gate_resolution_order_doc_precedes_function && cargo doc --no-deps -p but-api`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-api · **FLOW_REF:** UC-IDENT-03
- **Scenario:** `start_ref=commit_gate_doc_comment`; `must_observe` = ["Doc-comment contains literal 'resolve_principal_with_runtime_registry'", "Doc-comment contains literal 'but_authz::resolve_principal_with_registry'", "Doc-comment contains literal 'BUT_AUTHZ_ALLOW_ENV_HANDLE'", "Doc-comment contains literal 'Denial::unregistered'"]; `must_not_observe` = ['doc-comment missing', 'empty doc', 'omitted keywords']; `negative_control.would_fail_if` = ['Doc-comment not added (absent)', 'Doc-comment omits runtime wrapper or resolution order keywords', 'Grep stubbed or not executed', 'Keywords deleted or removed from doc'].

**AC-2** — legacy/merge_gate.rs has resolution order doc-comment
- **GIVEN:** legacy/merge_gate.rs: enforce_merge_gate function
- **WHEN:** Reading the function's doc-comment
- **THEN:** Doc-comment states the resolution order (registry → flag-gated env → denial)
- **Verify:** `cargo test -p but-authz --test invariant_build_gates -- test_merge_gate_resolution_order_doc_precedes_function`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-api · **FLOW_REF:** UC-IDENT-03
- **Scenario:** `start_ref=merge_gate_doc_comment`; `must_observe` = ["Doc-comment contains literal 'resolve_principal_with_runtime_registry'", "Doc-comment contains literal 'but_authz::resolve_principal_with_registry'", 'Resolution order described']; `must_not_observe` = ['doc-comment missing', 'empty doc', 'omitted keywords']; `negative_control.would_fail_if` = ['Doc-comment not added (absent)', 'Doc-comment omits runtime wrapper or resolution order keywords', 'Grep stubbed or not executed', 'Keywords deleted or removed from doc'].

**AC-3** — legacy/governance.rs (6 sites) have resolution order doc-comments
- **GIVEN:** legacy/governance.rs: 6 functions (governance_status_read, branch_gates_read_with_repo, group_list_with_repo, perm_list_with_repo, whoami_with_repo, can_i_with_repo)
- **WHEN:** Reading each function's doc-comment
- **THEN:** Each doc-comment states the resolution order, including discovery/query surfaces `whoami_with_repo` and `can_i_with_repo`
- **Verify:** `cargo test -p but-authz --test invariant_build_gates -- test_governance_resolution_order_docs_precede_functions`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-api · **FLOW_REF:** UC-IDENT-03
- **Scenario:** `start_ref=governance_doc_comments`; `must_observe` = ['6 of 6 governance functions have resolution order docs', 'grep finds docs at all 6 sites', 'docs include literal `whoami_with_repo` and `can_i_with_repo` surfaces']; `must_not_observe` = ['doc-comment missing', 'empty doc', 'omitted keywords', 'whoami/can-i omitted']; `negative_control.would_fail_if` = ['Doc-comment not added (absent)', 'Doc-comment omits resolution order keywords', 'Grep stubbed or not executed', 'Keywords deleted or removed from doc', 'discovery surfaces omitted as non-gate without invariant classification'].

**AC-4** — legacy/forge.rs has resolution order doc-comment
- **GIVEN:** legacy/forge.rs: authorize_branch_action function
- **WHEN:** Reading the function's doc-comment
- **THEN:** Doc-comment states the resolution order
- **Verify:** `cargo test -p but-authz --test invariant_build_gates -- test_forge_resolution_order_doc_precedes_function`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-api · **FLOW_REF:** UC-IDENT-03
- **Scenario:** `start_ref=forge_doc_comment`; `must_observe` = ["Doc-comment contains literal 'resolve_principal_with_runtime_registry'", "Doc-comment contains literal 'but_authz::resolve_principal_with_registry'", 'Resolution order described']; `must_not_observe` = ['doc-comment missing', 'empty doc', 'omitted keywords']; `negative_control.would_fail_if` = ['Doc-comment not added (absent)', 'Doc-comment omits runtime wrapper or resolution order keywords', 'Grep stubbed or not executed', 'Keywords deleted or removed from doc'].

**AC-5** — legacy/config_mutate.rs has resolution order doc-comment
- **GIVEN:** legacy/config_mutate.rs: enforce_administration_write_gate function
- **WHEN:** Reading the function's doc-comment
- **THEN:** Doc-comment states the resolution order
- **Verify:** `cargo test -p but-authz --test invariant_build_gates -- test_config_mutate_resolution_order_doc_precedes_function`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-api · **FLOW_REF:** UC-IDENT-03
- **Scenario:** `start_ref=config_mutate_doc_comment`; `must_observe` = ["Doc-comment contains literal 'resolve_principal_with_runtime_registry'", "Doc-comment contains literal 'but_authz::resolve_principal_with_registry'", 'Resolution order described']; `must_not_observe` = ['doc-comment missing', 'empty doc', 'omitted keywords']; `negative_control.would_fail_if` = ['Doc-comment not added (absent)', 'Doc-comment omits runtime wrapper or resolution order keywords', 'Grep stubbed or not executed', 'Keywords deleted or removed from doc'].

**AC-6** — legacy/rules.rs has resolution order doc-comment
- **GIVEN:** legacy/rules.rs: list_workspace_rules_scoped_for_caller function
- **WHEN:** Reading the function's doc-comment
- **THEN:** Doc-comment states the resolution order
- **Verify:** `cargo test -p but-authz --test invariant_build_gates -- test_rules_resolution_order_doc_precedes_function`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-api · **FLOW_REF:** UC-IDENT-03
- **Scenario:** `start_ref=rules_doc_comment`; `must_observe` = ["Doc-comment contains literal 'resolve_principal_with_runtime_registry'", "Doc-comment contains literal 'but_authz::resolve_principal_with_registry'", 'Resolution order described', "Doc-comment contains literal 'BUT_AUTHZ_ALLOW_ENV_HANDLE'", "Doc-comment contains literal 'Denial::unregistered'"]; `must_not_observe` = ['doc-comment missing', 'empty doc', 'omitted keywords']; `negative_control.would_fail_if` = ['Doc-comment not added (absent)', 'Doc-comment omits runtime wrapper or resolution order keywords', 'Grep stubbed or not executed', 'legacy/rules.rs omitted from the audit'].

**AC-7** — invariant_build_gates.rs includes preceding-doc assertion for all callsites
- **GIVEN:** invariant_build_gates.rs with Sprint-10 invariants (IDENT-020)
- **WHEN:** Adding a helper that collects contiguous `///` lines immediately before each target function
- **THEN:** Test checks each of the 11 preceding doc blocks for 'resolve_principal_with_runtime_registry' + 'but_authz::resolve_principal_with_registry' + 'BUT_AUTHZ_ALLOW_ENV_HANDLE' + 'Denial::unregistered'
- **Verify:** `cargo test -p but-authz --test invariant_build_gates -- test_resolution_order_documented`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-authz · **FLOW_REF:** UC-IDENT-03
- **Scenario:** `start_ref=invariant_build_gates_extended`; `must_observe` = ['preceding-doc assertion checks 11 function doc blocks', 'each doc block contains all 4 required literals']; `must_not_observe` = ['doc-comment after function declaration counted as valid', 'doc-comment missing', 'empty doc', 'omitted keywords', 'rules/whoami/can-i omitted']; `negative_control.would_fail_if` = ['Doc-comment not added (absent)', 'Doc-comment placed after the function and still counted', 'Doc-comment omits resolution order keywords', 'Keywords deleted or removed from doc', 'checker only validates 8 legacy callsites'].

**AC-8** — cargo doc passes with all doc-comments
- **GIVEN:** All 11 doc-comments added
- **WHEN:** Running cargo doc --no-deps -p but-api
- **THEN:** Documentation builds successfully with no warnings
- **Verify:** `cargo doc --no-deps -p but-api`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-api · **FLOW_REF:** None
- **Scenario:** `start_ref=doc_build`; `must_observe` = ['cargo doc exits with code 0', 'No doc warnings or errors']; `must_not_observe` = ['doc-comment missing', 'empty doc', 'omitted keywords']; `negative_control.would_fail_if` = ['Doc-comment not added (absent)', 'Doc-comment omits resolution order keywords', 'Grep stubbed or not executed', 'Keywords deleted or removed from doc'].

**AC-9** — Manual review confirms all 11 doc-comments are accurate
- **GIVEN:** All 11 doc-comments present
- **WHEN:** Manually reviewing each doc-comment
- **THEN:** All docs accurately describe the Sprint-10 resolution order with no misleading statements
- **Verify:** `Manual review of all 11 doc-comments`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-api · **FLOW_REF:** UC-IDENT-03
- **Scenario:** `start_ref=manual_doc_review`; `must_observe` = ['11 of 11 surfaces have accurate resolution order docs', '0 misleading statements found', 'literal `legacy/rules.rs`, `whoami_with_repo`, and `can_i_with_repo` are included']; `must_not_observe` = ['doc-comment missing', 'empty doc', 'omitted keywords', 'rules/whoami/can-i omitted']; `negative_control.would_fail_if` = ['Doc-comment not added (absent)', 'Doc-comment omits resolution order keywords', 'Grep stubbed or not executed', 'Keywords deleted or removed from doc', 'manual audit stops at 8 callsites'].

## Test Criteria

| ID | Boolean | Maps to AC |
|----|---------|------------|
| TC-1 | commit/gate.rs has resolution order doc is true | AC-1 |
| TC-2 | legacy/merge_gate.rs has resolution order doc is true | AC-2 |
| TC-3 | legacy/governance.rs (6 sites including whoami_with_repo and can_i_with_repo) have docs is true | AC-3 |
| TC-4 | legacy/forge.rs has resolution order doc is true | AC-4 |
| TC-5 | legacy/config_mutate.rs has resolution order doc is true | AC-5 |
| TC-6 | legacy/rules.rs has resolution order doc is true | AC-6 |
| TC-7 | invariant_build_gates includes doc assertion for all 11 surfaces is true | AC-7 |
| TC-8 | cargo doc passes is true | AC-8 |
| TC-9 | Manual review confirms accuracy for all 11 surfaces is true | AC-9 |

## Reading List

1. `crates/but-api/src/commit/gate.rs:54-78` — enforce_commit_gate_for_target — add/update doc-comment
2. `crates/but-api/src/legacy/merge_gate.rs:40-64` — enforce_merge_gate — add/update doc-comment
3. `crates/but-api/src/legacy/governance.rs:479-495,558-568,1158-1168,1459-1470,1629-1642,1680-1694` — 6 functions, including `whoami_with_repo` and `can_i_with_repo` — add/update doc-comments
4. `crates/but-api/src/legacy/forge.rs:47-68` — authorize_branch_action — add/update doc-comment
5. `crates/but-api/src/legacy/config_mutate.rs:18-28` — enforce_administration_write_gate — add/update doc-comment
6. `crates/but-api/src/legacy/rules.rs:91-105` — list_workspace_rules_scoped_for_caller — add/update doc-comment
7. `crates/but-authz/tests/invariant_build_gates.rs:1-100` — Add preceding-doc assertions for resolution order keywords

## Guardrails

**WRITE-ALLOWED:**
- crates/but-api/src/commit/gate.rs (MODIFY-DOC-ONLY — add/update doc-comment for enforce_commit_gate_for_target)
- crates/but-api/src/legacy/merge_gate.rs (MODIFY-DOC-ONLY — add/update doc-comment for enforce_merge_gate)
- crates/but-api/src/legacy/governance.rs (MODIFY-DOC-ONLY — add/update doc-comments for 6 functions)
- crates/but-api/src/legacy/forge.rs (MODIFY-DOC-ONLY — add/update doc-comment for authorize_branch_action)
- crates/but-api/src/legacy/config_mutate.rs (MODIFY-DOC-ONLY — add/update doc-comment for enforce_administration_write_gate)
- crates/but-api/src/legacy/rules.rs (MODIFY-DOC-ONLY — add/update doc-comment for list_workspace_rules_scoped_for_caller)
- crates/but-authz/tests/invariant_build_gates.rs (MODIFY — add preceding-doc assertions for resolution order keywords)

**WRITE-PROHIBITED:**
- Modifying function signatures or logic — doc-comments ONLY
- Skipping any of the 11 surfaces — all must be documented
- Modifying but-authz/src/** — only invariant_build_gates.rs test modification allowed

## Code Pattern

**Reference:** Sprint-09 IDENT-010 swapped callsites to `resolve_principal_with_runtime_registry`, whose wrapper delegates to `but_authz::resolve_principal_with_registry`; IDENT-017 documented direct resolver/env helpers as test-only where appropriate; IDENT-020 extended invariant_build_gates.rs

**Pattern:** Doc-comment pattern: Each gate function doc-comment states 'Gate resolution uses resolve_principal_with_runtime_registry; underlying order: (1) registry via but_authz::resolve_principal_with_registry, (2) env fallback if BUT_AUTHZ_ALLOW_ENV_HANDLE=1, (3) Denial::unregistered with code perm.denied'. Context-specific details allowed, but the runtime wrapper and underlying resolution order must be explicit.

**Source:** `Existing but-api doc-comment patterns — same discipline, new resolution order`

**Design notes:**
- Doc-comments are the PRIMARY user-facing documentation of the resolution order
- The invariant_build_gates preceding-doc assertion ensures docs stay in sync with code
- All 11 surfaces must have docs — no partial credit

**Anti-pattern:** Do NOT use vague language like 'resolves identity'. Do NOT skip callsites. Do NOT modify function code.

## Agent Instructions

TDD RED→GREEN per AC (integration against the real crate — `but-authz` / `but-api` — real git/gitoxide, NO mocks):
1. **RED:** write each AC's failing test first (against the live code / current start state).
2. **GREEN:** make the minimal change (test-only for IDENT-017/018/019; invariant assertions for IDENT-020; doc-comments for IDENT-021).
3. Run `cargo fmt`, `cargo clippy -p <crate> --all-targets -- -D warnings`, then the task's verify commands.
4. Commit via `but commit` (governed). Note: this task is BLOCKED-UNTIL Sprint-09 IDENT-009/010/011 land.

## Orchestrator Verification Protocol

- `cargo test -p but-authz --test invariant_build_gates -- test_commit_gate_resolution_order_doc_precedes_function` → exit 0
- `cargo test -p but-authz --test invariant_build_gates -- test_merge_gate_resolution_order_doc_precedes_function` → exit 0
- `cargo test -p but-authz --test invariant_build_gates -- test_governance_resolution_order_docs_precede_functions` → exit 0
- `cargo test -p but-authz --test invariant_build_gates -- test_forge_resolution_order_doc_precedes_function` → exit 0
- `cargo test -p but-authz --test invariant_build_gates -- test_config_mutate_resolution_order_doc_precedes_function` → exit 0
- `cargo test -p but-authz --test invariant_build_gates -- test_rules_resolution_order_doc_precedes_function` → exit 0
- `cargo test -p but-authz --test invariant_build_gates -- test_resolution_order_documented` → exit 0
- `cargo doc --no-deps -p but-api` → exit 0

## Agent Assignment

**Agent:** `rust-reviewer` — rust-reviewer owns documentation quality and must audit all 11 runtime-registry identity surfaces to ensure the resolution order is documented — this is review-qualified work requiring attention to doc-comment accuracy.
**Pairing:** none (single-surface Rust task). Honors `crates/AGENTS.md` + `crates/WORKSPACE_MODEL.md`.

## Evidence Gates

- `cargo test -p but-authz --test invariant_build_gates -- test_commit_gate_resolution_order_doc_precedes_function` (exit 0)
- `cargo test -p but-authz --test invariant_build_gates -- test_merge_gate_resolution_order_doc_precedes_function` (exit 0)
- `cargo test -p but-authz --test invariant_build_gates -- test_governance_resolution_order_docs_precede_functions` (exit 0)
- `cargo test -p but-authz --test invariant_build_gates -- test_forge_resolution_order_doc_precedes_function` (exit 0)
- `cargo test -p but-authz --test invariant_build_gates -- test_config_mutate_resolution_order_doc_precedes_function` (exit 0)
- `cargo test -p but-authz --test invariant_build_gates -- test_rules_resolution_order_doc_precedes_function` (exit 0)
- `cargo test -p but-authz --test invariant_build_gates -- test_resolution_order_documented` (exit 0)
- `cargo doc --no-deps -p but-api` (exit 0)

## Review Criteria

- AC-1: PRIMARY — commit/gate.rs has resolution order doc-comment — verified by `cargo test -p but-authz --test invariant_build_gates -- test_commit_gate_resolution_order_doc_precedes_function && cargo doc --no-deps -p but-api`.
- AC-2: legacy/merge_gate.rs has resolution order doc-comment — verified by `cargo test -p but-authz --test invariant_build_gates -- test_merge_gate_resolution_order_doc_precedes_function`.
- AC-3: legacy/governance.rs (6 sites, including whoami_with_repo and can_i_with_repo) have resolution order doc-comments — verified by `cargo test -p but-authz --test invariant_build_gates -- test_governance_resolution_order_docs_precede_functions`.
- AC-4: legacy/forge.rs has resolution order doc-comment — verified by `cargo test -p but-authz --test invariant_build_gates -- test_forge_resolution_order_doc_precedes_function`.
- AC-5: legacy/config_mutate.rs has resolution order doc-comment — verified by `cargo test -p but-authz --test invariant_build_gates -- test_config_mutate_resolution_order_doc_precedes_function`.
- AC-6: legacy/rules.rs has resolution order doc-comment — verified by `cargo test -p but-authz --test invariant_build_gates -- test_rules_resolution_order_doc_precedes_function`.
- AC-7: invariant_build_gates.rs includes preceding-doc assertion for all 11 surfaces — verified by `cargo test -p but-authz --test invariant_build_gates -- test_resolution_order_documented`.
- AC-8: cargo doc passes with all doc-comments — verified by `cargo doc --no-deps -p but-api`.
- AC-9: Manual review confirms all 11 doc-comments are accurate — verified by `Manual review of all 11 doc-comments`.
- Honors NEVER: Modify function signatures or logic — this is doc-comment ONLY.

## Dependencies

- **Depends on:** Sprint-09 IDENT-010 (callsites must be swapped to `resolve_principal_with_runtime_registry` and wrapper delegation must exist), IDENT-017 (resolver documented), IDENT-020 (invariant gates extended)
- **Blocks:** none
- **Capabilities:** CAP-AUTHZ-01

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "IDENT-021",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "commit_gate_doc_comment": {
      "description": "commit/gate.rs enforce_commit_gate_for_target function with preceding doc-comment",
      "seed_method": "public_api",
      "records": [
        "enforce_commit_gate_for_target function exists",
        "Doc-comment precedes the function",
        "Doc-comment must describe registry -> flag-gated env -> Denial::unregistered"
      ]
    },
    "merge_gate_doc_comment": {
      "description": "legacy/merge_gate.rs enforce_merge_gate function with preceding doc-comment",
      "seed_method": "public_api",
      "records": [
        "enforce_merge_gate function exists",
        "Doc-comment precedes the function",
        "Doc-comment must describe registry -> flag-gated env -> Denial::unregistered"
      ]
    },
    "governance_doc_comments": {
      "description": "legacy/governance.rs with six runtime-registry identity surfaces needing preceding doc-comments",
      "seed_method": "public_api",
      "records": [
        "governance_status_read function exists",
        "branch_gates_read_with_repo function exists",
        "group_list_with_repo function exists",
        "perm_list_with_repo function exists",
        "whoami_with_repo function exists",
        "can_i_with_repo function exists"
      ]
    },
    "forge_doc_comment": {
      "description": "legacy/forge.rs authorize_branch_action function with preceding doc-comment",
      "seed_method": "public_api",
      "records": [
        "authorize_branch_action function exists",
        "Doc-comment precedes the function"
      ]
    },
    "config_mutate_doc_comment": {
      "description": "legacy/config_mutate.rs enforce_administration_write_gate function with preceding doc-comment",
      "seed_method": "public_api",
      "records": [
        "enforce_administration_write_gate function exists",
        "Doc-comment precedes the function"
      ]
    },
    "rules_doc_comment": {
      "description": "legacy/rules.rs list_workspace_rules_scoped_for_caller function with preceding doc-comment",
      "seed_method": "public_api",
      "records": [
        "list_workspace_rules_scoped_for_caller function exists",
        "Doc-comment precedes the function"
      ]
    },
    "invariant_build_gates_extended": {
      "description": "invariant_build_gates.rs with preceding-doc resolution order checks for all 11 surfaces",
      "seed_method": "public_api",
      "records": [
        "checker collects contiguous /// lines immediately before each target function",
        "checker asserts 11 target doc blocks",
        "checker requires resolve_principal_with_runtime_registry + but_authz::resolve_principal_with_registry + BUT_AUTHZ_ALLOW_ENV_HANDLE + Denial::unregistered"
      ]
    },
    "doc_build": {
      "description": "cargo doc build output for but-api after all 11 doc-comments are present",
      "seed_method": "public_api",
      "records": [
        "cargo doc --no-deps -p but-api will be run",
        "All 11 doc-comments will be present"
      ]
    },
    "manual_doc_review": {
      "description": "manual review of all 11 runtime-registry identity surface doc-comments",
      "seed_method": "public_api",
      "records": [
        "11 doc-comments exist",
        "rules, whoami_with_repo, and can_i_with_repo are included",
        "Each doc-comment describes the resolution order"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "PRIMARY \u2014 commit/gate.rs has resolution order doc-comment",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-authz --test invariant_build_gates -- test_commit_gate_resolution_order_doc_precedes_function && cargo doc --no-deps -p but-api",
      "flow_ref": "UC-IDENT-03",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "commit_gate_doc_comment",
        "must_observe": [
          "Doc-comment contains literal `resolve_principal_with_runtime_registry`",
          "Doc-comment contains literal `but_authz::resolve_principal_with_registry`",
          "Doc-comment contains literal `BUT_AUTHZ_ALLOW_ENV_HANDLE`",
          "Doc-comment contains literal `Denial::unregistered`"
        ],
        "must_not_observe": [
          "doc-comment missing",
          "empty doc",
          "0 keyword matches"
        ],
        "negative_control": {
          "would_fail_if": [
            "Doc-comment not added (absent)",
            "Doc-comment omits resolution order keywords",
            "Grep stubbed or not executed",
            "Keywords deleted or removed from doc"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "commit_gate_doc_comment",
            "action": {
              "actor": "test_harness",
              "steps": [
                "Read enforce_commit_gate_for_target preceding doc-comment",
                "Verify all 4 resolution order keywords"
              ]
            },
            "end_state": {
              "must_observe": [
                "Doc-comment contains literal `resolve_principal_with_runtime_registry`",
          "Doc-comment contains literal `but_authz::resolve_principal_with_registry`",
                "Doc-comment contains literal `BUT_AUTHZ_ALLOW_ENV_HANDLE`",
                "Doc-comment contains literal `Denial::unregistered`"
              ],
              "must_not_observe": [
                "doc-comment missing",
                "empty doc",
                "0 keyword matches"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "legacy/merge_gate.rs has resolution order doc-comment",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-authz --test invariant_build_gates -- test_merge_gate_resolution_order_doc_precedes_function",
      "flow_ref": "UC-IDENT-03",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "merge_gate_doc_comment",
        "must_observe": [
          "Doc-comment contains literal `resolve_principal_with_runtime_registry`",
          "Doc-comment contains literal `but_authz::resolve_principal_with_registry`",
          "Doc-comment contains literal `BUT_AUTHZ_ALLOW_ENV_HANDLE`",
          "Doc-comment contains literal `Denial::unregistered`"
        ],
        "must_not_observe": [
          "doc-comment missing",
          "empty doc",
          "0 keyword matches"
        ],
        "negative_control": {
          "would_fail_if": [
            "Doc-comment not added (absent)",
            "Doc-comment omits resolution order keywords",
            "Grep stubbed or not executed",
            "Keywords deleted or removed from doc"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "merge_gate_doc_comment",
            "action": {
              "actor": "test_harness",
              "steps": [
                "Read enforce_merge_gate preceding doc-comment",
                "Verify all 4 resolution order keywords"
              ]
            },
            "end_state": {
              "must_observe": [
                "Doc-comment contains literal `resolve_principal_with_runtime_registry`",
          "Doc-comment contains literal `but_authz::resolve_principal_with_registry`",
                "Doc-comment contains literal `BUT_AUTHZ_ALLOW_ENV_HANDLE`",
                "Doc-comment contains literal `Denial::unregistered`"
              ],
              "must_not_observe": [
                "doc-comment missing",
                "empty doc",
                "0 keyword matches"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "legacy/governance.rs (6 sites, including whoami_with_repo and can_i_with_repo) have resolution order doc-comments",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-authz --test invariant_build_gates -- test_governance_resolution_order_docs_precede_functions",
      "flow_ref": "UC-IDENT-03",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "governance_doc_comments",
        "must_observe": [
          "6 of 6 governance functions have resolution order docs",
          "docs include literal `whoami_with_repo` surface",
          "docs include literal `can_i_with_repo` surface",
          "each doc block contains literal `resolve_principal_with_runtime_registry`",
          "each doc block contains literal `but_authz::resolve_principal_with_registry`",
          "each doc block contains literal `Denial::unregistered`"
        ],
        "must_not_observe": [
          "doc-comment missing",
          "empty doc",
          "whoami/can-i omitted",
          "0 keyword matches"
        ],
        "negative_control": {
          "would_fail_if": [
            "Doc-comment not added (absent)",
            "Doc-comment omits resolution order keywords",
            "Grep stubbed or not executed",
            "Keywords deleted or removed from doc",
            "discovery surfaces omitted as non-gate without invariant classification"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "governance_doc_comments",
            "action": {
              "actor": "test_harness",
              "steps": [
                "Read six governance function preceding doc-comments",
                "Verify whoami_with_repo and can_i_with_repo are included",
                "Verify all 4 resolution order keywords per doc block"
              ]
            },
            "end_state": {
              "must_observe": [
                "6 of 6 governance functions have resolution order docs",
                "docs include literal `whoami_with_repo` surface",
                "docs include literal `can_i_with_repo` surface",
                "each doc block contains literal `resolve_principal_with_runtime_registry`",
                "each doc block contains literal `but_authz::resolve_principal_with_registry`",
                "each doc block contains literal `Denial::unregistered`"
              ],
              "must_not_observe": [
                "doc-comment missing",
                "empty doc",
                "whoami/can-i omitted",
                "0 keyword matches"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "legacy/forge.rs has resolution order doc-comment",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-authz --test invariant_build_gates -- test_forge_resolution_order_doc_precedes_function",
      "flow_ref": "UC-IDENT-03",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "forge_doc_comment",
        "must_observe": [
          "Doc-comment contains literal `resolve_principal_with_runtime_registry`",
          "Doc-comment contains literal `but_authz::resolve_principal_with_registry`",
          "Doc-comment contains literal `BUT_AUTHZ_ALLOW_ENV_HANDLE`",
          "Doc-comment contains literal `Denial::unregistered`"
        ],
        "must_not_observe": [
          "doc-comment missing",
          "empty doc",
          "0 keyword matches"
        ],
        "negative_control": {
          "would_fail_if": [
            "Doc-comment not added (absent)",
            "Doc-comment omits resolution order keywords",
            "Grep stubbed or not executed",
            "Keywords deleted or removed from doc"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "forge_doc_comment",
            "action": {
              "actor": "test_harness",
              "steps": [
                "Read authorize_branch_action preceding doc-comment",
                "Verify all 4 resolution order keywords"
              ]
            },
            "end_state": {
              "must_observe": [
                "Doc-comment contains literal `resolve_principal_with_runtime_registry`",
          "Doc-comment contains literal `but_authz::resolve_principal_with_registry`",
                "Doc-comment contains literal `BUT_AUTHZ_ALLOW_ENV_HANDLE`",
                "Doc-comment contains literal `Denial::unregistered`"
              ],
              "must_not_observe": [
                "doc-comment missing",
                "empty doc",
                "0 keyword matches"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "legacy/config_mutate.rs has resolution order doc-comment",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-authz --test invariant_build_gates -- test_config_mutate_resolution_order_doc_precedes_function",
      "flow_ref": "UC-IDENT-03",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "config_mutate_doc_comment",
        "must_observe": [
          "Doc-comment contains literal `resolve_principal_with_runtime_registry`",
          "Doc-comment contains literal `but_authz::resolve_principal_with_registry`",
          "Doc-comment contains literal `BUT_AUTHZ_ALLOW_ENV_HANDLE`",
          "Doc-comment contains literal `Denial::unregistered`"
        ],
        "must_not_observe": [
          "doc-comment missing",
          "empty doc",
          "0 keyword matches"
        ],
        "negative_control": {
          "would_fail_if": [
            "Doc-comment not added (absent)",
            "Doc-comment omits resolution order keywords",
            "Grep stubbed or not executed",
            "Keywords deleted or removed from doc"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "config_mutate_doc_comment",
            "action": {
              "actor": "test_harness",
              "steps": [
                "Read enforce_administration_write_gate preceding doc-comment",
                "Verify all 4 resolution order keywords"
              ]
            },
            "end_state": {
              "must_observe": [
                "Doc-comment contains literal `resolve_principal_with_runtime_registry`",
          "Doc-comment contains literal `but_authz::resolve_principal_with_registry`",
                "Doc-comment contains literal `BUT_AUTHZ_ALLOW_ENV_HANDLE`",
                "Doc-comment contains literal `Denial::unregistered`"
              ],
              "must_not_observe": [
                "doc-comment missing",
                "empty doc",
                "0 keyword matches"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-6",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "legacy/rules.rs has resolution order doc-comment",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo test -p but-authz --test invariant_build_gates -- test_rules_resolution_order_doc_precedes_function",
      "flow_ref": "UC-IDENT-03",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "rules_doc_comment",
        "must_observe": [
          "Doc-comment contains literal `resolve_principal_with_runtime_registry`",
          "Doc-comment contains literal `but_authz::resolve_principal_with_registry`",
          "Doc-comment contains literal `BUT_AUTHZ_ALLOW_ENV_HANDLE`",
          "Doc-comment contains literal `Denial::unregistered`",
          "Doc-comment names literal `list_workspace_rules_scoped_for_caller`"
        ],
        "must_not_observe": [
          "doc-comment missing",
          "empty doc",
          "0 keyword matches"
        ],
        "negative_control": {
          "would_fail_if": [
            "Doc-comment not added (absent)",
            "Doc-comment omits resolution order keywords",
            "Grep stubbed or not executed",
            "Keywords deleted or removed from doc",
            "legacy/rules.rs omitted from the audit"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "rules_doc_comment",
            "action": {
              "actor": "test_harness",
              "steps": [
                "Read list_workspace_rules_scoped_for_caller preceding doc-comment",
                "Verify all 4 resolution order keywords"
              ]
            },
            "end_state": {
              "must_observe": [
                "Doc-comment contains literal `resolve_principal_with_runtime_registry`",
          "Doc-comment contains literal `but_authz::resolve_principal_with_registry`",
                "Doc-comment contains literal `BUT_AUTHZ_ALLOW_ENV_HANDLE`",
                "Doc-comment contains literal `Denial::unregistered`",
                "Doc-comment names literal `list_workspace_rules_scoped_for_caller`"
              ],
              "must_not_observe": [
                "doc-comment missing",
                "empty doc",
                "0 keyword matches"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-7",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "invariant_build_gates.rs includes preceding-doc assertion for all 11 surfaces",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "verify": "cargo test -p but-authz --test invariant_build_gates -- test_resolution_order_documented",
      "flow_ref": "UC-IDENT-03",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "invariant_build_gates_extended",
        "must_observe": [
          "preceding-doc assertion checks exactly 11 function doc blocks",
          "each doc block contains literal `resolve_principal_with_runtime_registry`",
          "each doc block contains literal `but_authz::resolve_principal_with_registry`",
          "each doc block contains literal `BUT_AUTHZ_ALLOW_ENV_HANDLE`",
          "each doc block contains literal `Denial::unregistered`"
        ],
        "must_not_observe": [
          "doc-comment after function declaration counted as valid",
          "doc-comment missing",
          "empty doc",
          "rules/whoami/can-i omitted",
          "0 checked doc blocks"
        ],
        "negative_control": {
          "would_fail_if": [
            "Doc-comment not added (absent)",
            "Doc-comment omits resolution order keywords",
            "Grep stubbed or not executed",
            "Keywords deleted or removed from doc",
            "checker only validates 8 legacy callsites"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "invariant_build_gates_extended",
            "action": {
              "actor": "test_harness",
              "steps": [
                "Collect contiguous /// lines immediately before each target function",
                "Assert exactly 11 target function docs",
                "Verify required keywords in each doc block"
              ]
            },
            "end_state": {
              "must_observe": [
                "preceding-doc assertion checks exactly 11 function doc blocks",
                "each doc block contains literal `resolve_principal_with_runtime_registry`",
          "each doc block contains literal `but_authz::resolve_principal_with_registry`",
                "each doc block contains literal `BUT_AUTHZ_ALLOW_ENV_HANDLE`",
                "each doc block contains literal `Denial::unregistered`"
              ],
              "must_not_observe": [
                "doc-comment after function declaration counted as valid",
                "doc-comment missing",
                "empty doc",
                "rules/whoami/can-i omitted",
                "0 checked doc blocks"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-8",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "cargo doc passes with all doc-comments",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "cargo doc --no-deps -p but-api",
      "flow_ref": null,
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "doc_build",
        "must_observe": [
          "cargo doc exits with code 0",
          "documentation build includes 11 runtime-registry identity surface doc-comments"
        ],
        "must_not_observe": [
          "doc build exits non-zero",
          "doc-comment missing",
          "empty doc",
          "0 generated docs"
        ],
        "negative_control": {
          "would_fail_if": [
            "Doc-comment syntax breaks rustdoc",
            "Doc-comment omitted from a public surface",
            "cargo doc command skipped"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "doc_build",
            "action": {
              "actor": "test_harness",
              "steps": [
                "Run cargo doc --no-deps -p but-api",
                "Capture exit code and stderr"
              ]
            },
            "end_state": {
              "must_observe": [
                "cargo doc exits with code 0",
                "documentation build includes 11 runtime-registry identity surface doc-comments"
              ],
              "must_not_observe": [
                "doc build exits non-zero",
                "doc-comment missing",
                "empty doc",
                "0 generated docs"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-9",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "Manual review confirms all 11 doc-comments are accurate",
      "test_tier": "integration",
      "verification_service": "but-api",
      "verify": "Manual review of all 11 doc-comments",
      "flow_ref": "UC-IDENT-03",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-api",
        "start_ref": "manual_doc_review",
        "must_observe": [
          "11 of 11 surfaces have accurate resolution order docs",
          "0 misleading statements found",
          "literal `legacy/rules.rs`, `whoami_with_repo`, and `can_i_with_repo` are included"
        ],
        "must_not_observe": [
          "manual audit stops at 8 callsites",
          "doc-comment missing",
          "empty doc",
          "rules/whoami/can-i omitted"
        ],
        "negative_control": {
          "would_fail_if": [
            "Doc-comment not added (absent)",
            "Doc-comment omits resolution order keywords",
            "Grep stubbed or not executed",
            "Keywords deleted or removed from doc",
            "manual audit stops at 8 callsites"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "manual_doc_review",
            "action": {
              "actor": "test_harness",
              "steps": [
                "Review all 11 doc-comments",
                "Confirm resolution order and no production-env fallback misstatement"
              ]
            },
            "end_state": {
              "must_observe": [
                "11 of 11 surfaces have accurate resolution order docs",
                "0 misleading statements found",
                "literal `legacy/rules.rs`, `whoami_with_repo`, and `can_i_with_repo` are included"
              ],
              "must_not_observe": [
                "manual audit stops at 8 callsites",
                "doc-comment missing",
                "empty doc",
                "rules/whoami/can-i omitted"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "commit/gate.rs has resolution order doc-comment is true",
      "maps_to_ac": "AC-1",
      "verify": "cargo test -p but-authz --test invariant_build_gates -- test_commit_gate_resolution_order_doc_precedes_function && cargo doc --no-deps -p but-api"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "legacy/merge_gate.rs has resolution order doc-comment is true",
      "maps_to_ac": "AC-2",
      "verify": "cargo test -p but-authz --test invariant_build_gates -- test_merge_gate_resolution_order_doc_precedes_function"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "legacy/governance.rs (6 sites, including whoami_with_repo and can_i_with_repo) have resolution order doc-comments is true",
      "maps_to_ac": "AC-3",
      "verify": "cargo test -p but-authz --test invariant_build_gates -- test_governance_resolution_order_docs_precede_functions"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "legacy/forge.rs has resolution order doc-comment is true",
      "maps_to_ac": "AC-4",
      "verify": "cargo test -p but-authz --test invariant_build_gates -- test_forge_resolution_order_doc_precedes_function"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "legacy/config_mutate.rs has resolution order doc-comment is true",
      "maps_to_ac": "AC-5",
      "verify": "cargo test -p but-authz --test invariant_build_gates -- test_config_mutate_resolution_order_doc_precedes_function"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "description": "legacy/rules.rs has resolution order doc-comment is true",
      "maps_to_ac": "AC-6",
      "verify": "cargo test -p but-authz --test invariant_build_gates -- test_rules_resolution_order_doc_precedes_function"
    },
    {
      "id": "TC-7",
      "type": "test_criterion",
      "description": "invariant_build_gates.rs includes preceding-doc assertion for all 11 surfaces is true",
      "maps_to_ac": "AC-7",
      "verify": "cargo test -p but-authz --test invariant_build_gates -- test_resolution_order_documented"
    },
    {
      "id": "TC-8",
      "type": "test_criterion",
      "description": "cargo doc passes with all doc-comments is true",
      "maps_to_ac": "AC-8",
      "verify": "cargo doc --no-deps -p but-api"
    },
    {
      "id": "TC-9",
      "type": "test_criterion",
      "description": "Manual review confirms all 11 doc-comments are accurate is true",
      "maps_to_ac": "AC-9",
      "verify": "Manual review of all 11 doc-comments"
    }
  ]
}
-->
