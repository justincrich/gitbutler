<script lang="ts">
	import CloudForm from "$components/projectSettings/CloudForm.svelte";
	import ErrorBoundary from "$components/shared/ErrorBoundary.svelte";
	import GeneralSettings from "$components/projectSettings/GeneralSettings.svelte";
	import GitForm from "$components/projectSettings/GitForm.svelte";
	import GovernanceSettings from "$components/governance/GovernanceSettings.svelte";
	import PreferencesForm from "$components/projectSettings/PreferencesForm.svelte";
	import { BACKEND } from "$lib/backend";
	import {
		createGovernanceRendererContract,
		type GovernanceRendererContract,
	} from "$lib/governance";
	import SettingsModalLayout from "$components/settings/SettingsModalLayout.svelte";
	import { projectSettingsPages } from "$lib/settings/projectSettingsPages";
	import type { ProjectSettingsModalState, ProjectSettingsPageId } from "$lib/state/uiState.svelte";
	import { USER_SERVICE } from "$lib/user/userService.svelte";
	import { inject, injectOptional } from "@gitbutler/core/context";
	import { untrack } from "svelte";

	type Props = {
		data: ProjectSettingsModalState;
		governanceService?: GovernanceRendererContract;
	};

	const { data, governanceService: providedGovernanceService }: Props = $props();

	const userService = inject(USER_SERVICE);
	const backend = injectOptional(BACKEND, undefined);
	const governanceService = untrack(
		() =>
			providedGovernanceService ??
			(backend ? createGovernanceRendererContract(backend) : undefined),
	);
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
				<ErrorBoundary title="Governance settings failed to load" compact={false}>
					<GovernanceSettings projectId={data.projectId} service={governanceService} />
				</ErrorBoundary>
			{:else}
				Settings page {currentPage.id} not Found.
			{/if}
		{:else}
			Settings page {currentSelectedId} not Found.
		{/if}
	{/snippet}
</SettingsModalLayout>
