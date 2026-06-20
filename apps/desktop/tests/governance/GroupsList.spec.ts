import GroupsList from "$components/governance/GroupsList.svelte";
import { expect, test } from "@playwright/experimental-ct-svelte";
import type { GroupListEntry, GroupListOutcome, GroupWriteOutcome } from "@gitbutler/but-sdk";

const projectId = "project-1";
const targetRef = "refs/remotes/origin/main";

const seededGroups: GroupListEntry[] = [
	{
		name: "eng",
		authorities: ["contents:write"],
		members: ["claude-agent", "codex-agent"],
	},
	{
		name: "platform",
		authorities: ["contents:read"],
		members: ["cursor-bot"],
	},
];

type Call = {
	command: string;
	group: string;
	authority?: string;
	member?: string;
};

function createService(groups: GroupListEntry[] = seededGroups) {
	const calls: Call[] = [];
	let currentGroups = groups.map((group) => ({
		...group,
		authorities: [...group.authorities],
		members: [...group.members],
	}));

	function outcome(group: string, values: Partial<GroupWriteOutcome> = {}): GroupWriteOutcome {
		return {
			group,
			authorities: values.authorities ?? [],
			member: values.member,
			caveat: targetRef,
		};
	}

	return {
		calls,
		service: {
			async listGroups(): Promise<GroupListOutcome> {
				return { groups: currentGroups };
			},
			async groupCreate(
				_projectId: string,
				_targetRef: string,
				group: string,
				authorities: string[],
			): Promise<GroupWriteOutcome> {
				calls.push({ command: "group_create", group, authority: authorities[0] });
				currentGroups = [...currentGroups, { name: group, authorities, members: [] }];
				return outcome(group, { authorities });
			},
			async groupGrant(
				_projectId: string,
				_targetRef: string,
				group: string,
				authorities: string[],
			): Promise<GroupWriteOutcome> {
				calls.push({ command: "group_grant", group, authority: authorities[0] });
				currentGroups = currentGroups.map((entry) =>
					entry.name === group
						? { ...entry, authorities: [...new Set([...entry.authorities, ...authorities])] }
						: entry,
				);
				return outcome(group, { authorities });
			},
			async groupRevoke(
				_projectId: string,
				_targetRef: string,
				group: string,
				authorities: string[],
			): Promise<GroupWriteOutcome> {
				calls.push({ command: "group_revoke", group, authority: authorities[0] });
				currentGroups = currentGroups.map((entry) =>
					entry.name === group
						? {
								...entry,
								authorities: entry.authorities.filter(
									(authority) => !authorities.includes(authority),
								),
							}
						: entry,
				);
				return outcome(group, { authorities });
			},
			async groupAddMember(
				_projectId: string,
				_targetRef: string,
				group: string,
				member: string,
			): Promise<GroupWriteOutcome> {
				calls.push({ command: "group_add_member", group, member });
				currentGroups = currentGroups.map((entry) =>
					entry.name === group
						? { ...entry, members: [...new Set([...entry.members, member])] }
						: entry,
				);
				return outcome(group, { member });
			},
			async groupRemoveMember(
				_projectId: string,
				_targetRef: string,
				group: string,
				member: string,
			): Promise<GroupWriteOutcome> {
				calls.push({ command: "group_remove_member", group, member });
				currentGroups = currentGroups.map((entry) =>
					entry.name === group
						? { ...entry, members: entry.members.filter((value) => value !== member) }
						: entry,
				);
				return outcome(group, { member });
			},
			async groupDelete(
				_projectId: string,
				_targetRef: string,
				group: string,
			): Promise<GroupWriteOutcome> {
				calls.push({ command: "group_delete", group });
				currentGroups = currentGroups.filter((entry) => entry.name !== group);
				return outcome(group);
			},
		},
	};
}

function props(groups = seededGroups) {
	const { calls, service } = createService(groups);
	return {
		calls,
		mountProps: {
			projectId,
			targetRef,
			groups,
			service,
		},
	};
}

test("GroupsListRows renders expandable groups with grants and members", async ({ mount }) => {
	const { mountProps } = props();
	const component = await mount(GroupsList, { props: mountProps });

	await expect(component.getByTestId("groups-list-row-eng")).toContainText("eng");
	await expect(component.getByTestId("groups-list-row-platform")).toContainText("platform");

	await component.getByTestId("groups-list-row-eng").getByRole("button", { name: /eng/ }).click();

	await expect(component.getByTestId("groups-list-toggle-eng-contents-write")).toBeChecked();
	await expect(component.getByTestId("groups-list-members-eng")).toContainText("claude-agent");
	await expect(component.getByTestId("groups-list-members-eng")).toContainText("codex-agent");
});

test("GroupsListSDKCalls creates groups and grants permissions immediately", async ({ mount }) => {
	const { calls, mountProps } = props();
	const component = await mount(GroupsList, { props: mountProps });

	await component.getByTestId("groups-list-create-name").fill("security");
	await component.getByRole("button", { name: "+ Create group" }).click();

	await component.getByTestId("groups-list-row-eng").getByRole("button", { name: /eng/ }).click();
	await component.getByTestId("groups-list-toggle-eng-merge").click();

	expect(calls).toContainEqual({
		command: "group_create",
		group: "security",
		authority: undefined,
	});
	expect(calls).toContainEqual({ command: "group_grant", group: "eng", authority: "merge" });
});

test("GroupsListEmpty renders empty state with create action", async ({ mount }) => {
	const { mountProps } = props([]);
	const component = await mount(GroupsList, { props: mountProps });

	await expect(component.getByTestId("groups-list-empty")).toContainText("No groups yet");
	await expect(
		component.getByTestId("groups-list-empty").getByRole("button", { name: "+ Create group" }),
	).toBeVisible();
});

test("GroupsListDeleteConfirm requires confirmation before delete", async ({ mount, page }) => {
	const { calls, mountProps } = props();
	const component = await mount(GroupsList, { props: mountProps });

	await component.getByTestId("groups-list-row-eng").getByRole("button", { name: /eng/ }).click();
	await component.getByTestId("groups-list-delete-eng").click();
	await expect(page.getByTestId("groups-list-delete-modal")).toContainText("eng");
	expect(calls.filter((call) => call.command === "group_delete")).toHaveLength(0);

	await page.getByRole("button", { name: "Cancel" }).click();
	expect(calls.filter((call) => call.command === "group_delete")).toHaveLength(0);

	await component.getByTestId("groups-list-delete-eng").click();
	await page
		.getByTestId("groups-list-delete-modal")
		.getByRole("button", { name: "Delete group" })
		.click();
	expect(calls).toContainEqual({ command: "group_delete", group: "eng" });
});

test("GroupsListRemoveMember removes a non-last member chip immediately", async ({ mount }) => {
	const { calls, mountProps } = props();
	const component = await mount(GroupsList, { props: mountProps });

	await component.getByTestId("groups-list-row-eng").getByRole("button", { name: /eng/ }).click();
	await component
		.getByTestId("groups-list-members-eng")
		.getByRole("button", { name: "Remove tag" })
		.last()
		.click();

	expect(calls).toContainEqual({
		command: "group_remove_member",
		group: "eng",
		member: "codex-agent",
	});
	await expect(component.getByTestId("groups-list-members-eng")).not.toContainText("codex-agent");
});

test("GroupsListLastMemberWarning can cancel last member removal for gate-referenced group", async ({
	mount,
	page,
}) => {
	const { calls, mountProps } = props([
		{
			name: "eng",
			authorities: ["contents:write"],
			members: ["claude-agent"],
		},
	]);
	const component = await mount(GroupsList, {
		props: {
			...mountProps,
			gateReferencedGroups: ["eng"],
		},
	});

	await component.getByTestId("groups-list-row-eng").getByRole("button", { name: /eng/ }).click();
	await component
		.getByTestId("groups-list-members-eng")
		.getByRole("button", { name: "Remove tag", exact: true })
		.click();

	await expect(page.getByTestId("groups-list-last-member-modal")).toContainText(
		"referenced by a branch gate",
	);
	expect(calls.filter((call) => call.command === "group_remove_member")).toHaveLength(0);

	await page.getByRole("button", { name: "Cancel" }).click();

	expect(calls.filter((call) => call.command === "group_remove_member")).toHaveLength(0);
	await expect(component.getByTestId("groups-list-members-eng")).toContainText("claude-agent");
});

test("GroupsListRevokeToggle revokes an enabled grant immediately", async ({ mount }) => {
	const { calls, mountProps } = props();
	const component = await mount(GroupsList, { props: mountProps });

	await component.getByTestId("groups-list-row-eng").getByRole("button", { name: /eng/ }).click();
	await component.getByTestId("groups-list-toggle-eng-contents-write").click();

	expect(calls).toContainEqual({
		command: "group_revoke",
		group: "eng",
		authority: "contents:write",
	});
});

test("GroupsListReadOnly disables mutating controls and fires zero SDK calls", async ({
	mount,
}) => {
	const { calls, mountProps } = props();
	const component = await mount(GroupsList, {
		props: {
			...mountProps,
			isReadOnly: true,
		},
	});

	await component.getByTestId("groups-list-row-eng").getByRole("button", { name: /eng/ }).click();

	await expect(component.getByTestId("groups-list-toggle-eng-contents-write")).toBeDisabled();
	await component.getByTestId("groups-list-toggle-eng-contents-write").click({ force: true });
	await component.getByRole("button", { name: "+ Create group" }).click({ force: true });

	expect(calls.filter((call) => call.command.startsWith("group_"))).toHaveLength(0);
});
