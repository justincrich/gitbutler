# Red-Hat Review Report — Sprint 04 (GATES Deepening)

**Report Date**: 2026-06-22
**Target**: Sprint 04 (GATES Deepening)
**Reviewed By**: rust-reviewer (adversarial re-review)
**Master HEAD inspected**: `c789845940` (Sprint 04 review); remediation branched from there.

## Executive Summary

**4 actionable findings remediated.** Sprint 04's core scope (GATES-006 per-required-group matrix, GATES-007 mechanism-agnostic commit gate, GATES-008 standalone target-ref-only proof, GATES-REM-001..007) is substantially complete with genuine behavioral tests — the per-required-group "only-one-blocked" matrix is proven with structured `denial.unmet` assertions, the mechanism-agnostic gate is wired at all three but-api public seams AND the CLI, the target-ref-only discriminating proof exists, and the no-human-vs-AI grep is clean.

Four gaps required remediation:
- G1+G2: GATES-008 AC-2 verifying script was a false positive (baseline predated STEER-001 cross-cutting steering field additions).
- G3: CLI vs library config_ref asymmetry — governed repo with `target_ref=None` could be ungated on the CLI path.
- G5: Missing CLI-level gating test for `but branch update` (integrate).

## Remediation Summary (landed in worktree `kb-rrh-sprint4-remediate`)

1. **G1+G2** — Advanced `baseline_sha` in `tools/governance-checks/check_merge_gate_production_unchanged.sh` from `a80ce888a` (AUTHZ-004) to `1402ce0132` (the most recent pre-Sprint-4 commit touching merge_gate.rs, which is Sprint-02's `Authority::Merge` qualification, including STEER-001's carrier-field additions). Documented the baseline choice and reasoning in the script header. The GATES-008 contract ("must add ZERO lines to merge_gate.rs") is preserved.
2. **G3** — Exposed `workspace_config_ref` as a public function in `but-api::branch` and rewired the CLI gate at `crates/but/src/command/branch/apply.rs` to use it. The CLI now uses the SAME governance-aware config_ref resolver as the library path (target_ref → `first_governed_ref` fallback). A governed repo with `target_ref=None` but committed `.gitbutler/*.toml` is now gated from the CLI.
3. **G5** — Added `integrate_readonly_denied` test in `crates/but/tests/but/command/branch/update.rs` that drives the real CLI on a governed workspace, asserts `perm.denied` naming `contents:write`, and asserts the workspace ref is unchanged.
4. **G4** — No remediation required. `branch_apply_readonly_denied` already exercises both the ro-denied and dev-proceeds paths (with fresh `governed_apply_env` per phase); the reviewer's "no happy-path test" classification was a misread of the test name.

## AC VERDICT TABLE (Post-Remediation)

| Task | AC Item | Verdict |
|------|---------|---------|
| GATES-006 AC-1 [PRIMARY] | AI-only + human-only blocked (only-one-blocked matrix) | PASS |
| GATES-006 AC-2 | Both-present proceeds | PASS |
| GATES-006 AC-3 | No human-vs-AI branch; no role-name literal | PASS |
| GATES-007 AC-1 [PRIMARY] | worktree_integrate → branch.protected | PASS |
| GATES-007 AC-2 | apply/integrate → perm.denied; contents:write proceeds; no-target skip | PASS |
| GATES-007 AC-3 | DryRun gating + target-ref-pinned + shared helper | PASS |
| GATES-008 AC-1 [PRIMARY] | Feature-head requirement-drop ignored (discriminating) | PASS |
| **GATES-008 AC-2** | NO competing classification; AUTHZ-004 remains owner | **PASS (post-fix — was FAIL)** |
| GATES-REM-001 AC-1 [PRIMARY] | CLI apply readonly denied | PASS |
| GATES-REM-001 AC-2 | CLI apply contents:write proceeds | PASS (covered by `branch_apply_readonly_denied` second phase) |
| GATES-REM-001 AC-3 | CLI apply ungoverned proceeds | PASS |
| GATES-REM-002 AC-1..3 | Sprint human-testing gate prose fixed | PASS |
| GATES-REM-003 AC-1 [PRIMARY] | Dual-member approval blocks (distinct-identity policy) | PASS |
| GATES-REM-003 AC-2 | Disjoint groups still proceed | PASS |
| GATES-REM-003 AC-3 | Policy documented | PASS |
| GATES-REM-004 AC-1 [PRIMARY] | Governed + missing target fail-closed | PASS |
| GATES-REM-005 AC-1 [PRIMARY] | Scripts exist and pass | **PASS (post-fix — was FAIL on check_merge_gate_production_unchanged.sh)** |
| GATES-REM-006 AC-1 [PRIMARY] | Gate-before-guard source contract | PASS |
| **GATES-REM-007 AC-1 [PRIMARY]** | Zero-diff script exits 0 | **PASS (post-fix — was FAIL)** |

## Confidence Summary

| Confidence | Count | Items |
|---|---|---|
| **HIGH** (3+ proofs) | All GATES-006/007/008 + GATES-REM-001..007 ACs | Verified PASS with file:line evidence |
| **MEDIUM** (pre-remediation) | G1+G2 (false-positive script), G3 (CLI bypass on governed repo without target_ref), G5 (missing CLI integrate test) | All remediated |
| **LOW** | G6 (fixture helper fragility), G7 (first_governed_ref O(n) scan) | Tracked for future cleanup |

## Post-Remediation Gate State

| Gate | Result |
|---|---|
| `cargo test -p but-api` | ✅ all green |
| `cargo test -p but-authz` | ✅ 50/0 |
| `cargo test -p but branch_apply_readonly_denied` | ✅ PASS |
| `cargo test -p but integrate_readonly_denied` | ✅ PASS (+1 new) |
| `cargo clippy -p but-api -p but-authz --all-targets -- -D warnings` | ✅ clean |
| `cargo fmt --check` | ✅ clean |
| `bash tools/governance-checks/check_merge_gate_production_unchanged.sh` | ✅ OK |
| `bash tools/governance-checks/check_no_role_literals.sh` | ✅ OK |
| `bash tools/governance-checks/check_gate_helper_parity.sh` | ✅ OK |
| `python3 tools/governance-checks/check_gate_before_guard.py` | ✅ OK |

## Status

**REMEDIATED** — Sprint 04's GATES-008 AC-2 verifying script is honest, the CLI bypass on governed-no-target repos is closed, and CLI integrate is now gated-tested. Fixes pending merge to master via the `kb-rrh-sprint4-remediate` branch.
