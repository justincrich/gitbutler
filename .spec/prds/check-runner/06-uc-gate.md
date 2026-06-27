---
stability: FEATURE_SPEC
last_validated: 2026-06-26
prd_version: 1.2.0
functional_group: GATE
---
# Use Cases: Required-Checks Gate (GATE)

The gate is the **consumer** — the exact seam GitHub's branch protection occupies, replicated for butler: *a change cannot land until every named required check is `success` on the current head OID.* It is a **required-checks clause** that **composes** with the governance merge gate — evaluated by the same deterministic `enforce_merge_gate` code as the `merge`-authority and review clauses — and it is a **pure read-only consumer** of `check_results`: it reads results for the current head OID, requires `success` for every required check, and blocks otherwise. **The gate never runs a check.** For `on-merge-attempt` checks the trusted CLI/daemon runs them in the **pre-merge step** (the runner, UC-RUN-04); the gate then consumes current-head results. It **fails closed** on a required check that is missing, stale (a result bound to a non-current head OID), failed, or whose target-ref config is unreadable, emits the governance-consistent denial + **STEER** fields so the orchestrator can redirect the agent, and enforces the **bootstrap-invariant**. It never runs a check, never trusts an agent's "tests pass," and never writes `check_results`. **Identity precondition (new — governance IDENT).** The clause composes into `enforce_merge_gate`, which now resolves the acting principal through the runtime registry (`resolve_principal_with_runtime_registry`, `crates/but-api/src/legacy/merge_gate.rs:82` → `but_authz::resolve_principal_with_registry`): the resolver chain is (1) the runtime registry, (2) the `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` env handle, else (3) `Denial::unregistered` (`perm.denied`). So any process that reaches the required-checks clause must be a **registered principal** (via `but agent register`) or be denied before the clause is consulted — a precondition this PRD predates.

> **The exact seam.** *A change may merge into the target iff, for every required check name in the target-ref `[[required_check]]` policy, there exists a `success` bound to the current head OID.* Missing ⇒ blocked; `failure`/`timed_out`/`cancelled` ⇒ blocked; stale (a result exists for the check but not at the current head OID) ⇒ blocked (+ STEER "run the check", never auto-re-run); malformed target-ref config ⇒ `config.invalid` blocked. Identity = `(name, head_oid)`; gate = all required names `success` on the current head. This binds the **governed** merge only (raw `git push` / forge auto-merge are the same accepted-leak class as governance R11/R14).

| ID | Title | Description |
|----|-------|-------------|
| UC-GATE-01 | Required-checks clause — block unless all required green @ current head | At the governed merge boundary, the gate blocks unless every required check (from the target-ref `[[required_check]]` policy) has a `success` bound to the current head OID; a missing, failed, or stale required check blocks the merge. |
| UC-GATE-02 | Composes with the governance merge gate | The required-checks clause is evaluated as part of the same deterministic `enforce_merge_gate`, alongside the `merge`-authority and review clauses, so a merge lands only when process (permission + review) **and** quality (required checks) are both satisfied. |
| UC-GATE-03 | Fail-closed on missing/stale/failed/invalid-config + STEER redirect | The gate fails closed on a required check that is missing, stale, failed, or whose target-ref config is unreadable (`config.invalid`), and emits the governance-consistent `{code, message, remediation_hint}` + STEER fields (`class`/`authorized_actions`/`do_not`) naming the unmet check + miss-reason + corrective action, so the orchestrator can redirect the agent. |
| UC-GATE-04 | Read-only deterministic consumer — never runs a check, never trusts a claim | The gate is a pure read-only deterministic consumer: it reads `check_results` for the current head, requires `success` for every required check, and proceeds otherwise blocks — it never runs a check, never writes `check_results`, never accepts a caller-supplied conclusion, and never trusts an agent's textual assertion. |
| UC-GATE-05 | Enforces the bootstrap-invariant, independent of the `protected` flag | A change that edits the required-set or a required check's definition must itself clear the currently-required checks at the target ref; and the required-checks clause runs whenever a `[[required_check]]` set exists, **independent of the branch's `protected` flag**, so it is never short-circuited into fail-open by the protected-branch early-return. |

---

## UC-GATE-01: Required-checks clause — block unless all required green @ current head
This use case is the product claim made enforceable: a change cannot land until its committed checks **provably ran and passed**. At the governed `but` merge boundary, the gate reads the **target-ref** `[[required_check]]` policy in `gates.toml` (the same `enforce_merge_gate` path that already reads the `[[gate]]` review requirement) to determine which defined checks the branch requires, and for **each** required check name it requires a **`success` bound to the current head OID** in `check_results` (UC-RUN-03). If every required check is a current-head `success`, the clause is satisfied; if any is **missing** (never produced for this head), **failed** (`failure`/`timed_out`/`cancelled`), or **stale** (a `success` exists for it but bound to a non-current head OID), the clause **blocks**. `neutral`/`skipped` are non-blocking by default.

### Acceptance Criteria
☐ At the governed merge boundary, the gate determines the required-set from the **target-ref** `[[required_check]]` policy in `gates.toml`, so which checks are required is read from committed config, not the feature head
☐ The gate counts a required check as satisfied only when a **`success` bound to the current head OID** exists for it in `check_results`, so a non-current or non-success result never satisfies
☐ The gate blocks the merge when any required check is **missing** for the current head OID (never produced), so an unrun required check is treated as not satisfied
☐ The gate blocks the merge when any required check's current-head conclusion is `failure` / `timed_out` / `cancelled`, so a failed or timed-out check cannot be ignored
☐ The gate blocks the merge when a required check's latest `success` is bound to a **non-current** head OID (stale — e.g. a green produced before a graph mutation), so a pre-mutation green does not let the merge land (the pure evaluator distinguishes `check_missing` = no result for the check from `check_stale_at_head` = a result exists but not at the current head, without computing git ancestry)
☐ System has a passing integration test against the real gate + real runner + real git that attempts a governed merge with a required check missing (asserts blocked), failed (asserts blocked), stale (asserts blocked), and with a current-head `success` for every required check (asserts the merge proceeds)

---

## UC-GATE-02: Composes with the governance merge gate
Check Runner is the quality half of "governance + quality," so the required-checks clause must **compose** with the governance merge gate, not duplicate or replace it. This use case wires the clause into the **same** `enforce_merge_gate` that already enforces `merge`-authority and the review requirement — one more clause evaluated by the same on-action code at the governed `but` merge boundary. The result is a single merge decision that lands a change only when **process** (the principal holds `merge` and the configured reviews exist at head) **and** **quality** (every required check is a current-head `success`) are both satisfied. Neither weakens the other; both read their config at the **target ref**.

### Acceptance Criteria
☐ System evaluates the required-checks clause as part of the same deterministic `enforce_merge_gate` that enforces `merge`-authority and the review requirement, so it is one composed gate rather than a separate, bypassable check
☐ The composed gate blocks a merge that satisfies every required check but lacks the governance review requirement, so quality does not override process
☐ The composed gate blocks a merge that satisfies the governance review requirement but has a missing or failed required check, so process does not override quality
☐ The composed gate allows a merge only when both the governance clauses (`merge`-authority + review at head) and the required-checks clause (all required `success` at current head) are satisfied
☐ System reads both the governance config (`gates.toml` + the identity config, now `.gitbutler/agents.toml`; legacy `.gitbutler/permissions.toml` is read only as a one-release fallback) and the check config (`.gitbutler/checks/*.toml`) at the **target ref**, so neither clause can be weakened by the change being judged
☐ System has a passing integration test against the real composed gate + real git asserting (a) all-checks-green-but-no-review is blocked, (b) review-present-but-required-check-failed is blocked, and (c) both-satisfied proceeds — proving the clauses compose rather than replace

---

## UC-GATE-03: Fail-closed on missing/stale/failed/invalid-config + STEER redirect
A quality gate is only safe if the **absence, staleness, or failure of a result denies, never allows** — and only useful if the denial tells the agent what to do next. This use case makes both explicit. The gate **fails closed**: a required check with no current-head result, an unreadable/malformed target-ref config (`config.invalid`, mirroring governance), or a result bound to a **different OID** (stale) is treated as **not satisfied** — the gate denies rather than vacuously passing. On any miss, the gate returns the governance-consistent denial via the **`MergeGateError` carrier** — `{code: "gate.check_required", message, remediation_hint, unmet}` plus the STEER steering fields below. **STEER has landed (governance is closed).** The steering fields (`class` / `held_permissions` / `authorized_actions` / `do_not`) already exist on `MergeGateError` (`crates/but-api/src/legacy/merge_gate.rs:45-56`; landed in commit `353bbcdc1a`, an ancestor of `master`), and the carrier serializes to the uniform STEER envelope — the same shape `but_authz::to_envelope` emits for `Denial` (`crates/but-authz/src/denial.rs`), proven in `crates/but-api/tests/steer_envelope.rs`. The GATE group therefore no longer sequences after STEER; the check clause populates the steering fields directly. The steering fields:

- **`class`** — `ActorCorrectable` for a runnable miss (the agent can fix it by running/fixing the check), `OperatorRequired` for `config.invalid` (a human must fix the committed config).
- **`unmet`** — the list of unmet checks with their miss-reason (`check_missing` / `check_failed` / `check_stale_at_head` / `config.invalid`).
- **`authorized_actions`** — e.g. `{command: "but check run <name> --head <oid>", effect: "produce the missing/stale check result"}` for a missing/stale check; the failing check points the agent at its captured output.
- **`do_not`** — set where a retry is futile (e.g. `config.invalid`: "do not retry — fix the committed check config at the target ref").

A **stale-at-head** check specifically yields `check_stale_at_head` + a "run the check" `authorized_action` — the gate does **not** auto-re-run it (it is a read-only consumer; the runner re-produces it). Output is **dual-audience**: human-readable CLI text + a `--json` envelope (via `to_envelope()`) the orchestrator parses to STEER the agent.

### Acceptance Criteria
☐ The gate fails closed (blocks) when a required check has no result bound to the current head OID, returning a denial rather than vacuously satisfying the requirement
☐ The gate fails closed when the target-ref check config is unreadable or malformed, denying with `config.invalid` rather than treating the required-set as empty/satisfied (consistent with governance fail-closed)
☐ The gate fails closed when a required check's result is bound to a different OID (stale), treating a stale result as not satisfied, so a green from before a graph mutation never satisfies
☐ The gate returns, for a **stale-at-head** required check, the `check_stale_at_head` miss-reason with an `authorized_action` "run the check" and does **not** auto-re-run the check itself, so a stale result blocks-and-redirects rather than silently re-running inside the gate
☐ The gate returns the governance-consistent denial on the `MergeGateError` carrier — `code: "gate.check_required"`, `message`, `remediation_hint`, `unmet` (`Vec<String>` of `"<name>: <miss-reason>"` — the existing carrier field), and the STEER fields `class` (`ActorCorrectable` for a runnable miss, `OperatorRequired` for `config.invalid`), `authorized_actions` (naming the `but check run …` next action), and `do_not` where a retry is futile — serialized to the uniform STEER envelope (these fields exist on `MergeGateError` now — STEER landed — so this is a live assertion, not a deferred one) so the orchestrator can redirect the agent
☐ System has a passing integration test against the real gate that asserts (a) missing/failed/stale/`config.invalid` each block with the correct miss-reason in `unmet`, (b) a stale check yields `check_stale_at_head` + a "run the check" `authorized_action` with no auto-re-run, (c) the denial's `to_envelope()` JSON carries `code`/`message`/`remediation_hint` + `class`/`authorized_actions` and (for `config.invalid`) `do_not`, and (d) a legacy consumer reading only `code`/`message`/`remediation_hint` sees no regression

### CLI denial surface (v1 present)
The v1 surface for "the merge is blocked by a required check" is the **CLI merge denial** (`but merge …` → structured denial). A desktop merge-dialog summary is deferred (it composes inside governance's deferred merge-gate UI).

**Human-readable (`but merge feature/x into main` blocked):**
```
$ but merge feature/x into main
✕ gate.check_required — merge blocked: 2 required checks not satisfied at head abc123
   • typecheck   check_failed         fix the failing check, then `but check run typecheck`
   • lint        check_missing        run: `but check run lint --head abc123`
   • sig-check   check_stale_at_head  run: `but check run sig-check --head abc123` (last result at 999eee, not the current head)
   remediation: run/fix the named checks, then re-attempt `but merge feature/x into main`
exit 1
```
**`--json` (the orchestrator's STEER input, via `to_envelope()`):**
```json
{ "code": "gate.check_required", "message": "merge blocked: 2 required checks not satisfied at head abc123",
  "remediation_hint": "run/fix the named checks, then re-attempt the merge",
  "class": "ActorCorrectable",
  "unmet": [ "typecheck: check_failed", "lint: check_missing",
             "sig-check: check_stale_at_head (last at 999eee)" ],
  "authorized_actions": [ {"command":"but check run lint --head abc123","effect":"produce the missing check result"},
                          {"command":"but check run sig-check --head abc123","effect":"refresh the stale check at head"} ] }
```
A `config.invalid` denial instead sets `class: "OperatorRequired"` and `do_not: "do not retry — fix the committed check config at the target ref"`.

### UI/UX Wireframe — required-checks gate summary (v1.1 *iff* a governance merge dialog exists)
> **Surface:** the CLI `but merge` denial (above) is the **v1.0** surface. A desktop **`RequiredChecksGateSummary`** composes inside a merge dialog — but `MergeButton` is a plain dropdown today and **no merge dialog exists**, so this surface is **blocked on governance's own (still-deferred) merge-gate UI**, not on this design. It can render `code/message/remediation_hint/unmet` now; the `class/authorized_actions/do_not` enrichment is **available now (STEER landed)** — the fields exist on `MergeGateError` (merge_gate.rs:45-56), so the only thing still gating this surface is the absent merge dialog. **Sequential dependencies: (1) Surface 1/UC-RUN-05 must ship first** — `CheckResultRow` and `CheckConclusionBadge` are defined there and reused here; **(2) the governance merge dialog must exist** before this surface has a host (STEER is no longer a dependency).
```
┌─ Required Checks — Quality Gate ───────────────────────────────────┐
│ [!] 2 required checks not satisfied at abc123f   (InfoMessage danger)│
│  ✓ cargo-test  passed  @abc123f   required                         │
│  ✗ pnpm-check  failed  @abc123f   required          [▶ Run now]     │
│  ○ lint        missing @abc123f   required          [▶ Run now]     │
│  remediation: run/fix the named checks, then re-attempt the merge   │
│ [Cancel]                                    [Merge] ← disabled      │
└─────────────────────────────────────────────────────────────────────┘
```
Row badge states rendered via `CheckConclusionBadge` with the `state` prop: `success`, `failure`, `timed_out`, `missing`, `stale` (every conclusion the gate blocks on) — consistent with the UC-RUN-05 re-modeled badge (§3/§5). **Component map** (full table → [`08-technical-requirements/10-frontend-ui.md`](./08-technical-requirements/10-frontend-ui.md) §3, §6): **reuse** `InfoMessage` (danger/success), `MergeButton` (pass `disabled` + STEER-hint tooltip), `Button`; **net-new** `RequiredChecksGateSummary` + **reuse `CheckResultRow` / `CheckConclusionBadge`** (produced by Surface 1/UC-RUN-05 — hard dependency). The disabled `[Merge]` is UX only — the server gate denies regardless. **Lite (React):** port, not a share (deferred).

---

## UC-GATE-04: Read-only deterministic consumer — never runs a check, never trusts a claim
The gate's integrity rests on it being a **pure read-only deterministic adjudicator**, structurally separate from production. This use case fixes that boundary. The gate **reads and decides** — it loads the target-ref required-set, queries `check_results` for current-head results, and decides merge/block as a deterministic function of (target-ref config, current head OID, recorded results). It **never runs a check** (production is the runner's job, UC-RUN-01/04; for `on-merge-attempt` the runner runs them in the pre-merge step), **never writes `check_results`** (recording is the engine's job, UC-RUN-03), **never accepts a caller-supplied conclusion** as a substitute for a real run, and **never trusts an agent's textual assertion** that a check passed — provenance over claims, the same no-stub / real-services discipline the repo enforces. When every required check is a current-head `success`, the deterministic decision is to allow the governed merge to proceed (composing with the governance clauses).

### Acceptance Criteria
☐ The gate decides merge/block as a deterministic function of (target-ref required-set, current head OID, recorded results), so the same inputs always yield the same decision
☐ The gate never runs a check itself — production is the runner's responsibility (the `on-merge-attempt` pre-merge run is the runner's, UC-RUN-04) — so the read-only adjudicator and the producer remain separate
☐ The gate never writes `check_results` — recording is the engine's responsibility (UC-RUN-03) — so the consumer cannot manufacture the result it reads
☐ System exposes no public path (CLI, `but-api`, N-API, or DB) by which a caller-supplied conclusion is recorded as a satisfying result without a real runner run, so a result is only ever a real run's output
☐ The gate never accepts an agent's textual claim that a check passed as satisfying a required check — only a current-head `success` in `check_results` — so the gate is a fact-check, not a trust exercise
☐ System has a passing integration test against the real gate + real `check_results` asserting (a) an agent-supplied "tests pass" claim with no current-head `success` does not satisfy while a real current-head `success` for every required check allows the merge, and (b) no public path records a caller-supplied conclusion without a real run — exercised by attempting injection through every public entry point and asserting none yields a gate-counted result

---

## UC-GATE-05: Enforces the bootstrap-invariant, independent of the `protected` flag
This use case closes two related fail-open holes. First, the **bootstrap-invariant**: a change whose diff **adds, removes, weakens, flips `required` on, or edits the `run-spec` of a required check** must clear the **currently-required** checks (read at the target ref) before it can land — so an agent cannot land "delete the test check" to weaken the gate for everything after, because that very change must first pass the checks it is trying to remove (the same self-escalation-prevention shape governance uses for its ref-pin). Second, the **protected-flag fail-open**: `enforce_merge_gate` returns early for a branch not flagged `protected`; the required-checks clause must be consulted **before** that early-return and must run whenever a `[[required_check]]` set exists — **independent of the `protected` flag** — so a branch carrying required checks but not in the governance *protected* set is still gated, and a `[[required_check]]` that cannot otherwise be enforced fails closed as `config.invalid` rather than being silently dropped.

### Acceptance Criteria
☐ The gate recognizes when the change being merged modifies the required-set or a required check's definition (adds/removes a required check, flips `required`, or edits a required check's `run-spec`), so a policy-affecting config change is gated as such
☐ The gate requires such a change to itself have every **currently-required** check `success` at its head (read at the target ref) before it can land, so a weakening change cannot escape the checks it must itself pass (the bootstrap-invariant)
☐ System makes a weakened configuration take effect only for **future** changes, once the weakening change has itself cleared the currently-required set
☐ System enforces the required-checks clause whenever a `[[required_check]]` set exists at the target ref **independent of the branch's `protected` flag**, so the gate is never short-circuited into fail-open by the protected-branch early-return, and a `[[required_check]]` that cannot be enforced fails closed as `config.invalid`
☐ System has a passing integration test against the real gate + real git asserting (a) a change that deletes (or flips `required: false` on) a currently-required check is blocked unless its own head satisfies the currently-required checks, and (b) a branch carrying a required check but **not** flagged `protected` still blocks on that check (the clause is not bypassed by the protected-branch early-return)
