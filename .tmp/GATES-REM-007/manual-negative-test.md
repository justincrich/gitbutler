# Manual Negative Test Documentation

Documented in `tools/governance-checks/check_merge_gate_production_unchanged.sh`
under `NEGATIVE_TEST`.

Procedure for a reviewer in a disposable worktree:

1. Delete one production line from `crates/but-api/src/legacy/merge_gate.rs` or
   rename that file.
2. Run `./tools/governance-checks/check_merge_gate_production_unchanged.sh`.
3. Confirm the script exits non-zero and reports the missing file, staged
   change, unstaged change, or committed baseline-range diff.
4. Restore the disposable worktree.

I did not execute this destructive production-file edit in the sprint worktree
because the task explicitly prohibits modifying production source files.
