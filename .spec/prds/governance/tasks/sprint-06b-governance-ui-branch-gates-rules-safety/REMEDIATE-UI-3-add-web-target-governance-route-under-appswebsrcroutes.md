# REMEDIATE-UI-3: Add web-target governance route under apps/web/src/routes

**Type:** FEATURE | **Status:** Backlog | **Priority:** P0 | **Effort:** M (180 min)
**Agent:** sveltekit-implementer | **Reviewer:** sveltekit-reviewer | **Proposed by:** sveltekit-planner
**Closes red-hat findings:** M2
**Depends on:** MGMT-UI-002, MGMT-UI-003, DESIGN-MGMT-007 | **Blocks:** E2E-MGMT-UI-001
**PRD refs:** UC-MGMT-01, UC-MGMT-03, .spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md | **Capabilities:** CAP-AUTHZ-01, CAP-NAV-01, CAP-CONFIG-01

## What this does

The capstone E2E (E2E-MGMT-UI-001) claims to drive the real web-target Svelte governance UI, but apps/web/src/routes currently contains no governance routes. Add a client-side route at /governance under apps/web/src/routes/(app)/governance/+page.svelte. The route must mount a governance settings view, enforce isAdmin via context, and redirect or show a read-only/denied state for non-admins. If importing GovernanceSettings.svelte from apps/desktop is disallowed by package boundaries, create a web-local GovernanceSettings page component that follows the same four-tab contract, wrapping the shared @gitbutler/ui primitives. No server files are allowed.

## Why

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: REMEDIATE-UI-3 — Add web-target governance route under apps/web/src/routes
================================================================================

TASK_TYPE:   FEATURE
STATUS:      Backlog
PRIORITY:    P0
EFFORT:      M  (180 min)
AGENT:       implementer=sveltekit-implementer | reviewer=sveltekit-reviewer
PROPOSED-BY: sveltekit-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-MGMT-01, UC-MGMT-03, .spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md
CAPABILITIES:CAP-AUTHZ-01,CAP-NAV-01,CAP-CONFIG-01
CLOSES:      M2

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
The capstone E2E (E2E-MGMT-UI-001) claims to drive the real web-target Svelte governance UI, but apps/web/src/routes currently contains no governance routes. Add a client-side route at /governance under apps/web/src/routes/(app)/governance/+page.svelte. The route must mount a governance settings view, enforce isAdmin via context, and redirect or show a read-only/denied state for non-admins. If importing GovernanceSettings.svelte from apps/desktop is disallowed by package boundaries, create a web-local GovernanceSettings page component that follows the same four-tab contract, wrapping the shared @gitbutler/ui primitives. No server files are allowed.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] The route must be client-side only: adapter-static; no +page.server.ts, +layout.server.ts, or +server.ts.
- [MUST] isAdmin gating must mirror MGMT-UI-002 (web) by reading from user-service context and redirecting non-admins.
- [MUST] The governance page must reuse the existing GovernanceSettings.svelte desktop component from packages/ui or apps/desktop as appropriate, or create a web-local wrapper if import boundaries prohibit cross-app reuse.
- [MUST] E2E-MGMT-UI-001 must be able to navigate to /governance and assert a rendered governance UI.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: the page renders without a 404 and shows governance chrome (tabs or settings heading)
- [ ] AC-2: the page redirects to the app home, shows an access-denied placeholder, or renders the view in read-only mode according to the web isAdmin contract
- [ ] AC-3: zero +page.server.ts, +layout.server.ts, or +server.ts files are found
- [ ] AC-4: the page loads and the Branch Gates tab is reachable

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: the page renders without a 404 and shows governance chrome (tabs or settings heading)
  GIVEN: the web dev server is running and no governance route existed before
  WHEN: a user navigates to /governance
  THEN: the page renders without a 404 and shows governance chrome (tabs or settings heading)
  TEST_TIER: e2e   VERIFICATION_SERVICE: playwright-web
  SCENARIO:
    tier: visible   test_tier: e2e
    verification_service: pnpm dev:web & pnpm test:e2e:playwright -- governance-route-web (real Svelte 5 runtime + Playwright DOM assertions per B14)
    negative_control.would_fail_if:
      - the route is missing and the server returns 404
      - the route renders an empty page
      - the route mounts a placeholder instead of GovernanceSettings
    evidence: artifact_type=screenshot required_capture=True
    case[0] start_ref=web_target_no_governance
      action.actor=ci
        - start pnpm dev:web
        - navigate to /governance
      end_state.must_observe:
        - HTTP status is not 404
        - document contains element with testid='governance-settings'
        - exactly 4 tab triggers are visible inside the governance tablist
      end_state.must_not_observe:
        - 404 page body
        - 0 governance-settings elements
        - fewer than 4 tab triggers

AC-2 : the page redirects to the app home, shows an access-denied placeholder, or renders the view in read-only mode according to the web isAdmin contract
  GIVEN: a non-admin user context is injected into the web app
  WHEN: the user navigates to /governance
  THEN: the page redirects to the app home, shows an access-denied placeholder, or renders the view in read-only mode according to the web isAdmin contract
  TEST_TIER: e2e   VERIFICATION_SERVICE: playwright-web
  SCENARIO:
    tier: visible   test_tier: e2e
    verification_service: pnpm test:e2e:playwright -- governance-route-web-nonadmin (real browser + non-admin context seed per B14)
    negative_control.would_fail_if:
      - non-admin sees writable governance controls
      - non-admin sees a 500 instead of a controlled denial
      - the isAdmin check is skipped because it relies on a missing web service
    evidence: artifact_type=screenshot required_capture=True
    case[0] start_ref=web_nonadmin_context
      action.actor=nonadmin
        - inject non-admin user context
        - navigate to /governance
      end_state.must_observe:
        - URL is not /governance OR the page shows read-only/denied chrome
        - no writable governance controls are present
      end_state.must_not_observe:
        - enabled Toggles or text inputs when isAdmin=false
        - unconditional admin view rendered for non-admin

AC-3 : zero +page.server.ts, +layout.server.ts, or +server.ts files are found
  GIVEN: the route directory exists under apps/web/src/routes/(app)/governance/
  WHEN: find runs for forbidden patterns
  THEN: zero +page.server.ts, +layout.server.ts, or +server.ts files are found
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-build-gate
  SCENARIO:
    tier: structural   test_tier: integration
    verification_service: find apps/web/src/routes -type f \( -name '+page.server.ts' -o -name '+layout.server.ts' -o -name '+server.ts' \) (real tree scan + zero-match assertion per B14)
    negative_control.would_fail_if:
      - a +page.server.ts is introduced for SSR admin checks
      - a +server.ts is introduced for an admin-only API route
      - the find command misses server files in nested subdirectories
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=clean_web_routes
      action.actor=maintainer
        - create apps/web/src/routes/(app)/governance/+page.svelte
        - run server-file grep across apps/web/src/routes
      end_state.must_observe:
        - grep prints 0 matching paths
      end_state.must_not_observe:
        - +page.server.ts in the output
        - +server.ts in the output

AC-4 : the page loads and the Branch Gates tab is reachable
  GIVEN: E2E-MGMT-UI-001 is configured for the web target
  WHEN: the spec calls page.goto('/governance') and seeds an admin principal
  THEN: the page loads and the Branch Gates tab is reachable
  TEST_TIER: e2e   VERIFICATION_SERVICE: playwright-web
  SCENARIO:
    tier: visible   test_tier: e2e
    verification_service: pnpm test:e2e:playwright -- E2E-MGMT-UI-001-web-navigation (real browser + admin seed per B14)
    negative_control.would_fail_if:
      - the capstone times out waiting for /governance navigation
      - the Branch Gates tab is not keyboard/click accessible in the web build
      - the web route is guarded behind a different URL path than the capstone expects
    evidence: artifact_type=screenshot required_capture=True
    case[0] start_ref=capstone_web_target
      action.actor=ci
        - seed admin principal
        - page.goto('/governance')
        - click or tab to the Branch Gates tab
      end_state.must_observe:
        - page URL is /governance
        - Branch Gates tab has aria-selected='true' or visible active state
      end_state.must_not_observe:
        - 404 response
        - Branch Gates tab missing from the tablist

--------------------------------------------------------------------------------
TEST CRITERIA
--------------------------------------------------------------------------------
- TC-1: Navigating to /governance renders governance chrome in the Playwright web harness
- TC-2: Non-admin context is gated on the web governance route
- TC-3: No server files exist under apps/web/src/routes/(app)/governance
- TC-4: Capstone E2E can reach the governance page on web target

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------

--------------------------------------------------------------------------------
GUARDRAILS
--------------------------------------------------------------------------------
WRITE_ALLOWED:

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- pnpm dev:web & sleep 12 && pnpm test:e2e:playwright -- governance-route-web; pkill -f 'dev:web' || true  →  ?
- find apps/web/src/routes -type f \( -name '+page.server.ts' -o -name '+layout.server.ts' -o -name '+server.ts' \) | wc -l | grep '^0$'  →  ?
- pnpm -F @gitbutler/web check  →  ?
- pnpm lint  →  ?

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
depends_on: MGMT-UI-002, MGMT-UI-003, DESIGN-MGMT-007
blocks:     E2E-MGMT-UI-001

--------------------------------------------------------------------------------
NOTES
--------------------------------------------------------------------------------
This remediation closes the M2 finding. It unblocks the web-target capstone. The web app uses a different adapter than the desktop app; ensure prerender still succeeds with the new route. If GovernanceSettings.svelte cannot be imported from apps/desktop, the implementer must create a web-local wrapper and note the divergence in the PR.

```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "REMEDIATE-UI-3",
  "proposed_by": "sveltekit-planner",
  "supersedes": [],
  "closes_redhat_findings": [
    "M2"
  ],
  "fixtures": {
    "web_target_no_governance": {
      "description": "Web dev server running with adapter-static and no governance route under apps/web/src/routes/(app)/governance",
      "seed_method": "public_api",
      "records": [
        "apps/web dev server is running",
        "apps/web/src/routes/(app)/governance/ does not exist"
      ]
    },
    "web_nonadmin_context": {
      "description": "A signed-in web user context with isAdmin=false",
      "seed_method": "public_api",
      "records": [
        "authenticated session",
        "isAdmin flag set to false"
      ]
    },
    "clean_web_routes": {
      "description": "apps/web/src/routes tree containing only client-side route files before adding governance",
      "seed_method": "component_mount",
      "records": [
        "0 +page.server.ts",
        "0 +layout.server.ts",
        "0 +server.ts"
      ]
    },
    "capstone_web_target": {
      "description": "Web-target E2E harness with an admin principal seeded and the capstone spec loaded",
      "seed_method": "public_api",
      "records": [
        "admin principal seeded",
        "page.goto('/governance') available"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN the web dev server is running and no governance route existed before WHEN a user navigates to /governance THEN the page renders without a 404 and shows governance chrome (tabs or settings heading)",
      "verify": "",
      "scenario": {
        "tier": "visible",
        "test_tier": "e2e",
        "verification_service": "pnpm dev:web & pnpm test:e2e:playwright -- governance-route-web (real Svelte 5 runtime + Playwright DOM assertions per B14)",
        "negative_control": {
          "would_fail_if": [
            "the route is missing and the server returns 404",
            "the route renders an empty page",
            "the route mounts a placeholder instead of GovernanceSettings"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "web_target_no_governance",
            "action": {
              "actor": "ci",
              "steps": [
                "start pnpm dev:web",
                "navigate to /governance"
              ]
            },
            "end_state": {
              "must_observe": [
                "HTTP status is not 404",
                "document contains element with testid='governance-settings'",
                "exactly 4 tab triggers are visible inside the governance tablist"
              ],
              "must_not_observe": [
                "404 page body",
                "0 governance-settings elements",
                "fewer than 4 tab triggers"
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
      "description": "GIVEN a non-admin user context is injected into the web app WHEN the user navigates to /governance THEN the page redirects to the app home, shows an access-denied placeholder, or renders the view in read-only mode according to the web isAdmin contract",
      "verify": "",
      "scenario": {
        "tier": "visible",
        "test_tier": "e2e",
        "verification_service": "pnpm test:e2e:playwright -- governance-route-web-nonadmin (real browser + non-admin context seed per B14)",
        "negative_control": {
          "would_fail_if": [
            "non-admin sees writable governance controls",
            "non-admin sees a 500 instead of a controlled denial",
            "the isAdmin check is skipped because it relies on a missing web service"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "web_nonadmin_context",
            "action": {
              "actor": "nonadmin",
              "steps": [
                "inject non-admin user context",
                "navigate to /governance"
              ]
            },
            "end_state": {
              "must_observe": [
                "URL is not /governance OR the page shows read-only/denied chrome",
                "no writable governance controls are present"
              ],
              "must_not_observe": [
                "enabled Toggles or text inputs when isAdmin=false",
                "unconditional admin view rendered for non-admin"
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
      "description": "GIVEN the route directory exists under apps/web/src/routes/(app)/governance/ WHEN find runs for forbidden patterns THEN zero +page.server.ts, +layout.server.ts, or +server.ts files are found",
      "verify": "",
      "scenario": {
        "tier": "structural",
        "test_tier": "integration",
        "verification_service": "find apps/web/src/routes -type f \\( -name '+page.server.ts' -o -name '+layout.server.ts' -o -name '+server.ts' \\) (real tree scan + zero-match assertion per B14)",
        "negative_control": {
          "would_fail_if": [
            "a +page.server.ts is introduced for SSR admin checks",
            "a +server.ts is introduced for an admin-only API route",
            "the find command misses server files in nested subdirectories"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "clean_web_routes",
            "action": {
              "actor": "maintainer",
              "steps": [
                "create apps/web/src/routes/(app)/governance/+page.svelte",
                "run server-file grep across apps/web/src/routes"
              ]
            },
            "end_state": {
              "must_observe": [
                "grep prints 0 matching paths"
              ],
              "must_not_observe": [
                "+page.server.ts in the output",
                "+server.ts in the output"
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
      "description": "GIVEN E2E-MGMT-UI-001 is configured for the web target WHEN the spec calls page.goto('/governance') and seeds an admin principal THEN the page loads and the Branch Gates tab is reachable",
      "verify": "",
      "scenario": {
        "tier": "visible",
        "test_tier": "e2e",
        "verification_service": "pnpm test:e2e:playwright -- E2E-MGMT-UI-001-web-navigation (real browser + admin seed per B14)",
        "negative_control": {
          "would_fail_if": [
            "the capstone times out waiting for /governance navigation",
            "the Branch Gates tab is not keyboard/click accessible in the web build",
            "the web route is guarded behind a different URL path than the capstone expects"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "capstone_web_target",
            "action": {
              "actor": "ci",
              "steps": [
                "seed admin principal",
                "page.goto('/governance')",
                "click or tab to the Branch Gates tab"
              ]
            },
            "end_state": {
              "must_observe": [
                "page URL is /governance",
                "Branch Gates tab has aria-selected='true' or visible active state"
              ],
              "must_not_observe": [
                "404 response",
                "Branch Gates tab missing from the tablist"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "Navigating to /governance renders governance chrome in the Playwright web harness",
      "verify": "pnpm test:e2e:playwright -- governance-route-web",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "Non-admin context is gated on the web governance route",
      "verify": "pnpm test:e2e:playwright -- governance-route-web-nonadmin",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "No server files exist under apps/web/src/routes/(app)/governance",
      "verify": "find apps/web/src/routes \\( -name '+page.server.ts' -o -name '+layout.server.ts' -o -name '+server.ts' \\) | wc -l | grep '^0$'",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "Capstone E2E can reach the governance page on web target",
      "verify": "pnpm test:e2e:playwright -- E2E-MGMT-UI-001-web-navigation",
      "maps_to_ac": "AC-4"
    }
  ]
}
-->
