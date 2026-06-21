<script lang="ts" module>
	import type { GrantOutcome, GroupWriteOutcome, PermWriteOutcome } from "@gitbutler/but-sdk";

	export type PrincipalInheritedGrant = {
		authority: string;
		sourceLabel: string;
	};

	export type PrincipalEditorWriteFailure = {
		code: string;
		message?: string;
		remediation_hint?: string;
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

	type WriteDenial = {
		code: string;
		message: string;
		remediationHint?: string;
		canRetry: boolean;
	};

	type WriteSet = {
		ownGrants: string[];
		groups: string[];
		grantsToAdd: string[];
		grantsToRemove: string[];
		groupsToAdd: string[];
		groupsToRemove: string[];
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
	let saveError = $state<WriteDenial | undefined>();
	let retryFailedWrite = $state<(() => Promise<void>) | undefined>();
	let isSaving = $state(false);
	const isWriteLocked = $derived(Boolean(saveError?.canRetry));
	const controlsDisabled = $derived(isReadOnly || isSaving || isWriteLocked);
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
		retryFailedWrite = undefined;

		if (isCurrentUser && authority === "administration:write" && checked) {
			saveError = {
				code: "perm.denied",
				message: "You cannot modify your own administration grants",
				canRetry: false,
			};
			return;
		}

		if (checked) {
			stagedOwnGrants = uniqueSorted([...stagedOwnGrants, authority]);
		} else {
			stagedOwnGrants = stagedOwnGrants.filter((grant) => grant !== authority);
		}

		selectedPreset = resolvePreset(stagedOwnGrants);
	}

	function applyPreset(preset: string) {
		saveError = undefined;
		retryFailedWrite = undefined;
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
		retryFailedWrite = undefined;
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

	function parseStructuredError(value: string): WriteDenial | undefined {
		try {
			const parsed: unknown = JSON.parse(value);
			if (typeof parsed !== "object" || parsed === null || !("code" in parsed)) return undefined;

			const code = (parsed as { code: unknown }).code;
			if (typeof code !== "string") return undefined;

			const message = (parsed as { message?: unknown }).message;
			const remediationHint = (parsed as { remediation_hint?: unknown }).remediation_hint;

			return {
				code,
				message: typeof message === "string" ? message : code,
				remediationHint: typeof remediationHint === "string" ? remediationHint : undefined,
				canRetry: true,
			};
		} catch {
			return undefined;
		}
	}

	function writeDenial(error: unknown, canRetry: boolean): WriteDenial {
		if (error instanceof Error && error.message) {
			const candidate = error as Error & { code?: unknown };
			const structured = parseStructuredError(error.message);
			if (structured) return { ...structured, canRetry };

			return {
				code: typeof candidate.code === "string" ? candidate.code : "governance.write_failed",
				message: error.message,
				canRetry,
			};
		}

		if (typeof error === "object" && error !== null && "code" in error) {
			const code = (error as { code: unknown }).code;
			if (typeof code === "string") {
				const message = (error as { message?: unknown }).message;
				const remediationHint = (error as { remediation_hint?: unknown }).remediation_hint;

				return {
					code,
					message: typeof message === "string" ? message : code,
					remediationHint: typeof remediationHint === "string" ? remediationHint : undefined,
					canRetry,
				};
			}
		}

		return {
			code: "governance.write_failed",
			message: "The requested governance write failed.",
			canRetry,
		};
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

	function createWriteSet(): WriteSet {
		return {
			ownGrants: [...stagedOwnGrants],
			groups: [...stagedGroups],
			grantsToAdd: difference(stagedOwnGrants, committedOwnGrants),
			grantsToRemove: difference(committedOwnGrants, stagedOwnGrants),
			groupsToAdd: difference(stagedGroups, committedGroups),
			groupsToRemove: difference(committedGroups, stagedGroups),
		};
	}

	async function applyWriteSet(writeSet: WriteSet) {
		if (writeSet.grantsToAdd.length > 0) {
			assertWriteSucceeded(
				await service.permGrant(projectId, targetRef, principalId, writeSet.grantsToAdd),
			);
			assertServiceAllowed();
		}

		if (writeSet.grantsToRemove.length > 0) {
			assertWriteSucceeded(
				await service.permRevoke(projectId, targetRef, principalId, writeSet.grantsToRemove),
			);
			assertServiceAllowed();
		}

		for (const group of writeSet.groupsToAdd) {
			assertWriteSucceeded(await service.groupAddMember(projectId, targetRef, group, principalId));
			assertServiceAllowed();
		}

		for (const group of writeSet.groupsToRemove) {
			assertWriteSucceeded(
				await service.groupRemoveMember(projectId, targetRef, group, principalId),
			);
			assertServiceAllowed();
		}

		committedOwnGrants = [...writeSet.ownGrants];
		stagedOwnGrants = [...writeSet.ownGrants];
		committedGroups = [...writeSet.groups];
		stagedGroups = [...writeSet.groups];
		selectedPreset = resolvePreset(stagedOwnGrants);
		onSaved?.();
	}

	async function runWriteSet(writeSet: WriteSet) {
		isSaving = true;
		saveError = undefined;

		try {
			await applyWriteSet(writeSet);
			retryFailedWrite = undefined;
		} catch (error) {
			retryFailedWrite = () => runWriteSet(writeSet);
			saveError = writeDenial(error, true);
			resetStaged();
		} finally {
			isSaving = false;
		}
	}

	function save() {
		if (isSaving || !hasStagedChanges || isReadOnly || isWriteLocked) return;

		void runWriteSet(createWriteSet());
	}

	function retryWrite() {
		void retryFailedWrite?.();
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
		{@const denial = saveError}
		<InfoMessage testId="principal-editor-denial" style="danger" outlined>
			{#snippet title()}{denial.code}{/snippet}
			{#snippet content()}
				{denial.message}
				{#if denial.remediationHint}
					{denial.remediationHint}
				{/if}
				{#if denial.canRetry}
					<button class="principal-editor__retry" type="button" onclick={retryWrite}>Retry</button>
				{/if}
			{/snippet}
		</InfoMessage>
	{/if}

	<div class="principal-editor__section">
		<span class="principal-editor__label">Preset</span>
		<SegmentControl selected={selectedPreset} onselect={applyPreset}>
			<SegmentControl.Item id="read" disabled={controlsDisabled}>Read</SegmentControl.Item>
			<SegmentControl.Item id="triage" disabled={controlsDisabled}>Triage</SegmentControl.Item>
			<SegmentControl.Item id="write" disabled={controlsDisabled}>Write</SegmentControl.Item>
			<SegmentControl.Item id="maintain" disabled={controlsDisabled}>Maintain</SegmentControl.Item>
			<SegmentControl.Item id="admin" disabled={controlsDisabled}>Admin</SegmentControl.Item>
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
					{#key `${row.authority}-${isChecked}-${controlsDisabled || isInherited}-${saveError?.message ?? ""}`}
						<Toggle
							testId={`principal-editor-toggle-${slug(row.authority)}`}
							checked={isChecked}
							disabled={controlsDisabled || isInherited}
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
			readonly={controlsDisabled}
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
			disabled={!hasStagedChanges || controlsDisabled}
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
		padding: var(--clr-space-12);
		gap: var(--clr-space-12);
		border: 1px solid var(--clr-border-2);
		border-radius: var(--radius-m);
		background: var(--clr-bg-1);
	}

	.principal-editor__header {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: var(--clr-space-8);
	}

	.principal-editor__header h3,
	.principal-editor__header p {
		margin: 0;
	}

	.principal-editor__header p,
	.permission-row__copy span,
	.permission-row__source {
		color: var(--clr-text-2);
	}

	.principal-editor__section {
		display: flex;
		flex-direction: column;
		gap: var(--clr-space-6);
	}

	.principal-editor__label {
		font-weight: 600;
	}

	.permission-table {
		display: flex;
		flex-direction: column;
		overflow: hidden;
		border: 1px solid var(--clr-border-2);
		border-radius: var(--radius-m);
	}

	.permission-row {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto auto;
		align-items: center;
		padding: var(--clr-space-8);
		gap: var(--clr-space-8);
		border-bottom: 1px solid var(--clr-border-2);
	}

	.permission-row:last-child {
		border-bottom: 0;
	}

	.permission-row.inherited {
		background: var(--clr-bg-2);
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
		gap: var(--clr-space-6);
	}

	.principal-editor__retry {
		display: inline-flex;
		margin-left: var(--clr-space-6);
		padding: var(--clr-space-2) var(--clr-space-6);
		border: 1px solid currentColor;
		border-radius: var(--radius-s);
		background: transparent;
		color: inherit;
		font: inherit;
		cursor: pointer;
	}
</style>
