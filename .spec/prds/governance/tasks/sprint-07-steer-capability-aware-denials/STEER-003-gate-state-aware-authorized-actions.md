# STEER-003: Gate-state-aware authorized_actions derivation: effective_set ∩ table minus the failed (route,predicate,ref), intent-scoped via the curated AFFORDANCE_MAP, self-approve excluded on own branch, all command/effect text from a closed &'static str CATALOG + appended discovery affordance

## What this does

Implement the gate-state-aware authorized_actions derivation: intersect the caller's effective set with the ROUTE_AUTHORITY_TABLE, intent-scope it via the curated AFFORDANCE_MAP, subtract the (route, predicate, ref) that actually fired (so a branch.protected denial offers a feature-branch commit + review, not the protected-ref commit), exclude self-approve on own-branch denials, render every entry from a closed &'static str CATALOG, and append the degradable discovery affordance.

## Why

Sprint 07 (STEER — Capability-Aware Denials) · PRD UC-STEER-02, UC-STEER-06 · Capability CAP-STEER-01. For any actor-correctable denial, authorized_actions ⊆ {routes the caller is authorized to run}, is scoped to the denied intent, lists each entry as a {command, effect} catalog pair, never reproduces the failed (route,predicate,ref), and never includes `but review approve` on an own-branch denial; a reviewer denied a commit sees runnable `but review request-changes`/`comment`; all text is catalog-sourced; cargo test -p but-authz + -p but green; clippy clean.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-authz steer_menu_subset_of_effective_set` (Menu ⊆ effective set ∩ table; no entry requires an unheld authority). Full gate set in the spec below.

## Scope

- crates/but-authz/src/menu.rs (NEW — authorized_actions derivation + AFFORDANCE_MAP + CATALOG, beside the route table)
- crates/but-authz/src/lib.rs (MODIFY — pub use authorized_actions, AFFORDANCE_MAP, CATALOG)
- crates/but-authz/tests/steer_menu.rs (NEW — subset/intent/closed-catalog proofs)
- crates/but-api/tests/steer_menu_gate_state.rs (NEW — branch.protected feature-not-protected proof against real git)
- crates/but/tests/but/command/governed_loop.rs (MODIFY — add steer reviewer-menu-runnable + discovery cases; do NOT weaken existing assertions)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: STEER-003 - Gate-state-aware authorized_actions derivation: effective_set ∩ table minus the failed (route,predicate,ref), intent-scoped via the curated AFFORDANCE_MAP, self-approve excluded on own branch, all command/effect text from a closed &'static str CATALOG + appended discovery affordance
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     L  (240 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-STEER-02, UC-STEER-06
CAPABILITIES: CAP-STEER-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-authz steer_menu_subset_of_effective_set   |   cargo test -p but-api steer_branch_protected_menu_feature_not_protected && cargo test -p but governed_loop_steer_protected_menu   |   cargo test -p but governed_loop_steer_reviewer_menu_runnable_no_self_approve   |   cargo test -p but-authz steer_menu_intent_scoped_entries_well_formed   |   cargo test -p but-authz steer_menu_text_is_closed_catalog_constants   |   cargo test -p but governed_loop_steer_menu_includes_discovery
  lint:  cargo clippy -p but-authz -p but-api -p but --all-targets

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
For any actor-correctable denial, authorized_actions ⊆ {routes the caller is authorized to run}, is scoped to the denied intent, lists each entry as a {command, effect} catalog pair, never reproduces the failed (route,predicate,ref), and never includes `but review approve` on an own-branch denial; a reviewer denied a commit sees runnable `but review request-changes`/`comment`; all text is catalog-sourced; cargo test -p but-authz + -p but green; clippy clean.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST derive the menu per 03 §3: `held = effective_authority(principal,&cfg)` (the cfg PASSED IN by the gate, never re-loaded); `usable = {r ∈ ROUTE_AUTHORITY_TABLE | r.required_authority ⊆ held}`; `cands = AFFORDANCE_MAP[denied]`; `scoped = {c ∈ cands | c.route ∈ usable AND c does NOT reproduce (route_d, predicate_d) at ref_d}` — the C5 subtraction; render via CATALOG and append `CATALOG[discovery]`.
- [MUST] MUST be gate-state-aware: for a `branch.protected` denial, the affordance is a commit to a DIFFERENT, unprotected FEATURE ref + review — NEVER the protected-ref commit that just failed (the C5 subtraction). A pure `required_authority ⊆ held` is unsound here because branch protection is `authority ∧ ¬protected` and the caller still holds `contents:write`.
- [MUST] MUST EXCLUDE `but review approve` from authorized_actions when the denial targets the CALLER'S OWN BRANCH (the L1 self-approve exclusion, security HIGH #3) — yield `but review request-changes` + `comment` instead; this is a code contract, never deferred to the L2 primer.
- [MUST] MUST draw EVERY command and effect string from a CLOSED, code-owned `&'static str` CATALOG — never `format!`, interpolated, config-sourced, principal-supplied, or model-generated (invariant §9.2). The curated `AFFORDANCE_MAP` (~7 gated routes, each naming a route in a SUCCEEDING context) is the one curated piece, sited beside the table.
- [MUST] MUST keep the menu INTENT-SCOPED: `AFFORDANCE_MAP[denied]` yields only categories relevant to the denied intent (a denied protected-commit surfaces landing/review affordances + discovery, NOT the whole command catalog).
- [NEVER] NEVER offer an action that is itself denied for this caller at this ref (no lying menu, invariant §9.1) — the subtraction of the (route,predicate,ref) that fired is the mechanism.
- [NEVER] NEVER surface `but review approve` on a denial targeting the caller's own branch (self-approval path).
- [NEVER] NEVER construct any menu `command`/`effect`/`do_not` text via `format!`/interpolation/config values (invariant §9.2) — only CATALOG constants and already-typed identifiers.
- [NEVER] NEVER re-load cfg inside the derivation — use the cfg the gate already loaded at the target ref (the same-cfg/ref-by-construction property, M2).
- [NEVER] NEVER edit any frozen Sprint 01a–06b task file.
- [NEVER] NEVER claim R15 closed (SA-7): the closed-catalog invariant (§9.2) covers ONLY the NEW steering fields (command/effect/do_not). The pre-existing config-derived interpolation in `message`/`unmet[]` (principal/branch names, `gates.toml` required_groups — attacker-influenceable per R13) is the NAMED accepted-leak R15 (enrichment §9.3), whose bounding/sanitization mitigation is EXPLICITLY OUT OF SCOPE for this sprint (named, not silently gapped).
- [STRICTLY] STRICTLY make the discovery affordance DEGRADABLE: append `CATALOG[discovery]` (the `but perm list` self-scoped command from Sprint 05) only when that verb exists; if no discovery verb is available, OMIT it rather than emit a phantom command (preserving no-lying-menu, C3/D8).
- [STRICTLY] STRICTLY keep AFFORDANCE_MAP and CATALOG sited in but-authz beside ROUTE_AUTHORITY_TABLE so STEER-010's coverage grep (every table route has an AFFORDANCE_MAP entry not naming the denied route) and closed-catalog grep can target them.
- [STRICTLY] STRICTLY map the SPRINT.md gate step 2 cleanly: a reviewer committing to protected main produces `perm.denied` (missing `contents:write`), not `branch.protected` (authority is checked before the protection predicate) — AC-3's reviewer-denied-commit menu is exactly this case (request-changes/comment, no approve).

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: Menu ⊆ effective set ∩ table; no entry requires an unheld authority
- [ ] AC-2: Gate-state-aware: branch.protected offers a feature-branch commit + review, NOT the protected-ref commit just denied
- [ ] AC-3: Reviewer denied a commit sees runnable review actions, and following one returns exit 0; self-approve excluded on own branch
- [ ] AC-4: Each entry is {command, effect}; menu is intent-scoped, not the whole catalog
- [ ] AC-5: All command/effect text comes from the closed catalog (no format!/interpolation/config-sourced)
- [ ] AC-6: Discovery affordance appended and degradable
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: Menu ⊆ effective set ∩ table; no entry requires an unheld authority [PRIMARY]
  GIVEN: a principal holding {contents:read, comments:write} hitting a denial
  WHEN:  authorized_actions is derived from effective_authority ∩ ROUTE_AUTHORITY_TABLE
  THEN:  every listed entry's required authority ⊆ the caller's held set; no entry requires an authority the caller does not hold
  TEST_TIER: integration   VERIFICATION_SERVICE: but-authz
  VERIFY: cargo test -p but-authz steer_menu_subset_of_effective_set
  SCENARIO: would fail if the derivation returns the whole CATALOG ignoring the held set (lying menu); an entry requiring merge is offered to a caller without merge; the menu is a static empty Vec (stub) | must observe: a menu entry whose route requires reviews:write (e.g. `but review request-changes`); every entry's required authority ∈ {comments:write, reviews:write} | must NOT observe: a `but pr merge` entry (requires merge, unheld); an empty menu

AC-2: Gate-state-aware: branch.protected offers a feature-branch commit + review, NOT the protected-ref commit just denied
  GIVEN: a `branch.protected` denial where the caller HOLDS contents:write but the target ref (main) is protected
  WHEN:  authorized_actions is derived with the failed (commit route, branch-protected predicate, main ref) subtracted
  THEN:  the menu offers a commit to an unprotected FEATURE branch (a different ref) + review affordances, and EXCLUDES the protected-ref commit the caller just failed
  TEST_TIER: integration   VERIFICATION_SERVICE: but-api
  VERIFY: cargo test -p but-api steer_branch_protected_menu_feature_not_protected && cargo test -p but governed_loop_steer_protected_menu
  SCENARIO: would fail if a pure `required_authority ⊆ held` offers the protected-ref commit the caller still holds contents:write for (lying menu); the subtraction is a no-op so the failed route reappears; the menu lists the same `commit main` route that just denied | must observe: an `authorized_actions` entry whose `command` commits to an unprotected feature ref (e.g. `but commit` on `feat`); an `authorized_actions` entry with `command` `but review request-changes` | must NOT observe: a commit-to-protected-`main` affordance (no feature-ref scoping); the route that just produced branch.protected

AC-3: Reviewer denied a commit sees runnable review actions, and following one returns exit 0; self-approve excluded on own branch
  GIVEN: principal `rev`/`reviewer` (reviews:write, no contents:write) attempts a commit on its OWN branch
  WHEN:  the commit is denied and authorized_actions is derived
  THEN:  the menu includes `but review request-changes` and `comment` (runnable for this caller), following one returns exit 0, and `but review approve` is ABSENT (own-branch self-approve exclusion)
  TEST_TIER: integration   VERIFICATION_SERVICE: but-cli
  VERIFY: cargo test -p but governed_loop_steer_reviewer_menu_runnable_no_self_approve
  SCENARIO: would fail if `but review approve` appears for the caller's own branch (self-approval path); a listed `but review request-changes` is itself denied when run (lying menu); the menu is empty so nothing can be followed | must observe: `but review request-changes` present in authorized_actions; `but review comment` present; the followed `request-changes` command exits 0 (governed action succeeds) | must NOT observe: `but review approve` in authorized_actions; the followed command re-denied with perm.denied; an empty authorized_actions array

AC-4: Each entry is {command, effect}; menu is intent-scoped, not the whole catalog
  GIVEN: an actor-correctable denial for a commit-to-protected by a review-capable principal
  WHEN:  authorized_actions is rendered
  THEN:  each entry has a literal `but …` `command` and a non-empty `effect`; the menu lists only landing/review/discovery affordances for the denied intent — it does NOT list unrelated admin/group verbs
  TEST_TIER: integration   VERIFICATION_SERVICE: but-authz
  VERIFY: cargo test -p but-authz steer_menu_intent_scoped_entries_well_formed
  SCENARIO: would fail if the menu lists every catalog command regardless of intent (no AFFORDANCE_MAP scoping); an entry has an empty effect string; an entry's command is not a literal `but` command | must observe: each entry.command starts with `but `; each entry `effect` has length `>= 1` (non-empty); a review-category entry present (an entry whose `command` is `but review request-changes`) | must NOT observe: a `but perm grant` / admin-write verb (unrelated intent — no scoping); an entry with effect == ""

AC-5: All command/effect text comes from the closed catalog (no format!/interpolation/config-sourced)
  GIVEN: the AFFORDANCE_MAP/CATALOG derivation site
  WHEN:  the closed-catalog build-gate greps the menu construction
  THEN:  every `command`/`effect` is an `&'static str` catalog constant; no `format!`, interpolated, or config-sourced text appears in authorized_actions construction
  TEST_TIER: unit   VERIFICATION_SERVICE: but-authz
  UNIT_TEST_JUSTIFIED: Pure constness/closedness check over CATALOG entries — asserting every command/effect resolves to an `&'static str` constant and that the derivation never interpolates is a static-data property with zero I/O; the build-gate grep counterpart is owned by STEER-010.
  VERIFY: cargo test -p but-authz steer_menu_text_is_closed_catalog_constants
  SCENARIO: would fail if a menu entry's effect is built with `format!("... {branch}")` (config-sourced/injection surface); a command interpolates a principal/branch name; CATALOG entries are String not &'static str | must observe: every `CATALOG` command literal is a backticked `&'static str` (e.g. `but review request-changes`); a derived entry `==` a `CATALOG` constant exactly | must NOT observe: a menu entry containing an interpolated branch/principal substring (no closed-catalog constant); a String-typed (owned) command field

AC-6: Discovery affordance appended and degradable
  GIVEN: an actor-correctable denial with the Sprint 05 `but perm list` discovery verb present
  WHEN:  the menu is derived
  THEN:  authorized_actions includes the `but perm list` self-scoped discovery command as the appended discovery affordance; if no discovery verb exists, it is omitted (no phantom command)
  TEST_TIER: integration   VERIFICATION_SERVICE: but-cli
  VERIFY: cargo test -p but governed_loop_steer_menu_includes_discovery
  SCENARIO: would fail if the discovery affordance is hardcoded even when the verb is absent (phantom command — lying menu); the appended entry is not `but perm list` (the shipped Sprint 05 verb); discovery is never appended | must observe: `but perm list` present in authorized_actions | must NOT observe: a discovery command that is not a real `but` verb (a placeholder/phantom command); the discovery entry absent on an actor-correctable denial

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, happy_path): Every authorized_actions entry's required authority is a subset of the caller's held set
    VERIFY: cargo test -p but-authz steer_menu_subset_of_effective_set
- TC-2 (-> AC-2, happy_path): A branch.protected menu contains a feature-branch commit affordance on a different unprotected ref
    VERIFY: cargo test -p but-api steer_branch_protected_menu_feature_not_protected
- TC-3 (-> AC-2, error_case): A branch.protected menu does NOT contain the protected-ref commit just denied
    VERIFY: cargo test -p but-api steer_branch_protected_menu_feature_not_protected
- TC-4 (-> AC-3, happy_path): A reviewer-denied-commit menu lists `but review request-changes` and following it returns exit 0
    VERIFY: cargo test -p but governed_loop_steer_reviewer_menu_runnable_no_self_approve
- TC-5 (-> AC-3, error_case): A reviewer-denied-commit menu on the caller's own branch does NOT contain `but review approve`
    VERIFY: cargo test -p but governed_loop_steer_reviewer_menu_runnable_no_self_approve
- TC-6 (-> AC-4, happy_path): Each authorized_actions entry has a `but `-prefixed command and a non-empty effect
    VERIFY: cargo test -p but-authz steer_menu_intent_scoped_entries_well_formed
- TC-7 (-> AC-4, error_case): A commit-to-protected menu lists no admin-write verb (intent-scoped)
    VERIFY: cargo test -p but-authz steer_menu_intent_scoped_entries_well_formed
- TC-8 (-> AC-5, edge_case): Every CATALOG command/effect is an &'static str constant and a derived entry equals a CATALOG constant
    VERIFY: cargo test -p but-authz steer_menu_text_is_closed_catalog_constants
- TC-9 (-> AC-6, happy_path): An actor-correctable denial's menu includes the `but perm list` discovery affordance
    VERIFY: cargo test -p but governed_loop_steer_menu_includes_discovery
- TC-10 (-> AC-3, edge_case): A reviewer committing to protected main is denied `perm.denied` (missing contents:write), not `branch.protected`, and its menu still lists `but review request-changes`/`comment` and excludes `but review approve` (RR-5 gate-step map)
    VERIFY: cargo test -p but governed_loop_steer_reviewer_menu_runnable_no_self_approve

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-STEER-01
provides: but_authz::authorized_actions(principal, denied={route_d,predicate_d,ref_d}, cfg) -> Vec<AuthorizedAction>; but_authz::AFFORDANCE_MAP (curated, ~7 gated routes); but_authz::CATALOG (closed &'static str command/effect entries incl. discovery); the C5 subtraction (menu never reproduces the failed route/predicate at the failed ref); the L1 self-approve exclusion on own-branch denials
consumes: but_authz::ROUTE_AUTHORITY_TABLE + Route (STEER-002); but_authz::AuthorizedAction (STEER-001); but_authz::effective_authority(principal, &cfg) (authorize.rs:51); but_authz::AuthoritySet::contains (authority.rs:280); but_authz::GovConfig (config.rs:84)
boundary_contracts:
  - CAP-STEER-01 (producer): authorized_actions is the intersection of the caller's effective set and the table, intent-scoped via AFFORDANCE_MAP, with the failed (route,predicate,ref) subtracted and self-approve excluded on own-branch; every entry's command/effect is a closed-catalog &'static str; the discovery affordance is appended (degradable). Same cfg/ref the gate judged against is supplied by STEER-004 — proven at runtime by STEER-009, not here.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-authz/src/menu.rs (NEW — authorized_actions derivation + AFFORDANCE_MAP + CATALOG, beside the route table)
  - crates/but-authz/src/lib.rs (MODIFY — pub use authorized_actions, AFFORDANCE_MAP, CATALOG)
  - crates/but-authz/tests/steer_menu.rs (NEW — subset/intent/closed-catalog proofs)
  - crates/but-api/tests/steer_menu_gate_state.rs (NEW — branch.protected feature-not-protected proof against real git)
  - crates/but/tests/but/command/governed_loop.rs (MODIFY — add steer reviewer-menu-runnable + discovery cases; do NOT weaken existing assertions)
writeProhibited:
  - the gate deny/allow decision — NEVER weaken (the menu derives FROM the denial, never changes it)
  - the ROUTE_AUTHORITY_TABLE contents/decision mapping (STEER-002 owns it) — consume only
  - any menu text via format!/interpolation/config values — NEVER (closed catalog only)
  - the self-approve exclusion — NEVER leave to the L2 primer
  - .spec/prds/governance/tasks/sprint-0[1-6]* — frozen task files

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
  - crates/but-authz/src/authorize.rs (lines 51-58, 104-136): effective_authority(principal, &cfg) :51 (the held set) and missing_permission(missing, held) :113 — the derivation's held input and the existing held-summary prose it formalizes.
  - crates/but-authz/src/authority.rs (lines 165-322): AuthoritySet::contains :280 / iter :320 — the `required_authority ⊆ held` test.
  - crates/but-api/src/commit/gate.rs (lines 55-78, 159-170): the commit route + branch-protection predicate (the (route_d, predicate_d, ref_d) for a branch.protected denial) and branch_protected :159 — STEER-004 threads cfg here; STEER-003 consumes the subtraction inputs.
  - crates/but/tests/but/command/governed_loop.rs (lines 44-101, 263-347, 365-498): the reviewer-denied-commit flow + the governed_loop fixture + assert_denial/parse_cli_error_envelope — the harness style to replay a listed action to exit 0.
  - .spec/prds/governance/enrichments/v1.4.0-capability-aware-denials/03-technical-requirements-delta.md (lines 73-118): §3 the derivation pseudo-code (the C5 subtraction) and §5 the AFFORDANCE_MAP table (the curated ~7-route intent map naming succeeding contexts).

--------------------------------------------------------------------------------
CODE PATTERN
--------------------------------------------------------------------------------
pattern: A pure derivation fn `authorized_actions(principal, denied, cfg) -> Vec<AuthorizedAction>` that intersects effective_authority with ROUTE_AUTHORITY_TABLE, looks up AFFORDANCE_MAP[denied] for intent categories (each naming a route in a SUCCEEDING context), filters out the (route_d,predicate_d,ref_d) that fired and the self-approve own-branch case, maps to CATALOG &'static str entries, and appends CATALOG[discovery] when available.
pattern_source: crates/but-authz/src/authorize.rs:113 (missing_permission already computes a held summary by mapping Authority::name over held — the same effective-set source the menu intersects); crates/but-authz/src/authority.rs:46 (Authority::ALL static-slice pattern for AFFORDANCE_MAP/CATALOG).
anti_pattern: A pure `required_authority ⊆ held` with NO subtraction (offers the branch.protected commit the caller still holds contents:write for — the C5 lying menu); building effect strings with `format!("commit to {branch}")` (closed-catalog violation + injection surface); emitting the whole CATALOG ignoring AFFORDANCE_MAP (menu bloat / irrelevant-advice failure).
references: 03-technical-requirements-delta.md §3 + §5; 02-uc-steer.md UC-STEER-02 (AC-1..7) + UC-STEER-06 AC-3; 05-delta-replan.md UC-STEER-02/06 C5
interaction_notes:
  - STEER-004 supplies the cfg/ref and the (route_d, predicate_d, ref_d) tuple and calls authorized_actions from the constructors/gates; STEER-009 proves at runtime that every offered action succeeds in its stated context; STEER-010 adds the closed-catalog + AFFORDANCE_MAP-coverage greps.
  - RR-5 gate-step map: SPRINT.md human-gate step 2 ('reviewer commits to protected main') resolves to a `perm.denied` (the reviewer lacks `contents:write`), NOT `branch.protected` — the commit gate's authority check fails BEFORE the branch-protection predicate is reached. Its menu therefore still surfaces `but review request-changes`/`comment` (reviews:write held) and excludes `but review approve` (own-branch self-approve exclusion). AC-3 already proves this shape; this note maps the gate step to the right denial code (no structural change).
  - SA-7 R15 out-of-scope: bounding/sanitizing the config-derived strings interpolated into `message`/`unmet[]` is the NAMED accepted-leak R15 (enrichment §9.3) and is NOT owned by this sprint — the closed-catalog invariant guarantees only the NEW fields are catalog-sourced; R15 mitigation is named and deferred, not silently gapped.

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
depends_on: STEER-002
blocks: STEER-004, STEER-010

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
      "description": "GIVEN a principal's held set WHEN the menu is derived THEN every entry's required authority \u2286 held",
      "verify": "cargo test -p but-authz steer_menu_subset_of_effective_set"
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "GIVEN a branch.protected denial WHEN the failed (route,predicate,ref) is subtracted THEN the menu offers a feature-branch commit + review, not the protected-ref commit",
      "verify": "cargo test -p but-api steer_branch_protected_menu_feature_not_protected && cargo test -p but governed_loop_steer_protected_menu"
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "GIVEN a reviewer denied a commit on its own branch WHEN the menu is derived THEN it lists runnable review actions (following one \u2192 exit 0) and excludes `but review approve`",
      "verify": "cargo test -p but governed_loop_steer_reviewer_menu_runnable_no_self_approve"
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "description": "GIVEN an actor-correctable denial WHEN rendered THEN each entry is {command, effect} and the menu is intent-scoped",
      "verify": "cargo test -p but-authz steer_menu_intent_scoped_entries_well_formed"
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "description": "GIVEN the derivation site WHEN inspected THEN all command/effect text is closed-catalog &'static str, no format!/interpolation",
      "verify": "cargo test -p but-authz steer_menu_text_is_closed_catalog_constants"
    },
    {
      "id": "AC-6",
      "type": "acceptance_criterion",
      "description": "GIVEN the Sprint 05 discovery verb present WHEN the menu is derived THEN `but perm list` is appended (degradable to omission)",
      "verify": "cargo test -p but governed_loop_steer_menu_includes_discovery"
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "Menu entry authority \u2286 held",
      "maps_to_ac": "AC-1",
      "verify": "cargo test -p but-authz steer_menu_subset_of_effective_set"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "branch.protected menu has a feature-branch commit affordance",
      "maps_to_ac": "AC-2",
      "verify": "cargo test -p but-api steer_branch_protected_menu_feature_not_protected"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "branch.protected menu excludes the protected-ref commit",
      "maps_to_ac": "AC-2",
      "verify": "cargo test -p but-api steer_branch_protected_menu_feature_not_protected"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "reviewer menu lists request-changes and following it \u2192 exit 0",
      "maps_to_ac": "AC-3",
      "verify": "cargo test -p but governed_loop_steer_reviewer_menu_runnable_no_self_approve"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "reviewer own-branch menu excludes `but review approve`",
      "maps_to_ac": "AC-3",
      "verify": "cargo test -p but governed_loop_steer_reviewer_menu_runnable_no_self_approve"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "description": "each entry is a well-formed {command, effect}",
      "maps_to_ac": "AC-4",
      "verify": "cargo test -p but-authz steer_menu_intent_scoped_entries_well_formed"
    },
    {
      "id": "TC-7",
      "type": "test_criterion",
      "description": "commit-to-protected menu lists no admin-write verb",
      "maps_to_ac": "AC-4",
      "verify": "cargo test -p but-authz steer_menu_intent_scoped_entries_well_formed"
    },
    {
      "id": "TC-8",
      "type": "test_criterion",
      "description": "menu text is closed-catalog constants",
      "maps_to_ac": "AC-5",
      "verify": "cargo test -p but-authz steer_menu_text_is_closed_catalog_constants"
    },
    {
      "id": "TC-9",
      "type": "test_criterion",
      "description": "menu includes the discovery affordance",
      "maps_to_ac": "AC-6",
      "verify": "cargo test -p but governed_loop_steer_menu_includes_discovery"
    },
    {
      "id": "TC-10",
      "type": "test_criterion",
      "description": "A reviewer committing to protected main is denied `perm.denied` (missing contents:write), not `branch.protected`, and its menu still lists `but review request-changes`/`comment` and excludes `but review approve` (RR-5 gate-step map)",
      "maps_to_ac": "AC-3",
      "verify": "cargo test -p but governed_loop_steer_reviewer_menu_runnable_no_self_approve"
    }
  ]
}
-->
