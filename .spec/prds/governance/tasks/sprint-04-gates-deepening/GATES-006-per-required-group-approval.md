# GATES-006: Per-required-group approval evaluation — only-one-group-blocked strictness matrix (two-group AI + human model) + no-human-vs-AI-branch build-gate

## What this does

Proves the merge gate requires a distinct approval from **each** group named in `require_approval_from_group` at the current head — as standalone strictness cases the walking skeleton deferred. With a two-group requirement `["code-reviewers", "maintainers"]`, an AI-only (`code-reviewers`) approval blocks (the human hasn't owned it), a human-only (`maintainers`) approval blocks (the AI code-level pass is also required), order-independent, and only both-present-at-head proceeds. The per-group satisfaction logic ALREADY EXISTS in `crates/but-api/src/legacy/review_requirement.rs::evaluate` (GATES-005, `has_group_approval` lines 50-59 + 114-124); the per-group `unmet[]` entry is ALREADY the structured `format!("require_approval_from_group {group}: {reason}")` (review_requirement.rs:54-57) — so it ALREADY names the specific group. GATES-006 **consumes** that evaluator's `denial.unmet` Vec and proves the only-one-blocked matrix through the real merge gate by asserting the EXACT structured entry naming the still-missing group while the SATISFIED group's entry is ABSENT (the proof that the per-group check is AND, not OR) — it does NOT re-implement the evaluator. It also lands the two honesty build-gates 01b deferred: no enforcement branch on human-vs-AI (T-LOOP-011) and no role-name in the evaluator (T-LOOP-005 family).

## Why

Sprint 04 · PRD UC-GATES-02 (AC-5 each-required-group), UC-LOOP-02 (two-group AI+human) · capabilities CAP-AUTHZ-01. Sprint 01b shipped the two-group requirement *plumbing* (`merge_gate_two_group_both_present_proceeds` already exists) and single-group evaluation; the **only-one-blocked** matrix (AI-only blocks, human-only blocks) was deliberately deferred to here. This is the "human at the feature level, AI at the code level" quality model expressed entirely as `.gitbutler/gates.toml` config + group membership, with zero role-specific enforcement code.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-api merge_gate_two_group_only_one_blocked` (integration, real but-api merge gate + real git + real but-db). Full gate set in the spec below.

## Scope

- `crates/but-api/tests/merge_gate.rs` (MODIFY) — add the only-one-group-blocked strictness cases (AI-only blocked; human-only blocked; both-present proceeds positive control) through the real merge gate + real git + real but-db, seeding approvals via the governed `but-api::legacy::forge::approve_review` action; assert against the structured `denial.unmet` Vec (the EXACT `require_approval_from_group {group}: {reason}` entries) — the missing group present, the satisfied group absent; add a `MixedApprovals` discriminant to the existing `GateConfig`/`merge_gated_repo` helper ONLY if a single-approval-per-group seeding shape is needed beyond the existing `TwoGroup` fixture
- `crates/but-api/src/legacy/review_requirement.rs` (MODIFY — EXPECTED NO-OP) — the per-group `unmet[]` entry ALREADY names the specific group via `format!("require_approval_from_group {}: {reason}", group_name.as_str())` (review_requirement.rs:54-57), so the conditional sharpening this task budgeted is EXPECTED TO BE A NO-OP. Do NOT add any group-name string LITERAL ("code-reviewers"/"maintainers") to the enforcement source — the group name flows from `cfg`/`require_approval_from_group` at runtime; a hardcoded literal would trip the AC-3 role grep. Do NOT change the satisfaction semantics (they already pass `merge_gate_two_group_both_present_proceeds`)
- `crates/but/tests/but/command/merge_gate.rs` (MODIFY — optional) — CLI snapbox for an only-one-group-blocked denial surfacing the per-group unmet discriminator

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: GATES-006 - Per-required-group approval evaluation: only-one-group-blocked strictness matrix (two-group AI + human model) + no-human-vs-AI-branch build-gate
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     M  (150 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-GATES-02, UC-LOOP-02
CAPABILITIES: CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-api merge_gate_two_group_only_one_blocked
  check: cargo check -p but-api --all-targets
  lint:  cargo clippy -p but-api --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Integration tests are green through the real but-api merge gate + real git + real but-db: with a target-ref gate requiring require_approval_from_group=["code-reviewers","maintainers"] at min_approvals=1 distinct-from-author, a merge with ONLY a code-reviewers (AI) approval at head is blocked gate.review_required whose denial.unmet Vec CONTAINS the exact `require_approval_from_group maintainers: <reason>` entry AND does NOT contain a `require_approval_from_group code-reviewers: ...` entry (the satisfied AI group is absent — proving AND, not OR); a merge with ONLY a maintainers (human) approval at head is blocked gate.review_required whose denial.unmet Vec CONTAINS `require_approval_from_group code-reviewers: <reason>` AND does NOT contain a `require_approval_from_group maintainers: ...` entry (order-independent — neither single group satisfies); and a merge with a distinct approval from EACH required group at the current head, by a merge-holding principal, is PERMITTED by the gate (the evaluator returns Ok → NO gate.review_required raised; execution reaches the forge merge_review call — the forge-network completion is structural/out-of-local-scope, mirroring GATES-003/005). The per-group satisfaction is computed by the EXISTING review_requirement::evaluate (GATES-005), whose unmet[] entry is ALREADY the structured `require_approval_from_group {group}: {reason}` form (review_requirement.rs:54-57) — this task proves the only-one-blocked matrix against that structured payload; it does NOT re-implement the evaluator and adds NO group-name literal to the enforcement source. The two-tier model is pure config: no enforcement path branches on human-vs-AI, and no role label appears in the evaluator (build-gate).

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST require an approving review from EACH group named in require_approval_from_group: a merge with an approval from only ONE of two required groups is blocked, order-independent (UC-GATES-02 AC-5; UC-LOOP-02 AC-2/AC-3; T-GATES-012, T-LOOP-008, T-LOOP-009). The satisfaction logic already lives in review_requirement::evaluate's has_group_approval (review_requirement.rs:50-59, 114-124); the per-group unmet[] entry is ALREADY the structured `require_approval_from_group {group}: {reason}` (review_requirement.rs:54-57). This task PROVES the only-one-blocked matrix by asserting that structured entry for the still-missing group is present in denial.unmet while the satisfied group's entry is absent.
- [MUST] MUST CONSUME the GATES-005 evaluator (review_requirement::evaluate) — do NOT re-implement self-approval exclusion, stale-@head dismissal, or per-group satisfaction. The evaluator is OWNED by GATES-005 (review_requirement.rs); the merge-gate wrapper (merge_gate.rs) is OWNED by GATES-003. This task's writeAllowed on review_requirement.rs is EXPECTED to be a NO-OP: the per-group unmet[] entry ALREADY names the specific group (review_requirement.rs:54-57). DOCUMENT the seam: the only-one-blocked behavior is already produced by evaluate()'s per-group loop and its already-structured unmet entry; this task surfaces and proves it against the denial.unmet Vec.
- [MUST] MUST seed every approving verdict THROUGH the governed `but-api::legacy::forge::approve_review` action (the same helper `approve_branch(&ctx, principal_id)` the existing merge_gate tests use), NOT via a direct `db.local_review_verdicts_mut().insert(...)` — a direct insert is exactly the forgeable R6 accepted-leak path the gate is not supposed to exercise (R6). A code-reviewers approval is seeded as `approve_branch(&ctx, "reviewer-a")` (a member of code-reviewers); a maintainers approval as `approve_branch(&ctx, "reviewer-b")` (a member of maintainers).
- [MUST] MUST treat the merge GATE DECISION as the locally-provable surface for the POSITIVE (both-groups-present) case: `merge_review` is forge-bound (errors on a bare local repo) and has no `but pr merge` CLI verb, so "merge proceeds" proves the gate PERMITS (evaluate returns Ok → NO gate.review_required) and execution reaches the forge call past the gate — NOT that the change lands on the remote trunk (forge completion is structural/out-of-local-scope, the GATES-003/005 re-scope). DENIAL cases (only-one-group → gate.review_required) are fully locally provable.
- [MUST] MUST keep the two-tier model PURE CONFIG: the engine evaluates group-membership of the signers against the target-ref config and NEVER distinguishes a human principal from an AI principal — there is NO enforcement branch keyed on "human"/"ai"/"bot" or any role label (UC-LOOP-02 AC-5; T-LOOP-011, deferred from 01b, lands here as a build-gate).
- [STRICTLY] STRICTLY assert against the structured `denial.unmet` Vec — for the AI-only case assert it CONTAINS the exact `require_approval_from_group maintainers: <reason>` entry and does NOT contain a `require_approval_from_group code-reviewers: ...` entry (and the mirror for the human-only case). Use CONCRETE group names ("code-reviewers", "maintainers") IN THE TEST ASSERTIONS — not a generic 'blocked'. Do NOT weaken to a bare `denial.message.contains`. The existing `merge_gate_two_group_both_present_proceeds` asserts the denial message contains BOTH group names when NEITHER is approved; the new only-one-blocked cases assert exactly ONE structured unmet entry (the still-missing group) is present and the satisfied group's entry is absent.
- [NEVER] NEVER add a group-name string LITERAL ("code-reviewers"/"maintainers") or any role label to the ENFORCEMENT source (review_requirement.rs) — it would trip the AC-3 role grep and re-introduce role-specific code. The group name must flow from cfg/require_approval_from_group at runtime. Concrete group names live ONLY in the test fixtures + assertions (merge_gate.rs tests), which are out of the AC-3 grep scope.
- [NEVER] NEVER add a test asserting the forgeable direct-DB-write to local_review_verdicts is blocked, nor a forge-UI/auto-merge-on-platform/raw-push bypass — those are documented accepted-leaks (R6/R1); asserting them encodes a false guarantee.
- [NEVER] NEVER overload GitButler's repo-access Permission/RepoExclusive lock as the authorization carrier — authorization is the orthogonal Authority axis (02-system-components.md; RULES.md lock discipline). The gate keys off the Authority axis (but_authz::authorize / Authority::Merge in the wrapper) and the evaluator's group-membership read.
- [STRICTLY] STRICTLY confine edits to the test surface — the per-group unmet[] entry in review_requirement.rs ALREADY names the group, so the budgeted review_requirement.rs touch is EXPECTED to be a NO-OP; do NOT edit the merge-gate wrapper (merge_gate.rs, OWNED by GATES-003), the principal resolution / authorize(merge) (GATES-003), the local_review_verdicts table (GATES-002), or but-authz (consume only).

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: two-group requirement — AI-only (code-reviewers) approval blocks with denial.unmet CONTAINING `require_approval_from_group maintainers: ...` AND NOT containing the code-reviewers entry; human-only (maintainers) approval blocks with the mirror (order-independent)
- [ ] AC-2: a distinct approval from EACH required group at head, by a merge-holder, proceeds (positive/non-degenerate control — the per-group check does not over-reject)
- [ ] AC-3: the two-tier model is pure config — no enforcement branch on human-vs-AI; no role label in the evaluator (build-gate, word-boundary grep)
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: Only-one-required-group approval blocks, order-independent (AI-only blocks; human-only blocks) [PRIMARY]
  GIVEN: fixture `merge_two_group` (target-ref gate require_approval_from_group=["code-reviewers","maintainers"], min_approvals=1, require_distinct_from_author=true; code-reviewers={reviewer-a}, maintainers={reviewer-b,maint}; maint holds merge), BUT_AGENT_HANDLE=maint
  WHEN:  only reviewer-a (code-reviewers / AI) approves @head and a merge is attempted; then in a fresh fixture only reviewer-b (maintainers / human) approves @head and a merge is attempted
  THEN:  the AI-only attempt is blocked error.code=="gate.review_required" whose denial.unmet Vec CONTAINS the EXACT structured entry `require_approval_from_group maintainers: <reason>` (naming the missing human group) AND does NOT contain any `require_approval_from_group code-reviewers: ...` entry (the SATISFIED AI group is absent from unmet — this proves AND, not OR) (exit 1, trunk HEAD sha == base); the human-only attempt is the mirror — blocked error.code=="gate.review_required" whose denial.unmet Vec CONTAINS `require_approval_from_group code-reviewers: <reason>` AND does NOT contain a `require_approval_from_group maintainers: ...` entry (exit 1, trunk HEAD sha == base) — neither single group satisfies, order-independent. Assert against the denial.unmet Vec (the structured payload), NOT just denial.message.contains
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api merge gate + real git + real but-db
  VERIFY: cargo test -p but-api merge_gate_two_group_only_one_blocked
  SCENARIO: NEGATIVE_CONTROL would fail if a single required group's approval is treated as satisfying the whole require_approval_from_group (the AND collapses to OR) so the SATISFIED group's `require_approval_from_group <group>: ...` entry wrongly appears in denial.unmet OR the still-missing group's entry is absent; the unmet[] is empty or a generic static 'blocked' rather than the exact `require_approval_from_group <group>: <reason>` form; the gate is a no-op stub so the single-group merge proceeds; require_approval_from_group is read from the working tree/feature head rather than the target ref.

AC-2: A distinct approval from each required group at head proceeds (positive control: AND satisfied)
  GIVEN: fixture `merge_two_group`, BUT_AGENT_HANDLE=maint
  WHEN:  reviewer-a (code-reviewers) AND reviewer-b (maintainers) each approve @head, then a merge is attempted by maint
  THEN:  the evaluator counts one distinct approval from each required group, the requirement is satisfied, the merge is PERMITTED by the gate (Ok / NO gate.review_required raised) and execution reaches the forge merge_review call past the gate — the per-group check does not over-reject a fully-satisfied requirement
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api merge gate + real git + real but-db
  VERIFY: cargo test -p but-api merge_gate_two_group_both_present_proceeds
  SCENARIO: NEGATIVE_CONTROL would fail if a fully-satisfied two-group requirement is over-rejected (the evaluator is a degenerate always-Err stub); only one group's approval is required despite both being configured; the both-present merge is blocked despite a distinct approval from each group @head.

AC-3: The two-tier model is pure config — no human-vs-AI enforcement branch; no role label in the evaluator [build-gate]
  GIVEN: the per-group evaluation lives in review_requirement::evaluate (GATES-005), which compares verdict-signer group-membership against the target-ref GovConfig with NO knowledge of whether a principal is a human or an AI; the per-group unmet[] entry names the group via a runtime cfg value, not a hardcoded literal
  WHEN:  the enforcement source is structurally inspected with WORD-BOUNDARY grep (so a config FIELD name like require_approval_from_group does not false-positive on a role/group substring)
  THEN:  review_requirement.rs contains NO branch keyed on human-vs-AI (no `human`/`\bai\b`/`is_bot`/`is_human` enforcement discriminator) and NO role/group-name LITERAL as a standalone word (no `\bimplementer\b`/`\breviewer\b`/`\bmaintainer\b`) in its source; the two-group behavior is driven entirely by the config's require_approval_from_group + group membership
  TEST_TIER: unit (build-gate)   VERIFICATION_SERVICE: source grep + compile   UNIT_TEST_JUSTIFIED: pure structural invariants (no human-vs-AI branch, no role-name leakage) verified by grep/compile with zero runtime I/O; the behavioral strictness is proven by AC-1/AC-2 integration cases
  VERIFY: ./tools/governance-checks/check_no_role_literals.sh

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, edge): AI-only (code-reviewers) approval @head blocked gate.review_required; denial.unmet CONTAINS `require_approval_from_group maintainers: <reason>` AND does NOT contain a code-reviewers entry (T-LOOP-008, T-GATES-012)
    VERIFY: cargo test -p but-api merge_gate_two_group_only_one_blocked
- TC-2 (-> AC-1, edge): human-only (maintainers) approval @head blocked gate.review_required; denial.unmet CONTAINS `require_approval_from_group code-reviewers: <reason>` AND does NOT contain a maintainers entry, order-independent (T-LOOP-009)
    VERIFY: cargo test -p but-api merge_gate_two_group_only_one_blocked
- TC-3 (-> AC-2, happy_path): a distinct approval from each required group @head, by a merge-holder, proceeds (T-LOOP-010, T-LOOP-012, T-GATES-012)
    VERIFY: cargo test -p but-api merge_gate_two_group_both_present_proceeds
- TC-4 (-> AC-3, structural): no human-vs-AI enforcement branch + no role/group-name word-literal in the evaluator (T-LOOP-011, T-LOOP-005 family)
    VERIFY: ./tools/governance-checks/check_no_role_literals.sh
- TC-5 (-> AC-1, edge): the per-group denial.unmet entry is the EXACT `require_approval_from_group {group}: {reason}` structured form naming the SPECIFIC still-missing group (not a generic 'blocked'), so an orchestrator re-routes the right group (UC-LOOP-02)
    VERIFY: cargo test -p but-api merge_gate_two_group_only_one_blocked

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides: the proven only-one-required-group-blocked strictness matrix (AI-only blocks, human-only blocks, order-independent; both-present proceeds) through the real merge gate; the assertion that the per-group denial.unmet entry is the structured `require_approval_from_group {group}: {reason}` naming the specific missing group while the satisfied group's entry is absent; the T-LOOP-011 no-human-vs-AI-branch + no-role-name (word-boundary) build-gate on the evaluator
consumes: review_requirement::evaluate's per-group satisfaction (has_group_approval) + its already-structured `require_approval_from_group {group}: {reason}` unmet entry (GATES-005, crates/but-api/src/legacy/review_requirement.rs:50-59 loop incl. 54-57 entry, 114-124 membership); the merge-gate wrapper's invocation + Denial mapping (GATES-003, merge_gate.rs); the local_review_verdicts verdict rows incl. principal_id + head_oid (GATES-002); the target-ref group membership resolved by GovConfig (GRPS-001 consolidated union, config.rs load-time fold); the governed `but-api::legacy::forge::approve_review` seed action + the existing `approve_branch`/`merge_gated_repo(GateConfig::TwoGroup)` test harness (merge_gate.rs tests)
boundary_contracts:
  - CAP-AUTHZ-01: the per-group review requirement is evaluated as part of the merge gate's read-only enforcement; an approval from only one required group does not satisfy the AND-of-groups requirement; the evaluator judges signer group-membership against the target-ref config (GRPS-001), never an agent claim, and emits a per-group unmet entry naming the unsatisfied group from a runtime cfg value (never a hardcoded role literal).

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-api/tests/merge_gate.rs (MODIFY) — add the only-one-group-blocked strictness cases (AI-only blocked; human-only blocked) + reuse the existing both-present positive control; seed approvals via the governed `approve_branch` helper; assert against the structured `denial.unmet` Vec (missing group's `require_approval_from_group {group}: {reason}` entry present, satisfied group's entry absent); add a `MixedApprovals`/`TwoGroup`-variant fixture path only if the existing TwoGroup fixture cannot express one-group-only seeding
  - crates/but-api/src/legacy/review_requirement.rs (MODIFY — EXPECTED NO-OP) — the per-group unmet[] entry ALREADY names the group via `format!("require_approval_from_group {}: {reason}", group_name.as_str())` (54-57), so this touch is EXPECTED to be empty; do NOT change satisfaction semantics, do NOT add self/stale logic (already owned), do NOT add any group-name literal or role label (would trip AC-3 grep), keep grep-clean of role names + human/AI labels
  - crates/but/tests/but/command/merge_gate.rs (MODIFY — optional) — CLI snapbox for the only-one-group-blocked denial surfacing the per-group unmet discriminator
writeProhibited:
  - crates/but-api/src/legacy/merge_gate.rs — OWNED by GATES-003 (wrapper, principal resolution, authorize(merge), DB query, Denial mapping) and hardened by AUTHZ-004 (fail-closed classification + undefined-group hard-deny); do NOT edit the wrapper, only consume its denial
  - crates/but-authz/** — CONSUME PrincipalId/GroupName/GovConfig/group membership; do NOT modify the primitive
  - crates/but-db/src/table/local_review_verdicts.rs — OWNED by GATES-002; consume the verdict rows + the governed approve_review seed path, do not redefine
  - crates/but-error/src/lib.rs — the gate.review_required code is a merge-gate-owned &'static str (GATES-003/AUTHZ-004); no Code variants
  - any gitbutler-* crate beyond what the action boundary strictly requires (crates/AGENTS.md)
  - Any file not listed in write_allowed

--------------------------------------------------------------------------------
OUT OF SCOPE
--------------------------------------------------------------------------------
  - The review-requirement evaluator's self-approval exclusion + stale-@head dismissal + the no_approval/approval_stale_at_head discriminator are OWNED by GATES-005 (already Complete) — this task consumes them, does not re-implement them. The per-group `require_approval_from_group {group}: {reason}` unmet entry ALREADY exists (54-57) — this task ASSERTS it, it does not author it (the budgeted review_requirement.rs edit is an expected NO-OP).
  - The merge-gate wrapper (merge_gate.rs), BUT_AGENT_HANDLE→Principal resolution, authorize(merge), the DB query, the target-ref reads, AND the fail-closed classification (config.invalid/perm.denied ordering) + the undefined-required-group hard-deny are OWNED by GATES-003 + AUTHZ-004 (already landed). GATES-008 owns the standalone target-ref-only read proof. This task owns ONLY the per-group only-one-blocked strictness matrix + the T-LOOP-011 build-gate.
  - `merge_gate_two_group_both_present_proceeds` ALREADY EXISTS (merge_gate.rs tests) — AC-2 REUSES it as the positive control rather than re-authoring it; this task adds the only-one-blocked cases that do not yet exist.
  - No test asserts the forgeable direct-DB-write to local_review_verdicts is blocked (R6 accepted-leak); raw-git / forge-UI / auto-merge-on-platform bypasses are accepted-leaks (R1), NOT tested.
  - The forge-network merge COMPLETION (the change landing on the remote trunk) is NOT asserted locally — merge_review is forge-bound; the POSITIVE path proves the gate DECISION + reaching the forge call (GATES-003/005 re-scope).

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-api/src/legacy/review_requirement.rs (33-66, 114-124)
   Focus: THE EVALUATOR THIS TASK CONSUMES — evaluate() already loops require_approval_from_group and calls has_group_approval(cfg, group_name, &current_approvals) (50-59); has_group_approval (114-124) checks whether any current-head approver is a member of the named group in the target-ref GovConfig. CRITICALLY, the per-group unmet entry is ALREADY `format!("require_approval_from_group {}: {reason}", group_name.as_str())` (54-57) — it ALREADY names the specific group from a runtime cfg value. The only-one-blocked behavior is ALREADY produced here — this task ASSERTS it against denial.unmet; the budgeted review_requirement.rs edit is an EXPECTED NO-OP. Do NOT change the satisfaction logic and do NOT add a group-name literal.
2. crates/but-api/tests/merge_gate.rs (185-221, 451-587, 650-655)
   Focus: THE HARNESS TO EXTEND — `merge_gate_two_group_both_present_proceeds` (185-221) already proves the both-present positive control with GateConfig::TwoGroup (475-487 gates.toml require_approval_from_group=["code-reviewers","maintainers"]; 512-543 permissions: reviewer-a∈code-reviewers, reviewer-b∈maintainers, maint∈maintainers+merge). `approve_branch(&ctx, "reviewer-a")` / `approve_branch(&ctx, "reviewer-b")` (650-655) seed governed approvals via but-api forge::approve_review. Use `denial.unmet` (the Vec<String>) — for ONLY reviewer-a approved, assert it contains `require_approval_from_group maintainers: ...` and does NOT contain any `require_approval_from_group code-reviewers: ...`; for ONLY reviewer-b, the mirror.
3. .spec/prds/governance/07-uc-loop.md (37-46)
   Focus: UC-LOOP-02 — AC-2 (AI-only blocks: human's feature-level sign-off required), AC-3 (human-only blocks: AI code-level pass also required, order-independent), AC-4 (both proceed), AC-5 (no code distinguishes human from AI — grep-asserted, T-LOOP-011).
4. .spec/prds/governance/11-e2e-testing-criteria.md (119, 147-152)
   Focus: T-GATES-012 (approval required from each required group), T-LOOP-008 (AI-only blocks), T-LOOP-009 (human-only blocks), T-LOOP-010 (both proceed), T-LOOP-011 (build-gate: no human-vs-AI enforcement branch), T-LOOP-012 (only-one blocked, both pass integration).
5. .spec/prds/governance/tasks/sprint-01b-governed-loop-reference-flow/GATES-005-stale-self-approval.md (full)
   Focus: SIBLING that OWNS review_requirement::evaluate. This task consumes the evaluator + its already-structured per-group unmet entry; do NOT change GATES-005's no_approval/approval_stale_at_head reason shape — the `require_approval_from_group {group}: {reason}` wrapper around that reason is the existing entry this task asserts.
6. .spec/prds/governance/tasks/sprint-03-grps-groups-ref-pin/GRPS-001-effective-set-union-group-ceiling.md (full)
   Focus: the consolidated effective-set union + the target-ref group-membership read the evaluator's has_group_approval relies on (config.rs load-time fold is the single source of truth). The group membership counted for require_approval_from_group is the target-ref version.
7. crates/but-testsupport/src/lib.rs (71-97)
   Focus: writable_scenario + invoke_bash to seed `.gitbutler/` config blobs at refs/heads/main, branch to feat; NEVER std::env::temp_dir().join(...). The existing merge_gated_repo already uses writable_scenario("checkout-head-info") + invoke_bash — extend that, do not reinvent.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- Only-one-group-blocked strictness integration tests pass: `cargo test -p but-api merge_gate_two_group_only_one_blocked`  -> Exit 0; AC-1 (AI-only blocked: denial.unmet has `require_approval_from_group maintainers: ...`, no code-reviewers entry; human-only blocked: the mirror) green
- Positive control still passes: `cargo test -p but-api merge_gate_two_group_both_present_proceeds`  -> Exit 0; a fully-satisfied two-group requirement is NOT over-rejected
- No human-vs-AI enforcement branch and no role/group-name word-literal in the evaluator: `./tools/governance-checks/check_no_role_literals.sh`  -> Exit 0 (T-LOOP-011, T-LOOP-005 family). NOTE: the per-group unmet entry already names the group via a runtime cfg value (54-57), NOT a literal — so this grep stays clean as long as no concrete group-name literal is added. The test FIXTURE strings "reviewer-a"/"maintainers" live in merge_gate.rs tests, which is OUT of this grep scope — the grep targets the enforcement file only
- Per-group denial.unmet entry is the structured group-naming form: the only-one-blocked test asserts `denial.unmet` CONTAINS the exact `require_approval_from_group {missing-group}: {reason}` entry and does NOT contain the satisfied group's entry (maintainers present / code-reviewers absent when only code-reviewers approved, and vice-versa), not a generic 'blocked' and not a bare message.contains
- Crate compiles incl. tests: `cargo check -p but-api --all-targets`  -> Exit 0
- Clippy clean: `cargo clippy -p but-api --all-targets`  -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: Strictness-matrix proof over the existing per-group evaluator — drive the real merge gate (GATES-003 wrapper → review_requirement::evaluate) with a two-group target-ref requirement and seed governed approvals one group at a time, asserting against the structured `denial.unmet` Vec that a single required group's approval leaves the AND-of-groups requirement UNMET: the still-missing group's `require_approval_from_group {group}: {reason}` entry is present AND the satisfied group's entry is absent (the discriminator between AND and OR), and only both-present-at-head returns Ok. The evaluator's existing per-group loop (has_group_approval per require_approval_from_group entry) already implements the AND and already emits the structured per-group unmet entry (54-57); this task proves the only-one-blocked matrix against that payload — the review_requirement.rs touch is an expected NO-OP. The two-tier "human at feature, AI at code" model is config-only — the engine never branches on human-vs-AI (build-gate, word-boundary grep).
pattern_source: crates/but-api/src/legacy/review_requirement.rs:50-59 (the per-group AND loop, incl. 54-57 the already-structured `require_approval_from_group {group}: {reason}` unmet entry) + crates/but-api/tests/merge_gate.rs:185-221 (the existing TwoGroup both-present harness to extend) + crates/but-api/tests/merge_gate.rs:650-655 (governed approve_branch seeding)
anti_pattern: Re-implementing per-group satisfaction in the test or the wrapper instead of consuming evaluate() (ownership overlap with GATES-005); collapsing the AND-of-groups into an OR so one group satisfies (the soundness hole this matrix closes) — caught by asserting the satisfied group's entry is ABSENT from denial.unmet; asserting only `denial.message.contains` instead of the structured `denial.unmet` Vec (loses the AND-vs-OR discriminator); a generic 'blocked' unmet[] that does not name the specific missing group (loses the re-route signal); adding a group-name LITERAL or role label to review_requirement.rs (trips the AC-3 word-boundary grep, re-introduces role-specific code); seeding approvals via a direct local_review_verdicts insert (forgeable R6 path); branching on human-vs-AI or a role label in the evaluator (T-LOOP-011/T-LOOP-005 violation); asserting the forge-network merge completion locally (out of no-mocks scope); or asserting the forgeable direct-DB-write / raw-git bypass is blocked (false guarantee, R6/R1).

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — Proves GitButler's merge gate enforces an approval from EACH required group as a standalone only-one-blocked matrix (AI-only blocks, human-only blocks, order-independent; both-present proceeds), seeding governed approvals one group at a time via the real but-api forge::approve_review path and asserting against the structured denial.unmet Vec that the still-missing group's `require_approval_from_group {group}: {reason}` entry is present while the satisfied group's entry is absent (the AND-vs-OR discriminator). Consumes the GATES-005 evaluator (no re-implementation; the per-group unmet entry already names the group, so the review_requirement.rs touch is an expected NO-OP), lands the T-LOOP-011 no-human-vs-AI-branch + word-boundary no-role-name build-gate, and extends the existing merge_gate integration harness against real but-api + real git + real but-db.
reviewer: rust-reviewer
coding_standards: crates/AGENTS.md, crates/WORKSPACE_MODEL.md, crates/but/AGENTS.md, crates/but-api/src/legacy/review_requirement.rs

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: GATES-003, GATES-005, GRPS-001, GATES-002   (the merge-gate wrapper + the per-group evaluator + the consolidated target-ref group-membership union + the verdict store)
Blocks:     Sprint 05, Sprint 06b
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "GATES-006",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "notes": [
    "DE-CONFLICT (GATES-006 vs GATES-005): per-group satisfaction ALREADY EXISTS in review_requirement::evaluate (crates/but-api/src/legacy/review_requirement.rs:50-59 loop + 114-124 has_group_approval), AND the per-group unmet[] entry is ALREADY the structured `format!(\"require_approval_from_group {}: {reason}\", group_name.as_str())` (review_requirement.rs:54-57) — it already names the specific group from a runtime cfg value. GATES-006 CONSUMES it and proves the only-one-blocked matrix against the denial.unmet Vec; it does NOT re-implement the evaluator and the budgeted review_requirement.rs edit is an EXPECTED NO-OP (S3/R9 remediation: the live form was verified at review_requirement.rs:54-57).",
    "S3/R9: AC-1 + TC-1/TC-2 + TC-5 now assert the EXACT structured denial.unmet entries — AI-only: unmet CONTAINS `require_approval_from_group maintainers: ...` AND does NOT contain a code-reviewers entry (proving AND, not OR); human-only: the mirror. Assertion is against the denial.unmet Vec, not denial.message.contains.",
    "S5/R8: the AC-3 role grep is now WORD-BOUNDARY (\\bimplementer\\b|\\breviewer\\b|\\bmaintainer\\b) so the config FIELD name require_approval_from_group does not false-positive on a role/group substring. Explicit note: the per-group unmet entry already names the group at runtime (54-57), so the conditional review_requirement.rs edit is an expected NO-OP, and NO group-name literal may be added to the enforcement source (it would trip the word-boundary role grep).",
    "T-LOOP-011 (no human-vs-AI enforcement branch) was deferred from Sprint 01b and lands here as a build-gate (AC-3). Both greps target the enforcement file review_requirement.rs ONLY — the test FIXTURE strings (reviewer-a/maintainers) live in merge_gate.rs tests and are out of grep scope.",
    "Reviews seeded ONLY via the governed `approve_branch` helper (but-api forge::approve_review). NEVER a direct local_review_verdicts insert (R6 accepted-leak). POSITIVE path proves the gate DECISION + reaching the forge call (forge completion out-of-local-scope, GATES-003/005 re-scope)."
  ],
  "fixtures": {
    "merge_two_group": {
      "description": "A real git repo (but-testsupport writable_scenario, the existing merge_gated_repo(GateConfig::TwoGroup) shape) whose target ref main has committed .gitbutler/gates.toml ([[branch]] main protected=true; [[gate]] branch=main type=review min_approvals=1 require_distinct_from_author=true require_approval_from_group=[\"code-reviewers\",\"maintainers\"]) and .gitbutler/permissions.toml defining principal impl (contents:write/pull_requests:write/reviews:write, the change author), reviewer-a (reviews:write, groups=[code-reviewers]), reviewer-b (reviews:write, groups=[maintainers]), maint (merge+reviews:write, groups=[maintainers]); groups code-reviewers={reviewer-a} and maintainers={reviewer-b,maint}. A feat branch carries the change authored by impl, with an open governed review (forge_reviews upsert author=impl, source=feat, target=main, sha=<feat head>). Approvals are seeded one group at a time via the governed `approve_branch(&ctx, \"reviewer-a\")` (code-reviewers) / `approve_branch(&ctx, \"reviewer-b\")` (maintainers) — NEVER a direct DB insert.",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = merge_gated_repo(GateConfig::TwoGroup); // existing helper: writable_scenario(\"checkout-head-info\") + invoke_bash",
        "invoke_bash (already in merge_gated_repo): write .gitbutler/permissions.toml (impl=[contents:write,pull_requests:write,reviews:write]; reviewer-a=[reviews:write] groups=[code-reviewers]; reviewer-b=[reviews:write] groups=[maintainers]; maint=[merge,reviews:write] groups=[maintainers]; [[group]] code-reviewers members=[reviewer-a]; [[group]] maintainers members=[reviewer-b,maint])",
        "invoke_bash (already in merge_gated_repo): write .gitbutler/gates.toml ([[branch]] main protected=true; [[gate]] branch=main type=review min_approvals=1 require_distinct_from_author=true require_approval_from_group=[\"code-reviewers\",\"maintainers\"]); git add -A && git commit at refs/heads/main; git checkout -b feat; commit a change authored by impl; git checkout main",
        "let ctx = context_with_review(&repo, ref_id(&repo, FEAT_REF)?)?; // seeds the open governed forge review",
        "AI-only case: approve_branch(&ctx, \"reviewer-a\").await? (code-reviewers approval ONLY, governed) — leave maintainers UN-approved",
        "human-only case (fresh fixture): approve_branch(&ctx, \"reviewer-b\").await? (maintainers approval ONLY, governed) — leave code-reviewers UN-approved",
        "both-present positive control: approve_branch(&ctx, \"reviewer-a\").await? AND approve_branch(&ctx, \"reviewer-b\").await? (one distinct approval per required group @head, governed)"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN merge_two_group (require_approval_from_group=[\"code-reviewers\",\"maintainers\"]) WHEN only reviewer-a (code-reviewers/AI) approves @head and maint attempts the merge, then in a fresh fixture only reviewer-b (maintainers/human) approves @head and maint attempts the merge THEN the AI-only attempt is blocked gate.review_required whose denial.unmet Vec CONTAINS the exact `require_approval_from_group maintainers: <reason>` entry AND does NOT contain a `require_approval_from_group code-reviewers: ...` entry (satisfied group absent — AND, not OR), and the human-only attempt is the mirror (denial.unmet CONTAINS `require_approval_from_group code-reviewers: <reason>` AND does NOT contain a maintainers entry) — order-independent, trunk HEAD sha == base in both",
      "verify": "cargo test -p but-api merge_gate_two_group_only_one_blocked",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api merge gate + real git + real but-db",
        "negative_control": {
          "would_fail_if": [
            "a single required group's approval is treated as satisfying the whole require_approval_from_group (the AND-of-groups collapses to OR) so the satisfied group's `require_approval_from_group <group>: ...` entry wrongly appears in denial.unmet or the still-missing group's entry is absent",
            "the unmet[] is empty or a generic static 'blocked' rather than the exact `require_approval_from_group <group>: <reason>` form naming the still-missing group",
            "the gate is a no-op stub so the single-group merge proceeds (trunk advances)",
            "require_approval_from_group is read from the working-tree/feature-head gates.toml rather than the target ref (a mock/stub config source)",
            "approvals are seeded by a direct local_review_verdicts insert rather than the governed approve_review action (forgeable R6 path)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "merge_two_group",
            "action": {
              "actor": "cli_user",
              "steps": [
                "approve_branch(&ctx, \"reviewer-a\") — ONLY a code-reviewers (AI) approval @head, governed",
                "BUT_AGENT_HANDLE=maint: invoke the governed merge action on the open review (cite T-LOOP-008, T-GATES-012, UC-LOOP-02 AC-2)"
              ]
            },
            "end_state": {
              "must_observe": [
                "`error.code == \"gate.review_required\"`",
                "`denial.unmet` Vec contains the exact entry `require_approval_from_group maintainers: <reason>` (the human group has not approved)",
                "process exits `1`",
                "the review is NOT merged (trunk/main HEAD sha `==` the seeded base sha)"
              ],
              "must_not_observe": [
                "no `require_approval_from_group code-reviewers` entry in denial.unmet (the satisfied AI group is absent — AND, not OR)",
                "an empty unmet (the AND collapsed to OR)",
                "merge proceeded",
                "exit `0`"
              ]
            }
          },
          {
            "start_ref": "merge_two_group",
            "action": {
              "actor": "cli_user",
              "steps": [
                "in a FRESH merge_two_group fixture: approve_branch(&ctx, \"reviewer-b\") — ONLY a maintainers (human) approval @head, governed",
                "BUT_AGENT_HANDLE=maint: invoke the governed merge action on the open review (cite T-LOOP-009, order-independent, UC-LOOP-02 AC-3)"
              ]
            },
            "end_state": {
              "must_observe": [
                "`error.code == \"gate.review_required\"`",
                "`denial.unmet` Vec contains the exact entry `require_approval_from_group code-reviewers: <reason>` (the AI group has not approved)",
                "process exits `1`",
                "the review is NOT merged (trunk/main HEAD sha `==` the seeded base sha)"
              ],
              "must_not_observe": [
                "no `require_approval_from_group maintainers` entry in denial.unmet (the satisfied human group is absent — AND, not OR)",
                "an empty unmet (the AND collapsed to OR)",
                "merge proceeded",
                "exit `0`"
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
      "description": "GIVEN merge_two_group WHEN reviewer-a (code-reviewers) AND reviewer-b (maintainers) each approve @head and maint attempts the merge THEN the evaluator counts one distinct approval per required group, the requirement is satisfied, and the merge is PERMITTED by the gate (Ok / NO gate.review_required raised) reaching the forge call — the per-group check does not over-reject a fully-satisfied requirement (positive/non-degenerate control; reuses the existing merge_gate_two_group_both_present_proceeds test)",
      "verify": "cargo test -p but-api merge_gate_two_group_both_present_proceeds",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api merge gate + real git + real but-db",
        "negative_control": {
          "would_fail_if": [
            "a fully-satisfied two-group requirement is over-rejected (the evaluator is a degenerate always-Err stub)",
            "only one group's approval is required despite both being configured (the both-present case is treated identically to one-present)",
            "the both-present merge is blocked despite a distinct approval from each required group @head (a static block)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "merge_two_group",
            "action": {
              "actor": "cli_user",
              "steps": [
                "approve_branch(&ctx, \"reviewer-a\") AND approve_branch(&ctx, \"reviewer-b\") — one distinct governed approval per required group @head",
                "BUT_AGENT_HANDLE=maint: invoke the governed merge action (cite T-LOOP-010, T-LOOP-012)"
              ]
            },
            "end_state": {
              "must_observe": [
                "the gate PERMITS the merge — the output contains NO `error.code == \"gate.review_required\"` and NO `perm.denied` (0 governance denials: `classify_error(&err).is_none()`)",
                "execution reaches the governed `merge_review` body past the gate (any failure is a forge/remote error, NOT a governance Denial)"
              ],
              "must_not_observe": [
                "`error.code == \"gate.review_required\"` raised when both required groups have approved @head",
                "the both-present requirement over-rejected (0 governance denials expected)",
                "a governance Denial blocks the merge"
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
      "description": "GIVEN the per-group evaluation lives in review_requirement::evaluate (which compares signer group-membership against the target-ref GovConfig with no human/AI knowledge, and names the unsatisfied group via a runtime cfg value not a literal) WHEN the enforcement source is structurally inspected with word-boundary grep THEN review_requirement.rs contains NO human-vs-AI enforcement branch and NO role/group-name word-literal — the two-group behavior is driven entirely by the config's require_approval_from_group + group membership",
      "verify": "! grep -rEni 'is_bot|is_human|\"human\"|\"ai\"|\\bhuman\\b|\\bbot\\b' crates/but-api/src/legacy/review_requirement.rs && ! grep -rEni '\\bimplementer\\b|\\breviewer\\b|\\bmaintainer\\b' crates/but-api/src/legacy/review_requirement.rs",
      "maps_to_ac": null,
      "scenario": {
        "tier": "holdout",
        "test_tier": "unit",
        "unit_test_justified": "pure structural invariants (no human-vs-AI enforcement branch, no role-name word-literal leakage) verified by word-boundary grep/compile with zero runtime I/O; the behavioral strictness of the two-group matrix is proven by the AC-1/AC-2 integration cases",
        "verification_service": "source grep + compile (build-gate, no runtime I/O)",
        "negative_control": {
          "would_fail_if": [
            "the evaluator branches on a human-vs-AI discriminator (is_human/is_bot/\"human\"/\"ai\") instead of pure group-membership",
            "a role/group-name word-literal (implementer/reviewer/maintainer) is hardcoded in the evaluator's source instead of flowing from the runtime cfg value",
            "the two-tier behavior is implemented with a hardcoded human/AI mapping or a static role map rather than config-driven group membership"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "merge_two_group",
            "action": {
              "actor": "ci",
              "steps": [
                "word-boundary grep the evaluator review_requirement.rs for human-vs-AI discriminators and role-name word-literals (cite T-LOOP-011, T-LOOP-005)"
              ]
            },
            "end_state": {
              "must_observe": [
                "`! grep -rEni 'is_bot|is_human|\"human\"|\"ai\"|\\bhuman\\b|\\bbot\\b' crates/but-api/src/legacy/review_requirement.rs` → 0 matches (no human-vs-AI enforcement branch)",
                "`! grep -rEni '\\bimplementer\\b|\\breviewer\\b|\\bmaintainer\\b' crates/but-api/src/legacy/review_requirement.rs` → 0 matches (no role/group-name word-literal; the require_approval_from_group field name does not false-positive under word boundaries)"
              ],
              "must_not_observe": [
                "a `human`/`bot`/`is_human`/`is_bot` enforcement branch in the evaluator",
                "a role/group-name word-literal (`implementer`/`reviewer`/`maintainer`) hardcoded in the evaluator",
                "a non-empty grep result — 1+ matches where 0 (empty / clean) is required"
              ]
            }
          }
        ]
      }
    },
    { "id": "TC-1", "type": "test_criterion", "description": "AI-only (code-reviewers) approval @head blocked gate.review_required; denial.unmet contains `require_approval_from_group maintainers: <reason>` AND no code-reviewers entry (T-LOOP-008, T-GATES-012)", "verify": "cargo test -p but-api merge_gate_two_group_only_one_blocked", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "human-only (maintainers) approval @head blocked gate.review_required; denial.unmet contains `require_approval_from_group code-reviewers: <reason>` AND no maintainers entry, order-independent (T-LOOP-009)", "verify": "cargo test -p but-api merge_gate_two_group_only_one_blocked", "maps_to_ac": "AC-1" },
    { "id": "TC-3", "type": "test_criterion", "description": "a distinct approval from each required group @head, by a merge-holder, proceeds (positive control; T-LOOP-010, T-LOOP-012)", "verify": "cargo test -p but-api merge_gate_two_group_both_present_proceeds", "maps_to_ac": "AC-2" },
    { "id": "TC-4", "type": "test_criterion", "description": "no human-vs-AI enforcement branch + no role/group-name word-literal in the evaluator (word-boundary grep; T-LOOP-011, T-LOOP-005 family)", "verify": "./tools/governance-checks/check_no_role_literals.sh", "maps_to_ac": "AC-3" },
    { "id": "TC-5", "type": "test_criterion", "description": "the per-group denial.unmet entry is the EXACT `require_approval_from_group {group}: {reason}` structured form naming the SPECIFIC still-missing group (not a generic 'blocked') so an orchestrator re-routes the right group (UC-LOOP-02)", "verify": "cargo test -p but-api merge_gate_two_group_only_one_blocked", "maps_to_ac": "AC-1" }
  ]
}
-->
</details>
</content>
</invoke>
