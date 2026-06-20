# Adapter Static Fallback Blocker

## Finding

`apps/desktop/svelte.config.js:14` contains a pre-existing adapter-static SPA fallback:

```js
fallback: "index.html",
```

This repair did not introduce that setting. The source commit for this repair only changed the governance settings scaffold, registration, desktop CT harness, CT spec, package metadata, and lockfile.

## Write Scope

This repair's `write_allowed` list does not include `apps/desktop/svelte.config.js`, so this task cannot change the adapter-static fallback without violating the repair contract.

## Reviewer Impact

The current SvelteKit reviewer policy treats adapter-static fallback mode as non-compliant for static SvelteKit work. Because `apps/desktop/svelte.config.js` still sets `fallback: "index.html"`, reviewer approval is blocked even though the governance modal-state task did not add a route, server load, server layout, or SPA fallback.

## Recommended Follow-Up

Create follow-up task `SPEC-REPAIR-DESKTOP-ADAPTER-STATIC-FALLBACK`.

Success criteria:

- Either remove `fallback: "index.html"` safely from `apps/desktop/svelte.config.js` and verify the desktop static build plus desktop route behavior still work.
- Or document and encode an explicit architecture exception for the Tauri desktop SPA fallback so SvelteKit reviewers do not block unrelated modal-state tasks.
