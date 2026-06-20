---
sprint: 07
sequence: 9
timeline: Phase 5 — Capability-aware denials (v1.4.0 enrichment; appended after Sprint 06b)
status: In Progress
proposed_by: rust-planner
milestone: sprint-07-steer-capability-aware-denials
prd: ../../README.md
enrichment: ../../enrichments/v1.4.0-capability-aware-denials/README.md
roadmap: ../../ROADMAP.md
generated_by: kb-sprint-tasks-plan
---

# Sprint 07: STEER — Capability-Aware Denials

**Sequence:** 9
**Timeline:** Phase 5 — Capability-aware denials (v1.4.0 enrichment; appended after Sprint 06b)
**Status:** In Progress
**Proposed by:** rust-planner
**Milestone:** — (`sprint-07-steer-capability-aware-denials`)

## Overview

The **STEER** group is the v1.4.0 *capability-aware-denials* enrichment — the **tuning step** that turns
GitButler's already-strong *informative* denials into optimal *steering* ones. Sprints 01a–06b built and
hardened the enforcement core: the `but-authz` `Denial`/`AuthoritySet`/`authorize()` primitive, the commit +
merge gates, fail-closed identity confinement, ref-pinned grouping, the `but perm`/`but group` CLI write
verbs, and the Governance UI. Every one of those denials is honest and well-formed — but it mostly points
**up and out** (*"ask a maintainer to grant `merge`"*), an out-of-band action the agent cannot perform
itself. That is where a goal-directed agent **pools and overflows**: it hard-quits ("I'm blocked"), retry-loops
the same denied call, or defects to a destructive bypass (raw `git`, `--no-verify`).

STEER makes every **actor-correctable** denial point **down and across** — *"with what you hold, here is what
you can do right now"* — so the water always finds a channel. It is **net-additive**: it changes **no** gate
decision, **no** denial `code`, and **no** fail-closed posture. It adds four fields to the denial *payload*:

```jsonc
{"error":{
  "code":"branch.protected",                 // existing, stable machine code — UNCHANGED
  "class":"actor_correctable",               // NEW — actor_correctable | operator_required (gates retry vs escalate)
  "message":"direct commits to protected 'main' are denied for principal 'rev'",   // existing prose — UNCHANGED
  "remediation_hint":"land 'main' via a reviewed merge",   // existing — vertical path to the ORIGINAL intent
  "held_permissions":["reviews:write","comments:write"],   // NEW — caller's EFFECTIVE set (own ∪ groups), self-scoped
  "authorized_actions":[                       // NEW — lateral menu, derived + intent-scoped (no self-approve, no lying menu)
    {"command":"but review request-changes","effect":"reject this change with line comments"},
    {"command":"but perm list","effect":"see full permissions, groups, and authorized actions"}
  ],
  "do_not":"do not commit directly or bypass with raw git — protected refs only move via reviewed merge"  // NEW — anti-pool prohibition
}}
```

`remediation_hint` (the **vertical** channel — keep the *original goal* flowing toward landing) and
`authorized_actions` (the **lateral** channel — stop the agent *pooling*) are deliberately distinct and both
retained. This is HATEOAS-for-authz: each rejection carries the state-transitions available to *this* principal.

> **Re-grounded against the shipped tree (rust red-hat pass).** The early "it's already there / it's free /
> behavior-neutral" framing was wrong at several points and is corrected in
> [`03-technical-requirements-delta.md`](../../enrichments/v1.4.0-capability-aware-denials/03-technical-requirements-delta.md):
> there are **four** denial carriers, not one — `Denial` (`but-authz/src/denial.rs:13`, exactly three fields,
> no `Serialize`), `MergeGateError` (`but-api/src/legacy/merge_gate.rs:19`, has `unmet`+`Serialize`),
> `ConfigError` (`but-authz/src/config.rs`, `thiserror`), and the commit gate's two-field `CommitGateError`
> wrapper (`but-api/src/commit/gate.rs:9`). `unmet` is a `MergeGateError` field **only** — never on `Denial`.
> The CLI hand-rolls denial JSON in **three** places (no single `json::Error` CLI path); two already drop
> `remediation_hint` today. `Authority` does **not** derive `Serialize`. `branch_protected(principal, branch)`
> (`gate.rs:159`) does **not** receive `cfg`, so the gate-state-aware menu requires a real signature change.
> There is **no** `Route` type / `ROUTE_AUTHORITY_TABLE` yet — STEER-002 is a genuine multi-site refactor,
> behavior-neutral only for the deny/allow decision. `but perm list` **is** present
> (`crates/but/src/command/perm.rs` → `governance::perm_list`, Sprint 05), so the discovery affordance is
> non-phantom. The honesty grep (`but-authz/tests/invariant_build_gates.rs`) already covers `governance.rs`
> and asserts `AUTHORITY_POSITIVE_PATTERN` — STEER-002 must keep those literals matching; STEER-010 adds new
> patterns beside them.

### The load-bearing mechanism — gate-state-aware derivation (no lying menu)

A pure authority intersection is **unsound** for `branch.protected`: branch protection is
`authority ∧ ¬protected`, so a caller who hit `branch.protected` still *holds* `contents:write` — a naive
`required_authority ⊆ held` would offer the very `commit` that was just denied (a *lying menu* that loops the
agent down a blocked channel). The corrected derivation **subtracts the (route, predicate, ref) that actually
fired** and binds every entry to a *succeeding context* (e.g. commit to a **feature** branch — a different,
unprotected ref — plus review actions), derives from the **exact `cfg` the gate already loaded at the target
ref** (menu and gate cannot diverge), and **excludes `but review approve`** when the denial targets the
caller's own branch (an L1 contract exclusion, never left to the reference primer).

This sprint is **headless/CLI and UI-independent** — the MGMT render of the menu is deferred to a future
Sprint 06c. Every property is verified by running a `but` command and asserting on the **structured JSON
denial on stderr + exit code 1**, the same hand-assertion style (not `insta` snapshots) as the shipped
`commit_gate` / `merge_gate` / `governed_loop` tests. Every gate proof draws from
[`04-e2e-testing-criteria.md`](../../enrichments/v1.4.0-capability-aware-denials/04-e2e-testing-criteria.md)
(T-STEER-001..031).

## Human Testing Gate

**Gate:** A denied principal receives a `class`, its `held_permissions`, an `authorized_actions` menu of
governed `but` commands runnable in their stated context, and a `do_not` — and following any listed action
succeeds while no listed action reproduces the denial that was just returned.

### Test Steps

1. Commit `permissions.toml` + protected `gates.toml`; run a commit denial as `dev` → stderr JSON carries `class`, `held_permissions`, `authorized_actions`, `do_not`.
2. Run a commit on protected `main` as a reviewer → menu lists `but review request-changes`/`comment`, never `but review approve`.
3. Follow a listed `authorized_actions` command from step 2 → exit 0, governed action succeeds.
4. Run a commit with `BUT_AGENT_HANDLE` unset → `class: operator_required`, `authorized_actions == []`, `do_not` says do-not-retry.
5. Commit a malformed `gates.toml`, run a gated action → `config.invalid`, `class: operator_required`, empty menu.
6. Run any actor-correctable denial → `authorized_actions` includes the `but perm list` discovery command.
7. Parse a merge denial with a `code`/`message`/`remediation_hint`/`unmet` reader → those keys + exit 1 unchanged.

## Tasks

| ID | Title | Agent | Estimate |
|----|-------|-------|----------|
| STEER-001 | Steering fields (`class`/`held_permissions`/`authorized_actions`/`do_not`) on `Denial` + `MergeGateError` + `ConfigError` + the `CommitGateError` envelope; add `DenialClass`/`AuthorizedAction` types + derives; `Authority` `Serialize` (stable `:` token) or per-serializer `name()` mapping, stable lexical order | rust-implementer | 210 min |
| STEER-002 | `Route` enum + single-source `ROUTE_AUTHORITY_TABLE` in `but-authz`; compose non-authority predicates around it; reconcile the `forge.rs` `authorize_branch_action` match incl. `other =>`; preserve the `AUTHORITY_POSITIVE_PATTERN` honesty grep (keep literal `authorize`/`Authority::*` or update `invariant_build_gates.rs`) | rust-implementer | 270 min |
| STEER-003 | Gate-state-aware `authorized_actions` derivation: `effective_set ∩ table` minus the failed `(route, predicate, ref)`, intent-scoped via the curated `AFFORDANCE_MAP`, self-approve excluded on own-branch, all text from the closed `&'static str` catalog | rust-implementer | 240 min |
| STEER-004 | Wire the payload + exhaustive non-defaulted `(code, principal-resolution) → class` match into all constructors/gates; change `branch_protected(principal, &cfg, branch)` to re-call `effective_authority` for a gate-state-aware menu; `config.invalid`/no-handle/unknown-principal → `operator_required` + empty menu + `do_not` | rust-implementer | 210 min |
| STEER-005 | Add the four fields to the three hand-rolled CLI serializers (`commit_gate_cli_error`, `review_gate_cli_error`, `merge_gate_cli_error`); coordinate the desktop surface with Sprint 06a `MGMT-IPC-002` (`json::Error`); best-effort serialization — a fault still emits `code`/`message`/`remediation_hint` + exit 1 | rust-implementer | 180 min |
| STEER-006 | `but whoami` / `but can-i` self-scoped discovery (effective perms + own group memberships + authorized-action set); surface `but perm list` as the menu discovery affordance, degrade (omit) if absent; no other-group-member enumeration without `administration:read` | rust-implementer | 210 min |
| STEER-007 | Denial-steering telemetry event (`code`, `class`, `had_lateral_action`, menu length) on the existing tracing path | rust-implementer | 120 min |
| STEER-008 | Ship the non-enforced agent-priming reference primer (denials=redirects, affordances=options-not-orders, no-bypass, `class`/`do_not` contract); prove no `but-authz`/`but-api` path depends on it for correctness | rust-implementer | 90 min |
| STEER-009 | Extend `governed_loop` for gate-state-aware no-lying-menu — replay each offered action in its stated context, plus a concurrent-ref-advance case (clean re-denial) and a serialization-fault case (exit 1); audit and update any whole-object-equality assertions on `Denial`/`MergeGateError` | rust-implementer | 240 min |
| STEER-010 | Net-new honesty build-gates: closed-catalog grep (no `format!`/interpolation/config-sourced text in `authorized_actions`/`do_not`) + table/affordance coverage grep (every gated route ∈ `ROUTE_AUTHORITY_TABLE`; every table route has an `AFFORDANCE_MAP` entry not naming the denied route) + review | rust-reviewer | 120 min |

## Dependencies

- **Blocks:** None
- **Dependent on:** Sprint 02 (denial primitive + fail-closed), Sprint 04 (merge strictness + `unmet` requirement engine), Sprint 05 (`but perm list` + persisted governance config + the honesty grep). **Coordinates with Sprint 06a `MGMT-IPC-002`** (the `json::Error` Tauri serializer) for desktop-surface steering fields.
- **Intra-sprint order is a strict chain:** STEER-001 → STEER-002 → STEER-003 → STEER-004 → STEER-005; STEER-006 / STEER-007 / STEER-008 layer after STEER-004; STEER-009 → STEER-010 close the proof.

## PRD Coverage

- **Use cases:** UC-STEER-01, UC-STEER-02, UC-STEER-03, UC-STEER-04, UC-STEER-05, UC-STEER-06
- **Criteria:** T-STEER-001..031 (18 integration · 6 api-contract · 7 build-gate)

## Capability Coverage

- **CAP-STEER-01 — capability-aware denial.** Producer: gate-state-aware `authorized_actions` derivation
  (STEER-003) over the single-source `ROUTE_AUTHORITY_TABLE` (STEER-002), wired through the exhaustive `class`
  mapping (STEER-004) and serialized at the three CLI sites (STEER-005); no-lying-menu proven by the extended
  `governed_loop` (STEER-009); closed-catalog + single-source coverage proven by the net-new honesty greps
  (STEER-010). Fail-closed preserved — a derivation or serialization fault still returns
  `code`/`message`/`remediation_hint` + exit 1 (STEER-005/009). Owner: `rust-implementer`; reviewers:
  `rust-reviewer` + `security-auditor`.

## Coverage Notes

- **Four denial carriers, not one (STEER-001).** The steering fields land on `Denial`, `MergeGateError`,
  `ConfigError`, **and** the commit gate's `CommitGateError` envelope. `ConfigError` gets `class` + `do_not`
  only (no held set / menu — it is always `operator_required`). `held_permissions` is populated **only** on
  the `missing_permission` (resolved-principal) path; it is structurally empty on the unresolved-principal and
  `config.invalid` paths.
- **`class` is exhaustive by (code, principal-resolution), not by code alone (STEER-004).** `perm.denied`
  splits: a *resolved* principal lacking authority is `actor_correctable`; an *unresolved* principal
  (no-handle / unknown-principal — same `perm.denied` code) is `operator_required` with an empty menu and a
  do-not-retry `do_not`, because such a caller cannot self-correct in-system (security HIGH #2). The mapping
  is a non-defaulted `match` — adding a future code/cause without classifying it is a **compile break**, never
  a silent `actor_correctable`.
- **No lying menu, gate-state-aware (STEER-003).** Every offered action, run in its stated context, must
  succeed for that caller. For `branch.protected` the affordance is a **feature-branch** commit (a different,
  unprotected ref) + review — never the protected-ref commit just denied. `but review approve` is **excluded**
  on the caller's own branch (L1 exclusion). The menu derives from the same `cfg`/ref the gate judged against
  (a runtime property — proven by T-STEER-009/024 integration tests, **not** a static grep).
- **`ROUTE_AUTHORITY_TABLE` is a real refactor (STEER-002).** No `Route` type or table exists; authority
  checks are scattered (commit: authorize + branch-protection predicate; merge: authorize + review-requirement
  engine; forge `authorize_branch_action`: a `match` with an `other => authorize(p, other)` arm; admin write).
  The table + catalog live in **`but-authz`** (so `authorize`/the menu use them with no `but-authz → but-api`
  cycle, per RULES.md); the gates in `but-api` consume them. Non-authority predicates stay **out** of the
  table but composed around it. Behavior-neutral for the deny/allow **decision** only — size accordingly.
- **Honesty grep is net-new, not an extension (STEER-010).** `invariant_build_gates.rs` asserts
  no-role-preset / no-human-vs-AI / positive-`authorize` / no-`Permission`. STEER-010 **adds** patterns beside
  those: a closed-catalog grep (no `format!`/interpolation/config-sourced text in `authorized_actions`/`do_not`
  construction) and a table/affordance coverage grep (every gated route ∈ `ROUTE_AUTHORITY_TABLE`; every table
  route has an `AFFORDANCE_MAP` entry not naming the denied route). The closed-catalog grep covers the **new**
  fields only — `message`/`unmet[]` already interpolate config strings (R15, mitigated separately) and must
  **not** be claimed closed.
- **Discovery is a Sprint 05 dependency + net-new STEER work (STEER-006).** `but perm list` ships from Sprint
  05 (`crates/but/src/command/perm.rs`) — STEER surfaces it as the menu discovery affordance, **degradable**
  (omitted, never a phantom command, if absent). The friendlier `but whoami` / `but can-i` bundle (effective
  perms + own groups + authorized-action set, self-scoped) is net-new STEER work. Discovery stays self-scoped:
  it discloses the caller's own memberships but **not** the other members of its groups (group-roster recon
  stays gated by `administration:read`, Sprint 05).
- **Best-effort additive — enforcement never weakens (STEER-005/009).** Existing fields render independently
  of the new ones; a fault deriving **or** serializing the steering payload still yields
  `code`/`message`/`remediation_hint` + exit 1, and never turns a deny into an allow or drops an existing
  field. The full steering payload is emitted under **DryRun** while persisting nothing (DryRun-no-bypass
  preserved).
- **Tests are assertion-based, not snapshots (STEER-009).** `governed_loop.rs` /
  `but-api/tests/commit_gate.rs` / `merge_gate.rs` parse the envelope and substring-match — `SNAPSHOTS=overwrite`
  does **not** apply. Additive JSON fields keep key-readers passing; STEER-009 must **audit and update any
  whole-object-equality** assertion (`assert_eq!` on a full `Denial`/`MergeGateError` or serialized blob),
  which breaks on new fields, and add positive assertions for the new fields.
- **Agent priming is L2, non-enforced (STEER-008).** The primer (denials are redirects; affordances are
  options not orders; bypass is never the faster path; stop on `operator_required`, never bypass) is shippable
  **reference** material — STEER-008 must prove **no** `but-authz`/`but-api` code path depends on it for
  correctness (Stance 6: the harness owns the agent).
- **Implementation is out of scope for this artifact:** these are TDD **task contracts**. The Rust (the carrier
  fields + `to_envelope()`, the `Route` table, the gate-state-aware derivation, the `class` wiring +
  `branch_protected` signature change, the three CLI serializers, the `whoami`/`can-i` discovery verbs, the
  telemetry event, the primer, the extended `governed_loop`, and the net-new greps) is written at execution
  time by `/kb-run-sprint`, RED→GREEN against these specs against **real `but-authz` + real `gix` git fixtures**.

## Source Specification

This sprint materializes the **v1.4.0 Capability-Aware Denials enrichment**:

- [`enrichments/v1.4.0-capability-aware-denials/02-uc-steer.md`](../../enrichments/v1.4.0-capability-aware-denials/02-uc-steer.md) — UC-STEER-01..06 (32 ACs)
- [`enrichments/v1.4.0-capability-aware-denials/03-technical-requirements-delta.md`](../../enrichments/v1.4.0-capability-aware-denials/03-technical-requirements-delta.md) — the four carriers, gate-state-aware derivation, the `Route` table, invariants, L1/L2/L3
- [`enrichments/v1.4.0-capability-aware-denials/04-e2e-testing-criteria.md`](../../enrichments/v1.4.0-capability-aware-denials/04-e2e-testing-criteria.md) — T-STEER-001..031
- [`enrichments/v1.4.0-capability-aware-denials/05-delta-replan.md`](../../enrichments/v1.4.0-capability-aware-denials/05-delta-replan.md) — code deltas D1–D10, risks R15–R17, the proposed Sprint 07 task table

## Red-Hat Review Summary

Expanded by `/kb-sprint-tasks-plan` on 2026-06-19 — **1 full red-hat goal loop, 2 cycles + retained-writer
remediation + deterministic re-validation**. 10/10 tasks fakeability-CLEAN (`validate_scenario.py`, 0
CRITICAL/HIGH on all 54 behavioral ACs) · `proposed_by` tripwire 10/10 (`rust-planner`) · stable gapless
AC-N/TC-N (54 ACs / 79 TCs) · acyclic intra-sprint chain.

A fresh adversarial panel (`rust-reviewer` + `security-auditor`, no authoring context) **BLOCKed** the cycle-1
draft with **4 CRITICAL + 6 MEDIUM + 4 LOW** spec-correctness/coverage findings the rubric + fakeability gates
cannot see — all coverage-side (no locked-PRD escalation), all remediated by the retained `rust-planner`
writers and confirmed **22/22 CLOSED, 0 reopened** by a fresh cycle-2 panel (both APPROVE):

- **RR-1 / RR-2 (CRITICAL)** — the locked enrichment §1 under-counts the denial surface: the `review` CLI path
  flattens through **`ForgeGateError`** (forge.rs:16) and a **fourth** CLI serializer **`governance_cli_error`**
  (perm.rs:118) serializes admin-write **`AdminWriteGateError`** (config_mutate.rs) — both omitted from the
  draft, leaving `administration:write` actor-correctable denials non-uniform (violating UC-STEER-01 AC-1 + the
  §5 admin-write affordance row). Remediated: STEER-001 now covers **all SIX carriers** (the two wrappers gain
  the fields and their `classify_error` copies them off the underlying `Denial`/`ConfigError`); STEER-005 adds
  `governance_cli_error` as a fourth serialization site with a covering AC.
- **SA-1 / RR-4 (CRITICAL/MEDIUM)** — STEER-009's serialization-fault proof depended on a seam STEER-005 did
  not contractually provide. Remediated: STEER-005 provides a **test-only** fault seam keyed on env var
  `BUT_STEER_FORCE_SERIALIZATION_FAULT`, `#[cfg(debug_assertions)]`/`cfg(test)`-gated and **compiled out of
  release** (never a production bypass); STEER-009 consumes that exact name.
- **SA-2 (CRITICAL)** — the no-lying-menu replay covered only `branch.protected`. Remediated: STEER-009 now
  replays the `gate.review_required` (merge) and `perm.denied` (missing-authority) menus too — all three
  menu-bearing denial types proven.
- **MEDIUM** — class as a stable serialized enum string on the denial envelope (RR-3, T-STEER-016); bounded
  closed-catalog grep scope + an R15 boundary teeth control (SA-3); a DryRun-under-fault fail-closed case
  (SA-4); exhaustive `class` via a `DenialCause` enum match (no `_ =>`) proven by a real `trybuild` compile-fail
  (SA-5); a principal-existence-oracle AC on `but can-i` (SA-6).

**Upstream advisory (record, non-blocking):** the locked enrichment §1's "four carriers / three CLI serializers"
count is short — `ForgeGateError` + `AdminWriteGateError` wrappers and the `governance_cli_error` 4th site exist.
The task set covers them within Sprint 07; reconcile the enrichment §1 wording via a future delta-replan.

**Deferred advisories (LOW, ACCEPTED — carried to the implementer at GREEN, not a remediation run):**

- **SA2-1** — prefer `cfg(test)` / a dedicated dev-only feature flag over bare `#[cfg(debug_assertions)]` for
  the fault seam (a debug-profile non-test binary also has `debug_assertions`); STEER-010's reviewer pass
  should verify the seam site.
- **SA2-2** — STEER-006 AC-6 should assert full message **equality** between the unknown-target and
  existing-target denials (not only a token blocklist) to fully close the existence-oracle channel.
- **RR2-1** — pin STEER-003's `CATALOG` to `crates/but-authz/src/catalog.rs` (or let STEER-010's grep glob the
  realized module) so the closed-catalog grep scope matches the implemented layout.
- **RR2-2** — prune transitive `blocks` over-declarations (cosmetic; `depends_on` is authoritative and acyclic).
- **RR2-3** — backfill STEER-006 AC-4/5/6 `negative_control.would_fail_if` with the concrete failures already
  named in their THEN clauses (the teeth already live in the THEN + verify).

## Task Detail Files

Generated by `/kb-sprint-tasks-plan` on 2026-06-19.

- STEER-001-steering-fields-on-denial-carriers.md
- STEER-002-route-authority-table-single-source.md
- STEER-003-gate-state-aware-authorized-actions.md
- STEER-004-class-mapping-branch-protected-wiring.md
- STEER-005-cli-denial-serializers.md
- STEER-006-whoami-cani-self-discovery.md
- STEER-007-denial-steering-telemetry.md
- STEER-008-agent-priming-reference-primer.md
- STEER-009-governed-loop-no-lying-menu.md
- STEER-010-honesty-build-gates.md
