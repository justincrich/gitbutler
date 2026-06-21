# Pre-existing Issues

## `pnpm lint`

Command:

```bash
pnpm lint
```

Exit code: `1`

After formatting the files touched by `DEP-E2E-UI-001-SURFACE`, lint still fails in the root `globallint` Prettier check on repo-wide pre-existing Markdown and E2E formatting debt outside the task write scope.

The exact failure class reported by lint is:

```text
//:globallint: > prettier --check . && eslint . && pnpm run oxlint && pnpm knip:non-prod && pnpm knip:prod
//:globallint: Checking formatting...
//:globallint: [warn] .spec/artifacts/team-product/01-definition-of-done.md
//:globallint: [warn] .spec/artifacts/team-product/02-feature-inventory.md
//:globallint: [warn] .spec/artifacts/team-product/03-gap-analysis.md
//:globallint: [warn] .spec/artifacts/team-product/04-synthesis-report.md
//:globallint: [warn] .spec/prds/actions.superseded/00-overview.md
//:globallint: [warn] .spec/prds/actions.superseded/01-scope.md
//:globallint: [warn] .spec/prds/check-runner/00-overview.md
//:globallint: [warn] .spec/prds/governance/00-overview.md
//:globallint: [warn] .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-009-branch-gates-list.md
//:globallint: [warn] .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-010-ruleslist-principalid.md
//:globallint: [warn] apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md
//:globallint: [warn] e2e/playwright/src/governance.ts
//:globallint: [warn] e2e/playwright/tests/governance-fixture/governance-fixture.spec.ts
//:globallint: [warn] e2e/playwright/tests/governance-settings-access.spec.ts
//:globallint: [warn] RULES.md
//:globallint: [warn] Code style issues found in 134 files. Run Prettier with --write to fix.
//:globallint:  ELIFECYCLE  Command failed with exit code 1.
```

The full exact command output is captured by the evidence harvester in `lint-output.txt`.
