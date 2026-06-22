<script lang="ts">
	import ReduxResult from "$components/shared/ReduxResult.svelte";
	import SettingsSection from "$components/shared/SettingsSection.svelte";
	import { PROJECTS_SERVICE } from "$lib/project/projectsService";
	import { inject } from "@gitbutler/core/context";
	import { CardGroup, InfoMessage, Toggle } from "@gitbutler/ui";

	const { projectId }: { projectId: string } = $props();
	const projectsService = inject(PROJECTS_SERVICE);
	const projectQuery = $derived(projectsService.getProject(projectId));

	// Committed value from the project store. The `?? true` is the DefaultTrue
	// UI rule (LPR-006 AC-1 serde behavior mirrored at the UI layer): an older
	// project JSON without the key deserializes to true on the backend, and the
	// UI must render to match. There is intentionally NO `?? false` anywhere —
	// a missing field always reads as local.
	const committed = $derived(
		(projectQuery.response as { keep_reviews_local?: boolean | null } | undefined)
			?.keep_reviews_local ?? true,
	);

	// Optimistic-update state. `undefined` means "no pending optimistic write —
	// follow `committed`". On click we set this to the new value; on write
	// success it stays (the next refetch will align `committed`); on write
	// error we revert to the prior effective value.
	let optimistic = $state<boolean | undefined>(undefined);
	let writeError = $state<string | undefined>(undefined);

	const checked = $derived(optimistic ?? committed);

	async function handleChange(value: boolean): Promise<void> {
		const project = projectQuery.response;
		// Defensive: the toggle only renders inside <ReduxResult children> once
		// the project has loaded, so this should always be defined here.
		if (!project) return;

		const previous = checked;
		// Optimistic flip — the toggle visually reflects the new value immediately.
		optimistic = value;
		writeError = undefined;

		try {
			// Same write path PreferencesForm / ForgeForm use: spread the project
			// and overwrite the single field. This is a per-project operator
			// preference under the R12 trusted-desktop model — not a ref-pinned
			// config mutation, not an admin-gated write. The R21 residual (an
			// untrusted project-store write flips it) stays named, not closed.
			await projectsService.updateProject({ ...project, keep_reviews_local: value });
		} catch (err) {
			// Revert the optimistic flip; surface the error in a danger InfoMessage.
			optimistic = previous;
			writeError = err instanceof Error ? err.message : String(err);
		}
	}
</script>

<ReduxResult {projectId} result={projectQuery.result}>
	{#snippet children(_project)}
		<SettingsSection gap={8}>
			{#if writeError}
				<InfoMessage style="danger" error={writeError} testId="keepReviewsLocalError">
					{#snippet title()}
						Couldn't save the keep-reviews-local preference
					{/snippet}
					{#snippet content()}
						The preference wasn't persisted — reverting to the last committed value. (Local project
						preference, not a security boundary.)
					{/snippet}
				</InfoMessage>
			{/if}

			<CardGroup.Item standalone labelFor="keepReviewsLocal">
				{#snippet title()}
					Keep agent reviews local
				{/snippet}

				{#snippet caption()}
					{#if checked}
						Agent-authored PRs stay on the local review layer — no GitHub PR is opened. Change this
						only if you want agent reviews mirrored to your forge. (This is a local project
						preference, not a security boundary — the project store is not independently verified.)
					{:else}
						Agent-authored PRs will be mirrored to your forge when approved. Internal principal
						identifiers may be disclosed to the forge API; ensure all principals have forge accounts
						before enabling. (See: local project preference, not a security boundary.)
					{/if}
				{/snippet}

				{#snippet actions()}
					<Toggle
						id="keepReviewsLocal"
						testId="keepReviewsLocalToggle"
						{checked}
						onchange={handleChange}
					/>
				{/snippet}
			</CardGroup.Item>
		</SettingsSection>
	{/snippet}
</ReduxResult>
