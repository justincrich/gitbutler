<script lang="ts">
	import BranchGatesList from "$components/governance/BranchGatesList.svelte";
	import { BACKEND, type IBackend } from "$lib/backend";
	import { provide } from "@gitbutler/core/context";
	import { writable } from "svelte/store";
	import type {
		BranchGateEntry,
		BranchGatesOutcome,
		BranchProtectionInput,
	} from "$components/governance/BranchGatesList.svelte";

	type Props = {
		scenario?:
			| "seeded_gates_two_branches"
			| "seeded_empty_gates"
			| "seeded_gates_readonly"
			| "seeded_write_denied";
		isReadOnly?: boolean;
	};

	type BackendCall = {
		command: string;
		args: unknown;
	};

	type GroupEntry = {
		name: string;
		authorities: string[];
		members: string[];
	};

	const { scenario = "seeded_gates_two_branches", isReadOnly = false }: Props = $props();
	const projectId = "project-1";
	const targetRef = "refs/remotes/origin/main";

	const definedGroups: GroupEntry[] = [
		{
			name: "eng",
			authorities: ["reviews:write"],
			members: ["alice"],
		},
		{
			name: "security",
			authorities: ["reviews:write"],
			members: ["bob"],
		},
		{
			name: "platform",
			authorities: ["reviews:write"],
			members: ["carol"],
		},
	];

	const seededGatesTwoBranches: BranchGateEntry[] = [
		{
			name: "main",
			protected: true,
			min_approvals: 2,
			require_distinct_from_author: true,
			require_approval_from_group: ["eng", "security"],
			pending: false,
		},
		{
			name: "release",
			protected: true,
			min_approvals: 1,
			require_distinct_from_author: false,
			require_approval_from_group: ["platform"],
			pending: false,
		},
	];

	let calls = $state<BackendCall[]>([]);
	let currentBranches = $state<BranchGateEntry[]>(
		scenario === "seeded_empty_gates" ? [] : cloneGates(seededGatesTwoBranches),
	);

	function cloneGate(gate: BranchGateEntry): BranchGateEntry {
		return {
			...gate,
			require_approval_from_group: [...gate.require_approval_from_group],
		};
	}

	function cloneGates(gates: BranchGateEntry[]): BranchGateEntry[] {
		return gates.map(cloneGate);
	}

	function readOutcome(): BranchGatesOutcome {
		return {
			branches: cloneGates(currentBranches),
			caveat: targetRef,
		};
	}

	function updateBranches(branch: string, protection: BranchProtectionInput) {
		const nextBranch: BranchGateEntry = {
			name: branch,
			protected: protection.protected,
			min_approvals: protection.min_approvals ?? 0,
			require_distinct_from_author: protection.require_distinct_from_author ?? false,
			require_approval_from_group: protection.require_approval_from_group ?? [],
			pending: true,
		};

		currentBranches = currentBranches.some((entry) => entry.name === branch)
			? currentBranches.map((entry) => (entry.name === branch ? nextBranch : entry))
			: [...currentBranches, nextBranch];
	}

	async function invoke<T>(command: string, args: unknown): Promise<T> {
		calls = [...calls, { command, args }];

		if (command === "branch_gates_read") {
			return readOutcome() as T;
		}

		if (command === "group_list") {
			return { groups: definedGroups } as T;
		}

		if (command === "branch_gates_update") {
			if (scenario === "seeded_write_denied") {
				return {
					branches: cloneGates(currentBranches),
					caveat: "perm.denied Permission denied",
				} as T;
			}

			const update = args as {
				branch: string;
				protection: BranchProtectionInput;
			};
			updateBranches(update.branch, update.protection);
			return readOutcome() as T;
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

<BranchGatesList {projectId} {targetRef} {isReadOnly} />

<output data-testid="branch-gates-backend-calls">{JSON.stringify(calls)}</output>
<output data-testid="branch-gates-backend-branches">{JSON.stringify(currentBranches)}</output>
