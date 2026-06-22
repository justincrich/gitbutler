# STEER-007: Denial-steering telemetry event (`code`, `class`, `had_lateral_action`, menu length) on the existing tracing path — no new infra

## What this does

On the existing `tracing` path, emit a structured denial-steering event carrying `code`, `class`, `had_lateral_action`, and `menu_length` every time a governed denial is produced, so a fleet operator can measure whether steering reduces hard-quits and loops. No new telemetry infrastructure.

## Why

Sprint 08 (STEER — Capability-Aware Denials) · PRD UC-STEER-03 · Capability CAP-STEER-01. Triggering a `branch.protected` denial (lateral menu present) emits an event with code="branch.protected", class="actor_correctable", had_lateral_action=true, menu_length≥2; triggering an `operator_required` denial (config.invalid / no-handle) emits code present, class="operator_required", had_lateral_action=false, menu_length=0; both captured from a real tracing_subscriber layer in an integration test.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-api steer_telemetry_actor_correctable_event_fields` (Actor-correctable denial emits an event with all four fields [PRIMARY]). Full gate set in the spec below.

## Scope

- crates/but-api/src/commit/gate.rs (MODIFY) — emit the denial-steering tracing event at the payload-build site (commit path)
- crates/but-api/src/legacy/merge_gate.rs (MODIFY) — emit the event on the merge-path denial construction
- crates/but-api/src/legacy/forge.rs (MODIFY, if the forge review-gate constructs a distinct denial path) — emit the event there too
- a shared helper (e.g. in the carrier/envelope module STEER-001 adds, or a small fn in but-authz/but-api) to compute had_lateral_action + menu_length once — MODIFY/NEW as STEER-001's envelope allows
- crates/but-api/tests/steer_telemetry.rs (NEW) — the capturing-subscriber integration proofs

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: STEER-007 - Denial-steering telemetry event (`code`, `class`, `had_lateral_action`, menu length) on the existing tracing path — no new infra
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P1
EFFORT:     S  (120 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-STEER-03
CAPABILITIES: CAP-STEER-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-api steer_telemetry_actor_correctable_event_fields   |   cargo test -p but-api steer_telemetry_operator_required_empty_menu steer_telemetry_discovery_only_no_lateral   |   cargo test -p but-api steer_telemetry_event_is_observation_only
  lint:  cargo clippy -p but-api --all-targets && cargo check -p but-api --all-targets && cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Triggering a `branch.protected` denial (lateral menu present) emits an event with code="branch.protected", class="actor_correctable", had_lateral_action=true, menu_length≥2; triggering an `operator_required` denial (config.invalid / no-handle) emits code present, class="operator_required", had_lateral_action=false, menu_length=0; both captured from a real tracing_subscriber layer in an integration test.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST emit the event with the `tracing` macros already in use across but-api (`tracing::warn!`/`tracing::info!`/`tracing::event!`) at the single site where the steering payload is constructed/serialized — NEVER introduce a new telemetry crate, sink, or exporter (tracing is already a workspace dep; trace setup is crates/but/src/trace.rs).
- [MUST] MUST carry exactly these four fields on the event: `code` (the stable denial code string), `class` (the DenialClass enum rendered as its stable string, e.g. "actor_correctable"/"operator_required"), `had_lateral_action` (bool — true iff authorized_actions has ≥1 non-discovery lateral entry), and `menu_length` (usize — authorized_actions.len()).
- [MUST] MUST emit the event WITHOUT changing the deny/allow decision, the exit code, or any existing denial field — it is observation-only, fired on the denial path the gate already takes.
- [NEVER] NEVER compute `had_lateral_action` as merely `menu_length > 0`: the discovery affordance (`but perm list`) is always appended, so a menu of only the discovery entry has NO lateral action — `had_lateral_action` must be false in that case (the metric exists to measure whether steering actually offered a lateral move).
- [NEVER] NEVER log principal-supplied or config-derived free text in the event beyond the stable code/class enum strings and the two numeric/bool metrics (avoid re-introducing the R15 injection surface into telemetry).
- [NEVER] NEVER make the test assert against a mocked subscriber — capture from a real `tracing_subscriber` layer installed in-process for the test (the real sink the daemon/CLI uses).
- [STRICTLY] STRICTLY emit one event per denial (not per gate hop) so menu_length/had_lateral_action are unambiguous; site it once at the payload-build boundary shared by all carriers.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: Actor-correctable denial emits an event with all four fields [PRIMARY]
- [ ] AC-2: Operator-required denial emits had_lateral_action=false, menu_length=0
- [ ] AC-3: Discovery-only menu sets had_lateral_action=false
- [ ] AC-4: Event does not alter the denial decision or exit
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: Actor-correctable denial emits an event with all four fields [PRIMARY] [PRIMARY]
  GIVEN: the committed `governed_protected_main` fixture; a real `tracing_subscriber` capturing layer installed in-process for the test; BUT_AGENT_HANDLE=rev (holds reviews:write, lacks contents:write) attempting a commit to protected `main`
  WHEN:  the commit gate denies (branch.protected for a holder, or perm.denied for missing contents:write) and the steering payload is built
  THEN:  the captured tracing events contain exactly one denial-steering event carrying `code` (the stable code), `class`="actor_correctable", `had_lateral_action`=true, and `menu_length`≥1
  TEST_TIER: integration   VERIFICATION_SERVICE: tracing-subscriber
  VERIFY: cargo test -p but-api steer_telemetry_actor_correctable_event_fields
  SCENARIO: would fail if disconnect; stub; empty; static | must observe: exactly `1` denial-steering event; field `code` present with a non-empty value (e.g. `branch.protected`); field `class` `== "actor_correctable"`; field `had_lateral_action` `== true`; field `menu_length` `>= 1` | must NOT observe: `0` captured denial-steering events; a `class` field absent or `empty`; `menu_length` `== 0` on an actor-correctable denial that has a lateral menu

AC-2: Operator-required denial emits had_lateral_action=false, menu_length=0
  GIVEN: the committed `governed_protected_main` fixture mutated to commit a malformed `gates.toml` at the target ref (config.invalid path); a capturing tracing layer; BUT_AGENT_HANDLE=dev
  WHEN:  a gated action runs against the malformed config and is denied operator_required
  THEN:  the captured event carries class="operator_required", had_lateral_action=false, and menu_length=0
  TEST_TIER: integration   VERIFICATION_SERVICE: tracing-subscriber
  VERIFY: cargo test -p but-api steer_telemetry_operator_required_empty_menu
  SCENARIO: would fail if stub; static; empty | must observe: a denial-steering event with `class` `== "operator_required"`; `had_lateral_action` `== false`; `menu_length` `== 0` | must NOT observe: `class` `== "actor_correctable"` on config.invalid (wrong constant); `had_lateral_action` `== true` with an `empty` menu

AC-3: Discovery-only menu sets had_lateral_action=false
  GIVEN: the committed `governed_protected_main` fixture; a capturing tracing layer; a principal whose effective set yields ONLY the discovery affordance (no lateral action) on a denial
  WHEN:  the denial is produced with authorized_actions == [discovery only]
  THEN:  the event carries had_lateral_action=false while menu_length=1 (the discovery entry), proving the metric distinguishes a lateral move from the always-present discovery affordance
  TEST_TIER: integration   VERIFICATION_SERVICE: tracing-subscriber
  VERIFY: cargo test -p but-api steer_telemetry_discovery_only_no_lateral
  SCENARIO: would fail if stub; static | must observe: `had_lateral_action` `== false`; `menu_length` `== 1` (the discovery entry only) | must NOT observe: `had_lateral_action` `== true` for a discovery-only menu (it must be `false`); `had_lateral_action` computed as `menu_length > 0` (the `default`/naive formula)

AC-4: Event does not alter the denial decision or exit
  GIVEN: the committed `governed_protected_main` fixture; BUT_AGENT_HANDLE=rev attempting a commit to protected main
  WHEN:  the gate denies with the event emitted
  THEN:  the denial still returns the same stable code and the ref is unchanged (the event is observation-only — no deny->allow flip, no field dropped)
  TEST_TIER: integration   VERIFICATION_SERVICE: but-api
  VERIFY: cargo test -p but-api steer_telemetry_event_is_observation_only
  SCENARIO: would fail if disconnect; stub | must observe: `classify_error` `code` is the stable denial code (e.g. `branch.protected` or `perm.denied`); `refs/heads/main` `ref_id` `==` its value before the call (identical after) | must NOT observe: a successful commit (deny->allow flip; the ref must be `unchanged`); an advanced `main` `ref_id` (it must be `unchanged`) (and no such entry/value present — the empty/start state must be excluded)

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, happy_path): An actor-correctable denial emits one event with code + class==actor_correctable + had_lateral_action==true + menu_length>=1
    VERIFY: cargo test -p but-api steer_telemetry_actor_correctable_event_fields
- TC-2 (-> AC-2, edge): An operator_required denial emits class==operator_required + had_lateral_action==false + menu_length==0
    VERIFY: cargo test -p but-api steer_telemetry_operator_required_empty_menu
- TC-3 (-> AC-3, edge): A discovery-only menu emits had_lateral_action==false with menu_length==1
    VERIFY: cargo test -p but-api steer_telemetry_discovery_only_no_lateral
- TC-4 (-> AC-4, structural): The event does not change the denial code or advance the ref (observation-only)
    VERIFY: cargo test -p but-api steer_telemetry_event_is_observation_only

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-STEER-01
provides: a structured `tracing` denial-steering event carrying code + class + had_lateral_action + menu_length, emitted on the existing tracing path at the denial-payload construction site
consumes: STEER-004 wired payload (the DenialClass + authorized_actions menu the event reads its fields from); the existing `tracing` workspace dependency + the `but` trace subscriber (crates/but/src/trace.rs)
boundary_contracts:
  - CAP-STEER-01: each actor-correctable AND operator-required denial emits exactly one observable telemetry event carrying the four steering metrics, captured from a REAL tracing sink — no new telemetry infrastructure, no change to the deny/allow decision.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-api/src/commit/gate.rs (MODIFY) — emit the denial-steering tracing event at the payload-build site (commit path)
  - crates/but-api/src/legacy/merge_gate.rs (MODIFY) — emit the event on the merge-path denial construction
  - crates/but-api/src/legacy/forge.rs (MODIFY, if the forge review-gate constructs a distinct denial path) — emit the event there too
  - a shared helper (e.g. in the carrier/envelope module STEER-001 adds, or a small fn in but-authz/but-api) to compute had_lateral_action + menu_length once — MODIFY/NEW as STEER-001's envelope allows
  - crates/but-api/tests/steer_telemetry.rs (NEW) — the capturing-subscriber integration proofs
writeProhibited:
  - the gate deny/allow decision - NEVER weaken; the event is observation-only
  - crates/but/src/trace.rs production subscriber config - do not change the CLI's real subscriber; the test installs its own capturing layer
  - any new telemetry crate / Cargo.toml telemetry dependency - tracing is already present; NEVER add new infra
  - crates/but-authz/tests/invariant_build_gates.rs - do not touch the honesty greps here
  - .spec/prds/governance/tasks/sprint-0[1-6]* - frozen
  - Any file not explicitly listed above

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
  - crates/but-api/src/legacy/virtual_branches.rs (lines 768-772): PRIMARY PATTERN — an existing `tracing::warn!(...)` event with structured fields in but-api; emit the denial-steering event in the same idiom (no new infra).
  - crates/but/src/trace.rs (lines 1-75): the real tracing_subscriber registry/layer setup the CLI uses — the test's capturing layer mirrors this real sink; field filter keys on `but`/`but_*` module paths so the event must originate from a but-api module path.
  - crates/but-api/src/commit/gate.rs (lines 55-78, 159-170): the denial-construction site (branch_protected + the authorize/config.invalid path) where the event must fire once per denial after STEER-004 wires the payload.
  - crates/but-api/src/legacy/merge_gate.rs (lines 40-110, 113-124): the merge-path denial construction + classify_error; the event must also fire here so merge denials are measured.
  - crates/but-api/tests/commit_gate.rs (lines 7-59, 665-701): integration-test + governed_repo() fixture style; the telemetry test reuses this fixture and installs a capturing subscriber layer in-process.

--------------------------------------------------------------------------------
CODE PATTERN
--------------------------------------------------------------------------------
pattern: structured tracing event at the denial-construction boundary using the existing `tracing::warn!`/`event!` macros with named fields; the test installs an in-process capturing `tracing_subscriber` layer (real sink) and asserts the captured field values.
pattern_source: crates/but-api/src/legacy/virtual_branches.rs:768 (tracing::warn! with structured fields)
anti_pattern: Computing had_lateral_action as menu_length>0 (the discovery affordance is always present); asserting against a mocked subscriber; adding a new telemetry exporter.
references: 02-uc-steer.md UC-STEER-03 AC-6; 04-e2e-testing-criteria.md T-STEER-030; 03-technical-requirements-delta.md §7 (L-fields on the existing path)
interaction_notes:
  - reads class + authorized_actions from STEER-004's wired payload
  - had_lateral_action must exclude the discovery affordance — coordinate with STEER-003's CATALOG[discovery] / AFFORDANCE_MAP so the discovery entry is identifiable

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
depends_on: STEER-004
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
      "description": "GIVEN a capturing tracing layer and an actor-correctable denial, WHEN it fires, THEN one event carries code + class==actor_correctable + had_lateral_action==true + menu_length>=1",
      "verify": "cargo test -p but-api steer_telemetry_actor_correctable_event_fields"
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "GIVEN config.invalid, WHEN denied, THEN the event carries class==operator_required + had_lateral_action==false + menu_length==0",
      "verify": "cargo test -p but-api steer_telemetry_operator_required_empty_menu"
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "GIVEN a discovery-only menu, WHEN denied, THEN had_lateral_action==false with menu_length==1",
      "verify": "cargo test -p but-api steer_telemetry_discovery_only_no_lateral"
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "description": "GIVEN a denial with the event emitted, WHEN classified, THEN the code is unchanged and the ref does not advance",
      "verify": "cargo test -p but-api steer_telemetry_event_is_observation_only"
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "actor-correctable event field set",
      "maps_to_ac": "AC-1",
      "verify": "cargo test -p but-api steer_telemetry_actor_correctable_event_fields"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "operator-required event field set",
      "maps_to_ac": "AC-2",
      "verify": "cargo test -p but-api steer_telemetry_operator_required_empty_menu"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "discovery-only had_lateral_action false",
      "maps_to_ac": "AC-3",
      "verify": "cargo test -p but-api steer_telemetry_discovery_only_no_lateral"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "observation-only (no decision change)",
      "maps_to_ac": "AC-4",
      "verify": "cargo test -p but-api steer_telemetry_event_is_observation_only"
    }
  ]
}
-->
