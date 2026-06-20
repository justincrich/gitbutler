#!/usr/bin/env bash
set -euo pipefail

# Checks the GATES-008 ownership invariant: merge_gate.rs production
# classification remains unchanged from the AUTHZ-004 baseline. This is a whole
# file diff against the recorded owner commit, not a narrow grep.

baseline_commit="a80ce888a894ae3568ee8127866e987e89d6c092"
target="crates/but-api/src/legacy/merge_gate.rs"
repo_root="$(git rev-parse --show-toplevel)"

cd "${repo_root}"

if [[ ! -f "${target}" ]]; then
	echo "missing target file: ${target}" >&2
	exit 1
fi

if ! git cat-file -e "${baseline_commit}^{commit}"; then
	echo "missing AUTHZ-004 baseline commit: ${baseline_commit}" >&2
	exit 1
fi

if ! git diff --quiet --exit-code "${baseline_commit}" -- "${target}"; then
	echo "${target} differs from AUTHZ-004 baseline ${baseline_commit}" >&2
	git diff -- "${baseline_commit}" -- "${target}" >&2
	exit 1
fi

echo "OK: ${target} is unchanged from AUTHZ-004 baseline ${baseline_commit}"
