<script lang="ts">
	import { Button, InfoMessage } from "@gitbutler/ui";

	type Props = {
		pendingCount: number;
		onCommit: () => void;
	};

	const { pendingCount, onCommit }: Props = $props();
</script>

{#if pendingCount > 0}
	<div>
		<div data-testid="governance-pending-banner">
			<InfoMessage filled outlined={false} style="warning">
				{#snippet title()}{pendingCount} pending changes{/snippet}
				{#snippet content()}
					Commit governance changes to the configured target branch.
					<Button style="warning" testId="governance-commit-button" onclick={onCommit}>
						Commit changes
					</Button>
				{/snippet}
			</InfoMessage>
		</div>
	</div>
{/if}
