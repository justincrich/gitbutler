import { expect, test } from "@playwright/experimental-ct-svelte";
import GovernanceSettingsHarness from "./GovernanceSettingsHarness.svelte";

test("preserves pending count when switching tabs", async ({ mount }) => {
	const component = await mount(GovernanceSettingsHarness, {
		props: { pendingCount: 3 },
	});

	await expect(component.getByTestId("governance-pending-banner")).toContainText("3");
	await component.getByRole("tab", { name: "Groups" }).click();
	await expect(component.getByTestId("governance-pending-banner")).toContainText("3");
	await expect(component.getByTestId("governance-groups-panel")).toBeVisible();
});
