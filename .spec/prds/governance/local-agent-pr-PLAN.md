# Local Agent PR — Master Plan & Status (don't-forget anchor)

**Purpose.** One durable place that captures the whole **Local Agent PR / governed-review parity** effort —
what it is, the locked decisions, the 3-stage planning pipeline, current status, and what's left — so it
survives across threads/sessions. This is the index; the detail lives in the linked artifacts.

**Last updated:** 2026-06-21 · **Owner:** governance PRD · **Driving directive:** human goal (see §Decisions)

---

## 1. What this is

Give GitButler's **local** review layer GitHub-PR parity (reviewer assignment, comment threads, a derived PR
lifecycle, a per-project local-by-default setting, an automatic agent-PR tag) so an orchestrator drives the
whole implement→review→merge loop off `but`'s own review state — **a reconciler over `but`, not a private
state machine** — with agent PRs kept **local by default** and the merge gate's land-truth **untouched**.

Two related-but-separate bodies of work:

- **ENGINE** (Rust `but` + but-db + but-api/CLI/napi + project setting) — _being built in another thread._
- **CONSUMING SKILLS** (`but-*` skills drive off the new state) — documented separately, lands after the engine.

This doc is the **planning** for the engine work (PRD → sprint → task files) plus the index to the skills doc.

## 2. Locked decisions (human-directed — instruction-precedence #1)

| Decision         | Value                                                                                                                                                                 |
| ---------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Scope option     | **(2) Full local PR** — assignments **and** comment threads (not the minimal assignments-only)                                                                        |
| "Keep PRs local" | A **per-project setting**, **default LOCAL** (`Project.keep_reviews_local: DefaultTrue`); remote mirroring gated behind it; the mirror bridge is **named, not built** |
| Agent-PR tag     | **Automatic** `agent-authored` tag derived from the confined opening principal (never a caller arg)                                                                   |
| Skills behavior  | The `but-*` skills **auto-set these PRs local** on governed-project init (consumer contract; see skills doc)                                                          |
| **Sprint slot**  | **LPR = Sprint 07**; the existing **STEER sprint is renumbered 07 → 08** (the "N1" renumber)                                                                          |
| Hard invariant   | The merge gate reads **only** `local_review_verdicts` at head; the two new tables are additive drive-metadata that **never gate** (safe-seam, proven by file:line)    |

## 3. Artifacts (where everything lives)

| Artifact                             | Path                                                       | Role                                                                                                                                                                                                                                                           |
| ------------------------------------ | ---------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **PRD enrichment** (Stage 1 output)  | `.spec/prds/governance/enrichments/v1.5.0-local-agent-pr/` | The feature spec: 00-overview · 01-scope-delta · 02-uc-lpr (UC-LPR-01..07) · 03-technical-requirements-delta (tables/verbs/§E safe-seam/blast-radius/R18-20) · 04-e2e-testing-criteria · 05-delta-replan (Sprint 07 proposal + N1 renumber + I-edits) · README |
| **Skills migration plan** (consumer) | `.spec/prds/governance/skills-migration-local-agent-pr.md` | W1–W9: how the `but-*` skills rework to drive off the new state, _after_ the engine lands                                                                                                                                                                      |
| **Live ROADMAP**                     | `.spec/prds/governance/ROADMAP.md`                         | Stage 2 edits this (N1 renumber + add Sprint 07 (LPR))                                                                                                                                                                                                         |
| **TaskList**                         | `#952` (Stage 1) · `#953` (Stage 2) · `#954` (Stage 3)     | progress tracking                                                                                                                                                                                                                                              |

## 4. The 3-stage planning pipeline + status

### Stage 1 — PRD enrichment (`/kb-prd-plan`) · STATUS: ✅ content complete · 🔵 red-hat review in flight

- The `v1.5.0-local-agent-pr/` enrichment was found ~80% pre-authored (excellent, grounded). Completed it:
  authored the missing `05-delta-replan.md` + `README.md`; **reconciled the sprint numbering** from an earlier
  "Sprint 08 (LPR)" auto-numbering to the directed **LPR = Sprint 07, STEER → 08** across 00/01/03.
- A fresh red-hat `rust-reviewer` is validating groundedness (claims vs. real code), scope coverage, the
  safe-seam, AC testability, and number consistency. **Next:** remediate any CRITICAL/MEDIUM, then Stage 1 done.

### Stage 2 — sprint plan (`/kb-sprint-plan`) · STATUS: ⬜ pending

- **N1 renumber (STEER 07 → 08):** in `ROADMAP.md` (the Sprint 07 STEER row + details + dependency edges);
  rename `tasks/sprint-07-steer-capability-aware-denials/` → `tasks/sprint-08-steer-capability-aware-denials/`
  (and "Sprint 07"→"Sprint 08" inside its SPRINT.md + task files); update the v1.4.0 STEER enrichment refs
  (`enrichments/v1.4.0-capability-aware-denials/05-delta-replan.md` §2/I4 + `README.md`). **Numbering-only —
  STEER scope unchanged.**
- **Add Sprint 07 (LPR):** append the `sprint-07-local-agent-pr` row + per-sprint details + dependency edges
  (depends on 01b/04/05; NOT on STEER) to `ROADMAP.md`; `sprint_count` 9 → 10.

### Stage 3 — task files (`/kb-sprint-tasks-plan`) · STATUS: ⬜ pending

- Generate `tasks/sprint-07-local-agent-pr/` SPRINT.md + per-task files for **LPR-001..010** (see
  `05-delta-replan.md` §2): the 2 tables + migrations, the typed enum, the 6 `but review` verbs + CLI, the
  derived lifecycle + agent tag, the `keep_reviews_local` setting, the `but-rules` auto-hook, the **safe-seam
  proof** (build-gate grep + forged-table test), the reconciler/skill-contract doc, and SDK regen.

## 5. Count delta (v1.4.0 → v1.5.0)

7 groups (+LPR) · 30 UCs (+7) · 20 risks (+R18/R19/R20) · ROADMAP 10 sprints (LPR=07, STEER→08).
AC/criteria totals filled honestly from 02/04 at I-edit time.

## 6. Guardrails carried through every stage

- **Net-additive / frozen-aware:** edits no frozen sprint's _scope_; the N1 STEER renumber is numbering-only.
- **Safe-seam is load-bearing:** no stage may wire the new tables into `merge_gate.rs` / `review_requirement.rs`.
- **Grounded, not faked:** every technical claim cites real `but` code; ungrounded spec is the cardinal sin.
- **Risks named, never mitigated-closed:** R18 (loop-sourced-receipt forgeability), R19 (tag spoof via
  `BUT_AGENT_HANDLE` re-export), R20 (comment-body injection) stay named as accepted residuals.
- **Uncommitted by default:** these `.spec/` planning artifacts live alongside other in-flight governance work
  in this worktree; bundle/commit them via the `but` workflow when ready.
