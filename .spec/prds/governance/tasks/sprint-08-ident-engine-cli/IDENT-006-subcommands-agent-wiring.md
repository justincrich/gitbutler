# IDENT-006 — Wire `Subcommands::Agent(args::agent::Platform { cmd })` into `crates/but/src/lib.rs`

**Sprint:** [Sprint 08](./SPRINT.md) · **Agent:** `rust-implementer` · **Estimate:** 60 min · **Type:** FEATURE · **Status:** READY · **Proposed By:** rust-planner (`--no-specialists`)

## Background

IDENT-005 creates the `agent` args + handler modules. This task plugs them into the dispatch table in `crates/but/src/lib.rs` so `but agent <verb>` actually routes to the handler. It's a small but load-bearing task — without it, the verb doesn't exist at the CLI.

**Why it matters.** The `but agent` noun needs to round-trip end-to-end (clap parse → dispatch arm → handler → output) for the gate (T-IDENT-030) and the human test (the sprint's gate sentence).

**Current state.** `crates/but/src/lib.rs:125` has `Subcommands::Perm(args::perm::Platform { cmd })`, `Subcommands::Group(args::group::Platform { cmd })`, etc. `crates/but/src/lib.rs:380` (the dispatch match) has corresponding arms calling `command::perm::exec(...)` etc.

**Desired state.** `Subcommands::Agent(args::agent::Platform { cmd })` exists in the enum; the dispatch arm at line ~380 calls `command::agent::exec(ctx, out, cmd).await` mirroring the `Perm` arm.

## Critical Constraints

- **MUST** mirror the exact `Perm` / `Group` shape — both the enum variant AND the dispatch arm.
- **NEVER** add the variant without the dispatch arm (or vice versa) — `cargo build` will fail on the missing match arm.
- **MUST** place the dispatch arm in dependency-graph order if applicable (the existing arms are roughly alphabetical but with grouping — follow whatever's already there).
- **STRICTLY** preserve the existing match arms — this is an additive change only.

## Specification

**Objective:** Wire the `Agent` variant into the `Subcommands` enum + the dispatch match.

**Success state:** `cargo build -p but` succeeds. `but agent --help` works (lists subcommands from IDENT-005). `but agent list` doesn't panic on the missing arm.

## Acceptance Criteria

**AC-1** — GIVEN IDENT-005 has landed WHEN `Subcommands::Agent(args::agent::Platform { cmd })` is added to the enum AND the dispatch arm `Subcommands::Agent(args::agent::Platform { cmd }) => { ... command::agent::exec(ctx, out, cmd).await }` is added THEN `cargo build -p but` succeeds.

**AC-2** — GIVEN the wiring is complete WHEN `but agent --help` is run THEN exit 0 AND the help text lists `register`, `unregister`, `list`, `whoami`.

**AC-3** — GIVEN the wiring is complete WHEN `but agent list` is run against a fixture repo with an empty registry THEN exit 0 AND stdout is empty (or contains only a "no registrations" message — match `perm list` empty-state behavior).

**AC-4** — GIVEN the existing `Perm` and `Group` arms WHEN `cargo build -p but` is run THEN no existing arm breaks (the new arm is purely additive).

## Test Criteria

| ID | Boolean | Maps to AC |
|----|---------|------------|
| TC-1 | `cargo build -p but` exits 0 after wiring is true | AC-1 |
| TC-2 | `but agent --help` lists all 4 subcommands is true | AC-2 |
| TC-3 | `but agent list` exits 0 on an empty registry is true | AC-3 |
| TC-4 | Existing `perm`/`group` arms unchanged is true | AC-4 |

## Reading List

- `crates/but/src/lib.rs:118-130` — the `Subcommands` enum (clap derive) with existing variants
- `crates/but/src/lib.rs:380-420` — the dispatch match (`match args.cmd { … }`) with existing arms
- `crates/but/src/lib.rs:489-515` — `Perm(args::perm::Platform { cmd })` arm specifically (the template)
- `crates/but/src/args/perm.rs` — `Platform` wrapper shape (IDENT-005 mirrors in `args/agent.rs`)
- `crates/but/src/command/perm.rs:9-15` — `exec` signature (IDENT-005 mirrors)

## Guardrails

**WRITE-ALLOWED:**
- `crates/but/src/lib.rs` (add the enum variant + dispatch arm — typically ~6 lines)

**WRITE-PROHIBITED:**
- `crates/but/src/args/**` (IDENT-005)
- `crates/but/src/command/**` (IDENT-005)
- Any other file

## Code Pattern

**Reference:** `crates/but/src/lib.rs:489-503` — the `Perm` arm:
```rust
Subcommands::Perm(args::perm::Platform { cmd }) => {
    command::perm::exec(ctx, out, cmd).await
}
```

**Source:** add at the appropriate location (mirror `Perm` / `Group` ordering):
```rust
// in the Subcommands enum:
Agent(args::agent::Platform { cmd }),

// in the dispatch match:
Subcommands::Agent(args::agent::Platform { cmd }) => {
    command::agent::exec(ctx, out, cmd).await
}
```

**Anti-pattern:** do NOT inline the handler logic into the dispatch arm. Always delegate to `command::agent::exec`. The arm is a routing shim.

## Agent Instructions

1. Confirm IDENT-005 has landed (`ls crates/but/src/args/agent.rs crates/but/src/command/agent.rs`).
2. Add the `Agent(args::agent::Platform { cmd })` variant to `Subcommands`.
3. Add the dispatch arm mirroring `Perm`.
4. `cargo build -p but` → must succeed.
5. `cargo run -p but -- agent --help` → must list the 4 verbs.
6. `cargo run -p but -- agent list` against a scratch fixture → must exit 0.
7. Commit via `but commit`.

## Orchestrator Verification Protocol

1. `cargo build -p but` exit 0.
2. `but agent --help` lists the 4 verbs.
3. `but agent list` against an empty fixture exits 0.
4. The diff to `crates/but/src/lib.rs` is ≤10 lines (additive only).

## Agent Assignment

**Agent:** `rust-implementer` — owns `crates/but/src/lib.rs`. Trivial mechanical change.

**Pairing:** depends entirely on IDENT-005 (no point wiring non-existent modules).

## Evidence Gates

- `cargo build -p but` exit 0
- `but agent --help` snapshot exists (IDENT-007 captures)

## Review Criteria

- Only `crates/but/src/lib.rs` is touched.
- The change is purely additive.
- The arm matches the `Perm`/`Group` shape exactly.

## Dependencies

- **depends_on:** IDENT-005 (modules must exist).
- **blocks:** IDENT-007 (CLI tests need the wiring).

## Notes

- This is the smallest task in the sprint by line count but it's on the critical path — IDENT-007 can't snapshot-test the verbs until the wiring lands.

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "tdd_mode": "shared",
  "shared_test_ref": "crates/but/tests/but/command/agent.rs",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": false,
    "requires_seeded_evidence": true,
    "tdd_mode": "shared"
  },
  "tdd_justification": "Small CLI wiring task whose observable behavior is intentionally verified by the shared IDENT-007 snapbox suite for help output and agent list dispatch. Requiring RED evidence directly on the routing shim would not add useful product signal.",
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN IDENT-005 landed WHEN Subcommands::Agent is added to the enum AND a dispatch arm is added THEN cargo build -p but succeeds",
      "test_tier": "integration",
      "verification_service": "cargo",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "cargo",
        "negative_control": {
          "would_fail_if": [
            "an omitted enum variant would leave Subcommands::Agent absent",
            "an omitted dispatch arm would leave command::agent::exec disconnected"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "ident_005_landed",
            "action": {
              "actor": "ci",
              "steps": [
                "add Agent variant to Subcommands enum",
                "add dispatch arm mirroring Perm",
                "cargo build -p but"
              ]
            },
            "end_state": {
              "must_observe": [
                "cargo build -p but exit code == 0",
                "contains \"Subcommands::Agent(args::agent::Platform { cmd })\" == true",
                "dispatch calls \"command::agent::exec(ctx, out, cmd).await\""
              ],
              "must_not_observe": [
                "compile error count > 0",
                "missing match arm",
                "empty dispatch arm"
              ]
            }
          }
        ]
      },
      "verify": "cargo build -p but"
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN wiring complete WHEN `but agent --help` runs THEN exit 0 AND help lists register/unregister/list/whoami",
      "test_tier": "integration",
      "verification_service": "but-cli",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-cli",
        "negative_control": {
          "would_fail_if": [
            "a clap exposure stub would omit the register verb from help",
            "a static help string would omit whoami"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "wiring_complete",
            "action": {
              "actor": "ci",
              "steps": [
                "but agent --help",
                "capture stdout + exit"
              ]
            },
            "end_state": {
              "must_observe": [
                "exit code == 0",
                "stdout contains \"register\"",
                "stdout contains \"unregister\"",
                "stdout contains \"list\"",
                "stdout contains \"whoami\""
              ],
              "must_not_observe": [
                "exit code != 0",
                "missing \"register\"",
                "empty help stdout"
              ]
            }
          }
        ]
      },
      "verify": "cargo run -p but -- agent --help"
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN wiring complete WHEN `but agent list` runs against a fixture repo with empty registry THEN exit 0 AND stdout is empty or \"no registrations\"",
      "test_tier": "integration",
      "verification_service": "but-cli",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-cli",
        "negative_control": {
          "would_fail_if": [
            "a disconnected dispatch arm would panic on empty registry",
            "a static error stub would exit nonzero for 0 registrations"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "fixture_empty_registry",
            "action": {
              "actor": "ci",
              "steps": [
                "but agent list",
                "capture stdout + exit"
              ]
            },
            "end_state": {
              "must_observe": [
                "exit code == 0",
                "stdout == \"\" or stdout contains \"no registrations\""
              ],
              "must_not_observe": [
                "exit code != 0",
                "panic",
                "stdout contains \"pid=12345\""
              ]
            }
          }
        ]
      },
      "verify": "cargo run -p but -- agent list"
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the existing Perm and Group arms WHEN cargo build -p but runs THEN no existing arm breaks (purely additive)",
      "test_tier": "integration",
      "verification_service": "cargo",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "cargo",
        "negative_control": {
          "would_fail_if": [
            "a reordered match-arm edit would delete the Perm arm",
            "a static compile-only stub would omit group test coverage"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "wiring_complete",
            "action": {
              "actor": "ci",
              "steps": [
                "cargo build -p but",
                "cargo test -p but --test perm",
                "cargo test -p but --test group"
              ]
            },
            "end_state": {
              "must_observe": [
                "cargo build -p but exit code == 0",
                "cargo test -p but --test perm exit code == 0",
                "cargo test -p but --test group exit code == 0"
              ],
              "must_not_observe": [
                "compile error count > 0",
                "perm test regression",
                "group test regression"
              ]
            }
          }
        ]
      },
      "verify": "cargo test -p but --test perm && cargo test -p but --test group"
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "maps_to_ac": "AC-1",
      "description": "cargo build -p but exits 0 after wiring",
      "verify": "cargo build -p but"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "maps_to_ac": "AC-2",
      "description": "but agent --help lists the 4 verbs",
      "verify": "cargo run -p but -- agent --help"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "maps_to_ac": "AC-3",
      "description": "but agent list exits 0 on empty registry",
      "verify": "cargo run -p but -- agent list"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "maps_to_ac": "AC-4",
      "description": "Existing perm/group arms unchanged",
      "verify": "cargo test -p but --test perm && cargo test -p but --test group"
    }
  ],
  "fixtures": {
    "ident_005_landed": {
      "seed_method": "migration_fixture",
      "description": "IDENT-005 files exist before wiring: args/agent.rs and command/agent.rs compile with Platform { cmd } shape.",
      "records": [
        {
          "file": "crates/but/src/args/agent.rs",
          "exists": true
        },
        {
          "file": "crates/but/src/command/agent.rs",
          "exists": true
        },
        {
          "platform_shape": "Platform { cmd }"
        }
      ]
    },
    "wiring_complete": {
      "seed_method": "migration_fixture",
      "description": "crates/but/src/lib.rs includes Subcommands::Agent and dispatches to command::agent::exec.",
      "records": [
        {
          "enum_variant": "Subcommands::Agent(args::agent::Platform { cmd })"
        },
        {
          "dispatch_arm": "command::agent::exec(ctx, out, cmd).await"
        },
        {
          "verbs": [
            "register",
            "unregister",
            "list",
            "whoami"
          ]
        }
      ]
    },
    "fixture_empty_registry": {
      "seed_method": "cli",
      "description": "Fixture repo has committed agents.toml but no agents-runtime.toml entries before but agent list.",
      "records": [
        {
          "file": ".gitbutler/agents.toml",
          "committed": true,
          "agents": [
            {
              "id": "rust-implementer"
            }
          ]
        },
        {
          "runtime_registry_entries": 0
        }
      ]
    }
  }
}
-->
