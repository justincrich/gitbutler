---
stability: CONSTITUTION
last_validated: 2026-06-19
prd_version: 1.0.1
section: technical-requirements
---

# Architecture Posture

## The control model in one line

> **Validations ("Checks") are a producer/consumer system: the merge gate is a deterministic, READ-ONLY CONSUMER of a signed, agent-unwritable check-result ledger; butler ships a butler-controlled DEFAULT EXECUTOR as the PRODUCER. The gate NEVER invokes the executor — `on-merge-attempt` checks are run by the trusted CLI/daemon IMMEDIATELY BEFORE the merge action, and the gate then consumes the current-head results. What the gate proves is narrow and exact: "the committed, target-ref-pinned check ran under the trusted executor and exited 0 at the current head OID" — NOT "the code is correct." The hardness of forging a green check = who-produces (executor ≠ the gated agent) + how-bound (signed, SHA-bound, agent-unwritable recording); it does NOT live in the gate. Results are valid for the current head OID only and are invalidated on every graph/ref mutation.**

> **Naming.** The implementation surface is **`but check`** (CLI), **`but-checks`** (crate), **`check_results`** (`but-db` table). This checks/validations system is deliberately distinct from GitButler's pre-existing "Actions" feature (the `butler_actions` macro table, the `actions::Platform` noun) — see `02-system-components.md` for the disambiguation note. "Checks" is the engineering system; the product PRD may surface it under a user-facing label owned by the product-manager sections.

## Stance 0 — consume + default-produce (the governing decomposition)

Two halves, deliberately separated:

1. **Consume.** The merge gate gains a **required-checks clause**: it blocks a merge unless every _required_ check name has a recorded result whose `conclusion` is `success`, whose `signature` verifies against a trusted producer identity, and whose `head_oid` equals the **current** head OID. The gate is pure, deterministic, **read-only** — it _reads_ the ledger and decides. It **never runs anything**, never invokes the executor, and is **not** where the hardness of forgery lives.
2. **Default-produce.** Butler ships a **default executor** that resolves a ref-pinned check definition (`.gitbutler/actions/*.toml`), runs it, derives a typed `Conclusion`, **signs** the result with the butler producer identity, and **records** it in an agent-unwritable ledger. The executor is the _producer_; the hardness of forging a green check lives **here**, in who runs it and how the result is bound.

The decomposition is the whole design: a read-only consumer that trusts only signed, SHA-bound records, and a producer that is structurally outside the gated agent's reach. Swapping the gate's _policy_ (which checks are required) never weakens forgery-hardness, because the gate never produces a result — it only reads one.

### `on-merge-attempt` is an orchestrator/CLI pre-merge step, NOT a gate trigger (DECISION A)

A check definition may declare it should run **`on-merge-attempt`**. This does **not** mean the gate runs it. The gate is the same pure, synchronous, read-only consumer that the real `enforce_merge_gate` (`merge_gate.rs:40`) already is. `on-merge-attempt` is honored by the **trusted CLI/daemon**, which orchestrates the merge:

1. The trusted CLI/daemon (the same process that holds `Merge` and the producer key) resolves the set of `on-merge-attempt` checks for the target branch (from `.gitbutler/gates.toml [[required_check]]` at the target ref).
2. It runs `but check run` for each, **immediately before** calling the merge action — producing/refreshing signed rows at the current head OID.
3. It then calls the merge action; `enforce_merge_gate` runs its read-only required-checks clause, consuming the rows that were just produced at the current head.

The gate's contract is unchanged from governance: it reads, verifies, and decides. The producer→record step and the consumer→read step are two distinct synchronous phases the orchestrator sequences; **the gate never produces a result.** This preserves the trust claim end-to-end: an agent driving `but merge` cannot smuggle execution into the gate, because the gate has no execution path — it only has a ledger read. See `04-api-design.md` for the orchestration sequence and `08-capability-chains.md` (CAP-CHECK-01) for the hop-by-hop contract.

## Stance 1 — the executor is NOT the gated agent (the load-bearing trust boundary)

The single most important security property: **the entity that produces a check result is structurally distinct from the agent the check gates.** The default executor runs **in the trusted daemon / CLI that the human (or the trusted orchestrator) runs** — the same trust class as the human at the keyboard — **not** inside the agent's process. The gated agent can edit files, can ask (via the orchestrator) for a check to run, can read results; it **cannot** mint a result that the gate will trust, because it does not hold the producer signing key and cannot write the ledger through any governed path.

|          | The default executor (producer)                                                     | The gated agent (subject)                                                             |
| -------- | ----------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| Trust    | **Trusted** — runs in the daemon/CLI the human runs; holds the producer signing key | **Semi-trusted** — bound at the gate by what the executor signed                      |
| Produces | the signed, SHA-bound check result recorded in the ledger                           | nothing the gate trusts — its textual claim "the check passed" is **never** consulted |
| Site     | the trusted daemon/CLI (not the agent's reasoning loop)                             | the harness (its reasoning is out of scope, Assumption: governance Stance 6)          |

The gate **trusts signed records, never the agent's textual claim.** An agent saying "tests passed" in its output is inert: the gate reads the ledger, verifies the signature, checks the head OID — it never parses agent prose. This is the Checks analogue of the governance posture's "control on the action, not the tool."

## Stance 2 — no broker in v1 (the executor runs in the trusted process; the runner protocol is deferred)

GitHub Actions' full model has a control plane that brokers jobs to _labelled runners_ via a lease/long-poll protocol. **v1 ships none of that.** The default executor runs **synchronously in the trusted daemon/CLI** the human runs; there is no job broker, no lease, no long-poll, no remote runner registration, no label-based dispatch. This is a deliberate scope line: it makes the trust boundary trivial to reason about (the producer _is_ the trusted process), and it defers the genuinely hard distributed-systems problem.

| Layer                                                                    | v1                                                                                      |
| ------------------------------------------------------------------------ | --------------------------------------------------------------------------------------- |
| Executor in the trusted daemon/CLI the human runs                        | **the v1 producer**                                                                     |
| Lease / long-poll / label-matched remote runner protocol                 | **deferred** (named, not built)                                                         |
| Untrusted / fork-PR execution (running an untrusted contributor's check) | **deferred + out of scope** (the v1 executor runs trusted, repo-local definitions only) |

The deferred runner protocol is the layer that would let an untrusted or remote producer participate; until it exists, the only producer is the trusted, repo-local default executor, and the forgery-hardness claim holds **only** for results it signs. The build must NOT present v1 as supporting remote/untrusted runners.

## Stance 3 — the check-result contract (identity + typed conclusion + signed producer + agent-unwritable recording)

A check result is a four-part contract, and every part is load-bearing:

| Part                           | Value                                                                                                                                                                                                      | Why it is load-bearing                                                                                                                                                                                                                                                                                          |
| ------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Identity**                   | `(name, head_oid)`                                                                                                                                                                                         | a result is _about_ a named check _at a specific head OID_; the gate matches required-check names against results at the current head                                                                                                                                                                           |
| **Typed conclusion**           | a `Conclusion` enum — the 6-variant _type_ (`success` / `failure` / `neutral` / `cancelled` / `timed_out` / `skipped`, mirroring GitHub's check-run conclusions) — **never a free string**                 | the gate's allow-decision is `conclusion == success`; a stringly-typed conclusion would let a typo or an unknown variant silently pass or fail. False-friend: this is NOT `String`. v1 only _produces_ `{success, failure, timed_out}` (see `03-data-schema.md`), but parses/stores all six for forward-compat. |
| **Signed producer identity**   | `producer_identity` + `signature` over the canonical `(name, head_oid, conclusion)` tuple, keyed by the butler producer key                                                                                | the gate verifies the signature; an unsigned or wrongly-signed record is **not** a satisfying result — this is the "how-bound" half of forgery-hardness                                                                                                                                                         |
| **Agent-unwritable recording** | recorded in a NEW `but-db` ledger table the agent **cannot write through any governed path**; recording is **deterministic engine code** invoked by the executor, never an agent tool call or LLM decision | the agent must not be able to INSERT a forged `success` — this is the lesson learned from `local_review_verdicts` (governance R6, forgeable by direct DB write), corrected here by signing + producer-binding so a direct INSERT without a valid signature is rejected by the gate                              |

## Stance 4 — recording is deterministic engine code (never an agent decision)

Per the project's deterministic-vs-probabilistic doctrine: **things that MUST always happen are engine code, not an agent decision.** Recording a check result, signing it, and the gate's evaluation are all deterministic. The _probabilistic_ part — what the agent does in response to a `failure` (read the denial, fix the code, re-run) — is owned by the harness, outside this PRD.

| Concern                                                                                                 | Owner                                                                    | Determinism                                                                                                                                                                                                |
| ------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Whether a recorded result _satisfies_ a required check (signature + head-OID + `conclusion == success`) | the gate's required-checks clause (`enforce_merge_gate`) — **read-only** | **deterministic**                                                                                                                                                                                          |
| Running the check and deriving its `Conclusion`                                                         | the default executor (`but-checks`)                                      | **deterministic given the check definition + working state** (the check itself may be a flaky test suite; the executor faithfully records whatever conclusion the run produced — it does not editorialize) |
| Signing + recording the result in the ledger                                                            | `but-checks` recording engine code                                       | **deterministic** — always happens after a run, never skippable by the agent                                                                                                                               |
| Sequencing `on-merge-attempt` run(s) before the merge call                                              | the trusted CLI/daemon orchestrator (DECISION A)                         | **deterministic** — the orchestrator runs the checks, then calls merge; the gate stays read-only                                                                                                           |
| What the agent does about a `failure`                                                                   | the agent (in the harness)                                               | **probabilistic — owned by the harness, not this PRD**                                                                                                                                                     |

The boundary is the gate and the ledger: butler owns the deterministic _produce → sign → record → consume_; the harness owns the probabilistic _react-to-result_. The gate's slice of this is **consume only**.

## Stance 5 — the SHA-reset invariant (load-bearing, butler-specific)

This is the property that has no GitHub analogue and that the build MUST get exactly right. **A check result is valid for the head OID it was produced against, and ONLY that OID.** GitButler's model is virtual branches over one working tree, and history is routinely rewritten — rebases, commit edits, amends, reorders all go through `but_rebase::graph_rebase::Editor`, which materializes new commit objects (new OIDs) via `Editor::rebase()` and exposes the old→new mapping via `Editor::commit_mappings()` (`crates/but-rebase/src/graph_rebase/mod.rs:479`). When the head OID moves, **every result keyed to the old OID is stale** and must not satisfy the gate.

The mechanism (specified, not hand-waved):

1. **Key by head OID.** Every ledger record carries `head_oid` (the head OID; named `head_oid` to match the shipped governance schema — see `03-data-schema.md`). The result for check `X` at OID `a1b2…` is a _different row_ from the result for `X` at OID `c3d4…`.
2. **The gate reads only the current head.** `enforce_merge_gate` resolves the _current_ source head OID (it already does this — `merge_gate.rs:78` `current_head_oid`) and matches required checks against ledger rows where `head_oid == current_head_oid`. A row at any other OID is invisible to the gate. (Per `crates/WORKSPACE_MODEL.md`: commit IDs/refs at the boundary; the gate keys on the current commit ID.)
3. **A mutation → stale.** After any graph/ref mutation, the head OID differs from the OID prior results were keyed to, so those results are automatically non-satisfying — no explicit "invalidate" write is required for _correctness_ (a stale row simply no longer matches the current head). The executor must **re-run** against the new head to produce satisfying results; an explicit prune of superseded rows is an optimization (LEDGER housekeeping), not a correctness requirement.

The honest framing: this invariant is **automatic by construction** if and only if every result is keyed by OID and the gate matches on the current OID. The failure mode (governance R4's analogue) is matching by check _name_ alone, ignoring `head_oid` — that would let a `success` produced at an old, un-reviewed head satisfy the gate after the agent rewrites history. The build MUST match on `(name, head_oid == current_head_oid)`, never on `name` alone.

**Caveat — this invariant is correct but NOT atomic (TOCTOU).** Reading the current head OID and consuming the merge are two steps; the head can advance between them. The gate-read → merge-commit window is a named race, **R12** in `07-technical-risks.md` (it is worse on the forge-merge accepted-leak path, where the local gate's accepted `success` can be left behind by a forge-side head advance). The closure ("require up to date" / a merge queue) is deferred; do not present the SHA-reset invariant as eliminating the TOCTOU. See the SHA-reset race in `07-technical-risks.md` (R3) and the SHA-reset TOCTOU (R12).

## Stance 6 — fail closed (an absent, unreadable, or unverifiable result is NOT a pass)

The gate fails closed at every boundary, exactly as the governance gates do:

- A **required check with no result** at the current head → **blocked** (`gate.check_required`, exit 1) — never an implicit allow.
- A result whose **signature does not verify** → treated as **absent** (not a pass).
- A result whose **`head_oid` ≠ current head OID** → invisible (stale; not a pass).
- An **unreadable / malformed check definition** (`.gitbutler/actions/*.toml`) → `config.invalid`, fail closed (mirrors `merge_gate.rs`'s `config_invalid`).
- A result with `conclusion ∈ {failure, cancelled, timed_out}` → **blocked**; `neutral`/`skipped` are **configurable** (a check definition declares whether `neutral` satisfies — default: only `success` satisfies a _required_ check).

No partial success, no silent skip. This is the same fail-closed posture as `enforce_merge_gate`'s existing review clause.

## Stance 7 — composition with governance (producer↔consumer over one gate)

Checks does not stand up a second gate. It **extends the one merge gate** that governance shipped, composing with its existing clauses in a fixed order inside `enforce_merge_gate`:

1. **Authority** — the acting principal must hold `Merge` (`merge_gate.rs:48`, unchanged).
2. **Review requirement** — the existing distinct-approval-@head clause (`merge_gate.rs:86`, unchanged).
3. **Required checks** — the NEW clause: every required check name has a signed `success` at the current head OID (added after authority, composed with review). **Read-only** — it consumes ledger rows; it does not run checks.

All three share the governance denial contract — `{code, message, remediation_hint}` (`but_authz::Denial`) — with one new `code`: `gate.check_required` (alongside the existing `perm.denied`, `gate.review_required`, `config.invalid`). The producer side reuses governance's `statuses:write` authority (already in the `but-authz` catalog at `authority.rs`, previously "catalog-only") to gate the recording **trigger** — Checks makes `statuses:write` a _real_ gated route. This is the producer↔consumer relationship made concrete: governance owns _who may merge / who may review_; Checks owns _what must have passed_ — both judged at the one read-only gate.

## Stance 8 — the fence is accepted-leaky (inherited from governance, by design)

Soundness in the _full_ vision rests on butler being the agent's sole path to the canonical repo (governance Stance 5). This slice does **not** build that boundary; it inherits the accepted-leak fence wholesale. The Checks-specific residuals of the same class are named in `07-technical-risks.md`:

| Residual                                                                                                                                                                                                                                                                           | Class                                      | This slice                                                                                                                                                                                                                                                                     |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| The default executor (producer) signs results; the **signing key's secrecy** is the root of forgery-hardness. In the **personal-tenant model the agent shares the OS user with the executor** and can read the `but-secret` keyring producer secret **without elevated privilege** | the steel-trap analogue                    | v1 stores the key via `but-secret` (keyring); reading it **raises the bar** but does not close the boundary — a **named accepted residual** (R2). The real closure is **Ed25519 (asymmetric) + an OS-sandboxed executor** where the key is unreachable by the agent (deferred) |
| A merge done entirely on the **forge** (auto-merge / UI / CI) or by **raw push** bypasses the local gate (so no local check is consulted)                                                                                                                                          | same accepted-leak class as governance R11 | inherited residual; the deferred server-side pre-receive + forge-side required-checks close it (also where R12's TOCTOU is worst)                                                                                                                                              |
| An **un-audited N-API path** reaching a lower-level merge route directly                                                                                                                                                                                                           | same class as governance R14               | inherited residual; the N-API audit closes it                                                                                                                                                                                                                                  |
| The gate-read → merge-commit window is **not atomic** (TOCTOU)                                                                                                                                                                                                                     | a new butler-specific residual             | named **R12**; closure is "require up to date" / merge-queue (deferred)                                                                                                                                                                                                        |

The build must NOT present the executor's signing as unbreakable, the local gate as binding forge/push merges, the gate as atomic with the merge commit, or v1 as supporting untrusted runners. **What the gate proves is narrow:** "the committed, target-ref-pinned check ran under the trusted executor and exited 0 at the current head OID" — never "the code is correct," and never "non-fakeable." The value is the same as governance's irrigation bet: the governed path (run the check through the trusted executor, land through the read-only gated merge) is the cheapest path; defection is possible but uphill, and the producer/signing boundary makes _forging a green check_ materially harder than the governance review-row forgery it learns from.

## Stance 9 — the required-check config is self-protecting (the bootstrap invariant, R11's real closure)

A green check proves only that _the committed check ran_. The obvious bypass is therefore not to fake a result but to **weaken the required-check set**: an `administration:write` holder could rewrite `.gitbutler/actions/cargo-test.toml` to `run = "true"`, or drop a name from the `[[required_check]]` block, so the gate's judgment becomes trivial. The closure — the **bootstrap invariant** — mirrors governance's ref-pin self-escalation prevention exactly:

> **A change that modifies the required-check set or a required check's definition (`.gitbutler/actions/*.toml` or the `[[required_check]]` set in `.gitbutler/gates.toml`) MUST itself clear the currently-required checks at the target ref before it can land.** The required-check config is **self-protecting**: you cannot weaken the checks that judge a merge in the same merge those weakened checks would wave through, because the _pre-change_ required set (read at the target ref) still governs that landing.

This is the same shape as governance's "a change to the ref-pin / authority config must clear the pre-change gate." Because both the check definitions and the `[[required_check]]` policy are read **at the target ref** (ref-pinned — `merge_gate.rs:211` pattern), a _feature-head_ edit is already inert against the merge that head requests. The bootstrap invariant extends that to the _config-change merge itself_: the diff that loosens the gate is judged by the gate as it stood before the diff. The honest residual (named in R11, now **High**): a same-ref admin who _also_ reaches the fence (governance R10's class) remains a named accepted residual — but the self-protecting config removes the "silently weaken the check, then merge under the weakened check" path that would otherwise make R11 trivial. The build MUST implement the bootstrap invariant as the closure for R11 and MUST NOT present the ref-pin alone (feature-head inertness) as sufficient.

## Trust boundary — who is controlled

|          | The trusted producer (default executor + its signing key)                             | The gated agent                                                                                |
| -------- | ------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| Trust    | **Trusted / uncontrollable** — runs in the human's daemon/CLI; holds the producer key | **Semi-trusted** — bound at the merge gate by what the producer signed                         |
| Enforced | **Outcomes only** — it produces signed results; the read-only gate trusts them        | **The merge** — blocked unless every required check has a signed `success` at the current head |
| Site     | the trusted daemon/CLI; the ledger recording path                                     | the merge gate's read-only required-checks clause                                              |

A "superuser orchestrator" that runs the executor itself is the user's prerogative — same trust class as the human; under DECISION A it is exactly this orchestrator that runs `on-merge-attempt` checks before calling merge. The point is that the **agent it drives** cannot mint a green check: it neither holds the producer key nor can write the ledger through a governed path, so its only route to a satisfying result is to actually make the trusted executor run the check and have it genuinely conclude `success` at the current head — and it cannot loosen which checks are required without first clearing those same checks (Stance 9).
