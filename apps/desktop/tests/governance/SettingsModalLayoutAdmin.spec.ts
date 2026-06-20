import SettingsModalLayoutAdminHarness from "./SettingsModalLayoutAdminHarness.svelte";
import { expect, test } from "@playwright/experimental-ct-svelte";

test.describe("ProjectSettingsModalContent admin settings", () => {
	test("shows Permissions & Governance in the sidebar for admin users", async ({ mount }) => {
		const component = await mount(SettingsModalLayoutAdminHarness, {
			props: { role: "admin", selectedId: "governance" },
		});

		await expect(component.getByRole("button", { name: "Project" })).toBeVisible();
		await expect(component.getByRole("button", { name: "AI options" })).toBeVisible();
		await expect(component.getByRole("button", { name: "Permissions & Governance" })).toBeVisible();
	});

	test("hides Permissions & Governance in the sidebar for non-admin users", async ({ mount }) => {
		const component = await mount(SettingsModalLayoutAdminHarness, {
			props: { role: "member", selectedId: "governance" },
		});

		await expect(component.getByRole("button", { name: "Project" })).toBeVisible();
		await expect(component.getByRole("button", { name: "AI options" })).toBeVisible();
		await expect(component.getByRole("button", { name: "Permissions & Governance" })).toHaveCount(
			0,
		);
	});

	test("renders governance settings when the governance page is selected", async ({ mount }) => {
		const component = await mount(SettingsModalLayoutAdminHarness, {
			props: { role: "admin", selectedId: "governance" },
		});

		await expect(
			component.getByRole("heading", { name: "Permissions & Governance" }),
		).toBeVisible();
		await expect(component.getByText("Settings page governance not Found.")).toHaveCount(0);
	});
});
