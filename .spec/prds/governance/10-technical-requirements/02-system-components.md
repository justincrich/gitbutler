---
stability: CONSTITUTION
last_validated: 2026-06-18
prd_version: 1.3.0
section: technical-requirements
---
# System Components

This initiative adds **one new crate** (`but-authz`) and **extends** a small number of existing crates at GitButler's action boundary. It deliberately reuses GitButler's existing composition idiom rather than inventing a parallel one. Crate roles below are grounded in the scanned crate map (`crates/AGENTS.md`, `but-action`/`but-rules` lib headers); where a precise function/module is not yet verified against source, it is marked **(confirm at planning)**.

## The enforcement seam is only sound if ALL FOUR callers route through it

`but-api` is the **single enforcement seam**, but it only governs the callers that actually go through it. GitButler has **four callers** of the shared API surface, and the seam is effective **only if all four route through `but-api`'s gated wrappers** (the `_with_authz` composition for local actions, the pre-call `authorize()` guard for async forge actions):

| Caller | Routes through `but-api`? | Governance note |
|---|---|---|
| **Tauri desktop** (`gitbutler-tauri`) | yes — Tauri commands wrap `but-api` functions | gated as long as the command calls the `_with_authz`-wrapped function |
| **`but` CLI** (`crates/but/src/args/` verbs) | yes — CLI verbs call `but-api` | gated; principal from `BUT_AGENT_HANDLE` |
| **TUI** (the `but` family TUI) | yes — shares the CLI's `but-api` path | gated identically to the CLI |
| **`but-napi` (N-API / Electron *lite*)** | **MUST be audited** — N-API may call `but-workspace`/`but-core` **directly**, skipping `but-api` | a direct lower-level call by N-API is an **ungoverned bypass** (R14) — the build MUST audit every N-API entry point for a governance bypass (build-gate grep-assert, T-AUTHZ-016b) and route each consequential route through `but-api` |

The seam's guarantee is therefore conditional: it gates the actions that flow through `but-api`. A consequential N-API route that reaches a lower-level crate directly is the **same accepted-leak class as the fence (R1)** until the audit routes it through the seam — see `07-technical-risks.md` R14 and UC-AUTHZ-02.

## NEW vs EXTENDED delta

| Crate | NEW / EXTENDED | What changes |
|---|---|---|
| `but-authz` | **NEW** | The whole POL engine: `Authority` (functional catalog enum), `AuthoritySet` (`parse` / `from_role` desugar / `union` / `contains`), `Principal`, `Group`, `authorize(principal, action) -> Result<(), Denial>`, the `Denial{code,message,remediation_hint}` contract, and the ref-pinned loader for `.gitbutler/permissions.toml` + group config. Pure, no I/O beyond reading committed config blobs handed to it. |
| `but-api` | **EXTENDED** | The enforcement seam — **two different shapes** (F-ENG-03, verified against source): (a) **local lock-taking** commit wrappers genuinely parallel `_with_perm` (acquire `Authority` near the wrapper, delegate inward); (b) **async forge actions** on `ThreadSafeContext` (review/comment/open-PR/merge) **cannot** take a repo-permission param (`but-api-macros/src/lib.rs:575` rejects it), so there `authorize()` is a plain **pre-call guard before the `.await`** with the same `Denial` contract — NOT the lock-mirroring idiom. **Must not overload the existing `Permission`** (a repo-access lock). The seam governs only callers that route through it (see "all four callers" note above — N-API must be audited, R14). |
| `but-action` | **EXTENDED (principal resolution only — NOT the sole gate)** | `but-action::on_uncommitted_changes` is *one* commit driver, not the choke point; it resolves the acting principal (`BUT_AGENT_HANDLE`) for the automation flow. The **commit gate itself** must sit at the narrow waist (below) — else `but-api::commit_create_only` (and `but-transaction`, `gitbutler-branch-actions`, the CLI) reach the same engine ungated. |
| `but-workspace` / `gitbutler-stack` | **EXTENDED (commit narrow-waist; NOT a local merge site)** | The **commit gate** chokepoint is `but_workspace::commit_engine::create_commit` (ultimately `but-core::commit::mod.rs:284` `repo.write_object`). There is **no local merge-to-trunk site** — the target ref is `ExtraRef::immutable` (`upstream_integration.rs:159`); landing is remote. |
| `but-api` `legacy/forge` | **EXTENDED (the merge gate's real home)** | The **merge gate** lives at the PR-merge action boundary (`legacy/forge.rs::merge_review` / `set_review_auto_merge` / `publish_review`), gating the *governed* merge action only — forge auto-merge / UI / raw push are accepted-leak (R1/R7). |
| `but` (CLI verbs in `crates/but/src/args/`) | **EXTENDED** | New CLI nouns: `but perm {list,grant,revoke}`, `but group {create,grant,add-member,remove-member,delete,list}`; principal-aware invocation (reads `BUT_AGENT_HANDLE`). The verb definitions live in `crates/but/src/args/` (the existing forge/branch/commit verb modules). **Not** `but-clap`: the `but-clap` crate is a CLI-**docs** generator (`crates/but-clap/src/main.rs` writes a `cli-docs/` dir from the clap command tree) — it does not define the verb surface. |
| `but-db` | **EXTENDED — 1 NEW table** | A NEW local table `local_review_verdicts(principal_id, verdict, head_oid, target, created_at)` so the merge gate can evaluate distinct-approval-@head. **Not** `but-forge-storage` (it holds only `forge_settings.json`) and **not** the `forge_reviews` table (a disposable remote-PR cache, `DELETE`d on sync, no principal/verdict). Follows the `workspace_rules`/`forge_reviews` migration pattern + `MIGRATIONS` registration. **Forgeability caveat (R6, High):** this store is not integrity-protected — a direct DB write forges an approval; the merge gate must be tested through the governed review-submission path, not by direct INSERT. |

**No change** to `gitbutler-*` legacy crates beyond what the action boundary requires; per `crates/AGENTS.md`, new logic stays in `but-*` and avoids new `gitbutler-*`/`VirtualBranchesHandle` usage.

## The naming-collision guardrail (load-bearing)

GitButler already has a `Permission` concept and a `_with_perm` composition — but it is a **repository-access lock** (the right to exclusively touch the worktree; a concurrency primitive), *not* principal authorization. This initiative introduces an **orthogonal** axis (*may this principal do this action*). The guardrail, asserted in review:

| Axis | Existing (do NOT overload) | This initiative (NEW) |
|---|---|---|
| Type | `Permission` (repo-access lock) | `Authority` / `AuthoritySet` |
| Module path | `but-api` / `but-ctx` | `but-authz` (NEW) — types live at `but_authz::Authority` / `but_authz::AuthoritySet` |
| Wrapper | `_with_perm` | `_with_authz` |
| Question | "do I hold the repo lock right now?" | "may this principal perform this action?" |
| Macro axis | `but-api-macros` `PermissionParam` = repo-lock (unchanged) | authorization handled by a pre-call guard for async actions — NOT a macro param |

Mirror the *composition shape* (acquire near the wrapper, delegate inward); never the *type*. **Do NOT** extend `but-api-macros` to accept an authority parameter on `ThreadSafeContext` — the macro explicitly rejects repo-permission params there (`but-api-macros/src/lib.rs:575`); async forge actions use a pre-call `authorize()` guard instead.

## Per-crate module map

### `but-authz` (NEW — AUTHZ, GRPS)
| File | Change |
|---|---|
| `src/authority.rs` *(NEW)* | `Authority` enum (functional catalog), `AuthoritySet` (`parse`, `from_role` desugar, `union`, `contains`) |
| `src/principal.rs` *(NEW)* | `Principal` (id/handle), `Group` (name + grants + members), effective-set resolution (own ∪ groups) |
| `src/config.rs` *(NEW)* | ref-pinned loader for `.gitbutler/permissions.toml` + group config (read at a given ref, never the working tree) |
| `src/authorize.rs` *(NEW)* | `authorize(principal, action) -> Result<(), Denial>`; `Denial{code,message,remediation_hint}` |
| `src/lib.rs` *(NEW)* | crate surface; re-exports |

### `but-api` (EXTENDED — enforcement seam, TWO shapes)
| File | Change |
|---|---|
| commit wrappers (`commit/create.rs` etc.) — local lock-taking | acquire `Authority` near the wrapper (genuine `_with_perm` parallel), delegate inward; `Denial` on a miss |
| `legacy/forge.rs` (`publish_review` / `merge_review` / `set_review_auto_merge`) — async on `ThreadSafeContext` | a **pre-call `authorize()` guard** before the `.await` (NOT a repo-permission param — `but-api-macros:575` rejects those here); same `Denial` contract |
| N-API entry-point audit (R14) | the build MUST grep-audit every `but-napi` consequential entry point and assert it routes through the `_with_authz`-wrapped / pre-call-guarded `but-api` action — no direct lower-level call (T-AUTHZ-016b) |

### `but-action` (EXTENDED — principal resolution for the automation flow, NOT the gate)
| File | Change |
|---|---|
| `on_uncommitted_changes` (`src/lib.rs:46`) | resolve principal from `BUT_AGENT_HANDLE` for the watcher/agent commit flow — this is one commit caller (the principal-resolution site for the automation flow), NOT the commit gate |

### `but-workspace` (EXTENDED — the commit-gate chokepoint)
| File | Change |
|---|---|
| `commit_engine::create_commit` (`mod.rs:123`) — the narrow waist all commit callers funnel through (→ `but-core::commit::mod.rs:284` `write_object`) | the **commit gate**: `contents:write` + branch protection (target-ref `.gitbutler/gates.toml`); resolve principal; fail closed. (Alternative: wrap ALL `but-api` commit wrappers — see seam note.) |
| *(no local merge site)* | target ref is `ExtraRef::immutable` (`upstream_integration.rs:159`); the **merge gate is in `but-api` `legacy/forge`**, not here |

### `but` CLI verbs (`crates/but/src/args/`) (EXTENDED — AUTHZ, GRPS CLI)
| File | Change |
|---|---|
| `crates/but/src/args/` verb modules (alongside the existing `forge.rs`/`branch.rs`/`commit.rs` etc.) | `perm` noun (`list/grant/revoke`), `group` noun (`create/grant/add-member/remove-member/delete/list`); read `BUT_AGENT_HANDLE` for principal-aware invocation. The verb surface is defined here — **not** in `but-clap` (a docs generator). Governed PR/review verbs (`but pr`/`but review`) extend the existing `forge::pr` surface — see `04-api-design.md` New CLI verbs and UC-LOOP-01. |

## Committed config + the new local state
| Path | Change |
|---|---|
| `.gitbutler/permissions.toml` *(NEW)* | per-principal functional set (role OR functional list) + `[[group]]` entries + membership; ref-pinned |
| `.gitbutler/gates.toml` *(NEW)* | per-branch protection + review requirement (`min_approvals`, `require_distinct_from_author`, `require_approval_from_group`); ref-pinned |
| `local_review_verdicts` table *(NEW — `but-db`)* | principal_id, verdict, `head_oid`, target, created_at — the merge gate's approval source; a **NEW `but-db` table** (not `but-forge-storage`, not the disposable `forge_reviews` cache). See `07-technical-risks.md` R6 (High) for its forgeability/tamperability caveat — a direct DB write forges an approval (accepted-leak class). |

## Component reuse summary
| Capability | Reused existing GitButler component | New code |
|---|---|---|
| Permissioned action composition | the `_with_perm` acquire-near-wrapper idiom (`crates/AGENTS.md`) | a parallel `_with_authz` over `but-authz::authorize` |
| Evaluated-policy precedent (shape only) | `but-rules` — note it is **DB-backed** (`but_db::WorkspaceRule`, a SQLite row), NOT committed config | `.gitbutler/*.toml` committed config is a **NEW convention** (inspired by prior art, not reusing it); GitButler has no committed-config precedent — the only precedent is committed-blob *reads* (`but-core::commit::mod.rs:383` parses TOML from a blob). Confirm `.gitbutler/` namespace with maintainers. |
| Commit creation | `but_workspace::commit_engine::create_commit` (the narrow waist) | the commit-gate check at that chokepoint (NOT `but-action`, one caller) |
| Merge / land | **no local site** — the `but-api` `legacy/forge` PR-merge action | the merge-gate check at the governed action; forge/UI/push bypass = accepted-leak (R1/R7) |
| CLI surface | the `but` verb modules in `crates/but/src/args/` (the docs are generated by `but-clap`) | the `perm` + `group` nouns; the governed `but pr`/`but review` extensions |
