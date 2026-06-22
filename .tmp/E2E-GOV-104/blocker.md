E2E-GOV-104 blocker:

The commit-flow product E2E requires real but-server pending and commit commands
after visible UI edits. The required web backend routes are absent in
`crates/but-server/src/lib.rs`.

Evidence:

- `.tmp/E2E-GOV-104/but-server-governance-route-rg.txt` shows no matches for
  `governance_pending` or `governance_commit` in but-server route registration,
  plus the catch-all `Command {command} not found!` path.
- `.tmp/E2E-GOV-104/but-server-governance-route-probe.txt` shows live POSTs to
  `/governance_commit` and `/governance_pending` returning
  `Command ... not found!`.

Writing GOV-104 product E2E now would be a known-failing test dependent on
backend route work outside this E2E-only scope.
