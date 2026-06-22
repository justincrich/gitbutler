import {
	collectGovernanceWriteRequests,
	expectActionBlocked,
	openGovernanceProject,
	openGovernanceTab,
	openGroup,
	openPrincipalEditor,
} from "../src/governance.ts";
import { test } from "../src/test.ts";
import { expect } from "@playwright/test";

test.describe("governance tabs and read-only admin", () => {
	test.use({ gitbutlerOptions: { env: { BUT_AGENT_HANDLE: "admin-readonly" } } });

	test("read-only app admin can view all governance tabs but cannot write", async ({
		page,
		gitbutler,
	}) => {
		await gitbutler.runScript("governance-project-with-origin-main.sh");
		const writeRequests = collectGovernanceWriteRequests(page);

		await openGovernanceProject(page, "admin");

		const tabs = page.getByRole("tab");
		await expect(tabs).toHaveText(["Principals", "Groups", "Branch Gates", "Rules"]);
		await expect(page.getByText("Settings page governance not Found.")).toHaveCount(0);
		await expect(page.getByTestId("governance-read-only-message")).toContainText(
			"administration:write",
		);

		const editor = await openPrincipalEditor(page, "test-principal");
		const principalReviewsToggle = editor.getByTestId("principal-editor-toggle-reviews-write");
		const principalAdminToggle = editor.getByTestId("principal-editor-toggle-administration-write");
		const principalSave = editor.getByTestId("principal-editor-save");
		await expect(principalReviewsToggle).toBeDisabled();
		await expect(principalAdminToggle).toBeDisabled();
		await expect(principalSave).toBeDisabled();
		await expectActionBlocked(() => principalReviewsToggle.click({ timeout: 500 }));
		await principalAdminToggle.click({ force: true });
		await principalSave.click({ force: true });
		await expect(principalReviewsToggle).not.toBeChecked();
		await expect(principalAdminToggle).not.toBeChecked();
		await expect(page.getByTestId("governance-pending-banner")).toHaveCount(0);

		await openGovernanceTab(page, "Groups");
		const group = await openGroup(page, "test-group");
		const groupReviewsToggle = group.getByTestId("groups-list-toggle-test-group-reviews-write");
		const groupCreateName = page.getByTestId("groups-list-create-name");
		const groupDelete = group.getByTestId("groups-list-delete-test-group");
		await expect(groupReviewsToggle).toBeDisabled();
		await expect(groupCreateName).toBeDisabled();
		await expect(groupDelete).toBeDisabled();
		await expectActionBlocked(() => groupReviewsToggle.click({ timeout: 500 }));
		await expectActionBlocked(() => groupCreateName.fill("blocked-group", { timeout: 500 }));
		await groupDelete.click({ force: true });
		await expect(groupReviewsToggle).not.toBeChecked();
		await expect(page.getByTestId("groups-list-delete-modal")).toHaveCount(0);

		await openGovernanceTab(page, "Branch Gates");
		await expect(page.getByTestId("governance-branch-gates-panel")).toBeVisible();
		await expect(page.getByTestId("governance-branch-gates-control")).toBeDisabled();

		await openGovernanceTab(page, "Rules");
		await expect(page.getByTestId("governance-rules-panel")).toBeVisible();
		await expect(page.getByTestId("governance-rules-control")).toBeDisabled();
		await expect(page.getByTestId("governance-commit-button")).toHaveCount(0);
		await page.waitForTimeout(100);
		expect(writeRequests).toEqual([]);
	});
});
