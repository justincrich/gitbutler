import { expect, test } from "@playwright/experimental-ct-svelte";
import * as fs from "node:fs";
import * as path from "node:path";
import GovernanceBoundaryHarness from "./GovernanceErrorBoundaryHarness.svelte";

test.describe("GovernanceErrorBoundary", () => {
	test("renders shared ErrorBoundary fallback for a governance child throw while keeping the settings modal mounted", async ({
		mount,
	}) => {
		const component = await mount(GovernanceBoundaryHarness, {
			props: { mode: "throwing-principals" },
		});

		await expect(component.getByRole("heading", { name: "Project settings" })).toBeVisible();
		await expect(component.locator(".boundary-error")).toHaveCount(1);
		await expect(component.locator(".boundary-error")).toContainText(
			"Governance settings failed to load",
		);
	});

	test("does not add a governance-specific ErrorBoundary component", async () => {
		const srcRoot = path.resolve(process.cwd(), "src");
		const matches = fs
			.readdirSync(srcRoot, { recursive: true, withFileTypes: true })
			.filter((entry) => entry.isFile() && entry.name === "GovernanceErrorBoundary.svelte");

		expect(matches).toHaveLength(0);
	});

	test("renders normal governance settings with no boundary fallback and exactly five tabs", async ({
		mount,
	}) => {
		const component = await mount(GovernanceBoundaryHarness, {
			props: { mode: "normal" },
		});

		await expect(component.locator(".boundary-error")).toHaveCount(0);
		await expect(component.getByRole("tab")).toHaveCount(5);
		await expect(component.getByRole("tab", { name: "Principals" })).toBeVisible();
		await expect(component.getByRole("tab", { name: "Groups" })).toBeVisible();
		await expect(component.getByRole("tab", { name: "Branch Gates" })).toBeVisible();
		await expect(component.getByRole("tab", { name: "Rules" })).toBeVisible();
		await expect(component.getByRole("tab", { name: "Local Review" })).toBeVisible();
	});
});
