# Setting up GitButler governance

GitButler governance holds the agents (and people) working in a project to the
**same commit/merge gates** — so code lands through a reviewed, permissioned path
instead of whoever has push access. The **Permissions & Governance** settings page
reads this configuration; until it's set up, that page shows a "not configured"
state and this guide.

There is **no in-app setup wizard yet** — governance is configured by committing two
files to your project's **target branch**, or via the `but` CLI. This page explains
both.

## How it's read (important)

Governance config is **ref-pinned**: GitButler reads it from the commit at your
**target branch** (e.g. `refs/remotes/origin/master`), _not_ from your working tree.
This is deliberate — it means an agent can't grant itself authority just by editing a
local file; the rules only take effect once they're committed to the branch everyone
merges into. So: add the files, **commit them to the target branch**, and the page
populates.

The two files both live under `.gitbutler/` at the repository root:

| File                          | Purpose                                                                    |
| ----------------------------- | -------------------------------------------------------------------------- |
| `.gitbutler/permissions.toml` | **Who** can do what — principals, groups, roles                            |
| `.gitbutler/gates.toml`       | **Where** it's enforced — branch protection + the merge review requirement |

## `.gitbutler/permissions.toml`

Define **principals** (each agent or person, identified by its handle) and **groups**.
A principal's effective authority is its own grants ∪ the grants of every group it
belongs to.

```toml
# Groups bundle authorities so you don't repeat them per principal.
[[group]]
name = "code-reviewers"
permissions = ["reviews:write", "comments:write", "contents:read"]

[[group]]
name = "maintainers"
permissions = ["merge", "reviews:write", "administration:write"]

# Implementers can commit + open PRs, but CANNOT merge.
[[principal]]
id = "rust-implementer"
role = "write"

# Reviewers approve via membership in code-reviewers.
[[principal]]
id = "rust-reviewer"
groups = ["code-reviewers"]

# The operator/orchestrator can merge.
[[principal]]
id = "operator"
role = "maintain"
```

**Roles** desugar to authority sets:

| Role       | Grants (summary)                                       | Can commit? | Can merge? |
| ---------- | ------------------------------------------------------ | ----------- | ---------- |
| `read`     | metadata/contents/PR read                              | —           | —          |
| `triage`   | read + statuses:read                                   | —           | —          |
| `write`    | read + contents/PR/reviews/comments/statuses **write** | ✅          | ❌         |
| `maintain` | write + **merge** + administration:read                | ✅          | ✅         |
| `admin`    | everything                                             | ✅          | ✅         |

You can also grant explicit authority tokens instead of a role via
`permissions = ["contents:write", "pull_requests:write", ...]`.

## `.gitbutler/gates.toml`

Protect a branch and require a distinct reviewer approval before a merge can land.

```toml
[[branch]]
name = "master"        # your protected/target branch
protected = true       # no direct commits — changes land via a gated merge

[[gate]]
branch = "master"
type = "review"
min_approvals = 1
require_distinct_from_author = true               # the approver can't be the author
require_approval_from_group = ["code-reviewers", "maintainers"]
```

## Identity

The acting principal is resolved from the **`BUT_AGENT_HANDLE`** environment variable
(e.g. `BUT_AGENT_HANDLE=rust-implementer`), matched against `permissions.toml`. An
agent can't claim another principal — its authority always comes from the committed
config, never from the agent itself. (In the desktop app, the signed-in user acts as
the trusted _fleet-owner_ for editing governance.)

## CLI alternative

You can build the same config with the `but` CLI instead of hand-writing TOML, then
commit it:

```bash
but group create code-reviewers --permissions reviews:write comments:write contents:read
but group add-member code-reviewers rust-reviewer
but perm grant --principal rust-implementer contents:write pull_requests:write
but perm list      # verify the effective config
but group list
```

## Verify it's live

After committing `.gitbutler/permissions.toml` + `.gitbutler/gates.toml` to your target
branch:

```bash
but perm list      # principals + their effective authorities
but group list     # groups + members
```

Reopen **Project Settings → Permissions & Governance** — the not-configured state is
replaced by your principals, groups, and branch gates.
