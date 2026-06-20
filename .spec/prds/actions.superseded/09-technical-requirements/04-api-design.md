---
stability: CONSTITUTION
last_validated: 2026-06-19
prd_version: 1.0.1
section: technical-requirements
---
# API Design

The surface is **CLI-first** (`but check …`, defined in `crates/but/src/command/` + `crates/but/src/args/`) over a **core executor + verifier API** (`but-checks`) wired into the merge gate's **read-only** required-checks clause and the `statuses:write`-gated producer-trigger boundary (`crates/but-api/src/legacy/checks.rs`). No new daemon, no new HTTP, no MCP, **no runner protocol** (no-broker v1, Stance 2). It composes with — and reuses — governance's denial contract.

> **DECISION A — the gate is read-only; `on-merge-attempt` is an orchestrator step.** `enforce_merge_gate` **never invokes the executor.** When a check is declared `on-merge-attempt`, the **trusted CLI/daemon orchestrating the merge** runs it via `but check run` *immediately before* calling the merge action; the gate then consumes the freshly-produced current-head rows. The producer (run+record) and the consumer (read+decide) are two distinct synchronous phases the orchestrator sequences. See the orchestration sequence below.

> **Naming:** crate `but-checks`, CLI noun `but check`, table `check_results` — distinct from the pre-existing `butler_actions` feature (see `02-system-components.md`).

## The core verifier API (`but-checks`) — what the gate calls (read-only)

```rust
// but-checks — the pure, READ-ONLY required-checks verifier the merge gate consumes.
// It NEVER runs a check. It takes already-fetched, already-signature-checked rows and decides.
pub fn required_checks_satisfied(
    required: &[CheckName],          // from .gitbutler/gates.toml [[required_check]] at the TARGET ref
    results_at_head: &[CheckResult], // ledger rows for (target, current_head_oid), signatures already verified
    trusted_producer: &ProducerIdentity, // the butler producer the gate trusts
) -> Result<(), CheckGateError>;     // Ok(()) → clause passes; Err → gate.check_required, exit 1

pub struct CheckGateError {          // shares the MergeGateError shape (merge_gate.rs:20)
    pub code: &'static str,          // "gate.check_required"
    pub message: String,             // names the unmet checks
    pub remediation_hint: String,    // "run the required checks at the current head, then merge"
    pub unmet: Vec<String>,          // ["cargo-test: check_missing", "clippy: check_failed (failure)", ...]
}

// signature verification is applied BEFORE results reach the verifier:
pub fn verify(result: &CheckResult, trusted: &ProducerIdentity) -> bool; // false → treat as absent
```

The verifier is **pure** (no I/O, no subprocess): it takes already-fetched, already-signature-checked rows and decides. This mirrors `but-api`'s `review_requirement::evaluate` (`review_requirement.rs:37`) — a pure evaluator the gate calls after fetching rows. The unmet **reasons** are stable `check_*` tokens — the **same shape** as the review evaluator's `no_approval` / `approval_stale_at_head` (`review_requirement.rs:11-17`), renamed for the check domain so the gate emits **one** miss-reason vocabulary that matches the spine and the wire `unmet[]` array (canonical: `check_missing` / `check_stale_at_head` / `check_failed` / `check_unverifiable`):

| Unmet reason | Meaning |
|---|---|
| `check_missing` | the required check has no verifying result at the current head OID (none ran, or only stale-head results exist) |
| `check_stale_at_head` | a verifying `success` exists, but only at an older head OID (the SHA-reset case — history moved after the check ran) |
| `check_failed` (`failure` / `cancelled` / `timed_out`) | a verifying result exists at head but its conclusion does not satisfy — the message names the specific terminal conclusion |
| `check_unverifiable` | a result row exists at head but its signature does not verify against the trusted producer |

## The producer-trigger API (`but-api`) — `statuses:write`-gated

Running a check (invoking the executor to run-and-record) is gated by `statuses:write` — the authority already in the `but-authz` catalog (`authority.rs`), previously "catalog-only", now a real gated route. The guard is the pre-call `authorize()` pattern that async forge actions already use (`forge.rs:488` `authorize_branch_action(&repo, &params.source_branch, Authority::PullRequestsWrite)`; the `authorize_branch_action` helper is defined at `forge.rs:47`):

```rust
// crates/but-api/src/legacy/checks.rs (NEW)
pub async fn run_check(ctx: ThreadSafeContext, target: String, name: String) -> Result<CheckResult, Error> {
    let ctx = ctx.into_thread_local();
    let repo = ctx.repo.get()?;
    let target_ref = /* resolve target ref, as command/perm.rs:56 does */;
    // pre-call guard — only a statuses:write principal may ask the executor to run-and-record
    authorize_for_statuses_write(&repo, &target_ref)?;            // mirrors enforce_administration_write_gate (config_mutate.rs:18)
    // the executor runs IN THIS TRUSTED PROCESS (no broker), then signs + records deterministically.
    // `name` selects WHICH committed check runs; the COMMAND is loaded from the def at the TARGET ref —
    // the caller cannot supply or alter the command (R1), and cannot supply a Conclusion (R6).
    but_checks::executor::run_and_record(&ctx, &target, &CheckName::new(name)).await
}
```

The **recording itself is deterministic engine code** inside `but-checks::executor::run_and_record` — it always signs + records after a run; the agent cannot skip it or substitute a value. The `statuses:write` guard governs *who may trigger* a run; the signature governs *whether the gate trusts the row*. (The agent does NOT need `statuses:write` to make progress through the governed path — the trusted executor, run by the human/orchestrator, holds it; an agent without `statuses:write` simply cannot self-produce results, which is the point.)

### What `statuses:write` does and does NOT grant (finding 4 — precise)
`statuses:write` lets a principal **request that a *named* check run** — and nothing more. It is **not** a fakeability hole and it is **not** the ability to influence what the executor does:

- **The agent may REQUEST (via the orchestrator) which check name runs.** It picks a `name`; the trusted process runs that check.
- **The agent CANNOT influence the command the executor invokes.** The `run` command is loaded from the check definition **at the target ref** (ref-pinned, R1/R11) — not from the caller. Requesting `cargo-test` runs the *committed* `cargo-test` command, whatever the agent's feature head says.
- **The agent CANNOT mint a row.** The signature comes from the producer key the trusted executor holds, not from the trigger principal (trigger-authority ≠ producer-key, below).
- **The real abuse shape is DoS / confusion, not fakeability.** A principal with `statuses:write` could (a) trigger **non-required** checks (checks not in the `[[required_check]]` set) to burn compute / spam the ledger, or (b) trigger repeated runs to contend on the worktree/checkout. This is a **denial-of-service / confusion vector** (run-cost and ledger-noise), explicitly NOT a path to a forged green required check. The build SHOULD bound run frequency / scope `statuses:write` triggers to defined checks; it MUST NOT treat non-required-check injection as a gate-bypass (it is not — only `[[required_check]]` names are consulted by the gate).

| Action route | Required `Authority` | Notes |
|---|---|---|
| run a check (`but check run`) — invoke the executor to run-and-record | `statuses:write` | the producer trigger; requests a *named* check; command is target-ref-pinned; recording is deterministic engine code |
| read results (`but check results`) | `statuses:read` (or self/`contents:read`) | read-only ledger query |
| show required-checks status (`but check required`) | `statuses:read` | read-only; shows pass/unmet per required check at the current head |
| define / edit a check (`but check define`) | `administration:write` | writes `.gitbutler/actions/*.toml` (governed config — same gate as `but perm`, `config_mutate.rs:18`); subject to the bootstrap invariant (Stance 9) |
| list definitions (`but check list`) | `administration:read` (or `statuses:read`) | read-only |
| **merge (the consumer)** | `merge` + review requirement + **required checks** | the gate composes all three; required-checks is the NEW **read-only** clause |

## The merge-gate required-checks clause (the read-only consumer)

The clause is added **inside** `enforce_merge_gate` (`merge_gate.rs:40`), composed in a fixed order with the existing clauses. **It is read-only — it consumes ledger rows and does NOT invoke the executor (DECISION A).**

```rust
pub fn enforce_merge_gate(ctx: &but_ctx::Context, review_id: usize) -> anyhow::Result<()> {
    // ... existing: resolve review, target_ref, repo, load config ...
    let principal = but_authz::resolve_principal_from_env(&config.gov)?;
    but_authz::authorize(&principal, Authority::Merge, &config.gov)?;          // clause 1 (existing, :48)

    // C1 — the existing protected() early-return (:50-56) MUST account for required checks, else a
    // branch with [[required_check]] but NOT flagged `protected` returns Ok here and clause 3 below is
    // NEVER reached (fail-open). Read the required-set FIRST; short-circuit only when neither applies:
    let required = required_checks_for(&config, &review.target_branch);         // .gitbutler/gates.toml [[required_check]] @ target ref
    let is_protected = config.gov.branch(&review.target_branch)
        .is_some_and(|b| b.protected());
    if !is_protected && required.is_empty() {
        return Ok(());                                                          // nothing to enforce (replaces the bare `if !protected { return Ok(()) }`, :50-56)
    }
    // ... existing review-requirement clause (:58–:109) runs when `is_protected` ... // clause 2 (existing)

    // clause 3 — NEW required-checks (Checks), READ-ONLY — runs whenever `required` is non-empty,
    //            INDEPENDENT of the `protected` flag (a branch carrying required checks is gated on
    //            them even if it is not in the governance "protected" set):
    if !required.is_empty() {
        let current_head_oid = current_head_oid(&repo, &source_ref)?;          // reuse :78
        let rows = ctx.db.get_cache()?.check_results()
            .list_by_target_and_head(&review.source_branch, &current_head_oid)?; // ledger @ current head only — a READ
        let verified: Vec<CheckResult> = rows.into_iter().filter(|r| but_checks::verify(r, &trusted_producer)).collect();
        but_checks::required_checks_satisfied(&required, &verified, &trusted_producer)
            .map_err(|e| MergeGateError {                                       // reuse MergeGateError shape (:20)
                code: "gate.check_required",
                message: format!("required checks for {} are not satisfied: {}", review.target_branch, e.unmet.join("; ")),
                remediation_hint: "run the required checks at the current head, then merge".to_owned(),
                unmet: e.unmet,
            })?;
    }
    Ok(())
}
```

The clause **never runs a check.** If a required check has no current-head row, the gate returns `gate.check_required` (it does not "helpfully" run the check). Producing the row is the orchestrator's job, before it calls merge (next section).

**C1 — required checks are NOT gated by the `protected` flag (fail-open closure).** The real governance gate short-circuits with `Ok` when the target branch is not flagged `protected` (`merge_gate.rs:50-56`). Naively appending clause 3 *after* that guard would let a branch carrying a `[[required_check]]` set but not marked `protected` merge with **zero** check enforcement — a fail-open. So the required-set is read **before** the early-return (above), the early-return is taken **only** when the branch is neither `protected` nor carries required checks, and clause 3 runs whenever `required` is non-empty regardless of `protected`. The loader reads `[[required_check]]` independent of protection; a `[[required_check]]` configured on a branch the gate cannot otherwise resolve is a `config.invalid` fail-closed — never a silent un-enforced merge. (Tracked as **R14**.)

| Gate clause | Entry | Checks | On fail |
|---|---|---|---|
| Authority (existing) | `enforce_merge_gate` (`:48`) | acting principal holds `merge` | `{code:"perm.denied"}`, exit 1 |
| Review requirement (existing) | `enforce_merge_gate` (`:86`) | distinct approval @head from required groups | `{code:"gate.review_required", unmet:[…]}`, exit 1 |
| **Required checks (NEW, read-only)** | `enforce_merge_gate` (after `:48`, composed with review) | every required check name has a **verifying `success` at the current head OID** | `{code:"gate.check_required", unmet:[…]}`, exit 1 |

The gate reads the ledger at the **current source head OID only** (the SHA-reset invariant); a `success` produced at an old head does not satisfy — it surfaces as `check_stale_at_head`, the same shape as the review evaluator's `approval_stale_at_head` (`review_requirement.rs:14`). (The read-then-merge window is not atomic — R12; named, deferred closure.)

## The `on-merge-attempt` orchestration sequence (DECISION A — caller-side, not gate-side)

When the trusted CLI/daemon performs a merge, it sequences the producer phase before the consumer phase. The gate is untouched (still the pure read-only consumer); the *orchestrator* is what honors `on-merge-attempt`:

```text
trusted CLI/daemon  `but merge <review>` (or auto-merge orchestration)
  1. resolve target branch + its [[required_check]] set @ target ref
  2. for each required check whose definition has on_merge_attempt = true:
         but check run <name> --target <branch>     # statuses:write-gated; runs in THIS trusted process;
                                                     # signs + records a row @ current head OID
  3. call the merge action (forge merge / set_review_auto_merge)
         └─ enforce_merge_gate runs (READ-ONLY):
              clause 1 authority → clause 2 review → clause 3 required-checks (consumes the rows from step 2)
  4. gate Ok → merge proceeds;  gate Err(gate.check_required) → merge denied, STEER the agent
```

Key properties this preserves:
- **The gate never produces a result** — step 2 (produce) and step 3's clause 3 (consume) are distinct. An agent calling `but merge` cannot smuggle execution into the gate; the gate has only a ledger read.
- **`on-merge-attempt` is advisory to the orchestrator, not a gate trigger** — if the orchestrator (or a non-orchestrating caller) does not run step 2, the gate simply finds no current-head row and denies with `check_missing`. Fail-closed by default.
- **The agent cannot make the orchestrator run a different command** — step 2 loads the command from the target-ref-pinned definition (R1/R11).

## New CLI verbs — `but check …`

Defined in `crates/but/src/command/check.rs` + `crates/but/src/args/check.rs` (mirroring `command/perm.rs` + `args/perm.rs`), **not** `but-clap` (a docs generator).

| Command | Purpose | Gated by |
|---|---|---|
| `but check define <name> --run <cmd> [--satisfying success,neutral] [--timeout 600] [--on-merge-attempt]` | write/update a check definition in `.gitbutler/actions/<name>.toml` (effective once committed to the target ref; weakening it is subject to the bootstrap invariant) | `administration:write` |
| `but check list` | show defined checks (name, run spec, satisfying set, on-merge-attempt) | `administration:read` |
| `but check run <name> [--target <branch>]` | invoke the trusted executor to run-and-record `<name>` at the current head; prints the produced `Conclusion` | `statuses:write` |
| `but check results [--target <branch>] [--head <oid>]` | list recorded results (name, head_oid, conclusion, producer, verified?) | `statuses:read` (or self) |
| `but check required [--target <branch>]` | show, per required check, pass/unmet at the current head (the gate's read-only view) | `statuses:read` |

All write commands print the ref-pin caveat. **Reuse governance's runtime caveat field** — the ref-pin caveat is surfaced as the `outcome.caveat` value the governance CLI already prints (`crates/but/src/command/perm.rs:95`; also `crates/but/src/command/group.rs:107`), e.g. *"takes effect once committed to the target branch."* (There is **no** `REF_PIN_CAVEAT` constant and **no** `governance.rs` module — the caveat is a runtime `outcome.caveat` string, not a compile-time constant; cite `perm.rs:95` / `group.rs:107`, never `governance.rs:17`.)

## The result / denial contract (consistent with governance)

Every denial — `statuses:write` missing, config invalid, required-checks unmet — is the same shape and exit code as governance's, so an agent parses one contract across both systems:

```json
{ "error": { "code": "gate.check_required",
             "message": "required checks for main are not satisfied: cargo-test: check_stale_at_head; clippy: check_missing",
             "remediation_hint": "run the required checks at the current head, then merge",
             "unmet": ["cargo-test: check_stale_at_head", "clippy: check_missing"] } }
```

`code` ∈ `{ perm.denied, branch.protected, gate.review_required, gate.check_required, config.invalid }` — the governance set **plus** `gate.check_required`. Exit code `1` on every rejection. No partial success, no silent skip; a missing/unreadable/malformed check definition or gate config fails closed with `config.invalid` (never an implicit allow — mirrors `merge_gate.rs`'s `config_invalid`, `:369`). **Denial-code meaning:** `config.invalid` = a system/config error requiring operator action; `gate.check_required` / `perm.denied` = user-correctable (run the check / get the authority). The CLI renders denials via the same `governance_cli_error` → `{error:{code,message}}` pattern (`command/perm.rs:118`); `but-checks` errors are classified out of the `anyhow` chain via a `classify_error` downcast, exactly as `merge_gate::classify_error` (`merge_gate.rs:113`) and `config_mutate::classify_error` (`config_mutate.rs:31`) do.

## Producer identity & confinement

- The **trusted producer** identity + signing key are resolved in the trusted process (daemon/CLI) from `but-secret` (keyring) — **never** from an agent-supplied value. (Residual: in the personal-tenant model the keyring is readable by the agent's OS user — R2.)
- The signature is over the canonical `(name, head_oid, conclusion)` tuple; the gate verifies it against the trusted producer's `key_id`. A row with no/wrong signature is treated as **absent** (not a pass).
- The acting *principal* for the `statuses:write` trigger guard is resolved from `BUT_AGENT_HANDLE` (governance's mechanism, `authorize.rs:5`) — but holding `statuses:write` only lets a principal **request a *named* run** (it does **not** let it mint a signed row, nor alter the target-ref-pinned command). This is the key separation: **trigger-authority ≠ producer-key, and request-which ≠ control-what.** An agent that somehow held `statuses:write` still could not forge a green check without the producer key (R2 names the residual where the key is readable by the agent), and could not change the command the check runs (R1/R11).
- **No `--as` / no caller-supplied producer.** As in governance (UC-AUTHZ-03), acting as another producer identity is not accepted; the producer identity is the trusted process's own.
- **No caller-supplied `Conclusion`.** There is no public API that records a conclusion without a run (R6); `run_check` runs-then-records.
