# IDENT-011 — `crates/but/src/command/agent.rs` — add `migrate` verb (working-tree `permissions.toml` → `agents.toml`, ref-pin caveat, idempotent)

**Sprint:** [Sprint 09](./SPRINT.md) · **Agent:** `rust-implementer` · **Estimate:** 180 min · **Type:** FEATURE · **Status:** READY · **Proposed By:** rust-planner

## Background

The IDENT file rename (`permissions.toml` → `agents.toml`) needs a CLI verb operators can run during the migration window. Sprint 08's IDENT-005/006 created `crates/but/src/command/agent.rs` + `crates/but/src/args/agent.rs` with the `register`/`unregister`/`list`/`whoami` verbs. This task EXTENDS that module with `migrate`.

**Why it matters.** Without a migration verb, operators would have to hand-edit the rename. `but agent migrate` automates the working-tree transform and prints the ref-pin caveat (the file is inert until committed — same pattern as `but perm grant`).

**Current state.** `crates/but/src/command/agent.rs` and `crates/but/src/args/agent.rs` do NOT exist in the working tree yet — they are Sprint 08 deliverables. This task extends the module Sprint 08 creates. The migration is a working-tree text/TOML rewrite (renames `[[principal]]` → `[[agent]]` and the file path); it deliberately does NOT import `AgentWire`/`AgentsWire` so byte-equivalence is a pure text transform and there is no `lib.rs` dependency on IDENT-016. The ref-pin caveat mirrors `perm.rs`'s `PermWriteOutcome.caveat` pattern.

**Desired state.** `but agent migrate` reads working-tree `permissions.toml`, writes `agents.toml` with `[[agent]]` blocks, prints the ref-pin caveat, and is idempotent.

## Critical Constraints

- **MUST** add `Migrate` to `args::agent::Subcommands` (Sprint 08's enum) with no required args.
- **MUST** dispatch `Migrate` in `command::agent::exec` alongside Sprint 08's verbs.
- **MUST** read `.gitbutler/permissions.toml` from the working tree (`repo.workdir()`), error clearly if absent.
- **MUST** write `.gitbutler/agents.toml` by renaming `[[principal]]` → `[[agent]]` at the TOML table-header level (line-oriented text transform — preserve all fields/comments/whitespace).
- **MUST** print the ref-pin caveat naming the operator step ("commit the add of agents.toml and the delete of permissions.toml together").
- **MUST** be idempotent: if `agents.toml` already exists and is non-empty, exit 0 with a distinct "already migrated; no change" message and do NOT overwrite.
- **MUST** route human/JSON output via `OutputChannel` like `perm.rs`.
- **MUST** use `anyhow::Context` on all fs ops.
- **NEVER** read or write the target ref — working tree only (the caveat is the contract).
- **NEVER** delete `permissions.toml` automatically (operator commits add+delete together — UC-IDENT-01 AC-5).
- **NEVER** use `AgentWire`/`AgentsWire` (keeps the transform pure-text; avoids a `lib.rs` dependency on IDENT-016).
- **NEVER** overwrite an existing non-empty `agents.toml` on re-run.
- **STRICTLY** preserve byte-content of every `[[principal]]` body — only the table-header token changes.
- **STRICTLY** mirror `perm.rs`'s `OutputChannel` usage (`for_human_or_shell` / `for_json`).
- **NEVER** use `std::env::temp_dir().join(format!(...))` — `but_testsupport::writable_scenario`.

## Specification

**Objective:** Add a `migrate` verb to the `but agent` CLI (created by Sprint 08) that rewrites a working-tree `permissions.toml` into `agents.toml` by renaming `[[principal]]` → `[[agent]]`, prints the ref-pin caveat, and is idempotent.

**Success state:** `cargo test -p but --features but-2 --test but -- agent_migrate` `migrate` snapshots pass (initial migrate writes `agents.toml`; re-run is a no-op; missing-source errors cleanly). The CLI exits 0 on success and nonzero on missing `permissions.toml`.

## Acceptance Criteria

**AC-1 (PRIMARY)** — GIVEN a working tree with `.gitbutler/permissions.toml` (`[[principal]]` blocks) and no `agents.toml` WHEN `but agent migrate` THEN `.gitbutler/agents.toml` is written whose content equals the `permissions.toml` with every `[[principal]]` header replaced by `[[agent]]` (fields/comments unchanged); exit 0; stdout prints the ref-pin caveat naming the add+delete commit step.
- **Verify:** `cargo test -p but --features but-2 --test but -- agent_migrate_writes_agents_toml`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-cli · **FLOW_REF:** UC-IDENT-04
- **Scenario:** `start_ref=worktree_permissions_only`; `must_observe` = [`agents.toml` exists, content == `permissions.toml` with `[[principal]]`→`[[agent]]`, stdout has ref-pin caveat, exit 0]; `must_not_observe` = [`agents.toml` absent, field names changed, `permissions.toml` deleted, target ref mutated]; `negative_control.would_fail_if` = [migrate no-ops, migrate writes target ref, `[[principal]]`→`[[agent]]` rename missing or also rewrites field names].

**AC-2** — GIVEN a working tree that already has `.gitbutler/agents.toml` (from a prior migrate) WHEN `but agent migrate` again THEN exit 0, `agents.toml` byte-content unchanged, stdout prints a distinct "already migrated; no change" message.
- **Verify:** `cargo test -p but --features but-2 --test but -- agent_migrate_idempotent`
- **Scenario:** `must_observe` = [exit 0, bytes unchanged, distinct no-op snapshot]; `must_not_observe` = [`agents.toml` overwritten, nonzero exit, first-run message repeated].

**AC-3** — GIVEN a working tree with NEITHER `permissions.toml` nor `agents.toml` WHEN `but agent migrate` THEN nonzero exit, stderr names the missing `.gitbutler/permissions.toml` path, no `agents.toml` written.
- **Verify:** `cargo test -p but --features but-2 --test but -- agent_migrate_missing_source`
- **Scenario:** `must_observe` = [nonzero exit, stderr names `permissions.toml`]; `must_not_observe` = [exit 0, `agents.toml` created].

## Test Criteria

| ID | Boolean | Maps to AC |
|----|---------|------------|
| TC-1 | `but agent migrate` writes `agents.toml` with `[[principal]]`→`[[agent]]` and prints the ref-pin caveat is true | AC-1 |
| TC-2 | A second `but agent migrate` is a no-op (exit 0, no file change) is true | AC-2 |
| TC-3 | `but agent migrate` with no `permissions.toml` exits nonzero naming the missing file is true | AC-3 |

## Reading List

1. `crates/but/src/command/perm.rs:1-134` — THE template — `exec()` shape, `OutputChannel` usage (`for_human_or_shell` / `for_json`), `PermWriteOutcome.caveat` pattern, `governance_cli_error` mapping.
2. `crates/but/src/args/perm.rs:1-50` — `Subcommands` enum shape — add `Migrate` to `args::agent::Subcommands` in the same clap style.
3. `crates/but-authz/src/config.rs:8-19` — `PERMISSIONS_PATH` / `permissions_path()` — the source path constant; `agents.toml` target path is the sibling `AGENTS_PATH` (IDENT-009).
4. `crates/but/tests/but/command/perm.rs:1-60` — CLI snapshot test shape (`env.but(...).assert().success().stdout_eq(snapbox::str![...])`) — IDENT-014 extends `agent.rs` tests in this style.

## Guardrails

**WRITE-ALLOWED:**
- `crates/but/src/command/agent.rs` (EXTEND — Sprint 08 IDENT-005 creates this file; add the `Migrate` arm to `exec` + a `migrate` helper fn)
- `crates/but/src/args/agent.rs` (EXTEND — Sprint 08 IDENT-005 creates this file; add `Migrate` to `Subcommands`)

**WRITE-PROHIBITED:**
- `crates/but/tests/but/command/agent.rs` — IDENT-014 owns the snapshot tests
- `crates/but-authz/**` — IDENT-009/016 own but-authz
- `crates/but-api/**` — IDENT-010/012/013 own but-api
- `crates/but/src/command/perm.rs` and `group.rs` — do not drive-by refactor siblings

## Code Pattern

**Reference:** `crates/but/src/command/perm.rs:28-57` (`Grant`/`Revoke` arms — the match-arm shape for `Migrate`).
**Source:** `crates/but/src/command/perm.rs:87-104` (`write_mutation` + caveat `OutputChannel` pattern).

**Design notes:**
- The `[[principal]]` → `[[agent]]` transform: read `permissions.toml` as a `String`, replace the literal table-header line `[[principal]]` with `[[agent]]` (line-oriented, preserves indentation/comments). Safer than a `toml` round-trip which could reorder keys and break byte-equivalence (AC-1 / UC-IDENT-01 AC-5).
- Idempotency check: if `.gitbutler/agents.toml` exists and is non-empty, short-circuit to the "already migrated" message without reading `permissions.toml` (so a hand-edited `agents.toml` is never clobbered).
- Caveat string: `"agents.toml written to the working tree; inert until committed. Commit the add of .gitbutler/agents.toml and the delete of .gitbutler/permissions.toml together."` — route via `OutputChannel` human + JSON.
- Resolve the repo workdir via `ctx.repo.get()?.workdir()` (`gix`), same as `perm.rs`.

**Anti-pattern:** Do NOT round-trip through `toml::Value` for the rewrite (reorders keys, breaks byte-equivalence). Do NOT delete `permissions.toml` automatically. Do NOT read/write the target ref.

## Agent Instructions

TDD RED→GREEN→REFACTOR per AC:
1. **RED AC-1:** Write `tests/but/command/agent.rs::agent_migrate_writes_agents_toml` — seed a permissions.toml-only repo, run `but agent migrate`, assert `agents.toml` exists with the renamed headers. Run → fails (`Migrate` variant doesn't exist).
2. **GREEN:** Add `Migrate` to `args::agent::Subcommands`; add the match arm + helper fn in `command::agent::exec`; do the line-oriented text transform; print the caveat via `OutputChannel`.
3. **REFACTOR:** Extract the transform into a private `fn rewrite_principals_to_agents(contents: &str) -> String` for testability.
4. Repeat for AC-2 (idempotent) and AC-3 (missing source).
5. Run `cargo fmt`, `cargo clippy -p but --all-targets -- -D warnings`, `cargo test -p but --features but-2 --test but -- agent_migrate`.
6. Commit via `but commit`.

## Orchestrator Verification Protocol

1. `cargo test -p but --features but-2 --test but -- agent_migrate` exit 0 (new `migrate` snapshots pass).
2. `cargo check -p but --all-targets` clean.
3. `crates/but/src/command/agent.rs` has a `Migrate` arm; `crates/but/src/args/agent.rs` has a `Migrate` variant.

## Agent Assignment

**Agent:** `rust-implementer` — owns the `but` CLI command layer (mirrors `perm.rs`/`group.rs`). NOTE FOR REVIEWER: `command/agent.rs` + `args/agent.rs` are Sprint 08 deliverables; this task EXTENDS them.
**Pairing:** none. IDENT-014 (snapshot tests) consumes the verb; IDENT-013 (round-trip test) consumes the underlying transform.

## Evidence Gates

- `cargo test -p but --features but-2 --test but -- agent_migrate` exit 0
- `Migrate` variant present in `args::agent::Subcommands`
- `Migrate` arm present in `command::agent::exec`

## Review Criteria

- The `[[principal]]` → `[[agent]]` transform is line-oriented text, NOT a `toml` round-trip.
- Idempotency short-circuits before reading `permissions.toml` (no clobber risk).
- Ref-pin caveat printed via `OutputChannel` (human + JSON).
- No drive-by changes to `perm.rs`/`group.rs`/sibling CLI modules.
- Missing-source path returns `Err` (via `anyhow::Context`), not a panic.

## Dependencies

- **Depends on:** IDENT-009 (consumes `AGENTS_PATH` for the target filename — though the value is a literal here, importing the constant from `but_authz::config::AGENTS_PATH` keeps it DRY).
- **Blocks:** IDENT-013 (round-trip test invokes the migration transform), IDENT-014 (snapshot tests cover the verb).

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "IDENT-011",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "worktree_permissions_only": {
      "description": "working tree has .gitbutler/permissions.toml ([[principal]] dev=contents:write, release-bot role=maintain) committed at refs/heads/main, no agents.toml",
      "seed_method": "public_api",
      "records": [
        "writable_scenario + invoke_bash commits permissions.toml + gates.toml"
      ]
    },
    "worktree_already_migrated": {
      "description": "same repo after a successful migrate: working tree has BOTH permissions.toml (unchanged) and agents.toml (the renamed equivalent)",
      "seed_method": "public_api",
      "records": [
        "worktree_permissions_only then run but agent migrate once"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN working tree with permissions.toml and no agents.toml WHEN `but agent migrate` THEN agents.toml written ([[principal]]→[[agent]], fields unchanged), exit 0, stdout prints ref-pin caveat",
      "test_tier": "integration",
      "verification_service": "but-cli",
      "verify": "cargo test -p but --features but-2 --test but -- agent_migrate_writes_agents_toml",
      "maps_to_ac": null,
      "flow_ref": "UC-IDENT-04",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-cli",
        "start_ref": "worktree_permissions_only",
        "must_observe": [
          "agents.toml exists",
          "content == permissions.toml with [[principal]]→[[agent]]",
          "stdout has ref-pin caveat",
          "exit 0"
        ],
        "must_not_observe": [
          "agents.toml absent",
          "field names changed",
          "permissions.toml deleted",
          "target ref mutated"
        ],
        "negative_control": {
          "would_fail_if": [
            "migrate no-ops",
            "migrate writes target ref",
            "rename missing or rewrites field names"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "worktree_permissions_only",
            "action": {
              "actor": "test_harness",
              "steps": [
                "run but agent migrate",
                "read agents.toml",
                "diff vs permissions.toml with header rename"
              ]
            },
            "end_state": {
              "must_observe": [
                "file `.gitbutler/agents.toml` exists after command",
                "content contains `[[agent]]` and no `[[principal]]`",
                "stdout contains `ref-pin` caveat",
                "exit code == 0"
              ],
              "must_not_observe": [
                "empty `.gitbutler/agents.toml`",
                "file `.gitbutler/agents.toml` absent",
                "field names changed beyond `[[principal]]` header",
                "`.gitbutler/permissions.toml` deleted"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "GIVEN agents.toml already present WHEN `but agent migrate` again THEN exit 0, no file change, 'already migrated' message",
      "test_tier": "integration",
      "verification_service": "but-cli",
      "verify": "cargo test -p but --features but-2 --test but -- agent_migrate_idempotent",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-cli",
        "start_ref": "worktree_already_migrated",
        "must_observe": [
          "exit 0",
          "bytes unchanged",
          "'already migrated' message"
        ],
        "must_not_observe": [
          "agents.toml overwritten",
          "nonzero exit",
          "first-run message"
        ],
        "negative_control": {
          "would_fail_if": [
            "re-run overwrites file bytes",
            "re-run errors instead of no-op",
            "idempotent branch omitted/no-op message missing"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "worktree_already_migrated",
            "action": {
              "actor": "test_harness",
              "steps": [
                "snapshot agents.toml",
                "run but agent migrate",
                "re-read agents.toml"
              ]
            },
            "end_state": {
              "must_observe": [
                "exit code == 0",
                "agents.toml bytes before == after",
                "stdout contains `already migrated`"
              ],
              "must_not_observe": [
                "agents.toml bytes before != after",
                "empty `.gitbutler/agents.toml`",
                "first-run caveat message repeated"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "GIVEN no permissions.toml and no agents.toml WHEN `but agent migrate` THEN nonzero exit, stderr names permissions.toml, no agents.toml written",
      "test_tier": "integration",
      "verification_service": "but-cli",
      "verify": "cargo test -p but --features but-2 --test but -- agent_migrate_missing_source",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-cli",
        "start_ref": "worktree_permissions_only",
        "must_observe": [
          "nonzero exit",
          "stderr names permissions.toml"
        ],
        "must_not_observe": [
          "exit 0",
          "agents.toml created"
        ],
        "negative_control": {
          "would_fail_if": [
            "migrate creates empty agents.toml",
            "migrate panics"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "worktree_permissions_only",
            "action": {
              "actor": "test_harness",
              "steps": [
                "rm permissions.toml",
                "run but agent migrate"
              ]
            },
            "end_state": {
              "must_observe": [
                "exit code != 0",
                "stderr contains `.gitbutler/permissions.toml`"
              ],
              "must_not_observe": [
                "exit code == 0",
                "file `.gitbutler/agents.toml` created",
                "empty `.gitbutler/agents.toml`"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "migrate writes agents.toml + caveat",
      "verify": "cargo test -p but --features but-2 --test but -- agent_migrate_writes_agents_toml",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "migrate idempotent",
      "verify": "cargo test -p but --features but-2 --test but -- agent_migrate_idempotent",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "migrate missing source errors",
      "verify": "cargo test -p but --features but-2 --test but -- agent_migrate_missing_source",
      "maps_to_ac": "AC-3"
    }
  ]
}
-->
