<script lang="ts" module>
	import type { GrantOutcome, GroupWriteOutcome, PermWriteOutcome } from "@gitbutler/but-sdk";

	export type PrincipalInheritedGrant = {
		authority: string;
		sourceLabel: string;
	};

	export type PrincipalEditorWriteFailure = {
		code: string;
		message?: string;
	};

	export type PrincipalEditorWriteResult<T> = T | PrincipalEditorWriteFailure;

	export type PrincipalEditorService = {
		deniedCode?: string;
		permGrant: (
			projectId: string,
			targetRef: string,
			principal: string,
			authorities: string[],
		) => Promise<PrincipalEditorWriteResult<GrantOutcome>>;
		permRevoke: (
			projectId: string,
			targetRef: string,
			principal: string,
			authorities: string[],
		) => Promise<PrincipalEditorWriteResult<PermWriteOutcome>>;
		groupAddMember: (
			projectId: string,
			targetRef: string,
			group: string,
			member: string,
		) => Promise<PrincipalEditorWriteResult<GroupWriteOutcome>>;
		groupRemoveMember: (
			projectId: string,
			targetRef: string,
			group: string,
			member: string,
		) => Promise<PrincipalEditorWriteResult<GroupWriteOutcome>>;
	};
</script>

<script lang="ts">
	import { BACKEND } from "$lib/backend";
	import { injectOptional } from "@gitbutler/core/context";
	import { Badge, Button, InfoMessage, SegmentControl, TagInput, Toggle } from "@gitbutler/ui";
	import { untrack } from "svelte";
	import type { Tag } from "@gitbutler/ui";

	type Props = {
		projectId: string;
		targetRef: string;
		principalId: string;
		ownGrants?: string[];
		inheritedGrants?: PrincipalInheritedGrant[];
		groupMemberships?: string[];
		availableGroups?: string[];
		isCurrentUser?: boolean;
		isReadOnly?: boolean;
		service?: PrincipalEditorService;
		onCancel?: () => void;
		onSaved?: () => void;
	};

	type PermissionRow = {
		authority: string;
		label: string;
	};

	const permissionRows: PermissionRow[] = [
		{ authority: "contents:read", label: "Read contents" },
		{ authority: "contents:write", label: "Write contents" },
		{ authority: "reviews:write", label: "Write reviews" },
		{ authority: "administration:write", label: "Administration" },
	];

	const presetAuthorities: Record<string, string[]> = {
		read: ["contents:read"],
		triage: ["contents:read", "reviews:write"],
		write: ["contents:read", "contents:write", "reviews:write"],
		maintain: ["contents:read", "contents:write", "reviews:write"],
		admin: ["contents:read", "contents:write", "reviews:write", "administration:write"],
	};

	const {
		projectId,
		targetRef,
		principalId,
		ownGrants = [],
		inheritedGrants = [],
		groupMemberships = [],
		availableGroups = [],
		isCurrentUser = false,
		isReadOnly = false,
		service: providedService,
		onCancel,
		onSaved,
	}: Props = $props();

	const backend = injectOptional(BACKEND, undefined);
	const service = untrack(() => providedService ?? createBackendService());
	const initialOwnGrants = untrack(() => uniqueSorted(ownGrants));
	const initialGroups = untrack(() => uniqueSorted(groupMemberships));
	const inheritedAuthorityMap = $derived(
		new Map(inheritedGrants.map((grant) => [grant.authority, grant.sourceLabel])),
	);

	let committedOwnGrants = $state([...initialOwnGrants]);
	let stagedOwnGrants = $state([...initialOwnGrants]);
	let committedGroups = $state([...initialGroups]);
	let stagedGroups = $state([...initialGroups]);
	let selectedPreset = $state(untrack(() => resolvePreset(initialOwnGrants)));
	let saveError = $state<string | undefined>();
	let isSaving = $state(false);
	const hasStagedChanges = $derived(
		!sameMembers(committedOwnGrants, stagedOwnGrants) ||
			!sameMembers(committedGroups, stagedGroups),
	);
	const groupTags = $derived(stagedGroups.map((group) => ({ id: group, label: group })));

	function uniqueSorted(values: string[]): string[] {
		return [...new Set(values)].sort((left, right) => left.localeCompare(right));
	}

	function slug(authority: string): string {
		return authority.replace(/[^a-z0-9]+/gi, "-");
	}

	function includes(values: string[], value: string): boolean {
		return values.includes(value);
	}

	function difference(left: string[], right: string[]): string[] {
		return left.filter((value) => !right.includes(value));
	}

	function sameMembers(left: string[], right: string[]): boolean {
		if (left.length !== right.length) return false;
		return left.every((value) => right.includes(value));
	}

	function createBackendService(): PrincipalEditorService {
		return {
			permGrant(projectId, targetRef, principal, authorities) {
				if (!backend) throw new Error("governance.backend_unavailable");
				return backend.invoke<GrantOutcome>("perm_grant", {
					projectId,
					targetRef,
					principal,
					authorities,
				});
			},
			permRevoke(projectId, targetRef, principal, authorities) {
				if (!backend) throw new Error("governance.backend_unavailable");
				return backend.invoke<PermWriteOutcome>("perm_revoke", {
					projectId,
					targetRef,
					principal,
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
		};
	}

	function resolvePreset(authorities: string[]): string {
		const directEditableAuthorities = authorities.filter(
			(authority) => !inheritedAuthorityMap.has(authority),
		);
		const found = Object.entries(presetAuthorities).find(([_preset, preset]) =>
			sameMembers(
				directEditableAuthorities,
				preset.filter((authority) => !inheritedAuthorityMap.has(authority)),
			),
		);

		return found?.[0] ?? "read";
	}

	function setAuthority(authority: string, checked: boolean) {
		saveError = undefined;

		if (checked) {
			stagedOwnGrants = uniqueSorted([...stagedOwnGrants, authority]);
		} else {
			stagedOwnGrants = stagedOwnGrants.filter((grant) => grant !== authority);
		}

		selectedPreset = resolvePreset(stagedOwnGrants);
	}

	function applyPreset(preset: string) {
		saveError = undefined;
		selectedPreset = preset;

		const inheritedAuthorities = new Set(inheritedGrants.map((grant) => grant.authority));
		const presetOwnGrants = (presetAuthorities[preset] ?? []).filter(
			(authority) => !inheritedAuthorities.has(authority),
		);
		const preservedUnknownGrants = stagedOwnGrants.filter(
			(authority) =>
				!permissionRows.some((row) => row.authority === authority) &&
				!inheritedAuthorities.has(authority),
		);

		stagedOwnGrants = uniqueSorted([...presetOwnGrants, ...preservedUnknownGrants]);
	}

	function setGroups(tags: Tag[]) {
		saveError = undefined;
		const allowedGroups = availableGroups.length > 0 ? new Set(availableGroups) : undefined;
		stagedGroups = uniqueSorted(
			tags
				.map((tag) => tag.label.trim())
				.filter((label) => label && (allowedGroups ? allowedGroups.has(label) : true)),
		);
	}

	function resetStaged() {
		stagedOwnGrants = [...committedOwnGrants];
		stagedGroups = [...committedGroups];
		selectedPreset = resolvePreset(stagedOwnGrants);
	}

	function errorCode(error: unknown): string {
		if (error instanceof Error && error.message) {
			const candidate = error as Error & { code?: unknown };
			return typeof candidate.code === "string" ? candidate.code : error.message;
		}

		if (typeof error === "object" && error !== null && "code" in error) {
			const code = (error as { code: unknown }).code;
			if (typeof code === "string") return code;
		}

		return "governance.write_failed";
	}

	function isWriteFailure(result: unknown): result is PrincipalEditorWriteFailure {
		return (
			typeof result === "object" &&
			result !== null &&
			"code" in result &&
			typeof (result as { code: unknown }).code === "string"
		);
	}

	function assertWriteSucceeded(result: unknown) {
		if (!isWriteFailure(result)) return;

		throw Object.assign(new Error(result.message ?? result.code), { code: result.code });
	}

	function assertServiceAllowed() {
		if (!service.deniedCode) return;

		throw Object.assign(new Error(service.deniedCode), { code: service.deniedCode });
	}

	async function save() {
		if (isSaving || !hasStagedChanges || isReadOnly) return;

		isSaving = true;
		saveError = undefined;

		const grantsToAdd = difference(stagedOwnGrants, committedOwnGrants);
		const grantsToRemove = difference(committedOwnGrants, stagedOwnGrants);
		const groupsToAdd = difference(stagedGroups, committedGroups);
		const groupsToRemove = difference(committedGroups, stagedGroups);

		try {
			if (grantsToAdd.length > 0) {
				assertWriteSucceeded(
					await service.permGrant(projectId, targetRef, principalId, grantsToAdd),
				);
				assertServiceAllowed();
			}

			if (grantsToRemove.length > 0) {
				assertWriteSucceeded(
					await service.permRevoke(projectId, targetRef, principalId, grantsToRemove),
				);
				assertServiceAllowed();
			}

			for (const group of groupsToAdd) {
				assertWriteSucceeded(
					await service.groupAddMember(projectId, targetRef, group, principalId),
				);
				assertServiceAllowed();
			}

			for (const group of groupsToRemove) {
				assertWriteSucceeded(
					await service.groupRemoveMember(projectId, targetRef, group, principalId),
				);
				assertServiceAllowed();
			}

			committedOwnGrants = [...stagedOwnGrants];
			committedGroups = [...stagedGroups];
			onSaved?.();
		} catch (error) {
			saveError = errorCode(error);
			resetStaged();
		} finally {
			isSaving = false;
		}
	}
</script>

<section class="principal-editor" data-testid="principal-editor">
	<header class="principal-editor__header">
		<div>
			<h3>{principalId}</h3>
			<p>Direct permissions and group membership</p>
		</div>
		{#if isCurrentUser}
			<Badge style="pop" kind="soft" size="tag">You</Badge>
		{/if}
	</header>

	{#if saveError}
		<InfoMessage testId="principal-editor-denial" style="danger" outlined>
			{#snippet title()}{saveError}{/snippet}
			{#snippet content()}The requested governance write was denied and staged changes were reset.{/snippet}
		</InfoMessage>
	{/if}

	<div class="principal-editor__section">
		<span class="principal-editor__label">Preset</span>
		<SegmentControl selected={selectedPreset} onselect={applyPreset}>
			<SegmentControl.Item id="read" disabled={isReadOnly}>Read</SegmentControl.Item>
			<SegmentControl.Item id="triage" disabled={isReadOnly}>Triage</SegmentControl.Item>
			<SegmentControl.Item id="write" disabled={isReadOnly}>Write</SegmentControl.Item>
			<SegmentControl.Item id="maintain" disabled={isReadOnly}>Maintain</SegmentControl.Item>
			<SegmentControl.Item id="admin" disabled={isReadOnly}>Admin</SegmentControl.Item>
		</SegmentControl>
	</div>

	<div class="principal-editor__section">
		<span class="principal-editor__label">Permissions</span>
		<div class="permission-table">
			{#each permissionRows as row (row.authority)}
				{@const inheritedSource = inheritedAuthorityMap.get(row.authority)}
				{@const isInherited = Boolean(inheritedSource)}
				{@const isChecked = isInherited || includes(stagedOwnGrants, row.authority)}
				<div
					class="permission-row"
					class:inherited={isInherited}
					data-testid={`principal-editor-row-${slug(row.authority)}`}
				>
					<div class="permission-row__copy">
						<strong>{row.authority}</strong>
						<span>{row.label}</span>
					</div>
					{#if inheritedSource}
						<span class="permission-row__source">{inheritedSource}</span>
					{:else if includes(committedOwnGrants, row.authority) !== includes(stagedOwnGrants, row.authority)}
						<Badge style="warning" kind="soft" size="icon">o</Badge>
					{/if}
					{#key `${row.authority}-${isChecked}-${isReadOnly || isInherited}`}
						<Toggle
							testId={`principal-editor-toggle-${slug(row.authority)}`}
							checked={isChecked}
							disabled={isReadOnly || isInherited}
							onchange={(checked) => setAuthority(row.authority, checked)}
						/>
					{/key}
				</div>
			{/each}
		</div>
	</div>

	{#if inheritedGrants.length > 0}
		<InfoMessage style="info" outlined>
			{#snippet title()}Inherited permissions{/snippet}
			{#snippet content()}Inherited rows come from groups; remove a group to revoke.{/snippet}
		</InfoMessage>
	{/if}

	<div class="principal-editor__section">
		<TagInput
			testId="principal-editor-groups"
			label="Groups"
			tags={groupTags}
			readonly={isReadOnly}
			placeholder="Add group"
			onTagsChange={setGroups}
			wide
		/>
	</div>

	<footer class="principal-editor__actions">
		{#if onCancel}
			<Button kind="ghost" disabled={isSaving} onclick={onCancel}>Cancel</Button>
		{/if}
		<Button
			testId="principal-editor-save"
			style="pop"
			disabled={!hasStagedChanges || isReadOnly || isSaving}
			loading={isSaving}
			onclick={save}
		>
			Save changes
		</Button>
	</footer>
</section>

<style>
	.principal-editor {
		display: flex;
		flex-direction: column;
		padding: 12px;
		gap: 12px;
		border: 1px solid var(--border-2);
		border-radius: var(--radius-m);
		background: var(--bg-1);
	}

	.principal-editor__header {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 8px;
	}

	.principal-editor__header h3,
	.principal-editor__header p {
		margin: 0;
	}

	.principal-editor__header p,
	.permission-row__copy span,
	.permission-row__source {
		color: var(--text-2);
	}

	.principal-editor__section {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.principal-editor__label {
		font-weight: 600;
	}

	.permission-table {
		display: flex;
		flex-direction: column;
		overflow: hidden;
		border: 1px solid var(--border-2);
		border-radius: var(--radius-m);
	}

	.permission-row {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto auto;
		align-items: center;
		padding: 8px;
		gap: 8px;
		border-bottom: 1px solid var(--border-2);
	}

	.permission-row:last-child {
		border-bottom: 0;
	}

	.permission-row.inherited {
		background: var(--bg-2);
	}

	.permission-row__copy {
		display: flex;
		flex-direction: column;
		min-width: 0;
	}

	.permission-row__source {
		white-space: nowrap;
	}

	.principal-editor__actions {
		display: flex;
		justify-content: flex-end;
		gap: 6px;
	}
</style>
