# E2E-GOV-003 Fixture Evidence Note

This is fixture evidence only. It proves pending UI state from visible fixture controls, not product backend persistence.

Validation logs:
- `check-governance-fixture.log`
- `test-e2e-governance-fixture.log`

AC/TC mapping:
- AC-1 / TC-1: `principal and group changes stay pending across tabs and clear after commit` opens the principal row, checks `reviews:write own grant`, saves, and asserts the principal pending marker plus `1 pending changes`.
- AC-2 / TC-2: the same test asserts `contents:write inherited from group eng` remains disabled while the own grant is staged.
- AC-3 / TC-3: the same test switches to Groups, expands `eng`, checks `reviews:write`, and asserts the group pending marker plus `2 pending changes`.
- Fixture labeling: the test asserts `principal-pending.fixture-evidence` and `group-pending.fixture-evidence`, and the fixture header says `Not product E2E evidence`.
