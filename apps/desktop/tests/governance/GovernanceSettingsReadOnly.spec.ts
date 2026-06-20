import { expect, test } from "@playwright/experimental-ct-svelte";
import GovernanceSettingsHarness from "./GovernanceSettingsHarness.svelte";

test("uses backend hasAdminWrite=false to explain and disable governance controls", async ({
	mount,
}) => {
	const component = await mount(GovernanceSettingsHarness, {
		props: { pendingCount: 3, hasAdminWrite: false },
	});

	await expect(component.getByTestId("governance-read-only-message")).toContainText(
		"administration:write",
	);
	await expect(component.getByTestId("governance-commit-button")).toBeDisabled();
	await expect(component.getByTestId("principals-list")).toBeVisible();
	await expect(component.getByTestId("governance-principals-control")).toHaveCount(0);
	await component.getByRole("tab", { name: "Groups" }).click();
	await expect(component.getByTestId("governance-groups-panel")).toBeVisible();
	await expect(component.getByTestId("governance-groups-control")).toBeDisabled();
	await expect(component.getByTestId("governance-read-only-message")).toBeVisible();
});
