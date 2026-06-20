#!/usr/bin/env bash
set -euo pipefail

REQ_ID="${1:?requirement id required}"
FILE="apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md"

require_literal() {
	local needle="$1"
	if ! grep -Fq "$needle" "$FILE"; then
		echo "missing: $needle" >&2
		exit 1
	fi
	echo "found: $needle"
}

case "$REQ_ID" in
	AC-1|TC-1)
		require_literal 'Toggle.svelte` exposes `disabled`'
		require_literal 'Pass `disabled=true`'
		require_literal 'TagInput.svelte` exposes `readonly`'
		require_literal 'Pass `readonly=true`'
		require_literal 'Button.svelte` exposes `disabled`'
		require_literal 'SegmentControl` is non-interactive'
		require_literal 'KebabButton.svelte` opens the menu'
		require_literal 'ContextMenuItem.svelte` exposes `disabled`'
		require_literal 'render those mutating entries as `ContextMenuItem disabled=true`'
		;;
	AC-2|TC-2)
		require_literal '`style="info"`'
		require_literal '`outlined=true`'
		require_literal 'Read-only: administration:write is required to change governance settings'
		require_literal 'Do not pass `primaryLabel`, `secondaryLabel`, or `tertiaryLabel`'
		require_literal 'banner has no action buttons'
		;;
	AC-3|TC-3)
		require_literal 'SettingsModalLayout.svelte:53'
		require_literal 'pages.filter((p) => !p.adminOnly || isAdmin)'
		require_literal 'Read-only is the functional'
		require_literal 'permission state for a viewer who can navigate'
		require_literal 'These layers are independent'
		require_literal '`administration:write`'
		;;
	AC-4|TC-4)
		require_literal 'GovernanceSettings.svelte` derives one'
		require_literal 'boolean, `isReadOnly`, from the `administration:write` check'
		require_literal 'prop to `PrincipalsList`, `PrincipalEditor`, `GroupsList`, and'
		require_literal '`BranchGatesList`'
		require_literal 'Child components consume the prop; they do not re-derive'
		require_literal '`GovernancePendingBanner` is hidden'
		require_literal 'commit affordance is unavailable'
		;;
	*)
		echo "unknown requirement id: $REQ_ID" >&2
		exit 2
		;;
esac
