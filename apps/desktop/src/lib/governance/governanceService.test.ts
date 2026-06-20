import {
	GOVERNANCE_COMMIT_MESSAGE,
	createGovernanceRendererContract,
	governanceAccessFromStatus,
	type GovernanceCommitOutcome,
	type GovernancePending,
} from "$lib/governance/governanceService";
import { describe, expect, test } from "vitest";

describe("governance renderer contract", () => {
	test("reads typed pendingCount from the backend governance_pending command", async () => {
		const pending: GovernancePending = {
			pendingCount: 2,
			principals: [
				{
					id: "rust-implementer",
					committedEffective: ["contents:write"],
					workingEffective: ["contents:write", "administration:write"],
					tokens: [
						{
							authority: "administration:write",
							committed: false,
							working: true,
							pending: true,
							change: "grant",
						},
					],
				},
			],
		};
		const calls: Array<{ command: string; args: unknown }> = [];
		const contract = createGovernanceRendererContract({
			invoke: async <T>(command: string, args: unknown): Promise<T> => {
				calls.push({ command, args });
				return pending as T;
			},
		});

		const result = await contract.readPending({
			projectId: "project-1",
			targetRef: "refs/remotes/origin/main",
		});

		expect(result.pendingCount).toBe(2);
		expect(calls).toEqual([
			{
				command: "governance_pending",
				args: { projectId: "project-1", targetRef: "refs/remotes/origin/main" },
			},
		]);
	});

	test("derives hasAdminWrite and read-only from backend authority tokens", async () => {
		expect(
			governanceAccessFromStatus({ authorities: ["contents:write", "administration:write"] }),
		).toEqual({
			authorities: ["contents:write", "administration:write"],
			hasAdminWrite: true,
			isReadOnly: false,
		});
		expect(governanceAccessFromStatus({ authorities: ["administration:read"] })).toEqual({
			authorities: ["administration:read"],
			hasAdminWrite: false,
			isReadOnly: true,
		});
	});

	test("commits pending governance changes through the backend command contract", async () => {
		const outcome: GovernanceCommitOutcome = {
			commitId: "abc123",
			message: GOVERNANCE_COMMIT_MESSAGE,
			committedPaths: ["governance-config"],
		};
		const calls: Array<{ command: string; args: unknown }> = [];
		const contract = createGovernanceRendererContract({
			invoke: async <T>(command: string, args: unknown): Promise<T> => {
				calls.push({ command, args });
				return outcome as T;
			},
		});

		const result = await contract.commitPending({
			projectId: "project-1",
			targetRef: "refs/remotes/origin/main",
		});

		expect(result.message).toBe("chore: update governance config");
		expect(calls).toEqual([
			{
				command: "governance_commit",
				args: { projectId: "project-1", targetRef: "refs/remotes/origin/main" },
			},
		]);
	});
});
