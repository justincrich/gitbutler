#!/usr/bin/env bash
set -euo pipefail

# Checks the GATES-006 structural invariant: review requirement enforcement must
# not branch on human-vs-AI labels or hardcoded role/group names. Fixture role
# names belong in tests; this script targets only the evaluator source.

target="crates/but-api/src/legacy/review_requirement.rs"
repo_root="$(git rev-parse --show-toplevel)"
target_path="${repo_root}/${target}"

if [[ ! -f "${target_path}" ]]; then
	echo "missing target file: ${target}" >&2
	exit 1
fi

forbidden='(^|[^[:alnum:]_])(is_bot|is_human|human|ai|bot|role|admin|admins|owner|owners|implementer|reviewer|maintainer|maintainers|code-reviewers)([^[:alnum:]_]|$)'

if grep -nEi "${forbidden}" "${target_path}"; then
	echo "forbidden human/AI/role literal found in ${target}" >&2
	exit 1
fi

echo "OK: ${target} contains no forbidden human/AI/role literals"
