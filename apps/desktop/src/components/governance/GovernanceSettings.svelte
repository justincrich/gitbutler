<script lang="ts">
	import BranchGatesList from "$components/governance/BranchGatesList.svelte";
	import GroupsList from "$components/governance/GroupsList.svelte";
	import PrincipalsList from "$components/governance/PrincipalsList.svelte";
	import RulesList from "$components/rules/RulesList.svelte";
	import TabContent from "$components/shared/TabContent.svelte";
	import TabList from "$components/shared/TabList.svelte";
	import Tabs from "$components/shared/Tabs.svelte";
	import TabTrigger from "$components/shared/TabTrigger.svelte";
	import { BACKEND } from "$lib/backend";
	import {
		createGovernanceRendererContract,
		type GovernanceRendererContract,
	} from "$lib/governance";
	import { createGovernancePendingStore } from "$lib/governance/pendingStore.svelte";
	import { injectOptional } from "@gitbutler/core/context";
	import { EmptyStatePlaceholder, InfoMessage } from "@gitbutler/ui";
	import { untrack } from "svelte";
	import type { GroupListEntry } from "@gitbutler/but-sdk";
	import type { PrincipalListEntry } from "$lib/governance";

	type Props = {
		projectId?: string;
		targetRef?: string;
		service?: GovernanceRendererContract;
		initialGroups?: GroupListEntry[];
		initialPendingGroups?: string[];
		rulesPrincipalId?: string;
	};

	const {
		projectId = "",
		targetRef = "refs/remotes/origin/main",
		service: providedService,
		initialGroups,
		initialPendingGroups = [],
		rulesPrincipalId,
	}: Props = $props();

	const backend = injectOptional(BACKEND, undefined);
	const service = untrack(
		() => providedService ?? (backend ? createGovernanceRendererContract(backend) : undefined),
	);
	const target = untrack(() => ({ projectId, targetRef }));
	const pendingStore = service ? createGovernancePendingStore(service, target) : undefined;

	const pendingCount = $derived(pendingStore?.pendingCount ?? 0);
	const isReadOnly = $derived(pendingStore?.access.isReadOnly ?? false);
	const commitDisabled = $derived(
		isReadOnly || pendingCount === 0 || Boolean(pendingStore?.isCommitting),
	);
	let pendingGroupNames = $state<string[]>(untrack(() => [...initialPendingGroups]));
	let pendingBranchNames = $state<string[]>([]);
	let rulesPrincipals = $state<PrincipalListEntry[]>([]);
	let selectedRulesPrincipalId = $state<string | undefined>(untrack(() => rulesPrincipalId));
	let hasLoadedPending = $state(false);

	$effect(() => {
		if (pendingStore && projectId) {
			untrack(() => {
				void refreshGovernance();
			});
		}
	});

	$effect(() => {
		if (hasLoadedPending && pendingCount === 0) {
			pendingGroupNames = [];
			pendingBranchNames = [];
		}
	});

	$effect(() => {
		if (service && projectId) {
			untrack(() => {
				void loadRulesPrincipals();
			});
		}
	});

	async function commitChanges() {
		await pendingStore?.commit();
		hasLoadedPending = true;
	}

	async function refreshGovernance() {
		await pendingStore?.refresh();
		hasLoadedPending = true;
	}

	async function loadRulesPrincipals() {
		const principalList = await service?.readPrincipals(target);
		rulesPrincipals = principalList?.principals ?? [];
	}

	function markGroupPending(groupName: string) {
		if (pendingGroupNames.includes(groupName)) return;
		pendingGroupNames = [...pendingGroupNames, groupName].sort((left, right) =>
			left.localeCompare(right),
		);
	}

	function markBranchPending(branchName: string) {
		if (pendingBranchNames.includes(branchName)) return;
		pendingBranchNames = [...pendingBranchNames, branchName].sort((left, right) =>
			left.localeCompare(right),
		);
	}
</script>

<section class="governance-settings" data-testid="governance-settings">
	<h2>Permissions & Governance</h2>

	{#if isReadOnly}
		<div data-testid="GovernanceReadOnlyMessage">
			<InfoMessage testId="governance-read-only-message" filled outlined={false} style="info">
				{#snippet title()}Read-only governance settings{/snippet}
				{#snippet content()}
					You need administration:write authority to edit governance settings.
				{/snippet}
			</InfoMessage>
		</div>
	{/if}

	{#if pendingCount > 0}
		<div data-testid="GovernancePendingBanner">
			<div class="pending-banner" data-testid="governance-pending-banner">
				<div class="pending-banner__copy">
					<strong>{pendingCount} pending changes</strong>
					<span>Commit governance changes to the configured target branch.</span>
				</div>
				<button
					disabled={commitDisabled}
					onclick={commitChanges}
					type="button"
					class="governance-button governance-button--primary"
					data-testid="governance-commit-button"
				>
					{pendingStore?.isCommitting ? "Committing..." : "Commit changes"}
				</button>
			</div>
		</div>
	{/if}

	<Tabs defaultSelected="principals">
		<TabList ariaLabel="Governance sections">
			<TabTrigger value="principals">Principals</TabTrigger>
			<TabTrigger value="groups">Groups</TabTrigger>
			<TabTrigger value="branch-gates">Branch Gates</TabTrigger>
			<TabTrigger value="rules">Rules</TabTrigger>
		</TabList>

		<TabContent value="principals">
			<section
				class="governance-panel governance-panel--principals"
				data-testid="governance-principals-panel"
			>
				<h3>Principals</h3>
				<PrincipalsList
					{projectId}
					{targetRef}
					{isReadOnly}
					{service}
					onRefresh={refreshGovernance}
				/>
			</section>
		</TabContent>

		<TabContent value="groups">
			<section
				class="governance-panel governance-panel--groups"
				data-testid="governance-groups-panel"
			>
				<h3>Groups</h3>
				<GroupsList
					{projectId}
					{targetRef}
					{isReadOnly}
					groups={initialGroups}
					pendingGroups={pendingGroupNames}
					onRefresh={refreshGovernance}
					onGroupPending={markGroupPending}
				/>
			</section>
		</TabContent>

		<TabContent value="branch-gates">
			<section class="governance-panel" data-testid="governance-branch-gates-panel">
				<h3>Branch Gates</h3>
				<BranchGatesList
					{projectId}
					{targetRef}
					{isReadOnly}
					pendingBranches={pendingBranchNames}
					onRefresh={refreshGovernance}
					onBranchPending={markBranchPending}
				/>
			</section>
		</TabContent>

		<TabContent value="rules">
			<section
				class="governance-panel governance-panel--rules"
				data-testid="governance-rules-panel"
			>
				<h3>Rules</h3>
				<fieldset
					class="governance-rules-selector"
					data-testid="governance-rules-control"
					disabled={isReadOnly}
				>
					<label for="governance-rules-principal-select">Principal</label>
					<select
						id="governance-rules-principal-select"
						data-testid="governance-rules-principal-select"
						bind:value={selectedRulesPrincipalId}
						disabled={isReadOnly}
					>
						<option value={undefined}>Select a principal</option>
						{#each rulesPrincipals as principal (principal.principalId)}
							<option value={principal.principalId}>{principal.principalId}</option>
						{/each}
					</select>
				</fieldset>
				{#if selectedRulesPrincipalId && !isReadOnly}
					<div class="governance-rules-list" data-testid="governance-rules-list">
						<RulesList {projectId} principalId={selectedRulesPrincipalId} />
					</div>
				{:else if !selectedRulesPrincipalId}
					<div data-testid="governance-rules-no-principal">
						<EmptyStatePlaceholder gap={12} topBottomPadding={24}>
							{#snippet title()}Select a principal to view their rules{/snippet}
						</EmptyStatePlaceholder>
					</div>
				{/if}
			</section>
		</TabContent>
	</Tabs>

	{#if pendingStore?.error}
		{@const readFailure = pendingStore.error}
		<InfoMessage testId="governance-read-failure" style="danger" outlined>
			{#snippet title()}{readFailure.code}{/snippet}
			{#snippet content()}
				{readFailure.message}
				{#if readFailure.remediationHint}
					{readFailure.remediationHint}
				{/if}
				<button
					class="governance-button governance-button--retry"
					type="button"
					onclick={refreshGovernance}
				>
					Retry
				</button>
			{/snippet}
		</InfoMessage>
	{/if}
</section>

<style>
	.governance-settings {
		display: flex;
		flex-direction: column;
		gap: var(--clr-space-8);
	}

	.pending-banner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--clr-space-8);
		gap: var(--clr-space-8);
		border: 1px solid var(--clr-border-2);
		border-radius: var(--radius-m);
		background: var(--clr-bg-2);
	}

	.pending-banner__copy {
		display: flex;
		flex-direction: column;
		gap: var(--clr-space-4);
	}

	.governance-button {
		padding: var(--clr-space-4) var(--clr-space-8);
		border: 1px solid var(--clr-border-2);
		border-radius: var(--radius-s);
		background: var(--clr-bg-1);
		color: var(--clr-text-1);
		font: inherit;
		cursor: pointer;
	}

	.governance-button--primary {
		border-color: var(--clr-theme-pop-element);
		background: var(--clr-theme-pop-element);
		color: var(--clr-white);
	}

	.governance-button--retry {
		margin-left: var(--clr-space-6);
	}

	.governance-button:disabled {
		cursor: not-allowed;
		opacity: 0.5;
	}

	.governance-panel {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		min-height: 120px;
		padding: var(--clr-space-8) 0;
		gap: var(--clr-space-8);
	}

	.governance-panel--principals,
	.governance-panel--groups,
	.governance-panel--rules {
		flex-direction: column;
	}

	.governance-rules-list {
		width: 100%;
	}

	.governance-rules-selector {
		display: flex;
		flex-direction: column;
		width: 100%;
		padding: 0;
		gap: var(--clr-space-4);
		border: 0;
	}

	.governance-rules-selector select {
		max-width: 360px;
		padding: var(--clr-space-4) var(--clr-space-6);
		border: 1px solid var(--clr-border-2);
		border-radius: var(--radius-s);
		background: var(--clr-bg-1);
		color: var(--clr-text-1);
		font: inherit;
	}
</style>
