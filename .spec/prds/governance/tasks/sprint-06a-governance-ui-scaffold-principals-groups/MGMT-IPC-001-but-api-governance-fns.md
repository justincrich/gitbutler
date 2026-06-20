# MGMT-IPC-001: `#[but_api]` governance wrapper layer (perm/group/status) + `json::Error` transport

> **Red-Hat Remediation (cycle 1):** resolved R1 (the "wrap without touching bodies" premise was impossible — `#[but_api]` REQUIRES a `Context` param, macro lib.rs:560, but the Sprint-05 `governance.rs` fns take `&gix::Repository` + explicit `target_ref`; RE-SCOPED to a NEW thin Context-param `#[but_api]` wrapper layer that resolves repo + workspace target ref from `ctx` and DELEGATES to the un-forked Sprint-05 `&repo` fns — the Sprint-05 BODIES stay untouched), R5 (pinned `governance_status_read`'s exact Context-param signature: resolves the workspace target ref from `ctx`, reads config via `load_governance_config`, returns the caller's own `AuthoritySet` via `effective_authority`), R6 (pinned a `#[derive(Serialize)]` `GrantOutcome` carrying the ref-pin caveat as a RETURNED value, asserted over transport), R3 (added the identity-layering note: this proves the gate decision under the `BUT_AGENT_HANDLE` identity model; the desktop fleet-owner identity / `UserService` (T-MGMT-042) is wired + re-tested in MGMT-IPC-003 — a layered boundary, not a contradiction), R4 (strengthened BLOCKED-UNTIL: `governance.rs` MUST be in `invariant_build_gates` ENFORCEMENT_PATHS — added by Sprint-05 CLI-001 — and TC-9 now asserts the grep NON-VACUOUSLY matches `governance.rs`), SEC3 (added a negative AC-5/TC-10 proving `governance_status_read` is self-scoped — its generated signature takes NO principal arg).

## What this does

Authors a NEW thin `#[but_api]` **governance wrapper layer** (e.g. `perm_grant_cmd`/`perm_list_cmd`/`perm_revoke_cmd`/`group_create_cmd`/`group_grant_cmd`/`group_add_member_cmd`/`group_remove_member_cmd`/`group_delete_cmd`/`group_list_cmd`) that takes a `Context` (the param the `#[but_api]` macro REQUIRES), resolves the repo + workspace target ref from `ctx`, and **DELEGATES** to the un-forked Sprint-05 `governance.rs` `&gix::Repository`-based fns. It also authors one new self-scoped `governance_status_read` (the caller's own effective `AuthoritySet` via `but_authz::effective_authority`). It **re-uses** the Sprint-05 functions verbatim — the wrapper is new transport code; the Sprint-05 fn BODIES are never edited, and the AUTHZ-006 `administration:write` gate already composed inside them is never re-implemented or weakened. Denials flow through `json::Error` carrying `remediation_hint` (depends on MGMT-IPC-002).

> **Why a wrapper layer, not in-place `#[but_api]` (R1):** the `#[but_api]` proc-macro **requires** a `Context`/`&Context`/`&mut Context`/`ThreadSafeContext` parameter to translate to `project_id` (macro lib.rs:560 — explicit repository-permission params *require* a `Context` param). The Sprint-05 `governance.rs` fns take `&gix::Repository` + an explicit `target_ref` (no `Context`). Applying `#[but_api]` to them in place is impossible without rewriting their signatures — which would touch Sprint-05-owned bodies. The fix is a thin wrapper: each `*_cmd(ctx, …)` fn carries the `Context` the macro needs, derives `&repo` + the workspace target ref from `ctx`, then calls the Sprint-05 `&repo` fn unchanged.

> **Identity-layering boundary (R3):** this task proves the but-api GATE DECISION under the **`BUT_AGENT_HANDLE` identity model** (the CLI/agent path: `resolve_principal_from_env`). The **desktop fleet-owner identity** (`UserService`, criteria T-MGMT-042) is a DIFFERENT identity source — it is wired into the Tauri command path and RE-TESTED in **MGMT-IPC-003** (tauri-implementer). These are two layers of the same gate (identity resolution → `authorize()`), not a contradiction: MGMT-IPC-001 locks the gate behavior under the agent-handle identity; MGMT-IPC-003 re-verifies it under the desktop identity. Do not attempt to wire `UserService` here.

## Why

Sprint 06a · PRD UC-MGMT-06/07, UC-MGMT-02/03, UC-AUTHZ-03 · criteria T-MGMT-027 · capability CAP-AUTHZ-01. This is the Rust producer the MGMT renderer ultimately invokes (via MGMT-IPC-003 registration + MGMT-IPC-004 SDK regen) — the "UI is never a bypass" seam: server-side `but-authz` enforcement at the `but-api` boundary.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-api governance_api_perm_grant_admin_lands_inert`: an admin `perm_grant_cmd` invoked through the new `#[but_api]` wrapper against a real git repo writes `reviews:write` into the working-tree `.gitbutler/permissions.toml`, returns a `#[derive(Serialize)]` `GrantOutcome` whose ref-pin caveat field survives transport, and leaves `refs/heads/main` unmoved (inert). Full gate set in the spec below.

## Scope

- `crates/but-api/src/legacy/governance.rs` (MODIFY — Sprint-05-owned, EXTENDED) — ADD the new `#[but_api]` wrapper fns (`*_cmd(ctx, …)`) that resolve repo + target ref from `ctx` and delegate to the existing Sprint-05 `&repo` fns; ADD self-scoped `governance_status_read`; ADD the `#[derive(Serialize)] GrantOutcome` return type. The Sprint-05 fn BODIES are NOT edited.
- `crates/but-api/tests/governance_api.rs` (NEW) — AC-1..AC-5 integration proofs.

## ⚠ BLOCKED-UNTIL

`crates/but-api/src/legacy/governance.rs` is **authored by Sprint-05 (CLI-001/CLI-002)** and is **not on disk yet** (Sprint 05 In Progress). MGMT-IPC-001 GREENs only after Sprint-05 lands `governance.rs` **AND** Sprint-05 CLI-001 has added `governance.rs` to the `invariant_build_gates` ENFORCEMENT_PATHS (so the AUTHZ-007/008 honesty grep covers the new file). Do **not** stub the governance fns to unblock — that is the cardinal stubbing sin. Read the planned signatures (the `&gix::Repository` + `target_ref` shape) from `../sprint-05-cli-perm-group/CLI-001-perm-cli-verbs.md` + `CLI-002-group-cli-verbs.md`. If `governance.rs` is absent from ENFORCEMENT_PATHS when this task starts, FLAG it (TC-9 would be vacuous) and coordinate with Sprint-05 before proceeding.

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: MGMT-IPC-001 — #[but_api] governance wrapper layer (perm/group/status) + json::Error transport
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     M  (120 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ./SPRINT.md
PRD_REFS:   UC-MGMT-06, UC-MGMT-07, UC-MGMT-02, UC-MGMT-03, UC-AUTHZ-03, T-MGMT-027
CAPABILITIES: CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-api governance_api_perm_grant_admin_lands_inert
  lint:  cargo clippy -p but-api --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
A NEW thin #[but_api] wrapper layer exposes each Sprint-05 governance fn (perm_*, group_*): each *_cmd(ctx, ..)
takes the Context the macro requires, resolves &repo + the workspace target ref from ctx, and DELEGATES to the
un-forked Sprint-05 &repo fn (bodies untouched). governance_status_read returns the caller's OWN effective
AuthoritySet (self-scoped, NO principal arg). A non-admin perm_grant_cmd returns json::Error code "perm.denied"
with a non-empty remediation_hint and writes nothing; an admin perm_grant_cmd returns Ok + a #[derive(Serialize)]
GrantOutcome whose ref-pin caveat survives transport and lands the token inert in the working tree; the
AUTHZ-007/008 honesty grep NON-VACUOUSLY covers governance.rs. No parallel implementation, no weakened gate.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] Author a NEW thin #[but_api] wrapper layer: for each Sprint-05 governance fn (perm_*, group_*),
  add a *_cmd(ctx: &Context /* or Context/ThreadSafeContext per the macro */, ..) fn that (a) resolves
  &gix::Repository + the workspace target ref from ctx, (b) DELEGATES to the un-forked Sprint-05 &repo fn.
  The #[but_api] macro REQUIRES a Context param (macro lib.rs:560) — that is WHY the wrapper exists; the
  Sprint-05 fns take &repo + explicit target_ref and CANNOT be #[but_api]'d in place without editing their
  signatures (forbidden).
- [MUST] Re-use the Sprint-05 governance.rs fn BODIES verbatim — the wrapper adds transport + ctx-resolution
  ONLY; the administration:write gate (AUTHZ-006 enforce_administration_write_gate) is ALREADY inside the
  delegated fns; do NOT re-implement, duplicate, or weaken it, and do NOT add a 2nd authorize() in the wrapper.
- [MUST] Follow the existing #[but_api] convention (platform.rs:22, github.rs:20, legacy/absorb.rs:37);
  the macro generates func_json/func_cmd/func_napi -> Result<JsonRVal, json::Error> and translates
  Context/ThreadSafeContext -> project_id.
- [MUST] Author a NEW self-scoped governance_status_read with EXACT signature (R5):
    #[but_api] pub fn governance_status_read(ctx: &Context) -> anyhow::Result<AuthoritySet>
  (NO principal arg — self scope). It resolves &repo + the workspace target ref from ctx, resolves the
  caller via but_authz::resolve_principal_from_env (BUT_AGENT_HANDLE), loads config at the target ref via
  load_governance_config(&repo, target_ref), and returns effective_authority(&principal, &cfg) — the
  caller's OWN union (own ∪ groups). It NEVER accepts or reads another principal's identity.
- [MUST] Pin a #[derive(Serialize)] outcome type (R6): the admin perm_grant_cmd returns a
    #[derive(Serialize)] struct GrantOutcome { caveat: String /* the ref-pin caveat */, .. }
  so the ref-pin caveat is a RETURNED VALUE that survives json transport — NOT a printed/log side-effect.
  AC-1 asserts the caveat in the returned/serialized value.
- [MUST] Surface denials as json::Error carrying remediation_hint (requires MGMT-IPC-002 merged first;
  the real Denial::missing_permission hint is "request a reviewed merge or ask a maintainer to grant {name}").
- [MUST] Prove against REAL services: real but-api governance fns + real but-authz + real gix via
  but_testsupport::writable_scenario, BUT_AGENT_HANDLE via temp_env under #[serial_test::serial]. No mocks.
- [NEVER] NEVER fork/re-author the Sprint-05 fn BODIES; NEVER edit their &repo+target_ref signatures;
  NEVER add a parallel authorize(AdministrationWrite) call (UI is never a bypass).
- [NEVER] NEVER let governance_status_read (or any command) accept a principal arg or read ANOTHER
  principal's set — self-scoped, identity from BUT_AGENT_HANDLE only.
- [NEVER] NEVER branch on role names or human-vs-AI predicates (the AUTHZ-007/008 grep must stay green).
- [NEVER] NEVER wire the desktop UserService / fleet-owner identity (T-MGMT-042) here — that is the
  layered identity re-tested in MGMT-IPC-003 (see Identity-layering boundary). This task uses the
  BUT_AGENT_HANDLE identity model only.
- [NEVER] NEVER register the Tauri generate_handler!, capability, identity, or SDK regen here — that is
  MGMT-IPC-003/004 (tauri-implementer). This task stops at the #[but_api] wrapper layer + governance_status_read + proofs.
- [STRICTLY] Treat the Sprint-05 gate + writer as a CONSUMED upstream seam reached via DELEGATION;
  resolve ctx -> &repo + target ref in the wrapper, then call the Sprint-05 fn.
- [STRICTLY] BLOCKED-UNTIL Sprint-05 governance.rs is merged AND present in invariant_build_gates
  ENFORCEMENT_PATHS; do not stub to unblock; if governance.rs is absent from ENFORCEMENT_PATHS, FLAG it
  (TC-9 would be vacuous) and coordinate with Sprint-05.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: admin perm_grant_cmd via the wrapper lands the token + returns a serialized GrantOutcome with the ref-pin caveat (inert)
- [ ] AC-2: non-admin perm_grant_cmd denied perm.denied + non-empty remediation_hint, writes nothing
- [ ] AC-3: governance_status_read returns the caller's own effective AuthoritySet (own ∪ groups), self-scoped
- [ ] AC-4: non-admin group_add_member_cmd denied with a hint, writes nothing
- [ ] AC-5: governance_status_read is self-scoped — its generated command signature takes NO principal arg (cannot read a foreign principal)
- [ ] All verification gates pass; the honesty grep NON-VACUOUSLY covers governance.rs; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: An admin perm_grant_cmd invoked through the new #[but_api] wrapper lands the token and returns a serialized GrantOutcome with the ref-pin caveat
  GIVEN: a real git repo (but_testsupport::writable_scenario) with committed .gitbutler/permissions.toml
         (admin=administration:write, rust-implementer=[contents:write]); BUT_AGENT_HANDLE=admin
  WHEN:  exposed perm_grant_cmd(ctx, "refs/heads/main", "rust-implementer", ["reviews:write"]) is invoked
         (the wrapper resolves &repo + target ref from ctx and delegates to the Sprint-05 &repo fn)
  THEN:  Ok(GrantOutcome); working-tree permissions.toml contains reviews:write under rust-implementer; the
         SERIALIZED GrantOutcome.caveat contains "takes effect once committed to the target branch";
         refs/heads/main ref_id unchanged (inert)
  TEST_TIER: integration   VERIFICATION_SERVICE: but-api
  VERIFY: cargo test -p but-api governance_api_perm_grant_admin_lands_inert

AC-2: A non-admin perm_grant_cmd through the wrapper is denied with code perm.denied + non-empty remediation_hint, writing nothing
  GIVEN: the AC-1 config; BUT_AGENT_HANDLE=rust-implementer (no administration:write); working-tree captured byte-for-byte
  WHEN:  exposed perm_grant_cmd(ctx, target_ref, "rust-implementer", ["administration:write"]) (self-grant attempt)
  THEN:  Err whose json::Error has code "perm.denied" + NON-EMPTY remediation_hint (message names
         administration:write); working-tree permissions.toml byte-for-byte UNCHANGED
  TEST_TIER: integration   VERIFICATION_SERVICE: but-api
  VERIFY: cargo test -p but-api governance_api_perm_grant_non_admin_denied_with_hint

AC-3: governance_status_read returns the CALLER's own effective AuthoritySet (own ∪ groups), self-scoped
  GIVEN: committed config where rust-implementer has own contents:write and is a member of group eng
         (pull_requests:write); BUT_AGENT_HANDLE=rust-implementer
  WHEN:  exposed governance_status_read(ctx) (no principal arg; resolves the workspace target ref from ctx)
  THEN:  the AuthoritySet contains BOTH contents:write (own) AND pull_requests:write (inherited from eng),
         union at the target ref; Ok; reflects BUT_AGENT_HANDLE identity, not an agent-supplied claim
  TEST_TIER: integration   VERIFICATION_SERVICE: but-api
  VERIFY: cargo test -p but-api governance_api_status_read_returns_own_effective_set

AC-4: An exposed group mutation (group_add_member_cmd) by a non-admin is denied with a hint, writing nothing
  GIVEN: committed config with [[group]] eng, admin (administration:write), rust-reviewer (no admin);
         BUT_AGENT_HANDLE=rust-reviewer; working-tree captured byte-for-byte
  WHEN:  exposed group_add_member_cmd(ctx, target_ref, "eng", "rust-reviewer") (self-add escalation attempt)
  THEN:  Err json::Error code "perm.denied", message names administration:write, remediation_hint non-empty;
         working-tree permissions.toml byte-for-byte UNCHANGED (the group path composes the same gate)
  TEST_TIER: integration   VERIFICATION_SERVICE: but-api
  VERIFY: cargo test -p but-api governance_api_group_add_member_non_admin_denied_with_hint

AC-5: governance_status_read is self-scoped — it cannot read a FOREIGN principal because its generated signature takes NO principal arg
  GIVEN: committed config with rust-implementer (contents:write) and a DISTINCT principal admin
         (administration:write); BUT_AGENT_HANDLE=rust-implementer
  WHEN:  governance_status_read(ctx) is invoked (the only identity input is BUT_AGENT_HANDLE; there is no
         parameter through which a caller could request admin's set)
  THEN:  the returned AuthoritySet reflects rust-implementer (contains contents:write) and does NOT contain
         administration:write (admin's foreign authority is never returned); AND a structural assertion proves
         the governance_status_read command signature exposes NO principal/handle/subject parameter
  TEST_TIER: integration   VERIFICATION_SERVICE: but-api
  VERIFY: cargo test -p but-api governance_api_status_read_is_self_scoped_no_foreign_principal

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): admin perm_grant_cmd via the wrapper returns Ok and the working-tree file contains reviews:write
- TC-2 (-> AC-1): the admin perm_grant_cmd result's SERIALIZED GrantOutcome.caveat contains "takes effect once committed to the target branch" and ref_id(main) is unchanged
- TC-3 (-> AC-2): non-admin perm_grant_cmd returns Err whose serialized json::Error has code perm.denied + non-empty remediation_hint naming administration:write
- TC-4 (-> AC-2): after the denied non-admin perm_grant_cmd the working-tree file is byte-for-byte unchanged
- TC-5 (-> AC-3): governance_status_read returns own contents:write + inherited pull_requests:write
- TC-6 (-> AC-3): governance_status_read returns a union set of at least two distinct authorities
- TC-7 (-> AC-4): non-admin group_add_member_cmd returns Err with serialized code perm.denied + non-empty remediation_hint
- TC-8 (-> AC-4): after the denied non-admin group_add_member_cmd the working-tree file is byte-for-byte unchanged
- TC-9 (-> AC-2): the AUTHZ-007/008 honesty grep (invariant_build_gates) NON-VACUOUSLY covers governance.rs — assert governance.rs IS in ENFORCEMENT_PATHS (grep matches the file, not merely "stays green" on an empty match) AND no role-name branching was introduced
- TC-10 (-> AC-5): governance_status_read returns rust-implementer's set WITHOUT administration:write, and a structural assertion proves the command signature has NO principal/handle/subject parameter
  (all VERIFY commands in the REQUIREMENT-CONTRACT below)

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides: a NEW thin #[but_api] wrapper layer exposing the Sprint-05 governance fns (ctx-resolution + transport,
          delegating to the un-forked &repo fns) + self-scoped governance_status_read + a #[derive(Serialize)] GrantOutcome
consumes: but_api::legacy::governance::{perm_*, group_*} (Sprint-05, &gix::Repository + target_ref — DELEGATED to);
          but_api_macros::but_api (REQUIRES a Context param, lib.rs:560);
          but_ctx::Context (repo + workspace target-ref resolution);
          but_authz::{effective_authority, resolve_principal_from_env, load_governance_config, AuthoritySet};
          but_api::json::Error (with the MGMT-IPC-002 fix); but_testsupport::{writable_scenario}; serial_test; temp_env
boundary_contracts:
  - CAP-AUTHZ-01: every governed write flows through the SAME but-authz authorize()/admin gate the Sprint-05
    fns compose; the #[but_api] wrapper adds ctx-resolution + transport ONLY, never a bypass; governance_status_read is self-scoped.
  - Identity-layering: this task proves the gate under the BUT_AGENT_HANDLE identity model; the desktop
    fleet-owner identity (UserService, T-MGMT-042) is wired + re-tested in MGMT-IPC-003 (a layered boundary, not a contradiction).

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-api/src/legacy/governance.rs (MODIFY — Sprint-05-owned, EXTENDED) — ADD the new #[but_api]
    wrapper fns (*_cmd(ctx, ..)) that resolve &repo + target ref from ctx and DELEGATE to the existing
    Sprint-05 &repo perm_*/group_* fns; ADD self-scoped governance_status_read(ctx); ADD #[derive(Serialize)]
    GrantOutcome. (Authoring NEW wrapper code + return type is permitted; editing the Sprint-05 fn BODIES is NOT.)
  - crates/but-api/tests/governance_api.rs (NEW) — AC-1..AC-5 proofs (real but-api + real but-authz + real gix)
writeProhibited:
  - the BODIES + the &repo+target_ref SIGNATURES of the Sprint-05 perm_*/group_* fns — DELEGATE/expose only;
    do not fork/weaken the AUTHZ-006 gate, do not rewrite their signatures
  - crates/but-api/src/legacy/config_mutate.rs (AUTHZ-006 guard consumed, not modified)
  - crates/but-authz/** (authorize/effective_authority/Denial/load_governance_config closed; consume only)
  - crates/but-authz/tests/invariant_build_gates.rs (the honesty grep — must stay green AND must already cover
    governance.rs via Sprint-05 CLI-001; do not weaken or add the path here)
  - crates/but-api/src/json.rs (the remediation_hint fix is MGMT-IPC-002)
  - the desktop UserService / fleet-owner identity wiring (T-MGMT-042 — MGMT-IPC-003)
  - the Tauri generate_handler!, capabilities, identity wiring, packages/but-sdk regen (MGMT-IPC-003/004)
  - crates/but/** (no governance logic in the CLI crate); any gitbutler-* crate; any file not in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. ../sprint-05-cli-perm-group/CLI-001-perm-cli-verbs.md — planned perm_*/GovWriteError/REF_PIN_CAVEAT + AUTHZ-006 composition + governance.rs added to invariant_build_gates ENFORCEMENT_PATHS (BLOCKED-UNTIL merged)
2. ../sprint-05-cli-perm-group/CLI-002-group-cli-verbs.md — planned group_* signatures (&repo + target_ref) + same admin-gate composition
3. crates/but-api-macros/src/lib.rs:8-71, :555-565 — the #[but_api] macro: func_json/func_cmd/func_napi, Context->project_id, and the HARD requirement that explicit repository-permission params need a Context param (lib.rs:560)
4. crates/but-api/src/platform.rs:22 / github.rs:20 / legacy/absorb.rs:37 — concrete #[but_api]/#[but_api(napi)] Context-param usage to mirror in the wrapper
5. crates/but-ctx/** — Context: how to resolve &gix::Repository + the workspace target ref from ctx (the wrapper's job)
6. .spec/prds/governance/10-technical-requirements/04-api-design.md:80-95 — Tauri command<->CLI verb mapping; governance_status_read self-scoped over effective_authority
7. .spec/prds/governance/10-technical-requirements/08-capability-chains.md:11-24 — CAP-AUTHZ-01 (resolve principal -> authorize; never bypass)
8. crates/but-authz/src/authorize.rs:104-132 — authorize, resolve_principal_from_env, Denial constructors (missing_permission hint "request a reviewed merge or ask a maintainer to grant {name}"); effective_authority
9. crates/but-authz/src/config.rs:24-29,230-256 — load_governance_config(&repo, target_ref) -> GovConfig (read at the target ref)
10. crates/but-api/src/legacy/config_mutate.rs:18-43 — enforce_administration_write_gate (the gate composed inside the delegated Sprint-05 fns)
11. crates/but-authz/tests/invariant_build_gates.rs — the AUTHZ-007/008 ENFORCEMENT_PATHS grep (must NON-VACUOUSLY cover governance.rs)
12. crates/AGENTS.md / crates/but/AGENTS.md — but-api boundary rules; brain/docs/rust/{error-handling,testing,traits-generics}.md

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- cargo test -p but-api governance_api_perm_grant_admin_lands_inert                       -> Exit 0
- cargo test -p but-api governance_api_perm_grant_non_admin_denied_with_hint              -> Exit 0
- cargo test -p but-api governance_api_status_read_returns_own_effective_set              -> Exit 0
- cargo test -p but-api governance_api_group_add_member_non_admin_denied_with_hint        -> Exit 0
- cargo test -p but-api governance_api_status_read_is_self_scoped_no_foreign_principal    -> Exit 0
- cargo test -p but-authz invariant_build_gates                                           -> Exit 0 (honesty grep green AND covers governance.rs)
- cargo clippy -p but-api --all-targets                                                   -> clean
- cargo fmt --check                                                                       -> clean

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: A NEW thin #[but_api] wrapper layer. For each Sprint-05 governance fn, author a *_cmd(ctx, ..) wrapper
  carrying the Context the macro requires; the wrapper resolves &gix::Repository + the workspace target ref from
  ctx, then DELEGATES to the un-forked Sprint-05 &repo fn (bodies untouched). The macro generates
  func_json/func_cmd/func_napi -> Result<.., json::Error>. Plus one new self-scoped read fn
  governance_status_read(ctx) composing resolve_principal_from_env + load_governance_config + effective_authority
  (NO principal arg). The admin perm_grant_cmd returns a #[derive(Serialize)] GrantOutcome whose caveat field
  carries the ref-pin caveat as a returned value. Denials flow as anyhow->json::Error carrying the Denial.
pattern_source: crates/but-api/src/legacy/absorb.rs:37 (#[but_api(napi)] Context-param wrapper over a legacy/* fn);
  platform.rs:22 / github.rs:20; admin-gate composition from config_mutate.rs:18-43 as used by Sprint-05 governance.rs;
  MergeGateError (merge_gate.rs:19-37) as the #[derive(Serialize)] returned-outcome shape
anti_pattern: applying #[but_api] in place to the &repo fns (won't compile — no Context param); editing the Sprint-05
  fn bodies/signatures; forking a parallel governance impl; re-implementing/weakening the AUTHZ-006 gate; a 2nd
  authorize() in the wrapper; governance_status_read accepting a principal arg or reading another principal's set
  (cross-principal leak); returning the ref-pin caveat as a printed/log side-effect instead of a serialized value;
  role-name branching; wiring the desktop UserService identity here; burying logic in crates/but/; stubbing the
  Sprint-05 fns to unblock; registering generate_handler!/capability/SDK here.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — but-api macro convention, Context->repo/target-ref resolution, but-authz composition, anyhow/Denial transport
reviewer: rust-reviewer
coding_standards: crates/AGENTS.md, crates/but/AGENTS.md, brain/docs/rust/{traits-generics,error-handling,testing}.md

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: MGMT-IPC-002 (hint transport); Sprint-05 CLI-001 + CLI-002 (author governance.rs + add it to
            invariant_build_gates ENFORCEMENT_PATHS — BLOCKED-UNTIL merged)
Blocks:     MGMT-IPC-003 (registers the wrapper commands + wires/re-tests the desktop UserService identity), MGMT-IPC-004
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "MGMT-IPC-001",
  "proposed_by": "rust-planner",
  "verification_policy": { "requires_tests": true, "requires_red_evidence": true, "requires_seeded_evidence": true },
  "fixtures": {
    "committed_perms_admin_and_impl": { "description": "Real git repo via but_testsupport::writable_scenario; refs/heads/main carries a committed .gitbutler/permissions.toml where admin holds administration:write and rust-implementer holds [\"contents:write\"]. Seeded via a real entrypoint (invoke_bash writes + git-commits the config at main).", "seed_method": "cli", "records": ["admin = administration:write", "rust-implementer = [contents:write]", "committed at refs/heads/main; working tree clean"] },
    "committed_perms_with_group_membership": { "description": "Real git repo; committed .gitbutler/permissions.toml where rust-implementer has own contents:write AND is a member of [[group]] eng (permissions=[pull_requests:write]). Seeded via invoke_bash + git commit at main.", "seed_method": "cli", "records": ["rust-implementer own = contents:write", "[[group]] eng permissions = [pull_requests:write], members = [rust-implementer]", "committed at refs/heads/main"] },
    "committed_perms_with_group_eng": { "description": "Real git repo; committed .gitbutler/permissions.toml with admin (administration:write), a [[group]] eng, and rust-reviewer (no administration:write). Seeded via invoke_bash + git commit at main.", "seed_method": "cli", "records": ["admin = administration:write", "[[group]] eng", "rust-reviewer = no administration:write", "committed at refs/heads/main"] },
    "committed_perms_two_distinct_principals": { "description": "Real git repo; committed .gitbutler/permissions.toml with rust-implementer (own contents:write) and a DISTINCT principal admin (administration:write). Used to prove governance_status_read cannot return admin's foreign authority. Seeded via invoke_bash + git commit at main.", "seed_method": "cli", "records": ["rust-implementer own = contents:write", "admin = administration:write (distinct principal)", "committed at refs/heads/main"] }
  },
  "requirements": [
    { "id": "AC-1", "type": "acceptance_criterion", "primary": true, "description": "GIVEN a committed config (admin=administration:write, rust-implementer=[contents:write]), BUT_AGENT_HANDLE=admin WHEN the new #[but_api] wrapper perm_grant_cmd (which resolves &repo + target ref from ctx and delegates to the Sprint-05 &repo fn) adds reviews:write THEN Ok(GrantOutcome), the working-tree file gains reviews:write, the SERIALIZED GrantOutcome.caveat carries the ref-pin caveat, and refs/heads/main is unmoved (inert)", "verify": "cargo test -p but-api governance_api_perm_grant_admin_lands_inert", "scenario": { "id": "AC-1", "primary": true, "tier": "visible", "test_tier": "integration", "verification_service": "but-api", "negative_control": { "would_fail_if": ["the wrapper bypasses but-authz / forks a parallel implementation that omits the admin gate instead of delegating to the Sprint-05 fn", "the wrapper is a stub that returns Ok without writing the token (working-tree file unchanged)", "the ref-pin caveat is a printed/log side-effect (hardcoded empty / omitted from the serialized GrantOutcome)", "the wrapper commits the grant (ref_id changes) instead of leaving it inert"] }, "evidence": { "artifact_type": "api_response", "required_capture": true }, "cases": [ { "start_ref": "committed_perms_admin_and_impl", "action": { "actor": "ci", "steps": ["temp_env BUT_AGENT_HANDLE=admin under #[serial_test::serial]", "capture ref_id(refs/heads/main) before", "invoke exposed perm_grant_cmd(ctx, \"refs/heads/main\", \"rust-implementer\", [\"reviews:write\"]) — wrapper resolves &repo+target ref from ctx, delegates to the Sprint-05 fn", "serialize the GrantOutcome via the real json path; read back working-tree permissions.toml + ref_id after"] }, "end_state": { "must_observe": ["the call returns `Ok` with a `GrantOutcome`", "the working-tree `.gitbutler/permissions.toml` contains `\"reviews:write\"` under `rust-implementer`", "the serialized `GrantOutcome.caveat` contains `\"takes effect once committed to the target branch\"`", "`ref_id(refs/heads/main)` is identical before and after"], "must_not_observe": ["the working-tree file unchanged with no reviews:write added (stub no-op)", "refs/heads/main moved (the grant was wrongly committed)", "the caveat absent from the serialized GrantOutcome (it was only printed, not returned)"] } } ] } },
    { "id": "AC-2", "type": "acceptance_criterion", "primary": false, "description": "GIVEN BUT_AGENT_HANDLE=rust-implementer (non-admin) WHEN the wrapper perm_grant_cmd attempts a self-grant of administration:write THEN Err json::Error code perm.denied + non-empty remediation_hint naming administration:write, and the working-tree file is byte-for-byte unchanged", "verify": "cargo test -p but-api governance_api_perm_grant_non_admin_denied_with_hint", "scenario": { "id": "AC-2", "primary": false, "tier": "visible", "test_tier": "integration", "verification_service": "but-api", "negative_control": { "would_fail_if": ["the wrapper bypasses but-authz / the admin gate is not composed (the grant succeeds for a non-admin)", "the wrapper returns Ok for a non-admin (stub / no gate)", "the remediation_hint is dropped/empty in transport (the MGMT-IPC-002 dependency is unmet)", "the writer ran before the gate so the working-tree file changed (fail-open ordering)"] }, "evidence": { "artifact_type": "api_response", "required_capture": true }, "cases": [ { "start_ref": "committed_perms_admin_and_impl", "action": { "actor": "ci", "steps": ["capture working-tree permissions.toml bytes", "temp_env BUT_AGENT_HANDLE=rust-implementer (non-admin)", "invoke exposed perm_grant_cmd(ctx, \"refs/heads/main\", \"rust-implementer\", [\"administration:write\"])", "serialize the error via the real json::Error path; re-read the file bytes"] }, "end_state": { "must_observe": ["the serialized error JSON has `\"code\":\"perm.denied\"`", "the `\"remediation_hint\"` value is non-empty (length > 0)", "the error message contains `\"administration:write\"`", "the working-tree `.gitbutler/permissions.toml` bytes are byte-identical (`==`) to the pre-call capture"], "must_not_observe": ["the call returning `Ok` for a non-admin", "a `remediation_hint` absent or empty in the serialized error", "the working-tree file changed after a denied non-admin grant"] } } ] } },
    { "id": "AC-3", "type": "acceptance_criterion", "primary": false, "description": "GIVEN rust-implementer with own contents:write and group-inherited pull_requests:write from eng, BUT_AGENT_HANDLE=rust-implementer WHEN governance_status_read(ctx) (no principal arg) is invoked THEN it returns the caller's own ∪ group union (contents:write + pull_requests:write), self-scoped, target ref resolved from ctx", "verify": "cargo test -p but-api governance_api_status_read_returns_own_effective_set", "scenario": { "id": "AC-3", "primary": false, "tier": "visible", "test_tier": "integration", "verification_service": "but-api", "negative_control": { "would_fail_if": ["governance_status_read returns a hardcoded/static AuthoritySet instead of computing effective_authority", "it omits the group-inherited pull_requests:write (union not computed)", "it returns an empty set (stub) for a principal that holds grants", "it reads another principal's set instead of the caller's (not self-scoped)"] }, "evidence": { "artifact_type": "api_response", "required_capture": true }, "cases": [ { "start_ref": "committed_perms_with_group_membership", "action": { "actor": "ci", "steps": ["temp_env BUT_AGENT_HANDLE=rust-implementer", "invoke exposed governance_status_read(ctx) — resolves the workspace target ref + caller principal from ctx/env", "inspect the returned AuthoritySet"] }, "end_state": { "must_observe": ["the returned AuthoritySet contains `contents:write`", "the returned AuthoritySet contains `pull_requests:write` (inherited from group `eng`)", "the set has at least 2 distinct authorities (own ∪ group)", "the call returns `Ok`"], "must_not_observe": ["an empty AuthoritySet for a principal that holds grants", "pull_requests:write absent (group union not computed)"] } } ] } },
    { "id": "AC-4", "type": "acceptance_criterion", "primary": false, "description": "GIVEN BUT_AGENT_HANDLE=rust-reviewer (non-admin) WHEN the wrapper group_add_member_cmd attempts a self-add to eng THEN Err json::Error code perm.denied naming administration:write + non-empty remediation_hint, and the working-tree file is byte-for-byte unchanged (group path composes the same gate)", "verify": "cargo test -p but-api governance_api_group_add_member_non_admin_denied_with_hint", "scenario": { "id": "AC-4", "primary": false, "tier": "visible", "test_tier": "integration", "verification_service": "but-api", "negative_control": { "would_fail_if": ["admin gate not composed for the group path (the self-add succeeds for a non-admin)", "the wrapper group fn bypasses but-authz / forks a parallel implementation without the gate", "remediation_hint dropped/empty in the serialized group denial", "the membership write ran before the gate so the working-tree file changed (fail-open)"] }, "evidence": { "artifact_type": "api_response", "required_capture": true }, "cases": [ { "start_ref": "committed_perms_with_group_eng", "action": { "actor": "ci", "steps": ["capture working-tree permissions.toml bytes", "temp_env BUT_AGENT_HANDLE=rust-reviewer (non-admin)", "invoke exposed group_add_member_cmd(ctx, \"refs/heads/main\", \"eng\", \"rust-reviewer\")", "serialize the error via the real json::Error path; re-read the file bytes"] }, "end_state": { "must_observe": ["the serialized error JSON has `\"code\":\"perm.denied\"`", "the message contains `\"administration:write\"`", "the `\"remediation_hint\"` value is non-empty (length > 0)", "the working-tree `.gitbutler/permissions.toml` bytes are byte-identical (`==`) to the pre-call capture"], "must_not_observe": ["the call returning `Ok` (self-add succeeded for a non-admin)", "the working-tree file changed after a denied group mutation", "a `remediation_hint` absent or empty in the serialized group denial"] } } ] } },
    { "id": "AC-5", "type": "acceptance_criterion", "primary": false, "description": "GIVEN two distinct principals (rust-implementer contents:write, admin administration:write), BUT_AGENT_HANDLE=rust-implementer WHEN governance_status_read(ctx) is invoked THEN the returned set is rust-implementer's (contains contents:write) and NEVER admin's foreign administration:write, AND a structural assertion proves the command signature exposes NO principal/handle/subject parameter (self-scoped by construction)", "verify": "cargo test -p but-api governance_api_status_read_is_self_scoped_no_foreign_principal", "scenario": { "id": "AC-5", "primary": false, "tier": "visible", "test_tier": "integration", "verification_service": "but-api", "negative_control": { "would_fail_if": ["governance_status_read accepts a principal/handle/subject parameter through which a foreign set could be requested", "it returns admin's administration:write while BUT_AGENT_HANDLE=rust-implementer (cross-principal leak)", "it ignores BUT_AGENT_HANDLE and returns a static/hardcoded set", "the signature has a second identity arg removed only at runtime (still present in the generated command)"] }, "evidence": { "artifact_type": "api_response", "required_capture": true }, "cases": [ { "start_ref": "committed_perms_two_distinct_principals", "action": { "actor": "ci", "steps": ["temp_env BUT_AGENT_HANDLE=rust-implementer", "invoke exposed governance_status_read(ctx)", "inspect the returned AuthoritySet", "assert structurally that the governance_status_read command signature takes only ctx (no principal/handle/subject param)"] }, "end_state": { "must_observe": ["the returned AuthoritySet contains `contents:write` (rust-implementer's own)", "the call returns `Ok`", "a structural assertion confirms the command signature has `0` principal/handle/subject parameters (only `ctx`)"], "must_not_observe": ["`administration:write` present in the returned set (admin's foreign authority leaked)", "more than `0` principal/handle/subject parameters on the governance_status_read command signature (it must take none beyond `ctx`)"] } } ] } },
    { "id": "TC-1", "type": "test_criterion", "description": "admin perm_grant_cmd via the wrapper returns Ok and the working-tree permissions.toml contains reviews:write under rust-implementer", "verify": "cargo test -p but-api governance_api_perm_grant_admin_lands_inert", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "the admin perm_grant_cmd result's serialized GrantOutcome.caveat contains the ref-pin caveat and refs/heads/main ref_id is unchanged", "verify": "cargo test -p but-api governance_api_perm_grant_admin_lands_inert", "maps_to_ac": "AC-1" },
    { "id": "TC-3", "type": "test_criterion", "description": "non-admin perm_grant_cmd returns Err whose serialized json::Error has code perm.denied + non-empty remediation_hint naming administration:write", "verify": "cargo test -p but-api governance_api_perm_grant_non_admin_denied_with_hint", "maps_to_ac": "AC-2" },
    { "id": "TC-4", "type": "test_criterion", "description": "after the denied non-admin perm_grant_cmd the working-tree permissions.toml is byte-for-byte unchanged", "verify": "cargo test -p but-api governance_api_perm_grant_non_admin_denied_with_hint", "maps_to_ac": "AC-2" },
    { "id": "TC-5", "type": "test_criterion", "description": "governance_status_read returns an AuthoritySet containing own contents:write and group-inherited pull_requests:write", "verify": "cargo test -p but-api governance_api_status_read_returns_own_effective_set", "maps_to_ac": "AC-3" },
    { "id": "TC-6", "type": "test_criterion", "description": "governance_status_read returns a union set of at least two distinct authorities", "verify": "cargo test -p but-api governance_api_status_read_returns_own_effective_set", "maps_to_ac": "AC-3" },
    { "id": "TC-7", "type": "test_criterion", "description": "non-admin group_add_member_cmd returns Err with serialized code perm.denied + non-empty remediation_hint", "verify": "cargo test -p but-api governance_api_group_add_member_non_admin_denied_with_hint", "maps_to_ac": "AC-4" },
    { "id": "TC-8", "type": "test_criterion", "description": "after the denied non-admin group_add_member_cmd the working-tree permissions.toml is byte-for-byte unchanged", "verify": "cargo test -p but-api governance_api_group_add_member_non_admin_denied_with_hint", "maps_to_ac": "AC-4" },
    { "id": "TC-9", "type": "test_criterion", "description": "the AUTHZ-007/008 honesty grep NON-VACUOUSLY covers governance.rs (governance.rs IS in ENFORCEMENT_PATHS — the grep matches the file, not an empty match) and no role-name branching was introduced in the wrapper", "verify": "cargo test -p but-authz invariant_build_gates", "maps_to_ac": "AC-2" },
    { "id": "TC-10", "type": "test_criterion", "description": "governance_status_read returns rust-implementer's set WITHOUT administration:write, and a structural assertion proves the command signature exposes NO principal/handle/subject parameter (self-scoped by construction)", "verify": "cargo test -p but-api governance_api_status_read_is_self_scoped_no_foreign_principal", "maps_to_ac": "AC-5" }
  ]
}
-->
</details>
