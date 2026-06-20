import GovernanceSettings from "$components/governance/GovernanceSettings.svelte";
import { projectSettingsPages } from "$lib/settings/projectSettingsPages";
import { expect, test } from "@playwright/experimental-ct-svelte";

test.describe("GovernanceSettings", () => {
	test("mounts the governance settings scaffold", async ({ mount }) => {
		const component = await mount(GovernanceSettings);

		await expect(
			component.getByRole("heading", { name: "Permissions & Governance" }),
		).toBeVisible();
	});

	test("registers one admin-only governance project settings page", () => {
		const governancePages = projectSettingsPages.filter((page) => page.id === "governance");

		expect(governancePages).toEqual([
			{
				id: "governance",
				label: "Permissions & Governance",
				icon: "lock",
				adminOnly: true,
			},
		]);
	});
});
