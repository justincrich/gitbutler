import {
	openPermissionsGovernanceSettings,
	openProjectSettings,
	projectSettingsSidebar,
	seedSignedInUser,
} from "../src/governance.ts";
import { openWorkspace } from "../src/setup.ts";
import { test } from "../src/test.ts";
import { expect } from "@playwright/test";

test("admin sees Permissions & Governance in the real Project Settings sidebar", async ({
	page,
	gitbutler,
}) => {
	await gitbutler.runScript("governance-project-with-origin-main.sh");
	await seedSignedInUser("admin");
	await openWorkspace(page);

	await openProjectSettings(page);
	await openPermissionsGovernanceSettings(page);

	const settingsSidebar = projectSettingsSidebar(page);
	await expect(page.getByTestId("project-settings-modal")).toBeVisible();
	await expect(
		settingsSidebar.getByRole("button", { name: "Project", exact: true }),
	).toHaveCount(1);
	await expect(
		settingsSidebar.getByRole("button", { name: "Permissions & Governance", exact: true }),
	).toHaveCount(1);
	await expect(page.getByText("Settings page governance not Found.")).toHaveCount(0);
});

test("member does not see Permissions & Governance in the real Project Settings sidebar", async ({
	page,
	gitbutler,
}) => {
	await gitbutler.runScript("governance-project-with-origin-main.sh");
	await seedSignedInUser("member");
	await openWorkspace(page);

	await openProjectSettings(page);

	const settingsSidebar = projectSettingsSidebar(page);
	await expect(page.getByTestId("project-settings-modal")).toBeVisible();
	await expect(
		settingsSidebar.getByRole("button", { name: "Project", exact: true }),
	).toHaveCount(1);
	await expect(
		settingsSidebar.getByRole("button", { name: "Permissions & Governance", exact: true }),
	).toHaveCount(0);
});
