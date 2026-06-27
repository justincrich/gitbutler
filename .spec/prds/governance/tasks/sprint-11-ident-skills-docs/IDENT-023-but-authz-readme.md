# IDENT-023 — `crates/but-authz/README.md` (NEW) — threat model, file layout, migration path, env-var deprecation timeline, examples

**Sprint:** [Sprint 11](./SPRINT.md) · **Agent:** `rust-implementer` · **Estimate:** 120 min · **Type:** FEATURE · **Status:** Backlog · **Proposed By:** rust-planner

## Background

rust-implementer owns `crates/but-authz/` and can grep the live engine (authorize.rs, registry.rs, process.rs, config.rs, denial.rs) to write an accurate threat model + file layout rather than a guessed one; the crate README is implementer-owned documentation.

**Provides:** (documentation/consumer layer — no new capability)

**Consumes:** resolve_principal_with_registry resolution order, Denial::unregistered / Denial::stale_registration (perm.denied), agents.toml committed config (gix ref-pinned), agents-runtime.toml runtime registry (mode 0600, BUT_AGENT_REGISTRY_PATH), process_start_time PID-reuse defense (Linux /proc stat field 22; macOS proc_pidinfo)

**Boundary contracts:**
- The README's documented identity contract (resolution order, file roles, env-handle gate, denial codes) matches the engine's resolve_principal_with_registry behavior and the agents.toml/agents-runtime.toml file model.

## Critical Constraints

**MUST:**
- Create `crates/but-authz/README.md` (new file) documenting the honest threat model: spoofing collapses to writing the registry file you already have fs access to; cross-host non-repudiation, crypto signatures, keychain, and sandboxing are explicitly OUT OF SCOPE.
- Document the file layout: committed ref-pinned `.gitbutler/agents.toml` (`[[agent]]` blocks) vs gitignored runtime `agents-runtime.toml` (mode 0600, `(pid,start_time,expiry)->agent_id`, `BUT_AGENT_REGISTRY_PATH` override).
- Document the migration path (`but agent migrate`; one-release `permissions.toml` legacy fallback) and the env-var deprecation timeline (`BUT_AGENT_HANDLE` test-only behind `BUT_AUTHZ_ALLOW_ENV_HANDLE=1`).
- Ground every claim in the real source modules (authorize.rs, registry.rs, process.rs, config.rs, denial.rs) — grep them, do not invent.

**NEVER:**
- Never overstate the security guarantee (no claims of cryptographic identity, cross-host trust, or spoof-proofing).
- Never edit `crates/but-authz/src/**` or any code — this is a NEW doc file only.
- Never describe `BUT_AGENT_HANDLE` as the production identity path.

**STRICTLY:**
- BLOCKED-UNTIL Sprint-10 (final env-handle deny-default locked) and transitively Sprint-09 (`but agent migrate` verb + agents.toml loader + 8-callsite swap) — the README cannot honestly describe the registry path as the enforced default until it is.
- Documentation only — describe existing engine behavior; introduce no new contract.

## Specification

**Objective:** Author crates/but-authz/README.md documenting the agent-identity model — threat model, agents.toml vs agents-runtime.toml file layout, resolution order, denial codes, migration path, and the BUT_AGENT_HANDLE deprecation timeline — accurately reflecting the but-authz engine.

**Success state:** crates/but-authz/README.md exists and contains four anchored sections (Threat model, File layout, Migration path, Env-var deprecation) plus a worked example; grep finds the honest-threat phrasing, both file names, `but agent migrate`, and `BUT_AUTHZ_ALLOW_ENV_HANDLE`.

## Acceptance Criteria

**AC-1 (PRIMARY)** — README threat model: 4 out-of-scope items + process-level mechanism, no overclaim (build-gate per T-IDENT-032..035)
- **GIVEN:** crates/but-authz/ with no README.md
- **WHEN:** Creating crates/but-authz/README.md with a Threat model section
- **THEN:** README names cross-host, cryptographic-signature, keychain, and sandbox as out of scope, names the process-level (pid+start_time) mechanism, and makes no spoof-prevention/cryptographic-identity overclaim
- **Verify:** `test -f crates/but-authz/README.md && grep -iE 'cross.host' crates/but-authz/README.md && grep -iE 'crypto.*signature|cryptographic.*signature' crates/but-authz/README.md && grep -iE 'keychain' crates/but-authz/README.md && grep -iE 'sandbox' crates/but-authz/README.md && grep -iE 'process.level|pid.*start_time|start_time.*pid' crates/but-authz/README.md && ! grep -iE 'prevents spoofing|provides cryptographic|guarantees identity|ensures identity|spoofing is impossible' crates/but-authz/README.md`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** source · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=but_authz_no_readme`; must_observe = [README names `cross-host`, `cryptographic signature`, `keychain`, `sandbox` as out of scope; names mechanism `pid` + `start_time` (process-level)]; must_not_observe = [affirmative overclaim `prevents spoofing` / `provides cryptographic identity guarantees` present; README absent (0 bytes)]; negative_control.would_fail_if = [README not created (absent); threat-model section omitted; grep stubbed or not executed; out-of-scope caveats removed].

**AC-2** — File layout section documents agents.toml vs agents-runtime.toml (build-gate per T-IDENT-032..035)
- **GIVEN:** The README
- **WHEN:** Reading the File layout section
- **THEN:** Section names committed `agents.toml` (`[[agent]]`, ref-pinned) and gitignored runtime `agents-runtime.toml` (mode 0600, `BUT_AGENT_REGISTRY_PATH` override)
- **Verify:** `grep 'agents.toml' crates/but-authz/README.md && grep 'agents-runtime.toml' crates/but-authz/README.md && grep 'BUT_AGENT_REGISTRY_PATH' crates/but-authz/README.md && grep '0600' crates/but-authz/README.md`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** source · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=but_authz_no_readme`; must_observe = [literal 'agents.toml'; literal 'agents-runtime.toml'; literal 'BUT_AGENT_REGISTRY_PATH'; literal '0600']; must_not_observe = [only one file documented; section absent; empty file]; negative_control.would_fail_if = [file-layout section omitted; runtime-file role not documented; grep stubbed or not executed; mode/override literals removed].

**AC-3** — Migration path section documents `but agent migrate` + legacy fallback (build-gate per T-IDENT-032..035)
- **GIVEN:** The README
- **WHEN:** Reading the Migration path section
- **THEN:** Section names `but agent migrate` and the one-release `permissions.toml` legacy-fallback window
- **Verify:** `grep 'but agent migrate' crates/but-authz/README.md && grep 'permissions.toml' crates/but-authz/README.md`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** source · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=but_authz_no_readme`; must_observe = [literal 'but agent migrate'; literal 'permissions.toml'; legacy-fallback window described]; must_not_observe = [migration path absent; no mention of legacy file; empty file]; negative_control.would_fail_if = [migration section omitted; 'but agent migrate' literal removed; grep stubbed or not executed].

**AC-4** — Env-var deprecation timeline documents BUT_AGENT_HANDLE as test-only (build-gate per T-IDENT-032..035)
- **GIVEN:** The README
- **WHEN:** Reading the env-var deprecation section
- **THEN:** Section states `BUT_AGENT_HANDLE` is test-only behind `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` and slated for deprecation
- **Verify:** `grep 'BUT_AGENT_HANDLE' crates/but-authz/README.md && grep 'BUT_AUTHZ_ALLOW_ENV_HANDLE' crates/but-authz/README.md && grep -iE 'deprecat' crates/but-authz/README.md`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** source · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=but_authz_no_readme`; must_observe = [literal 'BUT_AGENT_HANDLE'; literal 'BUT_AUTHZ_ALLOW_ENV_HANDLE'; 'deprecat' wording; test-only framing]; must_not_observe = [env var as default path; deprecation section absent; empty file]; negative_control.would_fail_if = [deprecation timeline omitted; gate var literal removed; grep stubbed or not executed; env var documented as normal path].

**AC-5** — Resolution order + worked example present and accurate (build-gate per T-IDENT-032..035)
- **GIVEN:** The README
- **WHEN:** Reading the resolution-order/example section
- **THEN:** Section documents resolution order (registry -> flag-gated env -> denial) naming `resolve_principal_with_registry` and `Denial::unregistered`, with a worked `but agent register` example
- **Verify:** `grep -q 'resolve_principal_with_registry' crates/but-authz/README.md && grep -q 'Denial::unregistered' crates/but-authz/README.md && grep -q 'but agent register' crates/but-authz/README.md`
- **TEST_TIER:** integration · **VERIFICATION_SERVICE:** source · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=but_authz_no_readme`; must_observe = [literal 'resolve_principal_with_registry'; literal 'Denial::unregistered'; worked example containing 'but agent register']; must_not_observe = [resolution order absent; no example block; empty file]; negative_control.would_fail_if = [resolution order / example omitted; engine symbol literals removed; grep stubbed or not executed; example invents non-existent verbs].

## Test Criteria

| ID | Boolean | Maps to AC |
|----|---------|------------|
| TC-1 | README names all 4 out-of-scope items and makes no spoof-prevention overclaim is true | AC-1 |
| TC-2 | README names both agents.toml and agents-runtime.toml with mode 0600 is true | AC-2 |
| TC-3 | README documents `but agent migrate` migration path is true | AC-3 |
| TC-4 | README documents BUT_AGENT_HANDLE deprecation behind BUT_AUTHZ_ALLOW_ENV_HANDLE is true | AC-4 |
| TC-5 | README documents resolution order + a worked `but agent register` example is true | AC-5 |

## Reading List

1. `crates/but-authz/src/authorize.rs`:60-248 — PRIMARY PATTERN — resolution order, BUT_AUTHZ_ALLOW_ENV_HANDLE gate, Denial::unregistered/stale_registration; quote these accurately.
2. `crates/but-authz/src/registry.rs`:1-120 — Runtime registry shape: (pid,start_time,expiry)->agent_id, atomic write, GC, mode 0600 — the agents-runtime.toml description.
3. `crates/but-authz/src/process.rs`:1-60 — PID-reuse defense: Linux /proc/[pid]/stat field 22, macOS proc_pidinfo PROC_PIDTBSDINFO — the threat-model detail.
4. `crates/but-authz/src/config.rs`:1-80 — AgentWire/AgentsWire vs legacy PrincipalWire/PermissionsWire; agents.toml committed config + gix ref-pin — the file-layout section.
5. `.spec/prds/governance/12-uc-agent-identity.md`:9-19 — Honest threat-model paragraph + UC-IDENT-01..05 scope to mirror in the README.

## Guardrails

**WRITE-ALLOWED:**
- crates/but-authz/README.md (NEW — threat model, file layout, migration path, env-var deprecation, examples)

**WRITE-PROHIBITED:**
- crates/but-authz/src/** - no code changes; README is a new doc file describing existing behavior
- RULES.md / crates/AGENTS.md - cross-referencing is IDENT-022/IDENT-024 scope
- Any file other than crates/but-authz/README.md

## Code Pattern

**Reference:** crates/but-authz/src/authorize.rs, crates/but-authz/src/registry.rs, crates/but-authz/src/config.rs, .spec/prds/governance/12-uc-agent-identity.md

**Pattern:** Crate README with anchored sections: ## Threat model (honest scope + out-of-scope list), ## File layout (agents.toml committed vs agents-runtime.toml runtime, mode 0600, BUT_AGENT_REGISTRY_PATH), ## Resolution order (registry -> flag-gated env -> denial, naming resolve_principal_with_registry + Denial variants), ## Migration path (but agent migrate; permissions.toml one-release fallback), ## Env-var deprecation (BUT_AGENT_HANDLE test-only behind BUT_AUTHZ_ALLOW_ENV_HANDLE=1), ## Example (but agent register usage).

**Source:** `The UC-IDENT-05 PRD prose + the but-authz source modules (the README is a faithful narration of them)`

**Anti-pattern:** Do NOT claim cryptographic/cross-host identity guarantees; do NOT invent verbs or file names not present in the engine; do NOT present the env handle as a production path.

## Agent Instructions

TDD RED→GREEN per AC (build-gate/source greps + real `but`/skill execution against real files — NO mocks):
1. **RED:** write each AC's failing check first (against the current start state — docs/skill absent or still self-asserting `BUT_AGENT_HANDLE`).
2. **GREEN:** make the minimal edit (docs/skill/migration) to satisfy the AC.
3. For brain-skill tasks: edit the CANONICAL copy under `~/Projects/brain/skills/`, MIRROR to `~/.claude/skills/`, then `diff -rq` clean.
4. Run each AC's verify command; commit via `but commit` (governed).

## Orchestrator Verification Protocol

- `test -f crates/but-authz/README.md && grep -iqE 'out of scope' crates/but-authz/README.md` → exit 0
- `grep -q 'agents-runtime.toml' crates/but-authz/README.md && grep -q 'but agent migrate' crates/but-authz/README.md && grep -q 'BUT_AUTHZ_ALLOW_ENV_HANDLE' crates/but-authz/README.md` → exit 0
- `git diff --name-only` → only crates/but-authz/README.md

## Agent Assignment

**Agent:** `rust-implementer` — rust-implementer owns `crates/but-authz/` and can grep the live engine (authorize.rs, registry.rs, process.rs, config.rs, denial.rs) to write an accurate threat model + file layout rather than a guessed one; the crate README is implementer-owned documentation.
**Pairing:** none (single-surface Rust/docs/skill task). Honors `crates/AGENTS.md` + `BUT-SKILL-CONVENTIONS.md`.

## Evidence Gates

- `test -f crates/but-authz/README.md && grep -iqE 'out of scope' crates/but-authz/README.md` (exit 0)
- `grep -q 'agents-runtime.toml' crates/but-authz/README.md && grep -q 'but agent migrate' crates/but-authz/README.md && grep -q 'BUT_AUTHZ_ALLOW_ENV_HANDLE' crates/but-authz/README.md` (exit 0)
- `git diff --name-only` (only crates/but-authz/README.md)

## Review Criteria

- AC-1: PRIMARY — README threat model: 4 out-of-scope items + process-level mechanism, no overclaim (build-gate per T-IDENT-032..035) — verified by `test -f crates/but-authz/README.md && grep -iE 'cross.host' crates/but-authz/README.md && grep -iE 'crypto.*signature|cryptographic.*signature' crates/but-authz/README.md && grep -iE 'keychain' crates/but-authz/README.md && grep -iE 'sandbox' crates/but-authz/README.md && grep -iE 'process.level|pid.*start_time|start_time.*pid' crates/but-authz/README.md && ! grep -iE 'prevents spoofing|provides cryptographic|guarantees identity|ensures identity|spoofing is impossible' crates/but-authz/README.md`.
- AC-2: File layout section documents agents.toml vs agents-runtime.toml (build-gate per T-IDENT-032..035) — verified by `grep 'agents.toml' crates/but-authz/README.md && grep 'agents-runtime.toml' crates/but-authz/README.md && grep 'BUT_AGENT_REGISTRY_PATH' crates/but-authz/README.md && grep '0600' crates/but-authz/README.md`.
- AC-3: Migration path section documents `but agent migrate` + legacy fallback (build-gate per T-IDENT-032..035) — verified by `grep 'but agent migrate' crates/but-authz/README.md && grep 'permissions.toml' crates/but-authz/README.md`.
- AC-4: Env-var deprecation timeline documents BUT_AGENT_HANDLE as test-only (build-gate per T-IDENT-032..035) — verified by `grep 'BUT_AGENT_HANDLE' crates/but-authz/README.md && grep 'BUT_AUTHZ_ALLOW_ENV_HANDLE' crates/but-authz/README.md && grep -iE 'deprecat' crates/but-authz/README.md`.
- AC-5: Resolution order + worked example present and accurate (build-gate per T-IDENT-032..035) — verified by `grep -q 'resolve_principal_with_registry' crates/but-authz/README.md && grep -q 'Denial::unregistered' crates/but-authz/README.md && grep -q 'but agent register' crates/but-authz/README.md`.
- Honors NEVER: Never overstate the security guarantee (no claims of cryptographic identity, cross-host trust, or spoof-proofing).

## Dependencies

- **Depends on:** none (BLOCKED-UNTIL Sprint-10 per Critical Constraints)
- **Blocks:** IDENT-024
- **Capabilities:** CAP-AUTHZ-01, CAP-CONFIG-01

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "IDENT-023",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "but_authz_no_readme": {
      "description": "crates/but-authz/ with src/ engine (authorize.rs, registry.rs, process.rs, config.rs, denial.rs) but no README.md",
      "seed_method": "public_api",
      "records": [
        "crates/but-authz/src/authorize.rs documents resolution order at lines 185-248",
        "crates/but-authz/src/registry.rs implements the runtime registry",
        "crates/but-authz/README.md is ABSENT at start"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN crates/but-authz/ with no README.md WHEN Creating crates/but-authz/README.md with a Threat model section THEN README names cross-host, cryptographic-signature, keychain, and sandbox as out of scope, names the process-level (pid+start_time) mechanism, and makes no spoof-prevention/cryptographic-identity overclaim",
      "test_tier": "integration",
      "verification_service": "source",
      "verify": "test -f crates/but-authz/README.md && grep -iE 'cross.host' crates/but-authz/README.md && grep -iE 'crypto.*signature|cryptographic.*signature' crates/but-authz/README.md && grep -iE 'keychain' crates/but-authz/README.md && grep -iE 'sandbox' crates/but-authz/README.md && grep -iE 'process.level|pid.*start_time|start_time.*pid' crates/but-authz/README.md && ! grep -iE 'prevents spoofing|provides cryptographic|guarantees identity|ensures identity|spoofing is impossible' crates/but-authz/README.md",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "source",
        "start_ref": "but_authz_no_readme",
        "must_observe": [
          "README names `cross-host`, `cryptographic signature`, `keychain`, `sandbox` as out of scope",
          "names mechanism `pid` + `start_time` (process-level)"
        ],
        "must_not_observe": [
          "affirmative overclaim `prevents spoofing` / `provides cryptographic identity guarantees` present",
          "README absent (0 bytes)"
        ],
        "negative_control": {
          "would_fail_if": [
            "README not created (absent)",
            "threat-model section omitted",
            "grep stubbed or not executed",
            "out-of-scope caveats removed"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "but_authz_no_readme",
            "action": {
              "actor": "test_harness",
              "steps": [
                "assert crates/but-authz/README.md exists",
                "grep for 'Threat model' + 'out of scope' + fs-access phrasing"
              ]
            },
            "end_state": {
              "must_observe": [
                "README names `cross-host`, `cryptographic signature`, `keychain`, `sandbox` as out of scope",
                "names mechanism `pid` + `start_time` (process-level)"
              ],
              "must_not_observe": [
                "affirmative overclaim `prevents spoofing` / `provides cryptographic identity guarantees` present",
                "README absent (0 bytes)"
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
      "description": "GIVEN The README WHEN Reading the File layout section THEN Section names committed `agents.toml` (`[[agent]]`, ref-pinned) and gitignored runtime `agents-runtime.toml` (mode 0600, `BUT_AGENT_REGISTRY_PATH` override)",
      "test_tier": "integration",
      "verification_service": "source",
      "verify": "grep 'agents.toml' crates/but-authz/README.md && grep 'agents-runtime.toml' crates/but-authz/README.md && grep 'BUT_AGENT_REGISTRY_PATH' crates/but-authz/README.md && grep '0600' crates/but-authz/README.md",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "source",
        "start_ref": "but_authz_no_readme",
        "must_observe": [
          "literal 'agents.toml'",
          "literal 'agents-runtime.toml'",
          "literal 'BUT_AGENT_REGISTRY_PATH'",
          "literal '0600'"
        ],
        "must_not_observe": [
          "only one file documented",
          "section absent",
          "empty file"
        ],
        "negative_control": {
          "would_fail_if": [
            "file-layout section omitted",
            "runtime-file role not documented",
            "grep stubbed or not executed",
            "mode/override literals removed"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "but_authz_no_readme",
            "action": {
              "actor": "test_harness",
              "steps": [
                "grep README for both file names + 0600 + BUT_AGENT_REGISTRY_PATH"
              ]
            },
            "end_state": {
              "must_observe": [
                "contains `agents.toml` + `agents-runtime.toml`",
                "names `BUT_AGENT_REGISTRY_PATH` + `0600`"
              ],
              "must_not_observe": [
                "runtime file undocumented (0 matches)",
                "only one file (none of the pair)"
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
      "description": "GIVEN The README WHEN Reading the Migration path section THEN Section names `but agent migrate` and the one-release `permissions.toml` legacy-fallback window",
      "test_tier": "integration",
      "verification_service": "source",
      "verify": "grep 'but agent migrate' crates/but-authz/README.md && grep 'permissions.toml' crates/but-authz/README.md",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "source",
        "start_ref": "but_authz_no_readme",
        "must_observe": [
          "literal 'but agent migrate'",
          "literal 'permissions.toml'",
          "legacy-fallback window described"
        ],
        "must_not_observe": [
          "migration path absent",
          "no mention of legacy file",
          "empty file"
        ],
        "negative_control": {
          "would_fail_if": [
            "migration section omitted",
            "'but agent migrate' literal removed",
            "grep stubbed or not executed"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "but_authz_no_readme",
            "action": {
              "actor": "test_harness",
              "steps": [
                "grep README for 'but agent migrate' + 'permissions.toml'"
              ]
            },
            "end_state": {
              "must_observe": [
                "contains `but agent migrate`",
                "names legacy `permissions.toml` fallback"
              ],
              "must_not_observe": [
                "migrate verb missing (0 matches)",
                "no legacy-file mention (none)"
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
      "description": "GIVEN The README WHEN Reading the env-var deprecation section THEN Section states `BUT_AGENT_HANDLE` is test-only behind `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` and slated for deprecation",
      "test_tier": "integration",
      "verification_service": "source",
      "verify": "grep 'BUT_AGENT_HANDLE' crates/but-authz/README.md && grep 'BUT_AUTHZ_ALLOW_ENV_HANDLE' crates/but-authz/README.md && grep -iE 'deprecat' crates/but-authz/README.md",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "source",
        "start_ref": "but_authz_no_readme",
        "must_observe": [
          "literal 'BUT_AGENT_HANDLE'",
          "literal 'BUT_AUTHZ_ALLOW_ENV_HANDLE'",
          "'deprecat' wording",
          "test-only framing"
        ],
        "must_not_observe": [
          "env var as default path",
          "deprecation section absent",
          "empty file"
        ],
        "negative_control": {
          "would_fail_if": [
            "deprecation timeline omitted",
            "gate var literal removed",
            "grep stubbed or not executed",
            "env var documented as normal path"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "but_authz_no_readme",
            "action": {
              "actor": "test_harness",
              "steps": [
                "grep README for both env literals + 'deprecat'"
              ]
            },
            "end_state": {
              "must_observe": [
                "contains `BUT_AGENT_HANDLE` + `BUT_AUTHZ_ALLOW_ENV_HANDLE`",
                "states `deprecated` test-only"
              ],
              "must_not_observe": [
                "gate var missing (0 matches)",
                "env var as default (no deprecation)"
              ]
            }
          }
        ]
      },
      "criteria_layer": "build-gate"
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN The README WHEN Reading the resolution-order/example section THEN Section documents resolution order (registry -> flag-gated env -> denial) naming `resolve_principal_with_registry` and `Denial::unregistered`, with a worked `but agent register` example",
      "test_tier": "integration",
      "verification_service": "source",
      "verify": "grep -q 'resolve_principal_with_registry' crates/but-authz/README.md && grep -q 'Denial::unregistered' crates/but-authz/README.md && grep -q 'but agent register' crates/but-authz/README.md",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "source",
        "start_ref": "but_authz_no_readme",
        "must_observe": [
          "literal 'resolve_principal_with_registry'",
          "literal 'Denial::unregistered'",
          "worked example containing 'but agent register'"
        ],
        "must_not_observe": [
          "resolution order absent",
          "no example block",
          "empty file"
        ],
        "negative_control": {
          "would_fail_if": [
            "resolution order / example omitted",
            "engine symbol literals removed",
            "grep stubbed or not executed",
            "example invents non-existent verbs"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "but_authz_no_readme",
            "action": {
              "actor": "test_harness",
              "steps": [
                "grep README for resolve_principal_with_registry + Denial::unregistered",
                "grep example block for 'but agent register'"
              ]
            },
            "end_state": {
              "must_observe": [
                "contains `resolve_principal_with_registry` + `Denial::unregistered`",
                "worked example with `but agent register`"
              ],
              "must_not_observe": [
                "resolution order absent (0 matches)",
                "no example block (none)"
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
      "description": "README names all 4 out-of-scope items and makes no spoof-prevention overclaim is true",
      "maps_to_ac": "AC-1",
      "verify": "grep -iE 'cross.host' crates/but-authz/README.md && grep -iE 'keychain' crates/but-authz/README.md && grep -iE 'sandbox' crates/but-authz/README.md && ! grep -iE 'prevents spoofing|provides cryptographic|guarantees identity|ensures identity|spoofing is impossible' crates/but-authz/README.md"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "README names both agents.toml and agents-runtime.toml with mode 0600 is true",
      "maps_to_ac": "AC-2",
      "verify": "grep -q 'agents-runtime.toml' crates/but-authz/README.md && grep -q '0600' crates/but-authz/README.md"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "README documents `but agent migrate` migration path is true",
      "maps_to_ac": "AC-3",
      "verify": "grep -q 'but agent migrate' crates/but-authz/README.md"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "README documents BUT_AGENT_HANDLE deprecation behind BUT_AUTHZ_ALLOW_ENV_HANDLE is true",
      "maps_to_ac": "AC-4",
      "verify": "grep -q 'BUT_AUTHZ_ALLOW_ENV_HANDLE' crates/but-authz/README.md && grep -qiE 'deprecat' crates/but-authz/README.md"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "README documents resolution order + a worked `but agent register` example is true",
      "maps_to_ac": "AC-5",
      "verify": "grep -q 'resolve_principal_with_registry' crates/but-authz/README.md && grep -q 'but agent register' crates/but-authz/README.md"
    }
  ]
}
-->
