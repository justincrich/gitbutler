---
stability: TEST_SPEC
last_validated: 2026-06-19
prd_version: 1.4.0
enrichment: capability-aware-denials
---

# E2E / Testing Criteria — STEER (v1.4.0)

31 criteria covering 32 STEER ACs (T-STEER-023 spans two ACs). Verification is **real `but-authz` + real `gix` git fixtures**, driven through the `but` CLI and asserted on the structured JSON denial on stderr + exit code — the same style as the shipped `commit_gate` / `merge_gate` / `governed_loop` tests (`crates/but/tests/`, `crates/but-api/tests/`), which are **hand-assertion tests (not `insta` snapshots)**. No mocks. The traversability proof (T-STEER-024) **extends `governed_loop`** so each offered action is replayed in its stated context against real subsequent runs.

Type tags: `[integration-test]` · `[build-gate]` · `[api-contract]`. (18 integration · 6 api-contract · 7 build-gate.)

## STEER: Capability-Aware Denials

### UC-STEER-01: Capability-aware denial shape
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|-----------|--------|------|-------|-----------|
| T-STEER-001 | Denial from each carrier carries the additive superset | AC-1 | integration-test | Seed committed config; trigger a `perm.denied` (commit, `Denial`), a `branch.protected` (commit, `Denial`), a `gate.review_required` (merge, `MergeGateError`) | PASS: each stderr JSON has `class`, `held_permissions`, `authorized_actions`, `do_not` alongside the existing fields |
| T-STEER-002 | Back-compat: each serializer's existing keys + exit unchanged | AC-2 | api-contract | Parse a denial with a reader of `code`/`message`/`remediation_hint` only | PASS: those keys + exit 1 are unchanged **relative to each site's current output** (note: commit/review CLI sites already omit `remediation_hint` today — adding it is an improvement, not a regression); `unmet` is asserted only on merge denials (it is a `MergeGateError` field, not on `Denial`) |
| T-STEER-003 | `held_permissions` is the structured effective set, stable order | AC-3 | integration-test | Principal `rev` (own `comments:write` + group `reviews:write`) hits a `missing_permission` denial | PASS: `held_permissions` equals `{comments:write, reviews:write}` compared as a **set / sorted** (not positional); populated only on the `missing_permission` path |
| T-STEER-004 | DryRun denial carries full steering payload, persists nothing | AC-4 | integration-test | Run a denied action under `--dry-run` | PASS: denial has all new fields; FAIL if any ref/object/oplog mutated |
| T-STEER-005 | `remediation_hint` and `authorized_actions` both present and distinct | AC-5 | api-contract | Trigger `branch.protected` for a principal with review grants | PASS: `remediation_hint` names the vertical land path AND `authorized_actions` lists lateral moves; neither substitutes for the other |

### UC-STEER-02: Derived, intent-scoped action menu
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|-----------|--------|------|-------|-----------|
| T-STEER-006 | Menu = effective_set ∩ route→Authority | AC-1 | integration-test | Principal holding `{contents:read, comments:write}`; trigger a denial | PASS: every entry's required authority ⊆ held; no entry requires an unheld authority |
| T-STEER-007 | Menu scoped to denied intent, not whole catalog | AC-2 | integration-test | Denied commit-to-protected by a review-capable principal | PASS: menu = landing/review affordances; FAIL if it lists unrelated admin/group verbs |
| T-STEER-008 | Each entry is `{command, effect}` | AC-3 | api-contract | Any actor-correctable denial | PASS: each entry has a literal `but …` `command` and non-empty `effect` |
| T-STEER-009 | Menu derived from the same `cfg`/ref the gate loaded (gate-state-aware) | AC-4 | integration-test | `branch.protected`: principal HOLDS `contents:write` but branch is protected; feature-head edits config | PASS: menu offers a **feature-branch** commit (different ref) + review, and **excludes** the protected-ref commit the caller just failed; reflects the target-ref config the gate used |
| T-STEER-010 | Reviewer denied commit sees runnable review actions, no self-approve | AC-5 | integration-test | `rev` (`reviews:write`, no `contents:write`) attempts a commit on its own branch | PASS: menu includes `but review request-changes` / `comment` and following one returns exit 0; FAIL if `but review approve` (self-approval) appears |
| T-STEER-011 | All menu text from the closed catalog | AC-6 | build-gate | grep over denial construction | PASS: `command`/`effect` are `&'static str` catalog constants; FAIL on any `format!`/interpolated/config-sourced menu text |
| T-STEER-028 | Self-approve excluded for the caller's own branch | AC-7 | integration-test | `branch.protected`/own-branch denial for a principal holding `reviews:write` | PASS: `authorized_actions` contains `request-changes`/`comment` but NOT `but review approve` — an L1 exclusion independent of the primer |

### UC-STEER-03: Recoverability class + graceful degradation
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|-----------|--------|------|-------|-----------|
| T-STEER-012 | `class` correct per (code, resolution) | AC-1 | integration-test | Trigger: missing-authority `perm.denied`; `branch.protected`; `gate.review_required`; **unknown principal / unset handle**; `config.invalid` | PASS: first three → `actor_correctable`; unknown-principal/no-handle AND `config.invalid` → `operator_required` |
| T-STEER-013 | operator_required → empty menu + "do not retry" | AC-2 | integration-test | Commit a malformed `gates.toml` to the target ref; run a gated action | PASS: `config.invalid`, `authorized_actions == []`, `do_not` says do-not-retry/operator |
| T-STEER-014 | Degrade to vertical path when no lateral move | AC-3 | integration-test | Principal holding nothing relevant is denied | PASS: `authorized_actions == []` AND `remediation_hint` names a handoff/admin grant |
| T-STEER-015 | `do_not` positive-only framing by default | AC-4 | api-contract | Any actor-correctable denial | PASS: `do_not` frames the governed path as the only route to landing; FAIL if it enumerates bypass mechanics (default config) |
| T-STEER-016 | Orchestrator can branch on `class` without prose | AC-5 | api-contract | Parse `class` | PASS: `class` is a stable enum string; retry-vs-escalate decidable without reading `message` |
| T-STEER-030 | Denial emits a structured steering telemetry event | AC-6 | integration-test | Capture the tracing/log sink while triggering denials | PASS: each denial emits an event carrying `code`, `class`, `had_lateral_action`, and menu length |

### UC-STEER-04: Self-discovery on demand
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|-----------|--------|------|-------|-----------|
| T-STEER-017 | Discovery action surfaced in the menu (when the verb exists) | AC-1 | integration-test | Any actor-correctable denial, with Sprint 05 `but perm list` present | PASS: `authorized_actions` includes the self-scoped discovery command; if no discovery verb exists yet, it is omitted (no phantom command — no-lying-menu) |
| T-STEER-018 | Group/team membership NOT inline by default | AC-2 | api-contract | Principal in groups `reviewers`,`leads` | PASS: denial has no inline `groups` field; membership reachable only via discovery |
| T-STEER-019 | Discovery returns full self picture | AC-3 | integration-test | Run discovery as `rev` | PASS: shows effective permissions + the caller's own group memberships + full authorized-action set (self) |
| T-STEER-020 | Discovery is self-scoped (no cross-principal) | AC-4 | integration-test | Non-admin runs discovery targeting another principal | PASS: cross-principal recon denied `perm.denied` (Sprint 05 scoping preserved) |
| T-STEER-031 | `whoami`/`can-i` does not enumerate other group members | AC-5 | integration-test | `rev` (member of `reviewers` with other members) runs `but whoami` without `administration:read` | PASS: own membership shown; the other members of `reviewers` are NOT listed |

### UC-STEER-05: Agent-priming reference guidance
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|-----------|--------|------|-------|-----------|
| T-STEER-021 | Reference primer ships | AC-1 | build-gate | Repo check | PASS: a primer doc/snippet exists stating denials=redirects, affordances=options, no-bypass |
| T-STEER-022 | Primer is non-enforced (engine independent) | AC-2 | build-gate | grep engine code | PASS: no `but-authz`/`but-api` code path depends on the primer for correctness (Stance 6) |
| T-STEER-023 | Primer encodes goal-integrity + class contract | AC-3, AC-4 | build-gate | Content check | PASS: primer states "choose the action that serves your task" (affordances≠orders) AND documents `class`/`do_not` (stop on operator_required) |

### UC-STEER-06: Traversability + anti-injection invariants
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|-----------|--------|------|-------|-----------|
| T-STEER-024 | No lying menu — every offered action succeeds in its stated context | AC-1 | integration-test | **Extend `governed_loop`**: for each denial, replay each `authorized_actions[i].command` in its stated context; plus a **concurrent-ref-advance** case (config advances between denial and replay) and a **serialization-fault** case | PASS: every offered command succeeds (or its own legitimate non-`perm.denied` gate); a concurrent-ref-advance yields a CLEAN re-denial (no panic/inconsistent state); a serialization fault still denies with exit 1 |
| T-STEER-025 | Single-source table + affordance coverage (build-gate scope) | AC-2 | build-gate | Static assert | PASS: a single `ROUTE_AUTHORITY_TABLE` symbol is referenced by both the gate and the menu module; every gated route ∈ the table; every table route has an `AFFORDANCE_MAP` entry that does not name the denied route. (Same-`cfg`/ref equality is a runtime property — covered by T-STEER-009/024, NOT this grep) |
| T-STEER-026 | Closed-catalog anti-injection (new fields) | AC-3 | build-gate | grep denial construction | PASS: no `authorized_actions`/`do_not` text derived from config values, principal-supplied data, or `format!`. (NOTE: `message`/`unmet[]` interpolate config strings — that is R15, mitigated separately, NOT claimed closed here) |
| T-STEER-027 | Best-effort additive — fail-closed at derivation AND serialization | AC-4 | integration-test | Force (a) menu-derivation failure and (b) a serialization fault on the new fields, on a denied action | PASS: action still denied with `code`/`message`/`remediation_hint` + exit 1; existing fields render independently of the new ones; FAIL if a fault drops existing fields or turns deny→allow |
| T-STEER-029 | `class` mapping is exhaustive / non-defaulted | AC-5 | build-gate | Compile + grep | PASS: the (code, resolution)→`class` mapping is a non-defaulted match (a new code/cause without classification is a build break, never silent `actor_correctable`) |

> **Counting.** 31 criteria rows cover 32 STEER ACs — T-STEER-023 covers UC-STEER-05 AC-3 **and** AC-4. Type tally: 18 integration-test · 6 api-contract · 7 build-gate = 31. Every AC has ≥1 criterion. Reconciles with [05-delta-replan.md §4](./05-delta-replan.md).
