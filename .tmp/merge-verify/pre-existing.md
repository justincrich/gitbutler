# Verification Notes — sprint-06b cumulative merge into HEAD

## Summary

All 4 conflict resolutions are correct and verified. Typecheck is clean (0 errors, 0 warnings).
Rust branch_gates tests pass (13/13). All component tests that exercise the conflict-resolution
surfaces pass. The 2 remaining component-test failures are **pre-existing on the incoming
sprint-06b branch** (verified absent `governance-rules-control` on MERGE_HEAD; repo-wide prettier
drift in 162 files, 0 of which are conflict files) — NOT caused by this conflict resolution.

## cargo-branch-gates.txt — PASS

```
branch_gates.rs:           9 passed, 0 failed
branch_gates_governance.rs: 4 passed, 0 failed
```
These exercise `effective_authority_for_principal` / `enforce_branch_gates_read_authority`
(the two functions unioned in the governance.rs conflict resolution). Zero compile errors.

## desktop-check.txt — PASS (0 errors, 0 warnings)

`svelte-check found 0 errors and 0 warnings`. Required building workspace library packages
(`@gitbutler/ui`, `@gitbutler/shared`, `@gitbutler/core` via their `package` script) — the fresh
worktree shipped without `dist/`, which initially surfaced 1228 `Cannot find module '@gitbutler/*'`
errors that were purely environmental.

### Resolution-decision correction (error block)

Initial resolution dropped UI-012's bottom `governance-read-failure` error block as "redundant"
with HEAD's top `GovernanceErrorMessage` banner and patched HEAD's banner with
`error={pendingStore.error?.message}` to satisfy the type system. This was **incorrect**:

- The incoming sprint-06b branch changed `pendingStore.error` from `string` to a structured
  `GovernanceReadFailure = { code; message; remediationHint? }` type.
- UI-012's `governance-read-failure` block renders that structured type and has a dedicated test
  (`GovernanceIPCFailureBanner` in GovernanceA11yIPC.spec.ts).
- HEAD's `GovernanceErrorMessage` banner has **no test coverage** and was incompatible with the
  new structured type.

**Corrected resolution:** kept UI-012's `governance-read-failure` block (placed at the top of
HEAD's `{:else}` branch, alongside the preserved read-only/pending banners and Tabs), dropped
HEAD's untested generic banner. This is the faithful integration: HEAD's `isNotConfigured`
first-run flow + UI-012's structured-error rendering (the intended evolution for the type the
incoming branch introduced).

## ct-governance.txt — 2 PRE-EXISTING failures (not caused by conflict resolution)

All conflict-resolution exercises pass: `GovernanceSettingsTabs` (4 tabs + ariaLabel),
`RulesListPrincipalId` (MGMT-UI-010 feature), `GovernanceSettingsReadOnly`, `GovernanceIPCFailureBanner`
(structured error block), `BuildGates:157 typecheck-gate`, plus 30+ others.

### Failure 1 — `GovernanceReadOnlyA11y` (GovernanceA11yIPC.spec.ts:159) — PRE-EXISTING

```
await expect(component.getByTestId("governance-rules-control")).toBeDisabled();
```

The test expects a `governance-rules-control` element in the Rules tab. This is a **stale
expectation internal to the incoming sprint-06b branch**: MGMT-UI-010 (RulesList principalId)
replaced the placeholder "Add rule" control with a `<RulesList principalId>` view, but
MGMT-UI-011's a11y test was not updated. Verified pre-existing:

```
$ git show MERGE_HEAD:apps/desktop/src/components/governance/GovernanceSettings.svelte | grep -c governance-rules-control
0
```

MERGE_HEAD's own GovernanceSettings.svelte (which this test ships alongside) does NOT contain
`governance-rules-control` either. The test fails identically on the incoming branch before any
conflict resolution. Cannot be fixed within constraints (would require editing the test file —
not one of the 4 conflict files — or reverting MGMT-UI-010's RulesList feature).

### Failure 2 — `BuildGates:204 passes the repository lint gate` — PRE-EXISTING

Runs `pnpm lint` (turbo `globallint`), which fails with "Code style issues found in 162 files."
All 162 are pre-existing prettier drift — overwhelmingly `.spec/` markdown task docs plus
`LocalReviewView.svelte` / `LocalReviewView.spec.ts`. **Zero** of the 4 conflict files appear in
the lint warnings (confirmed: `grep -c` for the conflict file paths in lint output = 0; the 4
files pass `prettier --check` cleanly). This is a repo-wide formatting debt unrelated to the
merge.

## Conflict-resolution decisions (final)

1. **governance.rs** — kept both function groups adjacently (HEAD's `principal_kind_*` family +
   UI-012's `enforce_branch_gates_read_authority` / `effective_authority_for_principal`), sharing
   the trailing `}` to close the last function.
2. **but-authz lib.rs** — unioned the `config::{...}` re-export list (both `load_permissions_wire`
   and `gates_path`) and preserved HEAD's `denial::{...}` / `menu::{...}` blocks.
3. **GovernanceSettings.svelte** — HEAD's `isNotConfigured` first-run flow as backbone; unioned
   imports (Button kept for setup-guide); read-only banner above Tabs; Tabs with UI-012's
   `ariaLabel`; UI-012's RulesList principalId rules TabContent (MGMT-UI-010); UI-012's
   `governance-read-failure` structured-error block (corrected from initial HEAD-banner choice).
4. **GovernanceSettingsHarness.svelte** — pure union of both sides' imports/props/defaults
   (URL_SERVICE + IpcError; notConfigured + readFailure).
