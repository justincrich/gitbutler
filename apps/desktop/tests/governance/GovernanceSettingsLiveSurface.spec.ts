import GovernanceSettingsHarness from "$tests/governance/GovernanceSettingsHarness.svelte";
import { expect, test } from "@playwright/experimental-ct-svelte";
import type { Locator } from "@playwright/test";

type BackendCall = {
	command: string;
	args: {
		projectId?: string;
		targetRef?: string;
		branch?: string;
		protection?: {
			protected: boolean;
			min_approvals: number | null;
			require_distinct_from_author: boolean | null;
			require_approval_from_group: string[] | null;
		};
	};
};

async function backendCalls(component: Locator) {
	const rawCalls = await component.getByTestId("governance-branch-gates-calls").textContent();
	return JSON.parse(rawCalls ?? "[]") as BackendCall[];
}

function updateCalls(calls: BackendCall[]): BackendCall[] {
	return calls.filter((call) => call.command === "branch_gates_update");
}

function latestUpdate(calls: BackendCall[]): BackendCall | undefined {
	return updateCalls(calls).at(-1);
}

test("GovernanceSettings Branch Gates renders real gates, writes changes, and enables commit", async ({
	mount,
}) => {
	const component = await mount(GovernanceSettingsHarness, {
		props: {
			pendingCount: 0,
		},
	});

	await component.getByRole("tab", { name: "Branch Gates" }).click();
	await expect(component.getByTestId("branch-gates-list")).toBeVisible();
	await expect(component.getByTestId("governance-branch-gates-control")).toHaveCount(0);

	await component
		.getByTestId("branch-gates-list-row-main")
		.getByRole("button", { name: /main/ })
		.click();
	await expect(component.getByTestId("branch-gates-list-distinct-main")).toBeChecked();

	await component.getByTestId("branch-gates-list-distinct-main").click();

	await expect(component.getByTestId("branch-gates-list-pending-main")).toHaveText("Pending");
	await expect(component.getByTestId("governance-pending-banner")).toContainText(
		"1 pending changes",
	);
	await expect(component.getByTestId("governance-commit-button")).toBeEnabled();

	expect(latestUpdate(await backendCalls(component))).toEqual(
		expect.objectContaining({
			command: "branch_gates_update",
			args: {
				projectId: "ct-project",
				targetRef: "refs/remotes/origin/main",
				branch: "main",
				protection: {
					protected: true,
					min_approvals: 2,
					require_distinct_from_author: false,
					require_approval_from_group: ["eng"],
				},
			},
		}),
	);
});

test("GovernanceSettings Rules principal selector switches scoped rules queries", async ({
	mount,
}) => {
	const component = await mount(GovernanceSettingsHarness, {
		props: {
			pendingCount: 0,
			principals: [
				{
					principalId: "agent:codex-staging",
					ownGrants: ["contents:read"],
					inheritedGrants: [],
					groupMemberships: [],
					pending: false,
				},
				{
					principalId: "agent:cursor-bot",
					ownGrants: ["contents:read"],
					inheritedGrants: [],
					groupMemberships: [],
					pending: false,
				},
			],
		},
	});

	await component.getByRole("tab", { name: "Rules" }).click();
	await expect(component.getByTestId("governance-rules-principal-select")).toBeVisible();
	await expect(component.getByTestId("governance-rules-control")).toBeVisible();

	await component
		.getByTestId("governance-rules-principal-select")
		.selectOption("agent:codex-staging");

	await expect(component.getByTestId("governance-last-principal-project-id")).toHaveText(
		"ct-project",
	);
	await expect(component.getByTestId("governance-last-principal-id")).toHaveText(
		"agent:codex-staging",
	);
	await expect(component.getByText("src/a1.ts")).toBeVisible();
	await expect(component.getByText("src/a2.ts")).toBeVisible();
	await expect(component.getByText("src/b1.ts")).toHaveCount(0);

	await component.getByTestId("governance-rules-principal-select").selectOption("agent:cursor-bot");

	await expect(component.getByTestId("governance-last-principal-id")).toHaveText(
		"agent:cursor-bot",
	);
	await expect(component.getByText("src/b1.ts")).toBeVisible();
	await expect(component.getByText("src/a1.ts")).toHaveCount(0);
	await expect(component.getByText("src/a2.ts")).toHaveCount(0);
	await expect(component.getByTestId("governance-workspace-rules-call-count")).toHaveText("0");
});

test("GovernanceSettings read-only disables real branch gates and rules controls", async ({
	mount,
}) => {
	const component = await mount(GovernanceSettingsHarness, {
		props: {
			pendingCount: 2,
			hasAdminWrite: false,
			principals: [
				{
					principalId: "agent:codex-staging",
					ownGrants: ["contents:read"],
					inheritedGrants: [],
					groupMemberships: [],
					pending: false,
				},
			],
		},
	});

	await component.getByRole("tab", { name: "Branch Gates" }).click();
	await expect(component.getByTestId("branch-gates-list")).toBeVisible();
	await component
		.getByTestId("branch-gates-list-row-main")
		.getByRole("button", { name: /main/ })
		.click();
	await expect(component.getByTestId("branch-gates-list-distinct-main")).toBeDisabled();
	await expect(component.getByRole("button", { name: "+ Add" })).toBeDisabled();

	await component.getByRole("tab", { name: "Rules" }).click();
	await expect(component.getByTestId("governance-rules-principal-select")).toBeDisabled();
	await expect(component.getByTestId("governance-rules-control")).toBeVisible();
	await expect(component.getByTestId("governance-principal-rules-call-count")).toHaveText("0");

	expect(updateCalls(await backendCalls(component))).toHaveLength(0);
});
