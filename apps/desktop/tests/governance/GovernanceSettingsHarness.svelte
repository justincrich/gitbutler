<script lang="ts">
	import GovernanceSettings from "$components/governance/GovernanceSettings.svelte";
	import type {
		GovernanceAccess,
		GovernancePending,
		GovernancePrincipalsList,
		GovernanceRendererContract,
		GovernanceTarget,
		PrincipalListEntry,
	} from "$lib/governance";

	type UserRole = "admin" | "member";

	type Props = {
		pendingCount: number;
		pendingCountAfterCommit?: number;
		hasAdminWrite?: boolean;
		role?: UserRole;
		principals?: PrincipalListEntry[];
	};

	const {
		pendingCount,
		pendingCountAfterCommit = 0,
		hasAdminWrite = true,
		role = "member",
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

	let currentPendingCount = $state(pendingCount);
	let commitCount = $state(0);
	let lastCommitMessage = $state("");

	function pending(): GovernancePending {
		return {
			pendingCount: currentPendingCount,
			principals: [],
		};
	}

	const service: GovernanceRendererContract = {
		async readPending(_target: GovernanceTarget) {
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
			return {
				commitId: "ct-governance-commit",
				message: "chore: update governance config",
				committedPaths: ["governance.toml"],
			};
		},
	};
</script>

<GovernanceSettings projectId="ct-project" targetRef="refs/remotes/origin/main" {service} />

<output data-testid="governance-user-role">{role}</output>
<output data-testid="governance-commit-count">{commitCount}</output>
<output data-testid="governance-commit-message">{lastCommitMessage}</output>
