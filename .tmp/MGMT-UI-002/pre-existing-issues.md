`pnpm lint` is blocked by pre-existing Prettier warnings outside the MGMT-UI-002
scope. The failing files reported in `.tmp/MGMT-UI-002/lint-output.txt` are
unrelated Markdown/spec/design files, including `.spec/prds/governance/ROADMAP.md`,
multiple `.spec/prds/governance/tasks/**` files, `.spec/reviews/red-hat-20260620T051414Z.md`,
`apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md`, and `RULES.md`.

Scoped formatting passed for:

- `apps/desktop/src/components/settings/ProjectSettingsModalContent.svelte`
- `apps/desktop/tests/governance/SettingsModalLayoutAdmin.spec.ts`
