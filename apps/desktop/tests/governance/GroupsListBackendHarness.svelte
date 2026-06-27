<script lang="ts">
	import GroupsList from "$components/governance/GroupsList.svelte";
	import { BACKEND, type IBackend } from "$lib/backend";
	import { provide } from "@gitbutler/core/context";
	import { writable } from "svelte/store";
	import type { GroupListOutcome } from "@gitbutler/but-sdk";

	const calls = $state<string[]>([]);
	const callArgs = $state<unknown[]>([]);
	const result: GroupListOutcome = {
		groups: [
			{
				name: "code-reviewers",
				authorities: ["reviews:write", "comments:write", "contents:read"],
				members: ["rust-reviewer", "security-reviewer"],
			},
		],
	};

	async function invoke<T>(command: string, args: unknown): Promise<T> {
		calls.push(command);
		callArgs.push(args);

		// The renderer read goes through the ungated `governance_groups_list`, NOT the
		// governed `group_list` CLI command that denies an unregistered desktop process.
		if (command === "governance_groups_list") {
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

<GroupsList projectId="project-1" targetRef="refs/remotes/origin/master" />

<output data-testid="groups-list-backend-calls">{calls.join(",")}</output>
<output data-testid="groups-list-backend-args">{JSON.stringify(callArgs)}</output>
