# GATES-003 Blocker

The dispatch prompt declares `tdd_mode: red_first` and explicitly requires the
prewritten RED tests to already exist before implementation begins.

The assigned worktree does not contain the declared test files:

- `crates/but-api/tests/merge_gate.rs`
- `crates/but/tests/but/command/merge_gate.rs`

The required RED verification command was run:

```bash
cargo test -p but-api merge_gate && cargo test -p but merge_gate
```

It exited `0` and reported `running 0 tests` for the matching filters in both
crates. The full output is captured in `.tmp/GATES-003/red-output.txt`.

Per the dispatch prompt:

> If the test passes (all GREEN) or doesn't exist -> STOP. Report to orchestrator.

No production code was changed.
