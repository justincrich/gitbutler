---
feature: UC-LOOP-02
type: security
tier: build-gate
---

# UC-LOOP-02 — edge-no-role-invariant

After all AUTHZ hardening, re-assert the honesty invariant: no role name, no human-vs-AI predicate, no Permission overload enters any enforcement path. The invariant grep-gates are green.
