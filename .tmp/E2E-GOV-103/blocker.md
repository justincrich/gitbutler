E2E-GOV-103 blocker:

The pending-edits product E2E requires real but-server write/read commands for
principal and group edits. The required web backend routes are absent in
`crates/but-server/src/lib.rs`.

Evidence:

- `.tmp/E2E-GOV-103/but-server-governance-route-rg.txt` shows no matches for
  `perm_grant`, `group_grant`, or `governance_principals_list` in but-server
  route registration, plus the catch-all `Command {command} not found!` path.
- `.tmp/E2E-GOV-103/but-server-governance-route-probe.txt` shows live POSTs to
  `/perm_grant`, `/group_grant`, and `/governance_principals_list` returning
  `Command ... not found!`.

Writing GOV-103 product E2E now would only prove the missing backend routes, not
the requested product pending-edit behavior.
