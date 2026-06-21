<script lang="ts">
	import RulesList from "$components/rules/RulesList.svelte";
	import { RULES_SERVICE } from "$lib/rules/rulesService.svelte";
	import { UI_STATE, type UiState } from "$lib/state/uiState.svelte";
	import { STACK_SERVICE } from "$lib/stacks/stackService.svelte";
	import { setEphemeralStorageItem } from "@gitbutler/shared/persisted";
	import { provide } from "@gitbutler/core/context";
	import { QueryStatus } from "@reduxjs/toolkit/query";
	import { untrack } from "svelte";
	import type RulesService from "$lib/rules/rulesService.svelte";
	import type { WorkspaceRule } from "$lib/rules/rule";
	import type { StackService } from "$lib/stacks/stackService.svelte";

	type Props = {
		principalId?: string;
	};

	const { principalId }: Props = $props();

	const projectId = "ct-project";
	const drawerPersistId = `rules-drawer-${projectId}`;
	if (typeof window !== "undefined") {
		setEphemeralStorageItem(drawerPersistId, false, 1440);
	}

	let selectedPrincipalId = $state<string | undefined>(untrack(() => principalId));
	let workspaceRulesCallCount = $state(0);
	let principalRulesCallCount = $state(0);
	let lastWorkspaceProjectId = $state("");
	let lastPrincipalProjectId = $state("");
	let lastPrincipalId = $state("");

	const workspaceRules = [
		createRule("rule-A1", "src/a1.ts"),
		createRule("rule-A2", "src/a2.ts"),
		createRule("rule-B1", "src/b1.ts"),
		createRule("rule-B2", "src/b2.ts"),
	];

	const rulesByPrincipal: Record<string, WorkspaceRule[]> = {
		"agent:codex-staging": [workspaceRules[0]!, workspaceRules[1]!],
		"agent:cursor-bot": [workspaceRules[2]!],
		"agent:empty-bot": [],
	};

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

	provide(RULES_SERVICE, rulesService);
	provide(STACK_SERVICE, stackService);
	provide(UI_STATE, uiState);

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

<button
	type="button"
	data-testid="show-cursor-bot"
	onclick={() => (selectedPrincipalId = "agent:cursor-bot")}
>
	Show Cursor Bot
</button>

<div data-testid="rules-list-harness">
	<RulesList {projectId} principalId={selectedPrincipalId} />
</div>

<output data-testid="workspace-rules-call-count">{workspaceRulesCallCount}</output>
<output data-testid="principal-rules-call-count">{principalRulesCallCount}</output>
<output data-testid="last-workspace-project-id">{lastWorkspaceProjectId}</output>
<output data-testid="last-principal-project-id">{lastPrincipalProjectId}</output>
<output data-testid="last-principal-id">{lastPrincipalId}</output>
