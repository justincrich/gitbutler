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

	test("supplies projectId to governance settings and reads access and pending state", async ({
		mount,
		page,
	}) => {
		const component = await mount(SettingsModalLayoutAdminHarness, {
			props: { role: "admin", selectedId: "governance" },
		});

		await expect
			.poll(async () => {
				return await page.evaluate(() => JSON.stringify(globalThis.__governanceBackendCalls));
			})
			.toContain("governance_status_read");
		await expect(component.getByTestId("governance-backend-calls")).toContainText("ct-project");
		await expect
			.poll(async () => {
				return await page.evaluate(() => JSON.stringify(globalThis.__governanceBackendCalls));
			})
			.toContain("governance_pending");
		await expect
			.poll(async () => {
				return await page.evaluate(() => JSON.stringify(globalThis.__governanceBackendCalls));
			})
			.toContain('"targetRef":"refs/remotes/origin/main"');
	});
});
