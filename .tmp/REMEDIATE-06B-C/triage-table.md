# Backlog Triage (REMEDIATE-06B-C)

Triage of the 14 backlog follow-up files to make the sprint boundary honest. Verdicts are based on HEAD `b3449afbb2` and the current Human Testing Gate definition.

| Task | Verdict | Justification | Target |
|------|---------|---------------|--------|
| MGMT-BE-004A | CLOSED-SUPERSEDED | MGMT-BE-004 landed at HEAD and `branch_gates_update_round_trips_full_gate_schema_lossless` passes; the full-schema writer behavior 004A intended is already realized. | Close as Superseded by MGMT-BE-004 |
| REMEDIATE-RUST-1 | ALREADY-SATISFIED | `but-authz` `gates_path()` and widened `GatesWire.gate` array are present at HEAD; the lossless round-trip test `branch_gates_update_round_trips_full_gate_schema_lossless` passes at HEAD b3449afbb2. | Close as Done |
| REMEDIATE-RUST-3 | OUT-OF-SPRINT (06c) | The Tauri None-path admin:read gate is not implemented (`list_workspace_rules_scoped_for_caller` delegates `None` to the ungated `list_workspace_rules_scoped`), but the manual Human Testing Gate step 2 uses the principal-scoped `Some` path only. | sprint-06c-governance-followups |
| REMEDIATE-RUST-5 | CANCELLED | Already folded into E2E-MGMT-BE-002A AC-4; no independent work. | leave Cancelled |
| REMEDIATE-UI-1 | CANCELLED | Superseded by REMEDIATE-06B-D AC-3, which adds the symmetric self-revoke no-flip proof. | Close as Superseded by REMEDIATE-06B-D |
| REMEDIATE-UI-2 | OUT-OF-SPRINT (06c) | Widening the build-gate grep to all server files is perimeter hardening, not required by any Human Testing Gate step. | sprint-06c-governance-followups |
| REMEDIATE-UI-3 | OUT-OF-SPRINT (06c) | Web-target governance route supports the capstone E2E, not the manual Human Testing Gate. | sprint-06c-governance-followups |
| REMEDIATE-UI-4 | OUT-OF-SPRINT (06c) | Adding `verified_by` pointers is design-contract hygiene, not gate-critical. | sprint-06c-governance-followups |
| REMEDIATE-UI-5 | OUT-OF-SPRINT (06c) | Stronger pre-click oracle for MGMT-UI-011 AC-4 is useful but not required for gate step 4. | sprint-06c-governance-followups |
| REMEDIATE-UI-6 | OUT-OF-SPRINT (06c) | Re-protect toggle AC adds inverse-flow coverage; no gate step exercises it. | sprint-06c-governance-followups |
| REM-DESIGN-MGMT-004-A | OUT-OF-SPRINT (06c) | Corrected U1 wording/self-revoke design contract is needed, but the current gate can run with DESIGN-MGMT-004 annotations. | sprint-06c-governance-followups |
| E2E-MGMT-BE-001 | OUT-OF-SPRINT (06c) | Governed-repo E2E fixtures serve the automated Playwright capstone, not the manual Human Testing Gate. | sprint-06c-governance-followups |
| E2E-MGMT-BE-002 | CLOSED-SUPERSEDED | Original premise "but-server routes zero governance commands" is false at HEAD; 16 governance routes are already registered in `crates/but-server/src/lib.rs:34-99`. | Close as Superseded by E2E-MGMT-BE-002A |
| E2E-MGMT-BE-002A | OUT-OF-SPRINT (06c) | Integration tests for the already-registered routes are hardening for the automated capstone, not required for the manual gate. | sprint-06c-governance-followups |
| E2E-MGMT-UI-001 | OUT-OF-SPRINT (06c) | The Playwright capstone is the automated successor to the manual Human Testing Gate, not the gate itself. | sprint-06c-governance-followups |

**Tally:** 1 ALREADY-SATISFIED, 3 CLOSED-SUPERSEDED, 2 CANCELLED, 9 OUT-OF-SPRINT (06c).
