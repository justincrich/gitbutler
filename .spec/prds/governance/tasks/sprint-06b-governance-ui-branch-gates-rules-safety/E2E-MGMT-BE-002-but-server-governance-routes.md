# E2E-MGMT-BE-002: Route governed governance commands through but-server (web-target HTTP surface)

## What this does

Registers 13 `but-server` HTTP routes (`POST /{command}`) in the `Router::new()` table — one per governed governance command — each binding the GOVERNED `legacy::governance::<cmd>_cmd` wrapper via `but_post`, mirroring the existing workspace-rules routes (lib.rs:822-837). Today `but-server` routes **zero** governance commands, so the web-target UI (`webInvoke → POST ${getApiBaseUrl()}/${command}`) cannot reach governance at all. This closes that gap so the capstone Playwright suite can drive the real governance UI against a real backend — with the real `but-authz` gate enforcing `perm.denied` for non-admins.

## Why

Sprint 06b · PRD UC-MGMT-06/07 · capability CAP-AUTHZ-01. The Playwright capstone drives the **web** build target, whose SDK transport is HTTP-to-`but-server`. The governance `#[but_api(napi)]` fns generate Tauri + napi bindings but **no** but-server routes — so without this task, every governance call from the web UI returns "Command not found". This is the backend that makes the capstone reachable. It is also a genuine product gap (web/lite surfaces) — not test-only scaffolding.

## How to verify

PRIMARY **AC-3** (integration, load-bearing) — spawn `but-server` with `BUT_AGENT_HANDLE=NONADMIN_HANDLE` over the E2E-MGMT-BE-001 seed; `POST /branch_gates_update` returns `{type:error, code:"perm.denied"}` and the gate state is unchanged. (A fleet-owner-bypass wiring would succeed → fail.)

## Scope

- `crates/but-server/src/lib.rs` (route registrations in the `Router::new()` table)
- `crates/but-server/src/**` (a new integration test, e.g. `crates/but-server/tests/governance_routes.rs`)
- `crates/but-server/Cargo.toml` (dev-dependencies only, if needed for the test)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: E2E-MGMT-BE-002 — Route governed governance commands through but-server
================================================================================

TASK_TYPE:   FEATURE
STATUS:      Backlog
PRIORITY:    P1
EFFORT:      L  (180 min)
AGENT:       rust-implementer
PROPOSED-BY: rust-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-MGMT-06, UC-MGMT-07, 11-e2e-testing-criteria.md#T-MGMT-032, #T-MGMT-041
CAPABILITIES: CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-server governance_routes
  check: cargo build -p but-server
  lint:  cargo clippy -p but-server --all-targets

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
13 governed governance routes are registered in the Router::new() table; cargo build -p but-server
resolves all 13 *_cmd symbols; a server spawned with BUT_AGENT_HANDLE=ADMIN_HANDLE returns real
GovernanceStatus/BranchGatesOutcome for reads and persists branch_gates_update; a server spawned with
BUT_AGENT_HANDLE=NONADMIN_HANDLE returns {type:error, code:perm.denied} for branch_gates_update and
perm_grant with state unchanged; a write with a mismatched target_ref returns the structured
"does not match workspace target" error before the gate; grep finds no _as_fleet_owner under
crates/but-server/src.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] Register routes ONLY for the 13 GOVERNED commands, each as
  `.route("/<command>", but_post(legacy::governance::<command>_cmd))` in the Router::new() table,
  mirroring the workspace-rules exemplar at crates/but-server/src/lib.rs:822-837.
- [MUST] The 13 commands: reads = governance_status_read, group_list, perm_list, branch_gates_read;
  writes = perm_grant, perm_revoke, group_create, group_grant, group_revoke, group_add_member,
  group_remove_member, group_delete, branch_gates_update.
- [MUST] Route paths match the command strings the web UI sends verbatim (webInvoke posts to
  `${getApiBaseUrl()}/${command}`, apps/desktop/src/lib/backend/web.ts:186-188).
- [MUST] Verify each <command>_cmd signature composes with but_post (Value -> anyhow::Result<Value>);
  json::Error -> anyhow via `?` inside the macro body — no hand-rolled adapter.
- [MUST] AC-5 asserts `cargo build -p but-server` resolves all 13 *_cmd symbols (the macro emits them
  under #[cfg(feature="legacy")], which but-server enables in Cargo.toml:22 — a compile-time guard
  against a future feature-flag regression).
- [NEVER] NEVER route a *_as_fleet_owner variant — they skip the gate (a non-admin write would SUCCEED,
  a false pass). They also lack the #[but_api] macro, so they have no *_cmd wrapper and are
  structurally unroutable via but_post — keep it that way.
- [NEVER] NEVER add backend code for step-6 fault injection — step 6 is pure Playwright page.route() 500
  (owned by the UI task).
- [NEVER] NEVER introduce a per-request identity mechanism — process-level BUT_AGENT_HANDLE is the
  contract; per-request switching is out of scope and, if ever added, MUST still route through
  but_authz and never fleet-owner.
- [NEVER] NEVER register routes in a handle_command match / catch-all path — use the Router::new() table.
- [NEVER] NEVER weaken the target_ref contract (a write with target_ref != workspace target must be
  rejected before the gate, governance.rs:957-967).
- [STRICTLY] The structural no-bypass guarantee (no _cmd wrapper for fleet-owner fns) is the primary
  defense; the grep guard is a secondary belt; the load-bearing behavioral proof is AC-3.
- [STRICTLY] AC-2 must re-read after the admin write and assert the change persisted (round-trip).

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1: read routes reachable as admin → real GovernanceStatus/BranchGatesOutcome (master protected==true) ← PARTIAL: generated routes are present, but the route test exercises `/group_list`, not `/governance_status_read` or `/branch_gates_read` with the seeded protected master oracle (evidence: crates/but-server/src/lib.rs:76, crates/but-server/src/lib.rs:84, crates/but-server/src/lib.rs:1681)
- [ ] AC-2: branch_gates_update as admin → success + round-trips on re-read ← PARTIAL: generated `/branch_gates_update` route is registered, but the route round-trip test uses `/perm_grant` + `/perm_list`, not `branch_gates_update` + `branch_gates_read` (evidence: crates/but-server/src/lib.rs:80, crates/but-server/src/lib.rs:1701, crates/but-server/src/lib.rs:1723)
- [ ] AC-3 [PRIMARY]: branch_gates_update as non-admin → {type:error, code:perm.denied}; gates unchanged ← PARTIAL: production handler enforces administration:write, but the route denial test does not call `/branch_gates_update` or verify gates remain unchanged (evidence: crates/but-api/src/legacy/governance.rs:548, crates/but-api/src/legacy/governance.rs:554, crates/but-server/src/lib.rs:1744)
- [x] AC-4: non-admin self-grant perm_grant → perm.denied; grep finds zero _as_fleet_owner in but-server
- [ ] AC-5: cargo build/clippy pass; all 13 *_cmd resolve under feature="legacy"; all 13 routes present ← PARTIAL: the route table compiles and lists the governance routes, but clippy was not run in this review and the static test now covers 16 route entries rather than the task's exact 13-command contract (evidence: crates/but-server/src/lib.rs:34, crates/but-server/src/lib.rs:1645)
- [ ] AC-6: write with target_ref != workspace target rejected before the gate; matching target_ref reaches the gate ← PARTIAL: target mismatch is tested through `/branch_gates_read`, not a write, and the test never proves a matching write reaches the gate/succeeds (evidence: crates/but-api/src/legacy/governance.rs:1640, crates/but-server/src/lib.rs:1797)

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each with a real start→end scenario; integration tier)
--------------------------------------------------------------------------------
AC-1 — Governance reads reachable as admin, reflecting the seed
  GIVEN a but-server spawned with BUT_AGENT_HANDLE=ADMIN_HANDLE over the BE-001-seeded repo
  WHEN  the client POSTs {projectId,...} to /governance_status_read and /branch_gates_read
  THEN  branch_gates_read returns a BranchGatesOutcome whose master entry has protected==true (the seeded
        value); governance_status_read returns the admin AuthoritySet incl. administration:write — NOT "Command not found"
  VERIFY cargo test -p but-server governance_routes::reads_admin
  TEST_TIER integration   VERIFICATION_SERVICE real but-server route → legacy::governance reads over the committed config
  NEGATIVE CONTROL would fail if: routes unregistered (pre-routing "Command not found"); stub returns canned data without the seeded master entry.

AC-2 — Admin branch_gates_update persists (round-trip)
  GIVEN the AC-1 admin server
  WHEN  the client POSTs /branch_gates_update to change a gate, then POSTs /branch_gates_read
  THEN  the update returns a non-error success body and the re-read reflects the change
  VERIFY cargo test -p but-server governance_routes::write_admin_roundtrip
  TEST_TIER integration   VERIFICATION_SERVICE real but-server route → branch_gates_update + re-read
  NEGATIVE CONTROL would fail if: no-op stub (re-read shows pre-update state); fleet-owner variant routed (caught by AC-3/AC-4).

AC-3 [PRIMARY] — Non-admin branch_gates_update is denied and inert (no-bypass proof)
  GIVEN a but-server spawned with BUT_AGENT_HANDLE=NONADMIN_HANDLE over the same seeded repo
  WHEN  the client POSTs /branch_gates_update {branch:"master", protection:{protected:false}}
  THEN  the response is {type:error} with code perm.denied AND a follow-up branch_gates_read shows the
        gate state unchanged by the denied write
  VERIFY cargo test -p but-server governance_routes::write_nonadmin_denied
  TEST_TIER integration   VERIFICATION_SERVICE real but-server route → governed gate (enforce_administration_write_gate)
  NEGATIVE CONTROL would fail if: route bound branch_gates_update_with_repo_as_fleet_owner (write succeeds, no denial — also structurally impossible: no _cmd wrapper); dev mis-seeded with admin:write.

AC-4 — Non-admin self-grant denied; no fleet-owner reachable
  GIVEN the non-admin server
  WHEN  the client POSTs /perm_grant attempting to grant self administration:write; AND the source tree is grepped
  THEN  perm_grant returns {type:error} code perm.denied, a follow-up perm_list shows admin:write NOT added,
        and `grep -rq "_as_fleet_owner" crates/but-server/src` finds nothing
  VERIFY cargo test -p but-server governance_routes::self_grant_denied && ! grep -rq "_as_fleet_owner" crates/but-server/src
  TEST_TIER integration   VERIFICATION_SERVICE real but-server (governed perm_grant) + source-tree structural guard
  NEGATIVE CONTROL would fail if: any write route bound a *_as_fleet_owner variant (self-grant succeeds); fleet-owner symbol present in but-server.

AC-5 — Build + route coverage under the legacy feature
  GIVEN the edited lib.rs
  WHEN  cargo build -p but-server and cargo clippy -p but-server --all-targets run
  THEN  both succeed; all 13 legacy::governance::<cmd>_cmd paths resolve (they exist only under
        feature="legacy", which but-server enables) and all 13 routes are registered in the Router table
  VERIFY cargo build -p but-server && cargo clippy -p but-server --all-targets
  TEST_TIER integration   VERIFICATION_SERVICE cargo (compile-time symbol resolution)
  NEGATIVE CONTROL would fail if: the "legacy" feature were dropped from but-server's Cargo.toml (the 13 *_cmd would not exist → build fails); a command omitted/misspelled.

AC-6 — target_ref mismatch rejected before the gate
  GIVEN the admin server
  WHEN  the client POSTs /branch_gates_update with target_ref != workspace target, then with the matching target_ref
  THEN  the mismatched request returns the structured "requested target ref … does not match workspace target"
        error BEFORE the gate (no mutation); the matching request reaches the gate (admin → success)
  VERIFY cargo test -p but-server governance_routes::target_ref_contract
  TEST_TIER integration   VERIFICATION_SERVICE real but-server route → target_ref_from_ctx (governance.rs:957-967)
  NEGATIVE CONTROL would fail if: target_ref_from_ctx bypassed (mismatched write proceeds/mutates); matching target_ref wrongly rejected.

--------------------------------------------------------------------------------
GUARDRAILS
--------------------------------------------------------------------------------
WRITE-ALLOWED:
  - crates/but-server/src/** (route registrations in the Router table + an integration test)
  - crates/but-server/Cargo.toml (dev-dependencies only, if the integration test needs them)
WRITE-PROHIBITED:
  - crates/but-api/** governance/authz functions (the *_cmd and *_as_fleet_owner surfaces are contracts to bind, not change)
  - crates/but-authz/** (gate logic is a contract to enforce)
  - apps/desktop/** and any UI/SDK code (the UI task owns the client + step-6 page.route)
  - e2e/** (consumed, not modified)

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
- crates/but-server/src/lib.rs:515 — Router::new() route table (registration site)
- crates/but-server/src/lib.rs:822-837 — workspace-rules routes via but_post(legacy::rules::*_cmd) (the exemplar)
- crates/but-server/src/lib.rs:75-86 — but_post: Fn(Value)->anyhow::Result<Value>; confirms *_cmd composes
- crates/but-api-macros/src/lib.rs:347-358 — macro emits <fn>_cmd UNDER #[cfg(feature="legacy")] with from_value(params)?
- crates/but-server/Cargo.toml:22 — but-api features include "legacy" (why the wrappers exist)
- crates/but-api/src/legacy/governance.rs:218-360 — the 13 governed #[but_api(napi)] fns
- crates/but-api/src/legacy/governance.rs:408+ — *_as_fleet_owner fns: NO #[but_api], hence NO _cmd wrapper (unroutable)
- crates/but-api/src/legacy/governance.rs:957-967 — target_ref_from_ctx rejects mismatched target_ref before the gate
- apps/desktop/src/lib/backend/web.ts:186-188,303 — webInvoke posts to ${getApiBaseUrl()}/${command} (the path shape)
- e2e/playwright/src/setup.ts:352-364 — startGitButler env forwarding (process-level identity)

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- cargo build -p but-server                       → all 13 legacy::governance::<cmd>_cmd resolve under feature="legacy"
- cargo clippy -p but-server --all-targets         → clean
- cargo test -p but-server governance_routes       → AC-1..AC-6 over a real seeded repo + spawned router
- grep -rn "_as_fleet_owner" crates/but-server/src → MUST return nothing
- cargo fmt

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
depends_on: E2E-MGMT-BE-001, MGMT-BE-003, MGMT-BE-004
blocks:     E2E-MGMT-UI-001

--------------------------------------------------------------------------------
NOTES
--------------------------------------------------------------------------------
- UPSTREAM ADVISORY U1 (escalation, NOT resolved here): SPRINT.md step-4 wording ("As an admin … denial")
  is unprovable as written — an admin HOLDS administration:write so a governed self-grant PASSES the gate
  (stages pending, no denial); the Tauri product resolves the human as fleet-owner superuser (R12), also
  no denial. A real perm.denied arises ONLY for a non-admin (dev) actor. The capstone proves the NON-admin
  self-grant denial. Recommend rewording SPRINT.md step 4. Do NOT edit the locked SPRINT.md.
- Identity model (process-level BUT_AGENT_HANDLE per spawned server) → the capstone groups steps by
  identity into separate test() blocks (ADMIN: 1,2,5,6; NON-ADMIN: 3,4). Per-request switching out of scope.
- Structural no-bypass: *_as_fleet_owner fns lack #[but_api] so have no _cmd wrapper → unroutable via but_post.
  The grep guard (AC-4) is a secondary belt; AC-3 is the load-bearing proof.
- Step-6 fault injection needs NO backend change — pure Playwright page.route('**/branch_gates_update', …500).
  Endpoint pattern: http://{butlerHost}:{butlerPort}/{command}.
- Verified against live code on this branch: lib.rs:515/822-837/75-86, Cargo.toml:22, but-api-macros
  lib.rs:347-358, governance.rs:218-360/408+/957-967, web.ts:186-188.
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "requirements": [
    { "id": "AC-1", "type": "acceptance_criterion", "description": "Governance read routes (governance_status_read, branch_gates_read) reachable as admin return real data reflecting the seed (master protected==true); not 'Command not found'", "verify": "cargo test -p but-server governance_routes::reads_admin", "test_tier": "integration" },
    { "id": "AC-2", "type": "acceptance_criterion", "description": "Admin branch_gates_update returns success and the change round-trips on a follow-up branch_gates_read", "verify": "cargo test -p but-server governance_routes::write_admin_roundtrip", "test_tier": "integration" },
    { "id": "AC-3", "type": "acceptance_criterion", "description": "Non-admin branch_gates_update returns {type:error, code:perm.denied} and leaves gate state unchanged (load-bearing no-bypass proof)", "verify": "cargo test -p but-server governance_routes::write_nonadmin_denied", "test_tier": "integration", "primary": true },
    { "id": "AC-4", "type": "acceptance_criterion", "description": "Non-admin self-grant perm_grant returns perm.denied with no authority added; grep finds no _as_fleet_owner under crates/but-server/src", "verify": "cargo test -p but-server governance_routes::self_grant_denied && ! grep -rq '_as_fleet_owner' crates/but-server/src", "test_tier": "integration" },
    { "id": "AC-5", "type": "acceptance_criterion", "description": "cargo build/clippy -p but-server pass; all 13 *_cmd resolve under feature=legacy; all 13 governance routes registered in the Router table", "verify": "cargo build -p but-server && cargo clippy -p but-server --all-targets", "test_tier": "integration" },
    { "id": "AC-6", "type": "acceptance_criterion", "description": "A write with target_ref != workspace target is rejected before the gate (governance.rs:957-967); a matching target_ref reaches the gate", "verify": "cargo test -p but-server governance_routes::target_ref_contract", "test_tier": "integration" },
    { "id": "TC-1", "type": "test_criterion", "description": "Routed reads return real payloads reflecting the BE-001 seed (master protected==true); pre-routing returned 'Command not found'", "verify": "cargo test -p but-server governance_routes::reads_admin", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "Routed admin branch_gates_update succeeds and round-trips via a follow-up read", "verify": "cargo test -p but-server governance_routes::write_admin_roundtrip", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "Routed non-admin branch_gates_update returns {type:error, code:perm.denied} and leaves state unchanged", "verify": "cargo test -p but-server governance_routes::write_nonadmin_denied", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "Routed non-admin self-grant perm_grant returns perm.denied with no authority added; no _as_fleet_owner under but-server and none bindable (no _cmd wrapper)", "verify": "cargo test -p but-server governance_routes::self_grant_denied && ! grep -rq '_as_fleet_owner' crates/but-server/src", "maps_to_ac": "AC-4" },
    { "id": "TC-5", "type": "test_criterion", "description": "cargo build/clippy pass; all 13 *_cmd resolve under feature=legacy and all 13 routes registered", "verify": "cargo build -p but-server && cargo clippy -p but-server --all-targets", "maps_to_ac": "AC-5" },
    { "id": "TC-6", "type": "test_criterion", "description": "A write with a mismatched target_ref returns the structured mismatch error before the gate; a matching target_ref reaches the gate", "verify": "cargo test -p but-server governance_routes::target_ref_contract", "maps_to_ac": "AC-6" }
  ]
}
-->
