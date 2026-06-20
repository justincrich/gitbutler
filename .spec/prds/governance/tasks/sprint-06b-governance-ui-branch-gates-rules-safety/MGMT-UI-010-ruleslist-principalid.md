# MGMT-UI-010: Extend RulesList with optional principalId prop (backward compatible)

## What this does

Add an optional principalId prop to RulesList.svelte that scopes the rules query to a specific principal when set, while preserving byte-identical behavior when unset.

## Why

Sprint 06b · PRD UC-MGMT-05 · capability CAP-AUTHZ-01. Mounting RulesList with principalId='agent:codex-staging' calls rulesService.principalRules(projectId, 'agent:codex-staging') and renders only that principal's rules. Mounting without principalId calls rulesService.workspaceRules(projectId)

## How to verify

PRIMARY **AC-1** — `pnpm test:ct:desktop -- RulesListPrincipalIdScoped`: principalId set scopes query to that principal's rules only. Full gate set in the spec below.

## Scope

  - apps/desktop/src/components/rules/RulesList.svelte (MODIFY — add optional principalId prop + conditional query branch ONLY)
  - apps/desktop/tests/governance/RulesListPrincipalId.spec.ts (NEW — CT specs)
  - apps/desktop/src/components/settings/GovernanceSettings.svelte (MODIFY — add no-principal empty state for Rules tab: when principalId is undefined in the Rules tab context, render EmptyStatePlaceholder instead of RulesList; this is the SOLE additional change beyond MGMT-UI-004's ErrorBoundary wrap)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: MGMT-UI-010 — Extend RulesList with optional principalId prop (backward compatible)
================================================================================

TASK_TYPE:   FEATURE
STATUS:      Backlog
PRIORITY:    P1
EFFORT:      S  (45 min)
AGENT:       sveltekit-implementer
PROPOSED-BY: sveltekit-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-MGMT-05
CAPABILITIES:CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  pnpm test:ct:desktop -- RulesListPrincipalIdScoped
  check: pnpm -F @gitbutler/desktop check
  lint:  pnpm lint

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Mounting RulesList with principalId='agent:codex-staging' calls rulesService.principalRules(projectId, 'agent:codex-staging') and renders only that principal's rules. Mounting without principalId calls rulesService.workspaceRules(projectId) and renders identically to today. Rule/RuleEditor render unchanged in both cases. pnpm test:ct:desktop -- RulesListPrincipalId passes. pnpm -F @gitbutler/desktop check and pnpm lint pass.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] principalId is strictly OPTIONAL (principalId?: string) — existing callers of RulesList that pass only projectId must continue to work without any change.
- [MUST] When principalId is unset, the exact same rulesService.workspaceRules(projectId) call is made as today — zero regression to existing workspace rules surface.
- [MUST] When principalId is set, a distinct scoped query (from MGMT-BE-003) is called instead; the result feeds the same rendering path.
- [MUST] Rule, RuleEditor, RuleFiltersEditor, and NewRuleMenu receive the same props as today and render unchanged.
- [MUST] The rulesService.principalRules method MUST return the same Redux EntityState<WorkspaceRule> shape as rulesService.workspaceRules so the existing ReduxResult wrapper and workspaceRulesSelectors.selectAll chain work unchanged. MGMT-BE-003 must document and enforce this shape contract.
- [NEVER] NEVER modify Rule.svelte, RuleEditor.svelte, RuleFiltersEditor.svelte, or NewRuleMenu.svelte — they are read-only in this task.
- [NEVER] NEVER change the behavior of the unset-principalId code path in any way.
- [NEVER] NEVER add +page.server.ts.
- [NEVER] NEVER introduce a module-level store.
- [STRICTLY] The prop is a purely additive API change — no breaking change to the existing Props type.
- [STRICTLY] No relative imports — @gitbutler/ package references. No console.log.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: principalId set scopes query to that principal's rules only
- [ ] AC-2: principalId unset — behavior identical to existing workspace rules surface
- [ ] AC-3: Rule/RuleEditor render unchanged in scoped context
- [ ] AC-4: Empty/placeholder state when no rules for principal
- [ ] AC-5: Rules tab 'no principal selected' empty state renders in GovernanceSettings when principalId is absent (DESIGN-MGMT-006 AC-2 case a)
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: principalId set scopes query to that principal's rules only
  GIVEN: RulesList mounted with projectId and principalId='agent:codex-staging', seeded_principal_a_rules fixture
  WHEN:  the component renders
  THEN:  rulesService.principalRules is called with ('agent:codex-staging') and exactly 2 rule rows render (rule-A1, rule-A2); rule-B1 and rule-B2 are absent
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- RulesListPrincipalIdScoped

AC-2: principalId unset — behavior identical to existing workspace rules surface
  GIVEN: RulesList mounted with only projectId (no principalId), seeded_workspace_rules fixture
  WHEN:  the component renders
  THEN:  rulesService.workspaceRules is called and all 4 rules render; behavior is byte-identical to the existing surface
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- RulesListWorkspaceUnchanged

AC-3: Rule/RuleEditor render unchanged in scoped context
  GIVEN: RulesList mounted with principalId='agent:codex-staging', seeded_principal_a_rules (2 rules with edit actions)
  WHEN:  user clicks edit on rule-A1
  THEN:  RuleEditor slides in for rule-A1 with the same props and behavior as in the unscoped workspace-rules surface
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- RulesListEditorUnchanged

AC-4: Empty/placeholder state when no rules for principal
  GIVEN: RulesList mounted with principalId='agent:empty-bot', seeded_no_rules_for_principal
  WHEN:  the component renders
  THEN:  the existing empty/placeholder state renders (the same placeholder text as in the workspace-rules surface with 0 rules)
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- RulesListPrincipalEmpty

AC-5: Rules tab 'no principal selected' empty state renders in GovernanceSettings when principalId is absent (DESIGN-MGMT-006 AC-2 case a)
  GIVEN: GovernanceSettings mounted with the Rules tab active and no principalId selected (principalId=undefined in the Rules tab context)
  WHEN:  the Rules tab renders
  THEN:  an EmptyStatePlaceholder with title 'Select a principal to view their rules' renders; RulesList is NOT mounted (0 RulesList elements)
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- GovernanceRulesTabNoPrincipal

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): principalId='agent:codex-staging' -> principalRules called; exactly 2 rules (A1+A2) render; switching to 'agent:cursor-bot' -> 1 rule (B1) renders
    VERIFY: pnpm test:ct:desktop -- RulesListPrincipalIdScoped
- TC-2 (-> AC-2): No principalId -> workspaceRules called; all 4 rules render unchanged
    VERIFY: pnpm test:ct:desktop -- RulesListWorkspaceUnchanged
- TC-3 (-> AC-3): RuleEditor renders for a scoped rule with the same props as the workspace surface
    VERIFY: pnpm test:ct:desktop -- RulesListEditorUnchanged
- TC-4 (-> AC-4): 0 rules for a principal -> the existing empty/placeholder state renders
    VERIFY: pnpm test:ct:desktop -- RulesListPrincipalEmpty
- TC-5 (-> AC-5): Rules tab with no principalId selected: EmptyStatePlaceholder with 'Select a principal to view their rules'; 0 RulesList mounted
    VERIFY: pnpm test:ct:desktop -- GovernanceRulesTabNoPrincipal

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides:
  - a
  - p
  - p
  - s
  - /
  - d
  - e
  - s
  - k
  - t
  - o
  - p
  - /
  - s
  - r
  - c
  - /
  - c
  - o
  - m
  - p
  - o
  - n
  - e
  - n
  - t
  - s
  - /
  - r
  - u
  - l
  - e
  - s
  - /
  - R
  - u
  - l
  - e
  - s
  - L
  - i
  - s
  - t
  - .
  - s
  - v
  - e
  - l
  - t
  - e
  -  
  - w
  - i
  - t
  - h
  -  
  - a
  - n
  -  
  - o
  - p
  - t
  - i
  - o
  - n
  - a
  - l
  -  
  - p
  - r
  - i
  - n
  - c
  - i
  - p
  - a
  - l
  - I
  - d
  - ?
  - :
  -  
  - s
  - t
  - r
  - i
  - n
  - g
  -  
  - p
  - r
  - o
  - p
  - :
  -  
  - w
  - h
  - e
  - n
  -  
  - s
  - e
  - t
  - ,
  -  
  - t
  - h
  - e
  -  
  - r
  - u
  - l
  - e
  - s
  - S
  - e
  - r
  - v
  - i
  - c
  - e
  -  
  - q
  - u
  - e
  - r
  - y
  -  
  - i
  - s
  -  
  - s
  - c
  - o
  - p
  - e
  - d
  -  
  - t
  - o
  -  
  - t
  - h
  - a
  - t
  -  
  - p
  - r
  - i
  - n
  - c
  - i
  - p
  - a
  - l
  -  
  - (
  - v
  - i
  - a
  -  
  - t
  - h
  - e
  -  
  - M
  - G
  - M
  - T
  - -
  - B
  - E
  - -
  - 0
  - 0
  - 3
  -  
  - p
  - r
  - i
  - n
  - c
  - i
  - p
  - a
  - l
  - I
  - d
  - -
  - s
  - c
  - o
  - p
  - e
  - d
  -  
  - b
  - a
  - c
  - k
  - e
  - n
  - d
  -  
  - q
  - u
  - e
  - r
  - y
  - )
  - ;
  -  
  - w
  - h
  - e
  - n
  -  
  - u
  - n
  - s
  - e
  - t
  - ,
  -  
  - b
  - e
  - h
  - a
  - v
  - i
  - o
  - r
  -  
  - i
  - s
  -  
  - b
  - y
  - t
  - e
  - -
  - i
  - d
  - e
  - n
  - t
  - i
  - c
  - a
  - l
  -  
  - t
  - o
  -  
  - t
  - o
  - d
  - a
  - y
  -  
  - (
  - b
  - a
  - c
  - k
  - w
  - a
  - r
  - d
  -  
  - c
  - o
  - m
  - p
  - a
  - t
  - i
  - b
  - l
  - e
  - )
  - .
  -  
  - T
  - h
  - i
  - s
  -  
  - i
  - s
  -  
  - t
  - h
  - e
  -  
  - S
  - O
  - L
  - E
  -  
  - c
  - h
  - a
  - n
  - g
  - e
  -  
  - t
  - o
  -  
  - t
  - h
  - e
  -  
  - r
  - u
  - l
  - e
  - s
  - /
  -  
  - d
  - i
  - r
  - e
  - c
  - t
  - o
  - r
  - y
  - .
consumes:
  - MGMT-BE-003 (principalId-scoped rules query — a new backend endpoint / rulesService method; this task wires the prop to it)
  - MGMT-UI-001 (desktop CT harness)
  - apps/desktop/src/components/rules/RulesList.svelte (current: takes only projectId, queries rulesService.workspaceRules(projectId))
  - apps/desktop/src/lib/rules/rulesService.svelte.ts (current workspaceRules method; MGMT-BE-003 adds a scoped variant)
boundary_contracts:
  - When principalId is unset (undefined), RulesList.svelte MUST behave byte-identically to the current implementation: same rulesService.workspaceRules(projectId) call, same rendering, same empty state. Zero regression.
  - When principalId is set, the query switches to the scoped method provided by MGMT-BE-003 (e.g. rulesService.principalRules(projectId, principalId)). The Rule/RuleEditor/RuleFiltersEditor/NewRuleMenu components receive the same projectId prop and render unchanged.
  - This component is the SPEC-SANCTIONED seam (B14): the CT mocks the rulesService at the but-sdk layer and asserts the correct query method is called. The real principalId scoping is proven by MGMT-BE-003's Rust integration tests.
  - SOLE CHANGE to rules/ directory: no other rules/ files are modified.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - apps/desktop/src/components/rules/RulesList.svelte (MODIFY — add optional principalId prop + conditional query branch ONLY)
  - apps/desktop/tests/governance/RulesListPrincipalId.spec.ts (NEW — CT specs)
  - apps/desktop/src/components/settings/GovernanceSettings.svelte (MODIFY — add no-principal empty state for Rules tab: when principalId is undefined in the Rules tab context, render EmptyStatePlaceholder instead of RulesList; this is the SOLE additional change beyond MGMT-UI-004's ErrorBoundary wrap)
writeProhibited:
  - apps/desktop/src/components/rules/Rule.svelte — read-only; no prop added
  - apps/desktop/src/components/rules/RuleEditor.svelte — read-only; unchanged
  - apps/desktop/src/components/rules/RuleFiltersEditor.svelte — read-only; unchanged
  - apps/desktop/src/components/rules/NewRuleMenu.svelte — read-only; unchanged
  - apps/desktop/src/lib/rules/rulesService.svelte.ts — the new principalRules method is added by MGMT-BE-003; this task only wires the call
  - Any +page.server.ts
  - packages/but-sdk/src/generated

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. apps/desktop/src/components/rules/RulesList.svelte [1-110] — PRIMARY PATTERN — current Props type (line 20-24), workspaceRules query (line 40), rendering path; understand exactly where to add the optional principalId branch without touching Rule/RuleEditor.
2. apps/desktop/src/lib/rules/rulesService.svelte.ts [1-41] — Current workspaceRules method (line 31) — this is what gets a scoped sibling method from MGMT-BE-003; read the existing pattern to understand how to wire the new method.
3. apps/desktop/src/components/rules/Rule.svelte [1-30] — Rule props — confirm it takes only projectId + rule (no principalId) so the pass-through is correct and no prop leaks.
4. .spec/prds/governance/08-uc-mgmt.md [120-128] — UC-MGMT-05 acceptance criteria — the exact backward-compat + scoping contract this task closes.
5. .spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md [99-110] — Rules tab wireframe — left principal selector panel + right RulesList panel; understand the layout context BranchGatesList lives beside.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- pnpm test:ct:desktop -- RulesListPrincipalIdScoped   -> Exit 0
- pnpm test:ct:desktop -- RulesListWorkspaceUnchanged   -> Exit 0
- pnpm test:ct:desktop -- RulesListEditorUnchanged   -> Exit 0
- pnpm test:ct:desktop -- RulesListPrincipalEmpty   -> Exit 0
- pnpm -F @gitbutler/desktop check   -> Exit 0
- pnpm lint   -> Exit 0
- git diff --name-only | grep 'rules/' | grep -v 'RulesList.svelte' | wc -l | grep '^0$'   -> Exit 0 (only RulesList.svelte touched in rules/)
- pnpm test:ct:desktop -- GovernanceRulesTabNoPrincipal   -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references:
  - DESIGN-MGMT-006 (empty state for Rules tab when no principal selected or no rules)
  - DESIGN-MGMT-006 AC-2: Rules tab two empty-state sub-cases: (a) no principal selected → EmptyStatePlaceholder title='Select a principal to view their rules', no action button; (b) principal selected, no rules → defer to RulesList built-in empty state (existing component owns this — do not override)
  - DESIGN-MGMT-003 (Sprint 06a): isReadOnly=true prop from GovernanceSettings.svelte propagates to the Rules tab; the principalId-scoped RulesList must pass isReadOnly or disabled context to suppress write affordances when the viewer lacks administration:write
  - .spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md Rules tab wireframe (lines 99-110): left-column principal picker + right-column RulesList panel; the sole change to rules/ components is the optional principalId prop on RulesList.svelte
notes:
  - RulesList.svelte gains: `type Props = { projectId: string; principalId?: string }`. In the query derivation: `const rules = $derived(principalId ? rulesService.principalRules(projectId, principalId) : rulesService.workspaceRules(projectId))`. All downstream rendering (Drawer, RuleEditor, Rule) is unchanged.
  - The Rules tab introduces no new visual states beyond the two empty-state sub-cases and the existing RulesList states — the design contract for Rules is minimal by design (reuse-only)
  - The principal picker (left column) is a list of principals; selecting one sets the principalId prop on RulesList; deselecting or loading with no principal shows the no-principal-selected empty state
  - RulesList with principalId=undefined must behave byte-identically to the existing workspace-rules surface — the SOLE change is adding the optional prop and scoping the query when it is set
pattern: Optional prop with conditional query selection — a single $derived expression switching on the prop's presence. All rendering downstream is unchanged.
pattern_source: apps/desktop/src/components/rules/RulesList.svelte (current workspaceRules derivation at line 40)
anti_pattern: Modifying Rule/RuleEditor/RuleFiltersEditor; adding principalId to inner components; changing the unset-prop code path in any way

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agent: sveltekit-implementer
rationale: Surgical MODIFY to one existing component (RulesList.svelte) — add one optional prop and a conditional query branch. Existing Rule/RuleEditor/RuleFiltersEditor/NewRuleMenu components are read-only. sveltekit-implementer owns adapter-static component work.
coding_standards: No relative imports — @gitbutler/ package references, Prettier: tabs, double quotes, no trailing commas, 100-col, No console.log, Svelte 5 $props()/$derived() rune syntax, principalId?: string — strictly optional, no required-ness added, CT describe blocks MUST use the component name as the outermost describe string (e.g. describe('RulesList', () => {...})) so `pnpm test:ct:desktop -- <ComponentName>` grep matches reliably.

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: MGMT-BE-003 (principalId-scoped rules query — the backend method RulesList will call when principalId is set); MGMT-UI-001 (desktop CT harness; from Sprint 06a)
Blocks:     MGMT-UI-012 (build-gate tests verify the rules/ sole-change invariant)
```
</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "MGMT-UI-010",
  "proposed_by": "sveltekit-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "seeded_principal_a_rules": {
      "description": "rulesService.principalRules SDK mock returns 2 rules for principalId='agent:codex-staging' and 0 rules for any other principal. rulesService.workspaceRules returns 4 rules (the full workspace set).",
      "seed_method": "ui_flow",
      "records": [
        "principalId='agent:codex-staging' -> 2 rules: rule-A1, rule-A2",
        "workspaceRules -> 4 rules: rule-A1, rule-A2, rule-B1, rule-B2"
      ]
    },
    "seeded_principal_b_rules": {
      "description": "rulesService.principalRules SDK mock returns 1 rule for principalId='agent:cursor-bot' and 0 for 'agent:codex-staging'. workspaceRules returns 4 rules.",
      "seed_method": "ui_flow",
      "records": [
        "principalId='agent:cursor-bot' -> 1 rule: rule-B1",
        "workspaceRules -> 4 rules"
      ]
    },
    "seeded_workspace_rules": {
      "description": "rulesService.workspaceRules SDK mock returns 4 rules. No principalId prop passed.",
      "seed_method": "ui_flow",
      "records": [
        "workspaceRules -> 4 rules: rule-A1, rule-A2, rule-B1, rule-B2"
      ]
    },
    "seeded_no_rules_for_principal": {
      "description": "rulesService.principalRules returns 0 rules for principalId='agent:empty-bot'.",
      "seed_method": "ui_flow",
      "records": [
        "principalId='agent:empty-bot' -> 0 rules"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN RulesList mounted with projectId and principalId='agent:codex-staging', seeded_principal_a_rules fixture WHEN the component renders THEN rulesService.principalRules is called with ('agent:codex-staging') and exactly 2 rule rows render (rule-A1, rule-A2); rule-B1 and rule-B2 are absent",
      "verify": "pnpm test:ct:desktop -- RulesListPrincipalIdScoped",
      "scenario": {
        "id": "SC-MGMT-UI-010-1",
        "primary": true,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "`rulesService.workspaceRules` is called instead of `principalRules` (all `4` rules would render, not `2`)",
            "principalId prop is ignored and all `4` rules render (static, disconnected from prop)",
            "the `principalRules` call is a no-op stub returning `empty` (`0` rules shown)"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_principal_a_rules",
            "action": {
              "actor": "user",
              "steps": [
                "mount RulesList with projectId and principalId='agent:codex-staging'",
                "observe the rendered rule rows"
              ]
            },
            "end_state": {
              "must_observe": [
                "exactly `2` rule rows visible (`rule-A1` and `rule-A2`)",
                "the `rulesService.principalRules` spy called `== 1` time with args `(projectId, 'agent:codex-staging')` and NO call to `rulesService.workspaceRules`"
              ],
              "must_not_observe": [
                "`4` rule rows (workspace rules leaking into the scoped view)",
                "`rulesService.workspaceRules` called when `principalId` is set (`0` such calls expected)",
                "`0` rules when seeded with `2` (stub returning empty)"
              ]
            }
          },
          {
            "start_ref": "seeded_principal_b_rules",
            "action": {
              "actor": "user",
              "steps": [
                "mount RulesList with projectId and principalId='agent:cursor-bot'",
                "observe the rendered rule rows"
              ]
            },
            "end_state": {
              "must_observe": [
                "exactly `1` rule row visible (`rule-B1`)",
                "`rule-A1` and `rule-A2` absent (`0` rows for those rule ids)"
              ],
              "must_not_observe": [
                "`rule-A1` or `rule-A2` in the list for `'agent:cursor-bot'` (`0` such rows expected)",
                "`4` rules rendered for the scoped view (workspace rules leaking)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN RulesList mounted with only projectId (no principalId), seeded_workspace_rules fixture WHEN the component renders THEN rulesService.workspaceRules is called and all 4 rules render; behavior is byte-identical to the existing surface",
      "verify": "pnpm test:ct:desktop -- RulesListWorkspaceUnchanged",
      "scenario": {
        "id": "SC-MGMT-UI-010-2",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "`rulesService.principalRules` is called even when `principalId` is unset (wrong query path; `0` principalRules calls expected)",
            "fewer than `4` rules render (backward-compat regression, e.g. `0` or `2` rules shown)",
            "`rulesService.workspaceRules` is not called (stub; call count `0`)"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_workspace_rules",
            "action": {
              "actor": "user",
              "steps": [
                "mount RulesList with only projectId (principalId not passed)",
                "observe the rendered rule rows"
              ]
            },
            "end_state": {
              "must_observe": [
                "exactly `4` rule rows visible (`rule-A1`, `rule-A2`, `rule-B1`, `rule-B2`)",
                "`rulesService.workspaceRules` spy called `== 1` time with `{projectId}` and `0` calls to `rulesService.principalRules`"
              ],
              "must_not_observe": [
                "fewer than `4` rules rendered (`0`, `2`, or `3` rules shown \u2014 regression)",
                "`rulesService.principalRules` called when `principalId` is `undefined` (`0` such calls expected)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN RulesList mounted with principalId='agent:codex-staging', seeded_principal_a_rules (2 rules with edit actions) WHEN user clicks edit on rule-A1 THEN RuleEditor slides in for rule-A1 with the same props and behavior as in the unscoped workspace-rules surface",
      "verify": "pnpm test:ct:desktop -- RulesListEditorUnchanged",
      "scenario": {
        "id": "SC-MGMT-UI-010-3",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "RuleEditor does not render when `principalId` is set (regression \u2014 static shell missing the editor slot)",
            "RuleEditor receives different props in the scoped context (e.g. receives `principalId` it does not accept, or `projectId` is absent)",
            "the edit button is absent or `disabled` in the scoped rules list (`0` edit buttons rendered)"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_principal_a_rules",
            "action": {
              "actor": "user",
              "steps": [
                "mount RulesList with principalId='agent:codex-staging'",
                "click the edit action on rule-A1"
              ]
            },
            "end_state": {
              "must_observe": [
                "the RuleEditor slide-in panel present with a heading or label identifying `rule-A1`",
                "RuleEditor receives `projectId` prop equal to the mounted `projectId` value (spy on RuleEditor props confirms `principalId` prop is `absent`/`undefined` \u2014 not passed to RuleEditor)",
                "the ReduxResult component renders without error in the scoped case (no selector shape mismatch \u2014 `1` ReduxResult component in non-error state)"
              ],
              "must_not_observe": [
                "an error or `empty` RuleEditor when in scoped mode (`0` RuleEditor elements rendered)",
                "RuleEditor receiving a `principalId` prop (it does not accept one \u2014 `0` such prop assignments)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN RulesList mounted with principalId='agent:empty-bot', seeded_no_rules_for_principal WHEN the component renders THEN the existing empty/placeholder state renders (the same placeholder text as in the workspace-rules surface with 0 rules)",
      "verify": "pnpm test:ct:desktop -- RulesListPrincipalEmpty",
      "scenario": {
        "id": "SC-MGMT-UI-010-4",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "rule rows render when the principal has `0` rules (stub returning stale data \u2014 static rows)",
            "the empty state is absent (component renders nothing, no placeholder, `0` placeholder elements)"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_no_rules_for_principal",
            "action": {
              "actor": "user",
              "steps": [
                "mount RulesList with principalId='agent:empty-bot'",
                "observe the rendered state"
              ]
            },
            "end_state": {
              "must_observe": [
                "the rules-placeholder element with text `'Let rules automatically sort your changes'` (or the equivalent empty-state text from the existing workspace-rules surface) rendered",
                "`0` rule rows in the DOM"
              ],
              "must_not_observe": [
                "rule rows rendered when the principal has `0` rules",
                "an unhandled error when `principalRules` returns `empty` (`0` error elements)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN GovernanceSettings mounted with the Rules tab active and no principalId selected (principalId=undefined in the Rules tab context) WHEN the Rules tab renders THEN an EmptyStatePlaceholder with title 'Select a principal to view their rules' renders; RulesList is NOT mounted (0 RulesList elements)",
      "verify": "pnpm test:ct:desktop -- GovernanceRulesTabNoPrincipal",
      "scenario": {
        "id": "SC-MGMT-UI-010-5",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "EmptyStatePlaceholder absent when no principalId (static shell renders RulesList unconditionally \u2014 `1` RulesList element when none expected)",
            "placeholder text hardcoded differently than 'Select a principal to view their rules' (wrong message)",
            "RulesList mounted and visible when principalId is undefined (missing guard in GovernanceSettings Rules tab host)"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_workspace_rules",
            "action": {
              "actor": "user",
              "steps": [
                "mount GovernanceSettings with the Rules tab active",
                "ensure no principalId is selected (principalId=undefined)",
                "observe the Rules tab content"
              ]
            },
            "end_state": {
              "must_observe": [
                "EmptyStatePlaceholder containing text `'Select a principal to view their rules'`",
                "`0` RulesList elements mounted in the DOM"
              ],
              "must_not_observe": [
                "RulesList mounted when no principalId selected",
                "empty DOM with no placeholder (neither RulesList nor EmptyStatePlaceholder)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "principalId='agent:codex-staging' -> principalRules called; exactly 2 rules (A1+A2) render; switching to 'agent:cursor-bot' -> 1 rule (B1) renders",
      "verify": "pnpm test:ct:desktop -- RulesListPrincipalIdScoped",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "No principalId -> workspaceRules called; all 4 rules render unchanged",
      "verify": "pnpm test:ct:desktop -- RulesListWorkspaceUnchanged",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "RuleEditor renders for a scoped rule with the same props as the workspace surface",
      "verify": "pnpm test:ct:desktop -- RulesListEditorUnchanged",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "0 rules for a principal -> the existing empty/placeholder state renders",
      "verify": "pnpm test:ct:desktop -- RulesListPrincipalEmpty",
      "maps_to_ac": "AC-4"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "Rules tab with no principalId selected: EmptyStatePlaceholder with 'Select a principal to view their rules'; 0 RulesList mounted",
      "verify": "pnpm test:ct:desktop -- GovernanceRulesTabNoPrincipal",
      "maps_to_ac": "AC-5"
    }
  ]
}
-->
