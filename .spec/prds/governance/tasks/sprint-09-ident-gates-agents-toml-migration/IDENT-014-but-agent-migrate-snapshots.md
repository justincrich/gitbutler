# IDENT-014 — `crates/but/tests/but/command/agent.rs` — extend with `migrate` snapshots (initial + idempotent re-run + permissions-only warning)

**Sprint:** [Sprint 09](./SPRINT.md) · **Agent:** `rust-reviewer` · **Estimate:** 120 min · **Type:** FEATURE (TEST) · **Status:** READY · **Proposed By:** rust-planner

## Background

Sprint 08's IDENT-007 created `crates/but/tests/but/command/agent.rs` with `register`/`whoami`/`list`/`unregister` snapbox snapshots. This task EXTENDS that file with 3 new snapshots covering IDENT-011's `but agent migrate` verb + IDENT-009's permissions-only legacy deprecation warning.

**Why it matters.** CLI snapshots are the human-readable contract: they document the exact operator-facing output and catch regressions in caveat wording, exit codes, and idempotent messaging. Sprint 11's `but-migrate` skill consumes this verb — these snapshots are the regression net for skill-driven repo migrations.

**Current state.** `crates/but/tests/but/command/agent.rs` exists (Sprint 08) with `register`/`whoami`/`list`/`unregister` snapshots. IDENT-011 adds the `migrate` verb; this task captures its snapshots. The deprecation warning (IDENT-009) is observable via any loader-invoking CLI command against a legacy repo with only `permissions.toml` committed.

**Desired state.** 3 new `#[test]` fns in `agent.rs`: `agent_migrate_initial_writes_agents_toml_with_caveat`, `agent_migrate_idempotent_rerun_is_noop`, `agent_migrate_permissions_only_emits_deprecation_warning`.

## Critical Constraints

- **MUST** use `env.but(...).assert().success()/failure()` with `.stdout_eq(snapbox::str![...])` / `.stderr_eq(snapbox::str![...])` — the snapbox pattern from `crates/but/tests/but/command/merge_gate.rs:14-27`.
- **MUST** use `[..]` or `...` wildcards for volatile portions (repo paths, OIDs, timestamps) instead of weakening the snapshot.
- **MUST** seed permissions-only legacy state for AC-3 via `env.invoke_bash(...)` — never `std::process::Command::new("git")`.
- **MUST** mark each `#[test]` with `#[serial_test::serial]` (mirrors `perm.rs:6` — shared `projects_root` workspace state).
- **NEVER** rewrite or rename any existing test fn in `agent.rs` (IDENT-007 owns those bodies).
- **NEVER** use `env.but(...).output()` followed by manual stdout string assertions — keep output checks in snapbox (per `crates/but/AGENTS.md`).
- **NEVER** call `load_governance_config` or any `but-api`/`but-authz` Rust function directly from this CLI test file — this is a CLI snapshot suite; the loader is exercised through a real CLI command.
- **NEVER** introduce a new CLI command to observe the warning — reuse an existing loader-invoking command (`but perm list` / `but agent list --committed`).
- **STRICTLY** the idempotent re-run snapshot (AC-2) MUST be textually DISTINCT from the initial-run snapshot (AC-1) — the migrate verb must print a different message when `agents.toml` already exists. If IDENT-011 emits identical output for both, file that as an IDENT-011 finding, do not paper over it here.
- **STRICTLY** the permissions-only warning (AC-3) MUST be observed on a legacy repo where `.gitbutler/permissions.toml` is committed and `.gitbutler/agents.toml` is absent. When `agents.toml` is present, the loader must prefer it and ignore `permissions.toml` without this warning.

## Specification

**Objective:** Add 3 snapbox snapshot tests to `crates/but/tests/but/command/agent.rs` covering the `but agent migrate` verb: (1) initial run writes `agents.toml` + prints the ref-pin caveat; (2) a second run is a no-op with a distinct message; (3) when only legacy `permissions.toml` is committed, a loader-invoking CLI command emits exactly one deprecation warning naming `permissions.toml` + `but agent migrate`.

**Success state:** `cargo test -p but --features but-2 --test but -- agent_migrate` passes with the 3 new snapshots captured. `SNAPSHOTS=overwrite` regenerates them cleanly. No existing `agent.rs` snapshot is modified.

## Acceptance Criteria

**AC-1 (PRIMARY)** — initial migrate writes `agents.toml` with caveat: GIVEN a governed Sandbox with ONLY `.gitbutler/permissions.toml` committed (no `agents.toml` in working tree or at target ref) WHEN `but agent migrate` is invoked via `env.but("agent").arg("migrate").assert().success().stdout_eq(...)` THEN exit 0; stdout matches the snapbox snapshot which includes (a) confirmation that `.gitbutler/agents.toml` was written and (b) the ref-pin caveat ("takes effect once committed to the target branch" — same caveat string as `but perm grant` per `perm.rs:3`); the working tree now contains `.gitbutler/agents.toml`.
- **Verify:** `cargo test -p but --features but-2 --test but -- agent_migrate_initial_writes_agents_toml_with_caveat` (update with `SNAPSHOTS=overwrite`)
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** but-cli · **FLOW_REF:** UC-IDENT-04
- **Scenario:** `start_ref=permissions_only_committed`; `must_observe` = [exit 0, stdout snapshot matches (contains agents.toml-written confirmation + ref-pin caveat), `.gitbutler/agents.toml` exists in working tree]; `must_not_observe` = [non-zero exit, stdout missing ref-pin caveat, no agents.toml file written]; `negative_control.would_fail_if` = [IDENT-011's migrate verb does not print the ref-pin caveat, verb writes wrong path or does not write at all, snapshot omits caveat line and a future regression silently drops it].

**AC-2** — idempotent re-run is no-op: GIVEN a governed Sandbox where `but agent migrate` has already run (`agents.toml` exists in working tree from AC-1's state, `permissions.toml` still present) WHEN `but agent migrate` is invoked a SECOND time THEN exit 0; stdout matches a DISTINCT snapbox snapshot that communicates the no-op (e.g. "agents.toml already exists; nothing to do"); working-tree `agents.toml` content is byte-unchanged from the first run (no rewrite).
- **Verify:** `cargo test -p but --features but-2 --test but -- agent_migrate_idempotent_rerun_is_noop`
- **Scenario:** `start_ref=agents_toml_already_written`; `must_observe` = [exit 0, stdout snapshot matches AND is textually distinct from AC-1's snapshot, `agents.toml` working-tree content unchanged (sha or byte equality before/after the second run)]; `must_not_observe` = [AC-1's caveat+confirmation message repeated verbatim, `agents.toml` rewritten (content diff), non-zero exit].

**AC-3** — permissions-only legacy repo emits deprecation warning: GIVEN a governed Sandbox where ONLY `.gitbutler/permissions.toml` is committed (`.gitbutler/agents.toml` absent) WHEN a loader-invoking CLI command is run (e.g. `but perm list` or `but agent list --committed` — whichever exercises `load_governance_config`) THEN the command emits exactly one deprecation warning line naming `permissions.toml` as deprecated AND naming `but agent migrate` (or the remediation) — captured via snapbox on stderr or stdout per where IDENT-009 emits it.
- **Verify:** `cargo test -p but --features but-2 --test but -- agent_migrate_permissions_only_emits_deprecation_warning`
- **Scenario:** `start_ref=permissions_only_committed`; `must_observe` = [loader-invoking command exits 0, exactly one warning line containing "permissions.toml" AND ("deprecated" OR "but agent migrate")]; `must_not_observe` = [zero warning lines, more than one warning line, command failing (warning must not escalate during migration window), `agents.toml` present in the fixture].

## Test Criteria

| ID | Boolean | Maps to AC |
|----|---------|------------|
| TC-1 | `but agent migrate` on a permissions-only repo exits 0 and stdout matches the snapshot containing the ref-pin caveat is true | AC-1 |
| TC-2 | `.gitbutler/agents.toml` exists in the working tree after the initial migrate is true | AC-1 |
| TC-3 | A second `but agent migrate` exits 0 with a stdout snapshot textually distinct from the initial run AND leaves `agents.toml` byte-unchanged is true | AC-2 |
| TC-4 | On a permissions-only legacy repo, a loader-invoking CLI command emits exactly one warning line naming `permissions.toml` + `but agent migrate` is true | AC-3 |

## Reading List

1. `crates/but/tests/but/command/merge_gate.rs:9-30` — snapbox `.stdout_eq` / `.stderr_eq` assertion pattern (the canonical CLI snapshot shape).
2. `crates/but/tests/but/command/perm.rs:1-36` — `Sandbox` + `env.but(...).env("BUT_AGENT_HANDLE", ...)` + `REF_PIN_CAVEAT` constant (mirror this caveat assertion for AC-1).
3. `crates/but/tests/but/utils.rs:71-84` — `Sandbox::open_scenario_with_target_and_default_settings` + `env.invoke_bash`.
4. `crates/but/AGENTS.md` — CLI test rules (snapbox, `[..]` wildcards, `SNAPSHOTS=overwrite`, no `std::process::Command::new("git")`).
5. `.spec/prds/governance/12-uc-agent-identity.md:76-78` — UC-IDENT-04 acceptance criteria for the `but agent migrate` verb + CLI snapshot test requirement.

## Guardrails

**WRITE-ALLOWED:**
- `crates/but/tests/but/command/agent.rs` (EXTEND — add 3 new `#[test]` fns + any private helpers; do NOT edit IDENT-007's existing fns)
- `crates/but/tests/but/command/snapshots/agent/**` (NEW snapshot files captured via `SNAPSHOTS=overwrite`)

**WRITE-PROHIBITED:**
- `crates/but/src/command/agent.rs` (IDENT-011 owns the migrate verb implementation)
- `crates/but-authz/src/**` (IDENT-009 owns `config.rs`; IDENT-016 owns `lib.rs`/`authorize.rs`)
- `crates/but-api/**` (IDENT-010 owns the gate callsite swap; IDENT-013 owns `but-api/tests/agents_toml_migration.rs`)
- Any existing `#[test]` fn body in `agent.rs` authored by IDENT-007 (EXTEND-only contract)

## Code Pattern

**Reference:** `crates/but/tests/but/command/merge_gate.rs:14-27` (`env.but(...).assert().failure().stdout_eq(snapbox::str![[r#""#]]).stderr_eq(...)`).
**Source:** `crates/but/tests/but/command/perm.rs:3` (`REF_PIN_CAVEAT` constant — "takes effect once committed to the target branch").

**Design notes:**
- EXTEND coordination with Sprint 08's IDENT-007: additive only. If IDENT-007's helper fns (e.g. a `governed_agent_env()` builder) can be reused, reuse them; if a new permissions-only fixture helper is needed, add a new private helper fn, do not fork an existing one.
- AC-3 command choice: `but perm list` is the safest loader-invoking path (it calls `load_governance_config` to resolve the principal). If IDENT-011 also lands `but agent list --committed`, that is the more thematically appropriate choice. Pick whichever compiles against the post-IDENT-011 state.
- Snapshot stability: use `[..]` wildcards for the working-tree path prefix (the `Sandbox` `projects_root` is temp-dir-derived) and for any OIDs.

**Anti-pattern:** Do NOT use `env.but(...).output()` + `String::from_utf8_lossy` + `.contains()` (per `crates/but/AGENTS.md` — keep output checks in snapbox); do NOT capture a snapshot so loose (`[..]` everywhere) that a regression silently passes.

## Agent Instructions

TDD RED→GREEN per AC:
1. **RED AC-1:** Write `agent_migrate_initial_writes_agents_toml_with_caveat` — seed permissions-only Sandbox, run `env.but("agent").arg("migrate").assert().success().stdout_eq(snapbox::str![r#""#])` (empty snapshot placeholder), assert `agents.toml` exists. Run → snapshot mismatch (or migrate verb missing if IDENT-011 hasn't landed).
2. **GREEN:** `SNAPSHOTS=overwrite cargo test -p but --features but-2 --test but -- agent_migrate_initial_writes_agents_toml_with_caveat` — review captured snapshot, commit it.
3. Repeat for AC-2 (idempotent) and AC-3 (permissions-only warning).
4. Run `cargo fmt`, `cargo clippy -p but --all-targets -- -D warnings`, `cargo test -p but --features but-2 --test but -- agent_migrate`.
5. Commit via `but commit`.

## Orchestrator Verification Protocol

1. `cargo test -p but --features but-2 --test but -- agent_migrate` exit 0 (3 new snapshots pass; existing snapshots unchanged).
2. `cargo check -p but --all-targets` clean.
3. Snapshot files exist under `crates/but/tests/but/command/snapshots/agent/`.

## Agent Assignment

**Agent:** `rust-reviewer` — test-extension task per the stub. The work is snapshot assertion authoring against an already-built CLI verb (IDENT-011's `migrate`), not implementation. `rust-reviewer` is assigned because this is verification-surface extension layered over IDENT-011's implementation.
**Pairing:** none.

## Evidence Gates

- `cargo test -p but --features but-2 --test but -- agent_migrate` exit 0
- 3 new snapshot files captured
- No existing `agent.rs` snapshot modified (diff is purely additive)

## Review Criteria

- All 3 new tests use snapbox `.stdout_eq`/`.stderr_eq` (no manual stdout assertions).
- AC-2's no-op snapshot is textually distinct from AC-1's first-run snapshot.
- AC-3 observes the permissions-only warning via a real CLI command (not a direct `load_governance_config` call).
- `[..]` wildcards limited to volatile portions (paths, OIDs); no over-loose snapshots.
- `#[serial_test::serial]` on every new `#[test]`.

## Dependencies

- **Depends on:** IDENT-007 (Sprint 08 — created the file being extended), IDENT-009 (permissions-only warning), IDENT-011 (migrate verb).
- **Blocks:** Sprint 11 (snapshots are the regression net for `but-migrate` skill's repo migration path).

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "IDENT-014",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "permissions_only_committed": {
      "description": "governed Sandbox with ONLY .gitbutler/permissions.toml committed",
      "seed_method": "public_api",
      "records": [
        "Sandbox::open_scenario_with_target_and_default_settings + invoke_bash seeds permissions.toml + gates.toml"
      ]
    },
    "agents_toml_already_written": {
      "description": "same Sandbox after a successful migrate: working tree has both files, agents.toml is the renamed equivalent",
      "seed_method": "public_api",
      "records": [
        "permissions_only_committed then run but agent migrate once"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN permissions-only Sandbox WHEN `but agent migrate` THEN exit 0 + stdout snapshot with ref-pin caveat + agents.toml written",
      "test_tier": "integration",
      "verification_service": "but-cli",
      "verify": "cargo test -p but --features but-2 --test but -- agent_migrate_initial_writes_agents_toml_with_caveat",
      "maps_to_ac": null,
      "flow_ref": "UC-IDENT-04",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-cli",
        "start_ref": "permissions_only_committed",
        "must_observe": [
          "exit 0",
          "snapshot with caveat",
          "agents.toml present"
        ],
        "must_not_observe": [
          "non-zero exit",
          "missing caveat"
        ],
        "negative_control": {
          "would_fail_if": [
            "IDENT-011 omits caveat",
            "wrong path written",
            "snapshot drops caveat line"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "permissions_only_committed",
            "action": {
              "actor": "ci",
              "steps": [
                "but agent migrate → success → stdout_eq",
                "assert agents.toml exists"
              ]
            },
            "end_state": {
              "must_observe": [
                "exit code == 0",
                "stdout snapshot contains `ref-pin` caveat",
                "file `.gitbutler/agents.toml` exists"
              ],
              "must_not_observe": [
                "exit code != 0",
                "stdout snapshot has no `ref-pin` caveat",
                "empty `.gitbutler/agents.toml`"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "GIVEN agents.toml already exists WHEN `but agent migrate` again THEN exit 0 + distinct no-op snapshot + agents.toml byte-unchanged",
      "test_tier": "integration",
      "verification_service": "but-cli",
      "verify": "cargo test -p but --features but-2 --test but -- agent_migrate_idempotent_rerun_is_noop",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-cli",
        "start_ref": "agents_toml_already_written",
        "must_observe": [
          "exit 0",
          "distinct no-op snapshot",
          "bytes unchanged"
        ],
        "must_not_observe": [
          "first-run message repeated",
          "byte diff"
        ],
        "negative_control": {
          "would_fail_if": [
            "migrate re-run is not idempotent and rewrites file",
            "same first-run message emitted on re-run",
            "no-op branch omitted"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "agents_toml_already_written",
            "action": {
              "actor": "ci",
              "steps": [
                "capture bytes",
                "but agent migrate → success → stdout_eq(noop)",
                "assert bytes unchanged"
              ]
            },
            "end_state": {
              "must_observe": [
                "exit code == 0",
                "stdout snapshot contains `already migrated`",
                "agents.toml bytes before == after"
              ],
              "must_not_observe": [
                "first-run message repeated",
                "agents.toml bytes before != after",
                "empty `.gitbutler/agents.toml`"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "GIVEN permissions-only legacy repo WHEN loader-invoking CLI command runs THEN exactly one warning line naming permissions.toml + but agent migrate",
      "test_tier": "integration",
      "verification_service": "but-cli",
      "verify": "cargo test -p but --features but-2 --test but -- agent_migrate_permissions_only_emits_deprecation_warning",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-cli",
        "start_ref": "permissions_only_committed",
        "must_observe": [
          "exit 0",
          "one warning naming permissions.toml + but agent migrate"
        ],
        "must_not_observe": [
          "zero warnings",
          ">1 warnings",
          "non-zero exit",
          "agents.toml present"
        ],
        "negative_control": {
          "would_fail_if": [
            "IDENT-009 emits no permissions-only warning",
            "warning omits remediation",
            "wrong CLI command picked"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "permissions_only_committed",
            "action": {
              "actor": "ci",
              "steps": [
                "but perm list (or agent list --committed) → success → stderr/stdout_eq(warn)"
              ]
            },
            "end_state": {
              "must_observe": [
                "exit code == 0",
                "warning count == 1 and line contains `.gitbutler/permissions.toml` plus `but agent migrate`"
              ],
              "must_not_observe": [
                "warning count == 0",
                "warning count > 1",
                "empty warning text",
                "fixture includes `.gitbutler/agents.toml`"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "initial migrate exit 0 + caveat snapshot",
      "verify": "cargo test -p but --features but-2 --test but -- agent_migrate_initial_writes_agents_toml_with_caveat",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "agents.toml written to working tree",
      "verify": "cargo test -p but --features but-2 --test but -- agent_migrate_initial_writes_agents_toml_with_caveat",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "second migrate distinct no-op snapshot + bytes unchanged",
      "verify": "cargo test -p but --features but-2 --test but -- agent_migrate_idempotent_rerun_is_noop",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "permissions-only loader command emits one warning naming permissions.toml + but agent migrate",
      "verify": "cargo test -p but --features but-2 --test but -- agent_migrate_permissions_only_emits_deprecation_warning",
      "maps_to_ac": "AC-3"
    }
  ]
}
-->
