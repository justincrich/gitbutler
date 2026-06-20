# MGMT-UI-003 blocker report

## Classification

`recoverable_dependency_missing`

## Blocking dependency

The generated renderer SDK contract does not expose the data or mutation API required by
MGMT-UI-003:

- `GovernanceStatus` lacks `pendingCount`.
- `GovernanceStatus` lacks `hasAdminWrite`.
- No generated SDK governance commit function exists for committing `.gitbutler/*.toml`
  with message `chore: update governance config`.

Because the task explicitly forbids inventing production placeholders, deriving pending state
optimistically, or writing `.gitbutler/*.toml` directly from the renderer, a production-honest
implementation cannot be completed in this branch.

## Probe commands

```bash
sed -n '1590,1655p' packages/but-sdk/src/generated/index.d.ts
```

Observed:

```ts
/** Serializable authority set returned by generated governance API wrappers. */
export type GovernanceStatus = {
  /** Effective functional authority tokens for the caller. */
  authorities: Array<string>;
};
```

```bash
rg -n "governance.*commit|commit.*governance|Governance.*Commit|governanceCommit|governance_commit|pendingCount|hasAdminWrite" packages/but-sdk/src/generated/index.d.ts packages/but-sdk/src/generated/index.js crates packages apps/desktop/src -g '!target' -g '!node_modules'
```

Observed:

- Generated SDK exports `governanceStatusRead` only for governance status.
- Generated SDK does not export `pendingCount`.
- Generated SDK does not export `hasAdminWrite`.
- Generated SDK does not export a governance commit function.
- Rust/Tauri contains a separate `governance_pending` command with `pendingCount`, but it is not
  present in `packages/but-sdk/src/generated`.

```bash
rg -n "governance_(pending|status|commit)|governancePending|governanceStatus|pendingCount|hasAdminWrite" crates/gitbutler-tauri/src packages/but-sdk/src/generated -g '!target'
```

Observed:

- `crates/gitbutler-tauri/src/governance.rs` defines `GovernancePending.pending_count`.
- `crates/gitbutler-tauri/src/lib.rs` registers `governance_pending`.
- `packages/but-sdk/src/generated/index.d.ts` exposes only
  `governanceStatusRead(projectId): Promise<GovernanceStatus>`.

```bash
sed -n '320,350p' crates/but-api/src/legacy/governance.rs
```

Observed:

```rust
/// Return the caller's own effective governance authorities (`governance_status_read`).
#[but_api(napi, GovernanceStatus)]
pub fn governance_status_read(ctx: &Context) -> anyhow::Result<AuthoritySet> {
    let repo = ctx.repo.get()?;
    let target_ref = target_ref_from_ctx(ctx, None)?;
    let config = load_governance_config(&repo, &target_ref)?;
    let caller = but_authz::resolve_principal_from_env(&config)?;
    Ok(but_authz::effective_authority(&caller, &config))
}
```

## Why no RED/GREEN test files were produced

The behavioral ACs require production code to call real generated SDK fields/functions. Writing CT
tests first would be possible only by mocking a contract that does not exist in production, and the
task explicitly allows CT mocks only for component exercise while banning production invention. Since
the missing SDK surface is known before implementation, the honest result is a dependency blocker
rather than assertion-level RED evidence.

## Suggested follow-up

Create a `DEPENDENCY` or `SPEC-REPAIR` task to expose a single generated SDK contract that includes:

- `governanceStatusRead(projectId)` returning `pendingCount` and `hasAdminWrite`, or a separate
  generated `governancePending(projectId, targetRef)` contract plus an explicit status read contract
  containing `hasAdminWrite`.
- A generated SDK governance commit function that commits pending governance file edits through the
  Tauri/but-api boundary with a supplied commit message.
