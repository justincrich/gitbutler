# CLI-REM-003: Prove CLI perm/group resolve workspace target ref when HEAD differs

## What this does

Adds explicit CLI-level coverage that `but perm` and `but group` authorize against the workspace target ref, not the current checkout `HEAD`. This remediation pins the self-escalation boundary for real CLI commands when a feature branch head differs from protected `refs/heads/main`.

## Why

The red-hat review identified that the original tasks repeatedly described target-ref authorization, but did not force the CLI command layer to pass the workspace target ref into `but-api`. A command implementation could accidentally authorize from `HEAD`, allowing a feature-branch self-grant to unlock governance writes before the grant is committed to the target branch.

## How to verify

PRIMARY **AC-1** — `cargo test -p but perm_cli_uses_workspace_target_ref_not_head`.

## Scope

- `crates/but/src/command/perm.rs` (MODIFY) — ensure `but perm` resolves and passes the workspace target ref.
- `crates/but/src/command/group.rs` (MODIFY) — ensure `but group` resolves and passes the workspace target ref.
- `crates/but/tests/but/command/perm.rs` (MODIFY) — add CLI target-ref tests for permission commands.
- `crates/but/tests/but/command/group.rs` (MODIFY) — add CLI target-ref tests for group commands.

`crates/but-api/src/legacy/governance.rs` is write-prohibited unless the existing public functions do not accept an explicit `target_ref`. The intended fix is command-layer target-ref resolution, not an API authorization rewrite.

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: CLI-REM-003 - Prove CLI target-ref resolution when HEAD differs
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     S (75 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-AUTHZ-01, UC-GRPS-01, UC-GRPS-02
CAPABILITIES: CAP-AUTHZ-01, CAP-CONFIG-01

RUNTIME_COMMANDS:
  test:  cargo test -p but perm_cli_uses_workspace_target_ref_not_head
         cargo test -p but group_cli_uses_workspace_target_ref_not_head
         cargo test -p but group_cli_denies_using_workspace_target_when_head_self_grants
  check: cargo check -p but --all-targets
  fmt:   cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
The CLI command layer cannot pass unless `but perm` and `but group` both resolve the workspace target ref and remain immune to a feature-head self-grant.

--------------------------------------------------------------------------------
CRITICAL CONSTRAINTS
--------------------------------------------------------------------------------
- [MUST] MUST create a fixture where checkout `HEAD` differs from the workspace target ref.
- [MUST] MUST prove `but perm` authorization and writes use target `refs/heads/main`, not feature `HEAD`.
- [MUST] MUST prove `but group` authorization and writes use target `refs/heads/main`, not feature `HEAD`.
- [MUST] MUST prove a feature-head self-grant of `administration:write` cannot authorize a group write.
- [NEVER] NEVER infer authorization from the current checkout branch when a workspace target ref exists.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [x] AC-1 [PRIMARY]: `but perm` uses workspace target ref instead of HEAD.
- [x] AC-2: `but group` uses workspace target ref instead of HEAD.
- [x] AC-3: feature HEAD self-grant cannot authorize group grant when target ref lacks admin.
- [x] All verification gates pass; only writeAllowed files modified.

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA
--------------------------------------------------------------------------------

AC-1: but perm uses workspace target ref, not HEAD [PRIMARY]
  GIVEN: checkout is on `feature/self-admin`, but workspace target ref is `refs/heads/main`; main grants admin to `admin` and HEAD contains a different governance blob.
  WHEN: `BUT_AGENT_HANDLE=admin but perm grant --principal rust-reviewer reviews:write` runs from the feature checkout.
  THEN: authorization succeeds because `admin` is admin at `refs/heads/main`; the write lands in the working tree with the ref-pin caveat; `refs/heads/main` object id remains unchanged.
  TEST_TIER: integration   VERIFICATION_SERVICE: real but CLI snapbox command + real git refs
  VERIFY: cargo test -p but perm_cli_uses_workspace_target_ref_not_head

AC-2: but group uses workspace target ref, not HEAD
  GIVEN: the same HEAD-differs-target fixture.
  WHEN: `BUT_AGENT_HANDLE=admin but group add-member maintainers --principal rust-reviewer` runs from feature checkout.
  THEN: authorization succeeds from target `refs/heads/main`, the working-tree config changes only there, and the target ref id remains unchanged.
  TEST_TIER: integration   VERIFICATION_SERVICE: real but CLI snapbox command + real git refs
  VERIFY: cargo test -p but group_cli_uses_workspace_target_ref_not_head

AC-3: HEAD self-grant cannot authorize group grant
  GIVEN: `refs/heads/main` does not grant `rust-reviewer` `administration:write`, but feature HEAD adds a self-grant for `rust-reviewer`.
  WHEN: `BUT_AGENT_HANDLE=rust-reviewer but group grant maintainers reviews:write` runs from the feature checkout.
  THEN: the command is denied `perm.denied`, includes non-empty `remediation_hint`, and the working-tree file is unchanged.
  TEST_TIER: integration   VERIFICATION_SERVICE: real but CLI snapbox command + real but-authz target-ref load
  VERIFY: cargo test -p but group_cli_denies_using_workspace_target_when_head_self_grants

--------------------------------------------------------------------------------
TEST CRITERIA
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): perm CLI succeeds for target-ref admin while HEAD differs. VERIFY: `cargo test -p but perm_cli_uses_workspace_target_ref_not_head`
- TC-2 (-> AC-2): group CLI succeeds for target-ref admin while HEAD differs. VERIFY: `cargo test -p but group_cli_uses_workspace_target_ref_not_head`
- TC-3 (-> AC-3): group CLI denies feature-head self-grant. VERIFY: `cargo test -p but group_cli_denies_using_workspace_target_when_head_self_grants`

--------------------------------------------------------------------------------
SCOPE
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but/src/command/perm.rs (MODIFY)
  - crates/but/src/command/group.rs (MODIFY)
  - crates/but/tests/but/command/perm.rs (MODIFY)
  - crates/but/tests/but/command/group.rs (MODIFY)
writeProhibited:
  - crates/but-api/src/legacy/governance.rs unless the function signatures lack explicit target_ref.
  - Any authz-layer change that reads HEAD instead of target ref.
  - Any unrelated UI, Tauri, SDK, or generated file.

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but/src/command/perm.rs (full)
   Focus: how the command resolves workspace/project and target ref before calling but-api.
2. crates/but/src/command/group.rs (full)
   Focus: same target-ref path for group verbs.
3. crates/but/tests/but/command/perm.rs and crates/but/tests/but/command/group.rs
   Focus: existing snapbox fixtures and env.but usage.
4. crates/but/AGENTS.md
   Focus: CLI test economy and sandbox helper rules.
5. .spec/reviews/red-hat-20260620T051414Z.md
   Focus: CLI target-ref resolution finding.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- `cargo test -p but perm_cli_uses_workspace_target_ref_not_head` -> Exit 0.
- `cargo test -p but group_cli_uses_workspace_target_ref_not_head` -> Exit 0.
- `cargo test -p but group_cli_denies_using_workspace_target_when_head_self_grants` -> Exit 0.
- `cargo check -p but --all-targets` -> Exit 0.
- `cargo fmt --check` -> Exit 0.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — owns CLI target-ref wiring and snapbox tests.
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
  "task_id": "CLI-REM-003",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "head_differs_target_main": {
      "description": "Real CLI scenario where checkout HEAD is feature/self-admin while the workspace target ref remains refs/heads/main. Main contains committed governance granting admin administration:write; feature HEAD contains a different governance blob so HEAD reads would behave differently.",
      "seed_method": "cli",
      "records": [
        "create writable scenario and commit .gitbutler/permissions.toml on refs/heads/main with admin permissions=[\"administration:write\"] and maintainers group",
        "create branch feature/self-admin from main and edit .gitbutler/permissions.toml to make HEAD visibly different",
        "configure the GitButler workspace target branch as refs/heads/main",
        "capture ref_id(repo, \"refs/heads/main\") before CLI commands"
      ]
    },
    "head_self_grants_non_admin": {
      "description": "Real CLI scenario where refs/heads/main lacks rust-reviewer administration:write, while feature/self-admin HEAD adds a self-grant for rust-reviewer. Correct target-ref authorization must deny rust-reviewer.",
      "seed_method": "cli",
      "records": [
        "commit main governance with admin administration:write and rust-reviewer reviews:read only",
        "checkout feature/self-admin and add rust-reviewer administration:write only on feature HEAD",
        "configure workspace target branch as refs/heads/main",
        "capture working-tree permissions text before denied group grant"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "but perm resolves the workspace target ref rather than HEAD when checkout branch differs.",
      "verify": "cargo test -p but perm_cli_uses_workspace_target_ref_not_head",
      "scenario": {
        "id": "AC-1",
        "test_tier": "integration",
        "negative_control": {
          "would_fail_if": [
            "perm command passes HEAD as target_ref",
            "target ref id changes during CLI write",
            "a stub command prints the caveat without writing permissions"
          ]
        },
        "evidence": { "artifact_type": "stdout", "required_capture": true },
        "cases": [
          {
            "start_ref": "head_differs_target_main",
            "action": {
              "steps": [
                "set BUT_AGENT_HANDLE=admin",
                "run env.but([\"perm\", \"grant\", \"--principal\", \"rust-reviewer\", \"reviews:write\"])",
                "read working-tree .gitbutler/permissions.toml",
                "capture ref_id(repo, \"refs/heads/main\") after command"
              ]
            },
            "end_state": {
              "must_observe": [
                "CLI exits `0`",
                "stdout contains `takes effect once committed to the target branch`",
                "working-tree rust-reviewer permissions include `reviews:write`",
                "refs/heads/main ref_id after == ref_id before"
              ],
              "must_not_observe": [
                "no HEAD authorization dependency",
                "no target-ref object change",
                "empty working-tree write"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "but group resolves the workspace target ref rather than HEAD when checkout branch differs.",
      "verify": "cargo test -p but group_cli_uses_workspace_target_ref_not_head",
      "scenario": {
        "id": "AC-2",
        "test_tier": "integration",
        "negative_control": {
          "would_fail_if": [
            "group command passes HEAD as target_ref",
            "group add-member only modifies an in-memory stub",
            "target ref id changes during CLI write"
          ]
        },
        "evidence": { "artifact_type": "stdout", "required_capture": true },
        "cases": [
          {
            "start_ref": "head_differs_target_main",
            "action": {
              "steps": [
                "set BUT_AGENT_HANDLE=admin",
                "run env.but([\"group\", \"add-member\", \"maintainers\", \"--principal\", \"rust-reviewer\"])",
                "read working-tree .gitbutler/permissions.toml",
                "capture ref_id(repo, \"refs/heads/main\") after command"
              ]
            },
            "end_state": {
              "must_observe": [
                "CLI exits `0`",
                "stdout contains `takes effect once committed to the target branch`",
                "working-tree maintainers members include `rust-reviewer`",
                "refs/heads/main ref_id after == ref_id before"
              ],
              "must_not_observe": [
                "no HEAD authorization dependency",
                "no target-ref object change",
                "empty member write"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "A feature HEAD self-grant does not authorize group writes when the workspace target ref lacks the grant.",
      "verify": "cargo test -p but group_cli_denies_using_workspace_target_when_head_self_grants",
      "scenario": {
        "id": "AC-3",
        "test_tier": "integration",
        "negative_control": {
          "would_fail_if": [
            "a stub target-ref resolver returns HEAD instead of the workspace target ref",
            "authorization reads feature HEAD permissions",
            "self-grant on HEAD unlocks administration:write",
            "denied group grant mutates the file"
          ]
        },
        "evidence": { "artifact_type": "stderr", "required_capture": true },
        "cases": [
          {
            "start_ref": "head_self_grants_non_admin",
            "action": {
              "steps": [
                "capture working-tree permissions text",
                "set BUT_AGENT_HANDLE=rust-reviewer",
                "run env.but([\"group\", \"grant\", \"maintainers\", \"reviews:write\"])",
                "read working-tree permissions text after command"
              ]
            },
            "end_state": {
              "must_observe": [
                "CLI exits `1`",
                "stderr contains `perm.denied`",
                "stderr contains non-empty `remediation_hint`",
                "working-tree text after == text before"
              ],
              "must_not_observe": [
                "no group grant from HEAD self-grant",
                "no working-tree mutation",
                "empty remediation_hint"
              ]
            }
          }
        ]
      }
    },
    { "id": "TC-1", "type": "test_criterion", "description": "perm CLI succeeds for target-ref admin while HEAD differs", "verify": "cargo test -p but perm_cli_uses_workspace_target_ref_not_head", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "group CLI succeeds for target-ref admin while HEAD differs", "verify": "cargo test -p but group_cli_uses_workspace_target_ref_not_head", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "group CLI denies feature-head self-grant", "verify": "cargo test -p but group_cli_denies_using_workspace_target_when_head_self_grants", "maps_to_ac": "AC-3" }
  ]
}
-->
