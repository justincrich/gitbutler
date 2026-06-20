import type {
	GovernanceAccess,
	GovernancePending,
	GovernanceRendererContract,
	GovernanceTarget,
} from "$lib/governance";

const DEFAULT_ACCESS: GovernanceAccess = {
	authorities: [],
	hasAdminWrite: false,
	isReadOnly: true,
};

export type GovernancePendingStore = ReturnType<typeof createGovernancePendingStore>;

export function createGovernancePendingStore(
	service: GovernanceRendererContract,
	target: GovernanceTarget,
) {
	let access = $state<GovernanceAccess>(DEFAULT_ACCESS);
	let pending = $state<GovernancePending>({
		principals: [],
		pendingCount: 0,
	});
	let isLoading = $state(false);
	let isCommitting = $state(false);
	let error = $state<string | undefined>(undefined);

	async function refresh() {
		isLoading = true;
		error = undefined;
		try {
			const [nextAccess, nextPending] = await Promise.all([
				service.readAccess(target.projectId),
				service.readPending(target),
			]);
			access = nextAccess;
			pending = nextPending;
		} catch (err: unknown) {
			error = err instanceof Error ? err.message : String(err);
		} finally {
			isLoading = false;
		}
	}

	async function commit() {
		if (access.isReadOnly || pending.pendingCount === 0 || isCommitting) return;

		isCommitting = true;
		error = undefined;
		try {
			await service.commitPending(target);
			await refresh();
		} catch (err: unknown) {
			error = err instanceof Error ? err.message : String(err);
		} finally {
			isCommitting = false;
		}
	}

	return {
		get access() {
			return access;
		},
		get error() {
			return error;
		},
		get isCommitting() {
			return isCommitting;
		},
		get isLoading() {
			return isLoading;
		},
		get pendingCount() {
			return pending.pendingCount;
		},
		commit,
		refresh,
	};
}
