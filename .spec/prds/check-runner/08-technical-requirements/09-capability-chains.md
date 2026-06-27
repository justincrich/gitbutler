---
stability: CONSTITUTION
last_validated: 2026-06-26
prd_version: 1.2.0
---

# 09 — Capability Chains

Each capability is a chain of hops crossing real boundaries, with a real-service
proof at the end. Check Runner adds the **execution axis** that composes with
governance's authorization chains (`CAP-AUTHZ-01` / `CAP-CONFIG-01`).

## CAP-CHECK-01 — run → record → consume → deny+STEER → re-run

The core loop: a local runner produces a result bound to the head OID; the
read-only gate consumes it; a failure steers the agent to fix it.

| Hop | From → To | Boundary contract | Failure mode | Real-service proof |
|-----|-----------|-------------------|--------------|--------------------|
| 1. Define | `.gitbutler/checks/*.toml` → `but_checks::load_check_defs` | Ref-pinned blob read from the committed **target** tree (`read_config_blob`, merge_gate.rs:211); working tree never consulted | Malformed/unsatisfiable → `config_invalid` (`OperatorRequired`) | Commit a malformed def; assert `config_invalid` on the gate, not a vacuous pass |
| 2. Run (produce) | `but check run` → `runner.rs` via `checkout.rs` | Materialize the **exact head OID** in an isolated tree (07); spawn the real command; conclusion from real `ExitStatus` | Checkout against the dirty shared tree (R-CHECKOUT); timeout → `timed_out` | Run with a dirty shared worktree at a different OID; assert the check ran against the requested OID's tree and the shared tree is unchanged |
| 3. Record | `runner.rs` → `check_results` (but-db) | Append-only `CheckResult` keyed `(name, head_oid)`; `signature` NULL | — (no UPDATE path) | Inspect the row; assert `head_oid` == the run OID, conclusion == exit-derived |
| 4. Consume (gate) | `enforce_merge_gate` → `evaluate_required_checks` (PURE) | Read-only; match `(name, head_oid == current)`; reached **independent of `protected`** (R-FAILOPEN) | Missing → `check_missing`; non-`Success` → `check_failed`; stale → `check_stale_at_head` | Merge attempt with a missing required result is blocked `gate.check_required`; the gate is **read-only at code level** — `evaluate_required_checks` calls **no runner entry point** and accepts **no caller-supplied `Conclusion`** (see Critical proofs (4) for the code-level vs. OS-instrumentation distinction) |
| 5. Deny + STEER | gate → `MergeGateError` → CLI / `to_envelope` | `code:"gate.check_required"` + `unmet` (base fields, present today) **+** STEER fields (`class: ActorCorrectable`, `authorized_actions: [but check run …]`) — **the STEER fields + `to_envelope()` are LANDED (governance closed; on `MergeGateError` at merge_gate.rs:45-56), so the clause carries them directly** | Denial missing steering fields | Assert the JSON envelope carries `class`/`authorized_actions` (live now — STEER landed); exit 1 |
| 6. Re-run | agent reads `authorized_actions` → `but check run <name> --head <oid>` → hop 2 | The remediation command is the exact producer entry point | Agent cannot fabricate a pass via an API (R-LYING) | Behavioral negative test: no API stamps `Success` without a run; after a real re-run the merge proceeds |

- **Owner:** `rust-implementer` (build) → `rust-reviewer` + `security-auditor`
  (review the read-only/negative-space/fail-closed properties).
- **Composition:** terminates at the same read-only `enforce_merge_gate` as
  governance; `Merge` authorization (`CAP-AUTHZ-01`) still gates first, the check
  clause is an additional requirement.
- **Critical real-service proofs (must be behavioral, not grep):**
  - (4) the gate is **read-only** — assert it at the **code level**:
    `evaluate_required_checks` is a pure function over `(req, defs, results,
    head)` that **calls no runner entry point** (`but check run` / `runner.rs`)
    and **accepts no caller-supplied `Conclusion`**, so consuming a result cannot
    execute a check. This is the behavioral/negative-path assertion the gate
    actually supports. **Caveat:** literally *observing* "no child process was
    spawned" during `enforce_merge_gate` requires **OS-level instrumentation
    (ptrace / seccomp), which v1 does not specify** — so the proof is the
    code-level "no runner call site + no caller-conclusion" assertion, not a
    process-count assertion.
  - (R-FAILOPEN) a non-`protected` target carrying a required check still blocks.
  - (R-LYING) **negative**: there is no caller-supplied-conclusion API path.

## CAP-CHECK-02 — SHA-reset invalidation

A result is valid only at the OID it ran against; moving the head invalidates it.

| Hop | From → To | Boundary contract | Failure mode | Real-service proof |
|-----|-----------|-------------------|--------------|--------------------|
| 1. Pass @ A | `but check run --head A` → `check_results{name, A, Success}` | Result bound to OID `A` | — | Row exists at `A` |
| 2. Head moves A → B | rebase/amend → `Editor::commit_mappings()` (graph_rebase/mod.rs:479) | `current_head_oid` now resolves to `B` (merge_gate.rs:78) | — | `current_head_oid` == `B` |
| 3. Evaluate @ B | `evaluate_required_checks(req, defs, results, head=B)` | Match on `(name, head_oid==B)`; the `A` result does not count | Matching on `name` alone (R-SHARESET) would falsely pass | Merge @ `B` blocked `check_stale_at_head` |
| 4. Re-run @ B | `but check run --head B` → `check_results{name, B, Success}` | New result bound to `B` | — | Row exists at `B` |
| 5. Evaluate @ B | gate consumes the `B` result | Passes | — | Merge @ `B` proceeds |

- **Owner:** `rust-implementer` → `rust-reviewer`.
- **Proof:** the `Success@A` → rebase→`B` → `check_stale_at_head` → re-run@`B` →
  allow sequence, end to end, against real git (`but-testsupport`).

## CAP-CHECK-03 — bootstrap-invariant (self-protecting required set)

Weakening the required set is itself gated by the required set in force.

| Hop | From → To | Boundary contract | Failure mode | Real-service proof |
|-----|-----------|-------------------|--------------|--------------------|
| 1. Propose weakening | commit removes a `[[required_check]]` on target | The change is a code change to a governed target | — | Diff removes the requirement |
| 2. Evaluate before-state | gate evaluates the required set **in force before** the weakening commit | Ref-pin is necessary but not sufficient — evaluate the pre-change set (R-BOOTSTRAP) | Evaluating the *proposed* (weakened) set would let it self-escalate | Weakening commit blocked until currently-required checks pass |
| 3. Land after passing | currently-required checks pass → weakening lands | — | — | After the required checks pass, the weakening merges |

- **Owner:** `rust-implementer` → `security-auditor` (self-escalation is a
  security property).
- **Proof:** integration test — a required-set-weakening commit is blocked until
  the currently-required checks pass; the negative control (ref-pin only, no
  before-state evaluation) lets the self-escalation through.
- **Limitation (state it alongside the property — do not over-claim):** the
  bootstrap-invariant forces the weakening change to **satisfy the checks it
  removes**, but does **NOT** force human review of the weakening *itself*. An
  agent can run the currently-required checks honestly, then land code **plus** a
  required-set weakening in the **same** change — the gate evaluates the
  pre-change set it just satisfied, and nothing in the engine demands a human sign
  off on the policy diff. The honest companion is the
  [`01-scope.md`](./01-scope.md) Known Limitations row **"No human-in-the-loop is
  forced for a required-set change"**: the config path is `administration:write`-
  gated and the diff is reviewable, but forcing human review of a gate-weakening
  diff is a **governance**-side policy concern (deferred there), consciously not
  re-specified here. Reference it so this property is not presented as more
  protective than it is — the bootstrap-invariant is the engine leg, not a
  human-in-loop guarantee.

## Composition map

```text
   CAP-AUTHZ-01 (governance: authorize Merge) ──┐
   CAP-CONFIG-01 (governance: ref-pinned config)─┤
                                                 ├──► enforce_merge_gate (READ-ONLY)
   CAP-CHECK-01 (run→record→consume→steer)  ─────┤        Ok ⇒ merge
   CAP-CHECK-02 (SHA-reset invalidation)    ─────┤        Err ⇒ exit 1 + STEER
   CAP-CHECK-03 (bootstrap-invariant)       ─────┘
```

All chains terminate at one deterministic, read-only consumer. The producer side
(`but-checks`) is the only side-effectful part, and it is trusted only as far as
its **reproducible** recorded row (01 §3).

## Cross-references

- The run/record/consume sequence: [`04-api-design.md`](./04-api-design.md) §2
- The checkout hop's hard problem: [`07-mechanism-agnostic-checkout.md`](./07-mechanism-agnostic-checkout.md)
- The risks each hop guards against: [`08-technical-risks.md`](./08-technical-risks.md)
- The STEER denial at hop 5: governance `sprint-07-steer-capability-aware-denials/STEER-001`
