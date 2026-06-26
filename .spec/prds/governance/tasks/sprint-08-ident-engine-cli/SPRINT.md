---
sprint: 08
sequence: 10
timeline: Phase 6 — IDENT engine core + CLI (v1.4.0; appended after Sprint 07 STEER)
status: In Progress
proposed_by: rust-planner (--no-specialists mode — mechanical scope; upstream design fixed file paths + signatures + CLI shapes)
milestone: sprint-08-ident-engine-cli
prd: ../../README.md
roadmap: ../../ROADMAP.md
generated_by: but-sprint-tasks-plan
---

# Sprint 08: IDENT Engine + `but agent` CLI

**Sequence:** 10
**Timeline:** Phase 6 — IDENT engine core + CLI (v1.4.0; appended after Sprint 07 STEER)
**Status:** In Progress
**Proposed by:** rust-planner (`--no-specialists` mode — mechanical scope, upstream design fixed file paths + CLI shapes)
**Milestone:** — (`sprint-08-ident-engine-cli`)

## Overview

The **first IDENT sprint** lands the runtime identity layer that replaces the self-asserted `BUT_AGENT_HANDLE`
env var: a `Registry` mapping `(pid, start_time) → agent_id`, a per-OS `process_start_time` helper, a new
`resolve_principal_with_registry` resolver on `but-authz`, and the `but agent register / unregister / list /
whoami` CLI verbs. **No behavior change yet** — the 8 gate callsites in `but-api` keep calling
`resolve_principal_from_env` (Sprint 09 wires them to the new resolver). Sprint 08 only delivers the
machinery + the CLI to exercise it, so the registry path can be observed independently before policy flips
in Sprint 10.

The sprint depends on Sprint 07 (STEER) because the new `Denial::unregistered` / `stale_registration`
denials reuse the structured `{code, class, message, remediation_hint, ...}` carrier shape STEER added.
Sprint 09 (IDENT gates + `agents.toml` migration) is the downstream consumer.

> **`--no-specialists` provenance.** Per `but-sprint-plan`'s NEVER-TIER escape hatch for mechanical/pure-infra
> plans: the upstream design plan (`12-uc-agent-identity.md` + ROADMAP §Sprint-08) already fixed every file
> path, signature, and CLI verb shape. Task expansion here is deterministic translation of that design into
> TASK-TEMPLATE v5.1 files. Re-run `/but-sprint-tasks-plan --skip-review` is default; a fresh red-hat pass
> can be invoked with `/but-sprint-tasks-plan --only IDENT-XXX` after any code drift.

## Human Testing Gate

**Gate:** Registering a live PID via `but agent register --as <existing-agent>` succeeds and prints the resolved `(pid, start_time, agent_id, expires_at)` tuple, while `but agent register --as <unknown>` exits 1 naming the missing id, and `but agent whoami` from the registered shell returns the agent_id.

## Test Deliverable

1. Seed `.gitbutler/agents.toml` with a `rust-implementer` agent + commit it
2. Run `but agent register --pid $$ --as rust-implementer` → exit 0, observe the resolved tuple printed
3. Run `but agent whoami` → observe `rust-implementer` echoed
4. Run `but agent list` → observe the registered PID row
5. Run `but agent register --as ghost` → exit 1, observe the missing-id message
6. Run `but agent unregister --pid $$` → exit 0; subsequent `but agent list` shows the PID absent

## Tasks

| ID | Title | Agent | Estimate |
|----|-------|-------|----------|
| IDENT-001 | `crates/but-authz/src/registry.rs` — `Registry` struct + `load`/`write`/`register`/`unregister`/`resolve`/`gc` (atomic write, TTL, PID-reuse defense) | rust-implementer | 240 min |
| IDENT-002 | `crates/but-authz/src/process.rs` — `current_pid()` + `process_start_time(pid)` (Linux procfs field 22; macOS `libproc` `proc_pidinfo`) | rust-implementer | 180 min |
| IDENT-003 | `crates/but-authz/src/authorize.rs` — `resolve_principal_with_registry(reg, cfg)` + `Denial::unregistered`/`stale_registration` (registry → env fallback policy) | rust-implementer | 180 min |
| IDENT-004 | `crates/but-authz/tests/registry.rs` + `tests/process.rs` — register/unregister/TTL/PID-reuse/concurrent-writes + monotonic start_time | rust-implementer | 180 min |
| IDENT-005 | `crates/but/src/args/agent.rs` + `crates/but/src/command/agent.rs` — clap subcommands + handler mirroring `perm.rs` shape (register/unregister/list/whoami; migrate stubbed to Sprint 09) | rust-implementer | 240 min |
| IDENT-006 | Wire `Subcommands::Agent(args::agent::Platform { cmd })` into `crates/but/src/lib.rs` (variant + dispatch arm, mirroring `perm`/`group`) | rust-implementer | 60 min |
| IDENT-007 | `crates/but/tests/but/command/agent.rs` — snapbox snapshots for register/whoami/list/unregister/unknown-id (happy path + fail-fast) | rust-reviewer | 180 min |
| IDENT-008 | Add `libc` to `crates/but-authz/Cargo.toml` `[dependencies]`; document the per-OS `process_start_time` source choice in module docs | rust-implementer | 30 min |

## Source Coverage

- **UC-IDENT-02** (Runtime PID registry) — IDENT-001, IDENT-002, IDENT-004
- **UC-IDENT-03** (Enforced resolution — *resolver only*; callsite wiring in Sprint 09) — IDENT-003
- **UC-IDENT-04** (`but agent` CLI — `register`/`unregister`/`list`/`whoami`; `migrate` deferred to Sprint 09) — IDENT-005, IDENT-006, IDENT-007
- Test criteria covered: T-IDENT-009..015 (registry + process unit tests), T-IDENT-023..028 (CLI verbs), T-IDENT-030 (subcommand wiring), T-IDENT-031 (CLI snapshot suite)

## Blocks

- **Sprint 09** (IDENT Gates + `agents.toml` Migration) — the 8 gate callsites and `but agent migrate` consume this sprint's `Registry`, `resolve_principal_with_registry`, and `Subcommands::Agent` wiring.

## Task Detail Files

Generated by /but-sprint-tasks-plan on 2026-06-24 (`--no-specialists` mode, `--skip-review` first pass):

- `IDENT-001-registry-module.md`
- `IDENT-002-process-module.md`
- `IDENT-003-resolve-principal-with-registry.md`
- `IDENT-004-registry-process-unit-tests.md`
- `IDENT-005-agent-cli-args-and-handler.md`
- `IDENT-006-subcommands-agent-wiring.md`
- `IDENT-007-cli-snapshot-tests.md`
- `IDENT-008-libc-dep-and-module-docs.md`
