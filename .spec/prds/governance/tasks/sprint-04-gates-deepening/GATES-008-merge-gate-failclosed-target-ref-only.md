# GATES-008: Standalone target-ref-only read proof for the merge gate — a feature-head `gates.toml` requirement-drop is ignored (commit-gate target-ref-only is already proven by GATES-001 + GATES-007); deepening, not a competing owner of AUTHZ-004's merge-path fail-closed

## What this does

Proves, as a DEDICATED standalone test, that the merge gate reads its requirement ONLY from the committed TARGET-REF `.gitbutler/gates.toml` blob — never the feature head being merged (T-GATES-019). A feature head that commits a `gates.toml` DROPPING the merge requirement is STILL judged by the target-ref requirement, so a change can never weaken its own gate. The proof is DISCRIMINATING: the feature head drops the requirement to `min_approvals=0` (no `[[gate]]`), so (a) at zero approvals the merge is blocked `gate.review_required` (the target-ref `min_approvals=1` binds — a feature-head-reading stub would have PERMITTED here), and (b) after ONE distinct governed approval the merge is PERMITTED because the TARGET-REF `min_approvals=1` is satisfied (NOT because the feature-head `min_approvals=0` trivially passed). The commit-gate target-ref-only property is ALREADY proven dedicated by GATES-001's `commit_gate_edit_cannot_unprotect` (commit_gate.rs:102 — working-tree edit + feature-head-committed unprotect, both → branch.protected, 8/8 commit_gate tests pass) and GATES-007's target-ref-pin — so GATES-008 does NOT re-author a commit-gate target-ref case; it references them. This is the standalone-proof DEEPENING of the target-ref-only property on the merge path — it is NOT a re-implementation of the merge-path fail-closed classification (deterministic `config.invalid` vs `perm.denied` vs `gate.review_required`, unknown/no-handle deny, and the undefined-`require_approval_from_group` hard-deny), which ALREADY LANDED in Sprint 02 AUTHZ-004 (`crates/but-api/src/legacy/merge_gate.rs`) and whose dedicated tests already pass.

## Why

Sprint 04 · PRD UC-GATES-02 (AC-6/AC-10 target-ref-only), UC-AUTHZ-04 · capabilities CAP-CONFIG-01. The roadmap deliberately re-grounds GATES-008 against AUTHZ-004: the merge-path malformed/undefined-group fail-closed is fully owned by AUTHZ-004 (Sprint 02). The honest, non-duplicative residual is the DEDICATED merge-path target-ref-only read proof (T-GATES-019) — proven as a discriminating standalone test (zero-approval blocked → one-approval permitted) rather than as a side-effect of another case — that a feature-head `gates.toml` requirement-drop is ignored because the gate reads only the target-ref blob.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-api merge_gate_targetref_only_feature_head_drop_ignored` (integration, real but-api merge gate + real git + real but-db). Full gate set in the spec below.

## Scope

- `crates/but-api/tests/merge_gate.rs` (MODIFY) — add the DEDICATED discriminating target-ref-only standalone proof on the merge path: a feature-head `gates.toml` that drops the requirement (min_approvals=0 / no `[[gate]]`) is ignored; at zero approvals the gate blocks per the target-ref `min_approvals=1` (a feature-head-reading stub would have permitted), then PERMITS only after one distinct governed approval satisfies the TARGET-REF requirement. Add a `FeatureHeadDropsRequirement` discriminant to the existing `GateConfig`/`merge_gated_repo` helper that commits a WEAKER gates.toml on the feat head while main keeps the strong requirement
- `crates/but/tests/but/command/merge_gate.rs` (MODIFY — optional) — CLI snapbox proving the feature-head requirement drop does not weaken the governed merge

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: GATES-008 - Standalone target-ref-only read proof for the merge gate: a feature-head gates.toml requirement-drop is ignored (commit-gate target-ref-only already proven by GATES-001 + GATES-007); deepening, NOT a competing owner of AUTHZ-004's merge-path fail-closed
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     M  (120 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-GATES-02, UC-AUTHZ-04
CAPABILITIES: CAP-CONFIG-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-api merge_gate_targetref_only_feature_head_drop_ignored
  check: cargo check -p but-api --all-targets
  lint:  cargo clippy -p but-api --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Integration tests are green through the real but-api merge gate + real git + real but-db: a merge whose FEATURE HEAD commits a `.gitbutler/gates.toml` that DROPS the review requirement (min_approvals=0 / no [[gate]]) is STILL judged by the TARGET-REF (main) requirement (T-GATES-019). The proof is DISCRIMINATING — it distinguishes a target-ref read from a feature-head read: (a) at ZERO qualifying approvals the merge is blocked gate.review_required with a non-empty unmet[] (the target-ref min_approvals=1 distinct binds; a feature-head-reading stub would have PERMITTED here because the feature head's min_approvals=0 is vacuously satisfied), and (b) after ONE distinct governed approval @head the merge is PERMITTED because the TARGET-REF min_approvals=1 is satisfied (NOT because the feature-head min_approvals=0 trivially passed). The commit-gate target-ref-only property is consumed-by-reference from GATES-001's commit_gate_edit_cannot_unprotect (commit_gate.rs:102) + GATES-007's target-ref-pin — GATES-008 does NOT re-author a commit-gate target-ref case. This task is the DEEPENING of the merge-path target-ref-only property; it does NOT re-implement and does NOT compete with AUTHZ-004's merge-path fail-closed classification (config.invalid/perm.denied/gate.review_required ordering + the undefined-group hard-deny), which already landed in Sprint 02 and whose tests already pass.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST RE-GROUND HONESTLY against AUTHZ-004 (Sprint 02). The merge-path fail-closed classification — deterministic `config.invalid` vs `perm.denied` vs `gate.review_required` ordering (config-load-first, caller-independent), unknown/no-handle deny, DryRun-no-bypass, AND the undefined-`require_approval_from_group` HARD-DENY — is ALREADY IMPLEMENTED in crates/but-api/src/legacy/merge_gate.rs (load_merge_governance_config → config.invalid; resolve_principal_from_env + authorize(Merge) → perm.denied; undefined_required_groups → gate.review_required) and its dedicated tests ALREADY PASS (merge_gate_malformed_config_is_config_invalid, merge_gate_undefined_required_group_denied, merge_gate_dryrun_unknown_failclosed_persists_nothing). AUTHZ-004's OUT-OF-SCOPE explicitly states: "AUTHZ-004 OWNS the merge-path undefined-group hard-deny now; GATES-008 is the deepening / standalone-proof (commit-gate + dedicated target-ref-only proofs), NOT a competing owner of the merge-path check." DO NOT re-implement, DO NOT add a competing undefined-group / malformed-config check, DO NOT modify merge_gate.rs's classification logic. Consume it.
- [MUST] MUST scope GATES-008 to the RESIDUAL: the DEDICATED discriminating target-ref-only read proof (T-GATES-019) on the merge path — a feature-head `gates.toml` that drops/weakens the requirement is ignored; the gate is judged by the target-ref requirement (blocked at zero approvals → permitted after one target-ref-satisfying approval). The commit-gate target-ref-only property is ALREADY proven dedicated by GATES-001 (commit_gate_edit_cannot_unprotect, commit_gate.rs:102 — working-tree edit AND feature-head committed unprotect, both → branch.protected, 8/8 commit_gate tests pass) + GATES-007's target-ref-pin — GATES-008 references those and does NOT author a duplicate commit-gate target-ref case.
- [MUST] MUST read the requirement ONLY from the TARGET-REF gates.toml blob via the existing loader (merge_gate.rs::load_merge_governance_config(repo, target_ref)) — the feature-head/working-tree blob MUST be ignored (CAP-CONFIG-01). The existing merge_gated_repo fixture ALREADY commits an EMPTY gates.toml on the feat head (merge_gate.rs:576 `: >.gitbutler/gates.toml`) — GATES-008 makes the feat-head weakening EXPLICIT (min_approvals=0 / no [[gate]]) and asserts it is ignored via the DISCRIMINATING zero-approval-blocked → one-approval-permitted cycle.
- [MUST] MUST make the proof DISCRIMINATING (target-ref read vs feature-head read): the positive (permitted) phase ALONE cannot distinguish the two reads (both permit). So (a) prove zero approvals → blocked gate.review_required (the target-ref binds; a feature-head-reading stub would PERMIT here because the feature head's min_approvals=0 is vacuously met), and (b) one distinct governed approval → PERMITTED (target-ref min_approvals=1 satisfied — NOT the feature-head's min_approvals=0 trivially passing). Document in the test that a feature-head-reading stub would have permitted at step (a).
- [MUST] MUST seed every approving verdict THROUGH the governed `but-api::legacy::forge::approve_review` action (the existing `approve_branch` helper), NOT a direct local_review_verdicts insert (R6 accepted-leak).
- [NEVER] NEVER add a test asserting the forgeable direct-DB-write to local_review_verdicts is blocked, nor a forge-UI/auto-merge-on-platform/raw-push bypass — accepted-leaks (R6/R1).
- [NEVER] NEVER overload GitButler's repo-access Permission/RepoExclusive lock as the authorization carrier — authorization is the orthogonal Authority axis. The merge gate keys off but_authz::authorize / Authority::Merge in the wrapper (already wired by GATES-003/AUTHZ-004).
- [STRICTLY] STRICTLY treat the merge GATE DECISION as the locally-provable surface for POSITIVE cases (merge_review is forge-bound; "proceeds" proves the gate PERMITS + reaches the forge call — GATES-003/005 re-scope). The DENIAL (target-ref requirement still binds at zero approvals) is fully locally provable.
- [STRICTLY] STRICTLY confine edits to the TEST surface (merge_gate.rs tests + the GateConfig fixture helper; optionally a CLI snapbox) — do NOT modify the merge-gate wrapper (merge_gate.rs production code, OWNED by GATES-003/AUTHZ-004), the commit gate (GATES-001/GATES-007), the evaluator (GATES-005/006), or but-authz. This is a standalone PROOF task; the production target-ref-only behavior already exists.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: a feature-head gates.toml dropping the merge requirement is IGNORED — at zero approvals the merge is blocked per the target-ref requirement (discriminating: a feature-head-reading stub would permit), then proceeds only when a distinct approval satisfies the target-ref requirement @head
- [ ] AC-2: GATES-008 adds NO competing merge-path fail-closed check — merge_gate.rs production classification (config.invalid/perm.denied/undefined-group) is UNCHANGED; AUTHZ-004 remains the owner (build-gate, direct-diff form)
- [ ] All verification gates pass; only write_allowed files (tests) modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: A feature-head gates.toml dropping the merge requirement is ignored — the target-ref requirement still binds (discriminating: blocked at zero approvals, permitted after one target-ref-satisfying approval) [PRIMARY]
  GIVEN: fixture `merge_targetref_only` (target ref main commits gates.toml with the STRONG requirement min_approvals=1 require_distinct_from_author=true; the feat HEAD commits a WEAKER gates.toml — no [[gate]] / min_approvals=0 — attempting to drop the requirement), an open governed review on feat with ZERO qualifying approvals @head, BUT_AGENT_HANDLE=maint (holds merge)
  WHEN:  a merge is attempted by maint at ZERO approvals (the feature head's gates.toml would, if read, drop the requirement to vacuously satisfied); then a distinct approval is added @head via governed `but review approve` and the merge is re-attempted
  THEN:  the first (zero-approval) attempt is blocked error.code=="gate.review_required" with a non-empty unmet[] — the feature-head drop is IGNORED, the target-ref min_approvals=1 distinct requirement binds (exit 1, trunk HEAD sha == base) — and CRITICALLY a feature-head-reading stub would have PERMITTED here (the feature head's min_approvals=0 is vacuously satisfied), so the block is the target-ref read; after the distinct approval @head the gate is satisfied per the TARGET-REF requirement and the merge is PERMITTED (Ok / no gate.review_required — reaches the forge call) — permitted because target-ref min_approvals=1 is met, NOT because the feature-head min_approvals=0 trivially passed
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api merge gate + real git + real but-db
  VERIFY: cargo test -p but-api merge_gate_targetref_only_feature_head_drop_ignored
  SCENARIO: NEGATIVE_CONTROL would fail if the gate reads the feature-head gates.toml so the dropped requirement weakens the gate and the merge proceeds with zero approvals (a feature-head-reading stub permits at step a); the requirement is read from the working tree; the gate is a no-op stub; the requirement is satisfied vacuously because the feature head removed the [[gate]] entry.

AC-2: GATES-008 adds NO competing merge-path fail-closed check — AUTHZ-004 remains the owner [build-gate]
  GIVEN: AUTHZ-004 owns the merge-path fail-closed classification (config.invalid/perm.denied/gate.review_required ordering + the undefined-require_approval_from_group hard-deny) in merge_gate.rs production code
  WHEN:  the diff for this task is structurally inspected with the DIRECT-DIFF form (added production lines in merge_gate.rs)
  THEN:  this task modifies NO production classification logic in crates/but-api/src/legacy/merge_gate.rs (the change set is test files + the GateConfig fixture helper only) — the merge-path fail-closed implementation remains exactly as AUTHZ-004 landed it; GATES-008 is a standalone proof, not a competing owner
  TEST_TIER: unit (build-gate)   VERIFICATION_SERVICE: git diff scope + compile   UNIT_TEST_JUSTIFIED: the ownership/no-duplication invariant (production classification unchanged) is a diff-scope/compile build-gate with zero runtime I/O; the behavioral target-ref-only property is proven by AC-1 integration cases
  VERIFY: ./tools/governance-checks/check_merge_gate_production_unchanged.sh

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, edge): a feature-head gates.toml dropping the merge requirement is ignored; at zero approvals the target-ref requirement binds and blocks gate.review_required — a feature-head-reading stub would permit (T-GATES-019, T-GATES-013, UC-GATES-02 AC-6/AC-10)
    VERIFY: cargo test -p but-api merge_gate_targetref_only_feature_head_drop_ignored
- TC-2 (-> AC-1, happy_path): after a distinct approval @head per the TARGET-REF requirement, the merge is permitted (the target-ref requirement, once satisfied, proceeds — NOT the feature-head min_approvals=0 trivially passing)
    VERIFY: cargo test -p but-api merge_gate_targetref_only_feature_head_drop_ignored
- TC-3 (-> AC-2, structural): GATES-008 modifies NO production merge-path classification logic (merge_gate.rs production unchanged); AUTHZ-004 remains the owner (no duplicate undefined-group/malformed check) — direct-diff form
    VERIFY: ./tools/governance-checks/check_merge_gate_production_unchanged.sh
- TC-4 (-> AC-1, edge): the proof is a DEDICATED discriminating standalone case (not a side-effect of another case) capturing concrete target-ref vs feature-head gates.toml before/after (T-GATES-019)
    VERIFY: cargo test -p but-api merge_gate_targetref_only_feature_head_drop_ignored

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-CONFIG-01
provides: the DEDICATED discriminating standalone target-ref-only read proof (T-GATES-019) for the merge gate — a feature-head/working-tree gates.toml requirement-drop is ignored, the target-ref requirement binds (blocked at zero approvals, permitted after one target-ref-satisfying approval); a build-gate that this task adds no competing merge-path fail-closed check (AUTHZ-004 remains the owner). The commit-gate target-ref-only property is referenced from GATES-001 (commit_gate_edit_cannot_unprotect) + GATES-007, not re-authored.
consumes: the merge-gate wrapper's target-ref load + classification, ENTIRELY AS-IS from AUTHZ-004/GATES-003 (crates/but-api/src/legacy/merge_gate.rs: load_merge_governance_config target-ref read; undefined_required_groups; the config.invalid/perm.denied ordering); the commit gate's target-ref-only proof (GATES-001 commit_gate_edit_cannot_unprotect, commit_gate.rs:102; GATES-007 target-ref-pin); the review-requirement evaluator (GATES-005/006); the governed approve_review seed action + the existing merge_gated_repo/GateConfig test harness (merge_gate.rs tests, which already commits an empty gates.toml on feat)
boundary_contracts:
  - CAP-CONFIG-01: the merge gate reads its requirement ONLY from the target-ref committed gates.toml blob; a feature-head or working-tree edit that weakens the requirement is inert because the gate never reads it. Proven from the consumer side with a DISCRIMINATING control — a stub reading the feature head would falsely PERMIT at zero approvals (the feature head's min_approvals=0 is vacuously satisfied).

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-api/tests/merge_gate.rs (MODIFY) — add the DEDICATED discriminating merge-path target-ref-only standalone proof (feature-head requirement drop ignored → blocked at zero approvals per target-ref → proceeds after a target-ref-satisfying approval); add a `FeatureHeadDropsRequirement` discriminant to the GateConfig/merge_gated_repo helper that commits a WEAKER gates.toml on the feat head while main keeps the STRONG requirement
  - crates/but/tests/but/command/merge_gate.rs (MODIFY — optional) — CLI snapbox proving the feature-head requirement drop does not weaken the governed merge
writeProhibited:
  - crates/but-api/src/legacy/merge_gate.rs — PRODUCTION code OWNED by GATES-003 (wrapper) + AUTHZ-004 (fail-closed classification + undefined-group hard-deny + config-load-first ordering). GATES-008 is a STANDALONE PROOF — it does NOT modify the classification logic, does NOT add a competing undefined-group/malformed check. CONSUME the existing behavior.
  - crates/but-api/src/legacy/review_requirement.rs — OWNED by GATES-005/GATES-006; consume the evaluator
  - crates/but-api/src/commit/gate.rs + crates/but-api/src/branch.rs + crates/but-api/src/legacy/worktree.rs — the commit gate + its mechanism-agnostic coverage are GATES-001/GATES-007; GATES-008 consumes their target-ref-only behavior by reference, does not modify them
  - crates/but-api/tests/commit_gate.rs — the commit-gate target-ref-only property is ALREADY proven by GATES-001's commit_gate_edit_cannot_unprotect (:102); GATES-008 does NOT add a duplicate commit-gate target-ref test (R7 option a: reference, do not re-author)
  - crates/but-authz/** — consume load_governance_config + the target-ref read; do NOT modify the primitive
  - crates/but-db/src/table/local_review_verdicts.rs — OWNED by GATES-002; consume the governed approve_review seed path
  - crates/but-error/src/lib.rs — no Code variants
  - any gitbutler-* crate (crates/AGENTS.md)
  - Any file not listed in write_allowed

--------------------------------------------------------------------------------
OUT OF SCOPE
--------------------------------------------------------------------------------
  - The merge-path fail-closed classification (deterministic config.invalid vs perm.denied vs gate.review_required ordering, config-load-first caller-independent, unknown/no-handle deny, DryRun-no-bypass) AND the undefined-`require_approval_from_group` hard-deny are OWNED by AUTHZ-004 (Sprint 02, ALREADY LANDED in merge_gate.rs) and their dedicated tests ALREADY PASS (merge_gate_malformed_config_is_config_invalid, merge_gate_undefined_required_group_denied, merge_gate_dryrun_unknown_failclosed_persists_nothing). GATES-008 is the DEEPENING / standalone target-ref-only proof, NOT a competing owner. No duplicate implementation. (Cite AUTHZ-004 OUT-OF-SCOPE: "GATES-008 is the deepening / standalone-proof, NOT a competing owner of the merge-path check.")
  - The COMMIT-gate target-ref-only property (a working-tree/feature-head gates.toml cannot unprotect a protected branch on the commit path) is ALREADY proven DEDICATED by GATES-001's commit_gate_edit_cannot_unprotect (crates/but-api/tests/commit_gate.rs:102 — working-tree edit AND feature-head committed unprotect, both → branch.protected; 8/8 commit_gate tests pass) and GATES-007's commit_gate_apply_integrate_dryrun_targetref_pinned. GATES-008 REFERENCES these as full coverage and adds NO commit-gate target-ref test (R7 option a — deterministic, no "skip if already covered" ambiguity; the prior AC-2 commit-gate standalone case is DROPPED because the dedicated coverage already exists).
  - The per-required-group strictness matrix (only-one-blocked) is GATES-006.
  - The mechanism-agnostic commit-gate coverage (worktree-integrate branch.protected; apply/integrate contents:write) is GATES-007.
  - No test asserts the forgeable direct-DB-write to local_review_verdicts is blocked (R6); raw-git / forge-UI / auto-merge-on-platform bypasses are accepted-leaks (R1), NOT tested.
  - The forge-network merge COMPLETION is NOT asserted locally (merge_review is forge-bound; POSITIVE proves the gate DECISION + reaching the forge call — GATES-003/005 re-scope).

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. .spec/prds/governance/tasks/sprint-02-authz-fail-closed-identity-confinement/AUTHZ-004-merge-gate-fail-closed.md (full)
   Focus: THE DE-CONFLICTION SOURCE + OWNER — AUTHZ-004 OWNS the merge-path fail-closed classification (config.invalid/perm.denied/gate.review_required ordering, config-load-first caller-independent) AND the undefined-require_approval_from_group hard-deny IN THE MERGE-GATE LAYER. Its OUT-OF-SCOPE: "AUTHZ-004 OWNS the merge-path undefined-group hard-deny now; GATES-008 is the deepening / standalone-proof (commit-gate + dedicated target-ref-only proofs), NOT a competing owner of the merge-path check." GATES-008 consumes this — it does NOT re-implement it.
2. crates/but-api/src/legacy/merge_gate.rs (40-110, 181-209)
   Focus: CONFIRM THE TARGET-REF READ ALREADY EXISTS — enforce_merge_gate loads load_merge_governance_config(&repo, &target_ref) (the TARGET-REF read), authorizes Merge, checks undefined_required_groups (AUTHZ-004's hard-deny), then calls review_requirement::evaluate. load_merge_governance_config + read_config_blob read the TARGET-REF tree blob via gix — never the working tree. GATES-008 proves a feature-head weakening is ignored BY this existing read; it modifies NONE of this.
3. crates/but-api/tests/merge_gate.rs (451-587, 650-668)
   Focus: THE HARNESS TO EXTEND — merge_gated_repo(GateConfig) seeds gates.toml at refs/heads/main, branches to feat, AND ALREADY commits an EMPTY gates.toml on the feat head (576 `: >.gitbutler/gates.toml`). GATES-008 makes the feat-head weakening EXPLICIT (a `FeatureHeadDropsRequirement` GateConfig committing a min_approvals=0 / no-[[gate]] gates.toml on feat while main keeps min_approvals=1) and asserts the gate STILL blocks per the target-ref requirement at zero approvals, then permits after one approval. Use approve_branch (650) for the governed approval, ref_id for the sha-unchanged assertion, assert_gate_denied for the code.
4. crates/but-api/tests/commit_gate.rs (100-160)
   Focus: THE COMMIT-GATE TARGET-REF-ONLY PROOF GATES-008 REFERENCES (does not duplicate) — commit_gate_edit_cannot_unprotect (:102) already proves a WORKING-TREE gates.toml edit (uncommitted) AND a FEATURE-HEAD committed gates.toml unprotect both fail to unprotect main on the COMMIT path (both → branch.protected, main HEAD sha == base). 8/8 commit_gate tests pass. GATES-008 cites this as full commit-gate target-ref-only coverage (R7 option a) and authors NO commit-gate target-ref test.
5. .spec/prds/governance/06-uc-gates.md (47, 51)
   Focus: UC-GATES-02 AC-6 (the merge gate reads its requirement from committed gates.toml at the target ref, so a head that edits the requirement to drop it cannot weaken its own gate) + AC-10 (the gate NEVER reads gates.toml from the working tree or the feature head — ONLY the target-ref config blob).
6. .spec/prds/governance/11-e2e-testing-criteria.md (120, 124)
   Focus: T-GATES-013 (requirement read at target ref — head drops requirement → judged by target-ref requirement) + T-GATES-019 (gate reads requirement ONLY from target-ref blob; working-tree/feature-head edits ignored). Both integration through the real merge gate.
7. crates/but-testsupport/src/lib.rs (71-97)
   Focus: writable_scenario + invoke_bash to seed the target-ref blob at refs/heads/main and a DIFFERENT committed gates.toml on feat; NEVER std::env::temp_dir().join(...). Mirror the existing merge_gated_repo invoke_bash shape (commit on main, checkout -b feat, commit a weaker gates.toml on feat, checkout main).

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- Merge-path target-ref-only discriminating standalone proof passes: `cargo test -p but-api merge_gate_targetref_only_feature_head_drop_ignored`  -> Exit 0; the feature-head requirement drop is ignored, the target-ref requirement binds at zero approvals (a feature-head-reading stub would permit), then proceeds after a target-ref-satisfying approval
- Commit-gate target-ref-only coverage referenced (NOT re-authored): GATES-001 `commit_gate_edit_cannot_unprotect` (commit_gate.rs:102, working-tree + feature-head, 8/8 pass) + GATES-007 `commit_gate_apply_integrate_dryrun_targetref_pinned` are the dedicated commit-gate proofs — GATES-008 adds none (R7 option a)
- NO competing merge-path classification change (DIRECT-DIFF form, S1): `./tools/governance-checks/check_merge_gate_production_unchanged.sh` -> Exit 0 (no production merge-gate diff from AUTHZ-004 baseline; AUTHZ-004 remains the owner)
- Reviews seeded via governed action only: `grep -qE 'approve_branch|forge::approve_review' crates/but-api/tests/merge_gate.rs` -> Match; `! grep -rEn 'local_review_verdicts_mut\(\)\.\.\.insert|\.upsert\(LocalReviewVerdict' crates/but-api/tests/merge_gate.rs` -> No direct-insert of a verdict (R6)
- Crate compiles incl. tests: `cargo check -p but-api --all-targets`  -> Exit 0
- Clippy clean: `cargo clippy -p but-api --all-targets`  -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: Discriminating standalone target-ref-only read proof — seed a repo whose TARGET REF (main) commits the STRONG requirement (min_approvals=1 distinct) and whose FEATURE HEAD (feat) commits a WEAKER gates.toml (no [[gate]] / min_approvals=0), then drive the real merge gate and assert it judges by the TARGET-REF version: (a) at zero approvals it BLOCKS gate.review_required (the target-ref binds; a feature-head-reading stub would PERMIT because the feature head's min_approvals=0 is vacuously satisfied — this is the discriminator), and (b) after one distinct governed approval it PERMITS (target-ref min_approvals=1 satisfied). The feature-head/working-tree edit is inert because the gate's loader reads only the target-ref blob (load_merge_governance_config(repo, target_ref)). GATES-008 adds NO production classification — it CONSUMES AUTHZ-004's already-landed merge-path fail-closed behavior and proves only the residual target-ref-only property as a dedicated discriminating case; the commit-gate target-ref-only property is referenced from GATES-001 (commit_gate_edit_cannot_unprotect) + GATES-007.
pattern_source: crates/but-api/src/legacy/merge_gate.rs (load_merge_governance_config reads the TARGET REF tree blob via gix, never the working tree) + crates/but-api/tests/merge_gate.rs:576 (the existing fixture already commits an empty gates.toml on feat — GATES-008 makes it an explicit weaker requirement and asserts it is ignored via the discriminating cycle) + crates/but-api/tests/commit_gate.rs:102 (commit_gate_edit_cannot_unprotect — the commit-gate target-ref-only precedent GATES-008 references)
anti_pattern: Re-implementing the merge-path fail-closed classification or the undefined-group hard-deny (AUTHZ-004 owns it — a competing owner that drifts); modifying merge_gate.rs production code (a standalone proof must not change the implementation it proves); reading the feature-head/working-tree gates.toml (the soundness hole the target-ref pin closes); a NON-discriminating positive-only proof that cannot distinguish a target-ref read from a feature-head read (both permit — the zero-approval block is the discriminator); authoring a DUPLICATE commit-gate target-ref test instead of referencing GATES-001 commit_gate_edit_cannot_unprotect (R7 option a); leaving a "skip if already covered" ambiguity rather than a deterministic reference; the INVERTED `git diff --name-only | grep -qvE '...merge_gate\.rs$'` build-gate that passes trivially whenever any test file changes (S1 — use the direct-diff form); seeding approvals via a direct local_review_verdicts insert (forgeable R6 path); asserting the forge-network merge completion locally; or asserting the forgeable direct-DB-write / raw-git bypass is blocked (false guarantee, R6/R1).

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — Proves, as a DEDICATED discriminating standalone test, that GitButler's merge gate reads its requirement ONLY from the committed target-ref gates.toml: a feature head that commits a weaker gates.toml dropping the requirement is ignored — at zero approvals the merge is blocked per the target-ref requirement (a feature-head-reading stub would permit), then permitted after one distinct governed approval satisfies the target-ref requirement @head. Consumes AUTHZ-004's already-landed merge-path fail-closed classification ENTIRELY AS-IS (no re-implementation, no competing owner; direct-diff build-gate), references GATES-001's commit_gate_edit_cannot_unprotect for the commit-gate target-ref-only property (no duplicate), seeds governed approvals via the real forge::approve_review path, and extends the existing merge_gate harness against real but-api + real git + real but-db.
reviewer: rust-reviewer
coding_standards: crates/AGENTS.md, crates/WORKSPACE_MODEL.md, crates/but-api/src/legacy/merge_gate.rs, .spec/prds/governance/tasks/sprint-02-authz-fail-closed-identity-confinement/AUTHZ-004-merge-gate-fail-closed.md

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: GATES-003, GATES-005, GATES-006, AUTHZ-004, GATES-001, GATES-007   (the merge gate wrapper + the evaluator + GATES-006's shared GateConfig enum extension — both extend the shared GateConfig in crates/but-api/tests/merge_gate.rs, serialize after GATES-006 + AUTHZ-004's fail-closed classification it consumes + the commit gate it references for the commit-gate target-ref-only property + GATES-007's commit-gate mechanism-coverage target-ref-pin it references for the shared commit_gate.rs/merge_gate.rs touch)
Blocks:     Sprint 05, Sprint 06b
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "GATES-008",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "notes": [
    "DE-CONFLICT (GATES-008 vs AUTHZ-004): AUTHZ-004 (Sprint 02) ALREADY LANDED the full merge-path fail-closed classification in crates/but-api/src/legacy/merge_gate.rs — deterministic config.invalid vs perm.denied vs gate.review_required, config-load-first caller-independent, unknown/no-handle deny, DryRun-no-bypass, AND the undefined-require_approval_from_group hard-deny. Its dedicated tests ALREADY PASS (merge_gate_malformed_config_is_config_invalid, merge_gate_undefined_required_group_denied, merge_gate_dryrun_unknown_failclosed_persists_nothing). GATES-008 is re-grounded to the RESIDUAL: the DEDICATED discriminating target-ref-only read proof (T-GATES-019). It is NOT a competing owner and adds NO production classification (AC-2 build-gate enforces this).",
    "S1 (CRITICAL, fixed): the AC-3 verify + AC-3 scenario must_observe previously used a logically-INVERTED command (git diff --name-only | grep -qvE '^...merge_gate.rs$') that passes trivially whenever ANY test file changes. BOTH are now the DIRECT-DIFF form: `! git diff -- crates/but-api/src/legacy/merge_gate.rs | grep -E '^\\+' | grep -iE 'undefined_required_groups|fn load_merge_governance_config|config_invalid'` (exit 0 / no matches = no added production classification). The (now AC-2) scenario must_observe/must_not_observe describe THAT check.",
    "S6 (fixed): AC-1's positive case alone cannot distinguish a target-ref read from a feature-head read (both permit). AC-1 is now DISCRIMINATING — the feature head drops the requirement to min_approvals=0/no [[gate]], so (a) zero approvals -> blocked gate.review_required (target-ref min_approvals=1 binds; a feature-head-reading stub would PERMIT because min_approvals=0 is vacuously satisfied), (b) one distinct governed approval -> PERMITTED (target-ref min_approvals=1 satisfied, NOT the feature-head min_approvals=0 trivially passing). The negative_control + must_observe note that a feature-head-reading stub permits at step (a).",
    "R3 (fixed): Depends on now includes GATES-006 (both extend the shared GateConfig enum in crates/but-api/tests/merge_gate.rs — GATES-008 serializes after GATES-006) AND GATES-007 is promoted from 'coordinates with' to a hard Depends on (shared commit_gate.rs/merge_gate.rs touch + GATES-008 references GATES-007's target-ref-pin).",
    "R7 (fixed, option a — DETERMINISTIC): the prior AC-2 'commit-gate target-ref-only standalone (or skip if already covered)' was ambiguous. RESOLVED deterministically: GATES-001's commit_gate_edit_cannot_unprotect (crates/but-api/tests/commit_gate.rs:102 — working-tree edit + feature-head committed unprotect, both -> branch.protected, 8/8 commit_gate tests pass) + GATES-007's target-ref-pin ARE the full commit-gate target-ref-only coverage. GATES-008 REFERENCES them and DROPS the commit-gate AC entirely. ACs renumbered AC-1..AC-2 (merge target-ref-only; no-competing-check build-gate) with no gaps; TCs renumbered TC-1..TC-4; requirements[] + maps_to_ac kept consistent; the dropped commit_targetref_only fixture is removed.",
    "Reviews seeded ONLY via the governed approve_branch helper (but-api forge::approve_review). NEVER a direct local_review_verdicts insert (R6). POSITIVE path proves the gate DECISION + reaching the forge call (forge completion out-of-local-scope, GATES-003/005 re-scope)."
  ],
  "fixtures": {
    "merge_targetref_only": {
      "description": "A real git repo (but-testsupport writable_scenario, extending the merge_gated_repo helper with a FeatureHeadDropsRequirement GateConfig) whose TARGET REF main commits .gitbutler/gates.toml with the STRONG requirement ([[branch]] main protected=true; [[gate]] branch=main type=review min_approvals=1 require_distinct_from_author=true) and .gitbutler/permissions.toml (impl=[contents:write,pull_requests:write,reviews:write] the author; reviewer=[reviews:write]; maint=[merge]); the FEAT HEAD commits a WEAKER .gitbutler/gates.toml (no [[gate]] entry / min_approvals=0) attempting to drop the requirement. An open governed review on feat (forge_reviews upsert author=impl, source=feat, target=main, sha=<feat head>) with ZERO qualifying approvals @head initially. The discriminator: a feature-head-reading gate would treat the feature head's min_approvals=0 as vacuously satisfied and PERMIT at zero approvals; the target-ref gate BLOCKS.",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = merge_gated_repo(GateConfig::FeatureHeadDropsRequirement); // NEW discriminant extending the existing helper (serializes after GATES-006's GateConfig extension)",
        "invoke_bash: write .gitbutler/permissions.toml (impl=[contents:write,pull_requests:write,reviews:write]; reviewer=[reviews:write]; maint=[merge]) + .gitbutler/gates.toml with the STRONG requirement ([[branch]] main protected=true; [[gate]] branch=main type=review min_approvals=1 require_distinct_from_author=true); git add -A && git commit -m \"governance config\" at refs/heads/main",
        "invoke_bash: git checkout -b feat; OVERWRITE .gitbutler/gates.toml with a WEAKER config (only [[branch]] main protected=false, NO [[gate]] entry — dropping the merge requirement, equivalent to min_approvals=0); echo feat >feat.txt; git add -A && git commit -m \"feat drops the requirement\"; git checkout main",
        "let ctx = context_with_review(&repo, ref_id(&repo, FEAT_REF)?)?; // open governed review, zero approvals @head",
        "positive phase: approve_branch(&ctx, \"reviewer\").await? — a distinct governed approval @head satisfying the TARGET-REF requirement (min_approvals=1 distinct-from-author impl)"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN merge_targetref_only (target ref main: min_approvals=1 distinct; feat HEAD commits a WEAKER gates.toml dropping the requirement to min_approvals=0) + zero qualifying approvals @head + BUT_AGENT_HANDLE=maint WHEN maint attempts the merge at zero approvals, then a distinct approval is added @head and the merge is re-attempted THEN the zero-approval attempt is blocked gate.review_required with a non-empty unmet[] — the feature-head drop is IGNORED, the target-ref min_approvals=1 distinct requirement binds (exit 1, trunk HEAD sha == base) and a feature-head-reading stub would have PERMITTED here (discriminating); after the distinct approval @head the gate is satisfied per the TARGET-REF requirement and the merge is PERMITTED (no gate.review_required — reaches the forge call), permitted because target-ref min_approvals=1 is met not because feature-head min_approvals=0 trivially passed",
      "verify": "cargo test -p but-api merge_gate_targetref_only_feature_head_drop_ignored",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api merge gate + real git + real but-db",
        "negative_control": {
          "would_fail_if": [
            "the gate reads the feature-head gates.toml so the dropped requirement weakens the gate and the merge proceeds with zero approvals (a feature-head-reading stub PERMITS at step a because the feature head's min_approvals=0 is vacuously satisfied)",
            "the requirement is read from the working tree rather than the target-ref blob (a mock config source)",
            "the gate is a no-op stub so the merge proceeds regardless of the target-ref requirement",
            "the requirement is treated as vacuously satisfied because the feature head removed the [[gate]] entry (a static empty-requirement pass)",
            "approvals are seeded by a direct local_review_verdicts insert rather than the governed approve_review action (forgeable R6 path)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "merge_targetref_only",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=maint: invoke the governed merge action on the open review while the feat HEAD's committed gates.toml drops the requirement (min_approvals=0) and ZERO qualifying approvals exist @head — a feature-head-reading stub would PERMIT here (cite T-GATES-019, T-GATES-013, UC-GATES-02 AC-6/AC-10)"
              ]
            },
            "end_state": {
              "must_observe": [
                "`error.code == \"gate.review_required\"` (the target-ref min_approvals=1 distinct requirement binds despite the feature-head drop; a feature-head-reading stub would have permitted)",
                "the `unmet[]` payload is non-empty (names the unmet approval shortfall, e.g. `no_approval`)",
                "process exits `1`",
                "the review is NOT merged (trunk/main HEAD sha `==` the seeded base sha)"
              ],
              "must_not_observe": [
                "merge proceeded",
                "exit `0`",
                "the feature-head gates.toml drop weakening the gate (the requirement treated as dropped/empty)",
                "an empty unmet (the requirement vacuously satisfied by the feature head's min_approvals=0)"
              ]
            }
          },
          {
            "start_ref": "merge_targetref_only",
            "action": {
              "actor": "cli_user",
              "steps": [
                "approve_branch(&ctx, \"reviewer\") — a distinct governed approval @head satisfying the TARGET-REF requirement (reviewer != author impl)",
                "BUT_AGENT_HANDLE=maint: re-invoke the governed merge action (cite T-GATES-013 — once the target-ref min_approvals=1 is met, it proceeds; NOT because the feature-head min_approvals=0 trivially passed)"
              ]
            },
            "end_state": {
              "must_observe": [
                "the gate PERMITS the merge — the output contains NO `error.code == \"gate.review_required\"` and NO `perm.denied` (0 governance denials: `classify_error(&err) == None`)",
                "execution reaches the governed `merge_review` body past the gate (any failure is a forge/remote error, NOT a governance Denial)"
              ],
              "must_not_observe": [
                "`error.code == \"gate.review_required\"` raised when the TARGET-REF requirement is satisfied @head",
                "the distinct approval @head wrongly rejected (0 governance denials expected)",
                "a governance Denial blocks the merge"
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
      "description": "GIVEN AUTHZ-004 owns the merge-path fail-closed classification (config.invalid/perm.denied/gate.review_required ordering + the undefined-required-group hard-deny) in merge_gate.rs production code WHEN this task's diff is structurally inspected with the DIRECT-DIFF form THEN this task modifies NO production classification logic in crates/but-api/src/legacy/merge_gate.rs (the change set is test files + the GateConfig fixture helper only) — AUTHZ-004 remains the owner, GATES-008 is a standalone proof, not a competing owner",
      "verify": "./tools/governance-checks/check_merge_gate_production_unchanged.sh",
      "maps_to_ac": null,
      "scenario": {
        "tier": "holdout",
        "test_tier": "unit",
        "unit_test_justified": "the ownership/no-duplication invariant (production classification unchanged, AUTHZ-004 remains owner) is a diff-scope/compile build-gate with zero runtime I/O; the behavioral target-ref-only property is proven by the AC-1 integration cases",
        "verification_service": "git diff scope + compile (build-gate, no runtime I/O)",
        "negative_control": {
          "would_fail_if": [
            "this task adds a competing undefined-group / malformed-config classification in merge_gate.rs production code (duplicating AUTHZ-004) — the direct-diff grep finds 1+ added lines",
            "this task modifies load_merge_governance_config or undefined_required_groups (changing the implementation it should only prove)",
            "the proof reaches green by re-implementing the fail-closed logic as a duplicate static/stub check rather than consuming AUTHZ-004's existing implementation",
            "the build-gate uses the INVERTED `git diff --name-only | grep -qvE '...merge_gate.rs$'` form that passes trivially whenever any test file changes (the prior S1 defect)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "merge_targetref_only",
            "action": {
              "actor": "ci",
              "steps": [
                "DIRECT-DIFF the merge_gate.rs production file to confirm GATES-008 adds NO competing classification logic: `! git diff -- crates/but-api/src/legacy/merge_gate.rs | grep -E '^\\+' | grep -iE 'undefined_required_groups|fn load_merge_governance_config|config_invalid'` (cite AUTHZ-004 OUT-OF-SCOPE: GATES-008 is the deepening, not a competing owner)"
              ]
            },
            "end_state": {
              "must_observe": [
                "`! git diff -- crates/but-api/src/legacy/merge_gate.rs | grep -E '^\\+' | grep -iE 'undefined_required_groups|fn load_merge_governance_config|config_invalid'` → `0` matches (no added production classification)",
                "the change set is restricted to test files (`crates/but-api/tests/merge_gate.rs`, optionally a CLI snapbox) + the GateConfig fixture helper"
              ],
              "must_not_observe": [
                "a non-empty production diff (1+ added lines where 0 / empty is required) re-implementing undefined_required_groups / load_merge_governance_config / config_invalid in merge_gate.rs production code",
                "a competing merge-path fail-closed check duplicating AUTHZ-004",
                "modification of the AUTHZ-004-owned classification logic",
                "the inverted `--name-only | grep -qvE` build-gate form (the prior S1 defect)"
              ]
            }
          }
        ]
      }
    },
    { "id": "TC-1", "type": "test_criterion", "description": "a feature-head gates.toml dropping the merge requirement is ignored; at zero approvals the target-ref requirement binds and blocks gate.review_required — a feature-head-reading stub would permit (T-GATES-019, T-GATES-013, UC-GATES-02 AC-6/AC-10)", "verify": "cargo test -p but-api merge_gate_targetref_only_feature_head_drop_ignored", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "after a distinct approval @head per the TARGET-REF requirement, the merge is permitted (the target-ref min_approvals=1, once satisfied, proceeds — NOT the feature-head min_approvals=0 trivially passing)", "verify": "cargo test -p but-api merge_gate_targetref_only_feature_head_drop_ignored", "maps_to_ac": "AC-1" },
    { "id": "TC-3", "type": "test_criterion", "description": "GATES-008 modifies NO production merge-path classification logic (merge_gate.rs production unchanged via direct-diff); AUTHZ-004 remains the owner (no duplicate undefined-group/malformed check)", "verify": "./tools/governance-checks/check_merge_gate_production_unchanged.sh", "maps_to_ac": "AC-2" },
    { "id": "TC-4", "type": "test_criterion", "description": "the proof is a DEDICATED discriminating standalone case (not a side-effect of another case) capturing concrete target-ref vs feature-head gates.toml before/after, with a zero-approval block that a feature-head-reading stub would not produce (T-GATES-019)", "verify": "cargo test -p but-api merge_gate_targetref_only_feature_head_drop_ignored", "maps_to_ac": "AC-1" }
  ]
}
-->
</details>
</content>
</invoke>
