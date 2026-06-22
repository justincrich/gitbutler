# STEER-005: Add the four steering fields to ALL FOUR hand-rolled CLI denial serializers (commit_gate_cli_error, review_gate_cli_error, merge_gate_cli_error, AND governance_cli_error for admin-write); a TEST-ONLY serialization-fault seam (BUT_STEER_FORCE_SERIALIZATION_FAULT, debug-only); class is a stable enum STRING on the serialized envelope; coordinate the Tauri json::Error surface with Sprint 06a MGMT-IPC-002; best-effort fail-closed

## What this does

Serialize the four steering fields at ALL FOUR hand-rolled CLI denial sites (commit_gate_cli_error, review_gate_cli_error, merge_gate_cli_error, governance_cli_error for admin-write) — preferably via to_envelope() — adding the long-missing remediation_hint to the commit/review/governance sites, rendering `class` as a stable enum STRING token branchable without parsing message, preserving each site's existing keys + exit 1, adding a TEST-ONLY `BUT_STEER_FORCE_SERIALIZATION_FAULT` fault-injection seam (debug-only) so the serialization is best-effort fail-closed, and recording the Tauri json::Error surface decision (co-land via MGMT-IPC-002 or a tracked deferral note).

## Why

Sprint 08 (STEER — Capability-Aware Denials) · PRD UC-STEER-01, UC-STEER-06 · Capability CAP-STEER-01. Each of the four CLI serializers emits the four steering fields alongside its existing keys; commit/review/governance sites now also carry remediation_hint; the merge site retains remediation_hint + unmet; an admin-write actor-correctable denial carries the steering payload + the admin-write affordance row; the serialized `class` is a stable enum string token branchable without reading message; a `BUT_STEER_FORCE_SERIALIZATION_FAULT`-forced fault still denies with the legacy fields + exit 1 and never drops a field or flips deny->allow (and the seam is compiled out of release builds); the Tauri-surface decision is explicitly recorded; cargo test -p but green; clippy clean.

## How to verify

PRIMARY **AC-1** — `cargo test -p but governed_loop_steer_commit_cli_serializer` (commit_gate_cli_error emits the four steering fields + the long-missing remediation_hint, exit 1 unchanged). Full gate set in the spec below.

## Scope

- crates/but/src/command/legacy/commit2.rs (MODIFY — commit_gate_cli_error emits the 4 steering fields + remediation_hint via to_envelope)
- crates/but/src/command/legacy/forge/review.rs (MODIFY — review_gate_cli_error + merge_gate_cli_error emit the 4 steering fields)
- crates/but/src/command/perm.rs (MODIFY — governance_cli_error :118 emits the 4 steering fields + remediation_hint for admin-write AdminWriteGateError; RR-2)
- crates/but-authz/src/lib.rs or but-api serializer module (MODIFY — add the `#[cfg(debug_assertions)]`/`cfg(test)`-gated BUT_STEER_FORCE_SERIALIZATION_FAULT fault seam on the to_envelope()/serializer path; SA-1/RR-4)
- crates/but-api/src/json.rs (MODIFY — co-land steering fields in Error IFF MGMT-IPC-002 is being co-landed; otherwise leave and record the explicit deferral note; SA-8)
- crates/but/tests/but/command/governed_loop.rs (MODIFY — add the four serializer-shape cases + the class-token case + the serialization-fault case; do NOT weaken existing assertions)
- crates/but/tests/but/command/steer_cli_serializers.rs (NEW — if a separate module is preferred)
- crates/but-api/tests/steer_json_error_decision.rs (NEW — asserts the recorded Tauri json::Error decision; SA-8)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: STEER-005 - Add the four steering fields to ALL FOUR hand-rolled CLI denial serializers (commit_gate_cli_error, review_gate_cli_error, merge_gate_cli_error, AND governance_cli_error for admin-write); a TEST-ONLY serialization-fault seam (BUT_STEER_FORCE_SERIALIZATION_FAULT, debug-only); class is a stable enum STRING on the serialized envelope; coordinate the Tauri json::Error surface with Sprint 06a MGMT-IPC-002; best-effort fail-closed
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     S  (240 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-STEER-01, UC-STEER-06
CAPABILITIES: CAP-STEER-01

RUNTIME_COMMANDS:
  test:  cargo test -p but governed_loop_steer_commit_cli_serializer   |   cargo test -p but governed_loop_steer_review_cli_serializer   |   cargo test -p but governed_loop_steer_merge_cli_serializer   |   cargo test -p but governed_loop_steer_governance_cli_serializer   |   cargo test -p but governed_loop_steer_class_token_branchable   |   cargo test -p but steer_cli_serialization_fault_fail_closed   |   cargo test -p but-api steer_json_error_decision_recorded   |   cargo test -p but governed_loop
  lint:  cargo clippy -p but -p but-api --all-targets

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Each of the four CLI serializers emits the four steering fields alongside its existing keys; commit/review/governance sites now also carry remediation_hint; the merge site retains remediation_hint + unmet; an admin-write actor-correctable denial carries the steering payload + the admin-write affordance row; the serialized `class` is a stable enum string token branchable without reading message; a `BUT_STEER_FORCE_SERIALIZATION_FAULT`-forced fault still denies with the legacy fields + exit 1 and never drops a field or flips deny->allow (and the seam is compiled out of release builds); the Tauri-surface decision is explicitly recorded; cargo test -p but green; clippy clean.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST add the four steering fields to ALL FOUR hand-rolled CLI serializers: `commit_gate_cli_error` (commit2.rs:679, emits `{code,message}` today — also ADD the long-missing `remediation_hint`), `review_gate_cli_error` (forge/review.rs:89, reads `ForgeGateError` which now carries the four copied fields from STEER-001 RR-1 — also ADD `remediation_hint`), `merge_gate_cli_error` (forge/review.rs:246, already emits `{code,message,remediation_hint,unmet}`), AND `governance_cli_error` (perm.rs:118, emits `{code,message}` today, reads `AdminWriteGateError` which now carries the four copied fields from STEER-001 RR-2 — also ADD `remediation_hint`) — preferably by rendering through the shared `to_envelope()` (STEER-001).
- [MUST] MUST cover the admin-write actor-correctable case (RR-2): a resolved principal lacking `administration:write` is `actor_correctable` (§2), so `but perm grant/revoke` denials serialized by `governance_cli_error` MUST carry `class`/`held_permissions`/`authorized_actions`/`do_not` and surface the §5 admin-write affordance row (`read/inspect`, `request-config-change`, `discovery`) — without this the admin route's uniform-shape (UC-STEER-01 AC-1) is violated. (The admin route is already in STEER-002's ROUTE_AUTHORITY_TABLE and STEER-003's AFFORDANCE_MAP per §5.)
- [MUST] MUST keep each serializer's EXISTING keys + exit code 1 unchanged RELATIVE TO ITS CURRENT OUTPUT: the commit/review/governance sites ADD the missing `remediation_hint` (an improvement, not a regression — UC-STEER-01 AC-2 / C2-fix); `unmet` is asserted ONLY on the merge site (it is a MergeGateError field, not on the commit/review/governance carriers).
- [MUST] MUST render the serialized denial envelope's `class` as a STABLE enum STRING token — exactly `"actor_correctable"` or `"operator_required"` — so an orchestrator can branch on `class` as a stable enum on the SERIALIZED denial JSON WITHOUT parsing `message` (RR-3 / T-STEER-016 / UC-STEER-03 AC-5). The token MUST match the DenialClass serialization chosen in STEER-001/004.
- [MUST] MUST add a TEST-ONLY serialization-fault-injection seam on the best-effort `to_envelope()`/serializer path, activated by the environment variable `BUT_STEER_FORCE_SERIALIZATION_FAULT` (this exact name — STEER-009 consumes it), gated so it exists ONLY in test/debug builds (`#[cfg(debug_assertions)]` or a `cfg(test)`/feature flag) and is COMPILED OUT of release builds — never a production bypass (SA-1/RR-4). AC-6 activates the fault via that env var.
- [MUST] MUST make serialization BEST-EFFORT FAIL-CLOSED (invariant §9.5 / security MED #6): if deriving OR serializing the steering payload faults (incl. via the fault seam), the serializer STILL emits `code`/`message`/`remediation_hint` (where available) + exit 1, NEVER drops an existing field, and NEVER turns a deny into an allow. Existing fields render independently of the new ones.
- [MUST] MUST COORDINATE the Tauri/MGMT desktop surface — `but-api/src/json.rs` `Error` (json.rs:258, emits `code`+`message`) — with Sprint 06a `MGMT-IPC-002` (which adds `remediation_hint` there): EITHER co-land the steering fields in `json.rs` Error OR record a TRACKING note/task documenting the desktop steering-field gap + its MGMT-IPC-002 dependency (an explicit, verifiable recorded decision — never a silent gap; 03 §7 / D6 / SA-8).
- [MUST] MUST drive every behavioral AC through the REAL `but` CLI against the real governed gix fixture, asserting on the structured JSON denial on stderr + exit 1 (the assertion style of governed_loop.rs, not insta snapshots).
- [NEVER] NEVER let a serialization fault drop an existing field or turn a deny into an allow (fail-closed at serialization — the cardinal regression this task guards).
- [NEVER] NEVER ship the `BUT_STEER_FORCE_SERIALIZATION_FAULT` fault seam in a release build — it MUST be `#[cfg(debug_assertions)]`/`cfg(test)`/feature-gated and compiled out of release; it is a test-only fault injector, never a production bypass (SA-1/RR-4).
- [NEVER] NEVER omit `governance_cli_error` (perm.rs:118) — an admin-write actor-correctable denial without the steering payload violates the §5 admin-write affordance row + UC-STEER-01 AC-1 uniform shape (RR-2).
- [NEVER] NEVER regress the merge site's existing `remediation_hint`/`unmet` keys, nor the exit-1 behavior of any site.
- [NEVER] NEVER claim the Tauri surface covered if it is deferred — record the deferral as an explicit tracked decision/note in the completion report and a tracking artifact (SA-8).
- [NEVER] NEVER change a denial code, message, or the deny/allow decision.
- [NEVER] NEVER edit any frozen Sprint 01a-06b task file (the Sprint 06a MGMT-IPC-002 coordination is a read-only dependency note unless the freeze has lifted and co-landing is sanctioned).
- [STRICTLY] STRICTLY render the CLI denial JSON through the shared `to_envelope()` (STEER-001) where possible so all FOUR sites converge on one shape rather than hand-rolling four divergent `serde_json::json!` blocks.
- [STRICTLY] STRICTLY add a holdout serialization-fault case (the `BUT_STEER_FORCE_SERIALIZATION_FAULT` seam forced while deriving/serializing the steering payload) proving the denial still emits the legacy fields + exit 1 (T-STEER-027) — this is the load-bearing fail-closed proof and pairs with STEER-009's serialization-fault scenario.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: commit_gate_cli_error emits the four steering fields + the long-missing remediation_hint, exit 1 unchanged
- [ ] AC-2: review_gate_cli_error emits the four steering fields + remediation_hint, exit 1 unchanged
- [ ] AC-3: merge_gate_cli_error adds the four steering fields while preserving remediation_hint + unmet, exit 1 unchanged
- [ ] AC-4: governance_cli_error (admin-write) emits the four steering fields + remediation_hint + the admin-write affordance row, exit 1 unchanged
- [ ] AC-5: Serialized `class` is a stable enum STRING token branchable without parsing `message`
- [ ] AC-6: Best-effort fail-closed via the BUT_STEER_FORCE_SERIALIZATION_FAULT seam: a forced steering-payload serialization fault still denies with code/message/remediation_hint + exit 1
- [ ] AC-7: The Tauri/N-API json::Error steering-field decision is an explicit, verifiable recorded outcome (co-land or tracked deferral)
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: commit_gate_cli_error emits the four steering fields + the long-missing remediation_hint, exit 1 unchanged [PRIMARY]
  GIVEN: a real commit denial through the `but` CLI commit path
  WHEN:  commit_gate_cli_error serializes the denial to stderr JSON
  THEN:  the JSON carries `class`/`held_permissions`/`authorized_actions`/`do_not` (when Some) PLUS `code`/`message` AND the newly-added `remediation_hint`; exit code 1 unchanged
  TEST_TIER: integration   VERIFICATION_SERVICE: but-cli
  VERIFY: cargo test -p but governed_loop_steer_commit_cli_serializer
  SCENARIO: would fail if the commit serializer still emits only {code,message} (no steering fields, no remediation_hint); the serializer is a static stub returning a fixed envelope; exit code is not 1 | must observe: stderr JSON contains `class`; stderr JSON contains `authorized_actions`; stderr JSON contains `remediation_hint`; exit code == 1 | must NOT observe: a `{"error":{"code":..,"message":..}}` object with only those two keys; exit code 0

AC-2: review_gate_cli_error emits the four steering fields + remediation_hint, exit 1 unchanged
  GIVEN: a real review denial through the `but` CLI review path
  WHEN:  review_gate_cli_error serializes the denial
  THEN:  the JSON carries the four steering fields PLUS code/message AND the newly-added remediation_hint; exit code 1 unchanged
  TEST_TIER: integration   VERIFICATION_SERVICE: but-cli
  VERIFY: cargo test -p but governed_loop_steer_review_cli_serializer
  SCENARIO: would fail if the review serializer still emits only {code,message}; remediation_hint is omitted on the review site; exit code is not 1 | must observe: stderr JSON contains `class`; stderr JSON contains `remediation_hint`; exit code == 1 | must NOT observe: a two-key {code,message}-only review envelope; exit code 0

AC-3: merge_gate_cli_error adds the four steering fields while preserving remediation_hint + unmet, exit 1 unchanged
  GIVEN: a real merge denial (gate.review_required) through the `but` CLI merge path
  WHEN:  merge_gate_cli_error serializes the denial
  THEN:  the JSON carries the four steering fields PLUS the existing code/message/remediation_hint/unmet keys (unmet asserted here); exit code 1 unchanged
  TEST_TIER: integration   VERIFICATION_SERVICE: but-cli
  VERIFY: cargo test -p but governed_loop_steer_merge_cli_serializer
  SCENARIO: would fail if the merge serializer loses `unmet` when the steering fields are added; remediation_hint is dropped; the steering fields are absent on the merge site | must observe: stderr JSON contains `class`; stderr JSON contains `unmet` as an array; stderr JSON contains `remediation_hint`; exit code == 1 | must NOT observe: a merge envelope missing `unmet`; a merge envelope missing `remediation_hint`; exit code 0

AC-4: governance_cli_error (admin-write) emits the four steering fields + remediation_hint + the admin-write affordance row, exit 1 unchanged
  GIVEN: a real admin-write denial through the `but perm grant` CLI path by a resolved principal lacking `administration:write` (actor_correctable per §2)
  WHEN:  governance_cli_error (perm.rs:118) serializes the AdminWriteGateError denial
  THEN:  the JSON carries `class`=`"actor_correctable"`/`held_permissions`/`authorized_actions`/`do_not` (when Some) PLUS `code`/`message` AND the newly-added `remediation_hint`; `authorized_actions` surfaces the §5 admin-write affordance row (read/inspect, request-config-change, discovery); exit code 1 unchanged
  TEST_TIER: integration   VERIFICATION_SERVICE: but-cli
  VERIFY: cargo test -p but governed_loop_steer_governance_cli_serializer
  SCENARIO: would fail if governance_cli_error still emits only `{code,message}` (the pre-STEER perm.rs:118 flatten — no steering payload for admin-write); the admin-write denial carries an empty authorized_actions (no admin-write affordance row surfaced); exit code is not 1 | must observe: stderr JSON contains `"class":"actor_correctable"`; stderr JSON contains `authorized_actions`; stderr JSON contains `remediation_hint`; exit code == 1 | must NOT observe: a two-key `{"error":{"code":..,"message":..}}` governance envelope (the pre-STEER perm.rs:118 flatten); an empty `authorized_actions` on the admin-write actor-correctable denial

AC-5: Serialized `class` is a stable enum STRING token branchable without parsing `message`
  GIVEN: two real CLI denials of different classes — an actor_correctable commit denial and an operator_required config.invalid denial — serialized to stderr JSON
  WHEN:  an orchestrator reads the serialized denial envelope's `class` field
  THEN:  `class` is exactly the stable token `"actor_correctable"` (first) or `"operator_required"` (second), a JSON string an orchestrator can branch on directly WITHOUT parsing `message` (RR-3 / T-STEER-016 / UC-STEER-03 AC-5)
  TEST_TIER: integration   VERIFICATION_SERVICE: but-cli
  VERIFY: cargo test -p but governed_loop_steer_class_token_branchable
  SCENARIO: would fail if `class` serializes as a nested object / Debug form (`ActorCorrectable`) rather than the snake-case token; branching requires substring-matching `message` because `class` is absent on the serialized envelope; the two denials emit the same `class` token (the dimension collapsed) | must observe: the commit denial's `class` == `"actor_correctable"` (a JSON string); the config.invalid denial's `class` == `"operator_required"` (a JSON string) | must NOT observe: a `class` rendered as `{"ActorCorrectable":...}` or `"ActorCorrectable"` (Debug/PascalCase — not the stable token); no `class` key on the serialized envelope (an empty/absent class field forcing a `message` substring branch)

AC-6: Best-effort fail-closed via the BUT_STEER_FORCE_SERIALIZATION_FAULT seam: a forced steering-payload serialization fault still denies with code/message/remediation_hint + exit 1
  GIVEN: the TEST-ONLY `BUT_STEER_FORCE_SERIALIZATION_FAULT` fault seam (debug-only) forcing a fault while deriving/serializing the steering payload
  WHEN:  a real denial is serialized through that faulting path with `BUT_STEER_FORCE_SERIALIZATION_FAULT=1`
  THEN:  the action is STILL denied with `code`/`message`/`remediation_hint` + exit 1, no existing field is dropped, and the deny is never turned into an allow
  TEST_TIER: integration   VERIFICATION_SERVICE: but-cli
  VERIFY: cargo test -p but steer_cli_serialization_fault_fail_closed
  SCENARIO: would fail if a serialization fault drops code/message/remediation_hint; a fault turns the deny into a success (exit 0); a fault panics (a no-op fallback) instead of degrading to the legacy envelope; the BUT_STEER_FORCE_SERIALIZATION_FAULT seam is compiled into a release build (production bypass) | must observe: stderr JSON still contains `code`; stderr JSON still contains `message`; stderr JSON still contains `remediation_hint`; exit code == 1 | must NOT observe: exit code 0 (deny→allow); a missing `code`/`message` field; a panic/abort instead of a denial envelope

AC-7: The Tauri/N-API json::Error steering-field decision is an explicit, verifiable recorded outcome (co-land or tracked deferral)
  GIVEN: the Tauri/MGMT desktop surface rides `but-api/src/json.rs` Error (json.rs:258, emits code+message) and Sprint 06a MGMT-IPC-002 owns its remediation_hint
  WHEN:  the STEER serialization work completes and the desktop steering-field coverage is assessed
  THEN:  EITHER the four steering fields co-land in `but-api/src/json.rs` Error (a test asserts json::Error carries `class`) OR a tracking note/task is recorded in the completion report + a tracked artifact documenting the desktop steering-field gap and its MGMT-IPC-002 dependency — an explicit recorded decision, not a silent gap (SA-8)
  TEST_TIER: integration   VERIFICATION_SERVICE: but-api
  VERIFY: cargo test -p but-api steer_json_error_decision_recorded
  SCENARIO: would fail if the Tauri json::Error surface is left unchanged with no recorded decision (a silent steering-field gap); the completion report claims the desktop surface covered while json::Error still emits only `{code,message}`; the deferral exists only as inline guardrail prose with no verifiable test/tracked artifact | must observe: either json::Error JSON contains `"class"` (co-landed) OR a tracked deferral note referencing `MGMT-IPC-002` is present; the recorded decision is asserted by the test (exit `0`) | must NOT observe: json::Error silently unchanged with NO recorded decision (a silent desktop steering gap); a completion claim of desktop coverage with json::Error still a two-key `{code,message}` object

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, happy_path): The commit CLI denial JSON contains `class`, `authorized_actions`, and `remediation_hint`
    VERIFY: cargo test -p but governed_loop_steer_commit_cli_serializer
- TC-2 (-> AC-1, edge_case): The commit CLI denial exits with code 1
    VERIFY: cargo test -p but governed_loop_steer_commit_cli_serializer
- TC-3 (-> AC-2, happy_path): The review CLI denial JSON contains the four steering fields and `remediation_hint`
    VERIFY: cargo test -p but governed_loop_steer_review_cli_serializer
- TC-4 (-> AC-3, edge_case): The merge CLI denial JSON contains `class` AND retains `unmet` and `remediation_hint`
    VERIFY: cargo test -p but governed_loop_steer_merge_cli_serializer
- TC-5 (-> AC-4, happy_path): The governance (admin-write) CLI denial JSON contains `class`=`actor_correctable`, `authorized_actions`, and `remediation_hint`
    VERIFY: cargo test -p but governed_loop_steer_governance_cli_serializer
- TC-6 (-> AC-4, edge_case): The governance (admin-write) CLI denial surfaces the §5 admin-write affordance row (read/inspect, request-config-change, discovery) and exits 1
    VERIFY: cargo test -p but governed_loop_steer_governance_cli_serializer
- TC-7 (-> AC-5, happy_path): An actor_correctable denial's serialized `class` is the JSON string `"actor_correctable"`, branchable without parsing `message`
    VERIFY: cargo test -p but governed_loop_steer_class_token_branchable
- TC-8 (-> AC-5, edge_case): An operator_required (config.invalid) denial's serialized `class` is the JSON string `"operator_required"` (the class dimension is not collapsed)
    VERIFY: cargo test -p but governed_loop_steer_class_token_branchable
- TC-9 (-> AC-6, error_case): With BUT_STEER_FORCE_SERIALIZATION_FAULT set, a forced steering-payload serialization fault still emits code/message/remediation_hint + exit 1
    VERIFY: cargo test -p but steer_cli_serialization_fault_fail_closed
- TC-10 (-> AC-6, error_case): A forced serialization fault never produces exit code 0 (deny is never turned into allow)
    VERIFY: cargo test -p but steer_cli_serialization_fault_fail_closed
- TC-11 (-> AC-7, edge_case): The Tauri json::Error steering-field decision is recorded (json::Error carries `class`, OR a tracked MGMT-IPC-002 deferral note exists and is asserted)
    VERIFY: cargo test -p but-api steer_json_error_decision_recorded

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-STEER-01
provides: commit_gate_cli_error emitting the 4 steering fields + the long-missing remediation_hint; review_gate_cli_error emitting the 4 steering fields (its field source is ForgeGateError's copied fields from STEER-001) + the long-missing remediation_hint; merge_gate_cli_error emitting the 4 steering fields (already has remediation_hint + unmet); governance_cli_error (perm.rs:118) emitting the 4 steering fields for admin-write AdminWriteGateError denials (the FOURTH CLI serializer — RR-2); a TEST-ONLY serialization-fault-injection seam on the to_envelope()/serializer path, gated to debug/test builds only, activated by the env var `BUT_STEER_FORCE_SERIALIZATION_FAULT` (consumed by STEER-009) — NEVER present in release builds (SA-1/RR-4); the serialized denial envelope's `class` rendered as a stable enum STRING token (`actor_correctable`/`operator_required`) branchable without parsing `message` (RR-3); best-effort fail-closed serialization (a derivation/serialization fault still emits code/message/remediation_hint + exit 1); an explicitly RECORDED decision for the Tauri json::Error surface — co-land via MGMT-IPC-002 or a tracked deferral note documenting the desktop steering-field gap + its MGMT-IPC-002 dependency (SA-8)
consumes: but_api::commit::create::gate::classify_error -> CommitGateError (commit/gate.rs:81, fields from STEER-001/004); but_api::legacy::forge::classify_error -> ForgeGateError (forge.rs:24, NOW carrying the four copied fields from STEER-001 RR-1); but_api::legacy::merge_gate::classify_error -> MergeGateError (merge_gate.rs:113); but_api::legacy::config_mutate::classify_error -> AdminWriteGateError (config_mutate.rs:31, NOW carrying the four copied fields from STEER-001 RR-2); to_envelope() (STEER-001); commit_gate_cli_error (commit2.rs:679); review_gate_cli_error (forge/review.rs:89); merge_gate_cli_error (forge/review.rs:246); governance_cli_error (perm.rs:118 — serializes AdminWriteGateError for `but perm grant/revoke`); but-api/src/json.rs Error (json.rs:258)
boundary_contracts:
  - CAP-STEER-01 (serialization site): all FOUR hand-rolled CLI serializers — commit_gate_cli_error, review_gate_cli_error, merge_gate_cli_error, governance_cli_error (admin-write) — emit the four steering fields alongside their existing keys; existing keys + exit 1 are preserved relative to EACH site's current output (commit/review/governance sites additionally gain the long-missing remediation_hint — an improvement); the serialized `class` is a stable enum STRING token (`actor_correctable`/`operator_required`) branchable without reading `message`; a TEST-ONLY fault seam (`BUT_STEER_FORCE_SERIALIZATION_FAULT`, debug-only) proves a fault deriving OR serializing the steering payload still emits code/message/remediation_hint + exit 1 and never turns deny->allow (invariant §9.5); the Tauri json::Error surface decision is explicitly recorded (co-land via MGMT-IPC-002 or a tracked deferral note).

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but/src/command/legacy/commit2.rs (MODIFY — commit_gate_cli_error emits the 4 steering fields + remediation_hint via to_envelope)
  - crates/but/src/command/legacy/forge/review.rs (MODIFY — review_gate_cli_error + merge_gate_cli_error emit the 4 steering fields)
  - crates/but/src/command/perm.rs (MODIFY — governance_cli_error :118 emits the 4 steering fields + remediation_hint for admin-write AdminWriteGateError; RR-2)
  - crates/but-authz/src/lib.rs or but-api serializer module (MODIFY — add the `#[cfg(debug_assertions)]`/`cfg(test)`-gated BUT_STEER_FORCE_SERIALIZATION_FAULT fault seam on the to_envelope()/serializer path; SA-1/RR-4)
  - crates/but-api/src/json.rs (MODIFY — co-land steering fields in Error IFF MGMT-IPC-002 is being co-landed; otherwise leave and record the explicit deferral note; SA-8)
  - crates/but/tests/but/command/governed_loop.rs (MODIFY — add the four serializer-shape cases + the class-token case + the serialization-fault case; do NOT weaken existing assertions)
  - crates/but/tests/but/command/steer_cli_serializers.rs (NEW — if a separate module is preferred)
  - crates/but-api/tests/steer_json_error_decision.rs (NEW — asserts the recorded Tauri json::Error decision; SA-8)
writeProhibited:
  - the deny/allow decision or exit-1 behavior — NEVER weaken (a serialization fault must still deny)
  - the merge site's existing remediation_hint/unmet keys — NEVER drop
  - the BUT_STEER_FORCE_SERIALIZATION_FAULT seam in release builds — NEVER ship un-gated (debug/test-only)
  - the Tauri json::Error tests — NEVER claim covered if deferred (record the explicit decision)
  - any denial code/message — NEVER change
  - .spec/prds/governance/tasks/sprint-0[1-6]* — frozen task files (the MGMT-IPC-002 coordination is read-only unless co-landing is sanctioned)

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
  - crates/but/src/command/legacy/commit2.rs (lines 679-694): commit_gate_cli_error :679 — emits `{code,message}` today via serde_json::json!; add the 4 steering fields + remediation_hint (render via to_envelope).
  - crates/but/src/command/legacy/forge/review.rs (lines 89-104, 246-262): review_gate_cli_error :89 ({code,message}) and merge_gate_cli_error :246 ({code,message,remediation_hint,unmet}) — the two forge serializers; merge already has remediation_hint+unmet.
  - crates/but-api/src/commit/gate.rs (lines 80-94): classify_error → CommitGateError :81 — the carrier the commit CLI serializer reads (now carrying the 4 fields after STEER-001/004).
  - crates/but-api/src/legacy/forge.rs (lines 23-37): ForgeGateError + classify_error :24 — the carrier the review serializer reads; AFTER STEER-001 RR-1 it carries the four copied steering fields (the field source for review_gate_cli_error).
  - crates/but/src/command/perm.rs (lines 118-133): governance_cli_error :118 — the FOURTH CLI serializer; emits `{code,message}` today from AdminWriteGateError for `but perm grant/revoke`; add the 4 steering fields + remediation_hint via to_envelope (RR-2).
  - crates/but-api/src/legacy/merge_gate.rs (lines 112-124): classify_error → MergeGateError :113 — the carrier the merge serializer reads.
  - crates/but-api/src/json.rs (lines 174-276): the Tauri/MGMT json::Error serializer :258 (emits code+message) — the surface to coordinate with Sprint 06a MGMT-IPC-002 (co-land or defer).
  - crates/but/tests/but/command/governed_loop.rs (lines 365-498): assert_denial / parse_cli_error_envelope_opt / json_object_from_line — the stderr-JSON reader to extend for the steering-field assertions.

--------------------------------------------------------------------------------
CODE PATTERN
--------------------------------------------------------------------------------
pattern: Render the classified carrier through the shared to_envelope() into a serde_json::Value, wrapping in `{"error": ...}` on stderr with exit 1, at all FOUR sites (commit/review/merge/governance); `class` serializes as a snake-case enum string token (`serde(rename_all="snake_case")` or an explicit DenialClass Serialize); a `#[cfg(debug_assertions)]`/`cfg(test)`-gated fault hook keyed on `BUT_STEER_FORCE_SERIALIZATION_FAULT` forces a fault; a best-effort path where, if to_envelope/serialization faults, the serializer falls back to a minimal `{code, message, remediation_hint}` envelope (still exit 1) rather than panicking or succeeding.
pattern_source: crates/but/src/command/legacy/forge/review.rs:246 (merge_gate_cli_error already emits {code,message,remediation_hint,unmet} via serde_json::json! — extend this shape to all four sites); crates/but/src/command/perm.rs:118 (governance_cli_error — the fourth site, currently a two-key flatten); crates/but/tests/but/command/governed_loop.rs:474-491 (parse_cli_error_envelope_opt reads the stderr envelope).
anti_pattern: Four divergent hand-rolled json! blocks that drift; a serializer that `unwrap()`s the steering payload (a fault panics or aborts instead of degrading); dropping `unmet` on the merge site when adding the steering fields; shipping BUT_STEER_FORCE_SERIALIZATION_FAULT un-gated in release; rendering `class` as a Debug/PascalCase form an orchestrator can't branch on; silently leaving the Tauri surface ungapped without recording the decision.
references: 03-technical-requirements-delta.md §1 (serialization sites) + §2 (admin-write actor_correctable row) + §5 (admin-write affordance row) + §7 (caller coverage); 05-delta-replan.md D6; 02-uc-steer.md UC-STEER-01 AC-1/AC-2 + UC-STEER-03 AC-5 + UC-STEER-06 AC-4; invariant §9.5 (best-effort fail-closed)
interaction_notes:
  - Reads the carriers wired by STEER-001/004 (incl. ForgeGateError/AdminWriteGateError's copied fields from STEER-001 RR-1/RR-2) and renders via to_envelope() (STEER-001); the BUT_STEER_FORCE_SERIALIZATION_FAULT seam is consumed by STEER-009's serialization-fault scenario; coordinates with Sprint 06a MGMT-IPC-002 for the desktop surface.

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
depends_on: STEER-004
blocks: STEER-009

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
      "description": "GIVEN a CLI commit denial WHEN commit_gate_cli_error serializes THEN the four steering fields + the newly-added remediation_hint appear and exit 1 is unchanged",
      "verify": "cargo test -p but governed_loop_steer_commit_cli_serializer"
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "GIVEN a CLI review denial WHEN review_gate_cli_error serializes THEN the four steering fields + remediation_hint appear and exit 1 is unchanged",
      "verify": "cargo test -p but governed_loop_steer_review_cli_serializer"
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "GIVEN a CLI merge denial WHEN merge_gate_cli_error serializes THEN the four steering fields appear and remediation_hint + unmet are retained, exit 1 unchanged",
      "verify": "cargo test -p but governed_loop_steer_merge_cli_serializer"
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "description": "GIVEN a forced steering-payload serialization fault WHEN a denial is serialized THEN it still denies with code/message/remediation_hint + exit 1 and never flips deny\u2192allow",
      "verify": "cargo test -p but governed_loop_steer_governance_cli_serializer"
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "description": "GIVEN two real CLI denials of different classes \u2014 an actor_correctable commit denial and an operator_required config.invalid denial \u2014 serialized to stderr JSON WHEN an orchestrator reads the serialized denial envelope's `class` field THEN `class` is exactly the stable token `\"actor_correctable\"` (first) or `\"operator_required\"` (second), a JSON string an orchestrator can branch on directly WITHOUT parsing `message` (RR-3 / T-STEER-016 / UC-STEER-03 AC-5)",
      "verify": "cargo test -p but governed_loop_steer_class_token_branchable"
    },
    {
      "id": "AC-6",
      "type": "acceptance_criterion",
      "description": "GIVEN the TEST-ONLY `BUT_STEER_FORCE_SERIALIZATION_FAULT` fault seam (debug-only) forcing a fault while deriving/serializing the steering payload WHEN a real denial is serialized through that faulting path with `BUT_STEER_FORCE_SERIALIZATION_FAULT=1` THEN the action is STILL denied with `code`/`message`/`remediation_hint` + exit 1, no existing field is dropped, and the deny is never turned into an allow",
      "verify": "cargo test -p but steer_cli_serialization_fault_fail_closed"
    },
    {
      "id": "AC-7",
      "type": "acceptance_criterion",
      "description": "GIVEN the Tauri/MGMT desktop surface rides `but-api/src/json.rs` Error (json.rs:258, emits code+message) and Sprint 06a MGMT-IPC-002 owns its remediation_hint WHEN the STEER serialization work completes and the desktop steering-field coverage is assessed THEN EITHER the four steering fields co-land in `but-api/src/json.rs` Error (a test asserts json::Error carries `class`) OR a tracking note/task is recorded in the completion report + a tracked artifact documenting the desktop steering-field gap and its MGMT-IPC-002 dependency \u2014 an explicit recorded decision, not a silent gap (SA-8)",
      "verify": "cargo test -p but-api steer_json_error_decision_recorded"
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "commit CLI JSON has class/authorized_actions/remediation_hint",
      "maps_to_ac": "AC-1",
      "verify": "cargo test -p but governed_loop_steer_commit_cli_serializer"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "commit CLI denial exit 1",
      "maps_to_ac": "AC-1",
      "verify": "cargo test -p but governed_loop_steer_commit_cli_serializer"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "review CLI JSON has steering fields + remediation_hint",
      "maps_to_ac": "AC-2",
      "verify": "cargo test -p but governed_loop_steer_review_cli_serializer"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "merge CLI JSON has class + retains unmet + remediation_hint",
      "maps_to_ac": "AC-3",
      "verify": "cargo test -p but governed_loop_steer_merge_cli_serializer"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "serialization fault still emits legacy fields + exit 1",
      "maps_to_ac": "AC-4",
      "verify": "cargo test -p but governed_loop_steer_governance_cli_serializer"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "description": "serialization fault never exits 0",
      "maps_to_ac": "AC-4",
      "verify": "cargo test -p but governed_loop_steer_governance_cli_serializer"
    },
    {
      "id": "TC-7",
      "type": "test_criterion",
      "description": "An actor_correctable denial's serialized `class` is the JSON string `\"actor_correctable\"`, branchable without parsing `message`",
      "maps_to_ac": "AC-5",
      "verify": "cargo test -p but governed_loop_steer_class_token_branchable"
    },
    {
      "id": "TC-8",
      "type": "test_criterion",
      "description": "An operator_required (config.invalid) denial's serialized `class` is the JSON string `\"operator_required\"` (the class dimension is not collapsed)",
      "maps_to_ac": "AC-5",
      "verify": "cargo test -p but governed_loop_steer_class_token_branchable"
    },
    {
      "id": "TC-9",
      "type": "test_criterion",
      "description": "With BUT_STEER_FORCE_SERIALIZATION_FAULT set, a forced steering-payload serialization fault still emits code/message/remediation_hint + exit 1",
      "maps_to_ac": "AC-6",
      "verify": "cargo test -p but steer_cli_serialization_fault_fail_closed"
    },
    {
      "id": "TC-10",
      "type": "test_criterion",
      "description": "A forced serialization fault never produces exit code 0 (deny is never turned into allow)",
      "maps_to_ac": "AC-6",
      "verify": "cargo test -p but steer_cli_serialization_fault_fail_closed"
    },
    {
      "id": "TC-11",
      "type": "test_criterion",
      "description": "The Tauri json::Error steering-field decision is recorded (json::Error carries `class`, OR a tracked MGMT-IPC-002 deferral note exists and is asserted)",
      "maps_to_ac": "AC-7",
      "verify": "cargo test -p but-api steer_json_error_decision_recorded"
    }
  ]
}
-->
