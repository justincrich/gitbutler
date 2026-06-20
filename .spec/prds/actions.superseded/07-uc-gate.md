---
stability: FEATURE_SPEC
last_validated: 2026-06-19
prd_version: 1.0.1
functional_group: GATE
---
# Use Cases: Required-Checks Merge Gate (GATE)

The gate is the **consumer** — the exact seam GitHub's branch protection occupies, replicated for butler: *a change cannot land until every named required check reports a passing conclusion on the current head SHA.* It is a **required-checks clause** that **composes** with the governance merge gate — evaluated by the same deterministic on-action code as the `merge`-authority and review clauses, at the governed `but` merge boundary — and it is a **pure read-only consumer** of the signed ledger: it reads results for the current head SHA, verifies signature + SHA-binding + `success` for every required check, and blocks otherwise. **The gate never runs a check.** For `on-merge-attempt` checks the trusted CLI/daemon runs them in the **pre-merge step immediately before** the merge (the executor, UC-EXEC-04); the gate then consumes the current-head results. It **fails closed** on a required check that is missing, unreadable, stale (ancestor-SHA), unverifiable, or non-success, denies a malformed target-ref config as `config.invalid`, and emits the governance-consistent `{code, message, remediation_hint}` + **STEER** fields so the orchestrator can redirect the agent to the right check (a stale-at-head check → "run the check"). It also enforces the **bootstrap-invariant** — a change that edits the required-check set must itself clear the currently-required checks. It never runs a check, never trusts an agent's "tests pass," and never writes the ledger.

> **The exact seam (carried throughout).** *A change may merge into the target iff, for every required check name in the target-ref check config, there exists a signed `success` bound to the current head SHA.* Missing ⇒ blocked; `failure`/`timed_out`/`cancelled` ⇒ blocked; stale (ancestor-SHA) ⇒ blocked (+ STEER "run the check", never auto-re-run inside the gate); unverifiable signature ⇒ blocked; malformed target-ref config ⇒ `config.invalid` blocked; `neutral`/`skipped` ⇒ non-blocking by default. Identity = `(name, head_sha)`; gate = all required names green on the current head. This binds the **governed** merge only (raw `git push` / forge auto-merge are the same accepted-leak class as governance R1/R11 — see [01-scope.md § Known Limitations](./01-scope.md#known-limitations)).

| ID | Title | Description |
|----|-------|-------------|
| UC-GATE-01 | Required-checks clause — block unless all required checks green @ current head | At the governed merge boundary, the gate blocks a merge unless every required check (from the target-ref check config) has a signed `success` bound to the current head SHA; a missing, failed (`failure`/`timed_out`/`cancelled`), or stale required check blocks the merge. |
| UC-GATE-02 | Composes with the governance merge gate | The required-checks clause is evaluated as part of the same deterministic governance merge gate, alongside the `merge`-authority and review clauses, so a merge lands only when process (permission + review) **and** quality (required checks) are both satisfied — neither clause replaces the other. |
| UC-GATE-03 | Fail-closed on missing/stale/unverifiable/invalid-config + STEER redirect | The gate fails closed on a required check that is missing, unreadable, stale, or unverifiable, and on a malformed target-ref config (`config.invalid`); on any miss it returns the governance-consistent `{code, message, remediation_hint}` naming the unmet check(s) and the miss-reason plus STEER fields, so the orchestrator can redirect the agent to run/fix the right check. |
| UC-GATE-04 | Read-only deterministic consumer — never runs a check, never trusts a claim | The gate is a pure read-only deterministic consumer: it reads and adjudicates the signed ledger for the current head, verifies signature + SHA-binding + conclusion, and proceeds when every required check is satisfied — it never runs a check, never writes the ledger, never accepts a caller-supplied conclusion, and never trusts an agent's textual assertion. |
| UC-GATE-05 | Enforces the bootstrap-invariant (required-set change clears current required set) | The gate enforces that a change which adds, removes, weakens, or flips `required` on a required check — or edits a required check's `run-spec` — must itself clear the **currently-required** checks at the target ref before it can land, so the required-check configuration is self-protecting (the same self-escalation-prevention shape governance uses for its ref-pin). |

---

## UC-GATE-01: Required-checks clause — block unless all required checks green @ current head
This use case is the product claim made enforceable: agent-written code cannot land until it has **provably run the committed checks and they passed**. At the governed `but` merge boundary, the gate evaluates a **required-checks clause**: for the change being merged into the target, it reads the **target-ref** `[[required_check]]` policy in `gates.toml` (the authoritative required-set, read by the same `enforce_merge_gate` path that already reads the `[[gate]]` review requirement) to determine which **defined** checks (`.gitbutler/actions/*.toml`) the branch requires, and for **each** required check name it requires a **signed `success` bound to the current head SHA** in the ledger (UC-LEDG-01/04). If every required check is a verified current-head `success`, the clause is satisfied; if any required check is **missing** (never produced for this head), **failed** (`failure`/`timed_out`/`cancelled`), or **stale** (its latest `success` is bound to an ancestor SHA, UC-LEDG-03), the clause **blocks** the merge. The clause counts `neutral`/`skipped` as non-blocking by default. This is GitHub's "all required named checks green on the head SHA," replicated for the change being landed.

### Acceptance Criteria
☐ At the governed merge boundary, the gate determines the required-set from the **target-ref** `[[required_check]]` policy in `gates.toml` (mirroring governance's `[[gate]]` review requirement), naming which defined checks must be green, so which checks are required is read from committed config, not the feature head
☐ The gate counts a required check as satisfied only when a **signed `success` bound to the current head SHA** exists for it in the ledger, so an unverified or non-current result never satisfies
☐ The gate blocks the merge when any required check is **missing** for the current head SHA (never produced), so an unrun required check is treated as not satisfied
☐ The gate blocks the merge when any required check's current-head conclusion is `failure` / `timed_out` / `cancelled`, so a failed or timed-out check cannot be ignored
☐ The gate blocks the merge when a required check's latest `success` is bound to an **ancestor** SHA (stale), so a green from before a graph mutation does not let the merge land (consistent with UC-LEDG-03)
☐ The gate counts `neutral` / `skipped` as non-blocking by default, so a skipped check does not block while a missing/failed required check does
☐ System has a passing integration test against the real gate + real executor + real git that attempts a governed merge with a required check missing (asserts blocked), with the check failed (asserts blocked), with a stale ancestor-SHA green (asserts blocked), and with a current-head signed `success` for every required check (asserts the merge proceeds)

### UI/UX Wireframe

> **Scope calibration.** The v1 surface for "the merge is blocked by a required check" is the **CLI merge denial** (`but merge …` → structured denial) — **v1 present**. A merge dialog showing required-checks state in the desktop app is **deferred** (governance's merge-gate UI itself is still being built in governance sprint-06b; actions composes *under* that surface).

**Surface:** CLI (`but merge` denial) — **v1 present**; desktop merge dialog / branch-gate summary — **deferred**.
**Entry point:** CLI — the governed `but merge` verb; GUI (deferred) — the merge dialog (governance) gains a required-checks summary.
**Trigger:** A merge is attempted; the gate evaluates the required-checks clause; if any required check is not a current-head signed `success`, the merge is blocked.

**Layout sketch — CLI, v1 (`but merge feature/x into main` blocked):**
```
$ but merge feature/x into main
✕ merge blocked — required checks not satisfied at head abc123
   gate.check_required
   tests:        success ✓
   typecheck:    failure ✕      (check_failed)
   lint:         (no result)    (check_missing)
   sig-check:    stale @ 999eee (check_stale_at_head)  → run: `but check run sig-check --head abc123`
   remediation: run/fix the failing or missing checks, then re-attempt the merge
exit 1 (denied)
```
When all required checks are green, the merge proceeds (no extra output beyond the normal merge result).

**Layout sketch — GUI, deferred (Required-Checks summary inside the merge dialog; composes with governance's review/permission summary):**
```
┌──────────────────────────────────────────────────────────────────┐
│ Merge feature/x → main                                  [Cancel] │
│ ─ Process (governance) ───────────────────────────────────────── │
│   Permission: merge ✓     Review: 1/1 approved ✓                  │
│ ─ Quality (Actions) ──────────────────────────────────────────── │
│   Required checks @ abc123      2/4 satisfied                     │
│   [tests ✓] [typecheck ✕] [lint ○] [sig-check ⏱stale]            │
│   ✕ 2 required checks not satisfied — merge blocked.             │
│     [▶ Run missing/stale]  [View results]                        │
│                                          [Merge] (disabled)      │
└──────────────────────────────────────────────────────────────────┘
```

**Key regions:**
- Required-checks summary — one `CheckStatusBadge` per required check at the current head, with the miss-reason (`check_missing`/`check_failed`/`check_stale_at_head`/`check_unverifiable`).
- `[▶ Run missing/stale]` — runs the executor for the missing/stale checks (the gate never auto-runs them; UC-GATE-04); after a successful re-run, the summary re-evaluates.
- `[Merge]` — **disabled** until every required check is a current-head signed `success` (the `[Merge]` button's disabled state *is* the gate's block made visible; enforcement is still server-side — the button is UX convenience).

**Interaction flow:**
1. User/orchestrator attempts merge (CLI `but merge` / GUI `[Merge]`).
2. Gate evaluates the required-checks clause; the denial (CLI) or disabled-button + summary (GUI) names each unmet check + miss-reason.
3. User runs/fixes the checks (`but check run` / `[▶ Run missing/stale]`); the summary updates; merge becomes available once all required checks are green at head.

**States:** all-green (merge enabled), partial (blocked, summary shows miss-reasons), all-missing (blocked, "run all"), config.invalid (blocked, danger `InfoMessage` "malformed check config at target ref — merge blocked" — fail-closed).

**Existing components to use:**
- `packages/ui/src/lib/components/InfoMessage.svelte` — the blocked banner (danger) + config.invalid.
- `packages/ui/src/lib/components/Button.svelte` — `[Merge]` (disabled), `[▶ Run missing/stale]`, `[View results]`.
- Net-new `CheckStatusBadge.svelte` (UC-DEFN-01) — per-check status.
- Net-new `RequiredChecksGateSummary` (below).

**Net-new components (atomic):**
- `RequiredCheckGateSummary.svelte` (molecule) — the "Required checks @ <head> · N/M satisfied" strip + per-check badges + run/results affordances; composes `CheckStatusBadge` + `Button`. Lives in `apps/desktop/src/components/checks/`. Props: `headOid`, `requiredResults` (name→conclusion/miss-reason), `onRun(names)`, `onViewResults`.

**UI mods to existing components:**
- Governance's merge dialog (deferred, sprint-06b) — MODIFY: add the Quality (Actions) section + the `RequiredCheckGateSummary`. Reason: actions composes *inside* governance's merge-gate UI (process + quality), exactly as the clauses compose server-side.

**Accessibility notes:** the blocked banner is `role="alert"`. The `[Merge]` disabled state exposes `aria-disabled="true"` + `aria-label="Merge — disabled: 2 required checks not satisfied"`. The summary strip is `aria-label="Required checks: 2 of 4 satisfied"`. Each badge's miss-reason is in its `aria-label`.

**Edge cases / responsive / platform-specific:** `[Merge]` being disabled is UX only — a determined renderer cannot bypass the server gate (the denial is returned by `enforce_merge_gate` regardless). The summary must re-evaluate after a re-run without a full reload (poll the current-head ledger). CLI denial exit code is non-zero and `--json` for the orchestrator (dual-audience). Lite (React) port required for the dialog section.

---

## UC-GATE-02: Composes with the governance merge gate
Actions is the verification half of "governance + accountability," so the required-checks clause must **compose** with the governance merge gate, not duplicate or replace it. This use case wires the clause into the **same deterministic governance merge gate** that already enforces `merge`-authority and the review requirement — it is one more clause evaluated by the same on-action code at the governed `but` merge boundary. The result is a single merge decision that lands a change only when **process** (the principal holds `merge` and the configured reviews exist at head) **and** **quality** (every required check is a current-head signed `success`) are both satisfied. Neither side weakens the other: a change with all checks green but no required review is still blocked by governance; a change with the required reviews but a failed required check is blocked by the Actions clause. Both clauses read their config at the **target ref**, so neither can be weakened by the change being judged.

### Acceptance Criteria
☐ System evaluates the required-checks clause as part of the same deterministic governance merge gate that enforces `merge`-authority and the review requirement, so it is one composed gate rather than a separate, bypassable check
☐ The composed gate blocks a merge that satisfies every required check but lacks the governance review requirement, so quality does not override process
☐ The composed gate blocks a merge that satisfies the governance review requirement but has a missing or failed required check, so process does not override quality
☐ The composed gate allows a merge only when both the governance clauses (`merge`-authority + review at head) and the required-checks clause (all required green at current head) are satisfied, so a change lands only when process and quality both hold
☐ System reads both the governance config (`gates.toml`/`permissions.toml`) and the check config (`.gitbutler/actions/*.toml`) at the **target ref** when composing the decision, so neither clause can be weakened by the change being judged
☐ System has a passing integration test against the real composed gate + real git that asserts (a) all-checks-green-but-no-review is blocked, (b) review-present-but-required-check-failed is blocked, and (c) both-satisfied proceeds — proving the clauses compose rather than replace

> **No dedicated UI work.** Composition is server-side gate logic (one `enforce_merge_gate` evaluates permission + review + required-checks clauses together). Its UI consequence — a single merge decision showing both Process (governance) and Quality (Actions) sections — is the merge-dialog composition specified in [UC-GATE-01](#uc-gate-01-required-checks-clause--block-unless-all-required-checks-green--current-head); no net-new component belongs here.

---

## UC-GATE-03: Fail-closed on missing/stale/unverifiable/invalid-config + STEER redirect
A quality gate whose source of truth is a ledger is only safe if the **absence, staleness, or untrustworthiness of a result denies, never allows** — and it is only useful if the denial tells the agent what to do next. This use case makes both explicit. The gate **fails closed**: a required check with no current-head result, an unreadable/malformed check config at the target ref (`config.invalid`, mirroring governance's fail-closed config handling), a result whose **signature does not verify**, or a result bound to a **different SHA** is treated as **not satisfied** — the gate denies rather than vacuously passing (a fail-open here would let every positive-path test stay green while broken code lands). Fail-closed also governs **control flow**: the required-checks clause must **not be short-circuited by the governance protected-branch early-return** (`merge_gate.rs:50-56`, which returns `Ok` for a branch that is not flagged `protected`). A branch that carries a `[[required_check]]` set is gated on those checks **independent of the `protected` flag** — requiring a check can never silently fail open because the target ref happens not to be in the governance *protected* set, and a `[[required_check]]` that cannot be enforced fails closed as `config.invalid` rather than being silently dropped. On any miss, the gate returns the **governance-consistent** denial `{code, message, remediation_hint}` naming the unmet check(s) and the **miss-reason** (`check_missing` / `check_failed` / `check_stale_at_head` / `check_unverifiable` / `config.invalid`), plus **STEER** fields, so the orchestrator can **redirect** the agent to the precise corrective action (run the missing/stale check / fix the failing check / re-run after the mutation). A **stale-at-head** required check specifically yields `check_stale_at_head` + STEER "run the check" — the gate does **not** auto-re-run it (it is a read-only consumer; the executor re-produces it). This is the third leg of the produce → consume → redirect loop.

### Acceptance Criteria
☐ The gate fails closed (blocks) when a required check has no result bound to the current head SHA, returning a denial rather than vacuously satisfying the requirement
☐ The gate fails closed when the target-ref check config is unreadable or malformed, denying with a `config.invalid` contract rather than treating the required-set as empty/satisfied (consistent with governance fail-closed)
☐ The gate fails closed when a required check's result signature does not verify or is bound to a different SHA, treating an unverifiable/replayed result as not satisfied, so a forged or stale record never satisfies
☐ The gate returns, for a **stale-at-head** required check, the `check_stale_at_head` miss-reason with a STEER "run the check" redirect and does **not** auto-re-run the check itself (the read-only gate defers re-production to the executor, UC-EXEC-04/UC-GATE-04), so a stale result blocks-and-redirects rather than silently re-running inside the gate
☐ The gate returns the governance-consistent denial `{code, message, remediation_hint}` naming the unmet required check(s) and distinguishing the miss-reason (`check_missing` / `check_failed` / `check_stale_at_head` / `check_unverifiable` / `config.invalid`), plus STEER fields naming a corrective next action (run the missing/stale check / fix the failing check / re-run after the graph mutation), so a denied agent is redirected rather than merely blocked
☐ System enforces the required-checks clause whenever a `[[required_check]]` set exists at the target ref **independent of the branch's `protected` flag** — a branch carrying required checks is gated on them even if it is not in the governance *protected* set, so the gate is never short-circuited into fail-open by the protected-branch early-return (`merge_gate.rs:50-56`), and a `[[required_check]]` that cannot be enforced fails closed as `config.invalid` rather than silently allowing the merge
☐ System has a passing integration test against the real gate that asserts (a) missing/failed/stale/unverifiable each block with the correct miss-reason code (stale → `check_stale_at_head` + "run the check", no auto-re-run), (b) malformed target-ref config denies as `config.invalid` rather than passing, (c) the denial carries a non-empty `remediation_hint` + STEER fields naming the unmet check, and (d) a branch that carries a required check but is **not** flagged `protected` still blocks on that check (the clause is not bypassed by the protected-branch early-return)

### UI/UX Wireframe

> **Scope calibration.** This UC *is* the user-facing denial surface — the most UI-rich UC in the initiative. The v1 contract is the **structured denial** `{code, message, remediation_hint}` + STEER fields, surfaced dual-audience: machine-parseable (`--json`, for the orchestrator to STEER the agent) and human-readable (CLI text + a deferred desktop denial banner). **v1 present** on CLI; the desktop denial banner is **deferred**.

**Surface:** CLI (`but merge` denial, `--json` for orchestrator) — **v1 present**; desktop denial banner / orchestrator STEER card — **deferred (desktop)** / **out-of-scope (the orchestrator renders STEER in its own UI)**.
**Entry point:** CLI — any governed action the gate blocks (`but merge`); GUI (deferred) — the merge dialog blocked banner; Orchestrator — parses the denial's STEER fields to redirect the agent.
**Trigger:** The gate fails closed (missing/stale/unverifiable/non-success required check, or malformed config).

**Layout sketch — CLI, v1 (human-readable denial with miss-reasons):**
```
$ but merge feature/x into main
✕ gate.check_required — merge blocked: 2 required checks not satisfied at head abc123
   • typecheck   check_failed        fix the failing check, then `but check run typecheck`
   • lint        check_missing       run: `but check run lint --head abc123`
   • sig-check   check_stale_at_head run: `but check run sig-check --head abc123` (bound to ancestor 999eee)
   remediation: run/fix the named checks, then re-attempt `but merge feature/x into main`
exit 1
```
`--json` emits the identical facts machine-parseable (the orchestrator's STEER input):
```json
{ "code": "gate.check_required", "denied": true,
  "unmet": [ {"name":"typecheck","reason":"check_failed"},
             {"name":"lint","reason":"check_missing"},
             {"name":"sig-check","reason":"check_stale_at_head","bound_to":"999eee"} ],
  "remediation_hint": "run/fix the named checks, then re-attempt the merge",
  "steer": { "action": "run-checks", "checks": ["lint","sig-check"], "head": "abc123" } }
```

**Layout sketch — CLI, v1 (`config.invalid` fail-closed):**
```
$ but merge feature/x into main
✕ config.invalid — malformed check config at target ref (.gitbutler/actions/*.toml @ main):
   expected `trigger` to be "on-commit"|"on-merge-attempt", got "on-push"
   remediation: fix the check config at the target ref; merge is blocked until config parses
exit 1
```

**Layout sketch — GUI, deferred (denial banner inside the merge dialog; reuses InfoMessage):**
```
┌──────────────────────────────────────────────────────────────────┐
│ ✕ Merge blocked — 2 required checks not satisfied at abc123       │
│   • typecheck failed · • lint missing · • sig-check stale         │
│   [▶ Run lint, sig-check]  [View typecheck output]  [Copy JSON]   │
└──────────────────────────────────────────────────────────────────┘
```

**Key regions:**
- Denial banner (danger `InfoMessage`) — `code` (`gate.check_required` / `config.invalid`), message, per-check miss-reason list.
- Corrective affordances — `[▶ Run <missing/stale>]` (executor re-produces; never auto-run inside the gate), `[View <failed> output]` (the captured stderr), `[Copy JSON]` (hand the denial to the orchestrator / a bug report).
- STEER fields (machine-only) — `steer.action`, `steer.checks`, `steer.head`; consumed by the orchestrator (e.g. Claude Code) to redirect the agent — not rendered in the human banner, but present in `--json`.

**Miss-reason → visual mapping (this UC owns it):**
```
check_missing        → ○ gray  · clock       · "no result @ head"
check_failed         → ✕ danger· cross-circle · "failure"
check_stale_at_head  → ⏱ warning· refresh     · "stale @ <ancestor>"
check_unverifiable   → ⚿ danger· lock-auth    · "signature unverifiable"
config.invalid       → ✕ danger· danger       · "malformed config (fail-closed)"
```
(All via `CheckStatusBadge` variants, UC-LEDG-04. Never color-only.)

**Interaction flow:**
1. Gate blocks → denial returned (CLI text + `--json`; GUI banner).
2. Human reads the miss-reasons; clicks `[▶ Run …]` for missing/stale or `[View output]` for failed.
3. Orchestrator parses `--json` `steer` fields → redirects the agent to run/fix the named checks → re-attempts the merge.

**States:** check_missing / check_failed / check_stale_at_head / check_unverifiable / config.invalid (each a distinct miss-reason + affordance); multi-check (several unmet at once, listed).

**Existing components to use:**
- `packages/ui/src/lib/components/InfoMessage.svelte` — the denial banner (danger; with `primaryAction` = run, `error` block = JSON, existing "Copy error message" button). This is the **primary reuse** — `InfoMessage` already supports icon + title + content + error-block + primary/secondary actions + copy, which matches the denial contract almost exactly.
- `packages/ui/src/lib/components/Button.svelte` — run/view affordances.
- Net-new `CheckStatusBadge.svelte` (UC-DEFN-01) — per-check miss-reason badges.

**Net-new components (atomic):**
- `CheckDenialBanner.svelte` (molecule) — a thin composition of `InfoMessage` (danger) + the per-check miss-reason list + run/view affordances + `Copy JSON`; composes `InfoMessage` + `CheckStatusBadge` + `Button`. Lives in `apps/desktop/src/components/checks/`. Props: `denial` (`{code, message, remediation_hint, unmet[], steer?}`), `onRun(names)`, `onViewResult(name)`. (Deliberately thin: `InfoMessage` does the heavy lifting.)

**UI mods to existing components:**
- `packages/ui/src/lib/components/InfoMessage.svelte` — MODIFY (small): confirm/extend the `error` block to render the denial JSON with a "Copy as JSON" affordance, and ensure the primary action can be keyed off the denial's `steer.action`. Reason: the denial is the *dual-audience* artifact — the human reads the banner, the orchestrator reads the JSON; `InfoMessage` already exposes `error` + copy, so the mod is likely just wiring, possibly adding a `snippet` for structured per-check rows. Verify against source before claiming a code change — it may need no change beyond usage.

**Accessibility notes:** the banner is `role="alert"` (announced on appearance). Each miss-reason line is a list item with the badge's `aria-label` (e.g. "lint: no result at head — run the check"). `[▶ Run …]` announces "Running lint, sig-check…" via a polite live region. The JSON block is `role="region" aria-label="Denial JSON"` with a copy button. Keyboard: Tab reaches run → view → copy in order; Escape dismisses the banner (does not un-block the merge).

**Edge cases / responsive / platform-specific:** A denial with many unmet checks must not overflow — cap the visible list (e.g. 5) with "+N more" and a "View all" expand. The denial must render even when the check config is malformed (`config.invalid`) — at that point per-check badges are unavailable, so the banner shows the parse error instead. The orchestrator's STEER rendering is **out of GitButler's UI scope** (the orchestrator — Claude Code/Codex/etc. — renders the redirect in its own UI; GitButler only emits the `steer` fields). Lite (React) port required for the banner.

---

## UC-GATE-04: Read-only deterministic consumer — never runs a check, never trusts a claim
The gate's integrity rests on it being a **pure read-only deterministic adjudicator**, structurally separate from production. This use case fixes that boundary. The gate **reads and verifies** — it loads the target-ref required-set, queries the ledger for current-head results, verifies each required check's signature + SHA-binding + conclusion, and decides merge/block as a deterministic function of (target-ref config, current head SHA, signed ledger). It **never runs a check** (production is the executor's job, UC-EXEC-01/04; for `on-merge-attempt` the executor runs them in the pre-merge step, not the gate), **never writes the ledger** (recording is the engine's job, UC-LEDG-02), **never accepts a caller-supplied conclusion** as a substitute for a real run, and **never trusts an agent's textual assertion** that a check passed — provenance over claims (mirroring the repo's no-stub / real-services discipline: the gate is a fact-check, not a trust exercise). When every required check is a verified current-head `success`, the deterministic decision is to allow the governed merge to proceed (composing with the governance clauses, UC-GATE-02).

### Acceptance Criteria
☐ The gate decides merge/block as a deterministic function of (target-ref required-set, current head SHA, signed ledger results), so the same inputs always yield the same decision
☐ The gate never runs a check itself — production is the executor's responsibility (the `on-merge-attempt` pre-merge run is the executor's, UC-EXEC-04) — so the read-only adjudicator and the producer remain separate (who-executes ≠ who-adjudicates)
☐ The gate never writes the ledger — recording is the engine's responsibility (UC-LEDG-02) — so the consumer cannot manufacture the result it reads
☐ System exposes no public path (CLI, `but-api`, N-API, or DB) by which a caller-supplied `Conclusion` is recorded as a satisfying result without a real executor run, so a result is only ever a real run's signed output, never a value passed in by a caller
☐ The gate never accepts an agent's textual claim that a check passed as satisfying a required check — only a verified current-head signed `success` — so the gate is a fact-check, not a trust exercise
☐ System has a passing integration test against the real gate + real ledger that asserts (a) the merge decision is a deterministic function of the signed ledger (an agent-supplied "tests pass" claim with no signed current-head `success` does not satisfy; a verified current-head signed `success` for every required check allows the governed merge to proceed), and (b) no public path (CLI/`but-api`/N-API) records a caller-supplied `Conclusion` without a real run — exercised by attempting to inject a conclusion through every public entry point and asserting none yields a gate-counted result

> **No dedicated UI work.** This is the read-only/pure-evaluator guarantee — the gate never runs a check, never writes the ledger, never trusts a textual claim. It is server-side gate integrity. Its UI-relevant corollary is **negative**: the UI must never offer a "mark this check as passed" / "override" affordance anywhere (by design), and the results view must never accept a caller-supplied conclusion — both already honored by the components specified in UC-EXEC-05 / UC-GATE-03. No net-new component.

---

## UC-GATE-05: Enforces the bootstrap-invariant (required-set change clears current required set)
Reading config at the target ref stops a change weakening the gate *for itself*; this use case closes the complementary hole at the **gate**: the change that *lands* a weaker required-check configuration must itself be gated by the **currently-required** checks. The gate enforces the **bootstrap-invariant** — a change whose diff **adds, removes, weakens, flips `required` on, or edits the `run-spec` of a required check** must clear the currently-required checks (read at the target ref) before it can land. The required-check configuration is therefore **self-protecting**: an agent cannot land "delete the test check" (or flip it to `required: false`) as an ordinary change to weaken the gate for everything after, because that very change must first pass the checks it is trying to remove. This is the same self-escalation-prevention shape governance uses for its ref-pin (a permission change cannot grant itself the authority it needs to land), and it pairs with the v1 process requirement of a **human in the loop** for required-set changes (an automated agent reviewer may not catch the policy implication — see [01-scope.md](./01-scope.md) and [UC-DEFN-05](./04-uc-defn.md#uc-defn-05-self-protecting-required-check-set-the-bootstrap-invariant)).

### Acceptance Criteria
☐ The gate recognizes when the change being merged modifies the required-check set or a required check's definition (adds/removes a required check, flips `required`, or edits a required check's `run-spec`), so a policy-affecting config change is gated as such, not as ordinary content
☐ The gate requires a change that modifies the required-check set or a required check's definition to itself have every **currently-required** check (read at the target ref) green at its head before it can land, so a weakening change cannot escape the checks it must itself pass (the bootstrap-invariant)
☐ System makes a weakened required-check configuration take effect only for **future** changes — once the weakening change has itself cleared the currently-required set — so the config that defines "good" is always governed by the checks it currently mandates (self-protecting, mirroring governance's ref-pin self-escalation prevention)
☐ System has a passing integration test against the real gate + real git that attempts to land a change deleting (or flipping `required: false` on) a currently-required check, asserts the merge is blocked unless that change's own head satisfies the currently-required checks at the target ref, and asserts the weakened config only governs subsequent changes after it has itself cleared the current required set

> **No dedicated UI work.** The bootstrap-invariant is engine-enforced gate logic (a required-set change must clear the currently-required checks). Its UI-visible consequence is a **denial** when such a change's own head is not green — surfaced via the same `CheckDenialBanner` / `gate.check_required` denial specified in [UC-GATE-03](#uc-gate-03-fail-closed-on-missingstaleunverifiableinvalid-config--steer-redirect), plus the self-protecting hint in the `RequiredChecksEditor` (UC-DEFN-03). No net-new component.
