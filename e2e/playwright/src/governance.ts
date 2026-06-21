import { getButlerPort, openWorkspace as openWorkspaceFromSetup } from "./setup.ts";
import { clickByTestId } from "./util.ts";
import { expect, type Locator, type Page } from "@playwright/test";

type TestUserRole = "admin" | "member";

type TestUser = {
	id: number;
	name: string;
	login: string;
	email: string;
	picture: string;
	locale: string;
	created_at: string;
	updated_at: string;
	access_token: string;
	role: TestUserRole;
	supporter: boolean;
};

type ButlerResponse<T> =
	| {
			type: "success";
			subject: T;
	  }
	| {
			type: "error";
			subject: unknown;
	  };

const usersByRole: Record<TestUserRole, TestUser> = {
	admin: {
		id: 6101,
		name: "Governance Admin",
		login: "governance-admin",
		email: "governance-admin@example.com",
		picture: "",
		locale: "en-US",
		created_at: "2026-01-01T00:00:00Z",
		updated_at: "2026-01-01T00:00:00Z",
		access_token: "e2e-governance-admin-token",
		role: "admin",
		supporter: false,
	},
	member: {
		id: 6102,
		name: "Governance Member",
		login: "governance-member",
		email: "governance-member@example.com",
		picture: "",
		locale: "en-US",
		created_at: "2026-01-01T00:00:00Z",
		updated_at: "2026-01-01T00:00:00Z",
		access_token: "e2e-governance-member-token",
		role: "member",
		supporter: false,
	},
};

export async function seedSignedInUser(role: TestUserRole): Promise<void> {
	const response = await fetch(`http://localhost:${getButlerPort()}/set_user`, {
		method: "POST",
		headers: { "content-type": "application/json" },
		body: JSON.stringify({ user: usersByRole[role] }),
	});

	if (!response.ok) {
		throw new Error(`set_user returned HTTP ${response.status}`);
	}

	const payload = (await response.json()) as ButlerResponse<unknown>;
	if (payload.type !== "success") {
		throw new Error(`set_user failed: ${JSON.stringify(payload.subject)}`);
	}
}

export async function openProjectSettings(page: Page): Promise<void> {
	await clickByTestId(page, "chrome-sidebar-project-settings-button");
	await expect(page.getByTestId("project-settings-modal")).toBeVisible();
}

export function projectSettingsSidebar(page: Page): Locator {
	return page.getByTestId("project-settings-modal").locator(".settings-sidebar__links");
}

export async function openPermissionsGovernanceSettings(page: Page): Promise<void> {
	await projectSettingsSidebar(page)
		.getByRole("button", { name: "Permissions & Governance", exact: true })
		.click();
	await expect(page.getByTestId("governance-settings")).toBeVisible();
}

export async function openGovernanceProject(page: Page, userRole: TestUserRole): Promise<void> {
	await seedSignedInUser(userRole);
	await openWorkspaceFromSetup(page);
	await openProjectSettings(page);
	await openPermissionsGovernanceSettings(page);
}

export function governanceSlug(value: string): string {
	return value.replace(/[^a-z0-9]+/gi, "-");
}

export async function openGovernanceTab(page: Page, name: string): Promise<void> {
	await page.getByRole("tab", { name, exact: true }).click();
}

export async function openPrincipalEditor(page: Page, principalId: string): Promise<Locator> {
	await page.getByTestId(`principals-list-row-${governanceSlug(principalId)}`).click();
	const editor = page.getByTestId("principal-editor");
	await expect(editor).toBeVisible();
	return editor;
}

export async function openGroup(page: Page, groupName: string): Promise<Locator> {
	const group = page.getByTestId(`groups-list-row-${governanceSlug(groupName)}`);
	await expect(group).toBeVisible();
	await group.click();
	return group;
}

export async function stagePrincipalReviewsGrant(page: Page): Promise<void> {
	const editor = await openPrincipalEditor(page, "test-principal");
	await expect(editor.getByTestId("principal-editor-toggle-contents-write")).toBeDisabled();
	await editor.getByTestId("principal-editor-toggle-reviews-write").check();
	await editor.getByTestId("principal-editor-save").click();
	await expect(page.getByTestId("principals-list-pending-test-principal")).toBeVisible();
	await expect(page.getByTestId("governance-pending-banner")).toContainText("1 pending changes");
}

export async function stageGroupReviewsGrant(page: Page): Promise<void> {
	await openGovernanceTab(page, "Groups");
	const group = await openGroup(page, "test-group");
	const toggle = group.getByTestId("groups-list-toggle-test-group-reviews-write");
	await toggle.check();
	await expect(toggle).toBeChecked();
	await expect(page.getByTestId("governance-pending-banner")).toContainText("2 pending changes");
}

const governanceWriteCommands = [
	"perm_grant",
	"perm_revoke",
	"group_grant",
	"group_revoke",
	"group_add_member",
	"group_remove_member",
	"group_create",
	"group_delete",
	"branch_gates_update",
	"governance_commit",
];

export function collectGovernanceWriteRequests(page: Page): string[] {
	const urls: string[] = [];
	page.on("request", (request) => {
		const url = request.url();
		if (
			request.method() === "POST" &&
			governanceWriteCommands.some((command) => url.endsWith(`/${command}`))
		) {
			urls.push(url);
		}
	});
	return urls;
}
