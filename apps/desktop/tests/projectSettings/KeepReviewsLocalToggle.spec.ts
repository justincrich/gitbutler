import KeepReviewsLocalToggleHarness, {
	type FixtureProject,
} from "./KeepReviewsLocalToggleHarness.svelte";
import { expect, test } from "@playwright/experimental-ct-svelte";

// Verbatim copy strings from DESIGN-LPR-001 / LPR-012 per-task contract.
// The per-task contract WINS over the consolidated UI-DESIGN-CONTRACTS.md copy
// (it is grep-audited) — these exact strings are load-bearing.
const LABEL_TEXT = "Keep agent reviews local";
const ON_STATE_DESCRIPTION =
	"Agent-authored PRs stay on the local review layer — no GitHub PR is opened. " +
	"Change this only if you want agent reviews mirrored to your forge. " +
	"(This is a local project preference, not a security boundary — the project store is not independently verified.)";
const ON_STATE_PARENTHETICAL =
	"(This is a local project preference, not a security boundary — the project store is not independently verified.)";

// The base project shape — all required `Project` fields populated.
function baseProject(overrides: Partial<FixtureProject> = {}): FixtureProject {
	return {
		id: "ct-project",
		title: "ct-repo",
		path: "/tmp/ct-repo",
		ok_with_force_push: false,
		force_push_protection: true,
		husky_hooks_enabled: false,
		omit_certificate_check: false,
		use_diff_context: false,
		is_open: true,
		forge_override: undefined,
		preferred_forge_user: null,
		gerrit_mode: false,
		forge_review_template_path: null,
		...overrides,
	};
}

// AC-1 fixture: an older project file WITHOUT the keep_reviews_local key.
// The DefaultTrue UI rule must render the Toggle ON (local).
const seeded_no_field_project: FixtureProject = baseProject({});

// AC-2 fixture: a project that has explicitly opted out (keep_reviews_local=false).
const seeded_explicit_false: FixtureProject = baseProject({ keep_reviews_local: false });

test.describe("KeepReviewsLocalToggle", () => {
	test("KeepReviewsLocalDefaultTrue", async ({ mount }) => {
		// AC-1 [PRIMARY]: default (no field / missing) renders Toggle ON (local)
		const component = await mount(KeepReviewsLocalToggleHarness, {
			props: { project: seeded_no_field_project },
		});

		const toggle = component.getByTestId("keepReviewsLocalToggle");
		await expect(toggle).toBeVisible();
		await expect(toggle).toBeChecked();

		// The label text matches DESIGN-LPR-001 / LPR-012 verbatim.
		await expect(component.getByText(LABEL_TEXT)).toBeVisible();

		// The on-state caption is present verbatim.
		await expect(component.getByText(ON_STATE_DESCRIPTION)).toBeVisible();

		// 0 project-settings write SDK calls fire on render (no side-effects on mount).
		await expect(component.getByTestId("keep-reviews-local-update-calls")).toHaveText("[]");
	});

	test("KeepReviewsLocalExplicitFalse", async ({ mount }) => {
		// AC-2: explicit keep_reviews_local=false renders Toggle OFF
		const component = await mount(KeepReviewsLocalToggleHarness, {
			props: { project: seeded_explicit_false },
		});

		const toggle = component.getByTestId("keepReviewsLocalToggle");
		await expect(toggle).toBeVisible();
		await expect(toggle).not.toBeChecked();

		// Label and caption are still present.
		await expect(component.getByText(LABEL_TEXT)).toBeVisible();
	});

	test("KeepReviewsLocalPersistsOnReopen", async ({ mount }) => {
		// AC-3: clicking persists to the project store and reflects on reopen.
		// Phase 1: mount with seeded_no_field_project (Toggle ON), click OFF.
		const first = await mount(KeepReviewsLocalToggleHarness, {
			props: { project: seeded_no_field_project },
		});

		const firstToggle = first.getByTestId("keepReviewsLocalToggle");
		await expect(firstToggle).toBeChecked();

		await firstToggle.click();
		await expect(firstToggle).not.toBeChecked();

		// The project-settings SDK write spy is called exactly once with
		// { keep_reviews_local: false }.
		await expect(first.getByTestId("keep-reviews-local-update-calls")).toContainText(
			'"keep_reviews_local":false',
		);
		await expect(first.getByTestId("keep-reviews-local-update-calls")).not.toContainText(
			'"keep_reviews_local":true',
		);

		// Phase 2: remount with seeded_explicit_false (simulating reopen after
		// the write persisted). The Toggle must render OFF.
		const second = await mount(KeepReviewsLocalToggleHarness, {
			props: { project: seeded_explicit_false },
		});

		const secondToggle = second.getByTestId("keepReviewsLocalToggle");
		await expect(secondToggle).not.toBeChecked();
	});

	test("KeepReviewsLocalWriteError", async ({ mount }) => {
		// AC-4: write error reverts the Toggle and surfaces danger InfoMessage.
		const component = await mount(KeepReviewsLocalToggleHarness, {
			props: {
				project: seeded_no_field_project,
				updateOutcome: "error",
			},
		});

		const toggle = component.getByTestId("keepReviewsLocalToggle");
		await expect(toggle).toBeChecked();

		// Click OFF — the write will reject.
		await toggle.click();

		// The Toggle reverts to ON (aria-checked='true').
		await expect(toggle).toBeChecked();

		// A danger InfoMessage is rendered containing the error context.
		await expect(component.getByTestId("keepReviewsLocalError")).toBeVisible();
		await expect(component.getByText("project.settings_write_failed")).toBeVisible();

		// Exactly 1 write call fired (no retry storm).
		await expect(component.getByTestId("keep-reviews-local-update-calls")).toHaveText(
			JSON.stringify([{ keep_reviews_local: false }]),
		);
	});

	test("KeepReviewsLocalCopy", async ({ mount }) => {
		// AC-5: copy matches DESIGN-LPR-001 / LPR-012 verbatim.
		const component = await mount(KeepReviewsLocalToggleHarness, {
			props: { project: seeded_no_field_project },
		});

		// Label text exactly.
		await expect(component.getByText(LABEL_TEXT)).toBeVisible();

		// On-state description begins with the verbatim lead.
		await expect(component.getByText(ON_STATE_DESCRIPTION)).toBeVisible();

		// R21 parenthetical is present (grep-audited honesty residual).
		await expect(component.getByText(ON_STATE_PARENTHETICAL, { exact: false })).toBeVisible();
	});
});
