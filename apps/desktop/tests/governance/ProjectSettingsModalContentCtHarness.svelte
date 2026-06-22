<script lang="ts">
	import GovernanceSettings from "$components/governance/GovernanceSettings.svelte";
	import ProjectSettingsModalContent from "$components/views/ProjectSettingsModalContent.svelte";
	import { BACKEND, type IBackend } from "$lib/backend";
	import { UI_STATE, type UiState } from "$lib/state/uiState.svelte";
	import { USER_SERVICE, UserService } from "$lib/user/userService.svelte";
	import { provide } from "@gitbutler/core/context";
	import type { User } from "$lib/user/user";

	// A real `get user()` (not Object.create(prototype), whose getter dereferences an
	// uninitialized userQuery and throws). Admin so the governance settings page renders.
	const userService = {
		get user(): User {
			return {
				id: 1,
				name: "CT Tester",
				email: "ct@example.com",
				locale: "en-US",
				created_at: "2026-06-20T00:00:00.000Z",
				updated_at: "2026-06-20T00:00:00.000Z",
				access_token: "desktop-ct-token",
				role: "admin",
				supporter: false,
			};
		},
	} as unknown as UserService;
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
				return {
					authorities: ["administration:write"],
					not_configured: false,
					target_ref: "refs/remotes/origin/main",
				} as T;
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
