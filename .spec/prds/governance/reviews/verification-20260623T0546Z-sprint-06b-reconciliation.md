# Sprint 06b Reconciliation — Verification Report

**Generated:** 2026-06-23 (kb-run-sprint PHASE 0 discovery)
**Verifier:** opencode orchestrator (merge-tree + scratch-worktree dry run)
**Scratch worktree:** `/private/var/folders/.../T/opencode/verify-06b-merge` (branch `kb-verify-06b-merge`)

## TL;DR

The 4 "un-done" UI tasks (MGMT-UI-009/010/011/012) are **not un-done — they are un-merged**.
The prior `/kb-run-sprint` run (state archived 2026-06-21) reviewed and committed the work
into 4 worktree branches but the orchestrator never recorded commit SHAs in metadata and
never executed the `[12.7]` merge step. The red-hat review (2026-06-23) correctly observed
"not at HEAD" because HEAD only sees mainline, not worktree branches.

**MGMT-UI-012 is the cumulative branch** — its history contains MGMT-UI-009/010/011 as
ancestors (verified via `git merge-base --is-ancestor`). Merging UI-012 alone brings all
4 tasks' work in one operation.

## What was verified

| Check                           | Result                                                                                        |
| ------------------------------- | --------------------------------------------------------------------------------------------- |
| Worktree existence              | ✅ All 4 worktrees present + on their branches                                                |
| Branch ahead of merge-base      | UI-009=20 commits · UI-010=16 · UI-011=5 · UI-012=23 (cumulative)                             |
| Merge-base vs HEAD              | All branched from `35eb196749` (41 commits behind current HEAD `4b3d506ee7`)                  |
| HEAD's 41 commits scope         | Mostly `.tmp/` evidence + `.spec/` bookkeeping + unrelated sprint-07/08 work (LPR-_, STEER-_) |
| UI-012 contains UI-009/010/111  | ✅ All 3 are ancestors — UI-012 IS the cumulative merge target                                |
| Files changed in UI-012 vs base | 33 source files (governance UI + backend + tests + SDK regen)                                 |
| Dry-run merge UI-012 → HEAD     | ⚠️ 4 conflict files, 8 conflict hunks total                                                   |

## Conflicts (all tractable — both-additions patterns)

### 1. `crates/but-api/src/legacy/governance.rs` — 1 hunk, ~195 lines

Both sides added new functions at the same insertion point (after line 710 closing brace):

- **HEAD side** (sprint-07/08 LPR/STEER work): `principal_kind_read_with_repo` + body
- **UI-012 side** (sprint-06b governance work): `effective_authority_for_principal` + body

**Resolution:** keep both functions (mechanical — adjacent additions, no overlap).

### 2. `apps/desktop/src/components/governance/GovernanceSettings.svelte` — 3 hunks

- **Hunk 1 (line 17-21):** import block — both added imports (`Button, EmptyStatePlaceholder, InfoMessage`).
  **Resolution:** union of imports.
- **Hunk 2 (line 158-177):** substantive — HEAD added a read-only InfoMessage conditional block,
  UI-012 added the 4-tab `<Tabs>` structure. Both should coexist: read-only message ABOVE the tabs
  (or as a guard). **Resolution:** requires sveltekit-implementer judgment on placement/disabled-state.
- **Hunk 3 (further down):** similar pattern.

### 3. `apps/desktop/tests/governance/GovernanceSettingsHarness.svelte` — 3 hunks

Test harness updates mirroring the production component changes. Same both-additions pattern.

### 4. `crates/but-authz/src/lib.rs` — 1 hunk, 1 line block

Both sides extended the `pub use config::{...}` re-export list with different items:

- **HEAD:** added `load_permissions_wire`, plus new `pub use denial::{...}` and `pub use menu::{...}` blocks.
- **UI-012:** added `gates_path`.

**Resolution:** union — keep all items + the new denial/menu blocks from HEAD.

## What was NOT verified (yet)

- Build + test pass on the merged result (requires resolving the conflicts first)
- Whether the 4 UI tasks actually satisfy their ACs at the merged HEAD (the prior reviews
  were against the worktree branches as-of 2026-06-21, not against current HEAD)
- Whether workspace:2 has pending changes to these same files

## Recommended path forward

1. **Dispatch a `sveltekit-implementer`** to resolve the 4 conflicts in the scratch worktree
   (already staged at `/private/var/.../verify-06b-merge`). It runs `cargo test -p but-api
branch_gates` + `pnpm test:ct -- Governance` + `pnpm -F @gitbutler/desktop check` after
   resolving, captures evidence, commits to the `kb-verify-06b-merge` branch.
2. **User coordinates with workspace:2** to clear the merge window.
3. **Orchestrator merges** `kb-verify-06b-merge` → `design/lpr-ui-design-contracts` (current
   working branch — NOT main/master directly per the concurrent-agent constraint).
4. **Post-merge REMEDIATE tasks** (A/B/C/D/E) run against the merged HEAD, no re-implementation.

## Files

- Archived prior state: `.kb-run-sprint/state.sprint-06b-governance-ui-branch-gates-rules-safety.20260623T054551ZZ.pre-redhat-reconciliation.json`
- Current tracker: `.kb-run-sprint/state.json` (9 active tasks)
- Red-hat review: `.spec/prds/governance/reviews/red-hat-20260623T031824Z-sprint-06b.md`
- Scratch worktree: `/private/var/folders/rt/8_hh9gmj43x4763f7_l_nk3c0000gn/T/opencode/verify-06b-merge`
  (in-progress merge state with 4 conflicts marked)
