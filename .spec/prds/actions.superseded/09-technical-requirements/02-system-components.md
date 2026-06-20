---
stability: CONSTITUTION
last_validated: 2026-06-19
prd_version: 1.0.1
section: technical-requirements
---
# System Components

This initiative adds **one new crate** (`but-checks`), **one new `but-db` table** (`check_results`), and **extends** a small number of existing surfaces at GitButler's merge boundary. It deliberately reuses the governance machinery already in source — `but-authz`, `enforce_merge_gate`, the ref-pinned `.gitbutler/*.toml` config convention, the `but-db` table + `MIGRATIONS` pattern — rather than inventing parallels. Crate roles below are grounded in the scanned crate map; where a precise function/module is not yet verified against source, it is marked **(confirm at planning)**.

## Naming disambiguation — `but-checks` ≠ the existing `butler_actions` (DECISION B, load-bearing)

GitButler **already has an "Actions" feature**, and it is a name collision trap for this design. The pre-existing feature is the **`butler_actions` macro/automation table** (`crates/but-db/src/table/butler_actions.rs` *(confirm at planning)*) surfaced through the **`actions::Platform`** noun — an agent-automation / macro-recording concept, unrelated to merge-gating validations. This initiative is a **checks / validations** system: it produces signed, gate-consumable check results. To avoid colliding with `butler_actions`:

| This initiative (checks/validations) | Pre-existing GitButler feature (do NOT conflate) |
|---|---|
| Crate **`but-checks`** (the producer + verifier) | the `actions::Platform` macro/automation surface |
| Table **`check_results`** (`but-db`) | the **`butler_actions`** macro table (`but-db`) |
| CLI noun **`but check …`** | (n/a — different feature) |
| Concept: "did the required *check* pass at this head OID?" | concept: "recorded/automated *action* macro" |

The committed config path `.gitbutler/actions/*.toml` is retained because it is the natural directory name for check definitions and is governance-adjacent config (confirm at planning whether to rename to `.gitbutler/checks/*.toml` to fully avoid the "actions" word); **the crate, table, and CLI noun are `but-checks` / `check_results` / `but check` and must not be named `but-actions` / any `*actions*` identifier that collides with `butler_actions`.** Reviewer asserts: no new symbol named `but-actions`/`but_actions` is introduced.

## NEW vs EXTENDED delta

| Component | NEW / EXTENDED | What changes | Real target |
|---|---|---|---|
| `but-checks` | **NEW crate** | the whole producer side: the check-result contract types (`CheckName`, `HeadOid`, `Conclusion`, `CheckResult`, `ProducerIdentity`, `Signature`), the **default executor** (resolve a ref-pinned check definition → run it → derive a `Conclusion` → sign → record), the signing/verification primitive, the ref-pinned `.gitbutler/actions/*.toml` loader (mirroring `but_authz::load_governance_config`), and the **required-checks verifier** the gate calls (`required_checks_satisfied(...) -> Result<(), CheckGateError>`). Pure + deterministic except for (a) reading committed config blobs via `gix`, (b) reading/writing the ledger via `but-db`, (c) running the check subprocess, (d) reading the signing key via `but-secret`. | new dir `crates/but-checks/` |
| `but-db` `check_results` table | **EXTENDED — 1 NEW table** | a NEW local table `check_results(id, target, name, head_oid, conclusion, producer_identity, signature, recorded_at, metadata)` — the **agent-unwritable, signed** ledger the gate consumes. Follows the `local_review_verdicts`/`ci_checks` migration pattern (`M::up(...)` + `MIGRATIONS` registration at `crates/but-db/src/lib.rs:130`). **Corrects** `local_review_verdicts`' forgeability (governance R6) by carrying `producer_identity` + `signature`; corrects `ci_checks`' disposability (it is a sync-overwritten cache) by being a **durable, append-only-per-(name, head_oid)** ledger. | new file `crates/but-db/src/table/check_results.rs`; register in `crates/but-db/src/lib.rs` |
| `merge_gate.rs` required-checks clause | **EXTENDED — the read-only consumer** | `enforce_merge_gate` (`crates/but-api/src/legacy/merge_gate.rs:40`) gains a **third clause**: after the `Merge` authorization (`:48`) and composed with the review requirement (`:86`), call `but_checks::required_checks_satisfied(...)` against ledger rows at the **current source head OID** (`current_head_oid`, `:78`). The clause is **read-only** — it consumes ledger rows; it never invokes the executor (DECISION A). On a miss, return a `MergeGateError`-shaped payload with the NEW code `gate.check_required`. | `crates/but-api/src/legacy/merge_gate.rs` (edit `enforce_merge_gate`); reuse the existing `MergeGateError` struct (`:20`) shape, add the `check_required` code |
| `but-api` recording entry point | **EXTENDED — the producer's API boundary** | a NEW `but-api` function (e.g. `legacy/checks.rs::run_check`) that the CLI/Tauri call to invoke the executor; it is **gated by `statuses:write`** via the same pre-call `authorize()` guard pattern that `publish_review` uses (`forge.rs:488` `authorize_branch_action(&repo, &params.source_branch, Authority::PullRequestsWrite)`; the helper is defined at `forge.rs:47`). The agent-facing *recording* is deterministic engine code inside `but-checks`; the *trigger* is `statuses:write`-gated so only a producer-authorized principal can ask the executor to run-and-record. No public API records a caller-supplied `Conclusion` (R6). | new file `crates/but-api/src/legacy/checks.rs`; register in `crates/but-api/src/legacy/mod.rs` |
| `but` CLI verbs (`but check …`) | **EXTENDED** | new noun `but check {define, list, run, results, required}` — define a check, list definitions, run a check via the executor, read recorded results, show required-checks status for a branch. Verb impl in `crates/but/src/command/` (alongside `command/perm.rs`); clap definitions in `crates/but/src/args/` (alongside `args/perm.rs`). **Not** `but-clap` (a CLI-docs generator). | new files `crates/but/src/command/check.rs` + `crates/but/src/args/check.rs`; wire in `command/mod.rs` + `args/mod.rs` |
| `but-secret` (signing key) | **REUSED — no change** | the butler producer signing key is stored/read via the existing `but-secret` keyring stack (`crates/but-secret/`, `keyring.workspace = true`). No new secret crate. **Residual:** in the personal-tenant model the keyring is readable by the agent's own OS user (R2). | `crates/but-secret/` (existing API) |
| `but-rebase` graph editor | **REUSED — read-only (SHA-reset basis)** | the SHA-reset invariant keys on the current head OID; when history is rewritten, `but_rebase::graph_rebase::Editor::rebase()` produces new OIDs and `Editor::commit_mappings()` (`mod.rs:479`) exposes old→new. Checks does **not** modify the editor; it relies on the OID change so stale results stop matching. An *optional* LEDGER housekeeping step may consult `commit_mappings()` to prune superseded rows (optimization, not correctness). | `crates/but-rebase/src/graph_rebase/mod.rs` (read-only dependency) |

**No change** to `gitbutler-*` legacy crates beyond what the merge boundary requires; per `crates/AGENTS.md`, new logic stays in `but-*` and avoids new `gitbutler-*`/`VirtualBranchesHandle` usage.

## The clean-workspace-checkout hidden cost (under-specified — planning risk)

The executor must run each check against **the code at the exact current head OID**. In a conventional one-branch repo that is trivial (the worktree *is* the head). In GitButler's **virtual-branches-over-ONE-worktree** model it is **not** trivial and is currently **under-specified**: the single shared worktree is where the gated agent is actively editing, may carry uncommitted changes, and reflects a *workspace* projection of several virtual branches — not a clean checkout of one branch's head OID.

Open questions the planner MUST resolve (flagged as a hidden cost, not yet designed):

- How does the executor obtain an **isolated checkout at the head OID** without disturbing the shared worktree the agent is using? (candidate: a throwaway `gix` worktree / temp checkout from the object DB at the OID; candidate: `git worktree add --detach <oid>` at a temp path; candidate: a bare-object check that needs no worktree for purely-git checks.)
- What is the **cost/latency** of materializing that checkout on every `on-merge-attempt` run, and does it interact with the shared index/locks (`BUT_WS_LOCK_DEBUG`)?
- Does running the check mutate or contend on the shared worktree at all (it MUST NOT — a check run cannot disturb the agent's working state)?

This is named here and carried as a planning risk in `07-technical-risks.md` (R13). The build MUST NOT assume "just run the command in the repo dir" — that would run against the agent's live, possibly-dirty worktree rather than the head OID the result is signed against, silently breaking the head-OID binding.

## The `ci_checks`-is-NOT-the-ledger guardrail (load-bearing)

GitButler already has a `ci_checks` table (`crates/but-db/src/table/ci_checks.rs`) — and it is a **trap** for this design. It looks right (`name`, `head_sha`, `status_conclusion` mirror GitHub's check shape) but is the wrong home, for the same reason governance could not reuse `forge_reviews`:

| Axis | `ci_checks` (do NOT reuse) | `check_results` (NEW) |
|---|---|---|
| Source of truth | a **remote-forge cache** — populated by `list_ci_checks` (`forge.rs:450`) from GitHub's API | **butler-local truth** — produced by the trusted default executor |
| Durability | **disposable** — `set_for_reference` does `DELETE FROM ci_checks WHERE reference = ?1` then re-INSERT on every sync (`ci_checks.rs:151`) | **durable** — a result at `(name, head_oid)` is the record of a real run, not overwritten by a sync |
| Producer binding | **none** — no `producer_identity`, no `signature`; it is whatever the forge reported | **signed** — `producer_identity` + `signature` over `(name, head_oid, conclusion)` |
| Agent-writability | not relevant (it caches remote state) | **agent-unwritable through governed paths** — the gate's forgery-hardness depends on it |
| Keyed by | `reference` (a ref name) | `head_oid` (a commit OID) — the SHA-reset invariant requires OID keying |

The guardrail, asserted in review: **the Checks ledger is the NEW `check_results` table, never `ci_checks`.** A build that records butler-produced check results into `ci_checks` would (a) have them wiped on the next forge sync, (b) carry no signature, and (c) key on a ref name instead of an OID — silently breaking the SHA-reset invariant. `ci_checks` stays exactly what it is: a read-through cache of *remote* CI for display.

## The enforcement seam is the SAME one governance audited (four-caller rule, inherited)

The required-checks clause lives inside `enforce_merge_gate`, which is already called from the `but-api` PR-merge boundary: `merge_review` (`forge.rs:607`), `set_review_auto_merge` (`forge.rs:650`), and the dry-run companion `dry_run_merge_review` (`forge.rs:637`). Because Checks adds its clause *inside* `enforce_merge_gate`, it inherits the seam's coverage **and its limits**: the gate binds the **callers that route through `but-api`** (Tauri desktop, the `but` CLI, the TUI). The **N-API residual** (governance R14) is inherited unchanged — an un-audited `but-napi` merge route that skips `but-api` skips the required-checks clause too. The build MUST NOT claim the required-checks clause binds N-API callers it has not audited (see `07-technical-risks.md`, inherited residual R10).

> **DECISION A note on the seam:** the `on-merge-attempt` *run* is sequenced by the **caller** (the trusted CLI/daemon orchestrating the merge) immediately before it calls `merge_review` / `set_review_auto_merge`; it is NOT triggered from inside `enforce_merge_gate`. The gate at this seam stays the pure read-only consumer. See `04-api-design.md` for the orchestration sequence.

## Per-component module map

### `but-checks` (NEW — EXEC, LEDGER-record, GATE-verify, DEFINE)
| File | Change |
|---|---|
| `src/contract.rs` *(NEW)* | `CheckName` (newtype over `String`), `HeadOid` (newtype over `gix::ObjectId`/`String`), `Conclusion` enum, `CheckResult` struct, `ProducerIdentity`, `Signature` newtype |
| `src/conclusion.rs` *(NEW)* | the `Conclusion` enum + parse/serialize; the `satisfies_required(conclusion, policy)` predicate (default: only `success`) |
| `src/executor.rs` *(NEW)* | the default executor: resolve definition → materialize an **isolated checkout at the head OID** (see clean-workspace-checkout note) → run subprocess → derive `Conclusion` → sign → record; runs in-process in the trusted caller (no broker) |
| `src/signing.rs` *(NEW)* | sign `(name, head_oid, conclusion)` with the producer key from `but-secret`; verify a signature against the trusted producer identity |
| `src/config.rs` *(NEW)* | ref-pinned loader for `.gitbutler/actions/*.toml` (read at the target ref via `gix`, mirroring `but_authz::load_governance_config`); never the working tree |
| `src/ledger.rs` *(NEW)* | the record API (deterministic engine code) + the read API the gate uses; wraps `but-db`'s `check_results` handle |
| `src/gate.rs` *(NEW)* | `required_checks_satisfied(required: &[CheckName], results_at_head: &[CheckResult], trusted: &ProducerIdentity) -> Result<(), CheckGateError>` — the pure, read-only verifier the merge gate calls |
| `src/lib.rs` *(NEW)* | crate surface; re-exports |

### `but-db` (EXTENDED — LEDGER storage)
| File | Change |
|---|---|
| `src/table/check_results.rs` *(NEW)* | the `M::up(...)` migration + `CheckResult` row + `CheckResultsHandle`/`CheckResultsHandleMut` (read by target/head, insert) — mirrors `local_review_verdicts.rs` |
| `src/lib.rs` | register `table::check_results::M` in `MIGRATIONS` (`:130`) and re-export the row type (alongside `:78`/`:82`) |

### `but-api` (EXTENDED — the read-only consumer clause + the producer trigger)
| File | Change |
|---|---|
| `legacy/merge_gate.rs` (`enforce_merge_gate`, `:40`) | add the **read-only** required-checks clause after the `Merge` authorization (`:48`), composed with the review requirement; new `gate.check_required` code on the existing `MergeGateError` shape. The clause does NOT run checks. |
| `legacy/checks.rs` *(NEW)* | `run_check` API boundary; `statuses:write` pre-call guard (the `authorize()`-before-`.await` pattern, mirroring `forge.rs:488`; helper at `forge.rs:47`). No `record(conclusion)` entry point. |
| `legacy/mod.rs` | declare `pub mod checks;` |

### `but` CLI verbs (EXTENDED — EXEC/DEFINE/LEDGER-read CLI)
| File | Change |
|---|---|
| `src/command/check.rs` *(NEW)* | `but check {define,list,run,results,required}` impl; calls `but-api` `legacy::checks` + the gate's required-checks read; reuses the `governance_cli_error` → `{error:{code,message}}` rendering pattern (`command/perm.rs:118`). For `but merge` (or the merge path), the CLI orchestrates `on-merge-attempt` runs before calling the merge action (DECISION A). |
| `src/args/check.rs` *(NEW)* | clap subcommand definitions (mirrors `args/perm.rs`) |
| `src/command/mod.rs`, `src/args/mod.rs` | wire the `check` noun |

## Committed config + the new local state
| Path | Change |
|---|---|
| `.gitbutler/actions/*.toml` *(NEW)* | per-check definitions (name, command/run spec, which `conclusion`s satisfy, timeout, optional `on-merge-attempt` flag) — committed, ref-pinned, read at the target ref. The *required-checks* policy (which check names a branch requires) lives alongside the gate config; see `03-data-schema.md` for whether it extends `.gitbutler/gates.toml` or is a new `[[required_check]]` block. (Confirm at planning whether to rename the directory to `.gitbutler/checks/*.toml` per the naming disambiguation.) |
| `check_results` table *(NEW — `but-db`)* | `(id, target, name, head_oid, conclusion, producer_identity, signature, recorded_at, metadata)` — the signed, agent-unwritable ledger. The single piece of new persistent local state. See `03-data-schema.md` + `07-technical-risks.md` R4 for the agent-unwritability requirement and its v1 strength. |

## Component reuse summary
| Capability | Reused existing GitButler component | New code |
|---|---|---|
| The gate the clause lives in | `enforce_merge_gate` (`merge_gate.rs:40`) + its `MergeGateError` contract + `current_head_oid` (`:78`) | the read-only required-checks clause + `gate.check_required` code |
| Authorization for the producer trigger | `but_authz::authorize` + the `statuses:write` authority (already in `authority.rs`) + the pre-call-guard pattern (`forge.rs:488`; helper `forge.rs:47`) | the `statuses:write`-gated `run_check` boundary |
| Ref-pinned config read | `but_authz::load_governance_config` (target-ref blob read via `gix`) | the `.gitbutler/actions/*.toml` loader (same shape) |
| The ledger table pattern | `but-db` `local_review_verdicts.rs` (`M::up` + `MIGRATIONS` + handle) | `check_results.rs` (+ `producer_identity`/`signature` columns) |
| Signing key storage | `but-secret` (keyring) | the producer-key read + the sign/verify primitive |
| SHA-reset basis | `but_rebase::graph_rebase::Editor` (`commit_mappings`, `mod.rs:479`) | OID-keyed results; (optional) prune-superseded housekeeping |
| The denial/result contract | `but_authz::Denial` `{code,message,remediation_hint}` + `MergeGateError` | the `gate.check_required` code |
