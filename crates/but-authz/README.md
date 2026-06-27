# but-authz

Functional authorization primitives for governed GitButler actions.

`but-authz` answers two questions for every governed `but` invocation: **who is
acting**, and **are they allowed**. It resolves an acting principal from the
`BUT_AGENT_HANDLE` environment variable, loads committed governance config, and
authorizes functional authorities (`contents:write`, `reviews:write`, …) against
branch protection and review gates.

## Threat model

> **Honest, scoped threat model.** Read this before relying on `but-authz` for
> any security decision.

Agent identity in `but-authz` is **environment-primary**, not cryptographic. The
acting principal is resolved from the `BUT_AGENT_HANDLE` environment variable and
matched against the committed `.gitbutler/agents.toml` catalog. The handle is a
**plaintext string** — there is no signature, no token, no shared secret. Its
integrity comes entirely from **who sets it**: the trusted **harness wrapper**
(the git→but steerer) assigns each agent its handle; the agent does **not**
self-assert it.

**Explicit forgeability caveat.** A plaintext handle is only as strong as the
harness that sets it. Any actor that can set `BUT_AGENT_HANDLE` in a governed
`but` process's environment can present that identity. The trust root is
therefore **the host OS plus the harness wrapper that sets the env var** — the
same trust class the prior runtime registry already conceded (spoofing it
collapsed to "write the registry file you already have filesystem access to").
The improvement is not a stronger identity primitive: it is that identity now
tracks the *real* execution model (one-shot `but` child processes, many
subagents multiplexed into one host process) instead of a process registry the
gate could never resolve.

**Explicitly OUT OF SCOPE** for this crate:

- **Cross-host non-repudiation** — identity is local to one host; there is no
  protocol for asserting an agent identity across machines.
- **Cryptographic signatures** — no signing keys, no signed attestations, no
  signature verification path. Identity is not cryptographically anchored.
- **Keychain / secret storage** — no credentials are stored in or read from an
  OS keychain. There are no shared secrets.
- **Sandboxing** — the engine does not confine, jail, or otherwise restrict the
  process it identifies. It trusts the host OS and the harness to set
  `BUT_AGENT_HANDLE` truthfully.

A **sealed (signed) token** that would let the engine verify the handle
independently of the harness is noted as a possible follow-on — it is **not
built**.

## Per-harness identity mechanism

The handle is un-forgeable only to the degree the harness wrapper controls the
child environment. The steerer uses the strongest mechanism each harness allows:

| Harness | Mechanism | Property |
|---|---|---|
| **OpenCode** | `shell.env` plugin hook **injects** `BUT_AGENT_HANDLE` into each subagent's shell. | **Host-set, un-forgeable** — the agent never controls the value; the host writes it. |
| **Claude Code** / **Codex** | PreToolUse **match-enforcement**: their hooks cannot mutate the child env, so the steerer **denies** any governed `but` whose handle ≠ the harness-assigned `agent_type`. | Forgery is detected and **blocked at the boundary** rather than prevented at the source. |

Both paths anchor the same trust root (host + harness wrapper); they differ only
in whether the handle is *injected* or *match-enforced*.

A **PID ancestry walk** was considered and rejected as an alternative: OpenCode
and Claude Code multiplex many subagents into **one** host process, so sibling
agents are indistinguishable by process lineage.

## File layout

Governance state lives in committed, ref-pinned config — there is no runtime
state file.

### Committed: `agents.toml` (ref-pinned)

`.gitbutler/agents.toml` is the **committed, ref-pinned** principal catalog.
It defines `[[agent]]` blocks (each with `id`, `permissions`, optional `role`,
and `groups`) and `[[group]]` blocks. The engine reads it at the **target ref
through `gix`** — never the working tree — preserving the ref-pin (anti-self-
escalation) contract. A companion `.gitbutler/gates.toml` carries
`[[branch]]` protection records.

```toml
# .gitbutler/agents.toml — committed, ref-pinned
[[agent]]
id = "rust-implementer"
permissions = ["contents:write"]
groups = ["implementers"]

[[group]]
name = "implementers"
permissions = ["contents:read"]
members = ["rust-implementer", "rust-reviewer"]
```

The Rust domain types (`Principal`, `PrincipalId`) are unchanged by the rename;
only the wire format and filename moved from `[[principal]]` /
`permissions.toml` to `[[agent]]` / `agents.toml`.

A ref is considered **governed** once it commits at least one of
`agents.toml`, `permissions.toml` (legacy), or `gates.toml` into its tree
(`governance_present`).

## Migration path

`permissions.toml` → `agents.toml` ships with a one-release legacy-fallback
window so existing governed repos keep authorizing unchanged.

1. **Legacy fallback during the window.** `load_governance_config` prefers
   `agents.toml` when both exist. When only `permissions.toml` is present it
   reads that and emits a one-line deprecation warning:
   `warning: .gitbutler/permissions.toml is deprecated; run: but agent migrate`.
   `governance_present` returns true if **either** file is committed at the
   target ref, so no repo loses governance status mid-migration.

2. **`but agent migrate`.** Rewrites the working-tree `permissions.toml` into
   `agents.toml` (`[[principal]]` → `[[agent]]`, byte-equivalent round-trip of
   the same `GovConfig`) and prints the ref-pin caveat (the new file is inert
   until committed — same pattern as `perm_grant`). The operator commits the
   add and delete together. The command is **idempotent**: a second run when
   `agents.toml` exists is a no-op (exit 0, no file change).

The `but agent` CLI surface is intentionally small: `but agent list --committed`
prints the committed `[[agent]]` roster at the target ref, and `but agent
migrate` performs the rename. There are no runtime registration verbs — identity
is set in the environment by the harness, not registered in-engine.

## Resolution order

Gates resolve the acting principal through `resolve_principal_from_env(&GovConfig)`
— the **production resolver**. Resolution is strict and fail-closed:

1. **Read `BUT_AGENT_HANDLE`.** If the variable is unset or empty, deny with
   `Denial::no_handle()` (`perm.denied`, `class = operator_required`).
2. **Resolve against committed config.** Look the handle up in the committed
   `agents.toml` at the target ref. An unknown handle (no matching `[[agent]]`)
   denies with `Denial::unknown_principal(handle)` (`perm.denied`,
   `class = operator_required`).
3. **Resolve the principal.** On a hit, the handle resolves to a `Principal`
   carrying its authorities and group memberships, which `authorize()` then
   checks against the requested authority.

The same denial code (`perm.denied`) is reused for `no_handle` and
`unknown_principal`; both carry `class = operator_required` and a do-not-retry
hint, because no principal was resolved and the actor cannot self-correct
in-system (the operator/harness must set a valid handle).

### Worked example

The trusted harness wrapper assigns each agent's handle; the agent's first
governed call resolves it:

```bash
# The harness wrapper sets BUT_AGENT_HANDLE for the subagent it spawned
# (OpenCode: shell.env injection; Claude Code/Codex: match-enforced).
# The agent's governed call then resolves through the env:
#   resolve_principal_from_env(cfg)
#     -> env BUT_AGENT_HANDLE = "rust-implementer"
#     -> principal_from_handle("rust-implementer", cfg) -> Principal
but commit -m "..."
```

With no handle set (and no harness to set it), the same governed call resolves
no principal and is denied:

```
code:             perm.denied
class:            operator_required
message:          BUT_AGENT_HANDLE is required to resolve a governed principal
remediation:      set BUT_AGENT_HANDLE to a principal committed in governance config
```

## Denial codes

| Code | Carrier | `class` | Meaning |
|------|---------|---------|---------|
| `perm.denied` | missing authority | `actor_correctable` | A resolved principal lacks the required authority. The actor can self-correct (request a reviewed merge, use a different verb). |
| `perm.denied` | unresolved principal | `operator_required` | No principal resolved (`no_handle`, `unknown_principal`). The operator/harness must set a valid `BUT_AGENT_HANDLE`. |
| `branch.protected` | branch protection | `actor_correctable` | The target ref is branch-protected. |
| `gate.review_required` | review gate | `actor_correctable` | The review requirement is unmet. |
| `config.invalid` | config load | `operator_required` | The committed `.gitbutler` config is malformed, incomplete, or unreadable. |

The `DenialCause` → `DenialClass` mapping is an **exhaustive `match` with no
wildcard arm**, so adding a new cause without updating the classification is a
compile error — a new denial cause can never silently default to
`actor_correctable`. This non-defaulted match is the security property that
keeps operator-required denials from being miscategorized as self-recoverable.

## Public API surface

Resolution & authorization:

- `resolve_principal_from_env(&GovConfig) -> Result<Principal, Denial>` — the
  production gate resolver (reads `BUT_AGENT_HANDLE`).
- `resolve_principal(lookup, &GovConfig) -> Result<Principal, Denial>` — the
  injected-lookup variant tests use to avoid mutating process environment.
- `authorize(&Principal, Authority, &GovConfig) -> Result<(), Denial>`
- `effective_authority(&Principal, &GovConfig) -> AuthoritySet`

Config:

- `load_governance_config(&gix::Repository, target_ref) -> Result<GovConfig, ConfigError>`
- `governance_present(&gix::Repository, target_ref) -> Result<bool>`
- `agents_path()` / `permissions_path()` — single source of truth for the
  `.gitbutler/*.toml` literals.

## Superseded: the PID registry

IDENT first shipped a runtime **PID registry** — `but agent register` mapping
`(pid, start_time) → agent_id`, with the gates resolving the *current* pid via
`resolve_principal_with_registry`. Dogfooding exposed a structural flaw: every
agent runs `but` as a **one-shot child process** (`cd … && but commit`), so the
pid the gate saw was an ephemeral grandchild that was never registered — the
registry resolved nothing and agents fell back to the env handle for 100% of
governed operations. The registry was **reverted** and `BUT_AGENT_HANDLE` made
the primary identifier again, now **set by the trusted harness wrapper** rather
than self-asserted by the agent. The `registry.rs` / `process.rs` modules, the
`agents-runtime.toml` file, the `(pid, start_time)` tuple, the
`but agent register/unregister/whoami` verbs, and the `BUT_AUTHZ_ALLOW_ENV_HANDLE`
flag are all gone.

For the full reversal rationale see
[`.spec/prds/governance/12-uc-agent-identity.md`](../../.spec/prds/governance/12-uc-agent-identity.md)
and the **"Identity: why env-primary"** section of
[`.spec/README.md`](../../.spec/README.md).
