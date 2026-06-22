import {
	ADMIN_HANDLE,
	NONADMIN_HANDLE,
	currentProjectId,
	governanceSlug,
	openGovernanceProject,
	openGovernanceTab,
	openPrincipalEditor,
	readGovernancePending,
} from "../src/governance.ts";
import { getButlerPort } from "../src/setup.ts";
import { test } from "../src/test.ts";
import { expect, type Locator, type Page, type Response } from "@playwright/test";

type ButlerResponse<T> =
	| {
			type: "success";
			subject: T;
	  }
	| {
			type: "error";
			subject: unknown;
	  };

type ObservedPost<T> = {
	response: Response;
	body: ButlerResponse<T>;
	requestBody: unknown;
};

type GovernanceStatus = {
	authorities: string[];
};

type BranchGate = {
	name: string;
	protected: boolean;
	min_approvals: number;
	require_distinct_from_author: boolean;
	require_approval_from_group: string[];
};

type BranchGatesOutcome = {
	branches: BranchGate[];
};

type GovernanceCommitOutcome = {
	commitId: string;
	committedPaths: string[];
};

type WorkspaceRule = {
	id: string;
	filters: Array<{ type: string; subject?: unknown }>;
};

const RULE_PRINCIPAL_A = "test-principal";
const RULE_PRINCIPAL_B = "group-principal";
const RULE_LABEL_A = "capstone-agent-a-only";
const RULE_LABEL_B = "capstone-agent-b-only";

async function openGovernance(
	page: Page,
	gitbutler: { runScript: (name: string) => Promise<void> },
) {
	await gitbutler.runScript("governance-project-with-origin-main.sh");
	await openGovernanceProject(page, "admin");
}

async function observedCommandPost<T>(
	page: Page,
	command: string,
	trigger: () => Promise<void>,
): Promise<ObservedPost<T>> {
	const responsePromise = page.waitForResponse(
		(response) => response.url().endsWith(`/${command}`) && response.request().method() === "POST",
	);
	await trigger();
	const response = await responsePromise;
	const body = (await response.json()) as ButlerResponse<T>;
	const request = response.request();
	return {
		response,
		body,
		requestBody: request.postDataJSON(),
	};
}

function expectSuccess<T>(observed: ObservedPost<T>): T {
	expect(
		observed.response.status(),
		`${observed.response.url()} must return HTTP 2xx`,
	).toBeGreaterThanOrEqual(200);
	expect(
		observed.response.status(),
		`${observed.response.url()} must return HTTP 2xx`,
	).toBeLessThan(300);
	expect(observed.body.type, JSON.stringify(observed.body.subject)).toBe("success");
	return observed.body.subject as T;
}

function expectErrorCode(observed: ObservedPost<unknown>, code: string) {
	expect(
		observed.response.status(),
		`${observed.response.url()} must still be an HTTP response`,
	).toBeGreaterThanOrEqual(200);
	expect(
		observed.response.status(),
		`${observed.response.url()} must still be an HTTP response`,
	).toBeLessThan(300);
	expect(observed.body.type).toBe("error");
	expect(JSON.stringify(observed.body.subject)).toContain(code);
}

function commandRequestHasPrincipalId(
	observed: ObservedPost<unknown>,
	principalId: string,
): boolean {
	const requestBody = observed.requestBody;
	return (
		typeof requestBody === "object" &&
		requestBody !== null &&
		"principalId" in requestBody &&
		(requestBody as { principalId: unknown }).principalId === principalId
	);
}

async function expandBranchGate(page: Page, branchName = "main"): Promise<Locator> {
	await openGovernanceTab(page, "Branch Gates");
	const row = page.getByTestId(`branch-gates-list-row-${governanceSlug(branchName)}`);
	await expect(row).toBeVisible();
	await row.getByRole("button", { name: new RegExp(`^${branchName}\\b`) }).click();
	return row;
}

async function branchProtectedToggle(page: Page, branchName = "main"): Promise<Locator> {
	const row = await expandBranchGate(page, branchName);
	const toggle = row.getByTestId(`branch-gates-list-protected-${governanceSlug(branchName)}`);
	await expect(toggle).toBeVisible();
	return toggle;
}

async function createSessionRule(
	page: Page,
	principalId: string,
	label: string,
): Promise<WorkspaceRule> {
	const port = getButlerPort();
	const response = await page.request.post(`http://localhost:${port}/create_workspace_rule`, {
		data: {
			projectId: currentProjectId(page),
			request: {
				trigger: "fileSytemChange",
				filters: [
					{ type: "claudeCodeSessionId", subject: principalId },
					{ type: "pathMatchesRegex", subject: label },
				],
				action: {
					type: "explicit",
					subject: { type: "assign", subject: { target: { type: "leftmost" } } },
				},
			},
		},
	});
	expect(response.ok(), `create_workspace_rule for ${principalId} must return 2xx`).toBe(true);
	const body = (await response.json()) as ButlerResponse<WorkspaceRule>;
	expect(body.type, JSON.stringify(body.subject)).toBe("success");
	return body.subject as WorkspaceRule;
}

async function selectRulesPrincipal(
	page: Page,
	principalId: string,
): Promise<ObservedPost<WorkspaceRule[]>> {
	return await observedCommandPost<WorkspaceRule[]>(page, "list_workspace_rules", async () => {
		await page.getByTestId("governance-rules-principal-select").selectOption(principalId);
	});
}

async function expandRulesDrawer(page: Page) {
	const rulesList = page.getByTestId("governance-rules-list");
	await rulesList.getByRole("button").first().click();
}

async function expectActiveTab(page: Page, name: string) {
	const active = page.getByRole("tab", { name, exact: true });
	await expect(active).toHaveAttribute("aria-selected", "true");
	await expect(active).toHaveAttribute("tabindex", "0");
	await expect(page.getByRole("tabpanel")).toBeVisible();
}

test.describe("governance capstone admin", () => {
	test.use({ gitbutlerOptions: { env: { BUT_AGENT_HANDLE: ADMIN_HANDLE } } });

	test("step1 toggles Branch Gates protected control and stages pending update", async ({
		page,
		gitbutler,
	}) => {
		await openGovernance(page, gitbutler);

		const toggle = await branchProtectedToggle(page);
		await expect(toggle).toBeChecked();

		const observed = await observedCommandPost<BranchGatesOutcome>(
			page,
			"branch_gates_update",
			async () => {
				await toggle.click();
				await page
					.getByTestId("branch-gates-list-unprotect-modal")
					.getByRole("button", {
						name: "Unprotect branch",
					})
					.click();
			},
		);
		const outcome = expectSuccess(observed);
		expect(outcome.branches.some((branch) => branch.name === "main" && !branch.protected)).toBe(
			true,
		);

		await expect(page.getByTestId("governance-pending-banner")).toContainText(/^[1-9]\d* pending/);
		await expect(page.getByTestId("governance-commit-button")).toBeEnabled();
	});

	test("step2 selects principal-scoped Rules and sends principalId on real calls", async ({
		page,
		gitbutler,
	}) => {
		await openGovernance(page, gitbutler);
		await createSessionRule(page, RULE_PRINCIPAL_A, RULE_LABEL_A);
		await createSessionRule(page, RULE_PRINCIPAL_B, RULE_LABEL_B);
		await openGovernanceTab(page, "Rules");

		const scopedA = await selectRulesPrincipal(page, RULE_PRINCIPAL_A);
		expectSuccess(scopedA);
		expect(commandRequestHasPrincipalId(scopedA, RULE_PRINCIPAL_A)).toBe(true);
		await expandRulesDrawer(page);
		await expect(page.getByTestId("governance-rules-list")).toContainText(RULE_LABEL_A);
		await expect(page.getByTestId("governance-rules-list")).not.toContainText(RULE_LABEL_B);

		const scopedB = await selectRulesPrincipal(page, RULE_PRINCIPAL_B);
		expectSuccess(scopedB);
		expect(commandRequestHasPrincipalId(scopedB, RULE_PRINCIPAL_B)).toBe(true);
		await expandRulesDrawer(page);
		await expect(page.getByTestId("governance-rules-list")).toContainText(RULE_LABEL_B);
		await expect(page.getByTestId("governance-rules-list")).not.toContainText(RULE_LABEL_A);
	});

	test("step4 admin keeps pending edits across tabs and clears them after commit", async ({
		page,
		gitbutler,
	}) => {
		await openGovernance(page, gitbutler);
		const toggle = await branchProtectedToggle(page);
		await observedCommandPost<BranchGatesOutcome>(page, "branch_gates_update", async () => {
			await toggle.click();
			await page
				.getByTestId("branch-gates-list-unprotect-modal")
				.getByRole("button", {
					name: "Unprotect branch",
				})
				.click();
		});

		await expect(page.getByTestId("governance-pending-banner")).toContainText(/^[1-9]\d* pending/);
		await openGovernanceTab(page, "Groups");
		await expect(page.getByTestId("governance-pending-banner")).toContainText(/^[1-9]\d* pending/);
		await openGovernanceTab(page, "Branch Gates");
		await expect(page.getByTestId("governance-pending-banner")).toContainText(/^[1-9]\d* pending/);

		const commit = await observedCommandPost<GovernanceCommitOutcome>(
			page,
			"governance_commit",
			async () => {
				await page.getByTestId("governance-commit-button").click();
			},
		);
		expectSuccess(commit);
		await expect(page.getByTestId("governance-pending-banner")).toHaveCount(0);
	});

	test("step5 supports keyboard navigation across governance tabs", async ({ page, gitbutler }) => {
		await openGovernance(page, gitbutler);
		const principals = page.getByRole("tab", { name: "Principals", exact: true });
		const groups = page.getByRole("tab", { name: "Groups", exact: true });
		const branchGates = page.getByRole("tab", { name: "Branch Gates", exact: true });
		const rules = page.getByRole("tab", { name: "Rules", exact: true });

		await principals.focus();
		await expectActiveTab(page, "Principals");
		await page.keyboard.press("ArrowRight");
		await expect(groups).toHaveAttribute("aria-selected", "true");
		await expect(principals).toHaveAttribute("aria-selected", "false");
		await page.keyboard.press("ArrowRight");
		await expect(branchGates).toHaveAttribute("aria-selected", "true");
		await expect(groups).toHaveAttribute("aria-selected", "false");
		await page.keyboard.press("ArrowRight");
		await expectActiveTab(page, "Rules");
		await expect(branchGates).toHaveAttribute("aria-selected", "false");
		await page.keyboard.press("Home");
		await expectActiveTab(page, "Principals");
		await expect(rules).toHaveAttribute("aria-selected", "false");
		await page.keyboard.press("End");
		await expectActiveTab(page, "Rules");
		await expect(principals).toHaveAttribute("aria-selected", "false");
	});

	test("step6 surfaces IPC failure, keeps retry disabled state, and recovers after clear", async ({
		page,
		gitbutler,
	}) => {
		await gitbutler.runScript("governance-project-with-origin-main.sh");
		await page.route("**/governance_status_read", async (route) => {
			await route.fulfill({
				status: 500,
				contentType: "application/json",
				body: JSON.stringify({
					type: "error",
					subject: {
						code: "governance.status_read_failed",
						message: "capstone injected governance status failure",
					},
				}),
			});
		});

		try {
			await openGovernanceProject(page, "admin");
			await expect(page.getByTestId("governance-read-failure")).toBeVisible();
			await expect(
				page.getByTestId("governance-read-failure").getByRole("button", { name: "Retry" }),
			).toBeVisible();
			await expect(page.getByTestId("governance-read-only-message")).toBeVisible();
			await branchProtectedToggle(page);
			await expect(page.getByTestId("branch-gates-list-protected-main")).toBeDisabled();

			await page
				.getByTestId("governance-read-failure")
				.getByRole("button", { name: "Retry" })
				.click();
			await expect(page.getByTestId("governance-read-failure")).toBeVisible();
			await expect(page.getByTestId("branch-gates-list-protected-main")).toBeDisabled();

			await page.unroute("**/governance_status_read");
			await page
				.getByTestId("governance-read-failure")
				.getByRole("button", { name: "Retry" })
				.click();
			await expect(page.getByTestId("governance-read-failure")).toHaveCount(0);
			await expect(page.getByTestId("governance-read-only-message")).toHaveCount(0);
			await expect(page.getByTestId("branch-gates-list-protected-main")).toBeEnabled();
		} finally {
			await page.unroute("**/governance_status_read").catch(() => {});
		}
	});
});

test.describe("governance capstone nonadmin", () => {
	test.use({ gitbutlerOptions: { env: { BUT_AGENT_HANDLE: NONADMIN_HANDLE } } });

	test("step3 proves nonadmin read-only status and disabled write controls", async ({
		page,
		gitbutler,
	}) => {
		await gitbutler.runScript("governance-project-with-origin-main.sh");
		const status = await observedCommandPost<GovernanceStatus>(
			page,
			"governance_status_read",
			async () => {
				await openGovernanceProject(page, "admin");
			},
		);
		const subject = expectSuccess(status);
		expect(subject.authorities).not.toContain("administration:write");
		await expect(page.getByTestId("governance-read-only-message")).toBeVisible();

		const editor = await openPrincipalEditor(page, NONADMIN_HANDLE);
		await expect(editor.getByTestId("principal-editor-toggle-administration-write")).toBeDisabled();
		await expect(editor.getByTestId("principal-editor-toggle-contents-write")).toBeDisabled();
		await expect(editor.getByTestId("principal-editor-save")).toBeDisabled();
		// The branch-gates protected toggle renders inside an ExpandableSection
		// whose content is lazy ({#if expanded}); expand the row first so the
		// toggle exists in the DOM, then assert it is disabled for nonadmin.
		const gateRow = await expandBranchGate(page);
		await expect(gateRow.getByTestId("branch-gates-list-protected-main")).toBeDisabled();
		await openGovernanceTab(page, "Rules");
		// The <fieldset disabled> wrapper (governance-rules-control) is not
		// recognised as disabled by Playwright's toBeDisabled on the fieldset
		// element itself in webkit; assert the actual interactive <select>
		// (governance-rules-principal-select) which IS the write control.
		await expect(page.getByTestId("governance-rules-principal-select")).toBeDisabled();
	});

	test("step4 nonadmin self-grant is denied without changing admin control or pending count", async ({
		page,
		gitbutler,
	}) => {
		await gitbutler.runScript("governance-project-with-origin-main.sh");
		await openGovernanceProject(page, "admin");
		const beforePending = await readGovernancePending(page);
		const editor = await openPrincipalEditor(page, NONADMIN_HANDLE);
		const adminToggle = editor.getByTestId("principal-editor-toggle-administration-write");
		await expect(adminToggle).not.toBeChecked();

		const selfGrant = await observedCommandPost<unknown>(page, "perm_grant", async () => {
			await adminToggle.click({ force: true });
		});
		expectErrorCode(selfGrant, "perm.denied");
		await expect(editor.getByTestId("principal-editor-denial")).toContainText("perm.denied");
		await expect(adminToggle).not.toBeChecked();
		expect(await adminToggle.getAttribute("aria-checked")).not.toBe("true");

		const afterPending = await readGovernancePending(page);
		expect(afterPending.pendingCount).toBe(beforePending.pendingCount);
		await expect(page.getByTestId("governance-pending-banner")).toHaveCount(0);
	});
});
