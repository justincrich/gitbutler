import { expect, test } from "@playwright/experimental-ct-svelte";
import GovernancePendingBanner from "$components/governance/GovernancePendingBanner.svelte";

test.describe("GovernancePendingBanner", () => {
	test("shows warning message with pending count and commit action", async ({ mount }) => {
		const component = await mount(GovernancePendingBanner, {
			props: {
				pendingCount: 4,
				onCommit: () => {},
			},
		});

		await expect(component.getByTestId("governance-pending-banner")).toContainText("4");
		await expect(component.locator(".info-message.warning")).toHaveCount(1);
		await expect(component.getByRole("button", { name: "Commit changes" })).toBeVisible();
	});

	test("renders nothing when there are no pending changes", async ({ mount }) => {
		const clean = await mount(GovernancePendingBanner, {
			props: {
				pendingCount: 0,
				onCommit: () => {},
			},
		});

		await expect(clean.getByTestId("governance-pending-banner")).toHaveCount(0);
		await expect(clean.locator(".info-message.warning")).toHaveCount(0);
		await expect(clean.getByRole("button", { name: "Commit changes" })).toHaveCount(0);

		const dirty = await mount(GovernancePendingBanner, {
			props: {
				pendingCount: 4,
				onCommit: () => {},
			},
		});

		await expect(dirty.getByTestId("governance-pending-banner")).toContainText("4");
	});

	test("delegates commit through the callback exactly once", async ({ mount }) => {
		let commitCount = 0;
		const component = await mount(GovernancePendingBanner, {
			props: {
				pendingCount: 4,
				onCommit: () => {
					commitCount += 1;
				},
			},
		});

		await component.getByRole("button", { name: "Commit changes" }).click();

		expect(commitCount).toBe(1);
	});
});
