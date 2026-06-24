---
title: Local Agent PR / Governed-Review Parity (LPR) — Governance PRD Enrichment v1.5.0
version: 1.5.0
enriches: .spec/prds/governance/README.md
from_version: 1.4.0
status: planned (net-additive; sprints 01a–06b frozen; sprint slot assigned by human directive)
scope_posture: net-additive progression
pr_sequencing: false
---

# Local Agent PR / Governed-Review Parity (LPR) — Enrichment v1.5.0

A **net-additive** enrichment of the Functional-Permission Agent Governance PRD. It adds one functional group
— **LPR** — that gives GitButler's **local** review layer GitHub-PR parity: reviewer assignment, a review
comment thread, a derived PR lifecycle, a per-project local-by-default setting, and an automatic agent-PR tag
— so a goal-directed orchestrator drives the whole implement→review→merge loop off `but`'s own review state
instead of being pushed out onto the forge.

> **The reconciler thesis — drive off `but` state, do not invent a second one.** The orchestrator should be a
> **reconciler over `but` review state, not a private state machine that shadows it.** It dispatches a reviewer
> because `but` shows an _open assignment_; it triggers remediation because `but` shows an _unresolved
> comment_; it merges because `but` shows an _approved verdict at head_. Every decision is a projection of
> `but`'s own committed/local state — so two orchestrators reading the same repo converge, and the human and
> the agents share one source of truth. Agent PRs are kept **local by default**; the forge becomes optional,
> not load-bearing.

This is the **recommended option (2) "Full local PR"** (assignments **and** comment threads), chosen because
the reconciler model needs all three drive signals (assignment + comment + approval) to run the loop locally;
half the drive layer would push the comment half of the loop back onto the forge and defeat the thesis.

## Status — frozen-aware

- **Net-additive.** Changes no existing scope, no gate decision, no code set, and **no line of
  `merge_gate.rs` / `review_requirement.rs`**. Adds three `but-db` tables + a CLI/API surface + a project
  setting only.
- **Sprints 01a–06b are FROZEN** (in-flight agents). This enrichment **edits none of them**. Implied changes
  to shipped code and to the frozen PRD index files are recorded as deltas in
  [05-delta-replan.md](./05-delta-replan.md), to apply when the freeze lifts.
- **🔴 Sprint slot (human directive — instruction-precedence #1): LPR = Sprint 07, STEER → Sprint 08.** The
  v1.4.0 STEER enrichment had claimed Sprint 07 on the live ROADMAP; the human reassigned the priority. LPR
  takes **Sprint 07** and the existing STEER sprint is renumbered to **Sprint 08** (numbering-only; STEER's
  scope is untouched). The 00/01/03/04 files were first drafted "Sprint 08 (LPR)"; that wording is reconciled
  to "Sprint 07 (LPR)" — the honest record of the reassignment is kept in
  [05-delta-replan.md](./05-delta-replan.md) §N1 + §R-edits. The actual ROADMAP/folder renumber is executed
  downstream in the `/kb-sprint-plan` stage.

> **The one-sentence invariant — gate gates; drive drives.** The merge gate reads **only**
> `local_review_verdicts` at head; the three new tables (`local_review_assignments`, `local_review_comments`,
> `local_review_meta`) are **orchestration drive-metadata that never gate**. A forged or empty drive table
> yields an identical merge decision. This is what makes the enrichment safe-by-construction under the freeze
> (proven by file:line in [03 §E](./03-technical-requirements-delta.md) + a build-gate grep).

## Document Index

| File                                                                       | Section                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                | Stability       |
| -------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------- |
| [00-overview.md](./00-overview.md)                                         | The reconciler thesis; the four drive gaps; the safe-seam invariant                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    | PRODUCT_CONTEXT |
| [01-scope-delta.md](./01-scope-delta.md)                                   | In scope / out of scope (net-add); what is preserved from sprints 1–8 + STEER                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          | FEATURE_SPEC    |
| [02-uc-lpr.md](./02-uc-lpr.md)                                             | LPR functional group + UC-LPR-01..07 (40 ACs)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          | FEATURE_SPEC    |
| [03-technical-requirements-delta.md](./03-technical-requirements-delta.md) | 3 additive `but-db` tables + migrations, the `but review` verb surface (with drive-layer integrity constraints; `post_comment`/`changes_requested` replace the shipped `comment_review`/`request_changes_review` stubs), the project setting, the agent tag (derived from the opener's declared `kind` in committed `permissions.toml` — an additive optional `kind` field on `PrincipalWire`, read at the target ref — cached in a dedicated `local_review_meta` row), the deferred remote-mirror seam, **§E the safe seam (file:line proof)**, blast-radius, R18–R23 | CONSTITUTION    |
| [04-e2e-testing-criteria.md](./04-e2e-testing-criteria.md)                 | T-LPR-001..044 (+ T-LPR-029h) — real `but-db`/`but-api`/`gix`; the safe-seam proof (forged ≡ empty; drive-only cannot land) + drive-layer-integrity proofs (self-assignment rejected; unauthorized self-resolve cannot suppress a signal); the agent-tag proofs assert on the opener's declared `kind` in committed `permissions.toml`, not handle-resolution                                                                                                                                                                                                          | TEST_SPEC       |
| [05-delta-replan.md](./05-delta-replan.md)                                 | Code deltas (D1–D10) + risks (R18–R23) + proposed Sprint 07 (LPR) + the STEER 07→08 renumber (N1) + count reconciliation + frozen-file integration edits (I1–I6)                                                                                                                                                                                                                                                                                                                                                                                                       | —               |

## The contract at a glance

A **local PR object** — derived, not a fourth stored truth: an assignment + a comment thread + a verdict-at-head

- an auto-applied agent tag, with the lifecycle **computed** from commits + `local_review_verdicts`@head + open
  assignments. Strings below are illustrative; the new tables drive orchestration and **never feed the merge
  gate**.

```jsonc
{
	"local_pr": {
		"target": "feature/login",
		"source_branch": "feature/login",
		"sha": "a1b2c3d", // ForgeReview-shape fields
		"author": "rust-implementer",
		"title": "Add login flow",
		"draft": false,
		"labels": ["agent-authored"], // auto-set from the opener principal's DECLARED kind in committed permissions.toml (read at the target ref; cached in a dedicated local_review_meta row; never a caller arg, never handle-resolution, never a comment body)
		"lifecycle": "AwaitingReview", // DERIVED: commits + verdict@head + open assignments (no stored 4th truth)
		"assignments": [
			// local_review_assignments — drive-only, NEVER gates; reviewer must be distinct from author
			{ "reviewer": "rust-reviewer", "state": "pending", "assigned_at": "2026-06-21T12:00:00Z" },
		],
		"comment_threads": [
			// local_review_comments — drive-only, NEVER gates; resolve constrained to author/reviewer/reviews:write
			{
				"thread_id": "t1",
				"file": "src/login.rs",
				"line": 42,
				"resolved": false,
				"comments": [{ "author": "rust-reviewer", "body": "handle the None case" }],
			},
		],
		"verdict_at_head": null, // local_review_verdicts@head — the ONLY thing the merge gate reads
	},
}
```

> An open `pending` assignment, a `changes_requested` state, or an unresolved comment thread **does not block a
> merge**. Only a distinct `local_review_verdicts` approval at the current head can — exactly as the shipped
> merge gate decides today.

## Quick-Stats Delta (v1.4.0 → v1.5.0)

| Metric              | v1.4.0 | Δ                                                                                       | v1.5.0 (after integration) |
| ------------------- | ------ | --------------------------------------------------------------------------------------- | -------------------------- |
| Functional Groups   | 6      | +1 (LPR)                                                                                | **7**                      |
| Use Cases           | 23     | +7 (UC-LPR-01..07)                                                                      | **30**                     |
| Acceptance Criteria | 161    | +40 (LPR)                                                                               | **201**                    |
| Testing Criteria    | 160    | +45 (T-LPR; +33 integration · +6 api-contract · +3 build-gate · +1 human-gate · +2 e2e) | **205**                    |
| Risk register       | 17     | +6 (R18–R23, named)                                                                     | **23**                     |
| ROADMAP sprints     | 9 †    | +1 (LPR = Sprint 07; STEER → Sprint 08)                                                 | **10**                     |

> † The **9-sprint** baseline is the v1.4.0 _enrichment_ count (it counts STEER as sequence #9). The **live `ROADMAP.md` reads `sprint_count: 8`** — STEER's v1.4.0 I4 "APPLIED 8→9" never wrote a row into `ROADMAP.md`. Stage 2 lands **both** the STEER row (08) and the LPR row (07), taking the live `sprint_count` **8 → 10**; the final ROADMAP holds **10** either way (see [03-technical-requirements-delta.md](./03-technical-requirements-delta.md) §15 + [05-delta-replan.md §4](./05-delta-replan.md)).

Counts reconcile in [05-delta-replan.md §4](./05-delta-replan.md). Per-AC criterion coverage: **40/40** ACs
have ≥1 T-LPR criterion (45 criterion rows; T-LPR-029h is a human-gate alongside the automated T-LPR-029,
T-LPR-043/044 are the two drive-layer-integrity proofs — self-assignment rejected and unauthorized self-resolve
cannot suppress a signal — and T-LPR-041/042 are the two UC-LPR-07 safe-seam capstones).

## Integration points (apply when the freeze lifts / downstream at Stage 2)

Summarized from [05-delta-replan.md §5](./05-delta-replan.md) — all **append-style**, no rewrites of frozen
files. Tracked here as they're applied (mirrors the v1.4.0 I-edit ledger).

| ID     | Edit                                                                                                                                                                                                                                                                                                                                                       | Status                                 |
| ------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------- |
| **N1** | **STEER Sprint 07 → Sprint 08 renumber** (ROADMAP row + details + deps, `tasks/sprint-07-steer*` → `sprint-08-steer*`, v1.4.0 enrichment refs) — numbering-only, STEER scope unchanged                                                                                                                                                                     | ⬜ pending (Stage 2 `/kb-sprint-plan`) |
| **I4** | `ROADMAP.md`: **after N1**, add the **Sprint 07 (LPR)** row (slug `sprint-07-local-agent-pr`) + details + dependency edges (01b/04/05). Live `ROADMAP.md` is `sprint_count: 8` (STEER's v1.4.0 I4 never landed), so Stage 2 lands **both** rows → `sprint_count` **8 → 10** (≡ 9→10 on the v1.4.0 enrichment baseline); order `(…, 06b, 07-LPR, 08-STEER)` | ⬜ pending (Stage 2 `/kb-sprint-plan`) |
| I1     | copy `02-uc-lpr.md` → top-level `13-uc-lpr.md` (no renumbering)                                                                                                                                                                                                                                                                                            | ⬜ pending (freeze)                    |
| I2     | `03-functional-groups.md`: +LPR row + Use-Case-Summary row (LPR · 7 · 40); totals → 7 groups / 30 UCs / 201 ACs                                                                                                                                                                                                                                            | ⬜ pending (freeze)                    |
| I3     | `README.md`: Document Index + Quick Stats (6→7 / 23→30 / 161→201 / 160→205 / 17→23 / 9→10) + Version History + `version: 1.5.0`                                                                                                                                                                                                                            | ⬜ pending (freeze)                    |
| I5     | fold T-LPR-001..044 (+ T-LPR-029h) into `11-e2e-testing-criteria.md` (+ count line → 205)                                                                                                                                                                                                                                                                  | ⬜ pending (freeze)                    |
| I6     | fold R18/R19/R20/R21/R22/R23 into `10-technical-requirements/07-technical-risks.md` (risk count 17 → 23)                                                                                                                                                                                                                                                   | ⬜ pending (freeze)                    |

And the code deltas D1–D10 (three additive `but-db` tables + migrations, the six `#[but_api(napi)]` `but review`
verbs reusing `PullRequestsWrite` (open) / `ReviewsWrite` (assign + approve) / `CommentsWrite` (comments) with
drive-layer integrity constraints, the five `but review *` CLI verbs, the DERIVED PR lifecycle, the
auto-derived agent tag (from the opener's declared `kind` in committed `permissions.toml`, cached on a dedicated `local_review_meta` row; `post_comment`/`changes_requested` replace the shipped `comment_review`/`request_changes_review` stubs), the `Project.keep_reviews_local` setting, the
`but-rules` auto "review-requested" hook, the safe-seam build-gate grep, and the named-not-built remote-mirror
seam) land in **Sprint 07 (LPR)**, depending on Sprint 01b/04/05 and the shipped `but-rules` engine (Sprint 06b).

## Version History

| Version                     | Date       | Changes                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                | Trigger    |
| --------------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ---------- |
| 1.5.0 (enrichment, planned) | 2026-06-21 | New **LPR** group (7 UCs / 40 ACs / 45 criteria, +6 named risks R18–R23): Local Agent PR / governed-review parity — three additive `but-db` tables (`local_review_assignments`, `local_review_comments`, `local_review_meta`), the `but review request/assign/comment/list/resolve/status` surface (reuses `PullRequestsWrite` for open, `ReviewsWrite` for assign+approve, `CommentsWrite` for comments — **no new authority** — with drive-layer integrity constraints: `assign_reviewer` distinct-from-author and `resolve_thread` resolver-identity), a DERIVED PR lifecycle (commits + verdict-at-head + open assignments), an auto-applied `agent-authored` tag sourced from the opener principal's declared `kind` in committed `permissions.toml` (an additive optional `kind` field on `PrincipalWire`, read at the target ref — not handle-resolution, which cannot tell agent from human; the computed tag is cached in a dedicated `local_review_meta` opener row, not a comment-body sentinel), a per-project `keep_reviews_local` default-local operator preference (R12 trusted-desktop, not `administration:write`-gated), the `but-rules` auto "review-requested" hook, and the **safe-seam invariant** (the new tables drive; they never gate — the merge gate still reads only `local_review_verdicts`@head). Reconciler-over-`but`-state usage model; remote-mirror bridge named-not-built. Net-additive, frozen-aware; lands as **Sprint 07 (LPR)** per the human directive, with the existing **STEER sprint renumbered 07→08**. | Enrichment |

## Next Steps

- Execute **N1 + I4 downstream in `/kb-sprint-plan`**: renumber STEER 07→08, then insert **Sprint 07 (LPR)**
  (`sprint_count` 9→10). The R-edits reconcile this enrichment's 00/01/03/04 to "Sprint 07 (LPR)".
- Apply integration edits I1–I3, I5–I6 + bump the PRD to v1.5.0 **once the freeze lifts** (`/kb-prd-plan
--update` on the live PRD, or manual append). Final counts read **7 / 30 / 201 / 205 / 23 / 10**.
- Materialize **Sprint 07 (LPR)** via `/kb-sprint-tasks-plan` after Sprint 06b (LPR-001..011), depending on
  Sprint 01b/04/05 and the `but-rules` engine.
- Optional future **MGMT enrichment**: render the local PR object / comment threads (`but review status`
  payload) in the desktop Governance UI.
