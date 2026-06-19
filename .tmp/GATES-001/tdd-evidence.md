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
