# STEER-004: Wire the payload + the exhaustive non-defaulted DenialCause enum -> class match (no `_ =>` arm; a removed variant is a COMPILE ERROR) into all constructors/gates; change branch_protected(principal,&cfg,branch) to re-call effective_authority for a gate-state-aware menu; no-handle/unknown-principal/config.invalid -> operator_required + empty menu + do_not

## What this does

Wire the steering payload and the exhaustive `match` over a `DenialCause` enum -> `DenialClass` (no `_ =>` arm) into every denial constructor and gate: missing_permission carries held + menu + ActorCorrectable; no_handle/unknown_principal/config.invalid carry OperatorRequired + empty menu + a do-not-retry do_not; and branch_protected gains a &cfg parameter so it re-calls effective_authority and builds a gate-state-aware menu via STEER-003 — all without changing a single deny/allow decision and with the full payload emitted under DryRun.

## Why

Sprint 08 (STEER — Capability-Aware Denials) · PRD UC-STEER-01, UC-STEER-03, UC-STEER-06 · Capability CAP-STEER-01. Each denial carries the correct class per (code, resolution); operator_required denials have empty menus + do-not-retry; actor_correctable denials degrade to the vertical path when no lateral move exists; do_not is positive-only; DryRun carries the full payload and persists nothing; the class match is exhaustive (a removed arm fails to compile); cargo test -p but-authz/-p but-api/-p but green; clippy clean.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-api steer_class_per_code_and_resolution && cargo test -p but governed_loop_steer_class_matrix` (class is correct per (code, principal-resolution)). Full gate set in the spec below.

## Scope

- crates/but-authz/src/authorize.rs (MODIFY — missing_permission/no_handle/unknown_principal populate class/held/menu/do_not; add the exhaustive class mapping)
- crates/but-authz/src/denial.rs (MODIFY — class mapping helper if sited on Denial)
- crates/but-authz/src/config.rs (MODIFY — ConfigError carries class=OperatorRequired + do_not)
- crates/but-api/src/commit/gate.rs (MODIFY — branch_protected(principal, &cfg, branch_name) + re-call effective_authority + classify_error carries the new fields)
- crates/but-api/src/legacy/merge_gate.rs (MODIFY — config_invalid + gate.review_required carry class + menu)
- crates/but-api/tests/commit_gate.rs (MODIFY — add class/payload assertions; do NOT weaken existing decision/mutation assertions)
- crates/but-api/tests/steer_class_wiring.rs (NEW)
- crates/but/tests/but/command/governed_loop.rs (MODIFY — add class-matrix + config.invalid-operator cases)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: STEER-004 - Wire the payload + the exhaustive non-defaulted DenialCause enum -> class match (no `_ =>` arm; a removed variant is a COMPILE ERROR) into all constructors/gates; change branch_protected(principal,&cfg,branch) to re-call effective_authority for a gate-state-aware menu; no-handle/unknown-principal/config.invalid -> operator_required + empty menu + do_not
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     M  (210 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-STEER-01, UC-STEER-03, UC-STEER-06
CAPABILITIES: CAP-STEER-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-api steer_class_per_code_and_resolution && cargo test -p but governed_loop_steer_class_matrix   |   cargo test -p but-api steer_operator_required_empty_menu_do_not && cargo test -p but governed_loop_steer_config_invalid_operator   |   cargo test -p but-api commit_gate && cargo test -p but-api steer_branch_protected_threads_cfg   |   cargo test -p but-api steer_degrade_vertical_and_do_not_positive   |   cargo test -p but-api steer_dryrun_full_payload_no_mutation && cargo test -p but governed_loop_dryrun_no_bypass   |   cargo test -p but-authz steer_denial_cause_match_is_exhaustive_compile_guard
  lint:  cargo clippy -p but-authz -p but-api -p but --all-targets

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Each denial carries the correct class per (code, resolution); operator_required denials have empty menus + do-not-retry; actor_correctable denials degrade to the vertical path when no lateral move exists; do_not is positive-only; DryRun carries the full payload and persists nothing; the class match is exhaustive (a removed arm fails to compile); cargo test -p but-authz/-p but-api/-p but green; clippy clean.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST model the classification INPUT as a Rust enum `DenialCause { MissingAuthorityResolved, BranchProtected, ReviewRequired, UnresolvedPrincipal, ConfigInvalid }` in but-authz and determine `class` by an EXHAUSTIVE, NON-DEFAULTED `match cause { ... }` over it with NO `_ =>` arm (03 §2 / invariant §9.7): MissingAuthorityResolved (`perm.denied`, principal resolved) -> ActorCorrectable; BranchProtected (`branch.protected`) -> ActorCorrectable; ReviewRequired (`gate.review_required`) -> ActorCorrectable; UnresolvedPrincipal (no-handle/unknown-principal, which CARRY the `perm.denied` code but resolve NO principal) -> OperatorRequired; ConfigInvalid (`config.invalid`) -> OperatorRequired. Adding a future variant without an arm — or removing an arm — MUST be a NON-EXHAUSTIVE-MATCH COMPILE ERROR (the type system enforces it), never a silent ActorCorrectable.
- [MUST] MUST populate `held_permissions` + `authorized_actions` + `class=ActorCorrectable` on `missing_permission(missing, held)` (authorize.rs:113) — `held` is already passed in; derive the menu via STEER-003's authorized_actions with the cfg in scope.
- [MUST] MUST set `no_handle()` (authorize.rs:146) and `unknown_principal()` (authorize.rs:163) to `class=OperatorRequired`, empty `held_permissions`, empty `authorized_actions`, and `do_not = Some("register the principal / set BUT_AGENT_HANDLE; do not retry as-is")` (security HIGH #2 — such a caller cannot self-correct in-system, so an empty menu + do-not-retry is correct, not actor_correctable).
- [MUST] MUST set `config.invalid` on BOTH carriers — `ConfigError` (config.rs) and `MergeGateError::config_invalid()` (merge_gate.rs:369) — to `class=OperatorRequired`, empty menu, and `do_not = Some("do not retry — an operator must fix the committed .gitbutler config")` (D5). ConfigError carries class+do_not only.
- [MUST] MUST change `branch_protected(principal, branch_name)` (gate.rs:257) → `branch_protected(principal, &cfg, branch_name)` and re-call `effective_authority(principal, &cfg)` to build a gate-state-aware menu (the held set is dropped on authorize's Ok path today, gate.rs:67; the cfg is in scope at the call site, gate.rs:69-74) — the menu offers a feature-branch commit + review, never the protected-ref commit (D3/C5).
- [NEVER] NEVER add a defaulted/catch-all (`_ =>`) arm to the `match cause` over `DenialCause` — exhaustiveness IS the security property (a removed or unhandled variant must fail to compile).
- [NEVER] NEVER classify no-handle/unknown-principal as actor_correctable (they would loop the agent retrying actions it has no authority for — security HIGH #2).
- [NEVER] NEVER populate held_permissions on the unresolved-principal or config.invalid paths — it is structurally empty there (UC-STEER-01 AC-3).
- [NEVER] NEVER change any deny/allow decision, denial code, exit code, or DryRun persistence semantics — the payload is additive and DryRun must still persist nothing.
- [NEVER] NEVER edit any frozen Sprint 01a–06b task file.
- [STRICTLY] STRICTLY frame `do_not` on actor_correctable denials as positive-only ('the governed path is the only route to a landed change') — it must NOT enumerate bypass mechanics by default (UC-STEER-03 AC-4 / T-STEER-015).
- [STRICTLY] STRICTLY keep the full steering payload emitted under DryRun while persisting nothing — re-prove the DryRun-no-bypass property after threading the menu (T-STEER-004).

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: class is correct per (code, principal-resolution)
- [ ] AC-2: operator_required → empty menu + do-not-retry do_not
- [ ] AC-3: branch_protected threads &cfg and produces a gate-state-aware menu (feature-branch commit + review, no protected-ref commit)
- [ ] AC-4: Actor-correctable degrades to the vertical path when no lateral move exists; do_not positive-only
- [ ] AC-5: DryRun carries the full steering payload while persisting nothing
- [ ] AC-6: The DenialCause -> class match is exhaustive and non-defaulted (removing/omitting a variant is a compile error)
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: class is correct per (code, principal-resolution) [PRIMARY]
  GIVEN: denials triggered for missing-authority perm.denied, branch.protected, gate.review_required, unknown-principal/unset-handle, and config.invalid
  WHEN:  each denial's class is read
  THEN:  the first three are `actor_correctable`; unknown-principal/no-handle AND config.invalid are `operator_required`
  TEST_TIER: integration   VERIFICATION_SERVICE: but-api
  VERIFY: cargo test -p but-api steer_class_per_code_and_resolution && cargo test -p but governed_loop_steer_class_matrix
  SCENARIO: would fail if no-handle is classified actor_correctable (the security HIGH #2 loop bug); class is a static constant (all denials tagged the same); config.invalid is actor_correctable | must observe: the ro perm.denied carries `class:"actor_correctable"`; the branch.protected carries `class:"actor_correctable"`; the unset-handle perm.denied carries `class:"operator_required"` | must NOT observe: the unset-handle denial with `class:"actor_correctable"` (the wrong class — no operator_required); a missing `class` field (none present)

AC-2: operator_required → empty menu + do-not-retry do_not
  GIVEN: a malformed committed gates.toml at the target ref and an unset/unknown principal
  WHEN:  a gated action runs
  THEN:  the denial is config.invalid (or unresolved-principal perm.denied) with `authorized_actions == []`, `held_permissions` empty or absent, and a `do_not` that says do-not-retry / requires an operator
  TEST_TIER: integration   VERIFICATION_SERVICE: but-api
  VERIFY: cargo test -p but-api steer_operator_required_empty_menu_do_not && cargo test -p but governed_loop_steer_config_invalid_operator
  SCENARIO: would fail if config.invalid carries a non-empty menu (offers actions for an operator-only fault); do_not is None on operator_required; held_permissions leaks authority fragments to an unresolved caller; the menu derivation runs on a config it could not load (panic) | must observe: `code:"config.invalid"`; `authorized_actions` == `[]`; `held_permissions` is empty or absent; `do_not` containing `do not retry` | must NOT observe: a non-empty `authorized_actions` on config.invalid; a non-empty `held_permissions` on an operator_required unresolved-principal/config.invalid denial; `do_not` absent / null; an `actor_correctable` class

AC-3: branch_protected threads &cfg and produces a gate-state-aware menu (feature-branch commit + review, no protected-ref commit)
  GIVEN: the signature change `branch_protected(principal, &cfg, branch_name)` re-calling effective_authority
  WHEN:  dev (contents:write) is denied a direct commit to protected main
  THEN:  the denial carries held_permissions (the re-derived effective set) and a menu offering a feature-branch commit + review, NEVER the protected-main commit; the deny decision is unchanged
  TEST_TIER: integration   VERIFICATION_SERVICE: but-api
  VERIFY: cargo test -p but-api commit_gate && cargo test -p but-api steer_branch_protected_threads_cfg
  SCENARIO: would fail if branch_protected still drops the held set so held_permissions is empty on a branch.protected denial; the menu reproduces the protected-main commit (signature not actually threaded to the derivation); the branch.protected decision flips | must observe: `held_permissions` contains `contents:write` (re-derived, not dropped); an `authorized_actions` entry whose `command` commits to an unprotected feature ref (e.g. `but commit` on `feat`) | must NOT observe: empty `held_permissions` on the branch.protected denial; a commit-to-protected-main affordance; the branch.protected decision becoming an allow

AC-4: Actor-correctable degrades to the vertical path when no lateral move exists; do_not positive-only
  GIVEN: a resolved actor-correctable principal holding no relevant lateral action
  WHEN:  it is denied
  THEN:  `authorized_actions == []` AND `remediation_hint` names a handoff/admin grant; `do_not` (when present) frames the governed path as the only route to landing and does NOT enumerate bypass mechanics
  TEST_TIER: integration   VERIFICATION_SERVICE: but-api
  VERIFY: cargo test -p but-api steer_degrade_vertical_and_do_not_positive
  SCENARIO: would fail if a no-lateral actor_correctable denial fabricates a menu entry it cannot run (lying menu); do_not enumerates `git`/`--no-verify` bypass mechanics by default; remediation_hint is dropped when the menu is empty | must observe: `remediation_hint` names a grant/handoff path; `authorized_actions` is empty or contains only the discovery affordance (no fabricated lateral) | must NOT observe: a `do_not` mentioning `git push`/`--no-verify` bypass mechanics (no bypass enumeration); a lateral action `ro` cannot run; a missing `remediation_hint`

AC-5: DryRun carries the full steering payload while persisting nothing
  GIVEN: a denied action run under DryRun
  WHEN:  the denial is produced
  THEN:  the denial carries class/held_permissions/authorized_actions/do_not AND no ref/object/oplog is mutated
  TEST_TIER: integration   VERIFICATION_SERVICE: but-api
  VERIFY: cargo test -p but-api steer_dryrun_full_payload_no_mutation && cargo test -p but governed_loop_dryrun_no_bypass
  SCENARIO: would fail if DryRun drops the steering payload (only legacy fields under dry-run); DryRun persists a ref/object while building the menu; the menu derivation has a side effect that writes the oplog (state does not persist as a no-op under DryRun) | must observe: the dry-run denial carries `class`, `held_permissions`, `authorized_actions`; git object count `==` the pre-action count; `refs/heads/main` ref_id `==` its pre-action value (and `refs/heads/feat` unchanged) | must NOT observe: a new git object after the denied dry-run (`0` new objects expected); main ref advanced; a dry-run denial missing the steering fields

AC-6: The DenialCause -> class match is exhaustive and non-defaulted (removing/omitting a variant is a compile error)
  GIVEN: the `DenialCause` enum { MissingAuthorityResolved, BranchProtected, ReviewRequired, UnresolvedPrincipal, ConfigInvalid } and the non-defaulted `match cause -> DenialClass` (no `_ =>` arm)
  WHEN:  a holdout trybuild-style mutation removes a `match` arm (or adds a `DenialCause` variant without an arm)
  THEN:  the workspace fails to compile with a non-exhaustive-match error (no silent default to actor_correctable); the intact match maps all five variants to a concrete class with no `_ =>` arm
  TEST_TIER: unit   VERIFICATION_SERVICE: but-authz
  UNIT_TEST_JUSTIFIED: Exhaustiveness is a compile-time property of the `match cause` over the `DenialCause` enum — the test asserts (via a match-coverage construct, with the trybuild compile-fail control owned by STEER-010) that the enum match has no `_ =>` arm so omitting a variant is a non-exhaustive-match compile error; pure type/logic with zero I/O, justified per UC-STEER-06 AC-5 / T-STEER-029.
  VERIFY: cargo test -p but-authz steer_denial_cause_match_is_exhaustive_compile_guard
  SCENARIO: would fail if the match has a `_ => ActorCorrectable` catch-all (a new DenialCause variant silently becomes actor_correctable — no compile break); DenialCause is collapsed to a `code: &str` match so the compiler cannot enforce exhaustiveness over variants; an arm is removed but the build still compiles (a `_ =>` arm absorbed it) | must observe: all five `DenialCause` variants map to a concrete class with no `_ =>` arm (5 explicit arms); the test compiles and passes (exit `0`) | must NOT observe: a `_ =>` wildcard arm in the `match cause` mapping; a `DenialCause` variant reaching a default arm

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, happy_path): A resolved-principal missing-authority perm.denied carries class actor_correctable
    VERIFY: cargo test -p but-api steer_class_per_code_and_resolution
- TC-2 (-> AC-1, error_case): An unset-handle perm.denied carries class operator_required
    VERIFY: cargo test -p but governed_loop_steer_class_matrix
- TC-3 (-> AC-2, error_case): A config.invalid denial carries authorized_actions == [] and a do-not-retry do_not
    VERIFY: cargo test -p but-api steer_operator_required_empty_menu_do_not
- TC-4 (-> AC-3, happy_path): A branch.protected denial carries held_permissions including contents:write (re-derived via &cfg)
    VERIFY: cargo test -p but-api steer_branch_protected_threads_cfg
- TC-5 (-> AC-3, edge_case): The branch.protected deny decision is unchanged after the signature change
    VERIFY: cargo test -p but-api commit_gate
- TC-6 (-> AC-4, edge_case): A no-lateral actor_correctable denial has an empty/discovery-only menu and a remediation_hint naming a grant/handoff
    VERIFY: cargo test -p but-api steer_degrade_vertical_and_do_not_positive
- TC-7 (-> AC-4, error_case): An actor_correctable do_not does not enumerate bypass mechanics
    VERIFY: cargo test -p but-api steer_degrade_vertical_and_do_not_positive
- TC-8 (-> AC-5, edge_case): A DryRun branch.protected denial carries the full steering payload and mutates no object/ref
    VERIFY: cargo test -p but-api steer_dryrun_full_payload_no_mutation
- TC-9 (-> AC-6, error_case): The `match cause` over DenialCause has no wildcard/default arm (omitting a variant is a compile error)
    VERIFY: cargo test -p but-authz steer_denial_cause_match_is_exhaustive_compile_guard

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-STEER-01
provides: a `DenialCause` enum { MissingAuthorityResolved, BranchProtected, ReviewRequired, UnresolvedPrincipal, ConfigInvalid } + an exhaustive non-defaulted `match cause -> DenialClass` (no `_ =>` arm — omitting a variant is a compile error); missing_permission populating held+menu+class=ActorCorrectable; no_handle()/unknown_principal() → class=OperatorRequired + empty held + empty menu + do-not-retry do_not; config.invalid (ConfigError + MergeGateError::config_invalid) → OperatorRequired + empty menu + do_not; branch_protected(principal, &cfg, branch_name) signature change re-calling effective_authority for the gate-state-aware menu
consumes: but_authz::authorized_actions derivation (STEER-003); Denial + DenialClass + AuthorizedAction (STEER-001); effective_authority (authorize.rs:51); missing_permission :113 / no_handle :146 / unknown_principal :163; ConfigError (config.rs:241); MergeGateError::config_invalid (merge_gate.rs:369); commit gate authorize + branch_protected (gate.rs:67-74,257)
boundary_contracts:
  - CAP-STEER-01: class is determined by an EXHAUSTIVE, non-defaulted match over (code, principal-resolution); a resolved principal lacking authority + branch.protected + gate.review_required → actor_correctable; no-handle/unknown-principal (perm.denied code) + config.invalid → operator_required with empty menu + do-not-retry do_not. The branch.protected menu derives from the same cfg the gate loaded at the target ref (branch_protected receives &cfg). DryRun carries the full payload while persisting nothing.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-authz/src/authorize.rs (MODIFY — missing_permission/no_handle/unknown_principal populate class/held/menu/do_not; add the exhaustive class mapping)
  - crates/but-authz/src/denial.rs (MODIFY — class mapping helper if sited on Denial)
  - crates/but-authz/src/config.rs (MODIFY — ConfigError carries class=OperatorRequired + do_not)
  - crates/but-api/src/commit/gate.rs (MODIFY — branch_protected(principal, &cfg, branch_name) + re-call effective_authority + classify_error carries the new fields)
  - crates/but-api/src/legacy/merge_gate.rs (MODIFY — config_invalid + gate.review_required carry class + menu)
  - crates/but-api/tests/commit_gate.rs (MODIFY — add class/payload assertions; do NOT weaken existing decision/mutation assertions)
  - crates/but-api/tests/steer_class_wiring.rs (NEW)
  - crates/but/tests/but/command/governed_loop.rs (MODIFY — add class-matrix + config.invalid-operator cases)
writeProhibited:
  - the deny/allow decision in any gate — NEVER weaken
  - DryRun persistence semantics — NEVER allow a ref/object/oplog write under DryRun
  - the no-handle/unknown-principal classification — NEVER actor_correctable
  - a defaulted/wildcard (`_ =>`) arm in the `match cause` over DenialCause — NEVER add (exhaustiveness is the compile-break guarantee)
  - .spec/prds/governance/tasks/sprint-0[1-6]* — frozen task files

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
  - crates/but-authz/src/authorize.rs (lines 24-31, 51-58, 104-172): authorize :24 (drops held on Ok); effective_authority :51; missing_permission(missing, held) :113 (already has held -> populate menu); no_handle :146 / unknown_principal :163 (resolve no principal -> operator_required + empty + do_not). Site the `DenialCause` enum here so the `match cause -> DenialClass` is non-defaulted and exhaustive.
  - crates/but-api/src/commit/gate.rs (lines 55-94, 257-292): enforce_commit_gate_for_target :55, authorize(p, ContentsWrite) :67, the branch-protection predicate :69-74 (cfg in scope), branch_protected :257 (the signature to change to (principal, &cfg, branch_name)), classify_error :81.
  - crates/but-authz/src/config.rs (lines 239-265): ConfigError :241 + code() :255 → set class=OperatorRequired + do_not.
  - crates/but-api/src/legacy/merge_gate.rs (lines 365-376): config_invalid() :369 → set class=OperatorRequired + empty menu + do_not (the merge-path config.invalid carrier).
  - crates/but-api/tests/commit_gate.rs (lines 61-98, 163-251): commit_gate_readonly_and_bad_handle_denied + commit_gate_malformed_partial_and_dryrun — the (handle, label) matrix and the DryRun-no-mutation assertions to extend with class/payload checks.
  - crates/but/tests/but/command/governed_loop.rs (lines 232-261, 365-498): governed_loop_unset_handle_failclosed + the assert_denial/CliErrorEnvelope helpers — extend to read class and assert operator_required on the unset-handle path.

--------------------------------------------------------------------------------
CODE PATTERN
--------------------------------------------------------------------------------
pattern: A `DenialCause` enum + a non-defaulted `match cause { MissingAuthorityResolved => .., BranchProtected => .., ReviewRequired => .., UnresolvedPrincipal => .., ConfigInvalid => .. }` returning DenialClass with NO `_ =>` arm; constructors set class/held/menu/do_not at build time; branch_protected gains `&cfg` and calls effective_authority + authorized_actions; operator_required paths set empty Vecs + Some(do_not).
pattern_source: crates/but-authz/src/authorize.rs:113 (missing_permission already receives held — the natural site to populate the menu); crates/but-api/src/commit/gate.rs:69-74 (cfg is already in scope at the branch_protected call — thread it in).
anti_pattern: A `match cause { ... _ => ActorCorrectable }` wildcard (silently misclassifies a future variant — defeats the compile-break guarantee); classifying UnresolvedPrincipal as actor_correctable (loops the agent); building branch_protected's menu from a re-loaded cfg instead of the gate's cfg (menu/gate divergence).
references: 03-technical-requirements-delta.md §2 (class mapping) + §3 (branch_protected/C5); 05-delta-replan.md D2/D3/D5; 02-uc-steer.md UC-STEER-01 AC-3/4 + UC-STEER-03 AC-1/2/3/4 + UC-STEER-06 AC-5
interaction_notes:
  - Consumes STEER-003's authorized_actions for the actor_correctable menu; STEER-005 serializes the wired payload at the CLI sites; STEER-009 replays each offered action; the exhaustiveness compile-guard pairs with STEER-010's grep.

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
depends_on: STEER-003
blocks: STEER-005, STEER-006, STEER-007, STEER-009

CODING STANDARDS: crates/AGENTS.md, crates/but/AGENTS.md, crates/WORKSPACE_MODEL.md, RULES.md
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "description": "GIVEN denials per cause WHEN class is read THEN missing-authority/branch.protected/review_required \u2192 actor_correctable; no-handle/unknown/config.invalid \u2192 operator_required",
      "verify": "cargo test -p but-api steer_class_per_code_and_resolution && cargo test -p but governed_loop_steer_class_matrix"
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "GIVEN an operator_required cause WHEN denied THEN authorized_actions == [], held_permissions is empty or absent, and do_not says do-not-retry",
      "verify": "cargo test -p but-api steer_operator_required_empty_menu_do_not && cargo test -p but governed_loop_steer_config_invalid_operator"
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "GIVEN branch_protected(principal,&cfg,branch) WHEN a branch.protected denial is built THEN held_permissions is re-derived and the menu offers a feature-branch commit, not the protected-ref commit",
      "verify": "cargo test -p but-api commit_gate && cargo test -p but-api steer_branch_protected_threads_cfg"
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "description": "GIVEN a no-lateral actor_correctable caller WHEN denied THEN the menu degrades to the vertical path and do_not is positive-only",
      "verify": "cargo test -p but-api steer_degrade_vertical_and_do_not_positive"
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "description": "GIVEN a DryRun denial WHEN produced THEN it carries the full payload and persists nothing",
      "verify": "cargo test -p but-api steer_dryrun_full_payload_no_mutation && cargo test -p but governed_loop_dryrun_no_bypass"
    },
    {
      "id": "AC-6",
      "type": "acceptance_criterion",
      "description": "GIVEN the class match WHEN an arm is removed THEN compilation fails (non-defaulted exhaustiveness)",
      "verify": "cargo test -p but-authz steer_denial_cause_match_is_exhaustive_compile_guard"
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "missing-authority perm.denied \u2192 actor_correctable",
      "maps_to_ac": "AC-1",
      "verify": "cargo test -p but-api steer_class_per_code_and_resolution"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "unset-handle perm.denied \u2192 operator_required",
      "maps_to_ac": "AC-1",
      "verify": "cargo test -p but governed_loop_steer_class_matrix"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "config.invalid \u2192 empty menu + do-not-retry",
      "maps_to_ac": "AC-2",
      "verify": "cargo test -p but-api steer_operator_required_empty_menu_do_not"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "branch.protected held_permissions includes contents:write",
      "maps_to_ac": "AC-3",
      "verify": "cargo test -p but-api steer_branch_protected_threads_cfg"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "branch.protected decision unchanged",
      "maps_to_ac": "AC-3",
      "verify": "cargo test -p but-api commit_gate"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "description": "no-lateral degrade to vertical path",
      "maps_to_ac": "AC-4",
      "verify": "cargo test -p but-api steer_degrade_vertical_and_do_not_positive"
    },
    {
      "id": "TC-7",
      "type": "test_criterion",
      "description": "actor_correctable do_not is positive-only",
      "maps_to_ac": "AC-4",
      "verify": "cargo test -p but-api steer_degrade_vertical_and_do_not_positive"
    },
    {
      "id": "TC-8",
      "type": "test_criterion",
      "description": "DryRun full payload + no mutation",
      "maps_to_ac": "AC-5",
      "verify": "cargo test -p but-api steer_dryrun_full_payload_no_mutation"
    },
    {
      "id": "TC-9",
      "type": "test_criterion",
      "description": "class match has no wildcard arm",
      "maps_to_ac": "AC-6",
      "verify": "cargo test -p but-authz steer_denial_cause_match_is_exhaustive_compile_guard"
    }
  ]
}
-->
