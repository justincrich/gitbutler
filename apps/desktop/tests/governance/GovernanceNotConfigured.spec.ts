import GovernanceSettingsHarness from "./GovernanceSettingsHarness.svelte";
import { expect, test } from "@playwright/experimental-ct-svelte";

test.describe("GovernanceSettings (not configured)", () => {
	test("renders informative setup guidance instead of the management tabs", async ({ mount }) => {
		const component = await mount(GovernanceSettingsHarness, {
			props: { pendingCount: 0, notConfigured: true },
		});

		const emptyState = component.getByTestId("governance-not-configured");
		await expect(emptyState).toBeVisible();
		// Names the problem, the two config files, and the resolved target ref so the user
		// knows exactly what to add and where it's read from.
		await expect(emptyState).toContainText("Governance isn't set up yet");
		await expect(emptyState).toContainText(".gitbutler/permissions.toml");
		await expect(emptyState).toContainText(".gitbutler/gates.toml");
		await expect(emptyState).toContainText("refs/remotes/origin/main");

		// The not-configured state replaces the tabs — there's nothing to manage yet.
		await expect(component.getByTestId("governance-principals-panel")).toHaveCount(0);
		await expect(component.getByTestId("governance-pending-banner")).toHaveCount(0);
	});

	test("setup guide button opens the governance setup docs", async ({ mount }) => {
		const component = await mount(GovernanceSettingsHarness, {
			props: { pendingCount: 0, notConfigured: true },
		});

		const guide = component.getByTestId("governance-setup-guide-link");
		await expect(guide).toBeVisible();
		await guide.click();

		await expect(component.getByTestId("governance-opened-url")).toContainText(
			"docs/governance-setup.md",
		);
	});

	test("a configured project still renders the management tabs", async ({ mount }) => {
		const component = await mount(GovernanceSettingsHarness, {
			props: { pendingCount: 0, notConfigured: false },
		});

		await expect(component.getByTestId("governance-not-configured")).toHaveCount(0);
		await expect(component.getByTestId("governance-principals-panel")).toBeVisible();
	});
});
