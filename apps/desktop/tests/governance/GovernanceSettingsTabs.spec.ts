import { expect, test } from "@playwright/experimental-ct-svelte";
import GovernanceSettingsHarness from "./GovernanceSettingsHarness.svelte";

test("renders the four governance tabs", async ({ mount }) => {
	const component = await mount(GovernanceSettingsHarness, {
		props: { pendingCount: 0 },
	});

	await expect(component.getByRole("tab", { name: "Principals" })).toBeVisible();
	await expect(component.getByRole("tab", { name: "Groups" })).toBeVisible();
	await expect(component.getByRole("tab", { name: "Branch Gates" })).toBeVisible();
	await expect(component.getByRole("tab", { name: "Rules" })).toBeVisible();
});

test("renders PrincipalsList in the principals tab", async ({ mount }) => {
	const component = await mount(GovernanceSettingsHarness, {
		props: {
			pendingCount: 0,
			principals: [
				{
					principalId: "settings-agent",
					ownGrants: ["contents:read"],
					inheritedGrants: [],
					groupMemberships: [],
					pending: false,
				},
			],
		},
	});

	await expect(component.getByTestId("principals-list")).toBeVisible();
	await expect(component.getByTestId("principals-list-row-settings-agent")).toContainText(
		"settings-agent",
	);
	await expect(component.getByTestId("governance-principals-control")).toHaveCount(0);
});
