GRPS-001 evidence

Baseline after refresh: 55fa6829d8ca7a9d2378363bd198f08e88c36887
Worktree: /Users/justinrich/Projects/gitbutler/.kb-run-sprint/worktrees/GRPS-001
Branch: kb-run-sprint/sprint-03-grps-groups-ref-pin/GRPS-001

Implementation summary:
- Simplified `effective_authority` to return the already-folded `GovConfig::principal_authorities(principal.id())` set.
- Left `config.rs::normalize_permissions` as the single load-time fold for direct grants plus both group membership directions, adding only a clarifying comment.
- Added `grps_union.rs` integration coverage for group-only review authority, equality with loaded principal authorities, delegated admin ceiling, and caller-claim non-widening.

Refreshed verification on the new baseline:
- `! grep -nE 'principal\.authorities\(\)' crates/but-authz/src/authorize.rs`
  - Evidence: `.tmp/GRPS-001/verify-grep-no-principal-authorities.txt`
- `cargo test -p but-authz group_union_authorizes_review_denies_merge`
  - Evidence: `.tmp/GRPS-001/green-ac1-group-union.txt`
- `cargo test -p but-authz union_paths_stay_equal`
  - Evidence: `.tmp/GRPS-001/green-ac2-union-paths.txt`
- `cargo test -p but-authz delegated_admin_ceiling`
  - Evidence: `.tmp/GRPS-001/green-ac3-delegated-admin.txt`
- `cargo test -p but-authz claims_do_not_widen_union_even_with_group_backing`
  - Evidence: `.tmp/GRPS-001/green-ac4-claims.txt`
- `cargo test -p but-authz invariant_build_gates`
  - Evidence: `.tmp/GRPS-001/verify-invariant-build-gates.txt`
- `cargo check -p but-authz --all-targets`
  - Evidence: `.tmp/GRPS-001/verify-cargo-check.txt`
- `cargo clippy -p but-authz --all-targets -- -D warnings`
  - Evidence: `.tmp/GRPS-001/verify-cargo-clippy-deny-warnings.txt`
- `cargo test -p but-authz`
  - Evidence: `.tmp/GRPS-001/verify-cargo-test-but-authz.txt`
- `cargo fmt --check`
  - Evidence: `.tmp/GRPS-001/verify-cargo-fmt-check.txt`
- `cargo doc -p but-authz --no-deps`
  - Evidence: `.tmp/GRPS-001/verify-cargo-doc.txt`

RED evidence carried forward from the paused WIP:
- `.tmp/GRPS-001/red-ac1-group-union.txt`
- `.tmp/GRPS-001/red-structural-authorize-reunion.txt`
