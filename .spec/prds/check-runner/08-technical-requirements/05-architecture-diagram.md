---
stability: CONSTITUTION
last_validated: 2026-06-20
prd_version: 1.0.0
---

# 05 — Architecture Diagram

## Producer → store → consumer (the whole system)

```text
   ┌─────────────────────────── PRODUCER (but-checks) ───────────────────────────┐
   │                                                                              │
   │   .gitbutler/checks/*.toml  ──(ref-pinned blob read, read_config_blob)──┐    │
   │        [[check]] name/command/args/trigger/success_exit_codes/timeout   │    │
   │                                                                         ▼    │
   │   `but check run <name> --head <oid>`                          load_check_defs
   │        │                                                                      │
   │        ▼                                                                      │
   │   checkout.rs ── materialize HEAD OID in ISOLATED tree (07) ──┐               │
   │      A) git worktree add --detach <oid> <temp>  (default)     │  shared agent │
   │      B) gix object-DB-only read (no working tree)             │  worktree is  │
   │      C) warm-reused worktree / tmpfs (latency)                │  NEVER touched│
   │        │                                                      │               │
   │        ▼                                                      ▼               │
   │   runner.rs ── spawn command (std::process/tokio), wait ≤ timeout_seconds     │
   │        │                                                                      │
   │        ▼                                                                      │
   │   Conclusion::from_exit(REAL ExitStatus, success_exit_codes)                  │
   │        │   (the ONLY exit→conclusion map; NO caller-supplied conclusion)      │
   │        ▼                                                                      │
   │   recorder.rs ── append CheckResult{ name, head_oid, conclusion, metadata }   │
   └────────────────────────────────────┬─────────────────────────────────────────┘
                                         │
                                         ▼
                       ┌──────────── check_results (but-db) ───────────┐
                       │  PLAIN TABLE — no signing, no crypto           │
                       │  PK id | name | head_oid | conclusion          │
                       │  recorded_at | metadata | signature(NULL,FC)   │
                       │  INDEX (name, head_oid)                        │
                       │  append-only · latest-wins per (name,head_oid) │
                       │  no more protection than local_review_verdicts │
                       └────────────────────┬───────────────────────────┘
                                            │ (read only)
   ┌──────────────────── CONSUMER (extended merge_gate.rs) ───────────────────────┐
   │  enforce_merge_gate_for_refs(ctx, source_ref, target_ref) [READ-ONLY·DETERMIN.]│
   │   // mechanism-agnostic entry (§1a/R-ENTRY); local merge calls it directly,    │
   │   // forge path delegates here after resolving refs from its ForgeReview        │
   │     ├─ authorize(Merge)                              (existing)                 │
   │     ├─ review-requirement clause                     (existing)                 │
   │     └─ REQUIRED-CHECKS clause  (reached independent of `protected`, 01 §9)      │
   │           defs    = but_checks::load_check_defs(repo, target_ref)               │
   │           req     = [[required_check]] for target  (ref-pinned gates.toml)      │
   │           head    = gix ref-peel of source_ref  (merge_gate.rs:172-179,        │
   │                       NOT forge-keyed :78)                                      │
   │           results = check_results.list_for(name, head)                          │
   │           evaluate_required_checks(req, defs, results, head)   ── PURE          │
   │              Ok(())  → proceed                                                  │
   │              Err(unmet) → MergeGateError{ code:"gate.check_required", unmet }   │
   │                 // base fields TODAY; +class/held_permissions/authorized_actions│
   │                 // /do_not ONLY after governance STEER-001 lands (Backlog)      │
   └────────────────────────────────────┬─────────────────────────────────────────┘
                                         │
                                         ▼
                         merge proceeds  iff  gate == Ok
                         else exit 1 + denial → agent runs `but check run` → retry
                         (denial carries STEER steering fields once governance
                          STEER-001 lands; base code/message/unmet before that)
```

## SHA-binding / staleness (R-SHARESET)

```text
   head = A     but check run cargo-test --head A  →  CheckResult{cargo-test, A, Success}
                                                              │
   rebase / amend / new commit                                │  Editor::commit_mappings()
   head = B  (A → B in the mapping)                            │  (graph_rebase/mod.rs:479)
                                                              ▼
   merge attempt @ head B:
       evaluate_required_checks(req, defs, results, head=B)
          result for cargo-test exists only at A  →  unmet = ["cargo-test: check_stale_at_head"]
          BLOCK.   (match on (name, head_oid==current); NEVER on name alone)
```

## Orthogonal axes (what the gate does and does not prove)

```text
   AUTHORIZATION axis (governance)        EXECUTION axis (Check Runner)
   ───────────────────────────────        ─────────────────────────────
   "is this principal allowed to merge?"  "did the committed check exit 0 at the head OID?"
   authorize(Merge)                       evaluate_required_checks(...)
            │                                       │
            └──────────────┬────────────────────────┘
                           ▼
              both must pass — composed in ONE read-only enforce_merge_gate
              (Check Runner NEVER proves "the code is correct"; NEVER "non-fakeable", 01 §10)
```

## Mechanism-agnosticism (one checkout, any mechanism)

```text
   virtual branches ─┐
   worktrees        ─┼──►  resolve head OID (gix)  ──►  checkout.rs materializes THE OID
   plain git        ─┘         (not "the working tree", not "the branch", not a projection)
                                            │
                                            ▼
                        identical run + result regardless of branching mechanism
                        (execution analog of GATES-007's one-decision-helper
                         authorization — GATES-007 in progress, Backlog)
```

## Cross-references

- Component map behind these boxes: [`02-system-components.md`](./02-system-components.md)
- The checkout strategies (A/B/C): [`07-mechanism-agnostic-checkout.md`](./07-mechanism-agnostic-checkout.md)
- The run → record → consume sequence in prose: [`04-api-design.md`](./04-api-design.md) §2
