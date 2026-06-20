import SettingsModalLayoutAdminHarness from "./SettingsModalLayoutAdminHarness.svelte";
import { expect, test } from "@playwright/experimental-ct-svelte";

test.describe("ProjectSettingsModalContentGovernance", () => {
	test("renders governance settings when selected by an admin user", async ({ mount }) => {
		const component = await mount(SettingsModalLayoutAdminHarness, {
			props: { role: "admin", selectedId: "governance" },
		});

		await expect(
			component.getByRole("heading", { name: "Permissions & Governance" }),
		).toBeVisible();
		await expect(component.getByText("Settings page governance not Found.")).toHaveCount(0);
	});
});
