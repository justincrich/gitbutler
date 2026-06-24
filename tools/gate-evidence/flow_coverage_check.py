#!/usr/bin/env python3
"""Confirm every locked flow has a runnable `run_cmd` and covers the UC scope.

Parses the locked `human-flows.json` (never hardcodes flows) and asserts:

  1. Every flow has a non-empty `run_cmd` (a defined, executable command).
  2. With `--sprint`, every in-scope use case (from the json `scope` field,
     comma-separated, e.g. `UC-AUTHZ-01,UC-GATES-02`) is bound to at least one
     flow via that flow's `uc_ref`. An uncovered UC is a coverage gap and fails
     the gate.

Exits 0 if coverage holds, non-zero with a clear stderr message otherwise.

Usage:
    flow_coverage_check.py --sprint <sprint-id> [--flows PATH]

NEGATIVE_TEST / manual test:
    Edit a copy of human-flows.json: blank one flow's run_cmd, or drop every
    flow whose uc_ref is `UC-GATES-02`. Running this script against the copy
    must exit non-zero naming the gap.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


def repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def default_flows_path(sprint: str) -> Path:
    return (
        repo_root()
        / ".spec"
        / "prds"
        / "governance"
        / "tasks"
        / sprint
        / "human-flows.json"
    )


def load_flows(flows_path: Path) -> dict:
    try:
        data = json.loads(flows_path.read_text(encoding="utf-8"))
    except OSError as err:
        sys.exit(f"ERROR: cannot read flows file {flows_path}: {err}")
    except json.JSONDecodeError as err:
        sys.exit(f"ERROR: invalid JSON in {flows_path}: {err}")
    if "flows" not in data or not isinstance(data["flows"], list):
        sys.exit(f"ERROR: {flows_path} has no 'flows' list")
    return data


def parse_scope(raw: str) -> list[str]:
    """Split the json `scope` field into individual UC tokens."""
    return [tok.strip() for tok in raw.split(",") if tok.strip()]


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Confirm every locked flow has a run_cmd and covers the UC scope.",
    )
    parser.add_argument("--sprint", required=True, help="sprint id (folder name)")
    parser.add_argument(
        "--flows",
        type=Path,
        default=None,
        help="path to human-flows.json (default: derived from --sprint)",
    )
    args = parser.parse_args()

    flows_path = args.flows or default_flows_path(args.sprint)
    data = load_flows(flows_path)
    if data.get("sprint") and data["sprint"] != args.sprint:
        print(
            f"WARNING: --sprint={args.sprint} but flows file sprint field is "
            f"{data['sprint']!r}; proceeding with the file contents",
            file=sys.stderr,
        )

    flows = data["flows"]
    errors: list[str] = []

    # (1) Every flow must have a non-empty run_cmd.
    empty_run_cmds: list[str] = []
    covered_uc: set[str] = set()
    for flow in flows:
        flow_id = flow.get("id", "<no-id>")
        run_cmd = (flow.get("run_cmd") or "").strip()
        if not run_cmd:
            empty_run_cmds.append(flow_id)
        uc = (flow.get("uc_ref") or "").strip()
        if uc:
            covered_uc.add(uc)
    if empty_run_cmds:
        errors.append(
            f"{len(empty_run_cmds)} flow(s) have an empty run_cmd: {empty_run_cmds}"
        )

    # (2) Every in-scope UC must be bound to >=1 flow.
    scope_raw = data.get("scope") or ""
    scope = parse_scope(scope_raw)
    if not scope:
        errors.append(
            "no `scope` field in human-flows.json; cannot verify UC coverage"
        )
    else:
        uncovered = [uc for uc in scope if uc not in covered_uc]
        if uncovered:
            errors.append(
                f"in-scope UC(s) with no bound flow: {uncovered} "
                f"(covered: {sorted(covered_uc)})"
            )

    # Report coverage map for visibility.
    print(f"flows: {len(flows)}")
    print(f"scope: {scope}")
    for uc in scope:
        bound = [f.get("id", "?") for f in flows if (f.get("uc_ref") or "").strip() == uc]
        mark = "OK" if bound else "GAP"
        print(f"  {uc}: {mark} <- {bound}")

    if errors:
        for err in errors:
            print(f"ERROR: {err}", file=sys.stderr)
        return 1

    print(
        f"OK: all {len(flows)} flows have a run_cmd and all {len(scope)} in-scope UC(s) covered"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
