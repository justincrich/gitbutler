---
stability: CONSTITUTION
last_validated: 2026-06-19
prd_version: 1.4.0
enrichment: capability-aware-denials
section: technical-requirements-delta
---

# Technical Requirements Delta — v1.4.0 Capability-Aware Denials

Additive to `10-technical-requirements/`. Backward-compatible for the deny/allow **decision** and for existing field readers. **This delta has been re-grounded against the shipped tree (rust red-hat pass);** the earlier "it already exists / it's free / behavior-neutral" framing was wrong at several points and is corrected here. Real surfaces: `but-authz` (`denial.rs`, `authorize.rs`, `authority.rs`, `config.rs`), `but-api` (`commit/gate.rs`, `legacy/merge_gate.rs`, `legacy/forge.rs`, `config_mutate.rs`, `json.rs`), the **four** hand-rolled CLI denial serializers (`commit_gate_cli_error`, `review_gate_cli_error`, `merge_gate_cli_error`, and `governance_cli_error`), and `but-authz/tests/invariant_build_gates.rs`.

> **Grounding correction (load-bearing).** The shipped `but-authz::Denial` (`denial.rs:13`) has **exactly three fields** — `code`, `message`, `remediation_hint` — and derives `Debug, Clone, PartialEq, Eq` (no `Serialize`). **`unmet` is NOT a `Denial` field; it exists only on `MergeGateError`** (`merge_gate.rs`). `config.invalid` is produced by a **third** type, `ConfigError` (`config.rs`, `thiserror`). So there are **four denial carriers** ($Denial$, $MergeGateError$, $ConfigError$, and the commit gate's two-field $CommitGateError$ wrapper at `commit/gate.rs:9` — surfaced by the rust-planner grounding pass that authored the Sprint 08 roadmap entry; the first draft of this delta said "three"), not one — and the steering fields must be added to all of them.
>
> **Post-complete reconciliation (2026-06-25 red-hat pass).** The "four carriers / three CLI serializers" count above was itself under-counted by the cycle-1 grounding pass. The shipped tree has **SIX denial carriers** (the four above PLUS `ForgeGateError` (`forge.rs:16`) and `AdminWriteGateError` (`config_mutate.rs:6`) — both flatten an underlying `Denial`/`ConfigError` and were caught by RR-1/RR-2 during the planning red-hat loop) and **FOUR CLI serializers** (the three named below PLUS `governance_cli_error` at `perm.rs:118` for admin-write). STEER-001 covers all six carriers; STEER-005 covers all four sites. The §1 table below retains its original three-row shape for historical readability; see `SPRINT.md` "Red-Hat Review Summary" for the authoritative six-carrier list.

## 1. The denial carriers — additive fields on all six

> See the reconciliation note above — the table below lists the three
> carriers enumerated by the original grounding pass; the full shipped
> surface is six (adds `ForgeGateError` + `AdminWriteGateError`), each
> gaining the same four fields via their `classify_error` copies.

| Carrier          | Crate / file                          | Today                                                                                      | Produces                                                                     |
| ---------------- | ------------------------------------- | ------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------- |
| `Denial`         | `but-authz/src/denial.rs:13`          | `code, message, remediation_hint` (derives `Debug,Clone,PartialEq,Eq`)                     | `perm.denied` (missing authority, no-handle, unknown-principal)              |
| `MergeGateError` | `but-api/src/legacy/merge_gate.rs:19` | `code, message, remediation_hint, unmet: Vec<String>` (derives `Serialize, PartialEq, Eq`) | `gate.review_required`, `perm.denied` (merge), `config.invalid` (merge path) |
| `ConfigError`    | `but-authz/src/config.rs`             | `thiserror` struct: `message` + `#[source]` + `code()`                                     | `config.invalid` (authz/commit path)                                         |

Each gains the four additive steering fields:

```rust
pub struct Denial {                       // + the same 4 on MergeGateError; ConfigError gets class + do_not (it has no held/menu)
    // existing (unchanged):
    pub code: &'static str,
    pub message: String,
    pub remediation_hint: String,
    // NEW (v1.4.0):
    pub class: DenialClass,               // ActorCorrectable | OperatorRequired
    pub held_permissions: Vec<Authority>, // effective set; populated ONLY when a principal is resolved (see §3 table)
    pub authorized_actions: Vec<AuthorizedAction>,
    pub do_not: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]    // L2-fix: must derive these so Denial's own derives still hold
pub enum DenialClass { ActorCorrectable, OperatorRequired }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizedAction { pub command: &'static str, pub effect: &'static str }  // catalog-owned &'static str
```

**Serialization (M4-fix).** `Authority` (`authority.rs:10`) does **not** derive `Serialize` today; `Denial` doesn't either. To emit `held_permissions:["reviews:write",…]` the implementer either adds `Serialize` to `Authority` emitting its stable `:` token (matching `Authority::name()`) **or** maps `held.iter().map(Authority::name)` at each serializer. `MergeGateError` already derives `Serialize`, so adding a `Vec<Authority>` field forces one of these choices to compile. `held_permissions` is emitted in a **stable lexical order** (L3-fix) so equality assertions aren't order-flaky. `do_not: Option` serializes with `#[serde(skip_serializing_if = "Option::is_none")]`.

### Serialization sites (C2-fix — there is no single `json::Error` CLI path)

The CLI hand-rolls denial JSON in **three** places; the four new fields must be added to **each**, and two of them already drop `remediation_hint` today (a pre-existing asymmetry — so UC-STEER-01 AC-2's "no regression" means _relative to each site's current output_, not "all sites already emit all fields"):

| Site              | File                                                                 | Emits today                                | Add                                                 |
| ----------------- | -------------------------------------------------------------------- | ------------------------------------------ | --------------------------------------------------- |
| commit gate       | `but/src/command/legacy/commit2.rs:~679` `commit_gate_cli_error`     | `{code, message}`                          | 4 new fields (+ already-missing `remediation_hint`) |
| review/forge gate | `but/src/command/legacy/forge/review.rs:~89` `review_gate_cli_error` | `{code, message}`                          | 4 new fields (+ already-missing `remediation_hint`) |
| merge gate        | `but/src/command/legacy/forge/review.rs:~246` `merge_gate_cli_error` | `{code, message, remediation_hint, unmet}` | 4 new fields                                        |

The **Tauri/MGMT** surface uses a _separate_ serializer, `but-api/src/json.rs` `Error` (emits `code`+`message`). Sprint 06a **`MGMT-IPC-002`** is the task adding `remediation_hint` to that path; STEER's new fields for the desktop surface coordinate there (D6). The CLI and Tauri serializers are distinct — both must be updated for full coverage, or N-API/Tauri steering is explicitly deferred (see §7).

## 2. `class` mapping — exhaustive, by (code, principal-resolution)

`class` is NOT a pure function of `code` alone: `perm.denied` splits by whether a principal was resolved. The mapping is an **exhaustive, non-defaulted match** (M3/PM-fix) — adding a future code/cause without classifying it is a compile break, never a silent `actor_correctable`:

| Denial cause                       | code                   | class                   | menu                              | do_not                                                                                 |
| ---------------------------------- | ---------------------- | ----------------------- | --------------------------------- | -------------------------------------------------------------------------------------- |
| resolved principal lacks authority | `perm.denied`          | `actor_correctable`     | derived (§3)                      | positive-only                                                                          |
| direct commit to protected branch  | `branch.protected`     | `actor_correctable`     | derived (§3, gate-state-aware)    | positive-only                                                                          |
| review requirement unmet           | `gate.review_required` | `actor_correctable`     | derived (§3) + existing `unmet[]` | positive-only                                                                          |
| **no handle / unknown principal**  | `perm.denied`          | **`operator_required`** | **empty**                         | "an operator must register this principal / set BUT_AGENT_HANDLE — do not retry as-is" |
| malformed/incomplete config        | `config.invalid`       | `operator_required`     | empty                             | "do not retry — an operator must fix the committed `.gitbutler` config"                |

The **no-handle / unknown-principal** rows are the C5-adjacent correctness fix (security HIGH #2): such a caller has no resolved authority and **cannot self-correct in-system**, so `operator_required` (not `actor_correctable`) is correct — otherwise the agent loops trying actions it has no authority for.

## 3. `authorized_actions` — gate-state-aware derivation (the load-bearing mechanism, C5-corrected)

The earlier pure-authority intersection was **unsound for `branch.protected`**: branch protection is `authority ∧ ¬protected`, so a caller who hit `branch.protected` _holds_ `contents:write` — a pure `required_authority ⊆ held` would offer the very `commit` that was just denied (a lying menu). The corrected derivation **subtracts the (route, predicate, ref) that actually fired** and binds every entry to a _succeeding context_:

```
authorized_actions(principal, denied={route_d, predicate_d, ref_d}, cfg, target_ref):
    held    = effective_authority(principal, &cfg)          // cfg ALREADY loaded at target_ref by the gate (L1: GovConfig, not a ref)
    usable  = { r ∈ ROUTE_AUTHORITY_TABLE | r.required_authority ⊆ held }
    cands   = AFFORDANCE_MAP[denied]                         // intent categories; each names a route IN A SUCCEEDING CONTEXT
    scoped  = { c ∈ cands | c.route ∈ usable
                            AND c does NOT reproduce (route_d, predicate_d) at ref_d }   // ← the C5 subtraction
    scoped.map(c → CATALOG[c]) ++ [CATALOG[discovery]]
```

- **Gate-state-aware (C5).** For `branch.protected`, the affordance is "commit to a **feature branch**" (a _different_ ref that is not protected) + review actions — never "commit to the protected ref" the caller just hit. The menu offers a route+context that _succeeds_, not the authority-equivalent action that fails the predicate.
- **`effective_authority` is NOT free on the `branch.protected` path (C5).** `branch_protected(principal, branch_name)` (`gate.rs:159`) does not receive `cfg` or the held set (it is dropped on `authorize`'s `Ok` path). A signature change to `branch_protected(principal, &cfg, branch_name)` + a re-call of `effective_authority` is required. Captured in [05 D3](./05-delta-replan.md).
- **Same cfg/ref by construction (M2).** The menu derives from the **exact `cfg` the gate already loaded** at its ref (passed in, not re-loaded) — so menu and gate cannot diverge on config/ref. This is a runtime property (proven by integration test T-STEER-009/024), NOT something a static build-gate can assert (M2-fix).
- **Self-approval excluded (security HIGH #3).** `AFFORDANCE_MAP` for a `branch.protected`/own-branch denial yields `request-changes` + `comment`, never `but review approve` — an L1 exclusion, not left to the primer.

## 4. `ROUTE_AUTHORITY_TABLE` — a real refactor, not "behavior-neutral" (C6-corrected)

There is **no `Route` type and no table today** — each gate calls `authorize(p, Authority::X, &cfg)` at a scattered, heterogeneous site:

| Site        | File                                           | Shape                                                                                                                                     |
| ----------- | ---------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| commit gate | `commit/gate.rs:67`                            | `authorize(p, ContentsWrite)` **then** a separate branch-protection predicate                                                             |
| merge gate  | `legacy/merge_gate.rs:48`                      | `authorize(p, Merge)` **then** the review-requirement engine (min-approvals/distinct/groups)                                              |
| forge       | `legacy/forge.rs:47` `authorize_branch_action` | a **`match` on `Authority`** with arms `ReviewsWrite`/`CommentsWrite`/`PullRequestsWrite` and an `other => authorize(p, other)` catch-all |
| admin write | `config_mutate.rs:25`                          | `authorize(p, AdministrationWrite)`                                                                                                       |

Promoting this to one enumerable `&[(Route, Authority, command, effect)]` is the **single largest piece of work** in the enrichment: it must (a) introduce a `Route` enum that does not exist; (b) keep the **non-authority predicates** (branch protection, review requirement) _out_ of the table but composed around it; (c) reconcile the forge `match` incl. the `other =>` arm into explicit rows; (d) **preserve the shipped honesty build-gate** — `invariant_build_gates.rs` asserts the literal `but_authz::authorize` / `Authority::*` patterns appear in each enforcement file (the `AUTHORITY_POSITIVE_PATTERN` grep); a table-driven helper that hides those literals would break it, so either keep the literal `authorize` calls (table feeds the menu + a coverage assertion) or update the grep too. **Behavior-neutral for the deny/allow _decision_; a genuine multi-site refactor otherwise** — re-budget accordingly. Crate ownership (L4): the table + catalog live in **`but-authz`** (so `authorize`/the menu can use them without a `but-authz → but-api` cycle, RULES.md); the gates in `but-api` consume them.

## 5. The denied-intent → affordance-category map (curated, bounded)

The one curated (not derived) piece, bounded by the ~7 gated routes, maintained beside the table. Each category names a route **in a succeeding context** (per §3):

| denied (cause @ ref)                        | affordance categories (succeeding context)                                                                  |
| ------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| commit to protected branch                  | commit-to-a-**feature**-branch, review (request-changes/comment — **not** approve on own branch), discovery |
| commit, missing `contents:write`            | review, discovery                                                                                           |
| merge, missing `merge`                      | review-status, hand-off, discovery                                                                          |
| merge, `gate.review_required`               | review-status (+ existing `unmet[]`), discovery                                                             |
| submit review, missing `reviews:write`      | comment, discovery                                                                                          |
| admin write, missing `administration:write` | read/inspect, request-config-change, discovery                                                              |

Intent-scoping prevents bloat and the "irrelevant advice confuses agents" failure. A build-gate asserts every `ROUTE_AUTHORITY_TABLE` route has an `AFFORDANCE_MAP` entry that does **not** include the denied route at the denied ref (security LOW #9 — keeps the map from going stale independently of the table).

## 6. Self-discovery (C3-corrected — net-new CLI surface, not shipped)

`but perm list` does **not** exist in the shipped CLI; it is a **Sprint 05 (`CLI-001`) deliverable** (`but perm {list,grant,revoke}`). STEER (Sprint 08, after the roadmap) therefore **depends on** Sprint 05 having landed `but perm list`; it is not "already shipped." The friendlier `but whoami` / `but can-i` bundle (effective perms + groups + authorized-action set) is **net-new CLI work owned by STEER** (D8). Until a discovery verb exists, the discovery affordance is **degradable** (omitted from the menu rather than offering a phantom command — preserving the no-lying-menu invariant). Group/team **provenance** is served here on request (self-scoped, like `but perm list`), never inline in the denial; discovery discloses the caller's own membership but not other members of its groups (security LOW #8).

## 7. Layers (where steering lives)

| Layer                               | Owner                                | Grip            | This delta                                                                                                                      |
| ----------------------------------- | ------------------------------------ | --------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| **L1 — denial contract**            | GitButler (deterministic)            | full            | §1–§6: additive fields on all three carriers + gate-state-aware derivation                                                      |
| **L2 — agent priming**              | harness/orchestrator (probabilistic) | none (Stance 6) | a shippable **reference** primer (UC-STEER-05), non-enforced                                                                    |
| **L3 — traversability + integrity** | GitButler (gates + tests)            | full            | §3 gate-state-aware no-lying-menu + closed catalog, proven by the extended `governed_loop` ([04](./04-e2e-testing-criteria.md)) |

**Caller coverage (PM M1).** The four `but-api` callers (Tauri, `but` CLI, TUI, N-API) consume denials via the serializers in §1. The CLI serializers (3 sites) are STEER's primary target; the Tauri/N-API surface rides `json::Error` and coordinates with `MGMT-IPC-002` (D6). If the Tauri/N-API serializer change is deferred, that is an **explicit out-of-scope decision** ([01-scope-delta](./01-scope-delta.md)), not a silent gap.

## 8. Capability chain

**CAP-STEER-01 — capability-aware denial.** Trigger: any actor-correctable gate/authz denial. Hops: resolve principal → `effective_authority(principal, &cfg)` (cfg = the gate's already-loaded `GovConfig`) → intersect with `ROUTE_AUTHORITY_TABLE` → **subtract the failed (route, predicate, ref)** → intent-scope (excluding self-approve) → render from catalog → serialize at the 3 CLI sites (+ Tauri via `json::Error`/MGMT-IPC-002). Boundary contract: menu derived from the same `cfg`/ref the gate judged against (no divergence). Failure mode: if derivation fails, fall back to the existing fields + exit 1 (§9.5). Real-service proof: extended `governed_loop` (every offered action, in its stated context, succeeds). Owner: `rust-implementer`; reviewers: `rust-reviewer` + `security-auditor`.

## 9. Invariants (non-negotiable)

1. **No lying menu (gate-state-aware).** Every `authorized_actions` entry, run in its stated context, is authorization-**and-predicate**-equivalent to a permit at that context's ref — it never reproduces the (route, predicate, ref) that just failed. Valid **as of the target-ref OID at denial time**; under the ref-pin model a concurrent ref advance may cause a clean re-denial later (expected, not a bug — security MED #4).
2. **Closed catalog.** All `command`/`effect`/`do_not` strings are `&'static str` constants — never `format!`, interpolated, config-sourced, or model-generated.
3. **Pre-existing interpolated fields are an injection surface too.** `message` and `unmet[]` already interpolate config-derived strings (principal/branch names; `gates.toml` `required_groups`, attacker-influenceable per R13). Since v1.4.0 explicitly designs the denial as model-consumed, these are named as **R15** (see [05](./05-delta-replan.md)) with a bounding/sanitization mitigation; they are NOT covered by the closed-catalog grep (which is for the new fields) and must not be claimed closed.
4. **Affordances are disclosure, not orders.** The contract states capabilities; never which to pick (L2 primer reinforces; L1 self-approval exclusion is the one hard carve-out).
5. **Best-effort additive — enforcement never weakens, at derivation AND serialization (security MED #6).** Existing fields render independently of the new ones; a fault deriving or serializing the steering payload still yields `code`/`message`/`remediation_hint` + exit 1 and never turns a deny into an allow or drops the existing fields.
6. **Back-compatible.** Existing fields, codes, exit 1, and each serializer's current key set are preserved (modulo the additive fields + the snapshot updates in [05 D9](./05-delta-replan.md)); whole-object equality tests on `Denial`/`MergeGateError` will see new fields (D9).
7. **Exhaustive class.** §2's (code, resolution) → class mapping is a non-defaulted match; a new code/cause without classification is a build break.
8. **Self-scoped.** `held_permissions` + discovery expose only the caller's own set/memberships; cross-principal disclosure stays gated (Sprint 05).
