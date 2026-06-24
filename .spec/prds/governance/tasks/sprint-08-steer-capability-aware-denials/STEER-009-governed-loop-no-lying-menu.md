# STEER-009: Extend `governed_loop` for gate-state-aware no-lying-menu — replay each offered action in its stated context, plus concurrent-ref-advance + serialization-fault cases; audit/update any whole-object-equality assertion on `Denial`/`MergeGateError`

## What this does

Prove the menu never lies: extend governed_loop so every offered `authorized_actions` command, replayed in its stated context against a real subsequent run, succeeds (or hits its own legitimate non-perm.denied gate); add a concurrent-ref-advance case (clean re-denial) and a serialization-fault case (fail-closed exit 1); and audit/update any whole-object-equality assertion that the new fields would break, adding positive new-field assertions.

## Why

Sprint 08 (STEER — Capability-Aware Denials) · PRD UC-STEER-06 · Capability CAP-STEER-01. For a branch.protected denial against a feature-branch-commit + review menu, replaying the feature-branch commit advances the feature ref (exit 0) and replaying the review action succeeds, while neither reproduces the original branch.protected; a config advance between denial and replay yields a clean re-denial (exit 1, valid JSON, no panic); a forced serialization fault still emits code/message/remediation_hint + exit 1; no whole-object-equality assert_eq! on Denial/MergeGateError survives, and positive assertions confirm the new fields. The no-lying-menu proof covers all four menu-bearing denial types.

## How to verify

PRIMARY **AC-1** — `cargo test -p but governed_loop_no_lying_menu_replay` (Every offered command on a branch.protected menu succeeds in its stated context (no lying menu) [PRIMARY]). Full gate set in the spec below.

## Scope

- crates/but/tests/but/command/governed_loop.rs (MODIFY) — extend the CliErrorEnvelope reader for the new fields; add the replay, concurrent-ref-advance, and serialization-fault test functions; do not weaken existing assertions
- crates/but-api/tests/commit_gate.rs (MODIFY) — audit/convert any whole-object equality + add positive new-field assertions
- crates/but-api/tests/merge_gate.rs (MODIFY) — audit/convert any whole-object equality + add positive new-field assertions

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: STEER-009 - Extend `governed_loop` for gate-state-aware no-lying-menu — replay each offered action in its stated context, plus concurrent-ref-advance + serialization-fault cases; audit/update any whole-object-equality assertion on `Denial`/`MergeGateError`
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     XL  (300 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-STEER-06
CAPABILITIES: CAP-STEER-01

RUNTIME_COMMANDS:
  test:  cargo test -p but governed_loop_no_lying_menu_replay governed_loop_review_required_menu_replay governed_loop_perm_denied_menu_replay governed_loop_admin_write_menu_replay  |   cargo test -p but governed_loop_concurrent_ref_advance_clean_redenial governed_loop_serialization_fault_failclosed   |   cargo test -p but governed_loop_dryrun_serialization_fault_failclosed   |   cargo test -p but governed_loop && cargo test -p but-api commit_gate merge_gate
  lint:  cargo clippy -p but -p but-api --all-targets

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
For a branch.protected denial against a feature-branch-commit + review menu, replaying the feature-branch commit advances the feature ref (exit 0) and replaying the review action succeeds, while neither reproduces the original branch.protected; a config advance between denial and replay yields a clean re-denial (exit 1, valid JSON, no panic); a forced serialization fault still emits code/message/remediation_hint + exit 1; no whole-object-equality assert_eq! on Denial/MergeGateError survives, and positive assertions confirm the new fields. This is proven across all four menu-bearing denial types (branch.protected, gate.review_required, perm.denied, and admin-write/governance).

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST EXTEND the existing `crates/but/tests/but/command/governed_loop.rs` harness — reuse its `governed_loop_env` fixture builder, the `CliErrorEnvelope` reader (extend it to also read `class`/`held_permissions`/`authorized_actions`/`do_not`), and the assert_denial style. For EACH of the FOUR menu-bearing denial types — `branch.protected` (commit gate), `gate.review_required` (merge gate via `pr merge`), `perm.denied`/missing-authority (commit gate), AND `administration:write`/admin-write (governance CLI via `but perm grant/revoke`) — parse `authorized_actions[i].command` from the serialized JSON and REPLAY each command in its stated context against a real subsequent `but` run, asserting exit 0 OR its own legitimate non-`perm.denied`/non-`gate.review_required` gate (never reproducing the ORIGINAL denial's code/predicate at the denied ref). T-STEER-024 requires every denial type's menu replayed, not branch.protected alone.
- [MUST] MUST add a `gate.review_required` (merge-gate) menu-replay case: `maintainer` (group maintainers -> merge) attempts `pr merge 77` with zero distinct approvals so the merge gate denies `gate.review_required` (a `MergeGateError`); replay each of that menu's review-status/hand-off/discovery affordances and assert exit 0 OR a legitimate non-`gate.review_required` gate (e.g. the forge boundary), never reproducing `gate.review_required` at the merge target.
- [MUST] MUST add a `perm.denied` (missing-authority) commit-gate menu-replay case: `reviewer` (reviews:write, NO contents:write) attempts a commit so the commit gate denies `perm.denied` (a resolved principal lacking authority — a `Denial`); replay each of that menu's review/discovery affordances and assert exit 0 OR a legitimate non-`perm.denied` gate, never reproducing `perm.denied` at the denied commit ref.
- [MUST] MUST add a concurrent-ref-advance case: produce a denial (capturing its menu) on a ref pinned at OID X, then advance the target-ref governance config via invoke_bash (a new commit at the target ref) BETWEEN denial and replay, then replay an offered command — asserting a CLEAN re-denial (exit 1, parseable JSON error envelope, refs unchanged on the denied side, NO panic, NO inconsistent state) rather than a success or a crash (security MED #4 — the ref-pin temporal window).
- [MUST] MUST add a serialization-fault case driven by STEER-005's EXACT test/debug-gated seam: set the environment variable `BUT_STEER_FORCE_SERIALIZATION_FAULT` in the real `but` CLI subprocess (NEVER an improvised fault) so the NEW steering fields' serialization faults, and assert the denial STILL emits `code`/`message`/`remediation_hint` + exit 1 — existing fields render independently of the new ones (fail-closed at serialization, no-stub mandate: a real exercised seam).
- [MUST] MUST audit `governed_loop.rs`, `crates/but-api/tests/commit_gate.rs`, and `crates/but-api/tests/merge_gate.rs` for any WHOLE-OBJECT-EQUALITY assertion (`assert_eq!` on a full `Denial`/`MergeGateError` value or on a serialized-blob equality) — update any found to field-level assertions (these break on the new fields; these are hand-assertion tests, `SNAPSHOTS=overwrite` does NOT apply) AND add positive assertions for the new `class`/`held_permissions`/`authorized_actions`/`do_not` fields.
- [MUST] MUST add a DryRun-under-serialization-fault case (§9.5 requires fail-closed at DryRun too — DryRun-no-bypass exercises different early-exit branches): run the denied action under `--dry-run` WITH `BUT_STEER_FORCE_SERIALIZATION_FAULT` active and assert exit 1 + existing fields present + `0` new git objects/refs mutated (DryRun persists nothing even under the fault), reusing the `object_count`/`ref_id` before/after pattern from `governed_loop_dryrun_no_bypass`.
- [NEVER] NEVER assert via an `insta` snapshot or rely on `SNAPSHOTS=overwrite` — these are hand-assertion tests; field-level substring/equality asserts only.
- [NEVER] NEVER treat a replayed command hitting its OWN legitimate downstream gate (e.g. a forge-boundary failure with no governance denial, or a different non-perm.denied gate) as a lying-menu failure — only reproducing the ORIGINAL denial's (code, predicate) at the denied ref is the failure (use the shipped assert_no_governance_denial / forge-boundary helpers).
- [NEVER] NEVER weaken any existing governed_loop assertion (ref-unchanged, exit-code, code/message/hint) — extend only.
- [STRICTLY] STRICTLY drive every replay through the real `but` CLI subprocess against real gix fixtures (the env.but(...) harness), never an in-process stub or a mocked envelope — the menu must be the one the gate actually serialized.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: Every offered command on a branch.protected menu succeeds in its stated context (no lying menu) [PRIMARY]
- [ ] AC-2: Every offered command on a gate.review_required (merge-gate) menu succeeds in its stated context
- [ ] AC-3: Every offered command on a perm.denied (missing-authority) commit-gate menu succeeds in its stated context
- [ ] AC-4: Concurrent-ref-advance yields a clean re-denial (no panic / inconsistent state)
- [ ] AC-5: Serialization fault (BUT_STEER_FORCE_SERIALIZATION_FAULT) on the new fields still denies with existing fields + exit 1
- [ ] AC-6: DryRun under a serialization fault still fails closed (exit 1, existing fields, zero mutations)
- [ ] AC-7: Whole-object-equality assertions audited/updated + positive new-field assertions added
- [ ] AC-8: Every offered command on an admin-write (governance, `but perm grant` or equivalent admin mutator) menu succeeds in its stated context or hits a legitimate non-`perm.denied` gate; none reproduces the original `perm.denied` at the denied admin ref
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: Every offered command on a branch.protected menu succeeds in its stated context (no lying menu) [PRIMARY] [PRIMARY]
  GIVEN: the committed `governed_loop_branch_protected` fixture; a principal (e.g. implementer holding contents:write on its own feature branch, or reviewer with review grants) hits a denial whose `authorized_actions` lists a feature-branch commit and/or review actions in their stated contexts
  WHEN:  each `authorized_actions[i].command` is parsed from the serialized denial JSON and REPLAYED in its stated context against a real subsequent `but` run
  THEN:  every offered command exits 0 (the feature-branch commit advances the feature ref; the review action succeeds) OR hits its own legitimate non-`perm.denied` gate (e.g. forge boundary); NO offered command reproduces the ORIGINAL denial's code at the denied ref
  TEST_TIER: integration   VERIFICATION_SERVICE: governed_loop
  VERIFY: cargo test -p but governed_loop_no_lying_menu_replay
  SCENARIO: would fail if lying-menu; stub; empty; static | must observe: the replayed feature-branch commit exits `0` and the feature ref `ref_id` changes (`!=` its pre-replay value); each offered command exits `0` or hits a non-governance/non-`perm.denied` gate; at least `2` offered commands replayed (a non-degenerate menu) | must NOT observe: any offered command reproducing the original `perm.denied`/`branch.protected` code at the denied ref (`0` such); an `empty` authorized_actions for an actor-correctable denial; the feature ref `unchanged` after a replayed feature-branch commit

AC-2: Every offered command on a gate.review_required (merge-gate) menu succeeds in its stated context
  GIVEN: the committed `governed_loop_branch_protected` fixture; `maintainer` (group maintainers -> merge) attempts `pr merge 77` with zero distinct approvals so the merge gate denies `gate.review_required` (a `MergeGateError`), whose `authorized_actions` lists review-status / hand-off / discovery affordances
  WHEN:  each `authorized_actions[i].command` is parsed from the serialized merge-gate denial JSON and REPLAYED in its stated context against a real subsequent `but` run with the same BUT_AGENT_HANDLE=maintainer
  THEN:  every offered command exits 0 OR hits its own legitimate non-`perm.denied`/non-`gate.review_required` gate (e.g. the forge boundary for a review-status fetch); NO offered command reproduces the ORIGINAL `gate.review_required` at the denied merge target — proving the merge-gate menu is also non-lying, not just the branch.protected menu
  TEST_TIER: integration   VERIFICATION_SERVICE: governed_loop
  VERIFY: cargo test -p but governed_loop_review_required_menu_replay
  SCENARIO: would fail if lying-menu; stub; empty; static | must observe: at least `2` offered commands replayed off the `gate.review_required` menu (review-status + discovery, a non-degenerate menu); each offered command exits `0` or hits a non-governance/non-`gate.review_required` gate (e.g. `forge merge_review boundary` for a review-status action) | must NOT observe: any offered command reproducing the original `gate.review_required` code at the denied merge target (`0` such); an `empty` authorized_actions for the actor-correctable `gate.review_required` denial

AC-3: Every offered command on a perm.denied (missing-authority) commit-gate menu succeeds in its stated context
  GIVEN: the committed `governed_loop_branch_protected` fixture; `reviewer` (group code-reviewers -> reviews:write, NO contents:write) attempts a commit so the commit gate denies `perm.denied` (a resolved principal lacking contents:write — a `Denial`), whose `authorized_actions` lists review + discovery affordances
  WHEN:  each `authorized_actions[i].command` is parsed from the serialized commit-gate `perm.denied` JSON and REPLAYED in its stated context against a real subsequent `but` run with the same BUT_AGENT_HANDLE=reviewer
  THEN:  every offered command exits 0 (e.g. a review action the reviewer holds reviews:write for) OR hits its own legitimate non-`perm.denied` gate; NO offered command reproduces the ORIGINAL `perm.denied` at the denied commit ref — proving the missing-authority (perm.denied) menu is also non-lying
  TEST_TIER: integration   VERIFICATION_SERVICE: governed_loop
  VERIFY: cargo test -p but governed_loop_perm_denied_menu_replay
  SCENARIO: would fail if lying-menu; stub; empty; static | must observe: at least `2` offered commands replayed off the `perm.denied` menu (a review affordance + discovery, a non-degenerate menu); each offered command exits `0` or hits a non-governance/non-`perm.denied` gate (e.g. forge boundary for a review submission) | must NOT observe: any offered command reproducing the original `perm.denied` code at the denied commit ref (`0` such); an `empty` authorized_actions for the actor-correctable `perm.denied` denial

AC-4: Concurrent-ref-advance yields a clean re-denial (no panic / inconsistent state)
  GIVEN: the committed `governed_loop_branch_protected` fixture; a denial captured with its menu while the target ref is pinned at OID X
  WHEN:  the target-ref governance config is advanced via invoke_bash (a new commit at the target ref) BETWEEN denial and replay, then an offered command is replayed
  THEN:  the replay returns a CLEAN re-denial — exit 1, a parseable JSON error envelope, the denied-side ref unchanged, NO panic and NO inconsistent state (the ref-pin temporal window behaves as a clean re-denial, not a crash or a silent success)
  TEST_TIER: integration   VERIFICATION_SERVICE: governed_loop
  VERIFY: cargo test -p but governed_loop_concurrent_ref_advance_clean_redenial
  SCENARIO: would fail if stub; static; disconnect | must observe: the replay exits `1`; stderr is a parseable JSON envelope whose `code` is a stable string (e.g. `branch.protected`); the denied-side `ref_id` `==` its pre-replay value | must NOT observe: a panic / non-`1` abnormal exit / signal termination (`no` clean envelope); a successful landing that bypassed the advanced config (deny->allow flip); unparseable / truncated / `empty` JSON on stderr

AC-5: Serialization fault (BUT_STEER_FORCE_SERIALIZATION_FAULT) on the new fields still denies with existing fields + exit 1
  GIVEN: the committed `governed_loop_branch_protected` fixture; the real `but` CLI subprocess run with the environment variable `BUT_STEER_FORCE_SERIALIZATION_FAULT` set (STEER-005's test/debug-gated best-effort-serialization fault seam — the EXACT seam, not an improvised one)
  WHEN:  a denied action (e.g. implementer merge) is run through the real `but` CLI with `BUT_STEER_FORCE_SERIALIZATION_FAULT` active, faulting the steering-field serialization
  THEN:  the action is STILL denied with exit 1 and stderr carries `code`, `message`, and `remediation_hint` (existing fields render independently of the new ones — fail-closed at serialization via STEER-005's real seam, never deny->allow)
  TEST_TIER: integration   VERIFICATION_SERVICE: governed_loop
  VERIFY: cargo test -p but governed_loop_serialization_fault_failclosed
  SCENARIO: would fail if disconnect; stub; static | must observe: exit `1` with `BUT_STEER_FORCE_SERIALIZATION_FAULT` active; stderr `error.code` is a stable denial code (e.g. `perm.denied` or `branch.protected`); `error.message` length `>= 1`; `error.remediation_hint` present (non-empty) | must NOT observe: a successful action (deny->allow under fault; exit must `!= 0`); a dropped (`empty`/`absent`) code/message/remediation_hint; exit `0`

AC-6: DryRun under a serialization fault still fails closed (exit 1, existing fields, zero mutations)
  GIVEN: the committed `governed_loop_branch_protected` fixture; the real `but` CLI run with `BUT_STEER_FORCE_SERIALIZATION_FAULT` active AND the denied action invoked under `--dry-run` (§9.5 requires fail-closed at DryRun too; DryRun-no-bypass exercises different early-exit branches than the non-dry path)
  WHEN:  a denied action (e.g. implementer `pr merge 77 --dry-run`) is run with the steering-field serialization faulting under DryRun
  THEN:  the action is STILL denied with exit 1, stderr carries `code`/`message`/`remediation_hint`, AND zero new git objects/refs are mutated (DryRun persists nothing even under the serialization fault — fail-closed at the DryRun early-exit branch, never deny->allow)
  TEST_TIER: integration   VERIFICATION_SERVICE: governed_loop
  VERIFY: cargo test -p but governed_loop_dryrun_serialization_fault_failclosed
  SCENARIO: would fail if disconnect; stub; static | must observe: exit `1` under `--dry-run` + `BUT_STEER_FORCE_SERIALIZATION_FAULT`; stderr `error.code` stable (e.g. `perm.denied`) with `error.message` length `>= 1` and `error.remediation_hint` present; object_count after `==` object_count before (`0` new git objects persisted under the fault); refs/heads/main `ref_id` `==` its pre-run value (`0` refs mutated) | must NOT observe: exit `0` (a deny->allow flip under DryRun + fault); any new git object or advanced ref (`0` mutations — DryRun persists nothing even under the fault); a dropped (`empty`/`absent`) code/message/remediation_hint

AC-7: Whole-object-equality assertions audited/updated + positive new-field assertions added
  GIVEN: governed_loop.rs, commit_gate.rs, and merge_gate.rs (hand-assertion tests)
  WHEN:  the suites are audited for `assert_eq!` on a full `Denial`/`MergeGateError`/serialized blob and the new fields are wired
  THEN:  no whole-object-equality assertion that would break on the new fields survives (any found are converted to field-level asserts), AND the suites carry positive assertions confirming the new `class`/`held_permissions`/`authorized_actions`/`do_not` fields are present on actor-correctable denials — the whole suite compiles and passes against the STEER-001..005 fields
  TEST_TIER: integration   VERIFICATION_SERVICE: but-api
  VERIFY: cargo test -p but governed_loop && cargo test -p but-api commit_gate merge_gate
  SCENARIO: would fail if a left-over full-object assert_eq! on Denial/MergeGateError fails to compile/run against the new fields; stub; static | must observe: `cargo test` exit `0` for `governed_loop`, `commit_gate`, and `merge_gate` against the new fields; at least `1` positive assertion that `class` `== "actor_correctable"` and `authorized_actions` is non-empty on an actor-correctable denial | must NOT observe: a compile/run failure from a surviving whole-object equality on Denial/MergeGateError (a `removed`-field break); the new fields untested (`no` positive assertion present) (and no such entry/value present — the empty/start state must be excluded)

AC-8: Every offered command on an admin-write (governance) menu succeeds in its stated context
  GIVEN: the committed `governed_loop_branch_protected` fixture; a resolved principal lacking `administration:write` attempts an admin mutating action (e.g. `but perm grant`) so `governance_cli_error` denies `perm.denied` with an `authorized_actions` menu carrying the §5 admin-write affordance row (read/inspect configuration, request-config-change, discovery)
  WHEN:  each `authorized_actions[i].command` is parsed from the serialized governance denial JSON and REPLAYED in its stated context against a real subsequent `but` run
  THEN:  every offered command exits 0 (e.g. an inspect/list command without mutation when the principal can read governance state) OR hits its own legitimate non-`perm.denied` gate; NO offered command reproduces the ORIGINAL `perm.denied` at the denied admin ref — proving the admin-write actor-correctable menu is also non-lying
  TEST_TIER: integration   VERIFICATION_SERVICE: governed_loop
  VERIFY: cargo test -p but governed_loop_admin_write_menu_replay
  SCENARIO: would fail if lying-menu; stub; empty; static | must observe: at least `2` offered commands replayed off the admin-write menu (read/inspect or request-config-change + discovery, a non-degenerate menu); each offered command exits `0` or hits a non-governance/non-`perm.denied` gate | must NOT observe: any offered command reproducing the original `perm.denied` code at the denied admin ref (`0` such); an `empty` authorized_actions for the actor-correctable admin-write denial

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, happy_path): Every authorized_actions[i].command on a branch.protected menu replayed in its stated context exits 0 or hits its own legitimate non-perm.denied gate; none reproduces the original branch.protected at the denied ref
    VERIFY: cargo test -p but governed_loop_no_lying_menu_replay
- TC-2 (-> AC-1, structural): A replayed feature-branch commit advances the feature ref (ref_id changes) — the offered lateral move actually succeeds
    VERIFY: cargo test -p but governed_loop_no_lying_menu_replay
- TC-3 (-> AC-2, happy_path): Every authorized_actions[i].command on a gate.review_required (merge-gate) menu replayed in its stated context succeeds or hits its own legitimate non-gate.review_required gate; none reproduces the original gate.review_required
    VERIFY: cargo test -p but governed_loop_review_required_menu_replay
- TC-4 (-> AC-3, happy_path): Every authorized_actions[i].command on a perm.denied (missing-authority) commit-gate menu replayed in its stated context succeeds or hits its own legitimate non-perm.denied gate; none reproduces the original perm.denied at the denied commit ref
    VERIFY: cargo test -p but governed_loop_perm_denied_menu_replay
- TC-5 (-> AC-4, edge): A config advance between denial and replay yields exit 1 + parseable JSON + unchanged ref + no panic
    VERIFY: cargo test -p but governed_loop_concurrent_ref_advance_clean_redenial
- TC-6 (-> AC-5, error): BUT_STEER_FORCE_SERIALIZATION_FAULT (STEER-005's real seam) still emits code/message/remediation_hint + exit 1 (no deny->allow)
    VERIFY: cargo test -p but governed_loop_serialization_fault_failclosed
- TC-7 (-> AC-6, error): A denied action under --dry-run WITH BUT_STEER_FORCE_SERIALIZATION_FAULT still exits 1 with existing fields AND mutates 0 objects/refs (DryRun fail-closed under fault)
    VERIFY: cargo test -p but governed_loop_dryrun_serialization_fault_failclosed
- TC-8 (-> AC-7, structural): No whole-object-equality assert_eq! on Denial/MergeGateError survives the audit; governed_loop + commit_gate + merge_gate all pass with positive new-field assertions
    VERIFY: cargo test -p but governed_loop && cargo test -p but-api commit_gate merge_gate
- TC-9 (-> AC-8, happy_path): Every authorized_actions[i].command on an admin-write (governance) menu replayed in its stated context exits 0 or hits its own legitimate non-perm.denied gate; none reproduces the original perm.denied at the denied admin ref
    VERIFY: cargo test -p but governed_loop_admin_write_menu_replay

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-STEER-01
provides: the extended governed_loop no-lying-menu proof across ALL FOUR menu-bearing denial types: every authorized_actions[i].command replayed in its stated context succeeds (or hits its own legitimate non-perm.denied/non-gate.review_required gate) for branch.protected (commit gate), gate.review_required (merge gate), perm.denied/missing-authority (commit gate), and administration:write/admin-write (governance CLI via governance_cli_error); a concurrent-ref-advance regression case (clean re-denial); a serialization-fault regression case via STEER-005's BUT_STEER_FORCE_SERIALIZATION_FAULT seam (exit 1 with existing fields); a DryRun-under-serialization-fault fail-closed case (exit 1 + existing fields + 0 mutations); an audited test suite free of whole-object-equality assertions on Denial/MergeGateError, with positive field assertions for the new steering fields
consumes: STEER-003 gate-state-aware derivation (the menu being replayed); STEER-005 CLI serializers (the serialized authorized_actions[i].command strings the test reads + replays); the governed_loop fixture + CliErrorEnvelope reader (crates/but/tests/but/command/governed_loop.rs); but_testsupport invoke_bash/invoke_git for the concurrent-ref-advance; STEER-005 `BUT_STEER_FORCE_SERIALIZATION_FAULT` test/debug-gated best-effort-serialization fault seam (the EXACT seam STEER-009's serialization-fault + DryRun-fault cases activate)
boundary_contracts:
  - CAP-STEER-01: every offered command, replayed in its stated context against a REAL subsequent run, succeeds for that caller (or hits its own legitimate non-perm.denied/non-gate.review_required/non-admin-write-perm.denied gate) — proven across all four menu-bearing denial types (branch.protected via the commit gate, gate.review_required via the merge gate, perm.denied/missing-authority via the commit gate, and administration:write/admin-write via the governance CLI), not branch.protected alone. A concurrent ref advance between denial and replay yields a CLEAN re-denial (exit 1, valid JSON, no panic/inconsistent state). A serialization fault on the new fields (activated via STEER-005's BUT_STEER_FORCE_SERIALIZATION_FAULT seam) still denies with code/message/remediation_hint + exit 1 (fail-closed), including under --dry-run (which persists 0 objects/refs even under the fault).

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but/tests/but/command/governed_loop.rs (MODIFY) — extend the CliErrorEnvelope reader for the new fields; add the replay, concurrent-ref-advance, and serialization-fault test functions; do not weaken existing assertions
  - crates/but-api/tests/commit_gate.rs (MODIFY) — audit/convert any whole-object equality + add positive new-field assertions
  - crates/but-api/tests/merge_gate.rs (MODIFY) — audit/convert any whole-object equality + add positive new-field assertions
writeProhibited:
  - the gate deny/allow decision and any production gate/derivation source - this is a TEST task; do not change product code (if a real serialization-fault seam is needed, it is STEER-005's deliverable — FLAG if missing)
  - any insta snapshot mechanism / SNAPSHOTS=overwrite reliance - these are hand-assertion tests
  - the shipped honesty-grep patterns in invariant_build_gates.rs - not this task's surface
  - .spec/prds/governance/tasks/sprint-0[1-6]* - frozen
  - Any file not explicitly listed above

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
  - crates/but/tests/but/command/governed_loop.rs (lines 1-101, 263-347, 365-498): PRIMARY PATTERN — the harness to EXTEND: governed_loop_env fixture builder, the CliErrorEnvelope reader (:365-498, extend to read class/held_permissions/authorized_actions/do_not), assert_denial + assert_no_governance_denial + the forge-boundary helpers, and the env.but(...) replay style.
  - crates/but-api/tests/commit_gate.rs (lines 7-59, 946-993): the commit-gate hand-assertion style (denial.code / .message.contains, assert_commit_denied) — audit for whole-object equality (none found in grounding) and add positive class/authorized_actions assertions; reuse governed_repo().
  - crates/but-api/tests/merge_gate.rs (lines 19-130 (in src), test asserts at 24-50/140-180/440-460): merge-gate hand-assertion style (ref_id/count asserts + denial.message.contains, NO whole-object assert_eq found in grounding) — audit + add positive new-field assertions for the gate.review_required/perm.denied merge denials.
  - crates/but-api/src/commit/gate.rs (lines 55-78, 159-170): branch_protected + the authorize/protected-branch decision STEER-004 makes gate-state-aware — the source of the branch.protected menu being replayed; the menu must offer a feature-branch commit, never the protected-ref commit.
  - crates/but/src/command/legacy/forge/review.rs (lines 89-104, 246-259): the merge_gate_cli_error / review_gate_cli_error serializers STEER-005 extends — the JSON shape (incl. the new authorized_actions array) the replay test parses for command strings.
  - crates/but-api/src/legacy/merge_gate.rs (lines 40-130 (enforce_merge_gate + the gate.review_required MergeGateError construction), 113-124 (classify_error)): the merge-gate denial source for the AC-2 gate.review_required menu replay: authorize(Merge) then the review-requirement engine emit the gate.review_required MergeGateError whose authorized_actions are replayed; assert no replay reproduces gate.review_required at the merge target.
  - crates/but/tests/but/command/governed_loop.rs (lines 164-202 (governed_loop_dryrun_no_bypass), 555-557 (object_count)): the DryRun-no-bypass before/after pattern (ref_id + object_count captured before/after) to reuse for the AC-6 DryRun-under-fault case — assert 0 objects/refs mutated even with BUT_STEER_FORCE_SERIALIZATION_FAULT active.

--------------------------------------------------------------------------------
CODE PATTERN
--------------------------------------------------------------------------------
pattern: extend the real governed_loop CLI subprocess harness: parse the serialized authorized_actions, replay each command via env.but(...) against real gix fixtures, assert exit 0 / legitimate-gate / clean-re-denial; field-level hand-assertions only.
pattern_source: crates/but/tests/but/command/governed_loop.rs:11-101 (full-loop) + :365-498 (envelope reader + assert helpers)
anti_pattern: insta snapshots; treating a legitimate downstream gate (forge boundary) as a lying-menu failure; an in-process mocked envelope instead of the real serialized menu; a surviving whole-object assert_eq! on Denial/MergeGateError that silently breaks on new fields.
references: 02-uc-steer.md UC-STEER-06 AC-1; 04-e2e-testing-criteria.md T-STEER-024 (replay + concurrent-ref-advance + serialization-fault); 03-technical-requirements-delta.md §9.1 (no lying menu, ref-pin window) + §9.5 (fail-closed); 05-delta-replan.md D9 (assertion-based, whole-object-equality audit)
interaction_notes:
  - replays the menu STEER-003 derives + STEER-005 serializes through the real CLI
  - the serialization-fault + DryRun-fault cases consume STEER-005's BUT_STEER_FORCE_SERIALIZATION_FAULT test/debug seam — a real exercised seam, not an improvised fault; if absent, FLAG as a dependency gap rather than stubbing
  - replays all four menu-bearing denial types (branch.protected commit-gate, gate.review_required merge-gate, perm.denied/missing-authority commit-gate, and administration:write/admin-write governance via `governance_cli_error`) — T-STEER-024 requires every denial type's menu replayed

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
depends_on: STEER-003, STEER-005
blocks: STEER-010

CODING STANDARDS: crates/AGENTS.md, crates/but/AGENTS.md, RULES.md
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "description": "GIVEN a branch.protected denial with a derived menu, WHEN each offered command is replayed in its stated context, THEN every command succeeds or hits its own legitimate non-perm.denied gate and none reproduces the original branch.protected at the denied ref",
      "verify": "cargo test -p but governed_loop_no_lying_menu_replay",
      "primary": true
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "GIVEN a gate.review_required (merge-gate) denial menu, WHEN each offered command is replayed in its stated context, THEN every command succeeds or hits its own legitimate non-gate.review_required gate and none reproduces the original code",
      "verify": "cargo test -p but governed_loop_review_required_menu_replay"
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "GIVEN a perm.denied (missing-authority) commit-gate denial menu, WHEN each offered command is replayed in its stated context, THEN every command succeeds or hits its own legitimate non-perm.denied gate and none reproduces the original perm.denied at the denied commit ref",
      "verify": "cargo test -p but governed_loop_perm_denied_menu_replay"
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "description": "GIVEN a config advance between denial and replay, WHEN an offered command is replayed, THEN a clean re-denial (exit 1, parseable JSON, unchanged ref, no panic)",
      "verify": "cargo test -p but governed_loop_concurrent_ref_advance_clean_redenial"
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "description": "GIVEN BUT_STEER_FORCE_SERIALIZATION_FAULT active (STEER-005's real seam), WHEN a denied action runs, THEN it still denies with code/message/remediation_hint + exit 1",
      "verify": "cargo test -p but governed_loop_serialization_fault_failclosed"
    },
    {
      "id": "AC-6",
      "type": "acceptance_criterion",
      "description": "GIVEN a denied action under --dry-run WITH BUT_STEER_FORCE_SERIALIZATION_FAULT, WHEN it runs, THEN exit 1 + existing fields present + 0 objects/refs mutated (DryRun fail-closed under fault)",
      "verify": "cargo test -p but governed_loop_dryrun_serialization_fault_failclosed"
    },
    {
      "id": "AC-7",
      "type": "acceptance_criterion",
      "description": "GIVEN the three hand-assertion suites, WHEN audited, THEN no whole-object equality on Denial/MergeGateError survives and positive new-field assertions are added; all suites pass",
      "verify": "cargo test -p but governed_loop && cargo test -p but-api commit_gate merge_gate"
    },
    {
      "id": "AC-8",
      "type": "acceptance_criterion",
      "description": "GIVEN an admin-write (governance) denial menu (administration:write missing, e.g. `but perm grant`), WHEN each offered command is replayed in its stated context, THEN every command succeeds or hits its own legitimate non-perm.denied gate and none reproduces the original perm.denied at the denied admin ref",
      "verify": "cargo test -p but governed_loop_admin_write_menu_replay"
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "Every authorized_actions[i].command on a branch.protected menu replayed in its stated context exits 0 or hits its own legitimate non-perm.denied gate; none repr",
      "maps_to_ac": "AC-1",
      "verify": "cargo test -p but governed_loop_no_lying_menu_replay"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "A replayed feature-branch commit advances the feature ref (ref_id changes) \u2014 the offered lateral move actually succeeds",
      "maps_to_ac": "AC-1",
      "verify": "cargo test -p but governed_loop_no_lying_menu_replay"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "Every authorized_actions[i].command on a gate.review_required (merge-gate) menu replayed in its stated context succeeds or hits its own legitimate non-gate.revi",
      "maps_to_ac": "AC-2",
      "verify": "cargo test -p but governed_loop_review_required_menu_replay"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "Every authorized_actions[i].command on a perm.denied (missing-authority) commit-gate menu replayed in its stated context succeeds or hits its own legitimate non",
      "maps_to_ac": "AC-3",
      "verify": "cargo test -p but governed_loop_perm_denied_menu_replay"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "A config advance between denial and replay yields exit 1 + parseable JSON + unchanged ref + no panic",
      "maps_to_ac": "AC-4",
      "verify": "cargo test -p but governed_loop_concurrent_ref_advance_clean_redenial"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "description": "BUT_STEER_FORCE_SERIALIZATION_FAULT (STEER-005's real seam) still emits code/message/remediation_hint + exit 1 (no deny->allow)",
      "maps_to_ac": "AC-5",
      "verify": "cargo test -p but governed_loop_serialization_fault_failclosed"
    },
    {
      "id": "TC-7",
      "type": "test_criterion",
      "description": "A denied action under --dry-run WITH BUT_STEER_FORCE_SERIALIZATION_FAULT still exits 1 with existing fields AND mutates 0 objects/refs (DryRun fail-closed under",
      "maps_to_ac": "AC-6",
      "verify": "cargo test -p but governed_loop_dryrun_serialization_fault_failclosed"
    },
    {
      "id": "TC-8",
      "type": "test_criterion",
      "description": "No whole-object-equality assert_eq! on Denial/MergeGateError survives the audit; governed_loop + commit_gate + merge_gate all pass with positive new-field asser",
      "maps_to_ac": "AC-7",
      "verify": "cargo test -p but governed_loop && cargo test -p but-api commit_gate merge_gate"
    },
    {
      "id": "TC-9",
      "type": "test_criterion",
      "description": "Every authorized_actions[i].command on an admin-write (governance) menu replayed in its stated context exits 0 or hits its own legitimate non-perm.denied gate; none reproduces the original perm.denied at the denied admin ref",
      "maps_to_ac": "AC-8",
      "verify": "cargo test -p but governed_loop_admin_write_menu_replay"
    }
  ]
}
-->
