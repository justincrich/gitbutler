import {
	expectPendingGrant,
	openGovernanceProject,
	openGovernanceTab,
	openGroup,
	openPrincipalEditor,
	readGovernancePending,
	stageGroupReviewsGrant,
	stagePrincipalReviewsGrant,
} from "../src/governance.ts";
import { test } from "../src/test.ts";
import { expect } from "@playwright/test";

test.describe("governance pending edits", () => {
	test.use({ gitbutlerOptions: { env: { BUT_AGENT_HANDLE: "admin" } } });

	test("admin stages principal and group changes before commit", async ({ page, gitbutler }) => {
		await gitbutler.runScript("governance-project-with-origin-main.sh");
		await openGovernanceProject(page, "admin");

		await stagePrincipalReviewsGrant(page);

		const editor = await openPrincipalEditor(page, "test-principal");
		await expect(editor.getByTestId("principal-editor-toggle-contents-write")).toBeDisabled();
		await expect(editor.getByTestId("principal-editor-toggle-reviews-write")).toBeChecked();

		await stageGroupReviewsGrant(page);
		const group = await openGroup(page, "test-group");
		await expect(group.getByTestId("groups-list-toggle-test-group-reviews-write")).toBeChecked();
		await expect(page.getByTestId("groups-list-pending-test-group")).toContainText("Pending");
		await expect(page.getByTestId("governance-pending-banner")).toContainText("2 pending changes");

		const pending = await readGovernancePending(page);
		expectPendingGrant(pending, "test-principal", "reviews:write");
		expectPendingGrant(pending, "group-principal", "reviews:write");

		await openGovernanceTab(page, "Principals");
		await expect(page.getByTestId("principals-list-pending-test-principal")).toBeVisible();
		await expect(page.getByTestId("governance-pending-banner")).toContainText("2 pending changes");
		await expect(page.getByTestId("governance-commit-button")).toBeEnabled();
	});
});
