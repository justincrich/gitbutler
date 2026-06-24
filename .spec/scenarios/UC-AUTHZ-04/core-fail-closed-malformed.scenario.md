---
feature: UC-AUTHZ-04
type: happy_path
tier: integration-test
---

# UC-AUTHZ-04 — core-fail-closed-malformed

Commit a malformed gates.toml to the target ref, run a governed action → denied config.invalid (fail-closed; the engine does not run on a config it cannot parse).
