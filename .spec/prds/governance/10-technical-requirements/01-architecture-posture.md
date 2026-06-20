---
stability: CONSTITUTION
last_validated: 2026-06-18
prd_version: 1.3.0
section: technical-requirements
---
# Architecture Posture

## The control model in one line

> **Control is on the git ACTION, not the tool; permissions are FUNCTIONAL, not role-based; the control plane is exactly TWO gates over GitButler's own actions; the governed path is made cheaper than the bypass, not impossible to bypass.**

## Stance 0 — irrigation, not a dam (the governing philosophy)
We are not building a wall around the agent; we are grading a riverbed. The agent retains the *physical* ability to step outside the governed path (the fence is an accepted, leaky guardrail in this slice — see Stance 5). The design's wager is that a goal-directed agent takes the **cheapest** route to progress, and we make the **governed** route the cheapest: commit through `but`, get a review, land. Defection (raw plumbing, `--no-verify`, editing hooks) costs more effort and yields no faster path to "done." So the agent's own optimization carries it down the governed channel. Every other stance below serves this one: keep the governed path ergonomic and legible (so it stays cheap) and make consequence legible at the gates (so compliance is the obvious move).

## Stance 1 — control is on the git action, not the tool
An agent may freely edit files in the **shared working directory** — `fs.write` is **not** governed, and there is **no per-agent worktree** (GitButler's model is virtual branches over one working tree, not a worktree per branch). Nothing it writes matters until it becomes a **GitButler git action** (`commit`, `integrate`/`merge`, `push`, `review`, `comment`). Those are what is permission-checked and gate-checked. A reviewer with no `contents:write` can edit all it wants — the edits are **inert** because it cannot land them. Role separation falls out of *git-action permissions*, not tool-blocking. This keeps the agent surface ergonomic (so the governed path stays the cheap path, per Stance 0) while making the only thing with consequence — *landing* — pass through a gate.

## Stance 1b — governance is branching-mechanism-agnostic
GitButler supports more than one way to organize code, and governance must cover **all** of them — a gate wired to one is bypassed by another (source confirms many commit/ref-advancing entry points: `but-api/commit`, `but-api/legacy/virtual_branches`, `but-workspace/commit/*`, `but-worktrees/integrate`, `but-workspace/branch/apply`, `but-workspace/branch/unapply`, `gitbutler-branch-actions/*`):

| Mechanism | Default? | How code lands | Gated at |
|---|---|---|---|
| **Virtual branches / stacks** | **yes** — the signature model | changes in the **one shared working dir** → `but_workspace::commit_engine::create_commit` (→ a `stack_id`/ref); `but-workspace::branch::apply` | the commit narrow-waist + apply path |
| **Normal Git / single-branch** | when working unmanaged | an ordinary commit on the checked-out branch | the same commit narrow-waist |
| **Worktrees** (`but-worktrees`) | **opt-in only** (`but worktree new`) | a real git worktree off a workspace branch, brought back via `but-worktrees::integrate` (graph-rebase editor) | the worktree commit + `integrate` paths |

The invariant: **the same functional-permission + gate decision applies to a commit / integrate regardless of which mechanism produced it.** A governed `but` push action (where GitButler owns the push entry point) is gated identically; a **raw `git push` to a protected ref is an accepted-leak bypass (R1/R11)** — the local trunk ref is immutable, landing is a remote-forge op, and the deferred steel-trap pre-receive closes the push path. Worktrees are never forced (default is virtual branches); when a user opts into them, their commit/integrate are gated **identically**. The build MUST enumerate every consequential entry point (`commit_engine`, `branch apply`, `branch unapply` — which re-points rather than creating a commit, so it advances a protected branch only via apply, not unapply; unapply is therefore not a commit-gate land path but is covered by the same `administration:write` config discipline), `but-worktrees::integrate`, the forge PR-merge) and gate or explicitly accept each — a mechanism left ungated is a blocking gap (R7/R8).

### The seam binds only callers that route through `but-api` (the four-caller rule, R14)
Mechanism-agnostic gating is necessary but not sufficient: the seam also only governs the **callers** that reach the action through `but-api`. GitButler has **four** — Tauri desktop, the `but` CLI, the TUI, and **`but-napi` (N-API / Electron lite)**. The first three route through `but-api`; **`but-napi` may call `but-workspace`/`but-core` directly and skip the `_with_authz` wrapper / pre-call guard.** A consequential N-API route that reaches a lower-level crate directly is an **ungoverned bypass** — the **same accepted-leak class as the fence (R1)** until audited. The build MUST audit every N-API entry point (build-gate grep-assert, T-AUTHZ-016b) and route each consequential route through `but-api`. See `02-system-components.md` "all four callers" note and `07-technical-risks.md` R14.

## Stance 2 — permissions are functional, not role-based
The authoritative config is a per-principal **set of functional permissions** mirroring GitHub's fine-grained model (`contents:write`, `pull_requests:write`, `reviews:write`, `comments:write`, `merge`, `statuses:write`, `administration:write`, …). Named roles (`read ⊂ triage ⊂ write ⊂ maintain ⊂ admin`) are **optional presets** that desugar to a functional set at load; **enforcement only ever sees the functional set**. A "superuser" is just a principal granted every permission. The hard invariant: **no enforcement path keys governance off a role name** (grep-asserted, UC-AUTHZ-02).

Not every catalog token gates a POC action. The **enforced** subset — each with a real gated GitButler route — is `{contents:read, contents:write, pull_requests:write, reviews:write, comments:write, merge, administration:write}`. The remaining catalog tokens (`statuses:read/write`, `administration:read`, triage-only) are **parseable but catalog-only** in this slice (forward-compat for when those actions exist locally) and gate nothing — the route table (`04-api-design.md`) marks which authority gates which route, so the R8 "every consequential route is `_with_authz`-wrapped" checklist stays satisfiable (you cannot wrap a route that does not exist).

| Guarantee | Mechanism |
|---|---|
| Role is sugar, never enforcement | `AuthoritySet::from_role("write")` expands to `{contents:write, pull_requests:write, reviews:write, statuses:write, comments:write, metadata:read, contents:read, pull_requests:read}`; the action route checks `Authority::ReviewsWrite`, never the string `"write"`. |
| Superuser is a grant, not a code path | `from_role("admin")` includes `administration:write` + `merge`; there is no god-mode action that bypasses a gate. |
| Read-only is structurally inert | A principal holding only `contents:read` is denied at the commit gate; its edits never reach a ref. |
| Group authority is just more grants | A principal's effective set is `own_grants ∪ ⋃(group_grants)`; the route still asks "does the effective set contain `Authority::X`". |

## Stance 3 — two gates at the governed ACTION boundaries (deterministic there; bypass paths are accepted-leak)
The control plane is two gates wired into GitButler's own action boundaries — and the honest framing matters, because **source review (`but-workspace/src/upstream_integration.rs:159`) shows GitButler marks the trunk ref `ExtraRef::immutable(...)` and never writes trunk locally: landing onto a protected branch is a *remote-forge* operation** (`but-api`'s `merge_review` → `but_github::pr::merge`/`but_gitlab::mr::merge`; `set_review_auto_merge` is fire-and-forget on the forge). Commit creation likewise has **several callers** (the narrow waist is `but_workspace::commit_engine::create_commit`, not `but-action`). So a gate is **the governed `but` action boundary**, deterministic *there* — and the paths that route *around* the governed action (forge auto-merge, a UI/CI merge, a raw `git push` to the protected ref, a commit via an unwrapped path, **a direct N-API call to a lower-level crate**) are the **same accepted-leak class as the fence (R1)**, closed only by the deferred steel-trap.

| Gate | Wired at (source-grounded) | Deterministic for | Bypass paths (accepted-leak, R1) |
|---|---|---|---|
| **Commit gate** | the commit narrow-waist (`but_workspace::commit_engine::create_commit`) **or** the `but-api` commit boundary across **all** commit wrappers (`create`/`amend`/`move_changes`/`uncommit`…) — never `but-action` alone, which is one caller | a governed `but` commit: principal from `BUT_AGENT_HANDLE`; `contents:write` + branch protection from `.gitbutler/gates.toml` | raw `git commit`/plumbing (the fence); an ungoverned N-API commit path (R14) |
| **Merge gate** | the `but-api` PR-merge **action** boundary (`legacy/forge.rs::merge_review` / `set_review_auto_merge` / `publish_review`) | a governed `but` merge: acting principal has `merge`; review requirement @head (read from the `local_review_verdicts` store) | forge auto-merge, a UI/CI merge on the forge, raw `git push` to the protected ref; an ungoverned N-API merge path (R14); a **forged review row** in `local_review_verdicts` via direct DB write (R6, High) |

Permission says *who may attempt*; the gate says *whether the governed action proceeds*. The honest claim is therefore **not** "landing passes through exactly two deterministic local gates" — it is "**the governed `but` commit and merge actions are deterministically gated; the bypass paths are accepted-leak (R1)**, closed by the deferred steel-trap (a server-side pre-receive on a bare repo the agent reaches only by push)." This is the irrigation thesis applied to the gates themselves: the governed path is gated and cheap; the bypass is possible but uphill. The merge gate's review requirement reads the **forgeable** local `local_review_verdicts` store (R6, High) — a direct DB write forges an approval, so the gate is sound only for reviews submitted through the governed `but review` action. The build MUST enumerate every action path that can advance a protected ref (incl. rebase/reorder/amend/auto-merge) and either gate it or list it as an accepted bypass — a silently-ungated land path is a blocking gap (R7).

## Stance 4 — config is the only lever, and it is ref-pinned
The only way to change strictness or grant authority is a **governed config change** — committed, ref-pinned. `.gitbutler/permissions.toml`, `.gitbutler/gates.toml`, and group config are read at the **target ref** when authorizing/gating, so a change whose head edits them cannot weaken the gate that judges it or self-grant authority (UC-GRPS-02, UC-GATES-02). Editing governed config requires `administration:write` (UC-AUTHZ-03). This slice ships **no override / break-glass** at all — the only lever is the governed config change. (An audited break-glass override is a deferred hardening layer.)

## Stance 5 — the fence is accepted-leaky, by design (for the POC)
Soundness in the *full* vision rests on GitButler being the agent's sole path to the canonical repo. This POC does **not** build that boundary. Forcing the agent through `but` is delegated to **client git hooks or harness hooks**, which an adversarial actor can bypass (`--no-verify`, editing `core.hooksPath`/`.git/hooks`, raw plumbing). That residual is **named and accepted** (01-scope Known Limitations). It does not undermine the POC because the POC's claim is the *irrigation* claim (Stance 0): the governed path is cheapest. The **steel-trap** closure — a server-side pre-receive on a bare repo the agent reaches only by push, or an OS sandbox where GitButler is the sole channel — is the future layer that converts "cheapest path" into "only path."

| Boundary | Strength | This slice |
|---|---|---|
| Client git hooks / harness hooks | leaky (editable, `--no-verify`-able) | **the POC fence** |
| Server-side pre-receive on a bare repo | non-bypassable (`--no-verify` cannot skip it; agent can't edit it) | **deferred** |
| OS sandbox (GitButler sole repo channel) | physical | **deferred** |

The fence is not the *only* accepted leak: because the trunk is immutable locally (Stance 3), the **merge-land bypass paths** — forge auto-merge, a UI/CI merge on the forge, a raw `git push` to the protected ref — are the **same accepted-leak class**, and the same deferred server-side pre-receive closes them. Two further residuals are the same class: an **ungoverned N-API call** to a lower-level crate (R14, closed by the N-API audit + steel-trap) and a **forged review row** in `local_review_verdicts` via direct DB write (R6, High — closed by the deferred HMAC/Ed25519 review integrity). The POC gates the *governed* `but` actions; it does not, and honestly cannot, bind a merge that happens entirely on the forge or via raw push, an ungoverned N-API path, or a forged direct-DB review. No section of this PRD may present any of these as bound.

## Stance 6 — what is deterministic vs probabilistic
Per the project's deterministic-vs-probabilistic doctrine: **things that MUST always happen are engine code, not an agent decision.**

| Concern | Owner | Determinism |
|---|---|---|
| Whether a principal *may* perform an action | `but-authz::authorize()` | **deterministic** |
| Whether a change *lands* (the two gates) | the commit/merge gate code | **deterministic** |
| Resolving the effective `AuthoritySet` (own ∪ groups) at the target ref | `but-authz` loader | **deterministic** |
| What the agent *does* (reason, edit, choose a commit message, adapt to a denial) | the agent (in the harness) | **probabilistic — owned by the harness, not this PRD** |

The split is the boundary of this PRD: GitButler owns the deterministic *authorization + gating*; the harness owns the probabilistic *agent*. We govern actions; we do not drive reasoning (Assumption 2).

## Trust boundary — who is controlled
| | The orchestrator (harness driving `but`) | Principals GitButler runs actions for |
|---|---|---|
| Trust | **Trusted / uncontrollable** — your harness, your keys, your reasoning loop | **Semi-trusted** — bound at every git action |
| Enforced | **Outcomes only** — the two gates stop it landing a rule-violating commit/merge | **Each git action** — permission-checked + gate-checked, denied legibly |
| Site | the two gates | the authorization check + the two gates |

A "superuser orchestrator" that holds every permission is the user's prerogative — just a principal granted all functional permissions. The point is that the **principals it drives** are deterministically bound, action by action, by the configured functional permissions and gates.
