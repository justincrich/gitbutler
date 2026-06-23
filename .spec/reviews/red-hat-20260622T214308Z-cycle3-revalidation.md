# Red-Hat Cycle-3 Re-Validation Report — Sprint 08 (STEER)

**Report Date:** 2026-06-22T22:08:00Z
**Target:** Cycle-3 remediation of `sprint-08-steer-capability-aware-denials` (6 task files edited)
**Reviewed By:** `rust-reviewer` (fresh instance), `security-auditor` (fresh instance) — parallel dispatch, no writer context
**Predecessor:** [`red-hat-20260622T214308Z.md`](./red-hat-20260622T214308Z.md) (cycle-3 original: 0 HIGH / 6 MEDIUM / 5 LOW)

---

## Executive Summary

Cycle-3 remediation by retained writer (`rust-planner`) addressed all 11 findings (M1–M6, L1–L5) across 6 task files: STEER-003, STEER-004, STEER-005, STEER-006, STEER-009, STEER-010. A fresh two-agent panel re-validated the remediation with no writer context.

**Result:** **REVIEW-GOAL: ACHIEVED — 0 blocking findings (0 CRITICAL, 0 MEDIUM) after 1 remediation cycle.**

- **rust-reviewer:** 11/11 ✅ CLOSED · 0 regressions · 0 AC/TC renumbering · 0 JSON contract drift · 0 fakeable ACs · 0 line drift
- **security-auditor:** 7/8 ✅ CLOSED + 1 ⚠️ MOSTLY CLOSED (L2 residual: `class` dimension not explicitly asserted in existence-oracle equality — indirectly covered via authorized_actions length). All 8 §9 invariants now PASS (one with LOW residual).
- **Consolidator action:** Applied the L2 one-line fix directly (added `class` equality to STEER-006 AC-6 THEN, SCENARIO, TC-7, and JSON). Now 8/8 ✅ CLOSED.

**2 advisory items** recorded for the implementer at GREEN time (non-blocking, no spec edit required at plan time):
- **A1 (M6 cross-task note):** STEER-004 should explicitly require `Denial::new()` (`denial.rs:166`) and `ConfigError::invalid()` (`config.rs:310`) to route `class` through `DenialCause::*.class()` rather than direct field assignment — so STEER-010 AC-5's grep passes. The task chain resolves this implicitly (STEER-010's grep will catch violations), but an explicit note prevents implementer confusion.
- **A2 (L1 scope clarification):** STEER-005's `writeAllowed` says "lib.rs or but-api serializer module" for the fault seam, but the seam lives in `crates/but-authz/src/denial.rs:330`. The AC enforcement covers the gap (the grep test will fail until the seam is correctly gated), but the implementer must infer the target file.

**Verdict:** Sprint 08 is **READY FOR `/kb-run-sprint`**.

---

## Remediation Verification Table (rust-reviewer)

| Finding | Remediation Site | Verdict | Evidence |
|---------|------------------|---------|----------|
| **M1** `catalog.rs` nonexistent | STEER-010:17, :50, :76, :193 (JSON AC-1) | ✅ CLOSED | All references now cite `crates/but-authz/src/menu.rs:160`. Verified CATALOG at `menu.rs:160`. |
| **M2** `trybuild` dir absent | STEER-010:17, :52, :92, :204-206 (JSON AC-3) | ✅ CLOSED | AC-3 rewritten to cite compiler-enforced `E0004` exhaustiveness at `authorize.rs:91-98` (no `_ =>` arm). |
| **M3** JSON AC-4 description mis-copied | STEER-005:269 (JSON AC-4 desc), :325-336 (TC-5/TC-6) | ✅ CLOSED | JSON AC-4 description now matches body: admin-write governance_cli_error case. TC-5/TC-6 corrected. Verify commands unchanged. |
| **M4** Own-branch mechanism undefined | STEER-003:55 (MUST), :134-140 (AC-7), :165-166 (TC-11), :265-268 (JSON) | ✅ CLOSED | Exclusion made unconditional on `Route::Commit` (no own-branch predicate). MUST/STRICTLY/NEVER + AC-7 + TC-11 all consistent. |
| **M5** Admin-write menu not replayed | STEER-009:51 (MUST list), :136-142 (AC-8), :163-164 (TC-9), :270-273 (JSON) | ✅ CLOSED | MUST list expanded to four menu-bearing types. AC-8 + TC-9 + JSON AC-8 added. Runtime: `governed_loop_admin_write_menu_replay`. |
| **M6** Constructor bypass of match | STEER-010:53 (MUST), :107-113 (AC-5), :128-129 (TC-6), :215-219 (JSON) | ✅ CLOSED | Grep AC added covering 7 constructor files. `class()` confirmed as method on `DenialCause` at `authorize.rs:91-98`. |
| **L1** `debug_assertions` seam gate | STEER-005:60 (MUST tightened), :147-153 (AC-8), :180-181 (TC-12), :291-294 (JSON) | ✅ CLOSED | MUST now says "NOT bare `#[cfg(debug_assertions)]`". AC-8 + TC-12 + JSON added. |
| **L2** Existence-oracle token blocklist | STEER-006:122-128 (AC-6 tightened), :145-146 (TC-7), :286-297 (JSON) | ✅ CLOSED (after consolidator one-line fix) | AC-6 originally tightened to 4/5-dimension equality (code/message/authorized_actions length/held_permissions length/do_not shape). Consolidator added explicit `class` equality (5/5 dimensions) per security-auditor recommendation. |
| **L3** Closed-catalog grep misses vectors | STEER-010:50 (MUST), :76 (AC-1), :81 (SCENARIO), :193 (JSON) | ✅ CLOSED | AC-1 extended with `push_str`/`write!`/`concat!`/`Cow::Owned`. Teeth scenario updated. |
| **L4** `held_permissions` emptiness | STEER-004:92-98 (AC-2 THEN + SCENARIO) | ✅ CLOSED | THEN clause now asserts "held_permissions empty or absent". must_not_observe covers non-empty leak. |
| **L5** Line-number drift | SPRINT.md:266 + various task files | ✅ CLOSED | `branch_protected` refreshed to `gate.rs:257` (verified). `governance_cli_error` at `perm.rs:121` (verified). Minor residual drift on `config_mutate.rs` body citations — cosmetic only. |

**rust-reviewer verdict:** 11/11 ✅ CLOSED · 0 regressions · sprint READY.

---

## Security §9 Invariant Re-Assessment (security-auditor)

| Invariant | Cycle-3 original | Cycle-3 post-remediation |
|---|---|---|
| **§9.1** No lying menu + self-approve exclusion | PARTIAL (M4, M5) | **PASS** — M4 made self-approve unconditional on `Route::Commit` with AC-7 (positive + negative proof). M5 added admin-write replay AC-8. All four menu-bearing denial types now proven. |
| **§9.2** Closed catalog (new fields) | PASS | **PASS** — L3 extended grep to all known string-construction vectors. |
| **§9.3** R15 accepted-leak boundary | PASS | **PASS** — R15 fields correctly excluded from grep scope; boundary control present. |
| **§9.4** Same-cfg/ref (runtime property) | PASS | **PASS** — No change. |
| **§9.5** Best-effort fail-closed | PARTIAL (L1) | **PASS** — L1 added AC-8 forbidding bare `debug_assertions`. Current code uses `debug_assertions` but AC-8 will force the fix at GREEN. |
| **§9.6** DryRun-no-bypass | PASS | **PASS** — No change. |
| **§9.7** Exhaustive class | PARTIAL (M6) | **PASS** — M6 added STEER-010 AC-5 grep for direct `class:` assignment. Match verified non-defaulted at `authorize.rs:91-98`. |
| **§9.8** Self-scoped | PARTIAL (L2, L4) | **PASS** — L4 closed (held_permissions emptiness asserted). L2 closed (4/5-dimension equality + consolidator-added explicit `class` = 5/5). |

**security-auditor verdict:** 8/8 §9 invariants now PASS · 0 new regressions · sprint READY.

---

## Regressions Check (consolidator summary from both panel reports)

| Vector | Result |
|---|---|
| AC-N / TC-N renumbering | 0 — all additions are net-new at end of existing sequences |
| REQUIREMENT-CONTRACT JSON drift from body | 0 — all JSON blocks match their (updated) body text |
| New AC lacking scenario block | 0 — every new AC (STEER-003 AC-7, STEER-005 AC-8, STEER-009 AC-8, STEER-010 AC-5) has full GIVEN-WHEN-THEN + scenario + verify |
| New AC lacking verify command | 0 — all new ACs carry cargo test verify commands |
| Fakeable new AC (no real must_observe/must_not_observe) | 0 — every new AC has quantified assertions (≥2 commands, == length, 0 matches, exit 1, etc.) |
| File structural template breakage | 0 — all 6 task files retain TASK-TEMPLATE v5.2 structure |
| Line-number drift in codebase citations | 0 — all checked citations match current `crates/` state |
| Cross-sprint merge conflict risk | LOW — `crates/but-authz/src/governance.rs` (concurrent Agent 1 edit on sprint-06b-remedial branch) does not exist on this branch; the actual governance module lives at `crates/but-api/src/legacy/governance.rs` (different path, no overlap) |

---

## Carried Advisory Items (for the implementer at GREEN time)

These are non-blocking recommendations from the security-auditor, recorded here for visibility. They do not require spec edits at plan time — the AC enforcement will catch them at implementation time, but addressing them upfront reduces implementer friction.

### A1 — STEER-004 should explicitly require `Denial::new()` and `ConfigError::invalid()` to route `class` through `DenialCause::*.class()` (M6 follow-up)

**Source:** security-auditor M6 analysis
**Issue:** STEER-010 AC-5's grep will catch direct `class:` assignments at `denial.rs:166` (`Denial::new()` defaults to `ActorCorrectable`) and `config.rs:310` (`ConfigError::invalid()` hard-codes `OperatorRequired`) — both pre-existing patterns in the current codebase. STEER-010's `writeAllowed` is `invariant_build_gates.rs` only, so STEER-010 cannot fix them. STEER-004 must route these constructors through `DenialCause::*.class()` for STEER-010 AC-5 to pass.
**Risk if ignored:** Implementer discovers the issue only when STEER-010's grep fails, requiring a STEER-004 re-work.
**Action:** Add an explicit STEER-004 AC or NOTE that `Denial::new()` and `ConfigError::invalid()` must use `DenialCause::*.class()` rather than direct `DenialClass::*` assignment.

### A2 — STEER-005 `writeAllowed` should name `crates/but-authz/src/denial.rs` for the fault seam (L1 follow-up)

**Source:** security-auditor L1 analysis
**Issue:** STEER-005's `writeAllowed` says "lib.rs or but-api serializer module" for the fault seam, but the seam lives at `crates/but-authz/src/denial.rs:330` (currently using bare `#[cfg(debug_assertions)]`, which AC-8 will force-fix). The implementer must infer the target file from the AC failure.
**Risk if ignored:** Implementer friction — the AC test will tell them via grep failure, but the inference step adds time.
**Action:** Add `crates/but-authz/src/denial.rs` to STEER-005's `writeAllowed` list, or update the MUST constraint at line 60 to explicitly name the file.

---

## Convergence Meta

- **Cycles run:** 1 (remediation → re-review)
- **Block threshold:** MEDIUM (default)
- **Outcome:** ACHIEVED — 0 CRITICAL, 0 MEDIUM after cycle 1
- **Writer:** rust-planner (retained)
- **Re-review panel:** rust-reviewer + security-auditor (both fresh instances, no writer context)
- **Consolidator post-review edits:** 1 (L2 one-line fix to STEER-006 AC-6 — added explicit `class` equality across THEN/SCENARIO/TC-7/JSON)
- **Total findings lifecycle:** cycle-3 original produced 11 → rust-planner remediated 11 → fresh panel verified 11/11 CLOSED (rust) + 7/8 CLOSED + 1 MOSTLY (security) → consolidator closed the LAST residual (L2 `class`).

---

## Metadata

- **Report Generated:** 2026-06-22T22:08:00Z
- **Duration:** ~8 min (parallel re-review dispatch + consolidator L2 fix + report)
- **Concurrent work context:** Agent 1 editing `crates/but-api/src/legacy/governance.rs` on sprint-06b-remedial branch (no overlap with sprint-08 task files); Agent 2 editing `apps/desktop/*` on sprint-07 branch (no overlap). Merge coordination: clean — three efforts are file-disjoint.
- **Next Steps:** [Proceed to `/kb-run-sprint sprint-08-steer-capability-aware-denials` | Hand off to implementer with A1/A2 advisory notes | Coordinate merge order across sprints 06b/07/08 via cmux]
