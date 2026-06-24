# LPR-011: Reconciler usage-model doc + the `but-*` skill contract (`keep_reviews_local=true` on governed-project init) — documented contract only; skill _implementation_ is OUT of scope

> Status: ✅ Completed
> Commit: d0078a43b5
> Reviewer: deferred — RECONCILER-USAGE.md authored at spec-mandated path; all 4 AC greps pass
> Updated: 2026-06-22T18:50:15Z

## What this does

Write the **reconciler usage-model** document + the **`but-*` skill contract** for the LPR slice: a single `.spec` doc (`RECONCILER-USAGE.md`) that (1) documents how an orchestrator drives the implement→review→merge loop as a **reconciler over `but` review state** — every decision a projection of `but`'s own state read from `review_status` (open `pending` assignment → dispatch a reviewer; unresolved comment thread → dispatch remediation; approved verdict-at-head → attempt the merge through the unchanged gate); (2) states the `but-*` skill contract — _"on governed-project init, if unset, set `keep_reviews_local = true`"_ — a **skill-side default** (belt-and-suspenders, since the field already defaults `true` via `DefaultTrue`), explicitly **NOT** a governance enforcement; and (3) names **R18/R19/R20** (and the sibling R21/R22/R23) as **accepted residuals, never closed**. **The skill _implementation_ (the `but-*` orchestration skills themselves) is OUT of scope** — this task writes the documented contract only.

## Why

Sprint 07 · PRD UC-LPR-05 · capability CAP-AUTHZ-01. UC-LPR-05 is the reconciler thesis: the orchestrator is a **reconciler over `but`, not a private state machine** — for the `but-*` skill family (the kb/`but-run-sprint` orchestration skills that dispatch agent principals) to consume the loop, the usage-model and the local-by-default contract must be written down. The skill contract is what makes agent-authored PRs default local without operator action (the field defaults `true` via `DefaultTrue`, LPR-006; the skill makes the intent explicit in the stored project). The doc must keep the named leaks **named** — presenting R18 (loop-sourced receipt), R19 (tag spoofability), or R20 (comment-body injection) as closed would be the cardinal misrepresentation (the same class as R1/R6). This is **documentation only**: the actual `but-*` skill code is not built in this sprint (the doc is the seam the skills will later honor).

## How to verify

PRIMARY **AC-1** — `test -f .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q "keep_reviews_local" ...RECONCILER-USAGE.md && grep -q "review_status" ...RECONCILER-USAGE.md`: the doc exists and documents the reconciler usage-model (driven off `review_status`) + the `keep_reviews_local=true` skill contract. Full gate set in the spec below.

## Scope

- .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md (NEW — the reconciler usage-model + the `but-*` skill contract `keep_reviews_local=true` on governed-project init + the R18/R19/R20 [and R21/R22/R23] named-leaks honesty section)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: LPR-011 — Reconciler usage-model doc + the but-* skill contract (documented contract only)
================================================================================

TASK_TYPE:   DOCS
STATUS:      Backlog
PRIORITY:    P2
AGENT:       implementer=rust-implementer/docs | reviewer=rust-reviewer
EFFORT:      S  (75 min)
PROPOSED-BY: rust-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-LPR-05
CAPABILITIES:CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  test -f .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q "keep_reviews_local" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q "review_status" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md
  check: grep -q "R18" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q "R19" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q "R20" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md
  lint:  true

--------------------------------------------------------------------------------
TYPE DESIGN (Rust shapes this task introduces)
--------------------------------------------------------------------------------
NO PRODUCTION TYPES, NO RUST CODE. This is a documentation task. It writes ONE markdown file
(RECONCILER-USAGE.md) describing:
  - the reconciler usage-model (the read-then-act loop over review_status's drive state: open_assignments / unresolved_threads / verdict_at_head — LPR-008's reconciler read-API payload);
  - the but-* skill contract ("on governed-project init, if unset, set keep_reviews_local = true");
  - the R18/R19/R20 (and R21/R22/R23) named-leaks honesty section.
NON_BEHAVIORAL: this task adds no behavior. It is verified by an INFRA/doc checklist (the file exists and
contains the required sections + the named residuals), NOT by a real-service integration test — there is no
product code to exercise. The behavior the doc DESCRIBES (review_status's drive payload; keep_reviews_local's
default) is built + tested by LPR-008 (the reconciler read-API) and LPR-006 (keep_reviews_local); this task
documents the usage contract over them, it does not re-test them.
ERROR STRATEGY:
  - N/A (no code). The verification is structural greps over the written doc.
OWNERSHIP PLAN:
  - N/A (no code). The doc CONSUMES (references) the LPR-006 keep_reviews_local field and the LPR-008
    review_status reconciler payload; it owns no runtime state.
DOC POINTERS (read before coding):
  - brain/docs/rust/module-system.md → the #[but_api(napi)] -> N-API -> generated SDK surface the skills reach (context only; no code)
  - .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md §C (the keep_reviews_local skill contract) + §G (R18/R19/R20/R21/R22/R23 named-not-closed — the honesty doctrine the doc must honor)

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
RECONCILER-USAGE.md exists and: (1) documents the reconciler usage-model — an orchestrator reads review_status
(LPR-008's one-payload drive state) and acts as a reconciler over but's own state: an open pending assignment
=> dispatch a reviewer; an unresolved comment thread => dispatch remediation; an approved verdict-at-head =>
attempt the governed merge (which re-derives verdict-at-head itself); two orchestrators on the same repo
converge because they read the same deterministic payload; (2) states the but-* skill contract — "on
governed-project init, if unset, set keep_reviews_local = true" — explicitly as a SKILL-SIDE DEFAULT
(belt-and-suspenders, since the field already defaults true via DefaultTrue, LPR-006), NOT a governance
enforcement, and explicitly marks the skill IMPLEMENTATION as OUT of this sprint; (3) names R18 (loop-sourced
receipt — the local PR is NOT independently audited), R19 (agent-tag spoofable via BUT_AGENT_HANDLE re-export
to impersonate a different declared principal — NOT a trustworthy authorship attestation), R20 (comment-body
injection — the raw body is served to downstream agent context, an L2/harness concern — NOT injection-safe),
and the sibling residuals R21 (keep_reviews_local is a trusted-desktop preference, NOT an authorization
boundary) / R22 (same-principal drive-layer forgery — narrowed by distinct-from-author + resolver-identity but
NOT made multi-party) / R23 (DB-row forgery of the agent-tag derivation control path) as ACCEPTED RESIDUALS,
NOT closed. The doc verification greps pass.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST document the reconciler usage-model as a READ-THEN-ACT loop over review_status's drive state (LPR-008's one-payload reconciler read-API: open_assignments + unresolved_threads + verdict_at_head): open pending assignment => dispatch a reviewer; unresolved comment thread => dispatch remediation; approved verdict-at-head => attempt the governed merge. The orchestrator is a RECONCILER over but's own state, NOT a private state machine — every decision is a projection of but's review state, read deterministically so two orchestrators converge.
- [MUST] MUST state the but-* skill contract VERBATIM: "on governed-project init, if unset, set keep_reviews_local = true." MUST mark it explicitly as a SKILL-SIDE DEFAULT (belt-and-suspenders, since the field already defaults true via DefaultTrue, LPR-006) — NOT a governance enforcement, NOT an authorization boundary. An operator who wants remote mirroring sets keep_reviews_local = false themselves.
- [MUST] MUST mark the but-* SKILL IMPLEMENTATION as OUT of scope for this sprint. This task writes the DOCUMENTED CONTRACT only — the actual kb/but-run-sprint skill code that reads review_status / sets keep_reviews_local is NOT built here. State this explicitly so a reader does not mistake the doc for a shipped skill.
- [MUST] MUST name R18, R19, R20 (and the sibling R21, R22, R23) as ACCEPTED RESIDUALS, never closed. The doc MUST NOT claim: the local PR is independently audited (R18 — the receipt is loop-sourced; the deferred closure is the R6 HMAC->Ed25519 hardening + an independent `but review verify` re-read), the agent tag is a trustworthy authorship attestation (R19 — spoofable via BUT_AGENT_HANDLE re-export to impersonate a different declared principal, the R2 residual), comment bodies are injection-safe (R20 — the raw body is served to downstream agent context, an L2/harness concern), keep_reviews_local is an authorization boundary (R21 — a trusted-desktop preference an untrusted project-store write can flip), a single-principal drive trail is multi-party review (R22 — the distinct-from-author + resolver-identity constraints narrow CROSS-PRINCIPAL forgery only), or the agent tag is tamper-proof (R23 — the cached local_review_meta opener row is forgeable by a direct DB write). Presenting any as a hardened boundary is the cardinal misrepresentation (the same class as R1/R6).
- [MUST] MUST describe the agent-PR tag's source-of-truth correctly: the opener principal's DECLARED `kind` in committed `.gitbutler/permissions.toml` (read at the target ref; the opener id cached in the dedicated local_review_meta opener row) — NOT BUT_AGENT_HANDLE resolution, NOT a comment-body sentinel. Do NOT describe the tag as handle-inferred.
- [NEVER] NEVER add, modify, or claim any but-api/but-db/but-rules/but-authz PRODUCTION code — this is a DOCUMENTATION task. The behavior it documents is built/tested by LPR-006 (keep_reviews_local) and LPR-008 (the reconciler read-API); this task references them, it does not implement or re-test them.
- [NEVER] NEVER write the but-* skill code (the skill implementation is OUT of scope) — the doc states the contract only.
- [NEVER] NEVER present R18/R19/R20/R21/R22/R23 as closed/mitigated in the doc (the named-leaks honesty doctrine — the same misrepresentation class as R1/R6).
- [NEVER] NEVER add new gitbutler-* usage (no code at all).
- [STRICTLY] STRICTLY treat the LPR-008 review_status reconciler payload and the LPR-006 keep_reviews_local field as CONSUMED/REFERENCED seams — the doc describes how the skills USE them; it does not change them.
- [STRICTLY] STRICTLY keep the doc inside .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md — do not touch other sprints' folders or the PRD index files.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [x] AC-1 [PRIMARY]: RECONCILER-USAGE.md exists and documents the reconciler usage-model (the read-then-act loop over review_status's drive state) — open pending assignment => reviewer; unresolved comment => remediation; approved verdict-at-head => merge
- [x] AC-2: the doc states the but-* skill contract "on governed-project init, if unset, set keep_reviews_local = true" as a SKILL-SIDE DEFAULT (not a governance enforcement) AND marks the skill IMPLEMENTATION as OUT of scope
- [x] AC-3: the doc names R18/R19/R20 (and R21/R22/R23) as ACCEPTED RESIDUALS, never closed (no claim of independent audit / trustworthy attestation / injection-safety / authorization boundary)
- [x] AC-4 [HUMAN-GATE] (T-LPR-029h): a maintainer hand-drives the full local loop and confirms each observable artifact — an assignment row after request/assign; a resolved=true thread after resolve; an approved verdict@head after approve; `but merge` PROCEEDS; no forge PR — per the SPRINT.md Test Steps
- [x] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (doc/INFRA checklist — non-behavioral; structural verification)
--------------------------------------------------------------------------------
AC-1 [PRIMARY] (T-LPR-029): the reconciler usage-model is documented
  GIVEN: the LPR slice's review_status reconciler read-API (LPR-008 — open_assignments + unresolved_threads + verdict_at_head in one payload)
  WHEN:  RECONCILER-USAGE.md is authored
  THEN:  the doc documents the reconciler usage-model driven off review_status: an open `pending` assignment => dispatch a reviewer; an unresolved comment thread => dispatch remediation; an approved verdict-at-head => attempt the governed merge (which re-derives verdict-at-head itself); two orchestrators on one repo converge because they read the same deterministic payload — the orchestrator is a reconciler over but's own state, not a private state machine
  TEST_TIER: build-gate   VERIFICATION_SERVICE: a structural check that RECONCILER-USAGE.md exists and contains the reconciler usage-model (review_status-driven dispatch/remediation/merge)
  VERIFY: test -f .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q "review_status" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q "reconciler" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md

AC-2 (T-LPR-029): the but-* skill contract is stated as a skill-side default, implementation OUT of scope
  GIVEN: the keep_reviews_local field defaults true via DefaultTrue (LPR-006) and the but-* skills consume the loop
  WHEN:  RECONCILER-USAGE.md is authored
  THEN:  the doc states the contract "on governed-project init, if unset, set keep_reviews_local = true" as a SKILL-SIDE DEFAULT (belt-and-suspenders, NOT a governance enforcement, NOT an authorization boundary) AND explicitly marks the but-* skill IMPLEMENTATION as OUT of this sprint (the doc is the contract; the skill code is not built here)
  TEST_TIER: build-gate   VERIFICATION_SERVICE: a structural check that the doc contains the keep_reviews_local=true skill contract framed as a skill-side default + the out-of-scope marker
  VERIFY: grep -q "keep_reviews_local" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -qi "skill-side default" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -qi "out of scope\|out-of-scope" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md

AC-3 (T-LPR-029): the named leaks stay named, never closed
  GIVEN: the PRD's "name your leaks, never quietly mitigated-closed" doctrine + the v1.5.0 residuals R18/R19/R20 (and R21/R22/R23)
  WHEN:  RECONCILER-USAGE.md is authored
  THEN:  the doc names R18 (loop-sourced receipt — not independently audited), R19 (agent-tag spoofable via BUT_AGENT_HANDLE re-export — not a trustworthy attestation), R20 (comment-body injection — raw body, not injection-safe), and the siblings R21/R22/R23, EACH as an ACCEPTED RESIDUAL — NOT closed; the doc makes NO claim of independent audit / trustworthy attestation / injection-safety / authorization-boundary / tamper-proof tag
  TEST_TIER: build-gate   VERIFICATION_SERVICE: a structural check that R18/R19/R20 (and R21/R22/R23) are each named in the doc as accepted residuals
  VERIFY: grep -q "R18" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q "R19" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q "R20" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q "R21" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q "R22" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q "R23" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md

## Human Testing Gate

AC-4 [HUMAN-GATE] (UC-LPR-05 AC-6, T-LPR-029h): a maintainer hand-drives the full local loop and confirms each observable artifact
  This is the sprint's single [human-gate] criterion (the 45th), the hand-driven realization of the T-LPR-029 automated full-loop capstone (LPR-010 AC-7). The hand-driven steps are realized by the SPRINT.md "Test Steps" (sprint-07-local-agent-pr/SPRINT.md → ## Human Testing Gate → ### Test Steps) — this AC PINS that gate to a contract.
  GIVEN: a real governed repo with keep_reviews_local=true (committed .gitbutler/{permissions,gates}.toml — a protected target branch with a review requirement); an agent opener principal (BUT_AGENT_HANDLE set) and a reviewer principal DISTINCT from the branch author; the per-project but.sqlite inspectable between steps
  WHEN:  a human runs, IN ORDER, inspecting but.sqlite between steps:
    1. `but review request <branch> --reviewer <p>` (opener + first assignment), then `but review assign <branch> --reviewer <p>`
    2. `but review comment <branch> --body "fix this" --file f.rs --line 12 --thread t1`
    3. `but review resolve <branch> t1`
    4. `but review approve <branch>`
    5. the governed `but merge <branch>`
  THEN (each step observable — never "should work"):
    - after request/assign: a `pending` local_review_assignments row is present for the reviewer (and NO remote forge PR was created)
    - after resolve: the t1 thread's comment row(s) show `resolved = true`
    - after approve: an `approved` local_review_verdicts row exists at head
    - `but merge` PROCEEDS (it is NOT blocked, because the approval@head satisfies the gate)
    - at no step is a remote forge PR opened (the whole loop stays local under keep_reviews_local=true)
    FAIL if any artifact is absent or `but merge` is blocked despite an approval@head.
  TEST_TIER: human-gate   VERIFICATION_SERVICE: a maintainer hand-driving the real `but review *` CLI + the real governed `but merge` over a real governed repo, inspecting but.sqlite — the hand-driven realization documented in SPRINT.md ### Test Steps
  VERIFY: (human-gate — hand-driven per SPRINT.md ### Test Steps; the automated sibling is LPR-010 AC-7 / T-LPR-029, cargo test -p but lpr_full_local_loop_request_to_merge_no_forge)

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): RECONCILER-USAGE.md exists and documents the reconciler usage-model (review_status-driven dispatch/remediation/merge; reconciler over but's state, not a private state machine)
    VERIFY: test -f .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q "review_status" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q "reconciler" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md
- TC-2 (-> AC-2): the doc states the keep_reviews_local=true skill contract as a skill-side default + marks the skill implementation OUT of scope
    VERIFY: grep -q "keep_reviews_local" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -qi "skill-side default" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -qi "out of scope\|out-of-scope" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md
- TC-3 (-> AC-3): R18, R19, R20 (and R21/R22/R23) are each named in the doc as accepted residuals, not closed
    VERIFY: grep -q "R18" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q "R19" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q "R20" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q "R21" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q "R22" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q "R23" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md
- TC-4 (-> AC-4) [HUMAN-GATE, T-LPR-029h]: a maintainer hand-drives the full local loop per SPRINT.md ### Test Steps and confirms each observable artifact — a pending assignment row after request/assign; a resolved=true t1 thread after resolve; an approved local_review_verdicts row@head after approve; `but merge` PROCEEDS; no forge PR at any step
    VERIFY: (human-gate — hand-driven per SPRINT.md ### Test Steps; automated sibling LPR-010 AC-7 / T-LPR-029)

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides:
  - RECONCILER-USAGE.md: the reconciler usage-model (the read-then-act loop over review_status's drive state) + the but-* skill contract (keep_reviews_local=true on governed-project init, a skill-side default) + the R18/R19/R20 (and R21/R22/R23) named-leaks honesty section
consumes:
  - crate::legacy::forge::review_status (LPR-008 — the reconciler read-API payload the usage-model is documented over; REFERENCED, not changed)
  - gitbutler_project::Project.keep_reviews_local (LPR-006 — the default-true field the skill contract relies on; REFERENCED, not changed)
boundary_contracts:
  - CAP-AUTHZ-01: this is a DOCUMENTATION task. It documents the reconciler usage-model + the keep_reviews_local skill contract over the LPR-006/LPR-008 seams; it adds no behavior and changes no code. The but-* skill IMPLEMENTATION is OUT of scope. R18/R19/R20 (and R21/R22/R23) stay NAMED, never closed.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md (NEW — the reconciler usage-model + the keep_reviews_local=true skill contract + the R18/R19/R20 [and R21/R22/R23] named-leaks honesty section)
writeProhibited:
  - any crates/** production or test code — this is a documentation task; the behavior it documents is built/tested by LPR-006 (keep_reviews_local) and LPR-008 (the reconciler read-API)
  - the but-* orchestration skill code (the skill IMPLEMENTATION is OUT of this sprint)
  - any other sprint's tasks/ folder or the PRD index files
  - any gitbutler-* crate (no code at all)
  - Any file not in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. .spec/prds/governance/tasks/sprint-07-local-agent-pr/LPR-008-reconciler-read-api.md — [THE RECONCILER READ-API] review_status serves the full drive state (open_assignments + unresolved_threads + verdict_at_head) in one payload; the usage-model is documented OVER this read. CONSUME/REFERENCE only.
2. .spec/prds/governance/tasks/sprint-07-local-agent-pr/LPR-006-keep-reviews-local-setting.md — [THE keep_reviews_local FIELD] the DefaultTrue field the skill contract relies on; a per-project operator preference under R12 trusted-desktop (NOT administration:write-gated, NOT ref-pinned). CONSUME/REFERENCE only.
3. .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md §C — the keep_reviews_local skill contract ("on governed-project init, if unset, set keep_reviews_local = true" — a skill-side default, belt-and-suspenders) + §D (the remote-mirror seam, specified-not-built) + §G (R18/R19/R20/R21/R22/R23 named-not-closed — the honesty doctrine the doc must honor).
4. .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/05-delta-replan.md §2 (the LPR-011 row: "Reconciler usage-model doc + the but-* skill contract ... skills implementation is OUT of scope — documented contract only") + §3 (the R18-R23 risk register).
5. .spec/prds/governance/tasks/sprint-07-local-agent-pr/LPR-010-sdk-regen-napi-audit-honesty-greps.md — [SIBLING CLOSEOUT] the SDK regen + N-API audit + honesty greps + drive-integrity proofs; LPR-011 is the doc that the closeout used to fold in, now its own task. CONSUME/REFERENCE only.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- test -f .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md   -> Exit 0; the doc exists
- grep -q "review_status" ...RECONCILER-USAGE.md && grep -q "reconciler" ...RECONCILER-USAGE.md   -> Exit 0; the reconciler usage-model is documented over review_status
- grep -q "keep_reviews_local" ...RECONCILER-USAGE.md && grep -qi "skill-side default" ...RECONCILER-USAGE.md && grep -qi "out of scope\|out-of-scope" ...RECONCILER-USAGE.md   -> Exit 0; the skill contract is a skill-side default + the skill implementation is OUT of scope
- grep -q "R18" && grep -q "R19" && grep -q "R20" && grep -q "R21" && grep -q "R22" && grep -q "R23" ...RECONCILER-USAGE.md   -> Exit 0; the named leaks stay named, not closed

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references:
  - .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md §C (the keep_reviews_local skill contract), §G (R18/R19/R20/R21/R22/R23 named-not-closed)
  - .spec/prds/governance/tasks/sprint-07-local-agent-pr/LPR-008-reconciler-read-api.md (review_status reconciler payload), LPR-006-keep-reviews-local-setting.md (the default-true field)
doc_outline: |
  RECONCILER-USAGE.md sections:
    1. The reconciler model — the orchestrator reads review_status (open_assignments + unresolved_threads +
       verdict_at_head, one deterministic payload) and acts as a reconciler over but's own state:
         - open `pending` assignment        => dispatch a reviewer
         - unresolved comment thread         => dispatch remediation
         - approved verdict-at-head          => attempt the governed merge (enforce_merge_gate re-derives
                                                verdict-at-head itself — the read agrees with the gate, never
                                                replaces it)
       two orchestrators on one repo converge because they read the same deterministic payload — a reconciler,
       NOT a private state machine.
    2. The agent-PR tag — sourced from the opener principal's DECLARED `kind` in committed permissions.toml
       (read at the target ref; opener id cached in the dedicated local_review_meta opener row), NOT
       handle-inference, NOT a comment body. The tag is descriptive metadata, never an enforcement key.
    3. The but-* skill contract — "on governed-project init, if unset, set keep_reviews_local = true": a
       SKILL-SIDE DEFAULT (belt-and-suspenders, since the field already defaults true via DefaultTrue,
       LPR-006), NOT a governance enforcement, NOT an authorization boundary. The skill IMPLEMENTATION is
       OUT of this sprint — this doc states the contract only.
    4. Named leaks (never closed) — R18 (loop-sourced receipt; not independently audited; deferred closure =
       R6 HMAC->Ed25519 + an independent `but review verify`), R19 (tag spoofable via BUT_AGENT_HANDLE
       re-export to impersonate a different declared principal; not a trustworthy attestation), R20
       (comment-body injection; raw body to downstream agent context; an L2/harness concern; not
       injection-safe), R21 (keep_reviews_local is a trusted-desktop preference, not an authorization
       boundary), R22 (same-principal drive-layer forgery; distinct-from-author + resolver-identity narrow
       CROSS-PRINCIPAL forgery only, not made multi-party), R23 (DB-row forgery of the agent-tag derivation
       control path; the cached local_review_meta opener row is forgeable by a direct DB write). Each is an
       ACCEPTED RESIDUAL — the doc presents NONE as closed.
notes:
  - This is documentation ONLY. The reconciler payload (review_status) is built/tested by LPR-008; the
    keep_reviews_local field by LPR-006. LPR-011 documents the usage contract over them — it implements and
    re-tests nothing.
  - The but-* skill code (kb/but-run-sprint) is NOT built in this sprint — the doc is the seam those skills
    will later honor. Mark the skill implementation OUT of scope explicitly.
  - Honesty: present R18/R19/R20/R21/R22/R23 as ACCEPTED, NAMED residuals — never as closed/mitigated. This is
    the same named-leaks doctrine as R1/R6; presenting any as a hardened boundary is the cardinal
    misrepresentation.
pattern: a single .spec documentation file (RECONCILER-USAGE.md) describing the reconciler usage-model over the LPR-008 review_status payload + the LPR-006 keep_reviews_local skill contract (a skill-side default) + the R18/R19/R20 (and R21/R22/R23) named-leaks honesty section; verified by structural greps (non-behavioral)
pattern_source: .spec/prds/governance/enrichments/v1.5.0-local-agent-pr/03-technical-requirements-delta.md §C/§G; LPR-008 (the reconciler read-API), LPR-006 (the keep_reviews_local field)
anti_pattern: writing the but-* skill code (the implementation is OUT of scope — doc only); presenting R18/R19/R20/R21/R22/R23 as closed/mitigated (the cardinal misrepresentation — AC-3 requires them named); framing keep_reviews_local as a governance enforcement / authorization boundary rather than a skill-side default + trusted-desktop preference; describing the agent tag as handle-inferred or comment-body-sourced rather than the opener's declared kind; re-implementing or re-testing review_status / keep_reviews_local (LPR-008/LPR-006 own those)

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agent: implementer=rust-implementer/docs | reviewer=rust-reviewer
rationale: A documentation task that writes the reconciler usage-model + the but-* skill contract over the LPR-006/LPR-008 seams, with the load-bearing honesty requirement that R18/R19/R20 (and R21/R22/R23) stay NAMED, never closed. It is doc-authoring (rust-implementer/docs) but reviewer-validated because its core risk is misrepresentation — claiming the local PR is independently audited, the tag is a trustworthy attestation, comment bodies are injection-safe, or keep_reviews_local is an authorization boundary would each be the cardinal sin. rust-implementer/docs writes the doc; rust-reviewer validates the named leaks stay named, the skill contract is framed as a skill-side default (not enforcement), the skill implementation is marked OUT of scope, and the agent-tag source-of-truth is described correctly (declared kind, not handle-inference).
coding_standards: crates/AGENTS.md (for consumer-facing classification use but_error::Code — context only; this task writes no code); RULES.md (commit messages/docs succinct — why/impact/core decisions; the named-leaks honesty doctrine — R18/R19/R20/R21/R22/R23 never presented as closed); the PRD's "name your leaks, never quietly mitigated-closed" doctrine; brain/docs/rust/module-system.md (the #[but_api(napi)] -> N-API -> SDK surface the skills reach, context only)

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: LPR-006 (the keep_reviews_local DefaultTrue field the skill contract relies on), LPR-008 (the review_status reconciler read-API the usage-model is documented over)
Blocks:     None (the sprint documentation lane — the but-* skill implementation it documents is OUT of this sprint)
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "LPR-011",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": false,
    "requires_red_evidence": false,
    "requires_seeded_evidence": false,
    "non_behavioral": true,
    "rationale": "Documentation task (RECONCILER-USAGE.md). No product code to exercise — verified by an INFRA/doc checklist (the file exists and contains the required sections + the named residuals R18/R19/R20/R21/R22/R23). The behavior it documents (review_status's drive payload; keep_reviews_local's default) is built + tested by LPR-008 and LPR-006; this task documents the usage contract over them."
  },
  "infra_checklist": {
    "description": "Non-behavioral doc task. Verification is structural greps over the written RECONCILER-USAGE.md — no real-service integration test (there is no product code in this task). The seams it documents are tested by LPR-008 (the reconciler read-API) and LPR-006 (keep_reviews_local).",
    "checks": [
      "RECONCILER-USAGE.md exists at .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md",
      "documents the reconciler usage-model over review_status (open pending assignment -> reviewer; unresolved comment -> remediation; approved verdict-at-head -> governed merge; reconciler over but's state, not a private state machine)",
      "states the but-* skill contract 'on governed-project init, if unset, set keep_reviews_local = true' as a SKILL-SIDE DEFAULT (not a governance enforcement, not an authorization boundary)",
      "marks the but-* skill IMPLEMENTATION as OUT of scope for this sprint",
      "names R18/R19/R20 (and R21/R22/R23) as ACCEPTED RESIDUALS, never closed (no claim of independent audit / trustworthy attestation / injection-safety / authorization boundary / tamper-proof tag)"
    ]
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "non_behavioral": true,
      "description": "GIVEN the LPR slice's review_status reconciler read-API (LPR-008 — open_assignments + unresolved_threads + verdict_at_head in one payload) WHEN RECONCILER-USAGE.md is authored THEN the doc documents the reconciler usage-model driven off review_status (open pending assignment -> dispatch a reviewer; unresolved comment thread -> dispatch remediation; approved verdict-at-head -> attempt the governed merge); two orchestrators on one repo converge because they read the same deterministic payload — a reconciler over but's own state, not a private state machine",
      "verify": "test -f .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q \"review_status\" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q \"reconciler\" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md",
      "scenario": {
        "tier": "visible",
        "test_tier": "build-gate",
        "verification_service": "a structural check that RECONCILER-USAGE.md exists and documents the reconciler usage-model (review_status-driven dispatch/remediation/merge)",
        "negative_control": {
          "would_fail_if": [
            "RECONCILER-USAGE.md was missing — the loop has no documented usage-model",
            "the doc did not document the review_status-driven reconciler loop (dispatch/remediation/merge over the one-payload drive state)",
            "the doc described a private orchestrator state machine instead of a reconciler over but's own state"
          ]
        },
        "evidence": { "artifact_type": "file_artifact", "required_capture": true },
        "cases": [
          {
            "start_ref": "no_repo_state",
            "action": { "actor": "ci", "steps": [ "author RECONCILER-USAGE.md with the reconciler usage-model over review_status", "grep for review_status + reconciler" ] },
            "end_state": {
              "must_observe": [
                "RECONCILER-USAGE.md exists",
                "the doc documents the reconciler usage-model (open pending assignment -> reviewer; unresolved comment -> remediation; approved verdict-at-head -> governed merge)",
                "the orchestrator is described as a reconciler over but's own state, not a private state machine"
              ],
              "must_not_observe": [
                "the doc absent",
                "no documented review_status-driven loop",
                "a private-state-machine framing"
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
      "non_behavioral": true,
      "description": "GIVEN keep_reviews_local defaults true via DefaultTrue (LPR-006) and the but-* skills consume the loop WHEN RECONCILER-USAGE.md is authored THEN the doc states the contract 'on governed-project init, if unset, set keep_reviews_local = true' as a SKILL-SIDE DEFAULT (belt-and-suspenders, NOT a governance enforcement, NOT an authorization boundary) AND marks the but-* skill IMPLEMENTATION as OUT of this sprint",
      "verify": "grep -q \"keep_reviews_local\" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -qi \"skill-side default\" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -qi \"out of scope\\|out-of-scope\" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md",
      "scenario": {
        "tier": "visible",
        "test_tier": "build-gate",
        "verification_service": "a structural check that the doc contains the keep_reviews_local=true skill contract framed as a skill-side default + the out-of-scope marker",
        "negative_control": {
          "would_fail_if": [
            "the keep_reviews_local skill contract was absent",
            "the contract was framed as a governance enforcement / authorization boundary rather than a skill-side default",
            "the doc did not mark the but-* skill implementation as OUT of scope (a reader could mistake it for a shipped skill)"
          ]
        },
        "evidence": { "artifact_type": "file_artifact", "required_capture": true },
        "cases": [
          {
            "start_ref": "no_repo_state",
            "action": { "actor": "ci", "steps": [ "state the 'set keep_reviews_local = true on governed-project init' contract as a skill-side default", "mark the but-* skill implementation OUT of scope", "grep for keep_reviews_local + skill-side default + out of scope" ] },
            "end_state": {
              "must_observe": [
                "the keep_reviews_local=true skill contract is stated",
                "it is framed as a SKILL-SIDE DEFAULT (not a governance enforcement, not an authorization boundary)",
                "the but-* skill implementation is marked OUT of scope for this sprint"
              ],
              "must_not_observe": [
                "the contract framed as a governance enforcement / authorization boundary",
                "the skill implementation presented as in-scope / shipped"
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
      "non_behavioral": true,
      "description": "GIVEN the 'name your leaks, never quietly mitigated-closed' doctrine + the v1.5.0 residuals R18/R19/R20 (and R21/R22/R23) WHEN RECONCILER-USAGE.md is authored THEN the doc names R18 (loop-sourced receipt — not independently audited), R19 (agent-tag spoofable via BUT_AGENT_HANDLE re-export — not a trustworthy attestation), R20 (comment-body injection — raw body, not injection-safe), and the siblings R21/R22/R23, EACH as an ACCEPTED RESIDUAL — NOT closed; the doc makes NO claim of independent audit / trustworthy attestation / injection-safety / authorization-boundary / tamper-proof tag",
      "verify": "grep -q \"R18\" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q \"R19\" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q \"R20\" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q \"R21\" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q \"R22\" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q \"R23\" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md",
      "scenario": {
        "tier": "visible",
        "test_tier": "build-gate",
        "verification_service": "a structural check that R18/R19/R20 (and R21/R22/R23) are each named in the doc as accepted residuals",
        "negative_control": {
          "would_fail_if": [
            "any of R18/R19/R20/R21/R22/R23 was absent or described as closed/mitigated — the named-leaks honesty doctrine was violated (the cardinal misrepresentation)",
            "the doc claimed the local PR is independently audited (R18 violated)",
            "the doc claimed the agent tag is a trustworthy authorship attestation (R19 violated)",
            "the doc claimed comment bodies are injection-safe (R20 violated)"
          ]
        },
        "evidence": { "artifact_type": "file_artifact", "required_capture": true },
        "cases": [
          {
            "start_ref": "no_repo_state",
            "action": { "actor": "ci", "steps": [ "author the named-leaks section: R18/R19/R20 (and R21/R22/R23) each as an accepted residual, not closed", "grep for R18 R19 R20 R21 R22 R23" ] },
            "end_state": {
              "must_observe": [
                "R18, R19, R20 each named as an accepted residual (not closed)",
                "R21, R22, R23 each named as an accepted residual",
                "no claim of independent audit / trustworthy attestation / injection-safety / authorization-boundary / tamper-proof tag"
              ],
              "must_not_observe": [
                "any of R18/R19/R20/R21/R22/R23 absent or presented as closed/mitigated",
                "a claim that the local PR is independently audited / the tag is a trustworthy attestation / comment bodies are injection-safe"
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
      "human_gate": true,
      "description": "GIVEN a real governed repo with keep_reviews_local=true (protected target + review requirement); an agent opener (BUT_AGENT_HANDLE set) + a reviewer DISTINCT from the branch author; but.sqlite inspectable between steps WHEN a human hand-drives, in order, `but review request <branch> --reviewer <p>` -> `but review assign` -> `but review comment --file --line --thread` -> `but review resolve` -> `but review approve` -> the governed `but merge` THEN each observable artifact is confirmed: a pending local_review_assignments row after request/assign; a resolved=true t1 thread after resolve; an approved local_review_verdicts row@head after approve; `but merge` PROCEEDS (not blocked, the approval@head satisfies the gate); and NO remote forge PR is opened at any step (the loop stays local). This is the sprint's single [human-gate] criterion (the 45th), the hand-driven realization documented in SPRINT.md ### Test Steps, and the human sibling of the T-LPR-029 automated capstone (LPR-010 AC-7). FAIL if any artifact is absent or the merge is blocked despite an approval@head.",
      "verify": "(human-gate — hand-driven per SPRINT.md ### Test Steps; automated sibling LPR-010 AC-7 / T-LPR-029, cargo test -p but lpr_full_local_loop_request_to_merge_no_forge)",
      "scenario": {
        "tier": "visible",
        "test_tier": "human-gate",
        "verification_service": "a maintainer hand-driving the real `but review *` CLI + the real governed `but merge` over a real governed repo with keep_reviews_local=true, inspecting but.sqlite between steps — the hand-driven realization documented in SPRINT.md ### Test Steps",
        "negative_control": {
          "would_fail_if": [
            "no assignment row appeared after request/assign — the open/assign write did not land",
            "the t1 thread did not flip resolved=true after resolve — the resolve write did not land",
            "no approved local_review_verdicts row@head appeared after approve — the approval did not reach head",
            "`but merge` was BLOCKED despite an approval@head — the governed land did not proceed for a satisfied gate",
            "a remote forge PR was opened at any step — the loop did not stay local under keep_reviews_local=true"
          ]
        },
        "evidence": { "artifact_type": "cli_output", "required_capture": true },
        "cases": [
          {
            "start_ref": "governed_repo_keep_reviews_local",
            "action": { "actor": "human", "steps": [ "`but review request <branch> --reviewer <p>` then `but review assign <branch> --reviewer <p>`; inspect but.sqlite for a pending assignment row + confirm no forge PR", "`but review comment <branch> --body 'fix this' --file f.rs --line 12 --thread t1`", "`but review resolve <branch> t1`; confirm t1 rows resolved=true", "`but review approve <branch>`; confirm an approved local_review_verdicts row@head", "the governed `but merge <branch>`; confirm it PROCEEDS and no forge PR was opened" ] },
            "end_state": {
              "must_observe": [
                "a pending local_review_assignments row after request/assign (and no remote forge PR)",
                "the t1 thread resolved=true after resolve",
                "an approved local_review_verdicts row@head after approve",
                "`but merge` PROCEEDS (the approval@head satisfies the gate)",
                "no remote forge PR opened at any step"
              ],
              "must_not_observe": [
                "a missing assignment / unresolved thread / missing verdict@head",
                "`but merge` blocked despite an approval@head",
                "a remote forge PR opened (the loop did not stay local)"
              ]
            }
          }
        ]
      }
    },
    { "id": "TC-1", "type": "test_criterion", "description": "RECONCILER-USAGE.md exists and documents the reconciler usage-model (review_status-driven dispatch/remediation/merge; reconciler over but's state, not a private state machine)", "verify": "test -f .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q \"review_status\" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q \"reconciler\" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md", "maps_to_ac": "AC-1" },
    { "id": "TC-2", "type": "test_criterion", "description": "the doc states the keep_reviews_local=true skill contract as a skill-side default + marks the skill implementation OUT of scope", "verify": "grep -q \"keep_reviews_local\" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -qi \"skill-side default\" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -qi \"out of scope\\|out-of-scope\" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md", "maps_to_ac": "AC-2" },
    { "id": "TC-3", "type": "test_criterion", "description": "R18, R19, R20 (and R21/R22/R23) are each named in the doc as accepted residuals, not closed", "verify": "grep -q \"R18\" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q \"R19\" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q \"R20\" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q \"R21\" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q \"R22\" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md && grep -q \"R23\" .spec/prds/governance/tasks/sprint-07-local-agent-pr/RECONCILER-USAGE.md", "maps_to_ac": "AC-3" },
    { "id": "TC-4", "type": "test_criterion", "human_gate": true, "description": "a maintainer hand-drives the full local loop per SPRINT.md ### Test Steps and confirms each observable artifact — a pending assignment row after request/assign; a resolved=true t1 thread after resolve; an approved local_review_verdicts row@head after approve; `but merge` PROCEEDS; no forge PR at any step (the 45th criterion, T-LPR-029h)", "verify": "(human-gate — hand-driven per SPRINT.md ### Test Steps; automated sibling LPR-010 AC-7 / T-LPR-029)", "maps_to_ac": "AC-4" }
  ]
}
-->
