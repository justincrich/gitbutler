<script lang="ts" module>
	import type { GroupListOutcome } from "@gitbutler/but-sdk";

	export type BranchGateEntry = {
		name: string;
		protected: boolean;
		min_approvals: number;
		require_distinct_from_author: boolean;
		require_approval_from_group: string[];
		pending: boolean;
	};

	export type BranchGatesOutcome = {
		branches: BranchGateEntry[];
		caveat?: string;
	};

	export type BranchProtectionInput = {
		protected: boolean;
		min_approvals: number | null;
		require_distinct_from_author: boolean | null;
		require_approval_from_group: string[] | null;
	};

	type BranchGatesListService = {
		branchGatesRead: (projectId: string, targetRef: string) => Promise<BranchGatesOutcome>;
		branchGatesUpdate: (
			projectId: string,
			targetRef: string,
			branch: string,
			protection: BranchProtectionInput,
		) => Promise<BranchGatesOutcome | undefined>;
		listGroups: (projectId: string, targetRef: string) => Promise<GroupListOutcome>;
		branchGatesUpdateError?: string;
	};
</script>

<script lang="ts">
	import ExpandableSection from "$components/shared/ExpandableSection.svelte";
	import { BACKEND } from "$lib/backend";
	import { injectOptional } from "@gitbutler/core/context";
	import {
		Badge,
		Button,
		EmptyStatePlaceholder,
		InfoMessage,
		Modal,
		Textbox,
		Toggle,
	} from "@gitbutler/ui";
	import { untrack } from "svelte";

	type DefinedGroup = {
		name: string;
	};

	type Props = {
		projectId: string;
		targetRef: string;
		isReadOnly?: boolean;
		pendingBranches?: string[];
		onRefresh?: () => void;
		onBranchPending?: (branchName: string) => void;
	};

	type UnprotectTarget = {
		branch: string;
		protection: BranchProtectionInput;
	};

	const {
		projectId,
		targetRef,
		isReadOnly = false,
		pendingBranches = [],
		onRefresh,
		onBranchPending,
	}: Props = $props();

	const backend = injectOptional(BACKEND, undefined);
	const service = untrack(() => createBackendService());

	let branches = $state<BranchGateEntry[]>([]);
	let definedGroups = $state<DefinedGroup[]>([]);
	let expandedBranches = $state<string[]>([]);
	let isLoading = $state(true);
	let loadError = $state<string | undefined>();
	let writeError = $state<string | undefined>();
	let addBranchName = $state("");
	let showAddForm = $state(false);
	let unprotectTarget = $state<UnprotectTarget | undefined>();
	let unprotectModal = $state<Modal>();
	let inputVersion = $state(0);

	const sortedBranches = $derived(
		[...branches].sort((left, right) => left.name.localeCompare(right.name)),
	);
	const hasBranches = $derived(sortedBranches.length > 0);
	const pendingBranchSet = $derived(new Set(pendingBranches));
	const groupOptions = $derived(uniqueInOrder(definedGroups.map((group) => group.name)));

	$effect(() => {
		if (!projectId) {
			branches = [];
			definedGroups = [];
			isLoading = false;
			return;
		}

		untrack(() => {
			void loadBranchGates();
		});
	});

	function createBackendService(): BranchGatesListService {
		return {
			branchGatesRead(projectId, targetRef) {
				if (!backend) throw new Error("governance.backend_unavailable");
				return backend.invoke<BranchGatesOutcome>("branch_gates_read", { projectId, targetRef });
			},
			branchGatesUpdate(projectId, targetRef, branch, protection) {
				if (!backend) throw new Error("governance.backend_unavailable");
				return backend.invoke<BranchGatesOutcome>("branch_gates_update", {
					projectId,
					targetRef,
					branch,
					protection,
				});
			},
			listGroups(projectId) {
				if (!backend) throw new Error("governance.backend_unavailable");
				return backend.invoke<GroupListOutcome>("group_list", { projectId });
			},
		};
	}

	function cloneBranches(entries: BranchGateEntry[]): BranchGateEntry[] {
		return entries.map((branch) => ({
			...branch,
			require_approval_from_group: uniqueSorted(branch.require_approval_from_group),
		}));
	}

	function cloneDefinedGroups(entries: DefinedGroup[]): DefinedGroup[] {
		return entries.map((group) => ({ name: group.name }));
	}

	function uniqueSorted(values: string[]): string[] {
		return [...new Set(values)].sort((left, right) => left.localeCompare(right));
	}

	function uniqueInOrder(values: string[]): string[] {
		return [...new Set(values)];
	}

	function slug(value: string): string {
		return value.replace(/[^a-z0-9]+/gi, "-");
	}

	function errorCode(error: unknown): string {
		if (error instanceof Error && error.message) return error.message;
		if (typeof error === "object" && error !== null && "code" in error) {
			const code = (error as { code: unknown }).code;
			if (typeof code === "string") return code;
		}
		return "governance.branch_gate_write_failed";
	}

	function returnedErrorCode(outcome: unknown): string | undefined {
		if (typeof outcome !== "object" || outcome === null || !("code" in outcome)) return undefined;

		const code = (outcome as { code: unknown }).code;
		if (typeof code !== "string") return undefined;

		if ("message" in outcome) {
			const message = (outcome as { message: unknown }).message;
			if (typeof message === "string") return `${code} ${message}`;
		}

		return code;
	}

	function returnedCaveatError(outcome: BranchGatesOutcome | undefined): string | undefined {
		if (!outcome) return undefined;
		if (outcome.caveat?.startsWith("perm.denied")) return outcome.caveat;
		return undefined;
	}

	function protectionFor(branch: BranchGateEntry): BranchProtectionInput {
		return {
			protected: branch.protected,
			min_approvals: branch.min_approvals,
			require_distinct_from_author: branch.require_distinct_from_author,
			require_approval_from_group: uniqueSorted(branch.require_approval_from_group),
		};
	}

	async function loadBranchGates() {
		isLoading = true;
		loadError = undefined;

		try {
			const [branchOutcome, groupOutcome] = await Promise.all([
				service.branchGatesRead(projectId, targetRef),
				service.listGroups(projectId, targetRef),
			]);
			branches = cloneBranches(branchOutcome.branches);
			definedGroups = cloneDefinedGroups(groupOutcome.groups);
		} catch (error) {
			loadError = errorCode(error);
			branches = [];
			definedGroups = [];
		} finally {
			isLoading = false;
		}
	}

	async function refreshAfterWrite() {
		onRefresh?.();
		await loadBranchGates();
	}

	function markBranchPending(branchName: string) {
		onBranchPending?.(branchName);
		branches = branches.map((branch) =>
			branch.name === branchName ? { ...branch, pending: true } : branch,
		);
	}

	function updateBranch(branchName: string, updater: (branch: BranchGateEntry) => BranchGateEntry) {
		const nextBranches = branches.map((branch) =>
			branch.name === branchName ? updater(branch) : branch,
		);
		branches = nextBranches;
	}

	async function writeBranch(
		branchName: string,
		protection: BranchProtectionInput,
		optimistic?: (branch: BranchGateEntry) => BranchGateEntry,
	) {
		if (isReadOnly) return;

		writeError = undefined;
		const previousBranches = cloneBranches(branches);

		try {
			const outcome = await service.branchGatesUpdate(projectId, targetRef, branchName, protection);
			const outcomeError =
				returnedErrorCode(outcome) ??
				returnedCaveatError(outcome) ??
				service.branchGatesUpdateError;
			if (outcomeError) throw new Error(outcomeError);

			if (optimistic) updateBranch(branchName, optimistic);
			markBranchPending(branchName);
			await refreshAfterWrite();
		} catch (error) {
			branches = previousBranches;
			inputVersion += 1;
			writeError = errorCode(error);
		}
	}

	function setProtected(branch: BranchGateEntry, checked: boolean) {
		if (isReadOnly) return;

		const protection = { ...protectionFor(branch), protected: checked };
		if (!checked) {
			unprotectTarget = { branch: branch.name, protection };
			unprotectModal?.show();
			branches = cloneBranches(branches);
			return;
		}

		void writeBranch(branch.name, protection, (currentBranch) => ({
			...currentBranch,
			protected: checked,
		}));
	}

	async function confirmUnprotect(close: () => void) {
		if (!unprotectTarget || isReadOnly) return;

		const target = unprotectTarget;
		await writeBranch(target.branch, target.protection, (branch) => ({
			...branch,
			protected: false,
		}));
		unprotectTarget = undefined;
		await close();
	}

	async function setMinApprovals(branch: BranchGateEntry, value: string) {
		if (isReadOnly) return;

		const minApprovals = Number.parseInt(value, 10);
		if (
			!Number.isInteger(minApprovals) ||
			minApprovals < 0 ||
			minApprovals === branch.min_approvals
		) {
			branches = cloneBranches(branches);
			return;
		}

		await writeBranch(
			branch.name,
			{ ...protectionFor(branch), min_approvals: minApprovals },
			(currentBranch) => ({ ...currentBranch, min_approvals: minApprovals }),
		);
	}

	function setRequireDistinct(branch: BranchGateEntry, checked: boolean) {
		if (isReadOnly) return;

		void writeBranch(
			branch.name,
			{ ...protectionFor(branch), require_distinct_from_author: checked },
			(currentBranch) => ({ ...currentBranch, require_distinct_from_author: checked }),
		);
	}

	function setRequiredGroup(branch: BranchGateEntry, group: string, checked: boolean) {
		if (isReadOnly || !groupOptions.includes(group)) return;

		const groups = checked
			? uniqueSorted([...branch.require_approval_from_group, group])
			: branch.require_approval_from_group.filter((value) => value !== group);

		void writeBranch(
			branch.name,
			{ ...protectionFor(branch), require_approval_from_group: groups },
			(currentBranch) => ({ ...currentBranch, require_approval_from_group: groups }),
		);
	}

	async function createBranchGate() {
		const branchName = addBranchName.trim();
		if (!branchName || isReadOnly) return;

		const protection: BranchProtectionInput = {
			protected: true,
			min_approvals: 1,
			require_distinct_from_author: true,
			require_approval_from_group: [],
		};

		await writeBranch(branchName, protection);

		if (!writeError) {
			if (!branches.some((branch) => branch.name === branchName)) {
				branches = [
					...branches,
					{
						name: branchName,
						protected: true,
						min_approvals: 1,
						require_distinct_from_author: true,
						require_approval_from_group: [],
						pending: true,
					},
				];
			}
			addBranchName = "";
			showAddForm = false;
		}
	}
</script>

<section class="branch-gates-list" data-testid="branch-gates-list" aria-busy={isLoading}>
	{#if loadError}
		<InfoMessage style="danger" outlined>
			{#snippet title()}Could not load branch gates{/snippet}
			{#snippet content()}{loadError}{/snippet}
		</InfoMessage>
	{/if}

	{#if writeError}
		<InfoMessage testId="branch-gates-list-write-error" style="danger" outlined>
			{#snippet title()}{writeError}{/snippet}
			{#snippet content()}Permission denied. The requested branch gate write was not applied.{/snippet}
		</InfoMessage>
	{/if}

	<div class="branch-gates-list__create">
		<Button kind="outline" disabled={isReadOnly} onclick={() => (showAddForm = true)}>+ Add</Button>
		{#if showAddForm}
			<div class="branch-gates-list__add-form" data-testid="branch-gates-list-add-form">
				<Textbox
					testId="branch-gates-list-add-pattern"
					value={addBranchName}
					placeholder="Branch pattern"
					disabled={isReadOnly}
					oninput={(value) => (addBranchName = value)}
					onkeydown={(event) => {
						if (event.key === "Enter") {
							event.preventDefault();
							void createBranchGate();
						}
					}}
				/>
				<Button
					kind="outline"
					disabled={isReadOnly || !addBranchName.trim()}
					onclick={createBranchGate}
				>
					Add gate
				</Button>
			</div>
		{/if}
	</div>

	{#if !isLoading && !hasBranches}
		<div data-testid="branch-gates-list-empty">
			<EmptyStatePlaceholder gap={12} topBottomPadding={24}>
				{#snippet title()}No branch gates yet{/snippet}
				{#snippet caption()}Add a branch pattern to require review before merge.{/snippet}
				{#snippet actions()}
					<Button kind="outline" disabled={isReadOnly} onclick={() => (showAddForm = true)}
						>+ Add</Button
					>
				{/snippet}
			</EmptyStatePlaceholder>
		</div>
	{:else}
		<div class="branch-gates-list__rows">
			{#each sortedBranches as branch (branch.name)}
				{@const branchSlug = slug(branch.name)}
				{@const isPending = branch.pending || pendingBranchSet.has(branch.name)}
				<div class="branch-gates-list__row" data-testid={`branch-gates-list-row-${branchSlug}`}>
					<ExpandableSection
						label={branch.name}
						expanded={expandedBranches.includes(branch.name)}
						onToggle={(expanded) => {
							expandedBranches = expanded
								? uniqueSorted([...expandedBranches, branch.name])
								: expandedBranches.filter((name) => name !== branch.name);
						}}
					>
						{#snippet summary()}
							{#if isPending}
								<Badge
									testId={`branch-gates-list-pending-${branchSlug}`}
									style="warning"
									kind="soft"
									size="tag"
								>
									Pending
								</Badge>
							{/if}
							<Badge style={branch.protected ? "safe" : "gray"} kind="soft" size="tag">
								{branch.protected ? "Protected" : "Unprotected"}
							</Badge>
							<Badge style="gray" kind="soft" size="tag">
								{branch.min_approvals} approvals
							</Badge>
						{/snippet}

						{#snippet content()}
							<div class="branch-gates-list__content">
								<div class="branch-gates-list__field">
									<div>
										<strong>Protected</strong>
										<span>Require governance checks before merge.</span>
									</div>
									{#key `${branch.name}-protected-${branch.protected}-${isReadOnly}`}
										<Toggle
											testId={`branch-gates-list-protected-${branchSlug}`}
											checked={branch.protected}
											disabled={isReadOnly}
											onclick={(event) => {
												if (branch.protected) {
													event.preventDefault();
													setProtected(branch, false);
												}
											}}
											onchange={(checked) => setProtected(branch, checked)}
										/>
									{/key}
								</div>

								<div class="branch-gates-list__field">
									<div>
										<strong>Minimum approvals</strong>
										<span>Review approvals required before merge.</span>
									</div>
									{#key `${branch.name}-approvals-${branch.min_approvals}-${isReadOnly}-${inputVersion}`}
										<Textbox
											testId={`branch-gates-list-min-approvals-${branchSlug}`}
											type="number"
											value={String(branch.min_approvals)}
											minVal={0}
											disabled={isReadOnly}
											onchange={(value) => setMinApprovals(branch, value)}
										/>
									{/key}
								</div>

								<div class="branch-gates-list__field">
									<div>
										<strong>Distinct from author</strong>
										<span>Approval must come from someone other than the commit author.</span>
									</div>
									{#key `${branch.name}-distinct-${branch.require_distinct_from_author}-${isReadOnly}`}
										<Toggle
											testId={`branch-gates-list-distinct-${branchSlug}`}
											checked={branch.require_distinct_from_author}
											disabled={isReadOnly}
											onchange={(checked) => setRequireDistinct(branch, checked)}
										/>
									{/key}
								</div>

								<div
									class="branch-gates-list__section"
									data-testid={`branch-gates-list-groups-${branchSlug}`}
								>
									<span class="branch-gates-list__label">Required groups</span>
									<div class="branch-gates-list__chips">
										{#each branch.require_approval_from_group as group (group)}
											<Badge style="gray" kind="soft" size="tag">{group}</Badge>
										{/each}
									</div>
									<div
										class="branch-gates-list__group-options"
										data-testid={`branch-gates-list-group-options-${branchSlug}`}
									>
										{#each groupOptions as group (group)}
											{@const groupSlug = slug(group)}
											<label class="branch-gates-list__group-option">
												<input
													data-testid={`branch-gates-list-group-option-${branchSlug}`}
													type="checkbox"
													checked={branch.require_approval_from_group.includes(group)}
													disabled={isReadOnly}
													onchange={(event) =>
														setRequiredGroup(branch, group, event.currentTarget.checked)}
												/>
												<span>{group}</span>
											</label>
										{/each}
									</div>
								</div>
							</div>
						{/snippet}
					</ExpandableSection>
				</div>
			{/each}
		</div>
	{/if}
</section>

<Modal
	bind:this={unprotectModal}
	testId="branch-gates-list-unprotect-modal"
	title="Unprotect branch"
	type="warning"
	width="small"
	preventCloseOnClickOutside
>
	{#snippet children(_item, close)}
		<p>Unprotect branch {unprotectTarget?.branch}? Merges will no longer require review.</p>
	{/snippet}
	{#snippet controls(close)}
		<Button kind="outline" onclick={() => close()}>Cancel</Button>
		<Button style="warning" onclick={() => confirmUnprotect(close)}>Unprotect branch</Button>
	{/snippet}
</Modal>

<style>
	.branch-gates-list,
	.branch-gates-list__rows,
	.branch-gates-list__content,
	.branch-gates-list__section {
		display: flex;
		flex-direction: column;
		gap: var(--clr-space-8);
	}

	.branch-gates-list__create,
	.branch-gates-list__add-form {
		display: flex;
		align-items: end;
		gap: var(--clr-space-6);
	}

	.branch-gates-list__row {
		padding: var(--clr-space-8);
		border: 1px solid var(--clr-border-2);
		border-radius: var(--radius-s);
		background: var(--clr-bg-1);
	}

	.branch-gates-list__field {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto;
		align-items: center;
		padding: var(--clr-space-8);
		gap: var(--clr-space-8);
		border: 1px solid var(--clr-border-2);
		border-radius: var(--radius-s);
	}

	.branch-gates-list__field div,
	.branch-gates-list__section {
		min-width: 0;
	}

	.branch-gates-list__field div {
		display: flex;
		flex-direction: column;
		gap: var(--clr-space-2);
	}

	.branch-gates-list__field span,
	.branch-gates-list__label {
		color: var(--clr-text-2);
		font-size: 12px;
	}

	.branch-gates-list__chips,
	.branch-gates-list__group-options {
		display: flex;
		flex-wrap: wrap;
		gap: var(--clr-space-6);
	}

	.branch-gates-list__group-option {
		display: flex;
		align-items: center;
		gap: var(--clr-space-4);
	}
</style>
