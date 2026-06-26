# IDENT-002 — `crates/but-authz/src/process.rs` — `current_pid()` + `process_start_time(pid)`

**Sprint:** [Sprint 08](./SPRINT.md) · **Agent:** `rust-implementer` · **Estimate:** 180 min · **Type:** FEATURE · **Status:** READY · **Proposed By:** rust-planner (`--no-specialists`)

## Background

The registry (IDENT-001) keys on `(pid, start_time)`. `pid` is cheap (`std::process::id`); `start_time` is the PID-reuse defense — without it, a recycled PID inherits the previous owner's registration. Linux exposes start time as field 22 of `/proc/[pid]/stat`; macOS exposes it via `libproc`'s `proc_pidinfo` with `PROC_PIDTBSDINFO`. This task lands the cross-platform helper so the registry and resolver never duplicate the per-OS logic.

**Why it matters.** A pid-only key is forgeable by patience: register PID 1234 → exit → unrelated process spawns as PID 1234 → it inherits `rust-implementer`. The composite `(pid, start_time)` makes that attack explicit: the new process has a new `start_time`, so `resolve` returns `None`.

**Current state.** No `process_start_time` helper exists in the workspace. `crates/but-authz` has no `libc` dep, and the workspace root currently has no `[workspace.dependencies]` `libc` pin. IDENT-008 owns adding `libc = "0.2.186"` at the workspace root if absent and `libc.workspace = true` in `crates/but-authz/Cargo.toml`.

**Desired state.** `crates/but-authz/src/process.rs` exposes `current_pid() -> u32` and `process_start_time(pid: u32) -> anyhow::Result<u64>` returning unix seconds. Linux reads `/proc/[pid]/stat` field 22 (jiffies since boot, converted to unix seconds via `sysconf(_SC_CLK_TCK)` + `boot_time` from `/proc/stat`). macOS reads `proc_pidinfo(pid, PROC_PIDTBSDINFO, …)` and converts `pbs_start` (clock ticks since epoch) to seconds.

## Critical Constraints

- **MUST** support both Linux and macOS (the two platforms GitButler runs on). Other Unixes fall through to `Err` with a clear "unsupported platform" message.
- **NEVER** shell out to `ps` or `date` — use the kernel API (`/proc` on Linux, `libproc` on macOS) directly. Shelling is slow, fragile, and locale-dependent.
- **MUST** return unix seconds (`u64`) as the canonical form, not jiffies / mach ticks. The TOML schema in IDENT-001 stores `start_time` as an integer unix second.
- **STRICTLY** document the per-OS source in module-level rustdoc (`# Source` section) — future maintainers MUST be able to verify the field offset / struct layout against the OS docs without re-deriving it.
- **MUST** return `Err` (not panic, not zero) for a nonexistent pid; the resolver (IDENT-003) treats `process_start_time` failure as "registry miss → fall through to env."

## Specification

**Objective:** Add `crates/but-authz/src/process.rs` exposing `current_pid()` + `process_start_time(pid)`.

**Success state:** `cargo test -p but-authz --test process` (IDENT-004) passes. `current_pid()` matches `std::process::id()`. `process_start_time(current_pid())` returns a unix second value `>= boot_time_of_host` and `<= now()`. Two calls in succession return the same value (monotonic non-decreasing for the test process).

## Acceptance Criteria

**AC-1** — GIVEN the test process is running WHEN `current_pid()` is called THEN it returns the same value as `std::process::id()`.

**AC-2** — GIVEN the test process is running WHEN `process_start_time(current_pid())` is called THEN it returns `Ok(t)` where `t > 1_000_000_000` (sanity: after Y2020) AND `t <= SystemTime::now().as_secs()`.

**AC-3** — GIVEN two successive calls to `process_start_time(current_pid())` within the same test WHEN their results are compared THEN they are equal (monotonic non-decreasing — start_time doesn't change during a process's life).

**AC-4** — GIVEN `pid = u32::MAX` (almost certainly not running) WHEN `process_start_time(pid)` is called THEN it returns `Err` whose `Display` mentions the pid (NOT a panic, NOT `Ok(0)`).

**AC-5 (error)** — GIVEN an unsupported platform (e.g., `target_os = "freebsd"` when added) WHEN `process_start_time(pid)` is compiled THEN it returns `Err("process_start_time: unsupported platform: <os>")` at runtime — the function compiles on all Unix targets via `cfg!`, errors only at call time on unsupported ones.

## Test Criteria

| ID | Boolean | Maps to AC |
|----|---------|------------|
| TC-1 | `current_pid() == std::process::id()` is true | AC-1 |
| TC-2 | `process_start_time(current_pid()).unwrap() > 1_000_000_000 && <= now_secs()` is true | AC-2 |
| TC-3 | Two calls to `process_start_time(current_pid())` return equal values is true | AC-3 |
| TC-4 | `process_start_time(u32::MAX).is_err()` AND the err Display mentions the pid is true | AC-4 |
| TC-5 | On an unsupported target (guarded by `#[cfg(not(any(target_os = "linux", target_os = "macos")))]`), `process_start_time(any)` returns `Err` naming the OS is true | AC-5 |

## Reading List

- `man 5 proc` (Linux) — `/proc/[pid]/stat` field 22 is `starttime` in clock ticks since boot; `sysconf(_SC_CLK_TCK)` gives ticks-per-second (typically 100).
- `man 2 sysconf` (POSIX) — `_SC_CLK_TCK` and `_SC_HOSTID`.
- `libproc.h` (macOS) — `proc_pidinfo(pid, PROC_PIDTBSDINFO, ...) -> struct proc_bsdinfo { pbs_start: u64 (clock ticks since epoch) }`. Conversion: `pbs_start / 100` (clock-tick-to-seconds; mach_absolute_time is NOT used here — `pbs_start` is already in clock-tick units).
- `crates/but-authz/src/principal.rs:11` — `PrincipalId(String)` — the registry stores `agent_id` as a `PrincipalId`; this task only produces the pid/start_time helpers, not the principal mapping.

## Guardrails

**WRITE-ALLOWED:**
- `crates/but-authz/src/process.rs` (NEW)
- `crates/but-authz/src/lib.rs` (export `current_pid`, `process_start_time`)
- `crates/but-authz/src/lib.rs` `mod process;` declaration
- `crates/but-authz/tests/process.rs` (RED test slice for IDENT-002 only; IDENT-004 owns the full suite)
- `Cargo.toml` and `crates/but-authz/Cargo.toml` only when this branch also carries IDENT-008's paired libc dependency work

**WRITE-PROHIBITED:**
- `crates/but-authz/src/registry.rs` (IDENT-001 owns this; only consumes `process_start_time` via IDENT-003)
- `crates/but-authz/src/authorize.rs` (IDENT-003)
- unrelated dependency changes outside IDENT-008's libc pin/dependency lines

## Code Pattern

**Reference:** `crates/but-cli/src/utils/metrics.rs` (existing cross-platform helpers in the workspace — mirror the `cfg!(target_os = …)` discipline).

**Source (Linux):**
```rust,no_run
fn start_time_linux(pid: u32) -> anyhow::Result<u64> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat"))
        .with_context(|| format!("read /proc/{pid}/stat"))?;
    // field 22 is starttime in clock ticks since boot.
    // Parse defensively: the comm field (2) can contain spaces + parens.
    let start = stat.rfind(')').ok_or_else(|| anyhow!("malformed /proc/stat"))?;
    let fields: Vec<&str> = stat[start+1..].split_whitespace().collect();
    // After ')', field indexes shift: field 3 (state) is now fields[0], so field 22 (starttime) is fields[19].
    let starttime_ticks: u64 = fields.get(19)
        .ok_or_else(|| anyhow!("missing field 19 (starttime)"))?
        .parse()?;
    let clk_tck = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    anyhow::ensure!(clk_tck > 0, "sysconf(_SC_CLK_TCK) returned {clk_tck}");
    let boot_secs = boot_time_secs()?;
    Ok(boot_secs + (starttime_ticks / clk_tck as u64))
}

fn boot_time_secs() -> anyhow::Result<u64> {
    let stat = std::fs::read_to_string("/proc/stat")?;
    for line in stat.lines() {
        if let Some(rest) = line.strip_prefix("btime ") {
            return Ok(rest.trim().parse()?);
        }
    }
    anyhow::bail!("no btime in /proc/stat")
}
```

**Source (macOS):**
```rust,no_run
fn start_time_macos(pid: u32) -> anyhow::Result<u64> {
    use std::mem::{size_of, MaybeUninit};
    let mut info: MaybeUninit<libc::proc_bsdinfo> = MaybeUninit::uninit();
    let rc = unsafe {
        libc::proc_pidinfo(
            pid as i32,
            libc::PROC_PIDTBSDINFO,
            0,
            info.as_mut_ptr() as *mut _,
            size_of::<libc::proc_bsdinfo>() as i32,
        )
    };
    anyhow::ensure!(rc == size_of::<libc::proc_bsdinfo>() as i32, "proc_pidinfo failed for pid {pid}");
    let info = unsafe { info.assume_init() };
    // pbs_start is in clock ticks since epoch; convert to seconds.
    Ok(info.pbs_start as u64 / 100)
}
```

**Anti-pattern:** do NOT use `SystemTime::now()` as a proxy for start_time — it's the *current* time, not the process's start. The registry must key on when the process STARTED, not when the registry lookup happened.

## Agent Instructions

TDD RED→GREEN→REFACTOR per AC:

1. **RED:** Add the `process.rs` module skeleton with `unimplemented!()` bodies. Write `tests/process.rs` (placeholder — IDENT-004 owns the full suite; this task needs to compile + red-test its own surface). Assert `current_pid() == std::process::id()`. Run `cargo test -p but-authz --test process -- IDENT_002` → must panic with `not implemented`.
2. **GREEN:** Implement `current_pid()` (`std::process::id()` — trivial). Implement `process_start_time` per the per-OS source snippets above. Pair with IDENT-008's libc dependency work in the same branch/commit if the dependency has not already landed.
3. **REFACTOR:** Pull the per-OS branches behind a private `start_time_linux` / `start_time_macos` pair; the public `process_start_time` is a thin `cfg!`-dispatch.
4. Run `cargo check -p but-authz --all-targets` then `cargo test -p but-authz --test process`. Commit via `but commit` (governed path).

## Orchestrator Verification Protocol

1. `cargo test -p but-authz --test process` exit 0.
2. `cargo check -p but-authz --all-targets` clean.
3. `crates/but-authz/src/process.rs` exists and exports `current_pid` + `process_start_time`.
4. The module-level rustdoc names the per-OS source (field 22 / `proc_bsdinfo.pbs_start`).

## Agent Assignment

**Agent:** `rust-implementer` — owns `crates/but-authz`. The cross-platform / `libc`-binding work is standard Rust FFI; no domain-specific knowledge required.

**Pairing:** coordinate with IDENT-008 (libc dep/docs). The honest final proof for IDENT-008's `cargo machete` gate requires this task's real `libc` consumer, so prefer landing together when both are still open.

## Evidence Gates

- `cargo test -p but-authz --test process` exit 0 (RED→GREEN proof)
- `crates/but-authz/src/process.rs` exists with the documented API
- Module-level rustdoc documents the per-OS source

## Review Criteria

- `process_start_time` returns unix seconds, not jiffies.
- Nonexistent pid → `Err`, not panic.
- Unsupported platform → `Err` (compile-time `cfg!`, runtime error string).
- Two successive calls return the same value (no time drift).

## Dependencies

- **depends_on:** IDENT-008 (`libc` dep) if already landed cleanly with this consumer; otherwise land together as a paired change.
- **blocks:** IDENT-003 (resolver uses `current_pid` + `process_start_time`), IDENT-004 (process tests).

## Notes

- `pbs_start` on macOS is documented as "start time, in nanoseconds since epoch" in some Apple headers and as "clock ticks" in others. Verify against the live value: a sane `pbs_start / factor` should land between `boot_time` and `now`. The factor that works empirically on Darwin 20+ is `100` (clock-tick seconds). If the test fails on `>` Y2020 sanity, revisit this factor.
- `proc_pidinfo` requires `#include <libproc.h>` and linking against `libproc.dylib` (already in `libc`'s darwin deps; should work out of the box on the macOS toolchain).

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "tdd_mode": "red_first",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true,
    "tdd_mode": "red_first"
  },
  "tdd_justification": "Behavioral process identity implementation with meaningful assertions against the real current process, platform start-time lookup, stable repeated reads, and nonexistent-pid errors. Pre-dispatch RED evidence should come from the IDENT-002 slice in crates/but-authz/tests/process.rs failing before the helpers are implemented.",
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN the test process is running WHEN current_pid() is called THEN it returns std::process::id()",
      "test_tier": "unit",
      "verification_service": "but-authz",
      "scenario": {
        "tier": "visible",
        "test_tier": "unit",
        "verification_service": "but-authz",
        "unit_test_justified": "pure stdlib call — no I/O",
        "negative_control": {
          "would_fail_if": [
            "a hardcoded pid stub returning 0 would not match std::process::id()",
            "a static wrong constant pid would disconnect current_pid from the process"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "test_process",
            "action": {
              "actor": "ci",
              "steps": [
                "current_pid()",
                "std::process::id()"
              ]
            },
            "end_state": {
              "must_observe": [
                "current_pid() == std::process::id()",
                "current_pid() > 0"
              ],
              "must_not_observe": [
                "current_pid() == 0",
                "current_pid() != std::process::id()"
              ]
            }
          }
        ]
      },
      "verify": "cargo test -p but-authz --test process"
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the test process is running WHEN process_start_time(current_pid()) is called THEN Ok(t) where t > 1_000_000_000 AND t <= SystemTime::now().as_secs()",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-authz",
        "negative_control": {
          "would_fail_if": [
            "a static now() stub would drift instead of reading process start",
            "a jiffies-only stub would return a small value <= 1000000000"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "test_process",
            "action": {
              "actor": "ci",
              "steps": [
                "process_start_time(current_pid())",
                "compare to SystemTime::now()"
              ]
            },
            "end_state": {
              "must_observe": [
                "process_start_time(current_pid()) == Ok(t) where t > 1000000000",
                "t <= SystemTime::now().as_secs()"
              ],
              "must_not_observe": [
                "Err(\"read /proc\")",
                "Ok(0)",
                "Ok(t) where t > now_secs"
              ]
            }
          }
        ]
      },
      "verify": "cargo test -p but-authz --test process"
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN two successive calls to process_start_time(current_pid()) WHEN their results are compared THEN they are equal (monotonic non-decreasing)",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-authz",
        "negative_control": {
          "would_fail_if": [
            "a static current-time stub would return different seconds across calls",
            "a disconnected mock clock would not read the process start"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "test_process",
            "action": {
              "actor": "ci",
              "steps": [
                "t0 = process_start_time(current_pid())",
                "t1 = process_start_time(current_pid())"
              ]
            },
            "end_state": {
              "must_observe": [
                "t0 == t1",
                "t0 > 1000000000"
              ],
              "must_not_observe": [
                "t0 != t1",
                "t0 == 0"
              ]
            }
          }
        ]
      },
      "verify": "cargo test -p but-authz --test process"
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN pid = u32::MAX (almost certainly not running) WHEN process_start_time(pid) is called THEN Err whose Display mentions the pid",
      "test_tier": "integration",
      "verification_service": "but-authz",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "but-authz",
        "negative_control": {
          "would_fail_if": [
            "a missing-pid stub returning Ok(0) would hide absent pid 4294967295",
            "a panic stub would abort instead of returning Err"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "u32_max_pid",
            "action": {
              "actor": "ci",
              "steps": [
                "process_start_time(u32::MAX)",
                "format!(\"{}\", err)"
              ]
            },
            "end_state": {
              "must_observe": [
                "process_start_time(4294967295) == Err(...)",
                "Err display contains \"4294967295\""
              ],
              "must_not_observe": [
                "Ok(0)",
                "Ok(4294967295)",
                "panic"
              ]
            }
          }
        ]
      },
      "verify": "cargo test -p but-authz --test process"
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN an unsupported platform WHEN process_start_time(pid) is compiled and called THEN it returns Err(\"process_start_time: unsupported platform: <os>\") at runtime",
      "test_tier": "unit",
      "verification_service": "but-authz",
      "scenario": {
        "tier": "visible",
        "test_tier": "unit",
        "verification_service": "but-authz",
        "unit_test_justified": "cfg!-dispatched branch — no I/O",
        "negative_control": {
          "would_fail_if": [
            "an omitted unsupported-target branch would be absent on target_os=\"freebsd\"",
            "a panic stub would crash instead of returning the unsupported-platform Err"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "unsupported_target_stub",
            "action": {
              "actor": "ci",
              "steps": [
                "call the cfg-dispatch stub on a mock unsupported target"
              ]
            },
            "end_state": {
              "must_observe": [
                "Err display contains \"process_start_time: unsupported platform\"",
                "Err display contains target_os=\"freebsd\""
              ],
              "must_not_observe": [
                "Ok(0)",
                "Ok(t)",
                "panic"
              ]
            }
          }
        ]
      },
      "verify": "cargo test -p but-authz --test process"
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "maps_to_ac": "AC-1",
      "description": "current_pid() == std::process::id()",
      "verify": "cargo test -p but-authz --test process"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "maps_to_ac": "AC-2",
      "description": "process_start_time(current_pid()) returns Ok(t) with t in (1e9, now()]",
      "verify": "cargo test -p but-authz --test process"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "maps_to_ac": "AC-3",
      "description": "Two calls return equal values",
      "verify": "cargo test -p but-authz --test process"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "maps_to_ac": "AC-4",
      "description": "process_start_time(u32::MAX).is_err() and Display mentions pid",
      "verify": "cargo test -p but-authz --test process"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "maps_to_ac": "AC-5",
      "description": "Unsupported target → Err naming OS (no compile error)",
      "verify": "cargo test -p but-authz --test process"
    }
  ],
  "fixtures": {
    "test_process": {
      "seed_method": "public_api",
      "description": "The currently running cargo test process is the subject process and std::process::id() is available as the expected pid source.",
      "records": [
        {
          "pid_source": "std::process::id()",
          "process_state": "running",
          "expected_start_time_min": 1000000001
        }
      ]
    },
    "u32_max_pid": {
      "seed_method": "public_api",
      "description": "The test calls process_start_time with sentinel pid u32::MAX, which is not expected to exist on the host.",
      "records": [
        {
          "pid": 4294967295,
          "expected_process_state": "absent"
        }
      ]
    },
    "unsupported_target_stub": {
      "seed_method": "public_api",
      "description": "A cfg-gated unsupported-target unit test compiles the real fallback branch for target_os=\"freebsd\" style platforms.",
      "records": [
        {
          "target_os": "freebsd",
          "pid": 1234,
          "supported": false
        }
      ]
    }
  }
}
-->
