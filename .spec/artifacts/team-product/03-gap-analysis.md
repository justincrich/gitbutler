# Gap Analysis: Autonomous Verified Code in GitButler

**Date**: 2026-06-20  
**Context**: Task 3 — Adversarial gap analysis for autonomous agent verification  
**Scope**: What is MISSING for an AI agent to autonomously produce verified production code in GitButler, end-to-end

## Executive Summary

GitButler has **strong operator doctrine** (no stubbing, real services, integration-first) but **weak automated enforcement**. The governance PRD (`.spec/prds/governance/`) closes critical gaps around permissions and gates, but **9 residual gaps** remain—most critically: **no automated stub-detection**, **no test-tier enforcement in CI**, and **happy-path-only CLI coverage**. A confidently-wrong agent can ship broken code through trust-based conventions alone.

---

## Gap Table

| Gap                                                 | Impact on Autonomous Verified Code                                                                                                                                                                                                         | Evidence (repo, with path)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          | Already Covered by Governance Spec?                                                                                                                                                | Proposed Closing Sprint                    | Priority |
| --------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------ | -------- |
| **1. Automated stub-detection**                     | HIGH — Agent can leave `todo!()`, `unimplemented!()`, or hand-edit generated SDK and CI passes                                                                                                                                             | - `todo!()` in 11 files: `but-testsupport/src/in_memory_meta.rs`, `but-testsupport/src/sandbox.rs`, `but-workspace/src/commit/mod.rs`, `but-graph/src/init/walk/mod.rs`, `but-core/src/snapshot/mod.rs` (3×), `but/src/lib.rs`, `but/src/command/legacy/discard.rs`<br>- `unimplemented!()` not found in production (good)<br>- Only guard: one test in `crates/but/tests/but/command/group.rs` checking group surfaces for placeholders—**not a CI gate**<br>- `@ts-ignore`/`eslint-disable` used in 20+ files with no enforcement | PARTIAL — Governance spec has build-gate criteria (T-AUTHZ-016, T-GATES-008) that grep for structural invariants, but **no general stub-detection gate**                           | Sprint 1: Stub-Detection CI Gate           | **P0**   |
| **2. Test-tier enforcement in CI**                  | HIGH — CI runs unit tests (`pnpm test`, `cargo test`) but **does not enforce integration/E2E as primary**. An agent can add unit-only coverage and pass CI without real-service verification.                                              | - `.github/workflows/push.yaml`: `unittest-node` job runs `pnpm test` (unit only)<br>- `rust-test` job runs `cargo test` (no tier segregation)<br>- **No job requires `test:e2e:playwright` or `test:e2e:blackbox` to pass before merge**<br>- E2E jobs are conditional: `if: ${{ needs.changes.outputs.should_run == 'true' }}` (run on changes, not enforced as primary)                                                                                                                                                          | NO — Governance spec has **129 criteria** (67 integration, 38 component, 7 API-contract, 15 build-gate, 2 E2E) but **no CI enforcement that integration/E2E are required to pass** | Sprint 2: Test-Tier CI Enforcement         | **P0**   |
| **3. "Happy-path only" CLI coverage**               | MEDIUM — RULES.md admits "CLI tests are expensive — happy-path only" (line 190). 46 test files cover `but` commands, but error paths, edge cases, and failure modes are untested. Agent can break error handling and CI won't catch it.    | - `crates/but/tests/but/command/` has 46 `.rs` files with tests (confirmed via `find` + `grep #\[test\]`)<br>- `commit.rs`: 73 tests, `commit2.rs`: 38 tests (happy-path heavy)<br>- **No admission of negative/edge-case test coverage in CLI suite**<br>- RULES.md: "CLI tests are expensive — happy-path only, test what really matters."                                                                                                                                                                                        | NO — Governance spec's integration tests (T-AUTHZ-009 through T-AUTHZ-017) cover **denial paths**, but only for the **new authz/gates surface**, not existing `but` CLI behavior   | Sprint 2: CLI Error-Path Coverage (subset) | **P1**   |
| **4. Human-testing-gate automation**                | MEDIUM — Global operator doctrine requires "not done until watched working for real" (per global CLAUDE.md), but **no in-repo gate enforces this**. Agent can claim "done" without running the code.                                       | - No `.github/workflows/human-testing-gate.yml` or similar<br>- No PR template requiring E2E verification evidence<br>- No structured "verify it works for real before merge" gate in repo (purely convention-based)<br>- Governance spec has **human testing gates** (T-LOOP-006, T-MGMT-XXX) but they are **documented criteria, not automated**                                                                                                                                                                                  | PARTIAL — Governance spec defines **129 human-testing criteria** but **no automated gate** prevents merge without human verification signature                                     | Sprint 3: Human-Testing-Gate Automation    | **P1**   |
| **5. Parallel-agent / worktree safety enforcement** | LOW-MEDIUM — Global protocol exists (`.claude/CLAUDE.md` "Parallel Subagent Dispatch Protocol"), but **no in-repo tool** prevents agents from clobbering each other's changes. Safety is entirely operator convention.                     | - No `crates/*` or `.github/*` tooling for worktree safety<br>- No pre-commit hook detecting concurrent agent writes<br>- RULES.md: "Assume the worktree may contain other agents' / the user's changes" (advisory, not enforced)<br>- Governance spec **does not address** parallel-agent coordination                                                                                                                                                                                                                             | NO — Out of scope for governance POC (functional permissions only)                                                                                                                 | Sprint 4: Worktree-Safety Tooling          | **P2**   |
| **6. Byte-preserving Git-semantics verification**   | MEDIUM — RULES.md requires "Keep Git paths, refnames, commit messages, and diff payloads byte-preserving until UI/API boundaries" (line 74-75), but **no automated check** enforces this. Agent can normalize/transform and break interop. | - No `.github/workflows/*` job checking byte preservation<br>- No `insta` snapshot gate for Git boundaries<br>- `insta` used in 92/311 test files, but **no centralized assertion of byte-preservation at API boundaries**<br>- `WORKSPACE_MODEL.md` emphasizes commit/refs/graph primitives but no enforcement                                                                                                                                                                                                                     | NO — Not covered by governance spec (focuses on permissions/gates, not Git semantics)                                                                                              | Sprint 3: Byte-Preservation Gate           | **P1**   |
| **7. SDK regeneration enforcement**                 | **LOW (GOOD)** — CI **already enforces** SDK regeneration. `generate-sdk-types.sh` fails if generated files are out-of-date (line 10-16). Agent hand-editing or forgetting regen is caught.                                                | - `.github/workflows/push.yaml` lines 168-169: `./scripts/generate-sdk-types.sh`<br- Script exits 2 if `pnpm format` detects changes (e.g., hand-edited `packages/but-sdk/src/generated/`)<br>- **This gap is CLOSED**                                                                                                                                                                                                                                                                                                              | NO — Not governance's concern (already enforced)                                                                                                                                   | **N/A** (closed)                           | —        |
| **8. Build friction for agents**                    | MEDIUM — `cargo build` (whole workspace) needs Tauri system deps and is slow/fail-prone. RULES.md suggests `cargo build -p but` (narrow), but **no CI pattern** guides agents to build-check narrowly.                                     | - RULES.md line 102: `cargo build -p but` (no Tauri deps)<br>- Line 101: `cargo build` (needs Tauri deps, fails without system libs)<br>- CI uses `cargo build -p gitbutler-git -p but-server -p but` in E2E, but **no documented "agent-friendly" build pattern** in repo                                                                                                                                                                                                                                                          | PARTIAL — Governance spec's build-gate criteria (T-AUTHZ-016, T-GATES-008) use **grep-based checks**, not build-heavy patterns (smart, but not explicit)                           | Sprint 4: Agent-Friendly Build Patterns    | **P2**   |
| **9. "Verified" provenance**                        | HIGH — **No artifact/label** proves a change was run end-to-end. Agent can claim "it works" with zero evidence. Trust-based; no replayable verification trail.                                                                             | - No `.github/` workflow emitting a "verified" badge/artifact<br>- No PR template requiring verification link/evidence<br>- No `VERIFIED.md` or similar provenance file in repo<br>- Operator doctrine requires E2E verification but **no in-repo signal** that it happened                                                                                                                                                                                                                                                         | NO — Governance spec's 129 criteria are **test specifications**, not provenance artifacts                                                                                          | Sprint 3: Verification-Provenance Badge    | **P1**   |

---

## Prioritized Sprint Roadmap

### Sprint 1: Stub-Detection CI Gate (P0)

**Goal**: Close the #1 highest-risk gap — prevent agents from shipping placeholder code.  
**What gets verified**: Every PR must pass a CI job that greps for `todo!()`, `unimplemented!()`, and `@ts-ignore`/`eslint-disable` (with whitelist for documented exceptions).

**Scope**:

- Add `.github/workflows/stub-detection.yml` job that:
  - Greps `crates/**/*.rs` for `todo!()`, `unimplemented!()` (whitelist `but-testsupport`'s `todo!("Check manually")` etc.)
  - Greps `apps/**/*.svelte`, `apps/**/*.ts`, `packages/**/*.ts` for `@ts-ignore`, `eslint-disable` (whitelist func-style suppressions)
  - Fails if any found outside whitelist
- Update `push.yaml` to require this job before merge
- Document whitelist in `CLAUDE.md` for transparency

**Dependencies**: None (standalone gate)

**Success criteria**:

- PR with `todo!()` in production code fails CI
- PR with `@ts-ignore` suppressing a type error fails CI
- False-positive rate < 5% (measured over 10 historical PRs)

---

### Sprint 2: Test-Tier CI Enforcement (P0)

**Goal**: Enforce integration/E2E as primary — unit tests alone are insufficient to claim "done."

**What gets verified**: CI requires **at least one integration or E2E test** covering the changed surface before merge. Unit tests are allowed but **not sufficient**.

**Scope**:

- Add `.github/workflows/test-tier-enforcement.yml` that:
  - Parses PR diff for touched surfaces (e.g., `but-api`, `but-workspace`, `apps/desktop/routes`)
  - Checks that **≥1 integration-test or E2E-test** exists for the touched module (mapping in `.github/test-surface-map.json`)
  - Fails if only unit tests exist for a changed surface
- Update `push.yaml` to make `test:e2e:playwright` and `test:e2e:blackbox` **required checks** (not conditional on `should_run`)
- Document test-tier expectations in `frontend.md` and `CLAUDE.md`

**Dependencies**: Sprint 1 (stub-detection must pass first)

**Success criteria**:

- PR adding new `but-api` function without integration test fails CI
- PR changing Svelte route without component/E2E test fails CI
- Historical PRs (last 20) have > 80% compliance (baseline measurement)

**Subset**: CLI Error-Path Coverage (P1)

- Add **negative-path tests** for top 5 `but` commands (commit, merge, branch, rebase, push) as a P1 scoped slice of full coverage.

---

### Sprint 3: Human-Testing-Gate + Byte-Preservation + Provenance (P1)

**Goal**: Close verification gaps — enforce "watched it work for real" and prove it happened.

**What gets verified**:

1. PR cannot merge without a **verification comment** from the author (`/verify <command>` or `/verify-e2e <link>`)
2. Git boundaries enforce **byte-preservation** via snapshot tests
3. Merge emits a **VERIFIED badge** artifact for traceability

**Scope**:

1. **Human-testing-gate automation**:
   - Add `.github/workflows/verification-gate.yml` that blocks merge until `/verify` comment is present
   - Comment triggers a **verification run** (e.g., `pnpm test:e2e:playwright --grep "<PR-changes>"`) and posts results
   - Only PR author (or designated verifier) can post `/verify`
2. **Byte-preservation gate**:
   - Add `insta` snapshot tests for all `but-api` boundary functions that handle Git paths/refs/messages
   - CI job `byte-preservation-check.yml` fails if snapshots drift (force-update requires PR review)
3. **Verification-provenance badge**:
   - On successful merge, emit a `verified-{SHA}.json` artifact containing: verification run link, tests passed, timestamp
   - Add `VERIFIED.md` section to PR template referencing the artifact

**Dependencies**: Sprint 2 (test-tier enforcement ensures integration tests exist to verify)

**Success criteria**:

- PR without `/verify` comment cannot merge
- PR modifying Git path handling fails if byte-preservation snapshots drift
- `verified-{SHA}.json` artifact exists for every merge on `master`

---

### Sprint 4: Worktree-Safety + Agent-Friendly Builds (P2)

**Goal**: Reduce parallel-agent collision risk and improve build-check efficiency for agents.

**What gets verified**:

1. Agents can work in isolated worktrees without clobbering each other
2. Agents can narrow-build/check without Tauri friction

**Scope**:

1. **Worktree-safety tooling**:
   - Add pre-commit hook `.git/hooks/pre-commit-agent-safe` that:
     - Detects concurrent agent writes via `GIT_AUTHOR_IDENT` heuristics
     - Warns if `git status --porcelain` shows uncommitted changes from other agents
   - Document worktree usage in `CLAUDE.md` ("use `git worktree add` for parallel agent work")
2. **Agent-friendly build patterns**:
   - Add `.cargo/config.toml` aliases: `cargo check-but`, `cargo test-but` (narrow, no Tauri)
   - Document in RULES.md: "Prefer `cargo build -p but` over full workspace for agent checks"
   - Add `Makefile` target: `make agent-check` (runs narrow checks + lint, no full build)

**Dependencies**: Sprint 3 (verification infrastructure reduces collision urgency)

**Success criteria**:

- Pre-commit hook fires when multiple agents commit to same worktree within 5min window
- `cargo check-but` completes in < 30s (vs 5min for full workspace)
- Agent workflows documented in `CLAUDE.md` with examples

---

## Biggest Blind Spot

**The single gap most likely to let a confidently-wrong agent ship broken code undetected is:**

### **Gap #2: Test-Tier Enforcement in CI**

**Why this is the killer blind spot**:

- An agent can add **100% unit test coverage** (all green, all passing) and CI will happily merge the PR — **even if the code has never been run against a real database, real git, or real HTTP service**.
- Unit tests can mock everything — they prove **nothing about real-world behavior**.
- GitButler's operator doctrine forbids stubbing, but **CI does not enforce it**. The doctrine is trust-based; an agent optimized to "pass all tests" will choose unit tests (fast, easy) over integration tests (slow, fragile).
- Governance spec's 129 integration/E2E criteria are **paper specifications** until CI blocks unit-only PRs.

**What would slip through**:

- A bug in the `but-api` → `but-workspace` commit path that only manifests with real git repo state
- A DB transaction error that never appears with an in-memory mock
- A Tauri IPC handler that times out only under real desktop load

**Why Sprint 2 is the P0 fix**:

- Without test-tier enforcement, **all other verification gates are circumventable**. An agent can pass stub-detection (no `todo!()`), pass human-testing (with unit-only "verification"), and emit a provenance badge — all without ever running the code against real services.

**Evidence from repo**:

- `.github/workflows/push.yaml` `unittest-node` job runs `pnpm test` (unit only) and is **not blocked** by any integration/E2E requirement
- E2E jobs are conditional: `if: ${{ needs.changes.outputs.should_run == 'true' }}` — they can be **skipped** on some PRs
- No CI grep/pattern enforces "integration test required for surface X"

---

## Summary

| Top 3 Gaps                                               | Proposed Sprint                                               | Priority |
| -------------------------------------------------------- | ------------------------------------------------------------- | -------- |
| 1. No automated stub-detection (`todo!()`, `@ts-ignore`) | Sprint 1: Stub-Detection CI Gate                              | P0       |
| 2. No test-tier enforcement (unit-only can pass CI)      | Sprint 2: Test-Tier CI Enforcement                            | P0       |
| 3. No human-testing-gate automation (trust-based "done") | Sprint 3: Human-Testing-Gate + Byte-Preservation + Provenance | P1       |

**Biggest blind spot**: Test-tier enforcement (Gap #2) — without it, all other gates are bypassable via unit-only coverage.

**Next step**: Execute Sprint 1 (stub-detection) → Sprint 2 (test-tier) before any autonomous agent is trusted to merge code. Governance spec closes permission/gate gaps; these sprints close the **verification-enforcement gaps**.
