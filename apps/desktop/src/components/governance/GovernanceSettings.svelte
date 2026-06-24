<script lang="ts">
	import BranchGatesList from "$components/governance/BranchGatesList.svelte";
	import GroupsList from "$components/governance/GroupsList.svelte";
	import LocalReviewView, {
		createLocalReviewService,
	} from "$components/governance/LocalReviewView.svelte";
	import PrincipalsList from "$components/governance/PrincipalsList.svelte";
	import RulesList from "$components/rules/RulesList.svelte";
	import TabContent from "$components/shared/TabContent.svelte";
	import TabList from "$components/shared/TabList.svelte";
	import TabTrigger from "$components/shared/TabTrigger.svelte";
	import Tabs from "$components/shared/Tabs.svelte";
	import { BACKEND } from "$lib/backend";
	import { URL_SERVICE } from "$lib/backend/url";
	import {
		createGovernanceRendererContract,
		type GovernanceRendererContract,
	} from "$lib/governance";
	import { createGovernancePendingStore } from "$lib/governance/pendingStore.svelte";
	import { injectOptional } from "@gitbutler/core/context";
	import { Button, EmptyStatePlaceholder, InfoMessage } from "@gitbutler/ui";
	import { untrack } from "svelte";
	import type { GroupListEntry } from "@gitbutler/but-sdk";

	// Governance has no in-app builder yet; the not-configured state links here so the
	// user can set up `.gitbutler/permissions.toml` + `.gitbutler/gates.toml` themselves.
	const GOVERNANCE_SETUP_DOCS_URL =
		"https://github.com/justincrich/gitbutler/blob/master/docs/governance-setup.md";

	type Props = {
		projectId?: string;
		targetRef?: string;
		/**
		 * Branch to render local review status for in the "Local Review" tab.
		 * When empty (or when no backend is available), the tab renders the
		 * view's graceful "no local review open" empty state — never crashes.
		 */
		branch?: string;
		service?: GovernanceRendererContract;
		initialGroups?: GroupListEntry[];
		initialPendingGroups?: string[];
		rulesPrincipalId?: string;
	};

	const {
		projectId = "",
		targetRef = "",
		branch = "",
		service: providedService,
		initialGroups,
		initialPendingGroups = [],
		rulesPrincipalId,
	}: Props = $props();

	const backend = injectOptional(BACKEND, undefined);
	const urlService = injectOptional(URL_SERVICE, undefined);
	const service = untrack(
		() => providedService ?? (backend ? createGovernanceRendererContract(backend) : undefined),
	);
	// LocalReviewView takes a dedicated LocalReviewService (reviewStatus +
	// listComments) rather than the governance renderer contract. Build it from
	// the same backend; undefined when no backend is injected (tests/SSR).
	const localReviewService = untrack(() =>
		backend ? createLocalReviewService(backend) : undefined,
	);
	const target = untrack(() => ({ projectId, targetRef }));
	const pendingStore = service ? createGovernancePendingStore(service, target) : undefined;

	const pendingCount = $derived(pendingStore?.pendingCount ?? 0);
	const isReadOnly = $derived(pendingStore?.access.isReadOnly ?? false);
	// "Not configured" is a normal first-run state (no committed governance config on the
	// target branch), NOT an error — render guidance instead of the tabs or a red banner.
	const isNotConfigured = $derived(pendingStore?.access.isNotConfigured ?? false);
	// The backend resolves the real target ref (governance_status_read returns
	// target_ref); the renderer prop may be empty. resolvedTargetRef stays
	// undefined until the backend has resolved it — never fall back to a
	// hardcoded ref (origin/master), which the backend rejects with a
	// target-ref-mismatch error.
	const resolvedTargetRef = $derived(pendingStore?.access.targetRef || targetRef || undefined);
	const commitDisabled = $derived(
		isReadOnly || pendingCount === 0 || Boolean(pendingStore?.isCommitting),
	);
	let pendingGroupNames = $state<string[]>(untrack(() => [...initialPendingGroups]));
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

	async function openSetupGuide() {
		await urlService?.openExternalUrl(GOVERNANCE_SETUP_DOCS_URL);
	}

	function markGroupPending(groupName: string) {
		if (pendingGroupNames.includes(groupName)) return;
		pendingGroupNames = [...pendingGroupNames, groupName].sort((left, right) =>
			left.localeCompare(right),
		);
	}
</script>

<section class="governance-settings" data-testid="governance-settings">
	<h2>Permissions & Governance</h2>

	{#if isNotConfigured}
		<!-- First-run guidance: there's no in-app builder, so tell the user exactly how to
		     stand governance up themselves and link to the full setup guide. -->
		<div data-testid="governance-not-configured">
			<EmptyStatePlaceholder gap={12} topBottomPadding={32} width={440}>
				{#snippet title()}Governance isn't set up yet{/snippet}
				{#snippet caption()}
					Permissions and branch gates are read from
					<code>.gitbutler/permissions.toml</code> and
					<code>.gitbutler/gates.toml</code>, committed to the target branch (<code
						>{resolvedTargetRef}</code
					>). Nothing is committed there yet, so there's nothing to manage here.
					<br /><br />
					There's no in-app setup yet. To enable governance: add those two files (or build them with the
					<code>but</code> CLI), commit them to the target branch, then reopen this page. The setup guide
					walks through the schema and CLI.
				{/snippet}
				{#snippet actions()}
					<Button
						kind="solid"
						style="pop"
						icon="docs"
						onclick={openSetupGuide}
						disabled={!urlService}
						testId="governance-setup-guide-link"
					>
						Open setup guide
					</Button>
				{/snippet}
			</EmptyStatePlaceholder>
		</div>
	{:else}
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
				<TabTrigger value="local-review">Local Review</TabTrigger>
			</TabList>

			<TabContent value="principals">
				<section
					class="governance-panel governance-panel--principals"
					data-testid="governance-principals-panel"
				>
					<h3>Principals</h3>
					<PrincipalsList
						{projectId}
						targetRef={resolvedTargetRef ?? ""}
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
						targetRef={resolvedTargetRef ?? ""}
						{isReadOnly}
						groups={initialGroups}
						pendingGroups={pendingGroupNames}
						onRefresh={refreshGovernance}
						onGroupPending={markGroupPending}
					/>
				</section>
			</TabContent>

			<TabContent value="branch-gates">
				<section
					class="governance-panel governance-panel--branch-gates"
					data-testid="governance-branch-gates-panel"
				>
					<h3>Branch Gates</h3>
					<BranchGatesList
						{projectId}
						targetRef={resolvedTargetRef ?? ""}
						{isReadOnly}
						onRefresh={refreshGovernance}
					/>
				</section>
			</TabContent>

			<TabContent value="rules">
				<section
					class="governance-panel governance-panel--rules"
					data-testid="governance-rules-panel"
				>
					<h3>Rules</h3>
					<button
						type="button"
						class="governance-button"
						disabled={isReadOnly}
						data-testid="governance-rules-control"
					>
						Add rule
					</button>
					{#if rulesPrincipalId}
						<div class="governance-rules-list" data-testid="governance-rules-list">
							<RulesList {projectId} principalId={rulesPrincipalId} />
						</div>
					{:else}
						<div data-testid="governance-rules-no-principal">
							<EmptyStatePlaceholder gap={12} topBottomPadding={24}>
								{#snippet title()}Select a principal to view their rules{/snippet}
							</EmptyStatePlaceholder>
						</div>
					{/if}
				</section>
			</TabContent>

			<TabContent value="local-review">
				<section
					class="governance-panel governance-panel--local-review"
					data-testid="governance-local-review-panel"
				>
					<h3>Local Review</h3>
					<LocalReviewView {projectId} {branch} service={localReviewService} {isReadOnly} />
				</section>
			</TabContent>
		</Tabs>
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
	.governance-panel--branch-gates,
	.governance-panel--rules,
	.governance-panel--local-review {
		flex-direction: column;
	}

	.governance-rules-list {
		width: 100%;
	}
</style>
