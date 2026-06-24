---
feature: UC-AUTHZ-01
type: security
tier: integration-test
---

# UC-AUTHZ-01 — edge-absent-principal

Run a governed action as a principal whose BUT_AGENT_HANDLE is absent from committed permissions.toml → denied perm.denied (fail-closed default-deny; no implicit principal).
