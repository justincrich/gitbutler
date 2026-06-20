# GATES-REM-007 RED Evidence

Base SHA before edits: e236b9f2c47f838de5d989864e605cebc2a1ea8d

The pre-edit script happy path exited 0:

```text
$ ./tools/governance-checks/check_merge_gate_production_unchanged.sh
OK: crates/but-api/src/legacy/merge_gate.rs is unchanged from AUTHZ-004 baseline a80ce888a894ae3568ee8127866e987e89d6c092
```

The pre-edit task-file grep showed the script reference but no literal
`Baseline SHA` record:

```text
$ grep -n 'Baseline SHA' .spec/prds/governance/tasks/sprint-04-gates-deepening/GATES-008-merge-gate-failclosed-target-ref-only.md
exit_code=1
```

The pre-edit negative-test documentation grep failed:

```text
$ grep -E 'NEGATIVE_TEST|manual test|deletion|rename' tools/governance-checks/check_merge_gate_production_unchanged.sh
exit_code=1
```
