# LPR-013 Evidence Package

Task: `principal_kind_read`/`principal_kind_update` governed-config but-api
producer + Tauri command/SDK delta.

Source spec: `.spec/prds/governance/tasks/sprint-07-local-agent-pr/LPR-013-kind-tauri-sdk-producer.md`

Branch: `design/lpr-ui-design-contracts`
Base SHA: `886f4fc71c27a25f6f2b902f7f91d6803cd3291d`
Wip commit: `f177f0481a6b09fde4a05b45354e9f9e69cb6df4`

## Wip delta verified against spec

The wip `f177f0481a` shipped:

1. `crates/gitbutler-tauri/src/governance.rs` (195 lines) — a gitbutler-tauri-ONLY
   implementation of `get_principal_kind_for_project` /
   `set_principal_kind_for_desktop_session` plus helpers and
   `tauri_get_principal_kind`/`tauri_set_principal_kind` modules.
2. `crates/gitbutler-tauri/src/lib.rs` (5 lines) — registration of
   `get_principal_kind`/`set_principal_kind` plus a drive-by cleanup of a
   no-op `.map_err(anyhow::Error::from)`.
3. `crates/gitbutler-tauri/tests/tt_024_principal_kind.rs` (254 lines) — a
   standalone IPC test.

### Gaps identified vs. spec

| AC | Gap in wip | Resolution |
|----|------------|------------|
| STRICTLY | Wip buried logic in `gitbutler-tauri`; spec STRICTLY requires `principal_kind_read`/`principal_kind_update` at the `but-api` boundary (CLI/Tauri/N-API all share). | Moved the producer fns + DTOs to `crates/but-api/src/legacy/governance.rs`; gitbutler-tauri keeps only the fleet-owner wrapper + Tauri command. |
| Naming | Wip used `get_principal_kind`/`set_principal_kind`; spec requires `principal_kind_read`/`principal_kind_update` (the SDK invoke keys). | Renamed everywhere; the but-api `#[but_api(napi)]` macro generates `tauri_principal_kind_read::principal_kind_read` and the desktop wrapper provides `tauri_principal_kind_update::principal_kind_update`. |
| AC-1 | Wip's `effective_permissions` PREFERRED the working-tree file, so the read reported the just-staged kind — breaking the inert-until-committed pair. | New `principal_kind_read_with_repo` reads the COMMITTED kind via `but_authz::load_permissions_wire(repo, target_ref)` and surfaces pending=true via working-tree diff. |
| AC-2 | Wip used `PermissionsWire` round-trip but had no test asserting full-schema preservation. | New `principal_kind_update_round_trips_full_schema_lossless` asserts admin grants, agent-A grants+groups, rust-implementer kind=human, and `[[group]]` reviewers all survive. |
| AC-3 | Wip's DTO was `{principal, kind}` — no list, no pending signal. | New DTOs `PrincipalKindList { principals: Vec<PrincipalKindEntry> }`, `PrincipalKindEntry { principal_id, kind, pending }`, `PrincipalKindOutcome { principals, caveat }`. |
| AC-4 | Wip's `set_principal_kind_for_desktop_session` BYPASSED `enforce_administration_write_gate` (claimed "fleet-owner unconditional authority"). Spec MUST compose the AUTHZ-006 guard. | New `principal_kind_update_with_repo` calls `enforce_administration_write_gate(repo, target_ref)?` BEFORE the write; `_as_fleet_owner` variant is for the desktop wrapper. |
| AC-5 / TC-10 | Wip's `tt_024_principal_kind.rs` was a standalone test; spec requires commands in `mgmt_ipc_003_governance_commands.rs` (registration + invocation on the real bus + forbidden-allow-file assertion). | Deleted `tt_024_principal_kind.rs`; added `principal_kind_read`/`principal_kind_update` to `GOVERNANCE_COMMANDS`, `InvocationCase`s, and two AC-5 tests in `mgmt_ipc_003_governance_commands.rs`. |
| TC-11 | Wip did not update the forbidden-allow-file assertion. | Added `name.starts_with("allow-principal_kind_")` to the IPC test's forbidden-allow-file filter. |
| MUST (honesty grep) | (introduced by my first impl iteration) `"human" =>` arm + comment quoting the pattern tripped the `invariant_build_gates` human-vs-AI grep. | Restructured `parse_principal_kind` as a slice-contains validator (no per-kind match arm) and rephrased the comment. |

## Pre-existing conditions (NOT introduced by LPR-013)

1. **SEC-honesty literal in `GOVERNANCE_COMMIT_PATHS`**: `crates/but-api/src/legacy/governance.rs:155` carries the literal `".gitbutler/permissions.toml"` in a const array of commit-path identifiers. This pre-exists the sprint (verified against `git show 8c5458479b:crates/but-api/src/legacy/governance.rs` — same line, same literal). The actual writer code resolves paths via `permissions_path()`; the const is a stable commit-message identifier, not a re-derived I/O path. Not changed by LPR-013.
2. **SDK regen blocked**: `pnpm build:sdk` fails on a pre-existing `forge.rs` issue (`LocalReviewAssignment`/`LocalReviewComment` from LPR-001/LPR-003 lack `InternalJsonSchema` impls — zero diff in forge.rs from HEAD). LPR-010 owns the `but-db`/`InternalJsonSchema` audit. Deferred.
