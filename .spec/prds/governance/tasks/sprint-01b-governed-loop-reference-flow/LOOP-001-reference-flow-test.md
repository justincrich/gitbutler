# LOOP-001: Reference-flow integration test (T-LOOP-006) — the implement→review→merge loop across three principals through the REAL `but` CLI + real git + real but-db, plus the traversable irrigation proof (T-LOOP-013) and the DryRun-no-bypass proof (CAP-AUTHZ-01)

## What this does

A single integration/e2e test — the headline T-LOOP-006 canary the PRD requires green before the deep build — that drives the full implement→review→merge loop through the REAL `but` CLI against a real git repo seeded with three distinct principals: an implementer (`contents:write`+`pull_requests:write`, no `merge`), a reviewer (`reviews:write` via the `code-reviewers` group, no `contents:write`), and a maintainer (`merge` via the `maintainers` group). It proves role separation EMERGES from the functional permission set (no role-name in any enforcement path), the irrigation half (a denied implementer that follows its `remediation_hint` lands through a reviewed merge), DryRun-no-bypass, and the auto-merge denial. All reviews go through the governed `but review` action; the forgeable direct-DB-write path is NOT under test.

## Why

Sprint 01b · PRD UC-LOOP-01, UC-LOOP-02 · capabilities CAP-AUTHZ-01, CAP-CONFIG-01. This is the sprint CAPSTONE and the headline gate (11-e2e-testing-criteria.md: "T-LOOP-006 … must be green before the deep build proceeds"). It composes GATES-001..005 + AUTHZ-001..003; it builds none of them.

## How to verify

PRIMARY **AC-1** — `cargo test -p but governed_loop_reference_flow_full_loop` (e2e; real `but` CLI + real git + real but-db). Full gate set in the spec below.

## Scope

- `crates/but/tests/but/command/governed_loop.rs` (NEW) — the LOOP-001 integration/e2e test: seeds the 3-principal repo, drives the full loop + traversability + DryRun-no-bypass + auto-merge-denial through the real `but` CLI snapbox harness, asserting exit codes, the `{error:{code,message,remediation_hint}}` contract, and real-git ref invariants
- `crates/but/tests/but/command/mod.rs` (MODIFY) — add `mod governed_loop;`
- `crates/but/tests/fixtures/scenario/governed-loop.sh` (NEW, if scripted fixture preferred) — a real git scenario script committing `.gitbutler/permissions.toml` + `gates.toml` at refs/heads/main and branching to feat
- `crates/but/tests/but/utils/governed_loop_seed.rs` (NEW, OPTIONAL tiny helper) — `seed_governed_loop_repo(&repo)` wrapping `invoke_bash` TOML writes IF inline seeding bloats the test; no production logic

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: LOOP-001 - Reference-flow integration test (T-LOOP-006) + traversable proof (T-LOOP-013) + DryRun-no-bypass (CAP-AUTHZ-01)
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Complete
PRIORITY:   P0
EFFORT:     L  (240 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-LOOP-01, UC-LOOP-02
CAPABILITIES: CAP-AUTHZ-01, CAP-CONFIG-01

RUNTIME_COMMANDS:
  test:  cargo test -p but governed_loop
  check: cargo check -p but --all-targets
  lint:  cargo clippy -p but --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
A single integration/e2e test file is green against the REAL `but` CLI binary + real git + the real but-db review store, running the full implement→review→merge loop with three DISTINCT principals: (1) implementer commits to feat (accepted, exit 0, feat advances) + the open-PR is PERMITTED by the gate (no governance Denial; reaches the forge call); (2) implementer merge denied perm.denied naming merge (exit 1, trunk unchanged); (3) reviewer commit denied perm.denied (edits inert) then `but review approve` accepted (exit 0, verdict recorded @head); (4) maintainer merge with zero distinct approvals denied gate.review_required (exit 1, trunk unchanged); (5) maintainer merge after the distinct approval @head is PERMITTED by the gate (no governance Denial; execution reaches the forge merge_review call). Plus: a denied implementer following its remediation_hint traverses the governed path with every gate permitting it (T-LOOP-013); a DryRun implementer merge still perm.denied + persists nothing (CAP-AUTHZ-01); enabling auto-merge as the implementer is denied perm.denied (same gate); and an unset/empty BUT_AGENT_HANDLE is rejected perm.denied (fail-closed, T-AUTHZ-027/028). All reviews via the governed `but review` action; the forgeable direct-DB-write path is NOT under test. GATE-BOUNDARY RE-SCOPE (red-hat + user decision): `merge_review`/`publish_review` are forge-bound (error on a bare local repo) and there is no `but pr merge` CLI verb, so POSITIVE merge/PR-open paths prove the gate DECISION (permit, no Denial) on the real seam + that execution reaches the forge call — the forge-network landing is structural/out-of-local-scope. DENIAL + commit + local-verdict-write paths are fully locally provable.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST drive the entire loop through the REAL `but` CLI binary via the snapbox harness — Sandbox::but(args) (crates/but/tests/but/utils.rs:125) returning a snapbox::cmd::Command configured with .env("BUT_AGENT_HANDLE", <handle>), .assert(), .stdout_eq/.stderr_eq + [..]/... wildcards, exit-code assertions — so the test exercises the SAME but-api action boundary a real orchestrator hits. NEVER call the gate helpers or but_authz::authorize directly to simulate the loop in-process.
- [MUST] MUST use THREE DISTINCT principals whose role separation EMERGES from the functional permission set alone: implementer=[contents:write,pull_requests:write] (NO merge), reviewer=member of group code-reviewers (group grants [reviews:write], NO contents:write), maintainer=member of group maintainers (group grants [merge]). The test MUST NOT depend on any role name in an enforcement path (UC-LOOP-01 AC-5).
- [MUST] MUST seed the fixture repo via the REAL CLI + real git through but_testsupport::writable_scenario(name) + invoke_bash(script,&repo): write .gitbutler/permissions.toml (3 principals/groups) and .gitbutler/gates.toml (main protected + a [[gate]] review requirement min_approvals=1 require_distinct_from_author=true) as REAL committed blobs at refs/heads/main, branch to feat, stage a pending change. NEVER inject principals/config in-memory; NEVER use std::env::temp_dir().join(format!(...)).
- [MUST] MUST submit ALL reviews through the governed `but review approve` action so the test exercises the gated, traversable channel.
- [MUST] MUST assert the structured denial contract {error:{code,message,remediation_hint}} with exit 1 for every denial: impl merge → perm.denied naming merge; reviewer commit → perm.denied; zero-approval maintainer merge → gate.review_required; DryRun impl merge → perm.denied; auto-merge impl → perm.denied.
- [MUST] MUST assert real-git ref invariants: feat advances on the implementer commit; trunk/main is UNCHANGED on every denied merge and on the DryRun; trunk ADVANCES (the change lands) on the satisfied maintainer merge and on the completed remediation path. Read refs via repo.find_reference(name)?.peel_to_id()?; keep the TempDir alive.
- [MUST] MUST prove TRAVERSABILITY (T-LOOP-013): a denied implementer (denied a direct commit to protected main, OR denied the merge) FOLLOWS the denial's remediation_hint — commit to a feature branch → but pr new → obtain a reviewer but review approve → maintainer merge — and the change SUCCEEDS (lands on main, exit 0). Assert the path SUCCEEDS; do NOT merely assert the bypass is blocked.
- [MUST] MUST prove DryRun-no-bypass (CAP-AUTHZ-01): a --dry-run merge by the implementer still returns perm.denied + exit 1 AND persists nothing — trunk unchanged, no commit/ref object written.
- [NEVER] NEVER assert that the forgeable direct-DB-write path to local_review_verdicts is blocked, nor that a raw-git bypass is blocked — both encode FALSE guarantees (R6/R1). The test exercises the governed `but review` channel ONLY.
- [NEVER] NEVER mock the forge, the review store, git, or but-api. The verification service is the real `but` CLI + real git + the real but-db review store.
- [NEVER] NEVER re-implement or modify the gate logic. LOOP-001 CONSUMES GATES-001..005 + AUTHZ-001..003. Touching crates/but-authz/** or the gate implementations is a scope violation.
- [STRICTLY] STRICTLY keep this task a TEST plus only the tiny fixture/helper code it needs. Any production behavior that does not yet exist is a missing dependency (GATES-002..005) — surface it as a blocker, do NOT stub it.
- [STRICTLY] STRICTLY update snapbox snapshots only with SNAPSHOTS=overwrite cargo test -p but when intended; do not weaken assertions (no broad ... swallowing the error code or exit status).

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: the five-assertion reference loop holds in one continuous flow (T-LOOP-006/001/002/003/004)
- [ ] AC-2: the irrigation half — a denied implementer following its remediation_hint LANDS the change (T-LOOP-013)
- [ ] AC-3: DryRun-no-bypass — a --dry-run merge by the implementer still perm.denied + persists nothing (CAP-AUTHZ-01)
- [ ] AC-4: auto-merge denial — enabling auto-merge as the implementer denied perm.denied (ties GATES-003)
- [ ] AC-5: fail-closed at the e2e canary — unset/empty BUT_AGENT_HANDLE → structured perm.denied (exit 1) against the merge action (T-AUTHZ-027/028)
- [ ] All reviews via the governed `but review` action; NO test asserts the forgeable path is blocked (R6/R1)
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: The five-assertion reference loop holds in one continuous flow [PRIMARY]
  GIVEN: fixture `governed_loop_repo` (3 principals; main protected; review requirement min_approvals=1 distinct-from-author; feat with a pending change)
  WHEN:  the full loop is driven through the real `but` CLI (impl commit+pr new; impl merge; reviewer commit then but review approve; maintainer merge with 0 approvals; maintainer merge after the distinct approval @head)
  THEN:  (1) impl commit lands (exit 0, feat advances) + open-PR PERMITTED by the gate (no governance Denial; reaches forge call); (2) impl merge denied perm.denied naming merge (exit 1, trunk unchanged); (3) reviewer commit denied perm.denied + reviewer approve accepted @head; (4) maintainer 0-approval merge denied gate.review_required (exit 1, trunk unchanged); (5) maintainer merge after the distinct approval @head PERMITTED by the gate (no governance Denial; reaches forge merge_review call — forge-network landing structural/out-of-local-scope)
  TEST_TIER: e2e   VERIFICATION_SERVICE: real but CLI + real git + real but-db
  VERIFY: cargo test -p but governed_loop_reference_flow_full_loop
  SCENARIO: NEGATIVE_CONTROL would fail if the loop passes with the impl merge landed (gate is a no-op); a role-name branch in enforcement makes it pass for the wrong reason; the reviewer's commit is accepted; the maintainer merge proceeds with 0 approvals; the approval is counted off-head; the test simulates the loop in-process instead of via the real CLI.

AC-2: The irrigation half — a denied implementer following its remediation_hint lands the change
  GIVEN: fixture `governed_loop_repo` (main protected; review requirement)
  WHEN:  impl attempts a direct commit to protected main (denied branch.protected with a remediation_hint), then FOLLOWS the hint: commit to feat2 → but pr new feat2 → reviewer but review approve feat2 @head → maintainer merge feat2
  THEN:  the direct-to-main commit denied branch.protected (exit 1) with a remediation_hint naming the governed path; following it is PERMITTED end-to-end — feat2 commit lands, and PR-open + reviewer approve + maintainer merge each pass their gate (NO governance Denial at any hop); the governed channel is traversable (forge-network landing structural/out-of-local-scope)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but CLI + real git + real but-db
  VERIFY: cargo test -p but governed_loop_remediation_traversable
  SCENARIO: NEGATIVE_CONTROL would fail if the remediation path is asserted BLOCKED instead of traversable; the remediation_hint is empty or names a non-governed action; the test never follows the hint to a successful merge; following the hint still fails to land the change.

AC-3: DryRun-no-bypass — a --dry-run merge by the implementer still perm.denied + persists nothing
  GIVEN: fixture `governed_loop_repo` with an open feat review and the implementer (no merge)
  WHEN:  impl runs a --dry-run governed merge of feat
  THEN:  still error.code=="perm.denied" naming merge (exit 1) — DryRun does NOT bypass the gate — and persists nothing (trunk unchanged, no commit/ref object written)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but CLI + real git + real but-db
  VERIFY: cargo test -p but governed_loop_dryrun_no_bypass
  SCENARIO: NEGATIVE_CONTROL would fail if DryRun skips the gate so the impl merge is allowed (exit 0); the DryRun denial does not fire (authorization short-circuited under DryRun); a denied DryRun still writes a ref/object; the denial code is not perm.denied.

AC-4: auto-merge denial — enabling auto-merge as the implementer is denied perm.denied
  GIVEN: fixture `governed_loop_repo` with an open feat review and the implementer (no merge)
  WHEN:  impl enables auto-merge via `but pr auto-merge` (routing to set_review_auto_merge)
  THEN:  denied error.code=="perm.denied" naming merge (exit 1) — auto-merge passes through the SAME merge gate as explicit merge — trunk unchanged
  TEST_TIER: integration   VERIFICATION_SERVICE: real but CLI + real git + real but-db
  VERIFY: cargo test -p but governed_loop_auto_merge_denied
  SCENARIO: NEGATIVE_CONTROL would fail if enabling auto-merge as the implementer is allowed (the auto-merge path is ungated relative to explicit merge); auto-merge is gated by a weaker authority; the denial code is not perm.denied; the trunk advances because the auto-merge silently lands the change.

AC-5: Fail-closed at the e2e canary level — unset/empty BUT_AGENT_HANDLE denied perm.denied against the merge action
  GIVEN: fixture `governed_loop_repo` with an open feat review
  WHEN:  a governed merge is attempted with BUT_AGENT_HANDLE unset, then with BUT_AGENT_HANDLE="" (empty)
  THEN:  each is rejected with a STRUCTURED error.code=="perm.denied" (exit 1, no anonymous/default principal, trunk unchanged) — the fail-closed invariant exercised end-to-end at the canary (T-AUTHZ-027/028, UC-AUTHZ-04)
  TEST_TIER: e2e   VERIFICATION_SERVICE: real but CLI + real git + real but-db
  VERIFY: cargo test -p but governed_loop_unset_handle_failclosed
  SCENARIO: NEGATIVE_CONTROL would fail if an unset/empty handle defaults to an allowed/anonymous principal; the no-handle case exits 1 only by panicking rather than the structured contract; the empty handle is accepted as a valid principal.

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, happy_path): full reference loop holds (5 assertions) (T-LOOP-006/001/002/003/004)
    VERIFY: cargo test -p but governed_loop_reference_flow_full_loop
- TC-2 (-> AC-2, edge): remediation path traversable — denied implementer follows the hint and lands the change (T-LOOP-013)
    VERIFY: cargo test -p but governed_loop_remediation_traversable
- TC-3 (-> AC-3, error): DryRun-no-bypass — dry-run impl merge still perm.denied + persists nothing (CAP-AUTHZ-01)
    VERIFY: cargo test -p but governed_loop_dryrun_no_bypass
- TC-4 (-> AC-4, error): auto-merge denial — impl auto-merge denied perm.denied, same gate as explicit merge (ties GATES-003)
    VERIFY: cargo test -p but governed_loop_auto_merge_denied
- TC-5 (-> AC-1, edge): role separation emerges from the functional permission set — loop holds purely from the three AuthoritySets, no role-name in enforcement (UC-LOOP-01 AC-5; corroborates T-LOOP-005)
    VERIFY: cargo test -p but governed_loop_reference_flow_full_loop
- TC-6 (-> AC-2, edge): honest review channel — all reviews via governed but review; no case asserts the forgeable direct-DB-write or raw-git bypass is blocked (R6/R1)
    VERIFY: cargo test -p but governed_loop_remediation_traversable
- TC-7 (-> AC-5, error): fail-closed at the e2e canary — unset/empty BUT_AGENT_HANDLE → structured perm.denied exit 1 against the merge action (T-AUTHZ-027/028, UC-AUTHZ-04)
    VERIFY: cargo test -p but governed_loop_unset_handle_failclosed

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01, CAP-CONFIG-01
provides: the green T-LOOP-006 reference-flow canary (the sprint capstone the PRD requires green before the deep build), plus the T-LOOP-013 traversability proof and the CAP-AUTHZ-01 DryRun-no-bypass + auto-merge-denial proofs — all driven through the real `but` CLI + real git + real but-db, with three distinct principals whose role separation emerges from the functional permission set alone
consumes: GATES-001 (commit gate); GATES-002 (local_review_verdicts review record); GATES-003 (merge gate on merge_review AND set_review_auto_merge); GATES-004 (submit-review/open-PR/comment authz guards + governed verbs); GATES-005 (stale-@head + self-approval evaluator); AUTHZ-001/002/003 (the but-authz primitive)
boundary_contracts:
  - CAP-AUTHZ-01: a DryRun merge by the implementer still resolves BUT_AGENT_HANDLE→Principal and authorizes merge at the but-api forge seam, returning the Denial contract + exit 1 and persisting nothing — DryRun never bypasses the gate.
  - CAP-CONFIG-01: the merge gate's review requirement is read ONLY from the target-ref .gitbutler/gates.toml blob.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but/tests/but/command/governed_loop.rs (NEW) — the headline LOOP-001 integration/e2e test
  - crates/but/tests/but/command/mod.rs (MODIFY) — add `mod governed_loop;` (mirror the existing #[cfg(...)] mod commit2; pattern)
  - crates/but/tests/fixtures/scenario/governed-loop.sh (NEW, if a scripted fixture is preferred over inline invoke_bash seeding)
  - crates/but/tests/but/utils/governed_loop_seed.rs (NEW, OPTIONAL tiny helper) — no production logic
writeProhibited:
  - crates/but-authz/** — CONSUME authorize/Authority/Principal/the config loader, never modify
  - crates/but-api/src/legacy/forge.rs — the merge/auto-merge gate (GATES-003) + forge authz guards (GATES-004); LOOP-001 drives them through the CLI, does not edit them
  - crates/but-api/src/commit/** — the commit gate (GATES-001); consumed, not modified
  - crates/but-db/** and the local_review_verdicts table/migrations — the review record (GATES-002); consumed via the governed but review action, never written directly
  - crates/but/src/command/** and crates/but/src/args/** — the CLI verb definitions (GATES-004); LOOP-001 is a TEST, it does not add or change CLI verbs
  - any gitbutler-* crate beyond what the test harness strictly requires (crates/AGENTS.md)
  - Any file not listed in write_allowed

--------------------------------------------------------------------------------
OUT OF SCOPE
--------------------------------------------------------------------------------
  - The dedicated per-required-group strictness matrix — T-LOOP-008/T-LOOP-009 (only-AI / only-human approval blocks) and T-GATES-012 — DEFERRED to Sprint 04. This task establishes the single-required-group merge gate and the two-group plumbing only.
  - The forgeable local_review_verdicts direct-DB-write path and the raw-git bypass are ACCEPTED-LEAKS (R6/R1) and are explicitly NOT under test.
  - Building the gates themselves (GATES-001..005) and the but-authz primitive (AUTHZ-001..003) — dependencies LOOP-001 consumes; if a governed verb or gate does not yet exist, surface it as a blocker, do not stub it.
  - Mechanism-agnostic commit coverage (T-GATES-016/017) — DEFERRED to Sprint 04.
  - FORGE-COMPLETION (red-hat re-scope, user decision = gate-boundary): the forge-network merge/PR landing (the change/PR appearing on the remote) is NOT asserted locally — `merge_review`/`publish_review` are forge-bound and there is no `but pr merge` CLI verb. The POSITIVE paths prove the gate DECISION (permit, no Denial) on the real seam + that execution reaches the forge call; the forge completion is structural/out-of-local-scope. UPSTREAM ADVISORY: T-LOOP-006 ("merge succeeds") / T-LOOP-013 ("change LANDS") / T-LOOP-004 wording should be reconciled with this forge-locality via /kb-sprint-plan --delta-replan (the local canary proves the GATE traversability, not the forge landing).

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. .spec/prds/governance/07-uc-loop.md (UC-LOOP-01, 18-33)
   Focus: THE SPEC THIS PROVES — the five loop assertions + the traversability AC (32: the denial's remediation_hint names a governed next action that, when followed, succeeds) + the full-loop integration-test AC (33: real surface + real git, 3 principals, all reviews via governed but review, forgeable DB path NOT under test) + the R6 review-integrity caveat (23).
2. .spec/prds/governance/11-e2e-testing-criteria.md (LOOP table 132-142 + Summary 263-264)
   Focus: T-LOOP-006 (full-loop, the HEADLINE GATE), T-LOOP-013 (traversable), T-LOOP-001/002/003/004, T-LOOP-014. Verification bar (8): real but-authz + real but-api + real git, no mocks.
3. .spec/prds/governance/10-technical-requirements/04-api-design.md (57-67, 73-76, 131-138)
   Focus: the governed action surface (but pr new pull_requests:write; merge/but pr auto-merge merge+review requirement; but review approve reviews:write); the merge gate entry legacy/forge::merge_review / set_review_auto_merge; the denial contract; the DryRun note.
4. crates/but/tests/but/utils.rs (120-142)
   Focus: PRIMARY PATTERN (the CLI driver) — Sandbox::but(args) (125) returns a snapbox Command from cargo_bin!("but"); chain .env("BUT_AGENT_HANDLE", handle) per principal, then .assert() + .stdout_eq/.stderr_eq with [..]/... wildcards + .success()/.failure().code(1). Use the sandbox's git/bash helpers, NOT std::process::Command::new("git").
5. crates/but/tests/but/command/commit2.rs (1-60) + crates/but/tests/but/command/mod.rs (1-40)
   Focus: PATTERN_SOURCE for a CLI command test — Sandbox::init_scenario_with_target_and_default_settings("...") then env.but("...").assert().success().stdout_eq(snapbox::str![[...]]). mod.rs shows the #[cfg(...)] mod commit2; declaration to mirror for mod governed_loop;. CLI tests are expensive — happy-path-shaped; update snapshots with SNAPSHOTS=overwrite cargo test -p but.
6. crates/but-testsupport/src/lib.rs (writable_scenario 432; invoke_bash 71; git helper 55; isolated repo config 110)
   Focus: FIXTURE SEEDING — writable_scenario(name)->(gix::Repository, TempDir) yields a REAL writable repo; invoke_bash(script,&repo) runs a bash script anchored to the repo workdir to write+commit .gitbutler/*.toml at refs/heads/main, branch to feat, stage a change. Keep the TempDir alive; read refs via repo.find_reference(name)?.peel_to_id()?.
7. crates/but-api/src/legacy/forge.rs (merge_review 438; set_review_auto_merge 469)
   Focus: THE MERGE-GATE SEAM (consumed, not modified) — merge_review is the governed merge the maintainer drives + the implementer is denied; set_review_auto_merge is the auto-merge the implementer is denied (AC-4). GATES-003 adds the merge authz guard; LOOP-001 reaches both via the real CLI.
8. crates/but/src/args/forge.rs (pr::Platform alias review/mr; Subcommands::New, AutoMerge)
   Focus: THE CLI VERB SURFACE the loop drives — but pr / but review / but mr; GATES-004 adds the governed approve/request-changes/comment verbs. LOOP-001 invokes but pr new, but review approve, but pr auto-merge. If a governed verb is missing, that is a GATES-004 blocker, not something to stub.
9. crates/AGENTS.md, crates/but/AGENTS.md, /Users/justinrich/Projects/brain/docs/rust/testing.md
   Focus: house testing conventions — use but-testsupport for scenario creation; CLI tests use env.but(...).assert() with snapbox + [..]/... wildcards (update with SNAPSHOTS=overwrite cargo test -p but); use invoke_bash/invoke_git not std::process::Command::new("git"); CLI tests are expensive — happy-path only. Result<T,E> + anyhow; but_error::Code for consumer classification.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- Reference-flow canary green (the headline T-LOOP-006 gate): `cargo test -p but governed_loop_reference_flow_full_loop`  -> Exit 0; AC-1 green
- Traversability proof green: `cargo test -p but governed_loop_remediation_traversable`  -> Exit 0; AC-2 green
- DryRun-no-bypass + auto-merge denial green: `cargo test -p but governed_loop_dryrun_no_bypass governed_loop_auto_merge_denied`  -> Exit 0; AC-3 + AC-4 green
- All LOOP-001 tests green together: `cargo test -p but governed_loop`  -> Exit 0
- Test crate compiles: `cargo check -p but --all-targets`  -> Exit 0
- No forgeable-bypass assertion (R6/R1): `! grep -rEn 'local_review_verdicts|INSERT INTO|raw[ _-]?git[ _-]?bypass|direct[ _-]?db' crates/but/tests/but/command/governed_loop.rs`  -> No matches — the test exercises the governed but review channel ONLY
- No in-process gate simulation (real CLI boundary only): `! grep -rEn 'but_authz::authorize|enforce_commit_gate|enforce_merge_gate|in_memory|inject_principal' crates/but/tests/but/command/governed_loop.rs`  -> No matches — the loop is driven through the real but CLI via env.but(...)
- Only write_allowed files modified: `git diff --name-only`  -> Only governed_loop.rs, mod.rs, and (if used) the new fixture/helper files
- Clippy clean: `cargo clippy -p but --all-targets`  -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: CLI-driven 3-principal real-git loop — seed a single REAL repo with but_testsupport::writable_scenario + invoke_bash (committed permissions.toml + gates.toml at refs/heads/main, a feat branch with a pending change), then drive the entire implement→review→merge loop through the real `but` CLI via the snapbox harness — switching the acting principal per step with .env("BUT_AGENT_HANDLE", handle) — asserting exit codes, the {error:{code,message,remediation_hint}} JSON contract, and real-git ref invariants (feat advanced; trunk unchanged on denials; trunk advanced on the satisfied merge + remediation path). Role separation is an EMERGENT property of the three AuthoritySets, asserted by outcomes — not by any role-name branch.
pattern_source: crates/but/tests/but/command/commit2.rs:5 + crates/but/tests/but/utils.rs:125 (Sandbox::but with .env) + crates/but-testsupport/src/lib.rs:432 (writable_scenario) + :71 (invoke_bash)
anti_pattern: Mocking the forge or review store; injecting principals in-memory instead of a real committed permissions.toml; calling but_authz::authorize/the gate helpers in-process to simulate the loop instead of driving the real CLI; asserting the forgeable direct-DB-write or raw-git bypass is blocked (false guarantee, R6/R1); a role-name branch in enforcement that makes the loop pass for the wrong reason; asserting only that the bypass is blocked without proving the remediation path is traversable; letting DryRun skip the gate.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — Drives the headline canary end-to-end through the REAL `but` CLI binary against a real git repo seeded by but_testsupport::writable_scenario + invoke_bash, asserting exit codes and the structured denial contract via the snapbox harness. A TEST-CENTRIC task that CONSUMES GATES-001..005 + AUTHZ-001..003 and re-implements none of them. Owns the multi-principal sandbox fixture, the snapbox stdout/exit-code assertions, and the ref-advanced/ref-unchanged invariants against real git refs.
reviewer: rust-reviewer
coding_standards: crates/AGENTS.md, crates/but/AGENTS.md, crates/WORKSPACE_MODEL.md

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: GATES-001, GATES-002, GATES-003, GATES-004, GATES-005, AUTHZ-001, AUTHZ-002, AUTHZ-003
Blocks:     (sprint capstone — none)
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "LOOP-001",
  "proposed_by": "rust-planner",
  "verification_policy": { "requires_tests": true, "requires_red_evidence": true, "requires_seeded_evidence": true },
  "fixtures": {
    "governed_loop_repo": {
      "description": "A real git repo (but-testsupport writable_scenario) whose target ref main has committed .gitbutler/permissions.toml (implementer=contents:write+pull_requests:write [no merge]; reviewer in group code-reviewers->reviews:write [no contents:write]; maintainer in group maintainers->merge) and .gitbutler/gates.toml (main protected + a [[gate]] review requirement min_approvals=1 require_distinct_from_author=true), plus a feature branch feat with a pending worktree change. Seeded via the REAL CLI/git, not in-memory injection.",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"governed-loop\");",
        "invoke_bash: write .gitbutler/permissions.toml with [[principal]] id=\"implementer\" permissions=[\"contents:write\",\"pull_requests:write\"]; [[principal]] id=\"reviewer\" groups=[\"code-reviewers\"]; [[principal]] id=\"maintainer\" groups=[\"maintainers\"]; [[group]] name=\"code-reviewers\" permissions=[\"reviews:write\"]; [[group]] name=\"maintainers\" permissions=[\"merge\"]",
        "invoke_bash: write .gitbutler/gates.toml with [[branch]] name=\"main\" protected=true and [[gate]] branch=\"main\" min_approvals=1 require_distinct_from_author=true",
        "invoke_bash: git add -A && git commit -m \"governance config\" (commits both blobs at refs/heads/main)",
        "invoke_bash: git checkout -b feat; make an uncommitted change to file.txt (staged for commit)",
        "capture the seeded base sha of refs/heads/main for later ref-unchanged assertions"
      ]
    },
    "governed_loop_repo_head_advanced": {
      "description": "Variant of governed_loop_repo used to keep AC-2/AC-3/AC-4 independent: a fresh seeded instance (or a fresh feature branch feat2) so each proof (remediation traversal, DryRun, auto-merge) starts from a clean, re-runnable state with the same committed permissions.toml/gates.toml at refs/heads/main. Seeded via the REAL CLI/git.",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"governed-loop\"); (reuse the same scenario script)",
        "for AC-2: branch feat2 off main with a pending change; the implementer first attempts a direct commit to protected main (captures the branch.protected remediation_hint), then follows it on feat2",
        "for AC-3/AC-4: open a feat review as the implementer first, then exercise the --dry-run merge / the auto-merge enable as the implementer"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN governed_loop_repo (3 principals; main protected; review requirement min_approvals=1 distinct-from-author) WHEN the full implement->review->merge loop runs through the real `but` CLI THEN implementer commit+PR accepted; implementer merge perm.denied naming merge (exit 1, trunk unchanged); reviewer commit denied + reviewer `but review approve` accepted @head; maintainer zero-approval merge gate.review_required (exit 1, trunk unchanged); maintainer merge after the distinct approval @head proceeds (exit 0) and the change lands",
      "verify": "cargo test -p but governed_loop_reference_flow_full_loop",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "e2e",
        "verification_service": "real but CLI + real git + real but-db",
        "negative_control": { "would_fail_if": [
          "the loop passes even though the implementer's merge landed (the gate is a no-op stub so trunk advanced on step 2)",
          "a role-name branch in enforcement makes the test pass for the wrong reason (special-casing implementer/reviewer/maintainer instead of the functional permission set)",
          "the reviewer's commit is accepted (the commit gate omits the contents:write check)",
          "the maintainer merge proceeds with zero distinct approvals (the review requirement is vacuously satisfied / empty)",
          "the approval is counted off-head yet the merge proceeds (the gate ignores the head pin)",
          "the test simulates the loop in-process via but_authz::authorize / in-memory principal injection instead of the real CLI boundary"
        ] },
        "evidence": { "artifact_type": "stdout", "required_capture": true },
        "cases": [
          {
            "start_ref": "governed_loop_repo",
            "action": { "actor": "cli_user", "steps": ["BUT_AGENT_HANDLE=implementer: `but commit --branch feat -m \"feature work\"`", "BUT_AGENT_HANDLE=implementer: `but pr new feat -m \"PR: feature work\"`"] },
            "end_state": {
              "must_observe": ["the commit lands on `feat` (process exits `0`)", "`feat` HEAD sha `!=` the seeded base sha", "the `but pr new` open-PR is PERMITTED by the gate — pull_requests:write holder, NO `error.code == \"perm.denied\"` (0 governance denials); execution reaches the forge `publish_review` call (any failure is a forge/remote error, NOT a governance Denial)"],
              "must_not_observe": ["`error.code == \"perm.denied\"` on the commit or the open-PR", "exit `1` from a governance Denial", "trunk/main sha advanced", "no commit created on feat (0 new commits)"]
            }
          },
          {
            "start_ref": "governed_loop_repo",
            "action": { "actor": "cli_user", "steps": ["BUT_AGENT_HANDLE=implementer: attempt the governed merge of the `feat` review (the merge_review path)"] },
            "end_state": {
              "must_observe": ["`error.code == \"perm.denied\"`", "message names `\"merge\"`", "non-empty `remediation_hint`", "process exits `1`", "trunk/main HEAD sha `==` the seeded base sha"],
              "must_not_observe": ["the change landed on the trunk", "trunk/main sha advanced", "exit `0`"]
            }
          },
          {
            "start_ref": "governed_loop_repo",
            "action": { "actor": "cli_user", "steps": ["BUT_AGENT_HANDLE=reviewer: `but commit --branch feat -m \"reviewer edit\"` (lacks contents:write)", "BUT_AGENT_HANDLE=reviewer: `but review approve feat` (governed approval @head)"] },
            "end_state": {
              "must_observe": ["the commit is denied `error.code == \"perm.denied\"` (exit `1`)", "the reviewer's edit never reaches a ref (`feat` sha unchanged by the denied commit, 0 advance)", "the `but review approve` is accepted (exit `0`)", "an approving verdict is recorded (1 row) at the current `feat` head, head_oid `==` feat HEAD"],
              "must_not_observe": ["the reviewer commit landed", "`feat` advanced by the reviewer commit", "the review submission denied", "0 verdict rows recorded after the accepted approve"]
            }
          },
          {
            "start_ref": "governed_loop_repo",
            "action": { "actor": "cli_user", "steps": ["BUT_AGENT_HANDLE=maintainer: attempt the governed merge of `feat` with ZERO distinct approvals @head"] },
            "end_state": {
              "must_observe": ["`error.code == \"gate.review_required\"`", "the `unmet` payload lists the unmet review requirement", "process exits `1`", "trunk/main HEAD sha `==` the seeded base sha"],
              "must_not_observe": ["the change landed on the trunk", "trunk/main sha advanced", "exit `0`"]
            }
          },
          {
            "start_ref": "governed_loop_repo",
            "action": { "actor": "cli_user", "steps": ["(reviewer approved @head in the prior step) BUT_AGENT_HANDLE=maintainer: attempt the governed merge of `feat` (gate-boundary re-scope: assert the gate PERMITS; forge-network landing is out of local scope)"] },
            "end_state": {
              "must_observe": ["the merge is PERMITTED by the gate — `authorize(merge)` Ok + the distinct approval @head satisfies the requirement: the output contains NO `error.code == \"perm.denied\"` and NO `error.code == \"gate.review_required\"` (0 governance denials)", "execution reaches the governed `merge_review` body past the gate (any failure is a forge/remote error, NOT a governance Denial)"],
              "must_not_observe": ["`error.code == \"perm.denied\"` or `error.code == \"gate.review_required\"` raised for the satisfied, merge-holding request", "a governance Denial blocks the merge (0 denials expected)", "the maintainer denied despite holding merge with the approval @head"]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN governed_loop_repo WHEN a denied implementer follows its remediation_hint (feature branch -> but pr new -> reviewer but review approve @head -> maintainer merge) THEN the direct-to-main commit is branch.protected with a hint naming the governed path AND following that path lands the change successfully on the trunk (traversable irrigation half)",
      "verify": "cargo test -p but governed_loop_remediation_traversable",
      "maps_to_ac": null,
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but CLI + real git + real but-db",
        "negative_control": { "would_fail_if": [
          "the remediation path is asserted blocked instead of traversable (the test proves the dam but not the irrigation)",
          "the remediation_hint is empty or names a non-governed/non-existent next action",
          "the test only asserts the direct-to-main commit is denied and never follows the hint to a successful merge",
          "following the hint still fails to land the change (no change on trunk)"
        ] },
        "evidence": { "artifact_type": "stdout", "required_capture": true },
        "cases": [
          {
            "start_ref": "governed_loop_repo_head_advanced",
            "action": { "actor": "cli_user", "steps": ["BUT_AGENT_HANDLE=implementer: `but commit --branch main -m \"direct to main\"` (denied; capture the remediation_hint)"] },
            "end_state": {
              "must_observe": ["`error.code == \"branch.protected\"`", "non-empty `remediation_hint` naming the governed feature-branch -> reviewed-merge path", "process exits `1`", "trunk/main HEAD sha `==` the seeded base sha"],
              "must_not_observe": ["the commit landed on `main`", "trunk/main sha advanced", "exit `0`"]
            }
          },
          {
            "start_ref": "governed_loop_repo_head_advanced",
            "action": { "actor": "cli_user", "steps": ["Follow the remediation_hint: BUT_AGENT_HANDLE=implementer `but commit --branch feat2 -m \"feature work\"`", "BUT_AGENT_HANDLE=implementer `but pr new feat2 -m \"PR: feature work\"`", "BUT_AGENT_HANDLE=reviewer `but review approve feat2` (governed approval @head)", "BUT_AGENT_HANDLE=maintainer attempt the governed merge of `feat2`"] },
            "end_state": {
              "must_observe": ["the implementer feat2 commit lands (exit `0`, feat2 advances)", "the reviewer governed approval @head is recorded (exit `0`, 1 verdict row)", "EVERY gate along the governed remediation path PERMITS the right principal — NO `error.code == \"perm.denied\"` / `branch.protected` / `gate.review_required` at any hop (0 governance denials); the maintainer merge reaches the governed `merge_review` body past the gate", "the governed channel is traversable: the denial's `remediation_hint` named a path the gates permit end-to-end (forge-network landing is structural/out-of-local-scope)"],
              "must_not_observe": ["any step of the remediation path denied by a governance Denial", "a governance Denial at any hop (0 denials expected on the governed path)", "the merge blocked with `gate.review_required` after the distinct approval @head"]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN governed_loop_repo with an open feat review WHEN the implementer runs a --dry-run governed merge THEN it still returns perm.denied (exit 1) and persists nothing (trunk unchanged, no object/ref written) — DryRun never bypasses the gate (CAP-AUTHZ-01)",
      "verify": "cargo test -p but governed_loop_dryrun_no_bypass",
      "maps_to_ac": null,
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but CLI + real git + real but-db",
        "negative_control": { "would_fail_if": [
          "DryRun skips the gate so the implementer merge is allowed (exit 0)",
          "the DryRun denial does not fire because authorization is short-circuited under DryRun",
          "a denied DryRun still writes a ref/object (does not persist nothing — persistence leaked)",
          "the gate is a no-op stub under DryRun so the merge bypasses authorize() entirely",
          "the denial code is not perm.denied"
        ] },
        "evidence": { "artifact_type": "stdout", "required_capture": true },
        "cases": [
          {
            "start_ref": "governed_loop_repo_head_advanced",
            "action": { "actor": "cli_user", "steps": ["BUT_AGENT_HANDLE=implementer: run the governed merge of `feat` with `--dry-run`"] },
            "end_state": {
              "must_observe": ["`error.code == \"perm.denied\"`", "message names `\"merge\"`", "process exits `1`", "trunk/main HEAD sha `==` base AND no commit/ref object was persisted for this attempt"],
              "must_not_observe": ["exit `0`", "trunk/main sha advanced", "a persisted ref/object from the denied dry run", "DryRun reported as allowed/preview-Ok"]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN governed_loop_repo with an open feat review WHEN the implementer enables auto-merge via but pr auto-merge THEN it is denied perm.denied naming merge (exit 1) — auto-merge passes through the same merge gate as explicit merge (ties GATES-003)",
      "verify": "cargo test -p but governed_loop_auto_merge_denied",
      "maps_to_ac": null,
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but CLI + real git + real but-db",
        "negative_control": { "would_fail_if": [
          "enabling auto-merge as the implementer is allowed because set_review_auto_merge is a no-op stub ungated relative to merge_review",
          "auto-merge is gated by a different/weaker authority than explicit merge",
          "the denial code is not perm.denied / does not name merge",
          "the trunk advances because the auto-merge silently lands the change"
        ] },
        "evidence": { "artifact_type": "stdout", "required_capture": true },
        "cases": [
          {
            "start_ref": "governed_loop_repo_head_advanced",
            "action": { "actor": "cli_user", "steps": ["BUT_AGENT_HANDLE=implementer: `but pr auto-merge feat` (enable auto-merge on the feat review)"] },
            "end_state": {
              "must_observe": ["`error.code == \"perm.denied\"`", "message names `\"merge\"`", "process exits `1`", "trunk/main HEAD sha `==` the seeded base sha"],
              "must_not_observe": ["auto-merge enabled", "exit `0`", "trunk/main sha advanced"]
            }
          }
        ]
      }
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN governed_loop_repo with an open feat review WHEN a governed merge is attempted with BUT_AGENT_HANDLE unset, then with BUT_AGENT_HANDLE=\"\" THEN each is rejected with a STRUCTURED error.code==\"perm.denied\" (exit 1, no anonymous/default principal, trunk unchanged) — fail-closed at the e2e canary level (T-AUTHZ-027/028, UC-AUTHZ-04)",
      "verify": "cargo test -p but governed_loop_unset_handle_failclosed",
      "maps_to_ac": null,
      "scenario": {
        "tier": "holdout",
        "test_tier": "e2e",
        "verification_service": "real but CLI + real git + real but-db",
        "negative_control": { "would_fail_if": [
          "an unset/empty handle defaults to an allowed/anonymous principal and the merge is attempted",
          "the no-handle case exits 1 only by panicking rather than returning the structured perm.denied contract",
          "the gate is a no-op stub that skips principal resolution under an empty handle",
          "the empty-string handle is accepted as a valid principal"
        ] },
        "evidence": { "artifact_type": "stdout", "required_capture": true },
        "cases": [
          {
            "start_ref": "governed_loop_repo_head_advanced",
            "action": { "actor": "cli_user", "steps": ["BUT_AGENT_HANDLE unset: attempt the governed merge of `feat`", "BUT_AGENT_HANDLE=\"\" (empty string): attempt the governed merge of `feat`"] },
            "end_state": {
              "must_observe": ["each invocation: `error.code == \"perm.denied\"` (structured, not a bare panic)", "each process exits `1` with no principal bound", "trunk/main HEAD sha `==` the seeded base sha (0 advance)"],
              "must_not_observe": ["an anonymous/default principal accepted", "exit `0`", "the empty handle accepted as a principal", "trunk/main sha advanced"]
            }
          }
        ]
      }
    },
    { "id": "TC-1", "type": "test_criterion", "description": "full reference loop holds (5 assertions) (T-LOOP-006/001/002/003/004)", "verify": "cargo test -p but governed_loop_reference_flow_full_loop", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "remediation path is traversable — denied implementer follows the hint and lands the change (T-LOOP-013)", "verify": "cargo test -p but governed_loop_remediation_traversable", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "DryRun-no-bypass — dry-run implementer merge still perm.denied + persists nothing (CAP-AUTHZ-01)", "verify": "cargo test -p but governed_loop_dryrun_no_bypass", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "auto-merge denial — implementer auto-merge denied perm.denied, same gate as explicit merge (ties GATES-003)", "verify": "cargo test -p but governed_loop_auto_merge_denied", "maps_to_ac": "AC-4" },
    { "id": "TC-5", "type": "test_criterion", "description": "role separation emerges from the functional permission set — loop holds purely from the three AuthoritySets, no role-name in enforcement required (UC-LOOP-01 AC-5; corroborates T-LOOP-005)", "verify": "cargo test -p but governed_loop_reference_flow_full_loop", "maps_to_ac": "AC-1" },
    { "id": "TC-6", "type": "test_criterion", "description": "honest review channel — all reviews via governed but review; no case asserts the forgeable direct-DB-write or raw-git bypass is blocked (R6/R1)", "verify": "cargo test -p but governed_loop_remediation_traversable", "maps_to_ac": "AC-2" },
    { "id": "TC-7", "type": "test_criterion", "description": "fail-closed at the e2e canary level — unset/empty BUT_AGENT_HANDLE → structured perm.denied exit 1 against the merge action (T-AUTHZ-027/028, UC-AUTHZ-04)", "verify": "cargo test -p but governed_loop_unset_handle_failclosed", "maps_to_ac": "AC-5" }
  ]
}
-->
</details>
