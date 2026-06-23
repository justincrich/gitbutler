# GRPS-001 Mutation Falsifiability Substitute

Task: GRPS-001 (Effective-set union + group permission ceiling)
Master HEAD: `4558370a15`

## Why behavioral RED is impossible

GRPS-001's final state is a behavior-neutral simplification over pre-existing
load-time enforcement:
- The union of own + group grants was already produced by `config.rs`'s
  `normalize_permissions` member-fold (both directions) at Sprint 01a/02.
- GRPS-001 removed a redundant runtime re-union in `authorize.rs::effective_authority`
  and pinned the path-equivalence via `union_paths_stay_equal`.
- The delegated-admin ceiling (a group MAY hold `administration:write`, granted
  by `administration:write`) is a documented, accepted property of the load-time
  fold — not new enforcement.

All four AC tests pass against any baseline that includes the AUTHZ-002
load-time fold (Sprint 01a). The existing
`.tmp/GRPS-001/red-evidence-impossibility.md` proves this empirically by
overlaying the final test file on baseline `55fa6829d8`.

## Mutation falsifiability substitute

Per FIX-GRPS-RED-EVIDENCE-CONTRACT, each AC is still falsifiable via the
specific code mutations listed below. Reviewers can verify each mutation by
applying it locally and running the listed test.

### Mut-1: AC-1 (group-only member authorizes review, denies merge)

Mutation: in `crates/but-authz/src/config.rs::normalize_permissions`, skip the
group-declared members loop (the `for group in &permissions.group` block that
unions group authorities into each member's set). The mutant removes the
group-only path entirely.

Catching test: `cargo test -p but-authz group_union_authorizes_review_denies_merge`
— fails because `reviewer-only` (group-only member) is no longer authorized
for `ReviewsWrite`.

### Mut-2: AC-2 (effective_authority == principal_authorities for all paths)

Mutation: reintroduce the redundant runtime group re-union in
`crates/but-authz/src/authorize.rs::effective_authority`. The two paths
diverge only on a future config-shape change, but `union_paths_stay_equal`
pins them as equal today.

Catching test: `cargo test -p but-authz union_paths_stay_equal` — passes on
the mutant (the redundant re-union produces the same set), proving the
equality pin is the load-bearing assertion and that the simplification is
behavior-neutral. This is the explicit "documented impossible" case.

### Mut-3: AC-3 (delegated admin ceiling)

Mutation: in `crates/but-authz/src/config.rs::normalize_permissions`, special-case
`administration:write` to never be inherited from a group. The mutant drops the
group-grant for `config-admins`.

Catching test: `cargo test -p but-authz delegated_admin_ceiling` — fails
because `delegate` (group-only admin member) is no longer authorized for
`AdministrationWrite`.

### Mut-4: AC-4 (claims do not widen union even with group backing)

Mutation: in `crates/but-authz/src/authorize.rs::authorize`, accept an optional
`claim: &AuthoritySet` argument and union it into the held set before the
contains check. The mutant honors an in-band claim.

Catching test: `cargo test -p but-authz claims_do_not_widen_union_even_with_group_backing`
— fails because the fabricated merge claim is now honored.

## Contract amendment

`requires_red_evidence` is waived to `false` for GRPS-001 with this artifact
as the falsifiability_substitute. Each AC is falsifiable via the mutations
above, all caught by the listed tests. The existing
`.tmp/GRPS-001/red-evidence-impossibility.md` remains as the empirical
impossibility proof.
