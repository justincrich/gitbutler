---
stability: CONSTITUTION
last_validated: 2026-06-26
prd_version: 1.2.0
---

# 07 — Mechanism-Agnostic Clean Checkout at the Head OID

**This is the headline engineering content of Check Runner and the #1 technical
risk (08 R-CHECKOUT).** Everything else in the runner is a thin shell around one
hard problem: *get the exact current head OID materialized somewhere the check
can run, identically across GitButler virtual branches, worktrees, and plain
git, without disturbing the agent's live shared worktree.*

## §1 — Why "just run the command in the repo dir" is wrong

The naive implementation — `Command::new("cargo").current_dir(repo_root)` — is
the **explicit anti-pattern**. GitButler is **virtual-branches-over-one
worktree**:

- The single shared worktree is where the agent is **actively editing**. It may
  be **dirty**.
- It is a **workspace projection of several virtual branches** — not a clean
  checkout of any one branch's head OID. (See `crates/WORKSPACE_MODEL.md`:
  `but_graph::Workspace` is a *lossy presentation view*; the working tree is not
  a faithful single-branch tree.)
- The head OID the gate will match (`current_head_oid`, merge_gate.rs:78) is a
  **commit object**, not "whatever is currently on disk."

Running the command in the repo dir therefore runs it against the live, dirty,
multi-branch projection — silently **breaking the head-OID binding** the gate
depends on. A green produced there does not mean "the check passed at OID `A`";
it means "the check passed against whatever the agent had on disk." That is the
correctness hole this section closes.

## §2 — Two hard constraints

1. **Run against the exact head OID.** The checkout the runner executes in must
   be the tree of `current_head_oid`, byte-for-byte — not the working tree, not a
   feature-branch head, not a workspace projection.
2. **Never mutate or contend on the agent's shared worktree.** A check run is a
   read-side operation triggered by a *merge attempt* (and optionally on-commit).
   It must not touch the shared index, must not take the shared
   worktree-exclusive lock for its own working tree, and must not leave the
   shared tree in a different state than it found it.

These two constraints are jointly satisfied by materializing the head OID into an
**isolated** location.

## §3 — Materialization options

The runner resolves the head OID, then materializes it via one of these
strategies (selected per check and per platform). All three keep the shared
worktree untouched.

### Option A — throwaway detached worktree (default for checks that need a working tree)

```text
git worktree add --detach <temp_path> <head_oid>
# run the check with current_dir = <temp_path>/<working_subdir>
git worktree remove <temp_path>        # always, even on failure
```

- **Isolated tree**: a separate working directory at a temp path, checked out to
  the exact OID. The agent's shared worktree is never touched.
- **Shared object DB**: the temp worktree shares the repo's object database, so
  no re-clone — only the working tree is materialized.
- Use `gix` worktree APIs where available; fall back to the `git worktree`
  executable at the shell boundary (RULES.md permits shelling at hook/executable
  boundaries; `gix` is preferred for in-process logic). This is the **boundary
  escape hatch**, analogous to how `ci.rs` shells a `tokio` runtime onto a thread
  (`crates/but-forge/src/ci.rs:61-67`).
- **Temp path discipline**: per crates/AGENTS.md, tests must **never** use
  `std::env::temp_dir().join(format!(…))`; production materialization uses a
  managed temp dir (a `tempfile::TempDir` or a checks-owned cache root, 06) and
  **removes it on all exit paths** (Drop guard), so a panicking check cannot leak
  a worktree.

### Option B — object-DB-only, no working tree (for purely-git checks)

For a check that inspects Git data and needs **no working tree** (e.g. "the head
tree contains a committed `Cargo.lock`", "no merge markers in the head tree", "a
required file path exists at the head OID"), the runner reads the tree directly
via `gix` (`repo.find_object(head_oid).peel_to_tree()` →
`lookup_entry_by_path`, the exact pattern `governance_present` uses at
`crates/but-authz/src/config.rs:62-71` and `read_config_blob` uses at
merge_gate.rs:224-232). **Zero working-tree materialization, zero shared-tree
contention** — the cheapest path, and the right one for any check that does not
need to execute compiled code.

### Option C — tmpfs / warm-reused worktree (latency amortization)

- **tmpfs**: place Option A's temp path on a memory-backed filesystem to cut I/O
  on the materialization and the build artifacts.
- **Warm worktree**: keep a single persistent detached worktree per repo and
  `git checkout --detach <head_oid>` it for each run (re-using the build cache /
  `target/` dir between runs), rather than create+destroy. This trades a small
  steady-state footprint for amortized checkout + incremental-compile cost. The
  warm worktree is **checks-owned** and **never** the agent's shared tree (so
  reuse never violates constraint 2).

| Check kind | Default option |
|------------|----------------|
| Needs to execute code (`cargo test`, `pnpm check`, a build `./script`) | A (warm-reused → C when the latency budget demands) |
| Inspects Git data only (file presence, tree shape, no exec) | B (object-DB-only) |

## §4 — Latency budget (these run synchronously before a merge)

A required check runs on **`on-merge-attempt`** — synchronously, in the
pre-merge step (04 §2), before the gate is consulted. So checkout cost is on the
critical path of a human/agent merge.

| Phase | Budget (soft) | Mitigation |
|-------|---------------|------------|
| Resolve head OID + read config | < 50 ms | Pure `gix` ref/tree reads (the merge gate already pays this). |
| Materialize checkout (Option A cold) | seconds | Shared object DB (no clone); Option C warm worktree reuses the tree + build cache; tmpfs cuts I/O. |
| Run the check command | check-defined; capped by `timeout_seconds` (03 §2) | Hard timeout kills the process → `timed_out` conclusion (fail-closed). |
| Record + consume | < 50 ms | Single append + a `(name, head_oid)` indexed read (03 §1). |

**Design rule:** the gate **never runs the check inline**. The check runs in the
pre-merge orchestration step and **records** a result; the gate only **reads**
(04 §2). This keeps the read-only consumer fast and deterministic and lets the
expensive checkout happen out of the gate's critical section. A merge attempt
that finds a missing/stale required result is **blocked with a remediation hint**
to run the check — it does not block *waiting* for a synchronous run inside the
gate.

## §5 — Interaction with the shared index and locks

crates/AGENTS.md mandates: *acquire repository/worktree locks at top-level
API/command boundaries; do not call permission-acquiring helpers while holding a
guard; debug deadlocks with `BUT_WS_LOCK_DEBUG=1`.*

- The runner takes **its own** worktree (Option A/C) — it does **not** acquire
  the shared `ctx.exclusive_worktree_access()` guard that mutating operations
  take. A check run is not a workspace mutation; it must not contend for that
  lock. (Contrast the commit gate, which is designed to run *before* the guard at
  `crates/but-api/src/commit/gate.rs` and the apply/integrate seams — the commit
  gate's pre-guard design (**GATES — landed; governance closed**) is read-only and
  pre-guard, and the runner is likewise off to the side of the shared tree.)
- `git worktree add`/`remove` touch the repo's **worktree administrative state**
  (`.git/worktrees/`), a different lock domain than the shared working index.
  The runner serializes its own worktree lifecycle (one checks worktree per repo)
  so two concurrent runs do not race the admin state (08 R-CHECKOUT mitigation).
- Validate non-contention under `BUT_WS_LOCK_DEBUG=1` in the integration proof:
  assert the shared worktree's index and HEAD are **unchanged** before/after a
  check run, and that no shared guard was held during the run.

## §6 — The hard constraint, restated for the implementer

> A check run **MUST NOT** mutate or contend on the agent's shared worktree. It
> materializes the head OID into a checks-owned isolated location (detached
> worktree, object-DB-only read, or warm-reused worktree on tmpfs), runs there,
> records the result keyed by that OID, and tears its materialization down on
> every exit path. The shared tree is observably identical before and after.

## §7 — Mechanism-agnosticism (the payoff)

Because the runner materializes **the head OID** (a commit object) rather than
"the current branch's working tree", the same code path works identically across:

- **GitButler virtual branches** — the head OID is resolved from the workspace
  target/source ref via `gix` (merge_gate.rs:78); the projection on disk is
  irrelevant.
- **Worktrees** — a check materializes its own detached worktree regardless of
  how many worktrees the user has; it does not assume the invoking worktree ==
  the head OID.
- **Plain git** — with no virtual branches at all, the head OID is just the
  ref tip; Option A/B/C are unchanged.

This is the mechanism-agnostic property the governance commit gate **achieves**
for *authorization* (**GATES — landed; governance closed**: one decision helper
fired at every ref-mutating seam). Check Runner achieves the analogous property
for *execution*: one checkout strategy that binds to the OID, not the mechanism.

## Cross-references

- Why this is risk #1 (re-ranked above the dissolved forgery risks): [`08-technical-risks.md`](./08-technical-risks.md) R-CHECKOUT
- The runner component and its module map: [`02-system-components.md`](./02-system-components.md) §2
- The pre-merge orchestration step (gate stays read-only): [`04-api-design.md`](./04-api-design.md) §2
- SHA-binding (the head OID the checkout must match): [`01-architecture-posture.md`](./01-architecture-posture.md) §4
