import { expect, test } from "@playwright/experimental-ct-svelte";
import GovernanceSettingsHarness from "$tests/governance/GovernanceSettingsHarness.svelte";
import RulesListPrincipalIdHarness from "$tests/governance/RulesListPrincipalIdHarness.svelte";

test.describe("RulesList", () => {
	test("RulesListPrincipalIdScoped scopes the query and rows to principalId", async ({ mount }) => {
		const component = await mount(RulesListPrincipalIdHarness, {
			props: { principalId: "agent:codex-staging" },
		});

		await expect(component.getByTestId("principal-rules-call-count")).toHaveText("1");
		await expect(component.getByTestId("workspace-rules-call-count")).toHaveText("0");
		await expect(component.getByTestId("last-principal-project-id")).toHaveText("ct-project");
		await expect(component.getByTestId("last-principal-id")).toHaveText("agent:codex-staging");
		await expect(component.getByText("src/a1.ts")).toBeVisible();
		await expect(component.getByText("src/a2.ts")).toBeVisible();
		await expect(component.getByText("src/b1.ts")).toHaveCount(0);
		await expect(component.getByText("src/b2.ts")).toHaveCount(0);

		await component.getByTestId("show-cursor-bot").click();

		await expect(component.getByTestId("last-principal-id")).toHaveText("agent:cursor-bot");
		await expect(component.getByText("src/b1.ts")).toBeVisible();
		await expect(component.getByText("src/a1.ts")).toHaveCount(0);
		await expect(component.getByText("src/a2.ts")).toHaveCount(0);
	});

	test("RulesListWorkspaceUnchanged keeps workspace query when principalId is unset", async ({
		mount,
	}) => {
		const component = await mount(RulesListPrincipalIdHarness);

		await expect(component.getByTestId("workspace-rules-call-count")).toHaveText("1");
		await expect(component.getByTestId("principal-rules-call-count")).toHaveText("0");
		await expect(component.getByTestId("last-workspace-project-id")).toHaveText("ct-project");
		await expect(component.getByText("src/a1.ts")).toBeVisible();
		await expect(component.getByText("src/a2.ts")).toBeVisible();
		await expect(component.getByText("src/b1.ts")).toBeVisible();
		await expect(component.getByText("src/b2.ts")).toBeVisible();
	});

	test("RulesListEditorUnchanged opens editor for a scoped rule without principal props", async ({
		mount,
	}) => {
		const component = await mount(RulesListPrincipalIdHarness, {
			props: { principalId: "agent:codex-staging" },
		});

		await component.getByText("src/a1.ts").dblclick();

		await expect(component.getByRole("heading", { name: "Assign to branch" })).toBeVisible();
		await expect(component.getByTestId("last-principal-id")).toHaveText("agent:codex-staging");
		await expect(component.getByText("src/a2.ts")).toBeVisible();
	});

	test("RulesListPrincipalEmpty renders the existing empty state for empty principal rules", async ({
		mount,
	}) => {
		const component = await mount(RulesListPrincipalIdHarness, {
			props: { principalId: "agent:empty-bot" },
		});

		await expect(component.getByText("Let rules automatically sort your changes.")).toBeVisible();
		await expect(component.getByText("src/a1.ts")).toHaveCount(0);
		await expect(component.getByText("src/b1.ts")).toHaveCount(0);
	});
});

test.describe("GovernanceSettings", () => {
	test("GovernanceRulesTabNoPrincipal renders no-principal rules empty state", async ({
		mount,
	}) => {
		const component = await mount(GovernanceSettingsHarness, {
			props: { pendingCount: 0 },
		});

		await component.getByRole("tab", { name: "Rules" }).click();

		await expect(component.getByText("Select a principal to view their rules")).toBeVisible();
		await expect(component.getByTestId("rules-list-harness")).toHaveCount(0);
	});
});
