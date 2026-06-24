# LPR-005: `review_status` derived PR lifecycle (commits + verdict-at-head + open assignments; read-only `gix` walk, NO mutation) + the agent-PR tag derived from the opener's DECLARED `kind` in committed config (cached in `local_review_meta`) + the additive `kind` AUTHZ-config descriptor

> Status: ✅ Completed
> Commit: 975ce7e5db
> Reviewer: deferred to PHASE 4.5 red-hat closeout — committed prior session; review_status derived PR lifecycle + agent tag
> Updated: 2026-06-22T18:07:12Z

## What this does

Add the `review_status(ctx, branch)` self-scoped read that computes a **derived** local-PR object at query time over three already-present sources — the branch's commits ahead of base (a **read-only** `gix` graph walk, NO mutation), `local_review_verdicts` filtered to the verdict-at-head (the EXACT query `merge_gate` runs), and the open `pending` `local_review_assignments` + unresolved `local_review_comments` — yielding a derived status ∈ `{ Draft, AwaitingReview, ChangesRequested, Approved, Mergeable }`. There is **no `local_pull_requests` table**: the PR is a projection, never a fourth stored truth. The object mirrors the relevant `ForgeReview` fields and carries an `agent-authored` tag whose **source-of-truth is the opener principal's DECLARED `kind` in committed `.gitbutler/permissions.toml`** (read at the **target ref** — anti-self-escalation): the opener principal id is read from the `local_review_meta` opener row (LPR-003), that principal's committed entry is resolved at the target ref, and `agent_authored = true` **iff it declares `kind = "agent"`** (omitted → human). This task also adds the **additive optional `kind` field on `PrincipalWire`** in `but-authz` config parsing — a descriptor that **changes no enforcement** (no gate reads it; it does NOT enter `GovConfig.principals`). The computed tag is cached in the `local_review_meta` opener row — never a caller arg, never read by a gate, never handle-inferred. **`Approved`/`Mergeable` is a presentation label only; the merge decision stays `enforce_merge_gate`, which re-derives verdict-at-head itself and never reads this view.**

## Why

Sprint 07 · PRD UC-LPR-01, UC-LPR-04, UC-LPR-05 · capability CAP-AUTHZ-01. An orchestrator needs an inspectable local-PR object (lifecycle + assignments + the agent tag) to drive dispatch, and a human needs it to distinguish agent PRs from human PRs. Deriving the lifecycle keeps a **single source per fact** (commits + `local_review_verdicts`) — storing it would duplicate land-truth and invite drift from the gate (the failure `WORKSPACE_MODEL.md` warns against). The agent tag's source-of-truth is a **declared config fact** — the opener principal's `kind` in committed `permissions.toml`, read at the target ref — NOT handle-resolution: `resolve_principal` keys solely on `BUT_AGENT_HANDLE` and `Principal` is `{ id, authorities, groups }` with **no `is_agent`/`PrincipalKind` discriminator** (`crates/but-authz/src/authorize.rs:67`–`:90`, `principal.rs:82`), so deriving the tag from "the opener resolved from `BUT_AGENT_HANDLE`" cannot tell agent from human and would be a fabrication (tech-delta §A.4). The declared `kind` is read at the target ref like all governance config (`config.rs:23`–`:25`, anti-self-escalation). It is **not** a trustworthy authorship attestation: an actor can re-export `BUT_AGENT_HANDLE` to impersonate a _different declared principal_ and borrow its kind (R19), and the cached `local_review_meta` opener row is forgeable by a direct DB write (R23) — both stay named.

## How to verify

PRIMARY **AC-1** — `cargo test -p but-api review_status_is_derived_not_stored`: `review_status` returns a derived status (e.g. `AwaitingReview`) computed at query time from commits + verdict-at-head + open assignments; FAIL if a `local_pull_requests` table or any stored-lifecycle row exists (the PR object must be a projection, not a 4th truth). Full gate set in the spec below.

## Scope

- crates/but-authz/src/config.rs (MODIFY — add the additive optional `#[serde(default)] pub kind: Option<String>` field on `PrincipalWire` (config.rs:424, `#[serde(deny_unknown_fields)]`), mirroring the existing `role: Option<String>` (config.rs:429); expose a typed reader so the loaded config can answer "is principal X declared `kind = agent` at this target ref"; this descriptor does NOT enter `GovConfig.principals` (config.rs:85) and NO gate reads it)
- crates/but-api/src/legacy/forge.rs (MODIFY — add `review_status` self-scoped read fn beside `get_review` (forge.rs:401) + the private derivation helper `derive_pr_state(repo, branch, db) -> DerivedPr`; reuse the EXACT verdict-at-head query `merge_gate` runs and the `ForgeReview` field shape; the agent-tag derivation reads the `local_review_meta` opener row, then resolves that opener's committed `kind` at the target ref via the new config reader)
- crates/but/src/command/legacy/forge/review.rs (MODIFY — add the `status` CLI verb (→ `review_status`) beside approve/comment/close (review.rs:20/:55/:73); route errors through review_gate_cli_error (review.rs:89))
- crates/but/src/args/ (MODIFY — the `but review status <branch> [--agent-authored]` verb/arg definitions; NOT but-clap per tech-delta §B)
- crates/but-api/tests/review_status.rs (NEW — the PRIMARY but-api proofs AC-1..AC-6 against a real but-db + gix fixture via but_testsupport, hand-assertion style like merge_gate/governed_loop)
- packages/but-sdk/src/generated/\*\* (REGENERATE ONLY via `pnpm build:sdk && pnpm format` — NEVER hand-edit; the actual regen + tag-not-an-enforcement-key grep is LPR-010's gate)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: LPR-005 — review_status derived PR lifecycle + agent-PR tag from the opener's declared kind (cached in local_review_meta) + the additive kind AUTHZ-config descriptor
================================================================================

TASK_TYPE:   FEATURE
STATUS:      Backlog
PRIORITY:    P0
AGENT:       implementer=rust-implementer | reviewer=rust-reviewer
EFFORT:      L  (180 min)
PROPOSED-BY: rust-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-LPR-01, UC-LPR-04, UC-LPR-05
CAPABILITIES:CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-api review_status_is_derived_not_stored
  check: cargo check -p but-api --all-targets
  lint:  cargo clippy -p but-api --all-targets

--------------------------------------------------------------------------------
TYPE DESIGN (Rust shapes this task introduces)
--------------------------------------------------------------------------------
API SURFACE (additive #[but_api(napi)] fn — a SELF-SCOPED READ, no write authority, mirrors governance_status_read / get_review forge.rs:401):
  - `fn review_status(ctx: &Context, branch: String) -> Result<DerivedPr>` (sync like get_review forge.rs:401; promote to `async fn(ctx: ThreadSafeContext, …)` ONLY if it needs the repo head off the thread-safe context, mirroring approve_review's into_thread_local() frame). Authorize is the pre-call guard, but the action is a self-scoped read (NO write authority — like governance_status_read), disclosing only the caller's-target review surface.
AUTHZ CONFIG DESCRIPTOR (additive, enforcement-neutral — crates/but-authz/src/config.rs):
  - `PrincipalWire` (config.rs:424) gains `#[serde(default)] pub kind: Option<String>` — the EXACT optional-field pattern `role: Option<String>` already uses (config.rs:429). Because `PrincipalWire` is `#[serde(deny_unknown_fields)]` (config.rs:423) the field MUST be declared on the wire struct; `#[serde(default)]` means older committed `permissions.toml` files without `kind` deserialize cleanly (`None` → human). It does NOT flow into `GovConfig.principals: BTreeMap<PrincipalId, AuthoritySet>` (the enforcement map, config.rs:85); NO gate reads it. Expose a typed reader (e.g. `GovConfig::principal_kind(&PrincipalId) -> Option<&str>` or a `PrincipalKind { Agent, Human }` parse) surfaced to the tag-derivation + the desktop UI only.
NEWTYPES / ENUMS:
  - `pub enum DerivedPrStatus { Draft, AwaitingReview, ChangesRequested, Approved, Mergeable }` — derived at query time; `#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]`. PURELY presentational; NEVER consulted by a gate.
  - `pub struct DerivedPr { target_branch: String, source_branch: String, sha: String, author: Option<String>, title: Option<String>, draft: bool, created_at: ..., updated_at: ..., status: DerivedPrStatus, agent_authored: bool, labels: Vec<String>, open_assignments: Vec<...>, unresolved_thread_count: usize }` — mirrors the relevant ForgeReview fields (forge_reviews.rs:52) where a local analogue exists, plus the derived status + the agent tag.
OWNERSHIP PLAN:
  - The derivation BORROWS `&repo` for a READ-ONLY gix commit walk (commits ahead of base) and READS the three tables via the immutable Handles (`local_review_verdicts()`, `local_review_assignments()`, `local_review_comments()` — never the *_mut handles). Collect the walk + the rows into owned Vecs; fold into the DerivedPr by value. NO `&mut repo`, NO graph_rebase::Editor, NO ref/object/oplog write.
ERROR STRATEGY:
  - anyhow::Result; `.context(...)` for each read (the gix walk, the verdict query, the target-ref config load). The agent-tag derivation that can't find the `local_review_meta` opener row — OR whose opener has no committed entry / no declared `kind` — yields agent_authored=false (a human/unknown opener), NOT an error — absence is "not an agent" (the conservative default-human posture), never a panic.
DOC POINTERS (read before coding):
  - brain/docs/rust/ownership-borrowing.md → iterators + collect for the commit walk + the row folds; borrow &repo (read-only)
  - brain/docs/rust/traits-generics.md → enum match for the DerivedPrStatus derivation rules
  - brain/docs/rust/error-handling.md → Result + ? + anyhow::Context per read
  - brain/docs/rust/testing.md → real but-db + gix fixture via but_testsupport; ref/object/oplog snapshot before/after

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Proven against real but-db + real gix via but_testsupport (hand-assertion style, like merge_gate/governed_loop): (1) review_status returns a DERIVED status computed at query time from commits + verdict-at-head + open pending assignments (e.g. commits ahead + a pending assignment + no verdict@head ⇒ AwaitingReview) with NO local_pull_requests table / stored-lifecycle row anywhere; (2) the returned object mirrors the ForgeReview fields (target_branch, source_branch, sha, author, title, draft, timestamps) where a local analogue exists; (3) a review whose opener principal declares `kind = "agent"` in committed permissions.toml is auto-tagged agent-authored, sourced from the opener principal id in the dedicated `local_review_meta` opener row + that principal's declared `kind` read at the target ref — never a caller arg, never handle-inference; (4) a review whose opener declares `kind = "human"` (or omits `kind`) OMITS the tag; (5) review_status --agent-authored returns only the agent PR; (6) the commits-ahead-of-base gix walk is READ-ONLY (refs/objects/oplog byte-unchanged before/after) AND the derived Approved/Mergeable label NEVER authorizes a merge (enforce_merge_gate re-derives verdict-at-head itself and ignores the label); (7) the additive `kind: Option<String>` field on PrincipalWire deserializes cleanly on an OLDER permissions.toml without it (None → human), does NOT enter GovConfig.principals, and is read by NO gate (the invariant_build_gates honesty grep over the enforcement paths stays green); cargo test -p but-api / -p but-authz green; clippy clean; the forge.rs honesty grep stays green.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST derive the PR lifecycle at QUERY TIME over the three already-present sources (tech-delta §A.3) — there is NO local_pull_requests table and NO stored-lifecycle row. The derived status is a projection: commits ahead of base (gix walk) + verdict-at-head (local_review_verdicts) + open pending assignments + unresolved comment count ⇒ DerivedPrStatus. AC-1's negative control fails if any stored-lifecycle table/row exists.
- [MUST] MUST reuse the EXACT verdict-at-head query merge_gate runs: list local_review_verdicts.list_by_target(target) (the same review_verdicts call merge_gate.rs:40 makes) and filter to verdict.head_oid == current_head_oid AND verdict == "approved" (the review_requirement.rs:94 head_oid filter + :8 const APPROVED). Do NOT invent a divergent verdict query — the derivation's "Approved" must be computed from the SAME truth the gate enforces, so the orchestrator's read and the gate's read agree (UC-LPR-05).
- [MUST] MUST make the commits-ahead-of-base walk a READ-ONLY gix graph walk — `repo.rev_walk(...)` / ancestor iteration over &repo only. NO mutation: no graph_rebase::Editor, no ref write, no object write, no oplog entry. AC-6's ref/object/oplog-byte-unchanged snapshot is the structural proof. (This is the same read-only display posture WORKSPACE_MODEL.md mandates for Workspace/RefInfo.)
- [MUST] MUST keep Approved/Mergeable a PRESENTATION LABEL ONLY — the actual merge decision stays enforce_merge_gate (merge_gate.rs:40), which re-derives verdict-at-head ITSELF and never reads DerivedPr. A bug in the derivation can mislabel a PR but can NEVER authorize a merge. AC-6 proves the derived Mergeable label does not flip a merge the gate would deny.
- [MUST] MUST add the additive optional `kind` AUTHZ-config descriptor on `PrincipalWire` (crates/but-authz/src/config.rs:424): `#[serde(default)] pub kind: Option<String>`, mirroring the existing `role: Option<String>` (config.rs:429). Because PrincipalWire is `#[serde(deny_unknown_fields)]` (config.rs:423) the field MUST be declared on the wire struct; `#[serde(default)]` makes older committed permissions.toml files without `kind` deserialize cleanly (None → human, AC-7). The field is read at the TARGET REF like all governance config (config.rs:23–:25, anti-self-escalation). It does NOT enter `GovConfig.principals` (config.rs:85) and NO gate reads it — expose a typed reader (e.g. `principal_kind(&PrincipalId) -> Option<&str>` / a `PrincipalKind` parse) surfaced ONLY to the tag-derivation + the desktop UI.
- [MUST] MUST auto-derive the agent-authored tag from the OPENER principal's DECLARED `kind` in committed config — NOT handle-inference, NOT a comment marker. Read the opener principal id from the `local_review_meta(target, "opener_principal")` row (LPR-003's write-once meta opener), resolve THAT principal's committed `.gitbutler/permissions.toml` entry AT THE TARGET REF, and set `agent_authored = true` IFF that entry declares `kind = "agent"`. NEVER from `BUT_AGENT_HANDLE` resolution (it cannot tell agent from human — `Principal` has no kind discriminator, principal.rs:82); NEVER from a caller arg (there is NO --agent / --kind flag). A missing meta opener row, a missing committed entry, or an omitted `kind` ⇒ agent_authored=false (the conservative default-human posture), never an error. (R23: the cached meta opener row is forgeable by a direct DB write; R19: an actor can re-export BUT_AGENT_HANDLE to impersonate a different declared principal and borrow its kind — both stay named, neither is presented as closed.)
- [MUST] MUST mirror the relevant ForgeReview fields (target_branch, source_branch, sha, author, title, draft, timestamps) where a local analogue exists (forge_reviews.rs:52), so the derived object presents the same shape a forge PR would (T-LPR-004). Source sha from the current head of the source branch (the gix walk's tip); title/author/draft from the local review object's available data (the local_review_meta opener row for author; title may be derived/empty where no local analogue exists — name the gap, do not fabricate).
- [MUST] MUST support the --agent-authored filter on the status read (review_status / `but review status --agent-authored`) so an orchestrator can act on agent PRs without parsing author identity (T-LPR-021). The filter narrows to PRs whose agent_authored==true.
- [NEVER] NEVER create a local_pull_requests table or a stored-lifecycle column — the PR is DERIVED (tech-delta §A.3). AC-1 fails if a stored truth exists.
- [NEVER] NEVER let any gate read the agent-authored tag OR the new principal `kind` field (T-LPR-022). The tag is descriptive metadata and `kind` is a descriptor; role separation emerges from the functional permission set, not a label or a kind. Keep `agent_authored` AND `kind` OUT of every enforcement path (merge_gate.rs, review_requirement.rs, commit/gate.rs, authorize.rs, effective_authority, merge_gate) — the `kind` field MUST NOT enter `GovConfig.principals` (config.rs:85); LPR-010's grep proves the tag enters no enforcement path. R19: the tag is spoofable via BUT_AGENT_HANDLE re-export to impersonate a different declared principal and is NOT a trustworthy authorship attestation; R23: the cached meta opener row is forgeable by a direct DB write — name both, never present either as closed.
- [NEVER] NEVER mutate the git graph during the walk (no graph_rebase::Editor, no ref/object/oplog write) — AC-6's snapshot catches a mutation.
- [NEVER] NEVER add a new Authority variant or gate review_status on a write authority — it is a SELF-SCOPED READ (no write authority, mirrors governance_status_read), disclosing only the caller's-target surface (the self-scope invariant).
- [NEVER] NEVER branch on a role name / human-vs-AI predicate in any enforcement path (the invariant_build_gates honesty grep over forge.rs must stay green — forge.rs IS an ENFORCEMENT_PATH).
- [NEVER] NEVER hand-edit packages/but-sdk/src/generated — the regen is LPR-010's gate.
- [NEVER] NEVER add new gitbutler-* usage.
- [STRICTLY] STRICTLY treat merge_gate's verdict-at-head query (merge_gate.rs:40 → review_verdicts → review_requirement.rs:94/:8) as a CONSUMED reference — read it to learn the EXACT filter the derivation must reuse; do NOT modify merge_gate.rs/review_requirement.rs (the safe seam) or add any read of the new tables to the gate path.
- [STRICTLY] STRICTLY treat get_review (forge.rs:401) as the self-scoped-read shape template, the local_review_meta opener row (LPR-003) as the CONSUMED opener seam, and the target-ref config load (config.rs:23–:25) as the CONSUMED config-read seam (the new `kind` field is read at the target ref like all governance config).

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [x] AC-1 [PRIMARY]: review_status returns a DERIVED status computed at query time (commits + verdict-at-head + open assignments); NO local_pull_requests table / stored-lifecycle row exists
- [x] AC-2: the status object mirrors the ForgeReview fields (target_branch, source_branch, sha, author, title, draft, timestamps)
- [x] AC-3: a review whose opener's committed entry declares kind="agent" (read at the target ref) is auto-tagged agent-authored, sourced from the local_review_meta opener row + the declared kind (never a caller arg, never handle-inference)
- [x] AC-4: a review whose opener declares kind="human" (or omits kind) OMITS the agent-authored tag
- [x] AC-5: review_status --agent-authored returns only the agent PR (orchestrator filters without parsing author identity)
- [x] AC-6: the commits-ahead-of-base gix walk is READ-ONLY (refs/objects/oplog byte-unchanged) AND the derived Mergeable label NEVER flips a merge the gate would deny
- [x] AC-7: the additive `kind: Option<String>` on PrincipalWire deserializes cleanly on an OLDER permissions.toml without it (None → human), does NOT enter GovConfig.principals, and is read by NO gate (enforcement-neutral)
- [x] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: PR lifecycle is DERIVED at query time — no stored truth
  GIVEN: lpr_governed_repo: a branch with commits ahead of base; NO verdict@head in local_review_verdicts; ONE `pending` local_review_assignments row; BUT_AGENT_HANDLE=rev (caller can self-read)
  WHEN:  `but review status refs/heads/feat` runs (review_status)
  THEN:  the returned object's derived status is computed at query time (commits-ahead + pending-assignment + no-verdict@head ⇒ AwaitingReview); AND there is NO local_pull_requests table and NO stored-lifecycle row anywhere (the status is a projection over commits + verdicts + assignments, re-derived on each call)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api review_status deriving over real but-db (verdicts/assignments/comments) + a real gix commit walk via but_testsupport::writable_scenario
  VERIFY: cargo test -p but-api review_status_is_derived_not_stored

AC-2: the derived object mirrors the relevant ForgeReview fields
  GIVEN: lpr_governed_repo: a branch with a source/target/head/title and an opener author
  WHEN:  `but review status refs/heads/feat` runs
  THEN:  the status object carries target_branch, source_branch, sha (the source head), author, title, draft, and timestamps — the same shape a ForgeReview presents (forge_reviews.rs:52); a field with no local analogue is named/empty, never fabricated
  TEST_TIER: api-contract   VERIFICATION_SERVICE: real but-api review_status object shape vs the ForgeReview field set
  VERIFY: cargo test -p but-api review_status_mirrors_forge_review_fields

AC-3: a review whose opener declares kind="agent" in committed config is auto-tagged agent-authored
  GIVEN: lpr_governed_repo: a review opened by principal agent-A whose committed .gitbutler/permissions.toml entry declares `kind = "agent"` (the local_review_meta opener row records agent-A as opener; the kind is read at the TARGET REF) under #[serial_test::serial]
  WHEN:  `but review status refs/heads/feat` runs
  THEN:  the derived PR object carries the `agent-authored` label and agent_authored==true, sourced from the local_review_meta opener row + agent-A's DECLARED kind="agent" at the target ref — NOT from BUT_AGENT_HANDLE resolution and NOT from any caller arg (there is no --agent/--kind flag); the label mirrors the ForgeReview.labels precedent
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api review_status agent-tag derivation reading the local_review_meta opener row + resolving the opener's committed kind at the target ref
  VERIFY: cargo test -p but-api review_status_tags_agent_authored_from_declared_kind

AC-4: a review whose opener declares kind="human" (or omits kind) omits the agent-authored tag
  GIVEN: lpr_governed_repo: a review opened by principal human-H whose committed .gitbutler/permissions.toml entry declares `kind = "human"` (or OMITS kind — the default-human posture) — the local_review_meta opener row records human-H as opener. NOTE: human-H may itself resolve through BUT_AGENT_HANDLE (every governed principal does); the distinction is the DECLARED kind, not handle-resolution
  WHEN:  `but review status refs/heads/feat` runs
  THEN:  the derived PR object has agent_authored==false and NO `agent-authored` label, so human-declared PRs are distinguishable from agent-declared ones (an OMITTED kind also yields false — the conservative default)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api review_status with an opener whose committed kind is "human"/omitted
  VERIFY: cargo test -p but-api review_status_omits_tag_for_human_or_unkinded_opener

AC-5: review_status --agent-authored returns only the agent PR
  GIVEN: lpr_governed_repo: one review whose opener declares kind="agent" (branch A) and one whose opener declares kind="human"/omits kind (branch B)
  WHEN:  the agent-authored-filtered status read runs (review_status with the agent-authored filter / `but review status --agent-authored`)
  THEN:  only the agent-authored PR (branch A) is returned; the human PR (branch B) is excluded — the orchestrator acts on agent PRs without parsing author identity
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api review_status filtered to agent_authored
  VERIFY: cargo test -p but-api review_status_filters_by_agent_authored

AC-6: read-only gix walk + the derived label never gates
  GIVEN: lpr_governed_repo: a branch with commits ahead of base AND (separately) a branch the derivation labels Mergeable BUT with NO approved verdict@head (a derivation that would mislabel); refs/objects/oplog snapshotted
  WHEN:  `but review status` runs (the gix commits-ahead walk) AND a governed merge is attempted on the Mergeable-labeled-but-unverified branch
  THEN:  refs/objects/oplog are byte-identical before and after review_status (the walk is read-only); AND the governed merge is BLOCKED (enforce_merge_gate re-derives verdict-at-head and ignores the derived Mergeable label) — the presentation label does not authorize a merge the gate would deny
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api review_status (read-only gix walk, ref/object/oplog snapshot) + real enforce_merge_gate ignoring the derived label
  VERIFY: cargo test -p but-api review_status_gix_walk_is_read_only_and_label_does_not_gate

AC-7: the additive `kind` AUTHZ-config field is enforcement-neutral + backward-compatible
  GIVEN: an OLDER committed .gitbutler/permissions.toml WITHOUT any `kind` field, and a newer one declaring `kind = "agent"` on a principal; loaded at the target ref
  WHEN:  but-authz load_governance_config parses each, and the enforcement map (GovConfig.principals) + the gates are evaluated
  THEN:  the old file deserializes cleanly (the principal's kind is None → human; NO deny_unknown_fields parse error); the new file's `kind` is readable via the typed reader; AND the `kind` value does NOT enter GovConfig.principals (config.rs:85) and changes NO gate decision (authorize/effective_authority/merge_gate are byte-identical with kind="agent" vs kind="human" vs omitted) — the descriptor is read ONLY by the tag-derivation + the UI
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-authz load_governance_config over an old + a new permissions.toml at the target ref; enforcement decisions compared across kind values
  VERIFY: cargo test -p but-authz principal_kind_is_additive_and_enforcement_neutral

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): review_status returns a derived status (commits-ahead + pending-assignment + no-verdict@head ⇒ AwaitingReview) computed at query time
    VERIFY: cargo test -p but-api review_status_is_derived_not_stored
- TC-2 (-> AC-1): no local_pull_requests table / stored-lifecycle row exists (the PR is a projection, re-derived per call)
    VERIFY: cargo test -p but-api review_status_is_derived_not_stored
- TC-3 (-> AC-2): the status object carries target_branch/source_branch/sha/author/title/draft/timestamps (the ForgeReview field set)
    VERIFY: cargo test -p but-api review_status_mirrors_forge_review_fields
- TC-4 (-> AC-3): a review whose opener declares kind="agent" (read at the target ref) carries agent_authored==true + the agent-authored label, sourced from the local_review_meta opener row + the declared kind (not handle-inference, not a caller arg)
    VERIFY: cargo test -p but-api review_status_tags_agent_authored_from_declared_kind
- TC-5 (-> AC-4): a review whose opener declares kind="human" or OMITS kind carries agent_authored==false / no agent-authored label
    VERIFY: cargo test -p but-api review_status_omits_tag_for_human_or_unkinded_opener
- TC-6 (-> AC-5): the agent-authored-filtered read returns only the agent PR; the human PR is excluded
    VERIFY: cargo test -p but-api review_status_filters_by_agent_authored
- TC-7 (-> AC-6): review_status leaves refs/objects/oplog byte-unchanged (read-only gix walk)
    VERIFY: cargo test -p but-api review_status_gix_walk_is_read_only_and_label_does_not_gate
- TC-8 (-> AC-6): a branch the derivation labels Mergeable but with no approved verdict@head is STILL blocked by enforce_merge_gate (the label never gates)
    VERIFY: cargo test -p but-api review_status_gix_walk_is_read_only_and_label_does_not_gate
- TC-9 (-> AC-7): an old permissions.toml without kind deserializes cleanly (None->human); the kind value does NOT enter GovConfig.principals and changes no gate decision (enforcement-neutral)
    VERIFY: cargo test -p but-authz principal_kind_is_additive_and_enforcement_neutral

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides:
  - review_status(ctx, branch) — a self-scoped read returning a DERIVED DerivedPr (lifecycle status + ForgeReview-shaped fields + open assignments + unresolved-thread count + the agent-authored tag), computed at query time over commits (read-only gix walk) + verdict-at-head + assignments + comments
  - the agent-PR tag derived from the opener principal's declared `kind` in committed permissions.toml (opener id from the local_review_meta opener row), never a caller arg, never handle-inference, never read by a gate; PLUS the additive optional `kind` field on PrincipalWire (enforcement-neutral)
  - `but review status <branch> [--agent-authored]` CLI verb routed through review_gate_cli_error
consumes:
  - local_review_verdicts.list_by_target + the verdict-at-head filter (the EXACT query merge_gate runs, merge_gate.rs:40 → review_requirement.rs:94/:8) — REUSED as the derivation's "Approved" truth (read-only; the gate's loader semantics are untouched)
  - but_db::{LocalReviewAssignment, LocalReviewComment, LocalReviewMeta} Handles (LPR-001) + the local_review_meta opener row (LPR-003) + the committed-config PrincipalWire `kind` field
  - the ForgeReview field set (forge_reviews.rs:52) as the shape to mirror; get_review (forge.rs:401) as the self-scoped-read shape; the confined-principal identity split (forge.rs:526)
  - a READ-ONLY gix commit walk for commits-ahead-of-base (no mutation)
boundary_contracts:
  - CAP-AUTHZ-01: review_status is a self-scoped read (no write authority, like governance_status_read), disclosing only the caller's-target surface; no new Authority variant. The PR lifecycle is DERIVED (no stored truth). The Approved/Mergeable label is presentational — enforce_merge_gate re-derives verdict-at-head and never reads DerivedPr, so a derivation bug can mislabel a PR but can never authorize a merge. The agent-authored tag is derived from the opener's declared `kind` in committed config (opener id from the local_review_meta row), never a caller arg, never handle-inference, never read by a gate; the additive `kind` field changes no enforcement (not in GovConfig.principals, no gate reads it). (R19: spoofable via BUT_AGENT_HANDLE re-export, not a trustworthy attestation.)

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-authz/src/config.rs (MODIFY — add the additive optional `kind: Option<String>` field on PrincipalWire, config.rs:424, the role: Option<String> precedent; #[serde(default)]; it does NOT enter GovConfig.principals and no gate reads it)
  - crates/but-api/src/legacy/forge.rs (MODIFY — add review_status + the private derive_pr_state helper + the DerivedPr/DerivedPrStatus types beside get_review; reuse the verdict-at-head query + the ForgeReview field shape)
  - crates/but/src/command/legacy/forge/review.rs (MODIFY — add the status CLI verb; route via review_gate_cli_error)
  - crates/but/src/args/ (MODIFY — the `but review status <branch> [--agent-authored]` verb/arg definitions; NOT but-clap)
  - crates/but-api/tests/review_status.rs (NEW — the PRIMARY but-api proofs AC-1..AC-6)
  - packages/but-sdk/src/generated/** (REGENERATE ONLY — NEVER hand-edit; the regen + tag-not-an-enforcement-key grep is LPR-010)
writeProhibited:
  - crates/but-api/src/legacy/merge_gate.rs, review_requirement.rs — CONSUME-only (the safe seam); READ the verdict-at-head query to reuse it, but do NOT change its loader semantics and do NOT add any read of local_review_assignments/local_review_comments to the gate path (LPR-009 greps this)
  - crates/but_graph/**, crates/but-rebase/** and any graph_rebase::Editor — the commits-ahead walk is READ-ONLY; NO graph mutation
  - crates/but-db/** — CONSUME the LPR-001 tables/Handles; do NOT add a local_pull_requests table or a stored-lifecycle column (the PR is DERIVED)
  - crates/but-authz/src/authority.rs — no new Authority variant; the agent tag is never an authority
  - crates/but-api/src/legacy/forge.rs approve_review/merge_review/publish_review/request_review — CONSUME (read shapes); do NOT change the shipped/LPR-003 verbs
  - any gitbutler-* crate (no new gitbutler-* usage)
  - Any file not in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-api/src/legacy/merge_gate.rs [40-95, 155-170] — [PRIMARY PATTERN — reuse the verdict-at-head query] enforce_merge_gate → review_verdicts(ctx, &review.source_branch) → local_review_verdicts().list_by_target(target). Your derivation reuses THIS list call and filter for the "Approved" truth, so the orchestrator's read agrees with the gate's. Do NOT modify this file; do NOT add a read of the new tables here.
2. crates/but-api/src/legacy/review_requirement.rs [8, 79-100] — const APPROVED="approved" (:8) + the verdict.head_oid == current_head_oid filter (:94). The EXACT verdict-at-head predicate the derivation must apply (commits' current head + verdict=="approved").
3. crates/but-db/src/table/forge_reviews.rs [44-55] — the ForgeReview struct + its `labels` field (forge_reviews.rs:52) — the field set DerivedPr mirrors (target_branch, source_branch, sha, author, title, draft, timestamps) AND the labels precedent for the agent-authored tag.
4. crates/but-api/src/legacy/forge.rs [401-...] — get_review: the self-scoped-read shape (a sync `fn(ctx: &Context, …)` read, no write authority). Mirror this posture for review_status. ALSO forge.rs:526 — approve_review derives principal_id from authorize_branch_action (the confined-principal identity split the agent-tag derivation reuses); forge.rs:520 — the into_thread_local() frame if review_status needs the head async.
5. crates/but-db/src/table/local_review_assignments.rs + local_review_comments.rs (LPR-001) — list_by_target / list_by_thread (read the pending assignments + unresolved comment count); the local_review_meta opener row (LPR-003's request_review writes (target, "opener_principal", opener_id) write-once) is where the agent-tag opener id comes from.
6. crates/but-authz/src/config.rs [424-431, 85, 23-25] — PrincipalWire (config.rs:424, #[serde(deny_unknown_fields)]) + the existing `role: Option<String>` optional-field precedent (config.rs:427) the additive `kind: Option<String>` field mirrors. The field does NOT enter GovConfig.principals (config.rs:85, the enforcement map) and NO gate reads it; it is read at the target ref like all governance config (config.rs:23-25, anti-self-escalation).
7. crates/WORKSPACE_MODEL.md — the lossy-presentation discipline (Workspace/RefInfo are read-only display views, never mutated). The DerivedPr is the same: a read-only projection, NEVER a stored truth, NEVER a graph mutation.
8. crates/but-api/tests/ (the merge_gate / governed_loop hand-assertion tests) — [VERIFIED TEST IDIOM] real-but-db + gix + #[serial_test::serial] + temp_env BUT_AGENT_HANDLE; the ref/object/oplog snapshot helper for AC-6. Mirror it for review_status.rs (NOT insta snapshots).

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- cargo test -p but-api review_status_is_derived_not_stored   -> Exit 0; derived AwaitingReview at query time; no stored-lifecycle table/row
- cargo test -p but-api review_status_mirrors_forge_review_fields   -> Exit 0; target/source/sha/author/title/draft/timestamps present
- cargo test -p but-api review_status_tags_agent_authored_from_declared_kind   -> Exit 0; agent_authored==true from the opener's declared kind="agent" in committed config (opener id from the local_review_meta row; not a caller arg, not handle-inference)
- cargo test -p but-api review_status_omits_tag_for_human_or_unkinded_opener   -> Exit 0; an opener declaring kind="human" or omitting kind ⇒ agent_authored==false / no label
- cargo test -p but-api review_status_filters_by_agent_authored   -> Exit 0; --agent-authored returns only the agent PR
- cargo test -p but-api review_status_gix_walk_is_read_only_and_label_does_not_gate   -> Exit 0; refs/objects/oplog byte-unchanged; Mergeable-labeled-but-unverified branch still blocked by the gate
- cargo check -p but-api --all-targets   -> Exit 0
- cargo clippy -p but-api --all-targets   -> Exit 0
- cargo test -p but-authz invariant_build_gates   -> Exit 0; forge.rs honesty grep green (the agent tag enters no enforcement path)
- cargo fmt --check   -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references:
  - crates/but-api/src/legacy/merge_gate.rs:40 (the verdict-at-head query to reuse), review_requirement.rs:94/:8 (the head_oid + "approved" filter)
  - crates/but-db/src/table/forge_reviews.rs:52 (the ForgeReview field set + labels precedent)
  - crates/but-api/src/legacy/forge.rs:401 (get_review self-scoped read), :526 (the authorize-derived principal pattern)
  - crates/but-authz/src/config.rs:424 (the additive optional PrincipalWire `kind` field — the role: Option<String> precedent at :427; does NOT enter GovConfig.principals at :85)
  - .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md §A.3 (PR lifecycle DERIVED, no local_pull_requests table) + §A.4 (agent-PR tag from the opener's declared kind in committed config, opener id cached in local_review_meta) + §E (the safe seam — the derived label never gates)
  - .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/04-e2e-testing-criteria.md (T-LPR-003/004/019/020/021/022/023/024)
derivation_rules: |
  // pr_state(target):
  //   commits     = read-only gix walk of source-branch commits ahead of base (NO mutation)
  //   approved@head = local_review_verdicts.list_by_target(target)
  //                     .filter(|v| v.head_oid == current_head_oid && v.verdict == "approved")  (the merge_gate query, REUSED)
  //   pending      = local_review_assignments.list_by_target(target).filter(state == "pending")
  //   unresolved   = local_review_comments open-thread count (resolved == false)
  //   ⇒ status:
  //       no commits ahead                         -> Draft
  //       commits ahead, no verdict@head, pending  -> AwaitingReview
  //       any changes_requested assignment         -> ChangesRequested
  //       approved@head, unresolved == 0           -> Approved / Mergeable   (PRESENTATION ONLY — gate re-derives)
  //   agent_authored = (opener principal id from the local_review_meta opener row) declares kind="agent" in committed permissions.toml at the target ref
notes:
  - The status enum is a label for display/orchestration; the merge decision is enforce_merge_gate (merge_gate.rs:40) which re-derives verdict-at-head ITSELF — review_status's "Mergeable" is never consulted by the gate (AC-6 proves a mislabeled branch is still blocked).
  - The agent tag derivation: read the opener principal id from the local_review_meta(target, "opener_principal") row, resolve that principal's committed-config entry at the target ref, set agent_authored=true iff it declares kind="agent". Absence of the opener row ⇒ agent_authored=false, never an error. R19: this is spoofable via BUT_AGENT_HANDLE re-export to impersonate a DIFFERENT declared principal (borrowing its kind) — name it; do NOT present the tag as a trustworthy attestation. R23: the cached local_review_meta opener row is forgeable by a direct DB write — name it.
  - The walk MUST be read-only (repo.rev_walk / ancestors over &repo). AC-6 snapshots refs/objects/oplog before/after.
pattern: a self-scoped read that DERIVES the PR object at query time over commits (read-only gix walk) + the merge_gate verdict-at-head query (reused) + open assignments + unresolved comments, mirroring the ForgeReview field set, with an agent-authored tag derived from the opener's declared `kind` in committed config (opener id from the local_review_meta row) — never stored, never gating, never a caller arg
pattern_source: crates/but-api/src/legacy/merge_gate.rs:40 + review_requirement.rs:94/:8 (the verdict-at-head query); crates/but-db/src/table/forge_reviews.rs:52 (the ForgeReview fields + labels); crates/but-api/src/legacy/forge.rs:401 (get_review self-scoped read) + :526 (the authorize-derived principal pattern); crates/but-authz/src/config.rs:424 (the additive PrincipalWire kind field); crates/WORKSPACE_MODEL.md (lossy-presentation discipline)
anti_pattern: a local_pull_requests table / stored-lifecycle row (AC-1 fails — the PR must be derived); a divergent verdict query that disagrees with the gate; a graph mutation during the walk (AC-6 catches a ref/object/oplog change); the derived Mergeable label authorizing a merge the gate would deny (AC-6 inverse); the agent tag from a caller --agent flag, from BUT_AGENT_HANDLE resolution, or from a __pr_meta__ comment-body marker, rather than the opener's declared kind in committed config (opener id from the local_review_meta row); the agent tag read by any enforcement path (LPR-010's grep / T-LPR-022); presenting the tag as a trustworthy authorship attestation (R19 — it is spoofable)

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agent: implementer=rust-implementer | reviewer=rust-reviewer
rationale: A derived, read-only PR-projection query that must (a) reuse the merge_gate verdict-at-head query verbatim so the orchestrator's and the gate's reads agree, (b) keep the gix commits-ahead walk strictly read-only (no graph mutation), (c) keep the derived label purely presentational (a mislabel can never authorize a merge), and (d) auto-derive the agent tag from the confined opener, never a caller arg, never gating. This is the trickiest fakeability surface after LPR-003 — a stub returning a fixed "Mergeable" would pass a weak test, so AC-6 pins both the read-only walk and the label-never-gates inverse. rust-implementer writes the derivation; rust-reviewer validates no stored truth, the reused verdict query, the read-only walk, and that the tag enters no enforcement path.
coding_standards: crates/AGENTS.md (keep types in the crate that owns the concept; solve the present problem directly — no speculative stored state; prefer gix read APIs over shelling out; preserve Git graph semantics — read-only walk only); crates/but-api/src/legacy/ (the #[but_api] + self-scoped-read idiom); crates/WORKSPACE_MODEL.md (lossy presentation, read-only derivation); RULES.md (gix over git2 for new logic; after but-sdk-exposed API changes run pnpm build:sdk — the regen is LPR-010); brain/docs/rust/ (ownership-borrowing.md iterators/collect; traits-generics.md enum match; error-handling.md ? + Context)

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: LPR-001 (the three tables + Handles, incl. local_review_meta), LPR-002 (AssignmentState — read the typed assignment state), LPR-003 (request_review's local_review_meta opener row + the assignments), LPR-004 (local_review_comments for the unresolved-thread count)
Blocks:     LPR-008 (the reconciler read-API extends review_status to serve the full drive state in one payload), LPR-010 (SDK regen for review_status + the tag-not-an-enforcement-key grep)
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "LPR-005",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "lpr_governed_repo": {
      "description": "A real governed repo via but_testsupport::writable_scenario + invoke_bash committing .gitbutler/permissions.toml to the target ref (the committed config declares principal kinds: agent-A with kind=\"agent\", human-H with kind=\"human\"/omitted). A real but_ctx::Context + DbHandle (the LPR-001 tables migrated). Branches: refs/heads/feat with commits ahead of base; a review on branch A whose local_review_meta opener row records agent-A (opener declares kind=\"agent\" in committed config); a review on branch B whose opener row records human-H (opener declares kind=\"human\" or omits kind). Verdicts/assignments/comments seeded ONLY via the real verbs (request_review/assign_reviewer/post_comment/approve_review), never direct row injection. BUT_AGENT_HANDLE set per-case under #[serial_test::serial] via temp_env (it only resolves WHICH principal acted — the tag derives from the opener's DECLARED kind, not the handle). The merge_gate/governed_loop hand-assertion idiom (real but-db + real gix, no mocks, no insta). For AC-6, a ref/object/oplog snapshot is captured before/after review_status, and a separate branch is set up so the derivation would label it Mergeable while NO approved verdict@head exists.",
      "seed_method": "public_api",
      "records": [
        "but_testsupport::writable_scenario(...) + invoke_bash committing .gitbutler/permissions.toml (declaring agent-A kind=\"agent\", human-H kind=\"human\"/omitted) + a feature branch with commits ahead of base;",
        "request_review(branch A, reviewer) records the opener id (agent-A) in local_review_meta write-once + a pending assignment; agent-A declares kind=\"agent\" in committed permissions.toml;",
        "the opener principal for branch B (human-H) declares kind=\"human\" (or omits kind) in committed permissions.toml; its local_review_meta opener row records human-H;",
        "approve_review / leave-no-verdict as needed to exercise the derived-status rules; capture refs/objects/oplog before/after review_status for AC-6."
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN lpr_governed_repo: a branch with commits ahead of base, NO verdict@head, ONE pending assignment WHEN `but review status refs/heads/feat` runs THEN the derived status is computed at query time (commits-ahead + pending + no-verdict@head ⇒ AwaitingReview) AND there is NO local_pull_requests table / stored-lifecycle row (the PR is a projection re-derived per call)",
      "verify": "cargo test -p but-api review_status_is_derived_not_stored",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api review_status deriving over real but-db + a real gix commit walk via but_testsupport",
        "negative_control": {
          "would_fail_if": [
            "a local_pull_requests table / stored-lifecycle row existed — the PR would be a 4th stored truth that could drift from the gate",
            "the status were a fixed constant regardless of commits/verdicts/assignments — a stub not actually deriving",
            "the derivation ignored the pending assignment / verdict@head — the wrong status (e.g. Mergeable instead of AwaitingReview)"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_governed_repo",
            "action": { "actor": "agent", "steps": [ "BUT_AGENT_HANDLE=rev", "run review_status(refs/heads/feat)", "assert the derived status == AwaitingReview", "assert no local_pull_requests table exists and no stored-lifecycle row is read" ] },
            "end_state": {
              "must_observe": [
                "the derived status == AwaitingReview (computed from commits-ahead + pending + no-verdict@head)",
                "no local_pull_requests table / stored-lifecycle row anywhere",
                "the status re-derives on a second call (a projection, not a cached/stored value)"
              ],
              "must_not_observe": [
                "a stored-lifecycle table/row",
                "a fixed status independent of the seeded commits/verdicts/assignments"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN lpr_governed_repo: a branch with a source/target/head/title/author WHEN `but review status refs/heads/feat` runs THEN the object carries target_branch, source_branch, sha (source head), author, title, draft, timestamps — the ForgeReview field set; a field with no local analogue is named/empty, never fabricated",
      "verify": "cargo test -p but-api review_status_mirrors_forge_review_fields",
      "scenario": {
        "tier": "visible",
        "test_tier": "api-contract",
        "verification_service": "real but-api review_status object shape vs the ForgeReview field set (forge_reviews.rs:52)",
        "negative_control": {
          "would_fail_if": [
            "the object omitted a field with a local analogue (e.g. no sha / no source_branch) — not ForgeReview-shaped",
            "a field with no local analogue were fabricated with a fake value rather than named/empty"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_governed_repo",
            "action": { "actor": "agent", "steps": [ "run review_status(refs/heads/feat)", "assert target_branch/source_branch/sha/author/title/draft/timestamps are present" ] },
            "end_state": {
              "must_observe": [ "target_branch, source_branch, sha, author, title, draft, created_at/updated_at all present (the ForgeReview shape)" ],
              "must_not_observe": [ "a missing field that has a local analogue", "a fabricated value for a field with no local analogue (it must be empty/None, not invented)" ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN lpr_governed_repo: a review whose opener principal declares `kind = \"agent\"` in committed permissions.toml (the opener id recorded in the local_review_meta opener row by LPR-003's request_review) WHEN `but review status refs/heads/feat` runs THEN the derived PR carries agent_authored==true + the agent-authored label, sourced from the local_review_meta opener row + the opener's declared `kind = \"agent\"` read at the target ref (not a caller arg; no --agent flag; never handle-inference)",
      "verify": "cargo test -p but-api review_status_tags_agent_authored_from_declared_kind",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api review_status agent-tag derivation reading the local_review_meta opener row + resolving the opener's committed kind at the target ref",
        "negative_control": {
          "would_fail_if": [
            "the tag were derived from a caller-supplied --agent flag rather than the opener's declared kind in committed config — an agent could claim/disclaim authorship",
            "an opener declaring kind=human were wrongly tagged agent-authored (the kind check is broken)",
            "the tag were missing despite an opener declaring kind=agent (the local_review_meta opener row / the declared kind was not read)",
            "the tag were derived from BUT_AGENT_HANDLE resolution rather than the declared kind — handle-resolution cannot tell agent from human"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_governed_repo",
            "action": { "actor": "agent", "steps": [ "(review already opened via request_review; the local_review_meta opener row = agent-A; agent-A declares kind=agent in committed permissions.toml)", "run review_status(branch A)", "assert agent_authored==true and the labels include agent-authored" ] },
            "end_state": {
              "must_observe": [ "agent_authored == true", "the labels include `agent-authored`", "the tag is sourced from the local_review_meta opener row (agent-A) + agent-A's declared kind=agent at the target ref, not a caller arg" ],
              "must_not_observe": [ "the tag derived from a caller --agent flag", "the tag derived from BUT_AGENT_HANDLE resolution", "agent_authored==false for an opener declaring kind=agent" ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN lpr_governed_repo: a review whose opener principal declares `kind = \"human\"` (or omits `kind`) in committed permissions.toml (the opener id recorded in the local_review_meta opener row) WHEN `but review status refs/heads/feat` runs THEN agent_authored==false and NO agent-authored label (human PRs are distinguishable; an omitted kind also yields false — the conservative default)",
      "verify": "cargo test -p but-api review_status_omits_tag_for_human_or_unkinded_opener",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api review_status with an opener whose committed kind is \"human\"/omitted",
        "negative_control": {
          "would_fail_if": [
            "an opener declaring kind=human were wrongly tagged agent-authored (the kind check defaults to agent / is inverted)",
            "the tag were applied unconditionally regardless of the opener's declared kind"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_governed_repo",
            "action": { "actor": "human", "steps": [ "(the opener principal for branch B (human-H) declares kind=human or omits kind in committed permissions.toml; its local_review_meta opener row records human-H)", "run review_status(branch B)", "assert agent_authored==false / no agent-authored label" ] },
            "end_state": {
              "must_observe": [ "agent_authored == false", "no agent-authored label" ],
              "must_not_observe": [ "agent_authored==true for an opener declaring kind=human/omitted", "an agent-authored label on the human PR" ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN lpr_governed_repo: one agent-opened review (branch A) and one human-opened review (branch B) WHEN the agent-authored-filtered status read runs THEN only the agent PR (branch A) is returned; the human PR (branch B) is excluded",
      "verify": "cargo test -p but-api review_status_filters_by_agent_authored",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api review_status filtered to agent_authored",
        "negative_control": {
          "would_fail_if": [
            "the filter were a no-op (both A and B returned) — the orchestrator could not isolate agent PRs",
            "the filter excluded the agent PR (inverted) — branch A missing"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_governed_repo",
            "action": { "actor": "agent", "steps": [ "run review_status with the agent-authored filter (or `but review status --agent-authored`)", "assert only branch A is returned" ] },
            "end_state": {
              "must_observe": [ "branch A (agent-authored) is returned", "branch B (human) is excluded" ],
              "must_not_observe": [ "branch B in the filtered result (no-op filter)", "branch A excluded (inverted filter)" ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-6",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN lpr_governed_repo: a branch with commits ahead AND a branch the derivation would label Mergeable but with NO approved verdict@head; refs/objects/oplog snapshotted WHEN review_status runs (the gix walk) AND a governed merge is attempted on the Mergeable-labeled-but-unverified branch THEN refs/objects/oplog are byte-identical before/after review_status (read-only walk) AND the governed merge is BLOCKED (enforce_merge_gate re-derives verdict-at-head and ignores the derived label)",
      "verify": "cargo test -p but-api review_status_gix_walk_is_read_only_and_label_does_not_gate",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api review_status (read-only gix walk + ref/object/oplog snapshot) + real enforce_merge_gate ignoring the derived label",
        "negative_control": {
          "would_fail_if": [
            "the walk mutated a ref/object/oplog — the before/after snapshots differ (a graph mutation during a display read)",
            "the derived Mergeable label flipped a merge the gate would deny — the gate read the derived view instead of re-deriving verdict-at-head (a safe-seam violation)"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_governed_repo",
            "action": { "actor": "agent", "steps": [ "snapshot refs/objects/oplog", "run review_status (the commits-ahead gix walk)", "re-snapshot refs/objects/oplog", "attempt a governed merge on the branch the derivation labels Mergeable but with no approved verdict@head" ] },
            "end_state": {
              "must_observe": [ "refs/objects/oplog byte-identical before and after review_status", "the governed merge is BLOCKED (gate.review_required) despite the derived Mergeable label" ],
              "must_not_observe": [ "any ref/object/oplog mutation from the walk", "the merge proceeding because of the derived Mergeable label" ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-7",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN an OLD committed permissions.toml WITHOUT any kind field, and a newer one declaring kind=agent on a principal, loaded at the target ref WHEN but-authz load_governance_config parses each and the enforcement map + gates are evaluated THEN the old file deserializes cleanly (kind is None->human, no deny_unknown_fields error); the new kind is readable via the typed reader; AND the kind value does NOT enter GovConfig.principals (config.rs:85) and changes NO gate decision (authorize/effective_authority/merge_gate identical across kind=agent vs human vs omitted)",
      "verify": "cargo test -p but-authz principal_kind_is_additive_and_enforcement_neutral",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-authz load_governance_config over an old + a new permissions.toml at the target ref; enforcement decisions compared across kind values",
        "negative_control": {
          "would_fail_if": [
            "the old permissions.toml without kind failed to deserialize (a deny_unknown_fields error / a missing-field error) instead of defaulting None->human",
            "the kind value entered GovConfig.principals or changed a gate decision (authorize/effective_authority/merge_gate differed across kind values) -- the descriptor leaked into enforcement",
            "the field were declared somewhere other than PrincipalWire (a free-floating key), failing to parse under deny_unknown_fields"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_governed_repo",
            "action": { "actor": "ci", "steps": [ "load an OLD permissions.toml without kind; assert it deserializes and the principal kind is None->human", "load a new permissions.toml declaring kind=agent; read it via the typed reader", "evaluate authorize/effective_authority/merge_gate across kind=agent vs human vs omitted and assert the decisions are identical", "assert kind is not present in GovConfig.principals" ] },
            "end_state": {
              "must_observe": [ "the old (no-kind) file deserializes cleanly (None->human)", "the new kind is readable via the typed reader", "no gate decision changes across kind values; kind is absent from GovConfig.principals" ],
              "must_not_observe": [ "a deserialize error on the old no-kind file", "kind entering GovConfig.principals or changing any gate decision" ]
            }
          }
        ]
      }
    },
    { "id": "TC-1", "type": "test_criterion", "description": "review_status returns a derived AwaitingReview at query time (commits-ahead + pending + no-verdict@head)", "verify": "cargo test -p but-api review_status_is_derived_not_stored", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "no local_pull_requests table / stored-lifecycle row exists (the PR is a projection)", "verify": "cargo test -p but-api review_status_is_derived_not_stored", "maps_to_ac": "AC-1" },
    { "id": "TC-3", "type": "test_criterion", "description": "the status object carries the ForgeReview field set (target/source/sha/author/title/draft/timestamps)", "verify": "cargo test -p but-api review_status_mirrors_forge_review_fields", "maps_to_ac": "AC-2" },
    { "id": "TC-4", "type": "test_criterion", "description": "a review whose opener declares kind=agent in committed config carries agent_authored==true + the label, sourced from the local_review_meta opener row + the declared kind (not a caller arg, not handle-inference)", "verify": "cargo test -p but-api review_status_tags_agent_authored_from_declared_kind", "maps_to_ac": "AC-3" },
    { "id": "TC-5", "type": "test_criterion", "description": "a review whose opener declares kind=human or omits kind carries agent_authored==false / no agent-authored label", "verify": "cargo test -p but-api review_status_omits_tag_for_human_or_unkinded_opener", "maps_to_ac": "AC-4" },
    { "id": "TC-6", "type": "test_criterion", "description": "the agent-authored-filtered read returns only the agent PR; the human PR is excluded", "verify": "cargo test -p but-api review_status_filters_by_agent_authored", "maps_to_ac": "AC-5" },
    { "id": "TC-7", "type": "test_criterion", "description": "review_status leaves refs/objects/oplog byte-unchanged (read-only gix walk)", "verify": "cargo test -p but-api review_status_gix_walk_is_read_only_and_label_does_not_gate", "maps_to_ac": "AC-6" },
    { "id": "TC-8", "type": "test_criterion", "description": "a Mergeable-labeled-but-unverified branch is still blocked by enforce_merge_gate (the label never gates)", "verify": "cargo test -p but-api review_status_gix_walk_is_read_only_and_label_does_not_gate", "maps_to_ac": "AC-6" },
    { "id": "TC-9", "type": "test_criterion", "description": "an old permissions.toml without kind deserializes None->human; the kind value does not enter GovConfig.principals and changes no gate decision", "verify": "cargo test -p but-authz principal_kind_is_additive_and_enforcement_neutral", "maps_to_ac": "AC-7" }
  ]
}
-->
