---
stability: CONSTITUTION
last_validated: 2026-06-20
prd_version: 1.0.0
---

# 01 — Architecture Posture

## The control model in one line

> A required-check is **a second deterministic review whose verdict is produced
> by a local runner instead of a human.** The gate that already blocks a merge
> on a human review verdict blocks it on a runner verdict the same way; the only
> new part is the local producer that *runs a check and records pass/fail at the
> head OID*.

## §1 — Producer / consumer split

Check Runner is two cleanly separated halves bonded by one table:

| Half | What it is | Where it lives | Property |
|------|-----------|----------------|----------|
| **Producer** | The local runner: materialize a clean checkout at the head OID, run the configured command, derive a conclusion from the **real exit code**, append a `CheckResult` row. | `but-checks` (runner + config loader + recorder) | Side-effectful, but **never** trusted by the gate beyond the row it writes. |
| **Consumer** | The required-checks merge-gate clause: read the required set + the recorded results, block unless every required check is `Success` **at the current head OID**. | extended `crates/but-api/src/legacy/merge_gate.rs` | **Read-only.** Never runs a check, never trusts a claim, no caller-supplied conclusion. |

The producer and consumer share **only** the `check_results` table (03) and the
ref-pinned config (`.gitbutler/checks/*.toml` + `[[required_check]]`). They do
not call each other. This is the same shape governance already ships: the human
review producer writes `local_review_verdicts`; `enforce_merge_gate` reads it.

## §2 — Runner ≠ agent (and why hooks cannot be the producer)

The runner is a **separate, butler-controlled** code path from the agent's own
`but commit`. This matters because the conclusion is only meaningful if the
thing that produced it actually ran the command.

GitButler already has command-running code in
`crates/gitbutler-repo/src/hooks.rs` (`pre_commit_with_tree` at `:81`,
`post_commit` at `:121`, `commit_msg` at `:56`). **Hooks cannot be the trusted
producer:**

- They run **inside the agent's own `but commit`** — the agent *is* the caller,
  so a hook conclusion is an agent-produced claim, not an independent one.
- They are **`--no-hooks`-bypassable** at the commit boundary — the agent can
  simply not run them.
- They run against whatever tree the commit path hands them, not against a clean
  checkout of the head OID the gate will match.

So the runner is a distinct `but-checks` entry point (`but check run`,
04 §3) — adjacent to hooks as prior art for *how to spawn a command*, but
deliberately not the same trust path.

## §3 — Why reproducibility is the security basis (not cryptography)

The honest path must be cheaper than cheating. We achieve that with
**reproducibility**, not signing:

- A check is **re-runnable**. Anyone (the reviewer, a second runner, CI) can
  re-run `but check run <name> --head <oid>` and get the same conclusion,
  because the command is ref-pinned and the checkout is the exact OID.
- Therefore a **forged green is detectable by re-running**, and the honest path —
  letting the runner run the check — is cheap.

  **The *merge-time* guarantee is narrow** — a `success`-concluded row
  **exists** and is **head-OID-matched** — NOT that no forgery occurred.
  Reproducibility makes a forged green **detectable by a later re-run**, which
  happens **after** the merge has landed; pre-merge, the gate cannot distinguish
  a forged row from a genuine one. Post-merge detection requires an active
  re-run policy that is **out of scope** for v1.
- The "honest path is cheap" deterrent holds for checks whose honest-run time is
  short — **fast, deterministic** checks. For a **slow** check (e.g. a long
  `cargo test`) or a **flaky** check, the honest run can cost more than forging a
  row, and the deterrent then rests on the **structural locks** (the small
  gate-read→merge-commit window, the bootstrap-invariant, the no-caller-conclusion
  negative space) rather than on cost alone.
- This is the real contrast with governance's `local_review_verdicts`: a *human*
  review is **not reproducible** (you cannot re-derive a human's opinion). A
  check **is** reproducible. That makes `check_results` **safer in one
  dimension** — reproducibility enables post-merge lie-detection — but **weaker
  in another**: a `check_results` row carries **no principal identity**, so
  (unlike forging a human review, which requires impersonating a named reviewer)
  forgery is **not deterred by attribution**, only by reproducibility + the
  structural locks. So it is **safer in detectability, not strictly safer across
  all axes** — which is why it needs **no extra cryptographic protection**, but
  also why the merge-time guarantee is stated narrowly above.

Governance accepts `local_review_verdicts` as forgeable-by-direct-DB-write (its
R6) and still ships it as a plain table. A check is a *reproducible* second
review — forgery of it is **detectable** (post-merge), where forgery of a human
verdict is not. By consistency, `check_results` is correctly a **plain `but-db`
table** (03 §1): it is no worse-protected than the review store governance
already ships, and on the detectability axis it is better.

We therefore **drop**, as v1 security mechanisms: signing / HMAC /
Ed25519, producer signing keys / `but-secret` key storage, agent-unwritable
hardening, and OS-sandbox-as-security. (See 06 — zero new crypto deps; 08
R-FORGERY — the forgery risks are *deliberately not closed* under this threat
model, not owed debt.)

## §4 — SHA-binding: current head only

A `CheckResult` is bound to the OID it ran against (`head_oid`, 03 §1). The gate
matches on **`(name, head_oid == current_head)`** — never on `name` alone.

- `current_head_oid` is resolved by the same `gix` ref-peel the merge gate uses
  (`crates/but-api/src/legacy/merge_gate.rs:172-179` —
  `find_reference` → `peel_to_id` → `to_string()`). **Coupling caveat:** the
  shipped `current_head_oid` call site (merge_gate.rs:78) feeds it
  `review.source_branch` — a **`ForgeReview` field**. The runner's standalone
  `--head` default and the mechanism-agnostic gate entry (§ below; 04, R-ENTRY)
  must instead peel a `(source_ref, target_ref)` pair resolved **without** a
  `ForgeReview` — i.e. call the same `find_reference → peel_to_id` body
  (merge_gate.rs:172-179) on a ref the caller supplies, not on a forge field. The
  peel **helper** is reusable; its forge-keyed **caller** is not.
- A result recorded at OID `A` does **not** satisfy a required check once the
  head moves to `B` (rebase, amend, new commit). The basis for "the head moved"
  is `but_rebase::graph_rebase::Editor::commit_mappings()`
  (`crates/but-rebase/src/graph_rebase/mod.rs:479`, the old→new OID map). A stale
  result yields miss-reason `check_stale_at_head` (08 R-SHARESET).

This is the same staleness property the review requirement already has
(`approval_stale_at_head` in the review evaluator) — a check inherits it.

## §5 — The `signature` column is a forward-compat seam, not a v1 claim

`check_results.signature` is **nullable** and **always `NULL` in v1**. It exists
so that, the day producers run off-host (a remote runner, a shared CI box where
direct-DB-write is no longer the cheap attack), a producer-identity proof has a
place to live without a migration. **The v1 gate ignores it entirely.** It is
documented here and in 03 §1 as explicitly **not** a v1 security control, to
prevent a future reader mistaking its presence for cryptographic enforcement.

**Reconciliation with R-FORGERY ("deliberately not closed", 08).** The seam is a
forward-compat hook for the **off-host-producer** scenario — **not** a planned
closure of the forgery risk under the **current shared-OS-user threat model**.
Under that model, an agent sharing the OS user can write the row directly, so a
producer signature would not change the gate's merge-time decision and is not
owed work; the forgery risk is **deliberately left open** (R-FORGERY) and
absorbed by reproducibility, **not** scheduled to be sealed by this column. The
column only becomes load-bearing if and when the producer moves off-host, where
direct-DB-write stops being the cheap attack.

## §6 — Fail-closed

Every ambiguity resolves to *block the merge*:

- Malformed `.gitbutler/checks/*.toml` or an unsatisfiable `[[required_check]]`
  (naming a check that does not exist) → `config_invalid`, denial returned
  (parity with merge_gate.rs `config_invalid` at `:369` and
  `undefined_required_groups` at `:181-188`).
- A required check with **no recorded result at the current head** → blocked
  (`check_missing`).
- A required check whose latest result at the current head is non-`Success`
  (including `Neutral`/`Skipped` — a required check that no-ops does **not**
  pass) → blocked (`check_failed`).
- An unresolvable target ref → treated as governed so the loader classifies the
  fault (parity with `governance_present`, `crates/but-authz/src/config.rs:53-67`,
  which returns `true`/governed on an unresolvable ref).

## §7 — Composition with the governance merge gate

The required-checks clause is added **inside** `enforce_merge_gate`, after the
existing review-requirement clause, reusing its machinery:

- Same opt-in discriminator: governance present on the **target ref** (the
  clause only runs on a governed target).
- Same ref-pinned config read: `read_config_blob`
  (merge_gate.rs:211-242) for `.gitbutler/checks/*.toml`, and the
  extended `GatesWire`/`normalize_gates` (merge_gate.rs:308-343) for
  `[[required_check]]`.
- Same denial carrier: **`MergeGateError`** (merge_gate.rs:19), which **today
  carries only** `code`/`message`/`remediation_hint`/`unmet: Vec<String>`
  (verified at merge_gate.rs:19-29). **EXTENDS — requires governance STEER-001
  (sprint-07, `STATUS: Backlog`, NOT yet merged) to first land the four steering
  fields `class`/`held_permissions`/`authorized_actions`/`do_not` + `to_envelope()`
  on `MergeGateError`.** Until STEER-001 lands, the carrier exposes only
  `code`/`message`/`remediation_hint`/`unmet`. Once it lands, the check clause sets
  `code: "gate.check_required"` and reuses the STEER fields (04 §5). The GATE
  group therefore sequences **after** STEER-001 (README "Dependencies").
- Same classification path: `classify_error` (merge_gate.rs:113) downcasts the
  carrier out of the `anyhow` chain unchanged.

A required check and a required review are siblings: both are merge requirements
on the same target, both ref-pinned in `.gitbutler/gates.toml`, both evaluated by
the same read-only gate.

## §8 — Bootstrap-invariant (self-protecting required set)

The required-check set protects **its own weakening**. A commit that removes or
weakens a `[[required_check]]` (or a `[[check]]` it depends on) is itself a code
change to a governed target, and must therefore clear the **currently-required**
checks before it can land. Ref-pinning the config (reading it from the committed
target tree, never the working tree) is **necessary but not sufficient**: the
gate must evaluate the required set that is in force *before* the weakening
commit, not the set the weakening commit proposes. (08 R-BOOTSTRAP; UC-DEFN-05,
UC-GATE-05.)

## §9 — Required-checks must be consulted independent of the `protected` flag

`enforce_merge_gate` has an early-return: if the target branch is not flagged
`protected`, it returns `Ok(())` at
`crates/but-api/src/legacy/merge_gate.rs:50-56` **before** the review-requirement
clause. A required-check clause placed after that early-return would be a
**fail-open hole**: a branch that carries `[[required_check]]` but is not flagged
`protected` would skip the check entirely.

**Control-flow requirement:** the required-checks clause must be reached for any
target that carries a `[[required_check]]`, independent of `protected`. The
clause is consulted **before** (or the early-return is restructured to not
precede) the required-checks evaluation. This is a Blocking control-flow risk
(08 R-FAILOPEN, UC-GATE-05).

## §9a — The shipped gate entry is forge-review-keyed (mechanism-agnostic entry is a v1 requirement)

The SHIPPED gate entry point is `enforce_merge_gate(ctx, review_id: usize)`
(`crates/but-api/src/legacy/merge_gate.rs:40`). It looks up a `ForgeReview` from
the local forge cache (`review_for_id`, merge_gate.rs:148), derives
`source_branch`/`target_branch` from it, and resolves `current_head_oid` from
`review.source_branch` (merge_gate.rs:78). Its only non-test callers are the
forge PR-merge path (`crates/but-api/src/legacy/forge.rs:607/637/650). **A
purely-local virtual-branch / worktree / plain-git merge with no PR has no
`review_id` and cannot reach the clause at all** — which would silently void the
mechanism-agnostic local-gating thesis.

**v1 requirement (R-ENTRY, Blocking, 08; specified in 04):** a
**mechanism-agnostic gate entry point** that runs the required-checks evaluation
on a resolved `(source_ref, target_ref, head_oid)` triple **without a mandatory
forge `review_id`** — head OID resolved via the `gix` ref-peel
(merge_gate.rs:172-179), not from a `ForgeReview`. The existing forge path remains
one caller of the shared evaluation. This must be built explicitly because the
shipped `enforce_merge_gate` is forge-`review_id`-only today.

**Auth invariant (MUST NOT regress) — the new entry runs the same `Merge`
authorization precondition the forge path enforces.** The shipped
`enforce_merge_gate` authorizes **before** any review/checks clause and **before**
the `protected` early-return: it resolves the principal
(`resolve_principal_from_env`, merge_gate.rs:47) and calls
`but_authz::authorize(&principal, Authority::Merge, &config.gov)?`
(**merge_gate.rs:48** — the single `Merge`-authority call site, verified against
live source). The new `enforce_merge_gate_for_refs(ctx, source_ref, target_ref)`
local-merge entry MUST run that identical authority clause (plus the same review +
required-checks clauses) on the config it loads from the **target ref**.
Generalizing the entry to caller-supplied refs MUST NOT skip the
`authorize(_, Authority::Merge, _)` precondition the forge path enforces — a
refs-resolving caller that resolves `(source_ref, target_ref)` but bypasses the
`Merge` authorization would open an **authz bypass**, not merely a checks bypass.

## §10 — What the gate proves (the narrow, honest line)

> The gate proves: **the committed check ran under the trusted runner and exited
> 0 at the current head OID.**

It does **not** prove the code is correct, and it is **not** "non-fakeable." A
local-DB-write can fabricate a green — and that is fine, because the green is
reproducible: re-running catches the lie and the honest path is cheap. Stating
the guarantee narrowly is itself a control: it stops anyone downstream
over-trusting the result.

## Trust boundaries

| Boundary | Trusted? | Enforcement |
|----------|----------|-------------|
| Ref-pinned config (`.gitbutler/checks/*.toml`, `[[required_check]]`) read from the **committed target tree** | Trusted | `read_config_blob` (merge_gate.rs:211); working tree never consulted |
| The runner's recorded `conclusion` | Trusted *as a reproducible claim* | Derived from a real `ExitStatus`; re-runnable; no caller-supplied conclusion (04 §4) |
| The agent's live/dirty shared worktree | **Not** the check surface | The runner checks out the head OID into an **isolated** materialization (07); never runs against the shared tree |
| `signature` column | **Ignored in v1** | Nullable forward-compat seam (§5) |
| Forge / push path | Out of scope (Accepted) | Inherits governance's forge/raw-push merge-bypass posture (08 R11/R14); the local gate binds the governed local merge only |

## Cross-references

- The verdict store and config schema: [`03-data-schema.md`](./03-data-schema.md)
- The run → record → consume sequence and negative-space rule: [`04-api-design.md`](./04-api-design.md)
- The clean-checkout problem (§ this file's "shared worktree is not the check surface"): [`07-mechanism-agnostic-checkout.md`](./07-mechanism-agnostic-checkout.md)
- The re-ranked risk register: [`08-technical-risks.md`](./08-technical-risks.md)
