# LPR-006: `Project.keep_reviews_local: DefaultTrue` per-project setting + default-local wiring + remote-mirror gate (named seam only)

> Status: ✅ Completed
> Commit: 55923061e1
> Reviewer: deferred to PHASE 4.5 red-hat closeout — committed prior session; keep_reviews_local DefaultTrue + mirror gate
> Updated: 2026-06-22T18:07:12Z

## What this does

Add a per-project `keep_reviews_local: DefaultTrue` field to `gitbutler_project::Project` (beside the existing forge knobs `forge_override`/`preferred_forge_user`), so agent-authored reviews **default to the local layer** and older project files without the field deserialize to local. Wire the default-local behavior into `request_review` (an agent review stays local while the flag is true), and add the **remote-mirror gate** — the conditional that makes the remote-mirror path unreachable while `keep_reviews_local == true`. The mirror **path itself is NOT built** (it is the named-for-later `mirror_local_review` seam, §D). The setting is a per-project operator preference set through the normal project-settings surface (the same path that sets `forge_override`), owned by the desktop human under the R12 trusted-desktop model — NOT `administration:write`-gated, NOT ref-pinned committed config. An untrusted project-store write that flips it (→ agent PRs mirror to a public forge) is the named residual R21.

## Why

Sprint 07 · PRD UC-LPR-03 · capability CAP-CONFIG-01. The forge should be optional, not load-bearing: a per-project "keep PRs local" setting makes agent-authored reviews default to the local layer (no remote GitHub PR), and any remote mirroring is gated behind the setting. This is what makes the whole loop offline-capable and the local path the cheapest path. The field is a **local operator preference** about where review _artifacts_ go — the same class as `forge_override`/`preferred_forge_user`, persisted in the project store — **not** a governed authorization fact (which lives in ref-pinned `permissions.toml`/`gates.toml`).

## How to verify

PRIMARY **AC-1** — `cargo test -p gitbutler-project keep_reviews_local_defaults_true_for_old_project_file`: an older project JSON **without** the `keep_reviews_local` field deserializes with `keep_reviews_local == true` (local) via `DefaultTrue`, and the value is readable from the project store. Full gate set in the spec below.

## Scope

- crates/gitbutler-project/src/project.rs (MODIFY — add `#[serde(default)] #[cfg_attr(feature = "export-schema", schemars(schema_with = "but_schemars::default_true"))] pub keep_reviews_local: DefaultTrue` beside `ok_with_force_push` (project.rs:106) / the forge knobs (project.rs:129/:134); default it in `default_with_id` (project.rs:139) as `keep_reviews_local: Default::default()`)
- crates/but-api/src/legacy/forge.rs (MODIFY — wire `request_review` (LPR-003) default-local: when `keep_reviews_local` is true, the review stays local — no `create_forge_review`; add the remote-mirror GATE conditional `keep_reviews_local == false` that makes the mirror path unreachable while true — the mirror path itself is NOT built)
- the but-api / gitbutler-project project-settings write path that mutates `keep_reviews_local` (MODIFY/NEW — persist the value through the same project-settings surface that sets `forge_override` (the project store, storage.rs); it is a per-project operator preference under R12 trusted-desktop — NOT `administration:write`-gated, NOT ref-pinned; do NOT compose enforce_administration_write_gate)
- crates/gitbutler-project/tests/ (NEW/MODIFY — the serde-default deserialization proof AC-1) + crates/but-api/tests/keep_reviews_local.rs (NEW — the admin-gate + default-local + mirror-gate proofs AC-2..AC-5 against real but-db + gix via but_testsupport)
- packages/but-sdk/src/generated/\*\* (REGENERATE ONLY via `pnpm build:sdk && pnpm format` — NEVER hand-edit; the actual regen + N-API audit is LPR-010's gate)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: LPR-006 — Project.keep_reviews_local: DefaultTrue + default-local wiring + remote-mirror gate
================================================================================

TASK_TYPE:   FEATURE
STATUS:      Backlog
PRIORITY:    P1
AGENT:       implementer=rust-implementer | reviewer=rust-reviewer
EFFORT:      M  (120 min)
PROPOSED-BY: rust-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-LPR-03
CAPABILITIES:CAP-CONFIG-01

RUNTIME_COMMANDS:
  test:  cargo test -p gitbutler-project keep_reviews_local_defaults_true_for_old_project_file
  check: cargo check -p gitbutler-project --all-targets
  lint:  cargo clippy -p gitbutler-project -p but-api --all-targets

--------------------------------------------------------------------------------
TYPE DESIGN (Rust shapes this task introduces)
--------------------------------------------------------------------------------
FIELD (additive on gitbutler_project::Project, project.rs:72):
  - `#[serde(default)]`
    `#[cfg_attr(feature = "export-schema", schemars(schema_with = "but_schemars::default_true"))]`
    `pub keep_reviews_local: DefaultTrue`
  - placed beside `ok_with_force_push` (project.rs:106) — the EXACT precedent: `#[serde(default)] + DefaultTrue + the default_true schemars attr`. Defaulted in `default_with_id` (project.rs:139) as `keep_reviews_local: Default::default()` (which is `DefaultTrue(true)` per default_true.rs:18-23).
TYPE: `DefaultTrue` (crates/gitbutler-project/src/default_true.rs) — `Default::default()` is `DefaultTrue(true)` (default_true.rs:21); `From<DefaultTrue> for bool` (:25) and `From<bool> for DefaultTrue` (:32); `PartialEq<bool>` (:69) so `keep_reviews_local == false` reads directly; `Deref<Target = bool>` (:53). It is `Clone, Copy`.
ERROR STRATEGY:
  - The keep_reviews_local setter persists through the project store (storage.rs) the same way `forge_override` is set — it is NOT administration:write-gated, so it composes NO enforce_administration_write_gate and surfaces NO perm.denied for this preference. Project (de)serialization uses serde's existing Result; no new error type.
OWNERSHIP PLAN:
  - `keep_reviews_local` is `Copy` (DefaultTrue is `Copy`). Read it as a `bool` via `bool::from(project.keep_reviews_local)` or the `== false` comparison at the gate site. Borrow `&Project` to read; the setter takes `&mut Project` (or rewrites the stored project via the project store, storage.rs).
DOC POINTERS (read before coding):
  - brain/docs/rust/ownership-borrowing.md → Copy types + From conversions (DefaultTrue → bool); borrow &Project to read the flag
  - brain/docs/rust/error-handling.md → serde's existing Result for project (de)serialization (the keep_reviews_local preference is NOT administration:write-gated — no enforce_administration_write_gate, no perm.denied)
  - brain/docs/rust/testing.md → serde round-trip (old-file-without-field deserialize); real gitbutler-project project store round-trip via but_testsupport (the forge_override persistence path)

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Proven against the real gitbutler-project Project (de)serialization + the real gitbutler-project project store (storage.rs) + real but-api request_review/publish_review + real gix via but_testsupport: (1) an older project JSON WITHOUT the keep_reviews_local field deserializes with keep_reviews_local == true (local) via DefaultTrue, and the value is readable from the project store; (2) the desktop operator sets keep_reviews_local through the project-settings surface (the forge_override path) and it persists to the project store — it is a per-project operator preference under R12 trusted-desktop, NOT administration:write-gated; an untrusted project-store write that flips it is the named residual R21; (3) an agent-authored review (BUT_AGENT_HANDLE) defaults to local when keep_reviews_local == true — a local review object is created (assignment row present) and NO remote forge PR is opened (no create_forge_review call; the forge_reviews cache is unchanged); (4) the remote-mirror path is gated OFF while keep_reviews_local == true — the mirror verb is a named-for-later seam, NOT invoked; the forge_reviews + sync_reviews bridge is NOT exercised for the agent review; (5) the existing remote-PR behavior is preserved when keep_reviews_local == false — the shipped publish_review/open-PR path behaves as before v1.5.0; cargo test green; clippy clean.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST use `DefaultTrue` (NOT a plain `bool`) so the default is `true` = local AND older project files without the field deserialize to local. The EXACT precedent is `ok_with_force_push: DefaultTrue` (project.rs:106) with `#[serde(default)]` + the `schemars(schema_with = "but_schemars::default_true")` cfg_attr. Read project.rs:99-106 and mirror that field's three attributes line-for-line.
- [NEVER] NEVER use a plain `bool` for keep_reviews_local. `force_push_protection` (project.rs:108) and `husky_hooks_enabled` (project.rs:113) are `#[serde(default)] pub <name>: bool` — they default to FALSE, which for keep_reviews_local would mean REMOTE-by-default (the WRONG default; an old project file would silently default to remote mirroring). Those two are the anti-precedent. The correct precedent is ONLY `ok_with_force_push: DefaultTrue`.
- [MUST] MUST default keep_reviews_local in `default_with_id` (project.rs:139) as `keep_reviews_local: Default::default()` (= DefaultTrue(true) via default_true.rs:18-23), exactly as `ok_with_force_push: Default::default()` is defaulted at project.rs:146. A new project created via default_with_id MUST be local.
- [MUST] MUST place the field beside the per-project forge knobs (`forge_override` project.rs:129, `preferred_forge_user` project.rs:134) — keep_reviews_local is the same CLASS: a local operator preference about where review artifacts go, persisted in the project store (crates/gitbutler-project/src/storage.rs). It is NOT governed ref-pinned config.
- [MUST] MUST persist keep_reviews_local through the SAME project-settings surface that sets forge_override (the project store, storage.rs) — it is a per-project operator preference under the R12 trusted-desktop model, owned by the desktop human, NOT administration:write-gated and NOT ref-pinned committed config. Do NOT compose enforce_administration_write_gate and do NOT add a perm.denied gate for this preference. The accepted residual is R21: an untrusted project-store write can flip it to false (→ agent PRs mirror to a public forge) — name it, never present keep_reviews_local as an authorization boundary.
- [MUST] MUST gate the remote-mirror path behind `keep_reviews_local == false` (read via the DefaultTrue == bool PartialEq, default_true.rs:69). While true (the default), request_review (LPR-003) creates ONLY the local review object — no create_forge_review, no forge PR. AC-3/AC-4 prove no remote PR is opened and the forge_reviews + sync_reviews bridge is not exercised for the agent review.
- [NEVER] NEVER BUILD the remote-mirror path in this sprint. The mirror itself (a future `mirror_local_review` verb, gated by PullRequestsWrite, riding but_forge::create_forge_review (but-forge review.rs:1251) + sync_reviews (review.rs:1349)) is SPECIFIED-NOT-BUILT (tech-delta §D). This task builds ONLY the GATE (the `keep_reviews_local == false` conditional that makes the mirror path unreachable while true). Adding the mirror code is OUT of scope (AC-4 asserts the mirror verb is NOT invoked).
- [MUST] MUST preserve the existing remote-PR behavior when keep_reviews_local == false — the shipped publish_review/open-PR path (forge.rs:480) behaves as before v1.5.0. v1.5.0 ADDS a local default; it does NOT remove the forge path for projects that want it (AC-5).
- [NEVER] NEVER ref-pin keep_reviews_local as committed config (permissions.toml/gates.toml). Those gate DECISIONS and are read at the target ref; keep_reviews_local is a local artifact-routing preference in the project store, the same class as forge_override. Do NOT write it to gates.toml or read it through load_governance_config.
- [NEVER] NEVER administration:write-gate the keep_reviews_local setter (do NOT compose enforce_administration_write_gate). It is a per-project operator preference under R12 trusted-desktop (the forge_override class), not a governed-config mutation; gating it would misrepresent a trusted-desktop preference as an authorization boundary. The R21 residual (an untrusted project-store write flips it) stays NAMED, not closed.
- [NEVER] NEVER implement the but-* skill contract here. The skill contract ("on governed-project init, if unset, set keep_reviews_local = true" — tech-delta §C) is belt-and-suspenders (the field already defaults true via DefaultTrue) and is DOCUMENTED by LPR-010, not enforced by this task. This task provides only the default-true field the skill relies on.
- [NEVER] NEVER add new gitbutler-* usage in NEW code. NOTE: gitbutler-project IS a legacy gitbutler-* crate, and Project is where forge_override/preferred_forge_user already live — adding keep_reviews_local THERE is a sanctioned localized edit (the field's correct home), NOT new gitbutler-* usage introduced in newer code. State this in the completion report.
- [STRICTLY] STRICTLY persist keep_reviews_local through the project store (storage.rs) the same way forge_override is set — do NOT route it through enforce_administration_write_gate (config_mutate.rs:18). That gate is for ref-pinned governed-config mutations, NOT this trusted-desktop project preference (R12/R21).
- [STRICTLY] STRICTLY keep the SDK regen out of this task's hands beyond running it — the actual regen + N-API audit gate is LPR-010; do not hand-edit packages/but-sdk/src/generated.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [x] AC-1 [PRIMARY]: an older project file WITHOUT the field deserializes to local (keep_reviews_local == true via DefaultTrue); the value is readable from the project store
- [x] AC-2: keep_reviews_local persists through the project-settings surface (the forge_override path, project store) — a per-project operator preference under R12 trusted-desktop, NOT administration:write-gated; the R21 residual (untrusted project-store write flips it) is named, not closed
- [x] AC-3: an agent-authored review defaults to local when keep_reviews_local == true — a local review object is created and NO remote forge PR is opened
- [x] AC-4: the remote-mirror path is gated OFF while keep_reviews_local == true — the mirror verb is a named-for-later seam, NOT invoked; forge_reviews + sync_reviews NOT exercised for the agent review
- [x] AC-5: existing remote-PR behavior is preserved when keep_reviews_local == false (the shipped publish_review/open-PR path behaves as before v1.5.0)
- [x] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: old project file deserializes to local (DefaultTrue), value readable
  GIVEN: lpr_project_files: a project JSON string WITHOUT a keep_reviews_local key (an older project file shape) + a project JSON WITH keep_reviews_local=false
  WHEN:  each is deserialized into Project via serde, and keep_reviews_local is read from the resulting project / the project store
  THEN:  the old-file (no key) deserializes with keep_reviews_local == true (local — DefaultTrue's #[serde(default)] supplies true, default_true.rs:21); the explicit-false file deserializes with keep_reviews_local == false; the value round-trips through the project store (read back equals what was stored)
  TEST_TIER: integration   VERIFICATION_SERVICE: real gitbutler-project Project serde (de)serialization + the real project store (storage.rs)
  VERIFY: cargo test -p gitbutler-project keep_reviews_local_defaults_true_for_old_project_file

AC-2: keep_reviews_local persists through the project-settings surface (R12 trusted-desktop preference, NOT admin-gated)
  GIVEN: lpr_project_store: a real Project in the project store (storage.rs) at its default (keep_reviews_local == true)
  WHEN:  the desktop operator sets keep_reviews_local = false through the project-settings surface (the same path that persists forge_override)
  THEN:  the value persists to the project store (keep_reviews_local == false read back) WITHOUT any administration:write gate — it is a per-project operator preference under the R12 trusted-desktop model; there is NO enforce_administration_write_gate / perm.denied on this preference; the R21 residual (an untrusted project-store write can flip it) stays named, not closed
  TEST_TIER: integration   VERIFICATION_SERVICE: the real gitbutler-project project store (storage.rs) round-trip of keep_reviews_local via the project-settings surface (the forge_override path)
  VERIFY: cargo test -p gitbutler-project keep_reviews_local_persists_via_project_settings

AC-3: an agent review defaults to local when keep_reviews_local == true (no remote PR)
  GIVEN: lpr_governed_repo with keep_reviews_local == true (the default); BUT_AGENT_HANDLE=rev (an agent principal holding pull_requests:write); the forge_reviews cache captured before
  WHEN:  `but review request refs/heads/feat --reviewer rev2` runs (request_review, LPR-003) under the default keep_reviews_local
  THEN:  a local review object is created (a local_review_assignments row is present) AND NO remote forge PR is opened — no create_forge_review call occurs and the forge_reviews cache is byte-unchanged from before
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api request_review under keep_reviews_local=true + real but-db (assignment present; forge_reviews unchanged) + real gix
  VERIFY: cargo test -p but-api agent_review_defaults_local_no_remote_pr

AC-4: the remote-mirror path is gated off while local (mirror verb not invoked)
  GIVEN: lpr_governed_repo with keep_reviews_local == true; an agent review opened via request_review
  WHEN:  the mirror path is inspected for that agent review (the keep_reviews_local == false gate is false)
  THEN:  the remote-mirror path is unreachable — the (named-for-later) mirror verb is NOT invoked; the forge_reviews + but_forge::sync_reviews (review.rs:1349) bridge is NOT exercised for the agent review (the mirror is a specified-not-built seam, gated behind keep_reviews_local == false)
  TEST_TIER: api-contract   VERIFICATION_SERVICE: real but-api request_review path under keep_reviews_local=true; assert the mirror/create_forge_review/sync_reviews path is not reached (the gate short-circuits before it)
  VERIFY: cargo test -p but-api remote_mirror_gated_off_while_local

AC-5: existing remote-PR behavior preserved when keep_reviews_local == false
  GIVEN: lpr_governed_repo with keep_reviews_local == false; the shipped publish_review/open-PR path
  WHEN:  the existing publish_review (forge.rs:480) / open-PR path is driven (keep_reviews_local == false)
  THEN:  the shipped remote-PR path behaves exactly as before v1.5.0 (the forge PR is opened via the existing verb) — v1.5.0 added a local default WITHOUT removing the forge path for projects that opt out of local-only
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api publish_review path under keep_reviews_local=false + the shipped forge bridge (behavior-unchanged proof)
  VERIFY: cargo test -p but-api remote_pr_path_preserved_when_flag_false

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): a project JSON without keep_reviews_local deserializes with keep_reviews_local == true (DefaultTrue); an explicit false deserializes to false
    VERIFY: cargo test -p gitbutler-project keep_reviews_local_defaults_true_for_old_project_file
- TC-2 (-> AC-1): default_with_id() produces a Project with keep_reviews_local == true (a new project is local)
    VERIFY: cargo test -p gitbutler-project keep_reviews_local_defaults_true_for_old_project_file
- TC-3 (-> AC-2): keep_reviews_local persists through the project-settings surface (the forge_override path) to the project store; it is a per-project operator preference under R12 trusted-desktop, NOT administration:write-gated
    VERIFY: cargo test -p gitbutler-project keep_reviews_local_persists_via_project_settings
- TC-4 (-> AC-3): request_review under keep_reviews_local=true creates a local assignment row and leaves the forge_reviews cache unchanged (no create_forge_review)
    VERIFY: cargo test -p but-api agent_review_defaults_local_no_remote_pr
- TC-5 (-> AC-4): under keep_reviews_local=true the mirror verb is not invoked and sync_reviews is not exercised for the agent review (the gate short-circuits)
    VERIFY: cargo test -p but-api remote_mirror_gated_off_while_local
- TC-6 (-> AC-5): under keep_reviews_local=false the shipped publish_review/open-PR path behaves as before v1.5.0 (forge PR opened via the existing verb)
    VERIFY: cargo test -p but-api remote_pr_path_preserved_when_flag_false

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-CONFIG-01
provides:
  - gitbutler_project::Project.keep_reviews_local: DefaultTrue — a per-project local-by-default review-artifact-routing preference (default true; old project files deserialize to local), persisted in the project store
  - the default-local wiring in request_review (an agent review stays local while the flag is true — no create_forge_review)
  - the remote-mirror GATE (the keep_reviews_local == false conditional that makes the named-for-later mirror path unreachable while true) — the mirror path itself is NOT built
  - a project-settings write path for keep_reviews_local persisting to the project store the same way forge_override is set (a per-project operator preference under R12 trusted-desktop — NOT administration:write-gated)
consumes:
  - gitbutler_project::default_true::DefaultTrue (the field type + Default/From<bool> conversions) — the ok_with_force_push precedent (project.rs:106)
  - the gitbutler-project project store (storage.rs) — the same persistence path forge_override uses (the keep_reviews_local preference is persisted here, NOT through an admin gate)
  - crate::legacy::forge::request_review (LPR-003 — the verb whose default-local behavior this gates) + the shipped publish_review (forge.rs:480, behavior preserved when flag is false)
  - the named-for-later but_forge::{create_forge_review (review.rs:1251), sync_reviews (review.rs:1349)} mirror bridge — REFERENCED as the deferred seam, NOT invoked
boundary_contracts:
  - CAP-CONFIG-01: keep_reviews_local is a LOCAL operator preference in the project store (the forge_override class), NOT ref-pinned committed config. The default is local (DefaultTrue) and old project files deserialize to local. Setting it is a per-project operator preference through the project-settings surface (the forge_override path), owned by the desktop human under R12 trusted-desktop — NOT administration:write-gated, NOT ref-pinned; the R21 residual (an untrusted project-store write flips it) is named, not closed. Remote mirroring is reachable ONLY when keep_reviews_local == false; while true the loop is fully local and the mirror path (a specified-not-built seam) is unreachable. The existing forge path is preserved when the flag is false.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/gitbutler-project/src/project.rs (MODIFY — add the keep_reviews_local: DefaultTrue field beside ok_with_force_push/the forge knobs; default it in default_with_id)
  - crates/but-api/src/legacy/forge.rs (MODIFY — wire request_review default-local; add the keep_reviews_local == false remote-mirror GATE conditional; do NOT build the mirror path)
  - the project-settings write path that persists keep_reviews_local (MODIFY/NEW — persist via the project store, storage.rs, the same path forge_override uses; NOT administration:write-gated)
  - crates/gitbutler-project/tests/ (NEW/MODIFY — the serde-default deserialize proof, AC-1)
  - crates/but-api/tests/keep_reviews_local.rs (NEW — the admin-gate + default-local + mirror-gate proofs AC-2..AC-5)
  - packages/but-sdk/src/generated/** (REGENERATE ONLY — NEVER hand-edit; the regen gate is LPR-010)
writeProhibited:
  - crates/but-forge/src/review.rs — CONSUME-only (the named-for-later mirror bridge); do NOT build create_forge_review/sync_reviews wiring for the local review (the mirror is deferred — §D)
  - crates/but-api/src/legacy/merge_gate.rs, review_requirement.rs — CONSUME-only (the safe seam); keep_reviews_local NEVER feeds the merge gate
  - crates/but-api/src/legacy/config_mutate.rs — do NOT use here; keep_reviews_local is a trusted-desktop project preference, NOT an administration:write-gated config mutation (do NOT compose enforce_administration_write_gate)
  - crates/but-authz/src/{authority.rs, authorize.rs, config.rs} — no new Authority variant; keep_reviews_local is NOT governed ref-pinned config (do not route it through load_governance_config)
  - the permissions.toml/gates.toml writers (this is a Project preference in the project store, not committed config)
  - any OTHER gitbutler-* crate (gitbutler-project is the sanctioned home for this field; introduce no new gitbutler-* usage elsewhere)
  - Any file not in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/gitbutler-project/src/project.rs [99-106] — [PRIMARY PATTERN — mirror line-for-line] `ok_with_force_push: DefaultTrue` with `#[serde(default)]` + `#[cfg_attr(feature = "export-schema", schemars(schema_with = "but_schemars::default_true"))]`. This is the EXACT three-attribute shape keep_reviews_local must use so default=true (local) and old files deserialize to local.
2. crates/gitbutler-project/src/project.rs [107-113] — [THE ANTI-PRECEDENT — do NOT copy] `force_push_protection: bool` (:108) and `husky_hooks_enabled: bool` (:113) are plain `#[serde(default)] pub <name>: bool` defaulting FALSE. Using a plain bool here would default keep_reviews_local to FALSE = remote (the WRONG default). Use DefaultTrue, not bool.
3. crates/gitbutler-project/src/project.rs [128-134, 137-158] — `forge_override: Option<String>` (:129) + `preferred_forge_user` (:134) are the sibling per-project forge knobs keep_reviews_local sits beside (same class). `default_with_id` (:139) defaults every field, e.g. `ok_with_force_push: Default::default()` (:146) — add `keep_reviews_local: Default::default()` there.
4. crates/gitbutler-project/src/default_true.rs [1-37, 69-81] — DefaultTrue: `Default::default()` is DefaultTrue(true) (:21); `From<DefaultTrue> for bool` (:25) and `From<bool> for DefaultTrue` (:32); `PartialEq<bool>` (:69) so `project.keep_reviews_local == false` reads directly at the mirror gate.
5. crates/gitbutler-project/src/storage.rs + the project-settings surface that sets forge_override — keep_reviews_local persists the SAME way forge_override does (a per-project operator preference in the project store). It is NOT administration:write-gated; do NOT route it through config_mutate::enforce_administration_write_gate (that is for ref-pinned governed-config mutations, NOT a trusted-desktop project preference, R12/R21).
6. crates/but-api/src/legacy/forge.rs [480-..., 520-546] — the shipped publish_review (:480, the remote-PR path preserved when flag=false) and request_review (LPR-003, the verb whose default-local behavior this gates). Read to find where the keep_reviews_local == false gate must wrap the create_forge_review call.
7. crates/but-forge/src/review.rs [1251, 1349] — create_forge_review (:1251) + sync_reviews (:1349): the named-for-later mirror bridge (tech-delta §D). REFERENCE only — this task does NOT build the mirror; it only gates the path off while keep_reviews_local == true.
8. crates/gitbutler-project/src/storage.rs — the project store keep_reviews_local persists in (the same store forge_override uses). Read to round-trip the value (AC-1/AC-2).

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- cargo test -p gitbutler-project keep_reviews_local_defaults_true_for_old_project_file   -> Exit 0; no-field deserializes to true; explicit false to false; default_with_id is local
- cargo test -p gitbutler-project keep_reviews_local_persists_via_project_settings   -> Exit 0; the preference persists through the project-settings surface (the forge_override path); NOT administration:write-gated
- cargo test -p but-api agent_review_defaults_local_no_remote_pr   -> Exit 0; local assignment present; forge_reviews unchanged (no create_forge_review)
- cargo test -p but-api remote_mirror_gated_off_while_local   -> Exit 0; mirror verb not invoked; sync_reviews not exercised for the agent review
- cargo test -p but-api remote_pr_path_preserved_when_flag_false   -> Exit 0; shipped publish_review/open-PR path behaves as before v1.5.0
- cargo check -p gitbutler-project --all-targets   -> Exit 0
- cargo check -p but-api --all-targets   -> Exit 0
- cargo clippy -p gitbutler-project -p but-api --all-targets   -> Exit 0
- cargo test -p but-authz invariant_build_gates   -> Exit 0; forge.rs honesty grep green (no role-name/human-vs-AI branch added by the gate)
- cargo fmt --check   -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references:
  - crates/gitbutler-project/src/project.rs:106 (ok_with_force_push: DefaultTrue — THE precedent), :108/:113 (force_push_protection/husky_hooks_enabled — the plain-bool anti-precedent), :129/:134 (forge_override/preferred_forge_user — sibling forge knobs), :139/:146 (default_with_id)
  - crates/gitbutler-project/src/default_true.rs:18-37,69 (the DefaultTrue impl + == bool)
  - crates/gitbutler-project/src/storage.rs + the project-settings surface that sets forge_override (the same persistence path the keep_reviews_local preference uses — NOT administration:write-gated)
  - .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md §C (the keep_reviews_local field + the skill contract), §D (the remote-mirror seam — specified, NOT built)
  - .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/04-e2e-testing-criteria.md (T-LPR-014/015/016/017/018 — the criteria these ACs realize)
code_skeleton: |
  // crates/gitbutler-project/src/project.rs — beside ok_with_force_push (project.rs:106)
  /// Keep agent-authored reviews on the local layer (no remote forge PR) by default.
  #[serde(default)]
  #[cfg_attr(feature = "export-schema", schemars(schema_with = "but_schemars::default_true"))]
  pub keep_reviews_local: DefaultTrue,
  // ... and in default_with_id (project.rs:139): keep_reviews_local: Default::default(),
  //
  // crates/but-api — request_review default-local + the mirror GATE:
  //   if !bool::from(project.keep_reviews_local) {           // == keep_reviews_local == false
  //       // remote-mirror path — SPECIFIED-NOT-BUILT (§D); the future mirror_local_review verb
  //       // rides but_forge::create_forge_review + sync_reviews. NOT built in Sprint 07.
  //   }
  //   // while keep_reviews_local == true (the default) the review stays local: no create_forge_review.
  //
  // the keep_reviews_local setter (project-settings surface — the forge_override path):
  //   // persist project.keep_reviews_local = value via the project store (storage.rs) — the SAME
  //   // path that persists forge_override. NO enforce_administration_write_gate (R12 trusted-desktop
  //   // preference, NOT a governed-config mutation). The R21 residual (untrusted project-store write
  //   // flips it) stays named, not closed.
notes:
  - DefaultTrue == false reads via the PartialEq<bool> impl (default_true.rs:69); equivalently bool::from(project.keep_reviews_local). Use one consistently at the gate site.
  - The mirror GATE is the ONLY new mirror code: a conditional that the mirror path lives behind. The mirror verb body (create_forge_review/sync_reviews mapping) is NOT written — AC-4 asserts it is not invoked.
  - The skill contract ("on governed-project init, set keep_reviews_local = true if unset") is DOCUMENTED by LPR-010 — it is belt-and-suspenders since the field already defaults true. This task does NOT touch the skills.
  - keep_reviews_local persists through the project-settings surface (the forge_override path, the project store) — it is a per-project operator preference under R12 trusted-desktop, NOT administration:write-gated. Do NOT add enforce_administration_write_gate (AC-2 proves it persists via the normal project-settings path; R21 names the untrusted-write residual).
pattern: a per-project DefaultTrue setting mirroring ok_with_force_push (default local; old files deserialize local) + a project-settings write persisting via the project store (the forge_override path, NOT admin-gated) + a remote-mirror GATE conditional (keep_reviews_local == false) that makes the named-for-later mirror path unreachable while true
pattern_source: crates/gitbutler-project/src/project.rs:106 (ok_with_force_push: DefaultTrue + schemars attr); crates/gitbutler-project/src/default_true.rs (the type); crates/gitbutler-project/src/storage.rs (the project store the forge_override-class preference persists in)
anti_pattern: a plain bool field (defaults false = remote — the WRONG default; AC-1 catches an old-file deserialize to remote); ref-pinning keep_reviews_local as committed config (it is a project-store preference, not a gated decision); BUILDING the mirror path (create_forge_review/sync_reviews wiring — out of scope, AC-4 catches the mirror verb being invoked); administration:write-gating the keep_reviews_local setter (it is a trusted-desktop project preference, not a governed-config mutation — misrepresents a preference as an authorization boundary; R21 must stay named); copying force_push_protection's plain-bool shape

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agent: implementer=rust-implementer | reviewer=rust-reviewer
rationale: An additive DefaultTrue field on the legacy gitbutler_project::Project (the sanctioned home, beside forge_override) + default-local wiring + a remote-mirror GATE (not the mirror itself) + a project-settings write persisting via the project store (the forge_override path). The traps are subtle: using DefaultTrue (not plain bool) so the default and old-file deserialization are local; gating ONLY (the mirror path is specified-not-built); persisting via the project store (the forge_override path) — NOT administration:write-gating it; and proving via serde round-trip + a project-store round-trip test. rust-implementer writes it; rust-reviewer validates DefaultTrue (not bool), the default_with_id default, the project-store persistence path (NOT an admin gate), that the mirror is NOT built (only gated), and that keep_reviews_local never becomes ref-pinned config or reaches the merge gate.
coding_standards: crates/AGENTS.md (keep the field in the crate that owns the concept — gitbutler-project owns Project; solve the present problem directly — gate, don't build the mirror); RULES.md (gitbutler-project is legacy — preserve local ownership for a localized field add; but-api is THE API boundary for the setter; after changing but-sdk-exposed types run pnpm build:sdk && pnpm format — the regen is LPR-010); brain/docs/rust/ (ownership-borrowing.md From conversions DefaultTrue→bool; error-handling.md serde's existing Result for the project preference — NOT an admin gate)

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: LPR-003 (request_review — the verb whose default-local behavior this gates)
Blocks:     LPR-010 (the but-* skill contract doc relies on the default-true field; SDK regen for the new project-settings command)
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "LPR-006",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "lpr_project_files": {
      "description": "Two project JSON strings exercising the serde default: (a) an OLDER project file shape WITHOUT a keep_reviews_local key (proves #[serde(default)] + DefaultTrue supplies true = local); (b) a project file WITH keep_reviews_local=false (proves an explicit value round-trips). Deserialize each through the real gitbutler_project::Project serde path (NOT a hand-constructed struct) and round-trip through the real project store (storage.rs). No mocks.",
      "seed_method": "public_api",
      "records": [
        "let old_json = r#\"{ \"id\": ..., \"title\": \"p\", \"path\": ..., ... }\"#; // NO keep_reviews_local key (an older project file);",
        "let explicit_false_json = r#\"{ ..., \"keep_reviews_local\": false }\"#;",
        "deserialize both via serde_json::from_str::<Project>(...) and read keep_reviews_local; also call Project::default_with_id(id) and read keep_reviews_local."
      ]
    },
    "lpr_governed_repo": {
      "description": "A real governed repo via but_testsupport::writable_scenario + invoke_bash committing .gitbutler/permissions.toml to the target ref. Principals: `rev`/`rev2` agent + reviewer handles holding pull_requests:write (`admin`/`dev` are general principals present in the committed config). A real but_ctx::Context + DbHandle (the LPR-001 tables migrated) + a real Project in the project store with keep_reviews_local at its default (true). BUT_AGENT_HANDLE set per-case under #[serial_test::serial]. Seed via the real verbs — never direct row injection. The merge_gate/governed_loop hand-assertion idiom (real but-db + real gix, no mocks).",
      "seed_method": "public_api",
      "records": [
        "but_testsupport::writable_scenario(...) + invoke_bash committing .gitbutler/permissions.toml (rev: pull_requests:write; admin/dev present as general principals) to refs/heads/main;",
        "a real Project persisted in the project store with keep_reviews_local at its default (true);",
        "temp_env BUT_AGENT_HANDLE=rev under #[serial_test::serial]; drive request_review + publish_review through the but-api fns; capture the local_review_assignments rows and the forge_reviews cache before/after."
      ]
    },
    "lpr_project_store": {
      "description": "A real Project in the gitbutler-project project store (storage.rs) at its default keep_reviews_local == true. The keep_reviews_local preference is set through the project-settings surface (the SAME path that persists forge_override) and round-tripped through the store — NOT administration:write-gated (R12 trusted-desktop preference). No mocks.",
      "seed_method": "public_api",
      "records": [
        "a real Project persisted in the project store at keep_reviews_local default (true);",
        "set keep_reviews_local = false through the project-settings surface (the forge_override path); read it back from the store."
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN lpr_project_files (an old project JSON without keep_reviews_local + an explicit-false JSON) WHEN each deserializes into Project and keep_reviews_local is read / round-tripped through the project store THEN the no-field file yields keep_reviews_local == true (local, via DefaultTrue) and the explicit-false file yields false; default_with_id() yields true",
      "verify": "cargo test -p gitbutler-project keep_reviews_local_defaults_true_for_old_project_file",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real gitbutler-project Project serde (de)serialization + the real project store (storage.rs)",
        "negative_control": {
          "would_fail_if": [
            "keep_reviews_local were a plain bool — the no-field file would deserialize to FALSE (remote), failing the == true assertion",
            "the field lacked #[serde(default)] — the no-field file would fail to deserialize (a missing-field error) rather than defaulting to local",
            "default_with_id forgot to default it — a new project would be false/remote",
            "the explicit-false value did not round-trip through the store (the writer dropped it)"
          ]
        },
        "evidence": { "artifact_type": "file_artifact", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_project_files",
            "action": { "actor": "ci", "steps": [ "deserialize the no-field JSON into Project; read keep_reviews_local", "deserialize the explicit-false JSON; read keep_reviews_local", "Project::default_with_id(id); read keep_reviews_local", "round-trip the explicit-false project through the store; read it back" ] },
            "end_state": {
              "must_observe": [
                "the no-field project has keep_reviews_local == true (local)",
                "the explicit-false project has keep_reviews_local == false",
                "default_with_id() yields keep_reviews_local == true",
                "the explicit-false value round-trips through the project store unchanged"
              ],
              "must_not_observe": [
                "the no-field file deserializing to false/remote (a plain bool was used)",
                "a missing-field deserialize error (the #[serde(default)] is absent)",
                "default_with_id producing keep_reviews_local == false"
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
      "description": "GIVEN lpr_project_store (a real Project at keep_reviews_local default true) WHEN the desktop operator sets keep_reviews_local=false through the project-settings surface (the forge_override path) THEN the value persists to the project store (false read back) WITHOUT any administration:write gate — a per-project operator preference under R12 trusted-desktop; the R21 residual (untrusted project-store write flips it) stays named",
      "verify": "cargo test -p gitbutler-project keep_reviews_local_persists_via_project_settings",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "the real gitbutler-project project store (storage.rs) round-trip of keep_reviews_local via the project-settings surface (the forge_override path)",
        "negative_control": {
          "would_fail_if": [
            "the setter were administration:write-gated — it would deny a normal desktop project-settings write (misrepresenting a trusted-desktop preference as an authorization boundary)",
            "the value did not round-trip through the project store (the writer dropped it)",
            "keep_reviews_local were treated as ref-pinned committed config instead of a project-store preference"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_project_store",
            "action": { "actor": "ci", "steps": [ "load the real Project from the project store at keep_reviews_local default (true)", "set keep_reviews_local=false through the project-settings surface (the forge_override path)", "read keep_reviews_local back from the project store" ] },
            "end_state": {
              "must_observe": [
                "keep_reviews_local == false is read back from the project store (the value round-trips via the project-settings surface)",
                "the set persists WITHOUT any administration:write gate (no enforce_administration_write_gate / perm.denied on this preference)"
              ],
              "must_not_observe": [
                "an administration:write gate / perm.denied on the keep_reviews_local preference",
                "the value failing to round-trip through the project store (the writer dropped it)",
                "keep_reviews_local being routed through ref-pinned committed config instead of the project store"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN lpr_governed_repo with keep_reviews_local == true; BUT_AGENT_HANDLE=rev (pull_requests:write); forge_reviews captured WHEN `but review request refs/heads/feat --reviewer rev2` runs THEN a local review object is created (a local_review_assignments row present) AND NO remote forge PR is opened (no create_forge_review; forge_reviews cache byte-unchanged)",
      "verify": "cargo test -p but-api agent_review_defaults_local_no_remote_pr",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api request_review under keep_reviews_local=true + real but-db (assignment present; forge_reviews unchanged) + real gix",
        "negative_control": {
          "would_fail_if": [
            "request_review called create_forge_review while keep_reviews_local==true — a remote PR would open / forge_reviews would change",
            "the default-local gate were inverted (local meant remote) — a remote PR for the default case",
            "a stub created no local assignment (Ok with no write)"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_governed_repo",
            "action": { "actor": "agent", "steps": [ "ensure keep_reviews_local==true (default)", "BUT_AGENT_HANDLE=rev; run request_review(refs/heads/feat, rev2)", "read local_review_assignments + the forge_reviews cache" ] },
            "end_state": {
              "must_observe": [
                "a local_review_assignments row is present (the local review object exists)",
                "the forge_reviews cache is byte-identical to before (no create_forge_review)"
              ],
              "must_not_observe": [
                "a forge PR opened / forge_reviews cache changed while keep_reviews_local==true",
                "no local assignment row (a stub Ok)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN lpr_governed_repo with keep_reviews_local == true; an agent review opened WHEN the mirror path is inspected (the keep_reviews_local == false gate is false) THEN the remote-mirror path is unreachable — the named-for-later mirror verb is NOT invoked and the forge_reviews + sync_reviews (review.rs:1349) bridge is NOT exercised for the agent review",
      "verify": "cargo test -p but-api remote_mirror_gated_off_while_local",
      "scenario": {
        "tier": "holdout",
        "test_tier": "api-contract",
        "verification_service": "real but-api request_review path under keep_reviews_local=true; assert the mirror/create_forge_review/sync_reviews path is not reached",
        "negative_control": {
          "would_fail_if": [
            "the mirror path were actually BUILT and invoked while keep_reviews_local==true — sync_reviews/create_forge_review would run for the agent review",
            "the gate guarded the wrong branch (mirror reachable while local) — the bridge would be exercised"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_governed_repo",
            "action": { "actor": "agent", "steps": [ "keep_reviews_local==true; open an agent review via request_review", "assert the mirror verb / create_forge_review / sync_reviews path is not reached for this review (the keep_reviews_local==false gate short-circuits)" ] },
            "end_state": {
              "must_observe": [
                "the mirror verb is NOT invoked for the agent review",
                "sync_reviews / create_forge_review is NOT exercised for the agent review (the gate short-circuits before it)"
              ],
              "must_not_observe": [
                "create_forge_review or sync_reviews running for the agent review while keep_reviews_local==true",
                "the mirror path being reachable while the flag is true"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN lpr_governed_repo with keep_reviews_local == false WHEN the shipped publish_review (forge.rs:480) / open-PR path is driven THEN the remote-PR path behaves exactly as before v1.5.0 (the forge PR is opened via the existing verb) — the local default did not remove the forge path for opt-out projects",
      "verify": "cargo test -p but-api remote_pr_path_preserved_when_flag_false",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api publish_review path under keep_reviews_local=false + the shipped forge bridge (behavior-unchanged proof)",
        "negative_control": {
          "would_fail_if": [
            "setting keep_reviews_local=false did NOT re-enable the remote path — the forge PR would not open (v1.5.0 wrongly removed the forge path)",
            "the gate ignored the flag and stayed local even when false — no remote PR for an opt-out project"
          ]
        },
        "evidence": { "artifact_type": "api_response", "required_capture": true },
        "cases": [
          {
            "start_ref": "lpr_governed_repo",
            "action": { "actor": "agent", "steps": [ "set keep_reviews_local=false (admin)", "drive the shipped publish_review / open-PR path", "assert the forge PR opens via the existing verb, behaving as before v1.5.0" ] },
            "end_state": {
              "must_observe": [
                "the shipped publish_review/open-PR path opens the forge PR (behavior as before v1.5.0)",
                "the forge path is reachable when keep_reviews_local == false"
              ],
              "must_not_observe": [
                "the forge path being unreachable when the flag is false (v1.5.0 removed the forge path)",
                "the review staying local despite keep_reviews_local == false"
              ]
            }
          }
        ]
      }
    },
    { "id": "TC-1", "type": "test_criterion", "description": "a project JSON without keep_reviews_local deserializes to true (DefaultTrue); explicit false to false", "verify": "cargo test -p gitbutler-project keep_reviews_local_defaults_true_for_old_project_file", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "default_with_id() produces a Project with keep_reviews_local == true", "verify": "cargo test -p gitbutler-project keep_reviews_local_defaults_true_for_old_project_file", "maps_to_ac": "AC-1" },
    { "id": "TC-3", "type": "test_criterion", "description": "keep_reviews_local persists through the project-settings surface (the forge_override path) to the project store; NOT administration:write-gated", "verify": "cargo test -p gitbutler-project keep_reviews_local_persists_via_project_settings", "maps_to_ac": "AC-2" },
    { "id": "TC-4", "type": "test_criterion", "description": "request_review under keep_reviews_local=true creates a local assignment and leaves forge_reviews unchanged (no create_forge_review)", "verify": "cargo test -p but-api agent_review_defaults_local_no_remote_pr", "maps_to_ac": "AC-3" },
    { "id": "TC-5", "type": "test_criterion", "description": "under keep_reviews_local=true the mirror verb is not invoked and sync_reviews is not exercised for the agent review", "verify": "cargo test -p but-api remote_mirror_gated_off_while_local", "maps_to_ac": "AC-4" },
    { "id": "TC-6", "type": "test_criterion", "description": "under keep_reviews_local=false the shipped publish_review/open-PR path behaves as before v1.5.0", "verify": "cargo test -p but-api remote_pr_path_preserved_when_flag_false", "maps_to_ac": "AC-5" }
  ]
}
-->
