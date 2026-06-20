# CLI-002: `but group {create,grant,add-member,remove-member,list}` + admin-write gating

## What this does

Adds the second governance-management noun, `but group`, as a thin CLI shim over net-new `but-api` functions that EXTEND the CLI-001 governed-config writer. `create`/`grant`/`add-member`/`remove-member` mutate the `[[group]]` definitions, group grants, and membership in the **working-tree** `.gitbutler/permissions.toml` (read-modify-write, preserving every unrelated `[[principal]]`/`[[group]]` entry and role-preset sugar), each authorizing `administration:write` at the **target ref** via the Sprint-02 AUTHZ-006 guard (`enforce_administration_write_gate`) BEFORE the write, and each printing the ref-pin caveat *"takes effect once committed to the target branch."* The writes are **inert until committed** — working tree only, never staged/committed/touching a ref — so a feature head that adds its author to a `merge`-holding group cannot authorize its own merge (the property Sprint 03 GRPS-002 proved at the read layer is now PRODUCED by a real verb that warns the operator). Every mutating verb **fails closed**: a `group_grant` against an UNDEFINED group name, an unparseable `Authority` token, and an unset `BUT_AGENT_HANDLE` each return Err (no write), never a silent skip or anonymous action. `list` shows groups, their grants, and membership under `administration:read`. All write/read logic is sited at the `but-api` boundary (extending `crates/but-api/src/legacy/governance.rs` from CLI-001) so Sprint 06a's `group_create`/`group_grant`/`group_add_member`/`group_remove_member`/`group_list` Tauri commands re-invoke the SAME functions.

## Why

Sprint 05 · PRD UC-GRPS-01 (group as a first-class grantee; effective set = union; group ops gated by administration:write), UC-GRPS-02 (ref-pinned governed group membership; a `but group` membership change takes effect only once committed, and `but group` output says so). Serves the gate clauses: an admin's `but group create` writes a `[[group]]`, `but group add-member` prints the ref-pin caveat, and a non-admin's group op is denied `perm.denied`. These are the persisted-write consumers Sprint 03's GRPS tasks named with `BLOCKED-UNTIL` notes pointing here — the inert-until-committed *behavior* they proved against committed config is now produced by an operator-facing verb (CAP-CONFIG-01), admin-gated server-side (CAP-AUTHZ-01).

## How to verify

PRIMARY **AC-1** — `cargo test -p but-api group_add_member_writes_worktree_inert_until_committed` (Admin `group_add_member` writes the membership into the working-tree `.gitbutler/permissions.toml` while the target-ref group membership / the new member's effective set is UNCHANGED and the `refs/heads/main` ref_id is identical before and after — the inert-until-committed pair). Full gate set in the spec below.

## Scope

- `crates/but-api/src/legacy/governance.rs` (MODIFY — owned by CLI-001, EXTENDED here) — add `group_create`, `group_grant`, `group_add_member`, `group_remove_member`, `group_list`, each composing `enforce_administration_write_gate` (mutating verbs) / the administration:read scope (`group_list`), reusing the CLI-001 read-modify-write writer to mutate `[[group]]` blocks. **Sequenced after CLI-001** — see DEPENDENCIES.
- `crates/but/src/args/group.rs` (NEW) — the `Group` clap `Platform` + `Subcommands` (`Create`/`Grant`/`AddMember`/`RemoveMember`/`List`), mirroring `crates/but/src/args/config.rs` + the CLI-001 `args/perm.rs`. NO `--as`/identity-override flag is defined on any `Group` subcommand (S2).
- `crates/but/src/args/mod.rs` (MODIFY, SHARED with CLI-001) — TWO edit sites: (1) add the `Group(group::Platform)` variant to the `Subcommands` enum (the variant list at ~:1040), and (2) add `pub mod group;` to the `pub mod` declaration block (~:1272-1356). Add ONLY the Group variant/module decl; CLI-001 owns the Perm ones.
- `crates/but/src/command/help.rs` (MODIFY, SHARED with CLI-001) — add ONLY the exhaustive-help grouping arm for `SubcommandDiscriminant::Group` (mirror `Config`/`Perm` under `Group::OtherCommands`). This is compile-required by the generated `SubcommandDiscriminant` match; do not change unrelated help grouping/rendering behavior.
- `crates/but/src/command/group.rs` (NEW) — the thin shim: resolve gix repo + the WORKSPACE TARGET ref (not HEAD) from `ctx`, call the `but-api` `group_*` fn, print the result / ref-pin caveat / structured denial. Mirror `crates/but/src/command/perm.rs` (CLI-001).
- `crates/but/src/command/mod.rs` (MODIFY, SHARED with CLI-001) — add `pub mod group;` ONLY.
- `crates/but/src/lib.rs` (MODIFY, SHARED with CLI-001) — add the `Subcommands::Group(args::group::Platform { cmd }) => …` dispatch arm beside `Subcommands::Perm`/`Subcommands::Config` (:448). Add ONLY the Group arm.
- `crates/but/src/utils/metrics.rs` (MODIFY, SHARED with CLI-001) — add the `Subcommands::Group(..)` arm to the metrics match (mirror :175) so the build stays exhaustive. Add ONLY the Group arm.
- `crates/but-api/tests/group_governance.rs` (NEW) — the PRIMARY proofs: admin membership write lands inert in the working tree (ref_id unchanged); non-admin group op denied; create/grant land in the working tree preserving unrelated entries; group_grant fail-closed (undefined group / bad token / unset handle); list under admin:read (composes `but-api` `group_*` against real `but-authz` + real gix via `but_testsupport::writable_scenario`).
- `crates/but/tests/but/command/group.rs` (NEW) — the happy-path CLI verb wiring + ref-pin-caveat stdout contract + the `--as`-reject stdout contract (snapbox `env.but(...).assert()`).
- `crates/but/tests/but/command/mod.rs` (MODIFY, ONLY IF a `mod` index exists) — add `mod group;`.

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: CLI-002 - `but group {create,grant,add-member,remove-member,list}` + admin-write gating
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     L  (240 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-GRPS-01, UC-GRPS-02
CAPABILITIES: CAP-AUTHZ-01, CAP-CONFIG-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-api group_add_member_writes_worktree_inert_until_committed   |   cargo test -p but-api group_create_grant_writes_worktree   |   cargo test -p but-api group_ops_non_admin_denied   |   cargo test -p but-api group_grant_fail_closed_undefined_group_bad_token_and_unset_handle   |   cargo test -p but-api group_list_under_admin_read   |   cargo test -p but group   |   cargo test -p but-authz invariant_build_gates
  check: cargo check -p but-api --all-targets   |   cargo check -p but --all-targets
  lint:  cargo clippy -p but-authz -p but-api -p but --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
The `but group` noun exists and is proven against real git: (1) an admin `group_add_member` reads the target-ref config, authorizes administration:write, then writes the membership into the WORKING-TREE `.gitbutler/permissions.toml` while the target-ref `[[group]]` membership AND the new member's target-ref effective `AuthoritySet` are UNCHANGED and the `refs/heads/main` ref_id is identical before and after — the inert-until-committed pair — and prints the ref-pin caveat; (2) `group_create`/`group_grant` likewise land a new `[[group]]` block / a new group grant in the working tree, preserving every unrelated entry and role sugar by VALUE (read-modify-write, NOT a normalized GovConfig round-trip); (3) a non-admin `group_create`/`group_grant`/`group_add_member`/`group_remove_member` is denied perm.denied (names administration:write) and writes NOTHING (byte-unchanged); (4) `group_grant` against an UNDEFINED group name, with an unparseable Authority token, OR with `BUT_AGENT_HANDLE` unset fails closed (Err / non-zero exit) and writes NOTHING; (5) `group_list` under administration:read shows groups, grants, and membership. The CLI verb is a thin shim over the `but-api` `group_*` functions (sited so Sprint 06a Tauri commands reuse them), resolves the WORKSPACE TARGET ref (not HEAD), and clap-rejects an `--as` override before any governed action. `cargo test -p but-api` + `-p but` green; clippy clean; the AUTHZ-007/008 honesty grep — covering governance.rs (CLI-001 added the path) — stays green.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST compose the EXISTING admin-write guard, never author a second one. Every mutating verb calls `but_api::legacy::config_mutate::enforce_administration_write_gate(&repo, target_ref)` (config_mutate.rs:18) BEFORE the write, surfacing denial via `config_mutate::classify_error` (:31). Do NOT add a parallel authorize(AdministrationWrite) call — compose the guard (the same guard CLI-001 composes).
- [MUST] MUST reuse the CLI-001 read-modify-write writer in legacy/governance.rs, not author a second TOML writer. group_* mutate `[[group]]` blocks (name/permissions/members) via the same raw-wire read-modify-write that preserves unrelated entries + role sugar. If the CLI-001 writer is principal-only, EXTEND it to edit groups (add a group-targeting path) — do NOT fork a parallel serializer or round-trip a normalized GovConfig (which desugars role and folds membership both directions — config.rs:303-371). NOTE the value/format consequence inherited from CLI-001 (R4 decision a): CLI-001's writer re-serializes the raw wire structs (`PermissionsWire`/`PrincipalWire`/`GroupWire`, config.rs:400-429, which CLI-001 made `pub` + added `#[derive(Serialize)]` to) via `toml::to_string` — emitting CANONICAL TOML that preserves all VALUES (role="write" stays role="write", unrelated principals/groups stay present) but MAY drop comments/blank lines and normalize ordering. CLI-002 INHERITS that decision; the group-write contract is VALUE-preserving on a SUCCESSFUL write, byte-for-byte only on UNCHANGED files (denied/fail-closed writes AC-3/AC-4).
- [MUST] MUST write the WORKING TREE only. The writer `std::fs::write`s the file at `repo.workdir()` (mirror crates/but-api/tests/admin_write_guard.rs:158-164); it MUST NOT call git add/stage/commit or touch any ref. The inert-until-committed contract is STRUCTURAL — the next target-ref `load_governance_config` read makes the membership effective. A feature head that adds its author to a merge-holding group must NOT authorize its own merge (GRPS-002 proved the read side; this verb produces the inert write — AC-1's inert assertion AND the `ref_id(main)` before==after assertion catch a writer that commits).
- [MUST] MUST resolve the `.gitbutler/permissions.toml` path via `but_authz::permissions_path()` (the accessor CLI-001 added), never a re-derived literal.
- [MUST] MUST parse group-grant / member-permission tokens with `but_authz::Authority::parse` / `AuthoritySet::parse` (authority.rs:69,:193). Group names (`code-reviewers`, `maintainers`) and principal ids are config DATA, never enforcement branches. An unparseable token fails closed (Err, never a silent skip — AC-4).
- [MUST] MUST fail closed on an UNDEFINED target group, an unparseable token, AND an unresolvable caller. `group_grant <name> ...` against a group name with NO `[[group]]` block returns Err (a descriptive GovWriteError — NOT a silent skip and NOT an auto-create) and writes NOTHING; an unparseable `Authority` token surfaces a `ParseAuthorityError` (Err / non-zero, no write); when `BUT_AGENT_HANDLE` is UNSET, every group verb returns the `Denial::no_handle` → `perm.denied` (no anonymous action) and the mutating verbs write NOTHING (AC-4). Resolve the caller via `but_authz::resolve_principal_from_env` (inside the AUTHZ-006 guard).
- [MUST] MUST treat granting a group `administration:write` as an ALLOWED, named property (UC-GRPS-01 group-ceiling AC): `group_grant <name> administration:write` is permitted (it is itself administration:write-gated) — it is delegated admin, NOT a silent escalation, and NOT a special-cased rejection. Do not block it; the gate already requires the caller to hold administration:write at the target ref.
- [NEVER] NEVER branch on role names (read/triage/write/maintain/admin) or human-vs-AI predicates in any authorization/enforcement path. The CLI-001 honesty-grep coverage of governance.rs (it added `crates/but-api/src/legacy/governance.rs` to invariant_build_gates.rs ENFORCEMENT_PATHS + a positive Authority assertion) means CLI-002's group_* functions — which live INSIDE governance.rs — are ALREADY covered. The group_* code MUST stay in governance.rs with NO role-name / human-vs-AI branch (typed `Authority` only; names are DATA). A `match role { "admin" => }` branch there will fail the grep. Do NOT regress the grep, and do NOT move group_* out of governance.rs to escape the coverage.
- [NEVER] NEVER overload GitButler's pre-existing `Permission` (the repo-access lock) in any authorization path — use `Authority`/`AuthoritySet` exclusively.
- [NEVER] NEVER ship `but group delete` in this task. `delete` is a Sprint-06a/UC-MGMT-03 (B11) consumer surface; this sprint ships the FIVE gate verbs (create/grant/add-member/remove-member/list). Do not add a `Delete` clap variant, `group_delete` API function, `todo!()`, `unimplemented!()`, or placeholder — do not expand scope.
- [NEVER] NEVER define an `--as`/`--principal-override`/identity-override flag on any `Group` subcommand. The caller identity comes from `BUT_AGENT_HANDLE` only (mirror confinement.rs's `--as` rejection). `but group add-member maintainers --as admin --principal rust-reviewer` MUST clap-reject (unknown flag, non-zero exit) BEFORE any governed action (S2 / TC-13).
- [NEVER] NEVER bury the write/read logic in `crates/but/`. The `group_*` functions live at the but-api boundary (legacy/governance.rs) so Sprint 06a's group_* Tauri commands (api-design.md:87-92) reuse them. The `but` verb is a thin arg-parse + print shim.
- [STRICTLY] STRICTLY sequence AFTER CLI-001. CLI-001 establishes legacy/governance.rs (the writer + perm_*), the args/mod.rs `pub mod perm;` + dispatch/metrics pattern, `but_authz::permissions_path()`, AND the invariant_build_gates.rs ENFORCEMENT_PATHS edit folding governance.rs into the honesty grep (an ADDITIVE edit CLI-001 owns). CLI-002 EXTENDS those — if CLI-001 has not landed, the writer/accessor/dispatch pattern/grep-coverage does not exist. The orchestrator MUST land CLI-001 first; do not run CLI-002 in a parallel worktree without rebasing on CLI-001 (SHARED-EDIT files below). CLI-002 MUST NOT touch invariant_build_gates.rs — CLI-001 owns that additive edit; do not duplicate or weaken it.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: Admin group_add_member writes the membership into the working-tree config while the target-ref group membership / member effective set is unchanged and ref_id(main) is identical before==after (inert-until-committed pair), and prints the ref-pin caveat
- [ ] AC-2: group_create / group_grant land a new [[group]] block / grant in the working tree, preserving unrelated entries and role sugar by VALUE (read-modify-write, not a normalized round-trip)
- [ ] AC-3: A non-admin group_create/grant/add-member/remove-member is denied perm.denied (names administration:write) and writes nothing (byte-unchanged)
- [ ] AC-4: group_grant against an UNDEFINED group name OR with an unparseable Authority token OR with BUT_AGENT_HANDLE unset fails closed (Err / non-zero) and writes nothing
- [ ] AC-5: group_list under administration:read shows groups, grants, and membership; a caller without administration:read is denied
- [ ] All verification gates pass; only write_allowed files modified; the honesty grep (covering governance.rs) stays green; no `but group delete` shipped; invariant_build_gates.rs untouched

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: Admin group_add_member writes the membership into the working-tree config while the target-ref membership/effective set is unchanged and ref_id(main) is identical before==after (inert-until-committed pair) [PRIMARY]
  GIVEN: refs/heads/main with committed `.gitbutler/permissions.toml` where `admin` holds administration:write, a `[[group]]` `maintainers` (permissions=["merge"], members=["maint"]) does NOT list `rust-reviewer`, and `rust-reviewer` holds `["reviews:write"]` (no merge); clean working tree
  WHEN:  `group_add_member(&repo, target_ref, group="maintainers", principal="rust-reviewer")` runs with BUT_AGENT_HANDLE=admin (temp_env::with_var under #[serial_test::serial])
  THEN:  the call returns Ok; the WORKING-TREE `.gitbutler/permissions.toml` `[[group]] maintainers` members now include `rust-reviewer`; AND `load_governance_config(repo, "refs/heads/main")` STILL shows `maintainers` members EXCLUDING `rust-reviewer` and `rust-reviewer`'s effective set STILL does NOT contain `Authority::Merge` (inert — target ref unchanged); the `refs/heads/main` ref_id is identical before and after the write (no commit); the printed caveat contains "takes effect once committed to the target branch"
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api group_add_member + real but-authz load_governance_config + real gix repo (committed target-ref config vs working-tree write) via but_testsupport::writable_scenario
  VERIFY: cargo test -p but-api group_add_member_writes_worktree_inert_until_committed
  SCENARIO (negative controls): would fail if the writer wrote nothing (working-tree maintainers would lack rust-reviewer); would fail if the writer COMMITTED the membership (the target-ref effective set would then grant rust-reviewer Merge — the self-escalation GRPS-002 forbids — and ref_id(main) would change); would pass against a stub that always returns Ok without writing (guarded by asserting the working-tree group block literally lists rust-reviewer); would fail if the caveat string were absent

AC-2: group_create / group_grant land a new [[group]] block / grant in the working tree, preserving unrelated entries and role sugar by VALUE (read-modify-write, value-preserving, not a normalized round-trip)
  GIVEN: a committed `.gitbutler/permissions.toml` with `admin` (administration:write), a `rust-implementer` principal carrying `role = "write"` (sugar), and an unrelated `[[group]]` `maintainers` (permissions=["merge"], members=["maint"])
  WHEN:  `group_create(&repo, target_ref, "code-reviewers", ["reviews:write"])` then `group_grant(&repo, target_ref, "code-reviewers", ["comments:write"])` run as admin
  THEN:  the working-tree file now contains a `[[group]] code-reviewers` block with permissions including `reviews:write` and `comments:write`; the unrelated `[[group]] maintainers` block survives with its `merge` grant + `maint` member present (by VALUE, not byte-verbatim); the `rust-implementer` `role = "write"` VALUE survives (sugar preserved as a role assignment, NOT desugared to an expanded flat permissions list); only the code-reviewers block was added/edited in value
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api group_create + group_grant + real gix working-tree read-back; assert on the parsed wire shape + raw TOML text (VALUE-preserving), not a normalized GovConfig
  VERIFY: cargo test -p but-api group_create_grant_writes_worktree
  SCENARIO (negative controls): would fail if the writer re-serialized a normalized GovConfig — `role = "write"` would be desugared to a flat list and gone, and the membership fold would rewrite principals; would fail if the unrelated maintainers block were dropped (its merge grant / maint member absent); would fail if group_create wrote nothing (no code-reviewers block); creating a duplicate group name is an error, NOT a silent overwrite

AC-3: A non-admin group_create/grant/add-member/remove-member is denied perm.denied (names administration:write) and writes nothing
  GIVEN: the AC-1 committed config; caller `rust-reviewer` holds `["reviews:write"]` only (NO administration:write) at the target ref; the working-tree config captured byte-for-byte before the call
  WHEN:  `group_add_member(&repo, target_ref, "maintainers", "rust-reviewer")` (a self-add to a merge-holding group) runs with BUT_AGENT_HANDLE=rust-reviewer
  THEN:  the call returns Err; `config_mutate::classify_error(&err).code == "perm.denied"`; the message contains "administration:write"; AND the working-tree `.gitbutler/permissions.toml` is byte-for-byte UNCHANGED (gate ran BEFORE any write)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api group_add_member composing enforce_administration_write_gate + real but-authz + real gix
  VERIFY: cargo test -p but-api group_ops_non_admin_denied
  SCENARIO (negative controls): would pass (wrongly) against a stub that always writes / never gates — guarded by the byte-for-byte-unchanged assertion + classify_error.code=="perm.denied"; would fail-open if the writer ran before the gate (the file would change); relies on the self-add being inert because rust-reviewer lacks administration:write at the target ref (CAP-CONFIG-01 / GRPS-002 self-escalation), not on a role check

AC-4: group_grant against an UNDEFINED group name OR with an unparseable Authority token OR with BUT_AGENT_HANDLE unset fails closed (Err / non-zero) and writes nothing
  GIVEN: the AC-1 committed config (the only defined group is `maintainers`); the working-tree config captured byte-for-byte before each call
  WHEN:  (a) admin `group_grant(&repo, "refs/heads/main", "ghosts", ["reviews:write"])` runs with BUT_AGENT_HANDLE=admin where NO `[[group]] ghosts` block exists (an UNDEFINED target group); AND (b) admin `group_grant(&repo, "refs/heads/main", "maintainers", ["badtoken"])` runs with BUT_AGENT_HANDLE=admin (an unparseable Authority token); AND (c) `group_grant(&repo, "refs/heads/main", "maintainers", ["reviews:write"])` runs with BUT_AGENT_HANDLE UNSET (no resolvable caller)
  THEN:  (a) returns Err (a descriptive GovWriteError naming the undefined group — NOT a silent skip and NOT an auto-create of `ghosts`) and the working-tree file is byte-for-byte UNCHANGED; (b) returns Err (a ParseAuthorityError surfaced, classify_error code "config.invalid" or the parse-error code — NOT a silent skip) and the working-tree file is byte-for-byte UNCHANGED; (c) returns Err with code "perm.denied" (resolve_principal_from_env → Denial::no_handle, no anonymous action) and the working-tree file is byte-for-byte UNCHANGED
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api group_grant (undefined-group presence check + Authority::parse) + real resolve_principal_from_env + real gix working-tree read-back via but_testsupport::writable_scenario
  VERIFY: cargo test -p but-api group_grant_fail_closed_undefined_group_bad_token_and_unset_handle
  SCENARIO (negative controls): an impl that AUTO-CREATES `ghosts` on an undefined-group grant fails — the undefined-group call MUST Err and the file MUST be unchanged (no `[[group]] ghosts` block appears); an impl that silently skips an unparseable token (returns Ok writing nothing, or writes a partial set) fails — the bad-token call MUST Err and the file MUST be unchanged; an impl that performs an anonymous grant when BUT_AGENT_HANDLE is unset fails — the unset-handle call MUST Err perm.denied and write nothing; a fail-OPEN impl that writes before validating the group/token/caller fails the byte-for-byte-unchanged assertion

AC-5: group_list under administration:read shows groups, grants, and membership; a caller without administration:read is denied
  GIVEN: the AC-1 committed config (`maintainers` permissions=["merge"], members=["maint"]) plus a caller holding `administration:read` (e.g. a `maint-read` principal with role `maintain`, which desugars to include administration:read)
  WHEN:  `group_list(&repo, target_ref)` runs as that administration:read holder
  THEN:  the output enumerates the `maintainers` group, its `merge` grant, and its member `maint`; a caller WITHOUT administration:read (e.g. rust-reviewer holding only reviews:write) is denied `perm.denied`
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api group_list + real but-authz load_governance_config + real gix
  VERIFY: cargo test -p but-api group_list_under_admin_read
  SCENARIO (negative controls): would fail if group_list returned nothing / an empty set when groups exist; would leak if it returned groups to a caller WITHOUT administration:read (the denied case must Err perm.denied); would pass against a static fixture only if the listing is read from the real committed config — guarded by asserting the `maint` member and the `merge` grant appear

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, happy_path): after admin group_add_member, the working-tree [[group]] maintainers members include rust-reviewer
    VERIFY: cargo test -p but-api group_add_member_writes_worktree_inert_until_committed
- TC-2 (-> AC-1, structural): load_governance_config(repo, refs/heads/main) shows maintainers members EXCLUDE rust-reviewer and rust-reviewer's effective set excludes Authority::Merge after the add (target ref unchanged / inert)
    VERIFY: cargo test -p but-api group_add_member_writes_worktree_inert_until_committed
- TC-3 (-> AC-1, structural): ref_id(repo, "refs/heads/main") is identical before and after the group_add_member write (no commit performed); the result/printed caveat contains 'takes effect once committed to the target branch'
    VERIFY: cargo test -p but-api group_add_member_writes_worktree_inert_until_committed
- TC-4 (-> AC-2, happy_path): after group_create + group_grant, the working-tree file has a [[group]] code-reviewers block with reviews:write + comments:write
    VERIFY: cargo test -p but-api group_create_grant_writes_worktree
- TC-5 (-> AC-2, edge): the rewritten file still contains the unrelated [[group]] maintainers block (merge grant + maint member) and the rust-implementer role = "write" VALUE (unrelated entries + role sugar value-preserved)
    VERIFY: cargo test -p but-api group_create_grant_writes_worktree
- TC-6 (-> AC-3, error): non-admin group_add_member returns Err; config_mutate::classify_error(&err).code == perm.denied and message contains administration:write
    VERIFY: cargo test -p but-api group_ops_non_admin_denied
- TC-7 (-> AC-3, structural): the working-tree .gitbutler/permissions.toml is byte-for-byte unchanged after the denied non-admin group op
    VERIFY: cargo test -p but-api group_ops_non_admin_denied
- TC-8 (-> AC-4, error): group_grant against the UNDEFINED group "ghosts" returns Err (descriptive, no auto-create) and leaves the working-tree file byte-for-byte unchanged (no [[group]] ghosts block appears)
    VERIFY: cargo test -p but-api group_grant_fail_closed_undefined_group_bad_token_and_unset_handle
- TC-9 (-> AC-4, error): group_grant with token "badtoken" returns Err (parse error, not a silent skip) and leaves the working-tree file byte-for-byte unchanged
    VERIFY: cargo test -p but-api group_grant_fail_closed_undefined_group_bad_token_and_unset_handle
- TC-10 (-> AC-4, error): group_grant with BUT_AGENT_HANDLE unset returns Err code "perm.denied" (no anonymous action) and leaves the working-tree file byte-for-byte unchanged
    VERIFY: cargo test -p but-api group_grant_fail_closed_undefined_group_bad_token_and_unset_handle
- TC-11 (-> AC-5, happy_path): group_list under administration:read enumerates maintainers, its merge grant, and member maint; a caller without administration:read is denied perm.denied
    VERIFY: cargo test -p but-api group_list_under_admin_read
- TC-12 (-> AC-1/AC-3, integration CLI wiring): `but group add-member maintainers --principal rust-reviewer` as admin exits 0 and stdout contains the ref-pin caveat; the same as a non-admin exits 1 with stderr containing "perm.denied"
    VERIFY: cargo test -p but group
- TC-13 (-> AC-3, error CLI): `but group add-member maintainers --as admin --principal rust-reviewer` exits non-zero with a clap unknown/unsupported-flag error (no governed action performed, BUT_AGENT_HANDLE is the only identity source); no `--as` flag is defined on any Group subcommand
    VERIFY: cargo test -p but group

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01, CAP-CONFIG-01
provides: the `but group {create,grant,add-member,remove-member,list}` noun, sited at the but-api boundary for Sprint 06a Tauri reuse; admin-gated (composes AUTHZ-006) inert-until-committed working-tree group writes; fail-closed group_grant (undefined group / bad token / unset handle); group_list under administration:read; the named group-permission ceiling (a group MAY hold administration:write — delegated admin)
consumes: but_api::legacy::governance (the CLI-001 writer + perm_* module this extends), but_api::legacy::config_mutate::{enforce_administration_write_gate, classify_error}, but_authz::{load_governance_config, governance_present, resolve_principal_from_env, Authority, AuthoritySet, GovConfig, Group, GroupName, PrincipalId, permissions_path, Denial}, gix::Repository::workdir, but_testsupport::{writable_scenario, invoke_bash}
boundary_contracts:
  - CAP-AUTHZ-01: every group write verb authorizes administration:write (read at the target ref) via the AUTHZ-006 guard before mutating config; group_list enforces administration:read. Real-service proof: a non-admin group op is denied perm.denied and writes nothing; a group_grant against an undefined group / bad token / unset handle fails closed and writes nothing.
  - CAP-CONFIG-01: the group write path writes inert-until-committed config to the working tree only; effectiveness comes from the next target-ref read, so a feature head adding its author to a merge-holding group cannot authorize the same merge. Real-service proof: after a member add, the working-tree group block changed but the target-ref membership / member effective set did not, and ref_id(main) is unchanged.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-api/src/legacy/governance.rs (MODIFY — owned by CLI-001, EXTENDED here) — add group_create/group_grant/group_add_member/group_remove_member/group_list reusing the CLI-001 read-modify-write writer; extend the writer to edit [[group]] blocks if it is principal-only. group_* MUST live in this file (it is honesty-grep-covered by CLI-001) with NO role-name branch
  - crates/but/src/args/group.rs (NEW) — Group clap Platform + Subcommands (Create/Grant/AddMember/RemoveMember/List); NO --as/identity-override flag
  - crates/but/src/args/mod.rs (MODIFY, SHARED with CLI-001) — `pub mod group;` (in the pub-mod block ~:1272-1356) + `Group(group::Platform)` variant (in the Subcommands enum ~:1040) ONLY; do not touch the Perm variant (CLI-001 owns it)
  - crates/but/src/command/help.rs (MODIFY, SHARED with CLI-001) — add ONLY `SubcommandDiscriminant::Group => Group::OtherCommands` (or the equivalent single Group grouping arm if nearby help groups are renamed) so the exhaustive generated-discriminant match compiles; do not change ordering, text, hidden-command behavior, truncation, or unrelated grouping
  - crates/but/src/command/group.rs (NEW) — the thin CLI shim (resolves the WORKSPACE TARGET ref, not HEAD)
  - crates/but/src/command/mod.rs (MODIFY, SHARED with CLI-001) — `pub mod group;` ONLY
  - crates/but/src/lib.rs (MODIFY, SHARED with CLI-001) — the Subcommands::Group dispatch arm ONLY (beside :448)
  - crates/but/src/utils/metrics.rs (MODIFY, SHARED with CLI-001) — the Subcommands::Group metrics arm ONLY (mirror :175)
  - crates/but-api/tests/group_governance.rs (NEW) — the PRIMARY but-api proofs (AC-1..AC-5)
  - crates/but/tests/but/command/group.rs (NEW) — happy-path CLI verb wiring + ref-pin-caveat + --as-reject stdout (TC-12, TC-13)
  - crates/but/tests/but/command/mod.rs (MODIFY, ONLY IF a mod index exists) — `mod group;`
writeProhibited:
  - the perm_* functions + the WRITER CORE in legacy/governance.rs — owned by CLI-001; EXTEND with group_* and (if needed) a group-targeting writer path, but do NOT rewrite the perm_* functions or the principal write core
  - crates/but-api/src/legacy/config_mutate.rs — CONSUME-only (the AUTHZ-006 admin guard); compose enforce_administration_write_gate, do not fork a second admin check
  - crates/but-api/src/legacy/merge_gate.rs, crates/but-api/src/commit/gate.rs — Sprint-01a/01b gates; CONSUME-only
  - crates/but-authz/src/** — closed; permissions_path was added by CLI-001 — do not re-open the authz layer
  - crates/but-authz/tests/invariant_build_gates.rs — CLI-001 OWNS the additive governance.rs ENFORCEMENT_PATHS edit; CLI-002 MUST NOT touch this file (do not duplicate or weaken the honesty grep). group_* are covered because they live in the already-added governance.rs
  - crates/but/src/args/perm.rs, crates/but/src/command/perm.rs, the Perm variant/dispatch/metrics arm — owned by CLI-001
  - any gitbutler-* crate
  - `but group delete` implementation (Sprint-06a / UC-MGMT-03) — do not ship
  - Any file not in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-api/src/legacy/governance.rs (CLI-001 output — the module this EXTENDS)
   Focus: the CLI-001 perm_* functions, the read-modify-write writer (raw wire structs + toml::to_string), the REF_PIN_CAVEAT constant, the GovWriteError type, and the admin-gate composition pattern — group_* reuse all of these; extend the writer to edit [[group]] blocks. group_* MUST stay in this file (it is honesty-grep-covered)
2. crates/but-api/src/legacy/config_mutate.rs (1-44)
   Focus: enforce_administration_write_gate + classify_error — the guard EVERY group write verb composes (same as CLI-001); the unset-handle path returns Denial::no_handle -> perm.denied (AC-4 (c))
3. crates/but-api/tests/admin_write_guard.rs (8-68, 96-176)
   Focus: the temp_env BUT_AGENT_HANDLE + #[serial_test::serial] + writable_scenario + invoke_bash committed-config-at-main pattern, write_worktree_permissions (the working-tree write helper), committed_blob_text (byte-for-byte-unchanged for AC-3/AC-4). Mirror for group_governance.rs.
4. crates/but-api/tests/commit_gate.rs (1-98)
   Focus: the `ref_id(&repo, "refs/heads/main")` helper (:11, :48) for the before/after structural assertion (AC-1 inert + TC-3) — reuse ref_id for the target-ref-unchanged (no-commit) assertion
5. crates/but-authz/src/config.rs (303-371, 400-429)
   Focus: normalize_permissions (the group-membership-both-directions fold + role desugar) — PROVES you must read-modify-write the raw GroupWire shape, not round-trip GovConfig; the GroupWire struct (name/permissions/role/members) the writer edits (CLI-001 made it pub + added #[derive(Serialize)]); the undefined-group presence check (AC-4 (a)) reads whether a `[[group]]` block with that name exists before granting
6. crates/but-authz/src/principal.rs (145-215)
   Focus: Group::new(name, authorities, members) + .name()/.authorities()/.members() — how group_list renders a Group read from the committed config; GroupName/PrincipalId newtypes
7. crates/but-authz/src/authorize.rs (24-58, 67-102)
   Focus: authorize / effective_authority / resolve_principal_from_env — the inert assertion checks rust-reviewer's target-ref effective set excludes Merge (AC-1 TC-2); the unset-handle case is Denial::no_handle (AC-4 (c)); group_list's administration:read scope resolves the caller from env and checks AdministrationRead
8. crates/but-authz/src/authority.rs (69-110, 216-225, 343-365)
   Focus: Authority::parse / AuthoritySet::parse (group-grant tokens; the bad-token AC-4 (b) path: parse Err); the maintain role desugars to include AdministrationRead (the AC-5 admin:read holder); admin desugars to ALL incl administration:write (the group-ceiling delegated-admin property)
9. crates/but/src/args/config.rs (1-12, 424-467) + the CLI-001 crates/but/src/args/perm.rs
   Focus: the clap Platform + Subcommand shape to mirror in args/group.rs (Create <name> --permissions, Grant <name> <tokens>, AddMember <name> --principal, RemoveMember <name> --principal, List); note NO --as flag exists — group mirrors that (S2)
10. crates/but/src/args/mod.rs (~1040 Subcommands enum variant list, ~1272-1356 pub mod declaration block)
   Focus: the TWO additive edit sites for a new noun — the enum variant (Group(group::Platform)) at the variant list AND the `pub mod group;` at the module-declaration block; mirror the Config/Perm noun's two insertions (R7)
11. crates/but/src/lib.rs (448-487) + the CLI-001 Subcommands::Perm arm
   Focus: how a noun resolves ctx + the WORKSPACE TARGET ref (not HEAD) + calls the command module + maps errors to CliError; mirror for the Subcommands::Group arm (R6)
12. crates/but/tests/but/command/confinement.rs (1-130)
   Focus: the snapbox CLI harness committing governance config at main + env.but(...).env("BUT_AGENT_HANDLE", ...).assert(), asserting stderr contains "perm.denied", AND the `--as` rejection pattern (clap unknown-flag, non-zero exit) to mirror for TC-13; mirror for crates/but/tests/but/command/group.rs (TC-12, TC-13)
13. .spec/prds/governance/10-technical-requirements/04-api-design.md (82-114)
   Focus: the Tauri group_create/group_grant/group_add_member/group_remove_member/group_list table (the functions you site for Sprint 06a reuse — note group_delete is listed but is OUT OF SCOPE here), the administration:write / administration:read split, the ref-pin caveat wording
14. .spec/prds/governance/tasks/sprint-03-grps-groups-ref-pin/GRPS-002-ref-pinned-membership-self-grant-inert.md
   Focus: the read-side property this verb produces the write for — a feature head adding its author to a merge-holding group is denied merge; this task makes the INERT write that GRPS-002 proved is safe, with the operator-visible caveat

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- group add-member inert (PRIMARY): `cargo test -p but-api group_add_member_writes_worktree_inert_until_committed`  -> Exit 0; working-tree maintainers gains rust-reviewer, target-ref membership/effective set unchanged (no Merge), ref_id(main) before==after, caveat printed
- group create+grant write: `cargo test -p but-api group_create_grant_writes_worktree`  -> Exit 0; code-reviewers block lands with reviews:write+comments:write, unrelated entries + role sugar survive by value
- non-admin group op denied, no write: `cargo test -p but-api group_ops_non_admin_denied`  -> Exit 0; perm.denied naming administration:write; working-tree file byte-for-byte unchanged
- fail-closed undefined-group + bad-token + unset-handle: `cargo test -p but-api group_grant_fail_closed_undefined_group_bad_token_and_unset_handle`  -> Exit 0; undefined-group grant Errs (no auto-create), bad token Errs (no silent skip), unset handle Errs perm.denied; file byte-for-byte unchanged in all three
- group list under admin:read: `cargo test -p but-api group_list_under_admin_read`  -> Exit 0; lists maintainers + merge grant + member maint; caller without admin:read denied
- CLI verb wiring: `cargo test -p but group`  -> Exit 0; admin add-member exits 0 with caveat on stdout; non-admin exits 1 with perm.denied on stderr; `--as` clap-rejected
- honesty grep (guard-rail stays green, covering governance.rs from CLI-001): `cargo test -p but-authz invariant_build_gates`  -> Exit 0; no role-label/human-vs-AI branching in governance.rs incl. the new group_*
- clippy: `cargo clippy -p but-authz -p but-api -p but --all-targets`  -> Exit 0
- fmt: `cargo fmt --check`  -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: thin-CLI-shim over the CLI-001 but-api governance boundary; group_* compose the AUTHZ-006 admin guard, then reuse the CLI-001 read-modify-write writer extended to edit [[group]] blocks (name/permissions/members), VALUE-preserving via the raw wire structs + toml::to_string, inert until committed; group_grant presence-checks the target group (Err on undefined, no auto-create) + Authority::parse-validates tokens (Err on bad) + fails closed on an unset handle; group_list reads the committed target-ref config under administration:read
pattern_source: write guard = crates/but-api/src/legacy/config_mutate.rs:18-28 (compose); writer + perm_* = crates/but-api/src/legacy/governance.rs (CLI-001, reuse/extend); working-tree write = crates/but-api/tests/admin_write_guard.rs:158-164; ref_id before/after = crates/but-api/tests/commit_gate.rs:11,48; the raw GroupWire shape to edit = crates/but-authz/src/config.rs:420-429 (NOT the normalized GovConfig at 303-371); Group rendering = crates/but-authz/src/principal.rs:145-215; CLI shim = crates/but/src/lib.rs:448-487 + the CLI-001 command/perm.rs; the TWO args/mod.rs edit sites = ~:1040 (enum variant) + ~:1272-1356 (pub mod); exhaustive help grouping = crates/but/src/command/help.rs:80-176 (add the single Group arm); CLI test = crates/but/tests/but/command/confinement.rs:69-130 (+ its --as rejection)
anti_pattern: forking a second TOML writer or admin check instead of reusing the CLI-001 writer + composing enforce_administration_write_gate; re-serializing a normalized GovConfig (loses role sugar + folds membership — AC-2 fails); committing/staging the membership write (breaks inert-until-committed / re-enables self-escalation — AC-1 inert + ref_id assertion fails); running the write before the admin gate (AC-3 byte-for-byte-unchanged fails); AUTO-CREATING an undefined target group on group_grant instead of Erroring (AC-4 (a) fails); silently skipping an unparseable token (AC-4 (b) fails); performing an anonymous grant when BUT_AGENT_HANDLE is unset (AC-4 (c) fails); shipping `but group delete` (out of scope — Sprint 06a); special-casing/rejecting group_grant administration:write (it is allowed delegated admin); branching on role names anywhere in governance.rs (honesty grep — CLI-001 covers governance.rs); touching/duplicating/weakening invariant_build_gates.rs (CLI-001 owns that edit); defining an --as/identity-override flag on a Group subcommand (S2); resolving HEAD instead of the workspace target ref in the shim (R6); burying group_* in crates/but/ so Sprint 06a cannot reuse them
interaction_notes:
  - Target-ref resolution: the writer/reader take an explicit `target_ref: &str` (e.g. "refs/heads/main") — the CLI shim resolves it from the WORKSPACE TARGET (not HEAD; mirror how the commit gate / CLI-001 perm shim resolves the config ref). For the but-api tests, pass "refs/heads/main" directly (the fixture commits governance at main), matching admin_write_guard.rs / commit_gate.rs. Keep the function signature `(&gix::Repository, target_ref: &str, ...)` so Sprint 06a Tauri commands pass the same. (R6)
  - Env handling: group_* resolve the caller via resolve_principal_from_env (BUT_AGENT_HANDLE) inside enforce_administration_write_gate / the scope predicate, so the but-api tests use temp_env::with_var("BUT_AGENT_HANDLE", ...) under #[serial_test::serial]; the AC-4 unset-handle case uses temp_env::with_var("BUT_AGENT_HANDLE", None::<&str>, ...). temp-env IS declared in but-api dev-deps (Cargo.toml:158) and serial_test at :157 — NO dependency addition needed (FLAG only if a build surfaces otherwise).
  - Verb semantics: group_create(name, permissions) appends a new `[[group]]` block (error on duplicate name); group_grant(name, tokens) unions tokens into the named group's permissions (Err if the group is UNDEFINED — a presence check, NOT an auto-create — AC-4 (a)); group_add_member(name, principal) appends the principal id to the group's members (idempotent — adding an existing member is a no-op success); group_remove_member(name, principal) removes the id (removing a non-member is a no-op success); group_list reads the committed target-ref config and enumerates groups + grants + members. All four mutating verbs preserve every unrelated entry and role sugar by VALUE via the read-modify-write, and fail closed on a bad token / unset handle.
  - R4 decision (INHERITED from CLI-001 — option a): CLI-001's wire structs derive `#[derive(Serialize)]` (additive, non-semantic) and the writer re-serializes via `toml::to_string`. CLI-002 inherits this for the group-write path. The successful-write contract is VALUE-preserving (role="write" value survives, unrelated principals/groups survive with their values), NOT byte-verbatim — `toml::to_string` emits canonical TOML and MAY drop comments/blank lines and normalize ordering. No must_observe asserts byte-verbatim on a SUCCESSFUL write; the only byte-for-byte assertions are on UNCHANGED files (denied write AC-3, fail-closed writes AC-4) where nothing is written at all.
  - S1 / honesty-grep coverage: CLI-001 OWNS the additive edit to crates/but-authz/tests/invariant_build_gates.rs that folds `crates/but-api/src/legacy/governance.rs` into ENFORCEMENT_PATHS + a positive Authority assertion. CLI-002's group_* functions live INSIDE governance.rs, so they are ALREADY covered by that grep — CLI-002 MUST NOT touch invariant_build_gates.rs (no duplicate, no weakening). The discipline that follows: group_* stay in governance.rs and carry NO role-name / human-vs-AI branch (typed Authority only; names are DATA). A stub/role-branch in the new group_* code fails the grep CLI-001 wired.
  - Group-permission ceiling (UC-GRPS-01 AC): granting a group administration:write is ALLOWED (delegated admin) and is itself administration:write-gated. Do not reject it; the audit-surface property is that admin-holding-group members can change config — a named, accepted property, not a silent escalation. (No test asserts it must be BLOCKED; if you add a positive test that group_grant maintainers administration:write succeeds as admin, that is welcome but optional.)
  - Sprint 06a reuse: site group_create/group_grant/group_add_member/group_remove_member/group_list so the future Tauri commands (api-design.md:87-92) wrap them directly. group_delete is listed in that table but is OUT OF SCOPE here (Sprint 06a/UC-MGMT-03 B11) — do not implement it, do not add a clap Delete variant, and do not add a placeholder.
  - Help grouping scope: adding `Group(group::Platform)` generates `SubcommandDiscriminant::Group`; `crates/but/src/command/help.rs` matches that discriminant exhaustively, so the Group grouping arm is required for `cargo check -p but --all-targets`. This is NOT optional help-surface expansion: add only the single Group arm (mirror Config/Perm under `Group::OtherCommands`) and do not touch unrelated help behavior.
  - SHARED-EDIT discipline: in args/mod.rs, lib.rs, utils/metrics.rs, command/mod.rs, command/help.rs, add ONLY the Group variant/arm/decl — never touch the CLI-001 Perm ones. These are additive inserts in the same regions CLI-001 edited; rebase on CLI-001 before editing.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — Rust work extending the CLI-001 governance writer to edit [[group]] blocks, five new clap subcommands + a thin CLI shim, all composing the AUTHZ-006 guard and producing inert-until-committed working-tree writes, with fail-closed group_grant (undefined group / bad token / unset handle). gix working-tree writes, target-ref blob reads, ref_id before/after, Group rendering, structured Denial classification, and snapbox CLI tests are rust-implementer competencies; rust-reviewer adversarially validates the writes are inert (never committed; ref_id unchanged), the admin gate precedes every write, role sugar + unrelated entries survive by value, group_grant fails closed on an undefined group/bad token/unset handle, `but group delete` is NOT shipped, the `--as` flag is clap-rejected, invariant_build_gates.rs is untouched, and group_grant administration:write is correctly allowed as delegated admin.
reviewer: rust-reviewer
coding_standards: crates/AGENTS.md, crates/but/AGENTS.md (CLI test economy — happy-path snapbox only), crates/WORKSPACE_MODEL.md, crates/but-authz + crates/but-api/src/legacy (nearby patterns: the CLI-001 governance writer, gix workdir write, target-ref blob read, ref_id before/after, Group/GroupName/Denial, but-testsupport writable_scenario + invoke_bash, temp_env + serial_test for BUT_AGENT_HANDLE)

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: CLI-001 (the legacy/governance.rs writer + perm_* + GovWriteError + permissions_path + the args/mod.rs `pub mod perm;`/dispatch/metrics pattern + the invariant_build_gates.rs governance.rs honesty-grep coverage this RELIES ON — MUST land first), AUTHZ-006 (the administration:write guard), AUTHZ-001/002/003 (the but-authz primitive), GRPS-001 (the union loader group_list renders), GRPS-002 (the read-side ref-pin property this verb produces the inert write for)
Blocks:     Sprint 06a (the group_* but-api functions the MGMT Tauri commands reuse)
SHARED-EDIT COORDINATION (CLI-001 + CLI-002 both touch these):
  - crates/but/src/args/mod.rs (Subcommands enum variant ~:1040 + the pub mod block ~:1272-1356), crates/but/src/lib.rs (dispatch arm), crates/but/src/utils/metrics.rs (metrics arm), crates/but/src/command/mod.rs (module decl), crates/but/src/command/help.rs (exhaustive SubcommandDiscriminant grouping arm), crates/but-api/src/legacy/governance.rs (CLI-001 owns the writer + perm_* + the file; CLI-002 appends group_* into the same, now honesty-grep-covered, file).
  - invariant_build_gates.rs is owned by CLI-001 (the additive governance.rs ENFORCEMENT_PATHS edit) — CLI-002 does NOT touch it; group_* are covered because they live in the already-added governance.rs.
  - Sequence CLI-002 AFTER CLI-001 lands. CLI-002 adds ONLY the Group variant/module/dispatch/metrics arm + the group_* functions; it EXTENDS legacy/governance.rs (does not rewrite perm_*). Flag to the orchestrator: do NOT run CLI-001 and CLI-002 in parallel worktrees — rebase CLI-002 on the landed CLI-001 first (the merge-conflict surface is the five shared files; additive inserts in the same regions can conflict textually, and governance.rs does not exist until CLI-001 lands).
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "CLI-002",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "group_governance_base": {
      "description": "Real-git scenario via but_testsupport::writable_scenario(\"checkout-head-info\"). Target ref refs/heads/main carries a committed .gitbutler/permissions.toml where `admin` holds administration:write, a [[group]] `maintainers` (permissions=[\"merge\"], members=[\"maint\"]) does NOT list rust-reviewer, `rust-reviewer` holds [\"reviews:write\"] (no merge, no administration:write), `rust-implementer` carries the literal `role = \"write\"` sugar (for the AC-2 preservation check), `maint` holds [\"merge\"], and a `maint-read` principal carries role=\"maintain\" (which desugars to include administration:read, for the AC-5 admin:read holder); plus .gitbutler/gates.toml marking main protected. The ONLY defined group is `maintainers` (so `ghosts` is undefined for AC-4). The working tree starts clean (matching the committed blob). Seeded via a REAL entrypoint: invoke_bash writes the files and git-commits them at main (same pattern as crates/but-api/tests/admin_write_guard.rs:96-123).",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"checkout-head-info\");",
        "invoke_bash on main: mkdir -p .gitbutler; write .gitbutler/permissions.toml with [[principal]] id=\"admin\" permissions=[\"administration:write\",\"merge\"]; [[principal]] id=\"rust-reviewer\" permissions=[\"reviews:write\"]; [[principal]] id=\"rust-implementer\" role=\"write\"; [[principal]] id=\"maint\" permissions=[\"merge\"]; [[principal]] id=\"maint-read\" role=\"maintain\"; [[group]] name=\"maintainers\" permissions=[\"merge\"] members=[\"maint\"]; and .gitbutler/gates.toml [[branch]] name=\"main\" protected=true; then git add .gitbutler/permissions.toml .gitbutler/gates.toml && git commit -m \"governance config\".",
        "Capture the committed working-tree state: committed_blob_text(repo, but_authz::permissions_path()) and ref_id(repo, \"refs/heads/main\") BEFORE any write, for the AC-3/AC-4 byte-for-byte-unchanged assertion and the AC-1 inert (target-ref read + ref_id-unchanged) assertion.",
        "AC-1 add-member step: set BUT_AGENT_HANDLE=admin via temp_env::with_var under #[serial_test::serial], call group_add_member(&repo, \"refs/heads/main\", \"maintainers\", \"rust-reviewer\"); writes ONLY the working-tree file (no commit); assert ref_id(main) before==after.",
        "AC-2 create+grant step: as admin, call group_create(&repo, \"refs/heads/main\", \"code-reviewers\", [\"reviews:write\"]) then group_grant(&repo, \"refs/heads/main\", \"code-reviewers\", [\"comments:write\"]); read back the raw working-tree TOML + parse into the wire structs.",
        "AC-3 denial step: set BUT_AGENT_HANDLE=rust-reviewer, call group_add_member(&repo, \"refs/heads/main\", \"maintainers\", \"rust-reviewer\") (self-add to a merge-holding group); expect Err perm.denied and the working-tree file unchanged from the captured committed state.",
        "AC-4 fail-closed step: set BUT_AGENT_HANDLE=admin, call group_grant(&repo, \"refs/heads/main\", \"ghosts\", [\"reviews:write\"]) (undefined group -> Err, no auto-create, file unchanged) AND group_grant(&repo, \"refs/heads/main\", \"maintainers\", [\"badtoken\"]) (parse Err, file unchanged); then set BUT_AGENT_HANDLE=None via temp_env::with_var(\"BUT_AGENT_HANDLE\", None::<&str>, ...), call group_grant(&repo, \"refs/heads/main\", \"maintainers\", [\"reviews:write\"]) (Err perm.denied no_handle, file unchanged).",
        "AC-5 list step: set BUT_AGENT_HANDLE=maint-read (administration:read via role maintain), call group_list(&repo, \"refs/heads/main\") (expect Ok enumerating maintainers + merge + member maint); separately set BUT_AGENT_HANDLE=rust-reviewer and call group_list (expect Err perm.denied)."
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "description": "Admin group_add_member writes the membership into the working-tree config while the target-ref group membership / member effective set is unchanged and ref_id(main) is identical before==after (inert-until-committed pair), and prints the ref-pin caveat.",
      "verify": "cargo test -p but-api group_add_member_writes_worktree_inert_until_committed",
      "primary": true,
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api group_add_member + real but-authz load_governance_config + real gix repo (committed target-ref config vs working-tree write) via but_testsupport::writable_scenario",
        "negative_control": {
          "would_fail_if": [
            "the writer wrote nothing — the working-tree maintainers block would lack rust-reviewer",
            "the writer COMMITTED the membership — the target-ref effective set would then grant rust-reviewer Merge (the self-escalation GRPS-002 forbids), breaking the inert (must_not_observe) assertion, and ref_id(main) would change",
            "a stub returned Ok without writing — guarded by asserting the working-tree group block literally lists rust-reviewer",
            "the ref-pin caveat string were absent (silent success)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "group_governance_base",
            "action": {
              "actor": "ci",
              "steps": [
                "capture ref_id(&repo, \"refs/heads/main\") BEFORE the write",
                "temp_env::with_var(\"BUT_AGENT_HANDLE\", Some(\"admin\"), ...) under #[serial_test::serial]",
                "group_add_member(&repo, \"refs/heads/main\", \"maintainers\", \"rust-reviewer\")",
                "read the working-tree .gitbutler/permissions.toml back",
                "load_governance_config(&repo, \"refs/heads/main\") and inspect maintainers members + rust-reviewer effective set",
                "capture ref_id(&repo, \"refs/heads/main\") AFTER the write"
              ]
            },
            "end_state": {
              "must_observe": [
                "`group_add_member` returns `Ok`",
                "the working-tree `[[group]] maintainers` members include `rust-reviewer`",
                "the result/printed caveat contains `\"takes effect once committed to the target branch\"`",
                "the target-ref `maintainers` group still exists with member `maint`",
                "`ref_id(refs/heads/main)` AFTER the write == the `ref_id` captured BEFORE the write"
              ],
              "must_not_observe": [
                "the target-ref `maintainers` membership including `rust-reviewer` (must be inert until committed)",
                "`rust-reviewer`'s target-ref effective set containing `Authority::Merge`",
                "the target ref `refs/heads/main` HEAD sha changing (no commit performed)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "group_create / group_grant land a new [[group]] block / grant in the working tree, preserving unrelated entries and role sugar by VALUE (read-modify-write, value-preserving, not a normalized round-trip).",
      "verify": "cargo test -p but-api group_create_grant_writes_worktree",
      "primary": false,
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api group_create + group_grant + real gix working-tree read-back; assertions on the parsed wire shape + raw TOML text (VALUE-preserving), not a normalized GovConfig",
        "negative_control": {
          "would_fail_if": [
            "the writer re-serialized a normalized GovConfig — `role = \"write\"` would be desugared to a flat list and gone and the membership fold would rewrite principals",
            "the unrelated `[[group]] maintainers` block were dropped or destructively reordered (its merge grant / maint member value absent)",
            "group_create wrote nothing (no code-reviewers block appeared)",
            "group_create silently overwrote a duplicate name instead of erroring",
            "a no-op stub writer that leaves the file unchanged (writes nothing) is caught — the new entry would be absent"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "group_governance_base",
            "action": {
              "actor": "ci",
              "steps": [
                "temp_env BUT_AGENT_HANDLE=admin under #[serial_test::serial]",
                "group_create(&repo, \"refs/heads/main\", \"code-reviewers\", [\"reviews:write\"])",
                "group_grant(&repo, \"refs/heads/main\", \"code-reviewers\", [\"comments:write\"])",
                "read the rewritten working-tree .gitbutler/permissions.toml as raw text + parse it into the wire structs"
              ]
            },
            "end_state": {
              "must_observe": [
                "the rewritten file contains a `[[group]]` `code-reviewers` block",
                "the code-reviewers permissions include `reviews:write` and `comments:write`",
                "the rewritten file still contains the unrelated `[[group]]` `maintainers` block with its `merge` grant and `maint` member",
                "the rewritten file still carries the `rust-implementer` `role = \"write\"` VALUE (a role assignment, not an expanded flat list)"
              ],
              "must_not_observe": [
                "the `role = \"write\"` line replaced by an expanded flat permissions list (sugar lost)",
                "the `[[group]] maintainers` block missing from the rewritten file",
                "the mutated entry appearing as 0 entries / an empty/no-op write (a do-nothing stub leaves the start state, which this excludes)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "A non-admin group_create/grant/add-member/remove-member is denied perm.denied (names administration:write) and writes nothing.",
      "verify": "cargo test -p but-api group_ops_non_admin_denied",
      "primary": false,
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api group_add_member composing enforce_administration_write_gate + real but-authz + real gix",
        "negative_control": {
          "would_fail_if": [
            "a stub always wrote / never gated — the working-tree file would change (guarded by the byte-for-byte-unchanged assertion)",
            "the writer ran BEFORE the admin gate — the file would change despite the denial (ordering bug)",
            "the denial were not classified perm.denied or did not name administration:write"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "group_governance_base",
            "action": {
              "actor": "ci",
              "steps": [
                "capture the working-tree .gitbutler/permissions.toml bytes before the call",
                "temp_env BUT_AGENT_HANDLE=rust-reviewer under #[serial_test::serial]",
                "group_add_member(&repo, \"refs/heads/main\", \"maintainers\", \"rust-reviewer\") (self-add to a merge-holding group)",
                "config_mutate::classify_error on the returned error",
                "re-read the working-tree file bytes"
              ]
            },
            "end_state": {
              "must_observe": [
                "`group_add_member` returns `Err`",
                "`config_mutate::classify_error(&err).code == \"perm.denied\"`",
                "the denial message contains `\"administration:write\"`",
                "the working-tree `.gitbutler/permissions.toml` bytes are identical to the pre-call capture"
              ],
              "must_not_observe": [
                "the working-tree `maintainers` block listing `rust-reviewer` after the denial",
                "any change to the working-tree file bytes",
                "the mutated entry appearing as 0 entries / an empty/no-op write (a do-nothing stub leaves the start state, which this excludes)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "description": "group_grant against an UNDEFINED group name OR with an unparseable Authority token OR with BUT_AGENT_HANDLE unset fails closed (Err / non-zero) and writes nothing.",
      "verify": "cargo test -p but-api group_grant_fail_closed_undefined_group_bad_token_and_unset_handle",
      "primary": false,
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api group_grant (undefined-group presence check + Authority::parse) + real resolve_principal_from_env + real gix working-tree read-back via but_testsupport::writable_scenario",
        "negative_control": {
          "would_fail_if": [
            "an impl AUTO-CREATES the undefined `ghosts` group on grant — the call MUST Err and the file MUST be unchanged (no `[[group]] ghosts` block appears)",
            "an impl silently skips an unparseable token (returns Ok writing nothing, or writes a partial set) — the bad-token call MUST Err and the file MUST be unchanged",
            "an impl performs an anonymous grant when BUT_AGENT_HANDLE is unset — the unset-handle call MUST Err perm.denied and write nothing",
            "a fail-OPEN impl that writes before validating the group/token/caller — the byte-for-byte-unchanged assertion catches it"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "group_governance_base",
            "action": {
              "actor": "ci",
              "steps": [
                "capture the working-tree .gitbutler/permissions.toml bytes before each call",
                "temp_env BUT_AGENT_HANDLE=admin under #[serial_test::serial]; group_grant(&repo, \"refs/heads/main\", \"ghosts\", [\"reviews:write\"]) where no [[group]] ghosts block exists (undefined group)",
                "config_mutate::classify_error on the returned error; re-read the working-tree file bytes",
                "temp_env BUT_AGENT_HANDLE=admin; group_grant(&repo, \"refs/heads/main\", \"maintainers\", [\"badtoken\"]) (unparseable Authority token); re-read the working-tree file bytes",
                "temp_env::with_var(\"BUT_AGENT_HANDLE\", None::<&str>, ...); group_grant(&repo, \"refs/heads/main\", \"maintainers\", [\"reviews:write\"]) (no resolvable caller); config_mutate::classify_error; re-read the working-tree file bytes"
              ]
            },
            "end_state": {
              "must_observe": [
                "the undefined-group `group_grant(\"ghosts\", ...)` returns `Err` with a descriptive message naming the undefined group `ghosts`",
                "the bad-token `group_grant(\"maintainers\", [\"badtoken\"])` returns `Err` (a parse error, not a silent skip)",
                "the unset-handle `group_grant` returns `Err` with code `\"perm.denied\"` (Denial::no_handle, no anonymous action)",
                "the working-tree `.gitbutler/permissions.toml` bytes are identical to the pre-call capture after ALL THREE calls"
              ],
              "must_not_observe": [
                "a `[[group]] ghosts` block auto-created in the working-tree file (undefined-group grant must not create it)",
                "any change to the working-tree file bytes across the three fail-closed calls",
                "an empty/no-op success Ok where an Err is required (a silent-skip / anonymous-grant impl is excluded)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "description": "group_list under administration:read shows groups, grants, and membership; a caller without administration:read is denied.",
      "verify": "cargo test -p but-api group_list_under_admin_read",
      "primary": false,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api group_list + real but-authz load_governance_config + real gix",
        "negative_control": {
          "would_fail_if": [
            "group_list returned nothing / an empty set when groups exist",
            "group_list returned groups to a caller WITHOUT administration:read (the denied case must Err perm.denied)",
            "the listing were a static fixture rather than read from the committed config (guarded by asserting the maint member + merge grant appear)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "group_governance_base",
            "action": {
              "actor": "ci",
              "steps": [
                "temp_env BUT_AGENT_HANDLE=maint-read (role maintain -> administration:read) under #[serial_test::serial]",
                "group_list(&repo, \"refs/heads/main\")",
                "set BUT_AGENT_HANDLE=rust-reviewer (no administration:read); group_list again",
                "inspect both outputs"
              ]
            },
            "end_state": {
              "must_observe": [
                "the admin:read caller's `group_list` returns `Ok` and enumerates the `maintainers` group",
                "the listing shows the `merge` grant for `maintainers`",
                "the listing shows member `maint`",
                "the `rust-reviewer` caller (no administration:read) is denied with code `\"perm.denied\"`"
              ],
              "must_not_observe": [
                "an empty group listing when `maintainers` exists in committed config",
                "the `rust-reviewer` caller receiving the group listing"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "after admin group_add_member, the working-tree [[group]] maintainers members include rust-reviewer",
      "verify": "cargo test -p but-api group_add_member_writes_worktree_inert_until_committed",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "load_governance_config(repo, refs/heads/main) shows maintainers members EXCLUDE rust-reviewer and rust-reviewer's effective set excludes Authority::Merge after the add (target ref unchanged / inert)",
      "verify": "cargo test -p but-api group_add_member_writes_worktree_inert_until_committed",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "ref_id(repo, refs/heads/main) is identical before and after the group_add_member write (no commit performed); the result/printed caveat contains 'takes effect once committed to the target branch'",
      "verify": "cargo test -p but-api group_add_member_writes_worktree_inert_until_committed",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "after group_create + group_grant, the working-tree file has a [[group]] code-reviewers block with reviews:write + comments:write",
      "verify": "cargo test -p but-api group_create_grant_writes_worktree",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "the rewritten file still contains the unrelated [[group]] maintainers block (merge grant + maint member) and the rust-implementer role = \"write\" VALUE (unrelated entries + role sugar value-preserved)",
      "verify": "cargo test -p but-api group_create_grant_writes_worktree",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "description": "non-admin group_add_member returns Err; config_mutate::classify_error(&err).code == perm.denied and message contains administration:write",
      "verify": "cargo test -p but-api group_ops_non_admin_denied",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-7",
      "type": "test_criterion",
      "description": "the working-tree .gitbutler/permissions.toml is byte-for-byte unchanged after the denied non-admin group op",
      "verify": "cargo test -p but-api group_ops_non_admin_denied",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-8",
      "type": "test_criterion",
      "description": "group_grant against the UNDEFINED group \"ghosts\" returns Err (descriptive, no auto-create) and leaves the working-tree file byte-for-byte unchanged (no [[group]] ghosts block appears)",
      "verify": "cargo test -p but-api group_grant_fail_closed_undefined_group_bad_token_and_unset_handle",
      "maps_to_ac": "AC-4"
    },
    {
      "id": "TC-9",
      "type": "test_criterion",
      "description": "group_grant with token \"badtoken\" returns Err (parse error, not a silent skip) and leaves the working-tree file byte-for-byte unchanged",
      "verify": "cargo test -p but-api group_grant_fail_closed_undefined_group_bad_token_and_unset_handle",
      "maps_to_ac": "AC-4"
    },
    {
      "id": "TC-10",
      "type": "test_criterion",
      "description": "group_grant with BUT_AGENT_HANDLE unset returns Err code \"perm.denied\" (no anonymous action) and leaves the working-tree file byte-for-byte unchanged",
      "verify": "cargo test -p but-api group_grant_fail_closed_undefined_group_bad_token_and_unset_handle",
      "maps_to_ac": "AC-4"
    },
    {
      "id": "TC-11",
      "type": "test_criterion",
      "description": "group_list under administration:read enumerates maintainers, its merge grant, and member maint; a caller without administration:read is denied perm.denied",
      "verify": "cargo test -p but-api group_list_under_admin_read",
      "maps_to_ac": "AC-5"
    },
    {
      "id": "TC-12",
      "type": "test_criterion",
      "description": "but group add-member maintainers --principal rust-reviewer as admin exits 0 with the ref-pin caveat on stdout; the same as a non-admin exits 1 with perm.denied on stderr",
      "verify": "cargo test -p but group",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-13",
      "type": "test_criterion",
      "description": "but group add-member maintainers --as admin --principal rust-reviewer exits non-zero with a clap unknown/unsupported-flag error (no governed action performed, BUT_AGENT_HANDLE is the only identity source); no --as flag is defined on any Group subcommand",
      "verify": "cargo test -p but group",
      "maps_to_ac": "AC-3"
    }
  ]
}
-->
</details>
</output>
