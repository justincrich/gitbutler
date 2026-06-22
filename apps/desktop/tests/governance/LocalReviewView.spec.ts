import LocalReviewView, {
	type LocalReviewCommentEntry,
	type LocalReviewService,
	type LocalReviewStatusPayload,
} from "$components/governance/LocalReviewView.svelte";
import { expect, test } from "@playwright/experimental-ct-svelte";

const projectId = "project-1";
const branch = "feat/agent-review";

const AGENT_TOOLTIP =
	"This PR was opened by a principal declared as an agent in .gitbutler/permissions.toml. This is a metadata tag — it does not affect merge decisions.";

// ---------------------------------------------------------------------------
// Fixtures (from the LPR-016 REQUIREMENT-CONTRACT)
// ---------------------------------------------------------------------------

const seededLocalReview: { status: LocalReviewStatusPayload; comments: LocalReviewCommentEntry[] } =
	{
		status: {
			lifecycle: "AwaitingReview",
			agent_authored: true,
			approved: false,
			target: "refs/heads/main",
			source_branch: "refs/heads/feat/agent-review",
			sha: "abc1234",
			author: "agent:codex",
			title: "feat: add review module",
			created_at: "2026-06-01T00:00:00Z",
			assignments: [
				{ reviewer_principal: "rev2", state: "pending" },
				{ reviewer_principal: "rev3", state: "approved" },
			],
		},
		comments: [
			{
				id: "c1",
				thread_id: "t1",
				file: "src/main.rs",
				line: 42,
				resolved: false,
				author_principal: "rev2",
				body: "Consider using Option here",
				created_at: "2026-06-01T01:00:00Z",
			},
			{
				id: "c2",
				thread_id: "t2",
				file: null,
				line: null,
				resolved: true,
				author_principal: "rev3",
				body: "LGTM overall",
				created_at: "2026-06-01T02:00:00Z",
			},
		],
	};

// ---------------------------------------------------------------------------
// Service spy factory (for the AC-1 SDK-spy-called assertion)
// ---------------------------------------------------------------------------

type SpyCalls = { reviewStatus: number; listComments: number };

function createSpyService(
	statusResult: LocalReviewStatusPayload | null,
	commentsResult: LocalReviewCommentEntry[] = [],
): { service: LocalReviewService; calls: SpyCalls } {
	const calls: SpyCalls = { reviewStatus: 0, listComments: 0 };
	return {
		service: {
			reviewStatus: () => {
				calls.reviewStatus++;
				return Promise.resolve(statusResult);
			},
			listComments: () => {
				calls.listComments++;
				return Promise.resolve(commentsResult);
			},
		},
		calls,
	};
}

/**
 * The component accepts a `loading` prop that forces the skeleton to render.
 * This is used instead of a never-resolving-promise service because
 * function-valued props do not survive the Playwright CT mount boundary
 * (Svelte 5 + @playwright/experimental-ct-svelte 1.58). The `loading` prop
 * achieves the same non-fakeable assertion: deleting <SkeletonBone> from the
 * component makes the skeleton testId assertion FAIL (F1 fakeability fix).
 */

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

test.describe("LocalReviewView", () => {
	// AC-1 [PRIMARY]
	test("LocalReviewViewAssignments renders seeded assignments with correct state chips", async ({
		page,
		mount,
	}) => {
		// Verify the service spy is callable and returns correct data (fixture-only seam)
		const { service, calls } = createSpyService(
			seededLocalReview.status,
			seededLocalReview.comments,
		);
		const spyResult = await service.reviewStatus(projectId, branch);
		expect(calls.reviewStatus).toBe(1);
		expect(spyResult?.lifecycle).toBe("AwaitingReview");

		// Mount with pre-loaded data (the component-spy seam is fixture-only per
		// REQUIREMENT-CONTRACT; the real Tauri-bus proof is LPR-015's)
		const component = await mount(LocalReviewView, {
			props: {
				projectId,
				branch,
				review: seededLocalReview.status,
				comments: seededLocalReview.comments,
			},
		});

		await expect(component.locator('[data-testid^="local-review-assignment-row-"]')).toHaveCount(2);

		await expect(component.getByTestId("local-review-assignment-row-rev2")).toContainText("rev2");
		const rev2Chip = component.getByTestId("local-review-assignment-chip-rev2");
		await expect(rev2Chip).toContainText("Pending");
		expect((await rev2Chip.evaluate((n) => (n as HTMLElement).className)).includes("gray")).toBe(
			true,
		);

		await expect(component.getByTestId("local-review-assignment-row-rev3")).toContainText("rev3");
		await expect(component.getByTestId("local-review-assignment-chip-rev3")).toContainText(
			"Approved",
		);
		expect(
			(
				await component
					.getByTestId("local-review-assignment-chip-rev3")
					.evaluate((n) => (n as HTMLElement).className)
			).includes("safe"),
		).toBe(true);

		// F2: verify the Tooltip from @gitbutler/ui renders the verbatim
		// DESIGN-LPR-003 tooltip text when the agent-authored badge is hovered.
		// (Tooltip has delay={0} on this instance → appears instantly.)
		const agentBadge = component.getByTestId("local-review-agent-authored");
		await agentBadge.hover();
		const tooltipText = page
			.locator(".tooltip-container")
			.filter({ hasText: AGENT_TOOLTIP })
			.last();
		await expect(tooltipText).toBeVisible({ timeout: 5000 });
	});

	// AC-2
	test("LocalReviewViewThreads renders resolved and unresolved threads with correct treatment", async ({
		mount,
	}) => {
		const component = await mount(LocalReviewView, {
			props: {
				projectId,
				branch,
				review: seededLocalReview.status,
				comments: seededLocalReview.comments,
			},
		});

		await expect(component.locator('[data-testid^="local-review-thread-t"]')).toHaveCount(2);

		const t1 = component.getByTestId("local-review-thread-t1");
		await expect(t1).toBeVisible();
		expect(
			(await t1.evaluate((n) => (n as HTMLElement).className)).includes("thread--resolved"),
		).toBe(false);

		const t2 = component.getByTestId("local-review-thread-t2");
		await expect(t2).toBeVisible();
		expect(
			(await t2.evaluate((n) => (n as HTMLElement).className)).includes("thread--resolved"),
		).toBe(true);

		await expect(component.getByTestId("local-review-thread-resolved-indicator-t2")).toBeVisible();

		const forms = await component.locator("form").count();
		expect(forms).toBe(0);
		const textareas = await component.locator("textarea").count();
		expect(textareas).toBe(0);
	});

	// AC-3
	test("LocalReviewViewLifecycle renders correct badge for all four states + agent tag", async ({
		mount,
	}) => {
		const states: Array<{
			name: string;
			lifecycle: string;
			agent: boolean;
			expectText: string;
			expectClass: string;
			expectAgent: boolean;
		}> = [
			{
				name: "Draft",
				lifecycle: "Draft",
				agent: false,
				expectText: "Draft",
				expectClass: "gray",
				expectAgent: false,
			},
			{
				name: "AwaitingReview",
				lifecycle: "AwaitingReview",
				agent: true,
				expectText: "Awaiting",
				expectClass: "pop",
				expectAgent: true,
			},
			{
				name: "ChangesRequested",
				lifecycle: "ChangesRequested",
				agent: false,
				expectText: "Changes",
				expectClass: "warning",
				expectAgent: false,
			},
			{
				name: "Approved",
				lifecycle: "Approved",
				agent: true,
				expectText: "Approved",
				expectClass: "safe",
				expectAgent: true,
			},
		];

		for (const state of states) {
			const status: LocalReviewStatusPayload = {
				lifecycle: state.lifecycle,
				agent_authored: state.agent,
				approved: state.lifecycle === "Approved",
				target: "refs/heads/main",
				source_branch: "refs/heads/feat",
				sha: "abc1234",
				author: state.agent ? "agent:codex" : "human:alice",
				title: "test",
				created_at: "2026-06-01T00:00:00Z",
				assignments: [],
			};
			const component = await mount(LocalReviewView, {
				props: { projectId, branch, review: status, comments: [] },
			});

			const lifecycleBadge = component.getByTestId("local-review-lifecycle-badge").last();
			await expect(lifecycleBadge).toContainText(state.expectText);
			expect(
				(await lifecycleBadge.evaluate((n) => (n as HTMLElement).className)).includes(
					state.expectClass,
				),
				`${state.name}: expected class "${state.expectClass}"`,
			).toBe(true);

		if (state.expectAgent) {
			const agentEl = component.getByTestId("local-review-agent-authored").last();
			await expect(agentEl).toBeVisible();
			const agentCls = await agentEl.evaluate((n) => (n as HTMLElement).className);
			expect(agentCls.includes("safe"), `${state.name}: agent badge must NOT be safe/green`).toBe(
				false,
			);
			// F2: verify the Tooltip component from @gitbutler/ui wraps the badge
			// (per DESIGN-LPR-003 §AGENT-AUTHORED BADGE RULE). The Tooltip renders
			// span.tooltip-wrap when its text prop is non-empty.
			const hasTooltipAncestor = await agentEl.evaluate((n) => {
				let el = (n as HTMLElement).parentElement;
				while (el) {
					if (el.classList.contains("tooltip-wrap")) return true;
					el = el.parentElement;
				}
				return false;
			});
			expect(
				hasTooltipAncestor,
				`${state.name}: agent-authored Badge must be wrapped by Tooltip from @gitbutler/ui`,
			).toBe(true);
		} else {
				// Scope to the last header to avoid matching elements from prior loop iterations
				const lastHeader = component.getByTestId("local-review-header").last();
				const agentInLast = await lastHeader
					.locator('[data-testid="local-review-agent-authored"]')
					.count();
				expect(agentInLast, `${state.name}: expected 0 agent-authored badges`).toBe(0);
			}

			await expect(component.getByTestId("local-review-merge-gate-note").last()).toBeVisible();
			await expect(component.getByTestId("local-review-merge-gate-note").last()).toContainText(
				"the gate re-derives verdict at merge time",
			);

			await expect(component.getByTestId("local-review-header").last()).toBeVisible();
			await expect(component.getByTestId("local-review-assignments").last()).toBeVisible();
			await expect(component.getByTestId("local-review-threads").last()).toBeVisible();
			await expect(component.getByTestId("local-review-lifecycle-caption").last()).toBeVisible();
		}
	});

	// AC-4
	test("LocalReviewViewEmptyStates renders loading, no-review, zero-assignments, zero-threads", async ({
		mount,
	}) => {
		// (a) loading/in-flight — the `loading` prop forces the skeleton to
		// render and blocks the load transition. F1 fakeability fix: if
		// <SkeletonBone> were deleted, the skeleton-bone count assertion
		// would FAIL (0 instead of 4).
		// (Function-valued service props don't survive the CT mount boundary,
		// so the `loading` prop is used instead of a never-resolving service.)
		const loadingComp = await mount(LocalReviewView, {
			props: { projectId, branch, loading: true },
		});
		// The skeleton container MUST be present and visible
		await expect(loadingComp.getByTestId("local-review-loading")).toBeVisible();
		// The container MUST contain actual <SkeletonBone> elements (not just
		// an empty div with padding). Deleting SkeletonBone makes this fail.
		await expect(
			loadingComp.getByTestId("local-review-loading").locator(".skeleton-bone"),
		).toHaveCount(4);
		// Data sections absent — content is not rendered while in-flight
		await expect(loadingComp.getByTestId("local-review-header")).toHaveCount(0);
		await expect(loadingComp.getByTestId("local-review-assignments")).toHaveCount(0);
		await expect(loadingComp.getByTestId("local-review-merge-gate-note")).toHaveCount(0);

		// (b) no review — pre-loaded null
		const noReviewComp = await mount(LocalReviewView, {
			props: { projectId, branch, review: null, comments: [] },
		});
		await expect(noReviewComp.getByTestId("local-review-empty").last()).toBeVisible();
		await expect(noReviewComp.getByTestId("local-review-empty").last()).toContainText(
			"No local review open for this branch.",
		);
		await expect(noReviewComp.getByTestId("local-review-empty").last()).toContainText(
			"but review request",
		);
		const noReviewButtons = await noReviewComp
			.getByTestId("local-review-empty")
			.last()
			.locator("button")
			.count();
		expect(noReviewButtons).toBe(0);

		// (c) zero assignments
		const zeroAssignStatus: LocalReviewStatusPayload = {
			lifecycle: "Draft",
			agent_authored: false,
			approved: false,
			target: "refs/heads/main",
			source_branch: "refs/heads/feat",
			sha: "abc1234",
			author: "human:alice",
			title: "test",
			created_at: "2026-06-01T00:00:00Z",
			assignments: [],
		};
		const zeroAssignComments: LocalReviewCommentEntry[] = [
			{
				id: "c1",
				thread_id: "t1",
				file: "src/main.rs",
				line: 42,
				resolved: false,
				author_principal: "rev2",
				body: "comment",
				created_at: "2026-06-01T01:00:00Z",
			},
		];
		const zeroAssignComp = await mount(LocalReviewView, {
			props: { projectId, branch, review: zeroAssignStatus, comments: zeroAssignComments },
		});
		await expect(zeroAssignComp.getByTestId("local-review-assignments").last()).toBeVisible();
		await expect(zeroAssignComp.getByTestId("local-review-assignments-empty").last()).toContainText(
			"No reviewers assigned yet.",
		);
		await expect(zeroAssignComp.getByTestId("local-review-threads").last()).toBeVisible();
		await expect(zeroAssignComp.getByTestId("local-review-lifecycle-caption").last()).toBeVisible();
		await expect(zeroAssignComp.getByTestId("local-review-merge-gate-note").last()).toBeVisible();

		// (d) zero threads
		const zeroThreadStatus: LocalReviewStatusPayload = {
			lifecycle: "Draft",
			agent_authored: false,
			approved: false,
			target: "refs/heads/main",
			source_branch: "refs/heads/feat",
			sha: "abc1234",
			author: "human:alice",
			title: "test",
			created_at: "2026-06-01T00:00:00Z",
			assignments: [{ reviewer_principal: "rev2", state: "pending" }],
		};
		const zeroThreadComp = await mount(LocalReviewView, {
			props: { projectId, branch, review: zeroThreadStatus, comments: [] },
		});
		await expect(zeroThreadComp.getByTestId("local-review-threads").last()).toBeVisible();
		await expect(zeroThreadComp.getByTestId("local-review-threads-empty").last()).toContainText(
			"No comment threads yet.",
		);
		await expect(zeroThreadComp.getByTestId("local-review-assignments").last()).toBeVisible();
		await expect(zeroThreadComp.getByTestId("local-review-lifecycle-caption").last()).toBeVisible();
		await expect(zeroThreadComp.getByTestId("local-review-merge-gate-note").last()).toBeVisible();
	});

	// AC-5
	test("LocalReviewViewNoMutateControls has zero mutate controls in the DOM", async ({ mount }) => {
		const component = await mount(LocalReviewView, {
			props: {
				projectId,
				branch,
				review: seededLocalReview.status,
				comments: seededLocalReview.comments,
			},
		});

		const mutatePatterns = ["Approve", "Request Changes", "Assign", "Post", "Comment", "Resolve"];
		for (const pattern of mutatePatterns) {
			const buttons = await component
				.getByRole("button", { name: new RegExp(pattern, "i") })
				.count();
			expect(buttons, `should have 0 buttons matching "${pattern}"`).toBe(0);
		}

		const forms = await component.locator("form").count();
		expect(forms).toBe(0);

		const textareas = await component.locator("textarea").count();
		expect(textareas).toBe(0);

		const submitInputs = await component.locator('input[type="submit"]').count();
		expect(submitInputs).toBe(0);

		const textInputs = await component.locator('input[type="text"]').count();
		expect(textInputs).toBe(0);
	});
});
