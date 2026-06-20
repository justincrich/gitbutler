# GRPS-002: Ref-pin governed membership + self-grant-inert: target-ref-only read so a feature head can never authorize its own merge

## What this does

Proves that group membership, group grants, and self-grants are read ONLY from the committed target-ref config blob — never the working tree or the feature head being merged. The PRIMARY proof composes the Sprint-01b merge gate (crates/but-api/src/legacy/merge_gate.rs::enforce_merge_gate, which already authorizes Authority::Merge against load_merge_governance_config(repo, target_ref) + resolve_principal_from_env at line 48 BEFORE the protected-branch/review-requirement branch at 50-93): a feature head that adds its own author to a merge-holding `maintainers` group is STILL denied the merge at the Authority::Merge step, because the authorizing membership is the target-ref version where the author is not a member. It also proves self-granting administration:write on a feature head is inert until that change lands on the target ref, and that committing the membership to the target ref and advancing it removes the perm.denied at the merge-authority step (the same merge clears the Authority::Merge gate — its residual gate.review_required is the expected NEXT gate, not a regression).

## Why

Sprint 03 · PRD UC-GRPS-02 · CAP-CONFIG-01. Serves the gate clause [GRPS-02] (a feature head that adds its own author to a merge-holding group — or self-grants administration:write — is still denied, because membership and grants are read only at the target ref). This is the anti-self-escalation invariant that makes GitButler a trustworthy policy layer: a change can never grant itself authority or weaken its own gate.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-api self_add_to_maintainers_on_feature_head_still_denied_merge` (Feature head self-adding its author to a merge-holding group is still denied merge at the Authority::Merge step (target-ref membership governs)). Full gate set in the spec below.

## Scope

- crates/but-api/tests/merge_gate_self_escalation.rs (NEW) — the GRPS-002 PRIMARY proof: feature-head self-add to maintainers => merge denied at the Authority::Merge step; landed membership => merge-authority cleared (composes legacy::merge_gate::enforce_merge_gate read-only; seeds ForgeReview inline via forge_reviews_mut().upsert per merge_gate.rs:403-434)
- crates/but-authz/tests/grps_ref_pin.rs (NEW) — the but-authz-layer self-grant-inert + target-ref-only membership-read proofs (AC-3, AC-4) using the injected-lookup resolve_principal variant
- crates/but-authz/tests/fixtures/scenario/governance-base.sh (MODIFY, ONLY IF a shared helper is genuinely needed) — prefer per-test invoke_bash seeding over editing the shared fixture; if edited, keep it behavior-neutral for existing tests

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: GRPS-002 - Ref-pin governed membership + self-grant-inert: target-ref-only read so a feature head can never authorize its own merge
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     L  (210 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-GRPS-02
CAPABILITIES: CAP-CONFIG-01, CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-api self_add_to_maintainers_on_feature_head_still_denied_merge   |   cargo test -p but-api landed_membership_clears_merge_authority_step   |   cargo test -p but-authz self_grant_admin_inert_until_landed   |   cargo test -p but-authz membership_read_only_from_target_ref   |   cargo test -p but-authz invariant_build_gates
  check: cargo check -p but-authz --all-targets
  lint:  cargo clippy -p but-authz --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Integration tests against real git prove: (1) a feature head self-adding its author to a merge-holding maintainers group is denied merge (perm.denied at the Authority::Merge step) because membership is the target-ref version; (2) once that membership commits to refs/heads/main and main advances, the same merge CLEARS the Authority::Merge step (HEAD-sha-advanced precondition asserted; the residual gate.review_required from the unapproved review gate is the expected next gate, NOT a regression); (3) self-granting administration:write on a feature head is inert — an administration:write authorization is denied until the grant lands on the target ref; (4) the membership read provably comes from the target-ref blob, not the feature head (the feature-head blob carries the membership but is ignored). The merge gate (legacy/merge_gate.rs) is consumed read-only; the PRIMARY driver seeds the ForgeReview INLINE via the public but-db API (forge_reviews_mut().upsert) + but_ctx::Context::from_repo().with_memory_app_cache(). cargo test -p but-api and -p but-authz green; clippy clean.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST authorize the merge against the principal's TARGET-REF effective set only. The proof composes enforce_merge_gate (which calls load_merge_governance_config(repo, branch_ref(target_branch)) then resolve_principal_from_env + authorize(Authority::Merge) at line 48). The feature/source head MUST carry the self-added membership in its own tree, yet the merge MUST still be denied at the Authority::Merge step because the target-ref blob governs.
- [MUST] MUST seed the ForgeReview INLINE in the PRIMARY (AC-1/AC-2) enforce_merge_gate driver using the public but-db API demonstrated at crates/but-api/tests/merge_gate.rs:403-434: `but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache()` then `ctx.db.get_cache_mut()?.forge_reviews_mut()?.upsert(ForgeReview { number, target_branch:"main", source_branch:"feat", author:Some("feat-author"), sha:<source head>, ... })`. This is the REQUIRED AC-1 driver and is achievable today — there is NO blocker.
- [MUST] MUST assert a concrete HEAD-advancement precondition in the AC-2 positive case: capture refs/heads/main sha BEFORE landing the membership and AFTER; assert they differ (sha_before != sha_after), and that the SAME merge that was DENIED at the Authority::Merge step now CLEARS that step (classify_error(err).code != "perm.denied"). Name the residual gate.review_required (from the seeded unapproved review gate) as the EXPECTED next gate — do NOT claim the merge returns Ok.
- [MUST] MUST scope AC-2 to the merge-authority step: the AC-1 perm.denied is GONE after landing; the residual gate.review_required is the expected next gate (mirror the authorized-case assertion pattern at crates/but-api/tests/merge_gate.rs:135-145, which uses expect_err + classify_error(...).is_none() OR a non-perm.denied code, because the seeded review gate has min_approvals=1 with no distinct approval).
- [NEVER] NEVER read group membership, group definitions, or grants from the working tree or the feature/source head — only from the committed target-ref config blob. The self-escalation case is the failure that a head-read would (wrongly) allow; the target-ref read is what prevents it.
- [NEVER] NEVER assert AC-2 returns Ok / 'the same merge is authorized' — enforce_merge_gate authorizes Authority::Merge (line 48) BEFORE the protected-branch + review-requirement branch (50-93); the seeded review gate (min_approvals=1, no approval) returns gate.review_required AFTER the merge authority passes. AC-2 asserts the merge-AUTHORITY step is cleared, not end-to-end Ok.
- [NEVER] NEVER branch on role names (read/triage/write/maintain/admin) or human-vs-AI predicates in any enforcement path. `maintainers`/`code-reviewers` are fixture DATA only. The invariant_build_gates honesty grep covers authorize.rs/config.rs/commit/gate.rs (NOT merge_gate.rs); since GRPS-002 writes only NEW test files and modifies no enforcement source, the grep is a guard-rail that stays green, not a grep over merge_gate.rs.
- [NEVER] NEVER drive a `but group create`/`grant`/`add-member` CLI verb — those do not exist until Sprint 05 (CLI-002). Prove membership/self-grant by committing [[group]]/[[principal]] TOML to refs at the git layer and exercising the loader + merge gate.
- [STRICTLY] STRICTLY treat the merge gate (crates/but-api/src/legacy/merge_gate.rs) as a CONSUMED Sprint-01b seam (GATES-003). Carry a CONSUME-only note: there is NO persisted `but group`/admin-config write path in this sprint (Sprint 05 owns it), so the self-grant-inert and protected-branch-admin-gated proofs assert the AUTHORIZATION behavior (load + authorize denies) against committed-config/feature-head states, not a CLI write.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: Feature head self-adding its author to a merge-holding group is still denied merge at the Authority::Merge step (target-ref membership governs)
- [ ] AC-2: Once the membership lands on the target ref and main advances, the same merge clears the Authority::Merge step
- [ ] AC-3: Self-granted administration:write on a feature head is inert until landed on the target ref
- [ ] AC-4: Membership is provably read from the target-ref blob, not the feature head, and a working-tree edit has no authorization effect
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: Feature head self-adding its author to a merge-holding group is still denied merge at the Authority::Merge step (target-ref membership governs) [PRIMARY]
  GIVEN: refs/heads/main where `maintainers` (permissions=[merge]) does NOT include feat-author, and a feature/source head `feat` whose committed permissions.toml ADDS feat-author to maintainers, with the ForgeReview seeded inline via forge_reviews_mut().upsert (target_branch=main, source_branch=feat, author=feat-author)
  WHEN:  the merge from feat into main is authorized via enforce_merge_gate(&ctx, review_id), with BUT_AGENT_HANDLE=feat-author set via temp_env::async_with_vars under #[serial_test::serial]
  THEN:  the merge is DENIED at the Authority::Merge step (line 48) — classify_error yields code=="perm.denied" and a message naming "merge" — because the authorizing membership is the target-ref version (feat-author not a member); the feature-head membership is ignored
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api merge gate + real but-authz + real gix repo + committed target-ref vs feature-head TOML + inline-seeded forge-review cache
  VERIFY: cargo test -p but-api self_add_to_maintainers_on_feature_head_still_denied_merge
  SCENARIO (negative controls): would pass (wrongly authorize) if authorize read the feature/source head config where feat-author IS in maintainers — the target-ref read is exactly what makes this DENY; a head-read would clear the Authority::Merge step and fail this assertion; would pass against a stub merge gate that always returns Ok / never resolves the principal; would pass against an empty/no-governance target ref (then the gate would no-op and allow — guarded by committing real permissions.toml+gates.toml at main with maintainers excluding feat-author)

AC-2: Once the membership lands on the target ref and main advances, the same merge clears the Authority::Merge step
  GIVEN: the AC-1 scenario after the maintainers membership (feat-author added) is committed to refs/heads/main and main advances
  WHEN:  the SAME enforce_merge_gate(&ctx, review_id) is rerun with BUT_AGENT_HANDLE=feat-author against the advanced target ref
  THEN:  the perm.denied at the Authority::Merge step is GONE (classify_error(err).code != "perm.denied") — proving the flip is caused by the target ref advancing (sha-before != sha-after), not by reading the head; the residual gate.review_required from the seeded unapproved review gate (min_approvals=1) is the EXPECTED next gate and is named, NOT asserted as Ok
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api merge gate + real but-authz + real gix repo, before/after target-ref advancement
  VERIFY: cargo test -p but-api landed_membership_clears_merge_authority_step
  SCENARIO (negative controls): a constant-Ok stub fails AC-1 (which must DENY at the merge-authority step) and a constant-perm.denied stub fails AC-2 (the perm.denied must DISAPPEAR after landing), so the SAME code path must DENY merge-authority before landing and CLEAR it after; would pass without the sha advancing — the test asserts refs/heads/main sha changed between the denied attempt and the cleared attempt, so an unchanged ref means the authorization flip was not target-ref-driven; would pass if AC-2 asserted end-to-end Ok — that would be WRONG because the seeded review gate (min_approvals=1, no approval) returns gate.review_required after the merge authority passes; AC-2 must assert the merge-AUTHORITY perm.denied is gone, not that the gate returns Ok

AC-3: Self-granted administration:write on a feature head is inert until landed on the target ref
  GIVEN: a feature head `feat-admin` whose committed permissions.toml self-grants feat-author administration:write, while refs/heads/main grants feat-author no admin authority
  WHEN:  an administration:write authorization for feat-author is evaluated against the TARGET-REF config (load_governance_config(repo, "refs/heads/main") + resolve_principal via the injected-lookup variant for feat-author + authorize(Authority::AdministrationWrite)) before landing, then after the grant is committed to main
  THEN:  before landing the authorization is DENIED with code=="perm.denied" (the self-grant on the feature head is inert); after the grant lands on refs/heads/main and main advances, the same authorization is Ok
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-authz + real gix repo, target-ref before/after advancement
  VERIFY: cargo test -p but-authz self_grant_admin_inert_until_landed
  SCENARIO (negative controls): would pass (wrongly authorize) if load read the feature head config where feat-author self-granted administration:write — the target-ref read is what keeps it DENIED before landing; would pass against a stub authorize that always returns Ok (the before-landing case must DENY); would pass if the after-landing case did not actually advance refs/heads/main (sha must change before the authorization flips to Ok)

AC-4: Membership is provably read from the target-ref blob, not the feature head, and a working-tree edit has no authorization effect
  GIVEN: the self-escalation scenario where the feature head's tree AND an uncommitted working-tree edit both place feat-author in maintainers, but refs/heads/main does not
  WHEN:  load_governance_config(repo, "refs/heads/main") is read while HEAD points at the feature branch and an uncommitted permissions.toml edit also adds feat-author to maintainers
  THEN:  the loaded GovConfig shows feat-author is NOT a maintainers member and holds no merge authority — neither the feature head tree nor the working-tree edit affects the target-ref membership read
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-authz + real gix repo with HEAD on a feature branch + uncommitted working-tree edit
  VERIFY: cargo test -p but-authz membership_read_only_from_target_ref
  SCENARIO (negative controls): would pass (wrongly) if the loader peeled HEAD or read the working tree instead of refs/heads/main — feat-author would then appear in maintainers and hold merge, failing the must_not_observe; would pass against an empty config (guarded by asserting the loaded maintainers group exists with member `maint` and excludes feat-author)

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, error): enforce_merge_gate for the feature-head self-add returns Err and classify_error yields code=="perm.denied" at the merge-authority step (ForgeReview seeded inline)
    VERIFY: cargo test -p but-api self_add_to_maintainers_on_feature_head_still_denied_merge
- TC-2 (-> AC-1, structural): refs/heads/main HEAD sha is unchanged by the feature-head self-add (target ref not advanced)
    VERIFY: cargo test -p but-api self_add_to_maintainers_on_feature_head_still_denied_merge
- TC-3 (-> AC-2, happy_path): after the maintainers membership lands on refs/heads/main, the rerun clears the Authority::Merge step (classify_error.code != "perm.denied")
    VERIFY: cargo test -p but-api landed_membership_clears_merge_authority_step
- TC-4 (-> AC-2, structural): the refs/heads/main sha after landing differs from the sha before landing
    VERIFY: cargo test -p but-api landed_membership_clears_merge_authority_step
- TC-5 (-> AC-3, error): before landing, authorize(feat-author, AdministrationWrite) against refs/heads/main returns Err(Denial) code=="perm.denied"
    VERIFY: cargo test -p but-authz self_grant_admin_inert_until_landed
- TC-6 (-> AC-3, happy_path): after the administration:write grant lands on refs/heads/main, authorize(feat-author, AdministrationWrite) returns Ok(())
    VERIFY: cargo test -p but-authz self_grant_admin_inert_until_landed
- TC-7 (-> AC-4, edge): load_governance_config(repo, refs/heads/main) read while HEAD is on the feature branch shows maintainers members exclude feat-author
    VERIFY: cargo test -p but-authz membership_read_only_from_target_ref
- TC-8 (-> AC-4, edge): an uncommitted working-tree edit adding feat-author to maintainers does not change the target-ref effective authority for feat-author (no Authority::Merge)
    VERIFY: cargo test -p but-authz membership_read_only_from_target_ref

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-CONFIG-01, CAP-AUTHZ-01
provides: target-ref-only governed-membership read proof at the merge boundary (no self-escalation); self-grant-inert proof: administration:write self-granted on a feature head has zero authorization effect until landed on the target ref
consumes: but_api::legacy::merge_gate::enforce_merge_gate (Sprint-01b GATES-003 seam), but_api::legacy::merge_gate::classify_error, but_authz::load_governance_config, but_authz::resolve_principal_from_env, but_authz::resolve_principal, but_authz::authorize, but_authz::Authority::{Merge,AdministrationWrite}, but_db::ForgeReview + forge_reviews_mut().upsert, but_ctx::Context::{from_repo,with_memory_app_cache}, but_testsupport::writable_scenario, but_testsupport::invoke_bash
boundary_contracts:
  - CAP-CONFIG-01: a change can never grant itself authority or weaken its gate — all governance config (group definitions, membership, grants) is read at the target ref, never the head/working tree. Real-service proof: a feature head adding its author to a merge-holding group is still denied merge at the Authority::Merge step; a self-granted administration:write on a feature head is inert until landed.
  - CAP-AUTHZ-01: merge is permitted only if the acting principal's target-ref effective authority contains Authority::Merge; evaluated even under DryRun (DryRun never bypasses authorization, only suppresses persistence).

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-api/tests/merge_gate_self_escalation.rs (NEW) — the GRPS-002 PRIMARY proof: feature-head self-add to maintainers => merge denied at the Authority::Merge step; landed membership => merge-authority cleared (composes legacy::merge_gate::enforce_merge_gate read-only; seeds ForgeReview inline via forge_reviews_mut().upsert per merge_gate.rs:403-434)
  - crates/but-authz/tests/grps_ref_pin.rs (NEW) — the but-authz-layer self-grant-inert + target-ref-only membership-read proofs (AC-3, AC-4) using the injected-lookup resolve_principal variant
  - crates/but-authz/tests/fixtures/scenario/governance-base.sh (MODIFY, ONLY IF a shared helper is genuinely needed) — prefer per-test invoke_bash seeding over editing the shared fixture; if edited, keep it behavior-neutral for existing tests
writeProhibited:
  - crates/but-api/src/legacy/merge_gate.rs — CONSUME-only; GRPS-002 must not modify the Sprint-01b merge gate. If the gate genuinely lacks a needed seam, FLAG it as a GATES-003 dependency gap, do not patch it here. NOTE: merge_gate.rs contains a SECOND copy of the config loader (load_merge_governance_config / read_config_blob / normalize_permissions duplicating but-authz's) — flag as a known duplicate for GATES (Sprint 04) consolidation, do NOT consolidate it here.
  - crates/but-authz/src/** — GRPS-002 is a proof/consumer task over the single union GRPS-001 consolidated; do not re-open the union implementation
  - crates/but-authz/tests/invariant_build_gates.rs — do not weaken the honesty grep
  - any gitbutler-* crate
  - any but group CLI verb implementation (Sprint 05 / CLI-002)
  - files owned by GRPS-001 (the union consolidation in authorize.rs/config.rs)
  - Any file not in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-api/src/legacy/merge_gate.rs (39-94)
   Focus: enforce_merge_gate: load_merge_governance_config(repo, branch_ref(target_branch)) then resolve_principal_from_env + authorize(Authority::Merge) at line 48 — runs BEFORE the protected-branch check (50-56) and review-requirement (58-93); the merge-authority denial (perm.denied) is what AC-1 asserts, and the post-landing residual gate.review_required (81) is the expected next gate AC-2 names
2. crates/but-api/src/legacy/merge_gate.rs (96-108)
   Focus: classify_error: downcasts MergeGateError, then Denial -> MergeGateError{code,message} — how perm.denied (merge authority) vs gate.review_required (review gate) surface to the AC-1/AC-2 assertions
3. crates/but-api/src/legacy/merge_gate.rs (132-163)
   Focus: review_for_id (forge cache lookup by number) + current_head_oid (source-ref read) — the ForgeReview the inline seed must match (number, target_branch, source_branch, author, sha)
4. crates/but-api/tests/merge_gate.rs (398-434)
   Focus: context_with_review + seed_review: the EXACT inline ForgeReview seeding pattern (Context::from_repo(repo.clone())?.with_memory_app_cache() then forge_reviews_mut()?.upsert(ForgeReview{ ... }) ) the AC-1 fixture composes — this is the REQUIRED driver, no Sprint-01b helper needed
5. crates/but-api/tests/merge_gate.rs (9-17, 47, 105-154)
   Focus: temp_env::async_with_vars + #[serial_test::serial] for BUT_AGENT_HANDLE; the authorized-case pattern (135-145: expect_err + classify_error(...).is_none() because the seeded review gate fails outside governance) — AC-2 mirrors this 'merge-authority cleared, residual gate is the next one' assertion shape
6. crates/but-authz/src/config.rs (224-267)
   Focus: load_governance_config_inner + read_config_blob: peels target_ref to a commit and reads the blob from THAT tree — the proof that membership comes from the target ref, never HEAD/working tree (mirror config.rs tests config_loads_from_target_not_head / config_ignores_working_tree_edit)
7. crates/but-authz/src/authorize.rs (71-106)
   Focus: resolve_principal (injected lookup) / resolve_principal_from_env: BUT_AGENT_HANDLE -> Principal built from the target-ref config; use the injected-lookup resolve_principal variant in AC-3/AC-4 (direct load path) to set feat-author without mutating process env
8. crates/but-authz/tests/config.rs (164-193)
   Focus: config_loads_from_target_not_head: the exact HEAD-on-feature-branch pattern to reuse for AC-4 (git checkout -b feat then assert target-ref governs)
9. crates/but-authz/tests/config.rs (40-89)
   Focus: config_ignores_working_tree_edit: the uncommitted working-tree edit pattern to reuse for AC-4's working-tree negative control
10. crates/but-authz/tests/authorize.rs (45, 54-57, 74-77)
   Focus: resolve_principal injected-lookup usage (|key| (key=="BUT_AGENT_HANDLE").then(|| OsString::from(...))) — the no-process-env pattern for AC-3/AC-4
11. crates/but-authz/src/denial.rs (13-32)
   Focus: Denial{code,message,remediation_hint} + PERM_DENIED_CODE asserted by the self-grant-inert and self-escalation cases
12. crates/but-authz/tests/invariant_build_gates.rs (17-55)
   Focus: the honesty grep covers AUTHZ_AUTHORIZE/AUTHZ_CONFIG/COMMIT_GATE (authorize.rs/config.rs/commit/gate.rs) — it does NOT grep merge_gate.rs. For GRPS-002 (which writes only NEW test files and modifies no enforcement source) this gate is a guard-rail that stays green, not a grep over merge_gate.rs

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- self-escalation merge denied (PRIMARY): `cargo test -p but-api self_add_to_maintainers_on_feature_head_still_denied_merge`  -> Exit 0; merge denied with perm.denied at the Authority::Merge step (ForgeReview seeded inline); refs/heads/main sha unchanged by the feature-head self-add
- landed membership clears merge authority: `cargo test -p but-api landed_membership_clears_merge_authority_step`  -> Exit 0; after target-ref advances (sha changed), the same merge clears the Authority::Merge step (classify_error.code != perm.denied); residual gate.review_required is the named expected next gate
- self-grant admin inert until landed: `cargo test -p but-authz self_grant_admin_inert_until_landed`  -> Exit 0; administration:write denied before landing, Ok after the grant lands on the target ref
- membership read only from target ref: `cargo test -p but-authz membership_read_only_from_target_ref`  -> Exit 0; target-ref maintainers excludes feat-author despite head + working-tree edits
- honesty grep regression (guard-rail stays green): `cargo test -p but-authz invariant_build_gates`  -> Exit 0; no role-label/human-vs-AI branching in enforcement paths (authorize.rs/config.rs/commit/gate.rs — GRPS-002 touches none of these source files)
- clippy (authz): `cargo clippy -p but-authz --all-targets`  -> Exit 0
- clippy (api): `cargo clippy -p but-api --all-targets`  -> Exit 0
- fmt: `cargo fmt --check`  -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: target-ref-only governance read: peel refs/heads/main to its commit tree and read the config blob from THERE; authorize the merge/admin action against the principal's target-ref effective set; the feature head and working tree are never consulted for authorization
pattern_source: crates/but-authz/src/config.rs:242-267 (read_config_blob peels target_ref, never HEAD) + crates/but-api/src/legacy/merge_gate.rs:40-48 (enforce_merge_gate loads from target_branch ref then authorizes Merge); inline ForgeReview seeding per crates/but-api/tests/merge_gate.rs:398-434
anti_pattern: reading the feature/source head or working tree for membership/grants (the exact bug that would let a head authorize its own merge); asserting AC-2 returns Ok (wrong — the seeded review gate returns gate.review_required after merge authority passes; assert the merge-AUTHORITY perm.denied is gone instead); keying the AC-1 driver on a non-existent Sprint-01b ForgeReview helper instead of the inline forge_reviews_mut().upsert seam (merge_gate.rs:403-434); substituting the authz-layer composition for the enforce_merge_gate AC-1 driver (skips the forge lookup + branch_ref mapping); fabricating a `but group add-member` CLI verb to drive the test (does not exist until Sprint 05); modifying legacy/merge_gate.rs instead of composing it; asserting the flip without proving refs/heads/main actually advanced (sha-before != sha-after); resting the merge denial on a missing-config no-op instead of a real maintainers config that excludes feat-author at the target ref
interaction_notes:
  - PRIMARY merge driver: the AC-1/AC-2 enforce_merge_gate path seeds the ForgeReview INLINE via the public but-db API — `but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache()` then `ctx.db.get_cache_mut()?.forge_reviews_mut()?.upsert(ForgeReview { number, target_branch:"main", source_branch:"feat", author:Some("feat-author"), sha:<feat head>, ... })` (cite crates/but-api/tests/merge_gate.rs:398-434). This is achievable TODAY; there is NO blocker. The authz-layer composition (load_governance_config + resolve_principal + authorize(Merge)) is NOT an acceptable substitute for AC-1 — it skips the forge-review lookup + branch_ref target-ref mapping where a head-read regression in enforce_merge_gate would hide. Reserve the authz-layer path ONLY as a true last resort if the in-memory app cache genuinely cannot be constructed (it can — merge_gate.rs:398 demonstrates it).
  - Env handling: AC-1/AC-2 (enforce_merge_gate) MUST use temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("feat-author"))], ...) under #[serial_test::serial] (cite merge_gate.rs:9,17,47) because enforce_merge_gate resolves the principal via resolve_principal_from_env (process env is unavoidable). AC-3/AC-4 (direct resolve_principal/load path) SHOULD use the injected-lookup resolve_principal variant to avoid mutating process env (cite authorize.rs:71-93 + authorize.rs tests:45,54-57,74-77).
  - CONSUME-only: there is NO persisted `but group`/admin-config write path in Sprint 03 (Sprint 05 / CLI-002 owns it). The protected-branch admin-gate (T-GRPS-011) and the CLI inert-warning (T-GRPS-010 CLI half) are PROVEN at the AUTHORIZATION layer here (load + authorize denies) and the CLI surface is named as the Sprint 05 consumer.
  - T-GRPS-006 (group ops require administration:write) re-grounding: group-mutating operations are gated by the AUTHZ-006 administration:write guard composed at the authorization layer; the persisted `but group` write path is Sprint-05/CLI-002. GRPS-002 does NOT implement a group-write command — it proves the authorization predicate (authorize(Authority::AdministrationWrite) read at the target ref) that the future write path will compose, exactly as T-GRPS-011 is handled.
  - T-GRPS-011 protected-branch group-config admin-gate COMPOSES the AUTHZ-006 admin-write guard (Sprint 02) — GRPS-002 does NOT re-implement it; it exercises load + authorize(AdministrationWrite) to show a protected-branch config change requires administration:write read at the target ref.
  - Known duplicate (Sprint 04 GATES note): merge_gate.rs carries a SECOND config-loader copy (load_merge_governance_config / read_config_blob / normalize_permissions) duplicating but-authz's read_config_blob/normalize_permissions. Flagged for consolidation in GATES (Sprint 04). Do NOT scope that consolidation into GRPS-002 (CONSUME-only). OPTIONAL agreement sub-assertion in the GRPS-002 but-api test: assert the two loaders agree on the effective set for the same principal+target-ref (load_merge_governance_config(repo, "refs/heads/main").gov vs but_authz::load_governance_config(repo, "refs/heads/main") produce the same principal_authorities(feat-author)).

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — Rust work across but-authz config loading and the but-api merge-gate seam (legacy/merge_gate.rs), proving a security property (no self-escalation) against real git history. gix ref/blob reads, structured Denial/MergeGateError classification, and real-repo before/after head advancement are all rust-implementer competencies; rust-reviewer adversarially validates the self-escalation cannot be smuggled in via the working tree or feature head.
reviewer: rust-reviewer
coding_standards: crates/AGENTS.md, crates/WORKSPACE_MODEL.md, crates/but/AGENTS.md (CLI test conventions if any CLI surface is touched), crates/but-authz + crates/but-api/src/legacy (nearby patterns: gix target-ref blob read, Denial/MergeGateError classify_error, but-testsupport scenarios, inline ForgeReview seeding via forge_reviews_mut().upsert)

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: GRPS-001, GATES-003, AUTHZ-006, AUTHZ-001, AUTHZ-002, AUTHZ-003
Blocks:     Sprint 04, Sprint 05
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "GRPS-002",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "self_escalation_merge_base": {
      "description": "Real-git scenario via but_testsupport::writable_scenario(\"governance-base\"). Target ref refs/heads/main carries committed permissions.toml + gates.toml where author `feat-author` holds NO merge authority and the `maintainers` group (permissions=[\"merge\"]) does NOT list feat-author; gates.toml marks main protected with a review gate (min_approvals=1). A feature/source branch `feat` is created from main and commits a permissions.toml that ADDS feat-author to maintainers (the self-escalation attempt) in the feature head's tree only. The ForgeReview is seeded INLINE in the local in-memory app cache via the public but-db API (no Sprint-01b helper needed).",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"governance-base\");",
        "invoke_bash on main: write .gitbutler/permissions.toml with [[principal]] id=\"feat-author\" permissions=[\"contents:write\"] (NO merge); [[principal]] id=\"maint\" permissions=[\"merge\"]; [[group]] name=\"maintainers\" permissions=[\"merge\"] members=[\"maint\"] (feat-author NOT a member); and .gitbutler/gates.toml with [[branch]] name=\"main\" protected=true plus [[gate]] branch=\"main\" type=\"review\" min_approvals=1; then git add + commit on main.",
        "invoke_bash: git checkout -b feat; rewrite .gitbutler/permissions.toml so [[group]] maintainers members=[\"maint\",\"feat-author\"] (the self-add); also add an unrelated source change so feat has a distinct head; git add + commit on feat; git checkout main — the feature head tree now contains feat-author in maintainers, the target ref does NOT.",
        "Build the context and seed the ForgeReview INLINE (cite crates/but-api/tests/merge_gate.rs:398-434): `let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();` then `ctx.db.get_cache_mut()?.forge_reviews_mut()?.upsert(ForgeReview { number: REVIEW_ID.try_into()?, target_branch: \"main\".to_owned(), source_branch: \"feat\".to_owned(), author: Some(\"feat-author\".to_owned()), sha: <feat head oid>.to_string(), html_url, title, body:None, labels:\"[]\", draft:false, ... last_sync_at: fixed_time(0), struct_version: but_forge::ForgeReview::struct_version() })?;` so enforce_merge_gate(&ctx, REVIEW_ID) resolves this scenario. This inline seam is the REQUIRED AC-1 driver and works today.",
        "Set BUT_AGENT_HANDLE=feat-author for the enforce_merge_gate path using temp_env::async_with_vars([(\"BUT_AGENT_HANDLE\", Some(\"feat-author\"))], async { ... }) under #[serial_test::serial] (cite crates/but-api/tests/merge_gate.rs:9,17,47) — enforce_merge_gate resolves the principal via resolve_principal_from_env so env is unavoidable here.",
        "Landing step for the AC-2 positive case: git checkout main; commit the maintainers membership (feat-author added) onto refs/heads/main so main advances; capture main sha before and after."
      ]
    },
    "self_grant_admin_inert_base": {
      "description": "Real-git scenario via writable_scenario(\"governance-base\"). Target ref main: feat-author holds NO administration:write. A feature branch `feat-admin` commits a permissions.toml self-granting feat-author administration:write (and/or a gates.toml change to weaken main's protection). The committed target-ref config is unchanged. Used to prove the self-granted admin authority is inert: an administration:write authorization for feat-author is denied until the grant lands on the target ref. This fixture is but-authz-layer (load + resolve + authorize) and does NOT need a forge cache.",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"governance-base\");",
        "invoke_bash on main: .gitbutler/permissions.toml with [[principal]] id=\"feat-author\" permissions=[\"contents:write\"] (NO administration:write); .gitbutler/gates.toml [[branch]] name=\"main\" protected=true; git add + commit on main.",
        "invoke_bash: git checkout -b feat-admin; rewrite .gitbutler/permissions.toml so feat-author permissions=[\"contents:write\",\"administration:write\"] (the self-grant) AND/OR rewrite gates.toml to protected=false (gate-weakening attempt); git add + commit on feat-admin; git checkout main — only the feature head tree carries the escalation.",
        "Positive landing step: git checkout main; commit the administration:write grant onto refs/heads/main so main advances; capture sha before/after.",
        "Resolution: this fixture exercises the direct resolve_principal/load path, so SHOULD use the injected-lookup resolve_principal variant — `resolve_principal(|key| (key == \"BUT_AGENT_HANDLE\").then(|| OsString::from(\"feat-author\")), &cfg.gov)` — to avoid mutating process env (cite crates/but-authz/tests/authorize.rs:45,54-57,74-77). No temp_env/serial needed for AC-3/AC-4."
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "description": "Feature head self-adding its author to a merge-holding maintainers group is still denied merge at the Authority::Merge step (target-ref membership governs); ForgeReview seeded inline via forge_reviews_mut().upsert.",
      "verify": "cargo test -p but-api self_add_to_maintainers_on_feature_head_still_denied_merge",
      "primary": true,
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api legacy::merge_gate::enforce_merge_gate + real gix repo + committed target-ref permissions.toml/gates.toml + ForgeReview seeded inline via forge_reviews_mut().upsert",
        "negative_control": {
          "would_fail_if": [
            "would wrongly clear the merge-authority step if authorize read the feature/source head where feat-author IS in maintainers — the target-ref read is exactly what makes this DENY",
            "would pass against a stub merge gate that always returns Ok / never resolves the principal",
            "would pass against an empty/no-governance target ref (guarded by committing real maintainers config excluding feat-author at main)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "self_escalation_merge_base",
            "action": {
              "actor": "ci",
              "steps": [
                "seed ForgeReview inline via ctx.db.get_cache_mut()?.forge_reviews_mut()?.upsert(...) (merge_gate.rs:403-434)",
                "temp_env::async_with_vars([(\"BUT_AGENT_HANDLE\", Some(\"feat-author\"))], ...) under #[serial_test::serial]",
                "enforce_merge_gate(&ctx, review_id) for review {target:main, source:feat, author:feat-author}",
                "classify_error on the returned error"
              ]
            },
            "end_state": {
              "must_observe": [
                "`enforce_merge_gate` returns `Err`",
                "`classify_error(err).code == \"perm.denied\"`",
                "`classify_error(err).message` contains `\"merge\"`",
                "`refs/heads/main HEAD sha == the base sha` (target ref NOT advanced by the feature-head self-add)"
              ],
              "must_not_observe": [
                "exit `0` / `enforce_merge_gate` clears the Authority::Merge step (feature-head membership wrongly honored)",
                "`feat-author` appears as a `maintainers` member when read at `refs/heads/main`"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "Once the membership lands on the target ref and main advances, the same merge clears the Authority::Merge step (residual gate.review_required is the expected next gate).",
      "verify": "cargo test -p but-api landed_membership_clears_merge_authority_step",
      "primary": false,
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api legacy::merge_gate::enforce_merge_gate + real gix repo with target-ref advancement",
        "negative_control": {
          "would_fail_if": [
            "a constant-Ok stub fails AC-1 and a constant-perm.denied stub fails AC-2, so the SAME path must DENY merge-authority before landing and CLEAR it after",
            "would pass without the sha advancing — the test asserts refs/heads/main sha changed between the denied and cleared attempts",
            "would pass if AC-2 asserted end-to-end Ok — wrong, because the seeded review gate (min_approvals=1, no approval) returns gate.review_required after the merge authority passes"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "self_escalation_merge_base",
            "action": {
              "actor": "ci",
              "steps": [
                "capture sha_before = refs/heads/main",
                "enforce_merge_gate => Err classify_error.code==\"perm.denied\"",
                "commit the maintainers membership (feat-author added) onto refs/heads/main; capture sha_after",
                "rerun enforce_merge_gate; classify_error on any error"
              ]
            },
            "end_state": {
              "must_observe": [
                "the target-ref sha advanced: `sha_before != sha_after`",
                "the rerun CLEARS the Authority::Merge step: `classify_error(err).code != \"perm.denied\"` (`0` perm.denied at the merge-authority step)",
                "the residual denial (if any) is the EXPECTED `gate.review_required` next gate, named",
                "`feat-author` IS a `maintainers` member when read at the advanced `refs/heads/main`"
              ],
              "must_not_observe": [
                "the target-ref sha unchanged (`0`-distance advancement): `sha_before == sha_after`",
                "the rerun still returns `\"perm.denied\"` at the merge-authority step"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "Self-granted administration:write on a feature head is inert until landed on the target ref.",
      "verify": "cargo test -p but-authz self_grant_admin_inert_until_landed",
      "primary": false,
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-authz load_governance_config + resolve_principal (injected lookup) + authorize against the target ref",
        "negative_control": {
          "would_fail_if": [
            "would wrongly authorize if load read the feature head where feat-author self-granted administration:write",
            "would pass against a stub authorize that always returns Ok (before-landing must DENY)",
            "would pass if the after-landing case did not advance refs/heads/main"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "self_grant_admin_inert_base",
            "action": {
              "actor": "ci",
              "steps": [
                "resolve via injected lookup for feat-author (no process-env mutation)",
                "capture sha_before = refs/heads/main",
                "load_governance_config(repo, refs/heads/main); resolve_principal (injected); authorize(AdministrationWrite) BEFORE landing",
                "commit the administration:write self-grant onto refs/heads/main; capture sha_after",
                "reload from refs/heads/main and authorize(AdministrationWrite) AFTER landing"
              ]
            },
            "end_state": {
              "must_observe": [
                "`before-landing authorize(AdministrationWrite).err().code == \"perm.denied\"`",
                "`before-landing denial.message` contains `\"administration:write\"`",
                "the target-ref sha advanced: `sha_before != sha_after`",
                "`after-landing authorize(AdministrationWrite) == Ok(())`"
              ],
              "must_not_observe": [
                "the target-ref sha unchanged (`0`-distance advancement): `sha_before == sha_after`",
                "before-landing `authorize` returns `Ok` (self-grant honored from the feature head)",
                "after-landing `authorize` still returns `\"perm.denied\"`"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "description": "Membership is provably read from the target-ref blob, not the feature head; working-tree edit has no authorization effect.",
      "verify": "cargo test -p but-authz membership_read_only_from_target_ref",
      "primary": false,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-authz load_governance_config reading refs/heads/main while HEAD is on a feature branch",
        "negative_control": {
          "would_fail_if": [
            "would wrongly pass if the loader peeled HEAD or read the working tree — feat-author would appear in maintainers and hold merge",
            "would pass against an empty config (guarded by asserting maintainers exists with member maint and excludes feat-author)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "self_escalation_merge_base",
            "action": {
              "actor": "ci",
              "steps": [
                "git checkout feat (HEAD on feature branch with feat-author in maintainers)",
                "invoke_bash to write an uncommitted permissions.toml adding feat-author to maintainers",
                "load_governance_config(repo, refs/heads/main)",
                "inspect maintainers members and effective authority for feat-author"
              ]
            },
            "end_state": {
              "must_observe": [
                "`cfg.groups()[\"maintainers\"]` lists member `\"maint\"`",
                "`maintainers` members exclude `feat-author` when read at `refs/heads/main`",
                "effective authority for `feat-author` does NOT contain `Authority::Merge`"
              ],
              "must_not_observe": [
                "`0` filtering of head/working-tree edits so an empty-friction `feat-author` entry appears in the target-ref `maintainers` membership",
                "`feat-author` present in the target-ref `maintainers` membership",
                "`feat-author` holding `Authority::Merge` from the head or working-tree edit"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "enforce_merge_gate for the feature-head self-add returns Err; classify_error code==\"perm.denied\" at the merge-authority step",
      "verify": "cargo test -p but-api self_add_to_maintainers_on_feature_head_still_denied_merge",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "refs/heads/main HEAD sha unchanged by the feature-head self-add",
      "verify": "cargo test -p but-api self_add_to_maintainers_on_feature_head_still_denied_merge",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "after landing, the rerun clears the Authority::Merge step (classify_error.code != \"perm.denied\")",
      "verify": "cargo test -p but-api landed_membership_clears_merge_authority_step",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "refs/heads/main sha after landing differs from the sha before landing",
      "verify": "cargo test -p but-api landed_membership_clears_merge_authority_step",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "before landing, authorize(feat-author, AdministrationWrite) against main returns Err(Denial) perm.denied",
      "verify": "cargo test -p but-authz self_grant_admin_inert_until_landed",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "description": "after landing, authorize(feat-author, AdministrationWrite) returns Ok(())",
      "verify": "cargo test -p but-authz self_grant_admin_inert_until_landed",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-7",
      "type": "test_criterion",
      "description": "load from refs/heads/main while HEAD on feat shows maintainers excludes feat-author",
      "verify": "cargo test -p but-authz membership_read_only_from_target_ref",
      "maps_to_ac": "AC-4"
    },
    {
      "id": "TC-8",
      "type": "test_criterion",
      "description": "uncommitted working-tree edit does not change feat-author's target-ref effective authority (no Merge)",
      "verify": "cargo test -p but-authz membership_read_only_from_target_ref",
      "maps_to_ac": "AC-4"
    }
  ]
}
-->
</details>
