# E2E-GOV-001 Red / Negative-Control Evidence

Temporary mutation used for evidence only, then reverted before commit:
- In `GovernanceFixtureApp.tsx`, changed the normal settings labels from `Project` and `AI options` to negative-control labels.

Command captured:
- `GOVERNANCE_FIXTURE_PORT=4176 pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep "admin can see"`

Evidence log:
- `red-negative-control-admin-normal-settings.log`

Result:
- The run failed at the admin test because `getByRole("button", { name: "AI options" })` was not visible.
- This proves E2E-GOV-001 AC-1 / TC-1 catches a regression where admin sees `Permissions & Governance` but normal settings entries are not still visible.

The temporary mutation was reverted before the green validation and commit.
