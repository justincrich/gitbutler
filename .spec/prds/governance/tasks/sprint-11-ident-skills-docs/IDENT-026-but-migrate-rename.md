# IDENT-026 — `but-migrate` skill (brain) — detect `permissions.toml`, run `but agent migrate`, commit the rename; idempotent no-op once `agents.toml` exists

**Sprint:** [Sprint 11](./SPRINT.md) · **Agent:** `rust-planner` · **Estimate:** 120 min · **Type:** FEATURE · **Status:** Backlog · **Proposed By:** rust-planner

## Background

Per the sprint stub the but-migrate skill-migration is owned by rust-planner as governed-pipeline process authoring; the planner edits the canonical brain skill + mirror to wire the `but agent migrate` rename flow.

**Provides:** (documentation/consumer layer — no new capability)

**Consumes:** but agent migrate verb (permissions.toml -> agents.toml rewrite), agents.toml committed config (CAP-CONFIG-01)

**Boundary contracts:**
- The but-migrate skill detects a committed permissions.toml, runs `but agent migrate`, and commits the agents.toml write + permissions.toml delete together, idempotently — matching the engine's `but agent migrate` byte-equivalent round-trip.

## Critical Constraints

**MUST:**
- Edit the CANONICAL skill at `~/Projects/brain/skills/but-migrate/SKILL.md`, MIRROR to `~/.claude/skills/but-migrate/SKILL.md`, then verify `diff -rq` reports no differences.
- The skill must detect `.gitbutler/permissions.toml` and run `but agent migrate` to perform the rename.
- The skill must commit the agents.toml write + permissions.toml delete together (one commit).
- The skill must be idempotent: a re-run once `agents.toml` exists is a clean no-op.

**NEVER:**
- Never report the e2e fixture migration (AC-1) as PASSED inside the gitbutler repo's test suite — it is OWNED BY THE BRAIN REPO's skill tests (T-IDENT-037); the gitbutler repo asserts the SOURCE-grep contract.
- Never hand-edit the toml rename instead of invoking `but agent migrate`.
- Never leave the mirror out of sync with the canonical copy.

**STRICTLY:**
- BLOCKED-UNTIL Sprint-10 (final env-handle deny-default locked) and transitively Sprint-09 (`but agent migrate` verb landed).
- SCOPE-HONESTY: PRIMARY e2e (fixture migration) is BRAIN-REPO-OWNED; supplementary ACs are SOURCE greps on the skill file.

## Specification

**Objective:** Migrate the but-migrate skill to detect a committed permissions.toml, run `but agent migrate`, commit the agents.toml-write + permissions.toml-delete in one commit, and be a clean no-op when agents.toml already exists; canonical and mirror kept identical.

**Success state:** Running /but-migrate against a permissions.toml-only fixture writes agents.toml + deletes permissions.toml in one commit; a re-run is a no-op; on source, SKILL.md detects permissions.toml, invokes `but agent migrate`, documents idempotency, and canonical/mirror diff clean.

## Acceptance Criteria

**AC-1 (PRIMARY)** — /but-migrate converts permissions.toml -> agents.toml in one commit; re-run is no-op (e2e, BRAIN-REPO-OWNED)
- **GIVEN:** a fixture repo with a committed permissions.toml and no agents.toml
- **WHEN:** running the migrated /but-migrate skill end-to-end
- **THEN:** agents.toml is written and permissions.toml is removed in the same commit, and a second run makes no change
- **Verify:** `cd permissions_only_fixture && /but-migrate && git show HEAD:.gitbutler/agents.toml | grep -q '\[\[agent\]\]' && ! git cat-file -e HEAD:.gitbutler/permissions.toml 2>/dev/null && before=$(git rev-parse HEAD) && /but-migrate && test "$before" = "$(git rev-parse HEAD)"`
- **TEST_TIER:** e2e · **VERIFICATION_SERVICE:** but-migrate skill + permissions.toml fixture repo + but CLI (BRAIN-REPO-OWNED; T-IDENT-037) · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=permissions_only_fixture`; must_observe = [committed .gitbutler/agents.toml with '[[agent]]'; permissions.toml absent at HEAD after migration; second run leaves HEAD unchanged (same SHA)]; must_not_observe = [permissions.toml still present at HEAD; agents.toml absent; second run creates a new commit (not idempotent)]; negative_control.would_fail_if = [skill does not invoke `but agent migrate`; rename not committed; permissions.toml not deleted in the same commit; re-run mutates HEAD (idempotency broken)].

**AC-2** — SKILL.md invokes `but agent migrate`
- **GIVEN:** the canonical but-migrate SKILL.md
- **WHEN:** inspecting the migration flow
- **THEN:** SKILL.md invokes the `but agent migrate` verb (not a hand-edited toml rename)
- **Verify:** `grep -q 'but agent migrate' ~/Projects/brain/skills/but-migrate/SKILL.md`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** source (brain/skills/but-migrate/SKILL.md) · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=but_migrate_legacy_skill`; must_observe = [SKILL.md contains literal 'but agent migrate']; must_not_observe = [hand-rolled toml rename instead of the verb; no migrate invocation; unchanged SKILL.md]; negative_control.would_fail_if = [`but agent migrate` not added; rename done by manual mv/sed in the skill; grep stubbed or not executed].

**AC-3** — SKILL.md detects committed permissions.toml and commits the rename
- **GIVEN:** the canonical but-migrate SKILL.md
- **WHEN:** inspecting the detection + commit step
- **THEN:** SKILL.md detects `.gitbutler/permissions.toml` and commits the agents.toml write + permissions.toml delete together
- **Verify:** `grep -q '.gitbutler/permissions.toml' ~/Projects/brain/skills/but-migrate/SKILL.md && grep -A8 'but agent migrate' ~/Projects/brain/skills/but-migrate/SKILL.md | grep -qiE 'commit'`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** source (brain/skills/but-migrate/SKILL.md) · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=but_migrate_legacy_skill`; must_observe = [SKILL.md detects '.gitbutler/permissions.toml'; documents committing the rename (add agents.toml + delete permissions.toml)]; must_not_observe = [no detection of legacy file; rename not committed; unchanged SKILL.md]; negative_control.would_fail_if = [detection step omitted; commit-the-rename step omitted; grep stubbed or not executed].

**AC-4** — SKILL.md documents idempotent no-op once agents.toml exists
- **GIVEN:** the canonical but-migrate SKILL.md
- **WHEN:** inspecting the idempotency contract
- **THEN:** SKILL.md states a re-run is a clean no-op once `agents.toml` exists
- **Verify:** `grep -iE 'idempotent|no-op' ~/Projects/brain/skills/but-migrate/SKILL.md && grep -q 'agents.toml' ~/Projects/brain/skills/but-migrate/SKILL.md`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** source (brain/skills/but-migrate/SKILL.md) · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=but_migrate_legacy_skill`; must_observe = [SKILL.md states the migrate re-run is idempotent / no-op once agents.toml exists; literal 'agents.toml' present]; must_not_observe = [no idempotency statement for the migrate flow; unchanged SKILL.md]; negative_control.would_fail_if = [idempotency contract not documented for the rename; grep stubbed or not executed].

**AC-5** — Canonical and mirror copies are identical
- **GIVEN:** the canonical but-migrate skill and its ~/.claude mirror
- **WHEN:** diffing the two trees recursively
- **THEN:** `diff -rq` reports no differences
- **Verify:** `diff -rq ~/Projects/brain/skills/but-migrate ~/.claude/skills/but-migrate`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** filesystem (brain canonical vs ~/.claude mirror) · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=but_migrate_legacy_skill`; must_observe = [diff -rq prints nothing (exit 0); mirror reflects the `but agent migrate` edits]; must_not_observe = ['Files ... differ' output; 'Only in' output; mirror still legacy]; negative_control.would_fail_if = [only the canonical copy edited (mirror stale); diff stubbed or not executed].

## Test Criteria

| ID | Boolean | Maps to AC |
|----|---------|------------|
| TC-1 | /but-migrate writes agents.toml + removes permissions.toml in one commit and re-run is a no-op is true (BRAIN-REPO-OWNED) | AC-1 |
| TC-2 | SKILL.md invokes `but agent migrate` is true | AC-2 |
| TC-3 | SKILL.md detects .gitbutler/permissions.toml and commits the rename is true | AC-3 |
| TC-4 | SKILL.md documents idempotent no-op once agents.toml exists is true | AC-4 |
| TC-5 | Canonical and ~/.claude mirror copies are byte-identical is true | AC-5 |

## Reading List

1. `/Users/justinrich/Projects/brain/skills/but-migrate/SKILL.md`:54-163 — PRIMARY PATTERN — additive outputs + reconcile/idempotency block (lines ~54,118-163); add detect-permissions.toml + `but agent migrate` + commit-the-rename, preserving the idempotency contract.
2. `/Users/justinrich/Projects/gitbutler/.spec/prds/governance/12-uc-agent-identity.md`:31-33 — UC-IDENT-01 AC-5/6 — `but agent migrate` byte-equivalent round-trip + idempotent no-op; the operator commits add+delete together.
3. `/Users/justinrich/Projects/brain/skills/but-init/SKILL.md`:120-235 — Sibling skill's commit/idempotency style to keep but-migrate consistent (see IDENT-025).
4. `/Users/justinrich/Projects/brain/docs/BUT-SKILL-CONVENTIONS.md`:21-66 — §2 identity model + §6 artifacts/paths the migrate flow must honor.

## Guardrails

**WRITE-ALLOWED:**
- /Users/justinrich/Projects/brain/skills/but-migrate/SKILL.md (MODIFY — detect permissions.toml, run `but agent migrate`, commit rename, document idempotency)
- /Users/justinrich/.claude/skills/but-migrate/SKILL.md (MODIFY — mirror)

**WRITE-PROHIBITED:**
- /Users/justinrich/Projects/gitbutler/** - this skill task edits brain skills, not the gitbutler repo
- crates/** - the `but agent migrate` engine is upstream (Sprint 09)
- other brain skills (but-init/but-run-sprint) - separate tasks

## Code Pattern

**Reference:** /Users/justinrich/Projects/brain/skills/but-migrate/SKILL.md, 12-uc-agent-identity.md UC-IDENT-01 AC-5/6

**Pattern:** Add to the but-migrate flow a step that: (1) detects a committed `.gitbutler/permissions.toml` and no `agents.toml`; (2) runs `but agent migrate` (engine writes agents.toml, byte-equivalent round-trip); (3) commits the agents.toml add + permissions.toml delete together; (4) if `agents.toml` already exists, the migrate verb is a no-op and the skill makes no commit (idempotent). Reuse the skill's existing reconcile/idempotency framing.

**Source:** `Existing but-migrate idempotency/reconcile block + UC-IDENT-01 AC-5/6 migrate contract.`

**Design notes:**
- Edit canonical -> mirror -> diff -rq clean is mandatory (AC-5).

**Anti-pattern:** Do NOT hand-edit the toml with mv/sed; do NOT split the add+delete across two commits; do NOT break idempotency; do NOT edit only the canonical copy; do NOT claim the fixture e2e passes in the gitbutler test suite.

## Agent Instructions

TDD RED→GREEN per AC (build-gate/source greps + real `but`/skill execution against real files — NO mocks):
1. **RED:** write each AC's failing check first (against the current start state — docs/skill absent or still self-asserting `BUT_AGENT_HANDLE`).
2. **GREEN:** make the minimal edit (docs/skill/migration) to satisfy the AC.
3. For brain-skill tasks: edit the CANONICAL copy under `~/Projects/brain/skills/`, MIRROR to `~/.claude/skills/`, then `diff -rq` clean.
4. Run each AC's verify command; commit via `but commit` (governed).

## Orchestrator Verification Protocol

- `grep -q 'but agent migrate' ~/Projects/brain/skills/but-migrate/SKILL.md && grep -qiE 'idempotent|no-op' ~/Projects/brain/skills/but-migrate/SKILL.md` → exit 0
- `diff -rq ~/Projects/brain/skills/but-migrate ~/.claude/skills/but-migrate` → no output (exit 0)
- `run /but-migrate against a permissions.toml fixture; git show HEAD:.gitbutler/agents.toml; re-run /but-migrate` → agents.toml committed + permissions.toml removed in one commit; re-run no-op (run where the skill lives)

## Agent Assignment

**Agent:** `rust-planner` — Per the sprint stub the but-migrate skill-migration is owned by rust-planner as governed-pipeline process authoring; the planner edits the canonical brain skill + mirror to wire the `but agent migrate` rename flow.
**Pairing:** none (single-surface Rust/docs/skill task). Honors `crates/AGENTS.md` + `BUT-SKILL-CONVENTIONS.md`.

## Evidence Gates

- `grep -q 'but agent migrate' ~/Projects/brain/skills/but-migrate/SKILL.md && grep -qiE 'idempotent|no-op' ~/Projects/brain/skills/but-migrate/SKILL.md` (exit 0)
- `diff -rq ~/Projects/brain/skills/but-migrate ~/.claude/skills/but-migrate` (no output (exit 0))
- `run /but-migrate against a permissions.toml fixture; git show HEAD:.gitbutler/agents.toml; re-run /but-migrate` (agents.toml committed + permissions.toml removed in one commit; re-run no-op (run where the skill lives))

## Review Criteria

- AC-1: PRIMARY — /but-migrate converts permissions.toml -> agents.toml in one commit; re-run is no-op (e2e, BRAIN-REPO-OWNED) — verified by `cd permissions_only_fixture && /but-migrate && git show HEAD:.gitbutler/agents.toml | grep -q '\[\[agent\]\]' && ! git cat-file -e HEAD:.gitbutler/permissions.toml 2>/dev/null && before=$(git rev-parse HEAD) && /but-migrate && test "$before" = "$(git rev-parse HEAD)"`.
- AC-2: SKILL.md invokes `but agent migrate` — verified by `grep -q 'but agent migrate' ~/Projects/brain/skills/but-migrate/SKILL.md`.
- AC-3: SKILL.md detects committed permissions.toml and commits the rename — verified by `grep -q '.gitbutler/permissions.toml' ~/Projects/brain/skills/but-migrate/SKILL.md && grep -A8 'but agent migrate' ~/Projects/brain/skills/but-migrate/SKILL.md | grep -qiE 'commit'`.
- AC-4: SKILL.md documents idempotent no-op once agents.toml exists — verified by `grep -iE 'idempotent|no-op' ~/Projects/brain/skills/but-migrate/SKILL.md && grep -q 'agents.toml' ~/Projects/brain/skills/but-migrate/SKILL.md`.
- AC-5: Canonical and mirror copies are identical — verified by `diff -rq ~/Projects/brain/skills/but-migrate ~/.claude/skills/but-migrate`.
- Honors NEVER: Never report the e2e fixture migration (AC-1) as PASSED inside the gitbutler repo's test suite — it is OWNED BY THE BRAIN REPO's skill tests (T-IDENT-037); the gitbutler repo asserts the SOURCE-grep contract.

## Dependencies

- **Depends on:** none (BLOCKED-UNTIL Sprint-10 per Critical Constraints)
- **Blocks:** IDENT-028
- **Capabilities:** CAP-CONFIG-01, CAP-AUTHZ-01

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "IDENT-026",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "but_migrate_legacy_skill": {
      "description": "but-migrate canonical skill at ~/Projects/brain/skills/but-migrate/SKILL.md that provisions/reads permissions.toml and has no `but agent migrate` rename flow",
      "seed_method": "public_api",
      "records": [
        "SKILL.md references .gitbutler/permissions.toml (lines ~56,95,118,128)",
        "no `but agent migrate` invocation; no rename-commit step"
      ]
    },
    "permissions_only_fixture": {
      "description": "a fixture repo with a committed .gitbutler/permissions.toml and no agents.toml",
      "seed_method": "cli",
      "records": [
        "HEAD has .gitbutler/permissions.toml with [[principal]] blocks",
        "no .gitbutler/agents.toml at HEAD"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN a fixture repo with a committed permissions.toml and no agents.toml WHEN running the migrated /but-migrate skill end-to-end THEN agents.toml is written and permissions.toml is removed in the same commit, and a second run makes no change",
      "test_tier": "e2e",
      "verification_service": "but-migrate skill + permissions.toml fixture repo + but CLI (BRAIN-REPO-OWNED; T-IDENT-037)",
      "verify": "cd permissions_only_fixture && /but-migrate && git show HEAD:.gitbutler/agents.toml | grep -q '\\[\\[agent\\]\\]' && ! git cat-file -e HEAD:.gitbutler/permissions.toml 2>/dev/null && before=$(git rev-parse HEAD) && /but-migrate && test \"$before\" = \"$(git rev-parse HEAD)\"",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "e2e",
        "verification_service": "but-migrate skill + permissions.toml fixture repo + but CLI (BRAIN-REPO-OWNED)",
        "start_ref": "permissions_only_fixture",
        "must_observe": [
          "committed .gitbutler/agents.toml with '[[agent]]'",
          "permissions.toml absent at HEAD after migration",
          "second run leaves HEAD unchanged (same SHA)"
        ],
        "must_not_observe": [
          "permissions.toml still present at HEAD",
          "agents.toml absent",
          "second run creates a new commit (not idempotent)"
        ],
        "negative_control": {
          "would_fail_if": [
            "skill does not invoke `but agent migrate`",
            "rename not committed",
            "permissions.toml not deleted in the same commit",
            "re-run mutates HEAD (idempotency broken)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "permissions_only_fixture",
            "action": {
              "actor": "cli_user",
              "steps": [
                "run /but-migrate once",
                "inspect HEAD tree (agents.toml present, permissions.toml gone)",
                "run /but-migrate again",
                "compare HEAD SHA before/after"
              ]
            },
            "end_state": {
              "must_observe": [
                "`git show HEAD:.gitbutler/agents.toml` contains `[[agent]]`",
                "second run leaves HEAD `==` unchanged SHA"
              ],
              "must_not_observe": [
                "`permissions.toml` still at HEAD (start state)",
                "second run creates new commit (0 idempotency)"
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
      "description": "GIVEN the canonical but-migrate SKILL.md WHEN inspecting the migration flow THEN SKILL.md invokes the `but agent migrate` verb (not a hand-edited toml rename)",
      "test_tier": "integration",
      "verification_service": "source (brain/skills/but-migrate/SKILL.md)",
      "verify": "grep -q 'but agent migrate' ~/Projects/brain/skills/but-migrate/SKILL.md",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "source (brain/skills/but-migrate/SKILL.md)",
        "start_ref": "but_migrate_legacy_skill",
        "must_observe": [
          "SKILL.md contains literal 'but agent migrate'"
        ],
        "must_not_observe": [
          "hand-rolled toml rename instead of the verb",
          "no migrate invocation",
          "unchanged SKILL.md"
        ],
        "negative_control": {
          "would_fail_if": [
            "`but agent migrate` not added",
            "rename done by manual mv/sed in the skill",
            "grep stubbed or not executed"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "but_migrate_legacy_skill",
            "action": {
              "actor": "test_harness",
              "steps": [
                "grep SKILL.md for 'but agent migrate'"
              ]
            },
            "end_state": {
              "must_observe": [
                "SKILL.md contains `but agent migrate`",
                "migrate invoked in the flow (not a hand-rolled `mv`)"
              ],
              "must_not_observe": [
                "migrate verb absent (0 matches)",
                "hand-rolled rename (none of the verb)"
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
      "description": "GIVEN the canonical but-migrate SKILL.md WHEN inspecting the detection + commit step THEN SKILL.md detects `.gitbutler/permissions.toml` and commits the agents.toml write + permissions.toml delete together",
      "test_tier": "integration",
      "verification_service": "source (brain/skills/but-migrate/SKILL.md)",
      "verify": "grep -q '.gitbutler/permissions.toml' ~/Projects/brain/skills/but-migrate/SKILL.md && grep -A8 'but agent migrate' ~/Projects/brain/skills/but-migrate/SKILL.md | grep -qiE 'commit'",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "source (brain/skills/but-migrate/SKILL.md)",
        "start_ref": "but_migrate_legacy_skill",
        "must_observe": [
          "SKILL.md detects '.gitbutler/permissions.toml'",
          "documents committing the rename (add agents.toml + delete permissions.toml)"
        ],
        "must_not_observe": [
          "no detection of legacy file",
          "rename not committed",
          "unchanged SKILL.md"
        ],
        "negative_control": {
          "would_fail_if": [
            "detection step omitted",
            "commit-the-rename step omitted",
            "grep stubbed or not executed"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "but_migrate_legacy_skill",
            "action": {
              "actor": "test_harness",
              "steps": [
                "grep SKILL.md for permissions.toml detection + a commit step near `but agent migrate`"
              ]
            },
            "end_state": {
              "must_observe": [
                "SKILL.md detects `.gitbutler/permissions.toml`",
                "commit step near `but agent migrate` present"
              ],
              "must_not_observe": [
                "detection absent (0 matches)",
                "no commit step (none)"
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
      "description": "GIVEN the canonical but-migrate SKILL.md WHEN inspecting the idempotency contract THEN SKILL.md states a re-run is a clean no-op once `agents.toml` exists",
      "test_tier": "integration",
      "verification_service": "source (brain/skills/but-migrate/SKILL.md)",
      "verify": "grep -iE 'idempotent|no-op' ~/Projects/brain/skills/but-migrate/SKILL.md && grep -q 'agents.toml' ~/Projects/brain/skills/but-migrate/SKILL.md",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "source (brain/skills/but-migrate/SKILL.md)",
        "start_ref": "but_migrate_legacy_skill",
        "must_observe": [
          "SKILL.md states the migrate re-run is idempotent / no-op once agents.toml exists",
          "literal 'agents.toml' present"
        ],
        "must_not_observe": [
          "no idempotency statement for the migrate flow",
          "unchanged SKILL.md"
        ],
        "negative_control": {
          "would_fail_if": [
            "idempotency contract not documented for the rename",
            "grep stubbed or not executed"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "but_migrate_legacy_skill",
            "action": {
              "actor": "test_harness",
              "steps": [
                "grep SKILL.md for 'idempotent'/'no-op' + 'agents.toml'"
              ]
            },
            "end_state": {
              "must_observe": [
                "SKILL.md states `idempotent` no-op",
                "names `agents.toml` re-run guard"
              ],
              "must_not_observe": [
                "idempotency undocumented (0 matches)",
                "no guard (none)"
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
      "description": "GIVEN the canonical but-migrate skill and its ~/.claude mirror WHEN diffing the two trees recursively THEN `diff -rq` reports no differences",
      "test_tier": "integration",
      "verification_service": "filesystem (brain canonical vs ~/.claude mirror)",
      "verify": "diff -rq ~/Projects/brain/skills/but-migrate ~/.claude/skills/but-migrate",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "filesystem (brain canonical vs ~/.claude mirror)",
        "start_ref": "but_migrate_legacy_skill",
        "must_observe": [
          "diff -rq prints nothing (exit 0)",
          "mirror reflects the `but agent migrate` edits"
        ],
        "must_not_observe": [
          "'Files ... differ' output",
          "'Only in' output",
          "mirror still legacy"
        ],
        "negative_control": {
          "would_fail_if": [
            "only the canonical copy edited (mirror stale)",
            "diff stubbed or not executed"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "but_migrate_legacy_skill",
            "action": {
              "actor": "test_harness",
              "steps": [
                "run diff -rq canonical mirror",
                "assert empty output"
              ]
            },
            "end_state": {
              "must_observe": [
                "`diff -rq` prints `0` lines for `but-migrate`",
                "mirror contains `but agent migrate` matching canonical"
              ],
              "must_not_observe": [
                "`Files ... differ` lines (>0)",
                "mirror still legacy (start state, none migrated)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "/but-migrate writes agents.toml + removes permissions.toml in one commit and re-run is a no-op is true (BRAIN-REPO-OWNED)",
      "maps_to_ac": "AC-1",
      "verify": "git show HEAD:.gitbutler/agents.toml | grep -q '\\[\\[agent\\]\\]' && ! git cat-file -e HEAD:.gitbutler/permissions.toml 2>/dev/null"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "SKILL.md invokes `but agent migrate` is true",
      "maps_to_ac": "AC-2",
      "verify": "grep -q 'but agent migrate' ~/Projects/brain/skills/but-migrate/SKILL.md"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "SKILL.md detects .gitbutler/permissions.toml and commits the rename is true",
      "maps_to_ac": "AC-3",
      "verify": "grep -q '.gitbutler/permissions.toml' ~/Projects/brain/skills/but-migrate/SKILL.md && grep -A8 'but agent migrate' ~/Projects/brain/skills/but-migrate/SKILL.md | grep -qiE 'commit'"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "SKILL.md documents idempotent no-op once agents.toml exists is true",
      "maps_to_ac": "AC-4",
      "verify": "grep -qiE 'idempotent|no-op' ~/Projects/brain/skills/but-migrate/SKILL.md"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "Canonical and ~/.claude mirror copies are byte-identical is true",
      "maps_to_ac": "AC-5",
      "verify": "diff -rq ~/Projects/brain/skills/but-migrate ~/.claude/skills/but-migrate"
    }
  ]
}
-->
