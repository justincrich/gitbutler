---
sprint: 09
sequence: 11
timeline: Phase 6 — IDENT gates + migration (v1.4.0)
status: Done
proposed_by: rust-planner (upstream ROADMAP `--no-specialists` declaration; task expansion dispatches rust-planner per RULES.md specialist table)
milestone: sprint-09-ident-gates-migration
prd: ../../README.md
roadmap: ../../ROADMAP.md
generated_by: but-sprint-tasks-plan
---

# Sprint 09: IDENT Gates + `agents.toml` Migration

**Sequence:** 11
**Timeline:** Phase 6 — IDENT gates + migration (v1.4.0)
**Status:** Done — closed out 2026-06-26 (all 8 tasks IDENT-009–016 merged + verified; the registry-first gate swap had silently broken the legacy governance gate suites — remediated by migrating them to the spec's `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` env-fallback contract; full governance gate test suite green across `but-api` / `but` / `gitbutler-tauri`)
**Proposed by:** rust-planner (upstream ROADMAP declared `--no-specialists` for the sprint skeleton; task expansion here dispatches `rust-planner` per the RULES.md Specialist Agents table — `--no-specialists` is "never the default" per the skill NEVER-TIER, and the project resolves `rust-planner` for the `crates/` Rust backend surface)
**Milestone:** — (`sprint-09-ident-gates-migration`)

## Overview

The **second IDENT sprint** flips the engine from `resolve_principal_from_env` (the self-asserted
`BUT_AGENT_HANDLE` string that Sprint 08 left as the only identity source) to
`resolve_principal_with_registry` (the runtime PID registry Sprint 08 shipped). It also lands the
file-format migration: `permissions.toml` → `agents.toml` (`[[principal]]` → `[[agent]]`), with
`load_governance_config` reading **both** files during a one-release migration window
(preferring `agents.toml`, emitting a deprecation warning when only `permissions.toml` is present)
and a `but agent migrate` verb that produces a byte-equivalent `agents.toml` from a working-tree
`permissions.toml`.

Sprint 09 is the **first sprint where the registry actually governs a commit/merge/admin/forge
action**. The 8 gate callsites in `but-api` (`commit/gate.rs`, `legacy/merge_gate.rs`, four sites
in `legacy/governance.rs`, `legacy/forge.rs`, `legacy/config_mutate.rs`) all swap to the new
resolver. Policy in Sprint 09 is **migration-friendly**: registry hit → principal; registry miss +
`BUT_AUTHZ_ALLOW_ENV_HANDLE=1` → env fallback (test/CI escape hatch); else →
`Denial::unregistered(pid)`. Sprint 10 flips the default to deny env-only on governed repos.

> **Why this gate ordering.** Sprint 08 shipped the registry, resolver, and `but agent` CLI but
> deliberately **did not** wire the 8 gate callsites — so Sprint 09 lands the callsite swap as one
> observable behavior change with the migration verb beside it. Sprint 10 hardens the env-var path
> out. Reversing 09/10 would ship a deny-default the migration verb cannot satisfy on legacy repos.

The sprint depends on Sprint 08 (the registry, `resolve_principal_with_registry`, `Denial::unregistered`
/`stale_registration`, and `Subcommands::Agent` are all Sprint 08 deliverables). Sprint 10
(deprecation hardening) is the downstream consumer — it needs both file formats and the migration
verb landed before it can flip the default.

## Human Testing Gate

**Gate:** A process registered via `but agent register --as <contents-write-agent>` commits successfully through the governed commit gate, while unregistering it makes the next commit fail with `perm.denied`, and `but agent migrate` against a `permissions.toml`-only repo produces a byte-equivalent `agents.toml` that loads to the same `GovConfig`.

## Test Deliverable

1. Seed `permissions.toml` (legacy) + `gates.toml`, commit them at the target ref
2. Run `but agent migrate` → observe `agents.toml` written; load both files and observe equal `GovConfig`s
3. Run `but agent migrate` again → exit 0, no file change (idempotent)
4. Register a `dev` agent (`contents:write`); run `but commit` on a feature branch → exit 0, ref advances
5. Unregister the agent; run another `but commit` → denied with `perm.denied` naming the missing pid (env fallback still allowed because Sprint 10 hasn't hardened the flag yet)
6. Set `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` + `BUT_AGENT_HANDLE=dev`; re-run the commit → succeeds (env fallback works during migration window)

## Tasks

| ID | Title | Agent | Estimate |
|----|-------|-------|----------|
| IDENT-009 | `crates/but-authz/src/config.rs` — `AGENTS_PATH` constant + `AgentWire`/`AgentsWire` + `governance_present` recognizes either file + `load_governance_config` prefers `agents.toml` with deprecation warning | rust-implementer | 240 min |
| IDENT-010 | Update the 8 gate callsites in `but-api` (`commit/gate.rs`, `legacy/merge_gate.rs`, `legacy/governance.rs` ×4, `legacy/forge.rs`, `legacy/config_mutate.rs`) to `resolve_principal_with_registry` | rust-implementer | 180 min |
| IDENT-011 | `crates/but/src/command/agent.rs` — add `migrate` verb: read working-tree `permissions.toml`, rewrite as `agents.toml` (`[[principal]]` → `[[agent]]`), print ref-pin caveat, idempotent | rust-implementer | 180 min |
| IDENT-012 | `crates/but-api/tests/agent_registry.rs` — register→commit→unregister→commit-denied for each of the 4 gate surfaces (commit, merge, admin-write, forge review) | rust-implementer | 240 min |
| IDENT-013 | `crates/but-api/tests/agents_toml_migration.rs` — `permissions.toml` → `but agent migrate` → `agents.toml` byte-equivalent round-trip; legacy-only repo still authorizes | rust-implementer | 120 min |
| IDENT-014 | `crates/but/tests/but/command/agent.rs` — extend with `migrate` snapshots (initial + idempotent re-run + permissions-only warning) | rust-reviewer | 120 min |
| IDENT-015 | `crates/but-authz/tests/config.rs` — extend with `agents.toml` parse + both-formats-prefer-`agents.toml` + deprecation warning emission | rust-reviewer | 90 min |
| IDENT-016 | `crates/but-authz/src/lib.rs` + `src/authorize.rs` doc-comments — export `agents_path`, `Registry`, `resolve_principal_with_registry`; rustdoc the resolution order | rust-implementer | 60 min |

## Source Coverage

- **UC-IDENT-01** (`agents.toml` replaces `permissions.toml` — migration window + byte-equivalent round-trip) — IDENT-009, IDENT-011, IDENT-013, IDENT-015
- **UC-IDENT-03** (Enforced resolution at every gate — the 8 callsites swap) — IDENT-010, IDENT-012
- **UC-IDENT-04** (`but agent migrate` verb) — IDENT-011, IDENT-014
- **UC-IDENT-02** (Runtime PID registry — *consumer* this sprint; producer was Sprint 08) — IDENT-010 consumes `Registry::resolve`
- Test criteria covered: T-IDENT-001..008 (agents.toml format + migration), T-IDENT-016..022 (gate callsite enforcement), T-IDENT-029 (registry round-trip at every gate surface)

## Capability Coverage

- **CAP-AUTHZ-01** — every governed action resolves the principal via `resolve_principal_with_registry` (IDENT-010); the registry hit/miss/`BUT_AUTHZ_ALLOW_ENV_HANDLE` fallback policy is uniform across all 8 callsites.
- **CAP-CONFIG-01** — `load_governance_config` reads `agents.toml` OR `permissions.toml` at the target ref (IDENT-009); `governance_present` recognizes either file (migration window); `but agent migrate` rewrites the working-tree file (IDENT-011) — ref-pin contract preserved.

## Blocks

- **Sprint 10** (IDENT Deprecation Hardening) — the env-only-path deny flip needs both file formats and the migration verb landed.
- **Sprint 11** (IDENT Skills + Docs + Repo Migration) — `but-init` / `but-migrate` skills consume `but agent migrate`; `but-run-sprint` consumes the registry-resolved gates.

## Task Detail Files

Generated by `/but-sprint-tasks-plan` on 2026-06-24 (default mode per the skill NEVER-TIER — dispatched the `rust-planner` surface from RULES.md Specialist Agents table; the `rust-planner` agent was unavailable in this environment so the same-surface `rust-implementer` was used as the fallback specialist. 8/8 tasks fakeability-clean · `proposed_by` tripwire 8/8 · avg rubric ≈112/115 · **red-hat first pass deferred** — re-invoke `/but-sprint-tasks-plan --only IDENT-XXX` for a fresh `rust-reviewer` + `security-auditor` red-hat cycle on any specific task; matches the sprint-08 IDENT-chain precedent of `--skip-review` first pass with per-task re-review available):

- [`IDENT-009-config-agents-toml-loader.md`](./IDENT-009-config-agents-toml-loader.md)
- [`IDENT-010-gate-callsites-registry-swap.md`](./IDENT-010-gate-callsites-registry-swap.md)
- [`IDENT-011-but-agent-migrate-verb.md`](./IDENT-011-but-agent-migrate-verb.md)
- [`IDENT-012-agent-registry-surface-tests.md`](./IDENT-012-agent-registry-surface-tests.md)
- [`IDENT-013-agents-toml-migration-test.md`](./IDENT-013-agents-toml-migration-test.md)
- [`IDENT-014-but-agent-migrate-snapshots.md`](./IDENT-014-but-agent-migrate-snapshots.md)
- [`IDENT-015-config-tests-agents-toml.md`](./IDENT-015-config-tests-agents-toml.md)
- [`IDENT-016-authz-exports-and-resolution-order-rustdoc.md`](./IDENT-016-authz-exports-and-resolution-order-rustdoc.md)
