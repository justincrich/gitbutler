#!/usr/bin/env bash
set -euo pipefail

# Checks the GATES-008 ownership invariant: merge_gate.rs production
# classification remains unchanged from the AUTHZ-004 baseline. This is a
# baseline-range zero-diff check plus staged and unstaged checks, not a narrow
# grep for added tokens.
#
# NEGATIVE_TEST (manual test): in a disposable worktree, delete one production
# line from crates/but-api/src/legacy/merge_gate.rs or rename the file, then run
# this script. It must exit non-zero before the change is restored.

baseline_sha="a80ce888a894ae3568ee8127866e987e89d6c092"
target="crates/but-api/src/legacy/merge_gate.rs"
repo_root="$(git rev-parse --show-toplevel)"

cd "${repo_root}"

if [[ ! -f "${target}" ]]; then
	echo "missing target file: ${target}" >&2
	exit 1
fi

if ! git cat-file -e "${baseline_sha}^{commit}"; then
	echo "missing AUTHZ-004 baseline commit: ${baseline_sha}" >&2
	exit 1
fi

if ! git cat-file -e "${baseline_sha}:${target}"; then
	echo "baseline ${baseline_sha} does not contain ${target}" >&2
	exit 1
fi

if ! git diff --quiet --exit-code "${baseline_sha}...HEAD" -- "${target}"; then
	echo "${target} has committed changes since AUTHZ-004 baseline ${baseline_sha}" >&2
	git diff -- "${baseline_sha}...HEAD" -- "${target}" >&2
	exit 1
fi

if ! git diff --quiet --exit-code --cached -- "${target}"; then
	echo "${target} has staged changes relative to HEAD" >&2
	git diff --cached -- "${target}" >&2
	exit 1
fi

if ! git diff --quiet --exit-code -- "${target}"; then
	echo "${target} has unstaged changes relative to the index" >&2
	git diff -- "${target}" >&2
	exit 1
fi

echo "OK: ${target} is unchanged from AUTHZ-004 baseline ${baseline_sha}"
