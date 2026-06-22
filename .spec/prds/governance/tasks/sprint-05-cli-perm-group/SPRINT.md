---
sprint: 05
sequence: 6
timeline: Phase 3 — CLI governance management
status: In Progress
proposed_by: rust-planner
milestone: sprint-05
prd: ../../README.md
roadmap: ../../ROADMAP.md
generated_by: kb-sprint-tasks-plan
---

# Sprint 05: CLI `but perm` / `but group`

**Sequence:** 6
**Timeline:** Phase 3 — CLI governance management
**Status:** In Progress
**Proposed by:** rust-planner
**Milestone:** — (`sprint-05`)

## Overview

The first **governance-management** sprint. Sprints 01a–04 built the enforcement core — the `but-authz`
primitive, the commit + merge gates, fail-closed identity confinement, and ref-pinned grouping — and proved
every property against the **read** side of governed config (`.gitbutler/permissions.toml` /
`.gitbutler/gates.toml` loaded at the target ref). Sprint 05 builds the **write** side an admin uses to
_manage_ that config: the `but perm` and `but group` CLI nouns. This is the first sprint that **persists**
governed config — Sprints 02 and 03 deliberately deferred the persisted `but perm`/`but group` write path
here (the admin-write guard and the group definition/membership reads were proven at the authz layer with
`BLOCKED-UNTIL` notes naming CLI-001/CLI-002 as the consumer), and the `perm list` reconnaissance-scoping
criterion (T-AUTHZ-025) was relocated here from Sprint 02 to its natural CLI home.

Two nouns, one per task:

- **`but perm {list, grant, revoke}` (CLI-001):** the per-principal governance verbs.
  `grant`/`revoke` write functional permission(s) into a principal's `.gitbutler/permissions.toml` entry
  (registering the entry on first grant), each authorizing `administration:write` at the **target ref**
  before the write and printing the ref-pin caveat — _"takes effect once committed to the target branch."_
  `list` shows a principal's committed effective set **plus** any working-tree (uncommitted) grant marked
  **PENDING**, and is itself reconnaissance-scoped: `--principal <other>` is denied `perm.denied` unless the
  caller is that principal or holds `administration:read` (T-AUTHZ-025). The write is **inert until
  committed** — it edits the working tree only; the next target-ref read is what makes it effective, exactly
  as every Sprint 01a–04 read proved.
- **`but group {create, grant, add-member, remove-member, list}` (CLI-002):** the grouping verbs.
  `create`/`grant`/`add-member`/`remove-member` mutate the `[[group]]` definitions, grants, and membership
  in `.gitbutler/permissions.toml`, each gated by `administration:write` at the target ref and printing the
  same ref-pin caveat; `list` shows groups, grants, and membership under `administration:read`. These are
  the persisted-write consumers Sprint 03's GRPS tasks named — the inert-until-committed _behavior_ they
  proved at the read layer is now produced by a real verb that warns the operator.

Both nouns route their `but-api` functions through the **same** `administration:write` guard the enforcement
core uses (AUTHZ-006), and the `but-api` perm/group functions authored here are the exact functions Sprint
06a's MGMT renderer re-invokes as Tauri commands — so the CLI write path and the future UI write path share
one server-side authorization seam (the "UI is never a bypass" invariant). This sprint is **headless/CLI** —
every property is verified by running a `but` command and observing the structured denial
`{code, message, remediation_hint}` + exit code (or, for the positive cases, the write landing in the
working-tree config with the PENDING/ref-pin caveat printed). Every gate proof draws from
[`11-e2e-testing-criteria.md`](../../11-e2e-testing-criteria.md).

> **Net-new write path (this sprint OWNS the governed-config writer).** `but-authz` `config.rs` is
> **loader-only** today (`load_governance_config` reads + normalizes a target-ref blob; there is no
> serializer, no `grant`/`revoke`, no `.gitbutler/permissions.toml` writer), no `Perm`/`Group` verb exists in
> `crates/but/src/args/`, and no `but-api` perm/group function exists. CLI-001/CLI-002 author all three: the
> TOML read-modify-write helper (preserving the existing file's other entries), the CLI verb surface in
> `crates/but/src/args/`, and the `but-api` `_with_authz`-style functions. The writer edits the **working
> tree** only — it never commits — so the inert-until-committed contract is structural, not a runtime check.

> **Admin-write gating composes AUTHZ-006, never re-implements it.** Every mutating verb authorizes
> `administration:write` via the Sprint-02 AUTHZ-006 guard read at the target ref **before** the write. A
> self-grant of `administration:write` on a feature head is therefore inert for the same change (the caller
> must already hold it at the target ref) — the CAP-CONFIG-01 self-escalation contract Sprint 03 proved,
> now exercised through the real write verb. CLI tests are expensive (`crates/but/AGENTS.md`): prove the
> persisted-write + denial behavior primarily at the `but-api` function layer against real `but-authz` +
> real git, and use the `crates/but/tests/` snapbox CLI harness for the happy-path verb surface + the
> ref-pin-caveat / PENDING stdout contract.

## Human Testing Gate

**Gate:** An admin runs `but perm grant` and `but group add-member`, sees the takes-effect-once-committed
caveat, `but perm list` shows the committed effective set plus the new grant as pending, and a non-admin's
grant or cross-principal list is denied `perm.denied`.

### Test Steps

1. Run `but perm grant --principal rust-implementer reviews:write` as an admin → exit 0, prints the ref-pin caveat.
2. Run `but perm list --principal rust-implementer` → shows the committed effective set (unchanged) and the new grant as PENDING (not yet in effect).
3. Run `but group create code-reviewers --permissions reviews:write` → exit 0, `[[group]]` written.
4. Run `but group add-member code-reviewers --principal rust-reviewer` → exit 0, ref-pin caveat printed.
5. Run `but perm grant ...` as a non-admin principal → denied, exit 1, `perm.denied`.
6. Run `but perm list --principal <other>` as a non-admin (not self, no admin:read) → denied, `perm.denied`, no recon.
7. Run `but perm revoke --principal rust-implementer reviews:write` as admin → exit 0.

## Tasks

| ID          | Title                                                                                                      | Agent            | Estimate |
| ----------- | ---------------------------------------------------------------------------------------------------------- | ---------------- | -------- |
| CLI-001     | `but perm {list,grant,revoke}` + admin-write gating + ref-pin caveat + perm-list scoping                   | rust-implementer | 240 min  |
| CLI-002     | `but group {create,grant,add-member,remove-member,list}` + admin-write gating                              | rust-implementer | 210 min  |
| CLI-REM-001 | Close group verb contract gaps: denial matrix, remove-member, delegated admin, duplicate create, no delete | rust-implementer | 120 min  |
| CLI-REM-002 | Close perm fail-closed variants and structured denial remediation hints                                    | rust-implementer | 90 min   |
| CLI-REM-003 | Prove CLI perm/group resolve workspace target ref when HEAD differs                                        | rust-implementer | 75 min   |

## Dependencies

- **Blocks:** Sprint 06a
- **Dependent on:** Sprint 02, Sprint 03, Sprint 04

## PRD Coverage

- **Use cases:** UC-AUTHZ-01, UC-AUTHZ-03 (`perm list` scoping, relocated here), UC-GRPS-01, UC-GRPS-02
- **Criteria:** T-AUTHZ-007/021/025, T-GRPS-001/002/006/010/011

## Capability Coverage

- **CAP-AUTHZ-01** — every governance write verb authorizes `administration:write` (own ∪ groups, read at the
  target ref) via the AUTHZ-006 guard before mutating config; `perm list` enforces the reconnaissance scope.
- **CAP-CONFIG-01** — the CLI write path writes **inert-until-committed** config to the working tree only;
  effectiveness comes from the next target-ref read, so a self-grant on a feature head cannot authorize the
  same change.

## Coverage Notes

- **Authors the `but-api` perm/group governance functions reused as Tauri commands in Sprint 06a.** The
  `perm_list`/`perm_grant`/`perm_revoke`/`group_create`/`group_grant`/`group_add_member`/
  `group_remove_member`/`group_list` Tauri commands (`10-technical-requirements/04-api-design.md`) wrap the
  **same** `but-api` functions this sprint writes. CLI-001/CLI-002 must site them at the `but-api` boundary
  (not inline in `crates/but/`) so Sprint 06a's MGMT-IPC tasks can re-expose them without a parallel
  implementation. `but group delete` / `group_delete` is a Sprint-06a/UC-MGMT-03 surface and is out of scope
  for Sprint 05. Sprint 05 ships no delete CLI variant, no `group_delete` API function, and no placeholder or
  stub; CLI-REM-001 proves that boundary.
- **Net-new persisted writer (the inert-until-committed contract is structural).** The TOML writer performs a
  read-modify-write on the working-tree `.gitbutler/permissions.toml`, preserving unrelated entries; it MUST
  NOT commit, stage selectively, or touch any ref. The "PENDING" marker in `perm list` and the ref-pin caveat
  string are the operator-visible proof that the change is not yet in effect — they are not a substitute for
  the structural target-ref read.
- **Admin-write gating re-grounds to AUTHZ-006 (Sprint 02), not a fresh check.** Each mutating verb composes
  the AUTHZ-006 `administration:write` guard (`authorize(Authority::AdministrationWrite)` against the
  target-ref config) before the write. Non-admin denial returns the `perm.denied` contract + exit 1.
- **`perm list` scoping (T-AUTHZ-025, relocated from Sprint 02).** `but perm list --principal <other>` is
  denied `perm.denied` unless the caller is `<other>` or holds `administration:read` — the
  no-topology-reconnaissance property. `--principal` omitted resolves to the caller's own effective set.
- **CLI test economy (`crates/but/AGENTS.md`).** CLI integration tests are expensive — prove the persisted
  write + denial determinism primarily at the `but-api` function layer (real `but-authz` + real git), and
  reserve the `crates/but/tests/` snapbox harness for the happy-path verb wiring + the ref-pin-caveat /
  PENDING stdout contract. Use `env.but(...).assert()` with `.stdout_eq`/`.stderr_eq` + `[..]`/`...`
  wildcards; never `std::process::Command::new("git")` (use sandbox `invoke_bash`/`invoke_git`).
- **Honesty invariants still apply:** no role-name branching, no human-vs-AI predicate, no `Permission`
  overload in any enforcement/authorization path (grep-asserted by the AUTHZ-007/008 build-gates the
  implementer must not regress). The functional-permission tokens (`reviews:write`, `administration:write`)
  are typed `Authority` values, never role strings; group/principal names (`code-reviewers`,
  `rust-implementer`) are config DATA, not enforcement branches.
- **Implementation is out of scope for this artifact:** these are TDD **task contracts**. The Rust (the TOML
  writer, the `but perm`/`but group` verbs, the `but-api` functions, the admin-gating composition, the
  perm-list scoping) is written at execution time by `/kb-run-sprint`, RED→GREEN against these specs.

## Red-Hat Review Summary

Expanded by `/kb-sprint-tasks-plan` on 2026-06-19 — **1 full red-hat goal loop, 2 cycles + retained-writer
remediation + deterministic confirmation**. Both tasks fakeability-CLEAN (`validate_scenario.py`, 0 CRITICAL on
all 13 behavioral ACs) · `proposed_by` tripwire 2/2 · avg rubric 115/115 · stable gapless AC/TC ids.

A fresh adversarial panel (`rust-reviewer` + `security-auditor`, no authoring context) BLOCKed the first draft
with **3 CRITICAL + 5 MEDIUM + 3 LOW** spec-correctness findings the rubric + fakeability gates cannot see —
all coverage-side (no locked-PRD escalation), all remediated by the retained `rust-planner` and re-verified by a
fresh cycle-2 `rust-reviewer` as **genuinely closed with real teeth** (not cosmetic patches):

- **S1 (CRITICAL)** — the AUTHZ-007/008 honesty grep (`invariant_build_gates.rs` `ENFORCEMENT_PATHS`) did not
  cover the **net-new** `crates/but-api/src/legacy/governance.rs`, so "the grep stays green" was _vacuous_ — a
  role-label / human-vs-AI branch in the file that makes the governance authorization decision would not fail
  CI. Remediated: CLI-001 adds `governance.rs` to `ENFORCEMENT_PATHS` **plus** an `AUTHORITY_POSITIVE_PATTERN`
  assertion (additive-only; a stub/empty governance.rs now fails), with a covering AC/TC; CLI-002 keeps its
  `group_*` inside the now-covered file and must not touch the grep.
- **R1 (CRITICAL)** — gate step 7 (admin `perm revoke` → exit 0) had **no covering AC**; the misnamed AC only
  proved the _denial_ path, so a stub `perm_revoke → Ok` removing nothing would pass. Remediated: CLI-001 AC-4 —
  positive revoke (token removed, unrelated preserved, idempotent no-op byte-unchanged).
- **R2 (CRITICAL)** — `T-AUTHZ-007` (seeded day-one permissions) was claimed in coverage but provable by no AC
  (all fixtures pre-seeded; inert writes never prove a seeded principal _authorizes_). Remediated: CLI-001 AC-3 —
  first-grant into an **absent** config registers a `[[principal]]` block AND, once committed, the seeded
  principal authorizes (day-one effectiveness), via the new `perm_governance_seed` fixture.
- **MEDIUM** — `--as` identity-override clap-rejection on the new verbs (S2); new-verb fail-closed paths
  (unparseable token, unset `BUT_AGENT_HANDLE`, undefined-group `group_grant`) (S3); first-grant registration
  (R3, folded into AC-3); the wire-struct round-trip contradiction — blessed `#[derive(Serialize)]` as additive +
  downgraded "byte-verbatim" to **value-preserving** on a successful write, decision (a) (R4); and the
  perm-list scope predicate pinned as a genuine self-or-`administration:read` decision (not a blanket
  unknown-principal `perm.denied`, the live `resolve_principal_from_env` trap) (R5).
- **LOW** — hard `ref_id(target) before==after` inert assertion (S4); workspace-target-ref shim note (R6);
  dual `args/mod.rs` edit-site citation (R7).

**Grounding:** both panels independently confirmed **zero fabricated surfaces** — every cited API
(`enforce_administration_write_gate`, `classify_error`, `load_governance_config`, `normalize_permissions`, the
wire structs, `resolve_principal_from_env`, `write_worktree_permissions`/`committed_blob_text`/`ref_id`, the
`checkout-head-info` fixture, the `maintain`→`administration:read` desugar, the `Subcommands` dispatch/metrics
sites, the `temp-env`/`serial_test` deps) exists with the cited shape in the live crate. Cycle-2 verdict: **PASS**,
0 blocking open.

**Deferred advisories (LOW, ACCEPTED — carried to the implementer at GREEN, not a remediation run):**

- **N1** — the contracts assert build-dep facts (`toml = "0.9.10"` present, `toml_edit` absent, `temp-env`/
  `serial_test` in `but-api` dev-deps) load-bearing for the R4 decision (a); both gate with "FLAG only if a build
  surfaces otherwise", so an unmet assumption fails closed at compile time. Implementer confirms at GREEN.
- **N2** — CLI-001 AC-3 fixture wording says `principal_authorities("rust-implementer")` "is empty BEFORE"; the
  live API returns `Option<&AuthoritySet>` (`.is_none()` for an absent principal). Cosmetic; the `must_observe`
  ("0 new principal entries") makes the intent unambiguous.

## Sprint 05 Contract Corrections

Added by `/kb-sprint-tasks-plan` on 2026-06-20 from the retained `rust-planner` remediation plan after the
red-hat review at `.spec/reviews/red-hat-20260620T051414Z.md`.

- `but group delete` is explicitly out of scope for Sprint 05: no `Delete` clap variant, no `group_delete`
  `but-api` function, no `todo!()`, `unimplemented!()`, or placeholder.
- Remediation contracts use local stable `AC-n` / `TC-n` identifiers and every behavioral AC includes an
  executable scenario contract with `proposed_by: rust-planner`.
- All `perm.denied` assertions for the Sprint 05 CLI/API surface assert `code`, `message`, and non-empty
  `remediation_hint`.
- Group denial coverage must exercise all four group mutators: `group_create`, `group_grant`,
  `group_add_member`, and `group_remove_member`.
- Delegated admin coverage is required: `group_grant administration:write` must succeed for an existing admin
  and remain inert until committed to the target branch.
- CLI tests include `HEAD != workspace target` fixtures for both `but perm` and `but group`; feature-head
  self-grants must not authorize target-ref governance writes.

## Task Detail Files

Generated by `/kb-sprint-tasks-plan` on 2026-06-19. Remediation tasks added by `/kb-sprint-tasks-plan` on
2026-06-20.

- CLI-001-perm-cli-verbs.md
- CLI-002-group-cli-verbs.md
- CLI-REM-001-group-contract-hardening.md
- CLI-REM-002-perm-failclosed-denial-hints.md
- CLI-REM-003-cli-target-ref-resolution.md
