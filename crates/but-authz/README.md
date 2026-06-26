# but-authz

Functional authorization primitives for governed GitButler actions.

`but-authz` answers two questions for every governed `but` invocation: **who is
acting**, and **are they allowed**. It resolves an acting principal from a
runtime process registry, loads committed governance config, and authorizes
functional authorities (`contents:write`, `reviews:write`, …) against branch
protection and review gates.

## Threat model

> **Honest, scoped threat model.** Read this before relying on `but-authz` for
> any security decision.

Agent identity in `but-authz` is **process-level**, not cryptographic. The
acting principal is bound to the operating-system process via the tuple
`(pid, start_time)` — the `start_time` value is Unix seconds read from:

- **Linux** — `/proc/[pid]/stat` field 22 (`starttime`, in clock ticks after
  boot), converted with `sysconf(_SC_CLK_TCK)` and the system `btime`.
- **macOS** — `proc_pidinfo(pid, PROC_PIDTBSDINFO, …)` returning
  `proc_bsdinfo.pbi_start_tvsec`.

The `(pid, start_time)` tuple is a cheap **PID-reuse defense**: a recycled PID
acquires a new start time, so a stale registration whose start time no longer
matches the observed process is refused as `Denial::stale_registration`. It
does **not** make any spoof-prevention or cryptographic-identity claim.

**Explicitly OUT OF SCOPE** for this crate:

- **Cross-host non-repudiation** — identity is local to one host; there is no
  protocol for asserting an agent identity across machines.
- **Cryptographic signatures** — no signing keys, no signed attestations, no
  signature verification path. Identity is not cryptographically anchored.
- **Keychain / secret storage** — no credentials are stored in or read from an
  OS keychain. There are no shared secrets.
- **Sandboxing** — the engine does not confine, jail, or otherwise restrict the
  process it identifies. It trusts the host OS to report `pid` and
  `start_time` truthfully.

The trust root is the **host OS plus the orchestrator that writes the runtime
registry file**. Spoofing collapses to "write to the registry file you already
have filesystem access to" — the same trust root as the legacy
`permissions.toml` model. The improvement over the legacy model is **not** a
stronger identity primitive: it is that every governed call now **requires a
registry hit** instead of trusting a caller-set environment variable. An
unregistered process resolves no principal and is denied with
`perm.denied`.

## File layout

Governance state is split across two files with different lifecycles.

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

### Runtime: `agents-runtime.toml` (per-host, mode 0600)

The runtime registry maps `(pid, start_time, expiry) → agent_id` and is written
by `but agent register`. It is **per-host process state**, never committed.

**Resolution order** (`runtime_registry_location`, the single source of truth
shared by the CLI writer and the gate reader; first match wins):

1. **`BUT_AGENT_REGISTRY_PATH`** — explicit override. Wins over every
   host-derived default; its parent directory is assumed to already exist (the
   operator owns the path). Used for tests and sandboxed environments.
2. **`$XDG_RUNTIME_DIR/gitbutler/<repo-hash>/agents-runtime.toml`** — when
   `XDG_RUNTIME_DIR` is set (the normal Linux case). This is typically tmpfs and
   cleared on reboot. `<repo-hash>` is the lowercase hex of the first 16 bytes
   of the SHA-256 of the canonicalized git directory, keeping per-host
   registries isolated.
3. **`<workdir>/.gitbutler/agents-runtime.toml`** — the **worktree fallback**,
   used when `XDG_RUNTIME_DIR` is unset (the normal **macOS** case). This file
   lives **inside the working tree** and **persists there** — it is not on tmpfs
   and is not cleared on reboot.

A bare repository with no override and no `XDG_RUNTIME_DIR` has no default
registry location.

**Guarantees the code enforces:**

- **Mode `0600`, owned by the user.** The file is created owner-only: the write
  path sets `0600` on the locked temp file before the atomic rename
  (`Registry::write_inner`), so the mode survives commit instead of inheriting
  the process umask. The registry is the spoofing trust root, so on the worktree
  fallback path — where it sits in the working tree — owner-only matters.
- **Gitignored on the worktree fallback path.** Because the fallback file lives
  inside the working tree, `but agent register` (its creator) ensures
  `<workdir>/.gitbutler/.gitignore` contains a line `agents-runtime.toml`
  (`ensure_runtime_registry_gitignored`, idempotent — created if absent, the
  rule appended only when missing). This keeps the file one `git add` short of
  being committed. The override and XDG paths live outside the working tree and
  are not gitignored.
- **Atomic write.** Written through a Git-style lock file (`gix::lock` with
  `flush` + `sync_all` + atomic rename). Expired entries (`expires_at < now`)
  are garbage-collected lazily on read (`Registry::gc`).

```toml
# agents-runtime.toml — per-host runtime state, mode 0600
[[registration]]
pid = 4711
start_time = 1719417600
agent_id = "rust-implementer"
registered_at = 1719417600
expires_at = 1719432000
registered_by = "but-cli"
```

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

## Environment variable deprecation

`BUT_AGENT_HANDLE` — the legacy self-asserted identity string — is now
**test/CI-only**. It is consulted **only** when `BUT_AUTHZ_ALLOW_ENV_HANDLE=1`
is also set, and it is **slated for deprecation** once the runtime registry is
the sole production identity path. Production governed calls must not set
either variable; orchestrators call `but agent register` after spawning each
subagent instead.

## Resolution order

Gates resolve the acting principal through
`resolve_principal_with_registry(Option<&Registry>, &GovConfig)`. Resolution is
strict and fail-closed:

1. **Registry hit.** If the runtime registry has an entry for the current
   `(pid, start_time)`, that entry's `agent_id` resolves to the principal. A
   same-PID entry whose `start_time` differs from the observed value is denied
   as `Denial::stale_registration` (PID-reuse defense) — it never resolves a
   principal.
2. **Flag-gated env fallback.** On a registry miss, if
   `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` is set, fall back to the `BUT_AGENT_HANDLE`
   environment variable via `resolve_principal`. This is the test/CI escape
   hatch, not a production identity path.
3. **`Denial::unregistered`.** Otherwise deny with `Denial::unregistered`
   (`perm.denied`, `class = operator_required`). `reg = None` is treated as a
   registry miss (so callers without a registry still get a deterministic
   resolution attempt through the flag-gated fallback, then denial).

The same denial code (`perm.denied`) is reused for `no_handle`,
`unknown_principal`, and `unregistered_start_time_unresolved`; all carry
`class = operator_required` and a do-not-retry hint, because no principal was
resolved and the actor cannot self-correct in-system.

### Worked example

An orchestrator spawns a subagent and binds its identity before any governed
verb runs:

```bash
# 1. Register the child process under an agent committed in agents.toml.
but agent register --pid $CHILD_PID --as rust-implementer

# 2. The child's first governed call resolves through the registry.
#    Internally: current_pid() -> $CHILD_PID
#                process_start_time($CHILD_PID) -> 1719417600
#                registry.resolve(($CHILD_PID, 1719417600)) -> "rust-implementer"
#                principal_from_handle("rust-implementer", cfg) -> Principal
BUT_AUTHZ_ALLOW_ENV_HANDLE=1 but agent whoami   # sanity check (test-only flag)
```

Without `but agent register`, the same governed call resolves no principal and
is denied:

```
code:             perm.denied
class:            operator_required
message:          unregistered process pid 4711 start_time 1719417600; no governed principal resolved
do_not:           register the principal / set BUT_AGENT_HANDLE; do not retry as-is
```

## Denial codes

| Code | Carrier | `class` | Meaning |
|------|---------|---------|---------|
| `perm.denied` | missing authority | `actor_correctable` | A resolved principal lacks the required authority. The actor can self-correct (request a reviewed merge, use a different verb). |
| `perm.denied` | unresolved principal | `operator_required` | No principal resolved (`no_handle`, `unknown_principal`, `unregistered`, `stale_registration`). An operator must register the process/principal. |
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

- `resolve_principal_with_registry(Option<&Registry>, &GovConfig) -> Result<Principal, Denial>`
- `resolve_principal(lookup, &GovConfig) -> Result<Principal, Denial>` (env-driven, test-only)
- `authorize(&Principal, Authority, &GovConfig) -> Result<(), Denial>`
- `effective_authority(&Principal, &GovConfig) -> AuthoritySet`

Config & registry:

- `load_governance_config(&gix::Repository, target_ref) -> Result<GovConfig, ConfigError>`
- `governance_present(&gix::Repository, target_ref) -> Result<bool>`
- `agents_path()` / `permissions_path()` — single source of truth for the
  `.gitbutler/*.toml` literals.
- `Registry::{load, write, register, unregister, resolve, gc}`

Process identity:

- `current_pid() -> u32`
- `process_start_time(pid) -> Result<u64>` (Linux `/proc/[pid]/stat`, macOS
  `proc_pidinfo`; bails on unsupported platforms)
