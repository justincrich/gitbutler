# GATES-REM-005 Manual Negative Controls

Recorded baseline for `check_merge_gate_production_unchanged.sh`:

- AUTHZ-004 baseline commit: `a80ce888a894ae3568ee8127866e987e89d6c092`
- Guarded production file: `crates/but-api/src/legacy/merge_gate.rs`
- Baseline probe: `git diff --quiet a80ce888a894ae3568ee8127866e987e89d6c092 -- crates/but-api/src/legacy/merge_gate.rs`
- Baseline probe result during implementation: exit `0`

Manual negative controls are intentionally documented instead of applied in this
worktree because this remedial task write-prohibits production source changes.
Each script is fail-closed and can be manually probed in a disposable copy:

1. `check_no_role_literals.sh`
   - Probe: add a standalone forbidden literal such as `"human"` or
     `maintainer` to `crates/but-api/src/legacy/review_requirement.rs`.
   - Expected result: script exits nonzero and prints the matching line.

2. `check_gate_helper_parity.sh`
   - Probe: remove or rename one `enforce_commit_gate_for_target` call in
     `crates/but-api/src/branch.rs`, or the only call in
     `crates/but-api/src/legacy/worktree.rs`.
   - Expected result: script exits nonzero with the deficient per-file count.

3. `check_merge_gate_production_unchanged.sh`
   - Probe: change any line in `crates/but-api/src/legacy/merge_gate.rs`
     relative to baseline commit `a80ce888a894ae3568ee8127866e987e89d6c092`.
   - Expected result: script exits nonzero and prints the file diff.
