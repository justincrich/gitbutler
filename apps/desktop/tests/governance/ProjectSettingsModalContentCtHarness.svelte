<script lang="ts">
	import GovernanceSettings from "$components/governance/GovernanceSettings.svelte";
	import ProjectSettingsModalContent from "$components/settings/ProjectSettingsModalContent.svelte";
	import { BACKEND, type IBackend } from "$lib/backend";
	import { UI_STATE, type UiState } from "$lib/state/uiState.svelte";
	import { USER_SERVICE, UserService } from "$lib/user/userService.svelte";
	import { provide } from "@gitbutler/core/context";

	const userService = Object.create(UserService.prototype) as UserService;
	const uiState = {
		global: {
			scrollbarVisibilityState: {
				current: "scroll",
			},
		},
	} as unknown as UiState;
	const backend = {
		invoke: async <T,>(command: string): Promise<T> => {
			if (command === "governance_status_read") {
				return { authorities: ["administration:write"] } as T;
			}

			if (command === "governance_pending") {
				return { principals: [], pendingCount: 0 } as T;
			}

			throw new Error(`Unexpected backend command: ${command}`);
		},
	} as unknown as IBackend;

	provide(BACKEND, backend);
	provide(USER_SERVICE, userService);
	provide(UI_STATE, uiState);
</script>

<ProjectSettingsModalContent
	data={{
		type: "project-settings",
		projectId: "ct-project",
		selectedId: "governance",
	}}
/>

<div data-testid="governance-proof">
	<GovernanceSettings />
</div>
