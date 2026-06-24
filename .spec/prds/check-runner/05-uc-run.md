---
stability: FEATURE_SPEC
last_validated: 2026-06-20
prd_version: 1.0.0
functional_group: RUN
---

# Use Cases: Check Runner (RUN)

The runner is the **producer** — and the single property that makes the system trustworthy lives here: **the entity being gated never produces its own verdict.** The runner obtains a **clean checkout of the exact current head OID** — mechanism-agnostically, across virtual branches / worktrees / plain git, without disturbing the agent's live tree — **actually executes** the defined command/script (no stubbed success, ever), captures the exit code → typed conclusion, binds it to that head OID, and records it via deterministic engine code in a **plain `check_results` table**. It runs in the **trusted daemon/CLI the human runs — NOT the agent's process** — because whoever runs the check controls the result. It also runs the required `on-merge-attempt` checks in the **pre-merge step** so the read-only gate has current-head results to consume. It produces the inputs; it does **not** adjudicate the gate.

> **No stubbed success — the cardinal line.** A runner that reports `success` without running the real check (a stub, a fake conclusion, a no-op, or a skip-that-records-green) is the exact fake-success sin this design exists to prevent, and it would silently void every gate. The runner MUST run the real command and derive the conclusion from the real exit code; a check it cannot run fails **closed** (`failure`/`timed_out`), never green.

> **Why a plain store is correct (not a compromise).** The result lives in a plain `but-db` `check_results` table — no signing, no agent-unwritable hardening. Under the own-fleet threat model this is consistent with governance's accepted-forgeable review store (its R6): a check is **reproducible**, so a forged green is **detectable by a later re-run — post-merge, not at the gate**, and (for a fast, deterministic check) running it is cheaper than forging a row. It is _safer in detectability_ but **not** "strictly safer" — a check row carries no principal identity (see [01-scope.md](./01-scope.md) Known Limitations). The trust rests on **runner ≠ agent**, **the gate reads a stored fact not agent prose**, **no caller-supplied-conclusion API**, and **head-OID binding** — not on cryptography.

| ID        | Title                                                                       | Description                                                                                                                                                                                                                                                                                                                                               |
| --------- | --------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| UC-RUN-01 | Butler runner executes the real check (runner ≠ agent)                      | The butler-controlled runner actually runs a defined check's command/script in the trusted daemon/CLI the human runs — not the gated agent's process — and derives the conclusion from the real exit code, so the entity being gated cannot produce its own verdict and there is no agent-callable path that records a `success`.                         |
| UC-RUN-02 | Mechanism-agnostic clean checkout at the current head OID                   | The runner obtains a clean checkout of the exact current head OID and runs the check there, working identically across GitButler virtual branches, worktrees, and plain git, without assuming the live worktree == the head OID and without disturbing or contending on the agent's shared worktree.                                                      |
| UC-RUN-03 | Result bound to the head OID, recorded in a plain store                     | The runner records a typed conclusion (v1: `success`/`failure`/`timed_out`) bound to the head OID it ran against, via deterministic engine code, in a plain `check_results` table keyed `(name, head_oid)` — so the result attests the precise code and a graph mutation that moves the head leaves the new head unsatisfied (SHA-reset by construction). |
| UC-RUN-04 | Trigger-driven execution + pre-merge run (`on-commit` / `on-merge-attempt`) | The runner runs an `on-commit` check when a commit is created, and (for `on-merge-attempt`) in the trusted pre-merge step immediately before a governed merge, producing current-head results the gate consumes — the gate never runs a check, and a stale-at-head check is surfaced via STEER, not auto-re-run.                                          |
| UC-RUN-05 | Timeout, concurrency, observability                                         | The runner terminates a check exceeding `timeout_secs` (concludes `timed_out`), runs/records concurrently without clobbering (`(name, head_oid)`-keyed), and emits a dual-audience run event (human CLI + machine `--json`).                                                                                                                              |

---

## UC-RUN-01: Butler runner executes the real check (runner ≠ agent)

The deepest lesson from CI trust boundaries is that _whoever runs the check controls the result_ — a runner the gated party controls makes the gate theater. This use case enforces the inverse: the check is produced by a **butler-controlled runner** that runs in the **trusted daemon/CLI the human runs**, structurally **not** the gated agent's process. The runner invokes the defined `run-spec` for real, waits for it to complete, and derives the conclusion from the real exit code (UC-DEFN-03). The gated agent has **no entry point that produces a check result**: there is no `but` action an agent can call that records a `success` for a check it claims passed. A check the runner cannot run (missing interpreter, run-spec error, timeout) concludes **closed** — `failure`/`timed_out` — never `success`. (Threat-model caveat from scope: the agent shares the OS user with the runner and could in principle write the plain `check_results` table directly — a forged row the gate accepts **at merge time** (its merge-time guarantee is row-existence + head-OID match, not "no forgery"), **detected only by a later re-run post-merge** — no trigger auto-re-runs after a clean merge (see [01-scope.md](./01-scope.md) Known Limitations). The value rests on runner ≠ agent + head-OID binding, which hold regardless.)

### Acceptance Criteria

☐ The butler runner actually runs a defined check's `run-spec` (command or `./path` script) for real, so the conclusion reflects a real run rather than a claim
☐ The runner runs in the trusted daemon/CLI the human runs — structurally **not** the gated agent's process — so the entity being gated does not produce its own verdict (runner ≠ agent)
☐ System exposes **no** `but` action by which the gated agent can record or assert a passing check result for code it claims passed, so an agent cannot self-produce a `success`
☐ The runner derives the conclusion from the real process exit code (UC-DEFN-03), so a `success` requires the real check to have exited `0`
☐ The runner concludes a check it cannot run (missing interpreter, run-spec error, timeout) **closed** — `failure` / `timed_out`, never `success` — so an unrunnable check is never silently green
☐ System has a passing integration test against the real runner with a real exit-`0` and a real exit-`1` command asserting the runner runs each and derives `success` / `failure` from the real exit code, and asserting there is no agent-callable path that produces a `success` record

---

## UC-RUN-02: Mechanism-agnostic clean checkout at the current head OID

A check result is only meaningful if it attests **the exact code being landed**, run in a clean state — and GitButler organizes that code as **virtual branches over one shared worktree**, with optional worktrees and plain-git fallbacks. This use case makes the runner obtain a **clean checkout of the exact current head OID** and run the check there, **identically regardless of branching mechanism**. It binds to the head **commit OID**, never assuming the live worktree _is_ a clean checkout of one branch's head (it usually is not — it is dirty and is a projection of several virtual branches). The checkout is **isolated** — a throwaway/warm detached checkout from the object database, or an object-DB-only inspection for purely-git checks — and **never mutates or contends on the agent's shared worktree** (the "house guest leaves a mess" hazard, and the lock-contention hazard, are both excluded). The conclusion is bound to the head OID the runner actually checked out and ran — never to a different OID. (The materialization mechanism, latency budget, and lock discipline are specified in `08-technical-requirements/07-mechanism-agnostic-checkout.md` — the #1 technical risk.)

### Acceptance Criteria

☐ The runner obtains a clean checkout of the exact current head OID before running the check, so the result attests the precise code being validated, not the agent's live/dirty tree
☐ The runner resolves and binds to the head **commit OID** identically whether the change is a GitButler virtual branch, a worktree, or plain git, so the result is mechanism-agnostic (it never assumes the live worktree == the head OID)
☐ The runner runs the check in an isolated clean state (no carryover from a prior run, no use of the agent's working tree), so a stale artifact or the agent's uncommitted edits cannot fake or poison a pass
☐ A check run **does not mutate or contend on** the agent's shared worktree or its index/locks, so running a check never disrupts the agent's in-progress work
☐ The runner binds the produced conclusion to the head OID it actually checked out and ran — never to a different OID — yielding the `(name, head_oid, conclusion)` record (UC-RUN-03)
☐ System has a passing integration test against the real runner + real git that checks out a specific head OID, runs a check that inspects the worktree contents, and asserts the captured conclusion is derived from that exact OID's code — exercised across at least two branching mechanisms (a GitButler virtual-branch head and a plain detached head) to prove mechanism-agnosticism — while asserting the agent's shared worktree is untouched

---

## UC-RUN-03: Result bound to the head OID, recorded in a plain store

A result the gate can trust must attest exactly one check against exactly one commit, and must be recorded by the engine, not authored by the agent. This use case records a check result as a typed `(name, head_oid, conclusion)` row in a **plain `but-db` `check_results` table**, written by **deterministic engine code** (recording is never an agent decision — the deterministic-vs-probabilistic doctrine). Its identity is `(name, head_oid)`. The **SHA-reset** property is by construction: a result is valid for its head OID only; the instant a graph/ref mutation (rebase, amend, reorder via `but_rebase::graph_rebase::Editor`) moves the head OID, the new head has **no satisfying result** until the check is re-run — a green never carries across OIDs, because the gate matches `(name, head_oid == current_head_oid)`, never `name` alone. The store carries the captured conclusion + a log reference as `metadata` (agent-readable evidence, **outside the gate's trust input**), and a nullable `signature` column reserved as a forward-compat seam (unused in v1, never a v1 trust input).

### Acceptance Criteria

☐ System records a check result as a typed `(name, head_oid, conclusion)` row in a plain `but-db` `check_results` table, so identity is `(name, head_oid)` and the result attests one check against one commit
☐ System records a produced result via deterministic engine code (not an agent tool call or LLM decision), so recording always happens after a run and is never an agent's choice
☐ System exposes no `but` action, API, or path by which the gated agent supplies a `conclusion` value that is recorded as a result the gate counts, so a result is only ever a real run's output (the negative-space lock)
☐ System treats a result as valid for the exact head OID it was produced against and not for any other OID, so a green never carries across OIDs (SHA-reset by construction)
☐ System leaves a newly-produced head OID (after a rebase/amend/reorder) with no satisfying required-check result until the checks are re-produced and pass on that new OID, so an agent cannot mutate the change underneath an existing green
☐ System treats stored `metadata` (captured stdout/stderr + a log reference) as agent-readable evidence **outside the gate's trust input** — the gate keys only on `(name, head_oid, conclusion)` — so captured output is never consulted as a verdict
☐ System has a passing integration test against the real `check_results` store + real git that records a `success` at head H1, mutates the change so the head becomes H2 (rebase/amend), and asserts the gate sees no satisfying result at H2 (the H1 green does not carry over) until the check is re-produced and passes at H2

---

## UC-RUN-04: Trigger-driven execution + pre-merge run (`on-commit` / `on-merge-attempt`)

A check must run at the point its definition declares, and the read-only gate must always have a result for the **current** head to consume. This use case wires the runner to the two v1 triggers and the pre-merge step. An **`on-commit`** check becomes eligible when a commit is created (producing a result bound to the new head OID). For **`on-merge-attempt`**, the **trusted CLI/daemon runs the required checks in the pre-merge step immediately before** the governed merge — so the read-only gate then consumes current-head results. The gate itself **never runs a check**: if a required check is **stale at the merge head** and the pre-merge run did not refresh it, the gate **BLOCKS** with `check_stale_at_head` + STEER ("run the check"), and the runner re-produces the result via that explicit `but check run` redirect or the next pre-merge step — never auto-re-run from inside the gate. The runner reads which checks to run from the **target-ref** config (UC-DEFN-04), so the agent cannot inject an extra trivially-passing check at the feature head to satisfy the gate.

### Acceptance Criteria

☐ The runner runs an `on-commit` check when a commit is created, producing a result bound to the new commit's head OID
☐ The trusted CLI/daemon runs the required `on-merge-attempt` checks in the **pre-merge step immediately before** a governed merge (the runner, not the gate), so the read-only gate has current-head results to consume
☐ System surfaces a required check that is **stale at the merge head** by **blocking** at the gate with `check_stale_at_head` + a STEER "run the check" redirect, and re-produces it via the runner on that explicit redirect or the next pre-merge run — the gate never auto-re-runs a check itself
☐ The runner reads which checks to run from the **target-ref** config (UC-DEFN-04), so an agent cannot add a trivially-passing check at the feature head to satisfy the gate
☐ System produces every check result via the runner and records it via the engine (UC-RUN-03) — the gated agent produces no result at any trigger point or in the pre-merge step
☐ System has a passing integration test against the real runner + real git that (a) creates a commit and asserts an `on-commit` check runs and records a result bound to the new head OID, and (b) runs the pre-merge step before a merge and asserts the `on-merge-attempt` required checks are produced at the current head by the runner (not by the gate)

---

## UC-RUN-05: Timeout, concurrency, observability

A real runner must bound runtime, run many checks without races, and be observable. **Timeout**: a check exceeding its declared `timeout_secs` is terminated and concluded **`timed_out`** (a blocking conclusion at the gate), never left to hang or silently recorded green. **Concurrency**: multiple checks for a head — and across heads — run and record without clobbering, because results key on `(name, head_oid)`. **Observability**: the runner emits an observable run event, with **dual-audience** structured output consumable by both an automated orchestrator (`--json`) and a human at the CLI (`but check run` / `but check results`). (Secret injection for checks that need credentials is deferred — see [01-scope.md](./01-scope.md) Out of Scope.)

### Acceptance Criteria

☐ System terminates a check that exceeds its declared `timeout_secs` and concludes it `timed_out` (a blocking conclusion), so a runaway or hanging check never blocks indefinitely and is never silently recorded as `success`
☐ System runs and records multiple checks concurrently (for a head and across heads) without clobbering, because results key on `(name, head_oid)`, so a concurrent producer never overwrites another check's record
☐ A User can read a check run's outcome — including a failed check's captured output / log reference — via structured output that is both machine-parseable (`--json`, for an orchestrator) and human-readable at the CLI (`but check run` / `but check results`), so a denied agent can retrieve why a required check failed and the same fact serves both audiences
☐ System emits an observable run event when the runner runs a check, so a run is auditable from logs/telemetry rather than invisible
☐ System has a passing integration test against the real runner + real git that asserts (a) a check exceeding `timeout_secs` concludes `timed_out`, (b) two checks for the same head record concurrently without clobbering (both `(name, head_oid)` records persist), and (c) a run event is emitted and the `--json` output is parseable

### UI/UX Wireframe — per-head results panel + branches "Checks" state (v1.1, un-deferred)

> **Surface:** CLI `but check results` is the **v1.0** surface. **v1.1** adds a desktop **`CheckResultsPanel`** plus a **"Checks" segment** on the existing `[projectId]/branches` explorer — a **state of an existing route, not a new route** (TR §10 §2). The forge fetch already happens and the raw `CiCheck[]` is **already in the Redux cache**; delivering the panel is a **modest call-site read** (a new `useQueryState` on `listCiChecks` **without** the `parseChecks` transform — every current consumer goes through `ChecksMonitor` with that transform, so nothing reads the per-check detail today), plus **type-system surgery** for the new segment (TR §10 §2). No new network round-trip; not a state-layer refactor.

```
┌─ CheckResultsPanel · main @ abc123f ────────────[Run all] [Refresh]─┐
│  ✓ cargo-test   passed   2m14s   on-commit        producer: butler  │
│  ✗ pnpm-check   failed   0m33s   on-commit                        ▼ │
│     ┌ Codeblock (expand) ─ error TS2345: … (captured stderr) ────┐  │
│  ○ lint         missing   —      on-merge-attempt   [▶ Run now]   │
│  ↻ sig-check    stale (last at 999eee)              [▶ Run now]   │
└──────────────────────────────────────────────────────────────────────┘
branches explorer:  [ All | PRs | Local | ●Checks ]   ← new segment added to real
                    segments (All/PRs/Local from BRANCH_FILTER_OPTIONS); per-branch
                    rows show the existing CIChecksBadge rollup, drill-in → panel
```

**Component map** (full table → [`08-technical-requirements/10-frontend-ui.md`](./08-technical-requirements/10-frontend-ui.md) §3–4): **reuse** `cardGroup/CardGroupRoot|Item`, `Codeblock`, `Button` (Run all + Refresh + Run now — "Run all" triggers `but check run` for each missing/stale check at the head), `TimeAgo`, `EmptyStatePlaceholder`, `SegmentControl` (`packages/ui`), `CIChecksBadge` (the existing rollup); **net-new** `CheckConclusionBadge` + `CheckResultRow` + `CheckResultsPanel` in `apps/desktop/src/components/checks/`. `CheckConclusionBadge` uses a `state` prop (not `conclusion`): `success`/`failure`/`timed_out` are stored conclusions; `missing` (from `isMissing`) and `stale` (from `isStale`) are gate-derived display states; `running` is for an in-flight local run. The `○ lint missing` row renders `state="missing"` via `isMissing=true` — not a stored conclusion. Row trigger field resolved via a join to the check definition (not from `check_results`). **Lite (React):** no checks UI exists today → a **port**, not a share (deferred).
