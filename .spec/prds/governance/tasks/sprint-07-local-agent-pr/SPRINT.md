---
sprint: 07
sequence: 9
timeline: Phase 5 — Local Agent PR / Governed-Review Parity
status: Planned
proposed_by: rust-planner (LPR backend)
milestone: sprint-07
prd: ../../README.md
roadmap: ../../ROADMAP.md
generated_by: kb-sprint-tasks-plan
---

# Sprint 07: Local Agent PR — Governed-Review Parity (LPR)

**Sequence:** 9
**Timeline:** Phase 5 — Local Agent PR / Governed-Review Parity
**Status:** Planned
**Proposed by:** rust-planner (LPR backend); grounded against the shipped `but` tree at file:line
**Milestone:** — (`sprint-07`)

## Overview

The **LPR** sprint gives GitButler's local review layer GitHub-PR parity, so an orchestrator drives the whole
implement→review→merge loop off `but`'s own review state — a **reconciler over `but`, not a private state
machine** — with agent PRs kept **local by default**. Where AUTHZ/GRPS/GATES decide _whether_ an action
proceeds and LOOP demonstrates the end-to-end flow, LPR governs the **review-drive layer** around the
land-truth: assignments, comment threads, the derived PR object, the local-by-default setting, and the agent
tag. It is **additive** — every LPR behavior layers onto **three new `but-db` tables**
(`local_review_assignments`, `local_review_comments`, `local_review_meta`) and the existing `but review` write
path, and **none of it feeds the merge gate**.

> **The safe seam is the load-bearing invariant (UC-LPR-07, §E of the tech-delta).** The merge gate reads
> **only** `local_review_verdicts` at the current head (`crates/but-api/src/legacy/merge_gate.rs:40` →
> `review_verdicts(ctx, &review.source_branch)` → `local_review_verdicts().list_by_target(target)`; the
> verdict-at-head filter is `crates/but-api/src/legacy/review_requirement.rs:94` `verdict.head_oid ==
current_head_oid` + `:8` `const APPROVED: &str = "approved"`). `local_review_assignments`,
> `local_review_comments`, and `local_review_meta` are orchestration drive-metadata that **never gate**. A
> forged, malicious, or empty drive table cannot change a land decision, because the gate path never reads it.
> **"Gate gates (verdict-at-head, untouched); new tables drive (orchestration)."** This is what makes the whole
> sprint legal under the freeze: it cannot regress the land-truth because it never participates in the land
> decision (proven by build-gate grep over all THREE drive tables + the forged-vs-empty + inverse integration
> tests, LPR-009).

This sprint is **CLI/backend-first** (the MGMT desktop render of the local PR is deferred). Every behavior is a
real `but-db` + real `but-api` + real `gix` fixture driven through the **real `but review` CLI verbs**
(`request` / `assign` / `comment` / `resolve` / `status`, plus the shipped `approve` and the
`request-changes` whose `changes_requested` write LPR **implements**) and asserted on the per-project
`but.sqlite` row state, the structured JSON denial on stderr + exit code, and the **merge-gate decision** — the
same hand-assertion style as the shipped `commit_gate` / `merge_gate` / `governed_loop` tests
(`crates/but/tests/`, `crates/but-api/tests/`). **No mocks.**

> **`request_changes_review` is currently a CONTRACT STUB.** `crates/but-api/src/legacy/forge.rs:551`
> authorizes `Authority::ReviewsWrite` then returns `task_contract_invalid("request_changes_review", …)` and
> **writes nothing**. LPR-003 must **IMPLEMENT** the real `changes_requested` write (set
> `local_review_assignments.state='changes_requested'` on `ReviewsWrite`); this is NEW LPR work, **not** a
> reuse of an existing write. **`comment_review` (`forge.rs:569`) is ALSO a stub** (authorizes `CommentsWrite`
> then `task_contract_invalid`, writing nothing); LPR-004's `post_comment` **REPLACES** it (re-points the
> `but review comment` CLI verb at the real write), it does not wrap it.

> **No new `Authority`. Reuse only.** `request_review` gates on `PullRequestsWrite` (the open-PR authority);
> **`assign_reviewer` gates on `ReviewsWrite`** (assignment is a _review interaction_, not opening the PR —
> tech-delta §B) and enforces **`reviewer != author_principal_of_target_branch`** at the `but-api` boundary
> (R22); comment writes gate on `CommentsWrite`; `changes_requested`/`approved` continue to gate on
> `ReviewsWrite`. The catalog (`crates/but-authz/src/authority.rs:11`) is **unchanged**; the `write` role
> already grants all three (`WRITE_AUTHORITIES`, `authority.rs:343`). `keep_reviews_local` defaults **LOCAL**
> via `DefaultTrue` — the precedent is `ok_with_force_push` (`crates/gitbutler-project/src/project.rs:106`),
> **NOT** `force_push_protection` (a plain `bool` at `:108`). It is a per-project operator preference under the
> **R12 trusted-desktop** model — **NOT** `administration:write`-gated, **NOT** ref-pinned.

## Human Testing Gate

**Gate:** An agent principal opens a local review on a feature branch (no remote PR is created while
`keep_reviews_local` is true); a reviewer principal **distinct from the branch author** is assigned (`but
review request`/`assign`) and posts a file/line comment (`but review comment --file --line --thread`); `but
review status` shows the assignment, the open comment thread, the derived lifecycle, and an `agent-authored`
tag (sourced from the **opener principal's declared `kind = "agent"` in committed `.gitbutler/permissions.toml`**,
read at the target ref, cached in the dedicated `local_review_meta` opener row); the reviewer approves (`but
review approve`) and the orchestrator merges through the **unchanged** merge gate; a self-assignment is
rejected and an unauthorized self-resolve cannot clear another party's thread; and a fully forged / empty set
of `local_review_assignments` + `local_review_comments` rows yields an **identical** merge-gate decision (the
safe seam, proven by build-gate grep + test).

### Test Steps

1. As an agent principal (`BUT_AGENT_HANDLE` set), run `but review request <branch> --reviewer <p>` with
   `keep_reviews_local=true`; observe a `pending` `local_review_assignments` row in `but.sqlite` and **no**
   remote forge PR created.
2. Run `but review status <branch>`; observe the assignment, the derived lifecycle (e.g. `AwaitingReview`),
   and the `agent-authored` tag on the derived PR object — sourced from the **opener principal's declared
   `kind = "agent"` in committed `.gitbutler/permissions.toml`** (read at the target ref; cached in the
   dedicated `local_review_meta` opener row), **not** from `BUT_AGENT_HANDLE` resolution.
3. Assign a reviewer **equal to** the branch author (`but review assign <branch> --reviewer <author>`); confirm
   it is **rejected** (distinct-from-author enforced at the `but-api` boundary, R22) with no assignment row
   written.
4. As the reviewer, run `but review comment <branch> --body "fix this" --file f.rs --line 12 --thread t1`;
   observe a `local_review_comments` row (`resolved=false`).
5. Run `but review resolve <branch> t1`; observe every `t1` comment row flips `resolved=true`, with no
   verdict/assignment change. Then attempt a self-resolve of a `changes_requested`-style thread as a
   non-author/non-assigned/non-`reviews:write` principal; confirm it is **rejected** and the thread stays
   unresolved (no forged all-clear, R22).
6. As the reviewer, run `but review approve <branch>`; observe an `approved` `local_review_verdicts` row at
   head, then run the governed merge → it **proceeds**.
7. Write a fully forged `local_review_assignments` (all `approved`) + `local_review_comments` (all `resolved`)
   - `local_review_meta` (a forged `opener_principal` row) set directly with **no** approved verdict at head;
     attempt the governed merge → it is **blocked** (`gate.review_required`), identical to an empty drive set.

## Tasks

| ID             | Title                                                                                                                                                                                                                                                                                                                                                                                                                                        | Agent                   | Estimate |
| -------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------- | -------- |
| LPR-001        | `local_review_assignments` + `local_review_comments` + `local_review_meta` tables + 3 `SchemaVersion::Zero` migrations + 3 structs + Handle/HandleMut pairs (`list_by_target`, upsert/insert, `set_state`/`set_resolved`, `list_by_thread`, meta `upsert_if_absent`/`get`)                                                                                                                                                                   | rust-implementer        | 180 min  |
| LPR-002        | `AssignmentState { Pending, Approved, ChangesRequested }` typed enum + boundary (de)serialization (the `Authority` parse/`name` round-trip); column stays `TEXT`; **no** new `Authority` variant                                                                                                                                                                                                                                             | rust-implementer        | 75 min   |
| LPR-003        | `request_review` (`PullRequestsWrite`, +`local_review_meta` opener row) / `assign_reviewer` (`ReviewsWrite`, **distinct-from-author** at the boundary) `#[but_api(napi)]` + `but review request`/`assign` CLI; **implement the real `changes_requested` write** (`request_changes_review` is a stub today); structured `perm.denied` + exit 1 on missing authority; self-assignment rejected                                                 | rust-implementer        | 180 min  |
| LPR-004        | `post_comment`/`list_comments`/`resolve_thread` `#[but_api(napi)]` (`CommentsWrite` writes; **resolver-identity** constraint — author / assigned reviewer / `reviews:write` — on resolve; branch-scoped reads) + `but review comment --file/--line/--thread`/`comments`/`resolve` CLI; **`post_comment` REPLACES the stubbed `comment_review` (`forge.rs:569`)** (re-point the `comment` CLI verb at it); local-cache write, no DryRun guard | rust-implementer        | 150 min  |
| LPR-005        | `review_status` **derived** PR lifecycle (commits + verdict-at-head + open assignments; read-only `gix` walk, NO mutation) + the agent-PR tag derived from the **opener principal's declared `kind` in committed `permissions.toml`** (the additive optional `kind` field on `PrincipalWire`, read at the target ref — NOT handle-resolution, NOT a comment body), cached in the dedicated `local_review_meta` row                           | rust-implementer        | 180 min  |
| LPR-006        | `Project.keep_reviews_local: DefaultTrue` (per-project operator preference — NOT `administration:write`-gated, NOT ref-pinned; the `ok_with_force_push` `DefaultTrue` precedent) + default-local wiring + remote-mirror **gate** (the mirror path is NOT built — named seam only; principal→forge disclosure named under R21)                                                                                                                | rust-implementer        | 120 min  |
| LPR-007        | `but-rules` auto "review-requested" hook (commit Trigger → Filter → Action opens a `pending` assignment), reusing the Sprint-06b Trigger→Filter→Action engine — no new rules mechanism                                                                                                                                                                                                                                                       | rust-implementer        | 150 min  |
| LPR-008        | **Reconciler read-API**: `review_status` serves the full drive state (assignments + unresolved comments + verdict-at-head) in one payload, so two orchestrators converge; two-read agreement proof                                                                                                                                                                                                                                           | rust-implementer        | 120 min  |
| LPR-009        | **Safe-seam invariant**: net-new build-gate honesty grep (gate path has NO ref to the 3 new tables incl. `local_review_meta`) + the forged-vs-empty + inverse integration tests (drive metadata alone never lands; only verdict-at-head flips)                                                                                                                                                                                               | rust-reviewer           | 180 min  |
| LPR-010        | TS SDK regen (`pnpm build:sdk && pnpm format`) + N-API audit (R14 — the verbs ARE `but-api` fns) + happy-path CLI tests; honesty/anti-fakeability greps (tag-not-an-enforcement-key; tag sourced from `local_review_meta`, not a comment body) + the drive-layer-integrity proofs (self-assignment rejected T-LPR-043; unauthorized self-resolve cannot suppress a signal T-LPR-044)                                                         | rust-reviewer           | 150 min  |
| LPR-011        | Reconciler usage-model doc + the `but-*` skill contract (`keep_reviews_local=true` on governed-project init) — the skill workflow auto-sets local; skills _implementation_ is OUT of this sprint (documented contract only)                                                                                                                                                                                                                  | rust-implementer / docs | 75 min   |
| DESIGN-LPR-001 | `keep_reviews_local` toggle — Project-Settings design contract (placement by forge settings, default-local copy, R21-not-a-security-boundary caveat)                                                                                                                                                                                                                                                                                         | frontend-designer       | 30 min   |
| DESIGN-LPR-002 | Principal `kind` (agent/human) — Principals-tab display+edit design contract (neutral badges, non-enforcement disclosure)                                                                                                                                                                                                                                                                                                                    | frontend-designer       | 30 min   |
| DESIGN-LPR-003 | Local-review view — IA + four-lifecycle-state contract (read-only, mandatory merge-gate note)                                                                                                                                                                                                                                                                                                                                                | frontend-designer       | 50 min   |
| LPR-012        | `keep_reviews_local` toggle in Project Settings (project-settings path; DefaultTrue; NOT admin-gated)                                                                                                                                                                                                                                                                                                                                        | sveltekit-implementer   | 60 min   |
| LPR-013        | `principal_kind` Tauri command + SDK producer (governed-config, `administration:write`-gated, pending→commit)                                                                                                                                                                                                                                                                                                                                | tauri-implementer       | 180 min  |
| LPR-014        | Principal `kind` field in the Principals editor (read/write via governance IPC)                                                                                                                                                                                                                                                                                                                                                              | sveltekit-implementer   | 75 min   |
| LPR-015        | Local-review READ producer — `review_status`/`list_comments` Tauri commands + SDK (branch-scoped, R14 no-bypass)                                                                                                                                                                                                                                                                                                                             | tauri-implementer       | 120 min  |
| LPR-016        | `LocalReviewView` read-only panel (assignments/threads/lifecycle/agent-tag; no merge affordance)                                                                                                                                                                                                                                                                                                                                             | sveltekit-implementer   | 90 min   |

> **UI/full-stack extension** (appended 2026-06-21): `LPR-012`–`016` + `DESIGN-LPR-001`–`003` add the three UI surfaces (local-only toggle · principal agent/human tag · read-only local-review view) on top of the backend `LPR-001`–`011`. The backend slice stays runnable independently; the UI tasks depend on their producers (`LPR-013`→`014`, `LPR-015`→`016`) + the matching `DESIGN-LPR-*` contracts. None touch the merge gate (read-only + descriptive tags).

## Dependencies

- **Blocks:** None (Sprint 08 STEER does **not** depend on LPR — the renumber is clean).
- **Dependent on:** Sprint 01b (the `approve_review` verdict write + the merge gate `enforce_merge_gate`),
  Sprint 04 (merge strictness / the `[[gate]]` review requirement), Sprint 05 (`but perm`/`but group` CLI
  surface convention + persisted config), and the shipped `but-rules` engine (Sprint 06b) for the auto-hook.

### Intra-sprint dependency chain

```
LPR-001 (3 tables + Handles) ───────────────→ LPR-002 (AssignmentState typed over the TEXT column)
LPR-001 + LPR-002 ──────────────────────────→ LPR-003 (request/assign + changes_requested write)
LPR-001 ────────────────────────────────────→ LPR-004 (comment/list/resolve)
LPR-001 + LPR-003 + LPR-004 ─────────────────→ LPR-005 (derived PR lifecycle + agent tag from declared kind, cached in local_review_meta)
LPR-005 ────────────────────────────────────→ LPR-008 (reconciler read-API = full drive state in one payload)
LPR-003 (request_review default-local) ──────→ LPR-006 (keep_reviews_local + mirror gate)
LPR-001 + LPR-003 ──────────────────────────→ LPR-007 (but-rules auto review-requested hook)
LPR-001..008 ───────────────────────────────→ LPR-009 (safe-seam grep over all 3 tables + forged-vs-empty + inverse)
LPR-003 + LPR-004 + LPR-005 + LPR-008 ───────→ LPR-010 (SDK regen + N-API audit + honesty greps + drive-integrity proofs)
LPR-006 + LPR-008 ──────────────────────────→ LPR-011 (reconciler usage-model doc + the but-* skill contract — doc only)
```

## PRD Coverage

- **Use cases:** UC-LPR-01 (local PR object + reviewer assignment), UC-LPR-02 (local review comment thread),
  UC-LPR-03 (keep-PRs-local setting), UC-LPR-04 (agent-PR tagging), UC-LPR-05 (orchestrator reconciles off
  `but` review state + the skill contract), UC-LPR-06 (auto review-requested hook), UC-LPR-07 (safe-seam
  invariant).
- **Criteria:** T-LPR-001..044 (+ the hand-driven full-local-loop human-gate T-LPR-029h). All ACs covered;
  the load-bearing gate is the safe-seam proof T-LPR-040 → T-LPR-042 (LPR-009). The `[build-gate]` honesty
  invariants — no-gate-read-of-the-THREE-new-tables incl. `local_review_meta` (T-LPR-038),
  tag-not-an-enforcement-key (T-LPR-022), and R6/R18-preserved (T-LPR-039) — block the slice regardless of the
  other lanes. The R22 drive-layer integrity proofs (T-LPR-043 self-assignment rejected; T-LPR-044 self-posted+
  self-resolved thread does not suppress another party's remediation signal, LPR-003/004/010) and the R23
  forged-meta-opener negative control accompany them.

## Capability Coverage

- **CAP-AUTHZ-01** — the six new `#[but_api(napi)]` verbs (LPR-003/004/005/008) authorize via
  `authorize_branch_action(&repo, &branch, Authority::X)?` (`forge.rs:47`) at the `but-api` boundary, exactly
  as the shipped `approve_review` (`forge.rs:520`). **No new `Authority` variant** — `request_review` reuses
  `PullRequestsWrite`; **`assign_reviewer` reuses `ReviewsWrite`** (a review interaction, not opening the PR —
  tech-delta §B); comment writes reuse `CommentsWrite`; `changes_requested` reuses `ReviewsWrite`. The
  route→Authority table stays closed (`authority.rs:11` unchanged). `assign_reviewer` additionally enforces
  `reviewer != author_principal_of_target_branch` at the `but-api` boundary (R22); `resolve_thread` enforces
  resolver-identity (author / assigned reviewer / `reviews:write` holder, R22).
- **CAP-CONFIG-01** — `keep_reviews_local` (LPR-006) is a **per-project operator preference under the R12
  trusted-desktop model** persisted in the project store (`gitbutler-project`) — the same class as
  `forge_override`/`preferred_forge_user`. It is **NOT** ref-pinned committed config (that is for
  `permissions.toml`/`gates.toml`, which gate _decisions_) and it is **NOT** `administration:write`-gated: the
  desktop human is the trusted fleet owner (R12), so the preference is owned by implicit desktop ownership, not
  a checked grant. It defaults LOCAL via `DefaultTrue`; older project files deserialize to local. An untrusted
  project-store write that flips it (→ agent PRs mirror to a public forge) is the named accepted residual
  **R21 (§G of the tech-delta)**.

## Coverage Notes

- **The three tables are the `local_review_verdicts` class, NOT the `forge_reviews` class.** All three follow
  the shipped `local_review_verdicts` pattern precisely (`crates/but-db/src/table/local_review_verdicts.rs`):
  per-table module exporting `pub(crate) const M: &[M<'static>]`, registered in
  `crates/but-db/src/table/mod.rs:10` and appended to `MIGRATIONS` in `crates/but-db/src/lib.rs:130` (the slice
  that already lists `table::local_review_verdicts::M` last, `lib.rs:142`). All three are `SchemaVersion::Zero`
  (additive tables older binaries tolerate, `lib.rs:167`) with fresh monotonic-by-creation-time ids
  (`20260621120000`, `20260621120100`, `20260621120200`, sorted by `migration::run`'s
  `sort_by_key(|m| m.up_created_at)`, `migration.rs:39`). `principal_id`/`target` are deliberately **un-FK'd**
  (principals live in committed config, not a table — like `local_review_verdicts` storing `principal_id TEXT`
  un-FK'd). They are NOT the runtime-cleared remote-cache class (`forge_reviews`,
  `DELETE FROM forge_reviews` at `forge_reviews.rs:153`). The third table, `local_review_meta`
  (`target`, `key`, `value`, `created_at`, `PRIMARY KEY(target, key)`, tech-delta §A.4), caches the computed
  agent-PR tag in one `key="opener_principal"` row per target.
- **PR lifecycle is DERIVED — there is no `local_pull_requests` table.** `review_status` (LPR-005) computes
  the PR view at query time over three already-present sources: the branch's commits ahead of base (a
  **read-only** `gix` graph walk, NO mutation), `local_review_verdicts.list_by_target(target)` filtered to
  `head_oid == current head` (the **exact** query `merge_gate` runs, `merge_gate.rs:40`/`:159`), and open
  `pending` `local_review_assignments` + unresolved `local_review_comments`. `Approved`/`Mergeable` is a
  **presentation label only**; the actual merge decision stays `enforce_merge_gate`, which re-derives
  verdict-at-head itself and never reads the derived view. This is the lossy-presentation discipline
  `WORKSPACE_MODEL.md` mandates for `Workspace`/`RefInfo`.
- **The agent-PR tag's source-of-truth is the opener principal's DECLARED `kind` in committed config — never a
  caller arg, never handle-inference.** The agent-vs-human distinction does **not** exist in the resolved
  `Principal` (`resolve_principal` keys solely on `BUT_AGENT_HANDLE` and `Principal` is `{ id, authorities,
groups }` with no `is_agent`/`PrincipalKind` discriminator — `crates/but-authz/src/authorize.rs:67`–`:90`,
  `principal.rs:82`). Deriving the tag from handle-resolution is therefore a fabrication (it cannot tell agent
  from human). The tag is instead derived from a **new additive, optional `kind` field on the principal entry in
  committed `.gitbutler/permissions.toml`** (`kind = "agent"`/`"human"`; omitted → human), read at the **target
  ref** like all governance config (`crates/but-authz/src/config.rs:23`–`:25`, the anti-self-escalation
  property). The `kind` field rides the existing `PrincipalWire` (`config.rs:424`,
  `#[serde(deny_unknown_fields)]`) as `#[serde(default)] pub kind: Option<String>` — exactly the optional-field
  pattern `role: Option<String>` already uses (`config.rs:427`) — and changes **no enforcement** (it does NOT
  enter `GovConfig.principals`, `config.rs:85`; no gate reads it). `request_review` records the opener principal
  once in `local_review_meta(target, "opener_principal", <id>)` via `INSERT … ON CONFLICT(target,key) DO
NOTHING`; the derivation resolves that opener's committed entry at the target ref and sets `agent_authored =
true` iff it declares `kind = "agent"`, caching the computed tag in that `local_review_meta` row (NOT a
  comment-body sentinel — R23 ≠ R20). Spoofability via `BUT_AGENT_HANDLE` re-export to impersonate a _different
  declared principal_ (borrowing its kind) is named as **R19**, NOT closed; direct DB-row forgery of the cached
  opener row is named as **R23**, NOT closed.
- **`request_changes_review` AND `comment_review` are STUBS that LPR implements/replaces — they are not
  reused.** `request_changes_review` (`forge.rs:551`) authorizes `ReviewsWrite` then returns
  `task_contract_invalid` and writes nothing; LPR-003 implements the real `changes_requested` write (set
  `local_review_assignments.state='changes_requested'`). `comment_review` (`forge.rs:569`) authorizes
  `CommentsWrite` then returns `task_contract_invalid` and writes nothing; LPR-004's `post_comment` **REPLACES**
  it (re-points the `but review comment` CLI verb at the real write), it does not wrap it. State both
  explicitly in the completion report.
- **Local writes need NO `DryRun` guard (the `approve_review` precedent).** `approve_review` writes
  `local_review_verdicts` with **no** `DryRun` check (`forge.rs:520`–`:546`) — correct because the write
  touches **only** the local project cache (`ctx.db.get_cache_mut()` → SQLite), **not** refs, objects, or
  oplog. RULES.md's "dry runs must not persist refs, objects, or oplog" does not bind a local-cache row (none
  of those). The new verbs write the same cache the same way and omit the DryRun guard for the same reason.
  (Contrast the **merge** verb, which IS gated and IS forge-bound — `merge_review` — and is untouched here.)
- **The `but-rules` auto-hook ADDS two variants the shipped engine does not have.** `but-rules`'s `Trigger`
  enum today is only `{ FileSytemChange, ClaudeCodeHook }` (`crates/but-rules/src/lib.rs:77`) — **no commit
  trigger** — and its `Action` enum is only `{ Explicit(Operation), Implicit(ImplicitOperation) }`
  (`lib.rs:140`), all workspace-staging operations — **no review-assignment action**. LPR-007 **extends** the
  shipped Trigger → Filter → Action engine with the new commit `Trigger` variant + the review-assignment
  `Action` variant (the two variants `but-rules` does not yet have); it does **not** re-build a rules mechanism.
- **The remote-mirror seam is SPECIFIED, NOT BUILT.** LPR-006 gates the mirror path behind
  `keep_reviews_local == false`, but **no mirroring code lands in this sprint**. The bridge exists
  (`but_forge::create_forge_review`, `crates/but-forge/src/review.rs:1251`; `sync_reviews`, `:1349`) and the
  field mapping is specified in the tech-delta §D; a future `mirror_local_review` verb (gated by
  `PullRequestsWrite`, active only when the flag is false) can map the rows without schema change. The
  `reviewer_principal → forge reviewer` mapping discloses internal principals to a public forge — named under
  **R21**, to be designed fail-closed when built.
- **Named risks, never mitigated-closed.** R18 (local-review forgeability — no independent engine re-read of
  the verdict store; same R6 accepted-leak class), R19 (agent-tag spoofability via `BUT_AGENT_HANDLE` re-export
  to impersonate a _different declared principal_ and borrow its declared `kind`; same R2 residual), R20
  (comment-body injection into agent context; L2/harness residual), **R21** (`keep_reviews_local` is a
  trusted-desktop preference, not an authorization boundary — an untrusted project-store write flips it; same
  R12 accepted-leak class), **R22** (same-principal drive-layer forgery — self-assignment / self-resolve; the
  `but-api`-boundary distinct-from-author + resolver-identity checks narrow _cross-principal_ forgery but cannot
  make a one-principal repo multi-party), **R23** (DB-row forgery of the agent-tag derivation control path — the
  cached `local_review_meta` opener row is forgeable by a direct DB write; same R6/R18 accepted-leak class) are
  **accepted, named** residuals the build MUST NOT present as closed. R22's distinct-from-author/resolver-identity
  constraints ARE real, tested integrity checks (LPR-003/004), but they narrow cross-principal forgery only. Do
  not write a test asserting a forgeable direct DB write to `local_review_verdicts` (or the `local_review_meta`
  opener row) is blocked (that encodes a false guarantee — R6/R18/R23 accepted-leak).
- **Implementation is out of scope for this artifact:** these are TDD **task contracts**. The Rust tables,
  the `#[but_api(napi)]` verbs, the typed enum, the project field, the auto-hook, the derived PR view, and the
  safe-seam gate are written at execution time by `/kb-run-sprint`, RED→GREEN against these specs and the
  regenerated SDK.

> **Frozen-aware.** This is **Sprint 07 (LPR)** — the human-directed slot; the existing STEER sprint renumbers
> 07→08. References no frozen sprint's tests (Sprints 01a–06b are unchanged). The merge-gate criteria exercise
> the **governed review-submission path** only; a direct DB write to `local_review_verdicts` stays untestably
> forgeable (R6/R18) and is explicitly NOT a path under test.

## Task Detail Files

Generated by /kb-sprint-tasks-plan from ROADMAP.md Sprint 07. The per-task detail files (below) carry the
stable `AC-N`/`TC-N` Requirement Contract that `/kb-run-sprint` consumes.

- LPR-001-but-db-tables-migrations.md
- LPR-002-assignment-state-enum.md
- LPR-003-request-assign-changes-requested.md
- LPR-004-comment-thread-verbs.md
- LPR-005-derived-pr-lifecycle-agent-tag.md
- LPR-006-keep-reviews-local-setting.md
- LPR-007-but-rules-review-requested-hook.md
- LPR-008-reconciler-read-api.md
- LPR-009-safe-seam-invariant.md
- LPR-010-sdk-regen-napi-audit-honesty-greps.md
- LPR-011-reconciler-usage-skill-contract.md
