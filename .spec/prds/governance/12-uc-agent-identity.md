---
stability: FEATURE_SPEC
last_validated: 2026-06-24
prd_version: 1.4.0
functional_group: IDENT
---
# Use Cases: Agent Identity Registration (IDENT)

> **⚠️ SUPERSEDED IN PART (env-primary identity).** UC-IDENT-02 (runtime PID registry),
> UC-IDENT-03 (registry resolution at the gates), and UC-IDENT-04 (`but agent
> register`/`unregister`/`whoami` CLI) describe a **runtime PID registry** that was built,
> dogfooded on `agent-intel`, and then **reverted**. The registry could not govern the real
> execution model: every agent runs `but` as a one-shot child process whose pid is never the
> registered pid, so registration was inert and the env hatch carried 100% of governed
> operations. Identity is now **environment-primary** — `BUT_AGENT_HANDLE` resolved via
> `resolve_principal_from_env` against committed `agents.toml`, **set by the trusted harness
> wrapper** (OpenCode `shell.env` injection; Claude Code/Codex PreToolUse match-enforcement),
> not self-asserted by the agent. See [`.spec/README.md`](../../README.md) → *Identity: why
> env-primary* for the full decision record + evidence, and `crates/but-authz/README.md` for
> the current model. **UC-IDENT-01** (`agents.toml` rename) and **UC-IDENT-05** (skills stop
> self-asserting the handle) survive the reversal and are satisfied by harness injection. The
> UCs below are preserved as the historical record of what was tried.

Today an agent's identity is a self-asserted string in `BUT_AGENT_HANDLE` — a caller-controlled env var the engine trusts verbatim. This initiative replaces that string with a **runtime PID registry** whose identifiers are anchored in committed `.gitbutler/agents.toml`. Every governed `but` invocation must be attributable to a registered agent identifier, and the engine must refuse unregistered callers. Identity is process-level (`(pid, start_time)`), not cryptographic; the trust root is the host OS + the orchestrator that writes the registry file. `permissions.toml` is renamed to `agents.toml` (the `[[principal]]` block becomes `[[agent]]`); the runtime registry is a sibling file (`agents-runtime.toml`, gitignored) mapping `(pid, start_time, expiry) → agent_id`.

> **Honest threat model.** Spoofing collapses to "write to the registry file you already have fs access to" — same trust root as `permissions.toml` today, but now every governed call **requires a registry hit** instead of trusting a caller-set env var. Cross-host non-repudiation, cryptographic signatures, keychain storage, and sandboxing are explicitly **out of scope** this slice.

| ID | Title | Description |
|----|-------|-------------|
| UC-IDENT-01 | `agents.toml` replaces `permissions.toml` | A committed, ref-pinned `.gitbutler/agents.toml` defines `[[agent]]` blocks (id + permissions + groups); the engine reads it at the target ref. `permissions.toml` is read as a legacy fallback during a one-release migration window; `but agent migrate` rewrites the working tree. |
| UC-IDENT-02 | Runtime PID registry | A runtime, gitignored `.gitbutler/agents-runtime.toml` (or `$XDG_RUNTIME_DIR/gitbutler/<repo-hash>/agents-runtime.toml`) maps `(pid, start_time, expiry) → agent_id`; written by `but agent register`, GC'd on read; mode 0600. |
| UC-IDENT-03 | Enforced resolution at every gate | The 8 gate callsites in `but-api` resolve the acting principal via `resolve_principal_with_registry`: registry hit → principal; registry miss + `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` → env fallback; else → `Denial::unregistered`. |
| UC-IDENT-04 | `but agent` CLI surface | `but agent register / unregister / list / list --committed / whoami / migrate` verbs mirroring `but perm` / `but group` shape. Registration validates `agent_id` exists in committed `agents.toml` (fail-fast). |
| UC-IDENT-05 | Skill + doc migration | The `but-*` and `kb-*` skills (in brain) stop self-asserting `BUT_AGENT_HANDLE`; orchestrators call `but agent register --pid <child> --as <agent>` after spawning each subagent. `but-init` writes `agents.toml`; `but-migrate` performs the rename. Docs in this repo document the new identity model. |

---

## UC-IDENT-01: `agents.toml` replaces `permissions.toml`
The static, committed, ref-pinned principal catalog is renamed: `[[principal]]` → `[[agent]]`, file `permissions.toml` → `agents.toml`. The Rust domain types `Principal` / `PrincipalId` stay (they are internal domain nouns); only the wire format and file name change — this minimizes churn across the 80+ existing tests that use `PrincipalId::new(...)`. Wire types in `crates/but-authz/src/config.rs` gain `AgentWire`/`AgentsWire` alongside the legacy `PrincipalWire`/`PermissionsWire`. For one release, `load_governance_config` reads **both** files if both exist (prefer `agents.toml`, log a one-line deprecation warning when only `permissions.toml` is present). `governance_present` returns true if EITHER file is committed at the target ref.

### Acceptance Criteria
☐ System parses an `agents.toml` committed at the target ref into the same `GovConfig` shape as the legacy `permissions.toml` (rename `[[principal]]` → `[[agent]]`, field set unchanged)
☐ System reads `agents.toml` at the target ref via `gix` blob read — never the working tree — preserving the ref-pin contract from UC-AUTHZ-03
☐ `governance_present` returns true if either `agents.toml` OR `permissions.toml` is present at the target ref (migration window — both formats recognized)
☐ `load_governance_config`, when both files are present at the target ref, prefers `agents.toml` and emits a one-line warning naming `permissions.toml` as deprecated + the `but agent migrate` remediation
☐ `but agent migrate` reads working-tree `permissions.toml`, writes `agents.toml` with `[[agent]]` blocks (byte-equivalent round-trip of the same `GovConfig`), and prints the ref-pin caveat (inert until committed — same pattern as `perm_grant`); the operator commits add + delete together
☐ `but agent migrate` is idempotent: a second run when `agents.toml` exists is a no-op (exit 0, no file change)
☐ A `permissions.toml`-only repo continues to authorize governed actions unchanged during the migration window (no behavior change until the operator runs `but agent migrate`)
☐ System has a passing integration test against real `but-authz` + real git that parses both file formats into the same `GovConfig`, asserts byte-equivalent round-trip via `but agent migrate`, and confirms `governance_present` recognizes either file

---

## UC-IDENT-02: Runtime PID registry  · **SUPERSEDED (reverted; see header note)**
A runtime registry file maps `(pid, start_time) → (agent_id, expiry, registered_at, registered_by)`. Default location `$XDG_RUNTIME_DIR/gitbutler/<repo-hash>/agents-runtime.toml` (tmpfs on Linux, dies on reboot); override via `BUT_AGENT_REGISTRY_PATH` for tests/sandboxing. Mode 0600, owned by the user. `start_time` is unix seconds from `/proc/[pid]/stat` field 22 (Linux) or `libproc` `proc_pidinfo(PROC_PIDTBSDINFO)` (macOS) — cheap PID-reuse defense: a recycled PID with a new start_time is rejected as stale. Entries expire (default TTL 4h, overridable per-registration); expired entries are GC'd lazily on read.

### Acceptance Criteria
☐ `Registry::load(path)` parses the runtime file; `Registry::write(path)` persists it atomically (write-to-temp + rename); the file is always parseable after a write (no half-written state on crash)
☐ `Registry::register(pid, start_time, agent_id, ttl, registered_by)` adds an entry; `Registry::unregister(pid, start_time)` removes it; both round-trip via load→write→load
☐ `Registry::resolve(pid, start_time)` returns `Some(agent_id)` on a fresh hit; returns `None` on (a) missing entry, (b) PID-reuse mismatch (same pid, different start_time), or (c) expired entry (`now > expires_at`)
☐ `Registry::gc(now)` drops entries where `now > expires_at`; a GC'd entry is not resolvable
☐ `but_authz::process::current_pid()` returns `std::process::id()`; `process::process_start_time(pid)` returns unix seconds and is monotonic non-decreasing for the calling process across two reads
☐ On macOS, `process_start_time` reads via `libproc` (`proc_pidinfo` with `PROC_PIDTBSDINFO`); on Linux, via `/proc/[pid]/stat` field 22 — no third-party dep beyond `libc`
☐ System has a passing unit test that exercises register/unregister round-trips, TTL expiry via an injected clock, PID-reuse rejection (same pid, different start_time → stale), and concurrent writes (last-writer-wins, file always parseable)

---

## UC-IDENT-03: Enforced resolution at every gate  · **SUPERSEDED (env-primary; see header note)**
The 8 gate callsites in `but-api` (`commit/gate.rs:72`, `legacy/merge_gate.rs:114`, `legacy/governance.rs:{347,378,448,750}`, `legacy/forge.rs:58`, `legacy/config_mutate.rs:23`) switch from `resolve_principal_from_env(&cfg)` to `resolve_principal_with_registry(reg, &cfg)`. Resolution order: (1) registry hit → principal; (2) registry miss + `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` → env fallback; (3) else → `Denial::unregistered(pid)` (code = `perm.denied`, consistent with `no_handle`). A stale registration (start_time mismatch) yields `Denial::stale_registration(pid, start)`. The registry is loaded once per gate invocation (cheap — small file, in-memory map). Load path resolves via `BUT_AGENT_REGISTRY_PATH` → `$XDG_RUNTIME_DIR/gitbutler/<repo-hash>/agents-runtime.toml` → absence (fall through to env/denial).

### Acceptance Criteria
☐ `resolve_principal_with_registry(Some(reg), cfg)` returns the registered principal on a fresh registry hit (the engine trusts the registry over any env var)
☐ `resolve_principal_with_registry(None, cfg)` with `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` set falls through to the legacy `resolve_principal_from_env` path (test/CI escape hatch)
☐ `resolve_principal_with_registry(None, cfg)` with the flag UNSET on a governed repo returns `Denial::unregistered(pid)` with `code = perm.denied` (consistent with `no_handle`)
☐ A stale registration (same pid, different start_time) returns `Denial::stale_registration(pid, start)` with `code = perm.denied`
☐ Each of the 8 gate callsites in `but-api` calls `resolve_principal_with_registry`; no callsite reads `BUT_AGENT_HANDLE` directly except inside `authorize.rs` (grep-asserted build-gate)
☐ A governed action attempted by an unregistered process (registry miss + flag unset) is denied at every gate — commit, merge, admin-write, forge review — with `perm.denied`
☐ System has a passing integration test against real `but-api` + real git: register a `(pid, start_time) → rust-implementer`, run a commit, assert success; unregister, run another commit, assert `perm.denied` with `Denial::unregistered`

---

## UC-IDENT-04: `but agent` CLI surface  · **SUPERSEDED (register/unregister/whoami removed; see header note)**
A new `but agent` noun mirrors the `but perm` / `but group` shape (`crates/but/src/command/perm.rs` is the template). Verbs: `register`, `unregister`, `list`, `list --committed`, `whoami`, `migrate`. Registration validates that `agent_id` exists in committed `agents.toml` (fail-fast: `but agent register --as ghost` exits 1 immediately, not later at gate time). The orchestrator is allowed to register arbitrary PIDs (its children); a malicious caller could register a PID it doesn't own, but it could also just register its own PID and act — no additional risk.

### Acceptance Criteria
☐ `but agent register --pid <pid> --as <agent_id> [--ttl <duration>] [--by <caller>]` writes a registration to the runtime file, prints the resolved `(pid, start_time, agent_id, expires_at)` tuple, exits 0
☐ `but agent register --as <unknown_agent_id>` exits 1 with a message naming the missing id (the agent must exist in committed `agents.toml` before it can be registered)
☐ `but agent register` to an unwritable registry path exits 2 with a message naming the path
☐ `but agent unregister --pid <pid>` removes the registration (idempotent — unregistering an unknown pid exits 0)
☐ `but agent list` prints live registrations from the runtime file; `but agent list --committed` prints committed `[[agent]]` blocks from `agents.toml` at the target ref
☐ `but agent whoami` resolves THIS process's registration (looks up its own pid + start_time) and prints the agent_id, or exits 1 with `Denial::unregistered` if not registered
☐ `but agent migrate` performs the `permissions.toml` → `agents.toml` rewrite (UC-IDENT-01 AC-5/6)
☐ The `but agent` subcommand is wired into `Subcommands::Agent(args::agent::Platform { cmd })` in `crates/but/src/lib.rs` and dispatched in the same shape as `perm`/`group`
☐ System has CLI snapshot tests in `crates/but/tests/but/command/agent.rs` modeled on `commit_gate.rs` / `merge_gate.rs`: register+list+commit happy path; unknown agent_id denied; `whoami` round-trip; `migrate` produces byte-equivalent `agents.toml`

---

## UC-IDENT-05: Skill + documentation migration
The `but-*` and `kb-*` skills in `~/Projects/brain/skills/` stop self-asserting `BUT_AGENT_HANDLE` for governed repos. Orchestrators (`but-run-sprint`, `but-orchestrate`) call `but agent register --pid <child_pid> --as <assigned_agent>` immediately after spawning each implementer/reviewer subagent; subagents no longer `export BUT_AGENT_HANDLE=...`. `but-init` writes `agents.toml` (not `permissions.toml`). `but-migrate` performs the rename for existing repos. Docs in this repo (RULES.md, crates/but-authz/README.md, etc.) document the new identity model. (Skills are tracked in the brain repo; this UC covers the contract this repo expects of them.)

### Acceptance Criteria
☐ `RULES.md` adds an "Agent identity" subsection: governed repos require `but agent register` before any gate; `BUT_AGENT_HANDLE` is test-only (gated by `BUT_AUTHZ_ALLOW_ENV_HANDLE=1`)
☐ `crates/but-authz/README.md` (NEW) documents the threat model, file layout, migration path, and the env-var deprecation timeline
☐ `crates/AGENTS.md` cross-references `crates/but-authz/README.md` for the identity model
☐ `DEVELOPMENT.md` "Code Hitlist" adds the `permissions.toml` → `agents.toml` rename as a tracked migration
☐ The `but-init` skill writes `.gitbutler/agents.toml` (not `permissions.toml`) and registers each specialist via `but agent register` after governance commits (verified by re-running the skill against a fresh fixture repo)
☐ The `but-migrate` skill detects `.gitbutler/permissions.toml`, runs `but agent migrate`, commits the rename (idempotent — re-run is a no-op once `agents.toml` exists)
☐ The `but-run-sprint` skill drops `export BUT_AGENT_HANDLE=...` from implementer/reviewer dispatch templates and instead calls `but agent register --pid <child_pid> --as <assigned_agent>` after spawning each subagent; a single-task end-to-end sprint passes with zero `BUT_AGENT_HANDLE` references in dispatch templates
