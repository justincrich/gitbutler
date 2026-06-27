---
stability: CONSTITUTION
last_validated: 2026-06-26
prd_version: 1.2.0
---

# 08 — Technical Risks (re-ranked register)

The register is **re-ranked from the actions PRD** under the
cheapest-honest-path threat model. The headline change: the mechanism-agnostic
head-OID checkout is promoted to **#1 Blocking**, above the cryptographic
forgery risks — which are **dissolved/downgraded** (deliberately not closed under
the threat model, *not* owed debt).

| ID | Risk | Severity | Status |
|----|------|----------|--------|
| **R-CHECKOUT** | Mechanism-agnostic clean checkout at the head OID | **Blocking** | #1 — the headline engineering problem (07) |
| **R-ENTRY** | Shipped `enforce_merge_gate(ctx, review_id)` is forge-review-keyed; no mechanism-agnostic local-merge entry exists | **Blocking** | Still open after governance closed (GOV-LOCAL added local verdict persistence, not a refs-keyed entry); Check Runner must build it (04 §1a) |
| **R-IDENT** | Gate now resolves the acting principal via the runtime registry; an unregistered process is denied `perm.denied` before the checks clause | Low (assumption) | New since this PRD (governance IDENT landed); the pre-merge runner + merge caller must run as a registered principal (`but agent register`) |
| **R-FAILOPEN** | Required-checks fail-open via the `protected` early-return | **Blocking** | Control-flow correctness |
| **R-SHARESET** | SHA-reset: stale result satisfies the gate after the head moves | **Blocking** | Match `(name, head_oid==current)` only |
| **R-FAILCLOSED** | A failure mode silently allows instead of blocking | **Blocking** | Fail-closed posture |
| **R-BOOTSTRAP** | Config-bootstrap self-escalation (weaken the required set to merge the weakening) | High | Self-protecting required set |
| **R-LYING** | Caller-supplied conclusion / lying producer API | Medium | Negative-space (type + behavioral test) |
| **R-CONCLENUM** | Stringly-typed conclusion | Low | Typed enum, parse-or-fail |
| **R-FORGERY** | Direct-DB-write fabricates a green | **Deliberately not closed** | Reproducibility absorbs it |
| **R-FORGE-BYPASS** (governance **R11**) | Forge-side / raw-push merge bypasses the local gate | Accepted | Inherited from governance R11 |
| **R-NAPI-BYPASS** (governance **R14**) | Ungoverned N-API caller bypasses the gate | Accepted | Inherited from governance R14 |

---

## R-CHECKOUT — mechanism-agnostic clean checkout at the head OID (Blocking, #1)

- **Why it matters:** GitButler is virtual-branches-over-one-worktree. The shared
  tree is dirty and is a projection of several virtual branches — *not* a clean
  checkout of one branch's head OID. Running the check in the repo dir runs it
  against that live projection and **silently breaks the head-OID binding** the
  whole gate depends on: a "green" no longer means "passed at OID A." This is the
  correctness foundation of the product.
- **Mitigation (this slice):** the runner materializes the **exact head OID** into
  a checks-owned **isolated** location — a throwaway `git worktree add --detach
  <oid>`, an object-DB-only `gix` read for working-tree-free checks, or a
  warm-reused worktree on tmpfs (07 §3). The agent's shared worktree is
  **observably identical** before and after; the run does **not** take the shared
  `exclusive_worktree_access()` guard (07 §5). A latency budget governs the
  pre-merge run (07 §4); the gate itself never runs a check.
- **Proof:** integration test — set up a dirty shared worktree at HEAD `B` while
  the check is requested at HEAD `A`; assert the check ran against `A`'s tree (not
  the dirty `B` projection), the result is keyed to `A`, and the shared index +
  HEAD are unchanged (`BUT_WS_LOCK_DEBUG=1`). Run the same check under virtual
  branches, a multi-worktree repo, and a plain-git repo; assert identical results.

## R-ENTRY — shipped gate entry is forge-review-keyed; no local-merge entry exists (Blocking)

- **Why it matters:** the shipped gate entry is
  `enforce_merge_gate(ctx, review_id: usize)`
  (`crates/but-api/src/legacy/merge_gate.rs:75`). It looks up a `ForgeReview` from
  the local forge cache (`review_for_id`, merge_gate.rs:230), derives
  `source_branch`/`target_branch` from it, and resolves `current_head_oid` from
  `review.source_branch` (merge_gate.rs:78). Its only non-test callers are the
  forge PR-merge path (`crates/but-api/src/legacy/forge.rs:1251/1281/1294`). **A
  purely-local virtual-branch / worktree / plain-git `but merge` with no PR has no
  `review_id` and cannot reach the required-checks clause at all** — directly
  contradicting the "gate a local `but` merge, mechanism-agnostic across virtual
  branches AND worktrees" thesis. Governance's GOV-LOCAL work added local
  review-verdict persistence (`local_review_verdicts`, read at merge_gate.rs:241-252)
  around this entry but **did not** add a refs-keyed entry — the gate is still
  keyed on `review_id` — so this remains an **open v1 requirement Check Runner must
  build**, not a governance gap to wait on.
- **Mitigation (this slice):** build a **mechanism-agnostic gate entry point**
  (04 §1a) — `enforce_merge_gate_for_refs(ctx, source_ref, target_ref)` — that
  runs the required-checks evaluation on a resolved `(source_ref, target_ref)`
  pair with the head OID peeled via the `gix` ref-peel (merge_gate.rs:254-261),
  **not** from a `ForgeReview`. The local `but merge` path calls it directly; the
  existing forge path becomes one caller that resolves the refs from its
  `ForgeReview` and delegates. The shipped forge entry remains, but is no longer
  the *only* way in.
- **Auth invariant (MUST NOT regress, 01 §9a):** the new refs-keyed entry MUST sit
  behind the **same `Merge` authorization precondition** the forge path enforces.
  The shipped `enforce_merge_gate` resolves the principal
  (`resolve_principal_with_runtime_registry`, merge_gate.rs:82) and calls
  `but_authz::authorize(&principal, Authority::Merge, &config.gov)?`
  (**merge_gate.rs:91** — the single `Merge`-authority call site) **before** any
  review/checks clause and **before** the `protected` early-return. A
  refs-resolving caller that resolves `(source_ref, target_ref)` but skips this
  `authorize(_, Authority::Merge, _)` clause would be an **authz bypass**, not
  merely a checks bypass. Generalizing the entry to caller-supplied refs runs the
  identical authority + review + required-checks clauses.
- **Proof:** integration test — a local `but merge` of source → target carrying a
  `[[required_check]]`, with **no `ForgeReview` and no `review_id`**, reaches the
  required-checks clause and is blocked `gate.check_required` on a missing result;
  the same merge passes after `but check run` at the head. Run it under virtual
  branches, a multi-worktree repo, and plain git — identical behavior, none
  requiring a forge review. **Auth-invariant proof:** a principal **without
  `Merge` authority** invoking the refs-keyed entry is denied at the
  authorization clause (parity with the forge path's merge_gate.rs:91 denial)
  **before** the checks clause is consulted — confirming the refs entry does not
  bypass authz.

## R-IDENT — registered-principal precondition at the gate (Low, assumption — NEW)

- **Why it matters:** governance's IDENT model (landed, closed) changed how a
  principal is established at the gate. `enforce_merge_gate` now resolves the acting
  principal through the runtime registry: `resolve_principal_with_runtime_registry`
  (merge_gate.rs:82) → `but_authz::resolve_principal_with_registry`, whose chain is
  (1) a runtime PID registry hit, else (2) the `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` env
  handle (a test-only escape hatch), else (3) `Denial::unregistered` (`perm.denied`).
  The static roster mapping principals to authority moved from `permissions.toml` to
  `agents.toml` (legacy `permissions.toml` retained only as a one-release fallback).
  This is **new since this PRD was written** — the required-checks clause inherits it
  as a precondition by composing into `enforce_merge_gate`.
- **Implication:** the pre-merge runner and the merge caller must run as a
  **registered principal** (`but agent register`) or be denied `perm.denied`
  **before** the required-checks clause is consulted. The new refs-keyed entry that
  R-ENTRY must build (04 §1a) **MUST preserve this principal-resolution step** —
  generalizing the entry to caller-supplied refs must not skip the registry resolve
  any more than it skips the `Merge`-authority clause (01 §9a). It is an assumption
  on the deployment (the harness registers its `but` processes), not a Check Runner
  mechanism.
- **Proof:** integration test — an **unregistered** process attempting the governed
  merge is denied `perm.denied` (unregistered) **before** the checks clause runs;
  after `but agent register` with `Merge` authority, the same process reaches the
  required-checks clause (and is then blocked/allowed on the check result, not on
  identity).

## R-FAILOPEN — required-checks fail-open via the `protected` early-return (Blocking)

- **Why it matters:** `enforce_merge_gate` returns `Ok(())` at
  `crates/but-api/src/legacy/merge_gate.rs:103-109` when the target is not flagged
  `protected`, **before** the review clause. A required-checks clause placed after
  that early-return would let a branch carrying `[[required_check]]` but not
  flagged `protected` skip the check entirely — a silent open.
- **Mitigation:** the required-checks clause is reached **independent of**
  `protected` (01 §9, 04 §7): the clause is evaluated for any target that carries
  a `[[required_check]]`, by consulting it before the early-return or restructuring
  the early-return so it does not precede the check evaluation.
- **Proof:** integration test — a target with a `[[required_check]]` but
  `protected = false` (or no `[[branch]]` protected entry at all) and a missing
  result is **blocked** `gate.check_required`. The negative control (clause after
  the early-return) would let the merge through.

## R-SHARESET — SHA-reset staleness (Blocking)

- **Why it matters:** if the gate matched on `name` alone, a `Success` recorded at
  OID `A` would keep satisfying the requirement after a rebase/amend to OID `B`,
  even though the code at `B` was never checked.
- **Mitigation:** match strictly on `(name, head_oid == current_head)` (01 §4,
  03 §1). A result that exists only at a prior OID yields `check_stale_at_head`.
  The old→new basis is `Editor::commit_mappings()`
  (`crates/but-rebase/src/graph_rebase/mod.rs:479`).
- **Proof:** integration test — `Success@A`, rebase `A → B`, assert the merge at
  `B` is blocked `check_stale_at_head`; re-run at `B`, assert it then passes.
- **TOCTOU note (High, sub-risk):** the read (gate) and the merge are not atomic;
  worst on the forge path (a concurrent head-advance between a distributed
  read and merge). **Local same-process window:** even on a single machine, the
  gate's read (`evaluate_required_checks`) and the merge commit are two steps; an
  agent sharing the OS user can INSERT a forged `Success` row in that
  gate-read→merge-commit gap with **no** concurrent head-advance needed — the
  forged row simply has to exist at the matched head OID when the gate reads. This
  local window is **within the named shared-OS-user threat model** — small but
  non-zero, and not closed in v1 (it is the same forgery surface as R-FORGERY,
  reached through timing rather than a stale OID). "Require up to date" /
  merge-queue semantics are deferred (Accepted), consistent with governance's
  posture.

## R-FAILCLOSED — silent allow on a failure mode (Blocking)

- **Why it matters:** any ambiguity that resolves to *allow* is a hole.
- **Mitigation (fail-closed everywhere, 01 §6):** malformed/unsatisfiable config →
  `config_invalid` denial; missing result → `check_missing`; non-`Success`
  (including `Neutral`/`Skipped` on a *required* check) → `check_failed`;
  unresolvable target ref → treated as governed so the loader classifies the fault
  (parity with `governance_present` returning governed on an unresolvable ref,
  `config.rs:199`).
- **Direction note:** the "parity with `governance_present` (governed/`true` on an
  unresolvable ref)" reasoning is **fail-SAFE regardless of the soundness of the
  source decision** — treating an unresolvable ref as governed makes the gate
  **block**, which is the correct direction whether or not governance's own
  classification is itself the right call. The parity is therefore safe to inherit
  even if one disputes the upstream decision: erring toward "governed → blocked" on
  ambiguity is the conservative choice either way.
- **Proof:** integration tests for each: a `[[required_check]]` naming a
  nonexistent check is blocked `config_invalid`; a required check whose only result
  is `Neutral` is blocked `check_failed`.

## R-BOOTSTRAP — config-bootstrap self-escalation (High)

- **Why it matters:** if removing a `[[required_check]]` were not itself gated, an
  agent could weaken the required set in the same change it wants to land — making
  the gate self-defeating.
- **Mitigation:** the required set protects its own weakening (01 §8). Ref-pinning
  the config (reading the committed target tree, never the working tree) is
  **necessary but not sufficient**: the gate evaluates the set **in force before**
  the weakening commit, not the set the weakening commit proposes.
- **Proof:** integration test — a commit that deletes a `[[required_check]]` is
  blocked until the *currently-required* checks pass; ref-pin alone (without the
  before-state evaluation) is the negative control.

## R-LYING — caller-supplied conclusion / lying producer (Medium)

- **Why it matters:** an API that let a caller stamp `Conclusion::Success` without
  running anything would make the conclusion meaningless.
- **Mitigation (negative space, 04 §4):** no public function accepts a
  caller-supplied conclusion; `Conclusion::from_exit` requires a real
  `std::process::ExitStatus`. Runner ≠ agent (01 §2).
- **Proof:** behavioral negative test (not just a grep) — there is no API path by
  which an agent stamps a `Success` row for an unrun check. (A direct DB write
  still can — that is R-FORGERY, deliberately absorbed by reproducibility.)

## R-CONCLENUM — stringly-typed conclusion (Low)

- **Mitigation:** `Conclusion` is a typed enum (03 §4) mirroring `CiConclusion`;
  an unknown token is a parse error, never a silent pass. The gate's pass
  predicate is `Conclusion::is_passing` (== `Success`), not a string compare.

## R-FORGERY — direct-DB-write fabricates a green (DELIBERATELY NOT CLOSED)

- **Stance:** under the personal-tenant, own-fleet, cheapest-honest-path threat
  model, we **do not** try to make fabrication impossible. A green is
  **reproducible**: anyone can re-run the ref-pinned check at the OID and catch a
  forged result, and the honest path is cheap. **Consistency argument:** governance
  already gates merges on `local_review_verdicts`, which it accepts as
  forgeable-by-direct-DB-write (its R6); a check is a *reproducible* second review
  — safer in detectability (forgery detectable post-merge), **not** strictly safer
  (no principal identity) — so `check_results` needs **no more protection** than the
  review store: a plain table is correct (01 §3, 03 §1).
- **This is a deliberate non-goal, not owed debt.** The dropped mechanisms
  (signing / HMAC / Ed25519 / agent-unwritable hardening / OS-sandbox-as-security)
  are not "to be added later"; they are out of scope by design. The nullable
  `signature` column is the only forward-compat seam, and only for the day
  producers run off-host (where direct-DB-write stops being the cheap attack) — it
  is **not** a v1 security control (01 §5).
- **Reconciliation with the `signature` seam (explicit):** the seam is for the
  **off-host-producer** scenario, **not** a planned closure of this risk under the
  **current shared-OS-user threat model**. Under that model a signature would not
  change the gate's merge-time decision (the agent can still write the row), so
  R-FORGERY stays **deliberately open** here — the column is not a scheduled fix
  for it. Do not read the seam's existence as "forgery will be closed in a later
  version under the same threat model"; it only becomes load-bearing once the
  producer moves off-host (01 §5).

## R-FORGE-BYPASS (governance R11) / R-NAPI-BYPASS (governance R14) (Accepted, inherited from governance)

- **The local gate binds the governed local merge only.** Forge-side / raw-push
  merges (**governance R11**) and ungoverned N-API callers (**governance R14**)
  can bypass the local gate exactly as they bypass the governance review gate
  today. Check Runner does not widen or narrow this boundary; it inherits
  governance's accepted posture verbatim. The raw-git / `--no-verify` fence is
  **governance R1** and is likewise inherited. The local gate is the enforcement
  point for the local CLI/desktop merge path; it makes no claim over the forge,
  raw-push, or N-API surfaces.

## Relationship to the governance risk register

Check Runner's gate **composes with** governance's, terminating in the same
read-only gate. It inherits governance's accepted boundaries — forge/raw-push
merge bypass (**R11**), ungoverned N-API bypass (**R14**), and the raw-git /
`--no-verify` fence (**R1**) — and adds only the execution-axis risks above
(plus R-ENTRY, the local-merge entry point that must be built because the shipped
`enforce_merge_gate` is forge-`review_id`-only, and R-IDENT, the new
registered-principal precondition governance's IDENT model introduced at the
gate). The forgery risk is resolved by
the **same** consistency argument governance uses for its review store — not by
new cryptography.

## Cross-references

- The #1 risk in full engineering detail: [`07-mechanism-agnostic-checkout.md`](./07-mechanism-agnostic-checkout.md)
- The fail-open control-flow fix: [`01-architecture-posture.md`](./01-architecture-posture.md) §9
- The reproducibility argument that dissolves R-FORGERY: [`01-architecture-posture.md`](./01-architecture-posture.md) §3
- The capability chains these risks live on: [`09-capability-chains.md`](./09-capability-chains.md)
