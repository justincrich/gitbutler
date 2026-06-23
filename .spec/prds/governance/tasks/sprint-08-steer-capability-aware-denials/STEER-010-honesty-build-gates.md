# STEER-010: Net-new honesty build-gates: closed-catalog grep (no `format!`/interpolation/config-sourced text in `authorized_actions`/`do_not`) + table/affordance coverage grep (single `ROUTE_AUTHORITY_TABLE`; every gated route ∈ table; every table route has an `AFFORDANCE_MAP` entry not naming the denied route) + non-defaulted `class` match + reviewer pass

## What this does

Close the anti-injection + single-source proof: add net-new closed-catalog and table/affordance-coverage honesty greps beside the shipped patterns, prove the class match is non-defaulted (a compile break on omission), and run the adversarial rust-reviewer pass over the STEER chain — without over-claiming runtime properties that belong to STEER-009.

## Why

Sprint 08 (STEER — Capability-Aware Denials) · PRD UC-STEER-06 · Capability CAP-STEER-01. An injected `format!` in an authorized_actions/do_not construction trips the closed-catalog grep; a gated route missing from ROUTE_AUTHORITY_TABLE or an AFFORDANCE_MAP entry naming the denied route trips the coverage grep; a removed class arm fails to compile; the shipped greps stay green; and the reviewer emits an APPROVED/NEEDS_FIXES verdict over the chain.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-authz --test invariant_build_gates steer_closed_catalog_grep_has_teeth` (Closed-catalog grep (exact `menu.rs` path scope) trips on an injected format!/push_str/write!/concat!/Cow::Owned in authorized_actions/do_not but NOT on a format! in the R15 message/remediation_hint [PRIMARY]). Full gate set in the spec below.

## Scope

- crates/but-authz/tests/invariant_build_gates.rs (MODIFY — ADDITIVE ONLY) — add the closed-catalog + table/affordance-coverage pattern constants, path constants, assert calls, and seeded violating-fixture teeth controls; add the non-defaulted-class grep over `crates/but-authz/src/authorize.rs:91-98` (Rust type-system exhaustiveness; no trybuild/compile_fail fixture)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: STEER-010 - Net-new honesty build-gates: closed-catalog grep (no `format!`/interpolation/config-sourced text in `authorized_actions`/`do_not`) + table/affordance coverage grep (single `ROUTE_AUTHORITY_TABLE`; every gated route ∈ table; every table route has an `AFFORDANCE_MAP` entry not naming the denied route) + non-defaulted `class` match + reviewer pass
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     M  (120 min)
AGENT:      implementer=rust-reviewer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-STEER-06
CAPABILITIES: CAP-STEER-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-authz --test invariant_build_gates steer_closed_catalog_grep_has_teeth   |   cargo test -p but-authz --test invariant_build_gates steer_table_affordance_coverage_grep_has_teeth   |   cargo test -p but-authz --test invariant_build_gates steer_class_match_is_non_defaulted   |   cargo test -p but-authz --test invariant_build_gates steer_class_field_routed_through_match   |   cargo test -p but-authz --test invariant_build_gates
  lint:  cargo clippy -p but-authz --all-targets && cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
An injected `format!` in an authorized_actions/do_not construction trips the closed-catalog grep; a gated route missing from ROUTE_AUTHORITY_TABLE or an AFFORDANCE_MAP entry naming the denied route trips the coverage grep; a removed class arm fails to compile; the shipped greps stay green; and the reviewer emits an APPROVED/NEEDS_FIXES verdict over the chain.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST ADD the net-new patterns BESIDE the shipped ones in `crates/but-authz/tests/invariant_build_gates.rs` — NEVER replace or weaken the shipped no-role-preset (ROLE_BRANCH_PATTERN), no-human-vs-AI (HUMAN_OR_LABEL_BRANCH_PATTERN), positive-authorize (AUTHORITY_POSITIVE_PATTERN), or no-Permission (PERMISSION_CARRIER_PATTERN) assertions; add new path constants + new pattern constants + new assert_grep_* calls.
- [MUST] MUST add a CLOSED-CATALOG grep over the EXACT menu path set STEER-003 owns — `crates/but-authz/src/menu.rs` (the authorized_actions/do_not derivation AND the closed \&'static str CATALOG at `crates/but-authz/src/menu.rs:160`) — NOT the whole `crates/but-authz/src` tree — asserting NO `format!`, `push_str`, `write!`, `concat!`, `Cow::Owned`, or other string interpolation/config-sourced value flows into authorized_actions or do_not (the NEW fields ONLY; message/remediation_hint/unmet[] interpolate config and are R15, explicitly NOT claimed closed and explicitly OUTSIDE this grep's scope). Include TWO controls: (1) a TEETH control — an injected `format!`/`push_str`/`write!`/`concat!`/`Cow::Owned` in an `authorized_actions`/`do_not` construction the grep MUST trip; (2) a BOUNDARY control — a `format!` seeded in a `message`/`remediation_hint` construction the grep must NOT trip, proving the scope correctly excludes the R15 fields (mirroring assert_seeded_controls_fire for the positive teeth).
- [MUST] MUST add a TABLE/AFFORDANCE COVERAGE grep: a single `ROUTE_AUTHORITY_TABLE` symbol is referenced by both the gate path and the menu module (single source); every gated route appears in the table; every table route has an `AFFORDANCE_MAP` entry that does NOT name the denied route at the denied ref — with a seeded fixture proving the grep trips when a gated route is missing from the table OR an AFFORDANCE_MAP entry names the denied route.
- [MUST] MUST prove the `DenialCause` -> `class` mapping is exhaustive by the TYPE SYSTEM — the `match self { ... }` over `DenialCause` in `crates/but-authz/src/authorize.rs:91-98` carries NO `_ =>` catch-all arm; omitting or adding a variant without an arm is a Rust compiler `E0004` non-exhaustive-match error. PLUS a grep asserting the classification match has no `_ =>` wildcard over the `DenialCause` cases. This REPLACES the prior 'documented compile-break control' — a removed/unclassified cause is now a type-system-enforced compile break, not a silent `actor_correctable`. No `trybuild`/`compile_fail` fixture is required.
- [MUST] MUST add a grep proving every constructor/gate that sets `class` does so by routing through the `DenialCause` classification match (`crates/but-authz/src/authorize.rs:91-98`) and NOT by direct `class:` field assignment in the constructor. The grep covers `crates/but-authz/src/authorize.rs`, `crates/but-authz/src/denial.rs`, `crates/but-authz/src/config.rs`, `crates/but-api/src/commit/gate.rs`, `crates/but-api/src/legacy/forge.rs`, `crates/but-api/src/legacy/merge_gate.rs`, and `crates/but-api/src/legacy/config_mutate.rs`. Direct assignment outside the match defeats the type-system exhaustiveness guarantee because a future `DenialCause` variant could be classified silently by a stale constant.
- [MUST] MUST run the adversarial rust-reviewer pass over the STEER chain (STEER-001..009) and emit a verdict, blocking on stubs / lying-menu / weakened greps / over-claims.
- [NEVER] NEVER claim the closed-catalog grep covers `message`/`unmet[]` — those interpolate config strings (R15, mitigated separately); the grep is for the NEW fields (authorized_actions/do_not) ONLY, and the task must STATE this scope.
- [NEVER] NEVER let the coverage grep over-claim same-`cfg`/ref equality — that is a RUNTIME property proven by STEER-009 (T-STEER-009/024), NOT a static grep; the grep proves single-symbol + route coverage ONLY.
- [NEVER] NEVER weaken, narrow, or remove a shipped honesty-grep pattern or ENFORCEMENT_PATH; additive coverage only.
- [NEVER] NEVER scope the closed-catalog grep to the whole `crates/but-authz/src` tree — it MUST be scoped to the `menu.rs` module path only (`crates/but-authz/src/menu.rs`), so a legitimate `format!` in the R15 `message`/`remediation_hint` construction does not false-positive.
- [STRICTLY] STRICTLY include, for EACH new grep, a seeded violating-fixture teeth control asserting the grep matches the injected violation; AND for the closed-catalog grep ALSO include the R15 boundary control (a `format!` in message/remediation_hint that must NOT trip) — a grep with no teeth is a fake gate, and a grep that over-reaches onto R15 is a false gate.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: Closed-catalog grep (exact menu path scope) trips on an injected format!/push_str/write!/concat!/Cow::Owned in authorized_actions/do_not but NOT on a format! in the R15 message/remediation_hint [PRIMARY]
- [ ] AC-2: Coverage grep trips when a gated route is missing from the table or an AFFORDANCE_MAP entry names the denied route
- [ ] AC-3: The DenialCause->class mapping is exhaustive by type (a missing/unhandled variant is a Rust compiler `E0004` non-exhaustive-match error)
- [ ] AC-4: Shipped honesty greps stay green + reviewer verdict emitted
- [ ] AC-5: Denial constructors route `class` through the `DenialCause` classification match; no direct `class:` field assignment outside it
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: Closed-catalog grep (exact menu.rs path scope) trips on an injected format!/push_str/write!/concat!/Cow::Owned in authorized_actions/do_not but NOT on a format! in the R15 message/remediation_hint [PRIMARY] [PRIMARY]
  GIVEN: the committed `steer_engine_source`; the closed-catalog grep is scoped to the EXACT menu module path STEER-003 owns — `crates/but-authz/src/menu.rs` (authorized_actions/do_not derivation AND the closed \&'static str CATALOG at `crates/but-authz/src/menu.rs:160`) — NOT the whole `crates/but-authz/src` tree (so it cannot false-positive on the legitimate `format!` in `message`/`remediation_hint` construction, which are R15); plus two seeded temp fixtures — (a) an injected `format!`/`push_str`/`write!`/`concat!`/`Cow::Owned` in an `authorized_actions`/`do_not` construction, and (b) a `format!` in a `message`/`remediation_hint` construction
  WHEN:  the closed-catalog grep runs over the exact `menu.rs` path AND over both seeded fixtures
  THEN:  the grep finds ZERO matches on the real `menu.rs` source (closed catalog holds for the new fields) AND DOES match the seeded `authorized_actions`/`do_not` vector fixture (teeth) AND does NOT match the seeded `message`/`remediation_hint` `format!` fixture (the scope correctly EXCLUDES the R15 fields) — proving the grep is scoped to the new fields only, neither over- nor under-reaching
  TEST_TIER: build-gate   VERIFICATION_SERVICE: but-authz
  VERIFY: cargo test -p but-authz --test invariant_build_gates steer_closed_catalog_grep_has_teeth
  SCENARIO: would fail if an injected format!/push_str/write!/concat!/Cow::Owned in a menu/do_not (authorized_actions) construction does NOT fail the grep (the grep has no teeth on the new fields); a legitimate format! in the R15 message/remediation_hint construction DOES trip the grep (the scope over-reaches onto the R15 fields); stub; static | must observe: `0` matches over the real `menu.rs` construction sites (closed catalog holds for the new fields); the seeded authorized_actions/do_not vector fixture yields `>= 1` grep match (teeth on the new fields); the seeded message/remediation_hint `format!` fixture yields `0` grep matches (the R15 boundary control: legitimate `format!` in message/remediation_hint does NOT trip) | must NOT observe: a match in the real authorized_actions/do_not construction (must be `0`); the grep failing to detect the injected authorized_actions/do_not vector (`0` matches = `no` teeth on the new fields); the grep tripping on the message/remediation_hint `format!` (a `false`-positive on the R15 fields — the scope must exclude them)

AC-2: Coverage grep trips when a gated route is missing from the table or an AFFORDANCE_MAP entry names the denied route
  GIVEN: the committed `steer_engine_source` (single ROUTE_AUTHORITY_TABLE referenced by gate + menu; every gated route in the table; every table route's AFFORDANCE_MAP entry excludes the denied route); seeded temp fixtures (a) omitting a gated route from the table and (b) an AFFORDANCE_MAP entry naming the denied route
  WHEN:  the coverage grep runs over the real source AND over each seeded fixture
  THEN:  the grep passes on the real source (single symbol + full coverage + no self-naming affordance) AND trips on BOTH seeded fixtures — proving the coverage gate has teeth; the grep proves single-symbol + coverage ONLY (NOT same-ref equality, which is STEER-009's runtime property)
  TEST_TIER: build-gate   VERIFICATION_SERVICE: but-authz
  VERIFY: cargo test -p but-authz --test invariant_build_gates steer_table_affordance_coverage_grep_has_teeth
  SCENARIO: would fail if a gated route missing from ROUTE_AUTHORITY_TABLE does NOT fail the grep; an AFFORDANCE_MAP entry naming the denied route does NOT fail the grep; stub; static | must observe: the real source passes (`1` `ROUTE_AUTHORITY_TABLE` symbol, full route coverage, no self-naming affordance); the missing-route fixture yields `>= 1` coverage-grep match (teeth); the self-naming-affordance fixture yields `>= 1` coverage-grep match (teeth) | must NOT observe: two divergent table symbols passing as single-source (must be `1` symbol); the grep claiming same-cfg/ref equality (out of scope — STEER-009 owns it; `none` here); a seeded violation passing undetected (`0` matches on a real violation)

AC-3: The DenialCause->class mapping is exhaustive by type (a missing/unhandled variant is a Rust compiler `E0004` non-exhaustive-match error)
  GIVEN: the committed `steer_engine_source` class-classification fn in `crates/but-authz/src/authorize.rs:91-98`, which matches the `DenialCause` enum (modeling the (code, principal-resolution) input) with NO `_ =>` arm
  WHEN:  a grep asserts the classification match has no `_ =>` catch-all over the `DenialCause` cases
  THEN:  the match is non-defaulted (no silent `actor_correctable` fallthrough); adding a future `DenialCause` variant without an arm — or removing an existing arm — is enforced by the Rust compiler as a non-exhaustive-match error (`E0004`)
  TEST_TIER: build-gate   VERIFICATION_SERVICE: but-authz
  VERIFY: cargo test -p but-authz --test invariant_build_gates steer_class_match_is_non_defaulted
  SCENARIO: would fail if the class match carries a `_ =>` catch-all that silently classifies unknown causes as actor_correctable; the `match` is expressed over `code: &str` rather than the `DenialCause` enum so the compiler cannot enforce exhaustiveness; stub; static | must observe: `0` `_ =>` catch-all arms in the `DenialCause->class match` in `authorize.rs:91-98`; the match has explicit arms over the five `DenialCause` variants | must NOT observe: a `_ => DenialClass::ActorCorrectable` (or any silent `default`) in the classification match; a compile succeeding after a `DenialCause` variant is removed or left unhandled

AC-4: Shipped honesty greps stay green + reviewer verdict emitted
  GIVEN: the STEER chain (STEER-001..009) landed; invariant_build_gates.rs with the new patterns added beside the shipped ones
  WHEN:  the full invariant_build_gates harness runs and the rust-reviewer pass executes over the chain
  THEN:  the shipped no-role-preset / no-human-vs-AI / positive-authorize / no-Permission assertions still pass (not weakened), the new patterns pass, and the reviewer emits an APPROVED/NEEDS_FIXES verdict citing file:line for any finding (stubs, lying-menu, weakened greps, over-claims)
  TEST_TIER: build-gate   VERIFICATION_SERVICE: but-authz
  VERIFY: cargo test -p but-authz --test invariant_build_gates
  SCENARIO: would fail if a shipped pattern was weakened/removed (it would no longer fire on its seeded control); stub; static | must observe: all `4` shipped seeded controls (role/label/positive-authorize/Permission) still fire; both new seeded controls (closed-catalog, coverage) yield `>= 1` grep match; a reviewer verdict `"APPROVED"` or `"NEEDS_FIXES"` with `file:line` citations | must NOT observe: a shipped pattern that no longer fires on its seeded control (weakened / `removed`); a reviewer verdict that omits the chain or cites `no` evidence

AC-5: Constructors route `class` through the `DenialCause` classification match, never by direct field assignment
  GIVEN: the constructor/gate source files that populate a steering denial: `crates/but-authz/src/authorize.rs`, `crates/but-authz/src/denial.rs`, `crates/but-authz/src/config.rs`, `crates/but-api/src/commit/gate.rs`, `crates/but-api/src/legacy/forge.rs`, `crates/but-api/src/legacy/merge_gate.rs`, and `crates/but-api/src/legacy/config_mutate.rs`
  WHEN:  a grep for direct `class:` field assignment outside the `DenialCause` classification match runs over those files
  THEN:  ZERO direct `class:` assignments are found outside the classification match; every `class` value is produced by `DenialCause::*.class()` (the non-defaulted match in `crates/but-authz/src/authorize.rs:91-98`)
  TEST_TIER: build-gate   VERIFICATION_SERVICE: but-authz
  VERIFY: cargo test -p but-authz --test invariant_build_gates steer_class_field_routed_through_match
  SCENARIO: would fail if a constructor hard-codes `class: DenialClass::ActorCorrectable` directly, bypassing the exhaustive match; the grep ignores the `DenialCause::*.class()` callsite inside `authorize.rs:91-98` and flags only direct assignments; stub; static | must observe: `0` direct `class:` field assignments in the listed constructor files outside the classification match; the classification match in `authorize.rs:91-98` has explicit arms for all five `DenialCause` variants | must NOT observe: a constructor file containing `class:` assigned to a literal `DenialClass::*` value (e.g. `class: DenialClass::ActorCorrectable`) outside the match; the grep flagging the `pub fn class(self)` match itself as a violation

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, happy_path): Closed-catalog grep (scoped to menu.rs): 0 matches on real authorized_actions/do_not construction; the injected authorized_actions/do_not format!/push_str/write!/concat!/Cow::Owned fixture trips (teeth); a format! in message/remediation_hint does NOT trip (R15 boundary control)
    VERIFY: cargo test -p but-authz --test invariant_build_gates steer_closed_catalog_grep_has_teeth
- TC-2 (-> AC-2, structural): Coverage grep: single ROUTE_AUTHORITY_TABLE referenced by gate + menu, full route coverage, no self-naming affordance; missing-route and self-naming-affordance fixtures both detected (teeth)
    VERIFY: cargo test -p but-authz --test invariant_build_gates steer_table_affordance_coverage_grep_has_teeth
- TC-3 (-> AC-3, structural): The DenialCause->class match in `authorize.rs:91-98` has no `_ =>` arm; a missing/unhandled variant is a Rust compiler `E0004` non-exhaustive-match error
    VERIFY: cargo test -p but-authz --test invariant_build_gates steer_class_match_is_non_defaulted
- TC-4 (-> AC-4, structural): The shipped no-role-preset/no-human-vs-AI/positive-authorize/no-Permission greps + their seeded controls still pass alongside the new patterns
    VERIFY: cargo test -p but-authz --test invariant_build_gates
- TC-5 (-> AC-4, happy_path): The rust-reviewer pass emits an APPROVED/NEEDS_FIXES verdict over STEER-001..009 with file:line citations
    VERIFY: manual reviewer verdict recorded against the chain commit
- TC-6 (-> AC-5, structural): No direct `class:` field assignment in denial constructors outside the `DenialCause` classification match
    VERIFY: cargo test -p but-authz --test invariant_build_gates steer_class_field_routed_through_match

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-STEER-01
provides: a net-new closed-catalog honesty grep SCOPED to the exact `menu.rs` module path (`crates/but-authz/src/menu.rs`) — no `format!`, `push_str`, `write!`, `concat!`, `Cow::Owned`, or other string interpolation/config-sourced text in authorized_actions/do_not construction (new fields only), with BOTH a teeth control and an R15 boundary control; a net-new table/affordance coverage grep (single ROUTE_AUTHORITY_TABLE symbol; every gated route in the table; every table route has an AFFORDANCE_MAP entry not naming the denied route); type-system exhaustiveness of the DenialCause->class match (no `_ =>` arm in `authorize.rs:91-98`; a missing/unhandled variant is a compiler `E0004` error); a grep proving constructors route `class` through the match and never assign it directly; and the adversarial rust-reviewer verdict over the STEER chain
consumes: STEER-002 Route enum + ROUTE_AUTHORITY_TABLE symbol; STEER-003 AFFORDANCE_MAP + the closed &'static str CATALOG and the authorized_actions/do_not construction sites in `crates/but-authz/src/menu.rs` (the EXACT closed-catalog grep scope); STEER-004 the DenialCause enum (modeling the (code, principal-resolution) classification input) matched with no `_ =>` arm in `authorize.rs:91-98`
boundary_contracts:
  - CAP-STEER-01 anti-injection + single-source: all authorized_actions/do_not text is closed-catalog &'static str (no format!, push_str, write!, concat!, Cow::Owned, or other string interpolation/config-sourced — new fields ONLY, proven by a grep scoped to `crates/but-authz/src/menu.rs`; message/remediation_hint/unmet[] are R15, NOT claimed closed and OUTSIDE the grep scope, proven by an R15 boundary control that must NOT trip); a single ROUTE_AUTHORITY_TABLE is referenced by both gate and menu, every gated route is in it, and every table route has an AFFORDANCE_MAP entry that does not name the denied route at the denied ref; the DenialCause->class match is exhaustive by the type system (no `_ =>` arm in `authorize.rs:91-98`; a missing/unhandled variant is a non-exhaustive-match `E0004` compile error); constructors route `class` through the match, never by direct field assignment. Same-ref equality is a RUNTIME property (T-STEER-009/024), NOT this grep.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-authz/tests/invariant_build_gates.rs (MODIFY — ADDITIVE ONLY) — add the closed-catalog + table/affordance-coverage pattern constants, path constants, assert calls, and seeded violating-fixture teeth controls; add the non-defaulted-class grep over `crates/but-authz/src/authorize.rs:91-98` (Rust type-system exhaustiveness; no trybuild/compile_fail fixture)
writeProhibited:
  - the gate deny/allow decision - NEVER weaken
  - the shipped honesty-grep patterns (ROLE_BRANCH_PATTERN / HUMAN_OR_LABEL_BRANCH_PATTERN / AUTHORITY_POSITIVE_PATTERN / PERMISSION_CARRIER_PATTERN) and ENFORCEMENT_PATHS - NEVER replace, narrow, or remove; add beside
  - ROUTE_AUTHORITY_TABLE / AFFORDANCE_MAP / CATALOG / the class match production source - this is the build-gate + review task; product code is STEER-002/003/004 (FLAG gaps, do not author)
  - .spec/prds/governance/tasks/sprint-0[1-6]* - frozen
  - Any file not explicitly listed above

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
  - crates/but-authz/tests/invariant_build_gates.rs (lines 10-100, 126-237): PRIMARY PATTERN — the pattern constants, ENFORCEMENT_PATHS, assert_grep_has_matches/no_matches, and assert_seeded_controls_fire (the teeth mechanism). ADD new path/pattern constants + new assertions + seeded violating fixtures BESIDE these; never weaken the shipped ones.
  - .spec/prds/governance/enrichments/v1.4.0-capability-aware-denials/03-technical-requirements-delta.md (lines 92-118, 138-148): §4 ROUTE_AUTHORITY_TABLE (single-source, in but-authz, preserve positive-authorize grep) + §5 AFFORDANCE_MAP (every route has an entry not naming the denied route) + §9 invariants (closed catalog = new fields only; R15 message/unmet excluded; non-defaulted class).
  - .spec/prds/governance/enrichments/v1.4.0-capability-aware-denials/04-e2e-testing-criteria.md (lines 62-69): T-STEER-025/026/029 — the exact build-gate scope: single-symbol + coverage ONLY (same-ref equality is T-STEER-009/024 runtime, not the grep); closed-catalog for new fields only; non-defaulted class match.
  - crates/but-api/src/commit/gate.rs (lines 55-78, 257-292): a gated route call site that must reference the single ROUTE_AUTHORITY_TABLE symbol after STEER-002; the coverage grep asserts the gate + menu share the one symbol. `CommitGateError` is sited at `gate.rs:54`.
  - .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-008-merge-gate-failclosed-target-ref-only.md (lines all): house style for a honesty/target-ref build-gate proof + reviewer verdict shape (the closest analogue review-gate task in this PRD).
  - crates/but-authz/src/menu.rs (STEER-003 deliverable) (lines 150-249 — the CATALOG, AFFORDANCE_MAP, and authorized_actions derivation): the EXACT closed-catalog grep scope; do NOT widen the grep to the whole but-authz/src tree (the R15 message/remediation_hint construction in denial.rs/gate.rs legitimately uses format! and must be excluded).
  - crates/but-authz/src/authorize.rs (STEER-004 DenialCause enum + classification fn) (lines 80-99 — the DenialCause enum + the non-defaulted `DenialCause` -> `DenialClass` match at :91-98): assert the match is exhaustive by type (`E0004` on a missing/unhandled variant), with no trybuild required.

--------------------------------------------------------------------------------
CODE PATTERN
--------------------------------------------------------------------------------
pattern: additive honesty grep + seeded teeth control: add new pattern/path constants and assert_grep_* calls beside the shipped ones in invariant_build_gates.rs, each with a seeded violating fixture proving the grep bites; prove the class match is non-defaulted by build + grep.
pattern_source: crates/but-authz/tests/invariant_build_gates.rs:48-95 (assertions) + 176-237 (assert_seeded_controls_fire teeth)
anti_pattern: Claiming the closed-catalog grep covers message/remediation_hint/unmet[] (R15) or over-scoping it to the whole but-authz/src tree (false-positives on R15 format!); claiming the coverage grep proves same-cfg/ref equality (runtime, STEER-009); a grep with no seeded teeth control or no R15 boundary control; a merely 'documented' class-exhaustiveness claim instead of a real non-defaulted Rust `match` that the compiler checks (no `_ =>` arm); weakening a shipped pattern to make a new one pass.
references: 02-uc-steer.md UC-STEER-06 AC-2/3/5; 03-technical-requirements-delta.md §4/§5/§9; 04-e2e-testing-criteria.md T-STEER-025/026/029; 05-delta-replan.md D10
interaction_notes:
  - greps STEER-002's table + STEER-003's CATALOG/AFFORDANCE_MAP (closed-catalog grep scoped to `menu.rs` ONLY) + STEER-004's DenialCause->class match (non-defaulted `match` in `crates/but-authz/src/authorize.rs:91-98`; the compiler enforces exhaustiveness via `E0004`)
  - depends on STEER-009 too since it is the final review/grep that closes the proof
  - must NOT over-claim same-ref equality (STEER-009's runtime property); must NOT over-scope the closed-catalog grep onto the R15 message/remediation_hint fields

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
depends_on: STEER-002, STEER-003, STEER-009
blocks: (none)

CODING STANDARDS: crates/AGENTS.md, RULES.md
```
</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN closed-catalog construction scoped to menu.rs + two seeded fixtures (including format!/push_str/write!/concat!/Cow::Owned vectors), WHEN the grep runs, THEN 0 matches on real source, the authorized_actions/do_not vector fixture trips (teeth), and the message/remediation_hint format! fixture does NOT trip (R15 boundary)",
      "verify": "cargo test -p but-authz --test invariant_build_gates steer_closed_catalog_grep_has_teeth"
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "GIVEN single-source table + coverage, WHEN the coverage grep runs over real + seeded fixtures, THEN real passes and both violations are detected; same-ref equality NOT claimed",
      "verify": "cargo test -p but-authz --test invariant_build_gates steer_table_affordance_coverage_grep_has_teeth"
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "GIVEN the DenialCause->class match in authorize.rs:91-98, WHEN inspected, THEN no `_ =>` arm; a missing/unhandled variant is a Rust compiler E0004 non-exhaustive-match error",
      "verify": "cargo test -p but-authz --test invariant_build_gates steer_class_match_is_non_defaulted"
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "description": "GIVEN the STEER chain, WHEN the full harness + reviewer pass run, THEN shipped greps stay green, new patterns pass, and a reviewer verdict is emitted",
      "verify": "cargo test -p but-authz --test invariant_build_gates"
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "description": "GIVEN the denial constructor files, WHEN a grep for direct class: field assignment outside the DenialCause classification match runs, THEN 0 matches and every class is produced by DenialCause::*.class()",
      "verify": "cargo test -p but-authz --test invariant_build_gates steer_class_field_routed_through_match"
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "closed-catalog grep menu.rs scope + teeth + R15 boundary control (format!/push_str/write!/concat!/Cow::Owned vectors)",
      "maps_to_ac": "AC-1",
      "verify": "cargo test -p but-authz --test invariant_build_gates steer_closed_catalog_grep_has_teeth"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "coverage grep single-source + teeth",
      "maps_to_ac": "AC-2",
      "verify": "cargo test -p but-authz --test invariant_build_gates steer_table_affordance_coverage_grep_has_teeth"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "non-defaulted match over DenialCause in authorize.rs:91-98; missing/unhandled variant yields E0004 compile error",
      "maps_to_ac": "AC-3",
      "verify": "cargo test -p but-authz --test invariant_build_gates steer_class_match_is_non_defaulted"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "shipped greps still green beside new patterns",
      "maps_to_ac": "AC-4",
      "verify": "cargo test -p but-authz --test invariant_build_gates"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "reviewer verdict over the chain",
      "maps_to_ac": "AC-4",
      "verify": "manual reviewer verdict recorded against the chain commit"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "description": "no direct class: field assignment outside the DenialCause classification match",
      "maps_to_ac": "AC-5",
      "verify": "cargo test -p but-authz --test invariant_build_gates steer_class_field_routed_through_match"
    }
  ]
}
-->
