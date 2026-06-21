import {
	openGovernanceProject,
	openGovernanceTab,
	openGroup,
	openPrincipalEditor,
	stageGroupReviewsGrant,
	stagePrincipalReviewsGrant,
} from "../src/governance.ts";
import { test } from "../src/test.ts";
import { expect } from "@playwright/test";

test.describe("governance commit flow", () => {
	test.use({ gitbutlerOptions: { env: { BUT_AGENT_HANDLE: "admin" } } });

	test("pending governance changes persist across tabs and clear after commit", async ({
		page,
		gitbutler,
	}) => {
		await gitbutler.runScript("governance-project-with-origin-main.sh");
		await openGovernanceProject(page, "admin");

		await stagePrincipalReviewsGrant(page);
		await stageGroupReviewsGrant(page);

		await openGovernanceTab(page, "Branch Gates");
		await expect(page.getByTestId("governance-branch-gates-panel")).toBeVisible();
		await expect(page.getByTestId("governance-pending-banner")).toContainText("2 pending changes");

		await openGovernanceTab(page, "Rules");
		await expect(page.getByTestId("governance-rules-panel")).toBeVisible();
		await expect(page.getByTestId("governance-pending-banner")).toContainText("2 pending changes");

		await openGovernanceTab(page, "Principals");
		await expect(page.getByTestId("principals-list-pending-test-principal")).toBeVisible();
		await expect(page.getByTestId("governance-pending-banner")).toContainText("2 pending changes");

		await openGovernanceTab(page, "Groups");
		const pendingGroup = await openGroup(page, "test-group");
		await expect(
			pendingGroup.getByTestId("groups-list-toggle-test-group-reviews-write"),
		).toBeChecked();
		await expect(page.getByTestId("governance-pending-banner")).toContainText("2 pending changes");

		await page.getByTestId("governance-commit-button").click();
		await expect(page.getByTestId("governance-pending-banner")).toHaveCount(0);

		await openGovernanceTab(page, "Principals");
		await expect(page.getByTestId("principals-list-pending-test-principal")).toHaveCount(0);
		const editor = await openPrincipalEditor(page, "test-principal");
		await expect(editor.getByTestId("principal-editor-toggle-reviews-write")).toBeChecked();

		await openGovernanceTab(page, "Groups");
		const committedGroup = await openGroup(page, "test-group");
		await expect(
			committedGroup.getByTestId("groups-list-toggle-test-group-reviews-write"),
		).toBeChecked();
	});
});
