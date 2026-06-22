---
stability: FEATURE_SPEC
last_validated: 2026-06-19
prd_version: 1.0.0
functional_group: DEFN
---

# Use Cases: Action Definition (DEFN)

Validations are **config-as-code**, not engine opinion. The authoritative definition is a list of **named checks** in committed, ref-pinned `.gitbutler/actions/*.toml`, where each check is a `name` + a `trigger` + a `run-spec` + a `required` flag + a success mapping. The `name` is the **stable identity** the merge gate's required-checks policy references (the analog of a GitHub status `context` / check-run name). The config is read at the **target ref** when the gate evaluates required checks — exactly as governance reads `gates.toml` — so a change can never weaken or remove the checks that judge it, and a change to the required-check set must itself clear the currently-required checks (the **bootstrap-invariant**). v1 resolves **local actions only** (a command or a repo `./path` script, butler-run); the engine ships the schema + loader, and every opinion about _which_ validations matter lives in the committed file. The agent-facing surface is the **`but check`** CLI noun (distinct from the existing `butler_actions` macro feature — see [00-overview.md](./00-overview.md)).

> **Editable file, inert weakening (carried throughout).** `.gitbutler/actions/*.toml` is a file an agent with `contents:write` can edit, but — like all governed config — weakening it is **inert until committed to the target ref**, it is a reviewable diff, and (via governance) it is gated by `administration:write`. The DEFN ref-pin closes the _same-change_ self-weakening hole; the bootstrap-invariant (UC-DEFN-05) forces a required-set change to clear the currently-required checks; and a required-set change additionally requires a **human in the loop** (an automated agent reviewer may not catch the policy implication — see [01-scope.md § Known Limitations](./01-scope.md#known-limitations)). A config-weakening change that lands on the target ref taking effect for _future_ changes is the same accepted-leak scope line as governance R13.

| ID         | Title                                                             | Description                                                                                                                                                                                                                                                                                                                                                                            |
| ---------- | ----------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| UC-DEFN-01 | Named-check schema in committed `.gitbutler/actions/*.toml`       | A User defines named checks in committed config; each check parses to a typed definition with a stable `name`, a `trigger`, a `run-spec`, a `required` flag, and a success mapping; the loader rejects a malformed or ambiguous definition (a malformed config fails closed as `config.invalid`) rather than silently dropping it.                                                     |
| UC-DEFN-02 | Triggers + local run-spec (command or `./path` script)            | Each check declares a `trigger` (`on-commit` / `on-merge-attempt`) and a `run-spec` that is a command **or** a repo-local `./path` script — local-action resolution only in v1; a remote-ref / `uses` / Docker / JS / composite run-spec is rejected as out-of-v1-scope, not silently accepted.                                                                                        |
| UC-DEFN-03 | `required` flag + exit-code success mapping                       | Each check declares whether it is `required` (mandatory for the merge gate) and a deterministic success mapping (process exit `0` → conclusion `success`, non-zero → `failure`), so a conclusion is a function of the executed check and the required-set the gate enforces is auditable from the config.                                                                              |
| UC-DEFN-04 | Ref-pinned definitions read at the target ref (no self-weakening) | The check config is read at the **target ref** when the gate evaluates required checks, so a change whose head weakens, drops, or flips `required` on a check cannot weaken the gate that judges it; an edit takes effect only once committed to the target ref.                                                                                                                       |
| UC-DEFN-05 | Self-protecting required-check set (the bootstrap-invariant)      | A change that adds, removes, weakens, or flips `required` on a required check — or edits a required check's `run-spec` — must itself clear the **currently-required** checks at the target ref before it can land, so the config that defines "good" is governed by the very checks it currently mandates (the same self-escalation-prevention shape governance uses for its ref-pin). |

---

## UC-DEFN-01: Named-check schema in committed `.gitbutler/actions/*.toml`

A merge gate that enforces "the validations passed" needs a stable, committed definition of _what the validations are_. This use case establishes the config-as-code schema: `.gitbutler/actions/*.toml` holds a list of named checks, and the loader parses each entry into a typed check definition. The **`name`** is the load-bearing field — it is the stable identity the required-checks policy and every ledger record key on (the analog of GitHub's status `context` / check-run name), so two checks may not share a name and a check's name may not be empty. The loader is strict: a malformed entry, an unknown field, a duplicate name, or a missing required field is a **definition error** that fails closed (a structurally-broken config is `config.invalid`, mirroring governance's fail-closed config handling), never a silently-dropped check (a dropped required check would be a fail-open hole the gate could not see).

### Acceptance Criteria

☐ A User can define one or more named checks in committed `.gitbutler/actions/*.toml`, so the set of validations is config-as-code, versioned in the repo
☐ System parses each check entry into a typed definition carrying a `name`, a `trigger`, a `run-spec`, a `required` flag, and a success mapping, so enforcement sees a structured definition rather than raw text
☐ System treats the `name` as the stable check identity (the analog of a GitHub status `context` / check-run name) and rejects a config with a duplicate or empty `name`, so the required-checks policy and the ledger can key on it unambiguously
☐ System rejects a malformed, ambiguous, or unknown-field check definition with a definition-error contract rather than silently dropping the check, so a required check can never disappear unnoticed
☐ System treats a structurally-broken or unparseable `.gitbutler/actions/*.toml` at the target ref as `config.invalid` and fails closed (the required-set is never treated as empty/satisfied), mirroring governance's fail-closed config handling, so a corrupt config can never fail open into a satisfied gate
☐ A User can run `but check list` to print the parsed check definitions (name, trigger, required) read from the committed config, so the authored validation set is observable from the CLI with structured output and exit code 0
☐ System has a passing integration test against the real Actions config loader that parses a valid `.gitbutler/actions/*.toml` into the typed definitions (asserting name/trigger/run-spec/required/success-mapping) and asserts a duplicate-name, an unknown-field, and a structurally-broken (`config.invalid`) config each fail closed rather than loading or being treated as an empty required-set

### UI/UX Wireframe

> **Scope calibration (read first).** v1 ships **no GUI** for check definitions ([01-scope.md](./01-scope.md) "A new agent UI or a new app … out of scope"; [09-technical-requirements/README.md](./09-technical-requirements/README.md) "This slice adds NO new route and NO new app"). The v1 user-facing surface is the **`but check` CLI**; the management GUI shown below is the **deferred** surface the PRD names ("a human management surface, if any, would extend governance's existing MGMT settings section"). Both are specified here so the deferred GUI is design-ready and composes with the governance MGMT surface ([`.spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md`](../../governance/10-technical-requirements/10-ui-infrastructure.md)).

**Surface:** CLI (`but check list` / `but check define`) — **v1 present**; desktop SvelteKit/Tauri (governance-settings extension) — **deferred**.
**Entry point:** CLI — `but check list` to read, `but check define`/hand-edit `.gitbutler/actions/*.toml` to author; GUI (deferred) — Project Settings → "Permissions & Governance" → new **Checks** tab (extends governance's Principals · Groups · Branch Gates · Rules → 5 tabs).
**Trigger:** CLI — human fleet-owner/admin or orchestrator invokes the verb; GUI — admin opens project settings.

**Layout sketch — CLI, v1 (`but check list`, structured table):**

```
$ but check list
NAME            TRIGGER           REQUIRED   RUN-SPEC
tests           on-merge-attempt  yes        ./scripts/run-tests.sh
lint            on-commit         no         pnpm lint
typecheck       on-merge-attempt  yes        pnpm -F @gitbutler/ui check

3 checks (2 required) · config read from .gitbutler/actions/*.toml @ refs/heads/main
exit 0
```

`but check list --json` emits the same rows machine-parseable for the orchestrator (dual-audience output, UC-EXEC-05). A structurally-broken config exits non-zero with a `config.invalid` error block (fail-closed), never an empty list.

**Layout sketch — GUI, deferred (Checks tab; mirrors governance's Principals/Groups tabs):**

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Project settings        Permissions & Governance            (adminOnly page) │
│ ┌──────────────────┐  ⚠ Changes take effect once committed. [○] pending [Commit→]│
│ │ Project          │  [Principals][Groups][Branch Gates][Rules][●Checks]       │
│ │ AI options       │  ───────────────────────────────────────────────────────  │
│ │ ●Permissions &   │  Checks   reads .gitbutler/actions/*.toml        [+ Add]  │
│ │  Governance      │  ┌──────────────────────────────────────────────────┐    │
│ │                  │  │ ● tests        on-merge-attempt  [required]  [···]│    │
│ │                  │  │ ● lint         on-commit        optional    [···]│    │
│ │                  │  │ ● typecheck    on-merge-attempt  [required]  [···]│    │
│ │                  │  │ ○ new-check    on-commit        optional ○pend [···]│   │
│ │                  │  └──────────────────────────────────────────────────┘    │
│ │                  │  ─ empty: EmptyStatePlaceholder "No checks defined"       │
│ └──────────────────┘  ℹ Read-only: administration:write required to edit.      │
└─────────────────────────────────────────────────────────────────────────────┘
Legend: [●] committed  [○] pending (working-tree edit, inert until committed to ref)
```

**Key regions:**

- Checks list — one row per defined check: `name` (stable identity), `trigger`, required badge, `run-spec`, overflow `KebabButton` (edit/delete/view-results). Pending edits show ○.
- `+ Add` — opens the check-definition editor (UC-DEFN-02/03 fields).
- Pending banner + Commit — reuses governance's `GovernancePendingBanner` pattern; a definition change is inert until committed to the target ref (UC-DEFN-04).
- Read-only banner — shown when the viewer lacks `administration:write` (controls disabled).

**Interaction flow:**

1. Admin opens Project Settings → Permissions & Governance → **Checks** tab.
2. `but check list` (or the Tauri equivalent via `but-sdk`) loads the parsed definitions read at the target ref.
3. `+ Add` opens `CheckDefinitionEditor`; filling name/trigger/run-spec/required stages a working-tree edit (○ pending) — no enforcement change yet.
4. `[Commit →]` commits `.gitbutler/actions/*.toml`; pending indicators clear; the new check is now in the target-ref config.

**States:** populated (rows), empty (`EmptyStatePlaceholder`), pending (○ on edited rows + warning banner), read-only (`administration:write` missing), error (malformed config → danger `InfoMessage` with `config.invalid` + the parse error; list refuses to render a partial/empty set, matching fail-closed).

**Existing components to use:**

- `packages/ui/src/lib/components/cardGroup/CardGroupRoot.svelte` + `CardGroupItem.svelte` — check rows.
- `packages/ui/src/lib/components/Badge.svelte` — `required` badge (`warning`/`soft`), `optional` badge (`gray`/`soft`).
- `packages/ui/src/lib/components/Button.svelte` — `+ Add`, `Commit`.
- `packages/ui/src/lib/components/KebabButton.svelte` + `ContextMenu.svelte` — per-row overflow.
- `packages/ui/src/lib/components/InfoMessage.svelte` — pending/read-only/error banners.
- `packages/ui/src/lib/components/EmptyStatePlaceholder.svelte` — empty state.
- `packages/ui/src/lib/components/Tooltip.svelte` — name/trigger/help.
- `apps/desktop/src/components/shared/SettingsSection.svelte`, `Tabs.svelte`, `AppScrollableContainer.svelte` — tab scaffold.
- Governance (deferred sibling): `apps/desktop/src/components/settings/SettingsModalLayout.svelte`, `ProjectSettingsModalContent.svelte`, `GovernancePendingBanner.svelte` (pattern).

**Net-new components (atomic):**

- `CheckStatusBadge.svelte` (atom) — renders a GitHub-compatible check **conclusion** as a colored badge; composes `Badge` + `Icon`. Lives in `packages/ui/src/lib/components/`. Props: `conclusion: "success"|"failure"|"neutral"|"cancelled"|"timed_out"|"skipped"|"missing"|"stale"|"unverifiable"`; `kind?: "icon"|"text"|"both"`. Variants: success→`safe` + `tick-circle`; failure→`danger` + `cross-circle`; timed_out→`warning` + `clock`; skipped/neutral→`gray` + `stop`; stale→`warning` + `refresh`; missing→`gray` + `clock`; unverifiable→`danger` + `lock-auth`. (Reused by every check-result/gate surface; the closest existing analog is `CommitStatusBadge.svelte`/`PrStatusBadge.svelte` but those are review/PR vocabularies, not the check vocabulary.)
- `RequiredBadge.svelte` (atom) — thin wrapper: `required` → `Badge style="warning" kind="soft"`, `optional` → `Badge style="gray" kind="soft"`. Lives in `packages/ui/src/lib/components/`. (One-liner composition; could be inlined per Rule-of-2 — promote to component only if used in ≥2 places: list row + editor.)
- `CheckDefinitionRow.svelte` (molecule) — one row: name + trigger + `RequiredBadge` + run-spec + `KebabButton`; composes `CardGroupItem`. Lives in `apps/desktop/src/components/checks/`. Props: `definition`, `pending: boolean`, `readonly: boolean`; slots: `actions`.
- `ChecksList.svelte` (organism) — the Checks tab body: list + add + empty + read-only. Lives in `apps/desktop/src/components/checks/`.

**UI mods to existing components:**

- `apps/desktop/src/lib/settings/projectSettingsPages.ts` — MODIFY: no change needed if Checks is a sub-tab of the governance page; if it is its own admin page, add `{ id: "checks", label: "Checks", icon: "checklist", adminOnly: true }`. Reason: the settings registry is the single source of settings pages.
- `apps/desktop/src/components/settings/ProjectSettingsModalContent.svelte` — MODIFY: branch to render `ChecksSettingsTab.svelte`. Reason: the renderer dispatches page → component.
- Governance's `GovernanceSettings.svelte` (deferred) — MODIFY: add a 5th `TabTrigger` ("Checks") when present. Reason: Checks composes _inside_ governance's settings, not beside it.

**Accessibility notes:** Checks tab is a `TabTrigger` with `aria-label="Checks"`; arrow-key nav between tabs (existing `Tabs` contract). Each row is a `role="row"` with the required badge exposed via `aria-label="required"`/`aria-label="optional"`. Read-only rows: controls `disabled` + `aria-disabled`. Error block: `role="alert"`. All color states have a non-color affordance (icon + label) — never color-only.

**Edge cases / responsive / platform-specific:** CLI table truncates long `run-spec` with ellipsis + `--wide` for full width (terminal width detection). GUI: settings modal is fixed-width (no mobile); `run-spec` long values wrap inside the card. **`packages/ui` is Svelte-only** — if the **lite** (React/Electron) app ever needs this surface, `CheckStatusBadge`/`RequiredBadge` must be re-implemented in React (flag as a port, not a share). Web (`apps/web`) consumes `packages/ui` directly — no port.

---

## UC-DEFN-02: Triggers + local run-spec (command or `./path` script)

A check needs to declare _when_ it is eligible to run and _what_ it runs. This use case constrains both to the v1 scope. The **`trigger`** is one of exactly two values: `on-commit` (the check is eligible when a commit is created) or `on-merge-attempt` (the trusted CLI/daemon runs the check in the pre-merge step and the gate then evaluates the required set at head when a governed merge is attempted); any other trigger value (cron, dispatch, external event) is rejected as out-of-v1-scope. The **`run-spec`** is **local-action resolution only**: a `command` string, or a repo-local `./path` script — both butler-run. A run-spec that names a remote-ref action (`owner/repo@ref`), a `uses` registry/marketplace entry, a Docker image, a JS action, or a composite is rejected with a clear out-of-scope error, never silently accepted (silent acceptance would imply a capability the v1 executor does not have).

### Acceptance Criteria

☐ A User can declare a check's `trigger` as `on-commit` or `on-merge-attempt`, so a check is bound to a defined point in the change lifecycle
☐ System rejects a `trigger` value other than `on-commit` / `on-merge-attempt` (e.g. cron / dispatch / external) with an out-of-scope definition error, so an unsupported trigger never loads as if supported
☐ A User can declare a `run-spec` as a `command` string or a repo-local `./path` script, so a check runs a real validation the butler executor can execute locally
☐ System rejects a `run-spec` that names a remote-ref action (`owner/repo@ref`), a `uses` registry/marketplace entry, a Docker image, a JavaScript action, or a composite action with an out-of-v1-scope error, so an unresolvable run-spec never loads as a runnable check
☐ System has a passing integration test against the real loader asserting (a) `on-commit` and `on-merge-attempt` load, (b) an unknown trigger is rejected, (c) a command and a `./path` script run-spec load, and (d) a `uses: owner/repo@ref` / Docker / JS run-spec is rejected as out-of-v1-scope

### UI/UX Wireframe

> **Scope calibration.** v1 surface is the **CLI**; the GUI editor below is **deferred** (extends governance MGMT settings). See UC-DEFN-01 calibration note.

**Surface:** CLI (`but check list` / `but check define`) — **v1 present**; desktop check-definition editor — **deferred**.
**Entry point:** CLI — `but check define` writes a typed def (or hand-edit `.gitbutler/actions/*.toml`); GUI (deferred) — Checks tab → `+ Add` / row `[···]` → edit.
**Trigger:** Author selects a trigger + enters a run-spec; validation rejects out-of-v1-scope values immediately.

**Layout sketch — GUI, deferred (CheckDefinitionEditor; mirrors governance PrincipalEditor slide-in):**

```
┌──────────────────────────────────────────────────────────────────┐
│ Check: tests                                             [✕ Close] │
│ NAME        [ tests                           ]  ⓘ stable identity │
│ TRIGGER     ( on-commit )  [● on-merge-attempt ]                    │
│ RUN-SPEC    type: (● command  ○ ./path script )                     │
│             [ ./scripts/run-tests.sh             ]  [▶ Test run]    │
│ TIMEOUT     [ 600 ] secs                                           │
│ SECRETS     [ npm_token ✕ ]  [+ Add declared secret ▾]             │
│                                  [Cancel] [Save changes ○ pending] │
└──────────────────────────────────────────────────────────────────┘
```

**Layout sketch — CLI error, v1 (out-of-scope run-spec / trigger, fail-closed):**

```
$ but check define --file .gitbutler/actions/bad.toml
✕ config.invalid — check "ci" rejected: run-spec "uses: actions/checkout@v4"
  is out-of-v1-scope (local command or ./path script only). See UC-DEFN-02.
exit 2
```

**Key regions:**

- TRIGGER `SegmentControl` — exactly two options (`on-commit` / `on-merge-attempt`); any other value is unselectable (radio semantics enforce the enum).
- RUN-SPEC type toggle + input — `command` (free text) or `./path` script (path picker); a pasted `uses:`/Docker/JS value is rejected inline with an inline danger hint.
- `▶ Test run` — invokes the executor once locally (UC-EXEC) to validate the spec; result shown as a `CheckStatusBadge` (v1-produced `success`/`failure`/`timed_out` only).
- Inline validation — out-of-scope errors surface before save, mirroring the loader's fail-closed `config.invalid`.

**Interaction flow:**

1. Admin opens the editor (or runs `but check define`).
2. Selects trigger (2 options) and run-spec type; enters the value.
3. Inline validation rejects an out-of-scope trigger/run-spec on blur; `▶ Test run` optionally validates.
4. `[Save changes]` stages a working-tree edit (○ pending); `[Commit →]` lands it (inert until committed, UC-DEFN-04).

**States:** editing (populated), new (empty fields), inline-error (out-of-scope value, danger hint, save disabled), testing (`spinner` on `▶ Test run`), pending (○ after save), read-only (`administration:write` missing).

**Existing components to use:**

- `packages/ui/src/lib/components/segmentControl/SegmentControl.svelte` + `Segment.svelte` — trigger enum + run-spec type.
- `packages/ui/src/lib/components/Textbox.svelte` — name, command, timeout (`type=number`).
- `packages/ui/src/lib/components/TagInput.svelte` — declared secrets.
- `packages/ui/src/lib/components/Button.svelte` — `▶ Test run`, `Cancel`, `Save changes`.
- `packages/ui/src/lib/components/InfoMessage.svelte` — inline out-of-scope error + pending banner.
- `packages/ui/src/lib/components/Tooltip.svelte` — `⓫ stable identity` help.
- Net-new `CheckStatusBadge.svelte` (UC-DEFN-01) — `▶ Test run` result.

**Net-new components (atomic):**

- `CheckDefinitionEditor.svelte` (organism) — the slide-in form above; composes `SegmentControl` + `Textbox` + `TagInput` + `Button`. Lives in `apps/desktop/src/components/checks/`. Props: `definition?`, `readonly`, `onSave`; batch-save model (stages working-tree edit, no per-field write) mirroring governance's `PrincipalEditor` (B16).

**UI mods to existing components:** none beyond UC-DEFN-01's settings-registry/renderer mods. `SegmentControl` already supports a 2-option enum.

**Accessibility notes:** trigger is a radio group (`role="radiogroup"`, arrow-key nav); run-spec type is a radio group; the out-of-scope error is `aria-live="polite"` + `role="alert"` on submit. `▶ Test run` announces result via `aria-live`. Field labels are `<label for>`-bound.

**Edge cases / responsive / platform-specific:** `./path` script picker uses a Tauri file dialog on desktop (no such picker on web — text input only). CLI `but check define` emits the same `config.invalid` contract the loader uses, so error text is identical across surfaces. Lite (React) port required if the editor is needed there (Svelte-only library).

---

## UC-DEFN-03: `required` flag + exit-code success mapping

The gate's whole job is "every **required** check is green," so a check must declare whether it is required, and a "green" must be a deterministic function of the executed check rather than an agent's claim. This use case defines both. The **`required` flag** marks whether a check's `success` is mandatory for the merge gate; a `required: false` (or omitted-default-non-required) check runs and records but does not block, and the gate's required-set is therefore exactly the set of `required: true` checks in the target-ref config — auditable as a diff. The **success mapping** is the exit-code contract: a process exit code of `0` maps to conclusion `success`; a non-zero exit maps to `failure`. The mapping is fixed and deterministic in v1 (no `${{ }}` expression, no custom success predicate) so that a conclusion cannot be authored — it is computed from the real exit code by the executor.

### Acceptance Criteria

☐ A User can mark a check `required: true` to make its `success` mandatory for the merge gate, or `required: false` (the default) so it runs and records without blocking
☐ System computes the gate's required-set as exactly the `required: true` checks in the target-ref check config, so which checks block a merge is auditable from the committed config
☐ System maps a check's process exit code `0` to conclusion `success` and a non-zero exit to `failure` deterministically, so a conclusion is computed from the real run, never authored
☐ System does not provide a v1 mechanism to author a `success` conclusion independent of the executed run (no `${{ }}` expression, no custom success predicate), so a check cannot be marked green without actually exiting `0`
☐ System has a passing integration test against the real loader + a real exit-`0` and exit-`1` command asserting (a) the required-set equals the `required: true` checks, (b) exit `0` → `success`, and (c) exit non-zero → `failure`

### UI/UX Wireframe

> **Scope calibration.** v1 surface is the **CLI** (`but check list` shows the required column; `but check required` prints the required-set); the GUI required-flag toggle below is **deferred**. See UC-DEFN-01 calibration note.

**Surface:** CLI (`but check list` / `but check required`) — **v1 present**; desktop — **deferred**.
**Entry point:** CLI — `but check required` prints the target-ref required-set; GUI — Checks tab row + Branch Gates tab (where the required-set policy `[[required_check]]` actually lives).
**Trigger:** Admin flips a check's `required` flag, or adds a check name to the branch's `[[required_check]]` policy.

**Layout sketch — CLI, v1 (`but check required`):**

```
$ but check required
TARGET          REQUIRED CHECKS
main            tests, typecheck
develop         tests

required-set read from [[required_check]] in .gitbutler/gates.toml @ target ref
exit 0
```

**Layout sketch — GUI, deferred (the required-set is authored in governance's Branch Gates tab — composition):**

```
┌──────────────────────────────────────────────────────────────────┐
│ Branch Gates              reads .gitbutler/gates.toml             │
│ ▼ main                                              ○ pending      │
│   Protected branch                          [●] Toggle ON          │
│   Min. approvals required                  [ 2 ]                   │
│   REQUIRED CHECKS (Actions)   [tests ✕] [typecheck ✕] [+ ▾]        │
│     ⓘ must be a defined check (Checks tab). Self-protecting: a    │
│       change to this set must itself clear these checks.          │
│ ▶ develop                                          ● committed     │
└──────────────────────────────────────────────────────────────────┘
```

The `required` checkbox inside the Checks-tab editor (UC-DEFN-02) is the **per-check default**; the **authoritative required-set the gate enforces** is this `[[required_check]]` policy (per `03-functional-groups.md`), so the Branch Gates tab is where the gating required-set is visibly composed — and it is where the bootstrap-invariant (UC-DEFN-05) bites.

**Key regions:**

- REQUIRED CHECKS chip list (`TagInput`) — options are exactly the defined checks (Checks tab); a name not defined is rejected inline (consistent-set rule, mirrors governance's required-group selector).
- Self-protecting hint — explains the bootstrap-invariant in human terms.

**Interaction flow:**

1. Admin opens Branch Gates tab → expands a branch.
2. Adds/removes check names in REQUIRED CHECKS; options restricted to defined checks.
3. `[Save]` stages `gates.toml`; `[Commit →]` lands it. The change is itself gated by the currently-required checks (UC-DEFN-05) — surfaced as a denial if not (UC-GATE-03).

**States:** populated, empty (`EmptyStatePlaceholder` "No required checks — merge is ungated by checks"), pending (○), read-only, invalid (undefined check name → inline danger hint, save disabled).

**Existing components to use:**

- `packages/ui/src/lib/components/TagInput.svelte` — required-check chips.
- `packages/ui/src/lib/components/select/Select.svelte` — add-check dropdown (options = defined checks).
- `packages/ui/src/lib/components/Tooltip.svelte` + `InfoMessage.svelte` — self-protecting hint.
- Governance (deferred sibling): `BranchGatesList.svelte` (the host tab).

**Net-new components (atomic):**

- `RequiredChecksEditor.svelte` (molecule) — the REQUIRED CHECKS row inside Branch Gates; composes `TagInput` + `Select` + a validation that chips ⊆ defined-check names. Lives in `apps/desktop/src/components/checks/`. Props: `value: string[]`, `definedChecks: string[]`, `readonly`.

**UI mods to existing components:**

- Governance's `BranchGatesList.svelte` (deferred) — MODIFY: add the REQUIRED CHECKS field row per branch. Reason: the required-set policy lives in `gates.toml` alongside the gate fields, so it composes _inside_ the gates editor, not as a separate surface.

**Accessibility notes:** chip list is a `role="group"` with `aria-label="Required checks for <branch>"`; each chip's remove button has `aria-label="Remove <check> from required"`. The "undefined check" hint is `aria-live`.

**Edge cases / responsive / platform-specific:** A check deleted from the Checks tab while still referenced in `[[required_check]]` is a `config.invalid` (fail-closed) — the UI should cross-reference and warn at edit time. CLI `but check required` exit code is 0 even when the set is empty (empty = ungated, not an error) but non-zero on a malformed `gates.toml`.

---

## UC-DEFN-04: Ref-pinned definitions read at the target ref (no self-weakening)

Definitions are the lever that decides what blocks a merge, so they are the obvious target for self-weakening: if a change could drop a required check, or flip its `required` to false, **in its own head**, validations would be theater. This use case closes that by reading the check config at the **target ref** when the gate evaluates required checks — exactly as governance reads `permissions.toml` / `gates.toml`. A change whose head edits `.gitbutler/actions/*.toml` to remove a required check, flip its flag, or replace its run-spec with a trivially-passing command is judged against the **target-ref** definitions, not the head it is trying to land. A `but check` edit therefore takes effect only once committed to the target ref (never from the working tree), and the gate NEVER reads the config from the working tree or the feature head when deciding.

### Acceptance Criteria

☐ System reads the check config (`.gitbutler/actions/*.toml`) at the **target ref** when the gate evaluates required checks, consistent with the governance `gates.toml` ref-pin
☐ System makes a change whose head removes a required check, flips its `required` to false, or weakens its run-spec **ineffective** for weakening the gate that judges it, because the required-set is read at the target ref, not the feature head
☐ A User-edited `.gitbutler/actions/*.toml` takes effect only once committed to the target ref (a working-tree or feature-head edit is inert for the gate decision), so a definition change is governed exactly like a permission change
☐ System NEVER reads the check config from the working tree or the feature head when deciding required checks — ONLY the target-ref config blob — so a staging-area or uncommitted edit cannot influence the gate decision
☐ System has a passing integration test against real git that creates a feature change whose head deletes a required check from `.gitbutler/actions/*.toml`, attempts the merge, and asserts the required check is still enforced (read from the target-ref config) so the self-weakening change cannot slip the gate

> **No dedicated UI work.** This is an engine-enforced ref-pin (config read at the target ref) with no user-facing surface of its own. Its only UI-visible consequence is the **pending-until-committed** affordance already specified in UC-DEFN-01/02/03 (a working-tree edit shows ○ and is inert until committed to the target ref), which reuses governance's `GovernancePendingBanner` pattern verbatim — no net-new component.

---

## UC-DEFN-05: Self-protecting required-check set (the bootstrap-invariant)

Reading the config at the target ref (UC-DEFN-04) stops a change from weakening the gate _for itself_ — but the change that _lands_ a weaker config still has to be governed, or an agent could simply land "remove the test check" as an ordinary change and weaken the gate for everything after. This use case closes that with the **bootstrap-invariant**: a change that **adds, removes, weakens, flips `required` on, or edits the `run-spec` of a required check** must itself **clear the currently-required checks** at the target ref before it can land. The required-check configuration is therefore **self-protecting** — you cannot use a weakening change to escape the checks that weakening change must itself pass — exactly the self-escalation-prevention shape governance uses for its ref-pin (a permission change cannot grant itself the authority it needs to land). Because an automated agent reviewer may not recognize a `required:false` / required-set diff as policy-weakening, v1 also **requires a human in the loop** to approve such a change (named in [01-scope.md](./01-scope.md)); the bootstrap-invariant is the engine-enforced leg, the human-in-loop is the process-enforced leg.

### Acceptance Criteria

☐ System detects when a change's diff modifies the required-check set or a required check's definition (adds/removes a required check, flips `required`, or edits a required check's `run-spec`), so a policy-affecting config change is recognized as such rather than treated as ordinary content
☐ System requires a change that modifies the required-check set or a required check's definition to itself clear the **currently-required** checks (read at the target ref) before it can land, so a weakening change cannot escape the checks it must itself pass (the bootstrap-invariant)
☐ A User can land a change to the required-check configuration only when that change's own head has every currently-required check green at head, so tightening or loosening the gate is itself gated by the current gate
☐ System enforces that the required-check configuration is self-protecting — a change cannot use its own weakening of the required set to weaken the gate that judges it — mirroring governance's ref-pin self-escalation prevention
☐ System has a passing integration test against real git that attempts to land a change which deletes (or flips `required: false` on) a currently-required check, asserts the merge is blocked unless that change's own head satisfies the currently-required checks at the target ref, and asserts the weakened config only takes effect for _future_ changes once it has itself cleared the current required set

> **No dedicated UI work.** The bootstrap-invariant is engine-enforced gate logic (a required-set change must clear the currently-required checks). Its only UI-visible consequence is a **gate denial** when a weakening change's own head is not green — that denial is specified in [UC-GATE-03](./07-uc-gate.md) (the `gate.check_required` denial + STEER "this change weakens the required set and must itself pass: <checks>"). The `RequiredChecksEditor` (UC-DEFN-03) carries an inline hint explaining the invariant; no other net-new component.
