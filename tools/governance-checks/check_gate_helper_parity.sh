#!/usr/bin/env bash
set -euo pipefail

# Checks the GATES-007 structural invariant: each ref-mutating public seam must
# call the shared commit-gate decision helper. The counts are deliberately
# per-file, so an extra branch.rs call cannot hide an ungated worktree seam.

repo_root="$(git rev-parse --show-toplevel)"
branch_file="crates/but-api/src/branch.rs"
worktree_file="crates/but-api/src/legacy/worktree.rs"
helper="enforce_commit_gate_for_target"

require_file() {
	local path="$1"

	if [[ ! -f "${repo_root}/${path}" ]]; then
		echo "missing target file: ${path}" >&2
		exit 1
	fi
}

count_helper_calls() {
	local path="$1"

	grep -Ec "${helper}" "${repo_root}/${path}"
}

require_file "${branch_file}"
require_file "${worktree_file}"

branch_count="$(count_helper_calls "${branch_file}")"
worktree_count="$(count_helper_calls "${worktree_file}")"

if [[ "${branch_count}" -lt 2 ]]; then
	echo "${branch_file} has ${branch_count} ${helper} call(s), expected at least 2" >&2
	exit 1
fi

if [[ "${worktree_count}" -lt 1 ]]; then
	echo "${worktree_file} has ${worktree_count} ${helper} call(s), expected at least 1" >&2
	exit 1
fi

echo "OK: ${branch_file} has ${branch_count} ${helper} calls and ${worktree_file} has ${worktree_count}"
