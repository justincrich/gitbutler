# GATES-001 TDD Evidence

Base SHA: `220686834207ecd95d4b029e7589d1dea5b15db1`

## AC-1

RED command:

`cargo test -p but-api commit_gate_feature_ok_protected_rejected`

RED failure captured before the gate implementation:

```text
test commit_gate_feature_ok_protected_rejected ... FAILED
Error: protected main direct commit should be denied
```

GREEN command:

`cargo test -p but-api commit_gate_feature_ok_protected_rejected`

GREEN evidence: `.tmp/GATES-001/test-output.txt`

## AC-2

Seeded test:

`commit_gate_readonly_and_bad_handle_denied`

GREEN command:

`cargo test -p but-api commit_gate_readonly_and_bad_handle_denied`

GREEN evidence: `.tmp/GATES-001/test-output.txt`

## AC-3

Seeded test:

`commit_gate_edit_cannot_unprotect`

GREEN command:

`cargo test -p but-api commit_gate_edit_cannot_unprotect`

GREEN evidence: `.tmp/GATES-001/test-output.txt`

## AC-4

Seeded test:

`commit_gate_malformed_absent_and_dryrun`

GREEN command:

`cargo test -p but-api commit_gate_malformed_absent_and_dryrun`

GREEN evidence: `.tmp/GATES-001/test-output.txt`

## REVIEW-2 Regression

RED command:

`cargo test -p but --features legacy,but-2 commit_gate_allows_non_governed_commit2_flow --test but`

RED failure captured before scoping the gate:

```text
test command::commit_gate::commit_gate_allows_non_governed_commit2_flow ... FAILED
Error: {"error":{"code":"config.invalid","message":"invalid governance config: missing .gitbutler/permissions.toml at refs/heads/A"}}
```

GREEN command:

`cargo test -p but --features legacy,but-2 commit_gate_allows_non_governed_commit2_flow --test but`

GREEN evidence: `.tmp/GATES-001/but-cli-commit-gate-legacy-but2-output.txt`

Broad regression command:

`cargo test -p but --features legacy,but-2`

Broad evidence: `.tmp/GATES-001/but-broad-legacy-but2-output.txt`

## REVIEW-3 Regression

RED command:

`cargo test -p but-api commit_create_generated_entrypoint_authorizes_before_exclusive_guard`

RED failure captured before the API wrapper split:

```text
test commit_create_generated_entrypoint_authorizes_before_exclusive_guard ... FAILED
annotated commit_create must not take RepoExclusive because the macro acquires it before the function body
```

RED command:

`cargo test -p but-api commit_gate_commit_relative_checks_contents_write_without_branch_protection`

RED failure captured before commit-relative config-ref authorization:

```text
test commit_gate_commit_relative_checks_contents_write_without_branch_protection ... FAILED
commit gate errors should be structured
```

GREEN command:

`cargo test -p but-api commit_gate`

GREEN evidence: `.tmp/GATES-001/but-api-commit-gate-output.txt`

CLI regression command:

`cargo test -p but --features legacy,but-2 commit_gate`

CLI evidence: `.tmp/GATES-001/but-cli-commit-gate-legacy-but2-output.txt`

Broad regression command:

`cargo test -p but --features legacy,but-2`

Broad evidence: `.tmp/GATES-001/but-broad-legacy-but2-output.txt`
