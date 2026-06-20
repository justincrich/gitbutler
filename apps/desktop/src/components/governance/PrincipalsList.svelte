<script lang="ts" module>
	import type {
		PrincipalEditorService,
		PrincipalInheritedGrant,
	} from "$components/governance/PrincipalEditor.svelte";
	import type { GovernancePrincipalsList, GovernanceTarget } from "$lib/governance";

	export type PrincipalsListEntry = {
		principalId: string;
		ownGrants: string[];
		inheritedGrants?: PrincipalInheritedGrant[];
		groupMemberships?: string[];
		pending?: boolean;
		isCurrentUser?: boolean;
	};

	export type PrincipalsListService = {
		readPrincipals: (target: GovernanceTarget) => Promise<GovernancePrincipalsList>;
	};
</script>

<script lang="ts">
	import PrincipalEditor from "$components/governance/PrincipalEditor.svelte";
	import { BACKEND } from "$lib/backend";
	import { createGovernanceRendererContract } from "$lib/governance";
	import { injectOptional } from "@gitbutler/core/context";
	import { Badge, Button, EmptyStatePlaceholder, InfoMessage } from "@gitbutler/ui";
	import { untrack } from "svelte";
	import type { GovernanceRendererContract } from "$lib/governance";

	type Props = {
		projectId: string;
		targetRef: string;
		isReadOnly?: boolean;
		principals?: PrincipalsListEntry[];
		service?: PrincipalsListService;
		editorService?: PrincipalEditorService;
		availableGroups?: string[];
		onRefresh?: () => void;
		onAddFirst?: () => void;
	};

	const {
		projectId,
		targetRef,
		isReadOnly = false,
		principals: providedPrincipals,
		service: providedService,
		editorService,
		availableGroups = [],
		onRefresh,
		onAddFirst,
	}: Props = $props();

	const backend = injectOptional(BACKEND, undefined);
	const service = untrack(() => providedService ?? createPrincipalsListService());
	let principals = $state<PrincipalsListEntry[]>(untrack(() => providedPrincipals ?? []));
	let selectedPrincipalId = $state<string | undefined>();
	let isLoading = $state(untrack(() => providedPrincipals === undefined));
	let loadError = $state<string | undefined>();
	const selectedPrincipal = $derived(
		principals.find((principal) => principal.principalId === selectedPrincipalId),
	);

	$effect(() => {
		if (providedPrincipals !== undefined) {
			principals = providedPrincipals;
			isLoading = false;
			return;
		}

		if (!projectId) {
			principals = [];
			isLoading = false;
			return;
		}

		untrack(() => {
			void loadPrincipals();
		});
	});

	function createPrincipalsListService(): PrincipalsListService {
		const governanceService = backend ? createGovernanceRendererContract(backend) : undefined;
		return governanceService
			? createPrincipalsListServiceFromContract(governanceService)
			: unavailableService();
	}

	function createPrincipalsListServiceFromContract(
		governanceService: Pick<GovernanceRendererContract, "readPrincipals">,
	): PrincipalsListService {
		return governanceService;
	}

	function unavailableService(): PrincipalsListService {
		return {
			async readPrincipals() {
				throw new Error("governance.backend_unavailable");
			},
		};
	}

	function uniqueSorted(values: string[]): string[] {
		return [...new Set(values)].sort((left, right) => left.localeCompare(right));
	}

	function slug(value: string): string {
		return value.replace(/[^a-z0-9]+/gi, "-");
	}

	function effectiveGrants(principal: PrincipalsListEntry): string[] {
		return uniqueSorted([
			...principal.ownGrants,
			...(principal.inheritedGrants ?? []).map((grant) => grant.authority),
		]);
	}

	async function loadPrincipals() {
		isLoading = true;
		loadError = undefined;

		try {
			const outcome = await service.readPrincipals({ projectId, targetRef });
			principals = outcome.principals;
		} catch (error) {
			loadError = error instanceof Error ? error.message : "governance.principals_load_failed";
			principals = [];
		} finally {
			isLoading = false;
		}
	}

	function selectPrincipal(principalId: string) {
		selectedPrincipalId = selectedPrincipalId === principalId ? undefined : principalId;
	}

	function refreshAfterSave() {
		onRefresh?.();
		void loadPrincipals();
	}
</script>

<section class="principals-list" data-testid="principals-list">
	{#if loadError}
		<InfoMessage style="danger" outlined>
			{#snippet title()}Could not load principals{/snippet}
			{#snippet content()}{loadError}{/snippet}
		</InfoMessage>
	{/if}

	{#if !isLoading && principals.length === 0}
		<div data-testid="principals-list-empty">
			<EmptyStatePlaceholder gap={12} topBottomPadding={24}>
				{#snippet title()}No principals configured{/snippet}
				{#snippet actions()}
					<Button kind="outline" disabled={isReadOnly} onclick={onAddFirst}>+ Add first</Button>
				{/snippet}
			</EmptyStatePlaceholder>
		</div>
	{:else}
		<div class="principals-list__table" aria-busy={isLoading}>
			{#each principals as principal (principal.principalId)}
				{@const inheritedGrants = principal.inheritedGrants ?? []}
				{@const groupMemberships = principal.groupMemberships ?? []}
				<div
					class="principals-list__row-wrap"
					data-testid={`principals-list-row-${slug(principal.principalId)}`}
				>
					<button
						type="button"
						class="principals-list__row"
						data-testid="principals-list-row"
						aria-expanded={selectedPrincipalId === principal.principalId}
						onclick={() => selectPrincipal(principal.principalId)}
					>
						<span class="principals-list__principal">
							{#if principal.pending && principal.ownGrants.length > 0}
								<Badge
									testId={`principals-list-pending-${slug(principal.principalId)}`}
									style="warning"
									kind="soft"
									size="icon"
								>
									○
								</Badge>
							{/if}
							<strong>{principal.principalId}</strong>
						</span>

						<span class="principals-list__grants">
							{#each effectiveGrants(principal) as authority (authority)}
								<span class="principals-list__grant">
									<span>{authority}</span>
									{#if principal.ownGrants.includes(authority)}
										<small>own grant</small>
									{:else}
										<small>
											{inheritedGrants.find((grant) => grant.authority === authority)?.sourceLabel}
										</small>
									{/if}
								</span>
							{/each}
						</span>

						<span class="principals-list__groups">
							{#each groupMemberships as group (group)}
								<Badge style="gray" kind="soft" size="tag">group: {group}</Badge>
							{/each}
						</span>
					</button>

					{#if selectedPrincipal?.principalId === principal.principalId}
						<div class="principals-list__editor">
							<PrincipalEditor
								{projectId}
								{targetRef}
								principalId={principal.principalId}
								ownGrants={principal.ownGrants}
								{inheritedGrants}
								{groupMemberships}
								{availableGroups}
								isCurrentUser={principal.isCurrentUser}
								{isReadOnly}
								service={editorService}
								onCancel={() => (selectedPrincipalId = undefined)}
								onSaved={refreshAfterSave}
							/>
						</div>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</section>

<style>
	.principals-list,
	.principals-list__table,
	.principals-list__row-wrap {
		display: flex;
		flex-direction: column;
		gap: var(--clr-space-8);
	}

	.principals-list__row {
		display: grid;
		grid-template-columns: minmax(140px, 0.7fr) minmax(220px, 1.6fr) minmax(120px, 0.8fr);
		align-items: start;
		width: 100%;
		padding: var(--clr-space-8);
		gap: var(--clr-space-8);
		border: 1px solid var(--clr-border-2);
		border-radius: var(--radius-s);
		background: var(--clr-bg-1);
		color: var(--clr-text-1);
		font: inherit;
		text-align: left;
		cursor: pointer;
	}

	.principals-list__row:hover {
		background: var(--clr-bg-2);
	}

	.principals-list__principal,
	.principals-list__grants,
	.principals-list__groups,
	.principals-list__grant {
		display: flex;
		min-width: 0;
		gap: var(--clr-space-4);
	}

	.principals-list__principal {
		align-items: center;
	}

	.principals-list__grants,
	.principals-list__groups {
		flex-wrap: wrap;
	}

	.principals-list__grant {
		flex-direction: column;
		padding: var(--clr-space-4) var(--clr-space-6);
		border-radius: var(--radius-s);
		background: var(--clr-bg-2);
	}

	.principals-list__grant small {
		color: var(--clr-text-2);
	}

	.principals-list__editor {
		padding-left: var(--clr-space-12);
	}
</style>
