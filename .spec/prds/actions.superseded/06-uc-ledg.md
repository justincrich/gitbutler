---
stability: FEATURE_SPEC
last_validated: 2026-06-19
prd_version: 1.0.0
functional_group: LEDG
---
# Use Cases: Check-Result Ledger (LEDG)

The ledger is the **contract** the gate trusts — provenance over claims. A check result is a **signed `(name, head_sha, conclusion, producer-identity)`** record on an **agent-unwritable** recording surface, written by **deterministic engine code** (recording a result is never an agent decision — the deterministic-vs-probabilistic doctrine). Its identity is **`(name, head_sha)`**: a result attests exactly one check against exactly one commit. The load-bearing property is the **SHA-reset invariant** — a result is valid for the head SHA it was produced against and is **invalidated/recomputed on every graph/ref mutation** (GitButler rebases/edits/reorders commits constantly via the rebase editor), so a green bound to an ancestor SHA never satisfies the gate. This is the **hardened** counterpart to governance's deliberately-forgeable review store (its R6): a machine verdict for an agent with no out-of-band stakes cannot be left forgeable. (The v1 HMAC residual — the agent shares the OS user with the producer in the personal-tenant model — is named honestly in [01-scope.md](./01-scope.md); the closure is an OS-sandboxed executor + Ed25519.)

> **The contrast with governance's review store (deliberate).** Governance's `local_review_verdicts` is knowingly forgeable by direct DB write (its R6, accepted because a human reviewer carries accountability). The Actions ledger is the opposite by design — signed, agent-unwritable, deterministically recorded — because the verdict it carries is a machine's "this committed check ran and exited 0," produced for a goal-directed agent that will forge it the instant it is left casually forgeable. The non-fakeability is not optional hardening; it is what makes the verdict mean anything (see [02-roles.md](./02-roles.md)).

> **Metadata is outside the trust boundary.** The signed tuple is `(name, head_sha, conclusion, producer-identity)`. Captured stdout/stderr/log-refs live in a `metadata` column that is **agent-readable** (same OS user) and **not signed-as-truth** — it is evidence, not the verdict. Injected-secret values are masked before storage and raw-output retention is limited (UC-EXEC-03); the gate never trusts `metadata`, only the signed tuple.

| ID | Title | Description |
|----|-------|-------------|
| UC-LEDG-01 | Signed `(name, head_sha, conclusion, producer-identity)` records | A check result is recorded as a signed tuple binding the check name, the exact head SHA, the typed conclusion, and the producer identity, so the gate can verify the result was produced by the trusted executor and bound to this commit — never forged, replayed, or hand-edited by the agent (for a fixed check definition). |
| UC-LEDG-02 | Agent-unwritable surface, deterministic recording | Results live on a recording surface the gated agent cannot write to with producer authority, and recording is deterministic engine code (never an agent decision), so a passing record cannot be authored by the entity being gated. |
| UC-LEDG-03 | SHA-reset invariant — invalidate/recompute on every graph mutation | A result is valid for its bound head SHA only; every graph/ref mutation (rebase, commit edit, reorder) invalidates results for the prior SHA, so a green bound to an ancestor SHA never satisfies the gate — the anti-cheat property, made load-bearing by butler's constant SHA rewriting. |
| UC-LEDG-04 | Typed conclusion semantics + current-head-only read | The conclusion is a typed terminal verdict (stored vocabulary `success | failure | neutral | cancelled | timed_out | skipped`; v1 executor produces only `success`/`failure`/`timed_out`) distinct from a non-terminal status, and the gate reads results for the **current** head SHA only, so policy semantics port from GitHub and a result for any other SHA is never consulted. |

---

## UC-LEDG-01: Signed `(name, head_sha, conclusion, producer-identity)` records
A result the agent can forge, replay, or hand-edit is worthless as a gate. This use case makes a check result a **signed tuple**: the producer (the executor, UC-EXEC-01) signs `(name, head_sha, conclusion, producer-identity)` with a key the gated agent does not hold, and the engine records that signed record. The signature binds all four fields together so the result cannot be (a) forged by the agent, (b) replayed from another SHA (the `head_sha` is signed), or (c) hand-edited (any edit breaks the signature). The **producer identity** is part of the signed payload so the gate can require results from the trusted executor specifically (the analog of GitHub check runs being authored by the App via its installation token, not the PR author). At gate time the consumer verifies the signature and the SHA-match before counting the result (GATE). (Honesty caveat from scope: v1 uses symmetric HMAC, which raises but does not fully close agent forgery under the shared-OS-user trust model; the signed-tuple binding is what makes replay/hand-edit detectable regardless, and Ed25519 is the deferred closure.)

### Acceptance Criteria
☐ System records a check result as a tuple binding the check `name`, the exact `head_sha`, the typed `conclusion`, and the `producer-identity`, so identity is `(name, head_sha)` and provenance is part of the record
☐ The producer (the executor) signs the result tuple with a key the gated agent does not hold under the producer's authority, so a result cannot be authored by the entity being gated under that authority
☐ System binds the `head_sha` inside the signed payload, so a result cannot be replayed from a different SHA without breaking the signature
☐ System makes any hand-edit of a recorded result detectable by breaking its signature, so a tampered record is not counted as valid
☐ System records the producer identity in the signed payload so the gate can require a result from the trusted executor specifically (the analog of an App-authored check run), not from an arbitrary writer
☐ System has a passing integration test against the real ledger + real signer that records a signed result, verifies the signature + SHA-binding, and asserts that mutating any field (name / head_sha / conclusion) or re-pointing it to another SHA causes signature verification to fail

> **No dedicated UI work.** Signing is ledger/producer internals. The signed-tuple *fields* (`producer_identity`, `signed`, `bound-to head_oid`) are **displayed** as provenance in the results view (UC-EXEC-05 / UC-LEDG-04), but no net-new component belongs here. The `CheckStatusBadge` "signed ✓" affordance (UC-DEFN-01) is the only UI-visible derivative of the signature.

---

## UC-LEDG-02: Agent-unwritable surface, deterministic recording
Signing closes forgery; this use case closes the writing surface and the decision to write. Results live on a recording surface — a protected store / server-validated oplog — that the **gated agent cannot write to with producer authority**: there is no `but` action, DB path, or file an agent can use to insert a result the gate will trust. And recording is **deterministic engine code**, not an agent tool call or an LLM decision — when the executor produces a conclusion, the engine records it as a guaranteed step (honoring the deterministic-vs-probabilistic doctrine: an action that must always happen is deterministic code, never an agent's choice). This is the structural complement to signing: even setting aside the signature, the agent has no write path to the trusted record, and the act of recording is never something the agent can skip, fake, or substitute.

### Acceptance Criteria
☐ System records check results on a surface the gated agent cannot write to with producer authority, so there is no agent write path to a result the gate trusts
☐ System exposes no `but` action, DB path, or file by which the gated agent can insert a check result the gate counts as valid, so the recording surface is agent-unwritable in practice, not just by signature
☐ System records a produced result via deterministic engine code (not an agent tool call or an LLM decision), so recording always happens and is never an agent's choice (the deterministic-vs-probabilistic doctrine)
☐ System treats "produce → record" as a guaranteed deterministic step driven by the engine after the executor concludes, so a conclusion the executor produced cannot fail to be recorded due to agent behavior
☐ System keeps adjudication out of the recorder — recording a result and deciding the merge are distinct (the gate, UC-GATE-01, is the only adjudicator) — so the recorder never decides pass/fail, it only records the signed producer output
☐ System has a passing integration test asserting the recording surface rejects a write attempted with the gated agent's authority (no valid record produced) while the engine deterministically records the executor's signed result, so the only path to a trusted record is producer → engine

> **No dedicated UI work.** Agent-unwritability + deterministic recording are engine/store guarantees with no user surface. There is intentionally **no** "record a result" affordance anywhere in the UI (by design — an agent must not be able to author a result); this UC's UI consequence is the *absence* of any write control for results, which is the correct empty state.

---

## UC-LEDG-03: SHA-reset invariant — invalidate/recompute on every graph mutation
This is the anti-cheat property, and it is **more** load-bearing in butler than in GitHub because butler rewrites the git graph constantly (rebases, commit edits, reorders via `but_rebase::graph_rebase::Editor`). A result is valid for the **exact head SHA** it was produced against; the instant a graph/ref mutation changes the head SHA, results bound to the **prior** SHA are **invalidated** for the new head — they do not carry over. So an agent cannot pass a check once and then mutate the change underneath the green: any commit edit, rebase, amend, squash, or reorder that produces a new head SHA leaves the new SHA with **no satisfying results** until the checks are (re-)produced and pass on the **new** SHA. The gate reads results **for the current head SHA only** (UC-LEDG-04), so an ancestor-SHA green is structurally invisible to it. This mirrors GitHub's "new push invalidates," generalized to every graph mutation. (The residual eval-vs-merge TOCTOU race — head advancing between gate-eval and the commit — is named in [01-scope.md](./01-scope.md); "require up to date"/merge-queue is its closure.)

### Acceptance Criteria
☐ System treats a check result as valid for the exact head SHA it was produced against and not for any other SHA, so validity is per-commit, never per-change-loosely
☐ System invalidates results bound to the prior head SHA for the new head when a graph/ref mutation (rebase, commit edit, amend, squash, reorder) changes the head SHA, so a green never carries across SHAs
☐ System leaves a newly-produced head SHA (after a mutation) with no satisfying required-check results until the checks are (re-)produced and pass on that new SHA, so an agent cannot mutate the change underneath an existing green
☐ System makes an ancestor-SHA `success` structurally invisible to the gate (the gate reads the current head SHA only, UC-LEDG-04), so a stale green is never counted as satisfying
☐ System recomputes (or marks for re-run) the required checks on every SHA-changing mutation rather than reusing a prior SHA's results, consistent with keying on commit IDs at the boundary (`crates/WORKSPACE_MODEL.md`)
☐ System has a passing integration test against the real ledger + real git that records a `success` for a check at head H1, mutates the change so the head becomes H2 (rebase/amend), and asserts the gate sees no satisfying result at H2 (the H1 green does not carry over) until the check is re-produced and passes at H2

> **No dedicated UI work.** The SHA-reset invalidation is engine machinery keyed on head OID. Its UI-visible consequence is a **stale result** (bound to an ancestor SHA) rendering distinctly in the results view — already covered by the `stale` variant of `CheckStatusBadge` (`warning` + `refresh`, with a `[▶ Run now]` affordance) specified in UC-EXEC-05 / UC-LEDG-04. No net-new component.

---

## UC-LEDG-04: Typed conclusion semantics + current-head-only read
The gate's pass/block logic depends on a well-defined conclusion vocabulary and on never consulting a result for the wrong commit. This use case fixes both. The **conclusion** is a typed **terminal** verdict — the stored vocabulary is the GitHub-compatible `success | failure | neutral | cancelled | timed_out | skipped`, distinct from a non-terminal **status** (`queued | running | completed`) — so policy semantics port. v1's exit-code executor **produces** only `{success, failure}` (plus `timed_out` when the timeout wrapper fires); `{cancelled, neutral, skipped}` are **reserved/parsed-not-produced** — the ledger can store them if a future producer emits them, but the v1 default executor does not author them. Blocking semantics: `success` satisfies a required check; `failure` / `timed_out` / `cancelled` block; `neutral` / `skipped` are non-blocking by default (configurable later) — and the gate blocks on **any non-`success` required conclusion** regardless of which producer wrote it. The ledger read is **current-head-only**: a query for "the result of check `name`" for a change returns the record bound to the change's **current** head SHA, or none — a record bound to any other SHA is never returned. Together these make the gate's input a precise, current, typed fact.

### Acceptance Criteria
☐ System represents a conclusion as a typed terminal verdict whose stored vocabulary is `success | failure | neutral | cancelled | timed_out | skipped` (distinct from a non-terminal status), of which the v1 default executor **produces** only `success`/`failure`/`timed_out` while `cancelled`/`neutral`/`skipped` are reserved/parsed-not-produced (stored only if a future producer emits them), so policy semantics port from GitHub without the v1 executor authoring conclusions it does not compute
☐ System treats `success` as satisfying a required check and blocks on any non-`success` required conclusion (`failure` / `timed_out` / `cancelled`), so a non-success terminal conclusion never satisfies the gate regardless of producer
☐ System treats `neutral` / `skipped` as non-blocking by default (configurable later), so a skipped check does not block while a failed/timed-out one does
☐ System returns, for "the result of check `name`" on a change, only the record bound to the change's current head SHA (or none) — never a record bound to a different SHA — so the gate's input is always current-head
☐ System has a passing integration test against the real ledger that stores results across the conclusion vocabulary and asserts (a) the current-head read returns only the current-head record (an ancestor-SHA record is not returned), (b) the terminal-vs-status distinction is preserved, and (c) the v1 executor produces only `success`/`failure`/`timed_out` while a stored `neutral`/`skipped`/`cancelled` is parsed-not-produced by it

### UI/UX Wireframe

> **Scope calibration.** This UC fixes the **conclusion vocabulary** the UI renders and the **current-head-only read** the results view is bound to. The v1 surface is the CLI (`but check results`, UC-EXEC-05); the GUI panel is **deferred**. This UC's distinct UI contribution is the **conclusion → badge color/icon mapping** (a design-token decision, reused everywhere a conclusion appears).

**Surface:** CLI — **v1 present** (shares the `but check results` output with UC-EXEC-05); desktop — **deferred**.
**Entry point:** Same as UC-EXEC-05 (`but check results --head <sha>` / the results panel).
**Trigger:** Any view that lists check results (results panel, gate summary UC-GATE-01, denial UC-GATE-03).

**Conclusion → visual mapping (the design decision this UC owns):**
```
v1-PRODUCED (exit-code executor authors these):
  success      →  safe    · tick-circle   · "success"     (satisfies a required check)
  failure      →  danger  · cross-circle   · "failure"     (blocks)
  timed_out    →  warning · clock          · "timed out"   (blocks)

RESERVED (parsed-not-produced by v1; stored if a future producer emits them):
  cancelled    →  gray    · stop           · "cancelled"   (blocks)
  neutral      →  gray    · stop           · "neutral"     (non-blocking by default)
  skipped      →  gray    · stop           · "skipped"     (non-blocking by default)

GATE-DERIVED (not produced; computed at read time — used by the gate summary/denial):
  missing      →  gray    · clock          · "no result @ head"
  stale        →  warning · refresh        · "stale (bound to <ancestor>)"
  unverifiable →  danger  · lock-auth      · "signature unverifiable"
```
Every state has icon + label + `aria-label` — **never color-only** (color-blind safety). All colors reuse existing tokens (`--fill-safe-bg`, `--fill-danger-bg`, `--fill-warn-bg`, `--chip-gray-bg`) — **no new design tokens**.

**Layout sketch — CLI, v1 (current-head-only read; ancestor results are invisible):**
```
$ but check results --head abc123
NAME          CONCLUSION  BOUND-TO  SIGNED
tests         success     abc123     yes
typecheck     failure     abc123     yes
lint          (no result bound to abc123)

# an ancestor-SHA green (e.g. success @ parent 999eee) is NEVER returned here —
# the read is current-head-only (UC-LEDG-04). Use --head <other-sha> to inspect history.
exit 0
```

**Key regions / behavior:**
- The results list is always scoped to **one head OID** — there is no "mix SHAs" view; querying a different head is an explicit `--head`/panel switch. This makes an ancestor-SHA green structurally invisible (the load-bearing anti-replay property made visible).
- A `stale` row appears only when the UI is asked "what satisfies the gate at head X?" and a result exists for an ancestor — it is flagged, never silently counted green.

**Interaction flow:**
1. User opens results for a head; the query returns only current-head records (or none).
2. Each conclusion renders via the mapping above (`CheckStatusBadge`).
3. A stale/missing conclusion for a required check is the trigger for the gate summary (UC-GATE-01) and STEER denial (UC-GATE-03).

**States:** all eight conclusion/derived states above; empty (`EmptyStatePlaceholder` "No results for this head"); current-head-only (ancestor results not shown).

**Existing components to use:** same as UC-EXEC-05 (`CardGroup`, `Timestamp`, `Codeblock`, `EmptyStatePlaceholder`).

**Net-new components (atomic):**
- `CheckStatusBadge.svelte` (atom, defined in UC-DEFN-01) — **this UC owns its variant table** (the conclusion→color/icon mapping above). The component is the single source of truth for that mapping across results, gate summary, and denial surfaces. Lives in `packages/ui/src/lib/components/CheckStatusBadge.svelte`.

**UI mods to existing components:** none.

**Accessibility notes:** every badge carries `aria-label="<conclusion> — <one-line meaning>"` (e.g. `aria-label="timed out — blocks the merge"`). The "current-head-only" guarantee is conveyed in the panel header (`aria-label="Results for head abc123 only"`). Color is always paired with an icon + text label.

**Edge cases / responsive / platform-specific:** A stored `neutral`/`skipped`/`cancelled` (future producer) must still render correctly via the reserved mapping even though the v1 executor never authors them — the badge must not break on an unrecognized value (fail to a `gray` "unknown conclusion" rather than crash). CLI table column widths auto-fit; long SHAs truncate with a tooltip on `--wide`.

---

> LEDG note: the four UCs total **23 ACs**. Identity = `(name, head_sha)` and the SHA-reset invariant (UC-LEDG-03) are the load-bearing properties; UC-LEDG-02's agent-unwritable + deterministic-recording is the structural complement to UC-LEDG-01's signing. The signed tuple is `(name, head_sha, conclusion, producer-identity)`; `metadata` is agent-readable evidence outside the trust boundary. Together they make a passing record impossible for the gated agent to forge, replay, hand-edit, write, or carry across SHAs (for a fixed check definition, under the v1 trust model named in scope).
