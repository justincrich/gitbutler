# IDENT-008 — Add `libc` to `crates/but-authz/Cargo.toml`; document per-OS `process_start_time` source

**Sprint:** [Sprint 08](./SPRINT.md) · **Agent:** `rust-implementer` · **Estimate:** 30 min · **Type:** FEATURE · **Status:** READY · **Proposed By:** rust-planner (`--no-specialists`)

## Background

IDENT-002's `process_start_time` needs `libc` for macOS `libproc` bindings (`proc_pidinfo`, `PROC_PIDTBSDINFO`, `proc_bsdinfo`). The current `crates/but-authz/Cargo.toml` has only `anyhow`, `gix`, `serde`, `thiserror`, `toml`. This task adds the dep + lands the module-level rustdoc that documents the per-OS source.

**Why it matters.** The `libc` dep is workspace-managed (`libc.workspace = true`), so adding it is one line — but the rustdoc is non-trivial because future maintainers MUST be able to verify the field offset (Linux `/proc/[pid]/stat` field 22) and struct layout (macOS `proc_bsdinfo.pbs_start`) against the OS docs without re-deriving it. The doc is the load-bearing artifact.

**Current state.** `crates/but-authz/Cargo.toml` has no `libc` dep. `crates/but-authz/src/process.rs` doesn't exist yet (IDENT-002). The workspace root `Cargo.toml` does not currently pin `libc` in `[workspace.dependencies]`; `crates/but-db` pins `libc = "0.2.186"` locally.

**Desired state.** `crates/but-authz/Cargo.toml` `[dependencies]` includes `libc.workspace = true`. `crates/but-authz/src/process.rs` has a module-level rustdoc with a `# Source` section citing the man pages + Apple header for the field offsets.

## Critical Constraints

- **MUST** use `libc.workspace = true` (not `libc = "0.2"`) in `crates/but-authz/Cargo.toml`. If the workspace root still lacks `libc`, add `libc = "0.2.186"` to `[workspace.dependencies]` first.
- **MUST NOT** land as a standalone dependency-only branch that claims `cargo machete -p but-authz` is clean before IDENT-002's real `process.rs` implementation uses `libc`.
- **NEVER** vendor or re-implement `libproc` bindings — `libc` already provides them on Darwin targets.
- **MUST** document the per-OS source in `crates/but-authz/src/process.rs` module rustdoc:
  - Linux: `/proc/[pid]/stat` field 22 (`starttime`, clock ticks since boot), `sysconf(_SC_CLK_TCK)`, `/proc/stat` `btime` (boot time)
  - macOS: `libproc.h` `proc_pidinfo(pid, PROC_PIDTBSDINFO, …) -> struct proc_bsdinfo { pbs_start: u64 }`
- **STRICTLY** document the unit conversion (ticks → seconds) and any platform-specific quirks (e.g., `pbs_start` factor on Darwin).

## Specification

**Objective:** Add `libc.workspace = true` to `crates/but-authz/Cargo.toml`; ensure `crates/but-authz/src/process.rs` has the documented module-level rustdoc.

**Success state:** after the paired IDENT-002 implementation lands, `cargo build -p but-authz` succeeds on both Linux and macOS, `cargo machete -p but-authz` does not flag `libc`, and `cargo doc -p but-authz --no-deps` shows the `process` module rustdoc with the per-OS source citations.

## Acceptance Criteria

**AC-1** — GIVEN the paired IDENT-002 implementation exists and `crates/but-authz/Cargo.toml` is edited to add `libc.workspace = true` WHEN `cargo build -p but-authz && cargo machete -p but-authz` is run THEN build succeeds AND machete does NOT flag `libc` as unused because the real `process.rs` implementation uses `libc` symbols.

**AC-2** — GIVEN the workspace root `Cargo.toml` has `libc = "0.2.186"` in `[workspace.dependencies]` WHEN `cargo metadata` is inspected THEN `but-authz`'s resolved `libc` dependency comes from the workspace pin.

**AC-3** — GIVEN `crates/but-authz/src/process.rs` exists (from IDENT-002) WHEN `cargo doc -p but-authz --no-deps` is run THEN the generated docs include a module-level rustdoc on `process` with a `# Source` section citing both the Linux `/proc/[pid]/stat` field-22 reference AND the macOS `proc_bsdinfo.pbs_start` reference.

**AC-4** — GIVEN `libc` is added and IDENT-002's cfg-dispatched process implementation exists WHEN `cargo build -p but-authz --target x86_64-unknown-linux-gnu` AND `cargo build -p but-authz --target aarch64-apple-darwin` are run (or their native equivalents) THEN both succeed (the dep is cross-platform-safe).

## Test Criteria

| ID | Boolean | Maps to AC |
|----|---------|------------|
| TC-1 | After IDENT-002 lands, `cargo build -p but-authz` exits 0 is true | AC-1 |
| TC-2 | After IDENT-002 lands, `cargo machete -p but-authz` does NOT flag `libc` is true | AC-1 |
| TC-3 | `cargo metadata` shows but-authz's `libc` matches workspace pin is true | AC-2 |
| TC-4 | `cargo doc -p but-authz --no-deps` generates the `process` module rustdoc with Source section is true | AC-3 |
| TC-5 | The dep compiles on Linux and macOS targets is true | AC-4 |

## Reading List

- `crates/but-authz/Cargo.toml` — current `[dependencies]` table
- `Cargo.toml` (workspace root) — `[workspace.dependencies]` (add `libc = "0.2.186"` if absent)
- `crates/but-authz/src/process.rs` (IDENT-002) — the module whose rustdoc this task adds to / verifies
- `man 5 proc` (Linux) — the `/proc/[pid]/stat` field layout (field 22 = `starttime`)
- `<libproc.h>` (macOS) — `proc_pidinfo` + `proc_bsdinfo` struct

## Guardrails

**WRITE-ALLOWED:**
- `crates/but-authz/Cargo.toml` (one line in `[dependencies]`)
- `crates/but-authz/src/process.rs` (rustdoc only — production code is IDENT-002's responsibility; coordinate to land together)
- `Cargo.toml` (workspace root, only to add `libc = "0.2.186"` under `[workspace.dependencies]` if still absent)

**WRITE-PROHIBITED:**
- Any other crate's `Cargo.toml`

## Code Pattern

**Reference:** `crates/but-cli/Cargo.toml` (or any crate that uses `libc.workspace = true`) — copy the line exactly.

**Source (Cargo.toml):**
```toml
[dependencies]
anyhow.workspace = true
gix.workspace = true
libc.workspace = true       # NEW — for process_start_time macOS libproc bindings (IDENT-002)
serde.workspace = true
thiserror.workspace = true
toml.workspace = true
```

**Source (rustdoc on `process.rs`):**
```rust
//! Per-process identity helpers for the agent registry.
//!
//! # Source
//!
//! ## Linux
//!
//! `process_start_time(pid)` reads `/proc/[pid]/stat` field 22 (`starttime`,
//! clock ticks since system boot). The boot time is read from `/proc/stat`'s
//! `btime ` line. Seconds-since-epoch = `btime + (starttime / sysconf(_SC_CLK_TCK))`.
//! See `man 5 proc` (Fields /proc/[pid]/stat → field 22).
//!
//! ## macOS
//!
//! `process_start_time(pid)` calls `proc_pidinfo(pid, PROC_PIDTBSDINFO, …)`
//! which populates a `struct proc_bsdinfo` whose `pbs_start` field holds the
//! process start time in clock ticks since epoch. Conversion to seconds divides
//! by 100 (the Darwin clock-tick factor; empirically stable on Darwin 20+).
//! See `<libproc.h>` and `<sys/sysctl.h>` (PROC_PIDTBSDINFO constant).
//!
//! ## Unsupported platforms
//!
//! Returns `Err("process_start_time: unsupported platform: <os>")` on non-Linux
//! non-macOS Unix targets. Windows is not supported (GitButler does not run on
//! Windows as of v1.4.0).
```

**Anti-pattern:** do NOT add a `lazy_static` or `once_cell` dep to cache the boot time — read `/proc/stat` `btime` each call (cheap; avoids stale cache across NTP adjustments).

## Agent Instructions

1. Verify `libc` is in `[workspace.dependencies]` at repo root. If missing, add `libc = "0.2.186"` to match the existing `crates/but-db` local pin.
2. Add `libc.workspace = true` to `crates/but-authz/Cargo.toml` `[dependencies]`.
3. Verify IDENT-002 has landed with real `libc::sysconf` / `libc::proc_pidinfo` usage; if not, land this task in the same branch/commit as IDENT-002.
4. `cargo build -p but-authz` → must succeed after IDENT-002 is present.
5. `cargo machete -p but-authz` → must NOT flag `libc` after IDENT-002 is present.
6. `cargo doc -p but-authz --no-deps` → verify the `process` module rustdoc renders.
7. Commit via `but commit`.

## Orchestrator Verification Protocol

1. Assert `crates/but-authz/src/process.rs` exists and contains real `libc::sysconf` or `libc::proc_pidinfo` usage from IDENT-002; do not accept fake/dummy usage.
2. `cargo build -p but-authz` exit 0.
3. `cargo machete -p but-authz` clean for `libc`.
4. `cargo doc -p but-authz --no-deps` succeeds; rustdoc present.

## Agent Assignment

**Agent:** `rust-implementer` — owns `crates/but-authz/Cargo.toml`. Trivial change with non-trivial documentation.

**Pairing:** shared with IDENT-002 (the consumer). Its clean machete/doc proof is only complete after IDENT-002's real `process.rs` implementation lands.

## Evidence Gates

- IDENT-002 process implementation exists and uses `libc` for real OS process-start lookups
- `cargo build -p but-authz` exit 0
- `cargo machete -p but-authz` does NOT flag `libc`
- Rustdoc renders with the `# Source` section

## Review Criteria

- `libc.workspace = true` (not pinned version).
- Rustdoc on `process.rs` cites the man page (Linux) and Apple header (macOS).
- Cross-platform compile (Linux + macOS targets).
- No caching of boot time (read fresh each call).

## Dependencies

- **depends_on:** IDENT-002 for clean `cargo machete` / rustdoc proof, OR land in the same branch/commit as IDENT-002.
- **blocks:** IDENT-003 and IDENT-004 only through the paired IDENT-002 process module.

## Notes

- This task must not claim standalone completion with only dependency/doc edits. If dispatched separately before IDENT-002, it may add the workspace pin and dependency line, but the orchestrator must defer the clean machete/doc evidence gate until the paired IDENT-002 branch is present.

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "tdd_mode": "shared",
  "shared_test_ref": "IDENT-002; crates/but-authz/src/process.rs; crates/but-authz/tests/process.rs",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": false,
    "requires_seeded_evidence": true,
    "tdd_mode": "shared"
  },
  "tdd_justification": "Non-behavioral dependency and rustdoc wiring task paired with IDENT-002's behavioral process tests. Requiring standalone RED evidence for IDENT-008 would be fakeable because the clean machete proof only exists after IDENT-002's real libc-using process implementation lands.",
  "non_behavioral_justification": "Adds the workspace libc pin, the but-authz workspace dependency line, and documentation/source-citation requirements for process_start_time. Standalone dependency-only completion is not accepted; final evidence is command-verifiable only against the paired IDENT-002 implementation so workers cannot fake libc usage.",
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN IDENT-002's real process.rs implementation has landed and crates/but-authz/Cargo.toml adds libc.workspace = true WHEN cargo build -p but-authz and cargo machete -p but-authz run THEN build succeeds AND machete does NOT flag libc as unused",
      "test_tier": "integration",
      "verification_service": "cargo",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "cargo",
        "negative_control": {
          "would_fail_if": [
            "an unused-dependency stub would add libc but never reference libc::sysconf or libc::proc_pidinfo in real process_start_time code",
            "a dependency-only branch would run cargo machete before IDENT-002's process.rs consumer exists",
            "a missing workspace pin would omit libc.workspace = true"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "paired_process_impl_landed",
            "action": {
              "actor": "ci",
              "steps": [
                "cargo test -p but-authz --test process",
                "cargo build -p but-authz",
                "cargo machete -p but-authz"
              ]
            },
            "end_state": {
              "must_observe": [
                "cargo test -p but-authz --test process exit code == 0",
                "cargo build -p but-authz exit code == 0",
                "cargo machete -p but-authz output does not contain \"libc\""
              ],
              "must_not_observe": [
                "compile error count > 0",
                "machete output contains \"libc\"",
                "process.rs has no libc::sysconf or libc::proc_pidinfo",
                "empty dependency line"
              ]
            }
          }
        ]
      },
      "verify": "cargo test -p but-authz --test process && cargo build -p but-authz && cargo machete -p but-authz"
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN workspace root Cargo.toml has libc = \"0.2.186\" in [workspace.dependencies] and but-authz uses libc.workspace = true WHEN cargo metadata is inspected THEN but-authz's libc dependency resolves through the workspace pin",
      "test_tier": "integration",
      "verification_service": "cargo",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "cargo",
        "negative_control": {
          "would_fail_if": [
            "a local-version stub would pin libc = \"0.2\" instead of workspace dependency",
            "a missing-root-pin stub would add libc.workspace = true without adding workspace libc",
            "a disconnected Cargo.toml edit would omit but-authz libc dependency"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "cargo_toml_edited",
            "action": {
              "actor": "ci",
              "steps": [
                "cargo metadata --format-version 1 | jq '.packages[] | select(.name==\"but-authz\").dependencies[] | select(.name==\"libc\")'"
              ]
            },
            "end_state": {
              "must_observe": [
                "root Cargo.toml contains \"libc = \\\"0.2.186\\\"\" in [workspace.dependencies]",
                "cargo metadata shows but-authz dependency \"libc\"",
                "but-authz libc source == \"workspace\""
              ],
              "must_not_observe": [
                "local dependency line contains \"libc = \\\"0.2\\\"\"",
                "metadata has no \"libc\" dependency",
                "empty dependency list"
              ]
            }
          }
        ]
      },
      "verify": "rg -n 'libc = \"0.2.186\"' Cargo.toml && rg -n 'libc\\.workspace = true' crates/but-authz/Cargo.toml && cargo metadata --format-version 1 --no-deps | jq -e '.packages[] | select(.name==\"but-authz\").dependencies[] | select(.name==\"libc\")'"
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN process.rs exists from IDENT-002 WHEN cargo doc -p but-authz --no-deps runs THEN the process module rustdoc has a # Source section citing Linux man 5 proc field 22 AND macOS proc_bsdinfo.pbs_start",
      "test_tier": "integration",
      "verification_service": "cargo-doc",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "cargo-doc",
        "negative_control": {
          "would_fail_if": [
            "a vague rustdoc stub would omit \"man 5 proc\"",
            "a macOS-doc stub would omit \"proc_bsdinfo.pbs_start\""
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "process_rs_landed",
            "action": {
              "actor": "ci",
              "steps": [
                "cargo doc -p but-authz --no-deps",
                "grep the rendered HTML for 'man 5 proc' and 'proc_bsdinfo'"
              ]
            },
            "end_state": {
              "must_observe": [
                "rustdoc HTML contains \"man 5 proc\"",
                "rustdoc HTML contains \"field 22\"",
                "rustdoc HTML contains \"proc_bsdinfo.pbs_start\""
              ],
              "must_not_observe": [
                "missing \"man 5 proc\"",
                "missing \"proc_bsdinfo.pbs_start\"",
                "empty rustdoc Source section"
              ]
            }
          }
        ]
      },
      "verify": "cargo doc -p but-authz --no-deps"
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN libc is added and IDENT-002's cfg-dispatched process implementation exists WHEN cargo build -p but-authz runs on Linux AND macOS targets THEN both succeed (cross-platform-safe)",
      "test_tier": "integration",
      "verification_service": "cargo",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "cargo",
        "negative_control": {
          "would_fail_if": [
            "an omitted cfg branch would make target_os=\"macos\" absent",
            "an omitted Linux branch would make target_os=\"linux\" absent"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "paired_process_impl_landed",
            "action": {
              "actor": "ci",
              "steps": [
                "cargo build -p but-authz (host target = linux or macos)",
                "verify the cfg!-dispatch covers both target_os branches"
              ]
            },
            "end_state": {
              "must_observe": [
                "host cargo build -p but-authz exit code == 0",
                "cfg branch contains \"target_os = linux\"",
                "cfg branch contains \"target_os = macos\""
              ],
              "must_not_observe": [
                "compile error count > 0",
                "missing \"target_os = linux\"",
                "missing \"target_os = macos\""
              ]
            }
          }
        ]
      },
      "verify": "cargo build -p but-authz"
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "maps_to_ac": "AC-1",
      "description": "After IDENT-002 lands, cargo build -p but-authz exits 0",
      "verify": "cargo test -p but-authz --test process && cargo build -p but-authz"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "maps_to_ac": "AC-1",
      "description": "After IDENT-002 lands, cargo machete -p but-authz does NOT flag libc",
      "verify": "cargo test -p but-authz --test process && cargo machete -p but-authz"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "maps_to_ac": "AC-2",
      "description": "cargo metadata shows but-authz libc matches the workspace pin",
      "verify": "rg -n 'libc = \"0.2.186\"' Cargo.toml && rg -n 'libc\\.workspace = true' crates/but-authz/Cargo.toml && cargo metadata --format-version 1 --no-deps | jq -e '.packages[] | select(.name==\"but-authz\").dependencies[] | select(.name==\"libc\")'"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "maps_to_ac": "AC-3",
      "description": "cargo doc renders the # Source rustdoc",
      "verify": "cargo doc -p but-authz --no-deps"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "maps_to_ac": "AC-4",
      "description": "Cross-platform compile (Linux + macOS)",
      "verify": "cargo build -p but-authz"
    }
  ],
  "fixtures": {
    "paired_process_impl_landed": {
      "seed_method": "migration_fixture",
      "description": "IDENT-002 has landed real process_start_time code; workspace Cargo.toml pins libc, but-authz uses libc.workspace = true, and process.rs references libc symbols for real OS lookups.",
      "records": [
        {
          "file": "Cargo.toml",
          "workspace_dependency": "libc = \"0.2.186\""
        },
        {
          "file": "crates/but-authz/Cargo.toml",
          "dependency_line": "libc.workspace = true"
        },
        {
          "file": "crates/but-authz/src/process.rs",
          "uses": [
            "libc::sysconf",
            "libc::proc_pidinfo"
          ]
        }
      ]
    },
    "cargo_toml_edited": {
      "seed_method": "migration_fixture",
      "description": "Workspace Cargo.toml pins libc to 0.2.186 and but-authz depends on that workspace dependency rather than a local version.",
      "records": [
        {
          "workspace_dependency": "libc = \"0.2.186\"",
          "local_dependency_line": "libc.workspace = true",
          "forbidden_local_version": "libc = \"0.2\""
        }
      ]
    },
    "process_rs_landed": {
      "seed_method": "migration_fixture",
      "description": "crates/but-authz/src/process.rs exists with module rustdoc containing a # Source section for Linux and macOS.",
      "records": [
        {
          "file": "crates/but-authz/src/process.rs",
          "rustdoc_anchors": [
            "# Source",
            "man 5 proc",
            "field 22",
            "proc_bsdinfo.pbs_start"
          ]
        }
      ]
    },
    "libc_added": {
      "seed_method": "migration_fixture",
      "description": "libc dependency and process cfg branches are present before host/cross-target build verification.",
      "records": [
        {
          "dependency_line": "libc.workspace = true"
        },
        {
          "cfg_branches": [
            "target_os = \"linux\"",
            "target_os = \"macos\""
          ]
        }
      ]
    }
  }
}
-->
