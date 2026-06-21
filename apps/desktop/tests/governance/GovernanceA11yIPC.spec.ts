import { expect, test } from "@playwright/experimental-ct-svelte";
import PrincipalEditor from "$components/governance/PrincipalEditor.svelte";
import GovernanceSettingsHarness from "./GovernanceSettingsHarness.svelte";
import type { PrincipalEditorService } from "$components/governance/PrincipalEditor.svelte";

const targetRef = "refs/remotes/origin/main";
const projectId = "ct-project";

function structuredDenial() {
	return {
		code: "perm.denied",
		message: "Governance write denied by branch protection policy",
		remediation_hint: "Ask an administrator with administration:write to approve this change.",
	};
}

test("GovernanceTabsA11y: tablist is labelled and supports roving keyboard activation", async ({
	mount,
	page,
}) => {
	const component = await mount(GovernanceSettingsHarness, {
		props: {
			pendingCount: 0,
		},
	});

	const tabList = component.getByRole("tablist", { name: "Governance sections" });
	await expect(tabList).toBeVisible();

	const principalsTab = component.getByRole("tab", { name: "Principals" });
	const groupsTab = component.getByRole("tab", { name: "Groups" });

	await page.keyboard.press("Tab");
	await expect(principalsTab).toBeFocused();
	await expect(principalsTab).toHaveAttribute("aria-selected", "true");

	await page.keyboard.press("ArrowRight");
	await expect(groupsTab).toBeFocused();
	await expect(groupsTab).toHaveAttribute("aria-selected", "false");

	await page.keyboard.press("Enter");
	await expect(groupsTab).toHaveAttribute("aria-selected", "true");
	await expect(component.getByTestId("governance-groups-panel")).toBeVisible();
});

test("GovernanceIPCFailureBanner: structured write denial renders danger InfoMessage with Retry", async ({
	mount,
}) => {
	const { service } = structuredPrincipalDenialService();
	const component = await mount(PrincipalEditor, {
		props: {
			projectId,
			targetRef,
			principalId: "settings-agent",
			ownGrants: ["contents:read"],
			groupMemberships: [],
			inheritedGrants: [],
			service,
		},
	});

	await component.getByTestId("principal-editor-toggle-reviews-write").click();
	await expect(component.getByTestId("principal-editor-save")).toBeEnabled();
	await component.getByTestId("principal-editor-save").click();

	const denial = component.getByTestId("principal-editor-denial");
	await expect(denial).toBeVisible();
	await expect(denial).toHaveClass(/danger/);
	await expect(denial).toContainText("perm.denied");
	await expect(denial).toContainText("Governance write denied by branch protection policy");
	await expect(denial).toContainText("administration:write");
	await expect(component.getByRole("button", { name: "Retry" })).toBeVisible();
});

test("GovernanceIPCRetry: retry reissues the failing SDK call and persistent failure stays read-only", async ({
	mount,
}) => {
	const { calls, service } = structuredPrincipalDenialService();
	const component = await mount(PrincipalEditor, {
		props: {
			projectId,
			targetRef,
			principalId: "settings-agent",
			ownGrants: ["contents:read"],
			groupMemberships: [],
			inheritedGrants: [],
			service,
		},
	});

	await component.getByTestId("principal-editor-toggle-reviews-write").click();
	await expect(component.getByTestId("principal-editor-save")).toBeEnabled();
	await component.getByTestId("principal-editor-save").click();
	await expect(component.getByTestId("principal-editor-denial")).toBeVisible();
	expect(calls).toHaveLength(1);

	await component.getByRole("button", { name: "Retry" }).click();

	expect(calls).toHaveLength(2);
	await expect(component.getByTestId("principal-editor-denial")).toBeVisible();
	await expect(
		component.getByTestId("principal-editor-toggle-administration-write"),
	).toBeDisabled();
	await expect(component.getByTestId("principal-editor-save")).toBeDisabled();
});

test("GovernanceSelfEscalationNoFlip: denied administration grant shows banner and leaves toggle off", async ({
	mount,
}) => {
	const component = await mount(PrincipalEditor, {
		props: {
			projectId,
			targetRef,
			principalId: "settings-agent",
			ownGrants: ["contents:read"],
			groupMemberships: [],
			inheritedGrants: [],
			isCurrentUser: true,
			service: selfEscalationDeniedService(),
		},
	});

	const adminToggle = component.getByTestId("principal-editor-toggle-administration-write");
	await expect(adminToggle).not.toBeChecked();

	await adminToggle.click();

	await expect(component.getByTestId("principal-editor-denial")).toContainText(
		"You cannot modify your own administration grants",
	);
	await expect(adminToggle).not.toBeChecked();
});

test("GovernanceReadOnlyA11y: missing administration:write explains and disables write controls", async ({
	mount,
}) => {
	const component = await mount(GovernanceSettingsHarness, {
		props: {
			pendingCount: 2,
			hasAdminWrite: false,
		},
	});

	const readOnlyMessage = component.getByTestId("governance-read-only-message");
	await expect(readOnlyMessage).toBeVisible();
	await expect(readOnlyMessage).toHaveClass(/info/);
	await expect(readOnlyMessage).toContainText("administration:write");

	await expect(component.getByTestId("governance-commit-button")).toBeDisabled();
	await expect(component.getByTestId("principals-list")).toBeVisible();
	await expect(component.getByTestId("governance-principals-control")).toHaveCount(0);

	await component.getByRole("tab", { name: "Groups" }).click();
	await expect(component.getByRole("button", { name: "+ Create group" }).first()).toBeDisabled();

	await component.getByRole("tab", { name: "Branch Gates" }).click();
	await expect(component.getByTestId("governance-branch-gates-control")).toBeDisabled();

	await component.getByRole("tab", { name: "Rules" }).click();
	await expect(component.getByTestId("governance-rules-control")).toBeDisabled();
});

function structuredPrincipalDenialService(): { calls: string[]; service: PrincipalEditorService } {
	const calls: string[] = [];

	return {
		calls,
		service: {
			deniedCode: JSON.stringify(structuredDenial()),
			async permGrant(_projectId, _targetRef, _principal, authorities) {
				calls.push("perm_grant");
				return {
					authorities,
					caveat: "",
					principal: "settings-agent",
				};
			},
			async permRevoke(_projectId, _targetRef, _principal, authorities) {
				calls.push("perm_revoke");
				return {
					authorities,
					caveat: "",
					principal: "settings-agent",
				};
			},
			async groupAddMember(_projectId, _targetRef, group, member) {
				calls.push("group_add_member");
				return { authorities: [], caveat: "", group, member };
			},
			async groupRemoveMember(_projectId, _targetRef, group, member) {
				calls.push("group_remove_member");
				return { authorities: [], caveat: "", group, member };
			},
		},
	};
}

function selfEscalationDeniedService(): PrincipalEditorService {
	return {
		async permGrant(_projectId, _targetRef, _principal, authorities) {
			if (authorities.includes("administration:write")) {
				throw Object.assign(new Error("You cannot modify your own administration grants"), {
					code: "perm.denied",
				});
			}
			return {
				authorities,
				caveat: "",
				principal: "settings-agent",
			};
		},
		async permRevoke(_projectId, _targetRef, _principal, authorities) {
			return {
				authorities,
				caveat: "",
				principal: "settings-agent",
			};
		},
		async groupAddMember(_projectId, _targetRef, group, member) {
			return { authorities: [], caveat: "", group, member };
		},
		async groupRemoveMember(_projectId, _targetRef, group, member) {
			return { authorities: [], caveat: "", group, member };
		},
	};
}
