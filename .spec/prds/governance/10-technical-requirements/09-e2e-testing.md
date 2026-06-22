---
stability: CONSTITUTION
last_validated: 2026-06-18
prd_version: 1.3.0
section: technical-requirements
---

# E2E Testing — Harness Constitution

The verification rule for this initiative: **real crates, real git, no mocks.** Permission and gate behavior is only proven when the actual `but-authz` crate authorizes against a real committed config read at a real target ref, and the actual GitButler commit/merge path accepts or rejects a real operation on a real repository. A green unit test over a mocked authorizer proves nothing about whether a merge actually lands.

## Framework & real services

| Concern       | Real component (no mock)                                                                                                                                                                                    |
| ------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Authorization | the real `but-authz` crate (`authorize`, `effective_authority`, role desugar, group union)                                                                                                                  |
| Config        | real committed `.gitbutler/permissions.toml` + `.gitbutler/gates.toml` in a real repo, read at a real **target ref** via `gix`                                                                              |
| Commit gate   | the real `but-action` commit path on a real worktree/repo                                                                                                                                                   |
| Merge gate    | the real stack-integration / land path on a real repo, with the real local review record (`local_review_verdicts` — exercised via the governed `but review` submission path, not a forged direct write; R6) |
| CLI           | the real `but` binary (`but perm`, `but group`, the governed `but pr`/`but review`) against a real repo                                                                                                     |
| MGMT UI       | real `packages/ui`/desktop components via Playwright CT — **requires the desktop CT config** (T-MGMT-000 / B14), which does not exist today; `pnpm test:ct` alone runs only `packages/ui`                   |

## The determinism seam

This product governs an _agent_, but the **enforcement is fully deterministic** — there is no model output to fixture. The seam is therefore simple and strict:

- **Fixture the principal identity** (`BUT_AGENT_HANDLE`) and the committed config — these are the inputs.
- **Assert engine OUTCOMES, not prose:** accepted/denied, the exact `error.code`, the named missing `Authority`, whether the ref advanced. Never assert on agent reasoning (the harness owns that; it is out of scope).
- A test that needs an "agent" supplies a scripted principal performing a Butler action — no LLM in the loop. The governance decision is a pure function of (principal, action, target-ref config).

## Landmine ledger (must-have tests)

| Landmine                                    | Test                                                                                                                                                                                                                                                              |
| ------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Role leaks into enforcement                 | grep-assert: no role string (`"write"`,`"admin"`,…) in any enforcement path                                                                                                                                                                                       |
| `Permission` lock overloaded                | grep/structural assert: `Authority` never aliases the repo-access `Permission`                                                                                                                                                                                    |
| N-API bypasses the seam (R14)               | grep-audit: every `but-napi` consequential entry point routes through `but-api`'s `_with_authz`-wrapped / pre-call-guarded action — no direct `but-workspace`/`but-core` call (T-AUTHZ-016b)                                                                      |
| Self-escalation via head                    | feature head adds author to `merge` group → still denied (target-ref read)                                                                                                                                                                                        |
| Stale approval counts                       | approve @H1, advance to H2 → merge blocked until re-approval @H2                                                                                                                                                                                                  |
| Self-approval counts                        | author approves own change with `require_distinct_from_author` → blocked                                                                                                                                                                                          |
| One required group missing                  | only AI (or only human) approval present → blocked; both → lands                                                                                                                                                                                                  |
| Unwrapped action route                      | checklist: every consequential action route is `_with_authz`-wrapped                                                                                                                                                                                              |
| Fence mistaken for a boundary               | the suite asserts NOTHING about raw-git being blocked (it isn't — R1 accepted); a test claiming it is would be a false guarantee                                                                                                                                  |
| Review store mistaken for tamper-proof (R6) | the suite asserts NOTHING about a forged direct DB write to `local_review_verdicts` being blocked (it isn't — R6 accepted-leak); merge-gate tests use the governed `but review` path only; a test claiming the direct write is blocked would be a false guarantee |

## CI lanes

- **fast:** `but-authz` pure-logic (desugar, union, parse) — milliseconds, runs on every push.
- **integration:** real-git commit-gate + merge-gate + CLI — the load-bearing lane; required to pass scoring.
- **invariant:** the grep-asserts (no role names, no `Permission` overload, the N-API seam audit) — a build is not done if these fail, regardless of the other lanes.
- **component (gated on T-MGMT-000):** the MGMT Playwright CT lane — **blocked until the `apps/desktop` CT config exists** (B14); `pnpm test:ct` today runs only `packages/ui`.

## The proven-reference-flow gate (the spike that gates the deep build)

Before the full build, prove **one** end-to-end flow green — the LOOP integration test:

> Three principals (implementer `contents:write`; reviewer `reviews:write` via `code-reviewers`; human `merge` via `maintainers`), a real repo with `main` protected and `gates.toml` requiring an approval from each group. Implementer commits to a feature branch (✓ accepted), attempts merge (✓ denied — no `merge`). Reviewer attempts a commit (✓ denied — inert), submits an approval through the governed `but review` action (✓ recorded @head). Human attempts merge with only the AI approval (✓ blocked), then merges after its own approval (✓ lands). Head advances; a prior approval no longer satisfies (✓ stale dismissed).

That single test exercises AUTHZ + GRPS + GATES + LOOP against real components. **The harness constitution is incomplete until this reference flow is green** — it is the gating prerequisite for the rest of the build, and the canary that catches a fail-closed/fail-open mistake on first contact.
