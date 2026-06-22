---
stability: CONSTITUTION
last_validated: 2026-06-18
prd_version: 1.3.0
section: technical-requirements
---

# API Design

The surface is **CLI-first** (`but` nouns, defined in `crates/but/src/args/`) over a **core authorization API** (`but-authz`) wired into GitButler's action boundary via the `_with_authz` composition. No new daemon, no new HTTP, no MCP. (`but-clap` is the CLI-**docs** generator, not the verb surface — see the New CLI verbs note below.)

## The core authorization API (`but-authz`)

```rust
// but-authz — the single enforcement primitive
pub fn authorize(
    principal: &Principal,        // resolved from BUT_AGENT_HANDLE
    action: Authority,            // the functional permission the route requires
) -> Result<(), Denial>;          // Ok(()) to proceed; Err(Denial) → exit 1

// effective set = own grants ∪ every group's grants, all read at the TARGET ref
pub fn effective_authority(principal: &Principal, cfg: &GovConfig) -> AuthoritySet;

pub struct Denial {              // serialized as { error: { code, message, remediation_hint } }
    pub code: &'static str,      // "perm.denied"
    pub message: String,         // names the missing Authority
    pub remediation_hint: String // names the legitimate alternative
}
```

## The enforcement seam (`but-api`) — TWO shapes, FOUR callers

The seam governs **only the callers that route through `but-api`**. There are four — **Tauri desktop, the `but` CLI, the TUI, and `but-napi` (N-API / Electron lite)** — and the seam is effective only if all four go through the gated wrappers. The first three do; **`but-napi` must be audited** because it may call `but-workspace`/`but-core` directly and skip the wrapper (a direct lower-level call is an ungoverned bypass, R14 — closed by the T-AUTHZ-016b build-gate grep-audit and routing every consequential N-API route through `but-api`). See `02-system-components.md` "all four callers" note.

**Lock discipline (RULES.md):** authorization is evaluated **BEFORE** the `RepoExclusive` worktree guard is taken — the pattern is `authorize()` → `with_authz()` → acquire guard → run impl. For async actions on `ThreadSafeContext` that cannot take a repo-permission param, `authorize()` is a pre-call guard before the `.await`. This satisfies the RULES.md rule: _don't call permission-acquiring helpers while holding a guard_.

The seam differs by action kind (verified against source — `but-api-macros:560` rejects repo-permission params on `ThreadSafeContext`):

**(a) Local lock-taking actions (commit)** — genuinely parallel `_with_perm` (acquire near the wrapper, delegate inward):

```rust
pub fn commit(ctx, args) -> Result<Commit, Error> {
    let principal = principal_from_env(ctx)?;                 // BUT_AGENT_HANDLE
    with_authz(&principal, Authority::ContentsWrite, || {     // ← authorization (+ branch protection)
        commit_with_authz(ctx, args)                          // ← existing impl, unchanged
    })
}
```

**(b) Async forge actions (review / comment / open-PR / merge)** on `ThreadSafeContext` — a plain **pre-call `authorize()` guard** before the `.await` (NOT a repo-permission param; same `Denial` contract):

```rust
pub async fn submit_review(ctx: ThreadSafeContext, args) -> Result<Review, Error> {
    let principal = principal_from_env(&ctx)?;
    authorize(&principal, Authority::ReviewsWrite)?;          // ← pre-call guard
    submit_review_impl(ctx, args).await                       // ← existing async forge call
}
```

| Action route                                                                                                 | Required `Authority`                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| ------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| open PR (`but pr new`)                                                                                       | `pull_requests:write`                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| close PR (`but pr close` — governed verb to add)                                                             | `pull_requests:write`                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| submit review — approve / request-changes (`but review approve` / `request-changes` — governed verbs to add) | `reviews:write`                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| comment (`but review comment` / `but pr comment` — governed verb to add)                                     | `comments:write`                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| commit (commit gate)                                                                                         | `contents:write` (+ branch protection) — checked even under DryRun: a DryRun denial still returns the `Denial` contract + exit 1 (authorization is real enforcement, read-only; DryRun only suppresses persisting refs/objects/oplog). A dry-run reports the **would-be** outcome — a denied dry-run tells the caller the real action _would_ be denied; the caller always knows its own `--dry-run` flag, so a dry-run denial and a real denial are never ambiguous |
| merge (merge gate)                                                                                           | `merge` (+ review requirement)                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| `but perm` / `but group` / edit governed config                                                              | `administration:write`                                                                                                                                                                                                                                                                                                                                                                                                                                               |

These PR/review routes are the surface the LOOP demo's "open a PR" / "submit a review" steps gate on (UC-LOOP-01). `pull_requests:write`, `reviews:write`, and `comments:write` already appear in this route→Authority table; UC-LOOP-01 makes that surface **explicit** (make-explicit, not net-new scope — the route table already implied it and `but pr` already partly ships; see Version History).

## The two gate entry points

| Gate            | Entry                                                                                                                                                                                         | Checks                                                                                                                                                                         | On fail                                                                                                  |
| --------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------- |
| **Commit gate** | the commit narrow-waist (`but_workspace::commit_engine::create_commit`) — covers virtual-branch, normal-git, AND worktree (`but-worktrees::integrate`) + `but-workspace::branch::apply` paths | `authorize(p, contents:write)` → branch protection from target-ref `.gitbutler/gates.toml`; fail closed on unknown principal / unreadable config                               | `{code:"perm.denied"}`, `{code:"branch.protected", branch}`, or `{code:"config.invalid"}`, exit 1        |
| **Merge gate**  | the `but-api` PR-merge action (`legacy/forge::merge_review` / `set_review_auto_merge`) — the trunk ref is immutable locally                                                                   | `authorize(p, merge)` → review requirement @head from target-ref `.gitbutler/gates.toml` (min approvals, distinct-from-author, required groups); binds the governed merge only | `{code:"perm.denied"}`, `{code:"gate.review_required", unmet:[…]}`, or `{code:"config.invalid"}`, exit 1 |

The merge gate's review requirement reads the local `local_review_verdicts` store, which is **not integrity-protected** (R6, High): a direct DB write forges an approval (accepted-leak class, same as R1). The gate is satisfied only by reviews submitted through the governed `but review` action; the LOOP demo assumes honest review submission.

## Tauri command surface for the MGMT renderer

The governance management UI (UC-MGMT-01..07) invokes Tauri commands that wrap the **same `but-api` functions** the CLI verbs use. These commands follow GitButler's existing Tauri naming convention (snake*case `#[tauri::command]`) and are surfaced through the generated `packages/but-sdk` after `pnpm build:sdk && pnpm format`. Governance commands inherit GitButler's existing `contextIsolation`/CSP model — no raw eval; errors serialize via `but_api::json::Error` (the same contract as other Tauri commands). This is the **admin-config surface** — the commands an admin runs to *manage* governance config (perm/group/gates/status). Agent-facing *enforcement* actions (`merge`, `pull_requests:write`, `reviews:write`) are CLI/action-boundary actions (UC-AUTHZ-02), **not** UI management commands, and are intentionally absent here. `governance_status_read` is a UI-specific **self-scoped** read (the viewer's own effective set) for read-only display — distinct from the enforcement route→Authority table above. `tauri-implementer` adds the matching `allow-perm*\_`/`allow-group\_\_`/`allow-branch*gates*\*`capability/permission entries (in the desktop capability file under`src-tauri/capabilities/`, scoped to the main window) per GitButler's existing capability convention. Tauri command names below are snake_case renderer-callable wrappers; the **Invokes** column maps each to its kebab-case `but`CLI verb — so a UC that references`but perm grant`resolves to the`perm_grant`command (and the`perm_list`/`perm_grant`/… row below).

| Tauri command            | Invokes (`but-api` / CLI)                   | Authority required              | Notes                                                                                               |
| ------------------------ | ------------------------------------------- | ------------------------------- | --------------------------------------------------------------------------------------------------- |
| `perm_list`              | `but perm list`                             | `administration:read` (or self) | `--principal <id>` optional; omitted → caller's effective set                                       |
| `perm_grant`             | `but perm grant`                            | `administration:write`          | adds functional permission(s) to a principal's `.gitbutler/permissions.toml` entry                  |
| `perm_revoke`            | `but perm revoke`                           | `administration:write`          | removes functional permission(s)                                                                    |
| `group_create`           | `but group create`                          | `administration:write`          | defines a group with a functional set                                                               |
| `group_grant`            | `but group grant`                           | `administration:write`          | grants functional permission(s) to a group                                                          |
| `group_add_member`       | `but group add-member`                      | `administration:write`          | add a principal to a group (ref-pinned; inert until committed)                                      |
| `group_remove_member`    | `but group remove-member`                   | `administration:write`          | remove a principal from a group (ref-pinned; inert until committed)                                 |
| `group_delete`           | `but group delete`                          | `administration:write`          | delete a group; principals lose its inherited grants on the next target-ref read (B11 / UC-MGMT-03) |
| `group_list`             | `but group list`                            | `administration:read`           | groups, grants, membership                                                                          |
| `branch_gates_read`      | gate-config read (`.gitbutler/gates.toml`)  | `administration:read`           | branch protection rules for the target ref                                                          |
| `branch_gates_update`    | gate-config write (`.gitbutler/gates.toml`) | `administration:write`          | updates gate fields (protected, min_approvals, distinct_from_author, required_groups)               |
| `governance_status_read` | `but-authz` `effective_authority`           | self (own principal)            | caller's effective `AuthoritySet` (own ∪ groups) for read-only display                              |

Every write command is admin-gated server-side by `but-authz`; the renderer's `adminOnly`/disabled-controls (UC-MGMT-01) are UX convenience only — a renderer that bypassed its own guard would still hit the server gate.

## New CLI verbs

The verbs below are defined in `crates/but/src/args/` (alongside the existing `forge.rs` / `branch.rs` / `commit.rs` verb modules) — **not** in `but-clap`. The `but-clap` crate (`crates/but-clap/src/main.rs`) **generates CLI documentation** (it walks the clap command tree and writes a `cli-docs/` dir); the actual CLI verb definitions live in `crates/but/src/args/mod.rs` and the per-noun modules it declares.

### Governance admin nouns (NEW)

| Command                                           | Purpose                                                                                                         | Gated by                        |
| ------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- | ------------------------------- |
| `but perm list [--principal <id>]`                | show a principal's effective `AuthoritySet`                                                                     | `administration:read` (or self) |
| `but perm grant --principal <id> <authority>…`    | add functional permission(s); writes `.gitbutler/permissions.toml` (effective once committed to the target ref) | `administration:write`          |
| `but perm revoke --principal <id> <authority>…`   | remove functional permission(s)                                                                                 | `administration:write`          |
| `but group create <name> [--permissions …]`       | define a group with a functional set                                                                            | `administration:write`          |
| `but group grant <name> <authority>…`             | grant functional permission(s) to a group                                                                       | `administration:write`          |
| `but group add-member <name> --principal <id>`    | add a principal to a group                                                                                      | `administration:write`          |
| `but group remove-member <name> --principal <id>` | remove a principal from a group                                                                                 | `administration:write`          |
| `but group delete <name>`                         | delete a group; affected principals lose its inherited grants on the next target-ref read (B11 / UC-MGMT-03)    | `administration:write`          |
| `but group list`                                  | show groups, grants, membership                                                                                 | `administration:read`           |

### Governed PR / review verbs (extend the EXISTING `but pr` / `but review` surface — A1.12, R-NEW-3)

The forge CLI surface already ships at `crates/but/src/args/forge.rs`: the top-level verb is `Pr(forge::pr::Platform)` with `visible_alias = "review"` (and `"mr"`), so `but pr` / `but review` / `but mr` are the same command. Its existing subcommands are `new` (open a PR / review — `create_review`), `auto-merge`, `set-draft`, `set-ready`, and `template`. The governance layer **permission-checks each governed action and adds the genuinely-missing governed verbs under this same heading** — it does **not** introduce parallel `create`/`close` verbs that duplicate `but pr new`.

| Command                      | Status                                                       | Purpose                                                                | Gated by                       |
| ---------------------------- | ------------------------------------------------------------ | ---------------------------------------------------------------------- | ------------------------------ |
| `but pr new`                 | **exists** (`forge::pr::Subcommands::New`)                   | open a PR / review for a branch                                        | `pull_requests:write`          |
| `but pr auto-merge`          | **exists** (`forge::pr::Subcommands::AutoMerge`)             | enable/disable the governed auto-merge — passes through the merge gate | `merge` (+ review requirement) |
| `but pr close`               | **add** (new `forge::pr` subcommand)                         | close an open PR/review without merging                                | `pull_requests:write`          |
| `but review approve`         | **add** (new `forge::pr` subcommand, via the `review` alias) | submit an approving review — recorded in `local_review_verdicts` @head | `reviews:write`                |
| `but review request-changes` | **add** (new `forge::pr` subcommand)                         | submit a request-changes review                                        | `reviews:write`                |
| `but review comment`         | **add** (new `forge::pr` subcommand)                         | submit a comment                                                       | `comments:write`               |

All write commands print the ref-pin caveat: _"takes effect once committed to the target branch."_ The added `close`/`approve`/`request-changes`/`comment` verbs sit under the existing `but pr`/`but review` heading, **distinct** from the `but perm`/`but group` admin surface; each is permission-checked at the `but-api` boundary per the route→Authority table above.

## The agent-readable rejection contract (used everywhere)

Every denial — permission, branch protection, gate — is the same shape and exit code, so an agent can parse one contract and adapt:

```json
{
	"error": {
		"code": "perm.denied",
		"message": "action requires reviews:write; principal 'rust-implementer' holds {contents:write, pull_requests:write}",
		"remediation_hint": "ask an administrator to grant reviews:write, or hand this action to a principal that holds it"
	}
}
```

`code` ∈ `{ perm.denied, branch.protected, gate.review_required, config.invalid }`; exit code `1` on every rejection. No partial success, no silent skip; a missing/unreadable/malformed governed config fails closed with `config.invalid` (never an implicit allow). **Denial-code meaning:** `config.invalid` = a system/config error requiring operator action; `perm.denied` / `branch.protected` / `gate.review_required` = user-correctable failures (permission missing, path blocked, requirement unmet).

## Identity & confinement

- The acting principal is resolved from `BUT_AGENT_HANDLE` (orchestrator-injected per dispatch).
- The `AuthoritySet` is loaded from committed config at the target ref — **never** from an agent-supplied claim.
- While dispatched, acting as another handle (`--as <other>`) is denied (UC-AUTHZ-03).
- **The desktop/admin path (the human fleet-owner):** for v1, a MGMT config-management command (Tauri `perm_*` / `group_*` / `branch_gates_*`) resolves its acting principal from the signed-in desktop user (`UserService` / forge session) — the human fleet-owner, a **trusted superuser over the agent fleet** (personal-tenant trust: the human at the keyboard owns the repo and its agents). This is the _trust root_ for config management, the same trust class as the orchestrator, and is **distinct from the agent authorization model** — agents stay bound by functional permissions with no superuser path; the "UI is never a bypass" invariant means an _agent_ cannot circumvent its permissions via the UI (the human owner managing their own fleet is the intended use, not a bypass). Accepted v1 risk (R12); future: a real per-human authenticated principal checked against `permissions.toml`.
