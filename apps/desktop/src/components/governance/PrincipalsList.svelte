<script lang="ts" module>
	import type {
		PrincipalEditorService,
		PrincipalInheritedGrant,
	} from "$components/governance/PrincipalEditor.svelte";
	import type { GovernancePrincipalsList, GovernanceTarget } from "$lib/governance";

	/**
	 * Additive `kind` descriptor (LPR-005 / LPR-014). When `'agent'`, the row renders
	 * a decorative agent Badge. Absent or `'human'` renders no badge (the conservative
	 * default-human posture). `kind` is enforcement-neutral: it gates nothing and is
	 * never read by any gate predicate.
	 */
	export type PrincipalsListEntry = {
		principalId: string;
		ownGrants: string[];
		inheritedGrants?: PrincipalInheritedGrant[];
		groupMemberships?: string[];
		pending?: boolean;
		isCurrentUser?: boolean;
		kind?: "agent" | "human" | undefined;
	};

	export type PrincipalsListService = {
		readPrincipals: (target: GovernanceTarget) => Promise<GovernancePrincipalsList>;
	};
</script>

<script lang="ts">
	import GovernanceConfigHint from "$components/governance/GovernanceConfigHint.svelte";
	import PrincipalEditor from "$components/governance/PrincipalEditor.svelte";
	import ExpandableSection from "$components/shared/ExpandableSection.svelte";
	import { BACKEND } from "$lib/backend";
	import {
		CAPABILITY_AUTHORITIES,
		CAPABILITY_CATALOG,
		CAPABILITY_CATEGORIES,
		createGovernanceRendererContract,
	} from "$lib/governance";
	import { injectOptional } from "@gitbutler/core/context";
	import { Badge, Button, EmptyStatePlaceholder, Icon, InfoMessage, Tooltip } from "@gitbutler/ui";
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
		/**
		 * LPR-014: invoked after a successful governance write (e.g. kind write) for a
		 * principal, so the parent (GovernanceSettings) can increment the pending-banner
		 * count. Mirrors the `onGroupPending` callback on GroupsList.
		 */
		onPrincipalPending?: (principalId: string) => void;
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
		onPrincipalPending,
	}: Props = $props();

	const knownAuthorities = new Set(CAPABILITY_AUTHORITIES);
	const totalColumns = CAPABILITY_CATALOG.length + 1;

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

		// Wait for the backend (governance_status_read) to resolve the real target
		// ref before firing readPrincipals. Firing with a stale/guessed ref makes
		// the backend reject the request ("target ref mismatch"); keep the loading
		// state and re-fire once access.targetRef arrives.
		if (!targetRef) {
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

	function slug(value: string): string {
		return value.replace(/[^a-z0-9]+/gi, "-");
	}

	function grantState(
		principal: PrincipalsListEntry,
		authority: string,
	): "own" | "inherited" | "none" {
		if (principal.ownGrants.includes(authority)) return "own";
		if ((principal.inheritedGrants ?? []).some((grant) => grant.authority === authority)) {
			return "inherited";
		}
		return "none";
	}

	function inheritedSource(principal: PrincipalsListEntry, authority: string): string | undefined {
		return (principal.inheritedGrants ?? []).find((grant) => grant.authority === authority)
			?.sourceLabel;
	}

	// Surface any granted authority that isn't in the standard catalog rather than silently
	// dropping it — hiding a real grant on a security surface would be a lie.
	function customGrants(principal: PrincipalsListEntry): string[] {
		const all = [
			...principal.ownGrants,
			...(principal.inheritedGrants ?? []).map((grant) => grant.authority),
		];
		return [...new Set(all.filter((authority) => !knownAuthorities.has(authority)))].sort();
	}

	async function loadPrincipals() {
		if (!targetRef) return;

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
		// LPR-014: mark the just-saved principal as pending in local state so the
		// existing ○ Badge (lines 178-187) renders; notify the parent so the governance
		// pending-banner count can increment via the existing pendingStore pattern.
		if (selectedPrincipalId) {
			principals = principals.map((principal) =>
				principal.principalId === selectedPrincipalId ? { ...principal, pending: true } : principal,
			);
			onPrincipalPending?.(selectedPrincipalId);
		}
		onRefresh?.();
		// Only auto-reload when PrincipalsList owns the data (no providedPrincipals).
		// When the parent provides principals (test scope or explicit data), it owns
		// the lifecycle and we must not overwrite local state with a fetch.
		if (providedPrincipals === undefined) {
			void loadPrincipals();
		}
	}
</script>

<section class="principals" data-testid="principals-list">
	{#if loadError}
		<InfoMessage style="danger" outlined>
			{#snippet title()}Could not load principals{/snippet}
			{#snippet content()}{loadError}{/snippet}
		</InfoMessage>
	{/if}

	<GovernanceConfigHint file="agents" {targetRef} cli="but perm" />

	{#if !isLoading && principals.length === 0}
		<div data-testid="principals-list-empty">
			<EmptyStatePlaceholder gap={12} topBottomPadding={24}>
				{#snippet title()}No principals configured{/snippet}
				{#snippet caption()}
					Add agents to <code>.gitbutler/agents.toml</code> and commit to see them here.
				{/snippet}
				{#snippet actions()}
					<Button kind="outline" disabled={isReadOnly} onclick={onAddFirst}>+ Add first</Button>
				{/snippet}
			</EmptyStatePlaceholder>
		</div>
	{:else}
		<div class="principals__legend" data-testid="principals-legend">
			<span class="legend-item"><span class="cell-mark cell-mark--own">✓</span> Direct grant</span>
			<span class="legend-item">
				<span class="cell-mark cell-mark--inherited">◐</span> Inherited from group
			</span>
			<span class="legend-item legend-item--muted">Dimmed columns aren't enforced yet</span>
		</div>

		<div class="principals__scroll" data-testid="principals-scroll" aria-busy={isLoading}>
			<table class="matrix" data-testid="principals-matrix">
				<thead>
					<tr>
						<th class="matrix__corner" rowspan="2" scope="col">Agent</th>
						{#each CAPABILITY_CATEGORIES as category (category.id)}
							<th class="matrix__group" colspan={category.capabilities.length} scope="colgroup">
								{category.label}
							</th>
						{/each}
					</tr>
					<tr>
						{#each CAPABILITY_CATALOG as capability (capability.authority)}
							<th
								class="matrix__cap"
								class:matrix__cap--soon={!capability.enforced}
								scope="col"
							>
								<Tooltip
									text={`${capability.authority} — ${capability.description}${capability.enforced ? "" : " (defined, not yet enforced)"}`}
								>
									<span class="matrix__cap-label">{capability.short}</span>
								</Tooltip>
							</th>
						{/each}
					</tr>
				</thead>
				<tbody>
					{#each principals as principal (principal.principalId)}
						{@const groupMemberships = principal.groupMemberships ?? []}
						{@const extras = customGrants(principal)}
						<tr
							class="matrix__row"
							data-testid={`principals-list-row-${slug(principal.principalId)}`}
						>
							<th class="matrix__agent" scope="row">
								<button
									type="button"
									class="matrix__agent-button"
									data-testid="principals-list-row"
									aria-expanded={selectedPrincipalId === principal.principalId}
									onclick={() => selectPrincipal(principal.principalId)}
								>
									<span class="matrix__agent-name">
										{#if principal.pending}
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
										{#if principal.isCurrentUser}
											<Badge style="pop" kind="soft" size="tag">You</Badge>
										{/if}
									</span>
									{#if groupMemberships.length > 0}
										<span class="matrix__agent-groups">
											{#each groupMemberships as group (group)}
												<span class="group-chip" title={`Member of group: ${group}`}>
													<Icon name="folder" />{group}
												</span>
											{/each}
										</span>
									{/if}
									{#if extras.length > 0}
										<span class="matrix__agent-groups">
											{#each extras as extra (extra)}
												<Badge style="warning" kind="soft" size="tag">{extra}</Badge>
											{/each}
										</span>
									{/if}
								</button>
							</th>

							{#each CAPABILITY_CATALOG as capability (capability.authority)}
								{@const state = grantState(principal, capability.authority)}
								{@const source = inheritedSource(principal, capability.authority)}
								<td
									class="matrix__cell"
									class:matrix__cell--soon={!capability.enforced}
									data-testid={`principals-cell-${slug(principal.principalId)}-${slug(capability.authority)}`}
									data-grant={state}
								>
									{#if state === "own"}
										<Tooltip text={`${capability.label} — direct grant`}>
											<span class="cell-mark cell-mark--own">✓</span>
										</Tooltip>
									{:else if state === "inherited"}
										<Tooltip text={`${capability.label} — inherited from ${source}`}>
											<span class="cell-mark cell-mark--inherited">◐</span>
										</Tooltip>
									{:else}
										<span class="cell-mark cell-mark--none" aria-hidden="true">·</span>
									{/if}
								</td>
							{/each}
						</tr>

						{#if selectedPrincipal?.principalId === principal.principalId}
							<tr class="matrix__editor-row">
								<td colspan={totalColumns}>
									<PrincipalEditor
										{projectId}
										{targetRef}
										principalId={principal.principalId}
										ownGrants={principal.ownGrants}
										inheritedGrants={principal.inheritedGrants ?? []}
										{groupMemberships}
										{availableGroups}
										isCurrentUser={principal.isCurrentUser}
										{isReadOnly}
										service={editorService}
										onCancel={() => (selectedPrincipalId = undefined)}
										onSaved={refreshAfterSave}
									/>
								</td>
							</tr>
						{/if}
					{/each}
				</tbody>
			</table>
		</div>

		<ExpandableSection label="What these permissions mean" icon="info">
			{#snippet content()}
				<dl class="glossary" data-testid="principals-glossary">
					{#each CAPABILITY_CATALOG as capability (capability.authority)}
						<div class="glossary__row">
							<dt>
								<code>{capability.authority}</code>
								<span class="glossary__name">{capability.label}</span>
								{#if !capability.enforced}
									<Badge style="gray" kind="soft" size="tag">not yet enforced</Badge>
								{/if}
							</dt>
							<dd>{capability.description}</dd>
						</div>
					{/each}
				</dl>
			{/snippet}
		</ExpandableSection>
	{/if}
</section>

<style>
	.principals {
		display: flex;
		flex-direction: column;
		/* width:100% + min-width:0 keep this constrained to the parent's width even when an
		   ancestor flex column uses align-items other than stretch — otherwise the wide matrix
		   would size the column to its own content and never trigger horizontal scroll. */
		width: 100%;
		min-width: 0;
		gap: 10px;
	}

	.principals__legend {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 12px;
		color: var(--text-2);
		font-size: 12px;
	}

	.legend-item {
		display: inline-flex;
		align-items: center;
		gap: 4px;
	}

	.legend-item--muted {
		color: var(--text-3);
	}

	.principals__scroll {
		width: 100%;
		min-width: 0;
		max-width: 100%;
		overflow-x: auto;
		border: 1px solid var(--border-2);
		border-radius: var(--radius-m);
	}

	.matrix {
		width: 100%;
		border-collapse: collapse;
		font-size: 12px;
	}

	.matrix th,
	.matrix td {
		border-bottom: 1px solid var(--border-3);
		border-left: 1px solid var(--border-3);
	}

	.matrix__group,
	.matrix__cap {
		padding: 4px 6px;
		color: var(--text-2);
		font-weight: 600;
		text-align: center;
		white-space: nowrap;
		background: var(--bg-2);
	}

	.matrix__group {
		border-bottom: 1px solid var(--border-2);
		font-size: 11px;
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.matrix__cap {
		font-weight: 500;
	}

	.matrix__cap-label {
		cursor: help;
	}

	.matrix__cap--soon,
	.matrix__cell--soon {
		background: var(--bg-1);
		opacity: 0.65;
	}

	.matrix__corner {
		position: sticky;
		left: 0;
		z-index: 1;
		padding: 4px 8px;
		border-left: 0;
		color: var(--text-2);
		font-weight: 600;
		text-align: left;
		background: var(--bg-2);
	}

	.matrix__agent {
		position: sticky;
		left: 0;
		z-index: 1;
		border-left: 0;
		background: var(--bg-1);
	}

	.matrix__row:hover .matrix__agent {
		background: var(--bg-2);
	}

	.matrix__agent-button {
		display: flex;
		flex-direction: column;
		width: 100%;
		min-width: 160px;
		padding: 6px 8px;
		gap: 4px;
		background: transparent;
		color: var(--text-1);
		font: inherit;
		text-align: left;
		cursor: pointer;
	}

	.matrix__agent-name {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.matrix__agent-groups {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
	}

	.group-chip {
		display: inline-flex;
		align-items: center;
		padding: 0 4px;
		gap: 2px;
		border-radius: var(--radius-s);
		background: var(--bg-2);
		color: var(--text-2);
		font-size: 11px;
	}

	.matrix__cell {
		padding: 4px;
		text-align: center;
	}

	.matrix__row:hover .matrix__cell {
		background: var(--bg-2);
	}

	.cell-mark {
		display: inline-block;
		font-weight: 700;
	}

	.cell-mark--own {
		color: var(--fill-pop-bg);
	}

	.cell-mark--inherited {
		color: var(--text-2);
	}

	.cell-mark--none {
		color: var(--text-3);
	}

	.matrix__editor-row td {
		padding: 8px;
		background: var(--bg-2);
	}

	.glossary {
		display: flex;
		flex-direction: column;
		margin: 0;
		gap: 8px;
	}

	.glossary__row {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.glossary dt {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 6px;
	}

	.glossary__name {
		font-weight: 600;
	}

	.glossary dd {
		margin: 0;
		color: var(--text-2);
		font-size: 12px;
	}

	.glossary code {
		padding: 0 2px;
		border-radius: var(--radius-s);
		background: var(--bg-2);
		font-size: 11px;
	}
</style>
