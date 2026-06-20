---
stability: CONSTITUTION
last_validated: 2026-06-20
prd_version: 1.1.0
---

# Check Runner — Technical Requirements

A butler-controlled, **local, deterministic** runner that executes a configured
check (e.g. `cargo test`, `pnpm check`, a repo `./script`) and records a
pass/fail result **bound to the current head OID**, which a merge-gate clause
consumes to **block a code change unless every required check passes at that
head**. Check Runner is GitButler's analog of GitHub **Checks / required status
checks** (it is **not** GitHub Actions).

- CLI noun: `but check`
- Crate: `but-checks`
- Verdict store: `check_results` (`but-db`)
- Required-set policy: `[[required_check]]` in `.gitbutler/gates.toml` (ref-pinned)
- Check definitions: `.gitbutler/checks/*.toml` (ref-pinned)

## The framing (read this first)

A required-check is **"a second deterministic review whose verdict is produced
by a local runner instead of a human."** The governance PRD already gates merges
on a deterministic *human* review (`local_review_verdicts` consumed by
`merge_gate.rs::enforce_merge_gate`). Check Runner clones that review clause and
swaps the human producer for a local runner. The **only genuinely new
capability** is the local producer: *run a check + record pass/fail at the head
OID*.

The threat model is **personal-tenant, own-fleet, cheapest-honest-path**: make
the honest path (let the runner run the check) cheaper than cheating; we do
**not** try to make cheating impossible. There is **no runtime isolation and no
cryptography in v1**. Security rests on two cheap things:

1. **Reproducibility** — a check is re-runnable, so a forged green is caught by
   re-running, and honest is cheap. This is the real contrast with governance's
   non-reproducible human-review store — not signing.
2. **Cheap structural locks** — the gate reads a stored fact (not agent prose);
   runner ≠ agent; there is no caller-supplied-conclusion API (negative space);
   results are SHA-bound (keyed by head OID, matched against current head).

**Consistency argument for the trim:** governance already gates merges on
`local_review_verdicts`, which it *accepts* as forgeable-by-direct-DB-write (its
R6). A check is another deterministic review — safer in detectability
(reproducible → forgery detectable post-merge), **not** strictly safer (a check row
has no principal identity). So `check_results` needs **no more protection than the
review store** — a plain `but-db` table is correct. We drop signing / HMAC / Ed25519 /
agent-unwritable-hardening / OS-sandbox-as-security. A nullable `signature`
column is kept as a forward-compat seam (for the day producers go off-host),
explicitly **not** a v1 security claim.

## Section index

| # | File | Purpose |
|---|------|---------|
| 01 | [`01-architecture-posture.md`](./01-architecture-posture.md) | Producer/consumer split, read-only gate, cheapest-honest-path + reproducibility stance, SHA-binding, fail-closed, composition with governance, bootstrap-invariant, "what the gate proves" |
| 02 | [`02-system-components.md`](./02-system-components.md) | `but-checks` crate, `check_results` table, extended `merge_gate.rs` clause, `but check {define,list,run,results,required}` verbs, reused crates, mechanism-agnostic checkout component |
| 03 | [`03-data-schema.md`](./03-data-schema.md) | `check_results` plain table; `.gitbutler/checks/*.toml` def schema; `[[required_check]]` in gates.toml; `Conclusion` enum |
| 04 | [`04-api-design.md`](./04-api-design.md) | run → record → consume sequence; on-merge-attempt as a pre-merge CLI/orchestrator step; `but check` verbs; the negative-space rule |
| 05 | [`05-architecture-diagram.md`](./05-architecture-diagram.md) | ASCII data-flow + control-flow diagram |
| 06 | [`06-external-dependencies.md`](./06-external-dependencies.md) | Likely 0 new deps; explicitly no crypto dep in v1 |
| 07 | [`07-mechanism-agnostic-checkout.md`](./07-mechanism-agnostic-checkout.md) | **Headline section** — the head-OID clean-checkout problem across virtual branches / worktrees / plain git, materialization options, latency budget, shared-worktree non-contention, lock discipline |
| 08 | [`08-technical-risks.md`](./08-technical-risks.md) | Re-ranked register: #1 = mechanism-agnostic head-OID checkout; fail-open via protected early-return; SHA-reset staleness; forgery risks deliberately-not-closed |
| 09 | [`09-capability-chains.md`](./09-capability-chains.md) | CAP-CHECK-01: run → record → consume → deny+STEER → re-run |
| 10 | [`10-frontend-ui.md`](./10-frontend-ui.md) | **v1.1 desktop UI** — what exists (only an aggregate `CIChecksBadge`; per-check data fetched-but-discarded), route-vs-state verdict (state of `/branches`, no new route), 6 net-new components + the reuse inventory, scope tiering (per-head panel un-deferred; merge-summary / settings-tab / matrix / lite deferred) |

## Functional groups

- **DEFN** — check definition, config-as-code (`.gitbutler/checks/*.toml`).
- **RUN** — the local runner/producer + mechanism-agnostic checkout + plain
  result store.
- **GATE** — required-checks merge-gate clause + STEER denial +
  bootstrap-invariant.

## Dependencies (governance prerequisites — verified against live `crates/`)

The GATE group depends on governance work that is **not yet merged**. These are
hard prerequisites, not assumptions — the cited carriers/entry points do not have
the required shape in `crates/` today.

1. **Governance STEER-001** — *the four steering fields + `to_envelope()` on
   `MergeGateError`.* `MergeGateError`
   (`crates/but-api/src/legacy/merge_gate.rs:19-29`) today carries **only**
   `code`/`message`/`remediation_hint`/`unmet`. The steering fields
   (`class`/`held_permissions`/`authorized_actions`/`do_not`) and `to_envelope()`
   do **not** exist in `crates/`; they are deliverables of
   `governance/.../sprint-07-steer-capability-aware-denials/STEER-001`
   (`STATUS: Backlog`, unmerged). **Blocking for the GATE group's STEER denial**
   (01 §7, 04 §5). The GATE group therefore sequences **after** STEER-001; before
   it lands the check denial can carry only the four base fields.

2. **The governance merge gate itself** — `enforce_merge_gate`
   (`crates/but-api/src/legacy/merge_gate.rs:40`) plus the `gates.toml` /
   `[[gate]]` machinery (`GatesWire`/`normalize_gates`, the `read_config_blob`
   ref-pin read, `classify_error`). Check Runner **extends** this gate with a
   required-checks clause; it does not stand alone.

**Sequencing:** the GATE group runs **after** STEER-001 (1). The mechanism-agnostic
local-merge gate entry point (R-ENTRY, 04 §1a / 08) is built **within** the GATE
group, since the shipped `enforce_merge_gate` is forge-`review_id`-only.

## Routing

**No new route.** v1.0 is CLI-first. **v1.1 adds a desktop result-viewing surface**
(see [`10-frontend-ui.md`](./10-frontend-ui.md)) but it is a **state of the existing
`[projectId]/branches` route** — a "Checks" segment + a per-head `CheckResultsPanel`
in the detail pane — not a new route, matching the governance settings-as-modal-state
precedent. A dedicated `[projectId]/checks/` route is a v1.2 concern (warranted only if
the surface must be reached from outside the branches context, e.g. a denied-merge
redirect or a notification deep-link).

## Parent PRD

[`../README.md`](../README.md) — Check Runner PRD overview, scope, roles, use
cases, e2e criteria (authored separately).

## Version history

| Version | Date | Change |
|---------|------|--------|
| 1.0.0 | 2026-06-20 | Initial technical requirements. Reframed from the over-scoped `.spec/prds/actions/` PRD: dropped cryptographic agent-non-forgeable framing; adopted second-deterministic-review + reproducibility + cheapest-honest-path; promoted mechanism-agnostic head-OID checkout to the #1 risk. |
| 1.1.0 | 2026-06-20 | Added `10-frontend-ui.md` — the v1.1 desktop UI contract (component inventory, route-vs-state verdict, scope tiering) from a frontend-designer review of `apps/desktop/`. No new route (state of `/branches`); per-head results panel un-deferred, merge-summary / settings-tab / cross-branch-matrix / lite UI deferred. |
