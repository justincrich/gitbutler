---
stability: TEST_SPEC
last_validated: 2026-06-18
prd_version: 1.3.0
---

# E2E / Integration Testing Criteria — Functional-Permission Agent Governance for GitButler

Per-UC test criteria. **Verification bar: real `but-authz` + real `but-api` + real git, no mocks** (see [10-technical-requirements/09-e2e-testing.md](./10-technical-requirements/09-e2e-testing.md)). Every AC is covered by ≥1 criterion. Types: `[integration-test]` (real crate/git), `[component-test]` (real `packages/ui`/desktop components), `[api-contract]` (the structured rejection shape), `[build-gate]` (grep/structural invariant asserted in CI), `[e2e-automated]` (full-flow).

**Coverage:** 17 UCs · 129 ACs · 129 criteria · 129/129 ACs covered. Breakdown: 67 integration-test · 38 component-test · 7 api-contract · 15 build-gate · 2 e2e-automated.

> **Component-test prerequisite (B14 / T-MGMT-000).** All 38 `[component-test]` criteria specify `pnpm test:ct`, which today runs only `packages/ui` — `apps/desktop` has **no CT config**. T-MGMT-000 (a `[build-gate]`) is a **hard prerequisite** for the entire 38-criterion component-test surface; none are runnable until the desktop CT config lands.

---

## AUTHZ: Functional Permission System

### UC-AUTHZ-01: Functional permission model + permissions.toml + role desugar

| #           | Criterion                                                            | AC Ref       | Type             | Setup              | Pass/Fail                                                                 |
| ----------- | -------------------------------------------------------------------- | ------------ | ---------------- | ------------------ | ------------------------------------------------------------------------- |
| T-AUTHZ-001 | A permission token parses to the typed `Authority`                   | AC-1         | integration-test | `but-authz`        | `Authority::parse("contents:write")==ContentsWrite`; unknown token errors |
| T-AUTHZ-002 | A principal's `AuthoritySet` loads from committed `permissions.toml` | AC-2         | integration-test | real repo + config | role-entry and list-entry both load to a set                              |
| T-AUTHZ-003 | `write` desugars excluding `merge`/`administration:write`            | AC-3         | integration-test | `but-authz`        | set contains contents/reviews/pr:write; NOT merge/admin:write             |
| T-AUTHZ-004 | `admin` desugars to superuser                                        | AC-4         | integration-test | `but-authz`        | set contains every `Authority` incl. merge + admin:write                  |
| T-AUTHZ-005 | A raw functional list loads without a role                           | AC-5         | integration-test | `but-authz`        | `permissions=[…]` loads; no role required                                 |
| T-AUTHZ-006 | `role="write"` and the equivalent list resolve identically           | AC-6         | integration-test | `but-authz`        | the two `AuthoritySet`s are equal                                         |
| T-AUTHZ-007 | A seeded initial `permissions.toml` gives day-one permissions        | AC-7         | integration-test | fresh repo         | seeded principals authorize per their bundles                             |
| T-AUTHZ-008 | Desugar + list parse match the catalog (one assertion test)          | AC-8         | integration-test | `but-authz`        | write-excludes-merge, admin-superuser, list-without-role all hold         |
| T-AUTHZ-024 | `maintain` desugars incl. merge, excl. admin:write                   | AC: maintain | integration-test | `but-authz`        | set has merge + admin:read, NOT admin:write                               |

### UC-AUTHZ-02: Enforce on actions with agent-readable denial

| #            | Criterion                                                                              | AC Ref                | Type             | Setup                   | Pass/Fail                                                                                                                                                          |
| ------------ | -------------------------------------------------------------------------------------- | --------------------- | ---------------- | ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| T-AUTHZ-009  | read-only principal denied a review                                                    | AC-1                  | integration-test | real `but-api`          | review denied (requires reviews:write)                                                                                                                             |
| T-AUTHZ-010  | read-only principal denied a merge                                                     | AC-2                  | integration-test | real `but-api`+git      | merge denied (requires merge)                                                                                                                                      |
| T-AUTHZ-011  | commit gate denies a commit lacking contents:write                                     | AC-3                  | integration-test | real commit path        | commit denied, same functional check                                                                                                                               |
| T-AUTHZ-012  | denial is the structured contract + exit 1                                             | AC-4                  | api-contract     | real `but-api`          | `{error:{code:"perm.denied",…}}`, exit 1, no crash                                                                                                                 |
| T-AUTHZ-013  | denial names missing permission + remediation_hint                                     | AC-5                  | api-contract     | real `but-api`          | message names `reviews:write`; hint non-empty                                                                                                                      |
| T-AUTHZ-014  | split actions independently enforced                                                   | AC-6                  | integration-test | reviews:write principal | review allowed; commit denied                                                                                                                                      |
| T-AUTHZ-015  | authority acquired at the `but-api` wrapper (one boundary)                             | AC-7                  | build-gate       | source                  | each consequential route wrapped via `_with_authz`                                                                                                                 |
| T-AUTHZ-016b | every `but-napi` entry point routes through `but-api` (no ungoverned lower-level call) | AC: all-callers-N-API | build-gate       | source                  | grep-audit asserts each consequential N-API route is `_with_authz`-wrapped / pre-call-guarded — no direct `but-workspace`/`but-core` call bypassing the seam (R14) |
| T-AUTHZ-016  | no role name in any enforcement path                                                   | AC: no-role-name      | build-gate       | source                  | grep finds no role string in enforcement                                                                                                                           |
| T-AUTHZ-017  | end-to-end denial test (registered read-only principal) — the full contract            | AC: e2e-denial        | api-contract     | real `but-api`+git      | exit 1, `error.code=="perm.denied"`, `reviews:write` mention, non-empty `remediation_hint`                                                                         |

### UC-AUTHZ-03: Identity confinement & config-change authority

| #           | Criterion                                                            | AC Ref | Type             | Setup                  | Pass/Fail                                                                 |
| ----------- | -------------------------------------------------------------------- | ------ | ---------------- | ---------------------- | ------------------------------------------------------------------------- |
| T-AUTHZ-018 | dispatched session bound to own handle                               | AC-1   | integration-test | `BUT_AGENT_HANDLE` set | every `but` action acts as that handle                                    |
| T-AUTHZ-019 | acting as another handle denied while dispatched                     | AC-2   | integration-test | dispatched agent       | `--as <other>` style call denied                                          |
| T-AUTHZ-020 | authority read from committed config, not agent claim                | AC-3   | integration-test | real config            | agent-supplied authority ignored                                          |
| T-AUTHZ-021 | config change requires administration:write                          | AC-4   | integration-test | non-admin principal    | `but perm`/config-touch denied with `perm.denied`                         |
| T-AUTHZ-022 | config-change check keys off Authority, not role                     | AC-5   | build-gate       | source                 | grep: admin checks test `AdministrationWrite`                             |
| T-AUTHZ-023 | confinement+authority integration test                               | AC-8   | integration-test | reviews:write agent    | denied (a) other-handle merge and (b) self-grant merge edit               |
| T-AUTHZ-025 | `but perm list --principal <other>` scoping                          | AC-6   | integration-test | non-admin caller       | denied unless self or `administration:read` (no topology recon)           |
| T-AUTHZ-026 | identity-confinement honesty                                         | AC-7   | integration-test | dispatched agent       | `--as` denied; env re-export NOT prevented (documented accepted residual) |
| T-AUTHZ-033 | self-grant of `administration:write` inert until committed (ref-pin) | AC-9   | integration-test | real `but-api`+git     | a self-grant on a feature head does not authorize the same change         |

### UC-AUTHZ-04: Fail-closed by default

| #           | Criterion                                | AC Ref | Type             | Setup                                             | Pass/Fail                                                             |
| ----------- | ---------------------------------------- | ------ | ---------------- | ------------------------------------------------- | --------------------------------------------------------------------- |
| T-AUTHZ-027 | unknown principal denied                 | AC-1   | integration-test | principal absent from config                      | action denied, never default-allow                                    |
| T-AUTHZ-028 | no `BUT_AGENT_HANDLE` rejected           | AC-2   | integration-test | handle unset                                      | action rejected (no anonymous action)                                 |
| T-AUTHZ-029 | malformed/unreadable config fails closed | AC-3   | integration-test | broken `gates.toml` @target ref                   | gate denies with `config.invalid`, not skip                           |
| T-AUTHZ-030 | undefined required group not vacuous     | AC-4   | integration-test | `require_approval_from_group` names missing group | merge denied, not vacuously satisfied                                 |
| T-AUTHZ-031 | fail-closed integration test             | AC-5   | integration-test | real `but-authz`+git                              | unknown-principal denied; malformed config denies; no-handle rejected |

> AUTHZ note: 33 ACs / 33 criteria. The +1 vs v1.2.0 is the N-API all-callers AC (UC-AUTHZ-02) ↔ T-AUTHZ-016b (build-gate). T-AUTHZ-017 is typed `api-contract` (its assertion is the structured-denial shape). (T-AUTHZ-032 from v1.2.0 is re-keyed as T-AUTHZ-033 to keep the self-grant-inert criterion adjacent to UC-AUTHZ-03 AC-9; IDs remain stable in meaning.)

---

## GRPS: Principal Grouping

### UC-GRPS-01: Grant to groups; effective set = union

| #          | Criterion                                                 | AC Ref      | Type             | Setup                              | Pass/Fail                                                                                             |
| ---------- | --------------------------------------------------------- | ----------- | ---------------- | ---------------------------------- | ----------------------------------------------------------------------------------------------------- |
| T-GRPS-001 | define a group with a functional set                      | AC-1        | integration-test | `but group create`+`grant`         | `[[group]]` persisted to config                                                                       |
| T-GRPS-002 | add a principal to a group                                | AC-2        | integration-test | `but group add-member`             | principal inherits the group's permissions                                                            |
| T-GRPS-003 | effective set = own ∪ groups                              | AC-3        | integration-test | `but-authz`                        | resolver returns the union                                                                            |
| T-GRPS-004 | authorized via a group grant                              | AC-4        | integration-test | reviewer via group                 | review allowed though no direct grant                                                                 |
| T-GRPS-005 | denied when no source grants — the `perm.denied` contract | AC-5        | api-contract     | `but-authz`                        | merge denied with `{code:"perm.denied"}`                                                              |
| T-GRPS-006 | `but group` ops require administration:write              | AC-6        | integration-test | non-admin                          | group ops denied                                                                                      |
| T-GRPS-007 | group-union integration test                              | AC-7        | integration-test | `code-reviewers` group             | review via group ✓; merge denied                                                                      |
| T-GRPS-013 | group permission ceiling + delegated-admin is named       | AC: ceiling | integration-test | group holds `administration:write` | a member can change config (delegated admin); setting it requires `administration:write` — not silent |

### UC-GRPS-02: Ref-pinned governed membership (no self-escalation)

| #          | Criterion                                              | AC Ref              | Type             | Setup                         | Pass/Fail                                                                      |
| ---------- | ------------------------------------------------------ | ------------------- | ---------------- | ----------------------------- | ------------------------------------------------------------------------------ |
| T-GRPS-008 | group config read at the target ref                    | AC-1                | integration-test | real git                      | authorize reads target-ref membership                                          |
| T-GRPS-009 | self-add to a group is ineffective for the same change | AC-2                | integration-test | feature head edits membership | head membership ignored                                                        |
| T-GRPS-010 | membership change inert until committed to target      | AC-3                | integration-test | working-tree edit             | grant has no effect until landed; CLI warns                                    |
| T-GRPS-011 | protected-branch group change is admin-gated           | AC-4                | integration-test | non-admin                     | denied                                                                         |
| T-GRPS-012 | self-escalation prevention integration test            | AC-5                | integration-test | real git                      | head adds author to `maintainers` → merge still denied                         |
| T-GRPS-014 | group membership read ONLY from target-ref blob        | AC: target-ref-only | integration-test | real git                      | authorize reads target-ref membership; working-tree/feature-head edits ignored |

---

## GATES: The Two Gates (Control Plane)

### UC-GATES-01: Commit gate — functional permission + branch protection

| #           | Criterion                                                       | AC Ref | Type             | Setup                                 | Pass/Fail                                                                                                      |
| ----------- | --------------------------------------------------------------- | ------ | ---------------- | ------------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| T-GATES-001 | feature-branch commit by contents:write accepted                | AC-1   | integration-test | real commit path                      | commit lands on feature branch                                                                                 |
| T-GATES-002 | direct commit to protected branch rejected                      | AC-2   | integration-test | `main` protected                      | rejected, agent-readable contract                                                                              |
| T-GATES-003 | rejection names branch + legitimate path                        | AC-3   | api-contract     | protected `main`                      | message names `main` + "land through a reviewed merge"                                                         |
| T-GATES-004 | commit lacking contents:write denied                            | AC-4   | integration-test | read-only principal                   | `perm.denied`; edits never reach a ref                                                                         |
| T-GATES-005 | principal+authority resolved from env+ref config                | AC-6   | integration-test | `BUT_AGENT_HANDLE`                    | resolved from handle + ref-pinned config                                                                       |
| T-GATES-006 | branch protection from gates.toml, not hardcoded (structural)   | AC-7   | build-gate       | source                                | protection is config-derived (target-ref `gates.toml`), with no hardcoded protected-branch list in enforcement |
| T-GATES-007 | commit-gate integration test (accept feature, reject protected) | AC-8   | integration-test | real commit path+git                  | feature-branch commit accepted; direct `main` commit rejected with `branch.protected`                          |
| T-GATES-016 | commit gate applies identically across branching mechanisms     | AC-5   | integration-test | virtual-branch + normal-git commit    | same `contents:write` + branch-protection decision regardless of mechanism                                     |
| T-GATES-017 | commit gate covers the opt-in worktree path                     | AC-9   | integration-test | `but worktree new` → commit/integrate | same decision as the virtual-branch path; no ungated worktree path                                             |

### UC-GATES-02: Merge gate — merge authority + review requirement @head

| #           | Criterion                                                            | AC Ref | Type             | Setup                                       | Pass/Fail                                                                                                                                                                     |
| ----------- | -------------------------------------------------------------------- | ------ | ---------------- | ------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| T-GATES-008 | merge denied without `merge` even if reviewed (governed review path) | AC-1   | integration-test | reviewed PR, no merge perm                  | `perm.denied` (review via governed `but review`)                                                                                                                              |
| T-GATES-009 | merge blocked below min_approvals                                    | AC-2   | integration-test | 0 approvals                                 | blocked, names unmet requirement                                                                                                                                              |
| T-GATES-010 | self-approval not counted                                            | AC-3   | integration-test | author approves own                         | requirement unmet                                                                                                                                                             |
| T-GATES-011 | stale approval dismissed after head advances                         | AC-4   | integration-test | approve@H1→H2                               | blocked until re-approval@H2                                                                                                                                                  |
| T-GATES-012 | approval required from each required group                           | AC-5   | integration-test | two required groups                         | needs an approval from each                                                                                                                                                   |
| T-GATES-013 | requirement read at target ref                                       | AC-6   | integration-test | head drops requirement                      | judged by target-ref requirement                                                                                                                                              |
| T-GATES-018 | merge gate fails closed on malformed/undefined-group config          | AC-7   | integration-test | broken/undefined-group `gates.toml` @target | denies (`config.invalid`), not treated as satisfied                                                                                                                           |
| T-GATES-014 | satisfied requirement allows merge                                   | AC-8   | integration-test | distinct approval(s)@head + merge holder    | merge proceeds                                                                                                                                                                |
| T-GATES-015 | merge-gate integration test (block→satisfy, governed review path)    | AC-9   | integration-test | real merge path+git                         | no-review blocked with `gate.review_required`; self blocked; distinct-per-group proceeds — all reviews via governed `but review` (forgeable direct-DB write out of scope, R6) |
| T-GATES-019 | gate reads requirement ONLY from target-ref blob                     | AC-10  | integration-test | real git                                    | requirement read from target-ref config; working-tree/feature-head edits ignored                                                                                              |

> GATES note: 19 ACs / 19 criteria (unchanged from v1.2.0). A1.10 (R6 High) and B12 are prose-only caveats on the UC descriptions — no AC/criterion added. T-GATES-006 is typed `build-gate` (its assertion is the no-hardcoded-protected-list structural invariant). The merge-gate criteria exercise the **governed review-submission path**; a direct DB write to `local_review_verdicts` is untestably forgeable (R6) and is explicitly NOT a path under test.

---

## LOOP: Governed Orchestration Loop

### UC-LOOP-01: Role separation emerges from the permission set

| #          | Criterion                                                                                   | AC Ref                | Type             | Setup                            | Pass/Fail                                                                                                                                                                                                                                                                                                      |
| ---------- | ------------------------------------------------------------------------------------------- | --------------------- | ---------------- | -------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| T-LOOP-001 | implementer can commit + open PR                                                            | AC-1                  | integration-test | contents:write principal         | commit + PR succeed                                                                                                                                                                                                                                                                                            |
| T-LOOP-002 | implementer denied the merge — the `perm.denied` contract                                   | AC-2                  | api-contract     | no merge perm                    | merge denied with `{code:"perm.denied"}`                                                                                                                                                                                                                                                                       |
| T-LOOP-003 | reviewer denied commit, allowed review                                                      | AC-3                  | integration-test | reviews:write, no contents:write | commit denied; review accepted                                                                                                                                                                                                                                                                                 |
| T-LOOP-004 | maintainer merges only after requirement met                                                | AC-4                  | integration-test | merge holder                     | merge succeeds post-approval                                                                                                                                                                                                                                                                                   |
| T-LOOP-005 | no role label in any enforcement path                                                       | AC-5                  | build-gate       | source                           | grep: no "implementer"/"reviewer"/"maintainer" in enforcement                                                                                                                                                                                                                                                  |
| T-LOOP-014 | governed `but pr`/`but review` actions exist + are wired/permission-checked at the boundary | AC: pr-review-surface | build-gate       | source                           | `but pr new` / `but review approve`/`request-changes`/`comment` exist at the `but-api` boundary, each wrapped with its `Authority` (`pull_requests:write`/`reviews:write`/`comments:write`) — structural presence + wiring asserted (R-NEW-3, make-explicit); runtime denial covered by T-LOOP-002/T-AUTHZ-009 |
| T-LOOP-013 | governed path is traversable (irrigation half)                                              | AC: traversable       | integration-test | denied implementer               | following the `remediation_hint` (feature branch → reviewed merge) succeeds                                                                                                                                                                                                                                    |
| T-LOOP-006 | full-loop integration test (the reference flow)                                             | AC: full-loop         | integration-test | real surface+git, 3 principals   | all five loop assertions hold (reviews via governed `but review`)                                                                                                                                                                                                                                              |

### UC-LOOP-02: Human-at-feature + AI-at-code as pure config

| #          | Criterion                                               | AC Ref | Type             | Setup                        | Pass/Fail                                                  |
| ---------- | ------------------------------------------------------- | ------ | ---------------- | ---------------------------- | ---------------------------------------------------------- |
| T-LOOP-007 | configure two-group requirement in gates.toml           | AC-1   | integration-test | `gates.toml`                 | `require_approval_from_group=[code-reviewers,maintainers]` |
| T-LOOP-008 | AI-only approval blocks (human required)                | AC-2   | integration-test | only code-reviewers approval | blocked                                                    |
| T-LOOP-009 | human-only approval blocks (AI required)                | AC-3   | integration-test | only maintainers approval    | blocked                                                    |
| T-LOOP-010 | both approvals @head → merge proceeds                   | AC-4   | integration-test | both groups approve          | merge lands                                                |
| T-LOOP-011 | two-tier model is pure config, no human/AI code         | AC-5   | build-gate       | source                       | grep: no enforcement branch on human-vs-AI                 |
| T-LOOP-012 | two-tier integration test (only-one blocked, both pass) | AC-6   | integration-test | real surface+git             | AI-only blocked; human-only blocked; both proceeds         |

> LOOP note: 14 ACs / 14 criteria. The +1 vs v1.2.0 is the governed `but pr`/`but review` surface AC (UC-LOOP-01) ↔ T-LOOP-014 (A2.6 / R-NEW-3 make-explicit), typed `build-gate` (its core assertion is the structural presence + `_with_authz` wiring of the verbs; runtime denial is covered by T-LOOP-002 / T-AUTHZ-009). T-LOOP-002 is typed `api-contract` (the `perm.denied` shape).

---

## MGMT: Governance Management UI

UI criteria use Playwright **component tests** (`pnpm test:ct`) over the **real** `packages/ui`/desktop components (no mocked UI) and one **e2e** full-flow; `[build-gate]` for the no-bypass invariant. **All 38 component-test criteria are gated on T-MGMT-000** (the desktop CT config — B14).

### UC-MGMT-01: Admin-gated settings surface

| #          | Criterion                                                                                                | AC Ref                | Type           | Setup           | Pass/Fail                                                                                                                                                                                                                     |
| ---------- | -------------------------------------------------------------------------------------------------------- | --------------------- | -------------- | --------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| T-MGMT-000 | desktop CT config scaffolded; governance component tests compile + render                                | AC: desktop-CT-config | build-gate     | source          | an `apps/desktop` Playwright CT / Vitest config exists; `pnpm test:ct:desktop` runs governance component tests against a `but-sdk` mock layer — **hard prerequisite for all 38 component-test criteria** (B14)                |
| T-MGMT-001 | governance page added to `projectSettingsPages` (`adminOnly`)                                            | AC-1                  | component-test | real settings   | renders via `SettingsModalLayout`                                                                                                                                                                                             |
| T-MGMT-002 | page hidden from sidebar for a non-admin (cloud-role visibility, server `administration:write` enforces) | AC-2                  | component-test | `isAdmin=false` | governance item absent; backend gate still hit if bypassed (B18)                                                                                                                                                              |
| T-MGMT-003 | opened via existing settings shortcut; no new route                                                      | AC-3                  | component-test | settings modal  | no `[projectId]` route added                                                                                                                                                                                                  |
| T-MGMT-004 | four tabs via existing `shared/Tabs`                                                                     | AC-4                  | component-test | governance page | Principals/Groups/Branch Gates/Rules render                                                                                                                                                                                   |
| T-MGMT-005 | admin sees page, non-admin doesn't                                                                       | AC-5                  | component-test | test:ct         | admin renders; non-admin absent                                                                                                                                                                                               |
| T-MGMT-033 | `ProjectSettingsPageId` extended; branch renders `GovernanceSettings`                                    | AC-6                  | build-gate     | source          | union has the governance variant; switch branch renders GovernanceSettings.svelte                                                                                                                                             |
| T-MGMT-034 | SDK regenerated before UI wiring; components type-check                                                  | AC-7                  | build-gate     | source          | `pnpm build:sdk && pnpm format` ran; MGMT components import the new SDK types                                                                                                                                                 |
| T-MGMT-043 | fail-closed on missing config; `but governance init` named fast-follow (not built)                       | AC: bootstrap         | build-gate     | source          | governance fails closed on missing `permissions.toml` (UC-AUTHZ-04 family, covered by T-AUTHZ-029); the spec/CLI surface names `but governance init` as a documented fast-follow and asserts it is NOT built in the POC (B10) |

### UC-MGMT-02: View principals + edit permissions

| #          | Criterion                                                                     | AC Ref        | Type             | Setup                     | Pass/Fail                                                                                           |
| ---------- | ----------------------------------------------------------------------------- | ------------- | ---------------- | ------------------------- | --------------------------------------------------------------------------------------------------- |
| T-MGMT-006 | principals list shows effective set + source-of-grant                         | AC-1          | component-test   | seeded principals         | own vs group-inherited labeled                                                                      |
| T-MGMT-007 | per-principal editor (SegmentControl + Toggles); toggles are local-state only | AC-2          | component-test   | principal row             | preset + toggles render; no per-toggle write                                                        |
| T-MGMT-008 | inherited rows read-only; preset preserves union                              | AC-3          | component-test   | grouped principal         | inherited disabled; preset sets own grants only                                                     |
| T-MGMT-009 | batch `[Save changes]` → `but perm grant/revoke`(+`group`) sequence           | AC-4          | component-test   | editor                    | correct SDK call(s) issued on save, not per toggle (B16)                                            |
| T-MGMT-010 | effective display updates only after commit                                   | AC-5          | component-test   | edit+commit               | pending until commit                                                                                |
| T-MGMT-044 | register-on-first-grant; empty principal decommissioned                       | AC: lifecycle | integration-test | grant to new id; empty id | first grant creates the entry; an entry with no grants/memberships is denied all (B9 / UC-AUTHZ-04) |
| T-MGMT-011 | batch-save issues `but perm`; inherited non-interactive                       | AC-6          | component-test   | test:ct                   | asserts call(s) + disabled                                                                          |

### UC-MGMT-03: Groups

| #          | Criterion                                                             | AC Ref      | Type           | Setup                       | Pass/Fail                                                                                            |
| ---------- | --------------------------------------------------------------------- | ----------- | -------------- | --------------------------- | ---------------------------------------------------------------------------------------------------- |
| T-MGMT-012 | groups listed as `ExpandableSection` (grants + members)               | AC-1        | component-test | seeded groups               | renders                                                                                              |
| T-MGMT-013 | create/grant/add-member → `but group` SDK calls                       | AC-2        | component-test | groups tab                  | correct calls                                                                                        |
| T-MGMT-014 | empty state with create-first-group                                   | AC-3        | component-test | no groups                   | `EmptyStatePlaceholder`                                                                              |
| T-MGMT-015 | group change reflected in principals after commit                     | AC-4        | component-test | edit+commit                 | effective set updates                                                                                |
| T-MGMT-045 | group delete → `but group delete`; inherited grants drop at next read | AC: delete  | component-test | delete a group              | `but group delete` SDK call; affected principals lose inherited grants on next target-ref read (B11) |
| T-MGMT-046 | destructive-action confirmation (group delete / last-member warning)  | AC: confirm | component-test | delete / remove last member | confirmation dialog shown before staged write; last-member-of-required-group warning banner (B17)    |
| T-MGMT-016 | group create+grant+add-member SDK calls                               | AC-5        | component-test | test:ct                     | asserts calls                                                                                        |

### UC-MGMT-04: Branch gates

| #          | Criterion                                          | AC Ref      | Type           | Setup                | Pass/Fail                                                                                          |
| ---------- | -------------------------------------------------- | ----------- | -------------- | -------------------- | -------------------------------------------------------------------------------------------------- |
| T-MGMT-017 | branch gates listed with fields                    | AC-1        | component-test | seeded `gates.toml`  | fields render                                                                                      |
| T-MGMT-018 | edit fields → gate-config write (`gates.toml`)     | AC-2        | component-test | gates tab            | correct SDK call                                                                                   |
| T-MGMT-019 | add gate for new branch; empty state               | AC-3        | component-test | no gates             | add + `EmptyStatePlaceholder`                                                                      |
| T-MGMT-020 | required-group selector offers only defined groups | AC-4        | component-test | groups+gates         | options = groups                                                                                   |
| T-MGMT-047 | unprotect-branch confirmation dialog               | AC: confirm | component-test | toggle protected OFF | "Unprotect branch main? Merges will no longer require review." shown before the staged write (B17) |
| T-MGMT-021 | gate edit SDK call + pending state                 | AC-5        | component-test | test:ct              | asserts call + pending                                                                             |

### UC-MGMT-05: Per-agent rules (reuse)

| #          | Criterion                                             | AC Ref | Type           | Setup             | Pass/Fail                     |
| ---------- | ----------------------------------------------------- | ------ | -------------- | ----------------- | ----------------------------- |
| T-MGMT-022 | `RulesList` reused with new `principalId` prop        | AC-1   | component-test | rules tab         | scoped list                   |
| T-MGMT-023 | `Rule`/`RuleEditor`/etc render unchanged              | AC-2   | component-test | rules tab         | existing components unchanged |
| T-MGMT-024 | `principalId` scopes query; unset = existing behavior | AC-3   | component-test | with/without prop | scoped vs default             |
| T-MGMT-025 | empty/placeholder when no principal/rules             | AC-4   | component-test | no rules          | placeholder                   |
| T-MGMT-026 | `RulesList` principalId scopes; unset unchanged       | AC-5   | component-test | test:ct           | asserts both                  |

### UC-MGMT-06: Governed front-end

| #          | Criterion                                                                                                     | AC Ref               | Type           | Setup                  | Pass/Fail                                                                                                                                                                                                                               |
| ---------- | ------------------------------------------------------------------------------------------------------------- | -------------------- | -------------- | ---------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| T-MGMT-027 | every write via `but-api`→Tauri→`but-sdk`; never a direct config write                                        | AC-1                 | build-gate     | source                 | grep: governance components issue no direct `.gitbutler/*` writes                                                                                                                                                                       |
| T-MGMT-035 | pending-state derived from working-tree-vs-target-ref diff                                                    | AC-2                 | component-test | edit+commit            | UI derives ○ from diff; no renderer direct-file-write                                                                                                                                                                                   |
| T-MGMT-036 | pending-state store is CLIENT-ONLY (no `+page.server.ts`)                                                     | AC-3                 | build-gate     | source                 | grep: no `+page.server.ts` under governance; store uses client APIs                                                                                                                                                                     |
| T-MGMT-028 | staged change shows pending (○ + commit `InfoMessage`)                                                        | AC-4                 | component-test | edit                   | banner appears; clears on commit                                                                                                                                                                                                        |
| T-MGMT-048 | commit semantics (msg, clean-tree hides banner, implicit staging, through commit gate)                        | AC: commit-semantics | component-test | edit+commit            | "Commit changes" commits `.gitbutler/*.toml` with `chore: update governance config`; clean tree hides banner; commit passes the commit gate (B15)                                                                                       |
| T-MGMT-029 | read-only without `administration:write`                                                                      | AC-5                 | component-test | non-admin-write viewer | controls disabled + info banner                                                                                                                                                                                                         |
| T-MGMT-030 | self-escalation not optimistically applied                                                                    | AC-6                 | component-test | self admin grant       | denial surfaced, control not flipped                                                                                                                                                                                                    |
| T-MGMT-031 | structured denial via danger `InfoMessage`; errors via `chipToasts`                                           | AC-7                 | api-contract   | denied write           | `{code,message,remediation_hint}` shown                                                                                                                                                                                                 |
| T-MGMT-032 | pending banner + read-only + denied-no-apply (full flow)                                                      | AC-8                 | e2e-automated  | full admin flow        | all three hold                                                                                                                                                                                                                          |
| T-MGMT-042 | MGMT config write resolves the human fleet-owner; cloud-role visibility vs `administration:write` enforcement | AC-9                 | build-gate     | source                 | desktop config-management commands resolve the human principal from `UserService`/forge session; the agent authorization path has no superuser branch; cloud `User.role` gates visibility, `administration:write` gates the write (B18) |

### UC-MGMT-07: Error handling & accessibility

| #          | Criterion                                                      | AC Ref | Type           | Setup              | Pass/Fail                                      |
| ---------- | -------------------------------------------------------------- | ------ | -------------- | ------------------ | ---------------------------------------------- |
| T-MGMT-037 | error boundary catches `GovernanceSettings` failures           | AC-1   | component-test | error thrown       | fallback renders                               |
| T-MGMT-038 | tab navigation has aria-labels + keyboard nav                  | AC-2   | component-test | keyboard nav       | Tab/Enter/Arrow work; aria present             |
| T-MGMT-039 | IPC failure surfaces structured denial + retry                 | AC-3   | component-test | Tauri command fail | danger InfoMessage + retry button              |
| T-MGMT-040 | retry re-issues; persistent failure keeps safe read-only state | AC-4   | component-test | retry then fail    | re-issues; stays read-only                     |
| T-MGMT-041 | UC-MGMT-07 component test (boundary + keyboard + IPC)          | AC-5   | e2e-automated  | test:ct            | (a) boundary (b) keyboard (c) IPC denial+retry |

> MGMT note: 49 ACs / 49 criteria. The +7 vs v1.2.0: T-MGMT-043 (B10 bootstrap, build-gate), T-MGMT-044 (B9 lifecycle, integration), T-MGMT-045 (B11 group delete, component), T-MGMT-046 + T-MGMT-047 (B17 confirmations ×2, component), T-MGMT-048 (B15 commit semantics, component), and T-MGMT-000 (B14 desktop-CT-config, build-gate). All component criteria are gated on T-MGMT-000.

---

## Summary

| Type                 | Count   |
| -------------------- | ------- |
| `[integration-test]` | 67      |
| `[component-test]`   | 38      |
| `[api-contract]`     | 7       |
| `[build-gate]`       | 15      |
| `[e2e-automated]`    | 2       |
| **Total**            | **129** |

| Group     | UCs    | ACs     | Criteria |
| --------- | ------ | ------- | -------- |
| AUTHZ     | 4      | 33      | 33       |
| GRPS      | 2      | 14      | 14       |
| GATES     | 2      | 19      | 19       |
| LOOP      | 2      | 14      | 14       |
| MGMT      | 7      | 49      | 49       |
| **Total** | **17** | **129** | **129**  |

**AC coverage: 129/129 (100%).** The headline gate is **T-LOOP-006** (the proven-reference-flow): it must be green before the deep build proceeds. The `[build-gate]` honesty invariants — no role name in enforcement (T-AUTHZ-016, T-LOOP-005), no `Permission` lock overload (asserted structurally), the **N-API audit** (T-AUTHZ-016b, R14), and the **desktop-CT-config** (T-MGMT-000, B14) — block the slice regardless of the other lanes.

## Maintenance notes

- Adding a UC/AC: add a `T-{PREFIX}-NNN` row referencing the new AC; keep IDs stable.
- A `[build-gate]` grep/structural invariant failing means the slice is **not done** even if integration lanes are green (the functional-not-role, no-`Permission`-overload, and N-API-no-bypass invariants are non-negotiable).
- Criteria are tallied **by test type, not by risk severity** — R6's Medium→High upgrade and R14 add no ripple to the type counts (the type deltas vs v1.2.0 are the +2 build-gates for the N-API audit and the desktop CT config, +3 integration for the new make-explicit/lifecycle/bootstrap behaviors, and +4 component for the new MGMT UI behaviors; type _re-labels_ of a few existing rows reflect their true assertion class — contract-shape → api-contract, structural/grep → build-gate — and net to the canonical 67/38/7/15/2 split).
- The merge-gate review criteria test the **governed review-submission path** only; never add a test asserting the forgeable direct DB write is blocked, nor one asserting raw-git is blocked — both encode false guarantees (R6 / R1 accepted-leak).
