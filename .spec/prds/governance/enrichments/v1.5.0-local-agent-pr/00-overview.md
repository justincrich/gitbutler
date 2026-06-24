---
title: Local Agent PR / Governed-Review Parity (LPR) — Governance PRD Enrichment v1.5.0
stability: PRODUCT_CONTEXT
last_validated: 2026-06-21
prd_version: 1.5.0
enrichment: local-agent-pr
enriches: .spec/prds/governance/README.md
from_version: 1.4.0
status: planned (net-additive)
posture: net-additive progression (sprints 01a–06b frozen; STEER renumbered Sprint 07 → Sprint 08)
---

# Enrichment v1.5.0 — Local Agent PR / Governed-Review Parity (LPR)

> **One line.** Give GitButler's **local** review layer GitHub-PR parity, so an orchestrator can drive **all** of its implement→review→merge work off `but`'s own review state — a **reconciler over `but`, not a private state machine** — with agent-authored PRs kept **local by default** and the merge gate's land-truth **untouched**.

This is a **net-additive enrichment** of the existing governance PRD. It adds one functional group (**LPR**) and three additive `but-db` tables that carry _review-drive_ metadata — assignments, comment threads, and a small per-target metadata row (the opener/tag) — alongside the existing local review store. It changes **no** existing scope: the merge gate's land-truth, the codes, the fail-closed posture, the four-caller seam, and the git-graph/workspace/rebase model are all preserved exactly. It is a **progression** that lands as this enrichment's **Sprint 07 (LPR)** (the human-directed slot; the v1.4.0 STEER sprint is renumbered 07→08), _after_ the existing roadmap (Sprints 01a–06b); the changes it implies are recorded as a **delta** to apply when the freeze lifts, never edited into a frozen sprint.

## The reconciler thesis: drive off `but` state, do not invent a second one

The PRD's governing philosophy (the irrigation thesis, `00-overview.md`) is that the governed path must be the _cheapest_ path — so a goal-directed agent flows down it. v1.5.0 extends that bet from the _moment of denial_ (where v1.4.0 STEER lives) to the _whole loop_: today an orchestrator that wants to run a real review cycle on GitButler has two surfaces — GitButler's **local** verdict store (the merge-gate truth) and GitHub's **remote** PR (the place humans review, comment, and assign). The local surface is land-authoritative but **thin**: it records a verdict-at-head and nothing else. There is no local notion of _who is assigned to review this_, no local _comment thread_, no local _PR lifecycle_ — so an orchestrator that wants those affordances is pushed **out of `but` and onto the forge**, where the loop is no longer local, no longer offline, and no longer the cheapest path.

The reconciler thesis closes that gap: **the orchestrator should be a reconciler over `but` review state, not a private state machine that shadows it.** It dispatches a reviewer because `but` shows an _open assignment_; it triggers remediation because `but` shows an _unresolved comment_; it merges because `but` shows an _approved verdict at head_. Every decision is a projection of `but`'s own committed/local state — so two orchestrators reading the same repo reach the same conclusion, and the human and the agents share one source of truth. The forge becomes optional, not load-bearing.

## The structural gap this closes

The merge gate today reads exactly one thing — `local_review_verdicts` (verdict `approved` at the current `head_oid`, distinct from author, meeting `min_approvals`) — and that is **correct and complete as a land-truth**. What is missing is the **drive** layer around it: the metadata an orchestrator needs to _decide what to do next_ before a verdict ever exists.

| #   | Drive question the orchestrator asks                   | What exists today                                                                   | The pull toward the forge                                  | v1.5.0 close                                                                                                                                                                              |
| --- | ------------------------------------------------------ | ----------------------------------------------------------------------------------- | ---------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | _Who should review this change?_                       | nothing local — `local_review_verdicts` only records verdicts that already happened | assign a reviewer on GitHub → loop leaves the local layer  | `local_review_assignments` — a local reviewer assignment with a `pending`/`approved`/`changes_requested` state (reviewer constrained distinct-from-author at the write boundary)          |
| 2   | _What did the reviewer object to, and is it resolved?_ | nothing local — there is no local comment/thread table                              | open a review comment on GitHub → feedback lives off-`but` | `local_review_comments` — a local comment thread (`file`/`line`/`thread_id`/`resolved`; resolve constrained to author/reviewer/`reviews:write`)                                           |
| 3   | _Is this an agent's PR or a human's?_                  | nothing distinguishes them                                                          | infer from GitHub author/labels                            | an automatic **agent-authored** tag (a label, `ForgeReview.labels` is precedent) on agent-principal reviews, sourced from a dedicated `local_review_meta` opener row (not a comment body) |
| 4   | _Should this PR be visible on the forge at all?_       | every "PR" is a forge PR or nothing                                                 | no local-only mode                                         | a per-project **"keep PRs local"** operator preference — agent PRs default to the local layer, remote mirroring is gated behind it                                                        |

The reference for "what local parity should look like" already lives in the schema: **`forge_reviews`** is the remote GitHub-PR cache (`number`, `title`, `body`, `author`, `labels`, `draft`, `source_branch`, `target_branch`, `sha`, `created_at`/`modified_at`/`merged_at`/`closed_at`, `reviewers`, …). v1.5.0 mirrors that shape _locally_ where it makes sense — the local PR object is **derived** (commits + verdict-at-head + open assignments), not a fourth stored copy of truth — so the local loop reaches feature-parity with the forge loop without a forge.

## The safe seam (state this as the invariant)

The single most important property of this enrichment: **the new objects drive; they never gate.**

- The merge gate reads **only** `local_review_verdicts` at head. That code path is **untouched** by v1.5.0. (`crates/but-api/src/legacy/merge_gate.rs` + `review_requirement.rs`.)
- `local_review_assignments`, `local_review_comments`, and `local_review_meta` are **additive orchestration metadata**. An open assignment does **not** block a merge; an unresolved comment does **not** block a merge; a `changes_requested` assignment-state does **not** block a merge. The **gate gates** (verdict-at-head); the **new tables drive** (orchestration). They are read by orchestrators and by `but review`, never by `review_requirement.rs`.
- This makes the enrichment _safe by construction_: even a fully forged `local_review_assignments`/`local_review_comments`/`local_review_meta` row cannot weaken the merge gate, because the gate never reads those tables. The land-truth's threat model (R6 — `local_review_verdicts` forgeable by direct DB write, the accepted-leak class) is **neither widened nor narrowed**. (The new drive tables carry their own named residuals — R18–R23, `03 §G` — but none touches the gate.)

> **Gate gates; drive drives.** If you remember one sentence about LPR, remember that the merge gate's truth is sealed off from every new object this enrichment introduces. The drive layer can be wrong, forged, or empty and the land decision is identical.

## What "Full local PR" buys (the recommended option)

The brainstorm weighed a minimal option (assignments only) against **option (2) "Full local PR"** — assignments **and** comment threads, a derived PR lifecycle, the project-local setting, and the agent tag — and chose Full, because the orchestrator-as-reconciler model needs _all_ of the drive signals (assignment **and** comment **and** approval) to run the whole loop locally. Half the drive layer would push the comment half of the loop back onto the forge and defeat the thesis.

Concretely, Full local PR ships:

1. **A local PR object** — three new additive `but-db` tables (`local_review_assignments`, `local_review_comments`, `local_review_meta`) on the existing `SchemaVersion`-style additive migration system, with PR **lifecycle state derived** from commits + verdict-at-head + open assignments (not stored as a fourth truth), mirroring `ForgeReview` fields where sensible.
2. **A `but review` command/API surface** — request/assign a reviewer, post/list/resolve comments, query review status — built on the **already-shipped** `Authority::PullRequestsWrite` (open) + `Authority::ReviewsWrite` (assign/approve) + `Authority::CommentsWrite` (comments) (no new authority), with drive-layer integrity constraints (assign distinct-from-author; resolve resolver-identity).
3. **A per-project "keep PRs local" setting** — agent-authored reviews default to the local layer (no remote GitHub PR); remote mirroring is gated behind the setting. It is a per-project operator preference under the trusted-desktop model, not an `administration:write`-gated governed-config change.
4. **An automatic agent-PR tag** — reviews created by an agent principal are auto-tagged **agent-authored** (a label, sourced from a dedicated `local_review_meta` opener row) to differentiate them from human PRs.
5. **A skill contract** — the `but-*` skills auto-set these PRs local when used (the capability the skills depend on).
6. **A reconciler usage model + an auto-hook** — orchestrators drive work off `but` review state, and the existing `but-rules` (Trigger → Filter → Action) engine is the home for an auto "review-requested" hook on commit.

## Why this is a progression, not a rewrite

The merge gate, the authorities, the DB, and the additive-migration mechanism already exist and are _correct_ — v1.5.0 is the **drive-layer** step that makes the existing local truth _orchestratable_. `Authority::PullRequestsWrite` / `Authority::ReviewsWrite` / `Authority::CommentsWrite` are shipped (reuse, no new authority). `approve_review` already writes the verdict to `local_review_verdicts` **locally** (no network, no `DryRun` guard — correct, because the local verdict store is a cache, not a ref/object mutation). `forge_reviews` already proves the field shape the local PR object mirrors. `but-rules` already provides the Trigger→Filter→Action engine the auto-hook plugs into. The genuinely-new work — three additive tables, the `but review` verb surface, the project setting, the agent-tag derivation, and the reconciler usage doc — is sized honestly in [01-scope-delta.md](./01-scope-delta.md) and lands as a **new appended sprint** (this enrichment's **Sprint 07** in its own numbering; see the frozen-aware note below).

## Frozen-aware: a new sprint, no frozen edits

- **Net-additive.** Changes no existing scope, no gate decision, no code set, and **no line of `merge_gate.rs` / `review_requirement.rs`**. Adds three `but-db` tables + a CLI/API surface + a project setting only.
- **Sprints 01a–06b are FROZEN** (in-flight agents). This enrichment **edits none of them**.
- **Sprint slot is human-directed: LPR = Sprint 07, STEER → Sprint 08.** The v1.4.0 STEER enrichment had claimed Sprint 07 on the live roadmap; per instruction-precedence #1 the human reassigned the priority — LPR takes **Sprint 07** and STEER is renumbered to **Sprint 08** (a numbering-only change; STEER's scope is untouched). LPR depends on Sprint 05's CLI surface and the shipped merge gate; it does **not** depend on STEER, so the renumber is clean. The N1 renumber + the LPR ROADMAP row are applied in Stage 2 (`/kb-sprint-plan`); see [05-delta-replan.md](./05-delta-replan.md) §5.

## Documents in this enrichment

| File                                     | Section                                                                                          |
| ---------------------------------------- | ------------------------------------------------------------------------------------------------ |
| [00-overview.md](./00-overview.md)       | This file — the reconciler thesis + the safe-seam invariant + the four drive gaps                |
| [01-scope-delta.md](./01-scope-delta.md) | In scope / out of scope for the enrichment (net-add); what is preserved from sprints 1–8 + STEER |
| [02-uc-lpr.md](./02-uc-lpr.md)           | LPR functional group + use cases (UC-LPR-01..07)                                                 |
