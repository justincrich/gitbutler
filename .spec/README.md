# `.spec/` — Agent Governance & Verification for GitButler

> *Holding agents accountable to the same standards as humans — and making "done" provable.*

GitButler calls itself "Git, *but* better" — "built from the ground up for AI-powered
workflows … a friendlier, more powerful Git replacement, **for you and your agents**."
This directory is my attempt to help finish that sentence.

It holds **one deliverable** — a **governance system**, production-shaped, plus the
artifacts behind it. A meaningful slice of it is **already built, merged, and tested in
GitButler's own crates** — not a proposal on paper. Alongside it sits a second, fully-specced
PRD — a deterministic **check runner** — that is **not part of this submission**: it's the
leading piece of future work, kept here because it's where this layer goes next. I've tried
to be exact about what's built, what's specced, and what's proposed throughout.

```
.spec/
├── prds/
│   ├── governance/          # THE DELIVERABLE — permission + gates over GitButler's own actions (core BUILT)
│   └── check-runner/        # future work — local deterministic checks that gate a change (SPECCED)
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
in. That's what this deliverable builds on GitButler.

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
*enforcement* is trust-based. This deliverable closes the first gap — process — at the source;
the check runner, specced as future work, closes the second.

---

## The deliverable — Governance: hold agents accountable to the same standards as humans

**Full PRD → [`prds/governance/`](./prds/governance/README.md)** · v1.4.0 · 6 functional
groups · 17 audited use cases · 129 acceptance criteria · 13 sprints (8 core + STEER + 4
IDENT). All 13 sprints have task files generated and ACs verified; sprints 01a–11 are
implemented on the `kb/steer-integration` branch.

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
that writes the registry). The IDENT chain (sprints 08–11) is **complete**: the
registry/process engine, `but agent` CLI (register/whoami/list/migrate), the 8 gate callsite
swap to `resolve_principal_with_registry`, env-handle deny-default hardening, the governed
orchestration skills (but-init writes `agents.toml`; but-run-sprint registers subagents
after spawn — no `BUT_AGENT_HANDLE` self-assertion), and the full documentation surface
(`crates/but-authz/README.md`, RULES.md, cross-references) are all landed. A field migration
of the `agent-intel` repo proved the end-to-end registry-path commit. Full spec
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
| Agent identity registry (`but agent` register/whoami/migrate, `agents.toml`) | **Implemented** — engine + CLI + 8-callsite gate swap + env-handle deny-default + orchestration skills + docs, all on `kb/steer-integration`; field-proven on `agent-intel` | `crates/but-authz/src/{registry,process,authorize}.rs`, `crates/but/src/command/agent.rs`, [`crates/but-authz/README.md`](../../crates/but-authz/README.md); [`12-uc-agent-identity.md`](./prds/governance/12-uc-agent-identity.md) |

**What it deliberately does *not* do** (stated plainly, because honesty is part of the
design): it governs GitButler's own `but` actions, **not raw git or the filesystem** — the
fence is a guardrail, not a wall. The local review store is forgeable by a direct DB write
(tracked as risk R6). These aren't hidden; they're exactly why the check runner is the
natural next step.

---

## Architecture — the governance delta

Everything below is the **committed delta from the fork point** (`e44cff5`, the last
upstream `gitbutlerapp/gitbutler` commit before this work) to here: **187 files / ~40k
insertions** across engine, gates, persistence, CLI, and desktop — excluding the `.spec/`
planning corpus. Tests live beside each module (`crates/but-authz/tests/`,
`crates/but-db/tests/`, snapshot fixtures) and are omitted from the tree for signal.
Tags: **[STEER]** capability-aware denials · **[IDENT]** agent identity.

```
crates/
├── but-authz/                          ← NEW crate · the authorization engine (no git/FS I/O)
│   ├── Cargo.toml
│   ├── README.md         [IDENT]      agent identity docs — threat model, file layout, migration path, deprecation timeline
│   └── src/
│       ├── lib.rs                       crate root — re-exports the authz API
│       ├── authority.rs                 permission tokens (contents:write, merge, …) + parse/serialize
│       ├── principal.rs                 principals, groups, ids
│       ├── config.rs                    load agents.toml / gates.toml at the TARGET ref; branch protection
│       ├── authorize.rs                 the authorize() decision + principal resolution + effective authority
│       ├── denial.rs                    structured {code,message,remediation} Denial + steering envelope
│       ├── route.rs        [STEER]      ROUTE_AUTHORITY_TABLE — single source of route → required authority
│       ├── menu.rs         [STEER]      capability catalog + authorized_actions() for can-i / whoami
│       ├── assignment_state.rs [STEER]  gate-state input feeding gate-aware authorized actions
│       ├── registry.rs     [IDENT]      runtime PID registry backed by .gitbutler/agents-runtime.toml
│       └── process.rs      [IDENT]      process-identity primitives (pid, start_time)
│
├── but-api/                            ← gates wired into the existing action boundaries
│   └── src/
│       ├── commit/gate.rs               COMMIT GATE — contents:write + branch protection, fail-closed
│       └── legacy/
│           ├── merge_gate.rs            MERGE GATE — merge authority + review-at-head, self-escalation-proof
│           ├── review_requirement.rs    pure review-at-head evaluator (one approval / required group / head OID)
│           ├── config_mutate.rs         ADMIN-WRITE GATE for perm/group config edits + structured denial
│           └── governance.rs            read-side governance queries for the desktop UI
│
├── but-db/                             ← new persistence · the local review store
│   └── src/table/
│       ├── local_review_verdicts.rs     reviewer verdicts (approve / request-changes) bound to head OID
│       ├── local_review_assignments.rs  reviewer assignments
│       ├── local_review_comments.rs     review comments
│       └── local_review_meta.rs         review metadata
│
├── but/                               ← CLI surface (clap defs in args/, impls in command/)
│   ├── governance-denial-primer.md [STEER]   agent-priming reference shipped in-crate
│   └── src/{args,command}/
│       ├── perm.rs                      `but perm`   — inspect / administer permissions
│       ├── group.rs                     `but group`  — inspect / administer principal groups
│       ├── whoami.rs       [STEER]      `but whoami` — resolve & print the registered principal
│       ├── can_i.rs        [STEER]      `but can-i`  — authority self-check (blocked-agent self-discovery)
│       └── agent.rs       [IDENT]      `but agent`  — runtime register / migrate
│
└── gitbutler-tauri/
    └── src/governance.rs                Tauri IPC command boundary (signed-in fleet-owner identity)

apps/desktop/src/
├── components/governance/              Svelte settings UI
│   ├── GovernanceSettings.svelte        top-level governance settings panel
│   ├── PrincipalsList.svelte            list principals
│   ├── PrincipalEditor.svelte           add / edit a principal's authorities
│   ├── GroupsList.svelte                list groups
│   └── GovernancePendingBanner.svelte   unsaved / pending-change banner
└── lib/governance/
    ├── governanceService.ts             IPC client for the governance commands
    ├── pendingStore.svelte.ts           pending-change store
    └── index.ts

.gitbutler/                             ← committed governance config, read at the target ref
├── agents.toml            [IDENT]      principals/agents, groups, authorities (migrated from permissions.toml)
├── permissions.toml                    legacy fallback (one-release window; `but agent migrate` converts)
└── gates.toml                          commit/merge gate config (branch protection, review reqs)
```

| Layer | New / changed component | What it does | Status |
|---|---|---|---|
| **Engine** (new crate) | `crates/but-authz/` | Pure authorization core — no git/FS I/O. Authorities, principals/groups, config loaded at the target ref, the `authorize()` decision, and the structured `Denial` contract. Callers ask; it answers. | Merged |
| **Commit gate** | `but-api/src/commit/gate.rs` | At the commit boundary: requires `contents:write` and enforces branch protection; fail-closed. | Merged |
| **Merge gate** | `but-api/src/legacy/{merge_gate,review_requirement}.rs` | At the local-merge boundary: requires `merge` authority **plus** a satisfied review-at-head (one approval per required group at the current head OID); self-escalation-proof. | Merged |
| **Review store** | `but-db` `local_review_{verdicts,assignments,comments,meta}` | Persists the reviewer verdicts/assignments/comments the merge gate reads; verdicts bind to a head OID. | Merged |
| **Admin-write gate** | `but-api/src/legacy/config_mutate.rs` | Gates edits to the governance config itself — only authorized principals change the rules; same structured denial. | Merged |
| **Read API** | `but-api/src/legacy/governance.rs` | Read-side governance queries (current config, pending state) for the desktop UI. | Merged |
| **CLI — admin** | `but perm`, `but group` | Inspect / administer permissions and principal groups. | Merged |
| **CLI — self-discovery** [STEER] | `but can-i`, `but whoami`, `governance-denial-primer.md` (+ `route.rs`/`menu.rs`/`assignment_state.rs`) | A blocked agent can ask what it's allowed to do next instead of guessing — authority self-check + identity resolution over the single route-authority table. | Merged |
| **Identity** [IDENT] | `but agent` + `but-authz` `registry.rs`/`process.rs`/`authorize.rs` (`agents.toml`) + orchestration skills + `crates/but-authz/README.md` | Runtime PID registry replacing the self-asserted `BUT_AGENT_HANDLE`. Engine + CLI + 8 gate callsites swapped to `resolve_principal_with_registry` + env-handle deny-default locked + skills migrated (but-init writes `agents.toml`; but-run-sprint registers after spawn) + docs. Field-proven on `agent-intel`. | On `kb/steer-integration` |
| **Desktop** | `gitbutler-tauri/src/governance.rs`; `apps/desktop/src/{components,lib}/governance/*` | Tauri IPC boundary (fleet-owner identity) + Svelte settings UI to view/edit principals & groups. | IPC + read views merged; some edit forms pending |
| **Config** | `.gitbutler/agents.toml`, `.gitbutler/gates.toml` | The committed, ref-pinned source of truth both gates read at the target ref. `agents.toml` supersedes `permissions.toml` (one-release legacy fallback via `but agent migrate`). | Merged |

---

## Where this goes

The deliverable here enforces *process*. The other half of the merge decision — *quality* —
is specced to the same executable depth but **not part of this submission**; it's the first
and most-developed piece of where this layer goes next. Past it, the throughline is bigger
than any single gate.

### Next up — the Check Runner: "done" is proven by re-running, not claimed

**Full PRD → [`prds/check-runner/`](./prds/check-runner/README.md)** · 3 functional groups ·
15 use cases · 88 acceptance criteria.

A butler-controlled, **local, deterministic runner** that executes a configured check
(`cargo test`, `pnpm check`, a repo `./script`) and records a **pass/fail bound to the exact
head commit OID**, which a **new merge-gate clause** consumes to block a change unless every
required check passes at that head. The mental model is exact: **a required check is a second
deterministic review whose verdict a trusted runner produces instead of a human.**

Composed with the governance gates, the merge decision becomes a single check of **process
*and* quality** — both read their config at the target ref, both fail closed, and both speak
the same structured denial contract (`{code, message, remediation_hint}`):

| | Governance (built) | Check Runner (specced) |
|---|---|---|
| Answers | *May* this principal act? Did a human approve? | Did the committed checks really run and pass at this head? |
| Enforces | Process | Quality |

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

### The longer throughline

If I were building this full-time, the arc is bigger than two gates. The loud pains
of agentic engineering — the merge tax, silent overwrites, reward hacking, comprehension
debt, missing audit trails — all cluster at one place: **convergence**, where many streams
of cheap generation must become one verified, attributed, mergeable truth. That plane is
unclaimed, and GitButler's primitives already *are* a convergence engine. Four bets, each
grounded in something the engine already has:

- **From two gates to a governed action surface.** Today enforcement sits on the two acts
  with consequence — commit and merge. The natural extension is a single, enumerable
  route-authority over every `but` action plus capability-aware denials that tell a blocked
  agent its authorized next move (the [STEER][steer] direction, sprint 07 — core merged)
  — turning two checkpoints into a legible map of the whole governed surface, rather than a
  broad action-governance claim shipped today. Agent identity (the [IDENT] chain, sprints
  08–11) is now complete: every governed `but` invocation resolves through a runtime PID
  registry, not a self-asserted env var.
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
| [`prds/governance/`](./prds/governance/README.md) | The deliverable — permissions, groups, commit/merge gates, the governed loop, the management UI |
| [`prds/governance/ROADMAP.md`](./prds/governance/ROADMAP.md) | 13-sprint roadmap (8 core + STEER + 4 IDENT) with per-sprint human-testing gates and review provenance |
| [`prds/governance/enrichments/`](./prds/governance/enrichments/) | STEER — capability-aware denials (sprint 07; core merged) |
| [`prds/governance/12-uc-agent-identity.md`](./prds/governance/12-uc-agent-identity.md) | IDENT — agent identity registration use cases (sprints 08–11 complete: engine, CLI, gates, migration, skills, docs) |
| [`prds/check-runner/`](./prds/check-runner/README.md) | Future work — local deterministic checks + the required-checks merge clause |
| [`artifacts/team-product/`](./artifacts/team-product/04-synthesis-report.md) | The agent-verification definition-of-done, feature inventory, gap analysis, and synthesis |
| [`reviews/`](./reviews/) | Adversarial spec audits |

---

This is the layer I think GitButler is uniquely placed to own, and the direction I'd most
want to keep building.

— Justin Rich

[steer]: ./prds/governance/enrichments/v1.4.0-capability-aware-denials/README.md

[^evidence]: Sources — [Anthropic 2026 Agentic Coding Trends Report](https://resources.anthropic.com/2026-agentic-coding-trends-report) (~60% of work AI-assisted, 0–20% of tasks fully delegated) · [Dan Shapiro, "The Five Levels: from Spicy Autocomplete to the Dark Factory"](https://www.danshapiro.com/blog/2026/01/the-five-levels-from-spicy-autocomplete-to-the-software-factory) · [Swarmia, "Five levels of AI coding agent autonomy"](https://www.swarmia.com/blog/five-levels-ai-agent-autonomy) · ["Vibe coding," Wikipedia](https://en.wikipedia.org/wiki/Vibe_coding) (coined Feb 2025 → 2025 Collins Word of the Year) · [Burak Dede, "The Pull Request is Dead"](https://burakdede.com/blog/the-pull-request-is-dead-surviving-the-ai-code-avalanche) · [CodeRabbit on GitHub's AI-PR caps](https://www.coderabbit.ai/blog/github-gives-maintainers-a-throttle-for-the-ai-pull-request) · [Metacto, "AI Code Review Bottleneck"](https://www.metacto.com/blogs/code-review-bottleneck-ai-development) · [Factory Missions](https://factory.ai/news/missions) · [InfoQ, Claude Code dynamic workflows / `ultracode`](https://www.infoq.com/news/2026/06/dynamic-workflows-claude-code).
