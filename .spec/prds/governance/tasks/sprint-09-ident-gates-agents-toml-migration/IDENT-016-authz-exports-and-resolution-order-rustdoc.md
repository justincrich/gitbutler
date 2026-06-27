# IDENT-016 — `crates/but-authz/src/lib.rs` + `src/authorize.rs` doc-comments — export `agents_path`, verify `Registry`/`resolve_principal_with_registry` exports, rustdoc the resolution order

**Sprint:** [Sprint 09](./SPRINT.md) · **Agent:** `rust-implementer` · **Estimate:** 60 min · **Type:** FEATURE (DOCS) · **Status:** READY · **Proposed By:** rust-planner

## Background

The crate root of `but_authz` needs to expose the new `agents_path()` function (added by IDENT-009) alongside the existing `permissions_path()` re-export. Sprint 08 already exports `Registry` (IDENT-001) and `resolve_principal_with_registry` (IDENT-003); this task verifies those exports are present (re-add only if missing) and adds rustdoc to `authorize.rs` documenting the 3-step resolution order.

**Why it matters.** Callers (`but-api`, the `but agent` CLI, the `but-init`/`but-migrate` skills in brain) need `but_authz::agents_path()` from the crate root. The rustdoc on `resolve_principal_with_registry` is the canonical human-readable contract for the resolution order — referenced by `RULES.md` Agent identity subsection (Sprint 11 deliverable).

**Current state.** `crates/but-authz/src/lib.rs:13-16` has the existing `pub use config::{... permissions_path};` block. Sprint 08's IDENT-001 exports `Registry` (confirmed by previous dispatch reading the file). `resolve_principal_with_registry` is added by Sprint 08's IDENT-003.

**Desired state.** `pub use config::agents_path;` added; `Registry` and `resolve_principal_with_registry` verified at crate root; rustdoc on `resolve_principal_with_registry` (and/or a module-level section) documenting the 3-step order.

## Critical Constraints

- **MUST** add exactly one `pub use` line to `lib.rs`: `pub use config::agents_path;` (mirroring the existing `pub use config::{... permissions_path};` at `lib.rs:13-16`).
- **MUST** have the `authorize.rs` rustdoc name all three resolution steps in order: (1) registry hit → principal, (2) registry miss + `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` → env fallback, (3) else → `Denial::unregistered`.
- **MUST** have the rustdoc name the flag (`BUT_AUTHZ_ALLOW_ENV_HANDLE`) and the denial variant (`Denial::unregistered`) verbatim — these are the keywords the AC-1 grep asserts on.
- **MUST** verify (grep) that `but_authz::Registry` and `but_authz::resolve_principal_with_registry` resolve at the crate root; re-add the `pub use` only if missing.
- **NEVER** change any function signature, body, visibility, or error variant in `authorize.rs` or any other `src` file (doc-comments ONLY).
- **NEVER** add a new module, trait, struct, or function (this is a docs + re-export task, not a feature task).
- **NEVER** rewrite existing rustdoc on `resolve_principal` / `resolve_principal_from_env` / `authorize` — only the `resolve_principal_with_registry` doc (or a module-level doc section) is added.
- **NEVER** use `cargo nextest` (not in this repo's toolchain — use `cargo test`/`cargo doc` per `crates/AGENTS.md`).
- **STRICTLY** the behavior-preserving property is verified by `cargo test -p but-authz` passing unchanged — if any existing test breaks, the task has crossed the doc-only line and must back out the source change.
- **STRICTLY** the PRIMARY AC proof is `cargo doc -p but-authz --no-deps` exit 0 AND the rendered rustdoc HTML/text containing the resolution-order keywords — there is no behavioral integration test for doc-comment content, which is why AC-1 carries a `UNIT_TEST_JUSTIFIED` annotation.

## Specification

**Objective:** (1) Add `pub use config::agents_path;` to `crates/but-authz/src/lib.rs`. (2) Verify `Registry` and `resolve_principal_with_registry` are exported (re-add if missing). (3) Add rustdoc to `crates/but-authz/src/authorize.rs` documenting the 3-step resolution order used by `resolve_principal_with_registry`.

**Success state:** `cargo doc -p but-authz --no-deps` exits 0. The rendered rustdoc for `resolve_principal_with_registry` contains the keywords `registry hit`, `BUT_AUTHZ_ALLOW_ENV_HANDLE`, and `Denial::unregistered`. `but_authz::agents_path()`, `but_authz::Registry`, and `but_authz::resolve_principal_with_registry` all resolve at the crate root (proven by a doctest). `cargo test -p but-authz` passes unchanged (behavior byte-identical).

## Acceptance Criteria

**AC-1 (PRIMARY)** — rustdoc documents resolution order and doc builds: GIVEN `crates/but-authz/src/authorize.rs` has `resolve_principal_with_registry` (Sprint 08 IDENT-003) and this task has added rustdoc documenting the 3-step resolution order (registry hit → principal; miss + `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` → env fallback; else → `Denial::unregistered`) WHEN `cargo doc -p but-authz --no-deps` is run AND the rendered rustdoc output is scanned for the resolution-order keywords THEN `cargo doc` exits 0 (no warnings-as-errors, no broken intra-doc links) AND the rendered documentation for `resolve_principal_with_registry` contains all three keywords: "registry hit" (or equivalent), "BUT_AUTHZ_ALLOW_ENV_HANDLE", and "Denial::unregistered".
- **Verify:** `cargo doc -p but-authz --no-deps` (assert exit 0) + `grep` the rendered `target/doc/but_authz/authorize/struct.*.html` (or equivalent text) for the three keywords
- **TEST_TIER:** unit · **VERIFICATION_SERVICE:** but-authz · **FLOW_REF:** UC-IDENT-03
- **UNIT_TEST_JUSTIFIED:** This AC verifies doc-comment CONTENT, not runtime behavior. There is no behavioral integration test for rustdoc text — the standard proof is `cargo doc` exit 0 + rendered-output keyword presence. A unit-tier verification is the honest classification; asserting this via an integration test would require a synthetic runner that re-parses rustdoc, which is not a real integration surface.
- **Scenario:** `start_ref=authorize_rs_doc_commented`; `must_observe` = [`cargo doc` exit 0, rendered rustdoc contains "BUT_AUTHZ_ALLOW_ENV_HANDLE", rendered rustdoc contains "Denial::unregistered", rendered rustdoc contains a phrase naming the registry-hit step]; `must_not_observe` = [`cargo doc` non-zero exit (broken doc / warnings-as-errors), any of the three keywords absent, a broken intra-doc link warning]; `negative_control.would_fail_if` = [rustdoc omits any of the three resolution steps, names wrong flag (e.g. `BUT_AGENT_HANDLE` instead of `BUT_AUTHZ_ALLOW_ENV_HANDLE`), names wrong denial variant (e.g. `Denial::no_handle` instead of `Denial::unregistered`), `cargo doc` fails on a broken intra-doc link].

**AC-2** — crate-root exports present and importable: GIVEN `crates/but-authz/src/lib.rs` has the existing `permissions_path` re-export AND this task has added `pub use config::agents_path;` AND Sprint 08's `Registry` + `resolve_principal_with_registry` exports are present (verified, re-added only if missing) WHEN a doctest in `lib.rs` (or `authorize.rs`) references `but_authz::agents_path()`, `but_authz::Registry`, and `but_authz::resolve_principal_with_registry` at the crate root THEN the doctest compiles and passes (`cargo test --doc -p but-authz`), proving all three are importable from `but_authz` root without a path qualifier.
- **Verify:** `cargo test --doc -p but-authz` (the new doctest passes) + `cargo check -p but-authz`
- **TEST_TIER:** unit · **VERIFICATION_SERVICE:** but-authz
- **UNIT_TEST_JUSTIFIED:** Doctest proving crate-root importability is the canonical Rust verification for re-exports — there is no integration surface (no git, no gate, no CLI); the proof is compile-time resolution.
- **Scenario:** `start_ref=lib_rs_agents_path_reexported`; `must_observe` = [`cargo test --doc -p but-authz` passes, the doctest body references all three crate-root paths — all three resolve without a module qualifier]; `must_not_observe` = [compile error on any of the three crate-root paths, doctest failure, unused-import warning].

## Test Criteria

| ID | Boolean | Maps to AC |
|----|---------|------------|
| TC-1 | `cargo doc -p but-authz --no-deps` exits 0 is true | AC-1 |
| TC-2 | The rendered rustdoc for `resolve_principal_with_registry` contains "BUT_AUTHZ_ALLOW_ENV_HANDLE", "Denial::unregistered", and a registry-hit phrase is true | AC-1 |
| TC-3 | A doctest referencing `but_authz::agents_path()`, `but_authz::Registry`, and `but_authz::resolve_principal_with_registry` at the crate root compiles and passes is true | AC-2 |
| TC-4 | `cargo test -p but-authz` passes unchanged (no behavioral regression from the doc-comment + re-export edit) is true | AC-2 |

## Reading List

1. `crates/but-authz/src/lib.rs:1-18` — the existing `pub use` block (mirror the `config::agents_path` addition after `permissions_path` at line 15).
2. `crates/but-authz/src/authorize.rs:60-102` — `resolve_principal` / `resolve_principal_from_env` rustdoc shape (the `resolve_principal_with_registry` doc mirrors this discipline; Sprint 08 IDENT-003 added the function body).
3. `crates/but-authz/src/config.rs:12-19` — `permissions_path()` function + its doctest (the `agents_path()` function added by IDENT-009 mirrors this exactly).
4. `.spec/prds/governance/12-uc-agent-identity.md:52-62` — UC-IDENT-03 acceptance criteria (the 3-step resolution order this rustdoc documents).

## Guardrails

**WRITE-ALLOWED:**
- `crates/but-authz/src/lib.rs` (MODIFY — add one `pub use config::agents_path;` line; re-add `Registry`/`resolve_principal_with_registry` `pub use` ONLY if grep shows them missing)
- `crates/but-authz/src/authorize.rs` (MODIFY doc-comments ONLY — add rustdoc to `resolve_principal_with_registry`; no signature/body/visibility changes)

**WRITE-PROHIBITED:**
- `crates/but-authz/src/config.rs` (IDENT-009 owns `agents_path()` function + `AgentWire`)
- `crates/but-authz/src/registry.rs` (Sprint 08 IDENT-001 owns `Registry` impl)
- `crates/but-authz/tests/**` (IDENT-015 owns `config.rs` tests; Sprint 08 IDENT-004 owns registry tests)
- Any function signature, body, visibility modifier, or error variant in `authorize.rs` (doc-comments ONLY)
- `crates/but-api/**` (IDENT-010/013 own but-api)
- `crates/but/**` (IDENT-011/014 own the CLI)

## Code Pattern

**Reference:** `crates/but-authz/src/lib.rs:13-16` (`pub use config::{... permissions_path};` — the exact line to extend with `agents_path`).
**Source:** `crates/but-authz/src/authorize.rs:60-66` (`resolve_principal` rustdoc — the `///` doc-comment discipline to mirror for `resolve_principal_with_registry`).

**Design notes:**
- Hard dependency on IDENT-009: `agents_path()` must exist in `config.rs` before the `lib.rs` re-export compiles. If IDENT-009 lands first, this is a one-line edit. If not, defer.
- Hard dependency on Sprint 08: `Registry` (IDENT-001) and `resolve_principal_with_registry` (IDENT-003) must exist. The previous dispatch confirmed `Registry` is exported; this task grep-verifies and re-adds only if missing (defensive — should be a no-op).
- Keyword choice for the rustdoc: the AC-1 grep asserts on "BUT_AUTHZ_ALLOW_ENV_HANDLE" and "Denial::unregistered" verbatim. The registry-hit phrase is softer ("registry hit" or "registry") — pick natural prose, the grep is tolerant.

**Anti-pattern:** Do NOT add a module-level `//!` doc that duplicates the function-level rustdoc (pick one surface — function-level is more discoverable); do NOT change the function signature "to make the doc nicer" (doc-only contract); do NOT add a new example binary or integration test for doc verification (`cargo doc` + doctest is the proof).

## Agent Instructions

1. **Grep** `crates/but-authz/src/lib.rs` for `Registry` and `resolve_principal_with_registry` re-exports — note any missing.
2. **Add** `pub use config::agents_path;` to the existing `pub use` block. If grep showed `Registry` or `resolve_principal_with_registry` missing, add those `pub use` lines too.
3. **Add rustdoc** to `resolve_principal_with_registry` in `authorize.rs` documenting the 3-step order with the verbatim keywords.
4. **Add a doctest** in `lib.rs` (or `authorize.rs`) referencing all three crate-root paths.
5. **Run:** `cargo doc -p but-authz --no-deps`, `cargo test --doc -p but-authz`, `cargo test -p but-authz`, `cargo check -p but-authz --all-targets`, `cargo clippy -p but-authz --all-targets -- -D warnings`, `cargo fmt --check`.
6. Commit via `but commit`.

## Orchestrator Verification Protocol

1. `cargo doc -p but-authz --no-deps` exit 0.
2. `cargo test --doc -p but-authz` exit 0 (new doctest passes).
3. `cargo test -p but-authz` exit 0 (no behavioral regression).
4. `grep target/doc/but_authz/` for the three keywords → all present.
5. `crates/but-authz/src/lib.rs` has `pub use config::agents_path;`.

## Agent Assignment

**Agent:** `rust-implementer` — small-surface docs + re-export task in `crates/but-authz/src/`. Pure Rust library-surface work — no CLI/TUI/frontend domain.
**Pairing:** none.

## Evidence Gates

- `cargo doc -p but-authz --no-deps` exit 0
- `cargo test --doc -p but-authz` exit 0 (new doctest)
- `cargo test -p but-authz` exit 0 (no behavioral regression)
- Rendered rustdoc contains the three keywords
- `pub use config::agents_path;` present in `lib.rs`

## Review Criteria

- Only doc-comment + `pub use` changes — no signature/body/visibility/variant changes (`git diff` is purely doc + re-export).
- The rustdoc names all three resolution steps + the verbatim flag + denial variant.
- `cargo test -p but-authz` unchanged (no regressions).
- The new doctest exercises all three crate-root paths.
- `cargo fmt --check` passes.

## Dependencies

- **Depends on:** IDENT-001 (Sprint 08 — Registry), IDENT-003 (Sprint 08 — `resolve_principal_with_registry`), IDENT-009 (`agents_path()` function).
- **Blocks:** Sprint 11 (the crate-root exports + rustdoc are the contract the `but-init`/`but-migrate` skills and `RULES.md` docs reference).

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "IDENT-016",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": false,
    "requires_seeded_evidence": false
  },
  "fixtures": {
    "authorize_rs_doc_commented": {
      "description": "repository source includes crates/but-authz/src/authorize.rs rustdoc documenting the registry-hit, Denial::unregistered, and BUT_AUTHZ_ALLOW_ENV_HANDLE resolution-order anchors",
      "seed_method": "migration_fixture",
      "records": [
        "checked-out source file crates/but-authz/src/authorize.rs",
        "cargo doc renders crate docs from that source"
      ]
    },
    "lib_rs_agents_path_reexported": {
      "description": "repository source includes crates/but-authz/src/lib.rs crate-root exports for agents_path, Registry, and resolve_principal_with_registry used by doctests",
      "seed_method": "migration_fixture",
      "records": [
        "checked-out source file crates/but-authz/src/lib.rs",
        "cargo test --doc compiles doctest against crate root"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN authorize.rs rustdoc documents the 3-step resolution order WHEN cargo doc -p but-authz --no-deps runs THEN exit 0 AND rendered rustdoc contains BUT_AUTHZ_ALLOW_ENV_HANDLE + Denial::unregistered + a registry-hit phrase",
      "test_tier": "unit",
      "verification_service": "but-authz",
      "unit_test_justified": "Doc-comment content verification — no behavioral integration surface; standard proof is cargo doc exit 0 + rendered keyword presence",
      "verify": "cargo doc -p but-authz --no-deps + grep rendered output",
      "maps_to_ac": null,
      "flow_ref": "UC-IDENT-03",
      "scenario": {
        "tier": "visible",
        "test_tier": "unit",
        "verification_service": "but-authz",
        "start_ref": "authorize_rs_doc_commented",
        "must_observe": [
          "cargo doc exit 0",
          "3 keywords present in rendered output"
        ],
        "must_not_observe": [
          "non-zero exit",
          "any keyword absent",
          "broken intra-doc link"
        ],
        "negative_control": {
          "would_fail_if": [
            "rustdoc omits a resolution step",
            "wrong flag name hardcoded in doc",
            "wrong denial variant name in doc",
            "broken or removed intra-doc link"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "authorize_rs_doc_commented",
            "action": {
              "actor": "ci",
              "steps": [
                "cargo doc -p but-authz --no-deps",
                "grep rendered output for 3 keywords"
              ]
            },
            "end_state": {
              "must_observe": [
                "cargo doc exit code == 0",
                "rendered rustdoc contains `BUT_AUTHZ_ALLOW_ENV_HANDLE`",
                "rendered rustdoc contains `Denial::unregistered`",
                "rendered rustdoc contains `registry hit`"
              ],
              "must_not_observe": [
                "cargo doc exit code != 0",
                "empty rendered rustdoc",
                "missing any one of the 3 documented anchors"
              ]
            }
          }
        ],
        "unit_test_justified": "Rustdoc rendering is the product surface for documentation acceptance: cargo doc proves intra-doc links compile and keyword grep proves the rendered public docs include the required resolution-order anchors without needing a runtime repo."
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "description": "GIVEN lib.rs re-exports agents_path + Sprint 08 exports verified WHEN a doctest references but_authz::agents_path()/Registry/resolve_principal_with_registry at crate root THEN it compiles and passes",
      "test_tier": "unit",
      "verification_service": "but-authz",
      "unit_test_justified": "Doctest proving crate-root importability — canonical Rust verification for re-exports; compile-time resolution is the proof",
      "verify": "cargo test --doc -p but-authz",
      "maps_to_ac": null,
      "scenario": {
        "tier": "visible",
        "test_tier": "unit",
        "verification_service": "but-authz",
        "start_ref": "lib_rs_agents_path_reexported",
        "must_observe": [
          "doctest pass",
          "3 crate-root paths resolve"
        ],
        "must_not_observe": [
          "compile error",
          "doctest fail",
          "unused-import warning"
        ],
        "negative_control": {
          "would_fail_if": [
            "agents_path re-export omitted from crate root",
            "Registry export removed/absent from Sprint 08 surface",
            "resolve_principal_with_registry export removed/absent from Sprint 08 surface"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "lib_rs_agents_path_reexported",
            "action": {
              "actor": "ci",
              "steps": [
                "cargo test --doc -p but-authz (doctest uses 3 crate-root paths)"
              ]
            },
            "end_state": {
              "must_observe": [
                "doctest exit code == 0",
                "crate-root path `but_authz::agents_path` resolves",
                "crate-root path `but_authz::Registry` resolves",
                "crate-root path `but_authz::resolve_principal_with_registry` resolves"
              ],
              "must_not_observe": [
                "compile error for any crate-root path",
                "doctest exit code != 0",
                "empty doctest body"
              ]
            }
          }
        ],
        "unit_test_justified": "Doctest compilation is the canonical unit surface for crate-root re-export contracts: the required behavior is compile-time path resolution of but_authz exports, not runtime authorization."
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "cargo doc -p but-authz --no-deps exits 0",
      "verify": "cargo doc -p but-authz --no-deps",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "rendered rustdoc contains the 3 resolution-order keywords",
      "verify": "grep target/doc/but-authz/ for the three keywords after cargo doc",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "doctest referencing 3 crate-root paths compiles + passes",
      "verify": "cargo test --doc -p but-authz",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "cargo test -p but-authz passes unchanged (no behavioral regression)",
      "verify": "cargo test -p but-authz",
      "maps_to_ac": "AC-2"
    }
  ]
}
-->
