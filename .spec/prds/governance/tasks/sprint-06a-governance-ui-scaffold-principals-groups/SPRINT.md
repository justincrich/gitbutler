---
sprint: 06a
sequence: 7
timeline: Phase 4 — Governance management UI
status: In Progress
proposed_by: sveltekit-planner + tauri-planner + frontend-designer + rust-planner
milestone: sprint-06a
prd: ../../README.md
roadmap: ../../ROADMAP.md
generated_by: kb-sprint-tasks-plan
---

# Sprint 06a: Governance UI — Scaffold + Principals + Groups

**Sequence:** 7
**Timeline:** Phase 4 — Governance management UI
**Status:** In Progress
**Proposed by:** sveltekit-planner + tauri-planner + frontend-designer + rust-planner (MGMT backend); split authored by sveltekit-reviewer (red-hat)
**Milestone:** — (`sprint-06a`)

## Overview

The first **governance-management UI** sprint — and the first that surfaces governance to a human inside
the existing GitButler desktop app. Sprints 01a–05 built the enforcement core (the `but-authz` primitive,
the commit + merge gates, fail-closed identity confinement, ref-pinned grouping) and the **CLI write path**
(`but perm` / `but group`, Sprint 05) an admin uses to manage that config. Sprint 06a takes the _same_
`but-api` perm/group functions Sprint 05 authored and exposes them as **Tauri commands → generated
`packages/but-sdk` → SvelteKit components** — a new admin-only **Project Settings page**
(`"Permissions & Governance"`) with the **Principals** and **Groups** tabs, plus the cross-cutting
**pending-until-committed** banner. It is a _feature inside `apps/desktop`_, **not a new app and not a new
route** — a state of the existing settings modal.

> **The UI is a governed front-end, never a bypass (the load-bearing invariant).** Every write goes through
> the **same** `but-authz` governed path as the `but perm`/`but group` CLI — `but-api` function → Tauri
> command → `but-sdk`. So every governance invariant holds identically from the GUI: `administration:write`-
> gated **server-side**, ref-pinned, committed-config (a change is **pending until committed** to the
> governance ref), and self-escalation is impossible. The renderer `adminOnly` filter and disabled controls
> are **UX convenience only** — a renderer that bypassed its own guard still hits the server `but-authz`
> gate at the `but-api` command boundary. The GUI is a nicer `but perm`, not a new authority path.

The sprint is split across four disciplines, in a strict producer-before-consumer chain:

- **Rust `but-api` IPC seam (MGMT-IPC-001/002, `rust-implementer`):** wrap the Sprint-05 perm/group
  functions as `#[but_api]` governance commands (`perm_*`, `group_*`, `governance_status_read`) and fix the
  real `json::Error` transport drop — the `remediation_hint` (3rd field of the `Denial` contract) is not
  serialized today, so a denied governance write would reach the renderer without its remediation text.
- **Tauri command surface + SDK (MGMT-IPC-003/004/005, `tauri-implementer`):** register the governance
  commands in `generate_handler!` + the scoped capability file, wire the v1 human-fleet-owner identity
  (T-MGMT-042), **regenerate `packages/but-sdk`** (the hard predecessor of every UI task), and define the
  pending-until-committed read IPC contract (working-tree vs target-ref).
- **SvelteKit UI (MGMT-UI-001/002/003/005/006/007/008, `sveltekit-implementer`):** register the page +
  extend `ProjectSettingsPageId`, branch `ProjectSettingsModalContent` + **wire `isAdmin`**, build
  `GovernanceSettings.svelte` (client-only pending store, no `+page.server.ts`), the pending banner, and the
  Principals + Groups tabs (rows + inline editor + expandable groups) against the regenerated SDK.
- **Design (DESIGN-MGMT-001/002/003/005, `frontend-designer`):** wireframe-fidelity + visual-state
  annotations for the four tabs, the pending-state visual contract, the read-only state, and the
  inherited-vs-own permission distinction — all reusing existing `packages/ui` components (no new
  design-system work).

> **CT-harness prerequisite (B14 / T-MGMT-000 — carried as a hard gate).** Every MGMT `[component-test]`
> criterion specifies `pnpm test:ct`, which today runs **only** `packages/ui`'s CT config — `apps/desktop`
> has **no CT config**. A desktop CT/Vitest config is a hard prerequisite for the whole MGMT component-test
> surface; the first UI task (MGMT-UI-001) carries the scaffold obligation so the component tests below can
> actually run. See [`10-technical-requirements/10-ui-infrastructure.md`](../../10-technical-requirements/10-ui-infrastructure.md).

This sprint is **desktop UI** — its gate is verified by _using the Governance settings page_ (open Project
Settings, see the admin-gated item, navigate the four tabs, edit a principal grant + a group grant, watch
them stay pending until Commit). Every gate proof draws from
[`11-e2e-testing-criteria.md`](../../11-e2e-testing-criteria.md).

## Human Testing Gate

**Gate:** An admin opens Project Settings, sees the Permissions & Governance sidebar item, navigates to it,
observes four tabs, edits a principal's own-grant permission on the Principals tab, expands a group and
grants a permission on the Groups tab, and sees both changes remain pending with a commit banner until
clicking Commit changes.

### Test Steps

1. Open the GitButler desktop app and open Project Settings via the existing shortcut.
2. Observe the Permissions & Governance sidebar item shows for an admin; sign in as non-admin, confirm absent.
3. Click Permissions & Governance and observe four tabs: Principals, Groups, Branch Gates, Rules.
4. On Principals, click a principal row, toggle an own-grant permission on; observe a pending circle and commit banner.
5. On Groups, expand a group and grant a permission; observe the pending circle and banner persist across tabs.
6. Click Commit changes and observe all pending indicators clear and the effective set update.

## Tasks

| ID              | Title                                                                                                         | Agent                 | Estimate |
| --------------- | ------------------------------------------------------------------------------------------------------------- | --------------------- | -------- |
| MGMT-IPC-001    | `#[but_api]` governance fns (perm/group/status) + `json::Error` transport                                     | rust-implementer      | 90 min   |
| MGMT-IPC-002    | `json.rs` `Error` serializes the 3rd field `remediation_hint` (closes a real drop bug)                        | rust-implementer      | 75 min   |
| MGMT-IPC-003    | Register governance commands in `generate_handler!` + capability + v1 human-fleet-owner identity (T-MGMT-042) | tauri-implementer     | 60 min   |
| MGMT-IPC-004    | Regenerate `packages/but-sdk` (perm/group/status) — SDK build-gate before UI wiring                           | tauri-implementer     | 45 min   |
| MGMT-IPC-005    | Pending-until-committed read IPC contract (working-tree vs target-ref)                                        | tauri-implementer     | 60 min   |
| MGMT-UI-001     | Register the governance page + extend `ProjectSettingsPageId` (in `uiState.svelte.ts`)                        | sveltekit-implementer | 30 min   |
| MGMT-UI-002     | `ProjectSettingsModalContent` governance branch + **wire `isAdmin`** to `SettingsModalLayout`                 | sveltekit-implementer | 45 min   |
| MGMT-UI-003     | `GovernanceSettings.svelte` + client-only pending-state store (no `+page.server.ts`)                          | sveltekit-implementer | 90 min   |
| MGMT-UI-005     | `GovernancePendingBanner` (warning InfoMessage + Commit action)                                               | sveltekit-implementer | 30 min   |
| MGMT-UI-006     | `PrincipalsList` (rows + inline editor; inherited rows read-only)                                             | sveltekit-implementer | 90 min   |
| MGMT-UI-007     | `PrincipalEditor` (SegmentControl presets + Toggle table + group TagInput)                                    | sveltekit-implementer | 90 min   |
| MGMT-UI-008     | `GroupsList` (ExpandableSection per group; create/grant/add-member)                                           | sveltekit-implementer | 75 min   |
| DESIGN-MGMT-001 | Wireframe-fidelity + visual-state annotations for all four tabs                                               | frontend-designer     | 60 min   |
| DESIGN-MGMT-002 | Pending-state visual contract (○ badge, count banner, commit affordance)                                      | frontend-designer     | 45 min   |
| DESIGN-MGMT-003 | Read-only state (disabled-control treatment + `administration:write` info banner)                             | frontend-designer     | 30 min   |
| DESIGN-MGMT-005 | Inherited-vs-own permission row distinction in `PrincipalEditor`                                              | frontend-designer     | 40 min   |

## Dependencies

- **Blocks:** Sprint 06b
- **Dependent on:** Sprint 02 (admin-write), Sprint 05 (perm/group `but-api` fns). **MGMT-IPC-004 (SDK regen)
  is a hard predecessor of every `but-sdk`-importing UI task.**

### Intra-sprint dependency chain

```
MGMT-IPC-002 (json::Error transport) ─┐
MGMT-IPC-001 (but-api gov fns) ───────┼→ MGMT-IPC-003 (register + capability + identity)
                                       └→ MGMT-IPC-004 (SDK regen) ─→ every MGMT-UI-* task
DESIGN-MGMT-001/002/003/005 ──────────→ MGMT-UI-003/005/006/007/008 (design source)
MGMT-UI-001 (page id + CT harness) ─→ MGMT-UI-002 (branch + isAdmin) ─→ MGMT-UI-003 (page + pending store)
MGMT-UI-003 ─┬→ MGMT-UI-005 (pending banner)
             ├→ MGMT-UI-006 (PrincipalsList) ─→ MGMT-UI-007 (PrincipalEditor)
             └→ MGMT-UI-008 (GroupsList)
```

## PRD Coverage

- **Use cases:** UC-MGMT-01, UC-MGMT-02, UC-MGMT-03, UC-MGMT-06 (pending-until-committed half)
- **Criteria:** T-MGMT-001..016, T-MGMT-027/028/033/034/035/036 (all component-test criteria gated on the
  T-MGMT-000 desktop-CT-harness prerequisite carried by MGMT-UI-001)

## Capability Coverage

- **CAP-AUTHZ-01** — every governed write goes through `but-api` → `but-authz` `authorize()`
  (`administration:write`); the UI never provides a bypass (server-side enforcement; renderer `adminOnly` is
  UX only). MGMT-IPC-001/003 carry the producer obligation; the UI tasks consume it without weakening it.

## Coverage Notes

- **Re-uses the Sprint-05 `but-api` perm/group functions — does NOT re-author them.** MGMT-IPC-001 wraps the
  existing `perm_list`/`perm_grant`/`perm_revoke`/`group_*` functions (`crates/but-api/src/legacy/governance.rs`)
  as `#[but_api]` governance commands; it must not fork a parallel implementation. The Tauri command ↔ CLI
  verb mapping is fixed by [`04-api-design.md`](../../10-technical-requirements/04-api-design.md) (`perm_grant`
  → `but perm grant`, etc.). `governance_status_read` is the UI-specific **self-scoped** read (the viewer's
  own effective set) for read-only display.
- **The `remediation_hint` transport drop is a real bug, not a hypothetical (MGMT-IPC-002).** `but_api::json::Error`
  serializes only `{code, message}` today; the `Denial` contract's third field `remediation_hint`
  ([`04-api-design.md`](../../10-technical-requirements/04-api-design.md)) is dropped, so a denied governance
  write reaches the renderer without its remediation text and UC-MGMT-06/07's structured-denial banner cannot
  show the hint. The fix is additive (a 3rd serialized field), proven by a denial round-trip.
- **SDK regen is a hard gate, not a courtesy (MGMT-IPC-004).** Per the SDK generation flow (`RULES.md`),
  `packages/but-sdk` is **generated** from Rust APIs; `pnpm build:sdk && pnpm format` must run after the
  governance commands land and **before** any UI task imports the new types, or the MGMT components cannot
  type-check. MGMT-IPC-004 is the build-gate that produces the typed surface every MGMT-UI-\* task consumes.
- **No new route — a STATE of the settings modal (MGMT-UI-001/002).** Per the route-vs-state discriminator,
  the governance surface adds a _section_ to the existing Project Settings modal, so it extends the
  `ProjectSettingsPageId` union and branches `ProjectSettingsModalContent.svelte`; **no new top-level
  `[projectId]/…` route** is introduced. All pending-state tracking is CLIENT-ONLY (a Svelte store, no
  `+page.server.ts`), consistent with `apps/desktop` adapter-static (T-MGMT-036).
- **Admin-gating: renderer is UX, server is enforcement (MGMT-UI-002).** In v1, the sidebar `adminOnly`
  filter uses the cloud `User.role === 'admin'` flag (B18); the functional `administration:write` check at
  the `but-api` command boundary is the real enforcement boundary. MGMT-UI-002 must **wire `isAdmin`** into
  `SettingsModalLayout` (the documented `isAdmin` gap) — but the wiring is a convenience layer, never the
  authorization seam.
- **Pending-until-committed is derived from a diff, never optimistically applied (MGMT-IPC-005, MGMT-UI-003/005).**
  A governance write edits the working-tree `.gitbutler/*.toml`; the UI derives the pending (○) state from the
  working-tree-vs-target-ref diff (T-MGMT-035), shows a warning `InfoMessage` commit banner (T-MGMT-028), and
  clears it on commit. There is NO renderer direct-file-write and NO optimistic enforcement application
  (T-MGMT-027).
- **Batch-save, not per-toggle (MGMT-UI-006/007 — B16).** The per-principal editor stages toggle + group-chip
  changes in **local UI state only**; `[Save changes]` writes the staged set as a sequence of the existing
  `but perm grant`/`revoke` (+ `but group add/remove-member`) SDK calls — there is no per-toggle write and no
  new overwrite verb (`but perm set` deferred — C2). Inherited rows render read-only (disabled); a role
  preset sets own-grant toggles without stripping an inherited grant (union semantics preserved).
- **Component reuse — no new design-system work (all UI + DESIGN tasks).** Every control already exists in
  `packages/ui` or `apps/desktop/src/components/shared` (`Tabs`, `Toggle`, `SegmentControl`, `TagInput`,
  `ExpandableSection`, `InfoMessage`, `Badge`, `EmptyStatePlaceholder`, `Button`, `KebabButton`). The net-new
  components are thin compositions (`GovernanceSettings`, `PrincipalsList`, `PrincipalEditor`, `GroupsList`,
  `GovernancePendingBanner`). Cite the verified component paths from
  [`10-ui-infrastructure.md`](../../10-technical-requirements/10-ui-infrastructure.md).
- **Branch Gates + Rules tabs + safety land in Sprint 06b — this sprint ships the page shell + Principals +
  Groups + pending.** The four-tab IA is present (T-MGMT-004), but `BranchGatesList`/`RulesList(principalId)`,
  the error boundary, accessibility, and the read-only/denial-no-flip safety contract are Sprint 06b. The
  read-only state (DESIGN-MGMT-003) is designed here so the Principals/Groups controls disable correctly
  under missing `administration:write`.
- **Implementation is out of scope for this artifact:** these are TDD **task contracts**. The Rust (the
  `#[but_api]` wrappers, the `json::Error` field), the Tauri/SDK wiring, and the Svelte components are
  written at execution time by `/kb-run-sprint`, RED→GREEN against these specs and the regenerated SDK.

> **Replaces the SPRINT.md skeleton with the JIT-expanded contract.** This file was generated by
> `/kb-sprint-tasks-plan` from ROADMAP.md Sprint 06a. The per-task detail files (below) carry the stable
> `AC-N`/`TC-N` Requirement Contract that `/kb-run-sprint` consumes.

## Red-Hat Review Summary

Expanded by `/kb-sprint-tasks-plan` on 2026-06-19 — **1 full red-hat goal loop, 1 review cycle + retained-writer
remediation + cycle-2 deterministic re-validation**. All 16 tasks fakeability-CLEAN (`validate_scenario`,
0 CRITICAL/HIGH on every behavioral FEATURE AC) · `proposed_by` tripwire 16/16 · stable gapless AC-N/TC-N ids ·
rubric ≥80 all (FEATURE ≈110–115).

Authored by the dispatched specialist set — `rust-planner` (MGMT-IPC-001/002), `tauri-planner` (MGMT-IPC-003/
004/005), `sveltekit-planner` (the 7 MGMT-UI tasks), `frontend-designer` (the 4 DESIGN tasks). The orchestrator
consolidated (dedupe, dependency sequencing, stable-ID assignment, fakeability normalization, contract
rendering); it did not author task content.

A fresh adversarial 4-reviewer panel (`rust-reviewer` + `tauri-reviewer` + `sveltekit-reviewer` +
`security-auditor`, no authoring context) **BLOCKed** the first draft with **21 blocking findings (7 CRITICAL +
14 MEDIUM) + 12 LOW** — coverage- and spec-correctness gaps the rubric + fakeability gates cannot see — all
remediated by the retained domain writers and confirmed by a cycle-2 deterministic re-validation (contract
integrity + `proposed_by` + gapless ids + 0-CRITICAL fakeability + per-finding spot-checks, all GREEN):

- **R1 (CRITICAL)** — `#[but_api]` REQUIRES a `Context` param (macro lib.rs:560) but the Sprint-05
  `governance.rs` fns take `&gix::Repository`; the "wrap without touching bodies" premise was impossible.
  Re-scoped MGMT-IPC-001 to a NEW thin Context-param `#[but_api]` **wrapper layer** (`*_cmd(ctx,…)`) that
  resolves repo + workspace target ref from `ctx` and delegates to the un-forked Sprint-05 `&repo` fns.
- **R2 (CRITICAL)** — MGMT-IPC-002 AC-1 asserted a fabricated `remediation_hint`; corrected to the real
  `Denial::missing_permission` output `"request a reviewed merge or ask a maintainer to grant reviews:write"`.
- **T1 (CRITICAL)** — MGMT-IPC-003 fleet-owner authz model pinned to the v1 R12 **unconditional-superuser**
  bypass (NOT a `permissions.toml` lookup), with a bootstrap proof (no committed config → fleet-owner can act)
  and a negative AC (a `BUT_AGENT_HANDLE` agent handle cannot shadow the fleet-owner).
- **S1 (CRITICAL)** — MGMT-UI-001 AC-2 was missing the required `icon: IconName` field (would fail `tsc`); added
  a verified icon (`'lock'`).
- **S2 / S3 (CRITICAL)** — added the missing PRD-mandated ACs: T-MGMT-046 (last-member-of-required-group
  warning, MGMT-UI-008) and T-MGMT-030 (self-escalation not optimistically applied, MGMT-UI-007).
- **MEDIUM set** — identity-layering note (R3); honesty-grep non-vacuous coverage of `governance.rs` (R4);
  `governance_status_read` signature + self-scope negative (R5/SEC3); `GrantOutcome` caveat-as-returned-value
  (R6); fleet-owner identity-shim seam + IPC-002 dependency + capability-scope grep (SEC1/T2/T7); SDK gate
  reframed to a destructive round-trip (T4); `governance_pending` authority assignment (T5); `isReadOnly`
  sourced from `governance_status_read` not the cloud role (SEC2); commit-through-gate + clean-tree banner
  (S5); cross-tab pending persistence (S7); `group_revoke` path (S4); DESIGN-task dependency edges (S6).
- **SEC5 (cross-task)** — a `ConfigInvalid` carrier was added to the MGMT-IPC-002 downcast set so
  `config.invalid` errors carry a `remediation_hint` over transport; MGMT-IPC-005 AC-5 consumes it.

**Upstream advisories (locked PRD — carried, NOT edited; reconcile via `/kb-sprint-plan --delta-replan` or
`/kb-prd-plan`):** (1) `10-technical-requirements/04-api-design.md:80` `allow-perm_*`/`allow-group_*` capability
language is superseded by the live `core:default` convention (no per-command allow files in this repo); (2) the
`04-api-design.md` Tauri command table is missing the `governance_pending` 13th-command row; (3) `08-uc-mgmt.md:57`
says `ProjectSettingsPageId` lives in `projectSettingsPages.ts` but the live code + `10-ui-infrastructure.md` say
`uiState.svelte.ts` (the tasks correctly target `uiState.svelte.ts`).

## Task Detail Files

Generated by `/kb-sprint-tasks-plan` on 2026-06-19.

- `MGMT-IPC-001-but-api-governance-fns.md`
- `MGMT-IPC-002-json-error-remediation-hint.md`
- `MGMT-IPC-003-register-governance-commands.md`
- `MGMT-IPC-004-sdk-regen.md`
- `MGMT-IPC-005-pending-read-ipc-contract.md`
- `MGMT-UI-001-register-page-ct-harness.md`
- `MGMT-UI-002-settings-branch-isadmin.md`
- `MGMT-UI-003-governance-settings-pending-store.md`
- `MGMT-UI-005-governance-pending-banner.md`
- `MGMT-UI-006-principals-list.md`
- `MGMT-UI-007-principal-editor.md`
- `MGMT-UI-008-groups-list.md`
- `DESIGN-MGMT-001-four-tab-annotations.md`
- `DESIGN-MGMT-002-pending-state-contract.md`
- `DESIGN-MGMT-003-read-only-state.md`
- `DESIGN-MGMT-005-inherited-vs-own-rows.md`
