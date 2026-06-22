# DESIGN-LPR-002: Principal `kind` display + editing contract — Governance Principals tab

> Status: ✅ Completed (design contract)
> Commit: e09a46e36e
> Reviewer: deferred to PHASE 4.5 red-hat closeout — design contract committed prior session
> Updated: 2026-06-22T18:07:12Z

## What this does

Specify exactly how a principal's `kind = "agent" | "human"` is displayed and edited in the Governance Principals tab (the 6a surface, `PrincipalsList.svelte`). The contract defines the badge shape for read display, the select/input for editing, the default/omitted treatment (omitted `kind` = human), and the honest framing: `kind` is a descriptor that drives the agent-authored tag derivation — it is NOT an enforcement key and it does not change any gate decision.

## Why

Sprint 07 · PRD UC-LPR-04, UC-LPR-05 · capability CAP-AUTHZ-01. The additive `kind: Option<String>` field on `PrincipalWire` (LPR-005 / tech-delta §A.4) is rendered in the Principals tab so operators can see and set whether a principal is declared `agent` or `human`. Without this design contract, the sveltekit-implementer must decide badge shape, select options, the omitted-kind default display, and the honest non-enforcement framing independently — this contract pins all of those.

## How to verify

PRIMARY **AC-1** — `design review — reviewer confirms the DESIGN-ANNOTATIONS.md Sprint 07 LPR section carries: the badge component + its two variants (agent / human), the default display for omitted kind (human badge, not blank), the select or chip options for editing, and the non-enforcement disclosure`: Principal kind display + edit contract [PRIMARY]. Full gate set in the spec below.

## Scope

- apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md (MODIFY — extend with Sprint 07 LPR section covering principal kind display + editing in the Principals tab)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: DESIGN-LPR-002 — Principal kind display + editing design contract
================================================================================

TASK_TYPE:   DESIGN
STATUS:      Backlog
PRIORITY:    P1
EFFORT:      S  (30 min)
AGENT:       frontend-designer
PROPOSED-BY: frontend-designer
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-LPR-04, UC-LPR-05
CAPABILITIES:CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  pnpm test:ct:desktop -- GovernancePrincipalKind (exercised by the sveltekit-implementer's component test for PrincipalsList)

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
A sveltekit-implementer reading this contract knows: (a) which component to use for the kind badge in read mode, (b) which component to use for the kind selector in edit mode, (c) that omitted `kind` renders as "Human" (the conservative default-human posture), (d) that the badge is informational only — it must never imply enforcement or access control, and (e) the honest non-enforcement disclaimer that the label does not change gate decisions.

--------------------------------------------------------------------------------
CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST specify the `kind` display as a small inline Badge (from `@gitbutler/ui`) with two variants: 'Agent' (neutral/secondary style) and 'Human' (neutral/secondary style). Both variants use neutral styling — no green/red/success/error color that implies enabled/disabled or trusted/untrusted. This is a descriptor label, not a status.
- [MUST] MUST specify that an omitted `kind` (None / undefined) renders as the 'Human' badge — the conservative default-human posture mirrors the backend: absence means human, never agent. The badge must NOT be blank when kind is absent.
- [MUST] MUST specify the edit control: a two-option Select (or segmented chip pair) with values 'agent' | 'human', placed beside the principal's id in the principal list row, visible when the row is in edit mode. Use `@gitbutler/ui` `Select` + `SelectItem` (the same pattern `ForgeForm.svelte` uses for the forge override selector).
- [MUST] MUST specify the location within the Principals tab: the kind badge renders inline in the principal row, after the principal id, before the role/groups summary. In edit mode it becomes the select control.
- [MUST] MUST specify the non-enforcement disclosure as a tooltip or caption: 'This label identifies the principal as an agent or human for tagging purposes. It does not change any permission grant or gate decision.' The disclosure must appear on the badge (tooltip) or below the select in edit mode (caption).
- [MUST] MUST confirm the Badge is read-only by default and the select is only rendered when the row is in edit mode (the existing principal row edit mode in PrincipalsList — do not introduce a new edit mode).
- [MUST] MUST confirm no new design-system token is introduced — use only existing `@gitbutler/ui` CSS variables and Badge/Select components.
- [NEVER] NEVER use success (green) or error (red) Badge styling for kind. 'Agent' and 'Human' are peer-level descriptors, not statuses. A green 'Agent' badge implies trust privilege; a red 'Human' badge implies restriction. Neither is correct — `kind` changes no enforcement.
- [NEVER] NEVER describe `kind` as an authorization key, permission grant, or access control in the copy or tooltip. The tech-delta §A.4 states explicitly: the `kind` field does NOT enter `GovConfig.principals` (the enforcement map) and NO gate reads it.
- [NEVER] NEVER introduce a new Badge or Select component. The `@gitbutler/ui` `Badge` and `Select` + `SelectItem` components cover this.
- [NEVER] NEVER render the badge blank when `kind` is absent — the omitted case must show 'Human' (the conservative default).
- [STRICTLY] STRICTLY use neutral/secondary Badge styling for both variants so neither 'Agent' nor 'Human' reads as "better" or "worse" — they are peer descriptors.
- [STRICTLY] STRICTLY model the select on `apps/desktop/src/components/projectSettings/ForgeForm.svelte` `Select` + `SelectItem` pattern — same component imports, same option shape.
- [STRICTLY] STRICTLY the non-enforcement tooltip/caption must be present on both the read badge (tooltip) and the edit select (caption). An implementer reading only the badge variant must still see the non-enforcement disclosure.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [x] AC-1 [PRIMARY]: Principal kind display + edit contract
- [x] AC-2: Omitted kind default display (human, not blank)
- [x] AC-3: Non-enforcement disclosure (tooltip on badge, caption on select)
- [x] AC-4: Edit mode integration with existing principal row edit mode
- [x] AC-5: No new design-system tokens; neutral badge styling
- [x] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: Principal kind display + edit contract
  GIVEN: the Sprint 07 LPR section of DESIGN-ANNOTATIONS.md covers the principal kind display in the Principals tab
  WHEN:  a reviewer inspects the kind display specification
  THEN:  it specifies: an inline `@gitbutler/ui` `Badge` in neutral/secondary style with text 'Agent' or 'Human' (no success/error color); placed after the principal id in the principal row, before the role/groups summary; in edit mode, a `@gitbutler/ui` `Select` with two `SelectItem` values ('agent' → 'Agent', 'human' → 'Human') replaces or supplements the badge; select placement is beside the principal id field in the edit row
  TEST_TIER: component   VERIFICATION_SERVICE: apps/desktop CT harness (pnpm test:ct:desktop) — component test asserts the Badge renders for each kind variant and the select shows in edit mode
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test
  VERIFY: design review — reviewer confirms Badge component path + two variants (Agent/Human) + neutral styling + Select in edit mode + placement are all specified

AC-2: Omitted kind default display (human, not blank)
  GIVEN: the contract specifies the rendering for an omitted kind field
  WHEN:  a reviewer checks the omitted-kind treatment
  THEN:  it states: when the principal's committed `.gitbutler/permissions.toml` entry has no `kind` field (or the SDK returns `kind: undefined/null`), the Badge renders 'Human' — the conservative default-human posture; the badge is NEVER blank; this matches the backend: absence of `kind` = human (not agent), per tech-delta §A.4 "omitted kind defaults to human"
  TEST_TIER: component   VERIFICATION_SERVICE: apps/desktop CT harness — component test asserts Badge renders 'Human' when kind is undefined
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review
  VERIFY: design review — reviewer confirms omitted-kind → 'Human' badge rule is stated; blank-badge anti-pattern is explicitly excluded

AC-3: Non-enforcement disclosure (tooltip on badge, caption on select)
  GIVEN: the contract specifies the non-enforcement disclosure
  WHEN:  a reviewer inspects the tooltip and caption specifications
  THEN:  it states: the Badge has a tooltip with text 'This label identifies the principal as an agent or human for tagging purposes. It does not change any permission grant or gate decision.'; in edit mode the Select has a caption (below the control) with the same disclosure; the disclosure must NOT imply the kind field grants/denies anything
  TEST_TIER: component   VERIFICATION_SERVICE: apps/desktop CT harness — component test asserts tooltip content on the Badge
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review
  VERIFY: design review — reviewer confirms both tooltip (badge) and caption (select) carry the non-enforcement disclosure verbatim

AC-4: Edit mode integration with existing principal row edit mode
  GIVEN: the contract specifies edit mode behavior
  WHEN:  a reviewer checks how the kind select integrates with the existing principal row edit mode in PrincipalsList
  THEN:  it states: the kind Select is rendered ONLY when the principal row is in its existing edit mode (the same mode the implementer already uses for editing principal id/role/groups — no new edit mode is introduced); in read mode the Badge is shown; the select value binds to the committed kind and persists via the same project-settings write path that saves other principal fields; the implementer must NOT introduce a separate inline-edit toggle just for kind
  TEST_TIER: component   VERIFICATION_SERVICE: apps/desktop CT harness — component test asserts Select is absent in read mode and present in edit mode
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review
  VERIFY: design review — reviewer confirms edit-mode integration with the existing PrincipalsList row edit mode is named; separate inline-edit anti-pattern is excluded

AC-5: No new design-system tokens; neutral badge styling
  GIVEN: the Sprint 07 LPR section of DESIGN-ANNOTATIONS.md covers the kind badge
  WHEN:  a reviewer audits every visual reference in the kind display section
  THEN:  every visual attribute uses an existing CSS variable or defers to the component's own stylesheet — no hex literals, no new var(--kind-*)/var(--agent-*) tokens; both Badge variants (Agent / Human) use neutral/secondary styling — neither uses success (green), error (red), warning (yellow), or info (blue) that implies status
  TEST_TIER: component   VERIFICATION_SERVICE: grep audit on the annotation file
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by static grep
  VERIFY: grep -nE '#[0-9a-fA-F]{3,6}|var\(--(kind|agent)-' apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md -> zero matches in the LPR principal kind section

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): contract names @gitbutler/ui Badge in neutral/secondary style with 'Agent' and 'Human' variants; @gitbutler/ui Select + SelectItem with 'agent'/'human' options in edit mode; placement after principal id in the row
    VERIFY: design review of the kind display specification in DESIGN-ANNOTATIONS.md
- TC-2 (-> AC-2): contract states omitted kind renders as 'Human' badge (not blank); blank-badge anti-pattern explicitly excluded; matches backend default-human posture
    VERIFY: design review of the omitted-kind treatment in DESIGN-ANNOTATIONS.md
- TC-3 (-> AC-3): contract carries the non-enforcement tooltip text on the badge and the caption text below the select edit control
    VERIFY: design review of the tooltip and caption specifications in DESIGN-ANNOTATIONS.md
- TC-4 (-> AC-4): contract states Select renders only in the existing row edit mode; Badge renders in read mode; no separate inline-edit toggle introduced for kind alone
    VERIFY: design review of the edit-mode integration section in DESIGN-ANNOTATIONS.md
- TC-5 (-> AC-5): zero hex color literals and zero new var(--kind-*)/var(--agent-*) tokens; neutral/secondary Badge styling confirmed for both variants
    VERIFY: grep -nE '#[0-9a-fA-F]{3,6}|var\\(--(kind|agent)-' apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md -> zero matches

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md (MODIFY — extend with Sprint 07 LPR section for principal kind display + editing)
writeProhibited:
  - apps/desktop/src/components/governance/PrincipalsList.svelte — read only for pattern reference; do not modify
  - apps/desktop/src/components/projectSettings/ForgeForm.svelte — read only for Select component pattern reference; do not modify
  - packages/ui/src/lib/components/** — no design-system changes
  - any .svelte or .ts implementation file — design spec artifact only

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. apps/desktop/src/components/governance/PrincipalsList.svelte [1-end] — [PRIMARY PATTERN] the existing Principals tab component — read to understand the current principal row structure (id, role, groups), the edit mode toggle, and where the kind badge/select slots in
2. apps/desktop/src/components/projectSettings/ForgeForm.svelte [19-36] — the Select + SelectItem pattern (FORGE_OPTIONS shape) — mirror for the kind select (values: 'agent'/'human', labels: 'Agent'/'Human')
3. .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md §A.4 — the kind field spec: optional on PrincipalWire, omitted = human (conservative default), does NOT enter GovConfig.principals, no gate reads it, read at the target ref — the source for the non-enforcement disclosure
4. .spec/prds/governance/tasks/sprint-07-local-agent-pr/LPR-005-derived-pr-lifecycle-agent-tag.md — the backend task: the kind field on PrincipalWire (config.rs:424), the additive optional field, the agent-authored tag derivation it drives
5. .spec/prds/governance/tasks/sprint-06a-governance-ui-scaffold-principals-groups/ — Sprint 06a principals tab annotations: understand the existing PrincipalsList design context this task extends

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- design review of apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md Sprint 07 LPR principal kind section   -> Badge (neutral/secondary, Agent/Human variants) + omitted-kind=Human rule + Select in edit mode + non-enforcement tooltip/caption + edit-mode integration — all present
- grep -nE '#[0-9a-fA-F]{3,6}|var\(--(kind|agent)-' apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md   -> zero matches in the LPR section
- pnpm test:ct:desktop -- GovernancePrincipalKind (exercised by sveltekit-implementer's component test)   -> Badge renders 'Human' for omitted kind; Badge renders 'Agent'/'Human' for explicit values; Select absent in read mode, present in edit mode

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references:
  - apps/desktop/src/components/projectSettings/ForgeForm.svelte:30-36 (FORGE_OPTIONS Select + SelectItem — the select shape template)
  - apps/desktop/src/components/governance/PrincipalsList.svelte (existing principal row + edit mode — the insertion surface)
  - .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md §A.4 (kind field spec: enforcement-neutral, omitted=human, read at target ref)
notes:
  - The Badge uses neutral/secondary style (NOT success/error) because 'Agent' and 'Human' are peer descriptors — assigning different colors implies one is preferred or more trusted, which contradicts the enforcement-neutral nature of the field.
  - The non-enforcement disclosure must appear in both read (tooltip) and edit (caption) modes because an operator may only see one of the two during a session.
  - The omitted-kind=Human default must be visually explicit (a 'Human' badge, not blank) so an operator who reads a principal row without a kind field understands the current interpretation without consulting documentation.
  - The Select values must be lowercase ('agent', 'human') matching the backend TEXT storage; the labels are capitalized ('Agent', 'Human') for display.
pattern: Badge (neutral/secondary) for read display; Select + SelectItem for edit display; omitted kind = 'Human' badge; non-enforcement tooltip on badge + caption on select; no new edit mode — reuse existing PrincipalsList row edit mode
pattern_source: ForgeForm.svelte Select + SelectItem (the select shape); PrincipalsList.svelte (the insertion surface and edit mode); tech-delta §A.4 (the enforcement-neutral, omitted=human spec)
anti_pattern: green/red Badge styling (implies status); blank badge for omitted kind (hides the default-human posture); a separate inline-edit toggle for kind only (introduces a new edit mode where one already exists); describing kind as an access control; using a non-standard Badge or Select component

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agent: frontend-designer
rationale: frontend-designer owns the visual contract for the Governance UI; the Principals tab is 6a territory and this task adds a kind layer to the existing principal row. The traps are: (a) success/error Badge styling that implies enforcement, (b) blank-badge for omitted kind, and (c) omitting the non-enforcement disclosure on either the badge or the select. This contract pins all three.
coding_standards: Badge variants must use neutral/secondary styling only. Omitted kind = 'Human' badge (explicit, not blank). Non-enforcement disclosure present on both badge tooltip and select caption. Select values lowercase ('agent'/'human'); labels capitalized ('Agent'/'Human'). No new design-system tokens.

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: LPR-005 (the additive kind field on PrincipalWire and the agent-authored tag derivation it enables — this contract describes the UI surface); the Sprint 06a Principals tab design (DESIGN-MGMT-001 and the existing PrincipalsList.svelte — the surface this contract extends)
Blocks:     LPR-014 (the sveltekit UI task that IMPLEMENTS this kind display/edit contract in the Principals tab — this contract is its direct input). LPR-014 in turn depends on LPR-013 (the tauri `principal_kind` Tauri-command/SDK producer), which is NOT an implementer of this design contract — it provides the read/write binding LPR-014 consumes.
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "DESIGN-LPR-002",
  "proposed_by": "frontend-designer",
  "verification_policy": {
    "requires_tests": false,
    "requires_red_evidence": false,
    "requires_seeded_evidence": false
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN the Sprint 07 LPR section of DESIGN-ANNOTATIONS.md covers the principal kind display in the Principals tab WHEN a reviewer inspects the kind display specification THEN it specifies: an inline @gitbutler/ui Badge in neutral/secondary style with text 'Agent' or 'Human' (no success/error color); placed after the principal id in the principal row, before the role/groups summary; in edit mode, a @gitbutler/ui Select with two SelectItem values ('agent' -> 'Agent', 'human' -> 'Human') replaces or supplements the badge; select placement is beside the principal id field in the edit row",
      "verify": "design review — reviewer confirms Badge path + two variants (Agent/Human) + neutral styling + Select in edit mode + placement are all specified"
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the contract specifies the rendering for an omitted kind field WHEN a reviewer checks the omitted-kind treatment THEN it states: when the principal's committed permissions.toml entry has no kind field (or the SDK returns kind: undefined/null), the Badge renders 'Human' — the conservative default-human posture; the badge is NEVER blank; this matches the backend: absence of kind = human, per tech-delta §A.4",
      "verify": "design review — reviewer confirms omitted-kind -> 'Human' badge rule is stated; blank-badge anti-pattern is explicitly excluded"
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the contract specifies the non-enforcement disclosure WHEN a reviewer inspects the tooltip and caption specifications THEN it states: the Badge has a tooltip with text 'This label identifies the principal as an agent or human for tagging purposes. It does not change any permission grant or gate decision.'; in edit mode the Select has a caption below the control with the same disclosure; the disclosure must NOT imply the kind field grants or denies anything",
      "verify": "design review — reviewer confirms both tooltip (badge) and caption (select) carry the non-enforcement disclosure verbatim"
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the contract specifies edit mode behavior WHEN a reviewer checks how the kind select integrates with the existing principal row edit mode THEN it states: the kind Select is rendered ONLY when the principal row is in its existing edit mode (no new edit mode introduced); in read mode the Badge is shown; the implementer must NOT introduce a separate inline-edit toggle just for kind",
      "verify": "design review — reviewer confirms edit-mode integration with the existing PrincipalsList row edit mode is named; separate inline-edit anti-pattern is excluded"
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the Sprint 07 LPR section covers the kind badge WHEN a reviewer audits every visual reference THEN every visual attribute uses an existing CSS variable or defers to the component's own stylesheet — no hex literals, no new var(--kind-*)/var(--agent-*) tokens; both Badge variants use neutral/secondary styling — neither uses success, error, warning, or info styling that implies status",
      "verify": "grep -nE '#[0-9a-fA-F]{3,6}|var\\(--(kind|agent)-' apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md -> zero matches in the LPR principal kind section"
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "contract names @gitbutler/ui Badge in neutral/secondary style with 'Agent' and 'Human' variants; @gitbutler/ui Select + SelectItem with 'agent'/'human' options in edit mode; placement after principal id in the row",
      "verify": "design review of the kind display specification in DESIGN-ANNOTATIONS.md",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "contract states omitted kind renders as 'Human' badge (not blank); blank-badge anti-pattern explicitly excluded; matches backend default-human posture",
      "verify": "design review of the omitted-kind treatment in DESIGN-ANNOTATIONS.md",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "contract carries the non-enforcement tooltip text on the badge and the caption text below the select edit control",
      "verify": "design review of the tooltip and caption specifications in DESIGN-ANNOTATIONS.md",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "contract states Select renders only in the existing row edit mode; Badge renders in read mode; no separate inline-edit toggle introduced for kind alone",
      "verify": "design review of the edit-mode integration section in DESIGN-ANNOTATIONS.md",
      "maps_to_ac": "AC-4"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "zero hex color literals and zero new var(--kind-*)/var(--agent-*) tokens; neutral/secondary Badge styling confirmed for both variants",
      "verify": "grep -nE '#[0-9a-fA-F]{3,6}|var\\(--(kind|agent)-' apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md -> zero matches",
      "maps_to_ac": "AC-5"
    }
  ]
}
-->
