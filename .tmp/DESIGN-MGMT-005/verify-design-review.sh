#!/usr/bin/env bash
set -euo pipefail

DOC="apps/desktop/src/components/governance/DESIGN-ANNOTATIONS.md"
EVIDENCE_DIR=".tmp/DESIGN-MGMT-005"
RESULTS="$EVIDENCE_DIR/requirement-results.txt"
SUMMARY="$EVIDENCE_DIR/verification-summary.json"

mkdir -p "$EVIDENCE_DIR"
: > "$RESULTS"

total=0
passed=0

log() {
	printf '%s\n' "$*" | tee -a "$RESULTS"
}

check_requirement() {
	local id="$1"
	local description="$2"
	shift 2
	total=$((total + 1))

	local missing=()
	local pattern
	for pattern in "$@"; do
		if ! grep -Fq -- "$pattern" "$DOC"; then
			missing+=("$pattern")
		fi
	done

	if ((${#missing[@]} == 0)); then
		passed=$((passed + 1))
		log "PASS $id - $description"
	else
		log "FAIL $id - $description"
		for pattern in "${missing[@]}"; do
			log "  missing: $pattern"
		done
		return 1
	fi
}

failed=0

check_requirement \
	"AC-1" \
	"inherited row uses disabled Toggle, gray/soft group Badge, muted inherited text, no pending Badge, default background" \
	"Inherited group grant" \
	"Toggle disabled=true" \
	"Badge style='gray' kind='soft'" \
	"[group: {groupName}]" \
	"── inherited ──" \
	"var(--text-3)" \
	"No pending \`Badge\` on inherited rows" \
	"default \`var(--bg-1)\`" || failed=1

check_requirement \
	"AC-2" \
	"own-grant rows use enabled checked variants, own-grant source, Toggle grant, warning/soft pending Badge" \
	"Own-grant active" \
	"Own-grant inactive" \
	"\`own grant\` in \`var(--text-2)\`" \
	"Toggle disabled=false checked=true" \
	"Toggle disabled=false checked=false" \
	"this \`Toggle\` is the GRANT column control" \
	"Badge style='warning' kind='soft'" || failed=1

check_requirement \
	"AC-3" \
	"both row is explicit, inherited source wins, disabled, no pending Badge, cannot revoke until Groups tab removal" \
	"Both own-grant and group-inherited" \
	"Inherited source takes precedence" \
	"not \`own grant\`" \
	"Toggle disabled=true" \
	"No pending \`Badge\`" \
	"own grant cannot be revoked from \`PrincipalEditor\` while the inherited grant exists" \
	"Groups tab removal is required first" \
	"This is the explicit \"both\" row" || failed=1

check_requirement \
	"AC-4" \
	"SegmentControl presets drive only own-grant Toggles in local UI state with no immediate SDK write" \
	"### SegmentControl Interaction" \
	"Selecting a preset through \`onselect\` updates local UI state only" \
	"updates local UI state only and performs no" \
	"immediate SDK write" \
	"preset desugars to own-grant \`Toggle\` states" \
	"Inherited rows are never" \
	"touched by the preset" \
	"their \`Toggle disabled=true\` state stays disabled and" || failed=1

check_requirement \
	"AC-5" \
	"union semantics state own union group, inherited cannot be revoked in PrincipalEditor, Groups tab is revoke path, with example" \
	"Effective permission is own ∪ group" \
	"The inherited grant cannot be revoked from \`PrincipalEditor\`" \
	"choosing the \`read\` preset does not remove that effective permission" \
	"removing Alice from \`eng\` in the Groups tab" \
	"revoke by removing Alice from \`eng\` in the Groups tab" || failed=1

check_requirement \
	"AC-6" \
	"GROUPS region uses TagInput memberships, staged removals, Select/SelectItem add, readonly/disabled mode" \
	"### Groups Region" \
	"The editor's GROUPS region uses \`TagInput\`" \
	"Removing a tag with the component's remove affordance creates a staged group" \
	"removal in local editor state" \
	"batch-saved with \`[Save changes]\`" \
	"\`[+ Add to group]\` uses \`Select\`" \
	"\`SelectItem\`" \
	"\`TagInput readonly=true\`" \
	"\`Select disabled\`" || failed=1

check_requirement \
	"TC-1" \
	"contract names inherited Toggle disabled, Badge gray/soft, inherited text, no row-background change" \
	"Toggle disabled=true" \
	"Badge style='gray' kind='soft'" \
	"── inherited ──" \
	"var(--text-3)" \
	"row background remains default \`var(--bg-1)\`" || failed=1

check_requirement \
	"TC-2" \
	"contract names own Toggle checked variants, own grant text, pending warning/soft Badge" \
	"Toggle disabled=false checked=true" \
	"Toggle disabled=false checked=false" \
	"\`own grant\` in \`var(--text-2)\`" \
	"Badge style='warning' kind='soft'" || failed=1

check_requirement \
	"TC-3" \
	"contract names both row, group Badge not own grant, inherited text, no pending Badge, Groups-tab removal first, example" \
	"This is the explicit \"both\" row" \
	"Inherited source takes precedence" \
	"not \`own grant\`" \
	"── inherited ──" \
	"No pending \`Badge\`" \
	"Groups tab removal is required first" \
	"\`alice\` | \`contents:write\` with explicit own grant and group \`eng\` inheritance" || failed=1

check_requirement \
	"TC-4" \
	"contract states SegmentControl affects own-grant Toggles only and onselect updates local UI state only" \
	"Selecting a preset through \`onselect\` updates local UI state only" \
	"preset desugars to own-grant \`Toggle\` states" \
	"Inherited rows are never" \
	"touched by the preset" || failed=1

check_requirement \
	"TC-5" \
	"contract states inherited grants cannot be revoked from PrincipalEditor; Groups tab revoke path; example row present" \
	"The inherited grant cannot be revoked from \`PrincipalEditor\`" \
	"removing Alice from \`eng\` in the Groups tab" \
	"\`alice\` | \`contents:write\`" || failed=1

check_requirement \
	"TC-6" \
	"contract names TagInput group chips, Select/SelectItem add-to-group, readonly/disabled treatment" \
	"The editor's GROUPS region uses \`TagInput\`" \
	"\`[+ Add to group]\` uses \`Select\`" \
	"\`SelectItem\`" \
	"\`TagInput readonly=true\`" \
	"\`Select disabled\`" || failed=1

status="passed"
if ((failed != 0)); then
	status="failed"
fi

{
	printf '{\n'
	printf '  "task_id": "DESIGN-MGMT-005",\n'
	printf '  "artifact": "%s",\n' "$DOC"
	printf '  "requirements_total": %s,\n' "$total"
	printf '  "requirements_passed": %s,\n' "$passed"
	printf '  "status": "%s",\n' "$status"
	printf '  "typecheck_lint": {\n'
	printf '    "status": "skipped",\n'
	printf '    "exit_code": 0,\n'
	printf '    "reason": "Design Markdown artifact only; prompt allows typecheck/lint recorded as skipped/exit 0."\n'
	printf '  },\n'
	printf '  "results_path": "%s"\n' "$RESULTS"
	printf '}\n'
} > "$SUMMARY"

log "SUMMARY $status - $passed/$total requirements passed"
log "SUMMARY_JSON $SUMMARY"

if ((failed != 0)); then
	exit 1
fi
