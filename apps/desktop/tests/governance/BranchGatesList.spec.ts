import BranchGatesListBackendHarness from "$tests/governance/BranchGatesListBackendHarness.svelte";
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
	const rawCalls = await component.getByTestId("branch-gates-backend-calls").textContent();
	return JSON.parse(rawCalls ?? "[]") as BackendCall[];
}

async function waitForBackendCall(component: Locator, command: string) {
	await expect
		.poll(async () => (await backendCalls(component)).some((call) => call.command === command))
		.toBe(true);
}

function updateCalls(calls: BackendCall[]): BackendCall[] {
	return calls.filter((call) => call.command === "branch_gates_update");
}

function latestUpdate(calls: BackendCall[]): BackendCall | undefined {
	return updateCalls(calls).at(-1);
}

test("BranchGatesListRows renders expandable branch gate fields", async ({ mount }) => {
	const component = await mount(BranchGatesListBackendHarness);

	await expect(component.getByTestId("branch-gates-list-row-main")).toContainText("main");
	await expect(component.getByTestId("branch-gates-list-row-release")).toContainText("release");

	await waitForBackendCall(component, "branch_gates_read");
	await waitForBackendCall(component, "group_list");

	await component
		.getByTestId("branch-gates-list-row-main")
		.getByRole("button", { name: /main/ })
		.click();

	await expect(component.getByTestId("branch-gates-list-protected-main")).toBeChecked();
	await expect(component.getByTestId("branch-gates-list-min-approvals-main")).toHaveValue("2");
	await expect(component.getByTestId("branch-gates-list-distinct-main")).toBeChecked();
	await expect(component.getByTestId("branch-gates-list-groups-main")).toContainText("eng");
	await expect(component.getByTestId("branch-gates-list-groups-main")).toContainText("security");

	const calls = await backendCalls(component);
	expect(calls).toEqual(
		expect.arrayContaining([
			expect.objectContaining({
				command: "branch_gates_read",
				args: { projectId: "project-1", targetRef: "refs/remotes/origin/main" },
			}),
			expect.objectContaining({
				command: "group_list",
				args: { projectId: "project-1" },
			}),
		]),
	);
});

test("BranchGatesListEdit writes min approvals and marks the row pending", async ({ mount }) => {
	const component = await mount(BranchGatesListBackendHarness);

	await component
		.getByTestId("branch-gates-list-row-main")
		.getByRole("button", { name: /main/ })
		.click();
	await component.getByTestId("branch-gates-list-min-approvals-main").fill("3");
	await component.getByTestId("branch-gates-list-min-approvals-main").blur();

	await expect(component.getByTestId("branch-gates-list-pending-main")).toHaveText("Pending");

	const calls = await backendCalls(component);
	expect(latestUpdate(calls)).toEqual(
		expect.objectContaining({
			command: "branch_gates_update",
			args: {
				projectId: "project-1",
				targetRef: "refs/remotes/origin/main",
				branch: "main",
				protection: {
					protected: true,
					min_approvals: 3,
					require_distinct_from_author: true,
					require_approval_from_group: ["eng", "security"],
				},
			},
		}),
	);
});

test("BranchGatesListEmpty renders empty state and adds a staging gate", async ({ mount }) => {
	const component = await mount(BranchGatesListBackendHarness, {
		props: {
			scenario: "seeded_empty_gates",
		},
	});

	await expect(component.getByTestId("branch-gates-list-empty")).toContainText(
		"No branch gates yet",
	);
	await waitForBackendCall(component, "branch_gates_read");
	await waitForBackendCall(component, "group_list");

	await component
		.getByTestId("branch-gates-list-empty")
		.getByRole("button", { name: "+ Add" })
		.click();
	await expect(component.getByTestId("branch-gates-list-add-form")).toBeVisible();

	await component.getByTestId("branch-gates-list-add-pattern").fill("staging");
	await component.getByRole("button", { name: "Add gate" }).click();

	const calls = await backendCalls(component);
	expect(latestUpdate(calls)).toEqual(
		expect.objectContaining({
			command: "branch_gates_update",
			args: {
				projectId: "project-1",
				targetRef: "refs/remotes/origin/main",
				branch: "staging",
				protection: {
					protected: true,
					min_approvals: 1,
					require_distinct_from_author: true,
					require_approval_from_group: [],
				},
			},
		}),
	);
});

test("BranchGatesListGroupSelector offers only defined groups", async ({ mount }) => {
	const component = await mount(BranchGatesListBackendHarness);

	await component
		.getByTestId("branch-gates-list-row-main")
		.getByRole("button", { name: /main/ })
		.click();

	const options = component
		.getByTestId("branch-gates-list-group-options-main")
		.locator("label span");
	await expect(options).toHaveText(["eng", "security", "platform"]);
	await expect(component.getByTestId("branch-gates-list-group-options-main")).not.toContainText(
		"undefined-group",
	);

	const calls = await backendCalls(component);
	expect(calls.some((call) => call.command === "group_list")).toBe(true);
});

test("BranchGatesListUnprotectConfirm gates protected off behind a modal", async ({
	mount,
	page,
}) => {
	const component = await mount(BranchGatesListBackendHarness);

	await component
		.getByTestId("branch-gates-list-row-main")
		.getByRole("button", { name: /main/ })
		.click();
	await component.getByTestId("branch-gates-list-protected-main").click();
	await expect(page.getByTestId("branch-gates-list-unprotect-modal")).toContainText(
		"Unprotect branch main? Merges will no longer require review.",
	);
	expect(updateCalls(await backendCalls(component))).toHaveLength(0);
	await expect(component.getByTestId("branch-gates-list-protected-main")).toBeChecked();

	await page.getByRole("button", { name: "Cancel" }).click();
	await expect(page.getByTestId("branch-gates-list-unprotect-modal")).toBeHidden();
	expect(updateCalls(await backendCalls(component))).toHaveLength(0);
	await expect(component.getByTestId("branch-gates-list-protected-main")).toBeChecked();

	await component.getByTestId("branch-gates-list-protected-main").click();
	await page
		.getByTestId("branch-gates-list-unprotect-modal")
		.getByRole("button", { name: "Unprotect branch" })
		.click();

	await expect
		.poll(async () => updateCalls(await backendCalls(component)).length)
		.toBeGreaterThan(0);
	expect(latestUpdate(await backendCalls(component))).toEqual(
		expect.objectContaining({
			command: "branch_gates_update",
			args: {
				projectId: "project-1",
				targetRef: "refs/remotes/origin/main",
				branch: "main",
				protection: {
					protected: false,
					min_approvals: 2,
					require_distinct_from_author: true,
					require_approval_from_group: ["eng", "security"],
				},
			},
		}),
	);
});

test("BranchGatesListReadOnly disables mutating controls and fires zero SDK calls", async ({
	mount,
}) => {
	const component = await mount(BranchGatesListBackendHarness, {
		props: {
			scenario: "seeded_gates_readonly",
			isReadOnly: true,
		},
	});

	await component
		.getByTestId("branch-gates-list-row-main")
		.getByRole("button", { name: /main/ })
		.click();

	await expect(component.getByTestId("branch-gates-list-protected-main")).toBeDisabled();
	await expect(component.getByTestId("branch-gates-list-min-approvals-main")).toBeDisabled();
	await expect(component.getByTestId("branch-gates-list-distinct-main")).toBeDisabled();
	await expect(component.getByLabel("eng")).toBeDisabled();
	await expect(component.getByRole("button", { name: "+ Add" })).toBeDisabled();

	await component.getByTestId("branch-gates-list-protected-main").click({ force: true });
	await component.getByTestId("branch-gates-list-min-approvals-main").fill("3", { force: true });
	await component.getByLabel("eng").click({ force: true });

	expect(updateCalls(await backendCalls(component))).toHaveLength(0);
});

test("BranchGatesListWriteDenied shows permission error and reverts min approvals", async ({
	mount,
}) => {
	const component = await mount(BranchGatesListBackendHarness, {
		props: {
			scenario: "seeded_write_denied",
		},
	});

	await component
		.getByTestId("branch-gates-list-row-main")
		.getByRole("button", { name: /main/ })
		.click();
	await component.getByTestId("branch-gates-list-min-approvals-main").fill("3");
	await component.getByTestId("branch-gates-list-min-approvals-main").blur();

	await expect(component.getByTestId("branch-gates-list-write-error")).toContainText("perm.denied");
	await expect(component.getByTestId("branch-gates-list-write-error")).toContainText(
		"Permission denied",
	);
	await expect(component.getByTestId("branch-gates-list-min-approvals-main")).toHaveValue("2");

	const calls = await backendCalls(component);
	expect(latestUpdate(calls)).toEqual(
		expect.objectContaining({
			command: "branch_gates_update",
			args: expect.objectContaining({
				branch: "main",
				protection: expect.objectContaining({ min_approvals: 3 }),
			}),
		}),
	);
});
