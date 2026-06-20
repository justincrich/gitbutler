import type { IBackend } from "$lib/backend";
import type { GovernanceStatus } from "@gitbutler/but-sdk";

export const GOVERNANCE_COMMIT_MESSAGE = "chore: update governance config";
const ADMIN_WRITE_AUTHORITY = "administration:write";

export type GovernanceTarget = {
	projectId: string;
	targetRef: string;
};

export type GovernancePendingToken = {
	authority: string;
	committed: boolean;
	working: boolean;
	pending: boolean;
	change?: "grant" | "revoke";
};

export type GovernancePendingPrincipal = {
	id: string;
	committedEffective: string[];
	workingEffective: string[];
	tokens: GovernancePendingToken[];
};

export type GovernancePending = {
	principals: GovernancePendingPrincipal[];
	pendingCount: number;
};

export type GovernanceInheritedGrant = {
	authority: string;
	sourceLabel: string;
};

export type PrincipalListEntry = {
	principalId: string;
	ownGrants: string[];
	inheritedGrants: GovernanceInheritedGrant[];
	groupMemberships: string[];
	pending: boolean;
	isCurrentUser?: boolean;
};

export type GovernancePrincipalsList = {
	principals: PrincipalListEntry[];
};

export type GovernanceAccess = {
	authorities: string[];
	hasAdminWrite: boolean;
	isReadOnly: boolean;
};

export type GovernanceCommitOutcome = {
	commitId: string;
	message: typeof GOVERNANCE_COMMIT_MESSAGE;
	committedPaths: string[];
};

export type GovernanceRendererContract = {
	readPending(target: GovernanceTarget): Promise<GovernancePending>;
	readPrincipals(target: GovernanceTarget): Promise<GovernancePrincipalsList>;
	readAccess(projectId: string): Promise<GovernanceAccess>;
	commitPending(target: GovernanceTarget): Promise<GovernanceCommitOutcome>;
};

type GovernanceBackend = Pick<IBackend, "invoke">;

export function createGovernanceRendererContract(
	backend: GovernanceBackend,
): GovernanceRendererContract {
	return {
		readPending(target) {
			return backend.invoke<GovernancePending>("governance_pending", target);
		},
		readPrincipals(target) {
			return backend.invoke<GovernancePrincipalsList>("governance_principals_list", target);
		},
		async readAccess(projectId) {
			const status = await backend.invoke<GovernanceStatus>("governance_status_read", {
				projectId,
			});
			return governanceAccessFromStatus(status);
		},
		commitPending(target) {
			return backend.invoke<GovernanceCommitOutcome>("governance_commit", target);
		},
	};
}

export function governanceAccessFromStatus(status: GovernanceStatus): GovernanceAccess {
	const authorities = status.authorities;
	const hasAdminWrite = authorities.includes(ADMIN_WRITE_AUTHORITY);
	return {
		authorities,
		hasAdminWrite,
		isReadOnly: !hasAdminWrite,
	};
}
