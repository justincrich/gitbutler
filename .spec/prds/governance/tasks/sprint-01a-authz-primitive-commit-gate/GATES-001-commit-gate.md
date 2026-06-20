# GATES-001: Commit gate at the but-api `_with_authz` commit seam + CLI commit path (ref-aware, before the RepoExclusive guard, DryRun-enforced)

## What this does

Make GitButler's commit path enforcing at the ref-aware seam: in the but-api `_with_authz` commit wrapper (and the `but` CLI commit path), resolve the acting principal from `BUT_AGENT_HANDLE`, load the target-ref governance config, require `contents:write`, and reject a direct commit to a protected branch — all BEFORE the RepoExclusive guard is acquired and applying under DryRun — surfaced through the `but` commit CLI as the structured denial (exit 1). Governance is **opt-in by presence** of `.gitbutler/*.toml` at the target ref: a ref with NO committed governance config is ungoverned and the commit is ALLOWED, while the gate fails closed on an unknown principal / unset handle, on a protected branch, and on an incomplete (exactly one of the two files) or malformed config.

## Why

Sprint 01a · PRD UC-GATES-01, UC-AUTHZ-02, UC-AUTHZ-04 · capabilities CAP-AUTHZ-01, CAP-CONFIG-01. Part of the functional-permission governance walking skeleton (commit allow/deny through real `but-authz` + real git).

## How to verify

PRIMARY **AC-1** — `cargo test -p but-api commit_gate_feature_ok_protected_rejected` (integration). Full gate set in the spec below.

## Scope

- crates/but-api/src/commit/create.rs (MODIFY) — add the `_with_authz` gate at the TOP of the commit wrapper(s): resolve principal from BUT_AGENT_HANDLE, load target-ref config from `relative_to`'s `RelativeTo::Reference(name)`, authorize contents:write, check branch protection — BEFORE `ctx.exclusive_worktree_access()` / `guard.write_permission()`; honor DryRun (gate runs, persistence suppressed)
- crates/but-api/src/commit/gate.rs (NEW) — the gate helper (kept small; calls but-authz; resolves the target ref from RelativeTo)
- crates/but-api/Cargo.toml (MODIFY) — add `but-authz` workspace dep
- crates/but-api/tests/commit_gate.rs (NEW) — integration tests against the real commit wrapper + git
- crates/but/src/command/legacy/commit2.rs (MODIFY) — call the gate after target-ref resolution in resolve()/before run(); surface the Denial as the structured exit-1 contract at the CLI boundary
- crates/but/tests/but/command/commit_gate.rs (NEW) — CLI snapbox end-to-end denial/allow

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: GATES-001 - Commit gate at the but-api `_with_authz` commit seam + CLI commit path (ref-aware, before the RepoExclusive guard, DryRun-enforced)
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     L  (300 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-GATES-01, UC-AUTHZ-02, UC-AUTHZ-04
CAPABILITIES: CAP-AUTHZ-01, CAP-CONFIG-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-api commit_gate_feature_ok_protected_rejected
  check: cargo check -p but-authz --all-targets
  lint:  cargo clippy --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Integration tests are green against the real but-api commit wrapper + real git, and the `but` CLI via snapbox: a `contents:write` principal's commit to a feature branch lands (ref advances to the new sha); a `contents:read`-only principal's commit is denied `perm.denied` naming `contents:write`; a direct commit to protected `main` by a `contents:write` principal is denied `branch.protected` naming `main`; an unset OR empty `BUT_AGENT_HANDLE` is rejected with a structured perm.denied; a handle that resolves but is absent from the committed permissions.toml is rejected perm.denied; a working-tree edit unprotecting `main` does NOT let a `main` commit through (still `branch.protected`); a feature head whose committed gates.toml unprotects main does NOT let a `main` commit through; a malformed target-ref `gates.toml` denies `config.invalid`; a target ref with NO committed governance config is ungoverned and the commit is ALLOWED (opt-in by presence — governance activates only once `.gitbutler/*.toml` is committed); a PARTIAL config (exactly one of permissions.toml / gates.toml present) denies `config.invalid` (fail closed on incomplete governance); a denied DryRun commit returns the contract and persists nothing; AND a paired allowed DryRun commit to `feat` returns Ok/preview, leaves `feat` HEAD unchanged, and persists no object.

Opt-in is sound because landing on a governed trunk is mediated by the merge gate (Sprint 01b), which reads the *trunk's* target-ref config. A feature branch that self-ungates (commits a deletion of its governance files) only affects that branch; merging it into a governed trunk still requires `merge` authority + a distinct review. A repo whose trunk never committed governance is ungoverned by the owner's deliberate choice. A stronger anti-config-deletion guarantee (an explicit "governance enabled" enablement signal) is a noted future Sprint-04 hardening candidate, not taken here.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST place the gate at the REF-AWARE but-api commit seam — the `_with_authz` wrapper around `commit_create_only` / `commit_create` in crates/but-api/src/commit/create.rs, which receives `relative_to: RelativeTo` where `RelativeTo::Reference(gix::refs::FullName)` carries the TARGET BRANCH NAME needed for branch protection — AND the `but` CLI commit path in crates/but/src/command/legacy/commit2.rs (which resolves the target ref via `route_commit_operation` → `CommitRelativeToTarget::BranchTip{name}` / `RelativeTo::Reference(name)`). Cover BOTH callers; do NOT wire it only to one (e.g. but-action).
- [MUST] MUST run `authorize()` BEFORE the `RepoExclusive` worktree guard is acquired — per 04-api-design.md the mandated ordering is `authorize()` → `with_authz()` → acquire guard → run impl. In create.rs the guard is taken via `ctx.exclusive_worktree_access()` / `guard.write_permission()` (lines 32/41/107); the gate runs at the TOP of the wrapper, before that. In commit2.rs the target ref is resolved in `resolve()` and the guard's `write_permission()` is taken in `run()` (line 138) — authorize after target resolution, before `run()`.
- [MUST] MUST run the gate EVEN under DryRun — a dry-run commit is still permission-checked and a denial still returns the contract + exit 1; DryRun only suppresses persisting refs/objects/oplog (04-api-design.md; CAP-AUTHZ-01).
- [MUST] MUST read branch protection ONLY from the TARGET-REF `.gitbutler/gates.toml` blob via AUTHZ-002's loader — a working-tree `gates.toml` edit can NEVER unprotect the branch (CAP-CONFIG-01).
- [MUST] MUST fail closed: unknown principal / unset or empty handle → perm.denied; unreadable/malformed target-ref config → config.invalid; a PARTIAL (exactly one of the two files present) target-ref config → config.invalid (fail closed on incomplete/invalid governance); a `contents:write`-lacking principal → perm.denied; a direct commit to a protected branch → branch.protected — each exit 1. A ref with NO committed governance config is ungoverned (commit ALLOWED — opt-in by presence), NOT config.invalid.
- [MUST] MUST keep `commit_engine::create_commit` gate-free (or only a thin defensive assertion): its `Destination` enum carries NO reliable target-ref/branch name (`NewCommit.stack_segment` is OPTIONAL and `AmendCommit` has none), so branch-protection CANNOT be evaluated there — that is exactly why the gate lives at the ref-aware but-api/CLI seam (02-system-components.md marks the create_commit gate as the 'Alternative').
- [NEVER] NEVER use a hardcoded protected-branch list — protection comes from committed gates.toml (UC-GATES-01 AC-6).
- [NEVER] NEVER overload GitButler's repo-access `Permission`/`RepoExclusive` lock as the authorization carrier — authorization is the orthogonal `Authority` axis, evaluated BEFORE the worktree guard is taken (02-system-components.md; lock discipline in RULES.md: don't acquire permission-helpers while holding a guard).
- [NEVER] NEVER persist anything on a denied commit — a denial returns before the guard/write happens, and a denied DryRun persists nothing either.
- [NEVER] NEVER evaluate branch protection at `commit_engine::create_commit` — the Destination there cannot name the protected branch; doing so would silently fail to protect `main`.
- [STRICTLY] STRICTLY surface the denial as the structured contract (`{error:{code,message}}`, exit 1) at the `but` CLI boundary (commit2.rs), mapping branch.protected/perm.denied/config.invalid; the `branch.protected` message names the branch and tells the principal to land via a reviewed merge.
- [STRICTLY] STRICTLY add only the `Code` variants a consumer surfaces, and prefer a but-authz-owned code string for the new gate codes rather than mutating but-error's `Code` enum unless the frontend consumes it (the enum doc forbids unused variants).
- [STRICTLY] STRICTLY resolve the target ref from `RelativeTo::Reference(name)` (the only RelativeTo variant carrying a ref name); for `RelativeTo::Commit(id)` there is no protected-branch name to check, so contents:write authorization still runs but branch-protection is N/A (the protected-main path drives `BranchTip{name=main}`/`Reference(main)`, which IS ref-named).

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: Feature commit by contents:write lands; protected-branch commit is rejected branch.protected [PRIMARY]
- [ ] AC-2: Commit lacking contents:write denied perm.denied; unset/empty/ghost handle rejected
- [ ] AC-3: A working-tree OR feature-head gates.toml edit cannot unprotect the branch (ref-pin)
- [ ] AC-4: Malformed OR partial (incomplete) target-ref config fail closed config.invalid; an ABSENT (no governance) config is ungoverned (commit allowed); denial fires under DryRun and persists nothing; an allowed DryRun previews and persists nothing
- [ ] All verification gates pass; only write_allowed files modified (git diff --name-only)

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: Feature commit by contents:write lands; protected-branch commit is rejected branch.protected [PRIMARY] [PRIMARY]
  GIVEN: fixture `gated_repo` with BUT_AGENT_HANDLE=dev (holds contents:write)
  WHEN:  the gated commit path commits to feature `feat`, then attempts a direct commit to protected `main` (the `but commit --branch main` path that resolves to `CommitRelativeToTarget::BranchTip{name=main}` / `RelativeTo::Reference(main)`)
  THEN:  the feature commit succeeds and `feat` advances to the new sha; the `main` commit is denied with `error.code=="branch.protected"` naming `main`, exit 1, and `main` does NOT advance
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api commit wrapper + real git (but-testsupport)
  VERIFY: cargo test -p but-api commit_gate_feature_ok_protected_rejected
  SCENARIO (tier=visible, test_tier=integration):
    NEGATIVE_CONTROL would fail if: the gate is a no-op stub so the main commit lands (ref advances); branch protection is read from the working tree, not the target ref; the commit path never calls authorize; the gate runs AFTER the RepoExclusive guard is taken (lock-ordering violation)
    EVIDENCE: stdout (required_capture=True)
    case[0] (cli_user): BUT_AGENT_HANDLE=dev: `but commit --branch feat -m x` (commit the pending change on feature branch feat)
      MUST_OBSERVE:     ['commit lands on `feat` (process exits `0`)', '`feat` HEAD sha `!=` the seeded base sha']
      MUST_NOT_OBSERVE: ['`branch.protected`', 'exit `1`', 'no commit created']
    case[1] (cli_user): BUT_AGENT_HANDLE=dev: `but commit --branch main -m x` (direct commit to protected main → BranchTip{name=main})
      MUST_OBSERVE:     ['`error.code == "branch.protected"`', 'message names `"main"`', 'process exits `1`', '`main` HEAD sha `==` the seeded base sha']
      MUST_NOT_OBSERVE: ['`main` HEAD sha advanced', 'commit landed on `main`', 'exit `0`']

AC-2: Commit lacking contents:write denied perm.denied; unset/empty/ghost handle rejected
  GIVEN: fixtures `gated_repo` and `gated_repo_ghost_handle`
  WHEN:  a commit is attempted on `feat` with BUT_AGENT_HANDLE=ro (contents:read only), again with BUT_AGENT_HANDLE unset, again with BUT_AGENT_HANDLE="" (empty), and again with BUT_AGENT_HANDLE=ghost (resolves in env but absent from committed permissions.toml)
  THEN:  the `ro` commit is denied `error.code=="perm.denied"` naming `contents:write` (exit 1, edits never reach a ref); the unset-handle, empty-handle, and ghost-handle commits are each rejected with a STRUCTURED `error.code=="perm.denied"` (no anonymous/default principal, exit 1, feat HEAD == base)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api commit wrapper + real git
  VERIFY: cargo test -p but-api commit_gate_readonly_and_bad_handle_denied
  SCENARIO (tier=visible, test_tier=integration):
    NEGATIVE_CONTROL would fail if: the ro commit lands because the gate omits the contents:write check; an unset/empty handle defaults to an allowed principal; a ghost handle that resolves in env but is absent from config is granted an implicit identity and the commit lands; the denial is not perm.denied / does not name contents:write; the no-handle case exits 1 only by panicking rather than returning the structured perm.denied contract
    EVIDENCE: stdout (required_capture=True)
    case[0] (cli_user): BUT_AGENT_HANDLE=ro: `but commit --branch feat -m x`
      MUST_OBSERVE:     ['`error.code == "perm.denied"`', 'message names `"contents:write"`', 'process exits `1`', '`feat` HEAD sha `==` the seeded base sha']
      MUST_NOT_OBSERVE: ['commit landed', '`feat` HEAD sha advanced', 'exit `0`']
    case[1] (cli_user): BUT_AGENT_HANDLE unset: `but commit --branch feat -m x`
      MUST_OBSERVE:     ['`error.code == "perm.denied"` (structured denial, not a bare panic)', 'process exits `1` with no principal bound', '`feat` HEAD sha `==` the seeded base sha']
      MUST_NOT_OBSERVE: ['commit landed', 'exit `0`', 'default principal']
    case[2] (cli_user): BUT_AGENT_HANDLE="" (empty string): `but commit --branch feat -m x`
      MUST_OBSERVE:     ['`error.code == "perm.denied"` (empty handle rejected same as unset)', 'process exits `1`', '`feat` HEAD sha `==` the seeded base sha']
      MUST_NOT_OBSERVE: ['commit landed', 'exit `0`', 'empty handle accepted as a principal']
    case[3] (cli_user): BUT_AGENT_HANDLE=ghost (resolves in env, absent from committed permissions.toml): `but commit --branch feat -m x` (cite T-AUTHZ-027)
      MUST_OBSERVE:     ['`error.code == "perm.denied"` (unknown principal — handle absent from committed config)', 'process exits `1`', '`feat` HEAD sha `==` the seeded base sha']
      MUST_NOT_OBSERVE: ['commit landed', 'exit `0`', 'ghost granted an implicit identity']

AC-3: A working-tree OR feature-head gates.toml edit cannot unprotect the branch (ref-pin)
  GIVEN: fixture `gated_repo` after the WORKING-TREE `.gitbutler/gates.toml` is edited to set `main` unprotected (NOT committed), and separately fixture `gated_repo_feature_head_unprotects` (feat has a COMMITTED gates.toml unprotecting main), BUT_AGENT_HANDLE=dev
  WHEN:  a direct commit to `main` (BranchTip{name=main}) is attempted
  THEN:  the commit is still denied `branch.protected` in both cases — neither the uncommitted working-tree edit nor the feature head's committed unprotecting gates.toml weakens the gate (protection read from the TARGET ref `main`)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api commit wrapper + real git
  VERIFY: cargo test -p but-api commit_gate_edit_cannot_unprotect
  SCENARIO (tier=holdout, test_tier=integration):
    NEGATIVE_CONTROL would fail if: the gate reads working-tree gates.toml so the edit unprotects main and the commit lands; protection is resolved from the feature head instead of the target ref (the feat committed gates.toml wins and the commit lands); protection is resolved from HEAD instead of the supplied target ref (a no-op pin)
    EVIDENCE: stdout (required_capture=True)
    case[0] (cli_user): edit working-tree .gitbutler/gates.toml so main protected=false (do NOT commit); BUT_AGENT_HANDLE=dev: `but commit --branch main -m x` (attempt direct commit to main)
      MUST_OBSERVE:     ['`error.code == "branch.protected"`', 'process exits `1`', '`main` HEAD sha `==` the seeded base sha']
      MUST_NOT_OBSERVE: ['`main` HEAD sha advanced', 'commit landed on `main`', 'exit `0`']
    case[1] (cli_user): on feat (whose COMMITTED gates.toml unprotects main), BUT_AGENT_HANDLE=dev: `but commit --branch main -m x`
      MUST_OBSERVE:     ['`error.code == "branch.protected"` (protection read from target ref main, not the feature head)', 'process exits `1`', '`main` HEAD sha `==` the seeded base sha']
      MUST_NOT_OBSERVE: ['`main` HEAD sha advanced', 'commit landed on `main`', 'exit `0`']

AC-4: Malformed OR partial config fail closed config.invalid; an absent config is ungoverned (commit allowed); denial fires under DryRun and persists nothing; an allowed DryRun previews and persists nothing
  GIVEN: fixtures `gated_repo_malformed` (broken target-ref gates.toml), `gated_repo_partial_config` (exactly one of the two governance files committed at the target ref), `gated_repo_no_config` (NO committed governance config), and `gated_repo` for the DryRun cases
  WHEN:  a commit is attempted against the malformed config, against the partial (one-file) config, and against the no-config (ungoverned) repo; a DryRun commit to protected `main` is attempted by dev; and a DryRun commit to authorized `feat` is attempted by dev
  THEN:  the malformed-config commit denies `error.code=="config.invalid"` (not a skip); the partial-config commit denies `error.code=="config.invalid"` (fail closed on incomplete governance — the loader requires a complete config once governance is opted-in); the no-config commit is ALLOWED (ungoverned — opt-in by presence, the gate's discriminator `but_authz::governance_present` reports neither `.gitbutler/*.toml` present, so the gate returns Ok and the commit lands); the DryRun commit to `main` still returns `branch.protected` + exit 1 and persists no ref/object; the DryRun commit to `feat` returns Ok/preview, leaves `feat` HEAD unchanged, and persists no object

  NOTE: The gate's opt-in discriminator is `but_authz::governance_present(repo, target_ref)` (the single source of the `.gitbutler/*.toml` path truth — ≥1 of the two governance files committed at the target ref means governed). If NEITHER file is present the ref is ungoverned and the gate returns Ok (commit allowed); if ≥1 is present the gate invokes AUTHZ-002's loader, which fails closed config.invalid on a malformed/unreadable/incomplete (one-file) config.
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api commit wrapper + real git
  VERIFY: cargo test -p but-api commit_gate
  SCENARIO (tier=holdout, test_tier=integration):
    NEGATIVE_CONTROL would fail if: malformed config is skipped (treated as an empty config) and the commit proceeds (fail-open); a partial/incomplete config (one file present) is treated as ungoverned and the commit proceeds (must instead be config.invalid); an ungoverned ref (NO governance config) is treated as config.invalid and the commit is denied (it must be ALLOWED — opt-in by presence); DryRun bypasses authorization so the protected-branch denial does not fire; a denied DryRun still writes a ref/object; an allowed DryRun advances feat HEAD or writes a persisted object (DryRun must only preview)
    EVIDENCE: stdout (required_capture=True)
    case[0] (cli_user): BUT_AGENT_HANDLE=dev: `but commit --branch feat -m x` with malformed target-ref gates.toml
      MUST_OBSERVE:     ['`error.code == "config.invalid"`', 'process exits `1`']
      MUST_NOT_OBSERVE: ['commit landed', 'exit `0`', 'skipped check']
    case[1] (cli_user): BUT_AGENT_HANDLE=dev: `but commit --branch feat -m x` with a PARTIAL config (exactly one of permissions.toml / gates.toml committed at the target ref)
      MUST_OBSERVE:     ['`error.code == "config.invalid"` (incomplete governance — one file present, fail closed)', 'process exits `1`']
      MUST_NOT_OBSERVE: ['commit landed', 'exit `0`', 'partial config treated as ungoverned (commit proceeds)']
    case[2] (cli_user): BUT_AGENT_HANDLE=dev: `but commit --branch feat -m x` with NO governance config committed at the target ref (ungoverned — opt-in by presence; governance is inactive until `.gitbutler/*.toml` is committed)
      MUST_OBSERVE:     ['process exits `0` (ungoverned ref → commit allowed)', '`feat` HEAD sha `!=` the seeded base sha (commit landed)']
      MUST_NOT_OBSERVE: ['`error.code == "config.invalid"`', 'exit `1`', 'no commit created']
    case[3] (cli_user): BUT_AGENT_HANDLE=dev: DryRun commit to protected main (`but commit --branch main --dry-run -m x`)
      MUST_OBSERVE:     ['`error.code == "branch.protected"`', 'process exits `1`', '`main` HEAD sha `==` base AND no commit object was persisted for this attempt']
      MUST_NOT_OBSERVE: ['exit `0`', '`main` HEAD sha advanced', 'a persisted ref/object from the denied dry run']
    case[4] (cli_user): BUT_AGENT_HANDLE=dev: DryRun commit to authorized feat (`but commit --branch feat --dry-run -m x`) — paired ALLOWED DryRun (cite RF-008)
      MUST_OBSERVE:     ['process exits `0` with a preview/Ok outcome (authorization passed under DryRun)', '`feat` HEAD sha `==` the seeded base sha (DryRun persisted nothing)']
      MUST_NOT_OBSERVE: ['`feat` HEAD sha advanced', 'a persisted commit object from the dry run', 'exit `1`', 'no new commit object persisted (0 objects written by the dry run)']

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, happy_path): dev commit lands on feat (ref advances); dev direct commit to protected main is branch.protected exit 1, main unchanged (T-GATES-001/002/003/007)
    VERIFY: cargo test -p but-api commit_gate_feature_ok_protected_rejected
- TC-2 (-> AC-2, error): ro commit denied perm.denied naming contents:write; unset/empty/ghost handle each rejected with structured perm.denied exit 1 (T-GATES-004/005, T-AUTHZ-011/027/028)
    VERIFY: cargo test -p but-api commit_gate_readonly_and_bad_handle_denied
- TC-3 (-> AC-3, edge): Neither a working-tree gates.toml edit NOR a feature-head committed gates.toml unprotects main; commit still branch.protected (T-GATES-019 ref-pin, T-GATES-005 target-ref governs)
    VERIFY: cargo test -p but-api commit_gate_edit_cannot_unprotect
- TC-4 (-> AC-4, error): Malformed OR partial (one-file) target-ref config -> config.invalid (not skip / not default-allow); an absent (ungoverned) config -> commit allowed (opt-in by presence); DryRun protected-main commit still branch.protected + persists nothing; allowed DryRun to feat previews + persists nothing (T-AUTHZ-029, RF-008, DryRun-enforced CAP-AUTHZ-01)
    VERIFY: cargo test -p but-api commit_gate
- TC-5 (-> AC-1, edge): Branch protection is resolved from the TARGET-REF committed gates.toml, not a hardcoded list and not the working tree/feature head (T-GATES-006, T-GATES-005)
    VERIFY: cargo test -p but-api commit_gate_feature_ok_protected_rejected
- TC-6 (-> AC-1, edge): Authorization runs BEFORE the RepoExclusive guard is acquired (lock ordering: authorize() -> with_authz() -> acquire guard) — a denial returns before any worktree lock/write (04-api-design.md ordering)
    VERIFY: cargo test -p but-api commit_gate_feature_ok_protected_rejected

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01, CAP-CONFIG-01
provides: commit gate at the but-api `_with_authz` commit seam (contents:write + branch protection, fail-closed, DryRun-enforced) covering the Tauri/but-api commit path AND the `but` CLI commit path; branch.protected / perm.denied / config.invalid denial codes surfaced at the commit boundary + exit 1; opt-in-by-presence discrimination via `but_authz::governance_present` (ungoverned ref → commit allowed)
consumes: but_authz::authorize (AUTHZ-003); but_authz::resolve_principal (AUTHZ-003); but_authz::config::load_governance_config (AUTHZ-002); but_authz::governance_present (the `.gitbutler/*.toml`-presence discriminator); but_authz::Authority::ContentsWrite (AUTHZ-001)
boundary_contracts:
  - CAP-AUTHZ-01: the commit action resolves BUT_AGENT_HANDLE→Principal and authorizes contents:write at the ref-aware but-api/CLI seam BEFORE the RepoExclusive guard is acquired, EVEN under DryRun; CAP-CONFIG-01: branch protection is read ONLY from the target-ref gates.toml blob, so a working-tree edit cannot unprotect the branch

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-api/src/commit/create.rs (MODIFY) — add the `_with_authz` gate at the TOP of the commit wrapper(s): resolve principal from BUT_AGENT_HANDLE, load target-ref config from `relative_to`'s `RelativeTo::Reference(name)`, authorize contents:write, check branch protection — BEFORE `ctx.exclusive_worktree_access()` / `guard.write_permission()`; honor DryRun (gate runs, persistence suppressed)
  - crates/but-api/src/commit/gate.rs (NEW) — the gate helper (kept small; calls but-authz; resolves the target ref from RelativeTo)
  - crates/but-api/Cargo.toml (MODIFY) — add `but-authz` workspace dep
  - crates/but-api/tests/commit_gate.rs (NEW) — integration tests against the real commit wrapper + git
  - crates/but/src/command/legacy/commit2.rs (MODIFY) — call the gate after target-ref resolution in resolve()/before run(); surface the Denial as the structured exit-1 contract at the CLI boundary
  - crates/but/tests/but/command/commit_gate.rs (NEW) — CLI snapbox end-to-end denial/allow
writeProhibited:
  - crates/but-authz/** — consume authorize/load_governance_config/governance_present/Authority; do not modify the primitive here
  - crates/but-workspace/src/commit_engine/mod.rs — do NOT add branch-protection here; its Destination cannot name the protected branch (keep it gate-free; only a thin defensive assertion is permissible if a reviewer requires it)
  - crates/but-error/src/lib.rs — do not add Code variants unless the desktop frontend consumes the new gate code; prefer a but-authz-owned code string (the enum doc forbids unused variants)
  - crates/but-api/src/legacy/forge.rs — the MERGE gate is a later sprint, not this task
  - any `gitbutler-*` crate beyond what the action boundary strictly requires (crates/AGENTS.md)
  - Any file not listed in write_allowed

--------------------------------------------------------------------------------
OUT OF SCOPE
--------------------------------------------------------------------------------
  - T-GATES-016 (mechanism-agnostic enforcement across all commit mechanisms) is DEFERRED to Sprint 04. The narrow-waist seam placement makes the authorization decision mechanism-independent BY CONSTRUCTION (every ref-named commit funnels through the but-api/CLI seam), but proving cross-mechanism PARITY is a Sprint-04 concern.
  - T-GATES-017 (worktree / `but-worktrees::integrate` + `but-workspace::branch::apply` commit paths) is DEFERRED to Sprint 04. This task gates the but-api commit wrappers + the `but` CLI commit path; the worktree/normal-git PARITY proof lands in Sprint 04. Honest note: the seam does not architecturally preclude the worktree path, but it is NOT proven gated here.
  - A stronger anti-config-deletion guarantee (an explicit "governance enabled" enablement signal that prevents a branch from self-ungating by deleting its committed `.gitbutler/*.toml`) is DEFERRED to Sprint 04 as a hardening candidate. This sprint adopts opt-in-by-presence: an ungoverned ref is ungoverned by the owner's deliberate choice, and self-ungating a feature branch is contained by the merge gate (Sprint 01b) that reads the trunk's config.

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-api/src/commit/create.rs (lines 21-133)
   Focus: PRIMARY PATTERN + THE SEAM — `commit_create_only` (line 23) and `commit_create` (line 100) receive `relative_to: RelativeTo` (carries `RelativeTo::Reference(FullName)` = the target ref). The RepoExclusive guard is taken at line 32 (`ctx.exclusive_worktree_access()` / `guard.write_permission()`) and line 41/107. WRAP these with `_with_authz`: resolve principal + load target-ref config + authorize contents:write + branch protection AT THE TOP, BEFORE `exclusive_worktree_access()`. Mirror the `_with_perm` composition shape but NEVER reuse the `Permission` lock as the authority carrier; DryRun must NOT skip the gate.
2. crates/but/src/command/legacy/commit2.rs (lines 119-236 + 333-432)
   Focus: PRIMARY PATTERN (CLI seam) — `commit()` (line 119) takes the guard via `ctx.exclusive_worktree_access()` then `resolve()` runs `route_commit_operation` which yields `CommitRelativeToTarget::BranchTip{name}` (line 355, the protected-main path via `CommitOperationTargetIsh::Branch`) / `RelativeTo::Reference(name)`. `run()` takes `guard.write_permission()` at line 138. The gate runs AFTER target-ref resolution in resolve(), BEFORE run() — so authorize precedes the write lock. Surface the Denial as the structured exit-1 contract here.
3. crates/but-workspace/src/commit_engine/mod.rs (lines 23-44 + 123-150)
   Focus: WHY NOT HERE — `create_commit`'s `Destination` enum (lines 23-44): `NewCommit.stack_segment` is `Option<StackSegmentId>` (optional, virtual-branch disambiguation only) and `AmendCommit` carries NO ref. So this layer CANNOT reliably name the protected branch — confirm and keep it gate-free (02-system-components.md marks this the 'Alternative'). The gate's home is the ref-aware but-api/CLI seam above.
4. crates/but/tests/but/utils.rs (lines 120-142)
   Focus: The CLI test harness `env.but("commit --branch feat -m x")` returns a snapbox Command supporting `.env("BUT_AGENT_HANDLE", "dev")`, `.assert()`, `.stdout_eq`/`.stderr_eq` + `[..]`/`...` wildcards and exit-code assertions; drive the gate end-to-end. Use `Sandbox` + `but_testsupport::writable_scenario`/`invoke_bash`, NOT std::process::Command::new("git").
5. crates/but-testsupport/src/lib.rs (lines 432-441 + 71-97)
   Focus: `writable_scenario(name) -> (gix::Repository, tempfile::TempDir)` (a FUNCTION) + `invoke_bash(script, &repo)` to seed a real repo, commit `.gitbutler/*.toml` at refs/heads/main, branch to feat, and stage a pending change. Keep `_tmp` alive; assert refs advanced via `repo.find_reference(..).peel_to_id()`.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- Commit-gate integration tests pass (but-api seam): `cargo test -p but-api commit_gate`  -> Exit 0; AC-1..4 green
- CLI end-to-end denial/allow passes: `cargo test -p but commit_gate`  -> Exit 0; snapbox stderr_eq + exit codes match (update with SNAPSHOTS=overwrite if intended)
- Crates compile: `cargo check -p but-api -p but --all-targets`  -> Exit 0
- Gate does not read working-tree config: `! grep -rEn 'workdir|std::fs::read|read_to_string' crates/but-api/src/commit/gate.rs`  -> No matches — protection read only from the target-ref blob (via but-authz loader)
- No Permission-lock overload as authority carrier: `! grep -rEn 'write_permission\(|RepoExclusive' crates/but-api/src/commit/gate.rs`  -> No matches — the gate uses but-authz Authority, not the repo lock; the guard is acquired by the wrapper AFTER the gate, not inside it
- create_commit stays gate-free: `! grep -rEn 'load_governance_config|branch.protected|but_authz::authorize' crates/but-workspace/src/commit_engine/mod.rs`  -> No matches — the gate is at the but-api/CLI seam, not the engine (Destination cannot name the protected branch)
- Clippy clean: `cargo clippy -p but-api -p but --all-targets`  -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references: crates/but-api/src/commit/create.rs:21 (the ref-aware commit wrapper seam; acquire-near-wrapper composition + DryRun threading; guard at line 32/41/107); crates/but/src/command/legacy/commit2.rs:119 (CLI commit() — resolve() target-ref resolution before run()'s write_permission at line 138); crates/but-workspace/src/commit_engine/mod.rs:23 (Destination enum — confirmed to carry NO reliable target ref; the 'Alternative' that cannot name the protected branch); 04-api-design.md two-gate entry-point table (commit gate row, DryRun note) + the mandated ordering authorize()->with_authz()->acquire guard->run impl + the rejection contract; 02-system-components.md the enforcement-seam row (but-api commit wrappers; create_commit gate is 'Alternative') + naming-collision guardrail
notes:
  - The gate runs at the TOP of the but-api commit wrapper (`commit_create_only`/`commit_create`) and in the CLI commit path AFTER target-ref resolution — BEFORE `ctx.exclusive_worktree_access()` / `guard.write_permission()`. Ordering per 04-api-design.md: (1) resolve principal from BUT_AGENT_HANDLE (AUTHZ-003) — fail closed perm.denied on unset/empty/unknown; (2) determine the target ref from `relative_to` (`RelativeTo::Reference(name)`; `RelativeTo::Commit(id)` has no branch name so branch-protection is N/A but contents:write still checks); (3) decide governed-vs-ungoverned via `but_authz::governance_present` (≥1 of the two governance files committed at the target ref) — if NEITHER file is present the ref is ungoverned and the gate returns Ok (commit allowed, opt-in inactive); if governance is present, load_governance_config(repo, target_ref) (AUTHZ-002) — fail closed config.invalid on malformed/unreadable/incomplete (one-file) config; (4) authorize(principal, ContentsWrite) — perm.denied on miss; (5) if the target branch is protected, deny branch.protected. THEN acquire the guard and run the existing impl unchanged.
  - The gate's opt-in discriminator is `but_authz::governance_present` — the single source of truth for the `.gitbutler/*.toml` paths. It reports governed (≥1 of permissions.toml/gates.toml committed at the target ref) vs ungoverned (neither). An ungoverned ref short-circuits to Ok BEFORE the loader is invoked, so the loader's absence→config.invalid contract only ever bites the partial/incomplete (one-file) case, never a fully-ungoverned ref.
  - DryRun: the existing flag suppresses ref/object/oplog persistence AFTER a successful commit; the gate must run REGARDLESS of DryRun (authorization is read-only enforcement). Do not early-return on DryRun before the gate. A denied DryRun returns the contract and persists nothing; an allowed DryRun previews (Ok) and persists nothing (ref unchanged, no object) — both are asserted (AC-4).
  - Persistence assertion honesty: the but-api `commit_create` writes an oplog snapshot (create.rs:110-131); the CLI `commit2.rs::run` goes through `but_transaction::with_transaction_with_perm`. For the DryRun persistence check, assert `ref-unchanged` + `no commit object persisted for the attempt` (the universally-true invariant); only assert `0 new oplog entries` on the but-api path where oplog is actually written — do NOT assume the CLI path writes oplog.
  - Lock discipline (RULES.md): authorization is evaluated BEFORE the `RepoExclusive` worktree guard is taken; do not call permission-acquiring helpers while holding the guard — the authz check needs only `&repo` + the resolved principal + the target ref, not the write lock.
  - Mechanism-agnostic note: the seam placement makes the decision mechanism-independent by construction (every ref-named commit funnels through the but-api/CLI seam). Worktree/normal-git PARITY (T-GATES-016/017) is DEFERRED to Sprint 04 (see out_of_scope) — this task does not architecturally preclude it but does not prove it.
pattern: Gate-at-ref-aware-seam: a small `enforce_commit_gate(repo, relative_to, principal_lookup, dry_run)` called at the TOP of the but-api commit wrapper (and after target-ref resolution in the CLI), returning `Result<(), Denial>` BEFORE the RepoExclusive guard is taken; it first consults `but_authz::governance_present` (ungoverned ref → Ok, opt-in by presence) before loading config; the Denial is propagated as an anyhow error carrying the classification and surfaced as exit-1 structured JSON at the CLI.
pattern_source: crates/but-api/src/commit/create.rs:21 + crates/but/src/command/legacy/commit2.rs:119 (the ref-aware seam, guard taken after target resolution)
anti_pattern: Placing the gate at `commit_engine::create_commit` (its Destination cannot name the protected branch, so main is silently unprotected); wiring the gate only into `but-action::on_uncommitted_changes` (one caller) so the but-api wrapper / CLI reach the commit path ungated; acquiring the RepoExclusive guard BEFORE authorize (lock-ordering violation); treating an ungoverned ref (no committed `.gitbutler/*.toml`) as config.invalid (it must be allowed — opt-in by presence) OR treating a partial/incomplete (one-file) config as ungoverned (it must be config.invalid); or early-returning on DryRun before the gate so a dry-run bypasses authorization (CAP-AUTHZ-01 violation).

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — Wires the authz primitive into GitButler's REAL ref-aware commit chokepoints — the but-api commit wrappers (`commit_create_only`/`commit_create`, which receive `RelativeTo` carrying the target ref) and the `but` CLI commit path (`commit2.rs`, which resolves `RelativeTo::Reference(name)`/`BranchTip{name}`) — gating BEFORE the RepoExclusive guard is taken, proving end-to-end denial against real git. rust-implementer owns the but-api seam composition, the lock-ordering discipline, and integration TDD with but-testsupport + the CLI snapbox harness.
reviewer: rust-reviewer
coding_standards: crates/AGENTS.md, crates/WORKSPACE_MODEL.md, crates/but/AGENTS.md, /Users/justinrich/Projects/brain/docs/rust/error-handling.md

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: AUTHZ-002, AUTHZ-003
Blocks:     AUTHZ-007
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "GATES-001",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "gated_repo": {
      "description": "A real git repo (but-testsupport writable_scenario) whose target ref `main` has committed `.gitbutler/permissions.toml` (dev=contents:write, ro=contents:read) and `.gitbutler/gates.toml` (main protected), plus a feature branch `feat` with a pending worktree change to commit.",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"governance-base\");",
        "invoke_bash: write .gitbutler/permissions.toml with [[principal]] id=\"dev\" permissions=[\"contents:write\"]; [[principal]] id=\"ro\" permissions=[\"contents:read\"]",
        "invoke_bash: write .gitbutler/gates.toml with [[branch]] name=\"main\" protected=true",
        "invoke_bash: git add -A && git commit -m \"governance config\" (commits both blobs at refs/heads/main)",
        "invoke_bash: git checkout -b feat; make an uncommitted change to file.txt (staged for commit)"
      ]
    },
    "gated_repo_malformed": {
      "description": "Same repo, but the target-ref `.gitbutler/gates.toml` blob is committed with invalid TOML — to prove the commit fails closed config.invalid. [seeded via but_testsupport::writable_scenario(name) + invoke_bash git commit]",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"governance-base\");",
        "invoke_bash: write .gitbutler/permissions.toml (dev=contents:write) and .gitbutler/gates.toml with broken TOML `[[branch] name = \\\"main\\\"  protected = nope`",
        "invoke_bash: git add -A && git commit -m \"malformed gates\" (commit at refs/heads/main); git checkout -b feat; make an uncommitted change"
      ]
    },
    "gated_repo_partial_config": {
      "description": "A real repo committing EXACTLY ONE of the two governance files (`.gitbutler/permissions.toml` XOR `.gitbutler/gates.toml`) at the target ref `main` — to prove the gate fails closed config.invalid on INCOMPLETE governance (the loader requires a complete config once governance is opted-in by presence). [seeded via but_testsupport::writable_scenario(name) + invoke_bash git commit]",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"governance-base\");",
        "invoke_bash: write ONLY .gitbutler/permissions.toml (dev=contents:write) — do NOT write .gitbutler/gates.toml; git add -A && git commit -m \"partial governance (permissions only)\" (commit at refs/heads/main)",
        "invoke_bash: git checkout -b feat; make an uncommitted change (the target ref main has exactly one governance file → incomplete governance)"
      ]
    },
    "gated_repo_no_config": {
      "description": "A real repo with NO .gitbutler governance config committed at the target ref `main` — to prove the ref is UNGOVERNED and the commit is ALLOWED (opt-in by presence; governance activates only once `.gitbutler/*.toml` is committed). `but_authz::governance_present` reports neither file present, so the gate returns Ok. [seeded via but_testsupport::writable_scenario(name) + invoke_bash git commit]",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"governance-base\");",
        "invoke_bash: do NOT write any .gitbutler/*.toml; git checkout -b feat; make an uncommitted change (the target ref main has no governance config at all → ungoverned)"
      ]
    },
    "gated_repo_ghost_handle": {
      "description": "Same as gated_repo but used to drive a handle that is present in the environment yet ABSENT from the committed permissions.toml (BUT_AGENT_HANDLE=ghost). [seeded via but_testsupport::writable_scenario(name) + invoke_bash git commit]",
      "seed_method": "cli",
      "records": [
        "reuse gated_repo seeding (dev/ro committed); commit performed with BUT_AGENT_HANDLE=ghost, which resolves as an env value but is not a principal in permissions.toml"
      ]
    },
    "gated_repo_feature_head_unprotects": {
      "description": "Same as gated_repo (main protected at refs/heads/main) but feat has a COMMITTED .gitbutler/gates.toml that sets main protected=false — to prove the gate reads the TARGET ref (main), not HEAD. [seeded via but_testsupport::writable_scenario(name) + invoke_bash git commit]",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"governance-base\");",
        "invoke_bash: on main, commit .gitbutler/permissions.toml (dev=contents:write) + .gitbutler/gates.toml (main protected=true)",
        "invoke_bash: git checkout -b feat; overwrite .gitbutler/gates.toml so main protected=false; git add -A && git commit -m unprotect"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN gated_repo + BUT_AGENT_HANDLE=dev WHEN committing to feat then directly to protected main THEN feat advances and main commit is denied branch.protected naming main, exit 1, main unchanged",
      "verify": "cargo test -p but-api commit_gate_feature_ok_protected_rejected",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api commit wrapper + git",
        "negative_control": {
          "would_fail_if": [
            "the gate is a no-op stub so the main commit lands (ref advances)",
            "branch protection is read from the working tree, not the target ref",
            "the commit path never calls authorize",
            "the gate runs AFTER the RepoExclusive guard is taken (lock-ordering violation)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "gated_repo",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=dev: `but commit --branch feat -m x` (commit the pending change on feature branch feat)"
              ]
            },
            "end_state": {
              "must_observe": [
                "commit lands on `feat` (process exits `0`)",
                "`feat` HEAD sha `!=` the seeded base sha"
              ],
              "must_not_observe": [
                "`branch.protected`",
                "exit `1`",
                "no commit created"
              ]
            }
          },
          {
            "start_ref": "gated_repo",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=dev: `but commit --branch main -m x` (direct commit to protected main → BranchTip{name=main})"
              ]
            },
            "end_state": {
              "must_observe": [
                "`error.code == \"branch.protected\"`",
                "message names `\"main\"`",
                "process exits `1`",
                "`main` HEAD sha `==` the seeded base sha"
              ],
              "must_not_observe": [
                "`main` HEAD sha advanced",
                "commit landed on `main`",
                "exit `0`"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "GIVEN gated_repo + gated_repo_ghost_handle WHEN committing as ro, with no handle, with an empty handle, and with a ghost handle absent from config THEN ro is perm.denied naming contents:write and the unset/empty/ghost commits are each rejected with structured perm.denied, all exit 1, no ref change",
      "verify": "cargo test -p but-api commit_gate_readonly_and_bad_handle_denied",
      "maps_to_ac": null,
      "primary": false,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api commit wrapper + git",
        "negative_control": {
          "would_fail_if": [
            "the ro commit lands because the gate omits the contents:write check",
            "an unset/empty handle defaults to an allowed principal",
            "a ghost handle that resolves in env but is absent from config is granted an implicit identity and the commit lands",
            "the denial is not perm.denied / does not name contents:write",
            "the no-handle case exits 1 only by panicking rather than returning the structured perm.denied contract"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "gated_repo",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=ro: `but commit --branch feat -m x`"
              ]
            },
            "end_state": {
              "must_observe": [
                "`error.code == \"perm.denied\"`",
                "message names `\"contents:write\"`",
                "process exits `1`",
                "`feat` HEAD sha `==` the seeded base sha"
              ],
              "must_not_observe": [
                "commit landed",
                "`feat` HEAD sha advanced",
                "exit `0`"
              ]
            }
          },
          {
            "start_ref": "gated_repo",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE unset: `but commit --branch feat -m x`"
              ]
            },
            "end_state": {
              "must_observe": [
                "`error.code == \"perm.denied\"` (structured denial, not a bare panic)",
                "process exits `1` with no principal bound",
                "`feat` HEAD sha `==` the seeded base sha"
              ],
              "must_not_observe": [
                "commit landed",
                "exit `0`",
                "default principal"
              ]
            }
          },
          {
            "start_ref": "gated_repo",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=\"\" (empty string): `but commit --branch feat -m x`"
              ]
            },
            "end_state": {
              "must_observe": [
                "`error.code == \"perm.denied\"` (empty handle rejected same as unset)",
                "process exits `1`",
                "`feat` HEAD sha `==` the seeded base sha"
              ],
              "must_not_observe": [
                "commit landed",
                "exit `0`",
                "empty handle accepted as a principal"
              ]
            }
          },
          {
            "start_ref": "gated_repo_ghost_handle",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=ghost (resolves in env, absent from committed permissions.toml): `but commit --branch feat -m x` (cite T-AUTHZ-027)"
              ]
            },
            "end_state": {
              "must_observe": [
                "`error.code == \"perm.denied\"` (unknown principal — handle absent from committed config)",
                "process exits `1`",
                "`feat` HEAD sha `==` the seeded base sha"
              ],
              "must_not_observe": [
                "commit landed",
                "exit `0`",
                "ghost granted an implicit identity"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "GIVEN a working-tree edit OR a feature-head committed gates.toml unprotecting main WHEN committing to main as dev THEN still branch.protected (protection read from the target ref, not working tree/HEAD)",
      "verify": "cargo test -p but-api commit_gate_edit_cannot_unprotect",
      "maps_to_ac": null,
      "primary": false,
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api commit wrapper + git",
        "negative_control": {
          "would_fail_if": [
            "the gate reads working-tree gates.toml so the edit unprotects main and the commit lands",
            "protection is resolved from the feature head instead of the target ref (the feat committed gates.toml wins and the commit lands)",
            "protection is resolved from HEAD instead of the supplied target ref (a no-op pin)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "gated_repo",
            "action": {
              "actor": "cli_user",
              "steps": [
                "edit working-tree .gitbutler/gates.toml so main protected=false (do NOT commit)",
                "BUT_AGENT_HANDLE=dev: `but commit --branch main -m x` (attempt direct commit to main)"
              ]
            },
            "end_state": {
              "must_observe": [
                "`error.code == \"branch.protected\"`",
                "process exits `1`",
                "`main` HEAD sha `==` the seeded base sha"
              ],
              "must_not_observe": [
                "`main` HEAD sha advanced",
                "commit landed on `main`",
                "exit `0`"
              ]
            }
          },
          {
            "start_ref": "gated_repo_feature_head_unprotects",
            "action": {
              "actor": "cli_user",
              "steps": [
                "on feat (whose COMMITTED gates.toml unprotects main), BUT_AGENT_HANDLE=dev: `but commit --branch main -m x`"
              ]
            },
            "end_state": {
              "must_observe": [
                "`error.code == \"branch.protected\"` (protection read from target ref main, not the feature head)",
                "process exits `1`",
                "`main` HEAD sha `==` the seeded base sha"
              ],
              "must_not_observe": [
                "`main` HEAD sha advanced",
                "commit landed on `main`",
                "exit `0`"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "description": "GIVEN a malformed target-ref gates.toml AND a PARTIAL (one-file) config AND a target ref with NO committed governance config WHEN committing THEN config.invalid for malformed and partial, but the ungoverned (no-config) ref is ALLOWED (opt-in by presence, discriminated by but_authz::governance_present); AND a DryRun protected-main commit still denies branch.protected and persists nothing, while a DryRun feat commit previews Ok and persists nothing",
      "verify": "cargo test -p but-api commit_gate",
      "maps_to_ac": null,
      "primary": false,
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api commit wrapper + git",
        "negative_control": {
          "would_fail_if": [
            "malformed config is skipped (treated as an empty config) and the commit proceeds (fail-open)",
            "a partial/incomplete config (one file present) is treated as ungoverned and the commit proceeds (must instead be config.invalid)",
            "an ungoverned ref (no committed .gitbutler/*.toml) is treated as config.invalid and the commit is denied (it must be ALLOWED — opt-in by presence)",
            "DryRun bypasses authorization so the protected-branch denial does not fire",
            "a denied DryRun still writes a ref/object",
            "an allowed DryRun advances feat HEAD or writes a persisted object (DryRun must only preview)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "gated_repo_malformed",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=dev: `but commit --branch feat -m x` with malformed target-ref gates.toml"
              ]
            },
            "end_state": {
              "must_observe": [
                "`error.code == \"config.invalid\"`",
                "process exits `1`"
              ],
              "must_not_observe": [
                "commit landed",
                "exit `0`",
                "skipped check"
              ]
            }
          },
          {
            "start_ref": "gated_repo_partial_config",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=dev: `but commit --branch feat -m x` with a PARTIAL config (exactly one of permissions.toml / gates.toml committed at the target ref) — incomplete governance fails closed"
              ]
            },
            "end_state": {
              "must_observe": [
                "`error.code == \"config.invalid\"` (incomplete governance — one file present, fail closed)",
                "process exits `1`"
              ],
              "must_not_observe": [
                "commit landed",
                "exit `0`",
                "partial config treated as ungoverned (commit proceeds)"
              ]
            }
          },
          {
            "start_ref": "gated_repo_no_config",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=dev: `but commit --branch feat -m x` with NO governance config committed at the target ref (ungoverned — opt-in by presence; but_authz::governance_present reports neither file present, so the gate returns Ok)"
              ]
            },
            "end_state": {
              "must_observe": [
                "process exits `0` (ungoverned ref → commit allowed)",
                "`feat` HEAD sha `!=` the seeded base sha (commit landed)"
              ],
              "must_not_observe": [
                "`error.code == \"config.invalid\"`",
                "exit `1`",
                "no commit created"
              ]
            }
          },
          {
            "start_ref": "gated_repo",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=dev: DryRun commit to protected main (`but commit --branch main --dry-run -m x`)"
              ]
            },
            "end_state": {
              "must_observe": [
                "`error.code == \"branch.protected\"`",
                "process exits `1`",
                "`main` HEAD sha `==` base AND no commit object was persisted for this attempt"
              ],
              "must_not_observe": [
                "exit `0`",
                "`main` HEAD sha advanced",
                "a persisted ref/object from the denied dry run"
              ]
            }
          },
          {
            "start_ref": "gated_repo",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=dev: DryRun commit to authorized feat (`but commit --branch feat --dry-run -m x`) — paired ALLOWED DryRun (cite RF-008)"
              ]
            },
            "end_state": {
              "must_observe": [
                "process exits `0` with a preview/Ok outcome (authorization passed under DryRun)",
                "`feat` HEAD sha `==` the seeded base sha (DryRun persisted nothing)"
              ],
              "must_not_observe": [
                "`feat` HEAD sha advanced",
                "a persisted commit object from the dry run",
                "exit `1`",
                "no new commit object persisted (0 objects written by the dry run)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "feature ok / protected rejected with branch.protected",
      "verify": "cargo test -p but-api commit_gate_feature_ok_protected_rejected",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "read-only perm.denied + unset/empty/ghost handle rejected",
      "verify": "cargo test -p but-api commit_gate_readonly_and_bad_handle_denied",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "working-tree edit and feature-head commit cannot unprotect (ref-pin)",
      "verify": "cargo test -p but-api commit_gate_edit_cannot_unprotect",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "malformed + partial (one-file) -> config.invalid; absent (no governance) -> commit allowed (ungoverned, opt-in by presence); DryRun enforced (deny persists nothing) + allowed DryRun previews",
      "verify": "cargo test -p but-api commit_gate",
      "maps_to_ac": "AC-4"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "protection from target-ref committed gates.toml, not hardcoded / not working tree / not HEAD",
      "verify": "cargo test -p but-api commit_gate_feature_ok_protected_rejected",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "description": "authorization runs before the RepoExclusive guard (lock ordering)",
      "verify": "cargo test -p but-api commit_gate_feature_ok_protected_rejected",
      "maps_to_ac": "AC-1"
    }
  ]
}
-->
</details>
