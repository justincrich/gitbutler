---
feature: UC-LOOP-01
type: happy_path
tier: e2e-automated
---

# UC-LOOP-01 — core-reference-flow

The 3-principal reference flow: implementer (contents:write, no merge) merge denied; reviewer (reviews:write) submits review at head; maintainer (merge) merge proceeds only after a distinct approval. Role separation emerges from the functional permission set — no role-name in any enforcement path. [JOURNEY: spans the commit→review→merge chain.]
