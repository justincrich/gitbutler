# AUTHZ-006 RED evidence note

The RED run was captured by temporarily replacing
`crates/but-api/src/legacy/config_mutate.rs::enforce_administration_write_gate`
with a no-op body returning `Ok(())`, removing the real
`load_governance_config`, `resolve_principal_from_env`, and
`but_authz::authorize(... Authority::AdministrationWrite ...)` calls.

`AC-1-red-against-start.txt` is the real output from
`cargo test -p but-api admin_write_guard_denies_non_admin_allows_admin` in that
no-op state. It fails in `admin_write_guard_denies_non_admin_allows_admin`
because the dev-denied half reaches `classified_error(...)`, receives `Ok(())`,
and reports `Error: administration write gate should reject this scenario`.

`AC-3-red-against-start.txt` and `red-output.txt` were captured against the same
temporary no-op guard. The real guard was restored before all GREEN and seeded
captures, and `crates/but-api/src/legacy/config_mutate.rs` is unchanged in the
final committed state.
