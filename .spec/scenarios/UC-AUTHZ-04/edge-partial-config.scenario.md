---
feature: UC-AUTHZ-04
type: boundary
tier: integration-test
---

# UC-AUTHZ-04 — edge-partial-config

Commit exactly one of {permissions.toml, gates.toml} → config.invalid (partial config is fail-closed, not a silent permit). The opt-in-by-presence model requires BOTH or NEITHER.
