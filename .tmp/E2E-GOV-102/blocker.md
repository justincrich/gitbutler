E2E-GOV-102 blocker:

The product E2E flow needs the desktop web build to open Permissions & Governance
and read governance tab/read-only state from real but-server commands. The
required but-server routes are absent in `crates/but-server/src/lib.rs`.

Evidence:

- `.tmp/E2E-GOV-102/but-server-governance-route-rg.txt` shows no matches for
  `governance_status_read` or `governance_pending` in but-server route
  registration, plus the catch-all `Command {command} not found!` path.
- `.tmp/E2E-GOV-102/but-server-governance-route-probe.txt` shows live POSTs to
  `/governance_status_read` and `/governance_pending` returning
  `Command ... not found!`.

Writing the GOV-102 product E2E before backend route support would create a
known-failing test outside this E2E-only scope.
