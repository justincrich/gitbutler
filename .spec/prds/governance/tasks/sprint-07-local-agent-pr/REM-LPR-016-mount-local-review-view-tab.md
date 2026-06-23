# REM-LPR-016: Wire `<LocalReviewView>` into `GovernanceSettings.svelte` as the "Local Review" tab + extend backend `ReviewStatus` with PR-header fields

> Status: Backlog
> Reviewer: sveltekit-reviewer (UI mount) + rust-reviewer (backend DTO)
> Commit: (none yet)
> Updated: 2026-06-22T18:00:00Z
> Proposed-by: frontend-designer

## What this does

Close the two findings from the sprint-07 red-hat review (`.spec/reviews/red-hat-20260622-173510.md`):

- **C3 (CRITICAL)** — `LocalReviewView.svelte` is a fully-realized, 5-CT-spec Svelte 5 component that is **never mounted** in the live app. This task mounts it as the 5th tab ("Local Review") in `GovernanceSettings.svelte`, after the existing Rules tab.
- **M3 (HIGH)** — The component's `LocalReviewStatusPayload` type expects `source_branch/sha/author/title/created_at` to populate the PR-header section per DESIGN-LPR-003 Section 1. The backend `ReviewStatus` does not return them, so the header renders empty via `{#if}` guards. This task extends the Rust struct and populates the fields from the branch tip commit via a **read-only gix walk**.

The component itself (LPR-016), its children (`LocalReviewAssignments.svelte`, `LocalReviewThreads.svelte`), and its 5 CT specs are DONE — this task only mounts the component and feeds it the data its type already expects.

## Why

Sprint 07 red-hat review findings C3 + M3. The original LPR-016 task spec said *"mounts in GovernanceSettings as the Local Review tab or panel"* — that mount step was skipped, and the task status flag incorrectly claims Completed. SPRINT.md corroborates (*"the MGMT desktop render of the local PR is deferred"*). Additionally, DESIGN-LPR-003 Section 1 specifies a PR-header with `sha, author, created_at, title` fields, but the backend `ReviewStatus` payload lacks them. The component was written presciently against the design contract; the backend never caught up. This remediation closes both gaps.

## How to verify

PRIMARY **AC-1** — open the Project Settings → Governance modal, observe 5 tabs (Principals / Groups / Branch Gates / Rules / **Local Review**), click "Local Review", and confirm `<LocalReviewView>` renders with live assignments/threads/lifecycle data from the current branch.

## Scope

- `apps/desktop/src/components/governance/GovernanceSettings.svelte` (MODIFY — add the 5th "Local Review" tab via `<TabTrigger>` + `<TabContent>`, mounting `<LocalReviewView>`; add an optional `reviewBranch` prop)
- `crates/but-api/src/legacy/forge.rs` (MODIFY — extend `ReviewStatus` struct with 5 new `Option<String>` fields; populate them in `review_status` from the branch tip commit via gix read-only walk)
- `packages/but-sdk/src/generated/` (REGENERATED — via `pnpm build:sdk && pnpm format`; NEVER hand-edit)
- `apps/desktop/src/components/governance/LocalReviewView.svelte` (MINIMAL — only if SDK-type reconciliation needs a type import adjustment; the component already expects all 5 fields)
- `apps/desktop/tests/governance/GovernanceSettingsTabs.spec.ts` (MODIFY — extend the existing tab-count assertion from 4 to 5; add a click-Local-Review assertion)
- `crates/but-api/tests/local_review_status.rs` (MODIFY — add assertions for the 5 new fields)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: REM-LPR-016 — Mount LocalReviewView as the "Local Review" tab + extend backend ReviewStatus with PR-header fields
================================================================================

TASK_TYPE:   FEATURE (UI mount + backend DTO — closes the orphan-component gap)
STATUS:      Backlog
PRIORITY:    P1
EFFORT:      M  (90 min)
AGENT:       sveltekit-implementer (UI mount) + rust-implementer (backend DTO)
PROPOSED-BY: frontend-designer
SPRINT:      ./SPRINT.md (remediation cycle)
PRD_REFS:    UC-LPR-01, UC-LPR-02, UC-LPR-04, UC-LPR-05
CAPABILITIES:CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test_ui:   pnpm test:ct -- LocalReviewView && pnpm test:ct -- GovernanceSettings
  test_rust: cargo test -p but-api --test local_review_status
  sdk_regen: pnpm build:sdk && pnpm format
  check:     pnpm -F @gitbutler/desktop check && cargo check -p but-api --all-targets
  lint:      cargo clippy -p but-api --all-targets && cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Opening the Project Settings → Governance modal shows a 5th tab "Local Review" (after Rules). Clicking it renders <LocalReviewView> mounted against the current branch, showing: the lifecycle Badge + agent-authored tag, the PR-header section with SHA, author, title, and created-at date from the branch tip commit, reviewer assignments with state chips, comment threads, the lifecycle explanation caption, and the mandatory merge-gate InfoMessage note. The 5 existing CT specs for LocalReviewView still pass. The SDK regenerated with the 5 new optional fields. No merge button anywhere in the tab (READ-ONLY preserved).

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST mount <LocalReviewView> inside GovernanceSettings.svelte as a NEW 5th tab with the label "Local Review", positioned AFTER the existing "Rules" tab. Use the EXACT same <TabTrigger value="..."> + <TabContent value="..."> pattern as the existing four tabs (GovernanceSettings.svelte :182-250). The tab MUST be inside the existing <Tabs> component, not a parallel structure.
- [MUST] MUST extend the backend ReviewStatus struct (forge.rs:1018-1039) with EXACTLY these 5 new fields (all Option<String>, all nullable so existing callers don't break):
      pub source_branch: Option<String>,  // the queried branch ref
      pub sha: Option<String>,             // branch tip commit OID hex
      pub author: Option<String>,          // commit author identity (name + email)
      pub title: Option<String>,           // commit message summary (first line)
      pub created_at: Option<String>,      // commit author timestamp as RFC 3339 (ISO 8601) — STRING, not i64, to match the component's existing created_at?: string type
- [MUST] MUST populate those 5 fields in review_status (forge.rs:1062-1127) by reading the BRANCH TIP COMMIT — the same commit already peeled at forge.rs:1069-1075 (repo.find_reference(&ref_name).peel_to_commit()). Do NOT issue a second ref lookup; REUSE the commit reference. The read is gix READ-ONLY (no mutation, no index write, no ref update).
- [MUST] MUST run `pnpm build:sdk && pnpm format` after the Rust struct change to regenerate packages/but-sdk/src/generated/ with the 5 new optional fields on the ReviewStatus TS type. NEVER hand-edit generated files.
- [MUST] MUST verify LocalReviewView.svelte's existing LocalReviewStatusPayload type is compatible with the regenerated SDK ReviewStatus type. The component already declares all 5 fields as optional — they were prescient. After SDK regen, the types should align without component changes. If a type-import adjustment is needed, make the MINIMAL change.
- [MUST] MUST NOT add any merge affordance. Per SPRINT.md: "no merge affordance". The Local Review tab is READ-ONLY.
- [MUST] MUST NOT remove or weaken the mandatory merge-gate note (LocalReviewView.svelte:321-325 — the InfoMessage with MERGE_GATE_NOTE).
- [MUST] MUST NOT change the safe-seam (LPR-009). The 5 new fields are derived from COMMITS (read-only gix walk of the branch tip), NOT from the three drive tables. The merge-gate path gains NO new read.
- [MUST] MUST NOT introduce a parallel data path. LocalReviewView must use the EXISTING createLocalReviewService factory (LocalReviewView.svelte:118-150) which wires `review_status` + `list_comments` Tauri commands (LPR-015).
- [MUST] MUST pass the branch under review to LocalReviewView. Add an optional `reviewBranch?: string` prop to GovernanceSettings.svelte that flows to LocalReviewView's `branch` prop.
- [NEVER] NEVER add new gitbutler-* crate usage.
- [NEVER] NEVER modify LocalReviewAssignments.svelte or LocalReviewThreads.svelte — the children are DONE (LPR-016).
- [NEVER] NEVER modify the merge gate (merge_gate.rs), the review requirement (review_requirement.rs), or any safe-seam test.
- [NEVER] NEVER change the ReviewStatus fields that the merge gate or lifecycle derivation depend on. The 5 new fields are ADDITIVE and Optional.
- [NEVER] NEVER hand-edit packages/but-sdk/src/generated/. The SDK is generated via `pnpm build:sdk`.
- [STRICTLY] No relative imports — @gitbutler/ package references. Prettier: tabs, double quotes, no trailing commas, 100-col. No console.log.
- [STRICTLY] Svelte 5 $props()/$state()/$derived() rune syntax. No module-level state. No +page.server.ts (adapter-static).
- [STRICTLY] The new ReviewStatus fields MUST be serde::Serialize + serde::Deserialize + schemars::JsonSchema (the struct already derives all three).
- [STRICTLY] The gix commit read MUST use anyhow::Context on every step. If the commit read fails, the field is None (do NOT propagate the error — the review_status call must still succeed with the lifecycle data even if the header metadata read fails). Use a fallible helper that returns Option, swallowing extraction errors with a debug-level log or silent None.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: opening the Project Settings → Governance modal shows a 5th tab "Local Review" that renders <LocalReviewView> with live data
- [ ] AC-2: the PR-header section shows SHA, author, title, and created-at date from the branch tip commit (no longer empty — the 5 new ReviewStatus fields are populated and rendered)
- [ ] AC-3: the existing 5 CT specs for LocalReviewView still pass (the mount doesn't change the component's contract)
- [ ] AC-4: SDK regenerated; the ReviewStatus TS type carries the 5 new optional fields; pnpm typecheck clean
- [ ] AC-5: no merge button anywhere in the tab (READ-ONLY preserved); the mandatory merge-gate InfoMessage note still renders

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: GovernanceSettings shows 5 tabs including "Local Review"; clicking it renders LocalReviewView mounted against the current branch
  GIVEN: the GovernanceSettings modal is open with governance configured (not the first-run empty state) with a projectId and a reviewBranch (a branch that has a local review open)
  WHEN:  the user clicks the "Local Review" tab trigger
  THEN:  the LocalReviewView component renders inside the tab content area (data-testid="local-review-view" is visible); the lifecycle Badge, assignments section, threads section, lifecycle caption, and merge-gate note are all present; no error state is shown
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct -- GovernanceSettingsTabs

AC-2: LocalReviewView's PR-header section shows SHA + author + title + created_at from the branch tip commit (backend DTO extension)
  GIVEN: a governed repo with a branch that has a local review open, and the branch tip commit carries an author identity, a message summary, and a timestamp
  WHEN:  review_status (forge.rs:1062) runs against that branch
  THEN:  the returned ReviewStatus carries:
         - source_branch = Some(<the queried branch ref>)
         - sha = Some(<the branch tip commit OID hex>)
         - author = Some(<the commit author identity>)
         - title = Some(<the commit message summary / first line>)
         - created_at = Some(<RFC 3339 formatted commit time>)
         When LocalReviewView renders this payload, the PR-header <dl> shows all four fields — none hidden by the {#if} guards
  TEST_TIER: integration   VERIFICATION_SERVICE: cargo test (real but-db + real gix) for the backend; desktop-ct-harness for the render
  VERIFY: cargo test -p but-api --test local_review_status

AC-3: the existing 5 CT specs for LocalReviewView still pass (no regression from the mount)
  GIVEN: the LocalReviewView component (unchanged contract) and its 5 CT specs
  WHEN:  the CT suite runs after the mount is added
  THEN:  all 5 specs pass (the mount is in GovernanceSettings, NOT in LocalReviewView)
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct -- LocalReviewView

AC-4: SDK regenerated with the 5 new optional fields; pnpm typecheck clean
  GIVEN: the Rust ReviewStatus struct extended with source_branch, sha, author, title, created_at (all Option<String>)
  WHEN:  pnpm build:sdk && pnpm format runs
  THEN:  the generated ReviewStatus TS type carries all 5 new fields as `string | null`; pnpm -F @gitbutler/desktop check passes
  TEST_TIER: build   VERIFICATION_SERVICE: SDK generator + tsc
  VERIFY: pnpm build:sdk && pnpm format && pnpm -F @gitbutler/desktop check

AC-5: no merge button in the tab; the mandatory merge-gate InfoMessage note still renders (READ-ONLY preserved)
  GIVEN: the Local Review tab is open and LocalReviewView has rendered with a live review payload
  WHEN:  the tab's DOM is inspected
  THEN:  ZERO elements with role='button' matching 'Merge', 'Approve', 'Request Changes', 'Assign', 'Post', 'Comment', or 'Resolve' are present; the merge-gate InfoMessage (data-testid="local-review-merge-gate-note") IS present and contains the verbatim MERGE_GATE_NOTE text
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness + grep
  VERIFY: pnpm test:ct -- LocalReviewView (NoMutateControls spec) + grep audit

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): GovernanceSettingsTabs CT — the tab list shows 5 tabs including "Local Review"; clicking it renders data-testid="local-review-view"
- TC-2 (-> AC-2): cargo test — review_status returns source_branch, sha, author, title, created_at populated from the branch tip commit
- TC-3 (-> AC-3): LocalReviewView 5 CT specs still pass — no regression
- TC-4 (-> AC-4): SDK type carries the 5 new optional fields; typecheck clean
- TC-5 (-> AC-5): no merge/approve controls in the Local Review tab; merge-gate InfoMessage note present

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- cargo test -p but-api --test local_review_status   -> Exit 0 (4 existing + 1 new assertion; no regression)
- cargo clippy -p but-api --all-targets   -> Exit 0
- cargo fmt --check   -> Exit 0
- pnpm build:sdk && pnpm format   -> SDK updated with 5 new fields
- pnpm -F @gitbutler/desktop check   -> Exit 0 (typecheck clean)
- pnpm test:ct -- LocalReviewView   -> 5/5 PASS (no regression)
- pnpm test:ct -- GovernanceSettingsTabs   -> PASS (5 tabs + Local Review renders local-review-view)
- rg "source_branch\?:|sha\?:|author\?:|title\?:|created_at\?:" packages/but-sdk/src/generated/index.d.ts   -> matches on ReviewStatus
- rg -i 'role="button".*merge|role="button".*approve' apps/desktop/src/components/governance/GovernanceSettings.svelte   -> Exit 1 (zero merge/approve controls)

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - apps/desktop/src/components/governance/GovernanceSettings.svelte (MODIFY — add the 5th "Local Review" TabTrigger + TabContent mounting LocalReviewView; add the optional reviewBranch prop)
  - crates/but-api/src/legacy/forge.rs (MODIFY — extend ReviewStatus struct with 5 new Option<String> fields; populate them in review_status via a fallible gix helper; ADDITIVE only)
  - packages/but-sdk/src/generated/ (REGENERATED via pnpm build:sdk — ReviewStatus TS type gains the 5 new fields)
  - apps/desktop/src/components/governance/LocalReviewView.svelte (MINIMAL — only if a type-import adjustment is needed; do NOT change render logic, service factory, or props)
  - apps/desktop/tests/governance/GovernanceSettingsTabs.spec.ts (MODIFY — extend tab-count assertion from 4 to 5; add click-Local-Review assertion)
  - crates/but-api/tests/local_review_status.rs (MODIFY — add assertions for the 5 new fields)
writeProhibited:
  - apps/desktop/src/components/governance/LocalReviewAssignments.svelte — child is DONE
  - apps/desktop/src/components/governance/LocalReviewThreads.svelte — child is DONE
  - apps/desktop/tests/governance/LocalReviewView.spec.ts — 5 CT specs are DONE; must NOT change
  - crates/but-api/src/legacy/merge_gate.rs — CONSUME-only
  - crates/but-api/src/legacy/review_requirement.rs — CONSUME-only
  - crates/but-authz/tests/invariant_build_gates.rs — safe-seam grep must still pass
  - crates/but-api/tests/safe_seam.rs / safe_seam_invariant.rs — CONSUME-only
  - packages/ui/src/lib/components/** — read-only reuse
  - Any +page.server.ts or +layout.server.ts (adapter-static constraint)
  - Any gitbutler-* crate (no new gitbutler-* usage)
  - Any file not in write_allowed

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agents:
  - sveltekit-implementer: UI mount (GovernanceSettings.svelte 5th tab + GovernanceSettingsTabs.spec.ts extension). Owns AC-1, AC-3, AC-5.
  - rust-implementer: backend DTO extension (ReviewStatus struct + review_status fn + local_review_status.rs test). Owns AC-2.
  - Either: SDK regen (AC-4) — typically the rust-implementer runs it; sveltekit-implementer verifies the typecheck.
rationale: The UI mount is pure Svelte 5 component composition in an adapter-static SvelteKit app — sveltekit-implementer owns that surface. The backend DTO extension is Rust in but-api (gix read + struct + integration test) — rust-implementer owns that surface. The SDK regen is the bridge.

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: (none hard — all prerequisites are DONE)
  - LPR-015 (review_status + list_comments Tauri commands + regenerated SDK binding) — DONE
  - LPR-016 (LocalReviewView component + 5 CT specs) — DONE
  - DESIGN-LPR-003 (the Local-Review view IA + state contract) — DONE
  - LPR-005 (DerivedPr lifecycle + agent_authored derivation) — DONE
Blocks: nothing (this is a remediation task closing red-hat findings C3 + M3)
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "REM-LPR-016",
  "proposed_by": "frontend-designer",
  "verification_policy": { "requires_tests": true, "requires_red_evidence": true, "requires_seeded_evidence": true, "seam_stubs": [ { "ac": "AC-1", "kind": "component-mount", "reason": "the GovernanceSettingsTabs CT mounts GovernanceSettings via a harness; LocalReviewView renders inside it. The real Tauri-bus proof is LPR-015's (DONE).", "integration_bond": "LPR-015 AC-1", "status": "mount-only" }, { "ac": "AC-2", "kind": "integration", "reason": "real but-db + real gix via but_testsupport — the review_status call reads a real branch tip commit and returns the 5 new fields. No mocks.", "integration_bond": "none", "status": "real" } ] },
  "fixtures": {
    "governed_repo_with_review": {
      "description": "A real governed repo via but_testsupport::writable_scenario + invoke_bash committing .gitbutler/{permissions,gates}.toml. A branch 'feat' with at least one commit. A local review opened via governed request_review. review_status returns the full ReviewStatus including the 5 new PR-header fields.",
      "seed_method": "public_api",
      "records": [
        "but_testsupport::writable_scenario('checkout-head-info') + invoke_bash committing .gitbutler/{permissions,gates}.toml + creating the feat branch",
        "governed request_review(ctx, 'refs/heads/feat', Some('rev')) as opener principal",
        "but_api::legacy::forge::review_status(ctx, 'refs/heads/feat')"
      ]
    },
    "governance_settings_mount": {
      "description": "A Playwright CT mount of GovernanceSettings via the GovernanceSettingsHarness, with governance configured, a projectId, and a reviewBranch pointing to a branch with a local review. The LocalReviewView inside the 5th tab is fed pre-loaded review data.",
      "seed_method": "ui_flow",
      "records": [
        "GovernanceSettingsHarness mounted with configured governance + reviewBranch='feat/agent-review'",
        "LocalReviewView rendered inside the 5th tab with pre-loaded review data",
        "tab trigger 'Local Review' visible and clickable; clicking it renders data-testid='local-review-view'"
      ]
    }
  },
  "requirements": [
    { "id": "AC-1", "type": "acceptance_criterion", "primary": true, "description": "GIVEN GovernanceSettings modal open with governance configured + projectId + reviewBranch WHEN user clicks 'Local Review' tab trigger THEN LocalReviewView renders inside tab content area (data-testid='local-review-view' visible); lifecycle Badge, assignments, threads, lifecycle caption, and merge-gate note present; no error state", "verify": "pnpm test:ct -- GovernanceSettingsTabs", "scenario": { "tier": "visible", "test_tier": "integration", "verification_service": "desktop-ct-harness", "negative_control": { "would_fail_if": ["the 'Local Review' tab trigger is absent", "clicking 'Local Review' does not render data-testid='local-review-view'", "LocalReviewView renders an error state instead of review data", "only 4 tabs render"] }, "evidence": { "artifact_type": "screenshot", "required_capture": true }, "cases": [ { "start_ref": "governance_settings_mount", "action": { "actor": "user", "steps": ["mount GovernanceSettings via the harness", "observe the tab list shows 5 tabs", "click the 'Local Review' tab trigger", "observe the tab content area"] }, "end_state": { "must_observe": ["tab list shows 5 tabs: Principals, Groups, Branch Gates, Rules, Local Review", "data-testid='local-review-view' visible", "lifecycle Badge present", "assignments section present", "merge-gate InfoMessage note present"], "must_not_observe": ["only 4 tabs", "error state in the Local Review tab", "blank/empty tab content"] } } ] } },
    { "id": "AC-2", "type": "acceptance_criterion", "primary": false, "description": "GIVEN a governed repo with a branch with a local review open and the branch tip commit carries author/message/timestamp WHEN review_status runs THEN the returned ReviewStatus carries source_branch=Some(ref), sha=Some(OID hex), author=Some(identity), title=Some(summary), created_at=Some(RFC 3339); LocalReviewView renders the PR-header <dl> with all four fields", "verify": "cargo test -p but-api --test local_review_status", "scenario": { "tier": "holdout", "test_tier": "integration", "verification_service": "real but-api review_status + real but-db + real gix via but_testsupport", "negative_control": { "would_fail_if": ["source_branch is None", "sha is None (peel_to_commit result not reused)", "author is None", "title is None", "created_at is None", "the review_status call FAILED because header derivation propagated an error"] }, "evidence": { "artifact_type": "api_response", "required_capture": true }, "cases": [ { "start_ref": "governed_repo_with_review", "action": { "actor": "ci", "steps": ["open a local review via governed request_review on feat", "call review_status(ctx, 'refs/heads/feat')", "assert all 5 new fields are Some"] }, "end_state": { "must_observe": ["source_branch == Some('refs/heads/feat')", "sha is Some and non-empty", "author is Some and non-empty", "title is Some and non-empty", "created_at is Some and parses as RFC 3339"], "must_not_observe": ["any field is None", "the review_status call returning Err"] } } ] } },
    { "id": "AC-3", "type": "acceptance_criterion", "primary": false, "description": "GIVEN the LocalReviewView component (unchanged contract) and its 5 CT specs WHEN the CT suite runs after the mount is added THEN all 5 specs pass", "verify": "pnpm test:ct -- LocalReviewView", "scenario": { "tier": "visible", "test_tier": "integration", "verification_service": "desktop-ct-harness", "negative_control": { "would_fail_if": ["any of the 5 LocalReviewView CT specs broke", "the mount introduced a side effect that corrupts component CT isolation"] }, "evidence": { "artifact_type": "test_output", "required_capture": true }, "cases": [ { "start_ref": "governance_settings_mount", "action": { "actor": "ci", "steps": ["run pnpm test:ct -- LocalReviewView", "confirm all 5 specs pass"] }, "end_state": { "must_observe": ["5/5 LocalReviewView CT specs PASS"], "must_not_observe": ["any spec regression"] } } ] } },
    { "id": "AC-4", "type": "acceptance_criterion", "primary": false, "description": "GIVEN the Rust ReviewStatus struct extended with source_branch/sha/author/title/created_at (all Option<String>) WHEN pnpm build:sdk && pnpm format runs THEN the generated ReviewStatus TS type carries all 5 new fields as optional; pnpm -F @gitbutler/desktop check passes", "verify": "pnpm build:sdk && pnpm format && pnpm -F @gitbutler/desktop check", "scenario": { "tier": "visible", "test_tier": "build", "verification_service": "SDK generator + tsc", "negative_control": { "would_fail_if": ["the SDK was not regenerated", "the SDK was hand-edited", "typecheck fails because component's type is incompatible", "created_at type is number instead of string"] }, "evidence": { "artifact_type": "test_output", "required_capture": true }, "cases": [ { "start_ref": "governed_repo_with_review", "action": { "actor": "ci", "steps": ["run pnpm build:sdk && pnpm format", "grep the generated index.d.ts for the 5 new fields", "run pnpm -F @gitbutler/desktop check"] }, "end_state": { "must_observe": ["ReviewStatus TS type carries all 5 new fields", "pnpm -F @gitbutler/desktop check exits 0"], "must_not_observe": ["5 fields absent from SDK type", "typecheck error"] } } ] } },
    { "id": "AC-5", "type": "acceptance_criterion", "primary": false, "description": "GIVEN the Local Review tab is open and LocalReviewView has rendered with a live review payload WHEN the tab's DOM is inspected THEN ZERO elements with role='button' matching Merge/Approve/Request Changes/Assign/Post/Comment/Resolve are present; the merge-gate InfoMessage IS present with verbatim MERGE_GATE_NOTE text", "verify": "pnpm test:ct -- LocalReviewView (NoMutateControls spec) + grep audit", "scenario": { "tier": "visible", "test_tier": "integration", "verification_service": "desktop-ct-harness + grep", "negative_control": { "would_fail_if": ["a merge button is present (READ-ONLY violation)", "an approve/assign/comment/resolve control is present", "the merge-gate InfoMessage note is absent or altered"] }, "evidence": { "artifact_type": "screenshot", "required_capture": true }, "cases": [ { "start_ref": "governance_settings_mount", "action": { "actor": "ci", "steps": ["render the Local Review tab with a live review", "grep the tab DOM for merge/approve/assign/comment/resolve buttons", "confirm the merge-gate InfoMessage note is present"] }, "end_state": { "must_observe": ["zero merge/approve/assign/comment/resolve buttons", "data-testid='local-review-merge-gate-note' present with verbatim text"], "must_not_observe": ["any mutate control", "merge-gate note absent or altered"] } } ] } },
    { "id": "TC-1", "type": "test_criterion", "description": "GovernanceSettingsTabs CT — the tab list shows 5 tabs including 'Local Review'; clicking it renders data-testid='local-review-view'", "verify": "pnpm test:ct -- GovernanceSettingsTabs", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "cargo test — review_status returns source_branch, sha, author, title, created_at populated from the branch tip commit (real but-db + real gix)", "verify": "cargo test -p but-api --test local_review_status", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "LocalReviewView 5 CT specs still pass — no regression from the mount", "verify": "pnpm test:ct -- LocalReviewView", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "SDK type carries the 5 new optional fields; typecheck clean", "verify": "pnpm build:sdk && pnpm format && pnpm -F @gitbutler/desktop check", "maps_to_ac": "AC-4" },
    { "id": "TC-5", "type": "test_criterion", "description": "no merge/approve controls in the Local Review tab; merge-gate InfoMessage note present with verbatim text", "verify": "pnpm test:ct -- LocalReviewView (NoMutateControls) + rg -i 'merge|approve' apps/desktop/src/components/governance/GovernanceSettings.svelte", "maps_to_ac": "AC-5" }
  ]
}
-->
