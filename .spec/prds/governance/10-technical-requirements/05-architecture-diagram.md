---
stability: CONSTITUTION
last_validated: 2026-06-18
prd_version: 1.3.0
section: technical-requirements
---

# Architecture Diagram

## The governed action path

```
┌──────────────────────────────────────────────────────────────────────┐
│ Orchestrator  (the harness — owns the agents' reasoning, Assumption 2) │
│   dispatches each agent with  BUT_AGENT_HANDLE=<principal>            │
└──────────┬──────────────────────┬───────────────────────┬────────────┘
       implementer            reviewer                human (maintainers)
      (contents:write)       (reviews:write)          (merge)
           │                      │                        │
           ▼                      ▼                        ▼
   ┌────────────────────────────────────────────────────────────────────┐
   │  BUTLER ACTION SURFACE   (but CLI · but-api · Tauri · N-API)         │  ← the ONLY enforcement surface
   │  local action  → with_authz(principal, Authority::X) → _impl  (a)    │     (Assumption 1: govern the action,
   │  async forge   → authorize(principal, X)? → impl().await   (b)       │      not the tool, not the reasoning)
   │  ⚠ N-API must route through here — a direct lower-level call          │     seam binds ONLY but-api callers;
   │    bypasses the seam (R14, audited via T-AUTHZ-016b)                  │     audit every but-napi entry point
   └──────┬─────────────────────┬──────────────────────┬─────────────────┘
          │ commit              │ review / comment      │ merge (PR-merge action;
          ▼                     ▼                       ▼   trunk immutable locally)
   ┌─────────────┐      ┌──────────────┐       ┌────────────────────────┐
   │ COMMIT GATE │      │ review action│       │ MERGE GATE             │
   │contents:write│     │reviews:write │       │ merge  +  review reqt  │
   │+ branch prot │     │ → review rec │       │ @head, distinct, groups│
   │ commit_engine│     │ (but-db tbl  │       │ but-api legacy/forge   │
   │ (ALL mechs)  │     │  forgeable!) │       │ (governed action only) │
   └──────┬──────┘      └──────┬───────┘       └───────────┬────────────┘
          └────────────────────┴───────────────────────────┘
       commit gate covers virtual-branch · normal-git · worktree-integrate paths
       review record = local_review_verdicts (R6, High: forgeable by direct DB write)
                               ▼
   ┌────────────────────────────────────────────────────────────────────┐
   │  but-authz  (NEW crate)                                              │
   │   Authority · AuthoritySet · Principal · Group                       │
   │   authorize() → Ok | Denial{code, message, hint}(exit1); fail-closed │
   │   effective set = own ∪ ⋃(group grants)                             │
   │   ALL config read @ TARGET ref  (never the working tree)             │
   └───────────────┬──────────────────────────────┬─────────────────────┘
                   │ committed · ref-pinned · governed (admin:write to edit)
        ┌──────────┴───────────┐         ┌─────────┴──────────────────────┐
        │ .gitbutler/          │         │ .gitbutler/gates.toml           │
        │   permissions.toml   │         │  branch protection +            │
        │  [[principal]][[group]]│       │  review requirement (by group)  │
        └──────────────────────┘         └─────────────────────────────────┘

  ┌─ orthogonal axis, do NOT overload ───────────────────────────────────┐
  │ GitButler's existing  Permission / _with_perm  =  repo-ACCESS LOCK     │
  │ (concurrency). Mirror its composition SHAPE as _with_authz; not type.  │
  └───────────────────────────────────────────────────────────────────────┘

  FENCE (accepted-leak, POC — NOT part of the enforcement surface above):
    git hooks via core.hooksPath / harness hooks → reject raw `git commit|merge`
    so agents route through `but`. Known holes: --no-verify, editable hooks, plumbing.
    Same accepted-leak class: a merge done on the forge (auto-merge / UI / CI) or by
    raw push to the protected ref; an ungoverned N-API call (R14); a forged review row
    in local_review_verdicts via direct DB write (R6, High). ── STEEL-TRAP (deferred):
    server-side pre-receive on a bare repo, or a sandbox where butler is the sole repo
    channel — closes the push/forge class; the N-API audit + HMAC/Ed25519 review
    integrity close R14 / R6.
```

## Reading the diagram

- **Top → bottom = the only path with consequence.** Edits in the shared working directory are inert until they become a Butler action (GitButler uses virtual branches over one working tree — no per-agent worktree); the action surface is where authority is acquired and the gates fire.
- **The seam binds only callers that route through `but-api`** — Tauri / `but` CLI / TUI do; **N-API must be audited** so it cannot reach a lower-level crate ungoverned (R14, T-AUTHZ-016b). A direct N-API call below the seam is the same accepted-leak class as the fence.
- **The commit gate is mechanism-agnostic** — it sits at the `commit_engine` narrow-waist and covers virtual-branch/stack, normal-git, and opt-in worktree-integrate paths alike; a mechanism left ungated is a blocking gap.
- **`but-authz` is pure and fails closed** — it answers _may this principal do this action_ from config read at the target ref, denying on unknown principal / unreadable config. It never reaches the network or drives an agent.
- **The review record is forgeable (R6, High)** — `local_review_verdicts` is not integrity-protected; a direct DB write forges an approval. The merge gate is sound only for reviews submitted through the governed `but review` action; HMAC/Ed25519 review integrity is the deferred closure.
- **The fence is outside the enforcement surface** — it is what tries to keep agents _on_ this path, deliberately leaky in the POC (the irrigation bet, not the steel trap); the merge-land bypass paths (forge auto-merge / UI / raw push), the N-API residual (R14), and the forged-review residual (R6) are the same accepted-leak class.
- **The two gates are the choke points** — commit gate at the `commit_engine` narrow-waist, merge gate at the `but-api` PR-merge action, both consulting `but-authz` + `.gitbutler/gates.toml`.
