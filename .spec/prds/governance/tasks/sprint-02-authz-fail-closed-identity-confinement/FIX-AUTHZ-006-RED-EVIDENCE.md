# FIX-AUTHZ-006-RED-EVIDENCE: produce + commit the missing `.tmp/AUTHZ-006/` RED-against-start + seeded verification evidence per the AUTHZ-006 REQUIREMENT-CONTRACT

## What this does

Closes a verification-policy gap (red-hat M-2). AUTHZ-006's REQUIREMENT-CONTRACT sets `requires_red_evidence:true` + `requires_seeded_evidence:true` (AUTHZ-006-administration-write-guard.md, the trailing JSON `verification_policy`), but `.tmp/AUTHZ-006/` does not exist — evidence dirs were committed for AUTHZ-001/002/003/004/005/007/008 but not 006 (commit `fbb6ffe32b` committed only source + tests). The admin-write guard behavior (`crates/but-api/src/legacy/config_mutate.rs::enforce_administration_write_gate`, L18-28) is real and mutation-verified, but the required RED-against-start capture and seeded-artifact outputs were never produced. This task generates and commits a genuine `.tmp/AUTHZ-006/` artifact package mirroring the shape of `.tmp/AUTHZ-004/`: a real RED-against-start (the guard body temporarily emptied to `Ok(())` so `cargo test -p but-api admin_write_guard_denies_non_admin_allows_admin` FAILS on the dev-denied half), the GREEN output after restoring the real guard, the seeded-artifact outputs, and a `verify-manifest.json` mirroring the AUTHZ-004 manifest schema. This is an EVIDENCE task: its verification is the presence + integrity of the artifacts, NOT a new cargo test.

## Why

Sprint 02 · PRD UC-AUTHZ-03 · capability CAP-AUTHZ-01. The repo's verification policy (no stubbed/fake-success evidence; RED must be captured against the pre-implementation start, not fabricated) requires that a task declaring `requires_red_evidence` + `requires_seeded_evidence` actually ship those artifacts. AUTHZ-006 shipped the behavior but not the evidence, so its cross-cutting verdict was PARTIAL. A fabricated "RED" that passes with the real implementation in place would be the cardinal sin this policy exists to prevent — the RED here MUST be captured against a genuinely emptied guard body so it demonstrably fails.

## How to verify

PRIMARY **AC-1** — `.tmp/AUTHZ-006/` exists and contains a GENUINE RED-against-start capture (the guard emptied → the dev-denied/admin-Ok test fails), a GREEN capture (real guard restored → the test passes), seeded-artifact outputs, and a `verify-manifest.json` whose schema mirrors `.tmp/AUTHZ-004/verify-manifest.json`. The RED output must show the test FAILING — a "RED" that passes proves it was captured against the real implementation (fabricated) and FAILS this AC. Full gate set in the spec below.

## Scope

- `.tmp/AUTHZ-006/` (NEW directory) — the evidence package mirroring `.tmp/AUTHZ-004/`: `AC-1-red-against-start.txt` (genuine failing run with the guard body emptied), `AC-1-green.txt` (passing run with the real guard), `AC-1-seeded-artifact.txt`, `AC-3-red-against-start.txt` / `AC-3-green.txt` / `AC-3-seeded-artifact.txt` (the malformed-config gate), `red-output.txt`, `red-evidence-note.md` explaining how the RED was captured, and `verify-manifest.json` mirroring the AUTHZ-004 schema. OWNS producing + committing the AUTHZ-006 evidence package.

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: FIX-AUTHZ-006-RED-EVIDENCE - produce + commit the missing .tmp/AUTHZ-006/ RED-against-start + seeded verification evidence per the AUTHZ-006 REQUIREMENT-CONTRACT
================================================================================

TASK_TYPE:  EVIDENCE
STATUS:     Backlog
PRIORITY:   P1
EFFORT:     S  (60 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ./SPRINT.md
PRD_REFS:   UC-AUTHZ-03
CAPABILITIES: CAP-AUTHZ-01

RUNTIME_COMMANDS:
  evidence_red:   (temporarily empty the guard body) cargo test -p but-api admin_write_guard_denies_non_admin_allows_admin   2>&1 | tee .tmp/AUTHZ-006/AC-1-red-against-start.txt
  evidence_green: (restore the guard) cargo test -p but-api admin_write_guard   2>&1 | tee .tmp/AUTHZ-006/AC-1-green.txt
  integrity:      test -f .tmp/AUTHZ-006/verify-manifest.json && test -f .tmp/AUTHZ-006/AC-1-red-against-start.txt && test -f .tmp/AUTHZ-006/AC-1-green.txt
  NOTE: This is an EVIDENCE task. There is NO new cargo test; verification is the presence + integrity of the .tmp/AUTHZ-006/ artifacts. The cargo runs above are how the RED/GREEN captures are PRODUCED, not a new acceptance test.

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
.tmp/AUTHZ-006/ exists and is committed, mirroring the artifact shape of .tmp/AUTHZ-004/ (verify-manifest.json + per-AC red-against-start/green/seeded-artifact captures + red-output.txt + red-evidence-note.md). The RED-against-start capture is GENUINE: it was produced by temporarily replacing the enforce_administration_write_gate body (crates/but-api/src/legacy/config_mutate.rs:18-28) with a no-op that returns Ok(()) BEFORE running cargo test -p but-api admin_write_guard_denies_non_admin_allows_admin, so the test FAILS on the dev-denied half (the no-op guard permits dev, who holds only contents:write) — proving the test would have caught a missing guard. The GREEN capture is the same test passing against the RESTORED real guard. The seeded-artifact captures mirror the AUTHZ-004 seeded shape. verify-manifest.json mirrors .tmp/AUTHZ-004/verify-manifest.json: task_id="AUTHZ-006", project_commands, verification_policy {requires_tests:true, requires_red_evidence:true, requires_seeded_evidence:true}, and a requirements[] block of {id, verify, description} for AC-1/AC-2/AC-3/TC-1/TC-2/TC-3 lifted from the AUTHZ-006 contract. The real config_mutate.rs guard is UNCHANGED in the final commit (the emptied body was reverted after the RED capture); the only persisted source change is the new .tmp/AUTHZ-006/ evidence files.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST capture a GENUINE RED-against-start: temporarily replace the body of enforce_administration_write_gate (config_mutate.rs:18-28) with a no-op `Ok(())` (drop the load_governance_config + resolve_principal_from_env + but_authz::authorize calls), run cargo test -p but-api admin_write_guard_denies_non_admin_allows_admin, and SAVE the FAILING output to .tmp/AUTHZ-006/AC-1-red-against-start.txt. The dev-denied half MUST fail (the no-op permits dev). Then REVERT the guard body (git checkout config_mutate.rs) and run the test again for the GREEN capture. A "RED" that PASSES is FABRICATED and is an automatic FAIL — it means the capture was taken against the real implementation, not an emptied guard.
- [MUST] MUST mirror the .tmp/AUTHZ-004/ artifact shape EXACTLY: inspect `ls .tmp/AUTHZ-004/` and reproduce the file set for AUTHZ-006 — per-AC `AC-N-red-against-start.txt`, `AC-N-green.txt`, `AC-N-seeded-artifact.txt` for the AUTHZ-006 ACs that have a runtime test (AC-1 dev/admin/working-tree, AC-3 malformed-config; AC-2 is a build-gate grep), plus `red-output.txt`, `red-evidence-note.md`, and `verify-manifest.json`.
- [MUST] MUST produce verify-manifest.json mirroring .tmp/AUTHZ-004/verify-manifest.json's schema: top-level task_id ("AUTHZ-006"), project_commands {test:"cargo test -p but-api admin_write_guard", typecheck:"cargo check -p but-api --all-targets", lint:"cargo clippy -p but-api --all-targets"}, verification_policy {requires_tests:true, requires_red_evidence:true, requires_seeded_evidence:true}, and requirements[] = the AUTHZ-006 AC-1/AC-2/AC-3 + TC-1/TC-2/TC-3 {id, verify, description} lifted verbatim-in-spirit from the AUTHZ-006 REQUIREMENT-CONTRACT (the verify commands are admin_write_guard_denies_non_admin_allows_admin / the grep / admin_write_guard_malformed_config_invalid).
- [MUST] MUST write red-evidence-note.md explaining precisely HOW the RED was captured (which guard body was emptied, which test failed, on which half/assertion) — mirror the prose of .tmp/AUTHZ-004/red-evidence-note.md so a reviewer can confirm the RED is genuine and reproducible.
- [MUST] MUST leave the real config_mutate.rs guard UNCHANGED in the final committed state: the emptied-body mutation is a transient capture device, reverted before commit. The reviewer confirms `git diff` touches ONLY .tmp/AUTHZ-006/ (no source change to config_mutate.rs).
- [NEVER] NEVER fabricate a RED by hand-writing a failing-looking log, by asserting against a different test, or by capturing the real guard's pass and labeling it RED. The RED file MUST be the actual stdout of the test FAILING against the emptied guard, showing the FAILED line for admin_write_guard_denies_non_admin_allows_admin.
- [NEVER] NEVER weaken or delete the AUTHZ-006 tests or the guard to make capture easier. The capture device is ONLY the transient no-op guard body, reverted immediately.
- [STRICTLY] STRICTLY scope to producing + committing .tmp/AUTHZ-006/. Do NOT modify the AUTHZ-006 source/tests, do NOT touch other .tmp dirs, do NOT change the forge tasks' scope.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: .tmp/AUTHZ-006/ exists with a GENUINE RED-against-start (the emptied guard → admin_write_guard_denies_non_admin_allows_admin FAILS on the dev-denied half), a GREEN capture (real guard restored → passes), and verify-manifest.json mirroring the AUTHZ-004 schema
- [ ] AC-2: seeded-artifact captures + red-evidence-note.md present, mirroring the .tmp/AUTHZ-004/ file set; the note documents how the RED was captured
- [ ] AC-3: the final commit's diff touches ONLY .tmp/AUTHZ-006/ — config_mutate.rs is unchanged (the emptied body was reverted)
- [ ] All verification gates pass; only write_allowed paths modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — evidence-integrity first)
--------------------------------------------------------------------------------

AC-1: .tmp/AUTHZ-006/ exists with a genuine RED-against-start, a GREEN capture, and a schema-mirroring verify-manifest.json [PRIMARY]
  GIVEN: AUTHZ-006 shipped its guard + tests but no .tmp/AUTHZ-006/ evidence; the guard is enforce_administration_write_gate (config_mutate.rs:18-28); the runtime test is admin_write_guard_denies_non_admin_allows_admin
  WHEN:  the AUTHZ-006 evidence package is produced — empty the guard body to Ok(()), capture the FAILING test run, restore the guard, capture the PASSING run, and write verify-manifest.json
  THEN:  .tmp/AUTHZ-006/AC-1-red-against-start.txt shows the test FAILING (a "FAILED" line for admin_write_guard_denies_non_admin_allows_admin, the dev-denied assertion broken by the no-op guard); .tmp/AUTHZ-006/AC-1-green.txt shows the SAME test PASSING against the restored real guard; .tmp/AUTHZ-006/verify-manifest.json parses as JSON and mirrors .tmp/AUTHZ-004/verify-manifest.json's keys (task_id, project_commands, verification_policy, requirements[])
  TEST_TIER: evidence (artifact presence + integrity; the RED/GREEN are produced by the real cargo test against the real guard)   VERIFICATION_SERVICE: real `cargo test -p but-api admin_write_guard` against the real config_mutate guard (emptied for RED, restored for GREEN) + filesystem integrity of .tmp/AUTHZ-006/
  VERIFY: test -d .tmp/AUTHZ-006 && grep -q 'FAILED' .tmp/AUTHZ-006/AC-1-red-against-start.txt && grep -q 'test result: ok' .tmp/AUTHZ-006/AC-1-green.txt && python3 -c "import json;d=json.load(open('.tmp/AUTHZ-006/verify-manifest.json'));assert d['task_id']=='AUTHZ-006' and set(['project_commands','verification_policy','requirements'])<=set(d)"
  SCENARIO: NEGATIVE_CONTROL would fail if the "RED" artifact PASSES (no FAILED line) — meaning it was captured against the real implementation, not an emptied guard (fabricated RED); if the RED was hand-written rather than real cargo stdout; if verify-manifest.json is missing/unparseable or omits the AUTHZ-004 schema keys; if AC-1-green.txt shows the test still failing (the guard was not properly restored).

AC-2: seeded-artifact captures + red-evidence-note.md mirror the .tmp/AUTHZ-004/ file set
  GIVEN: the .tmp/AUTHZ-004/ artifact shape as the template (per-AC red/green/seeded + red-output.txt + red-evidence-note.md + verify-manifest.json)
  WHEN:  the AUTHZ-006 package is assembled
  THEN:  .tmp/AUTHZ-006/ contains the AUTHZ-004-equivalent file set for AUTHZ-006's runtime ACs (AC-1 and AC-3 each with red-against-start/green/seeded-artifact), red-output.txt, and red-evidence-note.md whose prose documents which guard body was emptied and which assertion failed
  TEST_TIER: evidence (artifact presence + integrity)   VERIFICATION_SERVICE: filesystem integrity of .tmp/AUTHZ-006/ against the .tmp/AUTHZ-004/ template
  VERIFY: test -f .tmp/AUTHZ-006/red-evidence-note.md && test -f .tmp/AUTHZ-006/AC-1-seeded-artifact.txt && test -f .tmp/AUTHZ-006/AC-3-red-against-start.txt && test -f .tmp/AUTHZ-006/AC-3-green.txt
  SCENARIO: NEGATIVE_CONTROL would fail if the seeded-artifact / red-evidence-note.md files are missing (the package does not mirror AUTHZ-004); if red-evidence-note.md is empty or does not name the emptied guard body and the failing assertion (so the RED's genuineness cannot be confirmed); if the AC-3 (malformed-config) captures are absent.

AC-3: the committed diff touches ONLY .tmp/AUTHZ-006/ — the real guard is unchanged [integrity]
  GIVEN: the emptied-guard mutation was a transient capture device
  WHEN:  the final commit is prepared
  THEN:  `git diff` (and the staged diff) shows changes ONLY under .tmp/AUTHZ-006/; crates/but-api/src/legacy/config_mutate.rs is byte-identical to its pre-task state (the emptied body was reverted via git checkout)
  TEST_TIER: evidence (source-integrity)   VERIFICATION_SERVICE: git diff over the working tree
  VERIFY: git diff --name-only HEAD | grep -vq '^crates/but-api/src/legacy/config_mutate.rs$' && [ -z "$(git diff --name-only HEAD -- crates/but-api/src/legacy/config_mutate.rs)" ]
  SCENARIO: NEGATIVE_CONTROL would fail if config_mutate.rs is left with the emptied (no-op) guard body committed — that would DISABLE the admin-write guard in production, the cardinal sin (a stubbed guard reported as complete); if any non-.tmp/AUTHZ-006 source file is modified by this evidence task.

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, evidence_integrity): .tmp/AUTHZ-006/ exists; AC-1-red-against-start.txt shows the test FAILED (genuine RED against the emptied guard); AC-1-green.txt shows it passing (real guard); verify-manifest.json mirrors the AUTHZ-004 schema (M-2)
    VERIFY: test -d .tmp/AUTHZ-006 && grep -q 'FAILED' .tmp/AUTHZ-006/AC-1-red-against-start.txt && grep -q 'test result: ok' .tmp/AUTHZ-006/AC-1-green.txt
- TC-2 (-> AC-2, evidence_integrity): seeded-artifact captures + red-evidence-note.md + AC-3 captures present, mirroring .tmp/AUTHZ-004/ (M-2)
    VERIFY: test -f .tmp/AUTHZ-006/red-evidence-note.md && test -f .tmp/AUTHZ-006/AC-3-red-against-start.txt
- TC-3 (-> AC-3, source_integrity): the committed diff touches only .tmp/AUTHZ-006/; config_mutate.rs unchanged (the emptied body was reverted)
    VERIFY: [ -z "$(git diff --name-only HEAD -- crates/but-api/src/legacy/config_mutate.rs)" ]

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01 (verification policy)
provides: the missing AUTHZ-006 verification evidence package (.tmp/AUTHZ-006/) — a genuine RED-against-start (emptied guard → failing test), GREEN (real guard → passing), seeded artifacts, and a schema-mirroring verify-manifest.json — satisfying AUTHZ-006's requires_red_evidence + requires_seeded_evidence policy. Closes the M-2 evidence-policy gap so AUTHZ-006's cross-cutting verdict is no longer PARTIAL.
consumes: the existing AUTHZ-006 guard (config_mutate.rs::enforce_administration_write_gate) + tests (admin_write_guard_denies_non_admin_allows_admin, admin_write_guard_malformed_config_invalid); the .tmp/AUTHZ-004/ artifact set as the schema/shape template
boundary_contracts:
  - CAP-AUTHZ-01 (verification): a task declaring requires_red_evidence + requires_seeded_evidence must ship genuine RED-against-start + seeded artifacts. The RED must be captured against an emptied guard body (demonstrably failing), never fabricated; the real guard must remain in place in the committed state.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - .tmp/AUTHZ-006/ (NEW) — the full evidence package: AC-1-red-against-start.txt, AC-1-green.txt, AC-1-seeded-artifact.txt, AC-3-red-against-start.txt, AC-3-green.txt, AC-3-seeded-artifact.txt, red-output.txt, red-evidence-note.md, verify-manifest.json (mirroring .tmp/AUTHZ-004/). OWNS the AUTHZ-006 evidence package.
  - crates/but-api/src/legacy/config_mutate.rs (TRANSIENT ONLY) — the guard body may be emptied to Ok(()) SOLELY to capture the RED, then MUST be reverted (git checkout) before commit. The final committed state must be byte-identical to pre-task. NOT a persisted change.
writeProhibited:
  - crates/but-api/tests/admin_write_guard.rs — do NOT modify the tests (do NOT weaken to ease capture)
  - crates/but-authz/** and crates/but-api/src/legacy/forge.rs / merge_gate.rs — out of scope for this evidence task
  - any other .tmp/* directory — only .tmp/AUTHZ-006/ is created
  - Any committed change to config_mutate.rs (the emptied body is transient; a committed no-op guard is the cardinal sin)
  - Any file not listed in write_allowed

--------------------------------------------------------------------------------
OUT OF SCOPE
--------------------------------------------------------------------------------
  - Changing the AUTHZ-006 guard behavior or tests — the behavior is real + mutation-verified (red-hat); only the evidence is missing.
  - The forge fail-closed fix (FIX-AUTHZ-FORGE-FAILCLOSED) and forge coverage (FIX-AUTHZ-FORGE-COVERAGE).
  - Re-producing evidence for other AUTHZ tasks (their .tmp dirs already exist).

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. .tmp/AUTHZ-004/ (ls + read verify-manifest.json, red-evidence-note.md, AC-1-red-against-start.txt, AC-1-green.txt)
   Focus: THE TEMPLATE — the exact file set and the verify-manifest.json schema (task_id, project_commands, verification_policy, requirements[]) to mirror for AUTHZ-006. AC-1-red-against-start.txt shows what a genuine FAILED capture looks like (the FAILED line + the panic message).
2. crates/but-api/src/legacy/config_mutate.rs (13-28)
   Focus: enforce_administration_write_gate — the body (load_governance_config + resolve_principal_from_env + but_authz::authorize, 22-25) to TEMPORARILY empty to `Ok(())` for the RED capture, then REVERT.
3. crates/but-api/tests/admin_write_guard.rs (8-72)
   Focus: the two tests — admin_write_guard_denies_non_admin_allows_admin (10, AC-1: dev denied / admin Ok / working-tree self-grant denied) and admin_write_guard_malformed_config_invalid (72, AC-3). The dev-denied half is what FAILS under the emptied guard (the no-op permits dev).
4. .spec/prds/governance/tasks/sprint-02-authz-fail-closed-identity-confinement/AUTHZ-006-administration-write-guard.md (the trailing REQUIREMENT-CONTRACT JSON)
   Focus: the AC-1/AC-2/AC-3 + TC-1/TC-2/TC-3 {id, verify, description} to lift into .tmp/AUTHZ-006/verify-manifest.json's requirements[]; verification_policy requires_red_evidence:true + requires_seeded_evidence:true (the policy this task satisfies).

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- Evidence dir exists: `test -d .tmp/AUTHZ-006` -> Exit 0
- Genuine RED captured: `grep -q 'FAILED' .tmp/AUTHZ-006/AC-1-red-against-start.txt` -> Exit 0 (the test FAILED against the emptied guard — a passing "RED" is fabricated and fails this gate)
- GREEN captured: `grep -q 'test result: ok' .tmp/AUTHZ-006/AC-1-green.txt` -> Exit 0 (real guard restored → passes)
- Manifest schema mirrors AUTHZ-004: `python3 -c "import json;d=json.load(open('.tmp/AUTHZ-006/verify-manifest.json'));assert d['task_id']=='AUTHZ-006' and set(['project_commands','verification_policy','requirements'])<=set(d)"` -> Exit 0
- Note + seeded + AC-3 captures present: `test -f .tmp/AUTHZ-006/red-evidence-note.md && test -f .tmp/AUTHZ-006/AC-1-seeded-artifact.txt && test -f .tmp/AUTHZ-006/AC-3-red-against-start.txt` -> Exit 0
- Real guard unchanged in commit: `[ -z "$(git diff --name-only HEAD -- crates/but-api/src/legacy/config_mutate.rs)" ]` -> Exit 0 (the emptied body was reverted)
- Sanity: the real guard still passes its tests: `cargo test -p but-api admin_write_guard` -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
pattern: Genuine-RED-against-emptied-guard evidence capture — (1) empty enforce_administration_write_gate's body to `Ok(())`; (2) `cargo test -p but-api admin_write_guard_denies_non_admin_allows_admin 2>&1 | tee .tmp/AUTHZ-006/AC-1-red-against-start.txt` (must show FAILED on the dev-denied half); (3) `git checkout crates/but-api/src/legacy/config_mutate.rs` to restore the real guard; (4) `cargo test -p but-api admin_write_guard 2>&1 | tee .tmp/AUTHZ-006/AC-1-green.txt` (must pass); (5) capture seeded artifacts + write red-evidence-note.md + verify-manifest.json mirroring .tmp/AUTHZ-004/; (6) commit ONLY .tmp/AUTHZ-006/. The capture device (emptied guard) is transient and reverted; the committed source is unchanged.
pattern_source: .tmp/AUTHZ-004/ (the artifact-shape + verify-manifest.json schema template) + crates/but-api/src/legacy/config_mutate.rs:18-28 (the guard whose body is transiently emptied) + crates/but-api/tests/admin_write_guard.rs:10 (the test the RED must fail)
anti_pattern: Capturing the real guard's pass and labeling it RED (fabricated — no FAILED line); hand-writing a failing-looking log; emptying the guard and FORGETTING to revert (committing a no-op guard = disabled production authorization, the cardinal sin); a verify-manifest.json that does not mirror the AUTHZ-004 schema; an empty/absent red-evidence-note.md so the RED's genuineness cannot be confirmed; modifying the AUTHZ-006 tests to ease capture.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — Produces + commits the missing .tmp/AUTHZ-006/ evidence package mirroring .tmp/AUTHZ-004/: captures a GENUINE RED-against-start by temporarily emptying enforce_administration_write_gate's body so admin_write_guard_denies_non_admin_allows_admin FAILS on the dev-denied half, restores the real guard for the GREEN capture, assembles the seeded artifacts + red-evidence-note.md + a schema-mirroring verify-manifest.json, and commits ONLY .tmp/AUTHZ-006/ with config_mutate.rs reverted to byte-identical. Owns the genuine-RED capture, the artifact integrity, and the source-unchanged guarantee.
reviewer: rust-reviewer — Confirms the RED is genuine (real cargo stdout showing FAILED against the emptied guard, documented in red-evidence-note.md), the GREEN passes against the restored guard, verify-manifest.json mirrors the AUTHZ-004 schema, and the committed diff touches ONLY .tmp/AUTHZ-006/ (config_mutate.rs unchanged — no committed no-op guard).
coding_standards: crates/AGENTS.md, the repo verification policy (RED-against-start, no fabricated evidence)

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: AUTHZ-006 (the merged guard + tests this evidence documents)
Blocks:     (none — independent verification-policy closure; runs in parallel with the forge fixes)
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "FIX-AUTHZ-006-RED-EVIDENCE",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": false,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "notes": [
    "M-2: .tmp/AUTHZ-006/ is absent (dirs exist for AUTHZ-001/002/003/004/005/007/008); AUTHZ-006's REQUIREMENT-CONTRACT sets requires_red_evidence:true + requires_seeded_evidence:true but the artifacts were never produced (commit fbb6ffe32b committed only source + tests). The guard is real + mutation-verified; the gap is the missing evidence package.",
    "This is an EVIDENCE task: requires_tests=false because the verification is the PRESENCE + INTEGRITY of the .tmp/AUTHZ-006/ artifacts, not a new cargo acceptance test. The cargo runs are how the RED/GREEN captures are PRODUCED.",
    "Genuine-RED rule: the RED must be captured against a TEMPORARILY-EMPTIED enforce_administration_write_gate body (config_mutate.rs:18-28 -> Ok(())) so admin_write_guard_denies_non_admin_allows_admin FAILS on the dev-denied half. A 'RED' that passes is fabricated (captured against the real impl) and is an automatic FAIL.",
    "Source-integrity rule: the emptied guard body is a transient capture device, reverted via git checkout before commit. The committed diff must touch ONLY .tmp/AUTHZ-006/; a committed no-op guard would DISABLE production authorization (the cardinal sin).",
    "Schema-mirror: verify-manifest.json mirrors .tmp/AUTHZ-004/verify-manifest.json (task_id, project_commands, verification_policy, requirements[] of AC-1/AC-2/AC-3 + TC-1/TC-2/TC-3 lifted from the AUTHZ-006 contract)."
  ],
  "fixtures": {
    "authz004_template": {
      "description": "The existing .tmp/AUTHZ-004/ evidence package used as the artifact-shape + verify-manifest.json schema template: per-AC red-against-start/green/seeded-artifact .txt captures, red-output.txt, red-evidence-note.md, and verify-manifest.json {task_id, project_commands, verification_policy, requirements[]}. AUTHZ-006's package mirrors this set for AUTHZ-006's runtime ACs (AC-1, AC-3).",
      "seed_method": "filesystem",
      "records": [
        "ls .tmp/AUTHZ-004/ to enumerate the file set to mirror",
        "read .tmp/AUTHZ-004/verify-manifest.json for the schema to mirror",
        "read .tmp/AUTHZ-004/red-evidence-note.md for the prose shape of the RED note"
      ]
    },
    "emptied_guard_capture_device": {
      "description": "A TRANSIENT mutation of crates/but-api/src/legacy/config_mutate.rs where enforce_administration_write_gate's body is replaced with `Ok(())` (no load_governance_config / resolve_principal_from_env / but_authz::authorize). Running admin_write_guard_denies_non_admin_allows_admin against this device FAILS on the dev-denied half (the no-op permits dev, who holds only contents:write), producing the genuine RED. The device is reverted (git checkout) before the real guard's GREEN capture and before commit.",
      "seed_method": "manual",
      "records": [
        "temporarily replace enforce_administration_write_gate body (config_mutate.rs:18-28) with `Ok(())`",
        "cargo test -p but-api admin_write_guard_denies_non_admin_allows_admin 2>&1 | tee .tmp/AUTHZ-006/AC-1-red-against-start.txt  (must show FAILED)",
        "git checkout crates/but-api/src/legacy/config_mutate.rs  (revert to the real guard)",
        "cargo test -p but-api admin_write_guard 2>&1 | tee .tmp/AUTHZ-006/AC-1-green.txt  (must pass)"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN AUTHZ-006 shipped its guard + tests but no .tmp/AUTHZ-006/ evidence WHEN the package is produced by emptying enforce_administration_write_gate, capturing the failing test, restoring the guard, capturing the passing test, and writing verify-manifest.json THEN .tmp/AUTHZ-006/AC-1-red-against-start.txt shows admin_write_guard_denies_non_admin_allows_admin FAILING (dev-denied half broken by the no-op guard), AC-1-green.txt shows it PASSING against the restored real guard, and verify-manifest.json parses and mirrors .tmp/AUTHZ-004/verify-manifest.json's keys",
      "verify": "test -d .tmp/AUTHZ-006 && grep -q 'FAILED' .tmp/AUTHZ-006/AC-1-red-against-start.txt && grep -q 'test result: ok' .tmp/AUTHZ-006/AC-1-green.txt && python3 -c \"import json;d=json.load(open('.tmp/AUTHZ-006/verify-manifest.json'));assert d['task_id']=='AUTHZ-006' and set(['project_commands','verification_policy','requirements'])<=set(d)\"",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "evidence",
        "verification_service": "real `cargo test -p but-api admin_write_guard` against the real config_mutate guard (emptied for RED, restored for GREEN) + filesystem integrity of .tmp/AUTHZ-006/",
        "negative_control": {
          "would_fail_if": [
            "the AC-1-red-against-start.txt 'RED' artifact PASSES (no FAILED line) — proving it was captured against the real implementation, not an emptied guard (a fabricated RED)",
            "the RED was hand-written rather than real cargo stdout (no genuine test-runner output / panic message)",
            "verify-manifest.json is missing, unparseable, or omits the AUTHZ-004 schema keys (task_id/project_commands/verification_policy/requirements)",
            "AC-1-green.txt shows the test still failing (the guard was not properly restored before the GREEN capture)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "emptied_guard_capture_device",
            "action": {
              "actor": "ci",
              "steps": [
                "empty enforce_administration_write_gate body to Ok(()); cargo test -p but-api admin_write_guard_denies_non_admin_allows_admin -> capture FAILING output to AC-1-red-against-start.txt (cite M-2 genuine RED)"
              ]
            },
            "end_state": {
              "must_observe": [
                ".tmp/AUTHZ-006/AC-1-red-against-start.txt contains a `FAILED` line for admin_write_guard_denies_non_admin_allows_admin (the no-op guard permits dev, breaking the dev-denied assertion)"
              ],
              "must_not_observe": [
                "a passing 'RED' capture (test result: ok with no FAILED) — a fabricated RED taken against the real guard"
              ]
            }
          },
          {
            "start_ref": "authz004_template",
            "action": {
              "actor": "ci",
              "steps": [
                "git checkout config_mutate.rs; cargo test -p but-api admin_write_guard -> capture PASSING output to AC-1-green.txt; write verify-manifest.json mirroring the AUTHZ-004 schema (cite M-2)"
              ]
            },
            "end_state": {
              "must_observe": [
                ".tmp/AUTHZ-006/AC-1-green.txt shows `test result: ok` (the restored real guard passes)",
                ".tmp/AUTHZ-006/verify-manifest.json parses with task_id==\"AUTHZ-006\" and the AUTHZ-004 schema keys present"
              ],
              "must_not_observe": [
                "AC-1-green.txt still showing a failure (guard not restored)",
                "verify-manifest.json missing the project_commands/verification_policy/requirements keys"
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
      "description": "GIVEN the .tmp/AUTHZ-004/ artifact shape as template WHEN the AUTHZ-006 package is assembled THEN .tmp/AUTHZ-006/ contains the AUTHZ-004-equivalent file set for AUTHZ-006's runtime ACs (AC-1 + AC-3 each with red-against-start/green/seeded-artifact), red-output.txt, and a red-evidence-note.md documenting which guard body was emptied and which assertion failed",
      "verify": "test -f .tmp/AUTHZ-006/red-evidence-note.md && test -f .tmp/AUTHZ-006/AC-1-seeded-artifact.txt && test -f .tmp/AUTHZ-006/AC-3-red-against-start.txt && test -f .tmp/AUTHZ-006/AC-3-green.txt",
      "maps_to_ac": null,
      "scenario": {
        "tier": "holdout",
        "test_tier": "evidence",
        "verification_service": "filesystem integrity of .tmp/AUTHZ-006/ against the .tmp/AUTHZ-004/ template",
        "negative_control": {
          "would_fail_if": [
            "the seeded-artifact / red-evidence-note.md files are missing (the package does not mirror AUTHZ-004)",
            "red-evidence-note.md is empty or does not name the emptied guard body and the failing assertion (the RED's genuineness cannot be confirmed)",
            "the AC-3 (malformed-config) captures are absent (only AC-1 documented)"
          ]
        },
        "evidence": {
          "artifact_type": "file",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "authz004_template",
            "action": {
              "actor": "ci",
              "steps": [
                "assemble the seeded-artifact captures + red-output.txt + red-evidence-note.md + AC-3 captures mirroring .tmp/AUTHZ-004/ (cite M-2)"
              ]
            },
            "end_state": {
              "must_observe": [
                ".tmp/AUTHZ-006/red-evidence-note.md exists and names the emptied guard body + the failing assertion",
                ".tmp/AUTHZ-006/AC-1-seeded-artifact.txt, AC-3-red-against-start.txt, AC-3-green.txt all present"
              ],
              "must_not_observe": [
                "an empty or absent red-evidence-note.md",
                "missing AC-3 (malformed-config) captures"
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
      "description": "GIVEN the emptied-guard mutation was a transient capture device WHEN the final commit is prepared THEN `git diff` shows changes ONLY under .tmp/AUTHZ-006/ and crates/but-api/src/legacy/config_mutate.rs is byte-identical to its pre-task state (the emptied body was reverted) — no committed no-op guard [integrity]",
      "verify": "[ -z \"$(git diff --name-only HEAD -- crates/but-api/src/legacy/config_mutate.rs)\" ] && cargo test -p but-api admin_write_guard",
      "maps_to_ac": null,
      "scenario": {
        "tier": "holdout",
        "test_tier": "evidence",
        "verification_service": "git diff over the working tree + real `cargo test -p but-api admin_write_guard` sanity",
        "negative_control": {
          "would_fail_if": [
            "config_mutate.rs is left with the emptied (no-op) guard body committed — DISABLING the admin-write guard in production (a stubbed guard reported as complete, the cardinal sin)",
            "any non-.tmp/AUTHZ-006 source file is modified by this evidence task",
            "the restored guard no longer passes admin_write_guard (the revert was incorrect)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "authz004_template",
            "action": {
              "actor": "ci",
              "steps": [
                "git diff --name-only HEAD -- config_mutate.rs (must be empty); cargo test -p but-api admin_write_guard (must pass) (cite M-2 source-integrity)"
              ]
            },
            "end_state": {
              "must_observe": [
                "`git diff --name-only HEAD -- crates/but-api/src/legacy/config_mutate.rs` is empty (the guard is unchanged)",
                "`cargo test -p but-api admin_write_guard` exits 0 (the restored real guard passes)"
              ],
              "must_not_observe": [
                "a committed change to config_mutate.rs (an emptied/no-op guard persisted)",
                "any non-.tmp/AUTHZ-006 file in the task's diff"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": ".tmp/AUTHZ-006/ exists; AC-1-red-against-start.txt shows the test FAILED (genuine RED against the emptied guard); AC-1-green.txt shows it passing; verify-manifest.json mirrors the AUTHZ-004 schema (M-2)",
      "verify": "test -d .tmp/AUTHZ-006 && grep -q 'FAILED' .tmp/AUTHZ-006/AC-1-red-against-start.txt && grep -q 'test result: ok' .tmp/AUTHZ-006/AC-1-green.txt",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "seeded-artifact captures + red-evidence-note.md + AC-3 captures present, mirroring .tmp/AUTHZ-004/ (M-2)",
      "verify": "test -f .tmp/AUTHZ-006/red-evidence-note.md && test -f .tmp/AUTHZ-006/AC-3-red-against-start.txt",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "the committed diff touches only .tmp/AUTHZ-006/; config_mutate.rs unchanged (the emptied capture device was reverted — no committed no-op guard)",
      "verify": "[ -z \"$(git diff --name-only HEAD -- crates/but-api/src/legacy/config_mutate.rs)\" ]",
      "maps_to_ac": "AC-3"
    }
  ]
}
-->
</details>
