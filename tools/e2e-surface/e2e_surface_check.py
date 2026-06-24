#!/usr/bin/env python3
"""Confirm each locked flow enters at the real `but` CLI / crate surface.

For sprint-00 the flows ARE `cargo test` commands that drive the real `but`
binary (assert_cmd/snapbox) or the real `but-api`/`but-authz` crate APIs. They
are at the right surface by construction. This check parses the locked
`human-flows.json` (never hardcodes flows) and, for each flow, confirms its
`run_cmd` drives a recognized real crate/binary surface: `cargo test -p but`,
`cargo test -p but-api`, or `cargo test -p but-authz`.

A flow that instead exercised an internal helper, a mock, or a private
function beneath the public surface would be flagged `surface_fail`.

Usage:
    e2e_surface_check.py --sprint <sprint-id> [--json] [--flows PATH]

NEGATIVE_TEST / manual test:
    Edit a copy of human-flows.json and change one run_cmd to
    `cargo test -p but-internal foo` (a non-existent/private crate), then run:
        e2e_surface_check.py --sprint <id> --flows <copy>
    The script must exit non-zero and report `surface_fail` for that flow.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

# Crates that constitute the real human/API surface for this sprint.
# `but` = the CLI/TUI binary (assert_cmd drives the real process boundary);
# `but-api` = the shared API surface for all four callers;
# `but-authz` = the authorization primitive exercised through its public API.
REAL_SURFACES = ("but", "but-api", "but-authz")


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


# Match `-p <crate>` with the crate bounded by whitespace, end of segment, or
# `&&` so `but` does not greedily swallow `but-api`/`but-authz`.
CRATE_RE = re.compile(r"-p\s+(but-authz|but-api|but)(?=\s|$|&)")


def extract_crates(run_cmd: str) -> list[str]:
    """Return the list of `-p <crate>` targets in a (possibly chained) run_cmd."""
    return CRATE_RE.findall(run_cmd)


def surface_status(run_cmd: str) -> tuple[str, str]:
    """Return (status, detail) for one flow's run_cmd."""
    if not run_cmd or not run_cmd.strip():
        return "surface_fail", "run_cmd is empty"
    crates = extract_crates(run_cmd)
    if not crates:
        return (
            "surface_fail",
            "run_cmd does not reference a real cargo-test surface "
            f"(expected one of {list(REAL_SURFACES)})",
        )
    unknown = sorted({c for c in crates if c not in REAL_SURFACES})
    if unknown:
        return (
            "surface_fail",
            f"run_cmd references non-surface crate(s): {unknown}",
        )
    return "surface_ok", f"drives real surface crate(s): {crates}"


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Confirm each locked flow enters at the real but/API surface.",
    )
    parser.add_argument("--sprint", required=True, help="sprint id (folder name)")
    parser.add_argument(
        "--flows",
        type=Path,
        default=None,
        help="path to human-flows.json (default: derived from --sprint)",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="emit machine-readable JSON instead of human lines",
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

    results = []
    failures: list[str] = []
    for flow in data["flows"]:
        flow_id = flow.get("id", "<no-id>")
        run_cmd = flow.get("run_cmd", "")
        status, detail = surface_status(run_cmd)
        results.append(
            {"id": flow_id, "status": status, "detail": detail, "run_cmd": run_cmd}
        )
        if status != "surface_ok":
            failures.append(f"{flow_id}: {detail}")

    ok = not failures

    if args.json:
        print(json.dumps({"ok": ok, "flows": results}, indent=2))
    else:
        for r in results:
            print(f"{r['id']}\t{r['status']}\t{r['detail']}")
        if ok:
            print(f"OK: all {len(results)} flows enter at the real surface")
        else:
            for f in failures:
                print(f"ERROR: {f}", file=sys.stderr)

    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
