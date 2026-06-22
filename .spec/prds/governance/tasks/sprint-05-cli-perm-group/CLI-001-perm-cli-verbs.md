# CLI-001: `but perm {list,grant,revoke}` + admin-write gating + ref-pin caveat + perm-list scoping + day-one seeding + honesty-grep coverage of governance.rs

## What this does

Authors the **net-new governed-config writer** and the first governance-management noun, `but perm`, as a thin CLI shim over net-new `but-api` functions. `grant`/`revoke` perform a read-modify-write on the **working-tree** `.gitbutler/permissions.toml` (registering a `[[principal]]` entry on first grant — the T-AUTHZ-007 day-one seeding path — preserving every unrelated entry and role-preset sugar), each authorizing `administration:write` at the **target ref** via the Sprint-02 AUTHZ-006 guard (`enforce_administration_write_gate`) BEFORE the write, and each printing the ref-pin caveat _"takes effect once committed to the target branch."_ The writer is **inert until committed** — it touches the working tree only, never stages/commits/touches a ref — so the next target-ref `load_governance_config` read is what makes the grant effective, exactly as every Sprint 01a–04 read proved. `list` shows a principal's **committed** effective `AuthoritySet` (read at the target ref) PLUS any working-tree (uncommitted) grant marked **PENDING**, and is reconnaissance-scoped: `but perm list --principal <other>` is denied `perm.denied` unless the caller IS `<other>` or holds `administration:read` (T-AUTHZ-025). All write/read logic is sited at the `but-api` boundary (`legacy/governance.rs`) so Sprint 06a's `perm_list`/`perm_grant`/`perm_revoke` Tauri commands re-invoke the SAME functions — and `governance.rs` (which makes the authorization decision for every governance write) is folded into the AUTHZ-007/008 honesty grep so its discipline is enforced mechanically, not assumed.

## Why

Sprint 05 · PRD UC-AUTHZ-01 (functional catalog the grant tokens parse against; AC-7 day-one effectiveness — seeded principals authorize per their bundles), UC-AUTHZ-03 (AC-4 `administration:write` to change governed config; AC-6 `perm list` reconnaissance scoping). Serves the gate clauses: an admin's `but perm grant` lands in the working-tree config with the ref-pin caveat; `but perm revoke` removes a token; the first grant to a fresh config SEEDS a `[[principal]]` and, once committed, AUTHORIZES that principal (T-AUTHZ-007); `but perm list` shows the committed effective set plus the new grant as PENDING; a non-admin's grant/revoke or cross-principal list is denied `perm.denied`. This is the first sprint that **persists** governed config — it builds the write side an admin uses to _manage_ the policy the enforcement core reads, and proves the write is structurally inert until landed (CAP-CONFIG-01) and admin-gated server-side (CAP-AUTHZ-01).

## How to verify

PRIMARY **AC-1** — `cargo test -p but-api perm_grant_writes_worktree_inert_until_committed` (Admin `perm_grant` writes the new permission into the working-tree `.gitbutler/permissions.toml` while the target-ref effective set is UNCHANGED — the inert-until-committed pair). Full gate set in the spec below.

## Scope

- `crates/but-authz/src/config.rs` (MODIFY) — add `pub fn permissions_path() -> &'static str` returning the `.gitbutler/permissions.toml` literal (today `PERMISSIONS_PATH` at :8 is private), so the net-new writer does not re-derive the path literal (honoring the `governance_present` doc: "single source of truth for the governance file paths — callers must not re-derive"). MAY add `#[derive(Serialize)]` to the wire structs (`PermissionsWire`/`PrincipalWire`/`GroupWire`, :400-429) — an ADDITIVE, non-semantic derive add (keep `#[serde(deny_unknown_fields)]`) — so the writer can re-serialize the raw TOML shape for the read-modify-write. Do NOT change loader behavior.
- `crates/but-authz/src/lib.rs` (MODIFY) — re-export `permissions_path` (and any newly-pub wire types) from the `config` `pub use` block (:13-16).
- `crates/but-api/src/legacy/governance.rs` (NEW) — the net-new `but-api` boundary AND the authorization-decision site for every governance write: `perm_list`, `perm_grant`, `perm_revoke` (each composing `enforce_administration_write_gate` for mutating verbs / the self-or-admin-read scope for list), plus the private TOML read-modify-write writer that edits the working-tree file preserving unrelated entries + role sugar, and the `GovWriteError`/caveat constant. CLI-002 extends THIS file with `group_*`. **This file is added to the honesty-grep ENFORCEMENT_PATHS by this task** (see below) so its authorization path holds the no-role-branch discipline mechanically.
- `crates/but-api/src/legacy/mod.rs` (MODIFY) — add `pub mod governance;` beside `pub mod config_mutate;`.
- `crates/but-authz/tests/invariant_build_gates.rs` (MODIFY — ADDITIVE ONLY) — add `crates/but-api/src/legacy/governance.rs` to `ENFORCEMENT_PATHS` (:23-30) and add an `AUTHORITY_POSITIVE_PATTERN` `assert_grep_has_matches` for `governance.rs` so an empty/stub governance.rs fails the grep. **NEVER weaken or remove an existing grep/path** — additive coverage only. The CLI shims (`args/perm.rs`/`command/perm.rs`) are thin arg-parse/print with NO authorization decision; **decision: governance.rs is the authorization-decision site and MUST be covered (it is); the CLI shims are NOT added to the grep — they hold no role/authority branch (the shim only parses args, calls the but-api fn, and prints), so adding them would assert a positive Authority pattern against files that legitimately contain none.** governance.rs coverage is the load-bearing fix.
- `crates/but/src/args/perm.rs` (NEW) — the `Perm` clap `Platform` + `Subcommands` (`List`/`Grant`/`Revoke`), mirroring `crates/but/src/args/config.rs`. NO `--as`/identity-override flag is defined on any `Perm` subcommand (S2).
- `crates/but/src/args/mod.rs` (MODIFY) — TWO edit sites: (1) add the `Perm(perm::Platform)` variant to the `Subcommands` enum (the variant list at ~:1040), and (2) add `pub mod perm;` to the `pub mod` declaration block (~:1272-1356). **Shared edit with CLI-002** — see DEPENDENCIES.
- `crates/but/src/command/help.rs` (MODIFY) — add ONLY the exhaustive-help grouping arm for `SubcommandDiscriminant::Perm` (mirror `Config` under `Group::OtherCommands`). This is compile-required by the generated `SubcommandDiscriminant` match; do not change unrelated help grouping/rendering behavior.
- `crates/but/src/command/perm.rs` (NEW) — the thin shim: resolve gix repo + the WORKSPACE TARGET ref (not HEAD) from `ctx`, call the `but-api` `perm_*` fn, print the result / ref-pin caveat / structured denial. Mirror `crates/but/src/command/config.rs::exec`.
- `crates/but/src/command/mod.rs` (MODIFY) — add `pub mod perm;`.
- `crates/but/src/lib.rs` (MODIFY) — add the `Subcommands::Perm(args::perm::Platform { cmd }) => …` dispatch arm beside `Subcommands::Config` (:448). **Shared edit with CLI-002** — see DEPENDENCIES.
- `crates/but/src/utils/metrics.rs` (MODIFY) — add the `Subcommands::Perm(..)` arm to the metrics match (mirror the `Subcommands::Config` arm at :175) so the build stays exhaustive. **Shared edit with CLI-002.**
- `crates/but-api/tests/perm_governance.rs` (NEW) — the PRIMARY proofs: admin write lands inert in the working tree; admin revoke removes the token + idempotent no-op; first-grant SEEDING + day-one effectiveness (T-AUTHZ-007); non-admin write denied; fail-closed (bad token / unset handle); perm-list scoping denial; list PENDING contract (composes `but-api` `perm_*` against real `but-authz` + real gix via `but_testsupport::writable_scenario`).
- `crates/but/tests/but/command/perm.rs` (NEW) — the happy-path CLI verb wiring + ref-pin-caveat / PENDING / revoke / `--as`-reject stdout contract (snapbox `env.but(...).assert()`).
- `crates/but/tests/but/command/mod.rs` (MODIFY, if a `mod` index exists) — add `mod perm;`.

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: CLI-001 - `but perm {list,grant,revoke}` + admin-write gating + ref-pin caveat + perm-list scoping + day-one seeding + honesty-grep coverage of governance.rs
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     L  (270 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-AUTHZ-01, UC-AUTHZ-03
CAPABILITIES: CAP-AUTHZ-01, CAP-CONFIG-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-api perm_grant_writes_worktree_inert_until_committed   |   cargo test -p but-api perm_grant_preserves_unrelated_entries_and_role_sugar   |   cargo test -p but-api perm_first_grant_seeds_principal_and_authorizes_when_committed   |   cargo test -p but-api perm_revoke_removes_token_and_idempotent_noop   |   cargo test -p but-api perm_grant_revoke_non_admin_denied   |   cargo test -p but-api perm_grant_fail_closed_bad_token_and_unset_handle   |   cargo test -p but-api perm_list_cross_principal_scoping   |   cargo test -p but-api perm_list_pending_marks_uncommitted_grant   |   cargo test -p but perm   |   cargo test -p but-authz invariant_build_gates
  check: cargo check -p but-api --all-targets   |   cargo check -p but --all-targets
  lint:  cargo clippy -p but-authz -p but-api -p but --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
A net-new governed-config writer + the `but perm` noun exist and are proven against real git: (1) an admin `perm_grant` reads the target-ref config, authorizes `administration:write`, then writes the new functional permission into the WORKING-TREE `.gitbutler/permissions.toml` (registering the `[[principal]]` on first grant) while the target-ref effective `AuthoritySet` for that principal is UNCHANGED — the inert-until-committed pair — and prints the ref-pin caveat; (2) the writer preserves every unrelated `[[principal]]`/`[[group]]` entry and role-preset sugar (it does NOT round-trip a normalized GovConfig); (3) the FIRST grant against an absent/empty config SEEDS a new `[[principal]]` block and, once that config is committed to the target ref, the seeded principal AUTHORIZES for the granted token (T-AUTHZ-007 day-one effectiveness); (4) admin `perm_revoke` removes the named token from the principal (preserving unrelated tokens/entries) and is an idempotent no-op (byte-unchanged) when the token is absent; (5) a non-admin `perm_grant`/`perm_revoke` is denied `perm.denied` (message names `administration:write`) and writes NOTHING; (6) `perm_grant` with an unparseable Authority token OR with `BUT_AGENT_HANDLE` unset fails closed (Err / non-zero exit) and writes NOTHING; (7) `perm_list --principal <other>` by a RESOLVED-but-non-admin caller (not self, no `administration:read`) is denied the SCOPE decision `perm.denied` and does NOT print `<other>`'s effective set, while self-read returns the caller's ACTUAL `contents:write` set; (8) `perm_list` prints the committed effective set PLUS a literal `PENDING` marker for an uncommitted working-tree grant. The CLI verb is a thin shim over the `but-api` `perm_*` functions (sited so Sprint 06a Tauri commands reuse them), resolves the WORKSPACE TARGET ref (not HEAD), and clap-rejects an `--as` override before any governed action. `cargo test -p but-api` + `-p but` green; clippy clean; the AUTHZ-007/008 honesty grep — now covering governance.rs — stays green (no role-name branching in any enforcement/authorization path, and governance.rs proven to carry the positive `Authority` axis so a stub fails the grep).

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST compose the EXISTING admin-write guard, never author a second one. Every mutating verb (`perm_grant`/`perm_revoke`) calls `but_api::legacy::config_mutate::enforce_administration_write_gate(&repo, target_ref)` (config_mutate.rs:18) BEFORE the write, and surfaces denial via `config_mutate::classify_error` (:31). Do NOT write a parallel `authorize(AdministrationWrite)` call in governance.rs — compose the guard.
- [MUST] MUST add governance.rs to the AUTHZ-007/008 honesty grep ENFORCEMENT_PATHS (ADDITIVE ONLY). In `crates/but-authz/tests/invariant_build_gates.rs`, add `const GOVERNANCE: &str = "crates/but-api/src/legacy/governance.rs";` and append it to `ENFORCEMENT_PATHS` (:23-30), and add an `assert_grep_has_matches("governance boundary must use the but-authz Authority axis", &workspace_root, AUTHORITY_POSITIVE_PATTERN, &[GOVERNANCE])?;` beside the existing CONFIG_MUTATE positive assertion (:67-72). This is REQUIRED — governance.rs makes the authorization decision for every governance write, so the no-role-branch scan AND the positive-Authority scan MUST cover it (a `match role { "admin" => }` branch there must fail the grep; an empty/stub governance.rs must fail the positive-pattern assertion). NEVER weaken, narrow, or remove an existing grep pattern or path. The CLI shims (args/perm.rs, command/perm.rs) are NOT added (they hold no authorization branch — only arg-parse + print — so a positive-Authority assertion against them would falsely fail); governance.rs coverage is the load-bearing fix. STATE this decision in the completion report.
- [MUST] MUST write the WORKING TREE only. The writer resolves `repo.workdir()` and `std::fs::write`s the file there (mirror `crates/but-api/tests/admin_write_guard.rs:158-164 write_worktree_permissions`). It MUST NOT call `git add`, stage, commit, or touch any ref. The inert-until-committed contract is STRUCTURAL — the next target-ref `load_governance_config` read is what makes the grant effective. A writer that commits is the cardinal failure AC-1's negative control catches. (The ONE place a commit happens is INSIDE the T-AUTHZ-007 test, via invoke_bash, AFTER the seeding write, to prove day-one effectiveness — the production `perm_grant` never commits.)
- [MUST] MUST preserve unrelated entries AND role-preset sugar via a read-modify-write on the TOML, NOT a round-trip through the normalized `GovConfig`. `GovConfig` desugars `role = "write"` to a flat set and folds group membership both directions at load (config.rs:303-371) — re-serializing it would LOSE role sugar and rewrite every principal. Approach: parse the working-tree file (or the target-ref blob on first write) into the raw wire structs (`PermissionsWire`/`PrincipalWire`/`GroupWire`, config.rs:400-429) that retain `role`/`permissions`/`groups`, mutate ONLY the target principal's `permissions` list (append a new `[[principal]]` if absent), and re-serialize via `toml::to_string`. NOTE the value/format consequence (see R4 decision below): `toml::to_string` emits CANONICAL TOML — it preserves all VALUES (role="write" stays role="write", unrelated principals/groups stay present with their values) but may DROP comments/blank lines and normalize key ordering. The contract is VALUE-preserving, NOT byte-verbatim, on a SUCCESSFUL write. Make the wire structs `pub` and add `#[derive(Serialize)]` (keep `#[serde(deny_unknown_fields)]`) so the writer round-trips the raw shape, not the normalized one. Cite config.rs:400-429.
- [MUST] MUST resolve the `.gitbutler/permissions.toml` path via a but-authz accessor, never a re-derived literal. Add `pub fn permissions_path() -> &'static str` to config.rs returning `PERMISSIONS_PATH` (:8) and re-export it (lib.rs:13-16); the writer calls `but_authz::permissions_path()`. This honors the `governance_present` doc contract (config.rs:42-43: "single source of truth for the governance file paths — callers must not re-derive `.gitbutler/*.toml` literals").
- [MUST] MUST parse grant/revoke tokens with `but_authz::Authority::parse` / `AuthoritySet::parse` (authority.rs:69, :193) — the tokens are typed `Authority` DATA, not role strings. An unparseable token fails closed (clap-level or `ParseAuthorityError` surfaced as a non-zero exit), NEVER a silent skip (AC-6). Group/principal names are config DATA, never enforcement branches.
- [MUST] MUST fail closed when the caller cannot be resolved. `perm_grant`/`perm_revoke`/`perm_list` resolve the caller via `but_authz::resolve_principal_from_env` (BUT_AGENT_HANDLE); when the handle is UNSET the call returns the `Denial::no_handle` → `perm.denied` (no anonymous action) and the mutating verbs write NOTHING (AC-6).
- [MUST] MUST seed on first grant AND prove day-one effectiveness. If `--principal <id>` has no `[[principal]]` entry (or the config file is absent/empty), `perm_grant` REGISTERS a new `[[principal]] id=<id> permissions=[<token>]` block in the working tree, preserving any existing blocks. The seeded config, once committed to the target ref, MUST make `load_governance_config(repo, target_ref)` authorize the seeded principal for the token (`effective_authority`/`principal_authorities` contains it) — this is T-AUTHZ-007 (UC-AUTHZ-01 AC-7), proven in AC-3 by an invoke_bash commit AFTER the seeding write. The production writer itself never commits.
- [MUST] MUST scope `perm_list --principal <other>` as a SCOPE decision, not a blanket unknown/error path. Allow only when the caller IS `<other>` (resolved from `BUT_AGENT_HANDLE`) OR the caller's target-ref effective set contains `Authority::AdministrationRead`. The cross-principal denial in AC-7 must fire for a caller who is RESOLVED + KNOWN (a registered principal present in committed config) but lacks administration:read and is not self — NOT because the caller is unknown. Print NOTHING about `<other>` (no effective-set leak). `--principal` omitted resolves to the caller's own set (self-read, always allowed) and returns the caller's ACTUAL set.
- [NEVER] NEVER branch on role names (read/triage/write/maintain/admin) or human-vs-AI predicates in any authorization/enforcement path. The AUTHZ-007/008 `invariant_build_gates` honesty grep now covers authorize.rs/config.rs/commit/gate.rs/merge_gate.rs/config_mutate.rs/forge.rs AND governance.rs (this task adds it) — governance.rs and the CLI shim must hold the same discipline (typed `Authority`, names are DATA). Do NOT regress the grep.
- [NEVER] NEVER overload GitButler's pre-existing `Permission` (the repo-access lock) in any authorization path — use `Authority`/`AuthoritySet` exclusively (UC-AUTHZ §9 distinct-crate invariant).
- [NEVER] NEVER bury the write/read logic in `crates/but/`. The `perm_*` functions live at the `but-api` boundary (legacy/governance.rs) so Sprint 06a's `perm_grant`/`perm_revoke`/`perm_list` Tauri commands (api-design.md:84-86) reuse them. The `but` verb is a thin arg-parse + print shim.
- [NEVER] NEVER define an `--as`/`--principal-override`/identity-override flag on any `Perm` subcommand. The caller identity comes from `BUT_AGENT_HANDLE` only (mirror confinement.rs's `--as` rejection). `but perm grant --as admin ...` MUST clap-reject (unknown flag, non-zero exit) BEFORE any governed action (S2 / TC-11).
- [STRICTLY] STRICTLY treat the AUTHZ-006 admin guard (config_mutate.rs) as a CONSUMED Sprint-02 seam. If it genuinely lacks a needed accessor (e.g. you need the resolved caller principal for the perm-list scope decision), add a small additive helper there or in governance.rs — do NOT fork a second admin check. The perm-list self-or-admin-read scope is a NEW read-side predicate (T-AUTHZ-025), distinct from the write guard; author it in governance.rs using `but_authz::resolve_principal_from_env` + `load_governance_config` + `AuthoritySet::contains(Authority::AdministrationRead)`.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [x] AC-1 [PRIMARY]: Admin `perm_grant` writes the new permission into the working-tree config while the target-ref effective set is unchanged (inert-until-committed pair), and prints the ref-pin caveat
- [x] AC-2: The writer preserves unrelated principal/group entries and role-preset sugar (read-modify-write, value-preserving, not a normalized round-trip)
- [x] AC-3: The first grant against an absent/empty config SEEDS a new `[[principal]]` block and, once committed, AUTHORIZES the seeded principal for the token (T-AUTHZ-007 day-one effectiveness)
- [x] AC-4: Admin `perm_revoke` removes the named token (preserving unrelated tokens/entries) and is an idempotent no-op (byte-unchanged) when the token is absent
- [x] AC-5: A non-admin `perm_grant`/`perm_revoke` is denied `perm.denied` (names `administration:write`) and writes nothing
- [x] AC-6: `perm_grant` with an unparseable token OR with `BUT_AGENT_HANDLE` unset fails closed (Err / non-zero) and writes nothing
- [x] AC-7: `perm_list --principal <other>` by a resolved-but-non-admin caller is denied the SCOPE decision `perm.denied` with no leak; self-read returns the caller's ACTUAL set
- [x] AC-8: `perm_list` prints the committed effective set PLUS a literal `PENDING` marker for an uncommitted working-tree grant
- [x] All verification gates pass; only write_allowed files modified; the honesty grep — now covering governance.rs — stays green

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: Admin `perm_grant` writes the new permission into the working-tree config while the target-ref effective set is unchanged (inert-until-committed pair) [PRIMARY]
  GIVEN: refs/heads/main with committed `.gitbutler/permissions.toml` where `admin` holds `administration:write` and `rust-implementer` holds `["contents:write"]` (NOT reviews:write), and a clean working tree
  WHEN:  `perm_grant(&repo, target_ref, principal="rust-implementer", ["reviews:write"])` runs with BUT_AGENT_HANDLE=admin (via temp_env::with_var under #[serial_test::serial])
  THEN:  the call returns Ok; the WORKING-TREE `.gitbutler/permissions.toml` now contains `reviews:write` under `rust-implementer`; AND `load_governance_config(repo, "refs/heads/main").principal_authorities("rust-implementer")` STILL does NOT contain `Authority::ReviewsWrite` (the grant is inert — target ref unchanged); the `refs/heads/main` ref_id is identical before and after the write (no commit); the returned/printed caveat contains "takes effect once committed to the target branch"
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api perm_grant + real but-authz load_governance_config + real gix repo (committed target-ref config vs working-tree write) via but_testsupport::writable_scenario
  VERIFY: cargo test -p but-api perm_grant_writes_worktree_inert_until_committed
  SCENARIO (negative controls): would fail if the writer wrote nothing (working-tree file would lack reviews:write); would fail if the writer COMMITTED the grant (the target-ref effective set would then contain ReviewsWrite, breaking the inert assertion; the ref_id-before==after assertion catches the commit too); would pass against a stub that always returns Ok without writing (guarded by asserting the working-tree file literally contains reviews:write under rust-implementer); would fail if the caveat string were absent (silent success)

AC-2: The writer preserves unrelated principal/group entries and role-preset sugar (read-modify-write, value-preserving, not a normalized round-trip)
  GIVEN: a committed `.gitbutler/permissions.toml` with `admin` (administration:write), a `rust-implementer` principal carrying `role = "write"` (role sugar, NOT an expanded list), an unrelated `security-bot` principal, and a `[[group]]` `code-reviewers` (permissions=["reviews:write"], members=["rust-reviewer"])
  WHEN:  `perm_grant(&repo, target_ref, principal="rust-implementer", ["merge"])` runs as admin
  THEN:  the rewritten working-tree file STILL carries the `role = "write"` VALUE for rust-implementer (sugar preserved as a role assignment, NOT desugared to an expanded flat permissions list), STILL contains the unrelated `security-bot` principal (its statuses:write value present), STILL contains the `[[group]] code-reviewers` block (its reviews:write grant + rust-reviewer member present), and now adds `merge` to rust-implementer's permissions — i.e. only the target principal's permission list changed in value
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api perm_grant + real gix working-tree read-back; assert on the parsed wire shape + raw TOML text (VALUE-preserving), not a normalized GovConfig
  VERIFY: cargo test -p but-api perm_grant_preserves_unrelated_entries_and_role_sugar
  SCENARIO (negative controls): would fail if the writer re-serialized a normalized GovConfig — `role = "write"` would be gone (desugared to a flat permissions list) and the group's member-direction fold would have rewritten security-bot/principals; would fail if the unrelated security-bot or code-reviewers block were dropped (their values absent); would pass against a no-op writer only if AC-1 (which requires the new token to land) also passed — it cannot, so the pair pins a real read-modify-write

AC-3: The first grant against an absent/empty config SEEDS a new `[[principal]]` block and, once committed, AUTHORIZES the seeded principal for the token (T-AUTHZ-007 day-one effectiveness)
  GIVEN: a target ref refs/heads/main whose committed tree has NO `[[principal]]` entry for `rust-implementer` — the seeding variant (perm_governance_seed) commits a `.gitbutler/permissions.toml` holding ONLY `admin` (administration:write) and `.gitbutler/gates.toml`; `rust-implementer` is absent from committed config; clean working tree
  WHEN:  admin `perm_grant(&repo, "refs/heads/main", "rust-implementer", ["reviews:write"])` runs (REGISTERS a new principal), THEN the resulting working-tree `.gitbutler/permissions.toml` is committed to refs/heads/main via invoke_bash, THEN `load_governance_config(&repo, "refs/heads/main")` is loaded
  THEN:  the working-tree file gains a NEW `[[principal]] id="rust-implementer"` block carrying `reviews:write` (the seeding half — the admin block is preserved); AND after the commit, `load_governance_config(...).principal_authorities("rust-implementer")` CONTAINS `Authority::ReviewsWrite` (the day-one-effectiveness half — the seeded principal authorizes per its bundle)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api perm_grant (registration) + real invoke_bash commit + real but-authz load_governance_config (day-one authorize) + real gix via but_testsupport::writable_scenario
  VERIFY: cargo test -p but-api perm_first_grant_seeds_principal_and_authorizes_when_committed
  SCENARIO (negative controls): a writer that errors/no-ops on an absent principal fails the SEEDING half (no rust-implementer block appears); a writer that registers the block but the seeded config does not authorize once committed fails the DAY-ONE half (principal_authorities lacks ReviewsWrite — the commit step is what makes it effective); a writer that drops the existing admin block fails (admin's administration:write must survive); maps to T-AUTHZ-007 (UC-AUTHZ-01 AC-7 "seeded principals authorize per their bundles")

AC-4: Admin `perm_revoke` removes the named token (preserving unrelated tokens/entries) and is an idempotent no-op (byte-unchanged) when the token is absent
  GIVEN: the AC-1 committed config where `rust-implementer` holds `["contents:write"]` (the token to revoke) PLUS an unrelated `security-bot` principal and a `[[group]] code-reviewers`; clean working tree; the working-tree file captured before the no-op call
  WHEN:  admin `perm_revoke(&repo, "refs/heads/main", "rust-implementer", ["contents:write"])` runs (removes a held token), THEN separately admin `perm_revoke(&repo, "refs/heads/main", "rust-implementer", ["merge"])` runs (revokes a token the principal does NOT hold — the idempotent path)
  THEN:  after the first revoke, the working-tree `.gitbutler/permissions.toml` no longer lists `contents:write` under `rust-implementer`, the `rust-implementer` `[[principal]]` entry SURVIVES (possibly with an empty permissions list), and the unrelated `security-bot` principal + `[[group]] code-reviewers` are still present; the returned/printed caveat contains "takes effect once committed to the target branch"; AND the idempotent revoke of `merge` (not held) returns Ok and leaves the working-tree file BYTE-UNCHANGED from its pre-call capture
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api perm_revoke + real gix working-tree read-back via but_testsupport::writable_scenario
  VERIFY: cargo test -p but-api perm_revoke_removes_token_and_idempotent_noop
  SCENARIO (negative controls): a stub that always returns Ok without removing fails (contents:write still present under rust-implementer after the first revoke); a writer that drops the whole `rust-implementer` entry OR drops the unrelated security-bot/code-reviewers blocks fails (the entry must survive, unrelated entries must persist); a revoke that ERRORS or MUTATES the file on a not-held token fails the idempotent half (byte-unchanged required)

AC-5: A non-admin `perm_grant`/`perm_revoke` is denied `perm.denied` (names administration:write) and writes nothing
  GIVEN: the AC-1 committed config; the caller `rust-implementer` holds `["contents:write"]` only (NO administration:write) at the target ref; the working-tree config is captured byte-for-byte before the call
  WHEN:  `perm_grant(&repo, target_ref, principal="rust-implementer", ["administration:write"])` (a self-grant attempt) runs with BUT_AGENT_HANDLE=rust-implementer, AND separately `perm_revoke(&repo, target_ref, "admin", ["administration:write"])` runs with BUT_AGENT_HANDLE=rust-implementer (a non-admin revoke attempt)
  THEN:  both calls return Err; `config_mutate::classify_error(&err).code == "perm.denied"`; the message contains "administration:write"; AND the working-tree `.gitbutler/permissions.toml` is byte-for-byte UNCHANGED from before the calls (the gate ran BEFORE any write, for BOTH grant and revoke)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api perm_grant/perm_revoke composing enforce_administration_write_gate + real but-authz + real gix
  VERIFY: cargo test -p but-api perm_grant_revoke_non_admin_denied
  SCENARIO (negative controls): would pass (wrongly) against a stub that always writes / never gates — guarded by asserting the working-tree file is unchanged AND classify_error.code=="perm.denied"; would fail-open if the writer ran before the gate (the file would change) — the byte-for-byte-unchanged assertion catches that ordering bug; relies on the self-grant/non-admin-revoke being inert because the caller lacks administration:write at the target ref (CAP-CONFIG-01), not on a role check

AC-6: `perm_grant` with an unparseable Authority token OR with `BUT_AGENT_HANDLE` unset fails closed (Err / non-zero) and writes nothing
  GIVEN: the AC-1 committed config; the working-tree config captured byte-for-byte before each call
  WHEN:  (a) admin `perm_grant(&repo, "refs/heads/main", "rust-implementer", ["badtoken"])` runs with BUT_AGENT_HANDLE=admin (an unparseable Authority token); AND (b) `perm_grant(&repo, "refs/heads/main", "rust-implementer", ["reviews:write"])` runs with BUT_AGENT_HANDLE UNSET (no resolvable caller)
  THEN:  (a) returns Err (a ParseAuthorityError surfaced, classify_error code "config.invalid" or the parse-error code — NOT a silent skip) and the working-tree file is byte-for-byte UNCHANGED; (b) returns Err with code "perm.denied" (resolve_principal_from_env → Denial::no_handle, no anonymous action) and the working-tree file is byte-for-byte UNCHANGED
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api perm_grant token parse (Authority::parse) + real resolve_principal_from_env + real gix working-tree read-back
  VERIFY: cargo test -p but-api perm_grant_fail_closed_bad_token_and_unset_handle
  SCENARIO (negative controls): an impl that silently skips an unparseable token (returns Ok writing nothing, or writes a partial set) fails — the bad-token call MUST Err and the file MUST be unchanged; an impl that performs an anonymous grant when BUT_AGENT_HANDLE is unset fails — the unset-handle call MUST Err perm.denied and write nothing; a fail-OPEN impl that writes before parsing/resolving fails the byte-for-byte-unchanged assertion

AC-7: `perm_list --principal <other>` by a resolved-but-non-admin caller is denied the SCOPE decision `perm.denied` (no leak); self-read returns the caller's ACTUAL set
  GIVEN: the AC-1 committed config plus a `maint` principal holding `merge`; caller `rust-implementer` is a REGISTERED principal in committed config holding `["contents:write"]` only (resolved + known, but no administration:read, not maint)
  WHEN:  `perm_list(&repo, target_ref, Some("maint"))` runs with BUT_AGENT_HANDLE=rust-implementer (a resolved, known, non-admin caller listing another principal)
  THEN:  the call returns Err with code "perm.denied" as the SCOPE decision (caller is resolved + present in committed config, lacks administration:read, target ≠ self) — NOT an unknown-principal/blanket-error denial; the returned value/rendered output does NOT contain `maint`'s effective set (no `merge`, no authority enumeration) — the no-topology-reconnaissance property; AND `perm_list(&repo, target_ref, None)` (self) by the same caller returns Ok and shows rust-implementer's ACTUAL `contents:write` set (not merely "an Ok")
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api perm_list scope predicate (resolved-known caller) + real but-authz load_governance_config + real gix
  VERIFY: cargo test -p but-api perm_list_cross_principal_scoping
  SCENARIO (negative controls): would leak (fail must_not_observe) if the scope predicate were absent — maint's effective set would be returned to a non-admin caller; would FAIL to pin a real scope predicate if the denial fired for the WRONG reason (an unknown/blanket-error path) — guarded by the caller being a registered principal whose own self-read returns the ACTUAL contents:write set (proving the caller resolves and is known, so the cross-principal denial is the scope decision); the admin-read positive path (a caller holding administration:read CAN list others) is covered by TC-9

AC-8: `perm_list` prints the committed effective set PLUS a literal `PENDING` marker for an uncommitted working-tree grant
  GIVEN: the AC-1 committed config (rust-implementer = contents:write at the target ref); then an admin `perm_grant(... "reviews:write")` has written reviews:write to the working tree (uncommitted) — i.e. AC-1's post-state
  WHEN:  `perm_list(&repo, target_ref, Some("rust-implementer"))` runs as admin
  THEN:  the output shows the COMMITTED effective set (`contents:write`, read at the target ref) AND a literal `PENDING` marker against the uncommitted `reviews:write` grant (read from the working-tree file and diffed against the committed set); the committed `contents:write` is NOT marked pending
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api perm_list reading both the target-ref committed config and the working-tree file + real gix
  VERIFY: cargo test -p but-api perm_list_pending_marks_uncommitted_grant
  SCENARIO (negative controls): would fail if list treated the working tree as committed (reviews:write would show with no PENDING marker, contents:write would be indistinguishable); would fail if list omitted the pending grant entirely (read only the target ref); would pass against a static fixture only if the marker were computed from the real committed-vs-working diff — guarded by asserting contents:write is NOT pending while reviews:write IS

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, happy_path): after admin perm_grant, the working-tree .gitbutler/permissions.toml contains reviews:write under rust-implementer
    VERIFY: cargo test -p but-api perm_grant_writes_worktree_inert_until_committed
- TC-2 (-> AC-1, structural): load_governance_config(repo, "refs/heads/main").principal_authorities("rust-implementer") does NOT contain Authority::ReviewsWrite after the grant, AND ref_id(repo, "refs/heads/main") is identical before and after the write (target ref unchanged / inert / no commit)
    VERIFY: cargo test -p but-api perm_grant_writes_worktree_inert_until_committed
- TC-3 (-> AC-1, happy_path): the perm_grant result/printed caveat contains "takes effect once committed to the target branch"
    VERIFY: cargo test -p but-api perm_grant_writes_worktree_inert_until_committed
- TC-4 (-> AC-2, edge): the rewritten file still carries the role = "write" VALUE for rust-implementer and the unrelated security-bot + code-reviewers entries (role sugar + unrelated entries value-preserved)
    VERIFY: cargo test -p but-api perm_grant_preserves_unrelated_entries_and_role_sugar
- TC-5 (-> AC-3, happy_path): admin perm_grant against a config with NO rust-implementer entry registers a new [[principal]] id="rust-implementer" block carrying reviews:write, preserving the existing admin block (seeding half)
    VERIFY: cargo test -p but-api perm_first_grant_seeds_principal_and_authorizes_when_committed
- TC-6 (-> AC-3, structural): after committing the seeded config to refs/heads/main, load_governance_config(...).principal_authorities("rust-implementer") CONTAINS Authority::ReviewsWrite (day-one effectiveness — T-AUTHZ-007)
    VERIFY: cargo test -p but-api perm_first_grant_seeds_principal_and_authorizes_when_committed
- TC-7 (-> AC-4, happy_path): after admin perm_revoke "contents:write", the working-tree file no longer lists contents:write under rust-implementer, the rust-implementer entry + unrelated security-bot/code-reviewers survive
    VERIFY: cargo test -p but-api perm_revoke_removes_token_and_idempotent_noop
- TC-8 (-> AC-4, edge): admin perm_revoke of a token the principal does NOT hold ("merge") returns Ok and leaves the working-tree file byte-for-byte unchanged (idempotent no-op)
    VERIFY: cargo test -p but-api perm_revoke_removes_token_and_idempotent_noop
- TC-9 (-> AC-5, error): non-admin perm_grant AND non-admin perm_revoke each return Err; config_mutate::classify_error(&err).code == "perm.denied" and message contains "administration:write"
    VERIFY: cargo test -p but-api perm_grant_revoke_non_admin_denied
- TC-10 (-> AC-5, structural): the working-tree .gitbutler/permissions.toml is byte-for-byte unchanged after the denied non-admin grant AND revoke
    VERIFY: cargo test -p but-api perm_grant_revoke_non_admin_denied
- TC-11 (-> AC-6, error): perm_grant with token "badtoken" returns Err (parse error, not a silent skip) and leaves the working-tree file byte-for-byte unchanged
    VERIFY: cargo test -p but-api perm_grant_fail_closed_bad_token_and_unset_handle
- TC-12 (-> AC-6, error): perm_grant with BUT_AGENT_HANDLE unset returns Err code "perm.denied" (no anonymous action) and leaves the working-tree file byte-for-byte unchanged
    VERIFY: cargo test -p but-api perm_grant_fail_closed_bad_token_and_unset_handle
- TC-13 (-> AC-7, edge): perm_list(None) self-read by the resolved-known non-admin rust-implementer returns Ok showing its ACTUAL contents:write set; perm_list(Some("maint")) by the same caller is Err "perm.denied" (scope decision) with no leak of maint's merge; an administration:read holder listing maint returns Ok
    VERIFY: cargo test -p but-api perm_list_cross_principal_scoping
- TC-14 (-> AC-8, happy_path): perm_list shows committed contents:write NOT marked PENDING and uncommitted reviews:write marked PENDING
    VERIFY: cargo test -p but-api perm_list_pending_marks_uncommitted_grant
- TC-15 (-> AC-1/AC-4/AC-5, integration CLI wiring): `but perm grant --principal rust-implementer reviews:write` as admin exits 0 and stdout contains the ref-pin caveat; `but perm revoke --principal rust-implementer contents:write` as admin exits 0 with the caveat; the grant as a non-admin exits 1 with stderr containing "perm.denied"
    VERIFY: cargo test -p but perm
- TC-16 (-> AC-1, structural CLI): the admin `but perm grant` CLI invocation does NOT move refs/heads/main (ref_id before == after across the subprocess; the working-tree file is staged-but-not-committed at most, i.e. an uncommitted working-tree edit)
    VERIFY: cargo test -p but perm
- TC-17 (-> AC-5/AC-6, error CLI): `but perm grant --as admin --principal rust-implementer reviews:write` exits non-zero with a clap unknown/unsupported-flag error (no governed action performed, BUT_AGENT_HANDLE is the only identity source); no `--as` flag is defined on any Perm subcommand
    VERIFY: cargo test -p but perm

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01, CAP-CONFIG-01
provides: the net-new governed-config writer + the `but perm {list,grant,revoke}` noun, sited at the but-api boundary for Sprint 06a Tauri reuse; admin-gated (composes AUTHZ-006) inert-until-committed working-tree writes; first-grant principal seeding + day-one effectiveness (T-AUTHZ-007); idempotent revoke; fail-closed token-parse + caller-resolution; perm-list reconnaissance scoping (self-or-administration:read); governance.rs folded into the AUTHZ-007/008 honesty grep ENFORCEMENT_PATHS
consumes: but_api::legacy::config_mutate::{enforce_administration_write_gate, classify_error, AdminWriteGateError}, but_authz::{load_governance_config, governance_present, resolve_principal_from_env, Authority, AuthoritySet, GovConfig, PrincipalId, permissions_path (NEW), Denial}, gix::Repository::workdir, but_testsupport::{writable_scenario, invoke_bash}
boundary_contracts:
  - CAP-AUTHZ-01: every governance write verb authorizes administration:write (own ∪ groups, read at the target ref) via the AUTHZ-006 guard before mutating config; perm list enforces the reconnaissance scope (self or administration:read). Real-service proof: a non-admin perm_grant/perm_revoke is denied perm.denied and writes nothing; a resolved-known non-admin cross-principal perm_list is denied with no leak; the honesty grep proves governance.rs branches only on the typed Authority axis.
  - CAP-CONFIG-01: the CLI write path writes inert-until-committed config to the working tree only; effectiveness comes from the next target-ref read, so a self-grant of administration:write on the working tree cannot authorize the same change — but a FIRST grant, once committed, DOES authorize the seeded principal (T-AUTHZ-007). Real-service proof: after a grant, the working-tree file changed but the target-ref effective set did not (ref_id unchanged); after a seeded config is committed, load_governance_config authorizes the seeded principal.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-authz/src/config.rs (MODIFY) — add `pub fn permissions_path() -> &'static str`; MAY make the wire structs `pub` and add `#[derive(Serialize)]` (additive, non-semantic; keep `#[serde(deny_unknown_fields)]`); no loader/normalize semantic change
  - crates/but-authz/src/lib.rs (MODIFY) — re-export permissions_path (and any newly-pub wire types) from the config pub use block
  - crates/but-authz/tests/invariant_build_gates.rs (MODIFY — ADDITIVE ONLY) — add crates/but-api/src/legacy/governance.rs to ENFORCEMENT_PATHS and an AUTHORITY_POSITIVE_PATTERN assert_grep_has_matches for governance.rs; NEVER weaken/remove an existing grep or path
  - crates/but-api/src/legacy/governance.rs (NEW) — perm_list/perm_grant/perm_revoke + the private TOML read-modify-write writer + GovWriteError + the REF_PIN_CAVEAT constant
  - crates/but-api/src/legacy/mod.rs (MODIFY) — `pub mod governance;`
  - crates/but/src/args/perm.rs (NEW) — Perm clap Platform + Subcommands (List/Grant/Revoke); NO --as/identity-override flag
  - crates/but/src/args/mod.rs (MODIFY, SHARED with CLI-002) — `pub mod perm;` (in the pub-mod block ~:1272-1356) + `Perm(perm::Platform)` variant (in the Subcommands enum ~:1040) ONLY; do not touch the Group variant (CLI-002 owns it)
  - crates/but/src/command/help.rs (MODIFY, SHARED with CLI-002) — add ONLY `SubcommandDiscriminant::Perm => Group::OtherCommands` (or the equivalent single Perm grouping arm if nearby help groups are renamed) so the exhaustive generated-discriminant match compiles; do not change ordering, text, hidden-command behavior, truncation, or unrelated grouping
  - crates/but/src/command/perm.rs (NEW) — the thin CLI shim (resolves the WORKSPACE TARGET ref, not HEAD)
  - crates/but/src/command/mod.rs (MODIFY, SHARED with CLI-002) — `pub mod perm;` ONLY
  - crates/but/src/lib.rs (MODIFY, SHARED with CLI-002) — the Subcommands::Perm dispatch arm ONLY (beside :448)
  - crates/but/src/utils/metrics.rs (MODIFY, SHARED with CLI-002) — the Subcommands::Perm metrics arm ONLY (mirror :175)
  - crates/but-api/tests/perm_governance.rs (NEW) — the PRIMARY but-api proofs (AC-1..AC-8)
  - crates/but/tests/but/command/perm.rs (NEW) — happy-path CLI verb wiring + ref-pin-caveat / PENDING / revoke / --as-reject stdout (TC-15..TC-17)
  - crates/but/tests/but/command/mod.rs (MODIFY, ONLY IF a mod index exists) — `mod perm;`
writeProhibited:
  - crates/but-api/src/legacy/config_mutate.rs — CONSUME-only (the AUTHZ-006 admin guard); compose enforce_administration_write_gate, do not author a second admin check. If it lacks a needed accessor, FLAG it; add an additive helper in governance.rs instead of forking the guard.
  - crates/but-api/src/legacy/merge_gate.rs, crates/but-api/src/commit/gate.rs, crates/but-api/src/legacy/forge.rs — the Sprint-01b/01a/forge gates; CONSUME-only, not touched
  - crates/but-authz/src/{authorize.rs, denial.rs, principal.rs, authority.rs} — the union/primitive layer is closed; do not re-open (you only ADD permissions_path + optional Serialize derive to config.rs)
  - crates/but-authz/tests/invariant_build_gates.rs — ADDITIVE coverage ONLY (add governance.rs + its positive assertion); do NOT weaken or remove any existing grep pattern or path
  - crates/but/src/args/group.rs, crates/but/src/command/group.rs, the Group variant/dispatch/metrics arm — owned by CLI-002
  - any gitbutler-* crate (no new gitbutler-* usage)
  - Any file not in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-api/src/legacy/config_mutate.rs (1-44)
   Focus: enforce_administration_write_gate(repo, target_ref) loads target-ref cfg, resolves BUT_AGENT_HANDLE, authorizes AdministrationWrite; classify_error -> AdminWriteGateError{code,message}. This is the guard EVERY mutating verb composes — do NOT re-implement.
2. crates/but-api/tests/admin_write_guard.rs (8-68, 96-176)
   Focus: the canonical admin-gate denial test pattern — temp_env::with_var("BUT_AGENT_HANDLE", ...) under #[serial_test::serial], writable_scenario("checkout-head-info") + invoke_bash committing .gitbutler/permissions.toml at main, write_worktree_permissions via repo.workdir() (the EXACT working-tree write helper the writer mirrors), committed_blob_text for the byte-for-byte-unchanged assertion (AC-5/AC-6). Mirror this file's shape for perm_governance.rs.
3. crates/but-authz/src/config.rs (8-9, 24-59, 258-301, 400-429)
   Focus: PERMISSIONS_PATH literal (the accessor wraps it); load_governance_config + governance_present (the target-ref read the inert assertion uses); load_governance_config_inner + read_config_blob (how the committed blob is read — your writer reads the working-tree file, NOT this); the private PermissionsWire/PrincipalWire/GroupWire serde structs you make `pub` + add `#[derive(Serialize)]` for the read-modify-write
4. crates/but-authz/src/config.rs (303-371)
   Focus: normalize_permissions — PROVES why you must NOT round-trip GovConfig: it desugars role and folds group membership both directions, so re-serializing a GovConfig loses role sugar and rewrites principals (AC-2's negative control)
5. crates/but-authz/src/authority.rs (69-110, 193-282)
   Focus: Authority::parse / AuthoritySet::parse (the bad-token AC-6 path: parse Err) / contains / iter / name — grant/revoke tokens parse here (typed, names are DATA); AdministrationRead is the perm-list scope authority; AuthoritySet::iter for rendering the effective set
6. crates/but-authz/src/authorize.rs (24-31, 51-58, 67-102)
   Focus: authorize / effective_authority / resolve_principal_from_env (the AC-6 unset-handle path: Denial::no_handle) — the perm-list self-or-admin-read predicate resolves the caller from env, loads the target-ref config, and checks AdministrationRead; the self case compares caller id to --principal
7. crates/but/src/args/config.rs (1-12, 298-345)
   Focus: the clap Platform { cmd: Option<Subcommands> } + per-subcommand enum shape to mirror in args/perm.rs (List/Grant/Revoke with --principal + positional authority tokens); note NO --as flag exists on config subcommands — perm mirrors that (S2)
8. crates/but/src/args/mod.rs (~1040 Subcommands enum variant list, ~1272-1356 pub mod declaration block)
   Focus: the TWO additive edit sites for a new noun — the enum variant (Perm(perm::Platform)) at the variant list AND the `pub mod perm;` at the module-declaration block; mirror the Config noun's two insertions (R7)
9. crates/but/src/lib.rs (448-487)
   Focus: the Subcommands::Config dispatch arm — how a noun resolves ctx, calls the command module, maps errors to CliError; mirror for the Subcommands::Perm arm
10. crates/but/src/command/config.rs (85-... , 138, 432)
   Focus: pub async fn exec signature and `let repo = ctx.repo.get()?;` + how the shim resolves the WORKSPACE TARGET ref (not HEAD) to pass to the but-api fn (R6)
11. crates/but/tests/but/command/confinement.rs (1-67, 69-130)
   Focus: the snapbox CLI harness — Sandbox::open_scenario_with_target_and_default_settings, env.invoke_bash committing governance config at main, env.but("...").env("BUT_AGENT_HANDLE", ...).assert().success()/.output(), asserting stderr contains "perm.denied", AND the `--as` rejection pattern (clap unknown-flag, non-zero exit) to mirror for TC-17. Mirror for crates/but/tests/but/command/perm.rs (TC-15..TC-17). NOTE the target-ref governance is committed at main here.
12. crates/but-api/tests/commit_gate.rs (1-98)
   Focus: the governed-path integration shape — but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache(), Denial downcast, the `ref_id(&repo, "refs/heads/main")` helper (:11, :48) for the before/after structural assertion (AC-1 inert + TC-2 / TC-16); reuse ref_id for the target-ref-unchanged assertion
13. crates/but-authz/tests/invariant_build_gates.rs (12-30, 60-72)
   Focus: ENFORCEMENT_PATHS + AUTHORITY_POSITIVE_PATTERN + the assert_grep_has_matches calls — the ADDITIVE edit adds GOVERNANCE to ENFORCEMENT_PATHS and a positive assert_grep_has_matches for governance.rs beside CONFIG_MUTATE; do NOT weaken any existing pattern/path (S1)
14. .spec/prds/governance/10-technical-requirements/04-api-design.md (82-114, 131-138)
   Focus: the Tauri perm_list/perm_grant/perm_revoke table (the functions you site for Sprint 06a reuse), the route->Authority table (administration:read or self for list; administration:write for grant/revoke), the ref-pin caveat wording, and the {error:{code,message,remediation_hint}} contract + exit 1

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- perm grant inert (PRIMARY): `cargo test -p but-api perm_grant_writes_worktree_inert_until_committed`  -> Exit 0; working-tree file gains reviews:write, target-ref effective set unchanged, ref_id(main) before==after, caveat printed
- writer preserves entries + sugar: `cargo test -p but-api perm_grant_preserves_unrelated_entries_and_role_sugar`  -> Exit 0; role="write" value + unrelated blocks present, only target principal's permissions changed in value
- first-grant seeding + day-one: `cargo test -p but-api perm_first_grant_seeds_principal_and_authorizes_when_committed`  -> Exit 0; new [[principal]] rust-implementer block registered preserving admin; after commit, load_governance_config authorizes rust-implementer for reviews:write (T-AUTHZ-007)
- revoke removes + idempotent: `cargo test -p but-api perm_revoke_removes_token_and_idempotent_noop`  -> Exit 0; contents:write removed under rust-implementer, entry + unrelated blocks survive; not-held revoke is byte-unchanged
- non-admin denied, no write: `cargo test -p but-api perm_grant_revoke_non_admin_denied`  -> Exit 0; perm.denied naming administration:write for BOTH grant and revoke; working-tree file byte-for-byte unchanged
- fail-closed bad-token + unset-handle: `cargo test -p but-api perm_grant_fail_closed_bad_token_and_unset_handle`  -> Exit 0; bad token Errs (no silent skip), unset handle Errs perm.denied; file byte-for-byte unchanged in both
- perm-list scoping: `cargo test -p but-api perm_list_cross_principal_scoping`  -> Exit 0; resolved-known non-admin cross-principal list denied (scope decision) with no leak; self-read returns ACTUAL contents:write; admin-read list allowed
- perm-list PENDING: `cargo test -p but-api perm_list_pending_marks_uncommitted_grant`  -> Exit 0; committed set unmarked, uncommitted grant marked PENDING
- CLI verb wiring: `cargo test -p but perm`  -> Exit 0; admin grant/revoke exit 0 with caveat on stdout; non-admin grant exits 1 with perm.denied on stderr; `--as` clap-rejected; ref_id(main) unchanged across the CLI grant
- honesty grep (guard-rail stays green, NOW covering governance.rs): `cargo test -p but-authz invariant_build_gates`  -> Exit 0; no role-label/human-vs-AI branching in enforcement paths incl. governance.rs; governance.rs carries the positive Authority axis (a stub governance.rs fails)
- clippy: `cargo clippy -p but-authz -p but-api -p but --all-targets`  -> Exit 0
- fmt: `cargo fmt --check`  -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: thin-CLI-shim over a but-api governance boundary that composes the AUTHZ-006 admin guard, then read-modify-writes the WORKING-TREE TOML preserving sugar/unrelated entries (VALUE-preserving via the raw wire structs + toml::to_string), inert until committed (seeding a [[principal]] on first grant, which authorizes once committed); perm-list reads BOTH the committed target-ref set and the working-tree file to compute the PENDING diff, and gates cross-principal reads on self-or-administration:read; governance.rs is the authorization-decision site and is folded into the honesty-grep ENFORCEMENT_PATHS
pattern_source: write guard = crates/but-api/src/legacy/config_mutate.rs:18-28 (compose, do not fork); working-tree write = crates/but-api/tests/admin_write_guard.rs:158-164 (repo.workdir()+fs::write); target-ref read = crates/but-authz/src/config.rs:24-59 + 276-301; read-modify-write over the raw wire structs (config.rs:400-429 made pub + #[derive(Serialize)], serialized via toml::to_string) NOT the normalized GovConfig (config.rs:303-371 shows the normalization that would destroy role sugar); ref_id before/after = crates/but-api/tests/commit_gate.rs:11,48 (the inert/no-commit assertion); honesty grep additive edit = crates/but-authz/tests/invariant_build_gates.rs:23-30,60-72; CLI shim = crates/but/src/lib.rs:448-487 + crates/but/src/command/config.rs:85,138; the TWO args/mod.rs edit sites = ~:1040 (enum variant) + ~:1272-1356 (pub mod); exhaustive help grouping = crates/but/src/command/help.rs:80-176 (add the single Perm arm); CLI test = crates/but/tests/but/command/confinement.rs:69-130 (+ its --as rejection)
anti_pattern: re-serializing a normalized GovConfig (loses role="write" sugar + rewrites every principal via the group-membership fold — AC-2 fails); committing/staging the production write (breaks inert-until-committed — AC-1 inert + ref_id assertion fails); authoring a second authorize(AdministrationWrite) in governance.rs instead of composing enforce_administration_write_gate; running the write BEFORE the admin gate (AC-5 byte-for-byte-unchanged fails); silently skipping an unparseable token instead of Erroring (AC-6 fails); performing an anonymous grant when BUT_AGENT_HANDLE is unset (AC-6 fails); no-op/erroring on a first-grant absent principal instead of SEEDING (AC-3 seeding half fails); dropping the whole entry on revoke / mutating on a not-held revoke (AC-4 fails); re-deriving the ".gitbutler/permissions.toml" literal instead of permissions_path(); treating the working tree as committed in perm_list (no PENDING marker — AC-8 fails); leaking <other>'s effective set in perm_list without the scope check, OR denying for an unknown/blanket reason rather than the scope decision (AC-7 must_not_observe + scope-pin); branching on role names anywhere in the authorization path (honesty grep + AUTHZ invariant — NOW enforced on governance.rs); weakening/removing an existing honesty-grep pattern or path (forbidden); defining an --as/identity-override flag on a Perm subcommand (S2); resolving HEAD instead of the workspace target ref in the shim (R6); burying perm_* in crates/but/ so Sprint 06a cannot reuse them
interaction_notes:
  - Target-ref resolution: the writer/reader take an explicit `target_ref: &str` (e.g. "refs/heads/main") — the CLI shim resolves it from the WORKSPACE TARGET (not HEAD; mirror how the commit gate resolves config_ref). For the but-api tests, pass "refs/heads/main" directly (the fixture commits governance at main), matching admin_write_guard.rs / commit_gate.rs. Keep the function signature `(&gix::Repository, target_ref: &str, ...)` so Sprint 06a Tauri commands pass the same.
  - Env handling: perm_grant/perm_revoke/perm_list resolve the caller via resolve_principal_from_env (BUT_AGENT_HANDLE) inside enforce_administration_write_gate / the scope predicate, so the but-api tests use temp_env::with_var("BUT_AGENT_HANDLE", Some(...), ...) under #[serial_test::serial]; the AC-6 unset-handle case uses temp_env::with_var("BUT_AGENT_HANDLE", None::<&str>, ...). temp-env IS declared in but-api dev-deps (Cargo.toml:158 `temp-env = { version="0.3", features=["async_closure"] }`) and serial_test at :157, so NO dependency addition is needed (FLAG only if a build surfaces otherwise).
  - First-grant registration (T-AUTHZ-007): if `--principal <id>` has no `[[principal]]` entry yet (or the config file is absent/empty), perm_grant APPENDS a new `[[principal]] id=<id> permissions=[<token>]` block (the "registering the entry on first grant" / day-one seeding behavior) — preserve all existing blocks. AC-3 then commits the seeded config via invoke_bash and asserts load_governance_config authorizes the seeded principal (the production writer never commits; only the test commits to prove effectiveness). perm_revoke removes the named tokens from the principal's list (leaving the entry, possibly empty); revoking a token the principal does not have is a no-op success (idempotent, byte-unchanged), NOT an error.
  - R4 decision (RESOLVED — option a): the wire structs derive `#[derive(Serialize)]` (additive, non-semantic) and the writer re-serializes via `toml::to_string` (toml="0.9.10" IS in workspace deps; toml_edit is NOT). The successful-write contract is VALUE-preserving (role="write" value survives, unrelated principals/groups survive with their values), NOT byte-verbatim — `toml::to_string` emits canonical TOML and MAY drop comments/blank lines and normalize ordering. No must_observe asserts byte-verbatim on a SUCCESSFUL write; the only byte-for-byte assertions are on UNCHANGED files (denied writes AC-5/AC-6, idempotent no-op AC-4) where nothing is written at all.
  - PENDING computation: perm_list reads the committed effective set (load_governance_config at the target ref) AND parses the working-tree file; a permission present in the working-tree principal entry but NOT in the committed effective set renders with a `PENDING` marker. A clean working tree (no uncommitted edit) shows no PENDING markers. Render via the OutputFormat the shim already has (Human text for the CLI; the but-api fn returns a structured value the test inspects + the shim formats).
  - perm-list scope decision (AC-7): the cross-principal denial is a SCOPE decision over a RESOLVED, KNOWN caller — resolve_principal_from_env succeeds, the caller is present in committed config, but lacks administration:read and target ≠ self. Pin this by asserting the same caller's self-read (None) returns its ACTUAL contents:write set (proving resolution + knownness), so the cross-principal Err cannot be a blanket unknown/error path.
  - Sprint 06a reuse: site perm_list/perm_grant/perm_revoke with signatures that the future Tauri `perm_list`/`perm_grant`/`perm_revoke` commands (api-design.md:84-86) wrap directly — return Result with a structured Ok payload + the Denial/GovWriteError on the err path, classifiable by config_mutate::classify_error (extend classify_error coverage in governance.rs if a new GovWriteError variant needs a code; prefer reusing perm.denied/config.invalid).
  - Help grouping scope: adding `Perm(perm::Platform)` generates `SubcommandDiscriminant::Perm`; `crates/but/src/command/help.rs` matches that discriminant exhaustively, so the Perm grouping arm is required for `cargo check -p but --all-targets`. This is NOT optional help-surface expansion: add only the single Perm arm (mirror Config under `Group::OtherCommands`) and do not touch unrelated help behavior.
  - delete/group are NOT in this task — `but group` is CLI-002; do not stub group verbs here.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — Rust work spanning a net-new TOML read-modify-write writer (preserving serde wire shape + role sugar via toml::to_string), a but-api governance boundary composing the AUTHZ-006 guard, a new clap noun + thin CLI shim, a first-grant seeding path proven day-one-effective via a committed-config read, a reconnaissance-scoping read predicate, fail-closed token-parse/caller-resolution, and an ADDITIVE honesty-grep coverage edge for governance.rs. gix working-tree writes, target-ref blob reads, ref_id before/after, structured Denial classification, and snapbox CLI tests are all rust-implementer competencies; rust-reviewer adversarially validates the write is inert (never committed; ref_id unchanged), the admin gate runs before any write, role sugar survives, the seeded principal authorizes once committed, revoke is idempotent, bad-token/unset-handle fail closed, no cross-principal leak, and the honesty grep covers governance.rs without weakening any existing pattern.
reviewer: rust-reviewer
coding_standards: crates/AGENTS.md, crates/but/AGENTS.md (CLI test economy — happy-path snapbox only), crates/WORKSPACE_MODEL.md, crates/but-authz + crates/but-api/src/legacy (nearby patterns: gix workdir write, target-ref blob read, ref_id before/after, Denial/AdminWriteGateError classify_error, but-testsupport writable_scenario + invoke_bash, temp_env + serial_test for BUT_AGENT_HANDLE)

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: AUTHZ-006 (the administration:write guard composed here), AUTHZ-001/002/003 (the but-authz primitive: Authority/AuthoritySet/load_governance_config/Denial), GRPS-001 (the union loader perm_list renders)
Blocks:     Sprint 06a (the perm_* but-api functions the MGMT Tauri commands reuse), CLI-002 (sequence CLI-001 FIRST — it establishes args/mod.rs `pub mod perm;` + the dispatch/metrics pattern, legacy/governance.rs + the writer, AND the governance.rs honesty-grep coverage; CLI-002 adds the parallel Group variant and its group_* code INSIDE the now-covered governance.rs)
SHARED-EDIT COORDINATION (CLI-001 + CLI-002 both touch these):
  - crates/but/src/args/mod.rs (Subcommands enum variant ~:1040 + the pub mod block ~:1272-1356), crates/but/src/lib.rs (dispatch arm), crates/but/src/utils/metrics.rs (metrics arm), crates/but/src/command/mod.rs (module decl), crates/but/src/command/help.rs (exhaustive SubcommandDiscriminant grouping arm), crates/but-api/src/legacy/governance.rs (CLI-001 owns the writer + perm_* + the file; CLI-002 appends group_* into the same, now honesty-grep-covered, file).
  - Sequence CLI-001 BEFORE CLI-002. CLI-001 adds ONLY the Perm variant/module/dispatch/metrics arm + the ENFORCEMENT_PATHS governance.rs coverage; CLI-002 adds ONLY the Group ones. Each touches DISTINCT lines (additive enum variants + match arms), so a clean merge is expected — but the orchestrator MUST land CLI-001 first and rebase CLI-002 on it (the merge-conflict surface is the shared files; an additive insert in the same region can conflict textually). Flag to the orchestrator: do NOT run CLI-001 and CLI-002 in parallel worktrees without a rebase gate.
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "CLI-001",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "perm_governance_base": {
      "description": "Real-git scenario via but_testsupport::writable_scenario(\"checkout-head-info\"). Target ref refs/heads/main carries a committed .gitbutler/permissions.toml where `admin` holds administration:write, `rust-implementer` holds [\"contents:write\"] (NO reviews:write, NO administration:write), `security-bot` is an unrelated principal, `maint` holds [\"merge\"], and a [[group]] code-reviewers (permissions=[\"reviews:write\"], members=[\"rust-reviewer\"]) exists; plus .gitbutler/gates.toml marking main protected. rust-implementer's entry uses the literal `role = \"write\"` sugar in the AC-2 variant so the read-modify-write can be proven to preserve it. The working tree starts clean (matching the committed blob). Seeded via a REAL entrypoint: invoke_bash writes the files and git-commits them at main (the same committed-config-at-main pattern as crates/but-api/tests/admin_write_guard.rs:96-123 and crates/but/tests/but/command/confinement.rs:69-130).",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"checkout-head-info\");",
        "invoke_bash on main: mkdir -p .gitbutler; write .gitbutler/permissions.toml with [[principal]] id=\"admin\" permissions=[\"administration:write\",\"merge\"]; [[principal]] id=\"rust-implementer\" role=\"write\" (role sugar, for AC-2) OR permissions=[\"contents:write\"] (for AC-1/AC-4/AC-5/AC-6/AC-7/AC-8 — use the variant the test needs); [[principal]] id=\"security-bot\" permissions=[\"statuses:write\"] (unrelated); [[principal]] id=\"maint\" permissions=[\"merge\"]; [[group]] name=\"code-reviewers\" permissions=[\"reviews:write\"] members=[\"rust-reviewer\"]; and .gitbutler/gates.toml [[branch]] name=\"main\" protected=true; then git add .gitbutler/permissions.toml .gitbutler/gates.toml && git commit -m \"governance config\".",
        "Capture the committed working-tree state: committed_blob_text(repo, but_authz::permissions_path()) and ref_id(repo, \"refs/heads/main\") BEFORE any write, for the AC-5/AC-6 byte-for-byte-unchanged assertion, the AC-4 idempotent no-op assertion, and the AC-1 inert (target-ref read + ref_id-unchanged) assertion.",
        "AC-1/AC-8 grant step: set BUT_AGENT_HANDLE=admin via temp_env::with_var under #[serial_test::serial], call perm_grant(&repo, \"refs/heads/main\", \"rust-implementer\", [\"reviews:write\"]); this writes ONLY the working-tree file (no commit) — AC-8 then lists against this post-state; AC-1 asserts ref_id(main) before==after.",
        "AC-4 revoke step: set BUT_AGENT_HANDLE=admin, call perm_revoke(&repo, \"refs/heads/main\", \"rust-implementer\", [\"contents:write\"]) (removes a held token); separately call perm_revoke(&repo, \"refs/heads/main\", \"rust-implementer\", [\"merge\"]) (not held — idempotent, byte-unchanged vs the captured working-tree state).",
        "AC-5 denial step: set BUT_AGENT_HANDLE=rust-implementer, call perm_grant(&repo, \"refs/heads/main\", \"rust-implementer\", [\"administration:write\"]) AND perm_revoke(&repo, \"refs/heads/main\", \"admin\", [\"administration:write\"]); expect Err perm.denied and the working-tree file unchanged from the captured committed state for both.",
        "AC-6 fail-closed step: set BUT_AGENT_HANDLE=admin, call perm_grant(&repo, \"refs/heads/main\", \"rust-implementer\", [\"badtoken\"]) (expect Err parse, file unchanged); set BUT_AGENT_HANDLE=None via temp_env::with_var(\"BUT_AGENT_HANDLE\", None::<&str>, ...), call perm_grant(&repo, \"refs/heads/main\", \"rust-implementer\", [\"reviews:write\"]) (expect Err perm.denied no_handle, file unchanged).",
        "AC-7 scope step: set BUT_AGENT_HANDLE=rust-implementer (a registered, resolved, known principal), call perm_list(&repo, \"refs/heads/main\", Some(\"maint\")) (expect Err perm.denied scope decision, no maint set leaked) and perm_list(&repo, \"refs/heads/main\", None) (expect Ok, ACTUAL contents:write set); separately set BUT_AGENT_HANDLE=admin and call perm_list(... Some(\"maint\")) for the admin-read positive (TC-13)."
      ]
    },
    "perm_governance_seed": {
      "description": "Real-git seeding variant via but_testsupport::writable_scenario(\"checkout-head-info\") for the T-AUTHZ-007 first-grant/day-one-effectiveness AC-3. Target ref refs/heads/main carries a committed .gitbutler/permissions.toml holding ONLY `admin` (administration:write) — there is NO [[principal]] entry for rust-implementer (the absent-principal/fresh-config start) — plus .gitbutler/gates.toml marking main protected. The working tree starts clean. Seeded via a REAL entrypoint: invoke_bash writes the files and git-commits them at main. AC-3 then performs an admin perm_grant (which must REGISTER a new [[principal]] block), invoke_bash-commits the resulting working-tree config to main, and asserts load_governance_config authorizes the seeded principal.",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"checkout-head-info\");",
        "invoke_bash on main: mkdir -p .gitbutler; write .gitbutler/permissions.toml with ONLY [[principal]] id=\"admin\" permissions=[\"administration:write\"] (NO rust-implementer entry); and .gitbutler/gates.toml [[branch]] name=\"main\" protected=true; then git add .gitbutler/permissions.toml .gitbutler/gates.toml && git commit -m \"governance config (admin only)\".",
        "Confirm the absent-principal start: load_governance_config(&repo, \"refs/heads/main\").principal_authorities(\"rust-implementer\") is empty BEFORE the grant (rust-implementer is not registered).",
        "AC-3 seeding grant: set BUT_AGENT_HANDLE=admin via temp_env::with_var under #[serial_test::serial], call perm_grant(&repo, \"refs/heads/main\", \"rust-implementer\", [\"reviews:write\"]); assert the working-tree .gitbutler/permissions.toml now has a NEW [[principal]] id=\"rust-implementer\" block carrying reviews:write AND the existing admin block is preserved.",
        "AC-3 day-one commit + authorize: invoke_bash on main: git add .gitbutler/permissions.toml && git commit -m \"seed rust-implementer\"; then load_governance_config(&repo, \"refs/heads/main\").principal_authorities(\"rust-implementer\") CONTAINS Authority::ReviewsWrite (the seeded principal authorizes day-one)."
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "description": "Admin perm_grant writes the new permission into the working-tree config while the target-ref effective set is unchanged (inert-until-committed pair), and prints the ref-pin caveat.",
      "verify": "cargo test -p but-api perm_grant_writes_worktree_inert_until_committed",
      "primary": true,
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api perm_grant + real but-authz load_governance_config + real gix repo (committed target-ref config vs working-tree write) via but_testsupport::writable_scenario",
        "negative_control": {
          "would_fail_if": [
            "the writer wrote nothing — the working-tree file would lack reviews:write under rust-implementer",
            "the writer COMMITTED the grant — the target-ref effective set would then contain ReviewsWrite, breaking the inert (must_not_observe) assertion, and ref_id(main) would change",
            "a stub returned Ok without writing — guarded by asserting the working-tree file literally contains reviews:write",
            "the ref-pin caveat string were absent (silent success without warning the operator)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "perm_governance_base",
            "action": {
              "actor": "ci",
              "steps": [
                "capture ref_id(&repo, \"refs/heads/main\") BEFORE the write",
                "temp_env::with_var(\"BUT_AGENT_HANDLE\", Some(\"admin\"), ...) under #[serial_test::serial]",
                "perm_grant(&repo, \"refs/heads/main\", \"rust-implementer\", [\"reviews:write\"])",
                "read the working-tree .gitbutler/permissions.toml back",
                "load_governance_config(&repo, \"refs/heads/main\").principal_authorities(\"rust-implementer\")",
                "capture ref_id(&repo, \"refs/heads/main\") AFTER the write"
              ]
            },
            "end_state": {
              "must_observe": [
                "`perm_grant` returns `Ok`",
                "the working-tree `.gitbutler/permissions.toml` contains `reviews:write` under `rust-implementer`",
                "the result/printed caveat contains `\"takes effect once committed to the target branch\"`",
                "the committed target-ref effective set for `rust-implementer` still contains `contents:write`",
                "`ref_id(refs/heads/main)` AFTER the write == the `ref_id` captured BEFORE the write"
              ],
              "must_not_observe": [
                "the target-ref effective set for `rust-implementer` containing `Authority::ReviewsWrite` (grant must be inert until committed)",
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
      "description": "The writer preserves unrelated principal/group entries and role-preset sugar (read-modify-write, value-preserving, not a normalized round-trip).",
      "verify": "cargo test -p but-api perm_grant_preserves_unrelated_entries_and_role_sugar",
      "primary": false,
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api perm_grant + real gix working-tree read-back; assertions on the parsed wire shape + raw TOML text (VALUE-preserving), not a normalized GovConfig",
        "negative_control": {
          "would_fail_if": [
            "the writer re-serialized a normalized GovConfig — `role = \"write\"` would be desugared to a flat permissions list and gone",
            "the unrelated security-bot or code-reviewers block were dropped by a normalized rewrite (its value absent)",
            "the group-membership fold rewrote principals (the normalization at config.rs:303-371) — unrelated entries would mutate",
            "a no-op stub writer that leaves the file unchanged (writes nothing) is caught — the new `merge` entry would be absent"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "perm_governance_base",
            "action": {
              "actor": "ci",
              "steps": [
                "temp_env BUT_AGENT_HANDLE=admin under #[serial_test::serial]",
                "perm_grant(&repo, \"refs/heads/main\", \"rust-implementer\", [\"merge\"]) where rust-implementer carries role=\"write\"",
                "read the rewritten working-tree .gitbutler/permissions.toml as raw text + parse it into the wire structs"
              ]
            },
            "end_state": {
              "must_observe": [
                "the rewritten file still carries the `role = \"write\"` VALUE for `rust-implementer` (a role assignment, not an expanded flat list)",
                "the rewritten file still contains the unrelated `security-bot` principal with its `statuses:write` value",
                "the rewritten file still contains the `[[group]]` `code-reviewers` block with its `reviews:write` grant",
                "`merge` is now present in `rust-implementer`'s permissions"
              ],
              "must_not_observe": [
                "the `role = \"write\"` value replaced by an expanded flat permissions list (sugar lost to normalization)",
                "the `security-bot` or `code-reviewers` block missing from the rewritten file",
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
      "description": "The first grant against an absent/empty config SEEDS a new [[principal]] block and, once committed, AUTHORIZES the seeded principal for the token (T-AUTHZ-007 day-one effectiveness).",
      "verify": "cargo test -p but-api perm_first_grant_seeds_principal_and_authorizes_when_committed",
      "primary": false,
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api perm_grant (registration) + real invoke_bash commit + real but-authz load_governance_config (day-one authorize) + real gix via but_testsupport::writable_scenario",
        "negative_control": {
          "would_fail_if": [
            "the writer errors or no-ops on an absent principal — no `[[principal]] id=\"rust-implementer\"` block appears (seeding half fails)",
            "the writer registers the block but the seeded config does NOT authorize once committed — principal_authorities(\"rust-implementer\") lacks ReviewsWrite (day-one half fails; the commit step is what makes it effective)",
            "the writer drops the pre-existing admin block — admin's administration:write would be absent (the seeding must preserve existing blocks)",
            "the writer registers an empty/placeholder block with no permissions — reviews:write would be absent under rust-implementer"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "perm_governance_seed",
            "action": {
              "actor": "ci",
              "steps": [
                "confirm load_governance_config(&repo, \"refs/heads/main\").principal_authorities(\"rust-implementer\") is empty BEFORE the grant (absent principal)",
                "temp_env::with_var(\"BUT_AGENT_HANDLE\", Some(\"admin\"), ...) under #[serial_test::serial]",
                "perm_grant(&repo, \"refs/heads/main\", \"rust-implementer\", [\"reviews:write\"]) (registers a new principal)",
                "read the working-tree .gitbutler/permissions.toml back and assert the new block + preserved admin block",
                "invoke_bash on main: git add .gitbutler/permissions.toml && git commit -m \"seed rust-implementer\"",
                "load_governance_config(&repo, \"refs/heads/main\").principal_authorities(\"rust-implementer\")"
              ]
            },
            "end_state": {
              "must_observe": [
                "the working-tree file gains a NEW `[[principal]] id=\"rust-implementer\"` block carrying `reviews:write`",
                "the pre-existing `admin` principal with `administration:write` is still present in the working-tree file",
                "after the commit, `load_governance_config(...).principal_authorities(\"rust-implementer\")` CONTAINS `Authority::ReviewsWrite`",
                "`perm_grant` returns `Ok` and the caveat contains `\"takes effect once committed to the target branch\"`"
              ],
              "must_not_observe": [
                "the seeded `rust-implementer` block being absent / 0 new principal entries after the grant (a no-op/erroring writer leaves none)",
                "an empty `principal_authorities(\"rust-implementer\")` after the seeded config is committed (no day-one effectiveness)",
                "the pre-existing `admin` block missing from the rewritten file"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "description": "Admin perm_revoke removes the named token (preserving unrelated tokens/entries) and is an idempotent no-op (byte-unchanged) when the token is absent.",
      "verify": "cargo test -p but-api perm_revoke_removes_token_and_idempotent_noop",
      "primary": false,
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api perm_revoke + real gix working-tree read-back via but_testsupport::writable_scenario",
        "negative_control": {
          "would_fail_if": [
            "a stub returns Ok without removing — `contents:write` would still be present under `rust-implementer` after the first revoke",
            "the writer drops the whole `rust-implementer` entry instead of just the token (the entry must survive, possibly empty)",
            "the writer drops the unrelated `security-bot` principal or the `[[group]] code-reviewers` block (unrelated entries must persist)",
            "the not-held revoke ERRORS or MUTATES the file — the idempotent no-op must return Ok and leave the file byte-unchanged"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "perm_governance_base",
            "action": {
              "actor": "ci",
              "steps": [
                "capture the working-tree .gitbutler/permissions.toml bytes before the not-held revoke",
                "temp_env BUT_AGENT_HANDLE=admin under #[serial_test::serial]",
                "perm_revoke(&repo, \"refs/heads/main\", \"rust-implementer\", [\"contents:write\"]) (removes a held token)",
                "read the rewritten working-tree file back",
                "perm_revoke(&repo, \"refs/heads/main\", \"rust-implementer\", [\"merge\"]) (not held — idempotent path)",
                "re-read the working-tree file bytes and compare to the pre-call capture"
              ]
            },
            "end_state": {
              "must_observe": [
                "after the first revoke, the working-tree file no longer lists `contents:write` under `rust-implementer`",
                "the `rust-implementer` `[[principal]]` entry SURVIVES after the revoke (the token was removed, not the entry)",
                "the unrelated `security-bot` principal and the `[[group]] code-reviewers` block are still present",
                "the not-held `perm_revoke([\"merge\"])` returns `Ok` and the working-tree file is byte-for-byte identical to its pre-call capture",
                "the returned/printed caveat contains `\"takes effect once committed to the target branch\"`"
              ],
              "must_not_observe": [
                "`contents:write` still present under `rust-implementer` after the first revoke (a stub that removes nothing)",
                "the `rust-implementer` entry or the unrelated `security-bot`/`code-reviewers` blocks dropped (0 surviving unrelated entries)",
                "the working-tree file bytes changing across the not-held idempotent revoke"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "description": "A non-admin perm_grant/perm_revoke is denied perm.denied (names administration:write) and writes nothing.",
      "verify": "cargo test -p but-api perm_grant_revoke_non_admin_denied",
      "primary": false,
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api perm_grant/perm_revoke composing enforce_administration_write_gate + real but-authz + real gix",
        "negative_control": {
          "would_fail_if": [
            "a stub always wrote / never gated — the working-tree file would change (guarded by the byte-for-byte-unchanged assertion)",
            "the writer ran BEFORE the admin gate — the file would change despite the denial (ordering bug)",
            "the denial were not classified perm.denied or did not name administration:write",
            "the revoke path skipped the gate that the grant path runs — the non-admin revoke would mutate the file"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "perm_governance_base",
            "action": {
              "actor": "ci",
              "steps": [
                "capture the working-tree .gitbutler/permissions.toml bytes before the calls",
                "temp_env BUT_AGENT_HANDLE=rust-implementer under #[serial_test::serial]",
                "perm_grant(&repo, \"refs/heads/main\", \"rust-implementer\", [\"administration:write\"]) (self-grant attempt)",
                "perm_revoke(&repo, \"refs/heads/main\", \"admin\", [\"administration:write\"]) (non-admin revoke attempt)",
                "config_mutate::classify_error on each returned error",
                "re-read the working-tree file bytes"
              ]
            },
            "end_state": {
              "must_observe": [
                "`perm_grant` returns `Err` and `perm_revoke` returns `Err`",
                "`config_mutate::classify_error(&err).code == \"perm.denied\"` for both",
                "the denial messages contain `\"administration:write\"`",
                "the working-tree `.gitbutler/permissions.toml` bytes are identical to the pre-call capture after BOTH calls"
              ],
              "must_not_observe": [
                "the working-tree file containing `administration:write` under `rust-implementer` after the denied grant",
                "the `admin` principal's `administration:write` removed after the denied revoke",
                "any change to the working-tree file bytes (a do-nothing-on-denial path leaves the start state, which this requires)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-6",
      "type": "acceptance_criterion",
      "description": "perm_grant with an unparseable Authority token OR with BUT_AGENT_HANDLE unset fails closed (Err / non-zero) and writes nothing.",
      "verify": "cargo test -p but-api perm_grant_fail_closed_bad_token_and_unset_handle",
      "primary": false,
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api perm_grant token parse (Authority::parse) + real resolve_principal_from_env + real gix working-tree read-back",
        "negative_control": {
          "would_fail_if": [
            "an impl silently skips an unparseable token (returns Ok writing nothing or a partial set) — the bad-token call MUST Err and the file MUST be unchanged",
            "an impl performs an anonymous grant when BUT_AGENT_HANDLE is unset — the unset-handle call MUST Err perm.denied and write nothing",
            "a fail-OPEN impl writes before parsing/resolving — the byte-for-byte-unchanged assertion catches it",
            "the unset-handle denial is not classified perm.denied (a wrong/blank code)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "perm_governance_base",
            "action": {
              "actor": "ci",
              "steps": [
                "capture the working-tree .gitbutler/permissions.toml bytes before each call",
                "temp_env BUT_AGENT_HANDLE=admin under #[serial_test::serial]; perm_grant(&repo, \"refs/heads/main\", \"rust-implementer\", [\"badtoken\"]) (unparseable Authority token)",
                "re-read the working-tree bytes",
                "temp_env::with_var(\"BUT_AGENT_HANDLE\", None::<&str>, ...); perm_grant(&repo, \"refs/heads/main\", \"rust-implementer\", [\"reviews:write\"]) (no resolvable caller)",
                "config_mutate::classify_error on the unset-handle error; re-read the working-tree bytes"
              ]
            },
            "end_state": {
              "must_observe": [
                "the `\"badtoken\"` grant returns `Err` (a parse error surfaced, NOT a silent skip)",
                "the working-tree `.gitbutler/permissions.toml` bytes are identical to the pre-call capture after the bad-token call",
                "the unset-handle grant returns `Err` with code `\"perm.denied\"` (Denial::no_handle — no anonymous action)",
                "the working-tree `.gitbutler/permissions.toml` bytes are identical to the pre-call capture after the unset-handle call"
              ],
              "must_not_observe": [
                "the working-tree file gaining `badtoken` (or any partial mutation) after the bad-token call — 0 new tokens written",
                "the working-tree file gaining `reviews:write` after the unset-handle call (an anonymous grant) — no new entry seeded",
                "any change to the working-tree file bytes across either fail-closed call (the file is byte-for-byte unchanged, as if nothing happened)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-7",
      "type": "acceptance_criterion",
      "description": "perm_list --principal <other> by a resolved-but-non-admin caller is denied the SCOPE decision perm.denied (no leak); self-read returns the caller's ACTUAL set.",
      "verify": "cargo test -p but-api perm_list_cross_principal_scoping",
      "primary": false,
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api perm_list scope predicate (resolved-known caller) + real but-authz load_governance_config + real gix",
        "negative_control": {
          "would_fail_if": [
            "the scope predicate were absent — maint's effective set would be returned to a non-admin caller (recon leak)",
            "the cross-principal denial fired for the WRONG reason (an unknown/blanket-error path rather than the scope decision) — caught because the same caller's self-read returns its ACTUAL contents:write set, proving the caller resolves and is known",
            "the self-read returned an empty/blank set instead of the caller's ACTUAL contents:write (a blanket-deny or unknown-principal impl)",
            "the predicate ignored administration:read — TC-13's admin-read positive would fail"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "perm_governance_base",
            "action": {
              "actor": "ci",
              "steps": [
                "temp_env BUT_AGENT_HANDLE=rust-implementer (a registered, resolved, known principal) under #[serial_test::serial]",
                "perm_list(&repo, \"refs/heads/main\", Some(\"maint\")) (cross-principal, expect Err scope decision)",
                "perm_list(&repo, \"refs/heads/main\", None) (self, expect Ok with the ACTUAL contents:write set)",
                "inspect the cross-principal returned/rendered output for any maint authority leak"
              ]
            },
            "end_state": {
              "must_observe": [
                "the cross-principal `perm_list(Some(\"maint\"))` returns `Err` with code `\"perm.denied\"` as the SCOPE decision (caller resolved + known, lacks administration:read, target != self)",
                "the self `perm_list(None)` returns `Ok` and shows `rust-implementer`'s ACTUAL `contents:write` authority (the caller's real set, proving resolution + knownness)"
              ],
              "must_not_observe": [
                "`maint`'s effective set (`merge`) appearing in the cross-principal call's output",
                "any enumeration of `maint`'s authorities to the non-admin caller",
                "the self-read returning an empty/blank/no authorities set for `rust-implementer` (which would mean the denial was an unknown/blanket-error, not a scope decision)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-8",
      "type": "acceptance_criterion",
      "description": "perm_list prints the committed effective set PLUS a literal PENDING marker for an uncommitted working-tree grant.",
      "verify": "cargo test -p but-api perm_list_pending_marks_uncommitted_grant",
      "primary": false,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api perm_list reading both the target-ref committed config and the working-tree file + real gix",
        "negative_control": {
          "would_fail_if": [
            "list treated the working tree as committed — reviews:write would show without a PENDING marker",
            "list omitted the pending grant entirely (read only the target ref)",
            "the PENDING marker were static rather than computed from the committed-vs-working diff (contents:write would be wrongly marked)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "perm_governance_base",
            "action": {
              "actor": "ci",
              "steps": [
                "temp_env BUT_AGENT_HANDLE=admin under #[serial_test::serial]",
                "perm_grant(&repo, \"refs/heads/main\", \"rust-implementer\", [\"reviews:write\"]) (writes working tree only)",
                "perm_list(&repo, \"refs/heads/main\", Some(\"rust-implementer\"))",
                "inspect the rendered output for committed vs PENDING markers"
              ]
            },
            "end_state": {
              "must_observe": [
                "the output shows the committed `contents:write` (read at the target ref)",
                "the output marks the uncommitted `reviews:write` grant with a literal `PENDING` marker",
                "the committed `contents:write` is NOT marked `PENDING`"
              ],
              "must_not_observe": [
                "`reviews:write` shown as already-effective (no PENDING marker)",
                "`reviews:write` omitted from the listing entirely"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "after admin perm_grant, the working-tree .gitbutler/permissions.toml contains reviews:write under rust-implementer",
      "verify": "cargo test -p but-api perm_grant_writes_worktree_inert_until_committed",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "load_governance_config(repo, refs/heads/main).principal_authorities(rust-implementer) does NOT contain Authority::ReviewsWrite after the grant, AND ref_id(repo, refs/heads/main) is identical before and after the write (target ref unchanged / inert / no commit)",
      "verify": "cargo test -p but-api perm_grant_writes_worktree_inert_until_committed",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "the perm_grant result/printed caveat contains 'takes effect once committed to the target branch'",
      "verify": "cargo test -p but-api perm_grant_writes_worktree_inert_until_committed",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "the rewritten file still carries the role = \"write\" VALUE for rust-implementer and the unrelated security-bot + code-reviewers entries (role sugar + unrelated entries value-preserved)",
      "verify": "cargo test -p but-api perm_grant_preserves_unrelated_entries_and_role_sugar",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "admin perm_grant against a config with NO rust-implementer entry registers a new [[principal]] id=\"rust-implementer\" block carrying reviews:write, preserving the existing admin block (seeding half)",
      "verify": "cargo test -p but-api perm_first_grant_seeds_principal_and_authorizes_when_committed",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "description": "after committing the seeded config to refs/heads/main, load_governance_config(...).principal_authorities(rust-implementer) CONTAINS Authority::ReviewsWrite (day-one effectiveness — T-AUTHZ-007)",
      "verify": "cargo test -p but-api perm_first_grant_seeds_principal_and_authorizes_when_committed",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-7",
      "type": "test_criterion",
      "description": "after admin perm_revoke contents:write, the working-tree file no longer lists contents:write under rust-implementer, the rust-implementer entry + unrelated security-bot/code-reviewers survive",
      "verify": "cargo test -p but-api perm_revoke_removes_token_and_idempotent_noop",
      "maps_to_ac": "AC-4"
    },
    {
      "id": "TC-8",
      "type": "test_criterion",
      "description": "admin perm_revoke of a token the principal does NOT hold (merge) returns Ok and leaves the working-tree file byte-for-byte unchanged (idempotent no-op)",
      "verify": "cargo test -p but-api perm_revoke_removes_token_and_idempotent_noop",
      "maps_to_ac": "AC-4"
    },
    {
      "id": "TC-9",
      "type": "test_criterion",
      "description": "non-admin perm_grant AND non-admin perm_revoke each return Err; config_mutate::classify_error(&err).code == perm.denied and message contains administration:write",
      "verify": "cargo test -p but-api perm_grant_revoke_non_admin_denied",
      "maps_to_ac": "AC-5"
    },
    {
      "id": "TC-10",
      "type": "test_criterion",
      "description": "the working-tree .gitbutler/permissions.toml is byte-for-byte unchanged after the denied non-admin grant AND revoke",
      "verify": "cargo test -p but-api perm_grant_revoke_non_admin_denied",
      "maps_to_ac": "AC-5"
    },
    {
      "id": "TC-11",
      "type": "test_criterion",
      "description": "perm_grant with token badtoken returns Err (parse error, not a silent skip) and leaves the working-tree file byte-for-byte unchanged",
      "verify": "cargo test -p but-api perm_grant_fail_closed_bad_token_and_unset_handle",
      "maps_to_ac": "AC-6"
    },
    {
      "id": "TC-12",
      "type": "test_criterion",
      "description": "perm_grant with BUT_AGENT_HANDLE unset returns Err code perm.denied (no anonymous action) and leaves the working-tree file byte-for-byte unchanged",
      "verify": "cargo test -p but-api perm_grant_fail_closed_bad_token_and_unset_handle",
      "maps_to_ac": "AC-6"
    },
    {
      "id": "TC-13",
      "type": "test_criterion",
      "description": "perm_list(None) self-read by the resolved-known non-admin rust-implementer returns Ok showing its ACTUAL contents:write set; perm_list(Some(maint)) by the same caller is Err perm.denied (scope decision) with no leak of maint's merge; an administration:read holder listing maint returns Ok",
      "verify": "cargo test -p but-api perm_list_cross_principal_scoping",
      "maps_to_ac": "AC-7"
    },
    {
      "id": "TC-14",
      "type": "test_criterion",
      "description": "perm_list shows committed contents:write NOT marked PENDING and uncommitted reviews:write marked PENDING",
      "verify": "cargo test -p but-api perm_list_pending_marks_uncommitted_grant",
      "maps_to_ac": "AC-8"
    },
    {
      "id": "TC-15",
      "type": "test_criterion",
      "description": "but perm grant --principal rust-implementer reviews:write as admin exits 0 with the ref-pin caveat on stdout; but perm revoke --principal rust-implementer contents:write as admin exits 0 with the caveat; the grant as a non-admin exits 1 with perm.denied on stderr",
      "verify": "cargo test -p but perm",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-16",
      "type": "test_criterion",
      "description": "the admin but perm grant CLI invocation does NOT move refs/heads/main (ref_id before == after across the subprocess; the working-tree file is an uncommitted edit at most)",
      "verify": "cargo test -p but perm",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-17",
      "type": "test_criterion",
      "description": "but perm grant --as admin --principal rust-implementer reviews:write exits non-zero with a clap unknown/unsupported-flag error (no governed action performed; BUT_AGENT_HANDLE is the only identity source); no --as flag is defined on any Perm subcommand",
      "verify": "cargo test -p but perm",
      "maps_to_ac": "AC-5"
    }
  ]
}
-->
</details>
</output>
</content>
</invoke>
