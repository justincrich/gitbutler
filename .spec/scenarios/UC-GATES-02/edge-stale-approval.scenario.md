---
feature: UC-GATES-02
type: boundary
tier: integration-test
---

# UC-GATES-02 — edge-stale-approval

Approve at head H1, advance to H2, run but merge → denied gate.review_required with approval_stale_at_head (the approval must be at the CURRENT head).
