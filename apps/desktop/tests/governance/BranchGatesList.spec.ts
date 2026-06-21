import BranchGatesList from "$components/governance/BranchGatesList.svelte";
import { expect, test } from "@playwright/experimental-ct-svelte";
import type { GroupListEntry, GroupListOutcome } from "@gitbutler/but-sdk";
import type {
	BranchGateEntry,
	BranchGatesOutcome,
	BranchProtectionInput,
} from "$components/governance/BranchGatesList.svelte";

const projectId = "project-1";
const targetRef = "refs/remotes/origin/main";

const seededDefinedGroups: GroupListEntry[] = [
	{
		name: "eng",
		authorities: ["reviews:write"],
		members: ["alice"],
	},
	{
		name: "security",
		authorities: ["reviews:write"],
		members: ["bob"],
	},
	{
		name: "platform",
		authorities: ["reviews:write"],
		members: ["carol"],
	},
];

const seededGatesTwoBranches: BranchGateEntry[] = [
	{
		name: "main",
		protected: true,
		min_approvals: 2,
		require_distinct_from_author: true,
		require_approval_from_group: ["eng", "security"],
		pending: false,
	},
	{
		name: "release",
		protected: true,
		min_approvals: 1,
		require_distinct_from_author: false,
		require_approval_from_group: ["platform"],
		pending: false,
	},
];

function cloneGate(gate: BranchGateEntry): BranchGateEntry {
	return {
		...gate,
		require_approval_from_group: [...gate.require_approval_from_group],
	};
}

type Call = {
	command: "branch_gates_read" | "branch_gates_update" | "group_list";
	branch?: string;
	protection?: BranchProtectionInput;
};

function createOutcome(branches: BranchGateEntry[]): BranchGatesOutcome {
	return {
		branches: branches.map(cloneGate),
		caveat: targetRef,
	};
}

function createService(branches = seededGatesTwoBranches, rejectWrites = false) {
	const calls: Call[] = [];
	let currentBranches = branches.map(cloneGate);

	return {
		calls,
		service: {
			async branchGatesRead(): Promise<BranchGatesOutcome> {
				calls.push({ command: "branch_gates_read" });
				return createOutcome(currentBranches);
			},
			async branchGatesUpdate(
				_projectId: string,
				_targetRef: string,
				branch: string,
				protection: BranchProtectionInput,
			): Promise<BranchGatesOutcome> {
				calls.push({ command: "branch_gates_update", branch, protection });
				if (rejectWrites) {
					return {
						code: "perm.denied",
						message: "Permission denied",
					} as unknown as BranchGatesOutcome;
				}
				const nextBranch: BranchGateEntry = {
					name: branch,
					protected: protection.protected,
					min_approvals: protection.min_approvals ?? 0,
					require_distinct_from_author: protection.require_distinct_from_author ?? false,
					require_approval_from_group: protection.require_approval_from_group ?? [],
					pending: true,
				};
				const exists = currentBranches.some((entry) => entry.name === branch);
				currentBranches = exists
					? currentBranches.map((entry) => (entry.name === branch ? nextBranch : entry))
					: [...currentBranches, nextBranch];
				return createOutcome(currentBranches);
			},
			async listGroups(): Promise<GroupListOutcome> {
				calls.push({ command: "group_list" });
				return { groups: seededDefinedGroups };
			},
		},
	};
}

function props(branches = seededGatesTwoBranches, rejectWrites = false) {
	const { calls, service } = createService(branches, rejectWrites);
	return {
		calls,
		mountProps: {
			projectId,
			targetRef,
			branches,
			definedGroups: seededDefinedGroups,
			service,
		},
	};
}

function deniedProps() {
	const calls: Call[] = [];

	return {
		calls,
		mountProps: {
			projectId,
			targetRef,
			branches: seededGatesTwoBranches,
			definedGroups: seededDefinedGroups,
			service: {
				branchGatesUpdateError: "perm.denied Permission denied",
				async branchGatesRead(): Promise<BranchGatesOutcome> {
					calls.push({ command: "branch_gates_read" });
					return createOutcome(seededGatesTwoBranches);
				},
				async branchGatesUpdate(
					_projectId: string,
					_targetRef: string,
					branch: string,
					protection: BranchProtectionInput,
				): Promise<BranchGatesOutcome> {
					calls.push({ command: "branch_gates_update", branch, protection });
					return {
						branches: seededGatesTwoBranches.map(cloneGate),
						caveat: "perm.denied Permission denied",
					};
				},
				async listGroups(): Promise<GroupListOutcome> {
					calls.push({ command: "group_list" });
					return { groups: seededDefinedGroups };
				},
			},
		},
	};
}

test("BranchGatesListRows renders expandable branch gate fields", async ({ mount }) => {
	const { mountProps } = props();
	const component = await mount(BranchGatesList, { props: mountProps });

	await expect(component.getByTestId("branch-gates-list-row-main")).toContainText("main");
	await expect(component.getByTestId("branch-gates-list-row-release")).toContainText("release");

	await component
		.getByTestId("branch-gates-list-row-main")
		.getByRole("button", { name: /main/ })
		.click();

	await expect(component.getByTestId("branch-gates-list-protected-main")).toBeChecked();
	await expect(component.getByTestId("branch-gates-list-min-approvals-main")).toHaveValue("2");
	await expect(component.getByTestId("branch-gates-list-distinct-main")).toBeChecked();
	await expect(component.getByTestId("branch-gates-list-groups-main")).toContainText("eng");
	await expect(component.getByTestId("branch-gates-list-groups-main")).toContainText("security");
});

test("BranchGatesListEdit writes min approvals and marks the row pending", async ({ mount }) => {
	const { calls, mountProps } = props();
	const component = await mount(BranchGatesList, { props: mountProps });

	await component
		.getByTestId("branch-gates-list-row-main")
		.getByRole("button", { name: /main/ })
		.click();
	await component.getByTestId("branch-gates-list-min-approvals-main").fill("3");
	await component.getByTestId("branch-gates-list-min-approvals-main").blur();

	expect(calls).toContainEqual({
		command: "branch_gates_update",
		branch: "main",
		protection: {
			protected: true,
			min_approvals: 3,
			require_distinct_from_author: true,
			require_approval_from_group: ["eng", "security"],
		},
	});
	await expect(component.getByTestId("branch-gates-list-pending-main")).toHaveText("Pending");
});

test("BranchGatesListEmpty renders empty state and adds a staging gate", async ({ mount }) => {
	const { calls, mountProps } = props([]);
	const component = await mount(BranchGatesList, { props: mountProps });

	await expect(component.getByTestId("branch-gates-list-empty")).toContainText(
		"No branch gates yet",
	);
	await component
		.getByTestId("branch-gates-list-empty")
		.getByRole("button", { name: "+ Add" })
		.click();
	await expect(component.getByTestId("branch-gates-list-add-form")).toBeVisible();

	await component.getByTestId("branch-gates-list-add-pattern").fill("staging");
	await component.getByRole("button", { name: "Add gate" }).click();

	expect(calls).toContainEqual({
		command: "branch_gates_update",
		branch: "staging",
		protection: {
			protected: true,
			min_approvals: 1,
			require_distinct_from_author: true,
			require_approval_from_group: [],
		},
	});
});

test("BranchGatesListGroupSelector offers only defined groups", async ({ mount }) => {
	const { mountProps } = props();
	const component = await mount(BranchGatesList, { props: mountProps });

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
});

test("BranchGatesListUnprotectConfirm gates protected off behind a modal", async ({
	mount,
	page,
}) => {
	const { calls, mountProps } = props();
	const component = await mount(BranchGatesList, { props: mountProps });

	await component
		.getByTestId("branch-gates-list-row-main")
		.getByRole("button", { name: /main/ })
		.click();
	await component.getByTestId("branch-gates-list-protected-main").click();
	await expect(page.getByTestId("branch-gates-list-unprotect-modal")).toContainText(
		"Unprotect branch main? Merges will no longer require review.",
	);
	expect(calls.filter((call) => call.command === "branch_gates_update")).toHaveLength(0);
	await expect(component.getByTestId("branch-gates-list-protected-main")).toBeChecked();

	await page.getByRole("button", { name: "Cancel" }).click();
	await expect(page.getByTestId("branch-gates-list-unprotect-modal")).toBeHidden();
	expect(calls.filter((call) => call.command === "branch_gates_update")).toHaveLength(0);
	await expect(component.getByTestId("branch-gates-list-protected-main")).toBeChecked();

	await component.getByTestId("branch-gates-list-protected-main").click();
	await page
		.getByTestId("branch-gates-list-unprotect-modal")
		.getByRole("button", { name: "Unprotect branch" })
		.click();

	expect(calls).toContainEqual({
		command: "branch_gates_update",
		branch: "main",
		protection: {
			protected: false,
			min_approvals: 2,
			require_distinct_from_author: true,
			require_approval_from_group: ["eng", "security"],
		},
	});
});

test("BranchGatesListReadOnly disables mutating controls and fires zero SDK calls", async ({
	mount,
}) => {
	const { calls, mountProps } = props();
	const component = await mount(BranchGatesList, {
		props: {
			...mountProps,
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

	expect(calls.filter((call) => call.command === "branch_gates_update")).toHaveLength(0);
});

test("BranchGatesListWriteDenied shows permission error and reverts min approvals", async ({
	mount,
}) => {
	const { mountProps } = deniedProps();
	const component = await mount(BranchGatesList, { props: mountProps });

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
});
