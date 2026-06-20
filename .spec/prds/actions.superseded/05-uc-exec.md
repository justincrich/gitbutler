---
stability: FEATURE_SPEC
last_validated: 2026-06-19
prd_version: 1.0.0
functional_group: EXEC
---
# Use Cases: Check Executor (EXEC)

The executor is the **producer** of check results, and the single property that makes the whole system agent-non-forgeable lives here: **the entity being gated never produces its own passing verdict.** The executor runs a defined check against a checked-out head SHA in a **clean workspace**, **actually executes** the command/script (no stubbed success, ever), captures the exit code → typed conclusion, enforces the check's `timeout_secs`, records concurrency-safely, and injects only **minimal, executor-isolated** butler secrets (masking injected-secret values before any captured output is persisted). It runs in the **trusted daemon/CLI the human runs — NOT the agent's process or a surface the agent controls** — because whoever runs the check controls the result. It is also the principal that runs the required `on-merge-attempt` checks in the **pre-merge step** (so the read-only gate has current-head results to consume). It produces the inputs the ledger signs; it does **not** adjudicate the gate (separation of who-executes from who-adjudicates). v1 is the butler-controlled default executor only; a detached runner lease/long-poll/label protocol + autoscaling is deferred.

> **No stubbed success — the cardinal line.** A v1 executor that reports `success` without running the real check (a stub, a fake conclusion, a no-op, or a skip-that-records-green) is not a simplification — it is the exact fake-success sin this design exists to make impossible, and it would silently void every gate. The executor MUST run the real command/script and derive the conclusion from the real exit code; an executor that cannot run a check fails the check closed (`failure`/`timed_out`), never green.

> **What the v1 executor PRODUCES (conclusion vocabulary).** The exit-code executor **produces** only `{success, failure}` — plus `timed_out` when the timeout wrapper fires. The full GitHub-compatible vocabulary (`success | failure | neutral | cancelled | timed_out | skipped`) is the **stored/parsed** type the ledger can hold (so a future producer may emit `neutral`/`skipped`/`cancelled`), but the v1 default executor does **not author** `neutral`, `skipped`, or `cancelled`. The gate blocks on any non-`success` required conclusion regardless of which producer wrote it.

| ID | Title | Description |
|----|-------|-------------|
| UC-EXEC-01 | Butler-controlled executor runs the real check (executor ≠ agent) | The butler-controlled executor actually runs a defined check's command/script in the trusted daemon/CLI the human runs — not the gated agent's process — and derives the conclusion from the real run, so the entity being gated cannot produce its own passing verdict. |
| UC-EXEC-02 | Checkout at head SHA in a clean workspace + conclusion capture | The executor checks out the exact head SHA into a clean workspace, runs the check there, captures stdout/stderr + the exit code, and produces a conclusion (v1: `success`/`failure`) bound to that head SHA, so the result attests the precise code being validated. |
| UC-EXEC-03 | Minimal, executor-isolated secret injection (masked in stored output) | The executor injects only the minimal butler-held secrets a check declares it needs, scoped and executor-isolated, and masks injected-secret values before any captured output is persisted, so a check has the credentials it legitimately requires without the gated agent reading them out-of-band through the executor or out of agent-readable ledger metadata. |
| UC-EXEC-04 | Trigger-driven execution + pre-merge run (`on-commit` / `on-merge-attempt`) | The executor runs a check at the point its `trigger` declares — `on-commit` when a commit is created, and (for `on-merge-attempt`) in the trusted **pre-merge step immediately before** a governed merge — producing current-head results the ledger records and the read-only gate consumes; the gate itself never runs a check, and a stale-at-head check is surfaced via STEER ("run the check"), not auto-re-run inside the gate. |
| UC-EXEC-05 | Timeout, concurrency, and observability of check runs | The executor terminates a check exceeding its `timeout_secs` and concludes `timed_out`; runs/records multiple checks concurrently without clobbering (`(name, head_sha)`-keyed); and emits an observable run/denial-steering event with structured output consumable by both an automated orchestrator and a human at the CLI. |

---

## UC-EXEC-01: Butler-controlled executor runs the real check (executor ≠ agent)
The deepest lesson from GitHub's runner trust boundary is that *whoever runs the check controls the result* — a self-hosted runner the gated party controls makes the gate theater. This use case enforces the inverse: the check is produced by a **butler-controlled executor** that runs in the **trusted daemon/CLI the human runs**, and that is **structurally not the gated agent's process**. The executor invokes the defined `run-spec` (command or `./path` script) for real, waits for it to complete, and derives the conclusion from the real exit code (UC-DEFN-03 mapping). The gated agent has no entry point that produces a check result: there is no `but` action an agent can call that records a `success` for a check it claims passed. An executor that cannot run a check (missing interpreter, run-spec error, timeout) concludes it **closed** — `failure`/`timed_out` — never `success`. (Honesty caveat named in scope: in the personal-tenant model the agent shares the OS user with the executor, so v1 HMAC raises but does not fully close agent forgery; the executor-≠-agent structural separation is what the gate's value rests on, with OS-sandbox + Ed25519 the deferred closure.)

### Acceptance Criteria
☐ The butler-controlled executor actually runs a defined check's `run-spec` (command or `./path` script) for real, so the conclusion reflects a real run rather than a claim
☐ The executor runs in the trusted daemon/CLI the human runs — structurally **not** the gated agent's process or a surface the agent controls — so the entity being gated does not produce its own verdict (executor ≠ agent)
☐ System exposes **no** `but` action by which the gated agent can record or assert a passing check result for code it claims passed, so an agent cannot self-produce a `success`
☐ The executor derives the conclusion from the real process exit code (UC-DEFN-03), so a `success` requires the real check to have exited `0`
☐ The executor concludes a check it cannot run (missing interpreter, run-spec error, timeout) **closed** — `failure` / `timed_out`, never `success` — so an unrunnable check is never silently green
☐ System has a passing integration test against the real executor + a real exit-`0` and a real exit-`1` command asserting the executor runs each and derives `success` / `failure` from the real exit code, and asserting there is no agent-callable path that produces a `success` record

> **No dedicated UI work.** This is the executor-≠-agent structural guarantee — pure backend/daemon behavior (the executor runs in the trusted daemon/CLI, never the agent's process; no agent-callable path records a `success`). No user-facing surface of its own. The user-facing proof that the executor (not the agent) ran a check surfaces via the **producer identity** field shown in the results view (UC-EXEC-05 / UC-LEDG-04).

---

## UC-EXEC-02: Checkout at head SHA in a clean workspace + conclusion capture
A check result is only meaningful if it attests **the exact code being landed**, run in a state not poisoned by a prior run. This use case makes the executor check out the **exact head SHA** of the change into a **clean workspace**, run the check there, and capture the outputs. The clean workspace prevents cross-run contamination (a stale build artifact or a leftover file faking a pass — the "house guest leaves a mess" hazard). The executor captures stdout/stderr (and a log reference) plus the **exit code**, produces a typed **conclusion** (v1 executor: `success`/`failure`; the stored type is the full GitHub-compatible terminal vocabulary), and **binds the conclusion to the head SHA it ran against** — the `(name, head_sha, conclusion)` tuple the ledger will sign (LEDG). The executor never reports a conclusion bound to a different SHA than the one it actually checked out and ran.

### Acceptance Criteria
☐ The executor checks out the exact head SHA of the change into a clean workspace before running the check, so the result attests the precise code being validated
☐ The executor runs the check in a clean workspace state (no carryover from a prior run), so a stale artifact cannot fake a pass
☐ The executor captures the check's stdout/stderr (with a log reference) and the process exit code, so the conclusion has auditable evidence behind it
☐ The executor produces a typed terminal conclusion — in v1 `success` or `failure` (the stored type admits the full GitHub-compatible vocabulary `success | failure | neutral | cancelled | timed_out | skipped`, of which v1 authors only `success`/`failure`/`timed_out`) — distinct from a non-terminal status, so policy semantics port from GitHub
☐ The executor binds the produced conclusion to the head SHA it actually checked out and ran — never to a different SHA — yielding the `(name, head_sha, conclusion)` tuple the ledger signs (UC-LEDG-01)
☐ System has a passing integration test against the real executor + real git that checks out a specific head SHA, runs a check that inspects the worktree, and asserts the captured conclusion is `success`/`failure` derived from the real run and bound to that exact SHA

> **No dedicated UI work.** Clean-workspace checkout + stdout/stderr/exit-code capture are executor internals. The captured output becomes **evidence shown in the results view** (UC-EXEC-05 / UC-LEDG-04's expandable metadata region) and the conclusion becomes a `CheckStatusBadge` — both specified there, not here.

---

## UC-EXEC-03: Minimal, executor-isolated secret injection (masked in stored output)
Real checks sometimes need credentials (a private registry token, a service key), but a secret reachable by the gated agent defeats the trust boundary. This use case scopes secret handling to the minimum: the executor injects only the butler-held secrets a check **declares it needs**, into the check's environment, **executor-isolated** from the gated agent's process. Because the executor (not the agent) runs the check, the agent cannot read an injected secret by inspecting the executor's environment. But the executor also captures stdout/stderr, and that captured output is persisted as ledger **`metadata` that is agent-readable** (same OS user) — so any injected-secret value that surfaces in output **must be masked before storage**, and raw-output retention is **limited**. v1 is a minimal butler secret store the executor injects per check — least-privilege by default (a check gets nothing it does not declare). The honesty limit is named: injection is best-effort masking, not a hostile-code sandbox — a check with legitimate secret access could exfiltrate it at runtime (the same caveat GitHub carries), and OIDC / fine-grained scoping / ephemeral isolation is deferred.

### Acceptance Criteria
☐ The executor injects only the butler-held secrets a check declares it needs, so a check has its legitimate credentials without blanket secret exposure (least-privilege by default)
☐ The executor injects secrets into the check's environment executor-isolated from the gated agent's process, so the agent cannot read an injected secret by inspecting the executor's environment
☐ System gives a check that declares no secrets none, so an undeclared check runs without credentials
☐ System masks injected-secret values in captured stdout/stderr before persisting them as ledger `metadata`, and limits raw-output retention, so an injected secret that surfaces in output is not exposed through the agent-readable metadata column (which is outside the trust boundary, not signed-as-truth)
☐ System scopes secret handling honestly: injection is best-effort and **not** a hostile-code sandbox — a check with legitimate secret access could exfiltrate it at runtime — and this residual is documented as a named limitation, not claimed as isolation (OIDC / ephemeral isolation deferred)
☐ System has a passing integration test against the real executor that injects a declared test secret into a check (asserting the check reads it), asserts a check declaring no secret receives none, asserts the secret is not present in the gated agent's process environment, and asserts the secret value is masked in the stored ledger `metadata` rather than persisted in the clear

> **No dedicated UI work.** Secret injection + masking are executor/ledger internals. The only UI-relevant note is forward-looking: when the results view (UC-EXEC-05) renders captured output, masked secret values must render as `***` (already masked before storage, so the UI just renders stored text) — no net-new component. Declared-secret authoring uses `TagInput` in the editor (UC-DEFN-02).

---

## UC-EXEC-04: Trigger-driven execution + pre-merge run (`on-commit` / `on-merge-attempt`)
A check must run at the point its definition declares, and the read-only gate must always have a result for the **current** head to consume. This use case wires the executor to the two v1 triggers and to the pre-merge step. An **`on-commit`** check becomes eligible to run when a commit is created (producing a result bound to the new head SHA). For **`on-merge-attempt`**, the **trusted CLI/daemon runs the required checks in the pre-merge step immediately before** the governed merge — so the read-only gate then consumes current-head results. The gate itself **never runs a check**: if a required check is **stale at the merge head** (its latest result is bound to an ancestor SHA) and the pre-merge run did not refresh it, the gate **BLOCKS** with `check_stale_at_head` and a STEER redirect ("run the check"), and the executor (re-)produces the result via that explicit `but check run` redirect or the next pre-merge step — it is **not** auto-re-run from inside the gate. In all cases the **executor** produces the result and the **engine** records it (LEDG); the **gated agent never produces a result**. The executor reads which checks to run from the target-ref check config (DEFN), so the agent cannot inject an extra trivially-passing check at the feature head to satisfy the gate.

### Acceptance Criteria
☐ The executor runs an `on-commit` check when a commit is created, producing a result bound to the new commit's head SHA
☐ The trusted CLI/daemon runs the required `on-merge-attempt` checks in the **pre-merge step immediately before** a governed merge (executor, not the gate), so the read-only gate has current-head results to consume
☐ System surfaces a required check that is **stale at the merge head** (latest result bound to an ancestor SHA) by **blocking** at the gate with `check_stale_at_head` + a STEER "run the check" redirect, and (re-)produces the result via the executor on that explicit redirect or the next pre-merge run — the gate never auto-re-runs a check itself (consistent with the read-only gate, UC-GATE-04, and the SHA-reset invariant, UC-LEDG-03)
☐ The executor reads which checks to run from the **target-ref** check config (UC-DEFN-04), so an agent cannot add a trivially-passing check at the feature head to satisfy the gate
☐ System produces every check result via the executor and records it via the engine (UC-LEDG-02) — the gated agent produces no result at any trigger point or in the pre-merge step
☐ System has a passing integration test against the real executor + real git that (a) creates a commit and asserts an `on-commit` check runs and records a result bound to the new head SHA, and (b) runs the pre-merge step before a merge and asserts the `on-merge-attempt` required checks are produced at the current head by the executor (not by the gate)

> **No dedicated UI work.** Trigger wiring + the `on-merge-attempt` pre-merge step are orchestrator/daemon orchestration (DECISION A: the trusted CLI/daemon runs `but check run` immediately before the merge; the gate is read-only). The user-visible consequence — a stale-at-head check blocks the merge and STEERs "run the check" — is the **gate denial surface** specified in [UC-GATE-03](./07-uc-gate.md). An `on-commit` check's just-produced result is observable via `but check results` (UC-EXEC-05).

---

## UC-EXEC-05: Timeout, concurrency, and observability of check runs
A real executor must bound runtime, run many checks without races, and be observable — properties the gate's correctness and an operator's trust both depend on. This use case covers all three. **Timeout**: a check that exceeds its declared `timeout_secs` is terminated and concluded **`timed_out`** (a blocking conclusion at the gate), never left to hang or silently recorded green. **Concurrency**: multiple checks for a head — and checks across heads — run and record without clobbering one another, because results are keyed by `(name, head_sha)`; a concurrent producer never overwrites another check's record. **Observability**: the executor emits an observable run event (and, on a gate denial, a denial-steering event) so a check run is auditable; the structured output is consumable by **both an automated orchestrator (machine-parseable) and a human at the CLI (`but check run` / `but check results`)** — the dual audience for the same fact, so neither a script nor a person is left guessing.

### Acceptance Criteria
☐ System terminates a check that exceeds its declared `timeout_secs` and concludes it `timed_out` (a blocking conclusion), so a runaway or hanging check never blocks indefinitely and is never silently recorded as `success`
☐ The gate blocks a merge when a required check's current-head conclusion is `timed_out`, so a timed-out required check is treated as not satisfied (consistent with UC-GATE-01)
☐ System runs and records multiple checks concurrently (for a head and across heads) without clobbering, because results are keyed by `(name, head_sha)`, so a concurrent producer never overwrites another check's record
☐ System emits an observable run event when the executor runs a check (and a denial-steering event when the gate blocks), so a check run and a gate denial are auditable from logs/telemetry rather than invisible
☐ A User can read a check run's outcome — including a failed check's captured output / log reference (UC-EXEC-02/03) — and the gate's denial via structured output that is both machine-parseable (for an automated orchestrator) and human-readable at the CLI (`but check run` / `but check results`), so a denied agent can retrieve why a required check failed and the same fact serves both the agent audience and a human at the terminal
☐ System has a passing integration test against the real executor + real git that asserts (a) a check exceeding `timeout_secs` concludes `timed_out` and blocks the gate, (b) two checks for the same head record concurrently without clobbering (both `(name, head_sha)` records persist), and (c) a run event is emitted and the structured run/denial output is parseable

### UI/UX Wireframe

> **Scope calibration.** This is the **one EXEC UC with a genuine v1 user-facing surface**: the AC explicitly requires "structured output that is both machine-parseable (for an automated orchestrator) and human-readable at the CLI (`but check run` / `but check results`)." That dual-audience CLI output is **v1-present** and is a real UX design concern (it is what a human at the terminal and an orchestrator script both consume). The results dashboard (annotations, live log-streaming) is explicitly **deferred** (`01-scope.md` "Rich result UX").

**Surface:** CLI (`but check run` / `but check results`) — **v1 present**; desktop results panel — **deferred**.
**Entry point:** CLI — `but check run <name> [--head <sha>]` / `but check results [--head <sha>] [--name <name>]`; GUI (deferred) — Checks tab row → "View results", or a per-head results panel.
**Trigger:** A run is initiated by the orchestrator (pre-merge), by an `on-commit` trigger, by a STEER redirect (UC-GATE-03 "run the check"), or manually.

**Layout sketch — CLI, v1 (`but check run tests --head abc123`):**
```
$ but check run tests --head abc123
▶ tests  on-merge-attempt  ./scripts/run-tests.sh        running…
  executor: butler-default  head: abc123  timeout: 600s
✓ tests  success  exit 0  in 42.1s
  producer: butler-default@justin-mac  signed: yes  bound-to: abc123
  390 passed, 0 failed
exit 0
```
Failure / timeout variants:
```
✕ typecheck  failure  exit 2  in 8.3s
  src/checks.ts(142,5): error TS2345 ── 14 errors (truncated; see --full)
✕ lint  timed_out  killed after 120s (timeout_secs)
```
`--json` emits the same facts machine-parseable (`{name, head_sha, conclusion, producer_identity, signed, duration_secs, exit_code, log_ref}`) for the orchestrator — the **dual-audience** contract.

**Layout sketch — CLI, v1 (`but check results --head abc123`):**
```
$ but check results --head abc123
NAME          CONCLUSION  PRODUCER            DURATION  SIGNED  BOUND-TO
tests         success     butler-default      42.1s     yes     abc123
typecheck     failure     butler-default       8.3s     yes     abc123
lint          —           (no result @ abc123)
3 checks · 2 results @ abc123 · 1 missing
exit 0
```

**Layout sketch — GUI, deferred (CheckResultsPanel for a head/branch):**
```
┌──────────────────────────────────────────────────────────────────┐
│ Check results · head abc123 · branch feature/x           [Refresh]│
│ [tests ✓] [typecheck ✕] [lint ○]   2/3 green · 1 missing          │
│ ▼ typecheck   failure · butler-default · 8.3s · signed ✓ @abc123  │
│   ┌──────────────────────────────────────────────────────────┐   │
│   │ src/checks.ts(142,5): error TS2345 ── 14 errors          │   │
│   │ (captured stderr; secrets masked)        [Copy] [Open log]│   │
│   └──────────────────────────────────────────────────────────┘   │
│ ▶ tests   success · butler-default · 42.1s · signed ✓ @abc123    │
│ ▶ lint    ○ no result at this head  [▶ Run now]                   │
└──────────────────────────────────────────────────────────────────┘
Legend: ✓ success(safe)  ✕ failure(danger)  ⏱ timed_out(warning)  ○ missing(gray)
```

**Key regions:**
- Per-check row — name + `CheckStatusBadge` + producer identity + duration + "signed ✓" + bound-to SHA (provenance, all four signed-tuple fields visible).
- Expandable captured output — stdout/stderr (secrets masked), with `Copy` + `Open log` (log_ref).
- Missing/stale affordance — a missing/stale check shows ○ + `[▶ Run now]` (the executor re-produces; the gate never auto-runs, UC-GATE-04).
- Summary strip — N/M green · K missing/stale/failed.

**Interaction flow:**
1. Human/orchestrator runs `but check results --head <sha>` (or opens the panel) to see which checks are green at a head.
2. A failed check expands to show captured output (masked); a missing/stale check offers `[▶ Run now]` → `but check run`.
3. The orchestrator parses `--json` to drive STEER (run the missing/stale check, fix the failed one).

**States:** running (`spinner` + "running…"), success, failure, timed_out, missing (no result @ head), stale (result bound to ancestor SHA — flagged distinctly), unverifiable (bad signature — `lock-auth` icon), populated, empty (`EmptyStatePlaceholder` "No checks run for this head yet · [Run all]").

**Existing components to use:**
- `packages/ui/src/lib/components/cardGroup/CardGroupRoot.svelte` + `CardGroupItem.svelte` — result rows.
- `packages/ui/src/lib/components/Timestamp.svelte` / `TimeAgo.svelte` — `recorded_at`.
- `packages/ui/src/lib/components/CopyButton.svelte` — copy output.
- `packages/ui/src/lib/components/Codeblock.svelte` — captured stdout/stderr.
- `packages/ui/src/lib/components/Button.svelte` — `[Run now]`, `[Refresh]`, `[Open log]`.
- `packages/ui/src/lib/components/EmptyStatePlaceholder.svelte` — empty state.
- `apps/desktop/src/components/shared/ExpandableSection.svelte` — expandable output.
- Net-new `CheckStatusBadge.svelte` (UC-DEFN-01) — every conclusion.

**Net-new components (atomic):**
- `CheckResultRow.svelte` (molecule) — one result row: name + `CheckStatusBadge` + producer + duration + signed + bound-to + expandable output; composes `CardGroupItem` + `CheckStatusBadge` + `Timestamp` + `ExpandableSection`. Lives in `apps/desktop/src/components/checks/`. Props: `result` (name, head_oid, conclusion, producer_identity, signed, recorded_at, metadata?), `staleFromSha?`, `onRun`.
- `CheckResultsPanel.svelte` (organism) — the per-head results panel (summary strip + rows + empty + run affordance). Lives in `apps/desktop/src/components/checks/`. Props: `headOid`, `results`, `definedChecks`, `onRun`.

**UI mods to existing components:**
- `packages/ui/src/lib/components/Codeblock.svelte` — possibly MODIFY: confirm it supports a `masked`/readonly rendering and long-output truncation with "show more" for captured stderr. Reason: captured output can be large; v1 retention is limited (`01-scope.md`), so the UI should not assume full output. (Verify against source before claiming a mod — may already suffice as-is.)

**Accessibility notes:** each row's conclusion is exposed via `aria-label` on `CheckStatusBadge` (never color-only: icon + label + `aria-label`). Expandable output region is `aria-expanded` on the trigger; the log region is `role="log"` with `aria-live="off"` (not live — large). `[▶ Run now]` announces "Running <check>…" via a polite live region. The summary "2/3 green · 1 missing" is an `aria-label` on the strip.

**Edge cases / responsive / platform-specific:** CLI output respects terminal width (`--wide`/`--full` for untruncated; `[--]` truncation marker). Concurrency: concurrent results for the same head never clobber (`(name, head_sha)`-keyed) — the UI lists the latest per `(name, head)`; a concurrent re-run updates the row in place. **Captured output may contain masked secrets (`***`)** — render as-is (already masked pre-storage); never offer a "show raw" that could unmask. The deferred dashboard (annotations, live streaming) is out of v1 — the panel shows a conclusion + a log link, not live tailing. Lite (React) port required if the panel is needed there.

---

> EXEC note: of the five EXEC UCs, only **UC-EXEC-05** carries a v1 user-facing surface (the dual-audience CLI run/results output). UC-EXEC-01..04 are executor/daemon internals whose user-visible consequences surface through UC-EXEC-05 (results) and UC-GATE-03 (denial/STEER); they carry no dedicated UI.
