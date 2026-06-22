# E2E-GOV-003 Red / Negative-Control Evidence

Temporary mutation used for evidence only, then reverted before commit:
- In `GovernanceFixtureApp.tsx`, changed the inherited principal grant checkbox from `disabled` to `readOnly`.

Command captured:
- `GOVERNANCE_FIXTURE_PORT=4176 pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep "principal and group changes"`

Evidence log:
- `red-negative-control-inherited-grant-editable.log`

Result:
- The run failed at `getByLabel("contents:write inherited from group eng").toBeDisabled()`.
- This proves E2E-GOV-003 AC-2 / TC-2 catches a regression where inherited grants become editable while staging an own grant.

The temporary mutation was reverted before the green validation and commit.
