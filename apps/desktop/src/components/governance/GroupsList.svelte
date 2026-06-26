<script lang="ts" module>
	import type { GroupListEntry, GroupListOutcome, GroupWriteOutcome } from "@gitbutler/but-sdk";

	export type GroupsListService = {
		listGroups: (projectId: string, targetRef: string) => Promise<GroupListOutcome>;
		groupCreate: (
			projectId: string,
			targetRef: string,
			group: string,
			authorities: string[],
		) => Promise<GroupWriteOutcome>;
		groupGrant: (
			projectId: string,
			targetRef: string,
			group: string,
			authorities: string[],
		) => Promise<GroupWriteOutcome>;
		groupRevoke: (
			projectId: string,
			targetRef: string,
			group: string,
			authorities: string[],
		) => Promise<GroupWriteOutcome>;
		groupAddMember: (
			projectId: string,
			targetRef: string,
			group: string,
			member: string,
		) => Promise<GroupWriteOutcome>;
		groupRemoveMember: (
			projectId: string,
			targetRef: string,
			group: string,
			member: string,
		) => Promise<GroupWriteOutcome>;
		groupDelete: (
			projectId: string,
			targetRef: string,
			group: string,
		) => Promise<GroupWriteOutcome>;
	};
</script>

<script lang="ts">
	import GovernanceConfigHint from "$components/governance/GovernanceConfigHint.svelte";
	import ExpandableSection from "$components/shared/ExpandableSection.svelte";
	import { BACKEND } from "$lib/backend";
	import { CAPABILITY_CATALOG } from "$lib/governance";
	import { injectOptional } from "@gitbutler/core/context";
	import {
		Badge,
		Button,
		EmptyStatePlaceholder,
		InfoMessage,
		Modal,
		TagInput,
		Textbox,
		Toggle,
	} from "@gitbutler/ui";
	import { untrack } from "svelte";
	import type { Tag } from "@gitbutler/ui";

	type PendingMemberRemoval = {
		group: string;
		member: string;
	};

	type Props = {
		projectId: string;
		targetRef: string;
		isReadOnly?: boolean;
		groups?: GroupListEntry[];
		gateReferencedGroups?: string[];
		pendingGroups?: string[];
		service?: GroupsListService;
		onRefresh?: () => void;
		onGroupPending?: (groupName: string) => void;
	};

	const {
		projectId,
		targetRef,
		isReadOnly = false,
		groups: providedGroups,
		gateReferencedGroups = [],
		pendingGroups = [],
		service: providedService,
		onRefresh,
		onGroupPending,
	}: Props = $props();

	const backend = injectOptional(BACKEND, undefined);
	const service = untrack(() => providedService ?? createBackendService());
	const initialGroups = untrack(() => providedGroups ?? []);

	let groups = $state<GroupListEntry[]>(cloneGroups(initialGroups));
	let expandedGroups = $state<string[]>([]);
	let isLoading = $state(untrack(() => providedGroups === undefined));
	let loadError = $state<string | undefined>();
	let writeError = $state<string | undefined>();
	let createGroupName = $state("");
	let deleteTarget = $state<GroupListEntry | undefined>();
	let pendingMemberRemoval = $state<PendingMemberRemoval | undefined>();
	let memberInputVersion = $state(0);
	let deleteModal = $state<Modal>();
	let lastMemberModal = $state<Modal>();

	const sortedGroups = $derived(
		[...groups].sort((left, right) => left.name.localeCompare(right.name)),
	);
	const hasGroups = $derived(sortedGroups.length > 0);
	const pendingGroupSet = $derived(new Set(pendingGroups));

	$effect(() => {
		if (providedGroups !== undefined) {
			groups = cloneGroups(providedGroups);
			isLoading = false;
			return;
		}

		if (!projectId) {
			groups = [];
			isLoading = false;
			return;
		}

		untrack(() => {
			void loadGroups();
		});
	});

	function createBackendService(): GroupsListService {
		return {
			listGroups(projectId, targetRef) {
				if (!backend) throw new Error("governance.backend_unavailable");
				// Read-only renderer path: the committed group roster is plaintext at the target
				// ref, so this read is ungated — unlike the governed `group_list` CLI command.
				return backend.invoke<GroupListOutcome>("governance_groups_list", {
					projectId,
					targetRef,
				});
			},
			groupCreate(projectId, targetRef, group, authorities) {
				if (!backend) throw new Error("governance.backend_unavailable");
				return backend.invoke<GroupWriteOutcome>("group_create", {
					projectId,
					targetRef,
					group,
					authorities,
				});
			},
			groupGrant(projectId, targetRef, group, authorities) {
				if (!backend) throw new Error("governance.backend_unavailable");
				return backend.invoke<GroupWriteOutcome>("group_grant", {
					projectId,
					targetRef,
					group,
					authorities,
				});
			},
			groupRevoke(projectId, targetRef, group, authorities) {
				if (!backend) throw new Error("governance.backend_unavailable");
				return backend.invoke<GroupWriteOutcome>("group_revoke", {
					projectId,
					targetRef,
					group,
					authorities,
				});
			},
			groupAddMember(projectId, targetRef, group, member) {
				if (!backend) throw new Error("governance.backend_unavailable");
				return backend.invoke<GroupWriteOutcome>("group_add_member", {
					projectId,
					targetRef,
					group,
					member,
				});
			},
			groupRemoveMember(projectId, targetRef, group, member) {
				if (!backend) throw new Error("governance.backend_unavailable");
				return backend.invoke<GroupWriteOutcome>("group_remove_member", {
					projectId,
					targetRef,
					group,
					member,
				});
			},
			groupDelete(projectId, targetRef, group) {
				if (!backend) throw new Error("governance.backend_unavailable");
				return backend.invoke<GroupWriteOutcome>("group_delete", {
					projectId,
					targetRef,
					group,
				});
			},
		};
	}

	function cloneGroups(entries: GroupListEntry[]): GroupListEntry[] {
		return entries.map((group) => ({
			name: group.name,
			authorities: uniqueSorted(group.authorities),
			members: uniqueSorted(group.members),
		}));
	}

	function uniqueSorted(values: string[]): string[] {
		return [...new Set(values)].sort((left, right) => left.localeCompare(right));
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
		return "governance.group_write_failed";
	}

	function memberTags(group: GroupListEntry): Tag[] {
		return group.members.map((member) => ({ id: member, label: member }));
	}

	async function loadGroups() {
		isLoading = true;
		loadError = undefined;

		try {
			const outcome = await service.listGroups(projectId, targetRef);
			groups = cloneGroups(outcome.groups);
		} catch (error) {
			loadError = errorCode(error);
			groups = [];
		} finally {
			isLoading = false;
		}
	}

	function updateGroup(groupName: string, updater: (group: GroupListEntry) => GroupListEntry) {
		groups = groups.map((group) => (group.name === groupName ? updater(group) : group));
	}

	async function refreshAfterWrite() {
		onRefresh?.();
		if (providedGroups === undefined) {
			await loadGroups();
		}
	}

	function markGroupPending(groupName: string) {
		onGroupPending?.(groupName);
	}

	async function createGroup() {
		const group = createGroupName.trim();
		if (!group || isReadOnly) return;

		writeError = undefined;
		try {
			await service.groupCreate(projectId, targetRef, group, []);
			groups = [...groups, { name: group, authorities: [], members: [] }];
			createGroupName = "";
			markGroupPending(group);
			await refreshAfterWrite();
		} catch (error) {
			writeError = errorCode(error);
		}
	}

	async function setGrant(groupName: string, authority: string, checked: boolean) {
		if (isReadOnly) return;

		writeError = undefined;
		const previousGroups = cloneGroups(groups);

		try {
			if (checked) {
				await service.groupGrant(projectId, targetRef, groupName, [authority]);
				updateGroup(groupName, (group) => ({
					...group,
					authorities: uniqueSorted([...group.authorities, authority]),
				}));
			} else {
				await service.groupRevoke(projectId, targetRef, groupName, [authority]);
				updateGroup(groupName, (group) => ({
					...group,
					authorities: group.authorities.filter((grant) => grant !== authority),
				}));
			}
			markGroupPending(groupName);
			await refreshAfterWrite();
		} catch (error) {
			groups = previousGroups;
			writeError = errorCode(error);
		}
	}

	async function addMember(groupName: string, tag: Tag) {
		const member = tag.label.trim();
		if (!member || isReadOnly) return;

		writeError = undefined;
		const previousGroups = cloneGroups(groups);

		try {
			await service.groupAddMember(projectId, targetRef, groupName, member);
			updateGroup(groupName, (group) => ({
				...group,
				members: uniqueSorted([...group.members, member]),
			}));
			markGroupPending(groupName);
			await refreshAfterWrite();
		} catch (error) {
			groups = previousGroups;
			writeError = errorCode(error);
		}
	}

	function removeMember(group: GroupListEntry, member: string) {
		if (isReadOnly) return;

		const isLastMember = group.members.length === 1 && group.members[0] === member;
		if (isLastMember && gateReferencedGroups.includes(group.name)) {
			pendingMemberRemoval = { group: group.name, member };
			memberInputVersion += 1;
			lastMemberModal?.show();
			groups = cloneGroups(groups);
			return;
		}

		void removeMemberNow(group.name, member);
	}

	async function removeMemberNow(groupName: string, member: string) {
		if (isReadOnly) return;

		writeError = undefined;
		const previousGroups = cloneGroups(groups);

		try {
			await service.groupRemoveMember(projectId, targetRef, groupName, member);
			updateGroup(groupName, (group) => ({
				...group,
				members: group.members.filter((value) => value !== member),
			}));
			markGroupPending(groupName);
			await refreshAfterWrite();
		} catch (error) {
			groups = previousGroups;
			writeError = errorCode(error);
		}
	}

	function openDeleteModal(group: GroupListEntry) {
		if (isReadOnly) return;

		deleteTarget = group;
		deleteModal?.show();
	}

	async function deleteGroup(close: () => void) {
		if (!deleteTarget || isReadOnly) return;

		writeError = undefined;
		const groupName = deleteTarget.name;
		const previousGroups = cloneGroups(groups);

		try {
			await service.groupDelete(projectId, targetRef, groupName);
			groups = groups.filter((group) => group.name !== groupName);
			deleteTarget = undefined;
			await close();
			await refreshAfterWrite();
		} catch (error) {
			groups = previousGroups;
			writeError = errorCode(error);
		}
	}

	async function confirmLastMemberRemoval(close: () => void) {
		if (!pendingMemberRemoval) return;

		const removal = pendingMemberRemoval;
		pendingMemberRemoval = undefined;
		await close();
		await removeMemberNow(removal.group, removal.member);
	}
</script>

<section class="groups-list" data-testid="groups-list" aria-busy={isLoading}>
	{#if loadError}
		<InfoMessage style="danger" outlined>
			{#snippet title()}Could not load groups{/snippet}
			{#snippet content()}{loadError}{/snippet}
		</InfoMessage>
	{/if}

	{#if writeError}
		<InfoMessage testId="groups-list-write-error" style="danger" outlined>
			{#snippet title()}{writeError}{/snippet}
			{#snippet content()}The requested governance group write was not applied.{/snippet}
		</InfoMessage>
	{/if}

	<GovernanceConfigHint file="agents" {targetRef} cli="but group" />

	<div class="groups-list__create">
		<Textbox
			testId="groups-list-create-name"
			value={createGroupName}
			placeholder="Group name"
			disabled={isReadOnly}
			oninput={(value) => (createGroupName = value)}
			onkeydown={(event) => {
				if (event.key === "Enter") {
					event.preventDefault();
					void createGroup();
				}
			}}
		/>
		<Button kind="outline" disabled={isReadOnly || !createGroupName.trim()} onclick={createGroup}>
			+ Create group
		</Button>
	</div>

	{#if !isLoading && !hasGroups}
		<div data-testid="groups-list-empty">
			<EmptyStatePlaceholder gap={12} topBottomPadding={24}>
				{#snippet title()}No groups yet{/snippet}
				{#snippet caption()}Create a group to share inherited permissions across principals.{/snippet}
				{#snippet actions()}
					<Button kind="outline" disabled={isReadOnly} onclick={createGroup}>+ Create group</Button>
				{/snippet}
			</EmptyStatePlaceholder>
		</div>
	{:else}
		<div class="groups-list__rows">
			{#each sortedGroups as group (group.name)}
				{@const groupSlug = slug(group.name)}
				<div class="groups-list__row" data-testid={`groups-list-row-${groupSlug}`}>
					<ExpandableSection
						label={group.name}
						expanded={expandedGroups.includes(group.name)}
						onToggle={(expanded) => {
							expandedGroups = expanded
								? uniqueSorted([...expandedGroups, group.name])
								: expandedGroups.filter((name) => name !== group.name);
						}}
					>
						{#snippet summary()}
							{#if pendingGroupSet.has(group.name)}
								<Badge
									testId={`groups-list-pending-${groupSlug}`}
									style="warning"
									kind="soft"
									size="tag"
								>
									Pending
								</Badge>
							{/if}
							<Badge style="gray" kind="soft" size="tag">{group.members.length} members</Badge>
							<Badge style="gray" kind="soft" size="tag">
								{group.authorities.length} grants
							</Badge>
						{/snippet}

						{#snippet content()}
							<div class="groups-list__content">
								<div class="groups-list__section">
									<span class="groups-list__label">Permissions</span>
									<p class="groups-list__hint-text">
										Every member of this group inherits the permissions enabled below.
									</p>
									<div class="groups-list__permission-table">
										{#each CAPABILITY_CATALOG as row (row.authority)}
											{@const rowSlug = slug(row.authority)}
											{@const checked = group.authorities.includes(row.authority)}
											<div class="groups-list__permission-row">
												<div class="groups-list__permission-copy">
													<span class="groups-list__permission-name">
														{row.label}
														<code>{row.authority}</code>
													</span>
													<span class="groups-list__permission-desc">{row.description}</span>
												</div>
												{#key `${group.name}-${row.authority}-${checked}-${isReadOnly}`}
													<Toggle
														testId={`groups-list-toggle-${groupSlug}-${rowSlug}`}
														{checked}
														disabled={isReadOnly}
														onchange={(nextChecked) =>
															setGrant(group.name, row.authority, nextChecked)}
													/>
												{/key}
											</div>
										{/each}
									</div>
								</div>

								<div class="groups-list__section" data-testid={`groups-list-members-${groupSlug}`}>
									{#key `${group.name}-${group.members.join("|")}-${isReadOnly}-${memberInputVersion}`}
										<TagInput
											testId={`groups-list-members-input-${groupSlug}`}
											label="Members"
											tags={memberTags(group)}
											readonly={isReadOnly}
											placeholder="Add member"
											onAddTag={(tag) => addMember(group.name, tag)}
											onRemoveTag={(member) => removeMember(group, member)}
											wide
										/>
									{/key}
								</div>

								<footer class="groups-list__actions">
									<Button
										testId={`groups-list-delete-${groupSlug}`}
										style="danger"
										kind="outline"
										disabled={isReadOnly}
										onclick={() => openDeleteModal(group)}
									>
										Delete group
									</Button>
								</footer>
							</div>
						{/snippet}
					</ExpandableSection>
				</div>
			{/each}
		</div>
	{/if}
</section>

<Modal
	bind:this={deleteModal}
	testId="groups-list-delete-modal"
	title="Delete group"
	type="danger"
	width="small"
	preventCloseOnClickOutside
>
	{#snippet children(_item, close)}
		<p>
			Remove group {deleteTarget?.name}? {deleteTarget?.members.length ?? 0} principals will lose inherited
			permissions.
		</p>
	{/snippet}
	{#snippet controls(close)}
		<Button kind="outline" onclick={() => close()}>Cancel</Button>
		<Button style="danger" onclick={() => deleteGroup(close)}>Delete group</Button>
	{/snippet}
</Modal>

<Modal
	bind:this={lastMemberModal}
	testId="groups-list-last-member-modal"
	title="Remove last member"
	type="warning"
	width="small"
	preventCloseOnClickOutside
>
	{#snippet children(_item, close)}
		<InfoMessage style="warning" outlined>
			{#snippet title()}This group is referenced by a branch gate{/snippet}
			{#snippet content()}
				Removing {pendingMemberRemoval?.member} will leave {pendingMemberRemoval?.group} empty while it
				is referenced by a branch gate.
			{/snippet}
		</InfoMessage>
	{/snippet}
	{#snippet controls(close)}
		<Button kind="outline" onclick={() => close()}>Cancel</Button>
		<Button style="warning" onclick={() => confirmLastMemberRemoval(close)}>Remove member</Button>
	{/snippet}
</Modal>

<style>
	.groups-list,
	.groups-list__rows,
	.groups-list__content,
	.groups-list__section {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.groups-list__create {
		display: grid;
		grid-template-columns: minmax(180px, 280px) max-content;
		align-items: end;
		gap: 6px;
	}

	.groups-list__row {
		padding: 8px;
		border: 1px solid var(--border-2);
		border-radius: var(--radius-s);
		background: var(--bg-1);
	}

	.groups-list__label {
		font-weight: 600;
	}

	.groups-list__permission-table {
		display: flex;
		flex-direction: column;
		overflow: hidden;
		border: 1px solid var(--border-2);
		border-radius: var(--radius-s);
	}

	.groups-list__permission-row {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto;
		align-items: center;
		padding: 8px;
		gap: 8px;
		border-bottom: 1px solid var(--border-2);
	}

	.groups-list__permission-row:last-child {
		border-bottom: 0;
	}

	.groups-list__permission-copy {
		display: flex;
		flex-direction: column;
		min-width: 0;
		gap: 2px;
	}

	.groups-list__permission-name {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 6px;
		font-weight: 600;
	}

	.groups-list__permission-name code {
		padding: 0 2px;
		border-radius: var(--radius-s);
		background: var(--bg-2);
		color: var(--text-2);
		font-size: 11px;
		font-weight: 400;
	}

	.groups-list__permission-desc {
		color: var(--text-2);
		font-size: 12px;
	}

	.groups-list__hint-text {
		margin: 0;
		color: var(--text-2);
		font-size: 12px;
	}

	.groups-list__actions {
		display: flex;
		justify-content: flex-end;
	}

	@media (max-width: 640px) {
		.groups-list__create {
			grid-template-columns: 1fr;
		}
	}
</style>
