<script lang="ts">
	import PrincipalsList from "$components/governance/PrincipalsList.svelte";
	import { BACKEND, type IBackend } from "$lib/backend";
	import { provide } from "@gitbutler/core/context";
	import { writable } from "svelte/store";
	import type { GovernancePrincipalsList } from "$lib/governance";

	const calls = $state<string[]>([]);
	const targetArgs = $state<unknown[]>([]);
	const result: GovernancePrincipalsList = {
		principals: [
			{
				principalId: "backend-agent",
				ownGrants: ["contents:read"],
				inheritedGrants: [{ authority: "contents:write", sourceLabel: "group: platform" }],
				groupMemberships: ["platform"],
				pending: false,
			},
		],
	};

	async function invoke<T>(command: string, args: unknown): Promise<T> {
		calls.push(command);
		targetArgs.push(args);

		if (command === "governance_principals_list") {
			return result as T;
		}

		throw new Error(`Unexpected backend command: ${command}`);
	}

	const backend = {
		platformName: "test",
		systemTheme: writable(null),
		invoke,
	} as unknown as IBackend;

	provide(BACKEND, backend);
</script>

<PrincipalsList projectId="project-1" targetRef="refs/remotes/origin/main" />

<output data-testid="principals-list-backend-calls">{calls.join(",")}</output>
<output data-testid="principals-list-backend-args">{JSON.stringify(targetArgs)}</output>
