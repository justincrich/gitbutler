<script lang="ts">
	import GovernanceSettings from "$components/governance/GovernanceSettings.svelte";
	import { IpcError } from "$lib/error/normalizedError";
	import { untrack } from "svelte";
	import type {
		GovernanceAccess,
		GovernancePending,
		GovernancePrincipalsList,
		GovernanceRendererContract,
		GovernanceTarget,
		PrincipalListEntry,
	} from "$lib/governance";
	import type { GroupListEntry } from "@gitbutler/but-sdk";

	type UserRole = "admin" | "member";

	type Props = {
		pendingCount: number;
		pendingCountAfterCommit?: number;
		hasAdminWrite?: boolean;
		readFailure?: boolean;
		role?: UserRole;
		principals?: PrincipalListEntry[];
		pendingGroups?: string[];
	};

	const {
		pendingCount,
		pendingCountAfterCommit = 0,
		hasAdminWrite = true,
		readFailure = false,
		role = "member",
		pendingGroups = [],
		principals = [
			{
				principalId: "settings-agent",
				ownGrants: ["contents:read"],
				inheritedGrants: [],
				groupMemberships: [],
				pending: false,
			},
		],
	}: Props = $props();

	let currentPendingCount = $state(untrack(() => pendingCount));
	let commitCount = $state(0);
	let readPendingCount = $state(0);
	let lastCommitMessage = $state("");
	let currentPendingGroups = $state(untrack(() => [...pendingGroups]));

	function pending(): GovernancePending {
		return {
			pendingCount: currentPendingCount,
			principals: [],
		};
	}

	const service: GovernanceRendererContract = {
		async readPending(_target: GovernanceTarget) {
			readPendingCount += 1;
			if (readFailure) {
				throw new IpcError(
					{
						code: "network.error",
						message: "Backend unreachable",
						remediation_hint: "Check the desktop backend connection and retry.",
					},
					"governance_status_read",
				);
			}
			return pending();
		},
		async readPrincipals(_target: GovernanceTarget): Promise<GovernancePrincipalsList> {
			return { principals };
		},
		async readAccess(_projectId: string): Promise<GovernanceAccess> {
			return {
				authorities: hasAdminWrite ? ["administration:write"] : ["administration:read"],
				hasAdminWrite,
				isReadOnly: !hasAdminWrite,
			};
		},
		async commitPending(_target: GovernanceTarget) {
			commitCount += 1;
			lastCommitMessage = "chore: update governance config";
			currentPendingCount = pendingCountAfterCommit;
			if (pendingCountAfterCommit === 0) {
				currentPendingGroups = [];
			}
			return {
				commitId: "ct-governance-commit",
				message: "chore: update governance config",
				committedPaths: ["governance.toml"],
			};
		},
	};

	const groups: GroupListEntry[] = [
		{
			name: "eng",
			authorities: ["contents:write"],
			members: ["settings-agent"],
		},
	];
</script>

<GovernanceSettings
	projectId="ct-project"
	targetRef="refs/remotes/origin/main"
	{service}
	initialGroups={groups}
	initialPendingGroups={currentPendingGroups}
/>

<output data-testid="governance-user-role">{role}</output>
<output data-testid="governance-commit-count">{commitCount}</output>
<output data-testid="governance-read-pending-count">{readPendingCount}</output>
<output data-testid="governance-commit-message">{lastCommitMessage}</output>
