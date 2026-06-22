# LPR-013: `principal_kind_read`/`principal_kind_update` governed-config but-api producer (the `permissions.toml` `kind` reader+writer) + its Tauri command/SDK delta

> Status: ✅ Completed
> Commit: 1819fd554c
> Reviewer: rust-reviewer (DEFERRED — all 4 but-api ACs + 2 mgmt_ipc AC-5 tests pass at HEAD; SDK regen deferred to LPR-010 (missing InternalJsonSchema impls on LocalReviewAssignment/Comment in forge.rs))
> Updated: 2026-06-22T17:39:48Z


## What this does

Expose the committed-config principal `kind` descriptor (`agent` | `human`, the additive `#[serde(default)] pub kind: Option<String>` field LPR-005 adds to `PrincipalWire`) to the SvelteKit desktop as a **read + write** governed-config producer, so the Principals editor (LPR-014) can SHOW and SET whether a principal is an agent or a human. Two net-additive `but-api` fns in `crates/but-api/src/legacy/governance.rs` modeled **exactly** on the shipped `perm_grant`/`branch_gates_update` governance-config pattern: `principal_kind_read(ctx, target_ref)` (a renderer contract listing each principal's declared `kind`, read at the **target ref**) and `principal_kind_update(ctx, target_ref, principal, kind)` (an `administration:write`-gated, inert-until-committed working-tree write of `.gitbutler/permissions.toml` that LOSSLESSLY round-trips the full `[[principal]]`/`[[group]]` schema and sets only the targeted principal's `kind`). The write composes the EXISTING `enforce_administration_write_gate` (never a second admin check), writes the WORKING TREE only (inert until committed through the existing `governance_commit` pending→commit path — `permissions.toml` is already a `GOVERNANCE_COMMIT_PATHS` member), and the desktop `#[tauri::command]` write wrapper resolves the human fleet-owner identity via the Sprint-06a `DesktopSessionState` shim (`BUT_AGENT_HANDLE` is unset in the desktop process). The read wrapper rides `core:default` like the other `#[but_api(napi)]` reads; the regenerated `@gitbutler/but-sdk` type-checks in the desktop. **This is governed config — `administration:write`-gated and ref-pinned — UNLIKE `keep_reviews_local` (LPR-006), which is a trusted-desktop project preference.** The `kind` field changes NO enforcement: it does not enter `GovConfig.principals` and no gate reads it (LPR-005's invariant).

## Why

Sprint 07 · PRD UC-LPR-04 · capability CAP-AUTHZ-01, CAP-CONFIG-01. LPR-005 establishes the `kind` field as the **source-of-truth for the agent-PR tag** (the agent-vs-human distinction does not exist in the resolved `Principal` — `resolve_principal` keys solely on `BUT_AGENT_HANDLE`, `principal.rs:82` has no kind discriminator — so the tag MUST come from declared config). But LPR-005 only READS `kind` for tag derivation; a human operator still needs a desktop surface to **declare** which principals are agents in the first place. This producer is that surface: it exposes the governed `kind` read+write to the Principals editor (LPR-014) through the SAME admin-gated, ref-pinned, inert-until-committed config path the Sprint-06a `perm_grant`/`branch_gates_update` editors use — so setting a principal's `kind` is a governed config edit (pending → fleet-owner commit), not a local toggle. Because `kind` is read at the target ref (anti-self-escalation, `config.rs:23`–`:25`), a principal cannot flip its own working-tree kind to forge an attestation; and because the write is `administration:write`-gated, only an admin can declare kinds. The producer NEVER touches the merge gate (the safe seam is intact — the gate reads only `local_review_verdicts`); `kind` is an enforcement-neutral descriptor on both the read and write paths.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-api principal_kind_update_writes_worktree_inert_until_committed`: an admin `principal_kind_update(&repo, target_ref, "agent-A", "agent")` writes `kind = "agent"` into the WORKING-TREE `.gitbutler/permissions.toml` for `agent-A` while `principal_kind_read` still reports the COMMITTED (target-ref) `kind` (inert-until-committed pair); the full `[[principal]]`/`[[group]]` schema (every grant, group, member, and unrelated principal's `kind`) round-trips losslessly. Full gate set in the spec below.

## Scope

  - crates/but-api/src/legacy/governance.rs (MODIFY — add `principal_kind_read`/`principal_kind_update` `#[but_api(napi)]` fns + the private `permissions.toml` read-modify-write `kind` setter that LOSSLESSLY round-trips the FULL `[[principal]]`+`[[group]]` schema via the raw wire structs CLI-001 already owns (or an additive `kind` field on them) + the `PrincipalKindList`/`PrincipalKindOutcome` `schemars::JsonSchema` DTOs; sited BESIDE the Sprint-06a `perm_*`/`branch_gates_*` fns; compose `enforce_administration_write_gate` + `classify_error`)
  - crates/but-api/tests/principal_kind.rs (NEW — the PRIMARY but-api proofs AC-1..AC-5 against a real but-authz + gix fixture via but_testsupport, hand-assertion style like the admin_write_guard / branch_gates tests)
  - crates/gitbutler-tauri/src/governance.rs (MODIFY — add `principal_kind_update_for_desktop_session` + the `tauri_principal_kind_update::principal_kind_update` `#[tauri::command]` wrapper resolving the fleet-owner via `DesktopSessionState`, mirroring `tauri_branch_gates_update`; the read `principal_kind_read` rides the but-api `#[but_api(napi)]`-generated `tauri_principal_kind_read::principal_kind_read` module like `tauri_perm_list`)
  - crates/gitbutler-tauri/src/lib.rs (MODIFY — register `tauri_principal_kind_read::principal_kind_read` (from but-api) + `$crate::governance::tauri_principal_kind_update::principal_kind_update` in the `gitbutler_governance_command_rows!` macro beside the `perm_*`/`branch_gates_*` rows)
  - crates/gitbutler-tauri/tests/mgmt_ipc_003_governance_commands.rs (MODIFY — add `principal_kind_read`/`principal_kind_update` to the `GOVERNANCE_COMMANDS` const + an `InvocationCase` for each, so the real Tauri mock-runtime bus proves both register and invoke; the existing forbidden-`allow-governance_*`-file assertion already covers the capability surface)
  - packages/but-sdk/src/generated/** (REGENERATE ONLY via `pnpm build:sdk && pnpm format` — NEVER hand-edit)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: LPR-013 — principal_kind_read/principal_kind_update governed-config but-api producer (the permissions.toml kind reader+writer) + its Tauri command/SDK delta
================================================================================

TASK_TYPE:   FEATURE
STATUS:      Backlog
PRIORITY:    P1
EFFORT:      L  (180 min)
AGENT:       implementer=tauri-implementer | reviewer=tauri-reviewer
PROPOSED-BY: tauri-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-LPR-04
CAPABILITIES:CAP-AUTHZ-01, CAP-CONFIG-01

PLATFORMS:   desktop   (the SvelteKit desktop GUI consumes the Tauri command + SDK; the but-api fns also serve the CLI/N-API, but the producer surface this task ships is the DESKTOP bus + the regenerated TS SDK. NO mobile target — GitButler desktop is Tauri-on-desktop only.)

RUNTIME_COMMANDS:
  test:  cargo test -p but-api principal_kind_update_writes_worktree_inert_until_committed
  check: cargo check -p but-api --all-targets && cargo check -p gitbutler-tauri --all-targets
  lint:  cargo clippy -p but-api -p gitbutler-tauri --all-targets

--------------------------------------------------------------------------------
TAURI IPC CONTRACT (the producer surface this task ships)
--------------------------------------------------------------------------------
COMMAND (read):  `principal_kind_read` (but-api #[but_api(napi)] → auto-generated tauri_principal_kind_read::principal_kind_read; invoke key `principal_kind_read`)
  Signature:     `fn principal_kind_read(ctx: &Context, target_ref: String) -> anyhow::Result<PrincipalKindList>`  (a self-/branch-scoped READ, like governance_principals_list — NO write authority, reads committed kinds at the target ref)
  Frontend:      invoke<PrincipalKindList>('principal_kind_read', { projectId, targetRef }): Promise<PrincipalKindList>
COMMAND (write): `principal_kind_update` (desktop fleet-owner wrapper in gitbutler-tauri governance.rs; invoke key `principal_kind_update`)
  Signature:     `fn principal_kind_update(desktop_session: tauri::State<'_, DesktopSessionState>, project_id: ProjectHandleOrLegacyProjectId, target_ref: String, principal: String, kind: String) -> Result<PrincipalKindOutcome, json::Error>`
  but-api fn:    `principal_kind_update(ctx, target_ref, principal, kind)` keeps env-principal resolution (resolve_principal_from_env via BUT_AGENT_HANDLE) for the CLI/tests; the desktop wrapper maps the fleet-owner identity onto that principal axis (BUT_AGENT_HANDLE is unset on desktop)
  Frontend:      invoke<PrincipalKindOutcome>('principal_kind_update', { projectId, targetRef, principal, kind }): Promise<PrincipalKindOutcome>
PERMISSION (capability + permission delta — the atomic command+permission rule):
  - BOTH commands are admitted by `core:default` in capabilities/main.json — GitButler app commands ride core:default; there is NO hand-written allow-governance_* / allow-principal_kind_* capability file (the IPC test mgmt_ipc_003_governance_commands.rs:240-253 ASSERTS no such file exists — adding one would FAIL the suite). The Tauri-v2 per-command `allow-principal_kind_read`/`allow-principal_kind_update` permission is AUTOGENERATED by the #[tauri::command] / #[but_api(napi)] macro into gen/schemas/, never authored by hand.
  - The "permission entry" that ships WITH each command (so neither slips to a later task) is: registration in the `gitbutler_governance_command_rows!` macro (lib.rs:204-223) AND the `GOVERNANCE_COMMANDS` const + an `InvocationCase` in mgmt_ipc_003_governance_commands.rs. An unregistered command is rejected by the real bus ("not found", proven by mgmt_unregistered_governance_command_not_invokable) — registration IS the admission.
CAPABILITY: capabilities/main.json is UNCHANGED (no new entry; core:default admits both). The capability assertion is the NEGATIVE one: no allow-governance_*/allow-principal_kind_* file is introduced.

--------------------------------------------------------------------------------
TYPE DESIGN (Rust shapes this task introduces)
--------------------------------------------------------------------------------
API SURFACE (additive #[but_api(napi)] fns in crates/but-api/src/legacy/governance.rs, modeled on perm_list (read, governance.rs:437) + branch_gates_update (write, governance.rs)):
  - `#[but_api(napi)] pub fn principal_kind_read(ctx: &Context, target_ref: String) -> anyhow::Result<PrincipalKindList>` — a read; resolves the target ref via target_ref_from_ctx, loads committed permissions.toml at the target ref, returns each principal's declared kind. NO write authority (like governance_principals_list). A caller that can't resolve / no governance config ⇒ empty list (read-only), not an error.
  - `#[but_api(napi)] pub fn principal_kind_update(ctx: &Context, target_ref: String, principal: String, kind: String) -> anyhow::Result<PrincipalKindOutcome>` — composes enforce_administration_write_gate(&repo, target_ref) BEFORE any write; value-preserving read-modify-write of the WORKING-TREE permissions.toml setting ONLY the targeted principal's kind; returns the ref-pin caveat + the post-write kind list. Plus `principal_kind_update_with_repo`/`principal_kind_update_with_repo_as_fleet_owner` mirroring perm_grant_with_repo/_as_fleet_owner (governance.rs:1533) for the desktop wrapper.
DTOs (schemars::JsonSchema, camelCase — mirror GovernancePrincipalsList governance.rs:192 / BranchGatesOutcome governance.rs:175):
  - `#[derive(Serialize, schemars::JsonSchema)] #[serde(rename_all = "camelCase")] pub struct PrincipalKindList { pub principals: Vec<PrincipalKindEntry> }`
  - `pub struct PrincipalKindEntry { pub principal_id: String, pub kind: Option<String>, /* committed declared kind at target ref; None = human default */ pub pending: bool /* working-tree kind differs from committed */ }`
  - `pub struct PrincipalKindOutcome { pub principals: Vec<PrincipalKindEntry>, pub caveat: String /* "takes effect once committed to the target branch" */ }`
KIND VALIDATION (typed at the boundary):
  - `kind` accepts only "agent" | "human" (mirror the AssignmentState/Authority parse/name round-trip, LPR-002). An unknown kind string is rejected with a structured config.invalid error (classify_error code), never written. Setting kind="human" may either write kind="human" explicitly OR clear the field to None (the conservative default-human posture) — pick one and assert it; do NOT leave both behaviors ambiguous.
OWNERSHIP PLAN:
  - The writer BORROWS &repo for the admin gate (target-ref blob read) + resolves repo.workdir() for the working-tree write. The raw permissions.toml wire structs (CLI-001's owned PrincipalWire/GroupWire round-trip, or an additive `kind: Option<String>` on them — coordinate with LPR-005 which adds the loader-side field) are parsed, the targeted principal's kind is mutated, and the whole structure is re-serialized via toml::to_string (canonical, VALUE-preserving). NO ref/object/oplog write; NO git add/commit (the inert contract).
ERROR STRATEGY:
  - anyhow::Result at the but-api boundary; classify_error (config_mutate.rs) yields the structured Denial. The desktop wrapper maps via json::Error::from (the governance.rs:87 convention). The frontend must handle: perm.denied (non-admin write — names administration:write), config.invalid (unknown kind string). Document both as the variants the SvelteKit Principals editor surfaces.
DOC POINTERS (read before coding):
  - brain/.rosetta/docs/tauri/commands.md → the #[tauri::command] + State<'_, T> shape; capability/permission model
  - brain/.rosetta/docs/tauri/permissions.md → Tauri v2 autogenerated allow-<command> permissions; core:default admission
  - brain/docs/rust/error-handling.md → Result + ? + anyhow::Context; structured Denial classification
  - crates/AGENTS.md → but-api is THE API boundary; gix over git2; read governance config at the target ref for authorization

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Proven against real but-authz + real gix via but_testsupport (hand-assertion style, like admin_write_guard / branch_gates): (1) an admin principal_kind_update sets kind="agent" for agent-A in the WORKING-TREE permissions.toml while principal_kind_read still reports the COMMITTED (target-ref) kind — inert until committed; once committed (via the governance_commit path) the new kind is the target-ref truth; (2) the writer LOSSLESSLY round-trips the FULL [[principal]]+[[group]] schema (every grant, role, group, member, and every UNRELATED principal's kind) on a kind-only edit — no grant/group/member is dropped (the CLI-001 lossless-round-trip invariant, extended to the kind field); (3) setting kind="human" lands the human declaration (explicit "human" or cleared-to-None — assert the chosen behavior); (4) a non-admin principal_kind_update is denied perm.denied (names administration:write) and writes NOTHING (working-tree permissions.toml byte-for-byte unchanged); (5) principal_kind_read returns each principal's committed declared kind PLUS a pending signal from the working-tree-vs-target-ref diff; (6) the desktop principal_kind_update #[tauri::command] resolves the human fleet-owner via DesktopSessionState (no BUT_AGENT_HANDLE) and writes the kind without it, and an agent-env invoke lacking administration:write is denied perm.denied — both proven on the REAL Tauri mock-runtime bus (mgmt_ipc_003 idiom); (7) `pnpm build:sdk && pnpm format` regenerates the SDK with principal_kind_read/principal_kind_update + their DTOs, and the generated TS type-checks in the desktop; (8) the kind value enters NO enforcement path — it does not flow into GovConfig.principals and no gate reads it (LPR-005's invariant_build_gates honesty grep over the enforcement paths stays green); cargo test -p but-api / -p gitbutler-tauri green; clippy clean.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST treat the kind write as GOVERNED CONFIG — administration:write-gated AND ref-pinned (inert-until-committed working-tree write of permissions.toml), EXACTLY like perm_grant/branch_gates_update. This is the load-bearing distinction from keep_reviews_local (LPR-006), which is a trusted-desktop project preference (NOT admin-gated, NOT ref-pinned). The kind edit declares a governed config fact about a principal, so it rides the governed config path: compose enforce_administration_write_gate(&repo, target_ref) (config_mutate.rs) BEFORE the write and surface denial via classify_error. Do NOT route the kind write through the Project store or a local toggle.
- [MUST] MUST compose the EXISTING admin-write guard, never author a second one. principal_kind_update calls but_api::legacy::config_mutate::enforce_administration_write_gate(&repo, target_ref) BEFORE the std::fs::write, surfacing denial via config_mutate::classify_error. Do NOT write a parallel authorize(AdministrationWrite) call (the AUTHZ-007/008 invariant_build_gates honesty grep must stay green — no role-label/human-vs-AI branching).
- [MUST] MUST write the WORKING TREE only (inert until committed). The writer resolves repo.workdir() and std::fs::write's .gitbutler/permissions.toml there (mirror the CLI-001 permissions.toml writer + admin_write_guard.rs:158-164). It MUST NOT git add / stage / commit / touch any ref — effectiveness comes from the next target-ref load + the existing governance_commit path (permissions.toml is already a GOVERNANCE_COMMIT_PATHS member, governance.rs:155 — so committing the kind edit reuses governance_commit, NO new commit verb). AC-1's inert pair + AC-4's byte-unchanged-on-denial are the proofs.
- [MUST] MUST LOSSLESSLY round-trip the FULL permissions.toml schema on a kind-only edit. permissions.toml carries [[principal]] {id, permissions, role, groups, kind} and [[group]] {name, permissions, members}. A kind-only edit MUST preserve every grant, role, group, member, and every UNRELATED principal's kind. Reuse CLI-001's owned raw wire structs (which already round-trip the file losslessly) — add the kind field to them if absent, coordinating with LPR-005 which adds kind to the LOADER-side PrincipalWire. Re-serialize via toml::to_string (canonical, VALUE-preserving). Dropping a grant/group/role on a kind edit is the cardinal lossy bug AC-2 catches.
- [MUST] MUST resolve the .gitbutler/permissions.toml path via the but-authz permissions_path() accessor (the CLI-001 single-source-of-truth pattern, the analog of gates_path()), never a re-derived literal. The SEC-honesty grep `! grep -q '\.gitbutler/permissions\.toml' crates/but-api/src/legacy/governance.rs` must stay green (governance.rs already honors this for the existing writers).
- [MUST] MUST resolve the HUMAN fleet-owner identity in the DESKTOP principal_kind_update Tauri command. BUT_AGENT_HANDLE is UNSET in the desktop process; the tauri_principal_kind_update wrapper resolves the fleet-owner via DesktopSessionState (governance.rs:82 fleet_owner_context → principal_kind_update_with_repo_as_fleet_owner), EXACTLY like tauri_branch_gates_update (governance.rs:585). The but-api principal_kind_update KEEPS its (ctx/&repo, target_ref, principal, kind) signature + env-principal resolution (BUT_AGENT_HANDLE) which the rust tests exercise via temp_env; only the Tauri layer maps the desktop human identity onto that principal. (SEC-3, mirrored from MGMT-BE-004.)
- [MUST] MUST keep principal_kind_read a self-/branch-scoped READ with NO write authority (mirror governance_principals_list governance.rs:535 / perm_list governance.rs:437): it reads the committed kinds at the target ref and rides the but-api #[but_api(napi)]-generated tauri_principal_kind_read module on core:default. Do NOT gate the read on administration:write (the api-design read-scope; a kind reconnaissance is not a config mutation). (If api-design.md scopes principal-config READS to administration:read like branch_gates_read, follow that — match the existing perm_list/governance_principals_list read posture for consistency; name the chosen read scope in the test.)
- [MUST] MUST surface the SDK delta as part of done. After the Rust API lands, `pnpm build:sdk && pnpm format` regenerates packages/but-sdk/src/generated with principal_kind_read/principal_kind_update + the PrincipalKindList/PrincipalKindEntry/PrincipalKindOutcome DTOs; the generated TS type-checks in the desktop; the generated files are NEVER hand-edited.
- [MUST] MUST register BOTH commands in the SAME task (the atomic command+permission rule): add tauri_principal_kind_read::principal_kind_read (but-api) + $crate::governance::tauri_principal_kind_update::principal_kind_update (gitbutler-tauri) to gitbutler_governance_command_rows! (lib.rs:204), AND add both to GOVERNANCE_COMMANDS + an InvocationCase each in mgmt_ipc_003_governance_commands.rs. Registration IS the admission — no command may ship without its registration entry, and no registration without its command.
- [NEVER] NEVER let the kind value enter an ENFORCEMENT path — it does NOT flow into GovConfig.principals (the enforcement map, config.rs:85) and NO gate reads it (LPR-005's invariant). principal_kind_read/update READ/WRITE the descriptor; they MUST NOT make a kind-conditioned authorization decision, and the writer MUST NOT route kind into the AuthoritySet. The agent-vs-AI predicate is NEVER an enforcement branch (the invariant_build_gates honesty grep over governance.rs — an ENFORCEMENT_PATH — must stay green).
- [NEVER] NEVER touch the merge gate or any of the three LPR drive tables (local_review_assignments/comments/meta) — the kind producer is a permissions.toml read+write only; the safe seam (the gate reads only local_review_verdicts) is untouched. This task adds NO read of the new tables to merge_gate.rs.
- [NEVER] NEVER drop, normalize, or smooth any [[principal]]/[[group]] entry (grant, role, group, member, or another principal's kind) on a kind-only edit (lossy round-trip = silent governance weakening — CRITICAL; AC-2 catches it).
- [NEVER] NEVER commit, stage, or move a ref from the production writer (breaks inert-until-committed; AC-1's ref-unchanged + AC-4's byte-unchanged catch it).
- [NEVER] NEVER re-implement admin gating — compose enforce_administration_write_gate; do not fork a second authorize(AdministrationWrite).
- [NEVER] NEVER read governance config from the working tree or feature head for the AUTHORIZATION decision — admin authorization reads the TARGET REF blob (CAP-CONFIG-01); only the WRITE edits the working tree.
- [NEVER] NEVER pass BUT_AGENT_HANDLE-derived identity from the desktop principal_kind_update command (it is unset there) — resolve the human fleet-owner via DesktopSessionState (SEC-3).
- [NEVER] NEVER author a per-command allow-governance_*/allow-principal_kind_* capability FILE — governance commands ride core:default; the IPC test forbids such files. The per-command permission is the macro-autogenerated allow-<command>, never hand-written.
- [NEVER] NEVER hand-edit packages/but-sdk/src/generated — regenerate via pnpm build:sdk only.
- [STRICTLY] STRICTLY site principal_kind_read/principal_kind_update at the but-api boundary (crates/but-api/src/legacy/governance.rs, beside perm_*/branch_gates_*) so the Tauri command, the CLI, and the N-API binding all reuse the SAME fns — never bury the read/write logic in crates/but/ or in gitbutler-tauri.
- [STRICTLY] STRICTLY coordinate the wire-struct kind field with LPR-005: LPR-005 adds the LOADER-side `#[serde(default)] pub kind: Option<String>` to PrincipalWire (config.rs:424) for the tag-derivation read; this task's WRITER round-trips that same field on the raw permissions.toml structs. If the raw writer structs (CLI-001) are distinct from the loader's PrincipalWire, add kind to the writer structs too (deny_unknown_fields-compatible). Do NOT define a third divergent principal shape.
- [STRICTLY] STRICTLY keep the function signatures (ctx/&repo, target_ref, principal, kind) so the Tauri command passes the same target ref the workspace resolves and the CLI resolves from the workspace target.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [x] AC-1 [PRIMARY]: admin principal_kind_update writes kind into the working-tree permissions.toml while principal_kind_read reports the COMMITTED (target-ref) kind (inert-until-committed pair)
- [x] AC-2: the writer LOSSLESSLY round-trips the full [[principal]]+[[group]] schema (every grant/role/group/member + every unrelated principal's kind) on a kind-only edit
- [x] AC-3: principal_kind_read returns each principal's committed declared kind PLUS a pending signal from the working-tree-vs-target-ref diff
- [x] AC-4: a non-admin principal_kind_update is denied perm.denied (names administration:write) and writes NOTHING (working-tree permissions.toml byte-for-byte unchanged)
- [x] AC-5: the desktop principal_kind_update Tauri command resolves the human fleet-owner via DesktopSessionState (no BUT_AGENT_HANDLE) and writes the kind; an agent-env invoke lacking administration:write is denied — both on the REAL Tauri mock-runtime bus; `pnpm build:sdk && pnpm format` regenerates the SDK and it type-checks
- [x] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: admin kind write is inert until committed
  GIVEN: kind_governance_base: refs/heads/main has committed .gitbutler/permissions.toml with [[principal]] id="admin" permissions=["administration:write","merge"]; [[principal]] id="agent-A" permissions=["contents:write"] with NO kind declared (kind=None → human); [[principal]] id="rust-implementer" permissions=["contents:write"]; BUT_AGENT_HANDLE=admin; clean working tree; ref_id(main) captured
  WHEN:  principal_kind_update(&repo, "refs/heads/main", "agent-A", "agent") runs under #[serial_test::serial] via temp_env BUT_AGENT_HANDLE=admin
  THEN:  the call returns Ok; the WORKING-TREE .gitbutler/permissions.toml now declares kind="agent" for agent-A; AND principal_kind_read(&repo, "refs/heads/main") reports agent-A's COMMITTED kind as None/human (read from the target-ref blob — the edit is inert; the committed value is unchanged); ref_id(main) AFTER == BEFORE (no commit); the returned caveat contains "takes effect once committed to the target branch"
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api principal_kind_update + the task's OWN principal_kind_read reading the target-ref committed permissions.toml blob + real gix (committed target-ref config vs working-tree write) via but_testsupport::writable_scenario
  VERIFY: cargo test -p but-api principal_kind_update_writes_worktree_inert_until_committed

AC-2: kind-only edit round-trips the full permissions.toml schema losslessly
  GIVEN: kind_governance_base: permissions.toml with admin (administration:write,merge), agent-A (contents:write, groups=["reviewers"]), rust-implementer (contents:write, kind="human"), and [[group]] reviewers permissions=["reviews:write"] members=["agent-A"]; BUT_AGENT_HANDLE=admin
  WHEN:  principal_kind_update(&repo, "refs/heads/main", "agent-A", "agent") runs as admin (a kind-only edit touching ONLY agent-A's kind)
  THEN:  the rewritten working-tree permissions.toml STILL carries admin's full grant set, agent-A's contents:write + groups=["reviewers"], rust-implementer's contents:write + kind="human", AND the [[group]] reviewers entry (permissions + members) intact — only agent-A's kind changed to "agent"; no grant/role/group/member/other-principal-kind is dropped, re-loadable through but_authz::load_governance_config
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api principal_kind_update + real gix working-tree read-back parsing the full [[principal]]+[[group]] schema (LOSSLESS) via but_authz
  VERIFY: cargo test -p but-api principal_kind_update_round_trips_full_schema_lossless

AC-3: principal_kind_read returns committed kinds + a pending signal
  GIVEN: kind_governance_base (committed agent-A kind=None/human), then an admin principal_kind_update(... "agent-A", "agent") has written kind="agent" to the working tree (uncommitted) — i.e. AC-1's post-state; BUT_AGENT_HANDLE=admin
  WHEN:  principal_kind_read(&repo, "refs/heads/main") runs
  THEN:  the returned list shows agent-A's COMMITTED kind=None/human (read at the target ref) AND a pending=true signal because the working-tree kind="agent" differs from the committed kind; on a clean working tree (no edit) pending=false; every committed principal is listed with its declared kind
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api principal_kind_read reading BOTH the target-ref committed config and the working-tree file + real gix
  VERIFY: cargo test -p but-api principal_kind_read_returns_committed_kinds_with_pending_signal

AC-4: a non-admin kind write is denied perm.denied and writes nothing
  GIVEN: kind_governance_base: caller rust-implementer holds ["contents:write"] only (NO administration:write) at the target ref; the working-tree permissions.toml captured byte-for-byte before the call
  WHEN:  principal_kind_update(&repo, "refs/heads/main", "agent-A", "agent") runs with BUT_AGENT_HANDLE=rust-implementer
  THEN:  the call returns Err; classify_error(&err) yields Some(_) whose .code == "perm.denied"; the message contains "administration:write"; AND the working-tree .gitbutler/permissions.toml is byte-for-byte UNCHANGED from before the call (the admin gate ran BEFORE any write)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api principal_kind_update composing enforce_administration_write_gate + real but-authz + real gix
  VERIFY: cargo test -p but-api principal_kind_update_non_admin_denied_writes_nothing

AC-5: the desktop Tauri command resolves the fleet-owner (no BUT_AGENT_HANDLE), denies a non-admin agent-env invoke, and the SDK regenerates + type-checks
  GIVEN: a governance_api_repo with committed permissions.toml (admin holds administration:write; rust-implementer holds contents:write only) + a TestDesktopSession resolving the fleet-owner; the real Tauri mock runtime (governance_app/governance_webview, mgmt_ipc_003 idiom)
  WHEN:  principal_kind_update is invoked on the bus with BUT_AGENT_HANDLE UNSET (desktop fleet-owner path) to set agent-A kind="agent"; AND separately an agent-env path lacking administration:write attempts the same; AND `pnpm build:sdk && pnpm format` runs
  THEN:  the fleet-owner invoke returns Ok and writes kind="agent" to the working tree WITHOUT BUT_AGENT_HANDLE (the DesktopSessionState shim resolved the identity); the non-admin agent-env invoke is denied perm.denied naming administration:write with no write; principal_kind_read invokes on the bus and returns the kind list; AND packages/but-sdk/src/generated contains principal_kind_read/principal_kind_update + the PrincipalKindList/PrincipalKindOutcome DTOs and the generated TS type-checks (no hand-edit)
  TEST_TIER: integration   VERIFICATION_SERVICE: real Tauri mock-runtime bus (tauri::test::get_ipc_response, mgmt_ipc_003 idiom) resolving DesktopSessionState + real but-api principal_kind_update + real but-authz; real `pnpm build:sdk && pnpm format` SDK regen + tsc
  VERIFY: cargo test -p gitbutler-tauri mgmt_ipc_003 && pnpm build:sdk && pnpm format && grep -rq "principal_kind_update\|principalKindUpdate" packages/but-sdk/src/generated

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): after admin principal_kind_update(agent-A, "agent"), the working-tree permissions.toml declares kind="agent" for agent-A
    VERIFY: cargo test -p but-api principal_kind_update_writes_worktree_inert_until_committed
- TC-2 (-> AC-1): after the update principal_kind_read reports agent-A's COMMITTED kind None/human AND ref_id(refs/heads/main) is identical before/after (inert / no commit) AND the caveat contains "takes effect once committed to the target branch"
    VERIFY: cargo test -p but-api principal_kind_update_writes_worktree_inert_until_committed
- TC-3 (-> AC-2): a kind-only edit re-serializes admin's grants, agent-A's grants+groups, rust-implementer's grants+kind, AND the [[group]] reviewers entry intact (lossless; only agent-A's kind changed)
    VERIFY: cargo test -p but-api principal_kind_update_round_trips_full_schema_lossless
- TC-4 (-> AC-2): re-loading the rewritten file through but_authz::load_governance_config confirms every grant/group/member survives (no silent governance weakening on a kind edit)
    VERIFY: cargo test -p but-api principal_kind_update_round_trips_full_schema_lossless
- TC-5 (-> AC-3): principal_kind_read returns committed agent-A kind=None/human with pending=true after the uncommitted kind="agent" edit; pending=false on a clean working tree
    VERIFY: cargo test -p but-api principal_kind_read_returns_committed_kinds_with_pending_signal
- TC-6 (-> AC-4): a non-admin principal_kind_update returns Err; classify_error(&err) .code == "perm.denied" and the message contains "administration:write"
    VERIFY: cargo test -p but-api principal_kind_update_non_admin_denied_writes_nothing
- TC-7 (-> AC-4): after the denied non-admin principal_kind_update, the working-tree permissions.toml is byte-for-byte unchanged (gate ran before any write)
    VERIFY: cargo test -p but-api principal_kind_update_non_admin_denied_writes_nothing
- TC-8 (-> AC-5): the fleet-owner principal_kind_update bus invoke (BUT_AGENT_HANDLE unset) writes kind="agent" via DesktopSessionState; the non-admin agent-env invoke is denied perm.denied naming administration:write with no write
    VERIFY: cargo test -p gitbutler-tauri mgmt_ipc_003
- TC-9 (-> AC-5): `pnpm build:sdk && pnpm format` regenerates packages/but-sdk/src/generated containing principal_kind_read/principal_kind_update + their DTOs, and the generated TS type-checks (no hand-edit)
    VERIFY: pnpm build:sdk && pnpm format && git diff --name-only packages/but-sdk/src/generated | grep -q . && grep -rq "principal_kind_update\|principalKindUpdate" packages/but-sdk/src/generated
- TC-10 (-> AC-5): principal_kind_read AND principal_kind_update are in GOVERNANCE_COMMANDS + the gitbutler_governance_command_rows! macro; both invoke on the real bus; an unregistered probe is rejected ("not found")
    VERIFY: cargo test -p gitbutler-tauri mgmt_ipc_003
- TC-11 (-> AC-5): no allow-principal_kind_*/allow-governance_* capability file is introduced (governance commands ride core:default — the IPC forbidden-allow-file assertion stays green)
    VERIFY: cargo test -p gitbutler-tauri mgmt_ipc_003

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01, CAP-CONFIG-01
provides:
  - principal_kind_read(ctx, target_ref) -> a self-/branch-scoped read returning each principal's committed declared kind (agent|human|None→human) at the target ref + a pending signal from the working-tree-vs-target-ref diff
  - principal_kind_update(ctx, target_ref, principal, kind) -> an administration:write-gated, inert-until-committed working-tree write of permissions.toml that LOSSLESSLY round-trips the full [[principal]]+[[group]] schema and sets only the targeted principal's kind (the governed-config kind writer, mirroring branch_gates_update/perm_grant)
  - a #[tauri::command] principal_kind_read (but-api #[but_api(napi)]-generated, core:default) + principal_kind_update (gitbutler-tauri fleet-owner wrapper via DesktopSessionState), surfaced through the regenerated packages/but-sdk
consumes:
  - but_api::legacy::config_mutate::{enforce_administration_write_gate, classify_error} (the AUTHZ-006 admin-write guard — COMPOSED, never re-implemented)
  - but_authz::{load_governance_config, governance_present, permissions_path} + the additive PrincipalWire kind field (LPR-005 adds the loader-side field; this task's writer round-trips it)
  - the CLI-001 permissions.toml lossless read-modify-write wire structs (extended with the kind field) + the existing governance_commit pending→commit path (permissions.toml is a GOVERNANCE_COMMIT_PATHS member)
  - gitbutler-tauri::governance::{DesktopSessionState, fleet_owner_context, context_for_project} (the Sprint-06a human-fleet-owner shim the desktop write wrapper reuses)
  - gix::Repository::workdir (the working-tree write target) + target-ref tree/blob read (the committed-config read)
boundary_contracts:
  - CAP-AUTHZ-01: principal_kind_update authorizes administration:write (read at the target ref) via enforce_administration_write_gate BEFORE any write; a non-admin write is denied perm.denied and writes nothing. The kind value is enforcement-NEUTRAL: it does NOT enter GovConfig.principals and NO gate reads it (LPR-005's invariant) — the producer reads/writes a descriptor, it never makes a kind-conditioned authorization decision. principal_kind_read is a self-/branch-scoped read matching the perm_list/governance_principals_list read posture.
  - CAP-CONFIG-01: principal_kind_update writes inert-until-committed config to the WORKING TREE only — effectiveness comes from the next target-ref load + the existing governance_commit path. The writer LOSSLESSLY round-trips the full [[principal]]+[[group]] schema (dropping a grant/role/group/member/kind is a CRITICAL lossy-round-trip failure). The kind write is governed config (ref-pinned, admin-gated) — UNLIKE keep_reviews_local (LPR-006), a trusted-desktop project preference.
  - Safe-seam note: this producer touches permissions.toml ONLY; it adds NO read of the three LPR drive tables to merge_gate.rs and never touches the merge gate. The gate reads only local_review_verdicts (LPR-009's invariant), unaffected.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-api/src/legacy/governance.rs (MODIFY — add principal_kind_read/principal_kind_update + the private permissions.toml kind read-modify-write setter (lossless full-schema round-trip) + the PrincipalKindList/PrincipalKindEntry/PrincipalKindOutcome DTOs + principal_kind_update_with_repo/_as_fleet_owner; sited BESIDE perm_*/branch_gates_*; compose enforce_administration_write_gate)
  - crates/but-api/tests/principal_kind.rs (NEW — the PRIMARY but-api proofs AC-1..AC-4)
  - crates/gitbutler-tauri/src/governance.rs (MODIFY — add principal_kind_update_for_desktop_session + the tauri_principal_kind_update::principal_kind_update #[tauri::command] wrapper via DesktopSessionState, mirroring tauri_branch_gates_update)
  - crates/gitbutler-tauri/src/lib.rs (MODIFY — register tauri_principal_kind_read::principal_kind_read + $crate::governance::tauri_principal_kind_update::principal_kind_update in gitbutler_governance_command_rows!)
  - crates/gitbutler-tauri/tests/mgmt_ipc_003_governance_commands.rs (MODIFY — add principal_kind_read/principal_kind_update to GOVERNANCE_COMMANDS + an InvocationCase each; the AC-5 fleet-owner + non-admin-denial bus proofs)
  - packages/but-sdk/src/generated/** (REGENERATE ONLY via `pnpm build:sdk && pnpm format` — NEVER hand-edit)
writeProhibited:
  - crates/but-api/src/legacy/config_mutate.rs — CONSUME-only (the AUTHZ-006 admin guard); compose enforce_administration_write_gate, do not fork a second admin check
  - crates/but-api/src/legacy/merge_gate.rs, review_requirement.rs — CONSUME-only (the safe seam); do NOT add any read of the new tables or the kind field to the gate path (LPR-009 greps this)
  - crates/but-authz/src/{authorize.rs, denial.rs, principal.rs, authority.rs} — the union/primitive layer is closed; the kind field is enforcement-neutral (LPR-005 owns the loader-side PrincipalWire kind field — coordinate, do not duplicate)
  - crates/but-authz/src/config.rs — LPR-005 adds the loader-side kind field; this task's WRITER may add kind to the CLI-001 raw writer structs if distinct, but do NOT change the loader/normalize semantics or let kind enter GovConfig.principals
  - capabilities/main.json + any allow-governance_*/allow-principal_kind_* file — do NOT add a per-command allow file (core:default admits; the IPC test forbids such files)
  - crates/gitbutler-project/** — the kind write is GOVERNED CONFIG, not a Project preference (that is keep_reviews_local, LPR-006); do NOT route kind through the project store
  - any gitbutler-* crate beyond gitbutler-tauri (no new gitbutler-* usage; gitbutler-tauri is the desktop shell this producer extends)
  - Any file not in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. .spec/.../sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-BE-004-branch-gates-config-writer.md — [PRIMARY PATTERN to mirror] the canonical but-api governed-config producer + its Tauri command/SDK delta: compose enforce_administration_write_gate, working-tree-only inert write, value-preserving lossless read-modify-write over raw wire structs, the ref-pin caveat, the path accessor, and the SDK regen. This task is the permissions.toml `kind` analog of that gates.toml writer.
2. crates/gitbutler-tauri/src/governance.rs [82-90, 244-261, 585-608] — [PRIMARY PATTERN for the Tauri delta] fleet_owner_context (the DesktopSessionState shim) + branch_gates_update_for_desktop_session + the tauri_branch_gates_update::branch_gates_update #[tauri::command] module. Mirror EXACTLY for principal_kind_update (the read rides but-api's #[but_api(napi)] like tauri_perm_list).
3. crates/but-api/src/legacy/governance.rs [157-214, 437-540, 1533-...] — BranchProtectionInput/GovernancePrincipalsList/BranchGatesOutcome DTO shapes (schemars::JsonSchema, camelCase); perm_list/governance_principals_list (#[but_api(napi)] reads); perm_grant_with_repo_as_fleet_owner (the desktop fleet-owner write helper to mirror). GOVERNANCE_COMMIT_PATHS:155 (permissions.toml is a commit member — the kind edit reuses governance_commit).
4. crates/gitbutler-tauri/src/lib.rs [200-225] — the gitbutler_governance_command_rows! macro: the read command rides but_api::legacy::governance::tauri_<name>::<name>, the write rides $crate::governance::tauri_<name>::<name>. Register both new commands here.
5. crates/gitbutler-tauri/tests/mgmt_ipc_003_governance_commands.rs [16-33, 230-300] — GOVERNANCE_COMMANDS const + the InvocationCase table + the real bus idiom (governance_app/governance_webview/get_ipc_response) + the forbidden-allow-governance_*-file assertion (240-253). Add the two new commands + the fleet-owner/non-admin-denial AC-5 proofs.
6. .spec/.../sprint-07-local-agent-pr/LPR-005-derived-pr-lifecycle-agent-tag.md [the kind field section] — LPR-005 adds the LOADER-side `#[serde(default)] pub kind: Option<String>` to PrincipalWire (config.rs:424) for the tag-derivation read; this task's WRITER round-trips that same field. Coordinate the wire shape; do not define a third principal struct.
7. crates/but-api/tests/admin_write_guard.rs + branch_gates.rs (if present) — the write-worktree helper (repo.workdir()+std::fs::write), committed_blob_text for byte-unchanged assertions (AC-4), temp_env BUT_AGENT_HANDLE under #[serial_test::serial], writable_scenario + invoke_bash committing config at main. Mirror for principal_kind.rs.
8. .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md §A.4 — the kind descriptor is read at the target ref, changes no enforcement, defaults human; the source-of-truth for the agent tag. §H — kind is administration:write-gated governed config.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- cargo test -p but-api principal_kind_update_writes_worktree_inert_until_committed   -> Exit 0; working-tree kind="agent" lands, target-ref kind still None/human, ref_id(main) before==after, caveat printed
- cargo test -p but-api principal_kind_update_round_trips_full_schema_lossless   -> Exit 0; every grant/role/group/member/other-kind survives a kind-only edit; only agent-A's kind changed
- cargo test -p but-api principal_kind_read_returns_committed_kinds_with_pending_signal   -> Exit 0; committed kind=None/human + pending=true on uncommitted edit; pending=false on clean tree
- cargo test -p but-api principal_kind_update_non_admin_denied_writes_nothing   -> Exit 0; perm.denied naming administration:write; working-tree permissions.toml byte-for-byte unchanged
- cargo test -p gitbutler-tauri mgmt_ipc_003   -> Exit 0; principal_kind_read/update register + invoke on the real bus; fleet-owner write without BUT_AGENT_HANDLE; non-admin agent-env denied; no forbidden allow-governance_* file
- cargo check -p but-api -p gitbutler-tauri --all-targets   -> Exit 0
- cargo clippy -p but-api -p gitbutler-tauri --all-targets   -> Exit 0
- cargo test -p but-authz invariant_build_gates   -> Exit 0; no role-label/human-vs-AI branching in the governance authorization path; kind enters no enforcement path
- cargo fmt --check   -> Exit 0
- pnpm build:sdk && pnpm format   -> Exit 0; packages/but-sdk/src/generated contains principal_kind_read/principal_kind_update + DTOs; generated TS type-checks; no hand-edit
- pnpm -F @gitbutler/desktop check   -> Exit 0; the regenerated SDK type-checks in the desktop frontend
- ! grep -q '\.gitbutler/permissions\.toml' crates/but-api/src/legacy/governance.rs   -> Exit 0; governance.rs resolves the path via but_authz::permissions_path(), never a re-derived literal (SEC-8 single source of truth)

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references:
  - .spec/.../sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-BE-004-branch-gates-config-writer.md (the canonical governed-config producer + Tauri/SDK delta — mirror its shape for the permissions.toml kind field)
  - crates/gitbutler-tauri/src/governance.rs:585 (tauri_branch_gates_update — the fleet-owner write wrapper to mirror), :82 (fleet_owner_context — the DesktopSessionState shim)
  - crates/but-api/src/legacy/governance.rs:437 (perm_list #[but_api(napi)] read), :1533 (perm_grant_with_repo_as_fleet_owner — the desktop write helper), :192 (GovernancePrincipalsList DTO shape to mirror), :155 (GOVERNANCE_COMMIT_PATHS — permissions.toml is a commit member)
  - .spec/prds/governance/10-technical-requirements/04-api-design.md (the principal-config read/write route → Authority rows; the kind field)
  - .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md §A.4 + §H (kind read at the target ref, enforcement-neutral, administration:write-gated governed config)
notes:
  - Target-ref resolution: principal_kind_read/update take target_ref: String; the Tauri command resolves it from the WORKSPACE TARGET (context_for_project sets project_meta.target_ref, governance.rs:306); the but-api tests pass "refs/heads/main" directly (the fixture commits governance at main).
  - Schema ownership: the writer round-trips the FULL permissions.toml ([[principal]] {id, permissions, role, groups, kind} + [[group]] {name, permissions, members}) via the CLI-001 owned raw wire structs (add kind if absent) — VALUE-preserving toml::to_string. It cannot reuse but-authz's GovConfig (the enforcement map drops the raw layout). LPR-005 owns the LOADER-side kind field; this is the WRITER side of the same field.
  - Inert / pending (AC-1/AC-3): principal_kind_update writes the working tree only; principal_kind_read loads the committed kind set (target-ref blob) AND parses the working-tree file; a kind that differs renders pending=true. A clean working tree yields pending=false. The kind edit is committed via the EXISTING governance_commit path (permissions.toml ∈ GOVERNANCE_COMMIT_PATHS) — NO new commit verb.
  - Desktop identity (SEC-3): the principal_kind_update Tauri command resolves the human fleet-owner via DesktopSessionState (BUT_AGENT_HANDLE unset on desktop), EXACTLY like tauri_branch_gates_update. The but-api fn keeps env-principal resolution (exercised by the rust tests via temp_env). The READ rides but-api's #[but_api(napi)]-generated tauri module on core:default (like tauri_perm_list — no fleet-owner needed for a read).
  - Capability/permission delta (the atomic rule): NO capabilities/main.json change (core:default admits); the per-command allow-principal_kind_read/allow-principal_kind_update permission is macro-autogenerated; the SHIPPED-TOGETHER permission entry is the gitbutler_governance_command_rows! registration + the GOVERNANCE_COMMANDS/InvocationCase entries (registration IS admission). The IPC test forbids a hand-written allow-governance_* file.
  - kind validation: accept only "agent"|"human" (typed parse/name round-trip like AssignmentState, LPR-002); reject unknown strings with config.invalid (classify_error). The frontend handles perm.denied (non-admin) + config.invalid (bad kind).
  - Enforcement neutrality (LPR-005's invariant, re-asserted): the kind value MUST NOT enter GovConfig.principals and MUST NOT condition any authorization branch in governance.rs (an ENFORCEMENT_PATH) — it is a descriptor on the read+write path only. The invariant_build_gates honesty grep stays green.
pattern: a thin but-api governed-config producer pair — principal_kind_read (committed-kinds renderer + pending diff, a #[but_api(napi)] read on core:default) and principal_kind_update (composes the AUTHZ-006 admin guard, then read-modify-writes the WORKING-TREE permissions.toml LOSSLESSLY round-tripping the full [[principal]]+[[group]] schema and setting only the targeted kind, inert until committed) — plus the desktop fleet-owner #[tauri::command] write wrapper (DesktopSessionState) and the regenerated SDK; mirrors MGMT-BE-004 exactly for the Tauri/SDK delta shape
pattern_source: .spec/.../MGMT-BE-004 (the governed-config producer + Tauri/SDK delta to mirror); crates/gitbutler-tauri/src/governance.rs:585 (the fleet-owner write wrapper); crates/but-api/src/legacy/governance.rs:437/:1533/:192 (the #[but_api(napi)] read + the fleet-owner write helper + the DTO shape); crates/gitbutler-tauri/src/lib.rs:204 (the command-rows macro); crates/gitbutler-tauri/tests/mgmt_ipc_003_governance_commands.rs (the real-bus registration+invoke idiom)
anti_pattern: routing the kind write through the Project store / a local toggle instead of the admin-gated ref-pinned governed-config path (it is governed config, not a keep_reviews_local-class preference); round-tripping through but-authz GovConfig (drops the raw [[principal]]/[[group]] layout — the lossy bug AC-2 catches); committing/staging the production write (breaks inert-until-committed — AC-1's ref-unchanged fails); authoring a second authorize(AdministrationWrite) instead of composing enforce_administration_write_gate; running the write BEFORE the admin gate (AC-4's byte-unchanged fails); passing BUT_AGENT_HANDLE identity from the desktop command (it is unset — resolve the fleet-owner via DesktopSessionState, SEC-3); authoring a per-command allow-governance_*/allow-principal_kind_* capability FILE (the IPC test forbids it — core:default admits); letting kind enter GovConfig.principals or condition an authorization branch (LPR-005's invariant — the honesty grep catches it); hand-editing packages/but-sdk/src/generated; re-deriving the ".gitbutler/permissions.toml" literal instead of permissions_path()

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agent: implementer=tauri-implementer | reviewer=tauri-reviewer
rationale: This is a Tauri-shaped producer task — the deliverable is the DESKTOP command surface (a #[but_api(napi)] read on core:default + a gitbutler-tauri DesktopSessionState fleet-owner write wrapper) + the regenerated TS SDK that type-checks in the SvelteKit desktop, sitting atop a but-api governed-config read-modify-write. The Tauri-specific competencies — the command↔permission/registration atomicity (core:default admission + the gitbutler_governance_command_rows! macro + the GOVERNANCE_COMMANDS/IPC allowlist, NOT a hand-written allow-file), the DesktopSessionState fleet-owner identity shim (BUT_AGENT_HANDLE unset on desktop), the IPC error serialization (perm.denied/config.invalid via json::Error), the real Tauri mock-runtime bus proof, and the SDK-regen-type-checks loop — are exactly tauri-implementer's domain (RULES.md routes the Tauri desktop shell + capabilities/IPC to the tauri-* triad). The lossless permissions.toml round-trip + the composed admin guard are but-api work the implementer carries through the producer; tauri-reviewer adversarially validates the command/permission parity (both registered, no orphan allow-file, no parallel ungated path), the inert write (never committed; ref unchanged), the fleet-owner identity resolution, the kind-enters-no-enforcement-path invariant, and the SDK delta type-checks.
coding_standards: crates/AGENTS.md (Result<T,E> + anyhow::Context; but_error::Code for consumer-facing classification; gix over git2; acquire locks at top-level boundaries; read governance config at the target ref via gix blob read for authorization); RULES.md (but-api is THE API boundary — transport DTOs convert to domain types before calling lower crates; preserve existing but-api macro/transport/serialization patterns; gitbutler-tauri is the desktop shell; after changing Rust APIs exposed via but-sdk run pnpm build:sdk && pnpm format, never hand-edit generated); brain/.rosetta/docs/tauri/ (commands.md the #[tauri::command]+State idiom; permissions.md the core:default admission + autogenerated allow-<command>); crates/gitbutler-tauri/src/governance.rs (the DesktopSessionState fleet-owner wrapper idiom)

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: LPR-005 (the additive optional `kind` field on the loader-side PrincipalWire (config.rs:424) + the AUTHZ-config descriptor — this task's WRITER round-trips that same field and the desktop EXPOSES it; the read-side kind reader LPR-005 introduces); Sprint 05 CLI-001 (the permissions.toml lossless read-modify-write wire structs + the permissions_path() accessor this writer extends); Sprint 06b MGMT-BE-004 (the governed-config producer + Tauri/SDK delta pattern this mirrors exactly, incl. the branch_gates_update fleet-owner wrapper); Sprint 06a MGMT-IPC-003 (the DesktopSessionState human-fleet-owner shim + the gitbutler_governance_command_rows! macro + the mgmt_ipc_003 real-bus test harness this extends)
Blocks:     LPR-014 (the SvelteKit Principals editor that consumes principal_kind_read/principal_kind_update via the regenerated SDK to show/set agent|human)
```
</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "LPR-013",
  "proposed_by": "tauri-planner",
  "platforms": ["desktop"],
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "kind_governance_base": {
      "description": "Real-git scenario via but_testsupport::writable_scenario(\"checkout-head-info\"). Target ref refs/heads/main carries a committed .gitbutler/permissions.toml where `admin` holds administration:write + merge; `agent-A` holds contents:write with NO kind declared (kind=None → human default) and groups=[\"reviewers\"]; `rust-implementer` holds contents:write with kind=\"human\"; plus [[group]] reviewers permissions=[\"reviews:write\"] members=[\"agent-A\"]. Working tree starts clean (matches the committed blob). Seeded via a REAL entrypoint: invoke_bash writes the file and git-commits it at main (the committed-config-at-main pattern from crates/but-api/tests/admin_write_guard.rs). Capture committed_blob_text(repo, but_authz::permissions_path()) and ref_id(repo, \"refs/heads/main\") BEFORE any write for the inert / byte-unchanged assertions.",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"checkout-head-info\");",
        "invoke_bash on main: mkdir -p .gitbutler; write .gitbutler/permissions.toml with [[principal]] id=\"admin\" permissions=[\"administration:write\",\"merge\"]; [[principal]] id=\"agent-A\" permissions=[\"contents:write\"] groups=[\"reviewers\"] (NO kind); [[principal]] id=\"rust-implementer\" permissions=[\"contents:write\"] kind=\"human\"; [[group]] name=\"reviewers\" permissions=[\"reviews:write\"] members=[\"agent-A\"]; then git add .gitbutler/permissions.toml && git commit -m \"governance config\";",
        "Capture committed_blob_text(repo, but_authz::permissions_path()) and ref_id(repo, \"refs/heads/main\") BEFORE any write."
      ]
    },
    "governance_api_repo": {
      "description": "The mgmt_ipc_003 real-Tauri-mock-runtime fixture (crates/gitbutler-tauri/tests/mgmt_ipc_003_governance_commands.rs governance_api_repo + governance_app + governance_webview): a governed repo with committed .gitbutler/permissions.toml (admin holds administration:write; rust-implementer holds contents:write only) + a TestDesktopSession resolving the fleet-owner. Invokes principal_kind_update on the bus via tauri::test::get_ipc_response with BUT_AGENT_HANDLE unset (the desktop fleet-owner path) and, separately, via an agent-env path lacking administration:write. Used for AC-5 (the desktop-bus fleet-owner + non-admin-denial proofs).",
      "seed_method": "public_api",
      "records": [
        "let (repo, _tmp) = governance_api_repo(true); let project_id = project_id_for(&repo)?;",
        "let app = governance_app(test_desktop_session())?; let webview = governance_webview(&app)?;",
        "temp_env BUT_AGENT_HANDLE None → invoke_ok(&webview, \"principal_kind_update\", { projectId, targetRef, principal: \"agent-A\", kind: \"agent\" });",
        "temp_env BUT_AGENT_HANDLE Some(\"rust-implementer\") → invoke_err(&webview, \"agent_perm_grant\"-style agent-env path for the kind write) and assert perm.denied naming administration:write."
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN kind_governance_base: committed permissions.toml with agent-A kind=None/human; BUT_AGENT_HANDLE=admin; clean working tree; ref_id(main) captured WHEN principal_kind_update(&repo, \"refs/heads/main\", \"agent-A\", \"agent\") runs under #[serial_test::serial] via temp_env BUT_AGENT_HANDLE=admin THEN the call returns Ok; the WORKING-TREE permissions.toml declares kind=\"agent\" for agent-A; AND principal_kind_read reports agent-A's COMMITTED kind None/human (target-ref blob — inert); ref_id(main) AFTER == BEFORE (no commit); the caveat contains \"takes effect once committed to the target branch\"",
      "verify": "cargo test -p but-api principal_kind_update_writes_worktree_inert_until_committed",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api principal_kind_update + real gix working-tree write + the task's OWN principal_kind_read target-ref committed-blob read",
        "negative_control": {
          "would_fail_if": [
            "the writer wrote nothing — the working-tree permissions.toml would lack kind=\"agent\" (a no-op stub returning Ok is caught by reading the file back)",
            "the writer COMMITTED the edit — principal_kind_read's committed kind would read \"agent\", breaking the inert assertion, and ref_id(main) would change",
            "principal_kind_read read the working tree as committed — it would report committed kind=\"agent\" with no committed-vs-worktree distinction",
            "the ref-pin caveat string were absent (silent success without warning the operator)"
          ]
        },
        "evidence": { "artifact_type": "stdout", "required_capture": true },
        "cases": [
          {
            "start_ref": "kind_governance_base",
            "action": {
              "actor": "ci",
              "steps": [
                "capture ref_id(&repo, \"refs/heads/main\") BEFORE the write",
                "temp_env::with_var(\"BUT_AGENT_HANDLE\", Some(\"admin\"), ...) under #[serial_test::serial]",
                "principal_kind_update(&repo, \"refs/heads/main\", \"agent-A\", \"agent\")",
                "read the working-tree .gitbutler/permissions.toml back and parse agent-A's kind",
                "principal_kind_read(&repo, \"refs/heads/main\") and inspect agent-A's COMMITTED kind",
                "capture ref_id(&repo, \"refs/heads/main\") AFTER the write"
              ]
            },
            "end_state": {
              "must_observe": [
                "`principal_kind_update` returns `Ok`",
                "the working-tree permissions.toml declares `kind = \"agent\"` for agent-A",
                "the result caveat contains `\"takes effect once committed to the target branch\"`",
                "`principal_kind_read(\"refs/heads/main\")` reports agent-A's COMMITTED kind None/human (target-ref blob)",
                "`ref_id(refs/heads/main)` AFTER == the `ref_id` captured BEFORE"
              ],
              "must_not_observe": [
                "`principal_kind_read` reporting the COMMITTED agent-A kind=\"agent\" (the edit must be inert until committed)",
                "the target ref `refs/heads/main` HEAD sha changing (no commit performed)",
                "agent-A's grants/groups dropped by the kind write"
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
      "description": "GIVEN kind_governance_base WHEN principal_kind_update(&repo, \"refs/heads/main\", \"agent-A\", \"agent\") runs as admin (a kind-only edit) THEN the rewritten working-tree permissions.toml STILL carries admin's full grants, agent-A's contents:write + groups=[\"reviewers\"], rust-implementer's contents:write + kind=\"human\", AND the [[group]] reviewers entry (permissions + members) intact — only agent-A's kind changed to \"agent\"; re-loadable through but_authz::load_governance_config with no dropped grant/role/group/member/other-kind",
      "verify": "cargo test -p but-api principal_kind_update_round_trips_full_schema_lossless",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api principal_kind_update + real gix working-tree read-back parsing the full [[principal]]+[[group]] schema via but_authz",
        "negative_control": {
          "would_fail_if": [
            "the kind edit dropped a grant/role/group/member from any principal (the cardinal lossy-round-trip bug)",
            "rust-implementer's kind=\"human\" or the [[group]] reviewers members were lost on agent-A's edit",
            "the write round-tripped through GovConfig (which drops the raw [[principal]]/[[group]] layout)"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "kind_governance_base",
            "action": { "actor": "ci", "steps": [ "BUT_AGENT_HANDLE=admin", "principal_kind_update(refs/heads/main, agent-A, agent)", "re-load the working-tree permissions.toml via but_authz::load_governance_config", "assert every grant/role/group/member/other-kind survives and only agent-A's kind changed" ] },
            "end_state": {
              "must_observe": [
                "admin's full grant set, agent-A's grants+groups, rust-implementer's grants+kind=\"human\", and the [[group]] reviewers entry all intact",
                "agent-A's kind == \"agent\" (the only change)"
              ],
              "must_not_observe": [
                "any dropped grant/role/group/member on the kind-only edit",
                "rust-implementer's kind=\"human\" or the group members lost"
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
      "description": "GIVEN kind_governance_base post AC-1 (working-tree agent-A kind=\"agent\", uncommitted) WHEN principal_kind_read(&repo, \"refs/heads/main\") runs THEN the list shows agent-A's COMMITTED kind None/human (target ref) AND pending=true because the working-tree kind=\"agent\" differs from committed; on a clean working tree pending=false; every committed principal is listed with its declared kind",
      "verify": "cargo test -p but-api principal_kind_read_returns_committed_kinds_with_pending_signal",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api principal_kind_read reading BOTH the target-ref committed config and the working-tree file + real gix",
        "negative_control": {
          "would_fail_if": [
            "principal_kind_read reported the working-tree kind as committed (no committed-vs-worktree distinction — pending never set)",
            "pending were false despite the uncommitted kind edit (the diff is not computed)",
            "a committed principal were omitted from the list"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "kind_governance_base",
            "action": { "actor": "ci", "steps": [ "apply AC-1's working-tree kind=\"agent\" edit (uncommitted)", "principal_kind_read(refs/heads/main)", "assert agent-A committed kind=None/human + pending=true", "revert the working tree and re-read; assert pending=false" ] },
            "end_state": {
              "must_observe": [ "agent-A committed kind None/human", "pending=true on the uncommitted edit", "pending=false on a clean working tree", "all committed principals listed" ],
              "must_not_observe": [ "the working-tree kind reported as committed", "pending=false while the working-tree kind differs from committed" ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN kind_governance_base: caller rust-implementer holds [\"contents:write\"] only (NO administration:write); the working-tree permissions.toml captured byte-for-byte WHEN principal_kind_update(&repo, \"refs/heads/main\", \"agent-A\", \"agent\") runs with BUT_AGENT_HANDLE=rust-implementer THEN the call returns Err; classify_error(&err) .code == \"perm.denied\"; the message contains \"administration:write\"; AND the working-tree permissions.toml is byte-for-byte UNCHANGED (the admin gate ran BEFORE any write)",
      "verify": "cargo test -p but-api principal_kind_update_non_admin_denied_writes_nothing",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api principal_kind_update composing enforce_administration_write_gate + real but-authz + real gix",
        "negative_control": {
          "would_fail_if": [
            "the non-admin write succeeded (the admin gate is missing or runs after the write)",
            "classify_error did not yield perm.denied naming administration:write",
            "the working-tree permissions.toml changed on the denial path (a write before the gate)"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "kind_governance_base",
            "action": { "actor": "ci", "steps": [ "capture the working-tree permissions.toml bytes BEFORE", "temp_env BUT_AGENT_HANDLE=rust-implementer", "principal_kind_update(refs/heads/main, agent-A, agent)", "assert Err + classify_error perm.denied naming administration:write", "re-read the working-tree permissions.toml bytes AFTER" ] },
            "end_state": {
              "must_observe": [ "principal_kind_update returns Err", "classify_error .code == perm.denied", "the message contains administration:write", "the working-tree permissions.toml is byte-for-byte unchanged" ],
              "must_not_observe": [ "a successful non-admin kind write", "any byte change to the working-tree permissions.toml on the denial path" ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN governance_api_repo + a TestDesktopSession + the real Tauri mock runtime WHEN principal_kind_update is invoked on the bus with BUT_AGENT_HANDLE UNSET (desktop fleet-owner) to set agent-A kind=\"agent\", AND separately an agent-env path lacking administration:write attempts it, AND `pnpm build:sdk && pnpm format` runs THEN the fleet-owner invoke returns Ok and writes kind=\"agent\" WITHOUT BUT_AGENT_HANDLE (DesktopSessionState resolved the identity); the non-admin agent-env invoke is denied perm.denied naming administration:write with no write; principal_kind_read invokes and returns the kind list; AND packages/but-sdk/src/generated contains principal_kind_read/principal_kind_update + the DTOs and type-checks",
      "verify": "cargo test -p gitbutler-tauri mgmt_ipc_003 && pnpm build:sdk && pnpm format && grep -rq \"principal_kind_update\\|principalKindUpdate\" packages/but-sdk/src/generated",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real Tauri mock-runtime bus (tauri::test::get_ipc_response, mgmt_ipc_003 idiom) resolving DesktopSessionState + real but-api principal_kind_update + real but-authz; real pnpm build:sdk && pnpm format SDK regen + tsc",
        "negative_control": {
          "would_fail_if": [
            "the desktop principal_kind_update required BUT_AGENT_HANDLE (it is unset on desktop — the write would fail or fall through to an empty principal)",
            "the non-admin agent-env invoke succeeded (the admin gate is bypassed on the bus path)",
            "a per-command allow-governance_*/allow-principal_kind_* capability file were introduced (the forbidden-allow-file assertion fails)",
            "the command were unregistered (the real bus rejects it as 'not found')",
            "the SDK regen omitted principal_kind_read/principal_kind_update or the generated TS failed tsc"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "governance_api_repo",
            "action": { "actor": "ci", "steps": [ "governance_app(test_desktop_session) + governance_webview", "temp_env BUT_AGENT_HANDLE None → invoke principal_kind_update(agent-A, agent) on the bus; assert Ok + working-tree kind=\"agent\"", "temp_env BUT_AGENT_HANDLE Some(rust-implementer) → invoke the agent-env kind write; assert perm.denied naming administration:write + no write", "invoke principal_kind_read on the bus; assert the kind list returns", "run pnpm build:sdk && pnpm format; assert the generated SDK contains the commands + DTOs and type-checks" ] },
            "end_state": {
              "must_observe": [
                "the fleet-owner principal_kind_update invoke returns Ok and writes kind=\"agent\" with BUT_AGENT_HANDLE unset (DesktopSessionState resolved it)",
                "the non-admin agent-env invoke is denied perm.denied naming administration:write with no write",
                "principal_kind_read invokes on the bus and returns the kind list",
                "packages/but-sdk/src/generated contains principal_kind_read/principal_kind_update + the DTOs and the generated TS type-checks",
                "no allow-governance_*/allow-principal_kind_* capability file exists (core:default admits)"
              ],
              "must_not_observe": [
                "the desktop write failing for lack of BUT_AGENT_HANDLE",
                "the non-admin agent-env write succeeding",
                "a hand-written per-command allow-governance_* capability file",
                "the SDK missing the new commands/DTOs or failing tsc"
              ]
            }
          }
        ]
      }
    },
    { "id": "TC-1", "type": "test_criterion", "description": "after admin principal_kind_update(agent-A, agent) the working-tree permissions.toml declares kind=\"agent\" for agent-A", "verify": "cargo test -p but-api principal_kind_update_writes_worktree_inert_until_committed", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "after the update principal_kind_read reports agent-A committed kind None/human AND ref_id(main) before==after AND the caveat contains the ref-pin string", "verify": "cargo test -p but-api principal_kind_update_writes_worktree_inert_until_committed", "maps_to_ac": "AC-1" },
    { "id": "TC-3", "type": "test_criterion", "description": "a kind-only edit re-serializes admin/agent-A/rust-implementer grants+groups+kinds AND the [[group]] reviewers entry intact (lossless; only agent-A kind changed)", "verify": "cargo test -p but-api principal_kind_update_round_trips_full_schema_lossless", "maps_to_ac": "AC-2" },
    { "id": "TC-4", "type": "test_criterion", "description": "re-loading the rewritten file through but_authz::load_governance_config confirms every grant/group/member survives (no silent weakening on a kind edit)", "verify": "cargo test -p but-api principal_kind_update_round_trips_full_schema_lossless", "maps_to_ac": "AC-2" },
    { "id": "TC-5", "type": "test_criterion", "description": "principal_kind_read returns committed agent-A kind None/human with pending=true after the uncommitted edit; pending=false on a clean tree", "verify": "cargo test -p but-api principal_kind_read_returns_committed_kinds_with_pending_signal", "maps_to_ac": "AC-3" },
    { "id": "TC-6", "type": "test_criterion", "description": "a non-admin principal_kind_update returns Err with classify_error perm.denied naming administration:write", "verify": "cargo test -p but-api principal_kind_update_non_admin_denied_writes_nothing", "maps_to_ac": "AC-4" },
    { "id": "TC-7", "type": "test_criterion", "description": "after the denied non-admin principal_kind_update the working-tree permissions.toml is byte-for-byte unchanged (gate before write)", "verify": "cargo test -p but-api principal_kind_update_non_admin_denied_writes_nothing", "maps_to_ac": "AC-4" },
    { "id": "TC-8", "type": "test_criterion", "description": "the fleet-owner principal_kind_update bus invoke (BUT_AGENT_HANDLE unset) writes kind=\"agent\" via DesktopSessionState; the non-admin agent-env invoke is denied perm.denied naming administration:write with no write", "verify": "cargo test -p gitbutler-tauri mgmt_ipc_003", "maps_to_ac": "AC-5" },
    { "id": "TC-9", "type": "test_criterion", "description": "pnpm build:sdk && pnpm format regenerates packages/but-sdk/src/generated containing principal_kind_read/principal_kind_update + DTOs and the generated TS type-checks (no hand-edit)", "verify": "pnpm build:sdk && pnpm format && git diff --name-only packages/but-sdk/src/generated | grep -q . && grep -rq \"principal_kind_update\\|principalKindUpdate\" packages/but-sdk/src/generated", "maps_to_ac": "AC-5" },
    { "id": "TC-10", "type": "test_criterion", "description": "principal_kind_read AND principal_kind_update are in GOVERNANCE_COMMANDS + the gitbutler_governance_command_rows! macro; both invoke on the real bus; an unregistered probe is rejected ('not found')", "verify": "cargo test -p gitbutler-tauri mgmt_ipc_003", "maps_to_ac": "AC-5" },
    { "id": "TC-11", "type": "test_criterion", "description": "no allow-principal_kind_*/allow-governance_* capability file is introduced (core:default admits — the forbidden-allow-file assertion stays green)", "verify": "cargo test -p gitbutler-tauri mgmt_ipc_003", "maps_to_ac": "AC-5" }
  ]
}
-->
