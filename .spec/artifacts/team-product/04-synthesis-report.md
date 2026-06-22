# Synthesis Report: Agents Writing Verifiable Production Code in GitButler

**Date**: 2026-06-20
**Team**: product-manager (DoD), engineering-manager→general-purpose (feature inventory), project-manager (gap analysis)
**Orchestrator**: Claude Code (team-product skill, via Agent-tool orchestration)
**Source deliverables**: `01-definition-of-done.md`, `02-feature-inventory.md`, `03-gap-analysis.md`

> **Read this first — verification corrections.** The orchestrator independently spot-checked every load-bearing claim before synthesizing. Two subagent findings were **wrong** and are corrected here, not relayed:
>
> - ❌ Inventory claimed `frontend.md`, `LINUX.md`, `CONTRIBUTING.md` are **absent**. ✅ **Verified: all three exist** at repo root. (Onboarding docs are fine.)
> - ❌ Gap analysis claimed **11** `todo!()` instances across 5 crates. ✅ **Verified: 5** in production (non-test-support) Rust — 4 in `but-core`, 1 in `but` (the agent conflated `but-testsupport` test-infra instances).
> - ✅ Orchestrator-verified CI structure refines the gap analysis (see §3) — the "unit-only can merge" risk is real but **frontend-scoped, not Rust-scoped**.
>
> This correction pattern is itself evidence for the report's central conclusion: **doctrine is not enforcement**. Three capable agents produced two material errors in a read-only analysis; only direct verification caught them. The same trust gap applies to agent-written code.

---

## Objective (restated)

Define what "good verifiable code" means when an AI agent writes it in GitButler; catalog the features that already empower autonomous production-quality work; identify the gaps blocking fully autonomous verified code production.

---

## Part 1 — What "good verifiable code" means here

Because GitButler **is** a Git engine, "verified" carries a stricter meaning than in a typical app repo. An agent's change is verified production code when it satisfies the **12-point DoD** (`01-definition-of-done.md`), whose load-bearing items are:

1. **Byte-preserving Git semantics** — paths, refnames, commit messages, diff payloads unchanged until UI/API boundaries (RULES.md:74-76). Verification = `insta` snapshots + structural assertions over `but-testsupport` fixtures, _not_ weakened assertions.
2. **Real services, no stubs** — no mocked DB/HTTP/filesystem/git. The only acceptable stub is a flagged, contract-derived, integration-bonded _seam-stub_ that is never reported complete.
3. **Right tier, right surface** — integration/E2E is primary; unit tests only for pure logic; the `but` CLI uses `snapbox` golden tests; the SDK must be regenerated (`pnpm build:sdk`), never hand-edited.
4. **"Watched it work for real"** — run the actual binary/UI against actual services and attach the observation.
5. **Commit discipline + scoped `AGENTS.md` compliance + workspace-model correctness** (`but_graph::Graph` for queries, `graph_rebase::Editor` for mutations).

**The verification bar** (minimum evidence): test output + committed snapshots + a manual-verification log + commit SHA + **zero suppressions** (`@ts-ignore`, `.skip`, `eslint-disable`, weakened assertions) + named real services hit. Missing any one ⇒ not verified.

**JTBD**: "When I am an AI coding agent in GitButler, I want to verify my code against real git operations and real services, so I can ship production code that actually works."

---

## Part 2 — Features that already empower agents (what exists)

GitButler is **genuinely well-instrumented for _deterministic_ verification**, especially on the Rust/engine side. Verified assets:

| Asset                                    | State   | Evidence                                                                                                                                                                                                                                                       |
| ---------------------------------------- | ------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `but-testsupport` scenario/fixture crate | working | `crates/but-testsupport/`, used across 5+ crates                                                                                                                                                                                                               |
| `insta` snapshot testing                 | working | graph/rebase/workspace/repo tests                                                                                                                                                                                                                              |
| `snapbox` CLI golden tests               | working | `crates/but/tests/**`                                                                                                                                                                                                                                          |
| Playwright CT + WebdriverIO blackbox E2E | working | `e2e/playwright/`, `e2e/blackbox/`                                                                                                                                                                                                                             |
| Scoped `AGENTS.md`                       | working | `crates/`, `crates/but/`, `apps/lite/`                                                                                                                                                                                                                         |
| `crates/WORKSPACE_MODEL.md` reference    | working | referenced from AGENTS.md                                                                                                                                                                                                                                      |
| SDK generation + **enforced regen**      | working | `but-sdk-build-check` job runs `generate-sdk-types.sh` and fails on drift (push.yaml:168-169) — **orchestrator-verified**                                                                                                                                      |
| `but` CLI/TUI git workflow               | working | `crates/but/src/args/`, `forge.rs`, `review.rs`                                                                                                                                                                                                                |
| Verification shortcuts                   | working | `pnpm isgood`/`begood`/`check`, `make check`/`clippy`                                                                                                                                                                                                          |
| Strong static CI gating                  | working | `cargo check --workspace --all-targets`, clippy `-D warnings` (3 feature configs), `RUSTFLAGS="--deny warnings"`, `cargo-machete`, `cargo-deny`, `cargo-doc`, Windows check                                                                                    |
| **Governance engine skeleton**           | partial | `but-authz` crate (`Authority`/`Principal`/`Denial`) + integration tests: `commit_gate`, `merge_gate`, `governed_loop`, `perm`, `group`, `review_guard`, `confinement` (`crates/but/tests/but/command/`) — **but-authz crate existence orchestrator-verified** |
| CI self-governance                       | working | `check-no-persist-credentials` job, `zizmr.yml` security scan — agent-authored CI changes get scanned                                                                                                                                                          |

**Strongest takeaway:** an agent working on the Rust engine has a real, fast verification loop (narrow `cargo test -p <crate>`, insta, snapbox, clippy-as-gate). The scaffolding for _doing_ verification is not the problem.

---

## Part 3 — The gaps (orchestrator-verified and reconciled)

The problem is **enforcement and the human-judgment tier**, not tooling. Reconciled gap list:

| #   | Gap                                                   | Impact                                                                                                                         | Orchestrator verification                                                                                                                                                                                                                                                                                                                                                                                              | Priority |
| --- | ----------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- |
| 1   | **No automated stub/placeholder detection**           | `todo!()` ships unguarded; no gate blocks _new_ `todo!()`/`unimplemented!()`/`@ts-ignore`/`eslint-disable`                     | ✅ Confirmed **5** `todo!()` in prod crates (`but-core`×4, `but`×1); no detection job in CI                                                                                                                                                                                                                                                                                                                            | **P0**   |
| 2   | **No test-tier enforcement**                          | CI never _requires_ integration/E2E for a changed surface                                                                      | ✅ Verified: `unittest-node` = Vitest unit (`type=unit`); Playwright CT only on `packages/ui`; full E2E in separate `test-e2e.yml` gated by `check-e2e-changes.yml` whose filter **excludes** `apps/web`, `apps/lite`, `crates/but` — those surfaces merge on unit alone. **Caveat:** Rust PRs _do_ run integration tests via `cargo nextest --workspace`, so this is a **frontend/IPC-scoped** gap, not engine-scoped | **P0**   |
| 3   | **No human-testing-gate automation**                  | "Watched it work" is operator doctrine, not a merge gate                                                                       | ✅ No verification-gate workflow or PR-template requirement in repo                                                                                                                                                                                                                                                                                                                                                    | **P1**   |
| 4   | **No byte-preservation assertion at Git boundaries**  | Agent can normalize Git data and break interop; only ad-hoc `insta` catches it                                                 | Consistent with repo (no centralized boundary-snapshot gate)                                                                                                                                                                                                                                                                                                                                                           | **P1**   |
| 5   | **No "verified" provenance artifact**                 | No replayable evidence a change ran end-to-end                                                                                 | ✅ No `verified-*` artifact or badge                                                                                                                                                                                                                                                                                                                                                                                   | **P1**   |
| 6   | **Parallel-agent/worktree safety is convention only** | No in-repo tooling prevents agent-vs-agent clobbering (relies on global protocol)                                              | Out of repo scope today                                                                                                                                                                                                                                                                                                                                                                                                | **P2**   |
| 7   | **Build friction (Tauri deps)**                       | Full-workspace builds fail/slow without system libs; mitigated by `cargo build -p but` but not documented as an agent shortcut | RULES.md mentions `-p but`; no `.cargo/config.toml` alias / `make agent-check`                                                                                                                                                                                                                                                                                                                                         | **P2**   |

**Closed gap (do not re-open):** SDK regeneration drift — **already enforced** by `but-sdk-build-check` (verified).

### Biggest blind spot

The gap analyst nominated test-tier enforcement (#2) alone. The orchestrator's refinement: the sharpest exposure is **#1 ∧ #2 together** — an agent can ship _placeholder_ or _unit-only-verified_ code through CI because (a) nothing blocks stubs and (b) nothing requires real-service verification on the frontend/IPC paths. On the Rust engine the risk is lower (nextest runs integration tests), but a `todo!()` in `but-core` would still pass today.

### The governance effort is the in-flight structural fix — and it is partially built

`.spec/prds/governance/` (v1.3.0, 129 ACs) is a serious attempt to govern agent work at the _repo_ level (functional permissions, principal groups, commit+merge gates, fail-closed denials, agent-readable steering). Current state (two agents converge; `but-authz` crate + integration tests orchestrator-verified):

- **Engine + tests exist**: `but-authz`, commit/merge gate tests, `governed_loop.rs` reference flow (T-LOOP-006), perm/group/review-guard/confinement tests.
- **Partially complete**: ~30-40% of 129 ACs have real tests; Sprint 04 (mechanism-agnostic gates, per-required-group merge) is "In Progress."
- **Not started**: desktop governance UI (Sprint 06a/06b) — 49 MGMT ACs, 38 component tests blocked on T-MGMT-000. Only `DESIGN-ANNOTATIONS.md` under `apps/desktop/src/components/governance/`.
- **Accepted risk**: R6 (High) — `local_review_verdicts` is forgeable by direct DB write; integrity protection (HMAC/Ed25519) deferred. The merge gate's review requirement is **not** tamper-proof today.

Governance closes the _permission/gate_ dimension; the gaps in §3 close the _verification-enforcement_ dimension. They are complementary, not overlapping.

---

## Cross-team insights

1. **Convergence on the core thesis** — all three specialists independently arrived at: _deterministic scaffolding strong, enforcement weak, human-judgment tier trust-based_. That triangulation is the headline.
2. **The product is a Git engine ⇒ verification must prove byte-preservation**, which is stricter than "tests pass." This raises the bar for what an agent must attach as evidence.
3. **Doctrine is not enforcement (meta-evidence).** The two subagent errors caught above are live proof: even capable agents rationalize and inflate when nothing checks them. The repo's no-stubbing/testing-hierarchy doctrine has the **exact same exposure** until it is backed by CI gates. This is the single most important finding for the user, whose global rules already encode this lesson.

---

## Recommended next steps (prioritized)

1. **[P0] Ship a stub/placeholder-detection CI gate.** Grep `todo!()`/`unimplemented!()` in prod Rust and `@ts-ignore`/`eslint-disable` in TS/Svelte, with a documented whitelist; add as required check. Standalone, cheap, immediate. _(Maps to gap #1.)_
2. **[P0] Add test-tier enforcement.** Require ≥1 integration/E2E test for changed surfaces; make Playwright/blackbox E2E non-conditional for `apps/desktop` + IPC-touching paths (today `check-e2e-changes.yml` skips them). _(Gap #2 — frontend/IPC-scoped.)_
3. **[P1] Operationalize "watched it work."** A `/verify` merge gate + a `verified-{SHA}.json` provenance artifact on merge. _(Gaps #3, #5.)_
4. **[P1] Byte-preservation gate at Git boundaries.** Centralized `insta` snapshot assertions for `but-api` boundary functions handling paths/refs/messages. _(Gap #4.)_
5. **[Continue] Finish the governance roadmap.** Sprint 04 hardening → Sprint 06 UI (unblocks T-MGMT-000 and 38 component tests) → Sprint 07 agent-readable denials. This is the structural fix for _governing_ agent code; steps 1-4 fix _verifying_ it.
6. **[Hygiene] Resolve doc/practice drift.** RULES.md says "Do not use `cargo nextest` for routine validation," yet CI's `rust-test` job uses `cargo nextest run`. Minor, but an agent reading the rules will be confused about the canonical runner. Either align the doc or annotate the exception.

---

## What "done" looks like for autonomy

An agent can be trusted to autonomously ship _verified_ production code in GitButler when: a PR cannot merge with stubs (gap 1), cannot merge with unit-only coverage on a real-service surface (gap 2), carries a provenance artifact proving an end-to-end run (gaps 3/5), and the governance gates actually enforce fail-closed permissions with an integrity-protected review store (R6 closed). Today the deterministic half is ready; the enforcement half is not.
