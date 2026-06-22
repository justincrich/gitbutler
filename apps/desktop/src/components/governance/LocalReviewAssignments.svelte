<script lang="ts" module>
	import type { LocalReviewAssignmentEntry } from "$components/governance/LocalReviewView.svelte";

	type BadgeColor = "gray" | "safe" | "warning";

	const STATE_STYLES: Record<string, BadgeColor> = {
		pending: "gray",
		approved: "safe",
		changes_requested: "warning",
	};

	const STATE_LABELS: Record<string, string> = {
		pending: "Pending",
		approved: "Approved",
		changes_requested: "Changes Requested",
	};

	export function assignmentStateStyle(state: string): BadgeColor {
		return STATE_STYLES[state] ?? "gray";
	}

	export function assignmentStateLabel(state: string): string {
		return STATE_LABELS[state] ?? "Pending";
	}

	export function slug(value: string): string {
		return value.replace(/[^a-z0-9]+/gi, "-");
	}
</script>

<script lang="ts">
	import { Badge } from "@gitbutler/ui";

	type Props = {
		assignments: LocalReviewAssignmentEntry[];
	};

	let { assignments }: Props = $props();
</script>

{#if assignments.length === 0}
	<p class="local-review-assignments-empty" data-testid="local-review-assignments-empty">
		No reviewers assigned yet.
	</p>
{:else}
	<ul class="local-review-assignments" data-testid="local-review-assignments-list">
		{#each assignments as assignment (assignment.reviewer_principal)}
			{@const s = slug(assignment.reviewer_principal)}
			<li class="local-review-assignment-row" data-testid={`local-review-assignment-row-${s}`}>
				<span class="local-review-assignment-row__reviewer">{assignment.reviewer_principal}</span>
				<Badge
					testId={`local-review-assignment-chip-${s}`}
					style={assignmentStateStyle(assignment.state)}
					kind="soft"
					size="tag"
				>
					{assignmentStateLabel(assignment.state)}
				</Badge>
			</li>
		{/each}
	</ul>
{/if}

<style>
	.local-review-assignments {
		display: flex;
		flex-direction: column;
		margin: 0;
		padding: 0;
		gap: var(--clr-space-4, 8px);
		list-style: none;
	}

	.local-review-assignment-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--clr-space-4, 8px) var(--clr-space-8, 12px);
		gap: var(--clr-space-4, 8px);
		border: 1px solid var(--clr-border-2, var(--border-2));
		border-radius: var(--radius-s, 4px);
		background: var(--clr-bg-1, var(--bg-1));
		color: var(--clr-text-1, var(--text-1));
	}

	.local-review-assignment-row__reviewer {
		font-weight: 600;
	}

	.local-review-assignments-empty {
		margin: 0;
		color: var(--clr-text-2, var(--text-2));
	}
</style>
