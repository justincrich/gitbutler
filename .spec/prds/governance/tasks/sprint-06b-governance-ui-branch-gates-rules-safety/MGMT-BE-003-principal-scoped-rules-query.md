# MGMT-BE-003: `principalId`-scoped rules query (backend for the Rules tab)

## What this does

Add a principalId-scoped rules query at the but-api boundary that filters existing but-rules WorkspaceRules by the rule's agent association (WorkspaceRule::session_id()), is backward compatible (None == the existing list_workspace_rules), and exposes its Tauri command + regenerated SDK delta — honestly grounded in the verified rules data model (no invented principal column).

## Why

Sprint 06b · PRD UC-MGMT-05 · capability CAP-AUTHZ-01. Proven against the real rules store: (1) list_workspace_rules_scoped(ctx, None) returns results identical to the existing list_workspace_rules(ctx) for a seeded multi-rule store, including the in-workspace stack filter; (2) list_workspace_r

## How to verify

PRIMARY **AC-1** — `cargo test -p but-api list_workspace_rules_scoped_none_equals_existing`: Unset principalId is the existing list_workspace_rules BY CONSTRUCTION — list_workspace_rules_scoped(ctx, None) delegates to list_workspace_rules(ctx) so the in-workspace stack filter cannot drift. Full gate set in the spec below.

## Scope

  - crates/but-api/src/legacy/rules.rs (MODIFY — add list_workspace_rules_scoped(ctx, principal_id: Option<&str>) beside list_workspace_rules; the None path delegates to / preserves the existing in-workspace filter, the Some path layers a session_id() filter on top)
  - crates/but-api/tests/rules_scoped.rs (NEW — the PRIMARY but-api proofs AC-1..AC-3 against a real but-db rules store via but_testsupport)
  - the desktop Tauri command file under crates/ that registers the rules-list #[tauri::command] (MODIFY — forward the optional principalId to list_workspace_rules_scoped; follow the existing convention)
  - packages/but-sdk/src/generated/** (REGENERATE ONLY via `pnpm build:sdk && pnpm format` — NEVER hand-edit)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: MGMT-BE-003 — `principalId`-scoped rules query (backend for the Rules tab)
================================================================================

TASK_TYPE:   FEATURE
STATUS:      Backlog
PRIORITY:    P1
EFFORT:      M  (120 min)
AGENT:       implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:      ./SPRINT.md
PRD_REFS:    UC-MGMT-05
CAPABILITIES:CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-api list_workspace_rules_scoped_none_equals_existing
  check: cargo check -p but-rules --all-targets
  lint:  cargo clippy -p but-api --all-targets

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Proven against the real rules store: (1) list_workspace_rules_scoped(ctx, None) returns results identical to the existing list_workspace_rules(ctx) for a seeded multi-rule store, including the in-workspace stack filter; (2) list_workspace_rules_scoped(ctx, Some("agent-A")) returns ONLY the rules whose session association is agent-A, excluding agent-B's rules and the unscoped/global rules; (3) a principal with no associated rules returns an empty list; (4) `pnpm build:sdk && pnpm format` regenerates the SDK with the optional principalId argument; cargo test -p but-api / -p but-rules green; clippy clean.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST make the None path EQUAL to list_workspace_rules BY CONSTRUCTION: list_workspace_rules_scoped(ctx, None) DELEGATES to crate::legacy::rules::list_workspace_rules(ctx) (the None branch literally calls the existing function) so the in-workspace stack filter (rules.rs:44-66) cannot drift; AC-1 asserts scoped(None) output EQUALS list_workspace_rules(ctx) on the same ctx. The Some path layers the session_id() filter on top of that same delegated result. Do NOT fork a parallel rules loader for the None path.
- [MUST] MUST be backward compatible: when principal_id is None, list_workspace_rules_scoped MUST return results BYTE-IDENTICAL to the existing crate::legacy::rules::list_workspace_rules(ctx) — INCLUDING its existing in-workspace stack filter (rules.rs:44-66 filters out Codegen rules referencing out-of-workspace stacks). The None path delegates to / preserves that exact filter; it does not bypass it.
- [MUST] MUST scope by the rule's agent association when principal_id is Some(id): return only rules where rule.session_id() == Some(id) (the principalId IS the agent/session handle in the rules domain — the same identity axis as BUT_AGENT_HANDLE). Rules with no session association (session_id() == None) are NOT returned for a specific principal scope (they are workspace-global, surfaced only by the unscoped query).
- [MUST] MUST site the scoped query at the but-api boundary (crates/but-api/src/legacy/rules.rs, beside list_workspace_rules) so the Tauri command and MGMT-UI-010's RulesList(principalId) consume the SAME function — never push the principal filter into the renderer or duplicate the rules query in the UI layer.
- [MUST] MUST surface the SDK delta as part of done. After the Rust API change, `pnpm build:sdk && pnpm format` regenerates packages/but-sdk/src/generated with the optional principalId argument on the rules-list command; the generated files are NEVER hand-edited.
- [MUST] MUST name the data-model gap in the completion report: rules carry no first-class principal column; the per-principal scope is implemented over the existing ClaudeCode session-id association. If the PRD's per-principal model later needs a true principal column, that is a NEW schema task (out of scope here) — state this explicitly rather than silently introducing a migration.
- [MUST] MUST identity/scope-check the renderer-supplied principalId at the boundary (SEC-5). principal_id is UNTRUSTED renderer input. v1 decision: the rules-list Tauri command resolves the caller identity and applies a self/admin-read scope — a non-admin caller may scope only to its OWN principal (requesting another principal yields Ok(empty), never another principal's rules); an admin may scope to any principal. The command MUST NOT forward principalId blindly to list_workspace_rules_scoped without that check (AC-5 proves it).
- [NEVER] NEVER invent a principalId/agent column on WorkspaceRule or add a but-db migration to add one (out of scope; the agent association is the existing ClaudeCode session id).
- [NEVER] NEVER change the behavior of the unset/None path — it must equal the existing list_workspace_rules including its in-workspace stack filter (regression-proof in AC-1).
- [NEVER] NEVER push the principal-scoping filter into the renderer / RulesList — the filter lives at the but-api boundary so the prop is a thin pass-through.
- [NEVER] NEVER return rules for ALL principals when a specific principalId is requested (the scoped query must actually narrow — AC-2's negative control catches a no-op filter).
- [NEVER] NEVER hand-edit packages/but-sdk/src/generated — regenerate via pnpm build:sdk only.
- [NEVER] NEVER add new gitbutler-* usage.
- [NEVER] NEVER forward the renderer-supplied principalId to the scoped query without a caller identity/scope check — a blind pass-through lets agent-A enumerate agent-B's rules by session id (SEC-5 recon leak; AC-5 catches it).
- [STRICTLY] STRICTLY treat the existing crate::legacy::rules::list_workspace_rules + its in-workspace stack filter as a CONSUMED seam — the scoped query reuses/preserves it for the None path and layers the optional principal filter on top; do not fork a parallel rules loader.
- [STRICTLY] STRICTLY keep the principal identity axis consistent with the rest of governance — the principalId argument is the agent handle (the same identity BUT_AGENT_HANDLE carries), matched against the rule's ClaudeCode session-id association.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: Unset principalId is the existing list_workspace_rules BY CONSTRUCTION — list_workspace_rules_scoped(ctx, None) delegates to list_workspace_rules(ctx) so the in-workspace stack filter cannot drift
- [ ] AC-2: principalId=Some narrows to only that principal's rules (the agent-association scope actually filters)
- [ ] AC-3: A principal with no associated rules returns an empty list (clean empty-state, not an error)
- [ ] AC-4: The scoped rules query is exposed via the but-api -> Tauri -> but-sdk governed path with the optional principalId argument
- [ ] AC-5: Cross-principal scoping is honest: a caller cannot enumerate another principal's rules via a renderer-supplied principalId (v1 self/admin-read scope)
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: Unset principalId is the existing list_workspace_rules BY CONSTRUCTION — list_workspace_rules_scoped(ctx, None) delegates to list_workspace_rules(ctx) so the in-workspace stack filter cannot drift
  GIVEN: rules_principal_scoped_store: a real rules store with rule_a (session agent-A), rule_b (session agent-B), rule_global (no session); the baseline list_workspace_rules(&ctx) captured
  WHEN:  list_workspace_rules_scoped(&ctx, None) runs
  THEN:  list_workspace_rules_scoped(&ctx, None) DELEGATES to list_workspace_rules(&ctx) (the None branch calls the existing function directly — the filter is not re-implemented), so the returned rules are EQUAL (same rule-id set, same order/contents) to list_workspace_rules(&ctx) on the SAME ctx — all three rules present — and the in-workspace stack filter is preserved by construction (it is the SAME code path)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api list_workspace_rules_scoped + real but-rules list_rules + real but-db DbHandle via but_testsupport
  VERIFY: cargo test -p but-api list_workspace_rules_scoped_none_equals_existing

AC-2: principalId=Some narrows to only that principal's rules (the agent-association scope actually filters)
  GIVEN: rules_principal_scoped_store: rule_a (session agent-A), rule_b (session agent-B), rule_global (no session)
  WHEN:  list_workspace_rules_scoped(&ctx, Some("agent-A")) runs
  THEN:  the result contains ONLY rule_a (session_id()==Some("agent-A")); rule_b (agent-B) is ABSENT and rule_global (no session) is ABSENT — the scope filters on the rule's ClaudeCode session association
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api list_workspace_rules_scoped principal filter + real but-rules WorkspaceRule::session_id + real but-db via but_testsupport
  VERIFY: cargo test -p but-api list_workspace_rules_scoped_some_narrows_to_principal

AC-3: A principal with no associated rules returns an empty list (clean empty-state, not an error)
  GIVEN: rules_principal_scoped_store: no rule has a session association of "agent-Z"
  WHEN:  list_workspace_rules_scoped(&ctx, Some("agent-Z")) runs
  THEN:  the call returns Ok(vec![]) — an empty rule list (the Rules tab's empty/placeholder state), NOT an error and NOT all rules
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api list_workspace_rules_scoped + real but-db via but_testsupport
  VERIFY: cargo test -p but-api list_workspace_rules_scoped_unknown_principal_empty

AC-4: The scoped rules query is exposed via the but-api -> Tauri -> but-sdk governed path with the optional principalId argument
  GIVEN: the list_workspace_rules_scoped but-api function exists with an optional principal_id argument
  WHEN:  the Tauri command for listing rules is extended to forward the optional principalId, and `pnpm build:sdk && pnpm format` runs
  THEN:  packages/but-sdk/src/generated regenerates with the optional principalId argument on the rules-list command/type; the generated TS type-checks; the principalId is a thin pass-through (the filter lives in the but-api fn, not the renderer)
  TEST_TIER: integration   VERIFICATION_SERVICE: real SDK generation pipeline (pnpm build:sdk) + real TypeScript type-check against the regenerated SDK
  VERIFY: pnpm build:sdk && pnpm format && grep -rq "principalId" packages/but-sdk/src/generated && pnpm -F @gitbutler/desktop check

AC-5: Cross-principal scoping is honest: a caller cannot enumerate another principal's rules via a renderer-supplied principalId (v1 self/admin-read scope)
  GIVEN: rules_principal_scoped_store: rule_a (session agent-A), rule_b (session agent-B); the renderer-supplied principal_id is untrusted input. v1 decision: a non-admin caller may only scope to its OWN principal; requesting another principal yields Ok(empty) (no rule disclosure), while an admin caller may scope to any principal
  WHEN:  the rules-list Tauri command path runs as BUT_AGENT_HANDLE=agent-A with a renderer-supplied principalId=Some("agent-B") (a session-id reconnaissance attempt), and separately as admin with Some("agent-B")
  THEN:  as agent-A requesting agent-B: the result is Ok(empty) — agent-B's rule_b is NOT disclosed to agent-A (the command applies the caller's identity/scope check before forwarding principalId); as admin requesting agent-B: the result contains exactly rule_b (admins may scope to any principal). principalId is NEVER forwarded blindly without an identity/scope check.
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-api scoped rules query + the caller-identity scope check (BUT_AGENT_HANDLE resolution) + real but-db via but_testsupport
  VERIFY: cargo test -p but-api list_workspace_rules_scoped_cross_principal_not_disclosed

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): list_workspace_rules_scoped(&ctx, None) returns the same rule-id set as list_workspace_rules(&ctx), including rule_global and the in-workspace stack filter behavior
    VERIFY: cargo test -p but-api list_workspace_rules_scoped_none_equals_existing
- TC-2 (-> AC-2): list_workspace_rules_scoped(&ctx, Some("agent-A")) returns exactly rule_a; rule_b and rule_global are absent
    VERIFY: cargo test -p but-api list_workspace_rules_scoped_some_narrows_to_principal
- TC-3 (-> AC-2): list_workspace_rules_scoped(&ctx, Some("agent-B")) returns exactly rule_b (the filter matches the rule's session association, not a fixed value)
    VERIFY: cargo test -p but-api list_workspace_rules_scoped_some_narrows_to_principal
- TC-4 (-> AC-3): list_workspace_rules_scoped(&ctx, Some("agent-Z")) (no associated rules) returns Ok(empty), not Err and not all rules
    VERIFY: cargo test -p but-api list_workspace_rules_scoped_unknown_principal_empty
- TC-5 (-> AC-4): `pnpm build:sdk && pnpm format` regenerates packages/but-sdk/src/generated with the optional principalId argument and the desktop TS type-check passes
    VERIFY: pnpm build:sdk && pnpm format && grep -rq "principalId" packages/but-sdk/src/generated && pnpm -F @gitbutler/desktop check
- TC-6 (-> AC-5): agent-A requesting principalId=Some("agent-B") yields Ok(empty) (no cross-principal disclosure); admin requesting Some("agent-B") yields exactly rule_b (renderer-supplied principalId is identity/scope-checked, never forwarded blindly)
    VERIFY: cargo test -p but-api list_workspace_rules_scoped_cross_principal_not_disclosed

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides:
  - list_workspace_rules_scoped(ctx, principal_id: Option<&str>) at the but-api boundary — when principal_id is set, returns only the rules associated with that principal (filtered on the rule's agent-association field, WorkspaceRule::session_id()); when None, returns identical results to the existing list_workspace_rules(ctx) (backward compatible)
  - a Tauri command delta + SDK regeneration exposing the optional principalId argument so MGMT-UI-010's RulesList(principalId) prop consumes it
consumes:
  - but_rules::{WorkspaceRule, list_rules, WorkspaceRule::session_id} (the existing rules data model + the only agent-association accessor)
  - crate::legacy::rules::list_workspace_rules (the existing unscoped but-api query whose behavior the None path must preserve byte-identically, INCLUDING its existing in-workspace stack filter)
  - but_ctx::Context + but_db::DbHandle (the real rules store) via but_testsupport for the integration scenario
boundary_contracts:
  - CAP-AUTHZ-01: the scoped rules read is exposed via the same but-api -> Tauri -> but-sdk governed path as the rest of the MGMT surface (no direct DB read from the renderer). principal_id is UNTRUSTED renderer input: the rules-list Tauri command resolves the caller identity and enforces a v1 self/admin-read scope BEFORE forwarding it — a non-admin caller may only scope to its own principal (another principal yields Ok(empty)); an admin may scope to any. The query never widens authority and never discloses another principal's rules to a caller not entitled to view them (AC-5 is the tested proof, not prose).

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-api/src/legacy/rules.rs (MODIFY — add list_workspace_rules_scoped(ctx, principal_id: Option<&str>) beside list_workspace_rules; the None path delegates to / preserves the existing in-workspace filter, the Some path layers a session_id() filter on top)
  - crates/but-api/tests/rules_scoped.rs (NEW — the PRIMARY but-api proofs AC-1..AC-3 against a real but-db rules store via but_testsupport)
  - the desktop Tauri command file under crates/ that registers the rules-list #[tauri::command] (MODIFY — forward the optional principalId to list_workspace_rules_scoped; follow the existing convention)
  - packages/but-sdk/src/generated/** (REGENERATE ONLY via `pnpm build:sdk && pnpm format` — NEVER hand-edit)
writeProhibited:
  - crates/but-rules/src/lib.rs and crates/but-rules/src/db.rs — do NOT add a principalId/agent column or change the WorkspaceRule schema; the agent association is the existing ClaudeCode session id (read-only consumption of session_id())
  - crates/but-db/** — do NOT add a migration to introduce a principal column on workspace_rules (out of scope; would be a NEW schema task)
  - the existing list_workspace_rules in-workspace stack filter semantics — the None path must PRESERVE it byte-identically (AC-1), not replace or bypass it
  - apps/desktop/src/components/rules/** and apps/desktop/src/lib/rules/** — the renderer-side RulesList(principalId) prop is MGMT-UI-010's scope, not this backend task; do not push the filter into the renderer
  - GATES-003's writeProhibited governance-config surfaces (this task touches rules, not governance config — do not modify but-authz config or gates/permissions writers)
  - any gitbutler-* crate (no new gitbutler-* usage)
  - Any file not in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-api/src/legacy/rules.rs [41-69] — [PRIMARY PATTERN] the existing list_workspace_rules(ctx) — its #[but_api] attribute, the in-workspace stack filter it already applies (filter_map on rule.session_id()+target_stack_id()), and the shape to mirror. The new list_workspace_rules_scoped is sited HERE beside it and the None path must preserve this exact filter.
2. crates/but-rules/src/lib.rs [9-72, 88-101] — [DATA-MODEL GROUNDING — verify before coding] WorkspaceRule has NO principal/agent field (fields: id, created_at, enabled, trigger, filters, action); the ONLY agent association is Filter::ClaudeCodeSessionId(String) surfaced via WorkspaceRule::session_id() -> Option<String>. The principalId scope MUST filter on session_id() — do NOT invent a principal column.
3. crates/but-rules/src/db.rs [1-43] — the but-db<->but-rules conversion + workspace_rules(ctx)/list_rules(db) — confirms the persisted columns carry no principal field; the scoping is a post-load filter on session_id(), not a DB column query.
4. .spec/prds/governance/08-uc-mgmt.md [120-128] — UC-MGMT-05: the Rules tab reuses RulesList scoped by an optional principalId prop; when set, the rulesService query is scoped to that principal; when unset, behavior is identical to today (backward compatible). This task is the backend producer for that prop.
5. apps/desktop/src/lib/rules/rulesService.svelte.ts [31-41] — the existing workspaceRules(projectId) query (listWorkspaceRules.useQuery({projectId})) that MGMT-UI-010 extends to pass the optional principalId — confirms the consumer shape this task's SDK delta must satisfy.
6. crates/but-api/tests/confinement.rs [213 (and forge_guard.rs:9)] — [VERIFIED TEST IDIOM — RUST-8] the established but-api test Context construction: but_ctx::Context::from_repo(repo)?.with_memory_app_cache() over but_testsupport::writable_scenario. Use THIS, not a guessed testsupport helper, to build the real ctx+DbHandle for the rules-scoped tests.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- cargo test -p but-api list_workspace_rules_scoped_none_equals_existing   -> Exit 0; None path returns the same rule-id set as list_workspace_rules incl. the global rule + in-workspace filter
- cargo test -p but-api list_workspace_rules_scoped_some_narrows_to_principal   -> Exit 0; Some(agent-A) returns exactly rule_a; Some(agent-B) exactly rule_b; agent-B/global excluded from agent-A's scope
- cargo test -p but-api list_workspace_rules_scoped_unknown_principal_empty   -> Exit 0; Some(agent-Z) returns Ok(empty), not Err / not all rules
- cargo check -p but-rules --all-targets   -> Exit 0; WorkspaceRule schema unchanged (no invented principal column)
- cargo check -p but-api --all-targets   -> Exit 0
- cargo clippy -p but-api --all-targets   -> Exit 0
- cargo fmt --check   -> Exit 0
- pnpm build:sdk && pnpm format && grep -rq "principalId" packages/but-sdk/src/generated && pnpm -F @gitbutler/desktop check   -> Exit 0; SDK carries the optional principalId argument; desktop TS type-checks; no hand-edit
- cargo test -p but-api list_workspace_rules_scoped_cross_principal_not_disclosed   -> Exit 0; agent-A->Some(agent-B) Ok(empty); admin->Some(agent-B) exactly rule_b; principalId scope-checked, not forwarded blindly

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references:
  - .spec/prds/governance/08-uc-mgmt.md:120-128 (UC-MGMT-05 — optional principalId scoping, backward compatible)
  - .spec/prds/governance/11-e2e-testing-criteria.md:210-214 (T-MGMT-022/024/026 — RulesList reuse with principalId; principalId scopes query, unset = existing behavior)
  - .spec/prds/governance/tasks/sprint-06b-.../SPRINT.md Coverage Notes (MGMT-BE-003: verify the data model first; do not assume a scoping field exists)
notes:
  - VERIFIED DATA-MODEL FINDING (the honest grounding the SPRINT.md demands): but_rules::WorkspaceRule carries NO first-class principal column. The agent association is Filter::ClaudeCodeSessionId(String) via WorkspaceRule::session_id() -> Option<String>. The principalId scope is therefore implemented as a post-load filter on session_id() == Some(principal_id). State this in the completion report; if a true principal column is later required by the product, that is a NEW schema task (out of scope here).
  - Backward compat (AC-1): list_workspace_rules_scoped(ctx, None) MUST equal the existing list_workspace_rules(ctx) — the cleanest implementation delegates to the existing function for None and applies the additional session_id() filter for Some, so the in-workspace stack filter (rules.rs:44-66) is preserved unchanged.
  - Scoping semantics (AC-2/AC-3): Some(id) returns rules where session_id()==Some(id). Rules with session_id()==None (workspace-global rules) are NOT returned for a specific principal scope — they belong to the unscoped view. An unknown principal yields Ok(empty), never Err.
  - Identity axis consistency: the principalId argument is the agent handle (the same identity BUT_AGENT_HANDLE carries across governance); it is matched against the rule's ClaudeCode session-id association — keeping the rules scope on the same principal axis as the rest of the MGMT surface.
  - SDK delta (AC-4): the principalId is forwarded by the Tauri command and surfaced through the regenerated SDK as an optional argument; the filter lives in the but-api fn (a thin pass-through), so MGMT-UI-010's RulesList(principalId) prop is just plumbing. Never hand-edit the generated SDK.
  - Renderer-supplied principalId scope (SEC-5): principal_id arrives from the renderer and is untrusted. The rules-list Tauri command resolves the caller identity (the same fleet-owner/agent identity axis the rest of MGMT uses) and enforces a v1 self/admin-read scope: a non-admin caller scoping to another principal gets Ok(empty); an admin may scope to any principal. Do NOT forward principalId blindly (AC-5).
  - Verified test idiom (RUST-8): construct the test Context via but_ctx::Context::from_repo(repo)?.with_memory_app_cache() (confinement.rs:213 / forge_guard.rs:9) over but_testsupport::writable_scenario; seed rules via the REAL but_rules::create_rule(&mut ctx, CreateRuleRequest{...}, ctx.exclusive_worktree_access().write_permission()) entrypoint — never direct row injection, never a guessed testsupport helper.
  - None delegation (RUST-9): the cleanest AND mandated implementation has the None branch literally call list_workspace_rules(ctx); AC-1 asserts scoped(None) == list_workspace_rules(ctx) on the same ctx so the in-workspace stack filter cannot drift (structural, not fixture-dependent).
pattern: optional-argument query at the but-api boundary that reuses the existing list_workspace_rules (preserving its in-workspace filter) for the None path and layers a session_id()-based principal filter for the Some path, exposed via Tauri + a regenerated SDK delta
pattern_source: existing unscoped query = crates/but-api/src/legacy/rules.rs:41-69 (mirror its shape + preserve its filter); agent association = crates/but-rules/src/lib.rs:30-36 (WorkspaceRule::session_id); consumer shape = apps/desktop/src/lib/rules/rulesService.svelte.ts:31-41
anti_pattern: inventing a principalId column on WorkspaceRule or adding a but-db migration (the agent association is the existing ClaudeCode session id); narrowing/broadening the None path vs the existing list_workspace_rules (AC-1 regression); a no-op filter that returns all rules for a specific principalId (AC-2 fails); returning Err for an unknown principal instead of Ok(empty) (AC-3 fails); pushing the principal filter into the renderer instead of the but-api fn; hand-editing packages/but-sdk/src/generated

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
agent: implementer=rust-implementer | reviewer=rust-reviewer
rationale: A net-new but-api list query that scopes existing but-rules WorkspaceRules by an optional principal, plus its Tauri command + SDK delta. Requires honest grounding of the but-rules data model (which carries NO first-class principal column — the agent association is the ClaudeCode session id via WorkspaceRule::session_id()), backward-compatible filtering at the but-api boundary, and a real-but-db integration test via but-testsupport. These are rust-implementer competencies; rust-reviewer validates that unset principalId is byte-identical to today and the scoping filters on the real session-association field, not an invented column.
coding_standards: crates/AGENTS.md (Result<T,E> + anyhow::Context; keep types/helpers in the crate that owns the concept; solve the present problem directly — no speculative abstractions), crates/but-api/src/legacy/rules.rs (nearby pattern: #[but_api] + #[instrument] query fns over but-rules; the in-workspace stack filter idiom to preserve), RULES.md: but-api is THE API boundary; lower-level crates must NOT depend on but-api; after changing Rust APIs exposed via but-sdk, run pnpm build:sdk && pnpm format (never hand-edit generated), Rust tests: use but-testsupport for scenario creation; NEVER std::env::temp_dir(); seed rules via the real but_rules::create_rule entrypoint, not direct row injection

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: Sprint 06a MGMT-IPC-004 (the SDK regen base this task's rules-list command/SDK delta extends)
Blocks:     MGMT-UI-010 (RulesList's optional principalId prop consumes the list_workspace_rules_scoped SDK)
```
</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "MGMT-BE-003",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "rules_principal_scoped_store": {
      "description": "Real-but-db rules scenario via but_testsupport (a writable scenario with a real but_ctx::Context + DbHandle). Seed via the REAL entrypoint but_rules::create_rule (NOT direct row injection): create three WorkspaceRules \u2014 rule_a with a Filter::ClaudeCodeSessionId(\"agent-A\") (so session_id()==Some(\"agent-A\")), rule_b with Filter::ClaudeCodeSessionId(\"agent-B\"), and rule_global with NO ClaudeCodeSessionId filter (a workspace-global rule, session_id()==None). All three are enabled. Capture the full unscoped list (list_workspace_rules) BEFORE the scoped reads for the backward-compat assertion.",
      "seed_method": "public_api",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"...\"); let mut ctx = but_ctx::Context::from_repo(repo)?.with_memory_app_cache(); (the verified but-api test idiom \u2014 confinement.rs:213 / forge_guard.rs:9; do NOT guess a non-existent testsupport helper);",
        "but_rules::create_rule(&mut ctx, CreateRuleRequest{ trigger: ClaudeCodeHook, filters: vec![Filter::ClaudeCodeSessionId(\"agent-A\".into())], action: <a benign Implicit action> }, ctx.exclusive_worktree_access().write_permission()) -> rule_a (session_id()==Some(\"agent-A\"));",
        "but_rules::create_rule(&mut ctx, CreateRuleRequest{ trigger: ClaudeCodeHook, filters: vec![Filter::ClaudeCodeSessionId(\"agent-B\".into())], action: <benign> }, ctx.exclusive_worktree_access().write_permission()) -> rule_b (session_id()==Some(\"agent-B\"));",
        "but_rules::create_rule(&mut ctx, CreateRuleRequest{ trigger: FileSytemChange, filters: vec![], action: <benign> }, ctx.exclusive_worktree_access().write_permission()) -> rule_global (session_id()==None);",
        "capture list_workspace_rules(&ctx) (the existing unscoped query incl. its in-workspace stack filter) as the backward-compat baseline."
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN rules_principal_scoped_store: a real rules store with rule_a (session agent-A), rule_b (session agent-B), rule_global (no session); the baseline list_workspace_rules(&ctx) captured WHEN list_workspace_rules_scoped(&ctx, None) runs THEN list_workspace_rules_scoped(&ctx, None) DELEGATES to list_workspace_rules(&ctx) (the None branch calls the existing function directly \u2014 the filter is not re-implemented), so the returned rules are EQUAL (same rule-id set, same order/contents) to list_workspace_rules(&ctx) on the SAME ctx \u2014 all three rules present \u2014 and the in-workspace stack filter is preserved by construction (it is the SAME code path)",
      "verify": "cargo test -p but-api list_workspace_rules_scoped_none_equals_existing",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api rules query against a real but-db rules store",
        "negative_control": {
          "would_fail_if": [
            "the None path did NOT delegate to list_workspace_rules (a forked/re-implemented loader) \u2014 the scoped(None) rule-id set would differ from list_workspace_rules(ctx) on the same ctx",
            "the None path narrowed the result (e.g. accidentally applied a principal filter) \u2014 rule ids would be missing vs the baseline",
            "the None path bypassed the existing in-workspace stack filter (a divergent code path) \u2014 scoped(None) would diverge from list_workspace_rules for an out-of-workspace Codegen-stack rule",
            "a stub returned an empty list \u2014 the three seeded rule ids would be absent (the baseline excludes the empty signature)"
          ]
        },
        "evidence": {
          "artifact_type": "api_response",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "rules_principal_scoped_store",
            "action": {
              "actor": "ci",
              "steps": [
                "baseline = list_workspace_rules(&ctx) (the existing unscoped query)",
                "scoped = list_workspace_rules_scoped(&ctx, None)",
                "assert scoped == baseline (same rule-id set AND same contents/order) on the SAME ctx \u2014 the structural equivalence that proves None delegates rather than re-implements"
              ]
            },
            "end_state": {
              "must_observe": [
                "`scoped` contains exactly 3 rules (rule_a, rule_b, rule_global)",
                "the rule-id set AND order of `scoped` equals `list_workspace_rules(&ctx)` on the same ctx (None delegates by construction)",
                "the None-scoped result includes `rule_global.id` (the global rule with session_id()==None)"
              ],
              "must_not_observe": [
                "`scoped` returning 0 rules (an empty/stub result)",
                "`scoped` differing from `list_workspace_rules(&ctx)` in any rule id (the None path forked / re-implemented the filter instead of delegating)",
                "`scoped` returning a rule the baseline filters out (the None path bypassed the in-workspace stack filter)"
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
      "description": "GIVEN rules_principal_scoped_store: rule_a (session agent-A), rule_b (session agent-B), rule_global (no session) WHEN list_workspace_rules_scoped(&ctx, Some(\"agent-A\")) runs THEN the result contains ONLY rule_a (session_id()==Some(\"agent-A\")); rule_b (agent-B) is ABSENT and rule_global (no session) is ABSENT \u2014 the scope filters on the rule's ClaudeCode session association",
      "verify": "cargo test -p but-api list_workspace_rules_scoped_some_narrows_to_principal",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api rules query against a real but-db rules store",
        "negative_control": {
          "would_fail_if": [
            "the filter were a no-op (returned all 3 rules regardless of principalId) \u2014 rule_b and rule_global would appear, failing must_not_observe",
            "the filter matched the wrong field (e.g. rule id) \u2014 rule_a would not be returned for session agent-A",
            "a stub returned an empty list \u2014 rule_a would be absent (the divide-by-zero/degenerate trap; this case asserts a NON-degenerate result of exactly rule_a)"
          ]
        },
        "evidence": {
          "artifact_type": "api_response",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "rules_principal_scoped_store",
            "action": {
              "actor": "ci",
              "steps": [
                "scoped_a = list_workspace_rules_scoped(&ctx, Some(\"agent-A\"))",
                "scoped_b = list_workspace_rules_scoped(&ctx, Some(\"agent-B\"))",
                "assert each result contains only its own principal's rule"
              ]
            },
            "end_state": {
              "must_observe": [
                "`scoped_a` contains exactly 1 rule whose id == rule_a.id (session_id()==Some(\"agent-A\"))",
                "`scoped_b` contains exactly 1 rule whose id == rule_b.id (session_id()==Some(\"agent-B\"))"
              ],
              "must_not_observe": [
                "`scoped_a` containing rule_b.id (agent-B's rule leaked into agent-A's scope)",
                "`scoped_a` containing rule_global.id (a no-session global rule wrongly scoped to a principal)",
                "`scoped_a` returning all 3 rules (the filter is a no-op)",
                "`scoped_a` containing `0` rules from agent-B in agent-A's scope (`none` of agent-B's rules may leak) while `scoped_a` itself must NOT be `empty` for a known principal"
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
      "description": "GIVEN rules_principal_scoped_store: no rule has a session association of \"agent-Z\" WHEN list_workspace_rules_scoped(&ctx, Some(\"agent-Z\")) runs THEN the call returns Ok(vec![]) \u2014 an empty rule list (the Rules tab's empty/placeholder state), NOT an error and NOT all rules",
      "verify": "cargo test -p but-api list_workspace_rules_scoped_unknown_principal_empty",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-api rules query against a real but-db rules store",
        "negative_control": {
          "would_fail_if": [
            "the unknown-principal path returned ALL rules (the filter is a no-op \u2014 rule_a/rule_b/rule_global would appear)",
            "the unknown-principal path returned an Err instead of Ok(empty) (it must be a clean empty-state, not a failure)"
          ]
        },
        "evidence": {
          "artifact_type": "api_response",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "rules_principal_scoped_store",
            "action": {
              "actor": "ci",
              "steps": [
                "scoped_z = list_workspace_rules_scoped(&ctx, Some(\"agent-Z\"))",
                "assert it is Ok and empty"
              ]
            },
            "end_state": {
              "must_observe": [
                "`scoped_z` is `Ok`",
                "`scoped_z` contains exactly 0 rules (empty list)"
              ],
              "must_not_observe": [
                "`scoped_z` containing any of rule_a / rule_b / rule_global (a no-op filter returning all)",
                "`scoped_z` returning an `Err` (an unknown principal is a clean empty-state, not a failure)"
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
      "description": "GIVEN the list_workspace_rules_scoped but-api function exists with an optional principal_id argument WHEN the Tauri command for listing rules is extended to forward the optional principalId, and `pnpm build:sdk && pnpm format` runs THEN packages/but-sdk/src/generated regenerates with the optional principalId argument on the rules-list command/type; the generated TS type-checks; the principalId is a thin pass-through (the filter lives in the but-api fn, not the renderer)",
      "verify": "pnpm build:sdk && pnpm format && grep -rq \"principalId\" packages/but-sdk/src/generated && pnpm -F @gitbutler/desktop check",
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real pnpm build:sdk generation + real tsc type-check",
        "negative_control": {
          "would_fail_if": [
            "the Tauri command did not forward principalId \u2014 the generated SDK would lack the principalId argument (the grep finds nothing)",
            "the SDK were hand-edited rather than regenerated \u2014 git diff would show a manual edit to generated files, and a re-run of build:sdk would overwrite it",
            "the principal filter were implemented in the renderer instead of the but-api fn \u2014 the SDK argument would be absent and the filter would not be a pass-through",
            "a stub/static SDK generator emitted unchanged generated files (no principalId) \u2014 the grep over packages/but-sdk/src/generated would find an empty result and the regenerated files would be disconnected from the new but-api argument"
          ]
        },
        "evidence": {
          "artifact_type": "file_artifact",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "rules_principal_scoped_store",
            "action": {
              "actor": "ci",
              "steps": [
                "extend the rules-list Tauri command to forward the optional principalId to list_workspace_rules_scoped",
                "run `pnpm build:sdk && pnpm format`",
                "grep the regenerated packages/but-sdk/src/generated for principalId",
                "run the desktop TS type-check against the regenerated SDK"
              ]
            },
            "end_state": {
              "must_observe": [
                "packages/but-sdk/src/generated contains the `principalId` argument on the rules-list command/type",
                "`pnpm -F @gitbutler/desktop check` exits 0 (the regenerated SDK type-checks)"
              ],
              "must_not_observe": [
                "the regenerated SDK lacking any `principalId` reference (the command did not forward it)",
                "a hand-edited generated file (a manual edit that `pnpm build:sdk` would overwrite)",
                "the regenerated SDK having `0` references to `principalId` (an `empty`/`none` forward leaving the argument `blank` / the generated files `unchanged` after build:sdk)"
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
      "description": "GIVEN rules_principal_scoped_store: rule_a (session agent-A), rule_b (session agent-B); the renderer-supplied principal_id is untrusted input. v1 decision: a non-admin caller may only scope to its OWN principal; requesting another principal yields Ok(empty) (no rule disclosure), while an admin caller may scope to any principal WHEN the rules-list Tauri command path runs as BUT_AGENT_HANDLE=agent-A with a renderer-supplied principalId=Some(\"agent-B\") (a session-id reconnaissance attempt), and separately as admin with Some(\"agent-B\") THEN as agent-A requesting agent-B: the result is Ok(empty) \u2014 agent-B's rule_b is NOT disclosed to agent-A (the command applies the caller's identity/scope check before forwarding principalId); as admin requesting agent-B: the result contains exactly rule_b (admins may scope to any principal). principalId is NEVER forwarded blindly without an identity/scope check.",
      "verify": "cargo test -p but-api list_workspace_rules_scoped_cross_principal_not_disclosed",
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-api rules query with caller-identity scoping against a real but-db rules store",
        "negative_control": {
          "would_fail_if": [
            "the command forwarded the renderer-supplied principalId blindly \u2014 agent-A requesting agent-B would receive rule_b (a session-id recon leak), failing the Ok(empty) assertion",
            "the scope check denied the admin too (over-restrictive) \u2014 admin requesting agent-B would not receive rule_b",
            "a stub returned all rules regardless of caller \u2014 agent-A would see rule_b (no scope enforcement)",
            "the scope check were a static/disconnected no-op \u2014 the cross-principal request would still disclose agent-B's rule"
          ]
        },
        "evidence": {
          "artifact_type": "api_response",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "rules_principal_scoped_store",
            "action": {
              "actor": "ci",
              "steps": [
                "temp_env BUT_AGENT_HANDLE=agent-A under #[serial_test::serial]; run the scoped rules read with principalId=Some(\"agent-B\") through the command path -> capture result",
                "temp_env BUT_AGENT_HANDLE=admin under #[serial_test::serial]; run the scoped rules read with principalId=Some(\"agent-B\") -> capture result"
              ]
            },
            "end_state": {
              "must_observe": [
                "agent-A requesting agent-B yields `Ok` with exactly `0` rules (rule_b NOT disclosed)",
                "admin requesting agent-B yields exactly `1` rule whose id == rule_b.id (admins may scope to any principal)"
              ],
              "must_not_observe": [
                "agent-A's result containing rule_b.id (agent-B's rule leaked across principals)",
                "agent-A's result containing all 2 rules (principalId forwarded blindly / no scope check)",
                "admin's result being `empty` / `0` rules for a known principal agent-B (the scope check wrongly denied the admin, leaving the read `blank`)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "list_workspace_rules_scoped(&ctx, None) returns the same rule-id set as list_workspace_rules(&ctx), including rule_global and the in-workspace stack filter behavior",
      "verify": "cargo test -p but-api list_workspace_rules_scoped_none_equals_existing",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "list_workspace_rules_scoped(&ctx, Some(\"agent-A\")) returns exactly rule_a; rule_b and rule_global are absent",
      "verify": "cargo test -p but-api list_workspace_rules_scoped_some_narrows_to_principal",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "list_workspace_rules_scoped(&ctx, Some(\"agent-B\")) returns exactly rule_b (the filter matches the rule's session association, not a fixed value)",
      "verify": "cargo test -p but-api list_workspace_rules_scoped_some_narrows_to_principal",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "list_workspace_rules_scoped(&ctx, Some(\"agent-Z\")) (no associated rules) returns Ok(empty), not Err and not all rules",
      "verify": "cargo test -p but-api list_workspace_rules_scoped_unknown_principal_empty",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "`pnpm build:sdk && pnpm format` regenerates packages/but-sdk/src/generated with the optional principalId argument and the desktop TS type-check passes",
      "verify": "pnpm build:sdk && pnpm format && grep -rq \"principalId\" packages/but-sdk/src/generated && pnpm -F @gitbutler/desktop check",
      "maps_to_ac": "AC-4"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "description": "agent-A requesting principalId=Some(\"agent-B\") yields Ok(empty) (no cross-principal disclosure); admin requesting Some(\"agent-B\") yields exactly rule_b (renderer-supplied principalId is identity/scope-checked, never forwarded blindly)",
      "verify": "cargo test -p but-api list_workspace_rules_scoped_cross_principal_not_disclosed",
      "maps_to_ac": "AC-5"
    }
  ]
}
-->
