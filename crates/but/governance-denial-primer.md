# Governance Denial Primer

> **Status: non-enforced reference material.**
>
> This document is guidance a harness or orchestrator MAY adopt. It is
> non-enforced reference material — the `but-authz` / `but-api` engine does
> **not** read, import, `include_str!`, or branch on this primer for
> correctness (Stance 6: the harness owns the agent; the engine owns the
> gate). A build-gate test proves no engine code path depends on it.

## Why this exists

A `but` denial carries everything an actor needs to redirect: a `code`, a
human-readable `message`, a `remediation_hint`, and an `authorized_actions`
menu derived from the same route→Authority table the gate judged against. The
denial is deterministic and code-owned (L1). How an agent *reasons* about that
denial — whether it treats it as a redirect or a wall — is decided in the
harness (L2). This primer is the L2 reference a harness MAY load into its
agent's context so the agent cooperates with the governed path instead of
fighting it.

## Denials are redirects, not terminal failures

When `but` denies an action, the denial is a **redirect**, not a terminal
failure. The denial names *what* was missing and offers the actions that *are*
authorized for this caller at this ref right now. A well-primed agent reads
the `authorized_actions`, picks the entry that serves its actual task, and
continues — it does not abort the run or escalate to a human unless the
denial's `class` says to.

## Authorized actions are options, not orders

The `authorized_actions` in a denial are **authorized options, not orders**.
They are the actions the caller is *permitted* to take, not instructions on
which one to choose. Pick the entry that **serves your actual task** — the one
that makes progress toward the goal you were given (goal integrity). If none
of the offered actions serve the task, that is a signal to stop and ask a
human, not to improvise a path outside the menu.

## Bypass is never the route to a landed change

Bypass — raw git, `--no-verify`, editing refs directly, disabling a gate — is
**never** the route to a landed change. The governed path (`but` commit /
merge / config) is the only route to a change that actually lands. Bypass
silently drops the guarantees the gates exist to provide (review, authority,
audit). Even when bypass *appears* faster in the moment, the change it
produces is not a governed change and will be rejected downstream. The faster
path is always to follow the redirect the denial offers.

## The `class` / `do_not` contract

Every actor-correctable denial carries a `class` and a `do_not` contract. The
`class` tells the actor whether *it* can correct the situation or whether a
human must. The `do_not` names exactly what the actor must not do.

- **`actor_correctable`** — the actor can resolve this itself by following the
  `authorized_actions` redirect. **Never bypass**: the correction is already
  within your authority via the offered actions. Pick one and proceed.
- **`operator_required`** — the actor cannot self-correct. **Stop** and surface
  the denial to a human operator. Do not attempt to improvise a fix; do not
  retry with different arguments hoping to slip past the gate.

The contract is simple: on `operator_required`, **stop**; on
`actor_correctable`, follow the menu and never bypass. The `do_not` field is
the explicit negative — honor it.
