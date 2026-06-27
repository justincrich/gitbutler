import PrincipalsList from "$components/governance/PrincipalsList.svelte";
import PrincipalsListBackendHarness from "./PrincipalsListBackendHarness.svelte";
import PrincipalsScrollHarness from "./PrincipalsScrollHarness.svelte";
import { expect, test } from "@playwright/experimental-ct-svelte";
import type { PrincipalEditorService } from "$components/governance/PrincipalEditor.svelte";
import type { PrincipalsListEntry } from "$components/governance/PrincipalsList.svelte";
import type { GrantOutcome, GroupWriteOutcome, PermWriteOutcome } from "@gitbutler/but-sdk";

const projectId = "project-1";
const targetRef = "refs/remotes/origin/main";

const seededPrincipals: PrincipalsListEntry[] = [
	{
		principalId: "codex-agent",
		ownGrants: ["contents:read"],
		inheritedGrants: [{ authority: "contents:write", sourceLabel: "group: eng" }],
		groupMemberships: ["eng"],
		pending: false,
	},
	{
		principalId: "claude-agent",
		ownGrants: ["pull_requests:write"],
		inheritedGrants: [],
		groupMemberships: [],
		pending: false,
	},
	{
		principalId: "cursor-bot",
		ownGrants: ["contents:read"],
		inheritedGrants: [{ authority: "contents:write", sourceLabel: "group: eng" }],
		groupMemberships: ["eng"],
		pending: true,
	},
];

function createEditorService(): PrincipalEditorService {
	return {
		async permGrant(projectId, targetRef, principal, authorities): Promise<GrantOutcome> {
			return { principal, authorities, caveat: targetRef };
		},
		async permRevoke(projectId, targetRef, principal, authorities): Promise<PermWriteOutcome> {
			return { principal, authorities, caveat: targetRef };
		},
		async groupAddMember(projectId, targetRef, group, member): Promise<GroupWriteOutcome> {
			return { group, member, authorities: [], caveat: targetRef };
		},
		async groupRemoveMember(projectId, targetRef, group, member): Promise<GroupWriteOutcome> {
			return { group, member, authorities: [], caveat: targetRef };
		},
	};
}

function baseProps(entries = seededPrincipals) {
	return {
		projectId,
		targetRef,
		principals: entries,
		editorService: createEditorService(),
		availableGroups: ["eng", "platform"],
	};
}

test("PrincipalsListRows renders a capability matrix with direct and inherited grants", async ({
	mount,
}) => {
	const component = await mount(PrincipalsList, {
		props: baseProps(),
	});

	await expect(component.getByTestId("principals-list-row")).toHaveCount(3);
	await expect(component.getByTestId("principals-list-row-codex-agent")).toContainText(
		"codex-agent",
	);
	// Direct own grant renders as a "own" cell; the group-inherited grant renders as "inherited".
	await expect(
		component.getByTestId("principals-cell-codex-agent-contents-read"),
	).toHaveAttribute("data-grant", "own");
	await expect(
		component.getByTestId("principals-cell-codex-agent-contents-write"),
	).toHaveAttribute("data-grant", "inherited");
	// Group membership is surfaced on the row.
	await expect(component.getByTestId("principals-list-row-codex-agent")).toContainText("eng");

	// A capability the agent does not hold renders as an empty cell.
	await expect(
		component.getByTestId("principals-cell-claude-agent-pull-requests-write"),
	).toHaveAttribute("data-grant", "own");
	await expect(
		component.getByTestId("principals-cell-claude-agent-administration-write"),
	).toHaveAttribute("data-grant", "none");
});

test("PrincipalsListGlossary explains what each capability means", async ({ mount }) => {
	const component = await mount(PrincipalsList, {
		props: baseProps(),
	});

	await component.getByRole("button", { name: "What these permissions mean" }).click();

	const glossary = component.getByTestId("principals-glossary");
	await expect(glossary).toContainText("Create commits and push changes");
	await expect(glossary).toContainText("Merge reviewed changes into a protected branch");
});

test("PrincipalsScroll matrix scrolls horizontally inside a constrained container", async ({
	mount,
}) => {
	const component = await mount(PrincipalsScrollHarness);
	const scroll = component.getByTestId("principals-scroll");

	const metrics = await scroll.evaluate((el) => ({
		clientWidth: el.clientWidth,
		scrollWidth: el.scrollWidth,
	}));

	// Constrained to the 420px container (does NOT overflow it)...
	expect(metrics.clientWidth).toBeLessThanOrEqual(420);
	// ...yet the wider matrix overflows the container, so it is horizontally scrollable.
	expect(metrics.scrollWidth).toBeGreaterThan(metrics.clientWidth);
});

test("PrincipalsListConfigHint points to the governed TOML file", async ({ mount }) => {
	const component = await mount(PrincipalsList, {
		props: baseProps(),
	});

	await expect(component.getByTestId("governance-config-hint")).toContainText(
		".gitbutler/agents.toml",
	);
});

test("PrincipalsListDefaultService reads all principals through governance backend", async ({
	mount,
}) => {
	const component = await mount(PrincipalsListBackendHarness);

	await expect(component.getByTestId("principals-list-backend-calls")).toContainText(
		"governance_principals_list",
	);
	await expect(component.getByTestId("principals-list-backend-args")).toContainText(
		"refs/remotes/origin/main",
	);
	await expect(component.getByTestId("principals-list-row-backend-agent")).toContainText(
		"backend-agent",
	);
	await expect(component.getByTestId("principals-list-row-backend-agent")).toContainText(
		"platform",
	);
});

test("PrincipalsListEditorToggle opens PrincipalEditor inline without navigation", async ({
	mount,
	page,
}) => {
	const component = await mount(PrincipalsList, {
		props: baseProps(),
	});
	const beforeUrl = page.url();

	await component
		.getByTestId("principals-list-row-claude-agent")
		.getByTestId("principals-list-row")
		.click();

	await expect(component.getByTestId("principal-editor")).toBeVisible();
	await expect(component.getByTestId("principal-editor")).toContainText("claude-agent");
	expect(page.url()).toBe(beforeUrl);
});

test("PrincipalsListPending marks pending row and keeps effective grants visible", async ({
	mount,
}) => {
	const component = await mount(PrincipalsList, {
		props: baseProps(),
	});

	await expect(component.getByTestId("principals-list-row-cursor-bot")).toContainText("○");
	await expect(
		component.getByTestId("principals-cell-cursor-bot-contents-read"),
	).toHaveAttribute("data-grant", "own");
	await expect(
		component.getByTestId("principals-cell-cursor-bot-contents-write"),
	).toHaveAttribute("data-grant", "inherited");
	await expect(component.getByTestId("principals-list-row-cursor-bot")).not.toContainText(
		"working-tree draft",
	);
});

test("PrincipalsListEmpty renders empty state and add action", async ({ mount }) => {
	const component = await mount(PrincipalsList, {
		props: baseProps([]),
	});

	await expect(component.getByTestId("principals-list-empty")).toContainText(
		"No principals configured",
	);
	await expect(component.getByRole("button", { name: "+ Add first" })).toBeVisible();
});
