---
stability: CONSTITUTION
last_validated: 2026-06-18
prd_version: 1.3.0
section: technical-requirements
---

# Data Schema

**The authoritative governance state is committed config, not a database** — a deliberate choice (committed, ref-pinned, governed, reviewable as a diff; committed `.gitbutler/*.toml` is a new GitButler convention). The only persistent _local_ state added is a small **review record** (a NEW `but-db` table) so the merge gate can evaluate distinct-approval-@head without a remote forge. Net: **2 committed config files + 1 new `but-db` table (`local_review_verdicts`) + the new `but-authz` Rust types.**

## New Rust types (`but-authz`)

| Type                     | Shape                                                                                                                                                                                                                                    | Notes                                                                                                                                     |
| ------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| `Authority` _(NEW enum)_ | the functional catalog: `MetadataRead`, `ContentsRead`, `ContentsWrite`, `PullRequestsRead`, `PullRequestsWrite`, `ReviewsWrite`, `CommentsWrite`, `Merge`, `StatusesRead`, `StatusesWrite`, `AdministrationRead`, `AdministrationWrite` | parsed from tokens like `contents:write`; the catalog the MVP enforces                                                                    |
| `AuthoritySet` _(NEW)_   | `parse(list)` · `from_role(name)` (desugar) · `union(other)` · `contains(authority)`                                                                                                                                                     | enforcement only ever sees this; `from_role` is pure load-time sugar                                                                      |
| `Principal` _(NEW)_      | `id`/`handle`, optional direct `AuthoritySet`, `groups: Vec<GroupName>`                                                                                                                                                                  | resolved from `BUT_AGENT_HANDLE`; effective set = direct ∪ groups; register-on-first-grant, decommission-by-emptiness (see `02-roles.md`) |
| `Group` _(NEW)_          | `name`, `AuthoritySet` (grants), `members: Vec<PrincipalId>`                                                                                                                                                                             | the net-new grantee; union semantics; deletable via `but group delete`                                                                    |
| `Denial` _(NEW)_         | `{ code: "perm.denied", message, remediation_hint }`                                                                                                                                                                                     | the agent-readable rejection contract, exit 1                                                                                             |

### Role desugar (exact, asserted by test)

| Preset     | Expands to                                                                                                                                     |
| ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| `read`     | `{metadata:read, contents:read, pull_requests:read}`                                                                                           |
| `triage`   | `read` + `{statuses:read}` (+ issue/PR triage where applicable)                                                                                |
| `write`    | `read` + `{contents:write, pull_requests:write, reviews:write, comments:write, statuses:write}` — **excludes** `merge`, `administration:write` |
| `maintain` | `write` + `{merge, administration:read}`                                                                                                       |
| `admin`    | **superuser** — every `Authority`, incl. `merge` + `administration:write`                                                                      |

## Committed config files (governed, ref-pinned)

### `.gitbutler/permissions.toml` _(NEW)_

Per-principal functional set (role preset OR raw functional list) + group definitions + membership. A row is desugared to an `AuthoritySet` at load; enforcement only ever sees the set. Read at the **target ref** when authorizing.

```toml
# --- groups: functional permissions granted to a team ---
[[group]]
name = "code-reviewers"            # AI code-level tier
permissions = ["reviews:write", "comments:write", "contents:read"]

[[group]]
name = "maintainers"               # human feature-level tier
permissions = ["merge", "reviews:write", "administration:write"]

# --- principals: users or agents ---
[[principal]]
id = "rust-implementer"            # agent — no merge, no review
permissions = ["contents:write", "pull_requests:write"]

[[principal]]
id = "rust-reviewer"               # agent — inherits reviews:write via group
groups = ["code-reviewers"]

[[principal]]
id = "justin"                      # human — final feature-level authority
groups = ["maintainers"]

[[principal]]
id = "release-bot"
role = "maintain"                  # desugars to write + merge + administration:read
```

### `.gitbutler/gates.toml` _(NEW)_

Per-target-branch protection + review requirement, read at the **target ref** so a change cannot weaken its own gate.

```toml
[[branch]]
name = "main"
protected = true                   # commit gate: no direct commits — land via merge

[[gate]]
branch = "main"
type = "review"
min_approvals = 1
require_distinct_from_author = true
require_approval_from_group = ["code-reviewers", "maintainers"]   # AI AND human

# No break_glass / force / skip_gate field exists in this slice (deferred).
```

## The one local record — review state

The merge gate must evaluate "≥1 approving review, distinct from author, from each required group, **at the current head**." That needs local review state (the POC is local — no remote forge round-trip).

| Field          | Purpose                                                                                                                          |
| -------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| `id`           | record id                                                                                                                        |
| `target`       | the PR / stack / branch under review                                                                                             |
| `principal_id` | who reviewed (for distinct-from-author + group membership)                                                                       |
| `verdict`      | `approved` / `commented`                                                                                                         |
| `head_oid`     | the head the review was recorded against — **load-bearing**: an approval at an old head does not satisfy after the head advances |
| `created_at`   | ordering                                                                                                                         |

**Home: a NEW `but-db` table `local_review_verdicts` — NOT `but-forge-storage` (it holds only `forge_settings.json`) and NOT the `forge_reviews` table (a disposable remote-PR cache, cleared on sync via the runtime `DELETE FROM forge_reviews` at `crates/but-db/src/table/forge_reviews.rs:153` — not just the `:40` migration — and carrying no principal/verdict). Follows the `workspace_rules`/`forge_reviews` migration pattern.** This is the single piece of new persistent state; everything else is committed config or in-memory resolution.

> **Integrity caveat (R6, High — accepted-leak).** `local_review_verdicts` is **not integrity-protected**: an agent with DB/filesystem write access can INSERT a forged approving row (`principal_id`/`verdict`/`head_oid`), trivially satisfying the merge gate by direct DB write — the **same accepted-leak class as the fence (R1)**. The merge gate is sound only for reviews submitted through the governed `but review` action; the gate tests exercise that governed path, not the forgeable direct write. **Deferred hardening:** an HMAC integrity check keyed by a repo-local admin secret (C3), then Ed25519-signed review artifacts. The schema must NOT be presented as tamper-proof. See `07-technical-risks.md` R6.

## What is explicitly NOT a schema change

- **No new permission _table_** — permissions/groups live in committed `.gitbutler/permissions.toml` (ref-pinned), not the DB. This is deliberate (governed, ref-pinnable, reviewable as a diff).
- **No change to GitButler's `Permission` (repo-access lock) type** — the new `Authority` axis is orthogonal (see `02-system-components.md`).
- **No agent-runtime / turn / session tables** — out of scope (Assumption 2).
