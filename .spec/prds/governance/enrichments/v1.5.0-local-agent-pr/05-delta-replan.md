---
title: Governance PRD v1.4.0 → v1.5.0 — Local Agent PR / Governed-Review Parity Delta-Replan
prd: governance
from_version: 1.4.0
to_version: 1.5.0
posture: net-additive enrichment — spec documents NOW; code lands as a NEW sprint (Sprint 07 LPR)
last_updated: 2026-06-21
status: planned (frozen-aware; sprint slot assigned by human directive — LPR = Sprint 07, STEER → Sprint 08)
---

# v1.4.0 → v1.5.0 — Local Agent PR / Governed-Review Parity Delta-Replan

**Freeze contract.** Sprints 01a–06b are FROZEN with in-flight agents. This plan **edits none of their task
files now**; it records (a) the code deltas v1.5.0 implies (the exact shapes — SQL, Rust structs, migration
ids, file:line — live in [03-technical-requirements-delta.md](./03-technical-requirements-delta.md)) and (b)
the additive edits to the frozen PRD index files — to apply **when the freeze lifts**, per the v1.4.0
precedent. Behavior ships as a **new Sprint 07 (LPR)** appended after the roadmap (§2).

> **🔴 SPRINT-SLOT DIRECTIVE (human instruction — overrides this enrichment's earlier auto-numbering).**
> The 00/01/03/04 section files of this enrichment were first drafted assuming LPR would *append after* STEER
> as **Sprint 08** (because the v1.4.0 STEER enrichment had already claimed Sprint 07 on the live ROADMAP —
> `v1.4.0-capability-aware-denials/05-delta-replan.md:141` I4 "✅ APPLIED … `sprint_count` 8→9"). **The human
> has since directed the opposite priority (instruction-precedence #1):** LPR takes slot **Sprint 07**, and the
> existing **STEER Sprint 07 is renumbered to Sprint 08**. This is applied here as the authoritative numbering;
> the "Sprint 08 (LPR)" wording in 00/01/03/04 is reconciled to "Sprint 07 (LPR)" (see §5 R-edits). Rationale
> (human): LPR is the higher-priority capability; STEER is independent — it depends on Sprints 02/04/05, **not**
> on LPR — so it renumbers cleanly to 08 with no dependency change. The actual ROADMAP/folder renumber is
> executed **downstream in the `/kb-sprint-plan` stage** (N1, §5); this enrichment states the directive and is
> internally consistent with it.

> **Re-grounded (frozen-aware).** Every code delta below is grounded in the shipped tree at the file:line cited
> in [03 §A–§H](./03-technical-requirements-delta.md). No "behavior-neutral" claim is made for anything the
> structural gates cannot see — the safe seam (§E of the tech-delta) is proven by file:line **and** a no-read
> honesty grep, not asserted.

> **Counting principle (carried from v1.3.0/v1.4.0).** Headline counts are recomputed honestly (§4), not held
> stable while the AC list grows.

---

## 0. Disposition summary — CODE DELTAS (derived from the tech-delta)

All deltas are **additive** and materialize as **Sprint 07 (LPR)**. None edits a frozen sprint now.

| ID | Area | Disposition | Touched crate(s) / file(s) | Apply when |
|---|---|---|---|---|
| **D1** | +3 additive `but-db` tables (`local_review_assignments`, `local_review_comments`, `local_review_meta`) + 3 `SchemaVersion::Zero` migrations + 3 structs + Handle/HandleMut pairs | code delta | `but-db` (`table/local_review_assignments.rs`, `table/local_review_comments.rs`, `table/local_review_meta.rs`, `table/mod.rs:10`, `lib.rs:130`) | Sprint 07 |
| **D2** | +6 additive `#[but_api(napi)]` verbs (`request_review`/`assign_reviewer`/`post_comment`/`list_comments`/`resolve_thread`/`review_status`) reusing `PullRequestsWrite` (open) / `ReviewsWrite` (assign) / `CommentsWrite` (+ branch-scoped reads) — **no new `Authority`** — plus the drive-layer integrity constraints (assign distinct-from-author; resolve resolver-identity); **`post_comment` REPLACES the stubbed `comment_review` (`forge.rs:569`) and the `changes_requested` write REPLACES the stubbed `request_changes_review` (`forge.rs:551`)** — both shipped verbs authorize then return `task_contract_invalid` writing nothing, so supplying their writes is additive | code delta | `but-api` (`legacy/forge.rs`, modeled on `approve_review` `:523`); `but-napi` regen | Sprint 07 |
| **D3** | +5 `but review *` CLI verbs (`request`/`assign`/`resolve`/`status`; extend `comment` with `--file/--line/--thread`) routed through `review_gate_cli_error` | code delta | `but` CLI (`command/legacy/forge/review.rs`, `args/`) | Sprint 07 |
| **D4** | PR lifecycle state **DERIVED** (commits + verdict-at-head + open assignments) — **no `local_pull_requests` table**; read-only `gix` walk, no graph mutation | code delta | `but-api` (`review_status`); read-only over `but_graph`/`gix` | Sprint 07 |
| **D5** | Agent-PR tag derived from the **opener principal's DECLARED `kind` in committed `.gitbutler/permissions.toml`** (read at the target ref — NOT from `BUT_AGENT_HANDLE` resolution, which cannot tell agent from human); the addition is **one optional `kind: Option<String>` field on `PrincipalWire`** (the existing `role: Option<String>` precedent) that **changes no enforcement** (it does not enter `GovConfig.principals`; no gate reads it). The computed `agent_authored` tag rides the derived PR object and is cached in a **dedicated `local_review_meta(target,'opener_principal')` row** (NOT a comment-body sentinel); never a caller arg | code delta | `but-authz` (`config.rs:424` `PrincipalWire` +`kind`), `but-api` (`review_status` derivation), `but-db` (`local_review_meta`) | Sprint 07 |
| **D6** | `Project.keep_reviews_local: DefaultTrue` (per-project operator preference under R12 trusted-desktop, default LOCAL — NOT `administration:write`-gated, NOT ref-pinned) + default-local wiring; older project files deserialize to local | code delta | `gitbutler-project` (`src/project.rs:72`/`:139`, `src/storage.rs`) | Sprint 07 |
| **D7** | `AssignmentState { Pending, Approved, ChangesRequested }` typed enum + boundary (de)serialization (the `Authority` parse/`name` round-trip pattern) — DB column stays `TEXT`; no new `Authority` variant | code delta | `but-authz` (typed enum at the boundary; `authority.rs:69`/`:94` pattern) | Sprint 07 |
| **D8** | `but-rules` auto "review-requested" hook (commit Trigger → Filter → Action opens a `pending` assignment) — **reuse** the shipped engine, no new rules mechanism | code delta | `but-rules` (Sprint 06b engine; additive hook) + `but-db` `workspace_rules` | Sprint 07 |
| **D9** | **Safe-seam build-gate grep**: the gate path (`merge_gate.rs` / `review_requirement.rs`) contains **NO** reference to any of the three new tables (the no-read honesty grep) | test delta | `but-authz/tests/invariant_build_gates.rs` (net-new pattern) | Sprint 07 |
| **D10** | **Remote-mirror seam — SPECIFIED, NOT BUILT** (rides the shipped `forge_reviews` + `but_forge::sync_reviews`; activated only when `keep_reviews_local == false`; the `reviewer_principal → forge reviewer` mapping discloses internal principals to a forge — R21 sub-point, design fail-closed when built) | design-for-later (named seam) | none built (anchors: `but-forge` `review.rs:1251`/`:1349`, `but-db` `forge_reviews.rs`) | deferred |
| **R18–R23** | Named new risks (loop-sourced-receipt forgeability; agent-tag spoofability incl. human-with-`BUT_AGENT_HANDLE` mis-tag; comment-body injection; keep-reviews-local preference-flip + deferred principal→forge disclosure; same-principal drive forgery; meta-row tag-control forgery) | risk delta | `10-technical-requirements/07-technical-risks.md` (fold-in I6) | freeze lifts |
| **I1–I6** | Additive edits to frozen PRD index files | doc delta | `README.md`, `03-functional-groups.md`, `ROADMAP.md`, `11-e2e-testing-criteria.md`, `07-technical-risks.md` | freeze lifts |
| **N1** | **STEER Sprint 07 → Sprint 08 renumber** (ROADMAP row + details + deps, `tasks/sprint-07-steer*` dir, v1.4.0 STEER enrichment refs) | numbering delta | v1.4.0 STEER artifacts + ROADMAP | Stage 2 (`/kb-sprint-plan`) |

Migration-tolerance is the criterion that makes D1 legal under the freeze: all three tables are
`SchemaVersion::Zero`, "migrations that older binaries can still tolerate after the migration runs, such as
adding tables" (`crates/but-db/src/lib.rs:167`). The safe seam (D9 / §E) is what makes the whole slice legal:
the merge gate reads only `local_review_verdicts`, so nothing here can flip a deny into an allow.

---

## 1. Code deltas — narrative (grounded; file:line in the tech-delta)

### D1 — three additive `but-db` tables + three migrations (`03 §A`)
`local_review_assignments` (`id, target, reviewer_principal, state TEXT, assigned_at`),
`local_review_comments` (`id, target, author_principal, body, file?, line?, thread_id, resolved, created_at`),
and `local_review_meta` (`target, key, value, created_at`, `PRIMARY KEY(target,key)` — the dedicated
opener/tag row, §A.4) are new per-table modules exporting `pub(crate) const M: &[M<'static>]` (the shape
`local_review_verdicts.rs:7` exports), registered in `crates/but-db/src/table/mod.rs` (alongside
`local_review_verdicts` at `mod.rs:10`) and appended to `MIGRATIONS` in `crates/but-db/src/lib.rs:130` (the
slice that already lists `table::local_review_verdicts::M` at `lib.rs:142`). All three are
`SchemaVersion::Zero` (the value `local_review_verdicts.rs:9` uses) with fresh monotonic-by-creation-time ids
(`20260621120000`, `20260621120100`, `20260621120200`), sorted by `migration::run`'s
`sort_by_key(|m| m.up_created_at)` (`migration.rs:39`). `principal_id`/`target` are deliberately **un-FK'd**
(principals live in committed config, not a table — exactly as `local_review_verdicts` stores
`principal_id TEXT` un-FK'd). This is the `local_review_verdicts` class (disposable, principal-scoped, local),
**not** the runtime-cleared remote-cache class (`forge_reviews`, `DELETE FROM forge_reviews` at
`forge_reviews.rs:153`).

### D2 — six additive `#[but_api(napi)]` verbs reusing existing authorities (`03 §B`)
All in `crates/but-api/src/legacy/forge.rs`, modeled **exactly** on the shipped `approve_review`
(`forge.rs:523`–`:546`): `authorize_branch_action(&repo,&branch,Authority::X)?` (`forge.rs:47`) **before** any
`.await`, then a local-cache write. `request_review` gates on `PullRequestsWrite` (the open-PR authority, the
same the shipped `publish_review` authorizes at `forge.rs:488`); **`assign_reviewer` gates on `ReviewsWrite`**
(`authority.rs:23`) — assignment is a *review interaction*, the authority the shipped
`approve_review`/`request_changes_review` authorize (`forge.rs:526`/`:558`), **not** the PR-open authority;
`post_comment`/`resolve_thread` gate on `CommentsWrite` (the same `comment_review` authorizes at
`forge.rs:574`); `list_comments`/`review_status` are **branch-scoped reads** (no write authority). **No new
`Authority` variant** — the route→Authority table stays closed (`authority.rs:11` unchanged), so `02-roles.md`
role presets already grant/deny these (`write` role holds `pull_requests:write` + `comments:write` +
`reviews:write`, `authority.rs:343`). **Drive-layer integrity at the `but-api` boundary:** `assign_reviewer`
enforces `reviewer != author_principal_of_target_branch` (the gate's `require_distinct_from_author`, mirrored
at the drive layer); `resolve_thread` requires the resolver to be the thread author, the assigned reviewer, or
a `reviews:write` holder — both named as R22 and tested (T-LPR-043/044). Like `approve_review`, the writes
touch **only** the local SQLite cache (`ctx.db.get_cache_mut()`) — **no `DryRun` guard** (RULES.md "dry runs
must not persist refs, objects, or oplog" does not bind a local-cache row). **Two shipped review verbs are
CONTRACT STUBS LPR REPLACES:** `comment_review` (`forge.rs:569`) authorizes `CommentsWrite` (`:574`) then
returns `task_contract_invalid` (`:575`–`:581`) **writing nothing** — `post_comment` REPLACES (does not wrap)
it, persisting to `local_review_comments`; `request_changes_review` (`forge.rs:551`) authorizes `ReviewsWrite`
(`:558`) then returns `task_contract_invalid` (`:559`–`:565`) writing nothing — LPR implements its real
`changes_requested` write. v1.5.0 adds **no approval verb**; the approve path stays the shipped `approve_review`
(`forge.rs:523`), and `request_changes_review` (`forge.rs:551`) is the stub LPR fills, not reuse.

### D3 — five `but review *` CLI verbs (`03 §B`)
In `crates/but/src/command/legacy/forge/review.rs` alongside the shipped `approve`/`request_changes`/`comment`/
`close` (`review.rs:20`/`:37`/`:55`/`:73`): add `request` (→ `request_review`), `assign` (→ `assign_reviewer`),
`resolve` (→ `resolve_thread`), `status` (→ `review_status`); **re-point the existing `comment` verb at the real
`post_comment`** (replacing its route to the stubbed `comment_review`) and extend it with
`--file`/`--line`/`--thread`. Each prints the ref-pin caveat where it writes config-visible
state and routes errors through the existing `review_gate_cli_error` serializer (`review.rs:89`). Verb
definitions live in `crates/but/src/args/` (not `but-clap`). CLI tests are **happy-path only** (RULES.md).

### D4 — PR lifecycle DERIVED, not stored (`03 §A.3`)
There is **no `local_pull_requests` table.** `review_status` computes the PR view at query time over the
already-present sources — the branch's commits ahead of base (a **read-only** `gix` graph walk, NO mutation),
`local_review_verdicts.list_by_target(target)` filtered to `head_oid == current head` (the **exact** query
`merge_gate` runs, `merge_gate.rs:84`), open `pending` `local_review_assignments` + unresolved
`local_review_comments`, plus the `local_review_meta` opener row for the tag — yielding a derived status ∈
{Draft, AwaitingReview, ChangesRequested, Approved, Mergeable}. `Approved`/`Mergeable` is a **presentation
label only**; the actual merge decision stays `enforce_merge_gate` (§E), which re-derives verdict-at-head
itself and never reads the derived view. This is the same lossy-presentation discipline `WORKSPACE_MODEL.md`
mandates for `Workspace`/`RefInfo`.

### D5 — agent-PR tag derived from the opener's DECLARED `kind`, cached in a dedicated meta row (`03 §A.4`)
The agent-vs-human distinction **does not exist in the resolved principal**: `resolve_principal`
(`crates/but-authz/src/authorize.rs:67`–`:90`) keys **solely** on `BUT_AGENT_HANDLE` and builds
`Principal::new(principal_id, authorities, groups)`; `Principal` is `{ id, authorities, groups }`
(`crates/but-authz/src/principal.rs:82`) with **no `is_agent`/`PrincipalKind` discriminator**, and the forge
write path resolves **every** caller — agent and human — through the same `resolve_principal_from_env`
(`crates/but-api/src/legacy/forge.rs:58`). So "resolved from `BUT_AGENT_HANDLE`" is true of **all** governed
principals and **cannot** tell agent from human — deriving the tag from it would be a fabrication (the v1.4.0
`Denial.unmet` class). Instead the tag derives from a **new additive, optional `kind` field on the principal
entry in committed `.gitbutler/permissions.toml`** (`kind = "agent"` / `kind = "human"`) — riding `PrincipalWire`
(`crates/but-authz/src/config.rs:424`) as `#[serde(default)] pub kind: Option<String>`, exactly the existing
optional `role: Option<String>` precedent (`config.rs:427`); older committed files without `kind` deserialize
to `None` → human. It is **read at the target ref** like all governance config (`config.rs:23`–`:25`,
anti-self-escalation) and **changes no enforcement** — it does NOT enter
`GovConfig.principals: BTreeMap<PrincipalId, AuthoritySet>` (`config.rs:85`) and **no gate reads it**.
When `request_review` opens the first review for a `target`, the opener principal id is recorded in the
dedicated `local_review_meta` table as `(target, "opener_principal", <principal_id>)` via an
`INSERT … ON CONFLICT(target,key) DO NOTHING` (the `UNIQUE(target,key)` makes the opener write-once per
target). The derivation (`review_status`) resolves that opener's committed-config entry at the target ref and
sets `agent_authored = true` **iff that entry declares `kind = "agent"`** (the env handle only resolves *which*
principal acted; the declared `kind` says whether it is an agent). The computed tag rides the **derived PR
object** (mirroring `ForgeReview.labels`, `forge_reviews.rs:52`) and is **cached** in the `local_review_meta`
row. It is **auto-set, never caller-supplied** (no `--agent`/`--kind` flag) — exactly as `approve_review`
derives `principal_id` from `authorize_branch_action(...)?` rather than a parameter (`forge.rs:526`–`:539`).
**The tag does NOT live in a comment body** — a comment body is attacker-influenceable free text (R20), so
sourcing the tag from it would let any comment-write actor forge the opener; the dedicated `local_review_meta`
row keeps the tag-cache off the comment surface. Spoofability via `BUT_AGENT_HANDLE` re-export to impersonate a
**different declared principal** (borrowing its kind) is named as **R19 (§3)**; direct DB-row forgery of the
cached `local_review_meta` opener row is named as **R23 (§3)** — distinct from R20. Neither is closed.

### D6 — `Project.keep_reviews_local` per-project setting (`03 §C`)
A new `#[serde(default)] pub keep_reviews_local: DefaultTrue` on `gitbutler_project::Project`
(`project.rs:72`), alongside `forge_override`/`preferred_forge_user` (`project.rs:129`/`:134`), defaulted in
`default_with_id` (`project.rs:139`). `DefaultTrue` means the default is **local** and older project files
without the field deserialize to local — reusing the `#[serde(default)] + DefaultTrue` combination
`ok_with_force_push` relies on at `project.rs:106` (the `DefaultTrue` type is imported at `project.rs:10`).
**Note the precedent correctly:** `force_push_protection`/`husky_hooks_enabled` at `project.rs:109`/`:113` are
plain `bool` defaulting `false` — the **WRONG** precedent here; `ok_with_force_push` (`project.rs:106`) is the
real `DefaultTrue` precedent. It is a **local operator preference** about where review *artifacts* go — **not**
governed ref-pinned config (that is for `permissions.toml`/`gates.toml`, which gate *decisions*) and **not**
`administration:write`-gated — owned by the desktop human under the **R12 trusted-desktop** model. Remote
mirroring (D10) is reachable only when `keep_reviews_local == false`. An untrusted project-store write that
flips it (→ agent PRs mirror to a public forge) is the named accepted residual **R21 (§3)**. The `but-*` skill
contract: *"on governed-project init, if unset, set `keep_reviews_local = true`"* — a skill-side default
(belt-and-suspenders, since the field already defaults true), not a governance enforcement.

### D7 — `AssignmentState` typed at the boundary (`03 §A.1`, §B blast-radius)
`local_review_assignments.state` is stored as `TEXT` (migration-tolerant, matching `LocalReviewVerdict.verdict:
String`); the `but-authz`/`but-api` layer maps it to a pure `enum AssignmentState { Pending, Approved,
ChangesRequested }` on read/write via a parse/`name` round-trip (the `Authority` shape discipline,
`authority.rs:69`/`:94`). **No new `Authority` variant, no change to `authorize`/`effective_authority`.**

### D8 — `but-rules` auto "review-requested" hook (`03 §B`, UC-LPR-06)
The existing `but-rules` (Trigger → Filter → Action) engine — the surface Sprint 06b exposes — hosts a new
hook: trigger = a commit on a watched branch; filter = branch/principal; action = open a `pending`
`local_review_assignments` row for the configured reviewer (the same drive-only row UC-LPR-01 defines). It
**reuses** the shipped engine; it does **not** re-build a rules mechanism. The auto-opened assignment is
drive-only — it blocks no commit and no merge.

### D9 — safe-seam build-gate grep (`03 §E`)
A net-new honesty grep asserts the gate path (`merge_gate.rs`, `review_requirement.rs`) contains **no
reference** to any of `local_review_assignments`/`local_review_comments`/`local_review_meta` — the same
honesty-grep discipline as the AUTHORITY_POSITIVE_PATTERN gate in
`but-authz/tests/invariant_build_gates.rs`. A static grep proves the single-symbol no-read; the **runtime**
equivalence (forged drive layer ≡ empty drive layer ⇒ identical merge decision) is proven by integration test
(T-LPR-040/041/042), not the grep.

### D10 — remote-mirror seam (specified, NOT built) (`03 §D`)
The local review object is designed to be mirrorable to a real GitHub/GitLab PR behind D6, via the shipped
`ForgeReview` + `but_forge::create_forge_review` (`review.rs:1251`) / `sync_reviews` (`review.rs:1349`)
bridge. The field mapping is specified in `03 §D`; **no mirroring code lands in Sprint 07.** A future
`mirror_local_review` verb (gated by `PullRequestsWrite`, active only when `keep_reviews_local == false`) can
map the rows without schema change. The mapping's `reviewer_principal → forge reviewer list` step **discloses
internal governance principals to a public forge** and has mapping-failure modes (no forge account, stale
handle, many-to-one collapse) — named as the **R21 (§3)** deferred-seam sub-point; it must be designed
fail-closed for unmapped principals when built, the same forward-seam discipline the merge gate already
follows.

---

## 2. Proposed Sprint 07 — LPR: Local Agent PR / Governed-Review Parity

Folder `sprint-07-local-agent-pr` (the new **slot 07** per the human directive). **This lands as Sprint 07;
the existing STEER sprint renumbers 07→08 (executed in the `/kb-sprint-plan` stage — see N1, §5).** Appended
after Sprint 06b. **CLI/backend-first** (MGMT desktop render of the local PR is deferred — see
[01-scope-delta.md](./01-scope-delta.md) Out-of-scope). **Depends on** Sprint 01b (the `approve_review`
verdict write + the merge gate), Sprint 04 (merge strictness), Sprint 05 (`but perm`/`but group` CLI surface
convention + persisted config), and the shipped `but-rules` engine (Sprint 06b) for the auto-hook. **Does NOT
depend on STEER (Sprint 08)** — the renumber is clean.

**Human Testing Gate.** An agent principal opens a local review on a feature branch (no remote PR is created
while `keep_reviews_local` is true); a reviewer principal **distinct from the branch author** is assigned
(`but review request`/`assign`) and posts a file/line comment (`but review comment --file --line --thread`);
`but review status` shows the assignment, the open comment thread, the derived lifecycle, and an
`agent-authored` tag (sourced from the `local_review_meta` opener row); the reviewer approves
(`but review approve`) and the orchestrator merges through the **unchanged** merge gate; a self-assignment is
rejected and an unauthorized self-resolve cannot clear another party's thread; and a fully forged / empty set
of `local_review_assignments` + `local_review_comments` rows yields an **identical** merge-gate decision (the
safe seam, proven by build-gate grep + test).

**Test Steps** (from [04-e2e-testing-criteria.md](./04-e2e-testing-criteria.md)): per T-LPR-001..044 (+ the
hand-driven full-local-loop human-gate T-LPR-029h).

**Tasks (proposed — expanded by `/kb-sprint-tasks-plan`):**

| ID | Title | Agent | Maps to (Dxx) | Maps to (UC / T-LPR) |
|----|-------|-------|---------------|----------------------|
| LPR-001 | `local_review_assignments` + `local_review_comments` + `local_review_meta` tables + 3 `SchemaVersion::Zero` migrations + 3 structs + Handle/HandleMut pairs (`list_by_target`, upsert/insert, `set_state`/`set_resolved`, `list_by_thread`, meta `upsert_if_absent`/`get`) | rust-implementer | D1 | UC-LPR-01/02/04 · T-LPR-001/008/019 |
| LPR-002 | `AssignmentState { Pending, Approved, ChangesRequested }` typed enum + boundary (de)serialization (the `Authority` parse/`name` round-trip); column stays `TEXT`; **no** new `Authority` variant | rust-implementer | D7 | UC-LPR-01 · T-LPR-005 |
| LPR-003 | `request_review` (`PullRequestsWrite`, +`local_review_meta` opener row) / `assign_reviewer` (`ReviewsWrite`, **distinct-from-author** at the boundary) `#[but_api(napi)]` + `but review request`/`assign` CLI; implement the real `changes_requested` write (stub today); structured `perm.denied` + exit 1 on missing authority; self-assignment rejected | rust-implementer | D2, D3 | UC-LPR-01 · T-LPR-002/005/006/007/043 |
| LPR-004 | `post_comment`/`list_comments`/`resolve_thread` `#[but_api(napi)]` (`CommentsWrite` writes; **resolver-identity** constraint on resolve; branch-scoped reads) + `but review comment --file/--line/--thread`/`comments`/`resolve` CLI; **`post_comment` REPLACES the stubbed `comment_review` (`forge.rs:569`)** (re-point the `comment` CLI verb at it); local-cache write, no DryRun guard | rust-implementer | D2, D3 | UC-LPR-02 · T-LPR-008..013/044 |
| LPR-005 | `review_status` **derived** PR lifecycle (commits + verdict-at-head + open assignments; read-only `gix` walk, NO mutation) + the agent-PR tag derivation from the **opener principal's declared `kind` in committed `permissions.toml`** (the additive optional `kind` field on `PrincipalWire`, read at the target ref — NOT handle-resolution, NOT a comment body), cached in the dedicated `local_review_meta` row | rust-implementer | D4, D5 | UC-LPR-01/04/05 · T-LPR-003/004/019..024 |
| LPR-006 | `Project.keep_reviews_local: DefaultTrue` (per-project operator preference — NOT `administration:write`-gated, NOT ref-pinned; the `ok_with_force_push` `DefaultTrue` precedent, not the plain-`bool` `force_push_protection`) + default-local wiring + remote-mirror **gate** (the mirror path is NOT built — named seam only; principal→forge disclosure named under R21) | rust-implementer | D6, D10 | UC-LPR-03 · T-LPR-014..018 |
| LPR-007 | `but-rules` auto "review-requested" hook (commit Trigger → Filter → Action opens a `pending` assignment), reusing the Sprint-06b Trigger→Filter→Action engine — no new rules mechanism | rust-implementer | D8 | UC-LPR-06 · T-LPR-030..034 |
| LPR-008 | **Reconciler read-API**: `review_status` serves the full drive state (assignments + unresolved comments + verdict-at-head) in one payload, so two orchestrators converge; two-read agreement proof | rust-implementer | D2, D4 | UC-LPR-05 · T-LPR-024..028 |
| LPR-009 | **Safe-seam invariant**: net-new build-gate honesty grep (gate path has NO ref to the 3 new tables) + the forged-vs-empty + inverse integration tests (drive metadata alone never lands; only verdict-at-head flips) | rust-reviewer | D9 | UC-LPR-07 · T-LPR-035..042 |
| LPR-010 | TS SDK regen (`pnpm build:sdk && pnpm format`) + N-API audit (R14 — the verbs ARE `but-api` fns) + happy-path CLI tests; honesty/anti-fakeability greps (tag-not-an-enforcement-key; tag sourced from `local_review_meta`, not a comment body); drive-layer-integrity proofs (self-assignment rejected T-LPR-043; unauthorized self-resolve cannot suppress a signal T-LPR-044) | rust-reviewer | D2, D3, D5 | UC-LPR-01/02/04/07 · T-LPR-022/038/043/044 |
| LPR-011 | Reconciler usage-model doc + the `but-*` skill contract (`keep_reviews_local=true` on governed-project init) — the skill workflow auto-sets local; skills *implementation* is OUT of this sprint (documented contract only) | rust-implementer / docs | D6 | UC-LPR-05 · T-LPR-029/029h |

**PRD coverage:** UC-LPR-01..07 / T-LPR-001..044 (+ T-LPR-029h). Every one of the 40 LPR ACs maps to ≥1 task
via the T-LPR references above.

---

## 3. Risk delta (R18–R23) — reconciled into the register

Per the PRD's "name your leaks, never quietly mitigated-closed" doctrine (R1/R6/R14), v1.5.0 adds six named
risks. Fully stated in [03-technical-requirements-delta.md §G](./03-technical-requirements-delta.md); folded
into `10-technical-requirements/07-technical-risks.md` as I6.

| Risk | Severity | Statement | Mitigation / residual (accepted, named) |
|---|---|---|---|
| **R18 — Local-review forgeability (no independent engine re-read)** | High (R6 class) | The local PR loop's "approved" status is recorded in `local_review_verdicts` by the governed `approve_review` (`forge.rs:534`), but **no CLI/engine path re-reads the verdict store to re-verify a receipt out-of-band** — the audit receipt is loop-sourced. An agent with DB/filesystem write can INSERT a forged approving row (the exact R6 leak); the new local-PR surface gives that forgery a friendlier UX (a "PR" that looks reviewed). | Approvals only via the governed `approve_review`; the gate tests exercise the governed path. **Residual:** a direct DB write forges approval — same accepted-leak class as R1/R6. Deferred closure = the R6 hardening (HMAC → Ed25519-signed review artifacts) **plus** an independent `but review verify` re-read that does not trust the writing loop. The build must NOT present the local PR as independently audited. |
| **R19 — Agent-tag spoofability via `BUT_AGENT_HANDLE` re-export to impersonate a DIFFERENT declared principal** | Medium (R2 residual) | The agent-PR tag (D5) is derived from the **opener principal's declared `kind` in committed `.gitbutler/permissions.toml`** (read at the target ref) — so an actor **cannot self-assert agent/human via a bare env var** (no `--kind` flag, no env input to the tag; `kind` is config-declared). **But** `BUT_AGENT_HANDLE` still selects *which* declared principal acts: a sub-process that re-exports it to a **different handle** (the R2 identity residual) acts **as that other declared principal and inherits its declared `kind`** — so an `agent`-kind handle re-exported by a human (or vice-versa) makes the tag reflect the impersonated principal's kind. The mis-attribution is **bounded to principals already in committed config** (an actor cannot conjure an arbitrary kind), but impersonating a *different declared principal* to borrow its kind is not closed. | Tag computed from the opener's **declared config `kind`**, **never a caller arg** and never the env handle's mere presence ("never from an agent-supplied claim"); `--as` is denied (UC-AUTHZ-03); `kind` is read at the target ref (anti-self-escalation), so an actor cannot flip its own kind in the working tree. **Residual:** sub-process `BUT_AGENT_HANDLE` re-export to impersonate a *different declared principal* (and borrow its kind) is not closed — same accepted-leak class as R2; per-agent key-mint is the deferred hardening. The build must NOT present the tag as a trustworthy authorship attestation. |
| **R20 — Comment-body injection into agent context** | Medium | `local_review_comments.body` (D1/`03 §A.2`) is attacker-influenceable free text written by one agent principal and **read as context by another** (`list_comments`/`review_status`). A crafted body can attempt prompt-injection against a downstream agent that ingests review threads — the same injection class the v1.4.0 delta named for `message`/`unmet[]` (R15 there), now reaching agent context through comment bodies. | Bodies are **data, never code** — the governance layer never interpolates them into a decision, and the opener/tag control path is a **dedicated `local_review_meta` row, not a comment body** (D5), so a comment body cannot reach the tag derivation. Bounding/escaping is an **L2 harness concern** (Stance 6, out of GitButler's grip). **Residual:** GitButler stores and serves the raw body; it does not sanitize it for arbitrary downstream consumers. The build must NOT claim comment bodies are injection-safe. |
| **R21 — `keep_reviews_local` is a trusted-desktop preference, not an authorization boundary (+ deferred principal→forge disclosure)** | Medium (R12 class) | `keep_reviews_local` (D6/`03 §C`) is a per-project `Project` preference under the **R12 trusted-desktop** model — not `administration:write`-gated, not ref-pinned. An **untrusted write to the project store** can flip it to `false`, after which agent PRs **mirror to a public forge** (D10). **Sub-point (deferred seam):** when mirroring is on, the `reviewer_principal → forge reviewer list` mapping (`03 §D`) **discloses internal governance principal identifiers to a public forge**, with mapping-failure modes (no forge account, stale/renamed handle, many-to-one collapse). | Default is `true` (local) via `DefaultTrue`; the desktop human is the trusted fleet owner (R12); while `true`, no principal crosses to a forge. **Residual:** an untrusted project-store write flips the flag — same accepted-leak class as R12; the deferred mirror's principal→forge-identity mapping is a real disclosure + failure surface to be designed fail-closed when built. The build must NOT present `keep_reviews_local` as an authorization boundary, nor the deferred mirror as a lossless bridge. |
| **R22 — Same-principal drive-layer forgery (self-assignment / self-resolve)** | Medium | The drive layer (assignments, comment threads) is read by the reconciler to decide "who reviewed" and "is it all-clear". Two same-principal forgeries at the *drive* layer (the *gate* still catches the verdict via its own distinct check): **(a)** an implementer **self-assigns** as its own reviewer → the PR object falsely reads "independently reviewed"; **(b)** a reviewer **posts a `changes_requested` thread and self-resolves it** → the reconciler reads a forged "all-clean" signal and suppresses remediation for another party. | **Mitigation (implemented at the `but-api` boundary):** `assign_reviewer` enforces `reviewer != author_principal_of_target_branch`; `resolve_thread` requires the resolver to be the thread author, the assigned reviewer, or a `reviews:write` holder — tested by T-LPR-043 / T-LPR-044. **Residual:** a single principal that legitimately holds both `reviews:write` and authorship can act on both sides; the constraints narrow *cross-principal* forgery but cannot make a one-principal repo multi-party. The build must NOT present a single-principal drive trail as multi-party review. |
| **R23 — DB-row forgery of the agent-tag derivation control path** | Medium | The agent-PR tag is derived from a dedicated `local_review_meta(target,'opener_principal',…)` row (D5/`03 §A.4`), chosen so the tag is **not** sourced from an attacker-influenceable comment body (R20). But the meta row is itself a DB row: an actor with **direct DB/filesystem write** can INSERT/overwrite the opener row to forge the tag-derivation input — distinct from R20 (comment-body → *agent context*); R23 is forgery of the *tag-derivation control path*. | The opener row is written **once** by the governed `request_review` via `INSERT … ON CONFLICT(target,key) DO NOTHING` (the `UNIQUE(target,key)` blocks a governed-path overwrite); the tag is never a caller arg. **Residual:** a direct DB write that races the first insert, or edits the row out-of-band, forges the tag — same accepted-leak class as R6/R18. Deferred closure = the same R6 integrity hardening (HMAC/Ed25519) extended to the meta row. The build must NOT present the agent tag as a tamper-proof authorship attestation. |

**Honesty-test note (mirrors `07-technical-risks.md`).** R18's loop-sourced-receipt forgeability, R19's tag
spoofability, R21's preference-not-boundary flip, and R23's tag-control-path forgery must **stay named, never
quietly "mitigated" into looking closed** — presenting any as a hardened boundary is the same misrepresentation
class as R1/R6. R22's drive-layer distinct-from-author/resolver-identity constraints are real integrity checks
(tested by T-LPR-043/044) but narrow *cross-principal* forgery only — they do not make a single-principal trail
multi-party, and that residual stays named. R20 is an accepted L2/harness residual, not a closed boundary. This
delta adds **+6** to the risk register (17 → 23), not "+0".

---

## 4. Count reconciliation (v1.4.0 → v1.5.0)

Baseline from the v1.4.0 delta-replan §4: v1.4.0 = **6 groups / 23 UCs / 161 ACs / 160 criteria / 17 risks**;
ROADMAP = **9 sprints**. LPR adds the **LPR** group (UC-LPR-01..07).

| Metric | v1.4.0 | Δ | **v1.5.0** |
|---|---|---|---|
| Functional Groups | 6 | +1 (LPR) | **7** |
| Use Cases | 23 | +7 (UC-LPR-01..07) | **30** |
| Acceptance Criteria | 161 | +40 (LPR) | **201** |
| ↳ LPR per UC | — | 7·6·5·5·6·5·6 | **40** |
| Testing Criteria | 160 | +45 | **205** |
| ↳ integration-test | 85 | +33 | **118** |
| ↳ api-contract | 13 | +6 | **19** |
| ↳ build-gate | 22 | +3 | **25** |
| ↳ human-gate | 0 | +1 | **1** |
| ↳ e2e-automated | 2 | +2 | **4** |
| ↳ component-test | 38 | 0 | **38** |
| Risk register | 17 | **+6** (R18–R23) | **23** |
| Sprints (ROADMAP) | 9 | +1 (LPR slot 07; STEER → 08 via N1) | **10** |

**Type-tally check:** 118 + 19 + 25 + 38 + 1 + 4 = **205** ✓.
**LPR per-UC AC tally:** 7 + 6 + 5 + 5 + 6 + 5 + 6 = **40** ✓ (the two new criteria T-LPR-043/044 reference
*existing* ACs — UC-LPR-01 AC-2 and UC-LPR-02 AC-3 — so the AC count is unchanged at 40).
**LPR criteria-row tally:** UC-LPR-01 = 8 (7 one-per-AC + 1 self-assignment-rejected) · 02 = 7 (6 one-per-AC +
1 self-resolve) · 03 = 5 · 04 = 5 · 05 = 7 (6 one-per-AC + 1 human-gate) · 06 = 5 · 07 = 8 (6 one-per-AC + 2
capstone) = **45** ✓ (the +5 over one-per-AC: T-LPR-029h human-gate, the two drive-layer-integrity proofs
T-LPR-043 self-assignment + T-LPR-044 self-resolve, and the two UC-LPR-07 capstones T-LPR-041 forged-vs-empty
and T-LPR-042 inverse).
**Per-AC coverage: 40/40** — every LPR AC carries ≥1 T-LPR criterion (see [04-e2e-testing-criteria.md](./04-e2e-testing-criteria.md) "Counting").

> **Sprint-count framing (two baselines, same end state — 10).** This table uses the **v1.4.0 enrichment
> baseline of 9** (the v1.4.0 enrichment counts STEER as sequence #9): N1 renumbers STEER 07→08 with **no count
> change**, and LPR adds the net +1 → **10**. Against the **live `ROADMAP.md` (`sprint_count: 8`)** the arithmetic
> differs but lands identically: the v1.4.0 STEER I4 "APPLIED 8→9" never actually wrote a STEER row into
> `ROADMAP.md` (STEER exists only in `tasks/sprint-07-steer*` + the v1.4.0 enrichment), so Stage 2 adds **both**
> the STEER row (08) **and** the LPR row (07), taking the live `sprint_count` **8 → 10**. Either way the final
> ROADMAP holds **10** sprints. See [03-technical-requirements-delta.md](./03-technical-requirements-delta.md) §15
> for the live-tree grounding.

---

## 5. Integration edits — to apply downstream

### N1 — STEER Sprint 07 → Sprint 08 renumber (a NAMED Stage-2 action, `/kb-sprint-plan`)
Per the human directive, **before** LPR is inserted as Sprint 07, the existing STEER sprint is renumbered
07→08. This is a **numbering-only** change — STEER's scope, ACs, criteria, and dependencies (02/04/05) are
untouched — executed downstream in the `/kb-sprint-plan` stage, not by editing a frozen sprint's content now:
- **ROADMAP.md** — the Sprint 07 (STEER) row in the Sprint Sequence table + its Per-Sprint-Details block +
  dependency edges: `07` → `08`; anchor/slug `sprint-07-steer*` → `sprint-08-steer*`. Append-style renumber.
- **`tasks/sprint-07-steer-capability-aware-denials/`** → **`tasks/sprint-08-steer-capability-aware-denials/`**
  (directory rename); the SPRINT.md + task files inside that reference "Sprint 07" → "Sprint 08".
- **v1.4.0 STEER enrichment** (`enrichments/v1.4.0-capability-aware-denials/`) — `05-delta-replan.md` §2/§I4
  and `README.md` references to "Sprint 07 (STEER)" → "Sprint 08 (STEER)"; the v1.4.0 `sprint_count 8→9` note
  annotated as not-yet-landed in the live `ROADMAP.md` (still `sprint_count: 8`) and superseded by the Stage-2
  `8 → 10` write that lands **both** the STEER row (08) and the LPR row (07).

### I4 (paired with N1) — insert LPR as Sprint 07 (Stage 2, `/kb-sprint-plan`)
**After N1**, add the **Sprint 07 (LPR)** row to ROADMAP.md (slug `sprint-07-local-agent-pr`) + its
Per-Sprint-Details block + dependency edges (depends on Sprint 01b/04/05; the `but-rules` engine from 06b).
Against the **live `ROADMAP.md` (`sprint_count: 8`)** Stage 2 lands **both** the STEER row (08, via N1) and the
LPR row (07), taking the live `sprint_count` **8 → 10** (equivalently **9 → 10** against the v1.4.0 enrichment
baseline that already counts STEER as #9). The resulting ROADMAP order is `(01a, 01b, 02, 03, 04, 05, 06a, 06b,
07-LPR, 08-STEER)`.

### R-edits — reconcile THIS enrichment's section files to "Sprint 07 (LPR)"
The 00/01/03/04 files were drafted "Sprint 08 (LPR)" under the earlier append-after-STEER assumption. Per the
directive, reconcile each to **"Sprint 07 (LPR)"** with a one-line note that STEER renumbers 07→08 (keep the
honest record that the *earlier* auto-numbering said 08 and the human reassigned it to 07):
- `00-overview.md` (the "Sprint 07 (LPR) in its own numbering" / frozen-aware note),
- `01-scope-delta.md` (the closing "new appended sprint … Sprint 07 (LPR)"),
- `03-technical-requirements-delta.md` (the "Grounding correction" + Freeze contract + §H — "Sprint 08 (LPR)"
  → "Sprint 07 (LPR)"; retain the record that the auto-numbering said 08 and the human reassigned to 07),
- `04-e2e-testing-criteria.md` (the intro "materialize as Sprint 08 (LPR)" + the "Frozen-aware" line + the
  closing maintenance note → "Sprint 07 (LPR)"; STEER moves to 08).

### I1–I3, I5–I6 — Integration edits to frozen PRD index files (apply when freeze lifts; all append-style)
- **I1** — copy `02-uc-lpr.md` → top-level `13-uc-lpr.md` (no renumbering of existing files).
- **I2** — `03-functional-groups.md`: add the LPR row + Use-Case-Summary row (LPR · 7 · 40); totals → 7 groups
  / 30 UCs / 201 ACs.
- **I3** — `README.md`: Document Index row, Quick Stats (groups 6→7, UCs 23→30, ACs 161→201, criteria 160→205,
  risks 17→23, sprints 9→10), Version History row, `version: 1.5.0`.
- **I5** — fold T-LPR-001..044 (+ T-LPR-029h) into `11-e2e-testing-criteria.md` (+ count line → 205).
- **I6** — fold R18/R19/R20/R21/R22/R23 into `10-technical-requirements/07-technical-risks.md` (risk count
  17 → 23).

---

## 6. Verification checklist

- [ ] **N1** STEER renumbered 07→08 everywhere (ROADMAP row + details + deps; `tasks/sprint-08-steer*`; v1.4.0
  enrichment refs) as a Stage-2 `/kb-sprint-plan` action; STEER scope unchanged.
- [ ] **I4** LPR added as Sprint 07 (slug `sprint-07-local-agent-pr`) **after** N1; `sprint_count` 9→10 (live
  8→10); order reads `(…, 06b, 07-LPR, 08-STEER)`.
- [ ] **R-edits** 00/01/03/04 reconciled to "Sprint 07 (LPR)"; the honest "auto-numbered 08 → human-directed
  07, STEER → 08" record retained in each.
- [ ] **D1/D7** all three tables additive `SchemaVersion::Zero` (`lib.rs:167` tolerance); `AssignmentState`
  typed at the boundary; **no** new `Authority` variant (`authority.rs:11` unchanged).
- [ ] **D2/D3** six `#[but_api(napi)]` verbs + five `but review *` CLI verbs reuse `PullRequestsWrite` (open) /
  `ReviewsWrite` (assign) / `CommentsWrite` (+ branch-scoped reads); `assign_reviewer` enforces
  distinct-from-author; `resolve_thread` enforces resolver-identity (R22); local-cache writes omit the DryRun
  guard (matching `approve_review`); SDK regenerated; N-API audited (R14).
- [ ] **D4/D5** PR lifecycle DERIVED (no `local_pull_requests` table); agent tag derived from the **opener
  principal's declared `kind` in committed `permissions.toml`** (additive optional `kind` field on `PrincipalWire`,
  read at the target ref — NOT handle-resolution), cached in the dedicated `local_review_meta` row (NOT a comment
  body), never a caller arg; the `kind` field changes no enforcement (not in `GovConfig.principals`, no gate reads it).
- [ ] **D6** `keep_reviews_local` defaults LOCAL (`DefaultTrue`, the `ok_with_force_push` precedent — NOT the
  plain-`bool` `force_push_protection`); per-project operator preference (NOT `administration:write`-gated, NOT
  ref-pinned); older project files deserialize to local; remote mirroring gated behind it.
- [ ] **D8** `but-rules` auto "review-requested" hook reuses the Sprint-06b engine (no new rules mechanism);
  the auto-opened assignment is drive-only.
- [ ] **D9 / LPR-009** safe-seam grep green (gate path references none of the three new tables); forged-vs-empty
  (T-LPR-041) ⇒ identical merge decision; inverse (T-LPR-042) drive-only-cannot-land green.
- [ ] **R22 / LPR-003/004/010** drive-layer integrity green: self-assignment rejected (T-LPR-043);
  unauthorized self-resolve cannot suppress another party's remediation signal (T-LPR-044).
- [ ] **D10** remote-mirror seam specified, **not built** in Sprint 07; principal→forge disclosure named under
  R21.
- [ ] **R18–R23 / I6** named in the risk register (loop-sourced receipt / tag spoof / comment injection /
  keep-reviews-local preference flip + principal→forge disclosure / same-principal drive forgery / meta-row
  tag-control forgery) — never "mitigated-closed".
- [ ] **I1–I3, I5** applied only after the freeze lifts; counts read **7 / 30 / 201 / 205 / 23 / 10**.
- [ ] No frozen sprint task file or section file edited before the freeze lifted (N1's STEER renumber is a
  Stage-2 numbering-only edit, not a scope edit).
