---
sprint: 10
sequence: 12
timeline: Phase 6 — IDENT hardening (v1.4.0)
status: Complete
proposed_by: rust-planner (upstream ROADMAP `--no-specialists` declaration; task expansion dispatches rust-planner per RULES.md specialist table)
milestone: sprint-10-ident-deprecation
prd: ../../README.md
roadmap: ../../ROADMAP.md
generated_by: kb-sprint-tasks-plan
---

# Sprint 10: IDENT Deprecation Hardening

**Sequence:** 12
**Timeline:** Phase 6 — IDENT hardening (v1.4.0)
**Status:** Complete
**Proposed by:** rust-planner (upstream ROADMAP declared `--no-specialists` for the sprint skeleton; task expansion here dispatches `rust-planner` per the RULES.md Specialist Agents table — `--no-specialists` is "never the default" per the skill NEVER-TIER, and the project resolves `rust-planner` for the `crates/` Rust backend surface)
**Milestone:** — (`sprint-10-ident-deprecation`)

## Overview

The **third IDENT sprint** closes the deprecation arc: on a governed repo, a self-asserted
`BUT_AGENT_HANDLE=dev` (no registry hit, no `BUT_AUTHZ_ALLOW_ENV_HANDLE=1`) must be **denied**
`perm.denied` naming the unregistered pid — not silently honored. The registry path is the default;
the env-var path survives only as an opt-in test/CI escape hatch behind the flag. This is the policy
Sprint 09 left migration-friendly; Sprint 10 hardens it and proves the invariant holds at **every**
gate surface, not just commit.

> **Grounding note (codebase reality, verified 2026-06-26).** The resolver-level flip is **already
> landed**: `crates/but-authz/src/authorize.rs` `resolve_principal_with_registry` (Sprint 08 /
> IDENT-003) implements registry-hit → principal; registry miss + `BUT_AUTHZ_ALLOW_ENV_HANDLE=1`
> → env fallback; else → `Denial::unregistered(pid)` (lines ~206–245). So IDENT-017's "flip the
> default" is **not** a change to `authorize.rs` — the deny-default is in place at the resolver. The
> gate callsite migration is now wrapper-shaped in the current tree: governed `but-api` gates call
> `resolve_principal_with_runtime_registry`, and that wrapper delegates to
> `but_authz::resolve_principal_with_registry(Some(&registry), cfg)`. Sprint 10 therefore hardens
> the policy with resolver tests, Track A env-fallback test migration, Track B registry-path tests,
> wrapper-aware build-gate invariants, and doc-comment audits rather than forcing direct calls at
> every gate surface.

Sprint 10 depends on Sprint 09 (both file formats + migration verb + runtime-registry
callsite migration). Sprint 11
(skills + docs + repo migration) is the downstream consumer — it needs the **final** deny-default
policy locked before the skills stop self-asserting `BUT_AGENT_HANDLE`.

> **⚠️ Execution prerequisite (red-hat, refreshed 2026-06-26).** The Human Testing Gate above describes
> the **end state after Sprint 09 + Sprint 10 have both landed**. Current source already shows the
> runtime-registry wrapper callsite shape and `AGENTS_PATH`; Sprint 10 still depends on Sprint 09's
> config/migration/test artifacts being present because its tasks extend and lock those contracts. If
> `/kb-run-sprint` executes this folder on a branch where those Sprint 09 artifacts are missing, the
> task-level `BLOCKED-UNTIL Sprint-09` notes remain authoritative.

## Human Testing Gate

**Gate:** On a governed repo, a commit attempted via `BUT_AGENT_HANDLE=dev` alone (no registry hit, no `BUT_AUTHZ_ALLOW_ENV_HANDLE=1`) is denied `perm.denied` naming the unregistered pid, while the same call with the flag set succeeds, and the same call with the agent registered (no flag, no env var) also succeeds.

## Test Deliverable

1. On a governed repo, run `BUT_AGENT_HANDLE=dev but commit` with NO registry hit and flag unset → denied `perm.denied` with the unregistered-pid message
2. Run the same governed commit command with `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` prefixed and an empty registry → succeeds (test/CI escape hatch; automated proof: `cargo test -p but-api --test gate_registry_swap -- env_fallback_still_allowed_on_registry_miss`)
3. Register the agent via `but agent register --as dev` (no flag, no env var); re-run `but commit` → succeeds (registry path is the default; automated CLI proof: `cargo test -p but --test but --features legacy,but-2 -- command::commit_gate::commit_gate_operator_runtime_registry_sequence`)
4. Unregister via `but agent unregister`; re-run `BUT_AGENT_HANDLE=dev but commit` without the flag → denied again
5. Trigger a merge attempt via `BUT_AGENT_HANDLE=maint` (no flag, no registry) → denied `perm.denied` (every gate surface, not just commit)
6. Seed an expired current-process registry entry, with `BUT_AGENT_HANDLE` and `BUT_AUTHZ_ALLOW_ENV_HANDLE` unset; run a governed commit gate → denied `perm.denied`, or the runtime registry load/gate path proves `Registry::gc(now)` runs before resolution (automated proof: `cargo test -p but-api --test gate_registry_swap -- expired_current_process_registry_entry_denied`)
7. Seed spoofed/stale process identities with wrong `start_time` for the current pid and wrong pid/current start-time mismatch; run governed commit gates with env unset → both deny `perm.denied` (automated proofs: `cargo test -p but-api --test gate_registry_swap -- current_pid_wrong_start_time_denied_at_commit_gate` and `cargo test -p but-api --test gate_registry_swap -- wrong_pid_current_start_time_denied_at_commit_gate`)

## Tasks

| ID | Title | Agent | Estimate |
|----|-------|-------|----------|
| IDENT-017 | `crates/but-authz/src/authorize.rs` — flip default: env-var path on governed repos requires `BUT_AUTHZ_ALLOW_ENV_HANDLE=1`; absent flag + registry miss → `Denial::unregistered` | rust-implementer | 90 min |
| IDENT-018 | Mechanical update of the 80+ `but-api` tests using `temp_env::with_var("BUT_AGENT_HANDLE", ...)` to set `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` in their helpers (Track A — no churn, keep working) | rust-implementer | 180 min |
| IDENT-019 | Add registry-only Track B coverage under `tests/agent_registry.rs` plus a CLI blackbox operator sequence: empty registry + flag-set env fallback succeeds; `but agent register --as dev` makes governed commit succeed without env; unregister, expired entries, and stale/spoofed process identity return `perm.denied` | rust-implementer | 180 min |
| IDENT-020 | Extend `crates/but-authz/tests/invariant_build_gates.rs`: add `registry.rs` + `process.rs` to `ENFORCEMENT_PATHS`; assert governed gates use `resolve_principal_with_runtime_registry` and that wrapper delegates to `but_authz::resolve_principal_with_registry`; negative grep on direct `BUT_AGENT_HANDLE` env reads outside `authorize.rs`; require TTL/process-identity negative controls; `AGENTS_PATH` constant required; `PERMISSIONS_PATH` `#[deprecated]` | rust-reviewer | 180 min |
| IDENT-021 | Audit doc-comments across the 11 runtime-registry identity surfaces — each names the resolution order (registry → flag-gated env → denial) so the invariant is documented in code, not just tested | rust-reviewer | 90 min |

## Source Coverage

- **UC-IDENT-03** (Enforced resolution at every gate — the deny-default is the FULL enforcement default) — IDENT-017, IDENT-019, IDENT-020, IDENT-021. The authoritative current runtime-registry identity surface set is 11 callsites: `commit/gate.rs::enforce_commit_gate_for_target`, `legacy/merge_gate.rs::enforce_merge_gate`, `legacy/config_mutate.rs::enforce_administration_write_gate`, `legacy/forge.rs::authorize_branch_action`, `legacy/rules.rs::list_workspace_rules_scoped_for_caller`, and `legacy/governance.rs::{governance_status_read, branch_gates_read_with_repo, group_list_with_repo, perm_list_with_repo, whoami_with_repo, can_i_with_repo}`.
- **UC-IDENT-01** (deprecation arc — `permissions.toml` → `agents.toml`; `PERMISSIONS_PATH` deprecation + `AGENTS_PATH` invariant) — IDENT-020
- Test criteria covered: T-IDENT-018 (flag-unset denial) + the invariant suite (T-IDENT-020 enforcement)

## Capability Coverage

- **CAP-AUTHZ-01** — every governed action resolves through `resolve_principal_with_runtime_registry`, which loads the runtime registry and delegates to `but_authz::resolve_principal_with_registry(Some(&registry), cfg)`; the deny-default (registry miss + flag unset → `Denial::unregistered`) is proven at representative API and CLI surfaces (IDENT-017/019), expired entries and spoofed/stale pid/start_time identities deny at real governed gates (IDENT-019/020), the flag-set governed commit escape hatch is explicitly proven (IDENT-019), the exact 11-callsite set is codified as a build-gate invariant (IDENT-020), and the same 11 surfaces carry doc-comment proof (IDENT-021). The env-var path is opt-in only.
- **CAP-CONFIG-01** — Sprint 10's deprecation invariant is intentionally narrow:
  `AGENTS_PATH` is the required constant and `PERMISSIONS_PATH` carries `#[deprecated]`
  (IDENT-020). The target-ref-only load behavior, `agents.toml` preference, and legacy
  `permissions.toml` fallback are Sprint 09 prerequisites evidenced by Sprint 09 config
  tests; Sprint 10 depends on them and does not re-prove that loader behavior.

## Blocks

- **Sprint 11** (IDENT Skills + Docs + Repo Migration) — `but-init` / `but-migrate` / `but-run-sprint` skills reflect the **final** policy (no env self-assert); the invariant must be locked first.

## Task Detail Files

Expanded by `/kb-sprint-tasks-plan` on 2026-06-25 (dispatched Rust planning per RULES.md Specialist Agents — `--no-specialists` is never the default; no design planner, single-surface Rust sprint · 5/5 tasks fakeability-CLEAN via embedded REQUIREMENT-CONTRACT · `proposed_by` tripwire 5/5 · stable gapless AC-N/TC-N + populated `requirements[]` · contract validation clean: scenarios 4/4/11/6/9 · final fresh security-auditor review clean · final focused code-reviewer re-review clean after source-oracle fixes):

- [`IDENT-017-resolver-deny-default-lock-verify.md`](./IDENT-017-resolver-deny-default-lock-verify.md)
- [`IDENT-018-track-a-env-flag-test-migration.md`](./IDENT-018-track-a-env-flag-test-migration.md)
- [`IDENT-019-track-b-registered-agent-helper.md`](./IDENT-019-track-b-registered-agent-helper.md)
- [`IDENT-020-invariant-build-gates-extension.md`](./IDENT-020-invariant-build-gates-extension.md)
- [`IDENT-021-gate-callsite-doc-audit.md`](./IDENT-021-gate-callsite-doc-audit.md)
