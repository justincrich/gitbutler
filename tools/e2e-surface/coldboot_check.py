#!/usr/bin/env python3
"""Confirm each flow's test fixtures start cold (committed config, not injected).

The cold-boot bug class this lint catches: a test that SEEDS governance config
(`permissions.toml`/`gates.toml`) by writing it to the working tree or injecting
it via a process-global env var at test time, instead of COMMITTING it via real
git operations the way a human would. Such a test passes against injected config
while a cold-boot user (who only has committed config) sees different behavior.

This is a STATIC ANALYSIS of the test-fixture seeding pattern in each flow's
crate test sources. For each flow it asserts two invariants:

  1. No leaky identity injection. A process-global `std::env::set_var(
     "BUT_AGENT_HANDLE", ...)` (which leaks across tests and diverges from
     cold-boot) is forbidden. The SCOPED `temp_env::with_var(
     "BUT_AGENT_HANDLE", ...)` pattern is ALLOWED: it is restored after the
     closure and faithfully simulates a human setting `BUT_AGENT_HANDLE` as a
     real env var before running `but`. That is the runtime identity input, not
     governance config.

  2. Governance config is committed, not merely written. Wherever a test fixture
     SEEDS a governance config file (heredoc/redirect write or plumbing
     `update-index --add --cacheinfo ... .gitbutler/(permissions|gates).toml`),
     the same test source must COMMIT it via a real git operation (`git add`,
     `git commit`, `git update-index --add`, or `git commit-tree`). A bare
     reference to the filename in an assertion/error string is NOT a seed and is
     ignored. This is what `governed_repo()` at
     `crates/but-api/tests/commit_gate.rs:727` does correctly, and what this
     lint enforces everywhere.

Usage:
    coldboot_check.py --sprint <sprint-id> [--json] [--flows PATH]

NEGATIVE_TEST / manual test:
    In a scratch copy of a test file, add `std::env::set_var("BUT_AGENT_HANDLE",
    "dev");` or write `echo x > .gitbutler/permissions.toml` with no following
    `git add`/`git commit`, then point this script at the crate. It must exit
    non-zero and report `coldboot_fail` for every flow bound to that crate.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

# A config SEED: a write targeting a governance config file. Covers:
#   - heredoc/redirect:  cat >.gitbutler/permissions.toml <<'EOF'
#   - redirect write:    echo ... > .gitbutler/gates.toml
#   - plumbing add:      git update-index --add --cacheinfo 100644 "$blob" .gitbutler/permissions.toml
CONFIG_SEED_RE = re.compile(
    r"""> \s* [^|&;]* \.gitbutler/ (?:permissions|gates) \.toml
        | update-index \s+ --add [^\n]* \.gitbutler/ (?:permissions|gates) \.toml
    """,
    re.VERBOSE,
)

# A real git COMMIT operation (porcelain or plumbing). Presence in a file that
# seeds config proves the seed is committed, not left in the working tree.
GIT_COMMIT_RE = re.compile(
    r"git \s+ add \b | git \s+ commit \b | update-index \s+ --add \b | git \s+ commit-tree \b",
    re.VERBOSE,
)

# Leaky process-global identity injection (restored: never; leaks across tests).
LEAKY_SETVAR_RE = re.compile(r'\bstd::env::set_var\s*\(\s*"BUT_AGENT_HANDLE"')

# Map crate -> test source directory (recursively scanned).
CRATE_TEST_DIRS = {
    "but-authz": "crates/but-authz/tests",
    "but": "crates/but/tests",
    "but-api": "crates/but-api/tests",
}

CRATE_RE = re.compile(r"-p\s+(but-authz|but-api|but)(?=\s|$|&)")


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


def gather_test_sources(crate: str) -> list[Path]:
    rel = CRATE_TEST_DIRS.get(crate)
    if rel is None:
        return []
    root = repo_root() / rel
    if not root.is_dir():
        return []
    return sorted(root.rglob("*.rs"))


def analyze_file(path: Path) -> list[str]:
    """Return a list of cold-boot violations found in one test source file."""
    try:
        text = path.read_text(encoding="utf-8")
    except OSError as err:
        return [f"{path}: unreadable: {err}"]

    violations: list[str] = []

    # (1) Leaky process-global BUT_AGENT_HANDLE injection is always a violation.
    for match in LEAKY_SETVAR_RE.finditer(text):
        line = text.count("\n", 0, match.start()) + 1
        violations.append(
            f"{path}:{line}: leaky std::env::set_var(BUT_AGENT_HANDLE) "
            "(use scoped temp_env::with_var instead)"
        )

    # (2) Config seeding must be accompanied by a real git commit in the same file.
    has_seed = bool(CONFIG_SEED_RE.search(text))
    has_commit = bool(GIT_COMMIT_RE.search(text))
    if has_seed and not has_commit:
        violations.append(
            f"{path}: seeds governance config (permissions.toml/gates.toml) "
            "but has no git add/commit/update-index/commit-tree — working-tree-only seed"
        )

    return violations


def analyze_crate(crate: str) -> tuple[str, list[str]]:
    """Return (status, violations) for all test sources of a crate."""
    if crate not in CRATE_TEST_DIRS:
        return (
            "coldboot_fail",
            [f"crate {crate!r} has no configured test-source directory"],
        )
    sources = gather_test_sources(crate)
    if not sources:
        return (
            "coldboot_fail",
            [f"no test sources found under {CRATE_TEST_DIRS[crate]}"],
        )
    all_violations: list[str] = []
    for src in sources:
        all_violations.extend(analyze_file(src))
    if all_violations:
        return "coldboot_fail", all_violations
    return "coldboot_ok", []


def crates_for_flow(run_cmd: str) -> list[str]:
    """Distinct crates targeted by a (possibly chained) run_cmd."""
    return list(dict.fromkeys(CRATE_RE.findall(run_cmd)))


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Confirm each flow's fixtures commit config (cold boot), not inject it.",
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

    # Analyze each distinct crate once, then attribute to every flow that uses it.
    crate_status: dict[str, tuple[str, list[str]]] = {}

    def status_for(crate: str) -> tuple[str, list[str]]:
        if crate not in crate_status:
            crate_status[crate] = analyze_crate(crate)
        return crate_status[crate]

    results = []
    failures: list[str] = []
    for flow in data["flows"]:
        flow_id = flow.get("id", "<no-id>")
        run_cmd = flow.get("run_cmd", "")
        crates = crates_for_flow(run_cmd)
        if not crates:
            results.append(
                {
                    "id": flow_id,
                    "status": "coldboot_fail",
                    "detail": "run_cmd does not target a recognized crate",
                    "violations": ["no crate target in run_cmd"],
                }
            )
            failures.append(f"{flow_id}: no crate target in run_cmd")
            continue

        flow_violations: list[str] = []
        statuses: list[str] = []
        for crate in crates:
            st, viol = status_for(crate)
            statuses.append(st)
            for v in viol:
                flow_violations.append(f"[{crate}] {v}")
        status = "coldboot_ok" if all(s == "coldboot_ok" for s in statuses) else "coldboot_fail"
        detail = (
            f"crate(s) {crates} commit config via git; no leaky identity injection"
            if status == "coldboot_ok"
            else f"{len(flow_violations)} cold-boot violation(s) across crate(s) {crates}"
        )
        results.append(
            {
                "id": flow_id,
                "status": status,
                "detail": detail,
                "violations": flow_violations,
            }
        )
        if status != "coldboot_ok":
            for v in flow_violations:
                failures.append(f"{flow_id}: {v}")

    ok = not failures

    if args.json:
        print(json.dumps({"ok": ok, "flows": results}, indent=2))
    else:
        for r in results:
            print(f"{r['id']}\t{r['status']}\t{r['detail']}")
        if ok:
            print(f"OK: all {len(results)} flows are cold-boot clean (config committed)")
        else:
            for f in failures:
                print(f"ERROR: {f}", file=sys.stderr)

    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
