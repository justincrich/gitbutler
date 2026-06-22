---
stability: FEATURE_SPEC
last_validated: 2026-06-20
prd_version: 1.0.0
---

# Functional Groups

Check Runner is three groups — the **definition** (what a check is), the **producer** (the runner that runs it and records the result at the head OID), and the **consumer** (the gate clause that blocks a merge on it). There is deliberately **no separate "ledger" group**: the result store is a plain `but-db` table folded into RUN, because under the own-fleet reproducibility bar it needs no more hardening than the governance review store (see [00-overview.md](./00-overview.md), [02-roles.md](./02-roles.md)).

| Group                    | Prefix | Description                                                                                                                                                                                                                                                                                                                                                             |
| ------------------------ | ------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Check Definition**     | DEFN   | Config-as-code named checks in committed, ref-pinned `.gitbutler/checks/*.toml` (name + trigger + local run-spec + `required` + exit-code success mapping); the required-set policy (`[[required_check]]` in `gates.toml`) read at the target ref; no self-weakening; the self-protecting bootstrap-invariant.                                                          |
| **Check Runner**         | RUN    | The butler-controlled local runner: runs the real check (runner ≠ agent) against a **mechanism-agnostic clean checkout at the current head OID**, derives the conclusion from the real exit code, records it bound to the head OID in a plain store, and handles the `on-commit` / pre-merge triggers, timeouts, concurrency, and dual-audience observability.          |
| **Required-Checks Gate** | GATE   | A new clause on the governance merge gate: block unless every required check is `success` @ the current head OID. Composes with the existing `merge`-authority + review clauses, is a read-only consumer (never runs a check), fails closed (independent of the `protected` flag), enforces the bootstrap-invariant, and emits a STEER denial that redirects the agent. |

## Use Case Summary

| Group                | Prefix | UCs                     | Count  |
| -------------------- | ------ | ----------------------- | ------ |
| Check Definition     | DEFN   | UC-DEFN-01 … UC-DEFN-05 | 5      |
| Check Runner         | RUN    | UC-RUN-01 … UC-RUN-05   | 5      |
| Required-Checks Gate | GATE   | UC-GATE-01 … UC-GATE-05 | 5      |
| **Total**            |        |                         | **15** |

## The produce → consume → redirect loop

The three groups form one loop, reusing the governance gate and STEER:

1. **DEFN** names the checks and which are required, ref-pinned at the target ref.
2. **RUN** (the runner, not the agent) checks out the head OID, runs the required `on-merge-attempt` checks in the pre-merge step, and records `success`/`failure` bound to that OID.
3. **GATE** reads the current-head results and blocks unless every required check is `success`; on a miss it emits a STEER denial naming the unmet check + the `but check run …` next action.
4. The orchestrator consumes STEER and re-routes the agent to run/fix the check; on success at the new head, the merge proceeds.

Each group's load-bearing property: DEFN's **ref-pin + bootstrap-invariant** (a change cannot weaken the checks that judge it), RUN's **runner ≠ agent + mechanism-agnostic head-OID binding** (the verdict is produced by the trusted process against the exact code, regardless of branching), and GATE's **read-only fail-closed consumption** (the gate never runs a check, never trusts a claim, and never fails open).
