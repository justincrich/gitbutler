# Red-Hat Review Report — Sprint 05 (CLI `but perm` / `but group`)

**Report Date**: 2026-06-22
**Target**: Sprint 05 (CLI `but perm` / `but group`)
**Reviewed By**: rust-reviewer (adversarial re-review)
**Master HEAD inspected**: `a1eb33bc08` (Sprint 05 review); remediation branched from there.

## Executive Summary

**3 actionable findings remediated.** Sprint 05's core implementation is strong: the 2420-line governance writer has zero stubs, zero `unwrap()`/`expect()` in library code, every mutating verb composes the existing `enforce_administration_write_gate` before working-tree-only writes, every verb prints the ref-pin caveat, and the HEAD-vs-target-ref property is structurally enforced and tested. The honesty grep coverage was correctly extended to `governance.rs`. All 25+ acceptance criteria across CLI-001/002 + CLI-REM-001..003 have real tests at the appropriate layer.

Three findings required remediation:
- GAP-1: `--principal` flag optional on `but perm grant`/`revoke` (UX hardening).
- GAP-4: `classify_governance_error` dropped `remediation_hint` to `None` in the `AdminWriteGateError` fallback path.
- GAP-5: `perm_grant_with_repo` and `group_grant_with_repo` parsed tokens before admin gate (minor info leak).

The reviewer also flagged two non-defect observations:
- GAP-2: `group_delete_with_repo` exists despite SPRINT.md saying "no group_delete API function". This is an over-delivery for Sprint 06a Tauri reuse (documented in CLI test as SPEC-REPAIR-IPC-003). No remediation needed.
- GAP-3: No CLI `revoke` verb for `but group`. Acceptable — not required by spec; API function exists for Tauri reuse.

## Remediation Summary (landed in worktree `kb-rrh-sprint5-remediate`)

1. **GAP-1** — Added `required = true` to the `principal` flag in both `Grant` and `Revoke` subcommands in `crates/but/src/args/perm.rs`. Without this, omitting `--principal` would default to `""` and create/modify an empty-string principal.
2. **GAP-4** — In `classify_governance_error`'s `AdminWriteGateError` fallback (`crates/but-api/src/legacy/governance.rs`), downcast to `but_authz::ConfigError` and synthesize a meaningful `remediation_hint` naming `config.invalid` and the underlying error. The CLI payload now carries recovery context for malformed-config failures surfacing through the admin-write guard.
3. **GAP-5** — Reordered `perm_grant_with_repo` and `group_grant_with_repo` to call `enforce_administration_write_gate` BEFORE `parse_authorities`. A non-admin caller now always sees `perm.denied`, never the token-validation error (which previously leaked token validity information).

## AC VERDICT TABLE (Post-Remediation)

All ACs across CLI-001, CLI-002, CLI-REM-001..003 render **PASS**. Highlights:

| Task | Key AC | Verdict |
|------|--------|---------|
| CLI-001 AC-1..8 | grant/revoke/list inert-until-committed, admin-gated, ref-pin caveat, PENDING marker, recon scoping | PASS |
| CLI-002 AC-1..5 | group create/grant/add-member/remove-member/list, admin-gated, ref-pin caveat | PASS |
| CLI-REM-001 AC-1..5 | All 5 mutating verbs deny non-admin; delegated admin; duplicate create errors; no delete CLI verb | PASS |
| CLI-REM-002 AC-1..4 | Fail-closed on bad token / unset handle; denials carry remediation_hint | **PASS (post-fix — GAP-4 + GAP-5)** |
| CLI-REM-003 AC-1..3 | CLI uses workspace target ref, not HEAD; HEAD self-grant denied | PASS |

## Confidence Summary

| Confidence | Count | Items |
|---|---|---|
| **HIGH** (3+ proofs) | All 25+ ACs | Verified PASS with file:line evidence |
| **MEDIUM** (pre-remediation) | 3 | GAP-1 (optional --principal), GAP-4 (dropped remediation_hint), GAP-5 (token-parse-before-gate) — all remediated |
| **LOW** (observations) | 2 | GAP-2 (group_delete exceeds spec — over-delivery, no fix needed), GAP-3 (no CLI revoke — not required by spec) |

## Post-Remediation Gate State

| Gate | Result |
|---|---|
| `cargo test -p but-api` | ✅ all green (172+ tests) |
| `cargo test -p but perm` | ✅ 3/0 |
| `cargo test -p but group` | ✅ 5/0 |
| `cargo clippy -p but-api -p but --all-targets -- -D warnings` | ✅ clean |
| `cargo fmt --check` | ✅ clean |

## Status

**REMEDIATED** — Sprint 05's 3 actionable findings are closed. The over-delivery observations (GAP-2, GAP-3) require no code changes. Fixes pending merge to master via the `kb-rrh-sprint5-remediate` branch.
