import PrincipalsList from "$components/governance/PrincipalsList.svelte";
import { expect, test } from "@playwright/experimental-ct-svelte";
import type { PrincipalEditorService } from "$components/governance/PrincipalEditor.svelte";
import type { PrincipalsListEntry } from "$components/governance/PrincipalsList.svelte";
import type { GrantOutcome, GroupWriteOutcome, PermWriteOutcome } from "@gitbutler/but-sdk";

const projectId = "project-1";
const targetRef = "refs/remotes/origin/main";

const seededPrincipals: PrincipalsListEntry[] = [
	{
		principalId: "codex-agent",
		ownGrants: ["contents:read"],
		inheritedGrants: [{ authority: "contents:write", sourceLabel: "group: eng" }],
		groupMemberships: ["eng"],
		pending: false,
	},
	{
		principalId: "claude-agent",
		ownGrants: ["pull_requests:write"],
		inheritedGrants: [],
		groupMemberships: [],
		pending: false,
	},
	{
		principalId: "cursor-bot",
		ownGrants: ["contents:read"],
		inheritedGrants: [{ authority: "contents:write", sourceLabel: "group: eng" }],
		groupMemberships: ["eng"],
		pending: true,
	},
];

function createEditorService(): PrincipalEditorService {
	return {
		async permGrant(projectId, targetRef, principal, authorities): Promise<GrantOutcome> {
			return { principal, authorities, caveat: targetRef };
		},
		async permRevoke(projectId, targetRef, principal, authorities): Promise<PermWriteOutcome> {
			return { principal, authorities, caveat: targetRef };
		},
		async groupAddMember(projectId, targetRef, group, member): Promise<GroupWriteOutcome> {
			return { group, member, authorities: [], caveat: targetRef };
		},
		async groupRemoveMember(projectId, targetRef, group, member): Promise<GroupWriteOutcome> {
			return { group, member, authorities: [], caveat: targetRef };
		},
	};
}

function baseProps(entries = seededPrincipals) {
	return {
		projectId,
		targetRef,
		principals: entries,
		editorService: createEditorService(),
		availableGroups: ["eng", "platform"],
	};
}

test("PrincipalsListRows renders seeded principals and grant sources", async ({ mount }) => {
	const component = await mount(PrincipalsList, {
		props: baseProps(),
	});

	await expect(component.getByTestId("principals-list-row")).toHaveCount(3);
	await expect(component.getByTestId("principals-list-row-codex-agent")).toContainText(
		"codex-agent",
	);
	await expect(component.getByTestId("principals-list-row-codex-agent")).toContainText(
		"contents:write",
	);
	await expect(component.getByTestId("principals-list-row-codex-agent")).toContainText(
		"group: eng",
	);
	await expect(component.getByTestId("principals-list-row-claude-agent")).toContainText(
		"pull_requests:write",
	);
	await expect(component.getByTestId("principals-list-row-claude-agent")).toContainText(
		"own grant",
	);
});

test("PrincipalsListEditorToggle opens PrincipalEditor inline without navigation", async ({
	mount,
	page,
}) => {
	const component = await mount(PrincipalsList, {
		props: baseProps(),
	});
	const beforeUrl = page.url();

	await component.getByTestId("principals-list-row-claude-agent").click();

	await expect(component.getByTestId("principal-editor")).toBeVisible();
	await expect(component.getByTestId("principal-editor")).toContainText("claude-agent");
	expect(page.url()).toBe(beforeUrl);
});

test("PrincipalsListPending marks pending row and keeps effective grants committed", async ({
	mount,
}) => {
	const component = await mount(PrincipalsList, {
		props: baseProps(),
	});

	await expect(component.getByTestId("principals-list-row-cursor-bot")).toContainText("○");
	await expect(component.getByTestId("principals-list-row-cursor-bot")).toContainText(
		"contents:read",
	);
	await expect(component.getByTestId("principals-list-row-cursor-bot")).toContainText(
		"contents:write",
	);
	await expect(component.getByTestId("principals-list-row-cursor-bot")).not.toContainText(
		"working-tree draft",
	);
});

test("PrincipalsListEmpty renders empty state and add action", async ({ mount }) => {
	const component = await mount(PrincipalsList, {
		props: baseProps([]),
	});

	await expect(component.getByTestId("principals-list-empty")).toContainText(
		"No principals configured",
	);
	await expect(component.getByRole("button", { name: "+ Add first" })).toBeVisible();
});
