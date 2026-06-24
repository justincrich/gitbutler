# DESIGN-LPR-003: Local-Review view IA + state contract (read-only)

> Status: ✅ Completed (design contract)
> Commit: e09a46e36e
> Reviewer: deferred to PHASE 4.5 red-hat closeout — design contract committed prior session
> Updated: 2026-06-22T18:07:12Z

## What this does

Specify the information-architecture and state contract for the Local-Review view: a READ-ONLY view that renders a local PR object — reviewer assignments (plus assignment state), comment threads (file/line scoped or PR-level, with resolved/unresolved status), the derived lifecycle (`Draft` / `AwaitingReview` / `ChangesRequested` / `Approved` / `Mergeable` — derived at query time from commits + verdict-at-head + open assignments per LPR-005), and the `agent-authored` badge when the opener's declared `kind = "agent"`. The view is one component with four lifecycle states (not four separate views), fully read-only (no mutate controls — the CLI/agent drives writes), and reuses existing forge-review UI surfaces and patterns where possible.

## Why

Sprint 07 · PRD UC-LPR-01, UC-LPR-04, UC-LPR-05 · capability CAP-AUTHZ-01. The `review_status` backend read (LPR-005) yields a `DerivedPr` object an orchestrator and a human both need to inspect. Without this design contract, the sveltekit-implementer must invent the IA, the four lifecycle states, the thread rendering, and the agent badge placement independently. This contract makes implementation a transcription.

## How to verify

PRIMARY **AC-1** — `design review — reviewer confirms the DESIGN-ANNOTATIONS.md Sprint 07 LPR section carries: the single-view / four-state model, the five data sections (lifecycle status, agent badge, reviewer assignments, comment threads, derived PR metadata), the read-only constraint (no mutate controls), and the empty-state specs for loading, no-review, and zero-items per section`: Local-Review view IA + four lifecycle states [PRIMARY]. Full gate set in the spec below.

## Scope

- apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md (MODIFY — extend with Sprint 07 LPR section covering the Local-Review view IA + state contract)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: DESIGN-LPR-003 — Local-Review view IA + state contract (read-only)
================================================================================

TASK_TYPE:   DESIGN
STATUS:      Backlog
PRIORITY:    P1
EFFORT:      M  (50 min)
AGENT:       frontend-designer
PROPOSED-BY: frontend-designer
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-LPR-01, UC-LPR-04, UC-LPR-05
CAPABILITIES:CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  pnpm test:ct:desktop -- LocalReviewView (exercised by the sveltekit-implementer's component test for the local review view)

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
A sveltekit-implementer reading this contract knows: (a) the single view that covers all four lifecycle states (not four separate views), (b) the exact five data sections and their visual order, (c) which existing forge-review UI patterns to reuse vs. which to avoid (read-only, no mutate controls), (d) empty/loading/error states per section, and (e) the read-only contract and why: the CLI/agent drives all writes; the UI is an observer.

--------------------------------------------------------------------------------
CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST specify a SINGLE view component (not four separate views, not a tab per lifecycle state) that renders one local PR object and reflects the current lifecycle state via a status indicator — one view, four possible states of the status indicator.
- [MUST] MUST specify the following five data sections in this top-to-bottom order:
    1. PR header: derived lifecycle status badge + branch name (source → target) + `agent-authored` tag (if agent_authored==true) + derived PR metadata (sha, author, created_at, title)
    2. Reviewer assignments: list of assigned reviewers with their assignment state (Pending / Approved / ChangesRequested), rendered as a row per reviewer
    3. Comment threads: list of threads grouped by file+line (code comments) and a PR-level group (file==null); each thread shows its comments in order with resolved/unresolved status; resolved threads are visually distinguished (muted/collapsed) but still visible
    4. Derived lifecycle explanation: a plain-text caption explaining what the current status means and what action (if any) drives it to the next state — e.g. "Awaiting review from 2 reviewers. Approve via `but review approve`."
    5. Merge gate note: a static informational note: 'Merge decisions are made by the merge gate, not this view. A status of Approved or Mergeable here reflects the derived state — the gate re-derives verdict at merge time.'
- [MUST] MUST specify four lifecycle states of the status badge (matching `DerivedPrStatus`): Draft, AwaitingReview, ChangesRequested, Approved/Mergeable — using a Badge component from `@gitbutler/ui`. Styling guidance: Draft = neutral, AwaitingReview = info/blue, ChangesRequested = warning/yellow, Approved = success/green, Mergeable = success/green (same as Approved — "ready to merge"). No new color tokens.
- [MUST] MUST specify the `agent-authored` badge: displayed ONLY when `agent_authored == true`; uses neutral/secondary Badge styling (not success/error — it is a descriptor, not a status); tooltip text: 'This PR was opened by a principal declared as an agent in .gitbutler/permissions.toml. This is a metadata tag — it does not affect merge decisions.'
- [MUST] MUST specify the empty states:
    (a) Loading: a skeleton/placeholder for the full view while `review_status` is in flight
    (b) No review open: `EmptyStatePlaceholder` (the same component used across all Governance tabs) with title='No local review open for this branch.' and a caption: 'Open one with `but review request <branch>` to start the review loop.'
    (c) No reviewers assigned: within the Reviewer Assignments section, inline text 'No reviewers assigned yet.'
    (d) No comment threads: within the Comment Threads section, inline text 'No comment threads yet.'
- [MUST] MUST specify the READ-ONLY constraint explicitly: this view renders NO mutate controls (no Approve button, no Request Changes button, no Post Comment form, no Assign button). All writes are driven by the CLI (`but review approve`, `but review comment`, `but review assign`, `but review resolve`). The design contract must name this as the intentional UI posture: "the Local-Review view is an observer — reads and renders, never writes."
- [MUST] MUST confirm that existing forge-review patterns are reused where possible: the comment thread rendering reuses the same visual idioms as the existing forge PR review comment display (if one exists in the desktop app), adapted for local data; the reviewer row reuses the same chip/row shape as reviewer assignments in the forge review surface.
- [MUST] MUST specify the Merge gate note (section 5) as a static informational component — an `InfoMessage` (from `@gitbutler/ui`) with kind='info', not kind='warning' or kind='error'. The note is not alarming; it is clarifying.
- [MUST] MUST confirm no new design-system token is introduced — use only existing `@gitbutler/ui` CSS variables.
- [NEVER] NEVER specify mutate controls (Approve, Request Changes, Post Comment form, Assign Reviewer button, Resolve Thread button) in this view. The UI is read-only. Including mutate controls would misrepresent the CLI-driven write discipline and create a confusing half-baked UX.
- [NEVER] NEVER specify separate views or tabs for the four lifecycle states — it is one view with a state indicator.
- [NEVER] NEVER describe the Approved/Mergeable status as authorizing a merge in the UI copy. The merge gate note (section 5) must make clear that the gate re-derives at merge time.
- [NEVER] NEVER use a hex color literal or a new CSS variable.
- [STRICTLY] STRICTLY the agent-authored badge is neutral/secondary style, not success (green). It is a descriptor — an agent-authored PR is not better or worse than a human-authored one.
- [STRICTLY] STRICTLY resolved comment threads are visible but visually muted/collapsed (not hidden). Hiding resolved threads removes context the operator or orchestrator may need to understand the review history.
- [STRICTLY] STRICTLY model the empty state for "No local review open" on `EmptyStatePlaceholder` (the same component used in the four Governance tab empty states, per DESIGN-MGMT-006) — same component, no new component.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [x] AC-1 [PRIMARY]: Local-Review view IA + four lifecycle states
- [x] AC-2: Five data sections in order with visual specs
- [x] AC-3: Agent-authored badge contract
- [x] AC-4: Empty / loading / no-items states
- [x] AC-5: Read-only constraint + merge gate note
- [x] AC-6: No new design-system tokens; lifecycle badge colors
- [x] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: Local-Review view IA + four lifecycle states
  GIVEN: the Sprint 07 LPR section of DESIGN-ANNOTATIONS.md covers the Local-Review view
  WHEN:  a reviewer inspects the view IA specification
  THEN:  it specifies: one view component (not four separate views); a status Badge with four states (Draft/AwaitingReview/ChangesRequested/Approved-or-Mergeable); the five data sections in top-to-bottom order (PR header, Reviewer Assignments, Comment Threads, Lifecycle explanation, Merge gate note); and the read-only posture named explicitly ("the Local-Review view is an observer — reads and renders, never writes")
  TEST_TIER: component   VERIFICATION_SERVICE: apps/desktop CT harness (pnpm test:ct:desktop) — component test asserts all four lifecycle state badges render and no mutate controls are present
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + downstream component test
  VERIFY: design review — reviewer confirms single-view / four-state model, five data sections in order, read-only posture all present

AC-2: Five data sections in order with visual specs
  GIVEN: the contract specifies the five data sections
  WHEN:  a reviewer reads each section specification
  THEN:  it specifies:
    Section 1 (PR header): status Badge (four states, see AC-6 for styling) + source-branch→target-branch display + agent-authored Badge (if agent_authored) + sha, author, created_at, title fields; all read-only text
    Section 2 (Reviewer Assignments): one row per assignment with reviewer principal id + assignment state chip ('Pending'/'Approved'/'Changes Requested') — chip is neutral for Pending, success for Approved, warning for Changes Requested; no assign/remove controls
    Section 3 (Comment Threads): threads grouped by file+line (code-scoped) then by PR-level (file=null); each thread shows comments in time order; resolved threads rendered with muted/collapsed visual treatment (not hidden); thread resolution state shown via a resolved icon; no Post Comment form, no Resolve button
    Section 4 (Lifecycle explanation): one-sentence plain-text caption per lifecycle state explaining what it means and naming the CLI verb that advances it; rendered below the threads section
    Section 5 (Merge gate note): `InfoMessage` kind='info' with text 'Merge decisions are made by the merge gate, not this view. A status of Approved or Mergeable here reflects the derived state — the gate re-derives verdict at merge time.'
  TEST_TIER: component   VERIFICATION_SERVICE: apps/desktop CT harness — component test asserts all five sections are rendered in the correct order
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review
  VERIFY: design review — reviewer confirms all five section specifications with their visual treatments are present in the annotation

AC-3: Agent-authored badge contract
  GIVEN: the contract specifies the agent-authored badge
  WHEN:  a reviewer inspects the agent-authored badge specification
  THEN:  it states: the agent-authored Badge renders ONLY when agent_authored==true; it uses neutral/secondary Badge styling (not success/green — it is a descriptor, not a status); tooltip text = 'This PR was opened by a principal declared as an agent in .gitbutler/permissions.toml. This is a metadata tag — it does not affect merge decisions.'; when agent_authored==false the badge is absent (not rendered as 'Human'); the badge placement is in the PR header (section 1), after the lifecycle status Badge, before the branch name display
  TEST_TIER: component   VERIFICATION_SERVICE: apps/desktop CT harness — component test asserts badge present when agent_authored==true; absent when false
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review
  VERIFY: design review — reviewer confirms badge present/absent logic, neutral styling, tooltip text, and placement are all specified

AC-4: Empty / loading / no-items states
  GIVEN: the contract specifies empty and loading states
  WHEN:  a reviewer checks the empty/loading specifications
  THEN:  it specifies: (a) loading = a skeleton/placeholder spanning the full view height while review_status is in flight; (b) no review open = EmptyStatePlaceholder (packages/ui/src/lib/components/EmptyStatePlaceholder.svelte) with title='No local review open for this branch.' + caption 'Open one with `but review request <branch>` to start the review loop.' — no primary action button (the action is a CLI command, not a UI button); (c) no reviewers assigned = inline text 'No reviewers assigned yet.' within the assignments section; (d) no comment threads = inline text 'No comment threads yet.' within the threads section; each empty sub-state uses existing component patterns, no new components
  TEST_TIER: component   VERIFICATION_SERVICE: apps/desktop CT harness — component test asserts each empty state renders the correct content
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review
  VERIFY: design review — reviewer confirms all four empty/loading specifications are present: loading skeleton, no-review EmptyStatePlaceholder, no-reviewers inline text, no-threads inline text

AC-5: Read-only constraint + merge gate note
  GIVEN: the contract specifies the read-only constraint
  WHEN:  a reviewer checks the mutate-control inventory
  THEN:  it explicitly lists all mutate controls that MUST NOT appear in this view (Approve button, Request Changes button, Post Comment form, Assign Reviewer button, Resolve Thread button) and names the reason: "the Local-Review view is an observer — reads and renders, never writes; all writes are driven by the CLI (`but review approve`, `but review comment`, `but review assign`, `but review resolve`)"; AND the Merge gate note (section 5) is specified as an InfoMessage kind='info' with the verbatim text from AC-2 section 5 specification
  TEST_TIER: component   VERIFICATION_SERVICE: apps/desktop CT harness — component test asserts zero buttons/forms with write actions are rendered in the view
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by design review + grep for mutate controls in the component
  VERIFY: design review — reviewer confirms the mutate-control exclusion list is present with reasons, and the InfoMessage merge gate note is specified

AC-6: No new design-system tokens; lifecycle badge colors
  GIVEN: the contract specifies the lifecycle status Badge styling
  WHEN:  a reviewer audits all visual references in the Local-Review view section
  THEN:  every visual attribute uses an existing CSS variable or defers to the component's own stylesheet — no hex literals, no new var(--review-*)/var(--lpr-*) tokens; lifecycle badge styling uses only existing @gitbutler/ui Badge variants: Draft=neutral, AwaitingReview=info/blue, ChangesRequested=warning/yellow, Approved=success/green, Mergeable=success/green; agent-authored badge=neutral/secondary; assignment state chips: Pending=neutral, Approved=success/green, ChangesRequested=warning/yellow
  TEST_TIER: component   VERIFICATION_SERVICE: grep audit on the annotation file
  UNIT_TEST_JUSTIFIED: design-spec artifact, no runtime I/O — verified by static grep
  VERIFY: grep -nE '#[0-9a-fA-F]{3,6}|var\(--(review|lpr)-' apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md -> zero matches in the LPR local-review section

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): contract names single-view / four-state model; five data sections in top-to-bottom order; read-only posture named explicitly
    VERIFY: design review of the view IA specification in DESIGN-ANNOTATIONS.md
- TC-2 (-> AC-2): contract specifies all five section visual treatments (header fields, assignment chip states, thread grouping, collapsed-not-hidden resolved, lifecycle caption, InfoMessage merge gate note)
    VERIFY: design review of the five-section specification in DESIGN-ANNOTATIONS.md
- TC-3 (-> AC-3): contract names agent-authored Badge: neutral/secondary style, present only when agent_authored==true, absent when false, tooltip text present, placement in PR header after lifecycle Badge
    VERIFY: design review of the agent-authored badge specification in DESIGN-ANNOTATIONS.md
- TC-4 (-> AC-4): contract specifies all four empty/loading states: loading skeleton, no-review EmptyStatePlaceholder (with CLI caption, no action button), no-reviewers inline text, no-threads inline text
    VERIFY: design review of the empty/loading state specifications in DESIGN-ANNOTATIONS.md
- TC-5 (-> AC-5): contract lists all excluded mutate controls with reasons; InfoMessage merge gate note specified with verbatim text
    VERIFY: design review of the read-only constraint + merge gate note section in DESIGN-ANNOTATIONS.md
- TC-6 (-> AC-6): zero hex color literals and zero new var(--review-*)/var(--lpr-*) tokens; lifecycle badge colors match existing Badge variants only
    VERIFY: grep -nE '#[0-9a-fA-F]{3,6}|var\\(--(review|lpr)-' apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md -> zero matches

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md (MODIFY — extend with Sprint 07 LPR section for the Local-Review view IA + state contract)
writeProhibited:
  - apps/desktop/src/components/governance/*.svelte — read only for pattern reference; do not modify
  - packages/ui/src/lib/components/** — no design-system changes
  - any .svelte or .ts implementation file — design spec artifact only

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. apps/desktop/src/components/governance/GovernanceSettings.svelte [1-60] — [PRIMARY CONTEXT] the Governance tab composition: four-tab layout, isReadOnly prop, EmptyStatePlaceholder pattern, InfoMessage usage — the Local-Review view slots into this surface as an additional view in the Governance section
2. packages/ui/src/lib/components/EmptyStatePlaceholder.svelte [1-50] — the empty state component used for "No local review open" — confirm the title/caption/action-slot API before specifying slot content (no action button for this empty state — the action is a CLI command)
3. .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md §A.1/A.2/A.3/A.4 — the data schema: local_review_assignments (state field), local_review_comments (file/line/thread_id/resolved), local_review_meta (opener_principal), derived PR lifecycle (DerivedPrStatus enum), agent-authored tag (from declared kind)
4. .spec/prds/governance/tasks/sprint-07-local-agent-pr/LPR-005-derived-pr-lifecycle-agent-tag.md — the backend read API: DerivedPr struct (target_branch, source_branch, sha, author, title, draft, created_at, updated_at, status, agent_authored, labels, open_assignments, unresolved_thread_count), DerivedPrStatus enum, agent-authored derivation
5. .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/DESIGN-MGMT-006-empty-states.md — the EmptyStatePlaceholder pattern established for all four Governance tabs — reuse for the no-review-open empty state
6. .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md §B — the API surface (review_status is a branch-scoped read; list_comments returns all threads for the branch; no write verbs in the view)

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- design review of apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md Sprint 07 LPR local-review section   -> single-view/four-state model, five sections in order, agent-authored badge contract, four empty/loading states, read-only posture + mutate exclusion list, InfoMessage merge gate note — all present
- grep -nE '#[0-9a-fA-F]{3,6}|var\(--(review|lpr)-' apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md   -> zero matches in the LPR local-review section
- pnpm test:ct:desktop -- LocalReviewView (exercised by sveltekit-implementer's component test)   -> four lifecycle state badges render; agent-authored badge present/absent based on agent_authored; no mutate buttons rendered; EmptyStatePlaceholder renders for no-review state

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references:
  - apps/desktop/src/components/governance/GovernanceSettings.svelte — the four-tab governance surface into which the Local-Review view integrates
  - packages/ui/src/lib/components/EmptyStatePlaceholder.svelte — the empty state for no-review-open (title + CLI caption; no action button because the action is CLI-driven)
  - packages/ui/src/lib/components/InfoMessage.svelte (from @gitbutler/ui) — the merge gate note (kind='info')
  - .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md §A.3 — the DerivedPrStatus derivation rules: commits + verdict-at-head + pending assignments + unresolved thread count
  - .spec/prds/governance/tasks/sprint-07-local-agent-pr/LPR-005-derived-pr-lifecycle-agent-tag.md — the DerivedPr struct (the data the view renders)
notes:
  - ONE VIEW, FOUR STATES: the lifecycle status Badge changes; the five data sections remain; no routing or tab switching per state.
  - READ-ONLY IS THE DESIGN INTENT: the CLI is the write surface; the UI is the read surface. Mutate controls in a read-only view create a false affordance and break the clean CLI-driven architecture. The design must name this explicitly so the implementer is not tempted to add controls "for convenience."
  - RESOLVED THREADS VISIBLE: resolved threads must be visible but visually muted (e.g. collapsed with a summary, or greyed). Hiding them removes review-history context. The operator or orchestrator may need to see that a concern was raised AND resolved.
  - MERGE GATE NOTE PLACEMENT: section 5 (the InfoMessage) must appear at the bottom of the view, below the lifecycle explanation, so the operator reads the data sections first and the context note last.
  - AGENT-AUTHORED BADGE ABSENCE: when agent_authored==false the badge is simply absent — do NOT render a 'Human' badge in the PR header. The human case is the default and needs no label. (Contrast with the Principals tab kind display in DESIGN-LPR-002, where omitted kind shows 'Human' to make the default explicit — in the PR header, the human case is the assumed default and the badge only appears for the non-default agent case.)
  - EMPTY STATE CLI CAPTION: the "No local review open" empty state caption references the CLI verb (`but review request <branch>`) but does NOT include a clickable action button — the view is read-only and the write path is CLI-only.
pattern: single read-only view with a status Badge driving four lifecycle states; five fixed data sections in order; agent-authored Badge in PR header (absent for human PRs); EmptyStatePlaceholder for no-review; InfoMessage for merge gate note; all mutate controls excluded; existing @gitbutler/ui components throughout; resolved threads muted but visible
pattern_source: GovernanceSettings.svelte (the four-tab surface); EmptyStatePlaceholder (the no-review empty state); InfoMessage (the merge gate note); tech-delta §A.3 (DerivedPrStatus derivation); LPR-005 (DerivedPr struct — the data contract)
anti_pattern: four separate views/tabs for the four lifecycle states; mutate controls (Approve/Comment/Assign buttons) in the view; hiding resolved threads instead of muting them; showing a 'Human' badge in the PR header when agent_authored==false; using the Approved/Mergeable label to imply the merge will succeed; hex color literals or new CSS variables; a new empty-state component instead of EmptyStatePlaceholder

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agent: frontend-designer
rationale: frontend-designer owns the IA and state contract for new Governance UI surfaces. The traps are: (a) designing four separate views instead of one multi-state view, (b) adding mutate controls to what must be a read-only view, (c) hiding resolved threads, (d) implying Approved/Mergeable means the merge will succeed, and (e) rendering a 'Human' badge in the PR header when agent_authored==false. This contract pins all five.
coding_standards: All sections reference existing @gitbutler/ui components. EmptyStatePlaceholder for empty states. InfoMessage (kind='info') for the merge gate note. No mutate controls — this is explicitly a read-only observer view. Resolved threads muted but visible. Agent-authored badge neutral styling. Lifecycle badge colors map to existing Badge variants only.

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: LPR-005 (the review_status backend read + DerivedPr struct + DerivedPrStatus — this contract describes the UI surface for the data LPR-005 yields); LPR-001 (the local_review_assignments table), LPR-004 (the local_review_comments table), LPR-003 (the local_review_meta opener row + agent-authored tag); DESIGN-MGMT-006 (the EmptyStatePlaceholder pattern this reuses for the no-review empty state)
Blocks:     LPR-015/LPR-016 (the sveltekit implementation tasks for the Local-Review view — this contract is the direct IA + state input for all rendering decisions)
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "DESIGN-LPR-003",
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
      "description": "GIVEN the Sprint 07 LPR section of DESIGN-ANNOTATIONS.md covers the Local-Review view WHEN a reviewer inspects the view IA specification THEN it specifies: one view component (not four separate views); a status Badge with four states (Draft/AwaitingReview/ChangesRequested/Approved-or-Mergeable); the five data sections in top-to-bottom order (PR header, Reviewer Assignments, Comment Threads, Lifecycle explanation, Merge gate note); and the read-only posture named explicitly ('the Local-Review view is an observer — reads and renders, never writes')",
      "verify": "design review — reviewer confirms single-view / four-state model, five data sections in order, read-only posture all present"
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the contract specifies the five data sections WHEN a reviewer reads each section specification THEN it specifies: Section 1 (PR header) — status Badge + branch display + agent-authored badge + sha/author/created_at/title; Section 2 (Reviewer Assignments) — one row per assignment with reviewer id + assignment state chip (Pending/Approved/ChangesRequested); Section 3 (Comment Threads) — threads grouped by file+line then PR-level, resolved threads muted/collapsed but visible, no Post Comment form; Section 4 (Lifecycle explanation) — one-sentence caption per state naming the CLI verb; Section 5 (Merge gate note) — InfoMessage kind='info' with the verbatim gate-re-derives-at-merge-time text",
      "verify": "design review — reviewer confirms all five section specifications with their visual treatments are present in the annotation"
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the contract specifies the agent-authored badge WHEN a reviewer inspects the badge specification THEN it states: the agent-authored Badge renders ONLY when agent_authored==true; neutral/secondary Badge styling (not success/green); tooltip text = 'This PR was opened by a principal declared as an agent in .gitbutler/permissions.toml. This is a metadata tag — it does not affect merge decisions.'; absent when agent_authored==false (no 'Human' badge in the PR header); placement in PR header after lifecycle Badge",
      "verify": "design review — reviewer confirms badge present/absent logic, neutral styling, tooltip text, and placement are all specified"
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the contract specifies empty and loading states WHEN a reviewer checks the empty/loading specifications THEN it specifies: loading = skeleton spanning the full view; no review open = EmptyStatePlaceholder with title='No local review open for this branch.' + CLI caption (no action button); no reviewers assigned = inline text within the assignments section; no comment threads = inline text within the threads section",
      "verify": "design review — reviewer confirms all four empty/loading specifications are present: loading skeleton, no-review EmptyStatePlaceholder, no-reviewers inline text, no-threads inline text"
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the contract specifies the read-only constraint WHEN a reviewer checks the mutate-control inventory THEN it explicitly lists all mutate controls that MUST NOT appear (Approve button, Request Changes button, Post Comment form, Assign Reviewer button, Resolve Thread button) with the reason: 'the Local-Review view is an observer — reads and renders, never writes; all writes are driven by the CLI'; AND the Merge gate note (section 5) is specified as InfoMessage kind='info' with verbatim text",
      "verify": "design review — reviewer confirms the mutate-control exclusion list is present with reasons, and the InfoMessage merge gate note is specified"
    },
    {
      "id": "AC-6",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the contract specifies the lifecycle status Badge styling WHEN a reviewer audits all visual references THEN every visual attribute uses an existing CSS variable — no hex literals, no new var(--review-*)/var(--lpr-*) tokens; lifecycle badge styling uses only existing @gitbutler/ui Badge variants: Draft=neutral, AwaitingReview=info/blue, ChangesRequested=warning/yellow, Approved=success/green, Mergeable=success/green; agent-authored badge=neutral/secondary; assignment chips: Pending=neutral, Approved=success/green, ChangesRequested=warning/yellow",
      "verify": "grep -nE '#[0-9a-fA-F]{3,6}|var\\(--(review|lpr)-' apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md -> zero matches in the LPR local-review section"
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "contract names single-view / four-state model; five data sections in top-to-bottom order; read-only posture named explicitly",
      "verify": "design review of the view IA specification in DESIGN-ANNOTATIONS.md",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "contract specifies all five section visual treatments (header fields, assignment chip states, thread grouping, collapsed-not-hidden resolved, lifecycle caption, InfoMessage merge gate note)",
      "verify": "design review of the five-section specification in DESIGN-ANNOTATIONS.md",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "contract names agent-authored Badge: neutral/secondary style, present only when agent_authored==true, absent when false, tooltip text present, placement in PR header after lifecycle Badge",
      "verify": "design review of the agent-authored badge specification in DESIGN-ANNOTATIONS.md",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "contract specifies all four empty/loading states: loading skeleton, no-review EmptyStatePlaceholder (with CLI caption, no action button), no-reviewers inline text, no-threads inline text",
      "verify": "design review of the empty/loading state specifications in DESIGN-ANNOTATIONS.md",
      "maps_to_ac": "AC-4"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "contract lists all excluded mutate controls with reasons; InfoMessage merge gate note specified with verbatim text",
      "verify": "design review of the read-only constraint + merge gate note section in DESIGN-ANNOTATIONS.md",
      "maps_to_ac": "AC-5"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "description": "zero hex color literals and zero new var(--review-*)/var(--lpr-*) tokens; lifecycle badge colors map to existing Badge variants only",
      "verify": "grep -nE '#[0-9a-fA-F]{3,6}|var\\(--(review|lpr)-' apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md -> zero matches",
      "maps_to_ac": "AC-6"
    }
  ]
}
-->
