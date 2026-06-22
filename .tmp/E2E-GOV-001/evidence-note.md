# E2E-GOV-001 Fixture Evidence Note

This is fixture evidence only. It proves the Vite governance fixture UI flow, not product backend authorization or committed governance refs.

Validation logs:
- `check-governance-fixture.log`
- `test-e2e-governance-fixture.log`

AC/TC mapping:
- AC-1 / TC-1: `admin can see the governance settings entry and all four tabs` selects the Admin persona, opens `Permissions & Governance`, and asserts Project/AI entries remain visible with the governance entry.
- AC-2 / TC-2: `non-admin cannot see the governance settings entry` selects the Member persona and asserts `Permissions & Governance` and `governance-settings` are absent while Project and AI options remain visible.
- AC-3 / TC-3: `admin can see the governance settings entry and all four tabs` asserts Principals, Groups, Branch Gates, and Rules tabs render and the not-found fallback is absent.
- TC-4: the same test asserts fixture-only labels including `admin-visible.fixture-evidence`, `four-tabs.fixture-evidence`, `Fixture governance harness`, and `Not product E2E evidence`; the member test asserts `non-admin-hidden.fixture-evidence`.
