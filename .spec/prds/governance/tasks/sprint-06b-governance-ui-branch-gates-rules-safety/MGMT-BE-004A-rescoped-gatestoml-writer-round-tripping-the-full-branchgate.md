# MGMT-BE-004A: Rescoped gates.toml writer round-tripping the full branch+gate schema (successor to MGMT-BE-004)

**Type:** FEATURE | **Status:** Done - superseded by MGMT-BE-004 at HEAD b3449afbb2 (per REMEDIATE-06B-C triage) | **Priority:** P0 | **Effort:** L (240 min)
**Agent:** rust-implementer | **Reviewer:** rust-reviewer | **Proposed by:** rust-planner
**Closes red-hat findings:** H1, M4, M5

**Supersedes:** MGMT-BE-004 (per `red-hat-20260622T145305Z.md` — original scope was unimplementable / false premise)
**Superseded by:** MGMT-BE-004 (lossless full-schema round-trip is already satisfied at HEAD b3449afbb2)
**Depends on:** REMEDIATE-RUST-1 | **Blocks:** MGMT-UI-009
**PRD refs:** UC-MGMT-04, UC-MGMT-06 | **Capabilities:** CAP-AUTHZ-01, CAP-CONFIG-01

## What this does

Replace MGMT-BE-004 with a writer that round-trips the full [[branch]]+[[gate]] schema now accepted by but-authz, enforces admin ordering structurally via a build-time grep gate, and keeps writes working-tree-only.

## Why

Supersedes MGMT-BE-004 per red-hat-20260622T145305Z.md (H1, M4, M5). Admin branch_gates_update edits any gate field losslessly; non-admin attempts are denied before any write; branch_gates_read reports committed state plus a pending diff; SDK/Tauri surfaces the new commands.

## Scope

- crates/but-api/src/legacy/governance.rs
- crates/but-api/src/legacy/mod.rs
- crates/gitbutler-tauri/src/.../governance.rs (register branch_gates_read/branch_gates_update commands)
- src-tauri/capabilities/ (allow entries mirroring perm*\*/group*\*)
- packages/but-sdk/src/generated/\*\* (REGENERATE ONLY via pnpm build:sdk)
- crates/but-api/tests/branch_gates.rs (NEW)
- crates/but-api/tests/ordering_grep.rs (NEW — AC-9 source-structural gate)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: MGMT-BE-004A — Rescoped gates.toml writer round-tripping the full branch+gate schema (successor to MGMT-BE-004)
================================================================================

TASK_TYPE:   FEATURE
STATUS:      Backlog
PRIORITY:    P0
EFFORT:      L  (240 min)
AGENT:       implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-MGMT-04, UC-MGMT-06
CAPABILITIES:CAP-AUTHZ-01,CAP-CONFIG-01
SUPERSEDES:  MGMT-BE-004
CLOSES:      H1, M4, M5

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Replace MGMT-BE-004 with a writer that round-trips the full [[branch]]+[[gate]] schema now accepted by but-authz, enforces admin ordering structurally via a build-time grep gate, and keeps writes working-tree-only.

Success state: Admin branch_gates_update edits any gate field losslessly; non-admin attempts are denied before any write; branch_gates_read reports committed state plus a pending diff; SDK/Tauri surfaces the new commands.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST supersede MGMT-BE-004 — the original contract's [[gate]] round-trip ACs were unimplementable against the but-authz deny_unknown_fields loader (red-hat-20260622T145305Z.md H1); this contract depends on REMEDIATE-RUST-1 landing first and re-grounds every fixture against the widened schema
- [MUST] MUST compose enforce_administration_write_gate (crates/but-api/src/legacy/config_mutate.rs:18) BEFORE any std::fs::write in branch_gates_update's body — the new AC-9 source-structural grep gate enforces this ordering at build time
- [MUST] MUST resolve the working-tree gates.toml path exclusively through but_authz::gates_path() (the accessor added by REMEDIATE-RUST-1); the literal '.gitbutler/gates.toml' MUST NOT appear in governance.rs production code
- [MUST] MUST round-trip the FULL [[branch]] + [[gate]] schema (GatesWire/BranchWire/GateWire matching merge_gate.rs:439-467) losslessly — the writer owns its own raw serde structs with #[derive(Serialize, Deserialize)] and #[serde(deny_unknown_fields)] preserved
- [MUST] MUST write the working tree only; NEVER git add/stage/commit/move a ref from the production writer
- [NEVER] NEVER drop, normalize, or smooth the [[gate]] review-requirement array on any edit (lossy round-trip = silent governance weakening — CRITICAL)
- [NEVER] NEVER re-implement admin gating — compose enforce_administration_write_gate; do not fork a second authorize(AdministrationWrite)
- [NEVER] NEVER hand-edit packages/but-sdk/src/generated — regenerate via pnpm build:sdk only
- [STRICTLY] STRICTLY site branch_gates_read/branch_gates_update at the but-api boundary (crates/but-api/src/legacy/governance.rs, beside CLI-001's perm_*/config_mutate.rs)
- [STRICTLY] STRICTLY keep signatures (&gix::Repository, target_ref: &str, ...) so Tauri commands pass the same target ref the CLI resolves
- [STRICTLY] STRICTLY treat branch_gates_read as administration:read-gated and branch_gates_update as administration:write-gated at the but-api boundary

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-?:
- [ ] AC-?:
- [ ] AC-?:
- [ ] AC-?:
- [ ] AC-?:
- [ ] AC-?:
- [ ] AC-?:
- [ ] AC-?:
- [ ] AC-?:

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
? :
  SCENARIO:
    tier: integration   test_tier: integration
    verification_service: cargo test -p but-api branch_gates_update_writes_worktree_inert_until_committed
    negative_control.would_fail_if:
      - writer committed the edit (ref changes)
      - writer dropped the [[gate]] array (re-parse gate.len()==0)
      - branch_gates_read read working tree as committed (would show min_approvals=3 with no pending)
      - no-op stub returned Ok without writing
    case[0] start_ref=committed gates.toml with [[branch]] main protected=true AND full [[gate]] main {min_approvals=2, require_distinct_from_author=true, require_approval_from_group=['code-reviewers','maintainers']}; clean working tree; BUT_AGENT_HANDLE=admin
      end_state.must_observe:
        - working-tree [[gate]] main min_approvals==3
        - branch_gates_read committed main min_approvals==2
        - pending==true for main
        - ref_id after==before
        - caveat contains "takes effect once committed to the target branch"
      end_state.must_not_observe:
        - committed main min_approvals==3
        - ref_id changed
        - [[gate]] array dropped

? :
  SCENARIO:
    tier: integration   test_tier: integration
    verification_service: cargo test -p but-api branch_gates_update_unprotect_preserves_gate_requirement
    negative_control.would_fail_if:
      - writer drops the full gate block when protected is toggled off
      - writer writes protected=false but also clears require_approval_from_group
      - stub returns Ok without modifying the file
    case[0] start_ref=main protected=true with full [[gate]]
      end_state.must_observe:
        - working-tree [[branch]] main protected==false
        - [[gate]] main {min_approvals==2, distinct==true, groups==['code-reviewers','maintainers']} survives
      end_state.must_not_observe:
        - [[gate]] main dropped or normalized
        - committed main protected changed before commit

? :
  SCENARIO:
    tier: integration   test_tier: integration
    verification_service: cargo test -p but-api branch_gates_update_round_trips_full_gate_schema_lossless
    negative_control.would_fail_if:
      - GovConfig round-trip drops [[gate]] array
      - writer only modeled [[branch]] and dropped gates
      - release's entries dropped
    case[0] start_ref=main protected=true with full gate; release protected=true with [[gate]] release {min_approvals==1, groups==['maintainers']}
      end_state.must_observe:
        - rewritten working-tree gates.toml STILL carries main's full [[gate]]
        - release's unrelated [[branch]]/[[gate]] entries survive
        - gate.len()==2, branch.len()==2, all field values identical
      end_state.must_not_observe:
        - any gate field altered
        - any [[branch]] or [[gate]] entry removed

? :
  SCENARIO:
    tier: integration   test_tier: integration
    verification_service: cargo test -p but-api branch_gates_update_denies_non_admin_unchanged
    negative_control.would_fail_if:
      - writer wrote then rolled back (byte-for-byte passes but ordering violated — AC-9 catches the source pattern)
      - admin gate missing entirely
      - writer checks only contents:write
    case[0] start_ref=caller rust-implementer holds ["contents:write"] only; working-tree gates.toml captured before
      end_state.must_observe:
        - returns Err
        - classify_error(&err) → Some(AdminWriteGateError) whose .code == "perm.denied"
        - message contains "administration:write"
        - working-tree gates.toml byte-for-byte UNCHANGED
      end_state.must_not_observe:
        - any mutation to gates.toml
        - error code != perm.denied

? :
  SCENARIO:
    tier: integration   test_tier: integration
    verification_service: cargo test -p but-api branch_gates_read_returns_pending_signal
    negative_control.would_fail_if:
      - reader returns working-tree value without committed comparison
      - reader returns no pending field
      - reader always returns pending==false
    case[0] start_ref=main committed protected=true min_approvals==2; admin has edited working-tree min_approvals==3 (uncommitted)
      end_state.must_observe:
        - returns BranchGatesOutcome with main gate {committed min_approvals==2, pending==true}
      end_state.must_not_observe:
        - only working-tree min_approvals without committed baseline
        - pending==false when working tree differs

? :
  SCENARIO:
    tier: integration   test_tier: integration
    verification_service: cargo test -p but-api branch_gates_update_sets_full_field_set
    negative_control.would_fail_if:
      - writer ignores array-of-strings fields
      - writer normalizes group ordering
      - writer loses min_approvals/protected values
    case[0] start_ref=main with full gate
      end_state.must_observe:
        - working-tree [[gate]] main require_distinct_from_author==false
        - working-tree [[gate]] main require_approval_from_group==['eng','security']
        - min_approvals/protected unchanged
      end_state.must_not_observe:
        - group array dropped or reordered
        - other gate fields mutated

? :
  SCENARIO:
    tier: integration   test_tier: integration
    verification_service: cargo test -p but-api branch_gates_update_appends_new_branch
    negative_control.would_fail_if:
      - writer overwrites main when appending staging
      - writer fails on absent gates.toml instead of creating
      - writer appends invalid TOML (missing [[gate]] block)
    case[0] start_ref=gates.toml has main only
      end_state.must_observe:
        - working-tree gates.toml has both main AND staging [[branch]]/[[gate]] entries
        - main entries unchanged
      end_state.must_not_observe:
        - main dropped
        - missing [[gate]] staging block
    case[1] start_ref=absent gates.toml
      end_state.must_observe:
        - file created with staging entry only
      end_state.must_not_observe:
        - panic or error on missing file
        - spurious extra entries

? :
  SCENARIO:
    tier: integration   test_tier: integration
    verification_service: cargo test -p but-api branch_gates_read_denies_without_admin_read
    negative_control.would_fail_if:
      - reader is public/unauthorized
      - reader only checks contents:read
      - reader leaks committed gate data before gating
    case[0] start_ref=caller lacking administration:read
      end_state.must_observe:
        - returns Err perm.denied naming administration:read
      end_state.must_not_observe:
        - gate data returned without authorization

? :
  SCENARIO:
    tier: integration   test_tier: integration
    verification_service: cargo test -p but-api branch_gates_update_ordering_grep
    negative_control.would_fail_if:
      - writer has no enforce call before write
      - writer calls write_authorized before the gate
      - writer writes to a temp file before the gate then rolls back
    case[0] start_ref=production governance.rs
      end_state.must_observe:
        - exit 0 on production code
        - exit non-zero on a planted re-ordered copy
        - message names the offending ordering
      end_state.must_not_observe:
        - test passing on a write-before-gate function body

--------------------------------------------------------------------------------
TEST CRITERIA
--------------------------------------------------------------------------------
- ?: Admin branch_gates_update writes the gate edit into the working-tree gates.toml while leaving the committed target-ref gate unchanged and reporting a pending diff.
- ?: Toggling protected=false in working tree preserves the branch's full [[gate]] requirement.
- ?: A protection-only edit round-trips the full [[gate]] set and every unrelated [[branch]]/[[gate]] entry losslessly.
- ?: Non-admin branch_gates_update returns perm.denied citing administration:write and leaves gates.toml byte-for-byte unchanged.
- ?: branch_gates_read returns the committed gate set and a pending=true signal when the working tree differs.
- ?: Admin branch_gates_update sets require_distinct_from_author and require_approval_from_group without mutating other gate fields.
- ?: branch_gates_update appends a new [[branch]]/[[gate]] pair without mutating existing entries and creates gates.toml when absent.
- ?: branch_gates_read denies callers lacking administration:read with perm.denied.
- ?: The source-structural ordering_grep test enforces enforce_administration_write_gate before any write in branch_gates_update.

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
- crates/but-authz/src/config.rs:531-543  (lines ?) — GatesWire widened by REMEDIATE-RUST-1 — this task CONSUMES it
- crates/but-api/src/legacy/merge_gate.rs:439-467  (lines ?) — Full schema to mirror in the writer's own GatesWire/BranchWire/GateWire
- crates/but-api/src/legacy/config_mutate.rs:18  (lines ?) — enforce_administration_write_gate — the composed admin guard
- crates/but-api/src/legacy/governance.rs:556-622  (lines ?) — Existing branch_gates_*_with_repo — expand these; do NOT add parallel functions
- crates/but-api/tests/admin_write_guard.rs:158-164  (lines ?) — write_worktree_permissions pattern — mirror for gates
- .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-BE-004-branch-gates-config-writer.md:1-861  (lines ?) — The superseded contract — every original AC's INTENT is preserved; only the schema grounding changes

--------------------------------------------------------------------------------
GUARDRAILS
--------------------------------------------------------------------------------
WRITE_ALLOWED:
  - crates/but-api/src/legacy/governance.rs
  - crates/but-api/src/legacy/mod.rs
  - crates/gitbutler-tauri/src/.../governance.rs (register branch_gates_read/branch_gates_update commands)
  - src-tauri/capabilities/ (allow entries mirroring perm_*/group_*)
  - packages/but-sdk/src/generated/** (REGENERATE ONLY via pnpm build:sdk)
  - crates/but-api/tests/branch_gates.rs (NEW)
  - crates/but-api/tests/ordering_grep.rs (NEW — AC-9 source-structural gate)
WRITE_PROHIBITED:
  - crates/but-authz/** (loader widened by REMEDIATE-RUST-1, NOT this task)
  - crates/but-server/**
  - apps/desktop/**
  - apps/web/**
  - packages/ui/**

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
depends_on: REMEDIATE-RUST-1
blocks:     MGMT-UI-009

--------------------------------------------------------------------------------
NOTES
--------------------------------------------------------------------------------
Successor to MGMT-BE-004 per red-hat-20260622T145305Z.md H1. Closes M4 (source-structural ordering gate via AC-9) and M5 (gates_path() via REMEDIATE-RUST-1 dependency). The original MGMT-BE-004 file is preserved as historical record — mark it 'Superseded by MGMT-BE-004A' in its header.

```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "MGMT-BE-004A",
  "proposed_by": "rust-planner",
  "supersedes": [
    "MGMT-BE-004"
  ],
  "closes_redhat_findings": [
    "H1",
    "M4",
    "M5"
  ],
  "fixtures": {},
  "requirements": [
    {
      "id": null,
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN  WHEN  THEN ",
      "verify": "",
      "scenario": {
        "tier": "integration",
        "test_tier": "integration",
        "verification_service": "cargo test -p but-api branch_gates_update_writes_worktree_inert_until_committed",
        "negative_control": {
          "would_fail_if": [
            "writer committed the edit (ref changes)",
            "writer dropped the [[gate]] array (re-parse gate.len()==0)",
            "branch_gates_read read working tree as committed (would show min_approvals=3 with no pending)",
            "no-op stub returned Ok without writing"
          ]
        },
        "evidence": "test output + insta snapshot of working-tree gates.toml + ref_id assertion",
        "cases": [
          {
            "start_ref": "committed gates.toml with [[branch]] main protected=true AND full [[gate]] main {min_approvals=2, require_distinct_from_author=true, require_approval_from_group=['code-reviewers','maintainers']}; clean working tree; BUT_AGENT_HANDLE=admin",
            "action": "branch_gates_update(&repo, \"refs/heads/main\", edit{branch=\"main\", min_approvals=3})",
            "end_state": {
              "must_observe": [
                "working-tree [[gate]] main min_approvals==3",
                "branch_gates_read committed main min_approvals==2",
                "pending==true for main",
                "ref_id after==before",
                "caveat contains \"takes effect once committed to the target branch\""
              ],
              "must_not_observe": [
                "committed main min_approvals==3",
                "ref_id changed",
                "[[gate]] array dropped"
              ]
            }
          }
        ]
      }
    },
    {
      "id": null,
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN  WHEN  THEN ",
      "verify": "",
      "scenario": {
        "tier": "integration",
        "test_tier": "integration",
        "verification_service": "cargo test -p but-api branch_gates_update_unprotect_preserves_gate_requirement",
        "negative_control": {
          "would_fail_if": [
            "writer drops the full gate block when protected is toggled off",
            "writer writes protected=false but also clears require_approval_from_group",
            "stub returns Ok without modifying the file"
          ]
        },
        "evidence": "re-parsed working-tree gates.toml assertions",
        "cases": [
          {
            "start_ref": "main protected=true with full [[gate]]",
            "action": "admin branch_gates_update edit{branch=\"main\", protected=false}",
            "end_state": {
              "must_observe": [
                "working-tree [[branch]] main protected==false",
                "[[gate]] main {min_approvals==2, distinct==true, groups==['code-reviewers','maintainers']} survives"
              ],
              "must_not_observe": [
                "[[gate]] main dropped or normalized",
                "committed main protected changed before commit"
              ]
            }
          }
        ]
      }
    },
    {
      "id": null,
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN  WHEN  THEN ",
      "verify": "",
      "scenario": {
        "tier": "integration",
        "test_tier": "integration",
        "verification_service": "cargo test -p but-api branch_gates_update_round_trips_full_gate_schema_lossless",
        "negative_control": {
          "would_fail_if": [
            "GovConfig round-trip drops [[gate]] array",
            "writer only modeled [[branch]] and dropped gates",
            "release's entries dropped"
          ]
        },
        "evidence": "snapshot comparison of pre/post working-tree gates.toml",
        "cases": [
          {
            "start_ref": "main protected=true with full gate; release protected=true with [[gate]] release {min_approvals==1, groups==['maintainers']}",
            "action": "admin branch_gates_update edit{branch=\"main\", protected=true} (no-op value)",
            "end_state": {
              "must_observe": [
                "rewritten working-tree gates.toml STILL carries main's full [[gate]]",
                "release's unrelated [[branch]]/[[gate]] entries survive",
                "gate.len()==2, branch.len()==2, all field values identical"
              ],
              "must_not_observe": [
                "any gate field altered",
                "any [[branch]] or [[gate]] entry removed"
              ]
            }
          }
        ]
      }
    },
    {
      "id": null,
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN  WHEN  THEN ",
      "verify": "",
      "scenario": {
        "tier": "integration",
        "test_tier": "integration",
        "verification_service": "cargo test -p but-api branch_gates_update_denies_non_admin_unchanged",
        "negative_control": {
          "would_fail_if": [
            "writer wrote then rolled back (byte-for-byte passes but ordering violated \u2014 AC-9 catches the source pattern)",
            "admin gate missing entirely",
            "writer checks only contents:write"
          ]
        },
        "evidence": "pre/post file SHA256 + classify_error assertion",
        "cases": [
          {
            "start_ref": "caller rust-implementer holds [\"contents:write\"] only; working-tree gates.toml captured before",
            "action": "branch_gates_update edit{branch=\"main\", protected=false} under BUT_AGENT_HANDLE=rust-implementer",
            "end_state": {
              "must_observe": [
                "returns Err",
                "classify_error(&err) \u2192 Some(AdminWriteGateError) whose .code == \"perm.denied\"",
                "message contains \"administration:write\"",
                "working-tree gates.toml byte-for-byte UNCHANGED"
              ],
              "must_not_observe": [
                "any mutation to gates.toml",
                "error code != perm.denied"
              ]
            }
          }
        ]
      }
    },
    {
      "id": null,
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN  WHEN  THEN ",
      "verify": "",
      "scenario": {
        "tier": "integration",
        "test_tier": "integration",
        "verification_service": "cargo test -p but-api branch_gates_read_returns_pending_signal",
        "negative_control": {
          "would_fail_if": [
            "reader returns working-tree value without committed comparison",
            "reader returns no pending field",
            "reader always returns pending==false"
          ]
        },
        "evidence": "BranchGatesOutcome debug snapshot",
        "cases": [
          {
            "start_ref": "main committed protected=true min_approvals==2; admin has edited working-tree min_approvals==3 (uncommitted)",
            "action": "branch_gates_read(&repo, \"refs/heads/main\")",
            "end_state": {
              "must_observe": [
                "returns BranchGatesOutcome with main gate {committed min_approvals==2, pending==true}"
              ],
              "must_not_observe": [
                "only working-tree min_approvals without committed baseline",
                "pending==false when working tree differs"
              ]
            }
          }
        ]
      }
    },
    {
      "id": null,
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN  WHEN  THEN ",
      "verify": "",
      "scenario": {
        "tier": "integration",
        "test_tier": "integration",
        "verification_service": "cargo test -p but-api branch_gates_update_sets_full_field_set",
        "negative_control": {
          "would_fail_if": [
            "writer ignores array-of-strings fields",
            "writer normalizes group ordering",
            "writer loses min_approvals/protected values"
          ]
        },
        "evidence": "re-parsed working-tree gate assertions",
        "cases": [
          {
            "start_ref": "main with full gate",
            "action": "admin branch_gates_update edit{branch=\"main\", require_distinct_from_author=false, require_approval_from_group=['eng','security']}",
            "end_state": {
              "must_observe": [
                "working-tree [[gate]] main require_distinct_from_author==false",
                "working-tree [[gate]] main require_approval_from_group==['eng','security']",
                "min_approvals/protected unchanged"
              ],
              "must_not_observe": [
                "group array dropped or reordered",
                "other gate fields mutated"
              ]
            }
          }
        ]
      }
    },
    {
      "id": null,
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN  WHEN  THEN ",
      "verify": "",
      "scenario": {
        "tier": "integration",
        "test_tier": "integration",
        "verification_service": "cargo test -p but-api branch_gates_update_appends_new_branch",
        "negative_control": {
          "would_fail_if": [
            "writer overwrites main when appending staging",
            "writer fails on absent gates.toml instead of creating",
            "writer appends invalid TOML (missing [[gate]] block)"
          ]
        },
        "evidence": "file existence check + re-parse assertions",
        "cases": [
          {
            "start_ref": "gates.toml has main only",
            "action": "admin branch_gates_update edit{branch=\"staging\", protected=true, min_approvals==1}",
            "end_state": {
              "must_observe": [
                "working-tree gates.toml has both main AND staging [[branch]]/[[gate]] entries",
                "main entries unchanged"
              ],
              "must_not_observe": [
                "main dropped",
                "missing [[gate]] staging block"
              ]
            }
          },
          {
            "start_ref": "absent gates.toml",
            "action": "same call",
            "end_state": {
              "must_observe": [
                "file created with staging entry only"
              ],
              "must_not_observe": [
                "panic or error on missing file",
                "spurious extra entries"
              ]
            }
          }
        ]
      }
    },
    {
      "id": null,
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN  WHEN  THEN ",
      "verify": "",
      "scenario": {
        "tier": "integration",
        "test_tier": "integration",
        "verification_service": "cargo test -p but-api branch_gates_read_denies_without_admin_read",
        "negative_control": {
          "would_fail_if": [
            "reader is public/unauthorized",
            "reader only checks contents:read",
            "reader leaks committed gate data before gating"
          ]
        },
        "evidence": "error classification assertion",
        "cases": [
          {
            "start_ref": "caller lacking administration:read",
            "action": "branch_gates_read(&repo, \"refs/heads/main\")",
            "end_state": {
              "must_observe": [
                "returns Err perm.denied naming administration:read"
              ],
              "must_not_observe": [
                "gate data returned without authorization"
              ]
            }
          }
        ]
      }
    },
    {
      "id": null,
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN  WHEN  THEN ",
      "verify": "",
      "scenario": {
        "tier": "integration",
        "test_tier": "integration",
        "verification_service": "cargo test -p but-api branch_gates_update_ordering_grep",
        "negative_control": {
          "would_fail_if": [
            "writer has no enforce call before write",
            "writer calls write_authorized before the gate",
            "writer writes to a temp file before the gate then rolls back"
          ]
        },
        "evidence": "AC-9 test logs: exit 0 on production code; exit non-zero on planted reordered copy",
        "cases": [
          {
            "start_ref": "production governance.rs",
            "action": "ordering_grep test parses governance.rs, locates branch_gates_update fn body, finds byte offset of FIRST enforce_administration_write_gate and FIRST std::fs::write (or write_worktree_gates), asserts gate_offset < write_offset",
            "end_state": {
              "must_observe": [
                "exit 0 on production code",
                "exit non-zero on a planted re-ordered copy",
                "message names the offending ordering"
              ],
              "must_not_observe": [
                "test passing on a write-before-gate function body"
              ]
            }
          }
        ]
      }
    },
    {
      "id": null,
      "type": "test_criterion",
      "description": "Admin branch_gates_update writes the gate edit into the working-tree gates.toml while leaving the committed target-ref gate unchanged and reporting a pending diff.",
      "verify": "",
      "maps_to_ac": null
    },
    {
      "id": null,
      "type": "test_criterion",
      "description": "Toggling protected=false in working tree preserves the branch's full [[gate]] requirement.",
      "verify": "",
      "maps_to_ac": null
    },
    {
      "id": null,
      "type": "test_criterion",
      "description": "A protection-only edit round-trips the full [[gate]] set and every unrelated [[branch]]/[[gate]] entry losslessly.",
      "verify": "",
      "maps_to_ac": null
    },
    {
      "id": null,
      "type": "test_criterion",
      "description": "Non-admin branch_gates_update returns perm.denied citing administration:write and leaves gates.toml byte-for-byte unchanged.",
      "verify": "",
      "maps_to_ac": null
    },
    {
      "id": null,
      "type": "test_criterion",
      "description": "branch_gates_read returns the committed gate set and a pending=true signal when the working tree differs.",
      "verify": "",
      "maps_to_ac": null
    },
    {
      "id": null,
      "type": "test_criterion",
      "description": "Admin branch_gates_update sets require_distinct_from_author and require_approval_from_group without mutating other gate fields.",
      "verify": "",
      "maps_to_ac": null
    },
    {
      "id": null,
      "type": "test_criterion",
      "description": "branch_gates_update appends a new [[branch]]/[[gate]] pair without mutating existing entries and creates gates.toml when absent.",
      "verify": "",
      "maps_to_ac": null
    },
    {
      "id": null,
      "type": "test_criterion",
      "description": "branch_gates_read denies callers lacking administration:read with perm.denied.",
      "verify": "",
      "maps_to_ac": null
    },
    {
      "id": null,
      "type": "test_criterion",
      "description": "The source-structural ordering_grep test enforces enforce_administration_write_gate before any write in branch_gates_update.",
      "verify": "",
      "maps_to_ac": null
    }
  ]
}
-->
