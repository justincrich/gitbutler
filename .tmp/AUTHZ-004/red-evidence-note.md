# AUTHZ-004 RED evidence note

The attempt-2 RED run is .tmp/AUTHZ-004/red-output.txt from `cargo test -p but-api merge_gate` after adding the required tests and before production changes. It shows a real failing assertion in `merge_gate_undefined_required_group_denied`: the gate returned `require_approval_from_group ghost-reviewers: no_approval` instead of the required `undefined required group ghost-reviewers`. The AC-1, AC-2, and AC-4 tests were added in the same RED slice and were already green against existing behavior; the production change for this attempt is scoped to the AC-3 gap.
