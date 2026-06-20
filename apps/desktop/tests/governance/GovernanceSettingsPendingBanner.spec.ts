import { expect, test } from "@playwright/experimental-ct-svelte";
import GovernanceSettingsHarness from "./GovernanceSettingsHarness.svelte";

test("shows pending count and commit action only when changes are pending", async ({ mount }) => {
	const dirty = await mount(GovernanceSettingsHarness, {
		props: { pendingCount: 3 },
	});

	await expect(dirty.getByTestId("governance-pending-banner")).toContainText("3");
	await expect(dirty.getByTestId("governance-commit-button")).toBeVisible();
	await dirty.unmount();

	const clean = await mount(GovernanceSettingsHarness, {
		props: { pendingCount: 0 },
	});

	await expect(clean.getByTestId("governance-pending-banner")).toHaveCount(0);
});
