# AUTHZ-002: Ref-pinned governance config loader (`gix`, target-ref blob read)

## What this does

Implement the ref-pinned loader in `but-authz`: read both committed config blobs (`.gitbutler/permissions.toml`, `.gitbutler/gates.toml`) at a caller-supplied target ref via `gix`, parse them into typed `GovConfig` (principals→AuthoritySet, groups, per-branch protection), and fail closed with a `config.invalid`-classified error on any read/parse fault — proving a working-tree edit can never influence the loaded config.

## Why

Sprint 01a · PRD UC-AUTHZ-01, UC-AUTHZ-04 · capabilities CAP-CONFIG-01. Part of the functional-permission governance walking skeleton (commit allow/deny through real `but-authz` + real git).

## How to verify

PRIMARY **AC-1** — `cargo test -p but-authz config_loads_from_target_ref` (integration). Full gate set in the spec below.

## Scope

- crates/but-authz/src/config.rs (NEW)
- crates/but-authz/src/lib.rs (MODIFY) — re-export the loader + GovConfig + ConfigError
- crates/but-authz/Cargo.toml (MODIFY) — add `gix`, `toml`, `serde`, `anyhow` workspace deps
- crates/but-authz/tests/config.rs (NEW)
- crates/but-authz/tests/fixtures/** (NEW) — committed scenario seed scripts if needed

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: AUTHZ-002 - Ref-pinned governance config loader (`gix`, target-ref blob read)
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     M  (210 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-AUTHZ-01, UC-AUTHZ-04
CAPABILITIES: CAP-CONFIG-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-authz config_loads_from_target_ref
  check: cargo check -p but-authz --all-targets
  lint:  cargo clippy --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
`cargo test -p but-authz config` is green against real `gix` repos seeded by `but-testsupport`: a committed `permissions.toml` loads each principal's `AuthoritySet`; editing the WORKING-TREE copy does not change the loaded result (target-ref pin proven); a malformed `gates.toml` committed at the target ref returns a `config.invalid`-classified `Err` (not an empty config); a role entry and the equivalent list entry load to equal sets.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST read config ONLY from the committed blob at the TARGET REF: `repo.find_reference(target_ref)?.peel_to_commit()?.tree()?.lookup_entry_by_path(".gitbutler/permissions.toml")` — NEVER `repo.workdir()`/`std::fs::read` of the working tree and NEVER the feature head (CAP-CONFIG-01; this is the self-escalation defense).
- [MUST] MUST use `gix` (gitoxide), not `git2` — new repository logic per crates/AGENTS.md; git2 is legacy/boundary-only.
- [MUST] MUST treat a missing/unreadable/malformed config (unparseable Authority token, bad TOML, undefined referenced group) as a typed fail-closed error that maps to `config.invalid` — never an empty-config-means-allow path.
- [NEVER] NEVER fall back to the working-tree file when the target-ref blob is absent — absence at the ref is its own state the gate decides on, not a reason to read the disk. The GATE (GATES-001) owns the opt-in decision — it only invokes this loader when governance is present (≥1 `.gitbutler/*.toml` committed via `but_authz::governance_present`); the loader's absence→config.invalid contract therefore applies to the partial/incomplete case (one file present), not to a fully-ungoverned ref.
- [NEVER] NEVER use `std::env::temp_dir().join(...)` in tests — use `but-testsupport` (`writable_scenario`/`Sandbox`/`invoke_bash`) to create real committed scenarios.
- [STRICTLY] STRICTLY use `anyhow::Context` to explain WHICH operation failed (e.g. `.with_context(|| format!("reading {path} at {target_ref}"))`) and attach the consumer-facing `config.invalid` classification — don't make consumers match error strings (crates/AGENTS.md).
- [STRICTLY] STRICTLY desugar role entries via AUTHZ-001's `AuthoritySet::from_role` at load — the returned `GovConfig` carries only `AuthoritySet`s, never role names.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: Config loads from the committed blob at the target ref [PRIMARY]
- [ ] AC-2: A working-tree edit cannot change the loaded config (ref-pin proven)
- [ ] AC-3: Malformed config at the target ref fails closed as config.invalid (gates.toml OR permissions.toml)
- [ ] AC-4: A role entry and the equivalent list entry load to equal sets
- [ ] AC-5: Loading at refs/heads/main reads main's committed config, not the feature head
- [ ] All verification gates pass; only write_allowed files modified (git diff --name-only)

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: Config loads from the committed blob at the target ref [PRIMARY] [PRIMARY]
  GIVEN: fixture `governed_repo` — a real gix repo whose `main` ref has committed permissions.toml/gates.toml
  WHEN:  `load_governance_config(&repo, "refs/heads/main")` is called
  THEN:  principal `dev` resolves to an `AuthoritySet` containing `contents:write`, `ro` resolves to a set WITHOUT `contents:write`, and `main` is marked protected — all read from the committed blob
  TEST_TIER: integration   VERIFICATION_SERVICE: real gix repo + but-authz (seeded by but-testsupport)
  VERIFY: cargo test -p but-authz config_loads_from_target_ref
  SCENARIO (tier=visible, test_tier=integration):
    NEGATIVE_CONTROL would fail if: loader reads the working-tree file instead of the target-ref blob; loader returns an empty/default GovConfig (no principals); load is a stub returning a hardcoded config
    EVIDENCE: stdout (required_capture=True)
    case[0] (api_client): load_governance_config(&repo, "refs/heads/main"); assert principal "dev" effective set contains ContentsWrite; assert principal "ro" set does NOT contain ContentsWrite; assert branch "main" protected == true
      MUST_OBSERVE:     ['`dev` effective set contains `Authority::ContentsWrite`', '`ro` effective set excludes `Authority::ContentsWrite`', 'branch `main` `protected == true`']
      MUST_NOT_OBSERVE: ['empty GovConfig', '`0` principals', 'branch `main` `protected == false`']

AC-2: A working-tree edit cannot change the loaded config (ref-pin proven)
  GIVEN: fixture `governed_repo` after a WORKING-TREE config file is edited (NOT committed)
  WHEN:  `load_governance_config(&repo, "refs/heads/main")` is called
  THEN:  the loaded config still reports the committed values — the uncommitted working-tree edit (of either gates.toml or permissions.toml) is ignored
  TEST_TIER: integration   VERIFICATION_SERVICE: real gix repo + but-authz
  VERIFY: cargo test -p but-authz config_ignores_working_tree_edit
  SCENARIO (tier=visible, test_tier=integration):
    NEGATIVE_CONTROL would fail if: loader reads working-tree gates.toml so the edit flips protection to false; loader uses a static std::fs::read of the workdir path (disconnected from the target-ref blob); loader peels the wrong (feature-head) ref; loader reads working-tree permissions.toml so an uncommitted grant widens ro's set
    EVIDENCE: stdout (required_capture=True)
    case[0] (api_client): overwrite working-tree .gitbutler/gates.toml so main protected=false (do NOT commit); load_governance_config(&repo, "refs/heads/main"); assert main still protected == true
      MUST_OBSERVE:     ['after working-tree edit, branch `main` `protected == true`']
      MUST_NOT_OBSERVE: ['branch `main` `protected == false`', 'no workdir path read']
    case[1] (api_client): overwrite working-tree .gitbutler/permissions.toml so `ro` is granted contents:write (do NOT commit); load_governance_config(&repo, "refs/heads/main"); assert `ro` effective set STILL excludes ContentsWrite
      MUST_OBSERVE:     ['after the uncommitted permissions.toml edit, `ro` effective set excludes `Authority::ContentsWrite` (target-ref blob governs, not the working tree)']
      MUST_NOT_OBSERVE: ['`ro` set contains `Authority::ContentsWrite`', 'no workdir path read']

AC-3: Malformed config at the target ref fails closed as config.invalid (gates.toml OR permissions.toml)
  GIVEN: fixture `malformed_gates_repo` (broken gates.toml) and fixture `malformed_permissions_repo` (broken permissions.toml) — each has the malformed blob committed at the `main` ref
  WHEN:  `load_governance_config(&repo, "refs/heads/main")` is called
  THEN:  it returns an `Err(ConfigError)` classified as `config.invalid`, never an empty/default config and never a silent skip
  TEST_TIER: integration   VERIFICATION_SERVICE: real gix repo + but-authz
  VERIFY: cargo test -p but-authz config_malformed_fails_closed
  SCENARIO (tier=visible, test_tier=integration):
    NEGATIVE_CONTROL would fail if: parse error is swallowed and an empty GovConfig is returned (fail-open); load returns Ok on broken TOML; the error is untyped so a consumer cannot classify config.invalid; a malformed permissions.toml is skipped (treated as no principals) instead of erroring config.invalid
    EVIDENCE: stdout (required_capture=True)
    case[0] (api_client): load_governance_config(&repo, "refs/heads/main"); assert Err; assert error classifies as config.invalid
      MUST_OBSERVE:     ['`load_governance_config` returns `Err(ConfigError)`', 'error `code == "config.invalid"`']
      MUST_NOT_OBSERVE: ['`Ok(`', 'empty GovConfig', 'branch `main` `protected == false`']
    case[1] (api_client): load_governance_config(&repo, "refs/heads/main"); assert Err; assert error classifies as config.invalid (malformed permissions.toml, not just gates.toml)
      MUST_OBSERVE:     ['`load_governance_config` returns `Err(ConfigError)`', 'error `code == "config.invalid"`']
      MUST_NOT_OBSERVE: ['`Ok(`', 'empty GovConfig', 'the malformed permissions file silently skipped']

AC-4: A role entry and the equivalent list entry load to equal sets
  GIVEN: fixture `governed_repo` where `release-bot` uses `role="maintain"`
  WHEN:  the config is loaded and `release-bot`'s set is compared to a directly-loaded `maintain` desugar
  THEN:  `release-bot`'s loaded `AuthoritySet` equals `AuthoritySet::from_role("maintain")` (contains merge, excludes administration:write) — enforcement only ever sees the set
  TEST_TIER: integration   VERIFICATION_SERVICE: real gix repo + but-authz
  VERIFY: cargo test -p but-authz config_role_entry_desugars
  SCENARIO (tier=visible, test_tier=integration):
    NEGATIVE_CONTROL would fail if: a role entry is stored as a role string and not desugared at load; the loaded maintain set silently includes administration:write; role entry yields an empty set
    EVIDENCE: stdout (required_capture=True)
    case[0] (api_client): load_governance_config(&repo, "refs/heads/main"); assert release-bot set contains Merge and AdministrationRead; assert release-bot set does NOT contain AdministrationWrite
      MUST_OBSERVE:     ['`release-bot` set contains `Authority::Merge`', '`release-bot` set contains `Authority::AdministrationRead`', '`release-bot` set excludes `Authority::AdministrationWrite`']
      MUST_NOT_OBSERVE: ['`release-bot` set contains `Authority::AdministrationWrite`', 'empty set', 'no role string retained (`role="maintain"`)']

AC-5: Loading at refs/heads/main reads main's committed config, not the feature head
  GIVEN: fixture `feature_head_unprotects` — `refs/heads/main` has committed gates.toml with main protected=true, while a feature branch `feat` has a COMMITTED gates.toml that sets main protected=false
  WHEN:  `load_governance_config(&repo, "refs/heads/main")` is called (target ref = main, while the worktree is on feat)
  THEN:  the loaded config reports `main` protected==true — the feature-head's committed unprotecting gates.toml is NOT consulted (target ref, not HEAD, governs)
  TEST_TIER: integration   VERIFICATION_SERVICE: real gix repo + but-authz (seeded by but-testsupport)
  VERIFY: cargo test -p but-authz config_loads_from_target_not_head
  SCENARIO (tier=holdout, test_tier=integration):
    NEGATIVE_CONTROL would fail if: loader peels HEAD/the feature branch instead of the supplied target ref so the unprotecting feat gates.toml wins; loader reads a static workdir path disconnected from the target ref; protection is read from a hardcoded list ignoring the ref entirely
    EVIDENCE: stdout (required_capture=True)
    case[0] (api_client): check out feat (whose committed gates.toml unprotects main); load_governance_config(&repo, "refs/heads/main"); assert branch main protected == true
      MUST_OBSERVE:     ['branch `main` `protected == true` (read from `refs/heads/main`, not the feature head)']
      MUST_NOT_OBSERVE: ['branch `main` `protected == false`', 'no target-ref read']

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, happy_path): load_governance_config reads dev's contents:write from the committed target-ref permissions.toml
    VERIFY: cargo test -p but-authz config_loads_from_target_ref
- TC-2 (-> AC-2, edge): An uncommitted working-tree gates.toml/permissions.toml edit does not change the loaded config (T-GRPS-014/T-GATES-019 ref-pin)
    VERIFY: cargo test -p but-authz config_ignores_working_tree_edit
- TC-3 (-> AC-3, error): Malformed target-ref gates.toml OR permissions.toml returns Err classified config.invalid, not empty config (T-AUTHZ-029 fail-closed)
    VERIFY: cargo test -p but-authz config_malformed_fails_closed
- TC-4 (-> AC-4, happy_path): release-bot role=maintain loads to a set with Merge, AdministrationRead, no AdministrationWrite
    VERIFY: cargo test -p but-authz config_role_entry_desugars
- TC-5 (-> AC-5, edge): load_governance_config(repo, refs/heads/main) reads main's committed gates.toml, NOT a feature head that unprotects main (T-GATES-005 target-ref-governs)
    VERIFY: cargo test -p but-authz config_loads_from_target_not_head

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-CONFIG-01
provides: but_authz::config::load_governance_config(repo, target_ref) -> Result<GovConfig, ConfigError>; but_authz::config::GovConfig; but_authz::config::ConfigError (carries config.invalid semantics)
consumes: but_authz::Authority; but_authz::AuthoritySet; but_authz::Principal; but_authz::Group
boundary_contracts:
  - CAP-CONFIG-01 hop-1/2/3: read the committed `.gitbutler/permissions.toml` + `.gitbutler/gates.toml` blob AT THE TARGET REF via gix, parse to typed config; a malformed/unreadable blob fails closed as a typed config.invalid error; working-tree/feature-head edits are never read

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-authz/src/config.rs (NEW)
  - crates/but-authz/src/lib.rs (MODIFY) — re-export the loader + GovConfig + ConfigError
  - crates/but-authz/Cargo.toml (MODIFY) — add `gix`, `toml`, `serde`, `anyhow` workspace deps
  - crates/but-authz/tests/config.rs (NEW)
  - crates/but-authz/tests/fixtures/** (NEW) — committed scenario seed scripts if needed
writeProhibited:
  - crates/but-authz/src/authority.rs — owned by AUTHZ-001; consume from_role, do not change the catalog
  - crates/but-workspace/** — the commit gate that USES this loader is GATES-001
  - crates/but-api/** — enforcement seam is AUTHZ-003/GATES-001
  - any `gitbutler-*` crate — no new legacy usage (read gitbutler-repo:284 for the gix pattern only, do not modify it)
  - Any file not listed in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/gitbutler-repo/src/commands.rs (lines 284-295)
   Focus: PRIMARY PATTERN — the exact gix blob-by-path read: `repo.find_commit(id)?.tree()?` then `tree.lookup_entry_by_path(path)?` → `repo.find_blob(entry.id())?.data`. Adapt the commit source to `find_reference(target_ref)?.peel_to_commit()?` for the ref-pin.
2. crates/but-core/src/commit/mod.rs (lines 681-692)
   Focus: PRIMARY PATTERN (TOML from a blob) — `toml::from_str(&blob.data.as_bstr().to_str_lossy())?` is the committed-blob→TOML precedent the PRD cites; use the same for permissions.toml/gates.toml.
3. crates/but-testsupport/src/lib.rs (lines 432-441 + 71-97)
   Focus: `writable_scenario(name) -> (gix::Repository, tempfile::TempDir)` (a FUNCTION, not a closure) + `invoke_bash(script, &repo)` to seed a real repo and `git add -A && git commit` `.gitbutler/*.toml` at refs/heads/main — NEVER temp_dir().join().
4. crates/but-workspace/src/upstream_integration.rs (lines 146-160)
   Focus: `repo.find_reference(&ref_name)?.id()` / immutable target-ref handling — confirms target ref resolution idiom and that the target is read-only.
5. /Users/justinrich/Projects/brain/docs/rust/error-handling.md (lines 1-90)
   Focus: anyhow::Context + a typed error that carries a stable classification (config.invalid) without forcing consumers to match strings.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- Integration tests pass: `cargo test -p but-authz config`  -> Exit 0; AC-1..4 green
- Crate compiles: `cargo check -p but-authz --all-targets`  -> Exit 0
- No working-tree read in the loader (ref-pin): `! grep -rEn 'workdir|std::fs::read|read_to_string' crates/but-authz/src/config.rs`  -> No matches (exit nonzero) — config is read only from the target-ref blob via gix
- No git2 in new logic: `! grep -rEn 'git2' crates/but-authz/src`  -> No matches — gix only
- Clippy clean: `cargo clippy -p but-authz --all-targets`  -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references: crates/gitbutler-repo/src/commands.rs:288 (lookup_entry_by_path + find_blob); crates/but-core/src/commit/mod.rs:691 (toml::from_str from blob data); 03-data-schema.md committed-config TOML shapes (permissions.toml/gates.toml examples); 08-capability-chains.md CAP-CONFIG-01 hop table
notes:
  - Signature: `pub fn load_governance_config(repo: &gix::Repository, target_ref: &gix::refs::FullNameRef /* or &str */) -> Result<GovConfig, ConfigError>`. Borrow the repo (&), never own it.
  - Read path: find_reference(target_ref) -> peel_to_commit() -> tree() -> lookup_entry_by_path(".gitbutler/permissions.toml") -> find_blob -> data -> toml::from_str. A `None` entry (file absent at ref) is a distinct GovConfig state, decided by the gate, NOT a working-tree fallback.
  - GovConfig holds: `principals: BTreeMap<PrincipalId, AuthoritySet>` (post-desugar), `groups: BTreeMap<GroupName, Group>`, `branches: BTreeMap<BranchName, BranchProtection>`. Wire structs (serde Deserialize) are private; GovConfig is the public typed view.
  - ConfigError variants map deterministically to `config.invalid` (the gate surfaces it); they are distinct from the unknown-principal case (perm.denied), which is AUTHZ-003's concern.
pattern: Borrow `&gix::Repository`, resolve target ref → tree → blob-by-path → typed TOML; fail closed on any read/parse error with a classified error. Two private serde wire structs, one public GovConfig.
pattern_source: crates/gitbutler-repo/src/commands.rs:284-295 + crates/but-core/src/commit/mod.rs:681-692
anti_pattern: Reading `repo.workdir().join(".gitbutler/...")` via std::fs (reads the working tree → self-escalation hole R4) or returning an empty GovConfig on parse failure (fail-open regression that passes every positive test).

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — Real-git blob reads via `gix` (find_reference → peel_to_commit → tree → lookup_entry_by_path), TOML parse, and fail-closed typed errors — integration TDD against `but-testsupport` scenarios. rust-implementer owns the gix idioms and the ref-pin invariant.
reviewer: rust-reviewer
coding_standards: crates/AGENTS.md, crates/WORKSPACE_MODEL.md, /Users/justinrich/Projects/brain/docs/rust/error-handling.md, /Users/justinrich/Projects/brain/docs/rust/ownership-borrowing.md

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: AUTHZ-001
Blocks:     GATES-001, AUTHZ-003
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "AUTHZ-002",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "governed_repo": {
      "description": "A real gix repo (via but-testsupport writable_scenario) whose target ref `main` has committed `.gitbutler/permissions.toml` (principal `dev`=contents:write; `ro`=contents:read; `release-bot` role=maintain) and `.gitbutler/gates.toml` (main protected).",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"governance-base\");",
        "invoke_bash: write .gitbutler/permissions.toml with [[principal]] id=\"dev\" permissions=[\"contents:write\"]; [[principal]] id=\"ro\" permissions=[\"contents:read\"]; [[principal]] id=\"release-bot\" role=\"maintain\"",
        "invoke_bash: write .gitbutler/gates.toml with [[branch]] name=\"main\" protected=true",
        "invoke_bash: git add -A && git commit -m \"governance config\" (commits both blobs at refs/heads/main)"
      ]
    },
    "malformed_gates_repo": {
      "description": "Same repo but the target-ref `.gitbutler/gates.toml` blob is committed with invalid TOML / an unparseable Authority token, to prove fail-closed. [seeded via but_testsupport::writable_scenario(name) + invoke_bash git commit]",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"governance-base\");",
        "invoke_bash: write .gitbutler/gates.toml with broken TOML `[[branch] name = \\\"main\\\"  protected = nope`",
        "invoke_bash: git add -A && git commit -m \"malformed gates\" (commits the broken blob at refs/heads/main)"
      ]
    },
    "malformed_permissions_repo": {
      "description": "Same repo but the target-ref `.gitbutler/permissions.toml` blob is committed with invalid TOML / an unparseable Authority token — to prove the loader fails closed config.invalid on a bad permissions file (not just gates). [seeded via but_testsupport::writable_scenario(name) + invoke_bash git commit]",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"governance-base\");",
        "invoke_bash: write .gitbutler/permissions.toml with broken TOML `[[principal] id = \\\"dev\\\"  permissions = nope` (or a valid-TOML file with an unparseable token permissions=[\"contents:bogus\"])",
        "invoke_bash: git add -A && git commit -m \"malformed permissions\" (commits the broken blob at refs/heads/main)"
      ]
    },
    "feature_head_unprotects": {
      "description": "A real gix repo (but-testsupport writable_scenario) where refs/heads/main commits .gitbutler/gates.toml [[branch]] name=\"main\" protected=true, and a feature branch feat commits .gitbutler/gates.toml [[branch]] name=\"main\" protected=false — proving the loader honors the supplied target ref, not HEAD.",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"governance-base\");",
        "invoke_bash: on main, write .gitbutler/gates.toml with [[branch]] name=\"main\" protected=true; git add -A && git commit -m gates",
        "invoke_bash: git checkout -b feat; overwrite .gitbutler/gates.toml with [[branch]] name=\"main\" protected=false; git add -A && git commit -m unprotect; git checkout main"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN a real gix repo with committed config at main WHEN load_governance_config(repo, refs/heads/main) is called THEN dev resolves to contents:write, ro does not, main is protected — read from the committed blob",
      "verify": "cargo test -p but-authz config_loads_from_target_ref",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real gix repo + but-authz",
        "negative_control": {
          "would_fail_if": [
            "loader reads the working-tree file instead of the target-ref blob",
            "loader returns an empty/default GovConfig (no principals)",
            "load is a stub returning a hardcoded config"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "governed_repo",
            "action": {
              "actor": "api_client",
              "steps": [
                "load_governance_config(&repo, \"refs/heads/main\")",
                "assert principal \"dev\" effective set contains ContentsWrite",
                "assert principal \"ro\" set does NOT contain ContentsWrite",
                "assert branch \"main\" protected == true"
              ]
            },
            "end_state": {
              "must_observe": [
                "`dev` effective set contains `Authority::ContentsWrite`",
                "`ro` effective set excludes `Authority::ContentsWrite`",
                "branch `main` `protected == true`"
              ],
              "must_not_observe": [
                "empty GovConfig",
                "`0` principals",
                "branch `main` `protected == false`"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "GIVEN a working-tree edit (gates.toml or permissions.toml, uncommitted) WHEN config is loaded at target ref THEN the loaded values still match the committed blob (ref-pin)",
      "verify": "cargo test -p but-authz config_ignores_working_tree_edit",
      "maps_to_ac": null,
      "primary": false,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real gix repo + but-authz",
        "negative_control": {
          "would_fail_if": [
            "loader reads working-tree gates.toml so the edit flips protection to false",
            "loader uses a static std::fs::read of the workdir path (disconnected from the target-ref blob)",
            "loader peels the wrong (feature-head) ref",
            "loader reads working-tree permissions.toml so an uncommitted grant widens ro's set"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "governed_repo",
            "action": {
              "actor": "api_client",
              "steps": [
                "overwrite working-tree .gitbutler/gates.toml so main protected=false (do NOT commit)",
                "load_governance_config(&repo, \"refs/heads/main\")",
                "assert main still protected == true"
              ]
            },
            "end_state": {
              "must_observe": [
                "after working-tree edit, branch `main` `protected == true`"
              ],
              "must_not_observe": [
                "branch `main` `protected == false`",
                "no workdir path read"
              ]
            }
          },
          {
            "start_ref": "governed_repo",
            "action": {
              "actor": "api_client",
              "steps": [
                "overwrite working-tree .gitbutler/permissions.toml so `ro` is granted contents:write (do NOT commit)",
                "load_governance_config(&repo, \"refs/heads/main\")",
                "assert `ro` effective set STILL excludes ContentsWrite"
              ]
            },
            "end_state": {
              "must_observe": [
                "after the uncommitted permissions.toml edit, `ro` effective set excludes `Authority::ContentsWrite` (target-ref blob governs, not the working tree)"
              ],
              "must_not_observe": [
                "`ro` set contains `Authority::ContentsWrite`",
                "no workdir path read"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "GIVEN a malformed committed gates.toml OR permissions.toml at target ref WHEN config is loaded THEN Err classified config.invalid, never empty/default",
      "verify": "cargo test -p but-authz config_malformed_fails_closed",
      "maps_to_ac": null,
      "primary": false,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real gix repo + but-authz",
        "negative_control": {
          "would_fail_if": [
            "parse error is swallowed and an empty GovConfig is returned (fail-open)",
            "load returns Ok on broken TOML",
            "the error is untyped so a consumer cannot classify config.invalid",
            "a malformed permissions.toml is skipped (treated as no principals) instead of erroring config.invalid"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "malformed_gates_repo",
            "action": {
              "actor": "api_client",
              "steps": [
                "load_governance_config(&repo, \"refs/heads/main\")",
                "assert Err",
                "assert error classifies as config.invalid"
              ]
            },
            "end_state": {
              "must_observe": [
                "`load_governance_config` returns `Err(ConfigError)`",
                "error `code == \"config.invalid\"`"
              ],
              "must_not_observe": [
                "`Ok(`",
                "empty GovConfig",
                "branch `main` `protected == false`"
              ]
            }
          },
          {
            "start_ref": "malformed_permissions_repo",
            "action": {
              "actor": "api_client",
              "steps": [
                "load_governance_config(&repo, \"refs/heads/main\")",
                "assert Err",
                "assert error classifies as config.invalid (malformed permissions.toml, not just gates.toml)"
              ]
            },
            "end_state": {
              "must_observe": [
                "`load_governance_config` returns `Err(ConfigError)`",
                "error `code == \"config.invalid\"`"
              ],
              "must_not_observe": [
                "`Ok(`",
                "empty GovConfig",
                "the malformed permissions file silently skipped"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "description": "GIVEN a role=maintain entry WHEN config is loaded THEN it desugars to the maintain set (merge yes, admin:write no)",
      "verify": "cargo test -p but-authz config_role_entry_desugars",
      "maps_to_ac": null,
      "primary": false,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real gix repo + but-authz",
        "negative_control": {
          "would_fail_if": [
            "a role entry is stored as a role string and not desugared at load",
            "the loaded maintain set silently includes administration:write",
            "role entry yields an empty set"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "governed_repo",
            "action": {
              "actor": "api_client",
              "steps": [
                "load_governance_config(&repo, \"refs/heads/main\")",
                "assert release-bot set contains Merge and AdministrationRead",
                "assert release-bot set does NOT contain AdministrationWrite"
              ]
            },
            "end_state": {
              "must_observe": [
                "`release-bot` set contains `Authority::Merge`",
                "`release-bot` set contains `Authority::AdministrationRead`",
                "`release-bot` set excludes `Authority::AdministrationWrite`"
              ],
              "must_not_observe": [
                "`release-bot` set contains `Authority::AdministrationWrite`",
                "empty set",
                "no role string retained (`role=\"maintain\"`)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "description": "GIVEN a feature head whose committed gates.toml unprotects main WHEN load_governance_config(repo, refs/heads/main) is called THEN main is still protected (target ref, not HEAD, governs)",
      "verify": "cargo test -p but-authz config_loads_from_target_not_head",
      "maps_to_ac": null,
      "primary": false,
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real gix repo + but-authz",
        "negative_control": {
          "would_fail_if": [
            "loader peels HEAD/the feature branch instead of the supplied target ref so the unprotecting feat gates.toml wins",
            "loader reads a static workdir path disconnected from the target ref",
            "protection is read from a hardcoded list ignoring the ref entirely"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "feature_head_unprotects",
            "action": {
              "actor": "api_client",
              "steps": [
                "check out feat (whose committed gates.toml unprotects main)",
                "load_governance_config(&repo, \"refs/heads/main\")",
                "assert branch main protected == true"
              ]
            },
            "end_state": {
              "must_observe": [
                "branch `main` `protected == true` (read from `refs/heads/main`, not the feature head)"
              ],
              "must_not_observe": [
                "branch `main` `protected == false`",
                "no target-ref read"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "dev's contents:write read from committed target-ref permissions.toml",
      "verify": "cargo test -p but-authz config_loads_from_target_ref",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "uncommitted working-tree edit (gates.toml or permissions.toml) ignored by loader",
      "verify": "cargo test -p but-authz config_ignores_working_tree_edit",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "malformed config (gates.toml or permissions.toml) -> Err config.invalid, not empty",
      "verify": "cargo test -p but-authz config_malformed_fails_closed",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "role=maintain desugars on load",
      "verify": "cargo test -p but-authz config_role_entry_desugars",
      "maps_to_ac": "AC-4"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "load reads the supplied target ref's config, not HEAD/feature-head",
      "verify": "cargo test -p but-authz config_loads_from_target_not_head",
      "maps_to_ac": "AC-5"
    }
  ]
}
-->
</details>
