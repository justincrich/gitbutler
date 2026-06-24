---
feature: UC-AUTHZ-01
type: happy_path
tier: integration-test
---

# UC-AUTHZ-01 — core-permission-model

Seed committed permissions.toml with ro (contents:read) and dev (contents:write); authorize() resolves the principal's authority set from committed config only — a principal with contents:write is permitted, one with only contents:read is denied with the exact Authority named in perm.denied.
