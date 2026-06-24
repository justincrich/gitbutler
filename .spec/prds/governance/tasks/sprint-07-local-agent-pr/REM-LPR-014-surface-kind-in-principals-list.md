# REM-LPR-014: Surface `kind` in `governance_principals_list` — extend `GovernancePrincipalListEntry` with `kind: Option<String>` + SDK regen + desktop type alignment

> Status: Backlog
> Reviewer: rust-reviewer (DTO + SDK) + sveltekit-reviewer (desktop read path)
> Commit: (none yet)
> Updated: 2026-06-22T18:00:00Z
> PROPOSED-BY: rust-planner

## What this does

Extend the additive `kind` descriptor (`"agent"` | `"human"` | absent) from the committed `.gitbutler/permissions.toml` config into the `governance_principals_list` read path so the desktop renderer can display the agent badge without a second IPC round-trip. This is a **read-side remediation only**:

- Add `pub kind: Option<String>` to `GovernancePrincipalListEntry` (`crates/but-api/src/legacy/governance.rs:203-214`).
- Populate `kind` inside `governance_principals_list_with_repo` by reading the committed `permissions.toml` at the target ref — reusing the same committed-wire read that `principal_kind_read_with_repo` already performs.
- Regenerate the TypeScript SDK via `pnpm build:sdk && pnpm format`.
- Align the desktop's local `PrincipalListEntry` / `PrincipalsListEntry` types so the value flows from the backend to the row badge.

The LPR-014 editor UI and the LPR-013 write path (`principal_kind_update`) are **already complete**; this remediation closes the missing read integration that makes the agent badge invisible after save+reload (`.spec/reviews/red-hat-20260622-173510.md`, finding **M2 — HIGH**).

## Why

The red-hat review found that the `kind` write path works (LPR-013's `principal_kind_update` is Tauri-registered and writes `kind` to `.gitbutler/permissions.toml`), but the read path is broken. The renderer DTO `GovernancePrincipalListEntry` has no `kind` field, so the generated SDK has no `kind`, the desktop's local `PrincipalListEntry` has no `kind`, and `GovernanceSettings.svelte` does not merge `principal_kind_read` into the list. Consequence: `{#if principal.kind === "agent"}` in `PrincipalsList.svelte` is unreachable in production. The badge only appears in CT specs that mock the prop.

This task picks **Option A**: fold `kind` into the existing list DTO. Single round-trip, matches how grants are already folded into the list, keeps the desktop read path unchanged beyond type alignment.

## How to verify

```bash
cargo test -p but-api governance_principals_list
pnpm build:sdk && pnpm format
pnpm -F @gitbutler/desktop check
```

## Scope

- `crates/but-api/src/legacy/governance.rs` (MODIFY — extend `GovernancePrincipalListEntry`; populate `kind` in `governance_principals_list_with_repo` from the committed wire)
- `packages/but-sdk/src/generated/**` (REGENERATE ONLY via `pnpm build:sdk && pnpm format`)
- `apps/desktop/src/lib/governance/governanceService.ts` (MODIFY — add `kind` to the local `PrincipalListEntry` type)
- `apps/desktop/src/components/governance/PrincipalsList.svelte` (MODIFY/VERIFY — ensure `PrincipalsListEntry` carries `kind` so the existing row badge can consume it)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: REM-LPR-014 — Surface `kind` in `governance_principals_list`
================================================================================

TASK_TYPE:   FEATURE (DTO extension + SDK regen + desktop type alignment)
STATUS:      Backlog
PRIORITY:    P1 (remediates HIGH red-hat finding M2)
EFFORT:      S  (45 min)
AGENT:       rust-implementer (DTO + SDK regen) + sveltekit-implementer (desktop read path)
REVIEWER:    rust-reviewer (DTO + SDK) + sveltekit-reviewer (desktop read path)
PROPOSED-BY: rust-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-LPR-04
CAPABILITIES:CAP-AUTHZ-01
DEPENDS_ON:  none (LPR-013 is complete; LPR-014 editor + write path are complete)
BLOCKS:      none

RUNTIME_COMMANDS:
  test:  cargo test -p but-api governance_principals_list
  check: cargo check -p but-api --all-targets && pnpm -F @gitbutler/desktop check
  lint:  cargo clippy -p but-api -p gitbutler-tauri --all-targets && pnpm lint

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
`GovernancePrincipalListEntry` carries `kind: Option<String>` populated from the committed target-ref `permissions.toml`. After `pnpm build:sdk && pnpm format`, the generated SDK type exposes `kind?: string | null`. The desktop's local types include `kind`, so a principal whose committed `kind` is `"agent"` renders the agent badge in `PrincipalsList.svelte` after a fresh list load. No enforcement path reads `kind`; `GovConfig.principals` remains `BTreeMap<PrincipalId, AuthoritySet>`.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST extend `GovernancePrincipalListEntry` with `pub kind: Option<String>` as an additive field. Existing consumers that deserialize without `kind` continue to work.
- [MUST] MUST populate `kind` in `governance_principals_list_with_repo` by reading the committed `permissions.toml` at the target ref. Reuse the same committed-wire read that `principal_kind_read_with_repo` performs. Consolidate by extracting a helper if useful, but do NOT duplicate.
- [MUST] MUST run `pnpm build:sdk && pnpm format` to regenerate the SDK. NEVER hand-edit `packages/but-sdk/src/generated`.
- [MUST] MUST verify the desktop's `PrincipalListEntry` TS type picks up the new field via the regenerated SDK. `PrincipalsList.svelte` must be able to consume `kind` on the principal row.
- [MUST] MUST keep `kind` as descriptive metadata only. `kind` does NOT enter `GovConfig.principals`; no gate reads it; no enforcement decision uses it.
- [MUST NOT] change any enforcement logic, the merge gate, review_requirement, or the safe-seam tests. `kind` is informational.
- [MUST NOT] change the write path (`principal_kind_update`) or its Tauri registration.
- [MUST NOT] introduce a parallel read path. Option A (extend the list DTO) is chosen; do not call `principal_kind_read` from the desktop and merge separately.
- [NEVER] NEVER add new `gitbutler-*` usage.
- [NEVER] NEVER hand-edit generated SDK files.
- [STRICTLY] STRICTLY follow existing `but-api` patterns for `#[but_api(napi)]` DTOs (`Serialize` + `schemars::JsonSchema`, `#[serde(rename_all = "camelCase")]`).

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: After setting `kind = "agent"` via the editor and reloading project settings, the agent badge in `PrincipalsList.svelte:184` renders in the live app (currently only renders in CT specs that mock the prop).
- [ ] AC-2: `GovernancePrincipalListEntry.kind: Option<String>` exists and is populated from committed `permissions.toml`.
- [ ] AC-3: SDK regenerated; `packages/but-sdk/src/generated/index.d.ts` carries `kind?: string | null` on the principal list entry type.
- [ ] AC-4: Existing `principal_kind_read` / `principal_kind_update` tests still pass; existing `governance_principals_list` tests still pass; no enforcement change.

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: a principal with committed `kind = "agent"` shows the agent badge in the row after a fresh list load
  GIVEN: a repository whose committed `.gitbutler/permissions.toml` declares `kind = "agent"` for principal `"agent:codex"`; the project settings page is opened so `governance_principals_list` is invoked
  WHEN:  `governance_principals_list` returns and `PrincipalsList.svelte` renders
  THEN:  the row for `"agent:codex"` contains a Badge with text containing `"agent"`; a principal with `kind = "human"` or no `kind` has no agent badge
  TEST_TIER: integration   VERIFICATION_SERVICE: but-api unit/integration + desktop CT
  VERIFY: cargo test -p but-api governance_principals_list && pnpm test:ct:desktop -- PrincipalsListAgentBadge

AC-2: a principal with no `kind` / `kind = "human"` shows no agent badge
  GIVEN: `governance_principals_list` returns a principal whose committed `kind` is absent or `"human"`
  WHEN:  the row renders
  THEN:  no agent badge appears for that principal
  TEST_TIER: unit   VERIFICATION_SERVICE: but-api test
  VERIFY: cargo test -p but-api governance_principals_list

AC-3: SDK regenerated with the new optional `kind` field
  GIVEN: the Rust DTO has been extended with `pub kind: Option<String>`
  WHEN:  `pnpm build:sdk && pnpm format` completes
  THEN:  `packages/but-sdk/src/generated/index.d.ts` contains `kind?: string | null` on the principal list entry type; the desktop type-checks clean
  TEST_TIER: build   VERIFICATION_SERVICE: SDK generator + TypeScript compiler
  VERIFY: pnpm build:sdk && pnpm format && pnpm -F @gitbutler/desktop check && rg "kind\?:" packages/but-sdk/src/generated/index.d.ts

AC-4: no enforcement change — `GovConfig.principals` is unchanged; no gate references `kind`
  GIVEN: the `kind` field is added to `GovernancePrincipalListEntry`
  WHEN:  the safe-seam / enforcement honesty greps and tests run
  THEN:  `GovConfig.principals` remains `BTreeMap<PrincipalId, AuthoritySet>`; `merge_gate.rs`, `review_requirement.rs`, and `config.rs` enforcement paths do not reference `kind`; the LPR-009 safe-seam invariant tests still pass
  TEST_TIER: build-gate   VERIFICATION_SERVICE: cargo test + grep
  VERIFY: cargo test -p but-authz invariant_build_gates && cargo test -p but-api safe_seam && ! rg "kind" crates/but-api/src/legacy/merge_gate.rs && ! rg "kind" crates/but-api/src/legacy/review_requirement.rs

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps_to_ac)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, AC-2): the list DTO carries `kind` when the committed `permissions.toml` declares it, and is `None`/`undefined` when not declared. VERIFY: cargo test -p but-api governance_principals_list
- TC-2 (-> AC-3): a snapshot/diff test asserts the regenerated SDK type carries `kind?: string | null` on `GovernancePrincipalListEntry`. VERIFY: pnpm build:sdk && pnpm format && rg "kind\?:" packages/but-sdk/src/generated/index.d.ts && git diff -- packages/but-sdk/src/generated/index.d.ts | grep -E "^\+\s*kind\?:"
- TC-3 (-> AC-4): existing LPR-013 tests still pass and no enforcement path references `kind`. VERIFY: cargo test -p but-api --test mgmt_ipc_003_governance_commands && cargo test -p gitbutler-tauri --test lpr_review_reads && cargo test -p but-authz invariant_build_gates

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-api/src/legacy/governance.rs (MODIFY — extend GovernancePrincipalListEntry with pub kind: Option<String>; populate kind in governance_principals_list_with_repo from committed PermissionsWire)
  - packages/but-sdk/src/generated/** (REGENERATE ONLY via pnpm build:sdk && pnpm format)
  - apps/desktop/src/lib/governance/governanceService.ts (MODIFY — add kind?: string | null to local PrincipalListEntry type)
  - apps/desktop/src/components/governance/PrincipalsList.svelte (VERIFY/MODIFY — ensure PrincipalsListEntry carries kind; do NOT change rendering logic if badge code from LPR-014 is already present)
writeProhibited:
  - crates/but-authz/src/config.rs (PrincipalWire already has kind from LPR-005; do NOT touch)
  - crates/but-api/src/legacy/merge_gate.rs (CONSUME-only; safe-seam invariant)
  - crates/but-api/src/legacy/review_requirement.rs (CONSUME-only; safe-seam invariant)
  - crates/but-api/src/legacy/config_mutate.rs (CONSUME-only)
  - crates/but-api/tests/safe_seam.rs and safe-seam invariant tests (do NOT weaken)
  - crates/gitbutler-tauri/src/governance.rs (the principal_kind_update write path is complete; do NOT change)
  - the LPR-013 registration or write path
  - any file not in writeAllowed

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: none (LPR-013 principal_kind_read/principal_kind_update complete; LPR-014 editor + write path complete)
Blocks:     none
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "REM-LPR-014",
  "proposed_by": "rust-planner",
  "red_hat_finding": ".spec/reviews/red-hat-20260622-173510.md M2 (HIGH) — LPR-014 agent badge invisible in production because governance_principals_list DTO has no kind field",
  "implementation_option": "Option A (extend the list DTO); Option B (UI-side merge via principal_kind_read) rejected",
  "option_rationale": "Option A requires a single round-trip, reuses the existing committed permissions.toml load inside governance_principals_list_with_repo, and keeps the desktop read path minimal.",
  "verification_policy": { "requires_tests": true, "requires_red_evidence": true, "requires_seeded_evidence": true },
  "fixtures": {
    "kind_governance_base": {
      "description": "A real governed repo via but_testsupport with committed .gitbutler/permissions.toml containing principals with and without kind.",
      "seed_method": "public_api",
      "records": [
        "principal agent:codex with kind='agent' and grants ['reviews:write'];",
        "principal human:alice with kind='human' and grants ['contents:read'];",
        "principal ci:runner with NO kind and grants ['contents:read'];"
      ]
    }
  },
  "requirements": [
    { "id": "AC-1", "type": "acceptance_criterion", "primary": true, "description": "GIVEN a repository with committed kind='agent' for agent:codex WHEN governance_principals_list is invoked and PrincipalsList.svelte renders THEN the agent badge appears in agent:codex's row and no agent badge appears for human/absent-kind principals", "verify": "cargo test -p but-api governance_principals_list && pnpm test:ct:desktop -- PrincipalsListAgentBadge", "scenario": { "tier": "visible", "test_tier": "integration", "verification_service": "but-api test + desktop CT", "negative_control": { "would_fail_if": ["GovernancePrincipalListEntry has no kind field", "kind is None for agent:codex despite committed kind='agent'", "human:alice or ci:runner show an agent badge"] }, "evidence": { "artifact_type": "test_output", "required_capture": true }, "cases": [ { "start_ref": "kind_governance_base", "action": { "actor": "user", "steps": ["invoke governance_principals_list", "render PrincipalsList.svelte"] }, "end_state": { "must_observe": ["agent:codex row contains an agent Badge", "human:alice row has 0 agent badges", "ci:runner row has 0 agent badges"], "must_not_observe": ["0 agent badges in agent:codex row", "agent badge in human:alice's row", "agent badge in ci:runner's row"] } } ] } },
    { "id": "AC-2", "type": "acceptance_criterion", "primary": false, "description": "GIVEN governance_principals_list returns a principal with no kind or kind='human' WHEN the row renders THEN no agent badge appears", "verify": "cargo test -p but-api governance_principals_list", "scenario": { "tier": "visible", "test_tier": "unit", "verification_service": "but-api test", "negative_control": { "would_fail_if": ["absent kind renders an agent badge", "kind='human' renders an agent badge"] }, "evidence": { "artifact_type": "test_output", "required_capture": true }, "cases": [ { "start_ref": "kind_governance_base", "action": { "actor": "ci", "steps": ["invoke governance_principals_list", "assert entries for human:alice and ci:runner"] }, "end_state": { "must_observe": ["human:alice kind is 'human' or None", "ci:runner kind is None"], "must_not_observe": ["agent badge for human:alice", "agent badge for ci:runner"] } } ] } },
    { "id": "AC-3", "type": "acceptance_criterion", "primary": false, "description": "GIVEN the Rust DTO is extended with kind WHEN pnpm build:sdk && pnpm format runs THEN packages/but-sdk/src/generated/index.d.ts carries kind?: string | null and the desktop typechecks clean", "verify": "pnpm build:sdk && pnpm format && rg \"kind\\?:\" packages/but-sdk/src/generated/index.d.ts && pnpm -F @gitbutler/desktop check", "scenario": { "tier": "build", "test_tier": "build", "verification_service": "SDK generator + tsc", "negative_control": { "would_fail_if": ["generated index.d.ts has no kind field", "desktop typecheck fails because local type does not accept kind"] }, "evidence": { "artifact_type": "test_output", "required_capture": true }, "cases": [ { "start_ref": "kind_governance_base", "action": { "actor": "ci", "steps": ["extend GovernancePrincipalListEntry in Rust", "run pnpm build:sdk && pnpm format", "run pnpm -F @gitbutler/desktop check"] }, "end_state": { "must_observe": ["generated SDK type includes kind?: string | null", "desktop typecheck passes"], "must_not_observe": ["hand-edited generated files", "type errors on PrincipalListEntry or PrincipalsListEntry"] } } ] } },
    { "id": "AC-4", "type": "acceptance_criterion", "primary": false, "description": "GIVEN the kind field is added to the list DTO WHEN safe-seam and enforcement tests run THEN GovConfig.principals remains unchanged, no gate references kind, and existing LPR-013 / governance_principals_list tests still pass", "verify": "cargo test -p but-authz invariant_build_gates && cargo test -p but-api safe_seam && cargo test -p but-api --test mgmt_ipc_003_governance_commands && cargo test -p gitbutler-tauri --test lpr_review_reads", "scenario": { "tier": "build-gate", "test_tier": "build-gate", "verification_service": "cargo test + grep", "negative_control": { "would_fail_if": ["kind enters merge_gate.rs or review_requirement.rs", "kind enters GovConfig.principals", "existing governance or principal_kind tests fail"] }, "evidence": { "artifact_type": "test_output", "required_capture": true }, "cases": [ { "start_ref": "kind_governance_base", "action": { "actor": "ci", "steps": ["run safe-seam invariant tests", "run existing governance_principals_list and principal_kind tests", "grep gate paths for kind references"] }, "end_state": { "must_observe": ["safe-seam tests pass", "existing tests pass", "no kind reference in merge_gate.rs / review_requirement.rs / GovConfig.principals"], "must_not_observe": ["kind in enforcement paths", "regression"] } } ] } },
    { "id": "TC-1", "type": "test_criterion", "description": "list DTO carries kind when committed config declares it; absent when not", "verify": "cargo test -p but-api governance_principals_list", "maps_to_ac": "AC-1,AC-2" },
    { "id": "TC-2", "type": "test_criterion", "description": "generated SDK type carries kind?: string | null on GovernancePrincipalListEntry", "verify": "pnpm build:sdk && pnpm format && rg \"kind\\?:\" packages/but-sdk/src/generated/index.d.ts", "maps_to_ac": "AC-3" },
    { "id": "TC-3", "type": "test_criterion", "description": "existing LPR-013 governance tests pass and no enforcement path references kind", "verify": "cargo test -p but-api --test mgmt_ipc_003_governance_commands && cargo test -p gitbutler-tauri --test lpr_review_reads && cargo test -p but-authz invariant_build_gates", "maps_to_ac": "AC-4" }
  ]
}
-->
