---
title: Governance PRD v1.3.0 → v1.4.0 — Capability-Aware Denials Delta-Replan
prd: governance
from_version: 1.3.0
to_version: 1.4.0
posture: net-additive enrichment — spec is documentation-only NOW; code lands as a NEW sprint AFTER the frozen roadmap
last_updated: 2026-06-19
status: planned (frozen-aware; re-grounded after rust/security/PM red-hat pass)
---

# v1.3.0 → v1.4.0 — Capability-Aware Denials Delta-Replan

**Freeze contract.** Sprints 01a–06b are FROZEN with in-flight agents. This plan **edits none of them now**; it records (a) code deltas v1.4.0 implies and (b) additive edits to frozen PRD index files — to apply **when the freeze lifts**, per the `v1.3.0-remediation-plan.md` precedent. Behavior ships as a **new Sprint 07 (STEER)** appended after the roadmap (§2).

> **Re-grounded (red-hat).** The first draft of this plan repeated several ungrounded claims (a `Denial.unmet` field that doesn't exist; a single `json::Error` CLI serializer; `but perm list` as shipped; `ROUTE_AUTHORITY_TABLE` as "behavior-neutral"). All corrected below against the shipped tree.

> **Counting principle (carried from v1.3.0).** Headline counts are recomputed honestly (§4), not held stable while the AC list grows.

---

## 0. Disposition summary

| ID | Area | Disposition | Touches (frozen) | Apply when |
|---|---|---|---|---|
| **D1** | `Denial` (3 fields: `code,message,remediation_hint`) gains 4 steering fields | code delta | Sprint 01a `AUTHZ-001` (`but-authz/src/denial.rs`) | Sprint 07 |
| **D2** | Constructors: `missing_permission` populates held+menu; `no_handle`/`unknown_principal` → `operator_required`, empty | code delta | Sprint 01a/02 (`but-authz/src/authorize.rs`) | Sprint 07 |
| **D3** | `branch_protected(principal,&cfg,branch)` — signature change to thread cfg/held; gate-state-aware menu | code delta | Sprint 01a `GATES-001` (`but-api/src/commit/gate.rs`) | Sprint 07 |
| **D4** | `MergeGateError` (already has `remediation_hint`+`unmet`) gains 4 steering fields | code delta | Sprint 01b/04 (`but-api/src/legacy/merge_gate.rs`) | Sprint 07 |
| **D5** | `config.invalid` has **three** carriers — `ConfigError` (thiserror) + `MergeGateError` — both get `class`+`do_not` | code delta | Sprint 01a/02 (`but-authz/src/config.rs`, `merge_gate.rs`) | Sprint 07 |
| **D6** | **Three** hand-rolled CLI serializers updated (+ Tauri `json::Error` via MGMT-IPC-002) | code delta | Sprint 01b/02/05 (`commit2.rs`, `forge/review.rs` ×2) + **06a `MGMT-IPC-002`** | Sprint 07 (coord 06a) |
| **D7** | `ROUTE_AUTHORITY_TABLE` + `Route` type — a **real refactor**, behavior-neutral only for the decision | code delta | `04-api-design.md` + all gate call sites (01a–05) + the honesty grep | Sprint 07 |
| **D8** | Discovery: `but perm list` is a **Sprint 05 dependency**; `but whoami`/`but can-i` is net-new STEER work | dependency + code delta | Sprint 05 `CLI-001` | Sprint 07 |
| **D9** | Test updates: most are **assertion-based** (not `insta`); additive fields keep key-readers passing; audit whole-object-equality | test delta | Sprints 01a–05 (`commit_gate.rs`, `merge_gate.rs`, `governed_loop.rs`, `but-api/tests/*`) | Sprint 07 |
| **D10** | Honesty grep — **net-new** closed-catalog + table-coverage patterns (no prior "AUTHZ-007 honesty grep" to extend) | test delta | `but-authz/tests/invariant_build_gates.rs` | Sprint 07 |
| **R15–R17** | Named new risks (injection-amplification, lying-menu, goal-hijack/oracle) | risk delta | `07-technical-risks.md` (fold-in I6) | freeze lifts |
| **I1–I6** | Additive edits to frozen PRD index + risk files | doc delta | `README.md`, `03-functional-groups.md`, `ROADMAP.md`, `07-technical-risks.md` | freeze lifts |

---

## 1. Code deltas to shipped sprints (apply at Sprint 07)

### D1 — `Denial` gains 4 additive fields
`but-authz/src/denial.rs:13`. Shipped `Denial { code, message, remediation_hint }` (derives `Debug,Clone,PartialEq,Eq`; **no `unmet`, no `Serialize`**). Add `class`/`held_permissions`/`authorized_actions`/`do_not` + the `DenialClass`/`AuthorizedAction` types (which must derive `Debug,Clone,PartialEq,Eq` so `Denial`'s derives still hold). Decide `Authority` serialization ([03 §1](./03-technical-requirements-delta.md)).

### D2 — Constructors
`authorize.rs`: `missing_permission(missing, held)` (`:113`) already has `held` → populate held + menu, `class=ActorCorrectable`. `no_handle()` (`:146`) / `unknown_principal()` (`:163`) resolve **no principal** → `class=OperatorRequired`, empty held, empty menu, `do_not`="register the principal / set BUT_AGENT_HANDLE; do not retry as-is" (security HIGH #2). `effective_authority(&Principal,&GovConfig)` (`:51`) is reused for `missing_permission` only.

### D3 — `branch_protected` (signature change, NOT free)
`commit/gate.rs`: `authorize(p,ContentsWrite,&cfg)` (`:67`) passes **before** the branch-protection predicate (`:69`), so a `branch.protected` caller holds `contents:write` but the held set is dropped on `Ok`. `branch_protected(principal, branch_name)` (`:159`) must become `branch_protected(principal, &cfg, branch_name)` and re-call `effective_authority` to build a **gate-state-aware** menu that offers a feature-branch commit (different ref) + review (no self-approve), never the protected-ref commit (C5).

### D4 — `MergeGateError` parity
`legacy/merge_gate.rs:19` (derives `Serialize`; has `remediation_hint`+`unmet`). Add the 4 steering fields; a shared `to_envelope()` renders `Denial` + `MergeGateError` to one JSON shape. `gate.review_required` keeps `unmet[]` and gains the menu.

### D5 — `config.invalid` across THREE carriers (C4)
`config.invalid` is produced by **`ConfigError`** (`config.rs`, `thiserror` with `#[source]`, via `classify_error` at `gate.rs:89`) on the authz/commit path **and** by `MergeGateError::config_invalid()` (`merge_gate.rs:369`) on the merge path. Both must carry `class=OperatorRequired` + `do_not`="do not retry — operator must fix committed `.gitbutler` config", empty menu. `ConfigError` gets `class`+`do_not` only (no held/menu).

### D6 — Three CLI serializers (+ Tauri) (C2)
No single `json::Error` CLI path. Update each hand-rolled site: `commit2.rs:~679` `commit_gate_cli_error` (`{code,message}` today — also gains the long-missing `remediation_hint`); `forge/review.rs:~89` `review_gate_cli_error` (same); `forge/review.rs:~246` `merge_gate_cli_error` (`{code,message,remediation_hint,unmet}`). The **Tauri/MGMT** surface uses `but-api/src/json.rs` `Error` (separate); coordinate the new-field add with Sprint 06a **`MGMT-IPC-002`** (which adds `remediation_hint` there). N-API/Tauri steering coverage is either co-landed here or explicitly deferred ([01-scope-delta](./01-scope-delta.md)).

### D7 — `ROUTE_AUTHORITY_TABLE` (a real refactor — C6)
No `Route` type or table exists; authority checks are scattered/heterogeneous (commit: authorize+predicate; merge: authorize+review-engine; forge `legacy/forge.rs:47` `authorize_branch_action`: a `match` with an `other => authorize(p, other)` arm; admin `config_mutate.rs:25`). Introduce a `Route` enum + the enumerable table **in `but-authz`** (no `but-authz→but-api` cycle, RULES.md), compose the non-authority predicates around it, reconcile the forge `match`, and **preserve `invariant_build_gates.rs`'s positive-`authorize` grep** (keep literal `authorize`/`Authority::*` calls or update the grep). Behavior-neutral for the decision; a multi-site refactor otherwise — size it as its own task (STEER-002).

### D8 — Discovery: dependency, not a freebie (C3)
`but perm list` does **not** ship; it is **Sprint 05 `CLI-001`** (`but perm {list,grant,revoke}`). Sprint 07 **depends on** Sprint 05 landing it. `but whoami`/`but can-i` (bundle perms+groups+actions, self-scoped, no other-member enumeration) is **net-new STEER work** (STEER-006). Discovery is **degradable**: omit it from the menu rather than offer a phantom command (preserves no-lying-menu).

### D9 — Tests (assertion-based, not snapshots — M6)
`governed_loop.rs` parses `CliErrorEnvelope { code, message, remediation_hint }` and substring-matches; `but-api/tests/commit_gate.rs`/`merge_gate.rs` assert `denial.code` / `.message.contains(...)`. These are **hand assertions, not `insta`** — `SNAPSHOTS=overwrite` does not apply. Additive JSON fields keep key-readers passing; **audit for any whole-object equality** (`assert_eq!` on a full `Denial`/`MergeGateError` or serialized blob) — those break on new fields and must be updated. Add positive assertions for the new fields. Behavior-neutral for the decision; "only the payload grows" is true only for key-readers.

### D10 — Honesty grep (net-new, not an extension — M1)
There is **no prior "AUTHZ-007 honesty grep"**; `invariant_build_gates.rs` asserts no-role-preset / no-human-vs-AI / positive-`authorize` / no-`Permission`. Add **new** patterns + path constants: closed-catalog (forbid `format!`/interpolation in `authorized_actions`/`do_not` construction) and table-coverage (every gated route ∈ `ROUTE_AUTHORITY_TABLE`; every table route has an `AFFORDANCE_MAP` entry not naming the denied route). A static grep proves single-symbol + coverage only; **same-ref equality is a runtime property** proven by integration tests, not the grep (M2).

---

## 2. Proposed Sprint 07 — STEER: Capability-Aware Denials

Folder `sprint-07-steer-capability-aware-denials` (**sequence #9, slug 07** — mirroring how `06a`/`06b` already diverge slug from sequence; L4). Appended after Sprint 06b; **UI-independent** (MGMT render of the menu deferred). Depends on Sprint 02 (denial primitive), Sprint 04 (merge strictness), **Sprint 05 (`but perm list` + persisted config + the CLI surface)**, and coordinates with Sprint 06a `MGMT-IPC-002`.

**Human Testing Gate.** A denied principal receives a `class`, its `held_permissions`, an `authorized_actions` menu of governed `but` commands runnable in their stated context, and a `do_not`; a reviewer denied a commit follows a listed `but review` action (NOT `approve` on its own branch) to a successful review; an unknown-principal/`config.invalid` denial returns `operator_required` + empty menu + "do not retry"; and every menu entry, run in its stated context, is not itself denied.

**Test Steps** (from [04](./04-e2e-testing-criteria.md)): per T-STEER-001..031.

**Tasks (proposed):**
| ID | Title | Agent | Maps to |
|----|-------|-------|---------|
| STEER-001 | Steering fields on all 3 carriers (`Denial`/`MergeGateError`/`ConfigError`) + `DenialClass`/`AuthorizedAction` + derives + `to_envelope()` + `Authority` serialization | rust-implementer | D1, D4, D5 |
| STEER-002 | `Route` enum + `ROUTE_AUTHORITY_TABLE` single-source in `but-authz` + preserve/adjust the positive-`authorize` honesty grep | rust-implementer | D7 |
| STEER-003 | Gate-state-aware `authorized_actions` derivation (intersection − failed predicate, intent map, self-approve exclusion, closed catalog) | rust-implementer | UC-STEER-02/06, C5 |
| STEER-004 | Wire payload + exhaustive `class` mapping into all constructors/gates; `branch_protected` signature change; `config.invalid` operator_required | rust-implementer | D2, D3, D5 |
| STEER-005 | Update the 3 CLI serializers (+ coordinate Tauri `json::Error` w/ 06a MGMT-IPC-002); best-effort serialization fail-closed | rust-implementer | D6 |
| STEER-006 | `but whoami`/`but can-i` self-discovery (self-scoped; no other-member enumeration); depends on Sprint 05 `but perm list` | rust-implementer | UC-STEER-04, D8 |
| STEER-007 | Denial-steering telemetry event (`code`,`class`,`had_lateral_action`,menu length) on the tracing path | rust-implementer | UC-STEER-03 (observability) |
| STEER-008 | Reference agent-priming primer (non-enforced) | rust-implementer / docs | UC-STEER-05 |
| STEER-009 | Extend `governed_loop` for gate-state-aware no-lying-menu (incl. concurrent-ref-advance + serialization-fault scenarios) + test/whole-object-equality audit | rust-implementer | UC-STEER-06, D9 |
| STEER-010 | Net-new honesty greps (closed-catalog + table/affordance coverage) + review | rust-reviewer | D10 |

**PRD coverage:** UC-STEER-01..06 / T-STEER-001..031.

---

## 3. Risk delta (R15–R17) — replacing the dishonest "+0 risks"

Per the PRD's "name your leaks, never mitigated-closed" doctrine (R1/R6/R14), v1.4.0 adds three named risks (fold into `07-technical-risks.md` = I6):

| Risk | Severity | Statement | Mitigation / residual |
|---|---|---|---|
| **R15 — Denial-as-injection** | Medium | `message` + `unmet[]` interpolate config-derived strings (principal/branch names; `gates.toml` `required_groups`, attacker-influenceable per R13) into the denial JSON that the agent consumes as context. v1.4.0 makes the denial an *explicitly designed* model channel, amplifying this. | Closed-catalog for the new fields; **bound + sanitize** config-derived substrings in `message`/`unmet[]` (length cap, strip control/instruction-like content). Residual = accepted-leak class (R1/R13) until done. NOT covered by the closed-catalog grep. |
| **R16 — Lying-menu / derivation divergence** | Low (mitigated-by-design) | A menu that offers an action then denied loops the agent down a blocked channel. | Gate-state-aware derivation (subtract failed predicate) + same-cfg/ref-by-construction + extended `governed_loop` proof. Residual = the ref-pin temporal window (concurrent advance → clean re-denial, expected). |
| **R17 — Menu goal-hijack + principal-existence oracle** | Low (accepted) | The structured menu could steer an agent to an authorized-but-off-task action (e.g. self-approve), and the structured `class` eases principal enumeration vs prose. | L1 self-approve exclusion + L2 primer (affordances≠orders); `operator_required` on unknown-principal. Residual = Stance-6 class (harness owns agent reasoning), R9-adjacent. |

---

## 4. Count reconciliation (v1.3.0 → v1.4.0)

Baseline cited from the live PRD README: v1.3.0 = 5 groups / 17 UCs / 129 ACs / 129 criteria (67 integration · 38 component · 7 api-contract · 15 build-gate · 2 e2e) / 14 risks; ROADMAP = 8 sprints.

| Metric | v1.3.0 | Δ | **v1.4.0** |
|---|---|---|---|
| Functional Groups | 5 | +1 (STEER) | **6** |
| Use Cases | 17 | +6 | **23** |
| Acceptance Criteria | 129 | +32 (STEER) | **161** |
| ↳ STEER per UC | — | 5·7·6·5·4·5 | **32** |
| Testing Criteria | 129 | +31 | **160** (T-STEER-023 spans 2 ACs ⇒ 31 rows cover 32 ACs) |
| ↳ integration-test | 67 | +18 | **85** |
| ↳ api-contract | 7 | +6 | **13** |
| ↳ build-gate | 15 | +7 | **22** |
| ↳ component-test | 38 | 0 | **38** |
| ↳ e2e-automated | 2 | 0 | **2** |
| Risk register | 14 | **+3** (R15/R16/R17) | **17** |
| Sprints (ROADMAP) | 8 | +1 (Sprint 07, seq #9) | **9** |

Type-tally check: 85+13+22+38+2 = **160** ✓. STEER per-UC AC tally: 5+7+6+5+4+5 = **32** ✓.

---

## 5. Integration edits to frozen PRD index files (apply when freeze lifts)

All **append-style**, no rewrites:
- **I1** — copy `02-uc-steer.md` → top-level `12-uc-steer.md` (no renumbering).
- **I2** — `03-functional-groups.md`: add STEER row + Use-Case-Summary row (STEER · 6 · 32); totals → 6 groups / 23 UCs / 161 ACs.
- **I3** — `README.md`: Document Index row, Quick Stats (groups 5→6, UCs 17→23, ACs 129→161, criteria 129→160, risks 14→17), Version History row, `version: 1.4.0`.
- **I4** — `ROADMAP.md`: Sprint 07 row (seq #9, slug `sprint-07-steer-capability-aware-denials`) + details block + dependency edge; `sprint_count` 8→9. **✅ APPLIED 2026-06-19** — authored by a dispatched rust-planner (NEVER-TIER), appended append-only; no existing sprint section touched. (The rust-planner pass also surfaced a 4th denial carrier, `CommitGateError` — folded into STEER-001.)
- **I5** — fold T-STEER-001..031 into `11-e2e-testing-criteria.md` (+ count line → 160).
- **I6** — fold R15/R16/R17 into `10-technical-requirements/07-technical-risks.md` (+ count → 17).

---

## 6. Verification checklist

- [ ] **D1/D4/D5** all three carriers (`Denial`/`MergeGateError`/`ConfigError`) carry the steering fields; `config.invalid` is `operator_required` + empty menu + do-not-retry.
- [ ] **D2** `no_handle`/`unknown_principal` are `operator_required` (not actor_correctable).
- [ ] **D3** `branch_protected` threads `cfg`/held; its menu offers a feature-branch commit + review (no self-approve), never the protected-ref commit (C5).
- [ ] **D6** all three CLI serializers + the Tauri `json::Error` carry the new fields (coordinated with MGMT-IPC-002); existing fields render independently (fail-closed at serialization).
- [ ] **D7** one `Route`/`ROUTE_AUTHORITY_TABLE` in `but-authz`; positive-`authorize` honesty grep still green.
- [ ] **D9** whole-object-equality tests audited/updated; key-reader tests still pass; not driven by `SNAPSHOTS=overwrite`.
- [ ] **D10/STEER-009** net-new closed-catalog + coverage greps green; extended `governed_loop` proves gate-state-aware no-lying-menu incl. concurrent-ref-advance + serialization-fault.
- [ ] **R15–R17 / I6** named in the risk register (not "+0").
- [ ] **I1–I5** applied only after the freeze lifts; counts read **6 / 23 / 161 / 160 / 17 / 9**.
- [ ] No frozen sprint task file, ROADMAP row, or section file edited before the freeze lifted.
