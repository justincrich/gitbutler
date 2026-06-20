---
stability: PRODUCT_CONTEXT
last_validated: 2026-06-20
prd_version: 1.0.0
---
# Check Runner — Local Deterministic Checks that Gate a Change — Overview

> **Naming (read first).** This system is surfaced on the CLI as **`but check`** (crate `but-checks`, table `check_results`). It is GitButler's analog of **GitHub Checks / required status checks** — a result that a merge gate enforces — **not** GitHub *Actions* (the workflow/runner platform). It is distinct from two pre-existing GitButler features it is easy to confuse it with: the **`butler_actions`** table (an audit log of what agents *did*) and **`ci_checks`** (a read-only cache of *forge* CI results). `but check` is the new thing: a **local producer** of check results plus the **gate clause** that consumes them. This PRD **supplants** the earlier `.spec/prds/actions/` PRD, which over-scoped the same capability as a cryptographic "agent-non-forgeable" system (see "What changed from the `actions` PRD").

## Product Description
Check Runner adds one missing capability to GitButler: **a butler-controlled, local, deterministic runner that executes a configured check** (`cargo test`, `pnpm check`, a repo `./script`, …) **and records a pass/fail result bound to the exact current head OID**, which a **merge-gate clause consumes to block a change unless every required check passes at that head.** Which checks exist and which are required is **config-as-code**, committed and ref-pinned. The result works **identically across GitButler's virtual branches, worktrees, and plain git** — it binds to the head *commit*, never to a branching mechanism.

The mental model is exact: **a required check is a *second deterministic review* whose verdict is produced by a local runner instead of a human.** GitButler already gates a merge on a human review (the governance review clause); Check Runner adds a clause of the same shape whose "approval" is a passing automated run.

## The thesis: done is proven by re-running, not claimed
A coding agent reports "tests pass" as a textual claim it has no stake in. Check Runner makes that claim irrelevant: the gate never reads the agent's prose — it reads a **stored result the runner produced**, bound to the head OID. The agent's only route to a green gate is to make the trusted runner actually run the check and have it genuinely exit `0` at the current head.

The load-bearing property is **reproducibility**, not cryptography. A check is re-runnable, so a forged "pass" is cheap to *catch* (re-run, see red) and — **for a fast, deterministic check** — the honest path is cheap to *take* (re-run, see green). This is the real difference from a human review — you cannot re-derive a person's judgment, so governance must accept its review store as forgeable; a check you can simply run again. Check Runner therefore does not try to make cheating *impossible*; it makes the **honest path the path of least resistance** for an agent that optimizes for least resistance.

> **What "caught by re-running" does and does not mean (read precisely).** Re-running detects a forged green **after** a merge has landed, not at the merge: the gate's *merge-time* guarantee is only that a row **exists** and is **head-OID-matched** — pre-merge it cannot distinguish a forged row from a genuine one, and post-merge detection needs a re-run policy that is **out of scope for v1** (no trigger re-runs automatically after a clean merge). And "the honest path is cheapest" holds for **fast, deterministic** checks; for a slow (`cargo test`) or flaky check the honest run can cost more than forging a row, so the deterrent then rests on the **structural locks** (the small gate-read→merge window, the bootstrap-invariant, the no-caller-conclusion path), not on cost. These bounds are stated in [01-scope.md](./01-scope.md) Known Limitations.

### What the gate proves (and what it does not)
The guarantee is **precise and bounded**: the gate proves that **the committed, target-ref-pinned check actually ran under the trusted runner and exited `0` against the exact head OID being landed** — it does **not** prove "the code is correct," and a weak or vacuous check yields a weak guarantee. The strength is exactly the committed check set's; the engine guarantees only that the check *really ran, on this commit*.

## Problem Statement — the pieces exist; the wire between them does not
GitButler already has nearly everything this needs — as the **review** clause:

1. **A merge gate** (`enforce_merge_gate`) that blocks on process (permission + a review at head) — but has **no notion of "a check passed."**
2. **A result store shape** — `local_review_verdicts` (human approvals, keyed by `target` + `head_oid`) and `ci_checks` (forge CI results, keyed by `reference`, with `name` / `head_sha` / `status_conclusion` columns) — both already carry a head column for the staleness key, but neither is a **locally-produced** check result.
3. **Current-head freshness** — the review clause already discards approvals bound to an ancestor OID (`approval_stale_at_head`) — but nothing applies it to checks.
4. **Command execution** — git hooks (`gitbutler-repo/hooks.rs`) already run commands — but **in the agent's own `but commit`, bypassable with `--no-hooks`** (runner *is* the agent), recording nothing the gate trusts.

The single missing capability is a **local producer**: something that runs a check **in the trusted CLI/daemon, not the agent's process**, and records pass/fail **at the head OID** for the gate to consume. Everything else is a near-mechanical clone of the review clause.

## Solution Summary
Three functional groups deliver the producer, the contract, and the consumer:

- **Check Definition (DEFN)** — config-as-code named checks in committed, ref-pinned `.gitbutler/checks/*.toml`: a `name` + a `trigger` (`on-commit` / `on-merge-attempt`) + a local `run-spec` (a command or a repo `./path` script) + a `required` flag + an exit-code success mapping (exit `0` → `success`). The required-set is a `[[required_check]]` policy in governance's `gates.toml`, read at the **target ref** (mirroring the `[[gate]]` review requirement), so a change cannot weaken the checks that judge it — and the required-set is **self-protecting** (a change to it must itself clear the currently-required checks: the bootstrap-invariant).

- **Check Runner (RUN)** — the butler-controlled local runner. It obtains a **clean checkout of the exact current head OID** — **mechanism-agnostically**, without disturbing the agent's live virtual-branch/worktree state — runs the real command, derives the conclusion from the real exit code, and records the result **bound to that head OID** in a plain `but-db` table. It runs in the trusted CLI/daemon, **not the agent's process**, and exposes **no path by which the agent supplies a conclusion**. It also runs the required `on-merge-attempt` checks in the pre-merge step so the read-only gate has current-head results to consume.

- **Required-Checks Gate (GATE)** — a new clause on the governance merge gate: block unless **every required check has a `success` bound to the current head OID**. It **composes** with the existing `merge`-authority and review clauses (one decision: process *and* quality), is a **read-only consumer** (it never runs a check), **fails closed** on missing / failed / stale / unreadable-config, and emits a **STEER denial** (reusing governance's exact denial contract) that names the unmet check and the corrective next action so an agent can self-correct.

## Mechanism-agnostic by construction
Because the result binds to the **head commit OID** and the gate matches `(name, head_oid == current_head_oid)`, Check Runner is indifferent to *how* the change is organized — a GitButler virtual branch, an opt-in worktree, or plain git all resolve to a head OID. The one place this is non-trivial is getting a *clean checkout at that OID* in GitButler's one-worktree, virtual-branches-over-one-tree model without touching the agent's live tree — which is the central engineering problem of this PRD (see `08-technical-requirements/07-mechanism-agnostic-checkout.md`).

## How it composes with governance + STEER
Check Runner is **not standalone** — it is the quality leg of the governance gate:

| Stage | System | Role |
|---|---|---|
| **Produce** | **Check Runner** (this PRD) | the local runner runs the check → records `(name, head_oid, conclusion)` in a plain store |
| **Consume / block** | Governance merge gate (extended) | a new **required-checks clause** alongside `merge`-authority + review — block unless every required check is `success` @ current head |
| **Redirect** | STEER | the denial's `class` / `authorized_actions` / `remediation_hint` name the failed/missing check and the next action (`but check run …`) so the agent self-corrects |

> **Two sequencing dependencies (named, not assumed).** (1) The STEER steering fields (`class` / `held_permissions` / `authorized_actions` / `do_not`) and `to_envelope()` are added to the `MergeGateError` carrier by governance **STEER-001** (sprint-07), which is **not yet merged** — until it lands the carrier exposes only `code` / `message` / `remediation_hint` / `unmet`, so the GATE group's STEER denial **sequences after STEER-001**. (2) The shipped `enforce_merge_gate` is reachable only via a forge `review_id`; gating a **local** `but merge` (virtual-branch / worktree / plain-git, no PR) requires a **mechanism-agnostic local-merge entry point** that resolves source/target/head without a forge review — a v1 requirement, not an assumption (see `08-technical-requirements/`).

Governance enforces **process** (a principal *may* act, and an approval exists); Check Runner adds **quality** (the committed checks provably ran and passed). Both read their config at the target ref; both fail closed; both speak the same denial contract.

## What changed from the `actions` PRD (and why this is smaller)
The earlier `actions` PRD framed this as making a passing record **"agent-non-forgeable"** — a cryptographic property — and carried a signed, agent-unwritable "ledger," HMAC-now/Ed25519-later, and an OS-sandboxed executor as owed debt. Under the **own-fleet, personal-tenant** threat model that is the wrong bar: the agent shares the OS user with the runner, so symmetric signing cannot actually close forgery, and a check is **reproducible** so it does not need to. This PRD therefore:

- **Drops** the non-forgery framing, signing-as-security, agent-unwritable hardening, and the OS-sandbox/Ed25519 debt. The result store is a **plain `but-db` table** — no more protected than the review store governance already accepts as forgeable (its R6); reproducibility makes a forged green **detectable post-merge**, though a check row carries no principal identity (no attribution deterrent), so it is *safer in detectability*, **not** strictly safer. (A nullable `signature` column is retained only as a forward-compat seam for the day a producer goes off-host — explicitly **not** a v1 security claim.)
- **Keeps** the cheap structural locks that flip the cost asymmetry: the gate reads a stored fact (not prose); runner ≠ agent; no caller-supplied-conclusion API; SHA-binding to the current head.
- **Promotes** the genuinely hard part — the mechanism-agnostic clean checkout at the head OID — from one under-specified risk row to a first-class section and the #1 technical risk.
- **Defers** (as out-of-threat-model, not debt) untrusted/fork execution, remote/labelled runners, the `uses` registry / Docker / JS / composite action types, the `${{ }}` expression engine, matrix/DAG/reusable workflows, and merge-queue.

The result is a focused "second deterministic-review clause + local runner," not a CI platform.
