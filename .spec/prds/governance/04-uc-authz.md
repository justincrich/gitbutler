---
stability: FEATURE_SPEC
last_validated: 2026-06-18
prd_version: 1.3.0
functional_group: AUTHZ
---

# Use Cases: Functional Permission System (AUTHZ)

Permissions are **functional, not role-based**. The authoritative config is a per-principal **set of functional permissions** mirroring GitHub's fine-grained model (`contents:write`, `pull_requests:write`, `reviews:write`, `merge`, …), stored in committed, ref-pinned `.gitbutler/permissions.toml`. Named roles (read/triage/write/maintain/admin) are **optional presets** that the loader desugars into a functional set; enforcement only ever sees the set, and a superuser is just a principal granted every permission. Every consequential GitButler action checks the principal's functional permission, and a denial is the structured, agent-readable rejection contract — never a silent skip and never keyed off a role name. The layer lives in a new `but-authz` crate, named distinctly from GitButler's pre-existing `Permission` (a repository-access lock), which it does not replace.

> **Principals have permission sets, not roles.** "Implementer" and "reviewer" anywhere in this PRD are _illustrative labels_ for common permission bundles — a user may grant any combination and name a principal anything (`coder`, `code-checker`, `security-bot`). A principal _is_ the `AuthoritySet` it holds; nothing in enforcement keys off a name (UC-AUTHZ-02 AC, grep-asserted). The built-in `read/triage/write/maintain/admin` presets are GitHub-mirrored sugar that desugar to functional sets. **Out of scope this slice:** _user-defined_ named permission templates.

| ID          | Title                                                                          | Description                                                                                                                                                                                                                                                                                                                                               |
| ----------- | ------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| UC-AUTHZ-01 | Functional permission model + `.gitbutler/permissions.toml` + role desugar     | The GitHub-mirrored functional catalog is the authoritative config as a per-principal set; named roles are optional presets that desugar to functional sets at load (write excludes merge; admin = superuser); a raw functional list loads without a role.                                                                                                |
| UC-AUTHZ-02 | Enforce functional permissions on GitButler actions with agent-readable denial | Every consequential GitButler action checks the principal's functional permission at the `but-api` boundary — for **every** caller (Tauri / `but` CLI / TUI / N-API); a `contents:read`-only principal is denied review/merge with the structured `{code:"perm.denied", message, remediation_hint}` + exit 1; no enforcement path references a role name. |
| UC-AUTHZ-03 | Identity confinement & config-change authority                                 | A dispatched agent's `but` calls are pinned to its own handle (`BUT_AGENT_HANDLE`), so it cannot escalate by _becoming_ a more-privileged principal; and changing governed config (`.gitbutler/{permissions,gates}.toml`, `but perm`/`but group`) requires `administration:write`.                                                                        |
| UC-AUTHZ-04 | Fail-closed by default                                                         | An unknown principal, a missing `BUT_AGENT_HANDLE`, or unreadable/malformed governed config at the target ref **denies** the action — never defaults to allow; a `require_approval_from_group` naming an undefined group is not vacuously satisfied.                                                                                                      |

---

## UC-AUTHZ-01: Functional permission model + `.gitbutler/permissions.toml` + role desugar

GitButler mirrors GitHub's functional permission model, scoped to the git + governance core, and splits review/comment/merge out as first-class actions because each must be independently controllable. The authoritative stored config is a per-principal functional set in committed, ref-pinned `.gitbutler/permissions.toml`. A principal entry is **either** a named role preset **or** a raw functional list — both desugar to the same `AuthoritySet` at load, and enforcement only ever sees the set. The desugar is exact: `write` includes `contents:write`/`reviews:write`/`pull_requests:write` but **not** `merge` or `administration:write`; `maintain` adds `merge` + `administration:read` but **not** `administration:write` (so it can merge but cannot change governed config); `admin` is the superuser with every permission.

### Acceptance Criteria

☐ System parses a functional permission token (e.g. `contents:write`, `reviews:write`, `merge`, `administration:write`) into the typed `Authority` value, covering the MVP catalog
☐ System loads a principal's authoritative `AuthoritySet` from committed `.gitbutler/permissions.toml`, where an entry is either a role preset or a raw functional list
☐ System desugars the `write` role preset to a functional set containing `contents:write`, `reviews:write`, and `pull_requests:write` but excluding `merge` and `administration:write`
☐ System desugars the `admin` role preset to the superuser set containing every permission, including `merge` and `administration:write`
☐ System desugars the `maintain` role preset to `write` ∪ `{merge, administration:read}` — including `merge` but **excluding** `administration:write`, so a `maintain` principal can merge but cannot change governed config
☐ System loads a raw functional list (`permissions = ["contents:write", "reviews:write"]`) into the correct set without requiring a role, so a principal need not use a preset
☐ System resolves a `role = "write"` entry and the equivalent functional list to the same `AuthoritySet`, so enforcement only ever sees the set
☐ A User can seed an initial `.gitbutler/permissions.toml` so a freshly-onboarded repo's registered principals have functional permissions on day one (example bundles — illustrative, not fixed roles)
☐ System has a passing integration test against the real `but-authz` crate that desugars each role preset and parses a raw functional list, asserting the resulting `AuthoritySet` membership matches the catalog (write-excludes-merge, admin-is-superuser, list-loads-without-a-role)

---

## UC-AUTHZ-02: Enforce functional permissions on GitButler actions with agent-readable denial

A permission model is only real when every consequential action checks it. This use case wires the functional `authorize(principal, action)` check into every consequential GitButler action — commit (via the commit gate), open PR, review, comment, merge, and config change — so a principal lacking the needed permission is denied. The denial is the structured agent-readable contract: `{ error: { code: "perm.denied", message, remediation_hint } }` with exit code 1, naming the missing permission and the legitimate alternative, so the agent can adapt its next move. Authority is acquired at the `but-api` action wrapper, mirroring GitButler's existing acquire-permission-near-the-wrapper / `_with_perm` composition idiom (the new path is `_with_authz`, kept distinct from the repo-access lock). The seam is only effective for callers that route through `but-api`: Tauri, the `but` CLI, and the TUI do; **`but-napi` (N-API / Electron lite) must be audited** because it can call lower-level crates directly and skip the wrapper (R14). Critically, **no enforcement path keys off a role name**.

### Acceptance Criteria

☐ A principal holding only `contents:read`/`pull_requests:read` is denied submitting a review by the authorization check, because the action requires `reviews:write`
☐ A principal holding only `contents:read` is denied executing a merge, because the action requires `merge`
☐ The commit gate denies a commit by a principal lacking `contents:write`, consistent with the same functional check the API actions use
☐ System returns the structured denial `{ error: { code: "perm.denied", message, remediation_hint } }` with exit code 1 on a permission miss, rather than a silent skip or a crash
☐ System names the missing permission (e.g. `reviews:write`) in the denial message and the legitimate alternative in the `remediation_hint`, so the agent can adapt
☐ A principal holding `reviews:write` can submit a review but is still denied a commit it lacks `contents:write` for, so split actions are independently enforced
☐ System acquires authority at the `but-api` action wrapper (the `_with_authz` composition mirroring `_with_perm`), so every consequential action route is checked at one boundary rather than ad hoc
☐ Every governed caller — Tauri commands, the `but` CLI, the TUI, **and `but-napi` (N-API / Electron lite)** — routes through `but-api`'s gated action wrappers; a direct lower-level crate call by N-API is a governance-bypass regression (R14), so the build MUST audit every N-API entry point for a governance bypass (a build-gate grep-assert that each consequential N-API route is `_with_authz`-wrapped / pre-call-guarded, never a direct ungoverned lower-level call)
☐ System references no role name in any enforcement path — every action check tests an `Authority`, never a role string (grep-asserted)
☐ System has a passing integration test against the real `but-api` + real git that registers a read-only principal, attempts a review, and asserts exit code 1 with `error.code == "perm.denied"`, a `reviews:write` mention in the message, and a non-empty `remediation_hint`

---

## UC-AUTHZ-03: Identity confinement & config-change authority

The functional permission model is only sound if a principal cannot exceed its authority by two routes: (a) **becoming a different, more-privileged principal**, or (b) **changing the config that defines authority**. This use case closes both. First, a dispatched agent's `but` invocations are bound to its own handle — the orchestrator injects `BUT_AGENT_HANDLE=<self>`, and while dispatched, acting as another handle (a `--as <other>` style call) is denied, so an agent holding `{reviews:write}` cannot borrow a `merge`-holding identity. Second, changing governed config requires `administration:write`: `but perm grant/revoke`, `but group …`, and any change touching `.gitbutler/permissions.toml`/`.gitbutler/gates.toml` is authorized against it. Both checks key off the dispatched handle + the functional permission, never a role name.

### Acceptance Criteria

☐ The orchestrator binds a dispatched agent's session to its own principal by injecting `BUT_AGENT_HANDLE=<self>`, so every `but` action the agent runs acts as that handle
☐ While dispatched, a principal attempting to act as another handle (a `--as <other>` style call) is denied, so an agent cannot escalate by assuming a more-privileged identity
☐ System reads the acting principal's `AuthoritySet` from the committed config (never from an agent-supplied claim), so an agent cannot present its own elevated permission set
☐ `administration:write` is required to change governed config — `but perm grant/revoke`, `but group …`, and any change touching `.gitbutler/permissions.toml` or `.gitbutler/gates.toml` — a principal lacking it is denied with the `perm.denied` contract
☐ System authorizes a config-changing action against the `administration:write` permission, never a role name (grep-asserted, consistent with UC-AUTHZ-02)
☐ `but perm list --principal <other>` is denied with `perm.denied` when the calling principal is neither the named principal nor holds `administration:read`, so an agent cannot enumerate another principal's authority for reconnaissance
☐ System scopes identity confinement honestly: the in-band `--as <other>` call is denied, but a process re-exporting `BUT_AGENT_HANDLE` is **not** prevented (personal-tenant trusts the orchestrator) — the env-var residual is documented as an accepted leak, not claimed as confinement
☐ System has a passing integration test against the real `but-api`: an agent holding only `{reviews:write}` is denied (a) acting as another handle to reach a merge and (b) a config change to `.gitbutler/permissions.toml` that self-grants `merge` (lacks `administration:write`)
☐ A grant of `administration:write` to the acting principal is **inert until committed to the target ref** (the ref-pin contract): the config-change action itself still requires the caller to already hold `administration:write` at the target ref, so an admin cannot self-grant `administration:write` on a feature head and immediately exercise it for the same change

---

## UC-AUTHZ-04: Fail-closed by default

A governance engine whose authority source is a hand- and tool-edited config file is only safe if **the absence or corruption of an answer denies, never allows**. This use case makes default-deny explicit and testable: an action by a principal with no entry in `.gitbutler/permissions.toml` (an unknown principal) is denied; an action with no resolvable `BUT_AGENT_HANDLE` is denied (no anonymous action); and an action whose governed config at the target ref is **unreadable or malformed** (an unparseable `Authority` token, or a `require_approval_from_group` naming an undefined group) is denied rather than silently skipping the requirement. This is the property the e2e reference flow treats as a first-class failure — a fail-_open_ regression under empty or broken config would otherwise pass every positive-path test green.

### Acceptance Criteria

☐ The authorization check denies an action by a principal with no entry in `.gitbutler/permissions.toml` (an unknown principal) with the `perm.denied` contract, never defaulting to allow
☐ A `but` action invoked with no resolvable `BUT_AGENT_HANDLE` is rejected (no anonymous action), rather than running as an implicit or default principal
☐ The commit gate and merge gate fail closed (deny) when `.gitbutler/permissions.toml` or `.gitbutler/gates.toml` at the target ref is unreadable or malformed, returning a `{code:"config.invalid"}` contract (deterministic: `config.invalid` for malformed/unparseable config; `perm.denied` only for an unknown/missing principal) rather than skipping the check
☐ The merge gate denies when `require_approval_from_group` names a group not defined in the target-ref config, rather than treating the requirement as vacuously satisfied
☐ System has a passing integration test against real `but-authz` + real git that (a) attempts an action as an unknown principal (asserts denied), (b) commits a malformed `.gitbutler/gates.toml` to the target ref and asserts the gate denies rather than allows, and (c) asserts an action with no `BUT_AGENT_HANDLE` is rejected
