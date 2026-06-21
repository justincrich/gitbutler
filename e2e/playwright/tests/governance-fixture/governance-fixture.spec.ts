import { expect, test } from "@playwright/test";

async function openGovernance(page: import("@playwright/test").Page) {
	await page.goto("/");
	await expect(page.getByText("Fixture governance harness")).toBeVisible();
	await expect(page.getByText("Not product E2E evidence")).toBeVisible();
	await page.getByRole("button", { name: "admin", exact: true }).click();
	await page.getByRole("button", { name: "Permissions & Governance" }).click();
}

async function expectFixtureEvidenceLabels(
	page: import("@playwright/test").Page,
	labels: string[],
) {
	const evidenceLabels = page.getByLabel("Fixture evidence labels");
	for (const label of labels) {
		await expect(evidenceLabels).toContainText(label);
	}
}

test("admin can see the governance settings entry and all four tabs", async ({ page }) => {
	await openGovernance(page);

	await expectFixtureEvidenceLabels(page, [
		"admin-visible.fixture-evidence",
		"four-tabs.fixture-evidence",
	]);
	await expect(page.getByTestId("fixture-persona")).toHaveText("Admin");
	await expect(page.getByRole("button", { name: "Project" })).toBeVisible();
	await expect(page.getByRole("button", { name: "AI options" })).toBeVisible();
	await expect(page.getByRole("button", { name: "Permissions & Governance" })).toBeVisible();
	await expect(page.getByRole("heading", { name: "Permissions & Governance" })).toBeVisible();
	await expect(page.getByRole("tab", { name: "Principals" })).toBeVisible();
	await expect(page.getByRole("tab", { name: "Groups" })).toBeVisible();
	await expect(page.getByRole("tab", { name: "Branch Gates" })).toBeVisible();
	await expect(page.getByRole("tab", { name: "Rules" })).toBeVisible();
	await expect(page.getByText("Settings page governance not Found.")).toHaveCount(0);
});

test("non-admin cannot see the governance settings entry", async ({ page }) => {
	await page.goto("/");
	await expect(page.getByText("Fixture governance harness")).toBeVisible();

	await page.getByRole("button", { name: "member" }).click();

	await expectFixtureEvidenceLabels(page, ["non-admin-hidden.fixture-evidence"]);
	await expect(page.getByTestId("fixture-persona")).toHaveText("Member");
	await expect(page.getByRole("button", { name: "Project" })).toBeVisible();
	await expect(page.getByRole("button", { name: "AI options" })).toBeVisible();
	await expect(page.getByRole("button", { name: "Permissions & Governance" })).toHaveCount(0);
	await expect(page.getByTestId("governance-settings")).toHaveCount(0);
});

test("read-only admin can view governance settings but cannot edit", async ({ page }) => {
	await page.goto("/");
	await expect(page.getByText("Fixture governance harness")).toBeVisible();

	await page.getByRole("button", { name: "Read-only admin" }).click();
	await page.getByRole("button", { name: "Permissions & Governance" }).click();

	await expectFixtureEvidenceLabels(page, ["read-only-admin.fixture-evidence"]);
	await expect(page.getByTestId("fixture-persona")).toHaveText("Read-only admin");
	await expect(page.getByTestId("governance-read-only-message")).toContainText(
		"administration:write",
	);
	await page.getByTestId("principals-list-row-settings-agent").click();
	await expect(page.getByLabel("reviews:write own grant")).toBeDisabled();
	await expect(page.getByTestId("principal-editor-save")).toBeDisabled();

	await page.getByRole("tab", { name: "Groups" }).click();
	await page.getByTestId("groups-list-row-eng").click();
	await expect(page.getByLabel("reviews:write")).toBeDisabled();

	await page.getByRole("tab", { name: "Branch Gates" }).click();
	await expect(page.getByTestId("governance-branch-gates-control")).toBeDisabled();
	await page.getByRole("tab", { name: "Rules" }).click();
	await expect(page.getByTestId("governance-rules-control")).toBeDisabled();
	await expect(page.getByTestId("governance-commit-button")).toHaveCount(0);
});

test("principal and group changes stay pending across tabs and clear after commit", async ({
	page,
}) => {
	await openGovernance(page);

	await expectFixtureEvidenceLabels(page, [
		"principal-pending.fixture-evidence",
		"group-pending.fixture-evidence",
		"cross-tab-pending.fixture-evidence",
		"post-commit-clean.fixture-evidence",
	]);
	await page.getByTestId("principals-list-row-settings-agent").click();
	await page.getByLabel("reviews:write own grant").check();
	await expect(page.getByLabel("contents:write inherited from group eng")).toBeDisabled();
	await page.getByTestId("principal-editor-save").click();

	await expect(page.getByTestId("principals-list-pending-settings-agent")).toBeVisible();
	await expect(page.getByTestId("governance-pending-banner")).toContainText("1 pending changes");

	await page.getByRole("tab", { name: "Groups" }).click();
	await page.getByTestId("groups-list-row-eng").click();
	await page.getByLabel("reviews:write").check();

	await expect(page.getByTestId("groups-list-pending-eng")).toBeVisible();
	await expect(page.getByTestId("governance-pending-banner")).toContainText("2 pending changes");

	await page.getByRole("tab", { name: "Branch Gates" }).click();
	await expect(page.getByTestId("governance-pending-banner")).toContainText("2 pending changes");
	await page.getByRole("tab", { name: "Rules" }).click();
	await expect(page.getByTestId("governance-pending-banner")).toContainText("2 pending changes");
	await page.getByRole("tab", { name: "Principals" }).click();
	await expect(page.getByTestId("principals-list-pending-settings-agent")).toBeVisible();
	await page.getByRole("tab", { name: "Groups" }).click();
	await expect(page.getByTestId("groups-list-pending-eng")).toBeVisible();

	await page.getByTestId("governance-commit-button").click();

	await expect(page.getByTestId("governance-pending-banner")).toHaveCount(0);
	await expect(page.getByTestId("groups-list-pending-eng")).toHaveCount(0);
	await expect(page.getByTestId("governance-commit-result")).toContainText(
		"chore: update governance config",
	);

	await page.getByRole("tab", { name: "Principals" }).click();
	await expect(page.getByTestId("principals-list-row-settings-agent")).toContainText("reviews:write");
	await page.getByTestId("principals-list-row-settings-agent").click();
	await expect(page.getByLabel("reviews:write own grant")).toBeChecked();
	await expect(page.getByTestId("principals-list-pending-settings-agent")).toHaveCount(0);
	await page.getByRole("tab", { name: "Groups" }).click();
	await expect(page.getByLabel("reviews:write")).toBeChecked();
	await expect(page.getByTestId("groups-list-pending-eng")).toHaveCount(0);
});
