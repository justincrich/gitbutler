---
stability: CONSTITUTION
last_validated: 2026-06-19
prd_version: 1.0.1
section: technical-requirements
---
# Data Schema

The Checks slice adds **one new `but-db` table** (`check_results` — the signed, agent-unwritable ledger), **one committed config convention** (`.gitbutler/actions/*.toml`, ref-pinned, mirroring governance's `.gitbutler/*.toml`), and the **new `but-checks` Rust types** (the check-result contract). Net: **1 new `but-db` table + a committed check-definition config + a required-checks policy block + the contract types.** The authoritative *policy* (which checks exist, which a branch requires) is committed config; the only persistent *local* state is the ledger, because results are produced locally and must survive across CLI invocations to be consumed by the gate.

> **Naming:** crate `but-checks`, table `check_results` — deliberately distinct from the pre-existing `butler_actions` macro table. See `02-system-components.md` for the disambiguation.

## New Rust types (`but-checks`)

| Type | Shape | Notes |
|---|---|---|
| `CheckName` *(NEW newtype)* | `struct CheckName(String)` | a check's stable name (e.g. `"cargo-test"`, `"clippy"`). **False-friend:** not a raw `String` at the boundary — newtype so the gate can't confuse a check name with arbitrary text |
| `HeadOid` *(NEW newtype)* | `struct HeadOid(String)` wrapping a `gix::ObjectId` rendering | the head OID a result is bound to; the SHA-reset invariant keys on this. Stored as the hex string in the ledger column `head_oid` (matches `merge_gate.rs`'s `current_head_oid` → `gix::Id::to_string()`). **Naming:** prefer `head_oid` to match the *shipped* governance schema (`local_review_verdicts.head_oid`) and `merge_gate.rs`'s `current_head_oid`; do NOT introduce a divergent `head_sha` name |
| `Conclusion` *(NEW enum)* | `Success` · `Failure` · `Neutral` · `Cancelled` · `TimedOut` · `Skipped` | the **type** mirrors GitHub's six check-run conclusions (defends no-stringly-typed). **False-friend:** not a `String` — the gate's allow-decision is `conclusion == Conclusion::Success`; an unknown variant must be a parse error, never a silent pass. Parsed from/serialized to a stable token (`success`, `failure`, `neutral`, `cancelled`, `timed_out`, `skipped`). **v1 producibility is narrower than the type — see "v1-producible vs reserved" below.** |
| `ProducerIdentity` *(NEW)* | `struct ProducerIdentity { id: String, key_id: String }` | who produced + which key signed; the gate verifies the signature against the trusted producer's `key_id` |
| `Signature` *(NEW newtype)* | `struct Signature(Vec<u8>)` (hex/base64 in the ledger) | a signature over the canonical `(name, head_oid, conclusion)` tuple |
| `CheckResult` *(NEW)* | `{ name: CheckName, head_oid: HeadOid, conclusion: Conclusion, producer: ProducerIdentity, signature: Signature, recorded_at, metadata: Option<String> }` | the in-memory contract; maps 1:1 to a ledger row |
| `CheckDefinition` *(NEW)* | `{ name: CheckName, run: RunSpec, satisfying: Vec<Conclusion>, timeout, on_merge_attempt: bool }` | parsed from `.gitbutler/actions/*.toml`; `satisfying` defaults to `[Success]` for a required check; `on_merge_attempt` tells the **orchestrator** (not the gate) to run this before a merge call (DECISION A) |
| `CheckGateError` *(NEW)* | `{ code: "gate.check_required", message, remediation_hint, unmet: Vec<String> }` | the consumer-side denial; shares the shape of the existing `MergeGateError` (`merge_gate.rs:20`) so the gate can return one contract |

### `Conclusion` — the 6-variant TYPE vs the v1-producible subset (Q3, exact)
The **enum has six variants** (the type defends against stringly-typed conclusions and is forward-compatible). What the **v1 local exit-code executor can actually emit** is a strict subset; the rest are **reserved** — parsed and stored (so a forward-compat schema and a future GitHub-import path round-trip cleanly) but **never produced by v1's executor**:

| Subset | Variants | v1 behavior |
|---|---|---|
| **v1-PRODUCIBLE** | `success`, `failure`, `timed_out` | emitted by the v1 executor from the actual run. `success`/`failure` come from the subprocess exit code (0 vs non-0); **`timed_out` only if a timeout wrapper ships** in v1 — if the timeout wrapper is deferred, v1 produces only `{success, failure}` and a hung check is a `failure`/operator concern (state which in the build). |
| **RESERVED (forward-compat + GH-import)** | `cancelled`, `neutral`, `skipped` | the enum carries them and the ledger column accepts them, so a later GitHub-checks import or a richer executor can store them. The **v1 local exit-code executor never emits them.** |

The gate's rule is independent of producibility: **the gate blocks on any non-`success` required `conclusion`** (and on absent/stale/unsigned), regardless of whether the variant is v1-producible or reserved. The build MUST NOT special-case reserved variants into a pass.

### `Conclusion` ↔ gate semantics (exact, asserted by test)
| `Conclusion` | Token | v1-producible? | Satisfies a *required* check by default? |
|---|---|---|---|
| `Success` | `success` | yes | **yes** |
| `Failure` | `failure` | yes | no → `gate.check_required` |
| `TimedOut` | `timed_out` | yes (only if timeout wrapper ships) | no → `gate.check_required` |
| `Cancelled` | `cancelled` | no (reserved) | no → `gate.check_required` |
| `Neutral` | `neutral` | no (reserved) | **configurable** per check (`satisfying = ["success", "neutral"]`); default no |
| `Skipped` | `skipped` | no (reserved) | **configurable**; default no |

## The new `but-db` table — `check_results` (the signed, agent-unwritable ledger)

The gate must answer: *"does every required check name have a signed `success` bound to the current head OID?"* That needs durable local result state (results are produced locally, consumed by a later gate call). It is a **NEW `but-db` table**, registered in `MIGRATIONS` (`crates/but-db/src/lib.rs:130`) following the `local_review_verdicts`/`ci_checks` migration pattern.

```sql
-- crates/but-db/src/table/check_results.rs (M::up migration)
CREATE TABLE `check_results`(
    `id`                TEXT NOT NULL PRIMARY KEY,    -- record id (uuid)
    `target`            TEXT NOT NULL,                 -- the branch/stack/PR the result is about (MUTABLE name — see below)
    `name`              TEXT NOT NULL,                 -- CheckName
    `head_oid`          TEXT NOT NULL,                 -- HeadOid — LOAD-BEARING (SHA-reset invariant; correctness backstop)
    `conclusion`        TEXT NOT NULL,                 -- Conclusion token (v1 emits success/failure[/timed_out]; cancelled/neutral/skipped reserved)
    `producer_identity` TEXT NOT NULL,                 -- ProducerIdentity (id + key_id), serialized
    `signature`         TEXT NOT NULL,                 -- signature over (name, head_oid, conclusion)
    `recorded_at`       TIMESTAMP NOT NULL,
    `metadata`          TEXT                           -- NULLABLE JSON: run duration, output summary (NOT consulted by the gate); mask secrets before storing
);
CREATE INDEX `idx_check_results_target_head` ON `check_results`(`target`, `head_oid`);
CREATE INDEX `idx_check_results_name_head`   ON `check_results`(`name`, `head_oid`);
```

| Field | Purpose |
|---|---|
| `id` | record id |
| `target` | the branch/stack/PR under check (lets the gate scope to the review being merged, like `local_review_verdicts.target`). **Mutable** — it is a branch *name* (`ForgeReview.source_branch`), which can be reused/renamed; it is a **scoping convenience, not the correctness key.** |
| `name` | the check name the gate matches against the branch's required-check list |
| `head_oid` | **load-bearing correctness backstop** — the head OID the result was produced against; the gate matches `head_oid == current_head_oid` only (the SHA-reset invariant). Because `target` (a branch name) can be reused, **`head_oid` is what makes a stale or name-reused row non-satisfying** — the gate's soundness rests on the OID match, not the name |
| `conclusion` | the typed outcome token; the gate's allow-decision is `conclusion == "success"` (or a configured satisfying set) |
| `producer_identity` | who produced + which key; the gate verifies the signature against the trusted producer's key |
| `signature` | the integrity binding — a record without a verifying signature is treated as absent (this is the correction of `local_review_verdicts`' forgeability) |
| `recorded_at` | ordering / latest-wins per `(name, head_oid)` |
| `metadata` | **nullable** run telemetry (duration, output summary) — informational, **never** consulted by the gate's decision. See the secret-handling note below. |

### `target` is a mutable name; `head_oid` is the correctness backstop (finding 9)
`target` mirrors `local_review_verdicts.target` for scoping, but it is a **branch name** (`ForgeReview.source_branch`) — mutable and reusable (a branch can be deleted and a new one created with the same name). The gate MUST NOT treat `(target, name)` as sufficient identity: a stale row from a *prior* branch incarnation with the same name must not satisfy the gate. **`head_oid` is the backstop** — because the gate matches `head_oid == current_head_oid`, a row from a reused name at a different OID is invisible. The build MUST key the gate decision on `head_oid`; `target` only narrows the query.

### Why signed + producer-bound (the `local_review_verdicts` lesson, corrected)
Governance's `local_review_verdicts` is **not** integrity-protected (governance R6, High): an agent with DB/filesystem write access can INSERT a forged approving row and trivially satisfy the merge gate. Checks **must not** repeat this. The `signature` + `producer_identity` columns make a forged-**unsigned** row inert: a direct INSERT by the agent (without the producer key) produces a row whose signature does not verify, so the gate treats it as **absent** and fails closed.

But be precise about the v1 ceiling. The signing primitive in v1 is **symmetric HMAC** (the same repo-local producer secret signs and verifies). In the **personal-tenant model the agent shares the OS user with the executor**, so it can read the `but-secret` keyring producer secret **without elevated privilege** — and with the secret it can mint a *correctly-signed* forged row. Therefore signing does **NOT close the agent-forgery path**; it **RAISES the bar** (forging now requires reading the producer secret, where before — `local_review_verdicts` — it required nothing). This is a **named residual** (R2/R4), not a closed boundary. The real closure is **Ed25519** (asymmetric — the agent holding the public key cannot forge) **plus an OS-sandboxed executor** where even the private key is unreachable by the agent process; both are **deferred**. The build MUST NOT present the v1 HMAC ledger as tamper-proof or as closing the forgery path.

### Secret-bearing `metadata` is agent-readable — mask before storing (finding 7)
`metadata` may capture run telemetry including **raw stdout/stderr** of the check. That stdout can contain injected secrets (env tokens, keys a check echoes). Because the `check_results` table is in the same agent-readable SQLite file as everything else, **storing raw secret-bearing stdout in `metadata` leaks it to the agent.** Requirements:

- `metadata` is **nullable** (`TEXT`, not `NOT NULL`) — a run may record no metadata at all; the gate never reads it.
- If raw stdout is stored, the executor MUST **mask injected secrets** (the values it injected into the check's environment) before writing `metadata`, and SHOULD **limit raw-stdout retention** (truncate / store a summary rather than full output).
- Defaulting `metadata` to `NULL` (store nothing) is the safe v1 default; opt-in summaries only.

### Latest-wins per `(name, head_oid)` (re-run semantics)
Re-running check `X` at the same head OID supersedes the prior result for `(X, head_oid)` — the gate consults the **most recent** verifying row per `(name, head_oid)` (ordered by `recorded_at`). A re-run at a *new* head OID is a new row at the new `head_oid` (and the old-head row simply stops matching the current head — the SHA-reset invariant). The build chooses one of: (a) UPSERT on `(target, name, head_oid)`, or (b) append + latest-wins read (mirrors `local_review_verdicts`' append + `ORDER BY created_at`). Either is acceptable; the read MUST be deterministic and latest-wins.

## Committed config — the check definitions + the required-checks policy (ref-pinned)

### `.gitbutler/actions/*.toml` *(NEW)* — what a check IS
Per-check definitions, committed, read at the **target ref** (mirroring `but_authz::load_governance_config`), so a feature head cannot redefine a check to trivially pass and have that judge its own merge.
```toml
# .gitbutler/actions/cargo-test.toml
name = "cargo-test"
run  = "cargo test -p but"            # the command the trusted executor runs
satisfying = ["success"]              # which Conclusions satisfy this as a required check
timeout_secs = 600                    # if a timeout wrapper ships, exceeding this → timed_out
on_merge_attempt = true               # the ORCHESTRATOR runs this before calling merge (DECISION A); the gate never runs it
```

### The required-checks policy — which checks a branch REQUIRES (ref-pinned)
The *policy* (a branch requires checks `["cargo-test", "clippy"]`) extends governance's gate config — a NEW `[[required_check]]` block in `.gitbutler/gates.toml`, read at the target ref by `enforce_merge_gate` exactly as it already reads the `[[gate]]` review requirement (`normalize_gates`, see `merge_gate.rs`):
```toml
# .gitbutler/gates.toml  (governance's existing file; Checks adds [[required_check]])
[[branch]]
name = "main"
protected = true

[[required_check]]                    # NEW — Checks
branch = "main"
checks = ["cargo-test", "clippy"]     # every name here must have a signed `success` @head to merge
```
Putting the policy in `.gitbutler/gates.toml` (not a new file) keeps the gate's config single-sourced and ref-pinned by the same mechanism the review requirement already uses; the per-check *definitions* live in `.gitbutler/actions/*.toml` because they are about *how a check runs*, not *what a branch requires*. (Confirm the file split with maintainers; the alternative — a single `.gitbutler/checks.toml` carrying both — is acceptable if the team prefers it, provided both are read at the target ref.)

> **Bootstrap invariant (the real closure for R11, see `01-architecture-posture.md` Stance 9 + `07-technical-risks.md` R11):** a change that edits a required check's definition (`.gitbutler/actions/*.toml`) or the `[[required_check]]` set MUST itself clear the *pre-change* required checks at the target ref before it can land. The required-check config is **self-protecting** — you cannot weaken the checks that judge a merge in that same merge.

## Keying + invalidation by head OID (the SHA-reset invariant, made concrete)

- **Key:** every ledger row carries `head_oid`. The unit of identity is `(name, head_oid)` (per `target`); the **correctness key is `head_oid`** (`target` is a mutable name).
- **Gate read:** `enforce_merge_gate` resolves the current source head OID (`current_head_oid`, `merge_gate.rs:78`) and reads `check_results WHERE target = ? AND head_oid = <current_head_oid>`, filtering to verifying signatures. A row at any other `head_oid` is **invisible** to the gate. **(Read-only — the gate does not run checks; DECISION A.)**
- **Invalidation:** a graph/ref mutation (rebase/commit-edit via `but_rebase::graph_rebase::Editor`) changes the head OID, so prior rows (keyed to the old OID) no longer match — **automatic staleness, no explicit invalidate write required for correctness.** The executor must re-run against the new head to produce satisfying rows.
- **TOCTOU caveat:** the current-head read and the merge commit are **not atomic** — the head can advance between them (R12 in `07-technical-risks.md`). The keying makes a *stale* row non-satisfying, but it does not make the read-then-merge a single atomic step; the "require up to date" / merge-queue closure is deferred.
- **(Optional) housekeeping:** a prune step may delete rows whose `head_oid` is no longer reachable, consulting `Editor::commit_mappings()` (`graph_rebase/mod.rs:479`) for the old→new mapping. This is a storage optimization, **not** a correctness requirement — a stale row is already non-satisfying by the keying alone.

## What is explicitly NOT a schema change
- **No reuse of `ci_checks`** — it is a disposable remote-PR cache (DELETE-then-INSERT on sync, `ci_checks.rs:151`), unsigned, keyed by ref name. The Checks ledger is the NEW `check_results` table (see `02-system-components.md` guardrail).
- **No reuse of `local_review_verdicts`** — that is governance's review store (forgeable, unsigned). Checks records *check* results, signed.
- **No reuse of / collision with `butler_actions`** — that is the pre-existing macro/automation table. The Checks ledger is `check_results`.
- **No new permission table** — the producer trigger is gated by the existing `statuses:write` authority in committed `.gitbutler/permissions.toml` (governance's file), not a new DB table.
- **No agent-runtime / turn / session tables** — out of scope.
- **No remote runner registry / lease table** — the no-broker v1 (Stance 2) ships no runner protocol, so no runner/lease/job-queue schema.
