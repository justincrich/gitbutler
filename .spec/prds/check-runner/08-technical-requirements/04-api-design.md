---
stability: CONSTITUTION
last_validated: 2026-06-26
prd_version: 1.2.0
---

# 04 — API Design

## §1 — The three operations

| Operation | Surface | Trust |
|-----------|---------|-------|
| **Run** (produce) | `but_checks::run_check` / `but check run` | Side-effectful producer; conclusion from real exit |
| **Record** | `but_checks::record_result` (append a `CheckResult`) | Append-only; no UPDATE |
| **Consume** (gate) | the required-checks clause in `enforce_merge_gate` | **Read-only**; pure evaluator |

## §1a — Mechanism-agnostic gate entry point (v1 requirement)

**The shipped gate entry is forge-review-keyed and cannot gate a local merge.**
`enforce_merge_gate(ctx, review_id: usize)`
(`crates/but-api/src/legacy/merge_gate.rs:75`) looks up a `ForgeReview` from the
local forge cache (`review_for_id`, merge_gate.rs:230), derives
`source_branch`/`target_branch` from it, and resolves `current_head_oid` from
`review.source_branch` (merge_gate.rs:78). Its only non-test callers are the
forge PR-merge path (`crates/but-api/src/legacy/forge.rs:1251/1281/1294`). Governance's GOV-LOCAL work shipped the **local** merge gate and local review-verdict persistence (`local_review_verdicts`, read at merge_gate.rs:241-252) around this entry, **but the entry is still keyed on `review_id`**. **A
purely-local virtual-branch / worktree / plain-git `but merge` with no PR has no
`review_id` and cannot reach the required-checks clause** — which would void the
"gate a local `but` merge, mechanism-agnostic across virtual branches AND
worktrees" thesis.

**v1 requirement:** generalize the gate entry so the required-checks evaluation
runs on a resolved `(source_ref, target_ref)` pair, with the head OID resolved by
a **mechanism-agnostic `gix` ref-peel** (the `find_reference → peel_to_id →
to_string()` body at merge_gate.rs:254-261), **not** from a `ForgeReview`. The
shape:

```rust
// NEW mechanism-agnostic entry — no mandatory forge review_id.
// The existing enforce_merge_gate(ctx, review_id) becomes ONE caller that
// resolves (source_ref, target_ref) from the ForgeReview, then delegates here.
pub fn enforce_merge_gate_for_refs(
    ctx: &but_ctx::Context,
    source_ref: &str,                 // caller-supplied; NOT a ForgeReview field
    target_ref: &str,                 // caller-supplied; NOT a ForgeReview field
) -> anyhow::Result<()>;              // head OID peeled via gix (merge_gate.rs:254-261)
```

- The **local `but merge` path** calls `enforce_merge_gate_for_refs` directly with
  the refs it already has, with **no** forge review.
- The existing **forge path** keeps `enforce_merge_gate(ctx, review_id)` as a thin
  caller that resolves `(source_ref, target_ref)` from the `ForgeReview` and
  delegates to `enforce_merge_gate_for_refs`.
- This is **R-ENTRY (Blocking, 08)**: it must be built explicitly because the
  shipped entry is forge-`review_id`-only.

## §2 — The run → record → consume sequence (`on-merge-attempt`)

**DECISION: `on-merge-attempt` is an orchestrator / CLI pre-merge step, not a
gate trigger.** The gate stays read-only and never runs a check. The expensive
checkout + run happens *before* the gate is invoked; the gate only reads the
recorded result.

```text
A human/agent initiates a merge of source → target
        │
        ▼
[PRE-MERGE STEP]  (orchestrator / `but` merge CLI path — NOT inside the gate)
   for each [[required_check]] on target whose trigger == on-merge-attempt:
        but check run <name> --head <current_head_oid>
          ├─ checkout.rs: materialize current_head_oid in an ISOLATED tree (07)
          ├─ runner.rs:   spawn the command, wait (≤ timeout_seconds)
          ├─ Conclusion::from_exit(real ExitStatus, success_exit_codes)
          └─ recorder.rs: append CheckResult{ name, head_oid, conclusion, … }
        │
        ▼
[GATE]  enforce_merge_gate_for_refs(ctx, source_ref, target_ref)   (read-only consumer)
   // local merge calls this directly (no forge review_id); the forge path
   // delegates here after resolving refs from its ForgeReview (§1a / R-ENTRY)
   ... existing Merge authorization + review clause ...
   required-checks clause (independent of `protected`, 01 §9):
        defs    = but_checks::load_check_defs(repo, target_ref)     // ref-pinned blob
        req     = [[required_check]] for target                     // ref-pinned gates.toml
        head    = gix ref-peel of source_ref                        // merge_gate.rs:254-261 (NOT forge-keyed :78)
        results = check_results.list_for(name, head) for each req
        evaluate_required_checks(req, defs, results, head)          // PURE
          ├─ Ok(())             → proceed
          └─ Err(unmet)         → MergeGateError{ code:"gate.check_required", unmet, …STEER }
        │
        ▼
   merge proceeds iff the gate returns Ok
```

Key property: a merge attempt that reaches the gate with a **missing or stale**
required result is **blocked with a remediation hint** (`but check run <name>
--head <oid>`) — the gate does **not** synchronously run the check itself. The
pre-merge step is where running happens; the gate is where reading happens. This
keeps the read-only consumer fast and deterministic (07 §4).

## §3 — `but-checks` public functions

```rust
// config.rs — ref-pinned definition load (working tree NEVER consulted)
pub fn load_check_defs(
    repo: &gix::Repository,
    target_ref: &str,
) -> anyhow::Result<Vec<CheckDefinition>>;   // config_invalid on malformed/unsatisfiable

// runner.rs — THE producer. The only path that constructs a non-Success
// exit-derived Conclusion, and it does so from a REAL ExitStatus.
pub fn run_check(
    repo: &gix::Repository,
    def: &CheckDefinition,
    head_oid: &gix::oid,         // bound at materialization time
) -> anyhow::Result<CheckResult>;

// recorder.rs — append-only record
pub fn record_result(
    db: &mut but_db::DbHandle,
    result: CheckResult,
) -> anyhow::Result<()>;

// evaluator.rs — PURE; the read-only gate calls this
pub fn evaluate_required_checks(
    required: &[RequiredCheck],
    defs: &[CheckDefinition],
    results_by_name: &BTreeMap<String, Vec<CheckResult>>,
    current_head_oid: &str,
) -> Result<(), Vec<String>>;    // Err = miss-reason tokens
```

## §4 — The negative-space rule (no caller-supplied conclusion)

> There is **no public function** that accepts a caller-supplied `Conclusion` and
> writes a `CheckResult` without a real run.

- `Conclusion` has exactly one constructor that yields `Success`/`Failure`/
  `TimedOut` from outside-the-type input: `Conclusion::from_exit(status, codes)`
  (03 §4), which takes a `std::process::ExitStatus` — a value you can only obtain
  by having actually spawned and awaited a process.
- `record_result` takes a fully-formed `CheckResult`, but the *only* code that
  builds one with an exit-derived conclusion is `run_check`. There is no
  `record_conclusion(name, head_oid, Conclusion::Success)` entry point.
- This is enforced by the **type surface**, and proven by a **behavioral**
  negative test (08 R-LYING): there is no API by which an agent can stamp a
  `Success` row for a check it did not run. (A direct DB write can still
  fabricate one — and that is fine, because it is reproducible: re-running
  catches it, 01 §3.)

## §5 — Denial contract (STEER) — **landed (governance closed)**

**Landed:** the four steering fields exist on `MergeGateError` today — `class`/`held_permissions`/`authorized_actions`/`do_not` (verified at merge_gate.rs:45-56) — alongside the legacy `code`/`message`/`remediation_hint`/`unmet`. The carrier serializes (via `#[derive(Serialize)]`) to the **uniform STEER envelope**: the same key set `but_authz::to_envelope` emits for `Denial` (`crates/but-authz/src/denial.rs`) plus the merge-only `unmet`, proven in `crates/but-api/tests/steer_envelope.rs`. This shipped with governance (now closed) in commit `353bbcdc1a`, an ancestor of `master`. The check clause therefore **sets the STEER fields directly** — there is no remaining dependency to wait on. The `gate.check_required` denial below is still **Check Runner's own new code**, but it now populates a carrier whose STEER fields already exist; a legacy reader of the four base fields sees no regression.

The check clause sets:

```rust
MergeGateError {
    code: "gate.check_required",
    message: format!("required checks for {target} are not satisfied: {}", unmet.join("; ")),
    remediation_hint: "run the missing checks at the current head before merging".to_owned(),
    unmet,                                   // Vec<String> of miss-reason tokens
    // --- STEER fields (STEER-001) ---
    class: DenialClass::ActorCorrectable,    // the actor can fix it by running the check
    held_permissions,                        // copied through, like the review clause
    authorized_actions: vec![AuthorizedAction {
        command: "but check run <name> --head <oid>",
        effect:  "produce the missing check result",
    }],
    do_not: None,
}
```

### Miss-reason tokens (carried in `unmet`)

| Token | Meaning | `class` |
|-------|---------|---------|
| `check_missing` | No recorded result for a required check at the current head | `ActorCorrectable` |
| `check_failed` | Latest result at the current head is non-`Success` | `ActorCorrectable` |
| `check_stale_at_head` | A result exists but only at a prior OID (head moved) | `ActorCorrectable` |
| `config_invalid` | Malformed `.gitbutler/checks/*.toml` or unsatisfiable `[[required_check]]` at the target ref | **`OperatorRequired`** |

`config_invalid` is `OperatorRequired` because a malformed/unsatisfiable check
*definition* is operator-owned (committed governance config), not something the
acting agent can fix by running a check — matching governance's treatment of
`config.invalid` (merge_gate.rs:369).

### Dual-audience rendering

```jsonc
// but check ... --json  AND  the gate denial envelope.
// The class/held_permissions/authorized_actions keys are present NOW: MergeGateError
// carries the STEER fields (merge_gate.rs:45-56) and serializes to the uniform envelope
// (STEER landed, governance closed) — not a base-keys-only fallback.
{
  "code": "gate.check_required",
  "message": "required checks for main are not satisfied: cargo-test: check_missing",
  "remediation_hint": "run the missing checks at the current head before merging",
  "unmet": ["cargo-test: check_missing"],
  "class": "actor_correctable",
  "held_permissions": ["contents:write"],
  "authorized_actions": [
    { "command": "but check run cargo-test --head <oid>", "effect": "produce the missing check result" }
  ]
}
```

Human CLI text renders the same facts as a readable block (exit code 1 on every
rejection, parity with the governance gate CLIs).

## §6 — `but check` CLI verbs (argument shapes)

```text
but check define <name> --command <cmd> [--arg <a>]... [--trigger on-merge-attempt|on-commit]
                        [--success-exit-code <n>]... [--timeout-seconds <n>]   # writes/validates .gitbutler/checks/<file>.toml
but check list            [--ref <target>] [--json]                            # defined checks + required flag
but check run <name>      [--head <oid>]  [--json]                             # produce + record (default --head = current)
but check results [<name>][--head <oid>] [--json]                             # recorded results, latest-first
but check required        [--ref <target>] [--json]                           # the [[required_check]] set
```

- `--head` defaults to the resolved current head OID via a **mechanism-agnostic
  `gix` ref-peel** (the `find_reference → peel_to_id → to_string()` body at
  merge_gate.rs:254-261), resolving the caller's source ref directly — **not** via
  the forge-coupled call site at merge_gate.rs:78, which feeds the peel
  `review.source_branch` (a `ForgeReview` field). The standalone runner has no
  `ForgeReview`, so it must peel a non-forge ref it resolves itself (ties to §1a /
  R-ENTRY). With that, `but check run cargo-test` runs against the current head and
  binds the result to it.
- `but check define` only *writes/validates the config file*; it does not commit
  — committing the governance change is a normal governed commit (and is itself
  subject to the bootstrap-invariant, 01 §8).

## §7 — Composition with the governance gate (ordering)

The required-checks clause is added to `enforce_merge_gate` **after** the
existing `Merge` authorization (merge_gate.rs:91) and review clause
(merge_gate.rs:86), and is reached **independent of** the `protected`
early-return (merge_gate.rs:103-109; 01 §9, 08 R-FAILOPEN). Authorization still
gates first (a principal lacking `Merge` is denied `perm.denied` before any check
is consulted); the check clause is an additional merge requirement, not a
replacement.

## Cross-references

- The verdict store + `Conclusion` enum + config schema: [`03-data-schema.md`](./03-data-schema.md)
- Why the checkout (run) is heavy and the gate (consume) is light: [`07-mechanism-agnostic-checkout.md`](./07-mechanism-agnostic-checkout.md) §4
- The STEER denial contract source: governance `sprint-07-steer-capability-aware-denials/STEER-001`
- The capability chain this sequence realizes: [`09-capability-chains.md`](./09-capability-chains.md)
