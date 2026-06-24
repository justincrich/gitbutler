# Skills Migration Plan — Driving `but-*` off the Local Agent PR

**Status:** Forward-looking. The engine work (local review/PR parity — "Local Agent PR") is being
built in a separate thread (tracked as the new governance **Sprint 07 — Local Agent PR / governed-review
parity**, with the prior STEER sprint renumbered to 08). **This document assumes that engine work is
largely done** and captures _all_ the `but-*` **skill** changes required to actually use it — i.e. to move
orchestration weight off the agents and onto `but`.

This is a SKILLS plan, not an engine plan. Engine deliverables are listed only as **preconditions**.

Audience: whoever picks up the skills rework once Sprint 07 lands. The Work Breakdown (§5) is written so
it decomposes 1:1 into a TaskList.

---

## 1. The shift this enables

Today the **orchestrator is the state machine**: it keeps the task lifecycle in its own tracker
(Claude TaskList / Codex `state.json`), decides each transition itself, and _calls_ `but` only at the
commit/merge moments. `but` is a gate it visits.

After this migration, **`but` holds the canonical review/PR state and the orchestrator becomes a
reconciler**: it continually reads `but` and drives every branch toward merged — dispatch the _assigned_
reviewer because `but` shows commits-with-no-verdict, remediate because `but` shows changes-requested with
open comments, merge because `but` shows an approval at head, pull the next task because `but` shows the
last one merged. Agents **work the system**: they read assignment/feedback/lifecycle _from `but`_ and react,
instead of trusting the orchestrator's private bookkeeping or a prompt-relayed copy.

This is the (c)/(d) win from the brainstorm: the loop logic stops living in fragile prompts and starts
living in queries against a shared engine every harness sees identically.

### The division of truth (keep this straight — it governs every change below)

| Axis                                                                                                                                  | Owner after migration              | Source                                                              |
| ------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------- | ------------------------------------------------------------------- |
| **Process / review lifecycle** — who's assigned, is a review requested, what feedback exists, is it approved-at-head, merge-readiness | **`but`** (the new local-PR store) | `but review status / comments / list`                               |
| **Land authorization** (the non-bypassable gate)                                                                                      | **`but`** (unchanged)              | `local_review_verdicts` distinct-verdict-at-head — **do not touch** |
| **Completion / substance** — is every AC satisfied, zero stubs (the thing the gate can't see)                                         | **spec + tracker** (unchanged)     | `metadata.requirements[]` iron gate                                 |

The migration moves the **process axis** into `but`. It does **not** change the gate, and it does **not**
move substance/AC-satisfaction out of the tracker. Don't conflate them.

---

## 2. Preconditions (assumed delivered by the engine thread)

The skills work below depends on these existing in the default-feature `but` build. If any is missing when
we start, that work item is **BLOCKED**, not faked.

- **Local PR/review object** — a per-branch local review with lifecycle state (draft / open /
  changes-requested / approved / merged / closed), author principal, source/target branch. (Mirrors the
  existing `ForgeReview` shape, localized.)
- **Reviewer assignment** — `but review request <reviewer>` / `but review assign`, stored locally
  (`local_review_assignments`), queryable.
- **Local feedback thread** — `but review comment` writing a **local** comment (file/line/body/thread,
  resolved-state) into `local_review_comments`, queryable — not the remote-forge comment path.
- **Read surface** — `but review status <branch>` and/or `but review list` returning, as structured JSON:
  lifecycle state, assigned reviewer(s), verdict-at-head, and open/unresolved comment count. **The
  reconciler is only as good as this read surface; it must be machine-parseable.**
- **Gate-truth invariant preserved** — the merge gate still reads **only** `local_review_verdicts`
  (distinct verdict at head). The new objects are adjacent metadata; merging still requires
  `but review approve` at head. (Confirmed safe-seam: the gate never reads assignment/comments/lifecycle.)
- **Authorities reused** — `reviews:write` (approve / request / assign) and `comments:write` (comment).
  No new authority; the principal model is unchanged.
- _(Optional, if shipped)_ **Rules-engine auto-hook** — a `workspace_rules` trigger that flips a branch to
  _review-requested_ on commit to a feature branch. If present, the orchestrator reacts to the state rather
  than issuing the request itself.

> If the engine ships different command/field names, treat the names here as placeholders and reconcile in
> W1 (conventions) first — every other work item reads from W1.

---

## 3. Design principles for the migration

1. **`but` is the source of truth for process; read it, don't re-derive it.** Every place a skill currently
   keeps process state in the tracker or relays it through a prompt, replace with a `but` query.
2. **Never weaken the gate.** The merge gate stays exactly as-is. The reconciler may _decide when to attempt_
   `but merge`, but `but` decides whether it lands. No skill ever bypasses or front-runs the verdict-at-head.
3. **Deterministic query, probabilistic judgment.** Reading `but review status` and deciding the next action
   from its fields is **deterministic** → lives in a script / transition helper, not agent prose. The
   _review itself_ (judging ACs, detecting stubs, writing the comment) is **probabilistic** → the dispatched
   reviewer agent. Keep the boundary crisp (deterministic-vs-probabilistic rule).
4. **Honest tiering (SUPREME RULE).** Every changed step keeps a real-`but` (no-mock) acceptance and a
   degraded fallback that labels missing pieces **BLOCKED**, never done. Never fake `but review status`.
5. **Cross-harness.** All changes hold for both Claude Code (TaskList) and Codex (`state.json`). The new
   `but` reads are harness-agnostic; the tracker abstraction (`tt.*`) is unchanged.
6. **Backward-compatible.** Governance-absent and older-`but` repos still run via the existing
   verdict-only path; the PR-driven path activates only when `but review status` is available.

---

## 4. What the reconciler looks like (illustrative — the target of §5)

```
# PHASE 2 becomes a reconciler over `but` state, NOT a private tracker state machine.
for each active branch B in `but` workspace:
  s = `but review status B`        # {lifecycle, assigned_reviewer, verdict_at_head, open_comments}
  match s.lifecycle:
    no-commits                  -> implementer in flight → facilitate (unchanged WIP-recovery)
    commits, not review-requested
        -> if auto-hook present: wait for it; else `but review request <assigned>`   # but is now the assigner
    review-requested, no verdict-at-head
        -> dispatch the ASSIGNED reviewer read from `but` (NOT re-resolved from RULES.md)
    changes-requested (open_comments>0)
        -> dispatch remediation; implementer reads the thread via `but review comments B`, not a relay
    approved-at-head            -> `but merge B`   (gate enforces; on gate.review_required re-review)
    merged                      -> pull next ready task

  # SUBSTANCE still gates marking the *task* complete in the tracker (requirements[] iron gate) — unchanged.
  # AUDIT is now an independent re-read of `but review list`, not a loop-held receipt (see W7).
```

---

## 5. Work breakdown (task-ready)

Each item: **Scope/files · Change · Acceptance · Depends · Executor.** Executor for all is the **skill
author (direct edits to `brain/skills/but-*` + `brain/docs`)** — these are markdown + bash, not a code
domain, so there is no domain specialist; the orchestrator/author does them and syncs via `skillshare`.
Run the `prompt-optimizer` over new agent-facing prose and keep Agent-Skills spec compliance (quoted
`description`, etc.).

### W1 — Update the shared contract (`BUT-SKILL-CONVENTIONS.md`)

- **Scope:** `brain/docs/BUT-SKILL-CONVENTIONS.md` (§2 governance model, §3 denial contract).
- **Change:** Add the **local-PR layer** to the model: the review/PR object, assignment, comment thread,
  and the `but review request/assign/comment/status/list` surface — with the explicit **division of truth**
  table from §1 (process→`but`, gate→`local_review_verdicts`, substance→tracker). State the reconciler
  principle. Lock the placeholder command/field names against the shipped engine so every downstream skill
  references one canonical vocabulary.
- **Acceptance:** Conventions doc names every new command + the three-axis division; downstream skills cite
  it rather than re-describing the model. `but review status --help` (or equivalent) asserted to exit 0
  before any skill relies on it (mirror the existing `perm --help` preflight).
- **Depends:** Engine names finalized.

### W2 — `but-run-sprint`: reconciler loop (the core change)

- **Scope:** `but-run-sprint/docs/algorithm.md` (PHASE 2 §[10]–[12]), `docs/transition-helpers.md`,
  `SKILL.md` (PHASES + Authority + the REUSE/REPLACE table).
- **Change:** Recast PHASE 2 from a tracker-driven dispatch loop into the **reconciler over `but` state**
  in §4. Add a deterministic `but-review-state` read (new helper/script, W6) as the loop input. The
  tracker keeps owning **completion truth** (requirements[]), but **process transitions** (needs-review,
  changes-requested, merge-ready) are now _read from `but`_, not computed privately. "Don't let a branch
  sit open" becomes a literal property of the reconciler: every cycle advances every branch toward merged.
- **Acceptance:** On a real governed sprint, the loop dispatches review/remediation/merge **purely from
  `but review status`**; killing and restarting the orchestrator mid-sprint resumes correctly because state
  lives in `but`, not the dead process. Verdict-at-head dismissal on remediation still drives re-review
  (unchanged invariant). No private process-state duplicated in the tracker.
- **Depends:** W1, W6.

### W3 — `but-run-sprint`: assignment lives in `but`

- **Scope:** `docs/algorithm.md` (reviewer-dispatch step [12.4]), `docs/specialist-resolution.md`,
  `SKILL.md` SPECIALIST RESOLUTION.
- **Change:** Specialist-resolution (RULES.md roster → reviewer) now **records the choice in `but`**
  (`but review request <reviewer>` / assign) at the moment a branch needs review, making `but` the source
  of truth for "who's assigned." Reviewer dispatch then **reads the assignee back from `but`**, so any
  harness (or a human in the desktop UI) sees the same assignment. Resolution becomes "assign once, in
  `but`," not "re-resolve from prose every cycle."
- **Acceptance:** `but review status` shows the assigned reviewer; a second orchestrator cycle dispatches
  the _same_ assignee by reading `but` (not by re-running resolution). Assignment respects `reviews:write`.
- **Depends:** W1, W2.

### W4 — `but-run-sprint`: feedback flows through the `but` PR (templates)

- **Scope:** `templates/reviewer-prompt.md`, `templates/implementer-prompt.md`,
  `templates/remediation-prompt.md`.
- **Change:**
  - **Reviewer** posts substantive feedback as **`but review comment`** (file/line/body) on the PR object,
    then records the verdict (`approve`/`request-changes`) as today. The structured `requirements[]` JSON
    stays (it feeds the tracker's substance axis), but the _human-readable feedback channel becomes the
    `but` thread_ — so it's durable, queryable, and visible to every agent and the desktop UI.
  - **Implementer / remediation** agents **read open comments from `but review comments <branch>`** and
    address them, marking threads resolved — instead of receiving feedback relayed in the orchestrator's
    remediation prompt. "Subagents read feedback from the `but` PR" becomes literally true.
  - Extend the **reactive denial contract** (already in these prompts) to the new verbs: a `but review
comment/request` denial is read and surfaced, never worked around.
- **Acceptance:** A remediation cycle works end-to-end with the implementer reading the reviewer's comments
  _from `but`_, not from the prompt; resolved threads show resolved in `but review status`.
- **Depends:** W1, W2.

### W5 — `but-init` / `but-migrate`: seed the review policy (+ optional auto-hook)

- **Scope:** `but-init/SKILL.md`, `but-init/scripts/seed-governance.py`, `but-migrate/SKILL.md`.
- **Change:** Extend seeding beyond `permissions.toml` / `gates.toml`:
  - Optionally seed a **default reviewer-assignment policy** (roster area → `code-reviewers` member) so the
    orchestrator's W3 assignment has a sensible default.
  - If the engine ships the **rules-engine auto-hook**, seed the `workspace_rules` entry that flips a branch
    to _review-requested_ on commit, so review state appears without the orchestrator issuing it. Gate this
    behind the engine-capability check; **BLOCKED**-label if the rules surface isn't present.
- **Acceptance:** Post-init, `but review status` on a fresh feature commit reflects the seeded policy
  (assignment default and/or auto-requested state). Selftest covers the new seed output (no `but` needed for
  the pure-transform selftest, real `but` for the runtime assertion).
- **Depends:** W1; engine rules-surface (for the auto-hook half only).

### W6 — Deterministic `but`-state read helper(s)

- **Scope:** `but-run-sprint/scripts/` (new, e.g. `but-review-state.sh`), referenced from `docs/algorithm.md`.
- **Change:** A small deterministic script wrapping `but review status/list` → normalized JSON the
  reconciler consumes (`{branch, lifecycle, assigned, verdict_at_head, open_comments}`), fail-soft and
  cross-harness. This is the **deterministic seam** between `but` and the loop (keeps process decisions out
  of agent prose).
- **Acceptance:** Run for real against a governed repo; returns correct normalized state for each lifecycle
  case; degrades to a labeled BLOCKED (not a fake) when `but review status` is unavailable.
- **Depends:** Engine read surface.

### W7 — Upgrade the audit from _receipt_ to _independent re-read_

- **Scope:** `but-run-sprint/scripts/record-governed-land.sh`, `scripts/build-run-summary.sh`,
  `references/output-format.md`, `docs/worktree-lifecycle.md` [12.7.2].
- **Change:** The current GOVERNED-LAND AUDIT is a _receipt_ sourced from loop-held facts (because no CLI
  surfaced verdicts). With the engine read surface, the PHASE 5 audit becomes an **independent re-read of
  `but review list`** — the authoritative engine record — closing the honesty gap we documented. Keep
  `record-governed-land.sh` only as a fallback for the no-read-surface path (or retire it if the read
  surface is always present). Update the audit caption: it's now an engine re-read, not just a receipt.
- **Acceptance:** PHASE 5 audit table is populated from `but`, and matches the loop's record; the
  "receipt, not independent re-read" caveat is removed where the read surface exists.
- **Depends:** W6.

### W8 — `but-orchestrate`: observe `but` review state as a stage signal

- **Scope:** `but-orchestrate/SKILL.md`, `references/cmux-orchestration.md`.
- **Change:** The multi-surface conductor already observes durable artifacts + the cmux socket; add
  **`but` review lifecycle** as an observation source so stage routing (run → review → remediate → qa) can
  key off `but` state, not just artifacts. The done-bit owner still never self-certifies; this just makes
  "is the work reviewed/merged" a `but` query.
- **Acceptance:** Orchestrate routes a stage transition triggered by a `but` lifecycle change in a real
  multi-surface run.
- **Depends:** W2, W6.

### W9 — Conventions/version bump + changelog + sync

- **Scope:** all touched `but-*` `SKILL.md` changelogs + version frontmatter; `skillshare sync`.
- **Change:** Bump versions, write changelog entries, run `prompt-optimizer` over new agent-facing prose,
  re-verify Agent-Skills spec compliance, `skillshare sync`, commit.
- **Acceptance:** `pnpm`/spec checks clean; brain == `~/.claude` for all touched files; committed.
- **Depends:** W1–W8.

---

## 6. Reconciling work already shipped this thread

| Already shipped                                                                   | What the migration does to it                                                                                                        |
| --------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| **Reactive denial contract** (implementer/reviewer prompts)                       | Kept and **extended** to the new `but review request/comment` verbs (W4).                                                            |
| **Two-axes Authority clarification** (SKILL.md)                                   | Kept; the _process axis_ it names now has a richer `but`-owned store (W1 makes this explicit).                                       |
| **Local governed-land audit** (`record-governed-land.sh` → `governed-land.jsonl`) | **Upgraded** (W7): from a loop-held _receipt_ to an independent re-read of `but review list`. Script demoted to fallback or retired. |

---

## 7. Sequencing

```
W1 (conventions/vocab)  ──► W6 (state-read helper) ──► W2 (reconciler loop) ──► W3 (assignment-in-but)
                                                                          └──► W4 (feedback-in-but)
W5 (init/migrate seed)  ── parallel after W1 (auto-hook half gated on engine)
W7 (audit upgrade)      ── after W6
W8 (orchestrate)        ── after W2 + W6
W9 (bump/sync/commit)   ── last
```

W1 + W6 are the unblockers; everything else reads from them. W5 can proceed in parallel once vocab is locked.

## 8. Risks & open questions

- **Command/field-name drift.** If the engine ships different names, W1 absorbs it and everything else
  references W1. Don't hardcode names in multiple skills.
- **Read-surface shape.** The reconciler needs `but review status` to be machine-parseable JSON. If it's
  human-prose only, W6 is harder (and we may need an engine ask). Flag early.
- **Auto-hook availability.** The rules-engine auto-request is the only piece that may not ship in Sprint 07.
  W5's auto-hook half and the loop's "auto vs self-request" branch must degrade cleanly to self-request.
- **Don't over-move substance.** Resist the temptation to push AC-satisfaction/anti-stub into `but`. That's
  the probabilistic reviewer's job; `but` owns process + gate, not whether the work is _good_.
- **End-to-end proof still owed.** None of this is "done" until watched against real `but` (the same bar as
  Wave 6 #946). Each work item carries a real-`but` acceptance; the migration is not complete on green tests.

## 9. Checklist (→ TaskList)

- [ ] W1 — BUT-SKILL-CONVENTIONS: local-PR layer + division-of-truth + locked vocab
- [ ] W6 — deterministic `but-review-state` read helper (normalized JSON, fail-soft)
- [ ] W2 — `but-run-sprint` reconciler loop (PHASE 2 reads `but` state)
- [ ] W3 — assignment recorded in / read from `but` (specialist-resolution → `but review request`)
- [ ] W4 — feedback through the `but` PR (reviewer comments; implementer reads thread; denial contract extended)
- [ ] W5 — `but-init`/`but-migrate` seed review policy (+ auto-hook if engine ships it)
- [ ] W7 — audit upgraded to independent `but review list` re-read
- [ ] W8 — `but-orchestrate` observes `but` review lifecycle
- [ ] W9 — version bump + changelog + prompt-optimizer + spec-check + skillshare sync + commit
