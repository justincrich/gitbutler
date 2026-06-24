---
stability: FEATURE_SPEC
last_validated: 2026-06-19
prd_version: 1.4.0
functional_group: STEER
enrichment: capability-aware-denials
---

# Use Cases: Capability-Aware Denials (STEER)

The **STEER** group turns each governance denial into a flow-diverter. Where AUTHZ/GRPS/GATES decide _whether_ an action proceeds, STEER governs _what the denial communicates back_ so the caller re-routes itself down the governed path. It is additive: every STEER behavior layers onto the existing denial carriers — `Denial { code, message, remediation_hint }` and `MergeGateError` (which additionally carries `unmet`) — without changing a single gate decision.

| ID          | Title                                       | Description                                                                                                                                                                        |
| ----------- | ------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| UC-STEER-01 | Capability-aware denial shape               | Every actor-correctable denial carries a uniform, back-compatible superset: existing fields + `class` + `held_permissions` + `authorized_actions` + `do_not`.                      |
| UC-STEER-02 | Derived, intent-scoped action menu          | `authorized_actions` is computed as `effective_set ∩ route→Authority`, scoped to the denied intent, each entry `{command, effect}`, from a single source of truth (no lying menu). |
| UC-STEER-03 | Recoverability class + graceful degradation | A `class` field distinguishes adapt-now from stop-and-escalate; the menu degrades to the vertical handoff path when no lateral action exists.                                      |
| UC-STEER-04 | Self-discovery on demand                    | Team/group membership and the full action set are reachable via a self-scoped discovery command surfaced in the menu — provenance served, not pushed inline.                       |
| UC-STEER-05 | Agent-priming reference guidance            | A non-enforced reference primer teaches a harness that denials are redirects and affordances are options, not orders (Stance 6).                                                   |
| UC-STEER-06 | Traversability + anti-injection invariants  | Every offered action actually succeeds for that caller; all steering text comes from a closed code-owned catalog; enforcement never weakens.                                       |

---

## UC-STEER-01: Capability-aware denial shape

Every actor-correctable denial returns one uniform shape: the existing `code` / `message` / `remediation_hint` / `unmet`, **plus** the steering superset `class` / `held_permissions` / `authorized_actions` / `do_not`. The addition is back-compatible — a consumer that reads only the existing fields sees no change — and applies identically across the commit gate, the merge gate, and the authz primitive, including under DryRun.

### Acceptance Criteria

- ☐ System can return, on every actor-correctable denial, the existing `code`/`message`/`remediation_hint`/`unmet` fields plus the new `class`, `held_permissions`, `authorized_actions`, and `do_not` fields in one uniform shape across the commit gate, the merge gate, and the authz primitive.
- ☐ System can preserve each denial serializer's existing keys for back-compatibility, so a consumer reading only `code`/`message`/`remediation_hint` (plus `unmet` on merge `gate.review_required` denials) observes no regression in those keys or exit code 1.
- ☐ System can populate `held_permissions` with the caller's effective `AuthoritySet` (own grants ∪ group grants) as a structured, stably-ordered array when a principal is resolved (the `missing_permission` path), formalizing the held-permissions list currently embedded in the `perm.denied` message prose; the field is structurally empty on the unresolved-principal and `config.invalid` paths.
- ☐ System can emit the full steering payload under DryRun, so a DryRun denial carries `class`/`held_permissions`/`authorized_actions`/`do_not` while persisting nothing (preserving DryRun-no-bypass).
- ☐ System can retain `remediation_hint` (the vertical path to the original intent) and `authorized_actions` (the lateral in-grant options) as distinct fields, so neither replaces the other.

---

## UC-STEER-02: Derived, intent-scoped action menu

`authorized_actions` is the choose-your-own-adventure menu: the governed `but` actions the caller is authorized to run _right now_, derived from its effective set intersected with the route→Authority table, scoped to the denied intent so the menu is relevant rather than exhaustive. It is the lateral channel that prevents the agent from pooling at the dam — and it is computed from the same source of truth the gate enforces against, so it can never offer an action that would itself be denied.

### Acceptance Criteria

- ☐ System can compute `authorized_actions` as the intersection of the caller's effective set and the route→Authority table, so every listed action is one the caller is authorized to run.
- ☐ System can scope the menu to the denied intent via a denied-action→affordance-category map, so a denied commit-to-protected surfaces landing and review actions rather than the entire command catalog.
- ☐ System can render each `authorized_actions` entry as a `{command, effect}` pair carrying the literal `but` command and a one-line effect description.
- ☐ System can derive the menu from the same route→Authority table and the same target ref the gate enforced against, so the menu never lists an action that would itself be denied (no lying menu).
- ☐ A Reviewer agent denied a commit can see its review actions (`but review request-changes` / `comment` / `approve`) in `authorized_actions` and follow one to a successful governed action.
- ☐ System can draw all `authorized_actions` command and effect text from a closed, code-owned catalog, so no free-form, interpolated, or model-generated text appears in the menu.
- ☐ System can exclude `but review approve` from `authorized_actions` when the denied action targets the caller's own branch, so the menu never surfaces a self-approval path — an L1 contract exclusion, not left to the reference primer.

---

## UC-STEER-03: Recoverability class + graceful degradation

A `class` field tells the agent, without parsing prose, whether the denial is something it can route around (`actor_correctable`) or something only an operator can fix (`operator_required`). Operator-required denials carry an empty menu and an explicit "do not retry" so the agent stops instead of looping; actor-correctable denials with no lateral move left degrade gracefully to the existing vertical handoff path.

### Acceptance Criteria

- ☐ System can tag actor-correctable denials — a resolved principal lacking authority (`perm.denied`), plus `branch.protected` and `gate.review_required` — `class: actor_correctable`, and operator-required denials — `config.invalid` plus the unresolved-principal cases (unset `BUT_AGENT_HANDLE` / unknown principal, which carry the `perm.denied` code but admit no in-system self-correction) — `class: operator_required`.
- ☐ System can return an empty `authorized_actions` and a `do_not` of "do not retry — this requires an operator" for `operator_required` denials, so the agent stops rather than re-firing the call.
- ☐ System can degrade to the vertical path — `authorized_actions` empty, `remediation_hint` naming the handoff or admin grant — when an actor-correctable caller holds no relevant lateral action, so the denial still routes the agent somewhere or honestly reports the dam.
- ☐ System can frame `do_not` on actor-correctable denials as "the governed path is the only route to a landed change" using positive-only framing that does not enumerate bypass mechanics by default.
- ☐ An Orchestrator can branch on `class` to choose retry-vs-escalate without parsing the human-readable message.
- ☐ System can emit a structured denial-steering event (`code`, `class`, `had_lateral_action`, menu length) on the existing tracing/log path, so a fleet operator can measure whether steering actually reduces hard-quits and loops.

---

## UC-STEER-04: Self-discovery on demand

A caller that wants the full picture — its complete permissions, the groups behind them, every action it is authorized for — gets it from a self-scoped discovery command surfaced as one of the `authorized_actions`. Team/group membership is provenance, not capability, so it is served on request rather than pushed inline into every denial.

### Acceptance Criteria

- ☐ System can surface a self-scoped discovery action (`but perm list`, or a `but whoami` / `but can-i` entry point) as one of the `authorized_actions` on every actor-correctable denial.
- ☐ System can omit group/team membership from the inline denial by default, so the denial stays lean and the effective set (which already subsumes group grants) carries the actionable information.
- ☐ A principal invoking the discovery action can see its full effective permissions, the groups behind them, and its complete authorized-action set (self-scoped, the same disclosure class as `but perm list`).
- ☐ System can keep discovery self-scoped, so it does not disclose other principals' permissions and cross-principal reconnaissance stays gated as in Sprint 05.
- ☐ System can scope the `but whoami` / `but can-i` discovery so it discloses the caller's own group memberships but does not enumerate the other members of those groups (group-roster recon stays gated by `administration:read` as in Sprint 05).

---

## UC-STEER-05: Agent-priming reference guidance

The denial is necessary but not sufficient: whether an agent treats it as a redirect or a wall is decided in the harness. GitButler ships a short reference primer a harness/orchestrator MAY adopt — denials are redirects, affordances are authorized options not orders, bypass is never the faster path — without making the engine depend on it (Stance 6: the harness owns the agent).

### Acceptance Criteria

- ☐ System can ship a reference primer (doc or snippet) in the repo stating that `but` denials are redirects rather than terminal failures, that `authorized_actions` are authorized options, and that bypass is never the route to a landed change.
- ☐ A maintainer can confirm the primer is marked non-enforced reference material and that no `but-authz` / `but-api` code path depends on it for correctness (Stance 6).
- ☐ A maintainer can find, in the primer, an explicit statement that an agent should choose the `authorized_actions` entry that serves its actual task — affordances are options, not orders (goal integrity).
- ☐ A maintainer can find, in the primer, documentation of the `class` and `do_not` contract (stop on `operator_required`; never bypass).

---

## UC-STEER-06: Traversability + anti-injection invariants

The steering payload must be _true_ and _safe_: every action it offers must actually succeed for that caller (a lying menu is worse than none — it sends the water down a blocked channel and loops), and the steering text must be un-forgeable (errors are a known prompt-injection surface). Both are proven by extending the existing `governed_loop` test harness, and the steering payload never weakens enforcement.

### Acceptance Criteria

- ☐ System can prove, via the extended `governed_loop` harness, that every action offered in a denial's `authorized_actions` actually succeeds for that caller (no lying menu).
- ☐ System can prove, via a test/build-gate, that the menu and the gate read the same route→Authority table at the same target ref (single-source derivation).
- ☐ System can prove, via a build-gate, that all `authorized_actions` and `do_not` text originates from the closed code-owned catalog, with no free-form or interpolated content beyond catalog entries and already-validated identifiers.
- ☐ System can preserve the fail-closed posture, so if the steering payload cannot be derived the denial still returns `code`/`message`/`remediation_hint` and exit 1 — steering degrades, enforcement never weakens.
- ☐ System can determine `class` by an exhaustive, non-defaulted mapping over the denial's (code, principal-resolution) cases, so adding a future denial code without explicitly classifying it is a build break rather than a silent `actor_correctable`.
