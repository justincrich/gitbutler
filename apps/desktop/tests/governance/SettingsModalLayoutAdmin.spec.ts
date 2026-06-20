import ProjectSettingsModalContent from "$components/settings/ProjectSettingsModalContent.svelte";
import type { ProjectSettingsModalState } from "$lib/state/uiState.svelte";
import { expect, test, type Page } from "@playwright/experimental-ct-svelte";

type UserRole = "admin" | "member";

async function provideUserService(page: Page, role: UserRole) {
	await page.evaluate(async (userRole) => {
		const { USER_SERVICE } = await import("/src/lib/user/userService.svelte.ts");
		const userService = {
			user: {
				id: 1,
				name: "Settings Tester",
				email: "settings@example.com",
				locale: "en-US",
				created_at: "2026-06-20T00:00:00.000Z",
				updated_at: "2026-06-20T00:00:00.000Z",
				access_token: "desktop-ct-token",
				role: userRole,
				supporter: false,
			},
		};

		window.__pw_hooks_before_mount.push(({ App }) => {
			return new App({
				context: new Map([[USER_SERVICE._key, userService]]),
			});
		});
	}, role);
}

function projectSettingsData(
	selectedId?: ProjectSettingsModalState["selectedId"],
): ProjectSettingsModalState {
	return {
		type: "project-settings",
		projectId: "desktop-ct-project",
		selectedId,
	};
}

test.describe("ProjectSettingsModalContent admin settings", () => {
	test("shows Permissions & Governance in the sidebar for admin users", async ({ mount, page }) => {
		await provideUserService(page, "admin");

		const component = await mount(ProjectSettingsModalContent, {
			props: { data: projectSettingsData() },
		});

		await expect(component.getByRole("button", { name: "Project" })).toBeVisible();
		await expect(component.getByRole("button", { name: "AI options" })).toBeVisible();
		await expect(component.getByRole("button", { name: "Permissions & Governance" })).toBeVisible();
	});

	test("hides Permissions & Governance in the sidebar for non-admin users", async ({
		mount,
		page,
	}) => {
		await provideUserService(page, "member");

		const component = await mount(ProjectSettingsModalContent, {
			props: { data: projectSettingsData() },
		});

		await expect(component.getByRole("button", { name: "Project" })).toBeVisible();
		await expect(component.getByRole("button", { name: "AI options" })).toBeVisible();
		await expect(component.getByRole("button", { name: "Permissions & Governance" })).toHaveCount(
			0,
		);
	});

	test("renders governance settings when the governance page is selected", async ({
		mount,
		page,
	}) => {
		await provideUserService(page, "admin");

		const component = await mount(ProjectSettingsModalContent, {
			props: { data: projectSettingsData("governance") },
		});

		await expect(
			component.getByRole("heading", { name: "Permissions & Governance" }),
		).toBeVisible();
		await expect(component.getByText("Settings page governance not Found.")).toHaveCount(0);
	});
});
