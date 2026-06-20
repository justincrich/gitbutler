# MGMT-IPC-003: Register governance commands in `generate_handler!` + capability scope + v1 human-fleet-owner identity (T-MGMT-042)

> **Red-Hat Remediation (cycle 1):** resolves T1, SEC1, R3, T2, T7, T3. The v1 fleet-owner authz model is now explicit (UNCONDITIONAL `administration:write` superuser bypass — the R12 accepted risk — NOT a `permissions.toml` lookup); the identity-substitution seam is named with a NEVER-fall-through-to-`resolve_principal_from_env` constraint + a non-admin-`BUT_AGENT_HANDLE`-does-not-shadow negative AC; the IPC-001/IPC-003 identity layering is disambiguated; MGMT-IPC-002 is added to `depends_on` with AC-5 asserting a non-empty `remediation_hint`; AC-2 is reframed as necessary-not-sufficient capability admission; and api-design.md:80's `allow-*` language is recorded as a superseded upstream advisory.

## What this does

Registers the 12 MGMT-IPC-001 governance `#[but_api]` commands (`perm_list`/`perm_grant`/`perm_revoke`/`group_create`/`group_grant`/`group_add_member`/`group_remove_member`/`group_delete`/`group_list`/`branch_gates_read`/`branch_gates_update`/`governance_status_read`) in `gitbutler-tauri`'s `tauri::generate_handler!`, confirms each resolves under the existing `main` capability scope, and wires the v1 acting principal for MGMT config-management commands to the **human fleet-owner** resolved from the signed-in desktop session (`legacy::users::get_user`, T-MGMT-042). It does **not** author the `#[but_api]` fns or the but-authz gate — it registers and capability-binds them (the canonical atomic command + permission rule).

### v1 fleet-owner authz model (R12 accepted-risk superuser path) — read before implementing

- **[MUST] The Tauri command-boundary identity helper grants the resolved human fleet-owner UNCONDITIONAL `administration:write` trust.** This is a deliberate superuser bypass (the R12 accepted risk): the fleet-owner's `administration:write` is asserted at the desktop command boundary and is **NOT** routed through `but-authz`'s `permissions.toml` lookup (`resolve_principal_from_env` / `principal_authorities`). The signed-in desktop human IS the fleet-owner; v1 has no UserService-principal-in-`permissions.toml` path. A correct implementation grants the fleet-owner even when **no** `.gitbutler/permissions.toml` is committed (bootstrap) — see AC-6. This falsifies any "look up the fleet-owner in `permissions.toml`" implementation (that would deny on a bootstrap repo).
- **[NOTE] The "UI is never a bypass" invariant binds AGENTS, not the human fleet-owner.** Agents (resolved from `BUT_AGENT_HANDLE`, looked up in `permissions.toml`) stay fully bound by their functional permissions and are denied by the server-side `but-authz` gate when they lack `administration:write`. The fleet-owner superuser path is the single sanctioned exception (R12), scoped to the desktop human at the command boundary — it does NOT loosen the agent path.
- **[SEC] Identity-substitution seam (named).** A thin `gitbutler-tauri`-side helper (e.g. `fleet_owner_context` / `with_fleet_owner_identity`) injects the fleet-owner identity into the `Context`/principal resolution **before** the but-api fn would otherwise reach `resolve_principal_from_env`. **[NEVER] The desktop governance command must NEVER fall through to `resolve_principal_from_env`** (which returns `Denial::no_handle()` when `BUT_AGENT_HANDLE` is unset — authorize.rs:71-72) — the desktop path is the fleet-owner path, not the env-handle path. A non-admin agent handle present in `BUT_AGENT_HANDLE` must NOT shadow the fleet-owner (AC-7).
- **[NOTE] Identity layering (IPC-001 vs IPC-003).** MGMT-IPC-001 proved the but-api `administration:write` gate **decision** under `BUT_AGENT_HANDLE` (the agent/env path). MGMT-IPC-003 wires AND re-tests the **desktop fleet-owner identity** (T-MGMT-042) at the Tauri boundary — a distinct layer from IPC-001's env-handle gate decision; the two are not the same proof.

## Why

Sprint 06a · PRD 04-api-design (Tauri command surface), UC-MGMT-06 (T-MGMT-042) · capability CAP-AUTHZ-01. Makes the governance fns invokable through the real desktop command bus so MGMT-IPC-004 can regenerate the SDK and the UI tasks can call them — while keeping server-side `but-authz` as the enforcement boundary for AGENTS (the UI is never an agent bypass; the human fleet-owner is the R12 sanctioned superuser).

## How to verify

PRIMARY **AC-1** — `cargo test -p gitbutler-tauri mgmt_governance_commands_registered_and_invokable`: all 12 governance command symbols are present in `generate_handler!` and `perm_grant` invoked through the real command bus reaches its but-api fn (returns `Ok` or a structured `Denial`, never command-not-found). Full gate set in the spec below.

## Scope

- `crates/gitbutler-tauri/src/main.rs` (MODIFY) — add the 12 governance command rows to `generate_handler!`.
- `crates/gitbutler-tauri/capabilities/main.json` (MODIFY only if a scope note requires; reuse the existing `main` capability).
- `crates/gitbutler-tauri/src/*` (the named fleet-owner identity-substitution shim — `fleet_owner_context` / `with_fleet_owner_identity` — if MGMT-IPC-001 does not already accept an injected identity).
- `crates/gitbutler-tauri/tests/*` (NEW) — AC-1..AC-7 integration proofs.

> **Capability false-friend (verified against live code).** GitButler does **not** emit per-command `allow-<cmd>.toml` permission files for its own app commands — the entire app-command surface rides on `core:default` under the `main` capability (`crates/gitbutler-tauri/capabilities/main.json`, `windows:["*"]`). The "capability entry" obligation here is to confirm the governance commands resolve under `main` (no restrictive scope drops them; no remote-URL widening). Do **not** invent `allow-perm_grant.toml` files — that diverges from the repo convention. The real enforcement axis is server-side `but-authz`, not the transport capability.

> **Upstream advisory (do NOT edit the locked PRD).** `.spec/prds/governance/10-technical-requirements/04-api-design.md:80` instructs "`tauri-implementer` adds the matching `allow-perm_*` / `allow-group_*` / `allow-branch_gates_*` capability/permission entries". That `allow-*` language is **superseded by the live repo convention**: GitButler app commands ride `core:default` + `windows:["*"]` under `main.json` and emit **no** per-command `allow-*.toml` files. Implement against the live convention (no `allow-*` files); the api-design.md:80 wording is a documentation drift to be reconciled upstream, NOT a license to hand-author `allow-*` files in this task.

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: MGMT-IPC-003 — Register governance commands in generate_handler! + capability scope + v1 human-fleet-owner identity (T-MGMT-042)
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     M  (60 min)
AGENT:      implementer=tauri-implementer | reviewer=tauri-reviewer
PROPOSED-BY: tauri-planner
SPRINT:     ./SPRINT.md
PRD_REFS:   04-api-design.md (Tauri command surface + Identity & confinement), 08-uc-mgmt.md UC-MGMT-06 (T-MGMT-042), 10-ui-infrastructure.md (Specialist ownership)
CAPABILITIES: CAP-AUTHZ-01
PLATFORMS:  desktop

RUNTIME_COMMANDS:
  check: cargo check -p gitbutler-tauri
  test:  cargo test -p gitbutler-tauri mgmt_governance_commands_registered_and_invokable
  lint:  cargo clippy -p gitbutler-tauri --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
All 12 governance commands appear in crates/gitbutler-tauri/src/main.rs generate_handler!; cargo check
-p gitbutler-tauri compiles (necessary-not-sufficient capability admission: core:default admits all app
commands, so the check proves compilation + that no governance-specific scope was dropped/widened — NOT
that a per-command capability exists); the commands resolve under main (windows:["*"]); a config-management
command invoked through the real bus resolves the human fleet-owner via the named identity-substitution
shim (UNCONDITIONAL administration:write — the R12 superuser path, NOT a permissions.toml lookup), grants
EVEN on a bootstrap repo with no committed permissions.toml, NEVER falls through to resolve_principal_from_env,
and is NOT shadowed by a non-admin BUT_AGENT_HANDLE; an unauthorized AGENT actor is denied with
{code:"perm.denied"} carrying a non-empty remediation_hint (the MGMT-IPC-002 fix, observable here); an
unregistered governance command is NOT invokable.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] Register ALL 12 MGMT-IPC-001 governance commands in tauri::generate_handler! (main.rs) using the
  but_api::<module>::tauri_<name>::<name> form matching existing rows.
- [MUST] Establish commands<->capability-scope parity: every registered governance command resolves under
  the main capability (capabilities/main.json) — no command registered without a window/capability scope.
- [MUST] Resolve the v1 acting principal for MGMT config-management commands as the human fleet-owner from
  the signed-in desktop user (legacy::users::get_user) via a NAMED identity-substitution shim
  (fleet_owner_context / with_fleet_owner_identity) that grants the fleet-owner UNCONDITIONAL
  administration:write — the R12 accepted-risk superuser bypass, asserted at the command boundary, NOT a
  but-authz permissions.toml lookup.
- [MUST] The fleet-owner superuser path MUST grant EVEN WHEN no .gitbutler/permissions.toml is committed
  (bootstrap) — a permissions.toml lookup would deny here; the superuser bypass must not.
- [MUST] Keep the server-side but-authz administration:write gate (authored in MGMT-IPC-001) as the
  enforcement boundary FOR AGENTS (BUT_AGENT_HANDLE path); the fleet-owner is the single R12 exception.
- [NEVER] NEVER let the desktop governance command fall through to resolve_principal_from_env (it returns
  Denial::no_handle() when BUT_AGENT_HANDLE is unset, authorize.rs:71-72) — the desktop path is the
  fleet-owner path, injected before the env-handle resolution would run.
- [NEVER] NEVER let a non-admin BUT_AGENT_HANDLE in the env shadow the fleet-owner — the desktop command
  still acts as the fleet-owner from the session, regardless of any env handle.
- [NEVER] NEVER register a governance command without confirming it resolves under a capability scope.
- [NEVER] NEVER treat the Tauri capability as the authorization decision for an AGENT — the UI is never an
  agent bypass; server-side but-authz is the authority for agents. The fleet-owner superuser is the only
  sanctioned (R12) exception and binds the human at the boundary, not agents.
- [NEVER] NEVER author or fork the #[but_api] governance fns / but-authz gate here (MGMT-IPC-001).
- [NEVER] NEVER widen the capability to remote URLs or add a new permissive capability file — reuse main.
- [NEVER] NEVER hand-invent allow-perm_grant.toml-style files (GitButler app commands do not use them;
  api-design.md:80's allow-* wording is a superseded upstream advisory — see the task note).
- [STRICTLY] Pair every command addition with its capability-scope confirmation + (for config-mgmt
  commands) its identity-substitution wiring in THIS task — no "wire it later".
- [STRICTLY] Preserve existing but-api macro/transport/serialization patterns (crates/AGENTS.md); no parallel wrappers.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: all 12 governance commands registered + invokable through the real command bus
- [ ] AC-2: cargo check passes AND main.json retains identifier=main, windows=["*"], core:default with no
      governance-specific scope dropped/widened (necessary-not-sufficient capability admission)
- [ ] AC-3: config-management command resolves the human fleet-owner via the named shim, NOT resolve_principal_from_env
- [ ] AC-4: an unregistered/unscoped governance command is NOT invokable (negative parity proof)
- [ ] AC-5: an unauthorized AGENT config-management invoke returns {code:"perm.denied"} carrying a non-empty
      remediation_hint (UI is not an agent bypass; the MGMT-IPC-002 fix is observable here)
- [ ] AC-6: the fleet-owner acts EVEN WITH NO committed permissions.toml (bootstrap superuser path)
- [ ] AC-7: a non-admin BUT_AGENT_HANDLE does NOT shadow the fleet-owner (the command still acts as fleet-owner)
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: All 12 governance commands registered and invokable through the real Tauri command bus
  GIVEN: the MGMT-IPC-001 governance #[but_api] fns exist and gitbutler-tauri compiles
  WHEN:  perm_grant (and each of the 12 commands) is invoked through the real command bus (the real
         generate_handler! registration) as the resolved human-fleet-owner identity against governed_repo_admin
  THEN:  the invocation reaches the corresponding but-api governance fn and returns its Result over the
         real but_api::json::Error (no command-not-found rejection); all 12 command symbols present in generate_handler!
  TEST_TIER: integration   VERIFICATION_SERVICE: real gitbutler-tauri command bus + but-testsupport gix repo
  VERIFY: cargo test -p gitbutler-tauri mgmt_governance_commands_registered_and_invokable

AC-2: cargo check is necessary-not-sufficient — compiles AND main.json scope is preserved (capability admission)
  GIVEN: the 12 governance commands are registered in generate_handler!
  WHEN:  cargo check -p gitbutler-tauri runs (compiling the capability schema) AND main.json is inspected
  THEN:  cargo check exits 0 AND main.json retains identifier "main", windows ["*"], core:default — no
         governance-specific scope dropped and no remote-URL/per-command scope widened (grep-assert). core:default
         admits ALL app commands, so cargo check alone is necessary-not-sufficient for admission; the scope
         preservation is the real assertion.
  TEST_TIER: integration   VERIFICATION_SERVICE: real Tauri capability resolver (cargo check) + main.json grep-assert
  VERIFY: cargo check -p gitbutler-tauri && grep -q '"identifier": "main"' crates/gitbutler-tauri/capabilities/main.json && grep -q '"core:default"' crates/gitbutler-tauri/capabilities/main.json

AC-3: Config-management command resolves the human fleet-owner via the named shim, NOT resolve_principal_from_env (T-MGMT-042)
  GIVEN: a signed-in desktop user (fleet-owner) and a config-management command (e.g. perm_grant)
  WHEN:  perm_grant is invoked through the command bus with NO BUT_AGENT_HANDLE set in the env (the desktop case)
  THEN:  the named identity-substitution shim (fleet_owner_context / with_fleet_owner_identity) injects the
         fleet-owner identity with UNCONDITIONAL administration:write BEFORE any resolve_principal_from_env
         call — the command does NOT fall through to resolve_principal_from_env (no Denial::no_handle()) and the
         grant proceeds as the fleet-owner
  TEST_TIER: integration   VERIFICATION_SERVICE: real gitbutler-tauri command bus + real desktop session (get_user)
  VERIFY: cargo test -p gitbutler-tauri mgmt_config_command_resolves_fleet_owner_via_shim

AC-4: EDGE — unregistered/unscoped governance command is NOT invokable (negative parity proof)
  GIVEN: a governance command deliberately NOT added to generate_handler! (or removed)
  WHEN:  the renderer attempts to invoke that command through the real command bus
  THEN:  the invocation is rejected by Tauri as an unregistered command (command-not-found), proving
         registration — not mere existence of the but-api fn — is what makes a command invokable
  TEST_TIER: integration   VERIFICATION_SERVICE: real gitbutler-tauri command bus
  VERIFY: cargo test -p gitbutler-tauri mgmt_unregistered_governance_command_not_invokable

AC-5: ERROR — unauthorized AGENT config-management invoke returns {code:"perm.denied"} + a non-empty remediation_hint (UI is not an agent bypass)
  GIVEN: an AGENT acting principal (resolved via BUT_AGENT_HANDLE) that lacks administration:write (governed_repo_nonadmin)
  WHEN:  perm_grant is invoked through the real command bus as that agent (NOT the fleet-owner path)
  THEN:  the server-side but-authz gate denies it and the command returns the structured Denial over
         but_api::json::Error ({code:"perm.denied", message naming administration:write, remediation_hint non-empty});
         the working-tree permissions.toml is unchanged — the capability/transport did not authorize the action,
         and the MGMT-IPC-002 remediation_hint carrier survives transport (observable here)
  TEST_TIER: integration   VERIFICATION_SERVICE: real gitbutler-tauri command bus + real but-authz administration:write gate
  VERIFY: cargo test -p gitbutler-tauri mgmt_unauthorized_agent_config_command_denied

AC-6: BOOTSTRAP — the fleet-owner acts EVEN WITH NO committed permissions.toml (superuser path, not a permissions.toml lookup)
  GIVEN: bootstrap_repo (a but-testsupport gix repo with NO committed .gitbutler/permissions.toml at the target ref)
  WHEN:  perm_grant is invoked through the command bus as the resolved human fleet-owner (BUT_AGENT_HANDLE unset)
  THEN:  the grant SUCCEEDS — the fleet-owner's administration:write is the unconditional R12 superuser bypass,
         NOT a permissions.toml lookup (which would deny on a bootstrap repo with no committed grants); this
         falsifies any "look up the fleet-owner in permissions.toml" implementation
  TEST_TIER: integration   VERIFICATION_SERVICE: real gitbutler-tauri command bus + but-testsupport bootstrap gix repo
  VERIFY: cargo test -p gitbutler-tauri mgmt_fleet_owner_grants_on_bootstrap_no_committed_config

AC-7: NEGATIVE — a non-admin BUT_AGENT_HANDLE does NOT shadow the fleet-owner
  GIVEN: governed_repo_admin AND a non-admin agent handle set in the process env (BUT_AGENT_HANDLE = a principal lacking administration:write)
  WHEN:  perm_grant is invoked through the desktop command bus (the fleet-owner path)
  THEN:  the command still acts as the human fleet-owner (grant SUCCEEDS) — the non-admin BUT_AGENT_HANDLE in
         the env does NOT shadow the injected fleet-owner identity and the desktop path does NOT fall through to
         resolve_principal_from_env (no perm.denied / no Denial::no_handle())
  TEST_TIER: integration   VERIFICATION_SERVICE: real gitbutler-tauri command bus + non-admin env handle
  VERIFY: cargo test -p gitbutler-tauri mgmt_nonadmin_env_handle_does_not_shadow_fleet_owner

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): all 12 governance command symbols present in generate_handler! and each routes to its but-api fn through the real bus
    VERIFY: cargo test -p gitbutler-tauri mgmt_governance_commands_registered_and_invokable
- TC-2 (-> AC-2): cargo check -p gitbutler-tauri compiles AND main.json retains identifier main, windows ["*"], core:default (no governance scope dropped/widened)
    VERIFY: cargo check -p gitbutler-tauri && grep -q '"identifier": "main"' crates/gitbutler-tauri/capabilities/main.json && grep -q '"core:default"' crates/gitbutler-tauri/capabilities/main.json
- TC-3 (-> AC-3): with BUT_AGENT_HANDLE unset, a config-management command resolves the fleet-owner via the named shim, never falling through to resolve_principal_from_env
    VERIFY: cargo test -p gitbutler-tauri mgmt_config_command_resolves_fleet_owner_via_shim
- TC-4 (-> AC-4): an unregistered governance command is rejected by the real bus as command-not-found
    VERIFY: cargo test -p gitbutler-tauri mgmt_unregistered_governance_command_not_invokable
- TC-5 (-> AC-5): an unauthorized AGENT invoke returns {code:perm.denied} with a non-empty remediation_hint and leaves the working-tree config unchanged
    VERIFY: cargo test -p gitbutler-tauri mgmt_unauthorized_agent_config_command_denied
- TC-6 (-> AC-6): the fleet-owner grant succeeds on a bootstrap repo with no committed permissions.toml (superuser path, not a config lookup)
    VERIFY: cargo test -p gitbutler-tauri mgmt_fleet_owner_grants_on_bootstrap_no_committed_config
- TC-7 (-> AC-7): a non-admin BUT_AGENT_HANDLE in the env does not shadow the fleet-owner — the grant still succeeds as the fleet-owner
    VERIFY: cargo test -p gitbutler-tauri mgmt_nonadmin_env_handle_does_not_shadow_fleet_owner

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides: the 12 governance commands invokable through the real Tauri bus, capability-scope-bound under main, with the v1 human-fleet-owner superuser identity (R12) injected via a named shim
consumes: MGMT-IPC-001 (#[but_api(napi,...)] governance fns + their tauri_<name> command modules; the but-authz administration:write gate decision under BUT_AGENT_HANDLE); MGMT-IPC-002 (structured-error remediation_hint carrier surfaced through transport)
boundary_contracts:
  - Tauri command registration: tauri::generate_handler![..., but_api::governance::tauri_perm_grant::perm_grant, ...] at main.rs (naming mirrors but_api::branch::tauri_branch_diff::branch_diff)
  - Capability scope: capabilities/main.json identifier "main", windows ["*"], core:default — governance commands inherit the same scope as all 230+ existing app commands (no per-command allow-file; api-design.md:80's allow-* wording is a superseded upstream advisory)
  - Identity-substitution shim (T-MGMT-042, R12): the named gitbutler-tauri helper (fleet_owner_context / with_fleet_owner_identity) injects the human fleet-owner (legacy::users::get_user) with UNCONDITIONAL administration:write BEFORE resolve_principal_from_env would run — the desktop path NEVER falls through to the env-handle resolution; a non-admin BUT_AGENT_HANDLE does not shadow it; the bypass is the single sanctioned R12 exception (the agent BUT_AGENT_HANDLE path stays bound by but-authz)

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/gitbutler-tauri/src/main.rs (the generate_handler! block — add the 12 governance command rows)
  - crates/gitbutler-tauri/capabilities/main.json (only if a scope confirmation note requires; reuse the existing main capability)
  - crates/gitbutler-tauri/src/* (the named fleet-owner identity-substitution shim — fleet_owner_context / with_fleet_owner_identity — if MGMT-IPC-001 does not already accept an injected identity)
  - crates/gitbutler-tauri/tests/* (new integration tests for AC-1..AC-7)
writeProhibited:
  - crates/but-api/src/** (the #[but_api] governance fns + but-authz gate are MGMT-IPC-001 territory; the remediation_hint carrier is MGMT-IPC-002 territory)
  - crates/but-authz/src/** (authorization logic)
  - packages/but-sdk/src/generated/** (generated; regenerated by MGMT-IPC-004)
  - apps/desktop/src/** (SvelteKit UI — sveltekit-implementer); any file not in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. .spec/prds/governance/10-technical-requirements/04-api-design.md (Tauri command surface table + Identity & confinement; NOTE :80 allow-* wording is a superseded upstream advisory)
2. .spec/prds/governance/08-uc-mgmt.md (UC-MGMT-06 / T-MGMT-042 human-fleet-owner identity)
3. crates/gitbutler-tauri/src/main.rs:290-528 (the live generate_handler! block + existing but_api::<mod>::tauri_<name>::<name> rows)
4. crates/gitbutler-tauri/capabilities/main.json (the live main capability — plugin-permission convention; app commands ride core:default + windows:["*"])
5. crates/but-api/src/legacy/config_mutate.rs:18-28 (enforce_administration_write_gate + resolve_principal_from_env — the AGENT-path server-side gate the fleet-owner shim must bypass for the human)
6. crates/but-authz/src/authorize.rs:71-72 (resolve_principal returns Denial::no_handle() when BUT_AGENT_HANDLE unset — the fall-through the desktop path must NEVER hit)
7. crates/but-api/tests/admin_write_guard.rs + confinement.rs (existing governance test scaffolding to mirror)
8. crates/but-api/src/legacy/users.rs:77 (get_user — desktop session resolution for the fleet-owner)
9. crates/AGENTS.md (preserve but-api macro/transport patterns; lock discipline)

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- cargo check -p gitbutler-tauri                                                          -> Exit 0 (compiles; necessary-not-sufficient for capability admission)
- grep main.json identifier "main" + core:default                                        -> present (scope preserved)
- cargo test -p gitbutler-tauri mgmt_governance_commands_registered_and_invokable         -> Exit 0
- cargo test -p gitbutler-tauri mgmt_config_command_resolves_fleet_owner_via_shim         -> Exit 0
- cargo test -p gitbutler-tauri mgmt_unregistered_governance_command_not_invokable        -> Exit 0
- cargo test -p gitbutler-tauri mgmt_unauthorized_agent_config_command_denied             -> Exit 0
- cargo test -p gitbutler-tauri mgmt_fleet_owner_grants_on_bootstrap_no_committed_config   -> Exit 0
- cargo test -p gitbutler-tauri mgmt_nonadmin_env_handle_does_not_shadow_fleet_owner       -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: Register-and-capability-bind + named identity-substitution shim — add the #[but_api]-generated
  command modules to generate_handler!, confirm capability-scope admission, and inject the v1 desktop
  fleet-owner identity (UNCONDITIONAL administration:write, R12 superuser bypass) at the Tauri boundary BEFORE
  resolve_principal_from_env — atomic command + permission(scope) + identity in one task.
pattern_source: crates/gitbutler-tauri/src/main.rs (existing but_api::* command rows, e.g. branch::tauri_branch_diff::branch_diff at :307) + 04-api-design.md (Tauri command surface) + 08-uc-mgmt.md T-MGMT-042 + crates/but-api/src/legacy/config_mutate.rs:18-28 (the AGENT gate to bypass for the human) + authorize.rs:71-72 (the no_handle fall-through to avoid)
anti_pattern: inventing per-command allow-perm_grant.toml files (GitButler app commands do not use them;
  api-design.md:80's allow-* wording is superseded); looking up the fleet-owner in permissions.toml (would deny
  on a bootstrap repo — falsified by AC-6); letting the desktop path fall through to resolve_principal_from_env
  (Denial::no_handle, authorize.rs:71-72); letting a non-admin BUT_AGENT_HANDLE shadow the fleet-owner (AC-7);
  treating the capability/transport as the authorization decision FOR AN AGENT (the agent path stays bound by
  but-authz — only the human fleet-owner is the R12 sanctioned superuser); registering a command without
  confirming capability-scope admission.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: tauri-implementer — gitbutler-tauri command-surface work (generate_handler!, capabilities/IPC, boundary identity)
reviewer: tauri-reviewer
coding_standards: crates/AGENTS.md (preserve but-api macro/transport patterns; lock discipline), CLAUDE.md/RULES.md, brain/docs/REQUIREMENT-TRACKING.md

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: MGMT-IPC-001 (the #[but_api] governance fns to register + the agent-path admin gate decision), MGMT-IPC-002 (the structured-error remediation_hint carrier surfaced through transport — observable in AC-5)
Blocks:     MGMT-IPC-004 (SDK regen needs the registered surface)
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "MGMT-IPC-003",
  "proposed_by": "tauri-planner",
  "verification_policy": { "requires_tests": true, "requires_red_evidence": true, "requires_seeded_evidence": true },
  "fixtures": {
    "governed_repo_admin": { "description": "but-testsupport gix repo with a committed .gitbutler/permissions.toml at refs/heads/main granting the dev principal administration:write (mirrors crates/but-api/tests/admin_write_guard.rs admin_write_repo). Seeded via a real entrypoint.", "seed_method": "cli", "records": ["dev principal = administration:write", "committed at refs/heads/main"] },
    "governed_repo_nonadmin": { "description": "Same but-testsupport scenario where the acting AGENT principal lacks administration:write (the dev principal in admin_write_guard.rs).", "seed_method": "cli", "records": ["dev principal = no administration:write", "committed at refs/heads/main"] },
    "bootstrap_repo": { "description": "but-testsupport gix repo with NO committed .gitbutler/permissions.toml at the target ref (bootstrap state). Proves the fleet-owner superuser path grants without a committed config — a permissions.toml lookup would deny here.", "seed_method": "cli", "records": ["no committed .gitbutler/permissions.toml at target ref", "clean repo, no governance grants"] }
  },
  "requirements": [
    { "id": "AC-1", "type": "acceptance_criterion", "primary": true, "description": "GIVEN the MGMT-IPC-001 governance #[but_api] fns exist and gitbutler-tauri compiles WHEN perm_grant (and each of the 12) is invoked through the real command bus as the human-fleet-owner identity against governed_repo_admin THEN the invocation reaches its but-api governance fn and returns its Result over real json::Error (no command-not-found); all 12 command symbols present in generate_handler!", "verify": "cargo test -p gitbutler-tauri mgmt_governance_commands_registered_and_invokable", "scenario": { "id": "AC-1", "primary": true, "tier": "visible", "test_tier": "integration", "verification_service": "gitbutler-tauri command bus", "negative_control": { "would_fail_if": ["the command is absent from generate_handler! (stub registration)", "the handler list is empty/mocked", "invocation is faked instead of routed through the real command bus"] }, "evidence": { "artifact_type": "stdout", "required_capture": true }, "cases": [ { "start_ref": "governed_repo_admin", "action": { "actor": "human fleet-owner", "steps": ["build the real app handler via the gitbutler-tauri generate_handler! surface", "invoke perm_grant through the command bus with {principal:'dev', authority:'reviews:write'}", "assert each of the 12 command names present in the registered command list"] }, "end_state": { "must_observe": ["command name `\"perm_grant\"` present in the registered generate_handler! command list", "all 12 governance command names present (perm_list/perm_grant/perm_revoke/group_create/group_grant/group_add_member/group_remove_member/group_delete/group_list/branch_gates_read/branch_gates_update/governance_status_read)", "the `perm_grant` invocation reaches the but-api governance fn (returns `Ok` or a structured `Denial`)"], "must_not_observe": ["`\"command perm_grant not found\"` / unregistered-command rejection", "an empty registered-command list", "fewer than 12 governance command names registered"] } } ] } },
    { "id": "AC-2", "type": "acceptance_criterion", "primary": false, "description": "GIVEN the 12 governance commands registered WHEN cargo check -p gitbutler-tauri runs AND main.json is inspected THEN cargo check exits 0 (necessary-not-sufficient: core:default admits all app commands) AND main.json retains identifier main, windows [\"*\"], core:default with no governance-specific scope dropped/widened", "verify": "cargo check -p gitbutler-tauri && grep -q '\"identifier\": \"main\"' crates/gitbutler-tauri/capabilities/main.json && grep -q '\"core:default\"' crates/gitbutler-tauri/capabilities/main.json", "scenario": { "id": "AC-2", "primary": false, "tier": "visible", "test_tier": "integration", "verification_service": "gitbutler-tauri build capability resolver + main.json grep-assert", "negative_control": { "would_fail_if": ["main.json identifier `main` is dropped/renamed", "core:default is removed so app windows lose the shared scope", "a new remote-URL or per-command restrictive capability is added that widens/narrows the governance scope", "the capability check is skipped/mocked"] }, "evidence": { "artifact_type": "stdout", "required_capture": true }, "cases": [ { "start_ref": "governed_repo_admin", "action": { "actor": "gitbutler-tauri build", "steps": ["run cargo check -p gitbutler-tauri (compiles the capability schema against the registered surface)", "grep main.json for identifier `\"main\"`, windows `[\"*\"]`, and `\"core:default\"`"] }, "end_state": { "must_observe": ["`cargo check -p gitbutler-tauri` exits 0", "main.json retains identifier `\"main\"`", "main.json retains windows `[\"*\"]` and `\"core:default\"`", "no governance-specific scope dropped and none widened to a remote URL (`core:default` + `windows:[\"*\"]` retained verbatim)"], "must_not_observe": ["identifier `\"main\"` dropped/renamed", "`core:default` removed (app windows lose the shared scope)", "a new remote-URL / per-command restrictive capability added for governance", "main.json reduced/empty"] } } ] } },
    { "id": "AC-3", "type": "acceptance_criterion", "primary": false, "description": "GIVEN a signed-in desktop user (fleet-owner) and a config-management command WHEN perm_grant is invoked with NO BUT_AGENT_HANDLE set THEN the named identity-substitution shim injects the fleet-owner with UNCONDITIONAL administration:write BEFORE resolve_principal_from_env — the command does NOT fall through to resolve_principal_from_env (no Denial::no_handle) and the grant proceeds as the fleet-owner", "verify": "cargo test -p gitbutler-tauri mgmt_config_command_resolves_fleet_owner_via_shim", "scenario": { "id": "AC-3", "primary": false, "tier": "visible", "test_tier": "integration", "verification_service": "gitbutler-tauri command bus + desktop session (get_user)", "negative_control": { "would_fail_if": ["the identity is faked/hardcoded", "resolution falls through to resolve_principal_from_env (Denial::no_handle with BUT_AGENT_HANDLE unset)", "the shim is bypassed so the desktop path uses the env handle"] }, "evidence": { "artifact_type": "api_response", "required_capture": true }, "cases": [ { "start_ref": "governed_repo_admin", "action": { "actor": "human fleet-owner (desktop session, BUT_AGENT_HANDLE unset)", "steps": ["ensure BUT_AGENT_HANDLE is unset in the process env", "invoke perm_grant {principal:'dev', authority:'reviews:write'} through the command bus via the fleet-owner shim"] }, "end_state": { "must_observe": ["the named identity-substitution shim injects the human fleet-owner identity (administration:write granted unconditionally)", "the command does NOT call/fall through to `resolve_principal_from_env`", "the grant proceeds to the working-tree `.gitbutler/permissions.toml`"], "must_not_observe": ["a `\"BUT_AGENT_HANDLE not set\"` / Denial::no_handle resolution error in the desktop path", "the principal resolved from BUT_AGENT_HANDLE / resolve_principal_from_env instead of the shim", "the grant applied with no identity injection"] } } ] } },
    { "id": "AC-4", "type": "acceptance_criterion", "primary": false, "description": "GIVEN a governance command deliberately NOT in generate_handler! WHEN the renderer invokes it through the real bus THEN it is rejected as command-not-found — proving registration (not mere existence of the but-api fn) makes a command invokable", "verify": "cargo test -p gitbutler-tauri mgmt_unregistered_governance_command_not_invokable", "scenario": { "id": "AC-4", "primary": false, "tier": "visible", "test_tier": "integration", "verification_service": "gitbutler-tauri command bus", "negative_control": { "would_fail_if": ["the test asserts success for an unregistered command (the bus is mocked)", "every command silently succeeds regardless of registration (stub)"] }, "evidence": { "artifact_type": "stdout", "required_capture": true }, "cases": [ { "start_ref": "governed_repo_admin", "action": { "actor": "renderer (simulated invoke)", "steps": ["invoke a deliberately-unregistered governance command name through the real command bus", "capture the bus rejection"] }, "end_state": { "must_observe": ["the bus rejects the unregistered command (`command-not-found` / not in the handler list)"], "must_not_observe": ["the unregistered command invocation succeeds", "a stubbed Ok response for an unregistered command", "no rejection (empty error)"] } } ] } },
    { "id": "AC-5", "type": "acceptance_criterion", "primary": false, "description": "GIVEN an AGENT acting principal lacking administration:write (governed_repo_nonadmin, via BUT_AGENT_HANDLE) WHEN perm_grant is invoked through the real bus as that agent THEN the server-side but-authz gate denies it, returns {code:perm.denied} naming administration:write and carrying a non-empty remediation_hint (the MGMT-IPC-002 fix), and leaves the working-tree config unchanged (UI is not an agent bypass)", "verify": "cargo test -p gitbutler-tauri mgmt_unauthorized_agent_config_command_denied", "scenario": { "id": "AC-5", "primary": false, "tier": "visible", "test_tier": "integration", "verification_service": "gitbutler-tauri command bus + but-authz administration:write gate", "negative_control": { "would_fail_if": ["denial is faked", "the gate is bypassed because the command reached the UI", "the working-tree config is mutated despite the denial (unchanged assertion fails)", "the remediation_hint is empty/absent (the MGMT-IPC-002 carrier did not survive transport)"] }, "evidence": { "artifact_type": "api_response", "required_capture": true }, "cases": [ { "start_ref": "governed_repo_nonadmin", "action": { "actor": "AGENT principal lacking administration:write (BUT_AGENT_HANDLE = non-admin)", "steps": ["set BUT_AGENT_HANDLE to the non-admin agent (the agent path, not the fleet-owner path)", "invoke perm_grant {principal:'dev', authority:'merge'} through the command bus", "read the working-tree .gitbutler/permissions.toml after"] }, "end_state": { "must_observe": ["structured Denial returned: `code == \"perm.denied\"`", "the denial message names `\"administration:write\"`", "the serialized error carries a non-empty `remediation_hint`", "the working-tree `.gitbutler/permissions.toml` is byte-unchanged from before the invoke"], "must_not_observe": ["the grant applied (UI used as an agent bypass)", "an unstructured/empty error", "an empty or absent `remediation_hint`", "a code other than `perm.denied` for a missing-authority denial"] } } ] } },
    { "id": "AC-6", "type": "acceptance_criterion", "primary": false, "description": "BOOTSTRAP — GIVEN bootstrap_repo (NO committed .gitbutler/permissions.toml) WHEN perm_grant is invoked as the human fleet-owner (BUT_AGENT_HANDLE unset) THEN the grant SUCCEEDS — the fleet-owner administration:write is the unconditional R12 superuser bypass, NOT a permissions.toml lookup (which would deny on a bootstrap repo); falsifies a look-up-fleet-owner-in-permissions.toml implementation", "verify": "cargo test -p gitbutler-tauri mgmt_fleet_owner_grants_on_bootstrap_no_committed_config", "scenario": { "id": "AC-6", "primary": false, "tier": "visible", "test_tier": "integration", "verification_service": "gitbutler-tauri command bus + but-testsupport bootstrap gix repo", "negative_control": { "would_fail_if": ["the fleet-owner is looked up in permissions.toml (denied on a bootstrap repo with no committed grants)", "the desktop path falls through to resolve_principal_from_env (Denial::no_handle)", "the grant is denied because no committed config grants the fleet-owner administration:write", "the superuser bypass is hardcoded/stubbed to always-Ok instead of really resolving the fleet-owner identity and really writing the grant", "the grant is a no-op that does not actually persist to the working-tree permissions.toml"] }, "evidence": { "artifact_type": "api_response", "required_capture": true }, "cases": [ { "start_ref": "bootstrap_repo", "action": { "actor": "human fleet-owner (BUT_AGENT_HANDLE unset)", "steps": ["confirm no committed .gitbutler/permissions.toml at the target ref", "invoke perm_grant {principal:'dev', authority:'contents:write'} through the command bus via the fleet-owner shim"] }, "end_state": { "must_observe": ["the perm_grant invocation SUCCEEDS (returns `Ok`) on a repo with no committed permissions.toml", "the fleet-owner is granted administration:write unconditionally (not via a permissions.toml lookup)", "the new grant is written to the working-tree `.gitbutler/permissions.toml`"], "must_not_observe": ["a `perm.denied` for the fleet-owner on a bootstrap repo", "a `Denial::no_handle` / `\"BUT_AGENT_HANDLE not set\"` error", "the grant blocked because no committed config grants the fleet-owner administration:write"] } } ] } },
    { "id": "AC-7", "type": "acceptance_criterion", "primary": false, "description": "NEGATIVE — GIVEN governed_repo_admin AND a non-admin agent handle set in BUT_AGENT_HANDLE WHEN perm_grant is invoked through the desktop command bus (the fleet-owner path) THEN the command still acts as the human fleet-owner (grant SUCCEEDS) — the non-admin BUT_AGENT_HANDLE does NOT shadow the injected fleet-owner identity and the path does NOT fall through to resolve_principal_from_env", "verify": "cargo test -p gitbutler-tauri mgmt_nonadmin_env_handle_does_not_shadow_fleet_owner", "scenario": { "id": "AC-7", "primary": false, "tier": "visible", "test_tier": "integration", "verification_service": "gitbutler-tauri command bus + non-admin env handle", "negative_control": { "would_fail_if": ["the non-admin BUT_AGENT_HANDLE shadows the fleet-owner so the command is denied", "the desktop path falls through to resolve_principal_from_env and uses the env handle", "the grant is denied because the env handle lacks administration:write", "the superuser path is hardcoded/stubbed to always-Ok instead of really injecting the fleet-owner over the env handle", "the grant is a no-op that does not actually persist to the working-tree permissions.toml"] }, "evidence": { "artifact_type": "api_response", "required_capture": true }, "cases": [ { "start_ref": "governed_repo_admin", "action": { "actor": "human fleet-owner (desktop path) with a non-admin BUT_AGENT_HANDLE present in the env", "steps": ["set BUT_AGENT_HANDLE to a principal that lacks administration:write", "invoke perm_grant {principal:'dev', authority:'reviews:write'} through the desktop command bus (the fleet-owner path)"] }, "end_state": { "must_observe": ["the perm_grant invocation returns `Ok` as the human fleet-owner despite the non-admin `BUT_AGENT_HANDLE`", "the injected fleet-owner identity is used (the env handle does NOT shadow it): the acting principal `== fleet-owner`, not the env handle", "the desktop path does NOT fall through to `resolve_principal_from_env`"], "must_not_observe": ["a `perm.denied` caused by the non-admin BUT_AGENT_HANDLE shadowing the fleet-owner", "a `Denial::no_handle` or env-handle-derived denial in the desktop path", "the principal resolved from BUT_AGENT_HANDLE instead of the fleet-owner shim", "no grant written / an empty (0-change) permissions.toml after a successful fleet-owner invoke"] } } ] } },
    { "id": "TC-1", "type": "test_criterion", "description": "all 12 governance command symbols present in generate_handler! and each routes to its but-api fn through the real bus", "verify": "cargo test -p gitbutler-tauri mgmt_governance_commands_registered_and_invokable", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "cargo check -p gitbutler-tauri compiles AND main.json retains identifier main, windows [\"*\"], core:default (no governance scope dropped/widened)", "verify": "cargo check -p gitbutler-tauri && grep -q '\"identifier\": \"main\"' crates/gitbutler-tauri/capabilities/main.json && grep -q '\"core:default\"' crates/gitbutler-tauri/capabilities/main.json", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "with BUT_AGENT_HANDLE unset, a config-management command resolves the fleet-owner via the named shim, never falling through to resolve_principal_from_env", "verify": "cargo test -p gitbutler-tauri mgmt_config_command_resolves_fleet_owner_via_shim", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "an unregistered governance command is rejected by the real bus as command-not-found", "verify": "cargo test -p gitbutler-tauri mgmt_unregistered_governance_command_not_invokable", "maps_to_ac": "AC-4" },
    { "id": "TC-5", "type": "test_criterion", "description": "an unauthorized AGENT invoke returns {code:perm.denied} with a non-empty remediation_hint and leaves the working-tree config unchanged", "verify": "cargo test -p gitbutler-tauri mgmt_unauthorized_agent_config_command_denied", "maps_to_ac": "AC-5" },
    { "id": "TC-6", "type": "test_criterion", "description": "the fleet-owner grant succeeds on a bootstrap repo with no committed permissions.toml (superuser path, not a config lookup)", "verify": "cargo test -p gitbutler-tauri mgmt_fleet_owner_grants_on_bootstrap_no_committed_config", "maps_to_ac": "AC-6" },
    { "id": "TC-7", "type": "test_criterion", "description": "a non-admin BUT_AGENT_HANDLE in the env does not shadow the fleet-owner — the grant still succeeds as the fleet-owner", "verify": "cargo test -p gitbutler-tauri mgmt_nonadmin_env_handle_does_not_shadow_fleet_owner", "maps_to_ac": "AC-7" }
  ]
}
-->
</details>
