#!/usr/bin/env python3
"""Assert GATES-007 commit-gate placement stays before worktree guards.

NEGATIVE_TEST / manual test:
  Copy the repo files under a scratch root, move
  `enforce_commit_gate_for_target` below `exclusive_worktree_access()` in one of
  the checked public functions or add it inside `apply_with_perm`, then run:

    GATE_BEFORE_GUARD_REPO_ROOT=<scratch-root> ./tools/governance-checks/check_gate_before_guard.py

  The script must fail with an error naming the violated function. This catches
  a gate after guard lock-order regression and a gate-in-helper placement that a
  whole-file occurrence count would miss.
"""

from __future__ import annotations

import os
import re
import sys
from pathlib import Path

GATE = "enforce_commit_gate_for_target"
GUARD = "exclusive_worktree_access"

PUBLIC_ORDER_CHECKS = (
    ("crates/but-api/src/branch.rs", "apply"),
    ("crates/but-api/src/branch.rs", "apply_branch_integration"),
    ("crates/but-api/src/legacy/worktree.rs", "worktree_integrate"),
)

HELPER_NO_GATE_CHECKS = (
    ("crates/but-api/src/branch.rs", "apply_with_perm"),
    ("crates/but-api/src/branch.rs", "apply_branch_integration_with_perm"),
)


def repo_root() -> Path:
    override = os.environ.get("GATE_BEFORE_GUARD_REPO_ROOT")
    if override:
        return Path(override).resolve()
    return Path(__file__).resolve().parents[2]


def mask_non_code(source: str) -> str:
    """Replace comments and string literals with spaces while preserving offsets."""
    chars = list(source)
    index = 0
    length = len(source)

    while index < length:
        if source.startswith("//", index):
            end = source.find("\n", index)
            end = length if end == -1 else end
            for pos in range(index, end):
                chars[pos] = " "
            index = end
            continue

        if source.startswith("/*", index):
            depth = 1
            pos = index + 2
            while pos < length and depth > 0:
                if source.startswith("/*", pos):
                    depth += 1
                    pos += 2
                elif source.startswith("*/", pos):
                    depth -= 1
                    pos += 2
                else:
                    pos += 1
            for mask_pos in range(index, min(pos, length)):
                if chars[mask_pos] != "\n":
                    chars[mask_pos] = " "
            index = pos
            continue

        raw_string = re.match(r"b?r(#+)?\"", source[index:])
        if raw_string:
            hashes = raw_string.group(1) or ""
            marker = f'"{hashes}'
            start = index
            pos = index + raw_string.end()
            end = source.find(marker, pos)
            end = length if end == -1 else end + len(marker)
            for mask_pos in range(start, end):
                if chars[mask_pos] != "\n":
                    chars[mask_pos] = " "
            index = end
            continue

        if source[index] == '"' or source.startswith('b"', index):
            start = index
            pos = index + (2 if source.startswith('b"', index) else 1)
            escaped = False
            while pos < length:
                char = source[pos]
                if escaped:
                    escaped = False
                elif char == "\\":
                    escaped = True
                elif char == '"':
                    pos += 1
                    break
                pos += 1
            for mask_pos in range(start, min(pos, length)):
                if chars[mask_pos] != "\n":
                    chars[mask_pos] = " "
            index = pos
            continue

        index += 1

    return "".join(chars)


def line_number(source: str, offset: int) -> int:
    return source.count("\n", 0, offset) + 1


def find_function_body(source: str, function_name: str) -> tuple[str, str, int]:
    masked = mask_non_code(source)
    pattern = re.compile(rf"\bpub\s+fn\s+{re.escape(function_name)}\b")
    match = pattern.search(masked)
    if match is None:
        raise ValueError(f"missing public function `{function_name}`")

    body_start = masked.find("{", match.end())
    if body_start == -1:
        raise ValueError(f"missing body for public function `{function_name}`")

    depth = 0
    for pos in range(body_start, len(masked)):
        if masked[pos] == "{":
            depth += 1
        elif masked[pos] == "}":
            depth -= 1
            if depth == 0:
                return (
                    source[body_start + 1 : pos],
                    masked[body_start + 1 : pos],
                    body_start + 1,
                )

    raise ValueError(f"unterminated body for public function `{function_name}`")


def assert_gate_before_guard(repo: Path, relative_path: str, function_name: str) -> list[str]:
    path = repo / relative_path
    source = path.read_text(encoding="utf-8")
    _body, masked_body, body_offset = find_function_body(source, function_name)
    gate_index = masked_body.find(GATE)
    guard_index = masked_body.find(GUARD)
    location = f"{relative_path}::{function_name}"

    if gate_index == -1:
        return [f"{location} is missing `{GATE}`"]
    if guard_index == -1:
        return [f"{location} is missing `{GUARD}`"]
    if gate_index > guard_index:
        gate_line = line_number(source, body_offset + gate_index)
        guard_line = line_number(source, body_offset + guard_index)
        return [
            f"{location} calls `{GATE}` at line {gate_line} after `{GUARD}` at line {guard_line}"
        ]

    gate_line = line_number(source, body_offset + gate_index)
    guard_line = line_number(source, body_offset + guard_index)
    print(f"OK: {location} calls `{GATE}` at line {gate_line} before `{GUARD}` at line {guard_line}")
    return []


def assert_helper_has_no_gate(repo: Path, relative_path: str, function_name: str) -> list[str]:
    path = repo / relative_path
    source = path.read_text(encoding="utf-8")
    _body, masked_body, body_offset = find_function_body(source, function_name)
    gate_index = masked_body.find(GATE)
    location = f"{relative_path}::{function_name}"

    if gate_index == -1:
        print(f"OK: {location} contains no `{GATE}` call")
        return []

    gate_line = line_number(source, body_offset + gate_index)
    return [f"{location} must not call `{GATE}`; found at line {gate_line}"]


def main() -> int:
    root = repo_root()
    errors: list[str] = []

    for relative_path, function_name in PUBLIC_ORDER_CHECKS:
        try:
            errors.extend(assert_gate_before_guard(root, relative_path, function_name))
        except OSError as err:
            errors.append(f"{relative_path}: {err}")
        except ValueError as err:
            errors.append(f"{relative_path}: {err}")

    for relative_path, function_name in HELPER_NO_GATE_CHECKS:
        try:
            errors.extend(assert_helper_has_no_gate(root, relative_path, function_name))
        except OSError as err:
            errors.append(f"{relative_path}: {err}")
        except ValueError as err:
            errors.append(f"{relative_path}: {err}")

    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1

    print("OK: gate-before-guard source contract holds")
    return 0


if __name__ == "__main__":
    sys.exit(main())
