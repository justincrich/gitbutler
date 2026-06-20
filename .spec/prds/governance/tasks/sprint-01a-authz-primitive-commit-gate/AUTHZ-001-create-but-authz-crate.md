# AUTHZ-001: Create `but-authz` crate: `Authority`, `AuthoritySet`, `Principal`, `Group`, `Denial`

## What this does

Stand up the NEW pure-logic `but-authz` crate with the functional permission catalog (`Authority` enum), the `AuthoritySet` operations (`parse`/`from_role` desugar/`union`/`contains`), the `Principal`/`Group` grantee types, and the `Denial` rejection contract — the type substrate every later AUTHZ task builds on.

## Why

Sprint 01a · PRD UC-AUTHZ-01 · capabilities CAP-AUTHZ-01. Part of the functional-permission governance walking skeleton (commit allow/deny through real `but-authz` + real git).

## How to verify

PRIMARY **AC-1** — `cargo test -p but-authz authority_parse` (unit). Full gate set in the spec below.

## Scope

- crates/but-authz/Cargo.toml (NEW)
- crates/but-authz/src/lib.rs (NEW)
- crates/but-authz/src/authority.rs (NEW)
- crates/but-authz/src/principal.rs (NEW)
- crates/but-authz/src/denial.rs (NEW)
- crates/but-authz/tests/authority.rs (NEW)
- Cargo.toml (MODIFY) — add `crates/but-authz` to [workspace.members] and a `but-authz = { path = "crates/but-authz" }` line to [workspace.dependencies]

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: AUTHZ-001 - Create `but-authz` crate: `Authority`, `AuthoritySet`, `Principal`, `Group`, `Denial`
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P0
EFFORT:     M  (180 min)
AGENT:      implementer=rust-implementer | reviewer=rust-reviewer
PROPOSED-BY: rust-planner
SPRINT:     ../SPRINT.md
PRD_REFS:   UC-AUTHZ-01
CAPABILITIES: CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  cargo test -p but-authz authority_parse
  check: cargo check -p but-authz --all-targets
  lint:  cargo clippy --all-targets   |   fmt: cargo fmt --check

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
`cargo test -p but-authz` is green: `Authority::parse("contents:write")` returns `ContentsWrite`, an unknown token errors, `AuthoritySet::from_role("write")` contains contents/reviews/pr/comments/statuses:write but NOT merge/administration:write, `from_role("admin")` contains every variant, `from_role("maintain")` contains merge + administration:read but NOT administration:write, and a raw list parses to the same set as the equivalent role. The crate compiles with zero `gix`/`std::fs`/`std::env` references.

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] MUST name the new authorization axis `Authority`/`AuthoritySet` in crate `but-authz` (module path `but_authz::Authority`) — orthogonal to GitButler's existing repo-access lock `Permission`; see 02-system-components.md naming-collision guardrail.
- [MUST] MUST keep this crate PURE — no I/O, no `gix`, no filesystem, no `std::env`; it only parses tokens/role strings already in memory (ref-pinned reads live in AUTHZ-002).
- [MUST] MUST make role desugar exact per 03-data-schema.md: `write` = read ∪ {contents:write, pull_requests:write, reviews:write, comments:write, statuses:write} and EXCLUDES `merge` and `administration:write`; `admin` = every Authority; `maintain` = write ∪ {merge, administration:read} EXCLUDING administration:write.
- [MUST] MUST expose the catalog as an iterable surface (`Authority::ALL` const slice or a strum `Authority::iter()`) so membership tests assert against the enum itself, never a hardcoded variant count.
- [NEVER] NEVER add a `role`/preset name field to `Authority` or `AuthoritySet` that enforcement could branch on — desugar produces a flat functional set and the preset name is discarded (enforcement is functional-only; grep-asserted in AUTHZ-007).
- [NEVER] NEVER `unwrap()`/`expect()`/`panic!` on a parse path in library code — an unknown token returns `Err(ParseAuthorityError)`.
- [STRICTLY] STRICTLY use `Result<T, E>` for fallible parse; an unknown permission token like `contents:bogus` is a typed error, not a silent default or skip.
- [STRICTLY] STRICTLY derive `serde::Deserialize` ONLY on the config wire types if needed; the `Authority` token parse is hand-rolled from the `name:scope` string form, not free serde, so unknown tokens fail closed.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: Token parses to typed Authority; unknown token errors [PRIMARY]
- [ ] AC-2: `write` role desugars excluding merge and administration:write
- [ ] AC-3: `admin` desugars to superuser; `maintain` includes merge but excludes administration:write
- [ ] AC-4: Raw functional list loads without a role and equals the equivalent role set
- [ ] All verification gates pass; only write_allowed files modified (git diff --name-only)

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (TDD beads — happy-path first)
--------------------------------------------------------------------------------

AC-1: Token parses to typed Authority; unknown token errors [PRIMARY] [PRIMARY]
  GIVEN: the in-memory functional catalog (no repo, no config files)
  WHEN:  `Authority::parse` is called on a valid token and on an unknown token
  THEN:  the valid token returns the exact typed variant and the unknown token returns `Err(ParseAuthorityError)`, never a default-allow
  TEST_TIER: unit   VERIFICATION_SERVICE: but-authz pure logic (zero I/O)
  UNIT_TEST_JUSTIFIED: UNIT_TEST_JUSTIFIED: pure string→enum parse with zero I/O, no network, no filesystem, no git — the realism ladder's library/pure-logic row; the negative control (unknown token must error) bites without any external service.
  VERIFY: cargo test -p but-authz authority_parse
  SCENARIO (tier=visible, test_tier=unit):
    NEGATIVE_CONTROL would fail if: parse is a stub returning a constant `Authority::ContentsRead`; parse default-allows an unknown token instead of erroring; the Err arm is a hardcoded static `Ok(ContentsRead)` shell
    EVIDENCE: stdout (required_capture=True)
    case[0] (api_client): call Authority::parse("contents:write"); assert == Authority::ContentsWrite
      MUST_OBSERVE:     ['`Authority::parse("contents:write")` returns `Ok(Authority::ContentsWrite)`']
      MUST_NOT_OBSERVE: ['`Err`', '`Authority::ContentsRead`', 'default Authority']
    case[1] (api_client): call Authority::parse("administration:write"); assert == Authority::AdministrationWrite
      MUST_OBSERVE:     ['`Authority::parse("administration:write")` returns `Ok(Authority::AdministrationWrite)`']
      MUST_NOT_OBSERVE: ['`Err`', '`Ok(ContentsRead)`', 'default Authority']
    case[2] (api_client): call Authority::parse("contents:bogus"); assert is Err(ParseAuthorityError)
      MUST_OBSERVE:     ['`Authority::parse("contents:bogus")` returns `Err(ParseAuthorityError)`', 'error names `"contents:bogus"`']
      MUST_NOT_OBSERVE: ['`Ok(`', 'default Authority']

AC-2: `write` role desugars excluding merge and administration:write
  GIVEN: the in-memory catalog and the role-desugar table
  WHEN:  `AuthoritySet::from_role("write")` is computed
  THEN:  the set contains contents:write, reviews:write, pull_requests:write (and the read base) but does NOT contain merge or administration:write
  TEST_TIER: unit   VERIFICATION_SERVICE: but-authz pure logic (zero I/O)
  UNIT_TEST_JUSTIFIED: UNIT_TEST_JUSTIFIED: role→set desugar is a pure in-memory transform (no I/O); the exclusion assertion (merge/admin:write absent) is the non-degenerate proof and bites against a desugar that grants everything.
  VERIFY: cargo test -p but-authz desugar_write_excludes_merge_admin
  SCENARIO (tier=visible, test_tier=unit):
    NEGATIVE_CONTROL would fail if: from_role returns the admin superset for every role (a hardcoded constant set); from_role returns an empty set (degenerate stub); exclusion of merge is omitted so write silently includes `Authority::Merge`
    EVIDENCE: stdout (required_capture=True)
    case[0] (api_client): compute AuthoritySet::from_role("write"); assert contains ContentsWrite, ReviewsWrite, PullRequestsWrite; assert NOT contains Merge, AdministrationWrite
      MUST_OBSERVE:     ['set contains `Authority::ContentsWrite`', 'set contains `Authority::ReviewsWrite`', 'set contains `Authority::PullRequestsWrite`', 'set excludes `Authority::Merge`', 'set excludes `Authority::AdministrationWrite`']
      MUST_NOT_OBSERVE: ['set contains `Authority::Merge`', 'empty set', 'set contains `Authority::AdministrationWrite`']

AC-3: `admin` desugars to superuser; `maintain` includes merge but excludes administration:write
  GIVEN: the in-memory catalog and the role-desugar table
  WHEN:  `AuthoritySet::from_role("admin")` and `from_role("maintain")` are computed
  THEN:  admin contains every Authority variant including merge and administration:write; maintain contains merge and administration:read but NOT administration:write
  TEST_TIER: unit   VERIFICATION_SERVICE: but-authz pure logic (zero I/O)
  UNIT_TEST_JUSTIFIED: UNIT_TEST_JUSTIFIED: pure desugar transform; admin-superset and maintain's precise merge-yes/admin:write-no boundary are non-degenerate membership assertions with zero I/O.
  VERIFY: cargo test -p but-authz desugar_admin_superuser_and_maintain
  SCENARIO (tier=visible, test_tier=unit):
    NEGATIVE_CONTROL would fail if: admin desugar omits any variant (not truly superuser); maintain silently includes `Authority::AdministrationWrite`; maintain omits `Authority::Merge`
    EVIDENCE: stdout (required_capture=True)
    case[0] (api_client): compute from_role("admin"); assert contains EVERY Authority variant by iterating Authority::ALL (or strum Authority::iter()), incl. Merge and AdministrationWrite
      MUST_OBSERVE:     ['set contains `Authority::Merge`', 'set contains `Authority::AdministrationWrite`', 'EVERY `Authority` variant is present — assert by iterating the catalog (`Authority::ALL` or strum `Authority::iter()`) and checking `set.contains(v)` for each variant, never a hardcoded count', 'admin set length `==` `Authority::ALL` length (tied to the enum, not a literal `12`)']
      MUST_NOT_OBSERVE: ['set excludes `Authority::Merge`', 'empty set']
    case[1] (api_client): compute from_role("maintain"); assert contains Merge and AdministrationRead; assert NOT contains AdministrationWrite
      MUST_OBSERVE:     ['set contains `Authority::Merge`', 'set contains `Authority::AdministrationRead`', 'set excludes `Authority::AdministrationWrite`']
      MUST_NOT_OBSERVE: ['set contains `Authority::AdministrationWrite`', 'no `Authority::Merge` in the set', 'empty set']

AC-4: Raw functional list loads without a role and equals the equivalent role set
  GIVEN: the in-memory catalog
  WHEN:  a raw list `["contents:write","reviews:write"]` is parsed and a `role="write"` entry is desugared
  THEN:  the raw list loads to exactly `{ContentsWrite, ReviewsWrite}` (no role required), and enforcement only ever sees an `AuthoritySet` regardless of which form produced it
  TEST_TIER: unit   VERIFICATION_SERVICE: but-authz pure logic (zero I/O)
  UNIT_TEST_JUSTIFIED: UNIT_TEST_JUSTIFIED: parsing a token list to a set is pure in-memory logic; the non-degenerate assertion is the exact 2-element membership, proving list-without-role and that both forms yield an AuthoritySet.
  VERIFY: cargo test -p but-authz list_loads_without_role
  SCENARIO (tier=visible, test_tier=unit):
    NEGATIVE_CONTROL would fail if: list parse requires a role and errors with an empty set without one; list parse returns an empty set (degenerate stub); list parse silently swallows an unknown token instead of erroring (no-op error path)
    EVIDENCE: stdout (required_capture=True)
    case[0] (api_client): AuthoritySet::parse(["contents:write","reviews:write"]); assert == {ContentsWrite, ReviewsWrite}; assert no role needed
      MUST_OBSERVE:     ['set contains `Authority::ContentsWrite`', 'set contains `Authority::ReviewsWrite`', 'set length `== 2`']
      MUST_NOT_OBSERVE: ['empty set', 'no role accepted (`Err(role required)`)']
    case[1] (api_client): compute AuthoritySet::parse(["metadata:read","contents:read","pull_requests:read","contents:write","pull_requests:write","reviews:write","comments:write","statuses:write"]); compute AuthoritySet::from_role("write"); assert the two sets are EQUAL
      MUST_OBSERVE:     ['`AuthoritySet::parse(<full write token list>)` `==` `AuthoritySet::from_role("write")` (role and explicit list resolve IDENTICALLY)']
      MUST_NOT_OBSERVE: ['the two sets differ', 'empty set', '`from_role` returns a superset (`admin`) for `write`']

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1, happy_path): Authority::parse("merge") == Authority::Merge (catalog completeness)
    VERIFY: cargo test -p but-authz authority_parse
- TC-2 (-> AC-1, error): Authority::parse("contents:bogus") is Err(ParseAuthorityError) — unknown token fails closed, never default-allow
    VERIFY: cargo test -p but-authz authority_parse_unknown_errors
- TC-3 (-> AC-2, edge): from_role("write") excludes Merge AND AdministrationWrite
    VERIFY: cargo test -p but-authz desugar_write_excludes_merge_admin
- TC-4 (-> AC-3, happy_path): from_role("admin") contains every Authority variant (asserted by iterating Authority::ALL, not a literal count)
    VERIFY: cargo test -p but-authz desugar_admin_superuser_and_maintain
- TC-5 (-> AC-3, edge): from_role("maintain") contains Merge + AdministrationRead but NOT AdministrationWrite (T-AUTHZ-024)
    VERIFY: cargo test -p but-authz desugar_admin_superuser_and_maintain
- TC-6 (-> AC-4, happy_path): AuthoritySet::parse(list) and from_role yield equal sets for equivalent inputs; list needs no role (T-AUTHZ-005/006)
    VERIFY: cargo test -p but-authz list_loads_without_role
- TC-7 (-> AC-4, edge): AuthoritySet::parse(<full write token list>) == AuthoritySet::from_role("write") — list and role resolve identically (T-AUTHZ-005/006)
    VERIFY: cargo test -p but-authz list_equals_role_write

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides: but_authz::Authority; but_authz::AuthoritySet; but_authz::Principal; but_authz::Group; but_authz::Denial; but_authz::ParseAuthorityError
consumes: n/a
boundary_contracts:
  - CAP-AUTHZ-01 hop-2/3 type substrate: the typed Authority axis the action wrapper and config loader operate on (role desugar is CONFIG-layer ergonomics, never seen by enforcement)

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - crates/but-authz/Cargo.toml (NEW)
  - crates/but-authz/src/lib.rs (NEW)
  - crates/but-authz/src/authority.rs (NEW)
  - crates/but-authz/src/principal.rs (NEW)
  - crates/but-authz/src/denial.rs (NEW)
  - crates/but-authz/tests/authority.rs (NEW)
  - Cargo.toml (MODIFY) — add `crates/but-authz` to [workspace.members] and a `but-authz = { path = "crates/but-authz" }` line to [workspace.dependencies]
writeProhibited:
  - crates/but-error/src/lib.rs — do NOT add Code variants here; the Denial code is a &'static str owned by but-authz (the enum note forbids unused Code variants)
  - crates/but-api/** — the enforcement seam is AUTHZ-003/GATES-001, not this task
  - crates/but-workspace/** — the commit gate is GATES-001
  - any `gitbutler-*` crate — no new legacy usage (crates/AGENTS.md)
  - Any file not listed in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. crates/but-error/src/lib.rs (lines 132-241)
   Focus: PRIMARY PATTERN — how GitButler models a typed `Code` enum with `Display`(=Debug variant name) and a `Context{code,message}` struct; mirror this shape for `Authority`/`Denial` (Denial.code is a &'static str like the wire codes). NOTE the doc rule: only add Code variants a consumer uses — do NOT touch Code here.
2. crates/but/src/utils/detect_agent.rs (lines 10-53)
   Focus: Idiomatic enum-with-`name()`/`Display`/exhaustive-`match` and a `#[derive(Debug,Clone,Copy,PartialEq,Eq)]` catalog enum — the exact shape for `Authority`'s variant catalog and `name()`/parse roundtrip.
3. /Users/justinrich/Projects/brain/docs/rust/traits-generics.md (lines 1-80)
   Focus: `#[derive(Serialize,Deserialize)]` on config wire types + enum modeling vs class inheritance; use a flat enum + a set, never a role-keyed hierarchy.
4. /Users/justinrich/Projects/brain/docs/rust/error-handling.md (lines 1-90)
   Focus: `Result<T,E>` + typed error enums (`thiserror`/hand-rolled) for the parse path; unknown token → typed Err, never panic/default.
5. /Users/justinrich/Projects/brain/docs/TDD-METHODOLOGY.md (lines 1-60)
   Focus: RED→GREEN→REFACTOR per AC; write the failing membership assertion first.

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- Unit tests pass: `cargo test -p but-authz`  -> Exit 0; all AC tests green
- Crate compiles in workspace: `cargo check -p but-authz --all-targets`  -> Exit 0
- Clippy clean: `cargo clippy -p but-authz --all-targets`  -> Exit 0, no warnings
- Formatting: `cargo fmt --check`  -> Exit 0
- Purity (no I/O leaked into the pure substrate): `! grep -rEn 'gix|std::fs|std::env|std::process' crates/but-authz/src/authority.rs crates/but-authz/src/principal.rs crates/but-authz/src/denial.rs`  -> No matches (exit nonzero from grep) — the Authority/AuthoritySet/Principal/Group/Denial substrate is pure (lib.rs is a thin re-export). NOTE: the ref-pinned loader (`config.rs`, AUTHZ-002, uses `gix`) and env handle resolution (`authorize.rs`, AUTHZ-003, uses `std::env`) legitimately share the crate; the purity invariant applies to the Authority/AuthoritySet/Principal/Denial substrate, and the enforced version is AUTHZ-007's file-scoped `invariant_build_gates`.

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references: crates/but-error/src/lib.rs:142 (Code enum shape + Display=variant-name); crates/but/src/utils/detect_agent.rs:10 (catalog enum + name()/match); 03-data-schema.md New Rust types table + Role desugar table (the exact catalog + expansions)
notes:
  - Authority: a flat `#[non_exhaustive]`-free enum of the 12 catalog variants. AuthoritySet wraps a `BTreeSet<Authority>` (deterministic ordering for snapshot/equality).
  - from_role discards the preset name AFTER expansion — the returned AuthoritySet carries NO role tag, so nothing downstream can branch on a role (AUTHZ-007 greps for this).
  - Denial { code: &'static str, message: String, remediation_hint: String } — code is "perm.denied" here; construction helpers belong in AUTHZ-003.
pattern: Newtype-over-set + exhaustive enum catalog; pure functions (`parse`, `from_role`, `union`, `contains`) — no `self`-mutating I/O.
pattern_source: crates/but-error/src/lib.rs:142-241
anti_pattern: A `Permission`/`Role` enum that enforcement matches by name (overloads GitButler's repo-lock `Permission` and re-introduces role-keyed enforcement) — forbidden by 02-system-components.md and AUTHZ-007.

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: rust-implementer — Pure-Rust type/parse/desugar logic in a NEW crate — idiomatic enum modeling, exhaustive `match`, no-I/O parsing, `thiserror`-style typed errors. This is rust-implementer's core competency (TDD on pure logic, ownership of a new crate boundary).
reviewer: rust-reviewer
coding_standards: crates/AGENTS.md, /Users/justinrich/Projects/brain/docs/rust/README.md, /Users/justinrich/Projects/brain/docs/rust/error-handling.md

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: none
Blocks:     AUTHZ-002, AUTHZ-003, GATES-001, AUTHZ-007
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "AUTHZ-001",
  "proposed_by": "rust-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "authority_catalog": {
      "description": "The in-memory MVP functional catalog — no repo, no files; representative tokens and role names parsed/desugared directly.",
      "seed_method": "public_api",
      "records": [
        "token \"contents:write\" -> Authority::ContentsWrite",
        "token \"merge\" -> Authority::Merge",
        "token \"administration:write\" -> Authority::AdministrationWrite",
        "role \"write\" -> {metadata:read, contents:read, pull_requests:read, contents:write, pull_requests:write, reviews:write, comments:write, statuses:write}",
        "role \"admin\" -> every Authority variant (superuser)",
        "role \"maintain\" -> write-set ∪ {merge, administration:read}",
        "raw list [\"contents:write\",\"reviews:write\"] -> {ContentsWrite, ReviewsWrite}"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN the in-memory functional catalog WHEN Authority::parse is called on a valid and an unknown token THEN the valid token returns the typed variant and the unknown token returns Err(ParseAuthorityError), never default-allow",
      "verify": "cargo test -p but-authz authority_parse",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "unit",
        "verification_service": "but-authz pure logic",
        "unit_test_justified": "Pure string→enum parse with zero I/O (no network, filesystem, or git); the negative control (`Authority::parse` on an unknown token must return `Err(ParseAuthorityError)`) bites without any external service.",
        "negative_control": {
          "would_fail_if": [
            "parse is a stub returning a constant `Authority::ContentsRead`",
            "parse default-allows an unknown token instead of erroring",
            "the Err arm is a hardcoded static `Ok(ContentsRead)` shell"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "authority_catalog",
            "action": {
              "actor": "api_client",
              "steps": [
                "call Authority::parse(\"contents:write\")",
                "assert == Authority::ContentsWrite"
              ]
            },
            "end_state": {
              "must_observe": [
                "`Authority::parse(\"contents:write\")` returns `Ok(Authority::ContentsWrite)`"
              ],
              "must_not_observe": [
                "`Err`",
                "`Authority::ContentsRead`",
                "default Authority"
              ]
            }
          },
          {
            "start_ref": "authority_catalog",
            "action": {
              "actor": "api_client",
              "steps": [
                "call Authority::parse(\"administration:write\")",
                "assert == Authority::AdministrationWrite"
              ]
            },
            "end_state": {
              "must_observe": [
                "`Authority::parse(\"administration:write\")` returns `Ok(Authority::AdministrationWrite)`"
              ],
              "must_not_observe": [
                "`Err`",
                "`Ok(ContentsRead)`",
                "default Authority"
              ]
            }
          },
          {
            "start_ref": "authority_catalog",
            "action": {
              "actor": "api_client",
              "steps": [
                "call Authority::parse(\"contents:bogus\")",
                "assert is Err(ParseAuthorityError)"
              ]
            },
            "end_state": {
              "must_observe": [
                "`Authority::parse(\"contents:bogus\")` returns `Err(ParseAuthorityError)`",
                "error names `\"contents:bogus\"`"
              ],
              "must_not_observe": [
                "`Ok(`",
                "default Authority"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "GIVEN the role-desugar table WHEN from_role(\"write\") is computed THEN the set contains contents/reviews/pr:write but NOT merge or administration:write",
      "verify": "cargo test -p but-authz desugar_write_excludes_merge_admin",
      "maps_to_ac": null,
      "primary": false,
      "scenario": {
        "tier": "visible",
        "test_tier": "unit",
        "verification_service": "but-authz pure logic",
        "unit_test_justified": "Role→set desugar is a pure in-memory transform with zero I/O; the exclusion assertion (`Authority::Merge`/`Authority::AdministrationWrite` absent) is the non-degenerate proof and needs no external service.",
        "negative_control": {
          "would_fail_if": [
            "from_role returns the admin superset for every role (a hardcoded constant set)",
            "from_role returns an empty set (degenerate stub)",
            "exclusion of merge is omitted so write silently includes `Authority::Merge`"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "authority_catalog",
            "action": {
              "actor": "api_client",
              "steps": [
                "compute AuthoritySet::from_role(\"write\")",
                "assert contains ContentsWrite, ReviewsWrite, PullRequestsWrite",
                "assert NOT contains Merge, AdministrationWrite"
              ]
            },
            "end_state": {
              "must_observe": [
                "set contains `Authority::ContentsWrite`",
                "set contains `Authority::ReviewsWrite`",
                "set contains `Authority::PullRequestsWrite`",
                "set excludes `Authority::Merge`",
                "set excludes `Authority::AdministrationWrite`"
              ],
              "must_not_observe": [
                "set contains `Authority::Merge`",
                "empty set",
                "set contains `Authority::AdministrationWrite`"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "description": "GIVEN the role-desugar table WHEN from_role(\"admin\") and from_role(\"maintain\") are computed THEN admin is superuser and maintain has merge+admin:read but NOT admin:write",
      "verify": "cargo test -p but-authz desugar_admin_superuser_and_maintain",
      "maps_to_ac": null,
      "primary": false,
      "scenario": {
        "tier": "visible",
        "test_tier": "unit",
        "verification_service": "but-authz pure logic",
        "unit_test_justified": "Pure desugar transform with zero I/O; admin-superset and maintain's precise merge-yes/admin:write-no boundary are non-degenerate membership assertions needing no external service.",
        "negative_control": {
          "would_fail_if": [
            "admin desugar omits any variant (not truly superuser)",
            "maintain silently includes `Authority::AdministrationWrite`",
            "maintain omits `Authority::Merge`"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "authority_catalog",
            "action": {
              "actor": "api_client",
              "steps": [
                "compute from_role(\"admin\")",
                "assert contains EVERY Authority variant by iterating Authority::ALL (or strum Authority::iter()), incl. Merge and AdministrationWrite"
              ]
            },
            "end_state": {
              "must_observe": [
                "set contains `Authority::Merge`",
                "set contains `Authority::AdministrationWrite`",
                "EVERY `Authority` variant is present — assert by iterating the catalog (`Authority::ALL` or strum `Authority::iter()`) and checking `set.contains(v)` for each variant, never a hardcoded count",
                "admin set length `==` `Authority::ALL` length (tied to the enum, not a literal `12`)"
              ],
              "must_not_observe": [
                "set excludes `Authority::Merge`",
                "empty set"
              ]
            }
          },
          {
            "start_ref": "authority_catalog",
            "action": {
              "actor": "api_client",
              "steps": [
                "compute from_role(\"maintain\")",
                "assert contains Merge and AdministrationRead",
                "assert NOT contains AdministrationWrite"
              ]
            },
            "end_state": {
              "must_observe": [
                "set contains `Authority::Merge`",
                "set contains `Authority::AdministrationRead`",
                "set excludes `Authority::AdministrationWrite`"
              ],
              "must_not_observe": [
                "set contains `Authority::AdministrationWrite`",
                "no `Authority::Merge` in the set",
                "empty set"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "description": "GIVEN the catalog WHEN a raw functional list is parsed THEN it loads without a role to the exact set, and both list and role forms yield an AuthoritySet",
      "verify": "cargo test -p but-authz list_loads_without_role",
      "maps_to_ac": null,
      "primary": false,
      "scenario": {
        "tier": "visible",
        "test_tier": "unit",
        "verification_service": "but-authz pure logic",
        "unit_test_justified": "Parsing a token list to a set is pure in-memory logic with zero I/O; the non-degenerate assertion is the exact 2-element membership, needing no external service.",
        "negative_control": {
          "would_fail_if": [
            "list parse requires a role and errors with an empty set without one",
            "list parse returns an empty set (degenerate stub)",
            "list parse silently swallows an unknown token instead of erroring (no-op error path)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "authority_catalog",
            "action": {
              "actor": "api_client",
              "steps": [
                "AuthoritySet::parse([\"contents:write\",\"reviews:write\"])",
                "assert == {ContentsWrite, ReviewsWrite}",
                "assert no role needed"
              ]
            },
            "end_state": {
              "must_observe": [
                "set contains `Authority::ContentsWrite`",
                "set contains `Authority::ReviewsWrite`",
                "set length `== 2`"
              ],
              "must_not_observe": [
                "empty set",
                "no role accepted (`Err(role required)`)"
              ]
            }
          },
          {
            "start_ref": "authority_catalog",
            "action": {
              "actor": "api_client",
              "steps": [
                "compute AuthoritySet::parse([\"metadata:read\",\"contents:read\",\"pull_requests:read\",\"contents:write\",\"pull_requests:write\",\"reviews:write\",\"comments:write\",\"statuses:write\"])",
                "compute AuthoritySet::from_role(\"write\")",
                "assert the two sets are EQUAL"
              ]
            },
            "end_state": {
              "must_observe": [
                "`AuthoritySet::parse(<full write token list>)` `==` `AuthoritySet::from_role(\"write\")` (role and explicit list resolve IDENTICALLY)"
              ],
              "must_not_observe": [
                "the two sets differ",
                "empty set",
                "`from_role` returns a superset (`admin`) for `write`"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "Authority::parse(\"merge\") == Authority::Merge",
      "verify": "cargo test -p but-authz authority_parse",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "Authority::parse unknown token is Err",
      "verify": "cargo test -p but-authz authority_parse_unknown_errors",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "from_role(\"write\") excludes Merge and AdministrationWrite",
      "verify": "cargo test -p but-authz desugar_write_excludes_merge_admin",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "from_role(\"admin\") contains every Authority variant (iterated over Authority::ALL, not a literal count)",
      "verify": "cargo test -p but-authz desugar_admin_superuser_and_maintain",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "from_role(\"maintain\") has merge+admin:read, not admin:write",
      "verify": "cargo test -p but-authz desugar_admin_superuser_and_maintain",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "description": "list and role forms resolve identically; list needs no role",
      "verify": "cargo test -p but-authz list_loads_without_role",
      "maps_to_ac": "AC-4"
    },
    {
      "id": "TC-7",
      "type": "test_criterion",
      "description": "AuthoritySet::parse(<full write token list>) == AuthoritySet::from_role(\"write\") — list and role resolve identically (T-AUTHZ-005/006)",
      "verify": "cargo test -p but-authz list_equals_role_write",
      "maps_to_ac": "AC-4"
    }
  ]
}
-->
</details>
