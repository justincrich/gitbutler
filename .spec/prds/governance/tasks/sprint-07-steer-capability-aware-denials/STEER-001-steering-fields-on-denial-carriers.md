# STEER-001: Steering fields (class/held_permissions/authorized_actions/do_not) on ALL SIX denial carriers (Denial, MergeGateError, ConfigError, CommitGateError, ForgeGateError, AdminWriteGateError) + DenialClass/AuthorizedAction types + derives + to_envelope() + Authority serialization (stable `:` token, stable lexical order)

## What this does

Add the four additive steering fields (`class: DenialClass`, `held_permissions: Vec<Authority>`, `authorized_actions: Vec<AuthorizedAction>`, `do_not: Option<&'static str>`) to ALL SIX denial carriers (Denial, MergeGateError, ConfigError [class+do_not only], CommitGateError, ForgeGateError, AdminWriteGateError), have ForgeGateError's and AdminWriteGateError's `classify_error` copy the four fields off the underlying `Denial`/`ConfigError`, introduce the `DenialClass`/`AuthorizedAction` types in but-authz with the derives that keep every carrier compiling, decide and implement `Authority` serialization to its stable `:` token in stable lexical order, and provide a shared `to_envelope()` that renders `Denial`+`MergeGateError` to one uniform JSON superset — preserving every back-compat key.

## Why

Sprint 07 (STEER — Capability-Aware Denials) · PRD UC-STEER-01 · Capability CAP-STEER-01. Every one of the six carriers carries the four fields and serializes to a JSON object containing `code`/`message`/`remediation_hint` (and `unmet` on `MergeGateError`) PLUS `class`/`held_permissions`/`authorized_actions` (and `do_not` when present); ForgeGateError/AdminWriteGateError's classify_error copies the four fields off the underlying Denial/ConfigError; `held_permissions` serializes as a stably-ordered array of `:`-token strings; `do_not` is omitted when None; a consumer reading only the legacy keys sees no regression; DryRun emits the full payload while persisting nothing; `cargo test -p but-authz` + `-p but-api` green, clippy clean.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-authz steer_carriers_serialize_additive_superset && cargo test -p but-api steer_merge_envelope_uniform_shape` (All four carriers serialize the additive superset alongside legacy keys). Full gate set in the spec below.

## Scope

- crates/but-authz/src/denial.rs (MODIFY — add 4 fields + DenialClass + AuthorizedAction + to_envelope)
- crates/but-authz/src/authority.rs (MODIFY — add #[derive(Serialize)] to Authority if that path is chosen)
- crates/but-authz/src/config.rs (MODIFY — add class + do_not to ConfigError)
- crates/but-authz/src/lib.rs (MODIFY — re-export the new types/fn)
- crates/but-api/src/legacy/merge_gate.rs (MODIFY — add 4 fields to MergeGateError + config_invalid populates class+do_not)
- crates/but-api/src/commit/gate.rs (MODIFY — add 4 fields to CommitGateError)
- crates/but-authz/tests/steer_carriers.rs (NEW — serialization shape proofs)
- crates/but-api/tests/steer_envelope.rs (NEW — to_envelope + backcompat reader)
- crates/but-api/src/legacy/forge.rs (MODIFY — add 4 fields to ForgeGateError + classify_error copies them off the underlying Denial/ConfigError)
- crates/but-api/src/legacy/config_mutate.rs (MODIFY — add 4 fields to AdminWriteGateError + classify_error copies them off the underlying Denial/ConfigError)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: STEER-001 - Steering fields (class/held_permissions/authorized_actions/do_not) on ALL SIX denial carriers (Denial, MergeGateError, ConfigError, CommitGateError, ForgeGateError, AdminWriteGateError) + DenialClass/AuthorizedAction types + derives + to_envelope() + Authority serialization (stable `:` token, stable lexical order)
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Completed
PRIORITY:   P0
EFFORT:     M  (255 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-STEER-01
CAPABILITIES: CAP-STEER-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-authz steer_carriers_serialize_additive_superset && cargo test -p but-api steer_merge_envelope_uniform_shape   |   cargo test -p but-authz steer_legacy_keys_unchanged && cargo test -p but-api steer_carrier_backcompat_reader   |   cargo test -p but-authz steer_held_permissions_stable_lexical_token_order   |   cargo test -p but-authz steer_do_not_skip_when_none   |   cargo test -p but-api steer_forge_and_admin_carriers_copy_steering_fields
  check: cargo check -p but-authz --all-targets && cargo check -p but-api --all-targets && cargo test -p but-authz steer_types_derive_eq_clone
  lint:  cargo clippy -p but-authz -p but-api --all-targets   |   cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Every one of the six carriers carries the four fields and serializes to a JSON object containing `code`/`message`/`remediation_hint` (and `unmet` on `MergeGateError`) PLUS `class`/`held_permissions`/`authorized_actions` (and `do_not` when present); ForgeGateError/AdminWriteGateError's classify_error copies the four fields off the underlying Denial/ConfigError; `held_permissions` serializes as a stably-ordered array of `:`-token strings; `do_not` is omitted when None; a consumer reading only the legacy keys sees no regression; DryRun emits the full payload while persisting nothing; `cargo test -p but-authz` + `-p but-api` green, clippy clean.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST add the four fields to ALL FOUR carriers — `Denial` (denial.rs:13), `MergeGateError` (merge_gate.rs:19), `ConfigError` (config.rs:241, class+do_not ONLY — no held/menu), and `CommitGateError` (commit/gate.rs:9) — and add `DenialClass{ActorCorrectable,OperatorRequired}` + `AuthorizedAction{command:&'static str, effect:&'static str}` to but-authz, each deriving `Debug,Clone,PartialEq,Eq` so every carrier's existing derives still compile (M4/L2-fix, 03 §1).
- [MUST] MUST decide `Authority` serialization explicitly: either add `#[derive(Serialize)]` to `Authority` (authority.rs:10) emitting its stable `:` token MATCHING `Authority::name()` (authority.rs:94), OR map `held.iter().map(Authority::name)` at each serializer. `MergeGateError` already derives `Serialize` (merge_gate.rs:19), so adding a `Vec<Authority>` field forces this choice to compile. State the decision in the completion report.
- [MUST] MUST emit `held_permissions` in a STABLE lexical order (L3-fix) so set/sorted equality assertions are not order-flaky; `AuthoritySet::iter()` (authority.rs:320) already yields deterministic `BTreeSet` order — preserve it.
- [MUST] MUST serialize `do_not: Option<&'static str>` with `#[serde(skip_serializing_if = "Option::is_none")]` (03 §1).
- [MUST] MUST add a shared `to_envelope()` (a JSON-shape renderer) that renders `Denial` + `MergeGateError` to ONE uniform JSON object (D4) — the canonical superset shape consumed at the CLI serializers (STEER-005).
- [MUST] MUST add the four steering fields to `ForgeGateError` (forge.rs:16) AND `AdminWriteGateError` (config_mutate.rs:6) — both currently flatten the underlying `but_authz::Denial`/`ConfigError` to `{code,message}` (forge.rs:24, config_mutate.rs:31) — and have BOTH `classify_error` functions COPY `class`/`held_permissions`/`authorized_actions`/`do_not` off the underlying `Denial` (and `class`+`do_not` off `ConfigError`). Without this the review CLI serializer (review_gate_cli_error reads ForgeGateError) and the governance CLI serializer (governance_cli_error reads AdminWriteGateError) have NO field source (RR-1/RR-2). Both already derive `Serialize,PartialEq,Eq` so the new fields' types must derive the same.
- [NEVER] NEVER change a single gate deny/allow DECISION, denial `code`, exit code, or fail-closed posture — this task is pure type/serialization plumbing; the field VALUES are wired by STEER-004.
- [NEVER] NEVER add `unmet` to `Denial` — `unmet: Vec<String>` is a `MergeGateError`-only field (merge_gate.rs:28); the grounding correction in 03 §0 is explicit.
- [NEVER] NEVER remove or weaken `Denial`/`MergeGateError`/`CommitGateError`'s existing `Debug,Clone,PartialEq,Eq` derives or the existing `code`/`message`/`remediation_hint`/`unmet` fields.
- [NEVER] NEVER edit any frozen Sprint 01a–06b task file under .spec/prds/governance/tasks/sprint-0[1-6]*.
- [NEVER] NEVER drop ForgeGateError's/AdminWriteGateError's existing `code`/`message` keys or their `Serialize,Debug,Clone,PartialEq,Eq` derives when adding the four fields.
- [STRICTLY] STRICTLY site the new types (`DenialClass`, `AuthorizedAction`) and `to_envelope()` in `but-authz` (L4) so `authorize`/the menu/the gates can use them with NO `but-authz → but-api` dependency cycle (RULES.md).

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [x] AC-1 [PRIMARY]: All four carriers serialize the additive superset alongside legacy keys
- [x] AC-2: Back-compat: legacy keys + exit code unchanged relative to each carrier's current output
- [x] AC-3: held_permissions serializes as a stably-ordered array of `:` tokens matching Authority::name()
- [x] AC-4: do_not is omitted when None (skip_serializing_if)
- [x] AC-5: DenialClass/AuthorizedAction derives keep all carriers' derives intact (compile + value-equality)
- [x] AC-6: ForgeGateError and AdminWriteGateError carry the four steering fields (classify_error copies them off the underlying Denial/ConfigError)
- [x] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: All four carriers serialize the additive superset alongside legacy keys [PRIMARY]
  GIVEN: the four denial carriers (Denial, MergeGateError, ConfigError, CommitGateError) with the new fields and a Denial/MergeGateError rendered through to_envelope()
  WHEN:  a representative denial of each carrier is serialized to JSON via to_envelope() / serde_json
  THEN:  each JSON object contains `class`, `held_permissions`, `authorized_actions`, and `do_not` (when Some) ALONGSIDE the existing `code`/`message`/`remediation_hint` (and `unmet` on MergeGateError); ConfigError carries `class`+`do_not` but no `held_permissions`/`authorized_actions`
  TEST_TIER: integration   VERIFICATION_SERVICE: but-authz
  VERIFY: cargo test -p but-authz steer_carriers_serialize_additive_superset && cargo test -p but-api steer_merge_envelope_uniform_shape
  SCENARIO: would fail if a carrier still has only three/four legacy fields (empty struct addition stub); to_envelope is a no-op returning the legacy shape; DenialClass does not derive PartialEq/Eq so a static placeholder compiles but the field is absent | must observe: the key `"class"`; the key `"held_permissions"`; the key `"authorized_actions"`; the key `"code"` with value `"perm.denied"`; the key `"message"`; the key `"remediation_hint"`; the key `"class"`; the key `"unmet"` with value `["no_approval"]`; the key `"code"` with value `"gate.review_required"`; the key `"remediation_hint"` | must NOT observe: an empty/three-key-only JSON object containing only `code`/`message`/`remediation_hint` (the pre-STEER three-key shape); an `unmet` key on the Denial-sourced envelope; a JSON object missing `class` (none present — an empty steering shape); a JSON object missing `unmet`

AC-2: Back-compat: legacy keys + exit code unchanged relative to each carrier's current output
  GIVEN: a consumer reading only `code`/`message`/`remediation_hint` (plus `unmet` on merge denials)
  WHEN:  it parses the serialized output of each carrier after the field additions
  THEN:  every legacy key it reads is present and value-identical to the pre-STEER carrier output, and exit code 1 is unchanged; the additive fields do not displace or rename any legacy key
  TEST_TIER: integration   VERIFICATION_SERVICE: but-authz
  VERIFY: cargo test -p but-authz steer_legacy_keys_unchanged && cargo test -p but-api steer_carrier_backcompat_reader
  SCENARIO: would fail if a field addition renames `remediation_hint` (mock/static rewrite); serde flatten collapses a legacy key; MergeGateError loses `unmet` when the new Vec<Authority> field is added | must observe: `code` == `"gate.review_required"`; `remediation_hint` non-empty string; `unmet` present as an array | must NOT observe: a missing `remediation_hint` key (none present — an empty value); a missing `unmet` key; a parse error on the legacy-key reader

AC-3: held_permissions serializes as a stably-ordered array of `:` tokens matching Authority::name()
  GIVEN: a Denial whose held_permissions = AuthoritySet{comments:write, reviews:write}
  WHEN:  the denial is serialized to JSON
  THEN:  `held_permissions` is the JSON array `["comments:write","reviews:write"]` (sorted lexical / BTreeSet order), each element exactly the `Authority::name()` `:` token, deterministic across repeated serializations
  TEST_TIER: integration   VERIFICATION_SERVICE: but-authz
  VERIFY: cargo test -p but-authz steer_held_permissions_stable_lexical_token_order
  SCENARIO: would fail if held_permissions serializes as Debug `[ReviewsWrite, CommentsWrite]` (no Serialize chosen); order is insertion-order / nondeterministic; tokens use a different (wrong constant) casing than Authority::name() | must observe: `held_permissions` == `["comments:write","reviews:write"]`; `identical` bytes (byte-for-byte `==`) on both serializations | must NOT observe: `["reviews:write","comments:write"]` (insertion order, no sorting applied); `ReviewsWrite` / Debug-form tokens

AC-4: do_not is omitted when None (skip_serializing_if)
  GIVEN: a Denial with do_not = None and another with do_not = Some("...")
  WHEN:  both are serialized to JSON
  THEN:  the None case emits NO `do_not` key at all (not `null`); the Some case emits `do_not` as the literal string
  TEST_TIER: integration   VERIFICATION_SERVICE: but-authz
  VERIFY: cargo test -p but-authz steer_do_not_skip_when_none
  SCENARIO: would fail if do_not always serializes as `null` (missing skip_serializing_if); do_not always serializes as `""` (static empty stub) | must observe: the Some-case JSON contains `"do_not":"do not retry — this requires an operator"` | must NOT observe: a `"do_not":null` substring in the None-case JSON; a `"do_not"` key at all in the None-case JSON

AC-5: DenialClass/AuthorizedAction derives keep all carriers' derives intact (compile + value-equality)
  GIVEN: the new types added to but-authz
  WHEN:  the workspace is type-checked and a Denial round-trips through Clone + PartialEq
  THEN:  DenialClass and AuthorizedAction derive Debug,Clone,PartialEq,Eq; a cloned Denial == the original; cargo check passes for but-authz and but-api (the carriers' existing derives still hold)
  TEST_TIER: unit   VERIFICATION_SERVICE: but-authz
  UNIT_TEST_JUSTIFIED: Pure type/derive verification with zero I/O — the assertion is that DenialClass/AuthorizedAction derive Debug,Clone,PartialEq,Eq and a cloned carrier compares equal; no git/config/authz runtime is exercised.
  VERIFY: cargo check -p but-authz --all-targets && cargo check -p but-api --all-targets && cargo test -p but-authz steer_types_derive_eq_clone
  SCENARIO: would fail if DenialClass lacks PartialEq so Denial can no longer derive PartialEq (compile break); AuthorizedAction lacks Clone so Denial::clone fails; the new types are added without derives and the carrier derive is silently removed | must observe: `assert_eq!` of clone vs original passes (1 passing test); cargo check exits 0 for but-authz and but-api | must NOT observe: a compile error `Denial: the trait PartialEq is not implemented`; 0 tests run

AC-6: ForgeGateError and AdminWriteGateError carry the four steering fields (classify_error copies them off the underlying Denial/ConfigError)
  GIVEN: a forge denial (an underlying Denial with steering fields populated) classified through forge::classify_error, and an admin-write denial classified through config_mutate::classify_error
  WHEN:  each classify_error builds its gate-error carrier and the result is serialized to JSON
  THEN:  ForgeGateError and AdminWriteGateError each carry `class`/`held_permissions`/`authorized_actions`/`do_not` copied off the underlying Denial (and `class`+`do_not` off ConfigError) ALONGSIDE their existing `code`/`message`; neither is left as a two-key `{code,message}` flatten
  TEST_TIER: integration   VERIFICATION_SERVICE: but-api
  VERIFY: cargo test -p but-api steer_forge_and_admin_carriers_copy_steering_fields
  SCENARIO: would fail if ForgeGateError::classify_error still builds a two-key `{code,message}` flatten (no field source for review_gate_cli_error); AdminWriteGateError::classify_error still flattens to `{code,message}` (no field source for governance_cli_error); classify_error returns a static placeholder carrier with empty/default steering fields rather than copying off the underlying Denial | must observe: the ForgeGateError JSON contains the key `"class"`; the ForgeGateError JSON contains the key `"authorized_actions"`; the AdminWriteGateError JSON contains the key `"class"`; the AdminWriteGateError JSON contains the key `"held_permissions"` | must NOT observe: a two-key `{code,message}`-only ForgeGateError JSON object (the pre-STEER flatten — an empty steering shape); a `class` value that does not match the underlying Denial's class

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, happy_path): Serialized Denial JSON contains the key `class`
    VERIFY: cargo test -p but-authz steer_carriers_serialize_additive_superset
- TC-2 (-> AC-1, happy_path): Serialized Denial JSON contains the key `held_permissions`
    VERIFY: cargo test -p but-authz steer_carriers_serialize_additive_superset
- TC-3 (-> AC-1, happy_path): Serialized Denial JSON contains the key `authorized_actions`
    VERIFY: cargo test -p but-authz steer_carriers_serialize_additive_superset
- TC-4 (-> AC-1, edge_case): ConfigError serialization carries `class` and `do_not` but neither `held_permissions` nor `authorized_actions`
    VERIFY: cargo test -p but-authz steer_config_error_class_and_do_not_only
- TC-5 (-> AC-2, edge_case): A legacy-key reader of {code,message,remediation_hint,unmet} parses a post-STEER MergeGateError with no missing key
    VERIFY: cargo test -p but-api steer_carrier_backcompat_reader
- TC-6 (-> AC-3, happy_path): held_permissions serializes to `["comments:write","reviews:write"]` in sorted order
    VERIFY: cargo test -p but-authz steer_held_permissions_stable_lexical_token_order
- TC-7 (-> AC-3, edge_case): Two serializations of the same Denial produce byte-identical held_permissions arrays
    VERIFY: cargo test -p but-authz steer_held_permissions_stable_lexical_token_order
- TC-8 (-> AC-4, edge_case): A Denial with do_not=None emits no `do_not` key
    VERIFY: cargo test -p but-authz steer_do_not_skip_when_none
- TC-9 (-> AC-5, happy_path): cargo check passes for but-authz and but-api with the new derives
    VERIFY: cargo check -p but-authz --all-targets && cargo check -p but-api --all-targets
- TC-10 (-> AC-5, happy_path): A cloned Denial compares equal to the original under PartialEq
    VERIFY: cargo test -p but-authz steer_types_derive_eq_clone
- TC-11 (-> AC-6, happy_path): ForgeGateError serialization contains the key `class` (copied off the underlying Denial)
    VERIFY: cargo test -p but-api steer_forge_and_admin_carriers_copy_steering_fields
- TC-12 (-> AC-6, happy_path): AdminWriteGateError serialization contains the keys `class` and `held_permissions` (copied off the underlying Denial)
    VERIFY: cargo test -p but-api steer_forge_and_admin_carriers_copy_steering_fields

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-STEER-01
provides: Denial::{class,held_permissions,authorized_actions,do_not}; MergeGateError::{class,held_permissions,authorized_actions,do_not}; ConfigError::{class,do_not}; CommitGateError::{class,held_permissions,authorized_actions,do_not}; but_authz::DenialClass; but_authz::AuthorizedAction; Authority Serialize (stable `:` token) or Authority::name() mapping; but_authz::to_envelope() shared JSON renderer; ForgeGateError::{class,held_permissions,authorized_actions,do_not} (classify_error copies from the underlying Denial/ConfigError) — the field source for review_gate_cli_error (STEER-005 AC-2); AdminWriteGateError::{class,held_permissions,authorized_actions,do_not} (classify_error copies from the underlying Denial/ConfigError) — the field source for governance_cli_error (STEER-005 admin-write AC)
consumes: but_authz::Denial (denial.rs:13); but_authz::Authority (authority.rs:10); but_authz::AuthoritySet (authority.rs:165); but_authz::ConfigError (config.rs:241); MergeGateError (merge_gate.rs:19); CommitGateError (commit/gate.rs:9); ForgeGateError (forge.rs:16) + forge::classify_error (forge.rs:24); AdminWriteGateError (config_mutate.rs:6) + config_mutate::classify_error (config_mutate.rs:31)
boundary_contracts:
  - CAP-STEER-01: the four steering fields exist on every denial carrier — Denial, MergeGateError, ConfigError (class+do_not only), CommitGateError, ForgeGateError, AdminWriteGateError — and serialize to one uniform JSON shape (`code`/`message`/`remediation_hint` preserved); `held_permissions` emitted in stable lexical order; `do_not` skipped when None; ForgeGateError/AdminWriteGateError's classify_error COPIES the four fields off the underlying Denial/ConfigError so the review and governance CLI serializers (STEER-005) have a real field source; values are wired by STEER-003/004 — this task proves the SHAPE.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-authz/src/denial.rs (MODIFY — add 4 fields + DenialClass + AuthorizedAction + to_envelope)
  - crates/but-authz/src/authority.rs (MODIFY — add #[derive(Serialize)] to Authority if that path is chosen)
  - crates/but-authz/src/config.rs (MODIFY — add class + do_not to ConfigError)
  - crates/but-authz/src/lib.rs (MODIFY — re-export the new types/fn)
  - crates/but-api/src/legacy/merge_gate.rs (MODIFY — add 4 fields to MergeGateError + config_invalid populates class+do_not)
  - crates/but-api/src/commit/gate.rs (MODIFY — add 4 fields to CommitGateError)
  - crates/but-authz/tests/steer_carriers.rs (NEW — serialization shape proofs)
  - crates/but-api/tests/steer_envelope.rs (NEW — to_envelope + backcompat reader)
  - crates/but-api/src/legacy/forge.rs (MODIFY — add 4 fields to ForgeGateError + classify_error copies them off the underlying Denial/ConfigError)
  - crates/but-api/src/legacy/config_mutate.rs (MODIFY — add 4 fields to AdminWriteGateError + classify_error copies them off the underlying Denial/ConfigError)
writeProhibited:
  - the gate deny/allow decision in any enforce_*_gate — NEVER weaken or change
  - any denial `code` string or exit code — NEVER change
  - crates/but-authz/tests/invariant_build_gates.rs — leave to STEER-010 (do not touch)
  - .spec/prds/governance/tasks/sprint-0[1-6]* — frozen task files

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
  - crates/but-authz/src/denial.rs (lines 1-32): Denial { code:&'static str, message:String, remediation_hint:String } at :13; derives Debug,Clone,PartialEq,Eq, no Serialize; PERM_DENIED_CODE const — this is where the four fields + new types land.
  - crates/but-authz/src/authority.rs (lines 1-116, 165-322): Authority enum :10 (no Serialize); Authority::name() :94 (stable `:` token); AuthoritySet :165, iter() :320 (BTreeSet deterministic order) — the serialization source of truth.
  - crates/but-authz/src/config.rs (lines 229-265): ConfigError thiserror struct :241 + code() :255 returning `config.invalid` — gets class+do_not only.
  - crates/but-authz/src/lib.rs (lines 1-19): public exports — add DenialClass, AuthorizedAction, to_envelope, (Authority Serialize if chosen) to the pub use blocks.
  - crates/but-api/src/legacy/merge_gate.rs (lines 18-37, 365-376): MergeGateError :19 derives Serialize + has unmet:Vec<String>; config_invalid() :369 — the second carrier; adding Vec<Authority> forces the Authority-Serialize decision.
  - crates/but-api/src/commit/gate.rs (lines 7-14): CommitGateError { code, message } :9 derives Serialize — the 4th carrier wrapper; STEER-001 adds the 4 fields, STEER-004/005 populate them.
  - crates/but-api/tests/commit_gate.rs (lines 295-322): governance_fixtures_are_structurally_distinct + governed_repo helper shape — the fixture seeding pattern to reuse for the serialization tests.
  - crates/but-api/src/legacy/forge.rs (lines 14-37): ForgeGateError {code,message} :16 derives Serialize,PartialEq,Eq; classify_error :24 currently flattens Denial/ConfigError to {code,message} — add the 4 fields + copy them off the underlying Denial/ConfigError (RR-1; the field source for review_gate_cli_error).
  - crates/but-api/src/legacy/config_mutate.rs (lines 4-44): AdminWriteGateError {code,message} :6 derives Serialize,PartialEq,Eq; classify_error :31 flattens Denial/ConfigError — add the 4 fields + copy them off the underlying Denial/ConfigError (RR-2; the field source for governance_cli_error at perm.rs:118).

--------------------------------------------------------------------------------
CODE PATTERN
--------------------------------------------------------------------------------
pattern: Additive struct fields + a new enum + a tuple-of-&'static-str struct, all deriving Debug,Clone,PartialEq,Eq; a hand-written or serde-derived Serialize for Authority emitting Authority::name(); a shared to_envelope() that maps both carriers into a serde_json::Value / a shared serializable struct.
pattern_source: crates/but-api/src/legacy/merge_gate.rs:19 (MergeGateError already derives Serialize with a Vec<String> field — mirror for Vec<AuthorizedAction>); crates/but-authz/src/authority.rs:94 (Authority::name() is the canonical token).
anti_pattern: Deriving Serialize on Authority that emits the Debug variant name (`ReviewsWrite`) instead of the `:` token; using serde(flatten) that collapses a legacy key; adding a field without the matching derive on the new types (breaks the carrier's derive — a compile break, not a silent pass).
references: 03-technical-requirements-delta.md §1; 05-delta-replan.md D1/D4/D5; 02-uc-steer.md UC-STEER-01
interaction_notes:
  - STEER-004 populates class/held/menu/do_not VALUES on these carriers; STEER-005 serializes them at the CLI sites; this task only proves the SHAPE compiles and serializes.

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
depends_on: (none)
blocks: STEER-002, STEER-004, STEER-005

CODING STANDARDS: crates/AGENTS.md, crates/WORKSPACE_MODEL.md, RULES.md
```
</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "description": "GIVEN the four carriers with new fields WHEN each is serialized THEN the JSON carries class/held_permissions/authorized_actions/do_not alongside the legacy keys (ConfigError: class+do_not only)",
      "verify": "cargo test -p but-authz steer_carriers_serialize_additive_superset && cargo test -p but-api steer_merge_envelope_uniform_shape"
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "GIVEN a legacy-key reader WHEN it parses post-STEER output THEN every legacy key + exit 1 is unchanged relative to each carrier's current output",
      "verify": "cargo test -p but-authz steer_legacy_keys_unchanged && cargo test -p but-api steer_carrier_backcompat_reader"
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "GIVEN held_permissions={comments:write,reviews:write} WHEN serialized THEN it is the sorted `:`-token array and deterministic",
      "verify": "cargo test -p but-authz steer_held_permissions_stable_lexical_token_order"
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "description": "GIVEN do_not=None vs Some WHEN serialized THEN None omits the key entirely and Some emits the literal string",
      "verify": "cargo test -p but-authz steer_do_not_skip_when_none"
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "description": "GIVEN the new types WHEN type-checked + cloned THEN derives hold and carriers still compile/compare equal",
      "verify": "cargo check -p but-authz --all-targets && cargo check -p but-api --all-targets && cargo test -p but-authz steer_types_derive_eq_clone"
    },
    {
      "id": "AC-6",
      "type": "acceptance_criterion",
      "description": "GIVEN a forge denial (an underlying Denial with steering fields populated) classified through forge::classify_error, and an admin-write denial classified through config_mutate::classify_error WHEN each classify_error builds its gate-error carrier and the result is serialized to JSON THEN ForgeGateError and AdminWriteGateError each carry `class`/`held_permissions`/`authorized_actions`/`do_not` copied off the underlying Denial (and `class`+`do_not` off ConfigError) ALONGSIDE their existing `code`/`message`; neither is left as a two-key `{code,message}` flatten",
      "verify": "cargo test -p but-api steer_forge_and_admin_carriers_copy_steering_fields"
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "Serialized Denial JSON contains the key class",
      "maps_to_ac": "AC-1",
      "verify": "cargo test -p but-authz steer_carriers_serialize_additive_superset"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "Serialized Denial JSON contains the key held_permissions",
      "maps_to_ac": "AC-1",
      "verify": "cargo test -p but-authz steer_carriers_serialize_additive_superset"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "Serialized Denial JSON contains the key authorized_actions",
      "maps_to_ac": "AC-1",
      "verify": "cargo test -p but-authz steer_carriers_serialize_additive_superset"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "ConfigError carries class+do_not but not held/menu",
      "maps_to_ac": "AC-1",
      "verify": "cargo test -p but-authz steer_config_error_class_and_do_not_only"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "Legacy-key reader parses post-STEER MergeGateError with no missing key",
      "maps_to_ac": "AC-2",
      "verify": "cargo test -p but-api steer_carrier_backcompat_reader"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "description": "held_permissions serializes sorted",
      "maps_to_ac": "AC-3",
      "verify": "cargo test -p but-authz steer_held_permissions_stable_lexical_token_order"
    },
    {
      "id": "TC-7",
      "type": "test_criterion",
      "description": "held_permissions serialization is deterministic across runs",
      "maps_to_ac": "AC-3",
      "verify": "cargo test -p but-authz steer_held_permissions_stable_lexical_token_order"
    },
    {
      "id": "TC-8",
      "type": "test_criterion",
      "description": "do_not=None emits no key",
      "maps_to_ac": "AC-4",
      "verify": "cargo test -p but-authz steer_do_not_skip_when_none"
    },
    {
      "id": "TC-9",
      "type": "test_criterion",
      "description": "cargo check passes with new derives",
      "maps_to_ac": "AC-5",
      "verify": "cargo check -p but-authz --all-targets && cargo check -p but-api --all-targets"
    },
    {
      "id": "TC-10",
      "type": "test_criterion",
      "description": "Cloned Denial == original",
      "maps_to_ac": "AC-5",
      "verify": "cargo test -p but-authz steer_types_derive_eq_clone"
    },
    {
      "id": "TC-11",
      "type": "test_criterion",
      "description": "ForgeGateError serialization contains the key `class` (copied off the underlying Denial)",
      "maps_to_ac": "AC-6",
      "verify": "cargo test -p but-api steer_forge_and_admin_carriers_copy_steering_fields"
    },
    {
      "id": "TC-12",
      "type": "test_criterion",
      "description": "AdminWriteGateError serialization contains the keys `class` and `held_permissions` (copied off the underlying Denial)",
      "maps_to_ac": "AC-6",
      "verify": "cargo test -p but-api steer_forge_and_admin_carriers_copy_steering_fields"
    }
  ]
}
-->
