---
sprint: 09
sequence: 11
timeline: Phase 6 — Remediation (post-audit hardening)
status: Done
proposed_by: rust-planner + sveltekit-planner
milestone: sprint-09-governance-remediation-lpr-mgmt-hardening
prd: ../../README.md
roadmap: ../../ROADMAP.md
generated_by: kb-sprint-tasks-plan
generated_at: 2026-06-23T13:30:00Z
origin: |
  NOT PRD-derived. Identified by an independent codebase investigation
  (6 parallel subagents: 5 code-tracing + 1 test-execution) on 2026-06-23
  that found Sprint 07 (LPR) and Sprint 06b (MGMT) had 6 blocking gaps
  despite their "Done" status in ROADMAP.md. Each task cites its
  `audit-finding-*` origin.
---

# Sprint 09: Governance Remediation — LPR/MGMT Hardening

**Sequence:** 11
**Timeline:** Phase 6 — Remediation (post-audit hardening)
**Status:** Done — closed 2026-06-23 (16/16 tasks merged to master at 396cdabab3)
**Proposed by:** `rust-planner` + `sveltekit-planner` (parallel dispatch 2026-06-23)
**Milestone:** — (`sprint-09-governance-remediation-lpr-mgmt-hardening`)

## Overview

This sprint is **remediation, not new feature work.** An independent codebase audit on 2026-06-23
found that Sprint 07 (LPR) and Sprint 06b (MGMT) had 6 blocking gaps despite ROADMAP.md claiming
them "Done":

1. **Branch Gates tab was a stub** — `BranchGatesList.svelte` (634 lines, fully implemented) was
   never imported or rendered in `GovernanceSettings.svelte`. Human Testing Gate Sprint 06b Step 1
   ("toggle Protected branch for a pattern") would visibly fail.
2. **`but review comment/comments/resolve` CLI verbs were stubbed or missing** — `comment_review`
   returns `task_contract_invalid` after the auth gate; the `comments` and `resolve` CLI verbs
   don't exist despite real backends (`post_comment`, `list_comments`, `resolve_thread`) existing.
   Sprint 07 Human Gate Steps 4 & 5 are unrunnable.
3. **`keep_reviews_local` UI toggle silently doesn't persist** — `From<Project> for UpdateRequest`
   explicitly discards the field. UI fires mutation, value silently dropped, next read defaults
   back to `true`.
4. **`process_commit_rules` is unwired** — the auto "review-requested" hook is real and tested
   but called only from tests. `but commit` never invokes it. The "auto-opened assignment so
   reconciler can dispatch reviewer" claim (UC-LPR-06 AC-4) is unreachable in production.
5. **LPR-009 safe-seam invariant is incomplete** — bidirectional forged-vs-empty equivalence test,
   3-step chained capstone, and sub-cases are MISSING. Honesty grep is in wrong file. SPRINT.md
   for sprint-07 says: "If LPR-009 is not green, the slice is NOT done."
6. **Test suites failing on master** — `lpr_review_reads` 2/4 fail, `gitbutler-tauri` crate has
   compile break, `but` CLI has 3 snapshot drifts, desktop CT suite CRASHES on default invocation.

Sprint 09 closes these gaps so the Sprint 07 + Sprint 06b "Done" claims become truthful.

## Human Testing Gate

**Gate:** A maintainer observing the desktop Branch Gates panel after running the full local review loop (`but review request → comment → comments → resolve → approve`) sees open assignments plus unresolved threads update at each step, with the Keep Reviews Local toggle persisting across reload.

## Test Deliverable

1. Run `but review comment <branch> --file f --line N --thread t -m "…"` → comment recorded
2. Run `but review comments <branch>` → listed thread appears
3. Run `but review resolve <branch> --thread t` → thread marked resolved
4. Open Governance page → Branch Gates tab → toggle a protected-branch gate
5. Toggle Keep Reviews Local in settings → reload → observe value persists
6. Commit on a branch with a review-requested rule → observe auto-assignment
7. Open Local Review panel → observe approval status plus source branch

## Tasks

| ID             | Title                                                                                      | Agent                 | Estimate |
| -------------- | ------------------------------------------------------------------------------------------ | --------------------- | -------- |
| LPR-REM-001    | Replace `comment_review` stub with real `post_comment` call (add `--file/--line/--thread`) | rust-implementer      | 180 min  |
| LPR-REM-002    | Add `but review comments` and `but review resolve` CLI verbs                               | rust-implementer      | 150 min  |
| LPR-REM-003    | Persist `keep_reviews_local` through `UpdateRequest` + `Storage::update()`                 | rust-implementer      | 120 min  |
| LPR-REM-004    | Wire `process_commit_rules` into `but commit` production path                              | rust-implementer      | 180 min  |
| LPR-REM-005    | Add `open_assignments`/`unresolved_threads` to Tauri `review_status` payload               | rust-implementer      | 90 min   |
| LPR-REM-006    | Complete LPR-009 safe-seam invariant (bidirectional equivalence + capstone)                | rust-reviewer         | 180 min  |
| LPR-REM-007    | Surface `kind` in `GovernancePrincipalListEntry` (Rust half)                               | rust-implementer      | 60 min   |
| LPR-REM-007-UI | Mount `LocalReviewView.svelte`; consume `kind` for agent/human badge (UI half)             | sveltekit-implementer | 60 min   |
| LPR-REM-008    | Regenerate stale `but` CLI snapshots for STEER enrichment                                  | rust-implementer      | 30 min   |
| LPR-REM-009    | Remove/fix untracked `list_workspace_rules_scoped.rs`                                      | rust-implementer      | 30 min   |
| MGMT-REM-001   | Wire `BranchGatesList.svelte` into `GovernanceSettings.svelte`                             | sveltekit-implementer | 30 min   |
| MGMT-REM-002   | Remove illegal test-import stub `PrincipalEditorInherited*.spec.ts`                        | sveltekit-implementer | 30 min   |
| MGMT-REM-003   | Add `but group delete` CLI verb (replace `group_no_delete_cli_verb_surface` test)          | rust-implementer      | 90 min   |
| MGMT-REM-004   | Strengthen `BuildGates.spec.ts:204` lint-gate assertion                                    | sveltekit-implementer | 30 min   |
| MGMT-REM-005   | Align root `test:ct` command (or docs) for desktop CT reachability                         | sveltekit-implementer | 30 min   |
| STEER-REM-001  | Add STEER fields to `branch/apply.rs` `commit_gate_cli_error` serializer                   | rust-implementer      | 30 min   |

## Source Coverage

- **NOT PRD-derived.** Coverage is against the audit findings from the 2026-06-23 codebase investigation.
- **Closes Sprint 07 / Sprint 06b gaps** so their "Done" claims become truthful.

## Capability Coverage

- **CAP-AUTHZ-01** — restored by LPR-REM-001/002 (real comment/resolve verbs gate via `CommentsWrite`/`ReviewsWrite`); STEER-REM-001 (uniform denial shape on `branch apply`).
- **CAP-CONFIG-01** — restored by LPR-REM-003 (`keep_reviews_local` write path) and MGMT-REM-001 (Branch Gates UI mutates `gates.toml`).
- **CAP-LPR-08** — `review_status` reconciler payload carries full drive state (LPR-REM-005); auto-hook fires (LPR-REM-004).
- **CAP-STEER-01** — uniform-shape invariant extended to all 4 commit-gate CLI sites (STEER-REM-001).

## Blocks

None (terminal remediation sprint).

## Dependencies

- **Dependent on:** Sprint 00 (walking skeleton), Sprint 04 (merge strictness), Sprint 05 (CLI surface), Sprint 06b (UI), Sprint 07 (LPR), Sprint 08 (STEER).
- **Intra-sprint edges:**
  - `LPR-REM-001` → `LPR-REM-002` (CLI comments/resolve verbs need comment verb real)
  - `LPR-REM-001` + `LPR-REM-002` + `LPR-REM-009` → `LPR-REM-005` (status payload needs real comment data + tauri compile fixed)
  - `LPR-REM-007` → `LPR-REM-007-UI` (UI half consumes Rust kind field)
  - `LPR-REM-001` + `LPR-REM-002` + `MGMT-REM-003` + `STEER-REM-001` → `LPR-REM-008` (regenerate snapshots AFTER all CLI changes land)

## Task Detail Files

Generated by `/kb-sprint-tasks-plan` on 2026-06-23T13:30:00Z. 16 task files (alphabetical):

- `LPR-REM-001-replace-comment-review-stub-with-real-post-comment-call.md`
- `LPR-REM-002-add-but-review-comments-and-but-review-resolve-cli-verbs.md`
- `LPR-REM-003-persist-keep-reviews-local-through-updaterequest.md`
- `LPR-REM-004-wire-process-commit-rules-into-but-commit-production-path.md`
- `LPR-REM-005-add-open-assignments-unresolved-threads-to-tauri-review-status-payload.md`
- `LPR-REM-006-complete-lpr-009-safe-seam-invariant.md`
- `LPR-REM-007-surface-kind-in-governanceprincipallistentry-rust-half.md`
- `LPR-REM-007-UI-mount-localreviewview-svelte-consume-kind-ui-half.md`
- `LPR-REM-008-regenerate-stale-but-cli-snapshots-for-steer-enrichment.md`
- `LPR-REM-009-remove-fix-untracked-list-workspace-rules-scoped-rs.md`
- `MGMT-REM-001-wire-branchgateslist-svelte-into-governancesettings-svelte.md`
- `MGMT-REM-002-remove-illegal-test-import-stub-from-governance-ct-suite.md`
- `MGMT-REM-003-add-but-group-delete-cli-verb.md`
- `MGMT-REM-004-strengthen-buildgates-spec-ts-lint-gate-assertion.md`
- `MGMT-REM-005-align-root-test-ct-command-with-desktop-ct-documentation.md`
- `STEER-REM-001-add-steer-fields-to-branch-apply-rs-commit-gate-cli-serializer.md`
