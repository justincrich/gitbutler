---
feature: UC-GATES-01
type: security
tier: integration-test
---

# UC-GATES-01 — edge-working-tree-edit-inert

Edit the working-tree gates.toml to unprotect main, then commit directly to main as dev → still branch.protected (branch protection is read target-ref-only; a working-tree edit cannot weaken the gate).
