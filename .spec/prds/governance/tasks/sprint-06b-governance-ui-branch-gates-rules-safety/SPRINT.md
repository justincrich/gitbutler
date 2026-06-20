---
sprint: 06b
sequence: 8
timeline: Phase 4 — Governance management UI
status: Planned
proposed_by: sveltekit-planner + frontend-designer + rust-planner (MGMT backend)
milestone: sprint-06b
prd: ../../README.md
roadmap: ../../ROADMAP.md
generated_by: kb-sprint-tasks-plan
---

# Sprint 06b: Governance UI — Branch Gates + Rules + Safety

**Sequence:** 8
**Timeline:** Phase 4 — Governance management UI
**Status:** Planned
**Proposed by:** sveltekit-planner + frontend-designer + rust-planner (MGMT backend); split authored by sveltekit-reviewer (red-hat)
**Milestone:** — (`sprint-06b`)

## Overview

The **second and final governance-management UI** sprint — it completes the four-tab surface Sprint 06a
scaffolded (Principals · Groups · **Branch Gates · Rules**) and lands the cross-cutting **safety contract**
that makes the whole surface honest: read-only under missing `administration:write`, self-escalation that is
surfaced as a structured denial without flipping the control, accessibility (aria + keyboard nav), and an
IPC-failure danger banner with Retry behind an error boundary. Sprint 06a built the page shell + Principals +
Groups + the pending-until-committed banner; 06b fills the remaining two tabs and the UC-MGMT-06/07 safety +
error-handling half.

> **The UI is a governed front-end, never a bypass (the load-bearing invariant — unchanged from 06a).** The
> two net-new write paths this sprint introduces — the **branch-gate config write** (`gates.toml`) and (for
> Rules) a **principal-scoped read** — go through the **same** `but-authz` governed path as `but perm`/`but
> group`: `but-api` function → Tauri command → generated `packages/but-sdk`. So every governance invariant
> holds identically from the GUI: `administration:write`-gated **server-side**, ref-pinned, committed-config
> (pending until committed to the governance ref), self-escalation impossible. The read-only state and
> disabled controls are **UX convenience only** — a renderer that bypassed its own guard still hits the
> server `but-authz` gate at the `but-api` command boundary. UC-MGMT-06's "self-escalation not optimistically
> applied" is the visible proof of this: the control does **not** flip; the governed path's `perm.denied` is
> shown instead.

The sprint is split across three disciplines, producer-before-consumer:

- **Rust `but-api` MGMT backend (MGMT-BE-003/004, `rust-implementer`):** the previously-unowned **gate-config
  writer** (`branch_gates_read`/`branch_gates_update`) — the first persisted writer of `.gitbutler/gates.toml`
  (mirroring CLI-001's working-tree TOML read-modify-write for `permissions.toml`), composing the Sprint-02
  `enforce_administration_write_gate` (never re-implementing it) and writing inert-until-committed; plus the
  `principalId`-scoped rules query that backs the Rules tab. Both expose their Tauri command + SDK delta.
- **SvelteKit UI (MGMT-UI-004/009/010/011/012, `sveltekit-implementer`):** wrap `GovernanceSettings` in the
  **existing** `shared/ErrorBoundary` (no new boundary component), build `BranchGatesList` (ExpandableSection
  per branch; required-group selector = the groups defined in the Groups tab), extend `RulesList` with an
  optional `principalId` prop (backward compatible — the sole `rules/` change), add accessibility (aria +
  keyboard nav) + the IPC-failure danger banner + Retry, and the build-gate test suite (no direct config
  write, no `+page.server.ts`, SDK type-check, human-principal).
- **Design (DESIGN-MGMT-004/006/007/008, `frontend-designer`):** the structured-denial banner +
  self-escalation no-flip contract, empty states for all four tabs, the four-tab IA + aria + keyboard-nav
  contract, and the error-boundary fallback + IPC-failure/retry pattern — all reusing existing `packages/ui`
  components (no new design-system work; design source is the wireframes in
  [`10-ui-infrastructure.md`](../../10-technical-requirements/10-ui-infrastructure.md)).

> **CT-harness inherited from Sprint 06a (B14 / T-MGMT-000).** Every MGMT `[component-test]` runs on the
> `apps/desktop` Playwright CT/Vitest config that **MGMT-UI-001 (Sprint 06a) carries**. Sprint 06b's component
> tests assume that harness exists (a hard cross-sprint dependency on 06a); they do not re-scaffold it.

This sprint is **desktop UI** — its gate is verified by *using the Governance settings page* (edit a branch
gate, scope the Rules tab to a principal, open the page without `administration:write` and see it read-only,
attempt a self-escalation and watch the toggle refuse to flip, keyboard-navigate the tabs, and force an IPC
failure to see the Retry banner). Every gate proof draws from
[`11-e2e-testing-criteria.md`](../../11-e2e-testing-criteria.md).

## Human Testing Gate

**Gate:** An admin edits a branch gate on the Branch Gates tab (pending indicator appears), selects a
principal on the Rules tab and confirms only that principal's rules are shown, then opens the page as a user
lacking `administration:write` and observes all controls disabled with a read-only InfoMessage, and attempts
a self-escalation and sees the denial InfoMessage without the toggle flipping.

### Test Steps

1. On Branch Gates, toggle Protected branch for a pattern; observe the pending indicator appears.
2. On Rules, select principal A and confirm only A's rules show; select B and confirm A's are absent.
3. Sign in as a user lacking `administration:write`; open the page; observe all controls disabled with a read-only banner.
4. As an admin, attempt to grant yourself `administration:write`; observe the denial banner and the toggle does not flip.
5. Navigate the four tabs by keyboard (Tab then Arrow keys); observe focus moves and tabs activate.
6. Trigger an IPC failure; observe a danger banner with a Retry button and the page stays read-only.

## Tasks

| ID | Title | Agent | Estimate |
|----|-------|-------|----------|
| MGMT-BE-004 | `branch_gates_read`/`branch_gates_update` gate-config `but-api` producer (the gates.toml writer) + its Tauri command/SDK delta | rust-implementer | 180 min |
| MGMT-BE-003 | `principalId`-scoped rules query (backend for the Rules tab) | rust-implementer | 120 min |
| MGMT-UI-004 | Wrap `GovernanceSettings` in the existing `shared/ErrorBoundary` (no new boundary component) | sveltekit-implementer | 30 min |
| MGMT-UI-009 | `BranchGatesList` (ExpandableSection per branch; required-group selector = defined groups) | sveltekit-implementer | 75 min |
| MGMT-UI-010 | Extend `RulesList` with optional `principalId` prop (backward compatible) | sveltekit-implementer | 45 min |
| MGMT-UI-011 | Accessibility (aria + keyboard nav) + IPC-failure danger banner + Retry | sveltekit-implementer | 60 min |
| MGMT-UI-012 | Build-gate tests: no direct config write, no `+page.server.ts`, SDK type-check, human-principal | sveltekit-implementer | 45 min |
| DESIGN-MGMT-004 | Structured-denial banner + self-escalation no-flip contract | frontend-designer | 30 min |
| DESIGN-MGMT-006 | Empty states for all four tabs | frontend-designer | 25 min |
| DESIGN-MGMT-007 | Four-tab IA + aria + keyboard-nav contract | frontend-designer | 35 min |
| DESIGN-MGMT-008 | Error-boundary fallback + IPC-failure/retry pattern | frontend-designer | 30 min |

## Dependencies

- **Blocks:** None (final sprint of the POC roadmap)
- **Dependent on:** Sprint 06a (page scaffold + pending store + IPC base + desktop CT harness), Sprint 04
  (gate engine the branch gates configure). **The desktop CT harness from MGMT-UI-001 (06a) is a hard
  predecessor of every `[component-test]` here; the SDK regen from MGMT-IPC-004 (06a) is extended by
  MGMT-BE-003/004's command/SDK deltas.**

### Intra-sprint dependency chain

```
DESIGN-MGMT-004/006/007/008 ──────────────→ MGMT-UI-009/011 (design source)
MGMT-BE-004 (gates.toml writer + SDK) ─────→ MGMT-UI-009 (BranchGatesList consumes the SDK)
MGMT-BE-003 (principalId rules query + SDK) → MGMT-UI-010 (RulesList principalId prop consumes the SDK)
MGMT-UI-004 (error boundary wrap) ─────────→ MGMT-UI-011 (a11y + IPC-failure banner inside the boundary)
MGMT-UI-009/010/011 ───────────────────────→ MGMT-UI-012 (build-gate tests assert the no-bypass invariants)
```

## PRD Coverage

- **Use cases:** UC-MGMT-04 (branch gates), UC-MGMT-05 (per-agent rules reuse), UC-MGMT-06 (read-only +
  denial-no-flip half), UC-MGMT-07 (error boundary + a11y + IPC retry)
- **Criteria:** T-MGMT-017..026 (branch-gates + rules tabs), T-MGMT-047 (unprotect confirmation),
  T-MGMT-029/030/031 (read-only / self-escalation-no-flip / structured-denial), T-MGMT-037/038/039/040/041
  (error boundary + keyboard/aria + IPC failure + retry + the UC-MGMT-07 e2e), T-MGMT-042 (human-principal
  build-gate). All `[component-test]` criteria are gated on the T-MGMT-000 desktop-CT harness carried by
  Sprint 06a (MGMT-UI-001).

## Capability Coverage

- **CAP-AUTHZ-01** + **CAP-CONFIG-01** — `branch_gates_update` (MGMT-BE-004) authorizes `administration:write`
  at the target ref (composing the Sprint-02 `enforce_administration_write_gate`) and writes
  inert-until-committed `gates.toml`; it is the previously-unowned gate-config **producer** (the RUST-3 fix).
  The UI tasks consume it without weakening it — self-escalation is surfaced, never optimistically applied
  (UC-MGMT-06), proving the no-bypass promise from the consumer side.

## Coverage Notes

- **MGMT-BE-004 is the first persisted writer of `gates.toml` — a net-new producer, not a loader edit.**
  `crates/but-authz/src/config.rs` is **loader-only** (`#[derive(Deserialize)]`, `#[serde(deny_unknown_fields)]`;
  the wire structs today model only `{name, protected}`). The writer must (a) verify the **full** gate-field
  set the merge gate actually reads — protected, `min_approvals`, `require_distinct_from_author`,
  `require_approval_from_group` (per UC-MGMT-04 + the Sprint-04 merge-gate work) — and round-trip **all** of
  them (additive `#[derive(Serialize)]` on raw wire structs, **not** a lossy domain round-trip), and (b) mirror
  CLI-001's (Sprint 05) working-tree TOML read-modify-write pattern. It is sited beside the Sprint-05
  `governance.rs` / the existing `config_mutate.rs` in `but-api`, and **composes** the Sprint-02
  `enforce_administration_write_gate` (`crates/but-api/src/legacy/config_mutate.rs:18`) — never re-implements
  admin gating. Writes are inert-until-committed (working tree only). This closes the RUST-3 "unowned
  `gates.toml` writer" gap the roadmap red-hat flagged.
- **MGMT-BE-003 scopes rules by principal — verify the data model first.** `RulesList` today queries
  `rulesService.workspaceRules(projectId)` (`apps/desktop/src/lib/rules/rulesService.svelte.ts:31`) with no
  principal notion. MGMT-BE-003 must verify whether `but-rules` rules carry a principal/agent association and
  whether a principal-scoped query exists or must be added at the `but-api` boundary; the backend query is the
  producer for the optional `principalId` prop MGMT-UI-010 adds. Ground against the live `but-rules` crate —
  do not assume a scoping field exists.
- **MGMT-UI-004 wraps the EXISTING `shared/ErrorBoundary` — no new component (supersedes the UI-infra doc's
  `GovernanceErrorBoundary`).** `apps/desktop/src/components/shared/ErrorBoundary.svelte` already exists and is
  built on the Svelte 5 `svelte:boundary` mechanism (this also resolves the roadmap's open advisory about the
  `svelte:boundary` choice). The task wraps `GovernanceSettings.svelte` (the 06a mount point) in it; it does
  NOT author a net-new `GovernanceErrorBoundary.svelte` (the 10-ui-infrastructure.md net-new list is
  superseded by this red-hat refinement).
- **MGMT-UI-010 is the SOLE change to the `rules/` components.** `RulesList.svelte` takes only `projectId`
  today (`apps/desktop/src/components/rules/RulesList.svelte:20-24`); the task adds an **optional**
  `principalId` prop — when set, the query is scoped via MGMT-BE-003; when unset, behavior is byte-identical to
  today (backward compatible). `Rule`/`RuleEditor`/`RuleFiltersEditor`/`NewRuleMenu` render unchanged.
- **The Branch Gates required-group selector offers only DEFINED groups (T-MGMT-020).** `BranchGatesList`
  (MGMT-UI-009) sources its required-group options from the same group set the Groups tab manages, so a gate
  can never require an undefined group. Unprotecting a branch is destructive → a confirmation dialog
  ("Unprotect branch main? Merges will no longer require review.") precedes the staged write (T-MGMT-047 / B17).
- **Component reuse — no new design-system work (all UI + DESIGN tasks).** Every control exists in
  `packages/ui` or `apps/desktop/src/components/shared` (`Tabs`, `ExpandableSection`, `Toggle`, `Textbox`
  type=number, `Select`/`TagInput`, `InfoMessage`, `EmptyStatePlaceholder`, `Modal` for confirmation,
  `Button`, `chipToasts`, `ErrorBoundary`). Cite the verified component paths from
  [`10-ui-infrastructure.md`](../../10-technical-requirements/10-ui-infrastructure.md). The design source is
  the ASCII wireframes in that doc + [`08-uc-mgmt.md`](../../08-uc-mgmt.md) — there are no external
  `concepts/*.html` design files for this PRD.
- **Self-escalation no-flip is the visible no-bypass proof (UC-MGMT-06 / T-MGMT-030).** When an admin grants
  itself `administration:write`, the governed path returns `perm.denied`; the UI surfaces the structured
  denial (danger `InfoMessage`) and **does not** flip the control. This is a consumer-side proof of
  CAP-AUTHZ-01 — the renderer never optimistically applies an authority change the server would refuse.
- **Component tests render real components against a sanctioned `but-sdk` mock layer (B14).** Per the PRD,
  desktop `[component-test]`s run the **real** Svelte components with the **Tauri IPC transport** mocked at the
  `but-sdk` seam (the desktop CT harness cannot spawn a real Tauri backend). The **real** governance
  enforcement is proven by the rust integration tasks (MGMT-BE-003/004 against real `but-authz` + real git);
  the UI component tests prove wiring/state/denial-surfacing against real components. This is the
  spec-sanctioned seam, not a stub of core logic.
- **Implementation is out of scope for this artifact:** these are TDD **task contracts**. The Rust
  (`branch_gates_*`, the principal-scoped rules query), the Tauri/SDK deltas, and the Svelte components are
  written at execution time by `/kb-run-sprint`, RED→GREEN against these specs and the regenerated SDK.

> **Replaces the SPRINT.md skeleton with the JIT-expanded contract.** This file was generated by
> `/kb-sprint-tasks-plan` from ROADMAP.md Sprint 06b. The per-task detail files (below, once written) carry
> the stable `AC-N`/`TC-N` Requirement Contract that `/kb-run-sprint` consumes.

## Task Detail Files

Generated by /kb-sprint-tasks-plan on 2026-06-19 (11 tasks · avg 114.3/115 rubric · fakeability floor 0 CRITICAL/0 HIGH on all 6 FEATURE tasks · 1 full red-hat goal-loop cycle — fresh rust-reviewer + sveltekit-reviewer + security-auditor; 9 CRITICAL + 15 MEDIUM resolved by the retained writers, 5 advisory recorded).

- MGMT-BE-004-branch-gates-config-writer.md
- MGMT-BE-003-principal-scoped-rules-query.md
- MGMT-UI-004-error-boundary-wrap.md
- MGMT-UI-009-branch-gates-list.md
- MGMT-UI-010-ruleslist-principalid.md
- MGMT-UI-011-accessibility-ipc-retry.md
- MGMT-UI-012-build-gate-tests.md
- DESIGN-MGMT-004-denial-no-flip-contract.md
- DESIGN-MGMT-006-empty-states.md
- DESIGN-MGMT-007-four-tab-a11y-contract.md
- DESIGN-MGMT-008-error-boundary-ipc-retry.md
