# `.spec/` — Agent Governance & Verification for GitButler

> *Holding agents accountable to the same standards as humans — and making "done" provable.*

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
│   └── check-runner/        # PRD #2 — local deterministic checks that gate a change (SPECCED)
├── artifacts/team-product/  # the doctrine + gap analysis the PRDs answer
└── reviews/                 # adversarial spec audits
```

---

## Why this work

**The timing.** Fully-agentic engineering — hand off a task and *don't read the diff* — is
still a minority today. [Anthropic's 2026 Agentic Coding Trends Report](https://resources.anthropic.com/2026-agentic-coding-trends-report)
finds developers use AI in ~60% of their work but **fully delegate only 0–20% of tasks**;
roughly 90% of "AI-native" developers still sit at the [pairing level](https://www.danshapiro.com/blog/2026/01/the-five-levels-from-spicy-autocomplete-to-the-software-factory),
a human reviewing every change. But that frontier is climbing the same curve
[vibe coding](https://en.wikipedia.org/wiki/Vibe_coding) did — a niche coinage in early 2025,
mainstream within a year. As the "don't-look-at-the-code" cohort grows over the coming months,
one thing hasn't kept up: **there is no standard for how a *team* of agents is governed.**

**The bottleneck moved.** Generation is cheap and abundant; what's scarce is **convergence and
verification** — reading, trusting, and safely landing what agents produce. The human "LGTM"
is now the constraint: AI-assisted teams [merge far more PRs while review time and PR size
balloon](https://www.metacto.com/blogs/code-review-bottleneck-ai-development), and GitHub itself
shipped [PR kill-switches (Feb 2026) and per-contributor PR caps (Jun 2026)](https://www.coderabbit.ai/blog/github-gives-maintainers-a-throttle-for-the-ai-pull-request)
to stem the flood of agent-generated changes. The [PR-and-review model](https://burakdede.com/blog/the-pull-request-is-dead-surviving-the-ai-code-avalanche)
was built for humans writing code slowly; under agent volume it buckles.[^evidence]

**Don't reinvent the wheel — and don't make it a black box.** The reflex is to reach for a
proprietary orchestrator — [Factory's Missions](https://factory.ai/news/missions), Claude Code's
[`ultracode`](https://www.infoq.com/news/2026/06/dynamic-workflows-claude-code) — that spawns
workers, validators, and subagents inside an opaque runtime. Those deliver autonomy, but they
hide *how the team of agents works and is governed.* We already have a battle-tested process for
shipping production code with many contributors: GitHub — functional permissions, review, branch
protection, an auditable trail. The missing piece isn't another black box; it's a **governed
convergence layer between the agents and the human review process** — where many agents' work is
held to the *same standards as humans* and made legible *before* it floods the queue humans work
in. That's what these two PRDs build on GitButler.

GitButler is a near-perfect place to put that layer — it already sits at the moment of
convergence, and it already brands itself "for you and your agents." But it has two gaps at
exactly the point that matters, the moment code lands:

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

## Deliverable 1 — Governance: hold agents accountable to the same standards as humans

**Full PRD → [`prds/governance/`](./prds/governance/README.md)** · v1.4.0 · 6 functional
groups · 17 audited use cases · 129 acceptance criteria · 13 sprints (8 core + STEER + 4
IDENT; the IDENT group's use cases are specced, its criteria still in progress).

A functional, GitHub-mirrored **permission system** (`but-authz`) plus **principal
grouping**, wired into **two thin gates on GitButler's own git actions** — a commit gate
(`contents:write` + branch protection) and a merge gate (`merge` authority + a configurable
review requirement at head) — applied **branching-mechanism-agnostically** (virtual
branches, plain git, opt-in worktrees). Role separation (implementer vs. reviewer vs.
maintainer) **emerges from the permission set**; no enforcement path keys off a role name.

A sixth functional group — **agent identity registration (IDENT)** — replaces the
self-asserted `BUT_AGENT_HANDLE` environment variable with a runtime PID registry anchored
in committed `.gitbutler/agents.toml`, so every governed `but` invocation resolves to a
registered `(pid, start_time)` instead of whatever an agent claims to be. Identity here is
process-level, **not cryptographic** (the trust root is the host OS plus the orchestrator
that writes the registry); it is **specced across sprints 08–11, not yet built**. Full spec
→ [`12-uc-agent-identity.md`](./prds/governance/12-uc-agent-identity.md).

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
| Agent identity registry (`but agent register/whoami/migrate`, `agents.toml`) | **Specced** — sprints 08–11, not yet built | [`12-uc-agent-identity.md`](./prds/governance/12-uc-agent-identity.md); planned in `crates/but-authz/` |

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

**A judgment note.** An earlier draft of this design chased a cryptographic
"agent-non-forgeable" guarantee (signed ledger, HMAC → Ed25519, sandboxed executor). Under
the real personal-tenant threat model — the agent shares the OS user with the runner —
signing can't actually close forgery, and a reproducible check doesn't need it to. I scoped
it down; cutting a feature to fit the threat model is part of the work.

---

## How the two compose

They are one system, not two features. The merge decision becomes a single check of
**process *and* quality**:

| | Governance | Check Runner |
|---|---|---|
| Answers | *May* this principal act? Did a human approve? | Did the committed checks really run and pass at this head? |
| Enforces | Process | Quality |

Both read their config at the target ref, both fail closed, and both speak the same
structured denial contract (`{code, message, remediation_hint}`). (A third column — denials
that name an agent's authorized next move, the [STEER][steer] direction — is specced, not
delivered; it lives under [future work](#where-this-goes).)

---

## Where this goes

If I were building this full-time, the throughline is bigger than two gates. The loud pains
of agentic engineering — the merge tax, silent overwrites, reward hacking, comprehension
debt, missing audit trails — all cluster at one place: **convergence**, where many streams
of cheap generation must become one verified, attributed, mergeable truth. That plane is
unclaimed, and GitButler's primitives already *are* a convergence engine. Four bets, each
grounded in something the engine already has:

- **From two gates to a governed action surface.** Today enforcement sits on the two acts
  with consequence — commit and merge. The natural extension is a single, enumerable
  route-authority over every `but` action plus capability-aware denials that tell a blocked
  agent its authorized next move (the [STEER][steer] direction, already specced) — turning
  two checkpoints into a legible map of the whole governed surface, rather than a broad
  action-governance claim shipped today.
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
| [`prds/governance/ROADMAP.md`](./prds/governance/ROADMAP.md) | 13-sprint roadmap (8 core + STEER + 4 IDENT) with per-sprint human-testing gates and review provenance |
| [`prds/governance/enrichments/`](./prds/governance/enrichments/) | STEER — capability-aware denials (specced, sprint 07) |
| [`prds/governance/12-uc-agent-identity.md`](./prds/governance/12-uc-agent-identity.md) | IDENT — agent identity registration use cases (specced, sprints 08–11) |
| [`prds/check-runner/`](./prds/check-runner/README.md) | PRD #2 — local deterministic checks + the required-checks merge clause |
| [`artifacts/team-product/`](./artifacts/team-product/04-synthesis-report.md) | The agent-verification definition-of-done, feature inventory, gap analysis, and synthesis |
| [`reviews/`](./reviews/) | Adversarial spec audits |

---

This is the layer I think GitButler is uniquely placed to own, and the direction I'd most
want to keep building.

— Justin Rich

[steer]: ./prds/governance/enrichments/v1.4.0-capability-aware-denials/README.md

[^evidence]: Sources — [Anthropic 2026 Agentic Coding Trends Report](https://resources.anthropic.com/2026-agentic-coding-trends-report) (~60% of work AI-assisted, 0–20% of tasks fully delegated) · [Dan Shapiro, "The Five Levels: from Spicy Autocomplete to the Dark Factory"](https://www.danshapiro.com/blog/2026/01/the-five-levels-from-spicy-autocomplete-to-the-software-factory) · [Swarmia, "Five levels of AI coding agent autonomy"](https://www.swarmia.com/blog/five-levels-ai-agent-autonomy) · ["Vibe coding," Wikipedia](https://en.wikipedia.org/wiki/Vibe_coding) (coined Feb 2025 → 2025 Collins Word of the Year) · [Burak Dede, "The Pull Request is Dead"](https://burakdede.com/blog/the-pull-request-is-dead-surviving-the-ai-code-avalanche) · [CodeRabbit on GitHub's AI-PR caps](https://www.coderabbit.ai/blog/github-gives-maintainers-a-throttle-for-the-ai-pull-request) · [Metacto, "AI Code Review Bottleneck"](https://www.metacto.com/blogs/code-review-bottleneck-ai-development) · [Factory Missions](https://factory.ai/news/missions) · [InfoQ, Claude Code dynamic workflows / `ultracode`](https://www.infoq.com/news/2026/06/dynamic-workflows-claude-code).
