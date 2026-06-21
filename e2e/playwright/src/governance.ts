import type { Locator, Page } from "@playwright/test";

export const ADMIN_HANDLE = "admin";
export const NONADMIN_HANDLE = "dev";
export const GOVERNANCE_TARGET_REF = "refs/heads/master";
export const GOVERNANCE_PROTECTED_BRANCH = "master";
export const GOVERNANCE_ADMIN_GROUP = "maintainers";
export const GOVERNANCE_NONADMIN_GROUP = "code-reviewers";
export const GOVERNANCE_ADMIN_AUTHORITIES = ["administration:write", "merge"] as const;
export const GOVERNANCE_NONADMIN_AUTHORITIES = ["contents:write"] as const;
export const GOVERNANCE_BRANCH_GATE = {
	name: GOVERNANCE_PROTECTED_BRANCH,
	protected: true,
} as const;

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

type GovernancePendingToken = {
	authority: string;
	committed: boolean;
	working: boolean;
	pending: boolean;
	change?: "grant" | "revoke";
};

type GovernancePendingPrincipal = {
	id: string;
	committedEffective: string[];
	workingEffective: string[];
	tokens: GovernancePendingToken[];
};

export type GovernancePendingResponse = {
	principals: GovernancePendingPrincipal[];
	pendingCount: number;
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

async function getButlerPort(): Promise<number> {
	const setup = await import("./setup.ts");
	return setup.getButlerPort();
}

async function openWorkspaceFromSetup(page: Page): Promise<void> {
	const setup = await import("./setup.ts");
	await setup.openWorkspace(page);
}

async function playwrightExpect() {
	const { expect } = await import("@playwright/test");
	return expect;
}

export async function seedSignedInUser(role: TestUserRole): Promise<void> {
	const port = await getButlerPort();
	const response = await fetch(`http://localhost:${port}/set_user`, {
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
	const [{ clickByTestId }, expect] = await Promise.all([
		import("./util.ts"),
		playwrightExpect(),
	]);
	await clickByTestId(page, "chrome-sidebar-project-settings-button");
	await expect(page.getByTestId("project-settings-modal")).toBeVisible();
}

export function projectSettingsSidebar(page: Page): Locator {
	return page.getByTestId("project-settings-modal").locator(".settings-sidebar__links");
}

export async function openPermissionsGovernanceSettings(page: Page): Promise<void> {
	const expect = await playwrightExpect();
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
	const expect = await playwrightExpect();
	await page.getByTestId(`principals-list-row-${governanceSlug(principalId)}`).click();
	const editor = page.getByTestId("principal-editor");
	await expect(editor).toBeVisible();
	return editor;
}

export async function openGroup(page: Page, groupName: string): Promise<Locator> {
	const expect = await playwrightExpect();
	const group = page.getByTestId(`groups-list-row-${governanceSlug(groupName)}`);
	await expect(group).toBeVisible();
	await group.click();
	return group;
}

export async function stagePrincipalReviewsGrant(page: Page): Promise<void> {
	const expect = await playwrightExpect();
	const editor = await openPrincipalEditor(page, "test-principal");
	await expect(editor.getByTestId("principal-editor-toggle-contents-write")).toBeDisabled();
	await editor.getByTestId("principal-editor-toggle-reviews-write").check();
	await editor.getByTestId("principal-editor-save").click();
	await expect(page.getByTestId("principals-list-pending-test-principal")).toBeVisible();
	await expect(page.getByTestId("governance-pending-banner")).toContainText("1 pending changes");
}

export async function stageGroupReviewsGrant(page: Page): Promise<void> {
	const expect = await playwrightExpect();
	await openGovernanceTab(page, "Groups");
	const group = await openGroup(page, "test-group");
	const toggle = group.getByTestId("groups-list-toggle-test-group-reviews-write");
	await toggle.check();
	await expect(toggle).toBeChecked();
	await expect(page.getByTestId("groups-list-pending-test-group")).toContainText("Pending");
	await expect(page.getByTestId("governance-pending-banner")).toContainText("2 pending changes");
}

export async function expectActionBlocked(action: () => Promise<unknown>): Promise<void> {
	let blocked = false;
	try {
		await action();
	} catch {
		blocked = true;
	}
	if (!blocked) {
		throw new Error("Expected action to be blocked");
	}
}

export function currentProjectId(page: Page): string {
	const projectId = page.url().split("/")[3];
	if (!projectId) {
		throw new Error(`Could not parse project id from URL ${page.url()}`);
	}
	return projectId;
}

export async function readGovernancePending(page: Page): Promise<GovernancePendingResponse> {
	const port = await getButlerPort();
	const response = await page.request.post(
		`http://localhost:${port}/governance_pending`,
		{
			data: {
				projectId: currentProjectId(page),
				targetRef: "refs/remotes/origin/main",
			},
		},
	);

	if (!response.ok()) {
		throw new Error(`governance_pending returned HTTP ${response.status()}`);
	}

	const payload = (await response.json()) as ButlerResponse<GovernancePendingResponse>;
	if (payload.type !== "success") {
		throw new Error(`governance_pending failed: ${JSON.stringify(payload.subject)}`);
	}
	return payload.subject;
}

export function expectPendingGrant(
	pending: GovernancePendingResponse,
	principalId: string,
	authority: string,
): void {
	const principal = pending.principals.find((entry) => entry.id === principalId);
	if (!principal) {
		throw new Error(`pending principal ${principalId} should exist`);
	}
	if (principal.committedEffective.includes(authority)) {
		throw new Error(`${principalId} must not have committed ${authority}`);
	}
	if (!principal.workingEffective.includes(authority)) {
		throw new Error(`${principalId} must have working ${authority}`);
	}
	const hasGrantToken = principal.tokens.some(
		(token) =>
			token.authority === authority &&
			!token.committed &&
			token.working &&
			token.pending &&
			token.change === "grant",
	);
	if (!hasGrantToken) {
		throw new Error(`${principalId} must have a pending grant token for ${authority}`);
	}
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
