# `.spec/` — Agent Governance & Verification for GitButler

> *Holding agents to the same gates as humans — and making "done" provable.*

GitButler calls itself "Git, *but* better" — "built from the ground up for AI-powered
workflows … a friendlier, more powerful Git replacement, **for you and your agents**."
This directory is my attempt to help finish that sentence.

It holds two complementary, production-shaped PRDs and the artifacts behind them. A
meaningful slice of the first one is **already built, merged, and tested in GitButler's
own crates** — not a proposal on paper. The second is specced to the same executable
depth and not yet built. I've tried to be exact about which is which throughout.

```
.spec/
├── prds/
│   ├── governance/          # PRD #1 — permission + gates over GitButler's own actions (core BUILT)
│   ├── check-runner/        # PRD #2 — local deterministic checks that gate a change (SPECCED)
│   └── actions.superseded/  # the earlier, over-scoped design check-runner replaced (archived)
├── artifacts/team-product/  # the doctrine + gap analysis the PRDs answer
└── reviews/                 # adversarial spec audits
```

---

## Why this work

The bottleneck in software has moved. Generation is cheap and abundant; what's scarce now
is **convergence and verification** — reading, trusting, and safely landing what agents
produce. The community evidence is blunt: when several agents work in parallel, a study of 142k
agentic PRs found ~28% hit merge conflicts; frontier models demonstrably reward-hack their
own evals (so "tests pass" is a claim with nothing behind it); and AI-authored PRs carry
markedly more logic defects, pushing the cost onto review.[^evidence]

GitButler is a near-perfect place to absorb that pressure — it already sits at the moment
of convergence, and it already markets itself to agents. But it has two gaps at exactly
the point that matters, the moment code lands:

1. **Process is unenforced.** An agent driving GitButler commits and merges on the same
   footing as the tool's owner. Nothing GitButler enforces says *this principal may not
   merge*, or *this change needs a human's approval first*.
2. **Quality is unverified.** "Tests pass" is prose the agent has no stake in. Nothing in
   the merge decision re-derives it.

My repo's own cross-team analysis names the root cause in one line — **doctrine is not
enforcement** (see [`artifacts/team-product/04-synthesis-report.md`](./artifacts/team-product/04-synthesis-report.md)).
GitButler's verification *bar* is high (it's a Git engine with strict semantics), but its
*enforcement* is trust-based. These two PRDs close that gap from both sides.

---

## Deliverable 1 — Governance: hold agents to the same gates as humans

**Full PRD → [`prds/governance/`](./prds/governance/README.md)** · v1.3.0 · 5 functional
groups · 17 use cases · 129 acceptance criteria · 8 sprints.

A functional, GitHub-mirrored **permission system** (`but-authz`) plus **principal
grouping**, wired into **two thin gates on GitButler's own git actions** — a commit gate
(`contents:write` + branch protection) and a merge gate (`merge` authority + a configurable
review requirement at head) — applied **branching-mechanism-agnostically** (virtual
branches, plain git, opt-in worktrees). Role separation (implementer vs. reviewer vs.
maintainer) **emerges from the permission set**; no enforcement path keys off a role name.

**The thesis — irrigation, not a dam.** You don't harness a river by stopping it; you
channel it. An agent *can* step outside the governed path, but if compliance is cheaper
than defection, a goal-directed agent flows toward good code. We grade the riverbed rather
than cage the water — and hold every actor, human or agent, to the same legible gates at
the only acts with consequence.

**Status — built, merged, and tested (you can read the code):**

| Capability | Status | Where |
|---|---|---|
| `but-authz` engine (`Authority`/`Principal`/`Group`/`Denial`, ref-pinned config loader) | **Merged + tested** | `crates/but-authz/` |
| Commit gate (`contents:write` + branch protection, fail-closed) | **Merged + tested** | `crates/but-api/src/commit/gate.rs` |
| Merge gate (`merge` authority + review-at-head, self-escalation-proof) | **Merged + tested** | `crates/but-api/src/legacy/merge_gate.rs`; `local_review_verdicts` in `but-db` |
| CLI: `but perm` / `but group` | **Merged** | `crates/but/src/args/{perm,group}.rs` |
| Desktop governance UI (Tauri IPC + settings scaffold) | **In progress** — IPC merged; principal/group/branch-gate forms pending | `apps/desktop/.../governance/` |

**What it deliberately does *not* do** (stated plainly, because honesty is part of the
design): it governs GitButler's own `but` actions, **not raw git or the filesystem** — the
fence is a guardrail, not a wall. The local review store is forgeable by a direct DB write
(tracked as risk R6). These aren't hidden; they're the reason Deliverable 2 exists.

---

## Deliverable 2 — Check Runner: "done" is proven by re-running, not claimed

**Full PRD → [`prds/check-runner/`](./prds/check-runner/README.md)** · 3 functional groups ·
15 use cases · 88 acceptance criteria.

A butler-controlled, **local, deterministic runner** that executes a configured check
(`cargo test`, `pnpm check`, a repo `./script`) and records a **pass/fail bound to the exact
head commit OID**, which a **new merge-gate clause** consumes to block a change unless every
required check passes at that head. The mental model is exact: **a required check is a second
deterministic review whose verdict a trusted runner produces instead of a human.**

**The thesis.** The gate never reads the agent's "tests pass" — it reads a stored result the
runner produced at the current head. Security rests on **reproducibility** (a forged green is
caught by re-running) plus cheap structural locks (the runner is not the agent; there's no
API to supply a conclusion; results are SHA-bound) — **not** on cryptography. The honest path
is the cheapest path for an agent that optimizes for least resistance.

**Status — specced to executable depth, not yet built.** No `but-checks` crate or
`check_results` table exists yet; named dependencies (a governance steering carrier and a
mechanism-agnostic local-merge entry point) and the #1 risk (a clean head-OID checkout in
GitButler's one-worktree model) are written down, not glossed over.

**A judgment note.** This PRD *supersedes* an earlier one
([`actions.superseded/`](./prds/actions.superseded/README.md)) that I scoped **down**: that
design tried to make results cryptographically "agent-non-forgeable" (signed ledger, HMAC →
Ed25519, sandboxed executor). Under the real personal-tenant threat model — the agent shares
the OS user with the runner — signing can't actually close forgery, and a reproducible check
doesn't need it to. Cutting a feature to fit the threat model is itself part of the work.

---

## How the two compose

They are one system, not two features. The merge decision becomes a single check of
**process *and* quality**:

| | Governance | Check Runner | STEER (governance enrichment) |
|---|---|---|---|
| Answers | *May* this principal act? Did a human approve? | Did the committed checks really run and pass at this head? | What is the agent's next legal move? |
| Enforces | Process | Quality | Redirection |

Both read their config at the target ref, both fail closed, and both speak the same
structured denial contract (`{code, message, remediation_hint}`). [STEER][steer] then makes
every denial name the agent's authorized next action — irrigation applied at the moment of
denial, so a blocked agent self-corrects instead of thrashing.

---

## Where this goes

If I were building this full-time, the throughline is bigger than two gates. The loud pains
of agentic engineering — the merge tax, silent overwrites, reward hacking, comprehension
debt, missing audit trails — all cluster at one place: **convergence**, where many streams
of cheap generation must become one verified, attributed, mergeable truth. That plane is
unclaimed, and GitButler's primitives already *are* a convergence engine. Three bets, each
grounded in something the engine already has:

- **Conflict-free parallel convergence.** N agents → N virtual branches over *one* working
  tree instead of N worktrees — no merge tax, no disk blow-up. Hunk-assignment already knows
  who owns what, so GitButler can *predict* a collision before it becomes a silent overwrite;
  `but-graph` + `but-rebase`'s editor can compute a safe integration order. No competitor can
  say "hunk-level ownership on one tree" without rebuilding GitButler.
- **A glass-box provenance & economics ledger.** `but-agentlog` already captures agent
  transcripts; the oplog already captures every state delta. Join them into a per-change
  *receipt* — prompt → diff → review → merge → token cost — and you get auditability and a
  real "cost per *shipped* feature" instead of cost per token.
- **Durable memory across context rot.** An agent's context is ephemeral; the stack, oplog,
  and agentlog are not. GitButler can be the ground-truth state a *fresh* agent boots from
  when the old one's context degrades — fresh starts over salvage.

The model stays invariant — hunk-ownership, receipts, merge-order intelligence — while the
projection scales from one excellent local working tree toward a cross-machine fleet.

---

## How this was built

- **Grounded in live GitButler source**, not greenfield — every gate is sited at a real
  narrow-waist (`commit_engine`, the `but-api` PR-merge action), reusing GitButler's own
  crates and patterns.
- **Adversarially reviewed.** Each sprint and task set went through red-hat review cycles and
  a fakeability audit; the review records live in [`reviews/`](./reviews/) and each sprint's
  provenance is in [`prds/governance/ROADMAP.md`](./prds/governance/ROADMAP.md).
- **Tested against real services, no mocks** — real `but-authz`, real git, real `but-db`;
  e.g. the merge gate's self-escalation proof runs against the actual ref-pinned config.
- **Honest about limits.** Accepted leaks (R6, R14, the fence) are documented as limits, never
  dressed up as boundaries — and the doctrine they answer is written down first
  ([`artifacts/team-product/`](./artifacts/team-product/01-definition-of-done.md)).

---

## Map

| Path | What it is |
|---|---|
| [`prds/governance/`](./prds/governance/README.md) | PRD #1 — permissions, groups, commit/merge gates, the governed loop, the management UI |
| [`prds/governance/ROADMAP.md`](./prds/governance/ROADMAP.md) | 8-sprint roadmap with per-sprint human-testing gates and review provenance |
| [`prds/governance/enrichments/`](./prds/governance/enrichments/) | STEER — capability-aware denials (planned) |
| [`prds/check-runner/`](./prds/check-runner/README.md) | PRD #2 — local deterministic checks + the required-checks merge clause |
| [`artifacts/team-product/`](./artifacts/team-product/04-synthesis-report.md) | The agent-verification definition-of-done, feature inventory, gap analysis, and synthesis |
| [`prds/actions.superseded/`](./prds/actions.superseded/README.md) | The earlier over-scoped design, archived for the record |
| [`reviews/`](./reviews/) | Adversarial spec audits |

---

This is the layer I think GitButler is uniquely placed to own, and the direction I'd most
want to keep building.

— Justin Rich

[steer]: ./prds/governance/enrichments/v1.4.0-capability-aware-denials/README.md

[^evidence]: Figures synthesized from external research — AgenticFlict (142k-PR study), METR/Abundant reward-hacking findings, and CodeRabbit AI-PR defect rates.
