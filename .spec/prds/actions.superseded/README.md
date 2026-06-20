---
title: Actions — Configurable Validations for GitButler (POC)
version: 1.0.1
scope_posture: full
pr_sequencing: false
---

# Actions — Configurable Validations for GitButler (POC) · PRD

Introduce a configurable, **GitHub-Actions-like validations** layer in GitButler: config-as-code **named checks** that a **butler-controlled executor** runs against a change, recording a **signed, SHA-bound result** in an **agent-unwritable ledger** that the **merge gate consumes as required checks** — so an agent's change cannot land until every required check is provably green on the current head. This is the **verification/quality** half of "the way to production code is governance + accountability": the governance PRD enforces *process* (permission + an approval exists); Actions enforces *quality* (the code provably works), non-fakeably.

> **Builds ON governance, composes — does not replace.** Actions is the **producer** of check results; the governance merge gate (`crates/but-api/src/legacy/merge_gate.rs` `enforce_merge_gate`, already shipped) is the **consumer** via a new required-checks clause; **STEER** redirects an agent on a missing/failed check. Denial semantics reuse governance's `{code, message, remediation_hint}` + STEER fields, adding one code `gate.check_required`. The **LEDG** group is the *hardened* counterpart to governance's knowingly-forgeable review store (its R6).

> **The thesis — done is proven, not claimed.** A coding agent has no out-of-band accountability, so quality cannot rest on its self-report ("tests pass"). Non-fakeability comes from **who produces the result (an executor that is NOT the gated agent) + how it is bound (signed, pinned to the head SHA, recorded in an agent-unwritable ledger)** — never from the gate. The gate is a deterministic consumer.

## PRD Metadata

| Field | Value |
|-------|-------|
| Version | 1.0.1 |
| Scope Posture | Full feature (default — a complete POC slice) |
| PR Sequencing | Disabled |
| Created | 2026-06-19 |
| Last Updated | 2026-06-19 |
| Specialists | product-manager (lead) · rust-planner (engineering) · security-auditor (non-fakeability/trust model) |
| Target | GitButler `crates/*` — a new `but-checks` crate + a new `but-db` `check_results` ledger table + an EXTENDED `but-api` merge gate (required-checks clause) + new `but check` CLI verbs; built atop the shipped governance merge gate + `but-authz` |
| Grounding | holocron `js7f9r8cf771qcf0t97dkx38hs890ywq` (GHA→butler capability map) + `js7058pqz024dsw5x3ap85mkx5891857` (execution/checks slice) |

## Document Index

| File | Section | Stability |
|------|---------|-----------|
| [00-overview.md](./00-overview.md) | Product description, the problem (process ≠ quality; agents fake "tests pass"), the producer/consumer solution, composition with governance + STEER | PRODUCT_CONTEXT |
| [01-scope.md](./01-scope.md) | In scope (v1) / out of scope (deferred) / Known Limitations | FEATURE_SPEC |
| [02-roles.md](./02-roles.md) | Human fleet-owner/admin · implementer/reviewer agents (subject to checks) · the butler executor (trusted producer) · GitButler engine (deterministic recorder + gate) | PRODUCT_CONTEXT |
| [03-functional-groups.md](./03-functional-groups.md) | 4 groups (DEFN · EXEC · LEDG · GATE) + use-case summary | FEATURE_SPEC |
| [04-uc-defn.md](./04-uc-defn.md) | UC-DEFN-01..05 — config-as-code named checks; ref-pinned (no self-weakening); local-action resolution; the bootstrap-invariant | FEATURE_SPEC |
| [05-uc-exec.md](./05-uc-exec.md) | UC-EXEC-01..05 — the butler default executor; executor ≠ agent; real execution; isolated secrets; timeout/concurrency/observability | FEATURE_SPEC |
| [06-uc-ledg.md](./06-uc-ledg.md) | UC-LEDG-01..04 — signed `(name, head_sha, conclusion)` records; agent-unwritable; the SHA-reset invariant | FEATURE_SPEC |
| [07-uc-gate.md](./07-uc-gate.md) | UC-GATE-01..04 — required-checks clause consuming the ledger; composes with governance; STEER redirect; fail-closed | FEATURE_SPEC |
| [08-team-contributions.md](./08-team-contributions.md) | Planning provenance (assimilation → capability map → specialist authoring → red-hat review) | - |
| [09-technical-requirements/](./09-technical-requirements/README.md) | Engineering contract — folder; producer/consumer split, the `check_results` ledger schema, the merge-gate integration point, SHA-reset machinery, capability chains, risks | CONSTITUTION |
| [10-e2e-testing-criteria.md](./10-e2e-testing-criteria.md) | Per-UC criteria (real executor + real git, no mocks); incl. the produce→consume→redirect loop and SHA-reset-under-mutation demos | TEST_SPEC |

## Quick Stats

| Metric | Value |
|--------|-------|
| Functional Groups | 4 (DEFN · EXEC · LEDG · GATE) |
| Use Cases | 16 |
| Acceptance Criteria | 91 (DEFN 21 · EXEC 23 · LEDG 23 · GATE 24) |
| E2E Testing Criteria | 93 (91 per-AC 1:1 — 76 integration-test · 7 api-contract · 8 build-gate — + 2 additive [human-gate] full-loop demos) |
| System Components | NEW `but-actions` crate (executor + contract types + signing/verify) · NEW `but-db` `check_results` ledger table · EXTENDED `but-api` `merge_gate.rs` (required-checks clause) · NEW `but check {define,list,run,results,required}` CLI verbs · REUSED `but-authz` / `but-secret` / `but-rebase` |
| Data Schema | NEW `check_results(name, head_sha, conclusion, producer_identity, signature, recorded_at, metadata)` (signed, agent-unwritable) · `.gitbutler/actions/*.toml` action defs (ref-pinned) · `[[required_check]]` in governance `gates.toml` (ref-pinned) |
| New CLI Surface | `but check {define, list, run, results, required}` |
| Risk Register | see [09-technical-requirements/07-technical-risks.md](./09-technical-requirements/07-technical-risks.md) — executor isolation (Blocking), SHA-reset race (Blocking), signing-key/ledger integrity (High; v1 HMAC symmetric = named accepted residual), fork/untrusted + remote-runner (Accepted/deferred), governed-merge-only binding (inherited governance R11/R14) |
| External Dependencies | 0 new expected (reuse `gix`/`but-db`/`toml`/`serde`/`sha2`/`hmac`/`but-secret`); a signing-lib add named conditionally |

## Version History

| Version | Date | Changes | Trigger |
|---------|------|---------|---------|
| 1.0.0 | 2026-06-19 | Initial PRD — 4 groups (DEFN · EXEC · LEDG · GATE), 16 UCs (91 ACs), TR folder (9 files), 93 e2e criteria. The configurable-validations / required-checks layer: a butler-run executor produces signed SHA-bound check results in an agent-unwritable ledger; the governance merge gate consumes them as required checks; STEER redirects on a miss. Assimilated from GitHub Actions (holocron `js7f9r8cf771qcf0t97dkx38hs890ywq`); built atop the shipped governance merge gate + `but-authz`. | New initiative |

## Next Steps

- `/review-red-hat` — adversarial review. Hostile eyes belong on: the **non-fakeability** four-part claim (executor≠agent · signed · SHA-bound · agent-unwritable), the **SHA-reset invariant** under graph rewrite (a green on an ancestor SHA must never satisfy the gate — the named "match-by-name-alone" hole), the **ref-pin** (an agent cannot weaken/drop a required check in its own change), and the **accepted-leak honesty** (v1 symmetric-HMAC forgeability, governed-merge-only binding) never presented as bound.
- `/kb-sprint-plan .spec/prds/actions` — build the sprint roadmap. The proven-reference-flow (produce→record→gate-consume→deny→STEER→fix→pass→land against real components) should precede the deep build.
- Land as a native GitButler capability composing with governance; the executor/ledger/gate are extensible toward the deferred layers (Ed25519-signed results, sandboxed executor, remote/labelled runners, the `${{ }}` expression engine, a management UI).
