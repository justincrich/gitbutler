import { getButlerPort } from "./setup.ts";
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
