# IDENT-028 — Migrate `agent-intel` (and any second governed repo) via `but agent migrate`; verify end-to-end governed action post-migration

**Sprint:** [Sprint 11](./SPRINT.md) · **Agent:** `rust-implementer` · **Estimate:** 60 min · **Type:** FEATURE · **Status:** Backlog · **Proposed By:** rust-planner

## Background

rust-implementer owns the `but` CLI runtime and is the right hand to run the real `but agent migrate` against the live `agent-intel` repo and prove an end-to-end governed `but commit` via the registry path — this is field-execution + verification of the engine, its domain.

**Provides:** (documentation/consumer layer — no new capability)

**Consumes:** but agent migrate verb (permissions.toml -> agents.toml rewrite), but agent register + resolve_principal_with_registry (registry path enforced), Denial::unregistered (perm.denied) for unregistered callers

**Boundary contracts:**
- After `but agent migrate`, agent-intel authorizes governed actions via the registry path (registered PID -> commit succeeds; unregistered -> perm.denied), with agents.toml byte-equivalent to the prior permissions.toml roster.

## Critical Constraints

**MUST:**
- Resolve a REAL agent-intel agent id at runtime — `AGENT_ID=$(cd ~/Projects/agent-intel && but agent list --committed | grep -oE 'operator|electron-implementer|node-implementer' | head -1)` — and use "$AGENT_ID" for register + the roster grep; never hardcode rust-* ids (agent-intel has none). Guard the resolution: `test -n "$AGENT_ID" || { echo ERROR...; exit 1; }` so an empty id fails cleanly (never runs BUT_AGENT_HANDLE='').
- The migrate flow runs before any agents.toml is committed, so BOTH `but agent migrate` AND its bootstrap commit are wrapped in the EXPLICIT env-handle escape hatch `BUT_AUTHZ_ALLOW_ENV_HANDLE=1 BUT_AGENT_HANDLE=$AGENT_ID` (migrate is 'same pattern as perm_grant', an admin-gated callsite, so a bare `but agent migrate` could be denied); this is bootstrap-only, NOT the registry path.
- Prove the registry path with a SECOND commit (after `but agent register --pid $$ --as $AGENT_ID`) carrying NO env flag and NO BUT_AGENT_HANDLE; assert a new HEAD SHA.
- During RED, BEFORE migrating, snapshot the pre-migration `permissions.toml` at a pinned ref; assert roster/GovConfig equivalence (every pre-migration id survives as an agent id) against that snapshot — independent of how many commits AC-1 creates.

**NEVER:**
- Never hardcode `rust-implementer`/`rust-reviewer`/`orchestrator` — they are NOT in agent-intel's committed roster.
- Never invert the denial check (`! ... | grep perm.denied` passes when the unregistered commit SUCCEEDS) — assert the denial directly.
- Never fabricate the registry-path commit success — the second `but commit` must actually land via the registry.
- Never touch agent-intel's `.spec/` content; restore README after the denial probe (`git restore README.md`).

**STRICTLY:**
- BLOCKED-UNTIL Sprint-10 (final env-handle deny-default locked) and transitively Sprint-09 (`but agent migrate` verb + agents.toml loader + 8-callsite swap) — the registry path must be the enforced default before a field migration is honest.
- BLOCKED-UNTIL IDENT-026 (the but-migrate skill flow this exercises must exist).
- Open question for the implementer: whether `but agent migrate` is itself admin-gated is unspecified upstream; AC-1 bootstrap-wraps it so it is robust either way.

## Specification

**Objective:** Run `but agent migrate` against the agent-intel repo (and any second governed repo), commit the agents.toml rename, and prove an end-to-end governed `but commit` succeeds via the registry path while an unregistered process is denied.

**Success state:** agent-intel has committed agents.toml (permissions.toml removed); `but agent list --committed` shows the migrated roster; a registered process's governed `but commit` lands; an unregistered process's governed action is denied with perm.denied; agents.toml is byte-equivalent to the prior permissions.toml roster.

## Acceptance Criteria

**AC-1 (PRIMARY)** — Migrate + bootstrap commit (env-handle escape hatch), then a registry-path commit lands
- **GIVEN:** the agent-intel repo with a committed permissions.toml and no agents.toml; AGENT_ID resolved from the committed roster
- **WHEN:** running `but agent migrate` (bootstrap-wrapped with the env handle), committing the rename via the bootstrap escape hatch, then `but agent register --pid $$ --as $AGENT_ID` and a SECOND commit with no env flag
- **THEN:** agents.toml is committed (permissions.toml removed) via the bootstrap escape hatch, and the second commit (registry path, no env handle) lands a new HEAD SHA
- **Verify:** `cd /Users/justinrich/Projects/agent-intel && AGENT_ID=$(but agent list --committed | grep -oE 'operator|electron-implementer|node-implementer' | head -1) && test -n "$AGENT_ID" || { echo 'ERROR: could not resolve an agent id from the committed roster (pre-migration legacy fallback not working)'; exit 1; } && BUT_AUTHZ_ALLOW_ENV_HANDLE=1 BUT_AGENT_HANDLE="$AGENT_ID" but agent migrate && BUT_AUTHZ_ALLOW_ENV_HANDLE=1 BUT_AGENT_HANDLE="$AGENT_ID" but commit -m 'chore: migrate governance to agents.toml' .gitbutler/agents.toml .gitbutler/permissions.toml && git show HEAD:.gitbutler/agents.toml | grep -q '\[\[agent\]\]' && ! git cat-file -e HEAD:.gitbutler/permissions.toml 2>/dev/null && but agent register --pid $$ --as "$AGENT_ID" && before=$(git rev-parse HEAD) && echo y >> README.md && but commit -m 'chore: governed registry-path commit' README.md && test "$(git rev-parse HEAD)" != "$before"`
- **TEST_TIER:** e2e · **VERIFICATION_SERVICE:** but CLI + agent-intel repo (bootstrap escape hatch -> registry path) · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=agent_intel_permissions_repo`; must_observe = [`git show HEAD:.gitbutler/agents.toml` contains `[[agent]]` (bootstrap commit); registry-path commit lands new HEAD SHA `!=` before (1 new commit, no env handle)]; must_not_observe = [`permissions.toml` still committed (start state); registry-path commit denied `perm.denied` (0 lands)]; negative_control.would_fail_if = [`but agent migrate` not run; governed commit succeeds without registration (registry path bypassed); agents.toml not committed; verify stubbed or not executed].

**AC-2** — `but agent list --committed` shows the migrated roster
- **GIVEN:** agent-intel post-migration
- **WHEN:** running `but agent list --committed`
- **THEN:** the migrated specialist roster is printed from committed agents.toml
- **Verify:** `cd /Users/justinrich/Projects/agent-intel && but agent list --committed | grep -Eq 'operator|electron-implementer|node-implementer'`
- **TEST_TIER:** e2e · **VERIFICATION_SERVICE:** but CLI + agent-intel repo · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=agent_intel_permissions_repo`; must_observe = [`but agent list --committed` prints `operator` or `electron-implementer`; >=1 migrated specialist listed]; must_not_observe = [empty roster (0 agents); no agents.toml committed (none)]; negative_control.would_fail_if = [agents.toml not committed; list --committed returns empty (loader disconnected); grep stubbed or not executed].

**AC-3** — Unregistered process is denied (registry path enforced, not env)
- **GIVEN:** agent-intel post-migration with no registration for this process
- **WHEN:** attempting a governed `but commit` from an unregistered process (env handle flag unset)
- **THEN:** the action is denied with `perm.denied` (Denial::unregistered)
- **Verify:** `cd /Users/justinrich/Projects/agent-intel && but agent unregister --pid $$ ; unset BUT_AGENT_HANDLE BUT_AUTHZ_ALLOW_ENV_HANDLE; echo z >> README.md && but commit -m 'should be denied' README.md 2>&1 | grep -q 'perm.denied'; rc=$?; git restore README.md; exit $rc`
- **TEST_TIER:** e2e · **VERIFICATION_SERVICE:** but CLI + agent-intel repo (denial path) · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=agent_intel_permissions_repo`; must_observe = [unregistered `but commit` denied with `perm.denied`; stderr contains `perm.denied` (1 denial)]; must_not_observe = [commit lands without registration (0 denial); env handle accepted (none gated)]; negative_control.would_fail_if = [unregistered commit succeeds (registry not enforced); env handle accepted with flag unset; denial check stubbed or not executed].

**AC-4** — Roster/GovConfig equivalence: every pre-migration principal id survives as an agent id
- **GIVEN:** the pre-migration permissions.toml snapshotted at its last-present (pinned) ref
- **WHEN:** comparing the committed agents.toml roster against the pre-migration permissions.toml ids
- **THEN:** every `id =` from the pinned pre-migration permissions.toml is present in `but agent list --committed`
- **Verify:** `cd /Users/justinrich/Projects/agent-intel && DEL=$(git log --diff-filter=D --format=%H -- .gitbutler/permissions.toml | head -1) && git show "$DEL^:.gitbutler/permissions.toml" > /tmp/agentintel_pre.toml && test -s /tmp/agentintel_pre.toml && while IFS= read -r id; do but agent list --committed | grep -qF "$id" || exit 1; done < <(grep '^id = ' /tmp/agentintel_pre.toml | sed -E 's/id = "(.*)"/\1/')`
- **TEST_TIER:** e2e · **VERIFICATION_SERVICE:** but CLI + agent-intel repo (roster/GovConfig equivalence vs pinned snapshot) · **FLOW_REF:** UC-IDENT-05
- **Scenario:** `start_ref=agent_intel_permissions_repo`; must_observe = [every `id = ` in `/tmp/agentintel_pre.toml` present in `but agent list --committed`; pre-migration snapshot non-empty (>=1 id)]; must_not_observe = [dropped roster entries (0 preserved); empty pre-snapshot (none)]; negative_control.would_fail_if = [migration drops or alters roster entries; permission fields changed; diff stubbed or not executed].

## Test Criteria

| ID | Boolean | Maps to AC |
|----|---------|------------|
| TC-1 | agent-intel has committed agents.toml + removed permissions.toml and a registered governed commit lands is true | AC-1 |
| TC-2 | `but agent list --committed` in agent-intel shows a real migrated id (operator|electron-implementer|node-implementer) is true | AC-2 |
| TC-3 | A governed `but commit` from an unregistered process is denied with perm.denied is true | AC-3 |
| TC-4 | every pre-migration permissions.toml id is present in the committed agents.toml roster is true | AC-4 |

## Reading List

1. `/Users/justinrich/Projects/gitbutler/.spec/prds/governance/12-uc-agent-identity.md`:52-78 — PRIMARY PATTERN — UC-IDENT-03 enforced resolution + UC-IDENT-04 `but agent register/list --committed/migrate/whoami` verbs to exercise.
2. `/Users/justinrich/Projects/gitbutler/crates/but-authz/src/authorize.rs`:185-248 — Resolution order + Denial::unregistered (perm.denied) — the registry-path behavior to verify in the field.
3. `/Users/justinrich/Projects/agent-intel/.gitbutler/permissions.toml`:1-40 — The pre-migration [[principal]] roster — snapshot at the pinned (last-present) ref for AC-4 roster/GovConfig equivalence (real ids: operator, electron-implementer, node-implementer, ...).
4. `/Users/justinrich/Projects/brain/skills/but-migrate/SKILL.md`:54-163 — The migrate flow (IDENT-026) this task exercises end-to-end.

## Guardrails

**WRITE-ALLOWED:**
- /Users/justinrich/Projects/agent-intel/.gitbutler/agents.toml (NEW — written by `but agent migrate`)
- /Users/justinrich/Projects/agent-intel/.gitbutler/permissions.toml (DELETE — removed in the migration commit)
- /Users/justinrich/Projects/agent-intel/README.md (MODIFY — bootstrap/registry-path commit artifact + denial probe; `git restore README.md` after the denial probe)

**WRITE-PROHIBITED:**
- /Users/justinrich/Projects/agent-intel/.spec/** - migration is additive on .gitbutler/; never touch PRD/sprint/task content
- /Users/justinrich/Projects/gitbutler/** - this is a field migration of agent-intel, not the gitbutler repo
- crates/** + brain/skills/** - engine + skills are upstream (IDENT-025/026/027)

## Code Pattern

**Reference:** 12-uc-agent-identity.md UC-IDENT-03/04, crates/but-authz/src/authorize.rs, /Users/justinrich/Projects/agent-intel/.gitbutler/permissions.toml

**Pattern:** Field migration + registry-path proof: (1) `but agent migrate` in agent-intel; (2) commit agents.toml add + permissions.toml delete together; (3) `but agent list --committed` shows the roster; (4) `but agent register --pid $$ --as <id>` then a governed `but commit` lands (registry hit); (5) unregister + unset env flag -> governed action denied with perm.denied (proves the registry path is enforced, not the env handle); (6) round-trip diff agents.toml vs the prior permissions.toml roster.

**Source:** `UC-IDENT-03 AC (register -> commit succeeds; unregister -> perm.denied) + UC-IDENT-01 AC-5/6 byte-equivalent round-trip.`

**Design notes:**
- Resolve the agent-intel path via glob (confirmed at /Users/justinrich/Projects/agent-intel); glob for any second committed permissions.toml under ~/Projects and migrate it too or record N/A.

**Anti-pattern:** Do NOT fabricate the governed-commit success; do NOT hand-edit the toml; do NOT touch agent-intel/.spec/; do NOT skip the unregistered-denial negative control.

## Agent Instructions

TDD RED→GREEN per AC (build-gate/source greps + real `but`/skill execution against real files — NO mocks):
1. **RED:** write each AC's failing check first (against the current start state — docs/skill absent or still self-asserting `BUT_AGENT_HANDLE`).
2. **GREEN:** make the minimal edit (docs/skill/migration) to satisfy the AC.
3. For brain-skill tasks: edit the CANONICAL copy under `~/Projects/brain/skills/`, MIRROR to `~/.claude/skills/`, then `diff -rq` clean.
4. Run each AC's verify command; commit via `but commit` (governed).

## Orchestrator Verification Protocol

- `cd /Users/justinrich/Projects/agent-intel && AGENT_ID=$(but agent list --committed | grep -oE 'operator|electron-implementer|node-implementer' | head -1) && test -n "$AGENT_ID" || { echo 'ERROR: could not resolve an agent id from the committed roster (pre-migration legacy fallback not working)'; exit 1; } && BUT_AUTHZ_ALLOW_ENV_HANDLE=1 BUT_AGENT_HANDLE="$AGENT_ID" but agent migrate && BUT_AUTHZ_ALLOW_ENV_HANDLE=1 BUT_AGENT_HANDLE="$AGENT_ID" but commit -m migrate .gitbutler/agents.toml .gitbutler/permissions.toml && git show HEAD:.gitbutler/agents.toml | grep -qF '[[agent]]' && ! git cat-file -e HEAD:.gitbutler/permissions.toml 2>/dev/null` → agents.toml committed via bootstrap escape hatch; permissions.toml gone
- `cd /Users/justinrich/Projects/agent-intel && AGENT_ID=$(but agent list --committed | grep -oE 'operator|electron-implementer|node-implementer' | head -1) && but agent register --pid $$ --as "$AGENT_ID" && before=$(git rev-parse HEAD) && echo y >> README.md && but commit -m proof README.md && test "$(git rev-parse HEAD)" != "$before"` → new commit lands via the registry path
- `cd /Users/justinrich/Projects/agent-intel && but agent unregister --pid $$ ; unset BUT_AGENT_HANDLE BUT_AUTHZ_ALLOW_ENV_HANDLE; echo z >> README.md && but commit -m x README.md 2>&1 | grep -q 'perm.denied'; rc=$?; git restore README.md; exit $rc` → perm.denied (README restored)
- `cd /Users/justinrich/Projects/agent-intel && DEL=$(git log --diff-filter=D --format=%H -- .gitbutler/permissions.toml | head -1) && git show "$DEL^:.gitbutler/permissions.toml" > /tmp/agentintel_pre.toml && while IFS= read -r id; do but agent list --committed | grep -qF "$id" || exit 1; done < <(grep '^id = ' /tmp/agentintel_pre.toml | sed -E 's/id = "(.*)"/\1/')` → every pre-migration id present in committed roster

## Agent Assignment

**Agent:** `rust-implementer` — rust-implementer owns the `but` CLI runtime and is the right hand to run the real `but agent migrate` against the live `agent-intel` repo and prove an end-to-end governed `but commit` via the registry path — this is field-execution + verification of the engine, its domain.
**Pairing:** none (single-surface Rust/docs/skill task). Honors `crates/AGENTS.md` + `BUT-SKILL-CONVENTIONS.md`.

## Evidence Gates

- `cd /Users/justinrich/Projects/agent-intel && AGENT_ID=$(but agent list --committed | grep -oE 'operator|electron-implementer|node-implementer' | head -1) && test -n "$AGENT_ID" || { echo 'ERROR: could not resolve an agent id from the committed roster (pre-migration legacy fallback not working)'; exit 1; } && BUT_AUTHZ_ALLOW_ENV_HANDLE=1 BUT_AGENT_HANDLE="$AGENT_ID" but agent migrate && BUT_AUTHZ_ALLOW_ENV_HANDLE=1 BUT_AGENT_HANDLE="$AGENT_ID" but commit -m migrate .gitbutler/agents.toml .gitbutler/permissions.toml && git show HEAD:.gitbutler/agents.toml | grep -qF '[[agent]]' && ! git cat-file -e HEAD:.gitbutler/permissions.toml 2>/dev/null` (agents.toml committed via bootstrap escape hatch; permissions.toml gone)
- `cd /Users/justinrich/Projects/agent-intel && AGENT_ID=$(but agent list --committed | grep -oE 'operator|electron-implementer|node-implementer' | head -1) && but agent register --pid $$ --as "$AGENT_ID" && before=$(git rev-parse HEAD) && echo y >> README.md && but commit -m proof README.md && test "$(git rev-parse HEAD)" != "$before"` (new commit lands via the registry path)
- `cd /Users/justinrich/Projects/agent-intel && but agent unregister --pid $$ ; unset BUT_AGENT_HANDLE BUT_AUTHZ_ALLOW_ENV_HANDLE; echo z >> README.md && but commit -m x README.md 2>&1 | grep -q 'perm.denied'; rc=$?; git restore README.md; exit $rc` (perm.denied (README restored))
- `cd /Users/justinrich/Projects/agent-intel && DEL=$(git log --diff-filter=D --format=%H -- .gitbutler/permissions.toml | head -1) && git show "$DEL^:.gitbutler/permissions.toml" > /tmp/agentintel_pre.toml && while IFS= read -r id; do but agent list --committed | grep -qF "$id" || exit 1; done < <(grep '^id = ' /tmp/agentintel_pre.toml | sed -E 's/id = "(.*)"/\1/')` (every pre-migration id present in committed roster)

## Review Criteria

- AC-1: PRIMARY — Migrate + bootstrap commit (env-handle escape hatch), then a registry-path commit lands — verified by `cd /Users/justinrich/Projects/agent-intel && AGENT_ID=$(but agent list --committed | grep -oE 'operator|electron-implementer|node-implementer' | head -1) && test -n "$AGENT_ID" || { echo 'ERROR: could not resolve an agent id from the committed roster (pre-migration legacy fallback not working)'; exit 1; } && BUT_AUTHZ_ALLOW_ENV_HANDLE=1 BUT_AGENT_HANDLE="$AGENT_ID" but agent migrate && BUT_AUTHZ_ALLOW_ENV_HANDLE=1 BUT_AGENT_HANDLE="$AGENT_ID" but commit -m 'chore: migrate governance to agents.toml' .gitbutler/agents.toml .gitbutler/permissions.toml && git show HEAD:.gitbutler/agents.toml | grep -q '\[\[agent\]\]' && ! git cat-file -e HEAD:.gitbutler/permissions.toml 2>/dev/null && but agent register --pid $$ --as "$AGENT_ID" && before=$(git rev-parse HEAD) && echo y >> README.md && but commit -m 'chore: governed registry-path commit' README.md && test "$(git rev-parse HEAD)" != "$before"`.
- AC-2: `but agent list --committed` shows the migrated roster — verified by `cd /Users/justinrich/Projects/agent-intel && but agent list --committed | grep -Eq 'operator|electron-implementer|node-implementer'`.
- AC-3: Unregistered process is denied (registry path enforced, not env) — verified by `cd /Users/justinrich/Projects/agent-intel && but agent unregister --pid $$ ; unset BUT_AGENT_HANDLE BUT_AUTHZ_ALLOW_ENV_HANDLE; echo z >> README.md && but commit -m 'should be denied' README.md 2>&1 | grep -q 'perm.denied'; rc=$?; git restore README.md; exit $rc`.
- AC-4: Roster/GovConfig equivalence: every pre-migration principal id survives as an agent id — verified by `cd /Users/justinrich/Projects/agent-intel && DEL=$(git log --diff-filter=D --format=%H -- .gitbutler/permissions.toml | head -1) && git show "$DEL^:.gitbutler/permissions.toml" > /tmp/agentintel_pre.toml && test -s /tmp/agentintel_pre.toml && while IFS= read -r id; do but agent list --committed | grep -qF "$id" || exit 1; done < <(grep '^id = ' /tmp/agentintel_pre.toml | sed -E 's/id = "(.*)"/\1/')`.
- Honors NEVER: Never hardcode `rust-implementer`/`rust-reviewer`/`orchestrator` — they are NOT in agent-intel's committed roster.

## Dependencies

- **Depends on:** IDENT-026
- **Blocks:** none
- **Capabilities:** CAP-AUTHZ-01, CAP-CONFIG-01

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "IDENT-028",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "agent_intel_permissions_repo": {
      "description": "the live agent-intel governed repo with a committed .gitbutler/permissions.toml + gates.toml and no agents.toml",
      "seed_method": "migration_fixture",
      "records": [
        "/Users/justinrich/Projects/agent-intel/.gitbutler/permissions.toml exists (committed [[principal]] roster)",
        "/Users/justinrich/Projects/agent-intel/.gitbutler/gates.toml exists",
        "no .gitbutler/agents.toml yet"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN the agent-intel repo with a committed permissions.toml and no agents.toml; AGENT_ID resolved from the committed roster WHEN running `but agent migrate` (bootstrap-wrapped with the env handle), committing the rename via the bootstrap escape hatch, then `but agent register --pid $$ --as $AGENT_ID` and a SECOND commit with no env flag THEN agents.toml is committed (permissions.toml removed) via the bootstrap escape hatch, and the second commit (registry path, no env handle) lands a new HEAD SHA",
      "test_tier": "e2e",
      "verification_service": "but CLI + agent-intel repo (bootstrap escape hatch -> registry path)",
      "verify": "cd /Users/justinrich/Projects/agent-intel && AGENT_ID=$(but agent list --committed | grep -oE 'operator|electron-implementer|node-implementer' | head -1) && test -n \"$AGENT_ID\" || { echo 'ERROR: could not resolve an agent id from the committed roster (pre-migration legacy fallback not working)'; exit 1; } && BUT_AUTHZ_ALLOW_ENV_HANDLE=1 BUT_AGENT_HANDLE=\"$AGENT_ID\" but agent migrate && BUT_AUTHZ_ALLOW_ENV_HANDLE=1 BUT_AGENT_HANDLE=\"$AGENT_ID\" but commit -m 'chore: migrate governance to agents.toml' .gitbutler/agents.toml .gitbutler/permissions.toml && git show HEAD:.gitbutler/agents.toml | grep -q '\\[\\[agent\\]\\]' && ! git cat-file -e HEAD:.gitbutler/permissions.toml 2>/dev/null && but agent register --pid $$ --as \"$AGENT_ID\" && before=$(git rev-parse HEAD) && echo y >> README.md && but commit -m 'chore: governed registry-path commit' README.md && test \"$(git rev-parse HEAD)\" != \"$before\"",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "e2e",
        "verification_service": "but CLI + agent-intel repo (bootstrap escape hatch -> registry path)",
        "start_ref": "agent_intel_permissions_repo",
        "must_observe": [
          "`git show HEAD:.gitbutler/agents.toml` contains `[[agent]]` (bootstrap commit)",
          "registry-path commit lands new HEAD SHA `!=` before (1 new commit, no env handle)"
        ],
        "must_not_observe": [
          "`permissions.toml` still committed (start state)",
          "registry-path commit denied `perm.denied` (0 lands)"
        ],
        "negative_control": {
          "would_fail_if": [
            "`but agent migrate` not run",
            "governed commit succeeds without registration (registry path bypassed)",
            "agents.toml not committed",
            "verify stubbed or not executed"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "agent_intel_permissions_repo",
            "action": {
              "actor": "cli_user",
              "steps": [
                "resolve AGENT_ID from `but agent list --committed`; guard non-empty",
                "bootstrap-wrapped `but agent migrate` (`BUT_AUTHZ_ALLOW_ENV_HANDLE=1 BUT_AGENT_HANDLE=$AGENT_ID`)",
                "bootstrap commit of the rename via the same escape hatch",
                "`but agent register --pid $$ --as $AGENT_ID`",
                "second `but commit` with no env handle; assert new SHA"
              ]
            },
            "end_state": {
              "must_observe": [
                "`git show HEAD:.gitbutler/agents.toml` contains `[[agent]]` (bootstrap commit)",
                "registry-path commit lands new HEAD SHA `!=` before (1 new commit, no env handle)"
              ],
              "must_not_observe": [
                "`permissions.toml` still committed (start state)",
                "registry-path commit denied `perm.denied` (0 lands)"
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
      "description": "GIVEN agent-intel post-migration WHEN running `but agent list --committed` THEN the migrated specialist roster is printed from committed agents.toml",
      "test_tier": "e2e",
      "verification_service": "but CLI + agent-intel repo",
      "verify": "cd /Users/justinrich/Projects/agent-intel && but agent list --committed | grep -Eq 'operator|electron-implementer|node-implementer'",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "e2e",
        "verification_service": "but CLI + agent-intel repo",
        "start_ref": "agent_intel_permissions_repo",
        "must_observe": [
          "`but agent list --committed` prints `operator` or `electron-implementer`",
          ">=1 migrated specialist listed"
        ],
        "must_not_observe": [
          "empty roster (0 agents)",
          "no agents.toml committed (none)"
        ],
        "negative_control": {
          "would_fail_if": [
            "agents.toml not committed",
            "list --committed returns empty (loader disconnected)",
            "grep stubbed or not executed"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "agent_intel_permissions_repo",
            "action": {
              "actor": "cli_user",
              "steps": [
                "but agent list --committed",
                "grep for a known specialist id"
              ]
            },
            "end_state": {
              "must_observe": [
                "`but agent list --committed` prints `operator` or `electron-implementer`",
                ">=1 migrated specialist listed"
              ],
              "must_not_observe": [
                "empty roster (0 agents)",
                "no agents.toml committed (none)"
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
      "description": "GIVEN agent-intel post-migration with no registration for this process WHEN attempting a governed `but commit` from an unregistered process (env handle flag unset) THEN the action is denied with `perm.denied` (Denial::unregistered)",
      "test_tier": "e2e",
      "verification_service": "but CLI + agent-intel repo (denial path)",
      "verify": "cd /Users/justinrich/Projects/agent-intel && but agent unregister --pid $$ ; unset BUT_AGENT_HANDLE BUT_AUTHZ_ALLOW_ENV_HANDLE; echo z >> README.md && but commit -m 'should be denied' README.md 2>&1 | grep -q 'perm.denied'; rc=$?; git restore README.md; exit $rc",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "e2e",
        "verification_service": "but CLI + agent-intel repo (denial path)",
        "start_ref": "agent_intel_permissions_repo",
        "must_observe": [
          "unregistered `but commit` denied with `perm.denied`",
          "stderr contains `perm.denied` (1 denial)"
        ],
        "must_not_observe": [
          "commit lands without registration (0 denial)",
          "env handle accepted (none gated)"
        ],
        "negative_control": {
          "would_fail_if": [
            "unregistered commit succeeds (registry not enforced)",
            "env handle accepted with flag unset",
            "denial check stubbed or not executed"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "agent_intel_permissions_repo",
            "action": {
              "actor": "cli_user",
              "steps": [
                "unregister this pid",
                "unset env handle + flag",
                "attempt a governed `but commit`",
                "assert perm.denied"
              ]
            },
            "end_state": {
              "must_observe": [
                "unregistered `but commit` denied with `perm.denied`",
                "stderr contains `perm.denied` (1 denial)"
              ],
              "must_not_observe": [
                "commit lands without registration (0 denial)",
                "env handle accepted (none gated)"
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
      "description": "GIVEN the pre-migration permissions.toml snapshotted at its last-present (pinned) ref WHEN comparing the committed agents.toml roster against the pre-migration permissions.toml ids THEN every `id =` from the pinned pre-migration permissions.toml is present in `but agent list --committed`",
      "test_tier": "e2e",
      "verification_service": "but CLI + agent-intel repo (roster/GovConfig equivalence vs pinned snapshot)",
      "verify": "cd /Users/justinrich/Projects/agent-intel && DEL=$(git log --diff-filter=D --format=%H -- .gitbutler/permissions.toml | head -1) && git show \"$DEL^:.gitbutler/permissions.toml\" > /tmp/agentintel_pre.toml && test -s /tmp/agentintel_pre.toml && while IFS= read -r id; do but agent list --committed | grep -qF \"$id\" || exit 1; done < <(grep '^id = ' /tmp/agentintel_pre.toml | sed -E 's/id = \"(.*)\"/\\1/')",
      "flow_ref": "UC-IDENT-05",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "e2e",
        "verification_service": "but CLI + agent-intel repo (roster/GovConfig equivalence vs pinned snapshot)",
        "start_ref": "agent_intel_permissions_repo",
        "must_observe": [
          "every `id = ` in `/tmp/agentintel_pre.toml` present in `but agent list --committed`",
          "pre-migration snapshot non-empty (>=1 id)"
        ],
        "must_not_observe": [
          "dropped roster entries (0 preserved)",
          "empty pre-snapshot (none)"
        ],
        "negative_control": {
          "would_fail_if": [
            "migration drops or alters roster entries",
            "permission fields changed",
            "diff stubbed or not executed"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "agent_intel_permissions_repo",
            "action": {
              "actor": "cli_user",
              "steps": [
                "find the commit that deleted permissions.toml (`--diff-filter=D`)",
                "`git show <del>^:.gitbutler/permissions.toml` into a temp snapshot",
                "assert every snapshot id resolves in `but agent list --committed`"
              ]
            },
            "end_state": {
              "must_observe": [
                "every `id = ` in `/tmp/agentintel_pre.toml` present in `but agent list --committed`",
                "pre-migration snapshot non-empty (>=1 id)"
              ],
              "must_not_observe": [
                "dropped roster entries (0 preserved)",
                "empty pre-snapshot (none)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "agent-intel has committed agents.toml + removed permissions.toml and a registered governed commit lands is true",
      "maps_to_ac": "AC-1",
      "verify": "cd /Users/justinrich/Projects/agent-intel && git show HEAD:.gitbutler/agents.toml | grep -q '\\[\\[agent\\]\\]' && ! git cat-file -e HEAD:.gitbutler/permissions.toml 2>/dev/null"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "`but agent list --committed` in agent-intel shows a real migrated id (operator|electron-implementer|node-implementer) is true",
      "maps_to_ac": "AC-2",
      "verify": "cd /Users/justinrich/Projects/agent-intel && but agent list --committed | grep -Eq 'operator|electron-implementer|node-implementer'"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "A governed `but commit` from an unregistered process is denied with perm.denied is true",
      "maps_to_ac": "AC-3",
      "verify": "cd /Users/justinrich/Projects/agent-intel && but agent unregister --pid $$ ; unset BUT_AGENT_HANDLE BUT_AUTHZ_ALLOW_ENV_HANDLE; echo z >> README.md && but commit -m x README.md 2>&1 | grep -q 'perm.denied'; rc=$?; git restore README.md; exit $rc"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "every pre-migration permissions.toml id is present in the committed agents.toml roster is true",
      "maps_to_ac": "AC-4",
      "verify": "cd /Users/justinrich/Projects/agent-intel && DEL=$(git log --diff-filter=D --format=%H -- .gitbutler/permissions.toml | head -1) && git show \"$DEL^:.gitbutler/permissions.toml\" > /tmp/agentintel_pre.toml && test -s /tmp/agentintel_pre.toml && while IFS= read -r id; do but agent list --committed | grep -qF \"$id\" || exit 1; done < <(grep '^id = ' /tmp/agentintel_pre.toml | sed -E 's/id = \"(.*)\"/\\1/')"
    }
  ]
}
-->
