---
stability: CONSTITUTION
last_validated: 2026-06-20
prd_version: 1.1.0
---
# 10 — Frontend UI (desktop)

v1.0 was CLI-first and deferred all UI. **v1.1 un-defers a focused desktop result-viewing surface** — the GitHub-"Checks"-style visualization the CLI's stored results make possible — grounded in a `frontend-designer` review of the real `apps/desktop/` forge UI and the `packages/ui` library. This section is the engineering contract for that surface: what exists, what's net-new, the route-vs-state decision, and the v1.1 / deferred tiering.

## §1 — What exists today (the starting point)
GitButler already fetches and parses forge CI checks, then **throws the per-check detail away**:

- **`CIChecksBadge`** (`apps/desktop/src/components/forge/CIChecksBadge.svelte`) renders **one aggregate pill** per branch/PR ("Checks passed/failed/running/…"). It is the *only* checks UI.
- **`ChecksMonitor.parseChecks()`** (`apps/desktop/src/lib/forge/checksMonitor.svelte.ts:45`) is passed as GitButler's custom call-site **`transform`** option — a **read-time selector** applied to `result.data`, **distinct from RTKQ's `transformResponse`** (see `customHooks.svelte.ts` / `butlerModule.ts`). It collapses the `CiCheck[]` to a single `ChecksStatus`. The `listCiChecks` query is `build.query<CiCheck[], …>`, so the **raw `CiCheck[]` IS cached**; but every current consumer goes through `ChecksMonitor` with `parseChecks`, so today nothing reads the per-check array — only the collapsed `ChecksStatus`.
- PR concepts are mature: `PullRequestCard` / `PrDetailsDrawer` / `PRListCard` / `ReviewBadge` (`packages/ui/src/lib/components/ReviewBadge.svelte`), with "Open checks" linking **externally** to `{pr.htmlUrl}/checks`.
- **No** per-check list, per-head panel, repo-wide matrix, "required vs optional" rendering, "stale at head" state, or any rendering of the new local `check_results`.

**A modest call-site read (not a freebie, not a refactor):** the panel reads the **same cached `listCiChecks` entry without the `parseChecks` transform** — a new `useQuery`/`useQueryState` call (or a `ChecksMonitor` API that returns the raw `CiCheck[]` alongside `ChecksStatus`). The raw array is already in the Redux cache and the forge fetch already happens, so there is **no new network round-trip** — it is a modest call-site addition, not a state-layer refactor. The new local-`check_results` producer feeds the same `CheckResultRow` component once the panel exists.

## §2 — Route-vs-state verdict (per `PRD-TECH-REQUIREMENTS.md` Part 2)
> **The all-checks visualization is a STATE of the existing `[projectId]/branches` route — NOT a new route.**

The branches page already hosts a `SegmentControl` with segments `All` / `PRs` / `Local` (real labels verified in `apps/desktop/src/components/branchesPage/BranchExplorer.svelte` `filterOptions` block, drawn from `BRANCH_FILTER_OPTIONS = ["all","pullRequest","local"]` in `apps/desktop/src/lib/branches/branchListing.ts`). Adding a **"Checks" segment** + a checks detail panel is a filter/content state of that view, not a product seam. A `[projectId]/checks/` route is warranted only if the surface must be reached from outside the branches context (e.g. a notification deep-link, or a redirect from a denied merge) — a **v1.2** concern. **v1.1 adds no new route.**

**What adding a "Checks" segment requires** (type-system surgery, not just a label): extend `BRANCH_FILTER_OPTIONS` (add `"checks"` to the const array), the `BranchFilterOption` union type and the `isBranchFilterOption` guard in `branchListing.ts`, the `filterOptions` derived block in `BranchExplorer.svelte`, and the `BranchesSelection` discriminated union in `BranchesView.svelte` (currently `{type:"branch"|"pr"|"target"}`). For the per-head Checks panel, the design adds a **new `{type:"checks"; headOid: string}` arm** to `BranchesSelection`; this is a distinct arm (not a reuse of the `branch` selection with a panel switch) so the right panel can unmistakably render `CheckResultsPanel` without conditional logic inside the branch arm.

## §3 — Net-new components (6; all in a new `apps/desktop/src/components/checks/`)
Colocated like `governance/` and `forge/`. Each justified by Rule-of-2 or unique semantics.

| Component | Props (sketch) | Used by | Justification |
|---|---|---|---|
| `CheckConclusionBadge.svelte` | `state: success\|failure\|timed_out\|missing\|stale\|running`, `size?: icon\|tag` | Surfaces 1, 2, 3 | Maps the full display-state vocabulary to icon+color+aria-label; distinct from `CIChecksBadge` (an aggregate, not a per-check state). Never color-only. The `state` prop unifies stored conclusions (`success`/`failure`/`timed_out` — stored in `check_results`), gate-derived display states (`missing` from `isMissing`, `stale` from `isStale` — computed, not stored), and a `running` state for an in-flight local run. `running` is the check-runner concept; it is distinct from the forge-SDK token `inProgress` (a `CiStatus` field, not a stored conclusion). |
| `CheckResultRow.svelte` | `name, state, trigger?, producer, durationSecs, capturedOutput?, headOid, isStale, isMissing, onRun?` | Surfaces 1, 2, 3 | One check row: badge + producer + duration + bound-to + expandable output + "Run now". `trigger` is a **definition-time** field from `.gitbutler/checks/*.toml`; it is NOT stored in `check_results` (which records `name, head_oid, conclusion, metadata`). The row resolves `trigger` via a **join to the check definition** at render time — `trigger?` is optional/undefined when the definition is not available. |
| `CheckResultsPanel.svelte` | `projectId, headOid, branchName, checks?` | Surfaces 1, 2 | The per-head list (summary strip + `CardGroupRoot` of `CheckResultRow`). |
| `RequiredChecksGateSummary.svelte` | `requiredChecks[], headOid, onMerge?` | Surface 3 | The merge-flow "N/M required satisfied" + miss-reasons + disabled-`[Merge]`. (Deferred-pending the governance merge dialog.) |
| `ChecksSettings.svelte` | `projectId` | Surface 4 | Settings-page host (Defined / Required-Sets tabs). (Deferred — needs DEFN CLI first; also requires confirming the governance settings wiring described below.) |
| `CheckEditor.svelte` | `check?, onSave, onCancel` | Surface 4 | Inline check add/edit form; mirrors the inline-editor pattern used in the governance surface (see `governance/DESIGN-ANNOTATIONS.md` for the `PrincipalEditor` plan — **planned, not yet implemented**). (Deferred.) |

> **Governance settings tab status (deferred Surface 4).** `projectSettingsPages.ts` has a `governance` entry (`{id:"governance", label:"Permissions & Governance", icon:"lock", adminOnly:true}`), but `ProjectSettingsModalContent.svelte` has **no branch for `"governance"`** — it falls through to the `else "Settings page X not Found"` path. `GovernanceSettings.svelte` itself is a stub (a heading `<h2>` only). The governance settings tab is an **in-flight scaffold whose wiring must be confirmed before Surface 4 extends it**; Surface 4 ("Checks" tab) adds `{id:"checks"}` to `projectSettingsPages.ts` and a branch in `ProjectSettingsModalContent.svelte`, following the same scaffold pattern, but can only be wired to a real settings modal once the governance wiring lands.

## §4 — Reused components (no net-new where one exists)
`Badge`, `Button`, `cardGroup/CardGroupRoot|Item`, `Codeblock`, `EmptyStatePlaceholder`, `InfoMessage`, `KebabButton`, `TagInput`, `Textbox`, `TimeAgo`, `Toggle` (all `packages/ui/src/lib/components/`); `SegmentControl` (`packages/ui/src/lib/components/segmentControl/SegmentControl.svelte`); `ExpandableSection`, `SettingsSection`, `SettingsModalLayout`, `ProjectSettingsModalContent` (all `apps/desktop/src/components/…`); `Tabs` / `TabList` / `TabTrigger` / `TabContent` (`apps/desktop/src/components/shared/Tabs.svelte`, `TabList.svelte`, `TabTrigger.svelte`, `TabContent.svelte`); `CIChecksBadge`, `MergeButton` (`apps/desktop/src/components/forge/`). **Package split:** `SegmentControl` is from `packages/ui` (shared library); `Tabs` and its sub-components are `apps/desktop`-local (not exported by `packages/ui`). Settings entry (deferred Surface 4) would extend `apps/desktop/src/lib/settings/projectSettingsPages.ts` (add `{ id: "checks", label: "Checks" }`) + a branch in `ProjectSettingsModalContent.svelte`.

## §5 — A `CheckConclusion` vocabulary, consistent with the forge `CiConclusion`
The SDK's `CiConclusion` (`actionRequired|cancelled|failure|neutral|skipped|success|timedOut|unknown`) and the check-runner stored conclusions (`success|failure|timed_out`) share most tokens. `CheckConclusionBadge` is the **single source of truth** for the display-`state` → icon/color/label mapping across all surfaces. The `state` prop models three categories:

- **Stored conclusions** (`success` / `failure` / `timed_out`): persisted in `check_results` rows.
- **Gate-derived display states** (`missing` / `stale`): derived from `isMissing` / `isStale` at the gate or the panel; not stored in `check_results`.
- **In-flight state** (`running`): an active local run in progress; not a stored conclusion and distinct from the forge-SDK `inProgress` (`CiStatus`) token.

Design tokens used: `--fill-safe-bg` / `--fill-safe-fg` (success), `--fill-danger-bg` / `--fill-danger-fg` (failure / timed_out), `--fill-warn-bg` / `--fill-warn-fg` (stale / missing), `--chip-gray-bg` / `--chip-gray-fg` (running) — all verified in `packages/ui/src/lib/components/Badge.svelte`. **No bare `--fill-safe`, `--fill-danger`, or `--fill-warn` tokens exist** — always use the `-bg`/`-fg` suffixed pair. **No new design tokens** are introduced.

## §6 — Scope tiering (which surfaces ship when)
| Surface | Tier | Rationale |
|---|---|---|
| **Per-head results panel** (Surface 2) + **branches "Checks" state** (Surface 1) | **v1.1 — un-deferred** | Modest: forge API fetch already happens, but delivering the panel requires a query-path change (§1) and type-system surgery for the new "Checks" segment (§2). Net-new: `CheckConclusionBadge` + `CheckResultRow` + `CheckResultsPanel` + a "Checks" segment. `CheckResultRow` / `CheckConclusionBadge` are produced here and consumed by Surface 3 — **Surface 3 has a hard sequential dependency on Surface 1 shipping first**. |
| **Required-checks gate summary** (Surface 3) | **v1.1 IFF the governance merge dialog exists** | `MergeButton` is a plain dropdown today; there is **no** merge dialog to compose into. The summary can render `code/message/remediation_hint/unmet` now; the `class/authorized_actions/do_not` enrichment is additive once STEER-001 lands (governance sprint-07 STEER-001). **Two sequential dependencies: (a) Surface 1 must ship first** (Surface 3 reuses `CheckResultRow` / `CheckConclusionBadge`); **(b) the governance merge dialog must exist** (otherwise there is no host to compose into). Blocked on governance's own (deferred) merge-gate UI, not on this design. |
| **Checks settings tab** (Surface 4) | **Deferred (v1.2)** | Needs the DEFN CLI shipped first (config to manage). Also requires governance settings tab wiring to be confirmed (§3): `ProjectSettingsModalContent.svelte` currently has no `governance` branch; Surface 4 follows the same scaffold pattern and cannot be wired until the modal itself is functional. |
| **Cross-branch checks matrix** (Surface 1 "all-repo table" variant) | **Deferred (v1.2)** | Needs a `but check results --all-branches` CLI capability not specified in v1. |
| **Lite (React/Electron) checks UI** | **Out of scope (all v1.x)** | `apps/lite/` has **no** PR/forge/checks UI at all, and `packages/ui` is Svelte-only → any lite checks UI is a full **port**, not a share. Flag, do not attempt under a checks-UI sprint. |

## §7 — Cross-references
- The surfaces' wireframes live inline in the use cases: per-head panel + branches state → [`../05-uc-run.md` UC-RUN-05]; gate summary → [`../06-uc-gate.md` UC-GATE-03]; settings tab → [`../04-uc-defn.md` UC-DEFN-01].
- Conclusion vocabulary → [`03-data-schema.md`](./03-data-schema.md). Denial/STEER fields the gate summary renders → [`04-api-design.md`](./04-api-design.md) §5 (STEER-001 dependency).
