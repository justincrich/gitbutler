---
stability: FEATURE_SPEC
last_validated: 2026-06-23
prd_version: 1.5.0
functional_group: LPR
---

# Use Cases: Local Agent PR / Governed-Review Parity (LPR)

The **LPR** group gives GitButler's local review layer GitHub-PR parity, so an orchestrator drives the whole implement→review→merge loop off `but`'s own review state — a **reconciler over `but`, not a private state machine** — with agent PRs kept **local by default**. Where AUTHZ/GRPS/GATES decide _whether_ an action proceeds and LOOP demonstrates the end-to-end flow, LPR governs the **review-drive layer** around the land-truth: assignments, comment threads, the derived PR object, the local-by-default setting, and the agent tag. It is additive — every LPR behavior layers onto two new `but-db` tables and the existing `but review` write path, and **none of it feeds the merge gate**. The gate gates (verdict-at-head, read from `local_review_verdicts`); the new tables drive (orchestration).

| ID        | Title                                                        | Description                                                                                                                                                                                                                                                                                                                                                                                                                                |
| --------- | ------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| UC-LPR-01 | Local PR object + reviewer assignment                        | A local PR object with reviewer assignment (`local_review_assignments`: target, reviewer, state, assigned_at) and a PR lifecycle **derived** from commits + verdict-at-head + open assignments, mirroring `ForgeReview` where sensible.                                                                                                                                                                                                    |
| UC-LPR-02 | Local review comment thread                                  | Post / list / resolve review comments locally (`local_review_comments`: target, author, body, file, line, thread_id, resolved, created_at) via `but review`, on `comments:write`.                                                                                                                                                                                                                                                          |
| UC-LPR-03 | Project "keep PRs local" setting                             | A per-project config makes agent-authored reviews default to the local layer (no remote GitHub PR); remote mirroring is gated behind the setting.                                                                                                                                                                                                                                                                                          |
| UC-LPR-04 | Automatic agent-PR tagging                                   | A review/PR opened by a principal whose committed `.gitbutler/permissions.toml` entry **declares `kind = "agent"`** is auto-tagged `agent-authored` (a label) so agent PRs are programmatically distinguishable from human PRs. The agent-vs-human distinction is a **declared config fact** (a new optional `kind` field on the principal entry, read at the target ref like all governance config), NOT inferred from handle-resolution. |
| UC-LPR-05 | Orchestrator reconciles work off `but` review state          | An orchestrator drives dispatch off `but` review state — assignment → dispatch reviewer; unresolved comment → remediation; approved-verdict-at-head → merge — as a reconciler, not a private state machine.                                                                                                                                                                                                                                |
| UC-LPR-06 | Auto "review-requested" hook via workspace rules             | The existing `but-rules` (Trigger → Filter → Action) engine hosts an auto "review-requested" hook: a commit on a watched branch opens a local review assignment.                                                                                                                                                                                                                                                                           |
| UC-LPR-07 | Safe-seam invariant — new state never weakens the merge gate | The merge gate reads only `local_review_verdicts` at head; assignments and comments are orchestration metadata and never gate, proven by build-gate + test.                                                                                                                                                                                                                                                                                |

---

## UC-LPR-01: Local PR object + reviewer assignment

The local PR object gives the local layer parity with a GitHub PR's _structure_ without a forge: a reviewer can be **assigned** to a target branch (`local_review_assignments`), and the PR's lifecycle is **derived** — open/in-review/approved is computed from the commits on the branch, the verdict-at-head in `local_review_verdicts`, and whether any assignment is still open — rather than stored as a separate, drift-prone truth. The assignment state (`pending` / `approved` / `changes_requested`) is the drive signal an orchestrator reads to know who is on the hook; it is **never** read by the merge gate.

### Acceptance Criteria

- ☐ The GitButler engine can create a `local_review_assignments` row (target branch, reviewer principal, state `pending`, assigned_at) when a caller with `pull_requests:write` opens a local review via `but review request` (the open-PR authority, mirroring the shipped `publish_review`), persisting it to the per-project SQLite DB via the additive migration without touching `local_review_verdicts`.
- ☐ An implementer agent (or orchestrator) holding `reviews:write` can assign a reviewer principal to a target branch through `but review assign`, **only when that reviewer principal is distinct from the target branch's author principal** (`reviewer != author_principal_of_target_branch`, enforced at the `but-api` boundary — the drive-layer mirror of the merge gate's `require_distinct_from_author`), and see the resulting assignment reflected in `but review status`. A self-assignment (reviewer == author) is rejected/flagged so the drive layer cannot narrate "independently reviewed" for a self-assigned reviewer — the same-principal forgery named as **R22 (§G in the [LPR tech-delta](./enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md))**.
- ☐ The GitButler engine can derive the local PR lifecycle state from commits on the branch + verdict-at-head (`local_review_verdicts`) + open assignments, so the PR object is a computed projection rather than a fourth stored copy of truth that could drift from the gate.
- ☐ The GitButler engine can mirror the relevant `ForgeReview` fields (target_branch, source_branch, sha, author, title, draft, timestamps) in the derived local PR object where a local analogue exists, so the local PR object presents the same shape a forge PR would.
- ☐ A reviewer agent can transition its assignment state to `approved` or `changes_requested` through the governed `but review` surface, and the GitButler engine records the new state in `local_review_assignments` as a drive signal that does not gate the merge.
- ☐ The GitButler engine can deny a `but review request` from a principal lacking `pull_requests:write` (or a `but review assign` from a principal lacking `reviews:write`) with the structured `{code:"perm.denied", message, remediation_hint}` + exit 1, so the assignment write is permission-checked at the `but-api` boundary like every other governed action.
- ☐ A human maintainer can list all open assignments for a target branch via `but review status`, so the local PR object is inspectable without a forge.

---

## UC-LPR-02: Local review comment thread

A reviewer needs to _say why_ — not just record a verdict. UC-LPR-02 gives the local layer a review-comment thread (`local_review_comments`) so a reviewer agent can post inline, file/line-anchored feedback, an implementer can list it, and either can resolve a thread once addressed — all locally, on the already-shipped `comments:write` authority. The unresolved/resolved flag is a **drive signal** (an orchestrator dispatches remediation while a thread is unresolved); it is **never** read by the merge gate.

### Acceptance Criteria

- ☐ A reviewer agent holding `comments:write` can post a review comment via `but review comment` (target, body, optional file + line, thread_id), and the GitButler engine can persist it to `local_review_comments` with `resolved=false` and a created_at timestamp.
- ☐ An implementer agent (or any caller) can list the comment threads for a target branch via `but review comments`, and the GitButler engine can return them grouped by `thread_id` with their `file`/`line`/`resolved` state.
- ☐ A caller can resolve a comment thread via `but review resolve <thread_id>`, and the GitButler engine can set `resolved=true` on every comment in that thread without affecting any verdict or assignment — **but the resolver must be the thread author, the assigned reviewer, or hold the higher `reviews:write` authority** (the constraint named with **R22 (§G in the [LPR tech-delta](./enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md))**), so a single principal cannot post a `changes_requested`-style thread and then self-resolve it to forge a clean "all-clear" drive signal for another party.
- ☐ The GitButler engine can deny a `but review comment` from a principal lacking `comments:write` with the structured `{code:"perm.denied", message, remediation_hint}` + exit 1, so comment writes are permission-checked at the `but-api` boundary.
- ☐ The GitButler engine can write `local_review_comments` as a local-cache write with no network call and no `DryRun` guard, mirroring the existing `approve_review` posture (the table is local cache, not a ref/object/oplog mutation).
- ☐ An orchestrator can read the unresolved-comment set for a target branch from `but review` output and treat each open thread as a remediation signal, without that signal ever reaching the merge gate.

---

## UC-LPR-03: Project "keep PRs local" setting

The forge should be optional, not load-bearing. UC-LPR-03 adds a **per-project** "keep PRs local" setting so that agent-authored reviews **default to the local layer** — no remote GitHub PR is opened — and any remote mirroring is **gated behind the setting**. This is what makes the whole loop offline-capable and the local path the cheapest path; the remote-mirror bridge itself is named-for-later (it rides the existing `forge_reviews` + forge-sync path), not built in this slice.

### Acceptance Criteria

- ☐ A human maintainer can set a per-project "keep PRs local" flag as **a per-project operator preference, default-local, under the R12 trusted-desktop model — NOT functional-permission-gated** (it is a `gitbutler_project::Project` setting persisted in the project store, the same class as `forge_override`/`preferred_forge_user`, set via the desktop project-settings surface — _not_ a ref-pinned `.gitbutler/{permissions,gates}.toml` governed-config change, and _not_ gated by `administration:write`). The desktop human is the trusted fleet owner (R12), so the preference is owned by implicit desktop ownership rather than a checked grant; an untrusted write to the project store that flips it (→ agent PRs mirror to a public forge) is the named accepted residual **R21 (§G in the [LPR tech-delta](./enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md))** — the same accepted-leak class as R12.
- ☐ The GitButler engine can default an agent-authored review (the opener whose committed `permissions.toml` entry declares `kind = "agent"`) to the local layer when "keep PRs local" is set, so no remote GitHub PR is created for that review.
- ☐ The GitButler engine can gate remote PR mirroring behind the "keep PRs local" setting, so the remote-mirror path is unreachable for agent reviews while the flag is set (the bridge is named-for-later via the `forge_reviews` + sync path, not built here).
- ☐ The GitButler engine can preserve the existing remote-PR behavior when "keep PRs local" is unset or false, so v1.5.0 adds a local default without removing the forge path for projects that want it.
- ☐ A human maintainer can read the effective "keep PRs local" value for the project via the project-settings surface, so the local-vs-remote posture is inspectable in the persisted project store (an operator preference that is readable, not implicit) — it is a per-project setting, **not** a ref-pinned committed-config fact.

---

## UC-LPR-04: Automatic agent-PR tagging

To run a fleet, an orchestrator (and a human) must tell _an agent's PR_ from _a human's PR_ at a glance and programmatically. UC-LPR-04 auto-tags any review/PR whose **opener principal is declared `kind = "agent"` in the committed `.gitbutler/permissions.toml`** with an `agent-authored` label — using `ForgeReview.labels` as the precedent for a label field — so the differentiation is **data declared in governed config**, not inference, and stays out of enforcement (no role name is read by any gate).

> **Source-of-truth: declared principal kind, NOT handle-resolution.** The agent-vs-human distinction is a **new additive, optional `kind` field on the principal entry in committed `.gitbutler/permissions.toml`** (e.g. `kind = "agent"` / `kind = "human"`), read at the **target ref** like all governance config (anti-self-escalation). The tag derives from the **opener principal's declared kind in committed config** — it is _not_ derived from "the opener resolved from `BUT_AGENT_HANDLE`" (every governed principal — agent and human — resolves through `resolve_principal_from_env` keyed on `BUT_AGENT_HANDLE`; that path **cannot** tell agent from human and there is no `is_agent`/`PrincipalKind` discriminator on `Principal`). The `kind` field is a small **additive AUTHZ-config descriptor** that **does NOT change enforcement** — no gate reads it; only the tag-derivation and the UI read it. The computed `agent-authored` tag is still cached in the `local_review_meta` opener row (the F-003 storage), but the _source-of-truth_ for "is this agent-authored" is the committed principal `kind`.

### Acceptance Criteria

- ☐ The GitButler engine can automatically apply an `agent-authored` label to a local review/PR object when the **opener principal's committed `.gitbutler/permissions.toml` entry declares `kind = "agent"`** (read at the target ref), mirroring the `ForgeReview.labels` field precedent — the tag derives from the declared principal kind in governed config, not from handle-resolution.
- ☐ The GitButler engine can omit the `agent-authored` label when the opener principal's committed entry declares `kind = "human"` (or omits `kind`, the default-human posture), so human-authored PRs are distinguishable from agent-authored ones.
- ☐ An orchestrator can filter local PR objects by the `agent-authored` label via `but review status`, so it can act on agent PRs without parsing author identity.
- ☐ The GitButler engine can keep the `agent-authored` tag as descriptive metadata that no gate reads, so the tag differentiates without becoming an enforcement key (role separation still emerges from the functional permission set, not from a label, and the `kind` field changes no gate decision).
- ☐ A human maintainer can see the `agent-authored` tag on a local PR object, so they can distinguish at the feature level which PRs originated from the agent fleet.

---

## UC-LPR-05: Orchestrator reconciles work off `but` review state

This is the thesis as a behavior: an orchestrator is a **reconciler over `but` review state**, not a private state machine that shadows it. It dispatches a reviewer because `but` shows an **open assignment**; it dispatches remediation because `but` shows an **unresolved comment**; it merges because `but` shows an **approved verdict at head**. Every decision is a projection of `but`'s own state, so two orchestrators on the same repo converge, and the human and the agents share one source of truth.

### Acceptance Criteria

- ☐ An orchestrator can read the full review-drive state for a target branch from `but` — open assignments, unresolved comment threads, and the verdict-at-head — through `but review status`, so it can decide the next action without a private shadow state.
- ☐ An orchestrator can dispatch the assigned reviewer agent when `but` shows an open `pending` assignment on a branch, so reviewer dispatch is driven by `but` state rather than by an orchestrator-internal queue.
- ☐ An orchestrator can dispatch a remediation pass to the implementer agent when `but` shows an unresolved comment thread, so remediation is driven by `but` state.
- ☐ An orchestrator can attempt the governed merge only when `but` shows an approved verdict at the current head, so the land step is driven by the same `local_review_verdicts` truth the merge gate enforces (the orchestrator's read and the gate's read agree).
- ☐ Two orchestrators reading the same repo's `but` review state can reach the same dispatch decision, so review-drive state is a shared source of truth rather than per-orchestrator memory.
- ☐ The GitButler engine can serve all review-drive state from `but` without requiring a forge, so an orchestrator can run the whole implement→review→merge loop locally with "keep PRs local" set.

---

## UC-LPR-06: Auto "review-requested" hook via workspace rules

The reconciler needs an assignment to _exist_ before it can dispatch a reviewer — so opening one should be automatic. UC-LPR-06 puts that automation in the **existing `but-rules` (Trigger → Filter → Action) engine**: a commit on a watched branch fires a "review-requested" action that opens a local review assignment. It **extends** the shipped rules surface (the same engine Sprint 06b exposes) with a new commit `Trigger` variant + a review-assignment `Action` variant (today's `but-rules` has neither), and the auto-opened assignment is the same drive-only `local_review_assignments` row UC-LPR-01 defines.

### Acceptance Criteria

- ☐ A human maintainer can configure a `but-rules` rule whose trigger is a commit and whose action opens a local review assignment ("review-requested"), extending the existing Trigger → Filter → Action engine with a new commit trigger + review-assignment action variant (the two variants `but-rules` does not yet have).
- ☐ The GitButler engine can fire the "review-requested" action when a commit matching the rule's filter (branch/principal) lands, creating a `pending` `local_review_assignments` row for the configured reviewer.
- ☐ The GitButler engine can scope the auto-hook via the rule's filter, so only commits on the watched branch (or by the watched principal) open an assignment, not every commit.
- ☐ The GitButler engine can keep the auto-opened assignment as drive-only metadata, so the auto-hook can create assignments without ever blocking a commit or a merge.
- ☐ An orchestrator can observe the auto-opened assignment in `but review status` and dispatch the assigned reviewer, so the auto-hook closes the loop from commit to reviewer-dispatch without an explicit `but review request` call.

---

## UC-LPR-07: Safe-seam invariant — new state never weakens the merge gate

The single most important property of this enrichment, asserted as a _testable invariant_: the merge gate's land-truth is sealed off from every object LPR introduces. The merge gate reads **only** `local_review_verdicts` at the current head; `local_review_assignments` and `local_review_comments` are orchestration metadata that **never gate**. A forged, malicious, or empty drive table cannot change a land decision, because `review_requirement.rs` never reads it — so the existing R6 threat model on the verdict store is neither widened nor narrowed.

### Acceptance Criteria

- ☐ The GitButler engine can decide a merge from `local_review_verdicts` at the current head alone, so the merge-gate code path (`merge_gate.rs` + `review_requirement.rs`) is unchanged by v1.5.0 and reads neither new table.
- ☐ The GitButler engine can leave an open `pending` or `changes_requested` `local_review_assignments` row with no effect on a merge that already satisfies the verdict-at-head requirement, so an assignment never gates the land decision.
- ☐ The GitButler engine can leave an unresolved `local_review_comments` thread with no effect on a merge that already satisfies the verdict-at-head requirement, so a comment never gates the land decision.
- ☐ A maintainer can confirm, via a build-gate grep/test, that no merge-gate code path references `local_review_assignments` or `local_review_comments`, so the drive/gate separation is enforced at build time rather than left to convention.
- ☐ The GitButler engine can preserve the existing R6 threat model unchanged, so a direct DB write to a drive table cannot weaken the merge gate (the gate reads only the verdict store, whose hardening — HMAC → Ed25519, C3 — remains the named follow-up) and the new tables widen no land-truth surface.
- ☐ A maintainer can prove, via test, that a fully forged set of `local_review_assignments` + `local_review_comments` rows produces an identical merge-gate decision to an empty set, so the safe seam holds under an adversarial drive layer.
