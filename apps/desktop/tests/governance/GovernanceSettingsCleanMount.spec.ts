import { expect, test } from "@playwright/experimental-ct-svelte";
import GovernanceSettingsHarness from "./GovernanceSettingsHarness.svelte";

test("does not render the pending banner for a seeded clean mount", async ({ mount }) => {
	const component = await mount(GovernanceSettingsHarness, {
		props: { pendingCount: 0 },
	});

	await expect(component.getByRole("tab", { name: "Principals" })).toBeVisible();
	await expect(component.getByTestId("governance-pending-banner")).toHaveCount(0);
});
