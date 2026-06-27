---
stability: TEST_SPEC
last_validated: 2026-06-26
prd_version: 1.2.0
---
# E2E / Integration Testing Criteria — Check Runner

Every criterion is verified against **real services** — the real `but-checks` runner, the real `check_results` store, real `git`, and the real `enforce_merge_gate`. **No mocks, no stubbed runs, no caller-supplied conclusions.** This honors the repo's testing hierarchy (integration/E2E is the primary acceptance tier; the runner that "watches it work for real" is the whole point of the feature). Type tags: `[integration-test]` (real runner + real git + real store), `[api-contract]` (denial-envelope shape / back-compat), `[build-gate]` (grep-/compile-asserted negative-space invariants), `[human-gate]` (a human-runnable full-loop demo).

Coverage rule: every AC in every UC maps to ≥1 criterion. The two load-bearing invariants — **mechanism-agnostic head-OID binding** (UC-RUN-02) and **the gate never trusts a claim / runs a check** (UC-GATE-04) — plus the **STEER denial** (UC-GATE-03) carry the most scrutiny.

## DEFN: Check Definition

### UC-DEFN-01: Named-check schema
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|-----------|--------|------|-------|-----------|
| T-DEFN-001 | A valid `.gitbutler/checks/*.toml` parses to typed definitions (name/trigger/run-spec/required/success-mapping) | AC-1,2 | [integration-test] | Real loader + a committed valid config | PASS: all fields parsed; FAIL: any field dropped or untyped |
| T-DEFN-002 | A duplicate or empty `name` is rejected | AC-3 | [integration-test] | Real loader + a dup-name config | PASS: rejected with a definition error; FAIL: loaded |
| T-DEFN-003 | A malformed/unknown-field/structurally-broken config fails closed as `config.invalid` (never an empty required-set) | AC-4,5 | [integration-test] | Real loader + a broken config at the target ref | PASS: `config.invalid`, required-set not treated as empty; FAIL: silent drop / empty set |
| T-DEFN-004 | `but check list` prints parsed definitions with exit 0 | AC-6 | [integration-test] | Real CLI + committed config | PASS: structured table + exit 0; FAIL: missing rows / non-zero |
| T-DEFN-004b | `but check define <name> --command …` writes a parseable `[[check]]` to an **uncommitted** file (exit 0); an invalid spec (dup/empty name, out-of-scope run-spec) exits non-zero **without** writing | AC-7 | [integration-test] | Real CLI + a temp repo | PASS: valid writes+parses, invalid rejected no-write; FAIL: writes an un-parseable/invalid entry |

### UC-DEFN-02: Triggers + local run-spec
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|-----------|--------|------|-------|-----------|
| T-DEFN-005 | `on-commit` and `on-merge-attempt` load; any other trigger is rejected | AC-1,2 | [integration-test] | Real loader | PASS: 2 valid load, unknown rejected; FAIL: unknown loads |
| T-DEFN-006 | A command and a `./path` script run-spec load; `uses`/Docker/JS/composite is rejected as out-of-v1-scope | AC-3,4,5 | [integration-test] | Real loader | PASS: local load, non-local rejected; FAIL: non-local accepted |

### UC-DEFN-03: required flag + exit-code mapping
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|-----------|--------|------|-------|-----------|
| T-DEFN-007 | Required-set equals the target-ref `[[required_check]]` policy; `required:false` runs but does not block | AC-1,2 | [integration-test] | Real loader + gates.toml | PASS: set matches policy; FAIL: derived from feature head |
| T-DEFN-008 | Exit `0` → `success`, non-zero → `failure`; no API authors a conclusion independent of the run | AC-3,4 | [integration-test] + [build-gate] | Real runner + exit-0/exit-1 commands | PASS: mapping holds + grep finds no conclusion-author path; FAIL: either |
| T-DEFN-009 | `but check required` prints the target-ref required-set | AC-5 | [integration-test] | Real CLI | PASS: per-branch set printed; FAIL: missing |

### UC-DEFN-04 / UC-DEFN-05: ref-pin + bootstrap-invariant
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|-----------|--------|------|-------|-----------|
| T-DEFN-010 | A feature head that deletes/weakens a required check does not weaken the gate (config read at target ref) | DEFN-04 all | [integration-test] | Real git + a self-weakening feature change | PASS: required check still enforced; FAIL: slipped |
| T-DEFN-011 | A change that weakens the required-set is itself blocked unless its head clears the currently-required checks; the weakened config governs only future changes | DEFN-05 all | [integration-test] | Real git + real gate + a required-set-deleting change | PASS: blocked until current set green, then governs forward; FAIL: lands weak |

## RUN: Check Runner

### UC-RUN-01: runner ≠ agent
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|-----------|--------|------|-------|-----------|
| T-RUN-001 | The runner runs the real run-spec in the trusted CLI/daemon and derives `success`/`failure` from the real exit code | AC-1,2,4 | [integration-test] | Real runner + exit-0 and exit-1 commands | PASS: conclusion from real exit; FAIL: stubbed/asserted |
| T-RUN-002 | No agent-callable path records a `success`; an unrunnable check concludes closed (`failure`/`timed_out`), never green | AC-3,5 | [integration-test] + [build-gate] | Real runner + a broken run-spec; grep for agent record paths | PASS: no agent record path + unrunnable→closed; FAIL: either |

### UC-RUN-02: mechanism-agnostic clean checkout at head OID  *(load-bearing)*
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|-----------|--------|------|-------|-----------|
| T-RUN-003 | The runner runs against a clean checkout of the exact head OID — proven across ≥2 branching mechanisms (a GitButler virtual-branch head and a plain detached head) | AC-1,2,3,5 | [integration-test] | Real runner + real git; a check that inspects worktree contents | PASS: conclusion reflects the OID's code in both mechanisms; FAIL: reflects the live/dirty tree or differs by mechanism |
| T-RUN-004 | A check run does not mutate or contend on the agent's shared worktree/index/locks | AC-4 | [integration-test] | Real runner + a dirty shared worktree | PASS: shared worktree byte-identical + no lock contention after a run; FAIL: mutated/contended |

### UC-RUN-03: result bound to head OID, plain store
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|-----------|--------|------|-------|-----------|
| T-RUN-005 | A result is a typed `(name, head_oid, conclusion)` row written by engine code; no public path records a caller-supplied conclusion | AC-1,2,3 | [integration-test] + [build-gate] | Real store + injection attempts | PASS: engine-written row + no caller-conclusion path; FAIL: either |
| T-RUN-006 | A `success` at H1 does not satisfy at H2 after a rebase/amend; the new head is unsatisfied until re-run (SHA-reset by construction) | AC-4,5 | [integration-test] | Real store + real git rebase | PASS: H1 green invisible at H2; FAIL: carries over |
| T-RUN-007 | Stored `metadata` (captured stdout/stderr + log ref) is agent-readable evidence **outside** the gate's trust input; the gate keys only on `(name, head_oid, conclusion)` | AC-6 | [integration-test] | Real runner + a check that prints output | PASS: metadata stored but never consulted as a verdict; FAIL: metadata treated as a trust input |

### UC-RUN-04: triggers + pre-merge run
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|-----------|--------|------|-------|-----------|
| T-RUN-008 | An `on-commit` check runs on commit and records a result bound to the new head OID | AC-1 | [integration-test] | Real runner + real commit | PASS: result @ new head; FAIL: missing/wrong OID |
| T-RUN-009 | The pre-merge step runs the `on-merge-attempt` required checks (runner, not gate) at the current head; a stale check blocks with `check_stale_at_head` + "run the check", never auto-re-run | AC-2,3,4,5 | [integration-test] | Real CLI/daemon + real gate | PASS: pre-merge produces results, gate stays read-only; FAIL: gate runs a check |

### UC-RUN-05: timeout / concurrency / observability
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|-----------|--------|------|-------|-----------|
| T-RUN-010 | A check exceeding `timeout_secs` concludes `timed_out`; two checks for one head record concurrently without clobbering | AC-1,2 | [integration-test] | Real runner + a sleeping check + 2 concurrent checks | PASS: timeout→`timed_out`, both records persist; FAIL: hang / clobber |
| T-RUN-011 | A run event is emitted and the dual-audience `--json` output is parseable | AC-3,4 | [integration-test] | Real runner | PASS: run event emitted + `--json` parses; FAIL: missing event / unparseable |

## GATE: Required-Checks Gate

### UC-GATE-01 / UC-GATE-02: the clause + composition
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|-----------|--------|------|-------|-----------|
| T-GATE-001 | Required-set read from target-ref `[[required_check]]`; merge blocks on missing/failed/stale and proceeds only on all-green @ current head | GATE-01 all | [integration-test] | Real gate + runner + git | PASS: 4 cases (missing/failed/stale block; all-green proceeds); FAIL: any |
| T-GATE-002 | The composed gate requires BOTH process (permission+review) AND quality (required checks): green-but-no-review blocks, review-but-failed-check blocks, both-satisfied proceeds | GATE-02 all | [integration-test] | Real composed `enforce_merge_gate` | PASS: all 3; FAIL: a clause overrides the other |
| T-GATE-002b | The required-checks clause is reachable from a **local `but merge`** (virtual-branch / worktree / plain-git, **no forge `review_id`**): the gate resolves source/target/head without a `ForgeReview` and blocks on a missing required check | GATE-01 + mechanism-agnostic entry | [integration-test] | Real local merge path (generalized gate entry) | PASS: local merge gated identically across mechanisms; FAIL: clause reachable only via a forge review |

### UC-GATE-03: fail-closed + STEER denial  *(load-bearing)*
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|-----------|--------|------|-------|-----------|
| T-GATE-003 | Missing/failed/stale/`config.invalid` each block with the correct miss-reason in `unmet`; stale yields `check_stale_at_head` + "run the check" with no auto-re-run | AC-1,2,3,4 | [integration-test] | Real gate, each miss condition | PASS: each blocks with correct reason; FAIL: any fail-open / auto-re-run |
| T-GATE-004 | The denial `to_envelope()` JSON carries `code:gate.check_required`/`message`/`remediation_hint` + `class` + `authorized_actions` (and `do_not` for `config.invalid`); a legacy `code`/`message`/`remediation_hint` reader sees no regression | AC-5 | [api-contract] | Real `MergeGateError` + `to_envelope()` | PASS: full envelope + back-compat; FAIL: missing field / legacy break |
| T-GATE-005 | `config.invalid` sets `class:OperatorRequired` + `do_not`; a runnable miss sets `class:ActorCorrectable` + a `but check run …` action | AC-5 | [api-contract] | Real classifier | PASS: class/do_not correct per reason; FAIL: wrong class |

> **T-GATE-004 / T-GATE-005 are LIVE now — STEER landed (governance closed).** The steering fields (`class`/`held_permissions`/`authorized_actions`/`do_not`) exist on `MergeGateError` today (`crates/but-api/src/legacy/merge_gate.rs:45-56`) and the carrier serializes to the uniform STEER envelope (the same shape `but_authz::to_envelope` emits for `Denial`, `crates/but-authz/src/denial.rs`; cf. `crates/but-api/tests/steer_envelope.rs`). So the full-envelope `class`/`authorized_actions`/`do_not` + back-compat assertions must run **for real against the real carrier** — they are no longer deferred. They must **not** be made green by stubbing the carrier (the no-stub gate), and — now that the fields exist — must **not** be downgraded to base-field-only assertions.

### UC-GATE-04: read-only consumer  *(load-bearing)*
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|-----------|--------|------|-------|-----------|
| T-GATE-006 | An agent "tests pass" claim with no current-head `success` does not satisfy; a real current-head `success` for every required check allows the merge | AC-1,5 | [integration-test] | Real gate | PASS: claim inert, real result allows; FAIL: claim satisfies |
| T-GATE-007 | No public path (CLI/`but-api`/N-API/DB) records a caller-supplied conclusion without a real run; the gate never writes `check_results` and never runs a check | AC-2,3,4 | [integration-test] + [build-gate] | Injection through every public entry point | PASS: none yields a gate-counted result; FAIL: any injection counts |

### UC-GATE-05: bootstrap-invariant + protected-flag independence
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|-----------|--------|------|-------|-----------|
| T-GATE-008 | A required-set-weakening change is blocked unless its head clears the currently-required checks | AC-1,2,3 | [integration-test] | Real gate + git | PASS: blocked, governs forward only; FAIL: lands weak |
| T-GATE-009 | A branch carrying a required check but **not** flagged `protected` still blocks on that check (no fail-open via the protected early-return); an unenforceable `[[required_check]]` fails closed as `config.invalid` | AC-4 | [integration-test] | Real gate + a non-protected branch with a required check | PASS: still gated; FAIL: merges unchecked |

## Full-loop demos (human-runnable)
| # | Criterion | Type | Pass/Fail |
|---|-----------|------|-----------|
| T-LOOP-001 | **produce → consume → deny → STEER → fix → pass → land**: define a required `tests` check; attempt a merge with it missing → gate blocks with a STEER denial naming `but check run tests`; run it (real `cargo test`/script) → `success` @ head; re-attempt → merge proceeds — all against real runner/store/git | [human-gate] | PASS: the loop completes against real services; FAIL: any stub / fake-success / fail-open |
| T-LOOP-002 | **mutation invalidates green**: pass the required check at H1, rebase to H2, attempt merge → gate blocks `check_stale_at_head` + "run the check"; re-run at H2 → proceeds | [human-gate] | PASS: stale green never lands; FAIL: H1 green satisfies H2 |

## Summary
| Type | Count |
|------|-------|
| [integration-test] | 31 |
| [api-contract] | 2 |
| [build-gate] | (shared on 4 integration rows) |
| [human-gate] | 2 |

**AC coverage.** Every AC across the 15 UCs (**88** total — DEFN 29 · RUN 30 · GATE 29) maps to ≥1 criterion. Multi-AC rows (e.g. `GATE-01 all`) intentionally fold several ACs of one UC; the load-bearing ACs are each named in a row's pass condition (RUN-05 run-event → T-RUN-011; GATE-04 deterministic-function → T-GATE-006). **One** criterion is dependency-gated on **Check Runner's own** work: T-GATE-002b exercises the mechanism-agnostic local-merge entry the GATE group must still build (R-ENTRY). T-GATE-004/005 are **no longer dependency-gated** — STEER landed (governance closed), so their full-envelope `class`/`authorized_actions`/`do_not` assertions run live against the real `MergeGateError` (merge_gate.rs:45-56).
