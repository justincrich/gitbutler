# LPR-015: local-review READ producers — `review_status`/`list_comments` Tauri commands + SDK binding exposing the branch-scoped drive state (derived lifecycle + assignments + threads) to the desktop

## What this does

Expose the LPR **read verbs** — `review_status` (the derived PR lifecycle + the full reconciler drive state: open `pending` assignments + unresolved comment threads + verdict-at-head, LPR-005/LPR-008) and `list_comments` (the comment/thread list for a target, LPR-004) — to the SvelteKit desktop as **READ-ONLY** Tauri commands + a regenerated `@gitbutler/but-sdk` binding, so the LocalReviewView (LPR-016) can render the local-PR drive state. Both verbs already live in `crates/but-api/src/legacy/forge.rs` as `#[but_api(napi)]` fns (LPR-004/005/008); this task's deliverable is the **producer surface** — registering their `#[but_api(napi)]`-macro-generated `tauri_review_status::review_status` + `tauri_list_comments::list_comments` command modules on the desktop bus (mirroring the already-registered `legacy::forge::tauri_get_review::get_review` / `tauri_list_reviews::list_reviews`), proving they register + invoke on the REAL Tauri mock-runtime bus, and regenerating the SDK so the generated TS type-checks in the desktop. These are **pure reads — NO mutate, NO write authority** (they share `get_review`'s branch-scoped read posture). The reads are **BRANCH-scoped, NOT self-scoped** (per the remediated F-006 design: a branch's review surface is shared drive-state — every principal's assignments/threads on the named branch are returned to any caller who can name the branch). **No parallel ungated path (R14):** the desktop reaches these reads ONLY through the `but-api` gated `#[but_api(napi)]` wrapper — the same audited seam the CLI and N-API use — never a hand-rolled IPC handler that bypasses it. The producer NEVER touches the merge gate (the safe seam is intact — the gate reads only `local_review_verdicts`).

## Why

Sprint 07 · PRD UC-LPR-01, UC-LPR-02, UC-LPR-05 · capability CAP-AUTHZ-01. LPR-005/008 make `review_status` the orchestrator's reconciler read and LPR-004 makes `list_comments` the thread reader — but those serve the CLI and the N-API (Electron lite) only. The SvelteKit desktop's LocalReviewView (LPR-016) needs the SAME drive state through the Tauri bus + the generated SDK to render the derived lifecycle, the reviewer assignments, the agent-authored tag, and the comment threads. This producer is that surface, mirroring the shipped forge reads (`get_review`/`list_reviews`) that already ride `#[but_api(napi)]`-generated `tauri_*` modules. Keeping the reads **branch-scoped** (not self-scoped) is the deliberate F-006 disclosure: the reconciler thesis needs every orchestrator AND the human desktop to read the SAME branch state, so a branch's review surface is visible to all callers on the project (the same posture as `LocalReviewVerdictsHandle::list_by_target`). Routing the desktop reads ONLY through the gated `but-api` wrapper (R14) means the desktop inherits the audited authorization seam — no consequential N-API/IPC route bypasses it. The producer is read-only and never mutates; it cannot regress the land-truth (the merge gate reads only `local_review_verdicts`, untouched).

## How to verify

PRIMARY **AC-1** — `cargo test -p gitbutler-tauri lpr_review_reads_register_and_return_branch_drive_state`: on the REAL Tauri mock-runtime bus (`tauri::test::get_ipc_response`, the mgmt_ipc_003 idiom), invoking `review_status` for a branch fixtured (via the real verbs) with a `pending` assignment + an unresolved thread + a verdict-at-head returns the SINGLE derived-lifecycle + drive-state payload, and `list_comments` returns the branch's threads — both as READ-ONLY commands registered on the bus. Full gate set in the spec below.

## Scope

- crates/gitbutler-tauri/src/lib.rs (MODIFY — register `legacy::forge::tauri_review_status::review_status` + `legacy::forge::tauri_list_comments::list_comments` in the forge command block beside the shipped `tauri_get_review::get_review` / `tauri_list_reviews::list_reviews` (lib.rs:339-340); these ride the but-api `#[but_api(napi)]`-generated modules — NO new gitbutler-tauri wrapper, NO DesktopSessionState (reads need no fleet-owner))
- crates/gitbutler-tauri/tests/lpr_review_reads.rs (NEW — the PRIMARY proofs AC-1..AC-4 on the REAL Tauri mock-runtime bus, mirroring the mgmt_ipc_003 governance_app/governance_webview/get_ipc_response idiom + a real but-db + gix fixture seeded via the real LPR verbs; asserts both commands register, invoke, and return the branch-scoped drive state; an unregistered probe is rejected)
- crates/but-api/src/legacy/forge.rs (READ-ONLY reference — review_status/list_comments are already #[but_api(napi)] fns from LPR-004/005/008; this task adds NO new but-api fn — it only registers their generated tauri modules on the desktop bus. If review_status/list_comments lack #[but_api(napi)] / the napi+tauri emission, FLAG it as a LPR-004/005/008 gap; do NOT add a second reader here.)
- packages/but-sdk/src/generated/\*\* (REGENERATE ONLY via `pnpm build:sdk && pnpm format` — NEVER hand-edit)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: LPR-015 — local-review READ producers: review_status/list_comments Tauri commands + SDK binding (branch-scoped drive state to the desktop)
================================================================================

TASK_TYPE:   FEATURE
STATUS:      Backlog
PRIORITY:    P1
EFFORT:      M  (120 min)
AGENT:       implementer=tauri-implementer | reviewer=tauri-reviewer
PROPOSED-BY: tauri-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-LPR-01, UC-LPR-02, UC-LPR-05
CAPABILITIES:CAP-AUTHZ-01

PLATFORMS:   desktop   (the SvelteKit desktop GUI consumes the Tauri command + SDK; the but-api read verbs also serve the CLI/N-API, but the producer surface this task ships is the DESKTOP bus + the regenerated TS SDK. NO mobile target — GitButler desktop is Tauri-on-desktop only.)

RUNTIME_COMMANDS:
  test:  cargo test -p gitbutler-tauri lpr_review_reads_register_and_return_branch_drive_state
  check: cargo check -p gitbutler-tauri --all-targets
  lint:  cargo clippy -p gitbutler-tauri --all-targets

--------------------------------------------------------------------------------
TAURI IPC CONTRACT (the producer surface this task ships)
--------------------------------------------------------------------------------
COMMAND (read):  `review_status` (but-api #[but_api(napi)] from LPR-005/008 → auto-generated legacy::forge::tauri_review_status::review_status; invoke key `review_status`)
  Signature:     `fn review_status(ctx, branch: String[, agent_authored_only: bool]) -> Result<ReviewStatus>` (the LPR-005/008 payload: derived DerivedPrStatus + ForgeReview-shaped fields + agent_authored tag + open_assignments + unresolved_threads + verdict_at_head). A BRANCH-SCOPED READ — NO write authority (shares get_review's read posture forge.rs:401), discloses the whole branch's drive state (F-006).
  Frontend:      invoke<ReviewStatus>('review_status', { projectId, branch }): Promise<ReviewStatus>
COMMAND (read):  `list_comments` (but-api #[but_api(napi)] from LPR-004 → auto-generated legacy::forge::tauri_list_comments::list_comments; invoke key `list_comments`)
  Signature:     `fn list_comments(ctx, branch: String) -> Result<Vec<LocalReviewComment>>` (or the LPR-004 thread-grouped payload). A BRANCH-SCOPED READ — NO write authority; returns ALL comments/threads for the named branch (F-006).
  Frontend:      invoke<LocalReviewComment[]>('list_comments', { projectId, branch }): Promise<LocalReviewComment[]>
PERMISSION (capability + permission delta — the atomic command+permission rule):
  - BOTH commands are admitted by `core:default` in capabilities/main.json — GitButler app commands ride core:default; there is NO hand-written allow-review_status / allow-list_comments / allow-forge_* capability file (the IPC test mgmt_ipc_003 ASSERTS no per-command allow-governance_* file exists for the governance surface; the forge reads follow the SAME core:default admission as the shipped tauri_get_review::get_review — NO allow-file). The Tauri-v2 per-command `allow-review_status`/`allow-list_comments` permission is AUTOGENERATED by the #[but_api(napi)]/#[tauri::command] macro into gen/schemas/, never authored by hand.
  - The "permission entry" that ships WITH each command (so neither slips to a later task) is: registration in lib.rs's forge command block (beside legacy::forge::tauri_get_review::get_review, lib.rs:340) AND an invocation case in crates/gitbutler-tauri/tests/lpr_review_reads.rs proving each registers + invokes on the real bus. An unregistered command is rejected by the real bus ("not found") — registration IS the admission.
CAPABILITY: capabilities/main.json is UNCHANGED (no new entry; core:default admits both reads, exactly as get_review/list_reviews). The capability assertion is the NEGATIVE one: no per-command allow-review_status/allow-list_comments file is introduced.

--------------------------------------------------------------------------------
TYPE DESIGN (Rust shapes this task introduces)
--------------------------------------------------------------------------------
API SURFACE:
  - NO new but-api fn. review_status (LPR-005 derived lifecycle + LPR-008 full drive state) and list_comments (LPR-004) are ALREADY #[but_api(napi)] reads in crates/but-api/src/legacy/forge.rs. This task REGISTERS their macro-generated tauri_<name>::<name> modules on the desktop bus and regenerates the SDK. The deliverable is the producer surface (the desktop command registration + the real-bus proof + the SDK type-check), NOT a new reader.
  - If review_status/list_comments are #[but_api] WITHOUT (napi) (i.e. they emit the tauri module but not the N-API binding, or vice-versa), confirm the emission covers BOTH the tauri command (for the desktop) AND the napi binding (for Electron lite, R14) — both are #[but_api(napi)] per the LPR-004/005/008 specs. FLAG a mismatch to LPR-004/005/008; do NOT patch the attribute here (that is the source task's surface).
RETURN TYPES (consumed, not defined here — LPR-004/005/008 own them; this task only ensures they are JsonSchema-serializable for the SDK):
  - ReviewStatus / DerivedPr { status: DerivedPrStatus, target_branch, source_branch, sha, author, title, draft, timestamps, agent_authored, labels, open_assignments: Vec<LocalReviewAssignment>, unresolved_threads: Vec<ThreadSummary>, verdict_at_head: Option<...> } (LPR-005 + LPR-008)
  - Vec<LocalReviewComment> or the thread-grouped list (LPR-004)
  - These must derive Serialize (+ JsonSchema where the SDK generator requires it, like the governance DTOs governance.rs:192) so the regenerated TS is well-typed. If a return type is not SDK-serializable, FLAG it to the owning LPR task.
ERROR STRATEGY:
  - anyhow::Result at the but-api boundary (the shipped forge read convention, get_review forge.rs:401). The IPC layer surfaces errors as the structured json::Error the frontend handles. A branch with NO drive state returns an Ok payload with empty vecs + verdict_at_head=None (a clean empty-state, never an Err — LPR-008's contract). Document the empty-state shape the SvelteKit LocalReviewView renders.
OWNERSHIP PLAN:
  - This task writes NO Rust read logic — the reads are owned by LPR-004/005/008. It modifies lib.rs's command registration (adding two paths to the forge generate_handler block) and adds a Tauri-mock-runtime test. NO &mut, NO write path, NO DesktopSessionState (reads need no fleet-owner identity — they are not governed-config writes).
DOC POINTERS (read before coding):
  - brain/.rosetta/docs/tauri/commands.md → the #[tauri::command] registration model; how generate_handler admits commands
  - brain/.rosetta/docs/tauri/permissions.md → Tauri v2 core:default admission; autogenerated allow-<command>; no hand-written allow-file
  - brain/.rosetta/docs/tauri/testing.md → tauri::test::get_ipc_response real-bus invocation; mock runtime
  - crates/AGENTS.md → but-api is THE API boundary; the desktop reaches reads through the gated but-api wrapper (R14), never a parallel IPC handler

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Proven against the REAL Tauri mock-runtime bus (tauri::test::get_ipc_response, mgmt_ipc_003 idiom) + a real but-db + gix fixture seeded via the real LPR verbs: (1) review_status registers on the desktop bus and, for a branch fixtured with a `pending` assignment + an unresolved thread + a verdict-at-head, returns the SINGLE payload carrying the derived lifecycle + open_assignments + unresolved_threads + verdict_at_head (the LPR-005/008 reconciler read, now on the desktop); (2) list_comments registers and returns the branch's comment threads (LPR-004); (3) BOTH are READ-ONLY — invoking them performs NO write (the assignment/comment/verdict store is byte-unchanged before/after; refs/objects/oplog unchanged) and they carry NO write authority; (4) the reads are BRANCH-scoped (F-006): review_status/list_comments return EVERY principal's assignments/threads on the named branch to any caller who can name the branch (not per-principal self-scoping) — the shared-drive-state disclosure the reconciler thesis needs; (5) there is NO parallel ungated path (R14): the desktop reaches these reads ONLY through the registered but-api #[but_api(napi)] wrapper (the same audited seam the CLI/N-API use) — no hand-rolled gitbutler-tauri IPC handler bypasses the but-api boundary; (6) an unregistered review-read probe is rejected by the real bus ("not found") — registration is the admission; (7) `pnpm build:sdk && pnpm format` regenerates the SDK with review_status/list_comments + their return types, and the generated TS type-checks in the desktop; cargo test -p gitbutler-tauri green; clippy clean.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST keep BOTH producers READ-ONLY — NO mutate. review_status and list_comments are pure reads (they share get_review's read posture, forge.rs:401): NO write authority, NO local-cache write, NO ref/object/oplog mutation. AC-3's no-write proof (the assignment/comment/verdict store byte-unchanged before/after the read; refs/objects/oplog unchanged) is the behavioral guard. Do NOT add a write side here — the writers are LPR-003/004 (request/assign/comment/resolve), a SEPARATE surface.
- [MUST] MUST register the reads via the but-api #[but_api(napi)]-generated tauri modules, mirroring the shipped forge reads. Add `legacy::forge::tauri_review_status::review_status` + `legacy::forge::tauri_list_comments::list_comments` to lib.rs's forge generate_handler block BESIDE `legacy::forge::tauri_get_review::get_review` (lib.rs:340) / `tauri_list_reviews::list_reviews` (lib.rs:339). Do NOT write a new gitbutler-tauri wrapper module and do NOT use DesktopSessionState — reads need no fleet-owner identity (that is the governed-config WRITE pattern, branch_gates_update/principal_kind_update). The read rides the same path as get_review.
- [MUST] MUST keep the reads BRANCH-scoped, NOT self-scoped (the remediated F-006 design). review_status/list_comments return ALL assignments/threads/meta for the NAMED branch to any caller who can name it — the shared-drive-state disclosure the reconciler thesis needs (the same posture as LocalReviewVerdictsHandle::list_by_target, local_review_verdicts.rs:64). Do NOT narrow them to per-principal self-scoping (that is governance_status_read's posture, a DIFFERENT contract). AC-4 proves branch-scoping: a second principal's assignment/thread on the branch IS returned. The honest-disclosure note (F-006) must NOT be contradicted — do NOT claim these reads keep cross-principal disclosure gated.
- [MUST] MUST route the desktop reads ONLY through the gated but-api #[but_api(napi)] wrapper (R14 — any consequential N-API/IPC route goes through the but-api gated seam). The desktop command IS the but-api fn's generated tauri module; there is NO hand-rolled gitbutler-tauri IPC handler that reads local_review_assignments/comments/meta directly and bypasses the but-api boundary. AC-5 proves no parallel ungated path: the ONLY desktop entry to the review reads is the registered but-api command. (R14 is SATISFIED because the reads ARE but-api fns — do not introduce a bypass.)
- [MUST] MUST prove registration + invocation on the REAL Tauri mock-runtime bus (NOT a source-only registration grep). Use tauri::test::get_ipc_response via the mgmt_ipc_003 governance_app/governance_webview idiom (or the forge-equivalent app builder) so the test exercises the real command bus: both commands invoke and return the branch drive state; an unregistered probe is rejected ("not found"). Seed the drive state via the REAL LPR verbs (request_review/assign_reviewer/post_comment/approve_review), never direct row injection.
- [MUST] MUST surface the SDK delta as part of done. After registration, `pnpm build:sdk && pnpm format` regenerates packages/but-sdk/src/generated with review_status/list_comments + their return types; the generated TS type-checks in the desktop; the generated files are NEVER hand-edited. (If LPR-004/005/008 already regenerated the SDK for the CLI/N-API, this task CONFIRMS the desktop-consumed bindings type-check in apps/desktop and re-runs the regen if the registration changed the surface.)
- [MUST] MUST register each command WITH its real-bus proof in the SAME task (the atomic command+permission rule): no command ships without its lib.rs registration AND its lpr_review_reads.rs invocation case (registration IS admission); no registration without the invocation proof.
- [NEVER] NEVER add a write authority, a mutate, or a local-cache/ref/object/oplog write to either read (AC-3 catches a write). The reads disclose; they never change drive state.
- [NEVER] NEVER narrow the reads to per-principal self-scoping or claim cross-principal disclosure is gated (F-006: the reads ARE branch-scoped; a branch's review surface is shared). AC-4's second-principal-visible proof fails if self-scoped.
- [NEVER] NEVER introduce a parallel ungated desktop read path (a hand-rolled gitbutler-tauri IPC handler reading the drive tables directly, bypassing the but-api gated wrapper) — R14. The desktop reads ONLY through the registered but-api #[but_api(napi)] command.
- [NEVER] NEVER touch the merge gate or add a read of the three LPR drive tables to merge_gate.rs — these are read producers over the drive surface; the safe seam (the gate reads only local_review_verdicts) is untouched (LPR-009's invariant).
- [NEVER] NEVER author a per-command allow-review_status/allow-list_comments/allow-forge_* capability FILE — the forge reads ride core:default like get_review; the per-command permission is the macro-autogenerated allow-<command>, never hand-written.
- [NEVER] NEVER add a new but-api reader fn for the reviews — review_status/list_comments are owned by LPR-004/005/008; this task registers + exposes them, it does not re-implement them. A second reader would risk a divergent (and potentially ungated) path.
- [NEVER] NEVER hand-edit packages/but-sdk/src/generated — regenerate via pnpm build:sdk only.
- [NEVER] NEVER add new gitbutler-* usage beyond gitbutler-tauri (the desktop shell this producer extends).
- [STRICTLY] STRICTLY treat review_status (LPR-005/008) + list_comments (LPR-004) + get_review (forge.rs:401, the read-shape precedent) + the lib.rs forge command block (lib.rs:330-345) + the mgmt_ipc_003 real-bus harness as CONSUMED seams — register the generated modules, mirror the read posture, reuse the bus harness; do not fork a parallel reader or a parallel registration path.
- [STRICTLY] STRICTLY keep the (ctx, branch) signatures the LPR verbs define so the Tauri command and the N-API binding pass the same branch the workspace resolves.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: review_status registers on the desktop bus and returns the SINGLE derived-lifecycle + drive-state payload (assignments + unresolved threads + verdict-at-head) for a fixtured branch
- [ ] AC-2: list_comments registers on the desktop bus and returns the branch's comment threads
- [ ] AC-3: both reads are READ-ONLY — invoking them performs NO write (assignment/comment/verdict store + refs/objects/oplog byte-unchanged) and carry NO write authority
- [ ] AC-4: the reads are BRANCH-scoped (F-006) — review_status/list_comments return a SECOND principal's assignments/threads on the named branch (not per-principal self-scoping)
- [ ] AC-5: no parallel ungated path (R14) — the ONLY desktop entry to the review reads is the registered but-api #[but_api(napi)] command (no hand-rolled IPC handler bypasses the but-api boundary); `pnpm build:sdk && pnpm format` regenerates the SDK and it type-checks in the desktop
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: review_status registers + returns the branch drive state on the desktop bus
  GIVEN: lpr_review_read_repo: a real governed repo + real gix + a TestDesktopSession + the real Tauri mock runtime; a branch refs/heads/feat fixtured via the real verbs with a `pending` assignment for rev2, an unresolved thread t1, and an approved verdict@head; caller `rev` holds reviews:write + pull_requests:write + comments:write; BUT_AGENT_HANDLE=rev under #[serial_test::serial]
  WHEN:  review_status is invoked on the bus (tauri::test::get_ipc_response) for refs/heads/feat
  THEN:  the command returns Ok with the SINGLE payload carrying the derived lifecycle status (e.g. AwaitingReview/Approved), open_assignments containing the `pending` rev2 assignment, unresolved_threads containing t1, and verdict_at_head/approved reflecting the approval@head — the LPR-005/008 reconciler read, now reachable from the desktop
  TEST_TIER: integration   VERIFICATION_SERVICE: real Tauri mock-runtime bus invoking the registered but-api review_status #[but_api(napi)] command over a real but-db + gix fixture seeded via the real LPR verbs
  VERIFY: cargo test -p gitbutler-tauri lpr_review_reads_register_and_return_branch_drive_state

AC-2: list_comments registers + returns the branch's threads on the desktop bus
  GIVEN: lpr_review_read_repo: a branch refs/heads/feat with two comment threads (t1 unresolved, t2 resolved) posted via the real post_comment verb; caller `rev` holds comments:write
  WHEN:  list_comments is invoked on the bus for refs/heads/feat
  THEN:  the command returns Ok with the branch's comment list/threads (t1 and t2 both present, with their resolved flags) — the LPR-004 thread reader, now reachable from the desktop
  TEST_TIER: integration   VERIFICATION_SERVICE: real Tauri mock-runtime bus invoking the registered but-api list_comments #[but_api(napi)] command over a real but-db comment store
  VERIFY: cargo test -p gitbutler-tauri list_comments_returns_branch_threads_on_bus

AC-3: both reads are READ-ONLY (no write, no write authority)
  GIVEN: lpr_review_read_repo: a branch with a `pending` assignment, an unresolved thread, and an approved verdict@head; the assignment/comment/verdict store + refs/objects/oplog snapshotted before the reads
  WHEN:  review_status AND list_comments are each invoked on the bus
  THEN:  after both reads, the local_review_assignments/local_review_comments/local_review_verdicts stores are byte-unchanged AND refs/objects/oplog are byte-unchanged (the reads mutate nothing); neither command requires or checks a write authority (a caller with read-but-not-write capability still gets the payload — the reads are not write-gated)
  TEST_TIER: integration   VERIFICATION_SERVICE: real Tauri mock-runtime bus + real but-db store snapshot before/after + real gix ref/object/oplog snapshot
  VERIFY: cargo test -p gitbutler-tauri review_reads_are_read_only_no_mutation

AC-4: the reads are BRANCH-scoped (F-006), not self-scoped
  GIVEN: lpr_review_read_repo: a branch refs/heads/feat with assignments/threads from TWO distinct principals (rev2's assignment + rev3's assignment; a thread authored by rev2 and a thread authored by rev3); the invoking caller is a THIRD principal `viewer` who authored none of them but can name the branch
  WHEN:  review_status AND list_comments are invoked on the bus by `viewer`
  THEN:  review_status's open_assignments includes BOTH rev2's and rev3's assignments, and list_comments returns BOTH rev2's and rev3's threads — the whole branch's review surface is disclosed to `viewer` (branch-scoped shared drive state, F-006), NOT narrowed to the caller's own assignments/threads (the reads are explicitly NOT per-principal self-scoped)
  TEST_TIER: integration   VERIFICATION_SERVICE: real Tauri mock-runtime bus invoking the reads as a third principal over a multi-principal drive fixture
  VERIFY: cargo test -p gitbutler-tauri review_reads_are_branch_scoped_not_self_scoped

AC-5: no parallel ungated path (R14) + the SDK regenerates + type-checks
  GIVEN: the gitbutler-tauri crate + the registered review_status/list_comments commands + the generated SDK
  WHEN:  the desktop command registration is inspected AND `pnpm build:sdk && pnpm format` runs AND apps/desktop type-checks
  THEN:  the ONLY desktop entry to review_status/list_comments is the registered but-api #[but_api(napi)]-generated tauri module (lib.rs forge block) — there is NO hand-rolled gitbutler-tauri IPC handler that reads local_review_assignments/local_review_comments/local_review_meta directly and bypasses the but-api boundary (a grep over gitbutler-tauri finds no direct drive-table read for these commands); AND packages/but-sdk/src/generated contains review_status/list_comments + their return types and the generated TS type-checks in apps/desktop (no hand-edit)
  TEST_TIER: integration   VERIFICATION_SERVICE: source-structure check (the desktop read entry is the but-api command, no bypass handler) + real `pnpm build:sdk && pnpm format` SDK regen + pnpm -F @gitbutler/desktop check
  VERIFY: cargo test -p gitbutler-tauri lpr_review_reads_register_and_return_branch_drive_state && pnpm build:sdk && pnpm format && grep -rq "review_status\|reviewStatus" packages/but-sdk/src/generated && grep -rq "list_comments\|listComments" packages/but-sdk/src/generated

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): review_status invokes on the real bus and returns the SINGLE payload with the derived lifecycle + the pending rev2 assignment + unresolved thread t1 + the verdict-at-head
    VERIFY: cargo test -p gitbutler-tauri lpr_review_reads_register_and_return_branch_drive_state
- TC-2 (-> AC-1/AC-5): review_status AND list_comments are registered in lib.rs's forge command block; an unregistered review-read probe is rejected by the real bus ("not found")
    VERIFY: cargo test -p gitbutler-tauri lpr_review_reads_register_and_return_branch_drive_state
- TC-3 (-> AC-2): list_comments invokes on the real bus and returns the branch's threads (t1 unresolved + t2 resolved, with resolved flags)
    VERIFY: cargo test -p gitbutler-tauri list_comments_returns_branch_threads_on_bus
- TC-4 (-> AC-3): after review_status AND list_comments, the local_review_assignments/comments/verdicts stores are byte-unchanged AND refs/objects/oplog are byte-unchanged (read-only)
    VERIFY: cargo test -p gitbutler-tauri review_reads_are_read_only_no_mutation
- TC-5 (-> AC-3): neither read requires a write authority — a read-capable-but-not-write caller still gets the payload (the reads are not write-gated)
    VERIFY: cargo test -p gitbutler-tauri review_reads_are_read_only_no_mutation
- TC-6 (-> AC-4): review_status's open_assignments includes BOTH rev2's and rev3's assignments when invoked by a third principal `viewer` (branch-scoped, not self-scoped)
    VERIFY: cargo test -p gitbutler-tauri review_reads_are_branch_scoped_not_self_scoped
- TC-7 (-> AC-4): list_comments returns BOTH rev2's and rev3's threads to `viewer` (branch-scoped disclosure, F-006)
    VERIFY: cargo test -p gitbutler-tauri review_reads_are_branch_scoped_not_self_scoped
- TC-8 (-> AC-5): the ONLY desktop entry to the review reads is the registered but-api #[but_api(napi)] command — no hand-rolled gitbutler-tauri IPC handler reads the drive tables directly (no R14 bypass)
    VERIFY: cargo test -p gitbutler-tauri lpr_review_reads_register_and_return_branch_drive_state
- TC-9 (-> AC-5): `pnpm build:sdk && pnpm format` regenerates packages/but-sdk/src/generated containing review_status/list_comments + their return types, and the generated TS type-checks in apps/desktop (no hand-edit)
    VERIFY: pnpm build:sdk && pnpm format && pnpm -F @gitbutler/desktop check && grep -rq "review_status\|reviewStatus" packages/but-sdk/src/generated && grep -rq "list_comments\|listComments" packages/but-sdk/src/generated
- TC-10 (-> AC-5): no allow-review_status/allow-list_comments capability file is introduced (the forge reads ride core:default like get_review)
    VERIFY: cargo test -p gitbutler-tauri lpr_review_reads_register_and_return_branch_drive_state

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides:
  - review_status (LPR-005 derived lifecycle + LPR-008 full drive state) exposed to the desktop as a READ-ONLY #[but_api(napi)]-generated tauri command, registered on the desktop bus, returning the branch-scoped reconciler payload
  - list_comments (LPR-004) exposed to the desktop as a READ-ONLY #[but_api(napi)]-generated tauri command, returning the branch's comment threads
  - the regenerated packages/but-sdk bindings for both reads, type-checking in apps/desktop
consumes:
  - crate::legacy::forge::{review_status, list_comments} (LPR-004/005/008 — the #[but_api(napi)] read fns this task registers on the desktop bus; NOT re-implemented), get_review (forge.rs:401 — the read-shape precedent)
  - crates/gitbutler-tauri/src/lib.rs forge command block (lib.rs:330-345, beside tauri_get_review::get_review/tauri_list_reviews::list_reviews — the registration site)
  - the mgmt_ipc_003 real-Tauri-mock-runtime harness (governance_app/governance_webview/get_ipc_response — the bus-proof idiom) + but_testsupport::writable_scenario + the real LPR verbs for seeding
  - but_db::{LocalReviewAssignment, LocalReviewComment, LocalReviewVerdict} as the read payload shapes (LPR-001) — read-only
boundary_contracts:
  - CAP-AUTHZ-01: review_status/list_comments are READ-ONLY, BRANCH-scoped reads (no write authority, no mutate; the whole branch's review surface is disclosed to any caller who can name it — F-006, NOT per-principal self-scoping). The desktop reaches them ONLY through the registered but-api #[but_api(napi)] gated wrapper (R14 — no parallel ungated IPC handler). They never write drive state, never touch the merge gate, and add no read of the three drive tables to the gate path (the safe seam, LPR-009, intact). No new Authority variant.
  - Safe-seam note: these reads serve the drive surface for display; enforce_merge_gate re-derives verdict-at-head ITSELF and never consults these reads (LPR-005/008). A read payload (even a mislabeled lifecycle) can never authorize a merge.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/gitbutler-tauri/src/lib.rs (MODIFY — register legacy::forge::tauri_review_status::review_status + legacy::forge::tauri_list_comments::list_comments in the forge generate_handler block beside tauri_get_review::get_review)
  - crates/gitbutler-tauri/tests/lpr_review_reads.rs (NEW — the PRIMARY real-Tauri-mock-runtime proofs AC-1..AC-5: both commands register + invoke + return the branch drive state; read-only; branch-scoped; no R14 bypass)
  - packages/but-sdk/src/generated/** (REGENERATE ONLY via `pnpm build:sdk && pnpm format` — NEVER hand-edit)
writeProhibited:
  - crates/but-api/src/legacy/forge.rs — CONSUME-only (review_status/list_comments are owned by LPR-004/005/008); register their generated tauri modules, do NOT add a new reader or change the existing ones. (If they lack #[but_api(napi)] / the tauri+napi emission, FLAG it to the owning LPR task — do not patch the attribute here.)
  - crates/but-api/src/legacy/merge_gate.rs, review_requirement.rs — CONSUME-only (the safe seam); do NOT add any read of the new tables to the gate path (LPR-009 greps this)
  - crates/but-db/** — CONSUME the LPR-001 tables/Handles (read-only payload shapes); do NOT change them
  - crates/but-authz/src/authority.rs — no new Authority variant; the reads carry no write authority
  - capabilities/main.json + any allow-review_status/allow-list_comments/allow-forge_* file — do NOT add a per-command allow file (core:default admits, like get_review)
  - crates/gitbutler-tauri/src/governance.rs — do NOT add a DesktopSessionState wrapper for these reads (reads need no fleet-owner identity — that is the governed-config WRITE pattern)
  - apps/desktop/** — the LocalReviewView is LPR-016 (a SvelteKit task); this task ships the producer + SDK only, NOT the view
  - the LPR write verbs (request_review/assign_reviewer/post_comment/resolve_thread, LPR-003/004) — CONSUME-only for seeding; this is a READ producer, do NOT touch the writers
  - any gitbutler-* crate beyond gitbutler-tauri (no new gitbutler-* usage)
  - Any file not in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. .spec/.../sprint-06b-governance-ui-branch-gates-rules-safety/MGMT-BE-004-branch-gates-config-writer.md — [PRIMARY PATTERN to mirror] the canonical but-api producer + its Tauri command/SDK delta. This task is the READ-ONLY analog: the verbs already exist (LPR-004/005/008), so the delta is the desktop-bus registration + the real-bus proof + the SDK type-check (mirror MGMT-BE-004's Tauri/SDK delta shape, minus the write/fleet-owner side).
2. crates/gitbutler-tauri/src/lib.rs [330-345] — [PRIMARY registration site] the forge command block: legacy::forge::tauri_get_review::get_review (:340), tauri_list_reviews::list_reviews (:339), tauri_get_review_merge_status (:341). Register tauri_review_status::review_status + tauri_list_comments::list_comments here — these ride the but-api #[but_api(napi)]-generated modules exactly like get_review (NO gitbutler-tauri wrapper, NO DesktopSessionState).
3. crates/but-api/src/legacy/forge.rs [401, + the LPR review_status/list_comments fns] — get_review (forge.rs:401) the branch-scoped read-shape precedent (a #[but_api(napi)] read, no write authority); review_status (LPR-005/008) + list_comments (LPR-004) the reads this task registers. Confirm they are #[but_api(napi)] (the tauri+napi emission); FLAG to the owning LPR task if not.
4. crates/gitbutler-tauri/tests/mgmt_ipc_003_governance_commands.rs [1-90, 230-300] — [VERIFIED real-bus idiom] governance_app/governance_webview + tauri::test::get_ipc_response (invoke_ok/invoke_err) + the unregistered-command "not found" proof + the forbidden-allow-file assertion. Mirror this harness for lpr_review_reads.rs (seed the drive state via the real LPR verbs, then invoke the reads on the bus).
5. .spec/.../sprint-07-local-agent-pr/LPR-005-derived-pr-lifecycle-agent-tag.md + LPR-008-reconciler-read-api.md + LPR-004-comment-thread-verbs.md — the read verbs' payload shapes (DerivedPr/ReviewStatus + ReviewDriveState; the comment/thread list) this task exposes; the branch-scoped (F-006) disclosure posture; the read-only contract.
6. .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md §B (the read verbs list_comments/review_status are BRANCH-scoped, NOT self-scoped — the F-006 honest-disclosure note) + §H (R14 — any consequential N-API/IPC route goes through the but-api gated wrapper; these reads ARE but-api fns, satisfying it) + §E (the safe seam — the reads never gate).
7. brain/.rosetta/docs/tauri/{commands,permissions,testing}.md — the #[tauri::command] registration/admission model (core:default, autogenerated allow-<command>, no hand-written allow-file) + the tauri::test::get_ipc_response real-bus testing idiom.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- cargo test -p gitbutler-tauri lpr_review_reads_register_and_return_branch_drive_state   -> Exit 0; review_status registers + returns the single derived-lifecycle + drive-state payload; an unregistered probe is rejected; no allow-review_status file
- cargo test -p gitbutler-tauri list_comments_returns_branch_threads_on_bus   -> Exit 0; list_comments registers + returns the branch's threads (t1 unresolved + t2 resolved)
- cargo test -p gitbutler-tauri review_reads_are_read_only_no_mutation   -> Exit 0; assignment/comment/verdict store + refs/objects/oplog byte-unchanged after both reads; no write authority required
- cargo test -p gitbutler-tauri review_reads_are_branch_scoped_not_self_scoped   -> Exit 0; a third principal `viewer` sees BOTH rev2's and rev3's assignments/threads (branch-scoped, F-006)
- cargo check -p gitbutler-tauri --all-targets   -> Exit 0
- cargo clippy -p gitbutler-tauri --all-targets   -> Exit 0
- cargo test -p but-authz invariant_build_gates   -> Exit 0; no read of the new tables added to the gate path; forge.rs honesty grep green
- cargo fmt --check   -> Exit 0
- pnpm build:sdk && pnpm format   -> Exit 0; packages/but-sdk/src/generated contains review_status/list_comments + their return types; generated TS type-checks; no hand-edit
- pnpm -F @gitbutler/desktop check   -> Exit 0; the regenerated SDK type-checks in the desktop frontend
- ! grep -rn "local_review_assignments()\|local_review_comments()\|local_review_meta()" crates/gitbutler-tauri/src   -> Exit 0; gitbutler-tauri has NO direct drive-table read (the reads go through the but-api gated wrapper — no R14 bypass handler)

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references:
  - .spec/.../MGMT-BE-004 (the producer + Tauri/SDK delta to mirror — the READ-ONLY analog, no write/fleet-owner side)
  - crates/gitbutler-tauri/src/lib.rs:339-340 (tauri_list_reviews::list_reviews / tauri_get_review::get_review — the forge read registration to mirror)
  - crates/but-api/src/legacy/forge.rs:401 (get_review — the branch-scoped #[but_api(napi)] read-shape precedent)
  - crates/gitbutler-tauri/tests/mgmt_ipc_003_governance_commands.rs (the real-Tauri-mock-runtime bus idiom)
  - .spec/.../sprint-07-local-agent-pr/{LPR-004,LPR-005,LPR-008}.md (the read verbs + their payloads + the branch-scoped F-006 disclosure)
  - .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md §B/§E/§H (branch-scoped reads, the safe seam, R14)
notes:
  - This is a REGISTRATION + PROOF + SDK task, not a new-reader task. The two reads exist (LPR-004/005/008); the deliverable is exposing them on the desktop bus, proving they register + invoke + return the branch drive state on the REAL bus, keeping them read-only + branch-scoped + R14-clean, and regenerating the SDK so apps/desktop type-checks.
  - Registration shape: in lib.rs's forge generate_handler block, add `legacy::forge::tauri_review_status::review_status,` and `legacy::forge::tauri_list_comments::list_comments,` beside the shipped get_review/list_reviews. These ride the but-api #[but_api(napi)]-generated modules — NO new gitbutler-tauri wrapper module, NO DesktopSessionState (reads have no fleet-owner). Contrast LPR-013's WRITE (principal_kind_update) which DOES need a gitbutler-tauri DesktopSessionState wrapper.
  - Read-only proof (AC-3): snapshot the local_review_assignments/comments/verdicts stores + refs/objects/oplog before the reads, invoke both on the bus, re-snapshot — byte-identical. The reads carry no write authority (a read-but-not-write caller still gets the payload).
  - Branch-scoped proof (AC-4, F-006): fixture two distinct principals' assignments/threads on the branch, invoke as a THIRD principal `viewer`, assert BOTH are returned — the branch's whole review surface is disclosed (the reconciler-thesis shared-drive-state, NOT per-principal self-scoping). Do NOT contradict the F-006 honest-disclosure note.
  - R14 proof (AC-5): the ONLY desktop entry to the review reads is the registered but-api #[but_api(napi)] command; a grep over crates/gitbutler-tauri/src finds no direct local_review_assignments()/local_review_comments()/local_review_meta() read (no bypass handler). R14 is SATISFIED because the reads ARE but-api fns — the constraint is to not introduce a bypass.
  - SDK: `pnpm build:sdk && pnpm format` regenerates the bindings; confirm review_status/list_comments + their return types are present and apps/desktop type-checks (pnpm -F @gitbutler/desktop check). If LPR-004/005/008 already regenerated for the CLI/N-API, re-run to capture the desktop-registration surface and confirm the type-check.
  - Capability/permission delta (the atomic rule): NO capabilities/main.json change (core:default admits, like get_review); the per-command allow-review_status/allow-list_comments permission is macro-autogenerated; the SHIPPED-TOGETHER permission entry is the lib.rs registration + the real-bus invocation proof (registration IS admission). No hand-written allow-file.
pattern: a READ-ONLY producer pair — register the already-existing #[but_api(napi)] review reads (review_status from LPR-005/008, list_comments from LPR-004) on the desktop bus (mirroring the shipped tauri_get_review::get_review forge registration), prove they register + invoke + return the branch-scoped drive state on the REAL Tauri mock-runtime bus, keep them read-only + branch-scoped (F-006) + R14-clean (only via the but-api gated wrapper), and regenerate the SDK so apps/desktop type-checks; mirrors MGMT-BE-004's Tauri/SDK delta shape minus the write/fleet-owner side
pattern_source: .spec/.../MGMT-BE-004 (the producer + Tauri/SDK delta to mirror); crates/gitbutler-tauri/src/lib.rs:339-340 (the forge read registration); crates/but-api/src/legacy/forge.rs:401 (get_review read-shape); crates/gitbutler-tauri/tests/mgmt_ipc_003_governance_commands.rs (the real-bus idiom); LPR-004/005/008 (the read verbs + the F-006 branch-scoped posture)
anti_pattern: adding a write/mutate/write-authority to a read (AC-3 catches it — the writers are LPR-003/004, a separate surface); narrowing the reads to per-principal self-scoping or claiming cross-principal disclosure is gated (F-006: they ARE branch-scoped — AC-4's second-principal-visible proof fails if self-scoped); a hand-rolled gitbutler-tauri IPC handler that reads the drive tables directly and bypasses the but-api gated wrapper (R14 violation — AC-5 greps for it); re-implementing review_status/list_comments as a new but-api reader (they are owned by LPR-004/005/008 — a second reader risks a divergent/ungated path); a source-only registration grep instead of the real-bus proof (use tauri::test::get_ipc_response); authoring a per-command allow-review_status/allow-list_comments capability FILE (core:default admits, like get_review); adding a read of the new tables to merge_gate.rs (a safe-seam violation, LPR-009 greps it); hand-editing packages/but-sdk/src/generated

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agent: implementer=tauri-implementer | reviewer=tauri-reviewer
rationale: This is a Tauri-shaped READ producer task — the deliverable is the DESKTOP command surface (registering the already-existing but-api #[but_api(napi)] review reads on the Tauri bus, mirroring the shipped tauri_get_review::get_review) + the real-Tauri-mock-runtime bus proof + the regenerated TS SDK that type-checks in the SvelteKit desktop. The Tauri-specific competencies — the command↔permission/registration atomicity (core:default admission + the lib.rs forge generate_handler registration + the real-bus invocation proof, NOT a hand-written allow-file), the read-only + branch-scoped (F-006) command posture, the R14 no-parallel-ungated-path discipline (the desktop reads ONLY through the gated but-api wrapper, no bypass IPC handler), the tauri::test::get_ipc_response real-bus idiom, and the SDK-regen-type-checks loop — are exactly tauri-implementer's domain (RULES.md routes the Tauri desktop shell + capabilities/IPC to the tauri-* triad). The underlying read logic is OWNED by LPR-004/005/008 (rust-implementer) and is NOT re-implemented here — this task exposes it; tauri-reviewer adversarially validates the command/permission parity (both registered, no orphan allow-file), the read-only contract (no mutate, no write authority), the branch-scoped disclosure (F-006, not self-scoped), the R14 no-bypass invariant (no direct drive-table read in gitbutler-tauri), and the SDK delta type-checks in apps/desktop.
coding_standards: crates/AGENTS.md (Result<T,E> + anyhow::Context; but-api is THE API boundary — the desktop reaches reads through the gated but-api wrapper, never a parallel IPC handler; solve the present problem directly — no new reader); RULES.md (but-api is THE API boundary; gitbutler-tauri is the desktop shell; after changing Rust APIs exposed via but-sdk run pnpm build:sdk && pnpm format, never hand-edit generated; the SvelteKit desktop consumes the generated SDK); brain/.rosetta/docs/tauri/ (commands.md the #[tauri::command] registration/admission; permissions.md core:default + autogenerated allow-<command>; testing.md tauri::test::get_ipc_response real-bus); crates/gitbutler-tauri/src/lib.rs (the forge command-registration block); crates/gitbutler-tauri/tests/mgmt_ipc_003_governance_commands.rs (the real-bus harness)

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: LPR-004 (list_comments — the comment/thread reader this exposes), LPR-005 (review_status derived PR lifecycle + the agent tag — the read this exposes), LPR-008 (review_status extended to the full reconciler drive state in one payload — the enriched read this exposes); Sprint 06a MGMT-IPC-003 (the real-Tauri-mock-runtime test harness — governance_app/governance_webview/get_ipc_response — this extends, and the lib.rs command-registration pattern); Sprint 06b MGMT-BE-004 (the producer + Tauri/SDK delta pattern this mirrors, READ-ONLY variant)
Blocks:     LPR-016 (the SvelteKit LocalReviewView that consumes review_status/list_comments via the regenerated SDK to render the local-PR drive state)
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "LPR-015",
  "proposed_by": "tauri-planner",
  "platforms": ["desktop"],
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "lpr_review_read_repo": {
      "description": "A real governed repo via but_testsupport::writable_scenario + invoke_bash committing .gitbutler/permissions.toml to the target ref (rev: reviews:write + pull_requests:write + comments:write; rev2/rev3 reviewer handles; viewer: a read-capable principal that authored no assignment/thread) + the mgmt_ipc_003 real-Tauri-mock-runtime harness (governance_app(test_desktop_session) + governance_webview + tauri::test::get_ipc_response). A real but_ctx::Context + DbHandle (the LPR-001 tables migrated). The branch refs/heads/feat drive state is seeded ONLY via the real LPR verbs (request_review/assign_reviewer → pending assignments for rev2 and rev3; post_comment → threads t1 (rev2, unresolved) and t2 (rev3, resolved); approve_review → an approved verdict@head), never direct row injection. BUT_AGENT_HANDLE is set per-case under #[serial_test::serial] via temp_env. For AC-3 the assignment/comment/verdict store + refs/objects/oplog are snapshotted before/after the reads; for AC-4 the reads are invoked as a THIRD principal `viewer` to prove branch-scoping.",
      "seed_method": "public_api",
      "records": [
        "but_testsupport::writable_scenario(...) + invoke_bash committing .gitbutler/permissions.toml (rev: reviews:write+pull_requests:write+comments:write; rev2/rev3 reviewers; viewer read-capable) to refs/heads/main;",
        "governance_app(test_desktop_session())? + governance_webview(&app)?;",
        "temp_env BUT_AGENT_HANDLE=rev under #[serial_test::serial]: request_review(refs/heads/feat, rev2) + assign_reviewer(refs/heads/feat, rev3) → two pending assignments; post_comment(refs/heads/feat, body, thread=t1) (rev2) + a resolved thread t2 (rev3); approve_review(refs/heads/feat) → an approved verdict@head;",
        "invoke review_status / list_comments on the bus via get_ipc_response (as `rev` for AC-1/2/3; as `viewer` for AC-4); snapshot the stores + refs/objects/oplog before/after for AC-3."
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN lpr_review_read_repo: refs/heads/feat fixtured (real verbs) with a pending rev2 assignment + an unresolved thread t1 + an approved verdict@head; the real Tauri mock runtime; BUT_AGENT_HANDLE=rev WHEN review_status is invoked on the bus (tauri::test::get_ipc_response) for refs/heads/feat THEN the command returns Ok with the SINGLE payload carrying the derived lifecycle status + open_assignments (the pending rev2 assignment) + unresolved_threads (t1) + verdict_at_head/approved (the approval@head) — the LPR-005/008 reconciler read, now reachable from the desktop bus",
      "verify": "cargo test -p gitbutler-tauri lpr_review_reads_register_and_return_branch_drive_state",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real Tauri mock-runtime bus invoking the registered but-api review_status #[but_api(napi)] command over a real but-db + gix fixture seeded via the real LPR verbs",
        "negative_control": {
          "would_fail_if": [
            "review_status were not registered on the bus — the real bus rejects it as 'not found'",
            "the payload omitted open_assignments / unresolved_threads / verdict_at_head — an incomplete drive state (the LPR-008 contract is one payload)",
            "a stub returned a fixed payload regardless of the fixtured state — the seeded assignment/thread/verdict would be absent or wrong",
            "the command were exposed via a hand-rolled gitbutler-tauri IPC handler bypassing the but-api wrapper (R14 violation)"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_review_read_repo",
            "action": { "actor": "ci", "steps": [ "seed refs/heads/feat with a pending rev2 assignment, an unresolved thread t1, and an approved verdict@head via the real verbs", "invoke review_status on the bus (get_ipc_response) for refs/heads/feat as rev", "inspect the single payload", "invoke an unregistered review-read probe; assert 'not found'" ] },
            "end_state": {
              "must_observe": [
                "review_status returns Ok with the derived lifecycle status",
                "open_assignments contains the pending rev2 assignment",
                "unresolved_threads contains thread t1",
                "verdict_at_head/approved reflects the approval@head — all in ONE payload",
                "the command is registered (invokes on the real bus); an unregistered probe is rejected 'not found'"
              ],
              "must_not_observe": [
                "review_status absent from the bus (unregistered)",
                "an incomplete drive-state payload (missing assignments/threads/verdict)",
                "a payload disconnected from the fixtured state (stub)",
                "a parallel ungated IPC handler serving the read"
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
      "description": "GIVEN lpr_review_read_repo: refs/heads/feat with two threads (t1 unresolved, t2 resolved) posted via post_comment; BUT_AGENT_HANDLE=rev WHEN list_comments is invoked on the bus for refs/heads/feat THEN the command returns Ok with the branch's comment list/threads (t1 and t2 both present, with their resolved flags) — the LPR-004 thread reader, now reachable from the desktop bus",
      "verify": "cargo test -p gitbutler-tauri list_comments_returns_branch_threads_on_bus",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real Tauri mock-runtime bus invoking the registered but-api list_comments #[but_api(napi)] command over a real but-db comment store",
        "negative_control": {
          "would_fail_if": [
            "list_comments were not registered on the bus ('not found')",
            "the payload omitted t1 or t2 (an incomplete thread list)",
            "the resolved flags were wrong (t2 reported unresolved or t1 reported resolved)"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_review_read_repo",
            "action": { "actor": "ci", "steps": [ "seed t1 (unresolved) + t2 (resolved) via post_comment/resolve_thread", "invoke list_comments on the bus for refs/heads/feat", "assert both threads present with correct resolved flags" ] },
            "end_state": {
              "must_observe": [ "list_comments returns Ok with t1 (unresolved) and t2 (resolved) both present" ],
              "must_not_observe": [ "list_comments absent from the bus", "a missing thread or a wrong resolved flag" ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN lpr_review_read_repo: refs/heads/feat with a pending assignment, an unresolved thread, and an approved verdict@head; the stores + refs/objects/oplog snapshotted WHEN review_status AND list_comments are each invoked on the bus THEN after both reads the local_review_assignments/comments/verdicts stores are byte-unchanged AND refs/objects/oplog are byte-unchanged (the reads mutate nothing); neither command requires a write authority (a read-capable-but-not-write caller still gets the payload)",
      "verify": "cargo test -p gitbutler-tauri review_reads_are_read_only_no_mutation",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real Tauri mock-runtime bus + real but-db store snapshot before/after + real gix ref/object/oplog snapshot",
        "negative_control": {
          "would_fail_if": [
            "a read mutated a drive-table row or a ref/object/oplog (the before/after snapshots differ — a read that writes)",
            "a read required a write authority (a read-capable caller is denied — the read is wrongly write-gated)"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_review_read_repo",
            "action": { "actor": "ci", "steps": [ "snapshot the local_review_assignments/comments/verdicts stores + refs/objects/oplog", "invoke review_status AND list_comments on the bus", "re-snapshot the stores + refs/objects/oplog", "invoke the reads as a read-capable-but-not-write caller; assert the payload still returns" ] },
            "end_state": {
              "must_observe": [ "the drive stores are byte-unchanged after both reads", "refs/objects/oplog are byte-unchanged", "a read-capable-but-not-write caller still gets the payload (no write authority required)" ],
              "must_not_observe": [ "any drive-table or ref/object/oplog mutation from a read", "a read denied for lack of a write authority" ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN lpr_review_read_repo: refs/heads/feat with assignments/threads from TWO distinct principals (rev2 + rev3); the invoking caller is a THIRD principal `viewer` who authored none WHEN review_status AND list_comments are invoked on the bus by `viewer` THEN review_status's open_assignments includes BOTH rev2's and rev3's assignments AND list_comments returns BOTH rev2's and rev3's threads — the whole branch's review surface is disclosed to `viewer` (branch-scoped shared drive state, F-006), NOT narrowed to the caller's own (the reads are explicitly NOT per-principal self-scoped)",
      "verify": "cargo test -p gitbutler-tauri review_reads_are_branch_scoped_not_self_scoped",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real Tauri mock-runtime bus invoking the reads as a third principal over a multi-principal drive fixture",
        "negative_control": {
          "would_fail_if": [
            "review_status/list_comments returned only `viewer`'s own assignments/threads (per-principal self-scoping — F-006 violated, the reconciler can't read the shared surface)",
            "rev2's or rev3's assignment/thread were filtered out for `viewer` (wrongly self-scoped)"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_review_read_repo",
            "action": { "actor": "ci", "steps": [ "fixture rev2's and rev3's assignments + threads on refs/heads/feat", "invoke review_status AND list_comments on the bus as a THIRD principal `viewer`", "assert BOTH rev2's and rev3's assignments/threads are returned to `viewer`" ] },
            "end_state": {
              "must_observe": [ "review_status open_assignments includes BOTH rev2's and rev3's assignments", "list_comments returns BOTH rev2's and rev3's threads", "the whole branch surface is disclosed to the third-principal viewer (branch-scoped, F-006)" ],
              "must_not_observe": [ "only the caller's own assignments/threads (self-scoping)", "rev2's or rev3's data filtered out for viewer" ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the gitbutler-tauri crate + the registered review_status/list_comments commands + the generated SDK WHEN the desktop registration is inspected AND `pnpm build:sdk && pnpm format` runs AND apps/desktop type-checks THEN the ONLY desktop entry to the reads is the registered but-api #[but_api(napi)]-generated tauri module (no hand-rolled gitbutler-tauri IPC handler reads the drive tables directly and bypasses the but-api boundary — a grep finds no direct drive-table read); AND packages/but-sdk/src/generated contains review_status/list_comments + their return types and the generated TS type-checks in apps/desktop (no hand-edit)",
      "verify": "cargo test -p gitbutler-tauri lpr_review_reads_register_and_return_branch_drive_state && pnpm build:sdk && pnpm format && pnpm -F @gitbutler/desktop check && grep -rq \"review_status\\|reviewStatus\" packages/but-sdk/src/generated && grep -rq \"list_comments\\|listComments\" packages/but-sdk/src/generated",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "source-structure check (the desktop read entry is the but-api command, no bypass handler) + real pnpm build:sdk && pnpm format SDK regen + pnpm -F @gitbutler/desktop check",
        "negative_control": {
          "would_fail_if": [
            "a hand-rolled gitbutler-tauri IPC handler reads local_review_assignments/comments/meta directly, bypassing the but-api gated wrapper (R14 violation — the grep finds a direct drive-table read)",
            "the SDK regen omitted review_status/list_comments or their return types",
            "the generated TS failed tsc in apps/desktop",
            "a per-command allow-review_status/allow-list_comments capability file were introduced (the forge reads ride core:default)"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_review_read_repo",
            "action": { "actor": "ci", "steps": [ "grep crates/gitbutler-tauri/src for a direct local_review_assignments()/local_review_comments()/local_review_meta() read; assert none (no bypass handler)", "run pnpm build:sdk && pnpm format", "assert the generated SDK contains review_status/list_comments + their return types", "run pnpm -F @gitbutler/desktop check; assert the generated TS type-checks" ] },
            "end_state": {
              "must_observe": [
                "the ONLY desktop entry to the reads is the registered but-api #[but_api(napi)] command (no direct drive-table read in gitbutler-tauri)",
                "packages/but-sdk/src/generated contains review_status/list_comments + their return types",
                "the generated TS type-checks in apps/desktop",
                "no allow-review_status/allow-list_comments capability file exists (core:default admits)"
              ],
              "must_not_observe": [
                "a hand-rolled IPC handler reading the drive tables directly (R14 bypass)",
                "the SDK missing the reads or failing tsc",
                "a hand-written per-command allow-review_status/allow-list_comments file"
              ]
            }
          }
        ]
      }
    },
    { "id": "TC-1", "type": "test_criterion", "description": "review_status invokes on the real bus and returns the SINGLE payload with the derived lifecycle + the pending rev2 assignment + unresolved thread t1 + the verdict-at-head", "verify": "cargo test -p gitbutler-tauri lpr_review_reads_register_and_return_branch_drive_state", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "review_status AND list_comments are registered in lib.rs's forge command block; an unregistered review-read probe is rejected ('not found')", "verify": "cargo test -p gitbutler-tauri lpr_review_reads_register_and_return_branch_drive_state", "maps_to_ac": "AC-1" },
    { "id": "TC-3", "type": "test_criterion", "description": "list_comments invokes on the real bus and returns the branch's threads (t1 unresolved + t2 resolved, with resolved flags)", "verify": "cargo test -p gitbutler-tauri list_comments_returns_branch_threads_on_bus", "maps_to_ac": "AC-2" },
    { "id": "TC-4", "type": "test_criterion", "description": "after review_status AND list_comments the local_review_assignments/comments/verdicts stores + refs/objects/oplog are byte-unchanged (read-only)", "verify": "cargo test -p gitbutler-tauri review_reads_are_read_only_no_mutation", "maps_to_ac": "AC-3" },
    { "id": "TC-5", "type": "test_criterion", "description": "neither read requires a write authority — a read-capable-but-not-write caller still gets the payload", "verify": "cargo test -p gitbutler-tauri review_reads_are_read_only_no_mutation", "maps_to_ac": "AC-3" },
    { "id": "TC-6", "type": "test_criterion", "description": "review_status open_assignments includes BOTH rev2's and rev3's assignments when invoked by a third principal `viewer` (branch-scoped, not self-scoped)", "verify": "cargo test -p gitbutler-tauri review_reads_are_branch_scoped_not_self_scoped", "maps_to_ac": "AC-4" },
    { "id": "TC-7", "type": "test_criterion", "description": "list_comments returns BOTH rev2's and rev3's threads to `viewer` (branch-scoped disclosure, F-006)", "verify": "cargo test -p gitbutler-tauri review_reads_are_branch_scoped_not_self_scoped", "maps_to_ac": "AC-4" },
    { "id": "TC-8", "type": "test_criterion", "description": "the ONLY desktop entry to the reads is the registered but-api #[but_api(napi)] command — no hand-rolled gitbutler-tauri IPC handler reads the drive tables directly (no R14 bypass)", "verify": "cargo test -p gitbutler-tauri lpr_review_reads_register_and_return_branch_drive_state", "maps_to_ac": "AC-5" },
    { "id": "TC-9", "type": "test_criterion", "description": "pnpm build:sdk && pnpm format regenerates packages/but-sdk/src/generated containing review_status/list_comments + their return types, and the generated TS type-checks in apps/desktop (no hand-edit)", "verify": "pnpm build:sdk && pnpm format && pnpm -F @gitbutler/desktop check && grep -rq \"review_status\\|reviewStatus\" packages/but-sdk/src/generated && grep -rq \"list_comments\\|listComments\" packages/but-sdk/src/generated", "maps_to_ac": "AC-5" },
    { "id": "TC-10", "type": "test_criterion", "description": "no allow-review_status/allow-list_comments capability file is introduced (the forge reads ride core:default like get_review)", "verify": "cargo test -p gitbutler-tauri lpr_review_reads_register_and_return_branch_drive_state", "maps_to_ac": "AC-5" }
  ]
}
-->
