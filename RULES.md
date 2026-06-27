# RULES.md

This file provides guidance to Claude Code (claude.ai/code), Codex, Copilot, and
other agents when working with code in this repository. `CLAUDE.md` and
`AGENTS.md` are pointers to this file.

GitButler is a Rust + SvelteKit + React/Electron monorepo: a Git-based version
control system with a Tauri desktop GUI, an Electron "lite" desktop app, a web
app, and the `but` CLI/TUI — all driving one shared Rust engine.

## Instruction Precedence

Apply all relevant instruction files. If instructions conflict, resolve in this
order:

1. Explicit human instructions
2. Nearest nested `AGENTS.md` (overrides this file for files in its subtree)
3. This file (`RULES.md`)

## Repo Map

- `crates/` — ~65 Rust crates. The backend engine and the `but` CLI/TUI.
- `apps/desktop/` — Tauri desktop GUI frontend. **SvelteKit** + `adapter-static`.
- `apps/web/` — Web app. **SvelteKit** + `adapter-vercel`.
- `apps/lite/` — "Lite" desktop app. **Electron 41 + React 19** (TanStack
  Query/Router + Redux).
- `packages/` — Shared TypeScript packages: `but-sdk` (generated TS SDK for the
  Rust API), `ui` (shared components), `core`, `shared`, ESLint plugins.
- `e2e/` — Playwright, WebdriverIO, and blackbox end-to-end tests.

## Big-Picture Architecture

Read this before making cross-cutting changes — it isn't obvious from any single
file.

### One engine, four frontends

The Rust backend in `crates/` is the single source of truth. It is consumed by
four callers through one shared API surface:

- **Tauri desktop** (`gitbutler-tauri`) → SvelteKit GUI in `apps/desktop/`
- **Electron lite** (via `but-napi` N-API bindings) → React app in `apps/lite/`
- **`but` CLI** (`crates/but/`)
- **TUI** (also under the `but` family)

**`but-api` is the API boundary** for all four callers. Outer callers prefer
existing `but-api` functions when they fit; lower-level crates must **not** depend
on `but-api`. Transport DTOs live at the boundary (commonly a local `json`
module) and are converted to domain types before calling lower-level crates.
Preserve existing `but-api` macro, transport, serialization, and conversion
patterns instead of hand-rolling parallel wrappers.

### Legacy `gitbutler-*` → modern `but-*` migration

Crates fall into two generations:

- `gitbutler-*` — **legacy-heavy.** Treat as legacy; preserve local ownership and
  nearby patterns for localized fixes. Do **not** introduce new `gitbutler-*`
  usage in newer code unless a surrounding legacy boundary requires it. Migrate
  only when necessary, tiny, tested, and behavior-neutral. Be extra careful with
  `gitbutler-reference`, `gitbutler-branch-actions`, `gitbutler-repo`, and
  `gitbutler-url` (see `DEVELOPMENT.md` "Code Hitlist").
- `but-*` — modern backend utilities and services. Prefer `but_ctx::Context`
  workspace helpers and newer `but-*` APIs. Minimize new `VirtualBranchesHandle`
  usage.

### Git access: `gix` over `git2`

Use `gix` (gitoxide) for new repository logic. Treat `git2` and
`Context::git2_repo` as legacy/boundary-only escape hatches (libgit2
checkout/index, hooks, transport/auth, or code that can't move yet). Prefer
repository APIs over shelling out to `git`, except at shell/executable boundaries
(hooks, debug tooling, tests, Git interop). Preserve Git graph semantics — do not
dedupe, reorder, or smooth graph data unless Git semantics allow it. Keep Git
paths, refnames, commit messages, and diff payloads byte-preserving until
UI/API boundaries.

### Workspace / graph model

Before changing or reviewing code that derives graph/workspace/branch/stack/commit
relationships, reachability, dependencies, ordering, operation targets, or Git
graph/history/ref-placement mutations, use **`crates/WORKSPACE_MODEL.md`** as the
reference. In short: prefer commit IDs and refs at API boundaries, convert to
operation-local selectors inside editor-backed operations, use `but_graph::Graph`
for relationship/reachability questions, use `but_rebase::graph_rebase::Editor`
for Git graph/history/ref rewrites, and treat `but_graph::Workspace` /
`but_workspace::RefInfo` as lossy presentation views.

### SDK generation flow

`packages/but-sdk` is **generated** from Rust types/APIs. After changing Rust
APIs or types exposed through `@gitbutler/but-sdk`, run `pnpm build:sdk &&
pnpm format` — this regenerates `packages/but-sdk/src/generated`. Don't hand-edit
generated files.

## Commands

### Rust

```bash
cargo build                       # build everything (needs Tauri system deps)
cargo build -p but                # build just the `but` CLI (no Tauri deps)
cargo test                        # run all Rust tests
cargo test -p <crate>             # test a single crate
cargo test -p <crate> <test-name> # run a single test (narrowest check first)
cargo check -p <crate> --all-targets
cargo fmt                         # format (run before committing)
cargo clippy --all-targets        # lint
cargo machete                     # check for unused deps (run when deps change)
```

Run the **narrowest relevant test or check first** — do not default to
workspace-wide runs. Use workspace-wide checks (`make check`, `make clippy`) only
when the change affects shared contracts or multiple crates. **Do not use
`cargo nextest`** for routine validation — use plain `cargo test`.

### JavaScript / TypeScript

```bash
pnpm install                      # install (corepack-enabled pnpm)

pnpm dev:desktop                  # run Tauri desktop GUI (debug: LOG_LEVEL=debug pnpm dev:desktop)
pnpm dev:web                      # run web app
pnpm dev:lite                     # run Electron lite app
pnpm dev:ui                       # run shared-UI Storybook

pnpm check                        # typecheck all (turbo)
pnpm -F @gitbutler/lite check     # typecheck the lite app (exact command for lite)

pnpm test                         # all unit tests (Vitest)
pnpm test:ct                      # component tests (@gitbutler/ui, Playwright)
pnpm test:e2e:playwright          # Playwright E2E
pnpm test:e2e                     # WebdriverIO E2E (non-Tauri)
pnpm test:e2e:blackbox            # blackbox E2E

pnpm lint                         # prettier + eslint + oxlint + knip
pnpm format                       # prettier --write
pnpm fix                          # eslint --fix
pnpm isgood                       # check + lint (verify shortcut)
pnpm begood                       # format + fix (autofix shortcut)
```

**Running a single frontend test:**

```bash
# Component test (Playwright) — pass file/pattern, no -t flag:
pnpm test:ct -- HardWrapPlugin.spec
# Unit test (Vitest) — cd into the package, use -t:
cd apps/desktop && pnpm test -- -t myComponent.test
cd packages/ui && pnpm test -- -t BranchLane
```

See `frontend.md` for full frontend-test detail.

## Conventions

### Rust

- Use Git/GitButler domain names; keep types/helpers in the crate/module that owns
  the concept. Avoid vague names (`Manager`, `Service`, `Helper`, `Util`,
  `Processor`) unless nearby code already uses them.
- Solve the present problem directly — avoid speculative abstractions, one-use
  traits, fake extension points, and public APIs larger than real callers need.
- Avoid drive-by refactors while fixing a specific bug.
- Use `Result<T, E>` + `anyhow`; use `anyhow::Context` to explain what operation
  failed. For consumer-facing classification use `but_error::Code` — don't make
  consumers match error strings.
- Acquire repository/worktree locks at top-level API/command boundaries. Don't
  call permission-acquiring helpers while holding a guard — drop the guard or use
  a `*_with_perm(...)` variant. Debug deadlocks with `BUT_WS_LOCK_DEBUG=1`.
- Preserve `DryRun` semantics; dry runs must not persist refs, objects, or oplog.
- Avoid implicit `SystemTime::now()` in testable business logic — pass time in.
- Format with `cargo fmt`. Run `pnpm rustfmt` (nightly import grouping) only when
  asked. Imports group Std / External / Crate at crate granularity.

### Rust tests

- Use `but-testsupport` for scenario creation; read-only behavior uses read-only
  fixtures. **Never** use `std::env::temp_dir().join(format!(…))`.
- For graph/rebase/workspace behavior, prefer fixture-backed before/after `insta`
  snapshots plus targeted structural assertions. Stabilize/normalize volatile
  output rather than weakening assertions.
- Explain why an assertion holds via its message arg (`assert!(…, "why")`,
  `insta::assert_debug_snapshot(x, "why", @r"")`). Use `insta` redactions for
  unstable output.
- CLI tests (`crates/but/tests/`): use `env.but(...).assert()` with snapbox
  `.stdout_eq`/`.stderr_eq` and `[..]`/`...` wildcards; update with
  `SNAPSHOTS=overwrite cargo test -p but`. Use sandbox helpers
  (`env.invoke_bash`/`env.invoke_git`), not `std::process::Command::new("git")`.
  CLI tests are expensive — happy-path only, test what really matters.

### TypeScript / Svelte / React

- **No relative imports** — use `@gitbutler/` package references (ESLint enforces).
- Prettier: tabs, double quotes, no trailing commas, 100-col. No `console.log`
  (use `console.warn`/`console.error`). Prefer top-level function declarations
  over arrow functions.
- Components: PascalCase (`BranchCard.svelte`). Files: kebab-case. Shared UI →
  `packages/ui`; shared utils → `packages/shared`.
- **Lite (React):** memoization (`useMemo`/`useCallback`/`React.memo`) is
  redundant — React Compiler handles it. Components follow
  `export const C: FC<Props> = (p) => {…}`. After lite work:
  `pnpm oxlint:fix && pnpm exec prettier --write apps/lite && pnpm knip:prod && pnpm knip:non-prod`.
- Add deps with `pnpm add <pkg> --filter @gitbutler/<pkg>`; workspace-level Rust
  deps go in root `Cargo.toml` `[workspace.dependencies]`. Check vulnerabilities
  before adding any dependency.

### Version control

- Assume the worktree may contain **other agents' / the user's** changes. Do not
  overwrite, clean up, stage, commit, or amend changes you did not make.
- When asked to branch/commit/push/open a PR, use the GitButler `but` CLI/workflow
  when available. On "ship it": commit on a session branch (create if needed),
  push, open/update the PR.
- Commit messages and PR descriptions: succinct — why, impact, core decisions. No
  local validation commands, no AI co-author trailers, no tool branding.

### Agent identity

- In governed repos, identity is resolved from `BUT_AGENT_HANDLE` set by the
  trusted harness wrapper (the git→but steerer), not self-asserted by the agent.
  The gates call `resolve_principal_from_env` against committed
  `.gitbutler/agents.toml`; an unset or unknown handle resolves no principal and
  is denied with `perm.denied`.
- The steerer assigns each agent's handle: OpenCode via a `shell.env` injection
  (host-set, un-forgeable); Claude Code / Codex via PreToolUse match-enforcement
  (deny a governed `but` whose handle ≠ the harness-assigned agent). There is no
  runtime registry.
- `but agent` exposes only `list --committed` and `migrate`. See
  `crates/but-authz/README.md` for the trust model and forgeability caveat.

## Scoped Instructions & Key Docs

- Rust work under `crates/` → `crates/AGENTS.md`
- `but` CLI work under `crates/but/` → `crates/but/AGENTS.md`
- Lite work under `apps/lite/` → `apps/lite/AGENTS.md`
- Graph/workspace/branch/stack/commit/rebase work → `crates/WORKSPACE_MODEL.md`
- Frontend testing detail → `frontend.md`
- Full setup, platform deps, "Code Hitlist" → `DEVELOPMENT.md`, `LINUX.md`
- Contribution guidelines → `CONTRIBUTING.md`

## Specialist Agents

When decomposing work for subagents, map tasks to these specialists by surface
and phase (planner → implementer → reviewer):

| Surface | Planner / Implementer / Reviewer |
|---|---|
| Rust backend & `but` CLI/TUI (`crates/`) | `rust-planner` / `rust-implementer` / `rust-reviewer` |
| Tauri desktop shell (`gitbutler-tauri`, capabilities/IPC) | `tauri-planner` / `tauri-implementer` / `tauri-reviewer` |
| SvelteKit frontends + shared Svelte UI (`apps/desktop` adapter-static, `apps/web` adapter-vercel, **`packages/ui`** — 185 `.svelte` components) | `sveltekit-planner` / `sveltekit-implementer` / `sveltekit-reviewer` |
| Electron lite shell (`apps/lite`) | `electron-planner` / `electron-implementer` / `electron-reviewer` |
| Distinctive UI design & implementation | `frontend-designer` |

Notes:

- **Rust and Tauri are the core surfaces** — the backend engine and desktop shell
  are the heart of the product; most backend/desktop tasks route through these.
- `sveltekit-*` agents carry some SpacetimeDB/adapter flavor, but their core
  SvelteKit competency (routes, stores, adapter-static compliance) fits
  `apps/desktop` and `apps/web` directly.
- **`packages/ui` is a Svelte component library** (185 `.svelte`, 0 React), consumed by
  the SvelteKit apps (`apps/desktop`/`apps/web`) — it is `sveltekit-*` / `frontend-designer`
  territory.
- For CI/CD workflow changes (`.github/`) consider `ghactions-{planner,implementer,reviewer}`;
  for cross-cutting quality/security passes consider `code-reviewer` and
  `security-reviewer`.
- If no agent is a clean fit, say so and use the closest generalist (`planner`,
  `code-reviewer`, `general-purpose`) — don't invent agent names.
