# E2E-GOV-004 Red / Negative-Control Evidence

Temporary mutation used for evidence only, then reverted before commit:
- In `GovernanceFixtureApp.tsx`, changed `savePrincipal()` so it did not persist the `reviews:write` own grant on `settings-agent`.

Command captured:
- `GOVERNANCE_FIXTURE_PORT=4176 pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep "clear after commit"`

Evidence log:
- `red-negative-control-principal-grant-persistence.log`

Result:
- The run failed at the scoped `principals-list-row-settings-agent` assertion because the row did not contain `reviews:write`.
- This proves E2E-GOV-004 AC-3 / TC-3 catches a regression where pending indicators clear but the committed principal grant is not retained.

The temporary mutation was reverted before the green validation and commit.
