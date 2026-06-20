# AUTHZ-006: `administration:write` guard on the config-mutating path — reusable enforce_administration_write_gate keying off Authority::AdministrationWrite

## What this does

Adds the `administration:write` authority primitive on the config-mutating path: a reusable `enforce_administration_write_gate(repo, target_ref)` in `crates/but-api/src/legacy/config_mutate.rs` that resolves the acting principal from `BUT_AGENT_HANDLE`, loads the target-ref governance config, and requires `Authority::AdministrationWrite` (the variant ALREADY exists in `but-authz`, authority.rs:35) before a governed config mutation proceeds. `Authority::AdministrationWrite` exists, so this WIRES the existing variant onto the config-mutate seam via the FULLY-QUALIFIED `but_authz::authorize(&p, Authority::AdministrationWrite, &cfg)` form — it does NOT add a new variant. There is NO `but perm` / `but group` CLI verb today and NO but-api config-write entrypoint (grep-confirmed; the persisted-write CONSUMER lands in Sprint 05), so this task provides the reusable AUTHORIZATION-DECISION guard plus an integration test that exercises the guard FUNCTION directly (real `load_governance_config` + real `authorize` + `BUT_AGENT_HANDLE`). It does NOT fabricate a CLI verb and does NOT claim a persisted-mutation block.

## Why

Sprint 02 · PRD UC-AUTHZ-03 · capabilities CAP-AUTHZ-01, CAP-CONFIG-01. `administration:write` is required to change governed config (`but perm`/`but group` in Sprint 05, any `.gitbutler/{permissions,gates}.toml` touch) else `perm.denied`; the config-change decision keys off `Authority`, never a role (grep-asserted by AUTHZ-008); admin-write is checked at the target ref against the COMMITTED blob (the working tree cannot grant authority).

## How to verify

PRIMARY **AC-1** — `cargo test -p but-api admin_write_guard_denies_non_admin_allows_admin` (integration: the guard FUNCTION over real but-authz + real git — dev denied, admin Ok, working-tree-grant still denied). Full gate set in the spec below.

## Scope

- `crates/but-api/src/legacy/config_mutate.rs` (NEW) — the reusable `enforce_administration_write_gate(repo, target_ref) -> Result<(), anyhow::Error>`: resolve principal from BUT_AGENT_HANDLE, load target-ref config, `but_authz::authorize(&p, Authority::AdministrationWrite, &cfg)` (FULLY-QUALIFIED). OWNS the admin-write guard + its classify path (Denial -> perm.denied, ConfigError -> config.invalid)
- `crates/but-api/src/legacy/mod.rs` (MODIFY) — register the `config_mutate` module
- `crates/but-api/tests/admin_write_guard.rs` (NEW) — integration over the guard FUNCTION: a non-admin (dev) denied perm.denied; an admin Ok; a working-tree permissions.toml self-grant still denied; a malformed target-ref config -> config.invalid

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: AUTHZ-006 - administration:write guard on the config-mutating path: reusable enforce_administration_write_gate keying off Authority::AdministrationWrite
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     M  (120 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-AUTHZ-03
CAPABILITIES: CAP-AUTHZ-01, CAP-CONFIG-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-api admin_write_guard
  check: cargo check -p but-api --all-targets
  lint:  cargo clippy -p but-api --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Integration tests are green against real but-api + real but-authz + real git, exercising the guard FUNCTION enforce_administration_write_gate(repo, target_ref) directly: a non-admin principal (BUT_AGENT_HANDLE=dev holding only contents:write) is denied perm.denied naming administration:write (Err(Denial)); an admin (BUT_AGENT_HANDLE=admin holding administration:write) is permitted (Ok(())); a self-escalation attempt that writes administration:write for the test principal into the WORKING-TREE permissions.toml is STILL denied (the guard reads the committed TARGET-REF blob, not the working tree — working-tree config cannot grant authority); a malformed target-ref config fails the guard closed config.invalid (never perm.denied, never a skip). The guard keys off Authority::AdministrationWrite via the FULLY-QUALIFIED but_authz::authorize form — NOT a role name, NOT a human-vs-AI label — and does NOT overload the GitButler Permission/RepoExclusive lock (grep-asserted, AUTHZ-008 T-AUTHZ-022). AC-1 proves the AUTHORIZATION DECISION; the persisted config-write CONSUMER is Sprint 05 (no `but perm`/`but group` verb is fabricated). The dev-denied case is kept in the SAME test as admin-passes so a no-op-always-Ok stub fails the dev-denied half.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST wire the EXISTING Authority::AdministrationWrite variant (already in crates/but-authz/src/authority.rs:35, token "administration:write" at :82) onto the config-mutate seam via the FULLY-QUALIFIED `but_authz::authorize(&principal, but_authz::Authority::AdministrationWrite, &cfg)` form. DO NOT add a new Authority variant — it exists; confirm by reading authority.rs and WIRE it. The fully-qualified form is REQUIRED so AUTHZ-008's AUTHORITY_POSITIVE_PATTERN (but_authz::authorize|Authority::contains|but_authz::Authority — NO bare `Authority::` branch) bites on this file.
- [MUST] MUST provide the guard as a REUSABLE function enforce_administration_write_gate(repo, target_ref) in crates/but-api/src/legacy/config_mutate.rs that Sprint 05's but perm / but group verbs will consume — resolve the acting principal from BUT_AGENT_HANDLE (resolve_principal_from_env), load the TARGET-REF governance config (load_governance_config), then but_authz::authorize(administration:write). Mirror the commit gate's enforce_commit_gate_for_target shape (crates/but-api/src/commit/gate.rs:57).
- [MUST] MUST scope AC-1 HONESTLY to the AUTHORIZATION DECISION: there is NO but perm / but group verb today (grep-confirmed) and NO but-api entrypoint mutates .gitbutler/{permissions,gates}.toml (grep-confirmed: 0 matches). The guard is a NEW reusable function with its persisted-write CONSUMER in Sprint 05. AC-1 therefore exercises the guard FUNCTION directly (call enforce_administration_write_gate and assert Err(Denial)/Ok) — it does NOT claim "execution reaches the config-mutate body" and does NOT claim "a real governed write path is protected." DO NOT fabricate a but perm / but group CLI verb. DO NOT assert a persisted-mutation block.
- [MUST] MUST make "admin passes" FALSIFIABLE: keep the dev-denied case in the SAME test as the admin-Ok case so a no-op-always-Ok guard (which would pass admin) FAILS the dev-denied half. A guard that returns Ok for everyone passes admin but must fail dev-denied — name this explicitly in the AC-1 negative_control.
- [MUST] MUST read the authority + config ONLY from the TARGET-REF committed blob via load_governance_config(repo, target_ref) — never the working tree (CAP-CONFIG-01; the self-escalation defense). Prove it: a sub-case writes administration:write for the test principal into the WORKING-TREE permissions.toml and the guard STILL denies (working-tree config cannot grant authority).
- [MUST] MUST classify deterministically: a malformed target-ref config -> config.invalid (from ConfigError::code()); an unknown/no-handle/non-admin principal -> perm.denied. The two codes are never blurred (mirror the commit gate's classify_error, crates/but-api/src/commit/gate.rs:83, which returns CommitGateError{code,message} — remediation_hint is dropped, MGMT-IPC-002; assert {code,message} only).
- [NEVER] NEVER key the config-change decision off a ROLE name or a human-vs-AI label — it keys off Authority::AdministrationWrite ALONE (T-AUTHZ-022 / T-LOOP-005 / T-AUTHZ-016 grep-asserted by AUTHZ-008). NEVER branch on the handle string.
- [NEVER] NEVER overload GitButler's repo-access Permission/RepoExclusive lock as the authorization carrier — authorization is the orthogonal Authority axis (02-system-components.md; RULES.md lock discipline). The guard carries its code as a gate-owned &'static str (Denial::PERM_DENIED_CODE), NOT a but-error::Code variant unless the desktop frontend consumes it.
- [STRICTLY] STRICTLY scope to the admin-write guard primitive — do NOT add the Sprint-05 but perm / but group verbs, and do NOT gate the merge path (that is AUTHZ-004). The guard is consumed downstream; this task ships the guard + its denial/allow/working-tree/malformed integration proof exercising the FUNCTION.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: enforce_administration_write_gate denies a non-admin (dev) perm.denied naming administration:write AND permits an admin (Ok) AND still denies a working-tree-grant self-escalation — all in one falsifiable test
- [ ] AC-2: the guard keys off Authority::AdministrationWrite via the fully-qualified but_authz::authorize form, no role-name/human-vs-AI label, no Permission-lock overload (build-gate)
- [ ] AC-3: malformed target-ref config -> config.invalid (never perm.denied, never a skip)
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: enforce_administration_write_gate denies non-admin, permits admin, and rejects a working-tree self-grant [PRIMARY]
  GIVEN: fixture `admin_write_repo` (committed target-ref permissions.toml: dev=[contents:write]; admin=[administration:write,merge]; main protected)
  WHEN:  enforce_administration_write_gate(repo, "refs/heads/main") is called with BUT_AGENT_HANDLE=dev, then with BUT_AGENT_HANDLE=admin, then (self-escalation sub-case) with BUT_AGENT_HANDLE=dev AFTER writing administration:write for dev into the WORKING-TREE .gitbutler/permissions.toml
  THEN:  dev -> Err(Denial) error.code=="perm.denied" naming administration:write; admin -> Ok(()) (no Denial); the working-tree-grant dev -> STILL Err(Denial) perm.denied (the committed target-ref blob is unchanged, so the guard does not see the working-tree grant)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api config-mutate guard FUNCTION + real but-authz + real git
  VERIFY: cargo test -p but-api admin_write_guard_denies_non_admin_allows_admin
  SCENARIO: NEGATIVE_CONTROL would fail if the guard returns Ok for everyone (passes admin but FAILS the dev-denied half — a no-op-always-Ok stub); the guard keys off a role name not Authority::AdministrationWrite; the guard reads the working tree so the working-tree self-grant flips dev to Ok; the admin holder is wrongly denied; either denial returns a code other than perm.denied.

AC-2: The guard keys off Authority::AdministrationWrite (fully-qualified), no role-name/label, no Permission overload [build-gate]
  GIVEN: the source tree after this task
  WHEN:  the build-gate greps run over config_mutate.rs
  THEN:  the admin-write decision keys off the fully-qualified but_authz::authorize / Authority::AdministrationWrite (1+ AUTHORITY match), no role-name/human-vs-AI label appears (0), the Permission/RepoExclusive lock is NOT the carrier (0)
  TEST_TIER: unit (build-gate)   VERIFICATION_SERVICE: source grep (no runtime I/O)   UNIT_TEST_JUSTIFIED: structural keys-off-Authority invariant (T-AUTHZ-022) verified by grep with zero runtime I/O; the behavioral allow/deny is proven by AC-1 integration. A runtime test cannot assert the structural absence of a role-name branch or the presence of the fully-qualified authorize form.
  VERIFY: grep -rEn 'but_authz::authorize|Authority::AdministrationWrite' crates/but-api/src/legacy/config_mutate.rs && ! grep -rEin 'implementer|reviewer|maintainer|is_human|is_ai' crates/but-api/src/legacy/config_mutate.rs && ! grep -rEn 'write_permission\(|RepoExclusive' crates/but-api/src/legacy/config_mutate.rs
  SCENARIO: NEGATIVE_CONTROL would fail if the guard keys off a role name instead of Authority::AdministrationWrite; branches on is_human/is_ai; overloads the Permission/RepoExclusive lock; uses a bare un-prefixed authorize (so the AUTHORITY_POSITIVE grep scores 0); or the AUTHORITY_POSITIVE match is absent (the guard never consults Authority).

AC-3: Malformed target-ref config -> config.invalid (never perm.denied, never a skip)
  GIVEN: fixture `admin_write_malformed` (the target-ref permissions.toml is unparseable TOML)
  WHEN:  enforce_administration_write_gate is called (BUT_AGENT_HANDLE=admin)
  THEN:  the guard fails closed with a ConfigError whose code()=="config.invalid" (Err, no Ok, no perm.denied), never a skip
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api config-mutate guard FUNCTION + real but-authz + real git
  VERIFY: cargo test -p but-api admin_write_guard_malformed_config_invalid
  SCENARIO: NEGATIVE_CONTROL would fail if the malformed config is skipped (fail-open, Ok); misclassified as perm.denied; the guard reads the working-tree blob; the guard panics instead of returning the structured config.invalid contract.

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, happy_path): enforce_administration_write_gate denies dev perm.denied naming administration:write; permits admin (Ok); a working-tree administration:write self-grant for dev is STILL denied (committed-blob-only) (T-AUTHZ-021, M6)
    VERIFY: cargo test -p but-api admin_write_guard_denies_non_admin_allows_admin
- TC-2 (-> AC-2, structural): config-mutate guard keys off the fully-qualified but_authz::authorize / Authority::AdministrationWrite, no role-name/human-vs-AI label, no Permission-lock overload (T-AUTHZ-022, T-LOOP-005, T-AUTHZ-016)
    VERIFY: grep -rEn 'but_authz::authorize|Authority::AdministrationWrite' crates/but-api/src/legacy/config_mutate.rs
- TC-3 (-> AC-3, error): malformed target-ref config -> config.invalid, never perm.denied, never skipped
    VERIFY: cargo test -p but-api admin_write_guard_malformed_config_invalid

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01, CAP-CONFIG-01
provides: the reusable enforce_administration_write_gate(repo, target_ref) keying off the fully-qualified but_authz::authorize(Authority::AdministrationWrite) on the config-mutating path — the AUTHORIZATION-DECISION guard Sprint 05's but perm / but group verbs consume; perm.denied for a non-admin config-touch, config.invalid for a malformed target-ref config, committed-blob-only (working-tree self-grant denied); the keys-off-Authority structural property
consumes: but_authz::Authority::AdministrationWrite (AUTHZ-001 — existing variant); but_authz::authorize + resolve_principal from BUT_AGENT_HANDLE + Denial (AUTHZ-003); but_authz::load_governance_config + GovConfig + ConfigError::code() (AUTHZ-002); the commit gate's enforce_commit_gate_for_target + classify_error shape returning CommitGateError{code,message} (crates/but-api/src/commit/gate.rs)
boundary_contracts:
  - CAP-AUTHZ-01: a governed config mutation resolves BUT_AGENT_HANDLE->Principal and authorizes(administration:write) at the config-mutate seam, failing closed perm.denied on a non-admin / unknown / no-handle principal. The persisted-write consumer is Sprint 05; this task proves the decision.
  - CAP-CONFIG-01: admin-write is checked at the TARGET ref (committed blob — working-tree config cannot grant), and a malformed target-ref config -> config.invalid.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-api/src/legacy/config_mutate.rs (NEW) — the reusable enforce_administration_write_gate + its classify path (OWNS the admin-write guard); uses the fully-qualified but_authz::authorize form
  - crates/but-api/src/legacy/mod.rs (MODIFY) — register the config_mutate module
  - crates/but-api/tests/admin_write_guard.rs (NEW) — dev denied / admin Ok / working-tree self-grant denied / malformed -> config.invalid integration over the guard FUNCTION
writeProhibited:
  - crates/but-authz/** — CONSUME Authority::AdministrationWrite/authorize/resolve_principal/load_governance_config/Denial; do NOT modify the primitive (the variant ALREADY exists — do not re-add it)
  - crates/but/src/args/** and crates/but/src/lib.rs — the but perm / but group CLI verbs are Sprint 05; do NOT fabricate them here
  - crates/but-api/src/legacy/merge_gate.rs — the merge gate is AUTHZ-004
  - crates/but-error/src/lib.rs — carry the code as a gate-owned &'static str, no Code variants
  - any gitbutler-* crate beyond what the action boundary strictly requires (crates/AGENTS.md)
  - Any file not listed in write_allowed

--------------------------------------------------------------------------------
OUT OF SCOPE
--------------------------------------------------------------------------------
  - The persisted config-WRITE consumer (the actual mutation of .gitbutler/permissions.toml on a target ref) is Sprint 05's but perm / but group verbs — NOT fabricated here. This task ships the reusable AUTHORIZATION-DECISION guard the verbs will consume + a denial/allow/working-tree/malformed proof exercising the FUNCTION. AC-1 proves the decision, not a persisted-mutation block.
  - Adding a new Authority variant — administration:write ALREADY exists (authority.rs:35); this task WIRES it.
  - The merge path (AUTHZ-004) and the confinement composition (AUTHZ-005).
  - remediation_hint surfacing — CommitGateError drops it (MGMT-IPC-002, Sprint 06a). Assert {code,message} only.
  - The forgeable direct-config-write outside the governed seam — an accepted leak, NOT tested as blocked.

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-authz/src/authority.rs (32-36, 81-82, 94-109)
   Focus: CONFIRM — Authority::AdministrationWrite EXISTS (35), token "administration:write" (82), name() (94-109). The guard WIRES this existing variant via the fully-qualified but_authz::authorize form; it does NOT add it.
2. crates/but-api/src/commit/gate.rs (43-96)
   Focus: THE GUARD SHAPE TO MIRROR — enforce_commit_gate_for_target (57) loads governance config, resolves the principal (resolve_principal_from_env), authorizes; classify_error (83) downcasts Denial -> CommitGateError{code,message} (NO remediation_hint — dropped) and ConfigError -> config.invalid. enforce_administration_write_gate mirrors this exactly, requiring Authority::AdministrationWrite instead of ContentsWrite.
3. crates/but-authz/src/authorize.rs (24-93)
   Focus: CONSUME-ONLY — authorize(&p, Authority::AdministrationWrite, &cfg) -> Ok|Denial; resolve_principal_from_env reads BUT_AGENT_HANDLE; Denial::missing_permission names the missing authority.
4. crates/but-authz/src/config.rs (8-9, 24-29, 196-222)
   Focus: load_governance_config(repo, target_ref) -> Result<GovConfig, ConfigError>; ConfigError::code()=="config.invalid" (the malformed-config classification AC-3 asserts); PERMISSIONS_PATH=".gitbutler/permissions.toml" (8) is the COMMITTED tree blob the guard reads — the working tree is never consulted (the M6 self-escalation defense).
5. crates/but-api/src/legacy/mod.rs (full)
   Focus: how the legacy modules are registered — add `pub mod config_mutate;` alongside config/forge/settings.
6. crates/but-api/tests/commit_gate.rs (1-130)
   Focus: THE HARNESS — temp_env::with_var("BUT_AGENT_HANDLE", Some("dev")/Some("admin"), ...), but_ctx::Context::from_repo (or repo directly), governed_repo() via writable_scenario + invoke_bash, err.downcast_ref::<but_authz::Denial>() / ConfigError, assert denial.code == "perm.denied" / error.code() == "config.invalid". For the M6 sub-case: invoke_bash to write a working-tree .gitbutler/permissions.toml WITHOUT committing it to the target ref, then assert the guard still denies.
7. crates/but-testsupport/src/lib.rs (71-97, 432-441)
   Focus: writable_scenario("governance-base") + invoke_bash to commit permissions.toml + gates.toml at refs/heads/main; NEVER std::env::temp_dir().join(...).

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- Admin-write guard integration passes: `cargo test -p but-api admin_write_guard`  -> Exit 0; AC-1 (dev denied / admin Ok / working-tree self-grant denied) + AC-3 (malformed -> config.invalid) green
- Guard keys off the fully-qualified Authority axis: `grep -rEn 'but_authz::authorize|Authority::AdministrationWrite' crates/but-api/src/legacy/config_mutate.rs`  -> 1+ match (fully-qualified form so AUTHZ-008 AUTHORITY_POSITIVE bites)
- No role-name / human-vs-AI label in the guard: `! grep -rEin 'implementer|reviewer|maintainer|is_human|is_ai' crates/but-api/src/legacy/config_mutate.rs`  -> No matches (T-LOOP-005 / T-AUTHZ-016)
- No Permission-lock overload: `! grep -rEn 'write_permission\(|RepoExclusive|exclusive_worktree_access' crates/but-api/src/legacy/config_mutate.rs`  -> No matches
- No fabricated CLI verb: `! grep -rEn 'Perm|perm.*Subcommand|Group.*Subcommand' crates/but/src/args/forge.rs crates/but/src/args/mod.rs` (this task adds NO but perm / but group verb — those are Sprint 05; reviewer confirms args unchanged)
- Crate compiles incl. tests: `cargo check -p but-api --all-targets`  -> Exit 0
- Clippy clean: `cargo clippy -p but-api --all-targets`  -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: Reusable-admin-write-guard mirroring the commit gate — enforce_administration_write_gate(repo, target_ref): resolve_principal_from_env(&cfg) -> but_authz::authorize(&p, but_authz::Authority::AdministrationWrite, &cfg)? after load_governance_config(repo, target_ref) (ConfigError -> config.invalid). The guard keys off the existing Authority::AdministrationWrite variant via the fully-qualified authorize form, carries a gate-owned &'static str code, reads ONLY the committed target-ref blob (working-tree self-grant denied), and is the AUTHORIZATION-DECISION primitive Sprint 05's but perm / but group verbs consume; the integration test drives the guard FUNCTION directly with dev-denied kept beside admin-Ok so a no-op stub fails.
pattern_source: crates/but-api/src/commit/gate.rs:57 (enforce_commit_gate_for_target — load->resolve->authorize) + :83 (classify_error returning {code,message}) + crates/but-authz/src/authority.rs:35 (the existing AdministrationWrite variant)
anti_pattern: Adding a new Authority variant (it exists); using a bare un-prefixed authorize so the AUTHORITY_POSITIVE grep scores 0; keying the decision off a role name or handle string; reading the working-tree config (the self-escalation hole); overloading the Permission/RepoExclusive lock as the authz carrier; fabricating a but perm / but group CLI verb (Sprint 05); claiming a persisted-mutation block (the consumer is Sprint 05); asserting an always-Ok admin pass without the dev-denied falsifier in the same test; asserting remediation_hint (dropped, MGMT-IPC-002); or blurring a malformed config into perm.denied.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — Wires the EXISTING Authority::AdministrationWrite variant onto a new reusable config-mutate guard mirroring the commit gate (load->resolve->fully-qualified authorize, Denial -> perm.denied / ConfigError -> config.invalid), proves a non-admin is denied and an admin passes against real but-authz + real git with the dev-denied falsifier beside admin-Ok, proves the committed-blob-only self-escalation defense, and keeps the keys-off-Authority structural property — WITHOUT fabricating the Sprint-05 CLI verbs or claiming a persisted-mutation block. Owns the reusable guard, the honest AUTHORIZATION-DECISION scoping, and integration TDD over the guard FUNCTION.
reviewer: rust-reviewer
coding_standards: crates/AGENTS.md, crates/WORKSPACE_MODEL.md, crates/but/AGENTS.md, crates/but-api/src/commit/gate.rs

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: AUTHZ-001, AUTHZ-002, AUTHZ-003   (the but-authz primitive incl. the existing Authority::AdministrationWrite variant)
Blocks:     AUTHZ-005 (the self-grant-config-edit half of the confinement integration); AUTHZ-008; Sprint 05, Sprint 06a
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "AUTHZ-006",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "notes": [
    "C2 re-scope: no but-api config-write seam exists (grep: 0). AC-1 exercises the guard FUNCTION enforce_administration_write_gate directly (real load_governance_config + real authorize + BUT_AGENT_HANDLE) and proves the AUTHORIZATION DECISION. The persisted-write CONSUMER is Sprint 05 — naming it is honest scoping, not a stub. This is REAL integration (real authz logic/config/git).",
    "C2 falsifiability: dev-denied is kept in the SAME test as admin-Ok, so a no-op-always-Ok guard passes admin but FAILS dev-denied. Named in the AC-1 negative_control.",
    "M6: a working-tree administration:write self-grant for the test principal is STILL denied (the guard reads the committed target-ref blob; working-tree config cannot grant authority) — closes the working-tree self-escalation path.",
    "C4(d): config_mutate.rs uses the FULLY-QUALIFIED but_authz::authorize form so AUTHZ-008's AUTHORITY_POSITIVE_PATTERN (no bare Authority:: branch) bites.",
    "M2: CommitGateError carries only {code,message}; remediation_hint dropped (MGMT-IPC-002, Sprint 06a). Assertions scope to {code,message}."
  ],
  "fixtures": {
    "admin_write_repo": {
      "description": "A real git repo (but-testsupport writable_scenario) whose target ref main has committed .gitbutler/permissions.toml (dev=[contents:write]; admin=[administration:write,merge]) and a valid .gitbutler/gates.toml ([[branch]] main protected=true). The admin-write guard keys off Authority::AdministrationWrite (the variant ALREADY exists in but-authz, authority.rs:35) at the target ref, reading the COMMITTED blob only. There is NO `but perm` / `but group` CLI verb and NO but-api config-write entrypoint today (grep-confirmed; the persisted-write consumer lands in Sprint 05) — so this task provides the reusable enforce_administration_write_gate the Sprint-05 verbs consume, and the test exercises the guard FUNCTION directly. The M6 self-escalation sub-case additionally writes administration:write for dev into the WORKING-TREE .gitbutler/permissions.toml WITHOUT committing it to the target ref.",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"governance-base\");",
        "invoke_bash: write .gitbutler/permissions.toml ([[principal]] id=\"dev\" permissions=[\"contents:write\"]; [[principal]] id=\"admin\" permissions=[\"administration:write\",\"merge\"])",
        "invoke_bash: write a VALID .gitbutler/gates.toml ([[branch]] name=\"main\" protected=true)",
        "invoke_bash: stage and commit both blobs at refs/heads/main",
        "M6 sub-case only: invoke_bash to write a WORKING-TREE .gitbutler/permissions.toml granting dev administration:write WITHOUT committing it to refs/heads/main (the committed target-ref blob stays dev=[contents:write])"
      ]
    },
    "admin_write_malformed": {
      "description": "Same shape as admin_write_repo but the target-ref .gitbutler/permissions.toml is committed with INVALID TOML so the admin-write guard's loader fails closed config.invalid (proving the guard classifies a malformed config deterministically as config.invalid, never perm.denied or a skip). admin=[administration:write] would hold the authority if the config parsed.",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"governance-base\");",
        "invoke_bash: write a BROKEN permissions.toml `[[principal] id = \"admin\" permissions = nope` (unparseable TOML) and a valid gates.toml; stage and commit at refs/heads/main"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN admin_write_repo WHEN enforce_administration_write_gate(repo, \"refs/heads/main\") is called with BUT_AGENT_HANDLE=dev (contents:write only), then with BUT_AGENT_HANDLE=admin (administration:write), then with BUT_AGENT_HANDLE=dev after writing administration:write for dev into the WORKING-TREE permissions.toml THEN dev -> Err(Denial) code==\"perm.denied\" naming administration:write; admin -> Ok(()); the working-tree-grant dev -> STILL Err(Denial) perm.denied (the guard reads the committed target-ref blob, not the working tree). This proves the AUTHORIZATION DECISION (the persisted-write consumer is Sprint 05); the dev-denied case sits beside admin-Ok so a no-op-always-Ok guard fails",
      "verify": "cargo test -p but-api admin_write_guard_denies_non_admin_allows_admin",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api config-mutate guard FUNCTION + real but-authz + real git",
        "negative_control": {
          "would_fail_if": [
            "the guard returns Ok for everyone — it passes the admin case but FAILS the dev-denied half (a no-op-always-Ok stub is caught by keeping dev-denied in the same test)",
            "the guard keys off a role name or handle string rather than Authority::AdministrationWrite",
            "the guard reads authority from the working tree instead of the target-ref committed blob, so the working-tree self-grant flips dev from denied to Ok (the M6 self-escalation hole)",
            "the admin (administration:write holder) is wrongly denied (the guard is too strict / checks the wrong Authority)",
            "either denial returns a code other than perm.denied"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "admin_write_repo",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=dev: call enforce_administration_write_gate(repo, \"refs/heads/main\") (dev holds contents:write but NOT administration:write) (cite T-AUTHZ-021)"
              ]
            },
            "end_state": {
              "must_observe": [
                "the guard returns `Err` carrying a `Denial` with `code == \"perm.denied\"`",
                "the `message` names the missing `\"administration:write\"` authority",
                "the structured error is `{code, message}` only (no remediation_hint — the mirror drops it, MGMT-IPC-002)"
              ],
              "must_not_observe": [
                "the guard returns `Ok` for dev (`0` denials where exactly `1` is required)",
                "dev permitted to mutate governed config",
                "a code other than perm.denied"
              ]
            }
          },
          {
            "start_ref": "admin_write_repo",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=admin: call enforce_administration_write_gate(repo, \"refs/heads/main\") (admin holds administration:write) — assert the guard returns Ok (the authorization decision permits); this is the AUTHORIZATION DECISION, not a persisted-mutation block (cite T-AUTHZ-021 positive control)"
              ]
            },
            "end_state": {
              "must_observe": [
                "the guard returns `Ok(())` for the administration:write holder (0 governance denials for the authorized admin)"
              ],
              "must_not_observe": [
                "a wrongful `Err(Denial)` / `error.code == \"perm.denied\"` for the admin holder (`1+` denials where `0` are expected)",
                "the admin denied by an over-strict guard"
              ]
            }
          },
          {
            "start_ref": "admin_write_repo",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=dev: after writing administration:write for dev into the WORKING-TREE .gitbutler/permissions.toml (uncommitted), call enforce_administration_write_gate(repo, \"refs/heads/main\") again — proves the guard reads the committed target-ref blob, not the working tree (cite M6 self-escalation defense, CAP-CONFIG-01)"
              ]
            },
            "end_state": {
              "must_observe": [
                "the guard STILL returns `Err(Denial)` `code == \"perm.denied\"` naming administration:write — the working-tree grant did not widen dev's authority",
                "the committed target-ref permissions.toml is unchanged (dev still holds only contents:write there)"
              ],
              "must_not_observe": [
                "the guard returns `Ok` because it read the working-tree grant (the self-escalation hole)",
                "dev escalated to administration:write via an uncommitted working-tree edit (the committed blob is UNCHANGED — `0` authority widened)"
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
      "description": "GIVEN the source tree after this task WHEN the build-gate greps run over the config-mutate guard THEN the admin-write decision keys off the fully-qualified but_authz::authorize / Authority::AdministrationWrite (a positive AUTHORITY match), and NO role name / human-vs-AI label appears in the guard path, and the GitButler Permission/RepoExclusive lock is NOT overloaded as the authz carrier [build-gate]",
      "verify": "grep -rEn 'but_authz::authorize|Authority::AdministrationWrite' crates/but-api/src/legacy/config_mutate.rs && ! grep -rEin 'implementer|reviewer|maintainer|is_human|is_ai' crates/but-api/src/legacy/config_mutate.rs && ! grep -rEn 'write_permission\\(|RepoExclusive' crates/but-api/src/legacy/config_mutate.rs",
      "maps_to_ac": null,
      "scenario": {
        "tier": "holdout",
        "test_tier": "unit",
        "unit_test_justified": "Build-gate structural/grep invariant with zero runtime I/O: it proves the admin-write decision keys off the functional, fully-qualified but_authz::authorize(Authority::AdministrationWrite) axis (not a role name) and does not overload the GitButler Permission lock. The behavioral guarantee (non-admin denied, admin permitted, working-tree self-grant denied) is proven by the AC-1 integration cases; this gate enforces the keys-off-Authority structural property (T-AUTHZ-022) that a runtime test cannot fully assert.",
        "verification_service": "source grep (build-gate, no runtime I/O)",
        "negative_control": {
          "would_fail_if": [
            "the config-mutate guard keys off a role name (e.g. matches \"admin\"/\"maintainer\") instead of Authority::AdministrationWrite — a role-keyed enforcement regression",
            "the guard branches on a human-vs-AI label (is_human/is_ai)",
            "the guard overloads the GitButler repo-access Permission / RepoExclusive lock as the authorization carrier instead of the orthogonal Authority axis",
            "the guard uses a bare un-prefixed authorize so the AUTHORITY_POSITIVE grep scores 0 matches (a disconnected/stubbed guard the fully-qualified form requirement closes)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "admin_write_repo",
            "action": {
              "actor": "ci",
              "steps": [
                "grep config_mutate.rs for the fully-qualified but_authz::authorize / Authority::AdministrationWrite positive match; grep it for role-name / human-vs-AI labels (must be 0); grep it for the Permission-lock carrier (must be 0) (cite T-AUTHZ-022, T-LOOP-005, T-AUTHZ-016)"
              ]
            },
            "end_state": {
              "must_observe": [
                "`grep -rEn 'but_authz::authorize|Authority::AdministrationWrite' crates/but-api/src/legacy/config_mutate.rs` returns `1+` matches (the guard keys off the fully-qualified Authority axis)",
                "`! grep -rEin 'implementer|reviewer|maintainer|is_human|is_ai' crates/but-api/src/legacy/config_mutate.rs` returns `0` role/label matches",
                "`! grep -rEn 'write_permission\\(|RepoExclusive' crates/but-api/src/legacy/config_mutate.rs` returns `0` Permission-carrier matches"
              ],
              "must_not_observe": [
                "`0` matches for the fully-qualified AUTHORITY_POSITIVE grep (the guard never consults Authority or uses a bare authorize)",
                "`1+` role-name / human-vs-AI matches in the guard path",
                "`1+` Permission/RepoExclusive carrier matches (the repo lock overloaded as authz)"
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
      "description": "GIVEN admin_write_malformed (the target-ref permissions.toml is unparseable TOML) WHEN enforce_administration_write_gate is called (BUT_AGENT_HANDLE=admin) THEN the guard fails closed with a ConfigError whose code()==\"config.invalid\" (Err, no Ok, no perm.denied), never a skip — the guard's loader classifies a malformed config deterministically",
      "verify": "cargo test -p but-api admin_write_guard_malformed_config_invalid",
      "maps_to_ac": null,
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api config-mutate guard FUNCTION + real but-authz + real git",
        "negative_control": {
          "would_fail_if": [
            "the malformed target-ref config is skipped (treated as no governance) so the guard returns Ok (fail-open, a no-op guard)",
            "the malformed config is misclassified as perm.denied instead of config.invalid (the codes are blurred)",
            "the guard reads the working-tree permissions.toml instead of the target-ref blob, bypassing the malformed committed config",
            "the guard panics on the malformed config instead of returning the structured config.invalid contract"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "admin_write_malformed",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=admin: call enforce_administration_write_gate against the malformed target-ref permissions.toml (cite UC-CONFIG-01, T-AUTHZ-029 family)"
              ]
            },
            "end_state": {
              "must_observe": [
                "the guard returns `Err` carrying a `ConfigError` whose `code() == \"config.invalid\"`",
                "no `Ok` and no `perm.denied`"
              ],
              "must_not_observe": [
                "`error.code == \"perm.denied\"` (a malformed config must NOT be misclassified as a permission denial)",
                "the guard returns `Ok` (the malformed config silently skipped / treated as ungoverned — `0` errors where a config.invalid is required)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "enforce_administration_write_gate denies dev perm.denied naming administration:write; permits admin (Ok); a working-tree administration:write self-grant for dev is STILL denied (committed-blob-only) — dev-denied beside admin-Ok so a no-op stub fails (T-AUTHZ-021, M6)",
      "verify": "cargo test -p but-api admin_write_guard_denies_non_admin_allows_admin",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "config-mutate guard keys off the fully-qualified but_authz::authorize / Authority::AdministrationWrite, no role-name / human-vs-AI label, no Permission-lock overload (T-AUTHZ-022, T-LOOP-005, T-AUTHZ-016)",
      "verify": "grep -rEn 'but_authz::authorize|Authority::AdministrationWrite' crates/but-api/src/legacy/config_mutate.rs",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "malformed target-ref config -> config.invalid, never perm.denied, never skipped (deterministic classification)",
      "verify": "cargo test -p but-api admin_write_guard_malformed_config_invalid",
      "maps_to_ac": "AC-3"
    }
  ]
}
-->
</details>
