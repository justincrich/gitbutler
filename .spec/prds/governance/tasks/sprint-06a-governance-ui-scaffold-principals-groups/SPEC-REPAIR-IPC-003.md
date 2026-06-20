# SPEC-REPAIR-IPC-003: Resolve TASK_CONTRACT_INVALID on MGMT-IPC-003

> **Origin:** `TASK_CONTRACT_INVALID` raised by the RED-FIRST phase of MGMT-IPC-003 (see
> `.tmp/MGMT-IPC-003/task-contract-invalid.md` on branch `kb-run-sprint/sprint-06a-…/RED-FIRST-MGMT-IPC-003`).
> The honest RED agent refused to fake source-grep tests against a private `generate_handler!` and recorded a
> real spec/code mismatch. This SPEC-REPAIR resolves all facets and lands the full IPC-003 GREEN so the sprint
> can resume (IPC-004 SDK regen → UI wave).

## What this does

Resolves the 5-facet contract gap blocking MGMT-IPC-003, then completes the IPC-003 implementation (RED→GREEN)
so the governance commands are registered, correctly-named, testable, capability-bound, and identity-wired per
the PRD command contract (`04-api-design.md:84-95`).

## Why

IPC-003 is the gatekeeper of the entire MGMT-UI surface — IPC-004 (SDK regen) depends on it, and every
`but-sdk`-importing UI task depends on IPC-004. The RED phase correctly blocked on a real mismatch; this repair
unblocks the sprint without weakening any contract.

## The 5 facets (root-caused)

### Facet 1 — Command-name mismatch (`*_cmd` vs PRD-mandated bare names)

`#[but_api]` (`crates/but-api-macros/src/lib.rs:165-178`) **generates** `<fn_name>_cmd` (server wrapper),
`<fn_name>_json`, `<fn_name>_napi`, and modules `tauri_<fn_name>` / `napi_<fn_name>`. The **source fn name IS
the Tauri command name**. IPC-001 named the source fns `perm_grant_cmd`, `group_list_cmd`, … which produces:

- Tauri command name `perm_grant_cmd` (WRONG — PRD `04-api-design.md:85` mandates `perm_grant`)
- Server wrapper `perm_grant_cmd_cmd` (doubly-suffixed)

The `_cmd` suffix was added to avoid colliding with the Sprint-05 underlying `&repo` fns (`perm_grant(repo,…)`)
in the SAME module. The macro design expects the source fn to be bare; the collision must be resolved differently.

**Resolution:**

- Rename all 8 `#[but_api]` wrappers in `crates/but-api/src/legacy/governance.rs` to **bare names** matching the
  PRD: `perm_list`, `perm_grant`, `perm_revoke`, `group_create`, `group_grant`, `group_add_member`,
  `group_remove_member`, `group_list`. (`governance_status_read` at line 265 is already bare — leave it.)
- Relocate the 8 Sprint-05 underlying `&repo` fns (lines ~274-499) by renaming them to a **`_with_repo`
  suffix** (`group_list_with_repo`, `group_create_with_repo`, …). This mirrors the repo's existing `_with_perm`
  convention documented in `crates/AGENTS.md` ("API consumers that already hold permission use `_with_perm`");
  `_with_repo` is the analog for "callers that already hold a resolved `&gix::Repository`." Update all internal
  callers + tests.
- The wrappers resolve repo + `target_ref` from `ctx` and delegate to `*_with_repo` — bodies otherwise unchanged.

### Facet 2 — Re-add `group_delete` (removed by `a78206fe2a`, required by UI-008 AC-4)

Commit `a78206fe2a` removed both the `#[but_api] group_delete_cmd` wrapper AND the underlying `group_delete(&repo,…)`
impl as "unsupported." UI-008 AC-4 requires the groups tab to delete a group (`group_delete` SDK call with a
confirmation modal). **User decision: re-add.**

**Resolution:** Author a real `group_delete` wrapper + `group_delete_with_repo` impl in `governance.rs`:

- `#[but_api] pub fn group_delete(ctx: &Context, target_ref: String, group: String) -> anyhow::Result<GroupWriteOutcome>`
  → resolves repo + target_ref from ctx → calls `group_delete_with_repo(repo, target_ref, group)`
- `pub fn group_delete_with_repo(repo: &gix::Repository, target_ref: &str, group: &str) -> anyhow::Result<GroupWriteOutcome>`
  → `enforce_administration_write_gate(repo, target_ref)?` → removes the group from `.gitbutler/groups.toml`
  (or wherever `group_create_with_repo` writes groups — match that writer's location/format) → returns
  `GroupWriteOutcome` consistent with the other group writes
- Pattern-match the existing `group_create_with_repo` / `group_remove_member_with_repo` shape exactly (same
  gate call, same config-load + mutate + write cycle, same outcome type).

### Facet 3 — Author `branch_gates_read` + `branch_gates_update` (06a backend seam for 06b UI tab)

IPC-003's 12-command contract includes `branch_gates_read` / `branch_gates_update` (`04-api-design.md:93-94`).
These but-api fns don't exist yet. **User decision: register them in 06a** (06a ships the backend seam; 06b
consumes it for the Branch Gates UI tab).

**Resolution:** Author minimal but-REAL wrappers + impls in `governance.rs`:

- `#[but_api] pub fn branch_gates_read(ctx: &Context, target_ref: String) -> anyhow::Result<BranchGatesOutcome>`
  → delegates to `branch_gates_read_with_repo(repo, target_ref)` → reads `.gitbutler/gates.toml` via
  `but_authz::load_governance_config` (already loads gates — `config.rs:9,23,43`) → returns the gates map
  (use the existing `BranchProtection` / normalized `BTreeMap<BranchName, BranchProtection>` from
  `but-authz/src/config.rs`; define a `#[derive(Serialize)] BranchGatesOutcome` carrier consistent with
  `GroupListOutcome`/`PermListOutcome`).
- `#[but_api] pub fn branch_gates_update(ctx: &Context, target_ref: String, branch: String, protection: BranchProtectionInput) -> anyhow::Result<BranchGatesOutcome>`
  → delegates to `branch_gates_update_with_repo(repo, target_ref, branch, protection)` →
  `enforce_administration_write_gate(repo, target_ref)?` → load current gates → upsert the branch entry →
  write `.gitbutler/gates.toml` → return updated gates.
- Define the input/return types in `governance.rs`, `#[derive(Serialize)]` for the outcome; `#[derive(Deserialize)]`
  for the input (the macro's transport needs both). Mirror the existing outcome-type pattern.
- Keep these REAL — load the actual gates file, write the actual gates file. No stubs. The 06b UI tab will call
  these through the SDK; they must work against a real repo now.

### Facet 4 — Extract `generate_handler!` into a testable factory

`crates/gitbutler-tauri/src/lib.rs` currently exposes nothing — the `generate_handler!` list is private to
`main.rs:290`. AC-1 of IPC-003 expects to "build the real app handler via the gitbutler-tauri generate_handler!
surface." A `tauri::test::mock_builder()` test that duplicates the command list is test theatre (the RED agent
correctly refused it).

**Resolution:**

- In `crates/gitbutler-tauri/src/lib.rs`, expose a function that returns the FULL real handler list in a form
  both `main.rs` and tests can consume. The clean shape, consistent with Tauri: a public function
  `pub fn app_handler_builder() -> tauri::Builder<tauri::Wry>` OR a macro/helper that emits the
  `generate_handler!` payload. Prefer whichever shape lets `main.rs::run()` stay the single binary entrypoint
  while tests can invoke the SAME registered command list without duplicating it.
  - If a Builder-returning factory is awkward (plugins/state differ between prod and test), the minimal shape
    is: a `pub fn invoke_handler() -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static`
    built from the real `generate_handler!`, which tests call via `tauri::test::mock_builder()` to assert
    command presence + dispatch. Match the approach the Tauri version in this repo supports; check
    `tauri::test` availability in `Cargo.toml` `[dev-dependencies]`.
- `main.rs:290` MUST consume the same factory (no duplicated list). Add the 12 governance commands to the
  factory's list as: `legacy::governance::tauri_perm_grant::perm_grant`, … (bare names, post-Facet-1 rename),
  matching the existing registration path shape (`legacy::users::tauri_set_user::set_user`, etc.).

### Facet 5 — Capability binding (live convention, not the PRD's `allow-*` drift)

`04-api-design.md:80` says "adds the matching `allow-perm_*` … capability/permission entries" but the live repo
convention is `core:default` + `windows:["*"]` under `src-tauri/capabilities/main.json` with NO per-command
`allow-*.toml` files. This is documented drift; the IPC-003 task already flags it.

**Resolution:** Implement against the LIVE convention. Confirm the existing `core:default` capability covers the
new governance commands (it should — they ride the same scope as the other `legacy::*` commands). If a
capability-scope grep is the IPC-003 AC, scope it to asserting `core:default` is present + the commands compile
under it, NOT to hand-authoring `allow-*.toml` files. Do NOT create per-command allow files.

## Scope (files)

- `crates/but-api/src/legacy/governance.rs` (MODIFY heavily) — rename 8 wrappers to bare; rename 8 underlying
  fns to `_with_repo`; add `group_delete` + `group_delete_with_repo`; add `branch_gates_read`/`branch_gates_update`
  + their `_with_repo` impls + their Serialize/Deserialize outcome/input types.
- `crates/but-api/tests/*.rs` (MODIFY) — update all call sites from `*_cmd`/bare-old to the new bare wrapper
  names + `_with_repo` underlying names. Tests that called `perm_grant_cmd(ctx,…)` now call `perm_grant(ctx,…)`;
  tests that called the underlying `perm_grant(repo,…)` now call `perm_grant_with_repo(repo,…)`.
- `crates/gitbutler-tauri/src/lib.rs` (MODIFY) — expose the handler factory (Facet 4).
- `crates/gitbutler-tauri/src/main.rs` (MODIFY) — consume the factory; register the 12 governance commands.
- `crates/gitbutler-tauri/tests/mgmt_ipc_003_governance_commands.rs` (PORT + GREEN) — port the honest RED test
  from commit `4b75045249` (branch `RED-FIRST-MGMT-IPC-003`), then make it GREEN via the real factory. The test
  asserts all 12 command names present in the registered list AND that an invocation routes to the but-api fn.

## Identity wiring (T-MGMT-042) — the IPC-003 AC beyond the blocker

IPC-003 also requires wiring the v1 acting principal for MGMT config-management commands to the **human
fleet-owner** resolved from the signed-in desktop session (`legacy::users::get_user`, per `04-api-design.md:144`
— the "human fleet-owner, trusted superuser" trust root). This was NOT flagged by the RED blocker but IS an
IPC-003 AC.

**In this SPEC-REPAIR:** wire the identity resolution IF the existing `#[but_api]` macro / `Context` resolution
path already supports injecting the desktop-user identity at the command boundary. If identity wiring requires
deeper changes (a new `Context` field, a `UserService` shim), land the registration + naming + test-harness work
FIRST (the blocker facets 1-5), mark identity-wiring progress in `.tmp/MGMT-IPC-003/`, and flag it as a
follow-up AC rather than blocking the whole repair. The load-bearing invariant (commands are registered +
governed by `but-authz` at the but-api boundary) holds either way; identity resolution selects WHICH principal
the gate evaluates, and the current `BUT_AGENT_HANDLE` fallback is a valid (if v1-incomplete) identity source.

## How to verify

PRIMARY **AC-1** — `cargo test -p gitbutler-tauri mgmt_governance_commands_registered_and_invokable` PASSES:
all 12 governance command names (`perm_list`/`perm_grant`/`perm_revoke`/`group_create`/`group_grant`/
`group_add_member`/`group_remove_member`/`group_delete`/`group_list`/`branch_gates_read`/`branch_gates_update`/
`governance_status_read`) are present in the REAL registered command list (via the extracted factory, not a
duplicated test list), and an invocation routes through the real command bus to the but-api fn.

- **AC-2** — `cargo test -p but-api governance` PASSES (existing IPC-001/002 proofs stay GREEN after the rename;
  update call sites, do not weaken assertions).
- **AC-3** — `cargo check -p but-api -p gitbutler-tauri --all-targets` clean.
- **AC-4** — `cargo clippy -p but-api -p gitbutler-tauri --all-targets` clean (no new warnings).
- **AC-5** — `cargo test -p but-api branch_gates` / `group_delete` proofs (NEW) PASS: `branch_gates_read`
  returns the parsed gates map from a fixture repo; `branch_gates_update` writes gates.toml under
  `administration:write` and rejects a non-admin; `group_delete` removes a group under `administration:write`
  and rejects a non-admin. These are the new-fn proofs (facets 2 + 3); use `but-testsupport` fixtures, never
  `std::env::temp_dir().join(format!(…))`.
- **AC-6** — No doubly-suffixed `*_cmd_cmd` symbols exist (`grep -rn "_cmd_cmd" crates/` returns nothing).
- **AC-7** — `main.rs` consumes the `lib.rs` factory (no duplicated command list between them).

## RUNTIME_COMMANDS

```
test:  cargo test -p gitbutler-tauri mgmt_governance_commands_registered_and_invokable && cargo test -p but-api governance
check: cargo check -p but-api -p gitbutler-tauri --all-targets
lint:  cargo clippy -p but-api -p gitbutler-tauri --all-targets
fmt:   cargo fmt --check
```

## NEVER

- NEVER re-introduce a `_cmd` suffix on `#[but_api]` source fns (the macro adds it itself).
- NEVER duplicate the `generate_handler!` command list between `lib.rs` and `main.rs` (single source via the factory).
- NEVER write a test that mocks the command registration to pass — the test MUST exercise the real factory.
- NEVER stub `branch_gates_*` or `group_delete` — they must read/write the real `.gitbutler/*.toml` files.
- NEVER edit the locked PRD (`04-api-design.md`) — its `allow-*` wording is documented drift; implement against live `core:default`.
- NEVER touch `.spec/` files other than this task's evidence under `.tmp/MGMT-IPC-003/`.

## PROPOSED-BY

opencode orchestrator (kb-run-sprint SPEC-REPAIR), scope decisions confirmed by user (re-add group_delete;
register branch_gates in 06a; author + dispatch then continue).
