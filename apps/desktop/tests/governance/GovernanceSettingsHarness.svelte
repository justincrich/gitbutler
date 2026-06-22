<script lang="ts">
	import GovernanceSettings from "$components/governance/GovernanceSettings.svelte";
	import { BACKEND, type IBackend } from "$lib/backend";
	import { IpcError } from "$lib/error/normalizedError";
	import { RULES_SERVICE } from "$lib/rules/rulesService.svelte";
	import { UI_STATE, type UiState } from "$lib/state/uiState.svelte";
	import { STACK_SERVICE } from "$lib/stacks/stackService.svelte";
	import { setEphemeralStorageItem } from "@gitbutler/shared/persisted";
	import { provide } from "@gitbutler/core/context";
	import { QueryStatus } from "@reduxjs/toolkit/query";
	import { untrack } from "svelte";
	import { writable } from "svelte/store";
	import type {
		BranchGateEntry,
		BranchGatesOutcome,
		BranchProtectionInput,
	} from "$components/governance/BranchGatesList.svelte";
	import type {
		GovernanceAccess,
		GovernancePending,
		GovernancePrincipalsList,
		GovernanceRendererContract,
		GovernanceTarget,
		PrincipalListEntry,
	} from "$lib/governance";
	import type { WorkspaceRule } from "$lib/rules/rule";
	import type RulesService from "$lib/rules/rulesService.svelte";
	import type { StackService } from "$lib/stacks/stackService.svelte";
	import type { GroupListEntry } from "@gitbutler/but-sdk";

	type UserRole = "admin" | "member";

	type BackendCall = {
		command: string;
		args: unknown;
	};

	type BranchGroupEntry = {
		name: string;
		authorities: string[];
		members: string[];
	};

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

	const projectId = "ct-project";
	const targetRef = "refs/remotes/origin/main";
	const drawerPersistId = `rules-drawer-${projectId}`;
	if (typeof window !== "undefined") {
		setEphemeralStorageItem(drawerPersistId, false, 1440);
	}

	let currentPendingCount = $state(untrack(() => pendingCount));
	let commitCount = $state(0);
	let readPendingCount = $state(0);
	let lastCommitMessage = $state("");
	let currentPendingGroups = $state(untrack(() => [...pendingGroups]));
	let backendCalls = $state<BackendCall[]>([]);
	let workspaceRulesCallCount = $state(0);
	let principalRulesCallCount = $state(0);
	let lastWorkspaceProjectId = $state("");
	let lastPrincipalProjectId = $state("");
	let lastPrincipalId = $state("");

	const branchGroups: BranchGroupEntry[] = [
		{
			name: "eng",
			authorities: ["reviews:write"],
			members: ["settings-agent"],
		},
		{
			name: "security",
			authorities: ["reviews:write"],
			members: ["reviewer-b"],
		},
	];

	let currentBranches = $state<BranchGateEntry[]>([
		{
			name: "main",
			protected: true,
			min_approvals: 2,
			require_distinct_from_author: true,
			require_approval_from_group: ["eng"],
			pending: false,
		},
	]);

	const workspaceRules = [
		createRule("rule-A1", "src/a1.ts"),
		createRule("rule-A2", "src/a2.ts"),
		createRule("rule-B1", "src/b1.ts"),
	];

	const rulesByPrincipal: Record<string, WorkspaceRule[]> = {
		"agent:codex-staging": [workspaceRules[0]!, workspaceRules[1]!],
		"agent:cursor-bot": [workspaceRules[2]!],
		"settings-agent": [workspaceRules[0]!],
	};

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

	const backend = {
		platformName: "test",
		systemTheme: writable(null),
		invoke,
	} as unknown as IBackend;

	const mutationState = { current: { isLoading: false } };
	const rulesService = {
		get createWorkspaceRule() {
			return [async () => workspaceRules[0], mutationState];
		},
		get updateWorkspaceRule() {
			return [async () => workspaceRules[0], mutationState];
		},
		get deleteWorkspaceRule() {
			return [async () => undefined, mutationState];
		},
		workspaceRules(receivedProjectId: string) {
			queueMicrotask(() => {
				workspaceRulesCallCount += 1;
				lastWorkspaceProjectId = receivedProjectId;
			});
			return fulfilledRules(workspaceRules);
		},
		principalRules(receivedProjectId: string, receivedPrincipalId: string) {
			queueMicrotask(() => {
				principalRulesCallCount += 1;
				lastPrincipalProjectId = receivedProjectId;
				lastPrincipalId = receivedPrincipalId;
			});
			return fulfilledRules(rulesByPrincipal[receivedPrincipalId] ?? []);
		},
	} as unknown as RulesService;

	const stackService = {
		stacks() {
			return fulfilledArray([]);
		},
		stackById() {
			return fulfilledValue(null);
		},
	} as unknown as StackService;

	const uiState = {
		global: {
			scrollbarVisibilityState: {
				current: "scroll",
			},
		},
	} as unknown as UiState;

	provide(BACKEND, backend);
	provide(RULES_SERVICE, rulesService);
	provide(STACK_SERVICE, stackService);
	provide(UI_STATE, uiState);

	const groups: GroupListEntry[] = [
		{
			name: "eng",
			authorities: ["contents:write"],
			members: ["settings-agent"],
		},
	];

	function cloneBranch(branch: BranchGateEntry): BranchGateEntry {
		return {
			...branch,
			require_approval_from_group: [...branch.require_approval_from_group],
		};
	}

	function cloneBranches(branches: BranchGateEntry[]): BranchGateEntry[] {
		return branches.map(cloneBranch);
	}

	function branchOutcome(): BranchGatesOutcome {
		return {
			branches: cloneBranches(currentBranches),
			caveat: targetRef,
		};
	}

	function updateBranch(branch: string, protection: BranchProtectionInput) {
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
		currentPendingCount = 1;
	}

	async function invoke<T>(command: string, args: unknown): Promise<T> {
		backendCalls = [...backendCalls, { command, args }];

		if (command === "branch_gates_read") {
			return branchOutcome() as T;
		}

		if (command === "group_list") {
			return { groups: branchGroups } as T;
		}

		if (command === "branch_gates_update") {
			const update = args as {
				branch: string;
				protection: BranchProtectionInput;
			};
			updateBranch(update.branch, update.protection);
			return branchOutcome() as T;
		}

		throw new Error(`Unexpected backend command: ${command}`);
	}

	function createRule(id: string, path: string): WorkspaceRule {
		return {
			id,
			createdAt: "2026-06-21T00:00:00.000Z",
			enabled: true,
			trigger: "fileSytemChange",
			filters: [{ type: "pathMatchesRegex", subject: path }],
			action: {
				type: "explicit",
				subject: {
					type: "assign",
					subject: { target: { type: "leftmost" } },
				},
			},
		} as WorkspaceRule;
	}

	function fulfilledRules(rules: WorkspaceRule[]) {
		const ids = rules.map((rule) => rule.id);
		return fulfilledValue({
			ids,
			entities: Object.fromEntries(rules.map((rule) => [rule.id, rule])),
		});
	}

	function fulfilledArray<T>(data: T[]) {
		return fulfilledValue(data);
	}

	function fulfilledValue<T>(data: T) {
		return {
			result: {
				data,
				status: QueryStatus.fulfilled,
				isSuccess: true,
			},
		};
	}
</script>

<GovernanceSettings
	{projectId}
	{targetRef}
	{service}
	initialGroups={groups}
	initialPendingGroups={currentPendingGroups}
/>

<output data-testid="governance-user-role">{role}</output>
<output data-testid="governance-commit-count">{commitCount}</output>
<output data-testid="governance-read-pending-count">{readPendingCount}</output>
<output data-testid="governance-commit-message">{lastCommitMessage}</output>
<output data-testid="governance-branch-gates-calls">{JSON.stringify(backendCalls)}</output>
<output data-testid="governance-branch-gates-branches">{JSON.stringify(currentBranches)}</output>
<output data-testid="governance-workspace-rules-call-count">{workspaceRulesCallCount}</output>
<output data-testid="governance-principal-rules-call-count">{principalRulesCallCount}</output>
<output data-testid="governance-last-workspace-project-id">{lastWorkspaceProjectId}</output>
<output data-testid="governance-last-principal-project-id">{lastPrincipalProjectId}</output>
<output data-testid="governance-last-principal-id">{lastPrincipalId}</output>
