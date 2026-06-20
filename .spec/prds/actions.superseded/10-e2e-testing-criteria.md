---
stability: TEST_SPEC
last_validated: 2026-06-19
prd_version: 1.0.1
---
# E2E / Integration Testing Criteria — Actions (Configurable Agent-Non-Forgeable Validations for GitButler)

Per-UC test criteria. **Verification bar: a real butler executor + a real git repo + a real signer/ledger, no mocks** — the gate is only proven when the actual executor runs an actual check against a real head SHA, the actual ledger signs and stores the real conclusion, and the actual merge gate accepts or rejects a real governed merge. A green test over a mocked executor or a fabricated "success" record proves nothing — and would itself be the fake-success sin this PRD exists to make impossible. Types: `[integration-test]` (real executor/git/ledger), `[build-gate]` (grep/structural invariant asserted in CI), `[api-contract]` (the structured denial / record shape), `[human-gate]` (a human-verifiable end-to-end demonstration). The CLI surface is the **`but check`** noun (not `but actions` — see [00-overview.md](./00-overview.md) naming disambiguation; not the existing `butler_actions` macro).

**Coverage:** 19 UCs · 110 ACs · **110 per-AC criteria (1:1, 110/110 ACs covered)** + **2 additive `[human-gate]` end-to-end demonstrations** = **112 criteria total**. Per-AC breakdown: 94 integration-test · 7 api-contract · 9 build-gate. Plus 2 human-gate (the produce → consume → redirect loop and the SHA-reset-under-mutation demonstration).

> **The load-bearing tests (must be green before the slice ships).** (1) A check `success` is bound to the head SHA and a graph mutation invalidates it — **T-LEDG-016 / T-LEDG-018 / T-GATE-005**. (2) The gate blocks until every required check is green on the current head — **T-GATE-001..007**. (3) The agent cannot write a passing record (executor ≠ agent / agent-unwritable ledger / no caller-supplied conclusion) — **T-EXEC-003 / T-LEDG-007..012 / T-EXEC-022 / T-GATE-027**. (4) A failed check is non-fakeable (real executor, real exit code, no stubbed success) — **T-EXEC-001 / T-EXEC-006 / T-DEFN-015**. (5) The required-check config is self-protecting (the bootstrap-invariant) — **T-DEFN-027 / T-GATE-031**. (6) The gate never runs a check; a stale check blocks + STEERs ("run the check"), never auto-re-runs — **T-EXEC-018 / T-GATE-021 / T-GATE-025**. A failing load-bearing test means the slice is **not done**, even if every other lane is green.

---

## DEFN: Action Definition

### UC-DEFN-01: Named-check schema in committed `.gitbutler/actions/*.toml`
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|---|---|---|---|---|
| T-DEFN-001 | named checks defined in committed `.gitbutler/actions/*.toml` load | AC-1 | integration-test | real repo + config | config-as-code list parses |
| T-DEFN-002 | each entry parses to a typed definition (name/trigger/run-spec/required/success-mapping) | AC-2 | integration-test | real loader | typed definition, not raw text |
| T-DEFN-003 | `name` is the stable identity; duplicate/empty name rejected | AC-3 | integration-test | real loader | duplicate & empty name error |
| T-DEFN-004 | malformed/unknown-field definition fails closed, not silently dropped | AC-4 | api-contract | real loader | definition-error contract; no silent drop |
| T-DEFN-022 | structurally-broken config at the target ref → `config.invalid`, fails closed (never empty/satisfied required-set) | AC-5 | integration-test | broken config @ target ref | `config.invalid`; required-set never empty-satisfied |
| T-DEFN-005 | `but check list` prints parsed definitions (name/trigger/required), exit 0 | AC-6 | integration-test | real `but` CLI | structured output + exit 0 |
| T-DEFN-006 | loader integration test (valid parse + duplicate-name + unknown-field + `config.invalid` reject) | AC-7 | integration-test | real loader | valid loads; bad configs each fail closed |

### UC-DEFN-02: Triggers + local run-spec (command or `./path` script)
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|---|---|---|---|---|
| T-DEFN-007 | `on-commit` / `on-merge-attempt` triggers load | AC-1 | integration-test | real loader | both trigger values accepted |
| T-DEFN-008 | unsupported trigger (cron/dispatch/external) rejected as out-of-scope | AC-2 | api-contract | real loader | out-of-scope definition error |
| T-DEFN-009 | command and `./path` script run-specs load (local resolution) | AC-3 | integration-test | real loader | both local run-specs accepted |
| T-DEFN-010 | remote-ref / `uses` / Docker / JS / composite run-spec rejected | AC-4 | api-contract | real loader | out-of-v1-scope error, not silent accept |
| T-DEFN-011 | trigger+run-spec integration test (accept local, reject remote/unsupported) | AC-5 | integration-test | real loader | local accepted; remote/unsupported rejected |

### UC-DEFN-03: `required` flag + exit-code success mapping
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|---|---|---|---|---|
| T-DEFN-012 | `required: true` mandatory; `required: false`/default non-blocking | AC-1 | integration-test | real loader | flag parsed; default non-required |
| T-DEFN-013 | a check's `required` default parsed from the target-ref def; the gate's authoritative required-set is the `[[required_check]]` policy in `gates.toml` (see T-GATE-001) | AC-2 | integration-test | real loader | required default parsed; gate required-set = gates.toml policy |
| T-DEFN-014 | exit 0 → `success`, non-zero → `failure` (deterministic) | AC-3 | integration-test | real executor + exit-0/1 cmd | conclusion computed from exit code |
| T-DEFN-015 | no v1 path to author a `success` independent of the run | AC-4 | build-gate | source | no `${{ }}`/custom-success-predicate path; conclusion only from exit code |
| T-DEFN-016 | required-flag + success-mapping integration test | AC-5 | integration-test | real loader + real cmds | required-set correct; exit-0→success; exit-nonzero→failure |

### UC-DEFN-04: Ref-pinned definitions read at the target ref (no self-weakening)
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|---|---|---|---|---|
| T-DEFN-017 | check config read at the **target ref** when gate evaluates required checks | AC-1 | integration-test | real git | gate reads target-ref config blob |
| T-DEFN-018 | head that removes/weakens/flips a required check is ineffective for the gate | AC-2 | integration-test | feature head edits config | required-set from target ref, not head |
| T-DEFN-019 | edit inert until committed to target ref (working-tree/feature-head edit ignored) | AC-3 | integration-test | working-tree edit | edit inert until landed |
| T-DEFN-020 | config read ONLY from target-ref blob (never working tree / feature head) | AC-4 | build-gate | source | no working-tree/feature-head read in gate's config load |
| T-DEFN-021 | self-weakening integration test (head deletes required check → still enforced) | AC-5 | integration-test | real git | deleted-at-head required check still gates |

### UC-DEFN-05: Self-protecting required-check set (the bootstrap-invariant)
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|---|---|---|---|---|
| T-DEFN-023 | engine detects a diff that modifies the required-check set / a required check's definition | AC-1 | integration-test | real git + diff | policy-affecting config change recognized as such |
| T-DEFN-024 | a required-set-modifying change must itself clear the currently-required checks (read @ target ref) | AC-2 | integration-test | real gate + git | weakening change blocked until it passes the current required set |
| T-DEFN-025 | a User can land a required-check-config change only when its own head has every current required check green | AC-3 | integration-test | real gate + git | tightening/loosening the gate is itself gated by the current gate |
| T-DEFN-026 | required-check config is self-protecting (cannot self-weaken the gate that judges it) | AC-4 | build-gate | source | bootstrap-invariant wired; mirrors governance ref-pin self-escalation prevention |
| T-DEFN-027 | bootstrap-invariant integration test (delete/flip a required check → blocked unless current set green; weakened config only governs future changes) | AC-5 | integration-test | real gate + git | the load-bearing self-protection holds |

---

## EXEC: Check Executor

### UC-EXEC-01: Butler-controlled executor runs the real check (executor ≠ agent)
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|---|---|---|---|---|
| T-EXEC-001 | executor actually runs the real `run-spec` (no stubbed success) | AC-1 | integration-test | real executor + real cmd | conclusion reflects a real run |
| T-EXEC-002 | executor runs in the trusted daemon/CLI, structurally not the agent's process | AC-2 | integration-test | real executor | executor process ≠ gated-agent process |
| T-EXEC-003 | no `but` action lets the gated agent record/assert a passing result | AC-3 | build-gate | source | grep: no agent-callable result-write/assert path |
| T-EXEC-004 | conclusion derived from real process exit code | AC-4 | integration-test | real executor + exit-0/1 | success requires real exit 0 |
| T-EXEC-005 | unrunnable check concludes closed (failure/timed_out), never success | AC-5 | integration-test | missing interpreter / timeout | never silently green |
| T-EXEC-006 | executor integration test (real exit-0→success, exit-1→failure; no self-produce path) | AC-6 | integration-test | real executor | exit-driven conclusions; no agent success path |

### UC-EXEC-02: Checkout at head SHA in a clean workspace + conclusion capture
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|---|---|---|---|---|
| T-EXEC-007 | executor checks out the exact head SHA before running | AC-1 | integration-test | real git | check runs against the precise SHA |
| T-EXEC-008 | executor runs in a clean workspace (no carryover) | AC-2 | integration-test | real executor | stale artifact cannot fake a pass |
| T-EXEC-009 | executor captures stdout/stderr (+ log ref) and exit code | AC-3 | integration-test | real executor | auditable evidence captured |
| T-EXEC-010 | v1 executor produces a `success`/`failure` terminal conclusion; stored type admits full vocab (vs non-terminal status) | AC-4 | integration-test | real executor | v1 authors success/failure; terminal-vs-status held; full vocab stored-not-produced |
| T-EXEC-011 | conclusion bound to the head SHA actually checked out and run | AC-5 | integration-test | real git | `(name, head_sha, conclusion)` SHA-correct |
| T-EXEC-012 | checkout+capture integration test (conclusion success/failure, SHA-bound, run-derived) | AC-6 | integration-test | real executor + git | conclusion SHA-bound + run-derived |

### UC-EXEC-03: Minimal, executor-isolated secret injection (masked in stored output)
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|---|---|---|---|---|
| T-EXEC-013 | executor injects only the declared secrets a check needs | AC-1 | integration-test | real executor + secret store | declared secret present; least-privilege |
| T-EXEC-014 | injected secrets executor-isolated from the gated agent's process | AC-2 | integration-test | real executor | agent process cannot read injected secret |
| T-EXEC-015 | a check declaring no secrets receives none | AC-3 | integration-test | real executor | undeclared check runs credential-free |
| T-EXEC-024 | injected-secret values masked in captured stdout/stderr before stored as ledger `metadata`; raw-output retention limited | AC-4 | integration-test | real executor + secret-printing check | secret masked in stored metadata, not in the clear |
| T-EXEC-016 | isolation honesty: best-effort, not a hostile-code sandbox (documented) | AC-5 | build-gate | source/docs | residual named, not claimed as sandbox |
| T-EXEC-017 | secret-injection integration test (declared injected, undeclared absent, agent can't read, masked in metadata) | AC-6 | integration-test | real executor | injection correct + agent-isolated + masked |

### UC-EXEC-04: Trigger-driven execution + pre-merge run (`on-commit` / `on-merge-attempt`)
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|---|---|---|---|---|
| T-EXEC-018 | `on-commit` check runs at commit, result bound to new head SHA | AC-1 | integration-test | real executor + git | result bound to the new commit SHA |
| T-EXEC-019 | trusted CLI/daemon runs required `on-merge-attempt` checks in the pre-merge step (executor, not the gate) | AC-2 | integration-test | real executor + git | pre-merge run produces current-head results |
| T-EXEC-020 | a check stale at the merge head → gate blocks with `check_stale_at_head` + STEER "run the check"; executor re-produces (gate never auto-re-runs) | AC-3 | integration-test | result @ ancestor SHA | stale → block+STEER; re-produced by executor, not the gate |
| T-EXEC-021 | executor reads which checks to run from the target-ref config | AC-4 | integration-test | feature head adds a check | feature-head extra check not run by executor |
| T-EXEC-022 | every result produced by executor + recorded by engine (agent produces none, incl. pre-merge step) | AC-5 | build-gate | source | no agent result-production path at any trigger/pre-merge |
| T-EXEC-023 | trigger integration test (on-commit runs+binds; pre-merge step produces on-merge-attempt @head via executor) | AC-6 | integration-test | real executor + git | both paths fire correctly; gate runs nothing |

### UC-EXEC-05: Timeout, concurrency, and observability of check runs
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|---|---|---|---|---|
| T-EXEC-025 | a check exceeding `timeout_secs` is terminated and concluded `timed_out` (never hangs / never silent green) | AC-1 | integration-test | real executor + slow check | terminated; `timed_out` conclusion |
| T-EXEC-026 | gate blocks a merge when a required check's current-head conclusion is `timed_out` | AC-2 | integration-test | real gate + timed-out check | timed-out required check ⇒ blocked |
| T-EXEC-027 | multiple checks run/record concurrently without clobbering (`(name, head_sha)`-keyed) | AC-3 | integration-test | real executor + concurrent checks | both records persist; no overwrite |
| T-EXEC-028 | observable run event emitted on a check run (+ denial-steering event on a gate block) | AC-4 | integration-test | real executor + telemetry/log | run/denial events emitted, auditable |
| T-EXEC-029 | structured run/denial output is machine-parseable AND human-readable (`but check run`/`but check results`) | AC-5 | api-contract | real `but` CLI | dual-audience structured output (agent + human) |
| T-EXEC-030 | timeout/concurrency/observability integration test (timeout→timed_out→block; concurrent no-clobber; run event + parseable output) | AC-6 | integration-test | real executor + git | all three behaviors hold |

---

## LEDG: Check-Result Ledger

### UC-LEDG-01: Signed `(name, head_sha, conclusion, producer-identity)` records
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|---|---|---|---|---|
| T-LEDG-001 | result records the `(name, head_sha, conclusion, producer-identity)` tuple | AC-1 | integration-test | real ledger | identity = (name, head_sha); provenance recorded |
| T-LEDG-002 | producer signs the tuple with a key the agent does not hold under producer authority | AC-2 | integration-test | real signer | result authored by producer only (under that authority) |
| T-LEDG-003 | `head_sha` bound inside the signed payload (no replay to another SHA) | AC-3 | integration-test | real signer | re-point breaks signature |
| T-LEDG-004 | any hand-edit detectable via broken signature | AC-4 | integration-test | real signer | tampered record fails verification |
| T-LEDG-005 | producer identity in the signed payload (gate can require trusted executor) | AC-5 | api-contract | real ledger | signed producer identity present |
| T-LEDG-006 | signed-record integration test (sign+verify; mutate any field → verify fails) | AC-6 | integration-test | real ledger + signer | field/SHA mutation fails verification |

### UC-LEDG-02: Agent-unwritable surface, deterministic recording
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|---|---|---|---|---|
| T-LEDG-007 | results live on a surface the agent cannot write to with producer authority | AC-1 | integration-test | real ledger | no agent write to a trusted record |
| T-LEDG-008 | no `but` action / DB path / file lets the agent insert a counted result | AC-2 | build-gate | source | grep: no agent-reachable trusted-write path |
| T-LEDG-009 | recording is deterministic engine code (not an agent tool call/LLM decision) | AC-3 | integration-test | real engine | recording always happens, not agent's choice |
| T-LEDG-010 | produce→record is a guaranteed deterministic step after the executor concludes | AC-4 | integration-test | real engine | conclusion always recorded |
| T-LEDG-011 | recorder does not adjudicate (record ≠ decide merge) | AC-5 | integration-test | real engine | recorder records, never decides pass/fail |
| T-LEDG-012 | agent-unwritable integration test (agent write rejected; engine records producer output) | AC-6 | integration-test | real ledger | only producer→engine yields a trusted record |

### UC-LEDG-03: SHA-reset invariant — invalidate/recompute on every graph mutation
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|---|---|---|---|---|
| T-LEDG-013 | a result is valid for its exact head SHA only | AC-1 | integration-test | real ledger | validity is per-commit |
| T-LEDG-014 | prior-SHA results invalidated for the new head on a graph mutation | AC-2 | integration-test | rebase/edit/amend/squash/reorder | green never carries across SHAs |
| T-LEDG-015 | new head after mutation has no satisfying results until re-produced | AC-3 | integration-test | real git | cannot mutate underneath a green |
| T-LEDG-016 | ancestor-SHA `success` structurally invisible to the gate | AC-4 | integration-test | real ledger + gate | stale green never counted |
| T-LEDG-017 | required checks recomputed/marked-for-rerun on every SHA-changing mutation | AC-5 | integration-test | real git | no reuse of a prior SHA's results |
| T-LEDG-018 | SHA-reset integration test (success@H1; mutate→H2; gate sees nothing until re-pass@H2) | AC-6 | integration-test | real ledger + git | the load-bearing anti-cheat holds |

### UC-LEDG-04: Typed conclusion semantics + current-head-only read
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|---|---|---|---|---|
| T-LEDG-019 | stored conclusion vocab = `success|failure|neutral|cancelled|timed_out|skipped`; v1 executor produces only success/failure/timed_out (others parsed-not-produced) | AC-1 | integration-test | real ledger + executor | v1 authors success/failure/timed_out; others stored-only |
| T-LEDG-020 | `success` satisfies; gate blocks on any non-success required conclusion (`failure`/`timed_out`/`cancelled`) | AC-2 | integration-test | real ledger + gate | non-success terminal never satisfies (any producer) |
| T-LEDG-021 | `neutral`/`skipped` non-blocking by default | AC-3 | integration-test | real ledger + gate | skipped non-blocking; failed/timed-out blocks |
| T-LEDG-022 | current-head read returns only the current-head record (or none) | AC-4 | integration-test | real ledger | no other-SHA record returned |
| T-LEDG-023 | conclusion-semantics integration test (current-head-only read; terminal-vs-status; v1 produces success/failure/timed_out, neutral/skipped/cancelled parsed-not-produced) | AC-5 | integration-test | real ledger | current-head only; produced-vs-stored distinction held |

---

## GATE: Required-Checks Merge Gate

### UC-GATE-01: Required-checks clause — block unless all required checks green @ current head
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|---|---|---|---|---|
| T-GATE-001 | required-set determined from the target-ref `[[required_check]]` policy in `gates.toml` (mirroring governance's `[[gate]]`) | AC-1 | integration-test | real git | required checks from committed `[[required_check]]` policy |
| T-GATE-002 | satisfied only by a signed `success` bound to the current head SHA | AC-2 | integration-test | real gate + ledger | unverified/non-current never satisfies |
| T-GATE-003 | merge blocked when a required check is missing for the current head | AC-3 | integration-test | real gate | unrun required check ⇒ blocked |
| T-GATE-004 | merge blocked when a required check's current-head conclusion is failure/timed_out/cancelled | AC-4 | integration-test | real gate | failed/timed-out check ⇒ blocked |
| T-GATE-005 | merge blocked when a required check's latest success is on an ancestor SHA (stale) | AC-5 | integration-test | real gate + git | stale green ⇒ blocked (SHA-reset) |
| T-GATE-006 | `neutral`/`skipped` non-blocking by default | AC-6 | integration-test | real gate | skipped doesn't block; missing/failed does |
| T-GATE-007 | required-checks integration test (missing/failed/stale block; all-green@head proceeds) | AC-7 | integration-test | real gate + executor + git | block→satisfy; merge proceeds when green |

### UC-GATE-02: Composes with the governance merge gate
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|---|---|---|---|---|
| T-GATE-008 | clause evaluated within the same governance merge gate (one composed gate) | AC-1 | build-gate | source | required-checks clause wired into the governance gate path, not a separate bypassable check |
| T-GATE-009 | all-checks-green-but-no-required-review is blocked (quality ⊀ process) | AC-2 | integration-test | real composed gate | review clause still blocks |
| T-GATE-010 | review-present-but-required-check-failed is blocked (process ⊀ quality) | AC-3 | integration-test | real composed gate | checks clause still blocks |
| T-GATE-011 | merge allowed only when governance clauses AND required-checks both hold | AC-4 | integration-test | real composed gate | process + quality both required |
| T-GATE-012 | both governance config and check config read at the target ref when composing | AC-5 | integration-test | real git | neither clause weakenable by the change |
| T-GATE-013 | composition integration test (green-no-review blocked; review-no-check blocked; both proceeds) | AC-6 | integration-test | real composed gate + git | clauses compose, not replace |

### UC-GATE-03: Fail-closed on missing/stale/unverifiable/invalid-config + STEER redirect
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|---|---|---|---|---|
| T-GATE-014 | fails closed when a required check has no current-head result | AC-1 | integration-test | real gate | denies, not vacuously satisfied |
| T-GATE-015 | fails closed when target-ref check config is unreadable/malformed (`config.invalid`) | AC-2 | integration-test | broken config @ target ref | `config.invalid` deny, not empty-required-set |
| T-GATE-016 | fails closed when a result signature doesn't verify / is bound to another SHA | AC-3 | integration-test | forged/replayed record | unverifiable/replayed ⇒ not satisfied |
| T-GATE-025 | stale-at-head required check → `check_stale_at_head` + STEER "run the check"; gate does NOT auto-re-run | AC-4 | integration-test | result @ ancestor SHA | block+redirect; executor re-produces, gate never runs it |
| T-GATE-017 | denial `{code,message,remediation_hint}` names unmet check(s) + miss-reason + STEER next-action | AC-5 | api-contract | real gate | check_missing/check_failed/check_stale_at_head/check_unverifiable/config.invalid; STEER names corrective action |
| T-GATE-032 | required-checks clause enforced independent of the `protected` flag — a branch with a `[[required_check]]` set but not flagged `protected` still blocks (not bypassed by the protected-branch early-return, `merge_gate.rs:50-56`); an unenforceable `required_check` → `config.invalid` | AC-6 | integration-test | real gate + non-`protected` branch carrying a required check | required check still gates; clause not short-circuited; unenforceable required_check → config.invalid |
| T-GATE-019 | fail-closed+STEER integration test (each miss-reason blocks incl. stale→run-the-check; config.invalid denies; hint+STEER name the check; a required check on a non-`protected` branch still blocks) | AC-7 | integration-test | real gate | correct codes; stale no-auto-rerun; non-empty hint + STEER; non-protected-branch required check still gates |

### UC-GATE-04: Read-only deterministic consumer — never runs a check, never trusts a claim
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|---|---|---|---|---|
| T-GATE-020 | merge decision is a deterministic function of (target-ref required-set, current head SHA, signed ledger) | AC-1 | integration-test | real gate | same inputs → same decision |
| T-GATE-021 | gate never runs a check (production is the executor's, incl. the pre-merge run) | AC-2 | build-gate | source | no check-execution in the gate path |
| T-GATE-022 | gate never writes the ledger (recording is the engine's) | AC-3 | api-contract | source/runtime | gate path issues no ledger write |
| T-GATE-027 | no public path (CLI/`but-api`/N-API/DB) records a caller-supplied `Conclusion` without a real run | AC-4 | integration-test | every public entry point | inject a conclusion through each public path → none yields a gate-counted result |
| T-GATE-023 | gate never accepts an agent's textual "tests pass" as satisfying | AC-5 | integration-test | real gate | only a verified current-head signed success satisfies |
| T-GATE-024 | read-only-consumer integration test (claim/caller-conclusion w/o signed success fails; verified success proceeds) | AC-6 | integration-test | real gate + ledger | fact-check, not trust exercise; no caller-supplied conclusion counted |

### UC-GATE-05: Enforces the bootstrap-invariant (required-set change clears current required set)
| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|---|---|---|---|---|
| T-GATE-028 | gate recognizes the merging change modifies the required-check set / a required check's definition | AC-1 | integration-test | real gate + diff | policy-affecting config change gated as such |
| T-GATE-029 | a required-set-modifying change must itself have every current required check green @ head to land | AC-2 | integration-test | real gate + git | weakening change blocked until it passes the current required set |
| T-GATE-030 | a weakened required-check config governs only FUTURE changes (after it itself clears the current set) | AC-3 | integration-test | real gate + git | self-protecting; current gate always governs the weakening change |
| T-GATE-031 | bootstrap-invariant gate integration test (delete/flip a required check → blocked unless current set green; weakened config governs only subsequent changes) | AC-4 | integration-test | real gate + git | the load-bearing self-protection holds at the gate |

---

## End-to-end demonstration (the produce → consume → redirect loop)

These two `[human-gate]` criteria prove the whole composition against real services and are the human-verifiable headline of the slice. They are **additive** to the 110 per-AC criteria (each demonstrates a path already covered 1:1 above) and are gated on the load-bearing integration tests being green.

| # | Criterion | AC Ref | Type | Setup | Pass/Fail |
|---|---|---|---|---|---|
| T-E2E-001 | Full loop: an agent's change with a failing required check is blocked at the governed merge with a STEER redirect; the agent fixes the code; the executor re-runs and records a current-head signed `success`; the merge then proceeds — all checks via the real executor, no mocks | UC-GATE-01/03 + UC-EXEC-01 + UC-LEDG-01 | human-gate | real executor + real git + real composed gate, 1 implementer principal | red check blocks + redirects; green check (after fix) lands |
| T-E2E-002 | SHA-reset under mutation: a change passes its required check at head H1, then a rebase/amend produces H2; the governed merge is blocked at H2 (the H1 green does not carry over) with `check_stale_at_head` + STEER "run the check"; the executor re-runs at H2 and records a signed `success`; the merge then proceeds — proving an agent cannot mutate the change underneath a green and the gate never auto-re-runs | UC-LEDG-03 + UC-GATE-01/03 + UC-EXEC-04 | human-gate | real executor + real git + real gate | H1 green invalid at H2; stale→block+STEER; re-pass@H2 lands |

> These compose with — they do not replace — the governance reference flow (`.spec/prds/governance/` T-LOOP-006). In a combined deployment, the governed merge must satisfy both the governance clauses (permission + review at head) and the Actions required-checks clause (all required green at current head).

---

## Summary

| Type | Per-AC criteria | Additive | Total |
|---|---|---|---|
| `[integration-test]` | 94 | — | 94 |
| `[api-contract]` | 7 | — | 7 |
| `[build-gate]` | 9 | — | 9 |
| `[human-gate]` | — | 2 (T-E2E-001/002) | 2 |
| **Total** | **110** | **2** | **112** |

| Group | UCs | ACs | Per-AC Criteria |
|---|---|---|---|
| DEFN | 5 | 27 | 27 |
| EXEC | 5 | 30 | 30 |
| LEDG | 4 | 23 | 23 |
| GATE | 5 | 30 | 30 |
| **Total** | **19** | **110** | **110** |

**AC coverage: 110/110 (100%)** — every AC has exactly one `T-{PREFIX}-NNN` per-AC criterion. The 2 `[human-gate]` end-to-end criteria (T-E2E-001/002) are additive demonstrations of the composed loop over ACs already covered, bringing the criteria total to 112.

## Maintenance notes
- Adding a UC/AC: add a `T-{PREFIX}-NNN` per-AC row referencing the new AC; keep IDs stable; update the per-group and type tallies.
- A `[build-gate]` grep/structural invariant failing means the slice is **not done** even if integration lanes are green — the non-fakeability invariants (no agent result-write path **T-EXEC-003 / T-LEDG-008 / T-EXEC-022**, the gate doesn't run a check **T-GATE-021**, the gate doesn't write the ledger **T-GATE-022**, no conclusion authored without a real run **T-DEFN-015 / T-GATE-027**, config read only at the target ref **T-DEFN-020**, the bootstrap-invariant **T-DEFN-026**) are non-negotiable.
- **Never write a test asserting raw `git push` / forge auto-merge is blocked** — the required-checks clause binds the governed `but` merge only (the same accepted-leak class as governance R1/R11); such a test would encode a false guarantee.
- **Never satisfy an executor test with a stubbed/fake conclusion** — every EXEC/GATE positive-path test must run the **real** executor against a **real** exit code; a fabricated `success` record in a test is the exact fake-success sin the gate exists to prevent and voids the test's meaning.
- **Never assert the v1 executor PRODUCES `neutral`/`skipped`/`cancelled`** — the exit-code executor authors only `success`/`failure`/`timed_out`; those three are stored/parsed-not-produced (T-EXEC-010 / T-LEDG-019 / T-LEDG-023). A test asserting the v1 default executor emits them is wrong.
- **The gate never runs a check** — `on-merge-attempt` checks are run by the executor in the pre-merge step (T-EXEC-019); a stale-at-head check blocks with `check_stale_at_head` + STEER "run the check" (T-EXEC-020 / T-GATE-025), it is NOT auto-re-run inside the gate (T-GATE-021). Never write a test asserting the gate itself re-runs a check.
- The merge-gate criteria exercise the **real executor → signed ledger → deterministic gate** path; a hand-inserted ledger row that bypasses the producer signature is not a path under test (the agent-unwritable + signed properties are asserted directly in T-LEDG-006..012 and T-GATE-027, not assumed).
