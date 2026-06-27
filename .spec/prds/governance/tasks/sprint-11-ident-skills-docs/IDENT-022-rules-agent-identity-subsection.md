# IDENT-022 — `RULES.md` — add "Agent identity" subsection under Conventions (governed repos require `but agent register`; env var is test-only)

**Sprint:** [Sprint 11](./SPRINT.md) · **Agent:** `rust-implementer` · **Estimate:** 30 min · **Type:** FEATURE · **Status:** Backlog · **Proposed By:** rust-planner

## Background

rust-implementer owns repo-level convention docs under `crates/`-adjacent root files and is the executor that lives the governed `but` path daily; documenting the registry-first identity contract in RULES.md is a single-file, build-gate-verifiable edit in its lane.

**Provides:** (documentation/consumer layer — no new capability)

**Consumes:** resolve_principal_with_registry resolution order (registry -> flag-gated env -> denial), Denial::unregistered (perm.denied), BUT_AUTHZ_ALLOW_ENV_HANDLE env-handle gate

**Boundary contracts:**
- The documented identity rule in RULES.md matches the engine's resolve_principal_with_registry behavior: registry hit -> principal; registry miss + BUT_AUTHZ_ALLOW_ENV_HANDLE=1 -> env fallback; else Denial::unregistered.

## Critical Constraints

**MUST:**
- Add a single 'Agent identity' subsection under `## Conventions` in RULES.md that states governed repos require `but agent register` before any gate.
- State that `BUT_AGENT_HANDLE` is test-only and gated by `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` — never the production identity path.
- Keep CLAUDE.md/AGENTS.md untouched (they are pointers to RULES.md); the subsection lives only in RULES.md.

**NEVER:**
- Never present `BUT_AGENT_HANDLE` as the default/normal identity mechanism.
- Never edit any file other than RULES.md (doc-only, single file).
- Never use vague wording like 'agents have identity' — name the verb (`but agent register`) and the gate var explicitly.

**STRICTLY:**
- BLOCKED-UNTIL Sprint-10: the final env-handle deny-default must be locked (transitively Sprint-09: `but agent` verbs + agents.toml loader + 8-callsite swap) before RULES.md can honestly say the registry path is required and the env var is test-only.
- This is documentation only — no code, no behavior change.

## Specification

**Objective:** Add an 'Agent identity' subsection to RULES.md under Conventions documenting that governed repos require `but agent register` before any gate and that `BUT_AGENT_HANDLE` is a test-only escape hatch gated by `BUT_AUTHZ_ALLOW_ENV_HANDLE=1`.

**Success state:** RULES.md contains a `### Agent identity` subsection inside `## Conventions` that literally names `but agent register`, states it is required before any gate on governed repos, and documents `BUT_AGENT_HANDLE` as test-only gated by `BUT_AUTHZ_ALLOW_ENV_HANDLE=1`; a build-gate grep finds all three literals.

## Acceptance Criteria

**AC-1 (PRIMARY)** — Agent identity subsection names `but agent register` (build-gate per T-IDENT-032..035)
- **GIVEN:** RULES.md with a `## Conventions` section and no agent-identity subsection
- **WHEN:** Adding a `### Agent identity` subsection under Conventions
- **THEN:** RULES.md literally contains `but agent register` inside an `### Agent identity` subsection
- **Verify:** `grep -q '### Agent identity' RULES.md && grep -q 'but agent register' RULES.md`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** source · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=rules_md_conventions`; must_observe = [`### Agent identity` heading present in RULES.md; RULES.md contains `but agent register`]; must_not_observe = [`### Agent identity` absent (`grep` 0 matches); `but agent register` missing (empty)]; negative_control.would_fail_if = [subsection not added (absent); keyword 'but agent register' removed; grep stubbed or not executed; only CLAUDE.md/AGENTS.md edited (RULES.md unchanged)].

**AC-2** — Subsection frames BUT_AGENT_HANDLE as test-only behind BUT_AUTHZ_ALLOW_ENV_HANDLE=1 (build-gate per T-IDENT-032..035)
- **GIVEN:** The `### Agent identity` subsection
- **WHEN:** Reading the env-var sentence
- **THEN:** RULES.md names `BUT_AGENT_HANDLE` + `BUT_AUTHZ_ALLOW_ENV_HANDLE` and frames the env var as test-only / escape hatch
- **Verify:** `grep -q 'BUT_AGENT_HANDLE' RULES.md && grep -q 'BUT_AUTHZ_ALLOW_ENV_HANDLE' RULES.md && grep -iE 'test.only|test only|escape.hatch|not.*production' RULES.md`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** source · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=rules_md_conventions`; must_observe = [RULES.md contains `BUT_AGENT_HANDLE` + `BUT_AUTHZ_ALLOW_ENV_HANDLE`; frames env var as `test-only` / `escape hatch`]; must_not_observe = [`BUT_AUTHZ_ALLOW_ENV_HANDLE` missing (0 matches); env var shown as default (no test-only framing)]; negative_control.would_fail_if = [env-var sentence not added; 'BUT_AUTHZ_ALLOW_ENV_HANDLE' literal omitted; grep stubbed or not executed; env var documented as the normal path].

**AC-3** — Subsection states registration precedes any gate on governed repos (build-gate per T-IDENT-032..035)
- **GIVEN:** The `### Agent identity` subsection
- **WHEN:** Reading the requirement sentence
- **THEN:** Subsection states governed repos require `but agent register` before any gate (unregistered processes are denied)
- **Verify:** `grep -iE 'before any gate|require[sd]? .*register|unregistered' RULES.md`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** source · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=rules_md_conventions`; must_observe = [RULES.md states registration required `before any gate`; names `but agent register` + `unregistered`]; must_not_observe = [requirement wording absent (0 matches); registration framed as optional (none required)]; negative_control.would_fail_if = [requirement sentence not added; wording weakened to optional; grep stubbed or not executed].

**AC-4** — Subsection is placed under `## Conventions`, not elsewhere (build-gate per T-IDENT-032..035)
- **GIVEN:** RULES.md structure
- **WHEN:** Checking the heading order
- **THEN:** `### Agent identity` appears after `## Conventions` and before `## Scoped Instructions & Key Docs`
- **Verify:** `awk '/^## Conventions/{c=1} /^### Agent identity/{if(c)found=1} /^## Scoped Instructions/{if(found)print "OK"; c=0}' RULES.md | grep OK`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** source · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=rules_md_conventions`; must_observe = [`### Agent identity` is nested within the `## Conventions` block; appears before `## Scoped Instructions & Key Docs`]; must_not_observe = [subsection placed outside Conventions; subsection at top of file; empty doc]; negative_control.would_fail_if = [subsection added under a different section; heading omitted; awk/grep stubbed or not executed].

## Test Criteria

| ID | Boolean | Maps to AC |
|----|---------|------------|
| TC-1 | RULES.md has an `### Agent identity` subsection naming `but agent register` is true | AC-1 |
| TC-2 | Subsection names BUT_AUTHZ_ALLOW_ENV_HANDLE and frames the env var as test-only is true | AC-2 |
| TC-3 | Subsection states registration is required before any gate is true | AC-3 |
| TC-4 | Subsection is nested under `## Conventions` is true | AC-4 |

## Reading List

1. `RULES.md`:155-227 — PRIMARY PATTERN — `## Conventions` section shape (### Rust, ### Version control, ### Scoped Instructions). Add `### Agent identity` here, matching the bullet-list style of sibling subsections.
2. `.spec/prds/governance/12-uc-agent-identity.md`:82-92 — UC-IDENT-05 wording the subsection must reflect — register-before-gate, env var test-only.
3. `crates/but-authz/src/authorize.rs`:185-248 — Authoritative resolution order + BUT_AUTHZ_ALLOW_ENV_HANDLE gate to quote accurately in the subsection.
4. `.spec/prds/governance/tasks/sprint-10-ident-deprecation-hardening/IDENT-021-gate-callsite-doc-audit.md`:51-114 — Exemplar grep-based source-presence AC pattern to mirror.

## Guardrails

**WRITE-ALLOWED:**
- RULES.md (MODIFY — add `### Agent identity` subsection under `## Conventions`)

**WRITE-PROHIBITED:**
- CLAUDE.md - it is a pointer to RULES.md; do not duplicate the subsection there
- AGENTS.md - pointer to RULES.md; out of scope
- crates/** - no code changes (doc-only task)
- Any file not RULES.md - reason: single-file documentation task

## Code Pattern

**Reference:** RULES.md `## Conventions` subsections, crates/but-authz/src/authorize.rs resolution order

**Pattern:** Markdown convention subsection: a `### Agent identity` heading with a short bullet list mirroring the prose density of `### Version control`. Bullets: (1) governed repos require `but agent register` before any gate; unregistered processes are denied (`perm.denied`). (2) `BUT_AGENT_HANDLE` is test-only, accepted only when `BUT_AUTHZ_ALLOW_ENV_HANDLE=1`. (3) Resolution order: registry -> flag-gated env -> denial.

**Source:** `RULES.md existing `### Version control` subsection (same bullet style)`

**Anti-pattern:** Do NOT present BUT_AGENT_HANDLE as the normal identity path; do NOT add the subsection to CLAUDE.md/AGENTS.md; do NOT use vague 'agents have identity' wording without naming the verb and gate var.

## Agent Instructions

TDD RED→GREEN per AC (build-gate/source greps + real `but`/skill execution against real files — NO mocks):
1. **RED:** write each AC's failing check first (against the current start state — docs/skill absent or still self-asserting `BUT_AGENT_HANDLE`).
2. **GREEN:** make the minimal edit (docs/skill/migration) to satisfy the AC.
3. For brain-skill tasks: edit the CANONICAL copy under `~/Projects/brain/skills/`, MIRROR to `~/.claude/skills/`, then `diff -rq` clean.
4. Run each AC's verify command; commit via `but commit` (governed).

## Orchestrator Verification Protocol

- `grep -A12 '### Agent identity' RULES.md | grep 'but agent register'` → match (exit 0)
- `grep -A12 '### Agent identity' RULES.md | grep 'BUT_AUTHZ_ALLOW_ENV_HANDLE'` → match (exit 0)
- `git diff --name-only` → only RULES.md

## Agent Assignment

**Agent:** `rust-implementer` — rust-implementer owns repo-level convention docs under `crates/`-adjacent root files and is the executor that lives the governed `but` path daily; documenting the registry-first identity contract in RULES.md is a single-file, build-gate-verifiable edit in its lane.
**Pairing:** none (single-surface Rust/docs/skill task). Honors `crates/AGENTS.md` + `BUT-SKILL-CONVENTIONS.md`.

## Evidence Gates

- `grep -A12 '### Agent identity' RULES.md | grep 'but agent register'` (match (exit 0))
- `grep -A12 '### Agent identity' RULES.md | grep 'BUT_AUTHZ_ALLOW_ENV_HANDLE'` (match (exit 0))
- `git diff --name-only` (only RULES.md)

## Review Criteria

- AC-1: PRIMARY — Agent identity subsection names `but agent register` (build-gate per T-IDENT-032..035) — verified by `grep -q '### Agent identity' RULES.md && grep -q 'but agent register' RULES.md`.
- AC-2: Subsection frames BUT_AGENT_HANDLE as test-only behind BUT_AUTHZ_ALLOW_ENV_HANDLE=1 (build-gate per T-IDENT-032..035) — verified by `grep -q 'BUT_AGENT_HANDLE' RULES.md && grep -q 'BUT_AUTHZ_ALLOW_ENV_HANDLE' RULES.md && grep -iE 'test.only|test only|escape.hatch|not.*production' RULES.md`.
- AC-3: Subsection states registration precedes any gate on governed repos (build-gate per T-IDENT-032..035) — verified by `grep -iE 'before any gate|require[sd]? .*register|unregistered' RULES.md`.
- AC-4: Subsection is placed under `## Conventions`, not elsewhere (build-gate per T-IDENT-032..035) — verified by `awk '/^## Conventions/{c=1} /^### Agent identity/{if(c)found=1} /^## Scoped Instructions/{if(found)print "OK"; c=0}' RULES.md | grep OK`.
- Honors NEVER: Never present `BUT_AGENT_HANDLE` as the default/normal identity mechanism.

## Dependencies

- **Depends on:** none (BLOCKED-UNTIL Sprint-10 per Critical Constraints)
- **Blocks:** none
- **Capabilities:** CAP-AUTHZ-01

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "IDENT-022",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "rules_md_conventions": {
      "description": "RULES.md at repo root with a `## Conventions` section (### Rust, ### Rust tests, ### TypeScript/Svelte/React, ### Version control) and no agent-identity subsection yet",
      "seed_method": "public_api",
      "records": [
        "`## Conventions` exists at RULES.md line ~155",
        "`### Version control` exists at line ~208",
        "No `Agent identity` / `but agent register` content present at start"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN RULES.md with a `## Conventions` section and no agent-identity subsection WHEN Adding a `### Agent identity` subsection under Conventions THEN RULES.md literally contains `but agent register` inside an `### Agent identity` subsection",
      "test_tier": "integration",
      "verification_service": "source",
      "verify": "grep -q '### Agent identity' RULES.md && grep -q 'but agent register' RULES.md",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "source",
        "start_ref": "rules_md_conventions",
        "must_observe": [
          "`### Agent identity` heading present in RULES.md",
          "RULES.md contains `but agent register`"
        ],
        "must_not_observe": [
          "`### Agent identity` absent (`grep` 0 matches)",
          "`but agent register` missing (empty)"
        ],
        "negative_control": {
          "would_fail_if": [
            "subsection not added (absent)",
            "keyword 'but agent register' removed",
            "grep stubbed or not executed",
            "only CLAUDE.md/AGENTS.md edited (RULES.md unchanged)"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "rules_md_conventions",
            "action": {
              "actor": "test_harness",
              "steps": [
                "grep for '### Agent identity' heading in RULES.md",
                "grep the subsection body for 'but agent register'"
              ]
            },
            "end_state": {
              "must_observe": [
                "`### Agent identity` heading present in RULES.md",
                "RULES.md contains `but agent register`"
              ],
              "must_not_observe": [
                "`### Agent identity` absent (`grep` 0 matches)",
                "`but agent register` missing (empty)"
              ]
            }
          }
        ]
      },
      "criteria_layer": "build-gate"
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN The `### Agent identity` subsection WHEN Reading the env-var sentence THEN RULES.md names `BUT_AGENT_HANDLE` + `BUT_AUTHZ_ALLOW_ENV_HANDLE` and frames the env var as test-only / escape hatch",
      "test_tier": "integration",
      "verification_service": "source",
      "verify": "grep -q 'BUT_AGENT_HANDLE' RULES.md && grep -q 'BUT_AUTHZ_ALLOW_ENV_HANDLE' RULES.md && grep -iE 'test.only|test only|escape.hatch|not.*production' RULES.md",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "source",
        "start_ref": "rules_md_conventions",
        "must_observe": [
          "RULES.md contains `BUT_AGENT_HANDLE` + `BUT_AUTHZ_ALLOW_ENV_HANDLE`",
          "frames env var as `test-only` / `escape hatch`"
        ],
        "must_not_observe": [
          "`BUT_AUTHZ_ALLOW_ENV_HANDLE` missing (0 matches)",
          "env var shown as default (no test-only framing)"
        ],
        "negative_control": {
          "would_fail_if": [
            "env-var sentence not added",
            "'BUT_AUTHZ_ALLOW_ENV_HANDLE' literal omitted",
            "grep stubbed or not executed",
            "env var documented as the normal path"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "rules_md_conventions",
            "action": {
              "actor": "test_harness",
              "steps": [
                "grep subsection for 'BUT_AGENT_HANDLE'",
                "grep subsection for 'BUT_AUTHZ_ALLOW_ENV_HANDLE'"
              ]
            },
            "end_state": {
              "must_observe": [
                "RULES.md contains `BUT_AGENT_HANDLE` + `BUT_AUTHZ_ALLOW_ENV_HANDLE`",
                "frames env var as `test-only` / `escape hatch`"
              ],
              "must_not_observe": [
                "`BUT_AUTHZ_ALLOW_ENV_HANDLE` missing (0 matches)",
                "env var shown as default (no test-only framing)"
              ]
            }
          }
        ]
      },
      "criteria_layer": "build-gate"
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN The `### Agent identity` subsection WHEN Reading the requirement sentence THEN Subsection states governed repos require `but agent register` before any gate (unregistered processes are denied)",
      "test_tier": "integration",
      "verification_service": "source",
      "verify": "grep -iE 'before any gate|require[sd]? .*register|unregistered' RULES.md",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "source",
        "start_ref": "rules_md_conventions",
        "must_observe": [
          "RULES.md states registration required `before any gate`",
          "names `but agent register` + `unregistered`"
        ],
        "must_not_observe": [
          "requirement wording absent (0 matches)",
          "registration framed as optional (none required)"
        ],
        "negative_control": {
          "would_fail_if": [
            "requirement sentence not added",
            "wording weakened to optional",
            "grep stubbed or not executed"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "rules_md_conventions",
            "action": {
              "actor": "test_harness",
              "steps": [
                "grep subsection for 'before any gate' / 'require...register' / 'unregistered'"
              ]
            },
            "end_state": {
              "must_observe": [
                "RULES.md states registration required `before any gate`",
                "names `but agent register` + `unregistered`"
              ],
              "must_not_observe": [
                "requirement wording absent (0 matches)",
                "registration framed as optional (none required)"
              ]
            }
          }
        ]
      },
      "criteria_layer": "build-gate"
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN RULES.md structure WHEN Checking the heading order THEN `### Agent identity` appears after `## Conventions` and before `## Scoped Instructions & Key Docs`",
      "test_tier": "integration",
      "verification_service": "source",
      "verify": "awk '/^## Conventions/{c=1} /^### Agent identity/{if(c)found=1} /^## Scoped Instructions/{if(found)print \"OK\"; c=0}' RULES.md | grep OK",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "source",
        "start_ref": "rules_md_conventions",
        "must_observe": [
          "`### Agent identity` is nested within the `## Conventions` block",
          "appears before `## Scoped Instructions & Key Docs`"
        ],
        "must_not_observe": [
          "subsection placed outside Conventions",
          "subsection at top of file",
          "empty doc"
        ],
        "negative_control": {
          "would_fail_if": [
            "subsection added under a different section",
            "heading omitted",
            "awk/grep stubbed or not executed"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "rules_md_conventions",
            "action": {
              "actor": "test_harness",
              "steps": [
                "scan headings between `## Conventions` and `## Scoped Instructions`",
                "assert `### Agent identity` falls inside that range"
              ]
            },
            "end_state": {
              "must_observe": [
                "`### Agent identity` nested under `## Conventions`",
                "`awk` placement check prints `OK`"
              ],
              "must_not_observe": [
                "placement check prints nothing (0 lines)",
                "heading outside Conventions (none in section)"
              ]
            }
          }
        ]
      },
      "criteria_layer": "build-gate"
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "RULES.md has an `### Agent identity` subsection naming `but agent register` is true",
      "maps_to_ac": "AC-1",
      "verify": "grep -q '### Agent identity' RULES.md && grep -q 'but agent register' RULES.md"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "Subsection names BUT_AUTHZ_ALLOW_ENV_HANDLE and frames the env var as test-only is true",
      "maps_to_ac": "AC-2",
      "verify": "grep -q 'BUT_AUTHZ_ALLOW_ENV_HANDLE' RULES.md && grep -iE 'test.only|test only|escape.hatch|not.*production' RULES.md"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "Subsection states registration is required before any gate is true",
      "maps_to_ac": "AC-3",
      "verify": "grep -iE 'before any gate|require[sd]? .*register|unregistered' RULES.md"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "Subsection is nested under `## Conventions` is true",
      "maps_to_ac": "AC-4",
      "verify": "awk '/^## Conventions/{c=1} /^### Agent identity/{if(c)found=1} /^## Scoped Instructions/{if(found)print \"OK\"; c=0}' RULES.md | grep -q OK"
    }
  ]
}
-->
