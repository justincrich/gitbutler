---
stability: CONSTITUTION
last_validated: 2026-06-19
prd_version: 1.0.1
section: technical-requirements
---

# Architecture Diagram

> **Naming:** crate `but-checks`, CLI noun `but check`, table `check_results` — distinct from the pre-existing `butler_actions` feature (see `02-system-components.md`).
> **DECISION A:** the merge gate is **read-only** and never invokes the executor. The `but check run` step is sequenced by the **trusted CLI/daemon orchestrator** immediately before it calls the merge action (for `on-merge-attempt` checks); the gate then consumes the current-head rows.

## The producer → ledger → consumer path

```
┌───────────────────────────────────────────────────────────────────────────┐
│ TRUSTED PROCESS  (the daemon / CLI the human or trusted orchestrator runs)   │
│   holds the producer signing key (via but-secret); runs the DEFAULT EXECUTOR │
│   ── this is NOT the agent's process (Stance 1: executor ≠ gated agent) ──    │
│   ── on a merge, it runs on-merge-attempt checks HERE, then calls merge ──    │
└───────────────────────────────┬─────────────────────────────────────────────┘
                                 │ but check run <name>   (statuses:write-gated trigger,
                                 │   sequenced by the ORCHESTRATOR before merge — DECISION A)
                                 ▼
   ┌─────────────────────────────────────────────────────────────────────────┐
   │ DEFAULT EXECUTOR   (but-checks::executor — the PRODUCER)                   │
   │  1. load .gitbutler/actions/<name>.toml  @ TARGET ref (ref-pinned, gix)   │
   │     (caller picks the NAME; cannot supply/alter the COMMAND — R1/R11)      │
   │  2. materialize an ISOLATED checkout @ head OID (one-worktree model — R13) │
   │  3. run the check IN THIS TRUSTED PROCESS   (no broker / no runner — v1)   │
   │  4. derive a typed Conclusion (v1 emits success|failure|timed_out;         │
   │     cancelled|neutral|skipped reserved) — never a free string             │
   │  5. SIGN (name, head_oid, conclusion) with the producer key               │
   │  6. RECORD — deterministic engine code (always happens; agent can't skip)  │
   └───────────────────────────────┬───────────────────────────────────────────┘
                                    │ insert signed row, keyed by head OID
                                    ▼
   ┌─────────────────────────────────────────────────────────────────────────┐
   │ LEDGER   check_results  (NEW but-db table — agent-UNWRITABLE that the gate │
   │   trusts; a forged UNSIGNED direct INSERT → no valid signature → ignored)  │
   │   (id, target, name, head_oid, conclusion, producer_identity, signature,   │
   │    recorded_at, metadata[nullable])                                        │
   │   ✗ NOT ci_checks (disposable remote cache) · ✗ NOT local_review_verdicts  │
   │     (unsigned, forgeable — governance R6, corrected here by signing)        │
   │   ✗ NOT butler_actions (the pre-existing macro table)                       │
   └───────────────────────────────┬───────────────────────────────────────────┘
                                    │ read rows WHERE target=? AND head_oid=<CURRENT head OID>
                                    │ (signatures verified; stale-OID rows invisible; READ-ONLY)
                                    ▼
   ┌─────────────────────────────────────────────────────────────────────────┐
   │ MERGE GATE   enforce_merge_gate  (but-api/legacy/merge_gate.rs:40 — the    │
   │   deterministic, READ-ONLY CONSUMER; it NEVER runs a check)                │
   │     clause 1  authorize(principal, Merge)            (:48, existing)       │
   │     clause 2  review requirement @head, distinct     (:86, existing)       │
   │     clause 3  required_checks_satisfied(...)          (NEW — Checks)       │
   │               every required name has a VERIFYING `success` @ CURRENT head │
   │   proves: "the committed, ref-pinned check ran & exited 0 @ head OID"      │
   │           — NOT "the code is correct", NOT "non-fakeable"                  │
   └──────────────┬──────────────────────────────────────┬─────────────────────┘
                  │ all clauses pass                       │ any clause fails
                  ▼                                        ▼
            ┌───────────┐                       ┌─────────────────────────────┐
            │  ALLOW     │                       │ DENY → {code, message,       │
            │  merge     │                       │ remediation_hint, unmet}      │
            │  proceeds  │                       │ code=gate.check_required, exit1│
            └───────────┘                       │      ── STEER ──              │
              (read→merge NOT atomic — R12)      │ agent reads denial, has the   │
                                                │ orchestrator run the check at  │
                                                │ head, re-attempts             │
                                                └─────────────────────────────┘

  ┌─ SHA-RESET INVARIANT (Stance 5 — load-bearing, butler-specific) ───────────┐
  │  agent rewrites history → but_rebase::graph_rebase::Editor::rebase()        │
  │    materializes NEW commit OIDs (old→new via Editor::commit_mappings,        │
  │    graph_rebase/mod.rs:479).  Head OID moves ⇒ prior ledger rows keyed to    │
  │    the OLD OID no longer match the current head ⇒ AUTOMATICALLY STALE.       │
  │    The gate (reading head_oid == CURRENT) ignores them; the executor must    │
  │    RE-RUN at the new head to produce satisfying results. No explicit         │
  │    "invalidate" write is needed for correctness — staleness is by keying.    │
  │    BUT the read→merge window is NOT atomic (TOCTOU) — head can advance       │
  │    between the gate read and the merge commit (R12; worst on the forge path).│
  └─────────────────────────────────────────────────────────────────────────────┘

  ┌─ orthogonal axes, reused not overloaded ──────────────────────────────────┐
  │  Authorization (governance):  but-authz Authority/AuthoritySet/Denial.      │
  │    statuses:write gates the producer TRIGGER (already in the catalog).      │
  │  Producer KEY (Checks):  but-secret keyring — the root of forgery-hardness. │
  │    trigger-authority ≠ producer-key, request-WHICH ≠ control-WHAT:           │
  │    holding statuses:write lets you ASK the executor to run a NAMED check;    │
  │    only the producer key SIGNS a row the gate trusts, and the COMMAND is     │
  │    ref-pinned, not caller-supplied.                                          │
  └─────────────────────────────────────────────────────────────────────────────┘

  ┌─ self-protecting required-check config (Stance 9 / R11 — the bootstrap inv) ┐
  │  weakening a required check (.gitbutler/actions/*.toml) or the              │
  │  [[required_check]] set MUST itself clear the PRE-CHANGE required checks at  │
  │  the target ref before it can land. You cannot loosen the checks that judge  │
  │  a merge in that same merge. Ref-pinning (feature-head inertness) alone is   │
  │  necessary but NOT sufficient.                                              │
  └─────────────────────────────────────────────────────────────────────────────┘

  FENCE (accepted-leak, inherited from governance — NOT part of the surface above):
    Same class as governance's fence + R6/R11/R14. Checks-specific residuals:
    · the producer SIGNING KEY's secrecy is the root of forgery-hardness (R2) —
      in the personal-tenant model the agent shares the OS user and can READ the
      keyring secret WITHOUT elevated privilege → it can forge a signed row.
      RAISES the bar, does NOT close it; closed by Ed25519 + an OS-sandboxed
      executor where the key is unreachable.
    · a merge done on the forge (auto-merge/UI/CI) or by raw push bypasses the
      LOCAL gate (no local check consulted) — inherited governance R11 (R9 here),
      and where the R12 TOCTOU is worst.
    · an un-audited N-API merge route skips the required-checks clause — R14 (R10).
    STEEL-TRAP (deferred): server-side pre-receive + forge-side required-checks
    (push/forge class) · Ed25519 + OS-sandboxed executor holding the key (the
    signing-key class) · "require up to date"/merge-queue (the R12 TOCTOU class).
```

## Reading the diagram

- **Producer → ledger → consumer is the whole design.** The executor produces a signed, OID-bound result; the ledger stores it agent-unwritably (a forged _unsigned_ direct INSERT has no valid signature); the **read-only** gate consumes it. **Forgery-hardness lives in the producer + the signature, never in the gate** (Stance 0/1). What the gate _proves_ is narrow: "the committed, ref-pinned check ran under the trusted executor and exited 0 at the current head OID" — never "the code is correct," never "non-fakeable."
- **The executor is NOT the agent, and the gate does NOT run it.** The executor runs in the trusted daemon/CLI the human runs and holds the producer key; on a merge, the **orchestrator** runs `on-merge-attempt` checks just before calling merge (DECISION A). The gate trusts what the executor signed, never the agent's textual claim that "the check passed" (Stance 1), and never invokes a run itself.
- **No broker in v1.** The executor runs the check _in the trusted process_ — there is no lease/long-poll/labelled-runner protocol; that, and untrusted/fork-PR execution, are deferred (Stance 2).
- **The conclusion is typed.** The 6-variant `Conclusion` enum (v1 _produces_ `success`/`failure`/`timed_out`; `neutral`/`cancelled`/`skipped` reserved for forward-compat/GH-import) — the gate's allow-decision is `conclusion == success` (or a configured satisfying set), never a string compare (Stance 3; `03-data-schema.md` Q3).
- **Recording is deterministic engine code.** Sign + record always happen after a run; the agent cannot skip or fake them (Stance 4).
- **The SHA-reset invariant is automatic by keying — but not atomic.** Results are keyed by head OID; the gate reads only the current head; a rewrite moves the OID so prior rows stop matching. Matching by check _name alone_ (ignoring `head_oid`) would let an old-head `success` satisfy after a rewrite — the build must not do that (Stance 5; R3). The gate-read→merge-commit window is non-atomic (R12).
- **The required-check config is self-protecting.** Weakening a required check must clear the pre-change required checks at the target ref (Stance 9 / R11) — you cannot loosen the gate in the merge those loosened checks would wave through.
- **The gate composes with governance.** One read-only gate, three clauses (authority → review → required-checks), one denial contract with one new code `gate.check_required` (Stance 7).
- **The fence is inherited and leaky.** Same accepted-leak class as governance; the Checks-specific residual is the signing key's secrecy (R2 — agent-readable in the personal-tenant model) plus the non-atomic gate window (R12). The build must not present the executor's signing as unbreakable, the local gate as binding forge/push merges, or the gate as atomic with the merge commit.
