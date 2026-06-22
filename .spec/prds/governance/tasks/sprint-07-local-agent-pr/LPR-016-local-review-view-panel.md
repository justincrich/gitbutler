# LPR-016: `LocalReviewView` panel — READ-ONLY Svelte view rendering the local PR object (assignments, comment threads, derived lifecycle, agent tag)

## What this does

Build `LocalReviewView.svelte` in the Governance section of the desktop app as a READ-ONLY view that renders the full local PR object surfaced by the LPR-015 SDK binding (`review_status` + `list_comments`). The view presents five data sections per DESIGN-LPR-003: (1) PR header with the derived lifecycle status Badge, the `agent-authored` tag (when `agent_authored==true`), branch name, and PR metadata; (2) reviewer assignments with their state (Pending / Approved / ChangesRequested), one row per assignment; (3) comment threads grouped by file+line and PR-level, with resolved threads muted/collapsed but visible; (4) a lifecycle explanation caption naming the CLI verb that advances it; and (5) an `InfoMessage` merge gate note. The view is one component with four lifecycle states (`Draft` / `AwaitingReview` / `ChangesRequested` / `Approved` / `Mergeable`) — not four separate screens — derived from the `DerivedPrStatus` the SDK returns. It includes empty states: a loading skeleton, `EmptyStatePlaceholder` for the no-review case, and inline text for zero-reviewers and zero-threads sub-sections. **READ-ONLY in every sense: no mutate controls, no Approve button, no Comment form, no Assign button, no Resolve button.** All writes are CLI-driven (`but review approve`, `but review comment`, `but review assign`, `but review resolve`).

## Why

Sprint 07 · PRD UC-LPR-01, UC-LPR-02, UC-LPR-04, UC-LPR-05 · capability CAP-AUTHZ-01. LPR-005 / LPR-008 deliver the `review_status` reconciler read — the derived PR lifecycle, reviewer assignments, unresolved comment threads, verdict-at-head, and the `agent-authored` tag — and LPR-004 delivers `list_comments`. LPR-015 exposes both on the Tauri bus with a regenerated SDK. This task is the human-readable surface: the desktop operator and the orchestrator share one source of truth via `review_status`, and this view makes that truth visible in the Governance UI without introducing any write path that could conflict with the CLI-driven review workflow.

## How to verify

PRIMARY **AC-1** — `pnpm test:ct:desktop -- LocalReviewViewAssignments`: mounting `LocalReviewView` with `seeded_local_review` (a branch with one `Pending` and one `Approved` assignment from `review_status`) renders one assignment row per entry with the correct state chip. Full gate set in the spec below.

## Scope

- `apps/desktop/src/components/governance/LocalReviewView.svelte` (NEW — the main view component)
- `apps/desktop/src/components/governance/LocalReviewAssignments.svelte` (NEW — the reviewer assignments section; one row per assignment with state chip)
- `apps/desktop/src/components/governance/LocalReviewThreads.svelte` (NEW — the comment threads section; file+line grouping, resolved/unresolved visual treatment)
- `apps/desktop/tests/governance/LocalReviewView.spec.ts` (NEW — CT specs for all five ACs)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: LPR-016 — LocalReviewView panel (READ-ONLY; lifecycle, assignments, threads, agent tag)
================================================================================

TASK_TYPE:   FEATURE
STATUS:      Backlog
PRIORITY:    P1
EFFORT:      L  (150 min)
AGENT:       sveltekit-implementer
PROPOSED-BY: sveltekit-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-LPR-01, UC-LPR-02, UC-LPR-04, UC-LPR-05
CAPABILITIES:CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  pnpm test:ct:desktop -- LocalReviewViewAssignments
  check: pnpm -F @gitbutler/desktop check
  lint:  pnpm lint

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Mounting LocalReviewView with seeded_local_review (a branch with a `pending`
assignment for rev2, an `approved` assignment for rev3, one unresolved thread
t1, one resolved thread t2, derived lifecycle `AwaitingReview`, and
agent_authored=true in the review_status payload) renders: the lifecycle
status Badge with label 'Awaiting Review', the agent-authored Badge (neutral
style, no badge when agent_authored=false), two assignment rows with correct
state chips (Pending=neutral, Approved=success/green), two thread entries (t1
muted/collapsed as resolved, t2 shown unresolved), the lifecycle explanation
caption naming the CLI verb, and the InfoMessage merge gate note. Mounting with
seeded_no_review renders EmptyStatePlaceholder with title='No local review open
for this branch.' and no mutate controls anywhere in the DOM. Mounting with
seeded_loading renders a skeleton placeholder. All four lifecycle states
(Draft / AwaitingReview / ChangesRequested / Approved-or-Mergeable) render the
correct Badge variant (neutral / info / warning / success). pnpm
test:ct:desktop -- LocalReviewView passes. pnpm -F @gitbutler/desktop check
and pnpm lint pass.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] THIS VIEW IS READ-ONLY. It renders NO mutate controls: no Approve
  button, no Request Changes button, no Post Comment form, no Assign Reviewer
  button, no Resolve Thread button. All writes are CLI-driven. The view is an
  observer — it reads and renders, never writes. AC-5 (the no-mutate-controls
  assertion) and the verification grep enforce this. Any mutate control is a
  design violation.
- [MUST] ONE VIEW, FOUR LIFECYCLE STATES. LocalReviewView is a SINGLE component
  that renders one local PR object and reflects the current DerivedPrStatus via
  a status Badge. There are NOT four separate views, NOT a tab per lifecycle
  state. The Badge changes; the five data sections remain. AC-3 proves all four
  states render the correct Badge and all five sections remain present.
- [MUST] FIVE DATA SECTIONS IN ORDER (per DESIGN-LPR-003). The view renders
  top-to-bottom:
    1. PR header: lifecycle status Badge + agent-authored Badge (if
       agent_authored==true; ABSENT if false — no 'Human' badge) + branch name
       display (source → target) + sha, author, created_at, title (read-only text)
    2. Reviewer Assignments: one row per assignment with reviewer principal id +
       assignment state chip (Pending=neutral / Approved=success / Changes
       Requested=warning). No assign/remove controls.
    3. Comment Threads: threads grouped by file+line (code-scoped) then PR-level
       (file==null); each thread shows comments in time order; resolved threads
       are MUTED/COLLAPSED but VISIBLE (not hidden). Thread resolution state
       shown via a resolved icon or muted styling. No Post Comment form, no
       Resolve button.
    4. Lifecycle Explanation: a one-sentence plain-text caption per lifecycle
       state explaining what it means and naming the CLI verb that advances it.
    5. Merge Gate Note: InfoMessage kind='info' with text 'Merge decisions are
       made by the merge gate, not this view. A status of Approved or Mergeable
       here reflects the derived state — the gate re-derives verdict at merge
       time.' (verbatim per DESIGN-LPR-003). Must be the LAST section.
- [MUST] AGENT-AUTHORED BADGE RULE: render the agent-authored Badge ONLY when
  review_status returns agent_authored==true. Use neutral/secondary Badge
  styling (NOT success/green — it is a descriptor, not a status). Include a
  tooltip: 'This PR was opened by a principal declared as an agent in
  .gitbutler/permissions.toml. This is a metadata tag — it does not affect
  merge decisions.' When agent_authored==false, the badge is ABSENT — do NOT
  render a 'Human' badge. Placement: in the PR header, after the lifecycle
  status Badge, before the branch name. AC-1 (agent_authored=true case) and
  AC-3 (all four lifecycle states) enforce this.
- [MUST] EMPTY STATES per DESIGN-LPR-003:
    (a) Loading: a skeleton/placeholder spanning the full view height while the
        review_status SDK call is in flight. AC-4 proves this.
    (b) No review open: EmptyStatePlaceholder (packages/ui) with title='No
        local review open for this branch.' and caption 'Open one with `but
        review request <branch>` to start the review loop.' NO primary action
        button (the action is a CLI command). Use the SAME EmptyStatePlaceholder
        component used across all Governance tabs (DESIGN-MGMT-006 pattern).
        AC-4 proves this.
    (c) No reviewers assigned: inline text 'No reviewers assigned yet.' within
        the Reviewer Assignments section.
    (d) No comment threads: inline text 'No comment threads yet.' within the
        Comment Threads section.
- [MUST] RESOLVED THREADS VISIBLE, NOT HIDDEN. Resolved comment threads MUST
  appear in the view but with a visually muted/collapsed treatment (e.g. grey
  opacity, collapsed with a summary, or a resolved icon). Hiding them removes
  review-history context the operator or orchestrator may need.
- [MUST] LIFECYCLE BADGE COLORS use ONLY existing @gitbutler/ui Badge variants:
  Draft=neutral, AwaitingReview=info/blue, ChangesRequested=warning/yellow,
  Approved=success/green, Mergeable=success/green. NO hex literals, NO new CSS
  variables. Assignment chips: Pending=neutral, Approved=success/green,
  ChangesRequested=warning/yellow. Agent-authored badge=neutral/secondary.
- [MUST] CONSUME the LPR-015 SDK binding (review_status + list_comments)
  exclusively. The view reads from the SDK; it writes nothing. ErrorBoundary per
  the 6b governance-page pattern (an error in the SDK call renders an error
  state, not a crash).
- [NEVER] NEVER add +page.server.ts or +layout.server.ts (adapter-static
  constraint).
- [NEVER] NEVER use module-level state. Use Svelte 5 $props()/$state()/$derived()
  rune syntax throughout.
- [NEVER] NEVER render mutate controls (Approve, Request Changes, Post Comment,
  Assign Reviewer, Resolve Thread). The verification grep catches any button/form
  with a write action.
- [NEVER] NEVER render a 'Human' badge in the PR header when agent_authored==false.
  The human case is the assumed default; the badge only appears for the agent case.
- [NEVER] NEVER describe Approved/Mergeable as authorizing a merge in UI copy.
  The InfoMessage merge gate note (section 5) must clarify the gate re-derives
  at merge time.
- [NEVER] NEVER add four separate views or tabs for the four lifecycle states.
- [STRICTLY] No relative imports — @gitbutler/ package references. No console.log.
  Prettier: tabs, double quotes, no trailing commas, 100-col.
- [STRICTLY] CT describe blocks MUST use the component name as the outermost
  describe string (e.g. describe('LocalReviewView', () => {...})) so
  `pnpm test:ct:desktop -- <ComponentName>` grep matches reliably.
- [STRICTLY] STRICTLY resolved threads must be visible but muted/collapsed —
  never a conditional that entirely removes them from the DOM.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: reviewer assignments with their state render from seeded review
- [ ] AC-2: resolved vs unresolved comment threads render with correct visual treatment
- [ ] AC-3: derived lifecycle state + agent-authored tag render for all four states
- [ ] AC-4: empty states (loading, no-review, zero-assignments, zero-threads) render
- [ ] AC-5: no mutate controls are present anywhere in the view's DOM
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: reviewer assignments render with their state from a seeded local review
  SCOPE NOTE: AC-1 verifies that LocalReviewView renders the assignments returned
  by the review_status SDK binding correctly. The seeded_local_review fixture
  provides the SDK call result at component-test scope (a spy returning a
  well-defined payload). This proves the component is wired to the SDK and renders
  correctly. Full end-to-end proof that the SDK call reaches the real Tauri bus
  and the real but-db is LPR-015's acceptance criteria (the producer task). The
  component-test scope seam is legitimate here because LPR-015 owns the bus proof.
  GIVEN: LocalReviewView mounted with seeded_local_review (review_status SDK spy
         resolves: DerivedPrStatus='AwaitingReview', agent_authored=true, two
         assignments — rev2 state='pending', rev3 state='approved' — one
         unresolved thread t1 (file='src/main.rs', line=42), one resolved thread
         t2 (PR-level, file=null); list_comments spy resolves both threads)
  WHEN:  the component renders
  THEN:  the Reviewer Assignments section contains exactly two rows; the row for
         rev2 has a state chip with accessible text 'Pending' (neutral styling);
         the row for rev3 has a state chip with accessible text 'Approved'
         (success/green styling); no assign/remove/reject control is present in
         either row; the component made exactly 1 review_status SDK call
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- LocalReviewViewAssignments

AC-2: resolved vs unresolved comment threads render with correct visual treatment
  GIVEN: LocalReviewView mounted with seeded_local_review (t1 unresolved,
         file='src/main.rs', line=42; t2 resolved, PR-level, file=null; same
         review_status and list_comments spies as AC-1)
  WHEN:  the component renders
  THEN:  both threads appear in the Comment Threads section (t2 NOT hidden);
         thread t1 (unresolved) renders with normal/full styling; thread t2
         (resolved) renders with a visually muted/collapsed treatment (e.g. a
         resolved icon, reduced opacity, or collapsed summary — any of these is
         acceptable); a resolved indicator (icon or text) is present on t2;
         no Post Comment form and no Resolve button are present anywhere in the
         Comment Threads section
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- LocalReviewViewThreads

AC-3: all four lifecycle states render the correct Badge and the agent-authored tag
  GIVEN: LocalReviewView mounted four times, once per lifecycle state:
         (a) seeded_draft_review (DerivedPrStatus='Draft', agent_authored=false)
         (b) seeded_awaiting_review (DerivedPrStatus='AwaitingReview', agent_authored=true)
         (c) seeded_changes_requested (DerivedPrStatus='ChangesRequested', agent_authored=false)
         (d) seeded_approved_review (DerivedPrStatus='Approved', agent_authored=true)
  WHEN:  each variant renders
  THEN:  (a) Draft: the lifecycle Badge has accessible text 'Draft' and neutral
             styling; no agent-authored badge in the DOM
         (b) AwaitingReview: the lifecycle Badge has accessible text containing
             'Awaiting' and info/blue styling; the agent-authored Badge is present
             with neutral/secondary styling; the PR header contains the agent-
             authored Badge tooltip text 'This PR was opened by a principal
             declared as an agent in .gitbutler/permissions.toml. This is a
             metadata tag — it does not affect merge decisions.'
         (c) ChangesRequested: the lifecycle Badge has accessible text containing
             'Changes' and warning/yellow styling; no agent-authored badge
         (d) Approved: the lifecycle Badge has accessible text 'Approved' and
             success/green styling; the agent-authored badge is present; the
             InfoMessage merge gate note (section 5) is present with text
             containing 'the gate re-derives verdict at merge time'
         All four variants have five data sections present in order; no mutate
         controls in any variant
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- LocalReviewViewLifecycle

AC-4: empty states (loading, no-review, zero-assignments, zero-threads) render correctly
  GIVEN: LocalReviewView mounted under four empty-state conditions:
         (a) seeded_loading (review_status SDK spy is pending/unresolved — the
             in-flight state)
         (b) seeded_no_review (review_status SDK spy resolves null / no review
             open for the branch)
         (c) seeded_zero_assignments (review_status resolves: DerivedPrStatus=
             'Draft', open_assignments=[], unresolved_threads=[t1], agent_authored=false)
         (d) seeded_zero_threads (review_status resolves: DerivedPrStatus='Draft',
             open_assignments=[rev2 pending], unresolved_threads=[], agent_authored=false;
             list_comments resolves [])
  WHEN:  each variant renders
  THEN:  (a) seeded_loading: a loading skeleton/placeholder element is present
             spanning the full view; the five data sections are NOT rendered
             (content is absent while in-flight)
         (b) seeded_no_review: EmptyStatePlaceholder renders with title text
             containing 'No local review open for this branch.'; caption text
             contains 'but review request'; NO primary action button is present
             (the action is CLI-only); the five data sections are NOT rendered
         (c) seeded_zero_assignments: the Reviewer Assignments section renders
             the inline text 'No reviewers assigned yet.'; the other sections
             (threads, lifecycle caption, merge gate note) are present
         (d) seeded_zero_threads: the Comment Threads section renders the inline
             text 'No comment threads yet.'; the other sections are present
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- LocalReviewViewEmptyStates

AC-5: no mutate controls are present in the view
  GIVEN: LocalReviewView mounted with seeded_local_review (the full review with
         assignments, threads, lifecycle)
  WHEN:  the component renders
  THEN:  the component's DOM contains ZERO elements with: role='button' AND
         text/aria-label matching any of 'Approve', 'Request Changes', 'Assign',
         'Post', 'Comment', 'Resolve'; ZERO <form> elements; ZERO <textarea>
         elements; ZERO <input type='text'> or <input type='submit'> elements
         that could constitute a comment form or assignment form; the SDK write
         spy (any mutation verb) is called == 0 times during render and after
         any user interaction with the read-only content
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- LocalReviewViewNoMutateControls

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): seeded_local_review → two assignment rows; rev2 chip='Pending'
    (neutral); rev3 chip='Approved' (success/green); 0 assign/remove controls;
    review_status SDK spy called == 1
    VERIFY: pnpm test:ct:desktop -- LocalReviewViewAssignments
- TC-2 (-> AC-2): both t1 (unresolved) and t2 (resolved) present in the DOM;
    t2 has muted/collapsed visual treatment and a resolved indicator; 0 Post
    Comment forms; 0 Resolve buttons
    VERIFY: pnpm test:ct:desktop -- LocalReviewViewThreads
- TC-3 (-> AC-3): four lifecycle mounts → correct Badge accessible text + color
    variant per state; agent-authored Badge present only when agent_authored==true
    (neutral/secondary, not success/green); tooltip text verbatim; InfoMessage
    merge gate note present in all four variants; five sections in all four
    variants; 0 mutate controls in any variant
    VERIFY: pnpm test:ct:desktop -- LocalReviewViewLifecycle
- TC-4 (-> AC-4): loading → skeleton present, sections absent; no-review →
    EmptyStatePlaceholder with title + CLI caption, no action button; zero-
    assignments → 'No reviewers assigned yet.' inline, other sections present;
    zero-threads → 'No comment threads yet.' inline, other sections present
    VERIFY: pnpm test:ct:desktop -- LocalReviewViewEmptyStates
- TC-5 (-> AC-5): seeded_local_review → 0 Approve/Comment/Assign/Resolve/
    RequestChanges buttons or forms in DOM; all SDK mutation spies called == 0
    VERIFY: pnpm test:ct:desktop -- LocalReviewViewNoMutateControls

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides:
  - apps/desktop/src/components/governance/LocalReviewView.svelte — a READ-ONLY
    view rendering the full local PR object (lifecycle, assignments, threads,
    agent-authored tag, merge gate note) from the LPR-015 SDK binding. Mounts
    in the Governance section as the Local Review tab or panel. No mutate
    controls. All four DerivedPrStatus lifecycle states rendered as a single
    multi-state view.
  - apps/desktop/src/components/governance/LocalReviewAssignments.svelte — the
    reviewer assignments section (one row per assignment with state chip;
    read-only).
  - apps/desktop/src/components/governance/LocalReviewThreads.svelte — the
    comment threads section (file+line grouping, resolved muted/collapsed but
    visible; read-only).
consumes:
  - LPR-015 (the review_status + list_comments Tauri commands + the regenerated
    SDK binding — the data source for all five sections; MUST land before this
    task can bind the SDK calls)
  - DESIGN-LPR-003 (the IA + state contract for the Local-Review view — the
    direct design input for all rendering decisions; the implementer must
    transcribe this contract, not invent the IA)
  - LPR-005 (DerivedPr struct + DerivedPrStatus enum: the payload shape this
    view renders; the agent_authored derivation)
  - LPR-008 (the full reconciler drive state in the review_status payload:
    open_assignments + unresolved_threads + verdict_at_head)
  - LPR-014 (context for the agent-authored badge: LPR-014 sets principal
    kind='agent' in permissions.toml; LPR-005 derives agent_authored from it;
    this view renders the tag — understanding the read surface is prerequisite
    context per LPR-014's DEPENDS section which states LPR-016 as a downstream)
  - LPR-012 (context: keep_reviews_local gates whether a local review exists;
    when false no local review is created and the no-review empty state fires)
  - packages/ui: Badge, InfoMessage (kind='info'), EmptyStatePlaceholder,
    Tooltip (for the agent-authored badge tooltip)
  - apps/desktop/src/components/governance/GovernanceSettings.svelte — the
    four-tab Governance surface this view slots into; the isReadOnly prop
    pattern; the ErrorBoundary pattern from the 6b governance surface
  - DESIGN-MGMT-006 (the EmptyStatePlaceholder pattern for the no-review empty
    state — same component, same usage as the four Governance tab empty states)
boundary_contracts:
  - CAP-AUTHZ-01: LocalReviewView is a READ-ONLY consumer of the LPR-015 SDK
    reads (review_status + list_comments). It carries NO write authority, calls
    NO mutation verb, and renders NO mutate control. The view cannot affect the
    merge gate or the review drive state. The agent-authored badge is purely
    informational (a descriptor, not a status); it gates nothing and has no
    interactive affordance.
  - The LocalReviewView is an observer: reads and renders, never writes. All
    writes are driven by the CLI (`but review approve`, `but review comment`,
    `but review assign`, `but review resolve`).
  - The lifecycle status Badge (section 1) reflects the derived DerivedPrStatus
    from the SDK payload. It is a PRESENTATION LABEL — it does not authorize a
    merge. The InfoMessage merge gate note (section 5) names this explicitly.
  - Resolved threads are visible (muted/collapsed) — never hidden. Hiding
    removes review-history context.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - apps/desktop/src/components/governance/LocalReviewView.svelte (NEW — main
    view: five data sections, four lifecycle states, empty states, error boundary;
    mounts in GovernanceSettings as the Local Review tab/panel)
  - apps/desktop/src/components/governance/LocalReviewAssignments.svelte (NEW —
    the reviewer assignments section molecule)
  - apps/desktop/src/components/governance/LocalReviewThreads.svelte (NEW —
    the comment threads section molecule; file+line grouping; resolved muted)
  - apps/desktop/tests/governance/LocalReviewView.spec.ts (NEW — CT specs for
    all five ACs)
writeProhibited:
  - apps/desktop/src/components/governance/GovernanceSettings.svelte — the mount
    point; add the LocalReviewView tab/panel slot if needed, but do NOT modify
    the existing tab content or pending-store logic beyond the mount
  - packages/but-sdk/src/generated — SDK regen is LPR-015 / LPR-010; NEVER
    hand-edit
  - Any +page.server.ts or +layout.server.ts
  - packages/ui/src/lib/components/** — read-only reuse; no new design-system
    tokens
  - Any Rust crate — this is a SvelteKit frontend task; the reads are owned by
    LPR-004/005/008/015
  - Any file not in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. .spec/prds/governance/tasks/sprint-07-local-agent-pr/DESIGN-LPR-003-local-review-view-contract.md
   — [PRIMARY DESIGN CONTRACT] the full IA + state contract for this view: one
   view / four states, five data sections in order, agent-authored badge spec
   (neutral styling, present only when agent_authored==true, tooltip text verbatim),
   four empty/loading states, read-only posture, mutate-control exclusion list,
   InfoMessage merge gate note, and the anti-pattern inventory. The implementer
   MUST transcribe this contract; do not invent the IA.
2. packages/but-sdk/src/generated/ (after LPR-015 regenerates the SDK) — the
   review_status / list_comments TypeScript bindings: the ReviewStatus /
   DerivedPr types (DerivedPrStatus enum, agent_authored, open_assignments,
   unresolved_threads, verdict_at_head) and LocalReviewComment. These are the
   TS types this view imports and renders.
3. apps/desktop/src/components/governance/GovernanceSettings.svelte [1-60] —
   the four-tab Governance surface: how tabs are added, the isReadOnly prop,
   the pendingStore context (NOT needed for a read-only view, but read for
   the ErrorBoundary pattern and the EmptyStatePlaceholder usage). LocalReviewView
   mounts as the fifth tab or as a panel in the Governance section — coordinate
   the mount point with the GovernanceSettings slot structure.
4. apps/desktop/src/components/governance/PrincipalsList.svelte [144-235] —
   the assignment row pattern and the Badge component usage (kind, style, size)
   for the reviewer assignment chip. The state chip for Pending/Approved/
   ChangesRequested follows the same Badge import pattern.
5. packages/ui/src/lib/components/EmptyStatePlaceholder.svelte [1-50] — the
   empty state API (title, caption, action slot) for the no-review-open empty
   state. Confirm the API: title='No local review open for this branch.',
   caption='Open one with `but review request <branch>` to start the review
   loop.', NO action button (the action is a CLI command).
6. packages/ui/src/lib/components/InfoMessage.svelte — the merge gate note
   component (kind='info'). Confirm the kind prop values before using.
7. .spec/prds/governance/tasks/sprint-07-local-agent-pr/LPR-005-derived-pr-lifecycle-agent-tag.md
   §DerivedPr struct + DerivedPrStatus enum — the exact payload field names
   (target_branch, source_branch, sha, author, title, draft, created_at,
   updated_at, status, agent_authored, labels, open_assignments,
   unresolved_thread_count). Use these as the TS type reference until the
   regenerated SDK is available.
8. .spec/prds/governance/tasks/sprint-07-local-agent-pr/LPR-015-local-review-read-tauri-sdk-producer.md
   §TAURI IPC CONTRACT — the SDK invoke shape:
   `invoke<ReviewStatus>('review_status', { projectId, branch })` and
   `invoke<LocalReviewComment[]>('list_comments', { projectId, branch })`.
   Use these EXACTLY so the calls wire to the registered commands.
9. .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-009-branch-gates-list.md
   [CT spec section] — the desktop CT harness pattern (MGMT-UI-001), the
   describe-block naming rule, the seeded_* fixture pattern, the must_observe /
   must_not_observe assertion style, and the negative_control (would_fail_if)
   discipline used across all governance component tests.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- pnpm test:ct:desktop -- LocalReviewViewAssignments   -> Exit 0
- pnpm test:ct:desktop -- LocalReviewViewThreads   -> Exit 0
- pnpm test:ct:desktop -- LocalReviewViewLifecycle   -> Exit 0
- pnpm test:ct:desktop -- LocalReviewViewEmptyStates   -> Exit 0
- pnpm test:ct:desktop -- LocalReviewViewNoMutateControls   -> Exit 0
- pnpm -F @gitbutler/desktop check   -> Exit 0
- pnpm lint   -> Exit 0
- grep -rn 'role="button".*Approve\|role="button".*Resolve\|role="button".*Assign\|<form\|<textarea\|<input.*type="submit"' \
    /Users/justinrich/Projects/gitbutler/apps/desktop/src/components/governance/LocalReviewView.svelte \
    /Users/justinrich/Projects/gitbutler/apps/desktop/src/components/governance/LocalReviewAssignments.svelte \
    /Users/justinrich/Projects/gitbutler/apps/desktop/src/components/governance/LocalReviewThreads.svelte \
    | wc -l | grep '^0$'   -> Exit 0 (no mutate controls in any of the three component files)
- grep -rn '#[0-9a-fA-F]\{3,6\}\|var(--(review\|lpr)-' \
    /Users/justinrich/Projects/gitbutler/apps/desktop/src/components/governance/LocalReviewView.svelte \
    | wc -l | grep '^0$'   -> Exit 0 (no hex literals, no new CSS variables)

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references:
  - DESIGN-LPR-003 (the single authoritative IA + state contract — the implementer
    transcribes this, does not invent the IA; the design contract WINS over any
    other copy/layout source)
  - apps/desktop/src/components/governance/GovernanceSettings.svelte (four-tab
    surface mount point + isReadOnly prop + ErrorBoundary pattern)
  - apps/desktop/src/components/governance/PrincipalsList.svelte:178-187 (the
    Badge component usage — kind/style/size props — for state chips)
  - packages/ui/src/lib/components/EmptyStatePlaceholder.svelte (no-review
    empty state — title + CLI caption; no action button)
  - packages/ui/src/lib/components/InfoMessage.svelte (merge gate note,
    kind='info')
notes:
  - SDK calls: use a Svelte 5 $state / $effect pattern for the
    loading/error/data trichotomy (loading=true while in-flight; data=null for
    no-review; data=ReviewStatus for the rendered state). While loading=true,
    render the skeleton. On data==null after load (no review open), render
    EmptyStatePlaceholder. On error, render the ErrorBoundary error state
    (consistent with the 6b governance page pattern).
  - FIVE SECTIONS ALWAYS PRESENT (except during loading and no-review empty
    states). When assignments or threads are empty, the section still renders
    with its inline empty-text ('No reviewers assigned yet.' / 'No comment
    threads yet.') — the section heading/container is always present.
  - LIFECYCLE EXPLANATION CAPTIONS (one per status, section 4):
      Draft: 'This review is in draft. Request a reviewer via `but review assign`.'
      AwaitingReview: 'Awaiting review. Reviewers can approve via `but review approve`.'
      ChangesRequested: 'Changes requested. Address comments and push; reviewers
        can re-review via `but review approve`.'
      Approved: 'Approved. The branch is ready for the merge gate.'
      Mergeable: 'Mergeable. All conditions are met; attempt merge via `but merge`.'
    These captions follow DESIGN-LPR-003's intent; the design contract wins if
    DESIGN-LPR-003 specifies different text.
  - RESOLVED THREAD PATTERN: use a CSS class (e.g. 'thread--resolved') to apply
    reduced opacity or a collapsed summary. The resolved icon can be a Badge
    kind='soft' style='success' with text 'Resolved' or an SVG icon. The key
    rule: the thread is VISIBLE but MUTED — it is in the DOM.
  - AGENT-AUTHORED BADGE: `{#if review.agent_authored}<Badge kind='soft'
    style='neutral'>agent-authored</Badge>{/if}`. Add a tooltip via the Tooltip
    component from @gitbutler/ui with the verbatim text from DESIGN-LPR-003.
    When agent_authored==false: the `{#if}` branch is absent — no badge rendered.
  - ASSIGNMENT STATE CHIP: one Badge per assignment in LocalReviewAssignments.svelte:
      state=='pending' → Badge kind='soft' style='neutral' text='Pending'
      state=='approved' → Badge kind='soft' style='success' text='Approved'
      state=='changes_requested' → Badge kind='soft' style='warning'
        text='Changes Requested'
    No interactive affordance on the chip (no role='button', no click handler).
  - isReadOnly: if GovernanceSettings passes isReadOnly=true as a prop, the view
    renders identically (it is already read-only by design — there is nothing
    extra to disable). Document the prop as accepted but inert for this view.
pattern: READ-ONLY view with a status Badge driving four lifecycle states; five
  fixed data sections in order; agent-authored Badge in PR header (absent for
  human PRs); LocalReviewAssignments molecule for reviewer rows (state chips);
  LocalReviewThreads molecule for thread grouping (resolved muted but visible);
  EmptyStatePlaceholder for no-review; skeleton for loading; InfoMessage merge
  gate note; ErrorBoundary; no mutate controls; existing @gitbutler/ui components
  only; no new design-system tokens.
pattern_source: DESIGN-LPR-003 (the design contract); GovernanceSettings.svelte
  (mount point + error boundary); PrincipalsList.svelte:178-187 (Badge usage);
  EmptyStatePlaceholder (no-review empty state); InfoMessage (merge gate note).
anti_pattern: four separate views/tabs per lifecycle state (one view, four
  states); any mutate control (Approve / Request Changes / Post Comment / Assign /
  Resolve); hiding resolved threads instead of muting them; rendering a 'Human'
  badge when agent_authored==false; stating Approved/Mergeable means the merge
  will succeed (the gate re-derives at merge time); hex color literals or new CSS
  variables; module-level state; +page.server.ts; mocking the SDK at a level
  that disconnects from the LPR-015 binding (use the SDK spy at the component-
  test seam, not a hardcoded stub that bypasses the SDK call entirely).

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agent: sveltekit-implementer
rationale: Net-new Svelte view in the Governance section of the desktop app,
  consuming the LPR-015 SDK binding (review_status + list_comments). The primary
  competencies required: (a) adapter-static SvelteKit component work (no server
  routes, no SSR data flows), (b) Svelte 5 rune syntax throughout, (c) the
  @gitbutler/ui component library (Badge, InfoMessage, EmptyStatePlaceholder,
  Tooltip), (d) the governance-page CT harness pattern (MGMT-UI-001, the 6b
  seeded-fixture idiom), and (e) the read-only constraint discipline (the single
  biggest trap: an implementer may be tempted to add mutation affordances "for
  convenience" — the design contract and the AC-5 no-mutate assertion guard
  against this). sveltekit-implementer owns adapter-static component work for
  apps/desktop.
coding_standards: No relative imports — @gitbutler/ package references; Prettier
  tabs, double quotes, no trailing commas, 100-col; no console.log; Svelte 5
  $props()/$state()/$derived() rune syntax; CT describe blocks use the component
  name as the outermost describe string; no module-level state; no +page.server.ts;
  no hex color literals; no new CSS variables.

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: LPR-015 (the review_status + list_comments Tauri commands + the
  regenerated @gitbutler/but-sdk binding that surfaces the full drive state to
  the desktop — this view CANNOT bind the SDK calls until LPR-015 ships the
  registered commands and the regenerated SDK; this is the hard blocking
  dependency)
Depends on: DESIGN-LPR-003 (the Local-Review view IA + state contract — the
  direct design input for all rendering and layout decisions; the implementer
  must transcribe this contract)
Depends on: LPR-005 (the DerivedPr struct + DerivedPrStatus enum + the
  agent_authored derivation — the payload shape this view renders; LPR-015
  exposes it on the desktop bus)
Depends on: LPR-008 (the full reconciler drive state in the review_status
  payload — open_assignments + unresolved_threads + verdict_at_head —
  LPR-015 exposes it on the desktop bus)
Blocks:     none (this is the terminal UI task in the local-review read surface)
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "LPR-016",
  "proposed_by": "sveltekit-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true,
    "seam_stubs": [
      { "ac": "AC-1", "kind": "component-spy", "reason": "the LPR-015 review_status/list_comments SDK binding is spied at desktop-ct (component) scope; the real Tauri-bus proof is LPR-015's", "integration_bond": "LPR-015 AC-1", "status": "fixture-only" }
    ],
    "fixture_only_until": "LPR-015 (component-spy ACs are NOT integrated-done until the LPR-015 real-bus producer lands)"
  },
  "fixtures": {
    "seeded_local_review": {
      "description": "The review_status SDK spy resolves: { status: 'AwaitingReview', agent_authored: true, target_branch: 'refs/heads/main', source_branch: 'refs/heads/feat/agent-review', sha: 'abc1234', author: 'agent:codex', title: 'feat: add review module', draft: false, created_at: '2026-06-01T00:00:00Z', open_assignments: [ { principalId: 'rev2', state: 'pending' }, { principalId: 'rev3', state: 'approved' } ], unresolved_threads: [ { threadId: 't1', file: 'src/main.rs', line: 42, resolved: false, comments: [ { body: 'Consider using Option here', author: 'rev2', created_at: '2026-06-01T01:00:00Z' } ] } ], verdict_at_head: null, approved: false }. The list_comments SDK spy resolves: [ { threadId: 't1', file: 'src/main.rs', line: 42, resolved: false, body: 'Consider using Option here', author: 'rev2' }, { threadId: 't2', file: null, line: null, resolved: true, body: 'LGTM overall', author: 'rev3' } ].",
      "seed_method": "ui_flow",
      "records": [
        "review_status spy resolves { status: 'AwaitingReview', agent_authored: true, open_assignments: [ {principalId:'rev2',state:'pending'}, {principalId:'rev3',state:'approved'} ], unresolved_threads: [ {threadId:'t1',file:'src/main.rs',line:42,resolved:false} ], verdict_at_head: null, approved: false, source_branch: 'refs/heads/feat', target_branch: 'refs/heads/main', sha: 'abc1234', author: 'agent:codex', title: 'feat: add review module', draft: false, created_at: '2026-06-01T00:00:00Z' }",
        "list_comments spy resolves [ {threadId:'t1',file:'src/main.rs',line:42,resolved:false,body:'Consider using Option here',author:'rev2'}, {threadId:'t2',file:null,line:null,resolved:true,body:'LGTM overall',author:'rev3'} ]"
      ]
    },
    "seeded_no_review": {
      "description": "The review_status SDK spy resolves null indicating no open review for the branch. The component must render EmptyStatePlaceholder.",
      "seed_method": "ui_flow",
      "records": [
        "review_status spy resolves null (no open review for the branch)",
        "list_comments spy resolves []"
      ]
    },
    "seeded_loading": {
      "description": "The review_status SDK spy is a Promise that does not resolve during the test assertion window (simulating the in-flight loading state). The component must render the skeleton placeholder and NOT render the five data sections.",
      "seed_method": "ui_flow",
      "records": [
        "review_status spy returns a Promise that does not resolve within the test assertion window",
        "list_comments spy returns a Promise that does not resolve within the test assertion window"
      ]
    },
    "seeded_draft_review": {
      "description": "review_status spy resolves: { status: 'Draft', agent_authored: false, open_assignments: [], unresolved_threads: [], verdict_at_head: null, approved: false, source_branch: 'refs/heads/draft', target_branch: 'refs/heads/main', sha: 'def5678', author: 'human:alice', title: 'WIP: draft feature', draft: true, created_at: '2026-06-01T00:00:00Z' }. list_comments spy resolves [].",
      "seed_method": "ui_flow",
      "records": [
        "review_status spy resolves { status: 'Draft', agent_authored: false, open_assignments: [], unresolved_threads: [], draft: true }",
        "list_comments spy resolves []"
      ]
    },
    "seeded_awaiting_review": {
      "description": "review_status spy resolves: { status: 'AwaitingReview', agent_authored: true, open_assignments: [ {principalId:'rev2',state:'pending'} ], unresolved_threads: [], verdict_at_head: null, approved: false, source_branch: 'refs/heads/feat', target_branch: 'refs/heads/main', sha: 'abc1234', author: 'agent:codex', title: 'feat: new module', draft: false, created_at: '2026-06-01T00:00:00Z' }. list_comments spy resolves [].",
      "seed_method": "ui_flow",
      "records": [
        "review_status spy resolves { status: 'AwaitingReview', agent_authored: true, open_assignments: [{principalId:'rev2',state:'pending'}] }"
      ]
    },
    "seeded_changes_requested": {
      "description": "review_status spy resolves: { status: 'ChangesRequested', agent_authored: false, open_assignments: [ {principalId:'rev2',state:'changes_requested'} ], unresolved_threads: [{threadId:'t3',file:'lib.rs',line:10,resolved:false}], verdict_at_head: null, approved: false, source_branch: 'refs/heads/feat', target_branch: 'refs/heads/main', sha: 'fed9012', author: 'human:bob', title: 'fix: address review feedback', draft: false, created_at: '2026-06-01T00:00:00Z' }. list_comments spy resolves [{threadId:'t3',file:'lib.rs',line:10,resolved:false,body:'Needs rework',author:'rev2'}].",
      "seed_method": "ui_flow",
      "records": [
        "review_status spy resolves { status: 'ChangesRequested', agent_authored: false, open_assignments: [{principalId:'rev2',state:'changes_requested'}] }"
      ]
    },
    "seeded_approved_review": {
      "description": "review_status spy resolves: { status: 'Approved', agent_authored: true, open_assignments: [ {principalId:'rev2',state:'approved'} ], unresolved_threads: [], verdict_at_head: { verdict: 'approved', head_oid: 'abc1234' }, approved: true, source_branch: 'refs/heads/feat', target_branch: 'refs/heads/main', sha: 'abc1234', author: 'agent:codex', title: 'feat: approved feature', draft: false, created_at: '2026-06-01T00:00:00Z' }. list_comments spy resolves [].",
      "seed_method": "ui_flow",
      "records": [
        "review_status spy resolves { status: 'Approved', agent_authored: true, approved: true, open_assignments: [{principalId:'rev2',state:'approved'}] }"
      ]
    },
    "seeded_zero_assignments": {
      "description": "review_status spy resolves: { status: 'Draft', agent_authored: false, open_assignments: [], unresolved_threads: [{threadId:'t1',file:'src/main.rs',line:42,resolved:false}], verdict_at_head: null, approved: false, source_branch: 'refs/heads/feat', target_branch: 'refs/heads/main', sha: 'abc1234', author: 'human:alice', title: 'feat: some feature', draft: false, created_at: '2026-06-01T00:00:00Z' }. list_comments spy resolves [{threadId:'t1',file:'src/main.rs',line:42,resolved:false,body:'Review comment',author:'rev2'}].",
      "seed_method": "ui_flow",
      "records": [
        "review_status spy resolves { status: 'Draft', agent_authored: false, open_assignments: [], unresolved_threads: [{threadId:'t1',file:'src/main.rs',line:42,resolved:false}] }"
      ]
    },
    "seeded_zero_threads": {
      "description": "review_status spy resolves: { status: 'Draft', agent_authored: false, open_assignments: [{principalId:'rev2',state:'pending'}], unresolved_threads: [], verdict_at_head: null, approved: false, source_branch: 'refs/heads/feat', target_branch: 'refs/heads/main', sha: 'abc1234', author: 'human:alice', title: 'feat: some feature', draft: false, created_at: '2026-06-01T00:00:00Z' }. list_comments spy resolves [].",
      "seed_method": "ui_flow",
      "records": [
        "review_status spy resolves { status: 'Draft', agent_authored: false, open_assignments: [{principalId:'rev2',state:'pending'}], unresolved_threads: [] }",
        "list_comments spy resolves []"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN seeded_local_review (review_status spy: AwaitingReview, agent_authored=true, rev2 pending, rev3 approved; list_comments spy: t1 unresolved, t2 resolved) WHEN LocalReviewView renders THEN the Reviewer Assignments section contains exactly two rows; rev2 row has state chip 'Pending' (neutral); rev3 row has state chip 'Approved' (success/green); no assign/remove controls; review_status SDK spy called == 1",
      "verify": "pnpm test:ct:desktop -- LocalReviewViewAssignments",
      "scenario": {
        "id": "SC-LPR-016-1",
        "primary": true,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "only 1 assignment row rendered (rev3 row absent — spy not consumed or list truncated)",
            "rev2 chip shows 'Approved' instead of 'Pending' (state mapping incorrect)",
            "rev3 chip shows 'Pending' instead of 'Approved' (state mapping incorrect)",
            "review_status SDK spy not called (component renders static/hardcoded rows disconnected from the SDK)",
            "an Assign or Remove button present in any row (mutate control leaked — violates read-only constraint)"
          ]
        },
        "evidence": { "artifact_type": "screenshot", "required_capture": true },
        "cases": [
          {
            "start_ref": "seeded_local_review",
            "action": { "actor": "user", "steps": [ "mount LocalReviewView with seeded_local_review", "observe the Reviewer Assignments section" ] },
            "end_state": {
              "must_observe": [
                "exactly 2 assignment rows in the Reviewer Assignments section",
                "rev2 row has a chip/badge with accessible text 'Pending' (neutral styling)",
                "rev3 row has a chip/badge with accessible text 'Approved' (success/green styling)",
                "review_status SDK spy called == 1 time"
              ],
              "must_not_observe": [
                "1 assignment row (rev3 absent)",
                "an Assign, Remove, or Reject button in any assignment row",
                "rev2 chip with text 'Approved' or 'Changes Requested' (wrong state)",
                "review_status SDK spy called == 0 (component not wired to SDK)"
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
      "description": "GIVEN seeded_local_review (list_comments spy: t1 unresolved file='src/main.rs' line=42; t2 resolved PR-level file=null) WHEN LocalReviewView renders THEN both t1 and t2 appear in the Comment Threads section (t2 NOT hidden); t1 renders with normal styling; t2 renders with muted/collapsed treatment and a resolved indicator; no Post Comment form or Resolve button anywhere in the Comment Threads section",
      "verify": "pnpm test:ct:desktop -- LocalReviewViewThreads",
      "scenario": {
        "id": "SC-LPR-016-2",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "t2 (resolved) is absent from the DOM (resolved thread hidden instead of muted — violation of the visible-but-muted requirement)",
            "t1 and t2 have identical styling (resolved thread not visually distinguished)",
            "a Post Comment form or textarea is present in the Comment Threads section (mutate control)",
            "a Resolve or Unresolve button is present in the Comment Threads section (mutate control)"
          ]
        },
        "evidence": { "artifact_type": "screenshot", "required_capture": true },
        "cases": [
          {
            "start_ref": "seeded_local_review",
            "action": { "actor": "user", "steps": [ "mount LocalReviewView with seeded_local_review", "observe the Comment Threads section" ] },
            "end_state": {
              "must_observe": [
                "t1 (unresolved) present in the Comment Threads section with normal/full styling",
                "t2 (resolved) present in the Comment Threads section (NOT hidden from DOM)",
                "t2 has a visually muted/collapsed treatment (e.g. resolved icon, reduced opacity, or collapsed summary)",
                "a resolved indicator (icon, 'Resolved' text, or Badge) associated with t2",
                "0 Post Comment forms or textarea elements in the Comment Threads section",
                "0 Resolve or Unresolve buttons in the Comment Threads section"
              ],
              "must_not_observe": [
                "t2 absent from the DOM (hidden — violation of the resolved-visible requirement)",
                "t1 and t2 with identical visual styling (no distinction between resolved and unresolved)",
                "a Post Comment textarea (write control in a read-only section)",
                "a Resolve button (write control in a read-only section)"
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
      "description": "GIVEN LocalReviewView mounted four times (seeded_draft_review, seeded_awaiting_review, seeded_changes_requested, seeded_approved_review) WHEN each renders THEN: Draft → Badge text 'Draft' neutral styling, no agent-authored badge; AwaitingReview → Badge text containing 'Awaiting' info/blue, agent-authored Badge present (neutral/secondary, tooltip verbatim), InfoMessage merge gate note present; ChangesRequested → Badge text containing 'Changes' warning/yellow, no agent-authored badge; Approved → Badge text 'Approved' success/green, agent-authored badge present, InfoMessage present; all four have five data sections; 0 mutate controls in any variant",
      "verify": "pnpm test:ct:desktop -- LocalReviewViewLifecycle",
      "scenario": {
        "id": "SC-LPR-016-3",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "all four lifecycle states render the same Badge text/color (lifecycle state not reflected)",
            "agent-authored Badge absent when agent_authored=true (badge logic missing)",
            "agent-authored Badge present when agent_authored=false (badge rendered for wrong case)",
            "agent-authored Badge has success/green styling instead of neutral/secondary (wrong styling)",
            "InfoMessage merge gate note absent in any lifecycle state (section 5 not rendered)",
            "fewer than five data sections in any lifecycle state (sections conditional on lifecycle)"
          ]
        },
        "evidence": { "artifact_type": "screenshot", "required_capture": true },
        "cases": [
          {
            "start_ref": "seeded_draft_review",
            "action": { "actor": "user", "steps": [ "mount with seeded_draft_review", "assert Badge text, styling, agent-authored badge absence" ] },
            "end_state": {
              "must_observe": [ "lifecycle Badge text 'Draft' with neutral styling", "0 agent-authored Badge elements" ],
              "must_not_observe": [ "lifecycle Badge with info/blue or success/green styling", "an agent-authored Badge when agent_authored=false" ]
            }
          },
          {
            "start_ref": "seeded_awaiting_review",
            "action": { "actor": "user", "steps": [ "mount with seeded_awaiting_review", "assert Badge, agent-authored badge, tooltip, merge gate note" ] },
            "end_state": {
              "must_observe": [
                "lifecycle Badge text containing 'Awaiting' with info/blue styling",
                "agent-authored Badge present with neutral/secondary styling (not success/green)",
                "tooltip text containing 'declared as an agent in .gitbutler/permissions.toml'",
                "InfoMessage kind='info' containing 'the gate re-derives verdict at merge time'"
              ],
              "must_not_observe": [ "agent-authored Badge with success/green styling", "InfoMessage absent" ]
            }
          },
          {
            "start_ref": "seeded_changes_requested",
            "action": { "actor": "user", "steps": [ "mount with seeded_changes_requested", "assert Badge and agent-authored badge absence" ] },
            "end_state": {
              "must_observe": [ "lifecycle Badge text containing 'Changes' with warning/yellow styling", "0 agent-authored Badge elements" ],
              "must_not_observe": [ "agent-authored Badge when agent_authored=false" ]
            }
          },
          {
            "start_ref": "seeded_approved_review",
            "action": { "actor": "user", "steps": [ "mount with seeded_approved_review", "assert Badge, agent-authored badge, merge gate note" ] },
            "end_state": {
              "must_observe": [
                "lifecycle Badge text 'Approved' with success/green styling",
                "agent-authored Badge present with neutral/secondary styling",
                "InfoMessage kind='info' containing 'the gate re-derives verdict at merge time'"
              ],
              "must_not_observe": [ "InfoMessage with kind='warning' or kind='error'", "lifecycle Badge with neutral or warning styling" ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN LocalReviewView mounted under four empty-state conditions (seeded_loading, seeded_no_review, seeded_zero_assignments, seeded_zero_threads) WHEN each renders THEN: loading → skeleton present, sections absent; no-review → EmptyStatePlaceholder with title='No local review open for this branch.' + CLI caption 'but review request', no action button; zero-assignments → 'No reviewers assigned yet.' in assignments section, other sections present; zero-threads → 'No comment threads yet.' in threads section, other sections present",
      "verify": "pnpm test:ct:desktop -- LocalReviewViewEmptyStates",
      "scenario": {
        "id": "SC-LPR-016-4",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "loading state renders the five data sections instead of a skeleton",
            "no-review state renders empty sections instead of EmptyStatePlaceholder (wrong empty state component)",
            "no-review EmptyStatePlaceholder includes a primary action button (write affordance in a read-only view)",
            "no-review caption does not contain 'but review request' (wrong CLI verb)",
            "seeded_zero_assignments renders the assignment section hidden instead of 'No reviewers assigned yet.' inline text"
          ]
        },
        "evidence": { "artifact_type": "screenshot", "required_capture": true },
        "cases": [
          {
            "start_ref": "seeded_loading",
            "action": { "actor": "user", "steps": [ "mount with seeded_loading (SDK spy pending)", "observe loading state" ] },
            "end_state": {
              "must_observe": [ "a loading skeleton/placeholder element spanning the view", "five data sections NOT rendered" ],
              "must_not_observe": [ "five data sections rendered during the loading state" ]
            }
          },
          {
            "start_ref": "seeded_no_review",
            "action": { "actor": "user", "steps": [ "mount with seeded_no_review (review_status resolves null)", "observe empty state" ] },
            "end_state": {
              "must_observe": [
                "EmptyStatePlaceholder with title text containing 'No local review open for this branch.'",
                "caption text containing 'but review request'",
                "0 primary action buttons in the EmptyStatePlaceholder"
              ],
              "must_not_observe": [ "a primary action Button inside the EmptyStatePlaceholder", "five data sections when no review exists" ]
            }
          },
          {
            "start_ref": "seeded_zero_assignments",
            "action": { "actor": "user", "steps": [ "mount with seeded_zero_assignments", "observe Reviewer Assignments section" ] },
            "end_state": {
              "must_observe": [ "Reviewer Assignments section present (not hidden)", "inline text 'No reviewers assigned yet.'", "other three sections present" ],
              "must_not_observe": [ "Reviewer Assignments section absent" ]
            }
          },
          {
            "start_ref": "seeded_zero_threads",
            "action": { "actor": "user", "steps": [ "mount with seeded_zero_threads", "observe Comment Threads section" ] },
            "end_state": {
              "must_observe": [ "Comment Threads section present (not hidden)", "inline text 'No comment threads yet.'", "other three sections present" ],
              "must_not_observe": [ "Comment Threads section absent" ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN seeded_local_review WHEN LocalReviewView renders THEN the DOM contains ZERO mutate controls: no Approve/RequestChanges/PostComment/Assign/Resolve buttons or forms; no <form> elements; no <textarea> elements; all SDK mutation spies called == 0 during render and after any user interaction",
      "verify": "pnpm test:ct:desktop -- LocalReviewViewNoMutateControls",
      "scenario": {
        "id": "SC-LPR-016-5",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "any button with accessible name containing 'Approve', 'Request Changes', 'Assign', 'Comment', or 'Resolve' is present (mutate control leaked)",
            "a <form> or <textarea> element is present (comment form leaked)",
            "any SDK mutation spy (assign_reviewer, post_comment, approve_review, resolve_thread) is called during render or on user interaction"
          ]
        },
        "evidence": { "artifact_type": "screenshot", "required_capture": true },
        "cases": [
          {
            "start_ref": "seeded_local_review",
            "action": { "actor": "user", "steps": [ "mount LocalReviewView with seeded_local_review", "query all button and form elements in the DOM", "attempt to click on assignment rows and thread entries", "observe SDK spy call counts" ] },
            "end_state": {
              "must_observe": [
                "0 button elements with accessible name matching 'Approve', 'Request Changes', 'Assign', 'Post', 'Comment', or 'Resolve'",
                "0 <form> elements in the view DOM",
                "0 <textarea> elements in the view DOM",
                "all SDK mutation spies (assign_reviewer, post_comment, approve_review, resolve_thread) called == 0 times"
              ],
              "must_not_observe": [
                "any button with write semantics in the view",
                "any form or textarea element in the view",
                "any SDK mutation call triggered by the read-only view"
              ]
            }
          }
        ]
      }
    },
    { "id": "TC-1", "type": "test_criterion", "description": "seeded_local_review → 2 assignment rows; rev2 chip='Pending' (neutral); rev3 chip='Approved' (success/green); 0 assign/remove controls; review_status SDK spy called == 1", "verify": "pnpm test:ct:desktop -- LocalReviewViewAssignments", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "both t1 (unresolved) and t2 (resolved) present in DOM; t2 muted/collapsed with resolved indicator; 0 Post Comment forms; 0 Resolve buttons", "verify": "pnpm test:ct:desktop -- LocalReviewViewThreads", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "four lifecycle mounts → correct Badge accessible text + variant per state; agent-authored Badge present only when agent_authored==true (neutral/secondary, not success/green); tooltip text verbatim; InfoMessage merge gate note present in all four variants; five sections in all four variants; 0 mutate controls", "verify": "pnpm test:ct:desktop -- LocalReviewViewLifecycle", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "loading → skeleton present, sections absent; no-review → EmptyStatePlaceholder with title + CLI caption, no action button; zero-assignments → 'No reviewers assigned yet.' inline, other sections present; zero-threads → 'No comment threads yet.' inline, other sections present", "verify": "pnpm test:ct:desktop -- LocalReviewViewEmptyStates", "maps_to_ac": "AC-4" },
    { "id": "TC-5", "type": "test_criterion", "description": "seeded_local_review → 0 Approve/Comment/Assign/Resolve/RequestChanges buttons or forms in DOM; all SDK mutation spies called == 0", "verify": "pnpm test:ct:desktop -- LocalReviewViewNoMutateControls", "maps_to_ac": "AC-5" }
  ]
}
-->
