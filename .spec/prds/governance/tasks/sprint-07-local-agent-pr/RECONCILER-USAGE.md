# Reconciler usage-model + the `but-*` skill contract

> Sprint 07 (LPR) — companion to LPR-006 (`keep_reviews_local`) and LPR-008
> (`review_status` reconciler read-API). PRD UC-LPR-05. Capability CAP-AUTHZ-01.
>
> **This is a documentation task.** It writes the usage-model and the skill
> **contract** that future `but-*` orchestration skills will honor. It
> implements no behavior and changes no code. The behavior it describes is built
> and tested by LPR-006 (the `keep_reviews_local` field) and LPR-008 (the
> `review_status` reconciler read-API). The `but-*` skill **implementation** is
> OUT of scope for this sprint (AC-2) — this doc states the contract only.

---

## 1. The reconciler model — the orchestrator is a reconciler over `but`, not a private state machine

The thesis of UC-LPR-05 is that an orchestrator (the future `but-*` skills in
the kb / `but-run-sprint` family, or a human driving the same loop) does **not**
keep a private shadow state of where a review is. It reads one payload —
**`review_status`** — and its next action is a pure projection of that payload.
Every decision the orchestrator makes is observable in `but`'s own state, read
deterministically, so two orchestrators on the same repo converge on the same
next action.

### 1.1 What the reconciler reads

`crate::legacy::forge::review_status(ctx, branch)` (LPR-005 + LPR-008) serves
the **full drive state** for a target branch in **one** branch-scoped read.
The payload carries:

| Field                          | Drive fact              | Source query (reused, never forked)                                                                                                                                  |
| ------------------------------ | ----------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `open_assignments`             | **dispatch trigger**    | `local_review_assignments.list_by_target(target)` filtered to `state == "pending"`, preserving the Handle's `assigned_at ASC, id ASC` order                          |
| `unresolved_threads`           | **remediation trigger** | `local_review_comments.list_by_target(target)` grouped by `thread_id` where at least one comment has `resolved = false` (the reserved `__pr_meta__` thread excluded) |
| `verdict_at_head` / `approved` | **merge trigger**       | `local_review_verdicts.list_by_target(target)` filtered to `head_oid == current_head_oid && verdict == "approved"` — the **exact** query `enforce_merge_gate` runs   |

Plus the derived PR lifecycle fields (`target`, `assignments`, `lifecycle`,
`agent_authored`, `open_threads`) that LPR-005 already carried.

A target with **no** drive state returns an `Ok` payload with empty vecs and
`verdict_at_head = None` — a clean empty-state, never an `Err`.

### 1.2 The read-then-act loop

The orchestrator runs a single read-then-act loop. Each branch is a pure
function of the payload:

| `review_status` payload                                         | Orchestrator action                                                                                                                                        |
| --------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `open_assignments` is non-empty (a `pending` assignment exists) | **Dispatch a reviewer** — wake / assign the reviewer principal named in the assignment row to drive the review toward a verdict.                           |
| `unresolved_threads` is non-empty (an unresolved thread exists) | **Dispatch remediation** — wake the branch author (or the assigned reviewer for that thread) to address the thread, then `but review resolve` to clear it. |
| `approved == true` (`verdict_at_head == Some("approved")`)      | **Attempt the governed merge** — call the governed `but merge <branch>`, which routes through `enforce_merge_gate` and re-derives verdict-at-head itself.  |
| None of the above                                               | **Wait** — there is no drive work to do; the loop idles on the next read.                                                                                  |

The branches are not mutually exclusive in the payload (an open assignment and
an unresolved thread can coexist) — the orchestrator chooses the highest-priority
remediation first, but the priority order is the orchestrator's own policy, not
a `but` concern. `but`'s only contract is to surface the three facts in one
deterministic payload.

### 1.3 The two-read agreement proof (why two orchestrators converge)

Two independent orchestrators reading `review_status` against the same
unchanged repo state yield **byte-identical** payloads. This is LPR-008 AC-5,
proven by `cargo test -p but-api two_reads_of_review_status_converge`. The
determinism contract has three load-bearing properties:

1. **No per-orchestrator in-memory state.** Every drive fact is read fresh from
   the `but-db` cache on each call; no field in the payload is computed from a
   per-call scratch or a sticky label.
2. **`Vec` everywhere, deterministic ordering.** Every collection in the payload
   reuses a Handle's existing `ORDER BY` (`local_review_assignments`,
   `local_review_comments`, and `local_review_verdicts` all `ORDER BY created_at
ASC, id ASC`). The thread grouping sorts by `thread_id`. No `HashMap` /
   `HashSet` iteration order leaks into the payload.
3. **No per-call timestamps / UUIDs in compared positions.** Every compared
   field is sourced from a stored row, not synthesized at read time.

This is what makes the orchestrator a **reconciler over `but`'s own state**:
two reconcilers reading the same cluster state converge by construction. There
is no "first orchestrator wins" race, no orchestrator-local opinion that
diverges from the engine — `but`'s `review_status` IS the shared source of
truth, and the reconciler pattern means the orchestrator never owns a parallel
copy of it.

> **Anti-pattern:** a private orchestrator state machine that caches
> "is branch X approved?" between reads and acts on the cache. The reconciler
> pattern forbids this — every decision is a projection of a fresh
> `review_status` read. Caching is an orchestrator-local optimization with a
> cache-invalidation surface; the contract is "read every iteration."

### 1.4 When to write vs read — the read-only reconciler surface

The reconciler read surface is strictly **read-only**. `review_status` shares
the read posture of `governance_status_read` / `get_review` (`forge.rs:401`) —
it discloses the whole branch's review drive state (every principal's
assignments / threads / verdict on the named branch, an accepted branch-scoped
disclosure, F-006), but it **writes nothing** and acquires **no write
authority**.

| Verbs that mutate (the reconciler dispatches INTO these)                             | Verbs that read (the reconciler surface)                  |
| ------------------------------------------------------------------------------------ | --------------------------------------------------------- |
| `request_review`, `assign_reviewer` (`ReviewsWrite`)                                 | `review_status` (LPR-005 / LPR-008) — the reconciler read |
| `post_comment`, `resolve_thread` (`CommentsWrite`)                                   | `get_review` (`forge.rs:401`) — the branch-scoped read    |
| `approve_review`, `request_changes_review` (`ReviewsWrite`)                          | `governance_status_read` — the broader governance read    |
| `merge_review` → `enforce_merge_gate` → the governed `but merge` (`Merge` authority) |                                                           |

The orchestrator's job is to **dispatch the write verbs in response to the
read payload**, never to bypass them. In particular the approved-verdict-at-head
read **does not authorize a merge** — see §1.5.

### 1.5 The safe-seam invariant — the read AGREES with the gate, never replaces it

The single most load-bearing property of this model:

> **"Gate gates (verdict-at-head, untouched); new tables drive (orchestration)."**

`review_status`'s `approved == true` label is **presentation only**. The actual
land stays `enforce_merge_gate`'s own re-derivation of verdict-at-head — the
**same** `local_review_verdicts.list_by_target(target)` query filtered to
`head_oid == current_head_oid && verdict == "approved"`, run again, inside the
gate. The read and the gate **agree** because they read the same truth; the
read does not replace the gate, and a merge is never authorized on the
`review_status` label alone.

This is what LPR-009 proves:

- **The static no-read proof** — `cargo test -p but-authz
safe_seam_gate_path_reads_no_new_table`: a build-gate honesty grep over
  `merge_gate.rs` + `review_requirement.rs` finds **zero** references to
  `local_review_assignments`, `local_review_comments`, or `local_review_meta`.
  The drive tables never participate in the land decision.
- **The runtime equivalence proof** — `cargo test -p but-api
safe_seam_forged_drive_equals_empty_drive`: a fully forged drive layer (all
  assignments approved, all comments resolved, written directly) yields an
  **identical** merge-gate decision to an empty drive layer for every
  verdict-at-head fixture.
- **The inverse proof** — `cargo test -p but-api
safe_seam_only_verdict_at_head_flips_the_land`: with no approved
  verdict-at-head, drive metadata alone (a pending assignment + an unresolved
  thread) still cannot land.

An orchestrator that trusts its `review_status` read and dispatches the
governed `but merge` is therefore safe by construction: if the read says
"approved at head", the gate will too (same query, same row); if the read says
"not approved", the orchestrator does not dispatch a merge. The reconciler
never finds itself in the position of having merged on a label the gate later
contradicts.

> See **LPR-009** (`LPR-009-safe-seam-invariant.md`) for the full proof, the
> honesty-grep discipline, and the threat-model preservation argument.

---

## 2. The agent-PR tag — sourced from the opener principal's DECLARED `kind`

The `ReviewStatus.agent_authored` boolean (LPR-005) is the agent-PR tag — a
descriptive label that surfaces whether the PR was opened by an `agent`-kind
principal or a `human`-kind principal. The tag's source-of-truth is the
**opener principal's declared `kind`** in committed
`.gitbutler/permissions.toml`, read at the target ref. It is:

- **NOT handle-inferred.** `BUT_AGENT_HANDLE` resolves _which_ declared
  principal acts; it does not and cannot assert a `kind`. The tag is never a
  function of the env handle's mere presence.
- **NOT a comment-body sentinel.** The reserved `__pr_meta__` thread id is
  explicitly **rejected** as the opener storage — a comment body is
  attacker-influenceable free text (R20), so making it a control-plane input
  would let any `local_review_comments`-write actor forge the opener and flip
  the tag.
- **NOT a caller argument.** There is no `--agent` flag and no `--kind` flag
  on `but review request`. The opener row is written by the governed
  `request_review` via `INSERT … ON CONFLICT(target, key) DO NOTHING` (the
  `UNIQUE(target, key)` blocks a later overwrite through the governed path).
- **NOT an enforcement key.** No gate reads it; the `kind` field does not
  enter `GovConfig.principals` and changes no enforcement.

The computed tag is **cached** in a dedicated `local_review_meta(target,
"opener_principal", …)` row (the F-003 storage, LPR-001) — a per-target
structured metadata row, separate from the free-text comment surface. The
**source-of-truth** for "is this agent-authored" remains the committed
principal `kind`; the cached row is a tag-derivation control path, not a
source of authority.

This framing matters because three of the named residuals (R19, R20, R23) are
about this exact control path — see §4.

---

## 3. The `but-*` skill contract — `keep_reviews_local = true` on governed-project init

> **Contract:** _"on governed-project init, if unset, set
> `keep_reviews_local = true`."_

This is the contract that the future `but-*` agent-orchestration skills (the
kb / `but-run-sprint` family that dispatches agent principals) will honor when
initializing a governed agent project. It is a **skill-side default**, not a
governance enforcement and not an authorization boundary.

### 3.1 Why it is a skill-side default (belt-and-suspenders)

The `keep_reviews_local` field (LPR-006) is a `DefaultTrue` per-project
preference on `gitbutler_project::Project` (`project.rs`, beside
`ok_with_force_push` at `:106` and the forge knobs at `:129` / `:134`).
`DefaultTrue` means:

- **The default is `true` = local.** A new project created via
  `Project::default_with_id(id)` is local.
- **Old project files without the field deserialize to `true` = local** via
  `#[serde(default)]` + `DefaultTrue`. (This is the exact precedent
  `ok_with_force_push` follows. The plain-`bool` siblings
  `force_push_protection` / `husky_hooks_enabled` are the **anti-precedent** —
  they default `false`, the wrong default here.)

Because the field is already local-by-default at the storage layer, the
skill's set-`true`-on-init action is **belt-and-suspenders** — it makes the
intent explicit in the stored project (so a project inspected later reads the
explicit `true` rather than relying on the implicit `DefaultTrue`), but it
adds no boundary the storage layer does not already provide.

### 3.2 Why it is NOT a governance enforcement or an authorization boundary

The skill contract is **explicitly framed** as a skill-side default, not a
governance enforcement, because that is what `keep_reviews_local` IS:

- `keep_reviews_local` is a **per-project operator preference under the R12
  trusted-desktop model**, persisted in the project store (the same path that
  sets `forge_override`). It is the **same class** as `forge_override` /
  `preferred_forge_user`.
- It is **NOT** `administration:write`-gated. The setter composes no
  `enforce_administration_write_gate` and surfaces no `perm.denied` for this
  preference.
- It is **NOT** ref-pinned committed config (those are `permissions.toml` /
  `gates.toml`, which gate _decisions_). It lives in the project store, which
  gates nothing.
- It is **NOT** read by the merge gate. The gate reads only
  `local_review_verdicts` at head — `keep_reviews_local` is not in the gate's
  read set (LPR-009).

An operator who wants remote mirroring sets `keep_reviews_local = false`
themselves through the project-settings surface. The skill does not fight
that, and it must not refuse to honor it — the skill's set-`true`-on-init is a
default at init time, not a re-applied override later.

### 3.3 What "the reconciler is the orchestrator's source of truth — NOT the forge" means

The combination of the reconciler model (§1) and the local-by-default skill
contract (§3.1–§3.2) makes `but`'s local review state the orchestrator's
**single source of truth**. Concretely:

- An orchestrator running the loop **does not consult a forge** (GitHub /
  GitLab) for review status. It reads `review_status` on the local repo, and
  the local payload is authoritative.
- An agent-authored PR opened under `keep_reviews_local = true` **does not
  open a remote forge PR** (LPR-006 AC-3) — the loop stays local.
- The **deferred remote-mirror seam** (tech-delta §D, LPR-006 AC-4) is
  specified-not-built. When (and if) it is built, mirroring will be reachable
  only when `keep_reviews_local == false` (the gate flips off), and even then
  the reconciler still reads the local layer — the mirror is a publishing
  path, not a parallel source of truth.

This is why `keep_reviews_local = true` on init is the contract: it makes the
default be the loop the orchestrator is built to drive. The orchestrator is
reconciling over `but`'s state, and `but`'s state is local.

### 3.4 The skill IMPLEMENTATION is OUT of scope for this sprint

This sprint (Sprint 07 LPR) writes the **documented contract** only. The
actual `but-*` skill code that:

- reads `review_status` to drive the dispatch / remediation / merge loop, and
- sets `keep_reviews_local = true` on governed-project init,

is **NOT built here**. The skill implementation is a future task. This doc is
the seam those skills will later honor. A reader must NOT mistake this doc for
a shipped skill — it states the contract, and the contract is what the future
skills will be measured against.

The skill implementation is explicitly **OUT of scope** for this sprint.

---

## 4. Named leaks — accepted residuals, never closed

This doc follows the PRD's **"name your leaks, never quietly mitigated-closed"**
doctrine. Presenting any of the residuals below as a hardened boundary is the
**cardinal misrepresentation** — the same misrepresentation class as R1 (the
fence) and R6 (the verdict store). Each residual is named here as an
**accepted residual**, with its mitigation (real, where one exists) and its
deferred closure (named, not done). The doc makes **NO claim** that:

- the local PR is independently audited (it is not — see R18);
- the agent tag is a trustworthy authorship attestation (it is not — see R19);
- comment bodies are injection-safe (they are not — see R20);
- `keep_reviews_local` is an authorization boundary (it is not — see R21);
- a same-principal drive trail is multi-party review (it is not — see R22);
- the agent tag is tamper-proof (it is not — see R23).

### R18 — Local-review receipt is loop-sourced, not independently audited

**Statement.** The local PR loop's "approved" status is recorded in
`local_review_verdicts` by the governed `approve_review` (`forge.rs:534`), but
no CLI/engine path re-reads the verdict store to _re-verify_ a receipt
out-of-band. The audit receipt is **loop-sourced** — produced by the same loop
that wrote it. An agent with direct DB/filesystem write can INSERT a forged
approving row (the **exact R6 leak**, `03-data-schema.md` integrity caveat),
and the new local-PR surface gives that forgery a friendlier UX (a "PR" that
looks reviewed).

**Mitigation (real).** Approvals land only via the governed `approve_review`
action; the governed path is what the gate tests exercise.

**Residual (accepted, named).** A direct DB write forges approval — **same
accepted-leak class as R1 (the fence) and R6 (the verdict store)**.

**Deferred closure (named, not done).** The R6 hardening (HMAC keyed by a
repo-local admin secret, then Ed25519-signed review artifacts) **plus** an
independent `but review verify` re-read that does not trust the writing loop.

### R19 — The agent tag is spoofable via `BUT_AGENT_HANDLE` re-export

**Statement.** The tag (§2) is derived from the opener principal's **declared
`kind`** in committed `.gitbutler/permissions.toml` (read at the target ref),
so an actor **cannot self-assert agent/human via a bare env var** — there is no
`--kind` flag and no env input to the tag; `kind` is config-declared. **But**
the env handle (`BUT_AGENT_HANDLE`) still selects _which_ declared principal
acts: a sub-process that re-exports `BUT_AGENT_HANDLE` to a **different
handle** (the R2 identity residual, `07-technical-risks.md`) acts **as that
other declared principal and inherits its declared `kind`**. The
mis-attribution is bounded to principals already in committed config (an
actor cannot conjure an arbitrary kind), but impersonating a _different
declared principal_ to borrow its kind is not closed.

**Mitigation (real).** The tag is computed from the opener's declared config
`kind`, never a caller arg, never the env handle's mere presence (mirrors
`04-api-design.md` "never from an agent-supplied claim"); in-band `--as` is
denied (UC-AUTHZ-03); the `kind` field is read at the target ref
(anti-self-escalation).

**Residual (accepted, named).** Sub-process `BUT_AGENT_HANDLE` re-export to
impersonate a _different declared principal_ (and borrow its kind) is **not**
closed — **same accepted-leak class as R2**.

**Deferred closure (named, not done).** Per-agent key-mint.

### R20 — Comment-body injection into agent context

**Statement.** `local_review_comments.body` is attacker-influenceable free
text written by one agent principal and **read as context by another**
(`list_comments` / `review_status`). A crafted body can attempt
prompt-injection against a downstream agent that ingests review threads — the
**same injection class** the v1.4.0 delta named for `message` / `unmet[]`
(R15 there), now reaching agent context through comment bodies.

**Mitigation (real).** Comment bodies are **data, never code**: the governance
layer never interpolates them into a decision (unlike the closed-catalog
`&'static str` denial fields), and the agent-tag control path is the
**declared config `kind` + a dedicated `local_review_meta` row, not a comment
body** (§2), so a comment body cannot reach the tag derivation. Downstream
agent harnesses that surface comments to a model should bound/escape them —
an **L2 harness concern**, out of GitButler's grip (Stance 6).

**Residual (accepted, named).** GitButler stores and serves the raw body; it
does not sanitize it for arbitrary downstream consumers.

**Deferred closure (named, not done).** None at the GitButler layer — this is
a permanently-accepted L2/harness residual, surfaced for downstream consumers
to handle.

### R21 — `keep_reviews_local` is a trusted-desktop preference, not an authorization boundary

**Statement.** `keep_reviews_local` (LPR-006, §3 above) is a per-project
`Project` preference under the **R12 trusted-desktop** model — **not**
`administration:write`-gated, **not** ref-pinned committed config. An
**untrusted write to the project store** (a compromised desktop session, or
any process that can write `gitbutler-project` storage) can **flip it to
`false`**, after which agent-authored PRs begin **mirroring to a public
forge** (tech-delta §D) instead of staying local — a confidentiality flip the
governance layer does not check. Sub-point (F-005, deferred-seam disclosure):
when mirroring is enabled, the §D `reviewer_principal → forge reviewer list`
mapping **discloses internal governance principal identifiers to a public
forge API**, with mapping-failure modes (no forge account for a principal,
stale/renamed handle, many-to-one collapse) that can drop, mis-attribute, or
leak the raw internal handle.

**Mitigation (real).** Default is `true` (local) via `DefaultTrue`; the
desktop human is the trusted fleet owner (R12) who owns the preference; while
`true`, no principal identifier crosses to any forge.

**Residual (accepted, named).** An untrusted project-store write flips the
flag — **same accepted-leak class as R12** (trusted-desktop, not a checked
grant); and the deferred mirror's principal→forge-identity mapping is a real
disclosure + failure surface that must be designed fail-closed when built.

**Deferred closure (named, not done).** None at the preference layer — the
residual is the cost of the trusted-desktop model. The mirror's
principal→forge-identity mapping is designed fail-closed when the mirror is
built (which is itself deferred — tech-delta §D).

### R22 — Same-principal drive-layer forgery (self-assign / self-resolve)

**Statement.** The drive layer (assignments, comment threads) is read by the
reconciler to decide "who reviewed" and "is it all-clear". Two same-principal
forkeries exist at the _drive_ layer (the _gate_ still catches the verdict via
its own distinct-from-author check, but the drive narrative is what an
orchestrator and a human read):

- **(a) Self-assign** — an implementer self-assigns as its own reviewer
  (`assign_reviewer` with `reviewer == author`) → the PR object falsely reads
  "independently reviewed".
- **(b) Self-post-and-self-resolve** — a reviewer posts a
  `changes_requested`-style thread and immediately self-resolves it
  (`resolve_thread` by the same principal that authored it) → the reconciler
  reads a forged "all-clean" signal and suppresses remediation for another
  party.

**Mitigation (real, implemented at the `but-api` boundary).**
`assign_reviewer` enforces `reviewer != author_principal_of_target_branch`
(the gate's `require_distinct_from_author`, mirrored at the drive layer);
`resolve_thread` requires the resolver to be the **thread author, the assigned
reviewer, or a `reviews:write` holder** — a self-posted-and-self-resolved
thread does not suppress the remediation signal for a distinct party. Proven
by T-LPR-043 (self-assignment rejected) and T-LPR-044 (self-posted +
self-resolved thread does not suppress another party's remediation signal).

**Residual (accepted, named).** A single principal that legitimately holds
both `reviews:write` and authorship can still act on both sides; the drive
layer narrows **cross-principal** forgery but cannot make a one-principal repo
multi-party.

**Deferred closure (named, not done).** Multi-party review at the drive layer
is a policy / operational concern, not a single technical closure.

### R23 — DB-row forgery of the agent-tag derivation control path

**Statement.** The agent-PR tag is derived from the opener's declared config
`kind` and cached in a dedicated `local_review_meta(target,
"opener_principal", …)` row (§2), chosen specifically so the tag is **not**
sourced from an attacker-influenceable comment body (which would be R20). But
the `local_review_meta` row is itself a DB row: an actor with **direct
DB/filesystem write** can INSERT/overwrite the opener row (subject to the
`UNIQUE(target, key)` write-once-on-conflict) to forge the cached
tag-derivation input. This is **distinct from R20** (R20 is comment-body →
_agent context_ injection; R23 is forgery of the _tag-derivation control
path_ itself). The committed-config `kind` source is itself read at the
target ref and not forgeable in the working tree, but the cached opener row
in the local DB is.

**Mitigation (real).** The opener row is written **once** by the governed
`request_review` via `INSERT … ON CONFLICT(target, key) DO NOTHING` (the
`UNIQUE(target, key)` blocks a later overwrite through the governed path);
the tag is never a caller arg; the declared `kind` source is read at the
target ref.

**Residual (accepted, named).** A direct DB write that races the first opener
insert, or edits the cached row out-of-band, forges the tag — **same
accepted-leak class as R6 / R18** (the verdict store is forgeable by direct
DB write; so is this cached metadata row).

**Deferred closure (named, not done).** The same R6 integrity hardening
(HMAC/Ed25519 over the local review artifacts), extended to the meta row.

### 4.7 Honesty doctrine

R18's loop-sourced-receipt forgeability, R19's tag spoofability (via
impersonating a different declared principal), R21's preference-not-boundary
flip, and R23's tag-control-path forgery must **stay named, never quietly
"mitigated" into looking closed** — presenting any as a hardened boundary is
the same misrepresentation class as R1 / R6. R22's drive-layer
distinct-from-author / resolver-identity constraints are real integrity
checks (and are tested — T-LPR-043, T-LPR-044), but they narrow
**cross-principal** forgery only — they do not make a single-principal trail
multi-party, and that residual stays named. R20 is an accepted L2/harness
residual, not a closed boundary.

---

## 5. References

- **LPR-006** — `keep_reviews_local: DefaultTrue` field, default-local wiring,
  remote-mirror gate. `.spec/prds/governance/tasks/sprint-07-local-agent-pr/LPR-006-keep-reviews-local-setting.md`.
- **LPR-008** — `review_status` reconciler read-API; the full drive state
  (open assignments + unresolved threads + verdict-at-head) in one payload.
  `.spec/prds/governance/tasks/sprint-07-local-agent-pr/LPR-008-reconciler-read-api.md`.
- **LPR-009** — the safe-seam invariant. The honesty grep that proves the gate
  path reads none of the three new tables; the forged-vs-empty equivalence;
  the inverse proof.
  `.spec/prds/governance/tasks/sprint-07-local-agent-pr/LPR-009-safe-seam-invariant.md`.
- **tech-delta §C** — the `keep_reviews_local` skill contract.
  `.spec/prds/governance/enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md`.
- **tech-delta §E** — the safe-seam invariant statement.
- **tech-delta §G** — the R18 / R19 / R20 / R21 / R22 / R23 risk register and
  the honesty-test note.
- **forge.rs** — the `ReviewStatus` payload and the `review_status(ctx,
branch)` function. `crates/but-api/src/legacy/forge.rs`.
- **merge_gate.rs / review_requirement.rs** — the verdict-at-head query the
  reconciler read reuses. `crates/but-api/src/legacy/`.
- **PRD UC-LPR-05** — the reconciler thesis.
  `.spec/prds/governance/enrichments/v1.5.0-local-agent-pr/02-uc-lpr.md`.
