<script lang="ts">
	import RulesList from "$components/rules/RulesList.svelte";
	import { RULES_SERVICE } from "$lib/rules/rulesService.svelte";
	import { UI_STATE, type UiState } from "$lib/state/uiState.svelte";
	import { STACK_SERVICE } from "$lib/stacks/stackService.svelte";
	import { setEphemeralStorageItem } from "@gitbutler/shared/persisted";
	import { provide } from "@gitbutler/core/context";
	import { QueryStatus } from "@reduxjs/toolkit/query";
	import type RulesService from "$lib/rules/rulesService.svelte";
	import type { WorkspaceRule } from "$lib/rules/rule";
	import type { StackService } from "$lib/stacks/stackService.svelte";

	const projectId = "ct-project";
	const drawerPersistId = `rules-drawer-${projectId}`;
	if (typeof window !== "undefined") {
		setEphemeralStorageItem(drawerPersistId, false, 1440);
	}

	// Rule carries a backend-originated claudeCodeSessionId filter alongside a
	// user-visible path filter. The UI must render both pills without crashing.
	const sessionRule: WorkspaceRule = {
		id: "rule-session-a",
		createdAt: "2026-06-21T00:00:00.000Z",
		enabled: true,
		trigger: "fileSytemChange",
		filters: [
			{ type: "claudeCodeSessionId", subject: "agent:codex-staging" },
			{ type: "pathMatchesRegex", subject: "src/session-scoped.ts" },
		],
		action: {
			type: "explicit",
			subject: {
				type: "assign",
				subject: { target: { type: "leftmost" } },
			},
		},
	} as WorkspaceRule;

	const workspaceRules: WorkspaceRule[] = [sessionRule];

	const mutationState = { current: { isLoading: false } };
	const rulesService = {
		get createWorkspaceRule() {
			return [async () => sessionRule, mutationState];
		},
		get updateWorkspaceRule() {
			return [async () => sessionRule, mutationState];
		},
		get deleteWorkspaceRule() {
			return [async () => undefined, mutationState];
		},
		workspaceRules() {
			return fulfilledRules(workspaceRules);
		},
		principalRules() {
			return fulfilledRules(workspaceRules);
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

<div data-testid="rules-list-harness">
	<RulesList {projectId} />
</div>
