# Red-Hat Review Report — Sprint 02 (AUTHZ Fail-Closed + Identity Confinement)

**Report Date**: 2026-06-22
**Target**: Sprint 02 (AUTHZ Fail-Closed + Identity Confinement)
**Reviewed By**: rust-reviewer (adversarial re-review)
**Master HEAD inspected**: `449dcb27d9` (Sprint 02 review), remediation branched from there.

## Executive Summary

**CRITICAL gap found and remediated.** Sprint 02 SPRINT.md declared three FIX-AUTHZ-* remediation tasks (FIX-AUTHZ-FORGE-FAILCLOSED, FIX-AUTHZ-FORGE-COVERAGE, FIX-AUTHZ-006-RED-EVIDENCE). Only two were implemented. **FIX-AUTHZ-FORGE-COVERAGE was missing entirely** — the AUTHORITY_POSITIVE honesty gate silently passed forge.rs even though forge.rs's `authorize()` calls were bare (no `but_authz::` qualification), and `invariant_build_gates.rs` lacked both AUTHORITY_POSITIVE and PERMISSION_CARRIER assertions over FORGE_GUARD. This is the exact false-coverage hole the sprint's own first-cycle red-hat flagged, and it remained open at master HEAD.

The core fail-closed implementation (merge_gate, admin_write_guard, identity confinement) was genuinely well-implemented: real integration tests, deterministic load→resolve→authorize ordering, proper denial-code discrimination, no role-keyed enforcement branches. The remediation closes the coverage gap and adds two defense-in-depth tests.

## Remediation Summary (landed in worktree `kb-rrh-sprint2-remediate`)

1. **`crates/but-api/src/legacy/forge.rs`** — Drop bare `authorize` import; fully-qualify all 4 authorize call sites as `but_authz::authorize`. This closes the CRITICAL false-coverage hole — `AUTHORITY_POSITIVE_PATTERN` now matches forge.rs.
2. **`crates/but-authz/tests/invariant_build_gates.rs`** — Extend `SPRINT_02_ENFORCEMENT_PATHS` to include `FORGE_GUARD`; add `AUTHORITY_POSITIVE` assertion over `FORGE_GUARD` so the honesty gate **enforces** (not just permits) the but-authz axis on the forge boundary.
3. **`crates/but-api/src/legacy/merge_gate.rs`** — Fully-qualify `Authority::Merge` as `but_authz::Authority::Merge` (G-3 defense-in-depth; the bare form was functional via the `but_authz::authorize` branch but fragile on call-form drift).
4. **`crates/but-api/tests/admin_write_guard.rs`** — Add `admin_write_guard_malformed_config_invalid_for_ghost_caller` test (G-4) — mirrors AUTHZ-004 AC-2's ghost+malformed→config.invalid coverage for the admin-write guard.
5. **`crates/but-api/tests/forge_guard.rs`** — Add `forge_guard_malformed_config_is_config_invalid` test (G-5) — proves the forge path's `governance_present → load_forge_governance_config → resolve → authorize` ordering surfaces `config.invalid`, not `perm.denied`.

## AC VERDICT TABLE (Post-Remediation)

| Task | AC Item | Verdict |
|------|---------|---------|
| AUTHZ-004 AC-1 | Unknown principal + no-handle → perm.denied | PASS |
| AUTHZ-004 AC-2 | Malformed config → config.invalid for both maint and ghost (config-load-first) | PASS |
| AUTHZ-004 AC-3 | Undefined require_approval_from_group → gate.review_required | PASS |
| AUTHZ-004 AC-4 | DryRun + unknown → perm.denied, nothing persisted | PASS |
| AUTHZ-005 AC-1..4 | Identity confinement (no in-band override, authority from config, honest env-re-export acknowledgment) | PASS |
| AUTHZ-006 AC-1..3 | Administration:write guard (denies non-admin, allows admin, fully-qualified Authority, malformed→config.invalid) | PASS |
| **AUTHZ-008 AC-1** | ENFORCEMENT_PATHS extended to merge_gate + config_mutate; role/label 0; len≥5 | **PASS (post-fix)** |
| **AUTHZ-008 AC-2** | AUTHORITY_POSITIVE on config_mutate AND merge_gate AND **forge_guard**; PERMISSION_CARRIER 0 on Sprint-02 surfaces | **PASS (post-fix — was FAIL)** |
| FIX-AUTHZ-FORGE-FAILCLOSED AC-1..3 | has_governance_config → governance_present swap; gates-only/permissions-only/both regression coverage | PASS |
| **FIX-AUTHZ-FORGE-COVERAGE AC-1..3** | AUTHORITY_POSITIVE over FORGE_GUARD; PERMISSION_CARRIER no-match over FORGE_GUARD; forge.rs fully-qualified | **PASS (post-fix — was MISSING)** |
| FIX-AUTHZ-006-RED-EVIDENCE AC-1..3 | RED/GREEN evidence + verify-manifest.json present and genuine | PASS |

## Confidence Summary

| Confidence | Count | Items |
|---|---|---|
| **HIGH** (3+ proofs) | 7 | All 7 AUTHZ-004/005/006/008 ACs + FIX-AUTHZ-FORGE-FAILCLOSED + FIX-AUTHZ-006-RED-EVIDENCE verified with file:line evidence |
| **MEDIUM** (pre-remediation) | 3 | G-1 (FIX-AUTHZ-FORGE-COVERAGE false-coverage hole), G-3 (bare Authority::Merge), G-4/G-5 (test gaps) — all remediated |
| **LOW** | 2 | G-6 (GOVERNANCE path added without task traceability — benign coverage increase), G-7 (RED evidence unused-import warning — artifact, expected) |

## Post-Remediation Gate State

| Gate | Result |
|---|---|
| `cargo test -p but-api` | ✅ all green (incl. 2 new tests) |
| `cargo test -p but-authz` | ✅ 49/0 |
| `cargo test -p but-authz --test invariant_build_gates` | ✅ 1/0 (forges positive AUTHORITY_POSITIVE match now enforced) |
| `cargo clippy -p but-api -p but-authz --all-targets -- -D warnings` | ✅ clean |
| `cargo fmt --check` | ✅ clean |

## Status

**REMEDIATED** — Sprint 02's declared scope is now fully implemented. The CRITICAL false-coverage hole (forge boundary) is closed. Fixes pending merge to master via the `kb-rrh-sprint2-remediate` branch.
