# GRPS-001 RED Evidence Impossibility Proof

Task: GRPS-001
Baseline under test: `55fa6829d8ca7a9d2378363bd198f08e88c36887`
Final task branch HEAD before this remediation: `e814be6ae559e369b663b62b702a94faa49e3304`
Isolated baseline overlay directory: `/tmp/grps-001-baseline-evidence`

## Method

I created an isolated archive copy of baseline `55fa6829d8ca7a9d2378363bd198f08e88c36887`, copied only the final `crates/but-authz/tests/grps_union.rs` test file from the GRPS-001 branch into that baseline copy, and ran each required AC command there.

No product source was edited for this remediation.

## Baseline Behavior

Baseline `authorize.rs` still contains the runtime group re-union path that GRPS-001 simplified:

```rust
cfg.groups()
    .values()
    .filter(|group| group.members().contains(principal.id()))
    .fold(authorities.clone(), |held, group| {
        held.union(group.authorities())
    })
```

Baseline `config.rs` already folds both principal-declared groups and group `members = [...]` into `principal_authorities` at load time. The final GRPS-001 watched tests validate the externally visible folded-authority behavior, and they do not fail against this baseline implementation.

## Watched Baseline Overlay Results

### AC-1

Command:

```bash
cargo test -p but-authz group_union_authorizes_review_denies_merge -- --nocapture
```

Captured output file: `.tmp/GRPS-001/AC-1-baseline-overlay-output.txt`

Relevant output:

```text
running 1 test
`authorize(reviewer-only, ReviewsWrite, cfg)` returns `Ok(())`
`authorize(reviewer-only, Merge, cfg)` returns `code == "perm.denied"`
`Merge denial.message` contains "merge"
test group_union_authorizes_review_denies_merge ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.18s

EXIT_CODE=0
```

### AC-2

Command:

```bash
cargo test -p but-authz union_paths_stay_equal -- --nocapture
```

Captured output file: `.tmp/GRPS-001/AC-2-baseline-overlay-output.txt`

Relevant output:

```text
running 1 test
effective_authority(reviewer-only) == principal_authorities(reviewer-only)
effective_authority(reviewer-byref) == principal_authorities(reviewer-byref)
both reviewer paths contain exactly reviews:write; ro excludes reviews:write
test union_paths_stay_equal ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.17s

EXIT_CODE=0
```

### AC-3

Command:

```bash
cargo test -p but-authz delegated_admin_ceiling -- --nocapture
```

Captured output file: `.tmp/GRPS-001/AC-3-baseline-overlay-output.txt`

Relevant output:

```text
running 1 test
`authorize(delegate, AdministrationWrite, cfg)` returns `Ok(())`
`authorize(reviewer-only, AdministrationWrite, cfg)` returns `code == "perm.denied"`
test delegated_admin_ceiling ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.14s

EXIT_CODE=0
```

### AC-4

Command:

```bash
cargo test -p but-authz claims_do_not_widen_union_even_with_group_backing -- --nocapture
```

Captured output file: `.tmp/GRPS-001/AC-4-baseline-overlay-output.txt`

Relevant output:

```text
running 1 test
fabricated merge claim for reviewer-only is denied with perm.denied
fabricated administration:write claim for reviewer-only is denied with perm.denied
test claims_do_not_widen_union_even_with_group_backing ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.16s

EXIT_CODE=0
```

## Conclusion

Truthful watched RED evidence cannot be produced for AC-1 through AC-4 by applying the final GRPS-001 test file to baseline `55fa6829d8ca7a9d2378363bd198f08e88c36887`, because all four final watched AC tests pass on that baseline.

Normal remediation by relabeling structural grep output or passing test output as RED would be fake evidence. The task contract needs amendment to accept structural/behavior-neutral evidence or to waive/drop the `red_first` requirement for this behavior-neutral consolidation.
