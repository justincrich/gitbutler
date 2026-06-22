---
stability: CONSTITUTION
last_validated: 2026-06-19
prd_version: 1.0.1
section: technical-requirements
---

# Frontend & UI Requirements — Butler "Checks" (Actions)

Constitution-layer frontend/UI contract for the butler **Checks** surface. This file exists to make the UI design **explicit and design-ready** even though **v1 ships backend + CLI only** — it owns two things: (1) the **v1-present CLI output contract** (the dual-audience run/results/denial output that is genuinely in-scope), and (2) the **deferred desktop GUI plan** that composes with governance's MGMT settings surface ([`.spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md`](../../governance/10-technical-requirements/10-ui-infrastructure.md)).

> **v1 posture (load-bearing).** Per [01-scope.md](../01-scope.md) ("A new agent UI or a new app … out of scope") and the [folder README](./README.md) ("This slice adds NO new route and NO new app"), the **only v1 user-facing surface is the `but check` CLI** (`define`/`list`/`run`/`results`/`required`). Every GUI component below is **deferred** and is specified here so the management surface is design-ready when funded — it composes _inside_ governance's existing Project-Settings surface (a new **Checks** tab + a `[[required_check]]` field in the Branch Gates tab), never as a new app or route. The frontend specialists for the deferred GUI are **`sveltekit-*` + `tauri-*` + `frontend-designer`**.

> **Composes with governance, does not duplicate.** Checks is the _quality_ half; governance is the _process_ half. The GUI mirrors that composition: the merge dialog shows Process (permission + review) **and** Quality (required checks) sections; the settings surface adds a Checks tab **inside** the governance "Permissions & Governance" page. Every governance UI invariant (admin-gated, ref-pinned, pending-until-committed, governed front-end via `but-api`→Tauri→`but-sdk`, no self-escalation) holds identically for the Checks GUI — see governance's [`10-ui-infrastructure.md`](../../governance/10-technical-requirements/10-ui-infrastructure.md) § "the governed front-end wiring."

## Placement & the governed front-end wiring (deferred GUI)

- **Where:** a new **Checks** tab _inside_ the existing governance "Permissions & Governance" Project-Settings page (`apps/desktop/src/lib/settings/projectSettingsPages.ts` `adminOnly: true`, rendered through `apps/desktop/src/components/settings/SettingsModalLayout.svelte`, branched in `ProjectSettingsModalContent.svelte`). Plus a **REQUIRED CHECKS** field inside governance's Branch Gates tab (the authoritative `[[required_check]]` policy lives in `gates.toml` alongside the gate fields).
- **Internal nav:** the governance page's 4 tabs (Principals · Groups · Branch Gates · Rules) gain a 5th: **Checks**.
- **Wiring (the governed front-end):** Svelte components → generated `packages/but-sdk` → **Tauri command** → `but-api` function → the same `enforce_merge_gate` / `but check …` paths the CLI uses. After the Rust API lands, `pnpm build:sdk && pnpm format` regenerates `packages/but-sdk/src/generated`. The GUI never writes config directly; it is a nicer `but check`, not a new authority path.

## Routing & Views

**Routing decision: a STATE of the existing Settings surface — NOT a new route.** Same discriminator outcome as governance: the Checks surface adds a _tab_ to the governance settings section, so it is a state, not a new `[projectId]/…` route. The CLI/engine layers remain routing-N/A.

| Surface                                                      | Kind                     | v1?            | States added                                                  | Primary UCs                                 |
| ------------------------------------------------------------ | ------------------------ | -------------- | ------------------------------------------------------------- | ------------------------------------------- |
| `but check` CLI                                              | verb                     | **v1 present** | `define`/`list`/`run`/`results`/`required` (table + `--json`) | DEFN-01/02/03, EXEC-05, LEDG-04, GATE-01/03 |
| Project Settings → Permissions & Governance → **Checks** tab | settings state (desktop) | deferred       | +1 tab (check defs list + editor + results)                   | DEFN-01/02/03, EXEC-05                      |
| Branch Gates tab → **REQUIRED CHECKS** field                 | settings state (desktop) | deferred       | +1 field per branch (`[[required_check]]`)                    | DEFN-03, GATE-01/05                         |
| Merge dialog → **Quality (Actions)** section                 | dialog state (desktop)   | deferred       | +1 section (required-checks summary + denial)                 | GATE-01/02/03                               |

**Route Delta (when the deferred GUI lands):** | Project Settings modal | CHANGED | +1 tab (Checks) + 1 field (REQUIRED CHECKS) inside the governance section | a state of the existing settings surface → no new route |

## Component inventory delta (build checklist)

### Net-new atoms (`packages/ui/src/lib/components/` — shared, Svelte-only)

| Component                 | Purpose                                                                                                                                                                          | Composes from    | Props (key)                                                                                                                                             |
| ------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `CheckStatusBadge.svelte` | Render a GitHub-compatible check **conclusion** (or gate-derived miss-reason) as a colored badge. **Single source of truth** for the conclusion→color/icon mapping (UC-LEDG-04). | `Badge` + `Icon` | `conclusion: "success"\|"failure"\|"neutral"\|"cancelled"\|"timed_out"\|"skipped"\|"missing"\|"stale"\|"unverifiable"`; `kind?: "icon"\|"text"\|"both"` |
| `RequiredBadge.svelte`    | `required`/`optional` tag for a check row. (Promote from inline only if used ≥2 places — list row + editor — per Rule-of-2.)                                                     | `Badge`          | `required: boolean`                                                                                                                                     |

### Net-new molecules (`apps/desktop/src/components/checks/`)

| Component                         | Purpose                                                                                                          | Composes                                                                               |
| --------------------------------- | ---------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| `CheckDefinitionRow.svelte`       | One row in the Checks list (name + trigger + required + run-spec + overflow).                                    | `CardGroupItem` + `RequiredBadge` + `KebabButton`                                      |
| `CheckDefinitionEditor.svelte`    | Slide-in editor for a check def (trigger `SegmentControl` + run-spec + timeout + secrets); batch-save.           | `SegmentControl` + `Textbox` + `TagInput` + `Button` + `CheckStatusBadge` (test-run)   |
| `CheckResultRow.svelte`           | One result row (name + conclusion + producer + duration + signed + bound-to + expandable masked output).         | `CardGroupItem` + `CheckStatusBadge` + `Timestamp` + `ExpandableSection` + `Codeblock` |
| `RequiredChecksEditor.svelte`     | REQUIRED CHECKS chip row inside Branch Gates; chips ⊆ defined checks.                                            | `TagInput` + `Select`                                                                  |
| `RequiredCheckGateSummary.svelte` | "Required checks @ <head> · N/M satisfied" strip + badges + run/results affordances.                             | `CheckStatusBadge` + `Button`                                                          |
| `CheckDenialBanner.svelte`        | The gate denial banner (`gate.check_required` / `config.invalid`) + per-check miss-reasons + run/view/copy-JSON. | `InfoMessage` (danger) + `CheckStatusBadge` + `Button`                                 |

### Net-new organisms (`apps/desktop/src/components/checks/`)

| Component                    | Purpose                                                                                                                                                           |
| ---------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ChecksList.svelte`          | The Checks tab body (list + add + empty + read-only + pending).                                                                                                   |
| `CheckResultsPanel.svelte`   | Per-head results panel (summary strip + rows + empty + run affordance).                                                                                           |
| `ChecksSettingsTab.svelte`   | Top-level Checks tab host (mirrors governance's `GovernanceSettings.svelte`: client-only pending store, `administration:write` check, tab layout, commit action). |
| `ChecksErrorBoundary.svelte` | Error boundary wrapping the Checks surface (mirrors `GovernanceErrorBoundary.svelte`).                                                                            |

### UI mods to existing components

| Path                                                                      | Mod                                                                                                                                                                                                                                                | Reason                                                                                 |
| ------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| `apps/desktop/src/lib/settings/projectSettingsPages.ts`                   | If Checks is its own admin page (not a governance sub-tab): add `{ id: "checks", label: "Checks", icon: "checklist", adminOnly: true }`. (Preferred: a sub-tab of governance — then no change here.)                                               | Settings registry is the single source of settings pages.                              |
| `apps/desktop/src/components/settings/ProjectSettingsModalContent.svelte` | Branch to render `ChecksSettingsTab.svelte` (or the governance page renders the 5th tab).                                                                                                                                                          | Renderer dispatches page → component.                                                  |
| Governance `GovernanceSettings.svelte` (deferred)                         | Add a 5th `TabTrigger` ("Checks").                                                                                                                                                                                                                 | Checks composes _inside_ governance's settings.                                        |
| Governance `BranchGatesList.svelte` (deferred)                            | Add the REQUIRED CHECKS field row per branch (hosts `RequiredChecksEditor`).                                                                                                                                                                       | The required-set policy lives in `gates.toml` next to the gate fields.                 |
| Governance merge dialog (deferred, sprint-06b)                            | Add the Quality (Actions) section + `RequiredCheckGateSummary` / `CheckDenialBanner`.                                                                                                                                                              | Actions composes _inside_ governance's merge-gate UI (process + quality).              |
| `packages/ui/src/lib/components/InfoMessage.svelte`                       | Verify/extend the `error` block to render denial JSON with a "Copy as JSON" affordance + a per-check-rows snippet. **Verify against source first** — it already exposes `error` + copy + primary/secondary actions, so the mod may be wiring-only. | The denial is the dual-audience artifact (human banner + orchestrator JSON).           |
| `packages/ui/src/lib/components/Codeblock.svelte`                         | Verify masked/readonly rendering + long-output truncation ("show more"). **Verify against source first.**                                                                                                                                          | Captured output can be large; v1 retention is limited; masked secrets render as `***`. |

## Shared-library impact (`packages/ui`)

- **New exports:** `CheckStatusBadge` (and `RequiredBadge` if promoted) → add to `packages/ui/src/lib/index.ts` and ship via the existing `./Component.svelte` export map. `CheckStatusBadge` is the **one cross-surface atom** — it is reused by results, gate summary, and denial surfaces, so it belongs in the shared library (not `apps/desktop`).
- **package.json:** **no new dependencies.** All visuals reuse existing tokens + `Badge`/`Icon`. No icon additions needed (all icons exist: `tick-circle`, `cross-circle`, `clock`, `refresh`, `stop`, `lock-auth`, `checklist`, `play`, `warning`, `danger`, `info`). Do **not** add a dep casually — flag any proposal here for security review (per repo `AGENTS.md`).
- **Storybook:** add one story per conclusion/miss-reason variant for `CheckStatusBadge` (the mapping table in [UC-LEDG-04](../06-uc-ledg.md) is the story spec).

## Theming & tokens

- **No new design tokens.** The conclusion vocabulary maps to **existing** color tokens (verified against `packages/ui/src/lib/utils/colorTypes.ts`: `gray`/`pop`/`safe`/`danger`/`warning`/`purple`):
  - `success`→`safe`, `failure`/`unverifiable`/`config.invalid`→`danger`, `timed_out`/`stale`→`warning`, `missing`/`neutral`/`skipped`/`cancelled`→`gray`.
  - Fills: `--fill-safe-bg`, `--fill-danger-bg`, `--fill-warn-bg`, `--chip-gray-bg` (all pre-existing in `packages/ui/src/styles/`).
- **Color is never the only signal.** Every conclusion pairs its color with an icon (`tick-circle`/`cross-circle`/`clock`/`refresh`/`stop`/`lock-auth`) + a text label + `aria-label` — color-blind safe and screen-reader friendly.

## Cross-surface considerations

| Surface            | Stack                              | Checks UI?     | Notes                                                                                                                                                                                                                                                                                                                     |
| ------------------ | ---------------------------------- | -------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `but check` CLI    | Rust (`crates/but/`)               | **v1 present** | The only v1 user surface. Dual-audience: human table + `--json` for the orchestrator.                                                                                                                                                                                                                                     |
| Desktop            | SvelteKit + Tauri (`apps/desktop`) | deferred       | Consumes `packages/ui` directly. The primary GUI target (mirrors governance).                                                                                                                                                                                                                                             |
| Web                | SvelteKit (`apps/web`)             | deferred       | Consumes `packages/ui` directly — **no port**.                                                                                                                                                                                                                                                                            |
| Lite               | React 19 + Electron (`apps/lite`)  | not planned    | **`packages/ui` is Svelte-only** — any Checks GUI in lite requires a **React re-implementation** of `CheckStatusBadge` + the molecules. Flag explicitly: the check-vocabulary atom is small enough to port, but the molecules/organisms are Svelte-coupled. No lite GUI is in scope for v1 or the named deferred surface. |
| Orchestrator STEER | external (Claude Code/Codex)       | n/a            | GitButler **emits** the `steer` fields in the denial JSON; the orchestrator renders the redirect in **its own** UI (out of GitButler's UI scope).                                                                                                                                                                         |

## Accessibility requirements

- **WCAG 2.1 AA** (match governance's MGMT a11y contract). Color is never the sole signal (icon + label + `aria-label` for every state).
- **Keyboard navigation:** the Checks tab is a `TabTrigger` (arrow-key nav between tabs, per the existing `Tabs` contract); every row/affordance is reachable via Tab; `[▶ Run …]`, `[View output]`, `[Copy JSON]` are real buttons with visible focus rings (`Button` already implements `:focus-visible`).
- **Screen readers:** denial banner is `role="alert"`; each miss-reason/badge carries a descriptive `aria-label` (e.g. "lint: no result at head — run the check"); the JSON block is `aria-label="Denial JSON"`; live regions announce run start ("Running lint…").
- **Focus management:** the merge dialog keeps focus trapped while the denial banner is shown; Escape dismisses the banner (does not un-block the merge); the editor's slide-in restores focus to the triggering row on close.
- **Reduced motion:** denial/banner entrance animations respect `prefers-reduced-motion` (the `Modal`/`InfoMessage` transitions already use the repo's `--transition-*` tokens).

## Performance budgets

- **Bundle size:** `CheckStatusBadge` + the molecules are thin compositions of existing primitives — negligible bundle impact (no new dep). Verify with `pnpm knip` (no unused exports) after adding to `index.ts`.
- **Results list:** a head can have many checks, and history can be long — the results panel uses `VirtualList` (`packages/ui/src/lib/components/VirtualList.svelte`) for the history view; the current-head summary is bounded by the required-set size (small).
- **Captured output:** masked output can be large; render truncated with "show more" (`Codeblock`), and never load full logs eagerly (lazy on expand; v1 retention is limited — `01-scope.md`).
- **CLI:** table output respects terminal width; `--json` is the streaming contract for the orchestrator.

## Testing strategy

- **Unit (Vitest):** the conclusion→badge mapping in `CheckStatusBadge` (all 9 variants incl. the "unknown conclusion → gray fallback" for future-produced values).
- **Component (Playwright CT, `packages/ui`):** a `CheckStatusBadge` story/test per variant — color + icon + label + `aria-label`.
- **Desktop CT (`apps/desktop`):** **blocked on the desktop CT/Vitest harness prerequisite** governance names (B14 / T-MGMT-000 — `apps/desktop` has no CT config today; `pnpm test:ct` runs only `packages/ui`). This is a **hard prerequisite** for every desktop Checks component test — same blocker as governance's 38 MGMT criteria. Tests: (a) the Checks tab renders for an admin + is absent for a non-admin; (b) a check-definition edit shows pending + commits via `but check define`; (c) `RequiredChecksEditor` rejects an undefined check name; (d) the merge dialog's Quality section blocks + surfaces the denial + `[▶ Run]` re-evaluates; (e) read-only under missing `administration:write`.
- **Visual regression:** snapshot the denial banner across all miss-reasons + the all-green merge-enabled state.
- **E2E (WebdriverIO/Playwright):** the full admin loop open → define check → mark required → commit → run → view results → attempt merge → denial → run missing → merge proceeds. This is the human-testing-gate analog of governance's MGMT flow.
- **Real-services discipline:** component tests run against a `but-sdk` mock layer; E2E runs against the **real** executor + real git (no stubbed success — the cardinal line, [UC-EXEC-01](../05-uc-exec.md)).

## i18n / a11y / platform conventions

- **Strings:** all human-readable labels ("required", "optional", "no result @ head", "merge blocked — required checks not satisfied") are string literals today (the repo is not yet i18n'd); keep them as plain strings, not concatenated, so a future i18n pass can extract them.
- **Keyboard shortcuts:** none net-new for v1 (the CLI is the surface). The deferred GUI inherits governance's settings/merge-dialog shortcuts.
- **Platform-native menus (Tauri):** not in scope — the Checks surface is settings-modal + CLI, not a native menu.
- **Timestamps:** use `Timestamp`/`TimeAgo` (existing) for `recorded_at`; respect locale formatting already configured in the repo.

## Frontend risks

| Risk                                                                                          | Severity              | Mitigation                                                                                                                                                                                     |
| --------------------------------------------------------------------------------------------- | --------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **v1 ships no GUI, but the design assumes one** — stakeholders may expect a dashboard         | Medium                | This file is explicit: v1 = CLI only; the GUI is deferred and design-ready. The CLI output contract is the real v1 deliverable.                                                                |
| **`packages/ui` is Svelte-only; lite is React** — any Checks GUI in lite needs a port         | Medium                | `CheckStatusBadge` is small/portable; the molecules are Svelte-coupled. No lite GUI is in scope. Flag the port cost if lite is ever asked for it.                                              |
| **Desktop CT harness missing (B14 / T-MGMT-000)** — blocks all desktop Checks component tests | High (when GUI lands) | Same prerequisite as governance's 38 MGMT criteria; resolve once, both initiatives benefit.                                                                                                    |
| **Legacy `gitbutler-*` frontend debt near the merge/settings surfaces**                       | Low–Medium            | The settings + merge-dialog surfaces are modern SvelteKit; the legacy `gitbutler-*` TS is elsewhere. Avoid dragging legacy patterns into the new Checks components.                            |
| **Denial JSON must stay machine-stable** — orchestrators parse `--json`                       | High                  | The denial JSON shape (`{code, denied, unmet[], remediation_hint, steer}`) is a **contract**; version it and never rename fields without a migration. CI the `--json` output against a schema. |
| **`InfoMessage` may not need a mod** — claiming a code change risks churn                     | Low                   | "Verify against source first" on every UI mod above; prefer usage over modification.                                                                                                           |

## Specialist ownership (frontend, when the GUI is funded)

| Work                                                                                  | Owner                                                                 |
| ------------------------------------------------------------------------------------- | --------------------------------------------------------------------- |
| SvelteKit Checks components + reuse wiring                                            | `sveltekit-implementer` → `sveltekit-reviewer`                        |
| Tauri command surface + `but-sdk` regeneration (expose `but check …` to the frontend) | `tauri-implementer` → `tauri-reviewer`                                |
| Shared `CheckStatusBadge` atom + Storybook                                            | `frontend-designer` → `sveltekit-reviewer`                            |
| Desktop CT harness scaffold (B14)                                                     | `sveltekit-implementer` (with `tauri-*` for IPC mocks)                |
| The backend `but check …` / `enforce_merge_gate` these call                           | `rust-*` (see [`02-system-components.md`](./02-system-components.md)) |
