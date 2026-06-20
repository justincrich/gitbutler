---
stability: FEATURE_SPEC
last_validated: 2026-06-18
prd_version: 1.3.0
functional_group: MGMT
---
# Use Cases: Governance Management UI (MGMT)

A human surface **inside GitButler's existing SvelteKit + Tauri desktop app** (`apps/desktop`) — a new *feature*, **not a new app** — where an admin manages, **per agent (principal)**, the governance config the rest of this PRD enforces: functional permissions, group membership, branch gates, and per-agent automation rules. It **extends existing patterns** rather than rearchitecting — a new admin-only **Project Settings page** (mirroring the existing `*Settings.svelte` sections), four tabs of governance config, and a **reuse of the existing `rules/` components** for per-agent rules.

> **The UI is a governed front-end, never a bypass.** Every write goes through the same `but-authz` governed path as the `but perm`/`but group`/gate CLI — exposed via `but-api` → Tauri command → the generated `packages/but-sdk`. So the same invariants hold from the GUI: `administration:write`-gated, ref-pinned, committed-config (a change is **pending until committed** to the governance ref), and self-escalation is impossible. The GUI is a nicer `but perm`, not a new authority path. Frontend disciplines for this group are **sveltekit + tauri + design** (the desktop UI is SvelteKit).

> **CT-harness prerequisite (B14 / Y-NEW-9 — BLOCKER).** Every `[component-test]` criterion in this group specifies `pnpm test:ct`, but that command today runs **only** `packages/ui`'s CT config — `apps/desktop` has **no CT config**, so these tests literally cannot run yet. A desktop CT/Vitest config (T-MGMT-000) is a **hard prerequisite for all 38 component-test criteria** (incl. every MGMT component test below). See UC-MGMT-01 AC and [`10-technical-requirements/10-ui-infrastructure.md`](./10-technical-requirements/10-ui-infrastructure.md).

| ID | Title | Description |
|----|-------|-------------|
| UC-MGMT-01 | Admin-gated "Permissions & Governance" settings surface | A new admin-only page in Project Settings (`projectSettingsPages.ts` `adminOnly:true`, rendered via `SettingsModalLayout`), with four tabs (Principals · Groups · Branch Gates · Rules); hidden from non-admins; **no new top-level route** (a state of the settings modal). Includes the first-admin bootstrap framing (fail-closed on missing config; `but governance init` is a named fast-follow) and the desktop-CT-harness prerequisite. |
| UC-MGMT-02 | View principals & edit a principal's permissions | The Principals tab lists each agent/user + effective `AuthoritySet` (own ∪ group, source-of-grant); the per-principal editor (role-preset `SegmentControl` + functional `Toggle`s, inherited rows read-only; group chips) **batch-saves** edits via `[Save changes]`, mapping to `but perm grant/revoke` + `but group add/remove-member`. Granting a permission to a not-yet-existing principal registers it (register-on-first-grant). |
| UC-MGMT-03 | Manage groups & membership | The Groups tab (an `ExpandableSection` per group: grant toggles + member `TagInput`) maps to `but group create/grant/add-member/remove-member/delete`. Group delete and destructive-action confirmations are included. |
| UC-MGMT-04 | Edit branch gates | The Branch Gates tab (per-branch protected/min_approvals/distinct/required-groups) edits `.gitbutler/gates.toml` via the gate-config write; unprotecting a branch shows a confirmation dialog. |
| UC-MGMT-05 | Per-agent automation rules (reuse existing rules UI) | The Rules tab reuses the existing `RulesList` scoped by a **new optional `principalId` prop** — the sole change to the `rules/` components; `Rule`/`RuleEditor`/`RuleFiltersEditor`/`NewRuleMenu` render unchanged. |
| UC-MGMT-06 | Governed front-end — pending-until-committed, read-only, denial, SDK/IPC wiring | Every write goes through `but-authz` via `but-api`→Tauri→`but-sdk`; staged changes are **pending until committed** (○ + commit banner, with defined commit semantics); read-only when lacking `administration:write`; self-escalation is not optimistically applied; structured denials are surfaced. |
| UC-MGMT-07 | Error handling & accessibility | An error boundary wraps `GovernanceSettings.svelte`; the four-tab navigation has aria-labels + keyboard nav; IPC failures surface the structured denial with a retry action, consistent with existing GitButler error-handling patterns. |

Wireframes for every view/state and the full component-reuse + net-new-component lists live in [`10-technical-requirements/10-ui-infrastructure.md`](./10-technical-requirements/10-ui-infrastructure.md). Two key wireframes are embedded below for context.

---

## UC-MGMT-01: Admin-gated "Permissions & Governance" settings surface
Governance config is per-repo administrative configuration, so it belongs where GitButler already models per-project config: **Project Settings**, not a new route. The page is added to `apps/desktop/src/lib/settings/projectSettingsPages.ts` with `adminOnly: true`; `SettingsModalLayout` already filters the sidebar by `!p.adminOnly || isAdmin` (`SettingsModalLayout.svelte:53`), so non-admins never see or reach it — admin-gating is inherited, not rebuilt. The page subdivides into four tabs via the existing `shared/Tabs` components. An admin opens it through the existing project-settings shortcut; **no new top-level `[projectId]/…` route is introduced** (the route-vs-state discriminator classifies this as a *state* of the settings surface).

**Bootstrap & infrastructure framing.** Two prerequisites bound this surface. (1) *First-admin bootstrap (B10):* on a project with no `.gitbutler/permissions.toml`, governance **fails closed** (UC-AUTHZ-04) — the bootstrap path is the trusted human fleet-owner (R12) creating the config file directly (they own the repo), so the chicken-and-egg dissolves; a `but governance init` convenience verb is a **named fast-follow**, not built in the POC. (2) *Desktop CT harness (B14):* the MGMT component tests need an `apps/desktop` CT config that does not exist today — a build-gate prerequisite for the whole MGMT component-test surface.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Project settings                                                             │
│ ┌──────────────────┐ ┌───────────────────────────────────────────────────┐  │
│ │ Project          │ │ Permissions & Governance        (adminOnly page)  │  │
│ │ Git stuff        │ │ ⚠ Changes take effect once committed to the       │  │
│ │ AI options       │ │   governance ref. Pending edits show ○. [Commit→] │  │
│ │ Experimental     │ │ [Principals] [Groups] [Branch Gates] [Rules]       │  │
│ ●─Permissions &    │ │ ───────────────────────────────────────────────── │  │
│ │  Governance      │ │ Principals                               [+ Add]   │  │
│ │                  │ │ [●] claude-agent  admin   eng    contents:rw·merge │  │
│ │                  │ │ [●] codex-agent   write   eng    contents:rw (grp) │  │
│ │                  │ │ [○] cursor-bot    read    —      contents:read ○pend│  │
│ └──────────────────┘ └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Acceptance Criteria
☐ System adds a "Permissions & Governance" page to project settings (`apps/desktop/src/lib/settings/projectSettingsPages.ts`, `adminOnly: true`) rendered through the existing `SettingsModalLayout`
☐ The governance page is hidden from the settings sidebar when the viewer is not an admin (the existing `pages.filter((p) => !p.adminOnly || isAdmin)` mechanism), so a non-admin cannot navigate to it — the renderer-side adminOnly filter is **UX convenience**; enforcement is `but-authz` `administration:write` at the but-api command boundary, so even a renderer that bypassed its own guard still hits the server gate. **In v1, MGMT UI admin-gating uses the cloud `User.role === 'admin'` (sidebar visibility) combined with the implicit human-fleet-owner trust (R12)**; the `administration:write` functional permission gates the CLI/server writes, **not** UI visibility — the renderer `adminOnly` filter is UX convenience, the backend `administration:write` check is the enforcement boundary (B18). Post-v1: a per-project `administration:write` check replaces the global cloud role for governance-specific gating (C4)
☐ A User opens the surface via the existing project-settings shortcut (`ProjectSettingsShortcutHandler`), and **no new top-level route** is added — it is a state of the settings modal
☐ The surface presents four tabs — Principals, Groups, Branch Gates, Rules — using the existing `shared/Tabs`/`TabList`/`TabTrigger`/`TabContent` components
☐ System has a passing Playwright component test (`pnpm test:ct`) asserting the governance page renders for an admin and is absent from the sidebar for a non-admin
☐ The `ProjectSettingsPageId` type union in `apps/desktop/src/lib/settings/projectSettingsPages.ts` is extended with the governance page id, and `ProjectSettingsModalContent.svelte` gains the branch rendering `GovernanceSettings.svelte` when that page is active (no new top-level route)
☐ After the Rust API lands, running `pnpm build:sdk && pnpm format` regenerates `packages/but-sdk/src/generated` with the new governance commands/types, and the MGMT components type-check against them before UI wiring
☐ On a project with no `.gitbutler/permissions.toml`, governance **fails closed** (UC-AUTHZ-04); the bootstrap path is the trusted human fleet-owner (R12) creating the file directly. A `but governance init` convenience verb that writes a default config with the fleet-owner as admin is **named as a fast-follow** (C1), not built in the POC (correctness — fail-closed on empty/missing config — is already covered by UC-AUTHZ-04 / T-AUTHZ-029)
☐ An `apps/desktop` Playwright CT / Vitest config exists and the `pnpm test:ct:desktop` (or `pnpm test --filter @gitbutler/desktop`) command runs governance component tests against a `but-sdk` mock layer — a **hard prerequisite for all 38 MGMT/desktop component-test criteria** (B14 / T-MGMT-000); without it the component tests below cannot run

---

## UC-MGMT-02: View principals & edit a principal's permissions
The Principals tab is the heart of the surface: each registered principal with its **effective `AuthoritySet`** (own grants ∪ group grants), labelled by source-of-grant so an admin can see *why* a principal holds a permission. Opening a principal reveals an inline editor (mirroring `RuleEditor`'s slide-in pattern) with a role-preset `SegmentControl` and a functional-permission list of `Toggle`s; group-inherited permissions render **read-only** (disabled) while own grants are editable, preserving the union semantics (picking a preset sets own-grant toggles but never strips an inherited grant). Group membership is shown as removable chips (`TagInput`). Edits are **staged in the editor and batch-saved together** via `[Save changes]` (consistent with the pending-until-committed paradigm) — individual toggles update local UI state only; the batch is implemented as a sequence of the already-specified `but perm grant`/`revoke` + `but group add/remove-member` verbs (no new overwrite verb).

```
┌──────────────────────────────────────────────────────────────────┐
│ Principal: agent:codex-staging                          [✕ Close] │
│ ROLE PRESET  [read] [triage] [●write] [maintain] [admin]         │
│ FUNCTIONAL PERMISSIONS          SOURCE          GRANT             │
│  contents:write                 [group: eng]    ── inherited ──   │
│  pull_requests:write            own grant       [●] ON            │
│  reviews:write                  own grant       [●] ON            │
│  merge                          —               [○] OFF           │
│  administration:write           —               [○] OFF           │
│ GROUPS  [eng ✕] [platform ✕]            [+ Add to group ▾]       │
│                              [Cancel] [Save changes ○ pending]    │
└──────────────────────────────────────────────────────────────────┘
```

### Acceptance Criteria
☐ The Principals tab lists each registered principal with its effective `AuthoritySet`, distinguishing own grants from group-inherited grants (source-of-grant shown per permission)
☐ An admin can open a per-principal editor (inline, mirroring `RuleEditor`'s slide-in) showing a role-preset `SegmentControl` (read/triage/write/maintain/admin) and a functional-permission list of `Toggle`s; **individual toggles update local UI state only** (they do not write per-toggle)
☐ The editor renders group-inherited permissions as read-only (disabled), only own grants editable; selecting a role preset sets own-grant toggles to the desugared set without removing any inherited grant (union semantics preserved)
☐ Edits to a principal's permissions are **staged in the editor**; `[Save changes]` writes them together to the working-tree `permissions.toml` — implemented as a sequence of `but perm grant`/`but perm revoke` (and `but group add-member`/`remove-member` for chip changes) via the `but-sdk`; there is no per-toggle write and no new overwrite verb (`but perm set` is deferred — C2)
☐ The effective-permission display reflects a saved change only once it is committed to the governance ref (pending until then, per UC-MGMT-06)
☐ Granting a permission to a principal that does not yet exist in `permissions.toml` implicitly creates the principal entry (**register-on-first-grant**); a principal with no grants and no group memberships is effectively decommissioned — denied all actions by fail-closed (UC-AUTHZ-04)
☐ System has a passing Playwright component test asserting a `[Save changes]` batch issues the correct `but perm` SDK call(s) and an inherited row is non-interactive

---

## UC-MGMT-03: Manage groups & membership
The Groups tab lets an admin define the teams that the union semantics (GRPS) and the gate config (GATES `require_approval_from_group`) reference. Each group is an `ExpandableSection` showing its granted permission set (toggles) and its members (`TagInput`). Create / grant / add-member / remove-member / **delete** map to `but group …`. An empty state invites creating the first group. Destructive actions (group delete, removing the last member of a required group) surface a confirmation.

### Acceptance Criteria
☐ The Groups tab lists each group as an `ExpandableSection` showing its granted permission set and its members
☐ An admin can create a group, grant/revoke its permissions, and add/remove members — mapping to `but group create`/`grant`/`add-member`/`remove-member` via the `but-sdk`
☐ The Groups tab shows an `EmptyStatePlaceholder` with a create-first-group action when no groups exist
☐ A group's grant/member change is reflected in affected principals' effective sets (UC-MGMT-02) once committed
☐ An admin can delete a group (`but group delete <name>`); principals lose that group's inherited grants on the next target-ref read (B11)
☐ Deleting a group shows a confirmation dialog ("Remove group X? N principals will lose inherited permissions."); removing the last member from a required group shows a warning banner (B17)
☐ System has a passing Playwright component test asserting group create + grant + add-member issue the correct `but group` SDK calls

---

## UC-MGMT-04: Edit branch gates
The Branch Gates tab surfaces `.gitbutler/gates.toml` per target branch: a protected `Toggle`, a `min_approvals` number `Textbox`, a `require_distinct_from_author` `Toggle`, and a required-groups selector (`TagInput`/`Select`) whose options are exactly the groups defined in the Groups tab. Edits write the governed `gates.toml` (so they are ref-pinned and pending-until-committed like every other governance change). Unprotecting a branch is destructive and is confirmed.

### Acceptance Criteria
☐ The Branch Gates tab lists each configured target branch with its gate fields (protected, `min_approvals`, `require_distinct_from_author`, `require_approval_from_group`)
☐ An admin can edit a branch's gate fields (`Toggle`, `Textbox type=number`, `Select`/`TagInput`), mapping to the gate-config write that edits `.gitbutler/gates.toml`
☐ An admin can add a gate for a new branch pattern; the tab shows an `EmptyStatePlaceholder` when none are configured
☐ The required-group selector offers only groups defined in the Groups tab (a consistent set), so a gate cannot require an undefined group
☐ Unprotecting a branch shows a confirmation dialog ("Unprotect branch main? Merges will no longer require review.") before the gate-config write is staged (B17)
☐ System has a passing Playwright component test asserting a gate edit issues the correct gate-config SDK call and surfaces the pending state

---

## UC-MGMT-05: Per-agent automation rules (reuse existing rules UI)
GitButler already ships a rules-management UI (`apps/desktop/src/components/rules/` — `RulesList`/`Rule`/`RuleEditor`/`RuleFiltersEditor`/`NewRuleMenu`) over `but-rules`. The Rules tab **reuses it wholesale**, scoped to a selected principal via a **single new optional `principalId` prop** on `RulesList` (which today takes only `projectId`). When `principalId` is set the `rulesService` query is scoped to that principal; when unset, the component behaves exactly as today (backward compatible). This is the minimal-new-code path — no rule component is rewritten.

### Acceptance Criteria
☐ The Rules tab reuses the existing `RulesList` component, scoped to a selected principal via a NEW optional `principalId` prop — the sole change to the `rules/` components
☐ The existing `Rule`, `RuleEditor`, `RuleFiltersEditor`, and `NewRuleMenu` components render unchanged in the per-principal rules context
☐ When `principalId` is set, the `rulesService` query is scoped to that principal's rules; when unset, behavior is identical to the existing workspace-rules surface (backward compatible)
☐ With no principal selected or no rules present, the tab shows the appropriate empty/placeholder state
☐ System has a passing Playwright component test asserting `RulesList` with a `principalId` scopes the rule list and without it renders the existing behavior unchanged

---

## UC-MGMT-06: Governed front-end — pending-until-committed, read-only, denial, SDK/IPC wiring
The cross-cutting invariant that keeps the GUI honest. The frontend never writes config directly: each edit calls a `but-api` function (the same one the CLI uses) exposed as a **Tauri command** and surfaced through the generated `packages/but-sdk` (regenerated with `pnpm build:sdk` after the Rust API changes). A write **stages** into the governance ref and is shown **pending** (a ○ indicator on the row + a warning `InfoMessage` banner with a "Commit changes" action) until committed — exactly GitButler's existing optimistic-local-then-commit convention. The surface is **read-only** when the viewer lacks `administration:write` (controls disabled, an info `InfoMessage` explains why), a layer beneath the app-level admin sidebar gating. A **self-escalation** attempt (granting oneself `administration:write`) is not optimistically applied — the governed path's structured denial is surfaced instead.

```
⚠  4 pending governance changes — take effect once committed.   [Commit changes →]   (InfoMessage warning)
ℹ  Read-only: administration:write is required to change governance.                (InfoMessage info)
✕  perm.denied — you cannot modify your own administration grants.                  (InfoMessage danger)
```

### Acceptance Criteria
☐ Every UI write goes through the `but-authz` governed path exposed via `but-api` → Tauri command → generated `packages/but-sdk`, never writing config directly — the UI is a front-end, not a bypass (the same function the CLI calls)
☐ A governance write goes through the governed but-api command (admin-gated via `administration:write`), which edits the working-tree `.gitbutler/*.toml`; the UI derives pending-state from the working-tree-vs-target-ref diff (GitButler's existing optimistic-local-then-commit convention) — there is NO renderer direct-file-write and NO optimistic enforcement application
☐ The pending-state store is CLIENT-ONLY (a Svelte store / GitButler's existing client state) with NO `+page.server.ts` server loads — consistent with `apps/desktop` adapter-static (no SSR)
☐ A staged change is shown pending (○ indicator + a warning `InfoMessage` "Commit changes" banner) until committed to the governance ref; on commit, the pending indicators clear and effective sets update
☐ The "Commit changes" action commits working-tree `.gitbutler/{permissions,gates}.toml` to the current workspace branch (the branch checked out in the desktop session) with message `chore: update governance config`; if `.gitbutler/*.toml` is clean (no diff vs HEAD), the banner is hidden; staging is implicit (all `.gitbutler/*.toml` changes commit together); the commit itself goes through the commit gate (B15)
☐ The surface is read-only when the viewer lacks `administration:write` (controls `disabled`, an info `InfoMessage` explaining why), independent of the app-level admin flag that hides the sidebar item
☐ The UI does not optimistically apply a self-escalation (an admin granting itself `administration:write`); it surfaces the structured denial returned by the governed path rather than flipping the control
☐ A structured governance denial (`{code, message, remediation_hint}`) is surfaced via a danger `InfoMessage`, and transient write errors via `chipToasts`, so the human sees why a write was refused
☐ System has a passing Playwright component test asserting (a) an edit shows the pending banner, (b) an `administration:write`-lacking viewer sees the read-only state, and (c) a denied write surfaces the structured error without applying the change
☐ For v1, a MGMT config-management write acts as the **human fleet-owner** — the desktop user (resolved from the signed-in `UserService` / forge session), a trusted superuser over the agent fleet (personal-tenant trust: the human at the keyboard owns the repo and its agents). This is the designed admin action, not a bypass — the "UI is never a bypass" invariant means an *agent* cannot circumvent its functional permissions via the UI; agent authorization is unchanged (agents stay bound by functional permissions, no superuser path). **The cloud `User.role === 'admin'` governs UI visibility while the backend `administration:write` check governs the actual write (B18)** — the renderer flag is UX convenience, the functional permission is the enforcement boundary. Accepted v1 risk (R12); future: a real per-human authenticated principal checked against `permissions.toml` (C4)

---

## UC-MGMT-07: Error handling & accessibility
The governance surface must fail gracefully and be accessible to keyboard and screen-reader users, matching GitButler's existing error-handling patterns. An **error boundary** wraps `GovernanceSettings.svelte` (its mount point) so a render/runtime failure in the governance components shows a fallback rather than breaking the whole settings modal. The four-tab navigation carries `aria-label`/`aria-labelledby` and full keyboard nav (Tab to focus, Enter/Space to activate, Arrow keys to move between tabs). When an IPC call to a Tauri command fails (timeout, backend crash, transport error), the UI surfaces the structured denial `{code, message, remediation_hint}` from `but-authz` (or a degraded "connection lost" notice if the structured response is unavailable) via a danger `InfoMessage` with a **Retry** action; on persistent failure the UI stays in a safe read-only state.

### Acceptance Criteria
☐ An error boundary wraps `GovernanceSettings.svelte` (or its mount point) so render/runtime failures display a fallback error message rather than breaking the entire settings modal
☐ The four-tab navigation (Principals/Groups/Branch Gates/Rules) has proper `aria-label`/`aria-labelledby` attributes and keyboard navigation works (Tab to focus, Enter/Space to activate, Arrow keys to move between tabs)
☐ When an IPC call fails (timeout, backend crash, transport error), the UI surfaces the structured denial `{code, message, remediation_hint}` returned by `but-authz` (or a degraded "connection lost" message) via a danger `InfoMessage` with a Retry button
☐ The retry action re-issues the same SDK call; on success the UI updates, and on persistent failure the error remains visible and the UI stays in a safe read-only state, with errors communicated via existing feedback components (`InfoMessage` for banner-level, `chipToasts` for transient)
☐ System has a passing Playwright component test asserting (a) the error boundary catches a thrown error and shows a fallback, (b) a keyboard user can navigate and activate tabs, and (c) an IPC failure surfaces a structured denial with a working retry
