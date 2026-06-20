# MGMT-IPC-005: Pending-until-committed read IPC contract (working-tree vs target-ref governed-config diff) — T-MGMT-035

> **Red-Hat Remediation (cycle 1):** resolves T5, T6, SEC5. `governance_pending` is now explicitly assigned an AUTHORITY (self / any authenticated desktop user; read-only, no mutation) in the CAPABILITY BOUNDARY + DONE WHEN, with the api-design.md command-table row addition carried as an UPSTREAM ADVISORY note (not a locked-PRD edit). AC-4's THEN is narrowed — a Rust cargo test cannot prove the renderer does not read `.gitbutler/*.toml`, so AC-4 now scopes to "the read mutates nothing (byte-unchanged) AND authorize() denies the uncommitted grant", referencing T-MGMT-027's renderer-file-read invariant as proven by MGMT-UI-003's grep, not this Rust test. AC-5 now references the `ConfigInvalid` carrier (the one rust-planner is adding to the IPC-002 downcast set) so the `remediation_hint` survives transport.

## What this does

Defines and registers the read IPC contract the MGMT renderer invokes to derive pending (○) governance state from a **real working-tree-vs-target-ref governed-config diff**: committed read via `but_authz::load_governance_config(repo, target_ref)`, working-tree read of `.gitbutler/{permissions,gates}.toml`, pending = the per-token difference. The renderer derives the ○ indicator and pending count from this server-computed payload — there is **no** renderer direct `.gitbutler/*.toml` read and **no** optimistic enforcement (the committed-at-target-ref config remains the effective truth until commit).

## Why

Sprint 06a · PRD UC-MGMT-06 (T-MGMT-035 working-tree-vs-target-ref; T-MGMT-027 no optimistic enforcement) · capability CAP-AUTHZ-01 + CAP-CONFIG-01. The read side feeding the pending banner (MGMT-UI-003/005); the write side stays the governed perm/group/branch_gates commands (MGMT-IPC-003).

## How to verify

PRIMARY **AC-1** — `cargo test -p gitbutler-tauri governance_pending_reports_uncommitted_grant_as_pending`: for a repo with an uncommitted working-tree grant, the read command marks that token `pending=true` while the committed effective set excludes it; `pendingCount >= 1`. Full gate set in the spec below.

## Scope

- `crates/gitbutler-tauri/src/main.rs` (MODIFY) — register the read command under the `main` capability.
- `crates/gitbutler-tauri/src/*` (the Tauri read-contract wrapper + DTO, if not co-located with a but-api read fn).
- `crates/gitbutler-tauri/tests/*` (NEW) — AC-1..AC-5 integration proofs (real working-tree-vs-target-ref diff over a but-testsupport gix repo).

> **Upstream advisory (do NOT edit the locked PRD).** `governance_pending` is a **13th** Tauri command not present in `.spec/prds/governance/10-technical-requirements/04-api-design.md`'s command-surface table (lines ~84-95, which lists the 12 perm/group/branch_gates/status commands). This task adds it as a **self-scoped, read-only** command (authority: self / any authenticated desktop user — it reads only the caller's view of the working-tree-vs-target-ref diff and never mutates config). The api-design.md table-row addition (`governance_pending` | working-tree-vs-target-ref governed-config diff | self (read-only) | derives the ○ pending indicator + count) is carried here as an UPSTREAM ADVISORY to reconcile in the PRD — it is NOT edited by this task.

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: MGMT-IPC-005 — Pending-until-committed read IPC contract (working-tree vs target-ref governed-config diff) — T-MGMT-035
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     M  (60 min)
AGENT:      implementer=tauri-implementer | reviewer=tauri-reviewer
PROPOSED-BY: tauri-planner
SPRINT:     ./SPRINT.md
PRD_REFS:   08-uc-mgmt.md UC-MGMT-06 (T-MGMT-035, T-MGMT-027), 10-ui-infrastructure.md (cross-cutting states), 04-api-design.md (governance_status_read; governance_pending is an UPSTREAM-ADVISORY 13th command — see task note)
CAPABILITIES: CAP-AUTHZ-01, CAP-CONFIG-01
PLATFORMS:  desktop

RUNTIME_COMMANDS:
  check: cargo check -p gitbutler-tauri
  test:  cargo test -p gitbutler-tauri governance_pending_reports_uncommitted_grant_as_pending
  lint:  cargo clippy -p gitbutler-tauri --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
A registered read Tauri command (governance_pending — self-scoped, read-only, no mutation) returns, for a
repo with an uncommitted working-tree grant, that token marked pending=true while the committed effective
set excludes it; for a clean working tree pendingCount=0; the diff is computed server-side against the real
target ref; the renderer needs no direct file read; a malformed working-tree config fails closed with the
ConfigInvalid carrier ({code:"config.invalid"} + a remediation_hint that survives transport, per MGMT-IPC-002).

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] Define the read IPC command as a real working-tree-vs-target-ref diff over the governed config:
  committed read via but_authz::load_governance_config(repo, target_ref); working-tree read of
  .gitbutler/{permissions,gates}.toml; pending = the per-token difference.
- [MUST] Return a serde::Serialize payload over but_api::json::Error with camelCase keys exposing per-token
  pending flags + the committed effective set, so the renderer derives the ○ indicator + pending count
  WITHOUT any direct .gitbutler/*.toml read.
- [MUST] Register the read command (governance_pending) in generate_handler! under the same main capability
  scope as MGMT-IPC-003; AUTHORITY = self / any authenticated desktop user (read-only, no mutation — a
  presentational diff of the caller's view; it never gates on administration:read/write because it mutates nothing).
- [MUST] On a malformed/unreadable working-tree governed config, fail closed by constructing the ConfigInvalid
  carrier (the carrier rust-planner is adding to the MGMT-IPC-002 downcast set) so {code:"config.invalid"} +
  remediation_hint survive transport over but_api::json::Error.
- [NEVER] NEVER let the renderer read .gitbutler/*.toml directly — pending state is derived ONLY from this
  server diff. (This Rust task cannot itself prove the renderer's absence-of-read; that invariant is proven by
  MGMT-UI-003's grep — see AC-4 and T-MGMT-027.)
- [NEVER] NEVER apply the working-tree (pending) config to authorization — pending is presentational;
  committed-at-target-ref remains the effective truth (no optimistic enforcement, T-MGMT-027).
- [NEVER] NEVER report working-tree config as committed, return a clean diff when the working tree differs, or skip the diff.
- [NEVER] NEVER author the inner read/diff logic in the renderer — it is a server-side read surfaced through the Tauri command.
- [STRICTLY] Compute the diff against the REAL target ref vs the REAL working tree using a but-testsupport gix repo — never mock.
- [STRICTLY] Keep this command read-only — it never mutates config (the write side stays the governed perm/group/branch_gates commands).

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: uncommitted working-tree grant reported PENDING; committed effective set unchanged
- [ ] AC-2: clean working tree reports pendingCount==0 (banner hidden)
- [ ] AC-3: uncommitted working-tree REVOKE reported pending (removal direction)
- [ ] AC-4: read is read-only (blobs byte-unchanged) AND authorize() denies the uncommitted grant (no optimistic enforcement); the renderer-never-reads-.gitbutler/*.toml invariant (T-MGMT-027) is proven by MGMT-UI-003's grep, NOT this Rust test
- [ ] AC-5: malformed/unreadable working-tree governed config fails closed via the ConfigInvalid carrier (config.invalid + remediation_hint surviving transport)
- [ ] governance_pending registered under main with AUTHORITY = self / any authenticated desktop user (read-only)
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: Uncommitted working-tree grant is reported PENDING; committed effective set unchanged
  GIVEN: pending_grant_repo (committed dev contents:write; working tree adds reviews:write, uncommitted)
  WHEN:  the read IPC command is invoked through the real Tauri command bus
  THEN:  reviews:write for dev is pending=true (inWorkingTree && !inCommitted) AND committedEffective for
         dev does NOT contain reviews:write; pendingCount >= 1
  TEST_TIER: integration   VERIFICATION_SERVICE: real Tauri bus + but-authz governed-config read over a but-testsupport gix repo
  VERIFY: cargo test -p gitbutler-tauri governance_pending_reports_uncommitted_grant_as_pending

AC-2: Clean working tree reports no pending changes
  GIVEN: clean_repo (working-tree .gitbutler/*.toml byte-identical to the committed target-ref blobs)
  WHEN:  the read IPC command is invoked
  THEN:  pendingCount == 0 and no token is marked pending (the banner is hidden, UC-MGMT-06)
  TEST_TIER: integration   VERIFICATION_SERVICE: real Tauri bus + but-authz read over a clean gix repo
  VERIFY: cargo test -p gitbutler-tauri governance_pending_clean_tree_reports_zero

AC-3: EDGE — uncommitted working-tree REVOKE is reported pending (removal direction)
  GIVEN: pending_revoke_repo (committed dev contents:write + reviews:write; working tree removes reviews:write, uncommitted)
  WHEN:  the read IPC command is invoked
  THEN:  reviews:write for dev is pending=true (inCommitted && !inWorkingTree) AND committedEffective for dev
         STILL contains reviews:write (committed truth unchanged until commit) — diff handles both directions
  TEST_TIER: integration   VERIFICATION_SERVICE: real Tauri bus + but-authz read over a gix repo
  VERIFY: cargo test -p gitbutler-tauri governance_pending_reports_uncommitted_revoke_as_pending

AC-4: EDGE — read is read-only AND authorize() ignores the uncommitted grant (no optimistic enforcement)
  GIVEN: pending_grant_repo with an uncommitted working-tree grant
  WHEN:  the read IPC command is invoked AND a parallel authorize() (effective_authority at target ref) is evaluated
  THEN:  the read does NOT mutate config (committed + working-tree blobs byte-unchanged after) AND the
         authorization decision is based ONLY on the committed target-ref config (the pending token is NOT
         optimistically authorized — authorize(dev, reviews:write) is DENIED). NOTE: the separate invariant that
         the renderer never reads .gitbutler/*.toml directly (T-MGMT-027) is NOT provable by this Rust test — it is
         proven by MGMT-UI-003's renderer grep; this AC scopes only to the server-side byte-unchanged + deny facts.
  TEST_TIER: integration   VERIFICATION_SERVICE: real Tauri bus + real but-authz authorize() over a gix repo
  VERIFY: cargo test -p gitbutler-tauri governance_pending_read_is_readonly_no_optimistic_enforcement

AC-5: ERROR — malformed/unreadable working-tree governed config fails closed via the ConfigInvalid carrier
  GIVEN: a repo whose working-tree .gitbutler/permissions.toml is malformed (unparseable)
  WHEN:  the read IPC command is invoked
  THEN:  it returns the structured {code:"config.invalid"} error over but_api::json::Error by constructing the
         ConfigInvalid carrier (the carrier rust-planner adds to the MGMT-IPC-002 downcast set) so the code AND a
         non-empty remediation_hint survive transport — rather than fabricating a clean/empty diff or treating the
         malformed working tree as no-change
  TEST_TIER: integration   VERIFICATION_SERVICE: real Tauri bus + but-authz config parser over a gix repo with a malformed blob
  VERIFY: cargo test -p gitbutler-tauri governance_pending_malformed_worktree_fails_closed

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): an uncommitted working-tree grant (reviews:write for dev) is returned pending=true while committedEffective excludes it
    VERIFY: cargo test -p gitbutler-tauri governance_pending_reports_uncommitted_grant_as_pending
- TC-2 (-> AC-2): a clean working tree returns pendingCount==0 and no pending tokens
    VERIFY: cargo test -p gitbutler-tauri governance_pending_clean_tree_reports_zero
- TC-3 (-> AC-3): an uncommitted working-tree revoke is returned pending=true while committedEffective still contains the token
    VERIFY: cargo test -p gitbutler-tauri governance_pending_reports_uncommitted_revoke_as_pending
- TC-4 (-> AC-4): the read mutates nothing (blobs byte-unchanged) and authorize() denies the uncommitted grant — pending is presentational; the renderer-no-direct-read invariant is MGMT-UI-003's grep, not this test
    VERIFY: cargo test -p gitbutler-tauri governance_pending_read_is_readonly_no_optimistic_enforcement
- TC-5 (-> AC-5): a malformed working-tree governed config returns the ConfigInvalid carrier ({code:config.invalid} + non-empty remediation_hint surviving transport), not a clean diff
    VERIFY: cargo test -p gitbutler-tauri governance_pending_malformed_worktree_fails_closed

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01, CAP-CONFIG-01
provides: a read-only Tauri command (governance_pending) returning the working-tree-vs-target-ref governed-config diff (per-token {pending, inCommitted, inWorkingTree}) + committed effective set + pendingCount
authority: self / any authenticated desktop user — read-only, NO mutation. governance_pending is a 13th command NOT in api-design.md's table; it gates on nothing beyond an authenticated desktop session because it mutates no config (presentational diff of the caller's view). The api-design.md table-row addition is an UPSTREAM ADVISORY (see task note), NOT a locked-PRD edit.
consumes: MGMT-IPC-001 (read-only governed-config helpers: load_governance_config(repo, target_ref), working-tree blob read, effective_authority); MGMT-IPC-002 (the ConfigInvalid carrier in the downcast set so config.invalid + remediation_hint survive transport); MGMT-IPC-003 (command registration + capability scope)
boundary_contracts:
  - Read IPC contract: async fn governance_pending(state, projectId) -> Result<GovernancePending, json::Error> where GovernancePending = { pendingCount, tokens:[{token, principalOrGroup, pending, inCommitted, inWorkingTree}], committedEffective }
  - Authority: self / any authenticated desktop user; read-only (no mutation, no admin gate)
  - Diff source-of-truth: committed = config parsed at the target ref (load_governance_config); working-tree = the .gitbutler/{permissions,gates}.toml; pending = the set difference; pending is presentational, NOT applied to authorization (T-MGMT-027)
  - Fail-closed contract: a malformed working-tree config returns the ConfigInvalid carrier (MGMT-IPC-002 downcast set) — {code:"config.invalid"} + remediation_hint surviving transport

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/gitbutler-tauri/src/main.rs (register governance_pending in generate_handler! under the main capability)
  - crates/gitbutler-tauri/src/* (the Tauri read-contract wrapper + its DTO, if not co-located with a but-api read fn)
  - crates/gitbutler-tauri/tests/* (integration tests for AC-1..AC-5)
writeProhibited:
  - crates/but-authz/src/** (authorization/config logic)
  - crates/but-api/src/** ConfigInvalid carrier definition (it is MGMT-IPC-002 / rust-planner territory — this task CONSUMES it, does not author it)
  - the renderer / apps/desktop/src/** (the renderer must NOT read .gitbutler/*.toml directly; UI is MGMT-UI-003/005; the no-direct-read invariant is proven by MGMT-UI-003's grep)
  - any config-mutating path (this is the read side; the write side stays the governed perm/group/branch_gates commands)
  - packages/but-sdk/src/generated/** (generated; regenerated by MGMT-IPC-004)

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. .spec/prds/governance/08-uc-mgmt.md (UC-MGMT-06 — pending-until-committed; T-MGMT-035 working-tree-vs-target-ref; T-MGMT-027 no optimistic enforcement)
2. .spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md (cross-cutting pending states + commit semantics B15)
3. .spec/prds/governance/10-technical-requirements/04-api-design.md (governance_status_read self-scoped read; config.invalid contract; NOTE governance_pending is an UPSTREAM-ADVISORY 13th command not in the table)
4. crates/but-authz/src/config.rs (load_governance_config / read_config_blob at a target ref — the committed side)
5. crates/but-authz/src/authorize.rs (effective_authority — the committed effective set; resolve_principal no_handle behavior at :71-72)
6. crates/but-api/tests/admin_write_guard.rs (write_worktree_permissions + the working-tree-must-not-widen-target-ref invariant to mirror in fixtures)
7. MGMT-IPC-002 task spec (the ConfigInvalid carrier added to the downcast set — AC-5 constructs/consumes it)
8. crates/AGENTS.md (read-only DryRun semantics; gix over git2; byte-preserving paths until the UI boundary)

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- cargo check -p gitbutler-tauri                                                                    -> Exit 0 (governance_pending registered under main)
- cargo test -p gitbutler-tauri governance_pending_reports_uncommitted_grant_as_pending             -> Exit 0
- cargo test -p gitbutler-tauri governance_pending_clean_tree_reports_zero                          -> Exit 0
- cargo test -p gitbutler-tauri governance_pending_reports_uncommitted_revoke_as_pending            -> Exit 0
- cargo test -p gitbutler-tauri governance_pending_read_is_readonly_no_optimistic_enforcement       -> Exit 0
- cargo test -p gitbutler-tauri governance_pending_malformed_worktree_fails_closed                  -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: Server-computed working-tree-vs-target-ref diff surfaced as a read-only, self-scoped Tauri command
  (governance_pending) — committed via load_governance_config(target_ref), working-tree via the .gitbutler/*.toml
  blob, pending = set difference, per-token marked — the renderer derives presentation only; malformed working
  tree fails closed via the MGMT-IPC-002 ConfigInvalid carrier.
pattern_source: crates/but-authz/src/config.rs + crates/but-api/tests/admin_write_guard.rs (working-tree-vs-target-ref invariant) + 08-uc-mgmt.md T-MGMT-035 + MGMT-IPC-002 (ConfigInvalid carrier)
anti_pattern: renderer reading .gitbutler/*.toml directly to compute pending (its absence is proven by MGMT-UI-003's
  grep, not this Rust test); optimistically applying the working-tree config to authorization; returning the
  working-tree config as committed; returning a clean diff for a malformed working tree (must fail closed via the
  ConfigInvalid carrier so config.invalid + remediation_hint survive transport); asserting a renderer-no-read claim
  this Rust test cannot prove.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: tauri-implementer — command-surface read contract routed through but-api -> Tauri (renderer never reads .gitbutler/*.toml)
reviewer: tauri-reviewer
coding_standards: crates/AGENTS.md (gix over git2; read-only; structured config.invalid over error-string matching), CLAUDE.md/RULES.md (fail closed), brain/docs/REQUIREMENT-TRACKING.md

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: MGMT-IPC-001 (read helpers), MGMT-IPC-002 (the ConfigInvalid carrier in the downcast set — AC-5), MGMT-IPC-003 (registration + capability scope)
Blocks:     MGMT-UI-003, MGMT-UI-005
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "MGMT-IPC-005",
  "proposed_by": "tauri-planner",
  "verification_policy": { "requires_tests": true, "requires_red_evidence": true, "requires_seeded_evidence": true },
  "fixtures": {
    "pending_grant_repo": { "description": "but-testsupport gix repo: committed .gitbutler/permissions.toml at refs/heads/main grants dev {contents:write}; the WORKING-TREE permissions.toml additionally grants dev reviews:write (uncommitted). Mirrors admin_write_guard.rs write_worktree_permissions.", "seed_method": "cli", "records": ["committed: dev = [contents:write]", "working tree: dev = [contents:write, reviews:write] (uncommitted)"] },
    "clean_repo": { "description": "but-testsupport gix repo where the working-tree .gitbutler/*.toml is byte-identical to the committed target-ref blobs (no diff).", "seed_method": "cli", "records": ["working tree == committed target-ref blobs", "no governance diff"] },
    "pending_revoke_repo": { "description": "but-testsupport gix repo where committed grants dev {contents:write, reviews:write} and the working tree removes reviews:write (uncommitted revoke).", "seed_method": "cli", "records": ["committed: dev = [contents:write, reviews:write]", "working tree: dev = [contents:write] (reviews:write removed, uncommitted)"] },
    "malformed_worktree_repo": { "description": "but-testsupport gix repo with a committed valid config and a malformed (unparseable) working-tree .gitbutler/permissions.toml.", "seed_method": "cli", "records": ["committed: valid config", "working tree: malformed/unparseable permissions.toml"] }
  },
  "requirements": [
    { "id": "AC-1", "type": "acceptance_criterion", "primary": true, "description": "GIVEN pending_grant_repo (committed dev contents:write; working tree adds reviews:write uncommitted) WHEN the read IPC command is invoked through the real bus THEN reviews:write for dev is pending=true AND committedEffective excludes it; pendingCount >= 1", "verify": "cargo test -p gitbutler-tauri governance_pending_reports_uncommitted_grant_as_pending", "scenario": { "id": "AC-1", "primary": true, "tier": "visible", "test_tier": "integration", "verification_service": "gitbutler-tauri + but-authz governed-config read", "negative_control": { "would_fail_if": ["the read returns the working-tree config as committed", "the diff is not computed (always-clean / hardcoded)", "the result is a static/mocked payload", "the committed set already contains the uncommitted token"] }, "evidence": { "artifact_type": "api_response", "required_capture": true }, "cases": [ { "start_ref": "pending_grant_repo", "action": { "actor": "renderer (read invoke)", "steps": ["invoke the read IPC command (governance_pending) through the real command bus against pending_grant_repo"] }, "end_state": { "must_observe": ["the token `\"reviews:write\"` for principal `\"dev\"` is marked `pending=true`", "`pendingCount >= 1`", "committedEffective for `dev` includes `\"contents:write\"`"], "must_not_observe": ["`reviews:write` present in dev committedEffective set", "`pendingCount == 0` (clean) despite the working-tree diff", "an empty tokens array"] } } ] } },
    { "id": "AC-2", "type": "acceptance_criterion", "primary": false, "description": "GIVEN clean_repo (working tree == committed blobs) WHEN the read command is invoked THEN pendingCount == 0 and no token is pending (banner hidden)", "verify": "cargo test -p gitbutler-tauri governance_pending_clean_tree_reports_zero", "scenario": { "id": "AC-2", "primary": false, "tier": "visible", "test_tier": "integration", "verification_service": "gitbutler-tauri + but-authz governed-config read", "negative_control": { "would_fail_if": ["the read always reports pending (diff not computed / hardcoded non-empty)", "it fabricates pending tokens when none differ (static)"] }, "evidence": { "artifact_type": "api_response", "required_capture": true }, "cases": [ { "start_ref": "clean_repo", "action": { "actor": "renderer (read invoke)", "steps": ["invoke the read IPC command (governance_pending) against clean_repo"] }, "end_state": { "must_observe": ["`pendingCount == 0`", "committedEffective reflects the committed config (>=1 authority for dev)"], "must_not_observe": ["`pendingCount > 0` for a clean tree", "any token marked `pending=true`"] } } ] } },
    { "id": "AC-3", "type": "acceptance_criterion", "primary": false, "description": "EDGE — GIVEN pending_revoke_repo (committed dev contents:write+reviews:write; working tree removes reviews:write uncommitted) WHEN the read command is invoked THEN reviews:write for dev is pending=true (removal) AND committedEffective still includes reviews:write (committed truth unchanged) — diff handles both directions", "verify": "cargo test -p gitbutler-tauri governance_pending_reports_uncommitted_revoke_as_pending", "scenario": { "id": "AC-3", "primary": false, "tier": "visible", "test_tier": "integration", "verification_service": "gitbutler-tauri + but-authz governed-config read", "negative_control": { "would_fail_if": ["the diff only handles additions (revoke not detected)", "the committed set is mutated to drop reviews:write before commit (static)"] }, "evidence": { "artifact_type": "api_response", "required_capture": true }, "cases": [ { "start_ref": "pending_revoke_repo", "action": { "actor": "renderer (read invoke)", "steps": ["invoke the read IPC command (governance_pending) against pending_revoke_repo"] }, "end_state": { "must_observe": ["`\"reviews:write\"` for `dev` marked `pending=true` (removal pending)", "committedEffective for `dev` STILL includes `\"reviews:write\"` (committed truth unchanged)", "`pendingCount >= 1`"], "must_not_observe": ["`reviews:write` absent from dev committedEffective before commit", "the revoke not surfaced (pendingCount 0 despite a working-tree removal)"] } } ] } },
    { "id": "AC-4", "type": "acceptance_criterion", "primary": false, "description": "EDGE — GIVEN pending_grant_repo WHEN the read command is invoked AND a parallel authorize() is evaluated THEN the read mutates nothing (committed + working-tree blobs byte-unchanged) AND authorize(dev, reviews:write) is DENIED (committed-only; pending not optimistically enforced). The renderer-never-reads-.gitbutler/*.toml invariant (T-MGMT-027) is NOT provable by this Rust test — it is proven by MGMT-UI-003's renderer grep; this AC scopes only to the server-side byte-unchanged + deny facts", "verify": "cargo test -p gitbutler-tauri governance_pending_read_is_readonly_no_optimistic_enforcement", "scenario": { "id": "AC-4", "primary": false, "tier": "visible", "test_tier": "integration", "verification_service": "gitbutler-tauri + but-authz authorize()", "negative_control": { "would_fail_if": ["the read mutates config (blobs change)", "the pending read returns a static/hardcoded payload", "authorize() honors the uncommitted working-tree grant (optimistic enforcement)"] }, "evidence": { "artifact_type": "api_response", "required_capture": true }, "cases": [ { "start_ref": "pending_grant_repo", "action": { "actor": "renderer (read) + authorization check", "steps": ["snapshot committed + working-tree blob bytes", "invoke the read IPC command (governance_pending) to derive pending reviews:write", "evaluate authorize(dev, reviews:write) against the committed target ref", "re-snapshot the blob bytes"] }, "end_state": { "must_observe": ["working-tree + committed `.gitbutler/*.toml` byte-unchanged (`==`) after the read (read-only)", "`authorize(dev, reviews:write)` is DENIED (uncommitted grant not optimistically enforced)", "the read payload's `pending` flags carry the ○ state server-side (renderer needs no file read — invariant proven by MGMT-UI-003's grep, not this test)"], "must_not_observe": ["config mutated by the read", "the read returning `0` pending despite the staged grant", "authorize() granting reviews:write from the uncommitted working-tree config"] } } ] } },
    { "id": "AC-5", "type": "acceptance_criterion", "primary": false, "description": "ERROR — GIVEN a malformed (unparseable) working-tree permissions.toml WHEN the read command is invoked THEN it returns the structured {code:config.invalid} over json::Error by constructing the ConfigInvalid carrier (the carrier rust-planner adds to the MGMT-IPC-002 downcast set) so the code AND a non-empty remediation_hint survive transport (fail closed), not a fabricated clean diff", "verify": "cargo test -p gitbutler-tauri governance_pending_malformed_worktree_fails_closed", "scenario": { "id": "AC-5", "primary": false, "tier": "visible", "test_tier": "integration", "verification_service": "gitbutler-tauri + but-authz config parser + MGMT-IPC-002 ConfigInvalid carrier", "negative_control": { "would_fail_if": ["a malformed working tree returns a clean diff (empty)", "an empty payload is returned", "an unstructured error instead of the fail-closed config.invalid contract", "the remediation_hint is dropped in transport (carrier not in the downcast set)", "the malformed working tree is treated as no-change (static)"] }, "evidence": { "artifact_type": "api_response", "required_capture": true }, "cases": [ { "start_ref": "malformed_worktree_repo", "action": { "actor": "renderer (read invoke)", "steps": ["invoke the read IPC command (governance_pending) against malformed_worktree_repo", "inspect the structured error after it crosses the json::Error transport boundary"] }, "end_state": { "must_observe": ["structured error `code == \"config.invalid\"` (the ConfigInvalid carrier)", "the error carries a non-empty `remediation_hint` for operator action that survived transport"], "must_not_observe": ["`pendingCount 0` / clean diff for a malformed working tree", "an unstructured or swallowed error", "an empty/absent `remediation_hint` (carrier not in the downcast set)", "the malformed working tree treated as no-change"] } } ] } },
    { "id": "TC-1", "type": "test_criterion", "description": "an uncommitted working-tree grant (reviews:write for dev) is returned pending=true while committedEffective excludes it", "verify": "cargo test -p gitbutler-tauri governance_pending_reports_uncommitted_grant_as_pending", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "a clean working tree returns pendingCount==0 and no pending tokens", "verify": "cargo test -p gitbutler-tauri governance_pending_clean_tree_reports_zero", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "an uncommitted working-tree revoke is returned pending=true while committedEffective still contains the token", "verify": "cargo test -p gitbutler-tauri governance_pending_reports_uncommitted_revoke_as_pending", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "description": "the read mutates nothing (blobs byte-unchanged) and authorize() denies the uncommitted grant — pending is presentational; the renderer-no-direct-read invariant is MGMT-UI-003's grep, not this test", "verify": "cargo test -p gitbutler-tauri governance_pending_read_is_readonly_no_optimistic_enforcement", "maps_to_ac": "AC-4" },
    { "id": "TC-5", "type": "test_criterion", "description": "a malformed working-tree governed config returns the ConfigInvalid carrier ({code:config.invalid} + non-empty remediation_hint surviving transport), not a clean diff", "verify": "cargo test -p gitbutler-tauri governance_pending_malformed_worktree_fails_closed", "maps_to_ac": "AC-5" }
  ]
}
-->
</details>
