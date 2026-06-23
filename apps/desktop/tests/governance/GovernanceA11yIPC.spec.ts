import { expect, test } from "@playwright/experimental-ct-svelte";
import PrincipalEditor from "$components/governance/PrincipalEditor.svelte";
import GovernanceSettingsHarness from "./GovernanceSettingsHarness.svelte";
import PrincipalEditorIpcErrorHarness from "./PrincipalEditorIpcErrorHarness.svelte";
import type { PrincipalEditorService } from "$components/governance/PrincipalEditor.svelte";

const targetRef = "refs/remotes/origin/main";
const projectId = "ct-project";

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
	const component = await mount(PrincipalEditorIpcErrorHarness);

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

test("GovernanceIPCFailureBanner: read transport failure renders danger InfoMessage with Retry", async ({
	mount,
}) => {
	const component = await mount(GovernanceSettingsHarness, {
		props: {
			pendingCount: 0,
			readFailure: true,
		},
	});

	const failure = component.getByTestId("governance-read-failure");
	await expect(failure).toBeVisible();
	await expect(failure).toHaveClass(/danger/);
	await expect(failure).toContainText("network.error");
	await expect(failure).toContainText("Backend unreachable");
	await expect(failure).toContainText("Check the desktop backend connection and retry.");
	await expect(component.getByTestId("governance-read-pending-count")).toHaveText("1");

	await component.getByRole("button", { name: "Retry" }).click();

	await expect(component.getByTestId("governance-read-pending-count")).toHaveText("2");
	await expect(failure).toBeVisible();
});

test("GovernanceIPCRetry: retry reissues the failing SDK call and persistent failure stays read-only", async ({
	mount,
}) => {
	const component = await mount(PrincipalEditorIpcErrorHarness);

	await component.getByTestId("principal-editor-toggle-reviews-write").click();
	await expect(component.getByTestId("principal-editor-save")).toBeEnabled();
	await component.getByTestId("principal-editor-save").click();
	await expect(component.getByTestId("principal-editor-denial")).toBeVisible();
	await expect(component.getByTestId("principal-editor-ipc-error-calls")).toHaveText("1");

	await component.getByRole("button", { name: "Retry" }).click();

	await expect(component.getByTestId("principal-editor-ipc-error-calls")).toHaveText("2");
	await expect(component.getByTestId("principal-editor-denial")).toBeVisible();
	await expect(
		component.getByTestId("principal-editor-toggle-administration-write"),
	).toBeDisabled();
	await expect(component.getByTestId("principal-editor-save")).toBeDisabled();
});

test("GovernanceWriteResultFailureBanner: structured SDK result keeps remediation hint", async ({
	mount,
}) => {
	const component = await mount(PrincipalEditorIpcErrorHarness, {
		props: {
			failureMode: "result",
		},
	});

	await component.getByTestId("principal-editor-toggle-reviews-write").click();
	await component.getByTestId("principal-editor-save").click();

	const denial = component.getByTestId("principal-editor-denial");
	await expect(denial).toContainText("perm.denied");
	await expect(denial).toContainText("Governance write denied by branch protection policy");
	await expect(denial).toContainText("administration:write");
});

test("GovernanceSelfEscalationNoFlip: denied administration grant shows banner and leaves toggle off", async ({
	mount,
}) => {
	const serviceCalls: ServiceCall[] = [];
	const component = await mount(PrincipalEditor, {
		props: {
			projectId,
			targetRef,
			principalId: "settings-agent",
			ownGrants: ["contents:read"],
			groupMemberships: [],
			inheritedGrants: [],
			isCurrentUser: true,
			service: selfEscalationDeniedService(serviceCalls),
		},
	});

	const adminToggle = component.getByTestId("principal-editor-toggle-administration-write");
	await expect(adminToggle).not.toBeChecked();

	await adminToggle.click();

	await expect(component.getByTestId("principal-editor-denial")).toContainText(
		"You cannot modify your own administration grants",
	);
	await expect(component.getByTestId("principal-editor-denial")).toContainText(
		"Self-escalation is not permitted.",
	);
	await expect(component.getByRole("button", { name: "Retry" })).toHaveCount(0);
	await expect(adminToggle).not.toBeChecked();
	await expect(component.getByTestId("principal-editor-save")).toBeDisabled();
	expect(serviceCalls).toEqual([
		{
			name: "permGrant",
			args: [projectId, targetRef, "settings-agent", "administration:write"],
		},
	]);
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
	// MGMT-UI-010 replaced the placeholder governance-rules-control button with the
	// real RulesList, which only mounts once a principal is selected. With no
	// principal selected the Rules tab body exposes no write affordance at all, so
	// the read-only invariant holds by absence. Mirror the Principals assertion
	// above: the real panel rendered, the no-principal placeholder is shown, and
	// the legacy placeholder write control is gone.
	await expect(component.getByTestId("governance-rules-panel")).toBeVisible();
	await expect(component.getByTestId("governance-rules-no-principal")).toBeVisible();
	await expect(component.getByTestId("governance-rules-control")).toHaveCount(0);
});

type ServiceCall = {
	name: keyof PrincipalEditorService;
	args: string[];
};

function selfEscalationDeniedService(serviceCalls: ServiceCall[]): PrincipalEditorService {
	return {
		async permGrant(projectId, targetRef, principal, authorities) {
			serviceCalls.push({
				name: "permGrant",
				args: [projectId, targetRef, principal, ...authorities],
			});
			if (authorities.includes("administration:write")) {
				return {
					code: "perm.denied",
					message: "You cannot modify your own administration grants",
					remediation_hint: "Self-escalation is not permitted.",
				};
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
