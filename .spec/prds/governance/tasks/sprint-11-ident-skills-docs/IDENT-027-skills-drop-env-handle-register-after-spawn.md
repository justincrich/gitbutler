# IDENT-027 — `but-run-sprint` + `but-orchestrate` + `but-sprint-tasks-plan` + `but-sprint-plan` skills (brain) — drop `export BUT_AGENT_HANDLE=...` from dispatch templates; orchestrator calls `but agent register --pid <child> --as <agent>` after spawn; `BUT-SKILL-CONVENTIONS.md` §9 documents the new model

**Sprint:** [Sprint 11](./SPRINT.md) · **Agent:** `rust-planner` · **Estimate:** 240 min · **Type:** FEATURE · **Status:** Backlog · **Proposed By:** rust-planner

## Background

Per the sprint stub this dispatch-model migration across the four governed pipeline skills is owned by rust-planner — it is process-authoring over the `but-*` orchestration pipeline (planner is the head of that pipeline), editing canonical brain skills + the conventions doc and mirroring them.

**Provides:** (documentation/consumer layer — no new capability)

**Consumes:** but agent register --pid <child> --as <agent> verb (CAP-AUTHZ-01), registry-backed resolution as the enforced default (env handle test-only)

**Boundary contracts:**
- The orchestrator skills register each subagent's PID via `but agent register` after spawn and no longer self-assert BUT_AGENT_HANDLE in dispatch templates, matching the engine's resolve_principal_with_registry default.

## Critical Constraints

**MUST:**
- Edit the CANONICAL skills at `~/Projects/brain/skills/{but-run-sprint,but-orchestrate,but-sprint-tasks-plan,but-sprint-plan}/` and `~/Projects/brain/docs/BUT-SKILL-CONVENTIONS.md`, MIRROR each skill to `~/.claude/skills/`, then verify `diff -rq` reports no differences per skill.
- Remove every `export BUT_AGENT_HANDLE=...` from implementer/reviewer dispatch templates across the four skills.
- The orchestrator must call `but agent register --pid <child_pid> --as <assigned_agent>` immediately after spawning each subagent.
- `BUT-SKILL-CONVENTIONS.md` gains a NEW §9 documenting the register-after-spawn identity model (the doc currently stops at §8).
- Replace the self-asserted orchestrator merge identity in `but-run-sprint/docs/worktree-lifecycle.md` (`BUT_AGENT_HANDLE={orchestrator_principal} but merge`) with `but agent register --pid $$ --as {orchestrator_principal}` immediately before `but merge` — DISTINCT from the spawn-phase dispatch register (the merge gate at merge_gate.rs denies an unregistered orchestrator).

**NEVER:**
- Never report the e2e single-task sprint (AC-1) as PASSED inside the gitbutler repo's test suite — it is OWNED BY THE BRAIN REPO's skill tests (T-IDENT-038); the gitbutler repo asserts the SOURCE-grep contract.
- Never leave a subagent self-asserting `export BUT_AGENT_HANDLE` in any dispatch template.
- Never leave any of the four mirrors out of sync with its canonical copy.

**STRICTLY:**
- BLOCKED-UNTIL Sprint-10 (final env-handle deny-default locked) and transitively Sprint-09 (`but agent register` verb landed + 8-callsite swap) — the skills cannot honestly stop self-asserting BUT_AGENT_HANDLE until the registry path is the enforced default.
- SCOPE-HONESTY: PRIMARY e2e (single-task sprint) is BRAIN-REPO-OWNED; supplementary ACs are SOURCE greps on the skill/template/doc files.

## Specification

**Objective:** Migrate the four governed pipeline skills so dispatch templates no longer `export BUT_AGENT_HANDLE` and the orchestrator instead calls `but agent register --pid <child> --as <agent>` after spawning each subagent, with a new BUT-SKILL-CONVENTIONS §9 documenting the model; canonical and mirror copies kept identical.

**Success state:** A single-task end-to-end sprint passes with the orchestrator registering the implementer and zero BUT_AGENT_HANDLE references in dispatch templates; on source, dispatch templates contain no `export BUT_AGENT_HANDLE`, the orchestrator calls `but agent register --pid ... --as ...`, BUT-SKILL-CONVENTIONS §9 documents it, and all four canonical/mirror skill copies diff clean.

## Acceptance Criteria

**AC-1 (PRIMARY)** — Single-task sprint: orchestrator registers implementer; zero handle refs in templates (e2e, BRAIN-REPO-OWNED)
- **GIVEN:** a governed fixture with one ready single-task sprint and a committed agents.toml roster
- **WHEN:** running /but-run-sprint end-to-end on it
- **THEN:** the orchestrator calls `but agent register --pid <child> --as <implementer>` after spawn and the implementer's `but` calls succeed via the registry (no BUT_AGENT_HANDLE consumed; zero handle refs in dispatch templates)
- **Verify:** `cd single_task_sprint_fixture && before=$(git rev-parse HEAD) && /but-run-sprint sprint-x 2>&1 | grep -Eq 'but agent register --pid [0-9]+ --as ' && ! grep -rq 'export BUT_AGENT_HANDLE' ~/Projects/brain/skills/but-run-sprint/templates && test "$(git rev-parse HEAD)" != "$before"`
- **TEST_TIER:** e2e · **VERIFICATION_SERVICE:** but-run-sprint skill + single-task sprint fixture + but CLI (BRAIN-REPO-OWNED; T-IDENT-038) · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=single_task_sprint_fixture`; must_observe = [orchestrator log shows a `but agent register --pid` `--as` line; new HEAD SHA `!=` before (1 new registry-path commit)]; must_not_observe = [subagent runs `export BUT_AGENT_HANDLE` (start state); HEAD unchanged / commit denied (0 new commits)]; negative_control.would_fail_if = [orchestrator never calls `but agent register`; subagent still self-asserts BUT_AGENT_HANDLE; governed commit fails because the registry path is bypassed; greps stubbed or not executed].

**AC-2** — Zero `export BUT_AGENT_HANDLE` in dispatch templates across the four skills
- **GIVEN:** the canonical four skills' dispatch templates
- **WHEN:** grepping for the env-handle export
- **THEN:** no `export BUT_AGENT_HANDLE` remains in any dispatch template
- **Verify:** `! grep -rq 'export BUT_AGENT_HANDLE' ~/Projects/brain/skills/but-run-sprint ~/Projects/brain/skills/but-orchestrate ~/Projects/brain/skills/but-sprint-tasks-plan ~/Projects/brain/skills/but-sprint-plan`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** source (four brain skills' templates) · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=dispatch_self_identity_skills`; must_observe = [grep -r for 'export BUT_AGENT_HANDLE' across the four skills returns 0 matches (exit 1 on grep -q)]; must_not_observe = [any `export BUT_AGENT_HANDLE` line in dispatch templates; self-asserted identity left in but-run-sprint]; negative_control.would_fail_if = [export removed in only some skills; export left in templates; grep stubbed or not executed].

**AC-3** — Orchestrator calls `but agent register --pid <child> --as <agent>` after spawn
- **GIVEN:** the dispatch surfaces (but-run-sprint templates/ + SKILL.md, but-orchestrate spawn-stage.sh + SKILL.md) — NOT the merge doc
- **WHEN:** inspecting the post-spawn step
- **THEN:** the spawn dispatch register `but agent register --pid <child> --as <agent>` is present in the dispatch surfaces (scoped to exclude docs/worktree-lifecycle.md, where the merge self-register lives)
- **Verify:** `grep -rqE 'but agent register --pid .*--as' ~/Projects/brain/skills/but-run-sprint/templates ~/Projects/brain/skills/but-run-sprint/SKILL.md ~/Projects/brain/skills/but-orchestrate/references/spawn-stage.sh ~/Projects/brain/skills/but-orchestrate/SKILL.md`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** source (but-run-sprint + but-orchestrate dispatch surfaces; merge doc excluded) · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=dispatch_self_identity_skills`; must_observe = [dispatch surfaces (`templates/`, `spawn-stage.sh`, dispatch `SKILL.md`) contain `but agent register --pid` with `--as`; spawn register lives in dispatch surfaces, not `docs/worktree-lifecycle.md`]; must_not_observe = [dispatch register absent from templates (0 matches in dispatch surfaces); only the merge self-register present (none in dispatch templates)]; negative_control.would_fail_if = [dispatch register not added to the dispatch surfaces (absent); register only in `docs/worktree-lifecycle.md` (merge), dispatch templates unchanged; grep stubbed or not executed].

**AC-4** — BUT-SKILL-CONVENTIONS.md §9 documents the register-after-spawn model
- **GIVEN:** BUT-SKILL-CONVENTIONS.md (currently §1-§8)
- **WHEN:** inspecting the new section
- **THEN:** a new §9 documents that orchestrators register subagents via `but agent register` after spawn and no longer self-assert BUT_AGENT_HANDLE
- **Verify:** `grep -qE '^## 9' ~/Projects/brain/docs/BUT-SKILL-CONVENTIONS.md && grep -A20 '^## 9' ~/Projects/brain/docs/BUT-SKILL-CONVENTIONS.md | grep -q 'but agent register' && grep -A20 '^## 9' ~/Projects/brain/docs/BUT-SKILL-CONVENTIONS.md | grep -q 'BUT_AGENT_HANDLE'`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** source (brain/docs/BUT-SKILL-CONVENTIONS.md) · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=dispatch_self_identity_skills`; must_observe = [`BUT-SKILL-CONVENTIONS.md` has a `## 9` heading; §9 contains `but agent register` + deprecated `BUT_AGENT_HANDLE`]; must_not_observe = [doc ends at §8 (no `## 9` heading); §9 omits register model (0 matches)]; negative_control.would_fail_if = [§9 not added; register model not documented in §9; grep stubbed or not executed].

**AC-5** — All four canonical skills match their ~/.claude mirrors
- **GIVEN:** the four canonical skills and their ~/.claude mirrors
- **WHEN:** diffing each pair recursively
- **THEN:** `diff -rq` reports no differences for all four skills
- **Verify:** `for s in but-run-sprint but-orchestrate but-sprint-tasks-plan but-sprint-plan; do diff -rq ~/Projects/brain/skills/$s ~/.claude/skills/$s || exit 1; done`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** filesystem (brain canonical vs ~/.claude mirrors) · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=dispatch_self_identity_skills`; must_observe = [diff -rq prints nothing for all four skills (exit 0); mirrors reflect the register-after-spawn edits]; must_not_observe = ['Files ... differ' output; 'Only in' output; any mirror still self-asserting BUT_AGENT_HANDLE]; negative_control.would_fail_if = [only canonical copies edited (mirrors stale); diff stubbed or not executed; one of the four mirrors skipped].

**AC-6** — No residual `BUT_AGENT_HANDLE=` assignment; orchestrator MERGE self-registers its own pid (distinct from dispatch register)
- **GIVEN:** the four canonical skills (SKILL bodies, templates, docs/worktree-lifecycle.md, references/spawn-stage.sh)
- **WHEN:** grepping for any `BUT_AGENT_HANDLE=` assignment and for a merge-scoped `but agent register --pid` co-located with `but merge`
- **THEN:** no `BUT_AGENT_HANDLE=` assignment remains in any of the four skills, AND the merge step in worktree-lifecycle.md registers the orchestrator's own pid (`but agent register --pid`) within +/-3 lines of `but merge` — provable independently of the dispatch `--as` register
- **Verify:** `! grep -rqE 'BUT_AGENT_HANDLE=' ~/Projects/brain/skills/but-run-sprint ~/Projects/brain/skills/but-orchestrate ~/Projects/brain/skills/but-sprint-plan ~/Projects/brain/skills/but-sprint-tasks-plan && grep -A3 -B3 'but merge' ~/Projects/brain/skills/but-run-sprint/docs/worktree-lifecycle.md | grep -q 'but agent register --pid'`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** source (four brain skills; merge self-register scoped to worktree-lifecycle.md) · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=dispatch_self_identity_skills`; must_observe = [0 `BUT_AGENT_HANDLE=` assignments across the four skills; merge step in `worktree-lifecycle.md` has `but agent register --pid` within +/-3 lines of `but merge`]; must_not_observe = [merge step still self-asserts `BUT_AGENT_HANDLE=` (start state); merge self-register absent while dispatch register present (none in merge section)]; negative_control.would_fail_if = [merge step still self-asserts `BUT_AGENT_HANDLE=` (unchanged); merge self-register removed/absent (only the spawn `--as` register present); grep stubbed or not executed].

**AC-7** — Registered implementer process resolves via `but agent whoami` with BUT_AGENT_HANDLE unset (e2e, BRAIN-REPO-OWNED)
- **GIVEN:** a registered implementer subagent process in a single-task sprint, BUT_AGENT_HANDLE unset
- **WHEN:** running `but agent whoami` from inside that process
- **THEN:** `but agent whoami` prints the registered agent_id (resolved from the runtime registry, no env handle)
- **Verify:** `cd single_task_sprint_fixture && unset BUT_AGENT_HANDLE && but agent whoami | grep -Eq '[a-z-]+(implementer|reviewer)'`
- **TEST_TIER:** e2e · **VERIFICATION_SERVICE:** but agent whoami in the registered implementer process (BRAIN-REPO-OWNED; gate step 5) · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=single_task_sprint_fixture`; must_observe = [`but agent whoami` prints a registered agent_id (e.g. `rust-implementer`); resolved with `BUT_AGENT_HANDLE` unset (1 registry hit)]; must_not_observe = [whoami errors `Denial::unregistered` (0 id); agent_id only via env handle (none from registry)]; negative_control.would_fail_if = [the implementer pid was never registered (registry empty); agent_id resolvable only via a mock/env handle; grep stubbed or not executed].

## Test Criteria

| ID | Boolean | Maps to AC |
|----|---------|------------|
| TC-1 | Single-task sprint: orchestrator registers the implementer and zero BUT_AGENT_HANDLE refs in dispatch templates is true (BRAIN-REPO-OWNED) | AC-1 |
| TC-2 | No `export BUT_AGENT_HANDLE` remains in any of the four skills' dispatch templates is true | AC-2 |
| TC-3 | The spawn dispatch register `but agent register --pid ... --as ...` exists in the dispatch surfaces (excluding the merge doc) is true | AC-3 |
| TC-4 | BUT-SKILL-CONVENTIONS.md §9 documents the register-after-spawn model is true | AC-4 |
| TC-5 | All four canonical skills are byte-identical to their ~/.claude mirrors is true | AC-5 |
| TC-6 | No `BUT_AGENT_HANDLE=` assignment remains and the merge step self-registers its own pid (co-located with `but merge`) is true | AC-6 |
| TC-7 | `but agent whoami` in the registered implementer process prints the agent_id with BUT_AGENT_HANDLE unset is true (BRAIN-REPO-OWNED) | AC-7 |

## Reading List

1. `/Users/justinrich/Projects/brain/skills/but-run-sprint/SKILL.md`:40-60 — PRIMARY PATTERN — the current self-identity model (subagents `export BUT_AGENT_HANDLE`, lines ~44-50); replace with orchestrator register-after-spawn.
2. `/Users/justinrich/Projects/brain/skills/but-run-sprint/templates/implementer-prompt.md`:1-60 — Dispatch template that must drop `export BUT_AGENT_HANDLE`.
3. `/Users/justinrich/Projects/brain/skills/but-orchestrate/references/spawn-stage.sh`:55-62 — Stage spawn that exports its own BUT_AGENT_HANDLE (lines 58/60/62); switch to orchestrator `but agent register --pid <child> --as <agent>`.
4. `/Users/justinrich/Projects/brain/docs/BUT-SKILL-CONVENTIONS.md`:21-95 — §2 identity model (currently BUT_AGENT_HANDLE) + §8 end; add NEW §9 register-after-spawn model.
5. `/Users/justinrich/Projects/gitbutler/.spec/prds/governance/12-uc-agent-identity.md`:82-92 — UC-IDENT-05 AC-7 — the exact dispatch contract to encode.

## Guardrails

**WRITE-ALLOWED:**
- /Users/justinrich/Projects/brain/skills/but-run-sprint/** (MODIFY — drop export, add register-after-spawn in dispatch flow + templates)
- /Users/justinrich/Projects/brain/skills/but-orchestrate/** (MODIFY — drop stage self-handle, add register-after-spawn)
- /Users/justinrich/Projects/brain/skills/but-sprint-tasks-plan/** (MODIFY — drop any handle export from dispatch templates)
- /Users/justinrich/Projects/brain/skills/but-sprint-plan/** (MODIFY — drop any handle export from dispatch templates)
- /Users/justinrich/Projects/brain/docs/BUT-SKILL-CONVENTIONS.md (MODIFY — add §9)
- /Users/justinrich/.claude/skills/but-run-sprint/** + but-orchestrate/** + but-sprint-tasks-plan/** + but-sprint-plan/** (MODIFY — mirrors)

**WRITE-PROHIBITED:**
- /Users/justinrich/Projects/gitbutler/** - skill task edits brain skills/docs, not the gitbutler repo
- crates/** - the `but agent register` engine is upstream (Sprint 08)
- but-init/but-migrate skills - separate tasks (IDENT-025/026)

## Code Pattern

**Reference:** /Users/justinrich/Projects/brain/skills/but-run-sprint/SKILL.md, /Users/justinrich/Projects/brain/skills/but-orchestrate/references/spawn-stage.sh, /Users/justinrich/Projects/brain/docs/BUT-SKILL-CONVENTIONS.md, 12-uc-agent-identity.md UC-IDENT-05 AC-7

**Pattern:** Invert the identity flow: delete `export BUT_AGENT_HANDLE=...` from every implementer/reviewer dispatch template; in the orchestrator's post-spawn step (immediately after Task/spawn returns a child pid) call `but agent register --pid <child_pid> --as <assigned_agent>` so the engine's registry resolves the subagent's governed `but` calls. Add BUT-SKILL-CONVENTIONS.md §9 documenting this register-after-spawn contract and the deprecation of self-asserted handles.

**Source:** `UC-IDENT-05 AC-7 dispatch contract + the existing BUT-SKILL-CONVENTIONS §2 identity section (extended by §9).`

**Design notes:**
- Edit canonical -> mirror -> diff -rq clean per skill is mandatory (AC-5).

**Anti-pattern:** Do NOT leave any subagent self-asserting BUT_AGENT_HANDLE; do NOT register without --pid/--as; do NOT edit only canonical copies (mirror all four); do NOT claim the single-task-sprint e2e passes in the gitbutler test suite (brain-repo-owned).

## Agent Instructions

TDD RED→GREEN per AC (build-gate/source greps + real `but`/skill execution against real files — NO mocks):
1. **RED:** write each AC's failing check first (against the current start state — docs/skill absent or still self-asserting `BUT_AGENT_HANDLE`).
2. **GREEN:** make the minimal edit (docs/skill/migration) to satisfy the AC.
3. For brain-skill tasks: edit the CANONICAL copy under `~/Projects/brain/skills/`, MIRROR to `~/.claude/skills/`, then `diff -rq` clean.
4. Run each AC's verify command; commit via `but commit` (governed).

## Orchestrator Verification Protocol

- `! grep -rq 'export BUT_AGENT_HANDLE' ~/Projects/brain/skills/but-run-sprint ~/Projects/brain/skills/but-orchestrate ~/Projects/brain/skills/but-sprint-tasks-plan ~/Projects/brain/skills/but-sprint-plan` → exit 0
- `grep -rqE 'but agent register --pid .*--as' ~/Projects/brain/skills/but-run-sprint ~/Projects/brain/skills/but-orchestrate && grep -qE '^## 9' ~/Projects/brain/docs/BUT-SKILL-CONVENTIONS.md` → exit 0
- `for s in but-run-sprint but-orchestrate but-sprint-tasks-plan but-sprint-plan; do diff -rq ~/Projects/brain/skills/$s ~/.claude/skills/$s || exit 1; done` → no output (exit 0)
- `run /but-run-sprint on a single-task sprint fixture; capture the `but agent register --pid ... --as ...` call + landed commit` → orchestrator registers the implementer; commit lands via registry; 0 handle refs (run where the skills live)

## Agent Assignment

**Agent:** `rust-planner` — Per the sprint stub this dispatch-model migration across the four governed pipeline skills is owned by rust-planner — it is process-authoring over the `but-*` orchestration pipeline (planner is the head of that pipeline), editing canonical brain skills + the conventions doc and mirroring them.
**Pairing:** none (single-surface Rust/docs/skill task). Honors `crates/AGENTS.md` + `BUT-SKILL-CONVENTIONS.md`.

## Evidence Gates

- `! grep -rq 'export BUT_AGENT_HANDLE' ~/Projects/brain/skills/but-run-sprint ~/Projects/brain/skills/but-orchestrate ~/Projects/brain/skills/but-sprint-tasks-plan ~/Projects/brain/skills/but-sprint-plan` (exit 0)
- `grep -rqE 'but agent register --pid .*--as' ~/Projects/brain/skills/but-run-sprint ~/Projects/brain/skills/but-orchestrate && grep -qE '^## 9' ~/Projects/brain/docs/BUT-SKILL-CONVENTIONS.md` (exit 0)
- `for s in but-run-sprint but-orchestrate but-sprint-tasks-plan but-sprint-plan; do diff -rq ~/Projects/brain/skills/$s ~/.claude/skills/$s || exit 1; done` (no output (exit 0))
- `run /but-run-sprint on a single-task sprint fixture; capture the `but agent register --pid ... --as ...` call + landed commit` (orchestrator registers the implementer; commit lands via registry; 0 handle refs (run where the skills live))

## Review Criteria

- AC-1: PRIMARY — Single-task sprint: orchestrator registers implementer; zero handle refs in templates (e2e, BRAIN-REPO-OWNED) — verified by `cd single_task_sprint_fixture && before=$(git rev-parse HEAD) && /but-run-sprint sprint-x 2>&1 | grep -Eq 'but agent register --pid [0-9]+ --as ' && ! grep -rq 'export BUT_AGENT_HANDLE' ~/Projects/brain/skills/but-run-sprint/templates && test "$(git rev-parse HEAD)" != "$before"`.
- AC-2: Zero `export BUT_AGENT_HANDLE` in dispatch templates across the four skills — verified by `! grep -rq 'export BUT_AGENT_HANDLE' ~/Projects/brain/skills/but-run-sprint ~/Projects/brain/skills/but-orchestrate ~/Projects/brain/skills/but-sprint-tasks-plan ~/Projects/brain/skills/but-sprint-plan`.
- AC-3: Orchestrator calls `but agent register --pid <child> --as <agent>` after spawn — verified by `grep -rqE 'but agent register --pid .*--as' ~/Projects/brain/skills/but-run-sprint/templates ~/Projects/brain/skills/but-run-sprint/SKILL.md ~/Projects/brain/skills/but-orchestrate/references/spawn-stage.sh ~/Projects/brain/skills/but-orchestrate/SKILL.md`.
- AC-4: BUT-SKILL-CONVENTIONS.md §9 documents the register-after-spawn model — verified by `grep -qE '^## 9' ~/Projects/brain/docs/BUT-SKILL-CONVENTIONS.md && grep -A20 '^## 9' ~/Projects/brain/docs/BUT-SKILL-CONVENTIONS.md | grep -q 'but agent register' && grep -A20 '^## 9' ~/Projects/brain/docs/BUT-SKILL-CONVENTIONS.md | grep -q 'BUT_AGENT_HANDLE'`.
- AC-5: All four canonical skills match their ~/.claude mirrors — verified by `for s in but-run-sprint but-orchestrate but-sprint-tasks-plan but-sprint-plan; do diff -rq ~/Projects/brain/skills/$s ~/.claude/skills/$s || exit 1; done`.
- AC-6: No residual `BUT_AGENT_HANDLE=` assignment; orchestrator MERGE self-registers its own pid (distinct from dispatch register) — verified by `! grep -rqE 'BUT_AGENT_HANDLE=' ~/Projects/brain/skills/but-run-sprint ~/Projects/brain/skills/but-orchestrate ~/Projects/brain/skills/but-sprint-plan ~/Projects/brain/skills/but-sprint-tasks-plan && grep -A3 -B3 'but merge' ~/Projects/brain/skills/but-run-sprint/docs/worktree-lifecycle.md | grep -q 'but agent register --pid'`.
- AC-7: Registered implementer process resolves via `but agent whoami` with BUT_AGENT_HANDLE unset (e2e, BRAIN-REPO-OWNED) — verified by `cd single_task_sprint_fixture && unset BUT_AGENT_HANDLE && but agent whoami | grep -Eq '[a-z-]+(implementer|reviewer)'`.
- Honors NEVER: Never report the e2e single-task sprint (AC-1) as PASSED inside the gitbutler repo's test suite — it is OWNED BY THE BRAIN REPO's skill tests (T-IDENT-038); the gitbutler repo asserts the SOURCE-grep contract.

## Dependencies

- **Depends on:** none (BLOCKED-UNTIL Sprint-10 per Critical Constraints)
- **Blocks:** none
- **Capabilities:** CAP-AUTHZ-01

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "IDENT-027",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "dispatch_self_identity_skills": {
      "description": "the four canonical skills where subagents self-assert identity (`export BUT_AGENT_HANDLE=...`) and BUT-SKILL-CONVENTIONS.md stops at \u00a78",
      "seed_method": "public_api",
      "records": [
        "but-run-sprint/SKILL.md describes subagents setting their own `export BUT_AGENT_HANDLE` (lines ~44-50; 1 occurrence)",
        "but-orchestrate/SKILL.md spawns stages with their own BUT_AGENT_HANDLE (references/spawn-stage.sh, line ~17)",
        "BUT-SKILL-CONVENTIONS.md has sections \u00a71-\u00a78; no \u00a79"
      ]
    },
    "single_task_sprint_fixture": {
      "description": "a governed fixture repo with one ready single-task sprint and a committed agents.toml roster",
      "seed_method": "cli",
      "records": [
        "fixture repo registered as a GitButler project with agents.toml roster",
        "one sprint task ready to dispatch to an implementer"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN a governed fixture with one ready single-task sprint and a committed agents.toml roster WHEN running /but-run-sprint end-to-end on it THEN the orchestrator calls `but agent register --pid <child> --as <implementer>` after spawn and the implementer's `but` calls succeed via the registry (no BUT_AGENT_HANDLE consumed; zero handle refs in dispatch templates)",
      "test_tier": "e2e",
      "verification_service": "but-run-sprint skill + single-task sprint fixture + but CLI (BRAIN-REPO-OWNED; T-IDENT-038)",
      "verify": "cd single_task_sprint_fixture && before=$(git rev-parse HEAD) && /but-run-sprint sprint-x 2>&1 | grep -Eq 'but agent register --pid [0-9]+ --as ' && ! grep -rq 'export BUT_AGENT_HANDLE' ~/Projects/brain/skills/but-run-sprint/templates && test \"$(git rev-parse HEAD)\" != \"$before\"",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "e2e",
        "verification_service": "but-run-sprint skill + single-task sprint fixture + but CLI (BRAIN-REPO-OWNED)",
        "start_ref": "single_task_sprint_fixture",
        "must_observe": [
          "orchestrator log shows a `but agent register --pid` `--as` line",
          "new HEAD SHA `!=` before (1 new registry-path commit)"
        ],
        "must_not_observe": [
          "subagent runs `export BUT_AGENT_HANDLE` (start state)",
          "HEAD unchanged / commit denied (0 new commits)"
        ],
        "negative_control": {
          "would_fail_if": [
            "orchestrator never calls `but agent register`",
            "subagent still self-asserts BUT_AGENT_HANDLE",
            "governed commit fails because the registry path is bypassed",
            "greps stubbed or not executed"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "single_task_sprint_fixture",
            "action": {
              "actor": "cli_user",
              "steps": [
                "run /but-run-sprint on the single-task sprint",
                "capture the orchestrator register call",
                "confirm the implementer commit lands",
                "grep templates for export BUT_AGENT_HANDLE"
              ]
            },
            "end_state": {
              "must_observe": [
                "orchestrator log shows a `but agent register --pid` `--as` line",
                "new HEAD SHA `!=` before (1 new registry-path commit)"
              ],
              "must_not_observe": [
                "subagent runs `export BUT_AGENT_HANDLE` (start state)",
                "HEAD unchanged / commit denied (0 new commits)"
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
      "description": "GIVEN the canonical four skills' dispatch templates WHEN grepping for the env-handle export THEN no `export BUT_AGENT_HANDLE` remains in any dispatch template",
      "test_tier": "integration",
      "verification_service": "source (four brain skills' templates)",
      "verify": "! grep -rq 'export BUT_AGENT_HANDLE' ~/Projects/brain/skills/but-run-sprint ~/Projects/brain/skills/but-orchestrate ~/Projects/brain/skills/but-sprint-tasks-plan ~/Projects/brain/skills/but-sprint-plan",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "source (four brain skills' templates)",
        "start_ref": "dispatch_self_identity_skills",
        "must_observe": [
          "grep -r for 'export BUT_AGENT_HANDLE' across the four skills returns 0 matches (exit 1 on grep -q)"
        ],
        "must_not_observe": [
          "any `export BUT_AGENT_HANDLE` line in dispatch templates",
          "self-asserted identity left in but-run-sprint"
        ],
        "negative_control": {
          "would_fail_if": [
            "export removed in only some skills",
            "export left in templates",
            "grep stubbed or not executed"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "dispatch_self_identity_skills",
            "action": {
              "actor": "test_harness",
              "steps": [
                "grep -r 'export BUT_AGENT_HANDLE' across the four skill dirs",
                "assert 0 matches"
              ]
            },
            "end_state": {
              "must_observe": [
                "register-after-spawn present (>=1 `but agent register --pid` match)",
                "dispatch templates carry `but agent register` not `export`"
              ],
              "must_not_observe": [
                "`export BUT_AGENT_HANDLE` present (>0 matches)",
                "handle self-asserted in any skill (none removed)"
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
      "description": "GIVEN the dispatch surfaces (but-run-sprint templates/ + SKILL.md, but-orchestrate spawn-stage.sh + SKILL.md) \u2014 NOT the merge doc WHEN inspecting the post-spawn step THEN the spawn dispatch register `but agent register --pid <child> --as <agent>` is present in the dispatch surfaces (scoped to exclude docs/worktree-lifecycle.md, where the merge self-register lives)",
      "test_tier": "integration",
      "verification_service": "source (but-run-sprint + but-orchestrate dispatch surfaces; merge doc excluded)",
      "verify": "grep -rqE 'but agent register --pid .*--as' ~/Projects/brain/skills/but-run-sprint/templates ~/Projects/brain/skills/but-run-sprint/SKILL.md ~/Projects/brain/skills/but-orchestrate/references/spawn-stage.sh ~/Projects/brain/skills/but-orchestrate/SKILL.md",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "source (but-run-sprint + but-orchestrate dispatch surfaces; merge doc excluded)",
        "start_ref": "dispatch_self_identity_skills",
        "must_observe": [
          "dispatch surfaces (`templates/`, `spawn-stage.sh`, dispatch `SKILL.md`) contain `but agent register --pid` with `--as`",
          "spawn register lives in dispatch surfaces, not `docs/worktree-lifecycle.md`"
        ],
        "must_not_observe": [
          "dispatch register absent from templates (0 matches in dispatch surfaces)",
          "only the merge self-register present (none in dispatch templates)"
        ],
        "negative_control": {
          "would_fail_if": [
            "dispatch register not added to the dispatch surfaces (absent)",
            "register only in `docs/worktree-lifecycle.md` (merge), dispatch templates unchanged",
            "grep stubbed or not executed"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "dispatch_self_identity_skills",
            "action": {
              "actor": "test_harness",
              "steps": [
                "grep -rE 'but agent register --pid .*--as' across the dispatch surfaces only (templates/, spawn-stage.sh, SKILL.md)",
                "confirm the spawn dispatch register exists independently of the merge self-register in docs/worktree-lifecycle.md"
              ]
            },
            "end_state": {
              "must_observe": [
                "dispatch surfaces (`templates/`, `spawn-stage.sh`, dispatch `SKILL.md`) contain `but agent register --pid` with `--as`",
                "spawn register lives in dispatch surfaces, not `docs/worktree-lifecycle.md`"
              ],
              "must_not_observe": [
                "dispatch register absent from templates (0 matches in dispatch surfaces)",
                "only the merge self-register present (none in dispatch templates)"
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
      "description": "GIVEN BUT-SKILL-CONVENTIONS.md (currently \u00a71-\u00a78) WHEN inspecting the new section THEN a new \u00a79 documents that orchestrators register subagents via `but agent register` after spawn and no longer self-assert BUT_AGENT_HANDLE",
      "test_tier": "integration",
      "verification_service": "source (brain/docs/BUT-SKILL-CONVENTIONS.md)",
      "verify": "grep -qE '^## 9' ~/Projects/brain/docs/BUT-SKILL-CONVENTIONS.md && grep -A20 '^## 9' ~/Projects/brain/docs/BUT-SKILL-CONVENTIONS.md | grep -q 'but agent register' && grep -A20 '^## 9' ~/Projects/brain/docs/BUT-SKILL-CONVENTIONS.md | grep -q 'BUT_AGENT_HANDLE'",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "source (brain/docs/BUT-SKILL-CONVENTIONS.md)",
        "start_ref": "dispatch_self_identity_skills",
        "must_observe": [
          "`BUT-SKILL-CONVENTIONS.md` has a `## 9` heading",
          "\u00a79 contains `but agent register` + deprecated `BUT_AGENT_HANDLE`"
        ],
        "must_not_observe": [
          "doc ends at \u00a78 (no `## 9` heading)",
          "\u00a79 omits register model (0 matches)"
        ],
        "negative_control": {
          "would_fail_if": [
            "\u00a79 not added",
            "register model not documented in \u00a79",
            "grep stubbed or not executed"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "dispatch_self_identity_skills",
            "action": {
              "actor": "test_harness",
              "steps": [
                "grep BUT-SKILL-CONVENTIONS.md for '## 9'",
                "grep \u00a79 body for 'but agent register'"
              ]
            },
            "end_state": {
              "must_observe": [
                "`BUT-SKILL-CONVENTIONS.md` has a `## 9` heading",
                "\u00a79 contains `but agent register` + deprecated `BUT_AGENT_HANDLE`"
              ],
              "must_not_observe": [
                "doc ends at \u00a78 (no `## 9` heading)",
                "\u00a79 omits register model (0 matches)"
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
      "description": "GIVEN the four canonical skills and their ~/.claude mirrors WHEN diffing each pair recursively THEN `diff -rq` reports no differences for all four skills",
      "test_tier": "integration",
      "verification_service": "filesystem (brain canonical vs ~/.claude mirrors)",
      "verify": "for s in but-run-sprint but-orchestrate but-sprint-tasks-plan but-sprint-plan; do diff -rq ~/Projects/brain/skills/$s ~/.claude/skills/$s || exit 1; done",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "filesystem (brain canonical vs ~/.claude mirrors)",
        "start_ref": "dispatch_self_identity_skills",
        "must_observe": [
          "diff -rq prints nothing for all four skills (exit 0)",
          "mirrors reflect the register-after-spawn edits"
        ],
        "must_not_observe": [
          "'Files ... differ' output",
          "'Only in' output",
          "any mirror still self-asserting BUT_AGENT_HANDLE"
        ],
        "negative_control": {
          "would_fail_if": [
            "only canonical copies edited (mirrors stale)",
            "diff stubbed or not executed",
            "one of the four mirrors skipped"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "dispatch_self_identity_skills",
            "action": {
              "actor": "test_harness",
              "steps": [
                "loop diff -rq over the four skills",
                "assert empty output / exit 0"
              ]
            },
            "end_state": {
              "must_observe": [
                "`diff -rq` prints `0` lines for all 4 skills",
                "all 4 mirrors contain `but agent register --pid` (count = 4)"
              ],
              "must_not_observe": [
                "`Files ... differ` lines (>0)",
                "a mirror still has `export BUT_AGENT_HANDLE` (start state)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-6",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN the four canonical skills (SKILL bodies, templates, docs/worktree-lifecycle.md, references/spawn-stage.sh) WHEN grepping for any `BUT_AGENT_HANDLE=` assignment and for a merge-scoped `but agent register --pid` co-located with `but merge` THEN no `BUT_AGENT_HANDLE=` assignment remains in any of the four skills, AND the merge step in worktree-lifecycle.md registers the orchestrator's own pid (`but agent register --pid`) within +/-3 lines of `but merge` \u2014 provable independently of the dispatch `--as` register",
      "test_tier": "integration",
      "verification_service": "source (four brain skills; merge self-register scoped to worktree-lifecycle.md)",
      "verify": "! grep -rqE 'BUT_AGENT_HANDLE=' ~/Projects/brain/skills/but-run-sprint ~/Projects/brain/skills/but-orchestrate ~/Projects/brain/skills/but-sprint-plan ~/Projects/brain/skills/but-sprint-tasks-plan && grep -A3 -B3 'but merge' ~/Projects/brain/skills/but-run-sprint/docs/worktree-lifecycle.md | grep -q 'but agent register --pid'",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "source (four brain skills; merge self-register scoped to worktree-lifecycle.md)",
        "start_ref": "dispatch_self_identity_skills",
        "must_observe": [
          "0 `BUT_AGENT_HANDLE=` assignments across the four skills",
          "merge step in `worktree-lifecycle.md` has `but agent register --pid` within +/-3 lines of `but merge`"
        ],
        "must_not_observe": [
          "merge step still self-asserts `BUT_AGENT_HANDLE=` (start state)",
          "merge self-register absent while dispatch register present (none in merge section)"
        ],
        "negative_control": {
          "would_fail_if": [
            "merge step still self-asserts `BUT_AGENT_HANDLE=` (unchanged)",
            "merge self-register removed/absent (only the spawn `--as` register present)",
            "grep stubbed or not executed"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "dispatch_self_identity_skills",
            "action": {
              "actor": "test_harness",
              "steps": [
                "grep -rE 'BUT_AGENT_HANDLE=' across the four skills (expect 0)",
                "grep -A3 -B3 'but merge' worktree-lifecycle.md for a co-located `but agent register --pid` (merge self-register, distinct from dispatch)"
              ]
            },
            "end_state": {
              "must_observe": [
                "0 `BUT_AGENT_HANDLE=` assignments across the four skills",
                "merge step in `worktree-lifecycle.md` has `but agent register --pid` within +/-3 lines of `but merge`"
              ],
              "must_not_observe": [
                "merge step still self-asserts `BUT_AGENT_HANDLE=` (start state)",
                "merge self-register absent while dispatch register present (none in merge section)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-7",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN a registered implementer subagent process in a single-task sprint, BUT_AGENT_HANDLE unset WHEN running `but agent whoami` from inside that process THEN `but agent whoami` prints the registered agent_id (resolved from the runtime registry, no env handle)",
      "test_tier": "e2e",
      "verification_service": "but agent whoami in the registered implementer process (BRAIN-REPO-OWNED; gate step 5)",
      "verify": "cd single_task_sprint_fixture && unset BUT_AGENT_HANDLE && but agent whoami | grep -Eq '[a-z-]+(implementer|reviewer)'",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "e2e",
        "verification_service": "but agent whoami in the registered implementer process (BRAIN-REPO-OWNED)",
        "start_ref": "single_task_sprint_fixture",
        "must_observe": [
          "`but agent whoami` prints a registered agent_id (e.g. `rust-implementer`)",
          "resolved with `BUT_AGENT_HANDLE` unset (1 registry hit)"
        ],
        "must_not_observe": [
          "whoami errors `Denial::unregistered` (0 id)",
          "agent_id only via env handle (none from registry)"
        ],
        "negative_control": {
          "would_fail_if": [
            "the implementer pid was never registered (registry empty)",
            "agent_id resolvable only via a mock/env handle",
            "grep stubbed or not executed"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "single_task_sprint_fixture",
            "action": {
              "actor": "cli_user",
              "steps": [
                "register the implementer child pid via `but agent register`",
                "unset BUT_AGENT_HANDLE in the implementer process",
                "run `but agent whoami` and assert the agent_id"
              ]
            },
            "end_state": {
              "must_observe": [
                "`but agent whoami` prints a registered agent_id (e.g. `rust-implementer`)",
                "resolved with `BUT_AGENT_HANDLE` unset (1 registry hit)"
              ],
              "must_not_observe": [
                "whoami errors `Denial::unregistered` (0 id)",
                "agent_id only via env handle (none from registry)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "Single-task sprint: orchestrator registers the implementer and zero BUT_AGENT_HANDLE refs in dispatch templates is true (BRAIN-REPO-OWNED)",
      "maps_to_ac": "AC-1",
      "verify": "cd single_task_sprint_fixture && before=$(git rev-parse HEAD) && /but-run-sprint sprint-x 2>&1 | grep -Eq 'but agent register --pid [0-9]+ --as ' && test \"$(git rev-parse HEAD)\" != \"$before\""
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "No `export BUT_AGENT_HANDLE` remains in any of the four skills' dispatch templates is true",
      "maps_to_ac": "AC-2",
      "verify": "! grep -rq 'export BUT_AGENT_HANDLE' ~/Projects/brain/skills/but-run-sprint ~/Projects/brain/skills/but-orchestrate ~/Projects/brain/skills/but-sprint-tasks-plan ~/Projects/brain/skills/but-sprint-plan"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "The spawn dispatch register `but agent register --pid ... --as ...` exists in the dispatch surfaces (excluding the merge doc) is true",
      "maps_to_ac": "AC-3",
      "verify": "grep -rqE 'but agent register --pid .*--as' ~/Projects/brain/skills/but-run-sprint/templates ~/Projects/brain/skills/but-run-sprint/SKILL.md ~/Projects/brain/skills/but-orchestrate/references/spawn-stage.sh ~/Projects/brain/skills/but-orchestrate/SKILL.md"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "BUT-SKILL-CONVENTIONS.md \u00a79 documents the register-after-spawn model is true",
      "maps_to_ac": "AC-4",
      "verify": "grep -qE '^## 9' ~/Projects/brain/docs/BUT-SKILL-CONVENTIONS.md && grep -A20 '^## 9' ~/Projects/brain/docs/BUT-SKILL-CONVENTIONS.md | grep -q 'but agent register' && grep -A20 '^## 9' ~/Projects/brain/docs/BUT-SKILL-CONVENTIONS.md | grep -q 'BUT_AGENT_HANDLE'"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "All four canonical skills are byte-identical to their ~/.claude mirrors is true",
      "maps_to_ac": "AC-5",
      "verify": "for s in but-run-sprint but-orchestrate but-sprint-tasks-plan but-sprint-plan; do diff -rq ~/Projects/brain/skills/$s ~/.claude/skills/$s || exit 1; done"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "description": "No `BUT_AGENT_HANDLE=` assignment remains and the merge step self-registers its own pid (co-located with `but merge`) is true",
      "maps_to_ac": "AC-6",
      "verify": "! grep -rqE 'BUT_AGENT_HANDLE=' ~/Projects/brain/skills/but-run-sprint ~/Projects/brain/skills/but-orchestrate ~/Projects/brain/skills/but-sprint-plan ~/Projects/brain/skills/but-sprint-tasks-plan && grep -A3 -B3 'but merge' ~/Projects/brain/skills/but-run-sprint/docs/worktree-lifecycle.md | grep -q 'but agent register --pid'"
    },
    {
      "id": "TC-7",
      "type": "test_criterion",
      "description": "`but agent whoami` in the registered implementer process prints the agent_id with BUT_AGENT_HANDLE unset is true (BRAIN-REPO-OWNED)",
      "maps_to_ac": "AC-7",
      "verify": "cd single_task_sprint_fixture && unset BUT_AGENT_HANDLE && but agent whoami | grep -Eq '[a-z-]+(implementer|reviewer)'"
    }
  ]
}
-->
