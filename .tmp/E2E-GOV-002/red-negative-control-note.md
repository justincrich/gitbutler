# E2E-GOV-002 Red / Negative-Control Evidence

Temporary mutation used for evidence only, then reverted before commit:
- In `GovernanceFixtureApp.tsx`, changed `canWrite` so the `read-only-admin` persona incorrectly had write authority.

Command captured:
- `GOVERNANCE_FIXTURE_PORT=4176 pnpm --filter @gitbutler/e2e run test:e2e:governance-fixture --grep "read-only admin"`

Evidence log:
- `red-negative-control-read-only-write-enabled.log`

Result:
- The run failed because the read-only `administration:write` message was absent.
- This proves E2E-GOV-002 AC-1 / TC-1 catches a regression where read-only admins are treated as writable admins.

The temporary mutation was reverted before the green validation and commit.
