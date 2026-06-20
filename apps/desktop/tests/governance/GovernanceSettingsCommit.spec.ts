import { expect, test } from "@playwright/experimental-ct-svelte";
import GovernanceSettingsHarness from "./GovernanceSettingsHarness.svelte";

test("commits pending governance changes and clears pending state", async ({ mount }) => {
	const component = await mount(GovernanceSettingsHarness, {
		props: { pendingCount: 3, pendingCountAfterCommit: 0 },
	});

	await component.getByTestId("governance-commit-button").click();

	await expect(component.getByTestId("governance-commit-count")).toHaveText("1");
	await expect(component.getByTestId("governance-commit-message")).toHaveText(
		"chore: update governance config",
	);
	await expect(component.getByTestId("governance-pending-banner")).toHaveCount(0);
});
