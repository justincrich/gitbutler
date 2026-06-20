# GRPS-001: Consolidate the effective-set union to one authoritative path (config.rs load-time fold) + prove the group permission ceiling

## What this does

Establishes ONE authoritative effective-authority union for a governed principal: direct grants ∪ every group it joins via `principal.groups=[...]` ∪ every group whose `members=[...]` names it, read from the committed target-ref `.gitbutler/permissions.toml`. Today config.rs::normalize_permissions (294-332) folds BOTH membership directions into the stored principal AuthoritySet at load time — this is already the single, complete source of truth. authorize.rs::effective_authority (51-62) then redundantly re-unions the same group-member grants on top of the already-folded `principal_authorities(id)` it reads as its fold base, so `effective_authority(p)` is provably == `principal_authorities(p)` for every p (and both yield empty exactly when the principal is absent). This task removes that redundant authorize-time re-union (authorize.rs:56-61) so `effective_authority` simply returns the already-folded `principal_authorities(id)` — a behavior-NEUTRAL simplification, NOT a behavior change. It then proves the union contract end-to-end against real git: a member with NO direct review grant is authorized to review via its group and is still denied merge (no source grants it), and proves the named group permission ceiling (delegated admin: a group MAY hold administration:write).

## Why

Sprint 03 · PRD UC-GRPS-01 · CAP-AUTHZ-01. Serves the gate clause [GRPS-01] (a member with no direct grant is authorized via its group's union and denied an action no source grants). config.rs's load-time fold is the single, complete source of truth; the authorize-time re-union in effective_authority is dead weight that re-computes a subset of what config.rs already folded. Removing it leaves exactly one site that resolves group-only members and makes the simplification refactor-safe via an equality pin. GRPS-002 builds on this single consolidated union, and Sprints 04/05 consume it.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-authz group_union_authorizes_review_denies_merge` (Group-only member is authorized via the group union and denied an unsourced authority). Full gate set in the spec below.

## Scope

- crates/but-authz/src/authorize.rs (MODIFY) — simplify effective_authority to return the already-folded cfg.principal_authorities(id) set and DELETE the redundant group-member re-union at 56-61; add a comment documenting effective_authority == principal_authorities by construction; keep grep-clean
- crates/but-authz/src/config.rs (MODIFY, COMMENT-ONLY) — add a clarifying comment at normalize_permissions naming the load-time fold (302-332) the single source of truth for the effective set; do NOT remove the member-fold
- crates/but-authz/tests/grps_union.rs (NEW) — the GRPS-001 integration proof (union authorizes via group + denies unsourced + effective_authority==principal_authorities pin + delegated-admin ceiling + claims-do-not-widen-even-with-group-backing); RE-IMPLEMENT the AUTHZ_EMPTY_START guard from tests/config.rs:197-199 in this file's governed-repo helper

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: GRPS-001 - Consolidate the effective-set union to one authoritative path (config.rs load-time fold) + prove the group permission ceiling
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     M  (150 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-GRPS-01
CAPABILITIES: CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-authz group_union_authorizes_review_denies_merge   |   cargo test -p but-authz union_paths_stay_equal   |   cargo test -p but-authz delegated_admin_ceiling   |   cargo test -p but-authz claims_do_not_widen_union_even_with_group_backing   |   cargo test -p but-authz invariant_build_gates
  check: cargo check -p but-authz --all-targets
  lint:  cargo clippy -p but-authz --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
One authoritative union path: config.rs::normalize_permissions's load-time fold is the sole site that resolves group-only members; authorize.rs::effective_authority is simplified to return the already-folded `principal_authorities(id)` (the redundant re-union at authorize.rs:56-61 is removed) with NO behavior change. A regression test pins `effective_authority(p) == principal_authorities(p)` so the simplification stays behavior-neutral. A new integration test proves: (a) a group-only member with no direct grant is authorized for the group's authority and denied an authority no source supplies (perm.denied), (b) a principal joining a group via `groups=[...]` and a principal listed in a group's `members=[...]` resolve to the SAME effective set, (c) a group holding administration:write confers it on its members (named delegated-admin ceiling), distinct from a group that does not. `cargo test -p but-authz`, `cargo clippy -p but-authz --all-targets`, and the existing `invariant_build_gates` honesty grep are all green.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST keep config.rs::normalize_permissions's load-time fold (the principal.groups=[...] fold at 302-311 AND the group.members=[...] fold at 319-332) as the SINGLE authoritative site that resolves group-only members. This is the ONLY site that can resolve a member with no [[principal]] entry; it is canonical.
- [MUST] MUST simplify authorize.rs::effective_authority to read the already-folded set: return `cfg.principal_authorities(principal.id()).cloned().unwrap_or_else(AuthoritySet::empty)` and DELETE the redundant group-member re-union at authorize.rs:56-61. This is a behavior-NEUTRAL refactor — `effective_authority(p)` already equals `principal_authorities(p)` for every p because config.rs pre-folds; document this equality in a code comment at effective_authority.
- [MUST] MUST add a regression/safety pin that asserts `effective_authority(p) == cfg.principal_authorities(p)` (the already-folded set) for a group-only member AFTER the re-union is removed — a refactor-safety pin proving the simplification did NOT change the resolved set, NOT a divergence assertion.
- [MUST] MUST ensure the consolidated load-time fold honors BOTH membership directions — a group's `members=[...]` AND a principal's `groups=[...]` — so the by-reference path (reviewer-byref) and the by-membership path (reviewer-only) both resolve correctly; AC-2 guards this by asserting each resolves to exactly {reviews:write} (len()==1, contains(ReviewsWrite)).
- [NEVER] NEVER drop config.rs's group.members=[...] member-fold (319-332). Removing it makes a group-only member (a member named only in a group's members list, with no [[principal]] entry) resolve to principal_authorities==None, so effective_authority early-returns empty and the member is DENIED — breaking AC-1. config.rs is canonical; it is forbidden to relocate the canonical fold into authorize.rs.
- [NEVER] NEVER branch on role names or role presets (read/triage/write/maintain/admin) in any enforcement path — authority is the functional Authority set alone. The existing invariant_build_gates grep over authorize.rs/config.rs must stay clean (no new `from_role(`, no `"admin" =>` style match, no role-string compare).
- [NEVER] NEVER read the working tree or any non-target ref for membership or grants — union is computed only from the committed target-ref config blob via load_governance_config.
- [NEVER] NEVER weaken the proof by giving the test member a direct reviews:write grant (vacuous: authorization could then come from the direct grant). The member MUST hold zero direct review authority so authorization can ONLY originate from the group.
- [STRICTLY] STRICTLY present a group holding administration:write (delegated admin) as an ACCEPTED, NAMED property — never as a defect to 'fix' or a silent escalation. The audit surface 'who can change config' = direct administration:write holders ∪ members of administration:write-holding groups.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: Group-only member is authorized via the group union and denied an unsourced authority
- [ ] AC-2: effective_authority equals GovConfig::principal_authorities by construction — a refactor-safety pin for removing the redundant re-union
- [ ] AC-3: Group permission ceiling: a group holding administration:write confers delegated admin on its members (named, accepted)
- [ ] AC-4: Caller-supplied authority claims cannot widen the union even with real group backing, and lookup is keyed by principal.id() not the attached claim
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: Group-only member is authorized via the group union and denied an unsourced authority [PRIMARY]
  GIVEN: a committed target-ref config where `reviewer-only` holds NO direct grant and has NO [[principal]] entry but is named in `code-reviewers` members=[...] (reviews:write)
  WHEN:  load_governance_config(&repo, "refs/heads/main") then authorize(reviewer-only, Authority::ReviewsWrite, &cfg) and authorize(reviewer-only, Authority::Merge, &cfg)
  THEN:  ReviewsWrite returns Ok(()) (authority sourced only from the group via config.rs's load-time member-fold), Merge returns Err(Denial) with code=="perm.denied" and message naming "merge" (no source grants merge)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-authz + real gix repo + committed target-ref TOML via but-testsupport
  VERIFY: cargo test -p but-authz group_union_authorizes_review_denies_merge
  SCENARIO (negative controls): would pass against a stub/no-op authorize that always returns Ok (the Merge case must DENY); would pass even if the group member-fold were dropped — impossible to pass then because reviewer-only has NO [[principal]] entry, so dropping config.rs:319-332 => principal_authorities(reviewer-only)==None => empty set => ReviewsWrite would be DENIED, failing the happy half; would pass against an empty/AUTHZ_EMPTY_START config where reviewer-only is unknown (effective set empty => ReviewsWrite DENIED and the merge denial message would change)

AC-2: effective_authority equals GovConfig::principal_authorities by construction — a refactor-safety pin for removing the redundant re-union
  GIVEN: `reviewer-only` joins `code-reviewers` via the group's members=[...] (config.rs:319-332) and `reviewer-byref` joins the same group via the principal's own groups=[...] (config.rs:302-311), with the redundant authorize-time re-union removed
  WHEN:  the effective set is observed via both observation points — effective_authority(principal,&cfg) and the loaded GovConfig::principal_authorities(id) — for both members and for ro
  THEN:  for every principal, effective_authority(p) == principal_authorities(p) (equal BY CONSTRUCTION, since effective_authority now returns the already-folded set); both members hold exactly {reviews:write} (len()==1), and ro's set excludes reviews:write — pinning that removing the re-union did NOT change the resolved set
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-authz + real gix repo + committed target-ref TOML
  VERIFY: cargo test -p but-authz union_paths_stay_equal
  SCENARIO (negative controls): would pass (correctly DETECT a regression) if the simplified effective_authority returned a DIFFERENT set than principal_authorities for the group-only member — proving the re-union removal changed behavior; the assert_eq! must FAIL in that case, which is exactly what this pin guards against; would pass against a stub effective_authority returning a fixed set regardless of input (then byref and bylist would trivially match for the wrong reason — guarded by also asserting each set equals exactly {reviews:write} via len()==1 + contains(ReviewsWrite), and that ro's principal_authorities EXCLUDES ReviewsWrite so a constant set fails)

AC-3: Group permission ceiling: a group holding administration:write confers delegated admin on its members (named, accepted)
  GIVEN: a `config-admins` group holding administration:write with member `delegate` (no direct admin grant) and a control member of `code-reviewers` (no admin)
  WHEN:  authorize(delegate, Authority::AdministrationWrite, &cfg) and authorize(reviewer-only, Authority::AdministrationWrite, &cfg)
  THEN:  delegate is authorized for AdministrationWrite purely via the group (the named delegated-admin ceiling), while reviewer-only is denied AdministrationWrite with perm.denied — confirming the ceiling is a real, source-gated property, not a silent universal escalation
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-authz + real gix repo + committed target-ref TOML
  VERIFY: cargo test -p but-authz delegated_admin_ceiling
  SCENARIO (negative controls): would pass against a stub that grants administration:write to everyone (reviewer-only must be DENIED); would pass even if the config-admins group member-fold were dropped — impossible because delegate holds NO [[principal]] entry, so dropping config.rs:319-332 => denial, failing the authorized half; would pass against an AUTHZ_EMPTY_START config (delegate unknown => denied, changing the authorized observation)

AC-4: Caller-supplied authority claims cannot widen the union even with real group backing, and lookup is keyed by principal.id() not the attached claim
  GIVEN: (case A) an in-memory Principal for `reviewer-only` — a REAL group-backed member (code-reviewers => reviews:write) — fabricated with an EXTRA direct AuthoritySet claim containing merge (NOT present in any committed source); (case B) a Principal whose attached AuthoritySet claims administration:write but whose id() is `reviewer-only` (a non-admin id mismatched from the privileged claim)
  WHEN:  (A) authorize(fabricated reviewer-only, Authority::Merge, &cfg); (B) authorize(mismatched principal, Authority::AdministrationWrite, &cfg) — cfg is the committed target-ref config
  THEN:  (A) Err(Denial) perm.denied naming "merge" — the real reviews:write group backing does NOT make the fabricated merge claim honored, proving caller claims never widen the cfg-stored union even for a legitimately group-backed principal; (B) Err(Denial) perm.denied — the privileged administration:write claim is inert because the cfg lookup is keyed by principal.id() ("reviewer-only", a non-admin), NOT by the attached authorities. This is NET-NEW vs the existing authority_only_from_config test (which uses `ro` with no group backing and a contents:write claim).
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-authz + real gix repo + committed target-ref TOML
  VERIFY: cargo test -p but-authz claims_do_not_widen_union_even_with_group_backing
  SCENARIO (negative controls): would pass against a (buggy) authorize that trusts Principal::authorities() instead of the cfg-stored union — case A's fabricated merge claim would then be honored and Merge would return Ok, failing this DENY assertion; would pass against an authorize that resolved authority from the attached claim rather than by principal.id() — case B's administration:write claim would then be honored despite the non-admin id, failing the DENY; would pass against a no-op authorize that always denies regardless of input (guarded by AC-1/AC-3 which require Ok for legitimately-sourced authorities, and by case A's principal genuinely holding reviews:write via its real group so authorize(reviewer-only, ReviewsWrite) would still be Ok)

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, happy_path): authorize(reviewer-only, ReviewsWrite, cfg) returns Ok(()) for the group-only member (no [[principal]] entry)
    VERIFY: cargo test -p but-authz group_union_authorizes_review_denies_merge
- TC-2 (-> AC-1, error): authorize(reviewer-only, Merge, cfg) returns Err(Denial) with code=="perm.denied"
    VERIFY: cargo test -p but-authz group_union_authorizes_review_denies_merge
- TC-3 (-> AC-2, structural): effective_authority(reviewer-only, cfg) equals GovConfig::principal_authorities(reviewer-only) by construction after the redundant re-union is removed (refactor-safety pin)
    VERIFY: cargo test -p but-authz union_paths_stay_equal
- TC-4 (-> AC-2, edge): the membership-by-list (reviewer-only) and membership-by-reference (reviewer-byref) effective sets each equal principal_authorities and contain exactly Authority::ReviewsWrite (len()==1), while ro excludes it
    VERIFY: cargo test -p but-authz union_paths_stay_equal
- TC-5 (-> AC-3, happy_path): authorize(delegate, AdministrationWrite, cfg) returns Ok(()) via the config-admins group
    VERIFY: cargo test -p but-authz delegated_admin_ceiling
- TC-6 (-> AC-3, error): authorize(reviewer-only, AdministrationWrite, cfg) returns Err(Denial) with code=="perm.denied"
    VERIFY: cargo test -p but-authz delegated_admin_ceiling
- TC-7 (-> AC-4, error): authorize for a real group-backed reviewer-only Principal carrying a fabricated direct merge claim returns Err(Denial) perm.denied, and a Principal with id() reviewer-only carrying an administration:write claim is denied (lookup keyed by id())
    VERIFY: cargo test -p but-authz claims_do_not_widen_union_even_with_group_backing
- TC-8 (-> AC-2, structural): the invariant_build_gates honesty grep over authorize.rs and config.rs finds no role-preset or role-label branching after the re-union removal
    VERIFY: cargo test -p but-authz invariant_build_gates

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides: effective-set union: own ∪ group(by-reference) ∪ group(by-membership) computed once at load time in config.rs and read unchanged by effective_authority; group permission ceiling contract (a group MAY hold administration:write; delegated admin is a named accepted property)
consumes: but_authz::load_governance_config, but_authz::authorize, but_authz::effective_authority, but_authz::AuthoritySet::union, but_authz::GovConfig::{principals,groups,principal_authorities}, but_authz::Group::{authorities,members}, but_testsupport::writable_scenario, but_testsupport::invoke_bash
boundary_contracts:
  - CAP-AUTHZ-01: a consequential action is permitted iff the acting principal's effective authority (own ∪ group grants, folded once at load time and read at the target ref) contains the required Authority; otherwise a structured Denial{code=="perm.denied"} naming the missing authority. Authorization is read-only and identical whether the union is observed via effective_authority or via GovConfig::principal_authorities — equal BY CONSTRUCTION.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-authz/src/authorize.rs (MODIFY) — simplify effective_authority to return the already-folded cfg.principal_authorities(id) set and DELETE the redundant group-member re-union at 56-61; add a comment documenting effective_authority == principal_authorities by construction; keep grep-clean
  - crates/but-authz/src/config.rs (MODIFY, COMMENT-ONLY) — add a clarifying comment at normalize_permissions naming the load-time fold (302-332) the single source of truth for the effective set; do NOT remove the member-fold
  - crates/but-authz/tests/grps_union.rs (NEW) — the GRPS-001 integration proof (union authorizes via group + denies unsourced + effective_authority==principal_authorities pin + delegated-admin ceiling + claims-do-not-widen-even-with-group-backing); RE-IMPLEMENT the AUTHZ_EMPTY_START guard from tests/config.rs:197-199 in this file's governed-repo helper
writeProhibited:
  - crates/but-authz/src/config.rs member-fold (294-332 logic) — do NOT remove or relocate the canonical load-time fold; comment-only edits permitted
  - crates/but-api/** — GRPS-001 is but-authz-layer only; the merge gate is GRPS-002 territory
  - crates/but-authz/src/authority.rs — do not alter the Authority catalog or role presets for this task
  - crates/but-authz/tests/invariant_build_gates.rs — do not weaken the honesty grep to make changes pass; satisfy it instead
  - any gitbutler-* crate
  - files owned by GRPS-002 (merge-gate composition, self-escalation proofs)
  - Any file not in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-authz/src/config.rs (269-335)
   Focus: load-time union (CANONICAL site): normalize_permissions folds both principal.groups=[...] (302-311) AND group.members=[...] (319-332) into the stored principal AuthoritySet — this is the single, complete source of truth; group.members fold (319-332) is the ONLY site that resolves a group-only member with no [[principal]] entry and MUST NOT be removed
2. crates/but-authz/src/authorize.rs (24-62)
   Focus: authorize() + effective_authority(): the redundant authorize-time re-union at 56-61 re-folds group members already folded by config.rs onto the already-folded principal_authorities(id) base it reads at 52 — provably == principal_authorities(id) for every principal; simplify effective_authority to return that folded set and DELETE 56-61 (behavior-neutral)
3. crates/but-authz/src/authority.rs (248-282)
   Focus: AuthoritySet::union and ::contains — the set algebra config.rs's load-time fold relies on; no role branching here
4. crates/but-authz/src/principal.rs (145-215)
   Focus: Group::{authorities,members} accessors used by the load-time fold; Principal::new(id, authorities, groups) constructor used to fabricate the AC-4 claim principals
5. crates/but-authz/src/denial.rs (1-32)
   Focus: Denial{code,message,remediation_hint} + PERM_DENIED_CODE — the structured denial the negative cases assert
6. crates/but-authz/tests/authorize.rs (93-147)
   Focus: existing effective_authority_union (93-120) + authority_only_from_config (122-147, uses `ro` with a contents:write claim and NO group backing). AC-4 must be NET-NEW: a REAL group-backed reviewer-only with a fabricated merge claim + an id/claim mismatch — do NOT duplicate authority_only_from_config
7. crates/but-authz/tests/config.rs (195-230)
   Focus: governed_repo() helper + the AUTHZ_EMPTY_START guard (early return at 197-199). This guard is NOT in tests/authorize.rs::governed_repo — copy lines 197-199 verbatim into the NEW grps_union.rs helper
8. crates/but-authz/tests/invariant_build_gates.rs (8-55)
   Focus: the honesty grep patterns (ROLE_BRANCH_PATTERN, HUMAN_OR_LABEL_BRANCH_PATTERN) over authorize.rs/config.rs/commit/gate.rs that the re-union removal must keep clean — it greps these THREE source files, never the test files

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- authority-from-cfg-not-claim honesty guard: `! grep -nE 'principal\.authorities\(\)' crates/but-authz/src/authorize.rs`  -> Exit 0 (no match); effective_authority/authorize must NOT read Principal's direct authorities() field in the enforcement path — authority comes only from cfg.principal_authorities(id), locking the 'authority from cfg, never the Principal claim field' property structurally
- GRPS union integration: `cargo test -p but-authz group_union_authorizes_review_denies_merge`  -> Exit 0; reviewer-only (group-only member) authorized via group for ReviewsWrite, denied Merge with perm.denied
- effective_authority == principal_authorities pin: `cargo test -p but-authz union_paths_stay_equal`  -> Exit 0; effective_authority == principal_authorities for group-only and by-ref members; behavior-neutral after the re-union removal
- delegated-admin ceiling: `cargo test -p but-authz delegated_admin_ceiling`  -> Exit 0; delegate authorized for administration:write via group, control denied
- claims do not widen union: `cargo test -p but-authz claims_do_not_widen_union_even_with_group_backing`  -> Exit 0; fabricated merge claim on a real group-backed member denied; admin claim with non-admin id denied
- honesty grep regression: `cargo test -p but-authz invariant_build_gates`  -> Exit 0; no role-preset/role-label branching in authorize.rs/config.rs after the re-union removal
- clippy: `cargo clippy -p but-authz --all-targets`  -> Exit 0; no warnings
- fmt: `cargo fmt --check`  -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: single authoritative effective-authority union folded once at load time in config.rs (direct ∪ group(by principal.groups) ∪ group(by group.members)); effective_authority is a thin reader of the folded GovConfig principal set
pattern_source: crates/but-authz/src/config.rs:294-332 (canonical load-time fold) read by crates/but-authz/src/authorize.rs:52 (principal_authorities base) after deleting the redundant re-union at 56-61
anti_pattern: framing the simplification as fixing a 'divergence' (the two paths are equal by construction and CANNOT diverge); dropping config.rs's member-fold (319-332) which would break group-only members (None => empty => denied); resting the union proof on a member that ALSO holds a direct grant (vacuous); duplicating the existing authority_only_from_config test for AC-4 instead of making it net-new/adversarial; branching on role names to compute the union; fabricating a `but group` CLI verb to drive the test (verbs do not exist until Sprint 05)
interaction_notes:
  - effective_authority(p) is provably == GovConfig::principal_authorities(p) for EVERY p today: effective_authority reads principal_authorities(id) as its fold base (authorize.rs:52) and re-unions only group-member grants that config.rs already folded in (config.rs:319-332), and it returns empty exactly when principal_authorities is None. The re-union at authorize.rs:56-61 is therefore dead weight. Remove it; effective_authority becomes a thin reader of the already-folded set. config.rs's load-time fold stays canonical (it is the only site that can resolve a group-only member with no [[principal]] entry).
  - Pin the equality: assert effective_authority(p) == principal_authorities(p) for the group-only member so the simplification is verifiably behavior-neutral and any future drift fails a test, not production. This is a refactor-safety pin, NOT a divergence claim.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — Pure but-authz Rust work: set-algebra over AuthoritySet, gix-backed config load at a target ref, structured Denial contract, and a build-gate grep invariant. No frontend, no Tauri, no SvelteKit. rust-implementer owns the TDD cycle; rust-reviewer owns the adversarial pass (clippy, anti-stub, honesty-grep regression).
reviewer: rust-reviewer
coding_standards: crates/AGENTS.md, crates/WORKSPACE_MODEL.md, crates/but-authz (nearby patterns: thiserror Denial, gix target-ref blob read, but-testsupport scenarios)

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: AUTHZ-001, AUTHZ-002, AUTHZ-003
Blocks:     GRPS-002, Sprint 04, Sprint 05
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "GRPS-001",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "group_union_base": {
      "description": "Real-git scenario via but_testsupport::writable_scenario(\"governance-base\") with committed .gitbutler/permissions.toml + gates.toml at refs/heads/main. Defines a `code-reviewers` group holding reviews:write with a member (`reviewer-only`) that has NO direct grant and NO [[principal]] entry (membership comes ONLY from the group's members list — the group-only member that exercises config.rs:319-332), a second principal `reviewer-byref` with a [[principal]] entry carrying `groups=[\"code-reviewers\"]` (the by-reference path, config.rs:302-311) to prove path-equivalence, a read-only principal (`ro`), and a `config-admins` group holding administration:write with member `delegate` (no direct admin grant, no [[principal]] entry) for the delegated-admin ceiling. NOTE: the AUTHZ_EMPTY_START guard from tests/config.rs:197-199 is NOT inherited by tests/authorize.rs::governed_repo — it MUST be re-implemented in the new grps_union.rs helper (copy the `if std::env::var_os(\"AUTHZ_EMPTY_START\").is_some() { return (repo, tmp); }` early-return verbatim from tests/config.rs:197-199).",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"governance-base\");",
        "RE-IMPLEMENT the AUTHZ_EMPTY_START guard from tests/config.rs:197-199 inside the new grps_union.rs governed-repo helper (it is NOT inheritable from tests/authorize.rs): `if std::env::var_os(\"AUTHZ_EMPTY_START\").is_some() { return (repo, tmp); }` BEFORE any invoke_bash committing config.",
        "but_testsupport::invoke_bash writes .gitbutler/permissions.toml with: [[principal]] id=\"ro\" permissions=[\"contents:read\"]; [[principal]] id=\"reviewer-byref\" groups=[\"code-reviewers\"] (by-reference path, no direct grant); [[group]] name=\"code-reviewers\" permissions=[\"reviews:write\"] members=[\"reviewer-only\"] (reviewer-only is the group-only member — NO [[principal]] entry); [[group]] name=\"config-admins\" permissions=[\"administration:write\"] members=[\"delegate\"] (delegate is group-only — NO [[principal]] entry)",
        "and .gitbutler/gates.toml with [[branch]] name=\"main\" protected=true",
        "then `git add .gitbutler/permissions.toml .gitbutler/gates.toml && git commit -m \"group union config\"` so the blobs live at refs/heads/main (the target ref).",
        "AUTHZ_EMPTY_START env guard (re-implemented per tests/config.rs:197-199): when set, return the bare scenario with NO committed governance config — used by the negative-control case so load fails closed (config.invalid) or authorize denies (perm.denied) against a stub-empty start."
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "description": "Group-only member (no [[principal]] entry) authorized for ReviewsWrite via config.rs's load-time member-fold; denied Merge with perm.denied.",
      "verify": "cargo test -p but-authz group_union_authorizes_review_denies_merge",
      "primary": true,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-authz + real gix repo + committed target-ref permissions.toml/gates.toml",
        "negative_control": {
          "would_fail_if": [
            "would pass against a stub/no-op authorize that always returns Ok (the Merge case must DENY)",
            "would pass even if the group member-fold were dropped — impossible because reviewer-only has NO [[principal]] entry => dropping config.rs:319-332 => empty set => ReviewsWrite DENIED",
            "would pass against an AUTHZ_EMPTY_START config where reviewer-only is unknown"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "group_union_base",
            "action": {
              "actor": "ci",
              "steps": [
                "load_governance_config(&repo, \"refs/heads/main\")",
                "authorize(reviewer-only, Authority::ReviewsWrite, &cfg)",
                "authorize(reviewer-only, Authority::Merge, &cfg)"
              ]
            },
            "end_state": {
              "must_observe": [
                "`authorize(reviewer-only, ReviewsWrite) == Ok(())`",
                "`authorize(reviewer-only, Merge).err().code == \"perm.denied\"`",
                "`Merge denial.message` contains `\"merge\"`"
              ],
              "must_not_observe": [
                "`0` granted authorities for `reviewer-only` (an ignored/empty group union) such that `authorize(reviewer-only, ReviewsWrite)` returns `Err`",
                "`authorize(reviewer-only, ReviewsWrite)` returns `Err` (would mean group member-fold not applied)",
                "`authorize(reviewer-only, Merge)` returns `Ok` (would mean an unsourced authority was granted)"
              ]
            }
          },
          {
            "start_ref": "group_union_base",
            "action": {
              "actor": "ci",
              "steps": [
                "set AUTHZ_EMPTY_START (re-implemented guard per tests/config.rs:197-199)",
                "load_governance_config(&repo, \"refs/heads/main\") expecting config.invalid; else authorize reviewer-only"
              ]
            },
            "end_state": {
              "must_observe": [
                "exactly one fail-closed outcome: EITHER `load(...).err().code == \"config.invalid\"` OR `authorize(reviewer-only, ReviewsWrite).err().code == \"perm.denied\"`"
              ],
              "must_not_observe": [
                "`0`/empty effective set with no error and no denial (silent fail-open)",
                "reviewer-only authorized for ReviewsWrite from a stub-empty start"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "effective_authority == principal_authorities by construction; refactor-safety pin proving the redundant re-union removal is behavior-neutral.",
      "verify": "cargo test -p but-authz union_paths_stay_equal",
      "primary": false,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-authz + real gix repo + committed target-ref permissions.toml",
        "negative_control": {
          "would_fail_if": [
            "would correctly FAIL (catch a regression) if the simplified effective_authority returned a DIFFERENT set than principal_authorities for the group-only member — proving the re-union removal changed behavior",
            "would pass against a stub returning a fixed set (guarded by asserting exactly {reviews:write} via len()==1 + contains and that ro excludes reviews:write)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "group_union_base",
            "action": {
              "actor": "ci",
              "steps": [
                "load_governance_config",
                "compute effective_authority and principal_authorities for reviewer-only and reviewer-byref; compute principal_authorities(ro)"
              ]
            },
            "end_state": {
              "must_observe": [
                "effective_authority(reviewer-only) == principal_authorities(reviewer-only)",
                "effective_authority(reviewer-byref) == principal_authorities(reviewer-byref)",
                "both contain Authority::ReviewsWrite and len()==1",
                "principal_authorities(ro) excludes Authority::ReviewsWrite"
              ],
              "must_not_observe": [
                "effective_authority and principal_authorities differ for the same principal (`0` tolerated drift)",
                "either reviewer set is empty (`none` resolved)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "Group permission ceiling: config-admins group confers administration:write on member delegate; control member denied.",
      "verify": "cargo test -p but-authz delegated_admin_ceiling",
      "primary": false,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-authz + real gix repo + committed target-ref permissions.toml",
        "negative_control": {
          "would_fail_if": [
            "would pass against a stub granting administration:write to everyone (reviewer-only must be DENIED)",
            "would pass even if the config-admins member-fold were dropped — impossible because delegate has no [[principal]] entry",
            "would pass against AUTHZ_EMPTY_START (delegate unknown => denied)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "group_union_base",
            "action": {
              "actor": "ci",
              "steps": [
                "load_governance_config",
                "authorize(delegate, AdministrationWrite, &cfg)",
                "authorize(reviewer-only, AdministrationWrite, &cfg)"
              ]
            },
            "end_state": {
              "must_observe": [
                "`authorize(delegate, AdministrationWrite) == Ok(())`",
                "`authorize(reviewer-only, AdministrationWrite).err().code == \"perm.denied\"`",
                "`reviewer-only denial.message` contains `\"administration:write\"`"
              ],
              "must_not_observe": [
                "`0` ceiling enforcement so a `default`/universal grant lets `authorize(reviewer-only, AdministrationWrite)` return `Ok`",
                "`authorize(delegate, AdministrationWrite)` returns `Err` (ceiling not honored)",
                "`authorize(reviewer-only, AdministrationWrite)` returns `Ok` (universal escalation)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "description": "Caller claims do not widen the union even for a real group-backed principal; cfg lookup keyed by principal.id() not the attached claim — NET-NEW vs authority_only_from_config.",
      "verify": "cargo test -p but-authz claims_do_not_widen_union_even_with_group_backing",
      "primary": false,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-authz + real gix repo + committed target-ref permissions.toml",
        "negative_control": {
          "would_fail_if": [
            "would pass against an authorize that trusts Principal::authorities() (case A fabricated merge then honored => Ok, failing DENY)",
            "would pass against an authorize resolving authority from the attached claim rather than by principal.id() (case B administration:write claim honored for the non-admin id => Ok, failing DENY)",
            "would pass against a no-op authorize that always denies (guarded by AC-1/AC-3 requiring Ok for legitimate authorities)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "group_union_base",
            "action": {
              "actor": "ci",
              "steps": [
                "load_governance_config",
                "case A: construct Principal::new(PrincipalId::new(\"reviewer-only\"), AuthoritySet::parse([\"merge\"])?, [GroupName::new(\"code-reviewers\")]); authorize(case-A, Authority::Merge, &cfg)",
                "case B: construct Principal::new(PrincipalId::new(\"reviewer-only\"), AuthoritySet::parse([\"administration:write\"])?, std::iter::empty()); authorize(case-B, Authority::AdministrationWrite, &cfg)"
              ]
            },
            "end_state": {
              "must_observe": [
                "`authorize(case-A, Merge).err().code == \"perm.denied\"`",
                "`case-A denial.message` contains `\"merge\"`",
                "`authorize(case-B, AdministrationWrite).err().code == \"perm.denied\"`"
              ],
              "must_not_observe": [
                "`0` rejected caller claims (a `default`-granted empty-friction widening) so either authorize returns `Ok`",
                "case-A `authorize(Merge)` returns `Ok` (fabricated claim honored despite real group backing)",
                "case-B `authorize(AdministrationWrite)` returns `Ok` (privileged claim honored for a non-admin id)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "authorize(reviewer-only, ReviewsWrite, cfg) returns Ok(()) for the group-only member",
      "verify": "cargo test -p but-authz group_union_authorizes_review_denies_merge",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "authorize(reviewer-only, Merge, cfg) returns Err(Denial) with code==\"perm.denied\"",
      "verify": "cargo test -p but-authz group_union_authorizes_review_denies_merge",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "effective_authority(reviewer-only, cfg) equals GovConfig::principal_authorities(reviewer-only) by construction (refactor-safety pin)",
      "verify": "cargo test -p but-authz union_paths_stay_equal",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "by-list and by-reference sets each equal principal_authorities and contain exactly {reviews:write}; ro excludes it",
      "verify": "cargo test -p but-authz union_paths_stay_equal",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "authorize(delegate, AdministrationWrite, cfg) returns Ok(()) via config-admins",
      "verify": "cargo test -p but-authz delegated_admin_ceiling",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "description": "authorize(reviewer-only, AdministrationWrite, cfg) returns Err(Denial) perm.denied",
      "verify": "cargo test -p but-authz delegated_admin_ceiling",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-7",
      "type": "test_criterion",
      "description": "fabricated merge claim on a real group-backed reviewer-only is denied; administration:write claim with non-admin id is denied (lookup by id())",
      "verify": "cargo test -p but-authz claims_do_not_widen_union_even_with_group_backing",
      "maps_to_ac": "AC-4"
    },
    {
      "id": "TC-8",
      "type": "test_criterion",
      "description": "invariant_build_gates grep over authorize.rs/config.rs finds no role-preset/role-label branching after the re-union removal",
      "verify": "cargo test -p but-authz invariant_build_gates",
      "maps_to_ac": "AC-2"
    }
  ]
}
-->
</details>
