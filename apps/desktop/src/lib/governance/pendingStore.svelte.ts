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

export type GovernanceReadFailure = {
	code: string;
	message: string;
	remediationHint?: string;
};

export type GovernancePendingStore = ReturnType<typeof createGovernancePendingStore>;

function parseStructuredError(value: string): GovernanceReadFailure | undefined {
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
		};
	} catch {
		return undefined;
	}
}

function governanceReadFailure(error: unknown): GovernanceReadFailure {
	if (error instanceof Error && error.message) {
		const candidate = error as Error & {
			code?: unknown;
			remediation_hint?: unknown;
		};
		const structured = parseStructuredError(error.message);
		if (structured) return structured;

		return {
			code: typeof candidate.code === "string" ? candidate.code : "governance.read_failed",
			message: error.message,
			remediationHint:
				typeof candidate.remediation_hint === "string" ? candidate.remediation_hint : undefined,
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
			};
		}
	}

	return {
		code: "governance.read_failed",
		message: error instanceof Error ? error.message : String(error),
	};
}

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
	let error = $state<GovernanceReadFailure | undefined>(undefined);

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
			if (nextAccess.isNotConfigured || !nextAccess.targetRef) {
				// Not configured, or the backend returned no target ref — defensively skip
				// the pending read rather than falling through to a hardcoded/guessed ref.
				pending = { principals: [], pendingCount: 0 };
				return;
			}
			pending = await service.readPending(resolvedTarget());
		} catch (err: unknown) {
			error = governanceReadFailure(err);
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
			error = governanceReadFailure(err);
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
