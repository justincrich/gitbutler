---
sprint: 03
sequence: 4
timeline: Phase 2 — Hardening
status: In Progress
proposed_by: rust-planner
milestone: sprint-03
prd: ../../README.md
roadmap: ../../ROADMAP.md
generated_by: kb-sprint-tasks-plan
---

# Sprint 03: GRPS Groups + Ref-Pin

**Sequence:** 4
**Timeline:** Phase 2 — Hardening
**Status:** In Progress
**Proposed by:** rust-planner
**Milestone:** — (`sprint-03`)

## Overview

The second hardening sprint. Sprint 01a/01b proved the positive governed loop and Sprint 02 made the
negative space fail-closed. Sprint 03 adds the net-new **grouping** capability — the GitHub-teams model —
on top of the per-principal authority primitive, and proves the one property that makes grouping *safe*
rather than an escalation vector: **group definitions, grants, and membership are the same committed,
ref-pinned governed config as every other permission, read at the target ref**.

Two properties, one per task:

- **Effective-set union + group permission ceiling (GRPS-001):** a principal's effective `AuthoritySet`
  is the union of its own grants and the grants of every group it belongs to, so a reviewer can hold
  `reviews:write` purely by membership in a `code-reviewers` group — and is still denied `merge`, which no
  source grants, with the `perm.denied` contract. The authorization check is unchanged in shape (does the
  effective set contain the required `Authority`); nothing downstream knows whether a permission came from
  a direct grant or a group. The **group permission ceiling** is made explicit: a group **may** hold any
  authority including `administration:write` (delegated admin), and granting it is an *accepted, named
  property* that itself requires `administration:write` — never a silent escalation.
- **Ref-pinned governed membership + self-grant-inert (GRPS-002):** group config — definitions, grants,
  and membership — is read at the **target ref** when authorizing a git action, exactly as
  `.gitbutler/permissions.toml` is. A change whose head adds its own author to a `merge`-holding
  `maintainers` group is judged against the membership committed on the target branch, so the change
  cannot authorize itself; likewise a head that self-grants `administration:write` is inert until landed
  on the target ref. This is the CAP-CONFIG-01 self-escalation contract, proven against real git.

Every gate proof draws from [`11-e2e-testing-criteria.md`](../../11-e2e-testing-criteria.md). This sprint
is **headless/CLI** — every property is verified against the real `but-authz` crate + real git, observing
the structured denial `{code, message, remediation_hint}` and exit code (or, for the positive cases, the
authorized action proceeding).

> **Drift note (the union already exists — this sprint hardens, consolidates, and proves it).** Sprint
> 01a's config loader (`but-authz` `config.rs`) already normalizes a principal's group memberships into a
> unioned effective set at load time, and `authorize.rs::effective_authority` unions group grants again at
> authorize time — a **redundant double-union** with no integration test asserting the GRPS contract.
> GRPS-001 must investigate the live crate, establish a single authoritative union (one source of truth),
> and add the missing GRPS integration proofs. GRPS-002's target-ref read is structurally inherited from
> the Sprint-01a `gix` loader; the net-new work is the **self-escalation integration test against real
> git** and the explicit target-ref-only membership read.

> **Re-grounding (the `but group` CLI verbs are Sprint 05, by design).** T-GRPS-001/002 ("`but group
> create`/`grant`/`add-member`") and T-GRPS-010's "CLI warns" assertion name CLI verbs that **do not exist
> until Sprint 05** (CLI-002). Following the Sprint-02 precedent (which re-grounded the admin-write
> persisted-write to its Sprint-05 consumer), GRPS-001/002 prove group **definition and membership** at
> the `but-authz` config + real-git integration layer — by committing `[[group]]` / membership TOML to the
> target ref directly and asserting the loader + resolver — NOT by driving a `but group` verb. The
> "inert-until-committed CLI warning" surface (T-GRPS-010 CLI half) is named as the Sprint-05 consumer with
> a `BLOCKED-UNTIL` note; this sprint proves the inert-until-committed *behavior* (the working-tree /
> feature-head edit has no authorization effect) against real git.

## Human Testing Gate

**Gate:** Gate passes when BOTH: [GRPS-01] a member with no direct grant is authorized via its group's
permission (union) and denied an action no source grants; AND [GRPS-02] a feature head that adds its own
author to a merge-holding group — or self-grants `administration:write` — is still denied, because
membership and grants are read only at the target ref.

### Test Steps

1. Seed a `code-reviewers` group with `reviews:write` and a member holding no direct review grant.
2. Run a review as that member → exit 0, authorized via the group (union).
3. Run a merge as that member → denied, exit 1, `perm.denied` (no source grants merge).
4. Create a feature head that adds its author to a `merge`-holding `maintainers` group.
5. Run merge from that feature head → denied, the target-ref membership governs.
6. Self-grant `administration:write` on a feature head, run the same config change → denied (inert until target-ref commit).
7. Commit the membership to the target ref, advance it, rerun the merge → now authorized.

## Tasks

| ID | Title | Agent | Estimate |
|----|-------|-------|----------|
| GRPS-001 | Effective-set union via group membership + group permission ceiling | rust-implementer | 150 min |
| GRPS-002 | Ref-pinned governed membership + self-grant-inert (target-ref-only read) | rust-implementer | 210 min |

## Dependencies

- **Blocks:** Sprint 04, Sprint 05
- **Dependent on:** Sprint 01b

## PRD Coverage

- **Use cases:** UC-GRPS-01, UC-GRPS-02, UC-AUTHZ-03 (self-grant-inert, relocated here)
- **Criteria:** T-GRPS-001..014, T-AUTHZ-032 (re-keyed T-AUTHZ-033 — self-grant-inert ref-pin)

## Capability Coverage

- **CAP-AUTHZ-01** — effective-set union resolution (own ∪ ⋃groups), authorize unchanged in shape (GRPS-001).
- **CAP-CONFIG-01** — group definitions + grants + membership read target-ref-only; a head that self-adds
  to a privileged group or self-grants `administration:write` cannot authorize the same change (GRPS-002).

## Coverage Notes

- **Depends on Sprint 01b (In Progress, not yet merged):** GRPS-002's self-escalation proof drives the
  merge gate built in Sprint 01b (GATES-003) and reuses the merge-gate fixture/forge seam. The task files
  plan against the documented 01b/02 contracts; if a contract drifts during its build, re-run
  `/kb-sprint-tasks-plan --only <id> --overwrite`.
- **`but group` CLI is Sprint 05 (re-grounding, by design):** T-GRPS-001/002/006 and the T-GRPS-010 "CLI
  warns" half name `but group create/grant/add-member` verbs that do not exist until Sprint 05 (CLI-002).
  GRPS-001/002 prove group definition + membership at the `but-authz` config + integration layer (committed
  `[[group]]` / membership TOML asserted via the loader + resolver), carrying a `BLOCKED-UNTIL` note for the
  CLI-warning surface. The inert-until-committed *behavior* is proven against real git here; the CLI warning
  *message* is a Sprint-05 consumer obligation.
- **Drift / redundant double-union:** the union is computed both in `config.rs::normalize_permissions`
  (load-time) and `authorize.rs::effective_authority` (authorize-time). GRPS-001 must establish a single
  authoritative union and prove the GRPS contract; it must not leave two divergent union paths.
- **Group permission ceiling honesty (T-GRPS-013):** a group holding `administration:write` (delegated
  admin) is an accepted, named property — setting it requires `administration:write`, it is never a silent
  escalation, and the audit surface of "who can change governed config" is the union of direct holders and
  members of admin-holding groups. GRPS-001 proves this as a named property, not a defect.
- **Honesty invariants still apply:** no role-name branching, no human-vs-AI predicate, no `Permission`
  overload in any enforcement path (grep-asserted by the AUTHZ-007/008 build-gates the implementer must not
  regress). Fixture group/handle names (`code-reviewers`, `maintainers`) are test DATA, not enforcement
  branches.
- **Implementation is out of scope for this artifact:** these are TDD **task contracts**; the Rust (the
  consolidated union, the GRPS integration tests, the self-escalation real-git proof) is written at
  execution time by `/kb-run-sprint`, RED→GREEN against these specs.

## Red-Hat Review Summary

Expanded by `/kb-sprint-tasks-plan` on 2026-06-19 — **1 full red-hat cycle + retained-writer remediation +
deterministic confirmation**. A fresh panel (`rust-reviewer` + `security-auditor`, no authoring context) BLOCKed
the first draft with specification-correctness findings the rubric + fakeability gates cannot catch — most
notably two `rust-reviewer` CRITICALs:

- **R1** — the union "redundant double-union *divergence*" the task was framed to fix is a **fiction**:
  `effective_authority` uses `principal_authorities(id)` as its fold base and re-unions only group grants
  `config.rs` already folded, so it is provably **equal** to `principal_authorities(p)` for every `p`. AC-2's
  "divergence-regression" negative control could never fire. Re-grounded GRPS-001 to an honest behavior-neutral
  simplification (remove the dead authorize-time re-union) with an **equality pin**, and forbade the unviable
  consolidation option that would break group-only members (**R8**).
- **R2** — GRPS-002 AC-2's "the rerun authorizes (Ok)" was **impossible**: `enforce_merge_gate` authorizes
  `Authority::Merge` before the review branch, and the fixture's unapproved `min_approvals=1` gate returns
  `gate.review_required` after the merge authority passes. Re-scoped AC-2 to "the `perm.denied` at the
  merge-authority step is cleared", naming the residual `gate.review_required` as the expected next gate.

Also remediated: **R6** (the forge-cache seeding seam exists — re-grounded the PRIMARY merge driver to the
public `forge_reviews_mut().upsert` API, `merge_gate.rs:403-434`, removing a phantom-helper fallback);
**R5** (T-GRPS-006 admin-gating re-grounded to the AUTHZ-006 guard at the authz layer); **R4/S2** (AC-4 made
net-new + adversarial — group-backed claim + id/claim mismatch); **R3** (AUTHZ_EMPTY_START guard re-implement
note); **S7** (fail-closed AUTHZ_EMPTY_START tightening); plus LOW hardening (R10 serial/temp_env env
discipline, S10 `.authorities()` honesty grep gate, S6 cross-loader-duplicate note). `R9` (a claimed ROADMAP
self-dependency) was verified a **false positive** and not actioned. `S4`/`S9` were self-rebutted by the
reviewers.

Deterministic re-validation after remediation: **2/2 tasks fakeability-CLEAN** (`validate_scenario.py`,
0 CRITICAL/HIGH on all 8 behavioral ACs); `proposed_by` tripwire 2/2; stable AC-1..4 / TC-1..8 IDs, gapless;
avg rubric ≈113/115; every cited crate surface (`enforce_merge_gate`, `classify_error`,
`load_governance_config`, `effective_authority`, `resolve_principal`, the `forge_reviews` seeding seam, the
`AUTHZ_EMPTY_START` guard) confirmed present in the live crate. 0 blocking findings open. The second
fresh-panel pass was bounded to deterministic re-validation + grounding spot-checks for cost (planning
artifact); no open CRITICAL/MEDIUM was waived.

## Task Detail Files

Generated by `/kb-sprint-tasks-plan` on 2026-06-19.

- GRPS-001-effective-set-union-group-ceiling.md
- GRPS-002-ref-pinned-membership-self-grant-inert.md

## Deferred Follow-ups — Red-Hat Review 2026-06-19 (ACCEPTED AS-IS)

A post-merge adversarial red-hat review (rust-reviewer w/ mutation testing + security-reviewer +
security-auditor; report `../../../reviews/red-hat-sprint-03-2026-06-19.md`) found **0 product/security
defects** — the target-ref-only / no-self-escalation property is mutation-proven enforced, GRPS-001 is a
legitimate behavior-neutral refactor, and the tests are load-bearing (not theatre). Three **MEDIUM**
test-teeth / TDD-contract-honesty items were surfaced.

**Decision (user, 2026-06-19): ACCEPT Sprint 03 as-is. The three items below are DEFERRED tracked
follow-ups — NOT an active remediation run.** Task files were authored (by rust-planner) and are retained
as documentation of the deferred fixes; they are intentionally **not** in the active `## Tasks` table.

- `FIX-GRPS-002-AC3-TEETH.md` — strengthen GRPS-002 AC-3's negative control so it catches a generic
  HEAD-peel regression (fixture currently leaves HEAD == target ref; a HEAD-peel mutation survived). Not
  a product bug — AC-1 + AC-4 already catch that regression for the membership surface.
- `FIX-GRPS-001-EMPTY-START-CONTROL.md` — exercise the dead `AUTHZ_EMPTY_START` fail-closed control in
  `grps_union.rs` (specified case currently unrun).
- `FIX-GRPS-RED-EVIDENCE-CONTRACT.md` — capture mutation-falsifiability evidence + amend GRPS-001/002
  `requires_red_evidence` to a recorded waiver. **⚠️ Needs human acknowledgment** (records a
  RED-evidence waiver grounded in mutation proof, not a fabricated RED).

Also tracked → **Sprint 04 GATES**: the duplicate `merge_gate.rs` config-loader (flagged by both the
Sprint 02 and Sprint 03 reviews; both copies currently read the target ref correctly).
