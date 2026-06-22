---
stability: FEATURE_SPEC
last_validated: 2026-06-20
prd_version: 1.0.0
functional_group: DEFN
---

# Use Cases: Check Definition (DEFN)

Checks are **config-as-code**, not engine opinion. The authoritative definition is a list of **named checks** in committed, ref-pinned `.gitbutler/checks/*.toml` — each a `name` + a `trigger` + a local `run-spec` + a `required` flag + an exit-code success mapping. The `name` is the stable identity the required-checks policy and every result key on (the analog of a GitHub status `context`). The config is read at the **target ref** when the gate evaluates required checks — exactly as governance reads `gates.toml` — so a change can never weaken or remove the checks that judge it, and a change to the required-set must itself clear the currently-required checks (the **bootstrap-invariant**). v1 resolves **local checks only** (a command or a repo `./path` script, butler-run). The agent-facing surface is the **`but check`** CLI.

> **Editable file, inert weakening (carried throughout).** `.gitbutler/checks/*.toml` is a file an agent with `contents:write` can edit, but — like all governed config — the _file write_ is **ungated**; weakening is **inert until committed to the target ref**, because the _landing merge_ is governance-gated (`administration:write`) and the config is ref-pinned, so a working-tree edit changes nothing until it lands. UC-DEFN-04 closes the same-change self-weakening hole; UC-DEFN-05 (the bootstrap-invariant) forces a required-set change to clear the currently-required checks.

> **Surface.** v1 ships the **`but check` CLI** (`define`, `list`, `required`). A desktop management surface is **deferred** — it would extend governance's existing settings section (a "Checks" tab beside Principals / Branch Gates); this PRD ships no new desktop _management/settings_ surface and **no new route** (the v1.1 result-viewing panel is new state on the existing `/branches` route — see [01-scope.md](./01-scope.md) "Human-facing UI").

| ID         | Title                                                             | Description                                                                                                                                                                                                                                                                                                |
| ---------- | ----------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| UC-DEFN-01 | Named-check schema in committed `.gitbutler/checks/*.toml`        | A User defines named checks in committed config; each parses to a typed definition with a stable `name`, a `trigger`, a `run-spec`, a `required` flag, and a success mapping; a malformed or duplicate-name definition fails closed as `config.invalid` rather than being silently dropped.                |
| UC-DEFN-02 | Triggers + local run-spec (command or `./path` script)            | Each check declares a `trigger` (`on-commit` / `on-merge-attempt`) and a `run-spec` that is a command **or** a repo-local `./path` script — local resolution only; a remote-ref / `uses` / Docker / JS / composite run-spec is rejected as out-of-v1-scope.                                                |
| UC-DEFN-03 | `required` flag + exit-code success mapping                       | Each check declares whether it is `required` and a deterministic success mapping (exit `0` → `success`, non-zero → `failure`), so a conclusion is a function of the executed check, never an authored value, and the required-set the gate enforces is auditable from committed config.                    |
| UC-DEFN-04 | Ref-pinned definitions read at the target ref (no self-weakening) | The check config and required-set are read at the **target ref** when the gate evaluates, so a change whose head weakens, drops, or flips `required` on a check cannot weaken the gate that judges it; an edit takes effect only once committed to the target ref.                                         |
| UC-DEFN-05 | Self-protecting required-set (the bootstrap-invariant)            | A change that adds, removes, weakens, or flips `required` on a required check — or edits a required check's `run-spec` — must itself clear the **currently-required** checks at the target ref before it can land, so the config that defines "good" is governed by the very checks it currently mandates. |

---

## UC-DEFN-01: Named-check schema in committed `.gitbutler/checks/*.toml`

A gate that enforces "the validations passed" needs a stable, committed definition of _what the validations are_. This use case establishes the config-as-code schema: `.gitbutler/checks/*.toml` holds a list of named checks, and the loader parses each into a typed definition. The **`name`** is the load-bearing field — the stable identity the required-checks policy and every result key on — so two checks may not share a name and a name may not be empty. The loader is strict: a malformed entry, an unknown field, a duplicate name, or a missing required field is a **definition error** that fails closed (`config.invalid`, mirroring governance), never a silently-dropped check (a dropped required check would be a fail-open hole the gate could not see).

### Acceptance Criteria

☐ A User can define one or more named checks in committed `.gitbutler/checks/*.toml`, so the validation set is config-as-code, versioned in the repo
☐ System parses each entry into a typed definition carrying a `name`, a `trigger`, a `run-spec`, a `required` flag, and a success mapping, so enforcement sees a structured definition rather than raw text
☐ System treats the `name` as the stable check identity and rejects a config with a duplicate or empty `name`, so the required-checks policy and the result store can key on it unambiguously
☐ System rejects a malformed, ambiguous, or unknown-field definition with a definition-error contract rather than silently dropping the check, so a required check can never disappear unnoticed
☐ System treats a structurally-broken or unparseable `.gitbutler/checks/*.toml` at the target ref as `config.invalid` and fails closed (the required-set is never treated as empty/satisfied), so a corrupt config can never fail open into a satisfied gate
☐ A User can run `but check list` to print the parsed definitions (name, trigger, required, run-spec) read from committed config, so the authored set is observable from the CLI with structured output and exit code 0
☐ A User can scaffold/validate a named check via `but check define <name> --command <cmd> [--trigger on-commit|on-merge-attempt]`, which writes a parseable `[[check]]` entry to `.gitbutler/checks/*.toml` (an **uncommitted** file — ungated until committed) with exit 0 and rejects an invalid spec (duplicate/empty name, out-of-scope run-spec) non-zero **without** writing, so authoring has a validated CLI path
☐ System has a passing integration test against the real check-config loader that parses a valid `.gitbutler/checks/*.toml` into the typed definitions and asserts that a duplicate-name, an unknown-field, and a structurally-broken (`config.invalid`) config each fail closed rather than loading or being treated as an empty required-set

### UI/UX Wireframe — Checks settings tab (deferred v1.2)

> **Surface:** the CLI `but check define` / `list` / `required` (above) is the **v1.0** surface. A desktop **"Checks" tab** would extend governance's settings modal (Principals · Groups · Branch Gates · Rules → **+ Checks**). **Deferred to v1.2** — it needs the DEFN CLI shipped first; also requires confirming that the governance settings tab wiring has landed (`ProjectSettingsModalContent.svelte` currently has no `governance` branch — it falls to the `else "not Found"` path; `projectSettingsPages.ts` has a `governance` entry but it is an **in-flight scaffold, not a rendered surface**).

```
Project Settings · ►Checks (new tab)            [ Defined | Required-Sets ]
  ┌ CardGroupRoot ───────────────────────────────────────────┐
  │ cargo-test   on-commit  [required ✓]  cargo test   [···] │
  │ pnpm-check   on-commit  [required ✓]  pnpm check   [···] │
  │ lint         on-merge   [ optional ]  ./lint.sh    [···] │
  └───────────────────────────────────────────────────────────┘  [+ Add check]
  Required-Sets:  main → [cargo-test] [pnpm-check]  [+]
```

**Component map** (full table → [`08-technical-requirements/10-frontend-ui.md`](./08-technical-requirements/10-frontend-ui.md) §3–4): **extend** `projectSettingsPages.ts` (`+{id:"checks"}`) + add a branch in `ProjectSettingsModalContent.svelte`; **reuse** `Tabs` / `TabList` / `TabTrigger` / `TabContent` (`apps/desktop/src/components/shared/` — confirmed; not from `packages/ui`), `CardGroupRoot/Item`, `Toggle`, `TagInput`, `Textbox`, `KebabButton`, `Button`, `SettingsSection`; **net-new** `ChecksSettings` + `CheckEditor` (mirrors the inline-editor pattern — see `governance/DESIGN-ANNOTATIONS.md` for `PrincipalEditor`, which is a **planned thin composition from the Net-new table, not yet implemented**). **Lite (React):** port, not a share (deferred).

---

## UC-DEFN-02: Triggers + local run-spec (command or `./path` script)

A check declares _when_ it is eligible to run and _what_ it runs, both constrained to the v1 scope. The **`trigger`** is exactly one of `on-commit` (eligible when a commit is created) or `on-merge-attempt` (the trusted CLI/daemon runs it in the pre-merge step and the gate evaluates the required set at head); any other value (cron, dispatch, external event) is rejected as out-of-v1-scope. The **`run-spec`** is **local resolution only**: a `command` string, or a repo-local `./path` script — both butler-run. A run-spec naming a remote-ref action (`owner/repo@ref`), a `uses` registry entry, a Docker image, a JS action, or a composite is rejected with a clear out-of-scope error, never silently accepted (silent acceptance would imply a capability the v1 runner does not have).

### Acceptance Criteria

☐ A User can declare a check's `trigger` as `on-commit` or `on-merge-attempt`, so a check is bound to a defined point in the change lifecycle
☐ System rejects a `trigger` value other than `on-commit` / `on-merge-attempt` with an out-of-scope definition error, so an unsupported trigger never loads as if supported
☐ A User can declare a `run-spec` as a `command` string or a repo-local `./path` script, so a check runs a real validation the butler runner can execute locally
☐ System rejects a `run-spec` naming a remote-ref action, a `uses` registry/marketplace entry, a Docker image, a JavaScript action, or a composite action with an out-of-v1-scope error, so an unresolvable run-spec never loads as a runnable check
☐ System has a passing integration test against the real loader asserting (a) `on-commit` and `on-merge-attempt` load, (b) an unknown trigger is rejected, (c) a command and a `./path` script run-spec load, and (d) a `uses` / Docker / JS run-spec is rejected as out-of-v1-scope

---

## UC-DEFN-03: `required` flag + exit-code success mapping

The gate's whole job is "every **required** check is green," so a check must declare whether it is required, and "green" must be a deterministic function of the executed check rather than an agent's claim. The **`required` flag** marks whether a check's `success` is mandatory; a `required: false` (or omitted-default) check runs and records but does not block. The **authoritative required-set** the gate enforces is the `[[required_check]]` policy in `gates.toml` (UC-GATE-01) — a check's `required` field is its declared default; the gating set is the committed policy, auditable as a diff. The **success mapping** is the exit-code contract: exit `0` → `success`, non-zero → `failure` — fixed and deterministic in v1 (no `${{ }}` expression, no custom predicate), so a conclusion cannot be authored, only computed from the real exit code by the runner.

### Acceptance Criteria

☐ A User can mark a check `required: true` to make its `success` mandatory for the merge gate, or `required: false` (the default) so it runs and records without blocking
☐ System computes the gate's required-set from the target-ref `[[required_check]]` policy, so which checks block a merge is auditable from committed config
☐ System maps a check's process exit code `0` to conclusion `success` and a non-zero exit to `failure` deterministically, so a conclusion is computed from the real run, never authored
☐ System provides no v1 mechanism to author a `success` conclusion independent of the executed run (no `${{ }}` expression, no custom predicate), so a check cannot be marked green without actually exiting `0`
☐ A User can run `but check required` to print the target-ref required-set per branch, so the gating set is observable from the CLI
☐ System has a passing integration test against the real loader + a real exit-`0` and exit-`1` command asserting (a) the required-set equals the `[[required_check]]` policy, (b) exit `0` → `success`, and (c) exit non-zero → `failure`

---

## UC-DEFN-04: Ref-pinned definitions read at the target ref (no self-weakening)

Definitions decide what blocks a merge, so they are the obvious target for self-weakening: if a change could drop a required check, or flip its `required` to false, **in its own head**, validations would be theater. This use case closes that by reading the check config and the required-set at the **target ref** when the gate evaluates — exactly as governance reads `gates.toml`. A change whose head edits `.gitbutler/checks/*.toml` to remove a required check, flip its flag, or replace its run-spec with a trivially-passing command is judged against the **target-ref** definitions, not the head it is trying to land. A `but check` edit takes effect only once committed to the target ref, and the gate NEVER reads the config from the working tree or the feature head when deciding.

### Acceptance Criteria

☐ System reads the check config (`.gitbutler/checks/*.toml`) and the `[[required_check]]` policy at the **target ref** when the gate evaluates required checks, consistent with the governance `gates.toml` ref-pin
☐ System makes a change whose head removes a required check, flips its `required` to false, or weakens its run-spec **ineffective** for weakening the gate that judges it, because the required-set is read at the target ref, not the feature head
☐ A User-edited `.gitbutler/checks/*.toml` takes effect only once committed to the target ref (a working-tree or feature-head edit is inert for the gate decision), so a definition change is governed exactly like a permission change
☐ System NEVER reads the check config from the working tree or the feature head when deciding required checks — ONLY the target-ref config blob — so a staging-area or uncommitted edit cannot influence the gate decision
☐ System has a passing integration test against real git that creates a feature change whose head deletes a required check from `.gitbutler/checks/*.toml`, attempts the merge, and asserts the required check is still enforced (read from the target-ref config) so the self-weakening change cannot slip the gate

---

## UC-DEFN-05: Self-protecting required-set (the bootstrap-invariant)

Reading config at the target ref (UC-DEFN-04) stops a change weakening the gate _for itself_ — but the change that _lands_ a weaker config still has to be governed, or an agent could simply land "remove the test check" as an ordinary change and weaken the gate for everything after. This use case closes that with the **bootstrap-invariant**: a change that **adds, removes, weakens, flips `required` on, or edits the `run-spec` of a required check** must itself **clear the currently-required checks** at the target ref before it can land. The required-set is therefore **self-protecting** — you cannot use a weakening change to escape the checks that weakening change must itself pass — exactly the self-escalation-prevention shape governance uses for its ref-pin (a permission change cannot grant itself the authority it needs to land).

### Acceptance Criteria

☐ System detects when a change's diff modifies the required-set or a required check's definition (adds/removes a required check, flips `required`, or edits a required check's `run-spec`), so a policy-affecting config change is recognized as such rather than treated as ordinary content
☐ System requires a change that modifies the required-set or a required check's definition to itself clear the **currently-required** checks (read at the target ref) before it can land, so a weakening change cannot escape the checks it must itself pass
☐ A User can land a change to the required-check configuration only when that change's own head has every currently-required check `success` at head, so tightening or loosening the gate is itself gated by the current gate
☐ System makes a weakened configuration take effect only for **future** changes — once the weakening change has itself cleared the currently-required set — mirroring governance's ref-pin self-escalation prevention
☐ System has a passing integration test against real git that attempts to land a change which deletes (or flips `required: false` on) a currently-required check, asserts the merge is blocked unless that change's own head satisfies the currently-required checks at the target ref, and asserts the weakened config only governs subsequent changes
