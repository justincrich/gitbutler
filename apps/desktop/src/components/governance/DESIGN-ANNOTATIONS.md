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

| Region family | Component | Source path | Required props or state |
|---|---|---|---|
| Settings sidebar and page scaffold | `SettingsModalLayout` | `apps/desktop/src/components/settings/SettingsModalLayout.svelte` | Existing settings shell; governance page appears through the existing admin-only page filter. |
| Settings page host | `ProjectSettingsModalContent` | `apps/desktop/src/components/settings/ProjectSettingsModalContent.svelte` | Existing host branch for `GovernanceSettings.svelte`. |
| Governance page composition | `GovernanceSettings` | `apps/desktop/src/components/settings/GovernanceSettings.svelte` | Planned thin composition from the Net-new table; hosts `SettingsSection`, pending banner, tab IA, and per-tab content. |
| Page section header and copy | `SettingsSection` | `apps/desktop/src/components/shared/SettingsSection.svelte` | `title`, `description`, `top`, `gap` use the component stylesheet. |
| Secondary tab shell | `Tabs` | `apps/desktop/src/components/shared/Tabs.svelte` | `defaultSelected="principals"`. |
| Secondary tab list | `TabList` | `apps/desktop/src/components/shared/TabList.svelte` | The parent tab list carries `aria-label="Governance configuration tabs"`. |
| Secondary tab trigger | `TabTrigger` | `apps/desktop/src/components/shared/TabTrigger.svelte` | `value` is one of `principals`, `groups`, `branch-gates`, `rules`; `disabled` only when the whole surface is read-only unavailable. |
| Secondary tab panel | `TabContent` | `apps/desktop/src/components/shared/TabContent.svelte` | `value` matches the tab id. |
| Pending banner composition | `GovernancePendingBanner` | `apps/desktop/src/components/governance/GovernancePendingBanner.svelte` | Planned thin composition from the Net-new table; wraps `InfoMessage` warning state. |
| Banner feedback | `InfoMessage` | `packages/ui/src/lib/components/InfoMessage.svelte` | `style="warning"`, `style="info"`, or `style="danger"`; `outlined=true`; `primaryLabel` and `primaryIcon` when an action is present. |
| Status and role marker | `Badge` | `packages/ui/src/lib/components/Badge.svelte` | Pending: `style="warning"` `kind="soft"` with `○`; committed: `style="gray"` `kind="soft"` with `●`; role/status labels use component defaults unless a tab-specific state says otherwise. |
| Principal rows | `PrincipalsList` | `apps/desktop/src/components/governance/PrincipalsList.svelte` | Planned thin composition from the Net-new table; row surface uses `CardGroupRoot.svelte` and `CardGroupItem.svelte`. |
| Row group container | `CardGroupRoot` | `packages/ui/src/lib/components/cardGroup/CardGroupRoot.svelte` | Groups principal rows in the list body. |
| Row item | `CardGroupItem` | `packages/ui/src/lib/components/cardGroup/CardGroupItem.svelte` | Single principal row, group row summary, or status row as needed by the composition. |
| Row overflow actions | `KebabButton` | `packages/ui/src/lib/components/KebabButton.svelte` | Wireframe `...` overflow affordance. |
| Add, create, save, commit, cancel actions | `Button` | `packages/ui/src/lib/components/Button.svelte` | Use component variants already exposed by the Button component; no new button styling. |
| Principal editor | `PrincipalEditor` | `apps/desktop/src/components/governance/PrincipalEditor.svelte` | Planned thin composition from the Net-new table; inline editor mirrors `RuleEditor.svelte` spacing and slide-in placement. |
| Inline editor pattern | `RuleEditor` | `apps/desktop/src/components/rules/RuleEditor.svelte` | Pattern source only; governance editor keeps the same inline stacked editor treatment. |
| Role preset strip | `SegmentControl` | `packages/ui/src/lib/components/segmentControl/SegmentControl.svelte` | `selected="write"` for the shown wireframe; role segments in order `read`, `triage`, `write`, `maintain`, `admin`. |
| Role preset option | `Segment` | `packages/ui/src/lib/components/segmentControl/Segment.svelte` | `id` is the role id; `disabled=true` in read-only state. |
| Permission toggle | `Toggle` | `packages/ui/src/lib/components/Toggle.svelte` | `checked=true` for ON, `checked=false` for OFF, `disabled=true` for inherited rows and read-only state. |
| Group and member chips | `TagInput` | `packages/ui/src/lib/components/TagInput.svelte` | `tags` contain group/member labels; `readonly=true` in read-only state; `disabled=true` when actions are unavailable. |
| Groups tab sections | `GroupsList` | `apps/desktop/src/components/governance/GroupsList.svelte` | Planned thin composition from the Net-new table; each group uses `ExpandableSection`. |
| Branch gate sections | `BranchGatesList` | `apps/desktop/src/components/governance/BranchGatesList.svelte` | Planned thin composition from the Net-new table; each branch gate uses `ExpandableSection`. |
| Expandable rows | `ExpandableSection` | `apps/desktop/src/components/shared/ExpandableSection.svelte` | `label`, optional `summary`, `expanded`, `content`; group and branch sections use this directly. |
| Destructive confirmation | `Modal` | `packages/ui/src/lib/components/Modal.svelte` | Group delete and branch unprotect confirmation dialog. |
| Min approvals input | `Textbox` | `packages/ui/src/lib/components/Textbox.svelte` | `type="number"`; `disabled=true` or `readonly=true` in read-only state. |
| Required group selector | `Select` | `packages/ui/src/lib/components/select/Select.svelte` | Used for dropdown group selection when the design needs a single picker. |
| Empty states | `EmptyStatePlaceholder` | `packages/ui/src/lib/components/EmptyStatePlaceholder.svelte` | `title`, `caption`, and `actions` snippets exactly as listed below. |
| Rules tab list | `RulesList` | `apps/desktop/src/components/rules/RulesList.svelte` | Existing rules UI; Sprint implementation extends with optional `principalId` prop. |
| Rule row | `Rule` | `apps/desktop/src/components/rules/Rule.svelte` | Existing rule row unchanged. |
| Rule editor | `RuleEditor` | `apps/desktop/src/components/rules/RuleEditor.svelte` | Existing editor unchanged in the per-principal context. |
| Rule filters | `RuleFiltersEditor` | `apps/desktop/src/components/rules/RuleFiltersEditor.svelte` | Existing filters editor unchanged. |
| New rule menu | `NewRuleMenu` | `apps/desktop/src/components/rules/NewRuleMenu.svelte` | Existing menu unchanged. |
| Error boundary | `GovernanceErrorBoundary` | `apps/desktop/src/components/governance/GovernanceErrorBoundary.svelte` | Planned thin composition from the Net-new table; fallback uses `InfoMessage style="danger"`. |

## Tab IA Contract

The tab strip is one shared tab set, not four independent screens.

| Item | Component path | Props and content |
|---|---|---|
| Tab wrapper | `apps/desktop/src/components/shared/Tabs.svelte` | `defaultSelected="principals"`. |
| Tab list | `apps/desktop/src/components/shared/TabList.svelte` | Add `aria-label="Governance configuration tabs"` at use site. |
| Tab 1 trigger | `apps/desktop/src/components/shared/TabTrigger.svelte` | `value="principals"`; visible label `Principals`. |
| Tab 2 trigger | `apps/desktop/src/components/shared/TabTrigger.svelte` | `value="groups"`; visible label `Groups`. |
| Tab 3 trigger | `apps/desktop/src/components/shared/TabTrigger.svelte` | `value="branch-gates"`; visible label `Branch Gates`. |
| Tab 4 trigger | `apps/desktop/src/components/shared/TabTrigger.svelte` | `value="rules"`; visible label `Rules`. |
| Tab 1 content | `apps/desktop/src/components/shared/TabContent.svelte` | `value="principals"` renders `PrincipalsList`. |
| Tab 2 content | `apps/desktop/src/components/shared/TabContent.svelte` | `value="groups"` renders `GroupsList`. |
| Tab 3 content | `apps/desktop/src/components/shared/TabContent.svelte` | `value="branch-gates"` renders `BranchGatesList`. |
| Tab 4 content | `apps/desktop/src/components/shared/TabContent.svelte` | `value="rules"` renders existing rules components scoped by principal. |

Tab order is fixed: `principals`, `groups`, `branch-gates`, `rules`.

## Four-Tab State Matrix

| Wireframe region | Default or populated state | Pending state | Read-only state | Denial state | Empty state |
|---|---|---|---|---|---|
| Project Settings modal shell | `SettingsModalLayout` at `apps/desktop/src/components/settings/SettingsModalLayout.svelte`; active sidebar entry is the `Permissions & Governance` page. | Same shell; page-level pending banner appears above tabs. | Same shell; page remains visible for admins but controls below become disabled or readonly. | Same shell; denial banner appears above or inside the affected tab. | Same shell; active tab panel renders that tab's empty state. |
| Governance page heading | `GovernanceSettings` at `apps/desktop/src/components/settings/GovernanceSettings.svelte` using `SettingsSection` at `apps/desktop/src/components/shared/SettingsSection.svelte`; title text `Permissions & Governance`. | Same heading with `GovernancePendingBanner` at `apps/desktop/src/components/governance/GovernancePendingBanner.svelte`. | Same heading plus read-only `InfoMessage` info banner. | Same heading plus danger `InfoMessage`. | Same heading; empty-state component appears in the active `TabContent`. |
| Pending banner area | Hidden when there are no pending governance changes. | `GovernancePendingBanner` wraps `InfoMessage` at `packages/ui/src/lib/components/InfoMessage.svelte` with `style="warning"`, `outlined=true`, `primaryLabel="Commit changes"`, `primaryIcon="arrow-right"`, title `4 pending governance changes`, and content `Changes take effect once committed to the governance ref.` | Hidden unless pending changes are visible but not actionable; if visible, commit `Button` is disabled by the composition. | Replaced or accompanied by danger `InfoMessage` for the denied action. | Hidden. |
| Tab strip | `Tabs` at `apps/desktop/src/components/shared/Tabs.svelte` with `defaultSelected="principals"`; `TabList`, `TabTrigger`, and `TabContent` paths listed in the tab IA contract. | Same tab strip; pending badges remain within tab content rows, not in tab labels. | Same tab strip; triggers stay navigable unless the whole feature cannot render. | Same tab strip; the denied tab remains selected so the user sees the banner. | Same tab strip; selected tab shows `EmptyStatePlaceholder`. |
| Principals tab list header | `PrincipalsList` at `apps/desktop/src/components/governance/PrincipalsList.svelte`; visual primitives are `SettingsSection` and `Button` at `packages/ui/src/lib/components/Button.svelte` with label `Add`. | Header remains; Add action may show pending affordance only after save. | Add `Button` disabled; rows below use disabled controls. | Danger `InfoMessage` above list explains the rejected action. | `EmptyStatePlaceholder` path and slots listed in the Empty States section. |
| Principal row: committed actor | `CardGroupItem` at `packages/ui/src/lib/components/cardGroup/CardGroupItem.svelte`; status `Badge` at `packages/ui/src/lib/components/Badge.svelte` with `style="gray"` `kind="soft"` children `●`; row text such as `claude-agent`, `admin`, `eng`, `contents:rw`. | If the row has uncommitted edits, use the pending badge state in this row. | Row opens editor in read-only mode or exposes disabled controls only. | If a row action is denied, show danger `InfoMessage` near the row or page action area. | Not rendered. |
| Principal row: pending actor | `CardGroupItem` plus `Badge style="warning" kind="soft"` children `○`; pending permission copy such as `contents:read ○ pending` remains row text. | Same pending badge remains until commit clears it. | Pending badge may remain visible; editor controls disabled. | Denial banner appears; do not flip the denied control visually. | Not rendered. |
| Principal row overflow | `KebabButton` at `packages/ui/src/lib/components/KebabButton.svelte`. | Same overflow; destructive entries can be disabled while a write is pending. | `KebabButton` disabled or menu actions disabled by the composition. | Danger `InfoMessage` after denied menu action. | Not rendered. |
| Per-principal editor container | `PrincipalEditor` at `apps/desktop/src/components/governance/PrincipalEditor.svelte`; follows `RuleEditor` at `apps/desktop/src/components/rules/RuleEditor.svelte` as the inline slide-in pattern. | Save `Button` label `Save changes ○ pending`; pending badge uses `Badge style="warning" kind="soft"`. | Whole editor stays readable; `Segment`, `Toggle`, `TagInput`, and save action are disabled or readonly. | Denial banner with `InfoMessage style="danger"` remains visible and denied control does not change state. | Not rendered. |
| Principal editor title and close | `PrincipalEditor` composition; close action is `Button` at `packages/ui/src/lib/components/Button.svelte` or icon button equivalent from the existing button component, visible label or accessible label `Close`. | Close remains available. | Close remains available. | Close remains available after denial. | Not rendered. |
| Role preset strip | `SegmentControl` at `packages/ui/src/lib/components/segmentControl/SegmentControl.svelte` with `selected="write"` in the wireframe; each role is `Segment` at `packages/ui/src/lib/components/segmentControl/Segment.svelte` with ids `read`, `triage`, `write`, `maintain`, `admin`. | Selecting a role marks the editor dirty; save button carries pending text. | Every `Segment` gets `disabled=true`. | Denied self-escalation leaves the prior selected segment active and shows danger `InfoMessage`. | Not rendered. |
| Functional permission rows | `PrincipalEditor` composition uses row text plus `Toggle` at `packages/ui/src/lib/components/Toggle.svelte`; `checked=true` for ON and `checked=false` for OFF. | Dirty or saved pending permission row gets `Badge style="warning" kind="soft"` children `○`. | `Toggle disabled=true` for every editable row. | Denied row's `Toggle` remains at the last committed value; show `InfoMessage style="danger"`. | Not rendered. |
| Inherited permission row | `Toggle` at `packages/ui/src/lib/components/Toggle.svelte` with `disabled=true`; source label text `[group: eng]`; grant text `inherited`. | Inherited state does not become editable; pending state only applies to local own-grant edits. | Same disabled state. | Not applicable unless a group-removal action is denied; then use danger banner. | Not rendered. |
| Editor inherited explainer | `InfoMessage` at `packages/ui/src/lib/components/InfoMessage.svelte` with `style="info"`, `outlined=true`, title `Inherited permissions`, content `Inherited rows come from groups; remove a group to revoke.` | Same message. | Same message. | Can sit below denial banner if both are present. | Not rendered. |
| Editor groups chips | `TagInput` at `packages/ui/src/lib/components/TagInput.svelte` with `tags` labels `eng` and `platform`; add-group affordance uses `Button` or `Select` at `packages/ui/src/lib/components/select/Select.svelte`. | Chip additions/removals mark save button pending. | `TagInput readonly=true`; add-group `Button` disabled. | Denied chip change is not applied and shows `InfoMessage style="danger"`. | Not rendered. |
| Editor footer buttons | `Button` at `packages/ui/src/lib/components/Button.svelte`; labels `Cancel` and `Save changes ○ pending`. | Save button remains visible until the staged editor change is saved. | Save button disabled; Cancel stays available. | Save button stops loading and denial banner appears. | Not rendered. |
| Groups tab header | `GroupsList` at `apps/desktop/src/components/governance/GroupsList.svelte`; add action uses `Button` label `New group`. | Header remains; pending group rows show pending badge. | New group `Button` disabled. | Danger `InfoMessage` above group list. | `EmptyStatePlaceholder` path and slots listed in the Empty States section. |
| Group expandable row | `ExpandableSection` at `apps/desktop/src/components/shared/ExpandableSection.svelte`; `label="eng"` or group name; `summary` includes member count and `KebabButton`. | Summary includes `Badge style="warning" kind="soft"` children `○` when group changes are pending. | Row expands but contained controls are disabled or readonly. | Denial banner appears inside `GroupsList` or above affected section. | Not rendered. |
| Group granted permissions | `Toggle` at `packages/ui/src/lib/components/Toggle.svelte` for each grant; optional grant labels use existing text styles. | Changed grants show pending marker by the section or row. | `Toggle disabled=true`. | Denied grant toggle remains unchanged and danger banner appears. | Not rendered. |
| Group members | `TagInput` at `packages/ui/src/lib/components/TagInput.svelte` with member tags such as `claude-agent`, `codex-agent`, `agent:new`; `readonly=true` when view-only. | Added or removed members contribute to section pending state. | `TagInput readonly=true`. | Denied member edit is not applied and danger banner appears. | Not rendered. |
| Group delete confirmation | `Modal` at `packages/ui/src/lib/components/Modal.svelte`; destructive button uses `Button`; title copy `Remove group eng?`; body copy `N principals will lose inherited permissions.` | Confirmation precedes the staged pending write. | Delete action disabled, so modal does not open. | If delete is denied, close modal and show danger `InfoMessage`. | Not rendered. |
| Branch Gates tab header | `BranchGatesList` at `apps/desktop/src/components/governance/BranchGatesList.svelte`; add action uses `Button` label `Add`; helper copy `reads .gitbutler/gates.toml`. | Header remains; pending branches show pending badge. | Add `Button` disabled. | Danger `InfoMessage` above gate list. | `EmptyStatePlaceholder` path and slots listed in the Empty States section. |
| Branch gate row | `ExpandableSection` at `apps/desktop/src/components/shared/ExpandableSection.svelte`; `label="main"` or branch pattern; `summary` includes status badge. | `Badge style="warning" kind="soft"` children `○ pending`. | Row expands with controls disabled or readonly. | Denied gate edit does not flip controls and shows danger banner. | Not rendered. |
| Branch protected control | `Toggle` at `packages/ui/src/lib/components/Toggle.svelte` with `checked=true` for protected ON. | Turning OFF opens confirmation before pending write. | `Toggle disabled=true`. | Denied unprotect leaves `checked=true` and shows danger `InfoMessage`. | Not rendered. |
| Branch unprotect confirmation | `Modal` at `packages/ui/src/lib/components/Modal.svelte`; title `Unprotect branch main?`; body `Merges will no longer require review.` | Confirmation precedes pending write. | Not available. | Denial closes or keeps modal state consistent and shows danger banner. | Not rendered. |
| Min approvals field | `Textbox` at `packages/ui/src/lib/components/Textbox.svelte` with `type="number"` and value such as `2`. | Changed number contributes to pending badge. | `disabled=true` or `readonly=true`. | Denied edit reverts visible value and shows danger banner. | Not rendered. |
| Distinct approver control | `Toggle` at `packages/ui/src/lib/components/Toggle.svelte` with `checked=true` when required. | Changed toggle contributes to pending badge. | `disabled=true`. | Denied edit leaves prior value visible and shows danger banner. | Not rendered. |
| Required approval groups | `TagInput` at `packages/ui/src/lib/components/TagInput.svelte` for selected groups `eng` and `security`; `Select` at `packages/ui/src/lib/components/select/Select.svelte` for adding from defined group options. | Group selector changes contribute to pending badge. | `TagInput readonly=true`; `Select` disabled by composition. | Denied edit leaves prior group tags visible and shows danger banner. | Not rendered. |
| Rules tab principal selector | `CardGroupRoot` and `CardGroupItem` at `packages/ui/src/lib/components/cardGroup/CardGroupRoot.svelte` and `packages/ui/src/lib/components/cardGroup/CardGroupItem.svelte`; each principal row uses `Badge` committed or pending marker. | Pending principals use `Badge style="warning" kind="soft"` children `○`; selected row points to the rules panel. | Selector remains readable; rule creation/edit actions disabled. | Denial banner appears above rules panel. | If no principals exist, use Rules tab empty state listed below. |
| Rules tab rule panel | `RulesList` at `apps/desktop/src/components/rules/RulesList.svelte` with optional `principalId`; existing `Rule`, `RuleEditor`, `RuleFiltersEditor`, and `NewRuleMenu` paths listed in inventory. | Existing rules UI displays pending state via governance wrapper only; rules components remain visually unchanged. | Rule create/edit/delete actions disabled by wrapper or readonly state. | IPC or auth denial uses `InfoMessage style="danger"` with Retry when applicable. | If a selected principal has no rules, use Rules tab empty state listed below. |
| Governance render error fallback | `GovernanceErrorBoundary` at `apps/desktop/src/components/governance/GovernanceErrorBoundary.svelte`; fallback visual uses `InfoMessage` at `packages/ui/src/lib/components/InfoMessage.svelte` with `style="danger"`, `outlined=true`, and optional `primaryLabel="Retry"`. | Same fallback; pending state is not shown while boundary fallback is active. | Same fallback; retry may be disabled if unavailable. | Same fallback. | Not applicable. |

## Empty States

Each empty state uses `EmptyStatePlaceholder` at
`packages/ui/src/lib/components/EmptyStatePlaceholder.svelte`. The listed text is
the slot contract for `title`, `caption`, and `actions`; actions render with
`Button` at `packages/ui/src/lib/components/Button.svelte`.

| Tab | Empty state component and slots | Populated state component |
|---|---|---|
| Principals | `EmptyStatePlaceholder`; `title` slot `No principals configured`; `caption` slot `Grant a permission to register the first principal.`; `actions` slot `Button` label `Add first`. | `PrincipalsList` at `apps/desktop/src/components/governance/PrincipalsList.svelte` with rows built from `CardGroupRoot` and `CardGroupItem`. |
| Groups | `EmptyStatePlaceholder`; `title` slot `No groups yet…`; `caption` slot `Create a group to share inherited permissions across principals.`; `actions` slot `Button` label `Create group`. | `GroupsList` at `apps/desktop/src/components/governance/GroupsList.svelte` using `ExpandableSection` for each group. |
| Branch Gates | `EmptyStatePlaceholder`; `title` slot `No branch gates configured.`; `caption` slot `Add a protected branch rule before merges require review.`; `actions` slot `Button` label `Add gate`. | `BranchGatesList` at `apps/desktop/src/components/governance/BranchGatesList.svelte` using `ExpandableSection` for each branch. |
| Rules | `EmptyStatePlaceholder`; `title` slot `No rules for this principal`; `caption` slot `Select a principal or create an automation rule for the selected principal.`; `actions` slot `Button` label `Create rule`. | `RulesList` at `apps/desktop/src/components/rules/RulesList.svelte` with optional `principalId`, plus `Rule`, `RuleEditor`, `RuleFiltersEditor`, and `NewRuleMenu`. |

## Cross-Cutting Visual States

| State | Component path | Props and copy | Applies to |
|---|---|---|---|
| Pending banner | `packages/ui/src/lib/components/InfoMessage.svelte` via `apps/desktop/src/components/governance/GovernancePendingBanner.svelte` | `style="warning"`, `outlined=true`, `primaryLabel="Commit changes"`, `primaryIcon="arrow-right"`, title `4 pending governance changes`, content `Changes take effect once committed to the governance ref.` | Page-level area above tabs whenever pending count is greater than zero. |
| Read-only banner | `packages/ui/src/lib/components/InfoMessage.svelte` | `style="info"`, `outlined=true`, title `Read-only`, content `administration:write is required to change governance.` | Page-level area above tab content when the viewer lacks write permission. |
| Denial banner | `packages/ui/src/lib/components/InfoMessage.svelte` | `style="danger"`, `outlined=true`, `primaryLabel="Retry"` only for retryable IPC failures, title `perm.denied`, content `You cannot modify your own administration grants.` | Above the affected tab or editor after a refused write. |
| Pending row badge | `packages/ui/src/lib/components/Badge.svelte` | `style="warning"`, `kind="soft"`, children `○`; optional row text `pending`. | Principal rows, group rows, branch gate rows, editor save summary. |
| Committed row badge | `packages/ui/src/lib/components/Badge.svelte` | `style="gray"`, `kind="soft"`, children `●`; optional row text `committed`. | Principal rows, rule principal selector rows, branch gate summaries. |
| Inherited or unavailable control | `packages/ui/src/lib/components/Toggle.svelte` | `disabled=true`; `checked` mirrors the effective inherited value. | Inherited permission rows and read-only rows. |
| Read-only chips | `packages/ui/src/lib/components/TagInput.svelte` | `readonly=true`; tags remain visible and removal affordances are hidden by the component. | Principal groups, group members, branch required groups. |
| Read-only number field | `packages/ui/src/lib/components/Textbox.svelte` | `type="number"`, `readonly=true` or `disabled=true`. | Branch gate minimum approvals. |
| Destructive confirmation | `packages/ui/src/lib/components/Modal.svelte` | Confirmation copy from the specific row action; action `Button` uses existing Button styling. | Group delete and branch unprotect. |
| Error boundary fallback | `apps/desktop/src/components/governance/GovernanceErrorBoundary.svelte` plus `packages/ui/src/lib/components/InfoMessage.svelte` | `InfoMessage style="danger"`, `outlined=true`, title `Governance settings could not load`, optional `primaryLabel="Retry"`. | Whole governance surface if a render/runtime failure occurs. |

## Token And Styling Contract

- This annotation introduces no CSS variables, color literals, spacing values, or
  typography rules.
- Visual color, spacing, radius, type, hover, and focus treatment come from the
  component stylesheets listed above.
- Any use-site values are component props or slot content only.
- No governance-prefixed or management-prefixed CSS custom properties are part
  of this design.
