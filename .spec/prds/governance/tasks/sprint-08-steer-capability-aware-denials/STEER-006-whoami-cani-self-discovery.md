# STEER-006: `but whoami` / `but can-i` self-scoped discovery (effective perms + own group memberships + authorized-action set), reusing Sprint 05 self-scoping; surface `but perm list` as the menu discovery affordance (degrade if absent); no other-group-member enumeration

## What this does

Ship net-new self-scoped discovery: `but whoami` (full self picture) and `but can-i <authority>` (does the caller hold it), backed by net-new `but-api` functions over real `but-authz`+gix, and surface the existing `but perm list` as the discovery affordance on every actor-correctable denial — degraded (omitted) when no discovery verb exists, never a phantom command. Discovery returns the caller's effective permissions + its OWN group memberships + its authorized-action set, self-scoped: it does not enumerate other group members and cross-principal recon is denied `perm.denied`.

## Why

Sprint 08 (STEER — Capability-Aware Denials) · PRD UC-STEER-04 · Capability CAP-STEER-01. Running `but whoami` as a resolved principal prints its effective AuthoritySet, its own group memberships, and its authorized-action set; an actor-correctable denial's `authorized_actions` includes the `but perm list` discovery affordance (and omits it if absent); a denial carries no inline `groups` field; `but can-i --principal <other>` (or `but whoami` targeting another) by a non-admin caller is denied `perm.denied` with no leak; `but whoami` as `rev` shows its own `reviewers` membership but never the other members of `reviewers`. A cross-principal `but can-i --principal <unknown_id> <authority>` returns the same `perm.denied` code and a non-existence-revealing message as a cross-principal call against an existing target — discovery cannot enumerate principal ids.

## How to verify

PRIMARY **AC-1** — `cargo test -p but steer_discovery_affordance_surfaced_in_menu` (Discovery affordance surfaced in the menu when the verb exists [PRIMARY]). Full gate set in the spec below.

## Scope

- crates/but-api/src/legacy/governance.rs (MODIFY) — add net-new whoami/can_i discovery fns + their outcome structs beside perm_list, reusing the existing scope predicate
- crates/but/src/args/whoami.rs (NEW) and crates/but/src/args/can_i.rs (NEW) — or a single discovery args module; NO --as flag
- crates/but/src/args/mod.rs (MODIFY) — add the Subcommands variant(s) + pub mod only
- crates/but/src/command/whoami.rs (NEW) and crates/but/src/command/can_i.rs (NEW) — thin shims mirroring command/perm.rs
- crates/but/src/command/mod.rs (MODIFY) — pub mod additions only
- crates/but/src/command/help.rs (MODIFY) — the exhaustive SubcommandDiscriminant grouping arm(s) only
- crates/but/src/lib.rs (MODIFY) — the dispatch arm(s) only (beside :489)
- crates/but/src/utils/metrics.rs (MODIFY) — the metrics arm(s) only
- crates/but-api/tests/perm_governance.rs (MODIFY) or a new crates/but-api/tests/steer_discovery.rs (NEW) — but-api discovery proofs
- crates/but/tests/but/command/steer_discovery.rs (NEW) + crates/but/tests/but/command/mod.rs (MODIFY — mod steer_discovery;) — CLI proofs

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: STEER-006 - `but whoami` / `but can-i` self-scoped discovery (effective perms + own group memberships + authorized-action set), reusing Sprint 05 self-scoping; surface `but perm list` as the menu discovery affordance (degrade if absent); no other-group-member enumeration
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     L  (210 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-STEER-04
CAPABILITIES: CAP-STEER-01

RUNTIME_COMMANDS:
  test:  cargo test -p but steer_discovery_affordance_surfaced_in_menu steer_discovery_affordance_omitted_when_absent   |   cargo test -p but steer_whoami_returns_full_self_picture steer_whoami_hides_other_group_members   |   cargo test -p but steer_discovery_cross_principal_denied && cargo test -p but steer_discovery_membership_not_inline   |   cargo test -p but-authz --test invariant_build_gates
  lint:  cargo clippy -p but-api -p but --all-targets && cargo check -p but-api -p but --all-targets && cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
Running `but whoami` as a resolved principal prints its effective AuthoritySet, its own group memberships, and its authorized-action set; an actor-correctable denial's `authorized_actions` includes the `but perm list` discovery affordance (and omits it if absent); a denial carries no inline `groups` field; `but can-i --principal <other>` (or `but whoami` targeting another) by a non-admin caller is denied `perm.denied` with no leak; `but whoami` as `rev` shows its own `reviewers` membership but never the other members of `reviewers`. A cross-principal `but can-i --principal <unknown_id> <authority>` returns the same `perm.denied` code and a non-existence-revealing message as a cross-principal call against an existing target — discovery cannot enumerate principal ids.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST site the discovery logic at the `but-api` boundary (`crates/but-api/src/legacy/governance.rs`, beside `perm_list` at :63) as net-new `whoami(repo, target_ref)` / `can_i(repo, target_ref, &authority_token)` functions reusing `but_authz::resolve_principal_from_env(&config)` + `but_authz::effective_authority(&caller, &config)` + `Principal::groups()` — NEVER bury discovery in `crates/but/` (mirror the CLI-001 shim split so Sprint 06a IPC can reuse).
- [MUST] MUST keep discovery self-scoped exactly as `perm_list` is: `but can-i --principal <other>` (or any cross-principal target) follows the SAME scope predicate as `perm_list` (:73-84 — allowed only when caller IS the target OR holds AdministrationRead/AdministrationWrite), returning `Denial::missing_permission(AdministrationRead, &held)` -> `perm.denied` otherwise, leaking NOTHING about `<other>`.
- [MUST] MUST disclose the caller's OWN group memberships (`Principal::groups()` -> the `GroupName` list) but MUST NOT enumerate the OTHER members of those groups (do not read `Group::members()` for any group and emit other principals' ids) — group-roster recon stays gated by `administration:read` (Sprint 05).
- [NEVER] NEVER add group/team membership inline to the denial payload by default — the effective set already subsumes group grants; membership is served on request via discovery only (T-STEER-018).
- [NEVER] NEVER emit a phantom discovery command: the discovery affordance in `AFFORDANCE_MAP[discovery]` references a verb that EXISTS (`but perm list`, or `but whoami`/`but can-i` once shipped here); if no discovery verb is resolvable it is OMITTED from `authorized_actions`, never offered as a non-existent command (no lying menu).
- [NEVER] NEVER define an `--as`/identity-override flag on `whoami`/`can-i`; caller identity comes from `BUT_AGENT_HANDLE` only (mirror the CLI-001 `--as` rejection).
- [NEVER] NEVER let the cross-principal `can-i`/`whoami` path reveal whether the target principal EXISTS: an unknown target and an unauthorized-but-existing target must both return `perm.denied` with no existence-distinguishing wording (`unknown principal`/`not found`) — otherwise the endpoint is a principal-id enumeration oracle.
- [STRICTLY] STRICTLY draw the discovery affordance `command`/`effect` text from the closed `&'static str` CATALOG (STEER-003) — never `format!`, never config-sourced — so STEER-010's closed-catalog grep stays green.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: Discovery affordance surfaced in the menu when the verb exists [PRIMARY]
- [ ] AC-2: Group membership NOT inline in the denial by default
- [ ] AC-3: Discovery returns the full self picture (effective perms + own groups + authorized-action set)
- [ ] AC-4: Cross-principal recon denied perm.denied (self-scoped)
- [ ] AC-5: whoami/can-i does not list other members of the caller's groups
- [ ] AC-6: Discovery is not a principal-existence oracle (unknown-target indistinguishable from insufficient-authority)
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: Discovery affordance surfaced in the menu when the verb exists [PRIMARY] [PRIMARY]
  GIVEN: the committed `reviewers_group_with_members` fixture; `rev` (effective {comments:write, reviews:write via group}) attempts a governed action it cannot perform (a commit to protected `main`) with BUT_AGENT_HANDLE=rev, and `but perm list` is present (Sprint 05)
  WHEN:  the actor-correctable denial JSON is emitted on stderr and `authorized_actions` is read
  THEN:  `authorized_actions` includes an entry whose `command` is the self-scoped discovery command (`but perm list`) with a non-empty catalog `effect`; the discovery entry is present alongside review affordances
  TEST_TIER: integration   VERIFICATION_SERVICE: but-cli
  VERIFY: cargo test -p but steer_discovery_affordance_surfaced_in_menu
  SCENARIO: would fail if lying-menu; phantom-command; empty; static | must observe: an `authorized_actions` entry with `command` `== "but perm list"` (the self-scoped discovery command); a non-empty `effect` string for that entry such as `"see full permissions, groups, and authorized actions"` (length `>= 1`) | must NOT observe: an `authorized_actions` entry naming a discovery command that does not exist (a phantom/`no` real verb); an `empty` authorized_actions array

AC-2: Group membership NOT inline in the denial by default
  GIVEN: the committed `reviewers_group_with_members` fixture; `rev` (member of groups `reviewers`) hits an actor-correctable denial
  WHEN:  the denial JSON on stderr is parsed for an inline membership field
  THEN:  the denial object has NO inline `groups`/`memberships` key; the effective set (held_permissions, populated by STEER-004) carries the grants, and membership is reachable only via the discovery affordance
  TEST_TIER: api-contract   VERIFICATION_SERVICE: but-cli
  VERIFY: cargo test -p but steer_discovery_membership_not_inline
  SCENARIO: would fail if static; mock; empty | must observe: `error.held_permissions` containing `"reviews:write"`; the `error` object key set excludes `"groups"` and `"memberships"` (`0` such keys) | must NOT observe: an inline `error.groups` array listing `"reviewers"` (there must be `no` such key); an `empty` held_permissions when the caller holds grants

AC-3: Discovery returns the full self picture (effective perms + own groups + authorized-action set)
  GIVEN: the committed `reviewers_group_with_members` fixture; BUT_AGENT_HANDLE=rev
  WHEN:  `but whoami` runs (no target override)
  THEN:  exit 0; output shows `rev`'s effective permissions (comments:write AND reviews:write, the latter via the `reviewers` group), `rev`'s OWN group membership (`reviewers`), and `rev`'s authorized-action set — the self-scoped disclosure class of `but perm list`
  TEST_TIER: integration   VERIFICATION_SERVICE: but-cli
  VERIFY: cargo test -p but steer_whoami_returns_full_self_picture
  SCENARIO: would fail if stub; empty; static | must observe: the literal `"comments:write"`; the literal `"reviews:write"`; group membership `"reviewers"`; at least `1` `authorized_actions` entry with a `but ` command | must NOT observe: an `empty` permission listing; `"reviews:write"` absent (the group grant must be folded into the effective set; `0` such omissions)

AC-4: Cross-principal recon denied perm.denied (self-scoped)
  GIVEN: the committed `reviewers_group_with_members` fixture; caller `rev` is a registered, resolved, non-admin principal (holds comments:write/reviews:write only, NOT administration:read)
  WHEN:  `but can-i --principal maint merge` (or `but whoami --principal maint`) runs with BUT_AGENT_HANDLE=rev, targeting another principal
  THEN:  exit 1 with structured `perm.denied`; the output does NOT contain `maint`'s effective set (no `merge`, no enumeration) — the same scope decision as Sprint 05 `perm_list`; a self-targeted `but can-i reviews:write` by `rev` returns exit 0 (proving the caller resolves and the denial is the scope decision, not unknown-principal)
  TEST_TIER: integration   VERIFICATION_SERVICE: but-cli
  VERIFY: cargo test -p but steer_discovery_cross_principal_denied
  SCENARIO: would fail if disconnect; stub; static | must observe: `error.code` `== "perm.denied"` on the cross-principal call with exit `1`; exit `0` on the self `can-i reviews:write` call | must NOT observe: the token `"merge"` or maint's effective set in the cross-principal output (`0` leaked entries); exit `0` on the cross-principal recon call

AC-5: whoami/can-i does not list other members of the caller's groups
  GIVEN: the committed `reviewers_group_with_members` fixture where `reviewers` has members {rev, rev2}; BUT_AGENT_HANDLE=rev; caller lacks administration:read
  WHEN:  `but whoami` runs
  THEN:  exit 0; output shows `rev`'s OWN membership in `reviewers` but does NOT list `rev2` (or any other member of `reviewers`) — group-roster recon stays gated by administration:read (Sprint 05)
  TEST_TIER: integration   VERIFICATION_SERVICE: but-cli
  VERIFY: cargo test -p but steer_whoami_hides_other_group_members
  SCENARIO: would fail if disconnect; stub; static | must observe: the caller's own membership `"reviewers"` | must NOT observe: the other member id `"rev2"` (`0` other-member entries); any member-roster enumeration of the reviewers group (`none`)

AC-6: Discovery is not a principal-existence oracle (unknown-target indistinguishable from insufficient-authority)
  GIVEN: the committed `reviewers_group_with_members` fixture; caller `rev` is a resolved, non-admin principal (NOT administration:read); a target principal id `ghost-9f3a` that does NOT exist in `.gitbutler/permissions.toml`
  WHEN:  `but can-i --principal ghost-9f3a merge` runs (an unknown target) with BUT_AGENT_HANDLE=rev, compared against `but can-i --principal maint merge` (an existing target rev cannot recon)
  THEN:  BOTH cross-principal calls return exit 1 with the SAME `perm.denied` code, the SAME `class` (`actor_correctable`/`operator_required` — a `class` divergence would itself be an existence oracle), the SAME canonical denial `message` (or a message that differs ONLY by the target principal id, which is not an existence-revealing distinction), and the SAME `authorized_actions`/`held_permissions`/`do_not` envelope shape. The message contains no existence-revealing token distinguishing "unknown principal" from "insufficient authority" (no "unknown principal"/"not found"/"no such principal" wording), so the endpoint cannot be used to enumerate principal ids
  TEST_TIER: integration   VERIFICATION_SERVICE: but-cli
  VERIFY: cargo test -p but steer_discovery_not_a_principal_oracle
  SCENARIO: would fail if disconnect; stub; static; oracle-leak via wording, length, or shape | must observe: `error.code` `== "perm.denied"` on the unknown-target call with exit `1`; `error.code` `== "perm.denied"` on the existing-target call with exit `1` (the codes are identical); `error.class` on the unknown-target call `==` `error.class` on the existing-target call (a `class` divergence is itself existence-revealing); normalizing the target id in the message, the two messages are identical (or any difference is confined to the target-id substring); `authorized_actions` length on the unknown-target call `==` `authorized_actions` length on the existing-target call; `held_permissions` length on the unknown-target call `==` `held_permissions` length on the existing-target call | must NOT observe: the unknown-target `message` containing `"unknown principal"`, `"not found"`, or `"no such principal"` (`0` existence-revealing tokens); an `empty` / differing code that distinguishes the unknown target from the existing target; a differing `class` between the two paths; a difference in `message` beyond the substituted target principal id; a difference in the number, values, or shape of steering fields (`authorized_actions`, `held_permissions`, `do_not`) between the two denials; timing/channel side-channels being asserted as a substitute for equality

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, happy_path): An actor-correctable denial's authorized_actions includes an entry with command "but perm list" and a non-empty catalog effect
    VERIFY: cargo test -p but steer_discovery_affordance_surfaced_in_menu
- TC-2 (-> AC-1, edge): When no discovery verb is resolvable, the discovery affordance is OMITTED from authorized_actions rather than offered as a phantom command
    VERIFY: cargo test -p but steer_discovery_affordance_omitted_when_absent
- TC-3 (-> AC-2, happy_path): The denial JSON has no inline groups/memberships key while held_permissions carries the effective set
    VERIFY: cargo test -p but steer_discovery_membership_not_inline
- TC-4 (-> AC-3, happy_path): but whoami as rev prints comments:write, reviews:write (group-folded), membership reviewers, and an authorized-action set
    VERIFY: cargo test -p but steer_whoami_returns_full_self_picture
- TC-5 (-> AC-4, error): but can-i --principal maint merge by rev is Err perm.denied with no leak of maint's set; self can-i reviews:write by rev is Ok
    VERIFY: cargo test -p but steer_discovery_cross_principal_denied
- TC-6 (-> AC-5, edge): but whoami as rev shows membership reviewers but never lists the other member rev2
    VERIFY: cargo test -p but steer_whoami_hides_other_group_members
- TC-7 (-> AC-6, edge): `but can-i --principal <unknown>` by rev returns the same `perm.denied` code, the same `class`, AND, after normalizing the target principal id, the same canonical message and the same authorized_actions/held_permissions/do_not envelope shape as `--principal <existing>` — no principal-id enumeration oracle
    VERIFY: cargo test -p but steer_discovery_not_a_principal_oracle

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-STEER-01
provides: but-cli-self-discovery-verbs (`but whoami` / `but can-i`); but-api `whoami`/`can_i` discovery functions (self-scoped: effective AuthoritySet + own GroupName memberships + authorized-action set); the discovery affordance entry surfaced in AFFORDANCE_MAP[discovery] referencing `but perm list`
consumes: STEER-004 wired steering payload (the `authorized_actions` menu the discovery affordance is appended to); Sprint 05 `but perm list` (`crates/but/src/command/perm.rs` -> `but_api::legacy::governance::perm_list`); but_authz::{resolve_principal_from_env, effective_authority, load_governance_config, Principal::groups, AuthoritySet, GroupName, Denial}; but_testsupport::{writable_scenario, invoke_bash}
boundary_contracts:
  - CAP-STEER-01: discovery is self-scoped — it discloses ONLY the caller's own effective set + own group memberships, never the other members of those groups, and never another principal's set (cross-principal recon stays gated by administration:read per Sprint 05). The discovery affordance is non-phantom: surfaced only when the discovery verb exists; degraded (omitted) otherwise — no lying menu. Discovery is not a principal-existence oracle: a cross-principal `can-i` against an UNKNOWN target returns the SAME `perm.denied` code + a message that does not reveal whether the target exists (no unknown-principal vs insufficient-authority distinction leaking through), so the endpoint cannot enumerate principal ids.

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-api/src/legacy/governance.rs (MODIFY) — add net-new whoami/can_i discovery fns + their outcome structs beside perm_list, reusing the existing scope predicate
  - crates/but/src/args/whoami.rs (NEW) and crates/but/src/args/can_i.rs (NEW) — or a single discovery args module; NO --as flag
  - crates/but/src/args/mod.rs (MODIFY) — add the Subcommands variant(s) + pub mod only
  - crates/but/src/command/whoami.rs (NEW) and crates/but/src/command/can_i.rs (NEW) — thin shims mirroring command/perm.rs
  - crates/but/src/command/mod.rs (MODIFY) — pub mod additions only
  - crates/but/src/command/help.rs (MODIFY) — the exhaustive SubcommandDiscriminant grouping arm(s) only
  - crates/but/src/lib.rs (MODIFY) — the dispatch arm(s) only (beside :489)
  - crates/but/src/utils/metrics.rs (MODIFY) — the metrics arm(s) only
  - crates/but-api/tests/perm_governance.rs (MODIFY) or a new crates/but-api/tests/steer_discovery.rs (NEW) — but-api discovery proofs
  - crates/but/tests/but/command/steer_discovery.rs (NEW) + crates/but/tests/but/command/mod.rs (MODIFY — mod steer_discovery;) — CLI proofs
writeProhibited:
  - the gate deny/allow decision (commit/merge/forge gates, authorize.rs) - NEVER weaken
  - Sprint 05 perm_grant/perm_revoke/perm_list logic - CONSUME-only; do not fork the scope predicate, reuse it
  - crates/but-authz/tests/invariant_build_gates.rs honesty-grep patterns - NEVER replace the shipped no-role-preset/no-human-vs-AI/positive-authorize/no-Permission patterns; add beside (STEER-010 owns net-new additions)
  - .spec/prds/governance/tasks/sprint-0[1-6]* - frozen
  - Any file not explicitly listed above

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
  - crates/but-api/src/legacy/governance.rs (lines 63-119): PRIMARY PATTERN — perm_list: resolve_principal_from_env + effective_authority + the self-or-admin-read scope predicate (:73-84). Add whoami/can_i here as net-new fns reusing the SAME scope predicate and the same self-scoping; do not enumerate Group::members().
  - crates/but/src/command/perm.rs (lines all): the thin CLI shim shape (resolve_target_ref + repo + call the but-api fn + print/structured-error). whoami/can-i shims mirror this; reuse resolve_target_ref.
  - crates/but/src/args/perm.rs (lines all): clap Platform/Subcommands shape (no --as flag). Add args/whoami.rs + args/can_i.rs (or a single discovery noun) following this; wire into args/mod.rs (Subcommands variant ~:1045 + pub mod ~:1284), lib.rs dispatch (~:489), command/mod.rs (:17), utils/metrics.rs.
  - crates/but-authz/src/principal.rs (lines 82-141): Principal::groups() returns &[GroupName] — the caller's OWN membership. Group::members() (:212) is what discovery must NOT read for other-member enumeration.
  - crates/but/tests/but/command/governed_loop.rs (lines 263-347, 465-498): test fixture style (governance commit via invoke_bash) + the CliErrorEnvelope reader; mirror for the discovery tests' fixture seeding and stderr-JSON parsing.

--------------------------------------------------------------------------------
CODE PATTERN
--------------------------------------------------------------------------------
pattern: thin-CLI-shim-over-but-api boundary fn (mirror CLI-001): args module -> command shim -> but_api::legacy::governance fn that resolves the principal, computes effective_authority + Principal::groups(), and applies the self-or-admin-read scope.
pattern_source: crates/but-api/src/legacy/governance.rs:63-119 (perm_list) + crates/but/src/command/perm.rs:1-133
anti_pattern: Reading Group::members() to enrich the self picture (leaks other members); adding an inline `groups` field to the denial; offering a discovery command string that no clap subcommand backs (phantom/lying menu).
references: 03-technical-requirements-delta.md §6 (self-discovery, net-new CLI surface, degradable); 02-uc-steer.md UC-STEER-04 AC-1..5; 05-delta-replan.md D8
interaction_notes:
  - whoami/can_i are net-new but reuse the perm_list self-or-admin-read scope predicate verbatim
  - the discovery affordance entry text comes from STEER-003's CATALOG[discovery]; this task supplies the verb that backs it

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
depends_on: STEER-004
blocks: (none)

CODING STANDARDS: crates/AGENTS.md, crates/but/AGENTS.md, RULES.md
```

</details>

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN a committed governance fixture and rev hitting an actor-correctable denial with `but perm list` present, WHEN the denial JSON is read, THEN authorized_actions includes the `but perm list` discovery affordance with a catalog effect",
      "verify": "cargo test -p but steer_discovery_affordance_surfaced_in_menu"
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "GIVEN rev (member of reviewers) hits an actor-correctable denial, WHEN the denial JSON is parsed, THEN it has no inline groups key but carries held_permissions",
      "verify": "cargo test -p but steer_discovery_membership_not_inline"
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "GIVEN rev with own + group grants, WHEN but whoami runs, THEN it shows effective perms (incl group-folded reviews:write), own membership reviewers, and authorized-action set",
      "verify": "cargo test -p but steer_whoami_returns_full_self_picture"
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "description": "GIVEN resolved non-admin rev, WHEN but can-i targets maint, THEN perm.denied with no leak; self can-i is Ok",
      "verify": "cargo test -p but steer_discovery_cross_principal_denied"
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "description": "GIVEN reviewers has members {rev, rev2}, WHEN rev runs but whoami without administration:read, THEN own membership shown, rev2 not listed",
      "verify": "cargo test -p but steer_whoami_hides_other_group_members"
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "authorized_actions includes `but perm list` with a non-empty effect",
      "maps_to_ac": "AC-1",
      "verify": "cargo test -p but steer_discovery_affordance_surfaced_in_menu"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "discovery affordance omitted (not phantom) when no verb exists",
      "maps_to_ac": "AC-1",
      "verify": "cargo test -p but steer_discovery_affordance_omitted_when_absent"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "no inline groups key; held_permissions present",
      "maps_to_ac": "AC-2",
      "verify": "cargo test -p but steer_discovery_membership_not_inline"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "whoami shows full self picture incl group-folded grant + membership + actions",
      "maps_to_ac": "AC-3",
      "verify": "cargo test -p but steer_whoami_returns_full_self_picture"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "cross-principal can-i denied perm.denied no leak; self can-i Ok",
      "maps_to_ac": "AC-4",
      "verify": "cargo test -p but steer_discovery_cross_principal_denied"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "description": "whoami hides other group members (rev2)",
      "maps_to_ac": "AC-5",
      "verify": "cargo test -p but steer_whoami_hides_other_group_members"
    },
    {
      "id": "AC-6",
      "type": "acceptance_criterion",
      "description": "GIVEN resolved non-admin rev and an unknown target id, WHEN but can-i targets it vs. an existing target, THEN both return the same perm.denied code, the same class, the same canonical message (after normalizing the target id), and the same authorized_actions/held_permissions/do_not envelope shape — no enumeration oracle",
      "verify": "cargo test -p but steer_discovery_not_a_principal_oracle"
    },
    {
      "id": "TC-7",
      "type": "test_criterion",
      "description": "unknown-target can-i has identical code, identical class, identical normalized message, and identical steering envelope shape to existing-target can-i (no oracle)",
      "maps_to_ac": "AC-6",
      "verify": "cargo test -p but steer_discovery_not_a_principal_oracle"
    }
  ]
}
-->
