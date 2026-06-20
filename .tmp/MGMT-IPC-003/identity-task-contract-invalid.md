# MGMT-IPC-003 identity/denial RED blocker

Classification: `TASK_CONTRACT_INVALID`

The remaining AC-3/5/6/7 requirements cannot be tested honestly from the allowed RED scope without a production test seam or implementation repair.

## Concrete blocker

`crates/gitbutler-tauri` exposes `gitbutler_tauri::invoke_handler()`, but that only constructs the real `tauri::generate_handler!` closure. The existing IPC-003 test already documents the limitation: calling the handler requires a full `tauri::ipc::Invoke<tauri::Wry>` and runtime state. This crate has no test helper that can:

- build an invokable Tauri app/runtime with `gitbutler_tauri::invoke_handler()`,
- register a real GitButler test repository as the `project_id`/`Context` source,
- install a signed-in desktop `legacy::users::get_user` fleet-owner session,
- choose between desktop fleet-owner invocation and agent/env invocation,
- capture the transported `but_api::json::Error` payload from the real command boundary.

Separately, the current governance config-management path still calls `enforce_administration_write_gate()`, which calls `but_authz::resolve_principal_from_env()` directly. There is no named `fleet_owner_context` / `with_fleet_owner_identity` helper and no injection point that can be observed by an integration test.

## Why source grep or direct helper tests would be fake

The rejected prior approach can only prove strings in files, not behavior. A grep for a shim name would not show that `perm_grant` was invoked through Tauri, that the fleet-owner identity was injected before authz, or that `BUT_AGENT_HANDLE` failed to shadow the desktop user.

Calling `but_api::legacy::governance::perm_grant_with_repo()` directly is also not sufficient. That bypasses the command bus and the desktop session layer entirely; today it always routes through `resolve_principal_from_env()`, so it can only prove the agent path that MGMT-IPC-001 already covered.

Calling the generated Tauri wrapper directly would still be insufficient unless the test can supply the same state extraction and serialization path that the real command bus uses. Otherwise AC-5's transported `{code:"perm.denied", remediation_hint: ...}` and AC-3/6/7's desktop identity substitution remain unproven.

## Minimal SPEC-REPAIR required

Add one narrow production test seam in `crates/gitbutler-tauri`:

1. A command-boundary helper with the spec's named identity-substitution behavior, for example `with_fleet_owner_identity` or `fleet_owner_context`, that resolves the signed-in desktop user and injects unconditional `administration:write` for config-management commands before the but-api governance function can reach env-handle resolution.
2. An integration-test harness/factory that invokes the real `gitbutler_tauri::invoke_handler()` with controlled state, a but-testsupport/gix repository, and controlled desktop user/session identity.
3. A way for the harness to exercise both paths:
   - desktop fleet-owner path, where `BUT_AGENT_HANDLE` is unset or set to a non-admin and must not shadow the fleet-owner;
   - agent/env path, where a non-admin `BUT_AGENT_HANDLE` reaches the real but-authz denial and returns the real serialized `but_api::json::Error`.

With that seam, the RED tests should assert:

- AC-3: `BUT_AGENT_HANDLE` unset succeeds as fleet-owner and never returns `Denial::no_handle()`.
- AC-5: unauthorized agent invoke returns `perm.denied`, includes non-empty `remediation_hint`, names `administration:write`, and leaves working-tree `.gitbutler/permissions.toml` byte-identical.
- AC-6: fleet-owner `perm_grant` succeeds on a bootstrap repository with no committed `.gitbutler/permissions.toml`.
- AC-7: non-admin `BUT_AGENT_HANDLE` does not shadow desktop fleet-owner identity; the desktop command still succeeds.
