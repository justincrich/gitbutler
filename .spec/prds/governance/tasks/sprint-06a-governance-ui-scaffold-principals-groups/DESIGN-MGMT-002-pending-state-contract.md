# DESIGN-MGMT-002: Pending-state visual contract (○ badge, count banner, commit affordance)

## What this does

Defines a precise, prop-level visual contract for the pending-until-committed state across the governance surface: the per-row ○ pending indicator (`Badge`), the page-level count banner (`InfoMessage` warning with the commit affordance), and the state transitions from edit → pending → commit → cleared. This contract is the direct input for MGMT-UI-005 (`GovernancePendingBanner`) and the row-level pending indicator in MGMT-UI-006/007/008. **No new design-system work** — `Badge` + `InfoMessage` already exist in `packages/ui`.

## Why

Sprint 06a · PRD UC-MGMT-06 (pending-until-committed, banner, commit semantics B15) · `10-ui-infrastructure.md` Cross-cutting states + the Principals-list legend (`[●]` committed / `[○]` pending). Without a prop-level contract each implementer guesses badge/banner variants.

## How to verify

PRIMARY **AC-1** — design review: the per-row pending indicator is specified as `Badge` (`packages/ui/src/lib/components/Badge.svelte`) `style='warning' kind='soft'` for rows with staged-but-uncommitted changes; committed rows show **no** pending Badge (Badge is absent, not recolored). Full gate set in the spec below.

## Scope

- `apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md` (MODIFY — extend with the pending-state section).

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: DESIGN-MGMT-002 — Pending-state visual contract (○ badge, count banner, commit affordance)
================================================================================

TASK_TYPE:  DESIGN
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     S  (45 min)
AGENT:      designer=frontend-designer | reviewer=design-reviewer
PROPOSED-BY: frontend-designer
SPRINT:     ./SPRINT.md
PRD_REFS:   UC-MGMT-06
CAPABILITIES: (none — design-spec artifact)

RUNTIME_COMMANDS:
  review: design review of the pending-state section of DESIGN-ANNOTATIONS.md against the Cross-cutting-states wireframe
  downstream: the MGMT-UI-005 GovernancePendingBanner component test (pnpm test:ct:desktop) consumes this contract (T-MGMT-028/035)

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
A sveltekit-implementer reading this contract knows exactly which Badge props to pass for a pending row,
which InfoMessage props for the banner, how the count is rendered inside the banner, when the banner
appears/disappears, and what constitutes the commit affordance — without any design judgment at impl time.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] Specify the exact Badge props for the pending (○) and committed (●) row states.
- [MUST] Specify the exact InfoMessage props for the 'N pending governance changes' count banner incl.
  the primaryLabel + primaryAction (commit) slot.
- [MUST] Specify the cleared state: when pendingCount reaches 0 the InfoMessage banner is HIDDEN (not a 0-count banner).
- [MUST] Distinguish the per-row Badge (inline) from the page-level GovernancePendingBanner (above the tab strip).
- [MUST] Describe the transition: edit -> pending (○ appears, banner appears, count increments) -> commit
  -> committed (● restored / Badge removed, banner hidden).
- [NEVER] Introduce a new spinner/progress/toast pattern beyond InfoMessage + Badge + chipToasts.
- [NEVER] Specify optimistic enforcement — pending state is visual only; the governed path applies on commit.
- [NEVER] Introduce new CSS tokens or inline styles.
- [STRICTLY] Badge style='warning' kind='soft' for pending (○); committed rows show NO pending Badge (absent, not recolored).
- [STRICTLY] InfoMessage style='warning' outlined=true with primaryLabel='Commit changes' triggering the commit action.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: Badge warning/soft for pending rows; no Badge for committed
- [ ] AC-2: InfoMessage warning outlined primaryLabel='Commit changes' above TabList in GovernanceSettings
- [ ] AC-3: pendingCount===0 -> banner hidden; count from CLIENT-ONLY store
- [ ] AC-4: four-step state transition documented (default -> edit -> commit -> clean)
- [ ] AC-5: cross-tab pending persistence via GovernanceSettings.svelte store ownership

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (design-spec; verified by review + the downstream MGMT-UI-005 component test)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: Per-row pending indicator specified
  GIVEN: the pending-state contract exists
  WHEN:  a reviewer inspects the per-row pending indicator spec
  THEN:  it specifies Badge (packages/ui/src/lib/components/Badge.svelte) style='warning' kind='soft' size='icon'
         with a ○/'pending' label for staged-but-uncommitted rows; committed rows show NO pending Badge (absent)
  TEST_TIER: unit   UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test (pnpm test:ct:desktop)
  VERIFY: reviewer confirms Badge path + style='warning' + kind='soft' + the 'no Badge for committed rows' rule

AC-2: Page-level banner specified
  GIVEN: the contract covers the page-level banner
  WHEN:  a reviewer inspects the GovernancePendingBanner spec
  THEN:  it specifies InfoMessage (packages/ui/src/lib/components/InfoMessage.svelte) style='warning' outlined=true,
         a title rendering 'N pending governance change(s) — take effect once committed to the governance ref',
         primaryLabel='Commit changes' wired to the commit action, rendered above the Tabs TabList in GovernanceSettings
  TEST_TIER: unit   UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test
  VERIFY: reviewer confirms InfoMessage path + style='warning' + outlined=true + primaryLabel + placement (above TabList)

AC-3: pendingCount display rule
  GIVEN: the contract specifies the pendingCount display rule
  WHEN:  a reviewer checks the count rendering rule
  THEN:  the banner title interpolates pendingCount from the CLIENT-ONLY Svelte store; when pendingCount===0
         the banner is NOT rendered ({#if pendingCount > 0}); when > 0 the count shows as a numeral
  TEST_TIER: unit   UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test
  VERIFY: reviewer confirms the zero-hidden rule + store-driven count interpolation

AC-4: Full state-transition sequence
  GIVEN: the contract specifies the full state transition
  WHEN:  a reviewer reads the transition section
  THEN:  it describes (1) default — no row Badge, banner hidden; (2) after edit saved to working tree — row Badge
         warning/soft + banner with count; (3) after Commit — Badges removed, banner hidden, effective set updated;
         (4) banner hidden when .gitbutler/*.toml is clean vs HEAD regardless of UI state
  TEST_TIER: unit   UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test
  VERIFY: reviewer confirms all four transition steps + the clean-toml -> hidden-banner rule

AC-5: Cross-tab pending persistence
  GIVEN: the contract persists pending state across tab switches
  WHEN:  a reviewer checks multi-tab pending behavior
  THEN:  it states the pending Badge + banner count persist across Principals/Groups/Branch Gates/Rules tab switches
         because the Svelte pending store is owned by GovernanceSettings.svelte (parent of Tabs), not per tab content
  TEST_TIER: unit   UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test
  VERIFY: reviewer confirms the cross-tab persistence rule + store ownership at the GovernanceSettings level

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): contract names Badge style='warning' kind='soft' for pending rows; states committed rows have no Badge
- TC-2 (-> AC-2): contract names InfoMessage style='warning' outlined=true primaryLabel='Commit changes' placed above TabList in GovernanceSettings.svelte
- TC-3 (-> AC-3): contract states banner hidden when pendingCount===0 and count interpolated from the CLIENT-ONLY Svelte store
- TC-4 (-> AC-4): contract names all four transition steps (default -> edit -> commit -> clean)
- TC-5 (-> AC-5): contract states pending state persists across tab switches via GovernanceSettings.svelte store ownership

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: (none — design-spec artifact)
provides: the prop-level pending-state visual contract consumed by MGMT-UI-005 (+ row indicators in UI-006/007/008)
consumes: 10-ui-infrastructure.md Cross-cutting states + Principals-list legend; packages/ui Badge + InfoMessage; the GovernancePendingBanner net-new component

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md (MODIFY — pending-state section)
writeProhibited:
  - packages/ui/src/lib/components/** ; apps/desktop/src/components/shared/** ; any .svelte or .ts implementation file

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. .spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md — Cross-cutting states (⚠ banner wireframe) + Principals-list legend ([●] committed [○] pending)
2. .spec/prds/governance/08-uc-mgmt.md — UC-MGMT-06 (pending-until-committed, banner, commit semantics B15)
3. packages/ui/src/lib/components/InfoMessage.svelte — style MessageStyle ('warning'), outlined, primaryLabel, primaryAction, title/content Snippet
4. packages/ui/src/lib/components/Badge.svelte — style ComponentColorType ('warning'), kind ('solid'|'soft'), size ('icon'|'tag')
5. apps/desktop/src/components/governance/GovernancePendingBanner.svelte (net-new thin composition) ; GovernanceSettings.svelte (CLIENT-ONLY pending store owner)

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- Design review: contract names exact Badge + InfoMessage component paths with all required props
- Design review: the zero-count -> banner-hidden rule is stated
- Design review: the four-step state transition (default -> edit -> commit -> clean) is complete
- Downstream: the MGMT-UI-005 GovernancePendingBanner component test asserts the badge appears on edit and is absent after commit (T-MGMT-028/035)

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: GovernancePendingBanner wraps InfoMessage style='warning' outlined=true; rendered above TabList inside
  GovernanceSettings.svelte under an {#if pendingCount > 0} guard; per-row Badge warning/soft inline in the row slot.
pattern_source: apps/desktop existing optimistic-local-then-commit pending convention; packages/ui InfoMessage warning variant
anti_pattern: a spinner/progress indicator for pending; a 'committed' Badge on committed rows (absence is the committed signal);
  optimistically updating the effective-permission display before commit.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
designer: frontend-designer — maps Badge + InfoMessage props to the pending paradigm; no backend knowledge needed
reviewer: design-reviewer

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: DESIGN-MGMT-001
Blocks:     MGMT-UI-005
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "DESIGN-MGMT-002",
  "proposed_by": "frontend-designer",
  "verification_policy": { "requires_tests": false, "requires_red_evidence": false, "requires_seeded_evidence": false },
  "fixtures": {},
  "requirements": [
    { "id": "AC-1", "type": "acceptance_criterion", "primary": true, "test_tier": "unit", "unit_test_justified": "design-spec artifact, no runtime I/O — verified by design review + downstream component test (pnpm test:ct:desktop)", "description": "GIVEN the pending-state contract exists WHEN a reviewer inspects the per-row indicator THEN it specifies Badge style='warning' kind='soft' size='icon' for staged rows and NO pending Badge for committed rows (absent, not recolored)", "verify": "reviewer confirms Badge path + style='warning' kind='soft' + the 'no Badge for committed rows' rule" },
    { "id": "AC-2", "type": "acceptance_criterion", "primary": false, "test_tier": "unit", "unit_test_justified": "design-spec artifact, no runtime I/O — verified by design review + downstream component test", "description": "GIVEN the contract covers the page-level banner WHEN a reviewer inspects it THEN it specifies InfoMessage style='warning' outlined=true with an 'N pending governance change(s)' title and primaryLabel='Commit changes' rendered above the Tabs TabList in GovernanceSettings", "verify": "reviewer confirms InfoMessage path + style='warning' + outlined=true + primaryLabel + placement above TabList" },
    { "id": "AC-3", "type": "acceptance_criterion", "primary": false, "test_tier": "unit", "unit_test_justified": "design-spec artifact, no runtime I/O — verified by design review + downstream component test", "description": "GIVEN the contract specifies the pendingCount display rule WHEN a reviewer checks it THEN the banner interpolates pendingCount from the CLIENT-ONLY store, is not rendered when pendingCount===0 ({#if pendingCount > 0}), and shows the count as a numeral when > 0", "verify": "reviewer confirms the zero-hidden rule + store-driven count interpolation" },
    { "id": "AC-4", "type": "acceptance_criterion", "primary": false, "test_tier": "unit", "unit_test_justified": "design-spec artifact, no runtime I/O — verified by design review + downstream component test", "description": "GIVEN the contract specifies the full state transition WHEN a reviewer reads it THEN it describes default (no Badge, banner hidden) -> edit (row Badge warning/soft + banner with count) -> commit (Badges removed, banner hidden, effective set updated) -> clean (banner hidden when .gitbutler/*.toml is clean vs HEAD)", "verify": "reviewer confirms all four transition steps + the clean-toml -> hidden-banner rule" },
    { "id": "AC-5", "type": "acceptance_criterion", "primary": false, "test_tier": "unit", "unit_test_justified": "design-spec artifact, no runtime I/O — verified by design review + downstream component test", "description": "GIVEN the contract persists pending state across tab switches WHEN a reviewer checks it THEN it states the Badge + banner count persist across the four tabs because the pending store is owned by GovernanceSettings.svelte (parent of Tabs), not per tab content", "verify": "reviewer confirms the cross-tab persistence rule + store ownership at the GovernanceSettings level" },
    { "id": "TC-1", "type": "test_criterion", "description": "contract names Badge style='warning' kind='soft' for pending rows; committed rows have no Badge", "verify": "design review of the per-row pending section", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "contract names InfoMessage style='warning' outlined=true primaryLabel='Commit changes' above TabList in GovernanceSettings.svelte", "verify": "design review of the banner section", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "contract states banner hidden when pendingCount===0 and count interpolated from the CLIENT-ONLY Svelte store", "verify": "design review of the count display rule", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "contract names all four transition steps default -> edit -> commit -> clean", "verify": "design review of the transition section", "maps_to_ac": "AC-4" },
    { "id": "TC-5", "type": "test_criterion", "description": "contract states pending state persists across tab switches via GovernanceSettings.svelte store ownership", "verify": "design review of the cross-tab persistence rule", "maps_to_ac": "AC-5" }
  ]
}
-->
</details>
