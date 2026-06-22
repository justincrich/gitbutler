# REMEDIATE-UI-2: Widen MGMT-UI-012 build-gate grep to forbid all SvelteKit server files in governance components

**Type:** REMEDIATION | **Status:** Backlog | **Priority:** P0 | **Effort:** S (60 min)
**Agent:** sveltekit-implementer | **Reviewer:** sveltekit-reviewer | **Proposed by:** sveltekit-planner
**Closes red-hat findings:** M3
**Depends on:** MGMT-UI-012 | **Blocks:** (none)
**PRD refs:** UC-MGMT-04, .spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md | **Capabilities:** CAP-CONFIG-01, CAP-DEPLOY-01

## What this does

MGMT-UI-012 currently asserts only that no +page.server.ts exists in the governance feature directories. Adapter-static forbids ALL server-side SvelteKit files. Update AC-2 and its verification gate to grep for +page.server.ts, +layout.server.ts, and +server.ts under apps/desktop/src and apps/web/src. Add a RED-phase test that deliberately creates a +layout.server.ts file and verifies the gate catches it before allowing the gate to pass again. Update the task contract file and any CI script that consumes the gate.

## Why

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: REMEDIATE-UI-2 — Widen MGMT-UI-012 build-gate grep to forbid all SvelteKit server files in governance components
================================================================================

TASK_TYPE:   REMEDIATION
STATUS:      Backlog
PRIORITY:    P0
EFFORT:      S  (60 min)
AGENT:       implementer=sveltekit-implementer | reviewer=sveltekit-reviewer
PROPOSED-BY: sveltekit-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-MGMT-04, .spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md
CAPABILITIES:CAP-CONFIG-01,CAP-DEPLOY-01
CLOSES:      M3

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
MGMT-UI-012 currently asserts only that no +page.server.ts exists in the governance feature directories. Adapter-static forbids ALL server-side SvelteKit files. Update AC-2 and its verification gate to grep for +page.server.ts, +layout.server.ts, and +server.ts under apps/desktop/src and apps/web/src. Add a RED-phase test that deliberately creates a +layout.server.ts file and verifies the gate catches it before allowing the gate to pass again. Update the task contract file and any CI script that consumes the gate.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] Adapter-static output must never contain server-side data fetches.
- [MUST] Any +page.server.ts, +layout.server.ts, or +server.ts under apps/desktop/src or apps/web/src is a violation.
- [MUST] The grep gate must fail CI, not just emit a warning.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: the AC text and its verification command reject all three file kinds across the governance routes
- [ ] AC-2: the gate exits non-zero and names the offending file
- [ ] AC-3: the gate exits zero and prints no forbidden matches

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: the AC text and its verification command reject all three file kinds across the governance routes
  GIVEN: MGMT-UI-012 AC-2 currently checks only +page.server.ts
  WHEN: the contract is amended to name +page.server.ts, +layout.server.ts, and +server.ts as forbidden patterns
  THEN: the AC text and its verification command reject all three file kinds across the governance routes
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  SCENARIO:
    tier: visible   test_tier: integration
    verification_service: sh .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-012-gate.sh (real file system + forbidden fixture per B14)
    negative_control.would_fail_if:
      - the grep checks only +page.server.ts
      - the grep pattern is too narrow to match +layout.server.ts or +server.ts
      - the test fixture is missing
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=governance_component_tmp_dir
      action.actor=ci
        - plant a +layout.server.ts fixture under apps/desktop/src/routes/governance/tmp/
        - run the updated build-gate grep across apps/desktop/src and apps/web/src
        - inspect the grep exit code and output
      end_state.must_observe:
        - the build gate exits non-zero
        - gate output names the planted +layout.server.ts file
        - the verification command references +page.server.ts, +layout.server.ts, and +server.ts
      end_state.must_not_observe:
        - build gate exit code 0
        - gate output that only references +page.server.ts
        - no mention of the planted +layout.server.ts fixture

AC-2 : the gate exits non-zero and names the offending file
  GIVEN: a temporary +layout.server.ts file is added under apps/desktop/src/routes/settings/governance/
  WHEN: the build-gate command runs
  THEN: the gate exits non-zero and names the offending file
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-build-gate
  SCENARIO:
    tier: structural   test_tier: integration
    verification_service: sh .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-012-gate.sh (real file system + forbidden fixture per B14)
    negative_control.would_fail_if:
      - the gate exits 0 even when +layout.server.ts exists
      - the gate exits non-zero but names a different file
      - the gate only checks +page.server.ts and ignores +layout.server.ts
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=clean_tree
      action.actor=maintainer
        - mkdir -p apps/desktop/src/routes/governance/tmp
        - echo 'export const load = () => ({});' > apps/desktop/src/routes/governance/tmp/+layout.server.ts
        - run the updated build gate
      end_state.must_observe:
        - build gate exit code is non-zero
        - stderr/stdout contains 'governance/tmp/+layout.server.ts'
      end_state.must_not_observe:
        - build gate exit code 0
        - output that mentions only '+page.server.ts'
        - no output after forbidden file detected

AC-3 : the gate exits zero and prints no forbidden matches
  GIVEN: the temporary +layout.server.ts has been removed and only +page.svelte and client hooks remain
  WHEN: the build-gate command runs
  THEN: the gate exits zero and prints no forbidden matches
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-build-gate
  SCENARIO:
    tier: structural   test_tier: integration
    verification_service: find apps/desktop/src apps/web/src -type f \( -name '+page.server.ts' -o -name '+layout.server.ts' -o -name '+server.ts' \) (real tree scan + zero-match assertion per B14)
    negative_control.would_fail_if:
      - a legitimate +server.ts was introduced that the gate ignores
      - the gate was broadened to match all .server.ts strings and flags ordinary files
      - the grep still silently passes because it only checks one of the two apps
    evidence: artifact_type=stdout required_capture=True
    case[0] start_ref=clean_tree_after_removal
      action.actor=maintainer
        - remove the temporary +layout.server.ts fixture
        - run the updated build gate across apps/desktop/src and apps/web/src
      end_state.must_observe:
        - build gate exit code 0
        - grep prints zero matching paths
      end_state.must_not_observe:
        - forbidden file paths in the output
        - non-zero exit from the gate

--------------------------------------------------------------------------------
TEST CRITERIA
--------------------------------------------------------------------------------
- TC-1: MGMT-UI-012 contract text references +page.server.ts, +layout.server.ts, and +server.ts
- TC-2: A deliberately placed +layout.server.ts under governance routes trips the build gate
- TC-3: No server files exist under apps/desktop/src/governance or apps/web/src/routes/**/*governance*

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
- grep -E '\+page\.server\.ts|\+layout\.server\.ts|\+server\.ts' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-012*.md  →  ?
- pnpm -F @gitbutler/desktop check  →  ?
- pnpm lint  →  ?

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
depends_on: MGMT-UI-012
blocks:     (none)

--------------------------------------------------------------------------------
NOTES
--------------------------------------------------------------------------------
This remediation only changes the contract and the gate; it does not introduce new runtime behavior. The red-hat recommendation is categorized as MEDIUM because it is perimeter hardening, but the task is P0 because the capstone E2E depends on the adapter-static guarantee remaining true once code is merged.

```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "REMEDIATE-UI-2",
  "proposed_by": "sveltekit-planner",
  "supersedes": [],
  "closes_redhat_findings": [
    "M3"
  ],
  "fixtures": {
    "contract_before_amendment": {
      "description": "MGMT-UI-012 AC-2 contract text that checks only +page.server.ts under governance routes",
      "seed_method": "component_mount",
      "records": [
        "MGMT-UI-012*.md AC-2 verification with only +page.server.ts"
      ]
    },
    "governance_component_tmp_dir": {
      "description": "A temporary governance route directory that does not contain any server files yet",
      "seed_method": "component_mount",
      "records": [
        "apps/desktop/src/routes/governance/tmp/ exists and is empty"
      ]
    },
    "clean_tree": {
      "description": "Source tree with no forbidden server files before the negative-control fixture is planted",
      "seed_method": "component_mount",
      "records": [
        "0 forbidden server files under apps/desktop/src and apps/web/src"
      ]
    },
    "clean_tree_after_removal": {
      "description": "Source tree after the temporary +layout.server.ts fixture has been removed",
      "seed_method": "component_mount",
      "records": [
        "temporary +layout.server.ts removed",
        "0 forbidden server files remain"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN MGMT-UI-012 AC-2 currently checks only +page.server.ts WHEN the contract is amended to name +page.server.ts, +layout.server.ts, and +server.ts as forbidden patterns THEN the AC text and its verification command reject all three file kinds across the governance routes",
      "verify": "",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "sh .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-012-gate.sh (real file system + forbidden fixture per B14)",
        "negative_control": {
          "would_fail_if": [
            "the grep checks only +page.server.ts",
            "the grep pattern is too narrow to match +layout.server.ts or +server.ts",
            "the test fixture is missing"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "governance_component_tmp_dir",
            "action": {
              "actor": "ci",
              "steps": [
                "plant a +layout.server.ts fixture under apps/desktop/src/routes/governance/tmp/",
                "run the updated build-gate grep across apps/desktop/src and apps/web/src",
                "inspect the grep exit code and output"
              ]
            },
            "end_state": {
              "must_observe": [
                "the build gate exits non-zero",
                "gate output names the planted +layout.server.ts file",
                "the verification command references +page.server.ts, +layout.server.ts, and +server.ts"
              ],
              "must_not_observe": [
                "build gate exit code 0",
                "gate output that only references +page.server.ts",
                "no mention of the planted +layout.server.ts fixture"
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
      "description": "GIVEN a temporary +layout.server.ts file is added under apps/desktop/src/routes/settings/governance/ WHEN the build-gate command runs THEN the gate exits non-zero and names the offending file",
      "verify": "",
      "scenario": {
        "tier": "structural",
        "test_tier": "integration",
        "verification_service": "sh .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-012-gate.sh (real file system + forbidden fixture per B14)",
        "negative_control": {
          "would_fail_if": [
            "the gate exits 0 even when +layout.server.ts exists",
            "the gate exits non-zero but names a different file",
            "the gate only checks +page.server.ts and ignores +layout.server.ts"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "clean_tree",
            "action": {
              "actor": "maintainer",
              "steps": [
                "mkdir -p apps/desktop/src/routes/governance/tmp",
                "echo 'export const load = () => ({});' > apps/desktop/src/routes/governance/tmp/+layout.server.ts",
                "run the updated build gate"
              ]
            },
            "end_state": {
              "must_observe": [
                "build gate exit code is non-zero",
                "stderr/stdout contains 'governance/tmp/+layout.server.ts'"
              ],
              "must_not_observe": [
                "build gate exit code 0",
                "output that mentions only '+page.server.ts'",
                "no output after forbidden file detected"
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
      "description": "GIVEN the temporary +layout.server.ts has been removed and only +page.svelte and client hooks remain WHEN the build-gate command runs THEN the gate exits zero and prints no forbidden matches",
      "verify": "",
      "scenario": {
        "tier": "structural",
        "test_tier": "integration",
        "verification_service": "find apps/desktop/src apps/web/src -type f \\( -name '+page.server.ts' -o -name '+layout.server.ts' -o -name '+server.ts' \\) (real tree scan + zero-match assertion per B14)",
        "negative_control": {
          "would_fail_if": [
            "a legitimate +server.ts was introduced that the gate ignores",
            "the gate was broadened to match all .server.ts strings and flags ordinary files",
            "the grep still silently passes because it only checks one of the two apps"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "clean_tree_after_removal",
            "action": {
              "actor": "maintainer",
              "steps": [
                "remove the temporary +layout.server.ts fixture",
                "run the updated build gate across apps/desktop/src and apps/web/src"
              ]
            },
            "end_state": {
              "must_observe": [
                "build gate exit code 0",
                "grep prints zero matching paths"
              ],
              "must_not_observe": [
                "forbidden file paths in the output",
                "non-zero exit from the gate"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "MGMT-UI-012 contract text references +page.server.ts, +layout.server.ts, and +server.ts",
      "verify": "grep -E '\\+page\\.server\\.ts|\\+layout\\.server\\.ts|\\+server\\.ts' .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-012*.md",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "A deliberately placed +layout.server.ts under governance routes trips the build gate",
      "verify": "tmp=$(mktemp); mkdir -p apps/desktop/src/routes/governance/tmp; printf 'export const load = () => ({});\n' > apps/desktop/src/routes/governance/tmp/+layout.server.ts; NODECMD='pnpm exec svelte-package --check' || sh .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-012-gate.sh'; EXIT=$?; rm -f apps/desktop/src/routes/governance/tmp/+layout.server.ts; exit $EXIT",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "No server files exist under apps/desktop/src/governance or apps/web/src/routes/**/*governance*",
      "verify": "test $(find apps/desktop/src apps/web/src -type f \\( -name '+page.server.ts' -o -name '+layout.server.ts' -o -name '+server.ts' \\) 2>/dev/null | wc -l | tr -d ' ') -eq 0",
      "maps_to_ac": "AC-3"
    }
  ]
}
-->
