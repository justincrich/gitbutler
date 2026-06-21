import {
	collectGovernanceWriteRequests,
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
		await expect(editor.getByTestId("principal-editor-toggle-reviews-write")).toBeDisabled();
		await expect(editor.getByTestId("principal-editor-toggle-administration-write")).toBeDisabled();
		await expect(editor.getByTestId("principal-editor-save")).toBeDisabled();

		await openGovernanceTab(page, "Groups");
		const group = await openGroup(page, "test-group");
		await expect(group.getByTestId("groups-list-toggle-test-group-reviews-write")).toBeDisabled();
		await expect(page.getByTestId("groups-list-create-name")).toBeDisabled();
		await expect(group.getByTestId("groups-list-delete-test-group")).toBeDisabled();

		await openGovernanceTab(page, "Branch Gates");
		await expect(page.getByTestId("governance-branch-gates-panel")).toBeVisible();
		await expect(page.getByTestId("governance-branch-gates-control")).toBeDisabled();

		await openGovernanceTab(page, "Rules");
		await expect(page.getByTestId("governance-rules-panel")).toBeVisible();
		await expect(page.getByTestId("governance-rules-control")).toBeDisabled();
		await expect(page.getByTestId("governance-commit-button")).toHaveCount(0);
		expect(writeRequests).toEqual([]);
	});
});
