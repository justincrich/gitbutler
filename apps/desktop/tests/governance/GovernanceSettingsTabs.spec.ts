import { expect, test } from "@playwright/experimental-ct-svelte";
import GovernanceSettingsHarness from "./GovernanceSettingsHarness.svelte";

test("renders the five governance tabs", async ({ mount }) => {
	const component = await mount(GovernanceSettingsHarness, {
		props: { pendingCount: 0 },
	});

	await expect(component.getByRole("tab", { name: "Principals" })).toBeVisible();
	await expect(component.getByRole("tab", { name: "Groups" })).toBeVisible();
	await expect(component.getByRole("tab", { name: "Branch Gates" })).toBeVisible();
	await expect(component.getByRole("tab", { name: "Rules" })).toBeVisible();
	await expect(component.getByRole("tab", { name: "Local Review" })).toBeVisible();
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

test("renders GroupsList in the groups tab", async ({ mount }) => {
	const component = await mount(GovernanceSettingsHarness, {
		props: { pendingCount: 0 },
	});

	const principalsTab = component.getByRole("tab", { name: "Principals" });
	const groupsTab = component.getByRole("tab", { name: "Groups" });

	await expect(principalsTab).toHaveAttribute("aria-selected", "true");
	await groupsTab.click();

	await expect(groupsTab).toHaveAttribute("aria-selected", "true");
	await expect(groupsTab).toHaveAttribute("tabindex", "0");
	await expect(principalsTab).toHaveAttribute("aria-selected", "false");
	await expect(component.getByTestId("groups-list")).toBeVisible();
	await expect(component.getByTestId("governance-groups-control")).toHaveCount(0);
});

test("mounts LocalReviewView in the Local Review tab", async ({ mount }) => {
	const component = await mount(GovernanceSettingsHarness, {
		props: { pendingCount: 0 },
	});

	await component.getByRole("tab", { name: "Local Review" }).click();

	await expect(component.getByTestId("governance-local-review-panel")).toBeVisible();
	// With no branch/backend wired in the harness, the view renders its graceful
	// empty state inside its root — proving the orphaned component is now mounted.
	await expect(component.getByTestId("local-review-view")).toBeVisible();
	await expect(component.getByTestId("local-review-empty")).toBeVisible();
});
