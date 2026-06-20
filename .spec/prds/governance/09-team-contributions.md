---
stability: PRODUCT_CONTEXT
last_validated: 2026-06-18
prd_version: 1.3.0
---
# Team Contributions

This PRD was authored from a **brainstorming dialogue** between the governance owner (the human product owner) and the planning lead, then structured by the product-manager-lead pattern. The "phases" below record where the content came from; the **recorded human decisions** are the binding forks the owner resolved during the session — they are the spec's load-bearing choices.

## Phase 1 — Problem framing & user model (product-manager + owner)
- Established the target: hold discrete agentic actors to the **same review/permission gates as humans**, by running governance locally.
- Identified the structural mismatch: a transport-boundary approach (as in the Spoke prior art) owns the boundary, whereas GitButler is a frictionless client with no such boundary — so the work is **manufacturing an enforcement surface** inside GitButler's own actions.
- Defined the illustrative roles as **permission bundles, not enforcement keys** (implementer / reviewer / maintainer), and the trust boundary (orchestrator bounded by outcomes; principals bound action-by-action).

## Phase 2 — Architecture (engineering-manager seat: `rust-planner` + owner)
- Scanned the GitButler crate map; found the **`_with_perm` composition idiom** and the critical distinction that GitButler's `Permission` is a **repo-access lock**, not authz — yielding the naming guardrail (`Authority`/`_with_authz`, orthogonal axis).
- Located the enforcement surface against real source: the commit-gate **narrow-waist** is `but_workspace::commit_engine::create_commit` (covering all mechanisms — `but-action` is one caller, not the gate), and the merge-gate site is the `but-api` PR-merge action (`legacy/forge`) because the trunk ref is immutable locally (`upstream_integration.rs:159`).
- Confirmed the **CLI verb surface** lives in `crates/but/src/args/` (the `but` crate's per-noun modules, e.g. `forge.rs`'s `pr`/`review` verbs) — **not** in `but-clap`, which is a CLI-**docs** generator (`crates/but-clap/src/main.rs` writes a `cli-docs/` dir). The `perm`/`group` nouns and the governed `but pr`/`but review` extensions are added in `crates/but/src/args/`.
- Chose **committed, ref-pinned config** (`.gitbutler/permissions.toml`, `.gitbutler/gates.toml`) over DB tables for governed, reviewable, self-escalation-proof config (committed `.gitbutler/*.toml` is a new GitButler convention; `but-rules` is DB-backed and not a precedent for it).

## Phase 3 — Security / governance soundness (security-auditor seat + owner)
- Surfaced the **accepted-leak fence** explicitly and reframed it via the irrigation thesis (cheapest path, not impossible path) — the POC's philosophical center.
- Hardened the **ref-pin** as the anti-self-escalation contract, extended to **group membership** (the net-new grouping capability's main risk).
- Named the **review-store forgeability** (R6, High) and the **N-API bypass residual** (R14) as accepted-leak honesty items alongside the fence (R1) — the merge gate's review store is forgeable by direct DB write; the enforcement seam binds only callers that route through `but-api`.
- Confirmed the **honesty tests** for the build: never present the fence (or the forgeable review store, or the N-API residual) as a boundary; grep-assert no role-name-in-enforcement and no `Permission` overload.

## Phase 4 — Scope & test posture (product-manager + owner)
- Scoped to **permissions + grouping + two thin gates + the loop demo**; named the deferred layers (Ed25519 review signing + the intermediate HMAC review-integrity check, a full multi-clause gate, auto-run validation, a break-glass override, the steel-trap boundary) so they read as a roadmap, not gaps.
- Set the verification bar: **real `but-authz` + real `but-api` + real git, no mocks**; the LOOP integration test is the proven-reference-flow gate.

## Recorded human decisions (the binding forks)
| # | Decision | Rationale |
|---|---|---|
| D1 | **GitButler IS the forge** (native gates) — a self-contained GitButler feature, not a surface over an external daemon nor a client to a remote forge | One self-contained system; govern locally |
| D2 | **Functional permissions, never roles** — and configurable access for **users and groups** | Unopinionated mechanism; GitHub teams model; the owner's explicit constraint |
| D3 | **The system does not block git**; the fence is client/harness hooks, **accepted-leak** for the POC | "The 10% ignore risk would block the 90% reward"; prove the loop first, steel-trap later |
| D4 | **The system does not drive agent logic** — GitButler is a commit/orchestration policy-enforcement system only | The harness owns the agent; we govern actions, not reasoning |
| D5 | **Irrigation, not a dam** — make the governed path cheaper than the bypass | A goal-directed agent flows toward good code; we grade the riverbed |
| D6 | **Human at the feature level, AI at the code level** — expressed as `gates.toml` config (`require_approval_from_group`), not code | Only humans own quality; the AI gate filters obvious misses before the human; pure config keeps the engine unopinionated |
| D7 | **Write it as a native GitButler feature spec** at `gitbutler/.spec/prds/governance`, grounded in GitButler's own crates; borrow the functional-permission *concept* from prior art, integrate no external package | The change lands in GitButler; the spec lives with the code |
| D8 | **Governance is branching-mechanism-agnostic** — virtual branches/stacks are the default, worktrees are opt-in (`but-worktrees`); the gate applies to the git action across every mechanism | A gate wired to one mechanism is bypassed by another; GitButler has many commit/ref-advancing paths |
| D9 | **Grounded in this repo** — every GitButler technical claim is verified against `gitbutler/crates/` source (trunk immutable, merge is a forge action, commit narrow-waist, `Permission` is a lock, the CLI verbs live in `but/src/args/` and `but-clap` only generates docs), not carried from prior art | "Make sure we're not off base" — code is the source of truth |

## Specialist ownership for the build (downstream)
| Surface | Owner (planner → implementer → reviewer) |
|---|---|
| `but-authz` crate (types, desugar, union, authorize, ref-pinned loader) | `rust-planner` → `rust-implementer` → `rust-reviewer` |
| `but-api` `_with_authz` seam + commit/merge gate wiring + the N-API audit (R14) | `rust-planner` → `rust-implementer` → `rust-reviewer` |
| `but` CLI nouns (`crates/but/src/args/` — `perm`/`group` + governed `pr`/`review`) | `rust-implementer` → `rust-reviewer` |
| Authorization-model soundness, ref-pin, accepted-leak honesty (fence R1 / review-store R6 / N-API R14) | `security-auditor` (adversarial review) |
