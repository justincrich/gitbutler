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
	isNotConfigured: false,
	targetRef: "",
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

	// The backend resolves the real governance target ref; reuse it for follow-up
	// reads/commits instead of the (possibly stale) ref the renderer was constructed with.
	function resolvedTarget(): GovernanceTarget {
		return { projectId: target.projectId, targetRef: access.targetRef || target.targetRef };
	}

	async function refresh() {
		isLoading = true;
		error = undefined;
		try {
			// Resolve access first: it reports whether governance is configured AND the
			// workspace-resolved target ref. When it isn't set up we skip the pending read
			// entirely (it would error on the missing config) and surface guidance instead.
			const nextAccess = await service.readAccess(target.projectId);
			access = nextAccess;
			if (nextAccess.isNotConfigured) {
				pending = { principals: [], pendingCount: 0 };
				return;
			}
			pending = await service.readPending(resolvedTarget());
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
			await service.commitPending(resolvedTarget());
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
