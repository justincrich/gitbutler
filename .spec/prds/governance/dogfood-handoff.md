# Governance Dogfood — Session Handoff

> **Purpose:** let a fresh session pick up the GitButler-agent-governance dogfood
> without re-investigating. What was done, what's verified, which projects are set
> up, what's open, and how to watch the governed flow run for real.
> **Last updated:** 2026-06-28. **Owner:** Justin (with Claude).

## TL;DR
- Agent identity (`BUT_AGENT_HANDLE`, env-primary) is wired across **Claude Code,
  Codex, OpenCode** via the git→but steerer (hook-driven, un-forgeable). Both
  **commit and merge** are now governed on the engine + the global `but`.
- The **local `but merge` gate** was the one real defect found + fixed this workstream
  (it did a plain merge with no gate; now enforces identity + merge authority + review).
- The **global `~/bin/but`** is the **release build** with the merge gate, deployed +
  live-verified. Affected-crate tests green; `rust-reviewer` verdict SHIP.
- **2 of ~5 but-touched projects are fully set up** (agent-intel, fabrio); the rest
  have gaps (see matrix). Dogfood by running `/but-run-sprint` from a session
> **launched inside the project** (the steerer hook is project-scoped).

---

## Workstream 1 — env-primary identity across the 3 harnesses
**Problem:** identity was the dead PID-registry model in the installed skill assets;
the harness layer wasn't actually injecting/enforcing `BUT_AGENT_HANDLE`.

**What shipped (canonical = `brain/skills/`, mirrored to `~/.claude`, `~/.config/opencode`, `~/.codex`):**
- `steerer_core.identity_verdict` — a subagent (CC/Codex) running a governed `but` verb
  must carry `BUT_AGENT_HANDLE=<subagent_type>`; missing or forged → denied at the hook.
- `handle_check_core.py` — rewritten from dead `but agent whoami` to **env-primary**
  SessionStart advisory (reads `BUT_AGENT_HANDLE` vs committed roster).
- `claude_steerer.py` / `codex_steerer.py` — PreToolUse match-enforcement.
- `opencode-plugin.js` — `shell.env` injection (host-set, un-forgeable).
- `but-run-sprint` prompts + `BUT-SKILL-CONVENTIONS.md` — harness-correct identity guidance.

**Trust model:** the hook stamps identity from harness metadata (CC/Codex `agent_type`;
OpenCode `shell.env`) — **never** the orchestrator-as-agent (forgery collapses per-agent
permissions). The orchestrator only picks *who* to dispatch (`subagent_type` == principal id).

**Steerer tests:** `python3 but-init/assets/git-but-steerer/tests/test_steerer.py` (72) +
`test_handle_check.py` (38). Standalone scripts (not pytest).

## Workstream 2 — govern local `but merge` (the real defect)
**Problem:** `crates/but/src/command/legacy/merge.rs::handle` did a plain
`repo.merge_commits` with **no gate**. An implementer (no merge authority) could merge
locally. GOV-LOCAL task #1020 *claimed* this was wired — it never was (no
`enforce_local_merge_gate` existed). The forge/PR path was gated; the local path was not.

**Fix (commit `b2423b7e01` on `kb/steer-integration`, branch NOT merged to `master`):**
- Refactored `enforce_merge_gate` to share post-config enforcement
  (identity → `Route::Merge` authority → branch protection → review requirement) in
  `enforce_merge_gate_with_author`; forge path is a thin **behavior-neutral** wrapper.
- Added forge-less `enforce_local_merge_gate(target_ref, source_ref)` — `governance_present`
  ungoverned short-circuit (→ allow, NOT config.invalid) + head-pinned `local_review_verdicts`.
- Wired into `merge.rs::handle` (gb-local arm, before `repo.merge_commits`).
- Added `merge.rs` to `invariant_build_gates` `ENFORCEMENT_PATHS` (regression grep — the
  teeth that would have caught the never-wired gap).
- New `crates/but-api/tests/local_merge_gate.rs` (7-case matrix).

**Verified:** affected-crate suite green (`but-api`, `but`, `but-authz` — 0 failures);
`rust-reviewer` = **SHIP** (forge-neutral confirmed, no critical, no bypass).

**Known limitation (honest):** no commit-author→`PrincipalId` mapping exists, so
`require_distinct_from_author` is **skipped for local merges** (author=`None`). Identity +
merge authority + approval count + branch protection fully enforce; distinctness is a follow-up.

## The global `but` binary
- `~/bin/but` = **release build** (`cargo build --release -p but`), 61 MB, has the merge gate.
- Live-verified via PATH `but`: implementer `but merge` → `perm.denied: action requires merge`;
  no-handle → `perm.denied: BUT_AGENT_HANDLE is required`.
- To rebuild after engine changes: `cargo build --release -p but && cp target/release/but ~/bin/but`.
- ⚠️ **`b2423b7e01` is on `kb/steer-integration`, not `master`.** Your global binary has it;
  upstream `master` doesn't. Merge to `master` (or a PR) for the shared product.

## Verified protection (commit + merge, both gates)
Run: `cargo test -p but-api --test commit_gate --test local_merge_gate --test merge_gate`
- `commit_gate.rs` → **14 pass** (identity, contents:write authority, protected-branch, ungoverned→allow, forgery/DryRun).
- `local_merge_gate.rs` → **7 pass** (no-handle/unknown/read-only/implementer-no-merge denied; maint-no-approval `review_required`; maint-with-approval allow; ungoverned allow).
- `merge_gate.rs` → **15 pass** (forge/PR path — proves the refactor is behavior-neutral).

Both gates deny unauthorized + allow authorized, against real fixtures (real git repos,
real governance TOML, real `but-api` gates) — not mocks.

## Project setup matrix (`~/Projects`)
| Project | Governance (committed) | `but setup` | Steerer | Status |
|---|---|---|---|---|
| **agent-intel** | ✅ | ✅ `gitbutler/workspace` | ✅ current | **Fully set up — primary dogfood** |
| **fabrio** | ✅ | ✅ `gitbutler/workspace` | ✅ | **Fully set up** |
| **mega-button** | ❌ none | ✅ | ✅ | Ungoverned — run `/but-init` to seed governance |
| **LaneShadow-RN** | ✅ | ⚠️ on `main` | ✅ | Stale setup — run `/but-migrate` (has governance; gentle reconcile) |
| **gitbutler** (this repo) | ✅ | ⚠️ on `kb/steer-integration` | ✅ (untracked) | **Don't `but-init` mid-development** — it'd switch to `gitbutler/workspace` and disrupt the branch. Tool's own source; not a dogfood target. |
| brain, rml, ship-commander, mortal-context, ruthwell, career, … | — | — | — | Not `but`-initialized. `/but-init` the ones you want governed. |

`but-init` = fresh bootstrap (registers + scaffolds + hooks, idempotent).
`but-migrate` = existing governance/.spec, additive (never rewrites). Both project-scoped.

## How to dogfood + watch
The governed land flow is: **implementer commits (`but commit`) → reviewer approves
(`but review approve`) → orchestrator merges (`but merge`)** — each as its principal,
each gate enforcing.

**To watch it for real:**
1. Open a Claude Code / Codex / OpenCode session **launched from inside a governed
   project** (agent-intel or fabrio). ⚠️ The steerer hook is **project-scoped** — from a
   session in a different dir the hook does NOT fire on subagents, so identity isn't
   enforced at the harness layer (the engine gate still fires, but anti-forgery doesn't).
2. Run `/but-run-sprint <sprint-id>` (agent-intel has `sprint-01-walking-skeleton-spike`).
3. **Watch for:**
   - Each dispatched subagent's `but` commands carry its `BUT_AGENT_HANDLE`
     (CC/Codex: prefix; OpenCode: shell.env injects).
   - A governed commit lands **attributed to the acting subagent principal**.
   - Forgery (subagent claims another principal) → hook **denies** at PreToolUse.
   - An implementer attempting `but merge` → **denied** (no merge authority).
   - `but merge` without a distinct `but review approve` → **`gate.review_required`**.

**Quick gate spot-checks (no full sprint):** use `/tmp/gov-verify.sh` (builds a clean
governed repo, exercises the commit+merge matrix) — re-run after any engine change.

## Open follow-ups (prioritized)
1. **`but merge` source-ref governance read** — the CLI merge path additionally reads
   `gates.toml` from the **source** branch (emits `config.invalid` in virtual-branch
   fixtures without inherited governance). The gate *function* reads the target ref
   correctly; this is a separate read in the merge path. Likely worth tracing — may be a
   latent bug. (Unblocks a CI CLI test for local merge.)
2. **`require_distinct_from_author` for local merge** — needs a commit-author→PrincipalId
   mapping (currently `None`; distinctness skipped). Identity + authority + approvals still enforce.
3. **Merge `kb/steer-integration` → `master`** — the merge-gate fix is on the feature branch.
4. **`mega-button` governance + `LaneShadow-RN` reconcile** — run `/but-init` / `/but-migrate`.
5. **2 cosmetic clippy `needless_borrow` warnings** in `merge_gate.rs` (false-positive on
   Deref-through-ref; clippy `--fix` declined). No pre-commit hook in gitbutler, so non-blocking.
6. **Pre-existing clippy error** in `crates/but-api/src/legacy/forge.rs` (JsonSchema trait) —
   unrelated to this work; doesn't block the build.

## Key references
- **Merge-gate fix:** `b2423b7e01` (gitbutler, `kb/steer-integration`).
- **Identity harness (brain):** `30459e8` (gap fix), `485e86d` (adapters), `89f0006` (doc reconcile).
- **Engine:** `crates/but-authz/src/authorize.rs` (`resolve_principal_from_env`),
  `crates/but-api/src/commit/gate.rs` (commit gate), `crates/but-api/src/legacy/merge_gate.rs`
  (`enforce_merge_gate` + `enforce_local_merge_gate`),
  `crates/but/src/command/legacy/merge.rs` (wiring).
- **Tests:** `crates/but-api/tests/{commit_gate,local_merge_gate,merge_gate}.rs`,
  `crates/but-authz/tests/invariant_build_gates.rs`.
- **Steerer (canonical):** `brain/skills/but-init/assets/git-but-steerer/` (mirrors in
  `~/.claude`, `~/.config/opencode`, `~/.codex` under `skills/but-init/...`).
- **Docs:** `brain/skills/but-run-sprint/SKILL.md` (governed-substrate), `~/.claude/docs/BUT-SKILL-CONVENTIONS.md`.
- **Verification harness:** `/tmp/gov-verify.sh` (clean-repo commit+merge matrix; recreate if gone).
- **Agent-intel governance:** `~/Projects/agent-intel/.gitbutler/{agents,gates}.toml` + `.gitbutler-steerer/`.
- **Memory:** `gitbutler-identity-harness-wiring.md` (this project's memory dir).
