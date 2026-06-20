# AUTHZ-005: Identity confinement at the but-api/CLI boundary — acting principal only from BUT_AGENT_HANDLE, no honored in-band identity-override, authority never from an agent claim

## What this does

Confines the acting identity at the but-api/CLI boundary: the acting principal is resolved ONLY via `but_authz::resolve_principal` from `BUT_AGENT_HANDLE` (the same fail-closed resolver the commit gate already uses), and there is NO honored in-band identity-override path. A grep of the CLI today confirms NO `--as` / impersonation / `act_as` / `as_principal` flag exists, so the HONEST version of this task asserts that no in-band identity-override path exists or is honored AND that authority is never read from an agent-supplied claim (T-AUTHZ-020). The PRIMARY proof is the T-AUTHZ-023 confinement integration: a `{reviews:write}` agent (BUT_AGENT_HANDLE=reviewer) is denied BOTH a merge of another handle's change AND a self-grant-merge config edit (the latter by composing AUTHZ-006's guard FUNCTION, which denies BEFORE any write). The load-bearing proof that the resolver consumed the env var is that the resolved acting principal id == the `BUT_AGENT_HANDLE` value. The accepted env-re-export leak is honest — there is NO test asserting a re-exporting process is blocked.

## Why

Sprint 02 · PRD UC-AUTHZ-03 · capabilities CAP-AUTHZ-01. A dispatched agent must be bound to its own
`BUT_AGENT_HANDLE`; authority is read from committed config, never an agent claim. The gate: the
governed action remains bound to the dispatched handle and is denied `perm.denied` when that handle
lacks authority.

An in-band `--as <other>` identity override is not honored; the unsupported attempt is rejected
before any borrowed-handle action can run.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-api confinement_reviewer_denied_other_merge_and_self_grant` (integration: a `{reviews:write}` agent denied other-handle-merge AND self-grant config edit, the latter via AUTHZ-006's guard function which denies before any write). Full gate set in the spec below.

## Scope

- `crates/but-api/src/legacy/merge_gate.rs` (CONSUME — no edit beyond what AUTHZ-004 owns) — the merge path the confinement test drives; the reviewer's other-handle merge is denied here
- `crates/but-api/src/legacy/config_mutate.rs` (CONSUME — AUTHZ-006 owns) — the enforce_administration_write_gate the reviewer's self-grant edit is denied at (the guard denies BEFORE any write; there is no persisted write path to mutate yet)
- `crates/but-api/tests/confinement.rs` (NEW) — the T-AUTHZ-023 confinement integration (reviewer denied other-handle-merge AND self-grant config edit) + the authority-from-config-not-claim proof + the resolved-principal-id==BUT_AGENT_HANDLE proof
- `crates/but/tests/but/command/confinement.rs` (NEW) — CLI snapbox: the acting principal is resolved only from BUT_AGENT_HANDLE; an in-band `--as` override is NOT honored (clap rejects the unknown flag)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: AUTHZ-005 - Identity confinement at the but-api/CLI boundary: acting principal only from BUT_AGENT_HANDLE, no honored in-band identity-override, authority never from an agent claim
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     M  (150 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-AUTHZ-03
CAPABILITIES: CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-api confinement   |   cargo test -p but confinement
  check: cargo check -p but-api -p but --all-targets
  lint:  cargo clippy -p but-api -p but --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Integration tests are green against real but-api + real git + real but-db: a dispatched {reviews:write} agent (BUT_AGENT_HANDLE=reviewer) is denied BOTH (a) a merge of another handle's change (perm.denied naming merge; nothing merged) AND (b) a self-grant-merge config edit through AUTHZ-006's enforce_administration_write_gate FUNCTION (perm.denied naming administration:write; the guard denies BEFORE any write — there is no persisted write path to mutate yet) — the T-AUTHZ-023 confinement+authority integration. The acting principal is resolved ONLY from BUT_AGENT_HANDLE via but_authz::resolve_principal, PROVEN load-bearingly by the resolved acting principal id == the BUT_AGENT_HANDLE value (the denial message NAMES "reviewer"). An in-band identity-override (--as) is NOT honored (no such flag exists today, grep-confirmed — clap rejects it). Authority is read only from committed config, never an agent-supplied claim (T-AUTHZ-020). HONEST accepted-leak: there is NO test asserting a re-exporting process is blocked (T-AUTHZ-026).

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST resolve the acting principal ONLY via but_authz::resolve_principal from BUT_AGENT_HANDLE against the committed target-ref GovConfig — the SAME resolver the commit gate uses (crates/but-api/src/commit/gate.rs:67 resolve_principal_from_env). The principal is NEVER taken from a CLI argument, a claim, or any in-band override (CAP-AUTHZ-01; UC-AUTHZ-03).
- [MUST] MUST make AC-2's load-bearing POSITIVE assertion that the resolved acting principal id == the BUT_AGENT_HANDLE value: set BUT_AGENT_HANDLE=reviewer, invoke a governed action, and assert the denial message NAMES "reviewer" as the acting principal — proving resolve_principal CONSUMED the env var (a disconnected/hardcoded resolver that ignores BUT_AGENT_HANDLE would name a different/default id and FAIL this). The "clap rejects --as" check is the SECONDARY assertion (no in-band override path). DO NOT rest the proof on the clap-rejects check alone (it passes even for a resolver that ignores BUT_AGENT_HANDLE — the vacuous hole C3 closes).
- [MUST] MUST plan the HONEST version of the in-band identity-override: a grep of the CLI (rg '\-\-as|impersonat|act_as|as_principal' crates/but/src crates/but-api/src) returns ZERO matches — there is NO --as / impersonation flag today. The task therefore asserts (a) NO in-band identity-override path exists or is honored (an attempted `--as maint` is rejected by clap as an unknown argument, exit != 0, and the action does NOT run as maint), and (b) authority is never read from an agent-supplied claim (T-AUTHZ-020). DO NOT fabricate a --as flag to then "deny" it — assert the absence/non-honoring against what actually exists.
- [MUST] MUST drive the confinement through the REAL governed actions: the reviewer's OTHER-HANDLE MERGE is denied at the merge gate (AUTHZ-004 / GATES-003 path) and the reviewer's SELF-GRANT CONFIG EDIT is denied at AUTHZ-006's enforce_administration_write_gate FUNCTION. This task COMPOSES those two gates into the T-AUTHZ-023 integration; it does NOT re-implement either gate. The self-grant half proves the guard returns the structured denial for a {reviews:write} principal — i.e. the guard denies BEFORE any write — NOT that a persisted config write was blocked (the persisted-write consumer is Sprint 05).
- [NEVER] NEVER write a test asserting that an agent which re-exports BUT_AGENT_HANDLE to a different value in a child process is BLOCKED — env re-export is a DOCUMENTED ACCEPTED LEAK (T-AUTHZ-026 confinement honesty). Asserting it is prevented encodes a false guarantee. Document the accepted leak in notes; the build-gate (AC-4) enforces NO such test exists. NOTE (M7): the AC-4 honesty grep is TERMINOLOGY-based (best-effort) — it catches keyword-named false-guarantee tests only; a reviewer MUST manually inspect any std::process::Command / child-process env-manipulation test in confinement.rs for a smuggled re-export-blocked assertion the grep would miss.
- [NEVER] NEVER read the acting principal's AuthoritySet from an agent-supplied claim/argument — only from the committed config (UC-AUTHZ-03 AC-3 / T-AUTHZ-020). authorize() exposes no claim parameter (AUTHZ-003 proved this structurally); this task confirms the boundary preserves it. ACCEPTED LIMITATION (M5): but_authz::Principal::new(id, authorities, groups) is PUBLIC — a caller COULD construct a Principal with an arbitrary AuthoritySet and pass it to authorize(), bypassing resolve_principal. This is an accepted boundary trust: but-api routes MUST bind identity via resolve_principal (never Principal::new with caller-supplied authorities). AC-3 asserts the deny message names the env-derived principal id (weak structural evidence that resolve_principal — not a hand-built Principal — was the source). Document this in the accepted-leaks note.
- [NEVER] NEVER branch on a role NAME (implementer/reviewer/maintainer) in the confinement ENFORCEMENT path — confinement is by handle + the functional Authority set alone (T-LOOP-005 / T-AUTHZ-016 grep-asserted via AUTHZ-008). Fixture handle names like "reviewer" are acceptable as test DATA, not as enforcement branches.
- [STRICTLY] STRICTLY surface the two denials as the structured {error:{code,message}} exit-1 contract (the commit gate's classify_error pattern returns CommitGateError{code,message} — remediation_hint is dropped, MGMT-IPC-002; assert {code,message} only) — the other-handle merge denies perm.denied naming merge; the self-grant config edit denies perm.denied naming administration:write.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: a {reviews:write} agent denied BOTH an other-handle merge AND a self-grant-merge config edit (the guard denies before any write)
- [ ] AC-2: resolved acting principal id == BUT_AGENT_HANDLE value (denial names "reviewer"); no honored in-band identity-override (--as rejected by clap)
- [ ] AC-3: authority read only from committed config, never an agent-supplied claim; deny message names the env-derived principal id
- [ ] AC-4: HONESTY — NO test asserts the accepted env-re-export leak is blocked (build-gate; terminology-based, best-effort)
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: A {reviews:write} agent is denied BOTH an other-handle merge AND a self-grant-merge config edit [PRIMARY]
  GIVEN: fixture `confined_repo` (reviewer=[reviews:write]; maint=[merge]; admin=[administration:write,merge]; open review on feat with a distinct approval @head)
  WHEN:  BUT_AGENT_HANDLE=reviewer attempts (a) a merge of another handle's change, then (b) a self-grant-merge config edit composed via AUTHZ-006's enforce_administration_write_gate FUNCTION
  THEN:  (a) denied perm.denied naming merge (exit 1, nothing merged, trunk == base); (b) the admin-write guard returns Err(Denial) perm.denied naming administration:write — the guard denies BEFORE any write (there is no persisted write path yet; reviewer cannot reach a config mutation)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api forge seam + config-mutate guard function + real git + real but-db
  VERIFY: cargo test -p but-api confinement_reviewer_denied_other_merge_and_self_grant
  SCENARIO: NEGATIVE_CONTROL would fail if the reviewer's merge proceeds (no-op gate or authority from a claim); the admin-write guard returns Ok for the {reviews:write} reviewer (so it could reach a config mutation); the principal is taken from a caller argument; either denial returns a code other than perm.denied.

AC-2: Acting principal resolved only from BUT_AGENT_HANDLE (resolved id == env value); no honored in-band identity-override
  GIVEN: fixture `confined_repo`, BUT_AGENT_HANDLE=reviewer
  WHEN:  a governed action is invoked, and an in-band override `--as maint` is attempted (no honored such flag exists today)
  THEN:  the resolved acting principal id == "reviewer" (the BUT_AGENT_HANDLE value) — proven because the governed denial message NAMES "reviewer" as the acting principal; AND the `--as maint` override is NOT honored (clap rejects the unknown argument, exit != 0), and no action runs as maint
  TEST_TIER: integration   VERIFICATION_SERVICE: real but CLI + real but-api + real git
  VERIFY: cargo test -p but confinement_no_inband_identity_override
  SCENARIO: NEGATIVE_CONTROL would fail if the denial names a default/other id rather than "reviewer" (proving the resolver ignored BUT_AGENT_HANDLE — a disconnected/hardcoded resolver the clap-rejects check alone would not catch); a --as flag is added and honored so an agent acts as a different principal; the principal is read from an argument other than BUT_AGENT_HANDLE.

AC-3: Authority read only from committed config, never an agent-supplied claim; deny names the env-derived principal id
  GIVEN: fixture `confined_repo`, BUT_AGENT_HANDLE=reviewer
  WHEN:  a governed merge action is invoked; the test confirms no parameter/field injects an authority claim
  THEN:  reviewer is denied merge perm.denied AND the deny message names the env-derived principal id "reviewer" — its effective set is sourced solely from committed config; no claim path can widen it
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api + committed GovConfig + real git
  VERIFY: cargo test -p but-api confinement_authority_from_config_not_claim
  SCENARIO: NEGATIVE_CONTROL would fail if authority is read from an agent-supplied claim that widens reviewer's set; the enforcement path exposes a claim parameter; reviewer is allowed merge because a claim was honored; the deny names an id other than the env-derived "reviewer" (evidence the source was a hand-built Principal::new, not resolve_principal — the M5 accepted limitation).

AC-4: HONESTY — NO test asserts the accepted env-re-export leak is blocked [build-gate]
  GIVEN: the AUTHZ-005 test suite
  WHEN:  the build-gate grep runs over the test files
  THEN:  there is NO test asserting a re-exporting child process is BLOCKED from acting as a different handle — the env-re-export leak is documented and accepted, never claimed as prevented. The grep is TERMINOLOGY-based (best-effort); a reviewer manually inspects any std::process::Command / child-env test for a smuggled re-export-blocked assertion
  TEST_TIER: unit (build-gate)   VERIFICATION_SERVICE: source grep (no runtime I/O)   UNIT_TEST_JUSTIFIED: anti-false-guarantee structural invariant verified by grep with zero runtime I/O; it enforces the ABSENCE of a test that would encode a false guarantee. The confinement's real behavioral guarantees (AC-1..3) are integration-tested; this gate only enforces honesty. The grep is terminology-based and best-effort, complemented by manual reviewer inspection.
  VERIFY: ! grep -rEn 'reexport|re_export|re-export|child.*BUT_AGENT_HANDLE.*block|prevents?.*re.?export' crates/but-api/tests crates/but/tests
  SCENARIO: NEGATIVE_CONTROL would fail if a test asserts a re-exporting child is blocked (the false guarantee the PRD accepts as a leak); the suite claims re-export confinement is enforced; the honesty grep is a no-op.

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, error): {reviews:write} agent denied BOTH an other-handle merge AND a self-grant-merge config edit (the admin-write guard denies before any write), both perm.denied, nothing merged / guard returns Err (T-AUTHZ-023)
    VERIFY: cargo test -p but-api confinement_reviewer_denied_other_merge_and_self_grant
- TC-2 (-> AC-2, edge): resolved acting principal id == BUT_AGENT_HANDLE value (denial names "reviewer"); no honored in-band --as override; the override does not run as the borrowed handle (T-AUTHZ-018, T-AUTHZ-019)
    VERIFY: cargo test -p but confinement_no_inband_identity_override
- TC-3 (-> AC-3, edge): authority read only from committed config, never from an agent-supplied claim; deny names the env-derived id (T-AUTHZ-020)
    VERIFY: cargo test -p but-api confinement_authority_from_config_not_claim
- TC-4 (-> AC-4, structural): honesty — NO test asserts the accepted env-re-export leak is blocked (T-AUTHZ-026); terminology-based grep + manual reviewer inspection
    VERIFY: ! grep -rEn 'reexport|re_export|re-export|child.*BUT_AGENT_HANDLE.*block|prevents?.*re.?export' crates/but-api/tests crates/but/tests

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides: the identity-confinement composition at the but-api/CLI boundary — acting principal resolved only from BUT_AGENT_HANDLE (resolved id == env value, proven by the denial naming it); no honored in-band identity-override; authority never from an agent claim; the T-AUTHZ-023 {reviews:write}-agent denied-both integration (the self-grant half composes AUTHZ-006's guard, which denies before any write); honest accepted-leaks (env re-export not tested as blocked; Principal::new public documented)
consumes: but_authz::resolve_principal from BUT_AGENT_HANDLE + authorize + Denial (AUTHZ-003); the merge gate (AUTHZ-004 / GATES-003) for the other-handle-merge denial; the admin-write config-mutate guard FUNCTION (AUTHZ-006) for the self-grant-config-edit denial
boundary_contracts:
  - CAP-AUTHZ-01: the acting principal is resolved only from BUT_AGENT_HANDLE (resolved id == env value); no in-band override is honored; authority is read from committed config, never an agent claim; a {reviews:write} agent is denied both an other-handle merge and a self-grant config edit (the guard denies before any write).
  - Honesty: env re-export is a documented accepted leak — NOT tested as blocked. Principal::new is public — callers MUST bind identity via resolve_principal (documented accepted limitation).

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-api/tests/confinement.rs (NEW) — the T-AUTHZ-023 confinement integration + authority-from-config-not-claim proof + resolved-id==env proof
  - crates/but/tests/but/command/confinement.rs (NEW) — CLI snapbox: principal only from BUT_AGENT_HANDLE; --as not honored
writeProhibited:
  - crates/but-authz/** — CONSUME resolve_principal/authorize/Denial; do NOT modify the primitive (Principal::new staying public is an accepted limitation, not a fix scoped here)
  - crates/but-api/src/legacy/merge_gate.rs — OWNED by AUTHZ-004 / GATES-003; this task COMPOSES it, does not edit it
  - crates/but-api/src/legacy/config_mutate.rs — OWNED by AUTHZ-006; this task COMPOSES the admin-write guard FUNCTION, does not edit it
  - crates/but-error/src/lib.rs — reuse the but-authz Denial perm.denied contract
  - any gitbutler-* crate beyond what the action boundary strictly requires (crates/AGENTS.md)
  - Any file not listed in write_allowed

--------------------------------------------------------------------------------
OUT OF SCOPE
--------------------------------------------------------------------------------
  - Implementing/denying a --as flag that does not exist today: NO such flag exists (grep-confirmed). This task asserts the ABSENCE/non-honoring, it does NOT add a flag to then deny.
  - Env-var re-export confinement: a DOCUMENTED ACCEPTED LEAK (T-AUTHZ-026). No test asserts a re-exporting process is blocked.
  - Restricting Principal::new visibility: an ACCEPTED LIMITATION (M5). The boundary trusts but-api routes to bind identity via resolve_principal; AC-3 gives weak structural evidence (the deny names the env-derived id). Not fixed here.
  - The merge gate (AUTHZ-004/GATES-003) and the admin-write config-mutate guard (AUTHZ-006) internals — this task composes them.
  - The persisted config-write block — there is no write path yet; the consumer is Sprint 05. The self-grant half proves the guard denies before any write.
  - remediation_hint surfacing — dropped by the mirror (MGMT-IPC-002, Sprint 06a). Assert {code,message} only.
  - The forgeable direct-DB-write / forge-UI / raw-push bypass — accepted-leaks (R6/R1), NOT tested.

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-api/src/commit/gate.rs (50-96)
   Focus: THE RESOLVER PATTERN — resolve_principal_from_env(&cfg) (67) is the ONLY way the acting principal is bound; authorize(&principal, Authority, &cfg) (69); classify_error (83) returns CommitGateError{code,message} (no remediation_hint). The confinement reuses exactly this — there is no claim/argument path.
2. crates/but/src/utils/detect_agent.rs (60-115)
   Focus: how the CLI reads agent identity from env (detect_with(lookup)); confirm there is NO --as / impersonation flag — identity is env-only. The acting handle is BUT_AGENT_HANDLE, resolved by but_authz, NOT this detector (which is telemetry only).
3. crates/but-authz/src/authorize.rs (24-93) + the Principal::new signature
   Focus: CONSUME-ONLY — authorize takes only &Principal + Authority + &GovConfig (no claim param); resolve_principal reads BUT_AGENT_HANDLE only; Denial::no_handle / unknown_principal. CONFIRM Principal::new is PUBLIC (the M5 accepted limitation) — callers MUST use resolve_principal, never hand-build a Principal with caller-supplied authorities.
4. .spec/prds/governance/tasks/sprint-02-authz-fail-closed-identity-confinement/AUTHZ-004-merge-gate-fail-closed.md (full)
   Focus: SIBLING — the merge gate the reviewer's other-handle merge is denied at. Compose it; do not edit it.
5. .spec/prds/governance/tasks/sprint-02-authz-fail-closed-identity-confinement/AUTHZ-006-administration-write-guard.md (full)
   Focus: SIBLING — the enforce_administration_write_gate FUNCTION the reviewer's self-grant config edit is denied at (the guard denies before any write). Compose it; do not edit it.
6. crates/but-api/tests/commit_gate.rs (1-130)
   Focus: THE HARNESS — temp_env::with_var("BUT_AGENT_HANDLE", Some("reviewer"), ...), but_ctx::Context::from_repo, governed_repo() via writable_scenario + invoke_bash, err.downcast_ref::<but_authz::Denial>(), assert denial.code AND that denial.message NAMES "reviewer" (the resolved-id==env proof). Mirror for the two confinement denials.
7. crates/but/tests/but/utils.rs (14-155)
   Focus: THE CLI HARNESS — Sandbox + env.but(...).env("BUT_AGENT_HANDLE","reviewer").assert() with .stderr_eq + [..]/... wildcards + exit codes; how an unknown `--as` flag surfaces as a clap error (exit != 0).

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- Confinement integration passes: `cargo test -p but-api confinement`  -> Exit 0; AC-1/AC-3 green (reviewer denied both; authority from config; deny names env-derived id)
- CLI confinement passes: `cargo test -p but confinement`  -> Exit 0; AC-2 green (resolved id == BUT_AGENT_HANDLE value; --as not honored)
- No honored in-band identity-override added: `! grep -rEn '\-\-as|impersonat|act_as|as_principal' crates/but/src crates/but-api/src`  -> No matches (the task does NOT introduce one; it asserts the absence)
- HONESTY build-gate: `! grep -rEn 'reexport|re_export|re-export|child.*BUT_AGENT_HANDLE.*block|prevents?.*re.?export' crates/but-api/tests crates/but/tests`  -> No matches (no false-guarantee env-re-export-blocked test). MANUAL: reviewer inspects any std::process::Command/child-env test in confinement.rs for a smuggled re-export-blocked assertion the terminology grep would miss
- No role-name in the confinement ENFORCEMENT path: `! grep -rEni 'implementer|reviewer|maintainer' crates/but-api/tests/confinement.rs` (test fixture handle names like "reviewer" are acceptable ONLY as data, not as enforcement branches — reviewer confirms no role-branching)
- Crates compile incl. tests: `cargo check -p but-api -p but --all-targets`  -> Exit 0
- Clippy clean: `cargo clippy -p but-api -p but --all-targets`  -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: Boundary-confinement-by-composition — the acting principal is resolved ONLY via but_authz::resolve_principal(BUT_AGENT_HANDLE) (the commit-gate resolver), PROVEN by the resolved id == the env value (the denial names "reviewer"); there is no claim/argument/in-band-override path; and the T-AUTHZ-023 integration COMPOSES the merge gate (other-handle merge -> perm.denied) and the admin-write config-mutate guard FUNCTION (self-grant edit -> perm.denied, denied before any write) to prove a {reviews:write} agent can neither borrow merge authority nor reach its own grant. The env-re-export leak is documented and NOT tested as blocked; Principal::new staying public is a documented accepted limitation.
pattern_source: crates/but-api/src/commit/gate.rs:67 (resolve_principal_from_env — the only identity source) + the grep-confirmed absence of any --as flag + the denial message naming the env-derived id
anti_pattern: Resting the resolver proof on the clap-rejects check alone (vacuous — passes for a resolver that ignores BUT_AGENT_HANDLE); fabricating a --as flag to then deny it; reading authority from an agent claim; hand-building a Principal::new with caller-supplied authorities instead of resolve_principal; binding a default/anonymous principal; branching on a role name in the enforcement path; claiming the self-grant blocked a persisted write (the guard denies before any write; consumer is Sprint 05); or writing a test that asserts the accepted env-re-export leak is blocked (false guarantee, T-AUTHZ-026).

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — Composes the merge gate and the admin-write config-mutate guard FUNCTION into the T-AUTHZ-023 confinement integration, proves the acting principal is resolved only from BUT_AGENT_HANDLE (resolved id == env value, denial names it) with no honored in-band identity-override and no agent-claim authority path, and keeps the accepted leaks honest (no env-re-export false-guarantee test; Principal::new public documented). Owns the confinement composition, the load-bearing resolved-id==env assertion, and integration TDD against real but-api + real git + real but-db.
reviewer: rust-reviewer
coding_standards: crates/AGENTS.md, crates/WORKSPACE_MODEL.md, crates/but/AGENTS.md, crates/but-api/src/commit/gate.rs

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: AUTHZ-004, AUTHZ-006, GATES-003, GATES-004, AUTHZ-002, AUTHZ-003   (the merge gate + the admin-write guard + the governed `but review approve` verb for the fixture + the but-authz primitive)
Blocks:     AUTHZ-008; Sprint 05, Sprint 06a
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "AUTHZ-005",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "notes": [
    "C2 (AC-1b honest scoping): the self-grant half composes AUTHZ-006's enforce_administration_write_gate FUNCTION; it proves the guard returns the structured perm.denied for a {reviews:write} principal — i.e. the guard denies BEFORE any write. There is no persisted write path yet (consumer is Sprint 05), so the assertion is 'the guard denies before any write', NOT 'permissions.toml unchanged after a write'.",
    "C3 (AC-2 load-bearing): the resolved acting principal id == BUT_AGENT_HANDLE value, proven because the denial message NAMES 'reviewer'. A disconnected/hardcoded resolver that ignores BUT_AGENT_HANDLE names a different id and fails. 'clap rejects --as' is the secondary assertion only.",
    "M5 (accepted limitation): but_authz::Principal::new(id, authorities, groups) is PUBLIC — a caller could hand-build a Principal with arbitrary authorities and bypass resolve_principal. Accepted boundary trust: but-api routes MUST bind identity via resolve_principal. AC-3 asserts the deny message names the env-derived principal id (weak structural evidence resolve_principal was the source).",
    "M7 (honesty grep best-effort): the AC-4 grep is terminology-based; a reviewer must manually inspect any std::process::Command / child-env test in confinement.rs for a smuggled re-export-blocked assertion.",
    "L3 (dependency gate): confined_repo seeds verdicts via governed `but review approve` (GATES-004, Sprint-01b); the fixture must fail with a clear dependency error if the verb is unavailable.",
    "M2: assertions scope to {code,message} (CommitGateError drops remediation_hint; MGMT-IPC-002)."
  ],
  "fixtures": {
    "confined_repo": {
      "description": "A real git repo (but-testsupport writable_scenario) whose target ref main has committed .gitbutler/permissions.toml (reviewer=[reviews:write]; maint=[merge]; admin=[administration:write,merge]) and a valid .gitbutler/gates.toml ([[branch]] main protected=true plus the Sprint-01b gate-requirement type=review min_approvals=1 require_distinct_from_author=true), with an open governed review on feat carrying a distinct approving verdict @head. The acting principal is resolved ONLY from BUT_AGENT_HANDLE; there is NO --as / impersonation flag on the CLI today (grep-confirmed) so a dispatched reviewer agent (BUT_AGENT_HANDLE=reviewer) is confined to its own handle. DEPENDENCY GATE (L3): the distinct approving verdict is seeded via the governed `but review approve` verb (GATES-004, Sprint-01b, In Progress); the fixture MUST fail with a clear dependency error if that verb is not yet available — never a silent direct-DB insert.",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"governance-base\");",
        "invoke_bash: write .gitbutler/permissions.toml ([[principal]] id=\"reviewer\" permissions=[\"reviews:write\"]; [[principal]] id=\"maint\" permissions=[\"merge\"]; [[principal]] id=\"admin\" permissions=[\"administration:write\",\"merge\"])",
        "invoke_bash: write a VALID .gitbutler/gates.toml ([[branch]] name=\"main\" protected=true) plus the Sprint-01b gate-requirement (type=\"review\" min_approvals=1 require_distinct_from_author=true)",
        "invoke_bash: stage and commit both blobs at refs/heads/main; create branch feat; commit a change so the feature head differs from main",
        "open a governed review on feat via the but-api forge path; record a distinct approving verdict from a non-author reviewer @head via the governed `but review approve` action (NOT a direct DB insert; FAIL CLOSED with a clear dependency error if the GATES-004 verb is unavailable), so the ONLY barrier to a merge is the merge authority"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN confined_repo (a dispatched agent bound to BUT_AGENT_HANDLE=reviewer holding only reviews:write) WHEN it attempts (a) a merge of another handle's change and (b) a self-grant-merge config edit composed via AUTHZ-006's enforce_administration_write_gate FUNCTION THEN (a) is denied error.code==\"perm.denied\" naming merge (exit 1, nothing merged) AND (b) the admin-write guard returns Err(Denial) error.code==\"perm.denied\" naming administration:write — the guard denies BEFORE any write (no persisted write path yet; reviewer cannot reach a config mutation). Confinement: a {reviews:write} agent can neither borrow merge authority nor reach its own grant",
      "verify": "cargo test -p but-api confinement_reviewer_denied_other_merge_and_self_grant",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api forge seam + config-mutate guard function + real git + real but-db",
        "negative_control": {
          "would_fail_if": [
            "the {reviews:write} agent's merge of another handle's change proceeds because the merge gate is a no-op or reads authority from an agent-supplied claim instead of committed config",
            "AUTHZ-006's enforce_administration_write_gate returns Ok for the {reviews:write} reviewer so it could reach a config mutation (the guard fails to deny before any write)",
            "the acting principal is taken from a caller argument / claim rather than resolved from BUT_AGENT_HANDLE against committed config",
            "either denial returns a code other than perm.denied (the gate is disconnected/stubbed)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "confined_repo",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=reviewer: invoke the governed merge action on the open review whose change another handle authored (reviewer holds reviews:write but NOT merge) (cite T-AUTHZ-023a)"
              ]
            },
            "end_state": {
              "must_observe": [
                "`error.code == \"perm.denied\"`",
                "the `message` names the missing `\"merge\"` authority",
                "process exits `1`",
                "the review is NOT merged (trunk/main HEAD sha `==` the seeded base sha)"
              ],
              "must_not_observe": [
                "merge proceeded",
                "exit `0`",
                "reviewer borrowed merge authority",
                "trunk/main HEAD sha advanced"
              ]
            }
          },
          {
            "start_ref": "confined_repo",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=reviewer: call AUTHZ-006's enforce_administration_write_gate(repo, \"refs/heads/main\") (the path a self-grant config edit would take — reviewer holds reviews:write but NOT administration:write); the guard denies BEFORE any write (cite T-AUTHZ-023b)"
              ]
            },
            "end_state": {
              "must_observe": [
                "the admin-write guard returns `Err(Denial)` with `error.code == \"perm.denied\"`",
                "the `message` names the missing `\"administration:write\"` authority required to change governed config",
                "the guard denies BEFORE any write — reviewer never reaches a config mutation (there is no persisted write path yet; the consumer is Sprint 05)"
              ],
              "must_not_observe": [
                "the guard returns `Ok` for the {reviews:write} reviewer (`0` denials where `1` is required)",
                "reviewer's grant widened to include `merge`",
                "the self-grant accepted / a config mutation reached (the guard must deny BEFORE any write)"
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
      "description": "GIVEN confined_repo WHEN a governed action is invoked while BUT_AGENT_HANDLE=reviewer AND an in-band identity-override `--as maint` is attempted (no --as/impersonation flag exists on the CLI today, grep-confirmed) THEN the resolved acting principal id == \"reviewer\" (the BUT_AGENT_HANDLE value) — proven because the governed denial message NAMES \"reviewer\" as the acting principal — AND the `--as maint` override is NOT honored (clap rejects the unknown argument, exit != 0); there is NO honored in-band identity-override path",
      "verify": "cargo test -p but confinement_no_inband_identity_override",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but CLI + real but-api + real git",
        "negative_control": {
          "would_fail_if": [
            "the governed denial names a default/other id rather than \"reviewer\" — proving the resolver IGNORED BUT_AGENT_HANDLE and bound a hardcoded/default principal (the disconnected-resolver hole the clap-rejects check alone would NOT catch)",
            "a CLI flag (e.g. --as <other>) is added and HONORED so an agent can act as a different principal — the override is accepted and the action runs as the borrowed handle",
            "the acting principal is read from any argument other than BUT_AGENT_HANDLE",
            "the resolved principal is a default/anonymous identity rather than reviewer"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "confined_repo",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=reviewer: invoke a governed action and assert the governed denial message NAMES \"reviewer\" as the acting principal (the load-bearing proof that resolve_principal consumed BUT_AGENT_HANDLE)",
                "attempt to pass an in-band identity override `--as maint` (no such honored flag exists today): clap rejects the unknown flag (exit code != 0) and the action does NOT run as `maint` (cite C3, T-AUTHZ-019, T-AUTHZ-026)"
              ]
            },
            "end_state": {
              "must_observe": [
                "the governed denial message NAMES the acting principal `\"reviewer\"` (resolved id == the `BUT_AGENT_HANDLE` value — resolve_principal consumed the env var)",
                "the `--as maint` override is NOT honored: the process exits non-zero (clap unknown-argument error) and the action does not run as `maint`",
                "no governed action executes under the borrowed `maint` identity"
              ],
              "must_not_observe": [
                "the denial names a default/other id instead of `\"reviewer\"` (the resolver ignored BUT_AGENT_HANDLE)",
                "the action runs as `maint` via an in-band override",
                "exit `0` for the override attempt (`0` rejection where a non-zero clap error is required)",
                "a default/anonymous principal bound instead of `reviewer`"
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
      "description": "GIVEN confined_repo WHEN a governed merge action is invoked THEN the effective authority is read ONLY from the committed config (via but_authz::authorize over the loaded GovConfig) and NEVER from an agent-supplied authority claim — a {reviews:write} agent is denied merge perm.denied AND the deny message names the env-derived principal id \"reviewer\" (weak structural evidence that resolve_principal, not a hand-built Principal::new, was the source; M5 accepted limitation)",
      "verify": "cargo test -p but-api confinement_authority_from_config_not_claim",
      "maps_to_ac": null,
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api + committed GovConfig + real git",
        "negative_control": {
          "would_fail_if": [
            "authority is read from an agent-supplied claim/argument that widens reviewer's effective set to include merge",
            "the enforcement path exposes a parameter by which a caller injects an AuthoritySet, overriding the committed config",
            "reviewer is allowed merge because a claim was honored rather than the committed config consulted (a stubbed authorize that ignores the loaded GovConfig)",
            "the deny names an id other than the env-derived \"reviewer\" (evidence the source was a hand-built Principal::new with caller-supplied authorities, not resolve_principal — the M5 accepted-limitation hole)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "confined_repo",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=reviewer: invoke the governed merge action; the test confirms there is NO parameter/field by which a caller injects an authority claim, that authorize evaluates reviewer's effective set solely from the loaded committed GovConfig, and that the deny names the env-derived id \"reviewer\" (cite T-AUTHZ-020, M5)"
              ]
            },
            "end_state": {
              "must_observe": [
                "`error.code == \"perm.denied\"` naming the missing `\"merge\"` authority — reviewer's effective set is sourced only from committed config",
                "the deny message names the env-derived principal id `\"reviewer\"` (evidence resolve_principal, not a hand-built Principal, was the source)",
                "the merge does NOT proceed (trunk/main HEAD sha `==` base)"
              ],
              "must_not_observe": [
                "`Ok` / exit `0` from an honored authority claim (`0` denials where `1` is required)",
                "reviewer granted merge via a caller-supplied claim",
                "the deny naming an id other than `\"reviewer\"`",
                "merge proceeded (trunk advanced from base)"
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
      "description": "GIVEN the confinement is honest about its accepted leak WHEN the suite is inspected THEN there is NO test asserting that a re-exporting process (an agent that re-exports BUT_AGENT_HANDLE to a different value in a child process) is BLOCKED — the env re-export leak is documented and accepted, never claimed as prevented. The grep is terminology-based (best-effort); a reviewer manually inspects any std::process::Command / child-env test for a smuggled re-export-blocked assertion [build-gate/honesty]",
      "verify": "! grep -rEn 'reexport|re_export|re-export|child.*BUT_AGENT_HANDLE.*block|prevents?.*re.?export' crates/but-api/tests crates/but/tests",
      "maps_to_ac": null,
      "scenario": {
        "tier": "holdout",
        "test_tier": "unit",
        "unit_test_justified": "Honesty/anti-false-guarantee structural invariant verified by source grep with zero runtime I/O: it asserts the ABSENCE of a test that would encode a false guarantee (that env re-export is prevented). The confinement's real behavioral guarantees (no in-band override honored, resolved id == env value, authority from config not claim) are proven by the AC-1..3 integration cases; this build-gate only enforces that the accepted-leak is not falsely tested as blocked. The grep is terminology-based and best-effort, complemented by manual reviewer inspection of any child-process env-manipulation test (M7).",
        "verification_service": "source grep (build-gate, no runtime I/O) + manual reviewer inspection",
        "negative_control": {
          "would_fail_if": [
            "a test is added asserting a re-exporting child process is blocked from acting as a different handle (encoding the false guarantee the PRD explicitly accepts as a leak)",
            "the suite claims env-var re-export confinement is enforced",
            "a test asserts BUT_AGENT_HANDLE cannot be overridden by a child process",
            "the accepted-leak is silently converted into a tested guarantee",
            "the honesty grep is a no-op / disconnected from the test files so a false-guarantee test slips in unflagged"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "confined_repo",
            "action": {
              "actor": "ci",
              "steps": [
                "grep the AUTHZ-005 test files for any assertion that env re-export / a re-exporting child process is BLOCKED; a reviewer additionally inspects any std::process::Command / child-env test in confinement.rs (cite T-AUTHZ-026 confinement-honesty, M7)"
              ]
            },
            "end_state": {
              "must_observe": [
                "`! grep -rEn 'reexport|re_export|re-export|child.*BUT_AGENT_HANDLE.*block|prevents?.*re.?export' crates/but-api/tests crates/but/tests` returns `0` matches (no false-guarantee test exists)",
                "the accepted env-re-export leak is documented in the task `notes`, with `0` tests asserting it is prevented",
                "the manual reviewer inspection of any `std::process::Command` / child-env test finds `0` smuggled re-export-blocked assertions (the terminology grep is best-effort, M7)"
              ],
              "must_not_observe": [
                "a test asserting env re-export is blocked (`1+` grep match where `0` is required)",
                "a non-empty grep result naming a re-export-blocked assertion",
                "a claim that re-export confinement is enforced",
                "the accepted-leak converted into a tested guarantee"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "{reviews:write} agent denied BOTH an other-handle merge AND a self-grant-merge config edit (the admin-write guard denies before any write), both perm.denied (T-AUTHZ-023)",
      "verify": "cargo test -p but-api confinement_reviewer_denied_other_merge_and_self_grant",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "resolved acting principal id == BUT_AGENT_HANDLE value (denial names \"reviewer\"); no honored in-band --as override; override does not run as the borrowed handle (C3, T-AUTHZ-018, T-AUTHZ-019)",
      "verify": "cargo test -p but confinement_no_inband_identity_override",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "authority read only from committed config, never from an agent-supplied claim; deny names the env-derived id (T-AUTHZ-020, M5)",
      "verify": "cargo test -p but-api confinement_authority_from_config_not_claim",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "honesty: NO test asserts the accepted env-re-export leak is blocked; terminology grep + manual inspection (T-AUTHZ-026, M7)",
      "verify": "! grep -rEn 'reexport|re_export|re-export|child.*BUT_AGENT_HANDLE.*block|prevents?.*re.?export' crates/but-api/tests crates/but/tests",
      "maps_to_ac": "AC-4"
    }
  ]
}
-->
</details>
