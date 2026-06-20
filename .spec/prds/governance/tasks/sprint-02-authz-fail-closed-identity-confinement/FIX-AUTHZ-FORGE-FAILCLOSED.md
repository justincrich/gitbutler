# FIX-AUTHZ-FORGE-FAILCLOSED: forge governance opt-in is fail-OPEN on absence — replace bespoke `has_governance_config` with canonical `but_authz::governance_present`

## What this does

Fixes a fail-OPEN defect (red-hat H-1) on the non-merge forge boundary. `crates/but-api/src/legacy/forge.rs::has_governance_config` (L70-77) decides whether `authorize_branch_action` (L53) even runs the authorization gate, and it diverges from the canonical discriminator in TWO load-bearing ways: (1) it checks ONLY `.gitbutler/permissions.toml` (L75) — it never checks `.gitbutler/gates.toml`; and (2) it PROPAGATES every ref/commit/tree error via `?` (L71-73) instead of failing closed. The canonical `but_authz::governance_present` (`crates/but-authz/src/config.rs:44`) checks `permissions.toml` OR `gates.toml` (config.rs:58) AND treats every unresolvable ref/commit/tree as governed (`Ok(true)`, config.rs:47/51/55). Consequence: a repo that commits ONLY `.gitbutler/gates.toml` returns `Ok(false)` from `has_governance_config` → `authorize_branch_action` returns `Ok(None)` (L54) = UNGOVERNED → the non-merge forge verbs (`approve_review` `reviews:write` L516, `comment_review` `comments:write` L564, `close_review`/`publish_review` `pull_requests:write` L478/L580) run UNAUTHORIZED. This task DELETES the bespoke `has_governance_config` and routes the opt-in check through `but_authz::governance_present`, so the forge boundary fails closed exactly like the merge/commit/admin-write gates. The merge path (`merge_review` L590 → `enforce_merge_gate`, which already uses `load_merge_governance_config`) is UNCHANGED.

## Why

Sprint 02 · PRD UC-AUTHZ-03, UC-AUTHZ-04 · capabilities CAP-AUTHZ-01, CAP-CONFIG-01. Sprint 02's central thesis is fail-closed on absence: a governed repo must not let governed verbs run unauthorized just because one governance file is missing. AUTHZ-008 pulled `forge.rs` into `ENFORCEMENT_PATHS` (invariant_build_gates.rs:29), making this divergence Sprint 02's concern; the Boy-Scout + fail-closed thesis apply. `governance_present` is documented as "the single source of truth for the governance file paths — callers must not re-derive `.gitbutler/*.toml` literals" (config.rs:42-43) — the bespoke `has_governance_config` is exactly the re-derivation that doc forbids.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-api forge_guard_gates_toml_only_repo_is_governed` (integration: a repo committing ONLY `.gitbutler/gates.toml`, no `permissions.toml`, denies an unauthorized principal `perm.denied` on `approve_review` instead of running ungoverned, against real but-api forge seam + real but-authz + real git). Full gate set in the spec below.

## Scope

- `crates/but-api/src/legacy/forge.rs` (MODIFY) — DELETE the bespoke `has_governance_config` (L70-77); change `authorize_branch_action` (L53) to call `but_authz::governance_present(repo, &ref_name)?`. OWNS the forge opt-in discriminator becoming fail-closed-on-error and `permissions.toml`-OR-`gates.toml`. Merge-path code is untouched.
- `crates/but-api/tests/forge_guard.rs` (MODIFY) — add a `gates.toml`-only governed fixture + the new fail-closed integration cases (gates-only denies unauthorized; permissions-only regression; both-files regression) against the real forge seam.

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: FIX-AUTHZ-FORGE-FAILCLOSED - forge governance opt-in is fail-OPEN on absence: replace bespoke has_governance_config with canonical but_authz::governance_present
================================================================================

TASK_TYPE:  BUGFIX
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     M  (120 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ./SPRINT.md
PRD_REFS:   UC-AUTHZ-03, UC-AUTHZ-04
CAPABILITIES: CAP-AUTHZ-01, CAP-CONFIG-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-api forge_guard
  check: cargo check -p but-api --all-targets
  lint:  cargo clippy -p but-api --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Integration tests are green against the real but-api forge seam + real but-authz + real git: a repo that commits ONLY .gitbutler/gates.toml (NO permissions.toml) is recognized as GOVERNED, so a non-merge forge verb (approve_review) by a principal that lacks reviews:write is DENIED perm.denied — NOT permitted via Ok(None)/ungoverned as it is today. A repo committing only permissions.toml still governs (regression). A repo committing both files still governs (regression). The forge opt-in discriminator now delegates to but_authz::governance_present (config.rs:44) — which checks permissions.toml OR gates.toml AND fails closed (Ok(true)) on every unresolvable ref/commit/tree — instead of the deleted bespoke has_governance_config (forge.rs:70-77) that checked permissions.toml ONLY and propagated errors via `?`. The merge path (merge_review -> enforce_merge_gate via load_merge_governance_config) is unchanged. The gates-only-denies case is kept in a test that also proves the SAME unauthorized principal is permitted when the repo is truly ungoverned, so a guard that returns Ok(true) for everyone (always-governed) is NOT what is being asserted — the discriminator must distinguish governed from ungoverned correctly.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST DELETE the bespoke has_governance_config (crates/but-api/src/legacy/forge.rs:70-77) and route the opt-in check in authorize_branch_action (forge.rs:53) through but_authz::governance_present(repo, &ref_name). Do NOT keep both. The canonical fn checks permissions.toml OR gates.toml (config.rs:58) and fails closed Ok(true) on unresolvable ref/commit/tree (config.rs:47/51/55) — exactly the two properties the bespoke fn lacks.
- [MUST] MUST use the FULLY-QUALIFIED but_authz::governance_present form (governance_present is already importable; the call must be qualified or imported into the existing `use but_authz::{...}` at forge.rs:4). The point is to consume the canonical SOURCE OF TRUTH for governance file paths (config.rs:42-43 doc), not re-derive `.gitbutler/*.toml` literals — after this fix, ZERO `.gitbutler/permissions.toml` / `.gitbutler/gates.toml` string literals remain in forge.rs.
- [MUST] MUST make AC-1 FALSIFIABLE against the real bug: the gates.toml-only fixture commits gates.toml but NO permissions.toml, names a principal WITHOUT reviews:write, and asserts approve_review returns Err carrying perm.denied (via classify_error, forge.rs:24). Against the UNFIXED code (permissions.toml-only check), this principal would be permitted (Ok(None) -> the verb proceeds and records/no-ops) — so the test FAILS on the current bug and PASSES after the fix. Name this in the AC-1 negative_control.
- [MUST] MUST keep the merge path UNTOUCHED: merge_review (forge.rs:590), set_review_auto_merge (forge.rs:633), and dry_run_merge_review (forge.rs:625) call crate::legacy::merge_gate::enforce_merge_gate, which uses load_merge_governance_config — that path does NOT consult has_governance_config and MUST NOT change. The reviewer confirms the merge-gate call sites are byte-identical.
- [MUST] MUST classify deterministically via the EXISTING forge.rs classify_error (forge.rs:24): a Denial -> {code:"perm.denied", message} (forge.rs:25-30); a ConfigError -> {code:"config.invalid", message} (forge.rs:32-36). Do NOT add a new error type; do NOT blur perm.denied and config.invalid. governance_present returns anyhow::Result<bool>; an error from it propagates via `?` (the gate runs on Ok(true)), NOT as a default-allow.
- [NEVER] NEVER let an unresolvable ref/commit/tree DOWNGRADE the repo to ungoverned. The bespoke fn propagated those via `?` (forge.rs:71-73), which surfaces a non-governance error that can be mistaken for "skip the gate"; governance_present instead returns Ok(true) (governed) so load_governance_config then classifies the fault. Preserve that fail-closed-on-error contract — do NOT reintroduce a permissive branch.
- [NEVER] NEVER key the forge opt-in off a role name, a human-vs-AI label, or the handle string — the discriminator is purely "did this ref commit a governance file" (governance_present) and the decision is purely Authority (the existing authorize calls at forge.rs:60-65). This task does NOT touch the Authority axis; it only fixes the opt-in discriminator (the AUTHORITY_POSITIVE fully-qualify is FIX-AUTHZ-FORGE-COVERAGE).
- [STRICTLY] STRICTLY scope to the opt-in discriminator swap + its fail-closed integration proof. Do NOT re-architect authorize_branch_action's per-Authority match (forge.rs:59-66), do NOT change any forge verb's downstream behavior, do NOT add a CLI verb. This is a behavior-neutral-on-the-governed-path fix that closes the ungoverned hole on absence.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: a gates.toml-only governed repo DENIES an unauthorized principal perm.denied on approve_review (governed), and the SAME principal is permitted when the repo is genuinely ungoverned — proving the discriminator distinguishes governed from ungoverned
- [ ] AC-2: a permissions.toml-only governed repo still governs (denies unauthorized) AND a both-files repo still governs — regression cover
- [ ] AC-3: has_governance_config is deleted; the forge opt-in routes through the fully-qualified but_authz::governance_present; zero `.gitbutler/*.toml` literals remain in forge.rs; the merge-gate call sites are unchanged (build-gate)
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: A gates.toml-only governed repo denies an unauthorized principal on a non-merge forge verb (governed, not fail-open) [PRIMARY]
  GIVEN: fixture `forge_gates_only_repo` (committed target ref `refs/heads/feat`: a VALID .gitbutler/gates.toml is committed, but NO .gitbutler/permissions.toml; a principal `ro` is therefore absent/holds nothing) AND a parallel fixture `forge_ungoverned_repo` (no .gitbutler/ governance file committed at all)
  WHEN:  approve_review(ctx, "feat") is called with BUT_AGENT_HANDLE=ro against `forge_gates_only_repo`, then the same call is made against `forge_ungoverned_repo`
  THEN:  against `forge_gates_only_repo` -> Err carrying a Denial classified code=="perm.denied" (the repo is GOVERNED because gates.toml is present, and `ro` lacks reviews:write); against `forge_ungoverned_repo` -> the call is NOT denied perm.denied (Ok(None) governed-absent path: no verdict written / governance-required error path, not a perm.denied) — proving the discriminator distinguishes governed (gates-only) from ungoverned
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api forge seam (approve_review) + real but-authz governance_present + real git
  VERIFY: cargo test -p but-api forge_guard_gates_toml_only_repo_is_governed
  SCENARIO: NEGATIVE_CONTROL would fail if the forge opt-in checks permissions.toml ONLY (the current bug) so the gates-only repo is treated UNGOVERNED and `ro` is permitted (no perm.denied) — i.e. against the unfixed code this test fails; if governance_present is replaced by an always-Ok(true) stub so the genuinely-ungoverned repo is wrongly treated governed; if the gates-only denial returns a code other than perm.denied; if reviews:write is granted to `ro`.

AC-2: permissions.toml-only and both-files governed repos still govern (regression)
  GIVEN: fixture `forge_permissions_only_repo` (committed permissions.toml with `ro`=[contents:read] only, NO gates.toml) and fixture `forge_both_files_repo` (committed permissions.toml + gates.toml)
  WHEN:  approve_review is called with BUT_AGENT_HANDLE=ro against each
  THEN:  BOTH deny `ro` perm.denied (each remains governed; `ro` lacks reviews:write) — the swap to governance_present did not regress the permissions.toml-present cases
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api forge seam + real but-authz + real git
  VERIFY: cargo test -p but-api forge_guard_permissions_and_both_still_governed
  SCENARIO: NEGATIVE_CONTROL would fail if the swap to governance_present accidentally stops recognizing a permissions.toml-only repo as governed (regression), letting `ro` through; if the both-files repo is treated ungoverned; if either denial returns config.invalid instead of perm.denied.

AC-3: has_governance_config is deleted; the forge opt-in routes through fully-qualified but_authz::governance_present; no governance-path literals remain; merge gate unchanged [build-gate]
  GIVEN: the source tree after this task
  WHEN:  the build-gate greps run over forge.rs
  THEN:  `fn has_governance_config` is GONE (0 matches), `but_authz::governance_present` appears (1+), no `.gitbutler/permissions.toml` / `.gitbutler/gates.toml` string literal remains in forge.rs (0), and `merge_gate::enforce_merge_gate` is still called by merge_review/set_review_auto_merge/dry_run_merge_review (3 matches, unchanged)
  TEST_TIER: unit (build-gate)   VERIFICATION_SERVICE: source grep (no runtime I/O)   UNIT_TEST_JUSTIFIED: structural delete-the-bespoke-fn + use-the-canonical-source-of-truth invariant verified by grep with zero runtime I/O. The behavioral fail-closed guarantee is proven by AC-1/AC-2 integration; this gate enforces the "no re-derivation of governance literals" structural property (config.rs:42-43) and the merge-path-untouched invariant that a runtime test cannot assert by inspection.
  VERIFY: ! grep -rEn 'fn has_governance_config' crates/but-api/src/legacy/forge.rs && grep -rEn 'but_authz::governance_present' crates/but-api/src/legacy/forge.rs && ! grep -rEn '\.gitbutler/(permissions|gates)\.toml' crates/but-api/src/legacy/forge.rs && [ "$(grep -cE 'merge_gate::enforce_merge_gate' crates/but-api/src/legacy/forge.rs)" -ge 3 ]
  SCENARIO: NEGATIVE_CONTROL would fail if has_governance_config is left in place (the bespoke fail-open fn survives); if a `.gitbutler/*.toml` literal is re-derived in forge.rs (the canonical source of truth is bypassed); if governance_present is not called (the fix never lands); if the merge-gate call sites were altered (scope creep into the merge path).

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, happy_path): a gates.toml-only governed repo denies an unauthorized principal perm.denied on approve_review, and the same principal is not perm.denied against a genuinely-ungoverned repo (governed-vs-ungoverned discriminator correct) (H-1)
    VERIFY: cargo test -p but-api forge_guard_gates_toml_only_repo_is_governed
- TC-2 (-> AC-2, regression): permissions.toml-only and both-files governed repos each still deny an unauthorized principal perm.denied (no regression from the swap)
    VERIFY: cargo test -p but-api forge_guard_permissions_and_both_still_governed
- TC-3 (-> AC-3, structural): has_governance_config deleted; fully-qualified but_authz::governance_present used; no `.gitbutler/*.toml` literals in forge.rs; 3 merge_gate call sites preserved
    VERIFY: ! grep -rEn 'fn has_governance_config' crates/but-api/src/legacy/forge.rs && grep -rEn 'but_authz::governance_present' crates/but-api/src/legacy/forge.rs

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01, CAP-CONFIG-01
provides: a forge opt-in discriminator that fails closed on absence — a repo committing EITHER .gitbutler/permissions.toml OR .gitbutler/gates.toml is governed (so the non-merge forge verbs run their Authority gate), and an unresolvable ref/commit/tree is treated as governed (fail-closed-on-error). Closes the H-1 fail-OPEN hole on the non-merge forge verbs (approve_review/comment_review/close_review/publish_review).
consumes: but_authz::governance_present (crates/but-authz/src/config.rs:44 — the canonical permissions-OR-gates, fail-closed-on-error discriminator); the EXISTING forge.rs authorize_branch_action / classify_error / load_governance_config / resolve_principal_from_env wiring (unchanged otherwise); crate::legacy::merge_gate::enforce_merge_gate (the merge path, untouched)
boundary_contracts:
  - CAP-AUTHZ-01: a governed forge verb resolves BUT_AGENT_HANDLE->Principal and authorizes the required Authority, and the opt-in that decides "governed" fails CLOSED on absence (permissions.toml OR gates.toml present, or unresolvable ref) rather than fail-open on a missing permissions.toml.
  - CAP-CONFIG-01: governance presence is read at the TARGET ref committed tree (governance_present reads the ref tree, never the working tree); an unresolvable ref is governed so the loader classifies any fault.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-api/src/legacy/forge.rs (MODIFY) — DELETE has_governance_config (70-77); route authorize_branch_action (53) through fully-qualified but_authz::governance_present; OWNS the forge opt-in becoming fail-closed-on-error + permissions-OR-gates. Merge-path code untouched.
  - crates/but-api/tests/forge_guard.rs (MODIFY) — add the gates.toml-only / ungoverned / permissions-only / both-files fixtures + integration cases against the real forge seam
writeProhibited:
  - crates/but-authz/** — CONSUME governance_present; do NOT modify the canonical discriminator
  - crates/but-api/src/legacy/merge_gate.rs — the merge path is AUTHZ-004; do NOT touch it (and forge.rs's merge_gate call sites must stay byte-identical)
  - crates/but-api/src/legacy/config_mutate.rs — the admin-write guard is AUTHZ-006; not in scope
  - crates/but-authz/tests/invariant_build_gates.rs — the forge coverage build-gate is FIX-AUTHZ-FORGE-COVERAGE; do NOT edit it here
  - crates/but-error/src/lib.rs — reuse the existing forge.rs ForgeGateError &'static str codes, no Code variants
  - any gitbutler-* crate beyond what the action boundary strictly requires (crates/AGENTS.md)
  - Any file not listed in write_allowed

--------------------------------------------------------------------------------
OUT OF SCOPE
--------------------------------------------------------------------------------
  - Fully-qualifying the per-Authority authorize calls (forge.rs:60-65 import authorize BARE) so AUTHZ-008's AUTHORITY_POSITIVE bites on forge.rs — that is FIX-AUTHZ-FORGE-COVERAGE (this task may leave the bare authorize import; it only fixes the opt-in discriminator). Coupled but separate.
  - The merge path's fail-closed determinism (AUTHZ-004, already merged) and config_mutate admin-write guard (AUTHZ-006).
  - The accepted BUT_AGENT_HANDLE env re-export leak (AUTHZ-005) — not a forge concern.
  - The forgeable direct-config-write outside the governed seam — an accepted leak, NOT tested as blocked.

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-api/src/legacy/forge.rs (47-77)
   Focus: THE BUG — authorize_branch_action (47) calls has_governance_config (53) as the opt-in; has_governance_config (70-77) checks .gitbutler/permissions.toml ONLY (75) and propagates ref/commit/tree errors via `?` (71-73). DELETE it; call but_authz::governance_present instead.
2. crates/but-authz/src/config.rs (31-63)
   Focus: THE CANONICAL FIX — governance_present (44) checks permissions.toml OR gates.toml (58 via tree_has_path 61) AND returns Ok(true) on every unresolvable ref/commit/tree (47/51/55) = fail-closed. Doc (42-43): "single source of truth for the governance file paths — callers must not re-derive `.gitbutler/*.toml` literals."
3. crates/but-api/src/legacy/forge.rs (23-37, 510-585)
   Focus: classify_error (24) maps Denial -> perm.denied (25-30) and ConfigError -> config.invalid (32-36) — reuse it unchanged. The non-merge verbs whose hole this closes: approve_review (513, reviews:write), comment_review (561, comments:write), close_review (577, pull_requests:write), publish_review (470, pull_requests:write).
4. crates/but-api/src/legacy/forge.rs (588-660)
   Focus: THE MERGE PATH (untouched) — merge_review (590), dry_run_merge_review (625), set_review_auto_merge (633) call crate::legacy::merge_gate::enforce_merge_gate; these MUST stay byte-identical (they don't use has_governance_config).
5. crates/but-api/tests/forge_guard.rs (full)
   Focus: THE HARNESS — governed_review_repo (147) uses but_testsupport::writable_scenario("checkout-head-info") + invoke_bash to commit permissions.toml AND an (empty) gates.toml, then a `feat` branch; temp_env::with_var("BUT_AGENT_HANDLE", Some("ro"/"reviewer"), ...); classify_error(&err) asserts code=="perm.denied". Build the new fixtures by committing ONLY gates.toml (gates-only), ONLY permissions.toml (permissions-only), and NEITHER (ungoverned) at the `feat` ref.
6. crates/but-testsupport/src/lib.rs (71-97, 432-441)
   Focus: writable_scenario + invoke_bash to commit governance blobs at a ref; NEVER std::env::temp_dir().join(...).

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- Forge fail-closed integration passes: `cargo test -p but-api forge_guard` -> Exit 0; AC-1 (gates-only governed denies unauthorized; ungoverned does not perm.denied) + AC-2 (permissions-only + both-files still govern) green
- Bespoke fn deleted: `! grep -rEn 'fn has_governance_config' crates/but-api/src/legacy/forge.rs` -> No matches
- Canonical discriminator used (fully-qualified): `grep -rEn 'but_authz::governance_present' crates/but-api/src/legacy/forge.rs` -> 1+ match
- No re-derived governance literals: `! grep -rEn '\.gitbutler/(permissions|gates)\.toml' crates/but-api/src/legacy/forge.rs` -> No matches
- Merge path untouched: `grep -cE 'merge_gate::enforce_merge_gate' crates/but-api/src/legacy/forge.rs` -> 3 (merge_review, dry_run_merge_review, set_review_auto_merge)
- Crate compiles incl. tests: `cargo check -p but-api --all-targets` -> Exit 0
- Clippy clean: `cargo clippy -p but-api --all-targets` -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: Delete-the-bespoke-discriminator, consume-the-canonical-source-of-truth — authorize_branch_action's opt-in becomes `if !but_authz::governance_present(repo, &ref_name)? { return Ok(None); }`, deleting has_governance_config entirely. governance_present checks permissions.toml OR gates.toml and fails closed Ok(true) on unresolvable ref/commit/tree, so the forge boundary fails closed on absence exactly like the merge/commit/admin-write gates. classify_error (forge.rs:24) is reused unchanged for Denial -> perm.denied / ConfigError -> config.invalid. The merge path is untouched. The gates-only test sits beside a genuinely-ungoverned test so the discriminator is proven to distinguish governed from ungoverned (not just always-governed).
pattern_source: crates/but-authz/src/config.rs:44 (governance_present — permissions-OR-gates, fail-closed-on-error) + crates/but-api/src/legacy/forge.rs:47-77 (authorize_branch_action + the bespoke has_governance_config being deleted) + crates/but-api/src/commit/gate.rs / merge_gate.rs (the gates that already consume the canonical discriminator)
anti_pattern: Keeping has_governance_config alongside governance_present; re-deriving a `.gitbutler/*.toml` literal in forge.rs (bypassing the source of truth); reintroducing a permissive branch on ref/commit/tree error (fail-open-on-error); touching the merge-gate call sites; asserting only the gates-only denial without the ungoverned control (so an always-Ok(true) stub passes); blurring perm.denied into config.invalid.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — Deletes the bespoke has_governance_config and routes the forge opt-in through the fully-qualified canonical but_authz::governance_present, closing the H-1 fail-OPEN hole on absence (permissions-OR-gates, fail-closed-on-error). Proves against real but-api forge seam + real but-authz + real git that a gates.toml-only governed repo denies an unauthorized principal perm.denied on approve_review (vs the unfixed code which permits it), keeps the permissions-only + both-files cases governed, and leaves the merge path byte-identical. Owns the discriminator swap, the fail-closed integration proof with a governed-vs-ungoverned falsifier, and the no-re-derived-literals structural property.
reviewer: rust-reviewer
coding_standards: crates/AGENTS.md, crates/WORKSPACE_MODEL.md, crates/but/AGENTS.md, crates/but-authz/src/config.rs

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: AUTHZ-001, AUTHZ-002, AUTHZ-003, AUTHZ-004 (the but-authz primitive + governance_present + the merged forge seam)
Blocks:     FIX-AUTHZ-FORGE-COVERAGE (after this fix forge.rs is a first-class enforcement surface that must be fully covered); Sprint 05
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "FIX-AUTHZ-FORGE-FAILCLOSED",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "notes": [
    "H-1: forge.rs has_governance_config (70-77) checks .gitbutler/permissions.toml ONLY and propagates ref/commit/tree errors via `?`; canonical but_authz::governance_present (config.rs:44) checks permissions.toml OR gates.toml and fails closed Ok(true) on every unresolvable ref/commit/tree. The bespoke fn is fail-OPEN on absence for the non-merge forge verbs (approve_review/comment_review/close_review/publish_review). This task DELETES it and routes the opt-in through the canonical fn.",
    "Falsifiability: AC-1's gates.toml-only fixture FAILS against the unfixed permissions.toml-only check (the principal is permitted) and PASSES after the fix (denied perm.denied). The genuinely-ungoverned control proves the discriminator distinguishes governed from ungoverned, so an always-Ok(true) stub is caught.",
    "Merge-path untouched: merge_review/dry_run_merge_review/set_review_auto_merge call merge_gate::enforce_merge_gate (load_merge_governance_config), which never consults has_governance_config; AC-3 asserts these 3 call sites are preserved.",
    "Scope boundary: fully-qualifying the per-Authority authorize calls (forge.rs:60-65, bare import) for AUTHORITY_POSITIVE coverage is FIX-AUTHZ-FORGE-COVERAGE — coupled but a separate task.",
    "Classification reuse: the existing forge.rs classify_error (24) maps Denial -> perm.denied / ConfigError -> config.invalid; no new error type; codes never blurred."
  ],
  "fixtures": {
    "forge_gates_only_repo": {
      "description": "A real git repo (but_testsupport::writable_scenario(\"checkout-head-info\")) whose target ref refs/heads/feat has a committed, VALID .gitbutler/gates.toml ([[branch]] name=\"feat\" protected=true) but NO .gitbutler/permissions.toml committed. The repo is GOVERNED by presence of gates.toml (governance_present returns true); a principal `ro` is therefore absent from permissions and holds no authority, so approve_review (reviews:write) must be DENIED perm.denied. Against the UNFIXED has_governance_config (permissions.toml-only), governance_present's gates-only signal is missed -> Ok(None) ungoverned -> `ro` permitted (the bug).",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"checkout-head-info\");",
        "invoke_bash: mkdir -p .gitbutler; write a VALID .gitbutler/gates.toml ([[branch]] name=\"feat\" protected=true); DO NOT write .gitbutler/permissions.toml",
        "invoke_bash: git add .gitbutler/gates.toml; git commit -m 'gates-only governance'; git checkout -b feat; commit a feat-base change; git checkout main"
      ]
    },
    "forge_ungoverned_repo": {
      "description": "Same scenario base but NO .gitbutler governance file committed at the feat ref at all (neither permissions.toml nor gates.toml). governance_present returns false -> authorize_branch_action returns Ok(None) (governed-absent). approve_review for `ro` is therefore NOT denied perm.denied — this is the control that proves the discriminator distinguishes governed (gates-only) from ungoverned, catching an always-Ok(true) stub.",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"checkout-head-info\");",
        "invoke_bash: git checkout -b feat; commit a feat-base change; git checkout main (NO .gitbutler/ governance file committed)"
      ]
    },
    "forge_permissions_only_repo": {
      "description": "Committed .gitbutler/permissions.toml at feat with principal `ro`=[contents:read] only, NO gates.toml. Remains governed (governance_present true via permissions.toml). approve_review for `ro` denied perm.denied (regression cover for the permissions-present case after the swap).",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"checkout-head-info\");",
        "invoke_bash: mkdir -p .gitbutler; write .gitbutler/permissions.toml ([[principal]] id=\"ro\" permissions=[\"contents:read\"]); DO NOT write gates.toml; git add + commit; git checkout -b feat; commit; git checkout main"
      ]
    },
    "forge_both_files_repo": {
      "description": "Committed .gitbutler/permissions.toml (`ro`=[contents:read]) AND a valid .gitbutler/gates.toml at feat. Governed; approve_review for `ro` denied perm.denied (regression cover for the both-files case — mirrors the existing governed_review_repo shape).",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"checkout-head-info\");",
        "invoke_bash: mkdir -p .gitbutler; write permissions.toml (`ro`=[contents:read]) AND a valid gates.toml; git add both + commit; git checkout -b feat; commit; git checkout main"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN forge_gates_only_repo (gates.toml committed, NO permissions.toml) and forge_ungoverned_repo (no governance file) WHEN approve_review(ctx, \"feat\") is called with BUT_AGENT_HANDLE=ro against each THEN against forge_gates_only_repo -> Err carrying a Denial classified code==\"perm.denied\" (governed because gates.toml present; `ro` lacks reviews:write); against forge_ungoverned_repo -> NOT perm.denied (Ok(None) governed-absent path) — proving the forge opt-in recognizes a gates.toml-only repo as governed (closing H-1) while still treating a truly-ungoverned repo as ungoverned",
      "verify": "cargo test -p but-api forge_guard_gates_toml_only_repo_is_governed",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api forge seam (approve_review) + real but-authz governance_present + real git",
        "negative_control": {
          "would_fail_if": [
            "the forge opt-in checks .gitbutler/permissions.toml ONLY (the current has_governance_config bug) so the gates.toml-only repo is treated UNGOVERNED and `ro` is permitted (no perm.denied) — this test FAILS against the unfixed code and PASSES after the swap to governance_present",
            "governance_present is replaced by an always-Ok(true) stub so the genuinely-ungoverned repo is wrongly treated governed (the ungoverned control catches this)",
            "the gates-only denial returns a code other than perm.denied (e.g. config.invalid blurred)",
            "reviews:write is wrongly granted to `ro` so the governed repo permits the verb"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "forge_gates_only_repo",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=ro: call but_api::legacy::forge::approve_review(ctx.to_sync(), \"feat\") against a repo committing ONLY .gitbutler/gates.toml (NO permissions.toml) — proves the opt-in recognizes a gates-only repo as governed (cite H-1, governance_present permissions-OR-gates config.rs:58)"
              ]
            },
            "end_state": {
              "must_observe": [
                "approve_review returns `Err`; classify_error(&err) yields a payload with `code == \"perm.denied\"`",
                "no local_review_verdicts row is written for `ro` (the verb did not run ungoverned)"
              ],
              "must_not_observe": [
                "approve_review returns `Ok` for `ro` (the fail-OPEN bug: gates-only treated ungoverned, verb runs)",
                "a verdict row written by an unauthorized `ro`",
                "a code other than perm.denied"
              ]
            }
          },
          {
            "start_ref": "forge_ungoverned_repo",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=ro: call approve_review against a repo with NO committed .gitbutler governance file — the governed-vs-ungoverned control proving the discriminator is not an always-governed stub (cite H-1 control)"
              ]
            },
            "end_state": {
              "must_observe": [
                "approve_review against the ungoverned repo is NOT denied with code==\"perm.denied\" (authorize_branch_action returns Ok(None) governed-absent; the verb's own governance-required/no-op path applies, not a perm.denied)"
              ],
              "must_not_observe": [
                "a perm.denied for the ungoverned repo (which would mean governance_present always returns true — an always-governed stub)"
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
      "description": "GIVEN forge_permissions_only_repo (permissions.toml only) and forge_both_files_repo (permissions.toml + gates.toml) WHEN approve_review is called with BUT_AGENT_HANDLE=ro against each THEN BOTH deny `ro` perm.denied — the swap from has_governance_config to governance_present did not regress the permissions-present governed cases",
      "verify": "cargo test -p but-api forge_guard_permissions_and_both_still_governed",
      "maps_to_ac": null,
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api forge seam + real but-authz + real git",
        "negative_control": {
          "would_fail_if": [
            "the swap to governance_present stops recognizing a permissions.toml-only repo as governed (regression), letting `ro` through with Ok",
            "the both-files repo is treated ungoverned after the swap",
            "either denial returns config.invalid instead of perm.denied"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "forge_permissions_only_repo",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=ro: call approve_review against a repo committing ONLY permissions.toml — regression: must remain governed (cite H-1 regression)"
              ]
            },
            "end_state": {
              "must_observe": [
                "approve_review returns `Err` classified `code == \"perm.denied\"` (permissions-only repo still governed)"
              ],
              "must_not_observe": [
                "`Ok` for `ro` (permissions-only repo wrongly treated ungoverned after the swap)"
              ]
            }
          },
          {
            "start_ref": "forge_both_files_repo",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=ro: call approve_review against a repo committing permissions.toml + gates.toml — regression: must remain governed (cite H-1 regression)"
              ]
            },
            "end_state": {
              "must_observe": [
                "approve_review returns `Err` classified `code == \"perm.denied\"` (both-files repo still governed)"
              ],
              "must_not_observe": [
                "`Ok` for `ro` (both-files repo wrongly treated ungoverned)"
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
      "description": "GIVEN the source tree after this task WHEN the build-gate greps run over forge.rs THEN `fn has_governance_config` is gone (0 matches), the fully-qualified `but_authz::governance_present` is used (1+), no `.gitbutler/permissions.toml` / `.gitbutler/gates.toml` string literal remains in forge.rs (0), and the merge-gate call sites (merge_gate::enforce_merge_gate) are preserved (3) — the bespoke fail-open discriminator is deleted and the canonical source of truth is consumed without touching the merge path [build-gate]",
      "verify": "! grep -rEn 'fn has_governance_config' crates/but-api/src/legacy/forge.rs && grep -rEn 'but_authz::governance_present' crates/but-api/src/legacy/forge.rs && ! grep -rEn '\\.gitbutler/(permissions|gates)\\.toml' crates/but-api/src/legacy/forge.rs && [ \"$(grep -cE 'merge_gate::enforce_merge_gate' crates/but-api/src/legacy/forge.rs)\" -ge 3 ]",
      "maps_to_ac": null,
      "scenario": {
        "tier": "holdout",
        "test_tier": "unit",
        "unit_test_justified": "Build-gate structural/grep invariant with zero runtime I/O: it proves the bespoke fail-open has_governance_config is DELETED, the canonical fully-qualified but_authz::governance_present is consumed, no governance-path string literal is re-derived in forge.rs (config.rs:42-43 source-of-truth rule), and the merge path call sites are untouched. The behavioral fail-closed guarantee is proven by AC-1/AC-2 integration; this gate enforces the delete-and-delegate structural property a runtime test cannot assert by inspection.",
        "verification_service": "source grep (build-gate, no runtime I/O)",
        "negative_control": {
          "would_fail_if": [
            "has_governance_config is left in place (the bespoke fail-open fn survives the fix)",
            "a `.gitbutler/permissions.toml` or `.gitbutler/gates.toml` literal is re-derived in forge.rs (the canonical source of truth is bypassed)",
            "but_authz::governance_present is never called (the fix never lands)",
            "a merge-gate call site (merge_gate::enforce_merge_gate) was removed or altered (scope creep into the merge path; count drops below 3)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "forge_both_files_repo",
            "action": {
              "actor": "ci",
              "steps": [
                "grep forge.rs: assert `fn has_governance_config` absent (0), `but_authz::governance_present` present (1+), `.gitbutler/(permissions|gates).toml` literals absent (0), `merge_gate::enforce_merge_gate` present (>=3) (cite H-1, config.rs:42-43 source-of-truth)"
              ]
            },
            "end_state": {
              "must_observe": [
                "`! grep -rEn 'fn has_governance_config' crates/but-api/src/legacy/forge.rs` returns 0 matches",
                "`grep -rEn 'but_authz::governance_present' crates/but-api/src/legacy/forge.rs` returns 1+ matches",
                "`! grep -rEn '\\.gitbutler/(permissions|gates)\\.toml' crates/but-api/src/legacy/forge.rs` returns 0 matches",
                "`grep -cE 'merge_gate::enforce_merge_gate' crates/but-api/src/legacy/forge.rs` returns >= 3"
              ],
              "must_not_observe": [
                "any `fn has_governance_config` match (the bespoke fn survived)",
                "any `.gitbutler/*.toml` literal match (governance literals re-derived)",
                "0 `but_authz::governance_present` matches (the fix absent)",
                "fewer than 3 merge_gate::enforce_merge_gate matches (merge path altered)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "a gates.toml-only governed repo denies an unauthorized principal perm.denied on approve_review, and the same principal is NOT perm.denied against a genuinely-ungoverned repo (governed-vs-ungoverned discriminator correct, closing H-1)",
      "verify": "cargo test -p but-api forge_guard_gates_toml_only_repo_is_governed",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "permissions.toml-only and both-files governed repos each still deny an unauthorized principal perm.denied (no regression from the swap to governance_present)",
      "verify": "cargo test -p but-api forge_guard_permissions_and_both_still_governed",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "has_governance_config deleted; fully-qualified but_authz::governance_present used; no `.gitbutler/*.toml` literals in forge.rs; >=3 merge_gate::enforce_merge_gate call sites preserved (delete-and-delegate, merge path untouched)",
      "verify": "! grep -rEn 'fn has_governance_config' crates/but-api/src/legacy/forge.rs && grep -rEn 'but_authz::governance_present' crates/but-api/src/legacy/forge.rs",
      "maps_to_ac": "AC-3"
    }
  ]
}
-->
</details>
