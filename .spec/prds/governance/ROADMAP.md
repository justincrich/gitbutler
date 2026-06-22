---
roadmap: 1
project: Functional-Permission Agent Governance for GitButler (POC)
generated: 2026-06-18
prd: .spec/prds/governance/README.md
sprint_count: 10
pr_sequencing: false
---

# Sprint Roadmap: Functional-Permission Agent Governance for GitButler (POC)

## Overview

**Sprints:** 10
**Total Tasks:** 35 (24 backend/CLI · 11 MGMT backend/IPC + 12 MGMT UI + 8 MGMT design across the two MGMT halves)
**Current Sprint:** 1 — AUTHZ primitive + commit-gate skeleton (Planned)

This roadmap turns GitButler into a commit/merge **policy-enforcement layer** for orchestrated agents: a new
`but-authz` crate + two gates (commit + merge) over GitButler's own git actions, principal grouping, and an
admin governance UI in `apps/desktop`. Sequencing honors the PRD mandate — the **proven-reference-flow canary
(T-LOOP-006)** is the walking skeleton that must go green _before_ the deep build, so Sprint 1 is a thin
vertical slice (ref-pin loader + fail-closed primitive + a single observable commit allow/deny) and Sprint 2
completes the loop. Every sprint's human-testing gate draws its proof from
[`11-e2e-testing-criteria.md`](./11-e2e-testing-criteria.md).

The product is **headless/CLI** for the AUTHZ · GRPS · GATES · LOOP groups (gates are verified by running a
`but` command and observing the structured denial `{code, message, remediation_hint}` + exit code) and a
**desktop UI** for the MGMT group (verified by using the Governance settings page).

> **Planning provenance.** Sprint content was authored by the dispatched specialist set — `rust-planner`
> (backend/CLI), `tauri-planner` (IPC seam), `sveltekit-planner` (UI), `frontend-designer` (design) — then
> hardened through one red-hat review cycle (`rust-reviewer` + `sveltekit-reviewer` + `security-auditor`),
> which surfaced 8 CRITICAL gaps (an ungated `set_review_auto_merge` path, un-gated `integrate`/`apply`
> ref-advancing paths, an unowned `gates.toml` writer, the `isAdmin` wiring gap, DryRun/commit-gate
> target-ref proofs, and the missing T-LOOP-013 traversability proof) — all remediated by the original
> writers. The orchestrator consolidated; it did not author sprint content.

## Sprint Sequence

| #   | Milestone | Sprint                                                                                                                | Gate                                                                                                                                                                                                                                                           | Tasks | Dependencies          | Status      |
| --- | --------- | --------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----- | --------------------- | ----------- |
| 1   | —         | [Sprint 01a: AUTHZ Primitive + Commit Gate](#sprint-01a-authz-primitive--commit-gate)                                 | Read-only commit denied; `contents:write` commit lands; protection read target-ref-only                                                                                                                                                                        | 5     | —                     | In Progress |
| 2   | —         | [Sprint 01b: Governed Loop Reference Flow](#sprint-01b-governed-loop-reference-flow)                                  | 3-principal loop: merge + auto-merge gated; channel traversable                                                                                                                                                                                                | 5     | Sprint 01a            | Done        |
| 3   | —         | [Sprint 02: AUTHZ Fail-Closed + Identity Confinement](#sprint-02-authz-fail-closed--identity-confinement)             | Unknown principal / no handle / bad config / borrowed identity denied with exact code                                                                                                                                                                          | 4     | Sprint 01b            | In Progress |
| 4   | —         | [Sprint 03: GRPS Groups + Ref-Pin](#sprint-03-grps-groups--ref-pin)                                                   | Group grant inherited; self-add / self-grant still denied (target-ref read)                                                                                                                                                                                    | 2     | Sprint 01b            | In Progress |
| 5   | —         | [Sprint 04: GATES Deepening](#sprint-04-gates-deepening)                                                              | Stale/self/single-group merge blocked; commit gate covers integrate/apply/worktree                                                                                                                                                                             | 3     | Sprint 01b, Sprint 03 | In Progress |
| 6   | —         | [Sprint 05: CLI `but perm` / `but group`](#sprint-05-cli-but-perm--but-group)                                         | Admin grants/groups/lists via CLI with ref-pin caveat; non-admin denied                                                                                                                                                                                        | 2     | Sprint 02, 03, 04     | In Progress |
| 7   | —         | [Sprint 06a: Governance UI — Scaffold + Principals + Groups](#sprint-06a-governance-ui--scaffold--principals--groups) | Admin edits principal & group permissions on the Governance page; pending until commit                                                                                                                                                                         | 16    | Sprint 02, Sprint 05  | In Progress |
| 8   | —         | [Sprint 06b: Governance UI — Branch Gates + Rules + Safety](#sprint-06b-governance-ui--branch-gates--rules--safety)   | Branch-gate edit pending; rules scoped; read-only + denial-no-flip safety                                                                                                                                                                                      | 11    | Sprint 06a, Sprint 04 | In Progress |
| 9   | —         | [Sprint 07: Local Agent PR — Governed-Review Parity](#sprint-07-local-agent-pr--governed-review-parity)               | Agent opens a LOCAL review (no remote PR while `keep_reviews_local`); reviewer assigned + file/line comments; `but review status` shows assignment/thread/`agent-authored` tag; approve → governed merge; forged/empty drive-tables ⇒ identical merge decision | 19    | Sprint 01b, 04, 05    | Planned     |
| 10  | —         | [Sprint 08: STEER — Capability-Aware Denials](#sprint-08-steer--capability-aware-denials)                             | Denied principal receives `class` + `held_permissions` + an `authorized_actions` menu + `do_not`; a listed lateral action succeeds; unknown-principal/`config.invalid` ⇒ `operator_required` + empty menu                                                      | 10    | Sprint 02, 04, 05     | Planned     |

_Milestone cells are `—` until the sprints are materialized as GitHub Milestones._
_Sprint 07 (LPR) is the human-directed priority slot; the former Sprint 07 (STEER) is renumbered to Sprint 08. Both are net-additive and depend only on shipped sprints (not on each other)._

### Dependency graph

```
01a → 01b → ┬→ 02 ─────────┬→ 05 → 06a → 06b
            ├→ 03 ──┬───────┤        ↑(also 04)
            └→ 04 ◄─┘       │
               04 ──────────┘
```

---

## Per-Sprint Details

### Sprint 01a: AUTHZ Primitive + Commit Gate

**Sequence:** 1
**Timeline:** Phase 1 — Walking skeleton
**Status:** In Progress
**Proposed by:** rust-planner
**Milestone:** — (`sprint-01a`)

#### Human Testing Gate

**Gate:** Running a commit as a read-only principal is denied `perm.denied` while a `contents:write` principal's commit lands, and protection is read only from the target-ref blob so a working-tree `gates.toml` edit cannot unprotect the branch.

**Test Steps:**

1. Seed committed `.gitbutler/permissions.toml` (`ro` contents:read; `dev` contents:write) + `gates.toml` (`main` protected).
2. Run commit on a feature branch as `dev` → exit 0, ref advances.
3. Run commit on a feature branch as `ro` → denied, exit 1, `perm.denied` names contents:write.
4. Run direct commit to protected `main` as `dev` → denied, exit 1, `branch.protected`.
5. Run commit with `BUT_AGENT_HANDLE` unset → rejected, exit 1, no anonymous action.
6. Edit working-tree `gates.toml` to unprotect `main`, commit directly to `main` as `dev` → still `branch.protected`.
7. Run commit as a principal absent from committed `permissions.toml` → denied, `perm.denied`.
8. Commit a malformed `gates.toml` to the target ref, run a commit → denied, `config.invalid` (fail-closed).

#### Tasks

| ID        | Title                                                                                    | Agent            | Estimate |
| --------- | ---------------------------------------------------------------------------------------- | ---------------- | -------- |
| AUTHZ-001 | Create `but-authz` crate: `Authority`, `AuthoritySet`, `Principal`, `Group`, `Denial`    | rust-implementer | 180 min  |
| AUTHZ-002 | Ref-pinned governance config loader (`gix`, target-ref blob read)                        | rust-implementer | 210 min  |
| AUTHZ-003 | `authorize()` + `BUT_AGENT_HANDLE` resolution + fail-closed default-deny                 | rust-implementer | 180 min  |
| GATES-001 | Commit gate at `commit_engine::create_commit` (target-ref-only, DryRun-enforced)         | rust-implementer | 240 min  |
| AUTHZ-007 | Invariant build-gates — no role name, no human-vs-AI predicate, no `Permission` overload | rust-reviewer    | 90 min   |

#### Dependencies

- Blocks: Sprint 01b
- Dependent on: None

#### PRD Coverage

- UC-AUTHZ-01, UC-AUTHZ-02, UC-AUTHZ-04, UC-GATES-01
- Criteria: T-AUTHZ-001/003/004/009/010/012/016/024/027/028/029, T-GATES-001..007, T-LOOP-005/011

#### Capability Coverage

- **CAP-AUTHZ-01** — producer: `authorize()` (AUTHZ-003); the commit gate (GATES-001) runs **even under DryRun**.
- **CAP-CONFIG-01** — producer: ref-pinned loader (AUTHZ-002); the commit gate reads branch-protection **target-ref-only** (a working-tree `gates.toml` edit cannot weaken it).

#### Next Sprint Tasks

Expanded by `/kb-sprint-tasks-plan` on 2026-06-18 (avg 111/115 rubric · fakeability floor 0/0/0/0 · 1 full red-hat review cycle, 20 findings resolved). Detail files in [`tasks/sprint-01a-authz-primitive-commit-gate/`](./tasks/sprint-01a-authz-primitive-commit-gate/):

- `AUTHZ-001-create-but-authz-crate.md`
- `AUTHZ-002-ref-pinned-config-loader.md`
- `AUTHZ-003-authorize-handle-resolution.md`
- `GATES-001-commit-gate.md`
- `AUTHZ-007-invariant-build-gates.md`

> **Material re-plan in expansion (red-hat):** GATES-001's commit gate was re-sited from `commit_engine::create_commit` to the ref-aware **`but-api` `_with_authz` seam + CLI commit path** (authorize **before** the `RepoExclusive` guard, per `04-api-design.md`) — `create_commit` cannot name the target branch. `T-GATES-016/017` (mechanism-agnostic + worktree) are explicitly deferred to Sprint 04.

---

### Sprint 01b: Governed Loop Reference Flow

**Sequence:** 2
**Timeline:** Phase 1 — Walking skeleton (the T-LOOP-006 canary)
**Status:** Done — closed out 2026-06-19 (T-LOOP-006 canary green; 5/5 tasks merged + re-verified)
**Proposed by:** rust-planner
**Milestone:** — (`sprint-01b`)

#### Human Testing Gate

**Gate:** Running the reference flow with three principals, the implementer's merge AND auto-merge are denied, the maintainer's merge succeeds only after a distinct reviewer approval at head, and a denied implementer that follows its `remediation_hint` lands through a reviewed merge.

**Test Steps:**

1. Run `but` merge as implementer (`reviews:write`, no `merge`) → denied, exit 1, `perm.denied` names merge.
2. Enable auto-merge (`but review --auto-merge`) as that implementer → denied, exit 1, `perm.denied`.
3. Run commit as reviewer → denied; submit review as reviewer → exit 0, recorded at head.
4. Run merge as maintainer with zero distinct approvals → denied, exit 1, `gate.review_required`.
5. Run merge as maintainer after a distinct reviewer approval at head → exit 0, merge proceeds.
6. Advance head, rerun merge → denied, `gate.review_required` with `approval_stale_at_head`.
7. Run a DryRun merge as the implementer → still denied `perm.denied`, nothing persisted.
8. Follow the denied implementer's `remediation_hint` (feature branch → review → merge) → lands successfully.

#### Tasks

| ID        | Title                                                                                | Agent            | Estimate |
| --------- | ------------------------------------------------------------------------------------ | ---------------- | -------- |
| GATES-002 | Local review record: `but-db` `local_review_verdicts` table (head-pinned)            | rust-implementer | 150 min  |
| GATES-003 | Merge gate covering **both** `merge_review` AND `set_review_auto_merge`              | rust-implementer | 270 min  |
| GATES-004 | Submit-review / open-PR / comment authz guards on the forge boundary                 | rust-implementer | 150 min  |
| GATES-005 | Stale-approval-@head dismissal + self-approval exclusion                             | rust-implementer | 150 min  |
| LOOP-001  | Reference-flow test (T-LOOP-006) + traversable proof (T-LOOP-013) + DryRun-no-bypass | rust-implementer | 240 min  |

#### Dependencies

- Blocks: Sprint 02, Sprint 03, Sprint 04
- Dependent on: Sprint 01a

#### PRD Coverage

- UC-LOOP-01, UC-LOOP-02, UC-GATES-02 (review record)
- Criteria: T-LOOP-006/001/002/003/004/007/010/**013**, T-GATES-008/009/010/011/014/015

#### Capability Coverage

- **CAP-AUTHZ-01** — merge/auto-merge gate (GATES-003), forge guards (GATES-004); DryRun-no-bypass proven (LOOP-001).
- **CAP-CONFIG-01** — merge gate reads requirement at the target ref (GATES-003).

#### Next Sprint Tasks

Expanded by `/kb-sprint-tasks-plan` on 2026-06-18 (5/5 tasks fakeability-CLEAN · `proposed_by` tripwire 5/5 · 1 full red-hat cycle — fresh `rust-reviewer` + `security-auditor`; 1 CRITICAL resolved by the user's gate-boundary re-scope + upstream advisory, 5 MEDIUM remediated, all 9 prior CRITICALs confirmed closed). Detail files in [`tasks/sprint-01b-governed-loop-reference-flow/`](./tasks/sprint-01b-governed-loop-reference-flow/):

- `GATES-002-local-review-record.md`
- `GATES-003-merge-gate.md`
- `GATES-004-forge-authz-guards.md`
- `GATES-005-stale-self-approval.md`
- `LOOP-001-reference-flow-test.md`

> **Gate-boundary re-scope (red-hat, user decision).** `merge_review`/`publish_review` are forge-bound (error on a bare local repo) and there is no `but pr merge` CLI verb, so the **positive** governed-merge/PR-open paths prove the gate **DECISION** (permit/deny) on the real seam + that execution reaches the forge call; the forge-network **completion** is proven structurally. **Upstream advisory:** reconcile T-LOOP-006/004/013 "merge succeeds / change lands" wording (and the stale step-1 "implementer (`reviews:write`)" role token) with this forge-locality via `/kb-sprint-plan --delta-replan`. T-LOOP-011 (human-vs-AI grep) deferred to Sprint 04.

---

### Sprint 02: AUTHZ Fail-Closed + Identity Confinement

**Sequence:** 3
**Timeline:** Phase 2 — Hardening
**Status:** In Progress
**Proposed by:** rust-planner
**Milestone:** — (`sprint-02`)

#### Human Testing Gate

**Gate:** An action by an unknown principal, with no handle, against malformed config, naming an undefined required group, or borrowing another handle is denied with the exact structured code instead of running.

**Test Steps:**

1. Run a merge as a principal absent from `permissions.toml` → denied, exit 1, `perm.denied`.
2. Run a merge with `BUT_AGENT_HANDLE` unset → rejected, exit 1, no anonymous action.
3. Commit a malformed `gates.toml` to the target ref, run merge → denied, exit 1, `config.invalid`.
4. Run merge whose `gates.toml` names an undefined group → denied, not vacuously satisfied.
5. Run a governed action as a dispatched reviewer → denied, exit 1, `perm.denied`; attempt
   `--as <other>` → rejected as an unsupported flag, exits non-zero, and no action runs as the
   borrowed handle.
6. Inject an agent-supplied authority claim → ignored; authority comes from committed config only.
7. Re-run the reference-flow canary (T-LOOP-006) → still green after hardening.

#### Tasks

| ID        | Title                                                                                                       | Agent            | Estimate |
| --------- | ----------------------------------------------------------------------------------------------------------- | ---------------- | -------- |
| AUTHZ-004 | Merge/forge-gate fail-closed + `config.invalid` vs `perm.denied` determinism + undefined-group hard-deny    | rust-implementer | 150 min  |
| AUTHZ-005 | Identity confinement — no honored in-band identity override + handle-only resolution (honest accepted-leak) | rust-implementer | 150 min  |
| AUTHZ-006 | `administration:write` authority primitive on the config-mutating path                                      | rust-implementer | 120 min  |
| AUTHZ-008 | Re-assert the honesty invariant grep-gates after AUTHZ hardening                                            | rust-reviewer    | 45 min   |

#### Dependencies

- Blocks: Sprint 05, Sprint 06a
- Dependent on: Sprint 01b

#### PRD Coverage

- UC-AUTHZ-03 (confinement primitive), UC-AUTHZ-04
- Criteria: T-AUTHZ-018/019/020/021/023/026/027/028/029/030/031/016/022, T-LOOP-005/011
- _(The self-grant-inert ref-pin (T-AUTHZ-032) and `perm list` scoping (T-AUTHZ-025) were relocated to their natural gate homes — Sprint 03 and Sprint 05 — so every step here is covered by this sprint's fail-closed/confinement gate.)_

#### Capability Coverage

- **CAP-AUTHZ-01** — fail-closed enforcement (AUTHZ-004), confinement (AUTHZ-005), admin-write primitive (AUTHZ-006).
- **CAP-CONFIG-01** — `config.invalid` on malformed target-ref config; admin-write checked at the target ref.

#### Next Sprint Tasks

Expanded by `/kb-sprint-tasks-plan` on 2026-06-19 (4/4 tasks fakeability-CLEAN · `proposed_by` tripwire 4/4 ·
avg rubric ≈110/115 · 1 full red-hat cycle — fresh `rust-reviewer` + `security-auditor`; 4 CRITICAL + 7 MEDIUM

- 3 LOW spec-correctness findings all remediated and the fixes independently verified in the AC bodies, then
  confirmed by a fresh pass). Detail files in [`tasks/sprint-02-authz-fail-closed-identity-confinement/`](./tasks/sprint-02-authz-fail-closed-identity-confinement/):

* `AUTHZ-004-merge-gate-fail-closed.md`
* `AUTHZ-005-identity-confinement.md`
* `AUTHZ-006-administration-write-guard.md`
* `AUTHZ-008-honesty-invariant-build-gates.md`

> **Red-hat re-grounding (spec correctness).** The first draft was fakeability-clean and well-structured but
> rested on two non-existent surfaces the structural gates can't see: the undefined-`require_approval_from_group`
> hard-deny (AUTHZ-004) stood on a gate schema absent from `but-authz` `config.rs` (and forbidden there by
> GATES-003's `writeProhibited`), and the admin-write guard (AUTHZ-006/005) leaned on a config-mutate seam that
> won't exist until the Sprint-05 `but perm`/`but group` verbs. Both were re-grounded honestly — the
> undefined-group hard-deny to the but-api merge-gate layer with a `BLOCKED-UNTIL` note, the admin-write proof
> to the reusable guard **function** (real config-load + real `authorize`) with Sprint-05 named as the
> persisted-write consumer. Intra-sprint order is a strict chain: **AUTHZ-004 + AUTHZ-006 → AUTHZ-005 → AUTHZ-008**.

---

### Sprint 03: GRPS Groups + Ref-Pin

**Sequence:** 4
**Timeline:** Phase 2 — Hardening
**Status:** In Progress
**Proposed by:** rust-planner
**Milestone:** — (`sprint-03`)

#### Human Testing Gate

**Gate:** Gate passes when BOTH: [GRPS-01] a member with no direct grant is authorized via its group's permission (union) and denied an action no source grants; AND [GRPS-02] a feature head that adds its own author to a merge-holding group — or self-grants `administration:write` — is still denied, because membership and grants are read only at the target ref.

**Test Steps:**

1. Seed a `code-reviewers` group with `reviews:write` and a member holding no direct review grant.
2. Run a review as that member → exit 0, authorized via the group (union).
3. Run a merge as that member → denied, exit 1, `perm.denied` (no source grants merge).
4. Create a feature head that adds its author to a `merge`-holding `maintainers` group.
5. Run merge from that feature head → denied, the target-ref membership governs.
6. Self-grant `administration:write` on a feature head, run the same config change → denied (inert until target-ref commit).
7. Commit the membership to the target ref, advance it, rerun the merge → now authorized.

#### Tasks

| ID       | Title                                                                    | Agent            | Estimate |
| -------- | ------------------------------------------------------------------------ | ---------------- | -------- |
| GRPS-001 | Effective-set union via group membership + group permission ceiling      | rust-implementer | 150 min  |
| GRPS-002 | Ref-pinned governed membership + self-grant-inert (target-ref-only read) | rust-implementer | 210 min  |

#### Dependencies

- Blocks: Sprint 04, Sprint 05
- Dependent on: Sprint 01b

#### PRD Coverage

- UC-GRPS-01, UC-GRPS-02, UC-AUTHZ-03 (self-grant-inert, relocated here)
- Criteria: T-GRPS-001..014, T-AUTHZ-032

#### Capability Coverage

- **CAP-AUTHZ-01** — union resolution (GRPS-001).
- **CAP-CONFIG-01** — membership + grants read target-ref-only; no self-escalation (GRPS-002).

#### Next Sprint Tasks

Expanded by `/kb-sprint-tasks-plan` on 2026-06-19 (2/2 tasks fakeability-CLEAN · `proposed_by` tripwire 2/2 ·
avg rubric ≈113/115 · 1 full red-hat cycle — fresh `rust-reviewer` + `security-auditor`; 2 spec-correctness
CRITICALs (the fictional union-"divergence" thesis and the impossible AC-2 "authorizes Ok") + the actionable
MEDIUM/LOW set all remediated by the retained writer and confirmed against the live crate). Detail files in
[`tasks/sprint-03-grps-groups-ref-pin/`](./tasks/sprint-03-grps-groups-ref-pin/):

- `GRPS-001-effective-set-union-group-ceiling.md`
- `GRPS-002-ref-pinned-membership-self-grant-inert.md`

> **Material re-grounding in expansion (red-hat):** GRPS-001 was re-framed from "fix a redundant double-union
> _divergence_" (a fiction — `effective_authority` is provably equal to `principal_authorities` by
> construction) to an honest **behavior-neutral simplification** that removes the dead authorize-time re-union
> and **pins the equality**. GRPS-002's positive AC-2 was re-scoped from "the merge authorizes (Ok)" (impossible
> under the fixture's unapproved review gate) to "the `perm.denied` at the `Authority::Merge` step is cleared",
> with the residual `gate.review_required` named as the expected next gate; its PRIMARY driver seeds the
> `ForgeReview` inline via the real public `forge_reviews_mut().upsert` seam. `but group` CLI verbs (T-GRPS-001/
> 002/006/010-CLI) remain re-grounded to Sprint 05 (CLI-002); T-GRPS-006/011 admin-gating proven at the authz
> layer via the AUTHZ-006 guard.

---

### Sprint 04: GATES Deepening

**Sequence:** 5
**Timeline:** Phase 2 — Hardening
**Status:** In Progress
**Proposed by:** rust-planner
**Milestone:** — (`sprint-04`)

#### Human Testing Gate

**Gate:** Gate passes when BOTH: [MERGE-STRICTNESS] a merge with a stale, self, or single-group-only approval is blocked and lands only with a distinct approval from each required group at the current head; AND [COMMIT-COVERAGE] a protected-branch commit, integrate, and apply are each rejected `branch.protected` through the same commit gate, and a working-tree `gates.toml` edit cannot weaken either gate.

**Test Steps:**

1. Approve a PR at head H1, advance to H2, run merge → blocked, `gate.review_required` with `approval_stale_at_head`.
2. Have the author approve their own change (distinct required), run merge → blocked, requirement unmet.
3. Supply only a `code-reviewers` approval, run merge → blocked (maintainers required); only `maintainers` → blocked.
4. Supply a distinct approval from each required group at head, run merge → exit 0, proceeds.
5. Commit on `but worktree new` to a feature branch, then to protected `main` → feature accepted, `main` rejected `branch.protected`.
6. Run `integrate_branch_with_steps` / `branch apply` advancing protected `main` as contents:write → rejected `branch.protected`.
7. Edit `gates.toml` on the feature head to drop the merge requirement, run merge → still judged by the target-ref requirement.

#### Tasks

| ID        | Title                                                                                                                                               | Agent            | Estimate |
| --------- | --------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- | -------- |
| GATES-006 | Per-required-group approval evaluation (two-group AI + human model)                                                                                 | rust-implementer | 150 min  |
| GATES-007 | Mechanism-agnostic commit gate — **actually** gate `branch::apply`, `integrate_branch_with_steps`, worktree-integrate                               | rust-implementer | 270 min  |
| GATES-008 | Standalone target-ref-only read proof for the merge gate — feature-head requirement-drop ignored (deepening; AUTHZ-004 owns merge-path fail-closed) | rust-implementer | 120 min  |

#### Dependencies

- Blocks: Sprint 05, Sprint 06b
- Dependent on: Sprint 01b, Sprint 03

#### PRD Coverage

- UC-GATES-01 (mechanism-agnostic coverage), UC-GATES-02, UC-LOOP-02
- Criteria: T-GATES-012/013/016/017/019, T-LOOP-008/009/010/011/012 _(T-GATES-018 owned by AUTHZ-004/Sprint 02 — re-proven by GATES-008's target-ref-only deepening; T-LOOP-011 lands as GATES-006 AC-3; T-GATES-016/017 re-grounded — apply/integrate prove contents:write, worktree_integrate proves branch.protected)_

> **Upstream advisory (red-hat).** Sprint-04 gate step 6 ("`integrate_branch_with_steps` / `branch apply` advancing protected `main` → `branch.protected`") assumes a flow the live code contradicts (`branch::apply` bails on the target; `integrate_branch_with_steps` writes a feature branch; only `worktree_integrate` advances `target`). Reconcile via `/kb-sprint-plan --delta-replan` to: contents:write enforced on apply + integrate (read-only denied) + `worktree_integrate` advancing a protected target → `branch.protected`.

#### Capability Coverage

- **CAP-AUTHZ-01** — per-group approval evaluation (GATES-006).
- **CAP-CONFIG-01** — commit gate covers every ref-advancing entry point (GATES-007); both gates read target-ref-only (GATES-007/008).

#### Next Sprint Tasks

Expanded by `/kb-sprint-tasks-plan` on 2026-06-19 (3/3 tasks fakeability-CLEAN · `proposed_by` tripwire 3/3 · avg rubric 115/115 · 1 full red-hat goal loop, 3 cycles — fresh `rust-reviewer` + `security-auditor`; 5 CRITICAL + 8 MEDIUM + LOW resolved, incl. a fabricated `governance_present` grounding inversion, the incoherent apply/integrate-advances-main framing re-grounded against live code, and three remediation-introduced fixes (config_only vacuous-gate, lock-ordering placement, no-target regression) all re-verified). Detail files in [`tasks/sprint-04-gates-deepening/`](./tasks/sprint-04-gates-deepening/):

- `GATES-006-per-required-group-approval.md`
- `GATES-007-mechanism-agnostic-commit-gate.md`
- `GATES-008-merge-gate-failclosed-target-ref-only.md`

> **Material re-grounding in expansion (red-hat):** GATES-007's mechanism-agnostic coverage was re-grounded against live code — `branch::apply` bails on the target and `integrate_branch_with_steps` writes a feature branch (neither advances a protected trunk), so the gate proves **contents:write/perm.denied on apply + integrate** (via `config_only(workspace target ref)`, fired at the public seam before the worktree guard, no-target = ungoverned = permit) and **`worktree_integrate` advancing a protected `target` → `branch.protected`** (via `direct_ref(target)`) — not "apply/integrate advancing main". GATES-008 was re-grounded to the **standalone target-ref-only read proof** (AUTHZ-004 retains merge-path fail-closed ownership; T-GATES-018). **Upstream advisory:** reconcile the Human-Testing-Gate **step 6** wording (and T-GATES-018's Sprint-04 listing) via `/kb-sprint-plan --delta-replan`.

---

### Sprint 05: CLI `but perm` / `but group`

**Sequence:** 6
**Timeline:** Phase 3 — CLI governance management
**Status:** In Progress
**Proposed by:** rust-planner
**Milestone:** — (`sprint-05`)

#### Human Testing Gate

**Gate:** An admin runs `but perm grant` and `but group add-member`, sees the takes-effect-once-committed caveat, `but perm list` shows the committed effective set plus the new grant as pending, and a non-admin's grant or cross-principal list is denied `perm.denied`.

**Test Steps:**

1. Run `but perm grant --principal rust-implementer reviews:write` as an admin → exit 0, prints the ref-pin caveat.
2. Run `but perm list --principal rust-implementer` → shows the committed effective set (unchanged) and the new grant as PENDING (not yet in effect).
3. Run `but group create code-reviewers --permissions reviews:write` → exit 0, `[[group]]` written.
4. Run `but group add-member code-reviewers --principal rust-reviewer` → exit 0, ref-pin caveat printed.
5. Run `but perm grant ...` as a non-admin principal → denied, exit 1, `perm.denied`.
6. Run `but perm list --principal <other>` as a non-admin (not self, no admin:read) → denied, `perm.denied`, no recon.
7. Run `but perm revoke --principal rust-implementer reviews:write` as admin → exit 0.

#### Tasks

| ID      | Title                                                                                    | Agent            | Estimate |
| ------- | ---------------------------------------------------------------------------------------- | ---------------- | -------- |
| CLI-001 | `but perm {list,grant,revoke}` + admin-write gating + ref-pin caveat + perm-list scoping | rust-implementer | 240 min  |
| CLI-002 | `but group {create,grant,add-member,remove-member,list}` + admin-write gating            | rust-implementer | 210 min  |

#### Dependencies

- Blocks: Sprint 06a
- Dependent on: Sprint 02, Sprint 03, Sprint 04

#### PRD Coverage

- UC-AUTHZ-01, UC-AUTHZ-03 (`perm list` scoping, relocated here), UC-GRPS-01, UC-GRPS-02
- Criteria: T-AUTHZ-007/021/025, T-GRPS-001/002/006/010/011
- _Authors the `but-api` perm/group governance functions reused as Tauri commands in Sprint 06a._

#### Capability Coverage

- **CAP-AUTHZ-01** + **CAP-CONFIG-01** — CLI write path authorizes `administration:write` and writes inert-until-committed config (CLI-001/002).

#### Next Sprint Tasks

Expanded by `/kb-sprint-tasks-plan` on 2026-06-19 (2/2 tasks fakeability-CLEAN · `proposed_by` tripwire 2/2 ·
avg rubric 115/115 · 1 full red-hat goal loop, 2 cycles — fresh `rust-reviewer` + `security-auditor`; 3 CRITICAL

- 5 MEDIUM + 3 LOW spec-correctness findings all remediated by the retained writer and confirmed genuinely closed
  by a fresh cycle-2 pass). Detail files in [`tasks/sprint-05-cli-perm-group/`](./tasks/sprint-05-cli-perm-group/):

* `CLI-001-perm-cli-verbs.md`
* `CLI-002-group-cli-verbs.md`

> **Net-new persisted writer + honesty-grep extension (red-hat).** This is the first sprint that **persists**
> governed config — `but-authz` `config.rs` is loader-only, so CLI-001 authors the working-tree TOML
> read-modify-write (raw serde wire structs, `#[derive(Serialize)]` added additively — NOT a lossy `GovConfig`
> round-trip), a `permissions_path()` accessor, and the `but-api` `perm_*`/`group_*` functions sited beside
> `config_mutate.rs` for Sprint-06a Tauri reuse. Admin gating **composes** the Sprint-02 AUTHZ-006
> `enforce_administration_write_gate` (never re-implements it); the writes are inert-until-committed (working tree
> only). The red-hat loop forced a **CRITICAL** fix: CLI-001 extends the AUTHZ-007/008 honesty grep
> (`invariant_build_gates.rs` `ENFORCEMENT_PATHS` + a positive-Authority assertion) to cover the net-new
> `governance.rs`, and added the missing positive `perm revoke` proof (gate step 7) and the `T-AUTHZ-007`
> seeded-day-one-effectiveness AC. `but group delete` stays a Sprint-06a/UC-MGMT-03 consumer (explicitly not
> shipped). **Intra-sprint order is a strict chain: CLI-001 → CLI-002** (CLI-002 extends `governance.rs` + the
> shared `crates/but/src/args` wiring; rebase, do not run in parallel worktrees).

---

### Sprint 06a: Governance UI — Scaffold + Principals + Groups

**Sequence:** 7
**Timeline:** Phase 4 — Governance management UI
**Status:** In Progress
**Proposed by:** sveltekit-planner + tauri-planner + frontend-designer + rust-planner (MGMT backend); split authored by sveltekit-reviewer (red-hat)
**Milestone:** — (`sprint-06a`)

#### Human Testing Gate

**Gate:** An admin opens Project Settings, sees the Permissions & Governance sidebar item, navigates to it, observes four tabs, edits a principal's own-grant permission on the Principals tab, expands a group and grants a permission on the Groups tab, and sees both changes remain pending with a commit banner until clicking Commit changes.

**Test Steps:**

1. Open the GitButler desktop app and open Project Settings via the existing shortcut.
2. Observe the Permissions & Governance sidebar item shows for an admin; sign in as non-admin, confirm absent.
3. Click Permissions & Governance and observe four tabs: Principals, Groups, Branch Gates, Rules.
4. On Principals, click a principal row, toggle an own-grant permission on; observe a pending circle and commit banner.
5. On Groups, expand a group and grant a permission; observe the pending circle and banner persist across tabs.
6. Click Commit changes and observe all pending indicators clear and the effective set update.

#### Tasks

| ID              | Title                                                                                                         | Agent                 | Estimate |
| --------------- | ------------------------------------------------------------------------------------------------------------- | --------------------- | -------- |
| MGMT-IPC-001    | `#[but_api]` governance fns (perm/group/status) + `json::Error` transport                                     | rust-implementer      | 90 min   |
| MGMT-IPC-002    | `json.rs` `Error` serializes the 3rd field `remediation_hint` (closes a real drop bug)                        | rust-implementer      | 75 min   |
| MGMT-IPC-003    | Register governance commands in `generate_handler!` + capability + v1 human-fleet-owner identity (T-MGMT-042) | tauri-implementer     | 60 min   |
| MGMT-IPC-004    | Regenerate `packages/but-sdk` (perm/group/status) — SDK build-gate before UI wiring                           | tauri-implementer     | 45 min   |
| MGMT-IPC-005    | Pending-until-committed read IPC contract (working-tree vs target-ref)                                        | tauri-implementer     | 60 min   |
| MGMT-UI-001     | Register the governance page + extend `ProjectSettingsPageId` (in `uiState.svelte.ts`)                        | sveltekit-implementer | 30 min   |
| MGMT-UI-002     | `ProjectSettingsModalContent` governance branch + **wire `isAdmin`** to `SettingsModalLayout`                 | sveltekit-implementer | 45 min   |
| MGMT-UI-003     | `GovernanceSettings.svelte` + client-only pending-state store (no `+page.server.ts`)                          | sveltekit-implementer | 90 min   |
| MGMT-UI-005     | `GovernancePendingBanner` (warning InfoMessage + Commit action)                                               | sveltekit-implementer | 30 min   |
| MGMT-UI-006     | `PrincipalsList` (rows + inline editor; inherited rows read-only)                                             | sveltekit-implementer | 90 min   |
| MGMT-UI-007     | `PrincipalEditor` (SegmentControl presets + Toggle table + group TagInput)                                    | sveltekit-implementer | 90 min   |
| MGMT-UI-008     | `GroupsList` (ExpandableSection per group; create/grant/add-member)                                           | sveltekit-implementer | 75 min   |
| DESIGN-MGMT-001 | Wireframe-fidelity + visual-state annotations for all four tabs                                               | frontend-designer     | 60 min   |
| DESIGN-MGMT-002 | Pending-state visual contract (○ badge, count banner, commit affordance)                                      | frontend-designer     | 45 min   |
| DESIGN-MGMT-003 | Read-only state (disabled-control treatment + `administration:write` info banner)                             | frontend-designer     | 30 min   |
| DESIGN-MGMT-005 | Inherited-vs-own permission row distinction in `PrincipalEditor`                                              | frontend-designer     | 40 min   |

#### Dependencies

- Blocks: Sprint 06b
- Dependent on: Sprint 02 (admin-write), Sprint 05 (perm/group `but-api` fns). **MGMT-IPC-004 (SDK regen) is a hard predecessor of every `but-sdk`-importing UI task.**

#### PRD Coverage

- UC-MGMT-01, UC-MGMT-02, UC-MGMT-03, UC-MGMT-06 (pending-until-committed half)
- Criteria: T-MGMT-001..016, T-MGMT-027/028/033/034/035/036

#### Capability Coverage

- **CAP-AUTHZ-01** — every governed write goes through `but-api` → `but-authz` `authorize()`; the UI never provides a bypass (server-side enforcement; renderer `adminOnly` is UX only).

#### Next Sprint Tasks

Expanded by `/kb-sprint-tasks-plan` on 2026-06-19 (16/16 tasks fakeability-CLEAN · `proposed_by` tripwire 16/16 ·
avg rubric ≈112/115 · stable gapless AC-N/TC-N · 1 full red-hat cycle — fresh `rust-reviewer` + `tauri-reviewer`

- `sveltekit-reviewer` + `security-auditor`, all BLOCK; 7 CRITICAL + 14 MEDIUM + 12 LOW remediated by the retained
  domain writers and confirmed by cycle-2 deterministic re-validation). Detail files in
  [`tasks/sprint-06a-governance-ui-scaffold-principals-groups/`](./tasks/sprint-06a-governance-ui-scaffold-principals-groups/):

* `MGMT-IPC-001-but-api-governance-fns.md`
* `MGMT-IPC-002-json-error-remediation-hint.md`
* `MGMT-IPC-003-register-governance-commands.md`
* `MGMT-IPC-004-sdk-regen.md`
* `MGMT-IPC-005-pending-read-ipc-contract.md`
* `MGMT-UI-001-register-page-ct-harness.md`
* `MGMT-UI-002-settings-branch-isadmin.md`
* `MGMT-UI-003-governance-settings-pending-store.md`
* `MGMT-UI-005-governance-pending-banner.md`
* `MGMT-UI-006-principals-list.md`
* `MGMT-UI-007-principal-editor.md`
* `MGMT-UI-008-groups-list.md`
* `DESIGN-MGMT-001-four-tab-annotations.md`
* `DESIGN-MGMT-002-pending-state-contract.md`
* `DESIGN-MGMT-003-read-only-state.md`
* `DESIGN-MGMT-005-inherited-vs-own-rows.md`

> **Material re-grounding in expansion (red-hat):** MGMT-IPC-001's central premise was re-scoped — `#[but_api]`
> requires a `Context` param but the Sprint-05 `governance.rs` fns take `&gix::Repository`, so the task now owns a
> NEW thin Context-param **wrapper layer** (`*_cmd(ctx,…)`) delegating to the un-forked Sprint-05 `&repo` fns. The
> v1 **fleet-owner** identity (T-MGMT-042) is pinned as the R12 **unconditional-superuser** path (resolved from
> `UserService`, NOT a `permissions.toml` lookup; the "UI is never a bypass" invariant binds _agents_, not the human
> owner), wired + re-tested in MGMT-IPC-003 while MGMT-IPC-001 proves the gate under the `BUT_AGENT_HANDLE` identity.
> **MGMT-IPC-001 is BLOCKED-UNTIL Sprint-05 `governance.rs` merges** (and is added to the AUTHZ-007/008 honesty grep).
> **Upstream advisories** (reconcile via `/kb-sprint-plan --delta-replan`): `04-api-design.md:80` `allow-*` capability
> language is superseded by the live `core:default` convention; the `04-api-design.md` command table is missing the
> `governance_pending` row; `08-uc-mgmt.md:57` cites the wrong file for `ProjectSettingsPageId` (`uiState.svelte.ts`
> is correct). _(Tasks-table count corrected 17 → 16 — the materialized sprint has 16 tasks.)_

---

### Sprint 06b: Governance UI — Branch Gates + Rules + Safety

**Sequence:** 8
**Timeline:** Phase 4 — Governance management UI
**Status:** In Progress
**Proposed by:** sveltekit-planner + frontend-designer + rust-planner (MGMT backend); split authored by sveltekit-reviewer (red-hat)
**Milestone:** — (`sprint-06b`)

#### Human Testing Gate

**Gate:** An admin edits a branch gate on the Branch Gates tab (pending indicator appears), selects a principal on the Rules tab and confirms only that principal's rules are shown, then opens the page as a user lacking `administration:write` and observes all controls disabled with a read-only InfoMessage, and attempts a self-escalation and sees the denial InfoMessage without the toggle flipping.

**Test Steps:**

1. On Branch Gates, toggle Protected branch for a pattern; observe the pending indicator appears.
2. On Rules, select principal A and confirm only A's rules show; select B and confirm A's are absent.
3. Sign in as a user lacking `administration:write`; open the page; observe all controls disabled with a read-only banner.
4. As an admin, attempt to grant yourself `administration:write`; observe the denial banner and the toggle does not flip.
5. Navigate the four tabs by keyboard (Tab then Arrow keys); observe focus moves and tabs activate.
6. Trigger an IPC failure; observe a danger banner with a Retry button and the page stays read-only.

#### Tasks

| ID              | Title                                                                                                                          | Agent                 | Estimate |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------ | --------------------- | -------- |
| MGMT-BE-004     | `branch_gates_read`/`branch_gates_update` gate-config `but-api` producer (the gates.toml writer) + its Tauri command/SDK delta | rust-implementer      | 180 min  |
| MGMT-BE-003     | `principalId`-scoped rules query (backend for the Rules tab)                                                                   | rust-implementer      | 120 min  |
| MGMT-UI-004     | Wrap `GovernanceSettings` in the existing `shared/ErrorBoundary` (no new boundary component)                                   | sveltekit-implementer | 30 min   |
| MGMT-UI-009     | `BranchGatesList` (ExpandableSection per branch; required-group selector = defined groups)                                     | sveltekit-implementer | 75 min   |
| MGMT-UI-010     | Extend `RulesList` with optional `principalId` prop (backward compatible)                                                      | sveltekit-implementer | 45 min   |
| MGMT-UI-011     | Accessibility (aria + keyboard nav) + IPC-failure danger banner + Retry                                                        | sveltekit-implementer | 60 min   |
| MGMT-UI-012     | Build-gate tests: no direct config write, no `+page.server.ts`, SDK type-check, human-principal                                | sveltekit-implementer | 45 min   |
| DESIGN-MGMT-004 | Structured-denial banner + self-escalation no-flip contract                                                                    | frontend-designer     | 30 min   |
| DESIGN-MGMT-006 | Empty states for all four tabs                                                                                                 | frontend-designer     | 25 min   |
| DESIGN-MGMT-007 | Four-tab IA + aria + keyboard-nav contract                                                                                     | frontend-designer     | 35 min   |
| DESIGN-MGMT-008 | Error-boundary fallback + IPC-failure/retry pattern                                                                            | frontend-designer     | 30 min   |

#### Dependencies

- Blocks: None
- Dependent on: Sprint 06a (page scaffold + pending store + IPC base), Sprint 04 (gate engine for branch gates)

#### PRD Coverage

- UC-MGMT-04, UC-MGMT-05, UC-MGMT-06 (read-only + denial-no-flip), UC-MGMT-07 (error boundary + a11y + IPC retry)
- Criteria: T-MGMT-017..026, T-MGMT-029/030/031/037/038/039/040/041/042

#### Capability Coverage

- **CAP-AUTHZ-01** + **CAP-CONFIG-01** — `branch_gates_update` (MGMT-BE-004) authorizes `administration:write` at the target ref and writes inert-until-committed `gates.toml`; the gate-config writer is the previously-unowned producer (RUST-3 fix).

#### Next Sprint Tasks

Expanded by `/kb-sprint-tasks-plan` on 2026-06-19 (11/11 tasks fakeability-CLEAN · `proposed_by` tripwire 11/11 · avg rubric 114.3/115 · 1 full red-hat goal loop — fresh `rust-reviewer` + `sveltekit-reviewer` + `security-auditor`; 9 CRITICAL + 15 MEDIUM resolved by the retained writers and re-verified, 5 advisory recorded). Detail files in [`tasks/sprint-06b-governance-ui-branch-gates-rules-safety/`](./tasks/sprint-06b-governance-ui-branch-gates-rules-safety/):

- `MGMT-BE-004-branch-gates-config-writer.md`
- `MGMT-BE-003-principal-scoped-rules-query.md`
- `MGMT-UI-004-error-boundary-wrap.md`
- `MGMT-UI-009-branch-gates-list.md`
- `MGMT-UI-010-ruleslist-principalid.md`
- `MGMT-UI-011-accessibility-ipc-retry.md`
- `MGMT-UI-012-build-gate-tests.md`
- `DESIGN-MGMT-004-denial-no-flip-contract.md`
- `DESIGN-MGMT-006-empty-states.md`
- `DESIGN-MGMT-007-four-tab-a11y-contract.md`
- `DESIGN-MGMT-008-error-boundary-ipc-retry.md`

Capstone E2E appended by `/kb-sprint-tasks-plan` on 2026-06-20 (3 tasks · `proposed_by` tripwire 3/3 · fakeability-clean negative controls on all behavioral ACs · 1 full red-hat goal loop — fresh `rust-reviewer` + `sveltekit-reviewer`; 8 CRITICAL + 9 MEDIUM resolved by the retained writers; advisories U1/U2/U3 recorded). Drives the **real** web-target governance Svelte UI against a **real `but-server`** through all 6 human-test-gate steps (Playwright; Tauri-shell fidelity = WebdriverIO fast-follow):

- `E2E-MGMT-BE-001-governance-e2e-fixtures.md`
- `E2E-MGMT-BE-002-but-server-governance-routes.md`
- `E2E-MGMT-UI-001-governance-capstone-playwright.md`

---

### Sprint 07: Local Agent PR — Governed-Review Parity

**Sequence:** 9
**Timeline:** Phase 5 — Local governed-review parity (the drive layer)
**Status:** Planned
**Proposed by:** rust-planner (LPR enrichment v1.5.0); grounded against the shipped `but-db`/`but-api`/merge-gate tree + an adversarial `rust-reviewer` red-hat pass (2 CRITICAL grounding bugs caught + fixed — the agent-tag source and the table count)
**Milestone:** — (`sprint-07-local-agent-pr`)

> **Human-directed slot.** LPR was placed at **Sprint 07** by human directive (instruction-precedence #1), ahead of STEER (renumbered to Sprint 08). LPR depends on Sprint 05's CLI surface + the shipped merge gate; it does **not** depend on STEER. Scope: enrichment [`v1.5.0-local-agent-pr/`](./enrichments/v1.5.0-local-agent-pr/README.md).
>
> **Safe seam (load-bearing).** Every table/field LPR adds is additive **drive-metadata** that **never** feeds the merge gate — the gate reads only `local_review_verdicts` at head, unchanged. The three new tables (`local_review_assignments`, `local_review_comments`, `local_review_meta`) and the optional principal `kind` config field are orchestration/descriptor only; a forged or empty drive table yields an identical merge decision. Proven by file:line + a no-read build-gate grep over all three tables (v1.5.0 §E).

#### Human Testing Gate

**Gate:** An agent principal opens a local review on a feature branch with `keep_reviews_local` set (no remote GitHub PR is created); a reviewer principal **distinct from the branch author** is assigned via `but review request`/`assign` and posts a file/line comment via `but review comment`; `but review status` shows the assignment, the open comment thread, the derived lifecycle, and an `agent-authored` tag; the reviewer approves via `but review approve` and the orchestrator merges through the **unchanged** merge gate; a self-assignment is rejected and an unauthorized self-resolve cannot clear another party's thread; and a forged or empty set of `local_review_assignments` + `local_review_comments` rows yields an **identical** merge-gate decision.

**Test Steps:**

1. As an agent principal, `but review request <reviewer>` on a feature branch; confirm `but review status` shows a `pending` assignment and **no** remote PR was opened (`keep_reviews_local` default-true).
2. As the reviewer, `but review comment --file <f> --line <n> "..."`; confirm the thread appears in `but review status` as unresolved.
3. Confirm the local PR object carries the `agent-authored` tag — sourced from the **opener principal's declared `kind = "agent"` in committed `.gitbutler/permissions.toml`** (read at the target ref; cached in the dedicated `local_review_meta` opener row), **not** from `BUT_AGENT_HANDLE` resolution; a human-declared (`kind = "human"`/omitted) opener does not carry it.
4. Assign a reviewer **equal to** the branch author (`but review assign <author>`); confirm it is rejected (distinct-from-author enforced at the `but-api` boundary).
5. As the reviewer, `but review approve`; as the orchestrator (`maintain`), `but merge` — confirm it lands through the unchanged gate.
6. Post a `changes_requested`-style thread and attempt to self-resolve it as a non-author/non-assigned/non-`reviews:write` principal; confirm the resolve is rejected and the thread stays unresolved (no forged all-clear).
7. Forge a fake approving `local_review_assignments`/`local_review_comments` set with no verdict-at-head; confirm `but merge` is **still** denied `gate.review_required` (drive tables never gate).
8. A principal lacking `reviews:write` runs `but review assign`; confirm `{code:"perm.denied", remediation_hint}` + exit 1.

#### Tasks (proposed — expanded by `/kb-sprint-tasks-plan`)

| ID             | Title                                                                                                                                                                                                                                                                                                                                                                                                              | Agent                   |
| -------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ----------------------- |
| LPR-001        | `local_review_assignments` + `local_review_comments` + `local_review_meta` tables + 3 `SchemaVersion::Zero` migrations + 3 structs + Handle/HandleMut pairs (`list_by_target`, upsert/insert, `set_state`/`set_resolved`, `list_by_thread`, meta `upsert_if_absent`/`get`)                                                                                                                                         | rust-implementer        |
| LPR-002        | `AssignmentState { Pending, Approved, ChangesRequested }` typed enum + boundary (de)serialization (the `Authority` parse/`name` round-trip); column stays `TEXT`; **no** new `Authority` variant                                                                                                                                                                                                                   | rust-implementer        |
| LPR-003        | `request_review` (`PullRequestsWrite`, +`local_review_meta` opener row) / `assign_reviewer` (`ReviewsWrite`, **distinct-from-author** at the boundary) verbs + `but review request`/`assign` CLI; **implement the real `changes_requested` write** (`request_changes_review` is a stub today); structured `perm.denied` + exit 1 on missing authority; self-assignment rejected                                    | rust-implementer        |
| LPR-004        | `post_comment`/`list_comments`/`resolve_thread` verbs (`CommentsWrite` writes; **resolver-identity** constraint — author / assigned reviewer / `reviews:write` — on resolve; branch-scoped reads) + `but review comment --file/--line/--thread`/`comments`/`resolve` CLI; **`post_comment` REPLACES the stubbed `comment_review`** (re-point the `comment` CLI verb at it); local-cache write, no DryRun guard     | rust-implementer        |
| LPR-005        | `review_status` **derived** PR lifecycle (commits + verdict-at-head + open assignments; read-only `gix` walk, NO mutation) + the agent-PR tag derived from the **opener principal's declared `kind` in committed `permissions.toml`** (the additive optional `kind` field on `PrincipalWire`, read at the target ref — NOT handle-resolution, NOT a comment body), cached in the dedicated `local_review_meta` row | rust-implementer        |
| LPR-006        | `Project.keep_reviews_local: DefaultTrue` (per-project operator preference — NOT `administration:write`-gated, NOT ref-pinned; the `ok_with_force_push` `DefaultTrue` precedent) + default-local wiring + remote-mirror **gate** (the mirror path is NOT built — named seam only; principal→forge disclosure named under R21)                                                                                      | rust-implementer        |
| LPR-007        | `but-rules` auto "review-requested" hook (commit Trigger → Filter → Action opens a `pending` assignment), reusing the Sprint-06b Trigger→Filter→Action engine — no new rules mechanism                                                                                                                                                                                                                             | rust-implementer        |
| LPR-008        | **Reconciler read-API**: `review_status` serves the full drive state (assignments + unresolved comments + verdict-at-head) in one payload, so two orchestrators converge; two-read agreement proof                                                                                                                                                                                                                 | rust-implementer        |
| LPR-009        | **Safe-seam invariant**: net-new build-gate honesty grep (gate path has NO ref to the 3 new tables) + the forged-vs-empty + inverse integration tests (drive metadata alone never lands; only verdict-at-head flips)                                                                                                                                                                                               | rust-reviewer           |
| LPR-010        | TS SDK regen (`pnpm build:sdk && pnpm format`) + N-API audit (R14 — the verbs ARE `but-api` fns) + happy-path CLI tests; honesty/anti-fakeability greps (tag-not-an-enforcement-key; tag sourced from `local_review_meta`, not a comment body); drive-layer-integrity proofs (self-assignment rejected T-LPR-043; unauthorized self-resolve cannot suppress a signal T-LPR-044)                                    | rust-reviewer           |
| LPR-011        | Reconciler usage-model doc + the `but-*` skill contract (`keep_reviews_local=true` on governed-project init) — the skill workflow auto-sets local; skills _implementation_ is OUT of this sprint (documented contract only)                                                                                                                                                                                        | rust-implementer / docs |
| DESIGN-LPR-001 | `keep_reviews_local` toggle — Project-Settings design contract                                                                                                                                                                                                                                                                                                                                                     | frontend-designer       |
| DESIGN-LPR-002 | Principal `kind` (agent/human) — Principals-tab design contract                                                                                                                                                                                                                                                                                                                                                    | frontend-designer       |
| DESIGN-LPR-003 | Local-review view — IA + four-state read-only contract                                                                                                                                                                                                                                                                                                                                                             | frontend-designer       |
| LPR-012        | `keep_reviews_local` toggle in Project Settings (project-settings path; NOT admin-gated)                                                                                                                                                                                                                                                                                                                           | sveltekit-implementer   |
| LPR-013        | `principal_kind` Tauri command + SDK producer (governed-config, `administration:write`-gated)                                                                                                                                                                                                                                                                                                                      | tauri-implementer       |
| LPR-014        | Principal `kind` field in the Principals editor (governance IPC)                                                                                                                                                                                                                                                                                                                                                   | sveltekit-implementer   |
| LPR-015        | Local-review READ producer — `review_status`/`list_comments` Tauri commands + SDK (branch-scoped)                                                                                                                                                                                                                                                                                                                  | tauri-implementer       |
| LPR-016        | `LocalReviewView` read-only panel (assignments/threads/lifecycle/agent-tag; no merge affordance)                                                                                                                                                                                                                                                                                                                   | sveltekit-implementer   |

> **UI/full-stack extension** (appended 2026-06-21): LPR-012–016 + DESIGN-LPR-001–003 add the local-only toggle, the principal agent/human tag, and the read-only local-review view on top of the backend LPR-001–011 (which stays runnable independently). UI deps: LPR-013→014, LPR-015→016 + the DESIGN-LPR-\* contracts. None touch the merge gate.

#### Dependencies

- Blocks: None (drive layer; consumed by the `but-*` skills — see [`skills-migration-local-agent-pr.md`](./skills-migration-local-agent-pr.md))
- Dependent on: Sprint 01b (the `approve_review` verdict write + merge gate), Sprint 04 (merge strictness), Sprint 05 (CLI surface), the shipped `but-rules` engine (Sprint 06b)

#### PRD Coverage

- UC-LPR-01..07 / T-LPR-001..044 (+ T-LPR-029h) (enrichment v1.5.0)
- Risks: R18 (loop-sourced-receipt forgeability), R19 (agent-tag spoof via `BUT_AGENT_HANDLE` re-export to impersonate a _different declared principal_), R20 (comment-body injection into agent context), R21 (`keep_reviews_local` trusted-desktop preference + deferred principal→forge disclosure), R22 (same-principal drive-layer forgery — self-assignment / self-resolve, narrowed by the distinct-from-author + resolver-identity constraints), R23 (DB-row forgery of the agent-tag derivation control path) — named, accepted residuals (R22's constraints are real integrity checks but narrow _cross-principal_ forgery only)

#### Next Sprint Tasks

Pending `/kb-sprint-tasks-plan` (Stage 3) — task files at [`tasks/sprint-07-local-agent-pr/`](./tasks/sprint-07-local-agent-pr/).

---

### Sprint 08: STEER — Capability-Aware Denials

**Sequence:** 10
**Timeline:** Phase 6 — Capability-aware denial steering
**Status:** Planned
**Proposed by:** rust-planner (STEER enrichment v1.4.0)
**Milestone:** — (`sprint-08-steer`)

> **Renumbered 07→08** by human directive (LPR took slot 07). STEER's scope is unchanged; the prior "Sprint 07" wording in the v1.4.0 enrichment + the `tasks/` dir is reconciled to 08. Scope: enrichment [`v1.4.0-capability-aware-denials/`](./enrichments/v1.4.0-capability-aware-denials/README.md). Depends on Sprints 02/04/05; **not** on LPR.

#### Human Testing Gate

**Gate:** A denied principal receives a `class`, its `held_permissions`, an `authorized_actions` menu of governed `but` commands runnable in its stated context, and a `do_not`; a reviewer denied a commit follows a listed `but review` action (not `approve` on its own branch) to a successful review; an unknown-principal / `config.invalid` denial returns `operator_required` + empty menu + "do not retry"; and every menu entry, run in its stated context, is itself not denied.

#### Tasks (STEER-001..010)

Authored as [`tasks/sprint-08-steer-capability-aware-denials/`](./tasks/sprint-08-steer-capability-aware-denials/) (renamed from `sprint-07-steer*`). See enrichment v1.4.0 `05-delta-replan.md` §2 for STEER-001..010.

#### Dependencies

- Dependent on: Sprint 02 (denial primitive), Sprint 04 (merge strictness), Sprint 05 (`but perm list` + CLI surface)

#### PRD Coverage

- UC-STEER-01..06 / T-STEER-001..031 (enrichment v1.4.0); risks R15/R16/R17

---

## Red-Hat Review Summary

|                                   | Value                                                                                                                                                                                                                                                   |
| --------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Review cycles                     | 1 full cycle (3 fresh reviewers: `rust-reviewer`, `sveltekit-reviewer`, `security-auditor`) + writer remediation                                                                                                                                        |
| Blocking findings resolved        | 8 CRITICAL + ~10 MEDIUM (all by the original writers)                                                                                                                                                                                                   |
| Upstream (locked-PRD) escalations | 0 — every gap was a roadmap/grounding gap, fixable in re-planning                                                                                                                                                                                       |
| Convergence                       | Deterministic [5.4] re-validation of the remediated structure: 0 banned-pattern violations · all step counts 3–8 · acyclic graph · 17/17 UCs covered · both capability chains owned (incl. the new DryRun + commit-gate-target-ref + T-LOOP-013 proofs) |
| Residual                          | A second fresh-panel re-review was bounded to deterministic re-validation for cost (planning artifact). Advisory follow-ups carried into task briefs: governance sidebar icon-name verification; Svelte 5 `svelte:boundary` mechanism choice.           |

## Next Steps

1. Expand the first sprint's tasks when ready to execute:
   ```
   /kb-sprint-tasks-plan .spec/prds/governance/ROADMAP.md
   ```
2. Run a sprint:
   ```
   /kb-run-sprint sprint-01a-authz-primitive-commit-gate
   ```
3. Re-plan after PRD edits (updates ROADMAP.md in place):
   ```
   /kb-sprint-plan .spec/prds/governance --delta-replan
   ```
