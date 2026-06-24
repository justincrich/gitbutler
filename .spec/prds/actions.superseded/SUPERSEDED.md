# ⚠️ SUPERSEDED — see `../check-runner/`

This **`actions`** PRD has been **superseded** (2026-06-20) by [`../check-runner/`](../check-runner/README.md).

## Why

This PRD scoped the capability — _a local runner that gates a code change on a pass/fail bound to the head OID_ — as a **cryptographic "agent-non-forgeable validation"** system: a signed, agent-unwritable hardened ledger, HMAC→Ed25519, an OS-sandboxed executor, all carried as owed debt.

Under the actual threat model — **personal-tenant, own-fleet, cheapest-honest-path** — that bar is wrong:

- The agent shares the OS user with the runner, so symmetric signing cannot close forgery anyway.
- A check is **reproducible**, so it does not need to: a forged green is caught by re-running, and the honest path (let the runner run the check) is the path of least resistance.
- A required check is just **"a second deterministic review whose verdict a local runner produces"** — and governance already gates merges on a verdict store (`local_review_verdicts`) it accepts as forgeable (its R6). The check store needs no more protection, and is strictly safer because reproducible.

## What replaced it

`check-runner/` re-scopes this from a CI-platform-shaped non-forgery system down to a focused **"second deterministic-review clause + local runner"**:

- **Dropped:** signing-as-security, the hardened ledger, HMAC/Ed25519, the OS-sandbox debt (a nullable `signature` column is retained only as a forward-compat seam, explicitly not a v1 security claim).
- **Kept:** the cheap structural locks (gate reads a stored fact not agent prose; runner ≠ agent; no caller-supplied-conclusion API; SHA-binding) and the bootstrap-invariant.
- **Promoted:** the **mechanism-agnostic clean checkout at the head OID** (virtual branches / worktrees / plain git) from one under-specified risk row to the #1 technical risk.
- **Result:** 3 groups / 15 UCs (vs. this PRD's 4 / ~19), the crypto "ledger" group dissolved into a plain `but-db` table.

This folder is retained for historical reference only. **Do not plan or build from it** — use `../check-runner/`.
