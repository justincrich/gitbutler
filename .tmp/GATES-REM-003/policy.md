GATES-REM-003 policy decision

Selected policy: distinct-principal-per-required-group.

Reason: `review_requirement::evaluate` returns review shortfalls, while `config.invalid`
classification is produced by the merge-gate wrapper, which is outside this task's production
write scope. The evaluator now fails closed by assigning each required group to at most one
approving principal. If a required group has a current approval only through a principal already
assigned to another required group, the unmet entry uses `no_distinct_approval`.

This preserves disjoint required-group behavior and keeps enforcement based only on group
membership plus principal identity.
