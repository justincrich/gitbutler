<script lang="ts">
	import KeepReviewsLocalToggle from "$components/projectSettings/KeepReviewsLocalToggle.svelte";
	import { PROJECTS_SERVICE } from "$lib/project/projectsService";
	import { provide } from "@gitbutler/core/context";
	import { QueryStatus } from "@reduxjs/toolkit/query";
	import type { Project } from "$lib/project/project";

	/**
	 * The fixture project shape for CT seeds. `keep_reviews_local` is optional
	 * so the harness can seed `seeded_no_field_project` (no key — DefaultTrue)
	 * vs `seeded_explicit_false` (explicit false).
	 */
	export type FixtureProject = Project & { keep_reviews_local?: boolean | null };

	type UpdateCall = {
		keep_reviews_local: boolean | null | undefined;
	};

	type Props = {
		project: FixtureProject;
		/**
		 * "success" — updateProject resolves.
		 * "error"   — updateProject rejects with Error("project.settings_write_failed").
		 */
		updateOutcome?: "success" | "error";
	};

	const { project, updateOutcome = "success" }: Props = $props();

	// Reactive call log — mirrored to <output> so the spec can assert on
	// call count and args without reaching into globalThis. Pattern matches
	// SettingsModalLayoutAdminHarness's `<output data-testid="governance-backend-calls">`.
	let calls = $state<UpdateCall[]>([]);
	let callsJson = $state("[]");

	function recordCall(next: UpdateCall) {
		calls = [...calls, next];
		callsJson = JSON.stringify(calls);
	}

	// The mock projectsService. Only the surface KeepReviewsLocalToggle touches
	// is implemented — getProject returns the seeded fixture as a fulfilled
	// ReactiveQuery, and updateProject is the spy.
	const mockProjectsService = {
		getProject(_projectId: string) {
			return {
				response: project,
				result: {
					data: project,
					status: QueryStatus.fulfilled,
					error: undefined,
				},
			};
		},
		async updateProject(next: Project & Record<string, unknown>) {
			recordCall({ keep_reviews_local: next.keep_reviews_local as boolean | null | undefined });
			if (updateOutcome === "error") {
				throw new Error("project.settings_write_failed");
			}
		},
	} as unknown as typeof import("$lib/project/projectsService").ProjectsService;

	provide(PROJECTS_SERVICE, mockProjectsService);
</script>

<KeepReviewsLocalToggle projectId="ct-project" />

<output data-testid="keep-reviews-local-update-calls">{callsJson}</output>
