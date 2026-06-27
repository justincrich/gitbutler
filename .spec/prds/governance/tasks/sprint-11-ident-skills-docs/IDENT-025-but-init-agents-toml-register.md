# IDENT-025 — `but-init` skill (brain) — `scripts/seed-governance.py` emits `[[agent]]` blocks; step [4] writes `agents.toml`; step [4.6] NEW registers specialists via `but agent register`; acceptance changes `but perm list` → `but agent list --committed`

**Sprint:** [Sprint 11](./SPRINT.md) · **Agent:** `rust-planner` · **Estimate:** 180 min · **Type:** FEATURE · **Status:** Backlog · **Proposed By:** rust-planner

## Background

Per the sprint stub this skill-migration task is owned by rust-planner — it is process/skill-authoring meta-work over the governed `but-*` pipeline (the planning head of that pipeline), not crate code; the planner edits the canonical brain skill + seed script and mirrors them.

**Provides:** (documentation/consumer layer — no new capability)

**Consumes:** agents.toml committed config with [[agent]] blocks (CAP-CONFIG-01), but agent register verb (CAP-AUTHZ-01), but agent list --committed verb

**Boundary contracts:**
- The but-init skill produces .gitbutler/agents.toml (not permissions.toml) and registers each specialist via `but agent register`, matching the engine's agents.toml loader + registry path.

## Critical Constraints

**MUST:**
- Edit the CANONICAL skill at `~/Projects/brain/skills/but-init/` (SKILL.md + scripts/seed-governance.py), then MIRROR to `~/.claude/skills/but-init/`, then verify `diff -rq ~/Projects/brain/skills/but-init ~/.claude/skills/but-init` reports NO differences (both are untracked working copies).
- `scripts/seed-governance.py` must emit `[[agent]]` blocks (not `[[principal]]`) and write `agents.toml` (not `permissions.toml`).
- SKILL.md step [4] writes `.gitbutler/agents.toml`; a NEW step [4.6] registers each specialist via `but agent register` after governance commits.
- Acceptance/`met()` proof changes from `but perm list` to `but agent list --committed`.

**NEVER:**
- Never report the e2e fixture re-run (AC-1) as PASSED inside the gitbutler repo's test suite — that e2e is OWNED BY THE BRAIN REPO's skill tests; the gitbutler repo only asserts the SOURCE-grep contract here.
- Never leave the mirror out of sync with the canonical copy.
- Never write a fake `but agent list --committed` excerpt — the skill must run the real verb against real `but`.

**STRICTLY:**
- BLOCKED-UNTIL Sprint-10 (final env-handle deny-default locked) and transitively Sprint-09 (`but agent` verbs + agents.toml loader landed) — the skill cannot honestly write agents.toml + register specialists until the registry path is the enforced default.
- SCOPE-HONESTY: the PRIMARY e2e (fixture re-run) is real but BRAIN-REPO-OWNED (T-IDENT-036); the supplementary ACs are SOURCE greps provable on the skill files in this machine's brain working copy.

## Specification

**Objective:** Migrate the but-init skill so seed-governance.py emits [[agent]] blocks into agents.toml, SKILL.md step [4] writes agents.toml, a new step [4.6] registers each specialist via `but agent register`, and acceptance uses `but agent list --committed`; canonical and mirror copies kept identical.

**Success state:** Re-running /but-init on a fresh fixture repo commits .gitbutler/agents.toml (no permissions.toml) and `but agent list --committed` shows the roster; on source, seed-governance.py emits [[agent]]+agents.toml, SKILL.md has step [4.6] `but agent register`, acceptance uses `but agent list --committed`, and the canonical/mirror copies diff clean.

## Acceptance Criteria

**AC-1 (PRIMARY)** — Re-run /but-init on a fresh fixture commits agents.toml + roster visible (e2e, BRAIN-REPO-OWNED)
- **GIVEN:** a fresh fixture repo with a specialist roster and no governance config
- **WHEN:** running the migrated /but-init skill end-to-end against it
- **THEN:** .gitbutler/agents.toml is committed at the target ref (no permissions.toml written) and `but agent list --committed` prints the full specialist roster
- **Verify:** `cd "$(mktemp -d)" && git init -q fixture && cd fixture && cp "$FIXTURE_ROSTER" RULES.md && git add -A && git commit -qm init && /but-init --no-harness && git show HEAD:.gitbutler/agents.toml | grep -q '\[\[agent\]\]' && ! git cat-file -e HEAD:.gitbutler/permissions.toml 2>/dev/null && but agent list --committed | grep -Eq 'rust-implementer|rust-reviewer'`
- **TEST_TIER:** e2e · **VERIFICATION_SERVICE:** but-init skill + fresh fixture repo + but CLI (BRAIN-REPO-OWNED; T-IDENT-036) · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=fresh_fixture_repo`; must_observe = [committed .gitbutler/agents.toml containing '[[agent]]'; `but agent list --committed` lists 'rust-implementer' and 'rust-reviewer']; must_not_observe = [permissions.toml written/committed; empty roster output; agents.toml absent at HEAD]; negative_control.would_fail_if = [skill still writes permissions.toml; skill commits no governance file; but agent list --committed returns empty (registry/loader disconnected); roster greps stubbed or not executed].

**AC-2** — seed-governance.py emits [[agent]] and writes agents.toml
- **GIVEN:** the canonical seed-governance.py
- **WHEN:** inspecting its emitter + output filename
- **THEN:** the script emits `[[agent]]` blocks and writes `agents.toml` (not `[[principal]]`/`permissions.toml`)
- **Verify:** `grep -q '\[\[agent\]\]' ~/Projects/brain/skills/but-init/scripts/seed-governance.py && grep -q 'agents.toml' ~/Projects/brain/skills/but-init/scripts/seed-governance.py && ! grep -q '\[\[principal\]\]' ~/Projects/brain/skills/but-init/scripts/seed-governance.py`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** source (brain/skills/but-init/scripts/seed-governance.py) · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=but_init_legacy_skill`; must_observe = [script contains literal '[[agent]]'; script writes 'agents.toml']; must_not_observe = ['[[principal]]' still emitted; 'permissions.toml' still the output; unchanged script]; negative_control.would_fail_if = [emitter not switched to [[agent]]; output file not renamed to agents.toml; grep stubbed or not executed; legacy [[principal]] left in place].

**AC-3** — SKILL.md step [4] writes agents.toml
- **GIVEN:** the canonical but-init SKILL.md
- **WHEN:** inspecting step [4]
- **THEN:** step [4] writes `.gitbutler/agents.toml` (not permissions.toml)
- **Verify:** `grep -q '.gitbutler/agents.toml' ~/Projects/brain/skills/but-init/SKILL.md`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** source (brain/skills/but-init/SKILL.md) · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=but_init_legacy_skill`; must_observe = [SKILL.md contains literal '.gitbutler/agents.toml'; step [4] writes the committed agents.toml]; must_not_observe = [step [4] still writes permissions.toml only; no agents.toml mention; unchanged SKILL.md]; negative_control.would_fail_if = [step [4] not updated to agents.toml; agents.toml literal absent; grep stubbed or not executed].

**AC-4** — SKILL.md NEW step [4.6] registers specialists via `but agent register`
- **GIVEN:** the canonical but-init SKILL.md
- **WHEN:** inspecting the post-commit registration step
- **THEN:** a new step [4.6] registers each specialist via `but agent register` after governance commits
- **Verify:** `grep -q '\[4.6\]' ~/Projects/brain/skills/but-init/SKILL.md && grep -A6 '\[4.6\]' ~/Projects/brain/skills/but-init/SKILL.md | grep -q 'but agent register'`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** source (brain/skills/but-init/SKILL.md) · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=but_init_legacy_skill`; must_observe = [SKILL.md contains step '[4.6]'; step [4.6] body contains 'but agent register']; must_not_observe = [no [4.6] step; registration absent; unchanged SKILL.md]; negative_control.would_fail_if = [step [4.6] not added; 'but agent register' not in the step; grep stubbed or not executed].

**AC-5** — Acceptance proof uses `but agent list --committed`
- **GIVEN:** the canonical but-init SKILL.md acceptance/met()
- **WHEN:** inspecting the governance proof verb
- **THEN:** acceptance uses `but agent list --committed` AND no `but perm list` remains (replacement, not supplement)
- **Verify:** `grep -q 'but agent list --committed' ~/Projects/brain/skills/but-init/SKILL.md && ! grep -q 'but perm list' ~/Projects/brain/skills/but-init/SKILL.md`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** source (brain/skills/but-init/SKILL.md) · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=but_init_legacy_skill`; must_observe = [acceptance contains `but agent list --committed`; no `but perm list` remains (replacement, not supplement)]; must_not_observe = [`but perm list` still present (0 removed); agent-list verb absent (none)]; negative_control.would_fail_if = [acceptance not switched to `but agent list --committed`; verb literal absent; grep stubbed or not executed].

**AC-6** — Canonical and mirror copies are identical
- **GIVEN:** the canonical but-init skill and its ~/.claude mirror
- **WHEN:** diffing the two trees recursively
- **THEN:** `diff -rq` reports no differences
- **Verify:** `diff -rq ~/Projects/brain/skills/but-init ~/.claude/skills/but-init`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** filesystem (brain canonical vs ~/.claude mirror) · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=but_init_legacy_skill`; must_observe = [diff -rq prints nothing (exit 0); mirror reflects [[agent]]/agents.toml/[4.6] edits]; must_not_observe = ['Files ... differ' output; 'Only in' output; mirror still on legacy permissions.toml]; negative_control.would_fail_if = [only the canonical copy edited (mirror stale); diff stubbed or not executed; mirror not refreshed after edits].

## Test Criteria

| ID | Boolean | Maps to AC |
|----|---------|------------|
| TC-1 | /but-init on a fresh fixture commits agents.toml and `but agent list --committed` shows the roster is true (BRAIN-REPO-OWNED) | AC-1 |
| TC-2 | seed-governance.py emits [[agent]] + writes agents.toml and emits no [[principal]] is true | AC-2 |
| TC-3 | SKILL.md step [4] writes .gitbutler/agents.toml is true | AC-3 |
| TC-4 | SKILL.md step [4.6] calls `but agent register` is true | AC-4 |
| TC-5 | Acceptance uses `but agent list --committed` is true | AC-5 |
| TC-6 | Canonical and ~/.claude mirror copies are byte-identical is true | AC-6 |
| TC-7 | `but perm list` is fully removed from but-init SKILL.md (replacement, not supplement) is true | AC-5 |

## Reading List

1. `/Users/justinrich/Projects/brain/skills/but-init/scripts/seed-governance.py`:71-192 — PRIMARY PATTERN — build_permissions_toml emitter (line ~94 [[principal]], line ~115 permissions.toml) + the post-write proof; switch to [[agent]] + agents.toml.
2. `/Users/justinrich/Projects/brain/skills/but-init/SKILL.md`:120-235 — Step [4] config write + commit (lines ~153,197) and met() proof (line ~231); add step [4.6] register + switch met() to `but agent list --committed`.
3. `/Users/justinrich/Projects/gitbutler/.spec/prds/governance/12-uc-agent-identity.md`:66-92 — UC-IDENT-04/05 — `but agent register`/`list --committed`/`migrate` verbs the skill must invoke.
4. `/Users/justinrich/Projects/brain/docs/BUT-SKILL-CONVENTIONS.md`:21-49 — §2 identity & governance model + §6 artifacts/paths the skill must honor.
5. `/Users/justinrich/Projects/gitbutler/.spec/prds/governance/tasks/sprint-10-ident-deprecation-hardening/IDENT-021-gate-callsite-doc-audit.md`:51-114 — Exemplar grep-based source-presence AC pattern.

## Guardrails

**WRITE-ALLOWED:**
- /Users/justinrich/Projects/brain/skills/but-init/scripts/seed-governance.py (MODIFY — emit [[agent]] + write agents.toml)
- /Users/justinrich/Projects/brain/skills/but-init/SKILL.md (MODIFY — step [4] agents.toml, NEW step [4.6] register, met() -> `but agent list --committed`)
- /Users/justinrich/.claude/skills/but-init/scripts/seed-governance.py (MODIFY — mirror)
- /Users/justinrich/.claude/skills/but-init/SKILL.md (MODIFY — mirror)

**WRITE-PROHIBITED:**
- /Users/justinrich/Projects/gitbutler/** - this skill task edits brain skills, not the gitbutler repo source
- crates/but-authz/** - the engine is upstream (Sprint 08-10); the skill only consumes its verbs
- other brain skills (but-migrate/but-run-sprint) - separate tasks (IDENT-026/027)

## Code Pattern

**Reference:** /Users/justinrich/Projects/brain/skills/but-init/scripts/seed-governance.py, /Users/justinrich/Projects/brain/skills/but-init/SKILL.md, 12-uc-agent-identity.md UC-IDENT-04/05

**Pattern:** Skill migration: rename the seed emitter's block header [[principal]]->[[agent]] and output filename permissions.toml->agents.toml; SKILL.md step [4] commits .gitbutler/agents.toml; add a NEW step [4.6] that, after the governance commit, loops the roster and runs `but agent register --pid <child_pid> --as <agent_id>` (or registers committed specialists per UC-IDENT-04); met() proof greps `but agent list --committed` for the roster instead of `but perm list`.

**Source:** `Existing but-init seed-governance.py + SKILL.md (same flow, renamed config + added registration step) — and UC-IDENT-04's `but agent` verb shape modeled on `but perm`.`

**Design notes:**
- Edit canonical -> mirror -> diff -rq clean is mandatory and is its own AC (AC-6).

**Anti-pattern:** Do NOT fabricate a `but agent list --committed` excerpt; do NOT leave [[principal]]/permissions.toml emitters in place; do NOT edit only the canonical copy and skip the mirror; do NOT claim the e2e fixture re-run passes inside the gitbutler repo's test suite (it is brain-repo-owned).

## Agent Instructions

TDD RED→GREEN per AC (build-gate/source greps + real `but`/skill execution against real files — NO mocks):
1. **RED:** write each AC's failing check first (against the current start state — docs/skill absent or still self-asserting `BUT_AGENT_HANDLE`).
2. **GREEN:** make the minimal edit (docs/skill/migration) to satisfy the AC.
3. For brain-skill tasks: edit the CANONICAL copy under `~/Projects/brain/skills/`, MIRROR to `~/.claude/skills/`, then `diff -rq` clean.
4. Run each AC's verify command; commit via `but commit` (governed).

## Orchestrator Verification Protocol

- `grep -q '\[\[agent\]\]' ~/Projects/brain/skills/but-init/scripts/seed-governance.py && grep -q '\[4.6\]' ~/Projects/brain/skills/but-init/SKILL.md && grep -q 'but agent list --committed' ~/Projects/brain/skills/but-init/SKILL.md` → exit 0
- `! grep -q '\[\[principal\]\]' ~/Projects/brain/skills/but-init/scripts/seed-governance.py` → exit 0
- `diff -rq ~/Projects/brain/skills/but-init ~/.claude/skills/but-init` → no output (exit 0)
- `re-run /but-init against a fresh fixture; git show HEAD:.gitbutler/agents.toml | grep '[[agent]]'; but agent list --committed` → agents.toml committed + roster listed (run where the skill lives; not the gitbutler test suite)

## Agent Assignment

**Agent:** `rust-planner` — Per the sprint stub this skill-migration task is owned by rust-planner — it is process/skill-authoring meta-work over the governed `but-*` pipeline (the planning head of that pipeline), not crate code; the planner edits the canonical brain skill + seed script and mirrors them.
**Pairing:** none (single-surface Rust/docs/skill task). Honors `crates/AGENTS.md` + `BUT-SKILL-CONVENTIONS.md`.

## Evidence Gates

- `grep -q '\[\[agent\]\]' ~/Projects/brain/skills/but-init/scripts/seed-governance.py && grep -q '\[4.6\]' ~/Projects/brain/skills/but-init/SKILL.md && grep -q 'but agent list --committed' ~/Projects/brain/skills/but-init/SKILL.md` (exit 0)
- `! grep -q '\[\[principal\]\]' ~/Projects/brain/skills/but-init/scripts/seed-governance.py` (exit 0)
- `diff -rq ~/Projects/brain/skills/but-init ~/.claude/skills/but-init` (no output (exit 0))
- `re-run /but-init against a fresh fixture; git show HEAD:.gitbutler/agents.toml | grep '[[agent]]'; but agent list --committed` (agents.toml committed + roster listed (run where the skill lives; not the gitbutler test suite))

## Review Criteria

- AC-1: PRIMARY — Re-run /but-init on a fresh fixture commits agents.toml + roster visible (e2e, BRAIN-REPO-OWNED) — verified by `cd "$(mktemp -d)" && git init -q fixture && cd fixture && cp "$FIXTURE_ROSTER" RULES.md && git add -A && git commit -qm init && /but-init --no-harness && git show HEAD:.gitbutler/agents.toml | grep -q '\[\[agent\]\]' && ! git cat-file -e HEAD:.gitbutler/permissions.toml 2>/dev/null && but agent list --committed | grep -Eq 'rust-implementer|rust-reviewer'`.
- AC-2: seed-governance.py emits [[agent]] and writes agents.toml — verified by `grep -q '\[\[agent\]\]' ~/Projects/brain/skills/but-init/scripts/seed-governance.py && grep -q 'agents.toml' ~/Projects/brain/skills/but-init/scripts/seed-governance.py && ! grep -q '\[\[principal\]\]' ~/Projects/brain/skills/but-init/scripts/seed-governance.py`.
- AC-3: SKILL.md step [4] writes agents.toml — verified by `grep -q '.gitbutler/agents.toml' ~/Projects/brain/skills/but-init/SKILL.md`.
- AC-4: SKILL.md NEW step [4.6] registers specialists via `but agent register` — verified by `grep -q '\[4.6\]' ~/Projects/brain/skills/but-init/SKILL.md && grep -A6 '\[4.6\]' ~/Projects/brain/skills/but-init/SKILL.md | grep -q 'but agent register'`.
- AC-5: Acceptance proof uses `but agent list --committed` — verified by `grep -q 'but agent list --committed' ~/Projects/brain/skills/but-init/SKILL.md && ! grep -q 'but perm list' ~/Projects/brain/skills/but-init/SKILL.md`.
- AC-6: Canonical and mirror copies are identical — verified by `diff -rq ~/Projects/brain/skills/but-init ~/.claude/skills/but-init`.
- Honors NEVER: Never report the e2e fixture re-run (AC-1) as PASSED inside the gitbutler repo's test suite — that e2e is OWNED BY THE BRAIN REPO's skill tests; the gitbutler repo only asserts the SOURCE-grep contract here.

## Dependencies

- **Depends on:** none (BLOCKED-UNTIL Sprint-10 per Critical Constraints)
- **Blocks:** none
- **Capabilities:** CAP-AUTHZ-01, CAP-CONFIG-01

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "IDENT-025",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "but_init_legacy_skill": {
      "description": "but-init canonical skill at ~/Projects/brain/skills/but-init/ where seed-governance.py emits [[principal]] + writes permissions.toml and SKILL.md acceptance uses `but perm list`",
      "seed_method": "public_api",
      "records": [
        "scripts/seed-governance.py build_permissions_toml emits '[[principal]]' (line ~94) and writes permissions.toml (line ~115)",
        "SKILL.md step [4]/commit writes .gitbutler/permissions.toml (lines ~153,197); met() uses `but perm list`/`but group list` (line ~231)",
        "No step [4.6]; no `but agent register`; no agents.toml"
      ]
    },
    "fresh_fixture_repo": {
      "description": "a freshly git-init'd fixture repo with a specialist roster in RULES.md and no .gitbutler/ governance yet, registered as a GitButler project",
      "seed_method": "cli",
      "records": [
        "git repo with RULES.md specialist table (rust-implementer, rust-reviewer, orchestrator)",
        "no .gitbutler/agents.toml or permissions.toml at HEAD"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN a fresh fixture repo with a specialist roster and no governance config WHEN running the migrated /but-init skill end-to-end against it THEN .gitbutler/agents.toml is committed at the target ref (no permissions.toml written) and `but agent list --committed` prints the full specialist roster",
      "test_tier": "e2e",
      "verification_service": "but-init skill + fresh fixture repo + but CLI (BRAIN-REPO-OWNED; T-IDENT-036)",
      "verify": "cd \"$(mktemp -d)\" && git init -q fixture && cd fixture && cp \"$FIXTURE_ROSTER\" RULES.md && git add -A && git commit -qm init && /but-init --no-harness && git show HEAD:.gitbutler/agents.toml | grep -q '\\[\\[agent\\]\\]' && ! git cat-file -e HEAD:.gitbutler/permissions.toml 2>/dev/null && but agent list --committed | grep -Eq 'rust-implementer|rust-reviewer'",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "e2e",
        "verification_service": "but-init skill + fresh fixture repo + but CLI (BRAIN-REPO-OWNED)",
        "start_ref": "fresh_fixture_repo",
        "must_observe": [
          "committed .gitbutler/agents.toml containing '[[agent]]'",
          "`but agent list --committed` lists 'rust-implementer' and 'rust-reviewer'"
        ],
        "must_not_observe": [
          "permissions.toml written/committed",
          "empty roster output",
          "agents.toml absent at HEAD"
        ],
        "negative_control": {
          "would_fail_if": [
            "skill still writes permissions.toml",
            "skill commits no governance file",
            "but agent list --committed returns empty (registry/loader disconnected)",
            "roster greps stubbed or not executed"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "fresh_fixture_repo",
            "action": {
              "actor": "cli_user",
              "steps": [
                "run /but-init against the fixture",
                "git show HEAD:.gitbutler/agents.toml",
                "but agent list --committed"
              ]
            },
            "end_state": {
              "must_observe": [
                "`git show HEAD:.gitbutler/agents.toml` contains `[[agent]]`",
                "`but agent list --committed` prints `rust-implementer`"
              ],
              "must_not_observe": [
                "`permissions.toml` committed (start state)",
                "empty roster (0 agents)"
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
      "description": "GIVEN the canonical seed-governance.py WHEN inspecting its emitter + output filename THEN the script emits `[[agent]]` blocks and writes `agents.toml` (not `[[principal]]`/`permissions.toml`)",
      "test_tier": "integration",
      "verification_service": "source (brain/skills/but-init/scripts/seed-governance.py)",
      "verify": "grep -q '\\[\\[agent\\]\\]' ~/Projects/brain/skills/but-init/scripts/seed-governance.py && grep -q 'agents.toml' ~/Projects/brain/skills/but-init/scripts/seed-governance.py && ! grep -q '\\[\\[principal\\]\\]' ~/Projects/brain/skills/but-init/scripts/seed-governance.py",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "source (brain/skills/but-init/scripts/seed-governance.py)",
        "start_ref": "but_init_legacy_skill",
        "must_observe": [
          "script contains literal '[[agent]]'",
          "script writes 'agents.toml'"
        ],
        "must_not_observe": [
          "'[[principal]]' still emitted",
          "'permissions.toml' still the output",
          "unchanged script"
        ],
        "negative_control": {
          "would_fail_if": [
            "emitter not switched to [[agent]]",
            "output file not renamed to agents.toml",
            "grep stubbed or not executed",
            "legacy [[principal]] left in place"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "but_init_legacy_skill",
            "action": {
              "actor": "test_harness",
              "steps": [
                "grep seed-governance.py for '[[agent]]' + 'agents.toml'",
                "assert no '[[principal]]'"
              ]
            },
            "end_state": {
              "must_observe": [
                "seed-governance.py emits `[[agent]]`",
                "writes `agents.toml`"
              ],
              "must_not_observe": [
                "`[[principal]]` still emitted (start state)",
                "no `agents.toml` output (0 matches)"
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
      "description": "GIVEN the canonical but-init SKILL.md WHEN inspecting step [4] THEN step [4] writes `.gitbutler/agents.toml` (not permissions.toml)",
      "test_tier": "integration",
      "verification_service": "source (brain/skills/but-init/SKILL.md)",
      "verify": "grep -q '.gitbutler/agents.toml' ~/Projects/brain/skills/but-init/SKILL.md",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "source (brain/skills/but-init/SKILL.md)",
        "start_ref": "but_init_legacy_skill",
        "must_observe": [
          "SKILL.md contains literal '.gitbutler/agents.toml'",
          "step [4] writes the committed agents.toml"
        ],
        "must_not_observe": [
          "step [4] still writes permissions.toml only",
          "no agents.toml mention",
          "unchanged SKILL.md"
        ],
        "negative_control": {
          "would_fail_if": [
            "step [4] not updated to agents.toml",
            "agents.toml literal absent",
            "grep stubbed or not executed"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "but_init_legacy_skill",
            "action": {
              "actor": "test_harness",
              "steps": [
                "grep SKILL.md for '.gitbutler/agents.toml'"
              ]
            },
            "end_state": {
              "must_observe": [
                "SKILL.md step `[4]` writes `.gitbutler/agents.toml`",
                "names committed `agents.toml`"
              ],
              "must_not_observe": [
                "step writes `permissions.toml` only (start state)",
                "no `agents.toml` mention (0 matches)"
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
      "description": "GIVEN the canonical but-init SKILL.md WHEN inspecting the post-commit registration step THEN a new step [4.6] registers each specialist via `but agent register` after governance commits",
      "test_tier": "integration",
      "verification_service": "source (brain/skills/but-init/SKILL.md)",
      "verify": "grep -q '\\[4.6\\]' ~/Projects/brain/skills/but-init/SKILL.md && grep -A6 '\\[4.6\\]' ~/Projects/brain/skills/but-init/SKILL.md | grep -q 'but agent register'",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "source (brain/skills/but-init/SKILL.md)",
        "start_ref": "but_init_legacy_skill",
        "must_observe": [
          "SKILL.md contains step '[4.6]'",
          "step [4.6] body contains 'but agent register'"
        ],
        "must_not_observe": [
          "no [4.6] step",
          "registration absent",
          "unchanged SKILL.md"
        ],
        "negative_control": {
          "would_fail_if": [
            "step [4.6] not added",
            "'but agent register' not in the step",
            "grep stubbed or not executed"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "but_init_legacy_skill",
            "action": {
              "actor": "test_harness",
              "steps": [
                "grep SKILL.md for '[4.6]'",
                "grep the step body for 'but agent register'"
              ]
            },
            "end_state": {
              "must_observe": [
                "SKILL.md contains step `[4.6]`",
                "step body contains `but agent register`"
              ],
              "must_not_observe": [
                "step `[4.6]` absent (0 matches)",
                "registration missing (none)"
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
      "description": "GIVEN the canonical but-init SKILL.md acceptance/met() WHEN inspecting the governance proof verb THEN acceptance uses `but agent list --committed` AND no `but perm list` remains (replacement, not supplement)",
      "test_tier": "integration",
      "verification_service": "source (brain/skills/but-init/SKILL.md)",
      "verify": "grep -q 'but agent list --committed' ~/Projects/brain/skills/but-init/SKILL.md && ! grep -q 'but perm list' ~/Projects/brain/skills/but-init/SKILL.md",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "source (brain/skills/but-init/SKILL.md)",
        "start_ref": "but_init_legacy_skill",
        "must_observe": [
          "acceptance contains `but agent list --committed`",
          "no `but perm list` remains (replacement, not supplement)"
        ],
        "must_not_observe": [
          "`but perm list` still present (0 removed)",
          "agent-list verb absent (none)"
        ],
        "negative_control": {
          "would_fail_if": [
            "acceptance not switched to `but agent list --committed`",
            "verb literal absent",
            "grep stubbed or not executed"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "but_init_legacy_skill",
            "action": {
              "actor": "test_harness",
              "steps": [
                "grep SKILL.md for 'but agent list --committed'"
              ]
            },
            "end_state": {
              "must_observe": [
                "acceptance contains `but agent list --committed`",
                "no `but perm list` remains (replacement, not supplement)"
              ],
              "must_not_observe": [
                "`but perm list` still present (0 removed)",
                "agent-list verb absent (none)"
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
      "description": "GIVEN the canonical but-init skill and its ~/.claude mirror WHEN diffing the two trees recursively THEN `diff -rq` reports no differences",
      "test_tier": "integration",
      "verification_service": "filesystem (brain canonical vs ~/.claude mirror)",
      "verify": "diff -rq ~/Projects/brain/skills/but-init ~/.claude/skills/but-init",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "filesystem (brain canonical vs ~/.claude mirror)",
        "start_ref": "but_init_legacy_skill",
        "must_observe": [
          "diff -rq prints nothing (exit 0)",
          "mirror reflects [[agent]]/agents.toml/[4.6] edits"
        ],
        "must_not_observe": [
          "'Files ... differ' output",
          "'Only in' output",
          "mirror still on legacy permissions.toml"
        ],
        "negative_control": {
          "would_fail_if": [
            "only the canonical copy edited (mirror stale)",
            "diff stubbed or not executed",
            "mirror not refreshed after edits"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "but_init_legacy_skill",
            "action": {
              "actor": "test_harness",
              "steps": [
                "run diff -rq canonical mirror",
                "assert empty output / exit 0"
              ]
            },
            "end_state": {
              "must_observe": [
                "`diff -rq` prints `0` lines for `but-init`",
                "mirror contains `[[agent]]` + step `[4.6]` matching canonical"
              ],
              "must_not_observe": [
                "`Files ... differ` lines (>0)",
                "mirror still `permissions.toml` (start state)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "/but-init on a fresh fixture commits agents.toml and `but agent list --committed` shows the roster is true (BRAIN-REPO-OWNED)",
      "maps_to_ac": "AC-1",
      "verify": "git show HEAD:.gitbutler/agents.toml | grep -q '\\[\\[agent\\]\\]' && but agent list --committed | grep -Eq 'rust-implementer|rust-reviewer'"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "seed-governance.py emits [[agent]] + writes agents.toml and emits no [[principal]] is true",
      "maps_to_ac": "AC-2",
      "verify": "grep -q '\\[\\[agent\\]\\]' ~/Projects/brain/skills/but-init/scripts/seed-governance.py && ! grep -q '\\[\\[principal\\]\\]' ~/Projects/brain/skills/but-init/scripts/seed-governance.py"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "SKILL.md step [4] writes .gitbutler/agents.toml is true",
      "maps_to_ac": "AC-3",
      "verify": "grep -q '.gitbutler/agents.toml' ~/Projects/brain/skills/but-init/SKILL.md"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "SKILL.md step [4.6] calls `but agent register` is true",
      "maps_to_ac": "AC-4",
      "verify": "grep -A6 '\\[4.6\\]' ~/Projects/brain/skills/but-init/SKILL.md | grep -q 'but agent register'"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "Acceptance uses `but agent list --committed` is true",
      "maps_to_ac": "AC-5",
      "verify": "grep -q 'but agent list --committed' ~/Projects/brain/skills/but-init/SKILL.md"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "description": "Canonical and ~/.claude mirror copies are byte-identical is true",
      "maps_to_ac": "AC-6",
      "verify": "diff -rq ~/Projects/brain/skills/but-init ~/.claude/skills/but-init"
    },
    {
      "id": "TC-7",
      "type": "test_criterion",
      "description": "`but perm list` is fully removed from but-init SKILL.md (replacement, not supplement) is true",
      "maps_to_ac": "AC-5",
      "verify": "! grep -q 'but perm list' ~/Projects/brain/skills/but-init/SKILL.md"
    }
  ]
}
-->
