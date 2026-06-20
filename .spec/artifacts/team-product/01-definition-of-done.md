# Definition of Done: When Agent-Written Code is Verified Production Code

**Product Manager**: product-manager  
**Initiative**: Agent-Verification-DoD  
**Version**: 1.0.0  
**Last Updated**: 2026-06-20  

---

## JTBD: The Agent as User

**Job Statement**: "When I am an AI coding agent working in a GitButler repository, I want to verify my code against real services and real git operations, so that I can confidently ship production code that actually works."

### Four Forces

| Force | Finding |
|-------|---------|
| **Push (away from current)** | Current agent workflows produce stubbed tests, mocked services, and "happy-path only" validation that break in production. The cost of discovering these gaps late (blocked deploys, hotfixes, rollback races) drives agents toward better verification. |
| **Pull (toward new solution)** | Real verification against actual GitButler crates (`but-testsupport`, `but-api`, `but-workspace`), real git repositories, and real engine behavior gives agents confidence their code survives production. The definition makes this state explicit and achievable. |
| **Anxiety (about new)** | Agents worry that running real services is slower than mocks, that real tests are flaky, or that watching code work "for real" is wasted time when unit tests pass. The definition must answer why this investment prevents bigger failures. |
| **Habit (of current)** | Agents default to narrow unit tests and mocked interfaces because that's what existing tooling rewards. The definition must break this habit by making real verification the cheapest path to "done." |

---

## Definition of Done Criteria

An agent's change is **verified production code** when it satisfies ALL of the following:

### 1. Narrowest Test First, Then Expand

- **Rust crates**: Run `cargo test -p <crate> <test-name>` for the narrowest relevant test before any workspace-wide check. Quoted from CLAUDE.md:112-115: *"Run the **narrowest relevant test or check first** — do not default to workspace-wide runs. Use workspace-wide checks (`make check`, `make clippy`) only when the change affects shared contracts or multiple crates."*
- **Frontend**: Run `pnpm test:ct -- <Component>` for a single component test, or `cd <package> && pnpm test -- -t <test-name>` for a focused unit test (CLAUDE.md:143-150).
- **Evidence**: Test output shows the specific test passing before any broader suite runs.

### 2. Graph/Rebase/Workspace: Byte-Preserving `insta` Snapshots + Structural Assertions

- For any change affecting Git graph, workspace, branch, stack, commit, or rebase relationships, use **fixture-backed before/after `insta` snapshots** plus targeted structural assertions. CLAUDE.md:178-182: *"For graph/rebase/workspace behavior, prefer fixture-backed before/after `insta` snapshots plus targeted structural assertions. Stabilize/normalize volatile output rather than weakening assertions."*
- Use `but-testsupport` for scenario creation; never `std::env::temp_dir().join(format!(…))` (CLAUDE.md:178-179).
- Explain assertions with `assert!(…, "why")` and `insta::assert_debug_snapshot(x, "why", @r"")` (CLAUDE.md:183-184).
- **Evidence**: `.snap` files in the crate showing byte-preserving before/after states; commit messages reference the snapshot assertions.

### 3. `but` CLI: Golden Tests with `snapbox`

- For CLI behavior, use `snapbox` golden tests with `env.but(...).assert()`, `.stdout_eq`/`.stderr_eq`, and `[..]`/`...` wildcards (CLAUDE.md:186-188). Update with `SNAPSHOTS=overwrite cargo test -p but` (CLAUDE.md:187-188).
- Use sandbox helpers (`env.invoke_bash`/`env.invoke_git`), not raw `std::process::Command::new("git")` (CLAUDE.md:188-189).
- CLI tests are expensive — test happy-path only, what really matters (CLAUDE.md:190).
- **Evidence**: `.snap` files in `crates/but/tests/` showing exact CLI stdout/stderr for the happy path.

### 4. Frontend: Component + E2E, No "Happy-Path Only" Gaps

- **Component tests**: `pnpm test:ct` over real `packages/ui` and desktop components, no mocked UI. Per governance PRD E2E criteria (11-e2e-testing-criteria.md:12): *"Verification bar: real `but-authz` + real `but-api` + real git, no mocks."*
- **E2E**: `pnpm test:e2e:playwright` or `pnpm test:e2e:blackbox` for full-flow validation.
- **No happy-path only gaps**: Error states, loading states, empty states, and denial states must all have assertions. A "happy-path only" test is a **stub** of the unhappy paths.
- **Evidence**: Test coverage includes error/denial branches; component tests render real UI states, not mocks.

### 5. SDK Surface: Regenerated, Not Hand-Edited

- After changing Rust APIs or types exposed through `@gitbutler/but-sdk`, run `pnpm build:sdk && pnpm format` to regenerate `packages/but-sdk/src/generated` (CLAUDE.md:91-94).
- **Never** hand-edit generated files (CLAUDE.md:93).
- **Evidence**: Commit shows `packages/but-sdk/src/generated/` changed by the regeneration script, not by hand edits; TypeScript typechecks pass against the new SDK.

### 6. Git Semantics: Byte-Preserving Until UI/API Boundaries

- Preserve Git paths, refnames, commit messages, and diff payloads **byte-preserving** until UI/API boundaries (CLAUDE.md:74-76).
- Do not dedupe, reorder, or smooth graph data unless Git semantics allow it (CLAUDE.md:73-74).
- **Evidence**: Tests assert exact byte strings for git metadata; snapshots show raw commit messages and refnames unchanged.

### 7. No Stubbed Core Logic (Operator Doctrine)

- **Never stub, mock, or fake** core business logic, HTTP calls, database operations, filesystem I/O, or external services. Per operator doctrine: *"Stubbing core logic is a lie. It erodes trust in AI coding tools and makes a task look complete while the product is still broken."*
- **The only acceptable stub** is a *seam-stub* at a **locked concurrency-contract seam** that is flagged, contract-derived, honestly gated, and integration-bonded (all four required). Even then, it is **never reportable as complete** (brain/docs/CONCURRENCY-AND-INTEGRATION.md).
- **Evidence**: Production code calls real services; integration/E2E tests hit real databases, real HTTP endpoints, real filesystem. No `jest.mock()`, no `mockImplementation()`, no synthetic success returns.

### 8. Watched It Work For Real

- **You are not done until you have watched the actual code work against actual services.** Not "the tests pass," not "all green," not "coverage looks good" — **until you have watched it work end-to-end with real services, you are not done** (operator doctrine).
- For Rust: run the compiled binary or integration test and observe real git operations, real DB writes, real IPC.
- For frontend: run `pnpm dev:desktop` / `pnpm dev:web` / `pnpm dev:lite` and interact with the feature manually, observing real Tauri IPC, real SDK calls, real state transitions.
- **Evidence**: Agent report includes a "Verification" section describing the manual run: what binary was executed, what services were hit, what outputs were observed.

### 9. Commit Discipline: Leave the Tree Better Than You Found It

- Commit the change with a clear message describing **why** and **what changed** (CLAUDE.md:215).
- Before saying "done," run `git status` — if there are uncommitted changes, you are **not done** (operator doctrine: Boy Scout Rule, Commit Discipline).
- If pre-commit hooks fail (tests, lint, clippy), fix the **underlying cause**, not the test (operator doctrine: iron rule).
- **Evidence**: `git log` shows a commit containing the change; `git diff` against main is clean; pre-commit hooks passed.

### 10. Scoped `AGENTS.md` Compliance

- For scoped work (Rust under `crates/`, `but` CLI, Lite, graph/workspace), read and follow the scoped `AGENTS.md` (CLAUDE.md:220-226).
- **Evidence**: Change follows patterns documented in the scoped `AGENTS.md`; if no scoped file exists, the agent references CLAUDE.md as the fallback.

### 11. Workspace/Graph Model: Prefer Git-Representable Concepts

- For graph/workspace/branch/stack/commit work, use **`but_graph::Graph` for relationship questions** and **`but_rebase::graph_rebase::Editor` for mutations** (WORKSPACE_MODEL.md:30-33).
- Avoid legacy abstractions (`but-rebase::Rebase`, workspace projection as source of truth) unless a legacy boundary requires it (WORKSPACE_MODEL.md:28-36).
- **Evidence**: Code uses `but_graph::Graph` for queries and `graph_rebase::Editor` for mutations; commit message references the model guidance.

### 12. Human Testing Gates: Don't Refuse, Verify

- When a sprint completes and the documented binary/subcommand doesn't exist yet, **don't refuse the gate** — prove the claim by composing library modules into a verification harness (operator doctrine: HUMAN-TESTING-GATE-VERIFICATION.md).
- For real services: use `tmux` for long-running daemons, Docker for cross-platform parity, standalone library exercises for pure-function gates.
- **Evidence**: Report pass/fail per gate + list production wiring gaps separately. Don't claim "untestable" — claim "gate passed via harness; wiring gap: X."

---

## Failure Modes: Anti-Patterns That FAIL the DoD

| Anti-Pattern | Why It Fails | GitButler-Specific Risk |
|-------------|--------------|-------------------------|
| **Stubbed core logic** (mock DB, fake HTTP, synthetic success) | Violates operator doctrine's cardinal rule. Tests pass when code is broken. | A governance gate that accepts a mocked `but-authz` check would let unverified agents commit to protected branches. The gate **must** hit the real `but-authz` crate. |
| **Weakened assertions** (`toBe(50000)`, `expect.any()`, `.skip()`) | Papers over cracks instead of fixing them. | A graph test that loosens a commit-count assertion from `toBe(3)` to `toBe(50000)` would miss rebase drift. Use `insta` redactions instead (CLAUDE.md:184). |
| **Type suppressions** (`@ts-ignore`, `eslint-disable`, `.expect.any()`) | Hides real type errors or lint failures that indicate broken contracts. | Adding `@ts-ignore` to a `but-sdk` import would skip typechecking the regenerated SDK, breaking the Tauri→`but-api` contract. |
| **"Happy-path only" tests** | Error states, empty states, and denial states are untested. A "complete" feature that only asserts the happy path is **incomplete**. | A governance UI that only renders the "admin" state but never tests the non-admin denial (UC-MGMT-06 AC-5) would let unprivileged users bypass the gate. |
| **Claiming done without watching it run** | Tests can be green while the feature is broken in production. | An agent that claims "gate works" because `cargo test` passes, without running `pnpm dev:desktop` and clicking through the governance UI, hasn't verified the Tauri IPC path. |
| **Hand-edited SDK generated files** | Breaks the contract between Rust API and TypeScript callers. | Editing `packages/but-sdk/src/generated/` by hand would desync the TS types from the Rust `but-api` surface, causing runtime IPC failures. |
| **Leaving uncommitted work** | Uncommitted changes get bulldozed by parallel agents, snapshot cleanups, and working-tree races (operator doctrine: real incident). | An agent that leaves governance UI changes uncommitted while another agent touches `apps/desktop` risks losing work. |
| **Violating the workspace model** | Using legacy abstractions for graph logic introduces subtle bugs. | A new merge operation that uses `but-rebase::Rebase` instead of `graph_rebase::Editor` could corrupt commit ordering (WORKSPACE_MODEL.md:28-33). |
| **Narrowest test not run first** | Workspace-wide runs mask failures in the specific crate. | Running `cargo test` instead of `cargo test -p but-authz` would miss a crate-specific failure that gets hidden by other crates' test noise (CLAUDE.md:112-115). |
| **Refusing a human testing gate because wiring is missing** | The gate's functional claim is unverified. | Claiming "governance loop works" is dishonest without testing the real `but pr`/`but review` verbs. Build a harness if the wiring binary doesn't exist (operator doctrine: HUMAN-TESTING-GATE-VERIFICATION.md). |

---

## The Verification Bar: Minimum Evidence Required

An agent must attach **concrete evidence** to every change claimed as "verified":

1. **Test output** showing the narrowest test passing, then any broader suite.
2. **Snapshot files** (`.snap` for `insta`, `snapbox` for CLI) committed alongside the change.
3. **Manual verification log**: "Ran `pnpm dev:desktop`, clicked through governance UI, observed denial for non-admin principal."
4. **Commit SHA**: `git log` shows the change landed on `main` with a clear message.
5. **No suppressions**: No `@ts-ignore`, `.skip()`, `eslint-disable`, or weakened assertions in the touched files.
6. **Real services hit**: Agent report names the actual services exercised (e.g., "hit real `but-authz` authority check, not a mock").

**If any of these is missing, the change is NOT verified production code.**

---

## How This Relates to GitButler's Governance PRD

The governance PRD (`.spec/prds/governance/`) already encodes this verification philosophy:

- **Real services, no mocks**: E2E testing criteria (11-e2e-testing-criteria.md:8) state: *"Verification bar: real `but-authz` + real `but-api` + real git, no mocks."*
- **Integration/E2E as primary**: 67 integration-test and 2 e2e-automated criteria out of 129 total (11-e2e-testing-criteria.md:10). Unit tests are supplementary.
- **Build-gate honesty invariants**: T-AUTHZ-016b (N-API audit), T-GATES-006 (no hardcoded protected-branch list), T-LOOP-005 (no role name in enforcement) are **build-gate** criteria that block the slice regardless of other lanes (11-e2e-testing-criteria.md:267).
- **Component-test prerequisite**: T-MGMT-000 requires desktop CT config before all 38 component-test criteria can run (11-e2e-testing-criteria.md:12).

This Definition of Done operationalizes that philosophy into a checklist any agent can follow for **any** change, not just governance work.

---

## Maintenance Notes

- This document is **living doctrine**. Update it when GitButler's testing patterns evolve (e.g., new test frameworks, new scoped `AGENTS.md` files).
- When adding a new crate or surface, update the "Narrowest Test First" section with the exact command.
- When a new failure mode is discovered, add it to the anti-patterns table with the GitButler-specific risk.

**Source Truth**: This document reflects GitButler CLAUDE.md, WORKSPACE_MODEL.md, and the governance PRD's E2E testing criteria. Any conflict between this document and those sources should be resolved in favor of the source files.
