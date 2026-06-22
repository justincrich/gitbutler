---
stability: CONSTITUTION
last_validated: 2026-06-21
prd_version: 1.5.0
enriches: governance
from_version: 1.4.0
enrichment: local-agent-pr
section: technical-requirements-delta
---

# Technical Requirements Delta — v1.5.0 Local Agent PR

Additive to `10-technical-requirements/` and to the v1.4.0 (`capability-aware-denials`) delta. **Net-additive, frozen-aware.** This delta introduces a **local PR / governed-review-parity** surface (recommended option **(2) Full local PR**): a local review object an agent-principal can open, get assigned a reviewer on, comment on, resolve threads on, and query the status of — **without a remote forge** — so the full request→review→approve→merge loop runs locally and the existing merge gate gates it unchanged. Everything here is **drive-metadata + orchestration surface**; it adds **three new `but-db` tables, additive `#[but_api(napi)]` verbs that REUSE existing authorities, one per-project setting, and ONE additive optional AUTHZ-config descriptor field (`kind`) on the committed-config principal entry that changes no enforcement.** It changes **no land-truth** and stays entirely **out of the git-graph/workspace/rebase model**.

> **Grounding correction (load-bearing — sprint number).** The task brief first said this capability "lands as a new Sprint 07," then an interim draft reassigned it to "Sprint 08" because the v1.4.0 STEER enrichment had already claimed Sprint 07 on the live ROADMAP (`v1.4.0-capability-aware-denials/05-delta-replan.md:141` I4 "✅ APPLIED 2026-06-19 … `sprint_count` 8→9"). **The human has since directed the final numbering (instruction-precedence #1):** LPR takes slot **Sprint 07**, and the existing **STEER Sprint 07 is renumbered to Sprint 08**. So the materialization target here is **Sprint 07 (LPR — Local PR)**, and the ROADMAP becomes (01a, 01b, 02, 03, 04, 05, 06a, 06b, **07-LPR**, **08-STEER**). The STEER renumber (N1) and the LPR row (I4) are owned by [05-delta-replan.md](./05-delta-replan.md) §5; this delta specifies the technical shapes. Count reconciliation against the **live** ROADMAP (`sprint_count: 8` — the v1.4.0 STEER I4 "APPLIED 8→9" never actually landed in `ROADMAP.md`; STEER exists only in `tasks/sprint-07-steer*` + the v1.4.0 enrichment): Stage 2 adds **both** the LPR row (07) **and** the STEER row (08 — its `tasks/` dir + enrichment renumbered 07→08), so `sprint_count` **8 → 10**.

> **Freeze contract (mirrors v1.4.0).** Sprints 01a–06b are FROZEN (some with in-flight agents); STEER is renumbered 07→08 by a numbering-only edit (N1, [05-delta-replan.md](./05-delta-replan.md) §5) that changes no STEER scope. This delta **edits no frozen task file now**. It records (a) the three new tables + their migrations, (b) additive verbs, (c) the project setting, (d) the additive `kind` AUTHZ-config descriptor field, and (e) the remote-mirror seam — all to materialize as **Sprint 07 (LPR)** via `/kb-sprint-tasks-plan`. The safe seam (§E) is what makes this legal: the merge gate reads only `local_review_verdicts`, so nothing here can flip a deny into an allow.

---

## A. Data schema delta — three NEW additive `but-db` tables + their migrations

The authoritative governance state stays committed config (`02-roles.md` / `03-data-schema.md`); the only persistent local state added by **this** capability is **PR-orchestration drive-metadata**: who is assigned to review a target, the review-thread comments, and a small per-target metadata row (the opener/tag). All three are **disposable, principal-scoped, local** — exactly the `local_review_verdicts` class (`crates/but-db/src/table/local_review_verdicts.rs`), **NOT** the disposable remote cache class (`forge_reviews`, cleared by the runtime `DELETE FROM forge_reviews` at `crates/but-db/src/table/forge_reviews.rs:153`).

**Net of this delta: 3 new `but-db` tables + 3 new migration modules + the additive verbs/setting below + 1 additive optional `kind` field on the committed-config principal entry (§A.4). PR _lifecycle_ state is DERIVED, not stored (§A.3).**

### Migration registration (exact, mirroring the shipped pattern)

All three tables follow the shipped `local_review_verdicts` / `workspace_rules` pattern precisely:

- A new per-table module `crates/but-db/src/table/local_review_assignments.rs`, `crates/but-db/src/table/local_review_comments.rs`, and `crates/but-db/src/table/local_review_meta.rs`, each exporting `pub(crate) const M: &[M<'static>]` (the shape `local_review_verdicts.rs:7` exports).
- Each registered in `crates/but-db/src/table/mod.rs` (alongside `pub(crate) mod local_review_verdicts;` at `mod.rs:10`) and appended to `MIGRATIONS` in `crates/but-db/src/lib.rs:130` (the slice that already lists `table::local_review_verdicts::M` at `lib.rs:142`).
- **`SchemaVersion::Zero`** for all three (the value `local_review_verdicts.rs:9` uses) — they are **additive tables older binaries can tolerate** (the exact criterion in `crates/but-db/src/lib.rs:167` doc: "Keep using `Zero` for migrations that older binaries can still tolerate after the migration runs, such as adding tables or columns that they don't require"). Migration ids are fresh, monotonic-by-creation-time `u64`s (e.g. `20260621120000`, `20260621120100`, `20260621120200`) sorted by `migration::run`'s `sort_by_key(|m| m.up_created_at)` at `crates/but-db/src/migration.rs:39`.
- `improve_concurrency` already sets `PRAGMA foreign_keys = ON` (`migration.rs:223`), so FK references are enforced if declared; we **deliberately do NOT FK `principal_id`/`target` to anything** (principals live in committed config, not a table — `03-data-schema.md` "No new permission _table_"), matching `local_review_verdicts` which also stores `principal_id TEXT` un-FK'd.

### A.1 `local_review_assignments` (NEW)

Who is requested/assigned to review a target, and that assignment's standing state.

```sql
CREATE TABLE `local_review_assignments`(
	`id` TEXT NOT NULL PRIMARY KEY,
	`target` TEXT NOT NULL,
	`reviewer_principal` TEXT NOT NULL,
	`state` TEXT NOT NULL,           -- 'pending' | 'approved' | 'changes_requested'
	`assigned_at` TIMESTAMP NOT NULL
);

CREATE INDEX `idx_local_review_assignments_target_reviewer`
ON `local_review_assignments`(`target`, `reviewer_principal`);
```

```rust
// crates/but-db/src/table/local_review_assignments.rs
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};
use crate::{DbHandle, M, SchemaVersion, Transaction};

pub(crate) const M: &[M<'static>] = &[M::up(
    20260621120000,
    SchemaVersion::Zero,
    "CREATE TABLE `local_review_assignments`( \
        `id` TEXT NOT NULL PRIMARY KEY, \
        `target` TEXT NOT NULL, \
        `reviewer_principal` TEXT NOT NULL, \
        `state` TEXT NOT NULL, \
        `assigned_at` TIMESTAMP NOT NULL \
    ); \
    CREATE INDEX `idx_local_review_assignments_target_reviewer` \
    ON `local_review_assignments`(`target`, `reviewer_principal`);",
)];

/// Tests are in `but-db/tests/db/table/local_review_assignments.rs`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalReviewAssignment {
    pub id: String,
    pub target: String,
    pub reviewer_principal: String,
    pub state: String,            // serialized form of AssignmentState (see note)
    pub assigned_at: chrono::NaiveDateTime,
}
```

- **`state` is stored as `TEXT`, typed at the boundary.** Mirrors `LocalReviewVerdict.verdict: String` (a free `TEXT` validated by the writer; `merge_gate`'s `evaluate` filters on the literal `"approved"`, `review_requirement.rs:8`). The `but-authz`/`but-api` layer maps it to a typed `enum AssignmentState { Pending, Approved, ChangesRequested }` on read/write (the same shape discipline the `Authority` parse/`name` round-trip uses, `authority.rs:69`/`:94`). The DB column stays `TEXT` for migration-tolerance and to match the sibling table's convention.
- **Handle/Handle-Mut pair** mirrors `LocalReviewVerdictsHandle`/`…HandleMut` (`local_review_verdicts.rs:54`–`:109`): `list_by_target(&str)`, `upsert(row)` (assignment is idempotent per `(target, reviewer_principal)` — an `INSERT … ON CONFLICT` keyed on the index, or delete-then-insert), `set_state(target, reviewer_principal, state)`. Index `(target, reviewer_principal)` serves both the open-assignments query (§A.3) and the per-reviewer state update.
- **Distinct-from-author at the write boundary (drive-layer mirror of the gate).** `assign_reviewer` (§B) enforces `reviewer_principal != author_principal_of_target_branch` before the upsert — the same `require_distinct_from_author` constraint the merge gate applies to the _verdict_ (`review_requirement.rs`), now mirrored at the _drive_ layer so a self-assignment cannot narrate "independently reviewed". A reviewer == author request is rejected/flagged; the same-principal forgery is named as **R22 (§G)**.

### A.2 `local_review_comments` (NEW)

Review-thread comments on a target, threaded and resolvable.

```sql
CREATE TABLE `local_review_comments`(
	`id` TEXT NOT NULL PRIMARY KEY,
	`target` TEXT NOT NULL,
	`author_principal` TEXT NOT NULL,
	`body` TEXT NOT NULL,
	`file` TEXT,                     -- nullable: file-scoped vs PR-level comment
	`line` INTEGER,                  -- nullable: line within `file`
	`thread_id` TEXT NOT NULL,       -- groups a comment thread
	`resolved` BOOL NOT NULL,
	`created_at` TIMESTAMP NOT NULL
);

CREATE INDEX `idx_local_review_comments_target_thread`
ON `local_review_comments`(`target`, `thread_id`);
```

```rust
// crates/but-db/src/table/local_review_comments.rs
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};
use crate::{DbHandle, M, SchemaVersion, Transaction};

pub(crate) const M: &[M<'static>] = &[M::up(
    20260621120100,
    SchemaVersion::Zero,
    "CREATE TABLE `local_review_comments`( \
        `id` TEXT NOT NULL PRIMARY KEY, \
        `target` TEXT NOT NULL, \
        `author_principal` TEXT NOT NULL, \
        `body` TEXT NOT NULL, \
        `file` TEXT, \
        `line` INTEGER, \
        `thread_id` TEXT NOT NULL, \
        `resolved` BOOL NOT NULL, \
        `created_at` TIMESTAMP NOT NULL \
    ); \
    CREATE INDEX `idx_local_review_comments_target_thread` \
    ON `local_review_comments`(`target`, `thread_id`);",
)];

/// Tests are in `but-db/tests/db/table/local_review_comments.rs`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalReviewComment {
    pub id: String,
    pub target: String,
    pub author_principal: String,
    pub body: String,
    pub file: Option<String>,
    pub line: Option<i64>,        // SQLite INTEGER → i64 (cf. ForgeReview.number: i64, forge_reviews.rs:48)
    pub thread_id: String,
    pub resolved: bool,
    pub created_at: chrono::NaiveDateTime,
}
```

- **`file: Option<String>` / `line: Option<i64>`** follow the nullable-column convention already in `forge_reviews` (`body: Option<String>`, `forge_reviews.rs:50`). A PR-level comment has both `None`; a code comment carries both.
- **Handle pair** mirrors the sibling tables: `list_by_target(&str)`, `list_by_thread(target, thread_id)` (uses the `(target, thread_id)` index), `insert(row)`, `set_resolved(thread_id, resolved)`.
- **Resolve is authority-constrained (no self-clean-signal forgery).** `resolve_thread` (§B) requires the resolver to be the **thread author OR the assigned reviewer OR a holder of the higher `reviews:write` authority** — so a single principal cannot post a `changes_requested`-style thread and self-resolve it to forge a clean "all-clear" reconciler signal for another party. The same-principal post-and-resolve forgery is named as **R22 (§G)**.
- **`body` is attacker-influenceable text** (an agent-principal writes it; another agent-principal reads it as context). This is named as a **risk (R20, §G)** — it is the same injection class the v1.4.0 delta named for `message`/`unmet[]` (R15 there), surfaced here because comment bodies feed agent context. The schema must NOT be presented as sanitized.

### A.3 PR lifecycle state is DERIVED — no separate table

There is **no `local_pull_requests` table.** The "PR" is a **derived view** over already-present sources, computed at query time (the same lossy-presentation discipline `WORKSPACE_MODEL.md` mandates for `Workspace`/`RefInfo`):

```
pr_state(target) =
    commits      : the branch's commits ahead of base      (gix graph walk — read-only, NO mutation)
    verdict      : local_review_verdicts.list_by_target(target) filtered to head_oid == current head
                   (the EXACT query merge_gate already runs — merge_gate.rs:84 → review_requirement::evaluate)
    assignments  : local_review_assignments.list_by_target(target) where state == 'pending'
    comments     : local_review_comments open-thread count (resolved == false)
    meta         : local_review_meta(target, 'opener_principal') → cache of the computed agent-PR tag (§A.4)
  ⇒ derived status ∈ { Draft, AwaitingReview, ChangesRequested, Approved, Mergeable }
```

- **`Approved`/`Mergeable`** is a presentation label only — the **actual merge decision stays `enforce_merge_gate`** (§E), which re-derives verdict-at-head itself and never reads `local_review_assignments`/`local_review_comments`/`local_review_meta`. The derived `pr_state` is for display/orchestration; it is **not** consulted by the gate. This keeps the three tables strictly additive: a bug in the derivation can mislabel a PR but can never authorize a merge.
- **Why derived, not stored:** storing lifecycle would duplicate land-truth that already lives in commits + `local_review_verdicts`, inviting the two to diverge (the failure `WORKSPACE_MODEL.md` warns against). Deriving keeps a single source per fact.

### A.4 The agent-PR TAG — derived from the opener principal's DECLARED `kind` in committed config

A local review opened by an **agent principal** is tagged so humans/orchestrators can distinguish agent-authored PRs (the **precedent is `ForgeReview.labels: String`**, a comma-joined label blob — the SQL column at `crates/but-db/src/table/forge_reviews.rs:18`, the struct field at `:52`).

- **The source-of-truth is a DECLARED config fact, NOT handle-resolution.** The agent-vs-human distinction **does not exist in the resolved principal**: `resolve_principal` (`crates/but-authz/src/authorize.rs:67`–`:90`) keys **solely** on `BUT_AGENT_HANDLE` (the `lookup(BUT_AGENT_HANDLE)` at `authorize.rs:71`) and constructs `Principal::new(principal_id, authorities, groups)`; `Principal` is `{ id, authorities, groups }` (`crates/but-authz/src/principal.rs:82`) with **no `is_agent` / `PrincipalKind` / human discriminator**. The forge write path resolves **every** caller — agent _and_ human — through the same `resolve_principal_from_env(&cfg)` (`crates/but-api/src/legacy/forge.rs:58`). So "the opener resolved from `BUT_AGENT_HANDLE`" is true of **all** governed principals — it **cannot** tell an agent from a human, and there is **no `UserService`-based governed path** to contrast with. **Deriving the tag from handle-resolution is therefore a fabrication** (the same class as v1.4.0's invented `Denial.unmet`).
  Instead, the tag derives from a **new additive, optional `kind` field on the principal entry in committed `.gitbutler/permissions.toml`** — `kind = "agent"` / `kind = "human"` — read at the **target ref** like all governance config (`permissions.toml` is loaded from the target-ref tree via `gix`, never the working tree, `crates/but-authz/src/config.rs:23`–`:25` — the anti-self-escalation property). The derivation marks `agent_authored = true` **iff the opener principal's committed entry declares `kind = "agent"`**; an omitted `kind` defaults to human (the conservative posture — a principal is only "agent" if config says so).
- **The additive AUTHZ-config delta (grounded, enforcement-neutral).** The `kind` field rides the existing raw `[[principal]]` wire entry `PrincipalWire` (`crates/but-authz/src/config.rs:424`–`:431`: `{ id: String, permissions: Vec<String>, role: Option<String>, groups: Vec<String> }`, `#[serde(deny_unknown_fields)]`). The addition is **exactly the optional-field pattern the existing `role: Option<String>` already uses** (`config.rs:427`): add `#[serde(default)] pub kind: Option<String>` (or a typed `Option<PrincipalKind>`). Because `PrincipalWire` is `#[serde(deny_unknown_fields)]`, the field **must** be declared on the wire struct (a free-floating key would fail to parse) — and `#[serde(default)]` means **older committed `permissions.toml` files without `kind` deserialize cleanly** (`None` → human). **This field does NOT change enforcement:** it does **not** flow into `GovConfig.principals: BTreeMap<PrincipalId, AuthoritySet>` (the enforcement map, `config.rs:85`); **no gate reads it** (`authorize`/`effective_authority`/`merge_gate`/`commit/gate` are untouched). It is a **descriptor** surfaced to the tag-derivation (and the desktop UI) via the loaded config — the same committed-config trust model as the rest of `permissions.toml`, read at the target ref so an actor cannot self-escalate it in the working tree.
- **Where the computed tag is cached — a dedicated per-target metadata row, NOT a comment-body sentinel.** The _computed_ `agent-authored` tag is cached on the derived PR object and recorded in a dedicated `local_review_meta` row (the F-003 storage). Storing the opener/tag as a `__pr_meta__` row inside `local_review_comments.body` is **rejected**: a comment body is attacker-influenceable free text (R20), so making it a control-plane input would let any `local_review_comments`-write actor **forge the opener** and flip the tag. Instead the opener rides a **dedicated `local_review_meta` table** with a `UNIQUE(target, key)` constraint (`PRIMARY KEY (target, key)`), holding one row per `(target, key)` — e.g. `key = "opener_principal"`. An `agent_authored: bool` column on `local_review_assignments` is also **wrong** (assignments are per-reviewer, not per-PR). The chosen `local_review_meta` row is per-target structured metadata that **caches the computed tag**, separate from the free-text comment surface; the _source-of-truth_ for "is this agent-authored" remains the committed principal `kind`.

```sql
CREATE TABLE `local_review_meta`(
	`target` TEXT NOT NULL,
	`key` TEXT NOT NULL,             -- e.g. 'opener_principal'
	`value` TEXT NOT NULL,
	`created_at` TIMESTAMP NOT NULL,
	PRIMARY KEY (`target`, `key`)    -- UNIQUE(target, key): one row per metadata key per target
);
```

```rust
// crates/but-db/src/table/local_review_meta.rs
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};
use crate::{DbHandle, M, SchemaVersion, Transaction};

pub(crate) const M: &[M<'static>] = &[M::up(
    20260621120200,
    SchemaVersion::Zero,
    "CREATE TABLE `local_review_meta`( \
        `target` TEXT NOT NULL, \
        `key` TEXT NOT NULL, \
        `value` TEXT NOT NULL, \
        `created_at` TIMESTAMP NOT NULL, \
        PRIMARY KEY (`target`, `key`) \
    );",
)];

/// Tests are in `but-db/tests/db/table/local_review_meta.rs`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalReviewMeta {
    pub target: String,
    pub key: String,
    pub value: String,
    pub created_at: chrono::NaiveDateTime,
}
```

- **How it is set:** when `request_review` (§B) opens the first review for a `target`, it records the opener principal id in `local_review_meta(target, "opener_principal", <opener principal id>)` via an `INSERT … ON CONFLICT(target, key) DO NOTHING` (the `UNIQUE(target, key)` makes the opener write-once per target — a later caller cannot overwrite a recorded opener). The derivation (§A.3) resolves that opener principal's committed-config entry **at the target ref** and marks `agent_authored = true` **iff that entry declares `kind = "agent"`** (NOT from `BUT_AGENT_HANDLE` resolution — the env handle only resolves _which_ principal acted; the declared `kind` says whether that principal is an agent).
- **Set automatically, never caller-supplied:** the tag is **derived from the opener principal's declared kind**, never a parameter — exactly as `approve_review` derives `principal_id` from `authorize_branch_action(...)?` rather than a parameter (`crates/but-api/src/legacy/forge.rs:526`–`:539`). There is **no `--agent` flag** and **no `--kind` flag** on `but review request`. Two residuals stay named: (1) an actor who re-exports `BUT_AGENT_HANDLE` to act as a **different declared principal** inherits that principal's declared `kind` — the **R2 identity residual**, named as **R19 (§G)** (the actor cannot self-assert a _kind_ via a bare env var, but it can impersonate a different principal whose declared kind differs); (2) a DB-write actor forging the `local_review_meta` opener row directly — named as **R23 (§G)** (distinct from R20's comment-body injection: R23 is forgery of the _tag-derivation control path_, R20 is injection into _agent context_ via comment text).

---

## B. API / CLI / NAPI surface — additive verbs that REUSE existing authorities

All verbs are additive `#[but_api(napi)]` functions in **`crates/but-api/src/legacy/forge.rs`**, modeled **exactly** on the shipped `approve_review` (`forge.rs:523`–`:546`): resolve+authorize via `authorize_branch_action(&repo, &branch, Authority::X)?` (`forge.rs:47`), then write to the local cache. **No new authority is introduced** — every verb maps to an authority already in the catalog (`crates/but-authz/src/authority.rs:11`) and already in the route→Authority table (`04-api-design.md`).

| `but-api` fn (NEW, `#[but_api(napi)]`)                   | Action                                   | Required `Authority` (REUSED)                                                                                                                        | Writes                                                                                  |
| -------------------------------------------------------- | ---------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| `request_review(ctx, branch)`                            | open/request a local review for a branch | `PullRequestsWrite` (the open-PR authority — `publish_review` authorizes it at `forge.rs:488`)                                                       | `local_review_assignments` (+ `local_review_meta` opener row, §A.4)                     |
| `assign_reviewer(ctx, branch, reviewer)`                 | assign a reviewer principal to a target  | `ReviewsWrite` (the review-interaction authority — `approve_review` authorizes it at `forge.rs:526`; assignment is a review act, not opening the PR) | `local_review_assignments` upsert (`state='pending'`), gated `reviewer != author`       |
| `post_comment(ctx, branch, body, file, line, thread_id)` | post a review comment/thread             | `CommentsWrite` (the comment authority — the stubbed `comment_review` authorizes it at `forge.rs:574`)                                               | `local_review_comments` insert                                                          |
| `list_comments(ctx, branch)`                             | list comments/threads for a target       | branch-scoped read (no write authority; see note)                                                                                                    | — (reads `local_review_comments`)                                                       |
| `resolve_thread(ctx, branch, thread_id, resolved)`       | resolve/unresolve a comment thread       | `CommentsWrite` **+ resolver-identity constraint** (author / assigned reviewer / `reviews:write` holder)                                             | `local_review_comments` set_resolved                                                    |
| `review_status(ctx, branch)`                             | query the derived PR lifecycle (§A.3)    | branch-scoped read                                                                                                                                   | — (reads commits + verdicts + assignments + comments + meta + opener's declared `kind`) |

**Authority reuse is the contract (no new `Authority` variant).** `request_review` gates on `PullRequestsWrite` (the open-PR authority, `PullRequestsWrite` at `authority.rs:21`, the same the shipped `publish_review` authorizes at `forge.rs:488`); **`assign_reviewer` gates on `ReviewsWrite`** (`authority.rs:23`) — assignment is a _review interaction_, the same authority the shipped `approve_review`/(stubbed) `request_changes_review` authorize (`forge.rs:526`/`:558`), **not** the PR-opening authority. Comment writes gate on `CommentsWrite` (`authority.rs:25`, the same the stubbed `comment_review` authorizes at `forge.rs:574`); approve continues to gate on `ReviewsWrite` via the **shipped** `approve_review` (`forge.rs:523`). This keeps the route→Authority table closed and means `02-roles.md`'s role presets already grant/deny these correctly (an agent with `write` role holds `pull_requests:write` + `comments:write` + `reviews:write`; `authority.rs:343` `WRITE_AUTHORITIES`).

**Two shipped review verbs are CONTRACT STUBS that LPR must REPLACE — `request_changes_review` AND `comment_review`.** Both authorize the right authority and then write **nothing**, returning `task_contract_invalid`:

- **`request_changes_review` (`forge.rs:551`)** authorizes `ReviewsWrite` (`forge.rs:558`) then returns `task_contract_invalid("request_changes_review", …)` (`forge.rs:559`–`:565`) — it persists no state. LPR must **implement** the real changes-requested write (set `local_review_assignments.state='changes_requested'` on `ReviewsWrite`); that is NEW LPR work (**LPR-003**), **not** reuse.
- **`comment_review` (`forge.rs:569`)** authorizes `CommentsWrite` (`forge.rs:574`) then returns `task_contract_invalid("comment_review", …)` (`forge.rs:575`–`:581`) — it likewise **writes nothing**. LPR's new `post_comment` is the real implementation: it **REPLACES** (does NOT wrap) the stubbed `comment_review` path, persisting the comment to `local_review_comments`. The `but review comment` CLI verb routes to `post_comment`, not to the dead stub.

No new `Authority` variant either way — both stubs already authorize correctly; LPR supplies the missing **writes**, keeping the route→Authority table closed.

**`assign_reviewer` enforces distinct-from-author (drive-layer mirror of the gate, F-004/R22).** Before the `local_review_assignments` upsert, `assign_reviewer` requires `reviewer != author_principal_of_target_branch` — the same `require_distinct_from_author` the merge gate applies to the _verdict_ (`review_requirement.rs`), now mirrored at the _drive_ layer. Without it, an implementer could self-assign as its own reviewer and the drive narrative would falsely read "independently reviewed" even though the gate (which checks the _verdict's_ distinctness) would still catch the actual land. A self-assignment is rejected/flagged; the same-principal forgery is **R22 (§G)**.

**`resolve_thread` enforces a resolver-identity constraint (no self-clean-signal forgery, F-002/R22).** Beyond `CommentsWrite`, the resolver must be the **thread author OR the assigned reviewer OR a holder of the higher `reviews:write` authority**. Without this, a reviewer could post its own `changes_requested` thread and immediately self-resolve it, forging a false "all-clean" reconciler signal that suppresses remediation for another party. The same-principal post-and-resolve forgery is **R22 (§G)**; the e2e proof is T-LPR-044.

**Read verbs (`list_comments`, `review_status`) are BRANCH-scoped, not authority-gated (honest disclosure note, F-006).** They are **not** principal-self-scoped: they return **all** comments/assignments/meta for the named branch, so any caller on a governed project who can name a branch sees that branch's full review surface (every principal's threads and assignments on it). This is an **accepted disclosure** — the review surface of a branch is shared drive-state by design (the reconciler thesis needs every orchestrator to read the same branch state), the same posture as the read handles on the sibling tables (`LocalReviewVerdictsHandle::list_by_target`, `local_review_verdicts.rs:64`). It is **not** the per-principal self-scoping that `governance_status_read` provides; do not claim these reads keep cross-principal disclosure gated — they are branch-scoped, and a branch's review surface is visible to all callers on the project. (What stays gated is _writing_ — every write verb above checks an `Authority` at the `but-api` boundary.)

**Local writes need NO DryRun guard (cite the precedent + why).** `approve_review` writes `local_review_verdicts` with **no `DryRun` check** (`forge.rs:534`–`:543`) — and that is correct because the write touches **only the local project cache** (`ctx.db.get_cache_mut()` → SQLite), **not refs, objects, or oplog.** RULES.md's "Preserve `DryRun` semantics; dry runs must not persist refs, objects, or oplog" therefore does not bind these writes: a local-cache row is none of those. The new verbs write the same cache the same way, so they match `approve_review` exactly and **omit the DryRun guard for the same reason.** (Contrast the **merge** verb, which IS gated and IS forge-bound — `merge_review`, `forge.rs:600` — and is untouched here.)

**Async vs sync shape.** Follow the shipped split (`04-api-design.md` "(a)/(b)"): the write verbs that need the repo head (`request_review`, `post_comment` with a head-pinned thread) are `async fn(ctx: ThreadSafeContext, …)` with `let ctx = ctx.into_thread_local();` exactly like `approve_review` (`forge.rs:523`–`:524`); pure-cache reads (`list_comments`) may be sync `fn(ctx: &Context, …)` like `get_review` (`forge.rs:401`). Authorization (and the distinct-from-author / resolver-identity constraints) is the **pre-call guard** (`authorize_branch_action(...)?` + the identity check) before any `.await`, satisfying RULES.md "authorize before the guard."

**CLI verbs under `but review *`** (in `crates/but/src/command/legacy/forge/review.rs`, alongside the shipped `approve`/`request_changes`/`comment`/`close` at `review.rs:20`/`:37`/`:55`/`:73`): add `request` (→ `request_review`), `assign` (→ `assign_reviewer`), `comment` already exists and is **re-pointed at the real `post_comment`** (replacing the stubbed `comment_review` route) and extended to carry `--file`/`--line`/`--thread`, `resolve` (→ `resolve_thread`), `status` (→ `review_status`). Each prints the ref-pin caveat where it writes config-visible state and routes errors through the existing `review_gate_cli_error` serializer (`review.rs:89`). Verb definitions live in `crates/but/src/args/` (NOT `but-clap` — `04-api-design.md` "New CLI verbs" note).

**TS SDK regenerates.** Because these are `#[but_api(napi)]`, the generated `@gitbutler/but-sdk` must be regenerated (`pnpm build:sdk && pnpm format`, per RULES.md "SDK generation flow"); generated files in `packages/but-sdk/src/generated` are not hand-edited. The Electron lite app reaches these via the N-API binding — and per **R14** (`07-technical-risks.md`) any consequential N-API route must go through the `but-api` gated wrapper; these verbs do (they ARE `but-api` fns), so they inherit the audited seam.

---

## C. Project-based "keep PRs local" setting

Per-project config that controls whether reviews stay local-only or are mirrored to a remote forge. The default is **agent reviews default LOCAL.** It is a **per-project operator preference under the R12 trusted-desktop model — NOT functional-permission-gated** (no `administration:write`, not ref-pinned committed config); an untrusted write to the project store that flips it is the named residual **R21 (§G)**.

- **Where it lives:** a new field on `gitbutler_project::Project` (`crates/gitbutler-project/src/project.rs:72`), alongside the existing per-project forge knobs `forge_override: Option<String>` (`project.rs:129`) and `preferred_forge_user` (`project.rs:134`). Proposed: `#[serde(default)] pub keep_reviews_local: DefaultTrue` (reusing the `DefaultTrue` pattern already used by `ok_with_force_push` at `project.rs:106`, the type imported at `project.rs:10`), so **the default is `true` = local** and **older project files without the field deserialize to local** (the same `#[serde(default)]` + `DefaultTrue` combination `ok_with_force_push` relies on at `project.rs:106` — note `force_push_protection`/`husky_hooks_enabled` at `project.rs:109`/`:113` are plain `bool` defaulting `false`, the WRONG precedent here). Defaulted in `default_with_id` (`project.rs:139`) like every sibling field.
- **Why `Project`, not committed config (R12 trusted-desktop, not a checked grant):** this is a **local operator preference** about where review _artifacts_ go, not a governed authorization fact — it must NOT be ref-pinned committed config (that's for `permissions.toml`/`gates.toml`, which gate _decisions_) and it must NOT be `administration:write`-gated. It is the same class as `forge_override`/`preferred_forge_user`: a per-project setting persisted in the project store (`crates/gitbutler-project/src/storage.rs`), owned by the desktop human under the **R12 human-as-fleet-owner trust model** (the human owns the repo + its agents; "your machine, your keys"). The consequence — an untrusted process that can write the project store flips the flag and agent PRs begin mirroring to a public forge — is the **accepted residual R21 (§G)**, the same accepted-leak class as R12. The build must NOT present this setting as an authorization boundary.
- **The gate to remote mirroring:** mirroring (§D) is performed **only when `keep_reviews_local == false`.** While `true` (the default), the local review object is never pushed to a forge; the loop is fully local. Flipping to `false` is an explicit operator action (set via the project-settings surface, the same path that sets `forge_override`).
- **The contract the `but-*` skills consume to auto-set local:** the `but-*` agent-orchestration skills (the kb/`but-run-sprint` family that dispatches agent principals) read `review_status`/project settings and **auto-set `keep_reviews_local = true`** when initializing a governed agent project, so agent-authored PRs default local without operator action. This is a **skill-side default**, not a governance enforcement — an operator who wants remote mirroring sets `false` themselves. The skill contract is: _"on governed-project init, if unset, set `keep_reviews_local = true`."_ (Because the field defaults `true` via `DefaultTrue`, the skill's action is belt-and-suspenders, making the intent explicit in the stored project.)

---

## D. Remote-mirror seam (design-for-later — DO NOT build)

The local review object is designed to be **mirrorable** to a real GitHub/GitLab PR, behind the §C setting, via the **already-shipped** `ForgeReview` + `but-forge` `sync_reviews` bridge. **This is deferred — specify the mapping, build nothing now.**

- **The bridge exists:** `but_forge::create_forge_review` (`crates/but-forge/src/review.rs:1251` — exact fn entry) opens a real PR (GitHub via `but_github::pr::create`, `review.rs:1278`; GitLab MR via `but_gitlab::mr::create`, `review.rs:1298`), and `but_forge::sync_reviews` (`review.rs:1349` — exact) syncs review state into the `forge_reviews` cache (`crates/but-db/src/table/forge_reviews.rs`). The shipped `publish_review`/`update_review_footers` verbs (`forge.rs:480`/`:741`) already drive these. (All four anchors are exact: `create_forge_review:1251`, `but_github::pr::create:1278`, `but_gitlab::mr::create:1298`, `sync_reviews:1349`.)
- **The mapping (deferred):**

  | Local object (this delta)                                  | Mirrors to (`but-forge` / `ForgeReview`)                                                                            |
  | ---------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- |
  | derived PR (commits + opener tag)                          | `create_forge_review(CreateForgeReviewParams{ source_branch, target_branch, title, body, draft })` (`forge.rs:483`) |
  | `local_review_assignments.reviewer_principal`              | requested reviewers on the forge PR (forge-API reviewer list)                                                       |
  | `local_review_comments` (thread, file, line)               | forge review comments via `sync_reviews` (`review.rs:1349`)                                                         |
  | `local_review_verdicts` (approved@head)                    | forge review approval (the existing approve path)                                                                   |
  | agent-PR tag (§A.4, declared `kind` → `local_review_meta`) | `ForgeReview.labels` (`forge_reviews.rs:52`) — e.g. an `agent-authored` label                                       |

- **Disclosure at the deferred seam — internal principal → forge identity (F-005, folded into R21).** The `reviewer_principal → forge reviewer list` row above maps an **internal governance principal identifier** onto a **public forge reviewer/account identity** via a forge API call. When mirroring is enabled (`keep_reviews_local == false`), that mapping **discloses internal principal identifiers to a public forge** (the principal handle, or whatever account it resolves to), and it has **mapping-failure modes**: a principal with no corresponding forge account, a stale/renamed handle, or a many-to-one collapse — any of which can silently drop a reviewer, mis-attribute one, or leak the raw internal handle as a label/comment author. This is named with **R21 (§G)** (a sub-point of the keep-reviews-local residual): the deferred mirror must define an explicit principal→forge-identity mapping with a fail-closed policy for unmapped principals, and must NOT be presented as a clean lossless bridge. Until built, no principal identifier crosses to a forge (the default `keep_reviews_local = true` keeps the loop local).
- **Marked deferred.** No mirroring code lands in Sprint 07 (LPR); the tables and verbs are shaped so a future "mirror" verb (`mirror_local_review`, gated by `PullRequestsWrite`, active only when `keep_reviews_local == false`) can map the rows above into `create_forge_review` + `sync_reviews` without schema change. This is the **same forward-seam discipline** the merge gate already follows (it binds the governed action and names the forge path as the deferred closure, `07-technical-risks.md` R11).

---

## E. THE SAFE SEAM (the load-bearing invariant)

**The merge gate reads ONLY `local_review_verdicts`. Therefore every table and field in this delta is additive drive-metadata that NEVER changes the land-truth.** Proof, by file:line:

1. The governed merge calls `enforce_merge_gate(&ctx, review_id)` (`crates/but-api/src/legacy/forge.rs:607`, inside `merge_review`; also `forge.rs:650` inside `set_review_auto_merge`, and `forge.rs:637` inside `dry_run_merge_review`).
2. `enforce_merge_gate` (`crates/but-api/src/legacy/merge_gate.rs:40`) authorizes `Authority::Merge` (`merge_gate.rs:48`) then loads the review requirement and reads **only** `review_verdicts(ctx, &review.source_branch)` → `ctx.db.get_cache().local_review_verdicts().list_by_target(target)` (`merge_gate.rs:84` → the `review_verdicts` helper at `merge_gate.rs:159`–`:170`).
3. Those verdicts flow into `review_requirement::evaluate(...)` (`merge_gate.rs:86`; `evaluate` is defined at `crates/but-api/src/legacy/review_requirement.rs:37`–`:77`), whose approval filter (`current_approvals`) keeps `verdict.head_oid == current_head_oid` (`review_requirement.rs:94`) and `verdict == APPROVED` where `APPROVED = "approved"` (`review_requirement.rs:8`) — **verdict-at-head, untouched by this delta.**
4. **Nowhere** does `enforce_merge_gate` or `review_requirement::evaluate` read `local_review_assignments`, `local_review_comments`, or `local_review_meta` (the three new tables), nor the new principal `kind` config field — they did not exist when this code was written and this delta **adds no read of them to the gate path.**

**The invariant (state it verbatim in the gate test):**

> **"Gate gates (verdict-at-head, untouched); new tables drive (orchestration)."** `local_review_assignments`, `local_review_comments`, and `local_review_meta` are orchestration drive-metadata, and the principal `kind` field is a tag descriptor. The merge decision is `enforce_merge_gate` reading `local_review_verdicts` at head, exactly as it does today. No row in any new table, no value of the derived PR state (§A.3), and no principal `kind` declaration can cause `enforce_merge_gate` to permit a merge it would otherwise deny.

This is what makes the whole capability legal under the freeze: it cannot regress the land-truth because it never participates in the land decision. A build-gate grep asserts the gate path (`merge_gate.rs`, `review_requirement.rs`) contains **no reference** to `local_review_assignments`/`local_review_comments`/`local_review_meta` (the same honesty-grep discipline the AUTHORITY_POSITIVE_PATTERN gate uses in `but-authz/tests/invariant_build_gates.rs`).

---

## F. Blast-radius table by crate

Risk that this delta's changes could affect existing behavior, per crate:

| Crate / surface                                                                                                     | Change                                                                                                                                              | Risk         | Why                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| ------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- | ------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `but-db`                                                                                                            | +3 table modules, +3 migrations (`SchemaVersion::Zero`), +3 structs                                                                                 | **LOW**      | Additive tables only; `Zero` is forward-tolerable (`lib.rs:167`); no change to existing tables/queries. Migration order is creation-time-sorted (`migration.rs:39`) and the new ids are the highest, so they append.                                                                                                                                                                                                                                                                         |
| `but-api` (`legacy/forge.rs`)                                                                                       | +6 additive `#[but_api(napi)]` fns, modeled on `approve_review`; **replaces** the stubbed `comment_review`/`request_changes_review` writes          | **MODERATE** | New surface area + N-API binding (R14 applies but is satisfied — they ARE `but-api` fns). Reuses `authorize_branch_action` (`forge.rs:47`); the two replaced verbs were stubs (authorized then `task_contract_invalid`, `forge.rs:559`/`:575`) so supplying their writes is additive, not a behavior change to a working verb. The drive-layer integrity constraints (`assign_reviewer` distinct-from-author; `resolve_thread` resolver-identity) live here at the `but-api` boundary (R22). |
| `but-authz`                                                                                                         | typed `AssignmentState` enum + (de)serialization at the boundary; **+1 optional `kind: Option<String>` field on `PrincipalWire`** (`config.rs:424`) | **LOW**      | A new pure enum + parse/`name` round-trip (the `Authority` pattern, `authority.rs:69`/`:94`), and one additive `#[serde(default)]` field on the wire entry (the existing `role: Option<String>` precedent, `config.rs:427`) read at the target ref. **No new `Authority` variant, no change to `authorize`/`effective_authority`; the `kind` field does NOT enter `GovConfig.principals` and no gate reads it.**                                                                             |
| `but` CLI (`command/legacy/forge/review.rs`, `args/`)                                                               | +5 verbs under `but review *`; re-point `comment` at the real `post_comment`                                                                        | **LOW**      | Additive verbs; errors route through the existing `review_gate_cli_error` (`review.rs:89`). CLI tests are happy-path only (RULES.md).                                                                                                                                                                                                                                                                                                                                                        |
| `but-napi` (Electron lite binding)                                                                                  | regenerated bindings for the 6 new fns                                                                                                              | **LOW**      | Auto-generated from `#[but_api(napi)]`; consequential routes go through the audited `but-api` seam (R14).                                                                                                                                                                                                                                                                                                                                                                                    |
| generated SDK (`packages/but-sdk`)                                                                                  | regenerated TS                                                                                                                                      | **LOW**      | Mechanical `pnpm build:sdk && pnpm format`; no hand-edits.                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| `gitbutler-project`                                                                                                 | +1 `Project` field `keep_reviews_local: DefaultTrue`                                                                                                | **LOW**      | Additive, `#[serde(default)]` + `DefaultTrue` (older files deserialize to local); defaulted in `default_with_id` (`project.rs:139`). Same shape as `ok_with_force_push` (`project.rs:106`) — the real `DefaultTrue` precedent (NOT `force_push_protection`/`husky_hooks_enabled`, which are plain `bool` at `:109`/`:113`).                                                                                                                                                                  |
| **merge gate** (`merge_gate.rs`, `review_requirement.rs`)                                                           | **NONE**                                                                                                                                            | **ZERO**     | **Asserted (§E):** the gate reads only `local_review_verdicts` at head (`merge_gate.rs:84`/`:159`, `review_requirement.rs:94`). No read of the new tables or the `kind` field is added. DB metadata, not graph mutation.                                                                                                                                                                                                                                                                     |
| **git-graph / workspace / rebase model** (`but_graph`, `but_rebase`, `but_workspace`, `WORKSPACE_MODEL.md` surface) | **NONE**                                                                                                                                            | **ZERO**     | This delta writes **SQLite drive-metadata only** (+ one committed-config descriptor field). It performs **no ref/object/oplog mutation**, no `graph_rebase::Editor` use, no `Workspace`/`RefInfo` derivation change. The derived PR view (§A.3) does a **read-only** `gix` commit walk for display; it never mutates the graph. (Same `DryRun`-irrelevant argument as §B: local cache ≠ refs/objects/oplog.)                                                                                 |

**Explicit assertions:** merge-gate blast radius = **ZERO**; git-graph/workspace/rebase blast radius = **ZERO**. Both follow from the safe seam (§E) + the local-cache-only write discipline (§B). The additive `kind` config field is enforcement-neutral — it does not enter `GovConfig.principals` and no gate reads it.

---

## G. Risk delta — name your leaks, never mitigated-closed

Continuing the register from `07-technical-risks.md` (R1–R14) and the v1.4.0 delta (R15–R17). These are named in the PRD's **"name your leaks, never quietly mitigated-closed"** doctrine — each carries a residual that the build MUST NOT present as closed.

| #       | Risk                                                                                                      | Severity                                  | Statement                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      | Mitigation / residual                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| ------- | --------------------------------------------------------------------------------------------------------- | ----------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **R18** | **Local-review forgeability — no independent engine re-read of the verdict store**                        | **High** (same class as R6)               | The local PR loop's "approved" status is recorded in `local_review_verdicts` by the governed `approve_review` (`forge.rs:534`); but **no CLI/engine path re-reads the verdict store to _re-verify_ a receipt out-of-band** — the audit receipt is **loop-sourced** (produced by the same loop that wrote it). An agent with DB/filesystem write can INSERT a forged approving row (the **exact R6 leak**, `03-data-schema.md` integrity caveat), and the new local-PR surface gives that forgery a friendlier UX (a "PR" that looks reviewed).                                                                                                                                                                                                                                                                                                                                                                                                                                                                 | **Mitigation:** approvals only via the governed `approve_review` action; the gate tests exercise the governed path. **Residual (accepted, named):** a direct DB write forges approval — **same accepted-leak class as the fence (R1)** and R6. The deferred closure is the R6 hardening (HMAC keyed by a repo-local admin secret, then Ed25519-signed review artifacts) **plus** an independent `but review verify` re-read that does not trust the writing loop. The build must NOT present the local PR as independently audited.                                                                                                                                                                                                                                                                                                                                                                                                                       |
| **R19** | **Agent-tag spoofability via `BUT_AGENT_HANDLE` re-export to impersonate a different declared principal** | **Medium** (Accepted residual, R2 class)  | The agent-PR tag (§A.4) is derived from the **opener principal's declared `kind` in committed `.gitbutler/permissions.toml`** (read at the target ref) — so an actor **cannot self-assert agent/human via a bare env var** (there is no `--kind` flag and no env input to the tag; `kind` is config-declared). **But** the env handle (`BUT_AGENT_HANDLE`) still selects _which_ declared principal acts: a sub-process that re-exports `BUT_AGENT_HANDLE` to a **different handle** (the **R2 identity residual**, `07-technical-risks.md`) acts **as that other declared principal and inherits its declared `kind`** — so if an `agent`-kind principal's handle is re-exported by a human (or vice-versa), the tag reflects the impersonated principal's declared kind. The mis-attribution is therefore **bounded to principals that already exist in committed config** (an actor cannot conjure an arbitrary kind), but impersonating a _different declared principal_ to borrow its kind is not closed. | **Mitigation:** the tag is computed from the **opener's declared config `kind`**, **never a caller arg** and never the env handle's mere presence (mirrors `04-api-design.md` "never from an agent-supplied claim"); in-band `--as` is denied (UC-AUTHZ-03); the `kind` field is read at the target ref (anti-self-escalation), so an actor cannot edit its own working-tree config to flip its kind. **Residual (accepted, named):** sub-process `BUT_AGENT_HANDLE` re-export to impersonate a _different declared principal_ (and borrow its kind) is **not** closed — **same accepted-leak class as R2.** Per-agent key-mint is the deferred hardening. The build must NOT present the tag as a trustworthy authorship attestation.                                                                                                                                                                                                                    |
| **R20** | **Comment-body injection into agent context**                                                             | **Medium**                                | `local_review_comments.body` (§A.2) is attacker-influenceable free text written by one agent principal and **read as context by another** (`list_comments`/`review_status`). A crafted body can attempt prompt-injection against a downstream agent that ingests review threads — the **same injection class** the v1.4.0 delta named for `message`/`unmet[]` (R15 there), now reaching agent context through comment bodies.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  | **Mitigation:** comment bodies are **data, never code** — the governance layer never interpolates them into a decision (unlike the closed-catalog `&'static str` denial fields), and the agent-tag control path is the **declared config `kind` + a dedicated `local_review_meta` row, not a comment body** (§A.4), so a comment body cannot reach the tag derivation. Downstream agent harnesses that surface comments to a model should bound/escape them (an **L2 harness concern**, out of GitButler's grip — Stance 6). **Residual (accepted, named):** GitButler stores and serves the raw body; it does not sanitize it for arbitrary downstream consumers. The build must NOT claim comment bodies are injection-safe.                                                                                                                                                                                                                            |
| **R21** | **`keep_reviews_local` is a trusted-desktop preference, not an authorization boundary**                   | **Medium** (Accepted residual, R12 class) | `keep_reviews_local` (§C) is a per-project `Project` preference under the **R12 trusted-desktop** model — **not** `administration:write`-gated, **not** ref-pinned committed config. An **untrusted write to the project store** (a compromised desktop session, or any process that can write `gitbutler-project` storage) can **flip it to `false`**, after which agent-authored PRs begin **mirroring to a public forge** (§D) instead of staying local — a confidentiality flip the governance layer does not check. **Sub-point (F-005, deferred-seam disclosure):** when mirroring is enabled, the §D `reviewer_principal → forge reviewer list` mapping **discloses internal governance principal identifiers to a public forge API**, with mapping-failure modes (no forge account for a principal, stale/renamed handle, many-to-one collapse) that can drop, mis-attribute, or leak the raw internal handle.                                                                                         | **Mitigation:** default is `true` (local) via `DefaultTrue`; the desktop human is the trusted fleet owner (R12) who owns the preference; while `true`, no principal identifier crosses to any forge. **Residual (accepted, named):** an untrusted project-store write flips the flag — **same accepted-leak class as R12** (trusted-desktop, not a checked grant); and the deferred mirror's principal→forge-identity mapping is a real disclosure + failure surface that must be designed fail-closed when built. The build must NOT present `keep_reviews_local` as an authorization boundary, nor the deferred mirror as a lossless bridge.                                                                                                                                                                                                                                                                                                            |
| **R22** | **Same-principal drive-layer forgery — self-assignment and self-resolve forge a clean drive narrative**   | **Medium**                                | The drive layer (assignments, comment threads) is read by the reconciler to decide "who reviewed" and "is it all-clear". Two same-principal forgeries exist at the _drive_ layer (the _gate_ still catches the verdict via its own distinct-from-author check, but the drive narrative is what an orchestrator and a human read): **(a)** an implementer **self-assigns** as its own reviewer (`assign_reviewer` with `reviewer == author`) → the PR object falsely reads "independently reviewed"; **(b)** a reviewer **posts a `changes_requested`-style thread and self-resolves it** (`resolve_thread` by the same principal that authored it) → the reconciler reads a forged "all-clean" signal and suppresses remediation for another party.                                                                                                                                                                                                                                                            | **Mitigation (drive-layer integrity, implemented at the `but-api` boundary):** `assign_reviewer` enforces `reviewer != author_principal_of_target_branch` (the gate's `require_distinct_from_author`, mirrored at the drive layer); `resolve_thread` requires the resolver to be the **thread author, the assigned reviewer, or a `reviews:write` holder** — a self-posted-and-self-resolved thread does not suppress the remediation signal for a distinct party. Proven by T-LPR-043 (self-assignment rejected) and T-LPR-044 (self-posted+self-resolved thread does not suppress another party's remediation signal). **Residual (accepted, named):** a single principal that legitimately holds both `reviews:write` and authorship can still act on both sides; the drive layer narrows _cross-principal_ forgery but cannot make a one-principal repo multi-party. The build must NOT present a single-principal drive trail as multi-party review. |
| **R23** | **DB-row forgery of the agent-tag derivation control path**                                               | **Medium**                                | The agent-PR tag is derived from the opener's declared config `kind` and cached in a dedicated `local_review_meta(target, "opener_principal", …)` row (§A.4), chosen specifically so the tag is **not** sourced from an attacker-influenceable comment body (which would be R20). But the `local_review_meta` row is itself a DB row: an actor with **direct DB/filesystem write** can INSERT/overwrite the opener row (subject to the `UNIQUE(target, key)` write-once-on-conflict) to forge the cached tag-derivation input — distinct from R20 (comment-body → _agent context_ injection); R23 is forgery of the _tag-derivation control path_ itself. (The committed-config `kind` source is itself read at the target ref and not forgeable in the working tree, but the cached opener row in the local DB is.)                                                                                                                                                                                           | **Mitigation:** the opener row is written **once** by the governed `request_review` via `INSERT … ON CONFLICT(target,key) DO NOTHING` (the `UNIQUE(target,key)` blocks a later overwrite through the governed path); the tag is never a caller arg, and the declared `kind` source is read at the target ref. **Residual (accepted, named):** a direct DB write that races the first opener insert, or edits the cached row out-of-band, forges the tag — **same accepted-leak class as R6/R18** (the verdict store is forgeable by direct DB write; so is this cached metadata row). The deferred closure is the same R6 integrity hardening (HMAC/Ed25519 over the local review artifacts, extended to the meta row). The build must NOT present the agent tag as a tamper-proof authorship attestation.                                                                                                                                                |

**Honesty-test note (mirrors `07-technical-risks.md` "risks the reviewer must treat as honesty tests"):** R18's loop-sourced-receipt forgeability, R19's tag spoofability (via impersonating a different declared principal), R21's preference-not-boundary flip, and R23's tag-control-path forgery must **stay named, never quietly "mitigated" into looking closed** — presenting any as a hardened boundary is the same misrepresentation class as R1/R6. R22's drive-layer distinct-from-author/resolver-identity constraints are real integrity checks (and are tested), but they narrow _cross-principal_ forgery only — they do not make a single-principal trail multi-party, and that residual stays named. R20 is an accepted L2/harness residual, not a closed boundary.

---

## H. Constraints recap (what Sprint 07 (LPR) must honor)

- **Net-additive:** 3 new tables, 6 new verbs, 1 new project field, 1 typed enum, 1 optional `kind` AUTHZ-config descriptor field — **no edits to existing tables, authorities, or the gate; the two replaced verbs (`comment_review`/`request_changes_review`) were stubs that wrote nothing.**
- **Frozen-aware:** lands as **Sprint 07 (LPR)** (human-directed slot; STEER renumbered 07→08 via N1); edits no frozen task file now; code deltas recorded here to apply when implemented (`05-delta-replan.md` §5 owns the N1 renumber + the ROADMAP `sprint_count` 8→10 I4-edit — live ROADMAP is at 8; Stage 2 adds both LPR=07 and STEER=08).
- **Reuse existing authorities:** `PullRequestsWrite` (open: `request_review`) / `ReviewsWrite` (review interactions: `assign_reviewer`, approve/request-changes) / `CommentsWrite` (comments) only — **no new `Authority` variant** (`authority.rs:11` is unchanged).
- **Two shipped review verbs are stubs LPR REPLACES:** `request_changes_review` (`forge.rs:551`) and `comment_review` (`forge.rs:569`) both authorize then return `task_contract_invalid`, writing nothing; LPR supplies their real writes (`changes_requested` state; `post_comment` → `local_review_comments`) — `post_comment` REPLACES the stubbed comment path, it does not wrap it.
- **Drive-layer integrity is at the `but-api` boundary (R22):** `assign_reviewer` enforces distinct-from-author; `resolve_thread` enforces resolver-identity (author / assigned reviewer / `reviews:write`).
- **Additive-only migrations:** all three `SchemaVersion::Zero`, appended to `MIGRATIONS` (`lib.rs:130`); older binaries tolerate them (`lib.rs:167`).
- **The agent tag's source-of-truth is the opener principal's DECLARED `kind` in committed `.gitbutler/permissions.toml`** (an additive optional `kind` field on `PrincipalWire`, `config.rs:424`, read at the target ref) — NOT handle-resolution (which cannot tell agent from human); the computed tag is cached in a dedicated `local_review_meta` row, never a comment-body sentinel (R23 ≠ R20); never a caller arg; the `kind` field changes no enforcement (no gate reads it; it does not enter `GovConfig.principals`).
- **Out of the git-graph/workspace/rebase model:** SQLite drive-metadata + one committed-config descriptor field + read-only display walks only; **ZERO** graph mutation (§F).
- **The safe seam is load-bearing (§E):** gate gates, new tables drive — proven by file:line and a no-read honesty grep over all three new tables.
