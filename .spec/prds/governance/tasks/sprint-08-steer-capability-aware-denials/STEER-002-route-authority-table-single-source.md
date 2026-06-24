# STEER-002: Route enum + single-source ROUTE_AUTHORITY_TABLE in but-authz; compose non-authority predicates around it; reconcile forge authorize_branch_action match (incl. `other => authorize(p, other)`); preserve the AUTHORITY_POSITIVE_PATTERN honesty grep

## What this does

Promote the scattered, heterogeneous per-gate authority checks (commit authorize+predicate, merge authorize+review-engine, the forge `authorize_branch_action` match incl. its `other =>` arm, and the admin authorize) into ONE enumerable `Route` enum + `ROUTE_AUTHORITY_TABLE` in but-authz that maps each route to its required Authority, its literal `but` command, and a one-line effect — keeping the non-authority predicates composed around the table, and keeping the literal `authorize`/`Authority` calls that the honesty grep asserts.

## Why

Sprint 08 (STEER — Capability-Aware Denials) · PRD UC-STEER-06 · Capability CAP-STEER-01. A single `ROUTE_AUTHORITY_TABLE` symbol is referenced by both the gate call sites and (forward-declared for) the menu module; every gated route is a row; the forge match is reconciled into explicit rows with the catch-all arm enumerated; the branch-protection and review-requirement predicates remain separate; the deny/allow decision is unchanged (all existing gate tests + invariant_build_gates green); clippy clean.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-authz steer_route_table_covers_every_gated_route` (One ROUTE_AUTHORITY_TABLE symbol referenced by gate and menu module; every gated route present). Full gate set in the spec below.

## Scope

- crates/but-authz/src/route.rs (NEW — Route enum + ROUTE_AUTHORITY_TABLE)
- crates/but-authz/src/lib.rs (MODIFY — mod route; pub use Route, ROUTE_AUTHORITY_TABLE)
- crates/but-api/src/commit/gate.rs (MODIFY — look up required Authority via the table; keep literal authorize call + branch-protection predicate)
- crates/but-api/src/legacy/merge_gate.rs (MODIFY — Merge route via table; keep authorize + review engine)
- crates/but-api/src/legacy/forge.rs (MODIFY — reconcile authorize_branch_action match incl. `other =>` into table-backed rows)
- crates/but-api/src/legacy/config_mutate.rs (MODIFY — admin route via table)
- crates/but-authz/tests/steer_route_table.rs (NEW — table coverage proofs)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: STEER-002 - Route enum + single-source ROUTE_AUTHORITY_TABLE in but-authz; compose non-authority predicates around it; reconcile forge authorize_branch_action match (incl. `other => authorize(p, other)`); preserve the AUTHORITY_POSITIVE_PATTERN honesty grep
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     L  (270 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-STEER-06
CAPABILITIES: CAP-STEER-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-authz steer_route_table_covers_every_gated_route   |   cargo test -p but-api forge_guard && cargo test -p but-authz steer_route_table_includes_forge_routes   |   cargo test -p but-api commit_gate && cargo test -p but-api merge_gate && cargo test -p but-api forge_guard && cargo test -p but-api admin_write_guard && cargo test -p but governed_loop   |   cargo test -p but-authz --test invariant_build_gates
  lint:  cargo clippy -p but-authz -p but-api --all-targets

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
A single `ROUTE_AUTHORITY_TABLE` symbol is referenced by both the gate call sites and (forward-declared for) the menu module; every gated route is a row; the forge match is reconciled into explicit rows with the catch-all arm enumerated; the branch-protection and review-requirement predicates remain separate; the deny/allow decision is unchanged (all existing gate tests + invariant_build_gates green); clippy clean.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST introduce a `Route` enum + a single static `ROUTE_AUTHORITY_TABLE: &[(Route, Authority, &'static str /*command*/, &'static str /*effect*/)]` IN `but-authz` (L4 / 03 §4) so the gates (but-api) AND the menu (STEER-003) reference the SAME symbol with NO `but-authz → but-api` cycle.
- [MUST] MUST cover EVERY gated route as a row: commit (ContentsWrite), merge (Merge), forge reviews:write / comments:write / pull_requests:write, admin (AdministrationWrite) — reconciling the forge `authorize_branch_action` `match` (forge.rs:59-66) INCLUDING the `other => authorize(p, other)` catch-all arm into explicit rows.
- [MUST] MUST keep the NON-authority predicates OUT of the table but composed AROUND it: branch-protection (commit/gate.rs:69-74) and the review-requirement engine (merge_gate.rs review_requirement::evaluate) remain separate predicates applied after the table-driven authorize lookup (03 §4).
- [MUST] MUST preserve the shipped `invariant_build_gates.rs` honesty grep: `AUTHORITY_POSITIVE_PATTERN` (`but_authz::authorize|Authority::contains|but_authz::Authority`, :12-13) asserts a literal match in COMMIT_GATE, MERGE_GATE, CONFIG_MUTATE, GOVERNANCE (:60-83). Either KEEP the literal `but_authz::authorize`/`Authority::*` calls at each enforcement site (table feeds the menu + a coverage assertion) OR update the grep — choose the keep-literals path unless impossible, and STATE the decision in the completion report (D7).
- [MUST] MUST be behavior-NEUTRAL for the deny/allow DECISION: every existing commit_gate / merge_gate / forge_guard / governed_loop / admin_write_guard test stays green unchanged (no new RED in the existing suite).
- [NEVER] NEVER add a `but-authz → but-api` dependency — the table lives in but-authz; the gates in but-api consume it (RULES.md lower-level crates must not depend on but-api).
- [NEVER] NEVER move the branch-protection or review-requirement predicate INTO the table — they are authority∧¬predicate composites; folding them in would make `required_authority ⊆ held` a lying menu (the C5 unsoundness STEER-003 must avoid).
- [NEVER] NEVER weaken, narrow, or remove an existing grep pattern/path in invariant_build_gates.rs (ROLE_BRANCH_PATTERN, HUMAN_OR_LABEL_BRANCH_PATTERN, AUTHORITY_POSITIVE_PATTERN, PERMISSION_CARRIER_PATTERN) — only the literal-vs-grep reconciliation is in scope, additively if at all.
- [NEVER] NEVER change any denial code, message, exit code, or DryRun semantics.
- [NEVER] NEVER edit any frozen Sprint 01a–06b task file.
- [STRICTLY] STRICTLY treat the existing `authorize`/`effective_authority`/`branch_protected` call shapes at the gate sites as consumed Sprint 01a–05 seams — the table SUPPLIES the (route→required Authority, command, effect) data; it does NOT replace the `authorize()` call that makes the decision (so the positive-Authority grep keeps matching).
- [STRICTLY] STRICTLY note (RR-6) that forge.rs is outside the AUTHORITY_POSITIVE_PATTERN ENFORCEMENT_PATHS set (COMMIT_GATE/MERGE_GATE/CONFIG_MUTATE/GOVERNANCE only) — so the forge reconcile is guarded by the behavior-neutral forge_guard test (AC-2), not the honesty grep; adding forge.rs to the grep is deferred to STEER-010 as an additive decision.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: One ROUTE_AUTHORITY_TABLE symbol referenced by gate and menu module; every gated route present
- [ ] AC-2: Forge authorize_branch_action match reconciled incl. the `other =>` catch-all into explicit rows
- [ ] AC-3: Deny/allow decision unchanged across all gates (no new RED in the existing suite)
- [ ] AC-4: AUTHORITY_POSITIVE_PATTERN honesty grep stays green at every enforcement site
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: One ROUTE_AUTHORITY_TABLE symbol referenced by gate and menu module; every gated route present [PRIMARY]
  GIVEN: the refactored but-authz with Route + ROUTE_AUTHORITY_TABLE
  WHEN:  a test enumerates the table and a coverage assertion checks each enforcement route maps to a row
  THEN:  the table contains a row for commit(ContentsWrite), merge(Merge), reviews:write, comments:write, pull_requests:write, and admin(AdministrationWrite); the gate sites look up required-authority via the table; the symbol is `pub` and importable by the menu module
  TEST_TIER: unit   VERIFICATION_SERVICE: but-authz
  UNIT_TEST_JUSTIFIED: Pure table-coverage logic over a static `&[..]` slice — enumerating ROUTE_AUTHORITY_TABLE rows and asserting each expected (Route, Authority) pair is present has zero I/O; the runtime same-cfg/ref property is proven separately by STEER-009 integration tests.
  VERIFY: cargo test -p but-authz steer_route_table_covers_every_gated_route
  SCENARIO: would fail if ROUTE_AUTHORITY_TABLE is empty / a stub `&[]`; a gated route (e.g. Merge) is missing from the table; the table is duplicated per-crate rather than a single but-authz symbol | must observe: table length >= 6; `Authority::Merge` present in a row; `Authority::ContentsWrite` present in a row; `Authority::AdministrationWrite` present in a row | must NOT observe: an empty table (length 0); a missing Merge row

AC-2: Forge authorize_branch_action match reconciled incl. the `other =>` catch-all into explicit rows
  GIVEN: the forge boundary that previously matched Authority arms with an `other => authorize(p, other)` catch-all (forge.rs:59-66)
  WHEN:  the forge authorize path is driven for reviews:write, comments:write, and pull_requests:write actions against the real fixture
  THEN:  each forge action resolves its required Authority through the table (explicit rows, no opaque catch-all hiding a route), and the existing forge_guard authorize decision is unchanged (allowed/denied identically to pre-refactor)
  TEST_TIER: integration   VERIFICATION_SERVICE: but-api
  VERIFY: cargo test -p but-api forge_guard && cargo test -p but-authz steer_route_table_includes_forge_routes
  SCENARIO: would fail if the forge `other =>` arm is left as an un-enumerated catch-all so a route is invisible to the table/coverage grep; reconciling the match flips a deny to an allow (decision regression); the forge path stops calling but_authz::authorize (the call is removed) so the positive grep no longer matches | must observe: `rev` reviews:write authorizes (no governance denial); `ro` reviews:write is denied `perm.denied` | must NOT observe: `ro` reviews:write authorized (decision regression — no denial emitted); a panic from an unhandled Authority variant

AC-3: Deny/allow decision unchanged across all gates (no new RED in the existing suite)
  GIVEN: the table-driven refactor applied at the commit, merge, forge, and admin sites
  WHEN:  the full existing governance gate suite runs
  THEN:  every existing commit_gate / merge_gate / merge_gate_self_escalation / forge_guard / admin_write_guard / governed_loop test passes unchanged — the refactor is behavior-neutral for the decision
  TEST_TIER: integration   VERIFICATION_SERVICE: but-api
  VERIFY: cargo test -p but-api commit_gate && cargo test -p but-api merge_gate && cargo test -p but-api forge_guard && cargo test -p but-api admin_write_guard && cargo test -p but governed_loop
  SCENARIO: would fail if the refactor changes a commit-gate decision (protected main commit now allowed); the merge review-requirement predicate is dropped (removed/no-op) when folded toward the table; an existing gate test goes RED | must observe: the `feat` ref advances for `dev` (allowed commit; `feat` ref_id changes); protected main commit denied `branch.protected`; ro feat commit denied `perm.denied` | must NOT observe: protected `main` commit allowed (no denial emitted); feat ref unchanged for the allowed dev commit

AC-4: AUTHORITY_POSITIVE_PATTERN honesty grep stays green at every enforcement site
  GIVEN: the existing invariant_build_gates.rs grep asserting literal `but_authz::authorize`/`Authority::*` in COMMIT_GATE/MERGE_GATE/CONFIG_MUTATE/GOVERNANCE
  WHEN:  the build-gate test runs after the table refactor
  THEN:  the positive-Authority grep still matches each enforcement path (the literal authorize calls remain) and no role-preset / human-vs-AI / Permission-carrier match appears; invariant_build_gates passes
  TEST_TIER: integration   VERIFICATION_SERVICE: but-authz
  VERIFY: cargo test -p but-authz --test invariant_build_gates
  SCENARIO: would fail if a table-driven helper HIDES the literal `but_authz::authorize` call so the positive grep finds no match (grep RED); the refactor introduces a `match role { "admin" => }` style branch (ROLE_BRANCH_PATTERN matches → fail); an enforcement path file becomes empty | must observe: the test prints/exits success (exit `0`); the `AUTHORITY_POSITIVE_PATTERN` grep matches `1`+ time in each of `COMMIT_GATE`, `MERGE_GATE`, `CONFIG_MUTATE`, `GOVERNANCE` | must NOT observe: a `grep did not find the required structural match` failure (no matches); a forbidden role-preset/human-vs-AI match

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, happy_path): ROUTE_AUTHORITY_TABLE has a row mapping to Authority::Merge
    VERIFY: cargo test -p but-authz steer_route_table_covers_every_gated_route
- TC-2 (-> AC-1, happy_path): ROUTE_AUTHORITY_TABLE has a row mapping to Authority::ContentsWrite
    VERIFY: cargo test -p but-authz steer_route_table_covers_every_gated_route
- TC-3 (-> AC-1, happy_path): ROUTE_AUTHORITY_TABLE has a row mapping to Authority::AdministrationWrite
    VERIFY: cargo test -p but-authz steer_route_table_covers_every_gated_route
- TC-4 (-> AC-2, edge_case): ROUTE_AUTHORITY_TABLE includes reviews:write, comments:write, and pull_requests:write forge routes
    VERIFY: cargo test -p but-authz steer_route_table_includes_forge_routes
- TC-5 (-> AC-2, error_case): `ro` (contents:read only) forge reviews:write action is denied perm.denied after the reconcile
    VERIFY: cargo test -p but-api forge_guard
- TC-6 (-> AC-3, error_case): dev direct commit to protected main is still denied branch.protected after the refactor
    VERIFY: cargo test -p but-api commit_gate
- TC-7 (-> AC-3, happy_path): All existing merge_gate tests pass unchanged
    VERIFY: cargo test -p but-api merge_gate
- TC-8 (-> AC-4, happy_path): invariant_build_gates positive-Authority grep matches every enforcement path
    VERIFY: cargo test -p but-authz --test invariant_build_gates
- TC-9 (-> AC-4, error_case): invariant_build_gates finds no role-preset or human-vs-AI branch in the refactored sites
    VERIFY: cargo test -p but-authz --test invariant_build_gates

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-STEER-01
provides: but_authz::Route enum; but_authz::ROUTE_AUTHORITY_TABLE: &[(Route, Authority, &'static str command, &'static str effect)]; a single-source table referenced by both the gates and the menu derivation (STEER-003)
consumes: but_authz::Authority (authority.rs:11); but_authz::authorize (authorize.rs:24); commit gate authorize+predicate (commit/gate.rs:67-74); merge gate authorize+review-engine (merge_gate.rs:48); forge authorize_branch_action match (forge.rs:47-68); admin authorize (config_mutate.rs:25); invariant_build_gates.rs AUTHORITY_POSITIVE_PATTERN (:12-13) + ENFORCEMENT_PATHS (:24-32)
boundary_contracts:
  - CAP-STEER-01: ONE `ROUTE_AUTHORITY_TABLE` symbol in but-authz is referenced by every gate's required-authority lookup AND by the menu module (STEER-003); every gated route is present as a row; the non-authority predicates (branch-protection, review-requirement) stay OUT of the table but compose around it; the deny/allow DECISION is byte-for-byte unchanged (all existing gate tests stay green).

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-authz/src/route.rs (NEW — Route enum + ROUTE_AUTHORITY_TABLE)
  - crates/but-authz/src/lib.rs (MODIFY — mod route; pub use Route, ROUTE_AUTHORITY_TABLE)
  - crates/but-api/src/commit/gate.rs (MODIFY — look up required Authority via the table; keep literal authorize call + branch-protection predicate)
  - crates/but-api/src/legacy/merge_gate.rs (MODIFY — Merge route via table; keep authorize + review engine)
  - crates/but-api/src/legacy/forge.rs (MODIFY — reconcile authorize_branch_action match incl. `other =>` into table-backed rows)
  - crates/but-api/src/legacy/config_mutate.rs (MODIFY — admin route via table)
  - crates/but-authz/tests/steer_route_table.rs (NEW — table coverage proofs)
writeProhibited:
  - the gate deny/allow decision — NEVER weaken (behavior-neutral only)
  - the branch-protection / review-requirement predicates — NEVER fold into the table
  - any existing pattern or path in invariant_build_gates.rs — NEVER weaken/remove (additive reconciliation only)
  - any but-api → but-authz dependency reversal — NEVER add but-api to but-authz
  - .spec/prds/governance/tasks/sprint-0[1-6]* — frozen task files

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
  - crates/but-authz/src/authority.rs (lines 1-110): Authority enum :11 + name() :94 — the table's Authority column draws from here.
  - crates/but-authz/src/authorize.rs (lines 24-31): authorize(principal, action, cfg) :24 — the decision call the table feeds (route→required Authority); keep this literal call at the gate sites.
  - crates/but-api/src/commit/gate.rs (lines 55-78): authorize(p, ContentsWrite, &cfg) :67 then the branch-protection predicate :69-74 — the table supplies ContentsWrite for the commit route; the predicate stays composed around it.
  - crates/but-api/src/legacy/merge_gate.rs (lines 39-110): authorize(p, Merge) :48 then the review-requirement engine :58-109 — Merge is a table row; the review predicate stays separate.
  - crates/but-api/src/legacy/forge.rs (lines 47-68): authorize_branch_action match :47-68 incl. `other => authorize(p, other)` :65 — reconcile into explicit reviews:write/comments:write/pull_requests:write rows.
  - crates/but-api/src/legacy/config_mutate.rs (lines 13-28): enforce_administration_write_gate :18 → authorize(p, AdministrationWrite) :25 — the admin route row.
  - crates/but-authz/tests/invariant_build_gates.rs (lines 10-100): AUTHORITY_POSITIVE_PATTERN :12-13, ENFORCEMENT_PATHS :24-32, the assert_grep_has_matches calls :60-83 — keep these matching (keep literal authorize calls).

--------------------------------------------------------------------------------
CODE PATTERN
--------------------------------------------------------------------------------
pattern: A `Route` enum + a `pub const ROUTE_AUTHORITY_TABLE: &[(Route, Authority, &'static str, &'static str)]` in but-authz; each gate looks up its row to get the required Authority (and command/effect for the menu) but STILL calls `but_authz::authorize(p, Authority::X, &cfg)` literally so the honesty grep matches; non-authority predicates remain as separate `if`/`match` composed after the authorize call.
pattern_source: crates/but-authz/src/authority.rs:46 (Authority::ALL is the existing static-slice-of-enum pattern to mirror for ROUTE_AUTHORITY_TABLE); crates/but-api/src/legacy/forge.rs:59-66 (the match to reconcile).
anti_pattern: A table-driven `authorize_via_table(route)` helper that REPLACES the literal `but_authz::authorize` call at each site — this breaks the AUTHORITY_POSITIVE_PATTERN grep (its whole point is to assert the literal axis appears); folding branch-protection into the table (re-creates the C5 lying-menu unsoundness).
references: 03-technical-requirements-delta.md §4; 05-delta-replan.md D7; SPRINT.md Coverage Notes 'ROUTE_AUTHORITY_TABLE is a real refactor'
interaction_notes:
  - STEER-003 imports ROUTE_AUTHORITY_TABLE for the `usable = {r | r.required_authority ⊆ held}` step; STEER-010 adds a coverage grep asserting every gated route ∈ the table — design the table so that grep can target it.
  - RR-6 forge-grep coverage note: the `forge.rs` `authorize_branch_action` site this task refactors is NOT in the `AUTHORITY_POSITIVE_PATTERN` ENFORCEMENT_PATHS set — that grep covers only COMMIT_GATE/MERGE_GATE/CONFIG_MUTATE/GOVERNANCE (invariant_build_gates.rs:24-32). So the forge reconcile's safety rests on the behavior-neutral `forge_guard` test (AC-2), NOT the honesty grep. Whether to ADD forge.rs to the grep path set is a STEER-010 additive decision, NOT a STEER-002 change (this task never weakens or alters the grep).

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
depends_on: STEER-001
blocks: STEER-003, STEER-010

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
      "description": "GIVEN the refactor WHEN the table is enumerated THEN one ROUTE_AUTHORITY_TABLE symbol covers every gated route and is referenced by gate + menu module",
      "verify": "cargo test -p but-authz steer_route_table_covers_every_gated_route"
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "GIVEN the forge match incl. `other =>` WHEN reconciled THEN forge routes are explicit table rows and the decision is unchanged",
      "verify": "cargo test -p but-api forge_guard && cargo test -p but-authz steer_route_table_includes_forge_routes"
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "GIVEN the table-driven refactor WHEN the existing gate suite runs THEN every deny/allow decision is unchanged",
      "verify": "cargo test -p but-api commit_gate && cargo test -p but-api merge_gate && cargo test -p but-api forge_guard && cargo test -p but-api admin_write_guard && cargo test -p but governed_loop"
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "description": "GIVEN the AUTHORITY_POSITIVE_PATTERN grep WHEN the build-gate runs THEN it stays green (literal authorize calls preserved, no role/human branch)",
      "verify": "cargo test -p but-authz --test invariant_build_gates"
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "Table maps a row to Merge",
      "maps_to_ac": "AC-1",
      "verify": "cargo test -p but-authz steer_route_table_covers_every_gated_route"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "Table maps a row to ContentsWrite",
      "maps_to_ac": "AC-1",
      "verify": "cargo test -p but-authz steer_route_table_covers_every_gated_route"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "Table maps a row to AdministrationWrite",
      "maps_to_ac": "AC-1",
      "verify": "cargo test -p but-authz steer_route_table_covers_every_gated_route"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "Table includes the three forge routes",
      "maps_to_ac": "AC-2",
      "verify": "cargo test -p but-authz steer_route_table_includes_forge_routes"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "ro forge reviews:write denied after reconcile",
      "maps_to_ac": "AC-2",
      "verify": "cargo test -p but-api forge_guard"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "description": "dev protected-main commit still branch.protected",
      "maps_to_ac": "AC-3",
      "verify": "cargo test -p but-api commit_gate"
    },
    {
      "id": "TC-7",
      "type": "test_criterion",
      "description": "merge_gate suite passes unchanged",
      "maps_to_ac": "AC-3",
      "verify": "cargo test -p but-api merge_gate"
    },
    {
      "id": "TC-8",
      "type": "test_criterion",
      "description": "positive-Authority grep matches all enforcement paths",
      "maps_to_ac": "AC-4",
      "verify": "cargo test -p but-authz --test invariant_build_gates"
    },
    {
      "id": "TC-9",
      "type": "test_criterion",
      "description": "no role/human branch in refactored sites",
      "maps_to_ac": "AC-4",
      "verify": "cargo test -p but-authz --test invariant_build_gates"
    }
  ]
}
-->
