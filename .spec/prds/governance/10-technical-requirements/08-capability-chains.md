---
stability: CONSTITUTION
last_validated: 2026-06-18
prd_version: 1.3.0
section: technical-requirements
---
# Capability Chains

Boundary-crossing flows where a promise must hold across hops. This initiative has two — both about **resolving and enforcing authority across the config↔action boundary**, where the ref-pin is the contract that prevents self-escalation.

## CAP-AUTHZ-01 — resolve principal → authorize at the action boundary

**Promise:** every consequential GitButler action is permitted only if the *acting principal's effective authority* (own grants ∪ group grants, read at the target ref) contains the required `Authority`; otherwise it is denied with the structured contract and exit 1. **Authorization is a read-only permission check evaluated even under DryRun** — a dry-run commit/merge is still permission-checked (denials still fire); only persistence of refs/objects/oplog is suppressed (RULES.md DryRun semantics). DryRun never bypasses the gate.

| Hop | From → To | Boundary contract |
|---|---|---|
| 1 | orchestrator → Butler action | `BUT_AGENT_HANDLE` injected; the action surface resolves it to a `Principal` (never trusts an agent-supplied authority claim) |
| 2 | action wrapper → `but-authz` | `with_authz(principal, Authority::X)` acquires authority near the wrapper (mirrors `_with_perm`) before the impl runs |
| 3 | `but-authz` → config | load `.gitbutler/permissions.toml` + group config **at the target ref**; compute effective set = own ∪ ⋃(groups) |
| 4 | `but-authz` → action | `Ok(())` proceeds; `Err(Denial)` → `{error:{code:"perm.denied",message,remediation_hint}}`, exit 1 |

- **Failure modes:** missing handle → reject (no anonymous action); config unreadable at ref → fail closed (deny, do not default-allow); role string present in the check → invariant violation (grep-assert); **an N-API caller reaching a lower-level crate without passing hop 2 → ungoverned bypass (R14)** — the promise holds only for callers that route through `but-api`, so the build must audit every `but-napi` entry point (build-gate grep-assert, T-AUTHZ-016b).
- **Real-service proof:** integration test against real `but-api` + real git — a read-only principal is denied a review with the exact contract (UC-AUTHZ-02 AC); the N-API audit is a source/build-gate assertion (T-AUTHZ-016b).
- **Owner:** `rust-implementer` (build) → `security-auditor` + `rust-reviewer` (verify).

## CAP-CONFIG-01 — ref-pinned governed config read (no self-escalation)

**Promise:** a change can never grant itself authority or weaken its own gate, because all governance config is read at the **target ref**, not the change's head.

| Hop | From → To | Boundary contract |
|---|---|---|
| 1 | gate/authz → repo | read the committed config blob **at the target ref** (`gix`), never the working tree or the feature head |
| 2 | repo → resolver | parse permissions + groups + membership as committed on the target branch |
| 3 | resolver → decision | authorize/gate against the target-ref config; a head that edits config is judged by the target-ref version |
| 4 | edit path → effect | `but perm`/`but group` write the working-tree file; the grant is **inert** until committed to the target ref and that ref advances |

- **Failure modes:** reading from head instead of target ref → self-escalation hole (R4); editing protected-branch config without `administration:write` → deny.
- **Real-service proof:** integration test (real git) — a feature head that adds its author to a `merge`-holding group is still denied `merge` (UC-GRPS-02 AC); a head that drops a `gates.toml` requirement is still judged by the target-ref requirement (UC-GATES-02 AC).
- **Owner:** `rust-implementer` (build) → `security-auditor` + `rust-reviewer` (verify).

## Boundary-trigger scan
The PRD's boundary triggers (`grant`, `revoke`, `merge`/`land`, config-read-at-ref, `BUT_AGENT_HANDLE` resolution, the N-API entry-point audit) all map to one of the two chains above. No persistence/network/credential chain beyond these exists in this slice (no remote forge, no signing, no token mint — those are deferred layers). Capability-chain coverage is therefore **complete for the POC scope**.
