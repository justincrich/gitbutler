---
stability: FEATURE_SPEC
last_validated: 2026-06-18
prd_version: 1.3.0
functional_group: LOOP
---

# Use Cases: Governed Orchestration Loop (LOOP)

This group is the **demonstration** that the permission layer and the two gates compose into the product goal: discrete agentic actors held to the same review/permission gates as humans, with **role separation emerging from the functional permission set rather than from code**. An orchestrator runs three principals — an implementer (`contents:write`, no `merge`), a reviewer (`reviews:write`, no `contents:write`), and a human maintainer (`merge`) — through GitButler's gated actions, and the implement→review→merge loop falls out of _what each principal may do_, not from any code that knows what an "implementer" or a "human" is. The second use case shows the user's quality model — **human at the feature level, AI at the code level** — expressed entirely as `gates.toml` config + group membership, with zero role-specific enforcement code.

| ID         | Title                                                      | Description                                                                                                                                                                                                                                                                                                                                                                                  |
| ---------- | ---------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| UC-LOOP-01 | Role separation emerges from the functional permission set | The implement→review→merge loop runs across three distinct principals; the implementer is denied the merge, the reviewer's edits are inert, only the maintainer lands — and no enforcement path references the labels "implementer"/"reviewer"/"maintainer". The loop's "open a PR" / "submit a review" steps gate on the governed `but pr` / `but review` actions, each permission-checked. |
| UC-LOOP-02 | Human-at-feature + AI-at-code as pure config               | The merge gate is configured to require an approval from both a `code-reviewers` group (AI) and a `maintainers` group (human); a merge with only one is blocked; the two-tier quality model is expressed entirely in `.gitbutler/gates.toml` + group membership.                                                                                                                             |

---

## UC-LOOP-01: Role separation emerges from the functional permission set

The product claim is that an agent is bound by the _config_, action by action, the same way a human contributor is — and that the familiar role separation of a code-review workflow need not be hardcoded. This use case runs the full loop: the orchestrator dispatches an implementer principal that commits to a feature branch and opens a PR; the implementer attempts to merge and is **denied** (it lacks `merge`); a reviewer principal attempts a commit and is **denied** (its edits are inert without `contents:write`) but submits an approving review; the human maintainer principal merges through the gate once the review requirement is satisfied. The separation is entirely a consequence of the three `AuthoritySet`s — there is no code that branches on a role name.

The loop's consequential steps are **governed `but` actions** at the `but-api` boundary: "open a PR" is `but pr new` (`pull_requests:write`), "submit a review" is `but review approve`/`request-changes` (`reviews:write`) / `but review comment` (`comments:write`), and "merge" is the governed merge action (`merge` + review requirement). These PR/review actions already appear in the route→Authority table (`10-technical-requirements/04-api-design.md`) and the `but pr`/`but review` CLI surface partly already ships (`crates/but/src/args/forge.rs`) — UC-LOOP-01 makes that gated surface **explicit**, not net-new scope.

> **Review-integrity caveat (R6, High — B12 / Y-NEW-7).** The merge gate's review requirement is satisfied by review submissions made through the governed `but review` action; a forged review via direct DB write to `local_review_verdicts` (R6 / R-NEW-1) is **outside governance scope** (the same accepted-leak class as the fence, R1). **The LOOP demo assumes honest review submission** — it proves the channel is gated and traversable, not that the review store is tamper-proof. Deferred hardening: HMAC/Ed25519 review integrity.

### Acceptance Criteria

☐ An implementer principal (`contents:write`, no `merge`) can commit to a feature branch and open a PR through GitButler's gated actions
☐ The implementer principal is denied the merge with the `perm.denied` contract because it lacks `merge`, so it cannot land its own work
☐ A reviewer principal (`reviews:write`, no `contents:write`) is denied a commit (its edits are inert) but can submit an approving review
☐ A human maintainer principal (`merge`) lands the change through the merge gate only after the configured review requirement is satisfied at head
☐ System enforces the entire separation from the functional permission sets alone, referencing no role label ("implementer"/"reviewer"/"maintainer") in any enforcement path (grep-asserted)
☐ The governed `but pr new` / `but pr close` (open/close a PR) and `but review approve` / `request-changes` / `comment` actions exist at the `but-api` boundary and are permission-checked — `pull_requests:write`, `reviews:write`, and `comments:write` respectively — per the route→Authority table (`10-technical-requirements/04-api-design.md`); these are the surface the LOOP demo's "open a PR" / "submit a review" steps gate on. `but pr new` already ships (`crates/but/src/args/forge.rs` `forge::pr::New`, exposed as `but pr`/`but review`); the missing governed verbs (`close`/`approve`/`request-changes`/`comment`) extend that same heading without duplicating `but pr new`. _(make-explicit — the route table and the `but pr`/`but review` CLI surface already implied this; not net-new scope)_
☐ System proves the governed path is **traversable** (the irrigation half, not only the dam): when the implementer is denied a direct commit to a protected branch, the denial's `remediation_hint` names a governed next action (commit to a feature branch → open a PR → get a review → merge) that, when followed, **succeeds** — so the demo shows the channel is cheap, not merely that the bypass is blocked
☐ System has a passing integration test against the real GitButler action surface + real git running the full implement→review→merge loop with three distinct principals, asserting: implementer commit accepted, implementer merge denied, reviewer commit denied, reviewer review accepted, maintainer merge succeeds after the approval; and that a denied implementer, following its `remediation_hint`, completes the governed feature-branch → reviewed-merge path successfully (all reviews submitted through the governed `but review` action — the forgeable direct-DB-write path is out of scope per R6 and is NOT under test)

---

## UC-LOOP-02: Human-at-feature + AI-at-code as pure config

The user's quality model is "only humans can own quality, so a human is the final reviewer; the AI review catches the low-level, obvious requirement misses you can't programmatically assert — a wholly-missing feature shouldn't even reach the human." This use case shows that model is expressible with no special code: a `code-reviewers` group (AI agents, `reviews:write`) and a `maintainers` group (humans, `merge`), with `.gitbutler/gates.toml` requiring an approving review from **each**. A merge with only the AI approval is blocked (the human hasn't owned it); a merge with only the human approval is blocked (the AI code-level pass is also required); both present, at head, and it lands. The engine never distinguishes a human from an AI — it evaluates group-membership of the signers against the config.

### Acceptance Criteria

☐ A User can configure the merge gate to require an approval from both a `code-reviewers` group and a `maintainers` group via `.gitbutler/gates.toml` (`require_approval_from_group = ["code-reviewers", "maintainers"]`)
☐ The merge gate blocks a merge that has a `code-reviewers` (AI) approval but no `maintainers` (human) approval, so the human's feature-level sign-off is required to land
☐ The merge gate blocks a merge that has a `maintainers` (human) approval but no `code-reviewers` (AI) approval, so the AI code-level pass is also required, order-independent
☐ The merge gate allows the merge once both a `code-reviewers` and a `maintainers` approval exist at the current head, with a `merge`-holding principal performing the merge
☐ System expresses "human at the feature level, AI at the code level" entirely as `.gitbutler/gates.toml` config + group membership, with no enforcement code that distinguishes a human from an AI principal (grep-asserted)
☐ System has a passing integration test against the real GitButler action surface + real git that configures the two-group requirement, supplies only the AI approval (asserts blocked), supplies only the human approval (asserts blocked), then supplies both at head (asserts the merge proceeds)
