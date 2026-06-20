<script lang="ts">
	import ProjectSettingsModalContent from "$components/settings/ProjectSettingsModalContent.svelte";
	import {
		UI_STATE,
		type ProjectSettingsModalState,
		type UiState,
	} from "$lib/state/uiState.svelte";
	import { USER_SERVICE, UserService } from "$lib/user/userService.svelte";
	import type { User } from "$lib/user/user";
	import { provide } from "@gitbutler/core/context";

	type UserRole = "admin" | "member";

	type Props = {
		role: UserRole;
		selectedId?: ProjectSettingsModalState["selectedId"];
	};

	const { role, selectedId }: Props = $props();

	const userService = {
		get user(): User {
			return {
				id: 1,
				name: "Settings Tester",
				email: "settings@example.com",
				locale: "en-US",
				created_at: "2026-06-20T00:00:00.000Z",
				updated_at: "2026-06-20T00:00:00.000Z",
				access_token: "desktop-ct-token",
				role,
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

	provide(USER_SERVICE, userService);
	provide(UI_STATE, uiState);
</script>

<ProjectSettingsModalContent
	data={{
		type: "project-settings",
		projectId: "ct-project",
		selectedId,
	}}
/>
