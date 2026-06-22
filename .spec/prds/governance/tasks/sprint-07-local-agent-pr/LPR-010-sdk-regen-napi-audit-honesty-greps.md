# LPR-010: TS SDK regen + N-API audit (R14) + happy-path CLI tests + honesty/anti-fakeability greps + drive-layer-integrity proofs (R22)

> Status: ✅ Completed
> Commit: af2e76caed
> Reviewer: deferred to PHASE 4.5 red-hat closeout — SDK regen unblocked + R14 N-API audit + honesty greps + T-LPR-043/044 integrity proofs; all gates green
> Updated: 2026-06-22T18:07:12Z


## What this does

Close out the LPR slice with the cross-cutting audit lane: (1) regenerate the TS SDK (`pnpm build:sdk && pnpm format`) for the six new `#[but_api(napi)]` verbs (`request_review`/`assign_reviewer`/`post_comment`/`list_comments`/`resolve_thread`/`review_status`) and verify it type-checks; (2) audit (R14) that the regenerated N-API bindings route through the gated `but-api` fns, not a parallel ungated path; (3) add the two **honesty/anti-fakeability greps** — `__pr_meta__` is a **reserved/rejected** `thread_id` (the opener itself lives in the dedicated `local_review_meta` table — the agent-tag source is sourced from a table, **not a comment body** — so `__pr_meta__` is purely the R23 negative control proving a comment-body sentinel cannot forge the tag), and the `agent-authored` tag is referenced by **no enforcement path** (T-LPR-022); (4) add the happy-path `but review *` CLI snapbox tests; and (5) add the **drive-layer-integrity proofs** (R22): a self-assignment is rejected (T-LPR-043) and an unauthorized self-resolve cannot suppress a remediation signal (T-LPR-044).

## Why

Sprint 07 · PRD UC-LPR-01, UC-LPR-02, UC-LPR-04, UC-LPR-07 · capability CAP-AUTHZ-01. This is the reviewer-owned closeout (SDK/N-API audit + honesty greps + the R22 drive-layer-integrity proofs). The reconciler-usage-model + the `but-*` skill-contract doc are a SEPARATE task (LPR-011). The six verbs are `#[but_api(napi)]` so the SDK must regenerate (RULES.md "SDK generation flow"); the Electron lite app reaches them via N-API and R14 requires every consequential N-API route to go through the gated `but-api` seam. The two honesty greps are the build-time proof that the agent tag is **descriptive metadata, not an enforcement key**, and that `__pr_meta__` is a reserved/rejected `thread_id` (the opener is recorded in the dedicated `local_review_meta` table — sourced from a table, not a comment body — so `__pr_meta__` is the R23 negative control: a comment-body sentinel can't forge the tag). The R22 drive-layer-integrity proofs are the closeout audit of LPR-003's `assign_reviewer` distinct-from-author constraint (a self-assignment is rejected — T-LPR-043) and LPR-004's `resolve_thread` resolver-identity constraint (an unauthorized self-resolve cannot forge an all-clear and suppress another party's remediation signal — T-LPR-044).

## How to verify

PRIMARY **AC-1** — `pnpm build:sdk && pnpm format && grep -rq "reviewStatus\|review_status" packages/but-sdk/src/generated && pnpm -F @gitbutler/desktop check`: the regenerated SDK carries the six new verbs and the desktop TS type-checks; a hand-edit is overwritten by the regen. Full gate set in the spec below.

## Scope

- packages/but-sdk/src/generated/\*\* (REGENERATE ONLY via `pnpm build:sdk && pnpm format` — NEVER hand-edit; this is the regen gate for all six LPR verbs)
- crates/but-authz/tests/invariant_build_gates.rs (MODIFY — ADD the `agent-authored`-not-an-enforcement-key net-new grep pattern over the existing ENFORCEMENT_PATHS; ADDITIVE only — never weaken/remove an existing pattern) OR crates/but-authz/tests/lpr_honesty_gates.rs (NEW — a sibling grep test in the same `assert_grep_has_no_matches` discipline)
- crates/but-api/tests/pr_meta_reserved.rs (NEW — the `__pr_meta__` reserved-constant proof: post_comment rejects a caller thread_id=="**pr_meta**"; list_comments/review_status never surface it as a real thread)
- crates/but/tests/ (NEW — the happy-path `but review request`/`assign`/`comment --file --line --thread`/`comments`/`resolve`/`status`/`request-changes` snapbox suite; happy-path only per RULES.md)
- crates/but-api/tests/lpr_drive_layer_integrity.rs (NEW — the R22 closeout proofs: self-assignment rejected T-LPR-043; unauthorized self-resolve rejected T-LPR-044; via real but-api + but-db + gix)
- .spec/prds/governance/tasks/sprint-07-local-agent-pr/NAPI-AUDIT.md (NEW — the R14 N-API audit note: the six verbs are `#[but_api(napi)]` fns routing through the gated but-api seam; no parallel ungated N-API route)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: LPR-010 — SDK regen + N-API audit (R14) + happy-path CLI + honesty greps + drive-layer-integrity proofs (R22)
================================================================================

TASK_TYPE:   FEATURE
STATUS:      Backlog
PRIORITY:    P1
AGENT:       implementer=rust-reviewer | reviewer=rust-reviewer
EFFORT:      M  (150 min)
PROPOSED-BY: rust-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-LPR-01, UC-LPR-02, UC-LPR-04, UC-LPR-07
CAPABILITIES:CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  pnpm build:sdk && pnpm format && grep -rq "reviewStatus\|review_status" packages/but-sdk/src/generated && pnpm -F @gitbutler/desktop check
  check: cargo check -p but-authz -p but-api --all-targets
  lint:  cargo clippy -p but-authz -p but-api --all-targets

--------------------------------------------------------------------------------
TYPE DESIGN (Rust shapes this task introduces)
--------------------------------------------------------------------------------
NO NEW PRODUCTION TYPES — this is the audit/closeout task (SDK regen + greps + drive-integrity proofs + the NAPI-AUDIT.md note). It adds only:
  - a net-new grep PATTERN (a `&'static str` regex constant) in invariant_build_gates.rs (or a sibling test) asserting "agent-authored"/"agent_authored" appears in NO ENFORCEMENT_PATHS file — reusing the existing assert_grep_has_no_matches helper shape (invariant_build_gates.rs:126).
  - a test asserting the reserved `__pr_meta__` thread_id (LPR-004's reserved/rejected thread) is rejected as a caller-supplied thread_id and never surfaced as a real thread (the opener itself lives in the dedicated local_review_meta table — LPR-003 — so __pr_meta__ is purely the R23 negative control).
  - the R22 drive-layer-integrity proofs (closeout, real but-api): a self-assignment (assign_reviewer reviewer==author) is REJECTED with no row written (T-LPR-043, verifying LPR-003); an unauthorized self-resolve (resolve_thread by a non-author/non-assigned/non-reviews:write principal) is REJECTED and the thread stays unresolved (T-LPR-044, verifying LPR-004).
ERROR STRATEGY:
  - The grep tests return anyhow::Result and use the crate's existing assert_grep_has_no_matches / assert_paths_exist_and_non_empty helpers (invariant_build_gates.rs:111/:126); the pr_meta test uses the but-api hand-assertion idiom (Result + assert_eq!/assert!).
OWNERSHIP PLAN:
  - No new owned types. The grep test borrows the workspace root + the ENFORCEMENT_PATHS slice; the pr_meta test and the R22 drive-integrity proofs borrow the real ctx/DbHandle (the merge_gate/governed_loop fixture idiom) and drive the real verbs (assign_reviewer/post_comment/resolve_thread).
DOC POINTERS (read before coding):
  - brain/docs/rust/testing.md → #[test] build-gate / honesty-grep tests (assert_grep_has_no_matches over ENFORCEMENT_PATHS); the but-api hand-assertion idiom for the pr_meta test
  - brain/docs/rust/module-system.md → the #[but_api(napi)] -> N-API -> generated SDK surface (why regen, not hand-edit)

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
The slice is audited and closed: (1) `pnpm build:sdk && pnpm format` regenerates packages/but-sdk/src/generated with all six LPR verbs (request_review/assign_reviewer/post_comment/list_comments/resolve_thread/review_status), the generated TS type-checks (pnpm -F @gitbutler/desktop check), and a hand-edit is provably overwritten by a regen; (2) the regenerated N-API bindings for the six verbs derive from the #[but_api(napi)] fns (which authorize via authorize_branch_action) — no parallel ungated N-API path (R14 satisfied), recorded as the audit note in NAPI-AUDIT.md; (3) a net-new honesty grep proves "agent-authored"/"agent_authored" is referenced in NO ENFORCEMENT_PATHS file (merge_gate.rs, review_requirement.rs, commit/gate.rs, but-authz) — the tag is descriptive metadata, not an enforcement key (T-LPR-022); (4) a test proves __pr_meta__ is a reserved/rejected thread_id — post_comment rejects a caller-supplied thread_id=="__pr_meta__" and list_comments/review_status never surface it as a real thread (the opener lives in the dedicated local_review_meta table, so the agent-tag source is sourced from a table, not a comment body; __pr_meta__ is the R23 negative control); (5) the happy-path `but review *` CLI snapbox suite passes; (6) the R22 drive-layer-integrity proofs pass: a self-assignment (assign_reviewer reviewer==author) is REJECTED with no row written (T-LPR-043), and an unauthorized self-resolve (resolve_thread by a non-author/non-assigned/non-reviews:write principal) is REJECTED and the thread stays unresolved (T-LPR-044) — the drive layer cannot narrate a self-assigned reviewer as independently reviewed, nor let a cross-principal actor forge an all-clear and suppress another party's remediation signal; (7) the e2e full-loop capstone (T-LPR-029) runs ENTIRELY LOCAL over one governed repo with keep_reviews_local=true — request→assign→comment→resolve→approve then `but merge` all complete with no forge, the merge PROCEEDS (verdict-at-head satisfied), and no remote forge PR is opened (create_forge_review never invoked; the forge_reviews cache is unchanged) — the automated sibling of the T-LPR-029h human gate (LPR-011); cargo test -p but-authz / -p but-api / -p but green; clippy clean.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST regenerate the SDK via `pnpm build:sdk && pnpm format` ONLY — NEVER hand-edit packages/but-sdk/src/generated (RULES.md "SDK generation flow"). AC-1's negative control is that a hand-edit is overwritten by a re-run of build:sdk. If a verb is missing from the regenerated SDK, that is an LPR-003/004/005/009 bug to FLAG (the #[but_api(napi)] fn was not exposed), NOT to patch by hand-editing the generated file.
- [MUST] MUST add the agent-tag-not-an-enforcement-key grep as an ADDITIVE net-new pattern over the SHIPPED honesty-grep discipline (invariant_build_gates.rs: assert_grep_has_no_matches over ENFORCEMENT_PATHS — AUTHZ_AUTHORIZE/AUTHZ_CONFIG/COMMIT_GATE/MERGE_GATE/CONFIG_MUTATE/GOVERNANCE/FORGE_GUARD, invariant_build_gates.rs:23-31). The new pattern asserts "agent-authored"/"agent_authored" matches NOWHERE in those paths. NEVER weaken, remove, or narrow an existing pattern (ROLE_BRANCH_PATTERN / HUMAN_OR_LABEL_BRANCH_PATTERN / AUTHORITY_POSITIVE_PATTERN). (The HUMAN_OR_LABEL_BRANCH_PATTERN already bars is_human/is_ai/role-name branches; this adds the explicit agent-tag-label bar.)
- [MUST] MUST prove __pr_meta__ is a reserved/rejected thread_id: the test drives post_comment with a caller-supplied thread_id=="__pr_meta__" and asserts it is REJECTED (Err / normalized away — never written as a real comment), AND that list_comments/review_status do NOT surface the __pr_meta__ marker as a normal review thread. This is the R23 negative control: the opener principal is recorded in the dedicated local_review_meta table (LPR-003) — sourced from a table, NOT a comment body — so a comment-body sentinel cannot forge the agent-tag source. If post_comment does NOT reject __pr_meta__ today, FLAG it as an LPR-004 gap (the rejection lives in post_comment, LPR-004) — this task is the PROOF, the rejection is LPR-004's write.
- [MUST] MUST audit (R14) that the six verbs' N-API bindings route through the gated but-api fns. The verbs ARE but-api fns (they authorize via authorize_branch_action forge.rs:47), so they inherit the audited seam; the audit confirms the regenerated N-API surface derives from the #[but_api(napi)] fns and there is NO parallel ungated N-API route for any of the six. Record the audit as a structural check (grep the generated napi surface for the six verb names + confirm they map to the gated fns) + a written audit note in NAPI-AUDIT.md.
- [MUST] MUST keep the CLI tests HAPPY-PATH ONLY (RULES.md crates/but/AGENTS.md — CLI tests are expensive). Use the snapbox env.but(...).assert() idiom with .stdout_eq/.stderr_eq + [..]/... wildcards; update with SNAPSHOTS=overwrite cargo test -p but; seed via the sandbox helpers env.invoke_bash/env.invoke_git, NOT std::process::Command::new("git").
- [MUST] MUST prove the R22 drive-layer-integrity constraints as CLOSEOUT proofs against real but-api: (a) a self-assignment (assign_reviewer reviewer==the target branch author) is REJECTED with NO (refs/heads/feat, author) assignment row written — the drive layer cannot narrate a self-assigned reviewer as independently reviewed (T-LPR-043, verifying LPR-003's distinct-from-author constraint); (b) an unauthorized self-resolve (resolve_thread by a non-author/non-assigned/non-reviews:write principal) is REJECTED and the thread stays unresolved — a cross-principal actor cannot forge an all-clear and suppress another party's remediation signal (T-LPR-044, verifying LPR-004's resolver-identity constraint). R22 narrows CROSS-PRINCIPAL forgery only and stays NAMED: a single principal holding both authorship and reviews:write is NOT made multi-party. If a constraint is not yet enforced, FLAG it against LPR-003/LPR-004 — this task is the PROOF, the enforcement is the owning task's write.
- [NEVER] NEVER add or modify any but-api/but-db/but-rules PRODUCTION logic — this is the audit/closeout task. It regenerates (SDK), greps (honesty), tests (CLI happy-path + pr_meta proof + the R22 drive-integrity proofs), and records the R14 audit note. A missing verb / un-rejected __pr_meta__ / unenforced drive-integrity constraint is a FLAG against the owning task (LPR-003..009), not a fix here.
- [NEVER] NEVER weaken any existing invariant_build_gates pattern or remove a path from ENFORCEMENT_PATHS (additive only).
- [NEVER] NEVER assert a forgeable direct DB write to local_review_verdicts/local_review_assignees is blocked as if that proved R6/R18 — the R22 drive-integrity proofs verify the DRIVE-LAYER verbs (assign_reviewer/resolve_thread), not a tamper-evident storage boundary; R18 (independent audit) and R19/R20 stay accepted residuals, never presented as closed.
- [NEVER] NEVER add new gitbutler-* usage.
- [STRICTLY] STRICTLY reuse the shipped assert_grep_has_no_matches / assert_paths_exist_and_non_empty helpers (invariant_build_gates.rs:111/:126) for the agent-tag grep — do not hand-roll a parallel grep harness.
- [STRICTLY] STRICTLY treat the six #[but_api(napi)] verbs as a CONSUMED seam — this task regenerates and audits their SDK/N-API surface; it does not change the fns.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [x] AC-1 [PRIMARY]: `pnpm build:sdk && pnpm format` regenerates the SDK with all six LPR verbs; the desktop TS type-checks; a hand-edit is overwritten by the regen
- [x] AC-2: a net-new honesty grep proves "agent-authored"/"agent_authored" is referenced in NO ENFORCEMENT_PATHS file (the tag is not an enforcement key — T-LPR-022)
- [x] AC-3: __pr_meta__ is a reserved/rejected thread_id — post_comment rejects a caller thread_id=="__pr_meta__"; list_comments/review_status never surface it as a real thread (the opener lives in local_review_meta; __pr_meta__ is the R23 negative control)
- [x] AC-4: the six verbs' N-API bindings route through the gated but-api fns — no parallel ungated N-API path (R14 satisfied), recorded as the audit note in NAPI-AUDIT.md
- [x] AC-5: the happy-path `but review *` CLI snapbox suite passes (request/assign/comment --file --line --thread/comments/resolve/status/request-changes)
- [x] AC-6: the R22 drive-layer-integrity proofs pass — a self-assignment is REJECTED with no row written (T-LPR-043); an unauthorized self-resolve is REJECTED and the thread stays unresolved (T-LPR-044)
- [x] AC-7 (T-LPR-029): the e2e full-loop capstone runs ENTIRELY LOCAL — on ONE real governed repo with keep_reviews_local=true, the full CLI loop request→assign→comment→resolve→approve then `but merge` completes, the merge PROCEEDS (verdict-at-head satisfied), and NO remote forge PR was opened (no create_forge_review; the forge_reviews cache is unchanged)
- [x] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: SDK regenerates with the six verbs and type-checks; hand-edit is overwritten
  GIVEN: the six #[but_api(napi)] LPR verbs exist in crates/but-api/src/legacy/forge.rs (LPR-003/004/005/009); a clean packages/but-sdk working tree
  WHEN:  `pnpm build:sdk && pnpm format` runs, then a deliberate hand-edit to a generated file is made and `pnpm build:sdk` is re-run
  THEN:  packages/but-sdk/src/generated contains all six verbs (request_review/requestReview, assign_reviewer/assignReviewer, post_comment/postComment, list_comments/listComments, resolve_thread/resolveThread, review_status/reviewStatus); `pnpm -F @gitbutler/desktop check` exits 0 (the regenerated SDK type-checks); the hand-edit is OVERWRITTEN by the re-run (a regen is the source of truth, never a manual edit)
  TEST_TIER: integration   VERIFICATION_SERVICE: real pnpm build:sdk generation pipeline + real tsc type-check against the regenerated SDK
  VERIFY: pnpm build:sdk && pnpm format && grep -rq "reviewStatus\|review_status" packages/but-sdk/src/generated && grep -rq "postComment\|post_comment" packages/but-sdk/src/generated && pnpm -F @gitbutler/desktop check

AC-2: agent-authored tag is referenced by NO enforcement path (build-gate)
  GIVEN: the agent-authored tag is derived in review_status (LPR-005) as descriptive metadata; the shipped ENFORCEMENT_PATHS (invariant_build_gates.rs:23-31)
  WHEN:  the net-new honesty grep runs (assert_grep_has_no_matches for "agent-authored"/"agent_authored" over ENFORCEMENT_PATHS)
  THEN:  NO enforcement path (merge_gate.rs, review_requirement.rs, commit/gate.rs, authorize.rs, config.rs, config_mutate.rs, governance.rs, forge.rs) references the agent-authored tag symbol — role separation emerges from the functional permission set, not from a label; the existing patterns stay green (additive, never weakened)
  TEST_TIER: build-gate   VERIFICATION_SERVICE: the shipped but-authz invariant_build_gates honesty-grep harness (assert_grep_has_no_matches over ENFORCEMENT_PATHS) extended with the agent-tag pattern
  VERIFY: cargo test -p but-authz agent_tag_not_an_enforcement_key

AC-3: __pr_meta__ is a reserved/rejected thread_id — a caller cannot forge the agent-tag source (R23 negative control)
  GIVEN: lpr_governed_repo: a real governed repo; the opener principal is recorded in the dedicated local_review_meta table (LPR-003); "__pr_meta__" is a RESERVED/REJECTED thread_id (the R23 negative control — a comment-body sentinel cannot forge the agent-tag source); a caller `rev` holding comments:write
  WHEN:  post_comment(refs/heads/feat, body="x", thread_id="__pr_meta__") is attempted, then list_comments + review_status are read
  THEN:  the post_comment with the reserved thread_id is REJECTED (Err / normalized — never written as a real comment thread); list_comments and review_status do NOT surface the __pr_meta__ marker as a normal review thread (it is reserved/internal); a caller therefore cannot forge the agent-tag source via a comment write (the opener lives in local_review_meta, not a __pr_meta__ comment)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api post_comment/list_comments/review_status + real but-db local_review_comments + real gix via but_testsupport
  VERIFY: cargo test -p but-api pr_meta_marker_is_reserved_not_a_real_thread

AC-4: N-API bindings route through the gated but-api fns (R14)
  GIVEN: the six verbs are #[but_api(napi)] fns (they authorize via authorize_branch_action); the Electron lite app reaches them via the but-napi binding
  WHEN:  the regenerated N-API surface is audited (the six verb names are located in the generated napi bindings + traced to the #[but_api(napi)] fns)
  THEN:  every one of the six N-API bindings derives from the gated but-api fn — there is NO parallel ungated N-API route for any verb; the audit note in NAPI-AUDIT.md records the R14 finding (consequential N-API routes go through the audited but-api seam)
  TEST_TIER: api-contract   VERIFICATION_SERVICE: a structural grep over the generated N-API surface mapping the six verb names to the #[but_api(napi)] fns + the written R14 audit note
  VERIFY: grep -rq "request_review\|requestReview" packages/but-sdk/src/generated && test -f .spec/prds/governance/tasks/sprint-07-local-agent-pr/NAPI-AUDIT.md && grep -q "R14" .spec/prds/governance/tasks/sprint-07-local-agent-pr/NAPI-AUDIT.md

AC-5: happy-path `but review *` CLI suite passes
  GIVEN: a sandbox governed repo (env.invoke_bash/env.invoke_git seeded); BUT_AGENT_HANDLE set to an authorized principal
  WHEN:  `but review request`/`assign`/`comment --file --line --thread`/`comments`/`resolve`/`status`/`request-changes` each run happy-path via the snapbox env.but(...).assert() idiom
  THEN:  each verb exits 0 and its stdout matches the snapbox snapshot (with [..]/... wildcards for volatile fields); the suite is happy-path only (no exhaustive denial matrix — those are the but-api ACs)
  TEST_TIER: integration   VERIFICATION_SERVICE: real `but` CLI binary via the snapbox env.but(...) harness over a sandbox git repo (env.invoke_bash/env.invoke_git)
  VERIFY: cargo test -p but lpr_review_cli_happy_path

AC-6 (T-LPR-043/044): drive-layer-integrity proofs (R22) — self-assignment rejected + unauthorized self-resolve cannot suppress a signal
  GIVEN: lpr_governed_repo: a real governed repo; the target branch author principal is `auth`; `rev` holds reviews:write+comments:write; `other` holds comments:write but is neither the thread author nor the assigned reviewer
  WHEN:  (a) assign_reviewer(refs/heads/feat, reviewer=auth) is attempted (a self-assignment: reviewer == the target branch author); (b) `auth` posts a changes_requested-style thread t1, then `other` (non-author/non-assigned/non-reviews:write-as-resolver) attempts resolve_thread(t1, resolved=true)
  THEN:  (a) the self-assignment is REJECTED (Err, structured denial) with NO (refs/heads/feat, auth) assignment row written — the drive layer cannot narrate a self-assigned reviewer as independently reviewed (T-LPR-043, R22); (b) the unauthorized self-resolve is REJECTED and thread t1 stays unresolved — a non-author/non-assigned/non-reviews:write actor cannot forge an all-clear and suppress another party's remediation signal (T-LPR-044, R22). These are CLOSEOUT PROOFS of the LPR-003/004 drive-layer-integrity constraints; R22 narrows CROSS-PRINCIPAL forgery only and stays NAMED (a single principal holding both authorship and reviews:write is not made multi-party).
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api assign_reviewer (distinct-from-author) + resolve_thread (resolver-identity) + real but-db + real gix via but_testsupport
  VERIFY: cargo test -p but-api lpr_drive_layer_integrity_self_assign_and_self_resolve_rejected

AC-7 (T-LPR-029): e2e full-loop capstone — the whole implement→review→merge loop runs LOCALLY with no forge
  GIVEN: lpr_governed_loop_repo: ONE real governed repo (committed .gitbutler/{permissions,gates}.toml — a protected target branch with a review requirement) with keep_reviews_local=true; principals `auth` (target branch author, pull_requests:write) and `rev` (reviews:write+comments:write, distinct from auth); a real but_ctx::Context + DbHandle (the LPR-001 tables migrated) + real gix; the forge_reviews cache captured (empty) before the loop; BUT_AGENT_HANDLE set per-step under #[serial_test::serial]
  WHEN:  the full CLI loop runs IN ORDER over the one repo, all local: `but review request <branch> --reviewer rev` (open + first assignment) → `but review assign <branch> --reviewer rev` (idempotent) → `but review comment <branch> --body "x" --file f.rs --line 12 --thread t1` → `but review resolve <branch> t1` → `but review approve <branch>` (writes an approved local_review_verdicts row @head) → `but merge <branch>` (the governed merge through the unchanged gate)
  THEN:  every step completes locally; `but merge` PROCEEDS (the gate's verdict-at-head is satisfied by the approve@head — the land succeeds); AND NO remote forge PR was opened at any step — create_forge_review / the publish_review open-PR path is NEVER invoked and the forge_reviews cache is UNCHANGED (byte-identical / empty) before and after the loop. The whole review-drive loop is served from `but` without a forge (T-LPR-029) — this is the automated sibling of the T-LPR-029h human gate (LPR-011).
  TEST_TIER: e2e-automated   VERIFICATION_SERVICE: the real `but review *` CLI + the real governed `but merge` (enforce_merge_gate) over a real but-db + real gix governed repo via but_testsupport — driven end-to-end, no forge, no mocks
  VERIFY: cargo test -p but lpr_full_local_loop_request_to_merge_no_forge

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): pnpm build:sdk regenerates packages/but-sdk/src/generated with all six LPR verbs and the desktop TS type-check passes
    VERIFY: pnpm build:sdk && pnpm format && grep -rq "reviewStatus\|review_status" packages/but-sdk/src/generated && pnpm -F @gitbutler/desktop check
- TC-2 (-> AC-1): a hand-edit to a generated file is overwritten by a re-run of pnpm build:sdk (the regen is the source of truth)
    VERIFY: pnpm build:sdk && git diff --quiet packages/but-sdk/src/generated
- TC-3 (-> AC-2): the agent-tag honesty grep finds "agent-authored"/"agent_authored" in NO ENFORCEMENT_PATHS file
    VERIFY: cargo test -p but-authz agent_tag_not_an_enforcement_key
- TC-4 (-> AC-2): the existing invariant_build_gates patterns stay green (additive — no existing pattern weakened)
    VERIFY: cargo test -p but-authz invariant_build_gates
- TC-5 (-> AC-3): post_comment with thread_id=="__pr_meta__" is rejected; list_comments/review_status never surface the __pr_meta__ marker as a real thread
    VERIFY: cargo test -p but-api pr_meta_marker_is_reserved_not_a_real_thread
- TC-6 (-> AC-4): the regenerated N-API surface contains the six verbs mapped to the #[but_api(napi)] fns; the R14 audit note is recorded
    VERIFY: grep -rq "request_review\|requestReview" packages/but-sdk/src/generated && grep -q "R14" .spec/prds/governance/tasks/sprint-07-local-agent-pr/NAPI-AUDIT.md
- TC-7 (-> AC-5): the happy-path `but review *` CLI snapbox suite (request/assign/comment/comments/resolve/status/request-changes) passes
    VERIFY: cargo test -p but lpr_review_cli_happy_path
- TC-8 (-> AC-6): drive-layer integrity (R22): a self-assignment (assign_reviewer reviewer==author) is REJECTED (no row written) — T-LPR-043; AND an unauthorized self-resolve (resolve_thread by a non-author/non-assigned/non-reviews:write principal) is REJECTED and the thread stays unresolved (no forged all-clear) — T-LPR-044
    VERIFY: cargo test -p but-api lpr_drive_layer_integrity_self_assign_and_self_resolve_rejected
- TC-9 (-> AC-7): the e2e full-loop capstone (T-LPR-029) runs entirely local — request→assign→comment→resolve→approve→`but merge` over one governed repo with keep_reviews_local=true: the merge PROCEEDS (verdict-at-head satisfied) AND no remote forge PR was opened (create_forge_review never invoked; forge_reviews cache unchanged)
    VERIFY: cargo test -p but lpr_full_local_loop_request_to_merge_no_forge

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides:
  - the regenerated @gitbutler/but-sdk surface for the six LPR verbs (request_review/assign_reviewer/post_comment/list_comments/resolve_thread/review_status) + the desktop TS type-check pass
  - the R14 N-API audit (the six verbs route through the gated but-api seam, no parallel ungated path)
  - the agent-tag-not-an-enforcement-key honesty grep (T-LPR-022) + the __pr_meta__-reserved-constant proof (anti-fakeability)
  - the happy-path `but review *` CLI snapbox suite
  - the R22 drive-layer-integrity closeout proofs (self-assignment rejected T-LPR-043; unauthorized self-resolve rejected T-LPR-044)
  - the NAPI-AUDIT.md R14 audit note
consumes:
  - the six #[but_api(napi)] verbs from LPR-003/004/005/009 (the SDK/N-API surface this regenerates + audits — CONSUMED, not changed)
  - but_authz invariant_build_gates assert_grep_has_no_matches + ENFORCEMENT_PATHS (the honesty-grep discipline this extends additively)
  - the LPR-004 reserved __pr_meta__ thread_id + post_comment (the reserved-thread rejection this proves; the opener itself lives in the dedicated local_review_meta table, LPR-003)
  - LPR-003 assign_reviewer (distinct-from-author) + LPR-004 resolve_thread (resolver-identity) — the R22 constraints these closeout proofs verify
boundary_contracts:
  - CAP-AUTHZ-01: the six verbs ARE gated but-api fns; their N-API/SDK surface inherits the audited seam (R14). The agent tag is descriptive metadata referenced by NO enforcement path (the honesty grep is the build-time proof). __pr_meta__ is a reserved/rejected thread_id a caller cannot forge (the opener lives in local_review_meta; __pr_meta__ is the R23 negative control). The R22 drive-layer-integrity proofs verify a self-assignment is rejected (no row) and an unauthorized self-resolve is rejected (thread stays unresolved) — cross-principal forgery only, stays NAMED. R18/R19/R20 stay NAMED, never closed.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - packages/but-sdk/src/generated/** (REGENERATE ONLY via `pnpm build:sdk && pnpm format` — NEVER hand-edit)
  - crates/but-authz/tests/invariant_build_gates.rs (MODIFY — ADD the agent-tag-not-an-enforcement-key pattern; additive only) OR crates/but-authz/tests/lpr_honesty_gates.rs (NEW — a sibling grep test reusing the same helpers)
  - crates/but-api/tests/pr_meta_reserved.rs (NEW — the __pr_meta__ reserved-constant proof)
  - crates/but/tests/ (NEW — the happy-path `but review *` CLI snapbox suite + the e2e full-loop capstone lpr_full_local_loop_request_to_merge_no_forge (T-LPR-029): request→assign→comment→resolve→approve→`but merge`, all local, no forge; SNAPSHOTS=overwrite to update)
  - crates/but-api/tests/lpr_drive_layer_integrity.rs (NEW — the R22 closeout proofs: self-assignment rejected T-LPR-043; unauthorized self-resolve rejected T-LPR-044; via real but-api + but-db + gix)
  - .spec/prds/governance/tasks/sprint-07-local-agent-pr/NAPI-AUDIT.md (NEW — the R14 N-API audit note: the six verbs are #[but_api(napi)] fns routing through the gated but-api seam; no parallel ungated N-API route)
writeProhibited:
  - crates/but-api/src/**, crates/but-db/src/**, crates/but-rules/src/**, crates/but-authz/src/** — CONSUME-only; this is the audit/closeout task; a missing verb / un-rejected __pr_meta__ / unenforced drive-integrity constraint is a FLAG against LPR-003..009, not a fix here
  - the existing invariant_build_gates patterns (ROLE_BRANCH_PATTERN / HUMAN_OR_LABEL_BRANCH_PATTERN / AUTHORITY_POSITIVE_PATTERN) — ADDITIVE only; never weaken/remove/narrow
  - the six #[but_api(napi)] verb bodies (LPR-003/004/005/009) — this task regenerates + audits their surface, it does not change them; the R22 proofs verify LPR-003 assign_reviewer + LPR-004 resolve_thread, they do not change them
  - the reconciler-usage-model + the `but-*` skill-contract doc + the keep_reviews_local skill contract — those are a SEPARATE task (LPR-011), NOT this task's responsibility
  - any gitbutler-* crate (no new gitbutler-* usage)
  - Any file not in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-authz/tests/invariant_build_gates.rs [9-60, 111-150] — [PRIMARY PATTERN] the honesty-grep discipline: the ROLE_BRANCH_PATTERN/HUMAN_OR_LABEL_BRANCH_PATTERN/AUTHORITY_POSITIVE_PATTERN constants, the ENFORCEMENT_PATHS slice (AUTHZ_AUTHORIZE/AUTHZ_CONFIG/COMMIT_GATE/MERGE_GATE/CONFIG_MUTATE/GOVERNANCE/FORGE_GUARD, :23-31), and the assert_grep_has_no_matches/assert_paths_exist_and_non_empty helpers (:111/:126). ADD the agent-tag pattern as a net-new assert_grep_has_no_matches over ENFORCEMENT_PATHS — never weaken the existing ones.
2. RULES.md ("SDK generation flow") — `pnpm build:sdk && pnpm format` regenerates packages/but-sdk/src/generated after changing Rust APIs exposed via @gitbutler/but-sdk; generated files are NEVER hand-edited.
3. crates/but/AGENTS.md — the CLI snapbox idiom: env.but(...).assert() with .stdout_eq/.stderr_eq + [..]/... wildcards, SNAPSHOTS=overwrite to update, env.invoke_bash/env.invoke_git (NOT std::process::Command::new("git")), happy-path only (CLI tests are expensive).
4. .spec/prds/governance/10-technical-requirements/07-technical-risks.md (R14) — any consequential N-API route must go through the but-api gated wrapper; the six verbs ARE but-api fns so they inherit the audited seam — the R14 audit confirms no parallel ungated N-API path.
5. .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md §B (the six verbs + the SDK/N-API regen + R14), § delta-replan §2 LPR-010 row (the R22 drive-layer-integrity proofs — self-assignment rejected T-LPR-043; unauthorized self-resolve rejected T-LPR-044 — and the tag-sourced-from-local_review_meta / __pr_meta__-reserved honesty greps).
6. .spec/prds/governance/tasks/sprint-07-local-agent-pr/LPR-003-... + LPR-004-... + LPR-005-... — the six verbs, the reserved __pr_meta__ thread_id (LPR-004's reserved/rejected thread; the opener itself is in the dedicated local_review_meta table, LPR-003), the post_comment reserved-thread rejection (LPR-004), assign_reviewer distinct-from-author (LPR-003) + resolve_thread resolver-identity (LPR-004) — the R22 constraints these closeout proofs verify — and the agent-tag derivation (LPR-005) this task regenerates/audits/proves.
7. .spec/prds/governance/tasks/sprint-06b-.../MGMT-UI-012-build-gate-tests.md — [SIBLING SHAPE] a build-gate/honesty-test task in this PRD (the no-direct-config-write / no-+page.server.ts / SDK-type-check / human-principal grep suite) — mirror its build-gate task posture.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- pnpm build:sdk && pnpm format && grep -rq "reviewStatus\|review_status" packages/but-sdk/src/generated && pnpm -F @gitbutler/desktop check   -> Exit 0; SDK carries the six verbs; desktop TS type-checks
- pnpm build:sdk && git diff --quiet packages/but-sdk/src/generated   -> Exit 0 after a clean regen; a hand-edit would show a diff overwritten by the re-run
- cargo test -p but-authz agent_tag_not_an_enforcement_key   -> Exit 0; "agent-authored"/"agent_authored" in NO ENFORCEMENT_PATHS file
- cargo test -p but-authz invariant_build_gates   -> Exit 0; existing patterns green (additive, never weakened)
- cargo test -p but-api pr_meta_marker_is_reserved_not_a_real_thread   -> Exit 0; post_comment rejects __pr_meta__; not surfaced as a real thread
- cargo test -p but lpr_review_cli_happy_path   -> Exit 0; happy-path CLI suite green
- cargo test -p but-api lpr_drive_layer_integrity_self_assign_and_self_resolve_rejected   -> Exit 0; self-assignment rejected (no row) T-LPR-043; unauthorized self-resolve rejected, thread stays unresolved T-LPR-044 (R22)
- test -f .spec/prds/governance/tasks/sprint-07-local-agent-pr/NAPI-AUDIT.md && grep -q "R14" .spec/prds/governance/tasks/sprint-07-local-agent-pr/NAPI-AUDIT.md   -> Exit 0; the R14 N-API audit note recorded (the six verbs route through the gated but-api seam)
- cargo check -p but-authz -p but-api --all-targets   -> Exit 0
- cargo clippy -p but-authz -p but-api --all-targets   -> Exit 0
- cargo fmt --check   -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references:
  - crates/but-authz/tests/invariant_build_gates.rs:111/:126 (assert_grep_has_no_matches / assert_paths_exist_and_non_empty), :23-31 (ENFORCEMENT_PATHS) — the honesty-grep harness to extend additively
  - RULES.md "SDK generation flow" (pnpm build:sdk && pnpm format; never hand-edit generated)
  - crates/but/AGENTS.md (the snapbox CLI idiom, happy-path-only, SNAPSHOTS=overwrite)
  - .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md §B + the delta-replan §2 LPR-010 row (the R22 drive-layer-integrity proofs + the honesty greps)
notes:
  - The agent-tag grep is additive: define a net-new pattern constant (e.g. AGENT_TAG_LABEL_PATTERN = r#"agent[-_]authored"#) and call assert_grep_has_no_matches(&workspace_root, "agent-authored tag must not appear in any enforcement path", AGENT_TAG_LABEL_PATTERN, ENFORCEMENT_PATHS). Do NOT touch the existing ROLE/HUMAN_OR_LABEL/AUTHORITY patterns.
  - The __pr_meta__ proof uses the reserved "__pr_meta__" thread_id constant (LPR-004's reserved/rejected thread). The opener itself lives in the dedicated local_review_meta table (LPR-003), NOT a __pr_meta__ comment — so __pr_meta__ is purely the R23 negative control (a comment-body sentinel cannot forge the opener/tag). The test drives post_comment(thread_id="__pr_meta__") and asserts rejection; reads list_comments/review_status and asserts the marker is not surfaced as a normal thread. If the rejection is not yet implemented in post_comment, FLAG it as an LPR-004 gap (the rejection is LPR-004's write; this is the proof).
  - The R14 audit is a structural grep + a written note in NAPI-AUDIT.md (the six verbs ARE but-api fns; the audit confirms the generated N-API surface derives from them, no parallel ungated route).
  - The R22 drive-layer-integrity proofs (crates/but-api/tests/lpr_drive_layer_integrity.rs): seed lpr_governed_repo (auth=target branch author; rev: reviews:write+comments:write; other: comments:write only). (a) drive assign_reviewer(reviewer=auth) — a self-assignment — and assert it is REJECTED (Err / structured denial) with NO (refs/heads/feat, auth) assignment row written (T-LPR-043, verifying LPR-003's distinct-from-author constraint); (b) have auth post a changes_requested-style thread t1, then have other attempt resolve_thread(t1, resolved=true) and assert it is REJECTED and thread t1 stays unresolved (T-LPR-044, verifying LPR-004's resolver-identity constraint). R22 narrows CROSS-PRINCIPAL forgery only — a single principal holding both authorship and reviews:write is NOT made multi-party. Do NOT assert a forgeable direct DB write to local_review_verdicts/local_review_assignees is blocked (that would be a false R6/R18 guarantee); these proofs verify the drive-layer verbs, not a tamper-evident storage boundary. If a constraint is not yet enforced, FLAG it against LPR-003/LPR-004.
  - CLI happy-path: drive request -> assign -> comment --file --line --thread -> comments -> resolve -> status -> request-changes in order over one sandbox repo; assert each exits 0 with the snapbox snapshot. The full forged-vs-empty safe-seam proof is LPR-009's job, not this CLI suite.
pattern: an audit/closeout lane — regenerate the SDK (never hand-edit), audit the N-API seam (R14, note in NAPI-AUDIT.md), add two additive honesty greps (tag-not-enforcement-key + __pr_meta__-reserved), add the happy-path CLI snapbox suite, and prove the R22 drive-layer-integrity constraints (self-assignment rejected; unauthorized self-resolve rejected) as closeout proofs against real but-api
pattern_source: crates/but-authz/tests/invariant_build_gates.rs (the honesty-grep harness); RULES.md SDK generation flow; crates/but/AGENTS.md (snapbox CLI); .spec/prds/governance/tasks/sprint-06b-.../MGMT-UI-012-build-gate-tests.md (the sibling build-gate task)
anti_pattern: hand-editing packages/but-sdk/src/generated (AC-1's regen-overwrite catches it); weakening an existing invariant_build_gates pattern (must be additive); the agent tag leaking into a gate path (AC-2 fails); a caller successfully writing thread_id="__pr_meta__" (AC-3 catches the forge); an exhaustive denial CLI matrix (CLI tests are happy-path only); a self-assignment (assign_reviewer reviewer==author) succeeding with a row written, or an unauthorized self-resolve flipping a thread to resolved (AC-6's R22 proofs catch both forgeries); asserting a forgeable direct DB write is blocked as if it proved R6/R18 (a false guarantee — R18/R19/R20 stay accepted residuals, never presented as closed); fixing a missing-verb/un-rejected-marker/unenforced-constraint here instead of flagging it against the owning LPR task

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agent: implementer=rust-reviewer | reviewer=rust-reviewer
rationale: This is the cross-cutting audit/closeout lane — SDK regen, the R14 N-API audit (note in NAPI-AUDIT.md), two additive honesty greps (tag-not-an-enforcement-key + __pr_meta__-reserved), the happy-path CLI snapbox suite, and the R22 drive-layer-integrity proofs (self-assignment rejected T-LPR-043; unauthorized self-resolve rejected T-LPR-044). It is reviewer-owned because its core value is adversarial verification (does the SDK actually expose the verbs? does the tag stay out of the gate? can __pr_meta__ be forged? can a self-assigned reviewer be narrated as independently reviewed, or a cross-principal actor forge an all-clear?) rather than feature implementation. A missing verb / un-rejected marker / unenforced drive-integrity constraint is FLAGGED against the owning LPR task (LPR-003/LPR-004), not patched here — the discipline a rust-reviewer applies. rust-reviewer authors the greps + the audit + the R22 drive-integrity proofs; a second rust-reviewer pass confirms no existing honesty pattern was weakened and no production logic was changed. The reconciler-usage-model + the `but-*` skill-contract doc are a SEPARATE task (LPR-011), not this one.
coding_standards: crates/AGENTS.md (keep the honesty grep in but-authz's invariant gates; CLI tests in but happy-path only; the R22 drive-integrity proofs in but-api against real but-db + gix via but_testsupport); RULES.md (SDK generation flow — pnpm build:sdk && pnpm format, never hand-edit generated; CLI tests are expensive — happy-path only); crates/but/AGENTS.md (snapbox env.but(...).assert() + SNAPSHOTS=overwrite + env.invoke_bash/git); the named-leaks honesty doctrine (R18/R19/R20 never presented as closed; R22 narrows cross-principal forgery only and stays NAMED); brain/docs/rust/ (testing.md build-gate / honesty-grep tests + the but-api hand-assertion idiom; module-system.md the #[but_api(napi)] -> N-API -> SDK surface)

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: LPR-003 (request_review/assign_reviewer distinct-from-author + the local_review_meta opener record — the R22 self-assignment proof verifies this), LPR-004 (post_comment/list_comments/resolve_thread resolver-identity + the reserved __pr_meta__ thread rejection — the R22 self-resolve proof + the __pr_meta__ reserved-thread proof verify this), LPR-005 (review_status + the agent-tag derivation), LPR-006 (keep_reviews_local — for SDK regen of the new project-settings surface), LPR-008 (the reconciler read-API surface) — all the #[but_api(napi)] verbs whose SDK this regenerates and whose N-API seam it audits
Blocks:     None (the sprint closeout lane)
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "LPR-010",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "lpr_sdk_and_greps": {
      "description": "Two real surfaces, no mocks: (1) the real pnpm build:sdk generation pipeline over the six #[but_api(napi)] LPR verbs in crates/but-api/src/legacy/forge.rs (LPR-003/004/005/009), with the real tsc type-check (pnpm -F @gitbutler/desktop check) against the regenerated packages/but-sdk/src/generated; (2) the real but-authz invariant_build_gates honesty-grep harness (assert_grep_has_no_matches over the shipped ENFORCEMENT_PATHS) extended with the agent-tag pattern. For the __pr_meta__ and CLI ACs: a real governed repo via but_testsupport::writable_scenario / the but CLI snapbox harness (env.invoke_bash/env.invoke_git), BUT_AGENT_HANDLE set under #[serial_test::serial]. Drive through the real verbs/CLI; never hand-edit generated files; never inject rows directly.",
      "seed_method": "public_api",
      "records": [
        "run `pnpm build:sdk && pnpm format` over the six real #[but_api(napi)] verbs; type-check the regenerated SDK via `pnpm -F @gitbutler/desktop check`;",
        "extend invariant_build_gates with assert_grep_has_no_matches(\"agent[-_]authored\", ENFORCEMENT_PATHS);",
        "but_testsupport::writable_scenario + invoke_bash committing .gitbutler/permissions.toml; BUT_AGENT_HANDLE=rev under #[serial_test::serial]; drive post_comment(thread_id=__pr_meta__) + the `but review *` CLI happy-path via the snapbox env.but(...).assert() idiom."
      ]
    },
    "lpr_governed_repo": {
      "description": "A real governed repo via but_testsupport::writable_scenario + invoke_bash committing .gitbutler/permissions.toml. Principals: target branch author `auth`; `rev` holds reviews:write+comments:write; `other` holds comments:write but is neither the thread author nor the assigned reviewer. A real but_ctx::Context + DbHandle (the LPR-001 tables migrated). BUT_AGENT_HANDLE set per-case under #[serial_test::serial]. Seed via the real verbs (assign_reviewer/post_comment/resolve_thread); for the __pr_meta__ proof, attempt post_comment(thread_id=__pr_meta__). No mocks.",
      "seed_method": "public_api",
      "records": [
        "but_testsupport::writable_scenario + invoke_bash committing .gitbutler/permissions.toml (auth=author; rev: reviews:write+comments:write; other: comments:write) to refs/heads/main;",
        "temp_env BUT_AGENT_HANDLE per-case under #[serial_test::serial];",
        "drive assign_reviewer(reviewer=auth) for the self-assignment; auth posts thread t1, other attempts resolve_thread(t1); attempt post_comment(thread_id=__pr_meta__) for the reserved-thread proof."
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN the six #[but_api(napi)] LPR verbs exist; a clean packages/but-sdk tree WHEN `pnpm build:sdk && pnpm format` runs then a hand-edit is made and build:sdk re-runs THEN the generated SDK contains all six verbs, `pnpm -F @gitbutler/desktop check` exits 0, and the hand-edit is OVERWRITTEN by the regen",
      "verify": "pnpm build:sdk && pnpm format && grep -rq \"reviewStatus\\|review_status\" packages/but-sdk/src/generated && grep -rq \"postComment\\|post_comment\" packages/but-sdk/src/generated && pnpm -F @gitbutler/desktop check",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real pnpm build:sdk generation pipeline + real tsc type-check against the regenerated SDK",
        "negative_control": {
          "would_fail_if": [
            "a verb was not exposed as #[but_api(napi)] — the grep finds it missing from the generated SDK",
            "the SDK was hand-edited rather than regenerated — git diff shows a manual edit that a re-run of build:sdk overwrites",
            "a stub/static generator emitted unchanged generated files — the six verb names are absent and the regen disconnects from the new fns",
            "the regenerated SDK did not type-check — pnpm -F @gitbutler/desktop check exits non-zero"
          ]
        },
        "evidence": { "artifact_type": "file_artifact", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_sdk_and_greps",
            "action": { "actor": "ci", "steps": [ "pnpm build:sdk && pnpm format", "grep the regenerated packages/but-sdk/src/generated for all six verb names", "pnpm -F @gitbutler/desktop check", "hand-edit a generated file then re-run pnpm build:sdk and assert the edit is overwritten" ] },
            "end_state": {
              "must_observe": [
                "packages/but-sdk/src/generated contains request_review/assign_reviewer/post_comment/list_comments/resolve_thread/review_status (or their generated camelCase names)",
                "pnpm -F @gitbutler/desktop check exits 0",
                "a hand-edit to a generated file is overwritten by the next build:sdk (git diff clean after regen)"
              ],
              "must_not_observe": [
                "any of the six verbs missing from the generated SDK (the fn was not exposed)",
                "a surviving hand-edit in a generated file after a regen",
                "a non-zero desktop type-check"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the agent-authored tag is derived in review_status as descriptive metadata; the shipped ENFORCEMENT_PATHS WHEN the net-new honesty grep runs THEN no enforcement path references the agent-authored tag symbol and the existing patterns stay green (additive)",
      "verify": "cargo test -p but-authz agent_tag_not_an_enforcement_key",
      "scenario": {
        "tier": "visible",
        "test_tier": "build-gate",
        "verification_service": "the shipped but-authz invariant_build_gates honesty-grep harness (assert_grep_has_no_matches over ENFORCEMENT_PATHS) extended with the agent-tag pattern",
        "negative_control": {
          "would_fail_if": [
            "merge_gate.rs / review_requirement.rs / commit/gate.rs / but-authz referenced 'agent-authored'/'agent_authored' — the tag became an enforcement key (role separation leaking into a label)",
            "the grep ran over no paths (assert_paths_exist_and_non_empty would fail) — a disconnected no-op grep",
            "an existing invariant_build_gates pattern was weakened to make room — the existing test would change/regress"
          ]
        },
        "evidence": { "artifact_type": "test_output", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_sdk_and_greps",
            "action": { "actor": "ci", "steps": [ "add AGENT_TAG_LABEL_PATTERN = agent[-_]authored", "assert_grep_has_no_matches over the shipped ENFORCEMENT_PATHS", "run invariant_build_gates to confirm existing patterns unchanged" ] },
            "end_state": {
              "must_observe": [
                "the agent-tag pattern matches in NONE of the ENFORCEMENT_PATHS files",
                "the existing ROLE/HUMAN_OR_LABEL/AUTHORITY patterns still pass (additive)"
              ],
              "must_not_observe": [
                "'agent-authored'/'agent_authored' in any enforcement path",
                "an existing pattern weakened/removed/narrowed",
                "the grep running over an empty path set (a no-op)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN lpr_governed_repo; the opener principal is recorded in the dedicated local_review_meta table (LPR-003) and __pr_meta__ is a RESERVED/REJECTED thread_id (the R23 negative control — a comment-body sentinel cannot forge the agent-tag source); caller rev holds comments:write WHEN post_comment(thread_id=__pr_meta__) is attempted then list_comments + review_status are read THEN the reserved-thread post is REJECTED (never written as a real thread) and neither read surfaces the __pr_meta__ marker as a normal review thread",
      "verify": "cargo test -p but-api pr_meta_marker_is_reserved_not_a_real_thread",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api post_comment/list_comments/review_status + real but-db local_review_comments + real gix via but_testsupport",
        "negative_control": {
          "would_fail_if": [
            "post_comment accepted thread_id=='__pr_meta__' and wrote a real comment — a caller could forge the agent-tag source",
            "list_comments / review_status surfaced the __pr_meta__ reserved marker thread as a normal review thread — the internal marker leaked",
            "a stub returned Ok for the reserved write without rejecting it"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_governed_repo",
            "action": { "actor": "agent", "steps": [ "BUT_AGENT_HANDLE=rev", "attempt post_comment(refs/heads/feat, body=x, thread_id=__pr_meta__)", "read list_comments(refs/heads/feat) + review_status(refs/heads/feat)" ] },
            "end_state": {
              "must_observe": [
                "the post_comment(thread_id=__pr_meta__) is rejected (Err / normalized — no real comment written under __pr_meta__ by the caller)",
                "list_comments does not surface the __pr_meta__ marker as a normal review thread",
                "review_status does not expose the __pr_meta__ marker as a user-facing comment thread"
              ],
              "must_not_observe": [
                "a caller-written comment persisted under thread_id=__pr_meta__",
                "the __pr_meta__ reserved marker thread appearing in the user-facing thread list"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the six verbs are #[but_api(napi)] fns (authorize via authorize_branch_action); reached via but-napi WHEN the regenerated N-API surface is audited THEN every binding derives from the gated but-api fn (no parallel ungated route) and the R14 audit note is recorded in NAPI-AUDIT.md",
      "verify": "grep -rq \"request_review\\|requestReview\" packages/but-sdk/src/generated && test -f .spec/prds/governance/tasks/sprint-07-local-agent-pr/NAPI-AUDIT.md && grep -q \"R14\" .spec/prds/governance/tasks/sprint-07-local-agent-pr/NAPI-AUDIT.md",
      "scenario": {
        "tier": "holdout",
        "test_tier": "api-contract",
        "verification_service": "a structural grep over the generated N-API surface mapping the six verb names to the #[but_api(napi)] fns + the written R14 audit note",
        "negative_control": {
          "would_fail_if": [
            "a verb was exposed via N-API without the but-api authorize seam (a parallel ungated route) — the audit note would have to flag it",
            "the R14 audit note was absent from NAPI-AUDIT.md — the audit was not performed/recorded",
            "the generated N-API surface lacked a verb (it was not exposed)"
          ]
        },
        "evidence": { "artifact_type": "file_artifact", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_sdk_and_greps",
            "action": { "actor": "ci", "steps": [ "locate the six verb names in the generated N-API surface", "trace each to its #[but_api(napi)] fn (which authorizes via authorize_branch_action)", "record the R14 finding in NAPI-AUDIT.md" ] },
            "end_state": {
              "must_observe": [
                "all six verbs present in the generated N-API surface, each derived from the gated but-api fn",
                "NAPI-AUDIT.md records the R14 audit (no parallel ungated N-API route)"
              ],
              "must_not_observe": [
                "a verb reachable via N-API outside the gated but-api fn",
                "NAPI-AUDIT.md missing the R14 audit note"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN a sandbox governed repo; BUT_AGENT_HANDLE set to an authorized principal WHEN the `but review` verbs run happy-path via the snapbox idiom THEN each exits 0 and matches its snapbox snapshot (happy-path only)",
      "verify": "cargo test -p but lpr_review_cli_happy_path",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real `but` CLI binary via the snapbox env.but(...) harness over a sandbox git repo (env.invoke_bash/env.invoke_git)",
        "negative_control": {
          "would_fail_if": [
            "a verb errored on the happy path (the CLI wiring is broken) — exit non-zero / a stderr snapshot mismatch",
            "request-changes still hit the stub (task_contract_invalid) — the happy-path request-changes would fail (cross-checks LPR-003)",
            "the test seeded git via std::process::Command instead of env.invoke_git (a non-sandbox shell-out)"
          ]
        },
        "evidence": { "artifact_type": "cli_output", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_governed_repo",
            "action": { "actor": "agent", "steps": [ "seed a sandbox governed repo via env.invoke_bash/env.invoke_git", "BUT_AGENT_HANDLE=authorized", "run request -> assign -> comment --file --line --thread -> comments -> resolve -> status -> request-changes via env.but(...).assert()" ] },
            "end_state": {
              "must_observe": [ "each `but review *` verb exits 0", "each stdout matches the snapbox snapshot (with [..]/... wildcards)" ],
              "must_not_observe": [ "any happy-path verb exiting non-zero", "request-changes hitting the stub error", "a std::process::Command git shell-out" ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-6",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN lpr_governed_repo (auth=target branch author; rev: reviews:write+comments:write; other: comments:write only, neither thread author nor assigned reviewer) WHEN (a) assign_reviewer(refs/heads/feat, reviewer=auth) is attempted (a self-assignment: reviewer==author), and (b) auth posts a changes_requested-style thread t1 then other attempts resolve_thread(t1, resolved=true) THEN (a) the self-assignment is REJECTED (Err, structured denial) with NO (refs/heads/feat, auth) assignment row written (T-LPR-043, R22 — the drive layer cannot narrate a self-assigned reviewer as independently reviewed) AND (b) the unauthorized self-resolve is REJECTED and thread t1 stays unresolved (T-LPR-044, R22 — a non-author/non-assigned/non-reviews:write actor cannot forge an all-clear and suppress another party's remediation signal). R22 narrows CROSS-PRINCIPAL forgery only and stays NAMED.",
      "verify": "cargo test -p but-api lpr_drive_layer_integrity_self_assign_and_self_resolve_rejected",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api assign_reviewer + resolve_thread + real but-db + real gix",
        "negative_control": {
          "would_fail_if": [
            "the self-assignment (reviewer==author) SUCCEEDED with a row written — the drive layer would narrate a self-assigned reviewer as independently reviewed (R22 missing)",
            "the unauthorized self-resolve SUCCEEDED and flipped the thread to resolved — a non-author/non-assigned/non-reviews:write actor forged an all-clear (R22 missing)",
            "the test asserted a forgeable direct DB write to local_review_verdicts is blocked (a false R6/R18 guarantee — forbidden)"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_governed_repo",
            "action": { "actor": "agent", "steps": [ "BUT_AGENT_HANDLE=auth; attempt assign_reviewer(refs/heads/feat, reviewer=auth) (a self-assignment) and capture the result", "BUT_AGENT_HANDLE=auth; post a changes_requested-style thread t1 via post_comment", "BUT_AGENT_HANDLE=other; attempt resolve_thread(t1, resolved=true) and capture the result", "read review_status(refs/heads/feat) + list_comments(refs/heads/feat) to confirm no assignment row and t1 still unresolved" ] },
            "end_state": {
              "must_observe": [
                "assign_reviewer(reviewer=auth) is rejected (Err / structured denial) with NO (refs/heads/feat, auth) assignment row written (T-LPR-043)",
                "resolve_thread(t1) by other is rejected and thread t1 stays unresolved (T-LPR-044)"
              ],
              "must_not_observe": [
                "a (refs/heads/feat, auth) self-assignment row persisted",
                "thread t1 flipped to resolved by the unauthorized other principal",
                "an assertion that a forgeable direct DB write to local_review_verdicts/local_review_assignees is blocked (a false R6/R18 guarantee)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-7",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN lpr_governed_loop_repo: ONE real governed repo (committed .gitbutler/{permissions,gates}.toml, protected target with a review requirement) with keep_reviews_local=true; auth (target author, pull_requests:write) + rev (reviews:write+comments:write, distinct); the forge_reviews cache empty WHEN the full CLI loop runs in order — `but review request <branch> --reviewer rev` -> `but review assign` -> `but review comment --file --line --thread` -> `but review resolve` -> `but review approve` (approved verdict@head) -> `but merge <branch>` (governed) THEN every step completes locally, `but merge` PROCEEDS (verdict-at-head satisfied), and NO remote forge PR was opened (create_forge_review never invoked; forge_reviews cache unchanged) — the whole review-drive loop is served from but without a forge (T-LPR-029, the automated sibling of the T-LPR-029h human gate)",
      "verify": "cargo test -p but lpr_full_local_loop_request_to_merge_no_forge",
      "scenario": {
        "tier": "visible",
        "test_tier": "e2e-automated",
        "verification_service": "the real `but review *` CLI + the real governed `but merge` (enforce_merge_gate) over a real but-db + real gix governed repo via but_testsupport — driven end-to-end, no forge, no mocks",
        "negative_control": {
          "would_fail_if": [
            "`but merge` was BLOCKED despite an approved verdict@head — the governed land did not proceed for a satisfied gate (the loop is broken)",
            "any step opened a remote forge PR — create_forge_review / the publish_review open-PR path was invoked, or the forge_reviews cache changed (the loop did not stay local under keep_reviews_local=true)",
            "a step hit a stub (request-changes/comment task_contract_invalid) and the loop could not reach the merge"
          ]
        },
        "evidence": { "artifact_type": "cli_output", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_governed_repo",
            "action": { "actor": "agent", "steps": [ "seed ONE governed repo with keep_reviews_local=true (auth=author/pull_requests:write; rev=reviews:write+comments:write); capture forge_reviews (empty)", "BUT_AGENT_HANDLE=auth; `but review request <branch> --reviewer rev`", "`but review assign <branch> --reviewer rev`", "BUT_AGENT_HANDLE=rev; `but review comment <branch> --body x --file f.rs --line 12 --thread t1`", "`but review resolve <branch> t1`", "`but review approve <branch>` (approved verdict@head)", "`but merge <branch>` (governed)", "re-read forge_reviews to confirm it is unchanged" ] },
            "end_state": {
              "must_observe": [
                "every `but review *` step completes locally (exit 0)",
                "`but merge` PROCEEDS — the governed land succeeds because the verdict-at-head is satisfied",
                "the forge_reviews cache is UNCHANGED before and after the loop (no remote forge PR opened)"
              ],
              "must_not_observe": [
                "`but merge` blocked despite an approved verdict@head",
                "a remote forge PR opened at any step (create_forge_review invoked / forge_reviews cache changed)",
                "a stub error (task_contract_invalid) preventing the loop from reaching the merge"
              ]
            }
          }
        ]
      }
    },
    { "id": "TC-1", "type": "test_criterion", "description": "pnpm build:sdk regenerates the SDK with all six LPR verbs and the desktop TS type-check passes", "verify": "pnpm build:sdk && pnpm format && grep -rq \"reviewStatus\\|review_status\" packages/but-sdk/src/generated && pnpm -F @gitbutler/desktop check", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "a hand-edit to a generated file is overwritten by a re-run of pnpm build:sdk", "verify": "pnpm build:sdk && git diff --quiet packages/but-sdk/src/generated", "maps_to_ac": "AC-1" },
    { "id": "TC-3", "type": "test_criterion", "description": "the agent-tag honesty grep finds agent-authored/agent_authored in NO ENFORCEMENT_PATHS file", "verify": "cargo test -p but-authz agent_tag_not_an_enforcement_key", "maps_to_ac": "AC-2" },
    { "id": "TC-4", "type": "test_criterion", "description": "the existing invariant_build_gates patterns stay green (additive — none weakened)", "verify": "cargo test -p but-authz invariant_build_gates", "maps_to_ac": "AC-2" },
    { "id": "TC-5", "type": "test_criterion", "description": "post_comment with thread_id=__pr_meta__ is rejected; list_comments/review_status never surface the marker as a real thread", "verify": "cargo test -p but-api pr_meta_marker_is_reserved_not_a_real_thread", "maps_to_ac": "AC-3" },
    { "id": "TC-6", "type": "test_criterion", "description": "the regenerated N-API surface contains the six verbs mapped to the #[but_api(napi)] fns; the R14 audit note is recorded", "verify": "grep -rq \"request_review\\|requestReview\" packages/but-sdk/src/generated && grep -q \"R14\" .spec/prds/governance/tasks/sprint-07-local-agent-pr/NAPI-AUDIT.md", "maps_to_ac": "AC-4" },
    { "id": "TC-7", "type": "test_criterion", "description": "the happy-path `but review *` CLI snapbox suite passes", "verify": "cargo test -p but lpr_review_cli_happy_path", "maps_to_ac": "AC-5" },
    { "id": "TC-8", "type": "test_criterion", "description": "drive-layer integrity (R22): a self-assignment (assign_reviewer reviewer==author) is REJECTED (no row written) — T-LPR-043; AND an unauthorized self-resolve (resolve_thread by a non-author/non-assigned/non-reviews:write principal) is REJECTED and the thread stays unresolved (no forged all-clear) — T-LPR-044", "verify": "cargo test -p but-api lpr_drive_layer_integrity_self_assign_and_self_resolve_rejected", "maps_to_ac": "AC-6" },
    { "id": "TC-9", "type": "test_criterion", "description": "the e2e full-loop capstone (T-LPR-029) runs entirely local — request→assign→comment→resolve→approve→`but merge` over one governed repo with keep_reviews_local=true: the merge PROCEEDS (verdict-at-head satisfied) AND no remote forge PR was opened (create_forge_review never invoked; forge_reviews cache unchanged)", "verify": "cargo test -p but lpr_full_local_loop_request_to_merge_no_forge", "maps_to_ac": "AC-7" }
  ]
}
-->
