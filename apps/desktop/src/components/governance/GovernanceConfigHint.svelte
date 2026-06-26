<script lang="ts">
	import { Icon } from "@gitbutler/ui";

	type Props = {
		// Which governed file this surface reads from.
		file?: "agents" | "groups" | "gates";
		// The committed target ref the config is read at (display only).
		targetRef?: string;
		// CLI verb that edits this surface, e.g. `but perm` / `but group`.
		cli?: string;
	};

	const { file = "agents", targetRef, cli }: Props = $props();

	const fileName = $derived(
		file === "gates" ? ".gitbutler/gates.toml" : ".gitbutler/agents.toml"
	);
</script>

<div class="config-hint" data-testid="governance-config-hint">
	<Icon name="info" />
	<p class="config-hint__text">
		Defined in <code>{fileName}</code>{#if file !== "gates"} (or legacy
			<code>.gitbutler/permissions.toml</code>){/if}, committed to
		{#if targetRef}<code>{targetRef}</code>{:else}the target branch{/if}. To change it, edit that
		file and commit{#if cli}, or use <code>{cli}</code>{/if}.
	</p>
</div>

<style>
	.config-hint {
		display: flex;
		align-items: flex-start;
		padding: 8px;
		gap: 6px;
		border: 1px solid var(--border-2);
		border-radius: var(--radius-s);
		background: var(--bg-2);
		color: var(--text-2);
	}

	.config-hint__text {
		margin: 0;
		font-size: 12px;
		line-height: 1.5;
	}

	.config-hint code {
		padding: 0 2px;
		border-radius: var(--radius-s);
		background: var(--bg-1);
		color: var(--text-1);
		font-size: 11px;
	}
</style>
