import PrincipalEditor from "$components/governance/PrincipalEditor.svelte";
import { expect, test } from "@playwright/experimental-ct-svelte";
import type { GrantOutcome, GroupWriteOutcome, PermWriteOutcome } from "@gitbutler/but-sdk";

type PrincipalEditorService = {
	deniedCode?: string;
	permGrant: (
		projectId: string,
		targetRef: string,
		principal: string,
		authorities: string[],
	) => Promise<GrantOutcome | { code: string }>;
	permRevoke: (
		projectId: string,
		targetRef: string,
		principal: string,
		authorities: string[],
	) => Promise<PermWriteOutcome>;
	groupAddMember: (
		projectId: string,
		targetRef: string,
		group: string,
		member: string,
	) => Promise<GroupWriteOutcome>;
	groupRemoveMember: (
		projectId: string,
		targetRef: string,
		group: string,
		member: string,
	) => Promise<GroupWriteOutcome>;
};

type ServiceCall = {
	name: keyof PrincipalEditorService;
	args: string[];
};

const projectId = "project-1";
const targetRef = "refs/remotes/origin/main";
const principalId = "codex-agent";

function createService(calls: ServiceCall[], denied = false): PrincipalEditorService {
	return {
		deniedCode: denied ? "perm.denied" : undefined,
		async permGrant(projectId, targetRef, principal, authorities) {
			calls.push({ name: "permGrant", args: [projectId, targetRef, principal, ...authorities] });

			return { principal, authorities, caveat: targetRef };
		},
		async permRevoke(projectId, targetRef, principal, authorities) {
			calls.push({ name: "permRevoke", args: [projectId, targetRef, principal, ...authorities] });
			return { principal, authorities, caveat: targetRef };
		},
		async groupAddMember(projectId, targetRef, group, member) {
			calls.push({ name: "groupAddMember", args: [projectId, targetRef, group, member] });
			return { group, member, authorities: [], caveat: targetRef };
		},
		async groupRemoveMember(projectId, targetRef, group, member) {
			calls.push({ name: "groupRemoveMember", args: [projectId, targetRef, group, member] });
			return { group, member, authorities: [], caveat: targetRef };
		},
	};
}

function baseProps(service: PrincipalEditorService) {
	return {
		projectId,
		targetRef,
		principalId,
		ownGrants: ["contents:read"],
		inheritedGrants: [{ authority: "contents:write", sourceLabel: "group: eng" }],
		groupMemberships: ["eng"],
		availableGroups: ["eng", "platform"],
		service,
	};
}

test("PrincipalEditorInheritedReadOnly", async ({ mount }) => {
	const calls: ServiceCall[] = [];
	const component = await mount(PrincipalEditor, {
		props: baseProps(createService(calls)),
	});

	const inheritedRow = component.getByTestId("principal-editor-row-contents-write");
	await expect(inheritedRow).toContainText("contents:write");
	await expect(inheritedRow).toContainText("group: eng");
	await expect(component.getByTestId("principal-editor-toggle-contents-write")).toBeDisabled();
	await expect(component.getByTestId("principal-editor-toggle-contents-write")).toBeChecked();
});

test("PrincipalEditorLocalState", async ({ mount }) => {
	const calls: ServiceCall[] = [];
	const component = await mount(PrincipalEditor, {
		props: baseProps(createService(calls)),
	});

	await expect(component.getByTestId("principal-editor-save")).toBeDisabled();
	await component.getByTestId("principal-editor-toggle-reviews-write").click();

	await expect(component.getByTestId("principal-editor-toggle-reviews-write")).toBeChecked();
	await expect(component.getByTestId("principal-editor-save")).toBeEnabled();
	expect(calls).toEqual([]);
});

test("PrincipalEditorBatchSave", async ({ mount }) => {
	const calls: ServiceCall[] = [];
	const component = await mount(PrincipalEditor, {
		props: baseProps(createService(calls)),
	});

	await component.getByTestId("principal-editor-toggle-reviews-write").click();
	await component.getByTestId("principal-editor-save").click();

	await expect(component.getByTestId("principal-editor-save")).toBeDisabled();
	expect(calls).toEqual([
		{
			name: "permGrant",
			args: [projectId, targetRef, principalId, "reviews:write"],
		},
	]);
});

test("PrincipalEditorPreset", async ({ mount }) => {
	const calls: ServiceCall[] = [];
	const component = await mount(PrincipalEditor, {
		props: baseProps(createService(calls)),
	});

	await component.getByRole("tab", { name: "Write" }).click();

	await expect(component.getByTestId("principal-editor-toggle-contents-read")).toBeChecked();
	await expect(component.getByTestId("principal-editor-toggle-reviews-write")).toBeChecked();
	await expect(component.getByTestId("principal-editor-toggle-contents-write")).toBeDisabled();
	await expect(component.getByTestId("principal-editor-toggle-contents-write")).toBeChecked();
	expect(calls).toEqual([]);
});

test("PrincipalEditorGroupChip", async ({ mount }) => {
	const calls: ServiceCall[] = [];
	const component = await mount(PrincipalEditor, {
		props: baseProps(createService(calls)),
	});

	await component.getByTestId("principal-editor-groups").fill("platform");
	await component.getByTestId("principal-editor-groups").press(" ");

	await expect(component.getByText("platform")).toBeVisible();
	expect(calls).toEqual([]);

	await component.getByTestId("principal-editor-save").click();

	expect(calls).toEqual([
		{
			name: "groupAddMember",
			args: [projectId, targetRef, "platform", principalId],
		},
	]);
});

test("PrincipalEditorSelfEscalation", async ({ mount }) => {
	const calls: ServiceCall[] = [];
	const component = await mount(PrincipalEditor, {
		props: {
			...baseProps(createService(calls, true)),
			isCurrentUser: true,
		},
	});

	await component.getByTestId("principal-editor-toggle-administration-write").click();
	await expect(component.getByTestId("principal-editor-toggle-administration-write")).toBeChecked();

	await component.getByTestId("principal-editor-save").click();

	await expect(
		component.getByTestId("principal-editor-toggle-administration-write"),
	).not.toBeChecked();
	await expect(component.getByTestId("principal-editor-denial")).toContainText("perm.denied");
	expect(calls).toEqual([
		{
			name: "permGrant",
			args: [projectId, targetRef, principalId, "administration:write"],
		},
	]);
});
