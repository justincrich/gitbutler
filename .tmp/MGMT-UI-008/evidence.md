MGMT-UI-008 evidence

RED:
- Wrote `apps/desktop/tests/governance/GroupsList.spec.ts` before `GroupsList.svelte`.
- First run: `pnpm -F @gitbutler/desktop test:ct:desktop -- GroupsList`
- Captured in `.tmp/MGMT-UI-008/red-groupslist.log`.
- Initial failure was environmental in this isolated worktree: missing `node_modules` made Playwright unavailable (`unknown command 'test'` / pnpm warned `node_modules` missing). After linking installed workspace dependencies and generating `.svelte-kit`, the same focused CT failed against incomplete behavior until the component/test harness were corrected; intermediate failure/pass logs are also under `.tmp/MGMT-UI-008/`.

GREEN / REFACTOR:
- `pnpm -F @gitbutler/desktop test:ct:desktop -- GroupsList`
  - Final: 8 passed.
  - Log: `.tmp/MGMT-UI-008/final-groupslist.log`
- `pnpm -F @gitbutler/desktop test:ct:desktop -- GovernanceSettingsTabs`
  - Final: 3 passed, including Groups tab mounting `GroupsList`.
  - Log: `.tmp/MGMT-UI-008/final-governance-settings-tabs.log`
- `pnpm -F @gitbutler/desktop check`
  - Final: 0 errors, 0 warnings.
  - Log: `.tmp/MGMT-UI-008/final-desktop-check.log`
- `pnpm -F @gitbutler/desktop build`
  - Final: exit 0; adapter-static wrote `build`.
  - Log: `.tmp/MGMT-UI-008/final-desktop-build.log`
- `pnpm lint`
  - Final: failed at `prettier --check .` on 131 existing Markdown/design files outside this task, including `.spec/**`, `apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md`, and `RULES.md`.
  - Task-owned `GroupsList.svelte` and `GroupsList.spec.ts` were formatted and no longer appear in the final lint warning list.
  - Log: `.tmp/MGMT-UI-008/final-root-lint.log`

Backend / product boundary notes:
- `group_create`, `group_grant`, `group_revoke`, `group_add_member`, `group_remove_member`, `group_delete`, and `group_list` exist in Tauri/backend surfaces.
- Post-dependency refresh confirmed `group_revoke` is registered in `crates/gitbutler-tauri/src/lib.rs`, implemented in `crates/gitbutler-tauri/src/governance.rs`, and generated in `packages/but-sdk/src/generated/index.d.ts` as `groupRevoke(projectId, targetRef, group, authorities)`.
- `GroupsList.svelte` keeps the established desktop backend injection pattern and calls `BACKEND.invoke("group_revoke", { projectId, targetRef, group, authorities })`, which now targets the real command shape. AC-7 is no longer blocked by missing backend support.
- Branch-gate reference data is not exposed to this component yet. `GroupsList` therefore accepts an injected/testable `gateReferencedGroups: string[]` boundary and uses it to block last-member removal with a warning modal before `group_remove_member`.

Post-dependency refresh:
- `pnpm -F @gitbutler/desktop test:ct:desktop -- GroupsList`
  - Initial post-refresh attempt failed before test execution because Playwright revision 1208 was not installed locally.
  - After `pnpm -F @gitbutler/desktop exec playwright install chromium`, final result: 8 passed.
  - Logs: `.tmp/MGMT-UI-008/post-dependency-groupslist.log`, `.tmp/MGMT-UI-008/post-dependency-groupslist-final.log`, `.tmp/MGMT-UI-008/post-dependency-playwright-install-desktop.log`
- `pnpm -F @gitbutler/desktop test:ct:desktop -- GovernanceSettingsTabs`
  - Initial post-refresh attempt failed for the same missing Playwright browser revision.
  - Final result after browser install: 3 passed.
  - Logs: `.tmp/MGMT-UI-008/post-dependency-governance-settings-tabs.log`, `.tmp/MGMT-UI-008/post-dependency-governance-settings-tabs-final.log`
- `pnpm -F @gitbutler/desktop check`
  - Final: 0 errors, 0 warnings.
  - Log: `.tmp/MGMT-UI-008/post-dependency-desktop-check.log`
- `pnpm -F @gitbutler/desktop build`
  - Final: exit 0; adapter-static wrote `build`.
  - Log: `.tmp/MGMT-UI-008/post-dependency-desktop-build.log`
