---
stability: FEATURE_SPEC
last_validated: 2026-06-19
prd_version: 1.4.0
enrichment: capability-aware-denials
scope_posture: net-additive
---

# Scope Delta — v1.4.0 Capability-Aware Denials

This enrichment is **purely additive** to the v1.3.0 scope. Nothing in the existing five groups (AUTHZ · GRPS · GATES · LOOP · MGMT) is removed, narrowed, or re-scoped. The enforcement decision (who may do what, whether a change lands) is unchanged; v1.4.0 only changes **what a denial communicates back** so the caller can re-route itself.

## In scope (NEW — the STEER group)

- **A standard capability-aware denial shape.** Three additive fields on the existing `Denial` contract — `class`, `held_permissions`, `authorized_actions` — plus a `do_not` prohibition, carried uniformly by every actor-correctable denial across both gates and the authz primitive. `code`, `message`, `remediation_hint`, and `unmet` are **preserved unchanged** (back-compat + human readability).
- **A derived, intent-scoped `authorized_actions` menu.** Computed deterministically as `effective_set ∩ route→Authority table`, filtered to the actions relevant to the denied intent, with each entry `{command, effect}`. The single source of truth is the same route→Authority table the gates enforce against.
- **A recoverability `class`** (`actor_correctable` | `operator_required`) that tells the agent whether to adapt-now or stop-and-escalate, with graceful degradation to the existing vertical "hand off / ask an admin" path when no lateral action exists.
- **Self-discovery on demand.** Team/group membership and the full permission/action set are reachable via `but perm list` (self-scoped) — a **Sprint 05 deliverable this enrichment depends on**, not yet shipped — surfaced as one of the `authorized_actions` when present (degradable: omitted rather than offering a phantom command until the verb lands), optionally via a friendlier **net-new** `but whoami` / `but can-i` entry point that STEER builds. Provenance is served, not pushed inline; discovery shows the caller's own memberships but not other members of its groups.
- **Reference agent-priming guidance (L2, non-enforced).** A short, shippable "denials are redirects; affordances are not orders; never bypass" primer that a harness/orchestrator MAY adopt. Documented as reference; **not** an enforcement requirement (consistent with Stance 6 — the harness owns the agent).
- **Traversability + anti-injection invariants (L3).** Every action a denial offers must actually succeed for that caller (no "lying menu"); steering text is drawn from a closed, code-owned catalog (no free-form or injectable content). Proven by extending the existing `governed_loop` test harness.

## Out of scope (deferred / non-goals)

| Item                                                               | Disposition                                                        | Why                                                                                                                                                                                                                                                  |
| ------------------------------------------------------------------ | ------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Surfacing `authorized_actions` in the **MGMT desktop UI**          | DEFERRED (future Sprint 06c / MGMT enrichment)                     | The denial contract is backend/CLI-first; the UI can render the same payload later. Keeps this enrichment off the in-flight Sprint 06a/06b surface.                                                                                                  |
| **Cryptographic** trust on the steering payload (signing the menu) | OUT (same class as the deferred HMAC/Ed25519 review integrity, R6) | The menu is derived from already-trusted committed config; integrity rides on the existing deferred hardening, not a new mechanism.                                                                                                                  |
| Changing **which** actions are gated, or the **codes** themselves  | OUT (non-goal)                                                     | v1.4.0 changes the denial's _payload_, never the gate decision or the code set `{perm.denied, branch.protected, gate.review_required, config.invalid}`.                                                                                              |
| Driving the agent's **reasoning** (forcing it to obey the menu)    | OUT (non-goal — Stance 6)                                          | We disclose affordances; the harness owns whether/how the agent acts. The primer is reference-only.                                                                                                                                                  |
| An **LLM-generated** hint/menu                                     | OUT (non-goal)                                                     | The menu is a deterministic projection (static mapping), never model-generated — consistent with the deterministic-vs-probabilistic doctrine.                                                                                                        |
| Naming the bypass **mechanics** in `do_not`                        | OUT (default) — positive-only framing                              | `do_not` frames the governed path as the only route to a landed change without enumerating bypass techniques (avoids the information hazard). See [03](./03-technical-requirements-delta.md).                                                        |
| Human-TTY rendering of the new fields + i18n of catalog strings    | DEFERRED / best-effort                                             | The steering payload is the structured (stderr-JSON) contract for programmatic/agent consumers; human-TTY rendering of `class`/`authorized_actions`/`do_not` is best-effort, and catalog strings are English-only in this slice (named, not denied). |
| Tauri / N-API steering serialization (if not co-landed)            | DEFERRED (explicit decision, not a silent gap)                     | New fields reach the three CLI serializers in Sprint 08; the Tauri/N-API `json::Error` path co-lands with Sprint 06a `MGMT-IPC-002` or is explicitly deferred. See [03 §7](./03-technical-requirements-delta.md).                                    |

## Preserved from sprints 1–8 (explicit)

The enrichment **depends on and preserves** every shipped surface: the `but-authz` `Denial`/`AuthoritySet`/`authorize()` primitives (Sprint 01a/02), both gates (Sprint 01a/01b/04), the route→Authority table (`04-api-design.md`), `effective_authority()` (Sprint 03), the CLI JSON serialization path (Sprint 01b + Sprint 06a `MGMT-IPC-002`), and `but perm list` (Sprint 05). Where v1.4.0 implies a change to any of these, it is recorded as a delta to apply when the freeze lifts — never edited into the frozen sprint now. See [05-delta-replan.md](./05-delta-replan.md).
