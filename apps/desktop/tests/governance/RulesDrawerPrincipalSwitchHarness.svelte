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
	// IMPORTANT: start the drawer COLLAPSED (true) to match RulesList's
	// defaultCollapsed={true}. This lets the CT test exercise the
	// expand → switch-principal → expand flow the way the capstone spec does.
	const drawerPersistId = `rules-drawer-${projectId}`;
	if (typeof window !== "undefined") {
		setEphemeralStorageItem(drawerPersistId, true, 1440);
	}

	let selectedPrincipalId = $state<string | undefined>(untrack(() => principalId));

	const principalARules = [
		createRule("rule-A1", "src/a1.ts"),
		createRule("rule-A2", "src/a2.ts"),
	];
	const principalBRules = [createRule("rule-B1", "src/b1.ts")];

	const rulesByPrincipal: Record<string, WorkspaceRule[]> = {
		"agent:codex-staging": principalARules,
		"agent:cursor-bot": principalBRules,
	};

	const mutationState = { current: { isLoading: false } };
	const rulesService = {
		get createWorkspaceRule() {
			return [async () => principalARules[0], mutationState];
		},
		get updateWorkspaceRule() {
			return [async () => principalARules[0], mutationState];
		},
		get deleteWorkspaceRule() {
			return [async () => undefined, mutationState];
		},
		workspaceRules() {
			return fulfilledRules([...principalARules, ...principalBRules]);
		},
		principalRules(_receivedProjectId: string, receivedPrincipalId: string) {
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
<button
	type="button"
	data-testid="show-codex-staging"
	onclick={() => (selectedPrincipalId = "agent:codex-staging")}
>
	Show Codex Staging
</button>

<div data-testid="rules-list-harness">
	<RulesList {projectId} principalId={selectedPrincipalId} />
</div>
