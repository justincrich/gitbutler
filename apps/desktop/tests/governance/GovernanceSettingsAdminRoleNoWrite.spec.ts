import { expect, test } from "@playwright/experimental-ct-svelte";
import GovernanceSettingsHarness from "./GovernanceSettingsHarness.svelte";

test("keeps controls disabled for admin users without backend administration:write", async ({
	mount,
}) => {
	const component = await mount(GovernanceSettingsHarness, {
		props: { pendingCount: 3, hasAdminWrite: false, role: "admin" },
	});

	await expect(component.getByTestId("governance-user-role")).toHaveText("admin");
	await expect(component.getByTestId("governance-read-only-message")).toContainText(
		"administration:write",
	);
	await expect(component.getByTestId("governance-commit-button")).toBeDisabled();
	await expect(component.getByTestId("governance-principals-control")).toBeDisabled();
});
