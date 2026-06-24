---
feature: UC-AUTHZ-02
type: happy_path
tier: integration-test
---

# UC-AUTHZ-02 — core-structured-denial

A denied governed action emits a structured JSON denial {code, message, remediation_hint} on stderr + exit 1, naming the exact missing Authority; an authorized action exits 0 and proceeds.
