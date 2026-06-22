# Sprint 07 — UI Design Contracts (Local Agent PR)

> **Consolidated design reference** for the three Sprint-07 UI surfaces driven by the
> Local Agent PR (LPR) backend. This document is the **design authority**: it pins
> placement, component selection, copy, states, accessibility, edge cases, and
> dependencies so that the `sveltekit-implementer` tasks (LPR-012, LPR-014, LPR-016)
> are a transcription, not a design exercise.
>
> **This is a DESIGN contract.** No code, no HTML/CSS mockups. Every component named
> below already exists in `@gitbutler/ui`; every color reference uses an existing
> design-system token. No new tokens are introduced.

## Documents in this contract

| ID | Title | Effort | Implementer task | Backend dep |
|----|-------|--------|------------------|-------------|
| **DESIGN-LPR-001** | `keep_reviews_local` toggle — Project Settings | 30 min | LPR-012 | LPR-006 |
| **DESIGN-LPR-002** | Principal `kind` (Human/Agent) — Principals editor | 30 min | LPR-014 | LPR-005, LPR-013 |
| **DESIGN-LPR-003** | Local-review view panel (read-only) | 50 min | LPR-016 | LPR-005, LPR-015 |

The per-task execution contracts (`DESIGN-LPR-001-…md`, `DESIGN-LPR-002-…md`,
`DESIGN-LPR-003-…md` in this directory) carry the formal `AC-N`/`TC-N` Requirement
Contracts that `/kb-run-sprint` consumes. This file is the readable design synthesis;
where the two disagree on copy phrasing, **the per-task contracts' verbatim
R21/R19 parentheticals win** (they are grep-audited).

---

## Shared foundations (apply to all three surfaces)

### Component inventory (verified `@gitbutler/ui` APIs)

Every surface below is built **only** from these already-shipped components. No new
components, no new tokens.

| Component | Import path | Key props (verified) |
|-----------|-------------|----------------------|
| `Toggle` | `@gitbutler/ui` | `id`, `checked` (bindable), `disabled`, `onchange(checked)`, `testId`. Checkbox-based; Space/Enter handled internally. **No `style` prop** — styling is token-driven. |
| `Badge` | `@gitbutler/ui` | `style` (color type), `kind` (`"solid"`\|`"soft"`), `size` (`"icon"`\|`"tag"`), `tooltip`, `icon`, `testId`, `children` (snippet). |
| `CardGroup` / `CardGroup.Item` | `@gitbutler/ui` | `labelFor` (renders the item as a `<label for>` — clicking title/caption activates the bound control), `standalone`, `disabled`, snippets: `title`, `caption`, `actions`, `children`. |
| `Select` / `SelectItem` | `@gitbutler/ui` | `value`, `options: {label,value}[]`, `wide`, `onselect(value)`, `itemSnippet`. Used by `ForgeForm.svelte`. |
| `SegmentControl` / `SegmentControl.Item` | `@gitbutler/ui` | `selected`, `onselect(id)`, items: `id`, `disabled`. Radio-group equivalent; **already used in `PrincipalEditor.svelte`** for the Preset row. |
| `InfoMessage` | `@gitbutler/ui` | `style` (`"info"`\|`"warning"`\|`"danger"`\|`"success"`), `outlined`, `filled`, snippets: `title`, `content`, `testId`. |
| `EmptyStatePlaceholder` | `@gitbutler/ui` | snippets: `title`, `caption`, `actions` (omit `actions` for a no-button empty state). |

### Design-system color map (Badge `style` → meaning)

The `Badge` `style` prop accepts: `gray`, `pop`, `safe`, `warning`, `danger`, `purple`.

| Badge `style` | Visual | Semantic use in this contract |
|---------------|--------|-------------------------------|
| `gray` | neutral | **Descriptor / peer label** (Human, Agent, group, Draft, Pending). Never implies trust or status. |
| `pop` | brand accent | **Awaiting attention** — closest available tint to "info". (See note below.) |
| `safe` | green | **Success / approved** (Approved assignment, Approved/Mergeable lifecycle). |
| `warning` | yellow | **Action needed / changes requested**. |
| `danger` | red | Errors / denials (not used by these three surfaces except existing error paths). |

> **Honest gap — no `info`/`blue` Badge variant exists.** The prior per-task
> contracts phrase AwaitingReview as "info/blue". The shipped design system has no
> blue/info Badge tint. **AwaitingReview maps to `pop`** (the brand accent — the only
> remaining "draws the eye without signaling success/warning/danger" tint). If `pop`
> reads too much like a primary-action affordance in review, fall back to `gray` with
> an `icon`; **do not** invent a new token. This is the single deviation from the
> per-task copy, flagged here so the implementer does not silently introduce
> `var(--review-*)`.

### Honesty invariants (load-bearing — grep-audited at build time)

These three residuals are **named, accepted, never presented as closed**. Every copy
string in this contract honors them:

- **R12 / R21** — `keep_reviews_local` is a **trusted-desktop operator preference**
  stored in the project store, *not* an authorization boundary. An untrusted
  project-store write can flip it. Copy must say so; must **never** call it a security
  control, access gate, or authorization boundary.
- **R19** — the principal `kind` field is **descriptive metadata only**. It does not
  enter `GovConfig.principals`; no gate reads it. Changing it grants/revokes nothing.
- **R22 / safe-seam** — the merge gate reads **only** `local_review_verdicts` at head.
  The drive tables (assignments, comments, meta) **never gate**. The Local-Review view
  is an **observer**; the "Approved" label is a presentation label, not a merge
  authorization. The view renders **no merge affordance**.

### Forbidden (applies to all three surfaces)

- ❌ New CSS variables (`var(--lpr-*)`, `var(--review-*)`, `var(--kind-*)`, …).
- ❌ Hex color literals.
- ❌ New components — reuse the inventory above.
- ❌ Describing any of these surfaces as enforcement, authorization, or security.
- ❌ Mutate controls in the Local-Review view (DESIGN-LPR-003).

---

# DESIGN-LPR-001 — `keep_reviews_local` toggle

### Description

A per-project toggle that controls whether agent-authored review requests stay on the
local review layer (the default) or are mirrored to the remote forge. This is a
**desktop operator preference** in the project store — the same class as
`forge_override` and `preferred_forge_user` — *not* governed, ref-pinned config, and
*not* `administration:write`-gated.

### Placement

**Project Settings modal → General section, immediately after `ForgeForm.svelte`.**

- The toggle is the **forge/artifact-routing class** of preference, so it sits beside
  `forge_override` / `preferred_forge_user`, **not** in the Governance tab
  (`GovernanceSettings.svelte`). Putting it under Governance would misrepresent a
  project-store preference as ref-pinned committed config.
- Composition neighbor: `apps/desktop/src/components/projectSettings/ForgeForm.svelte`.
  The new toggle renders directly after the `ForgeAccountConfig` items, inside the same
  `CardGroup` (or a sibling `SettingsSection`), in
  `apps/desktop/src/components/views/ProjectSettingsModalContent.svelte`'s General
  composition.

### Component selection

**Exact model: `apps/desktop/src/components/projectSettings/PreferencesForm.svelte`** —
the `omit_certificate_check` toggle. Transcribe that pattern verbatim:

```
CardGroup.Item  (standalone, labelFor="keepReviewsLocal")
  ├─ title snippet    → "Keep review requests local"
  ├─ caption snippet  → help text (state-dependent, see Copy)
  └─ actions snippet  → Toggle (id="keepReviewsLocal", checked=project.keep_reviews_local)
```

`CardGroup.Item` with `labelFor` renders the whole card as a `<label for>`, so clicking
the title or caption text activates the toggle — the same a11y win
`PreferencesForm` already gets.

### Component states

| State | `checked` | Default? | Visual |
|-------|-----------|----------|--------|
| **Enabled (local)** | `true` | ✅ **Default** | Toggle in checked position. Caption = on-state copy. |
| **Disabled (mirror to forge)** | `false` | — | Toggle in unchecked position. Caption = off-state copy. |

**Default-on is mandatory.** `Project.keep_reviews_local` is `DefaultTrue`: an older
project JSON without the field deserializes to `true`. Bind `checked` directly to
`project.keep_reviews_local` — **no `?? false` / nullish-coalescing to false**. A
project file missing the key renders the toggle **on**.

### Content specification (copy)

**Label (title snippet)** — identical in both states:

> Keep review requests local

**On-state caption** (local; the default):

> When enabled, review requests from agent principals remain local and are never
> mirrored to the remote forge. *(This is a desktop preference — it is not an
> authorization boundary. The project store is not independently verified; an
> untrusted write could flip it.)*

**Off-state caption** (mirror to forge; shown when `checked === false`):

> Review requests from agent principals will be mirrored to your forge when approved.
> Internal principal identifiers may be disclosed to the forge API — ensure all
> principals have forge accounts before enabling. *(See: desktop preference, not an
> authorization boundary.)*

> The R21 parentheticals are **verbatim and load-bearing** — the per-task contract
> grep-audits that the words "security control" / "authorization boundary" appear
> **only** in the negating caveat, never as a description of what the toggle does.

### Interaction design

- **Control**: `Toggle` (`@gitbutler/ui`). On `onchange(value)`, call
  `projectsService.updateProject({ ...project, keep_reviews_local: value })` — the
  same write path `PreferencesForm` uses for `omit_certificate_check`.
- **Persistence**: immediate (no Save button); matches the existing per-project
  preference pattern.
- **Read-only mode**: if the settings modal is in a read-only context, pass
  `disabled` to `Toggle` and `disabled` to `CardGroup.Item` (which dims + blocks
  pointer events). Reference the `DESIGN-MGMT-003` read-only contract.

### Accessibility notes

- `Toggle` `id="keepReviewsLocal"`; `CardGroup.Item` `labelFor="keepReviewsLocal"` →
  the card renders as `<label for="keepReviewsLocal">`, so the title/caption are a
  click target and a screen-reader-associated name for the checkbox.
- **Keyboard**: `Toggle` handles Space natively (checkbox) and Enter internally
  (prevents form submit, flips the value). Reachable via Tab in document order.
  `focusable` mixin supplies the focus ring (`:focus-visible` outline uses
  `var(--fill-pop-bg)`). **Do not** suppress the outline.
- `testId="keepReviewsLocalToggle"` on the `Toggle`.
- The caption must remain in the DOM (not `aria-hidden`) so assistive tech reads the
  R21 caveat when the control is focused.

### Edge cases

| Case | Treatment |
|------|-----------|
| **Project JSON predates the field** (no `keep_reviews_local` key) | `DefaultTrue` deserializes to `true` → toggle renders **on**. This is the common case for existing projects. |
| **`updateProject` fails** (e.g. backend unavailable) | Revert `checked` to the last committed value; surface an `InfoMessage` `style="danger"` above the card. Do **not** leave the toggle visually flipped while the write failed. |
| **Settings modal read-only** | `Toggle` `disabled`, `CardGroup.Item` `disabled`. Caption unchanged. |
| **Forge not configured** | The toggle still renders and is still functional — `keep_reviews_local` governs a seam that is *specified, not built* this sprint. The off-state caption already warns about forge disclosure. Do not disable the toggle for a missing forge. |

### Dependencies

- **Backend**: LPR-006 — `Project.keep_reviews_local: DefaultTrue` (the
  `ok_with_force_push` precedent, *not* the plain-`bool` `force_push_protection`
  precedent). Project-store class, R12/R21 residual.
- **SDK**: the regenerated `@gitbutler/but-sdk` `Project` type must carry
  `keep_reviews_local: boolean` (LPR-010 regen).
- **Pattern source**: `apps/desktop/src/components/projectSettings/PreferencesForm.svelte`
  (`CardGroup.Item standalone labelFor` + `Toggle`).
- **Placement neighbor**: `apps/desktop/src/components/projectSettings/ForgeForm.svelte`.
- **Read-only contract**: `DESIGN-MGMT-003` (Sprint 06a).
- **Blocks**: LPR-012 (sveltekit-implementer — the direct consumer of this contract).

---

# DESIGN-LPR-002 — Principal `kind` (Human / Agent)

### Description

An optional `kind` field on a principal — `"human"` (default / omitted) or `"agent"` —
shown as a neutral descriptor badge in the Principals list and editable via a
segmented control in the Principals editor. The field drives the **agent-authored tag**
on derived PRs (DESIGN-LPR-003) and is **descriptive metadata only** — it does not
enter `GovConfig.principals`, no gate reads it, and changing it grants or revokes
nothing (R19).

### Placement

Two surfaces, both inside the Governance tab (`GovernanceSettings.svelte` → Principals
tab → `PrincipalsList.svelte`):

1. **List row (read mode)** — a neutral `Badge` in the `.principals-list__principal`
   span of each row, immediately after `<strong>{principalId}</strong>` (and beside the
   existing pending/`isCurrentUser` badges). Model: the existing
   `<Badge style="gray" kind="soft" size="tag">group: {group}</Badge>` already rendered
   in that row.
2. **Editor panel (edit mode)** — a new "Kind" section in `PrincipalEditor.svelte`,
   placed **below the Permissions section and above the Groups `TagInput`** (i.e.
   after authority grants, before group memberships — matching the task brief). The
   editor already uses `SegmentControl` for the Preset row; the Kind control reuses
   the same component for consistency.

> The list row expands the `PrincipalEditor` inline (accordion, not a route). The Kind
> badge therefore appears in **two** places: the collapsed row (read-only badge) and
> the expanded editor (editable segmented control). Both must carry the non-enforcement
> disclosure.

### Component selection

| Surface | Component | Props |
|---------|-----------|-------|
| List row (read) | `Badge` | `style="gray"`, `kind="soft"`, `size="tag"`, `tooltip=...`, `testId`, `children` → "Agent" \| "Human" |
| Editor (edit) | `SegmentControl` + `SegmentControl.Item` | `selected={stagedKind}`, `onselect={setKind}`; items `id="human"` / `id="agent"`, `disabled={isReadOnly}` |

**Why `SegmentControl` over `Select`?** The choice is binary and the editor already
uses `SegmentControl` for Presets — a segmented control makes "Human | Agent" scannable
as peer options and keeps the editor visually consistent. `Select` is the documented
fallback if vertical space is constrained.

**Both variants use `gray` (neutral).** `gray` is the peer-descriptor tint already used
for group badges. **Never** use `safe` (green) for "Agent" — that would imply a trust
privilege. **Never** use `danger`/`warning` for "Human" — that would imply a
restriction. They are peers.

### Component states

| `kind` value | List-row badge | Editor control | Default? |
|--------------|----------------|----------------|----------|
| `undefined` / `null` / omitted | **"Human"** `Badge` | `SegmentControl` `selected="human"` | ✅ **Default** (omitted ⇒ human) |
| `"human"` | "Human" `Badge` | `selected="human"` | — |
| `"agent"` | "Agent" `Badge` | `selected="agent"` | — |

**Omitted-kind ⇒ "Human" is mandatory and explicit.** The badge is **never blank**.
This mirrors the backend's conservative default-human posture (tech-delta §A.4: absence
means human, never agent). An operator reading a row without a `kind` field must see
"Human" so the current interpretation is unambiguous without consulting docs.

### Content specification (copy)

**Badge text** (children snippet):

- `"Human"` when `kind ∈ {undefined, null, "human"}`
- `"Agent"` when `kind === "agent"`

**Badge `tooltip`** (non-enforcement disclosure — appears on hover/focus of the
list-row badge):

> Declares whether this principal is a human operator or an automated agent. This is
> descriptive metadata used for UI tagging (agent-authored PRs). It does not affect
> authorization or enforcement.

**Editor section label + caption** (caption sits below the `SegmentControl`, so the
disclosure is visible in edit mode too):

- Label: **Kind**
- Caption (verbatim the same disclosure as the tooltip, so an operator sees it in both
  read and edit modes):
  > Declares whether this principal is a human or an automated agent. Descriptive
  > metadata for UI tagging only — it does not affect authorization or enforcement.

**Segmented control items**: `id="human"` → label "Human"; `id="agent"` → label
"Agent". Item ids are **lowercase** (matching the backend `TEXT` storage / SDK wire
value); labels are capitalized for display.

### Interaction design

- **Read mode** (`PrincipalsList` row, collapsed): the `Badge` is non-interactive
  (presentation only). The tooltip surfaces on hover/focus.
- **Edit mode** (`PrincipalEditor`, row expanded): the `SegmentControl` binds to a
  `stagedKind` local state; `onselect` updates the staged value. The value persists
  through the editor's existing **Save changes** button — the same pending→commit path
  used for authorities and groups. **Do not** introduce a separate inline-save for
  `kind`; it rides the existing editor save.
- **Write path**: persists via the LPR-013 `principal_kind` Tauri command
  (`administration:write`-gated, pending→commit). The editor's existing
  `PrincipalEditorService` gains a `setKind` method on the same shape as
  `permGrant`/`groupAddMember` (returns a write result / denial code).
- **Reverting on denial**: if the save is denied, the editor's existing `resetStaged()`
  path restores `stagedKind` to the committed value and surfaces the denial in the
  existing `saveError` `InfoMessage`. No new error UI.

### Accessibility notes

- `Badge` is `role="presentation"`; its `tooltip` prop is surfaced by the wrapping
  `Tooltip` (focusable). Give the badge `testId={`principals-list-kind-${slug(id)}`}`.
- `SegmentControl` is a radio group; each `SegmentControl.Item` is a focusable option.
  Arrow-key cycling and selection are handled by the control (already used for Presets).
  `disabled={isReadOnly}` on each item.
- The Kind section needs a programmatic label: wrap the `SegmentControl` in a
  `<fieldset>`/`<div>` with an associated `<span class="principal-editor__label">Kind
  </span>` — mirror the existing Preset section's markup
  (`.principal-editor__section` + `.principal-editor__label`).
- `testId`s: list badge → `principals-list-kind-{slug}`; editor control →
  `principal-editor-kind`.

### Edge cases

| Case | Treatment |
|------|-----------|
| **Omitted `kind`** (older `permissions.toml` with no `kind` key) | Renders **"Human"** badge. Conservative default-human. Never blank. |
| **Unknown `kind` value** (e.g. a future `"bot"` leaks through) | Render the raw value in a `gray` badge (do not crash, do not hide). The SegmentControl shows **neither** item selected, with the caption still visible. Flag in a follow-up; do not invent UI for unknown values this sprint. |
| **Read-only Governance tab** (`isReadOnly`) | List badge still renders. Editor SegmentControl items are `disabled`; the badge in the row is unchanged. |
| **Save denied** (`administration:write` not held) | Existing `saveError` `InfoMessage` (`style="danger"`) fires; `stagedKind` reverts. No new affordance. |
| **`isCurrentUser` principal** | The existing "You" badge (`style="pop"`) coexists with the Kind badge — they are independent descriptors. Do not merge them. |

### Dependencies

- **Backend**: LPR-005 — the additive `kind: Option<String>` on `PrincipalWire`
  (`config.rs:424`, `#[serde(default)]`, omitted ⇒ human, does **not** enter
  `GovConfig.principals`).
- **Producer**: LPR-013 — `principal_kind` Tauri command + SDK (governed-config,
  `administration:write`-gated, pending→commit). This is the read/write binding the
  editor consumes.
- **Pattern sources**:
  - List badge: `apps/desktop/src/components/governance/PrincipalsList.svelte` (the
    `gray soft tag` group badge).
  - Editor control: `apps/desktop/src/components/governance/PrincipalEditor.svelte`
    (the Preset `SegmentControl` + `.principal-editor__section` markup).
- **Surface**: Sprint 06a Principals tab (`DESIGN-MGMT-001`, `MGMT-UI-006`,
  `MGMT-UI-007`).
- **Blocks**: LPR-014 (sveltekit-implementer — the direct consumer of this contract).
  LPR-014 in turn depends on LPR-013 (the producer).

---

# DESIGN-LPR-003 — Local-review view panel (read-only)

### Description

A **read-only** panel that renders the local review state of a branch: the derived PR
lifecycle, the agent-authored tag, reviewer assignments, comment threads, the verdict
at head, and an explicit merge-gate note. It is a **single view with lifecycle-driven
state** — not four separate views or tabs. It renders **no mutate controls** (no
Approve, no Request Changes, no Post-Comment form, no Assign, no Resolve): all writes
are driven by the `but review` CLI. The UI is an observer; the merge decision is the
merge gate's, re-derived at merge time (R22 / safe-seam).

### Placement

**A new surface within the Governance section of the branch detail view.** Two
acceptable shells (pick the one that matches the host view's existing pattern):

1. **Tab** — a fifth tab in `GovernanceSettings.svelte`'s `TabList` ("Review"),
   rendering the panel in a `TabContent`. Best if the Governance surface is the primary
   per-branch governance home.
2. **Modal / panel** — opened from branch context ("View local review"), rendered as an
   overlay. Best if the branch detail view already uses modals for related panes.

Either way, the **panel body** is identical and is governed by this contract. The
panel is **branch-scoped**: it reads `review_status` for one target branch.

### Component selection

| Section | Components |
|---------|-----------|
| Lifecycle badge | `Badge` (style varies by state — see states) |
| Agent-authored tag | `Badge` `style="gray" kind="soft" size="tag"` + `tooltip` (rendered **only** when `agent_authored === true`) |
| Assignments table | native table / `.principals-list`-style rows + `Badge` per assignment state |
| Comment threads | list groups + `Badge` for resolved/unresolved count |
| Verdict at head | inline read-only text + `Badge` |
| Merge-gate note | `InfoMessage style="info" outlined` |
| Empty (no review) | `EmptyStatePlaceholder` (title + caption snippets, **no actions**) |
| Loading | `SkeletonBone` / full-view skeleton |

### Component states (single view, four lifecycle states)

The lifecycle badge is the only thing that changes between states; the five data
sections remain in place. There is **no routing per state**.

| `DerivedPrStatus` | Badge `style` | Badge label | Meaning |
|-------------------|---------------|-------------|---------|
| `Draft` / Open | `gray` | "Open" | A review has been opened; no assignments satisfied yet. |
| `AwaitingReview` | `pop` ⚠️ *(see gap note)* | "Awaiting review" | Reviewers are assigned; no verdict at head. |
| `ChangesRequested` | `warning` | "Changes requested" | At least one assignment is `changes_requested`. |
| `Approved` | `safe` | "Approved" | Verdict-at-head is `approved`. |
| (`Mergeable` presentation alias) | `safe` | "Mergeable" | Approved + no unresolved blocking threads — **presentation label only**, the gate re-derives. |

> ⚠️ The design system has **no blue/info Badge tint**. `AwaitingReview` uses `pop`
> (brand accent) as the closest "attention without success/warning/danger" tint. If
> `pop` reads as a primary-action affordance in usability review, fall back to `gray`
> with an `icon="eye"` — **do not** introduce a new token. This is the one place the
> per-task "info/blue" wording could not be honored verbatim against the shipped
> component API.

### Content specification (five sections, top-to-bottom order)

**Section 1 — PR header**
- Lifecycle `Badge` (per states above).
- Branch display: `{source_branch} → {target_branch}`.
- **Agent-authored `Badge`** — rendered **only** when `agent_authored === true`.
  `style="gray" kind="soft" size="tag"`, children "Agent PR",
  `tooltip="This PR was opened by a principal declared as an agent in .gitbutler/permissions.toml. This is a metadata tag — it does not affect merge decisions."`
  When `agent_authored === false` the badge is **absent** — do **not** render a
  "Human" badge here (the human case is the default and needs no label; contrast
  DESIGN-LPR-002 where the principal row *does* show "Human" to make the default
  explicit).
- Derived PR metadata (read-only text): `{sha}`, `{author}`, `{created_at}`,
  `{title}`.

**Section 2 — Assignments table**
- One row per `open_assignments[]`: `{reviewer}` principal id + assignment-state
  `Badge`.
- Assignment-state badge mapping:
  - `pending` → `gray`, label "Pending"
  - `approved` → `safe`, label "Approved"
  - `changes_requested` → `warning`, label "Changes requested"
- **No assign/remove controls.** Column headers: Reviewer · State · Assigned at
  (`assigned_at`).

**Section 3 — Comment threads**
- Threads grouped by scope: code-scoped (`file` + `line`) first, then PR-level
  (`file === null`).
- Each thread shows: `{thread_id}`, comment count, unresolved count (as a small
  `Badge` — `warning` if `unresolved > 0`, else `gray` "resolved"), and a latest-comment
  preview (truncated).
- **Resolved threads are muted/collapsed but visible** (a disclosure summary like
  "3 resolved · click to expand"). **Never hide them** — the review history is context
  the operator or orchestrator may need.
- **No Post-Comment form, no Resolve button.**

**Section 4 — Verdict at head**
- Inline read-only line:
  - `approved` → `Badge style="safe"` "Approved at head"
  - `changes_requested` → `Badge style="warning"` "Changes requested at head"
  - absent → muted text "No verdict at head" (`gray`).
- One-sentence lifecycle caption beneath the verdict explaining the current state and
  naming the CLI verb that advances it, e.g. *"Awaiting review from 2 reviewers.
  Approve via `but review approve <branch>`."*

**Section 5 — Merge-gate note**
- `InfoMessage style="info" outlined` at the **bottom** of the view:
  > **Merge decisions are made by the merge gate, not this view.** The actual merge
  > decision is made by the merge gate based on verdict-at-head; a status of "Approved"
  > or "Mergeable" here reflects the derived state — the gate re-derives verdict at
  > merge time. This view is informational only.

### Interaction design

- **Read-only.** The panel renders **no** buttons or forms that mutate review state.
  Explicitly excluded (named so the implementer is not "helpful"):
  ❌ Approve · ❌ Request Changes · ❌ Post Comment form · ❌ Assign Reviewer ·
  ❌ Resolve Thread · ❌ Merge.
- **Resolved-thread disclosure**: clicking a collapsed resolved thread expands it
  in-place (read-only expansion, not a write). This is the only interactive element.
- **Refresh**: the panel re-reads `review_status` when the branch or target ref
  changes (reactive `$derived` on the branch selector), and exposes a manual refresh
  via the host view's existing refresh affordance — **not** a panel-local button that
  implies a write.
- **Rationale (must be honored in code review)**: *the Local-Review view is an
  observer — it reads and renders, never writes. All writes are driven by the CLI
  (`but review approve`, `but review comment`, `but review assign`, `but review
  resolve`).*

### Accessibility notes

- The lifecycle badge + agent-authored badge are `role="presentation"`; their tooltips
  supply the accessible description. `testId`s: `local-review-lifecycle`,
  `local-review-agent-tag`.
- The assignments table is a real `<table>` with `<th scope="col">` headers (Reviewer,
  State, Assigned at) — or, if the host uses the `.principals-list__row` grid pattern,
  a `role="table"` with `role="row"`/`role="columnheader"`. Either way, give it
  `aria-busy` during refresh.
- Resolved-thread disclosure uses `<button aria-expanded>` (the existing
  disclosure/accordion idiom); do not rely on hover-only reveal.
- The merge-gate `InfoMessage` is announced politely (it is static, not an alert).
- `EmptyStatePlaceholder` for the no-review state: `title` snippet "No local review
  open for this branch.", `caption` snippet "Open one with `but review request
  <branch>` to start the review loop." — **omit the `actions` snippet** (the action is
  a CLI command, not a UI button; a button would contradict read-only).

### Edge cases

| Case | Treatment |
|------|-----------|
| **Loading** (`review_status` in flight) | Full-view skeleton (`SkeletonBone`) spanning all five sections. `aria-busy="true"` on the container. |
| **No review open** (branch has no `local_review_meta` opener row) | `EmptyStatePlaceholder` with the title/caption above; **no actions snippet**. The CLI caption is the only "call to action". |
| **No reviewers assigned** | Within Section 2: inline muted text "No reviewers assigned yet." (not an `EmptyStatePlaceholder` — the review exists; this section is just empty). |
| **No comment threads** | Within Section 3: inline muted text "No comment threads yet." |
| **`review_status` read error** (backend unavailable / denied) | `InfoMessage style="danger" outlined` with the error code in the title and "The local review state could not be loaded." in `content`. **Do not** render a partial view with stale data. |
| **`agent_authored === false`** | The agent-authored badge is **absent** (not rendered as "Human"). The lifecycle badge and all sections render normally. |
| **Forge review also exists** (mirrored PR) | Out of scope for this sprint (`keep_reviews_local` defaults local). If a forge review coexists, this panel shows the **local** review state only; do not merge the two. |
| **Approved but merge blocked elsewhere** | Section 5 merge-gate note already covers this — do not add a second warning. The "Approved" badge is a derived label, not a merge promise. |

### Dependencies

- **Backend reads**:
  - LPR-005 — `review_status` → `DerivedPr` (`target_branch`, `source_branch`, `sha`,
    `author`, `title`, `draft`, `created_at`, `updated_at`, `status`, `agent_authored`,
    `labels`, `open_assignments`, `unresolved_thread_count`) and `DerivedPrStatus`.
  - LPR-001 — `local_review_assignments` (state field), `local_review_comments`
    (file/line/thread_id/resolved), `local_review_meta` (opener_principal →
    agent-authored tag).
  - LPR-004 — `list_comments` (branch-scoped read of threads).
- **Producer**: LPR-015 — `review_status` / `list_comments` Tauri commands + SDK
  (branch-scoped, R14 no-bypass). This is the read binding the panel consumes.
- **Pattern sources**:
  - Empty state: `packages/ui/src/lib/components/EmptyStatePlaceholder.svelte` (and the
    Sprint 06b `DESIGN-MGMT-006` empty-state pattern used by all Governance tabs).
  - Merge-gate note: `packages/ui/src/lib/components/InfoMessage.svelte`
    (`style="info"`).
  - Assignment/thread rows: `apps/desktop/src/components/governance/PrincipalsList.svelte`
    row pattern (the `gray soft tag` badges, the `.principals-list__row` grid).
- **Safe-seam invariant**: LPR-009 — the merge gate reads **only**
  `local_review_verdicts` at head; the drive tables this view renders **never gate**.
  The merge-gate note (Section 5) is the UI's honest disclosure of this seam.
- **Blocks**: LPR-016 (sveltekit-implementer — the direct consumer). LPR-016 in turn
  depends on LPR-015 (the producer).

---

## Cross-cutting dependency graph

```
LPR-006 (keep_reviews_local DefaultTrue) ──▶ DESIGN-LPR-001 ──▶ LPR-012 (toggle UI)
LPR-010 (SDK regen: Project.keep_reviews_local) ┤

LPR-005 (kind on PrincipalWire) ──────────┐
LPR-013 (principal_kind Tauri+SDK) ───────┼─▶ DESIGN-LPR-002 ──▶ LPR-014 (kind editor UI)

LPR-005 (DerivedPr + DerivedPrStatus) ─────┐
LPR-001 (assignments/comments/meta tables)┤
LPR-004 (list_comments read) ─────────────┼─▶ DESIGN-LPR-003 ──▶ LPR-016 (review view UI)
LPR-015 (review_status/list_comments Tauri)┤
LPR-009 (safe-seam: drive tables never gate)┤
```

All three design contracts are **design-only** artifacts. None of them touch the merge
gate; none mutate review state; the only two "write" surfaces (the toggle in
DESIGN-LPR-001, the kind editor in DESIGN-LPR-002) persist through existing
project-store / governed-config write paths, and the view in DESIGN-LPR-003 is strictly
read-only.

## Verification summary (maps to per-task ACs)

| This contract's requirement | Per-task AC | Audit |
|-----------------------------|-------------|-------|
| DESIGN-LPR-001: `Toggle` + `CardGroup.Item labelFor` + default-on + R21 caveat | DESIGN-LPR-001 AC-1..AC-5 | grep `var(--(lpr\|review)-` → 0 matches; design review |
| DESIGN-LPR-001: copy never calls the toggle a security control | DESIGN-LPR-001 AC-4 | grep `security control` / `authorization` in toggle copy → only the negating caveat |
| DESIGN-LPR-002: `gray` Badge + omitted⇒"Human" + non-enforcement tooltip | DESIGN-LPR-002 AC-1..AC-5 | grep `var(--(kind\|agent)-` → 0 matches |
| DESIGN-LPR-003: single view / four states / five sections / read-only | DESIGN-LPR-003 AC-1, AC-2, AC-5 | component test: no mutate buttons rendered |
| DESIGN-LPR-003: agent-authored badge present iff `agent_authored` | DESIGN-LPR-003 AC-3 | component test: absent when false |
| DESIGN-LPR-003: `EmptyStatePlaceholder` no-action for no-review | DESIGN-LPR-003 AC-4 | component test: no `actions` snippet rendered |
| DESIGN-LPR-003: lifecycle colors use only shipped Badge styles | DESIGN-LPR-003 AC-6 | grep `var(--(review\|lpr)-` → 0 matches |

> **`pop` for AwaitingReview** is the single mapping that deviates from the per-task
> "info/blue" wording. It is flagged in-line above and in the Shared Foundations color
> map. The per-task contract's `AC-6` grep (which forbids new tokens) still passes
> cleanly because `pop` is an existing style.
