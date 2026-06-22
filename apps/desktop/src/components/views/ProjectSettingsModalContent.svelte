<script lang="ts">
	import GovernanceSettings from "$components/governance/GovernanceSettings.svelte";
	import CloudForm from "$components/projectSettings/CloudForm.svelte";
	import GeneralSettings from "$components/projectSettings/GeneralSettings.svelte";
	import GitForm from "$components/projectSettings/GitForm.svelte";
	import PreferencesForm from "$components/projectSettings/PreferencesForm.svelte";
	import SettingsModalLayout from "$components/settings/SettingsModalLayout.svelte";
	import ErrorBoundary from "$components/shared/ErrorBoundary.svelte";
	import { BACKEND } from "$lib/backend";
	import {
		createGovernanceRendererContract,
		type GovernanceRendererContract,
	} from "$lib/governance";
	import { projectSettingsPages } from "$lib/settings/projectSettingsPages";
	import { USER_SERVICE } from "$lib/user/userService.svelte";
	import { inject, injectOptional } from "@gitbutler/core/context";
	import { untrack } from "svelte";
	import type { ProjectSettingsModalState, ProjectSettingsPageId } from "$lib/state/uiState.svelte";

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
	// Dev-only override: set VITE_FORCE_ADMIN=true (e.g. in apps/desktop/.env.development.local) to
	// force admin so the admin-only settings (Permissions & Governance) show without a signed-in
	// admin. The `import.meta.env.DEV` guard makes this impossible to leak into a production build.
	const isAdmin = $derived(
		userService.user?.role === "admin" ||
			(import.meta.env.DEV && import.meta.env.VITE_FORCE_ADMIN === "true"),
	);
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
