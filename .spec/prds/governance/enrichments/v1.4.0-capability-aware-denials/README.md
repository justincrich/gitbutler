---
title: Capability-Aware Denials (STEER) — Governance PRD Enrichment v1.4.0
version: 1.4.0
enriches: .spec/prds/governance/README.md
from_version: 1.3.0
status: planned (net-additive; sprints 01a–06b frozen)
scope_posture: net-additive progression
pr_sequencing: false
---

# Capability-Aware Denials (STEER) — Enrichment v1.4.0

A **net-additive** enrichment of the Functional-Permission Agent Governance PRD. It adds one functional group — **STEER** — that turns every actor-correctable governance denial into a _flow-diverter_: alongside _why it failed_, the denial returns, in a standard machine-readable shape, _what this principal CAN do right now_ — so a goal-directed AI agent diverts down the governed path instead of hard-quitting, looping, or bypassing.

> **Irrigation, at the moment of denial.** Today's denials point _up and out_ ("ask a maintainer") — where the water pools and overflows (the agent quits or defects). A capability-aware denial points _down and across_ ("here's what you can do now") — so the water always finds a channel. Each rejection is a spillway, not a wall. This is Stance 0 applied to the one place the river is most likely to overflow.

## Status — frozen-aware

- **Net-additive.** Changes no existing scope, gate decision, code, or denial code. Adds fields to the denial _payload_ only.
- **Sprints 01a–06b are FROZEN** (in-flight agents). This enrichment **edits none of them**. Implied changes to shipped code and to the frozen PRD index files are recorded as deltas in [05-delta-replan.md](./05-delta-replan.md), to apply when the freeze lifts.
- **Lands as a new Sprint 08 (STEER)**, appended after the roadmap — UI-independent (MGMT rendering of the menu is deferred).
- **Research-grounded.** holocron `js7bqec1xmff8drgx35xav3xvs88ykcp` — "Error Messages as Steering Prompts for AI Coding Agents" (HIGH confidence).

## Document Index

| File                                                                       | Section                                                                                                                        | Stability       |
| -------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ | --------------- |
| [00-overview.md](./00-overview.md)                                         | Irrigation-at-the-denial framing; the three failure modes (hard-quit / loop / bypass)                                          | PRODUCT_CONTEXT |
| [01-scope-delta.md](./01-scope-delta.md)                                   | In scope / out of scope (net-add); what is preserved from sprints 1–8                                                          | FEATURE_SPEC    |
| [02-uc-steer.md](./02-uc-steer.md)                                         | STEER functional group + UC-STEER-01..06 (32 ACs)                                                                              | FEATURE_SPEC    |
| [03-technical-requirements-delta.md](./03-technical-requirements-delta.md) | Additive fields on all 3 denial carriers, gate-state-aware derivation, the route→Authority single-source, invariants, L1/L2/L3 | CONSTITUTION    |
| [04-e2e-testing-criteria.md](./04-e2e-testing-criteria.md)                 | T-STEER-001..031 (real services; the gate-state-aware no-lying-menu proof)                                                     | TEST_SPEC       |
| [05-delta-replan.md](./05-delta-replan.md)                                 | Sprint 1–8 code deltas (D1–D10) + risks (R15–R17) + proposed Sprint 08 + frozen-file integration edits (I1–I6)                 | —               |

## The contract at a glance

Additive to the shipped denial carriers — `Denial { code, message, remediation_hint }` and `MergeGateError` (which also carries `unmet`); both preserved. Strings below are illustrative (v1.4.0 changes no shipped `message`/`remediation_hint` text):

```jsonc
{
	"error": {
		"code": "branch.protected",
		"class": "actor_correctable",
		"message": "direct commits to protected 'main' are denied for principal 'rev'",
		"remediation_hint": "land 'main' via a reviewed merge", // vertical (original goal)
		"held_permissions": ["reviews:write", "comments:write"], // effective set (own ∪ groups)
		"authorized_actions": [
			// lateral menu, derived + intent-scoped (no self-approve)
			{
				"command": "but review request-changes",
				"effect": "reject this change with line comments",
			},
			{
				"command": "but perm list",
				"effect": "see full permissions, groups, and authorized actions",
			},
		],
		"do_not": "do not commit directly or bypass with raw git — protected refs only move via reviewed merge",
	},
}
```

> `unmet[]` is a `MergeGateError` field (merge `gate.review_required` denials), not a `Denial` field — absent from this commit-path example.

## Quick-Stats Delta (v1.3.0 → v1.4.0)

| Metric              | v1.3.0 | Δ                                  | v1.4.0 (after integration) |
| ------------------- | ------ | ---------------------------------- | -------------------------- |
| Functional Groups   | 5      | +1 (STEER)                         | **6**                      |
| Use Cases           | 17     | +6                                 | **23**                     |
| Acceptance Criteria | 129    | +32 (STEER)                        | **161**                    |
| Testing Criteria    | 129    | +31 (T-STEER; one row spans 2 ACs) | **160**                    |
| Risk register       | 14     | +3 (R15/R16/R17, named)            | **17**                     |
| ROADMAP sprints     | 8      | +1 (Sprint 08)                     | **9**                      |

Counts reconcile in [05-delta-replan.md §4](./05-delta-replan.md). Per-AC criterion coverage: 32/32 ACs have ≥1 T-STEER criterion (31 criterion rows; T-STEER-023 spans 2 ACs).

## Integration points (apply when the freeze lifts)

Summarized from [05-delta-replan.md §3](./05-delta-replan.md) — all **append-style**, no rewrites of frozen files:

1. **I1** — promote `02-uc-steer.md` → top-level `12-uc-steer.md` (no renumbering).
2. **I2** — add the STEER rows to `03-functional-groups.md` (+ totals).
3. **I3** — `README.md`: Document Index row, Quick Stats, Version History, `version: 1.4.0`.
4. **I4** — `ROADMAP.md`: Sprint 08 row + details + dependency edge; `sprint_count` 8→9. **✅ APPLIED 2026-06-19** — Sprint 08 (rust-planner-authored, NEVER-TIER) is now on the roadmap; I1–I3 / I5–I6 stay deferred until the freeze lifts.
5. **I5** — fold T-STEER criteria into `11-e2e-testing-criteria.md` (+ count line → 160).
6. **I6** — fold the named risks R15/R16/R17 into `10-technical-requirements/07-technical-risks.md` (risk count 14 → 17).

And the code deltas D1–D10 (steering fields on all 3 carriers, gate-state-aware derivation + `branch_protected` signature change, the 3 CLI serializers, the `ROUTE_AUTHORITY_TABLE` refactor, the `but whoami` discovery verb depending on Sprint 05's `but perm list`, test/snapshot audit, net-new honesty greps) land in **Sprint 08**, coordinated with Sprint 06a's `MGMT-IPC-002`.

## Version History

| Version                     | Date       | Changes                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            | Trigger    |
| --------------------------- | ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------- |
| 1.4.0 (enrichment, planned) | 2026-06-19 | New **STEER** group (6 UCs / 32 ACs / 31 criteria, +3 named risks R15–R17): capability-aware denials — additive `class` / `held_permissions` / `authorized_actions` / `do_not` on the denial contract; derived intent-scoped action menu from a single-source route→Authority table; recoverability class + graceful degradation; self-discovery; non-enforced agent-priming primer; no-lying-menu + closed-catalog invariants. Net-additive, frozen-aware; lands as Sprint 08 after the roadmap. Brainstorm + deep-research grounded (holocron js7bqec1xmff8drgx35xav3xvs88ykcp). | Enrichment |

## Next Steps

- Apply integration edits I1–I5 + bump the PRD to v1.4.0 **once the freeze lifts** (`/kb-prd-plan --update` on the live PRD, or manual append).
- Materialize **Sprint 08 (STEER)** via `/kb-sprint-tasks-plan` after Sprint 06b, coordinating D6 with Sprint 06a.
- Optional future **Sprint 06c / MGMT enrichment**: render `authorized_actions` in the desktop Governance UI.
