# Pre-Existing Issues Blocking Broad Gates

## but-api all-targets compile failure

- Command: `cargo check -p but-api --all-targets`
- Evidence: `.tmp/CLI-REM-002/but-api-check-output.txt`
- Failure: `crates/but-api/tests/governance_api.rs` imports unresolved symbols:
  `governance_status_read`, `group_add_member_cmd`, and `perm_grant_cmd`.
- Classification: pre-existing external Sprint06a blocker named in the task prompt.
- Scope decision: `crates/but-api/tests/governance_api.rs` is outside CLI-REM-002 writeAllowed and was not modified.

## Manifest command syntax issue

- Command: `cargo test -p but perm_denials_include_remediation_hint group_denials_include_remediation_hint`
- Evidence: `.tmp/CLI-REM-002/manifest-command-invalid.txt`
- Failure: Cargo accepts one positional test filter; the second filter is parsed as an unexpected argument.
- Mitigation: ran the two in-scope CLI tests separately:
  `cargo test -p but perm_denials_include_remediation_hint` and
  `cargo test -p but group_denials_include_remediation_hint`.
