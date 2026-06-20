# AUTHZ-003: `authorize()` + `BUT_AGENT_HANDLE` resolution + fail-closed default-deny

## What this does

Implement the single enforcement primitive `authorize(principal, action, cfg) -> Result<(), Denial>` plus `effective_authority` (own ∪ groups) and the `BUT_AGENT_HANDLE`→Principal resolver (injected-lookup, fail-closed), so a held permission proceeds and a missing handle / unknown principal / missing permission all deny with the structured contract.

## Why

Sprint 01a · PRD UC-AUTHZ-02, UC-AUTHZ-03, UC-AUTHZ-04 · capabilities CAP-AUTHZ-01. Part of the functional-permission governance walking skeleton (commit allow/deny through real `but-authz` + real git).

## How to verify

PRIMARY **AC-1** — `cargo test -p but-authz authorize_held_vs_missing` (integration). Full gate set in the spec below.

## Scope

- crates/but-authz/src/authorize.rs (NEW)
- crates/but-authz/src/lib.rs (MODIFY) — re-export authorize/effective_authority/resolve_principal/Denial constructors
- crates/but-authz/tests/authorize.rs (NEW)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: AUTHZ-003 - `authorize()` + `BUT_AGENT_HANDLE` resolution + fail-closed default-deny
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     M  (180 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-AUTHZ-02, UC-AUTHZ-03, UC-AUTHZ-04
CAPABILITIES: CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-authz authorize_held_vs_missing
  check: cargo check -p but-authz --all-targets
  lint:  cargo clippy --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
`cargo test -p but-authz authorize` is green: a principal holding `contents:write` is authorized for `ContentsWrite` (Ok); a principal lacking it is denied with `Denial.code=="perm.denied"`, a message naming the missing `contents:write`, and a non-empty `remediation_hint`; an unknown handle (absent from config) is denied; an unset `BUT_AGENT_HANDLE` is rejected with no principal resolved; effective_authority returns own ∪ group grants.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST resolve the acting principal ONLY from `BUT_AGENT_HANDLE` looked up against the committed GovConfig — a missing/empty handle is rejected (no anonymous action) and a handle absent from config is denied (unknown principal), never run as an implicit/default principal.
- [MUST] MUST make `authorize` test the required `Authority` against the principal's EFFECTIVE set (own grants ∪ every group's grants from GovConfig) — never against a role name or handle string (CAP-AUTHZ-01; AUTHZ-007 greps this).
- [MUST] MUST construct `Denial{ code:"perm.denied", message, remediation_hint }` naming the missing permission in `message` and a legitimate alternative in a non-empty `remediation_hint`.
- [NEVER] NEVER default to allow when the principal is unknown, the handle is unresolvable, or the GovConfig is the fail-closed/empty state — default-deny is the whole point (UC-AUTHZ-04).
- [NEVER] NEVER read the principal's AuthoritySet from an agent-supplied claim/argument — only from the committed config (UC-AUTHZ-03 AC-3).
- [STRICTLY] STRICTLY mirror `detect_agent::detect_with(lookup: impl Fn(&str)->Option<OsString>)` — make handle resolution take an injected lookup so tests drive it without mutating process env; the thin `resolve_principal_from_env()` wrapper passes `std::env::var_os`.
- [STRICTLY] STRICTLY keep `authorize` pure (no I/O): it takes a `&Principal` + `Authority` + `&GovConfig` and returns `Result<(), Denial>` — config reading happened in AUTHZ-002.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: Held permission authorizes; missing permission denies with the structured contract [PRIMARY]
- [ ] AC-2: Unset or empty BUT_AGENT_HANDLE is rejected — no anonymous action
- [ ] AC-3: Unknown principal (handle absent from config) is denied — fail closed
- [ ] AC-4: Effective authority is own grants ∪ group grants
- [ ] AC-5: Authority is read only from the committed GovConfig; an agent-supplied claim is ignored
- [ ] All verification gates pass; only write_allowed files modified (git diff --name-only)

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: Held permission authorizes; missing permission denies with the structured contract [PRIMARY] [PRIMARY]
  GIVEN: fixture `principals_cfg` loaded into a GovConfig
  WHEN:  `authorize(dev, ContentsWrite, cfg)` and `authorize(ro, ContentsWrite, cfg)` are called
  THEN:  `dev` returns `Ok(())`; `ro` returns `Err(Denial)` with `code=="perm.denied"`, a message naming `contents:write`, and a non-empty `remediation_hint`
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-authz + GovConfig loaded from a real git repo (but-testsupport)
  VERIFY: cargo test -p but-authz authorize_held_vs_missing
  SCENARIO (tier=visible, test_tier=integration):
    NEGATIVE_CONTROL would fail if: authorize always returns Ok (the canonical fail-open stub); Denial message is empty / does not name the missing permission; remediation_hint is empty
    EVIDENCE: stdout (required_capture=True)
    case[0] (api_client): authorize(dev, ContentsWrite, cfg); assert Ok(())
      MUST_OBSERVE:     ['`authorize(dev, ContentsWrite, cfg)` returns `Ok(())`']
      MUST_NOT_OBSERVE: ['`Err(`', 'code `"perm.denied"`', 'no Denial returned']
    case[1] (api_client): authorize(ro, ContentsWrite, cfg); assert Err(Denial); assert code==perm.denied; assert message contains "contents:write"; assert remediation_hint non-empty
      MUST_OBSERVE:     ['`code == "perm.denied"`', '`message` contains `"contents:write"`', '`remediation_hint` contains `"reviewed merge"`']
      MUST_NOT_OBSERVE: ['`Ok(())`', 'empty `remediation_hint`']

AC-2: Unset or empty BUT_AGENT_HANDLE is rejected — no anonymous action
  GIVEN: fixture `principals_cfg` and an injected env lookup where `BUT_AGENT_HANDLE` is unset, and separately Some(empty string)
  WHEN:  `resolve_principal(lookup, cfg)` is called
  THEN:  both the unset and the empty-string handle return `Err(Denial)` (no principal resolved, no default identity) — the action cannot proceed anonymously
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-authz + injected env lookup + committed GovConfig
  VERIFY: cargo test -p but-authz resolve_no_handle_rejected
  SCENARIO (tier=visible, test_tier=integration):
    NEGATIVE_CONTROL would fail if: resolver falls back to a default/anonymous principal when the handle is unset; resolver returns Ok with an empty handle; resolver panics instead of returning a typed Denial; resolver returns Ok with an empty handle (Some("") accepted as a principal)
    EVIDENCE: stdout (required_capture=True)
    case[0] (api_client): resolve_principal(|_| None /* BUT_AGENT_HANDLE unset */, cfg); assert Err(Denial)
      MUST_OBSERVE:     ['`resolve_principal` returns `Err(Denial)`', 'no principal resolved (`code == "perm.denied"`)']
      MUST_NOT_OBSERVE: ['`Ok(Principal`', 'default principal', 'anonymous principal']
    case[1] (api_client): resolve_principal(|k| (k=="BUT_AGENT_HANDLE").then(|| OsString::from("")) /* Some(empty string) */, cfg); assert Err(Denial) — same rejection as unset
      MUST_OBSERVE:     ['`resolve_principal` returns `Err(Denial)` for an empty-string handle', '`code == "perm.denied"` (no principal resolved)']
      MUST_NOT_OBSERVE: ['`Ok(Principal`', 'default principal', 'empty handle accepted (`Ok(`)']

AC-3: Unknown principal (handle absent from config) is denied — fail closed
  GIVEN: fixture `principals_cfg` and an injected lookup where `BUT_AGENT_HANDLE=ghost` (not in config)
  WHEN:  `resolve_principal(lookup, cfg)` (or `authorize` after it) is called
  THEN:  it denies with the `perm.denied` contract — an unknown principal never defaults to allow
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-authz + injected env lookup + committed GovConfig
  VERIFY: cargo test -p but-authz resolve_unknown_principal_denied
  SCENARIO (tier=visible, test_tier=integration):
    NEGATIVE_CONTROL would fail if: an unknown handle is granted an implicit empty-but-allowed identity; missing principal yields Ok; resolver invents a principal from the handle string
    EVIDENCE: stdout (required_capture=True)
    case[0] (api_client): resolve_principal(|k| (k=="BUT_AGENT_HANDLE").then(|| OsString::from("ghost")), cfg); assert Err(Denial) with code perm.denied
      MUST_OBSERVE:     ['`code == "perm.denied"`', 'principal `"ghost"` not found']
      MUST_NOT_OBSERVE: ['`Ok(Principal`', 'default-allow', 'no Denial returned']

AC-4: Effective authority is own grants ∪ group grants
  GIVEN: fixture `principals_cfg` where `reviewer` has no direct grants but is a member of `code-reviewers` (reviews:write)
  WHEN:  `effective_authority(reviewer, cfg)` is computed and `authorize(reviewer, ReviewsWrite, cfg)` is called
  THEN:  the effective set contains `reviews:write` (inherited via the group) so authorize returns `Ok(())`, while `authorize(reviewer, Merge, cfg)` denies
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-authz + committed GovConfig
  VERIFY: cargo test -p but-authz effective_authority_union
  SCENARIO (tier=visible, test_tier=integration):
    NEGATIVE_CONTROL would fail if: group grants are ignored so reviewer has an empty effective set; effective_authority returns only direct grants; authorize grants Merge it never inherited
    EVIDENCE: stdout (required_capture=True)
    case[0] (api_client): authorize(reviewer, ReviewsWrite, cfg); assert Ok(()); authorize(reviewer, Merge, cfg); assert Err perm.denied
      MUST_OBSERVE:     ['`authorize(reviewer, ReviewsWrite, cfg)` returns `Ok(())`', '`authorize(reviewer, Merge, cfg)` returns `code == "perm.denied"`']
      MUST_NOT_OBSERVE: ['`ReviewsWrite` returns `"perm.denied"`', '`Merge` returns `Ok(())`', 'empty effective set']

AC-5: Authority is read only from the committed GovConfig; an agent-supplied claim is ignored
  GIVEN: fixture `principals_cfg` where `ro` holds only contents:read in committed config
  WHEN:  authorize is evaluated for `ro` — and the test confirms there is NO parameter/field by which a caller could inject an authority claim (authorize takes only &Principal + Authority + &GovConfig)
  THEN:  `ro` is denied ContentsWrite (perm.denied) regardless of any caller intent — the effective set is computed solely from `cfg`, and `authorize`'s signature exposes no agent-claim input (structural + behavioral)
  TEST_TIER: integration   VERIFICATION_SERVICE: real but-authz + committed GovConfig
  VERIFY: cargo test -p but-authz authority_only_from_config
  SCENARIO (tier=holdout, test_tier=integration):
    NEGATIVE_CONTROL would fail if: authorize accepts an agent-supplied AuthoritySet/claim argument that overrides the committed config; ro is allowed ContentsWrite because a claim was honored; effective set is read from the Principal struct's caller-set field instead of cfg; authorize reads the effective set from a hardcoded/caller-supplied static field disconnected from the committed cfg
    EVIDENCE: stdout (required_capture=True)
    case[0] (api_client): authorize(ro, ContentsWrite, cfg); assert Err perm.denied (the committed config grants ro only contents:read; no claim path exists to widen it)
      MUST_OBSERVE:     ['`authorize(ro, ContentsWrite, cfg)` returns `code == "perm.denied"` — authority sourced only from `cfg`']
      MUST_NOT_OBSERVE: ['`Ok(())`', 'ro granted via a caller-supplied claim', 'default-allow']

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, happy_path): authorize(dev, ContentsWrite) is Ok; authorize(ro, ContentsWrite) is Err perm.denied naming contents:write (T-AUTHZ-012/013)
    VERIFY: cargo test -p but-authz authorize_held_vs_missing
- TC-2 (-> AC-1, edge): Denial.remediation_hint is non-empty on a miss
    VERIFY: cargo test -p but-authz authorize_held_vs_missing
- TC-3 (-> AC-2, error): Unset AND empty-string BUT_AGENT_HANDLE -> Err, no default principal (T-AUTHZ-028)
    VERIFY: cargo test -p but-authz resolve_no_handle_rejected
- TC-4 (-> AC-3, error): Handle absent from config -> perm.denied, never allow (T-AUTHZ-027)
    VERIFY: cargo test -p but-authz resolve_unknown_principal_denied
- TC-5 (-> AC-4, happy_path): effective_authority(reviewer) includes group-inherited reviews:write; Merge still denied (T-GRPS-003)
    VERIFY: cargo test -p but-authz effective_authority_union
- TC-6 (-> AC-5, edge): authorize reads the effective set only from the committed GovConfig; no agent-supplied authority claim can widen it (T-AUTHZ-020)
    VERIFY: cargo test -p but-authz authority_only_from_config

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides: but_authz::authorize(principal, action, &GovConfig) -> Result<(), Denial>; but_authz::effective_authority(principal, &GovConfig) -> AuthoritySet; but_authz::resolve_principal(lookup, &GovConfig) -> Result<Principal, Denial>; but_authz::Denial constructors
consumes: but_authz::Authority; but_authz::AuthoritySet; but_authz::Principal; but_authz::Group; but_authz::config::GovConfig (AUTHZ-002)
boundary_contracts:
  - CAP-AUTHZ-01 hop-1/2/4: resolve BUT_AGENT_HANDLE → Principal (missing handle / unknown principal → fail closed, no anonymous action); authorize() returns Ok(()) or a structured Denial{code,message,remediation_hint} with exit-1 semantics

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-authz/src/authorize.rs (NEW)
  - crates/but-authz/src/lib.rs (MODIFY) — re-export authorize/effective_authority/resolve_principal/Denial constructors
  - crates/but-authz/tests/authorize.rs (NEW)
writeProhibited:
  - crates/but-authz/src/config.rs — owned by AUTHZ-002; consume GovConfig, don't re-read git
  - crates/but-authz/src/authority.rs — owned by AUTHZ-001
  - crates/but-api/** — wiring authorize into the action boundary is GATES-001 (commit) / later sprints (forge)
  - crates/but/** — CLI principal-injection wiring is downstream; this task provides the resolver only
  - Any file not listed in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but/src/utils/detect_agent.rs (lines 60-115)
   Focus: PRIMARY PATTERN — `detect()` (thin std::env wrapper) delegating to `detect_with(lookup: impl Fn(&str)->Option<OsString>)`. Mirror EXACTLY for `resolve_principal_from_env()` → `resolve_principal(lookup, cfg)`; the injected lookup is how tests set/unset BUT_AGENT_HANDLE without process-env mutation.
2. crates/but-error/src/lib.rs (lines 142-241)
   Focus: Code/Context shape for building the Denial contract; Denial.code is a &'static str ("perm.denied"), message + remediation_hint owned Strings.
3. crates/but-authz/src/config.rs (lines 1-60)
   Focus: GovConfig shape (principals→AuthoritySet, groups) produced by AUTHZ-002 — authorize/effective_authority read from it; do not re-read git.
4. crates/but-testsupport/src/lib.rs (lines 432-499)
   Focus: Seed a real committed config to load a GovConfig for the integration tests (no temp_dir().join).
5. /Users/justinrich/Projects/brain/docs/rust/ownership-borrowing.md (lines 1-90)
   Focus: Option/Result handling: `lookup(...).filter(|v| !v.is_empty()).ok_or(Denial::...)?` — the missing-handle fail-closed path; no `if (!val)` reflex.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- Integration tests pass: `cargo test -p but-authz authorize`  -> Exit 0; AC-1..4 green
- Crate compiles: `cargo check -p but-authz --all-targets`  -> Exit 0
- No role-preset branch in the check (functional-only): `! grep -rEn '== "(read|triage|write|maintain|admin)"|"(read|triage|write|maintain|admin)" *=>|from_role\(' crates/but-authz/src/authorize.rs`  -> No matches — authorize tests an Authority, never a role-preset name as a branch (AUTHZ-007 confirms cross-crate)
- Clippy clean: `cargo clippy -p but-authz --all-targets`  -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references: crates/but/src/utils/detect_agent.rs:60 (injected-lookup env resolution); 04-api-design.md core authorization API (authorize signature, Denial struct, identity resolution); 08-capability-chains.md CAP-AUTHZ-01 hop table + failure modes
notes:
  - authorize(principal: &Principal, action: Authority, cfg: &GovConfig) -> Result<(), Denial>: compute effective_authority(principal, cfg), `.contains(action)` → Ok, else build the Denial. Pure, borrows everything.
  - resolve_principal(lookup: impl Fn(&str)->Option<OsString>, cfg: &GovConfig) -> Result<Principal, Denial>: read BUT_AGENT_HANDLE via lookup; None/empty → Denial (no anonymous); Some(handle) absent from cfg → Denial (unknown principal); else build Principal from the cfg entry.
  - resolve_principal_from_env(cfg) is the thin wrapper passing |k| std::env::var_os(k); add `BUT_AGENT_HANDLE` as a const in but-authz (not but/src/envs.rs, which is CLI-only).
  - Denial constructors: `Denial::missing_permission(missing: Authority, held: &AuthoritySet)`, `Denial::no_handle()`, `Denial::unknown_principal(handle)` — each sets code="perm.denied" and a concrete message + hint.
pattern: Pure decision function over borrowed config + injected-lookup identity resolver; fail-closed via `ok_or(Denial::...)?`, never a default branch.
pattern_source: crates/but/src/utils/detect_agent.rs:60-115
anti_pattern: `authorize` returning `Ok(())` unconditionally (the textbook fail-open stub the negative control names), or resolving a default/anonymous principal when the handle is unset.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — Combines pure decision logic (`authorize`) with testable env resolution (mirroring `detect_agent`'s parameterized lookup) and fail-closed error construction — rust-implementer's wheelhouse: `Result`/`Option` discipline, no anonymous default, typed Denial.
reviewer: rust-reviewer
coding_standards: crates/AGENTS.md, /Users/justinrich/Projects/brain/docs/rust/ownership-borrowing.md, /Users/justinrich/Projects/brain/docs/rust/error-handling.md

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: AUTHZ-001, AUTHZ-002
Blocks:     GATES-001
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "AUTHZ-003",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "principals_cfg": {
      "description": "A GovConfig loaded via AUTHZ-002 from a but-testsupport repo whose committed permissions.toml defines `dev`=contents:write, `ro`=contents:read, and a `code-reviewers` group=reviews:write with `reviewer` as a member (no direct grants). [seeded via but_testsupport::writable_scenario(name) + invoke_bash git commit]",
      "seed_method": "cli",
      "records": [
        "let (repo, _tmp) = but_testsupport::writable_scenario(\"governance-base\");",
        "invoke_bash: write .gitbutler/permissions.toml with [[principal]] id=\"dev\" permissions=[\"contents:write\"]; [[principal]] id=\"ro\" permissions=[\"contents:read\"]; [[group]] name=\"code-reviewers\" permissions=[\"reviews:write\"]; [[principal]] id=\"reviewer\" groups=[\"code-reviewers\"]",
        "invoke_bash: git add -A && git commit -m \"principals\" (commit at refs/heads/main); then load via but_authz::config::load_governance_config(&repo, \"refs/heads/main\")"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN a loaded GovConfig WHEN authorize(dev, ContentsWrite) and authorize(ro, ContentsWrite) are called THEN dev is Ok and ro is Err perm.denied naming contents:write with a non-empty remediation_hint",
      "verify": "cargo test -p but-authz authorize_held_vs_missing",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-authz + committed GovConfig",
        "negative_control": {
          "would_fail_if": [
            "authorize always returns Ok (the canonical fail-open stub)",
            "Denial message is empty / does not name the missing permission",
            "remediation_hint is empty"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "principals_cfg",
            "action": {
              "actor": "api_client",
              "steps": [
                "authorize(dev, ContentsWrite, cfg)",
                "assert Ok(())"
              ]
            },
            "end_state": {
              "must_observe": [
                "`authorize(dev, ContentsWrite, cfg)` returns `Ok(())`"
              ],
              "must_not_observe": [
                "`Err(`",
                "code `\"perm.denied\"`",
                "no Denial returned"
              ]
            }
          },
          {
            "start_ref": "principals_cfg",
            "action": {
              "actor": "api_client",
              "steps": [
                "authorize(ro, ContentsWrite, cfg)",
                "assert Err(Denial)",
                "assert code==perm.denied",
                "assert message contains \"contents:write\"",
                "assert remediation_hint non-empty"
              ]
            },
            "end_state": {
              "must_observe": [
                "`code == \"perm.denied\"`",
                "`message` contains `\"contents:write\"`",
                "`remediation_hint` contains `\"reviewed merge\"`"
              ],
              "must_not_observe": [
                "`Ok(())`",
                "empty `remediation_hint`"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "GIVEN an injected lookup with BUT_AGENT_HANDLE unset or empty WHEN resolve_principal is called THEN Err(Denial), no anonymous/default principal",
      "verify": "cargo test -p but-authz resolve_no_handle_rejected",
      "maps_to_ac": null,
      "primary": false,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-authz + GovConfig",
        "negative_control": {
          "would_fail_if": [
            "resolver falls back to a default/anonymous principal when the handle is unset",
            "resolver returns Ok with an empty handle",
            "resolver panics instead of returning a typed Denial",
            "resolver returns Ok with an empty handle (Some(\"\") accepted as a principal)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "principals_cfg",
            "action": {
              "actor": "api_client",
              "steps": [
                "resolve_principal(|_| None /* BUT_AGENT_HANDLE unset */, cfg)",
                "assert Err(Denial)"
              ]
            },
            "end_state": {
              "must_observe": [
                "`resolve_principal` returns `Err(Denial)`",
                "no principal resolved (`code == \"perm.denied\"`)"
              ],
              "must_not_observe": [
                "`Ok(Principal`",
                "default principal",
                "anonymous principal"
              ]
            }
          },
          {
            "start_ref": "principals_cfg",
            "action": {
              "actor": "api_client",
              "steps": [
                "resolve_principal(|k| (k==\"BUT_AGENT_HANDLE\").then(|| OsString::from(\"\")) /* Some(empty string) */, cfg)",
                "assert Err(Denial) — same rejection as unset"
              ]
            },
            "end_state": {
              "must_observe": [
                "`resolve_principal` returns `Err(Denial)` for an empty-string handle",
                "`code == \"perm.denied\"` (no principal resolved)"
              ],
              "must_not_observe": [
                "`Ok(Principal`",
                "default principal",
                "empty handle accepted (`Ok(`)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "GIVEN BUT_AGENT_HANDLE=ghost absent from config WHEN resolve_principal is called THEN perm.denied, never default-allow",
      "verify": "cargo test -p but-authz resolve_unknown_principal_denied",
      "maps_to_ac": null,
      "primary": false,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-authz + GovConfig",
        "negative_control": {
          "would_fail_if": [
            "an unknown handle is granted an implicit empty-but-allowed identity",
            "missing principal yields Ok",
            "resolver invents a principal from the handle string"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "principals_cfg",
            "action": {
              "actor": "api_client",
              "steps": [
                "resolve_principal(|k| (k==\"BUT_AGENT_HANDLE\").then(|| OsString::from(\"ghost\")), cfg)",
                "assert Err(Denial) with code perm.denied"
              ]
            },
            "end_state": {
              "must_observe": [
                "`code == \"perm.denied\"`",
                "principal `\"ghost\"` not found"
              ],
              "must_not_observe": [
                "`Ok(Principal`",
                "default-allow",
                "no Denial returned"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "description": "GIVEN reviewer with only group membership WHEN effective_authority/authorize is computed THEN reviews:write is inherited (Ok) and Merge denied",
      "verify": "cargo test -p but-authz effective_authority_union",
      "maps_to_ac": null,
      "primary": false,
      "scenario": {
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "real but-authz + GovConfig",
        "negative_control": {
          "would_fail_if": [
            "group grants are ignored so reviewer has an empty effective set",
            "effective_authority returns only direct grants",
            "authorize grants Merge it never inherited"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "principals_cfg",
            "action": {
              "actor": "api_client",
              "steps": [
                "authorize(reviewer, ReviewsWrite, cfg)",
                "assert Ok(())",
                "authorize(reviewer, Merge, cfg)",
                "assert Err perm.denied"
              ]
            },
            "end_state": {
              "must_observe": [
                "`authorize(reviewer, ReviewsWrite, cfg)` returns `Ok(())`",
                "`authorize(reviewer, Merge, cfg)` returns `code == \"perm.denied\"`"
              ],
              "must_not_observe": [
                "`ReviewsWrite` returns `\"perm.denied\"`",
                "`Merge` returns `Ok(())`",
                "empty effective set"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "description": "GIVEN ro holds only contents:read in committed config WHEN authorize is evaluated THEN ContentsWrite is denied and no caller-supplied claim can widen the set (authority only from cfg)",
      "verify": "cargo test -p but-authz authority_only_from_config",
      "maps_to_ac": null,
      "primary": false,
      "scenario": {
        "tier": "holdout",
        "test_tier": "integration",
        "verification_service": "real but-authz + GovConfig",
        "negative_control": {
          "would_fail_if": [
            "authorize accepts an agent-supplied AuthoritySet/claim argument that overrides the committed config",
            "ro is allowed ContentsWrite because a claim was honored",
            "effective set is read from the Principal struct's caller-set field instead of cfg",
            "authorize reads the effective set from a hardcoded/caller-supplied static field disconnected from the committed cfg"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "principals_cfg",
            "action": {
              "actor": "api_client",
              "steps": [
                "authorize(ro, ContentsWrite, cfg)",
                "assert Err perm.denied (the committed config grants ro only contents:read; no claim path exists to widen it)"
              ]
            },
            "end_state": {
              "must_observe": [
                "`authorize(ro, ContentsWrite, cfg)` returns `code == \"perm.denied\"` — authority sourced only from `cfg`"
              ],
              "must_not_observe": [
                "`Ok(())`",
                "ro granted via a caller-supplied claim",
                "default-allow"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "held vs missing authorize outcomes + perm.denied naming",
      "verify": "cargo test -p but-authz authorize_held_vs_missing",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "remediation_hint non-empty on miss",
      "verify": "cargo test -p but-authz authorize_held_vs_missing",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "unset and empty handle rejected, no default principal",
      "verify": "cargo test -p but-authz resolve_no_handle_rejected",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "unknown principal denied perm.denied",
      "verify": "cargo test -p but-authz resolve_unknown_principal_denied",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "effective set = own ∪ groups",
      "verify": "cargo test -p but-authz effective_authority_union",
      "maps_to_ac": "AC-4"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "description": "authority only from committed config, no agent claim (T-AUTHZ-020)",
      "verify": "cargo test -p but-authz authority_only_from_config",
      "maps_to_ac": "AC-5"
    }
  ]
}
-->
</details>
