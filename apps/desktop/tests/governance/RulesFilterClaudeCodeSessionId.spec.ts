import { expect, test } from "@playwright/experimental-ct-svelte";
import RulesFilterClaudeCodeSessionIdHarness from "$tests/governance/RulesFilterClaudeCodeSessionIdHarness.svelte";

test.describe("Rule", () => {
	test("RulesFilterClaudeCodeSessionId renders rule with claudeCodeSessionId filter without crashing", async ({
		mount,
	}) => {
		const component = await mount(RulesFilterClaudeCodeSessionIdHarness);

		// The pathMatchesRegex pill renders its label — proves the rule row mounted
		// at all (before the fix, Rule.svelte's getFilterConfig returned undefined
		// for the claudeCodeSessionId filter and renderBasicPill crashed evaluating
		// config.tooltip, preventing ANY rule rows from rendering).
		await expect(component.getByText("src/session-scoped.ts")).toBeVisible();

		// The claudeCodeSessionId pill renders its subject as the label.
		await expect(component.getByText("agent:codex-staging")).toBeVisible();
	});
});
