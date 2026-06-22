---
stability: PRODUCT_CONTEXT
last_validated: 2026-06-20
prd_version: 1.0.0
---

# Team Contributions

This PRD was produced by re-scoping the earlier `.spec/prds/actions/` PRD down to the capability the user actually wanted, then grounding it in real GitButler code.

## Phase 0 — Gap analysis (supersedes `actions`)

A line-by-line read of the `actions` PRD against the GitButler codebase established that GitButler already ships the merge gate (`enforce_merge_gate`), a ref-pinned policy config (`gates.toml`), a verdict-store shape (`local_review_verdicts`, `ci_checks`), and current-head freshness (`approval_stale_at_head`) — all via the governance **review** clause. The single missing capability is a **local producer** (run a check + record pass/fail at the head OID). The `actions` PRD over-scoped that producer as a cryptographic "agent-non-forgeable" system (signed agent-unwritable ledger, HMAC→Ed25519, OS-sandboxed executor). Under the own-fleet / personal-tenant threat model that bar is wrong: the agent shares the OS user with the runner (so symmetric signing cannot close forgery), and a check is **reproducible** (so it does not need to). The reframe: **a required check is a second deterministic review whose verdict a local runner produces; security rests on reproducibility + cheap structural locks, not crypto.**

## Phase 1 — Product framing (orchestrator as product lead)

The overview, scope, roles, functional groups, and use cases were authored from the gap analysis: three groups (DEFN / RUN / GATE), the "honest-path-is-cheapest" bar, the consistency argument with governance's accepted-forgeable review store (its R6), the plain `check_results` table (a `signature` column retained only as a forward-compat seam), and the promotion of the **mechanism-agnostic head-OID checkout** from one under-specified risk row to a first-class concern.

## Phase 2 — Technical grounding (rust-planner)

The `08-technical-requirements/` folder was authored by a `rust-planner`, grounded in real crates: the `enforce_merge_gate` extension point and its protected-branch early-return fail-open; the `check_results` table modeled on `local_review_verdicts`/`ci_checks`; the `but_rebase::graph_rebase::Editor::commit_mappings` SHA-reset basis; the `gix`/worktree options for the isolated head-OID checkout; and the reuse of `ci_checks` as producer-zero and `gitbutler-repo/hooks.rs` as the why-hooks-can't-be-the-producer prior art.

## Phase 3 — STEER reuse (from governance sprint-07)

The denial contract reuses governance's STEER design (`sprint-07-steer-capability-aware-denials` — **STEER-001, not yet merged**, named as a sequencing dependency): the four steering fields (`class: DenialClass{ActorCorrectable|OperatorRequired}`, `held_permissions`, `authorized_actions: Vec<{command, effect}>`, `do_not`) on the `MergeGateError` carrier, unified via `to_envelope()`, with a new `gate.check_required` code and check-specific miss-reasons (`check_missing` / `check_failed` / `check_stale_at_head` / `config.invalid`).

## Phase 4 — Adversarial review

A fresh red-hat pass checked the reframe for consistency (no residual crypto/non-forgery framing), the fail-closed and protected-flag-independence invariants, the mechanism-agnostic checkout treatment, AC testability against real services, and that the PRD is materially tighter than `actions` (3 groups / 15 UCs vs 4 groups / ~19 UCs, with the LEDG hardening group dissolved into a plain store).

## Phase 5 — Convergence review loop (3 cycles, source-grounded)

The v1.0 PRD + the v1.1 UI scope were run through a goal-driven convergence loop: fresh red-hat panels (product / technical-vs-live-`crates/` / UI), remediate, re-review until **0 CRITICAL/MEDIUM** across all surfaces.

- **Cycle 1** — Technical CONVERGED (all prior CRITICAL fixes re-verified against live source; the new UI doc's citations all real). Product surfaced 2 MEDIUM count-integrity defects (e2e subtotals). The never-before-reviewed v1.1 UI surfaced **4 CRITICAL + 4 HIGH + 3 MEDIUM** source-grounding errors — wrong `BranchExplorer` segment labels, a `parseChecks` data-access over-claim ("cheap freebie"), an `inProgress` conclusion-vocabulary category error, a wrong `ReviewBadge` path, missing `-bg`/`-fg` token suffixes, and a governance-settings-stub-as-precedent.
- **Cycle 2** — All cycle-1 findings closed; but the UI remediation **over-corrected** the data-access claim ("raw `CiCheck[]` never in state") → 2 new MEDIUM.
- **Cycle 3** — The corrected mechanism claim verified accurate against `checksMonitor.svelte.ts` / `customHooks.svelte.ts` (raw `CiCheck[]` IS cached; `transform` is a read-time call-site selector, not RTKQ `transformResponse`; the panel is a modest call-site read) → **CONVERGED**.

The loop caught real, source-grounded UI defects that structural self-validation had passed as clean — the reason the convergence (re-review-until-met) leg, not just a single review pass, is load-bearing.
