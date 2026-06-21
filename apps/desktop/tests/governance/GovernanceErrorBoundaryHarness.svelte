<script lang="ts">
	import ProjectSettingsModalContent from "$components/settings/ProjectSettingsModalContent.svelte";
	import type {
		GovernanceAccess,
		GovernancePending,
		GovernancePrincipalsList,
		GovernanceRendererContract,
		GovernanceTarget,
	} from "$lib/governance";
	import {
		UI_STATE,
		type ProjectSettingsModalState,
		type UiState,
	} from "$lib/state/uiState.svelte";
	import type { User } from "$lib/user/user";
	import { USER_SERVICE, UserService } from "$lib/user/userService.svelte";
	import { provide } from "@gitbutler/core/context";

	type BoundaryMode = "normal" | "throwing-principals";

	type Props = {
		mode?: BoundaryMode;
	};

	const { mode = "normal" }: Props = $props();
	const userService = {
		get user(): User {
			return {
				id: 1,
				name: "Governance Boundary Tester",
				email: "governance-boundary@example.com",
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
	const governanceService: GovernanceRendererContract = {
		async readAccess(_projectId: string): Promise<GovernanceAccess> {
			return {
				authorities: ["administration:write"],
				hasAdminWrite: true,
				isReadOnly: false,
			};
		},
		async readPending(_target: GovernanceTarget): Promise<GovernancePending> {
			return { principals: [], pendingCount: 0 };
		},
		async readPrincipals(_target: GovernanceTarget): Promise<GovernancePrincipalsList> {
			if (mode === "throwing-principals") {
				return {
					principals: [
						{
							principalId: "broken-governance-child",
							ownGrants: undefined,
							inheritedGrants: [],
							groupMemberships: [],
							pending: false,
						},
					],
				} as unknown as GovernancePrincipalsList;
			}

			return {
				principals: [
					{
						principalId: "settings-agent",
						ownGrants: ["contents:read"],
						inheritedGrants: [],
						groupMemberships: [],
						pending: false,
					},
				],
			};
		},
		async commitPending(_target: GovernanceTarget) {
			throw new Error("Governance boundary harness does not commit");
		},
	};
	const data: ProjectSettingsModalState = {
		type: "project-settings",
		projectId: "ct-project",
		selectedId: "governance",
	};

	provide(USER_SERVICE, userService);
	provide(UI_STATE, uiState);
</script>

<ProjectSettingsModalContent {data} {governanceService} />
