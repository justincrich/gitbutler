# MGMT-BE-004: `branch_gates_read`/`branch_gates_update` gate-config but-api producer (the gates.toml writer) + its Tauri command/SDK delta

## What this does

Author the first persisted writer of .gitbutler/gates.toml as a net-new but-api producer (branch_gates_read / branch_gates_update) that admin-gates via the Sprint-02 enforce_administration_write_gate, LOSSLESSLY round-trips the full [[branch]] + [[gate]] schema the merge gate consumes, writes inert-until-committed to the working tree, returns a pending signal from the working-tree-vs-target-ref diff, and exposes its Tauri command + regenerated SDK delta.

## Why

Sprint 06b · PRD UC-MGMT-04, UC-MGMT-06 · capability CAP-AUTHZ-01, CAP-CONFIG-01. Proven against real git: (1) an admin branch_gates_update edits a branch's gate field in the WORKING-TREE gates.toml while branch_gates_read still reports the COMMITTED (target-ref) value (inert pair via the task's OWN reader — merge_gate e

## How to verify

PRIMARY **AC-1** — `cargo test -p but-api branch_gates_update_writes_worktree_inert_until_committed`: Admin branch_gates_update writes the gate edit into the working-tree gates.toml while the COMMITTED (target-ref) gate read by branch_gates_read is unchanged (inert-until-committed pair). Full gate set in the spec below.

## Scope

  - crates/but-api/src/legacy/governance.rs (MODIFY — add branch_gates_read/branch_gates_update + the private gates.toml read-modify-write writer + the raw GatesWire/BranchWire/GateWire wire structs with #[derive(Serialize, Deserialize)] + the BranchGatesError/caveat constant; sited BESIDE CLI-001's perm_* fns)
  - crates/but-authz/src/config.rs (MODIFY — ADDITIVE ONLY: add `pub fn gates_path() -> &'static str` returning GATES_PATH; do NOT change loader/normalize semantics; do NOT widen BranchProtection or the loader's BranchWire)
  - crates/but-authz/src/lib.rs (MODIFY — re-export gates_path from the config pub use block)
  - crates/but-api/src/legacy/mod.rs (MODIFY — ensure governance module is declared; no-op if CLI-001 already added it)
  - crates/but-api/tests/branch_gates.rs (NEW — the PRIMARY but-api proofs AC-1..AC-5)
  - the desktop Tauri command file under crates/ that registers #[tauri::command] governance commands (MODIFY — register branch_gates_read/branch_gates_update beside the Sprint-06a perm_*/group_* commands; follow the existing snake_case #[tauri::command] convention) and src-tauri/capabilities/ allow-branch_gates_* entry (MODIFY — mirror the allow-perm_*/allow-group_* convention)
  - packages/but-sdk/src/generated/** (REGENERATE ONLY via `pnpm build:sdk && pnpm format` — NEVER hand-edit)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: MGMT-BE-004 — `branch_gates_read`/`branch_gates_update` gate-config but-api producer (the gates.toml writer) + its Tauri command/SDK delta
================================================================================

TASK_TYPE:   FEATURE
STATUS:      Backlog
PRIORITY:    P0
EFFORT:      L  (180 min)
AGENT:       implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-MGMT-04, UC-MGMT-06
CAPABILITIES:CAP-AUTHZ-01, CAP-CONFIG-01

RUNTIME_COMMANDS:
  test:  test -f crates/but-api/src/legacy/governance.rs && grep -q 'pub fn permissions_path' crates/but-authz/src/config.rs
  check: cargo check -p but-authz --all-targets
  lint:  cargo clippy -p but-authz -p but-api --all-targets

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Proven against real git: (1) an admin branch_gates_update edits a branch's gate field in the WORKING-TREE gates.toml while branch_gates_read still reports the COMMITTED (target-ref) value (inert pair via the task's OWN reader — merge_gate exposes no public min_approvals reader), and once committed the new value takes effect; (2) the writer preserves the FULL [[gate]] review-requirement array (min_approvals, require_distinct_from_author, require_approval_from_group) AND every unrelated [[branch]]/[[gate]] entry on a protection-only edit (no lossy drop); (3) toggling protected OFF lands protected=false; (4) a non-admin branch_gates_update is denied perm.denied (names administration:write) and writes nothing; a self-escalation that weakens one's own gate is surfaced as the structured Denial, not applied; (5) branch_gates_read returns the committed gate set plus a pending signal derived from the working-tree diff; (6) `pnpm build:sdk && pnpm format` regenerates the SDK with the new commands; cargo test -p but-api / -p but-authz green; clippy clean; the honesty grep stays green. Additionally: (7) the full editable field set lands — require_distinct_from_author AND require_approval_from_group are SET losslessly (AC-6); (8) a new branch APPENDS a [[branch]]+[[gate]] preserving existing entries and an absent gates.toml is CREATED (AC-7); (9) branch_gates_read is administration:read-gated — an unauthorized caller is denied perm.denied naming administration:read (AC-8).

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST compose the EXISTING admin-write guard, never author a second one. branch_gates_update calls but_api::legacy::config_mutate::enforce_administration_write_gate(&repo, target_ref) (config_mutate.rs:18) BEFORE the write and surfaces denial via config_mutate::classify_error (:31). Do NOT write a parallel authorize(AdministrationWrite) call.
- [MUST] MUST round-trip the FULL gates.toml schema LOSSLESSLY. gates.toml has TWO table arrays: [[branch]] {name, protected} (read by both but-authz/config.rs:438-443 AND merge_gate.rs:418-423) and [[gate]] {branch, type="review", min_approvals, require_approval_from_group, require_distinct_from_author} (read ONLY by merge_gate.rs:425-437, the merge requirement). The writer MUST parse and re-serialize BOTH arrays via raw wire structs that model the complete file; a writer that edits [[branch]] but drops [[gate]] silently strips every min_approvals / required-group / distinct requirement the merge gate enforces — the cardinal lossy bug AC-3 catches.
- [MUST] MUST write the WORKING TREE only. The writer resolves repo.workdir() and std::fs::write's .gitbutler/gates.toml there (mirror crates/but-api/tests/admin_write_guard.rs:158-164 write_worktree_permissions). It MUST NOT git add / stage / commit / touch any ref. The inert-until-committed contract is STRUCTURAL — the next target-ref load is what makes the edit effective. (The ONE commit is INSIDE the AC-1 test, via invoke_bash, AFTER the write, to prove effectiveness.)
- [MUST] MUST do a VALUE-preserving read-modify-write over RAW wire structs, NOT a normalized round-trip. but-authz's GovConfig models ONLY {name, protected} for branches (config.rs:195-217) and discards the [[gate]] requirement entirely — re-serializing it would LOSE every min_approvals / required-group / distinct field. Define an owned raw GatesWire/BranchWire/GateWire (matching merge_gate.rs:411-437) with #[derive(Serialize, Deserialize)] (keep #[serde(deny_unknown_fields)]), mutate only the targeted branch/gate, and re-serialize via toml::to_string (canonical TOML; VALUE-preserving, not byte-verbatim).
- [MUST] MUST resolve the .gitbutler/gates.toml path via a but-authz accessor, never a re-derived literal. Add pub fn gates_path() -> &'static str to config.rs returning GATES_PATH (:9) and re-export it (mirror CLI-001's permissions_path) — honors config.rs:42-43 'single source of truth for the governance file paths'.
- [MUST] MUST support toggling protected OFF (the unprotect path). branch_gates_update writing protected=false for an existing [[branch]] MUST land that value in the working-tree file (T-MGMT-047's confirmation dialog is UI; the write path must support the toggle). The required-group list values are config DATA, never enforcement branches.
- [MUST] MUST surface the SDK delta as part of done. After the Rust API lands, `pnpm build:sdk && pnpm format` regenerates packages/but-sdk/src/generated with branch_gates_read/branch_gates_update; the generated files are NEVER hand-edited.
- [MUST] MUST enforce ordering: enforce_administration_write_gate(&repo, target_ref) MUST execute and succeed BEFORE any std::fs::write of .gitbutler/gates.toml in branch_gates_update (fail-closed). The admin gate is a guard clause at the TOP of the function; no working-tree write may occur on the denial path (SEC-2 — AC-4's byte-for-byte-unchanged-after-denial is the behavioral proof; the source ordering is the structural requirement).
- [MUST] MUST resolve the HUMAN fleet-owner identity in the DESKTOP Tauri command. BUT_AGENT_HANDLE is UNSET in the desktop process; the branch_gates_read/branch_gates_update #[tauri::command] wrappers MUST resolve the fleet-owner principal via the Sprint-06a human-fleet-owner identity shim (UserService, per T-MGMT-042 / MGMT-IPC-003) and pass it as the env-principal the but-api fn authorizes against. The but-api functions KEEP their (&gix::Repository, target_ref: &str, ...) signature and env-principal resolution (resolve_principal_from_env via BUT_AGENT_HANDLE), which the rust tests exercise directly with temp_env BUT_AGENT_HANDLE; only the Tauri layer maps the desktop human identity onto that principal (SEC-3).
- [NEVER] NEVER drop, normalize, or smooth the [[gate]] review-requirement array on a branch-protection-only edit (lossy round-trip = silent governance weakening — CRITICAL).
- [NEVER] NEVER commit, stage, or move a ref from the production writer (breaks inert-until-committed; AC-1's ref-unchanged + AC-4's inert assertions catch it).
- [NEVER] NEVER re-implement admin gating — compose enforce_administration_write_gate (config_mutate.rs); do not fork a second authorize(AdministrationWrite).
- [NEVER] NEVER read governance config from the working tree or feature head for the AUTHORIZATION decision — admin authorization reads the TARGET REF blob (CAP-CONFIG-01); only the WRITE edits the working tree.
- [NEVER] NEVER branch on role names (read/triage/write/maintain/admin) or human-vs-AI predicates in the authorization path — use the typed Authority axis; the AUTHZ-007/008 invariant_build_gates honesty grep must stay green.
- [NEVER] NEVER overload GitButler's pre-existing Permission (the repo-access lock) — use but_authz Authority/AuthoritySet exclusively.
- [NEVER] NEVER hand-edit packages/but-sdk/src/generated — regenerate via pnpm build:sdk only.
- [NEVER] NEVER perform any working-tree write before the admin gate runs and succeeds — no fs::write, no file create, no path touch on the denial path (SEC-2 fail-open).
- [NEVER] NEVER pass BUT_AGENT_HANDLE-derived identity from the desktop Tauri command (it is unset there) — the desktop command resolves the human fleet-owner identity (T-MGMT-042) instead (SEC-3).
- [STRICTLY] STRICTLY treat the AUTHZ-006 admin guard (config_mutate.rs) and the Sprint-04 merge-gate requirement reader (merge_gate.rs) as CONSUMED seams — read merge_gate.rs's GatesWire/GateWire (:411-437) to learn the EXACT field set the writer must round-trip, but do NOT modify merge_gate.rs's loader semantics; the writer owns its own raw serde structs.
- [STRICTLY] STRICTLY site branch_gates_read/branch_gates_update at the but-api boundary (crates/but-api/src/legacy/governance.rs, beside CLI-001's perm_* / config_mutate.rs) so the Tauri command and the UI both reuse the SAME functions — never bury the write/read logic in crates/but/.
- [STRICTLY] STRICTLY keep the function signatures (&gix::Repository, target_ref: &str, ...) so the Tauri commands pass the same target ref the CLI resolves from the workspace target.
- [STRICTLY] STRICTLY treat a no-op edit (writing back the value already present) as a full-structure identity round-trip: re-serializing must re-parse (via the merge_gate-shaped GatesWire) to a byte-or-value-identical [[branch]]+[[gate]] set across ALL entries with gate.len()/branch.len() preserved (RUST-6 — AC-3/TC-12 cover this; deny_unknown_fields + owned wire structs keep it future-proof).

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [x] AC-1 [PRIMARY]: Admin branch_gates_update writes the gate edit into the working-tree gates.toml while the COMMITTED (target-ref) gate read by branch_gates_read is unchanged (inert-until-committed pair)
- [x] AC-2: Toggling protected OFF lands protected=false in the working tree while preserving the branch's [[gate]] requirement
- [ ] AC-3: The writer LOSSLESSLY round-trips the full [[gate]] requirement set and every unrelated entry on a protection-only edit (no lossy drop) /* PARTIAL: lossless round-trip unverified — see REMEDIATE-06B-B */
- [x] AC-4: A non-admin branch_gates_update is denied perm.denied (names administration:write) and writes nothing; a self-escalation gate-weakening is surfaced, not applied
- [ ] AC-5: branch_gates_read returns the committed gate set PLUS a pending signal computed from the working-tree-vs-target-ref diff
- [ ] AC-6: Admin branch_gates_update SETS require_distinct_from_author AND require_approval_from_group (the full editable field set lands losslessly)
- [ ] AC-7: branch_gates_update APPENDS a new [[branch]]+[[gate]] for a branch absent from gates.toml (preserving existing entries), and creates the file against an absent/empty gates.toml
- [ ] AC-8: branch_gates_read is administration:read-scoped — a caller lacking administration:read is denied perm.denied (governance reconnaissance is gated)
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: Admin branch_gates_update writes the gate edit into the working-tree gates.toml while the COMMITTED (target-ref) gate read by branch_gates_read is unchanged (inert-until-committed pair)
  GIVEN: gates_governance_base: refs/heads/main has committed gates.toml with [[branch]] main protected=true and the full [[gate]] main requirement (min_approvals=2, distinct=true, groups=[code-reviewers,maintainers]); BUT_AGENT_HANDLE=admin; clean working tree; ref_id(main) captured
  WHEN:  branch_gates_update(&repo, "refs/heads/main", edit{branch="main", min_approvals=3}) runs under #[serial_test::serial] via temp_env BUT_AGENT_HANDLE=admin
  THEN:  the call returns Ok; the WORKING-TREE .gitbutler/gates.toml now has min_approvals=3 for the main gate; AND branch_gates_read(&repo, "refs/heads/main") reports the COMMITTED main gate min_approvals=2 (read from the target-ref blob by the task's OWN reader — the edit is inert; the committed value is unchanged); ref_id(main) AFTER == BEFORE (no commit); the returned/printed caveat contains "takes effect once committed to the target branch"
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api branch_gates_update + the task's OWN branch_gates_read reading the target-ref committed gates.toml blob (NOT merge_gate — which exposes no public min_approvals reader) + real gix repo (committed target-ref config vs working-tree write) via but_testsupport::writable_scenario
  VERIFY: cargo test -p but-api branch_gates_update_writes_worktree_inert_until_committed

AC-2: Toggling protected OFF lands protected=false in the working tree while preserving the branch's [[gate]] requirement
  GIVEN: gates_governance_base: main is protected=true with a full [[gate]] requirement; BUT_AGENT_HANDLE=admin; clean working tree
  WHEN:  branch_gates_update(&repo, "refs/heads/main", edit{branch="main", protected=false}) runs as admin (the unprotect write path)
  THEN:  the rewritten working-tree gates.toml has [[branch]] main protected=false; AND main's [[gate]] requirement (min_approvals=2, distinct=true, groups=[code-reviewers,maintainers]) is STILL present (unprotecting does not strip the review requirement); the caveat is returned
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api branch_gates_update + real gix working-tree read-back parsing both the [[branch]] and [[gate]] arrays
  VERIFY: cargo test -p but-api branch_gates_update_unprotect_preserves_gate_requirement

AC-3: The writer LOSSLESSLY round-trips the full [[gate]] requirement set and every unrelated entry on a protection-only edit (no lossy drop)
  GIVEN: gates_governance_base: main protected=true with full [[gate]] main requirement; release protected=true with [[gate]] release requirement (min_approvals=1, groups=[maintainers]); BUT_AGENT_HANDLE=admin
  WHEN:  branch_gates_update(&repo, "refs/heads/main", edit{branch="main", protected=true}) runs as admin (a protection-flag write that touches ONLY the main [[branch]] entry)
  THEN:  the rewritten working-tree gates.toml STILL carries main's full [[gate]] requirement (min_approvals=2, distinct=true, groups=[code-reviewers,maintainers]) AND release's unrelated [[branch]] (protected=true) + [[gate]] (min_approvals=1, groups=[maintainers]) entries — i.e. only the targeted main [[branch]] entry's value could change; no requirement field is dropped
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api branch_gates_update + real gix working-tree read-back; assert on the parsed [[branch]] + [[gate]] wire shape (LOSSLESS), not a normalized GovConfig
  VERIFY: cargo test -p but-api branch_gates_update_round_trips_full_gate_schema_lossless

AC-4: A non-admin branch_gates_update is denied perm.denied (names administration:write) and writes nothing; a self-escalation gate-weakening is surfaced, not applied
  GIVEN: gates_governance_base: caller rust-implementer holds ["contents:write"] only (NO administration:write) at the target ref; the working-tree gates.toml captured byte-for-byte before the call
  WHEN:  branch_gates_update(&repo, "refs/heads/main", edit{branch="main", protected=false}) runs with BUT_AGENT_HANDLE=rust-implementer (a self-escalation attempt to weaken the gate that protects against the caller)
  THEN:  the call returns Err; classify_error(&err) yields Some(AdminWriteGateError) whose .code == "perm.denied"; the message contains "administration:write"; AND the working-tree .gitbutler/gates.toml is byte-for-byte UNCHANGED from before the call (the admin gate ran BEFORE any write)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api branch_gates_update composing enforce_administration_write_gate + real but-authz + real gix
  VERIFY: cargo test -p but-api branch_gates_update_non_admin_denied_writes_nothing

AC-5: branch_gates_read returns the committed gate set PLUS a pending signal computed from the working-tree-vs-target-ref diff
  GIVEN: gates_governance_base (committed main min_approvals=2), then an admin branch_gates_update(... min_approvals=3) has written min_approvals=3 to the working tree (uncommitted) — i.e. AC-1's post-state; BUT_AGENT_HANDLE=admin
  WHEN:  branch_gates_read(&repo, "refs/heads/main") runs as admin
  THEN:  the returned read shows the COMMITTED main gate (min_approvals=2, distinct=true, groups=[code-reviewers,maintainers], protected=true — read at the target ref) AND a pending=true signal because the working-tree min_approvals=3 differs from the committed min_approvals=2; on a clean working tree (no edit) pending=false
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api branch_gates_read reading BOTH the target-ref committed config and the working-tree file + real gix
  VERIFY: cargo test -p but-api branch_gates_read_returns_committed_set_with_pending_signal

AC-6: Admin branch_gates_update SETS require_distinct_from_author AND require_approval_from_group (the full editable field set lands losslessly)
  GIVEN: gates_governance_base: release is protected=true with [[gate]] release {min_approvals=1, require_approval_from_group=["maintainers"], require_distinct_from_author default false}; BUT_AGENT_HANDLE=admin; clean working tree
  WHEN:  branch_gates_update(&repo, "refs/heads/main", edit{branch="release", require_approval_from_group=["maintainers","code-reviewers"], require_distinct_from_author=true}) runs as admin (exercising the two fields no other AC writes)
  THEN:  the rewritten working-tree gates.toml [[gate]] release entry now has require_distinct_from_author=true AND require_approval_from_group=["maintainers","code-reviewers"] (exact list, exact values), re-loadable through the merge_gate-shaped GatesWire; release's min_approvals=1 is preserved; the caveat is returned
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api branch_gates_update + real gix working-tree read-back parsing the [[gate]] release entry through the merge_gate-shaped GatesWire
  VERIFY: cargo test -p but-api branch_gates_update_sets_distinct_and_required_groups

AC-7: branch_gates_update APPENDS a new [[branch]]+[[gate]] for a branch absent from gates.toml (preserving existing entries), and creates the file against an absent/empty gates.toml
  GIVEN: gates_governance_base: gates.toml has main+release entries but NO feature/x entry; BUT_AGENT_HANDLE=admin; SEPARATELY a second scenario whose working-tree .gitbutler/gates.toml is absent/empty
  WHEN:  branch_gates_update(&repo, "refs/heads/main", edit{branch="feature/x", protected=true, min_approvals=1, require_approval_from_group=["maintainers"]}) runs as admin against (case 1) the populated gates.toml and (case 2) an absent/empty gates.toml (the empty-state-then-add seeding path, mirroring CLI-001 perm_first_grant_seeds_principal)
  THEN:  case 1: the rewritten working-tree gates.toml APPENDS a new [[branch]] feature/x protected=true + [[gate]] feature/x min_approvals=1 groups=[maintainers] WHILE the pre-existing main+release [[branch]]/[[gate]] entries remain intact; case 2: the file is CREATED with exactly the new feature/x [[branch]]+[[gate]] entry
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api branch_gates_update + real gix working-tree read-back; populated-file APPEND case + absent/empty-file CREATE case via but_testsupport
  VERIFY: cargo test -p but-api branch_gates_update_appends_new_branch_and_creates_absent_file

AC-8: branch_gates_read is administration:read-scoped — a caller lacking administration:read is denied perm.denied (governance reconnaissance is gated)
  GIVEN: gates_governance_base: caller no-read-principal holds ["contents:write"] only (NO administration:read and NO administration:write) at the target ref; admin holds administration:read via administration:write
  WHEN:  branch_gates_read(&repo, "refs/heads/main") runs once as BUT_AGENT_HANDLE=admin (authorized) and once as BUT_AGENT_HANDLE=no-read-principal (the v1 decision: branch_gates_read enforces administration:read before returning any committed gate set)
  THEN:  as admin: returns Ok with the committed main gate set (min_approvals=2); as no-read-principal: returns Err whose classify_error(&err) yields Some(_) with .code == "perm.denied" naming "administration:read" — the gate set is NOT disclosed to an unauthorized caller
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api branch_gates_read composing an administration:read authorization check (the read-scope analog of enforce_administration_write_gate) + real but-authz + real gix
  VERIFY: cargo test -p but-api branch_gates_read_requires_administration_read

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): After admin branch_gates_update(min_approvals=3), the working-tree .gitbutler/gates.toml main gate reads min_approvals=3
    VERIFY: cargo test -p but-api branch_gates_update_writes_worktree_inert_until_committed
- TC-2 (-> AC-1): After the update branch_gates_read reports the COMMITTED (target-ref) main gate min_approvals=2 AND ref_id(refs/heads/main) is identical before and after (inert / no commit)
    VERIFY: cargo test -p but-api branch_gates_update_writes_worktree_inert_until_committed
- TC-3 (-> AC-1): The branch_gates_update result/printed caveat contains "takes effect once committed to the target branch"
    VERIFY: cargo test -p but-api branch_gates_update_writes_worktree_inert_until_committed
- TC-4 (-> AC-2): Unprotecting main (protected=false) lands protected=false AND main's [[gate]] requirement (min_approvals=2, distinct=true, groups=[code-reviewers,maintainers]) survives
    VERIFY: cargo test -p but-api branch_gates_update_unprotect_preserves_gate_requirement
- TC-5 (-> AC-3): A protection-only edit re-serializes BOTH the full main [[gate]] requirement AND the unrelated release [[branch]]/[[gate]] entries (lossless round-trip; no [[gate]] drop)
    VERIFY: cargo test -p but-api branch_gates_update_round_trips_full_gate_schema_lossless
- TC-6 (-> AC-3): Re-loading the rewritten file through the merge_gate-shaped GatesWire confirms min_approvals and require_approval_from_group survive (no silent weakening of the merge gate)
    VERIFY: cargo test -p but-api branch_gates_update_round_trips_full_gate_schema_lossless
- TC-7 (-> AC-4): A non-admin branch_gates_update returns Err; classify_error(&err) yields Some(AdminWriteGateError) whose .code == "perm.denied" and the message contains "administration:write"
    VERIFY: cargo test -p but-api branch_gates_update_non_admin_denied_writes_nothing
- TC-8 (-> AC-4): After the denied non-admin (self-escalation) branch_gates_update, the working-tree .gitbutler/gates.toml is byte-for-byte unchanged (gate ran before any write; control not flipped)
    VERIFY: cargo test -p but-api branch_gates_update_non_admin_denied_writes_nothing
- TC-9 (-> AC-5): branch_gates_read returns committed main min_approvals=2 with pending=true after an uncommitted min_approvals=3 edit; pending=false on a clean working tree
    VERIFY: cargo test -p but-api branch_gates_read_returns_committed_set_with_pending_signal
- TC-10 (-> AC-1): `pnpm build:sdk && pnpm format` regenerates packages/but-sdk/src/generated containing branch_gates_read and branch_gates_update commands/types, and the generated TS type-checks (no hand-edit)
    VERIFY: pnpm build:sdk && pnpm format && git diff --name-only packages/but-sdk/src/generated | grep -q . && grep -rq "branch_gates_update\|branchGatesUpdate" packages/but-sdk/src/generated
- TC-11 (-> AC-6): branch_gates_update(edit{branch=release, require_distinct_from_author=true, require_approval_from_group=["maintainers","code-reviewers"]}) lands require_distinct_from_author=true AND require_approval_from_group=["maintainers","code-reviewers"] on the release [[gate]] (re-loadable via the merge_gate-shaped GatesWire)
    VERIFY: cargo test -p but-api branch_gates_update_sets_distinct_and_required_groups
- TC-12 (-> AC-3): A protection-only edit re-serializes a file whose full [[branch]]+[[gate]] structure re-parses (via the merge_gate-shaped GatesWire) with branch.len() and gate.len() preserved and every [[gate]] field value identical (SEC-1 structural round-trip: gate.len() and the full set survive)
    VERIFY: cargo test -p but-api branch_gates_update_round_trips_full_gate_schema_lossless
- TC-13 (-> AC-7): branch_gates_update against a populated gates.toml APPENDS a new feature/x [[branch]]+[[gate]] while preserving the existing main+release entries (no clobber)
    VERIFY: cargo test -p but-api branch_gates_update_appends_new_branch_and_creates_absent_file
- TC-14 (-> AC-7): branch_gates_update against an absent/empty .gitbutler/gates.toml CREATES the file with exactly the new feature/x [[branch]]+[[gate]] entry (the empty-state-then-add seeding path)
    VERIFY: cargo test -p but-api branch_gates_update_appends_new_branch_and_creates_absent_file
- TC-15 (-> AC-8): branch_gates_read as admin returns Ok committed min_approvals=2; as no-read-principal (no administration:read) returns Err with classify_error(&err)=Some(_) .code==perm.denied naming administration:read (the read scope is gated, not a recon leak)
    VERIFY: cargo test -p but-api branch_gates_read_requires_administration_read

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01, CAP-CONFIG-01
provides:
  - branch_gates_read(&repo, target_ref) -> the committed gate set (per-branch protected + min_approvals + require_distinct_from_author + require_approval_from_group) read at the target ref, PLUS a pending signal computed from the working-tree-vs-target-ref diff
  - branch_gates_update(&repo, target_ref, edit) -> admin-gated inert-until-committed working-tree write of .gitbutler/gates.toml that LOSSLESSLY round-trips the full [[branch]] + [[gate]] table-array schema (the first persisted writer of gates.toml, mirroring CLI-001's permissions.toml writer)
  - a snake_case #[tauri::command] branch_gates_read / branch_gates_update wrapping the same but-api fns, surfaced through the regenerated packages/but-sdk (extending MGMT-IPC-004's SDK base)
consumes:
  - but_api::legacy::config_mutate::{enforce_administration_write_gate, classify_error, AdminWriteGateError} (the Sprint-02 AUTHZ-006 admin-write guard — COMPOSED, never re-implemented)
  - but_authz::{load_governance_config, governance_present, gates_path (NEW accessor mirroring permissions_path), Denial}
  - gix::Repository::workdir (the working-tree write target) + target-ref tree/blob read (the committed-config read)
  - but_testsupport::{writable_scenario, invoke_bash} for real-git scenario seeding + the day-one commit step
boundary_contracts:
  - CAP-AUTHZ-01: branch_gates_update authorizes administration:write (own ∪ groups, read at the target ref) via enforce_administration_write_gate BEFORE any write; a non-admin write is denied perm.denied and writes nothing; a self-escalation that weakens one's own gate is surfaced as the structured Denial, never optimistically applied. branch_gates_read is administration:read-or-self scoped per the api-design route table.
  - CAP-CONFIG-01: branch_gates_update writes inert-until-committed config to the WORKING TREE only — effectiveness comes from the next target-ref load_governance_config read; a working-tree gates.toml edit on a feature head does NOT change the target-ref protection/requirement decision. The writer LOSSLESSLY round-trips the full [[branch]] + [[gate]] schema the merge gate consumes (dropping min_approvals / require_approval_from_group / require_distinct_from_author is a CRITICAL lossy-round-trip failure).
  - Scope note (RUST-7): BE-004's self-escalation proof (AC-4) is the GATE-WEAKENING variant — a non-admin (contents:write only) trying to UNPROTECT the branch that gates them, denied perm.denied with the working tree byte-for-byte unchanged. The PERMISSION self-grant of administration:write is proven by the perm_/group_ path (Sprint-05 CLI-001 / Sprint-06a), NOT by BE-004 (gates.toml has no permission grants). So the CAP-AUTHZ-01 consumer-side self-escalation gate step maps to BOTH BE-004 (gate-weakening) and the perm path (permission self-grant), not to BE-004 alone.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-api/src/legacy/governance.rs (MODIFY — add branch_gates_read/branch_gates_update + the private gates.toml read-modify-write writer + the raw GatesWire/BranchWire/GateWire wire structs with #[derive(Serialize, Deserialize)] + the BranchGatesError/caveat constant; sited BESIDE CLI-001's perm_* fns)
  - crates/but-authz/src/config.rs (MODIFY — ADDITIVE ONLY: add `pub fn gates_path() -> &'static str` returning GATES_PATH; do NOT change loader/normalize semantics; do NOT widen BranchProtection or the loader's BranchWire)
  - crates/but-authz/src/lib.rs (MODIFY — re-export gates_path from the config pub use block)
  - crates/but-api/src/legacy/mod.rs (MODIFY — ensure governance module is declared; no-op if CLI-001 already added it)
  - crates/but-api/tests/branch_gates.rs (NEW — the PRIMARY but-api proofs AC-1..AC-5)
  - the desktop Tauri command file under crates/ that registers #[tauri::command] governance commands (MODIFY — register branch_gates_read/branch_gates_update beside the Sprint-06a perm_*/group_* commands; follow the existing snake_case #[tauri::command] convention) and src-tauri/capabilities/ allow-branch_gates_* entry (MODIFY — mirror the allow-perm_*/allow-group_* convention)
  - packages/but-sdk/src/generated/** (REGENERATE ONLY via `pnpm build:sdk && pnpm format` — NEVER hand-edit)
writeProhibited:
  - crates/but-api/src/legacy/config_mutate.rs — CONSUME-only (the AUTHZ-006 admin guard); compose enforce_administration_write_gate, do not fork a second admin check. If it lacks a needed accessor, FLAG it and add an additive helper in governance.rs.
  - crates/but-api/src/legacy/merge_gate.rs — CONSUME-only (Sprint-04 merge-requirement reader); READ its GatesWire/GateWire to learn the field set, but do NOT change its loader semantics or its read-at-target-ref authorization path
  - crates/but-authz/src/{authorize.rs, denial.rs, principal.rs, authority.rs} — the union/primitive layer is closed (you only ADD gates_path to config.rs)
  - the but-authz loader's read-at-target-ref authorization-path semantics — the target-ref read is the GATES-003 / CAP-CONFIG-01 invariant; the writer touches the WORKING TREE only, never the loader's decision path
  - crates/but-authz/tests/invariant_build_gates.rs — do NOT weaken/remove any existing honesty-grep pattern or ENFORCEMENT_PATH (if governance.rs gains a branch_gates authorization decision, the existing CLI-001 governance.rs coverage already applies — do not narrow it)
  - any gitbutler-* crate (no new gitbutler-* usage)
  - Any file not in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-api/src/legacy/merge_gate.rs [199-209, 308-343, 409-437] — [PRIMARY PATTERN for the schema] the EXACT full gates.toml wire shape the writer must round-trip: GatesWire{branch: Vec<BranchWire>, gate: Vec<GateWire>}, BranchWire{name, protected}, GateWire{branch, type, min_approvals, require_approval_from_group, require_distinct_from_author} with #[serde(deny_unknown_fields)]; normalize_gates shows how both arrays are consumed. Your writer owns its own raw struct set modeling THIS complete shape (add #[derive(Serialize, Deserialize)]) — do NOT modify merge_gate.rs's loader. NOTE (RUST-1): GatesWire/GateWire/BranchWire and MergeGovernanceConfig/ReviewRequirement/review_requirement_for/load_merge_governance_config are ALL PRIVATE — merge_gate exposes ONLY pub enforce_merge_gate + pub classify_error. There is NO public reader of committed min_approvals; AC-1's inert check uses the task's OWN branch_gates_read (committed-blob parse), never merge_gate.
2. .spec/prds/governance/tasks/sprint-05-cli-perm-group/CLI-001-perm-cli-verbs.md [5, 64-77, 287-298] — [PRIMARY PATTERN to mirror] CLI-001 is the permissions.toml analog: compose enforce_administration_write_gate, working-tree-only inert write via repo.workdir()+fs::write, value-preserving read-modify-write over RAW wire structs (NOT a normalized GovConfig), toml::to_string canonical serialization, the ref-pin caveat, and the gates_path() accessor pattern (mirror permissions_path).
3. crates/but-api/src/legacy/config_mutate.rs [1-44] — enforce_administration_write_gate(repo, target_ref) — reads target-ref cfg, resolves BUT_AGENT_HANDLE, authorizes AdministrationWrite; classify_error -> AdminWriteGateError{code,message}. COMPOSE this in branch_gates_update; never re-implement admin gating.
4. crates/but-api/tests/admin_write_guard.rs [150-176, 96-123] — the canonical write-worktree helper (repo.workdir()+std::fs::write — mirror for gates.toml), committed_blob_text for the byte-for-byte-unchanged assertion (AC-4), temp_env::with_var BUT_AGENT_HANDLE under #[serial_test::serial], writable_scenario + invoke_bash committing governance config at main. Mirror this file's shape for branch_gates.rs.
5. crates/but-authz/src/config.rs [8-9, 44-59, 195-217, 373-385, 438-443] — GATES_PATH literal (the gates_path() accessor wraps it); governance_present (the target-ref discriminator); BranchProtection models ONLY {protected} and normalize_gates discards the [[gate]] array — PROOF you must NOT round-trip GovConfig (it would silently drop every merge requirement, AC-3's negative control).

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- test -f crates/but-api/src/legacy/governance.rs && grep -q 'pub fn permissions_path' crates/but-authz/src/config.rs   -> Exit 0; CLI-001's governance.rs and but_authz::permissions_path accessor exist. If ABSENT, MGMT-BE-004 is BLOCKED on its Sprint-05 CLI-001 predecessor — do not start (every AC would fail to compile with no governance.rs to site into).
- cargo test -p but-api branch_gates_update_writes_worktree_inert_until_committed   -> Exit 0; working-tree gates.toml gains min_approvals=3, target-ref requirement still 2, ref_id(main) before==after, caveat printed
- cargo test -p but-api branch_gates_update_unprotect_preserves_gate_requirement   -> Exit 0; protected=false lands, main [[gate]] requirement (min_approvals=2/distinct/groups) survives
- cargo test -p but-api branch_gates_update_round_trips_full_gate_schema_lossless   -> Exit 0; main [[gate]] + unrelated release [[branch]]/[[gate]] entries survive a protection-only edit; no [[gate]] drop
- cargo test -p but-api branch_gates_update_non_admin_denied_writes_nothing   -> Exit 0; perm.denied naming administration:write; working-tree gates.toml byte-for-byte unchanged (control not flipped)
- cargo test -p but-api branch_gates_read_returns_committed_set_with_pending_signal   -> Exit 0; committed min_approvals=2 + pending=true on uncommitted edit; pending=false on clean tree
- cargo check -p but-authz --all-targets   -> Exit 0; gates_path() accessor + re-export compile; no loader semantic change
- cargo test -p but-authz invariant_build_gates   -> Exit 0; no role-label/human-vs-AI branching in the governance authorization path
- cargo clippy -p but-authz -p but-api --all-targets   -> Exit 0
- cargo fmt --check   -> Exit 0
- pnpm build:sdk && pnpm format   -> Exit 0; packages/but-sdk/src/generated contains branch_gates_read/branch_gates_update; generated TS type-checks; no hand-edit
- ! grep -q '\.gitbutler/gates\.toml' crates/but-api/src/legacy/governance.rs   -> Exit 0; governance.rs resolves the path via but_authz::gates_path(), never a re-derived ".gitbutler/gates.toml" literal (SEC-8 — single source of truth for the governance file paths).
- cargo test -p but-api branch_gates_update_sets_distinct_and_required_groups   -> Exit 0; release [[gate]] gains require_distinct_from_author=true + require_approval_from_group=[maintainers,code-reviewers] losslessly
- cargo test -p but-api branch_gates_update_appends_new_branch_and_creates_absent_file   -> Exit 0; feature/x [[branch]]+[[gate]] appended preserving main+release; absent gates.toml created with the new entry
- cargo test -p but-api branch_gates_read_requires_administration_read   -> Exit 0; admin Ok(committed min_approvals=2); no-read-principal perm.denied naming administration:read

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references:
  - .spec/prds/governance/10-technical-requirements/04-api-design.md:82-97 (the branch_gates_read=administration:read / branch_gates_update=administration:write Tauri command rows + the gate fields protected/min_approvals/distinct/required_groups)
  - .spec/prds/governance/10-technical-requirements/03-data-schema.md:62-77 (the canonical gates.toml [[branch]] + [[gate]] schema)
  - .spec/prds/governance/10-technical-requirements/08-capability-chains.md (CAP-AUTHZ-01 + CAP-CONFIG-01 boundary contracts)
notes:
  - Target-ref resolution: branch_gates_read/update take an explicit target_ref: &str (e.g. "refs/heads/main"); the future Tauri command resolves it from the WORKSPACE TARGET (not HEAD), and the but-api tests pass "refs/heads/main" directly (the fixture commits governance at main).
  - Schema ownership: the writer needs its OWN raw GatesWire/BranchWire/GateWire (matching merge_gate.rs:411-437) with #[derive(Serialize, Deserialize)] + #[serde(deny_unknown_fields)] — it cannot reuse but-authz's BranchProtection (loses the whole [[gate]] array) nor merge_gate.rs's private structs (not pub). Re-serialize via toml::to_string (canonical TOML; VALUE-preserving, not byte-verbatim) — no must_observe asserts byte-verbatim on a SUCCESSFUL write; the only byte-for-byte assertion is on the UNCHANGED file (denied write AC-4).
  - Pending signal (AC-5): branch_gates_read loads the committed gate set (target-ref blob) AND parses the working-tree file; a field that differs between them renders pending=true. A clean working tree (no diff) yields pending=false. Return a structured value the test inspects and the UI's working-tree-vs-target-ref pending derivation (T-MGMT-035) consumes.
  - Env handling: branch_gates_update resolves the caller via resolve_principal_from_env (BUT_AGENT_HANDLE) inside enforce_administration_write_gate, so the but-api tests use temp_env::with_var("BUT_AGENT_HANDLE", Some(...), ...) under #[serial_test::serial]; temp-env + serial_test are already in but-api dev-deps (Cargo.toml — FLAG only if a build surfaces otherwise).
  - Sprint 06a reuse: site branch_gates_read/update with Result signatures the Tauri commands wrap directly — structured Ok payload + Denial/BranchGatesError on err, classifiable by config_mutate::classify_error (reuse perm.denied/config.invalid codes; add a new code only if genuinely needed).
  - Inert-check reader (RUST-1): merge_gate.rs exposes NO public reader of the committed min_approvals at a target ref (MergeGovernanceConfig/ReviewRequirement/review_requirement_for/load_merge_governance_config/GatesWire are all private). AC-1's inert proof therefore reads the committed value via the task's OWN branch_gates_read(&repo, target_ref) (target-ref committed blob) — or a direct committed-blob parse mirroring admin_write_guard.rs:166-175 committed_blob_text — plus the ref_id(main) before==after assertion (the strongest, gix-grounded inert proof). Do NOT call merge_gate for a value read.
  - Desktop identity (SEC-3): the branch_gates_* Tauri command resolves the human fleet-owner identity (UserService / T-MGMT-042 / MGMT-IPC-003) — BUT_AGENT_HANDLE is unset in the desktop process. The but-api fn keeps its env-principal resolution (resolve_principal_from_env) exercised by the rust tests via temp_env BUT_AGENT_HANDLE; the Tauri layer maps the desktop human identity onto that principal axis.
  - Full field set (RUST-2): the edit{} struct MUST model the COMPLETE editable field set per api-design.md:94 — branch, protected, min_approvals, require_distinct_from_author, AND require_approval_from_group — not just protected+min_approvals. AC-6 proves distinct+groups are SET (not merely preserved).
  - Add-new-branch / create-on-absent (RUST-3): branch_gates_update for a branch absent from gates.toml APPENDS a new [[branch]]+[[gate]] (preserving existing entries); against an absent/empty .gitbutler/gates.toml it CREATES the file — the empty-state-then-add seeding path, mirroring CLI-001 perm_first_grant_seeds_principal (AC-7).
pattern: thin but-api governance producer that composes the AUTHZ-006 admin guard, then read-modify-writes the WORKING-TREE gates.toml LOSSLESSLY round-tripping the full [[branch]] + [[gate]] schema via raw serde wire structs + toml::to_string, inert until committed; branch_gates_read reads BOTH the committed target-ref set and the working-tree file to compute the pending diff
pattern_source: permissions.toml analog = .spec/.../sprint-05-cli-perm-group/CLI-001-perm-cli-verbs.md (the writer pattern to mirror); admin guard = crates/but-api/src/legacy/config_mutate.rs:18 (compose, do not fork); full gates.toml schema = crates/but-api/src/legacy/merge_gate.rs:409-437; working-tree write = crates/but-api/tests/admin_write_guard.rs:158-164; target-ref read + path accessor = crates/but-authz/src/config.rs:8-9,44-59
anti_pattern: round-tripping through but-authz GovConfig (models only {name, protected} — silently DROPS the entire [[gate]] review-requirement array: the CRITICAL lossy bug AC-3 catches); committing/staging the production write (breaks inert-until-committed — AC-1's ref-unchanged assertion fails); authoring a second authorize(AdministrationWrite) instead of composing enforce_administration_write_gate; running the write BEFORE the admin gate (AC-4's byte-unchanged fails); optimistically applying a self-escalation gate-weakening (AC-4's control-not-flipped fails); re-deriving the ".gitbutler/gates.toml" literal instead of gates_path(); treating the working tree as committed in branch_gates_read (no pending signal — AC-5 fails); pointing the inert-check at a merge_gate public reader that does NOT exist (merge_gate exposes only enforce_merge_gate+classify_error; the committed min_approvals is read by the task's OWN branch_gates_read / committed-blob parse — RUST-1); hand-editing packages/but-sdk/src/generated

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agent: implementer=rust-implementer | reviewer=rust-reviewer
rationale: Net-new TOML read-modify-write writer over a but-api governance boundary: additive #[derive(Serialize)] on raw serde wire structs modeling the FULL gates.toml shape ([[branch]] + [[gate]]), gix working-tree write (inert until committed), target-ref blob read for the pending diff, composition of the Sprint-02 enforce_administration_write_gate, structured Denial classification, and a Tauri command + SDK regen. These are exactly rust-implementer competencies and mirror CLI-001's permissions.toml writer; rust-reviewer adversarially validates the round-trip is lossless (no [[gate]] drop), the write is inert (never committed; ref unchanged), the admin gate runs before any write, and self-escalation is denied.
coding_standards: crates/AGENTS.md (Result<T,E> + anyhow::Context; but_error::Code for consumer-facing classification; gix over git2 for new repo logic; acquire locks at top-level boundaries), crates/WORKSPACE_MODEL.md (prefer commit IDs/refs at API boundaries; read governance config at the target ref via gix blob read, never the working tree, for authorization), crates/but-api/src/legacy (nearby patterns: gix workdir write, target-ref blob read, ref_id before/after, Denial/AdminWriteGateError classify_error, but-testsupport writable_scenario + invoke_bash, temp_env + serial_test for BUT_AGENT_HANDLE), RULES.md: but-api is THE API boundary; transport DTOs convert to domain types before calling lower crates; preserve existing but-api macro/transport/serialization patterns; after changing Rust APIs exposed via but-sdk, run pnpm build:sdk && pnpm format (never hand-edit generated)

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: Sprint 05 CLI-001 (the permissions.toml writer pattern + crates/but-api/src/legacy/governance.rs which this task is sited beside, plus the gates_path-analog permissions_path accessor pattern); Sprint 04 (the merge-requirement schema in crates/but-api/src/legacy/merge_gate.rs — the [[branch]]+[[gate]] field set the writer must round-trip losslessly); Sprint 06a MGMT-IPC-004 (the SDK regen base this task's command/SDK delta extends); Sprint 06a MGMT-IPC-003 (the v1 human-fleet-owner identity shim — UserService, T-MGMT-042 — that the desktop branch_gates_* Tauri command resolves, since BUT_AGENT_HANDLE is unset in the desktop process; SEC-3)
Blocks:     MGMT-UI-009 (BranchGatesList consumes the branch_gates_read/branch_gates_update SDK)
```
</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "MGMT-BE-004",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "gates_governance_base": {
      "description": "Real-git scenario via but_testsupport::writable_scenario(\"checkout-head-info\"). Target ref refs/heads/main carries a committed .gitbutler/permissions.toml where `admin` holds administration:write and `rust-implementer` holds [\"contents:write\"] (NO administration:write); and a committed .gitbutler/gates.toml with a FULL requirement set: [[branch]] name=\"main\" protected=true; [[branch]] name=\"release\" protected=true; [[gate]] branch=\"main\" type=\"review\" min_approvals=2 require_distinct_from_author=true require_approval_from_group=[\"code-reviewers\",\"maintainers\"]; [[gate]] branch=\"release\" type=\"review\" min_approvals=1 require_approval_from_group=[\"maintainers\"]. Working tree starts clean (matches the committed blobs). Seeded via a REAL entrypoint: invoke_bash writes the files and git-commits them at main (the committed-config-at-main pattern from crates/but-api/tests/admin_write_guard.rs:96-123). Capture committed_blob_text(repo, but_authz::gates_path()) and ref_id(repo, \"refs/heads/main\") BEFORE any write for the inert / byte-unchanged assertions.",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"checkout-head-info\");",
        "invoke_bash on main: mkdir -p .gitbutler; write .gitbutler/permissions.toml with [[principal]] id=\"admin\" permissions=[\"administration:write\",\"merge\"], [[principal]] id=\"rust-implementer\" permissions=[\"contents:write\"], [[principal]] id=\"no-read-principal\" permissions=[\"contents:write\"] (NO administration:read), plus [[group]] name=\"code-reviewers\" permissions=[\"reviews:write\"] and [[group]] name=\"maintainers\" permissions=[\"merge\"];",
        "invoke_bash on main: write .gitbutler/gates.toml with [[branch]] name=\"main\" protected=true; [[branch]] name=\"release\" protected=true; [[gate]] branch=\"main\" type=\"review\" min_approvals=2 require_distinct_from_author=true require_approval_from_group=[\"code-reviewers\",\"maintainers\"]; [[gate]] branch=\"release\" type=\"review\" min_approvals=1 require_approval_from_group=[\"maintainers\"]; then git add .gitbutler/permissions.toml .gitbutler/gates.toml && git commit -m \"governance config\";",
        "Capture committed_blob_text(repo, but_authz::gates_path()) and ref_id(repo, \"refs/heads/main\") BEFORE any write."
      ]
    },
    "gates_governance_absent_file": {
      "description": "Real-git scenario via but_testsupport::writable_scenario(\"checkout-head-info\") where the committed target ref carries .gitbutler/permissions.toml (admin holds administration:write; no-read-principal/rust-implementer hold contents:write) so authorization succeeds for admin, BUT the working-tree .gitbutler/gates.toml is ABSENT (never written) \u2014 the empty-state-then-add seeding path. branch_gates_update against this fixture must CREATE the file. Seeded via invoke_bash committing permissions.toml at main WITHOUT a gates.toml working-tree file.",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"checkout-head-info\");",
        "invoke_bash on main: mkdir -p .gitbutler; write .gitbutler/permissions.toml with [[principal]] id=\"admin\" permissions=[\"administration:write\",\"merge\"], [[principal]] id=\"rust-implementer\" permissions=[\"contents:write\"], [[principal]] id=\"no-read-principal\" permissions=[\"contents:write\"], [[group]] name=\"maintainers\" permissions=[\"merge\"]; git add .gitbutler/permissions.toml && git commit -m \"governance permissions only\";",
        "do NOT create a working-tree .gitbutler/gates.toml (it is absent \u2014 branch_gates_update must create it)."
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN gates_governance_base: refs/heads/main has committed gates.toml with [[branch]] main protected=true and the full [[gate]] main requirement (min_approvals=2, distinct=true, groups=[code-reviewers,maintainers]); BUT_AGENT_HANDLE=admin; clean working tree; ref_id(main) captured WHEN branch_gates_update(&repo, \"refs/heads/main\", edit{branch=\"main\", min_approvals=3}) runs under #[serial_test::serial] via temp_env BUT_AGENT_HANDLE=admin THEN the call returns Ok; the WORKING-TREE .gitbutler/gates.toml now has min_approvals=3 for the main gate; AND branch_gates_read(&repo, \"refs/heads/main\") reports the COMMITTED main gate min_approvals=2 (read from the target-ref blob by the task's OWN reader \u2014 the edit is inert; the committed value is unchanged); ref_id(main) AFTER == BEFORE (no commit); the returned/printed caveat contains \"takes effect once committed to the target branch\"",
      "verify": "cargo test -p but-api branch_gates_update_writes_worktree_inert_until_committed",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api branch_gates_update + real gix working-tree write + the task's OWN branch_gates_read target-ref committed-blob read",
        "negative_control": {
          "would_fail_if": [
            "the writer wrote nothing \u2014 the working-tree gates.toml would lack min_approvals=3 (a no-op stub returning Ok is caught by reading the file back)",
            "the writer COMMITTED the edit \u2014 branch_gates_read's committed (target-ref) min_approvals would then read 3, breaking the inert assertion, and ref_id(main) would change",
            "the writer read/decided against the working tree \u2014 branch_gates_read's committed value must_not_observe min_approvals=3 would fail",
            "branch_gates_read read the working tree as committed (a disconnected/static reader) \u2014 it would report committed min_approvals=3 with no committed-vs-worktree distinction",
            "the ref-pin caveat string were absent (silent success without warning the operator)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "gates_governance_base",
            "action": {
              "actor": "ci",
              "steps": [
                "capture ref_id(&repo, \"refs/heads/main\") BEFORE the write",
                "temp_env::with_var(\"BUT_AGENT_HANDLE\", Some(\"admin\"), ...) under #[serial_test::serial]",
                "branch_gates_update(&repo, \"refs/heads/main\", edit{branch=\"main\", min_approvals=3})",
                "read the working-tree .gitbutler/gates.toml back and parse it",
                "branch_gates_read(&repo, \"refs/heads/main\") and inspect its COMMITTED (target-ref) main gate value",
                "capture ref_id(&repo, \"refs/heads/main\") AFTER the write"
              ]
            },
            "end_state": {
              "must_observe": [
                "`branch_gates_update` returns `Ok`",
                "the working-tree `.gitbutler/gates.toml` main gate has `min_approvals = 3`",
                "the result/printed caveat contains `\"takes effect once committed to the target branch\"`",
                "`branch_gates_read(\"refs/heads/main\")` reports the COMMITTED main gate `min_approvals = 2` (target-ref blob)",
                "`ref_id(refs/heads/main)` AFTER == the `ref_id` captured BEFORE"
              ],
              "must_not_observe": [
                "`branch_gates_read` reporting the COMMITTED main gate `min_approvals = 3` (edit must be inert until committed)",
                "the target ref `refs/heads/main` HEAD sha changing (no commit performed)",
                "`branch_gates_read` returning an `empty`/`none` committed gate set for main while the working-tree edit is treated as the `unchanged` committed truth"
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
      "description": "GIVEN gates_governance_base: main is protected=true with a full [[gate]] requirement; BUT_AGENT_HANDLE=admin; clean working tree WHEN branch_gates_update(&repo, \"refs/heads/main\", edit{branch=\"main\", protected=false}) runs as admin (the unprotect write path) THEN the rewritten working-tree gates.toml has [[branch]] main protected=false; AND main's [[gate]] requirement (min_approvals=2, distinct=true, groups=[code-reviewers,maintainers]) is STILL present (unprotecting does not strip the review requirement); the caveat is returned",
      "verify": "cargo test -p but-api branch_gates_update_unprotect_preserves_gate_requirement",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api branch_gates_update + real gix working-tree read-back",
        "negative_control": {
          "would_fail_if": [
            "the writer wrote nothing \u2014 main would stay protected=true (a no-op stub leaves the start state, which this excludes)",
            "the writer dropped main's [[gate]] requirement while flipping protected (lossy unprotect) \u2014 min_approvals/groups would be absent",
            "the writer normalized through GovConfig (which models only {name, protected}) \u2014 the [[gate]] array would be gone entirely"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "gates_governance_base",
            "action": {
              "actor": "ci",
              "steps": [
                "temp_env BUT_AGENT_HANDLE=admin under #[serial_test::serial]",
                "branch_gates_update(&repo, \"refs/heads/main\", edit{branch=\"main\", protected=false})",
                "read the rewritten working-tree .gitbutler/gates.toml as raw text + parse both arrays"
              ]
            },
            "end_state": {
              "must_observe": [
                "the [[branch]] main entry now has `protected = false`",
                "the [[gate]] main entry still has `min_approvals = 2`",
                "the [[gate]] main entry still has `require_distinct_from_author = true`",
                "the [[gate]] main entry still has `require_approval_from_group = [\"code-reviewers\", \"maintainers\"]`"
              ],
              "must_not_observe": [
                "the [[branch]] main entry still reading `protected = true` (the toggle did not write)",
                "the [[gate]] main entry missing from the rewritten file (lossy unprotect)",
                "`min_approvals = 0` or an empty `require_approval_from_group = []` for main (the requirement silently dropped)"
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
      "description": "GIVEN gates_governance_base: main protected=true with full [[gate]] main requirement; release protected=true with [[gate]] release requirement (min_approvals=1, groups=[maintainers]); BUT_AGENT_HANDLE=admin WHEN branch_gates_update(&repo, \"refs/heads/main\", edit{branch=\"main\", protected=true}) runs as admin (a protection-flag write that touches ONLY the main [[branch]] entry) THEN the rewritten working-tree gates.toml STILL carries main's full [[gate]] requirement (min_approvals=2, distinct=true, groups=[code-reviewers,maintainers]) AND release's unrelated [[branch]] (protected=true) + [[gate]] (min_approvals=1, groups=[maintainers]) entries \u2014 i.e. only the targeted main [[branch]] entry's value could change; no requirement field is dropped",
      "verify": "cargo test -p but-api branch_gates_update_round_trips_full_gate_schema_lossless",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api branch_gates_update + real gix working-tree read-back parsing the full schema",
        "negative_control": {
          "would_fail_if": [
            "would fail if a GovConfig round-trip DROPS the [[gate]] array (lossy/stub writer): re-serializing through but-authz GovConfig (which models only {name, protected}) would DROP the entire [[gate]] array (main AND release requirements) \u2014 the CRITICAL lossy/dropped-writer bug",
            "the writer modeled only [[branch]] in its own wire struct and re-serialized \u2014 every [[gate]] requirement would vanish, silently weakening the merge gate to no review requirement",
            "the unrelated release [[branch]]/[[gate]] entries were dropped by a rewrite that only kept the targeted branch",
            "min_approvals defaulted to 0 or require_approval_from_group to [] on re-serialize (a partial/lossy wire struct)"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "gates_governance_base",
            "action": {
              "actor": "ci",
              "steps": [
                "temp_env BUT_AGENT_HANDLE=admin under #[serial_test::serial]",
                "branch_gates_update(&repo, \"refs/heads/main\", edit{branch=\"main\", protected=true})",
                "read the rewritten working-tree .gitbutler/gates.toml + parse BOTH the [[branch]] and [[gate]] arrays",
                "additionally load the rewritten file through the merge_gate-shaped GatesWire to confirm min_approvals/groups survive",
                "assert the parsed GatesWire has gate.len()==2 and branch.len()==2 (full-structure equality: no [[gate]]/[[branch]] entry added or dropped) and every [[gate]] field value is identical to the start fixture"
              ]
            },
            "end_state": {
              "must_observe": [
                "the main [[gate]] still reads `min_approvals = 2`",
                "the main [[gate]] still reads `require_approval_from_group = [\"code-reviewers\", \"maintainers\"]`",
                "the release [[branch]] still reads `protected = true`",
                "the release [[gate]] still reads `min_approvals = 1` with `require_approval_from_group = [\"maintainers\"]`",
                "the parsed GatesWire has `gate.len() == 2` and `branch.len() == 2` (the full [[branch]]+[[gate]] set re-parses equal \u2014 no entry added or dropped)"
              ],
              "must_not_observe": [
                "the [[gate]] array absent from the rewritten file (lossy GovConfig round-trip)",
                "main's `min_approvals` reading `0` or `require_approval_from_group` reading `[]` (requirement silently dropped)",
                "the release branch/gate entries missing (unrelated entries lost)",
                "the parsed `gate.len()` reading `0` or `1` (a [[gate]] entry silently dropped, leaving the merge requirement `empty`)"
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
      "description": "GIVEN gates_governance_base: caller rust-implementer holds [\"contents:write\"] only (NO administration:write) at the target ref; the working-tree gates.toml captured byte-for-byte before the call WHEN branch_gates_update(&repo, \"refs/heads/main\", edit{branch=\"main\", protected=false}) runs with BUT_AGENT_HANDLE=rust-implementer (a self-escalation attempt to weaken the gate that protects against the caller) THEN the call returns Err; classify_error(&err) yields Some(AdminWriteGateError) whose .code == \"perm.denied\"; the message contains \"administration:write\"; AND the working-tree .gitbutler/gates.toml is byte-for-byte UNCHANGED from before the call (the admin gate ran BEFORE any write)",
      "verify": "cargo test -p but-api branch_gates_update_non_admin_denied_writes_nothing",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api branch_gates_update + real enforce_administration_write_gate + real gix",
        "negative_control": {
          "would_fail_if": [
            "a stub always wrote / never gated \u2014 guarded by asserting the working-tree file is byte-for-byte unchanged AND classify_error.code==\"perm.denied\"",
            "the writer ran BEFORE the gate (fail-open ordering) \u2014 the file would change, which the byte-for-byte-unchanged assertion catches",
            "the denial fired for a non-authorization reason (the message must name administration:write, proving the admin guard, not a generic error)",
            "the control were optimistically applied (the gate would already be protected=false in the working tree)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "gates_governance_base",
            "action": {
              "actor": "ci",
              "steps": [
                "capture the working-tree .gitbutler/gates.toml bytes BEFORE the call",
                "temp_env::with_var(\"BUT_AGENT_HANDLE\", Some(\"rust-implementer\"), ...) under #[serial_test::serial]",
                "branch_gates_update(&repo, \"refs/heads/main\", edit{branch=\"main\", protected=false})",
                "classify the returned error via config_mutate::classify_error(&err) and unwrap the Some(AdminWriteGateError) (mirror admin_write_guard.rs:150-156 classified_error helper)",
                "re-read the working-tree .gitbutler/gates.toml bytes AFTER the call"
              ]
            },
            "end_state": {
              "must_observe": [
                "`branch_gates_update` returns `Err`",
                "`config_mutate::classify_error(&err)` yields `Some(AdminWriteGateError)` whose `.code == \"perm.denied\"`",
                "the denial message contains `\"administration:write\"`",
                "the working-tree `.gitbutler/gates.toml` bytes AFTER == the bytes captured BEFORE"
              ],
              "must_not_observe": [
                "the working-tree gates.toml main entry reading `protected = false` (the self-escalation was optimistically applied)",
                "`branch_gates_update` returning `Ok` for a non-admin caller (fail-open)",
                "any change to the working-tree gates.toml bytes (a write occurring before/despite the denial)",
                "the working-tree gates.toml has `0` byte changes (unchanged) but the denial silently allowing a later write (the file must remain byte-for-byte unchanged \u2014 `0` bytes differ \u2014 with `none` of the edit applied)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN gates_governance_base (committed main min_approvals=2), then an admin branch_gates_update(... min_approvals=3) has written min_approvals=3 to the working tree (uncommitted) \u2014 i.e. AC-1's post-state; BUT_AGENT_HANDLE=admin WHEN branch_gates_read(&repo, \"refs/heads/main\") runs as admin THEN the returned read shows the COMMITTED main gate (min_approvals=2, distinct=true, groups=[code-reviewers,maintainers], protected=true \u2014 read at the target ref) AND a pending=true signal because the working-tree min_approvals=3 differs from the committed min_approvals=2; on a clean working tree (no edit) pending=false",
      "verify": "cargo test -p but-api branch_gates_read_returns_committed_set_with_pending_signal",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api branch_gates_read + real gix target-ref blob + working-tree read",
        "negative_control": {
          "would_fail_if": [
            "branch_gates_read read the working tree as committed \u2014 it would return min_approvals=3 with pending=false (no diff signal)",
            "branch_gates_read ignored the working tree entirely \u2014 pending would always be false even with an uncommitted edit present",
            "a static fixture returned a hardcoded gate set \u2014 the pending signal would not track the real committed-vs-working diff (guarded by asserting committed=2 AND pending=true)"
          ]
        },
        "evidence": {
          "artifact_type": "api_response",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "gates_governance_base",
            "action": {
              "actor": "ci",
              "steps": [
                "temp_env BUT_AGENT_HANDLE=admin under #[serial_test::serial]",
                "branch_gates_update(&repo, \"refs/heads/main\", edit{branch=\"main\", min_approvals=3}) (writes the working tree, no commit)",
                "branch_gates_read(&repo, \"refs/heads/main\")",
                "inspect the returned committed gate set + pending signal",
                "separately, on a fresh clean checkout, branch_gates_read and assert pending=false"
              ]
            },
            "end_state": {
              "must_observe": [
                "the returned main gate's COMMITTED `min_approvals == 2`",
                "the returned main gate's `require_approval_from_group == [\"code-reviewers\", \"maintainers\"]`",
                "a `pending == true` signal for main (working-tree min_approvals=3 differs from committed 2)",
                "on a clean working tree, `pending == false`"
              ],
              "must_not_observe": [
                "the returned committed `min_approvals == 3` (working tree wrongly treated as committed)",
                "`pending == false` while an uncommitted working-tree edit is present (diff signal missing)",
                "the returned committed gate set being `empty` / `0` gates for main (a `blank`/`none` read returning no committed requirement, leaving the diff `unchanged`)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-6",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN gates_governance_base: release is protected=true with [[gate]] release {min_approvals=1, require_approval_from_group=[\"maintainers\"], require_distinct_from_author default false}; BUT_AGENT_HANDLE=admin; clean working tree WHEN branch_gates_update(&repo, \"refs/heads/main\", edit{branch=\"release\", require_approval_from_group=[\"maintainers\",\"code-reviewers\"], require_distinct_from_author=true}) runs as admin (exercising the two fields no other AC writes) THEN the rewritten working-tree gates.toml [[gate]] release entry now has require_distinct_from_author=true AND require_approval_from_group=[\"maintainers\",\"code-reviewers\"] (exact list, exact values), re-loadable through the merge_gate-shaped GatesWire; release's min_approvals=1 is preserved; the caveat is returned",
      "verify": "cargo test -p but-api branch_gates_update_sets_distinct_and_required_groups",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api branch_gates_update + real gix working-tree read-back of the full [[gate]] field set",
        "negative_control": {
          "would_fail_if": [
            "the writer no-oped on require_distinct_from_author / require_approval_from_group edits (a partial writer that only handles protected/min_approvals) \u2014 the release [[gate]] would keep require_distinct_from_author=false and require_approval_from_group=[\"maintainers\"], failing must_observe",
            "the edit{} struct modeled only protected+min_approvals (missing the distinct/groups fields) \u2014 the values would never reach the file",
            "a stub returned Ok without writing \u2014 the release [[gate]] would be unchanged (the start signature), excluded by must_not_observe",
            "require_approval_from_group were written as an empty list [] (a dropped/static field)"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "gates_governance_base",
            "action": {
              "actor": "ci",
              "steps": [
                "temp_env BUT_AGENT_HANDLE=admin under #[serial_test::serial]",
                "branch_gates_update(&repo, \"refs/heads/main\", edit{branch=\"release\", require_approval_from_group=[\"maintainers\",\"code-reviewers\"], require_distinct_from_author=true})",
                "read the rewritten working-tree .gitbutler/gates.toml + parse the [[gate]] release entry through the merge_gate-shaped GatesWire"
              ]
            },
            "end_state": {
              "must_observe": [
                "the [[gate]] release entry now has `require_distinct_from_author = true`",
                "the [[gate]] release entry now has `require_approval_from_group = [\"maintainers\", \"code-reviewers\"]`",
                "the [[gate]] release entry still has `min_approvals = 1`",
                "the result/printed caveat contains `\"takes effect once committed to the target branch\"`"
              ],
              "must_not_observe": [
                "the [[gate]] release entry still reading `require_distinct_from_author = false` (the distinct edit no-oped)",
                "the [[gate]] release entry still reading `require_approval_from_group = [\"maintainers\"]` (the groups edit no-oped \u2014 unchanged from start)",
                "an `empty` `require_approval_from_group = []` for release (the field silently dropped to `none`)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-7",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN gates_governance_base: gates.toml has main+release entries but NO feature/x entry; BUT_AGENT_HANDLE=admin; SEPARATELY a second scenario whose working-tree .gitbutler/gates.toml is absent/empty WHEN branch_gates_update(&repo, \"refs/heads/main\", edit{branch=\"feature/x\", protected=true, min_approvals=1, require_approval_from_group=[\"maintainers\"]}) runs as admin against (case 1) the populated gates.toml and (case 2) an absent/empty gates.toml (the empty-state-then-add seeding path, mirroring CLI-001 perm_first_grant_seeds_principal) THEN case 1: the rewritten working-tree gates.toml APPENDS a new [[branch]] feature/x protected=true + [[gate]] feature/x min_approvals=1 groups=[maintainers] WHILE the pre-existing main+release [[branch]]/[[gate]] entries remain intact; case 2: the file is CREATED with exactly the new feature/x [[branch]]+[[gate]] entry",
      "verify": "cargo test -p but-api branch_gates_update_appends_new_branch_and_creates_absent_file",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api branch_gates_update against a populated gates.toml (append) and an absent gates.toml (create)",
        "negative_control": {
          "would_fail_if": [
            "the writer only edited EXISTING entries and silently dropped an edit for an absent branch \u2014 feature/x would never appear (the add-new-branch no-op)",
            "the append clobbered the existing main+release entries (a rewrite that only kept the targeted branch) \u2014 main/release would be missing",
            "against an absent/empty gates.toml the writer errored or wrote nothing instead of creating the file \u2014 feature/x would be absent and the file would stay empty/`none`",
            "a stub returned Ok without writing \u2014 feature/x would be absent (the start signature excluded by must_not_observe)"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "gates_governance_base",
            "action": {
              "actor": "ci",
              "steps": [
                "temp_env BUT_AGENT_HANDLE=admin under #[serial_test::serial]",
                "branch_gates_update(&repo, \"refs/heads/main\", edit{branch=\"feature/x\", protected=true, min_approvals=1, require_approval_from_group=[\"maintainers\"]}) against the populated gates.toml",
                "read the rewritten working-tree .gitbutler/gates.toml + parse both arrays"
              ]
            },
            "end_state": {
              "must_observe": [
                "a NEW [[branch]] entry `name = \"feature/x\"` with `protected = true`",
                "a NEW [[gate]] entry `branch = \"feature/x\"` with `min_approvals = 1` and `require_approval_from_group = [\"maintainers\"]`",
                "the pre-existing [[branch]] main entry still present (`protected = true`)",
                "the pre-existing [[gate]] release entry still present (`min_approvals = 1`)"
              ],
              "must_not_observe": [
                "the rewritten file missing the feature/x [[branch]]/[[gate]] entries (the add-new-branch path no-oped)",
                "the pre-existing main/release entries dropped by the append (only the targeted branch kept)",
                "feature/x having `min_approvals = 0` or an `empty` `require_approval_from_group = []` (a partial append)"
              ]
            }
          },
          {
            "start_ref": "gates_governance_absent_file",
            "action": {
              "actor": "ci",
              "steps": [
                "temp_env BUT_AGENT_HANDLE=admin under #[serial_test::serial]",
                "branch_gates_update(&repo, \"refs/heads/main\", edit{branch=\"feature/x\", protected=true, min_approvals=1, require_approval_from_group=[\"maintainers\"]}) against an absent/empty working-tree .gitbutler/gates.toml",
                "read the CREATED working-tree .gitbutler/gates.toml + parse both arrays"
              ]
            },
            "end_state": {
              "must_observe": [
                "the file now EXISTS at `.gitbutler/gates.toml`",
                "a [[branch]] entry `name = \"feature/x\"` with `protected = true`",
                "a [[gate]] entry `branch = \"feature/x\"` with `min_approvals = 1`"
              ],
              "must_not_observe": [
                "the file remaining absent / `empty` after the write (the create-on-absent path no-oped)",
                "the write returning an error because the file did not exist (`none` of the entry created)",
                "`0` [[branch]] entries in the created file (the seed wrote `none`)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-8",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN gates_governance_base: caller no-read-principal holds [\"contents:write\"] only (NO administration:read and NO administration:write) at the target ref; admin holds administration:read via administration:write WHEN branch_gates_read(&repo, \"refs/heads/main\") runs once as BUT_AGENT_HANDLE=admin (authorized) and once as BUT_AGENT_HANDLE=no-read-principal (the v1 decision: branch_gates_read enforces administration:read before returning any committed gate set) THEN as admin: returns Ok with the committed main gate set (min_approvals=2); as no-read-principal: returns Err whose classify_error(&err) yields Some(_) with .code == \"perm.denied\" naming \"administration:read\" \u2014 the gate set is NOT disclosed to an unauthorized caller",
      "verify": "cargo test -p but-api branch_gates_read_requires_administration_read",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api branch_gates_read + real but-authz administration:read authorization + real gix target-ref read",
        "negative_control": {
          "would_fail_if": [
            "branch_gates_read were ungated \u2014 the no-read-principal caller would receive Ok with min_approvals=2 (governance recon leak), failing the perm.denied assertion",
            "the read denied for a non-authorization reason \u2014 the message must name administration:read, proving the read gate, not a generic error",
            "a stub returned Ok(empty) for the authorized admin too \u2014 admin's committed min_approvals=2 must_observe would fail",
            "the read returned a static/disconnected gate set ignoring authorization \u2014 the unauthorized caller would still see the committed values"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "gates_governance_base",
            "action": {
              "actor": "ci",
              "steps": [
                "temp_env BUT_AGENT_HANDLE=admin under #[serial_test::serial]; branch_gates_read(&repo, \"refs/heads/main\") -> capture Ok committed gate set",
                "temp_env BUT_AGENT_HANDLE=no-read-principal under #[serial_test::serial]; branch_gates_read(&repo, \"refs/heads/main\") -> classify the returned error via config_mutate::classify_error"
              ]
            },
            "end_state": {
              "must_observe": [
                "as admin, `branch_gates_read` returns `Ok` with the committed main gate `min_approvals = 2`",
                "as no-read-principal, `branch_gates_read` returns `Err`",
                "`classify_error(&err)` yields `Some(_)` whose `.code == \"perm.denied\"`",
                "the denial message contains `\"administration:read\"`"
              ],
              "must_not_observe": [
                "the no-read-principal caller receiving `Ok` with the committed gate set (governance recon leak)",
                "the admin caller receiving an `empty`/`none` committed gate set (`0` gates) on an authorized read",
                "the denial firing without naming `administration:read` (an `unchanged`/generic error not proving the read gate)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "After admin branch_gates_update(min_approvals=3), the working-tree .gitbutler/gates.toml main gate reads min_approvals=3",
      "verify": "cargo test -p but-api branch_gates_update_writes_worktree_inert_until_committed",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "After the update branch_gates_read reports the COMMITTED (target-ref) main gate min_approvals=2 AND ref_id(refs/heads/main) is identical before and after (inert / no commit)",
      "verify": "cargo test -p but-api branch_gates_update_writes_worktree_inert_until_committed",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "The branch_gates_update result/printed caveat contains \"takes effect once committed to the target branch\"",
      "verify": "cargo test -p but-api branch_gates_update_writes_worktree_inert_until_committed",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "Unprotecting main (protected=false) lands protected=false AND main's [[gate]] requirement (min_approvals=2, distinct=true, groups=[code-reviewers,maintainers]) survives",
      "verify": "cargo test -p but-api branch_gates_update_unprotect_preserves_gate_requirement",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "A protection-only edit re-serializes BOTH the full main [[gate]] requirement AND the unrelated release [[branch]]/[[gate]] entries (lossless round-trip; no [[gate]] drop)",
      "verify": "cargo test -p but-api branch_gates_update_round_trips_full_gate_schema_lossless",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "description": "Re-loading the rewritten file through the merge_gate-shaped GatesWire confirms min_approvals and require_approval_from_group survive (no silent weakening of the merge gate)",
      "verify": "cargo test -p but-api branch_gates_update_round_trips_full_gate_schema_lossless",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-7",
      "type": "test_criterion",
      "description": "A non-admin branch_gates_update returns Err; classify_error(&err) yields Some(AdminWriteGateError) whose .code == \"perm.denied\" and the message contains \"administration:write\"",
      "verify": "cargo test -p but-api branch_gates_update_non_admin_denied_writes_nothing",
      "maps_to_ac": "AC-4"
    },
    {
      "id": "TC-8",
      "type": "test_criterion",
      "description": "After the denied non-admin (self-escalation) branch_gates_update, the working-tree .gitbutler/gates.toml is byte-for-byte unchanged (gate ran before any write; control not flipped)",
      "verify": "cargo test -p but-api branch_gates_update_non_admin_denied_writes_nothing",
      "maps_to_ac": "AC-4"
    },
    {
      "id": "TC-9",
      "type": "test_criterion",
      "description": "branch_gates_read returns committed main min_approvals=2 with pending=true after an uncommitted min_approvals=3 edit; pending=false on a clean working tree",
      "verify": "cargo test -p but-api branch_gates_read_returns_committed_set_with_pending_signal",
      "maps_to_ac": "AC-5"
    },
    {
      "id": "TC-10",
      "type": "test_criterion",
      "description": "`pnpm build:sdk && pnpm format` regenerates packages/but-sdk/src/generated containing branch_gates_read and branch_gates_update commands/types, and the generated TS type-checks (no hand-edit)",
      "verify": "pnpm build:sdk && pnpm format && git diff --name-only packages/but-sdk/src/generated | grep -q . && grep -rq \"branch_gates_update\\|branchGatesUpdate\" packages/but-sdk/src/generated",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-11",
      "type": "test_criterion",
      "description": "branch_gates_update(edit{branch=release, require_distinct_from_author=true, require_approval_from_group=[\"maintainers\",\"code-reviewers\"]}) lands require_distinct_from_author=true AND require_approval_from_group=[\"maintainers\",\"code-reviewers\"] on the release [[gate]] (re-loadable via the merge_gate-shaped GatesWire)",
      "verify": "cargo test -p but-api branch_gates_update_sets_distinct_and_required_groups",
      "maps_to_ac": "AC-6"
    },
    {
      "id": "TC-12",
      "type": "test_criterion",
      "description": "A protection-only edit re-serializes a file whose full [[branch]]+[[gate]] structure re-parses (via the merge_gate-shaped GatesWire) with branch.len() and gate.len() preserved and every [[gate]] field value identical (SEC-1 structural round-trip: gate.len() and the full set survive)",
      "verify": "cargo test -p but-api branch_gates_update_round_trips_full_gate_schema_lossless",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-13",
      "type": "test_criterion",
      "description": "branch_gates_update against a populated gates.toml APPENDS a new feature/x [[branch]]+[[gate]] while preserving the existing main+release entries (no clobber)",
      "verify": "cargo test -p but-api branch_gates_update_appends_new_branch_and_creates_absent_file",
      "maps_to_ac": "AC-7"
    },
    {
      "id": "TC-14",
      "type": "test_criterion",
      "description": "branch_gates_update against an absent/empty .gitbutler/gates.toml CREATES the file with exactly the new feature/x [[branch]]+[[gate]] entry (the empty-state-then-add seeding path)",
      "verify": "cargo test -p but-api branch_gates_update_appends_new_branch_and_creates_absent_file",
      "maps_to_ac": "AC-7"
    },
    {
      "id": "TC-15",
      "type": "test_criterion",
      "description": "branch_gates_read as admin returns Ok committed min_approvals=2; as no-read-principal (no administration:read) returns Err with classify_error(&err)=Some(_) .code==perm.denied naming administration:read (the read scope is gated, not a recon leak)",
      "verify": "cargo test -p but-api branch_gates_read_requires_administration_read",
      "maps_to_ac": "AC-8"
    }
  ]
}
-->
