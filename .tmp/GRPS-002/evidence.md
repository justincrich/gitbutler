# GRPS-002 Evidence Remediation

## Harvested verification

`TASK_ID=GRPS-002 bash ~/Projects/brain/skills/kb-run-sprint/scripts/harvest-evidence.sh` completed at `2026-06-19T18:33:53Z`.

- `verification-summary.json` contains all 12 required rows: `AC-1` through `AC-4` and `TC-1` through `TC-8`.
- Every requirement row has a real `exit_code: 0` and a captured output file.
- The harvester reported `12 pass, 0 fail`.

## Behavioral green evidence

- `AC-1`: `.tmp/GRPS-002/ac1-primary-green.txt` shows `self_add_to_maintainers_on_feature_head_still_denied_merge` passing. The test seeds a feature-head self-add but asserts the target-ref membership still denies merge authority and leaves `refs/heads/main` unchanged.
- `AC-2`: `.tmp/GRPS-002/ac2-landed-membership.txt` shows `landed_membership_clears_merge_authority_step` passing. The test advances `refs/heads/main` with the membership, then observes the merge-authority `perm.denied` is gone and the expected residual `gate.review_required` remains.
- `AC-3`: `.tmp/GRPS-002/ac3-self-grant-admin-green.txt` shows `self_grant_admin_inert_until_landed` passing. The test denies `administration:write` from the feature-head self-grant, then authorizes after the grant lands on `refs/heads/main`.
- `AC-4`: `.tmp/GRPS-002/ac4-target-ref-membership.txt` shows `membership_read_only_from_target_ref` passing. The test keeps `HEAD` and the working tree carrying `feat-author` membership while `load_governance_config(repo, refs/heads/main)` still excludes `feat-author` from the target-ref maintainers group.

## RED evidence status

True RED-against-start evidence is not available and should not be invented.

The existing failure captures are not behavioral RED evidence:

- `.tmp/GRPS-002/ac1-primary.txt` fails because `tests/fixtures/scenario/governance-base.sh` could not be read from the `crates/but-api` current working directory.
- `.tmp/GRPS-002/ac3-self-grant-admin.txt` fails because the test referenced `tempfile::TempDir` before that test target could resolve the crate.

Neither failure proves the target-ref authorization property was missing.

The current implementation commit `d6ff466a64ca7b588d2718e941994d8e494f1800` changes only:

- `crates/but-api/tests/merge_gate_self_escalation.rs`
- `crates/but-authz/tests/grps_ref_pin.rs`

No product source changed between baseline `c33e6b9d79341c8950f012265912488cf5504c28` and the implementation commit. Because the final behavioral tests pass against unchanged product logic, normal completion would require fake RED evidence. The correct remediation status is `blocked` with `blocker_classification: task_contract_invalid`.
