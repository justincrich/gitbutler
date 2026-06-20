---
stability: FEATURE_SPEC
last_validated: 2026-06-18
prd_version: 1.3.0
functional_group: GRPS
---
# Use Cases: Principal Grouping (GRPS)

Functional permissions are granted to **users or groups** — the GitHub teams model, and the net-new grouping capability beyond a basic per-principal permission model. A principal's effective `AuthoritySet` is the **union** of its own grants and the grants of every group it belongs to, so a reviewer can hold `reviews:write` purely by membership in a `code-reviewers` group. Crucially, group definitions, grants, and membership are themselves **committed, ref-pinned governed config**, read at the **target ref** when authorizing — so a change that adds its own author to a privileged group is ineffective until landed on the target branch. This is what makes grouping safe: the lever that grants power is the same governed, ref-pinned lever as every other permission, never the working tree.

| ID | Title | Description |
|----|-------|-------------|
| UC-GRPS-01 | Grant functional permissions to groups; effective set = union | A User defines a group with a functional permission set and adds principals to it; the authorization check resolves a principal's effective `AuthoritySet` as the union of direct grants + every group's grants, so a principal authorized via a group is treated identically to one with a direct grant. |
| UC-GRPS-02 | Ref-pinned governed group membership (no self-escalation) | Group config is read at the **target ref** when authorizing, so a change whose head adds its author to a privileged group cannot authorize that same change; a `but group` membership change takes effect only once committed to the target branch. |

---

## UC-GRPS-01: Grant functional permissions to groups; effective set = union
Managing per-principal grants individually does not scale and does not match how teams think about access. This use case adds **groups** as a first-class grantee. A `[[group]]` entry in `.gitbutler/permissions.toml` carries a functional permission set; principals reference groups by membership; and the `but-authz` resolver computes a principal's effective `AuthoritySet` as the union of its own grants and every group it belongs to. The authorization check is unchanged in shape — it asks "does the effective set contain the required `Authority`" — so nothing downstream needs to know whether the permission came from a direct grant or a group. `but group` (create / grant / membership / delete) manages it, each operation itself gated by `administration:write`.

### Acceptance Criteria
☐ A User can define a group with a functional permission set in committed `.gitbutler/permissions.toml`, via `but group create` + `but group grant`
☐ A User can add a principal to a group with `but group add-member`, so the principal inherits the group's functional permissions
☐ System resolves a principal's effective `AuthoritySet` as the union of its own grants and the grants of every group it belongs to
☐ The authorization check authorizes an action when any source — a direct grant or a group grant — supplies the required `Authority`, so a reviewer inheriting `reviews:write` only via a `code-reviewers` group may submit a review
☐ System denies an action when neither the principal's direct grants nor any of its groups supply the required `Authority`, returning the `perm.denied` contract
☐ `but group` operations (create / grant / membership / delete) are themselves gated by `administration:write`, consistent with UC-AUTHZ-03
☐ System defines the group permission ceiling explicitly: a group **may** hold any functional permission including `administration:write`, and granting a group `administration:write` (delegated admin — its members may then change config) is an **accepted, named property** that requires `administration:write` to set, not a silent escalation — so the audit surface of "who can change governed config" is the union of direct holders and members of admin-holding groups
☐ System has a passing integration test against the real `but-authz` crate that grants `reviews:write` to a `code-reviewers` group, adds a principal with no direct review grant, and asserts the principal is authorized to review via the group — and is still denied `merge`, which no source grants

---

## UC-GRPS-02: Ref-pinned governed group membership (no self-escalation)
Groups concentrate authority, so they are the obvious escalation target: if an agent could add itself to a `merge`-holding group on its own change, grouping would *weaken* governance. This use case closes that by reading group config — definitions, grants, and membership — at the **target ref** when authorizing a git action, exactly as `.gitbutler/permissions.toml` is read. A change whose head edits the group config to add its author is judged against the membership committed on the target branch, not the head it is trying to land. A `but group` edit therefore takes effect only once committed to the target ref (never from the working tree), and changing a protected branch's group config is an `administration:write`-gated, governed change.

### Acceptance Criteria
☐ System reads group definitions, grants, and membership at the **target ref** when authorizing a git action, consistent with the `.gitbutler/permissions.toml` ref-pin
☐ System makes a change that adds its own author to a privileged group **ineffective** for authorizing that same change, because membership is read at the target ref, not the working tree or the feature head
☐ A `but group` membership change takes effect only once committed to the target branch, so a grant is inert until landed (and `but group` output says so)
☐ Changing a protected branch's group config is an `administration:write`-gated, governed change, consistent with UC-AUTHZ-03
☐ System has a passing integration test against real git that creates a feature change whose head adds its author to a `merge`-holding `maintainers` group, attempts the merge, and asserts the author is still denied `merge` because the authorizing membership is the target-ref version (self-escalation prevented)
☐ System reads group membership ONLY from the target-ref config blob when authorizing — NEVER from the working tree or the feature head — so a staging-area or uncommitted edit cannot influence the authorization decision
