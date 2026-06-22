# LPR-002: `AssignmentState { Pending, Approved, ChangesRequested }` typed enum + boundary (de)serialization

> Status: ✅ Completed
> Commit: 568915b041
> Reviewer: deferred to PHASE 4.5 red-hat closeout — committed prior session; AssignmentState typed enum
> Updated: 2026-06-22T18:07:12Z


## What this does

Add a pure `AssignmentState { Pending, Approved, ChangesRequested }` enum with a `parse`/`name` round-trip at the `but-authz`/`but-api` boundary, so the `local_review_assignments.state` TEXT column is validated and typed on read/write — exactly the shape discipline `Authority`'s `parse`/`name` round-trip uses (`crates/but-authz/src/authority.rs:69`/`:94`). The DB column stays `TEXT` (migration-tolerant, matching `LocalReviewVerdict.verdict: String`). **No new `Authority` variant; no change to `authorize`/`effective_authority`.**

## Why

Sprint 07 · PRD UC-LPR-01 · capability CAP-AUTHZ-01. The assignment state is a drive signal an orchestrator reads (`pending` → dispatch reviewer; `changes_requested` → remediation; `approved` → eligible). The TEXT column is migration-tolerant, but every reader/writer must agree on the three literals and reject garbage — so the boundary needs a typed enum, mapped to/from the column with a total `parse` (unknown string → error) and an injective `name` (enum → the literal). This is the `Authority` precedent: the DB/wire stays a string, the boundary is typed.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-authz assignment_state_parse_name_round_trips`: every `AssignmentState` variant round-trips `parse(name(v)) == v`, the three literals are exactly `pending`/`approved`/`changes_requested`, and an unknown string (`"merged"`, `""`) returns an error (not a default). Full gate set in the spec below.

## Scope

- crates/but-authz/src/assignment_state.rs (NEW — `pub enum AssignmentState { Pending, Approved, ChangesRequested }` + `pub fn name(&self) -> &'static str` + `pub fn parse(s: &str) -> Result<Self, …>` + the unit tests, mirroring the `Authority` parse/name shape)
- crates/but-authz/src/lib.rs (MODIFY — `pub mod assignment_state;` + re-export `pub use assignment_state::AssignmentState;` from the existing pub use block)
- crates/but-authz/tests/assignment_state.rs (NEW — OR in-module `#[cfg(test)] mod tests` — the round-trip + reject-unknown proofs; follow whichever location the existing Authority parse/name tests use)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: LPR-002 — `AssignmentState` typed enum + boundary (de)serialization
================================================================================

TASK_TYPE:   FEATURE
STATUS:      Backlog
PRIORITY:    P1
EFFORT:      S  (75 min)
AGENT:       implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-LPR-01
CAPABILITIES:CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-authz assignment_state_parse_name_round_trips
  check: cargo check -p but-authz --all-targets
  lint:  cargo clippy -p but-authz --all-targets

--------------------------------------------------------------------------------
TYPE DESIGN (Rust shapes this task introduces)
--------------------------------------------------------------------------------
NEWTYPES / ENUMS:
  - `pub enum AssignmentState { Pending, Approved, ChangesRequested }` with `#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]` (mirror Authority's derives, authority.rs:10)
  - `pub fn name(&self) -> &'static str` -> "pending" | "approved" | "changes_requested" (the EXACT three literals the local_review_assignments.state column stores, tech-delta §A.1)
  - `pub fn parse(s: &str) -> Result<Self, AssignmentStateParseError>` — total: every unknown string is an Err, never a silent default
  - `pub struct AssignmentStateParseError(String)` OR reuse the crate's existing parse-error type if Authority::parse already has one (read authority.rs:69 to match the convention)
ERROR STRATEGY:
  - parse returns a typed error (a `thiserror`-style or the crate's existing parse-error newtype, matching how Authority::parse reports an unknown authority name at authority.rs:69-80). NEVER `unwrap`/`expect` in the library; NEVER map an unknown string to a default variant.
OWNERSHIP PLAN:
  - `AssignmentState` is `Copy` (a 3-variant fieldless enum) — passed by value everywhere; `name` returns a `&'static str` (no allocation); `parse` borrows `&str` and returns an owned `AssignmentState`.
DOC POINTERS (read before coding):
  - brain/docs/rust/traits-generics.md → enum variants + pattern matching (no runtime type info; match exhaustively)
  - brain/docs/rust/error-handling.md → typed parse error (`#[derive(Error)]` enum / newtype), `?` propagation, NO panic for flow
  - brain/docs/rust/testing.md → `#[test]` round-trip + reject-unknown

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
A pure enum at the but-authz boundary: (1) `parse(name(v)) == v` for all three variants (injective name, total parse) — the round-trip; (2) the three literals are EXACTLY `pending`/`approved`/`changes_requested` (the column values LPR-001's table stores and the orchestrator reads); (3) `parse("merged")`, `parse("")`, `parse("Approved")` (wrong case) all return Err — an unknown/garbage state is rejected, never coerced to a default; (4) NO new `Authority` variant is added and `authorize`/`effective_authority` are unchanged (the invariant_build_gates honesty grep stays green); cargo test -p but-authz green; clippy clean.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST model the enum/parse/name on the SHIPPED `Authority` pattern (authority.rs:69 `parse`/`from_str`-style, :94 `name`): a fieldless enum, a `name(&self) -> &'static str` returning the literal, and a `parse(&str) -> Result<Self, _>` that is TOTAL (unknown -> Err). Read authority.rs:60-105 and follow its exact shape (derives, error reporting, the literal strings).
- [MUST] MUST use the EXACT three literals `pending` / `approved` / `changes_requested` — these are the values the `local_review_assignments.state` column stores (tech-delta §A.1) and the values `approve_review`/`request_changes_review` write. A typo in any literal silently breaks the orchestrator's state read.
- [MUST] MUST keep `parse` total and fail-closed: an unknown string returns Err (mirroring how an unknown authority name is an Err in Authority::parse). NEVER `unwrap`/`expect`; NEVER `_ => Self::Pending` (a default-on-unknown is a silent corruption — AC-3's reject-unknown catches it).
- [MUST] MUST keep the DB column `TEXT` (this task does NOT change LPR-001's schema). The enum lives at the but-authz boundary; the table stores the string. This matches `LocalReviewVerdict.verdict: String` (a free TEXT validated by the writer; merge_gate filters on the literal "approved", review_requirement.rs:8).
- [NEVER] NEVER add a new `Authority` variant or touch `authorize`/`effective_authority`/`AuthoritySet`. AssignmentState is a DRIVE-state enum, NOT an authority. The route->Authority table stays closed (authority.rs:11 unchanged); the invariant_build_gates honesty grep (no role-name / no human-vs-AI branch in the enforcement paths) must stay green.
- [NEVER] NEVER branch on AssignmentState in any merge-gate / commit-gate / authorize path — the gate reads `local_review_verdicts` ("approved" verdict-at-head), NOT assignment state (the safe seam, LPR-009). AssignmentState is consumed only by the drive surface (review_status, request/assign verbs).
- [NEVER] NEVER make the state column an enum in `but-db` (LPR-001's column is TEXT for migration-tolerance) — the typing is a boundary concern only.
- [NEVER] NEVER add new gitbutler-* usage.
- [STRICTLY] STRICTLY keep the enum PURE (no I/O, no DB, no ctx) — it is a parse/format type. UNIT_TEST_JUSTIFIED: this task is pure logic with zero I/O (a parse/name round-trip over an enum) — unit tests are the correct tier here, NOT integration, because there is no service to verify; the enum's only contract is the total injective parse/name round-trip.
- [STRICTLY] STRICTLY make the enum `Copy` + the full derive set (Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash) so it slots into match arms and sets exactly like Authority.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [x] AC-1 [PRIMARY]: `parse(name(v)) == v` for all three variants (injective name, total parse — the round-trip)
- [x] AC-2: `name` returns exactly `pending` / `approved` / `changes_requested` (the literals the table stores and the orchestrator reads)
- [x] AC-3: `parse` rejects an unknown/garbage/wrong-case string with Err (never a default variant)
- [x] AC-4: NO new `Authority` variant is added; `authorize`/`effective_authority` unchanged; the invariant_build_gates honesty grep stays green
- [x] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: parse/name round-trip for every variant
  GIVEN: the AssignmentState enum with three variants
  WHEN:  for each v in {Pending, Approved, ChangesRequested}, `AssignmentState::parse(v.name())` runs
  THEN:  `parse(name(v)) == Ok(v)` for all three (name is injective, parse is its total inverse on the valid set)
  TEST_TIER: unit   VERIFICATION_SERVICE: the pure AssignmentState parse/name functions (no service — UNIT_TEST_JUSTIFIED, pure logic)
  VERIFY: cargo test -p but-authz assignment_state_parse_name_round_trips

AC-2: the literals are exactly the three column values
  GIVEN: the AssignmentState enum
  WHEN:  name() is read for each variant
  THEN:  Pending.name()=="pending", Approved.name()=="approved", ChangesRequested.name()=="changes_requested" — the EXACT literals the local_review_assignments.state column stores (tech-delta §A.1) and approve_review/request_changes_review write
  TEST_TIER: unit   VERIFICATION_SERVICE: the pure name() function
  VERIFY: cargo test -p but-authz assignment_state_literals_match_column_values

AC-3: parse rejects unknown / garbage / wrong-case (fail-closed, no default)
  GIVEN: the AssignmentState parse function
  WHEN:  parse("merged"), parse(""), parse("Approved"), parse("PENDING") run
  THEN:  every call returns Err (an unknown/garbage/wrong-case state is rejected) — NONE coerces to a default variant; the round-trip set is closed to the three exact literals
  TEST_TIER: unit   VERIFICATION_SERVICE: the pure parse() function (fail-closed)
  VERIFY: cargo test -p but-authz assignment_state_parse_rejects_unknown

AC-4: no new Authority variant; authorize unchanged; honesty grep green
  GIVEN: the AssignmentState enum added to but-authz
  WHEN:  the Authority enum and authorize/effective_authority are inspected and the invariant_build_gates test runs
  THEN:  Authority has the SAME variant set as before (no AssignmentState leakage into Authority); authorize/effective_authority are byte-unchanged; `cargo test -p but-authz invariant_build_gates` stays green (AssignmentState introduces no role-name / human-vs-AI branch into any enforcement path)
  TEST_TIER: build-gate   VERIFICATION_SERVICE: the shipped but-authz invariant_build_gates honesty test + an Authority-variant-count assertion
  VERIFY: cargo test -p but-authz invariant_build_gates && cargo test -p but-authz assignment_state_not_an_authority

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): for each variant v, parse(name(v)) == Ok(v)
    VERIFY: cargo test -p but-authz assignment_state_parse_name_round_trips
- TC-2 (-> AC-2): name() yields exactly pending/approved/changes_requested
    VERIFY: cargo test -p but-authz assignment_state_literals_match_column_values
- TC-3 (-> AC-3): parse("merged")/parse("")/parse("Approved")/parse("PENDING") all return Err (no default)
    VERIFY: cargo test -p but-authz assignment_state_parse_rejects_unknown
- TC-4 (-> AC-4): Authority's variant set is unchanged and invariant_build_gates is green (AssignmentState is not an Authority)
    VERIFY: cargo test -p but-authz invariant_build_gates && cargo test -p but-authz assignment_state_not_an_authority

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides:
  - `but_authz::AssignmentState { Pending, Approved, ChangesRequested }` + `name(&self) -> &'static str` + `parse(&str) -> Result<Self, _>` (the typed boundary for the local_review_assignments.state TEXT column)
consumes:
  - the shipped Authority::parse/name shape (authority.rs:69/:94) as the structural template (mirror it; do not modify Authority)
boundary_contracts:
  - CAP-AUTHZ-01: AssignmentState is a DRIVE-state enum at the but-authz boundary, NOT an Authority. It introduces no new Authority variant and no branch into any enforcement path. The DB column stays TEXT; the enum validates the literal on read/write. The gate never reads assignment state (safe seam, LPR-009).

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-authz/src/assignment_state.rs (NEW — the enum + parse/name + #[cfg(test)] mod tests)
  - crates/but-authz/src/lib.rs (MODIFY — `pub mod assignment_state;` + `pub use assignment_state::AssignmentState;`)
  - crates/but-authz/tests/assignment_state.rs (NEW — IF the crate convention puts parse/name tests in a tests/ file rather than in-module; otherwise the in-module #[cfg(test)] tests suffice)
writeProhibited:
  - crates/but-authz/src/authority.rs — CONSUME-only (the parse/name structural template); do NOT add a variant or change it
  - crates/but-authz/src/{authorize.rs, denial.rs, principal.rs, config.rs} — the enforcement/union layer is closed
  - crates/but-authz/tests/invariant_build_gates.rs — do NOT weaken any honesty-grep pattern (AC-4 asserts it stays green unchanged)
  - crates/but-db/** — the state column stays TEXT (LPR-001's schema); the enum is a boundary type, not a column type
  - any gitbutler-* crate (no new gitbutler-* usage)
  - Any file not in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-authz/src/authority.rs [10, 60-105] — [PRIMARY PATTERN] the Authority enum's derives (:10), its `parse`/`from_str` (:69-80, total, unknown->Err) and its `name(&self) -> &'static str` (:94-105, the literal strings). Mirror this EXACT shape for AssignmentState (derives, error reporting, the &'static str literals). Do NOT add an Authority variant.
2. .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md (§A.1) — the state column is TEXT storing 'pending'|'approved'|'changes_requested'; "typed at the boundary" to an `enum AssignmentState { Pending, Approved, ChangesRequested }` via the Authority parse/name round-trip; DB column stays TEXT for migration-tolerance.
3. crates/but-db/src/table/local_review_verdicts.rs [11-16, 24-31] — the sibling precedent: `verdict: String` (a free TEXT) that merge_gate filters on the literal "approved" — proof the state column should stay TEXT and the enum is a boundary concern.
4. crates/but-authz/tests/invariant_build_gates.rs [9-34] — the honesty-grep patterns + ENFORCEMENT_PATHS; AC-4 asserts adding AssignmentState does NOT trip them (it is not a role name and never enters an enforcement path). Read to confirm AssignmentState must not be referenced from any ENFORCEMENT_PATHS file.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- cargo test -p but-authz assignment_state_parse_name_round_trips   -> Exit 0; parse(name(v))==Ok(v) for all 3
- cargo test -p but-authz assignment_state_literals_match_column_values   -> Exit 0; literals are pending/approved/changes_requested
- cargo test -p but-authz assignment_state_parse_rejects_unknown   -> Exit 0; unknown/garbage/wrong-case -> Err (no default)
- cargo test -p but-authz assignment_state_not_an_authority   -> Exit 0; Authority variant set unchanged
- cargo test -p but-authz invariant_build_gates   -> Exit 0; honesty grep green (AssignmentState introduces no enforcement-path branch)
- cargo check -p but-authz --all-targets   -> Exit 0
- cargo clippy -p but-authz --all-targets   -> Exit 0
- cargo fmt --check   -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references:
  - crates/but-authz/src/authority.rs:69 (parse), :94 (name) — the round-trip template
  - .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md §A.1 (state TEXT + typed enum at the boundary)
code_skeleton: |
  // crates/but-authz/src/assignment_state.rs
  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
  pub enum AssignmentState { Pending, Approved, ChangesRequested }
  impl AssignmentState {
      pub fn name(&self) -> &'static str {
          match self {
              Self::Pending => "pending",
              Self::Approved => "approved",
              Self::ChangesRequested => "changes_requested",
          }
      }
      pub fn parse(s: &str) -> Result<Self, AssignmentStateParseError> {
          match s {
              "pending" => Ok(Self::Pending),
              "approved" => Ok(Self::Approved),
              "changes_requested" => Ok(Self::ChangesRequested),
              other => Err(AssignmentStateParseError(other.to_owned())),
          }
      }
  }
notes:
  - UNIT_TEST_JUSTIFIED: this task involves pure logic with zero I/O — unit tests justified because the only contract is a total injective parse/name round-trip over a fieldless enum; there is no service, DB, or git to verify, so an integration test would have nothing real to exercise. (This is the ONE task in the sprint where unit-only is correct; LPR-001/003..010 are all integration.)
  - Match Authority's error-reporting convention exactly (read authority.rs:69 — if it returns a crate error type, use the same; if a String-newtype, mirror that). Do not invent a divergent error shape.
pattern: a pure boundary enum with a total injective parse/name round-trip, mirroring Authority::parse/name — DB column stays TEXT, the enum validates the literal
pattern_source: crates/but-authz/src/authority.rs:69/:94 (the parse/name round-trip); crates/but-db/src/table/local_review_verdicts.rs:11 (verdict: String, the TEXT-column precedent)
anti_pattern: adding an Authority variant (the catalog is closed); a default-on-unknown parse (`_ => Self::Pending` — silent corruption, AC-3 catches it); making the DB column an enum (LPR-001's column is TEXT); referencing AssignmentState from any ENFORCEMENT_PATHS file (would trip / pollute the honesty grep); panic/unwrap in the library

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agent: implementer=rust-implementer | reviewer=rust-reviewer
rationale: A pure boundary enum + a total parse/name round-trip mirroring Authority. The only subtlety is keeping it fail-closed (unknown -> Err, never a default), keeping the DB column TEXT, and proving it is NOT an Authority (the honesty grep stays green). rust-implementer writes the enum + the unit round-trip; rust-reviewer validates the literals match the column values, parse rejects unknowns, and no Authority variant / enforcement-path branch was introduced.
coding_standards: crates/AGENTS.md (keep the type in the crate that owns the concept — but-authz owns governance types; no speculative abstractions); brain/docs/rust/error-handling.md (typed parse error, no panic for flow); brain/docs/rust/traits-generics.md (exhaustive match, no runtime type info)

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: LPR-001 (the local_review_assignments.state TEXT column this enum types at the boundary)
Blocks:     LPR-003 (request/assign + changes_requested write maps AssignmentState <-> the column), LPR-005 (review_status reads assignment state typed), LPR-008 (the reconciler read-API surfaces the typed state)
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "LPR-002",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": false
  },
  "fixtures": {
    "assignment_state_enum": {
      "description": "No service fixture — this is a pure enum. The 'fixture' is the AssignmentState type itself: the three variants {Pending, Approved, ChangesRequested} and their name()/parse() functions. Tests are pure (UNIT_TEST_JUSTIFIED): no DbHandle, no ctx, no git.",
      "seed_method": "in_code",
      "records": [
        "AssignmentState::Pending / Approved / ChangesRequested (the three variants under test)"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN the AssignmentState enum WHEN for each v in {Pending, Approved, ChangesRequested} parse(name(v)) runs THEN parse(name(v)) == Ok(v) for all three (name injective, parse its total inverse)",
      "verify": "cargo test -p but-authz assignment_state_parse_name_round_trips",
      "scenario": {
        "tier": "visible",
        "test_tier": "unit",
        "verification_service": "the pure AssignmentState parse/name functions (UNIT_TEST_JUSTIFIED — pure logic, zero I/O)",
        "negative_control": {
          "would_fail_if": [
            "name() and parse() used different literals (parse(name(v)) != v) — the round-trip breaks",
            "two variants shared a literal (name not injective) — a round-trip collides",
            "parse defaulted on unknown — name(v) for one variant could map back to a different variant"
          ]
        },
        "evidence": { "artifact_type": "test_output", "required_capture": true },
        "cases": [
          {
            "start_ref": "assignment_state_enum",
            "action": { "actor": "ci", "steps": [ "for each variant v: assert AssignmentState::parse(v.name()) == Ok(v)" ] },
            "end_state": {
              "must_observe": [ "parse(name(Pending))==Ok(Pending)", "parse(name(Approved))==Ok(Approved)", "parse(name(ChangesRequested))==Ok(ChangesRequested)" ],
              "must_not_observe": [ "any parse(name(v)) != Ok(v)", "any two variants mapping to the same literal" ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the enum WHEN name() is read THEN Pending.name()==pending, Approved.name()==approved, ChangesRequested.name()==changes_requested (the EXACT column literals)",
      "verify": "cargo test -p but-authz assignment_state_literals_match_column_values",
      "scenario": {
        "tier": "visible",
        "test_tier": "unit",
        "verification_service": "the pure name() function",
        "negative_control": {
          "would_fail_if": [
            "a literal were misspelled or cased differently (e.g. 'changesRequested' camelCase) — it would not match the column value the table stores / approve_review writes",
            "approved were spelled differently from the verdict literal 'approved' merge_gate reads — drift between drive state and verdict"
          ]
        },
        "evidence": { "artifact_type": "test_output", "required_capture": true },
        "cases": [
          {
            "start_ref": "assignment_state_enum",
            "action": { "actor": "ci", "steps": [ "assert the three name() values equal pending / approved / changes_requested exactly" ] },
            "end_state": {
              "must_observe": [ "Pending.name()==\"pending\"", "Approved.name()==\"approved\"", "ChangesRequested.name()==\"changes_requested\"" ],
              "must_not_observe": [ "any camelCase / wrong-case / misspelled literal" ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the parse function WHEN parse(merged)/parse(empty)/parse(Approved-wrongcase)/parse(PENDING) run THEN every call returns Err (no default coercion) — the round-trip set is closed to the three exact literals",
      "verify": "cargo test -p but-authz assignment_state_parse_rejects_unknown",
      "scenario": {
        "tier": "visible",
        "test_tier": "unit",
        "verification_service": "the pure parse() function (fail-closed)",
        "negative_control": {
          "would_fail_if": [
            "parse had a `_ => Self::Pending` (or any) default arm — an unknown string would parse Ok instead of Err",
            "parse were case-insensitive — 'Approved'/'PENDING' would wrongly succeed"
          ]
        },
        "evidence": { "artifact_type": "test_output", "required_capture": true },
        "cases": [
          {
            "start_ref": "assignment_state_enum",
            "action": { "actor": "ci", "steps": [ "assert parse(\"merged\").is_err()", "assert parse(\"\").is_err()", "assert parse(\"Approved\").is_err()", "assert parse(\"PENDING\").is_err()" ] },
            "end_state": {
              "must_observe": [ "parse(\"merged\") is Err", "parse(\"\") is Err", "parse(\"Approved\") is Err", "parse(\"PENDING\") is Err" ],
              "must_not_observe": [ "any of those returning Ok(<a default variant>)" ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN AssignmentState added to but-authz WHEN the Authority enum + authorize/effective_authority are inspected and invariant_build_gates runs THEN Authority's variant set is unchanged, authorize/effective_authority are byte-unchanged, and the honesty grep stays green (AssignmentState is not an Authority and enters no enforcement path)",
      "verify": "cargo test -p but-authz invariant_build_gates && cargo test -p but-authz assignment_state_not_an_authority",
      "scenario": {
        "tier": "visible",
        "test_tier": "build-gate",
        "verification_service": "the shipped but-authz invariant_build_gates honesty test + an Authority-variant assertion",
        "negative_control": {
          "would_fail_if": [
            "an AssignmentState-shaped variant were added to Authority — Authority::ALL count changes / the variant assertion fails",
            "AssignmentState were referenced from an ENFORCEMENT_PATHS file — the gate path would now branch on drive state (a safe-seam violation precursor)"
          ]
        },
        "evidence": { "artifact_type": "test_output", "required_capture": true },
        "cases": [
          {
            "start_ref": "assignment_state_enum",
            "action": { "actor": "ci", "steps": [ "assert the Authority catalog (authority.rs:11) has the same 13 variants as before", "run invariant_build_gates" ] },
            "end_state": {
              "must_observe": [ "Authority's variant set unchanged", "invariant_build_gates passes" ],
              "must_not_observe": [ "a new Authority variant", "AssignmentState referenced from any ENFORCEMENT_PATHS file" ]
            }
          }
        ]
      }
    },
    { "id": "TC-1", "type": "test_criterion", "description": "for each variant v, parse(name(v)) == Ok(v)", "verify": "cargo test -p but-authz assignment_state_parse_name_round_trips", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "name() yields exactly pending/approved/changes_requested", "verify": "cargo test -p but-authz assignment_state_literals_match_column_values", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "parse(merged)/parse(empty)/parse(Approved)/parse(PENDING) all Err (no default)", "verify": "cargo test -p but-authz assignment_state_parse_rejects_unknown", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "Authority variant set unchanged + invariant_build_gates green (AssignmentState is not an Authority)", "verify": "cargo test -p but-authz invariant_build_gates && cargo test -p but-authz assignment_state_not_an_authority", "maps_to_ac": "AC-4" }
  ]
}
-->
