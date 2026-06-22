# E2E-MGMT-UI-001: Playwright capstone — governance Svelte UI (web target) against real but-server, 6-step gate

## What this does

A single Playwright spec `e2e/playwright/tests/governance-capstone.spec.ts` that drives the **real** `apps/desktop` governance Svelte UI built for the **web target** against a **real `but-server`**, covering all 6 human-test-gate steps from sprint-06b. Split into an **ADMIN** test block (steps 1, 2, 5, 6) and a **NON-ADMIN** block (steps 3, 4), each spawning its own `but-server` with the appropriate `BUT_AGENT_HANDLE`. This is the capstone "live product proof" that replaces manual click-through — it fails if any step's real backend behavior is missing, stubbed, mocked, or served by the React fixture.

## Why

Sprint 06b · PRD UC-MGMT-06/07 · capability CAP-AUTHZ-01, CAP-CONFIG-01 · criteria T-MGMT-032 + T-MGMT-041 (the two `[e2e-automated]` MGMT criteria). The user's bar: "if we can't play all the functionality, it failed." This proves the four-tab governance surface — branch gates, principal-scoped rules, read-only enforcement, self-escalation no-flip, keyboard a11y, and IPC failure/retry — works end-to-end against real services. NOT the React `governance-app` mock ("Not product E2E evidence"); NOT Tauri (Playwright can't drive it); the **web** build of the same Svelte UI against a real `but-server`.

## How to verify

PRIMARY **AC-4** (e2e) — `pnpm test:e2e:playwright -- governance-capstone -g "step4"`: as NONADMIN_HANDLE, attempting to self-grant `administration:write` returns a real `perm.denied`, the danger denial banner appears, and the `administration:write` control's `aria-checked` is UNCHANGED (false) — proving the no-bypass invariant from the consumer side. Full suite: `pnpm test:e2e:playwright -- governance-capstone`.

## Scope

- `e2e/playwright/tests/governance-capstone.spec.ts` (NEW — the sole deliverable)
- `e2e/playwright/src/governance.ts` (consume-only if E2E-MGMT-BE-001 already exports `ADMIN_HANDLE`/`NONADMIN_HANDLE`)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: E2E-MGMT-UI-001 — Playwright capstone (governance Svelte UI, web target, real but-server)
================================================================================

TASK_TYPE:   FEATURE
STATUS:      Backlog
PRIORITY:    P0
EFFORT:      L  (180 min)
AGENT:       sveltekit-implementer
PROPOSED-BY: sveltekit-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-MGMT-06, UC-MGMT-07
CAPABILITIES: CAP-AUTHZ-01, CAP-CONFIG-01

RUNTIME_COMMANDS:
  test:  pnpm test:e2e:playwright -- governance-capstone
  check: pnpm check
  lint:  pnpm lint

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Single spec e2e/playwright/tests/governance-capstone.spec.ts covering all 6 human-gate steps, split
into an ADMIN test block (steps 1,2,5,6) and a NONADMIN block (steps 3,4). Each block spawns a real
but-server with the appropriate BUT_AGENT_HANDLE. All assertions are against the real web-target Svelte
governance UI — not mocks, not the React fixture, not Tauri. pnpm test:e2e:playwright -- governance-capstone
passes; pnpm check and pnpm lint pass.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] Drive the REAL web-target Svelte governance UI (apps/desktop) against a REAL but-server using
  the existing e2e/playwright harness (startGitButler / gitbutlerOptions / the per-test fixture in
  e2e/playwright/src/test.ts:34-52).
- [MUST] Group steps by identity into separate test() blocks, each with its own
  test.use({ gitbutlerOptions: { env: { BUT_AGENT_HANDLE } } }) — ADMIN block (steps 1,2,5,6) and
  NONADMIN block (steps 3,4). The gitbutler fixture is per-test; identity cannot switch within one test().
- [MUST] Navigate to Permissions & Governance via page.getByRole('button', { name: 'Permissions & Governance' })
  — the SettingsModalLayout sidebar button has NO data-testid; getByRole/getByText is the ONLY honest locator.
- [MUST] Establish the admin user object before the nav renders (web target: isAdmin = userService.user?.role
  === 'admin', ProjectSettingsModalContent.svelte:34) via setCookie('user', JSON.stringify({role:'admin'}),
  context) (setup.ts:283) so SettingsModalLayout.svelte:53 does not filter out the Governance item. Record U3
  if unsatisfiable. NONADMIN blocks STILL set this cookie (visibility) and STILL see a read-only page.
- [MUST] AC-1 asserts branch_gates_update POST returns HTTP 2xx with a non-error body via page.on('response')
  — not merely that a request fired (pre-BE-002 returns "Command not found").
- [MUST] AC-3 asserts governance_status_read returns hasAdminWrite=false in the body — not merely that the
  read-only InfoMessage renders (a hardcoded isReadOnly=true stub would fake the latter).
- [MUST] AC-4 acting handle is NONADMIN_HANDLE; assert the administration:write control's aria-checked is
  UNCHANGED (false) after the denial. Banner-visible + pending-no-increment alone is insufficient.
- [MUST] AC-4 also includes an ADMIN-identity T-MGMT-032 sub-case: toggle gate → switch tab → assert pending
  PERSISTS → Commit → assert badge CLEARS after 2xx.
- [MUST] AC-6 includes a persistent-failure sub-case: keep page.route() 500 active, click Retry, assert the
  error banner STILL visible + controls remain disabled (T-MGMT-040), THEN clear route + Retry → success.
- [MUST] AC-5 asserts Home/End wrapping and that the previously-selected tab's aria-selected becomes false.
- [MUST] Wrap page.route() in try/finally with page.unroute() in finally; assert the IPC error banner is NOT
  visible BEFORE injection (must_not_observe pre-condition).
- [MUST] Use ONLY testids that exist in 06a OR are explicit cross-task commitments named in NOTES (with a
  documented role/aria/text fallback locator). NEVER assert governance-nav-item.
- [NEVER] NEVER drive e2e/playwright/fixtures/governance-app (the React mock — "Not product E2E evidence").
- [NEVER] NEVER use page.evaluate() to force client state — the ONLY sanctioned injection is page.route() at
  the HTTP layer (step 6 only).
- [NEVER] NEVER assert an admin self-grant produces a denial — an admin holds administration:write, so the
  governed self-grant stages as pending, it does not deny. Only a NON-admin produces real perm.denied.
- [NEVER] NEVER use a single test() that switches BUT_AGENT_HANDLE between steps.
- [NEVER] NEVER use *_as_fleet_owner; NEVER add +page.server.ts/+layout.server.ts.
- [STRICTLY] Every assertion must be something a stub/empty shell/React mock/no-flip violation would visibly
  FAIL — never assert only that a DOM element exists; assert its state (aria-checked, HTTP body, badge count,
  visible text).
- [STRICTLY] Mirror the e2e/playwright/tests/branches.spec.ts pattern (helper fns, getByTestId/clickByTestId/
  waitForTestId from src/util.ts, gitbutler.runScript() seeding). No relative imports; no console.log;
  Prettier tabs/double-quotes/no-trailing-commas/100-col.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 (ADMIN, step 1): toggle Branch Gate → pending badge + branch_gates_update 2xx non-error body
- [ ] AC-2 (ADMIN, step 2): Rules principal scoping — A only, then A absent after B; scoped HTTP call
- [ ] AC-3 (NONADMIN, step 3): read-only — governance_status_read hasAdminWrite=false + controls disabled
- [ ] AC-4 [PRIMARY] (NONADMIN denial + ADMIN pending sub-case): self-grant perm.denied + aria-checked unchanged; pending persists across tabs + clears on commit
- [ ] AC-5 (ADMIN, step 5): keyboard — ArrowRight x3 + Home/End; aria-selected toggles; panels switch
- [ ] AC-6 (ADMIN, step 6): IPC 500 → banner + retry; persistent failure stays read-only; clear route + Retry → success

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each with a real start→end scenario; e2e tier)
--------------------------------------------------------------------------------
AC-1 [ADMIN] — Branch Gates toggle → pending + real HTTP 2xx
  GIVEN ADMIN server; seed committed; admin user cookie set; Branch Gates tab active
  WHEN  the user toggles the master gate's protected control
  THEN  branch_gates_update POST returns HTTP 2xx with a NON-error body (page.on('response')); pending banner
        visible with count>0; governance-commit-button enabled
  VERIFY pnpm test:e2e:playwright -- governance-capstone -g "step1"
  TEST_TIER e2e   VERIFICATION_SERVICE real but-server + web-target Svelte UI
  NEGATIVE CONTROL would fail if: React mock driven (no HTTP fires); but-server returns "Command not found"
    (BE-002 not landed); toggle is a no-op; pending banner hardcoded (count would not increment).

AC-2 [ADMIN] — Rules tab principal scoping (real scoped HTTP)
  GIVEN ADMIN server; rules seeded for both principals; Rules tab active
  WHEN  the user selects principal A, then switches to principal B
  THEN  only A's rule rows visible after A; A's rows absent after B; each selection fires a real HTTP call
        carrying the principalId (page.on('request'))
  VERIFY pnpm test:e2e:playwright -- governance-capstone -g "step2"
  TEST_TIER e2e   VERIFICATION_SERVICE real but-server (principalId-scoped query) + RulesList
  NEGATIVE CONTROL would fail if: stub ignores principalId (all rules always shown); 0 HTTP calls on switch
    (local-state only); A's rows remain after switching to B.

AC-3 [NONADMIN] — Read-only: real hasAdminWrite=false + controls disabled
  GIVEN NONADMIN server (BUT_AGENT_HANDLE=NONADMIN_HANDLE); admin user cookie set (nav visibility); page loaded
  WHEN  governance_status_read fires for the non-admin identity
  THEN  governance-read-only-message visible; governance_status_read body has hasAdminWrite=false;
        governance-commit-button absent/disabled; governance-branch-gates-control + governance-rules-control disabled
  VERIFY pnpm test:e2e:playwright -- governance-capstone -g "step3"
  TEST_TIER e2e   VERIFICATION_SERVICE real but-server (governed status read) + GovernanceSettings isReadOnly
  NEGATIVE CONTROL would fail if: isReadOnly hardcoded true client-side (status body would show hasAdminWrite=true);
    governance_status_read never called (no response to capture); any write control enabled under non-admin.

AC-4 [PRIMARY] — Self-escalation denial/no-flip (NONADMIN) + T-MGMT-032 full flow (ADMIN sub-case)
  AC-4a (NONADMIN denial):
    GIVEN NONADMIN server; Principals tab; the administration:write control shows aria-checked=false
    WHEN  the non-admin clicks the administration:write control to self-grant and saves
    THEN  perm_grant returns {type:error, code:perm.denied}; the denial banner is visible; the
          administration:write control aria-checked is UNCHANGED (false); pending count does NOT increment
  AC-4b (ADMIN pending-persist + commit clears — T-MGMT-032):
    GIVEN ADMIN server; a gate toggled to create pending
    WHEN  the user switches to the Groups tab and back, then clicks Commit
    THEN  the pending banner PERSISTS across the tab switch; after the commit 2xx the badge CLEARS
  VERIFY pnpm test:e2e:playwright -- governance-capstone -g "step4"
  TEST_TIER e2e   VERIFICATION_SERVICE real but-server (governed perm_grant denial; commit) + UI
  NEGATIVE CONTROL would fail if: toggle flips to aria-checked=true (optimistic — CAP-AUTHZ-01 violation);
    denial banner absent; pending increments on a denied write; admin identity used (no denial possible);
    pending banner lost on tab switch; badge does not clear after a 2xx commit.

AC-5 [ADMIN] — Keyboard navigation (ArrowRight + Home/End + aria-selected toggle)
  GIVEN ADMIN server; page loaded; Principals tab active (aria-selected=true)
  WHEN  the user Tabs to the tablist, presses ArrowRight x3 (→ Rules), then Home (→ Principals), then End (→ Rules)
  THEN  each activated tab has aria-selected=true; the previously-selected tab has aria-selected=false; the
        corresponding panel becomes visible; the active tab has tabindex=0; Home/End wrap correctly
  VERIFY pnpm test:e2e:playwright -- governance-capstone -g "step5"
  TEST_TIER e2e   VERIFICATION_SERVICE real Svelte Tabs (role=tablist/tab/aria-selected from MGMT-UI-011)
  NEGATIVE CONTROL would fail if: ArrowRight doesn't move focus (no handler — pre-MGMT-UI-011); previous tab
    keeps aria-selected=true; active tab has tabindex=-1 (inverted bug); Home/End do nothing.

AC-6 [ADMIN] — IPC failure: banner + retry; persistent failure stays read-only; clear + Retry succeeds
  GIVEN ADMIN server; page loaded; the IPC error banner is NOT visible (pre-injection asserted)
  WHEN  page.route() returns 500 on a governance endpoint; then Retry is clicked while the route is still
        active; then the route is cleared (page.unroute, in finally) and Retry is clicked again
  THEN  post-injection: error banner visible + retry button visible + controls disabled; after Retry with the
        route active: banner STILL visible + controls STILL disabled (T-MGMT-040); after clear + Retry: banner
        gone + controls return to writable
  VERIFY pnpm test:e2e:playwright -- governance-capstone -g "step6"
  TEST_TIER e2e   VERIFICATION_SERVICE real but-server + page.route() HTTP-layer 500 injection
  NEGATIVE CONTROL would fail if: Retry navigates/reloads instead of re-issuing the SDK call; banner visible
    before injection (hardcoded); controls re-enable after persistent failure; banner disappears after Retry
    with the route still active.

--------------------------------------------------------------------------------
GUARDRAILS
--------------------------------------------------------------------------------
WRITE-ALLOWED:
  - e2e/playwright/tests/governance-capstone.spec.ts (NEW — the sole deliverable)
  - e2e/playwright/src/governance.ts (NEW only if E2E-MGMT-BE-001 has not already created it; otherwise consume-only)
WRITE-PROHIBITED:
  - apps/desktop/** (Svelte components owned by MGMT-UI-009/010/011/004)
  - crates/** (Rust routes owned by E2E-MGMT-BE-002)
  - e2e/playwright/fixtures/governance-app/** (the React mock — NOT the test target)
  - e2e/playwright/src/setup.ts, test.ts (consume-only harness)
  - packages/** ; any +page.server.ts / +layout.server.ts

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
- e2e/playwright/src/test.ts:34-52 — gitbutler fixture is PER-TEST; gitbutlerOptions is a per-test option (test.use)
- e2e/playwright/src/setup.ts:1-365 — startGitButler spawns one real but-server per test; setCookie at :283
- e2e/playwright/tests/branches.spec.ts:1-60 — real e2e spec pattern to mirror
- apps/desktop/src/components/settings/ProjectSettingsModalContent.svelte:30-42 — isAdmin = user.role==='admin' (web visibility gate)
- apps/desktop/src/components/settings/SettingsModalLayout.svelte:53-66 — adminOnly filter; sidebar buttons have NO data-testid
- apps/desktop/src/components/governance/GovernanceSettings.svelte:60-162 — existing 06a testids + error display
- .../MGMT-UI-009-branch-gates-list.md — gate toggle controls (cross-task testid: governance-pending-badge)
- .../MGMT-UI-010-ruleslist-principalid.md — principal-scoped query (principalId in HTTP request)
- .../MGMT-UI-011-accessibility-ipc-retry.md — role=tablist/tab/aria-selected, Home/End, tabindex fix, danger InfoMessage + Retry, denial banner
- .../MGMT-UI-004-error-boundary-wrap.md — render-throw boundary CT (T-MGMT-037 lives here, NOT in this e2e)

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- pnpm test:e2e:playwright -- governance-capstone           → exit 0 (all 6 steps pass)
- pnpm check                                                → exit 0 (no TS errors in the spec)
- pnpm lint                                                 → exit 0
- grep -c 'governance-nav-item' e2e/playwright/tests/governance-capstone.spec.ts   → 0 (banned locator absent)
- grep -c 'page\.evaluate'      e2e/playwright/tests/governance-capstone.spec.ts   → 0 (banned state injection)
- grep -c 'governance-app'      e2e/playwright/tests/governance-capstone.spec.ts   → 0 (React mock not referenced)
- grep -c 'page\.unroute'       e2e/playwright/tests/governance-capstone.spec.ts   → >=1 (route hygiene present)

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
depends_on: MGMT-BE-003, MGMT-BE-004, MGMT-UI-002, MGMT-UI-004, MGMT-UI-009, MGMT-UI-010, MGMT-UI-011, E2E-MGMT-BE-001, E2E-MGMT-BE-002
blocks:     (none — capstone)

--------------------------------------------------------------------------------
NOTES
--------------------------------------------------------------------------------
N1 — TESTID CONTRACT (explicit cross-task requirements + fallback locators). These testids are not yet in
  live code; each is a hard dependency on the named task. Use the data-testid if the task ships it; else the
  fallback role/aria/text locator:
   - governance-pending-badge (MGMT-UI-009)        fallback: getByText(/\d+ pending/i)
   - governance-ipc-error-banner (MGMT-UI-011)     fallback: getByRole('alert') with style=danger
   - governance-ipc-retry-button (MGMT-UI-011)     fallback: getByRole('button', { name: 'Retry' })
   - governance-denial-banner (MGMT-UI-011)        fallback: getByRole('alert') text /perm.denied/i
   - governance-rules-principal-select (MGMT-UI-010), governance-rule-row (MGMT-UI-010)
   - governance-principal-editor / governance-save-principal-button (MGMT-UI-009 or 011)
  Already exist (06a): governance-settings, -read-only-message, -pending-banner, -commit-button,
  -{principals,groups,branch-gates,rules}-panel, -branch-gates-control, -rules-control. NEVER assert
  governance-nav-item (no task owns SettingsModalLayout testids) — navigate by accessible text.
N2 — STEP-4 IDENTITY (definitive ruling): step 4 acts as NONADMIN_HANDLE. An admin holds administration:write
  so a governed admin self-grant stages pending (no denial); the Tauri product resolves the human as
  fleet-owner superuser (R12, also no denial). Only the non-admin governed self-grant yields a real
  perm.denied — the only honest no-bypass proof (CAP-AUTHZ-01). AC-4b (admin pending) is a separate sub-case.
N3 — UPSTREAM ADVISORY U1 (record only; do NOT edit SPRINT.md): SPRINT.md step-4 wording ("As an admin … denial")
  is unprovable as written; recommend rewording to "a non-admin attempts to self-grant administration:write → denial".
N4 — UPSTREAM ADVISORY U3: the governance nav is conditional on isAdmin (ProjectSettingsModalContent.svelte:34);
  MGMT-UI-002 owns making this satisfiable on the web target. This spec uses setCookie('user',{role:'admin'}) as
  the workaround. If MGMT-UI-002 doesn't land, the page won't render and ALL ACs block — escalate.
N5 — PER-STEP PLAYABILITY: ALL 6 ACs are blocked until the full dep chain lands (E2E-MGMT-BE-001 + BE-002 +
  MGMT-UI-002/004/009/010/011 + MGMT-BE-003/004). A skeleton (test.skip blocks) may be written first; no AC may
  be reported green until the REAL but-server serves real governed data for that step. This task writes no
  production code — pure consumer.
N6 — T-MGMT-037 render-throw boundary is covered by the MGMT-UI-004 component test (cannot be honestly forced in
  e2e without banned page.evaluate state-forcing). This e2e covers the IPC-failure boundary path (AC-6). Honest
  division of responsibility, recorded in coverage_gaps — not a gap.
N7 — gates.toml minimal schema: E2E-MGMT-BE-001 seeds ONLY [[branch]] name="master" protected=true (the live
  loader rejects unknown fields). See U2 in BE-001 notes.
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "requirements": [
    { "id": "AC-1", "type": "acceptance_criterion", "description": "[ADMIN step1] Toggling a Branch Gate's protected control fires branch_gates_update returning HTTP 2xx non-error body; pending banner visible (count>0); commit button enabled", "verify": "pnpm test:e2e:playwright -- governance-capstone -g \"step1\"", "test_tier": "e2e" },
    { "id": "AC-2", "type": "acceptance_criterion", "description": "[ADMIN step2] Selecting principal A shows only A's rule rows; selecting B removes A's; each selection fires a real principalId-scoped HTTP call", "verify": "pnpm test:e2e:playwright -- governance-capstone -g \"step2\"", "test_tier": "e2e" },
    { "id": "AC-3", "type": "acceptance_criterion", "description": "[NONADMIN step3] governance_status_read body has hasAdminWrite=false; governance-read-only-message visible; all write controls disabled", "verify": "pnpm test:e2e:playwright -- governance-capstone -g \"step3\"", "test_tier": "e2e" },
    { "id": "AC-4", "type": "acceptance_criterion", "description": "[PRIMARY] NONADMIN self-grant of administration:write returns perm.denied, denial banner visible, control aria-checked UNCHANGED, pending no-increment; AND (ADMIN sub-case T-MGMT-032) pending persists across tab switch and clears after a 2xx commit", "verify": "pnpm test:e2e:playwright -- governance-capstone -g \"step4\"", "test_tier": "e2e", "primary": true },
    { "id": "AC-5", "type": "acceptance_criterion", "description": "[ADMIN step5] ArrowRight x3 + Home/End navigate tabs; activated tab aria-selected=true, previous false; panels switch; active tab tabindex=0", "verify": "pnpm test:e2e:playwright -- governance-capstone -g \"step5\"", "test_tier": "e2e" },
    { "id": "AC-6", "type": "acceptance_criterion", "description": "[ADMIN step6] IPC 500 via page.route → error banner + retry button + controls disabled; persistent failure (route still active, Retry) keeps banner+disabled; clear route + Retry → banner gone + controls writable", "verify": "pnpm test:e2e:playwright -- governance-capstone -g \"step6\"", "test_tier": "e2e" },
    { "id": "TC-1", "type": "test_criterion", "description": "branch_gates_update POST returns 2xx non-error body; governance-pending-banner appears count>0; commit-button enabled", "verify": "pnpm test:e2e:playwright -- governance-capstone -g \"step1\"", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "Principal A selected: only A's rules + scoped HTTP call with principalId; switch to B: A absent, B visible", "verify": "pnpm test:e2e:playwright -- governance-capstone -g \"step2\"", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "NONADMIN server: governance_status_read body hasAdminWrite=false; read-only message visible; write controls disabled", "verify": "pnpm test:e2e:playwright -- governance-capstone -g \"step3\"", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "NONADMIN self-grant: perm_grant perm.denied; denial banner visible; administration:write control aria-checked='false' (UNCHANGED); pending count does not increment", "verify": "pnpm test:e2e:playwright -- governance-capstone -g \"step4\"", "maps_to_ac": "AC-4" },
    { "id": "TC-5", "type": "test_criterion", "description": "ADMIN pending-persist: banner survives tab switch to Groups and back; Commit fires 2xx; banner clears (T-MGMT-032 full flow)", "verify": "pnpm test:e2e:playwright -- governance-capstone -g \"step4\"", "maps_to_ac": "AC-4" },
    { "id": "TC-6", "type": "test_criterion", "description": "ArrowRight x3 navigates Principals→Groups→BranchGates→Rules with aria-selected true on new tab/false on previous; Home wraps to Principals; End jumps to Rules; active tab tabindex=0", "verify": "pnpm test:e2e:playwright -- governance-capstone -g \"step5\"", "maps_to_ac": "AC-5" },
    { "id": "TC-7", "type": "test_criterion", "description": "Pre-injection: error banner absent; post-injection 500: banner visible + retry-button visible + controls disabled", "verify": "pnpm test:e2e:playwright -- governance-capstone -g \"step6\"", "maps_to_ac": "AC-6" },
    { "id": "TC-8", "type": "test_criterion", "description": "Persistent failure (Retry with route still active): banner STILL visible + controls STILL disabled (T-MGMT-040)", "verify": "pnpm test:e2e:playwright -- governance-capstone -g \"step6\"", "maps_to_ac": "AC-6" },
    { "id": "TC-9", "type": "test_criterion", "description": "Route cleared + Retry: banner disappears; controls return to enabled write state; page functional", "verify": "pnpm test:e2e:playwright -- governance-capstone -g \"step6\"", "maps_to_ac": "AC-6" }
  ]
}
-->
