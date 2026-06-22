# MGMT-UI-012: Build-gate tests: no direct config write, no SvelteKit server files, SDK type-check, human-principal

## What this does

Encode the four structural invariants of the governed SvelteKit front-end as executable build-gate commands that CI and the implementer can run to prove: (1) no governance component issues a direct .gitbutler/\*.toml write bypassing the SDK; (2) no +page.server.ts, +layout.server.ts, or +server.ts exists anywhere in apps/desktop/src (adapter-static compliance — widened by REMEDIATE-UI-2 to cover all SvelteKit server file kinds; apps/web/src is also checked for +page.server.ts and +layout.server.ts as defense-in-depth, while +server.ts is permitted there because apps/web uses adapter-vercel); (3) the regenerated but-sdk types type-check cleanly against all governance components; (4) the desktop config-management write path resolves the human fleet-owner (never falls through to resolve_principal_from_env, no agent superuser branch). These are boolean structural assertions — source-level grep/find/typecheck invariants — not behavioral scenarios. They run in CI on every commit that touches the governance surface.

## Why

Sprint 06b · PRD UC-MGMT-06 · capability CAP-AUTHZ-01. All four gates exit 0 on a clean sprint-06b tree: (AC-1) governance component grep finds zero direct .gitbutler write calls; (AC-2) gate script finds zero +page.server.ts, +layout.server.ts, or +server.ts under apps/desktop/src (widened by REMEDIATE-UI-2); (AC-3) pnpm -F @gitbutler/desktop check exits 0 agai

## How to verify

PRIMARY **AC-1** — `grep -rn 'gitbutler.*\.toml\|writeFile\|fs\.write\|writeTextFile\|writeBinaryFile\|plugin-fs' apps/desktop/src/components/governance/ apps/desktop/src/components/settings/GovernanceSettings.svelte apps/desktop/src/components/rules/RulesList.svelte | grep -v 'but-sdk\|import\|//\|warn\|error\|log' | wc -l | grep '^0$'`: No direct .gitbutler/\*.toml write in governance components (T-MGMT-027). Full gate set in the spec below.

## Scope

- apps/desktop/tests/governance/BuildGates.spec.ts (optional: CI-runnable test file that wraps the gate commands as test cases, if the project CI convention requires a test file rather than raw shell commands)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: MGMT-UI-012 — Build-gate tests: no direct config write, no SvelteKit server files, SDK type-check, human-principal
================================================================================

TASK_TYPE:   INFRA
STATUS:      Backlog
PRIORITY:    P0
EFFORT:      S  (45 min)
AGENT:       sveltekit-implementer
PROPOSED-BY: sveltekit-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-MGMT-06
CAPABILITIES:CAP-AUTHZ-01

RUNTIME_COMMANDS:
  check: pnpm -F @gitbutler/desktop check
  lint:  pnpm lint

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
All four gates exit 0 on a clean sprint-06b tree: (AC-1) governance component grep finds zero direct .gitbutler write calls; (AC-2) find finds zero +page.server.ts, +layout.server.ts, or +server.ts under apps/desktop/src and zero +page.server.ts or +layout.server.ts under apps/web/src (adapter-static compliance, widened by REMEDIATE-UI-2); (AC-3) pnpm -F @gitbutler/desktop check exits 0 against the regenerated SDK types; (AC-4) grep finds zero resolve_principal_from_env references in governance Tauri command wiring; (AC-5) find finds zero GovernanceErrorBoundary.svelte files. pnpm lint also exits 0.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] Every gate command must exit 0 on a clean tree — use `| wc -l | grep '^0$'` for absence gates so CI sees a non-zero exit on violation.
- [MUST] Grep gates must exclude import and comment lines (pipe through `grep -v 'import'` and `grep -v '//'`) to avoid false positives on SDK import paths that mention the prohibited pattern.
- [MUST] The SvelteKit server-file gate (AC-2) must cover the FULL apps/desktop/src tree (not only the governance subdirectory) per adapter-static constraint, and must forbid all three server file kinds: +page.server.ts, +layout.server.ts, and +server.ts. Under apps/web/src (adapter-vercel) only +page.server.ts and +layout.server.ts are forbidden; +server.ts is a legitimate adapter-vercel API route pattern.
- [MUST] The GovernanceErrorBoundary.svelte absence gate must assert the net-new file does NOT exist — MGMT-UI-004 uses the EXISTING shared/ErrorBoundary.svelte; any GovernanceErrorBoundary.svelte is a violation.
- [MUST] The SDK type-check gate must run `pnpm -F @gitbutler/desktop check` against the regenerated but-sdk types (produced by MGMT-BE-003/004 + MGMT-IPC-004) — not against a cached or prior-sprint SDK snapshot.
- [MUST] The human-principal gate (T-MGMT-042) asserts governance config-management write paths resolve the human principal via UserService/forge session — verified by grepping for `resolve_principal_from_env` in the Tauri governance command wiring (must be absent, replaced by the fleet-owner shim).
- [MUST] All gates must be runnable from the repo root without changing the working directory.
- [MUST] Gates are additive over the 06a grep pattern set — reuse and extend, do not reinvent.
- [MUST] Grep exclusion pipeline MUST include 'warn\|error\|log' to avoid false-negatives from error message strings containing '.toml' path references (e.g. console.warn('Failed to load .gitbutler/gates.toml')).
- [NEVER] NEVER add +page.server.ts, +layout.server.ts, or +server.ts under apps/desktop/src (adapter-static).
- [NEVER] NEVER write or modify GovernanceErrorBoundary.svelte — that component must not exist; the existing shared/ErrorBoundary.svelte is the approved boundary.
- [NEVER] NEVER weaken a gate by excluding governance-specific paths from the grep scope.
- [NEVER] NEVER use `wc -l | grep -v '^0$'` (inverted logic) — the gate must exit 0 when the prohibited pattern is ABSENT.
- [NEVER] NEVER stub or skip a gate by making the grep match nothing via an overly narrow path argument.
- [NEVER] NEVER reference resolve_principal_from_env in any governance desktop config-management command path.
- [STRICTLY] Gate commands use the pipeline form: `grep -rn <pattern> <paths> | grep -v import | grep -v '//' | wc -l | grep '^0$'`
- [STRICTLY] Type-check gate uses exactly: `pnpm -F @gitbutler/desktop check`
- [STRICTLY] Lint gate uses exactly: `pnpm lint`
- [STRICTLY] The SvelteKit server-file absence gate uses: `sh .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-012-gate.sh` (which scans apps/desktop/src for +page.server.ts, +layout.server.ts, +server.ts and apps/web/src for +page.server.ts, +layout.server.ts; widened by REMEDIATE-UI-2)
- [STRICTLY] proposed_by field MUST be 'sveltekit-planner' — this is a hard tripwire.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1: No direct .gitbutler/*.toml write in governance components (T-MGMT-027)
- [ ] AC-2: No +page.server.ts, +layout.server.ts, or +server.ts anywhere under apps/desktop/src; no +page.server.ts or +layout.server.ts under apps/web/src (T-MGMT-036 — adapter-static, widened by REMEDIATE-UI-2)
- [ ] AC-3: SDK regenerated and governance components type-check (T-MGMT-034)
- [ ] AC-4: Fleet-owner identity shim IS present AND resolve_principal_from_env is absent on governance Tauri command path (T-MGMT-042 — two-part gate)
- [ ] AC-5: No GovernanceErrorBoundary.svelte file exists (MGMT-UI-004 net-new boundary prohibition)
- [ ] AC-6: Lint passes on governance surface
- [ ] AC-7: No @tauri-apps/plugin-fs import in governance components (SEC-4)
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1: No direct .gitbutler/*.toml write in governance components (T-MGMT-027)
  TEST_TIER: build-gate   VERIFICATION_SERVICE: grep-structural
  UNIT_TEST_JUSTIFIED: T-MGMT-027 governed-front-end invariant — filesystem structural check; no runtime service needed.
  VERIFY: grep -rn 'gitbutler.*\.toml\|writeFile\|fs\.write\|writeTextFile\|writeBinaryFile\|plugin-fs' apps/desktop/src/components/governance/ apps/desktop/src/components/settings/GovernanceSettings.svelte apps/desktop/src/components/rules/RulesList.svelte | grep -v 'but-sdk\|import\|//\|warn\|error\|log' | wc -l | grep '^0$'

AC-2: No +page.server.ts, +layout.server.ts, or +server.ts under apps/desktop/src; no +page.server.ts or +layout.server.ts under apps/web/src (T-MGMT-036 — adapter-static, widened by REMEDIATE-UI-2)
  TEST_TIER: build-gate   VERIFICATION_SERVICE: find-structural
  UNIT_TEST_JUSTIFIED: adapter-static constraint — filesystem structural check; no runtime service needed.
  VERIFY: sh .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-012-gate.sh

AC-3: SDK regenerated and governance components type-check (T-MGMT-034)
  TEST_TIER: build-gate   VERIFICATION_SERVICE: tsc-desktop
  UNIT_TEST_JUSTIFIED: Type-system structural check — SDK contract compliance gate; no runtime service needed.
  VERIFY: pnpm -F @gitbutler/desktop check

AC-4: Fleet-owner identity shim IS present AND resolve_principal_from_env is absent on governance Tauri command path (T-MGMT-042 — two-part gate)
  TEST_TIER: build-gate   VERIFICATION_SERVICE: grep-structural
  UNIT_TEST_JUSTIFIED: T-MGMT-042 identity invariant — two-part structural check: positive presence of fleet-owner shim + negative absence of resolve_principal_from_env on governance path. Depends on MGMT-IPC-003.
  VERIFY: PART-1 (positive): grep -rn 'fleet_owner\|with_fleet_owner_identity\|UserService\|forge_session' crates/gitbutler-tauri/src/ | grep -i 'governance\|perm_\|group_\|branch_gates' | grep -v '//' | wc -l | grep -v '^0$' && PART-2 (negative): grep -rn 'resolve_principal_from_env' crates/gitbutler-tauri/src/ | grep -i 'governance\|perm_\|group_\|branch_gates' | grep -v '//' | wc -l | grep '^0$'

AC-5: No GovernanceErrorBoundary.svelte file exists (MGMT-UI-004 net-new boundary prohibition)
  TEST_TIER: build-gate   VERIFICATION_SERVICE: find-structural
  UNIT_TEST_JUSTIFIED: Sprint-06b boundary contract — filesystem structural check; no runtime service needed.
  VERIFY: find apps/desktop/src -name 'GovernanceErrorBoundary.svelte' | wc -l | grep '^0$'

AC-6: Lint passes on governance surface
  TEST_TIER: build-gate   VERIFICATION_SERVICE: pnpm-lint
  UNIT_TEST_JUSTIFIED: Code quality structural check — linter is deterministic over source; no runtime service needed.
  VERIFY: pnpm lint

AC-7: No @tauri-apps/plugin-fs import in governance components (SEC-4)
  TEST_TIER: build-gate   VERIFICATION_SERVICE: grep-structural
  UNIT_TEST_JUSTIFIED: SEC-4 Tauri plugin-fs direct import prohibition — structural source check; no runtime service needed.
  VERIFY: grep -rn '@tauri-apps/plugin-fs' apps/desktop/src/components/governance/ apps/desktop/src/components/settings/GovernanceSettings.svelte apps/desktop/src/components/rules/RulesList.svelte | wc -l | grep '^0$'

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): grep of governance component paths finds zero direct .gitbutler write calls including Tauri fs plugin (writeTextFile/writeBinaryFile/@tauri-apps/plugin-fs) (exits 0)
    VERIFY: grep -rn 'gitbutler.*\.toml\|writeFile\|fs\.write\|writeTextFile\|writeBinaryFile\|plugin-fs' apps/desktop/src/components/governance/ apps/desktop/src/components/settings/GovernanceSettings.svelte apps/desktop/src/components/rules/RulesList.svelte | grep -v 'but-sdk\|import\|//\|warn\|error\|log' | wc -l | grep '^0$'
- TC-2 (-> AC-2): gate script finds zero +page.server.ts, +layout.server.ts, or +server.ts under apps/desktop/src and zero +page.server.ts or +layout.server.ts under apps/web/src (exits 0)
    VERIFY: sh .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-012-gate.sh
- TC-3 (-> AC-3): pnpm -F @gitbutler/desktop check exits 0 against the regenerated but-sdk types
    VERIFY: pnpm -F @gitbutler/desktop check
- TC-4 (-> AC-4): Two-part gate: (1) fleet-owner shim IS present on governance Tauri command wiring (positive grep exits non-zero before wc); (2) resolve_principal_from_env is absent (negative grep exits 0)
    VERIFY: PART-1: grep -rn 'fleet_owner\|with_fleet_owner_identity\|UserService\|forge_session' crates/gitbutler-tauri/src/ | grep -i 'governance\|perm_\|group_\|branch_gates' | grep -v '//' | wc -l | grep -v '^0$' && PART-2: grep -rn 'resolve_principal_from_env' crates/gitbutler-tauri/src/ | grep -i 'governance\|perm_\|group_\|branch_gates' | grep -v '//' | wc -l | grep '^0$'
- TC-5 (-> AC-5): find under apps/desktop/src finds zero GovernanceErrorBoundary.svelte files (exits 0)
    VERIFY: find apps/desktop/src -name 'GovernanceErrorBoundary.svelte' | wc -l | grep '^0$'
- TC-6 (-> AC-6): pnpm lint exits 0 across the full workspace including the governance surface
    VERIFY: pnpm lint
- TC-7 (-> AC-7): grep for @tauri-apps/plugin-fs in governance component paths finds zero results (exits 0)
    VERIFY: grep -rn '@tauri-apps/plugin-fs' apps/desktop/src/components/governance/ apps/desktop/src/components/settings/GovernanceSettings.svelte apps/desktop/src/components/rules/RulesList.svelte | wc -l | grep '^0$'

--------------------------------------------------------------------------------
IMPLEMENTATION STEPS
--------------------------------------------------------------------------------
1. {'step': 1, 'title': 'Verify dependency outputs are present', 'detail': 'Confirm MGMT-UI-009 (BranchGatesList.svelte), MGMT-UI-010 (RulesList.svelte with principalId prop), MGMT-UI-011 (accessibility + IPC-failure banner), and MGMT-UI-004 (ErrorBoundary wrap) have all landed. Confirm MGMT-BE-003/004 + MGMT-IPC-004 have regenerated packages/but-sdk/src/generated. This task gates on all four UI tasks and both backend tasks. Do not author gate scripts until the components exist — the type-check gate (AC-3) would trivially pass against missing files.', 'files_read': ['apps/desktop/src/components/governance/BranchGatesList.svelte', 'apps/desktop/src/components/rules/RulesList.svelte', 'apps/desktop/src/components/settings/GovernanceSettings.svelte', 'packages/but-sdk/src/generated/index.ts']}
2. {'step': 2, 'title': 'Run AC-1: no direct .gitbutler write gate', 'detail': 'Execute the grep gate against the governance component paths. If the gate fails (count > 0), identify the offending line, which component owns it, and surface it as a blocker against MGMT-UI-009/010/011. Do NOT weaken the grep — fix the component.', 'command': "grep -rn 'gitbutler.*\\.toml\\|writeFile\\|fs\\.write' apps/desktop/src/components/governance/ apps/desktop/src/components/settings/GovernanceSettings.svelte apps/desktop/src/components/rules/RulesList.svelte | grep -v 'but-sdk\\|import\\|//' | wc -l | grep '^0$'"}
3. {'step': 3, 'title': 'Run AC-2: no SvelteKit server files gate', 'detail': 'Execute the gate script over apps/desktop/src (all three server file kinds) and apps/web/src (+page.server.ts, +layout.server.ts). If the gate fails, identify which file was created and which task authored it — every server file under the desktop tree is a sprint contract violation.', 'command': 'sh .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-012-gate.sh'}
4. {'step': 4, 'title': 'Run AC-5: no GovernanceErrorBoundary.svelte gate', 'detail': 'Execute the find gate. If the file exists, it means a task created a net-new boundary component instead of reusing shared/ErrorBoundary.svelte — surface as a blocker against MGMT-UI-004.', 'command': "find apps/desktop/src -name 'GovernanceErrorBoundary.svelte' | wc -l | grep '^0$'"}
5. {'step': 5, 'title': 'Run AC-4: human-principal (no resolve_principal_from_env on governance path) gate', 'detail': 'Execute the grep gate over gitbutler-tauri/src/. If the gate fails, the governance Tauri command handler is falling through to the env-handle path (resolve_principal_from_env) instead of using the fleet-owner shim — surface as a blocker against MGMT-IPC-003.', 'command': "grep -rn 'resolve_principal_from_env' crates/gitbutler-tauri/src/ | grep -i 'governance\\|perm_\\|group_\\|branch_gates' | grep -v '//' | wc -l | grep '^0$'"}
6. {'step': 6, 'title': 'Run AC-3: SDK type-check gate', 'detail': 'Execute `pnpm -F @gitbutler/desktop check`. If it fails, read the tsc error output, identify the component and the offending type reference, and surface as a blocker against the relevant component task (MGMT-UI-009, MGMT-UI-010, or MGMT-UI-011) or against MGMT-IPC-004 if the SDK types are missing/malformed.', 'command': 'pnpm -F @gitbutler/desktop check'}
7. {'step': 7, 'title': 'Run AC-6: lint gate', 'detail': 'Execute `pnpm lint`. Fix any prettier/eslint/oxlint/knip errors introduced by the sprint-06b governance components before declaring the gate passed.', 'command': 'pnpm lint'}
8. {'step': 8, 'title': 'Write gate evidence to verification_checklist', 'detail': 'Record the exit code of each gate command. All six must be 0. If any gate is non-zero, the task is NOT complete — surface the blocker with the exact failing command and its output.'}

--------------------------------------------------------------------------------
VERIFICATION CHECKLIST (boolean build-gates)
--------------------------------------------------------------------------------
- VG-1 [AC-1 / T-MGMT-027 / SEC-4]: No direct .gitbutler write in governance components
    CMD: grep -rn 'gitbutler.*\.toml\|writeFile\|fs\.write\|writeTextFile\|writeBinaryFile\|plugin-fs' apps/desktop/src/components/governance/ apps/desktop/src/components/settings/GovernanceSettings.svelte apps/desktop/src/components/rules/RulesList.svelte | grep -v 'but-sdk\|import\|//\|warn\|error\|log' | wc -l | grep '^0$'
    PASS: Command exits 0; grep finds no prohibited write patterns (including writeTextFile, writeBinaryFile, @tauri-apps/plugin-fs) in governance component source excluding import/comment/log lines.
- VG-2 [AC-2 / T-MGMT-036 / REMEDIATE-UI-2]: No +page.server.ts, +layout.server.ts, or +server.ts under apps/desktop/src; no +page.server.ts or +layout.server.ts under apps/web/src
    CMD: sh .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-012-gate.sh
    PASS: Command exits 0; gate script returns no forbidden SvelteKit server files anywhere under the desktop source tree (all three kinds) or the web source tree (+page.server.ts, +layout.server.ts).
- VG-3 [AC-3 / T-MGMT-034]: Governance components type-check against regenerated SDK
    CMD: pnpm -F @gitbutler/desktop check
    PASS: Command exits 0; tsc reports no type errors in any governance component or store file.
- VG-4 [AC-4 / T-MGMT-042]: Fleet-owner shim present AND resolve_principal_from_env absent on governance Tauri path (two-part)
    CMD: grep -rn 'fleet_owner\|with_fleet_owner_identity\|UserService\|forge_session' crates/gitbutler-tauri/src/ | grep -i 'governance\|perm_\|group_\|branch_gates' | grep -v '//' | wc -l | grep -v '^0$' && grep -rn 'resolve_principal_from_env' crates/gitbutler-tauri/src/ | grep -i 'governance\|perm_\|group_\|branch_gates' | grep -v '//' | wc -l | grep '^0$'
    PASS: Both parts exit 0: (1) fleet-owner shim IS found on governance command path (positive grep finds >= 1 match); (2) resolve_principal_from_env is NOT found on governance command path (count = 0).
- VG-5 [AC-5 / MGMT-UI-004 boundary contract]: No GovernanceErrorBoundary.svelte file exists
    CMD: find apps/desktop/src -name 'GovernanceErrorBoundary.svelte' | wc -l | grep '^0$'
    PASS: Command exits 0; no net-new boundary component was created — shared/ErrorBoundary.svelte is the only approved boundary.
- VG-6 [AC-6]: pnpm lint passes across workspace
    CMD: pnpm lint
    PASS: Command exits 0; prettier, eslint, oxlint, and knip all report no errors.
- VG-7 [AC-7 / SEC-4]: No @tauri-apps/plugin-fs import in governance components (SEC-4)
    CMD: grep -rn '@tauri-apps/plugin-fs' apps/desktop/src/components/governance/ apps/desktop/src/components/settings/GovernanceSettings.svelte apps/desktop/src/components/rules/RulesList.svelte | wc -l | grep '^0$'
    PASS: Command exits 0; no governance component imports @tauri-apps/plugin-fs directly.

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - apps/desktop/tests/governance/BuildGates.spec.ts (optional: CI-runnable test file that wraps the gate commands as test cases, if the project CI convention requires a test file rather than raw shell commands)
writeProhibited:
  - apps/desktop/src/components/governance/GovernanceErrorBoundary.svelte (MUST NOT EXIST)
  - any +page.server.ts, +layout.server.ts, or +server.ts under apps/desktop/src/ (adapter-static — widened by REMEDIATE-UI-2)
  - packages/but-sdk/src/generated/ (generated by pnpm build:sdk — do not hand-edit)
  - apps/desktop/src/components/governance/BranchGatesList.svelte (authored by MGMT-UI-009)
  - apps/desktop/src/components/rules/RulesList.svelte (MGMT-UI-010 owns the principalId extension)
  - apps/desktop/src/components/settings/GovernanceSettings.svelte (MGMT-UI-004 owns the ErrorBoundary wrap)
  - crates/gitbutler-tauri/src/ (Tauri wiring owned by MGMT-IPC-003 / tauri-implementer)

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. .spec/prds/governance/tasks/sprint-06a-governance-ui-scaffold-principals-groups/MGMT-UI-003-governance-settings-pending-store.md
2. .spec/prds/governance/tasks/sprint-06a-governance-ui-scaffold-principals-groups/MGMT-IPC-003-register-governance-commands.md
3. .spec/prds/governance/11-e2e-testing-criteria.md
4. .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/SPRINT.md
5. apps/desktop/src/components/governance/
6. apps/desktop/src/components/settings/GovernanceSettings.svelte
7. apps/desktop/src/components/shared/ErrorBoundary.svelte
8. packages/but-sdk/src/generated/
9. crates/gitbutler-tauri/src/

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- grep -rn 'gitbutler.*\.toml\|writeFile\|fs\.write\|writeTextFile\|writeBinaryFile\|plugin-fs' apps/desktop/src/components/governance/ apps/desktop/src/components/settings/GovernanceSettings.svelte apps/desktop/src/components/rules/RulesList.svelte | grep -v 'but-sdk\|import\|//\|warn\|error\|log' | wc -l | grep '^0$'   -> exit 0
- sh .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-012-gate.sh   -> exit 0
- pnpm -F @gitbutler/desktop check   -> exit 0
- grep -rn 'fleet_owner\|with_fleet_owner_identity\|UserService\|forge_session' crates/gitbutler-tauri/src/ | grep -i 'governance\|perm_\|group_\|branch_gates' | grep -v '//' | wc -l | grep -v '^0$' && grep -rn 'resolve_principal_from_env' crates/gitbutler-tauri/src/ | grep -i 'governance\|perm_\|group_\|branch_gates' | grep -v '//' | wc -l | grep '^0$'   -> exit 0 (both parts pass)
- find apps/desktop/src -name 'GovernanceErrorBoundary.svelte' | wc -l | grep '^0$'   -> exit 0
- pnpm lint   -> exit 0
- grep -rn '@tauri-apps/plugin-fs' apps/desktop/src/components/governance/ apps/desktop/src/components/settings/GovernanceSettings.svelte apps/desktop/src/components/rules/RulesList.svelte | wc -l | grep '^0$'   -> exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references:
  - .spec/prds/governance/10-technical-requirements/04-api-design.md — Tauri command surface and identity/confinement model
  - .spec/prds/governance/08-uc-mgmt.md — UC-MGMT-06 governed front-end contract
  - .spec/prds/governance/11-e2e-testing-criteria.md lines 218-228 — T-MGMT-027/034/036/042 pass/fail semantics
notes:
  - T
  - h
  - i
  - s
  -
  - t
  - a
  - s
  - k
  -
  - h
  - a
  - s
  -
  - n
  - o
  -
  - U
  - I
  -
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
  - .
  -
  - I
  - t
  -
  - i
  - s
  -
  - a
  -
  - p
  - u
  - r
  - e
  -
  - I
  - N
  - F
  - R
  - A
  -
  - t
  - a
  - s
  - k
  - :
  -
  - g
  - a
  - t
  - e
  -
  - c
  - o
  - m
  - m
  - a
  - n
  - d
  - s
  -
  - t
  - h
  - a
  - t
  -
  - v
  - e
  - r
  - i
  - f
  - y
  -
  - s
  - t
  - r
  - u
  - c
  - t
  - u
  - r
  - a
  - l
  -
  - i
  - n
  - v
  - a
  - r
  - i
  - a
  - n
  - t
  - s
  -
  - o
  - f
  -
  - t
  - h
  - e
  -
  - s
  - p
  - r
  - i
  - n
  - t
  - -
  - 0
  - 6
  - b
  -
  - d
  - e
  - l
  - i
  - v
  - e
  - r
  - a
  - b
  - l
  - e
  - s
  - .
  -
  - T
  - h
  - e
  -
  - g
  - a
  - t
  - e
  - s
  -
  - r
  - u
  - n
  -
  - a
  - f
  - t
  - e
  - r
  -
  - a
  - l
  - l
  -
  - o
  - t
  - h
  - e
  - r
  -
  - s
  - p
  - r
  - i
  - n
  - t
  - -
  - 0
  - 6
  - b
  -
  - t
  - a
  - s
  - k
  - s
  -
  - c
  - o
  - m
  - p
  - l
  - e
  - t
  - e
  -
  - (
  - t
  - h
  - e
  -
  - d
  - e
  - p
  - e
  - n
  - d
  - e
  - n
  - c
  - y
  -
  - c
  - h
  - a
  - i
  - n
  -
  - e
  - n
  - d
  - s
  -
  - h
  - e
  - r
  - e
  - )
  - .
  -
  - I
  - f
  -
  - a
  - n
  - y
  -
  - g
  - a
  - t
  - e
  -
  - f
  - a
  - i
  - l
  - s
  - ,
  -
  - t
  - h
  - e
  -
  - f
  - a
  - i
  - l
  - u
  - r
  - e
  -
  - i
  - s
  -
  - a
  -
  - r
  - e
  - g
  - r
  - e
  - s
  - s
  - i
  - o
  - n
  -
  - i
  - n
  -
  - t
  - h
  - e
  -
  - d
  - e
  - p
  - e
  - n
  - d
  - e
  - n
  - t
  -
  - t
  - a
  - s
  - k
  -
  - (
  - M
  - G
  - M
  - T
  - -
  - U
  - I
  - -
  - 0
  - 0
  - 9
  - /
  - 0
  - 1
  - 0
  - /
  - 0
  - 1
  - 1
  -
  - f
  - o
  - r
  -
  - A
  - C
  - -
  - 1
  - ;
  -
  - M
  - G
  - M
  - T
  - -
  - I
  - P
  - C
  - -
  - 0
  - 0
  - 3
  -
  - f
  - o
  - r
  -
  - A
  - C
  - -
  - 4
  - )
  - ,
  -
  - n
  - o
  - t
  -
  - i
  - n
  -
  - t
  - h
  - i
  - s
  -
  - t
  - a
  - s
  - k
  -
  - —
  -
  - s
  - u
  - r
  - f
  - a
  - c
  - e
  -
  - t
  - h
  - e
  -
  - b
  - l
  - o
  - c
  - k
  - e
  - r
  -
  - a
  - g
  - a
  - i
  - n
  - s
  - t
  -
  - t
  - h
  - e
  -
  - o
  - w
  - n
  - i
  - n
  - g
  -
  - t
  - a
  - s
  - k
  - .
pattern: grep-pipeline build-gate — `grep ... | grep -v import | grep -v '//' | wc -l | grep '^0$'` exits 0 iff prohibited pattern is absent; used identically in the AC-5/AC-6/AC-7 gates of MGMT-UI-003 (06a). Extend that pattern set here.
pattern_source: .spec/prds/governance/tasks/sprint-06a-governance-ui-scaffold-principals-groups/MGMT-UI-003-governance-settings-pending-store.md (AC-5/AC-6/AC-7 grep patterns)
anti_pattern: Do NOT use `grep -c` (returns count, not 0/1 exit) or `! grep` (inverts exit, fragile in CI); always use `| wc -l | grep '^0$'`. Do NOT scope the +page.server.ts gate to a governance subdirectory only — the adapter-static invariant is workspace-wide. Do NOT omit 'warn|error|log' from grep exclusions — these can mask false-negatives where a log/error message string contains the prohibited path.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agent: sveltekit-implementer
rationale: This task authors grep/structural/typecheck build-gate scripts that target the SvelteKit desktop app surface (apps/desktop/src). The sveltekit-implementer owns the adapter-static compliance surface, knows the governance component file layout, and is the same agent that implemented the components under test — making it the correct agent to author and wire the invariant-enforcement gates.
coding_standards: Gate commands must be idempotent and runnable from the repo root without cd., Grep patterns must exclude import and comment lines to avoid false positives on SDK import paths., Use `| wc -l | grep '^0$'` (not `grep -c`, not `! grep`) for all absence gates — this form gives a predictable CI exit code., Do not suppress or weaken a failing gate by narrowing its path argument — fix the source component., The lint gate (`pnpm lint`) must run last — it is the most expensive and catches issues the structural gates cannot (e.g. unused imports added during the sprint)., Per CLAUDE.md: no stubs, no fake success — if a gate cannot pass because a dependency task has not landed, mark this task as BLOCKED with the exact failing gate and the dependency task ID.

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: MGMT-UI-009; MGMT-UI-010; MGMT-UI-011; MGMT-UI-004; MGMT-BE-003; MGMT-BE-004; MGMT-IPC-003; MGMT-IPC-004
Blocks:     none
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "MGMT-UI-012",
  "proposed_by": "sveltekit-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": false,
    "requires_seeded_evidence": false
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN  WHEN  THEN",
      "verify": "grep -rn 'gitbutler.*\\.toml\\|writeFile\\|fs\\.write\\|writeTextFile\\|writeBinaryFile\\|plugin-fs' apps/desktop/src/components/governance/ apps/desktop/src/components/settings/GovernanceSettings.svelte apps/desktop/src/components/rules/RulesList.svelte | grep -v 'but-sdk\\|import\\|//\\|warn\\|error\\|log' | wc -l | grep '^0$'"
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN apps/desktop/src uses adapter-static and apps/web/src uses adapter-vercel WHEN the gate script scans for forbidden SvelteKit server files THEN no +page.server.ts, +layout.server.ts, or +server.ts is found under apps/desktop/src and no +page.server.ts or +layout.server.ts is found under apps/web/src (widened by REMEDIATE-UI-2)",
      "verify": "sh .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-012-gate.sh"
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN  WHEN  THEN",
      "verify": "pnpm -F @gitbutler/desktop check"
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN  WHEN  THEN",
      "verify": "PART-1 (positive): grep -rn 'fleet_owner\\|with_fleet_owner_identity\\|UserService\\|forge_session' crates/gitbutler-tauri/src/ | grep -i 'governance\\|perm_\\|group_\\|branch_gates' | grep -v '//' | wc -l | grep -v '^0$' && PART-2 (negative): grep -rn 'resolve_principal_from_env' crates/gitbutler-tauri/src/ | grep -i 'governance\\|perm_\\|group_\\|branch_gates' | grep -v '//' | wc -l | grep '^0$'"
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN  WHEN  THEN",
      "verify": "find apps/desktop/src -name 'GovernanceErrorBoundary.svelte' | wc -l | grep '^0$'"
    },
    {
      "id": "AC-6",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN  WHEN  THEN",
      "verify": "pnpm lint"
    },
    {
      "id": "AC-7",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN  WHEN  THEN",
      "verify": "grep -rn '@tauri-apps/plugin-fs' apps/desktop/src/components/governance/ apps/desktop/src/components/settings/GovernanceSettings.svelte apps/desktop/src/components/rules/RulesList.svelte | wc -l | grep '^0$'"
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "grep of governance component paths finds zero direct .gitbutler write calls including Tauri fs plugin (writeTextFile/writeBinaryFile/@tauri-apps/plugin-fs) (exits 0)",
      "verify": "grep -rn 'gitbutler.*\\.toml\\|writeFile\\|fs\\.write\\|writeTextFile\\|writeBinaryFile\\|plugin-fs' apps/desktop/src/components/governance/ apps/desktop/src/components/settings/GovernanceSettings.svelte apps/desktop/src/components/rules/RulesList.svelte | grep -v 'but-sdk\\|import\\|//\\|warn\\|error\\|log' | wc -l | grep '^0$'",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "gate script finds zero +page.server.ts, +layout.server.ts, or +server.ts under apps/desktop/src and zero +page.server.ts or +layout.server.ts under apps/web/src (exits 0)",
      "verify": "sh .spec/prds/governance/tasks/sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-UI-012-gate.sh",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "pnpm -F @gitbutler/desktop check exits 0 against the regenerated but-sdk types",
      "verify": "pnpm -F @gitbutler/desktop check",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "Two-part gate: (1) fleet-owner shim IS present on governance Tauri command wiring (positive grep exits non-zero before wc); (2) resolve_principal_from_env is absent (negative grep exits 0)",
      "verify": "PART-1: grep -rn 'fleet_owner\\|with_fleet_owner_identity\\|UserService\\|forge_session' crates/gitbutler-tauri/src/ | grep -i 'governance\\|perm_\\|group_\\|branch_gates' | grep -v '//' | wc -l | grep -v '^0$' && PART-2: grep -rn 'resolve_principal_from_env' crates/gitbutler-tauri/src/ | grep -i 'governance\\|perm_\\|group_\\|branch_gates' | grep -v '//' | wc -l | grep '^0$'",
      "maps_to_ac": "AC-4"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "find under apps/desktop/src finds zero GovernanceErrorBoundary.svelte files (exits 0)",
      "verify": "find apps/desktop/src -name 'GovernanceErrorBoundary.svelte' | wc -l | grep '^0$'",
      "maps_to_ac": "AC-5"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "description": "pnpm lint exits 0 across the full workspace including the governance surface",
      "verify": "pnpm lint",
      "maps_to_ac": "AC-6"
    },
    {
      "id": "TC-7",
      "type": "test_criterion",
      "description": "grep for @tauri-apps/plugin-fs in governance component paths finds zero results (exits 0)",
      "verify": "grep -rn '@tauri-apps/plugin-fs' apps/desktop/src/components/governance/ apps/desktop/src/components/settings/GovernanceSettings.svelte apps/desktop/src/components/rules/RulesList.svelte | wc -l | grep '^0$'",
      "maps_to_ac": "AC-7"
    }
  ]
}
-->
