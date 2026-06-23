# Sprint 06a Human Testing Steps

This tutorial verifies the Sprint 06a Governance UI gate in the GitButler desktop app.

Sprint folder:

```text
.spec/prds/governance/tasks/sprint-06a-governance-ui-scaffold-principals-groups/
```

## Gate

An admin opens Project Settings, sees the Permissions & Governance sidebar item, navigates to it,
observes four tabs, edits a principal's own-grant permission on the Principals tab, expands a group and
grants a permission on the Groups tab, and sees both changes remain pending with a commit banner until
clicking Commit changes.

## Important Setup Note

Permissions & Governance is intentionally admin-gated in the renderer:

```text
apps/desktop/src/components/settings/ProjectSettingsModalContent.svelte
isAdmin = userService.user?.role === "admin"
```

If you do not see Permissions & Governance, you are not testing as an admin user. Do not use pasted
`window` console snippets to force the state; this project does not expose a global Tauri API in normal
desktop web mode. Use one of these supported paths:

- A real development admin account.
- A local E2E/dev fixture that seeds `user.role = "admin"`.
- A temporary local-only code change to force `isAdmin = true`, used only for manual verification and
  reverted before committing.

The backend authorization check is separate from this sidebar visibility gate. A user can be an admin in
the sidebar and still be read-only if `governance_status_read` does not include `administration:write`.

## E2E User Flow Matrix

All E2E coverage for this gate must cover these user flows, not only the happy path.

| Flow       | User state                           | Action                        | Expected result                                                             |
| ---------- | ------------------------------------ | ----------------------------- | --------------------------------------------------------------------------- |
| GOV-E2E-01 | Admin                                | Open Project Settings         | Permissions & Governance is visible in the sidebar.                         |
| GOV-E2E-02 | Non-admin/member                     | Open Project Settings         | Permissions & Governance is absent; normal settings remain visible.         |
| GOV-E2E-03 | Admin without `administration:write` | Open Permissions & Governance | Page is visible, read-only message appears, write controls are disabled.    |
| GOV-E2E-04 | Admin                                | Open Permissions & Governance | Four tabs appear: Principals, Groups, Branch Gates, Rules.                  |
| GOV-E2E-05 | Admin                                | Edit a principal own-grant    | Pending marker appears and Commit changes banner appears.                   |
| GOV-E2E-06 | Admin                                | Edit a group grant            | Group change becomes pending and the same Commit changes banner remains.    |
| GOV-E2E-07 | Admin with pending changes           | Switch across all tabs        | Pending banner persists across Principals, Groups, Branch Gates, and Rules. |
| GOV-E2E-08 | Admin with pending changes           | Click Commit changes          | Pending markers clear and effective grants reflect the committed state.     |

## Prerequisites

Start from the repository root:

```bash
cd /Users/justinrich/Projects/gitbutler
```

Install dependencies if needed:

```bash
pnpm install
```

Run the narrow component checks that already cover the governance scaffold:

```bash
pnpm --filter @gitbutler/desktop test:ct:desktop -- tests/governance/SettingsModalLayoutAdmin.spec.ts
pnpm --filter @gitbutler/desktop test:ct:desktop -- tests/governance/ProjectSettingsModalContentGovernance.spec.ts
pnpm --filter @gitbutler/desktop test:ct:desktop -- tests/governance/GovernanceSettingsTabs.spec.ts
pnpm --filter @gitbutler/desktop test:ct:desktop -- tests/governance/GovernanceSettingsCrossTabPending.spec.ts
pnpm --filter @gitbutler/desktop test:ct:desktop -- tests/governance/GovernanceSettingsCommit.spec.ts
```

These component tests are useful preflight checks. They do not replace the human gate or a headed E2E run.

## Manual Test Steps

### 1. Start The Desktop App

Run the desktop app locally:

```bash
pnpm dev:desktop
```

Expected result: the GitButler desktop app opens and the workspace is usable.

### 2. Verify Admin Sidebar Visibility

Sign in or seed the app as an admin user.

Open Project Settings through the existing app shortcut or settings entry.

Expected result:

- The Project settings modal opens.
- The sidebar includes Permissions & Governance.
- Existing sidebar entries such as Project, Git stuff, AI options, and Experimental still appear.

Capture evidence: `admin-visible`.

### 3. Verify Non-Admin Sidebar Absence

Sign in or seed the app as a non-admin/member user.

Open Project Settings again.

Expected result:

- The Project settings modal opens.
- Permissions & Governance is absent.
- Normal project settings are still visible.

Capture evidence: `non-admin-hidden`.

### 4. Open Permissions & Governance As Admin

Return to an admin user and open Project Settings.

Click Permissions & Governance.

Expected result:

- The page heading is Permissions & Governance.
- Four tabs are visible in this order: Principals, Groups, Branch Gates, Rules.
- No `Settings page governance not Found.` fallback is visible.

Capture evidence: `four-tabs`.

### 5. Verify Read-Only Admin State

Seed or sign in as an admin user whose governance access lacks `administration:write`.

Open Permissions & Governance.

Expected result:

- The page remains visible.
- A read-only message explains that `administration:write` is required.
- Principal, group, branch gate, rule, and Commit changes controls are disabled or absent.

Capture evidence: `read-only-admin`.

### 6. Edit A Principal Own Grant

Use an admin user with `administration:write`.

On the Principals tab:

1. Click a principal row.
2. Toggle an own-grant permission, not an inherited permission.
3. Save the principal change if the editor requires an explicit save.

Expected result:

- The changed principal or permission row shows a pending marker.
- A Commit changes banner appears.
- The banner count is greater than zero.
- Inherited permissions remain read-only and are not shown as editable own grants.

Capture evidence: `principal-pending`.

### 7. Edit A Group Grant

Switch to the Groups tab.

1. Expand a group.
2. Toggle or grant a permission on that group.

Expected result:

- The group remains expanded.
- The changed group grant is visible.
- The pending banner remains visible.
- The pending banner does not reset when moving from Principals to Groups.

Capture evidence: `group-pending`.

### 8. Verify Pending State Across Tabs

With both principal and group changes still pending, click through:

1. Principals
2. Groups
3. Branch Gates
4. Rules
5. Back to Principals
6. Back to Groups

Expected result:

- The Commit changes banner remains visible on every tab.
- Returning to Principals still shows the principal pending marker.
- Returning to Groups still shows the group pending state.

Capture evidence: `cross-tab-pending`.

### 9. Commit Governance Changes

Click Commit changes.

Expected result:

- The commit action runs.
- Pending markers clear.
- The Commit changes banner disappears.
- The edited principal and group grants remain visible as committed/effective state.

Capture evidence: `post-commit-clean`.

## Headed E2E Coverage

The repo has two E2E tracks for this gate:

- `E2E-GOV-001`..`004` cover a watchable Vite governance fixture. These are useful for local headed
  verification and screenshots, but they are fixture evidence, not product-backend proof.
- `E2E-GOV-101`..`104` define the real desktop-product Playwright tasks that must drive Project
  Settings in the GitButler app.

Run the fixture preflight first:

```bash
pnpm --filter @gitbutler/e2e run check:governance-fixture
pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture
pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture:headed
```

The fixture UI itself says `Fixture governance harness` and `Not product E2E evidence`; keep those labels
visible in captured evidence.

When the desktop-product E2E tasks are implemented, run the product specs headed:

```bash
PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-access.spec.ts
PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-tabs-readonly.spec.ts
PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-pending-edits.spec.ts
PLAYWRIGHT_UI=1 pnpm test:e2e:playwright -- governance-settings-commit-flow.spec.ts
```

Required E2E assertions:

- Admin sees Permissions & Governance.
- Non-admin does not see Permissions & Governance.
- Read-only admin can view but cannot edit.
- Four governance tabs render.
- Principal own-grant edit shows pending state.
- Group grant edit shows pending state.
- Pending banner persists across tab switches.
- Commit changes clears pending state and leaves effective grants updated.

## Evidence Checklist

Attach or record these artifacts for sprint closeout:

- `admin-visible` screenshot or video frame.
- `non-admin-hidden` screenshot or video frame.
- `read-only-admin` screenshot or video frame.
- `four-tabs` screenshot or video frame.
- `principal-pending` screenshot or video frame.
- `group-pending` screenshot or video frame.
- `cross-tab-pending` screenshot or video segment.
- `post-commit-clean` screenshot or video frame.
- Command output from the relevant component tests.
- Command output from the headed governance E2E run when available.

## Troubleshooting

If Permissions & Governance is missing for the admin case, verify `userService.user?.role` is `admin`.

If the page is visible but controls are disabled, verify the backend access response includes
`administration:write`.

If a `SyntaxError: Unexpected identifier 'window'` appears, the setup code was likely pasted into the
wrong runtime. Do not use console snippets for this gate; seed the user through the app, fixture, or a
temporary local-only code change that is reverted before commit.

If the pending banner disappears while switching tabs, the pending store is scoped too low. The expected
behavior is that the page-level pending state survives tab navigation until Commit changes succeeds.
