#!/usr/bin/env python3
"""Write sprint-goal-state.json with a code-computed pass/fail verdict.

THE VERDICT IS COMPUTED FROM REAL EXIT CODES, NEVER NARRATED.

For the given sprint this script:
  (a) parses the locked `human-flows.json` to get the flow list + run_cmds,
  (b) runs each flow's `run_cmd` via a real subprocess and captures its exit
      code (these are `cargo test` commands driving the real `but` binary /
      crate APIs — the load-bearing proof),
  (c) runs `e2e_surface_check.py` and `coldboot_check.py` (as subprocesses,
      `--json`) and captures their per-flow + overall verdict,
  (d) writes `sprint-goal-state.json` in the sprint folder with shape:
        {
          "sprint": "<id>",
          "verdict": "pass" | "fail",
          "flows": [ { "id", "run_cmd", "exit_code", "surface", "coldboot" } ],
          "computed_at": "<ISO timestamp>",
          "computed_by": "tools/gate-evidence/gate_evidence_check.py (exit codes, not narrated)"
        }

`verdict` is "pass" IFF every flow's exit_code == 0 AND the surface check AND
the cold-boot check both pass. The file is always written (evidence is captured
whether the verdict is pass or fail); the script exits 0 only when verdict is
"pass".

Usage:
    gate_evidence_check.py --sprint <sprint-id> [--flows PATH] [--flow-timeout SEC]

NEGATIVE_TEST / manual test:
    Run against a sprint whose human-flows.json has a run_cmd that exits non-zero
    (e.g. `cargo test -p but does_not_exist`). The script must write
    sprint-goal-state.json with `"verdict": "fail"` and exit non-zero.
"""

from __future__ import annotations

import argparse
import datetime as _dt
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

E2E_SURFACE_CHECK = (
    Path(__file__).resolve().parent.parent / "e2e-surface" / "e2e_surface_check.py"
)
COLDBOOT_CHECK = (
    Path(__file__).resolve().parent.parent / "e2e-surface" / "coldboot_check.py"
)
DEFAULT_FLOW_TIMEOUT = 3600  # seconds per flow (cargo test; generous compile bound)


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


def run_subcheck(script: Path, sprint: str) -> tuple[bool, dict[str, str]]:
    """Run an e2e-surface sub-check with --json; return (overall_ok, id->status)."""
    if not script.is_file():
        print(f"ERROR: missing sub-check script: {script}", file=sys.stderr)
        return False, {}
    cmd = [sys.executable, str(script), "--sprint", sprint, "--json"]
    try:
        proc = subprocess.run(
            cmd,
            cwd=repo_root(),
            capture_output=True,
            text=True,
            timeout=600,
        )
    except subprocess.TimeoutExpired:
        print(f"ERROR: sub-check timed out: {script}", file=sys.stderr)
        return False, {}
    if proc.returncode != 0:
        print(
            f"WARNING: sub-check exited {proc.returncode}: {script.name}\n"
            f"{proc.stderr.strip()}",
            file=sys.stderr,
        )
    try:
        payload = json.loads(proc.stdout)
    except json.JSONDecodeError:
        print(
            f"ERROR: sub-check {script.name} did not emit valid JSON. "
            f"stdout:\n{proc.stdout}\nstderr:\n{proc.stderr}",
            file=sys.stderr,
        )
        return False, {}
    status_by_id = {f["id"]: f.get("status", "unknown") for f in payload.get("flows", [])}
    return bool(payload.get("ok")), status_by_id


def run_flow(run_cmd: str, timeout: int) -> tuple[int, str, str]:
    """Run one flow's run_cmd in a real shell; return (exit_code, note, tail).

    cargo output is captured (in-memory, not written to disk — respects the
    sprint write-allowlist). On failure the last ~40 lines of combined output are
    returned as `tail` so the gate log explains WHY a flow went red; on success
    only the exit code is needed as evidence.
    """
    print(f"  RUN: {run_cmd}", file=sys.stderr)
    try:
        proc = subprocess.run(
            run_cmd,
            shell=True,
            cwd=repo_root(),
            timeout=timeout,
            capture_output=True,
            text=True,
        )
    except subprocess.TimeoutExpired:
        return 124, f"timed out after {timeout}s", ""
    except OSError as err:
        return 1, f"failed to spawn: {err}", ""
    combined = (proc.stdout or "") + (proc.stderr or "")
    tail = "\n".join(combined.splitlines()[-40:])
    return proc.returncode, "", tail


def atomic_write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, tmp_name = tempfile.mkstemp(
        prefix=path.name + ".", suffix=".tmp", dir=str(path.parent)
    )
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as fh:
            json.dump(payload, fh, indent=2)
            fh.write("\n")
        os.replace(tmp_name, path)
    finally:
        if os.path.exists(tmp_name):
            os.unlink(tmp_name)


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Compute and write the sprint goal-state verdict from real exit codes.",
    )
    parser.add_argument("--sprint", required=True, help="sprint id (folder name)")
    parser.add_argument(
        "--flows",
        type=Path,
        default=None,
        help="path to human-flows.json (default: derived from --sprint)",
    )
    parser.add_argument(
        "--flow-timeout",
        type=int,
        default=DEFAULT_FLOW_TIMEOUT,
        help=f"per-flow run_cmd timeout in seconds (default {DEFAULT_FLOW_TIMEOUT})",
    )
    args = parser.parse_args()

    flows_path = args.flows or default_flows_path(args.sprint)
    data = load_flows(flows_path)
    sprint_field = data.get("sprint") or args.sprint
    if data.get("sprint") and data["sprint"] != args.sprint:
        print(
            f"WARNING: --sprint={args.sprint} but flows file sprint field is "
            f"{data['sprint']!r}; proceeding with the file contents",
            file=sys.stderr,
        )

    # (c) Surface + cold-boot sub-checks (per-flow status + overall ok).
    surface_ok, surface_by_id = run_subcheck(E2E_SURFACE_CHECK, args.sprint)
    coldboot_ok, coldboot_by_id = run_subcheck(COLDBOOT_CHECK, args.sprint)

    # (b) Run each flow's run_cmd for real and capture the exit code.
    flow_records = []
    all_exits_zero = True
    for flow in data["flows"]:
        flow_id = flow.get("id", "<no-id>")
        run_cmd = flow.get("run_cmd", "")
        print(f"\n[{flow_id}]", file=sys.stderr)
        if not run_cmd.strip():
            print("  ERROR: empty run_cmd", file=sys.stderr)
            exit_code, note, tail = 1, "empty run_cmd", ""
        else:
            exit_code, note, tail = run_flow(run_cmd, args.flow_timeout)
        if exit_code != 0:
            all_exits_zero = False
        surface_status = surface_by_id.get(flow_id, "surface_unknown")
        coldboot_status = coldboot_by_id.get(flow_id, "coldboot_unknown")
        if note:
            print(f"  exit_code={exit_code} ({note})", file=sys.stderr)
        else:
            print(f"  exit_code={exit_code}", file=sys.stderr)
        if exit_code != 0 and tail:
            print("  --- output tail (last ~40 lines) ---", file=sys.stderr)
            print(tail, file=sys.stderr)
            print("  --- end tail ---", file=sys.stderr)
        flow_records.append(
            {
                "id": flow_id,
                "run_cmd": run_cmd,
                "exit_code": exit_code,
                "surface": surface_status,
                "coldboot": coldboot_status,
            }
        )

    # (d) Compute the verdict and write the state file.
    verdict = (
        "pass"
        if (all_exits_zero and surface_ok and coldboot_ok)
        else "fail"
    )

    computed_at = _dt.datetime.now(_dt.UTC).isoformat()
    state = {
        "sprint": sprint_field,
        "verdict": verdict,
        "flows": flow_records,
        "surface_check_ok": surface_ok,
        "coldboot_check_ok": coldboot_ok,
        "computed_at": computed_at,
        "computed_by": (
            "tools/gate-evidence/gate_evidence_check.py (exit codes, not narrated)"
        ),
    }

    state_path = (
        repo_root()
        / ".spec"
        / "prds"
        / "governance"
        / "tasks"
        / args.sprint
        / "sprint-goal-state.json"
    )
    atomic_write_json(state_path, state)
    print(f"\nWROTE: {state_path}", file=sys.stderr)

    # Human summary.
    print(json.dumps(state, indent=2))
    print(f"\nverdict: {verdict}", file=sys.stderr)
    if not all_exits_zero:
        bad = [r["id"] for r in flow_records if r["exit_code"] != 0]
        print(f"ERROR: {len(bad)} flow(s) exited non-zero: {bad}", file=sys.stderr)
    if not surface_ok:
        print("ERROR: e2e_surface_check reported surface_fail for >=1 flow", file=sys.stderr)
    if not coldboot_ok:
        print("ERROR: coldboot_check reported coldboot_fail for >=1 flow", file=sys.stderr)

    return 0 if verdict == "pass" else 1


if __name__ == "__main__":
    sys.exit(main())
