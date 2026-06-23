# Red-Hat Review Report — Sprint 03 (GRPS Groups + Ref-Pin)

**Report Date**: 2026-06-22
**Target**: Sprint 03 (GRPS Groups + Ref-Pin)
**Reviewed By**: rust-reviewer (adversarial re-review)
**Master HEAD inspected**: `4558370a15` (Sprint 03 review); remediation branched from there.

## Executive Summary

**4 HIGH-confidence MEDIUM-severity gaps remediated.** Sprint 03's core acceptance criteria (GRPS-001 and GRPS-002) are sound — the effective-set union correctly resolves group-only members, the group permission ceiling is proven, the self-grant-inert property is enforced against real `enforce_merge_gate`, and target-ref-only reads are proven. All 4 FIX-GRPS-* remediation tasks declared as DEFERRED in SPRINT.md were unimplemented. Per the user's instruction to remediate all HIGH/MEDIUM findings, all four were closed.

## Remediation Summary (landed in worktree `kb-rrh-sprint3-remediate`)

1. **FIX-GRPS-001-EMPTY-START-CONTROL** (GAP-R1) — Refactored `governed_repo` into `governed_repo_with(empty_start: bool)`; added `empty_start_fails_closed_config_invalid` test that drives the previously-dead `AUTHZ_EMPTY_START` fail-closed control deterministically (no env mutation, no new deps, no `unsafe set_var`). The existing `governed_repo()` continues to honor the env var for external harnesses.
2. **FIX-GRPS-002-AC3-TEETH** (GAP-R2) — Removed the trailing `git checkout main` from `self_grant_admin_repo` so HEAD stays on `feat-admin`. A HEAD-peel mutation in `load_governance_config` would now read the feat-admin tree (which carries the self-grant) and authorize when it should deny — AC-3 now has real teeth. Added `git checkout main` inside `land_admin_write` so landing still advances `refs/heads/main`.
3. **FIX-GRPS-002-AC3-TEETH AC-3** (GAP-R3) — Prepend `git remote remove origin 2>/dev/null || true` before `git remote add origin` in `merge_gate_self_escalation.rs` to make the fixture idempotent on re-runs.
4. **FIX-GRPS-RED-EVIDENCE-CONTRACT** (GAP-R4) — Wrote `.tmp/GRPS-001/mutation-evidence.md` and `.tmp/GRPS-002/mutation-evidence.md` documenting the specific code mutations that falsify each AC plus the catching tests. Waived `requires_red_evidence` to `false` on both task contracts with recorded reason, falsifiability_substitute pointer, waived_on date, and waived_by attribution per the contract waiver policy.

## AC VERDICT TABLE (Post-Remediation)

| Task | AC Item | Verdict |
|------|---------|---------|
| GRPS-001 AC-1 [PRIMARY] | group-only member authorizes review, denies merge | PASS |
| GRPS-001 AC-2 | effective_authority == principal_authorities for all paths | PASS |
| GRPS-001 AC-3 | delegated-admin ceiling proven | PASS |
| GRPS-001 AC-4 | claims do not widen union even with group backing | PASS |
| GRPS-002 AC-1 [PRIMARY] | feat-author self-add to maintainers on feature head denied merge | PASS |
| GRPS-002 AC-2 | landed membership clears merge-authority step | PASS |
| GRPS-002 AC-3 | self-grant administration:write inert until landed | **PASS (post-fix — HEAD stays on feat-admin, real teeth)** |
| GRPS-002 AC-4 | membership read only from target ref | PASS |
| **FIX-GRPS-001-EMPTY-START-CONTROL AC-1..3** | dead AUTHZ_EMPTY_START control exercised in-process | **PASS (post-fix — was DEFERRED)** |
| **FIX-GRPS-002-AC3-TEETH AC-1..4** | AC-3 has teeth; idempotent fixture | **PASS (post-fix — was DEFERRED)** |
| **FIX-GRPS-RED-EVIDENCE-CONTRACT AC-1..4** | mutation evidence + waiver recorded | **PASS (post-fix — was DEFERRED)** |

## Confidence Summary

| Confidence | Count | Items |
|---|---|---|
| **HIGH** (3+ proofs) | 8 | All 8 GRPS-001/002 ACs verified PASS with file:line evidence |
| **MEDIUM** (pre-remediation) | 4 | GAP-R1 (dead control), GAP-R2 (weak AC-3 teeth), GAP-R3 (non-idempotent fixture), GAP-R4 (unwaived contracts) — all remediated |
| **LOW** | 4 | GAP-R5 (unwrap_or_else in effective_authority — correct fail-closed by design), GAP-R6 (duplicate config loader — tracked for Sprint 04), GAP-R7 (Box<dyn Error> in doc test — cosmetic), GAP-R8 (governance_present fail-closed on unresolvable ref — intentional) |

## Post-Remediation Gate State

| Gate | Result |
|---|---|
| `cargo test -p but-authz` | ✅ 50/0 (+1 new: empty_start_fails_closed_config_invalid) |
| `cargo test -p but-api --test merge_gate_self_escalation` | ✅ 2/0 |
| `cargo clippy -p but-authz -p but-api --all-targets -- -D warnings` | ✅ clean |
| `cargo fmt --check` | ✅ clean |

## Status

**REMEDIATED** — Sprint 03's DEFERRED FIX tasks are now implemented. The self-grant-inert negative control has real teeth (HEAD stays on feature branch). The empty-start fail-closed control is exercised in-process. The RED-evidence contract is honestly waived with documented falsifiability substitutes. Fixes pending merge to master via the `kb-rrh-sprint3-remediate` branch.
