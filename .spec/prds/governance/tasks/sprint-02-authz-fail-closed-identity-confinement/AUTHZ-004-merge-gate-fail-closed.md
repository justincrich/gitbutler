# AUTHZ-004: Merge/forge gate fail-closed — deterministic config.invalid vs perm.denied + undefined-required-group hard-deny (merge-gate layer) + DryRun-no-bypass

## What this does

Hardens the Sprint-01b merge/forge gate (`crates/but-api/src/legacy/merge_gate.rs` + the forge seam in `crates/but-api/src/legacy/forge.rs`) so that an unknown principal, a no-handle invocation, a malformed target-ref `gates.toml`, and a gate whose `require_approval_from_group` names an UNDEFINED group are each DENIED with the EXACT structured code instead of running. Makes `config.invalid` vs `perm.denied` DETERMINISTIC: a malformed target-ref config surfaces `config.invalid` (never `perm.denied`, never a skip) even for a fully-authorized principal; an unknown/no-handle principal surfaces `perm.denied`; an undefined required group is hard-denied `gate.review_required` (never vacuously satisfied). The undefined-group hard-deny lives in the but-api **merge-gate layer** (review_requirement / merge_gate wrapper), NOT in but-authz config.rs. The gate fires EVEN under DryRun.

## Why

Sprint 02 · PRD UC-AUTHZ-03, UC-AUTHZ-04 · capabilities CAP-AUTHZ-01, CAP-CONFIG-01. The fail-closed half of the merge gate: an action by an unknown principal, with no handle, against malformed config, or naming an undefined required group must be denied with the right code rather than running. This is the merge/forge-scoped subset of Sprint 02; the mechanism-agnostic commit-gate coverage + standalone target-ref proofs (T-GATES-016..019) are Sprint 04.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-api merge_gate_unknown_and_no_handle_failclosed` (integration, real forge seam + real git + real but-db). Full gate set in the spec below.

## Scope

- `crates/but-api/src/legacy/merge_gate.rs` (MODIFY) — harden the merge-gate wrapper: deterministic classification ordering (config-load -> authorize -> requirement), add the undefined-`require_approval_from_group` hard-deny path in the merge-gate layer, ensure DryRun does not bypass. OWNS the fail-closed classification on the merge/forge path
- `crates/but-api/src/legacy/forge.rs` (MODIFY — minimal) — only if the gate-call ordering on `merge_review` / `set_review_auto_merge` needs the fail-closed gate fired before the inner `.await` (do NOT re-architect the Sprint-01b wiring)
- `crates/but-api/tests/merge_gate.rs` (MODIFY) — add the Sprint-02 fail-closed integration cases (unknown principal, no handle, malformed config, undefined group, DryRun, ghost+malformed ordering) against the real forge seam + real git + real but-db

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: AUTHZ-004 - Merge/forge gate fail-closed: deterministic config.invalid vs perm.denied + undefined-required-group hard-deny (merge-gate layer) + DryRun-no-bypass
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     M  (150 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-AUTHZ-03, UC-AUTHZ-04
CAPABILITIES: CAP-AUTHZ-01, CAP-CONFIG-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-api merge_gate
  check: cargo check -p but-api --all-targets
  lint:  cargo clippy -p but-api --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Integration tests are green against the real but-api forge seam + real git + real but-db: a merge by a principal ABSENT from permissions.toml (BUT_AGENT_HANDLE=ghost) is denied perm.denied (nothing merged); a merge with BUT_AGENT_HANDLE unset is rejected perm.denied (no anonymous action); a merge against a malformed target-ref gates.toml is denied config.invalid (NEVER perm.denied, never a skip) EVEN for a merge-holding principal AND even for a ghost caller (config-load-first regardless of who calls); a merge whose target-ref gate names an UNDEFINED require_approval_from_group is hard-denied gate.review_required (never vacuously satisfied) even with a distinct approval present @head; and a DryRun merge by an unknown principal still fires perm.denied and persists nothing. The classification is DETERMINISTIC: config-load happens first (a malformed config -> config.invalid), then principal resolution + authorize (unknown/no-handle/missing-authority -> perm.denied), then the review requirement (undefined group / unmet -> gate.review_required) — the three codes are NEVER blurred.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST make the classification DETERMINISTIC and ORDERED: (1) load the target-ref governance config FIRST — any read/parse fault is config.invalid (from but_authz::ConfigError::code()); (2) THEN resolve the principal from BUT_AGENT_HANDLE + authorize(merge) — unknown handle / no handle / missing merge authority is perm.denied; (3) THEN evaluate the review requirement — an unmet/undefined-group requirement is gate.review_required. A malformed config MUST surface config.invalid even when the principal would otherwise be denied perm.denied or permitted, AND even when the caller is an unknown ghost — config-load-first does not depend on the caller (UC-AUTHZ-04 DETERMINISTIC rule). The codes are never blurred.
- [MUST] MUST add the undefined-require_approval_from_group HARD-DENY IN THE BUT-API MERGE-GATE LAYER (the review_requirement evaluator / merge_gate wrapper), NOT in but-authz config.rs. CONFIRMED GROUNDING: but-authz `GatesWire` (crates/but-authz/src/config.rs:397-407) parses ONLY `[[branch]]{name,protected}` — there is NO `[[gate]]` table and NO `require_approval_from_group`/`min_approvals` field in but-authz, and `#[serde(deny_unknown_fields)]` means adding one there would parse-fail. The undefined-group check at config.rs:304-309 rejects a PRINCIPAL's `groups=[...]` membership referencing an undefined group — a DIFFERENT field that NEVER fires for a gate's require_approval_from_group. The gate-requirement schema ([[gate]] + require_approval_from_group/min_approvals) is a Sprint-01b GATES-003 deliverable that lands in the but-api merge layer (GATES-003 writeProhibited forbids modifying but-authz/**). This task wires the undefined-group hard-deny at the merge-gate wrapper / review_requirement::evaluate call site (owned by GATES-005) — when require_approval_from_group names a group NOT defined in the target-ref permissions.toml, the requirement is UNSATISFIABLE and the merge is denied gate.review_required naming the undefined group — NEVER vacuously satisfied (no members => "all approved" trivially true is the soundness hole this closes; T-AUTHZ-030). Do NOT silently rely on a vacuous-pass that collapses to the min_approvals-only path.
- [MUST] MUST resolve the principal ONLY from BUT_AGENT_HANDLE via but_authz::resolve_principal against the committed target-ref GovConfig and fail closed (perm.denied) on unset/empty/unknown handle — never a default/anonymous principal (CAP-AUTHZ-01; AUTHZ-003 already provides resolve_principal/Denial::no_handle/Denial::unknown_principal).
- [MUST] MUST run the gate EVEN under DryRun — a DryRun merge by an unknown principal still returns perm.denied + exit 1 and persists nothing (no merge ref/object, no local_review_verdicts mutation). DryRun only suppresses persisting refs/objects/oplog; do NOT early-return on DryRun before the gate (CAP-AUTHZ-01).
- [MUST] MUST read the review requirement + group membership ONLY from the TARGET-REF gates.toml/permissions.toml blobs via the Sprint-01b loader path — never the working tree or feature head (CAP-CONFIG-01).
- [NEVER] NEVER overload GitButler's repo-access Permission/RepoExclusive lock as the authorization carrier — authorization is the orthogonal Authority axis (02-system-components.md; RULES.md lock discipline). The merge gate carries its codes as gate-owned &'static str (mirroring Denial::PERM_DENIED_CODE), NOT but-error::Code variants unless the desktop frontend consumes them.
- [NEVER] NEVER add a test asserting the forgeable direct-DB-write to local_review_verdicts is blocked, nor a forge-UI/auto-merge-on-platform/raw-push bypass — those are documented accepted-leaks (R6/R1); asserting them encodes a false guarantee.
- [STRICTLY] STRICTLY scope to the MERGE/forge gate only — the mechanism-agnostic commit-gate fail-closed coverage + the standalone target-ref-only proofs (T-GATES-016..019) are Sprint 04 (GATES-006). This task covers T-AUTHZ-027/028/029/030/031 on the merge/forge path.
- [STRICTLY] STRICTLY surface the denial as the structured contract ({error:{code,message}}, exit 1) with code ∈ {perm.denied, gate.review_required, config.invalid} at the CLI boundary, the SAME contract the commit gate uses (classify_error in crates/but-api/src/commit/gate.rs:83 returns CommitGateError{code,message} — it has NO remediation_hint field; the Denial.remediation_hint is intentionally DROPPED by the mirror, a known limitation owned by MGMT-IPC-002 in Sprint 06a). Assert {code, message} only — do NOT assert remediation_hint at the CLI boundary in this task.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: unknown principal (ghost) AND no-handle merge each denied perm.denied; nothing merged
- [ ] AC-2: malformed target-ref config -> config.invalid (NEVER perm.denied, never a skip) even for a merge-holder AND even for a ghost caller (config-load-first regardless of caller)
- [ ] AC-3: undefined require_approval_from_group hard-denied gate.review_required in the merge-gate layer, not vacuously satisfied
- [ ] AC-4: DryRun merge by an unknown principal still perm.denied + persists nothing
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: Unknown principal AND no-handle merge each denied perm.denied (fail closed) [PRIMARY]
  GIVEN: fixture `merge_gated_unknown` (valid config; maint=[merge], reviewer=[reviews:write]; open governed review on feat with a distinct approving verdict @head)
  WHEN:  a merge is attempted with BUT_AGENT_HANDLE=ghost (absent from permissions.toml), then with BUT_AGENT_HANDLE unset
  THEN:  both denied error.code=="perm.denied" (exit 1, nothing merged, trunk HEAD sha == base); ghost's message reports principal not found, unset's message reports BUT_AGENT_HANDLE required
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api forge seam + real git + real but-db
  VERIFY: cargo test -p but-api merge_gate_unknown_and_no_handle_failclosed
  SCENARIO: NEGATIVE_CONTROL would fail if the gate default-allows an unknown principal; an unset handle resolves a default/anonymous principal; resolve_principal is never called; the denial is not perm.denied (no-op stub); the merge proceeds and trunk advances. Assert {code,message} only (the CLI mirror has no remediation_hint).

AC-2: Malformed target-ref config -> config.invalid, NEVER perm.denied (deterministic, caller-independent)
  GIVEN: fixture `merge_gated_malformed_targetref` (maint HOLDS merge, but the target-ref gates.toml is unparseable TOML)
  WHEN:  maint attempts the merge, then a ghost (BUT_AGENT_HANDLE=ghost, absent from permissions.toml) attempts the same merge against the same malformed config
  THEN:  BOTH denied error.code=="config.invalid" (NEVER perm.denied), exit 1, nothing merged — a malformed config is classified config.invalid even for a fully-authorized principal AND even for an unknown ghost caller, never blurred and never skipped (config-load happens before principal resolution regardless of caller)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api forge seam + real git + real but-db
  VERIFY: cargo test -p but-api merge_gate_malformed_config_is_config_invalid
  SCENARIO: NEGATIVE_CONTROL would fail if the malformed config is skipped (fail-open); misclassified as perm.denied for the ghost (proving authorize ran before config-load — the ordering bug this sub-case closes); the loader reads the working-tree blob; the gate panics instead of the structured config.invalid contract.

AC-3: Undefined require_approval_from_group hard-denied in the merge-gate layer (not vacuously satisfied)
  GIVEN: fixture `merge_gated_undefined_group` (a target-ref gate sets require_approval_from_group=["ghost-reviewers"] naming an UNDEFINED group; a distinct approval present @head from a DEFINED reviewer)
  WHEN:  maint attempts the merge
  THEN:  denied error.code=="gate.review_required" naming the undefined group "ghost-reviewers" in unmet[], exit 1, nothing merged — the undefined group can never be approved, so the requirement is unsatisfiable; the hard-deny is enforced in the but-api merge-gate layer (review_requirement/merge_gate), since but-authz config.rs has no gate-requirement schema
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api forge seam + real git + real but-db
  VERIFY: cargo test -p but-api merge_gate_undefined_required_group_denied
  SCENARIO: NEGATIVE_CONTROL would fail if the undefined group is vacuously satisfied; require_approval_from_group naming an undefined group is silently ignored so the requirement collapses to the min_approvals-only path; the gate is a no-op; the present distinct approval counts as satisfying the undefined-group requirement.

AC-4: DryRun merge by an unknown principal still perm.denied + persists nothing
  GIVEN: fixture `merge_gated_unknown`, DryRun
  WHEN:  a DryRun merge is attempted with BUT_AGENT_HANDLE=ghost
  THEN:  still denied perm.denied (exit 1) and persists nothing (no merge ref/object, no verdict mutation, trunk HEAD sha == base)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api forge seam + real git + real but-db
  VERIFY: cargo test -p but-api merge_gate_dryrun_unknown_failclosed_persists_nothing
  SCENARIO: NEGATIVE_CONTROL would fail if DryRun early-returns before the gate; a denied DryRun persists a ref/object/verdict; the DryRun path resolves a default principal; the gate runs only on the non-DryRun path.

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, error): unknown principal (ghost) merge denied perm.denied; no-handle merge denied perm.denied; nothing merged (T-AUTHZ-027, T-AUTHZ-028)
    VERIFY: cargo test -p but-api merge_gate_unknown_and_no_handle_failclosed
- TC-2 (-> AC-2, error): malformed target-ref gates.toml -> config.invalid for BOTH a merge-holder AND a ghost caller, NEVER perm.denied, never skipped (T-AUTHZ-029)
    VERIFY: cargo test -p but-api merge_gate_malformed_config_is_config_invalid
- TC-3 (-> AC-3, edge): require_approval_from_group naming an undefined group hard-denied in the merge-gate layer, not vacuously satisfied (T-AUTHZ-030)
    VERIFY: cargo test -p but-api merge_gate_undefined_required_group_denied
- TC-4 (-> AC-4, error): DryRun merge by an unknown principal still perm.denied + persists nothing (T-AUTHZ-031, CAP-AUTHZ-01)
    VERIFY: cargo test -p but-api merge_gate_dryrun_unknown_failclosed_persists_nothing
- TC-5 (-> AC-2, edge): deterministic ordering — config-load-first so malformed -> config.invalid regardless of caller (merge-holder or ghost), then authorize -> perm.denied, then requirement -> gate.review_required, never blurred
    VERIFY: cargo test -p but-api merge_gate_malformed_config_is_config_invalid

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01, CAP-CONFIG-01
provides: the fail-closed hardening on the merge/forge gate: deterministic config.invalid vs perm.denied vs gate.review_required ordering (caller-independent config-load-first); unknown-principal + no-handle fail-closed on merge; undefined-require_approval_from_group hard-deny in the merge-gate layer; DryRun-no-bypass on the merge path
consumes: but_authz::resolve_principal + authorize + Authority::Merge + Denial (AUTHZ-003); but_authz::load_governance_config + GovConfig + ConfigError::code()=="config.invalid" (AUTHZ-002); the Sprint-01b merge_gate wrapper + review_requirement::evaluate + the gate-requirement schema GATES-003 lands in the but-api merge layer (GATES-003/005); the commit gate's classify_error pattern returning CommitGateError{code,message} (crates/but-api/src/commit/gate.rs:83)
boundary_contracts:
  - CAP-AUTHZ-01: the merge action resolves BUT_AGENT_HANDLE->Principal and authorizes(merge) at the forge seam EVEN under DryRun; unknown/no-handle/missing-authority -> perm.denied.
  - CAP-CONFIG-01: a malformed target-ref config -> config.invalid (deterministically, never perm.denied, regardless of caller); an undefined required group is hard-denied in the merge-gate layer; the requirement is read only from the target-ref blob.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-api/src/legacy/merge_gate.rs (MODIFY) — fail-closed classification ordering + undefined-group hard-deny (merge-gate layer) + DryRun-no-bypass on the merge path
  - crates/but-api/src/legacy/forge.rs (MODIFY — minimal) — only the gate-call ordering on merge_review/set_review_auto_merge if the fail-closed gate must fire before the inner .await; do NOT re-architect Sprint-01b wiring
  - crates/but-api/tests/merge_gate.rs (MODIFY) — the Sprint-02 fail-closed integration cases
writeProhibited:
  - crates/but-authz/** — CONSUME resolve_principal/authorize/load_governance_config/ConfigError/Denial; do NOT modify the primitive. The undefined-group-for-a-GATE check CANNOT live here: GatesWire has no gate-requirement schema and GATES-003 writeProhibits but-authz/** (ADVISORY: GATES-003's writeProhibited on but-authz/** while it claims to read require_approval_from_group is reconciled by the schema living in the but-api merge layer, not but-authz config.rs)
  - crates/but-api/src/legacy/review_requirement.rs — OWNED by GATES-005; if the undefined-group hard-deny belongs in the evaluator, COORDINATE: this task adds the hard-deny at the merge-gate wrapper call site OR via GATES-005's evaluator unmet[] payload — DOCUMENT the decision, do not silently re-implement the evaluator
  - crates/but-db/src/table/local_review_verdicts.rs — OWNED by GATES-002; consume the query, do not redefine
  - crates/but-error/src/lib.rs — carry the gate codes as gate-owned &'static str, no Code variants
  - crates/but-api/src/commit/** — the commit gate is GATES-001 / Sprint 04 mechanism-agnostic coverage
  - any gitbutler-* crate beyond what the action boundary strictly requires (crates/AGENTS.md)
  - Any file not listed in write_allowed

--------------------------------------------------------------------------------
OUT OF SCOPE
--------------------------------------------------------------------------------
  - The Sprint-01b gate-requirement schema ([[gate]] + require_approval_from_group/min_approvals) is GATES-003's deliverable in the but-api merge layer. BLOCKED-UNTIL: AC-3 is contingent on that schema landing in a writable but-api location; when it lands, AUTHZ-004 wires the undefined-group hard-deny at the merge-gate call site. ADVISORY: GATES-003's writeProhibited "do not modify but-authz/**" contradicts any claim that require_approval_from_group is read via the but-authz loader — it is read in the but-api merge layer, which is where this hard-deny lives. Do NOT rely on a vacuous pass if the schema is not yet present; escalate the ordering instead.
  - Sprint-04 GATES-008 OWNERSHIP: AUTHZ-004 OWNS the merge-path undefined-group hard-deny now; GATES-008 is the deepening / standalone-proof (commit-gate + dedicated target-ref-only proofs), NOT a competing owner of the merge-path check. No race: the merge-path lands here.
  - T-GATES-016..019 (mechanism-agnostic commit-gate fail-closed coverage + dedicated standalone target-ref-only proofs) DEFERRED to Sprint 04 (GATES-006). This task hardens the MERGE/forge path only.
  - The forge-network merge COMPLETION (the change landing on the remote trunk) is NOT asserted locally — merge_review/set_review_auto_merge are forge-bound (derive_forge_repo_info errors on a bare repo). DENIAL paths are fully locally provable (the gate fires before the forge .await); the POSITIVE-path completion is structural/out-of-local-scope (Sprint-01b GATES-003 re-scope note).
  - remediation_hint surfacing at the CLI boundary is MGMT-IPC-002 (Sprint 06a): CommitGateError{code,message} drops Denial.remediation_hint. This task asserts {code,message} only.
  - The forgeable direct-DB-write / forge-UI / raw-push bypass are accepted-leaks (R6/R1), NOT tested.

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-api/src/commit/gate.rs (43-96)
   Focus: THE CLASSIFICATION PATTERN TO MIRROR — enforce_commit_gate loads governance config, resolves the principal from BUT_AGENT_HANDLE (resolve_principal_from_env), authorizes, and classify_error (83) downcasts Denial -> CommitGateError{code,message} (NO remediation_hint — it is dropped) and ConfigError -> config.invalid. The deterministic ordering (load config -> resolve -> authorize) and the structured {code,message} contract are already proven here; the merge gate mirrors it. WARNING (L2): governance_path() at gate.rs:185-189 builds [".git","butler"].concat() = `.git/butler/` — that is the gitdir MARKER-presence check, DISTINCT from the `.gitbutler/` TREE path the loader reads (but-authz config.rs:8-9 PERMISSIONS_PATH=".gitbutler/permissions.toml", GATES_PATH=".gitbutler/gates.toml"). When mirroring the commit gate, your fixtures write the `.gitbutler/` TREE blobs (what load_governance_config reads), NOT `.git/butler/`. Do not confuse the two.
2. .spec/prds/governance/tasks/sprint-01b-governed-loop-reference-flow/GATES-003-merge-gate.md (full)
   Focus: THE WRAPPER THIS TASK HARDENS — enforce_merge_gate (merge_gate.rs), the two forge entry points, the authorize(merge) pre-call guard, the target-ref pin, the DryRun-enforced property, AND the gate-requirement schema ([[gate]] + require_approval_from_group) that lands in the but-api merge layer. AUTHZ-004 adds the fail-closed classification + the undefined-group hard-deny on top, in this layer.
3. .spec/prds/governance/tasks/sprint-01b-governed-loop-reference-flow/GATES-005-stale-self-approval.md (full)
   Focus: review_requirement::evaluate (the evaluator the gate delegates to) + the unmet[] discriminator shape. The undefined-group hard-deny is added either at the wrapper call site or via the evaluator's unmet[] payload — DOCUMENT which.
4. crates/but-authz/src/config.rs (8-9, 24-29, 269-335, 364-407)
   Focus: CONFIRM — load_governance_config -> ConfigError::code()=="config.invalid"; PERMISSIONS_PATH/GATES_PATH = `.gitbutler/*` TREE paths (8-9). GatesWire (395-407) parses ONLY [[branch]]{name,protected} — NO [[gate]]/require_approval_from_group/min_approvals, and #[serde(deny_unknown_fields)] would REJECT a [[gate]] table as config.invalid. normalize_permissions (302-309) rejects a PRINCIPAL's undefined-group MEMBERSHIP — a DIFFERENT field that never fires for a gate's require_approval_from_group. THIS IS WHY the gate's undefined-group hard-deny lives in the but-api merge layer, not here.
5. crates/but-authz/src/authorize.rs (24-93, 142-175)
   Focus: CONSUME-ONLY — authorize, resolve_principal, Denial::no_handle / Denial::unknown_principal (the fail-closed constructors AC-1 asserts), Denial::PERM_DENIED_CODE.
6. crates/but-api/tests/commit_gate.rs (1-130)
   Focus: THE TEST HARNESS TO MIRROR — temp_env::with_var("BUT_AGENT_HANDLE", Some("ghost")/None, ...), but_ctx::Context::from_repo, governed_repo() fixture via but_testsupport::writable_scenario + invoke_bash, err.downcast_ref::<but_authz::Denial>(), assert denial.code, assert ref unchanged (ref_id == base). Add the malformed-config (ConfigError), the ghost+malformed ordering sub-case, and undefined-group cases in the same shape.
7. crates/but-testsupport/src/lib.rs (71-97, 432-441)
   Focus: writable_scenario("governance-base") + invoke_bash to seed `.gitbutler/` config blobs at refs/heads/main, branch to feat, advance the feature head; NEVER std::env::temp_dir().join(...).

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- Fail-closed integration tests pass: `cargo test -p but-api merge_gate`  -> Exit 0; AC-1..4 green
- Crate compiles incl. tests: `cargo check -p but-api --all-targets`  -> Exit 0
- Deterministic classification — config.invalid is never blurred into perm.denied: the malformed-config test asserts the EXACT code "config.invalid" for BOTH a merge-holder AND a ghost caller; the unknown/no-handle tests assert "perm.denied"; the undefined-group test asserts "gate.review_required"
- No Permission-lock overload: `! grep -rEn 'write_permission\(|RepoExclusive|exclusive_worktree_access' crates/but-api/src/legacy/merge_gate.rs`  -> No matches
- No role-name in the merge gate: `! grep -rEni 'implementer|reviewer|maintainer' crates/but-api/src/legacy/merge_gate.rs`  -> No matches (T-LOOP-005 family)
- Merge gate keys off the Authority axis (so AUTHZ-008 AUTHORITY_POSITIVE bites): `grep -rEn 'but_authz::authorize|Authority::Merge' crates/but-api/src/legacy/merge_gate.rs`  -> 1+ match; use the FULLY-QUALIFIED `but_authz::authorize(...)` or `Authority::Merge` form (the AUTHORITY_POSITIVE_PATTERN has no bare `Authority::` branch)
- Clippy clean: `cargo clippy -p but-api --all-targets`  -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: Deterministic-ordered fail-closed classification at the merge/forge gate — load target-ref config FIRST (ConfigError -> config.invalid, regardless of caller), THEN resolve_principal + authorize(merge) (no-handle/unknown/missing -> perm.denied), THEN evaluate the review requirement in the merge-gate layer (undefined-group / unmet -> gate.review_required). Each fault maps to its OWN stable &'static str code (never blurred). The gate fires even under DryRun; the Denial/ConfigError is propagated as an anyhow error and classified via the commit-gate's classify_error pattern (downcast Denial -> CommitGateError{code,message}, ConfigError -> config.invalid) into the exit-1 structured JSON. The undefined-group hard-deny is wired in the but-api merge layer because but-authz has no gate-requirement schema.
pattern_source: crates/but-api/src/commit/gate.rs:50 (enforce_commit_gate ordering) + :83 (classify_error returning {code,message}) — the proven load->resolve->authorize ordering and Denial/ConfigError classification
anti_pattern: Blurring a malformed config into perm.denied (or skipping it fail-open); classifying perm.denied before config-load for a ghost caller (ordering bug); attempting to add the gate-requirement schema to but-authz config.rs (GatesWire deny_unknown_fields rejects it; GATES-003 writeProhibits it); vacuously satisfying an undefined require_approval_from_group (no members => trivially approved); early-returning on DryRun before the gate; reading the requirement/group membership from the working tree; overloading the repo Permission lock as the authz carrier; asserting remediation_hint at the CLI boundary (the mirror drops it — MGMT-IPC-002); or asserting the forgeable direct-DB-write / forge-UI / raw-push bypass is blocked (false-guarantee, R6/R1).

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — Hardens GitButler's REAL merge/forge gate into a deterministic fail-closed classifier: orders config-load before authorize before requirement so config.invalid / perm.denied / gate.review_required are never blurred (even for a ghost caller against malformed config), adds the undefined-required-group hard-deny in the but-api merge layer, and proves DryRun-no-bypass against real but-api + real git + real but-db. Owns the classification ordering, the undefined-group soundness fix, and integration TDD mirroring the commit-gate harness.
reviewer: rust-reviewer
coding_standards: crates/AGENTS.md, crates/WORKSPACE_MODEL.md, crates/but/AGENTS.md, crates/but-api/src/commit/gate.rs

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: GATES-003, GATES-005, AUTHZ-002, AUTHZ-003   (Sprint-01b merge gate + gate-requirement schema + the but-authz primitive)
Blocks:     AUTHZ-005; AUTHZ-008; Sprint 05, Sprint 06a
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "AUTHZ-004",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "notes": [
    "C1/C4 grounding: but-authz GatesWire (config.rs:397-407) parses ONLY [[branch]]{name,protected}; no [[gate]]/require_approval_from_group/min_approvals and #[serde(deny_unknown_fields)] would reject one. The undefined-group hard-deny for a GATE lives in the but-api merge-gate layer, NOT but-authz config.rs. config.rs:304-309 rejects a PRINCIPAL's groups=[] membership (a different field).",
    "M2: the CLI mirror CommitGateError (gate.rs:10-16) carries only {code,message}; Denial.remediation_hint is dropped (known bug owned by MGMT-IPC-002, Sprint 06a). Assertions scope to {code,message}.",
    "L2: governance_path() in commit/gate.rs builds .git/butler/ (gitdir marker), DISTINCT from the .gitbutler/ tree path load_governance_config reads; fixtures write .gitbutler/."
  ],
  "fixtures": {
    "merge_gated_unknown": {
      "description": "A real git repo (but-testsupport writable_scenario) whose target ref main has committed .gitbutler/permissions.toml (maint=[merge]; reviewer=[reviews:write]) and a valid .gitbutler/gates.toml ([[branch]] main protected=true plus the Sprint-01b gate-requirement entry type=review min_approvals=1 require_distinct_from_author=true), with an open governed review on feat. Used to drive a merge as a principal absent from permissions.toml (BUT_AGENT_HANDLE=ghost) and with BUT_AGENT_HANDLE unset. DEPENDENCY GATE (L3): the distinct approving verdict is seeded via the governed `but review approve` verb (GATES-004, Sprint-01b); if that verb is not yet available the fixture MUST fail with a clear dependency error, never a silent direct-DB insert.",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"governance-base\");",
        "invoke_bash: write .gitbutler/permissions.toml ([[principal]] id=\"maint\" permissions=[\"merge\"]; [[principal]] id=\"reviewer\" permissions=[\"reviews:write\"])",
        "invoke_bash: write a VALID .gitbutler/gates.toml ([[branch]] name=\"main\" protected=true) plus the Sprint-01b gate-requirement entry (type=\"review\" min_approvals=1 require_distinct_from_author=true) in the but-api merge-layer schema",
        "invoke_bash: stage and commit both blobs at refs/heads/main; create branch feat; commit a change so the feature head differs from main",
        "open a governed review on feat via the but-api forge path so the merge entry point has a review target; record reviewer's distinct approving verdict via the governed `but review approve` action at the feat head (NOT a direct DB insert; fail-closed with a clear dependency error if the verb is unavailable)"
      ]
    },
    "merge_gated_malformed_targetref": {
      "description": "Same shape as merge_gated_unknown but the target-ref .gitbutler/gates.toml blob is committed with INVALID TOML, so the loader fails closed config.invalid. maint=[merge] holds the merge authority; a ghost caller (absent from permissions.toml) is also exercised. Proves the malformed config surfaces config.invalid and NEVER perm.denied for BOTH a fully-authorized principal AND an unknown ghost (config-load happens before principal resolution regardless of caller).",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"governance-base\");",
        "invoke_bash: write a VALID permissions.toml (maint=[merge], reviewer=[reviews:write]) and a BROKEN gates.toml `[[branch] name = \"main\" protected = nope` (unparseable TOML); stage and commit at refs/heads/main; create branch feat; commit a change; open a governed review on feat"
      ]
    },
    "merge_gated_undefined_group": {
      "description": "Same shape as merge_gated_unknown but the target-ref gate (in the but-api merge-layer gate-requirement schema) sets require_approval_from_group=[\"ghost-reviewers\"] where ghost-reviewers is NOT defined in permissions.toml. Proves the undefined-required-group hard-deny enforced in the merge-gate layer (the requirement is NOT vacuously satisfied; an undefined group can never be approved). maint=[merge] holds merge; reviewer=[reviews:write] exists; ghost-reviewers is undefined.",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"governance-base\");",
        "invoke_bash: write permissions.toml (maint=[merge]; reviewer=[reviews:write]) with NO group named ghost-reviewers; write the gate-requirement (type=\"review\" min_approvals=1 require_approval_from_group=[\"ghost-reviewers\"]) in the but-api merge-layer schema; stage and commit at refs/heads/main; create branch feat; commit a change; open a governed review on feat; record a distinct approving verdict from reviewer @head via governed `but review approve` (so the ONLY reason to block is the undefined group, not a missing approval)"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN merge_gated_unknown WHEN a merge is attempted by a principal absent from permissions.toml (BUT_AGENT_HANDLE=ghost), then with BUT_AGENT_HANDLE unset THEN both are denied error.code==\"perm.denied\" (exit 1, nothing merged, trunk HEAD sha == base) — an unknown principal and a no-handle invocation are fail-closed, never default-allow. CLI contract asserts {code,message} only (the mirror has no remediation_hint; MGMT-IPC-002)",
      "verify": "cargo test -p but-api merge_gate_unknown_and_no_handle_failclosed",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api forge seam + real git + real but-db",
        "negative_control": {
          "would_fail_if": [
            "the gate default-allows an unknown principal (ghost) so the merge proceeds (fail-open on unknown)",
            "an unset BUT_AGENT_HANDLE resolves a default/anonymous principal that is allowed to merge",
            "resolve_principal is never called so the merge runs without a bound principal",
            "the denial is not perm.denied because the gate is a no-op stub that returns Ok",
            "the merge proceeds and the trunk HEAD sha advances despite the denial"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "merge_gated_unknown",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=ghost: invoke the governed merge action on the open review (ghost is NOT a principal in permissions.toml) (cite T-AUTHZ-027)"
              ]
            },
            "end_state": {
              "must_observe": [
                "`error.code == \"perm.denied\"`",
                "the `message` reports the principal `\"ghost\"` was not found in committed governance config",
                "the structured CLI error carries `{code, message}` only (no remediation_hint field — the mirror drops it, MGMT-IPC-002)",
                "process exits `1`",
                "the review is NOT merged (trunk/main HEAD sha `==` the seeded base sha)"
              ],
              "must_not_observe": [
                "merge proceeded",
                "exit `0`",
                "an unknown principal default-allowed",
                "trunk/main HEAD sha advanced"
              ]
            }
          },
          {
            "start_ref": "merge_gated_unknown",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE unset: invoke the governed merge action on the open review (no handle to resolve a principal) (cite T-AUTHZ-028)"
              ]
            },
            "end_state": {
              "must_observe": [
                "`error.code == \"perm.denied\"`",
                "the `message` reports `BUT_AGENT_HANDLE` is required to resolve a governed principal",
                "process exits `1`",
                "the review is NOT merged (trunk/main HEAD sha `==` base)"
              ],
              "must_not_observe": [
                "merge proceeded",
                "exit `0`",
                "a default/anonymous principal bound",
                "trunk/main HEAD sha advanced"
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
      "description": "GIVEN merge_gated_malformed_targetref WHEN maint (HOLDS merge) attempts the merge, then a ghost (absent from permissions.toml) attempts the same merge against the same malformed config THEN BOTH are denied error.code==\"config.invalid\" (NEVER perm.denied), exit 1, nothing merged — config-load happens first regardless of caller, so a malformed config surfaces config.invalid even for a fully-authorized principal AND even for an unknown ghost, never blurred into perm.denied and never skipped",
      "verify": "cargo test -p but-api merge_gate_malformed_config_is_config_invalid",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api forge seam + real git + real but-db",
        "negative_control": {
          "would_fail_if": [
            "the malformed target-ref config is skipped (treated as no requirement) and the merge proceeds (fail-open, a no-op gate)",
            "the malformed config is misclassified as perm.denied instead of config.invalid for EITHER caller (the two codes are blurred)",
            "the ghost caller gets perm.denied instead of config.invalid — proving authorize ran before config-load (the ordering bug this ghost sub-case closes)",
            "the loader reads the working-tree gates.toml instead of the target-ref blob, so the malformed committed blob is bypassed",
            "the gate panics on the malformed config instead of returning the structured config.invalid contract"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "merge_gated_malformed_targetref",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=maint: invoke the governed merge against the malformed target-ref gates.toml (maint holds merge, so the ONLY reason to fail is the malformed config) (cite T-AUTHZ-029, UC-AUTHZ-04)"
              ]
            },
            "end_state": {
              "must_observe": [
                "`error.code == \"config.invalid\"`",
                "process exits `1`",
                "the review is NOT merged (trunk/main HEAD sha `==` base)"
              ],
              "must_not_observe": [
                "`error.code == \"perm.denied\"` because a malformed config must NOT be misclassified as a permission denial (deterministic ordering)",
                "merge proceeded",
                "exit `0`",
                "the malformed config silently skipped (treated as satisfied)"
              ]
            }
          },
          {
            "start_ref": "merge_gated_malformed_targetref",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=ghost: invoke the governed merge against the SAME malformed target-ref gates.toml (ghost is absent from permissions.toml) — proves config-load-first does not depend on the caller (cite M1, T-AUTHZ-029)"
              ]
            },
            "end_state": {
              "must_observe": [
                "`error.code == \"config.invalid\"` (NOT perm.denied — config-load precedes principal resolution even for an unknown ghost)",
                "process exits `1`",
                "the review is NOT merged (trunk/main HEAD sha `==` base)"
              ],
              "must_not_observe": [
                "`error.code == \"perm.denied\"` (would prove authorize ran before config-load — the ordering bug)",
                "merge proceeded",
                "exit `0`",
                "the malformed config skipped because the caller was unknown"
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
      "description": "GIVEN merge_gated_undefined_group (require_approval_from_group=[\"ghost-reviewers\"] naming a group UNDEFINED in target-ref permissions.toml, with a distinct approval present @head from a defined reviewer) WHEN maint attempts the merge THEN the merge is denied gate.review_required naming the undefined required group in unmet[], exit 1, nothing merged — an undefined required group is NEVER vacuously satisfied; the hard-deny is enforced in the but-api merge-gate layer (but-authz has no gate-requirement schema)",
      "verify": "cargo test -p but-api merge_gate_undefined_required_group_denied",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api forge seam + real git + real but-db",
        "negative_control": {
          "would_fail_if": [
            "an undefined required group is treated as vacuously satisfied (no member exists, so all-members-approved is trivially true) and the merge proceeds — the soundness hole this AC closes",
            "require_approval_from_group naming an undefined group is silently ignored so the requirement collapses to the min_approvals-only path and the present distinct approval satisfies it",
            "the gate is a no-op stub so the merge proceeds regardless of the undefined group",
            "the undefined-group denial is misclassified config.invalid only when there is ALSO a TOML parse error (this case must deny even though the TOML parses cleanly)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "merge_gated_undefined_group",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=maint: invoke the governed merge where require_approval_from_group=[\"ghost-reviewers\"] names an undefined group, even though a distinct approval from a DEFINED reviewer exists @head (cite T-AUTHZ-030, UC-AUTHZ-04)"
              ]
            },
            "end_state": {
              "must_observe": [
                "`error.code == \"gate.review_required\"`",
                "the `unmet[]` payload names the undefined required group `\"ghost-reviewers\"` (the requirement can never be satisfied)",
                "process exits `1`",
                "the review is NOT merged (trunk/main HEAD sha `==` base)"
              ],
              "must_not_observe": [
                "merge proceeded",
                "exit `0`",
                "the undefined group treated as vacuously satisfied (empty unmet)",
                "the present distinct approval counted as satisfying the undefined-group requirement"
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
      "description": "GIVEN merge_gated_unknown WHEN a DryRun merge is attempted by an unknown principal (BUT_AGENT_HANDLE=ghost) THEN the gate still fires perm.denied (exit 1) and persists nothing (no merge ref/object, no verdict mutation, trunk HEAD sha == base) — DryRun does NOT bypass the fail-closed gate",
      "verify": "cargo test -p but-api merge_gate_dryrun_unknown_failclosed_persists_nothing",
      "maps_to_ac": null,
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api forge seam + real git + real but-db",
        "negative_control": {
          "would_fail_if": [
            "DryRun early-returns before the gate so the unknown principal is never denied (DryRun bypass)",
            "a denied DryRun persists a merge ref/object or mutates local_review_verdicts (state not unchanged)",
            "the DryRun path resolves a default principal and proceeds",
            "the gate runs only on the non-DryRun path and is omitted under DryRun"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "merge_gated_unknown",
            "action": {
              "actor": "cli_user",
              "steps": [
                "BUT_AGENT_HANDLE=ghost: invoke the governed merge under DryRun (ghost is unknown) (cite CAP-AUTHZ-01 DryRun-enforced, T-AUTHZ-031)"
              ]
            },
            "end_state": {
              "must_observe": [
                "`error.code == \"perm.denied\"`",
                "process exits `1`",
                "no merge commit/ref persisted AND no local_review_verdicts mutation from this attempt (trunk/main HEAD sha `==` base, unchanged)"
              ],
              "must_not_observe": [
                "exit `0`",
                "trunk/main sha advanced",
                "a persisted merge ref/object from the denied dry run",
                "DryRun skipped the fail-closed gate"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "unknown principal (ghost) merge denied perm.denied; no-handle merge denied perm.denied; nothing merged; CLI error is {code,message} (T-AUTHZ-027, T-AUTHZ-028)",
      "verify": "cargo test -p but-api merge_gate_unknown_and_no_handle_failclosed",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "malformed target-ref gates.toml -> config.invalid for BOTH a merge-holder AND a ghost caller, NEVER perm.denied, never skipped (config-load-first, caller-independent) (T-AUTHZ-029, M1)",
      "verify": "cargo test -p but-api merge_gate_malformed_config_is_config_invalid",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "require_approval_from_group naming an undefined group hard-denied in the merge-gate layer, not vacuously satisfied (T-AUTHZ-030)",
      "verify": "cargo test -p but-api merge_gate_undefined_required_group_denied",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "DryRun merge by an unknown principal still perm.denied + persists nothing (T-AUTHZ-031, CAP-AUTHZ-01)",
      "verify": "cargo test -p but-api merge_gate_dryrun_unknown_failclosed_persists_nothing",
      "maps_to_ac": "AC-4"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "deterministic classification ordering: config-load-first so malformed config -> config.invalid regardless of caller, then authorize -> perm.denied, then requirement -> gate.review_required, never blurred (T-AUTHZ-029/030/031, M1)",
      "verify": "cargo test -p but-api merge_gate_malformed_config_is_config_invalid",
      "maps_to_ac": "AC-2"
    }
  ]
}
-->
</details>
