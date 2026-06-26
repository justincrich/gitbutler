---
sprint: 11
sequence: 13
timeline: Phase 6 — IDENT skills + docs (v1.4.0)
status: In Progress (docs landed; skills + field proof pending; IDENT-028 AC-1/AC-3 blocked-until-C1)
proposed_by: rust-planner (upstream ROADMAP `--no-specialists` declaration; task expansion dispatches rust-planner per RULES.md specialist table)
milestone: sprint-11-ident-skills-docs
prd: ../../README.md
roadmap: ../../ROADMAP.md
generated_by: kb-sprint-tasks-plan
---

# Sprint 11: IDENT Skills + Docs + Repo Migration

**Sequence:** 13
**Timeline:** Phase 6 — IDENT skills + docs (v1.4.0)
**Status:** In Progress (split) — repo-local docs **LANDED** (IDENT-022/023/024) · brain skills + field proof **PENDING** (IDENT-025/026/027/028) · IDENT-028 AC-1/AC-3 **BLOCKED-UNTIL-C1**
**Proposed by:** rust-planner (upstream ROADMAP declared `--no-specialists` for the sprint skeleton; task expansion here dispatches `rust-planner` per the RULES.md Specialist Agents table — `--no-specialists` is "never the default" per the skill NEVER-TIER, and the project resolves `rust-planner` for the `crates/` Rust backend + repo-docs surface)
**Milestone:** — (`sprint-11-ident-skills-docs`)

## Status

This sprint is **not a single flat state** — it splits three ways:

- **LANDED · repo-local docs (IDENT-022/023/024):** committed to this repo (commit `IDENT-022/023/024: document agent identity model in repo docs`) — the RULES.md "Agent identity" subsection, NEW `crates/but-authz/README.md`, and cross-references in `crates/AGENTS.md` / `crates/but/AGENTS.md` / `DEVELOPMENT.md` "Code Hitlist" / `crates/WORKSPACE_MODEL.md`. The build-gate / source-grep ACs (T-IDENT-032..035) are provable inside this repo and are done.
- **PENDING · brain skills + field proof (IDENT-025/026/027/028):** the `but-init` / `but-migrate` / dispatch-skill edits (canonical in `~/Projects/brain/skills/`, mirrored to `~/.claude/skills/`) and the `agent-intel` field migration are not yet landed/proved.
- **BLOCKED-UNTIL-C1 · IDENT-028 AC-1/AC-3 (field gate enforcement):** the registered-`but commit` success receipt (AC-1) and the unregistered `perm.denied` receipt (AC-3) cannot be honestly captured until the **registry-path fix (C1)** propagates — C1 unifies the split where the CLI writes `agents-runtime.toml` while the gate reads `agent-registry.toml`. Until C1 lands, the default-path round-trip is inert and AC-3 only "passes" because an empty registry denies everyone.

> **Evidence-pending.** The end-to-end ACs — **T-IDENT-036/037/038** (brain-repo skill e2e), **IDENT-027 AC-7** (`but agent whoami` round-trip), and **IDENT-028 AC-1/AC-3** (field gate receipts) — are evidence-pending: not yet backed by captured run-logs / gate receipts.

## Overview

The **terminal IDENT sprint** lands the human/agent-facing surface of the identity model now that
the engine (Sprints 08–10) enforces it: the orchestration **skills** stop self-asserting
`BUT_AGENT_HANDLE` and instead register each spawned subagent via `but agent register --pid <child>
--as <agent>`; `but-init` writes `.gitbutler/agents.toml` (not `permissions.toml`); `but-migrate`
performs the rename for existing repos; and this repo's **docs** (RULES.md, a NEW
`crates/but-authz/README.md`, `crates/AGENTS.md`, `DEVELOPMENT.md` Hitlist, `WORKSPACE_MODEL.md`)
document the new identity model and the env-var deprecation timeline. A second governed repo
(`agent-intel`) is migrated end-to-end as the field proof.

This sprint spans **three surfaces**, all owned by the `crates/` Rust + repo-docs specialist
(`rust-planner` → `rust-implementer`/`rust-reviewer`):

1. **This repo's docs** (IDENT-022/023/024) — RULES.md subsection, NEW `crates/but-authz/README.md`,
   cross-references in `crates/AGENTS.md` + `crates/but/AGENTS.md` + `DEVELOPMENT.md` Hitlist +
   `WORKSPACE_MODEL.md`. Verification is **build-gate / source grep** (T-IDENT-032..035).
2. **Brain skills** (IDENT-025/026/027) — `but-init`, `but-migrate`, `but-run-sprint`,
   `but-orchestrate`, `but-sprint-tasks-plan`, `but-sprint-plan` live in `~/Projects/brain/skills/`
   (canonical) and are mirrored to `~/.claude/skills/` — **edit canonical, mirror, then `diff -rq`**.
   `BUT-SKILL-CONVENTIONS.md` §9 (`~/Projects/brain/docs/`) documents the new dispatch model.
3. **Cross-repo migration** (IDENT-028) — run `but agent migrate` against `agent-intel` (and any
   second governed repo); verify an end-to-end governed `but commit` succeeds via the registry path.

> **Scope honesty (T-IDENT-036/037/038).** The skill **e2e** acceptance criteria (re-run `but-init`
> against a fresh fixture → `agents.toml` committed; `but-migrate` → rename committed; `but-run-sprint`
> single-task sprint → implementer registered by the orchestrator, zero `BUT_AGENT_HANDLE` refs in
> dispatch templates) are **owned by the brain repo's skill tests** and reported here only as the
> *contract this repo expects* (per `12-uc-agent-identity.md` IDENT note). The build-gate ACs
> (T-IDENT-032..035, the docs) are fully provable inside this repo; the skill e2e ACs are proved where
> the skills live (brain) and surfaced to `/but-run-sprint`'s human gate.

> **⚠️ Execution prerequisite (sequencing).** This sprint documents and consumes the **final** identity
> policy — it must land **after** Sprint 09 (`but agent migrate` verb + `agents.toml` loader +
> 8-callsite swap) and Sprint 10 (env-handle deny-default locked). The skills cannot honestly stop
> self-asserting `BUT_AGENT_HANDLE` until the registry path is the enforced default at every gate.
> Every task carries `BLOCKED-UNTIL Sprint-10` (transitively Sprint-09); `/kb-run-sprint` (or
> `/but-run-sprint`) MUST schedule Sprint 11 last in the IDENT chain. Planning it ahead is intentional
> (task files are contracts for later execution) and matches the project's plan-ahead pattern.

## Human Testing Gate

**Gate:** Re-running `/but-init` on a fresh fixture repo commits `.gitbutler/agents.toml` (not `permissions.toml`) and `but agent list --committed` shows the roster, while `/but-run-sprint` on a single-task sprint shows the implementer registered by the orchestrator via `but agent register` (no `BUT_AGENT_HANDLE` consumed by the implementer's `but` calls), and `/but-migrate` against a `permissions.toml`-only repo converts and commits the rename in one step.

## Test Deliverable

1. Run `/but-init` on a fresh fixture repo → observe `.gitbutler/agents.toml` committed at the target ref (no `permissions.toml` written)
2. Run `but agent list --committed` → observe the full specialist roster with expected groups
3. Run `/but-migrate` against a fixture with committed `permissions.toml` → observe `agents.toml` written + `permissions.toml` deleted in the same commit
4. Run `/but-run-sprint` on a single-task sprint → observe `but agent register --pid <child> --as <implementer>` called by the orchestrator after spawning the implementer subagent
5. From inside the implementer's process, run `but agent whoami` → observe the registered agent_id (no `BUT_AGENT_HANDLE` env var set in the implementer's shell)
6. Run `but-init` against the `agent-intel` repo → observe `agents.toml` migration committed end-to-end and a sample governed `but commit` succeed via the registry path

## Tasks

| ID | Title | Agent | Estimate |
|----|-------|-------|----------|
| IDENT-022 | `RULES.md` — add "Agent identity" subsection under Conventions (governed repos require `but agent register`; env var is test-only) | rust-implementer | 30 min |
| IDENT-023 | `crates/but-authz/README.md` (NEW) — threat model, file layout, migration path, env-var deprecation timeline, examples | rust-implementer | 120 min |
| IDENT-024 | `crates/AGENTS.md` + `crates/but/AGENTS.md` + `DEVELOPMENT.md` "Code Hitlist" + `crates/WORKSPACE_MODEL.md` — cross-reference the identity README, document `but agent` noun, track the rename | rust-implementer | 60 min |
| IDENT-025 | `but-init` skill (brain) — `scripts/seed-governance.py` emits `[[agent]]` blocks; step [4] writes `agents.toml`; step [4.6] NEW registers specialists via `but agent register`; acceptance changes `but perm list` → `but agent list --committed` | rust-planner | 180 min |
| IDENT-026 | `but-migrate` skill (brain) — detect `permissions.toml`, run `but agent migrate`, commit the rename; idempotent no-op once `agents.toml` exists | rust-planner | 120 min |
| IDENT-027 | `but-run-sprint` + `but-orchestrate` + `but-sprint-tasks-plan` + `but-sprint-plan` skills (brain) — drop `export BUT_AGENT_HANDLE=...` from dispatch templates; orchestrator calls `but agent register --pid <child> --as <agent>` after spawn; `BUT-SKILL-CONVENTIONS.md` §9 documents the new model | rust-planner | 240 min |
| IDENT-028 | Migrate `agent-intel` (and any second governed repo) via `but agent migrate`; verify end-to-end governed action post-migration | rust-implementer | 60 min |

## Source Coverage

- **UC-IDENT-05** (Skill + documentation migration — the entire sprint) — IDENT-022..028
- Test criteria covered:
  - **T-IDENT-032** (RULES.md subsection) → IDENT-022
  - **T-IDENT-033** (`crates/but-authz/README.md` NEW) → IDENT-023
  - **T-IDENT-034** (`crates/AGENTS.md` cross-ref) → IDENT-024
  - **T-IDENT-035** (`DEVELOPMENT.md` Hitlist rename) → IDENT-024
  - **T-IDENT-036** (`but-init` writes `agents.toml` + registers specialists — *brain-repo e2e*) → IDENT-025
  - **T-IDENT-037** (`but-migrate` migrates + commits rename — *brain-repo e2e*) → IDENT-026
  - **T-IDENT-038** (`but-run-sprint` dispatches via `but agent register`, no env self-assert — *brain-repo e2e*) → IDENT-027
  - Field proof (UC-IDENT-05 AC-5/AC-7 end-to-end on a real second repo) → IDENT-028

## Capability Coverage

UC-IDENT-05 is the **documentation + skill-migration consumer layer** — it produces no new capability;
it makes the existing chains usable and honest from the human/orchestrator surface:

- **CAP-AUTHZ-01** — the skills route every governed subagent through a registry registration
  (`but agent register`) instead of a self-asserted env var, so the gate's `resolve_principal_with_registry`
  default (Sprint 08–10) is what actually fires in normal operation (IDENT-025/027); the docs name this
  as the required contract (IDENT-022/023).
- **CAP-CONFIG-01** — `but-init` writes `.gitbutler/agents.toml` and `but-migrate` performs the
  `permissions.toml` → `agents.toml` rename, both inert-until-committed at the target ref; the README +
  Hitlist track the migration window and env-var deprecation timeline (IDENT-023/024/025/026).

## Blocks

- None — terminal sprint in the IDENT chain (and in the v1.4.0 roadmap).

## Dependencies

- **Dependent on:** Sprint 10 (final env-handle deny-default locked) — transitively Sprint 09
  (`agents.toml` loader + `but agent migrate` verb + 8-callsite swap). Every task carries
  `BLOCKED-UNTIL Sprint-10`.

## Task Detail Files

Generated by `/kb-sprint-tasks-plan` on 2026-06-26 (dispatched `rust-planner` per RULES.md Specialist
Agents — `--no-specialists` is never the default; single-surface Rust + repo-docs + brain-skill sprint,
no design planner · 7/7 tasks fakeability-CLEAN via embedded REQUIREMENT-CONTRACT (`validate_scenario`
exit 0) · `proposed_by` tripwire 7/7 · avg rubric 115/115 · stable gapless AC-N/TC-N + populated
`requirements[]` · **full red-hat goal loop, 3 cycles** — fresh `rust-reviewer` + `security-auditor`
each cycle; 13 blocking findings (5 CRITICAL + 8 MEDIUM) resolved by the retained writer + 10 advisory
folded in, **0 upstream escalations**, both panels APPROVE at cycle 3. The loop caught an inverted `!`
denial check, agent-intel's non-`rust-*` roster (oracles now runtime-resolved), the `but agent migrate`
admin-gating that required bootstrap-wrapping, and merge-phase self-registration test-theatre — none
visible to the rubric or the fakeability floor alone):

- [`IDENT-022-rules-agent-identity-subsection.md`](./IDENT-022-rules-agent-identity-subsection.md)
- [`IDENT-023-but-authz-readme.md`](./IDENT-023-but-authz-readme.md)
- [`IDENT-024-cross-reference-docs-hitlist.md`](./IDENT-024-cross-reference-docs-hitlist.md)
- [`IDENT-025-but-init-agents-toml-register.md`](./IDENT-025-but-init-agents-toml-register.md)
- [`IDENT-026-but-migrate-rename.md`](./IDENT-026-but-migrate-rename.md)
- [`IDENT-027-skills-drop-env-handle-register-after-spawn.md`](./IDENT-027-skills-drop-env-handle-register-after-spawn.md)
- [`IDENT-028-agent-intel-field-migration.md`](./IDENT-028-agent-intel-field-migration.md)
