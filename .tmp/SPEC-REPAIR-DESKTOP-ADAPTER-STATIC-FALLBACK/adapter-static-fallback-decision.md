# SPEC-REPAIR-DESKTOP-ADAPTER-STATIC-FALLBACK Evidence

## Decision

Removed `fallback: "index.html"` from `apps/desktop/svelte.config.js`.

The first fallback-free build passed but produced no HTML route entry because
`apps/desktop/src/routes/+layout.ts` had `ssr = false`, `csr = true`, and
`prerender = false`. That is not a safe static desktop output because Tauri needs
a concrete HTML app shell.

The safe replacement is to keep the desktop root layout as a client-side app
shell and set `export const prerender = true`. Dynamic project routes remain
non-prerendered via `apps/desktop/src/routes/[projectId]/+layout.ts`, while the
desktop package gets static entry files without using adapter-static fallback.

## RED Evidence

Initial focused check failed because the desktop adapter config still contained
the disallowed fallback:

```text
apps/desktop/svelte.config.js:14:			fallback: "index.html",
```

## GREEN Evidence

Required checks after removing the fallback and enabling root app-shell prerender:

```text
pnpm -F @gitbutler/desktop build
> Using @sveltejs/adapter-static
  Wrote site to "build"
  ✔ done

pnpm -F @gitbutler/desktop check
svelte-check found 0 errors and 0 warnings
```

Static route artifact check:

```text
apps/desktop/build/index.html
apps/desktop/build/onboarding.html
apps/desktop/build/onboarding/clone.html
```

Focused policy check:

```text
PASS: apps/desktop/svelte.config.js has no adapter-static fallback.
13:export const prerender = true;
PASS: apps/desktop/build/index.html exists.
```

## Scope

This change is narrow to `apps/desktop`. It does not create an exception for SPA
fallback usage elsewhere and does not touch the preserved
`SPEC-REPAIR-MGMT-UI-001` worktree or branch.
