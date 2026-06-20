# GATES-007: Mechanism-agnostic commit gate — gate `worktree_integrate` (advances a protected target → branch.protected) and `branch::apply` / `apply_branch_integration` (contents:write authorization) at the but-api PUBLIC seam, workspace-target-ref-pinned

## What this does

Closes the ungated ref-advancing entry points that bypass the plain-commit gate (GATES-001) by firing the SAME commit-gate decision (`crates/but-api/src/commit/gate.rs::enforce_commit_gate_for_target`) at three but-api PUBLIC seams, each gated for what it ACTUALLY mutates AND reading governance from the ref that actually carries it (verified against live code):

- **`worktree_integrate`** (`crates/but-api/src/legacy/worktree.rs:54`) genuinely advances its `target` ref ("squashed into a single commit which becomes the new tip of `target`", `crates/but-worktrees/src/integrate.rs:55-57`). This is the SOUND branch.protected case: gated via `CommitGateTarget::direct_ref(target)` resolved from the `target: gix::refs::FullName` argument (which IS the workspace trunk being advanced AND the ref where `.gitbutler/*.toml` is committed), so a worktree-integrate advancing a protected branch is rejected `branch.protected`. `worktree_integrate` is itself the public seam — the gate fires at the TOP of its body, BEFORE its own `exclusive_worktree_access()` guard (:59), obtaining the repo via `ctx.repo.get()?`.
- **`branch::apply`** (the PUBLIC `apply`, `branch.rs:643`) ADDS a branch to the workspace and `bail!`s on the target ("Cannot add the target '{branch}' branch to its own workspace", `crates/but-workspace/src/branch/apply.rs:232`) — it does NOT advance a protected trunk. So it gets the mechanism-agnostic coverage that is REAL: the **contents:write authorization** check via `CommitGateTarget::config_only(<workspace target ref>)`. CRITICAL (S9): governance lives on the workspace TARGET ref (the trunk, e.g. `main`), NOT on the feature branch being applied — so the gate's `config_ref` MUST be the workspace target ref resolved via `ctx.project_meta()?.target_ref_or_err()` (`crates/but-core/src/ref_metadata.rs:357`; field `target_ref: Option<gix::refs::FullName>` at :17), NOT `existing_branch`. CRITICAL (S9b): MATCH the accessor — do NOT `?`-propagate `target_ref_or_err()`. When NO default target is configured (`target_ref` is `None`, `ref_metadata.rs:357-362` returns `Err(but_error::Code::DefaultTargetNotFound)`), the operation is a LEGITIMATE ungoverned apply (`crates/but-workspace/src/branch/apply.rs:44-46`: "if there is no target … the current branch and the given one will be applied") — SKIP the gate and proceed (ungoverned = permit, exactly as the plain-commit gate's `commit_gate_absent_config_is_ungoverned`, `crates/but-api/tests/commit_gate.rs:255-289`). Wire it as `if let Ok(target_ref) = ctx.project_meta()?.target_ref_or_err() { commit::gate::enforce_commit_gate_for_target(&repo, &CommitGateTarget::config_only(target_ref.clone()))?; }`. (`ctx.project_meta()?` itself may still `?`-propagate a genuine metadata-read I/O error; only the no-target case is treated as ungoverned-permit, never a hard-error.) A read-only principal is denied `perm.denied` on this ref-mutating entry point when a target EXISTS and carries governance; no vacuous protected-branch assertion is made. The gate fires at the TOP of the PUBLIC `apply` (branch.rs:643), BEFORE `ctx.exclusive_worktree_access()` (:647) and before delegating to `apply_with_perm`, obtaining the repo via `ctx.repo.get()?` — mirroring `commit::gate::enforce_commit_gate` at `create.rs:35` (gate) then `:38` (guard).
- **`apply_branch_integration`** (the PUBLIC `apply_branch_integration`, `branch.rs:939`) integrates upstream INTO its `branch` arg (the local/feature branch) — it does NOT advance a protected trunk either. Same treatment, same S9b MATCH (not `?`-propagate): `if let Ok(target_ref) = ctx.project_meta()?.target_ref_or_err() { commit::gate::enforce_commit_gate_for_target(&repo, &CommitGateTarget::config_only(target_ref.clone()))?; }` (the trunk where governance is committed — NOT `branch`; no default target = ungoverned = permit), the gate fired at the TOP of the PUBLIC `apply_branch_integration` (branch.rs:939) BEFORE `ctx.exclusive_worktree_access()` (:946) and before delegating to `apply_branch_integration_with_perm`, plus a DEDICATED DryRun-no-bypass proof (the underlying `apply_branch_integration_with_perm` takes `dry_run: DryRun`, `branch.rs:961`). (`worktree_integrate` is NOT affected by S9b — it derives its config_ref from its own `target` arg via `direct_ref(target.clone())`, never calling `target_ref_or_err`.)

All three reuse the EXISTING commit-gate decision (which internally calls `but_authz::governance_present` — the live opt-in discriminator at `crates/but-authz/src/config.rs:44`, re-exported `crates/but-authz/src/lib.rs:14` — inside `enforce_commit_gate_for_target` at `gate.rs:60`), so the decision is provably identical across mechanisms; NO new `gitbutler-*` coupling is added. Because every `config_ref` is the ref that ACTUALLY carries governance — the advanced trunk for worktree_integrate, the workspace target ref for apply/integrate — `governance_present(target_ref)` is TRUE on a governed repo and the contents:write/branch.protected check genuinely runs (it does NOT exit `Ok()` early as it would if the feature branch ref were used).

## Why

Sprint 04 · PRD UC-GATES-01 (mechanism-agnostic, AC-5/AC-9) · capabilities CAP-AUTHZ-01, CAP-CONFIG-01. GATES-001 sited the commit gate at the but-api `_with_authz` commit seam + CLI commit path; T-GATES-016 (mechanism parity) and T-GATES-017 (worktree path) were explicitly deferred to Sprint 04. A gate wired to only the plain-commit mechanism is bypassed by another ref-advancing entry point. This task makes the gate actually mechanism-agnostic: `worktree_integrate` (which truly advances a protected target) is gated branch.protected, and `branch::apply` / `apply_branch_integration` (which are ref-mutating but do NOT advance a protected trunk) are gated for contents:write so a read-only principal cannot mutate refs through them. Governance is read from the ref that carries it — the workspace target ref — so a feature-branch ref (which has no committed `.gitbutler/*.toml`) can NEVER make the gate exit `Ok()` early and let a read-only principal through ungated. See the UPSTREAM ADVISORY below: ROADMAP/SPRINT gate step 6 assumed apply/integrate advance a protected trunk, which the live code contradicts.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-api commit_gate_worktree_integrate_protected_rejected` (integration, real but-api worktree seam + real git). Full gate set in the spec below.

## Scope

- `crates/but-api/src/legacy/worktree.rs` (MODIFY) — fire `commit::gate::enforce_commit_gate_for_target(&repo, &CommitGateTarget::direct_ref(target.clone()))` at the TOP of `worktree_integrate` (:54), resolving the gate target from its `target: gix::refs::FullName` (the ref actually advanced AND where governance is committed), obtaining the repo via `ctx.repo.get()?`, BEFORE the `exclusive_worktree_access()` guard (:59). `worktree_integrate` IS the public seam.
- `crates/but-api/src/branch.rs` (MODIFY) — fire the gate at the TOP of the PUBLIC `apply` (:643) and the PUBLIC `apply_branch_integration` (:939) via the S9b MATCH form `if let Ok(target_ref) = ctx.project_meta()?.target_ref_or_err() { enforce_commit_gate_for_target(&repo, &CommitGateTarget::config_only(target_ref.clone()))?; }` (the config_ref is the WORKSPACE TARGET ref — the trunk that carries committed governance, NOT the feature branch), obtaining the repo via `ctx.repo.get()?`, BEFORE `ctx.exclusive_worktree_access()` (:647 / :946) and before delegating to the `*_with_perm` body. MATCH (not `?`-propagate) `target_ref_or_err()`: a repo with NO configured default target (`target_ref` None -> `DefaultTargetNotFound`, ref_metadata.rs:357-362) is a LEGITIMATE ungoverned apply (apply.rs:44-46) — SKIP the gate and proceed (ungoverned = permit, parity with `commit_gate_absent_config_is_ungoverned`, commit_gate.rs:255-289); a `target_ref_or_err()?` propagation would hard-error a legitimate no-target apply (S9b regression). This enforces contents:write + fail-closed authorization on the ref-mutating entry points WITHOUT asserting a (false) protected-branch advancement; honor DryRun on the integrate path (the gate runs in the public `apply_branch_integration` before the guard, so DryRun never bypasses it)
- `crates/but-api/src/commit/gate.rs` (MODIFY — minimal, ONLY if a non-`RelativeTo` target-construction helper is needed) — `CommitGateTarget::direct_ref` (:25) and `config_only` (:33) BOTH already exist and are `pub`; do NOT change the gate decision logic (OWNED by GATES-001)
- `crates/but-api/tests/commit_gate.rs` (MODIFY) — add the worktree-integrate branch.protected case + the apply/integrate contents:write (perm.denied) cases + the DryRun-no-bypass case against real but-api + real git (no mocks)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: GATES-007 - Mechanism-agnostic commit gate: gate worktree_integrate (advances a protected target -> branch.protected) and branch::apply / apply_branch_integration (contents:write authorization) at the but-api PUBLIC seam, workspace-target-ref-pinned
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     L  (270 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-GATES-01
CAPABILITIES: CAP-AUTHZ-01, CAP-CONFIG-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-api commit_gate_worktree_integrate_protected_rejected
  check: cargo check -p but-api --all-targets
  lint:  cargo clippy -p but-api --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Integration tests are green against the real but-api branch/worktree seams + real git, gated for what each entry point ACTUALLY mutates AND reading governance from the ref that actually carries it (verified against live code): (1) a `worktree_integrate` that targets a protected branch is rejected `branch.protected` naming the branch (nothing advanced — the target ref sha == base), because worktree_integrate genuinely advances its `target` (but-worktrees/src/integrate.rs:55-57) and `direct_ref(target)` reads governance from that same advanced trunk; a worktree-integrate onto a feature target by a contents:write principal is PERMITTED by the gate. (2) a `branch::apply` (PUBLIC `apply`) and an `apply_branch_integration` (PUBLIC `apply_branch_integration`) by a `contents:write`-lacking principal are denied `perm.denied` naming contents:write — these ref-mutating entry points enforce contents:write authorization, gated via `config_only(target_ref)` where the workspace target ref is obtained by MATCHING `ctx.project_meta()?.target_ref_or_err()` (the trunk where `.gitbutler/*.toml` is committed, NOT the feature branch — using the feature branch would make governance_present return false and the gate would exit Ok ungated); a repo with NO configured default target (target_ref_or_err -> Err DefaultTargetNotFound) is a LEGITIMATE ungoverned apply that is PERMITTED (the gate is SKIPPED, parity with commit_gate_absent_config_is_ungoverned) — MATCHED, not `?`-propagated, so a no-target apply does NOT hard-error (S9b); they do NOT advance a protected trunk: apply bails on the target, integrate writes a feature branch — so NO vacuous branch.protected assertion is made on them; a feature-branch apply/integrate by a `contents:write` principal is PERMITTED by the gate (the gate returns Ok; any subsequent error is the operation's own, not a governance Denial). (3) the decision is read ONLY from the workspace-TARGET-REF `.gitbutler/gates.toml` blob; an ungoverned target ref (no committed `.gitbutler/*.toml`) is allowed (opt-in by presence via the EXISTING `but_authz::governance_present`, config.rs:44, called inside enforce_commit_gate_for_target gate.rs:60); the integrate gate fires EVEN under DryRun (a denied DryRun apply_branch_integration persists nothing) — proven as a DEDICATED sub-case, because the gate runs in the PUBLIC `apply_branch_integration` BEFORE the guard, not inside the DryRun-threaded `_with_perm` body. All three entry points use the SAME `commit::gate::enforce_commit_gate_for_target` decision as the plain-commit gate (GATES-001) — proving mechanism parity, not a parallel re-implementation (grep build-gate, PER-FILE: branch.rs >= 2, worktree.rs >= 1).

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST gate the THREE ref-advancing/ref-mutating entry points that bypass the plain-commit path, EACH for what it ACTUALLY mutates AND each reading governance from the ref that ACTUALLY carries it (verified against live code): (1) `worktree_integrate` (crates/but-api/src/legacy/worktree.rs:54) — gate `branch.protected` via CommitGateTarget::direct_ref(target), because it genuinely advances `target` (but-worktrees/src/integrate.rs:55-57 "becomes the new tip of `target`") AND governance is committed on that target trunk; (2) the PUBLIC `apply` (crates/but-api/src/branch.rs:643) — gate contents:write via CommitGateTarget::config_only(<workspace target ref>), because apply ADDS a branch to the workspace and bails on the target (but-workspace/src/branch/apply.rs:232), it does NOT advance a protected trunk; (3) the PUBLIC `apply_branch_integration` (branch.rs:939) — gate contents:write via CommitGateTarget::config_only(<workspace target ref>), because integrate_branch_with_steps integrates upstream INTO `branch` (the feature branch), it does NOT advance a protected trunk. Do NOT gate fewer — a gate wired to one mechanism is bypassed by the others (T-GATES-016/017).
- [MUST] (S9 — config ref) MUST use the WORKSPACE TARGET ref as the `config_ref` for apply + apply_branch_integration — `CommitGateTarget::config_only(target_ref.clone())` where `target_ref` is obtained by MATCHING `ctx.project_meta()?.target_ref_or_err()` (crates/but-core/src/ref_metadata.rs:357; the field is `target_ref: Option<gix::refs::FullName>` at :17, precedent read at crates/but-api/src/branch.rs:810 via the sibling `target_commit_id_or_err()`). MUST NOT use the FEATURE branch (`existing_branch` / `branch`) as the config_ref: governance lives at the workspace target ref (e.g. main) and NOT on a feature branch, so `but_authz::governance_present(repo, feature_ref)` returns FALSE -> the gate exits `Ok()` early (gate.rs:60-62) -> the contents:write check NEVER runs -> a read-only (`ro`) principal is PERMITTED ungated. The AC-2 fixture commits governance ONLY at refs/heads/main; reading from the feature branch makes the gate VACUOUS. (CAP-CONFIG-01: governance read at the target ref.) NOTE: for worktree_integrate, `target` IS the workspace trunk being advanced and carries governance, so direct_ref(target) is BOTH the protected-branch carrier AND the config_ref — correct (worktree_integrate never calls target_ref_or_err, so it is NOT affected by S9b).
- [MUST] (S9b — no-target apply MUST NOT hard-error) MUST MATCH the `target_ref_or_err()` accessor for apply + apply_branch_integration, NOT `?`-propagate it: `if let Ok(target_ref) = ctx.project_meta()?.target_ref_or_err() { enforce_commit_gate_for_target(&repo, &CommitGateTarget::config_only(target_ref.clone()))?; }`. A repo with NO configured default target (`target_ref` is None -> `target_ref_or_err()` returns `Err(but_error::Code::DefaultTargetNotFound)`, ref_metadata.rs:357-362) MUST NOT hard-error — that is a LEGITIMATE ungoverned operation today (crates/but-workspace/src/branch/apply.rs:44-46 "if there is no target … the current branch and the given one will be applied"; the existing plain-commit test `commit_gate_absent_config_is_ungoverned`, crates/but-api/tests/commit_gate.rs:255-289, drives a no-default-target repo and SUCCEEDS). Treat no-target as ungoverned (permit — SKIP the gate and proceed), matching the plain-commit gate's opt-in-by-presence; only a target that EXISTS but carries no committed `.gitbutler/*.toml` is the governance_present opt-in case (the gate runs and exits Ok early via governance_present). NOTE: `ctx.project_meta()?` may still `?`-propagate a genuine metadata-read I/O error; ONLY the no-target (DefaultTargetNotFound) case is the ungoverned-permit path, never a hard-error.
- [MUST] MUST NOT assert "apply/integrate advancing a protected main -> branch.protected" — that flow is INCOHERENT against the live code (apply bails on the target; integrate writes a feature branch). The branch.protected proof belongs ONLY to worktree_integrate (the entry point that truly advances `target`). For apply + apply_branch_integration, the REAL mechanism-agnostic coverage is the contents:write authorization check (a read-only principal is denied perm.denied), using config_only(<workspace target ref>) so the gate reads real governance and enforces contents:write + fail-closed WITHOUT a vacuous protected-branch assertion.
- [MUST] MUST REUSE the EXISTING commit-gate decision — call `crate::commit::gate::enforce_commit_gate_for_target(&repo, &target)` (crates/but-api/src/commit/gate.rs:55) with a `CommitGateTarget` built from the governance-carrying ref: `direct_ref(target)` for worktree_integrate (the advanced trunk), `config_only(<workspace target ref>)` for apply / apply_branch_integration (the trunk whose committed governance config is read). Do NOT re-implement contents:write/branch-protection here — that is GATES-001's logic; this task wires the SAME helper at the call sites so the decision is provably identical across mechanisms.
- [MUST] (R2b — gate BEFORE the guard, at the PUBLIC seam) MUST run the gate at the PUBLIC entry point, BEFORE the `RepoExclusive`/`exclusive_worktree_access()` guard is acquired and before any ref/object/oplog mutation — per 04-api-design.md the ordering is authorize -> acquire guard -> run impl. The PUBLIC `apply` (branch.rs:643) and PUBLIC `apply_branch_integration` (branch.rs:939) take the guard via `ctx.exclusive_worktree_access()` (:647 / :946) and THEN delegate to `apply_with_perm` / `apply_branch_integration_with_perm` — which therefore run UNDER the guard. So the gate MUST fire in the PUBLIC function (`apply` / `apply_branch_integration`), BEFORE :647 / :946, obtaining a pre-guard repo handle via `ctx.repo.get()?`. Do NOT place the gate at the top of `apply_with_perm` / `apply_branch_integration_with_perm` — those bodies already run under the guard (lock-ordering violation). In worktree.rs the PUBLIC seam IS `worktree_integrate`, whose guard is taken at :59 (first body line) — fire the gate before :59 via `ctx.repo.get()?`. This mirrors the commit-gate precedent: `crates/but-api/src/commit/create.rs:35` (gate) then `:38` (guard). NEVER overload the repo Permission/RepoExclusive lock as the authorization carrier (it is the orthogonal Authority axis; RULES.md lock discipline — do not acquire permission-helpers while holding a guard).
- [MUST] MUST read governance config ONLY from the WORKSPACE-TARGET-REF `.gitbutler/gates.toml`/permissions.toml blob via the commit gate's loader path (`load_governance_config` inside `enforce_commit_gate_for_target`, gate.rs:64) — a working-tree OR feature-head edit can NEVER weaken the gate (CAP-CONFIG-01). The `CommitGateTarget`'s `config_ref` IS the ref the gate reads from; for apply/integrate it MUST be the workspace target ref (not the working tree, not the feature head). Opt-in is determined by `but_authz::governance_present(repo, full_name)` (gate.rs:60 → config.rs:44), the live discriminator — an ungoverned target ref returns Ok early.
- [MUST] MUST run the gate EVEN under DryRun on the integrate path — the PUBLIC `apply_branch_integration` (branch.rs:939) forwards `dry_run: DryRun` to `apply_branch_integration_with_perm` (which takes it at branch.rs:961). Because the gate fires in the PUBLIC function BEFORE the guard and BEFORE the `_with_perm` body, it runs regardless of `dry_run` and a denial returns the contract; DryRun only suppresses persisting refs/objects/oplog (CAP-AUTHZ-01). Do NOT early-return on DryRun before the gate. NOTE: the public `apply` and `worktree_integrate` have NO DryRun param — the DryRun-no-bypass proof is SPECIFIC to apply_branch_integration and is a DEDICATED sub-case (AC-3).
- [MUST] MUST resolve the acting principal ONLY from BUT_AGENT_HANDLE (the commit gate's `resolve_principal_from_env` inside `enforce_commit_gate_for_target` at gate.rs:65 already does this) and fail closed perm.denied on unset/empty/unknown handle — never a default/anonymous principal (CAP-AUTHZ-01).
- [NEVER] NEVER add new `gitbutler-*` coupling beyond what the boundary strictly requires — the gate fires at the but-api boundary (branch.rs / worktree.rs), NOT inside `gitbutler-branch-actions/src/actions.rs` or `branch_upstream_integration.rs` (crates/AGENTS.md legacy caution). The but-api `branch.rs`/`worktree.rs` functions are the modern boundary; gating there covers the legacy callers without touching the legacy crate.
- [NEVER] NEVER persist anything on a denied operation — a denial returns from the PUBLIC function before the guard/mutation; a denied DryRun integrate persists nothing.
- [NEVER] NEVER use a hardcoded protected-branch list — protection (for the worktree_integrate case) comes from the committed target-ref gates.toml (UC-GATES-01 AC-7), via the reused commit-gate helper.
- [NEVER] NEVER cite `has_governance_marker` or a `gate.rs:161-179` opt-in discriminator — that was a FABRICATED-GROUNDING error in this task's prior output (now corrected). `has_governance_marker` has ZERO matches anywhere in crates/; the live opt-in discriminator is `but_authz::governance_present` (config.rs:44, exported lib.rs:14), invoked inside enforce_commit_gate_for_target (gate.rs:60). gate.rs is 170 lines total; gate.rs:159-170 is `branch_protected()`.
- [STRICTLY] STRICTLY surface the denial as the SAME structured contract the commit gate uses ({code,message} via `commit::gate::classify_error`, gate.rs:81; code ∈ {branch.protected, perm.denied, config.invalid}) — the apply/integrate/worktree paths propagate the gate's anyhow error unchanged so the boundary classifies it identically.
- [STRICTLY] STRICTLY scope to wiring the EXISTING gate at the three PUBLIC entry points + the parity tests — do NOT change the commit-gate decision logic (GATES-001), the merge gate (GATES-003/AUTHZ-004), or the review-requirement evaluator (GATES-005/GATES-006).

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: a `worktree_integrate` advancing a protected target is rejected branch.protected (the target ref sha == base); a worktree-integrate onto a feature target by contents:write proceeds
- [ ] AC-2: a contents:write-lacking principal is denied perm.denied on the PUBLIC `branch::apply` AND `apply_branch_integration` (these enforce contents:write read from the WORKSPACE TARGET ref but do NOT advance a protected trunk — NO branch.protected assertion); a feature-branch apply/integrate by a contents:write principal proceeds; (S9b) an apply/integrate on a repo with NO configured default target is PERMITTED (gate SKIPPED — ungoverned — not a hard-error on DefaultTargetNotFound)
- [ ] AC-3: the integrate gate fires EVEN under DryRun (a denied DryRun apply_branch_integration persists nothing) as a DEDICATED sub-case; the decision is workspace-target-ref-pinned; all three entry points call the SAME commit-gate decision helper (build-gate, PER-FILE: branch.rs >= 2, worktree.rs >= 1)
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: worktree_integrate advancing a protected target is rejected branch.protected; a feature-target worktree-integrate proceeds [PRIMARY]
  GIVEN: fixture `gated_worktree_repo` (target ref main protected via committed gates.toml; permissions: dev=contents:write; a worktree set up so worktree_integrate(id, target) has a valid id and a delta to squash onto target), BUT_AGENT_HANDLE=dev
  WHEN:  `worktree_integrate` is invoked targeting protected `main`; then `worktree_integrate` is invoked targeting a non-protected feature target
  THEN:  the protected-main worktree-integrate is denied error.code=="branch.protected" naming `main` (the operation does not advance main — main HEAD sha == base); the feature-target worktree-integrate is PERMITTED by the gate (the gate returns Ok; any subsequent error is the operation's own, not a governance Denial)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api worktree seam + real git (but-testsupport)
  VERIFY: cargo test -p but-api commit_gate_worktree_integrate_protected_rejected
  SCENARIO: NEGATIVE_CONTROL would fail if worktree_integrate is ungated so the protected-main integrate advances main (ref advances); branch protection is read from the working tree rather than the target ref; the gate is a no-op stub; the gate runs AFTER the exclusive_worktree_access guard (:59) (lock-ordering violation); the gate is wired to a different (non-target-advancing) entry point so worktree_integrate bypasses.

AC-2: A contents:write-lacking principal is denied perm.denied on the PUBLIC branch::apply and apply_branch_integration (config read from the WORKSPACE TARGET ref); a contents:write principal proceeds
  GIVEN: fixture `gated_apply_repo` (workspace target ref main protected via committed gates.toml at refs/heads/main — apply/integrate do NOT advance main; governance is read from this target ref, NOT from the feature branch; permissions: dev=contents:write, ro=contents:read) with a feature branch and an applicable/integratable change, and the project's default target set to main so `ctx.project_meta()?.target_ref_or_err()` resolves to main; AND fixture `apply_repo_no_target` (NO configured default target, NO committed `.gitbutler/*.toml`, a feature branch with an applicable change, BUT_AGENT_HANDLE=ro)
  WHEN:  the PUBLIC `branch::apply` and the PUBLIC `apply_branch_integration` (DryRun=No) are each invoked by ro (contents:read only); then each is invoked by dev (contents:write) on the feature branch; then (S9b) `branch::apply` and `apply_branch_integration` are invoked by ro on the no-default-target repo
  THEN:  each ro operation on the governed repo is denied error.code=="perm.denied" naming `contents:write` — ro's edits never reach a ref (the ref's HEAD sha == base on each path); each dev feature-branch operation is PERMITTED by the gate (the gate returns Ok; any subsequent error is the operation's own, not a governance Denial); the no-default-target apply/integrate is PERMITTED — the gate is SKIPPED because `target_ref_or_err()` returns Err(DefaultTargetNotFound) (matched, not `?`-propagated) so a legitimate ungoverned no-target apply does NOT hard-error (S9b, parity with commit_gate_absent_config_is_ungoverned). NO branch.protected assertion is made on apply/integrate — they do not advance a protected trunk
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api branch seam + real git
  VERIFY: cargo test -p but-api commit_gate_apply_integrate_readonly_denied && cargo test -p but-api commit_gate_apply_integrate_no_target_ungoverned
  SCENARIO: NEGATIVE_CONTROL would fail if the apply/integrate path omits the contents:write check so ro's operation lands; the denial is not perm.denied / does not name contents:write; only one of apply/integrate is gated so ro bypasses via the other; a missing handle defaults to an allowed principal; the gate vacuously asserts branch.protected on apply/integrate (a false guarantee — they do not advance a protected trunk); config_only uses the feature branch ref so governance_present(feat)=false and the gate exits Ok ungated, permitting ro; (S9b) `target_ref_or_err()?` propagates DefaultTargetNotFound, turning a legitimate no-target apply into a hard error (the gate is not skipped on no-target).

AC-3: The integrate gate fires under DryRun (dedicated) + decision is workspace-target-ref-pinned + all three share the SAME commit-gate decision helper [build-gate + integration]
  GIVEN: fixture `gated_apply_repo` under DryRun on the integrate path (the PUBLIC apply_branch_integration forwards dry_run: DryRun, branch.rs:961); and fixture `gated_apply_repo_wt_unprotect` (WORKING-TREE gates.toml edited to weaken governance, NOT committed); BUT_AGENT_HANDLE=ro for the DryRun denial
  WHEN:  a DryRun=Yes `apply_branch_integration` by ro is attempted; a `branch::apply` by ro is attempted with the working-tree edit present; AND the source is structurally inspected
  THEN:  the DryRun apply_branch_integration is STILL denied perm.denied (the gate runs in the PUBLIC function even under DryRun) and persists nothing (no ref/object/oplog) — this is the DEDICATED DryRun-no-bypass sub-case for apply_branch_integration (the public apply + worktree_integrate have no DryRun param, so they are not part of this sub-case); the working-tree-edited apply is still denied per the workspace-target-ref config (config read from the target ref, not the working tree); and all three entry points call `commit::gate::enforce_commit_gate_for_target` (build-gate PER-FILE: branch.rs >= 2, worktree.rs >= 1 — proving one shared decision, not three parallel re-implementations)
  TEST_TIER: integration + build-gate   VERIFICATION_SERVICE: real but-api branch/worktree seam + real git; source grep   UNIT_TEST_JUSTIFIED: the shared-helper structural invariant (all three call the one decision fn, per-file) is a grep/compile build-gate; the DryRun-no-bypass + target-ref-pin behaviors are integration-proven
  VERIFY: cargo test -p but-api commit_gate_apply_integrate_dryrun_targetref_pinned && ./tools/governance-checks/check_gate_before_guard.py

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, error): worktree_integrate onto protected main denied branch.protected naming main; main unchanged (T-GATES-017)
    VERIFY: cargo test -p but-api commit_gate_worktree_integrate_protected_rejected
- TC-2 (-> AC-1, happy_path): a feature-target worktree-integrate by a contents:write principal is permitted by the gate (no governance Denial)
    VERIFY: cargo test -p but-api commit_gate_worktree_integrate_protected_rejected
- TC-3 (-> AC-2, error): a contents:read-only principal denied perm.denied on the PUBLIC branch::apply AND apply_branch_integration (config read from the workspace target ref); NO branch.protected vacuously asserted on them (T-GATES-016, T-AUTHZ-011)
    VERIFY: cargo test -p but-api commit_gate_apply_integrate_readonly_denied
- TC-4 (-> AC-2, happy_path): a feature-branch apply/integrate by a contents:write principal permitted by the gate (no governance Denial)
    VERIFY: cargo test -p but-api commit_gate_apply_integrate_readonly_denied
- TC-5 (-> AC-3, edge): a DryRun=Yes apply_branch_integration is STILL gated (perm.denied for ro) and persists nothing — DEDICATED DryRun-no-bypass sub-case for apply_branch_integration (CAP-AUTHZ-01 DryRun-enforced)
    VERIFY: cargo test -p but-api commit_gate_apply_integrate_dryrun_targetref_pinned
- TC-6 (-> AC-3, edge): a working-tree gates.toml edit cannot weaken; the workspace-target-ref blob governs apply/integrate/worktree (T-GATES-019 family, CAP-CONFIG-01)
    VERIFY: cargo test -p but-api commit_gate_apply_integrate_dryrun_targetref_pinned
- TC-7 (-> AC-3, structural): all three entry points call the SAME commit::gate::enforce_commit_gate_for_target decision (mechanism parity, PER-FILE: branch.rs >= 2 AND worktree.rs >= 1, not a parallel gate) — the per-file parity script (REM-005) AND the gate-before-guard source-contract script (REM-006) both pass
    VERIFY: ./tools/governance-checks/check_gate_helper_parity.sh && ./tools/governance-checks/check_gate_before_guard.py
- TC-8 (-> AC-2, edge): (S9b) branch::apply AND apply_branch_integration on a repo with NO configured default target are PERMITTED — the gate is SKIPPED (target_ref_or_err() Err(DefaultTargetNotFound) MATCHED, not ?-propagated) so a legitimate ungoverned no-target apply does NOT hard-error (parity with commit_gate_absent_config_is_ungoverned, commit_gate.rs:255-289; apply.rs:44-46)
    VERIFY: cargo test -p but-api commit_gate_apply_integrate_no_target_ungoverned

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01, CAP-CONFIG-01
provides: the commit gate fired at the three ref-advancing/ref-mutating PUBLIC entry points that bypass the plain-commit path, each gated for what it ACTUALLY mutates AND reading governance from the ref that carries it — worktree_integrate (advances target -> branch.protected via direct_ref(target)); branch::apply + apply_branch_integration (ref-mutating but no protected-trunk advancement -> contents:write/perm.denied via config_only(workspace target ref)) — the SAME contents:write + (where applicable) branch-protection decision as GATES-001, workspace-target-ref-pinned, DryRun-enforced on the integrate path, surfaced as branch.protected/perm.denied/config.invalid; mechanism parity proven (one shared decision helper, per-file: branch.rs >= 2, worktree.rs >= 1)
consumes: commit::gate::enforce_commit_gate_for_target + CommitGateTarget::direct_ref (gate.rs:25) + CommitGateTarget::config_only (gate.rs:33) (GATES-001, crates/but-api/src/commit/gate.rs:25-39,55); the commit gate's classify_error {code,message} (gate.rs:81); but_authz::governance_present (the live opt-in discriminator, config.rs:44, exported lib.rs:14, called at gate.rs:60); the but-api PUBLIC branch.rs apply/integrate entry points (apply:643, apply_branch_integration:939) + the worktree.rs worktree_integrate (:54); ctx.project_meta()?.target_ref_or_err() for the workspace target ref (crates/but-core/src/ref_metadata.rs:357; field :17; precedent crates/but-api/src/branch.rs:810); ctx.repo.get() for the pre-guard repo handle (precedent crates/but-api/src/commit/gate.rs:50); but_authz::resolve_principal + authorize(ContentsWrite) + load_governance_config (consumed transitively through the commit gate, AUTHZ-002/003)
boundary_contracts:
  - CAP-AUTHZ-01: each of the three PUBLIC entry points resolves BUT_AGENT_HANDLE->Principal and authorizes contents:write at the but-api boundary BEFORE the exclusive_worktree_access guard, EVEN under DryRun on the integrate path; unknown/no-handle/missing-authority -> perm.denied. worktree_integrate additionally rejects branch.protected when its target is a protected ref.
  - CAP-CONFIG-01: governance for the gated operation is read ONLY from the ref that carries it — the advanced trunk (worktree_integrate's target) or the workspace target ref (apply/integrate, via ctx.project_meta()?.target_ref_or_err()?) — so a working-tree/feature-head edit cannot weaken it; an ungoverned target ref is allowed (opt-in by presence via but_authz::governance_present).

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-api/src/legacy/worktree.rs (MODIFY) — fire enforce_commit_gate_for_target(&repo, &CommitGateTarget::direct_ref(target.clone())) at the TOP of the PUBLIC worktree_integrate (:54), resolving the gate target from its target FullName (the ref actually advanced AND where governance is committed), via ctx.repo.get()?, BEFORE the exclusive_worktree_access guard (:59)
  - crates/but-api/src/branch.rs (MODIFY) — fire the gate at the TOP of the PUBLIC apply (:643) and the PUBLIC apply_branch_integration (:939) via the S9b MATCH form `if let Ok(target_ref) = ctx.project_meta()?.target_ref_or_err() { enforce_commit_gate_for_target(&repo, &CommitGateTarget::config_only(target_ref.clone()))?; }`, via ctx.repo.get()?, BEFORE ctx.exclusive_worktree_access() (:647 / :946) and before delegating to the *_with_perm body; the config_ref is the WORKSPACE TARGET ref (trunk carrying governance) NOT the feature branch; MATCH (not ?-propagate) target_ref_or_err so a NO-default-target repo (DefaultTargetNotFound, ref_metadata.rs:357-362) is ungoverned = permit (SKIP the gate, parity with commit_gate_absent_config_is_ungoverned, commit_gate.rs:255-289; apply.rs:44-46) rather than a hard-error (S9b); this enforces contents:write + fail-closed WITHOUT a vacuous protected-branch assertion (apply/integrate do not advance a protected trunk); the gate runs before the guard so DryRun on the integrate path never bypasses it
  - crates/but-api/src/commit/gate.rs (MODIFY — minimal, conditional) — direct_ref (:25) and config_only (:33) already exist + are pub; only add a constructor if the call sites need a different shape (unlikely); do NOT change the gate decision logic (OWNED by GATES-001)
  - crates/but-api/tests/commit_gate.rs (MODIFY) — add the worktree-integrate branch.protected case + the apply/integrate contents:write (perm.denied) cases + the DryRun-no-bypass case against real but-api + real git
writeProhibited:
  - crates/but-api/src/commit/create.rs — the plain-commit wrapper gate is GATES-001; do not re-gate it (it IS the gate-before-guard precedent: gate at :35, guard at :38)
  - crates/but-authz/** — consume governance_present/resolve_principal/authorize/load_governance_config transitively through the commit gate; do NOT modify the primitive
  - crates/but-core/src/ref_metadata.rs — target_ref_or_err (:357) already exists + is pub; CONSUME it, do NOT modify it
  - crates/gitbutler-branch-actions/src/{actions.rs,branch_upstream_integration.rs} — the gate fires at the but-api boundary (branch.rs/worktree.rs), NOT in the legacy crate; do NOT add gitbutler-* coupling (crates/AGENTS.md)
  - crates/but-worktrees/** — the gate fires at the but-api worktree.rs boundary, not inside but_worktrees::integrate
  - crates/but-workspace/** — the gate fires at the but-api branch.rs boundary, not inside but_workspace::branch::apply/integrate
  - crates/but-api/src/legacy/merge_gate.rs + review_requirement.rs — the merge gate + evaluator are GATES-003/005/006/AUTHZ-004
  - crates/but-error/src/lib.rs — the gate codes are commit-gate-owned &'static str (GATES-001); no Code variants
  - Any file not listed in write_allowed

--------------------------------------------------------------------------------
OUT OF SCOPE
--------------------------------------------------------------------------------
  - The plain-commit gate at the but-api commit wrapper + the `but` CLI commit path is OWNED by GATES-001 (already landed). This task does NOT re-gate the commit wrapper; it reuses GATES-001's enforce_commit_gate_for_target decision helper at the worktree/apply/integrate PUBLIC entry points.
  - GROUNDING CORRECTION (R1, FABRICATED-GROUNDING, now corrected): this task's prior output cited `but_authz::governance_present` as missing and claimed the live opt-in was `has_governance_marker` at gate.rs:161-179. That was INVERTED. VERIFIED against live code: `but_authz::governance_present` EXISTS (crates/but-authz/src/config.rs:44), is re-exported (crates/but-authz/src/lib.rs:14), and IS the live opt-in discriminator, invoked INSIDE enforce_commit_gate_for_target (gate.rs:60: `if !but_authz::governance_present(repo, full_name)?`). `has_governance_marker` has ZERO matches in crates/. gate.rs is 170 lines total; :159-170 is `branch_protected()`. Reuse enforce_commit_gate_for_target (gate.rs:55) — that part of the prior output was correct.
  - GROUNDING CORRECTION (S9, config-ref points at the wrong branch, now corrected): a prior output wired `config_only(existing_branch)` / `config_only(branch)` for apply / apply_branch_integration — the FEATURE branch. Governance lives at the workspace TARGET ref (e.g. main), NOT on a feature branch, so `governance_present(feature_ref)` returns FALSE and the gate exits `Ok()` early (gate.rs:60-62) — the contents:write check never runs and a read-only principal is PERMITTED ungated (vacuous gate). CORRECTED: the config_ref for apply + apply_branch_integration is the WORKSPACE TARGET ref via `ctx.project_meta()?.target_ref_or_err()` (crates/but-core/src/ref_metadata.rs:357; field :17) — MATCHED, not `?`-propagated (see S9b below: a no-default-target repo must be ungoverned = permit, not a hard-error). worktree_integrate's `target` is the advanced trunk and already carries governance, so direct_ref(target) is correct there.
  - GROUNDING CORRECTION (S9b, no-target apply hard-errors, accepted + corrected): the prior S9 fix wired the apply / apply_branch_integration gate as `config_only(ctx.project_meta()?.target_ref_or_err()?.clone())`. But `target_ref_or_err()` returns `Err(but_error::Code::DefaultTargetNotFound)` when the workspace has NO configured default target (`target_ref: Option<gix::refs::FullName>` is None — crates/but-core/src/ref_metadata.rs:357-362), and the `?` PROPAGATES it — so a `branch::apply` / `apply_branch_integration` on a repo with no default target HARD-ERRORS before doing any work. That is a LEGITIMATE ungoverned operation today: crates/but-workspace/src/branch/apply.rs:44-46 documents "if there is no target … the current branch and the given one will be applied", and the existing plain-commit test `commit_gate_absent_config_is_ungoverned` (crates/but-api/tests/commit_gate.rs:255-289) drives a no-default-target repo and SUCCEEDS. The plain-commit gate (GATES-001) never had this dependency — it derives the config_ref from the RelativeTo arg, never calling target_ref_or_err. CORRECTED: MATCH the accessor — `if let Ok(target_ref) = ctx.project_meta()?.target_ref_or_err() { enforce_commit_gate_for_target(&repo, &CommitGateTarget::config_only(target_ref.clone()))?; }` — so a missing target = ungoverned = permit (SKIP the gate, parity with commit_gate_absent_config_is_ungoverned), NOT a hard-error; only a target that EXISTS but carries no committed `.gitbutler/*.toml` is the governance_present opt-in case. `ctx.project_meta()?` may still `?`-propagate a genuine metadata-read I/O error; ONLY the DefaultTargetNotFound case is the ungoverned-permit path. worktree_integrate is NOT affected — it uses its own `target` arg via `direct_ref(target.clone())`, never target_ref_or_err. (cite apply.rs:44-46; commit_gate.rs:255-289 commit_gate_absent_config_is_ungoverned; ref_metadata.rs:357-362 DefaultTargetNotFound.)
  - GROUNDING CORRECTION (R2b, gate placed after the guard, now corrected): a prior output placed the gate "at the TOP of `apply_with_perm` (:658)" / "`apply_branch_integration_with_perm` (:957)". But the PUBLIC `apply`/`apply_branch_integration` acquire the guard (`ctx.exclusive_worktree_access()`, :647 / :946) BEFORE delegating to the `_with_perm` body — so the `_with_perm` top runs UNDER the guard (a lock-ordering violation the CRITICAL CONSTRAINT + AC-1 negative control forbid). CORRECTED: the gate fires in the PUBLIC `apply` (:643) and PUBLIC `apply_branch_integration` (:939), BEFORE the guard (:647 / :946), via `ctx.repo.get()?`, mirroring create.rs:35 (gate) then :38 (guard). worktree_integrate IS the public seam; its gate fires before its own guard at :59.
  - UPSTREAM ADVISORY (R2/S2): ROADMAP/SPRINT Sprint-04 gate STEP 6 ("Run integrate_branch_with_steps / branch apply advancing protected main as contents:write -> rejected branch.protected") assumes a flow the live code CONTRADICTS: `but_workspace::branch::apply` ADDS a branch to the workspace and bails on the target ("Cannot add the target '{branch}' branch to its own workspace", crates/but-workspace/src/branch/apply.rs:232); `integrate_branch_with_steps` integrates upstream INTO the local/feature branch (it does not advance a protected trunk). Only `worktree_integrate` advances `target` (but-worktrees/src/integrate.rs:55-57). The honest local proof this task delivers is: contents:write ENFORCED on apply/integrate (read-only denied perm.denied, governance read from the workspace target ref) + worktree_integrate advancing a protected target -> branch.protected. RECOMMEND the orchestrator reconcile ROADMAP/SPRINT step 6 via `/kb-sprint-plan --delta-replan` (precedent: the GATES-003/005 forge-locality re-scope). This task does NOT edit ROADMAP/SPRINT.
  - The CLI surfacing of worktree/apply/integrate denials (a `but` snapbox) is OPTIONAL and out of the primary scope — the integration proof drives the but-api functions directly (the desktop/lite callers consume the same but-api boundary).
  - The merge gate, the review-requirement evaluator, and the per-group strictness matrix are GATES-003/005/006/AUTHZ-004 — not this task.
  - The `but worktree new` COMMIT path funnels through the plain-commit narrow waist already gated by GATES-001; this task gates the worktree INTEGRATE entry point (worktree_integrate) which advances a target ref outside the commit wrapper. The human-gate step 5 (commit on `but worktree new`) is the commit-path parity already provided by GATES-001; the worktree-integrate branch.protected proof is THIS task (step 6, re-grounded per the advisory above).

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-api/src/commit/gate.rs (16-94)
   Focus: THE DECISION HELPER TO REUSE — `enforce_commit_gate_for_target(repo, target)` (55) is the public, ref-aware decision: it checks `but_authz::governance_present(repo, full_name)?` (60 — the LIVE opt-in discriminator, config.rs:44, NOT has_governance_marker) and RETURNS Ok EARLY (61-62) if governance is NOT present on `config_ref` — THIS is why config_ref MUST be the governance-carrying ref (the target trunk), not a feature branch; then it loads target-ref config (64), resolves the principal from BUT_AGENT_HANDLE (65), authorizes ContentsWrite (67), and rejects a protected branch (69-75) ONLY when target.protected_branch is Some. `CommitGateTarget::direct_ref(config_ref: FullName)` (25) sets protected_branch=Some(name) so the protection check fires — USE FOR worktree_integrate (config_ref=target, the advanced trunk). `CommitGateTarget::config_only(config_ref)` (33) sets protected_branch=None so ONLY contents:write fires — USE FOR apply / apply_branch_integration (config_ref=the WORKSPACE TARGET ref, NOT the feature branch). `enforce_commit_gate` (48) obtains the pre-guard repo via `ctx.repo.get()?` (50) — same handle you use at the apply/integrate/worktree seams. `classify_error` (81) maps Denial/ConfigError -> {code,message}. WIRE this helper at the call sites — do NOT re-implement it. NOTE gate.rs is 170 lines total; :159-170 is branch_protected() (NOT an opt-in marker).
2. crates/but-api/src/branch.rs (587-679, 810, 930-985)
   Focus: THE APPLY/INTEGRATE SEAMS — the PUBLIC `apply` (643) acquires the guard via `ctx.exclusive_worktree_access()` (647) THEN delegates to `apply_with_perm` (658, takes existing_branch: &FullNameRef, NO DryRun) -> `apply_only_with_perm` (607, the mutation). The PUBLIC `apply_branch_integration` (939) acquires the guard (946) THEN delegates to `apply_branch_integration_with_perm` (957, takes branch: &FullNameRef AND dry_run: DryRun at :961) which runs `integrate_branch_with_steps` (972) inside `branch_mutation_with_snapshot`. Because the guard is taken in the PUBLIC fn BEFORE delegation, fire the gate at the TOP of the PUBLIC `apply` (643) and PUBLIC `apply_branch_integration` (939) BEFORE the guard, NOT in the `_with_perm` bodies (which run under the guard). The config_ref is the WORKSPACE TARGET ref — see line 810 (`ctx.project_meta()?.target_commit_id_or_err()?`) for the sibling-accessor precedent; MATCH (do NOT `?`-propagate) it: `if let Ok(target_ref) = ctx.project_meta()?.target_ref_or_err() { … config_only(target_ref.clone()) … }` (S9b — a no-default-target repo returns Err(DefaultTargetNotFound) and MUST be ungoverned = permit, not a hard-error; cf. apply.rs:44-46 + commit_gate.rs:255-289). NEITHER apply nor integrate advances a protected trunk — gate them config_only (contents:write only).
3. crates/but-api/src/legacy/worktree.rs (52-64)
   Focus: THE WORKTREE INTEGRATE SEAM — `worktree_integrate(ctx, id, target: gix::refs::FullName)` (54) IS the public seam; it takes the guard at 59 (first body line) then calls `but_worktrees::integrate::worktree_integrate`. Fire the gate BEFORE the guard at :59, via `ctx.repo.get()?` for the repo handle, with direct_ref(target.clone()) — `target` IS the workspace trunk that advances AND carries governance.
4. crates/but-worktrees/src/integrate.rs (44-96)
   Focus: PROOF worktree_integrate ADVANCES target — `worktree_integrate` (58) doc: "the worktree's state is squashed into a single commit which becomes the new tip of `target`" (55-57); it materializes a rebase that updates refs (89-91). THIS is why direct_ref(target) (branch.protected) is sound here and nowhere else.
5. crates/but-workspace/src/branch/apply.rs (225-233)
   Focus: PROOF branch::apply does NOT advance a protected trunk — `apply` bails `"Cannot add the target '{branch}' branch to its own workspace"` (232) when the branch IS the target; it ADDS a branch to the workspace. Hence config_only(<workspace target ref>) (contents:write only), NOT direct_ref, on the apply seam.
6. crates/but-core/src/ref_metadata.rs (15-19, 354-362)
   Focus: THE WORKSPACE-TARGET-REF ACCESSOR — `ProjectMeta` (struct at :15) holds `target_ref: Option<gix::refs::FullName>` (:17); `target_ref_or_err()` (:357-362) returns `&gix::refs::FullName`, or `Err(but_error::Code::DefaultTargetNotFound)` when `target_ref` is None (no configured default target). Obtain it at the boundary by MATCHING (S9b — do NOT `?`-propagate): `if let Ok(target_ref) = ctx.project_meta()?.target_ref_or_err() { … config_only(target_ref.clone()) … }` — a no-default-target repo is a LEGITIMATE ungoverned apply (apply.rs:44-46; commit_gate_absent_config_is_ungoverned, commit_gate.rs:255-289), so SKIP the gate and proceed (permit) rather than hard-error. USE THIS (the workspace target ref) as the config_ref for apply + apply_branch_integration when a target exists — NOT the feature branch.
7. crates/but-api/src/commit/create.rs (25-49)
   Focus: THE GATE-BEFORE-GUARD PRECEDENT — `commit_create_only` (27) calls `gate::enforce_commit_gate(ctx, &relative_to)?` (35) FIRST, then `ctx.exclusive_worktree_access()` (38). Mirror this ordering at the apply/integrate/worktree PUBLIC seams: gate, then guard. (Note: create.rs:35 uses the ctx-based `enforce_commit_gate`; this task uses `enforce_commit_gate_for_target(&repo, &target)` with an explicit `ctx.repo.get()?` repo handle so it can build the CommitGateTarget from the workspace target ref / the worktree target.)
8. .spec/prds/governance/tasks/sprint-01a-authz-primitive-commit-gate/GATES-001-commit-gate.md (full)
   Focus: THE GATE THIS TASK EXTENDS — GATES-001 owns the commit-gate decision (contents:write + branch protection, target-ref-pinned, DryRun-enforced, opt-in by presence via governance_present). Its OUT-OF-SCOPE defers T-GATES-016 (mechanism parity) + T-GATES-017 (worktree path) to Sprint 04 — i.e. THIS task. Mirror its fixture shapes (gated_repo) + its target-ref-pin / DryRun assertions. Its `commit_gate_edit_cannot_unprotect` (commit_gate.rs:102) is the commit-path target-ref-only precedent.
9. .spec/prds/governance/06-uc-gates.md (20-33)
   Focus: UC-GATES-01 AC-5 (the gate applies identically regardless of branching mechanism) + AC-9 (the worktree integrate path is gated with the same decision as the virtual-branch path). NOTE the re-grounded reading: the SAME decision helper fires across mechanisms; the branch.protected outcome only manifests where a protected ref is actually advanced (worktree_integrate).
10. .spec/prds/governance/11-e2e-testing-criteria.md (109-110)
   Focus: T-GATES-016 (commit gate applies identically across branching mechanisms — proven here as the SAME helper + contents:write on apply/integrate) + T-GATES-017 (commit gate covers the opt-in worktree path — proven here as branch.protected on worktree_integrate).
11. crates/but-api/tests/commit_gate.rs (full) + crates/but-testsupport/src/lib.rs (71-97)
   Focus: THE TEST HARNESS TO EXTEND — the existing commit_gate integration tests build a governed repo via writable_scenario + invoke_bash (seed `.gitbutler/*.toml` at refs/heads/main, branch to feat), set BUT_AGENT_HANDLE via temp_env, and assert ref-unchanged via `repo.find_reference(..).peel_to_id()`. `commit_gate_edit_cannot_unprotect` (:102) is the target-ref-pin shape. For apply/integrate, ALSO set the project's default target to main (so `target_ref_or_err()` resolves) — see crates/but-api/src/branch.rs:1205-1212 for the `set_default_target` test shape. Add the worktree/apply/integrate cases in the same shape; NEVER std::env::temp_dir().join(...).
12. crates/AGENTS.md
   Focus: the legacy `gitbutler-*` caution — gate at the but-api boundary (branch.rs/worktree.rs), do NOT add new coupling into gitbutler-branch-actions, but-worktrees, or but-workspace.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- worktree-integrate branch.protected integration test passes: `cargo test -p but-api commit_gate_worktree_integrate_protected_rejected`  -> Exit 0; worktree_integrate onto protected main branch.protected (target unchanged), feature-target integrate permitted
- apply/integrate contents:write denial passes: `cargo test -p but-api commit_gate_apply_integrate_readonly_denied`  -> Exit 0; perm.denied naming contents:write on the PUBLIC branch::apply AND apply_branch_integration for ro (config read from the workspace target ref); dev feature-branch op permitted; NO vacuous branch.protected on apply/integrate
- (S9b) no-target apply/integrate is ungoverned = permit (NOT a hard-error): `cargo test -p but-api commit_gate_apply_integrate_no_target_ungoverned`  -> Exit 0; on a repo with NO configured default target (DefaultTargetNotFound), branch::apply AND apply_branch_integration by ro are PERMITTED (gate SKIPPED — target_ref_or_err MATCHED, not ?-propagated), no hard-error surfaced (parity with commit_gate_absent_config_is_ungoverned)
- DryRun-no-bypass + target-ref pin: `cargo test -p but-api commit_gate_apply_integrate_dryrun_targetref_pinned`  -> Exit 0; DryRun=Yes apply_branch_integration still denied + persists nothing; working-tree edit cannot weaken
- One shared decision helper (mechanism parity, PER-FILE): `./tools/governance-checks/check_gate_before_guard.py` -> Exit 0. A loose cross-file SUM is INSUFFICIENT — it could pass with worktree_integrate ungated (e.g. 3 in branch.rs, 0 in worktree.rs).
- Gate fires BEFORE the guard at the PUBLIC seam: in crates/but-api/src/branch.rs the `enforce_commit_gate_for_target(...)` call in `apply`/`apply_branch_integration` precedes `ctx.exclusive_worktree_access()`; in crates/but-api/src/legacy/worktree.rs it precedes the `let mut guard = ctx.exclusive_worktree_access();` at :59 (mirrors create.rs:35 before :38). NO gate call inside `apply_with_perm` / `apply_branch_integration_with_perm` bodies.
- Config ref is the workspace target ref (NOT the feature branch): `grep -nE 'config_only\(.*existing_branch|config_only\(.*\bbranch\b' crates/but-api/src/branch.rs`  -> No matches (the apply/integrate config_ref is the matched `target_ref` from `ctx.project_meta()?.target_ref_or_err()`, not existing_branch/branch)
- (S9b) No-target apply MUST NOT hard-error: `! grep -nE 'target_ref_or_err\(\)\?' crates/but-api/src/branch.rs`  -> No matches in `apply`/`apply_branch_integration` (the accessor is MATCHED via `if let Ok(target_ref) = ctx.project_meta()?.target_ref_or_err()`, NOT `?`-propagated — a no-default-target repo is ungoverned = permit, parity with commit_gate_absent_config_is_ungoverned, not a hard-error)
- No new gitbutler-* coupling for the gate: `! grep -rEn 'gitbutler_branch_actions|but_worktrees::integrate::worktree_integrate.*authorize' crates/but-api/src/branch.rs`  -> No matches (the gate fires via the commit-gate helper at the but-api boundary, not inside the legacy crate)
- No Permission-lock overload as authority carrier: `! grep -rEn 'write_permission\(\)\s*[,)].*authorize|RepoExclusive.*Authority' crates/but-api/src/branch.rs crates/but-api/src/legacy/worktree.rs`  -> No matches — the gate uses the but-authz Authority axis via the commit-gate helper, evaluated before the guard
- No fabricated opt-in marker citation: `! grep -rEn 'has_governance_marker' crates/but-api/src`  -> No matches (the live opt-in is but_authz::governance_present, gate.rs:60)
- Crate compiles incl. tests: `cargo check -p but-api --all-targets`  -> Exit 0
- Clippy clean: `cargo clippy -p but-api --all-targets`  -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: Reuse-the-commit-gate-decision-at-every-ref-advancing-PUBLIC-seam, gated for what each entry point ACTUALLY mutates AND reading governance from the ref that carries it — at the TOP of each PUBLIC but-api entry point that mutates refs outside the plain-commit wrapper, call `commit::gate::enforce_commit_gate_for_target(&ctx.repo.get()?, &target)?` BEFORE acquiring the exclusive_worktree_access guard and before any mutation (mirroring create.rs:35 before :38). For `worktree_integrate` (which advances `target` and carries governance there), build `CommitGateTarget::direct_ref(target.clone())` so the branch-protection check fires (branch.protected). For the PUBLIC `apply` / `apply_branch_integration` (which do NOT advance a protected trunk — apply bails on the target, integrate writes a feature branch), MATCH the workspace-target-ref accessor and gate only when a target exists: `if let Ok(target_ref) = ctx.project_meta()?.target_ref_or_err() { enforce_commit_gate_for_target(&repo, &CommitGateTarget::config_only(target_ref.clone()))?; }` — the config_ref is the WORKSPACE TARGET ref (the trunk where `.gitbutler/*.toml` is committed), NOT the feature branch — so the gate reads REAL governance (governance_present is TRUE) and ONLY contents:write fires (perm.denied for a read-only principal) WITHOUT a vacuous protected-branch assertion. CRITICAL (S9b): MATCH, do NOT `?`-propagate — `target_ref_or_err()?` would turn a legitimate no-default-target apply (DefaultTargetNotFound, ref_metadata.rs:357-362; apply.rs:44-46) into a hard error; a missing target = ungoverned = permit (SKIP the gate), parity with the plain-commit gate's commit_gate_absent_config_is_ungoverned (commit_gate.rs:255-289). worktree_integrate uses `direct_ref(target.clone())` from its own arg and is NOT affected by S9b. The helper internally opts in via `but_authz::governance_present` (gate.rs:60, the live discriminator) and returns Ok EARLY if the config_ref carries no governance — so using the feature branch ref would make the gate vacuous; the workspace target ref makes it real. The anyhow error propagates unchanged so the boundary's classify_error yields the identical {code,message} contract; DryRun threads through the PUBLIC integrate path but the gate runs before the guard so it never bypasses (dedicated sub-case).
pattern_source: crates/but-api/src/commit/gate.rs:55 (enforce_commit_gate_for_target), :25 (direct_ref → protected_branch=Some), :33 (config_only → protected_branch=None), :50 (ctx.repo.get()? pre-guard handle), :60-62 (governance_present opt-in + early Ok) + crates/but-api/src/commit/create.rs:35 (gate) then :38 (guard) (gate-before-guard precedent) + crates/but-core/src/ref_metadata.rs:357-362 (target_ref_or_err, the workspace target ref accessor; field :17; returns Err(DefaultTargetNotFound) when no default target -> MATCH not ?-propagate, S9b) + crates/but-workspace/src/branch/apply.rs:44-46 (no-target apply is legitimate/ungoverned) + crates/but-api/tests/commit_gate.rs:255-289 (commit_gate_absent_config_is_ungoverned — the no-target ungoverned-permit precedent) + crates/but-api/src/branch.rs:643 (PUBLIC apply, guard at :647),:939 (PUBLIC apply_branch_integration, guard at :946),:810 (project_meta sibling-accessor precedent) + crates/but-api/src/legacy/worktree.rs:54 (PUBLIC worktree_integrate, guard at :59, advances target) + crates/but-worktrees/src/integrate.rs:55-57 (proof worktree_integrate advances target) + crates/but-workspace/src/branch/apply.rs:232 (proof apply bails on the target)
anti_pattern: `?`-propagating `ctx.project_meta()?.target_ref_or_err()?` for apply / apply_branch_integration (S9b — a repo with NO configured default target returns Err(DefaultTargetNotFound) (ref_metadata.rs:357-362), so the `?` HARD-ERRORS a LEGITIMATE no-target apply (apply.rs:44-46) before any work — a correctness/availability regression; MATCH it instead: `if let Ok(target_ref) = … { … }` so no-target = ungoverned = permit, parity with commit_gate_absent_config_is_ungoverned, commit_gate.rs:255-289); using the FEATURE branch (existing_branch / branch) as the config_only config_ref (S9 — governance lives at the workspace target ref, not the feature branch; governance_present(feat)=false so the gate exits Ok ungated and a read-only principal is PERMITTED — a VACUOUS gate); placing the gate at the TOP of `apply_with_perm` / `apply_branch_integration_with_perm` (R2b — those bodies run UNDER the guard taken in the PUBLIC fn at :647 / :946, a lock-ordering violation); a loose cross-file grep SUM for the parity build-gate (S10 — could pass with worktree.rs at 0); asserting "apply/integrate advancing protected main -> branch.protected" (INCOHERENT against the live code — apply bails on the target, integrate writes a feature branch); using direct_ref on apply/integrate (a vacuous protected-branch assertion — they do not advance a protected trunk); re-implementing contents:write/branch-protection at the call sites instead of calling enforce_commit_gate_for_target (parallel gates that drift); gating fewer than the three entry points (the others bypass); acquiring the guard BEFORE the gate (lock-ordering violation); reading governance from the working tree / feature head instead of the workspace target ref; early-returning on DryRun before the gate on the integrate path (CAP-AUTHZ-01 violation); adding the gate inside gitbutler-branch-actions / but_worktrees / but_workspace (new coupling, crates/AGENTS.md); a hardcoded protected-branch list; or citing a non-existent `has_governance_marker` / `gate.rs:161-179` opt-in (the live discriminator is but_authz::governance_present at gate.rs:60).

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — Makes GitButler's commit gate actually mechanism-agnostic by firing the EXISTING commit-gate decision (enforce_commit_gate_for_target) at three ref-mutating but-api PUBLIC entry points, each gated for what it ACTUALLY mutates AND reading governance from the ref that carries it: worktree_integrate (advances target -> branch.protected via direct_ref(target)), and the PUBLIC branch::apply + apply_branch_integration (ref-mutating but no protected-trunk advancement -> contents:write/perm.denied via config_only(workspace target ref), the target ref obtained by MATCHING ctx.project_meta()?.target_ref_or_err() — a no-default-target repo is ungoverned = permit, NOT a hard-error, S9b). Wires the gate at the PUBLIC seam BEFORE the exclusive_worktree_access guard via ctx.repo.get()? (mirroring create.rs:35 before :38), workspace-target-ref-pinned, with a dedicated DryRun-no-bypass proof on the integrate path, proving cross-mechanism parity against real but-api + real git without adding new gitbutler-* coupling. Owns the boundary wiring, the config-ref-is-the-target-ref discipline, the gate-before-guard lock-ordering, and the parity integration tests.
reviewer: rust-reviewer
coding_standards: crates/AGENTS.md, crates/WORKSPACE_MODEL.md, crates/but-api/src/commit/gate.rs, /Users/justinrich/Projects/brain/docs/rust/error-handling.md

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: GATES-001, AUTHZ-002, AUTHZ-003   (the commit-gate decision helper + the but-authz primitive incl. governance_present)
Blocks:     Sprint 05, Sprint 06b
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "GATES-007",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "notes": [
    "R1 GROUNDING CORRECTION (FABRICATED-GROUNDING, accepted + corrected): the prior output INVERTED the opt-in discriminator. VERIFIED against live code: but_authz::governance_present EXISTS (crates/but-authz/src/config.rs:44), is re-exported (crates/but-authz/src/lib.rs:14), and IS the live opt-in discriminator, invoked inside enforce_commit_gate_for_target (crates/but-api/src/commit/gate.rs:60: `if !but_authz::governance_present(repo, full_name)?`). has_governance_marker has ZERO matches in crates/. gate.rs is 170 lines total; :159-170 is branch_protected(). Every reference (NOTES, OUT-OF-SCOPE, anti_pattern, pattern, READING LIST) now states governance_present; all has_governance_marker / gate.rs:161-179 citations are deleted. Reuse of enforce_commit_gate_for_target is preserved (it was correct).",
    "S9 GROUNDING CORRECTION (CRITICAL, config_ref points at the wrong branch, accepted + corrected): the prior output wired CommitGateTarget::config_only(existing_branch) for apply and config_only(branch) for apply_branch_integration — the FEATURE branch being applied/integrated. But governance lives at the workspace TARGET ref (e.g. main), NOT on a feature branch, so governance_present(repo, feature_ref) returns FALSE -> enforce_commit_gate_for_target exits Ok EARLY (gate.rs:60-62) -> the contents:write check NEVER runs -> a read-only (ro) principal is PERMITTED ungated (vacuous gate). VERIFIED accessor: the workspace target ref is ctx.project_meta()?.target_ref_or_err()? returning &gix::refs::FullName (crates/but-core/src/ref_metadata.rs:357; field target_ref: Option<gix::refs::FullName> at :17), with a but-api boundary precedent at crates/but-api/src/branch.rs:810 (sibling ctx.project_meta()?.target_commit_id_or_err()?). CORRECTED (superseded by S9b — see below): config_only for apply + apply_branch_integration uses the WORKSPACE TARGET ref from ctx.project_meta()?.target_ref_or_err() (MATCHED, not ?-propagated, per S9b) so governance_present(target_ref)=TRUE and contents:write is genuinely authorized; the AC-2 fixture description now states governance is read from the target ref (so ro is actually denied perm.denied); the AC-2 and AC-3 negative_control.would_fail_if both add 'config_only uses the feature branch ref so governance_present(feat)=false and the gate exits Ok ungated, permitting ro'. worktree_integrate's target IS the advanced trunk and carries governance, so direct_ref(target) is correct there (config_ref==target). (CAP-CONFIG-01: governance read at the target ref.)",
    "R2b GROUNDING CORRECTION (MEDIUM, gate placed after the guard, accepted + corrected): the prior output said fire the gate 'at the TOP of apply_with_perm (:658)' / 'apply_branch_integration_with_perm (:957)'. VERIFIED: the PUBLIC apply (branch.rs:643) acquires the guard via ctx.exclusive_worktree_access() (:647) THEN calls apply_with_perm (:648) with the guard ALREADY HELD; the PUBLIC apply_branch_integration (branch.rs:939) acquires the guard (:946) THEN calls apply_branch_integration_with_perm (:947) — so the _with_perm tops run UNDER the guard, the exact lock-ordering violation the CRITICAL CONSTRAINT + AC-1 negative control forbid. CORRECTED: the gate now fires at the TOP of the PUBLIC apply (:643) and PUBLIC apply_branch_integration (:939), BEFORE the guard (:647 / :946), via ctx.repo.get()? (pre-guard handle, precedent gate.rs:50), mirroring the commit-gate precedent create.rs:35 (gate) then :38 (guard). worktree_integrate (legacy/worktree.rs:54) IS itself the public seam — its guard exclusive_worktree_access() is its FIRST body line at :59, so the gate fires before :59. The inverted line-62/_with_perm-top claims are corrected everywhere (What/How/Scope, CRITICAL constraint, reading list, writeAllowed, pattern, anti_pattern, AC GIVEN/WHEN, negative controls). writeAllowed now names the PUBLIC apply / apply_branch_integration / worktree_integrate functions. (No direct _with_perm-caller-already-holds-a-guard re-fire is needed here: the public seams are the sole callers of interest for this task, so the gate lives in the public functions before the guard — not mislabeled 'before the guard' inside a _with_perm body.)",
    "S10 GROUNDING CORRECTION (MEDIUM, parity grep too loose, accepted + corrected): the prior AC-3/TC verify summed enforce_commit_gate_for_target across branch.rs + worktree.rs and required >=3, which could pass with worktree_integrate UNGATED (e.g. 3 in branch.rs, 0 in worktree.rs). CORRECTED: the parity build-gate is now PER-FILE — crates/but-api/src/branch.rs >= 2 (apply + apply_branch_integration public seams) AND crates/but-api/src/legacy/worktree.rs >= 1 (worktree_integrate). AC-3 verify, the VERIFICATION GATES, OUTCOME, DONE WHEN, CAPABILITY BOUNDARY, TC-7, and the AC-3 scenario must_observe are all updated to the per-file assertion.",
    "S4 DryRun isolation: apply_branch_integration is the ONLY one of the three whose path carries a dry_run: DryRun param (forwarded from the PUBLIC apply_branch_integration into apply_branch_integration_with_perm at branch.rs:961). AC-3 / TC-5 isolate the DryRun-no-bypass proof as a DEDICATED sub-case with its own negative control (a gate that early-returns Ok on DryRun=Yes passes the non-DryRun cases but FAILS this). The public apply and worktree_integrate have NO DryRun param — stated explicitly so the implementer does not invent one. Because the gate fires in the PUBLIC apply_branch_integration BEFORE the guard, it runs regardless of dry_run.",
    "UPSTREAM ADVISORY (for the orchestrator): ROADMAP/SPRINT Sprint-04 gate step 6 ('Run integrate_branch_with_steps / branch apply advancing protected main as contents:write -> rejected branch.protected') assumes a flow the code contradicts (apply bails on the target; integrate writes a feature branch). Recommend reconciling via /kb-sprint-plan --delta-replan (precedent: GATES-003/005 forge-locality re-scope). The honest local proof is: contents:write enforced on apply/integrate (read-only denied, governance read from the workspace target ref) + worktree_integrate advancing a protected target -> branch.protected. This task does NOT edit ROADMAP/SPRINT.",
    "BOUNDARY: the gate fires at the but-api boundary (branch.rs / worktree.rs), NOT inside gitbutler-branch-actions, but-worktrees, or but-workspace — per crates/AGENTS.md legacy caution.",
    "CommitGateTarget::direct_ref (gate.rs:25, protected_branch=Some) + config_only (gate.rs:33, protected_branch=None) both already exist and are pub; ctx.project_meta()?.target_ref_or_err() (ref_metadata.rs:357) + ctx.repo.get() (gate.rs:50 precedent) both already exist — no commit/gate.rs or ref_metadata.rs production edit expected.",
    "S9b GROUNDING CORRECTION (MEDIUM, no-target apply hard-errors, accepted + corrected): the prior S9 fix wired the apply / apply_branch_integration gate as config_only(ctx.project_meta()?.target_ref_or_err()?.clone()). target_ref_or_err() returns Err(but_error::Code::DefaultTargetNotFound) when the workspace has NO configured default target (target_ref: Option<gix::refs::FullName> is None -- crates/but-core/src/ref_metadata.rs:357-362), and the `?` PROPAGATES it -- so a branch::apply / apply_branch_integration on a repo with no default target HARD-ERRORS before doing any work. That is a LEGITIMATE ungoverned operation today: crates/but-workspace/src/branch/apply.rs:44-46 documents 'if there is no target ... the current branch and the given one will be applied', and the existing plain-commit test commit_gate_absent_config_is_ungoverned (crates/but-api/tests/commit_gate.rs:255-289) drives a no-default-target repo and SUCCEEDS. The plain-commit gate (GATES-001) never had this dependency -- it derives the config_ref from the RelativeTo arg, never calling target_ref_or_err. It fails CLOSED (no unauthorized mutation) -> a correctness/availability regression. CORRECTED: MATCH the accessor for apply + apply_branch_integration -- if let Ok(target_ref) = ctx.project_meta()?.target_ref_or_err() { enforce_commit_gate_for_target(&repo, &CommitGateTarget::config_only(target_ref.clone()))?; } -- so no default target = ungoverned = permit (SKIP the gate, parity with commit_gate_absent_config_is_ungoverned), NOT a hard-error; only a target that EXISTS but carries no committed .gitbutler/*.toml is the governance_present opt-in case. ctx.project_meta()? may still ?-propagate a genuine metadata-read I/O error; ONLY the DefaultTargetNotFound (no-target) case is the ungoverned-permit path. worktree_integrate is NOT affected -- it uses its own target arg via direct_ref(target.clone()), never target_ref_or_err. Added fixture apply_repo_no_target + a no-target ungoverned-permit case to AC-2 (cases[]) + TC-8; every config_only(...target_ref_or_err()?...) form in the spec (What/Why/Scope, OUTCOME, CRITICAL constraint, pattern/anti_pattern, AC GIVEN/WHEN, writeAllowed, VERIFICATION GATES, READING LIST) replaced with the MATCH form. (cite apply.rs:44-46; commit_gate.rs:255-289 commit_gate_absent_config_is_ungoverned; ref_metadata.rs:357-362 DefaultTargetNotFound.)"
  ],
  "fixtures": {
    "gated_worktree_repo": {
      "description": "A real git repo (but-testsupport writable_scenario) whose target ref main has committed .gitbutler/permissions.toml ([[principal]] id=\"dev\" permissions=[\"contents:write\"]) and .gitbutler/gates.toml ([[branch]] name=\"main\" protected=true), plus a worktree set up so worktree_integrate(id, target) has a valid WorktreeId and a delta to squash onto the target. A non-protected feature target is also available. Used to drive worktree_integrate against protected main (denied branch.protected — direct_ref(target) reads governance from main, the advanced trunk) and against a feature target (permitted).",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"governance-base\");",
        "invoke_bash: write .gitbutler/permissions.toml ([[principal]] id=\"dev\" permissions=[\"contents:write\"])",
        "invoke_bash: write .gitbutler/gates.toml ([[branch]] name=\"main\" protected=true); git add -A && git commit -m \"governance config\" at refs/heads/main",
        "invoke_bash: create a non-protected feature target ref; create a linked worktree with a delta to squash; capture its WorktreeId",
        "build a but_ctx::Context from the repo; resolve worktree_integrate(id, target) inputs against the seeded repo"
      ]
    },
    "gated_apply_repo": {
      "description": "A real git repo (but-testsupport writable_scenario) whose WORKSPACE TARGET ref main has committed .gitbutler/permissions.toml ([[principal]] id=\"dev\" permissions=[\"contents:write\"]; [[principal]] id=\"ro\" permissions=[\"contents:read\"]) and .gitbutler/gates.toml ([[branch]] name=\"main\" protected=true), with the project's default target set to main (so ctx.project_meta()?.target_ref_or_err()? resolves to refs/heads/main), plus a feature branch `feat` carrying a change that can be applied/integrated. CRITICAL: the apply/integrate gate reads governance from the WORKSPACE TARGET ref (main), NOT from the feature branch — config_only(target_ref) makes governance_present(main)=TRUE so contents:write is enforced; using config_only(feat) would make governance_present(feat)=FALSE and the gate would exit Ok ungated. apply/integrate do NOT advance main (apply bails on the target; integrate writes the feature branch) — main protected=true here only governs the contents:write read of the target ref's config; the proof on apply/integrate is the contents:write authorization (ro -> perm.denied), NOT a protected-branch advancement. Used to drive the PUBLIC branch::apply + apply_branch_integration by ro (denied perm.denied) and by dev (permitted).",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"governance-base\");",
        "invoke_bash: write .gitbutler/permissions.toml ([[principal]] id=\"dev\" permissions=[\"contents:write\"]; [[principal]] id=\"ro\" permissions=[\"contents:read\"])",
        "invoke_bash: write .gitbutler/gates.toml ([[branch]] name=\"main\" protected=true); git add -A && git commit -m \"governance config\" at refs/heads/main",
        "invoke_bash: git checkout -b feat; commit a change so feat has an applicable/integratable delta; git checkout main",
        "build a but_ctx::Context from the repo; set the project default target to main (so target_ref_or_err() resolves to refs/heads/main — cf. branch.rs:1205-1212 set_default_target shape); resolve the apply (existing_branch=feat) / integrate (branch=feat + InteractiveIntegration steps) inputs against the seeded repo"
      ]
    },
    "gated_apply_repo_wt_unprotect": {
      "description": "Same as gated_apply_repo but the WORKING-TREE .gitbutler/gates.toml/permissions.toml is edited (NOT committed) to weaken governance / grant ro contents:write — to prove the apply/integrate decision reads the WORKSPACE TARGET-ref blob (main), not the working tree, so the edit cannot weaken the gate (ro is still denied perm.denied).",
      "seed_method": "cli",
      "records": [
        "reuse gated_apply_repo seeding (governance committed at refs/heads/main; default target=main; dev=contents:write, ro=contents:read)",
        "invoke_bash: overwrite the WORKING-TREE .gitbutler/gates.toml/permissions.toml to grant ro contents:write; do NOT git add/commit (the target-ref blob at main still denies ro)"
      ]
    },
    "apply_repo_no_target": {
      "description": "A real git repo (but-testsupport writable_scenario) with NO configured workspace default target (ProjectMeta.target_ref is None, so ctx.project_meta()?.target_ref_or_err() returns Err(but_error::Code::DefaultTargetNotFound), crates/but-core/src/ref_metadata.rs:357-362) and NO committed .gitbutler/*.toml anywhere, plus a feature branch `feat` carrying a change that can be applied/integrated. BUT_AGENT_HANDLE=ro. This is a LEGITIMATE ungoverned no-target operation (crates/but-workspace/src/branch/apply.rs:44-46: 'if there is no target ... the current branch and the given one will be applied'); the existing plain-commit test commit_gate_absent_config_is_ungoverned (crates/but-api/tests/commit_gate.rs:255-289) uses the same no-default-target shape and SUCCEEDS. Used to prove (S9b) that branch::apply / apply_branch_integration MATCH target_ref_or_err (not ?-propagate) so a missing default target SKIPS the gate (ungoverned = permit) and does NOT hard-error on DefaultTargetNotFound.",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"governance-base\");",
        "invoke_bash: do NOT write or commit any .gitbutler/*.toml; do NOT set a project default target (ProjectMeta.target_ref stays None so target_ref_or_err() returns Err(DefaultTargetNotFound))",
        "invoke_bash: git checkout -b feat; commit a change so feat has an applicable/integratable delta; git checkout main",
        "build a but_ctx::Context from the repo; resolve the apply (existing_branch=feat) / integrate (branch=feat) inputs against the seeded repo (target_ref_or_err() Err -> the gate is skipped, ungoverned)"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN gated_worktree_repo + BUT_AGENT_HANDLE=dev WHEN worktree_integrate is invoked targeting protected main, then targeting a non-protected feature target THEN the protected-main worktree-integrate is denied branch.protected naming main (main HEAD sha == base — worktree_integrate genuinely advances target, so direct_ref(target) makes the protection check fire and reads governance from main, the advanced trunk) and the feature-target worktree-integrate is permitted by the gate (no governance Denial). The gate fires at the TOP of the PUBLIC worktree_integrate body BEFORE its exclusive_worktree_access guard (:59), via ctx.repo.get()?",
      "verify": "cargo test -p but-api commit_gate_worktree_integrate_protected_rejected",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api worktree seam + real git",
        "negative_control": {
          "would_fail_if": [
            "worktree_integrate is ungated (a no-op) so the protected-main integrate advances main (ref advances)",
            "branch protection is read from the working tree rather than the target-ref blob (a mock config source)",
            "the gate is a static stub that returns Ok so the protected-main integrate lands",
            "the gate runs AFTER the exclusive_worktree_access guard (:59) (lock-ordering violation) so the mutation begins before denial",
            "config_only is wrongly used for worktree_integrate so protected_branch is None and the protection check is skipped (the protected-main integrate lands)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "gated_worktree_repo",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=dev: invoke but_api worktree_integrate(id, target=refs/heads/main) — the ref worktree_integrate genuinely advances AND carries governance (cite T-GATES-017, UC-GATES-01 AC-9; but-worktrees/src/integrate.rs:55-57)"
              ]
            },
            "end_state": {
              "must_observe": [
                "the worktree-integrate is denied `error.code == \"branch.protected\"`",
                "the denial `message` names the protected branch `\"main\"`",
                "`main` HEAD sha `==` the seeded base sha (the integrate did not advance main)"
              ],
              "must_not_observe": [
                "`main` HEAD sha advanced",
                "exit `0` / Ok outcome for the protected-main worktree-integrate",
                "the integrate landed on `main`",
                "no governance denial raised (the gate was a no-op)"
              ]
            }
          },
          {
            "start_ref": "gated_worktree_repo",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=dev: invoke worktree_integrate(id, target=<non-protected feature target>) (a contents:write principal onto a non-protected target)"
              ]
            },
            "end_state": {
              "must_observe": [
                "the gate PERMITS the feature-target worktree-integrate — the output contains NO `error.code == \"branch.protected\"` and NO `perm.denied` (0 governance denials; `classify_error(&result)` returns `None`)",
                "execution reaches the worktree_integrate body past the gate: `classify_error(&result) == None` (0 governance denials; any subsequent error is the operation's own)"
              ],
              "must_not_observe": [
                "`error.code == \"branch.protected\"` raised for a non-protected feature target",
                "`error.code == \"perm.denied\"` for a contents:write principal",
                "the gate over-rejecting a legitimate feature-target integrate (0 governance denials expected)"
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
      "description": "GIVEN gated_apply_repo (governance committed at the WORKSPACE TARGET ref main; default target=main) WHEN the PUBLIC branch::apply and the PUBLIC apply_branch_integration (DryRun=No) are each invoked by ro (contents:read only), then each by dev (contents:write) on feat THEN each ro operation is denied perm.denied naming contents:write (ro's edits never reach a ref — the ref HEAD sha == base) and each dev feature-branch operation is permitted by the gate (no governance Denial). The gate reads governance from the workspace target ref (config_only(ctx.project_meta()?.target_ref_or_err()?), NOT the feature branch — so governance_present(main)=TRUE and contents:write is genuinely enforced) and fires at the TOP of the PUBLIC apply/apply_branch_integration BEFORE the guard (:647 / :946). NO branch.protected is asserted on apply/integrate — they do not advance a protected trunk (apply bails on the target; integrate writes the feature branch) (S9b) ADDITIONALLY: on a repo with NO configured default target (apply_repo_no_target — target_ref_or_err() returns Err(DefaultTargetNotFound)), branch::apply AND apply_branch_integration by ro are PERMITTED because the gate is SKIPPED (the accessor is MATCHED, not ?-propagated, so a legitimate ungoverned no-target apply does NOT hard-error — parity with commit_gate_absent_config_is_ungoverned, commit_gate.rs:255-289).",
      "verify": "cargo test -p but-api commit_gate_apply_integrate_readonly_denied && cargo test -p but-api commit_gate_apply_integrate_no_target_ungoverned",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api branch seam + real git",
        "negative_control": {
          "would_fail_if": [
            "the apply/integrate path omits the contents:write check so ro's operation lands (ref advances)",
            "the denial is not perm.denied / does not name contents:write",
            "only one of the PUBLIC branch::apply / apply_branch_integration is gated so ro bypasses via the other",
            "a read-only principal is granted an implicit contents:write so the operation proceeds (a static allow)",
            "the gate vacuously asserts branch.protected on apply/integrate (a false guarantee — they do not advance a protected trunk; direct_ref wrongly used instead of config_only)",
            "config_only uses the feature branch ref (existing_branch / branch) so governance_present(feat)=false and the gate exits Ok ungated, permitting ro",
            "the gate is placed at the top of apply_with_perm / apply_branch_integration_with_perm (which run UNDER the guard taken in the PUBLIC fn at :647 / :946) — a lock-ordering violation",
            "(S9b) target_ref_or_err()? propagates DefaultTargetNotFound, turning a legitimate no-target apply into a hard error (the gate is not skipped on no-target) — a static/?-propagated accessor that hard-errors instead of matching, so the no-default-target apply_repo_no_target operation fails closed where it must be permitted"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "gated_apply_repo",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=ro: invoke the PUBLIC branch::apply (existing_branch=feat) AND the PUBLIC apply_branch_integration (branch=feat, DryRun=No) — ro holds contents:read only; the gate reads governance from the workspace target ref main, not feat (cite T-AUTHZ-011, T-GATES-016)"
              ]
            },
            "end_state": {
              "must_observe": [
                "each of the two ro operations is denied `error.code == \"perm.denied\"`",
                "the denial `message` names the missing `\"contents:write\"` authority",
                "the operation's ref HEAD sha `==` the seeded base sha (ro's edits never reached a ref) on both apply and integrate"
              ],
              "must_not_observe": [
                "the operation landed (ref advanced)",
                "exit `0` / Ok outcome for a contents:read-only principal",
                "ro granted an implicit contents:write",
                "a `branch.protected` denial vacuously asserted on apply/integrate (they do not advance a protected trunk)"
              ]
            }
          },
          {
            "start_ref": "gated_apply_repo",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=dev: invoke the PUBLIC branch::apply (existing_branch=feat) AND the PUBLIC apply_branch_integration (branch=feat, DryRun=No) — dev holds contents:write on the feature branch"
              ]
            },
            "end_state": {
              "must_observe": [
                "the gate PERMITS each dev feature-branch operation — NO `error.code == \"perm.denied\"` and NO `branch.protected` (0 governance denials; `classify_error(&result)` returns `None`)",
                "execution reaches the operation body past the gate on both apply and integrate: `classify_error(&result) == None` (0 governance denials on each)"
              ],
              "must_not_observe": [
                "`error.code == \"perm.denied\"` raised for a contents:write principal",
                "`error.code == \"branch.protected\"` for a feature-branch operation",
                "the gate over-rejecting a legitimate contents:write feature-branch operation (0 governance denials expected)"
              ]
            }
          },
          {
            "start_ref": "apply_repo_no_target",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=ro: on the NO-default-target repo, invoke the PUBLIC branch::apply (existing_branch=feat) AND the PUBLIC apply_branch_integration (branch=feat, DryRun=No) — ctx.project_meta()?.target_ref_or_err() returns Err(DefaultTargetNotFound) so the gate is SKIPPED (ungoverned = permit, S9b; cite apply.rs:44-46, commit_gate_absent_config_is_ungoverned commit_gate.rs:255-289)"
              ]
            },
            "end_state": {
              "must_observe": [
                "`ctx.project_meta()?.target_ref_or_err()` returns `Err(DefaultTargetNotFound)` so the gate is SKIPPED (matched, not `?`-propagated)",
                "`0` governance denials on the no-target apply AND integrate: `classify_error(&result) == None` (no `perm.denied`, no `branch.protected`, no hard-error surfaced from the gate)",
                "the no-target apply/integrate proceeds past the gate: feat ref advances OR the op reaches its own body with `classify_error(&result) == None` (0 governance denials — the gate was skipped, not a denial)"
              ],
              "must_not_observe": [
                "`error.code == \"...\"` hard-error / `DefaultTargetNotFound` surfaced to the caller from the gate wiring",
                "`0` operations completed because the apply is blocked by a governance denial",
                "the apply is blocked by a governance denial (`perm.denied` / `branch.protected`) on a no-target repo",
                "the gate `?`-propagates `DefaultTargetNotFound` and hard-errors the legitimate ungoverned no-target apply"
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
      "description": "GIVEN the PUBLIC apply_branch_integration under DryRun (it forwards dry_run: DryRun to apply_branch_integration_with_perm, branch.rs:961 — the ONLY of the three paths with a DryRun param) with BUT_AGENT_HANDLE=ro, and gated_apply_repo_wt_unprotect (working-tree gates.toml weakened, NOT committed) WHEN a DryRun=Yes apply_branch_integration by ro is attempted, a branch::apply by ro is attempted with the working-tree edit present, AND the source is inspected THEN the DryRun apply_branch_integration is STILL denied perm.denied and persists nothing (DEDICATED DryRun-no-bypass sub-case for apply_branch_integration; the gate runs in the PUBLIC fn before the guard so DryRun never bypasses), the working-tree-edited apply is still denied per the WORKSPACE TARGET-ref config (main), and all three entry points call the SAME commit::gate::enforce_commit_gate_for_target decision (PER-FILE build-gate: branch.rs >= 2 AND worktree.rs >= 1 — mechanism parity, not a loose cross-file sum that could pass with worktree.rs at 0)",
      "verify": "cargo test -p but-api commit_gate_apply_integrate_dryrun_targetref_pinned && ./tools/governance-checks/check_gate_before_guard.py",
      "maps_to_ac": null,
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api branch/worktree seam + real git; source grep",
        "negative_control": {
          "would_fail_if": [
            "apply_branch_integration early-returns Ok on DryRun=Yes BEFORE the gate so ro's DryRun integrate is not denied (a gate that early-returns Ok on DryRun=Yes passes the non-DryRun apply/integrate cases but FAILS this dedicated DryRun sub-case)",
            "a denied DryRun integrate persists a ref/object/oplog (state not unchanged)",
            "the gate reads working-tree gates.toml so the uncommitted edit weakens the gate and ro's apply lands",
            "the parity build-gate is a loose cross-file SUM (>=3 total) that passes with worktree_integrate ungated (e.g. branch.rs at 3, worktree.rs at 0) — the per-file assertion (branch.rs >= 2 AND worktree.rs >= 1) catches this",
            "the three entry points re-implement the decision separately (a call site is missing in branch.rs or worktree.rs — parallel gates that can drift)",
            "governance is resolved from a hardcoded list rather than the workspace-target-ref committed config",
            "config_only uses the feature branch ref so governance_present(feat)=false and the working-tree-edited apply is permitted ungated regardless of the target-ref config"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "gated_apply_repo",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=ro: invoke the PUBLIC apply_branch_integration(branch=feat, DryRun=Yes) — the DEDICATED DryRun-no-bypass sub-case; the public apply + worktree_integrate have NO DryRun param so they are not part of this sub-case (cite CAP-AUTHZ-01 DryRun-enforced)",
                "grep the three but-api entry points PER FILE to confirm one shared decision helper (branch.rs >= 2 AND worktree.rs >= 1)"
              ]
            },
            "end_state": {
              "must_observe": [
                "the DryRun=Yes apply_branch_integration is STILL denied `error.code == \"perm.denied\"` (the gate runs in the PUBLIC fn before the guard, even under DryRun) — the gate did not early-return Ok on DryRun",
                "the denied DryRun integrate persists nothing (no ref/object/oplog written for the attempt; feat HEAD sha `==` base)",
                "`grep -cE 'enforce_commit_gate_for_target' crates/but-api/src/branch.rs` `>= 2` (apply + apply_branch_integration public seams) AND `grep -cE 'enforce_commit_gate_for_target' crates/but-api/src/legacy/worktree.rs` `>= 1` (worktree_integrate)"
              ],
              "must_not_observe": [
                "the DryRun=Yes integrate permitted (exit `0` / Ok) for ro — a DryRun bypass",
                "a persisted ref/object/oplog from the denied DryRun integrate",
                "`0` matches for enforce_commit_gate_for_target in crates/but-api/src/legacy/worktree.rs (worktree_integrate ungated — a parity gap a loose cross-file sum would miss)",
                "fewer than 2 matches in crates/but-api/src/branch.rs (one of the apply/integrate public seams ungated)"
              ]
            }
          },
          {
            "start_ref": "gated_apply_repo_wt_unprotect",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=ro: with the working-tree .gitbutler/*.toml weakened (uncommitted, would grant ro contents:write), invoke the PUBLIC branch::apply(existing_branch=feat) (cite T-GATES-019 family, CAP-CONFIG-01)"
              ]
            },
            "end_state": {
              "must_observe": [
                "the apply is still denied `error.code == \"perm.denied\"` (governance read from the WORKSPACE TARGET-ref blob at main, NOT the working-tree edit); feat HEAD sha `==` base"
              ],
              "must_not_observe": [
                "exit `0` / Ok for the working-tree-weakened apply (the edit weakened the gate)",
                "ro granted contents:write from the uncommitted working-tree edit",
                "the ref advanced"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "worktree_integrate onto protected main denied branch.protected naming main; main unchanged (T-GATES-017)",
      "verify": "cargo test -p but-api commit_gate_worktree_integrate_protected_rejected",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "a feature-target worktree-integrate by a contents:write principal is permitted by the gate (no governance Denial)",
      "verify": "cargo test -p but-api commit_gate_worktree_integrate_protected_rejected",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "a contents:read-only principal denied perm.denied on the PUBLIC branch::apply AND apply_branch_integration (config read from the workspace target ref); NO branch.protected vacuously asserted (T-GATES-016, T-AUTHZ-011)",
      "verify": "cargo test -p but-api commit_gate_apply_integrate_readonly_denied",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "a feature-branch apply/integrate by a contents:write principal permitted by the gate (no governance Denial)",
      "verify": "cargo test -p but-api commit_gate_apply_integrate_readonly_denied",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "a DryRun=Yes apply_branch_integration is STILL gated (perm.denied for ro) and persists nothing — DEDICATED DryRun-no-bypass sub-case for apply_branch_integration (CAP-AUTHZ-01)",
      "verify": "cargo test -p but-api commit_gate_apply_integrate_dryrun_targetref_pinned",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "description": "a working-tree gates.toml edit cannot weaken; the workspace-target-ref blob (main) governs apply/integrate/worktree (T-GATES-019 family, CAP-CONFIG-01)",
      "verify": "cargo test -p but-api commit_gate_apply_integrate_dryrun_targetref_pinned",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-7",
      "type": "test_criterion",
      "description": "all three entry points call the SAME commit::gate::enforce_commit_gate_for_target decision (mechanism parity, PER-FILE: crates/but-api/src/branch.rs >= 2 AND crates/but-api/src/legacy/worktree.rs >= 1, not a loose cross-file sum or a parallel gate)",
      "verify": "./tools/governance-checks/check_gate_before_guard.py",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-8",
      "type": "test_criterion",
      "description": "(S9b) branch::apply AND apply_branch_integration on a repo with NO configured default target are PERMITTED — the gate is SKIPPED (target_ref_or_err() Err(DefaultTargetNotFound) is MATCHED, not ?-propagated) so a legitimate ungoverned no-target apply does NOT hard-error (parity with commit_gate_absent_config_is_ungoverned, commit_gate.rs:255-289; apply.rs:44-46; ref_metadata.rs:357-362)",
      "verify": "cargo test -p but-api commit_gate_apply_integrate_no_target_ungoverned",
      "maps_to_ac": "AC-2"
    }
  ]
}
-->
</details>
