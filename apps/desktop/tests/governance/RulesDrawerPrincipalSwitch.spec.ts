import { expect, test } from "@playwright/experimental-ct-svelte";
import RulesDrawerPrincipalSwitchHarness from "$tests/governance/RulesDrawerPrincipalSwitchHarness.svelte";

test.describe("RulesList drawer across principal switches", () => {
	test("RulesDrawerPrincipalSwitch re-expands after switching principal", async ({ mount }) => {
		const component = await mount(RulesDrawerPrincipalSwitchHarness, {
			props: { principalId: "agent:codex-staging" },
		});

		// Drawer starts collapsed (defaultCollapsed=true); the rule content
		// for principal A must be hidden initially.
		await expect(component.getByText("src/a1.ts")).toHaveCount(0);

		// Expand the drawer by clicking the first button (chevron toggle) —
		// this mirrors the capstone's expandRulesDrawer helper.
		await component.getByTestId("rules-list-harness").getByRole("button").first().click();
		await expect(component.getByText("src/a1.ts")).toBeVisible();
		await expect(component.getByText("src/a2.ts")).toBeVisible();

		// Switch to principal B.
		await component.getByTestId("show-cursor-bot").click();

		// Expand the drawer again for principal B. This is the exact sequence
		// the capstone step2 spec exercises, and the bug being fixed: without
		// resetting drawer state on principal change, this second toggle CLOSES
		// the drawer (which was already open from principal A) and hides B's
		// content. After the fix, the drawer collapses on principal switch so
		// this toggle opens it fresh.
		await component.getByTestId("rules-list-harness").getByRole("button").first().click();
		await expect(component.getByText("src/b1.ts")).toBeVisible();
		await expect(component.getByText("src/a1.ts")).toHaveCount(0);
	});

	test("RulesDrawerPrincipalSwitch collapses drawer on principal change", async ({ mount }) => {
		const component = await mount(RulesDrawerPrincipalSwitchHarness, {
			props: { principalId: "agent:codex-staging" },
		});

		// Expand for principal A.
		await component.getByTestId("rules-list-harness").getByRole("button").first().click();
		await expect(component.getByText("src/a1.ts")).toBeVisible();

		// Switch to principal B — the drawer should reset to collapsed so the
		// user starts from a clean view of B's rules.
		await component.getByTestId("show-cursor-bot").click();
		await expect(component.getByText("src/b1.ts")).toHaveCount(0);

		// And switching back to A also resets.
		await component.getByTestId("show-codex-staging").click();
		await expect(component.getByText("src/a1.ts")).toHaveCount(0);
	});
});
