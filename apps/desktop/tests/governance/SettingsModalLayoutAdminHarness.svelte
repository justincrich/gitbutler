<script lang="ts">
	import ProjectSettingsModalContent from "$components/settings/ProjectSettingsModalContent.svelte";
	import { BACKEND, type IBackend } from "$lib/backend";
	import type {
		GovernanceAccess,
		GovernancePending,
		GovernanceRendererContract,
		GovernanceTarget,
	} from "$lib/governance";
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
	const backendCalls = $state<Array<{ command: string; args: unknown }>>([]);
	let backendCallsJson = $state("[]");
	globalThis.__governanceBackendCalls = [];

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

	const backend = {
		invoke: async <T,>(command: string, args: unknown): Promise<T> => {
			backendCalls.push({ command, args });
			backendCallsJson = JSON.stringify(backendCalls);
			globalThis.__governanceBackendCalls = backendCalls;

			if (command === "governance_status_read") {
				return { authorities: ["administration:write"] } as T;
			}

			if (command === "governance_pending") {
				return { principals: [], pendingCount: 0 } as T;
			}

			throw new Error(`Unexpected backend command: ${command}`);
		},
	} as unknown as IBackend;
	const governanceService: GovernanceRendererContract = {
		async readAccess(projectId: string): Promise<GovernanceAccess> {
			backendCalls.push({ command: "governance_status_read", args: { projectId } });
			backendCallsJson = JSON.stringify(backendCalls);
			globalThis.__governanceBackendCalls = backendCalls;
			return {
				authorities: ["administration:write"],
				hasAdminWrite: true,
				isReadOnly: false,
			};
		},
		async readPending(target: GovernanceTarget): Promise<GovernancePending> {
			backendCalls.push({ command: "governance_pending", args: target });
			backendCallsJson = JSON.stringify(backendCalls);
			globalThis.__governanceBackendCalls = backendCalls;
			return { principals: [], pendingCount: 0 };
		},
		async commitPending() {
			throw new Error("Modal harness does not commit governance changes");
		},
	};

	provide(BACKEND, backend);
	provide(USER_SERVICE, userService);
	provide(UI_STATE, uiState);
</script>

<ProjectSettingsModalContent
	{governanceService}
	data={{
		type: "project-settings",
		projectId: "ct-project",
		selectedId,
	}}
/>

<output data-testid="governance-backend-calls">{backendCallsJson}</output>
