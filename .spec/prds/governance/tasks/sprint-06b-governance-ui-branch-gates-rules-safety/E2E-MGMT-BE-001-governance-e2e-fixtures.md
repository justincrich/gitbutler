# E2E-MGMT-BE-001: Governed-repo E2E fixtures + two real identities (admin / non-admin)

## What this does

Bash seed scripts under `e2e/playwright/scripts/` that commit a real `.gitbutler/permissions.toml` and `.gitbutler/gates.toml` onto the **workspace target ref** (refs/heads/master, via real `git commit` — not working-tree-only), seeding two genuinely-different governed identities: `admin` (administration:write + merge) and `dev` (contents:write ONLY). Plus `e2e/playwright/src/governance.ts` exporting `ADMIN_HANDLE` / `NONADMIN_HANDLE` and the seeded principal/group/gate constants the capstone Playwright suite consumes. Identity is selected per Playwright test via `gitbutlerOptions.env.BUT_AGENT_HANDLE`, which `startGitButler` forwards into the spawned `but-server`.

## Why

Sprint 06b · PRD UC-MGMT-06 · capability CAP-AUTHZ-01. The capstone (E2E-MGMT-UI-001) drives the real web-target governance UI against a real `but-server`; for steps 3/4 to be **real** (read-only as non-admin; non-admin self-grant denied) the repo must be seeded with two genuinely-different governed principals committed to the ref the authz loader reads. This is the fixtures + identity foundation the rest of the capstone stands on.

## How to verify

PRIMARY **AC-1** (integration) — run `bash e2e/playwright/scripts/seed-governance.sh <fresh-workdir>` then `git -C <workdir> cat-file -p master:.gitbutler/permissions.toml` shows admin=administration:write and dev=contents:write-only; `git cat-file -p master:.gitbutler/gates.toml` shows ONLY `[[branch]] name="master" protected=true`; `git status --porcelain` is clean of `.gitbutler/*.toml`.

## Scope

- `e2e/playwright/scripts/seed-governance.sh` (NEW — bash seed committing both `.gitbutler/*.toml` to the target ref)
- `e2e/playwright/src/governance.ts` (NEW — exports `ADMIN_HANDLE`/`NONADMIN_HANDLE` + seeded constants)
- Rust integration test exercising the committed config through real `but_authz` (test-only; e.g. `crates/but-authz/tests/*`)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: E2E-MGMT-BE-001 — Governed-repo E2E fixtures + two real identities
================================================================================

TASK_TYPE:   FEATURE
STATUS:      Backlog
PRIORITY:    P1
EFFORT:      M  (120 min)
AGENT:       rust-implementer
PROPOSED-BY: rust-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-MGMT-06, 11-e2e-testing-criteria.md#T-MGMT-032, #T-MGMT-041
CAPABILITIES: CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  bash e2e/playwright/scripts/seed-governance.sh <workdir> && git -C <workdir> cat-file -p master:.gitbutler/permissions.toml
  check: cargo test -p but-authz <seed-verification-test>
  lint:  shellcheck e2e/playwright/scripts/governance-*.sh

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
After running the seed against a fresh harness repo: the workspace target ref (refs/heads/master)
tree contains a committed .gitbutler/permissions.toml (admin=administration:write+merge,
dev=contents:write only, groups code-reviewers/maintainers, per-principal rules for both) and a
committed .gitbutler/gates.toml ([[branch]] name="master" protected=true — and NOTHING else); the
working tree is clean; the live but_authz loader parses the committed config without error;
authorize(AdministrationWrite) is Ok for ADMIN_HANDLE and a perm.denied Denial naming
administration:write for NONADMIN_HANDLE; governance.ts exports both handles + seeded constants.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] Commit .gitbutler/permissions.toml AND .gitbutler/gates.toml onto the WORKSPACE TARGET REF
  (refs/heads/master) via real git so the gix loader at crates/but-authz/src/config.rs:285-310 reads
  them from the committed tree. Working-tree-only files are invisible to authorize().
- [MUST] Seed gates.toml as EXACTLY `[[branch]]` with name="master" and protected=true — nothing else.
  GatesWire/BranchWire (config.rs:447-459) are #[serde(deny_unknown_fields)] and model only
  { branch: [{name, protected}] }. Any extra key makes the whole config fail to load.
- [MUST] Seed two genuinely-different principals: admin = administration:write + merge; dev =
  contents:write ONLY (dev MUST NOT hold administration:write, directly or via a group).
- [MUST] Seed code-reviewers and maintainers as real permissions.toml [[group]] entries (with members),
  and seed >=1 per-principal RULE for ADMIN and one for NONADMIN so the step-2 Rules tab has real,
  distinct backing data.
- [MUST] Leave the working tree CLEAN after seeding (git status --porcelain shows no uncommitted
  .gitbutler/*.toml) — else the worktree-first gates read (governance.rs:1160-1166) serves a stray file
  and fakes branch_gates_read.
- [MUST] Reuse the harness verbatim: startGitButler / gitbutlerOptions.env forwarding — consume-only;
  do NOT modify setup.ts/test.ts.
- [NEVER] NEVER seed via working-tree-only files without committing to the target ref.
- [NEVER] NEVER use any *_as_fleet_owner path to seed or verify.
- [NEVER] NEVER grant the dev/non-admin principal administration:write.
- [NEVER] NEVER add min_approvals, require_distinct_from_author, require_approval_from_group, or a
  [[gate]] table to gates.toml (the live loader rejects unknown fields).
- [NEVER] NEVER use std::env::temp_dir().join(format!(…)) for scratch repos; use the harness workdir.
- [STRICTLY] AC-1 MUST assert BOTH (a) configs committed to the target ref (git cat-file -p succeeds)
  AND (b) the working tree has no uncommitted .gitbutler/*.toml after seed.
- [STRICTLY] The two identities MUST be exercised through resolve_principal_from_env + authorize over
  the COMMITTED config — not asserted by inspecting the TOML text alone.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: seed commits both TOMLs (minimal locked schema) to the target ref; working tree clean
- [ ] AC-2: committed config loads; authorize(admin, AdministrationWrite) == Ok
- [ ] AC-3: authorize(dev, AdministrationWrite) == Denial{code: perm.denied} naming administration:write
- [ ] AC-4: gitbutlerOptions.env.BUT_AGENT_HANDLE reaches the spawned server; governance.ts exports the two handles + constants

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each with a real start→end scenario; integration tier)
--------------------------------------------------------------------------------
AC-1 [PRIMARY] — Seed commits the governed config to the target ref + clean tree
  GIVEN a fresh harness repo with no governance config
  WHEN  e2e/playwright/scripts/seed-governance.sh runs against it
  THEN  git cat-file -p master:.gitbutler/permissions.toml exits 0 and shows [[principal]] admin with
        administration:write+merge AND [[principal]] dev with contents:write and NO administration:write;
        [[group]] code-reviewers + maintainers (>=1 member each); >=1 rule per principal;
        git cat-file -p master:.gitbutler/gates.toml shows [[branch]] name="master" protected=true ONLY;
        git status --porcelain shows NO uncommitted .gitbutler/*.toml.
  VERIFY bash e2e/playwright/scripts/seed-governance.sh <wd> && git -C <wd> cat-file -p master:.gitbutler/permissions.toml | grep -q 'administration:write' && git -C <wd> status --porcelain | (! grep -q '.gitbutler/')
  TEST_TIER integration   VERIFICATION_SERVICE real git (cat-file against committed target-ref tree)
  NEGATIVE CONTROL would fail if: TOMLs written to working tree only (cat-file fails); dev seeded with
    administration:write (steps 3/4 un-denyable); gates carries min_approvals/[[gate]] (deny_unknown_fields
    load error); uncommitted gates.toml left in working tree (status not clean).

AC-2 — Admin authorizes administration:write over the committed config
  GIVEN the seed committed (AC-1)
  WHEN  resolve_principal_from_env(ADMIN_HANDLE) loads from the committed GovConfig and authorize(AdministrationWrite) runs
  THEN  the admin principal resolves and authorize(admin, AdministrationWrite, cfg) == Ok(())
  VERIFY cargo test -p but-authz governance_e2e_admin_authorizes
  TEST_TIER integration   VERIFICATION_SERVICE real but_authz loader + authorize over the committed ref
  NEGATIVE CONTROL would fail if: config seeded to working tree only (loader Err); gates unknown field (load Err before authorize).

AC-3 — Non-admin (dev) is denied administration:write
  GIVEN the seed committed
  WHEN  resolve_principal_from_env(NONADMIN_HANDLE) loads dev and authorize(AdministrationWrite) runs
  THEN  authorize(dev, AdministrationWrite, cfg) == Err(Denial{code: perm.denied}) naming administration:write
  VERIFY cargo test -p but-authz governance_e2e_dev_denied
  TEST_TIER integration   VERIFICATION_SERVICE real but_authz authorize over the committed config
  NEGATIVE CONTROL would fail if: dev mis-seeded with administration:write (Ok instead of Denial); fleet-owner path used (gate skipped).

AC-4 — Env forwarding + module exports
  GIVEN the harness fixture wiring
  WHEN  a spec sets test.use({ gitbutlerOptions: { env: { BUT_AGENT_HANDLE } } }) and startGitButler spawns the server
  THEN  the spawned but-server resolves the principal from BUT_AGENT_HANDLE (no "BUT_AGENT_HANDLE is required" error);
        governance.ts exports ADMIN_HANDLE and NONADMIN_HANDLE (distinct) matching the seeded principal ids + group/gate constants
  VERIFY node -e "import('./e2e/playwright/src/governance.ts').then(m=>{if(!m.ADMIN_HANDLE||!m.NONADMIN_HANDLE||m.ADMIN_HANDLE===m.NONADMIN_HANDLE)process.exit(1)})"
  TEST_TIER integration   VERIFICATION_SERVICE real startGitButler spawn of the repo-built but-server
  NEGATIVE CONTROL would fail if: env not forwarded (server errors "BUT_AGENT_HANDLE is required"); governance.ts omits constants; ADMIN_HANDLE === NONADMIN_HANDLE.

--------------------------------------------------------------------------------
GUARDRAILS
--------------------------------------------------------------------------------
WRITE-ALLOWED:
  - e2e/playwright/scripts/** (new seed scripts: seed-governance.sh + optional assert helpers)
  - e2e/playwright/src/governance.ts (new module of typed exports/constants)
  - Rust integration test files exercising the committed config through but_authz (test-only)
WRITE-PROHIBITED:
  - e2e/playwright/src/setup.ts, test.ts (consume-only harness)
  - crates/but-authz/src/** production code (loader/authorize are contracts to satisfy)
  - crates/but-api/** production code
  - apps/desktop/** and any UI code

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
- crates/but-authz/src/config.rs:267-310 — loader reads blobs from the TARGET REF tree via gix (committed-only)
- crates/but-authz/src/config.rs:447-459 — GatesWire/BranchWire deny_unknown_fields (only {name, protected})
- crates/but-authz/src/config.rs:412-445 — PermissionsWire/PrincipalWire/GroupWire legal shape
- crates/but-authz/src/config.rs:337-377 — effective-authority fold (direct + groups)
- crates/but-authz/src/authorize.rs:5,24,71,100,149 — BUT_AGENT_HANDLE, authorize, resolve_principal_from_env, "required" Denial
- crates/but-api/src/legacy/governance.rs:1160-1166 — load_gates_for_write reads working tree FIRST (why AC-1 asserts clean tree)
- e2e/playwright/src/setup.ts:352-364 — startGitButler forwards env into the spawned but-server
- e2e/playwright/src/test.ts:34-52 — gitbutlerOptions is a per-test option

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- bash e2e/playwright/scripts/seed-governance.sh <fresh-workdir>      → exit 0
- git -C <wd> cat-file -p master:.gitbutler/permissions.toml          → admin=admin:write+merge, dev=contents:write only, groups + rules
- git -C <wd> cat-file -p master:.gitbutler/gates.toml                → [[branch]] name="master" protected=true ONLY
- git -C <wd> status --porcelain                                      → EMPTY for .gitbutler/*.toml
- cargo test -p but-authz <seed-verification>                         → admin Ok, dev perm.denied
- cargo fmt && cargo clippy -p but-authz --all-targets               → clean

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
depends_on: MGMT-BE-003, MGMT-BE-004, MGMT-IPC-001, MGMT-IPC-003
blocks:     E2E-MGMT-BE-002, E2E-MGMT-UI-001

--------------------------------------------------------------------------------
NOTES
--------------------------------------------------------------------------------
- UPSTREAM ADVISORY U2 (escalation, NOT resolved here): SPRINT.md MGMT-BE-004 Coverage Note (~lines
  145-146) and its "round-trip the full gate-field set" claim assert a gates schema (min_approvals,
  require_distinct_from_author, require_approval_from_group, [[gate]]) the LIVE loader does NOT support
  (config.rs:447-459, deny_unknown_fields, only {branch:[{name,protected}]}). This capstone seeds the
  minimal supported schema only. MGMT-BE-004 scope is possibly over-claimed vs live code — flag for the
  sprint owner. Do NOT edit the locked SPRINT.md.
- Identity model is process-level: each spawned but-server resolves ONE principal from BUT_AGENT_HANDLE.
  No per-request identity. The capstone groups steps by identity into separate test() blocks.
- Step-4 mapping (downstream): a real perm.denied requires the ACTING handle to LACK administration:write
  — step 4 acts as NONADMIN_HANDLE (see U1 in BE-002/UI notes).
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "requirements": [
    { "id": "AC-1", "type": "acceptance_criterion", "description": "Seed commits permissions.toml (admin=administration:write+merge, dev=contents:write only, groups code-reviewers/maintainers, per-principal rules) and gates.toml ([[branch]] master protected=true ONLY) to the workspace target ref; working tree clean", "verify": "bash e2e/playwright/scripts/seed-governance.sh <wd> && git -C <wd> cat-file -p master:.gitbutler/permissions.toml | grep -q 'administration:write' && git -C <wd> status --porcelain | (! grep -q '.gitbutler/')", "test_tier": "integration", "primary": true },
    { "id": "AC-2", "type": "acceptance_criterion", "description": "Committed config loads via but_authz and authorize(admin, AdministrationWrite) == Ok", "verify": "cargo test -p but-authz governance_e2e_admin_authorizes", "test_tier": "integration" },
    { "id": "AC-3", "type": "acceptance_criterion", "description": "authorize(dev, AdministrationWrite) == Denial{code:perm.denied} naming administration:write", "verify": "cargo test -p but-authz governance_e2e_dev_denied", "test_tier": "integration" },
    { "id": "AC-4", "type": "acceptance_criterion", "description": "gitbutlerOptions.env.BUT_AGENT_HANDLE reaches the spawned but-server; governance.ts exports distinct ADMIN_HANDLE/NONADMIN_HANDLE + seeded constants", "verify": "node -e \"import('./e2e/playwright/src/governance.ts').then(m=>{if(!m.ADMIN_HANDLE||!m.NONADMIN_HANDLE||m.ADMIN_HANDLE===m.NONADMIN_HANDLE)process.exit(1)})\"", "test_tier": "integration" },
    { "id": "TC-1", "type": "test_criterion", "description": "git cat-file confirms both TOMLs committed to the target ref with the locked minimal schema AND git status --porcelain shows the working tree clean of .gitbutler/*.toml", "verify": "git -C <wd> cat-file -p master:.gitbutler/gates.toml | grep -qx '.*protected = true' && git -C <wd> status --porcelain | (! grep -q '.gitbutler/')", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "but_authz loads the committed config without error and authorize(admin, AdministrationWrite) == Ok(())", "verify": "cargo test -p but-authz governance_e2e_admin_authorizes", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "authorize(dev, AdministrationWrite) == Err(Denial{code:perm.denied}) naming administration:write", "verify": "cargo test -p but-authz governance_e2e_dev_denied", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "A harness-spawned server with env BUT_AGENT_HANDLE resolves the principal; governance.ts exports the two distinct handles + seeded constants", "verify": "node -e \"import('./e2e/playwright/src/governance.ts').then(m=>{if(!m.ADMIN_HANDLE||!m.NONADMIN_HANDLE)process.exit(1)})\"", "maps_to_ac": "AC-4" }
  ]
}
-->
