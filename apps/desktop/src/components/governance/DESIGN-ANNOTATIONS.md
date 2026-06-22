# Governance Management UI Design Annotations

This document is the Sprint 06a visual contract for the Project Settings
`Permissions & Governance` surface. It maps every region in
`.spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md` to an
existing component path and, where the wireframe requires a governance-specific
row or section, to one of the seven planned thin composition components from the
Net-new components table.

It is intentionally visual-only. It does not define SDK calls, stores, Tauri
commands, authorization logic, or persistence behavior.

## Component Inventory

| Region family                             | Component                     | Source path                                                               | Required props or state                                                                                                                                                                 |
| ----------------------------------------- | ----------------------------- | ------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Settings sidebar and page scaffold        | `SettingsModalLayout`         | `apps/desktop/src/components/settings/SettingsModalLayout.svelte`         | Existing settings shell; governance page appears through the existing admin-only page filter.                                                                                           |
| Settings page host                        | `ProjectSettingsModalContent` | `apps/desktop/src/components/settings/ProjectSettingsModalContent.svelte` | Existing host branch for `GovernanceSettings.svelte`.                                                                                                                                   |
| Governance page composition               | `GovernanceSettings`          | `apps/desktop/src/components/settings/GovernanceSettings.svelte`          | Planned thin composition from the Net-new table; hosts `SettingsSection`, pending banner, tab IA, and per-tab content.                                                                  |
| Page section header and copy              | `SettingsSection`             | `apps/desktop/src/components/shared/SettingsSection.svelte`               | `title`, `description`, `top`, `gap` use the component stylesheet.                                                                                                                      |
| Secondary tab shell                       | `Tabs`                        | `apps/desktop/src/components/shared/Tabs.svelte`                          | `defaultSelected="principals"`.                                                                                                                                                         |
| Secondary tab list                        | `TabList`                     | `apps/desktop/src/components/shared/TabList.svelte`                       | The parent tab list carries `aria-label="Governance configuration tabs"`.                                                                                                               |
| Secondary tab trigger                     | `TabTrigger`                  | `apps/desktop/src/components/shared/TabTrigger.svelte`                    | `value` is one of `principals`, `groups`, `branch-gates`, `rules`; `disabled` only when the whole surface is read-only unavailable.                                                     |
| Secondary tab panel                       | `TabContent`                  | `apps/desktop/src/components/shared/TabContent.svelte`                    | `value` matches the tab id.                                                                                                                                                             |
| Pending banner composition                | `GovernancePendingBanner`     | `apps/desktop/src/components/governance/GovernancePendingBanner.svelte`   | Planned thin composition from the Net-new table; wraps `InfoMessage` warning state; hidden when `isReadOnly=true`.                                                                      |
| Banner feedback                           | `InfoMessage`                 | `packages/ui/src/lib/components/InfoMessage.svelte`                       | `style="warning"`, `style="info"`, or `style="danger"`; `outlined=true`; `primaryLabel` and `primaryIcon` when an action is present.                                                    |
| Status and role marker                    | `Badge`                       | `packages/ui/src/lib/components/Badge.svelte`                             | Pending only: `style="warning"` `kind="soft"` `size="icon"` with `○` and pending text. Committed rows render no pending `Badge`; absence is the committed signal for this contract.     |
| Principal rows                            | `PrincipalsList`              | `apps/desktop/src/components/governance/PrincipalsList.svelte`            | Planned thin composition from the Net-new table; row surface uses `CardGroupRoot.svelte` and `CardGroupItem.svelte`.                                                                    |
| Row group container                       | `CardGroupRoot`               | `packages/ui/src/lib/components/cardGroup/CardGroupRoot.svelte`           | Groups principal rows in the list body.                                                                                                                                                 |
| Row item                                  | `CardGroupItem`               | `packages/ui/src/lib/components/cardGroup/CardGroupItem.svelte`           | Single principal row, group row summary, or status row as needed by the composition.                                                                                                    |
| Row overflow actions                      | `KebabButton`                 | `packages/ui/src/lib/components/KebabButton.svelte`                       | Wireframe `...` overflow affordance; read-only compositions omit mutating actions or render menu entries through `ContextMenuItem disabled=true`.                                       |
| Add, create, save, commit, cancel actions | `Button`                      | `packages/ui/src/lib/components/Button.svelte`                            | Use component variants already exposed by the Button component; no new button styling.                                                                                                  |
| Principal editor                          | `PrincipalEditor`             | `apps/desktop/src/components/governance/PrincipalEditor.svelte`           | Planned thin composition from the Net-new table; inline editor mirrors `RuleEditor.svelte` spacing and slide-in placement.                                                              |
| Inline editor pattern                     | `RuleEditor`                  | `apps/desktop/src/components/rules/RuleEditor.svelte`                     | Pattern source only; governance editor keeps the same inline stacked editor treatment.                                                                                                  |
| Role preset strip                         | `SegmentControl`              | `packages/ui/src/lib/components/segmentControl/SegmentControl.svelte`     | `selected="write"` for the shown wireframe; role segments in order `read`, `triage`, `write`, `maintain`, `admin`.                                                                      |
| Role preset option                        | `Segment`                     | `packages/ui/src/lib/components/segmentControl/Segment.svelte`            | `id` is the role id; `disabled=true` in read-only state.                                                                                                                                |
| Permission toggle                         | `Toggle`                      | `packages/ui/src/lib/components/Toggle.svelte`                            | `checked=true` for ON, `checked=false` for OFF, `disabled=true` for inherited rows and read-only state.                                                                                 |
| Group and member chips                    | `TagInput`                    | `packages/ui/src/lib/components/TagInput.svelte`                          | `tags` contain group/member labels; `readonly=true` in read-only state; `disabled=true` when actions are unavailable.                                                                   |
| Groups tab sections                       | `GroupsList`                  | `apps/desktop/src/components/governance/GroupsList.svelte`                | Planned thin composition from the Net-new table; each group uses `ExpandableSection`.                                                                                                   |
| Branch gate sections                      | `BranchGatesList`             | `apps/desktop/src/components/governance/BranchGatesList.svelte`           | Planned thin composition from the Net-new table; each branch gate uses `ExpandableSection`.                                                                                             |
| Expandable rows                           | `ExpandableSection`           | `apps/desktop/src/components/shared/ExpandableSection.svelte`             | `label`, optional `summary`, `expanded`, `content`; group and branch sections use this directly.                                                                                        |
| Destructive confirmation                  | `Modal`                       | `packages/ui/src/lib/components/Modal.svelte`                             | Group delete and branch unprotect confirmation dialog.                                                                                                                                  |
| Min approvals input                       | `Textbox`                     | `packages/ui/src/lib/components/Textbox.svelte`                           | `type="number"`; `disabled=true` or `readonly=true` in read-only state.                                                                                                                 |
| Required group selector                   | `Select`                      | `packages/ui/src/lib/components/select/Select.svelte`                     | Used for dropdown group selection when the design needs a single picker.                                                                                                                |
| Empty states                              | `EmptyStatePlaceholder`       | `packages/ui/src/lib/components/EmptyStatePlaceholder.svelte`             | `title`, `caption`, and `actions` snippets exactly as listed below.                                                                                                                     |
| Rules tab list                            | `RulesList`                   | `apps/desktop/src/components/rules/RulesList.svelte`                      | Existing rules UI; Sprint implementation extends with optional `principalId` prop.                                                                                                      |
| Rule row                                  | `Rule`                        | `apps/desktop/src/components/rules/Rule.svelte`                           | Existing rule row unchanged.                                                                                                                                                            |
| Rule editor                               | `RuleEditor`                  | `apps/desktop/src/components/rules/RuleEditor.svelte`                     | Existing editor unchanged in the per-principal context.                                                                                                                                 |
| Rule filters                              | `RuleFiltersEditor`           | `apps/desktop/src/components/rules/RuleFiltersEditor.svelte`              | Existing filters editor unchanged.                                                                                                                                                      |
| New rule menu                             | `NewRuleMenu`                 | `apps/desktop/src/components/rules/NewRuleMenu.svelte`                    | Existing menu unchanged.                                                                                                                                                                |
| Error boundary                            | `ErrorBoundary`               | `apps/desktop/src/components/shared/ErrorBoundary.svelte`                 | Existing shared boundary only; `GovernanceSettings.svelte` is wrapped with `title="Governance settings failed to load"` and `compact=false`; no governance-specific boundary component. |

## Tab IA Contract

The tab strip is one shared tab set, not four independent screens.

| Item          | Component path                                         | Props and content                                                      |
| ------------- | ------------------------------------------------------ | ---------------------------------------------------------------------- |
| Tab wrapper   | `apps/desktop/src/components/shared/Tabs.svelte`       | `defaultSelected="principals"`.                                        |
| Tab list      | `apps/desktop/src/components/shared/TabList.svelte`    | Add `aria-label="Governance configuration tabs"` at use site.          |
| Tab 1 trigger | `apps/desktop/src/components/shared/TabTrigger.svelte` | `value="principals"`; visible label `Principals`.                      |
| Tab 2 trigger | `apps/desktop/src/components/shared/TabTrigger.svelte` | `value="groups"`; visible label `Groups`.                              |
| Tab 3 trigger | `apps/desktop/src/components/shared/TabTrigger.svelte` | `value="branch-gates"`; visible label `Branch Gates`.                  |
| Tab 4 trigger | `apps/desktop/src/components/shared/TabTrigger.svelte` | `value="rules"`; visible label `Rules`.                                |
| Tab 1 content | `apps/desktop/src/components/shared/TabContent.svelte` | `value="principals"` renders `PrincipalsList`.                         |
| Tab 2 content | `apps/desktop/src/components/shared/TabContent.svelte` | `value="groups"` renders `GroupsList`.                                 |
| Tab 3 content | `apps/desktop/src/components/shared/TabContent.svelte` | `value="branch-gates"` renders `BranchGatesList`.                      |
| Tab 4 content | `apps/desktop/src/components/shared/TabContent.svelte` | `value="rules"` renders existing rules components scoped by principal. |

Tab order is fixed: `principals`, `groups`, `branch-gates`, `rules`.

## Four-Tab Accessibility Contract

This section extends the `Tab IA Contract` above for Sprint 06b. It does not
change the component choice: the implementation vehicle remains the shared
`Tabs` composition at `apps/desktop/src/components/shared/Tabs.svelte`,
`TabList.svelte`, `TabTrigger.svelte`, and `TabContent.svelte`. The contract
follows the WAI-ARIA Tabs pattern with automatic activation: the active tab is
the selected tab, Arrow focus changes activate the destination tab immediately,
and exactly one associated `TabContent` panel is rendered.

### Aria Attribute Specification

The `TabList` wrapper for this governance tab set has:

- `role="tablist"`
- `aria-label="Governance configuration tabs"`

Every `TabTrigger` has:

- `role="tab"`
- `id="{tab-id}"`
- `aria-selected="true"` when active and `aria-selected="false"` otherwise
- `aria-controls="{panel-id}"`

Every `TabContent` has:

- `role="tabpanel"`
- `id="{panel-id}"`
- `aria-labelledby="{tab-id}"`

The stable id pairs are, in canonical order:

| Order | TabTrigger value | TabTrigger `id` | TabContent `id`      |
| ----- | ---------------- | --------------- | -------------------- |
| 1     | `principals`     | `principals`    | `principals-panel`   |
| 2     | `groups`         | `groups`        | `groups-panel`       |
| 3     | `branch-gates`   | `branch-gates`  | `branch-gates-panel` |
| 4     | `rules`          | `rules`         | `rules-panel`        |

The `aria-controls` value on each trigger is the matching panel id from this
table, and the `aria-labelledby` value on each panel is the matching trigger id
from this table. These IDs are static strings, not generated per render.

### Keyboard Navigation Contract

Keyboard interaction uses the WAI-ARIA automatic-activation model for a
horizontal tab list:

1. `Tab` from outside the tablist moves focus to the currently active
   `TabTrigger`, not to the first tab unless `principals` is already active.
2. `Arrow Left` moves focus to the previous `TabTrigger` in the fixed order and
   wraps from `principals` to `rules`; the newly focused tab becomes active,
   sets `aria-selected="true"`, and renders its panel.
3. `Arrow Right` moves focus to the next `TabTrigger` in the fixed order and
   wraps from `rules` to `principals`; the newly focused tab becomes active,
   sets `aria-selected="true"`, and renders its panel.
4. `Enter` or `Space` activates the focused `TabTrigger` if it is not already
   active, sets `aria-selected="true"`, and renders the associated panel.
5. `Tab` from within a `TabContent` panel follows normal document tab order and
   leaves the tabpanel for the next focusable element; it does not return focus
   to the tablist.

The roving tabindex rule is: only the currently active `TabTrigger` is in the
page tab sequence with `tabindex="0"`; all inactive triggers use
`tabindex="-1"`. Arrow keys, not repeated Tab presses, move focus among the four
triggers inside the tablist.

### Focus-Visible Treatment

The active-focus `TabTrigger` must show a visible `:focus-visible` indicator.
If the shared Tabs component delegates to the browser's native
`:focus-visible` outline, keep that native outline. If the component needs an
explicit replacement, use the existing project focus treatment such as
`var(--focus-outline)` only if that variable is already present in the design
tokens. Do not introduce a new focus token, do not add a governance-specific
focus ring, and do not suppress focus with `outline: none` unless an equivalent
visible replacement is applied.

### Shared Tabs Component Audit Note

Audit source:

- `apps/desktop/src/components/shared/Tabs.svelte` provides the shared context
  and wrapper only.
- `apps/desktop/src/components/shared/TabList.svelte` currently renders the list
  wrapper without `role="tablist"` and without an `aria-label` prop.
- `apps/desktop/src/components/shared/TabTrigger.svelte` currently sets
  `role="tab"` and `aria-selected`, but does not set `aria-controls`, does not
  implement Arrow Left/Right navigation, and does not expose the complete
  roving-tabindex behavior required above.
- `apps/desktop/src/components/shared/TabContent.svelte` currently renders the
  active panel without `role="tabpanel"`, `id`, or `aria-labelledby`.

Concrete action item for the sveltekit implementer: augment the shared Tabs
family so `TabList` accepts and applies `aria-label="Governance configuration
tabs"` with `role="tablist"`, `TabTrigger` applies `role="tab"`,
`aria-selected`, `aria-controls`, stable `id`, roving tabindex, and Arrow
Left/Right plus Enter/Space activation, and `TabContent` applies
`role="tabpanel"`, stable `id`, and `aria-labelledby`.

### Token Constraint

The a11y layer introduces no colors, spacing values, typography values, or
design-system variables. Visual treatment for keyboard focus either uses the
browser's native `:focus-visible` outline or an existing project focus token
already present in the design system. It must not add hex color literals and
must not add accessibility-prefixed, management-prefixed, or other
governance-specific tokens.

## Pending-State Visual Contract

Pending state is visual-only. It indicates that governed configuration has been
written to the working-tree governance config files and is waiting for the
human to commit those files to the governance ref. It does not optimistically
apply enforcement, change the effective permission set, or introduce a spinner,
progress row, toast, or new design-system primitive.

### Per-row pending indicator

Rows with staged-but-uncommitted governance changes render an inline pending
`Badge` from `packages/ui/src/lib/components/Badge.svelte` with these exact
props:

```svelte
<Badge style="warning" kind="soft" size="icon">○</Badge>
```

The row's accessible label or adjacent row copy includes `pending`. The pending
`Badge` applies to changed principal rows, group rows, branch-gate rows, and the
Rules tab principal selector when the selected principal has staged governance
changes.

Committed row pending `Badge` props are: none. Committed rows render no pending
`Badge` at all. Do not recolor the pending `Badge` to gray, do not render a
committed variant of the pending `Badge`, and do not use a `Badge` merely to show
`● committed`. If a wireframe or row label needs to communicate committed status,
use ordinary row text or the absence of the pending marker; the pending `Badge`
is absent.

### Page-level pending banner

`GovernancePendingBanner` at
`apps/desktop/src/components/governance/GovernancePendingBanner.svelte` is a
thin composition over `packages/ui/src/lib/components/InfoMessage.svelte`.
`GovernanceSettings.svelte` renders it above the `Tabs` `TabList`, after the
page heading/`SettingsSection` copy and before the tab strip.

The banner wraps `InfoMessage` with these exact props:

```svelte
<InfoMessage
	style="warning"
	outlined={true}
	primaryLabel="Commit changes"
	primaryIcon="arrow-right"
	primaryAction={commitGovernanceChanges}
>
```

The `title` snippet renders
`{pendingCount} pending governance change(s) - take effect once committed to the governance ref`.
This is the `N pending governance change(s)` title required by the wireframe,
with `N` replaced by the numeric `pendingCount`.
The count is a numeral interpolated from the CLIENT-ONLY Svelte pending store
owned by `GovernanceSettings.svelte`, not from a server load. The `content`
snippet may repeat the shorter helper copy
`Changes take effect once committed to the governance ref.` when the composition
needs secondary text.

`GovernanceSettings.svelte` guards the banner with the count:

```svelte
{#if pendingCount > 0}
	<GovernancePendingBanner {pendingCount} onCommit={commitGovernanceChanges} />
{/if}
```

When `pendingCount === 0`, the banner is not rendered. Do not render a zero-count
banner.

### State transition

The complete transition is default -> edit -> commit -> clean.

1. Default clean state: no row pending `Badge`; the page-level pending banner is
   hidden.
2. After an edit is saved to the working tree: each affected row receives the
   pending `Badge` (`style="warning"`, `kind="soft"`, `size="icon"` with `○`);
   `pendingCount` increments in the CLIENT-ONLY store; the warning
   `InfoMessage` banner appears above the `TabList`.
3. After the user activates `Commit changes`: the commit action commits the
   working-tree governance config files to the current
   workspace branch; the pending `Badge`s are removed; the banner is hidden; the
   effective set updates from the committed governance ref.
4. Clean reconciliation: if governance config files are clean versus `HEAD`, the
   banner is hidden regardless of stale UI state, and row pending `Badge`s are
   cleared.

### Cross-tab persistence

The pending store is owned by `GovernanceSettings.svelte`, the parent of the
shared `Tabs` shell. The pending row `Badge`s and the page-level `pendingCount`
therefore persist while switching between `Principals`, `Groups`,
`Branch Gates`, and `Rules`; tab content must not own or reset the pending store.

## Four-Tab State Matrix

| Wireframe region                 | Default or populated state                                                                                                                                                                                                                                                                                                           | Pending state                                                                                                                                                                                                                                                                                                                                                                                                                                                    | Read-only state                                                                                                                        | Denial state                                                                                              | Empty state                                                                   |
| -------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| Project Settings modal shell     | `SettingsModalLayout` at `apps/desktop/src/components/settings/SettingsModalLayout.svelte`; active sidebar entry is the `Permissions & Governance` page.                                                                                                                                                                             | Same shell; page-level pending banner appears above tabs.                                                                                                                                                                                                                                                                                                                                                                                                        | Same shell; page remains visible for admins but controls below become disabled or readonly.                                            | Same shell; denial banner appears above or inside the affected tab.                                       | Same shell; active tab panel renders that tab's empty state.                  |
| Governance page heading          | `GovernanceSettings` at `apps/desktop/src/components/settings/GovernanceSettings.svelte` using `SettingsSection` at `apps/desktop/src/components/shared/SettingsSection.svelte`; title text `Permissions & Governance`.                                                                                                              | Same heading with `GovernancePendingBanner` at `apps/desktop/src/components/governance/GovernancePendingBanner.svelte`.                                                                                                                                                                                                                                                                                                                                          | Same heading plus read-only `InfoMessage` info banner.                                                                                 | Same heading plus danger `InfoMessage`.                                                                   | Same heading; empty-state component appears in the active `TabContent`.       |
| Pending banner area              | Hidden when there are no pending governance changes.                                                                                                                                                                                                                                                                                 | `GovernancePendingBanner` wraps `InfoMessage` at `packages/ui/src/lib/components/InfoMessage.svelte` with `style="warning"`, `outlined=true`, `primaryLabel="Commit changes"`, `primaryIcon="arrow-right"`, title `{pendingCount} pending governance change(s) - take effect once committed to the governance ref`, and content `Changes take effect once committed to the governance ref.`; rendered above the `Tabs` `TabList` under `{#if pendingCount > 0}`. | Hidden while `isReadOnly=true`; the commit affordance is not rendered in read-only mode.                                               | Replaced or accompanied by danger `InfoMessage` for the denied action.                                    | Hidden.                                                                       |
| Tab strip                        | `Tabs` at `apps/desktop/src/components/shared/Tabs.svelte` with `defaultSelected="principals"`; `TabList`, `TabTrigger`, and `TabContent` paths listed in the tab IA contract.                                                                                                                                                       | Same tab strip; pending badges remain within tab content rows, not in tab labels.                                                                                                                                                                                                                                                                                                                                                                                | Same tab strip; triggers stay navigable unless the whole feature cannot render.                                                        | Same tab strip; the denied tab remains selected so the user sees the banner.                              | Same tab strip; selected tab shows `EmptyStatePlaceholder`.                   |
| Principals tab list header       | `PrincipalsList` at `apps/desktop/src/components/governance/PrincipalsList.svelte`; visual primitives are `SettingsSection` and `Button` at `packages/ui/src/lib/components/Button.svelte` with label `Add`.                                                                                                                         | Header remains; Add action may show pending affordance only after save.                                                                                                                                                                                                                                                                                                                                                                                          | Add `Button` disabled; rows below use disabled controls.                                                                               | Danger `InfoMessage` above list explains the rejected action.                                             | `EmptyStatePlaceholder` path and slots listed in the Empty States section.    |
| Principal row: committed actor   | `CardGroupItem` at `packages/ui/src/lib/components/cardGroup/CardGroupItem.svelte`; no pending `Badge`; row text such as `claude-agent`, `admin`, `eng`, `contents:rw`.                                                                                                                                                              | If the row has uncommitted edits, use the pending badge state in this row.                                                                                                                                                                                                                                                                                                                                                                                       | Row opens editor in read-only mode or exposes disabled controls only.                                                                  | If a row action is denied, show danger `InfoMessage` near the row or page action area.                    | Not rendered.                                                                 |
| Principal row: pending actor     | `CardGroupItem` plus `Badge style="warning" kind="soft" size="icon"` children `○`; pending permission copy such as `contents:read ○ pending` remains row text.                                                                                                                                                                       | Same pending badge remains until commit clears it.                                                                                                                                                                                                                                                                                                                                                                                                               | Pending badge may remain visible; editor controls disabled.                                                                            | Denial banner appears; do not flip the denied control visually.                                           | Not rendered.                                                                 |
| Principal row overflow           | `KebabButton` at `packages/ui/src/lib/components/KebabButton.svelte`.                                                                                                                                                                                                                                                                | Same overflow; destructive entries can be disabled while a write is pending.                                                                                                                                                                                                                                                                                                                                                                                     | Mutating menu actions are omitted; if the menu remains for non-mutating entries, mutating entries use `ContextMenuItem disabled=true`. | Danger `InfoMessage` after denied menu action.                                                            | Not rendered.                                                                 |
| Per-principal editor container   | `PrincipalEditor` at `apps/desktop/src/components/governance/PrincipalEditor.svelte`; follows `RuleEditor` at `apps/desktop/src/components/rules/RuleEditor.svelte` as the inline slide-in pattern.                                                                                                                                  | Save `Button` label `Save changes ○ pending`; pending badge uses `Badge style="warning" kind="soft" size="icon"`.                                                                                                                                                                                                                                                                                                                                                | Whole editor stays readable; `Segment`, `Toggle`, `TagInput`, and save action are disabled or readonly.                                | Denial banner with `InfoMessage style="danger"` remains visible and denied control does not change state. | Not rendered.                                                                 |
| Principal editor title and close | `PrincipalEditor` composition; close action is `Button` at `packages/ui/src/lib/components/Button.svelte` or icon button equivalent from the existing button component, visible label or accessible label `Close`.                                                                                                                   | Close remains available.                                                                                                                                                                                                                                                                                                                                                                                                                                         | Close remains available.                                                                                                               | Close remains available after denial.                                                                     | Not rendered.                                                                 |
| Role preset strip                | `SegmentControl` at `packages/ui/src/lib/components/segmentControl/SegmentControl.svelte` with `selected="write"` in the wireframe; each role is `Segment` at `packages/ui/src/lib/components/segmentControl/Segment.svelte` with ids `read`, `triage`, `write`, `maintain`, `admin`.                                                | Selecting a role marks the editor dirty; save button carries pending text.                                                                                                                                                                                                                                                                                                                                                                                       | Every `Segment` gets `disabled=true`.                                                                                                  | Denied self-escalation leaves the prior selected segment active and shows danger `InfoMessage`.           | Not rendered.                                                                 |
| Functional permission rows       | `PrincipalEditor` composition uses row text plus `Toggle` at `packages/ui/src/lib/components/Toggle.svelte`; `checked=true` for ON and `checked=false` for OFF.                                                                                                                                                                      | Dirty or saved pending permission row gets `Badge style="warning" kind="soft" size="icon"` children `○`.                                                                                                                                                                                                                                                                                                                                                         | `Toggle disabled=true` for every editable row.                                                                                         | Denied row's `Toggle` remains at the last committed value; show `InfoMessage style="danger"`.             | Not rendered.                                                                 |
| Inherited permission row         | `Toggle` at `packages/ui/src/lib/components/Toggle.svelte` with `disabled=true`; SOURCE column uses `Badge style='gray' kind='soft'` with label `[group: {groupName}]`; GRANT column shows `── inherited ──` in `var(--text-3)`; row background remains default `var(--bg-1)`.                                                       | Inherited state does not become editable; no pending `Badge` renders on inherited rows; pending state only applies to local own-grant edits.                                                                                                                                                                                                                                                                                                                     | Same disabled state.                                                                                                                   | Not applicable unless a group-removal action is denied; then use danger banner.                           | Not rendered.                                                                 |
| Editor inherited explainer       | `InfoMessage` at `packages/ui/src/lib/components/InfoMessage.svelte` with `style="info"`, `outlined=true`, title `Inherited permissions`, content `Inherited rows come from groups; remove a group to revoke.`                                                                                                                       | Same message.                                                                                                                                                                                                                                                                                                                                                                                                                                                    | Same message.                                                                                                                          | Can sit below denial banner if both are present.                                                          | Not rendered.                                                                 |
| Editor groups chips              | `TagInput` at `packages/ui/src/lib/components/TagInput.svelte` with `tags` labels `eng` and `platform`; `[+ Add to group]` uses `Select` at `packages/ui/src/lib/components/select/Select.svelte` and `SelectItem` at `packages/ui/src/lib/components/select/SelectItem.svelte`.                                                     | Chip additions/removals mark save button pending; remove creates a staged group removal that is batch-saved with `[Save changes]`.                                                                                                                                                                                                                                                                                                                               | `TagInput readonly=true`; add-group `Select disabled`.                                                                                 | Denied chip change is not applied and shows `InfoMessage style="danger"`.                                 | Not rendered.                                                                 |
| Editor footer buttons            | `Button` at `packages/ui/src/lib/components/Button.svelte`; labels `Cancel` and `Save changes ○ pending`.                                                                                                                                                                                                                            | Save button remains visible until the staged editor change is saved.                                                                                                                                                                                                                                                                                                                                                                                             | Save button disabled; Cancel stays available.                                                                                          | Save button stops loading and denial banner appears.                                                      | Not rendered.                                                                 |
| Groups tab header                | `GroupsList` at `apps/desktop/src/components/governance/GroupsList.svelte`; add action uses `Button` label `New group`.                                                                                                                                                                                                              | Header remains; pending group rows show pending badge.                                                                                                                                                                                                                                                                                                                                                                                                           | New group `Button` disabled.                                                                                                           | Danger `InfoMessage` above group list.                                                                    | `EmptyStatePlaceholder` path and slots listed in the Empty States section.    |
| Group expandable row             | `ExpandableSection` at `apps/desktop/src/components/shared/ExpandableSection.svelte`; `label="eng"` or group name; `summary` includes member count and `KebabButton`.                                                                                                                                                                | Summary includes `Badge style="warning" kind="soft" size="icon"` children `○` when group changes are pending.                                                                                                                                                                                                                                                                                                                                                    | Row expands but contained controls are disabled or readonly.                                                                           | Denial banner appears inside `GroupsList` or above affected section.                                      | Not rendered.                                                                 |
| Group granted permissions        | `Toggle` at `packages/ui/src/lib/components/Toggle.svelte` for each grant; optional grant labels use existing text styles.                                                                                                                                                                                                           | Changed grants show pending marker by the section or row.                                                                                                                                                                                                                                                                                                                                                                                                        | `Toggle disabled=true`.                                                                                                                | Denied grant toggle remains unchanged and danger banner appears.                                          | Not rendered.                                                                 |
| Group members                    | `TagInput` at `packages/ui/src/lib/components/TagInput.svelte` with member tags such as `claude-agent`, `codex-agent`, `agent:new`; `readonly=true` when view-only.                                                                                                                                                                  | Added or removed members contribute to section pending state.                                                                                                                                                                                                                                                                                                                                                                                                    | `TagInput readonly=true`.                                                                                                              | Denied member edit is not applied and danger banner appears.                                              | Not rendered.                                                                 |
| Group delete confirmation        | `Modal` at `packages/ui/src/lib/components/Modal.svelte`; destructive button uses `Button`; title copy `Remove group eng?`; body copy `N principals will lose inherited permissions.`                                                                                                                                                | Confirmation precedes the staged pending write.                                                                                                                                                                                                                                                                                                                                                                                                                  | Delete action disabled, so modal does not open.                                                                                        | If delete is denied, close modal and show danger `InfoMessage`.                                           | Not rendered.                                                                 |
| Branch Gates tab header          | `BranchGatesList` at `apps/desktop/src/components/governance/BranchGatesList.svelte`; add action uses `Button` label `Add`; helper copy says it reads governance gate configuration.                                                                                                                                                 | Header remains; pending branches show pending badge.                                                                                                                                                                                                                                                                                                                                                                                                             | Add `Button` disabled.                                                                                                                 | Danger `InfoMessage` above gate list.                                                                     | `EmptyStatePlaceholder` path and slots listed in the Empty States section.    |
| Branch gate row                  | `ExpandableSection` at `apps/desktop/src/components/shared/ExpandableSection.svelte`; `label="main"` or branch pattern; committed rows have no pending `Badge`.                                                                                                                                                                      | `Badge style="warning" kind="soft" size="icon"` children `○` plus pending row text.                                                                                                                                                                                                                                                                                                                                                                              | Row expands with controls disabled or readonly.                                                                                        | Denied gate edit does not flip controls and shows danger banner.                                          | Not rendered.                                                                 |
| Branch protected control         | `Toggle` at `packages/ui/src/lib/components/Toggle.svelte` with `checked=true` for protected ON.                                                                                                                                                                                                                                     | Turning OFF opens confirmation before pending write.                                                                                                                                                                                                                                                                                                                                                                                                             | `Toggle disabled=true`.                                                                                                                | Denied unprotect leaves `checked=true` and shows danger `InfoMessage`.                                    | Not rendered.                                                                 |
| Branch unprotect confirmation    | `Modal` at `packages/ui/src/lib/components/Modal.svelte`; title `Unprotect branch main?`; body `Merges will no longer require review.`                                                                                                                                                                                               | Confirmation precedes pending write.                                                                                                                                                                                                                                                                                                                                                                                                                             | Not available.                                                                                                                         | Denial closes or keeps modal state consistent and shows danger banner.                                    | Not rendered.                                                                 |
| Min approvals field              | `Textbox` at `packages/ui/src/lib/components/Textbox.svelte` with `type="number"` and value such as `2`.                                                                                                                                                                                                                             | Changed number contributes to pending badge.                                                                                                                                                                                                                                                                                                                                                                                                                     | `disabled=true` or `readonly=true`.                                                                                                    | Denied edit reverts visible value and shows danger banner.                                                | Not rendered.                                                                 |
| Distinct approver control        | `Toggle` at `packages/ui/src/lib/components/Toggle.svelte` with `checked=true` when required.                                                                                                                                                                                                                                        | Changed toggle contributes to pending badge.                                                                                                                                                                                                                                                                                                                                                                                                                     | `disabled=true`.                                                                                                                       | Denied edit leaves prior value visible and shows danger banner.                                           | Not rendered.                                                                 |
| Required approval groups         | `TagInput` at `packages/ui/src/lib/components/TagInput.svelte` for selected groups `eng` and `security`; `Select` at `packages/ui/src/lib/components/select/Select.svelte` for adding from defined group options.                                                                                                                    | Group selector changes contribute to pending badge.                                                                                                                                                                                                                                                                                                                                                                                                              | `TagInput readonly=true`; `Select` disabled by composition.                                                                            | Denied edit leaves prior group tags visible and shows danger banner.                                      | Not rendered.                                                                 |
| Rules tab principal selector     | `CardGroupRoot` and `CardGroupItem` at `packages/ui/src/lib/components/cardGroup/CardGroupRoot.svelte` and `packages/ui/src/lib/components/cardGroup/CardGroupItem.svelte`; committed principal rows have no pending `Badge`.                                                                                                        | Pending principals use `Badge style="warning" kind="soft" size="icon"` children `○`; selected row points to the rules panel.                                                                                                                                                                                                                                                                                                                                     | Selector remains readable; rule creation/edit actions disabled.                                                                        | Denial banner appears above rules panel.                                                                  | If no principals exist, use Rules tab empty state listed below.               |
| Rules tab rule panel             | `RulesList` at `apps/desktop/src/components/rules/RulesList.svelte` with optional `principalId`; existing `Rule`, `RuleEditor`, `RuleFiltersEditor`, and `NewRuleMenu` paths listed in inventory.                                                                                                                                    | Existing rules UI displays pending state via governance wrapper only; rules components remain visually unchanged.                                                                                                                                                                                                                                                                                                                                                | Rule create/edit/delete actions disabled by wrapper or readonly state.                                                                 | IPC or auth denial uses `InfoMessage style="danger"` with Retry when applicable.                          | If a selected principal has no rules, use Rules tab empty state listed below. |
| Governance render error fallback | Existing `ErrorBoundary` at `apps/desktop/src/components/shared/ErrorBoundary.svelte` wraps the `GovernanceSettings.svelte` mount point with `title="Governance settings failed to load"` and `compact=false`; the failed snippet renders the title and `error.message` sub-line when the thrown value is an `Error` with a message. | Same informational fallback; pending state is not shown while the governance mount point is replaced.                                                                                                                                                                                                                                                                                                                                                            | Same informational fallback.                                                                                                           | Not used for IPC or denied-write failures.                                                                | Not applicable.                                                               |

## PrincipalEditor Inherited-Vs-Own Row Contract

The `PrincipalEditor` FUNCTIONAL PERMISSIONS table is a two-column source/grant
contract rendered inside the editor's inline slide-in panel. The permission name
is the row label, the SOURCE column explains why the effective permission is
present, and the GRANT column shows the editable control only when the grant is
an own grant. Effective permission is own ∪ group; group-inherited grants are
visible in this table but are revoked through the Groups tab, not from the
permission row.

### Row Types

| Row type                           | SOURCE column                                                                                                           | GRANT column                                                                                                                                                                                                                                                 | Pending treatment                                                                                                                                               | Revocation path                                                                                                                                                                                                                                                |
| ---------------------------------- | ----------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Inherited group grant              | `Badge style='gray' kind='soft'` with label `[group: {groupName}]`; never plain `own grant`.                            | No editable control. The row's effective grant is represented by `Toggle disabled=true`; the displayed grant text is `── inherited ──` in `var(--text-3)`. The row background remains the default `var(--bg-1)` with no inherited-only background or border. | No pending `Badge` on inherited rows. Do not show the pending `Badge` merely because a group membership change is staged elsewhere.                             | The inherited grant cannot be revoked from `PrincipalEditor`; remove the principal from the group in the Groups tab first.                                                                                                                                     |
| Own-grant active                   | `own grant` in `var(--text-2)`.                                                                                         | `Toggle disabled=false checked=true`; this `Toggle` is the GRANT column control.                                                                                                                                                                             | When the own grant is toggled but not yet batch-saved, show the pending `Badge style='warning' kind='soft'` inline with adjacent `pending` copy.                | The enabled `Toggle` stages the local own-grant change; `[Save changes]` writes it.                                                                                                                                                                            |
| Own-grant inactive                 | `own grant` in `var(--text-2)`.                                                                                         | `Toggle disabled=false checked=false`; this `Toggle` is the GRANT column control.                                                                                                                                                                            | Same pending `Badge style='warning' kind='soft'` treatment only after an own-grant staged change.                                                               | The enabled `Toggle` stages the local own-grant change; `[Save changes]` writes it.                                                                                                                                                                            |
| Both own-grant and group-inherited | Inherited source takes precedence: `Badge style='gray' kind='soft'` with label `[group: {groupName}]`, not `own grant`. | Render as inherited and disabled: `Toggle disabled=true` and `── inherited ──` in `var(--text-3)`. The row background remains default `var(--bg-1)`.                                                                                                         | No pending `Badge`. A tooltip or muted sub-text may state that an own grant also exists, but it must not present a revocation control while inheritance exists. | The own grant cannot be revoked from `PrincipalEditor` while the inherited grant exists. Remove the principal from the group in the Groups tab first; after that removal is saved and effective, the row transitions to an own-grant row and becomes editable. |

Concrete example rows:

| Principal | Permission                                                           | SOURCE                                                                 | GRANT                                                             | Behavior                                                                                                                                                                          |
| --------- | -------------------------------------------------------------------- | ---------------------------------------------------------------------- | ----------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `alice`   | `contents:write`                                                     | `Badge style='gray' kind='soft'` label `[group: eng]`                  | `Toggle disabled=true`; text `── inherited ──` in `var(--text-3)` | Alice inherits the grant from `eng`; no pending `Badge`; revoke by removing Alice from `eng` in the Groups tab.                                                                   |
| `alice`   | `pull_requests:write`                                                | `own grant` in `var(--text-2)`                                         | `Toggle disabled=false checked=true`                              | Toggling stages an own-grant change and may show `Badge style='warning' kind='soft'` with `pending` copy.                                                                         |
| `alice`   | `reviews:write`                                                      | `own grant` in `var(--text-2)`                                         | `Toggle disabled=false checked=false`                             | Toggling stages an own-grant change and may show `Badge style='warning' kind='soft'` with `pending` copy.                                                                         |
| `alice`   | `contents:write` with explicit own grant and group `eng` inheritance | `Badge style='gray' kind='soft'` label `[group: eng]`, not `own grant` | `Toggle disabled=true`; text `── inherited ──` in `var(--text-3)` | This is the explicit "both" row: inherited source takes precedence, no pending `Badge`, own grant cannot be revoked from `PrincipalEditor`; Groups tab removal is required first. |

### SegmentControl Interaction

`SegmentControl` at
`packages/ui/src/lib/components/segmentControl/SegmentControl.svelte` presents
the role presets in this order: `read`, `triage`, `write`, `maintain`, `admin`.
Each role option is a `Segment` at
`packages/ui/src/lib/components/segmentControl/Segment.svelte`.

Selecting a preset through `onselect` updates local UI state only and performs no
immediate SDK write. The preset desugars to own-grant `Toggle` states: own-grant
rows in the preset set become `checked=true`, own-grant rows outside the preset
become `checked=false`, and each changed own-grant row can receive the pending
`Badge style='warning' kind='soft'` after it is staged. Inherited rows are never
touched by the preset: their `Toggle disabled=true` state stays disabled and
unchanged, their SOURCE column keeps the group `Badge style='gray' kind='soft'`,
their GRANT column keeps `── inherited ──` in `var(--text-3)`, and they never
receive a pending `Badge`.

Example: if Alice has an inherited `contents:write` grant from group `eng`, then
choosing the `read` preset does not remove that effective permission even though
the read preset omits it. The effective set remains own ∪ group, and the row
continues to show `[group: eng]` with `Toggle disabled=true`; the revoke path is
removing Alice from `eng` in the Groups tab.

### Groups Region

The editor's GROUPS region uses `TagInput` at
`packages/ui/src/lib/components/TagInput.svelte` for existing memberships. Each
membership tag uses the group name as its label, such as `eng` or `platform`.
Removing a tag with the component's remove affordance creates a staged group
removal in local editor state; it is batch-saved with `[Save changes]` alongside
permission changes and is not an immediate SDK write.

`[+ Add to group]` uses `Select` at
`packages/ui/src/lib/components/select/Select.svelte` with options from the
groups list, and each option renders through `SelectItem` at
`packages/ui/src/lib/components/select/SelectItem.svelte`. Do not build a custom
dropdown for this affordance. In read-only mode, render `TagInput readonly=true`
and the add-group `Select disabled`; membership tags remain visible and no
removal or add action is available.

## Empty States

Each empty state uses `EmptyStatePlaceholder` at
`packages/ui/src/lib/components/EmptyStatePlaceholder.svelte`. The listed text is
the slot contract for `title`, `caption`, and `actions`; actions render with
`Button` at `packages/ui/src/lib/components/Button.svelte`.
This section extends the Sprint 06a `DESIGN-MGMT-001` empty-state contract:
`Principals` and `Groups` keep the entries from `DESIGN-MGMT-001` AC-3/TC-3 and
are cited here by reference only. Sprint 06b adds the `Branch Gates` and `Rules`
contracts below.

| Tab          | Empty state component and slots                                                                                                                                                                                                                                                                                                                                                                                                                                            | Populated state component                                                                                                                                           |
| ------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Principals   | See `DESIGN-MGMT-001` AC-3/TC-3 for the existing `EmptyStatePlaceholder` `title`, `caption`, and `actions` slot contract. Do not duplicate or override it in Sprint 06b.                                                                                                                                                                                                                                                                                                   | `PrincipalsList` at `apps/desktop/src/components/governance/PrincipalsList.svelte` with rows built from `CardGroupRoot` and `CardGroupItem`.                        |
| Groups       | See `DESIGN-MGMT-001` AC-3/TC-3 for the existing `EmptyStatePlaceholder` `title`, `caption`, and `actions` slot contract. Do not duplicate or override it in Sprint 06b.                                                                                                                                                                                                                                                                                                   | `GroupsList` at `apps/desktop/src/components/governance/GroupsList.svelte` using `ExpandableSection` for each group.                                                |
| Branch Gates | When the SDK returns an empty gates array, render `EmptyStatePlaceholder` with `title` slot `No branch gates configured.`, `caption` slot `Branch gates control which branches require merge review and approval requirements before merging.`, and an `actions` slot containing a primary `Button` labeled `+ Add` that opens the add-gate flow. Pass `disabled=true` to this `Button` when `isReadOnly=true`.                                                            | `BranchGatesList` at `apps/desktop/src/components/governance/BranchGatesList.svelte` using `ExpandableSection` for each branch.                                     |
| Rules        | Two sub-cases: when no principal is selected, render `EmptyStatePlaceholder` with `title` slot `Select a principal to view their rules`, no primary action button, and no custom action slot; when a principal is selected but that principal has no rules, defer to the existing `RulesList` built-in empty state. The `RulesList` empty state is owned by `apps/desktop/src/components/rules/RulesList.svelte` and must not be overridden by the governance tab wrapper. | `RulesList` at `apps/desktop/src/components/rules/RulesList.svelte` with optional `principalId`, plus `Rule`, `RuleEditor`, `RuleFiltersEditor`, and `NewRuleMenu`. |

Empty-state read-only treatment is action-only. Every primary action `Button`
inside an empty state receives `disabled=true` when `isReadOnly=true`, derived in
`GovernanceSettings.svelte` per `DESIGN-MGMT-003`. The
`EmptyStatePlaceholder` remains visible in read-only mode; do not hide the
placeholder or replace it with a read-only-specific layout. Empty-state visual
attributes come from `EmptyStatePlaceholder`, `Button`, and their existing
component stylesheets; this section introduces no new color, spacing, or
typography tokens.

## Cross-Cutting Visual States

| State                            | Component path                                                                                                                  | Props and copy                                                                                                                                                                                                                                                                                                                                                         | Applies to                                                                                                                              |
| -------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| Pending banner                   | `packages/ui/src/lib/components/InfoMessage.svelte` via `apps/desktop/src/components/governance/GovernancePendingBanner.svelte` | `style="warning"`, `outlined=true`, `primaryLabel="Commit changes"`, `primaryIcon="arrow-right"`, `primaryAction` wired to the commit action, title `{pendingCount} pending governance change(s) - take effect once committed to the governance ref`, content `Changes take effect once committed to the governance ref.`; render only under `{#if pendingCount > 0}`. | Page-level area above the `Tabs` `TabList` whenever pending count is greater than zero.                                                 |
| Read-only banner                 | `packages/ui/src/lib/components/InfoMessage.svelte`                                                                             | `style="info"`, `outlined=true`, content `Read-only: administration:write is required to change governance settings`; omit `primaryLabel`, `secondaryLabel`, and `tertiaryLabel` so no action buttons render.                                                                                                                                                          | Page-level area above tab content when the viewer can navigate to the page but lacks `administration:write`.                            |
| Denial banner                    | `packages/ui/src/lib/components/InfoMessage.svelte`                                                                             | `style="danger"`, `outlined=true`, title `perm.denied — you cannot modify your own administration grants.`, content renders `{denial.remediation_hint}`; omit `primaryLabel`, `secondaryLabel`, and `tertiaryLabel` for the self-escalation denial path. Retry actions are reserved for retryable IPC/transport failures only.                                         | The shared `GovernanceSettings` banner slot, above the affected tab or editor after a refused write.                                    |
| Pending row badge                | `packages/ui/src/lib/components/Badge.svelte`                                                                                   | `style="warning"`, `kind="soft"`, `size="icon"`, children `○`; row label or adjacent row text includes `pending`.                                                                                                                                                                                                                                                      | Principal rows, group rows, branch gate rows, editor save summary.                                                                      |
| Committed row marker             | None                                                                                                                            | Committed rows render no pending `Badge`; absence of the pending marker is the committed signal. Do not recolor the pending `Badge` to gray.                                                                                                                                                                                                                           | Principal rows, rule principal selector rows, branch gate summaries.                                                                    |
| Inherited or unavailable control | `packages/ui/src/lib/components/Toggle.svelte`                                                                                  | `disabled=true`; `checked` mirrors the effective inherited value.                                                                                                                                                                                                                                                                                                      | Inherited permission rows and read-only rows.                                                                                           |
| Read-only chips                  | `packages/ui/src/lib/components/TagInput.svelte`                                                                                | `readonly=true`; tags remain visible and removal affordances are hidden by the component.                                                                                                                                                                                                                                                                              | Principal groups, group members, branch required groups.                                                                                |
| Read-only number field           | `packages/ui/src/lib/components/Textbox.svelte`                                                                                 | `type="number"`, `readonly=true` or `disabled=true`.                                                                                                                                                                                                                                                                                                                   | Branch gate minimum approvals.                                                                                                          |
| Destructive confirmation         | `packages/ui/src/lib/components/Modal.svelte`                                                                                   | Confirmation copy from the specific row action; action `Button` uses existing Button styling.                                                                                                                                                                                                                                                                          | Group delete and branch unprotect.                                                                                                      |
| Error boundary fallback          | `apps/desktop/src/components/shared/ErrorBoundary.svelte`                                                                       | Pass `title="Governance settings failed to load"` and `compact=false`; the existing failed snippet renders `error.message` as the sub-line when the thrown value is an `Error` instance with a message. Do not render a Retry button in this boundary fallback.                                                                                                        | Inside the `Permissions & Governance` section mount point only; the settings modal frame and other settings sections remain functional. |

## Denial And No-Flip Contract

This section adds the structured self-escalation denial layer on top of the
pending-state and read-only contracts above. It does not redefine pending
badges, the pending warning banner, read-only disabled controls, SDK call shape,
store internals, or Tauri command boundaries.

`GovernanceSettings.svelte` owns one banner slot for this surface. Render at
most one governance banner at a time. Retryable IPC or transport failure danger
banners take precedence per the contract below; otherwise the self-escalation
denial danger banner replaces the pending warning banner while denial is active.

The self-escalation denial banner uses `InfoMessage` at
`packages/ui/src/lib/components/InfoMessage.svelte` with these props and slots:

```svelte
<InfoMessage style="danger" outlined={true}>
	{#snippet title()}
		perm.denied — you cannot modify your own administration grants.
	{/snippet}
	{#snippet content()}
		{denial.remediation_hint}
	{/snippet}
</InfoMessage>
```

The `denial` payload is the structured refusal
`{code: "perm.denied", message: string, remediation_hint: string}`. The title
text is verbatim and includes the code plus message. The content sub-line
renders `remediation_hint` from the payload. Do not pass `primaryLabel`,
`secondaryLabel`, or `tertiaryLabel`: this banner is informational feedback, not
an actionable retry surface. While this denial banner is active, it occupies the
`GovernanceSettings` banner slot and replaces the pending warning banner rather
than stacking with it.

### Self-Escalation Toggle Rule

For an admin editing their own principal row, granting themselves
`administration:write` follows this visible sequence:

1. The admin toggles the `administration:write` `Toggle` ON for themselves; the
   `Toggle` moves to checked in local state for the attempted interaction.
2. The governed SDK call returns structured `perm.denied`.
3. The `Toggle` is synchronously reverted to unchecked, the prior state, as part
   of handling that denial response.
4. The danger `InfoMessage` appears in the `GovernanceSettings` banner slot.

The effective permission set is not updated. No `chipToast` is emitted for this
self-escalation denial path; the banner is the feedback. Transient write errors
from non-self-escalation operations keep their separate `chipToast` path and do
not show this denial banner.

### Denial State Machine

| State                           | Control state                                                                                                        | Banner type                                                                               | Banner text                                                                                                                                                                |
| ------------------------------- | -------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `idle`                          | `Toggle` reflects the current effective or staged value; normal controls follow the pending and read-only contracts. | No denial banner.                                                                         | Normal surface copy only.                                                                                                                                                  |
| `attempting-self-grant->denied` | Denied `Toggle` is reverted synchronously to the prior unchecked state; the effective permission set is not updated. | `InfoMessage` danger in the `GovernanceSettings` banner slot; pending banner is replaced. | `perm.denied — you cannot modify your own administration grants.` plus the structured denial `remediation_hint`; the structured payload also carries `code` and `message`. |
| `transient-error`               | Affected control is reverted to the prior state after a non-self-escalation write error.                             | `chipToast` only; no danger `InfoMessage` banner.                                         | Transient error toast copy for that write failure; no denial banner text.                                                                                                  |

## Error Boundary And IPC Failure Contract

Render/runtime failures and IPC/transport failures use different mechanisms. Do
not route an SDK rejection through the Svelte boundary, and do not put retry
behavior in the boundary fallback.

| Error category                  | Trigger                                                                                                          | Component treatment                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              | Recovery behavior                                                                                                      | Scope                                                                                                                                                                                                                                   |
| ------------------------------- | ---------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| render/runtime error (boundary) | A render path or child component under `GovernanceSettings.svelte` throws.                                       | The existing `apps/desktop/src/components/shared/ErrorBoundary.svelte` catches it through the Svelte boundary and renders its `failed` snippet with `title="Governance settings failed to load"` and `compact=false`. The fallback is title plus `error.message` only when the thrown value is an `Error` instance with a message. No Retry button renders in this fallback.                                                                                                                                                                                                                                     | Informational only; recovery is closing and reopening the settings modal or otherwise remounting the settings content. | The fallback replaces only the `Permissions & Governance` section mount point. `SettingsModalLayout`, the modal frame, sidebar, and other settings sections remain functional.                                                          |
| IPC/transport error (in-page)   | A Tauri SDK call rejects because of timeout, backend crash, connection loss, or unavailable structured response. | The failing governance surface catches the rejection in-page and shows `packages/ui/src/lib/components/InfoMessage.svelte` with `style="danger"` and `outlined=true`. The title renders the structured denial message when `{code, message}` is available from the SDK response; without a structured response it renders `Connection lost — governance service unavailable`. A Retry `packages/ui/src/lib/components/Button.svelte` is exposed through the `InfoMessage` primary action slot by setting `primaryLabel="Retry"` and `primaryAction` to the retry callback. The Svelte boundary is not triggered. | Activating Retry re-issues the same SDK call that failed; the component that owns that call owns the retry callback.   | The danger banner appears in the shared `GovernanceSettings` banner slot, the same slot used by the self-escalation denial banner, and takes highest priority over self-escalation danger, pending warning, and read-only info banners. |

### Persistent IPC Failure

On Retry success, hide the IPC-failure danger banner and resume the normal
governance surface state. If the successful response changes read-only or pending
state, reconcile those ordinary states from the response.

On Retry failure, keep the same danger `InfoMessage` visible. Update the title or
content if the new failure has fresher structured `{code, message}` data;
otherwise leave the existing message in place.

If the IPC failure persists, keep the governance surface mounted in a safe
read-only state equivalent to `isReadOnly=true`: all mutating controls are inert,
disabled, or readonly, and no additional write calls are attempted until Retry or
another explicit reload succeeds. Users can still inspect visible settings and
move between governance tabs. Persistent IPC failure must not unmount the surface
and must not trigger `ErrorBoundary`; it remains an in-page danger banner state.

## Read-Only State Contract

Read-only applies when the viewer can navigate to the Project Settings
`Permissions & Governance` page but the governed SDK `administration:write`
check is false. It is separate from the `adminOnly` settings-sidebar filter:
`apps/desktop/src/components/settings/SettingsModalLayout.svelte:53` filters
pages with `pages.filter((p) => !p.adminOnly || isAdmin)`, which hides the page
for non-admins so they cannot navigate to it. Read-only is the functional
permission state for a viewer who can navigate but cannot mutate governance
settings. These layers are independent and must not be conflated.

`apps/desktop/src/components/settings/GovernanceSettings.svelte` derives one
boolean, `isReadOnly`, from the `administration:write` check in the governed SDK
and passes it as a prop to `PrincipalsList`, `PrincipalEditor`, `GroupsList`, and
`BranchGatesList`. The Rules tab wrapper or `RulesList` also consumes the same
prop. Child components consume the prop; they do not re-derive the permission.
The same contract covers `Principals`, `Groups`, `Branch Gates`, and `Rules`:
rule create, edit, delete, and save affordances are disabled by the governance
wrapper when `isReadOnly=true`.

The banner slot is single-purpose in read-only mode. Render `InfoMessage` from
`packages/ui/src/lib/components/InfoMessage.svelte` with `style="info"`,
`outlined=true`, and content
`Read-only: administration:write is required to change governance settings`.
Do not pass `primaryLabel`, `secondaryLabel`, or `tertiaryLabel`; the read-only
banner has no action buttons. When `isReadOnly=true`,
`GovernancePendingBanner` is hidden even if there are pending changes, so the
commit affordance is unavailable rather than disabled in-place.

| Control family                                        | Existing prop or component source                                                                                                                                 | Read-only treatment                                                                                                                                             |
| ----------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Permission and gate toggles                           | `packages/ui/src/lib/components/Toggle.svelte` exposes `disabled`; its `:disabled` CSS provides the opacity and pointer-event treatment.                          | Pass `disabled=true`; do not wrap the tab in an extra grey overlay or add a second opacity treatment.                                                           |
| Group, member, and required-approval chips            | `packages/ui/src/lib/components/TagInput.svelte` exposes `readonly`; removal buttons are hidden when `readonly=true`.                                             | Pass `readonly=true` so tags remain readable and chip removal/addition is inert.                                                                                |
| Add, create, save, delete, and other mutating actions | `packages/ui/src/lib/components/Button.svelte` exposes `disabled`; `Button` also disables while `loading`.                                                        | Pass `disabled=true` for visible mutating buttons. Keep non-mutating close/cancel/navigation buttons available.                                                 |
| Role preset strip                                     | `packages/ui/src/lib/components/segmentControl/Segment.svelte` exposes `disabled`; `SegmentControl.svelte` coordinates selection through child `Segment` buttons. | Pass `disabled=true` to every role `Segment` so the `SegmentControl` is non-interactive while preserving the selected role.                                     |
| Row overflow actions                                  | `packages/ui/src/lib/components/KebabButton.svelte` opens the menu; `packages/ui/src/lib/components/ContextMenuItem.svelte` exposes `disabled`.                   | Prefer omitting mutating context actions from the `KebabButton` menu. If visible for context, render those mutating entries as `ContextMenuItem disabled=true`. |
| Branch gate number fields                             | `packages/ui/src/lib/components/Textbox.svelte` exposes `disabled` and `readonly`.                                                                                | Use `readonly=true` when the value should remain selectable/readable, or `disabled=true` when the field is part of a fully disabled mutating form row.          |

## Token And Styling Contract

- This annotation introduces no CSS variables, color literals, spacing values, or
  typography rules.
- Visual color, spacing, radius, type, hover, and focus treatment come from the
  component stylesheets listed above.
- Any use-site values are component props or slot content only.
- No governance-prefixed or management-prefixed CSS custom properties are part
  of this design.

## Sprint 07 LPR-014 — Principal `kind` Badge And Editor Selector

This section extends the Sprint 06a/Sprint 07 governance contracts for the
additive `kind` descriptor on `PrincipalsListEntry` (LPR-005 / LPR-014). `kind`
is a descriptor only: it is not consulted by any authorization decision, not
part of the enforcement configuration map, and not read by any gate predicate.
The UI representation is informational only.

### Badge Component And Variants

The kind read display uses the existing `Badge` component from
`packages/ui/src/lib/components/Badge.svelte` in neutral/secondary styling.
There is exactly one rendered variant in the principal row:

| `kind` value         | Row badge                                                                                                                                                                                                           | Reason                                                                                                                               |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `'agent'`            | `<Badge style="gray" kind="soft" size="tag" tooltip="...descriptor-only disclosure...">agent</Badge>` rendered beside the `<strong>{principalId}</strong>` in `PrincipalsList.svelte` after the pending `○` marker. | The agent value is the only one that needs a visible marker — it is what enables the agent-authored tag derivation.                  |
| `'human'`            | No badge.                                                                                                                                                                                                           | Peer descriptor; absence is the conservative default-human presentation.                                                             |
| absent / `undefined` | No badge.                                                                                                                                                                                                           | Conservative default-human posture (LPR-005 AC-4): absence means human, never agent. The badge must NOT be blank; it must be absent. |

Neutral/secondary styling is mandatory: `style="gray"`, `kind="soft"`. Do not
use `style="safe"`, `style="warning"`, `style="danger"`, `style="pop"`, or
`style="purple"` for the kind badge. 'Agent' and 'Human' are peer-level
descriptors, not statuses — color that implies trust/restriction is wrong.

### Non-Enforcement Disclosure

The kind `Badge` carries a `tooltip` prop with the verbatim text:

> This label identifies the principal as an agent or human for tagging purposes.
> It does not change any permission grant or gate decision.

In edit mode, the kind `SegmentControl` has a caption immediately below it
rendered as a `<p class="principal-editor__kind-caption">` element carrying the
same disclosure text. The caption uses `data-testid="principal-editor-kind-caption"`
so the disclosure is queryable in component tests. The disclosure must appear in
both read (badge tooltip) and edit (selector caption) modes because an operator
may only see one of the two during a session.

### Display-Only Contract

The agent `Badge` is decorative only. It MUST NOT:

- have `role="button"` (the `Badge` component renders `role="presentation"` when
  no `onclick` is passed — keep it that way),
- carry an `aria-pressed` attribute,
- attach an `onclick`/`onkeydown` handler that mutates state or fires an SDK
  write,
- be described in copy or tooltip as granting, denying, or otherwise changing
  authorization outcomes.

Clicking the badge may bubble up to the row's expand/collapse handler (the row's
existing behavior is unchanged) but MUST NOT fire any `principalKindUpdate` SDK
write. AC-2 verifies this with a spy.

### Edit Selector

The kind edit control is a two-option `SegmentControl` placed in the
`PrincipalEditor` form, immediately after the Preset `SegmentControl` and before
the Permissions table. It uses the same `SegmentControl` + `SegmentControl.Item`
components already imported by the editor for the role preset strip:

```svelte
<div class="principal-editor__section">
	<span class="principal-editor__label">Kind</span>
	<SegmentControl
		selected={stagedKind}
		onselect={(id) => setKind(id === "agent" ? "agent" : "human")}
	>
		<SegmentControl.Item
			id="human"
			testId="principal-editor-kind-human"
			disabled={controlsDisabled}
		>
			Human
		</SegmentControl.Item>
		<SegmentControl.Item
			id="agent"
			testId="principal-editor-kind-agent"
			disabled={controlsDisabled}
		>
			Agent
		</SegmentControl.Item>
	</SegmentControl>
	<p class="principal-editor__kind-caption" data-testid="principal-editor-kind-caption">
		This label identifies the principal as an agent or human for tagging purposes. It does not
		change any permission grant or gate decision.
	</p>
</div>
```

The selector offers exactly two options. Selecting one stages a local kind change
(`stagedKind`) that is committed on the existing `Save changes` button click — the
same write path that saves other principal fields (permissions, groups). The
write goes through the governance IPC path (administration:write-gated — the same
route as `permGrant`/`permRevoke` in `PrincipalEditorService`), NOT the
project-settings path.

### Default Selector Value

When the principal's SDK entry has no `kind` field (`undefined`), the selector
defaults to `'human'` (the conservative default-human posture per LPR-005 AC-4).
The default is explicit: `kind = "human"` as the prop default, and
`initialKind = kind === "agent" || kind === "human" ? kind : "human"` as the
resolved initial value. The selector must NEVER default to `'agent'`.

### Pending After Kind Write

After a successful kind write, the principal's row gains the existing pending
`Badge` pattern — exactly the same shape as the principals-list-pending-\* badge
at `PrincipalsList.svelte` lines 178-187:

```svelte
<Badge
	testId={`principals-list-pending-${slug(principal.principalId)}`}
	style="warning"
	kind="soft"
	size="icon"
>
	○
</Badge>
```

No new pending shape is introduced for kind writes. The kind-write pending
indicator is the existing pending indicator. `PrincipalsList.svelte` marks the
just-saved principal's `pending` field to `true` in local state on
`refreshAfterSave` and invokes the optional `onPrincipalPending(principalId)`
callback so the parent (`GovernanceSettings`) can increment the governance
pending-banner count via the existing `pendingStore`.

### Read-Only Treatment

When `isReadOnly=true`, both kind selector segments receive `disabled=true` via
the existing `controlsDisabled` derived flag. No `principalKindUpdate` SDK write
fires on any interaction (force-click included). The agent `Badge` continues to
render in the row when `kind='agent'` — read-only does not hide the descriptor.

### Integration With Existing Edit Mode

The kind `SegmentControl` is rendered alongside the existing editor sections
(Preset, Permissions, Groups). It does not introduce a new edit mode or a
separate inline-edit toggle. The selector is visible whenever the
`PrincipalEditor` is open (which is the existing principal row expand/collapse
behavior). In read mode (editor closed), only the row `Badge` is shown; in edit
mode (editor open), both the row `Badge` and the editor `SegmentControl` are
visible.

### Token And Styling Compliance

This section introduces no new CSS variables, color literals, spacing values, or
typography rules. The kind `Badge` uses `style="gray"` and `kind="soft"` —
existing component variants. The kind caption uses `var(--clr-text-2)` and
`var(--font-size-sm, 0.85rem)` — existing tokens. The kind `SegmentControl` and
`SegmentControl.Item` components use their own stylesheets. No `var(--kind-*)`
or `var(--agent-*)` tokens are introduced.
