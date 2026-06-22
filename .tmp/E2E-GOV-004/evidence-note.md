# E2E-GOV-004 Fixture Evidence Note

This is fixture evidence only. It proves pending persistence and commit clearing in the Vite governance fixture, not committed governance refs or backend durability.

Validation logs:
- `check-governance-fixture.log`
- `test-e2e-governance-fixture.log`

AC/TC mapping:
- AC-1 / TC-1: `principal and group changes stay pending across tabs and clear after commit` stages one principal and one group change, switches through Principals, Groups, Branch Gates, Rules, and back, and asserts the pending banner and row markers persist.
- AC-2 / TC-2: the same test clicks `Commit changes` and asserts the pending banner, principal pending marker, and group pending marker clear.
- AC-3 / TC-3: the same test revisits Principals and Groups and asserts the edited `reviews:write` grants remain visible after pending clears.
- TC-4: the test asserts `cross-tab-pending.fixture-evidence` and `post-commit-clean.fixture-evidence`, and the fixture header says `Fixture governance harness` and `Not product E2E evidence`.
