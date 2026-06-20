---
stability: PRODUCT_CONTEXT
last_validated: 2026-06-18
prd_version: 1.3.0
---
# Roles

This initiative defines the **trust boundary** the governance layer draws: who is controlled by **outcomes** (the orchestrator, bounded only by the two gates) versus who is controlled **action-by-action** (the principals GitButler runs git actions for). Every "role" below is **illustrative** — a label for a common *functional permission bundle*, never an enforcement key. A principal simply *is* the `AuthoritySet` it holds (its own grants ∪ its groups' grants); nothing in enforcement reads a role name (AUTHZ invariant, grep-asserted).

| Role | Description |
|------|-------------|
| **Human maintainer** | A human contributor/owner who, in the demonstration, holds `merge` + `administration:write` (typically via a `maintainers` group). The **final feature-level approver**: GitButler's merge gate can be configured to require an approval from this group before anything lands on a protected branch. Also the only principal who can change governed config (`.gitbutler/permissions.toml`, `.gitbutler/gates.toml`). A maintainer granted every permission is a **superuser** — just a principal holding all functional permissions. |
| **Orchestrator (coding harness)** | An external harness (Claude Code, Codex, OpenCode, a ralph loop) that dispatches agents and drives GitButler via the `but` CLI. **Trusted / uncontrollable** — your harness, your keys, your reasoning loop. Governs nothing about *how* it runs agents (Assumption 2). Enforced only by **outcomes**: the two gates stop it landing a rule-violating commit/merge; its behavior is not policed action-by-action. It injects each dispatched agent's `BUT_AGENT_HANDLE`. |
| **Implementer agent** | An LLM worker the orchestrator runs, holding (illustratively) `contents:write` + `pull_requests:write` but **not** `merge`. It can land commits on a feature branch/stack and open a PR, but the merge gate denies it the land — legibly. The **AI at the code level** in the demonstration, on the producing side. "Implementer" is a bundle label, not a built-in role. |
| **Reviewer agent** | An LLM worker holding (illustratively) `reviews:write` + `comments:write` + `contents:read`, typically via a `code-reviewers` group, but **not** `contents:write`. Its edits are **structurally inert** (it cannot commit them); it can review and comment. The **AI at the code level** on the checking side — it catches obvious, programmatically-hard-to-assert misses (a wholly-missing feature) before work reaches the human. |
| **Administrator** | The principal holding `administration:write`. Authority to edit governed config — grant/revoke functional permissions, manage groups, edit `gates.toml`. Usually the same human as the maintainer; called out separately because config-change authority is its own functional permission, independently grantable. |
| **GitButler (the enforcement engine)** | Not a principal — the **policy-enforcement system itself**. It resolves the acting principal from `BUT_AGENT_HANDLE`, loads the functional `AuthoritySet` from ref-pinned config, authorizes every consequential git action, and evaluates the two gates as deterministic code on its action path. The enforcer of every "cannot" in this PRD. (Distinct from GitButler's pre-existing repo-access `Permission` lock, which it does not replace.) |

## Principal lifecycle — register-on-first-grant / decommission (B9)

A **principal** is just an entry in `.gitbutler/permissions.toml` carrying a functional `AuthoritySet`. There is no separate registration ceremony:

- **Register-on-first-grant.** Granting a permission to a principal that does not yet exist in `permissions.toml` (`but perm grant --principal <new> …`, or the MGMT editor's first save for a new id) **implicitly creates the principal entry**. The principal comes into existence as soon as it holds its first grant or group membership.
- **Decommission by emptiness.** A principal with **no own grants and no group memberships** is effectively decommissioned — it is denied all actions by fail-closed default-deny (UC-AUTHZ-04). Revoking every grant and removing every membership is the decommission path; there is no separate "delete principal" verb in the POC (an entry with an empty set is inert).

This keeps the lifecycle declarative: the config file *is* the registry, and a principal's existence and authority are both read at the target ref.

## Admin role vs `administration:write` — the v1 reconciliation (B18)

GitButler already ships a cloud `User.role === 'admin'` flag (used for sidebar visibility in the desktop app). The governance layer's `administration:write` is a **distinct, functional** permission. The v1 split:

- **`User.role === 'admin'` (cloud)** gates **UI visibility** — whether the MGMT settings page appears in the sidebar (UC-MGMT-01). This is **UX convenience**, combined with the implicit human-fleet-owner trust (R12).
- **`administration:write` (functional permission)** gates **CLI/server writes** — `but perm`/`but group`/gate edits, and every governance Tauri command. This is the **enforcement boundary**; the renderer `adminOnly` filter is not.
- **Post-v1 (C4):** a per-project `administration:write` check replaces the global cloud role for governance-specific gating, so visibility and enforcement use the same per-project authority. See [01-scope.md § Known Limitations](./01-scope.md#known-limitations), [`04-api-design.md` Identity & confinement](./10-technical-requirements/04-api-design.md), and `07-technical-risks.md` R12.

## v1 trust assumption — the human fleet-owner

For v1, **the human at the keyboard is a trusted superuser over their own agent fleet**. The goal of this slice is to let any human manage a fleet of agents in their own codebase, so the desktop user (the signed-in `UserService` / forge session) is the fleet **owner**: they hold implicit authority over governance-config management — `but perm` / `but group`, gate edits, and the MGMT UI writes — by virtue of being the human who owns the repo and the agents, **not** by an explicit checked grant in `permissions.toml`. This is **personal-tenant trust**, the same trust class as the orchestrator (your machine, your keys): the human is trusted to be who the desktop session says they are.

This is **distinct from the agent authorization model**: agents remain bound action-by-action by functional permissions, with no superuser path and no implicit allow (the UC-AUTHZ invariants are unchanged). The human superuser is the *trust root* for config management — the owner who stands outside the agent-permission model and manages it — not an agent with elevated permissions. An agent cannot reach this trust root: it is still resolved from `BUT_AGENT_HANDLE` and denied any action its functional set does not contain.

**Accepted risk (v1):** anyone at the keyboard, or a compromised desktop session, can manage the fleet's governance config (grant/revoke agent permissions, edit gates, manage groups) as the owner. **Future improvement:** a real per-human **authenticated principal** — key-mint, or a checked `administration:write` entry in `permissions.toml` rather than implicit ownership — so the model scales beyond one trusted human. See [01-scope.md § Known Limitations](./01-scope.md#known-limitations) and [`10-technical-requirements/07-technical-risks.md`](./10-technical-requirements/07-technical-risks.md) R12.

## Persona-to-Role-to-UC Traceability

| Persona | Role | Primary UCs (most-served) |
|---------|------|---------------------------|
| **Justin** (fleet manager / governance owner) | Human maintainer + Administrator | UC-AUTHZ-01 (defines the functional sets), UC-GRPS-01/02 (creates groups, grants to them), UC-GATES-02 (configures the merge requirement), UC-LOOP-02 (final feature-level approval) |
| **An implementer** (`contents:write`, no `merge`) | Implementer agent | UC-GATES-01 (feature-branch commit accepted, protected-branch commit rejected), UC-AUTHZ-02 (denied `merge` legibly), UC-LOOP-01 (produces the change the gate judges) |
| **A reviewer** (`reviews:write`, no `contents:write`) | Reviewer agent | UC-AUTHZ-02 (denied a commit legibly; edits inert), UC-GATES-02 (its distinct approval@head satisfies the merge requirement), UC-LOOP-01 (catches the code-level miss) |
| **Claude Code** (coding harness) | Orchestrator | UC-AUTHZ-03 (injects/confines `BUT_AGENT_HANDLE`), UC-GATES-01/02 (bounded by the gates' outcomes), UC-LOOP-01 (drives the loop) |

Coverage check: every UC is primarily exercised by at least one persona; **GitButler (the engine)** is the deterministic actor behind every enforcement AC (authorization, gate evaluation, permission denial) — every such AC names the commit gate, the merge gate, or the authorization check as the WHO.
