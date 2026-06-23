# GRPS-002 RED Evidence Impossibility + Mutation Falsifiability Substitute

Task: GRPS-002 (Ref-pinned membership + self-grant-inert)
Master HEAD: `4558370a15`

## Why behavioral RED is impossible

GRPS-002 is a capstone over pre-existing enforcement:
- The ref-pinned read is produced by `load_governance_config` (Sprint 01a AUTHZ-002). The target-ref peeling already shipped with its own RED evidence in Sprint 01a.
- The self-grant-inert property is a direct consequence of target-ref-only reads (a self-grant on a feature branch is invisible to a `refs/heads/main` read).
- The merge-gate enforcement that consumes the loaded config was proven in Sprint 01b (GATES-003).

The four AC tests are capstones over already-shipped enforcement — applying them to any baseline after AUTHZ-002 landed produces GREEN, not RED. A behavioral RED is impossible without rolling back AUTHZ-002 itself, which would also roll back the contract the capstone is built on.

## Mutation falsifiability substitute

Per FIX-GRPS-RED-EVIDENCE-CONTRACT, when behavioral RED is impossible, the contract must record the specific code mutations that would falsify each AC, plus pointers to the negative-control tests that catch those mutations. Reviewers can verify each mutation by applying it locally and running the listed test.

### Mut-1: AC-1 (feat-author self-add on feature head still denied merge)

Mutation: in `crates/but-authz/src/config.rs::load_governance_config_inner`, replace
`repo.find_reference(target_ref)` with `repo.head()`. The loader now peels HEAD
instead of the target ref, reading the feat-admin tree (which carries the
self-grant) instead of main's pre-grant tree.

Catching test: `cargo test -p but-api --test merge_gate_self_escalation
self_add_to_maintainers_on_feature_head_still_denied_merge` — fails (mutant
authorizes when it should deny).

Also caught after FIX-GRPS-002-AC3-TEETH lands: `cargo test -p but-authz
self_grant_admin_inert_until_landed` — fails (the HEAD-peel mutant now
authorizes feat-author for AdministrationWrite before landing).

### Mut-2: AC-2 (landed membership clears merge-authority step)

Mutation: in `crates/but-api/src/legacy/merge_gate.rs::classify_error`,
remove the `gate.review_required` arm. The mutant would surface an
unstructured error instead of the expected next-gate code.

Catching test: `cargo test -p but-api --test merge_gate_self_escalation
landed_membership_clears_merge_authority_step` — the post-landing
`code != "perm.denied"` AND `code == "gate.review_required"` assertion
fails.

### Mut-3: AC-3 (self-grant administration:write is inert until landed)

Mutation: same as Mut-1 (HEAD peel). After FIX-GRPS-002-AC3-TEETH leaves
HEAD on feat-admin, a HEAD-peel mutant reads the feat-admin tree and
authorizes AdministrationWrite before landing.

Catching test: `cargo test -p but-authz self_grant_admin_inert_until_landed` —
fails on the pre-landing `assert_denied` (mutant returns Ok when it should
return Denial).

### Mut-4: AC-4 (membership read only from target ref)

Mutation: in `crates/but-authz/src/config.rs::load_governance_config_inner`,
peel `repo.head()` instead of `target_ref`. The mutant reads feat's
working-tree (or committed-on-feat) membership that includes feat-author.

Catching test: `cargo test -p but-authz membership_read_only_from_target_ref` —
the `!maintainers.members().contains(&PrincipalId::new("feat-author"))`
assertion fails.

## Contract amendment

`requires_red_evidence` is waived to `false` for GRPS-002 with this artifact
as the falsifiability_substitute. Each AC is falsifiable via the mutations
above, all caught by the listed tests.
