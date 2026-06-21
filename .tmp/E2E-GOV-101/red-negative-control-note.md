Negative control for E2E-GOV-101:

Temporarily changed the admin sidebar assertion for "Permissions & Governance" from
`toHaveCount(1)` to `toHaveCount(0)`, then ran:

`pnpm --filter @gitbutler/e2e run test:e2e:playwright -- governance-settings-access.spec.ts --workers=1`

The run failed because the real Project Settings sidebar resolved one exact
"Permissions & Governance" button for the seeded admin session. This proves the
test catches a regression where the admin governance entry is missing or hidden.
The temporary mutation was reverted before the final green run and commit.
