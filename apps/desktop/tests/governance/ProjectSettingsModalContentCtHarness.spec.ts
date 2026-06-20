import ProjectSettingsModalContentCtHarness from "./ProjectSettingsModalContentCtHarness.svelte";
import { expect, test } from "@playwright/experimental-ct-svelte";

test.describe("ProjectSettingsModalContent CT harness", () => {
	test("bundles the project settings modal import chain and renders the governance scaffold", async ({
		mount,
	}) => {
		const component = await mount(ProjectSettingsModalContentCtHarness);

		await expect(component.getByRole("heading", { name: "Project settings" })).toBeVisible();
		await expect(
			component.getByTestId("governance-proof").getByRole("heading", {
				name: "Permissions & Governance",
			}),
		).toBeVisible();
	});
});
