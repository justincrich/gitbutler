---
title: Check Runner — Local Deterministic Checks that Gate a Change
version: 1.2.0
scope_posture: full
pr_sequencing: false
supersedes: actions
---

# Check Runner — PRD

A butler-controlled, **local, deterministic runner** that executes a configured check (`cargo test`, `pnpm check`, a repo `./script`) and records a **pass/fail bound to the current head OID**, which a **merge-gate clause** consumes to **block a change unless every required check passes at that head** — working identically across GitButler virtual branches, worktrees, and plain git, with a **STEER denial** that redirects an agent on a miss. GitButler's analog of **GitHub Checks / required status checks** (CLI noun `but check`).

> **Supersedes the `actions` PRD.** This PRD replaces `.spec/prds/actions/`, which over-scoped the same capability as a cryptographic "agent-non-forgeable" system. The reframe: **a required check is a second deterministic review whose verdict a local runner produces.** Security rests on **reproducibility** (a forged green is caught by re-running) + **cheap structural locks** (gate reads a stored fact not agent prose; runner ≠ agent; no caller-supplied-conclusion API; SHA-binding) — **not** on signing/sandboxing. The result store is a **plain `but-db` table**, no more protected than the governance review store already accepted as forgeable (its R6); reproducibility makes a forged green detectable post-merge (so it is *safer in detectability*, **not** strictly safer — a check row carries no principal identity). See [00-overview.md](./00-overview.md) "What changed from the `actions` PRD".

## The thesis
A coding agent's "tests pass" is a textual claim it has no stake in. Check Runner makes that claim irrelevant: the gate reads a **stored result the runner produced** at the current head OID, never the agent's prose. The bar is **"make the honest path cheaper than cheating,"** not "make cheating impossible" — which is exactly the right bar for a reproducible, re-runnable check under a personal-tenant / own-fleet model.

## What the gate proves (bounded)
*The committed, target-ref-pinned check actually ran under the trusted runner and exited `0` against the exact current head OID* — **not** "the code is correct." The strength is the committed check set's; the engine guarantees only that the check really ran, on this commit.

## Dependencies (named, not assumed)
**Governance has closed and is frozen in `master`.** What were two pending prerequisites are now one landed dependency and one still-open requirement of *this* PRD, plus a new inherited precondition:

1. **Governance STEER — LANDED.** The four steering fields (`class` / `held_permissions` / `authorized_actions` / `do_not`) and `to_envelope()` now exist on the `MergeGateError` carrier in `crates/but-api`; the STEER denial (UC-GATE-03) reuses fields that are real, not pending. **No upstream sequencing remains — the GATE group no longer sequences after STEER.**
2. **A mechanism-agnostic local-merge entry point — still an open requirement of this PRD.** The shipped `enforce_merge_gate(ctx, review_id)` is still reachable only via a forge `review_id` (its non-test callers are the forge PR-merge path). Governance's GOV-LOCAL work keyed its local merge gate on a review id too, so it does **not** satisfy this. Gating a **local** `but merge` (virtual-branch / worktree / plain-git, no PR) requires generalizing the gate entry to resolve `(source_ref, target_ref, head_oid)` without a `ForgeReview` (head OID via a mechanism-agnostic `gix` ref-peel). This is a **v1 requirement** of this PRD — see `08-technical-requirements/` (risk **R-ENTRY**).
3. **Governance IDENT — an inherited precondition (new since this PRD).** The gate now resolves the acting principal through a runtime PID registry: a process must `but agent register` or be denied `perm.denied`, and the principal roster was renamed `permissions.toml` → `agents.toml`. This is **process-level, host-OS-rooted, not cryptographic** — it does not change the own-fleet threat model or the forgeability of the plain `check_results` store; it governs **who may invoke the gate**, not who may write the store. See [02-roles.md](./02-roles.md) "Identity at the gate".

Plus the composition base: the shipped governance merge gate (`enforce_merge_gate`) + `gates.toml` / `[[gate]]` machinery + `but-authz` (now with its runtime agent registry).

## PRD Metadata

| Field | Value |
|-------|-------|
| Version | 1.2.0 |
| Scope Posture | Full feature (deliberately scoped tight) |
| PR Sequencing | Disabled |
| Supersedes | `.spec/prds/actions/` (archived to `.spec/prds/actions.superseded/`) |
| Created | 2026-06-20 |
| Last Updated | 2026-06-26 |
| Specialists | orchestrator (product lead, from the `actions` gap analysis) · rust-planner (technical-requirements) · frontend-designer (v1.1 UI review + wireframes) · red-hat review |
| Target | GitButler `crates/*` — a new `but-checks` crate (runner + config loader + plain recorder + pure required-checks evaluator) + a new `but-db` `check_results` table + an EXTENDED `but-api` `enforce_merge_gate` (required-checks clause, reusing the STEER `MergeGateError` carrier) + new `but check {define,list,run,results,required}` CLI verbs; built atop the shipped governance merge gate + `but-authz` |

## Document Index

| File | Section | Stability |
|------|---------|-----------|
| [00-overview.md](./00-overview.md) | Product description; the gap (the pieces exist, the wire doesn't); the reproducibility thesis; what changed from `actions` | PRODUCT_CONTEXT |
| [01-scope.md](./01-scope.md) | In scope (v1) / out of scope (deferred as out-of-threat-model) / Known Limitations (mechanism-agnostic checkout = #1) | FEATURE_SPEC |
| [02-roles.md](./02-roles.md) | Human admin · gated agents · the butler runner (trusted producer) · the engine (recorder + gate) · orchestrator; the registered-principal precondition (IDENT); why no hardened ledger is needed | PRODUCT_CONTEXT |
| [03-functional-groups.md](./03-functional-groups.md) | 3 groups (DEFN · RUN · GATE) + the produce→consume→redirect loop | FEATURE_SPEC |
| [04-uc-defn.md](./04-uc-defn.md) | UC-DEFN-01..05 — config-as-code named checks in `.gitbutler/checks/*.toml`; ref-pinned; bootstrap-invariant | FEATURE_SPEC |
| [05-uc-run.md](./05-uc-run.md) | UC-RUN-01..05 — the local runner; runner ≠ agent; mechanism-agnostic head-OID checkout; plain result store; triggers/timeout/concurrency | FEATURE_SPEC |
| [06-uc-gate.md](./06-uc-gate.md) | UC-GATE-01..05 — required-checks clause; composes with governance; fail-closed + STEER denial; read-only consumer; bootstrap-invariant | FEATURE_SPEC |
| [07-team-contributions.md](./07-team-contributions.md) | Planning provenance (gap analysis → reframe → code grounding → STEER reuse → review) | - |
| [08-technical-requirements/](./08-technical-requirements/README.md) | Engineering contract — folder; producer/consumer split, the `check_results` schema, the merge-gate clause, the **mechanism-agnostic head-OID checkout** (the #1 risk), capability chains, risks | CONSTITUTION |
| [09-e2e-testing-criteria.md](./09-e2e-testing-criteria.md) | Per-UC criteria (real runner + real git + real store, no mocks); the produce→consume→STEER→fix→land loop + SHA-reset-under-mutation demos | TEST_SPEC |

## Quick Stats

| Metric | Value |
|--------|-------|
| Functional Groups | 3 (DEFN · RUN · GATE) |
| Use Cases | 15 |
| Acceptance Criteria | 88 (DEFN 29 · RUN 30 · GATE 29) |
| New CLI Surface | `but check {define, list, run, results, required}` |
| System Components | NEW `but-checks` crate · NEW `but-db` `check_results` table · EXTENDED `enforce_merge_gate` (required-checks clause) · REUSED `but-authz`/`gates.toml`/`but-rebase`/`gix` |
| External Dependencies | 0 new expected (no crypto dep in v1 — reuse `gix`/`but-db`/`toml`/`serde`/`std::process`/`tokio`) |
| Disambiguation | distinct from `butler_actions` (agent-action audit log) and `ci_checks` (read-only forge-CI cache) |

## Version History

| Version | Date | Changes | Trigger |
|---------|------|---------|---------|
| 1.0.0 | 2026-06-20 | Initial PRD — 3 groups (DEFN · RUN · GATE), 15 UCs (88 ACs), TR folder centered on the mechanism-agnostic head-OID checkout, e2e criteria with real-runner/real-git integration tests. Supersedes the `actions` PRD by re-scoping it from a cryptographic non-forgery system down to "a second deterministic-review clause + local runner," dropping signing/ledger-hardening/sandbox and promoting the checkout problem to the #1 risk. | Supersedes `actions` |
| 1.1.0 | 2026-06-20 | Un-defer a focused desktop UI (the GitHub-"Checks"-style result viewing the CLI's stored results enable). Added `08-technical-requirements/10-frontend-ui.md` (component inventory, route-vs-state verdict, scope tiering) + inline UI/UX wireframes & component maps on UC-RUN-05 (per-head panel + branches "Checks" state, **v1.1 in scope**), UC-GATE-03 (merge-gate summary, deferred on governance's merge dialog), UC-DEFN-01 (Checks settings tab, deferred). Grounded in a `frontend-designer` review: GitButler shows only an aggregate `CIChecksBadge` today; per-check data is already fetched but discarded. No new route (state of `/branches`); no new ACs. | UI scoping (frontend-designer) |
| 1.2.0 | 2026-06-26 | Delta-replan to absorb the now-closed governance initiative (frozen in `master`). STEER **landed** — the steering fields + `to_envelope()` on `MergeGateError` are real, not pending (dependency 1 no longer sequences upstream). Acknowledged governance's **IDENT** agent-identity precondition: the gate now resolves principals via a runtime PID registry (`but agent register`), `permissions.toml` → `agents.toml`; presented honestly as a process-level, host-OS-rooted precondition, **not** a security upgrade to the forgeable `check_results` store. **R-ENTRY re-grounded** as still-open (governance's GOV-LOCAL keyed its local gate on a review id; the mechanism-agnostic local-merge entry remains this PRD's v1 requirement). No UCs/ACs added or removed; no in/out scope change. | Absorb closed governance (delta-replan) |

## Next Steps
- `/kb-sprint-plan .spec/prds/check-runner` — build the sprint roadmap. The proven-reference-flow (define → run → record @ head → gate-consume → deny → STEER → fix → pass → land, against real components) should precede the deep build, and the **mechanism-agnostic head-OID checkout** spike should be proven before the runner build commits to a materialization strategy.
- Land as a native GitButler capability composing with governance; the runner/store/gate are extensible toward the deferred layers (forge `ci_checks` as a second producer, remote/labelled runners, untrusted-contributor isolation) — and the retained `signature` column is the seam for provenance-signing *if* the threat model ever leaves the trusted host.
