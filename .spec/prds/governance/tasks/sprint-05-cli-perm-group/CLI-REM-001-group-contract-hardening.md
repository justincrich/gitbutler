# CLI-REM-001: Close group verb contract gaps: denial matrix, remove-member, delegated admin, duplicate create, no delete

## What this does

Hardens the Sprint 05 `but group` contract so every mutating group verb has executable coverage, not prose-only coverage. This remediation covers the red-hat gaps for the non-admin denial matrix, positive `group_remove_member`, delegated admin via `group_grant administration:write`, duplicate create, and the Sprint 05 no-delete boundary.

## Why

The red-hat review found that CLI-002 claims coverage for all mutating group verbs while its concrete denial path only exercises `group_add_member`. A bad implementation could leave `group_create`, `group_grant`, or `group_remove_member` ungated, or ship a no-op `group_remove_member`, and still pass the original task.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-api group_ops_non_admin_denied_all_mutating_verbs`.

## Scope

- `crates/but-api/src/legacy/governance.rs` (MODIFY if needed) — keep all group governance behavior inside the existing Sprint 05 boundary.
- `crates/but-api/tests/group_governance.rs` (MODIFY) — add the API-level integration tests named below.
- `crates/but/tests/but/command/group.rs` (MODIFY) — add CLI no-delete / denial rendering assertions where needed.
- `.spec/prds/governance/tasks/sprint-05-cli-perm-group/SPRINT.md` (MODIFY) — remove the old allowance for a Sprint 05 delete stub.

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: CLI-REM-001 - Close group verb contract gaps
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     S (120 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-GRPS-01, UC-GRPS-02, UC-AUTHZ-03
CAPABILITIES: CAP-AUTHZ-01, CAP-CONFIG-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-api group_ops_non_admin_denied_all_mutating_verbs
         cargo test -p but-api group_remove_member_writes_worktree_inert_until_committed
         cargo test -p but-api group_grant_administration_write_delegates_admin_inert_until_committed
         cargo test -p but-api group_create_duplicate_errs_without_overwrite
         cargo test -p but group_no_delete_surface_in_sprint_05
  check: cargo check -p but-api --all-targets && cargo check -p but --all-targets
  fmt:   cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
The `but group` task cannot pass unless all four mutating group verbs are admin-gated, `group_remove_member` actually removes a member, delegated admin grants are allowed but inert, duplicate create cannot overwrite a group, and Sprint 05 ships no delete placeholder.

--------------------------------------------------------------------------------
CRITICAL CONSTRAINTS
--------------------------------------------------------------------------------
- [MUST] MUST exercise `group_create`, `group_grant`, `group_add_member`, and `group_remove_member` in the non-admin denial matrix.
- [MUST] MUST prove `group_remove_member` changes only the working-tree config and leaves the target ref unchanged.
- [MUST] MUST allow `group_grant administration:write` as delegated admin while proving it is inert until committed.
- [MUST] MUST treat duplicate `group_create` as Err/no-overwrite.
- [NEVER] NEVER ship a Sprint 05 `but group delete` clap variant, API function, `todo!()`, `unimplemented!()`, or placeholder.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: all four mutating group verbs deny non-admin with unchanged file and remediation hints.
- [ ] AC-2: admin `group_remove_member` removes a member in the working tree only.
- [ ] AC-3: admin `group_grant administration:write` succeeds in the working tree and remains inert.
- [ ] AC-4: duplicate `group_create` returns Err without overwriting the existing group.
- [ ] AC-5: Sprint 05 exposes no `group_delete` surface or placeholder.
- [ ] All verification gates pass; only writeAllowed files modified.

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA
--------------------------------------------------------------------------------

AC-1: Non-admin denial matrix covers all mutating group verbs [PRIMARY]
  GIVEN: committed governance where `admin` has `administration:write`, `rust-reviewer` is known but lacks it, and `maintainers` exists.
  WHEN: `rust-reviewer` calls `group_create`, `group_grant`, `group_add_member`, and `group_remove_member`.
  THEN: each returns `perm.denied`, names `administration:write`, includes non-empty `remediation_hint`, and leaves the working-tree permissions file byte-for-byte unchanged.
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api group_* + real but-authz + real git
  VERIFY: cargo test -p but-api group_ops_non_admin_denied_all_mutating_verbs

AC-2: Admin remove-member mutates working tree and stays inert
  GIVEN: `maintainers` has members `maint` and `rust-reviewer` in committed config.
  WHEN: admin calls `group_remove_member(&repo, "refs/heads/main", "maintainers", "rust-reviewer")`.
  THEN: the working-tree group no longer lists `rust-reviewer`; target-ref membership is unchanged; `ref_id(refs/heads/main)` is unchanged; the caveat is returned.
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api group_remove_member + real gix
  VERIFY: cargo test -p but-api group_remove_member_writes_worktree_inert_until_committed

AC-3: Delegated admin grant is required and inert
  GIVEN: admin holds `administration:write` and `maintainers` exists without `administration:write`.
  WHEN: admin calls `group_grant(&repo, "refs/heads/main", "maintainers", ["administration:write"])`.
  THEN: the working-tree group permissions include `administration:write`, target-ref effective authorities do not include the new grant until committed, and `ref_id(refs/heads/main)` is unchanged.
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api group_grant + real but-authz + real git
  VERIFY: cargo test -p but-api group_grant_administration_write_delegates_admin_inert_until_committed

AC-4: Duplicate group create is Err/no-overwrite
  GIVEN: committed `[[group]] name="maintainers"` with permissions and members.
  WHEN: admin calls `group_create(&repo, "refs/heads/main", "maintainers", ["reviews:write"])`.
  THEN: the call returns a duplicate-group/config error, no existing values are overwritten, and the working-tree file is byte-for-byte unchanged.
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api group_create + real gix
  VERIFY: cargo test -p but-api group_create_duplicate_errs_without_overwrite

AC-5: Sprint 05 ships no delete placeholder
  GIVEN: the Sprint 05 group CLI/API surface.
  WHEN: tests inspect the group args, command, and governance boundary.
  THEN: there is no `Delete` clap variant, no Sprint 05 `group_delete` API function, and no `todo!()`/`unimplemented!()` placeholder for group deletion.
  TEST_TIER: integration   VERIFICATION_SERVICE: real but CLI command tree/source assertions
  VERIFY: cargo test -p but group_no_delete_surface_in_sprint_05

--------------------------------------------------------------------------------
TEST CRITERIA
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): all four non-admin group mutators deny with unchanged file. VERIFY: `cargo test -p but-api group_ops_non_admin_denied_all_mutating_verbs`
- TC-2 (-> AC-2): admin remove-member writes working tree only. VERIFY: `cargo test -p but-api group_remove_member_writes_worktree_inert_until_committed`
- TC-3 (-> AC-3): `administration:write` group grant succeeds and is inert. VERIFY: `cargo test -p but-api group_grant_administration_write_delegates_admin_inert_until_committed`
- TC-4 (-> AC-4): duplicate create Err/no-overwrite. VERIFY: `cargo test -p but-api group_create_duplicate_errs_without_overwrite`
- TC-5 (-> AC-5): no delete surface or placeholder exists. VERIFY: `cargo test -p but group_no_delete_surface_in_sprint_05`

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01, CAP-CONFIG-01
provides: complete `but group` contract coverage for all mutating verbs, group membership removal, delegated admin grants, duplicate safety, and no delete placeholder.
consumes: CLI-001 governance writer + Sprint 05 group API/CLI surfaces.
boundary_contracts:
  - CAP-AUTHZ-01: every group mutator composes the admin-write guard and returns structured denial on non-admin.
  - CAP-CONFIG-01: successful group writes change only the working-tree config until committed.

--------------------------------------------------------------------------------
SCOPE
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-api/src/legacy/governance.rs (MODIFY if needed)
  - crates/but-api/tests/group_governance.rs (MODIFY)
  - crates/but/tests/but/command/group.rs (MODIFY)
  - .spec/prds/governance/tasks/sprint-05-cli-perm-group/SPRINT.md (MODIFY)
writeProhibited:
  - crates/but-authz/tests/invariant_build_gates.rs — CLI-001 owns governance.rs grep coverage.
  - Any file adding a Sprint 05 group delete surface.
  - Any file not listed in writeAllowed.

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-api/src/legacy/governance.rs (full)
   Focus: existing group functions and shared TOML writer.
2. crates/but-api/tests/group_governance.rs (full)
   Focus: current group fixture and denial test shape.
3. crates/but/tests/but/command/group.rs (full)
   Focus: CLI snapbox wiring and stderr/stdout assertions.
4. .spec/prds/governance/tasks/sprint-05-cli-perm-group/CLI-002-group-cli-verbs.md
   Focus: original group task contract and red-hat gaps.
5. .spec/reviews/red-hat-20260620T051414Z.md
   Focus: blocking findings for CLI-002.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- `cargo test -p but-api group_ops_non_admin_denied_all_mutating_verbs` -> Exit 0.
- `cargo test -p but-api group_remove_member_writes_worktree_inert_until_committed` -> Exit 0.
- `cargo test -p but-api group_grant_administration_write_delegates_admin_inert_until_committed` -> Exit 0.
- `cargo test -p but-api group_create_duplicate_errs_without_overwrite` -> Exit 0.
- `cargo test -p but group_no_delete_surface_in_sprint_05` -> Exit 0.
- `cargo check -p but-api --all-targets && cargo check -p but --all-targets` -> Exit 0.
- `cargo fmt --check` -> Exit 0.

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: Extend the existing `legacy::governance` group functions and shared raw-wire TOML writer. Keep all mutating verbs behind `enforce_administration_write_gate(&repo, target_ref)` before any write.
anti_pattern: A one-verb-only denial test, a no-op `group_remove_member`, rejecting `administration:write` specially, overwriting duplicate groups, or adding a delete placeholder.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — owns Rust API/CLI tests and governance writer behavior.
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
  "task_id": "CLI-REM-001",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "group_governance_base": {
      "description": "Real-git scenario via but_testsupport::writable_scenario(\"checkout-head-info\"). Target ref refs/heads/main carries committed .gitbutler/permissions.toml with admin holding administration:write, rust-reviewer known with reviews:write only, rust-implementer role=\"write\", group maintainers permissions=[\"merge\"] members=[\"maint\",\"rust-reviewer\"], and gates.toml protecting main.",
      "seed_method": "cli",
      "records": [
        "invoke_bash: mkdir -p .gitbutler",
        "write .gitbutler/permissions.toml with [[principal]] id=\"admin\" permissions=[\"administration:write\",\"merge\"]; [[principal]] id=\"rust-reviewer\" permissions=[\"reviews:write\"]; [[principal]] id=\"rust-implementer\" role=\"write\"; [[principal]] id=\"maint\" permissions=[\"merge\"]; [[group]] name=\"maintainers\" permissions=[\"merge\"] members=[\"maint\",\"rust-reviewer\"]",
        "write .gitbutler/gates.toml with [[branch]] name=\"main\" protected=true",
        "git add .gitbutler/permissions.toml .gitbutler/gates.toml && git commit -m \"governance config\"",
        "capture committed_blob_text(repo, but_authz::permissions_path()) and ref_id(repo, \"refs/heads/main\") before each mutation"
      ]
    },
    "group_no_delete_surface": {
      "description": "Real CLI crate source tree after CLI-002 group surface exists, with only create/grant/add-member/remove-member/list in Sprint 05.",
      "seed_method": "cli",
      "records": [
        "run cargo test -p but group_no_delete_surface_in_sprint_05 against crates/but/src/args/group.rs and crates/but-api/src/legacy/governance.rs",
        "run rg -n \"group_delete|Delete|todo!|unimplemented!\" crates/but/src/args/group.rs crates/but/src/command/group.rs crates/but-api/src/legacy/governance.rs"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "Non-admin denial matrix covers group_create/group_grant/group_add_member/group_remove_member with perm.denied, administration:write message, remediation_hint, and unchanged file.",
      "verify": "cargo test -p but-api group_ops_non_admin_denied_all_mutating_verbs",
      "scenario": {
        "id": "AC-1",
        "test_tier": "integration",
        "negative_control": {
          "would_fail_if": [
            "group_create/group_grant/group_remove_member omit the admin guard",
            "a stub only denies group_add_member",
            "a denied call writes or overwrites the permissions file"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "group_governance_base",
            "action": {
              "steps": [
                "set BUT_AGENT_HANDLE=rust-reviewer",
                "call group_create(..., \"new-team\", [\"reviews:write\"])",
                "call group_grant(..., \"maintainers\", [\"comments:write\"])",
                "call group_add_member(..., \"maintainers\", \"rust-implementer\")",
                "call group_remove_member(..., \"maintainers\", \"maint\")",
                "read working-tree .gitbutler/permissions.toml after each call"
              ]
            },
            "end_state": {
              "must_observe": [
                "all `4` calls return error code `perm.denied`",
                "each message contains `administration:write`",
                "each denial has remediation_hint length `> 0`",
                "working-tree file after each call == captured committed_blob_text"
              ],
              "must_not_observe": [
                "no `new-team` group",
                "no `comments:write` grant added",
                "no member change persisted",
                "empty remediation_hint"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "Admin group_remove_member writes working tree only and remains inert until committed.",
      "verify": "cargo test -p but-api group_remove_member_writes_worktree_inert_until_committed",
      "scenario": {
        "id": "AC-2",
        "test_tier": "integration",
        "negative_control": {
          "would_fail_if": [
            "group_remove_member is a no-op stub",
            "the writer commits the removal to refs/heads/main",
            "the implementation deletes the whole group instead of one member"
          ]
        },
        "evidence": { "artifact_type": "file_artifact", "required_capture": true },
        "cases": [
          {
            "start_ref": "group_governance_base",
            "action": {
              "steps": [
                "capture ref_id(repo, \"refs/heads/main\")",
                "set BUT_AGENT_HANDLE=admin",
                "call group_remove_member(&repo, \"refs/heads/main\", \"maintainers\", \"rust-reviewer\")",
                "read working-tree TOML",
                "load_governance_config(&repo, \"refs/heads/main\")"
              ]
            },
            "end_state": {
              "must_observe": [
                "call returns `Ok`",
                "working-tree `maintainers` members exclude `rust-reviewer`",
                "result caveat contains `takes effect once committed to the target branch`",
                "ref_id after == ref_id before"
              ],
              "must_not_observe": [
                "no target-ref member removal before commit",
                "empty `maintainers` group",
                "no caveat"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "group_grant administration:write succeeds as required delegated admin and remains inert until committed.",
      "verify": "cargo test -p but-api group_grant_administration_write_delegates_admin_inert_until_committed",
      "scenario": {
        "id": "AC-3",
        "test_tier": "integration",
        "negative_control": {
          "would_fail_if": [
            "administration:write is special-cased and rejected",
            "the writer commits the delegated admin grant",
            "a stub returns Ok without writing the token"
          ]
        },
        "evidence": { "artifact_type": "file_artifact", "required_capture": true },
        "cases": [
          {
            "start_ref": "group_governance_base",
            "action": {
              "steps": [
                "capture ref_id(repo, \"refs/heads/main\")",
                "set BUT_AGENT_HANDLE=admin",
                "call group_grant(&repo, \"refs/heads/main\", \"maintainers\", [\"administration:write\"])",
                "read working-tree TOML",
                "load_governance_config(&repo, \"refs/heads/main\")"
              ]
            },
            "end_state": {
              "must_observe": [
                "call returns `Ok`",
                "working-tree `maintainers` permissions include `administration:write`",
                "ref_id after == ref_id before"
              ],
              "must_not_observe": [
                "no target-ref `administration:write` before commit",
                "empty permissions list",
                "no working-tree token"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "description": "Duplicate group_create returns Err and does not overwrite existing group.",
      "verify": "cargo test -p but-api group_create_duplicate_errs_without_overwrite",
      "scenario": {
        "id": "AC-4",
        "test_tier": "integration",
        "negative_control": {
          "would_fail_if": [
            "a stub duplicate create returns Ok without checking existing groups",
            "duplicate create overwrites maintainers",
            "duplicate create silently succeeds",
            "duplicate create appends a second conflicting group"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "group_governance_base",
            "action": {
              "steps": [
                "capture working-tree permissions text",
                "set BUT_AGENT_HANDLE=admin",
                "call group_create(&repo, \"refs/heads/main\", \"maintainers\", [\"reviews:write\"])",
                "read working-tree permissions text again"
              ]
            },
            "end_state": {
              "must_observe": [
                "call returns error variant `DuplicateGroup` or code `config.invalid`",
                "working-tree text after == text before",
                "existing `maintainers` permissions still include `merge`"
              ],
              "must_not_observe": [
                "no overwrite of `maintainers`",
                "no second `maintainers` group",
                "empty original members"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "description": "Sprint 05 has no group_delete implementation or placeholder delete surface.",
      "verify": "cargo test -p but group_no_delete_surface_in_sprint_05",
      "scenario": {
        "id": "AC-5",
        "test_tier": "integration",
        "negative_control": {
          "would_fail_if": [
            "a placeholder Delete variant is added",
            "group_delete is stubbed with todo or unimplemented",
            "help exposes a static delete surface"
          ]
        },
        "evidence": { "artifact_type": "stdout", "required_capture": true },
        "cases": [
          {
            "start_ref": "group_no_delete_surface",
            "action": {
              "steps": [
                "run cargo test -p but group_no_delete_surface_in_sprint_05",
                "run rg scan for group_delete/Delete/todo!/unimplemented! in group governance files"
              ]
            },
            "end_state": {
              "must_observe": [
                "cargo test exits `0`",
                "rg scan returns `0` matches for `group_delete` in Sprint 05 group surface"
              ],
              "must_not_observe": [
                "no `Delete` clap variant",
                "no placeholder",
                "no stub"
              ]
            }
          }
        ]
      }
    },
    { "id": "TC-1", "type": "test_criterion", "description": "all four non-admin group mutators deny with unchanged file", "verify": "cargo test -p but-api group_ops_non_admin_denied_all_mutating_verbs", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "admin remove-member writes working tree only", "verify": "cargo test -p but-api group_remove_member_writes_worktree_inert_until_committed", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "administration:write group grant succeeds and is inert", "verify": "cargo test -p but-api group_grant_administration_write_delegates_admin_inert_until_committed", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "duplicate create Err/no-overwrite", "verify": "cargo test -p but-api group_create_duplicate_errs_without_overwrite", "maps_to_ac": "AC-4" },
    { "id": "TC-5", "type": "test_criterion", "description": "no delete surface or placeholder", "verify": "cargo test -p but group_no_delete_surface_in_sprint_05", "maps_to_ac": "AC-5" }
  ]
}
-->
