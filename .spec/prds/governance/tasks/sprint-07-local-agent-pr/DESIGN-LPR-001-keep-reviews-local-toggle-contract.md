# DESIGN-LPR-001: `keep_reviews_local` toggle design contract — Project Settings modal

> Status: ✅ Completed (design contract)
> Commit: e09a46e36e
> Reviewer: deferred to PHASE 4.5 red-hat closeout — design contract committed prior session
> Updated: 2026-06-22T18:07:12Z

## What this does

Specify the exact toggle control for `keep_reviews_local` in the Project Settings modal, placed beside the existing forge settings (`ForgeForm.svelte`). The contract defines the on/off copy that explains "agent PRs stay local (no remote GitHub PR) vs. mirror to the forge," the default-local treatment (reflects `DefaultTrue`), the accessibility contract (label/keyboard), and the honest R21 caveat that this is an operator preference stored in the project store — not an authorization boundary.

## Why

Sprint 07 · PRD UC-LPR-03 · capabilities CAP-CONFIG-01. The `keep_reviews_local` field (LPR-006) is a per-project operator preference persisted in the project store. The UI surface that exposes it sits in the same General/Forge settings section as `forge_override` and `preferred_forge_user`, where the desktop human configures per-project artifact-routing preferences. Without this design contract, the sveltekit-implementer must invent copy, placement, and default treatment — this contract pins all three so the implementation is a transcription.

## How to verify

PRIMARY **AC-1** — `design review — reviewer confirms the DESIGN-ANNOTATIONS.md sprint-07 section carries the toggle component path, label text, description text, default-on state and its visual treatment, the forge-mirror description for the off state, and the R21 caveat phrased honestly`: Toggle component + copy contract [PRIMARY]. Full gate set in the spec below.

## Scope

- apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md (MODIFY — extend with Sprint 07 LPR section covering the `keep_reviews_local` toggle)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: DESIGN-LPR-001 — keep_reviews_local toggle design contract
================================================================================

TASK_TYPE:   DESIGN
STATUS:      Backlog
PRIORITY:    P1
EFFORT:      S  (30 min)
AGENT:       frontend-designer
PROPOSED-BY: frontend-designer
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-LPR-03
CAPABILITIES:CAP-CONFIG-01

RUNTIME_COMMANDS:
  test:  pnpm test:ct:desktop -- KeepReviewsLocalToggle (exercised by the sveltekit-implementer's component test for ForgeForm)

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
A sveltekit-implementer reading this contract knows: (a) which component to use for the toggle, (b) the exact label and description copy for both on and off states, (c) that the toggle defaults on (reflects `DefaultTrue` — existing projects without the field in their project JSON are treated as on), (d) the a11y label/keyboard contract, and (e) that the R21 caveat must be surfaced honestly in the description copy — without making any of these design decisions independently.

--------------------------------------------------------------------------------
CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST specify the Toggle component from `@gitbutler/ui` (the same `Toggle` used by `PreferencesForm.svelte` for `omit_certificate_check`) for the `keep_reviews_local` control.
- [MUST] MUST place the toggle inside a `CardGroup.Item` (the same container `ForgeForm.svelte` + `PreferencesForm.svelte` use for per-project settings) immediately below the existing forge account configuration (below ForgeForm, in the General settings section or a new "Review" subsection — see PLACEMENT note below).
- [MUST] MUST specify label text and description text verbatim for the ON state: label = 'Keep agent reviews local', description = 'Agent-authored PRs stay on the local review layer — no GitHub PR is opened. Change this only if you want agent reviews mirrored to your forge. (This is a local project preference, not a security boundary — the project store is not independently verified.)'
- [MUST] MUST specify label text and description text verbatim for the OFF state: the label is unchanged ('Keep agent reviews local'); the toggle's checked=false state communicates the flip. The description becomes: 'Agent-authored PRs will be mirrored to your forge when approved. Internal principal identifiers may be disclosed to the forge API; ensure all principals have forge accounts before enabling. (See: local project preference, not a security boundary.)'
- [MUST] MUST specify that the toggle renders in the checked=true state by default (reflecting `DefaultTrue` — an older project file without `keep_reviews_local` in the JSON is treated as on). The implementer MUST NOT default to unchecked.
- [MUST] MUST specify the a11y contract: the Toggle `id` is `'keepReviewsLocal'`; the `CardGroup.Item` `labelFor` prop binds to that `id` so clicking the label text activates the toggle; keyboard: Space/Enter on the focused toggle flips it.
- [MUST] MUST surface the R21 caveat in the description copy: the preference is stored in the project store (not ref-pinned committed config, not `administration:write`-gated) and must NOT be described as a security control or authorization boundary. The caveat is: "(This is a local project preference, not a security boundary — the project store is not independently verified.)"
- [MUST] MUST confirm no new design-system token is introduced — use only existing `@gitbutler/ui` CSS variables.
- [NEVER] NEVER describe `keep_reviews_local` as a security control, access gate, or authorization boundary in the copy. It is an operator preference about where review artifacts are routed — naming it a security control misrepresents a trusted-desktop preference (R21).
- [NEVER] NEVER place the toggle inside the Governance tab (GovernanceSettings.svelte). `keep_reviews_local` is a project-store preference (the forge class), not a governed ref-pinned config mutation. It belongs in the General/Forge project settings section, beside `forge_override`/`preferred_forge_user`.
- [NEVER] NEVER invent a new toggle or switch component — use `@gitbutler/ui` `Toggle`.
- [NEVER] NEVER introduce a hex color literal or a new CSS variable in the annotation.
- [STRICTLY] STRICTLY model the `CardGroup.Item` + `Toggle` pattern on `apps/desktop/src/components/projectSettings/PreferencesForm.svelte` (the `omit_certificate_check` toggle) — same container, same Toggle import path, same labelFor/id binding.
- [STRICTLY] STRICTLY cite `apps/desktop/src/components/projectSettings/ForgeForm.svelte` as the placement neighbor — `keep_reviews_local` is the same class of preference as `forge_override`/`preferred_forge_user` and sits immediately after ForgeForm in the General settings view.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [x] AC-1 [PRIMARY]: Toggle component + copy contract
- [x] AC-2: Default-on treatment and DefaultTrue reflection
- [x] AC-3: Accessibility label/keyboard contract
- [x] AC-4: R21 caveat present in the description copy
- [x] AC-5: No new design-system tokens
- [x] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: Toggle component + copy contract
  GIVEN: the Sprint 07 LPR section of DESIGN-ANNOTATIONS.md covers the keep_reviews_local toggle
  WHEN:  a reviewer inspects the toggle specification
  THEN:  it specifies: `@gitbutler/ui` `Toggle` component inside a `CardGroup.Item standalone` with `labelFor='keepReviewsLocal'`; label text = 'Keep agent reviews local'; on-state description = 'Agent-authored PRs stay on the local review layer — no GitHub PR is opened. Change this only if you want agent reviews mirrored to your forge. (This is a local project preference, not a security boundary — the project store is not independently verified.)'; off-state note = 'Agent-authored PRs will be mirrored to your forge when approved. Internal principal identifiers may be disclosed to the forge API; ensure all principals have forge accounts before enabling. (See: local project preference, not a security boundary.)'; placement = immediately following ForgeForm in the General project settings section
  TEST_TIER: component   VERIFICATION_SERVICE: apps/desktop CT harness (pnpm test:ct:desktop) — component test asserts the toggle renders checked=true by default and the label text matches
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test
  VERIFY: design review — reviewer confirms Toggle path + label text + on-state description + off-state note + placement are all present and verbatim

AC-2: Default-on treatment and DefaultTrue reflection
  GIVEN: the contract specifies the toggle's default rendering
  WHEN:  a reviewer inspects the default-state specification
  THEN:  it states: the toggle renders checked=true by default because `Project.keep_reviews_local` uses `DefaultTrue` (older project files without the field deserialize to on); the implementer must bind the Toggle `checked` prop to `project.keep_reviews_local` (which is `true` when the field is absent); the toggle MUST NOT default to unchecked; a project file with no `keep_reviews_local` key renders the toggle on
  TEST_TIER: component   VERIFICATION_SERVICE: apps/desktop CT harness — component test asserts default checked=true when project has no keep_reviews_local field
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review
  VERIFY: design review — reviewer confirms default-on and DefaultTrue reflection are named; checked=false anti-pattern is explicitly excluded

AC-3: Accessibility label/keyboard contract
  GIVEN: the contract specifies the a11y requirements for the toggle
  WHEN:  a reviewer reads the a11y specification
  THEN:  it states: Toggle `id='keepReviewsLocal'`; `CardGroup.Item` `labelFor='keepReviewsLocal'` so clicking the label text activates the toggle; keyboard: Space or Enter on the focused toggle flips the value; the toggle is reachable via Tab in the normal document order; no focus suppression with outline:none without a replacement
  TEST_TIER: component   VERIFICATION_SERVICE: apps/desktop CT harness — component test asserts labelFor binding and keyboard Space activation
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review
  VERIFY: design review — reviewer confirms id + labelFor binding + Space/Enter keyboard model are named

AC-4: R21 caveat present in the description copy
  GIVEN: the contract specifies the description copy
  WHEN:  a reviewer audits the caveat language in the description text
  THEN:  the on-state description includes the exact parenthetical: "(This is a local project preference, not a security boundary — the project store is not independently verified.)"; the off-state note includes: "(See: local project preference, not a security boundary.)"; no part of the copy describes keep_reviews_local as an authorization control or access gate; the caveat honestly reflects R21 (an untrusted project-store write can flip the flag)
  TEST_TIER: component   VERIFICATION_SERVICE: design review
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + grep audit
  VERIFY: design review — reviewer confirms both R21 parentheticals are present verbatim and no security-control language appears

AC-5: No new design-system tokens
  GIVEN: the Sprint 07 LPR section of DESIGN-ANNOTATIONS.md
  WHEN:  a reviewer audits every visual reference in the section
  THEN:  every visual attribute uses an existing CSS variable or defers to the component's own stylesheet — no hex literals, no new var(--lpr-*)/var(--review-*) tokens
  TEST_TIER: component   VERIFICATION_SERVICE: grep audit on the annotation file
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by static grep
  VERIFY: grep -nE '#[0-9a-fA-F]{3,6}|var\(--(lpr|review)-' apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md -> zero matches in the LPR section

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): contract names Toggle path (`@gitbutler/ui` Toggle), CardGroup.Item standalone container, labelFor='keepReviewsLocal', label text = 'Keep agent reviews local', on-state description, off-state note, placement after ForgeForm
    VERIFY: design review of the toggle specification in DESIGN-ANNOTATIONS.md
- TC-2 (-> AC-2): contract states default checked=true reflecting DefaultTrue; names the anti-pattern (checked=false default) as explicitly excluded; project file without the field renders the toggle on
    VERIFY: design review of the default-state specification in DESIGN-ANNOTATIONS.md
- TC-3 (-> AC-3): contract states id='keepReviewsLocal', labelFor binding, Space/Enter keyboard activation, Tab reachability, no outline suppression
    VERIFY: design review of the a11y specification in DESIGN-ANNOTATIONS.md
- TC-4 (-> AC-4): contract includes both R21 parentheticals verbatim; no security-control language in the copy
    VERIFY: design review + grep for 'security control' / 'authorization' in the toggle copy — zero matches
- TC-5 (-> AC-5): zero hex color literals and zero new var(--lpr-*)/var(--review-*) tokens in the Sprint 07 LPR section
    VERIFY: grep -nE '#[0-9a-fA-F]{3,6}|var\(--(lpr|review)-' apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md -> zero matches

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md (MODIFY — extend with Sprint 07 LPR section)
writeProhibited:
  - apps/desktop/src/components/projectSettings/ForgeForm.svelte — read only for placement reference; do not modify
  - apps/desktop/src/components/projectSettings/PreferencesForm.svelte — read only for pattern reference; do not modify
  - packages/ui/src/lib/components/** — no design-system changes
  - any .svelte or .ts implementation file — design spec artifact only

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. apps/desktop/src/components/projectSettings/PreferencesForm.svelte [1-39] — [PRIMARY PATTERN] `CardGroup.Item standalone labelFor="omitCertificateCheck"` + `Toggle id="omitCertificateCheck"` — mirror this exact container + component pattern for the keep_reviews_local toggle
2. apps/desktop/src/components/projectSettings/ForgeForm.svelte [1-90] — the placement neighbor: `keep_reviews_local` is the same class as `forge_override`/`preferred_forge_user`; the toggle sits immediately after this form in the General settings view
3. apps/desktop/src/components/views/ProjectSettingsModalContent.svelte [59-70] — the General/Forge settings composition: GeneralSettings → ForgeForm → the new keep_reviews_local toggle location in the same section
4. .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md §C — the project-setting spec: DefaultTrue, project-store class, R21 (untrusted project-store write flips it), not administration:write-gated, not ref-pinned committed config, R12 trusted-desktop model
5. .spec/prds/governance/tasks/sprint-07-local-agent-pr/LPR-006-keep-reviews-local-setting.md — the backend task this contract serves: the DefaultTrue field, the forge_override placement, the R12/R21 residual

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- design review of apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md Sprint 07 LPR section   -> Toggle path + label + on-state description (with R21 parenthetical) + off-state note (with R21 parenthetical) + placement + default-on + a11y id/labelFor/keyboard — all present
- grep -nE '#[0-9a-fA-F]{3,6}|var\(--(lpr|review)-' apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md   -> zero matches in the LPR section
- pnpm test:ct:desktop -- KeepReviewsLocalToggle (exercised by the sveltekit-implementer's component test)   -> toggle renders checked=true by default; label text matches; labelFor binding activates the toggle

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references:
  - apps/desktop/src/components/projectSettings/PreferencesForm.svelte — CardGroup.Item standalone + Toggle (the omit_certificate_check pattern, the EXACT model)
  - apps/desktop/src/components/projectSettings/ForgeForm.svelte — placement neighbor (same settings class: per-project forge/artifact-routing preferences)
  - .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md §C — DefaultTrue, R12 trusted-desktop, R21 residual (not an authorization boundary)
notes:
  - PLACEMENT: the toggle lands in the General project settings section, immediately following `ForgeForm` — NOT in the Governance tab. `keep_reviews_local` is the artifact-routing class (`forge_override`/`preferred_forge_user`), not a governed ref-pinned config mutation.
  - DEFAULT-ON: `DefaultTrue` means an older project JSON without `keep_reviews_local` deserializes to `true`. The Toggle `checked` prop MUST reflect `project.keep_reviews_local` directly (no nullish coalescing to `false`).
  - R21 HONESTY: the copy must not describe this toggle as a security control. The parenthetical "(This is a local project preference, not a security boundary — the project store is not independently verified.)" is the minimum honest disclosure.
  - OFF-STATE DISCLOSURE: the off-state note must mention that internal principal identifiers may be disclosed to the forge API when mirroring is enabled (tech-delta §D / §G R21 sub-point F-005), so the operator understands the consequence before flipping.
pattern: CardGroup.Item standalone + labelFor binding + Toggle (checked = project.keep_reviews_local, default true) + on/off copy that names the local/forge routing and the R21 caveat — placed after ForgeForm in the General settings section
pattern_source: apps/desktop/src/components/projectSettings/PreferencesForm.svelte (the omit_certificate_check pattern); .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md §C (the field spec + R12/R21 residual)
anti_pattern: placing the toggle in the Governance tab (it is a project-store preference, not governed ref-pinned config); defaulting to unchecked (misrepresents DefaultTrue); describing it as an authorization boundary or security control (R21 — it is not); using a Switch/Checkbox other than the @gitbutler/ui Toggle; introducing hex color literals

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agent: frontend-designer
rationale: frontend-designer owns the copy contract and placement annotation for project-settings controls; no Rust/Tauri knowledge required. The traps are: (a) placing the toggle in the Governance tab instead of General settings, (b) defaulting to unchecked (misrepresenting DefaultTrue), and (c) omitting the R21 caveat or using security-control language. This contract pins all three so the sveltekit-implementer has no design decisions to make.
coding_standards: Copy must include the R21 parentheticals verbatim. Pattern source is PreferencesForm.svelte (CardGroup.Item + Toggle). Placement is beside ForgeForm — the same forge/artifact-routing class. No new design-system tokens.

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: LPR-006 (the keep_reviews_local DefaultTrue field — this contract describes the UI surface for it); DESIGN-MGMT-003 (read-only isReadOnly contract — if the toggle needs a disabled state when isReadOnly, reference that contract)
Blocks:     sveltekit-implementer adding the keep_reviews_local toggle to GeneralSettings/ForgeForm section (this contract is the direct input for all copy, placement, and default-state decisions)
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "DESIGN-LPR-001",
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
      "description": "GIVEN the Sprint 07 LPR section of DESIGN-ANNOTATIONS.md covers the keep_reviews_local toggle WHEN a reviewer inspects the toggle specification THEN it specifies: @gitbutler/ui Toggle inside a CardGroup.Item standalone with labelFor='keepReviewsLocal'; label text = 'Keep agent reviews local'; on-state description (with R21 parenthetical); off-state note (with principal-disclosure and R21 parenthetical); placement immediately following ForgeForm in the General project settings section",
      "verify": "design review — reviewer confirms Toggle path + label text + on-state description + off-state note + placement are all present and verbatim"
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the contract specifies the toggle's default rendering WHEN a reviewer inspects the default-state specification THEN it states: the toggle renders checked=true by default because Project.keep_reviews_local uses DefaultTrue (older project files without the field deserialize to on); the implementer must bind the Toggle checked prop to project.keep_reviews_local; the toggle MUST NOT default to unchecked",
      "verify": "design review — reviewer confirms default-on and DefaultTrue reflection are named; checked=false anti-pattern is explicitly excluded"
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the contract specifies the a11y requirements WHEN a reviewer reads the a11y specification THEN it states: Toggle id='keepReviewsLocal'; CardGroup.Item labelFor='keepReviewsLocal' so clicking the label text activates the toggle; keyboard: Space or Enter on the focused toggle flips the value; the toggle is reachable via Tab; no focus suppression without a replacement",
      "verify": "design review — reviewer confirms id + labelFor binding + Space/Enter keyboard model are named"
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the contract specifies the description copy WHEN a reviewer audits the caveat language THEN the on-state description includes the exact parenthetical: '(This is a local project preference, not a security boundary — the project store is not independently verified.)'; the off-state note includes: '(See: local project preference, not a security boundary.)'; no part of the copy describes keep_reviews_local as an authorization control or access gate",
      "verify": "design review — reviewer confirms both R21 parentheticals are present verbatim and no security-control language appears"
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the Sprint 07 LPR section of DESIGN-ANNOTATIONS.md WHEN a reviewer audits every visual reference in the section THEN every visual attribute uses an existing CSS variable or defers to the component's own stylesheet — no hex literals, no new var(--lpr-*)/var(--review-*) tokens",
      "verify": "grep -nE '#[0-9a-fA-F]{3,6}|var\\(--(lpr|review)-' apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md -> zero matches in the LPR section"
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "contract names Toggle path, CardGroup.Item standalone container, labelFor='keepReviewsLocal', label text = 'Keep agent reviews local', on-state description, off-state note, placement after ForgeForm",
      "verify": "design review of the toggle specification in DESIGN-ANNOTATIONS.md",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "contract states default checked=true reflecting DefaultTrue; checked=false anti-pattern explicitly excluded; project file without the field renders the toggle on",
      "verify": "design review of the default-state specification in DESIGN-ANNOTATIONS.md",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "contract states id='keepReviewsLocal', labelFor binding, Space/Enter keyboard activation, Tab reachability, no outline suppression",
      "verify": "design review of the a11y specification in DESIGN-ANNOTATIONS.md",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "contract includes both R21 parentheticals verbatim; no security-control language in the copy",
      "verify": "design review + grep for 'security control' / 'authorization' in the toggle copy section — zero matches",
      "maps_to_ac": "AC-4"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "zero hex color literals and zero new var(--lpr-*)/var(--review-*) tokens in the Sprint 07 LPR section",
      "verify": "grep -nE '#[0-9a-fA-F]{3,6}|var\\(--(lpr|review)-' apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md -> zero matches",
      "maps_to_ac": "AC-5"
    }
  ]
}
-->
