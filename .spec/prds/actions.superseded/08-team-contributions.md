---
stability: PRODUCT_CONTEXT
last_validated: 2026-06-19
prd_version: 1.0.0
---

# Team Contributions & Planning Provenance

This PRD was produced by the `kb-prd-plan` flow, grounded on a prior assimilation of GitHub Actions. The orchestrator consolidated; the specialists authored.

## Phase 0 — Assimilation (grounding)

Before any PRD content, GitHub Actions was assimilated (it is closed-source as a product; the runner/toolkit are OSS) from the official "understand GitHub Actions" docs + two backend/self-hosted-architecture sources, in two parallel passes:

- **Authoring surface** — workflow/`action.yml` schema, triggers, jobs/steps, `uses`/action types, reusable/composite, contexts/expressions, secrets, artifacts/caching.
- **Execution + merge-gate integration** — the trigger→run→job→step lifecycle, the runner agent + dispatch protocol, and the load-bearing seam: the Checks/Status API → branch-protection "required checks @head", plus the security model.

Synthesized into the **GHA → Butler Replication Capability Map** (holocron `js7f9r8cf771qcf0t97dkx38hs890ywq`; execution slice `js7058pqz024dsw5x3ap85mkx5891857`). The map pinned the two architecture decisions every section is built on: **consume + default-produce** (the gate is a deterministic consumer; butler ships the default executor as producer) and **no broker in v1** (executor in the trusted daemon/CLI, not the agent's process), plus the **non-fakeability four-part claim** and the **SHA-reset invariant**.

## Phase 1 — Product spine + use cases (product-manager)

Authored `00-overview`, `01-scope`, `02-roles`, `03-functional-groups`, the four UC files (DEFN/EXEC/LEDG/GATE — 16 UCs, 91 ACs in strict `☐ {WHO} can {ACTION} {CONTEXT}` form), and `10-e2e-testing-criteria` (93 criteria: 91 per-AC 1:1 + 2 additive full-loop human-gate demos). Made the load-bearing properties concrete, headless-observable ACs/tests: success-bound-to-head-SHA + invalidate-on-mutation; gate-blocks-until-green-on-current-head; agent-cannot-write-a-passing-record; failed-check-non-fakeable.

## Phase 2 — Engineering contract (rust-planner)

Authored the `09-technical-requirements/` folder (9 files), grounded against the real GitButler tree. Key discovery: the governance merge gate, `but-authz`, and the `local_review_verdicts` ledger are **already built**, so Actions is specified as an _extension of real surfaces_ — the required-checks clause inserts into `enforce_merge_gate` (`merge_gate.rs:40`), the ledger is a NEW signed `but-db` `check_results` table (explicitly NOT the disposable unsigned `ci_checks` cache), and the executor/contract types are a NEW `but-actions` crate. Pinned the producer/consumer split, the SHA-reset machinery (keyed by head OID via the rebase editor's old→new map), capability chains, and a governance-honesty-style risk register.

## Phase 3 — Adversarial review (fresh red-hat)

A fresh panel (rust-reviewer + security-auditor) red-hatted the drafted PRD against the locked intent + the capability map — concentrating on the non-fakeability claim, the SHA-reset soundness, the ref-pin, and accepted-leak honesty — and resolved the open questions surfaced by the authors (signing-root custody division, on-merge-attempt re-run vs cache semantics, the conclusion vocabulary, and the criteria-count convention). Findings were remediated by the authoring specialists; the orchestrator consolidated.

## Recorded decisions (the binding forks)

- **Consume + default-produce**, not a broker (v1): the gate consumes a signed ledger; butler runs the default executor; the lease/long-poll/label runner protocol is deferred.
- **Non-fakeability is structural**: executor ≠ gated agent · signed · SHA-bound · agent-unwritable ledger · deterministic recording — never the gate.
- **Composes with governance**: one merge gate, two clauses (review + required-checks), both ref-pinned at the target ref; STEER redirects on a miss.
- **v1 signing is symmetric HMAC** (a named accepted residual — a leaked producer secret could forge; Ed25519 + sandboxed executor deferred), chosen to ship the non-fakeability _shape_ while being honest about the residual, exactly as governance named R6.
