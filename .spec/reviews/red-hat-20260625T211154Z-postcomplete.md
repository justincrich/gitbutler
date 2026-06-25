# Red-Hat Review Report

**Report Date**: 2026-06-25T21:11:54Z
**Target**: Sprint 07 — STEER: Capability-Aware Denials (10 tasks, 54 ACs / 79 TCs)
**Reviewed By**: `rust-reviewer`, `security-reviewer` (fresh panel, no authoring context)
**Sprint Status (claimed)**: Completed (frontmatter) / In Progress (SPRINT.md body line 18 — inconsistency)
**Prior Reviews**: 2026-06-20 (planning), 2026-06-22 cycle-3 revalidation

## Executive Summary

A fresh adversarial panel re-reviewed the **shipped code** for Sprint 07 after completion. Both reviewers **APPROVE with advisories** — no CRITICAL or HIGH findings. All 54 ACs are verified PASS except one PARTIAL (admin-write menu enrichment). The eight core security controls (no lying menu, no self-approve, no enforcement weakening, operator-required isolation, no group-roster leak, no existence oracle, DryRun no-bypass, exhaustive `class` mapping) all hold. The implementation is real, not stubbed: zero `todo!()`/`unimplemented!()`/placeholder hits across STEER-touched files, all mandatory gates (`fmt`, `clippy -D warnings`, `cargo test -p but-authz`/`-p but-api`/`-p but` including `invariant_build_gates`) pass cleanly.

The findings that slipped through planning are coverage-side and defense-in-depth — none weaken enforcement, leak data, or turn deny→allow. Two MEDIUM findings warrant a follow-up sprint: the admin-write gate omits `with_authorized_actions` enrichment (G-2/M1+L1), and the `own_branch` self-approve mechanism is designed but never wired at any gate site (it's inert defense-in-depth because `AFFORDANCE_MAP` curation already excludes `but review approve`).

## HIGH Confidence Findings (Both Agents Agree)

- [ ] **Admin-write gate never enriches Denial with `with_authorized_actions`** | Severity: MEDIUM
      Agents: rust-reviewer (G-2), security-reviewer (L1)
      Evidence: `crates/but-api/src/legacy/config_mutate.rs:51-52` calls `but_authz::authorize(&principal, required, &cfg)?` without `.with_authorized_actions(..)`. Commit gate (`commit/gate.rs:146-157`) and merge gate (`merge_gate.rs:76-86`) both do this enrichment; admin-write path does not.
      Impact: Admin-write actor-correctable denials carry `authorized_actions: []` (no `but perm list` discovery affordance). STEER-005 AC-4 is formally PARTIAL: the carrier has the fields and the CLI serializer renders them, but the underlying menu is always empty.
      Fix: Call `denial.with_authorized_actions(&principal, &DeniedRoute::new(Route::Admin, DenialPredicate::Authority), &cfg)` on the `map_err` path in `enforce_administration_write_gate`, mirroring the commit/merge gates.

- [ ] **`DenialClass::default() = ActorCorrectable` is a silent-downgrade hazard** | Severity: LOW
      Agents: rust-reviewer (G-5), security-reviewer (R1+R2)
      Evidence: `crates/but-authz/src/denial.rs:25-35` derives `Default` with `ActorCorrectable` as default. Three `classify_error` sites call `error.class.unwrap_or_default()` when `ConfigError.class` is `None` (`commit/gate.rs:187`, `forge.rs:142`, `config_mutate.rs:80`). `Denial::new()` (`denial.rs:161-171`) also defaults to `ActorCorrectable`. Today every operational caller overrides `class` explicitly and `ConfigError::invalid()` always sets `Some(OperatorRequired)`, so the fallback never fires — but a future constructor that forgets `class` silently produces an `actor_correctable` payload.
      Fix: Either remove the `Default` derive (forcing explicit choice) or make the `classify_error` sites fail loudly via `expect("ConfigError must carry class")` instead of `unwrap_or_default()`.

## MEDIUM Confidence Findings (Single Agent — Not Cross-Validated)

- [ ] **`own_branch` self-approve mechanism is designed but never wired at any gate site** | Severity: MEDIUM
      Agent: security-reviewer (M1)
      Evidence: `DeniedRoute::own_branch` (`menu.rs:111`) and the exclusion check (`menu.rs:467`) exist for branch-scoped exclusion of `but review approve`, but no gate site calls `.with_own_branch(true)` — `commit/gate.rs:149,271`, `merge_gate.rs:79,146` all construct `DeniedRoute::new(...)` without it. Self-approve IS excluded (stronger than spec — it's excluded from ALL denials via `AFFORDANCE_MAP` curation, not just own-branch), so this is not a bypass. It's inert defense-in-depth plus dead code.
      Fix: Either wire `own_branch` with branch-ownership detection at gate sites (matching the spec), or document that `AFFORDANCE_MAP` curation is the sole exclusion mechanism and remove the unused `own_branch` machinery.

- [ ] **Existence-oracle test asserts token blocklist, not full message equality (SA-2 advisory not met)** | Severity: MEDIUM
      Agent: security-reviewer (control #6 PARTIAL)
      Evidence: `crates/but/tests/steer_discovery.rs:348-402` proves the `perm.denied` code is identical for unknown vs known targets and asserts the unknown-target message lacks `unknown`/`not found`/`no such` tokens. But the full-message-equality bar from the SA-2 deferred advisory (SPRINT.md:250-251) was not adopted.
      Fix: Tighten STEER-006 AC-6 to assert full message equality between unknown-target and existing-target denials.

- [ ] **Debug-only fault seam uses `cfg(debug_assertions)` instead of `cfg(test)` (SA2-1 advisory not adopted)** | Severity: MEDIUM
      Agent: security-reviewer (M2)
      Evidence: `crates/but-authz/src/denial.rs:330-339` gates `serialization_fault_forced()` on `#[cfg(debug_assertions)]`. A debug-profile (non-test) binary with `BUT_STEER_FORCE_SERIALIZATION_FAULT=1` could trigger the seam. Release builds compile it out (`#![cfg(not(debug_assertions))]` → `const false`). The deferred SA2-1 advisory (SPRINT.md:246-248) explicitly recommended `cfg(test)` or a dedicated feature flag.
      Fix: Switch the gate to `#[cfg(test)]` or `cfg(feature = "steer-fault-seam")`.

- [ ] **Enrichment §1 wording stale: "four carriers / three CLI serializers" but code ships six carriers / four sites** | Severity: MEDIUM
      Agent: rust-reviewer (G-1)
      Evidence: `enrichments/v1.4.0-capability-aware-denials/03-technical-requirements-delta.md:13` says four carriers. Shipped code covers six (adds `ForgeGateError` + `AdminWriteGateError`). Similarly under-counts CLI serializers. Recorded as "upstream advisory, non-blocking" in `SPRINT.md:240-242` but the source enrichment doc was never corrected.
      Fix: Update the enrichment §1 wording via a future delta-replan.

- [ ] **`undefined_required_groups` merge-gate path omits `do_not`** | Severity: LOW
      Agent: rust-reviewer (G-4)
      Evidence: `crates/but-api/src/legacy/merge_gate.rs:102-116` constructs a custom `MergeGateError` with `class: OperatorRequired` but `do_not: None`. The sibling `config_invalid` path (`merge_gate.rs:432-446`) correctly sets a do-not-retry `do_not`. Operator-required + empty menu is still strong enough to prevent pooling, so the impact is bounded.
      Fix: Set `do_not: Some("do not retry — an operator must fix the committed .gitbutler config")` on the undefined-required-groups error, or refactor to reuse `config_invalid`.

## LOW Confidence Findings (Single Agent, Code Quality)

- [ ] **`AUTHORIZED_ROUTE_COMMANDS` in `governance.rs` duplicates `AFFORDANCE_MAP`** — single-source drift risk. (`governance.rs:1551-1560` vs `menu.rs:277-355`) — security-reviewer L2. Fix: derive `self_authorized_actions` from `ROUTE_AUTHORITY_TABLE` directly.

- [ ] **`CATALOG` comment at `menu.rs:176-178` contradicts implementation** — says `but review approve` may appear on non-own-branch denials, but it never appears in any `AFFORDANCE_MAP` row. — security-reviewer L3. Fix: update the comment to reflect that curation excludes it unconditionally.

- [ ] **`whoami.rs:72-84` / `can_i.rs:73-85` CLI error serializers don't use `steer_envelope_from_parts`** — denials from `but whoami`/`but can-i` (cross-principal `perm.denied`) won't carry steering fields. — rust-reviewer G-7. Low impact: these are discovery verbs, primary output is `WhoamiOutcome`/`CanIOutcome` on success.

- [ ] **`AFFORDANCE_MAP` admin-write row has empty affordances** — `menu.rs:355` `(Route::Admin, DenialPredicate::Authority, &[])`. — rust-reviewer G-3. The enrichment §5 over-promised affordances (`read/inspect, request-config-change, discovery`) that aren't gated routes; the empty slice is the honest code choice. Recommend spec correction rather than code change.

- [ ] **`DenialClass` custom `Serialize` impl rather than `#[derive(Serialize)]`** — rust-reviewer G-6. No action: deliberate pattern for stable wire tokens.

## AC Verdict Coverage

`rust-reviewer` enumerated all 54 ACs across STEER-001..010: **53 PASS / 1 PARTIAL (STEER-005 AC-4, admin-write menu) / 0 FAIL**. The single PARTIAL cascades from the HIGH-confidence admin-write finding above — the carrier shape is correct, the underlying menu is empty. See the rust-reviewer task output for the full per-AC table with `file:line` evidence.

## Agent Contradictions & Debates

| Topic | rust-reviewer | security-reviewer | Assessment |
|-------|---------------|-------------------|------------|
| Admin-write menu severity | MEDIUM (G-2) | LOW (L1) | **MEDIUM carried.** Both agree on the mechanism; rust-reviewer weighs the AC-4 PARTIAL higher because it's an unmet contract. Security-reviewer weighs it lower because admin holders can run `but perm list` independently. Either way, the fix is small and uniform with peer gates. |
| Self-approve exclusion | Not flagged (treated AFFORDANCE_MAP curation as the spec mechanism) | MEDIUM (M1) — flagged the unwired `own_branch` machinery as dead code | **Both correct.** Security-reviewer's finding is defense-in-depth housekeeping, not a bypass. The spec language ("excluded on the caller's own branch") literally describes the unwired mechanism, so the code's stronger-than-spec behavior is a spec-vs-code reconciliation item. |
| No-lying-menu proof | PASS — `branch.protected` → feature-branch commit + review | PASS — verified across `AFFORDANCE_MAP` for all routes | **Agreement.** Core security property holds. |

## Recommendations by Category

1. **Gaps** — Three coverage gaps slipped past the planning-stage red-hat loop because they're runtime/wiring details invisible to spec review: (a) admin-write `with_authorized_actions` not called, (b) `own_branch` flag never set, (c) undefined-groups missing `do_not`. All three are small fixes; bundle into a Sprint 07 hardening task or fold into the next sprint's planner.
2. **Risks** — The `DenialClass::default()` silent-downgrade hazard is the only structural risk. It can't fire today but will the moment a future contributor adds a constructor path without setting `class`. Recommend making it impossible to construct an unclassified `Denial`/`ConfigError`.
3. **Assumptions** — Three SA/SA2 advisories from the planning review were ACCEPTED as "carried to GREEN" but were not adopted at GREEN: SA2-1 (`cfg(test)` for the fault seam), SA2-2 (full message equality for existence oracle). Both should either be retrofitted or explicitly re-accepted with a recorded rationale. The "same cfg/ref by construction" property is verified structurally (each gate loads cfg once and passes it through) but the temporal ref-advance window (security MED #4 from the enrichment) is not closed by STEER alone.
4. **Contradictions** — Two spec-vs-code reconciliations: (a) enrichment §1 carrier/serializer count is stale by 2 carriers + 1 site; (b) enrichment §5 admin-write affordance row lists three affordances that have no gated-route counterparts. Both are spec corrections, not code changes.

## Agent Reports (Summary)

- **rust-reviewer**: 52 PASS / 1 PARTIAL / 1 advisory across 54 ACs. Zero stubs. Verdict: **APPROVED with advisories**, recommend G-2 (admin-write menu enrichment) for follow-up remediation.
- **security-reviewer**: 6/8 controls PASS, 2/8 PARTIAL (#2 self-approve via stronger mechanism than spec, #6 token blocklist instead of full message equality). Zero CRITICAL/HIGH. Verdict: **APPROVED**. No privacy/data-exfil. No fail-open paths.

## Metadata

- **Agents**: `rust-reviewer` (Glob/Grep/Read/Bash/Write/Task — full tool access), `security-reviewer` (same)
- **Confidence Framework**: HIGH (both agents agree) · MEDIUM (single agent, mechanism verified) · LOW (single agent, code-quality only)
- **Report Generated**: 2026-06-25T21:11:54Z
- **Output**: `.spec/reviews/red-hat-20260625T211154Z-postcomplete.md`
- **Next Steps**:
  1. Remediate the admin-write `with_authorized_actions` gap (HIGH-confidence finding #1) — small, uniform with peer gates.
  2. Decide on `own_branch` wiring vs. dead-code removal (MEDIUM — reconcile code with spec language).
  3. Retrofit SA2-1 (`cfg(test)` seam) and SA2-2 (existence-oracle message equality) — either implement or formally re-accept with rationale.
  4. Correct enrichment §1 wording (carrier count) and §5 (admin affordances) via a future delta-replan.
  5. Sprint status field inconsistency in `SPRINT.md` (frontmatter `Completed` vs. body `In Progress`) — pick one.
