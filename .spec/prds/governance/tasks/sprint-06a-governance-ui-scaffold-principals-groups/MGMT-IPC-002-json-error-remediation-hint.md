# MGMT-IPC-002: `json.rs` `Error` serializes the 3rd field `remediation_hint` (closes a real drop bug)

> **Red-Hat Remediation (cycle 1):** resolved R2 (corrected the AC-1 fixture/scenario + TC-2 to the REAL `Denial::missing_permission` hint `"request a reviewed merge or ask a maintainer to grant reviews:write"`, authorize.rs:131-132), SEC5 (added a `ConfigInvalid` hint carrier to the downcast set + AC-5/TC-8 proving a `config.invalid` carrier serializes its `code` + non-empty `remediation_hint`; consumed by MGMT-IPC-005 AC-5), R7 (note: `json::Error` serializes a FLAT `{code,message,remediation_hint}` object; the `{error:{…}}` envelope is the Tauri-boundary wrap), R8 (note: only `{code,message,remediation_hint}` cross transport — `MergeGateError.unmet[]` is intentionally NOT serialized here).

## What this does

Extends `but_api::json::Error`'s `Serialize` impl so that when the wrapped `anyhow::Error` chain carries a structured denial (`but_authz::Denial`, `legacy::merge_gate::MergeGateError`, or a `config.invalid` carrier), the transport emits a 3rd JSON field `remediation_hint` recovered by downcasting the chain. Today the transport serializes only `{code, message}` (`json.rs:265` `serialize_map(Some(2))`), so the carrier contract's third field is **dropped** and a denied governance write (or a fail-closed `config.invalid`) reaches the SvelteKit renderer without its remediation text (UC-MGMT-06/07 cannot show the hint). The change is **additive**: when no structured carrier is in the chain, the JSON is byte-identical to today's `{code, message}`.

> **Transport shape (R7):** `json::Error`'s `Serialize` impl emits a **FLAT** object `{ "code", "message", "remediation_hint"? }` — these three keys live at the top level of the serialized error. The `{ "error": { … } }` envelope that some consumers see is added at the **Tauri command boundary** (the `#[but_api]`/`tauri::command` wrap), not here. UC-MGMT-06/07 / SDK consumers therefore key off the **flat** `{code,message,remediation_hint}` object that this impl produces (the SDK layer unwraps any envelope before reaching it).
>
> **Crossing-the-wire contract (R8):** only `{code, message, remediation_hint}` cross transport. `MergeGateError.unmet[]` (and any other carrier-private fields) are **intentionally NOT** serialized by this impl — the renderer surfaces the human-readable `remediation_hint`, not the machine fragment list. Do not widen the serialized key set beyond these three.

## Why

Sprint 06a · PRD UC-MGMT-06, UC-MGMT-07 · criteria T-MGMT-027/028 · capability CAP-AUTHZ-01. This is the transport seam every governance command (MGMT-IPC-001) and the pending-read IPC (MGMT-IPC-005, which fails closed `config.invalid` with a hint) depends on to surface a structured `{code, message, remediation_hint}` denial to the governance UI.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-api error_serializes_remediation_hint_from_denial`: a real `but_authz::Denial::missing_permission(Authority::ReviewsWrite, &held)` (whose `remediation_hint` is `"request a reviewed merge or ask a maintainer to grant reviews:write"`) wrapped in `json::Error` and serialized via real `serde_json` produces JSON containing the `"remediation_hint"` key with that substring, alongside `"code":"perm.denied"` — a 3-field object. Full gate set in the spec below.

## Scope

- `crates/but-api/src/json.rs` (MODIFY) — the `error::Error` `Serialize` impl only (sized 2 or 3 entries); ADD the `ConfigInvalid` carrier definition consulted by the downcast (a small `std::error::Error` with a `remediation_hint`, or a `Denial` with code `config.invalid`); ADD round-trip tests.

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: MGMT-IPC-002 — json.rs Error serializes the 3rd field remediation_hint (closes a real drop bug)
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     S  (90 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ./SPRINT.md
PRD_REFS:   UC-MGMT-06, UC-MGMT-07, T-MGMT-027, T-MGMT-028
CAPABILITIES: CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-api error_serializes_remediation_hint_from_denial
  lint:  cargo clippy -p but-api --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
A but_authz::Denial (or MergeGateError, or a config.invalid carrier) wrapped in anyhow::Error and
serialized through the real json::Error via real serde_json produces a JSON object that CONTAINS the
key "remediation_hint" with the carrier's exact hint string; a plain anyhow!("msg") with no structured
carrier in the chain serializes to EXACTLY {"code":..,"message":..} with NO remediation_hint key; all
pre-existing json.rs error tests stay green. Additive + non-breaking. The serialized object is FLAT
({code,message,remediation_hint}); carrier-private fields (MergeGateError.unmet[]) are NOT serialized.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] Make the change ADDITIVE on the error::Error Serialize impl (json.rs:258-276): recover the
  hint by downcasting the anyhow::Error chain (err.downcast_ref::<but_authz::Denial>() /
  ::<MergeGateError>() / ::<ConfigInvalid carrier>() walking the FULL chain), emit a 3rd map entry
  remediation_hint when present, else emit exactly the existing {code, message}. Size serialize_map to
  the real entry count (2 or 3).
- [MUST] Define a SMALL config.invalid hint carrier and ADD it to the downcast set: either (a) a new
  `pub struct ConfigInvalid { code: &'static str /* "config.invalid" */, message: String,
  remediation_hint: String }` impl std::fmt::Display + std::error::Error (mirroring MergeGateError,
  merge_gate.rs:19-37), OR (b) reuse but_authz::Denial constructed with code "config.invalid" + a hint.
  RATIONALE: the existing but_authz::ConfigError (config.rs:230-256) is a thiserror::Error with code()
  == "config.invalid" but carries NO remediation_hint field — so it cannot supply a hint as-is; this
  carrier is what gives config.invalid a remediation_hint over transport. MGMT-IPC-005 AC-5 CONSUMES
  this carrier (its read IPC must return {code:"config.invalid"} WITH a non-empty remediation_hint).
- [MUST] Keep the existing code/message extraction (custom_context_or_error_chain(), json.rs:263)
  byte-for-byte unchanged for the no-hint case — every existing json.rs test must still pass.
- [MUST] Add a real carrier→Error→JSON round-trip integration proof per carrier (construct the real
  carrier, convert to anyhow::Error, wrap in json::Error, serialize via real serde_json::to_string,
  assert the hint key+value).
- [NEVER] NEVER hardcode the remediation_hint value or emit an empty placeholder when no carrier is in
  the chain — emit NO 3rd key in that case.
- [NEVER] NEVER change the UnmarkedError Serialize impl (json.rs:209-227) — it stays 2-field.
- [NEVER] NEVER serialize carrier-private fields (MergeGateError.unmet[]) — only {code,message,
  remediation_hint} cross the wire.
- [NEVER] NEVER break the additive contract: a consumer parsing {code, message} must still parse the
  no-hint case identically (no field renames, no required new key).
- [STRICTLY] Treat but_authz::Denial, legacy::merge_gate::MergeGateError, and the config.invalid
  carrier as the canonical hint carriers; extend the downcast set additively if a fourth surfaces.
  Cite carriers in the report.
- [STRICTLY] Size the serde map to the real entry count — never serialize_map(Some(2)) then write a 3rd entry.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: A Denial in the chain serializes remediation_hint as a 3rd JSON field
- [ ] AC-2: A MergeGateError in the chain also exposes its remediation_hint (second carrier)
- [ ] AC-3: A plain error (no denial) serializes the existing 2-field shape unchanged (additive)
- [ ] AC-4: A Denial nested below a .context layer is still recovered (full-chain walk)
- [ ] AC-5: A config.invalid carrier serializes {code:"config.invalid"} + non-empty remediation_hint (3rd carrier)
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first; each behavioral AC carries a scenario)
--------------------------------------------------------------------------------

AC-1 [PRIMARY]: A Denial in the error chain serializes the remediation_hint as a 3rd JSON field
  GIVEN: a but_authz::Denial::missing_permission(Authority::ReviewsWrite, &held) { code:"perm.denied",
         message names reviews:write, remediation_hint: "request a reviewed merge or ask a maintainer
         to grant reviews:write" (authorize.rs:131-132) } converted to anyhow::Error, wrapped in json::Error
  WHEN:  serialized via real serde_json::to_string
  THEN:  the JSON contains key "remediation_hint" whose value contains "request a reviewed merge or ask
         a maintainer to grant reviews:write", AND "code":"perm.denied" and "message" (containing
         "reviews:write") are present — 3 keys
  TEST_TIER: integration   VERIFICATION_SERVICE: but-api
  VERIFY: cargo test -p but-api error_serializes_remediation_hint_from_denial
  SCENARIO (un-fakeable): seed a REAL Denial (public_api); must_observe the "remediation_hint" key + the exact
    substring + "code":"perm.denied" + 3 keys; must_not_observe a 2-key {code,message} object / empty hint.
    negative_control: serialize_map stays Some(2) (the bug) / hardcoded-empty hint / stub returns old 2-key JSON.

AC-2: A MergeGateError in the chain also exposes its remediation_hint over transport (second carrier)
  GIVEN: a legacy::merge_gate::MergeGateError { code:"gate.review_required", remediation_hint:"collect the
         required approvals at the current review head", unmet:["min_approvals"] } wrapped in json::Error
  WHEN:  serialized via real serde_json::to_string
  THEN:  the JSON contains "remediation_hint" with "collect the required approvals at the current review head"
         AND "code":"gate.review_required"; the JSON does NOT contain an "unmet" key (carrier-private) — proving
         the downcast set covers MergeGateError, not only Denial, and that only {code,message,remediation_hint} cross
  TEST_TIER: integration   VERIFICATION_SERVICE: but-api
  VERIFY: cargo test -p but-api error_serializes_remediation_hint_from_merge_gate_error

AC-3: A plain error with no structured denial serializes the existing 2-field shape unchanged (additive)
  GIVEN: a plain anyhow!("err msg").context(but_error::Code::Validation) with NO Denial/MergeGateError/ConfigInvalid in chain
  WHEN:  serialized via real serde_json::to_string
  THEN:  the JSON equals exactly {"code":"Validation","message":"err msg"} — 2 keys, NO remediation_hint
         (byte-identical to the existing find_code test, json.rs:304-317)
  TEST_TIER: integration   VERIFICATION_SERVICE: but-api
  VERIFY: cargo test -p but-api error_without_denial_keeps_two_field_shape

AC-4: A Denial nested deeper in the chain (via .context) is still recovered (full-chain walk)
  GIVEN: a Denial::no_handle()-shape denial wrapped, then .context("failed to authorize governance write")
         layered ON TOP, then wrapped in json::Error
  WHEN:  serialized via real serde_json::to_string
  THEN:  the JSON still contains "remediation_hint" with "set BUT_AGENT_HANDLE to a principal committed in
         governance config" — proving the impl walks the full anyhow chain, not only the head
  TEST_TIER: integration   VERIFICATION_SERVICE: but-api
  VERIFY: cargo test -p but-api error_recovers_hint_from_nested_denial

AC-5: A config.invalid carrier serializes its code + a non-empty remediation_hint over transport (third carrier)
  GIVEN: a real config.invalid hint carrier (a `ConfigInvalid { code:"config.invalid", message names the
         malformed file, remediation_hint:"fix the malformed governance config and recommit it to the target
         branch" }` impl std::error::Error, OR a Denial constructed with code "config.invalid" + that hint),
         wrapped in json::Error
  WHEN:  serialized via real serde_json::to_string
  THEN:  the JSON contains "code":"config.invalid" AND a NON-EMPTY "remediation_hint" — proving config.invalid
         errors carry their hint over transport (the carrier MGMT-IPC-005 AC-5 consumes), not just {code,message}
  TEST_TIER: integration   VERIFICATION_SERVICE: but-api
  VERIFY: cargo test -p but-api error_serializes_remediation_hint_from_config_invalid

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): serializing a json::Error wrapping a Denial produces JSON containing key remediation_hint
    VERIFY: cargo test -p but-api error_serializes_remediation_hint_from_denial
- TC-2 (-> AC-1): the serialized remediation_hint equals "request a reviewed merge or ask a maintainer to grant reviews:write"
    VERIFY: cargo test -p but-api error_serializes_remediation_hint_from_denial
- TC-3 (-> AC-2): MergeGateError-wrapped json::Error carries "collect the required approvals at the current review head" and emits no "unmet" key
    VERIFY: cargo test -p but-api error_serializes_remediation_hint_from_merge_gate_error
- TC-4 (-> AC-3): plain anyhow!("err msg").context(Code::Validation) serializes exactly {"code":"Validation","message":"err msg"} with no remediation_hint
    VERIFY: cargo test -p but-api error_without_denial_keeps_two_field_shape
- TC-5 (-> AC-3): the no-denial serialized object has exactly two keys code and message
    VERIFY: cargo test -p but-api error_without_denial_keeps_two_field_shape
- TC-6 (-> AC-4): a Denial nested below a .context layer still includes remediation_hint with the nested hint
    VERIFY: cargo test -p but-api error_recovers_hint_from_nested_denial
- TC-7 (-> AC-3): the pre-existing json.rs error tests (find_code, find_context, multiple_codes) still pass
    VERIFY: cargo test -p but-api json::error::tests
- TC-8 (-> AC-5): a config.invalid carrier wrapped in json::Error serializes {code:"config.invalid"} + a non-empty remediation_hint
    VERIFY: cargo test -p but-api error_serializes_remediation_hint_from_config_invalid

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides: faithful transport of the FLAT {code, message, remediation_hint} carrier contract to the renderer,
          across all three hint carriers (Denial, MergeGateError, config.invalid)
consumes: but_api::json::Error (json.rs:258-276); but_authz::Denial (denial.rs:13-21);
          legacy::merge_gate::MergeGateError (merge_gate.rs:19-37); but_authz::ConfigError (config.rs:230-256,
          code()=="config.invalid" but NO hint field — motivates the new ConfigInvalid carrier);
          the merge_gate/config_mutate classify_error downcast pattern
boundary_contracts:
  - CAP-AUTHZ-01: the {code, message, remediation_hint} contract (04-api-design.md:23-28,:136) must survive
    transport intact for ALL carriers; today the 3rd field is dropped at json.rs:265 — this task closes that.
  - MGMT-IPC-005 (tauri pending-read IPC) CONSUMES the config.invalid carrier: its AC-5 requires a malformed
    working-tree config to fail closed with {code:"config.invalid"} AND a non-empty remediation_hint over
    json::Error — only possible because this task serializes that carrier's hint.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-api/src/json.rs (MODIFY) — extend ONLY the error::Error Serialize impl to recover+emit
    remediation_hint sized 2|3 across Denial/MergeGateError/ConfigInvalid; DEFINE the small ConfigInvalid
    hint carrier here (pub struct impl std::error::Error, or a Denial-code-"config.invalid" constructor);
    ADD round-trip tests in error::tests or a sibling #[cfg(test)] mod
writeProhibited:
  - crates/but-api/src/json.rs UnmarkedError Serialize impl (209-227) — stays 2-field
  - crates/but-authz/src/denial.rs, authorize.rs — Denial carrier is CONSUME-only
  - crates/but-authz/src/config.rs — ConfigError is CONSUME-only (do NOT add a hint field to it; the new
    ConfigInvalid carrier lives in but-api json.rs)
  - crates/but-api/src/legacy/merge_gate.rs, config_mutate.rs, commit/gate.rs — CONSUME-only (mirror the downcast)
  - crates/but-error/** — Code/Context not the hint source
  - any gitbutler-* crate; any file not in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-api/src/json.rs:258-276 — the error::Error Serialize impl with the BUG: serialize_map(Some(2)) at :265
2. crates/but-api/src/json.rs:232-244 — struct Error(anyhow::Error) + From impls (the wrapped error to downcast)
3. crates/but-api/src/json.rs:209-227 — UnmarkedError Serialize (do NOT touch)
4. crates/but-api/src/json.rs:278-411 — existing error tests (json() helper at :285; find_code/find_context/multiple_codes) — must stay green
5. crates/but-authz/src/denial.rs:13-21 — Denial { code, message, remediation_hint } (primary carrier)
6. crates/but-authz/src/authorize.rs:104-132 — Denial constructors: missing_permission hint is "request a reviewed merge or ask a maintainer to grant {name}"; no_handle hint is "set BUT_AGENT_HANDLE to a principal committed in governance config"; + impl Error
7. crates/but-api/src/legacy/merge_gate.rs:19-37,113-124 — MergeGateError (struct impl std::error::Error w/ remediation_hint) + classify_error downcast pattern to mirror for the new ConfigInvalid carrier
8. crates/but-authz/src/config.rs:230-256 — ConfigError (thiserror, code()=="config.invalid", NO hint field) — shows WHY a new ConfigInvalid hint carrier is needed
9. crates/but-api/src/legacy/config_mutate.rs:31-43 — classify_error downcast to Denial (same recovery shape)
10. brain/docs/rust/error-handling.md — anyhow downcasting, Result/Error transport (no try/catch); crates/AGENTS.md — but_error::Code classification, no string-matching for consumers

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- cargo test -p but-api error_serializes_remediation_hint_from_denial          -> Exit 0
- cargo test -p but-api error_serializes_remediation_hint_from_merge_gate_error -> Exit 0
- cargo test -p but-api error_without_denial_keeps_two_field_shape             -> Exit 0
- cargo test -p but-api error_recovers_hint_from_nested_denial                 -> Exit 0
- cargo test -p but-api error_serializes_remediation_hint_from_config_invalid  -> Exit 0
- cargo test -p but-api json::error::tests                                     -> Exit 0 (no regression)
- cargo clippy -p but-api --all-targets                                        -> clean
- cargo fmt --check                                                            -> clean

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: Additive serde Serialize extension via error-chain downcast — keep code/message extraction, walk
  anyhow::Error::chain() for the first element downcastable to a hint carrier (Denial | MergeGateError |
  ConfigInvalid), conditionally serialize a 3rd remediation_hint entry; size SerializeMap to the actual count.
  The serialized object is FLAT {code,message,remediation_hint}; carrier-private fields (unmet[]) are excluded.
pattern_source: crates/but-api/src/legacy/merge_gate.rs:19-37 (carrier struct impl std::error::Error),
  :113-124 (classify_error downcast -> remediation_hint.clone())
anti_pattern: hardcoding/placeholdering the hint; always-present empty remediation_hint for non-carrier errors;
  adding remediation_hint to but_error::Context or to but_authz::ConfigError (wrong layer/crate); serialize_map(Some(2))
  then a 3rd entry; head-only chain inspection so a .context-wrapped Denial drops the hint (AC-4 catches this);
  serializing MergeGateError.unmet[] over the wire (R8 — only the 3 keys cross).

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — owns the but-api transport layer (Result/Error, serde, anyhow downcasting)
reviewer: rust-reviewer
coding_standards: crates/AGENTS.md, brain/docs/rust/error-handling.md, brain/docs/rust/testing.md

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: none
Blocks:     MGMT-IPC-001, MGMT-IPC-003, MGMT-IPC-004, MGMT-IPC-005 (config.invalid hint carrier)
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "MGMT-IPC-002",
  "proposed_by": "rust-planner",
  "verification_policy": { "requires_tests": true, "requires_red_evidence": true, "requires_seeded_evidence": true },
  "fixtures": {
    "denial_reviews_write": { "description": "A real but_authz::Denial::missing_permission(Authority::ReviewsWrite, &held) constructed in-process (Denial impls std::error::Error so it survives anyhow downcast). The real remediation_hint is authorize.rs:131-132.", "seed_method": "public_api", "records": ["Denial { code: \"perm.denied\", message names reviews:write, remediation_hint: \"request a reviewed merge or ask a maintainer to grant reviews:write\" }"] },
    "merge_gate_error_review_required": { "description": "A real but_api::legacy::merge_gate::MergeGateError (the second hint carrier) constructed in-process.", "seed_method": "public_api", "records": ["MergeGateError { code: \"gate.review_required\", remediation_hint: \"collect the required approvals at the current review head\", unmet: [\"min_approvals\"] }"] },
    "plain_validation_error": { "description": "A plain anyhow!(\"err msg\").context(but_error::Code::Validation) with NO structured carrier in the chain (the existing json.rs:305 find_code input).", "seed_method": "public_api", "records": ["anyhow error, code Validation, message \"err msg\", no Denial/MergeGateError/ConfigInvalid carrier"] },
    "nested_no_handle_denial": { "description": "A real but_authz::Denial::no_handle() wrapped then layered with .context(\"failed to authorize governance write\") so the Denial is a nested chain source, not the head.", "seed_method": "public_api", "records": ["Denial::no_handle() under a .context layer; remediation_hint = \"set BUT_AGENT_HANDLE to a principal committed in governance config\""] },
    "config_invalid_carrier": { "description": "A real config.invalid hint carrier constructed in-process — a ConfigInvalid { code:\"config.invalid\", message, remediation_hint } impl std::error::Error (mirroring MergeGateError), OR a Denial built with code \"config.invalid\" + a hint. This is the carrier MGMT-IPC-005 AC-5 consumes; it exists because but_authz::ConfigError (config.rs:230-256) has no remediation_hint field.", "seed_method": "public_api", "records": ["ConfigInvalid { code: \"config.invalid\", message names the malformed file, remediation_hint: \"fix the malformed governance config and recommit it to the target branch\" }"] }
  },
  "requirements": [
    { "id": "AC-1", "type": "acceptance_criterion", "primary": true, "description": "GIVEN a but_authz::Denial(remediation_hint=\"request a reviewed merge or ask a maintainer to grant reviews:write\") wrapped in json::Error WHEN serialized via real serde_json THEN the JSON has key remediation_hint with that substring + code perm.denied + 3 keys", "verify": "cargo test -p but-api error_serializes_remediation_hint_from_denial", "scenario": { "id": "AC-1", "primary": true, "tier": "visible", "test_tier": "integration", "verification_service": "but-api", "negative_control": { "would_fail_if": ["Error serialization drops the 3rd field (serialize_map stays Some(2))", "remediation_hint is hardcoded empty or a static placeholder", "the impl is a stub returning the old 2-key JSON unchanged"] }, "evidence": { "artifact_type": "stdout", "required_capture": true }, "cases": [ { "start_ref": "denial_reviews_write", "action": { "actor": "ci", "steps": ["construct the real Denial::missing_permission(Authority::ReviewsWrite, &held)", "anyhow::Error::from(denial)", "wrap in json::Error", "serde_json::to_string and parse back"] }, "end_state": { "must_observe": ["JSON contains key `\"remediation_hint\"`", "the `remediation_hint` value contains `\"request a reviewed merge or ask a maintainer to grant reviews:write\"`", "`\"code\":\"perm.denied\"` present", "the parsed object has 3 keys"], "must_not_observe": ["a 2-key object `{\"code\",\"message\"}` with remediation_hint absent", "an empty `\"remediation_hint\":\"\"` value"] } } ] } },
    { "id": "AC-2", "type": "acceptance_criterion", "primary": false, "description": "GIVEN a MergeGateError(remediation_hint=\"collect the required approvals at the current review head\") wrapped in json::Error WHEN serialized THEN JSON has remediation_hint with that substring + code gate.review_required, and NO unmet key (downcast covers MergeGateError; only 3 keys cross)", "verify": "cargo test -p but-api error_serializes_remediation_hint_from_merge_gate_error", "scenario": { "id": "AC-2", "primary": false, "tier": "visible", "test_tier": "integration", "verification_service": "but-api", "negative_control": { "would_fail_if": ["the downcast only handles Denial and omits MergeGateError — hint dropped for this carrier", "remediation_hint hardcoded to the Denial string", "the merge-gate hint serializes as an empty/static value", "the serialized object leaks the carrier-private unmet[] key"] }, "evidence": { "artifact_type": "stdout", "required_capture": true }, "cases": [ { "start_ref": "merge_gate_error_review_required", "action": { "actor": "ci", "steps": ["construct the real MergeGateError", "anyhow::Error::from + wrap in json::Error", "serde_json::to_string and parse back"] }, "end_state": { "must_observe": ["JSON key `\"remediation_hint\"` present", "its value contains `\"collect the required approvals at the current review head\"`", "`\"code\":\"gate.review_required\"` present", "the parsed object has exactly 3 keys (no `\"unmet\"` key)"], "must_not_observe": ["the `remediation_hint` key missing (none) for the merge-gate carrier — an empty 2-key object", "the Denial-specific hint string leaking in", "an `\"unmet\"` key present (carrier-private field wrongly serialized)"] } } ] } },
    { "id": "AC-3", "type": "acceptance_criterion", "primary": false, "description": "GIVEN a plain anyhow!(\"err msg\").context(Code::Validation) with no carrier in the chain WHEN serialized THEN JSON equals exactly {\"code\":\"Validation\",\"message\":\"err msg\"} (2 keys, no remediation_hint) — additive/non-breaking", "verify": "cargo test -p but-api error_without_denial_keeps_two_field_shape", "scenario": { "id": "AC-3", "primary": false, "tier": "visible", "test_tier": "integration", "verification_service": "but-api", "negative_control": { "would_fail_if": ["remediation_hint hardcoded so it appears even with no carrier (a static always-present key)", "an empty `\"remediation_hint\":\"\"` emitted for plain errors", "the code/message extraction was changed and the existing 2-field output no longer matches"] }, "evidence": { "artifact_type": "stdout", "required_capture": true }, "cases": [ { "start_ref": "plain_validation_error", "action": { "actor": "ci", "steps": ["build anyhow!(\"err msg\").context(Code::Validation)", "wrap in json::Error", "serde_json::to_string and compare to the literal"] }, "end_state": { "must_observe": ["the JSON equals exactly `{\"code\":\"Validation\",\"message\":\"err msg\"}`", "the parsed object has exactly 2 keys"], "must_not_observe": ["a `\"remediation_hint\"` key present for a plain non-carrier error", "any 3rd key added to the empty/no-hint case"] } } ] } },
    { "id": "AC-4", "type": "acceptance_criterion", "primary": false, "description": "GIVEN a Denial::no_handle() nested below a .context layer WHEN serialized THEN JSON still contains remediation_hint with the nested hint (full-chain walk, not head-only)", "verify": "cargo test -p but-api error_recovers_hint_from_nested_denial", "scenario": { "id": "AC-4", "primary": false, "tier": "visible", "test_tier": "integration", "verification_service": "but-api", "negative_control": { "would_fail_if": ["the impl only downcasts the chain head so a `.context`-wrapped Denial drops the hint", "the hint is hardcoded so the nested case passes for the wrong reason", "the chain walk is a no-op stub that never inspects nested sources"] }, "evidence": { "artifact_type": "stdout", "required_capture": true }, "cases": [ { "start_ref": "nested_no_handle_denial", "action": { "actor": "ci", "steps": ["construct Denial::no_handle()", "anyhow::Error::from then add `.context(...)` so the Denial is nested", "wrap in json::Error, serde_json::to_string, parse"] }, "end_state": { "must_observe": ["JSON key `\"remediation_hint\"` present despite the layered context", "its value contains `\"set BUT_AGENT_HANDLE to a principal committed in governance config\"`", "`\"code\":\"perm.denied\"` present"], "must_not_observe": ["remediation_hint absent because the Denial was nested below a context layer", "an empty hint value when a nested Denial carries one"] } } ] } },
    { "id": "AC-5", "type": "acceptance_criterion", "primary": false, "description": "GIVEN a config.invalid hint carrier (ConfigInvalid impl std::error::Error, or a Denial with code config.invalid) wrapped in json::Error WHEN serialized THEN JSON has code config.invalid + a non-empty remediation_hint (3rd carrier; the carrier MGMT-IPC-005 AC-5 consumes)", "verify": "cargo test -p but-api error_serializes_remediation_hint_from_config_invalid", "scenario": { "id": "AC-5", "primary": false, "tier": "visible", "test_tier": "integration", "verification_service": "but-api", "negative_control": { "would_fail_if": ["the downcast omits the config.invalid carrier so its hint is dropped — an empty 2-key {code,message} object", "remediation_hint is hardcoded/static instead of read from the carrier", "the carrier is a stub returning the old 2-key JSON", "config.invalid serializes with no remediation_hint key at all"] }, "evidence": { "artifact_type": "stdout", "required_capture": true }, "cases": [ { "start_ref": "config_invalid_carrier", "action": { "actor": "ci", "steps": ["construct the real config.invalid carrier (ConfigInvalid { code:\"config.invalid\", message, remediation_hint } or Denial code config.invalid + hint)", "anyhow::Error::from + wrap in json::Error", "serde_json::to_string and parse back"] }, "end_state": { "must_observe": ["`\"code\":\"config.invalid\"` present", "JSON key `\"remediation_hint\"` present", "the `remediation_hint` value is non-empty (length > 0)", "the parsed object has 3 keys"], "must_not_observe": ["a 2-key object `{\"code\",\"message\"}` with remediation_hint absent for config.invalid", "an empty `\"remediation_hint\":\"\"` value"] } } ] } },
    { "id": "TC-1", "type": "test_criterion", "description": "serializing a json::Error wrapping a Denial produces JSON containing key remediation_hint", "verify": "cargo test -p but-api error_serializes_remediation_hint_from_denial", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "the serialized remediation_hint equals the Denial's hint string \"request a reviewed merge or ask a maintainer to grant reviews:write\"", "verify": "cargo test -p but-api error_serializes_remediation_hint_from_denial", "maps_to_ac": "AC-1" },
    { "id": "TC-3", "type": "test_criterion", "description": "MergeGateError-wrapped json::Error carries the merge-gate hint substring and emits no unmet key (carrier-private field excluded)", "verify": "cargo test -p but-api error_serializes_remediation_hint_from_merge_gate_error", "maps_to_ac": "AC-2" },
    { "id": "TC-4", "type": "test_criterion", "description": "plain non-carrier error serializes exactly {\"code\":\"Validation\",\"message\":\"err msg\"} with no remediation_hint", "verify": "cargo test -p but-api error_without_denial_keeps_two_field_shape", "maps_to_ac": "AC-3" },
    { "id": "TC-5", "type": "test_criterion", "description": "the no-denial serialized object has exactly two keys code and message", "verify": "cargo test -p but-api error_without_denial_keeps_two_field_shape", "maps_to_ac": "AC-3" },
    { "id": "TC-6", "type": "test_criterion", "description": "a Denial nested below a .context layer still includes remediation_hint with the nested hint", "verify": "cargo test -p but-api error_recovers_hint_from_nested_denial", "maps_to_ac": "AC-4" },
    { "id": "TC-7", "type": "test_criterion", "description": "the pre-existing json.rs error tests still pass with their current two-field expected JSON", "verify": "cargo test -p but-api json::error::tests", "maps_to_ac": "AC-3" },
    { "id": "TC-8", "type": "test_criterion", "description": "a config.invalid carrier wrapped in json::Error serializes {code:\"config.invalid\"} + a non-empty remediation_hint (3 keys)", "verify": "cargo test -p but-api error_serializes_remediation_hint_from_config_invalid", "maps_to_ac": "AC-5" }
  ]
}
-->
</details>
