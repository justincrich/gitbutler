---
stability: PRODUCT_CONTEXT
last_validated: 2026-06-19
prd_version: 1.4.0
enrichment: capability-aware-denials
enriches: .spec/prds/governance/README.md
posture: net-additive progression (sprints 1–8 frozen)
---

# Enrichment v1.4.0 — Capability-Aware Denials (STEER)

> **One line.** Make every actor-correctable governance denial *steer* the caller: alongside *why it failed*, return — in a standard machine-readable shape — *what this principal CAN do right now given its effective permissions*, so a goal-directed AI agent diverts down the governed path instead of hard-quitting, looping, or bypassing.

This is a **net-additive enrichment** of the existing governance PRD. It adds one functional group (**STEER**) and the structured fields that carry the steering payload. It changes **no** existing scope: the gates, the codes, the fail-closed posture, and the four-caller seam are all preserved exactly. It is a **progression** that lands *after* the existing roadmap (Sprints 01a–06b); the changes it implies to already-shipped sprint work are captured as a **delta-replan** ([05-delta-replan.md](./05-delta-replan.md)), not edited into the frozen sprints.

## The irrigation thesis, applied at the moment of denial

The PRD's governing philosophy (Stance 0, `00-overview.md` / `01-architecture-posture.md`) is **irrigation, not a dam**: we grade the riverbed so a goal-directed agent takes the *cheapest* route — the governed one — rather than walling the river. v1.4.0 extends that thesis to the one place the river is most likely to **pool and overflow**: the denial.

Today's denials are honest and well-formed, but they mostly point **up and out** — *"ask a maintainer to grant `merge`"* — toward an out-of-band action the agent cannot perform itself. That is where the water pools: the agent waits on something it can't make happen, and overflows the channel (quits, or defects to a bypass). A **capability-aware** denial points **down and across** — *"with what you hold, you can do X, Y, Z right now"* — so the water always finds a channel. Each rejection becomes a **flow-diverter (a spillway), not a wall.** Role separation (Stance 1) stops being something the agent *discovers by repeated failure* and becomes something the denial *legibly hands it* the instant it strays from its lane.

## The three structural problems this closes

The deep-research base for this enrichment (holocron `js7bqec1xmff8drgx35xav3xvs88ykcp`, "Error Messages as Steering Prompts for AI Coding Agents", HIGH confidence) establishes that in agentic systems **the error string is consumed by the model as its next instruction** — it is a prompt, not a log line. Measured against that literature, three failure modes survive today's contract:

| # | Failure mode | What the agent does | Today's gap | v1.4.0 close |
|---|---|---|---|---|
| 1 | **Hard-quit / "I'm blocked"** | Treats the denial as terminal, abandons the task | the hint names an out-of-band escalation, not an in-grant action it can take now | `authorized_actions` — a lateral menu of moves the caller is authorized for *right now* |
| 2 | **Retry loop** | Re-fires the same denied call, burning tokens | no parseable signal that this class of error is not retryable | `class` (`actor_correctable` vs `operator_required`) + an explicit `do_not` |
| 3 | **Bypass / destructive shortcut** | Defects to raw `git`, `--no-verify`, hook edits | nothing at the denial says the shortcut is a dead end | `do_not` — names the bypass as the slower, non-landing path (Stance 0 spoken aloud) |

The research quantifies the stakes: unhelpful responses raised task **abandonment ~11×** in one study, and *structure* (a named code + recoverability class + a next-action-that-succeeds), not verbosity, is what drives recovery. A coding agent "doesn't need to be perfect — it needs to recover."

## The solution in one shape

A capability-aware denial is HATEOAS-for-authz: each rejection carries the state-transitions available to *this* principal. Additive to the existing denial carriers — `Denial { code, message, remediation_hint }` and `MergeGateError` (which also carries `unmet`); both preserved:

```jsonc
{"error":{
  "code":"branch.protected",                 // existing, stable machine code
  "class":"actor_correctable",               // NEW — actor_correctable | operator_required (gates retry vs escalate)
  "message":"direct commits to protected 'main' are denied for principal 'rev'",   // existing prose
  "remediation_hint":"land 'main' via a reviewed merge",   // existing — vertical path to the ORIGINAL intent
  "held_permissions":["reviews:write","comments:write"],   // NEW — caller's EFFECTIVE set (own ∪ groups), self-scoped
  "authorized_actions":[                       // NEW — lateral affordances derived from held set ∩ route→Authority, intent-scoped
    {"command":"but review request-changes","effect":"reject this change with line comments"},
    {"command":"but review comment","effect":"leave inline feedback"},
    {"command":"but perm list","effect":"see full permissions, groups, and authorized actions"}
  ],
  "do_not":"do not commit directly or bypass with raw git — protected refs only move via reviewed merge"  // NEW — anti-pool prohibition
}}
```

`remediation_hint` (the vertical channel — keep the *original goal* flowing toward landing) and `authorized_actions` (the lateral channel — stop the agent *pooling*) are deliberately distinct and both retained. (`unmet[]` is a `MergeGateError` field on merge denials, not shown on this commit-path example.)

## Why this is a progression, not a rewrite

The implemented contract already ships `code` + `message` (naming the missing authority *and* the held set), with `remediation_hint` + `unmet` on the merge path (the commit/review CLI serializers currently omit `remediation_hint`), serialized as JSON to stderr with exit 1 — genuinely strong. v1.4.0 is the **tuning step** that converts an already-good *informative* error into an optimal *steering* one. Some of what it needs already exists (`effective_authority()` on the deny path, held perms in the `perm.denied` message, the route→Authority mapping in `04-api-design.md`) — but the rust red-hat pass corrected an early "it's all free" framing: the real work (enumerating that mapping into a `Route` table, threading the held set into the `branch_protected` path, updating the three CLI serializers, and reconciling the three error carriers) is sized honestly in [05-delta-replan.md](./05-delta-replan.md) and [03-technical-requirements-delta.md](./03-technical-requirements-delta.md).

## Documents in this enrichment

| File | Section |
|------|---------|
| [README.md](./README.md) | Index, v1.4.0 quick-stats delta, integration points for frozen files |
| [00-overview.md](./00-overview.md) | This file — the irrigation-at-the-denial framing + the three failure modes |
| [01-scope-delta.md](./01-scope-delta.md) | In scope / out of scope for the enrichment (net-add) |
| [02-uc-steer.md](./02-uc-steer.md) | STEER functional group + use cases (UC-STEER-01..06) |
| [03-technical-requirements-delta.md](./03-technical-requirements-delta.md) | Additive contract fields, derivation, invariants, L1/L2/L3 layers |
| [04-e2e-testing-criteria.md](./04-e2e-testing-criteria.md) | Per-UC T-STEER criteria (incl. the no-lying-menu traversability test) |
| [05-delta-replan.md](./05-delta-replan.md) | Sprint 1–8 deltas + proposed Sprint 08 + frozen-file integration edits |
