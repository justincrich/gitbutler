#!/usr/bin/env bash
set -euo pipefail

# Checks the GATES-008 ownership invariant: merge_gate.rs production
# classification remains unchanged from the post-STEER-001 baseline. This is a
# baseline-range zero-diff check plus staged and unstaged checks, not a narrow
# grep for added tokens.
#
# BASELINE CHOICE: the original AUTHZ-004 baseline (`a80ce888a`) was advanced
# to `1402ce0132` (Sprint-02 red-hat remediation: close FIX-AUTHZ-FORGE-COVERAGE
# + defense-in-depth). That commit is the most recent commit touching
# merge_gate.rs that is NOT a GATES-008/Sprint-4 change. The intervening commits
# (353bbcdc1a STEER-001 adding steering fields to the MergeGateError carrier
# struct + 1402ce0132 fully-qualifying `but_authz::Authority::Merge`) are
# cross-cutting carrier/import changes; none of them touch classification logic
# (load_merge_governance_config, undefined_required_groups, enforce_merge_gate,
# classify_error, normalize_permissions, normalize_gates). The advanced baseline
# preserves the GATES-008 contract: GATES-008 must add ZERO lines to merge_gate.rs
# (whether classification or carrier).
#
# NEGATIVE_TEST (manual test): in a disposable worktree, delete one production
# line from crates/but-api/src/legacy/merge_gate.rs or rename the file, then run
# this script. It must exit non-zero before the change is restored.

baseline_sha="1402ce0132a464dfcb42ab26d5f26e26d9160ce7"
target="crates/but-api/src/legacy/merge_gate.rs"
repo_root="$(git rev-parse --show-toplevel)"

cd "${repo_root}"

if [[ ! -f "${target}" ]]; then
	echo "missing target file: ${target}" >&2
	exit 1
fi

if ! git cat-file -e "${baseline_sha}^{commit}"; then
	echo "missing baseline commit: ${baseline_sha}" >&2
	exit 1
fi

if ! git cat-file -e "${baseline_sha}:${target}"; then
	echo "baseline ${baseline_sha} does not contain ${target}" >&2
	exit 1
fi

if ! git diff --quiet --exit-code "${baseline_sha}...HEAD" -- "${target}"; then
	echo "${target} has committed changes since baseline ${baseline_sha}" >&2
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

echo "OK: ${target} is unchanged from baseline ${baseline_sha}"
