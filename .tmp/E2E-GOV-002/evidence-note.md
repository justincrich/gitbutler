# E2E-GOV-002 Fixture Evidence Note

This is fixture evidence only. It proves disabled UI state in the Vite governance fixture, not server-side authorization enforcement.

Validation logs:
- `check-governance-fixture.log`
- `test-e2e-governance-fixture.log`

AC/TC mapping:
- AC-1 / TC-1: `read-only admin can view governance settings but cannot edit` selects Read-only admin, opens governance, and asserts the visible message names `administration:write`.
- AC-2 / TC-2: the same test opens the principal editor and asserts `reviews:write own grant` and `Save changes` are disabled.
- AC-3 / TC-3: the same test inspects Groups, Branch Gates, Rules, and commit controls, asserting write controls are disabled or absent.
- Fixture labeling: the test asserts `read-only-admin.fixture-evidence`, and the fixture header says `Fixture governance harness` and `Not product E2E evidence`.
