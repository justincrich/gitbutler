<script lang="ts">
	import CloudForm from "$components/projectSettings/CloudForm.svelte";
	import GeneralSettings from "$components/projectSettings/GeneralSettings.svelte";
	import GitForm from "$components/projectSettings/GitForm.svelte";
	import GovernanceSettings from "$components/governance/GovernanceSettings.svelte";
	import PreferencesForm from "$components/projectSettings/PreferencesForm.svelte";
	import SettingsModalLayout from "$components/settings/SettingsModalLayout.svelte";
	import { projectSettingsPages } from "$lib/settings/projectSettingsPages";
	import type { ProjectSettingsModalState, ProjectSettingsPageId } from "$lib/state/uiState.svelte";
	import { USER_SERVICE } from "$lib/user/userService.svelte";
	import { inject } from "@gitbutler/core/context";

	type Props = {
		data: ProjectSettingsModalState;
	};

	const { data }: Props = $props();

	const userService = inject(USER_SERVICE);
	const isAdmin = $derived(userService.user?.role === "admin");
	const pages = projectSettingsPages;

	let currentSelectedId = $derived(data.selectedId || pages.at(0)?.id);

	function selectPage(pageId: ProjectSettingsPageId) {
		currentSelectedId = pageId;
	}
</script>

<SettingsModalLayout
	title="Project settings"
	{pages}
	selectedId={currentSelectedId}
	{isAdmin}
	onSelectPage={selectPage}
>
	{#snippet content({ currentPage })}
		{#if currentPage}
			{#if currentPage.id === "project"}
				<GeneralSettings projectId={data.projectId} />
			{:else if currentPage.id === "git"}
				<GitForm projectId={data.projectId} />
			{:else if currentPage.id === "ai"}
				<CloudForm projectId={data.projectId} />
			{:else if currentPage.id === "experimental"}
				<PreferencesForm projectId={data.projectId} />
			{:else if currentPage.id === "governance"}
				<GovernanceSettings />
			{:else}
				Settings page {currentPage.id} not Found.
			{/if}
		{:else}
			Settings page {currentSelectedId} not Found.
		{/if}
	{/snippet}
</SettingsModalLayout>
