---
stability: CONSTITUTION
last_validated: 2026-06-18
prd_version: 1.3.0
section: technical-requirements
---
# UI Infrastructure — Governance Management Surface (MGMT)

The human management surface added in v1.1.0 — **a feature inside the existing `apps/desktop` (SvelteKit + Tauri) app, not a new app.** The design **extends existing patterns** (the project-settings sections and the `rules/` components); it introduces **no new design-system work** — every token, control, and feedback component already exists in `packages/ui` (Svelte) or `apps/desktop/src/components/shared`. Frontend specialists: **`sveltekit-*` + `tauri-*` + `frontend-designer`**.

> **CT-harness prerequisite (B14 / Y-NEW-9 — BLOCKER for the MGMT component-test surface).** The MGMT component tests specify `pnpm test:ct`, but that command today runs **only** `packages/ui`'s CT config — **`apps/desktop` has no CT config**, so the governance component tests literally cannot run yet. A desktop CT/Vitest config (T-MGMT-000) is a **hard prerequisite for all 38 component-test criteria** (every MGMT component test). It may surface that the harness needs more than a spec note (a small infrastructure task); that work is out of this documentation pass but is the gating prerequisite for the 38 criteria. See the Verification posture section.

## Placement & the governed-front-end wiring

- **Where:** a new **Project Settings page** `"Permissions & Governance"`, added to `apps/desktop/src/lib/settings/projectSettingsPages.ts` with `adminOnly: true`, rendered through `SettingsModalLayout` (`apps/desktop/src/components/settings/SettingsModalLayout.svelte`) and branched in `ProjectSettingsModalContent.svelte`. The sidebar already filters `pages.filter((p) => !p.adminOnly || isAdmin)` (`SettingsModalLayout.svelte:53`), so admin-gating is inherited — but the renderer's `adminOnly`/disabled-controls are **UX convenience**; enforcement is `but-authz` `administration:write` at the but-api command boundary (a renderer that bypassed its own guard still hits the server gate). In v1 the renderer `adminOnly` uses the cloud `User.role === 'admin'` flag (B18); the functional `administration:write` check is the enforcement boundary.
- **Internal nav:** four tabs via the existing `apps/desktop/src/components/shared/Tabs` (`Tabs`/`TabList`/`TabTrigger`/`TabContent`) — Principals · Groups · Branch Gates · Rules.
- **Wiring (the governed front-end):** Svelte components → generated `packages/but-sdk` → **Tauri command** → `but-api` function → `but-authz`. The frontend calls the **same `but-api` functions the `but perm`/`but group`/gate CLI calls** (the CLI verbs live in `crates/but/src/args/`, not `but-clap`) — never writing config directly. After the Rust API lands, `pnpm build:sdk && pnpm format` regenerates `packages/but-sdk/src/generated`. So every governance invariant (admin-gated, ref-pinned, pending-until-committed, no self-escalation) holds identically from the GUI.

## Routing & Views

**Routing decision: a STATE of the existing Settings surface — NOT a new route.** Per the route-vs-state discriminator, the governance surface adds a *section* to the project Settings modal (whose own composition does not change), so it is a state, not a new `[projectId]/…` route. The CLI/engine layers remain routing-N/A; the desktop app gains one settings-section state. **Implementation:** extend the `ProjectSettingsPageId` type union (`projectSettingsPages.ts`) with the governance page id and branch `ProjectSettingsModalContent.svelte` to render `GovernanceSettings.svelte` when that page is active; all pending-state tracking is CLIENT-ONLY (Svelte stores, no `+page.server.ts`), consistent with `apps/desktop` adapter-static.

| Route / surface | Kind | States added | Primary UCs | Enter when |
|---|---|---|---|---|
| Project Settings modal | overlay (existing) | **+ `Permissions & Governance` section** (admin-only), with Principals · Groups · Branch Gates · Rules sub-tabs | UC-MGMT-01..07 | admin opens project settings |

**Route Delta (v1.1.0):** | Project Settings modal | CHANGED | +1 admin-only section (`Permissions & Governance`) with 4 sub-tabs | a state of the existing settings surface, not a seam → no new route |

## Wireframes

### Principals list (section entry)
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Project settings        Permissions & Governance            (adminOnly page) │
│ ┌──────────────────┐  ⚠ Changes take effect once committed to the           │
│ │ Project          │    governance ref. Pending edits show ○.   [Commit →]  │
│ │ AI options       │  [Principals] [Groups] [Branch Gates] [Rules]           │
│ │ Experimental     │  ─────────────────────────────────────────────────────  │
│ ●─Permissions &    │  Principals                                   [+ Add]   │
│ │  Governance      │  ┌──────────────────────────────────────────────────┐  │
│ │                  │  │ [●] claude-agent  admin   eng   contents:rw·merge │  │
│ │                  │  │     + administration:write (own grant)            │  │
│ │                  │  │ [●] codex-agent   write   eng   contents:rw (grp) │  │
│ │                  │  │ [○] cursor-bot    read    —     contents:read ○pend│ │
│ └──────────────────┘  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
Legend: [●] committed   [○] pending (not yet committed to ref)   ··· = KebabButton
```

### Per-principal permission editor (inline, mirrors RuleEditor slide-in)
```
┌──────────────────────────────────────────────────────────────────┐
│ Principal: agent:codex-staging                          [✕ Close] │
│ ROLE PRESET (sugar)  [read] [triage] [●write] [maintain] [admin] │
│   Applies: contents:read·write · pull_requests:write · reviews   │
│ FUNCTIONAL PERMISSIONS          SOURCE          GRANT             │
│  contents:write                 [group: eng]    ── inherited ──   │
│  pull_requests:write            own grant       [●] ON            │
│  reviews:write                  own grant       [●] ON            │
│  merge                          —               [○] OFF           │
│  administration:write           —               [○] OFF           │
│  ℹ Inherited (grey) rows come from groups; remove a group to revoke│
│ GROUPS  [eng ✕] [platform ✕]            [+ Add to group ▾]       │
│                              [Cancel] [Save changes ○ pending]    │
└──────────────────────────────────────────────────────────────────┘
```
> **Save model (B16 / Y-NEW-11 — batch-save).** The editor uses a **batch-save** model, consistent with the `[Save changes ○ pending]` control and the pending-until-committed paradigm: individual `Toggle`s and group-chip changes update **local UI state only**; `[Save changes]` writes the staged set together to the working-tree `permissions.toml`. The batch is implemented as a **sequence of the already-specified `but perm grant`/`revoke` (and `but group add-member`/`remove-member`) verbs** — additive, no new governed verb. A proposed `but perm set --principal … --permissions …` (overwrite) verb is **deferred (C2)** — there is no per-toggle write and no new overwrite verb in the POC.

### Groups tab
```
┌──────────────────────────────────────────────────────────────────┐
│ Groups                                               [+ New group]│
│ ▼ eng                                          3 members  [···]  │
│   GRANTED  [●]contents:read [●]contents:write [○]merge [○]admin   │
│   MEMBERS  [●]claude-agent [●]codex-agent  [agent:new ✕]          │
│   [Delete group]  ⚠ confirm: "Remove group eng? N principals…"   │
│ ▶ platform                                     1 member   [···]  │
│ ── empty: EmptyStatePlaceholder "No groups yet…" [+ Create group] │
└──────────────────────────────────────────────────────────────────┘
Delete (B11) maps to `but group delete`; the confirmation dialog (B17) precedes the staged write.
```

### Branch Gates tab
```
┌──────────────────────────────────────────────────────────────────┐
│ Branch Gates             reads .gitbutler/gates.toml   [+ Add]   │
│ ▼ main                                              ○ pending    │
│   Protected branch                          [●] Toggle ON        │
│     ⚠ turning OFF confirms: "Unprotect branch main? Merges will  │
│       no longer require review."                                 │
│   Min. approvals required   [ 2 ]  (Textbox number)              │
│   Require distinct approver from author     [●] Toggle ON        │
│   Require approval from groups   [eng ✕] [security ✕] [+ ▾]      │
│ ▶ develop                                           ● committed  │
│ ── empty: EmptyStatePlaceholder "No branch gates configured."    │
└──────────────────────────────────────────────────────────────────┘
```

### Rules tab (reuses existing RulesList, scoped by principalId)
```
┌──────────────────────────────────────────────────────────────────┐
│ Rules                  Automate per-agent workspace behavior     │
│ ┌──────────────────┐ ┌──────────────────────────────────────┐   │
│ │ ● claude-agent   │ │ Rules for agent:codex-staging         │   │
│ │ ● codex-agent  ← │ │  (existing RulesList + Drawer + Rule  │   │
│ │ ○ cursor-bot     │ │   + RuleEditor, unchanged; new        │   │
│ └──────────────────┘ │   principalId prop scopes the query)  │   │
│                      └──────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────┘
```

### Cross-cutting states
```
⚠  4 pending governance changes — take effect once committed.  [Commit →]   InfoMessage warning
ℹ  Read-only: administration:write is required to change governance.         InfoMessage info
✕  perm.denied — you cannot modify your own administration grants.           InfoMessage danger
   (empty) EmptyStatePlaceholder "No principals configured" [+ Add first]
```
> **Commit semantics (B15 / Y-NEW-10).** "Commit changes" commits working-tree `.gitbutler/{permissions,gates}.toml` to the current workspace branch (the branch checked out in the desktop session) with message `chore: update governance config`. If `.gitbutler/*.toml` is clean (no diff vs HEAD), the banner is hidden. Staging is implicit (all `.gitbutler/*.toml` changes commit together). The commit itself goes through the commit gate.

## Component reuse (verified against source — cite in tasks for efficient build)

| UI element | Existing component | Path | Reuse |
|---|---|---|---|
| Settings sidebar + page scaffold | `SettingsModalLayout` | `apps/desktop/src/components/settings/SettingsModalLayout.svelte` | as-is (+1 page in `projectSettingsPages.ts`) |
| Settings page host | `ProjectSettingsModalContent` | `apps/desktop/src/components/settings/ProjectSettingsModalContent.svelte` | extend (add `governance` branch) |
| Section header/description | `SettingsSection` | `apps/desktop/src/components/shared/SettingsSection.svelte` | as-is |
| Secondary tabs | `Tabs`/`TabList`/`TabTrigger`/`TabContent` | `apps/desktop/src/components/shared/Tabs.svelte` (+TabList/Trigger/Content) | as-is |
| Card rows | `CardGroup` | `packages/ui/src/lib/components/cardGroup/CardGroupRoot.svelte` + `CardGroupItem.svelte` | as-is |
| Permission toggles | `Toggle` | `packages/ui/src/lib/components/Toggle.svelte` | as-is (`disabled` for inherited/read-only) |
| Role preset strip | `SegmentControl`/`Segment` | `packages/ui/src/lib/components/segmentControl/SegmentControl.svelte` | as-is |
| Group/member/required-group chips | `TagInput` | `packages/ui/src/lib/components/TagInput.svelte` | as-is (`readonly` in read-only state) |
| Expandable group / gate rows | `ExpandableSection` | `apps/desktop/src/components/shared/ExpandableSection.svelte` | as-is |
| Badges (role/status/pending) | `Badge` | `packages/ui/src/lib/components/Badge.svelte` | as-is (`warning`/`soft` = pending) |
| Pending/read-only/denial banners | `InfoMessage` | `packages/ui/src/lib/components/InfoMessage.svelte` | as-is (warning/info/danger) |
| Destructive-action confirmation | `Modal`/confirmation dialog | `packages/ui/src/lib/components/Modal.svelte` | as-is (group delete / unprotect-branch confirmations, B17) |
| Empty states | `EmptyStatePlaceholder` | `packages/ui/src/lib/components/EmptyStatePlaceholder.svelte` | as-is |
| Buttons (add/create/commit) | `Button` | `packages/ui/src/lib/components/Button.svelte` | as-is |
| Row overflow actions | `KebabButton` (+ ContextMenu) | `packages/ui/src/lib/components/KebabButton.svelte` | as-is |
| Min-approvals field | `Textbox` | `packages/ui/src/lib/components/Textbox.svelte` | as-is (`type=number`) |
| Group selector dropdown | `Select`/`SelectItem` | `packages/ui/src/lib/components/select/Select.svelte` | as-is |
| Toasts (staged/error) | `chipToasts` | `packages/ui/src/lib/components/chipToast/chipToastStore.ts` | as-is |
| Scroll container | `AppScrollableContainer` | `apps/desktop/src/components/shared/AppScrollableContainer.svelte` | as-is |
| Async/loading | `ReduxResult` (+ `SkeletonBone`) | `apps/desktop/src/components/shared/ReduxResult.svelte` | as-is |
| **Per-principal rules** | `RulesList` (+ `Rule`/`RuleEditor`/`RuleFiltersEditor`/`NewRuleMenu`) | `apps/desktop/src/components/rules/RulesList.svelte` | **extend: add optional `principalId` prop** (sole rules change; today takes only `projectId`) |

## Net-new components (thin compositions of the above)

| New component | Purpose | Location |
|---|---|---|
| `GovernanceSettings.svelte` | Top-level page: CLIENT-ONLY pending-state store (Svelte store, no `+page.server.ts`), `administration:write` check, tab layout, commit action (commit semantics per B15) | `apps/desktop/src/components/settings/GovernanceSettings.svelte` |
| `GovernanceErrorBoundary.svelte` | Error boundary wrapping the governance surface to catch render/runtime failures and show a fallback | `apps/desktop/src/components/governance/GovernanceErrorBoundary.svelte` |
| `PrincipalsList.svelte` | Principals tab: principal rows + inline editor toggle | `apps/desktop/src/components/governance/PrincipalsList.svelte` |
| `PrincipalEditor.svelte` | Inline editor: preset `SegmentControl` + functional `Toggle` table + group `TagInput`; **batch-save** (B16) | `apps/desktop/src/components/governance/PrincipalEditor.svelte` |
| `GroupsList.svelte` | Groups tab: `ExpandableSection` per group; group delete + confirmation (B11/B17) | `apps/desktop/src/components/governance/GroupsList.svelte` |
| `BranchGatesList.svelte` | Branch Gates tab: `ExpandableSection` per branch; unprotect confirmation (B17) | `apps/desktop/src/components/governance/BranchGatesList.svelte` |
| `GovernancePendingBanner.svelte` | Warning `InfoMessage` tracking `pendingCount` + commit | `apps/desktop/src/components/governance/GovernancePendingBanner.svelte` |

Plus: **+1 entry** in `apps/desktop/src/lib/settings/projectSettingsPages.ts`, **+1 prop** (`principalId`) on `RulesList`, the **`but-sdk` regeneration** for the new `but perm`/`but group`/gate Tauri commands, and the **desktop CT/Vitest config** (B14 / T-MGMT-000) the component tests run against.

## Specialist ownership (frontend)

| Work | Owner |
|---|---|
| SvelteKit governance components + reuse wiring | `sveltekit-implementer` → `sveltekit-reviewer` |
| Tauri command surface + `but-sdk` regeneration (expose `but-authz` to the frontend) | `tauri-implementer` → `tauri-reviewer` |
| Desktop CT/Vitest harness scaffold (B14 / T-MGMT-000) — the prerequisite for the 38 component tests | `sveltekit-implementer` (with `tauri-*` for IPC mocks) |
| UX/layout/wireframe fidelity | `frontend-designer` |
| The backend `but-authz` / `but perm`/`but group`/gate functions these call | `rust-*` (see `02-system-components.md`) |

## Verification posture
Playwright component tests (`pnpm test:ct`) over the real `packages/ui`/desktop components (no mocked UI), asserting: admin-gated sidebar visibility, the per-principal batch-save → governed SDK call(s), pending-banner appearance on edit, read-only state under missing `administration:write`, and a denied write surfacing the structured error without applying. **Prerequisite (B14 / T-MGMT-000):** an `apps/desktop` CT/Vitest config must exist (`pnpm test:ct:desktop` or `pnpm test --filter @gitbutler/desktop`) running governance component tests against a `but-sdk` mock layer — `pnpm test:ct` today runs only `packages/ui`, so without the desktop CT config none of the 38 MGMT/desktop component tests can run. E2E (WebdriverIO/Playwright) covers the full admin flow open→edit→commit.
