# GATES-005: Review-requirement evaluator — self-approval exclusion + stale-approval-@head dismissal + `no_approval`/`approval_stale_at_head` unmet discriminator

## What this does

Implements the pure review-requirement evaluator that makes the merge gate SOUND — the function GATES-003 calls. (1) Self-approval exclusion: when `require_distinct_from_author` is set, a verdict whose `principal_id == the change author` is NOT counted. (2) Stale-@head dismissal: an approval recorded at a prior `head_oid` is dismissed once the head advances. (3) The `unmet[]` payload distinguishes `no_approval` from `approval_stale_at_head` so an orchestrator can re-route the right reviewer. Proven through the real merge gate (real but-api + real git + real but-db).

## Why

Sprint 01b · PRD UC-GATES-02 · capabilities CAP-AUTHZ-01, CAP-CONFIG-01. A merge gate that counts any approving review is not a gate; this refinement is what makes the review requirement sound. It is the evaluator GATES-003's merge gate delegates to.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-api merge_gate_self_and_stale_dismissed` (integration). Full gate set in the spec below.

## Scope

- `crates/but-api/src/legacy/review_requirement.rs` (NEW) — the pure evaluator (`evaluate(req, verdicts, head_oid, author, cfg) -> Result<(), ReviewUnmet>`): self-approval exclusion, stale-@head dismissal, per-group satisfaction, the `no_approval`/`approval_stale_at_head` unmet discriminator. OWNS the evaluator refinements
- `crates/but-api/src/legacy/merge_gate.rs` (MODIFY — wiring only) — wire the call to `review_requirement::evaluate(...)` and map `Err(ReviewUnmet)` to the `gate.review_required` Denial carrying `unmet[]`
- `crates/but-api/tests/merge_gate.rs` (MODIFY) OR `crates/but-api/tests/review_requirement_gate.rs` (NEW) — integration tests driving self/stale cases THROUGH the real merge gate + real git + real but-db
- `crates/but/tests/but/command/merge_gate.rs` (MODIFY — optional) — CLI snapbox for the stale/self denial surfacing the unmet discriminator

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: GATES-005 - Review-requirement evaluator: self-approval exclusion + stale-approval-@head dismissal + no_approval/approval_stale_at_head unmet discriminator
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Complete
PRIORITY:   P0
EFFORT:     M  (150 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-GATES-02
CAPABILITIES: CAP-AUTHZ-01, CAP-CONFIG-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-api merge_gate_self_and_stale_dismissed
  check: cargo check -p but-api --all-targets
  lint:  cargo clippy -p but-api --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Integration tests are green through the real merge gate + real git + real but-db: a self-approval (author's own verdict) is excluded when require_distinct_from_author is set, so the requirement is UNMET (gate.review_required with unmet reason no_approval); an approval recorded at head H1 is dismissed once the head advances to H2 (gate.review_required with unmet reason approval_stale_at_head), and a fresh re-approval at H2 lets the merge proceed; a distinct, current-head, non-author approval satisfies the requirement (positive/non-degenerate control — the dismissals do not over-reject a legitimate approval); and the evaluator is a pure function (no I/O — the merge_gate.rs wrapper supplies the verdicts/head/author/cfg) with no role label in its branching.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST self-exclude the author: when require_distinct_from_author is true on the target-ref gate, a verdict whose principal_id equals the change author's principal id is NOT counted (UC-GATES-02 AC-3; T-GATES-010). The author identity is the change author, resolved from the review/branch under merge — never an agent-supplied claim.
- [MUST] MUST dismiss stale approvals: an approving verdict recorded at a head_oid that is NOT the current head oid is dismissed; only verdicts at the CURRENT head count (UC-GATES-02 AC-4; T-GATES-011). After the head advances, the reviewer must re-approve at the new head.
- [MUST] MUST populate the unmet[] payload so it DISTINGUISHES no_approval (no qualifying verdict exists at all) from approval_stale_at_head (a qualifying approval exists but at a prior head) — load-bearing so an orchestrator re-routes the right reviewer (UC-GATES-02 AC-4).
- [MUST] MUST keep the evaluator a PURE function over its inputs (the requirement, the verdict rows, the current head oid, the change author, and the GovConfig for group resolution) — no I/O; GATES-003 does the DB query + repo reads and passes the results in (mirrors AUTHZ-003's pure authorize).
- [NEVER] NEVER count a verdict whose head_oid != current head as satisfying (stale must not pass), and NEVER count the author's own verdict when distinct-from-author is set (self must not pass) — the two soundness holes this task closes.
- [NEVER] NEVER add a test asserting the forgeable direct-DB-write to local_review_verdicts is blocked (R6 accepted-leak; the evaluator trusts the store's honest contents).
- [MUST] MUST seed every approving verdict THROUGH the governed `but review approve` CLI action (the GATES-004 verb), NOT via a direct `db.local_review_verdicts_mut().insert(...)` — a direct insert is exactly the forgeable R6 path the gate is not supposed to exercise. The self-approval fixture therefore grants the AUTHOR `reviews:write` (so it CAN submit a governed review) and relies on `require_distinct_from_author` to EXCLUDE that verdict; the stale case re-submits via `but review approve` at the new head H2.
- [MUST] MUST treat the merge GATE DECISION as the locally-provable surface for POSITIVE cases (gate-boundary re-scope, user decision): `merge_review` is forge-bound (errors on a bare local repo) and has no `but pr merge` CLI verb, so a "merge proceeds" assertion proves the gate PERMITS (the evaluator returns Ok → NO `gate.review_required` raised) and execution reaches the forge call past the gate — NOT that the change lands on the remote trunk (forge completion is structural/out-of-local-scope). DENIAL cases (self/stale → `gate.review_required`) are fully locally provable.
- [STRICTLY] STRICTLY use CONCRETE oids and principal ids in the integration cases (approve at head H1, advance to H2, assert approval_stale_at_head), and assert the EXACT unmet discriminator value — not a generic 'blocked'.
- [STRICTLY] STRICTLY confine this task to the evaluator refinements — GATES-003 owns the merge-gate wrapper file (merge_gate.rs) + the basic min_approvals/at-head plumbing; GATES-005 owns the requirement evaluator in the sibling review_requirement.rs. The split is non-overlapping: GATES-003 CALLS review_requirement::evaluate; GATES-005 implements it. Do not edit the merge-gate wrapper beyond wiring the call.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [x] AC-1 [PRIMARY]: self-approval not counted when require_distinct_from_author set → requirement unmet, gate.review_required unmet reason no_approval
- [x] AC-2: approve@H1 → advance to H2 → blocked with approval_stale_at_head; re-approve@H2 → proceeds
- [x] AC-3: a distinct, current-head, non-author approval satisfies the requirement → merge proceeds (positive/non-degenerate control)
- [x] AC-4: the evaluator is pure (no I/O); merge_gate.rs calls review_requirement::evaluate; no role label in the evaluator (build-gate)
- [x] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: Self-approval is not counted when require_distinct_from_author is set → requirement unmet [PRIMARY]
  GIVEN: fixture `merge_self_approval` (gate min_approvals=1 require_distinct_from_author=true; change author = impl-author), BUT_AGENT_HANDLE=maint (merge), with the ONLY approving verdict at head authored by impl-author itself (a self-approval via governed `but review`)
  WHEN:  a merge is attempted by maint through the gate (which calls this evaluator)
  THEN:  the self-approval is excluded, the requirement is UNMET, merge blocked gate.review_required whose unmet[] reports no_approval (the distinct-from-author slot has no qualifying approval), exit 1
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api merge gate + real git + real but-db
  VERIFY: cargo test -p but-api merge_gate_self_and_stale_dismissed
  SCENARIO: NEGATIVE_CONTROL would fail if the author's own verdict is counted so the self-approval satisfies the requirement; require_distinct_from_author is ignored; the denial omits the unmet[] discriminator; the change author is taken from an agent-supplied claim.

AC-2: Approval @H1 is dismissed after head advances to H2 → blocked until re-approval @H2
  GIVEN: fixture `merge_stale_at_head` (gate min_approvals=1 require_distinct_from_author=true; distinct reviewer-b approved at head H1), BUT_AGENT_HANDLE=maint
  WHEN:  the feature head advances H1→H2, then a merge is attempted by maint; then reviewer-b re-approves at H2 and the merge is re-attempted
  THEN:  the H1 approval is dismissed (stale at H2), merge blocked gate.review_required whose unmet[] reports approval_stale_at_head (exit 1); after the H2 re-approval the merge proceeds (Ok)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api merge gate + real git + real but-db
  VERIFY: cargo test -p but-api merge_gate_self_and_stale_dismissed
  SCENARIO: NEGATIVE_CONTROL would fail if the H1 approval still counts after head advances; the unmet reason is no_approval rather than approval_stale_at_head; head_oid is ignored; the re-approval @H2 does not allow the merge.

AC-3: A distinct, current-head approval satisfies the requirement (positive control: neither self nor stale)
  GIVEN: fixture `merge_stale_at_head` with a DISTINCT reviewer-b approving at the CURRENT head, BUT_AGENT_HANDLE=maint
  WHEN:  a merge is attempted by maint
  THEN:  the evaluator counts the distinct current-head approval, the requirement is satisfied, the merge proceeds (Ok) — the self/stale dismissals do not over-reject a legitimate approval
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api merge gate + real git + real but-db
  VERIFY: cargo test -p but-api merge_gate_distinct_current_head_satisfies
  SCENARIO: NEGATIVE_CONTROL would fail if a distinct current-head approval is over-dismissed; the merge is blocked despite a valid distinct approval @head; the evaluator is a stub that always returns Err (always blocks).

AC-4: The evaluator is a pure function the merge gate delegates to (no I/O; non-overlapping ownership) [build-gate]
  GIVEN: the merge-gate wrapper (GATES-003, merge_gate.rs) performs the DB query + repo reads and passes the verdict rows, current head oid, change author, GovConfig into review_requirement::evaluate
  WHEN:  the code is structurally inspected
  THEN:  review_requirement.rs performs NO I/O (no DB query, no repo/file read, no .await) — pure over its inputs; merge_gate.rs CALLS review_requirement::evaluate; no role label appears in the evaluator
  TEST_TIER: unit (build-gate)   VERIFICATION_SERVICE: source grep + compile   UNIT_TEST_JUSTIFIED: pure structural invariants (no-I/O, the call-delegation split, no role-name leakage) verified by grep/compile with zero runtime I/O; the behavioral soundness is proven by AC-1..3 integration cases
  VERIFY: ! grep -rEn 'rusqlite|local_review_verdicts|std::fs::read|workdir|repo\.find_reference|\.await' crates/but-api/src/legacy/review_requirement.rs && grep -qE 'review_requirement::evaluate' crates/but-api/src/legacy/merge_gate.rs

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, error): self-approval excluded when require_distinct_from_author set → requirement unmet, gate.review_required with unmet reason no_approval (T-GATES-010)
    VERIFY: cargo test -p but-api merge_gate_self_and_stale_dismissed
- TC-2 (-> AC-2, edge): approve@H1 → advance to H2 → blocked with approval_stale_at_head; re-approve@H2 → proceeds (T-GATES-011)
    VERIFY: cargo test -p but-api merge_gate_self_and_stale_dismissed
- TC-3 (-> AC-3, happy_path): a distinct, current-head, non-author approval satisfies the requirement → merge proceeds (positive/non-degenerate control)
    VERIFY: cargo test -p but-api merge_gate_distinct_current_head_satisfies
- TC-4 (-> AC-4, structural): the evaluator is pure (no I/O); merge_gate.rs calls review_requirement::evaluate; no role label in the evaluator
    VERIFY: cargo check -p but-api --all-targets
- TC-5 (-> AC-2, edge): the unmet[] discriminator distinguishes no_approval from approval_stale_at_head so an orchestrator can re-route the right reviewer (UC-GATES-02 AC-4)
    VERIFY: cargo test -p but-api merge_gate_self_and_stale_dismissed

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01, CAP-CONFIG-01
provides: the distinct-from-author + @head review-requirement evaluator (review_requirement::evaluate) with self-approval exclusion, stale-@head dismissal, per-group satisfaction, and the no_approval/approval_stale_at_head unmet discriminator — the function the merge gate (GATES-003) delegates to
consumes: the local_review_verdicts verdict row type incl. head_oid + principal_id (GATES-002); the merge gate's invocation + Denial mapping (GATES-003); but_authz::PrincipalId/GroupName + the GovConfig group resolution (AUTHZ-001/002)
boundary_contracts:
  - CAP-AUTHZ-01: the review requirement is evaluated as part of the merge gate's read-only enforcement (it runs even under DryRun via GATES-003).
  - CAP-CONFIG-01: the requirement + group membership the evaluator judges against are the target-ref-pinned config GATES-003 loads — a head that drops the requirement cannot weaken the gate.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-api/src/legacy/review_requirement.rs (NEW) — the pure review-requirement evaluator (OWNS the evaluator refinements: self/stale/per-group + unmet discriminator)
  - crates/but-api/src/legacy/merge_gate.rs (MODIFY — wiring only) — wire the call to review_requirement::evaluate(...) and map Err(ReviewUnmet) to the gate.review_required Denial carrying unmet[]. Do NOT implement self/stale logic here
  - crates/but-api/tests/merge_gate.rs (MODIFY — add the self/stale cases) OR crates/but-api/tests/review_requirement_gate.rs (NEW) — integration tests driving the cases THROUGH the real merge gate + real git + real but-db
  - crates/but/tests/but/command/merge_gate.rs (MODIFY — optional) — CLI snapbox for the stale/self denial surfacing the unmet discriminator
writeProhibited:
  - crates/but-authz/** — CONSUME PrincipalId/GroupName/GovConfig/AuthoritySet; do NOT modify the primitive
  - crates/but-api/src/legacy/forge.rs — the forge entry-point gate wiring is OWNED by GATES-003
  - the merge-gate wrapper's principal resolution / authorize(merge) / DB query / repo reads in merge_gate.rs — those are GATES-003's; restrict edits here to wiring the evaluator call + mapping the ReviewUnmet result
  - crates/but-db/src/table/local_review_verdicts.rs — OWNED by GATES-002; consume the verdict row type, do not redefine it
  - crates/but-error/src/lib.rs — do not add Code variants; the gate.review_required code is a but-authz/merge-gate-owned &'static str (GATES-003 maps it)
  - any gitbutler-* crate beyond what the action boundary strictly requires (crates/AGENTS.md)
  - Any file not listed in write_allowed

--------------------------------------------------------------------------------
OUT OF SCOPE
--------------------------------------------------------------------------------
  - The per-required-group strictness matrix (T-GATES-012 / T-LOOP-008 / T-LOOP-009 — only-one-group-blocked) DEFERRED to Sprint 04. This task's evaluator MUST correctly handle group requirements when GATES-003 passes them, but the dedicated only-one-blocked strictness proof is Sprint 04.
  - The merge-gate wrapper file (merge_gate.rs), BUT_AGENT_HANDLE→Principal resolution, authorize(merge), the DB query, and the repo/target-ref reads are OWNED by GATES-003.
  - The local_review_verdicts table + migration + query API are OWNED by GATES-002.
  - No test asserts the forgeable direct-DB-write to local_review_verdicts is blocked (R6 accepted-leak).

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. .spec/prds/governance/06-uc-gates.md (36-52)
   Focus: UC-GATES-02 AC-3 (self-approval not counted when require_distinct_from_author set) + AC-4 (stale approval dismissed after head advances; the unmet payload distinguishes no_approval from approval_stale_at_head).
2. .spec/prds/governance/11-e2e-testing-criteria.md (117-118)
   Focus: T-GATES-010 (self-approval not counted → requirement unmet) and T-GATES-011 (approve@H1 → advance to H2 → blocked until re-approval@H2) — both integration through the merge gate.
3. .spec/prds/governance/10-technical-requirements/03-data-schema.md (79-94)
   Focus: the local_review_verdicts fields — principal_id (for distinct-from-author) and head_oid (LOAD-BEARING for stale dismissal). The evaluator's two checks key off these.
4. .spec/prds/governance/tasks/sprint-01b-governed-loop-reference-flow/GATES-003-merge-gate.md (full)
   Focus: SIBLING TASK that CALLS this evaluator — GATES-003 owns merge_gate.rs (wrapper, DB query, repo reads, authorize(merge), the gate.review_required Denial mapping) and delegates to review_requirement::evaluate. Align the function signature + the ReviewUnmet/unmet[] payload shape with GATES-003's consumption.
5. crates/but-authz/src/principal.rs (1-71)
   Focus: CONSUME-ONLY — PrincipalId (the author + reviewer identity type) and GroupName. The evaluator compares verdict principal_id against the change author's PrincipalId and resolves group membership via the GovConfig (AUTHZ-002).
6. crates/but-db/src/table/forge_reviews.rs (44-141)
   Focus: PATTERN for the verdict row type GATES-002 provides (target/principal_id/verdict/head_oid/created_at). The evaluator receives a &[ReviewVerdict] from GATES-003 — it does not query the DB itself.
7. crates/but-api/src/legacy/forge.rs (435-495)
   Focus: the merge entry points GATES-003 gates; this evaluator is invoked inside that gate. The head oid + change author are resolved by GATES-003 from ctx.repo.get() + the review under merge and passed into the evaluator.
8. crates/but-testsupport/src/lib.rs (71-97,432-441)
   Focus: writable_scenario + invoke_bash to seed a real repo, commit config at refs/heads/main, branch to feat, and ADVANCE the feat head (H1→H2) with a second commit — how the stale-at-head case is driven. Capture concrete H1/H2 oids via repo.find_reference("refs/heads/feat")?.peel_to_id().

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- Self + stale dismissal integration tests pass: `cargo test -p but-api merge_gate_self_and_stale_dismissed`  -> Exit 0; AC-1 (self → no_approval) + AC-2 (stale → approval_stale_at_head, then re-approval proceeds) green
- Positive control passes: `cargo test -p but-api merge_gate_distinct_current_head_satisfies`  -> Exit 0; a legitimate distinct @head approval is NOT over-rejected
- The evaluator is pure (no I/O): `! grep -rEn 'rusqlite|local_review_verdicts|std::fs::read|workdir|repo\.find_reference|\.await' crates/but-api/src/legacy/review_requirement.rs`  -> No matches
- Ownership split — the wrapper delegates to the evaluator: `grep -qE 'review_requirement::evaluate' crates/but-api/src/legacy/merge_gate.rs`  -> Match
- No role-name in the evaluator: `! grep -rEni 'implementer|reviewer|maintainer' crates/but-api/src/legacy/review_requirement.rs`  -> No matches (T-LOOP-005 family)
- Crate compiles: `cargo check -p but-api --all-targets`  -> Exit 0
- Clippy clean: `cargo clippy -p but-api --all-targets`  -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: Pure-function requirement evaluator with a structured unmet-reason discriminator — evaluate(req, verdicts, head_oid, author, cfg) -> Result<(), ReviewUnmet> filters verdicts (drop stale-at-head, drop author-self when distinct-required), counts distinct approvers + per-group coverage, and on shortfall emits unmet[] entries each tagged no_approval vs approval_stale_at_head. The merge gate (GATES-003) provides the I/O inputs and maps the Err to gate.review_required.
pattern_source: crates/but-authz/src/principal.rs (PrincipalId comparison for distinct-from-author) + 03-data-schema.md:79-94 (head_oid load-bearing for stale dismissal) + AUTHZ-003's pure authorize (the same I/O-free-evaluator discipline)
anti_pattern: Counting a verdict whose head_oid != current head (stale passes — soundness hole); counting the author's own verdict when distinct-from-author is set (self passes — soundness hole); collapsing the unmet reason to a single generic 'blocked' (loses the re-route signal); doing DB/repo I/O inside the evaluator (purity violation); re-implementing the merge-gate wrapper here (overlaps GATES-003); branching on a role label; or a degenerate evaluator that always returns Err (over-strict — caught by AC-3).

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — Implements the pure review-requirement evaluator that makes the merge gate sound — self-approval exclusion (PrincipalId equality against the change author) and stale-@head dismissal (head_oid equality against the current head), with a structured no_approval/approval_stale_at_head unmet discriminator. Owns the pure-function discipline (no I/O — inputs passed in by GATES-003's wrapper), the non-overlapping ownership split with GATES-003, and integration TDD that seeds concrete oids/principals and advances the head against real git + real but-db.
reviewer: rust-reviewer
coding_standards: crates/AGENTS.md, crates/WORKSPACE_MODEL.md, crates/but/AGENTS.md, crates/but-api/src/legacy/forge.rs

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: GATES-002, GATES-003
Blocks:     LOOP-001
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "GATES-005",
  "proposed_by": "rust-planner",
  "verification_policy": { "requires_tests": true, "requires_red_evidence": true, "requires_seeded_evidence": true },
  "fixtures": {
    "merge_self_approval": {
      "description": "A real git repo (but-testsupport writable_scenario) whose target ref main has committed permissions.toml (impl-author=[contents:write,pull_requests:write,reviews:write] — reviews:write so the author CAN submit a governed review that is then EXCLUDED as the author's; reviewer-b=[reviews:write]; maint=[merge]) and gates.toml ([[gate]] branch=main type=review min_approvals=1 require_distinct_from_author=true), with an open review on feat authored by impl-author and a single approving verdict at the current head submitted by impl-author via the governed `but review approve` action (a self-approval).",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"governance-base\");",
        "invoke_bash: write permissions.toml (impl-author=[contents:write,pull_requests:write,reviews:write], reviewer-b=[reviews:write], maint=[merge]) + gates.toml (main protected, gate min_approvals=1 require_distinct_from_author=true); git add -A && git commit at refs/heads/main; git checkout -b feat; commit a change as impl-author (the change author)",
        "open a governed review on feat; record the self-approval via the governed CLI: `BUT_AGENT_HANDLE=impl-author but review approve feat` (NOT a direct DB insert) — this writes a local_review_verdicts row principal_id=impl-author, verdict=approved, head_oid=<current feat head>"
      ]
    },
    "merge_stale_at_head": {
      "description": "Same shape as merge_self_approval but the open review carries a DISTINCT approving verdict from reviewer-b (not the author) recorded at head oid H1; the test then advances the feat head to H2 to prove the H1 approval is dismissed, and (positive control AC-3) can instead seed a distinct approval at the current head.",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"governance-base\");",
        "invoke_bash: write permissions.toml (impl-author=[contents:write,pull_requests:write], reviewer-b=[reviews:write], maint=[merge]) + gates.toml (gate min_approvals=1 require_distinct_from_author=true); git add -A && git commit at refs/heads/main; git checkout -b feat; commit a change → record the head oid H1",
        "open a governed review on feat; record reviewer-b's distinct approval via the governed CLI: `BUT_AGENT_HANDLE=reviewer-b but review approve feat` at head H1 (writes a local_review_verdicts row principal_id=reviewer-b, verdict=approved, head_oid=H1) — NOT a direct DB insert",
        "for the stale case: invoke_bash commit a second change on feat so head advances H1 → H2 (the H1 verdict is now stale); for the re-approval/positive case: re-run `BUT_AGENT_HANDLE=reviewer-b but review approve feat` at the CURRENT head H2 (a fresh governed approval)"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN merge_self_approval (only approval @head is the author's self-approval; require_distinct_from_author=true) WHEN maint attempts the merge THEN the self-approval is excluded, requirement UNMET, merge blocked gate.review_required whose unmet[] reports no_approval, exit 1",
      "verify": "cargo test -p but-api merge_gate_self_and_stale_dismissed",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api merge gate + real git + real but-db",
        "negative_control": { "would_fail_if": [
          "the author's own verdict is counted so the self-approval satisfies the requirement and the merge proceeds",
          "require_distinct_from_author is ignored",
          "the denial is not gate.review_required or omits the unmet[] discriminator (empty unmet)",
          "the change author is taken from an agent-supplied claim rather than the change/review under merge"
        ] },
        "evidence": { "artifact_type": "stdout", "required_capture": true },
        "cases": [
          {
            "start_ref": "merge_self_approval",
            "action": { "actor": "cli_user", "steps": ["BUT_AGENT_HANDLE=impl-author: `but review approve feat` (the author submits a governed approving review — holds reviews:write; this is the self-approval, recorded via the governed action NOT a direct insert)", "BUT_AGENT_HANDLE=maint: invoke the governed merge action (cite T-GATES-010)"] },
            "end_state": {
              "must_observe": ["`error.code == \"gate.review_required\"`", "the `unmet[]` payload reports reason `no_approval` (the self-approval was excluded, leaving 0 qualifying distinct approvals)", "process exits `1`", "the review is NOT merged (trunk/main HEAD sha `==` base, 0 advance)"],
              "must_not_observe": ["merge proceeded", "exit `0`", "the self-approval counted as satisfying", "`approval_stale_at_head` (the verdict is at the current head — excluded for being the author's, not for being stale)"]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN merge_stale_at_head (reviewer-b approved at H1) WHEN the head advances H1→H2 and maint attempts the merge, then reviewer-b re-approves at H2 THEN the H1 approval is dismissed → gate.review_required unmet reason approval_stale_at_head (exit 1); after the H2 re-approval the merge proceeds (Ok)",
      "verify": "cargo test -p but-api merge_gate_self_and_stale_dismissed",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api merge gate + real git + real but-db",
        "negative_control": { "would_fail_if": [
          "the H1 approval still counts after the head advances to H2 so the merge proceeds with a stale approval",
          "the unmet reason is reported as no_approval rather than approval_stale_at_head (losing the re-route signal)",
          "head_oid is ignored in the evaluation (a no-op head check)",
          "the re-approval @H2 does not allow the merge (evaluator wrongly stays stale)"
        ] },
        "evidence": { "artifact_type": "stdout", "required_capture": true },
        "cases": [
          {
            "start_ref": "merge_stale_at_head",
            "action": { "actor": "cli_user", "steps": ["BUT_AGENT_HANDLE=reviewer-b: `but review approve feat` at head H1 (governed approval, NOT a direct insert)", "invoke_bash: commit a new change on feat so the head advances H1 → H2", "BUT_AGENT_HANDLE=maint: invoke the governed merge action (cite T-GATES-011)"] },
            "end_state": {
              "must_observe": ["`error.code == \"gate.review_required\"`", "the `unmet[]` payload reports reason `approval_stale_at_head` (a qualifying approval existed, but at the prior head H1)", "process exits `1`", "the review is NOT merged (trunk/main HEAD sha `==` base, 0 advance)"],
              "must_not_observe": ["merge proceeded", "exit `0`", "the H1 approval counted at H2", "`no_approval` (the discriminator must reflect that a stale approval exists, not that none ever did)"]
            }
          },
          {
            "start_ref": "merge_stale_at_head",
            "action": { "actor": "cli_user", "steps": ["BUT_AGENT_HANDLE=reviewer-b: `but review approve feat` at head H2 (fresh governed re-approval)", "BUT_AGENT_HANDLE=maint: invoke the governed merge action"] },
            "end_state": {
              "must_observe": ["the gate PERMITS the merge — the re-approval @H2 satisfies the requirement: the output contains NO `error.code == \"gate.review_required\"` and NO `perm.denied` (0 governance denials)", "execution reaches the governed `merge_review` body past the gate (any failure is a forge/remote error, NOT a governance Denial)"],
              "must_not_observe": ["`error.code == \"gate.review_required\"` raised when the fresh approval is present @H2", "the re-approval @H2 wrongly treated as stale (0 denials expected)", "a governance Denial blocks the merge"]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN merge_stale_at_head with a DISTINCT reviewer-b approving at the CURRENT head (not author, not stale) WHEN maint attempts the merge THEN the evaluator counts it, the requirement is satisfied, the merge proceeds (Ok) — the self/stale dismissals do not over-reject a legitimate approval",
      "verify": "cargo test -p but-api merge_gate_distinct_current_head_satisfies",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api merge gate + real git + real but-db",
        "negative_control": { "would_fail_if": [
          "a distinct current-head approval is over-dismissed (the evaluator rejects a legitimate approval — degenerate over-strictness)",
          "the merge is blocked despite a valid distinct approval @head",
          "the evaluator is a stub that only ever returns Err (always blocks)"
        ] },
        "evidence": { "artifact_type": "stdout", "required_capture": true },
        "cases": [
          {
            "start_ref": "merge_stale_at_head",
            "action": { "actor": "cli_user", "steps": ["BUT_AGENT_HANDLE=reviewer-b: `but review approve feat` at the CURRENT head (governed; reviewer-b != the change author)", "BUT_AGENT_HANDLE=maint: invoke the governed merge action — positive/non-degenerate control"] },
            "end_state": {
              "must_observe": ["the gate PERMITS the merge — the evaluator returns `Ok` for the distinct current-head approval: the output contains NO `error.code == \"gate.review_required\"` and NO `perm.denied` (0 governance denials)", "execution reaches the governed `merge_review` body past the gate (any failure is a forge/remote error, NOT a governance Denial)"],
              "must_not_observe": ["`error.code == \"gate.review_required\"` raised for a valid distinct current-head approval", "a legitimate distinct current-head approval over-rejected (0 denials expected)", "a governance Denial blocks the merge"]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the wrapper passes verdicts/head/author/cfg into review_requirement::evaluate WHEN the code is structurally inspected THEN review_requirement.rs performs NO I/O (pure), merge_gate.rs calls review_requirement::evaluate, and no role label appears in the evaluator",
      "verify": "! grep -rEn 'rusqlite|local_review_verdicts|std::fs::read|workdir|repo\\.find_reference|\\.await' crates/but-api/src/legacy/review_requirement.rs && grep -qE 'review_requirement::evaluate' crates/but-api/src/legacy/merge_gate.rs",
      "maps_to_ac": null,
      "scenario": {
        "tier": "holdout",
        "primary": false,
        "test_tier": "unit",
        "unit_test_justified": "pure structural invariants (no-I/O in the evaluator, the call-delegation split, no role-name leakage) verified by grep/compile with zero runtime I/O; the behavioral soundness of the evaluator is proven by the AC-1..3 integration cases",
        "verification_service": "source grep + compile (build-gate, no runtime I/O)",
        "negative_control": { "would_fail_if": [
          "review_requirement.rs reads the DB / repo / working tree directly (I/O leaked into the evaluator)",
          "the self/stale logic is implemented inside merge_gate.rs instead of the evaluator (ownership overlap with GATES-003)",
          "a role label (implementer/reviewer/maintainer) appears in the evaluator's branching",
          "the evaluator is a no-op stub that ignores its passed-in inputs (static pass-through)"
        ] },
        "evidence": { "artifact_type": "stdout", "required_capture": true },
        "cases": [
          {
            "start_ref": "merge_stale_at_head",
            "action": { "actor": "ci", "steps": ["grep the evaluator for I/O and role names; grep the merge-gate wrapper to confirm it calls review_requirement::evaluate"] },
            "end_state": {
              "must_observe": ["`! grep -rEn 'rusqlite|local_review_verdicts|std::fs::read|workdir|repo\\.find_reference|\\.await' crates/but-api/src/legacy/review_requirement.rs` → 0 matches (pure, no I/O)", "`grep -qE 'review_requirement::evaluate' crates/but-api/src/legacy/merge_gate.rs` → 1+ match (the wrapper delegates)", "`! grep -rEni 'implementer|reviewer|maintainer' crates/but-api/src/legacy/review_requirement.rs` → 0 matches"],
              "must_not_observe": ["`review_requirement.rs` contains `rusqlite`/a DB query/`.await` (I/O leaked into the evaluator)", "`merge_gate.rs` does NOT call `review_requirement::evaluate` (0 delegation — split violated)", "a role label (`implementer`/`reviewer`/`maintainer`) appears in the evaluator"]
            }
          }
        ]
      }
    },
    { "id": "TC-1", "type": "test_criterion", "description": "self-approval excluded when require_distinct_from_author set → requirement unmet, gate.review_required unmet reason no_approval (T-GATES-010)", "verify": "cargo test -p but-api merge_gate_self_and_stale_dismissed", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "approve@H1 → advance to H2 → blocked with approval_stale_at_head; re-approve@H2 → proceeds (T-GATES-011)", "verify": "cargo test -p but-api merge_gate_self_and_stale_dismissed", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "a distinct, current-head, non-author approval satisfies the requirement → merge proceeds (positive/non-degenerate control)", "verify": "cargo test -p but-api merge_gate_distinct_current_head_satisfies", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "the evaluator is pure (no I/O); merge_gate.rs calls review_requirement::evaluate; no role label in the evaluator", "verify": "cargo check -p but-api --all-targets", "maps_to_ac": "AC-4" },
    { "id": "TC-5", "type": "test_criterion", "description": "the unmet[] discriminator distinguishes no_approval from approval_stale_at_head so an orchestrator can re-route the right reviewer (UC-GATES-02 AC-4)", "verify": "cargo test -p but-api merge_gate_self_and_stale_dismissed", "maps_to_ac": "AC-2" }
  ]
}
-->
</details>
