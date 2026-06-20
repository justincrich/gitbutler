# GATES-003: Merge gate at the but-api forge seam covering BOTH `merge_review` AND `set_review_auto_merge` (authorize merge + target-ref review requirement @head, DryRun-enforced)

## What this does

At GitButler's PR-merge action (the but-api `legacy/forge` seam — the local trunk ref is immutable, so this is where a governed merge is initiated), gate BOTH async forge entry points identically before a change lands on a protected branch: resolve the acting principal from `BUT_AGENT_HANDLE` (fail closed), run a pre-call `authorize(merge)` guard BEFORE the `.await`, load the target-ref `.gitbutler/gates.toml` review requirement, evaluate it at the current head against `local_review_verdicts` (GATES-002) via the requirement evaluator (GATES-005), and proceed only on Ok. Gating `set_review_auto_merge` identically closes the auto-merge bypass.

## Why

Sprint 01b · PRD UC-GATES-02, UC-LOOP-01, UC-LOOP-02 · capabilities CAP-AUTHZ-01, CAP-CONFIG-01. The merge half of the walking skeleton — the T-LOOP-006 canary's merge step depends on this gate being enforcing on both forge merge entry points.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-api merge_gate_authorize_and_review_requirement` (integration, real forge seam + real git + real but-db). Full gate set in the spec below.

## Scope

- `crates/but-api/src/legacy/forge.rs` (MODIFY) — add the pre-call merge gate at the TOP of BOTH `merge_review` and `set_review_auto_merge`, before the inner `.await` (call `merge_gate::enforce_merge_gate`)
- `crates/but-api/src/legacy/merge_gate.rs` (NEW) — the merge-gate wrapper helper (resolve principal + target ref, `authorize(merge)`, load target-ref cfg, query `local_review_verdicts`, call the GATES-005 evaluator). OWNS the wrapper + basic min_approvals/at-head plumbing
- `crates/but-api/Cargo.toml` (MODIFY) — add `but-authz` (idempotent) + ensure `but-db`
- `crates/but-api/tests/merge_gate.rs` (NEW) — integration tests against the real forge seam + real git + real but-db
- `crates/but/src/command/legacy/forge/review.rs` (MODIFY) — surface the merge-gate Denial as the structured exit-1 contract at the CLI boundary
- `crates/but/tests/but/command/merge_gate.rs` (NEW) — CLI snapbox end-to-end denial/allow for `but pr auto-merge` + the governed merge verb

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: GATES-003 - Merge gate at the but-api forge seam covering BOTH merge_review AND set_review_auto_merge (authorize merge + target-ref review requirement @head, DryRun-enforced)
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Complete
PRIORITY:   P0
EFFORT:     L  (270 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-GATES-02, UC-LOOP-01, UC-LOOP-02
CAPABILITIES: CAP-AUTHZ-01, CAP-CONFIG-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-api merge_gate
  check: cargo check -p but-api --all-targets
  lint:  cargo clippy -p but-api --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Integration tests are green against the real but-api forge seam + real git + real but-db: a principal lacking `merge` is denied perm.denied naming merge on BOTH the merge_review path AND the set_review_auto_merge (auto-merge) path, even when a distinct approval exists at head; a `merge`-holder with a satisfied requirement at head is PERMITTED by the gate (authorize Ok + requirement satisfied → NO governance Denial raised; execution reaches the forge `merge_review` call — the forge-network merge COMPLETION is out of local-no-mocks scope, proven structurally); a merge below min_approvals is blocked gate.review_required with a non-empty unmet[]; the two-group requirement is parsed from the target-ref gates.toml and a merge with a distinct approval from each required group at head is PERMITTED by the gate; a DryRun merge by a non-merge principal still denies perm.denied + exit 1 and persists nothing; a malformed target-ref gates.toml denies config.invalid (not a vacuous pass). (Gate-boundary re-scope per the red-hat cycle + user decision: `merge_review`/`set_review_auto_merge` are forge-bound and error on a bare local repo, and there is no `but pr merge` CLI verb — so the POSITIVE path proves the gate DECISION locally and the forge completion structurally; the DENIAL paths are fully locally provable because the gate fires before the forge .await.)

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST gate BOTH merge_review (forge.rs:438) AND set_review_auto_merge (forge.rs:469) IDENTICALLY with the same authorize(merge) + review-requirement gate. An un-gated set_review_auto_merge was a red-hat CRITICAL: enabling auto-merge is a deferred merge — if not gated, auto-merge is an ungated bypass of the entire merge gate. Do NOT gate only merge_review.
- [MUST] MUST run authorize(&principal, Authority::Merge, &cfg) as a PRE-CALL guard BEFORE the inner .await — per 04-api-design.md these are async actions on ThreadSafeContext that CANNOT take a repo-permission param (but-api-macros:560); the pattern is principal_from_env → authorize()? → existing *_impl(...).await. NEVER use a _with_perm repo-permission param here.
- [MUST] MUST resolve the principal ONLY from BUT_AGENT_HANDLE via but_authz::resolve_principal against the committed GovConfig and fail closed (perm.denied) on unset/empty/unknown handle — never a default/anonymous principal (CAP-AUTHZ-01; AUTHZ-003).
- [MUST] MUST read the review requirement (min_approvals, require_distinct_from_author, require_approval_from_group[]) and branch protection ONLY from the TARGET-REF gates.toml blob via but_authz::config::load_governance_config(&repo, target_ref) — never the working tree or feature head, so a change whose head drops the requirement cannot weaken its own gate (CAP-CONFIG-01).
- [MUST] MUST run the gate EVEN under DryRun — a DryRun merge by a non-merge principal still returns perm.denied + exit 1; DryRun only suppresses persisting refs/objects/oplog (04-api-design.md; CAP-AUTHZ-01). Do NOT early-return on DryRun before the gate.
- [MUST] MUST evaluate the review requirement at the CURRENT head against local_review_verdicts (GATES-002 query) at a SINGLE call site — GATES-003 implements the BASIC min_approvals/at-head check inline at that call site (NO dependency on GATES-005, so the dependency graph is acyclic: GATES-002 -> GATES-003 -> GATES-005). GATES-005 (which depends on GATES-003) then provides the full review_requirement::evaluate (self-approval-exclusion + stale-@head-dismissal + per-group) and substitutes it at this call site. Do NOT implement the self/stale refinements here — leave the call site so GATES-005 can wire its evaluator in.
- [MUST] MUST treat the merge GATE DECISION (authorize(merge) + the review-requirement evaluation) as the locally-provable surface. The `merge_review` / `set_review_auto_merge` bodies themselves call `but_forge::derive_forge_repo_info(...)` and make a real GitHub/GitLab network call, which ERRORS on a bare local test repo (no remote) — so the POSITIVE path proves only that the gate PERMITS (returns Ok / raises NO Denial) and execution reaches the forge call past the gate; the forge-network merge COMPLETION (the change landing on the remote trunk) is NOT asserted locally (out of the no-mocks local scope; proven structurally / deferred to a forge-backed fixture). DENIAL paths are fully locally provable (the gate fires before the forge .await). See the red-hat re-scope note.
- [NEVER] NEVER overload GitButler's repo-access Permission/RepoExclusive lock as the authorization carrier — authorization is the orthogonal Authority axis (02-system-components.md; RULES.md lock discipline).
- [NEVER] NEVER add a test asserting the forgeable direct-DB-write path to local_review_verdicts is blocked, nor that a forge-UI/auto-merge-on-platform/raw-push is blocked — those are documented accepted-leaks (R6 High / R1); asserting them encodes a false guarantee. The gate binds the governed `but` merge action ONLY.
- [STRICTLY] STRICTLY surface the denial as the structured contract ({error:{code,message,remediation_hint}}, exit 1) with code ∈ {perm.denied, gate.review_required, config.invalid}; on gate.review_required the payload carries unmet[]. Carry the gate code as a but-authz/merge-gate-owned &'static str (mirroring Denial::PERM_DENIED_CODE) — do NOT add a but-error::Code variant unless the desktop frontend consumes it.
- [STRICTLY] STRICTLY add the but-authz workspace dep idempotently — GATES-001 (Sprint 01a) also adds it, and but-db is ALREADY a but-api dep (but-api/Cargo.toml:89); do not duplicate either.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: no-merge-perm denies BOTH entry points even when reviewed; a satisfied requirement + merge-holder proceeds
- [ ] AC-2: below min_approvals blocks gate.review_required with a non-empty unmet[]
- [ ] AC-3: two-group requirement parsed from target-ref gates.toml; both groups approve @head → merge proceeds
- [ ] AC-4: DryRun-no-bypass (perm.denied + persists nothing); malformed target-ref config → config.invalid
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: No `merge` perm denies even when reviewed (both entry points); a satisfied requirement + `merge`-holder proceeds [PRIMARY]
  GIVEN: fixture `merge_gated_repo` (main protected; gate min_approvals=1, require_distinct_from_author=true; impl=contents:write/no-merge, reviewer=reviews:write, maint=merge) with an open review whose head has a DISTINCT approving verdict (submitted via governed `but review`)
  WHEN:  a merge is attempted by impl (lacks merge) via merge_review; the same via set_review_auto_merge; then a merge by maint with the distinct approval present @head
  THEN:  both impl attempts denied perm.denied naming merge (exit 1, nothing merged); the maint merge is PERMITTED by the gate (authorize Ok + requirement satisfied → NO governance Denial; execution reaches the forge merge_review call — forge-network completion is structural/out-of-local-scope)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api forge seam + real git + real but-db
  VERIFY: cargo test -p but-api merge_gate_authorize_and_review_requirement
  SCENARIO: NEGATIVE_CONTROL would fail if the gate is a no-op stub so impl's merge proceeds; only merge_review is gated so set_review_auto_merge lets impl schedule an ungated merge; authorize is never called; authorize runs AFTER the .await; the satisfied-requirement merge by the holder is also blocked.

AC-2: Below min_approvals blocks with gate.review_required listing the unmet requirement
  GIVEN: fixture `merge_gated_repo` with ZERO approving verdicts at head (min_approvals=1), BUT_AGENT_HANDLE=maint (holds merge)
  WHEN:  a merge is attempted by maint
  THEN:  blocked error.code=="gate.review_required" with a non-empty unmet[] naming the shortfall, exit 1, nothing merged
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api forge seam + real git + real but-db
  VERIFY: cargo test -p but-api merge_gate_below_min_approvals_blocked
  SCENARIO: NEGATIVE_CONTROL would fail if the gate counts zero reviews as satisfying (vacuous pass); merge proceeds with 0 approvals; denial is not gate.review_required or carries an empty unmet[].

AC-3: Two-group requirement PLUMBING: parse require_approval_from_group; both groups approve @head → merge proceeds
  GIVEN: fixture `merge_gated_two_group` (target-ref gate require_approval_from_group=[code-reviewers,maintainers], both groups defined), BUT_AGENT_HANDLE=maint
  WHEN:  a merge with NO approvals (proves parse+enforce), then a merge with a distinct approval @head from a code-reviewers member AND a maintainers member
  THEN:  the no-approval merge is blocked gate.review_required naming the missing per-group approvals; the both-present merge is PERMITTED by the gate (NO governance Denial; execution reaches the forge merge_review call — forge completion structural)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api forge seam + real git + real but-db
  VERIFY: cargo test -p but-api merge_gate_two_group_both_present_proceeds
  SCENARIO: NEGATIVE_CONTROL would fail if require_approval_from_group is ignored/not parsed; group membership resolved from the working tree/feature head; the merge is blocked even with a distinct approval from each required group; a single approval counts for both groups.

AC-4: DryRun-no-bypass + fail-closed config
  GIVEN: fixtures `merge_gated_repo` (DryRun) and `merge_gated_malformed` (invalid target-ref gates.toml)
  WHEN:  a DryRun merge to protected main by impl (lacks merge); and a merge by maint against the malformed-config repo
  THEN:  the DryRun merge still perm.denied naming merge (exit 1) and persists nothing (no merge commit/ref, no verdict mutation); the malformed-config merge denies config.invalid (not a skip), exit 1
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api forge seam + real git + real but-db
  VERIFY: cargo test -p but-api merge_gate_dryrun_and_malformed_failclosed
  SCENARIO: NEGATIVE_CONTROL would fail if DryRun bypasses authorization; a denied DryRun persists a ref/object/verdict; malformed config is skipped (fail-open); malformed returns a non-config.invalid code.

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, happy_path): impl merge denied perm.denied (merge_review); impl auto-merge denied perm.denied (set_review_auto_merge); maint merge with distinct approval @head proceeds (T-GATES-008, T-LOOP-002, T-GATES-014)
    VERIFY: cargo test -p but-api merge_gate_authorize_and_review_requirement
- TC-2 (-> AC-2, error): below min_approvals blocked with gate.review_required + non-empty unmet[] (T-GATES-009)
    VERIFY: cargo test -p but-api merge_gate_below_min_approvals_blocked
- TC-3 (-> AC-3, edge): two-group requirement parsed from target-ref gates.toml (T-LOOP-007); both groups approve @head → merge proceeds (T-LOOP-010)
    VERIFY: cargo test -p but-api merge_gate_two_group_both_present_proceeds
- TC-4 (-> AC-4, error): DryRun merge by non-merge principal still perm.denied + persists nothing (CAP-AUTHZ-01); malformed target-ref config → config.invalid not skip
    VERIFY: cargo test -p but-api merge_gate_dryrun_and_malformed_failclosed
- TC-5 (-> AC-1, edge): the perm check (authorize merge) runs as a PRE-CALL guard BEFORE the inner .await on both forge entry points (04-api-design.md async-forge ordering)
    VERIFY: cargo test -p but-api merge_gate_authorize_and_review_requirement
- TC-6 (-> AC-3, edge): the review requirement is read ONLY from the TARGET-REF gates.toml blob (CAP-CONFIG-01)
    VERIFY: cargo test -p but-api merge_gate_two_group_both_present_proceeds

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01, CAP-CONFIG-01
provides: the merge gate on BOTH but-api forge merge entry points (merge_review + set_review_auto_merge): authorize(merge) + target-ref-pinned review-requirement @head, fail-closed, DryRun-enforced; perm.denied / gate.review_required / config.invalid codes surfaced at the forge boundary + exit 1; the single-required-group merge gate + the two-group requirement plumbing (parse + both-present-proceeds)
consumes: but_authz::authorize + Authority::Merge + resolve_principal (AUTHZ-003); but_authz::config::load_governance_config + GovConfig + ConfigError (AUTHZ-002); the local_review_verdicts query API (GATES-002); the review-requirement evaluator review_requirement::evaluate (GATES-005)
boundary_contracts:
  - CAP-AUTHZ-01: the merge AND auto-merge actions resolve BUT_AGENT_HANDLE→Principal and authorize(merge) at the async forge seam BEFORE the inner .await, EVEN under DryRun.
  - CAP-CONFIG-01: the review requirement + group membership are read ONLY from the target-ref gates.toml/permissions.toml blobs, so a head that drops the requirement cannot weaken its own gate.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-api/src/legacy/forge.rs (MODIFY) — add the pre-call merge gate at the TOP of BOTH merge_review and set_review_auto_merge, before the inner .await (call merge_gate::enforce_merge_gate)
  - crates/but-api/src/legacy/merge_gate.rs (NEW) — the merge-gate wrapper helper (OWNS the wrapper + basic min_approvals/at-head plumbing)
  - crates/but-api/Cargo.toml (MODIFY) — add but-authz (idempotent) + ensure but-db
  - crates/but-api/tests/merge_gate.rs (NEW) — integration tests against the real forge seam + real git + real but-db
  - crates/but/src/command/legacy/forge/review.rs (MODIFY) — surface the merge-gate Denial as the structured exit-1 contract at the CLI boundary (auto-merge + merge paths)
  - crates/but/tests/but/command/merge_gate.rs (NEW) — CLI snapbox end-to-end denial/allow for `but pr auto-merge` + the governed merge verb
writeProhibited:
  - crates/but-authz/** — CONSUME authorize/load_governance_config/Authority::Merge/resolve_principal/Denial; do NOT modify the primitive
  - crates/but-api/src/legacy/review_requirement.rs AND any self-approval-exclusion / stale-@head-dismissal / unmet-discriminator logic — OWNED by GATES-005; GATES-003 CALLS review_requirement::evaluate(...)
  - crates/but-db/src/table/local_review_verdicts.rs — OWNED by GATES-002; consume its query, do not redefine the table/migration
  - crates/but-error/src/lib.rs — do not add Code variants; carry the gate code as a but-authz/merge-gate-owned &'static str
  - crates/but-api/src/commit/** — the COMMIT gate is GATES-001
  - any gitbutler-* crate beyond what the action boundary strictly requires (crates/AGENTS.md)
  - Any file not listed in write_allowed

--------------------------------------------------------------------------------
OUT OF SCOPE
--------------------------------------------------------------------------------
  - T-GATES-012 / T-LOOP-008 / T-LOOP-009 (the only-ONE-group-blocked strictness matrix) DEFERRED to Sprint 04 (GATES-006). This task establishes the two-group requirement PLUMBING only (T-LOOP-007 parse + T-LOOP-010 both-present-proceeds).
  - T-GATES-018 (fail-closed on a require_approval_from_group naming a group UNDEFINED in target-ref config) DEFERRED to Sprint 04. Basic malformed-config fail-closed (config.invalid) IS in scope (AC-4).
  - T-GATES-013 / T-GATES-019 (dedicated standalone target-ref-only proofs) DEFERRED to Sprint 04. Basic target-ref read IS in scope (AC-3/TC-6).
  - Self-approval exclusion + stale-@head dismissal are GATES-005 (this gate CALLS that evaluator).
  - The merge gate binds the governed `but` merge action ONLY — forge-UI/auto-merge-on-platform/raw-push and the forgeable direct-DB-write are accepted-leaks (R6/R1), NOT tested.
  - FORGE-COMPLETION (red-hat re-scope, user decision = gate-boundary): the actual forge-network merge landing (the change appearing on the remote trunk) is NOT asserted locally — `merge_review`/`set_review_auto_merge` are forge-bound (`derive_forge_repo_info` errors on a bare repo) and there is no `but pr merge` CLI verb. The POSITIVE-path ACs prove the gate DECISION (permit, no Denial) on the real seam + that execution reaches the forge call; the forge completion is proven structurally / deferred to a forge-backed fixture. UPSTREAM ADVISORY: 07-uc-loop.md / 11-e2e-testing-criteria.md T-LOOP-006/T-LOOP-004 wording ("merge succeeds", "change lands") should be reconciled with this forge-locality via /kb-sprint-plan --delta-replan.

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-api/src/legacy/forge.rs (435-495)
   Focus: THE SEAM — merge_review (438) and set_review_auto_merge (469), both async on ThreadSafeContext. Each opens with `let ctx = ctx.into_thread_local(); let project_meta = ctx.project_meta()?; let repo = ctx.repo.get()?;` then but_forge::merge_review(...).await / set_review_auto_merge_state(...).await. Insert the pre-call gate at the TOP of each, BEFORE the .await. Cover BOTH.
2. crates/but/src/command/legacy/forge/review.rs (20-113)
   Focus: CLI seam — enable_auto_merge (20) calls but_api::legacy::forge::set_review_auto_merge(ctx.to_sync(), ...).await? (83); the merge path likewise routes through but-api. Surface the Denial as the structured exit-1 contract here.
3. crates/but/src/lib.rs (1229-1335)
   Focus: CLI dispatch — Subcommands::Pr(forge::pr::Platform{..}) routes AutoMerge{selector,off} → command::legacy::forge::review::enable_auto_merge (1310). The CLI verb path the auto-merge gate is exercised through.
4. crates/but-authz/src/authority.rs (11-110) + lib.rs (1-10)
   Focus: CONSUME-ONLY — Authority::Merge (27); Authority::name()/parse(). AUTHZ-002 adds but_authz::config::{load_governance_config, GovConfig, ConfigError}; AUTHZ-003 adds but_authz::{authorize, effective_authority, resolve_principal}.
5. .spec/prds/governance/tasks/sprint-01a-authz-primitive-commit-gate/AUTHZ-002-ref-pinned-config-loader.md (5-60) + AUTHZ-003-authorize-handle-resolution.md (5-58)
   Focus: the loader + authorize + handle-resolution contracts this task consumes (load_governance_config(&repo, target_ref) -> Result<GovConfig, ConfigError>; authorize(&Principal, Authority, &GovConfig) -> Result<(), Denial>; resolve_principal from BUT_AGENT_HANDLE, fail-closed).
6. crates/but-db/src/table/forge_reviews.rs (1-100)
   Focus: migration + handle PATTERN to mirror — GATES-002 adds the sibling local_review_verdicts table; GATES-003 CONSUMES GATES-002's query API (do not redefine). forge_reviews is a disposable remote cache (DELETE FROM forge_reviews at :153) and is NOT the approvals store.
7. .spec/prds/governance/10-technical-requirements/04-api-design.md (30-76)
   Focus: the async-forge gate shape (b) — pre-call authorize() before .await (NOT a repo-permission param); the two-gate entry-point table (merge gate → legacy/forge::merge_review / set_review_auto_merge, Authority merge + review requirement, fail codes); the route→Authority table (merge for both merge + auto-merge).
8. crates/but/tests/but/utils.rs (14-155) + crates/but-testsupport/src/lib.rs (71-97,432-441)
   Focus: CLI test harness — Sandbox + env.but("pr auto-merge ...") snapbox Command with .env("BUT_AGENT_HANDLE", "impl"), .assert(), .stdout_eq/.stderr_eq + [..]/... wildcards + exit codes; writable_scenario + invoke_bash to seed a real repo (config at refs/heads/main, branch to feat, advance the feature head). Keep _tmp alive.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- Merge-gate integration tests pass (both forge entry points): `cargo test -p but-api merge_gate`  -> Exit 0; AC-1..4 green
- CLI end-to-end denial/allow passes: `cargo test -p but merge_gate`  -> Exit 0; snapbox stderr_eq + exit codes match
- Crates compile: `cargo check -p but-api -p but --all-targets`  -> Exit 0
- BOTH forge merge entry points gated (no auto-merge bypass) — FUNCTION-SCOPED (a file-wide count is INSUFFICIENT: it passes with two guards in one fn and none in the other, re-opening the auto-merge bypass): `grep -A 40 'fn merge_review' crates/but-api/src/legacy/forge.rs | grep -c enforce_merge_gate`  -> >= 1  AND  `grep -A 40 'fn set_review_auto_merge' crates/but-api/src/legacy/forge.rs | grep -c enforce_merge_gate`  -> >= 1 (the guard appears in EACH function body)
- Gate does not read working-tree config: `! grep -rEn 'workdir|std::fs::read|read_to_string' crates/but-api/src/legacy/merge_gate.rs`  -> No matches
- No Permission-lock overload: `! grep -rEn 'write_permission\(|RepoExclusive|exclusive_worktree_access' crates/but-api/src/legacy/merge_gate.rs`  -> No matches
- No role-name in the merge gate: `! grep -rEni 'implementer|reviewer|maintainer' crates/but-api/src/legacy/merge_gate.rs`  -> No matches (T-LOOP-005 family)
- Clippy clean: `cargo clippy -p but-api -p but --all-targets`  -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: Pre-call-authorize-then-requirement-evaluate at the async forge seam — a small enforce_merge_gate(ctx, review_id) -> Result<(), Denial> called at the TOP of BOTH merge_review and set_review_auto_merge, BEFORE the inner .await; it resolves the principal (BUT_AGENT_HANDLE), runs authorize(merge), loads the target-ref gates.toml requirement, then delegates the @head review evaluation to GATES-005's review_requirement::evaluate; the Denial is propagated as an anyhow error carrying the classification and surfaced as exit-1 structured JSON at the CLI.
pattern_source: crates/but-api/src/legacy/forge.rs:438 (merge_review) + :469 (set_review_auto_merge) — the async ThreadSafeContext forge seam; 04-api-design.md async-forge gate shape (b)
anti_pattern: Gating only merge_review so set_review_auto_merge schedules an ungated merge (the red-hat CRITICAL auto-merge bypass); using a _with_perm repo-permission param on a ThreadSafeContext (rejected by but-api-macros:560); early-returning on DryRun before the gate; reading the requirement/group membership from the working tree or feature head; re-implementing the self/stale evaluator here instead of calling GATES-005; or asserting the forgeable direct-DB-write / forge-UI / raw-push bypass is blocked (false-guarantee, R6/R1).

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — Wires the authz primitive into GitButler's REAL async forge merge chokepoints — merge_review AND set_review_auto_merge — running a pre-call authorize(merge) guard before the .await, loading the target-ref review requirement, and evaluating it @head against the real but-db local_review_verdicts store. Owns the async-forge gate composition, the both-entry-points discipline (no auto-merge bypass), the target-ref-pin, the DryRun-no-bypass property, and integration TDD with but-testsupport + the CLI snapbox harness.
reviewer: rust-reviewer
coding_standards: crates/AGENTS.md, crates/WORKSPACE_MODEL.md, crates/but/AGENTS.md, crates/but-api/src/legacy/forge.rs

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: GATES-002, AUTHZ-002, AUTHZ-003   (one-directional — GATES-003 does NOT depend on GATES-005)
Blocks:     GATES-005, LOOP-001
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "GATES-003",
  "proposed_by": "rust-planner",
  "verification_policy": { "requires_tests": true, "requires_red_evidence": true, "requires_seeded_evidence": true },
  "fixtures": {
    "merge_gated_repo": {
      "description": "A real git repo (but-testsupport writable_scenario) whose target ref main has committed .gitbutler/permissions.toml (impl=[contents:write,pull_requests:write]; reviewer=[reviews:write]; maint=[merge]) and .gitbutler/gates.toml ([[branch]] main protected=true; [[gate]] branch=main type=review min_approvals=1 require_distinct_from_author=true), plus an open review on a feature branch whose head carries (or, per case, lacks) a distinct approving row in local_review_verdicts (GATES-002).",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"governance-base\");",
        "invoke_bash: write .gitbutler/permissions.toml ([[principal]] id=\"impl\" permissions=[\"contents:write\",\"pull_requests:write\"]; [[principal]] id=\"reviewer\" permissions=[\"reviews:write\"]; [[principal]] id=\"maint\" permissions=[\"merge\"])",
        "invoke_bash: write .gitbutler/gates.toml ([[branch]] name=\"main\" protected=true; [[gate]] branch=\"main\" type=\"review\" min_approvals=1 require_distinct_from_author=true)",
        "invoke_bash: git add -A && git commit -m \"governance config\" (commits both blobs at refs/heads/main); git checkout -b feat; commit a change so the feature head differs from main",
        "open a governed review on feat via the but-api forge path; seed local_review_verdicts (GATES-002) per case: a distinct approving verdict from reviewer at the feat head, or none"
      ]
    },
    "merge_gated_two_group": {
      "description": "Same shape as merge_gated_repo but the target-ref permissions.toml defines two groups ([[group]] code-reviewers permissions=[reviews:write]; [[group]] maintainers permissions=[merge,reviews:write]) with reviewer-a in code-reviewers and reviewer-b + maint in maintainers, and the target-ref gates.toml gate sets require_approval_from_group=[code-reviewers,maintainers].",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"governance-base\");",
        "invoke_bash: write permissions.toml with the two groups + members (reviewer-a->code-reviewers; reviewer-b,maint->maintainers) and gates.toml gate require_approval_from_group=[code-reviewers,maintainers] min_approvals=1 require_distinct_from_author=true; git add -A && git commit at refs/heads/main; git checkout -b feat; commit a change",
        "open a governed review on feat; per case seed local_review_verdicts (GATES-002) with none, or with a distinct approving verdict @head from a code-reviewers member AND a distinct approving verdict @head from a maintainers member"
      ]
    },
    "merge_gated_malformed": {
      "description": "Same as merge_gated_repo but the target-ref .gitbutler/gates.toml blob is committed with invalid TOML — to prove the merge fails closed config.invalid.",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"governance-base\");",
        "invoke_bash: write a valid permissions.toml (maint=[merge]) and a BROKEN gates.toml `[[gate] branch = \"main\" min_approvals = nope`; git add -A && git commit at refs/heads/main; git checkout -b feat; commit a change; open a governed review"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN merge_gated_repo with a distinct approval @head WHEN impl (no merge) attempts merge_review, then impl attempts set_review_auto_merge, then maint (merge) attempts the merge THEN both impl attempts perm.denied naming merge (exit 1, nothing merged), maint merge proceeds (Ok, exit 0, trunk advances)",
      "verify": "cargo test -p but-api merge_gate_authorize_and_review_requirement",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api forge seam + real git + real but-db",
        "negative_control": { "would_fail_if": [
          "the gate is a no-op stub so impl's merge proceeds",
          "only merge_review is gated so set_review_auto_merge (auto-merge) lets impl schedule an ungated merge",
          "authorize is never called on the forge entry point",
          "authorize runs AFTER the inner .await (post-merge) rather than as a pre-call guard",
          "the satisfied-requirement merge by the merge-holder is also blocked (gate too strict / requirement read from the wrong ref)"
        ] },
        "evidence": { "artifact_type": "stdout", "required_capture": true },
        "cases": [
          {
            "start_ref": "merge_gated_repo",
            "action": { "actor": "cli_user", "steps": ["BUT_AGENT_HANDLE=impl: invoke the governed merge action on the open review (the merge_review path) — impl holds contents:write but NOT merge, even though a distinct approval exists @head"] },
            "end_state": {
              "must_observe": ["`error.code == \"perm.denied\"`", "message names `\"merge\"`", "process exits `1`", "the review is NOT merged (trunk/main HEAD sha `==` the seeded base sha)"],
              "must_not_observe": ["merge proceeded", "exit `0`", "`gate.review_required` (the perm check must fire FIRST, before the review requirement)"]
            }
          },
          {
            "start_ref": "merge_gated_repo",
            "action": { "actor": "cli_user", "steps": ["BUT_AGENT_HANDLE=impl: `but pr auto-merge` to ENABLE auto-merge on the open review (the set_review_auto_merge path)"] },
            "end_state": {
              "must_observe": ["`error.code == \"perm.denied\"` naming `\"merge\"`", "process exits `1`", "auto-merge was NOT enabled (0 auto-merge state set)"],
              "must_not_observe": ["auto-merge enabled", "exit `0`", "set_review_auto_merge ran ungated"]
            }
          },
          {
            "start_ref": "merge_gated_repo",
            "action": { "actor": "cli_user", "steps": ["BUT_AGENT_HANDLE=maint: invoke the governed merge action on the open review whose head already carries a DISTINCT approving verdict from a reviews:write principal (submitted via governed `but review`) — maint holds merge (gate-boundary re-scope: assert the gate PERMITS; forge-network completion is out of local scope)"] },
            "end_state": {
              "must_observe": ["the gate PERMITS the merge — `authorize(merge)` returns `Ok` and the requirement is satisfied: the output contains NO `error.code == \"perm.denied\"` and NO `error.code == \"gate.review_required\"` (0 governance denials)", "execution reaches the governed `merge_review` body past the gate (any failure is a forge/remote error such as `No forge could be determined`, NOT a governance Denial)"],
              "must_not_observe": ["`error.code == \"perm.denied\"` raised for the satisfied, merge-holding request", "`error.code == \"gate.review_required\"` raised when the distinct approval is present @head", "a governance Denial blocks the merge (0 governance denials expected for the authorized, satisfied request)"]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN merge_gated_repo with ZERO approvals @head (min_approvals=1) WHEN maint (merge) attempts the merge THEN blocked gate.review_required with a non-empty unmet[] naming the shortfall, exit 1, nothing merged",
      "verify": "cargo test -p but-api merge_gate_below_min_approvals_blocked",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api forge seam + real git + real but-db",
        "negative_control": { "would_fail_if": [
          "the gate counts any/zero reviews as satisfying (vacuous pass)",
          "the merge proceeds with 0 approvals",
          "the denial is not gate.review_required or carries an empty/absent unmet[]",
          "the requirement is read from the working tree instead of the target ref (a no-op pin)"
        ] },
        "evidence": { "artifact_type": "stdout", "required_capture": true },
        "cases": [
          {
            "start_ref": "merge_gated_repo",
            "action": { "actor": "cli_user", "steps": ["seed local_review_verdicts with NO approving row for the review's target @head (0 rows)", "BUT_AGENT_HANDLE=maint: invoke the governed merge action (cite T-GATES-009)"] },
            "end_state": {
              "must_observe": ["`error.code == \"gate.review_required\"`", "the `unmet[]` payload is non-empty and names the min_approvals shortfall", "process exits `1`", "the review is NOT merged (trunk/main HEAD sha `==` base)"],
              "must_not_observe": ["merge proceeded", "exit `0`", "an empty `unmet[]`", "`perm.denied` (maint DOES hold merge — the requirement check must be what fails)"]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN merge_gated_two_group (require_approval_from_group=[code-reviewers,maintainers]) WHEN maint attempts a merge with no approvals (parse proof), then with a distinct approval @head from a code-reviewers member AND a maintainers member THEN no-approval blocked gate.review_required naming the per-group shortfall; both-present proceeds (Ok)",
      "verify": "cargo test -p but-api merge_gate_two_group_both_present_proceeds",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api forge seam + real git + real but-db",
        "negative_control": { "would_fail_if": [
          "require_approval_from_group is ignored / not parsed from the target-ref gates.toml (a no-op parse)",
          "group membership is resolved from the working tree or feature head rather than the target-ref permissions.toml",
          "the merge is blocked even with a distinct approval present from each required group (plumbing too strict)",
          "the gate is hardcoded/stubbed to satisfied so a single approval counts as satisfying both groups"
        ] },
        "evidence": { "artifact_type": "stdout", "required_capture": true },
        "cases": [
          {
            "start_ref": "merge_gated_two_group",
            "action": { "actor": "cli_user", "steps": ["BUT_AGENT_HANDLE=maint: invoke the governed merge when require_approval_from_group=[code-reviewers,maintainers] but 0 approvals exist (proves T-LOOP-007 parse: the requirement is read and enforced)"] },
            "end_state": {
              "must_observe": ["`error.code == \"gate.review_required\"`", "the `unmet[]` payload names the missing per-group approval(s)", "process exits `1`"],
              "must_not_observe": ["merge proceeded", "exit `0`", "the two-group requirement silently ignored (empty unmet)"]
            }
          },
          {
            "start_ref": "merge_gated_two_group",
            "action": { "actor": "cli_user", "steps": ["via governed `but review approve`, record a distinct approving verdict @head from a code-reviewers member AND from a maintainers member", "BUT_AGENT_HANDLE=maint: invoke the governed merge action (cite T-LOOP-010; gate-boundary re-scope: assert the gate PERMITS)"] },
            "end_state": {
              "must_observe": ["the gate PERMITS the merge — both required groups satisfied @head: the output contains NO `error.code == \"gate.review_required\"` and NO `error.code == \"perm.denied\"` (0 governance denials)", "execution reaches the governed `merge_review` body past the gate (any failure is a forge/remote error, NOT a governance Denial)"],
              "must_not_observe": ["`error.code == \"gate.review_required\"` raised when both required groups have a distinct approval @head", "a governance Denial blocks the merge (0 governance denials expected)", "merge blocked despite both groups approving @head"]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN merge_gated_repo (DryRun) and merge_gated_malformed WHEN impl runs a DryRun merge to protected main, and maint merges against the malformed-config repo THEN the DryRun still perm.denied naming merge (exit 1) and persists nothing; the malformed-config merge denies config.invalid (not a skip), exit 1",
      "verify": "cargo test -p but-api merge_gate_dryrun_and_malformed_failclosed",
      "maps_to_ac": null,
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api forge seam + real git + real but-db",
        "negative_control": { "would_fail_if": [
          "DryRun bypasses authorization so the perm.denied does not fire",
          "a denied DryRun persists a ref/object/verdict mutation (state not unchanged)",
          "malformed target-ref config is skipped (treated as no requirement) and the merge proceeds (fail-open, a no-op gate)",
          "malformed config returns a non-config.invalid code or panics instead of the structured contract"
        ] },
        "evidence": { "artifact_type": "stdout", "required_capture": true },
        "cases": [
          {
            "start_ref": "merge_gated_repo",
            "action": { "actor": "cli_user", "steps": ["BUT_AGENT_HANDLE=impl: attempt a DryRun merge to protected main (impl lacks merge) (cite CAP-AUTHZ-01 DryRun-enforced)"] },
            "end_state": {
              "must_observe": ["`error.code == \"perm.denied\"` naming `\"merge\"`", "process exits `1`", "no merge commit/ref persisted AND no local_review_verdicts mutation from this attempt (trunk/main HEAD sha `==` base, unchanged)"],
              "must_not_observe": ["exit `0`", "trunk/main sha advanced", "a persisted merge ref/object from the denied dry run", "DryRun skipped the authorization check"]
            }
          },
          {
            "start_ref": "merge_gated_malformed",
            "action": { "actor": "cli_user", "steps": ["BUT_AGENT_HANDLE=maint: invoke the governed merge with a malformed target-ref .gitbutler/gates.toml (cite UC-AUTHZ-04 family)"] },
            "end_state": {
              "must_observe": ["`error.code == \"config.invalid\"`", "process exits `1`", "the review is NOT merged (trunk/main HEAD sha `==` base)"],
              "must_not_observe": ["merge proceeded", "exit `0`", "the requirement treated as satisfied", "the malformed config silently skipped (empty/no requirement)"]
            }
          }
        ]
      }
    },
    { "id": "TC-1", "type": "test_criterion", "description": "impl merge denied perm.denied (merge_review); impl auto-merge denied perm.denied (set_review_auto_merge); maint merge with distinct approval @head proceeds (T-GATES-008, T-LOOP-002, T-GATES-014)", "verify": "cargo test -p but-api merge_gate_authorize_and_review_requirement", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "below min_approvals blocked with gate.review_required + non-empty unmet[] (T-GATES-009)", "verify": "cargo test -p but-api merge_gate_below_min_approvals_blocked", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "two-group requirement parsed from target-ref gates.toml (T-LOOP-007); both groups approve @head -> merge proceeds (T-LOOP-010)", "verify": "cargo test -p but-api merge_gate_two_group_both_present_proceeds", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "DryRun merge by non-merge principal still perm.denied + persists nothing (CAP-AUTHZ-01); malformed target-ref config -> config.invalid not skip", "verify": "cargo test -p but-api merge_gate_dryrun_and_malformed_failclosed", "maps_to_ac": "AC-4" },
    { "id": "TC-5", "type": "test_criterion", "description": "the perm check (authorize merge) runs as a PRE-CALL guard BEFORE the inner .await on both forge entry points (04-api-design.md async-forge ordering)", "verify": "cargo test -p but-api merge_gate_authorize_and_review_requirement", "maps_to_ac": "AC-1" },
    { "id": "TC-6", "type": "test_criterion", "description": "the review requirement is read ONLY from the TARGET-REF gates.toml blob (CAP-CONFIG-01)", "verify": "cargo test -p but-api merge_gate_two_group_both_present_proceeds", "maps_to_ac": "AC-3" }
  ]
}
-->
</details>
