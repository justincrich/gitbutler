# IDENT-024 — `crates/AGENTS.md` + `crates/but/AGENTS.md` + `DEVELOPMENT.md` "Code Hitlist" + `crates/WORKSPACE_MODEL.md` — cross-reference the identity README, document `but agent` noun, track the rename

**Sprint:** [Sprint 11](./SPRINT.md) · **Agent:** `rust-implementer` · **Estimate:** 60 min · **Type:** FEATURE · **Status:** Backlog · **Proposed By:** rust-planner

## Background

rust-implementer owns the `crates/` agent docs and DEVELOPMENT.md hitlist; wiring four existing docs to cross-reference the new identity README and recording the permissions.toml->agents.toml rename is a multi-file, grep-verifiable edit in its lane.

**Provides:** (documentation/consumer layer — no new capability)

**Consumes:** crates/but-authz/README.md identity model (IDENT-023), but agent CLI noun (register/migrate/list --committed/whoami), permissions.toml -> agents.toml rename (CAP-CONFIG-01)

**Boundary contracts:**
- The cross-references and Hitlist entry point developers at the authoritative identity README and record the permissions.toml->agents.toml rename as a tracked migration, matching the engine's agents.toml model.

## Critical Constraints

**MUST:**
- Add a cross-reference in `crates/AGENTS.md` pointing to `crates/but-authz/README.md` for the identity model.
- Add the `permissions.toml` -> `agents.toml` rename to the DEVELOPMENT.md `## Code Hitlist` as a tracked migration.
- Document the `but agent` noun in `crates/but/AGENTS.md` (cross-ref to the identity README) and cross-reference the identity README from `crates/WORKSPACE_MODEL.md`.
- Use a working relative link path to `crates/but-authz/README.md` from each doc.

**NEVER:**
- Never create the README here — it is IDENT-023's deliverable (this task only links to it).
- Never edit code or `.toml` config files — docs cross-referencing only.
- Never restructure the four target docs beyond the additive cross-reference/Hitlist entry.

**STRICTLY:**
- BLOCKED-UNTIL IDENT-023 (the README must exist before it can be cross-referenced).
- BLOCKED-UNTIL Sprint-10 (final env-handle deny-default locked) and transitively Sprint-09 (`but agent` verbs landed) so the documented `but agent` noun is real.
- Documentation only — additive cross-references + one Hitlist entry.

## Specification

**Objective:** Cross-reference crates/but-authz/README.md from crates/AGENTS.md and crates/WORKSPACE_MODEL.md, document the `but agent` noun in crates/but/AGENTS.md, and record the permissions.toml->agents.toml rename in the DEVELOPMENT.md Code Hitlist.

**Success state:** All four docs are updated: crates/AGENTS.md + crates/WORKSPACE_MODEL.md link to but-authz/README.md, crates/but/AGENTS.md documents the `but agent` noun, and DEVELOPMENT.md Code Hitlist lists the permissions.toml->agents.toml rename; grep confirms each.

## Acceptance Criteria

**AC-1 (PRIMARY)** — crates/AGENTS.md cross-references but-authz/README.md (build-gate per T-IDENT-032..035)
- **GIVEN:** crates/AGENTS.md with no identity-model cross-reference
- **WHEN:** Adding a cross-reference to the identity README
- **THEN:** crates/AGENTS.md contains a link/path to `but-authz/README.md` for the identity model
- **Verify:** `grep 'but-authz/README.md' crates/AGENTS.md`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** source · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=identity_docs_unlinked`; must_observe = [crates/AGENTS.md contains literal 'but-authz/README.md'; reference framed as the identity model]; must_not_observe = [cross-reference absent; empty/unchanged doc; broken link to a non-existent file]; negative_control.would_fail_if = [cross-reference not added; link points to a non-existent path; grep stubbed or not executed; doc unchanged from start].

**AC-2** — DEVELOPMENT.md Code Hitlist records the permissions.toml->agents.toml rename (build-gate per T-IDENT-032..035)
- **GIVEN:** DEVELOPMENT.md `## Code Hitlist`
- **WHEN:** Adding the rename as a tracked migration
- **THEN:** Hitlist contains an entry naming the `permissions.toml` -> `agents.toml` rename
- **Verify:** `awk '/^## Code Hitlist/{h=1} h&&/agents.toml/&&/permissions.toml/{print} /^## /{if($0!~/Code Hitlist/)h=0}' DEVELOPMENT.md | grep agents.toml`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** source · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=identity_docs_unlinked`; must_observe = [Code Hitlist entry naming 'permissions.toml' and 'agents.toml'; framed as a tracked rename/migration]; must_not_observe = [no Hitlist entry for the rename; entry placed outside the Code Hitlist section; unchanged doc]; negative_control.would_fail_if = [Hitlist entry not added; rename named only outside Code Hitlist; grep/awk stubbed or not executed; only one of the two filenames present].

**AC-3** — crates/but/AGENTS.md documents the `but agent` noun (build-gate per T-IDENT-032..035)
- **GIVEN:** crates/but/AGENTS.md
- **WHEN:** Adding documentation of the `but agent` CLI noun
- **THEN:** crates/but/AGENTS.md names `but agent` (register/migrate/whoami) and cross-references the identity README
- **Verify:** `grep 'but agent' crates/but/AGENTS.md && grep 'but-authz/README.md' crates/but/AGENTS.md`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** source · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=identity_docs_unlinked`; must_observe = [literal 'but agent'; literal 'but-authz/README.md' link; noun framed as identity surface]; must_not_observe = [no `but agent` mention; no cross-reference; unchanged doc]; negative_control.would_fail_if = [`but agent` not documented; cross-reference omitted; grep stubbed or not executed].

**AC-4** — crates/WORKSPACE_MODEL.md cross-references the identity README (build-gate per T-IDENT-032..035)
- **GIVEN:** crates/WORKSPACE_MODEL.md
- **WHEN:** Adding a cross-reference to the identity README
- **THEN:** crates/WORKSPACE_MODEL.md links to `but-authz/README.md` for the agent-identity model
- **Verify:** `grep 'but-authz/README.md' crates/WORKSPACE_MODEL.md`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** source · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=identity_docs_unlinked`; must_observe = [literal 'but-authz/README.md' in WORKSPACE_MODEL.md; reference framed as identity model]; must_not_observe = [cross-reference absent; broken link; unchanged doc]; negative_control.would_fail_if = [cross-reference not added; link to non-existent path; grep stubbed or not executed].

## Test Criteria

| ID | Boolean | Maps to AC |
|----|---------|------------|
| TC-1 | crates/AGENTS.md links to but-authz/README.md is true | AC-1 |
| TC-2 | DEVELOPMENT.md Code Hitlist lists the permissions.toml->agents.toml rename is true | AC-2 |
| TC-3 | crates/but/AGENTS.md documents the `but agent` noun and links the README is true | AC-3 |
| TC-4 | crates/WORKSPACE_MODEL.md links to but-authz/README.md is true | AC-4 |
| TC-5 | Every cross-referenced README link resolves to an existing file is true | AC-1 |

## Reading List

1. `crates/AGENTS.md`:1-130 — PRIMARY PATTERN — section layout (## API Boundaries and Legacy, ## Testing and Validation); add the identity-model cross-reference in the style of existing doc pointers.
2. `DEVELOPMENT.md`:534-543 — `## Code Hitlist` entry format (bulleted crate + parenthetical reason); add the permissions.toml->agents.toml rename entry matching it.
3. `crates/but/AGENTS.md`:1-60 — Where to document the `but agent` noun + link the identity README.
4. `crates/WORKSPACE_MODEL.md`:1-40 — Doc-pointer / reference style for adding the identity README cross-reference.
5. `crates/but-authz/README.md`:1-40 — The cross-reference target (created by IDENT-023) — confirm the relative path from each doc.

## Guardrails

**WRITE-ALLOWED:**
- crates/AGENTS.md (MODIFY — add identity README cross-reference)
- crates/but/AGENTS.md (MODIFY — document `but agent` noun + cross-reference)
- DEVELOPMENT.md (MODIFY — add permissions.toml->agents.toml rename to Code Hitlist)
- crates/WORKSPACE_MODEL.md (MODIFY — add identity README cross-reference)

**WRITE-PROHIBITED:**
- crates/but-authz/README.md - created by IDENT-023; this task only links to it
- crates/**/*.rs - no code changes
- *.toml - no config changes
- Any file not in the four-doc list above

## Code Pattern

**Reference:** crates/AGENTS.md `## Scoped Instructions`-style pointers, DEVELOPMENT.md `## Code Hitlist` bullet format, crates/but-authz/README.md (IDENT-023)

**Pattern:** Additive cross-references: in crates/AGENTS.md and crates/WORKSPACE_MODEL.md add a one-line pointer to `crates/but-authz/README.md` for the agent-identity model; in crates/but/AGENTS.md add a `but agent` noun note + the same pointer; in DEVELOPMENT.md `## Code Hitlist` add a bullet recording the `permissions.toml` -> `agents.toml` rename as a tracked migration (mirroring the existing crate bullets).

**Source:** `DEVELOPMENT.md existing Code Hitlist bullets + crates/AGENTS.md existing doc pointers`

**Anti-pattern:** Do NOT author the README here; do NOT restructure target docs; do NOT add the Hitlist entry outside the `## Code Hitlist` section; do NOT use a relative link path that does not resolve.

## Agent Instructions

TDD RED→GREEN per AC (build-gate/source greps + real `but`/skill execution against real files — NO mocks):
1. **RED:** write each AC's failing check first (against the current start state — docs/skill absent or still self-asserting `BUT_AGENT_HANDLE`).
2. **GREEN:** make the minimal edit (docs/skill/migration) to satisfy the AC.
3. For brain-skill tasks: edit the CANONICAL copy under `~/Projects/brain/skills/`, MIRROR to `~/.claude/skills/`, then `diff -rq` clean.
4. Run each AC's verify command; commit via `but commit` (governed).

## Orchestrator Verification Protocol

- `grep -q 'but-authz/README.md' crates/AGENTS.md && grep -q 'but-authz/README.md' crates/WORKSPACE_MODEL.md && grep -q 'but agent' crates/but/AGENTS.md` → exit 0
- `awk '/^## Code Hitlist/{h=1} h&&/agents.toml/&&/permissions.toml/{f=1} /^## /{if($0!~/Code Hitlist/)h=0} END{exit !f}' DEVELOPMENT.md` → exit 0
- `test -f crates/but-authz/README.md` → exit 0
- `git diff --name-only` → only the four target docs

## Agent Assignment

**Agent:** `rust-implementer` — rust-implementer owns the `crates/` agent docs and DEVELOPMENT.md hitlist; wiring four existing docs to cross-reference the new identity README and recording the permissions.toml->agents.toml rename is a multi-file, grep-verifiable edit in its lane.
**Pairing:** none (single-surface Rust/docs/skill task). Honors `crates/AGENTS.md` + `BUT-SKILL-CONVENTIONS.md`.

## Evidence Gates

- `grep -q 'but-authz/README.md' crates/AGENTS.md && grep -q 'but-authz/README.md' crates/WORKSPACE_MODEL.md && grep -q 'but agent' crates/but/AGENTS.md` (exit 0)
- `awk '/^## Code Hitlist/{h=1} h&&/agents.toml/&&/permissions.toml/{f=1} /^## /{if($0!~/Code Hitlist/)h=0} END{exit !f}' DEVELOPMENT.md` (exit 0)
- `test -f crates/but-authz/README.md` (exit 0)
- `git diff --name-only` (only the four target docs)

## Review Criteria

- AC-1: PRIMARY — crates/AGENTS.md cross-references but-authz/README.md (build-gate per T-IDENT-032..035) — verified by `grep 'but-authz/README.md' crates/AGENTS.md`.
- AC-2: DEVELOPMENT.md Code Hitlist records the permissions.toml->agents.toml rename (build-gate per T-IDENT-032..035) — verified by `awk '/^## Code Hitlist/{h=1} h&&/agents.toml/&&/permissions.toml/{print} /^## /{if($0!~/Code Hitlist/)h=0}' DEVELOPMENT.md | grep agents.toml`.
- AC-3: crates/but/AGENTS.md documents the `but agent` noun (build-gate per T-IDENT-032..035) — verified by `grep 'but agent' crates/but/AGENTS.md && grep 'but-authz/README.md' crates/but/AGENTS.md`.
- AC-4: crates/WORKSPACE_MODEL.md cross-references the identity README (build-gate per T-IDENT-032..035) — verified by `grep 'but-authz/README.md' crates/WORKSPACE_MODEL.md`.
- Honors NEVER: Never create the README here — it is IDENT-023's deliverable (this task only links to it).

## Dependencies

- **Depends on:** IDENT-023
- **Blocks:** none
- **Capabilities:** CAP-AUTHZ-01, CAP-CONFIG-01

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "IDENT-024",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "identity_docs_unlinked": {
      "description": "crates/AGENTS.md, crates/but/AGENTS.md, DEVELOPMENT.md (## Code Hitlist), crates/WORKSPACE_MODEL.md all present but none reference the identity README or the rename; crates/but-authz/README.md exists (IDENT-023)",
      "seed_method": "public_api",
      "records": [
        "crates/AGENTS.md exists with `## API Boundaries and Legacy` etc., no but-authz/README link",
        "DEVELOPMENT.md `## Code Hitlist` at line ~534 lists gitbutler-* crates, no permissions.toml->agents.toml entry",
        "crates/but-authz/README.md exists (created by IDENT-023)"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN crates/AGENTS.md with no identity-model cross-reference WHEN Adding a cross-reference to the identity README THEN crates/AGENTS.md contains a link/path to `but-authz/README.md` for the identity model",
      "test_tier": "integration",
      "verification_service": "source",
      "verify": "grep 'but-authz/README.md' crates/AGENTS.md",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "source",
        "start_ref": "identity_docs_unlinked",
        "must_observe": [
          "crates/AGENTS.md contains literal 'but-authz/README.md'",
          "reference framed as the identity model"
        ],
        "must_not_observe": [
          "cross-reference absent",
          "empty/unchanged doc",
          "broken link to a non-existent file"
        ],
        "negative_control": {
          "would_fail_if": [
            "cross-reference not added",
            "link points to a non-existent path",
            "grep stubbed or not executed",
            "doc unchanged from start"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "identity_docs_unlinked",
            "action": {
              "actor": "test_harness",
              "steps": [
                "grep crates/AGENTS.md for 'but-authz/README.md'",
                "confirm the linked file exists"
              ]
            },
            "end_state": {
              "must_observe": [
                "`crates/AGENTS.md` contains `but-authz/README.md` link",
                "target `crates/but-authz/README.md` exists"
              ],
              "must_not_observe": [
                "link absent (0 matches)",
                "dangling link (none resolves)"
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
      "description": "GIVEN DEVELOPMENT.md `## Code Hitlist` WHEN Adding the rename as a tracked migration THEN Hitlist contains an entry naming the `permissions.toml` -> `agents.toml` rename",
      "test_tier": "integration",
      "verification_service": "source",
      "verify": "awk '/^## Code Hitlist/{h=1} h&&/agents.toml/&&/permissions.toml/{print} /^## /{if($0!~/Code Hitlist/)h=0}' DEVELOPMENT.md | grep agents.toml",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "source",
        "start_ref": "identity_docs_unlinked",
        "must_observe": [
          "Code Hitlist entry naming 'permissions.toml' and 'agents.toml'",
          "framed as a tracked rename/migration"
        ],
        "must_not_observe": [
          "no Hitlist entry for the rename",
          "entry placed outside the Code Hitlist section",
          "unchanged doc"
        ],
        "negative_control": {
          "would_fail_if": [
            "Hitlist entry not added",
            "rename named only outside Code Hitlist",
            "grep/awk stubbed or not executed",
            "only one of the two filenames present"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "identity_docs_unlinked",
            "action": {
              "actor": "test_harness",
              "steps": [
                "scan the `## Code Hitlist` section",
                "assert an entry names both permissions.toml and agents.toml"
              ]
            },
            "end_state": {
              "must_observe": [
                "Code Hitlist entry names `permissions.toml` + `agents.toml`",
                "entry within `## Code Hitlist`"
              ],
              "must_not_observe": [
                "rename entry absent (0 matches)",
                "entry outside section (none in Hitlist)"
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
      "description": "GIVEN crates/but/AGENTS.md WHEN Adding documentation of the `but agent` CLI noun THEN crates/but/AGENTS.md names `but agent` (register/migrate/whoami) and cross-references the identity README",
      "test_tier": "integration",
      "verification_service": "source",
      "verify": "grep 'but agent' crates/but/AGENTS.md && grep 'but-authz/README.md' crates/but/AGENTS.md",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "source",
        "start_ref": "identity_docs_unlinked",
        "must_observe": [
          "literal 'but agent'",
          "literal 'but-authz/README.md' link",
          "noun framed as identity surface"
        ],
        "must_not_observe": [
          "no `but agent` mention",
          "no cross-reference",
          "unchanged doc"
        ],
        "negative_control": {
          "would_fail_if": [
            "`but agent` not documented",
            "cross-reference omitted",
            "grep stubbed or not executed"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "identity_docs_unlinked",
            "action": {
              "actor": "test_harness",
              "steps": [
                "grep crates/but/AGENTS.md for 'but agent' + 'but-authz/README.md'"
              ]
            },
            "end_state": {
              "must_observe": [
                "`crates/but/AGENTS.md` contains `but agent`",
                "links `but-authz/README.md`"
              ],
              "must_not_observe": [
                "`but agent` undocumented (0 matches)",
                "no cross-reference (none)"
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
      "description": "GIVEN crates/WORKSPACE_MODEL.md WHEN Adding a cross-reference to the identity README THEN crates/WORKSPACE_MODEL.md links to `but-authz/README.md` for the agent-identity model",
      "test_tier": "integration",
      "verification_service": "source",
      "verify": "grep 'but-authz/README.md' crates/WORKSPACE_MODEL.md",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "source",
        "start_ref": "identity_docs_unlinked",
        "must_observe": [
          "literal 'but-authz/README.md' in WORKSPACE_MODEL.md",
          "reference framed as identity model"
        ],
        "must_not_observe": [
          "cross-reference absent",
          "broken link",
          "unchanged doc"
        ],
        "negative_control": {
          "would_fail_if": [
            "cross-reference not added",
            "link to non-existent path",
            "grep stubbed or not executed"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "identity_docs_unlinked",
            "action": {
              "actor": "test_harness",
              "steps": [
                "grep crates/WORKSPACE_MODEL.md for 'but-authz/README.md'",
                "confirm linked file exists"
              ]
            },
            "end_state": {
              "must_observe": [
                "`crates/WORKSPACE_MODEL.md` contains `but-authz/README.md`",
                "target file `crates/but-authz/README.md` exists"
              ],
              "must_not_observe": [
                "link absent (0 matches)",
                "dangling link (none resolves)"
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
      "description": "crates/AGENTS.md links to but-authz/README.md is true",
      "maps_to_ac": "AC-1",
      "verify": "grep -q 'but-authz/README.md' crates/AGENTS.md"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "DEVELOPMENT.md Code Hitlist lists the permissions.toml->agents.toml rename is true",
      "maps_to_ac": "AC-2",
      "verify": "awk '/^## Code Hitlist/{h=1} h&&/agents.toml/&&/permissions.toml/{f=1} /^## /{if($0!~/Code Hitlist/)h=0} END{exit !f}' DEVELOPMENT.md"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "crates/but/AGENTS.md documents the `but agent` noun and links the README is true",
      "maps_to_ac": "AC-3",
      "verify": "grep -q 'but agent' crates/but/AGENTS.md && grep -q 'but-authz/README.md' crates/but/AGENTS.md"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "crates/WORKSPACE_MODEL.md links to but-authz/README.md is true",
      "maps_to_ac": "AC-4",
      "verify": "grep -q 'but-authz/README.md' crates/WORKSPACE_MODEL.md"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "Every cross-referenced README link resolves to an existing file is true",
      "maps_to_ac": "AC-1",
      "verify": "test -f crates/but-authz/README.md"
    }
  ]
}
-->
