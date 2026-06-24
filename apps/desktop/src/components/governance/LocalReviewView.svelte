<script lang="ts" module>
	/**
	 * LPR-016: LocalReviewView — a READ-ONLY view rendering the full local PR
	 * object (assignments, threads, derived lifecycle, agent tag) from the
	 * LPR-015 SDK binding (reviewStatus + listComments via backend.invoke).
	 * One view, four lifecycle states. No mutate controls — all writes are
	 * CLI-driven (`but review approve`, `but review comment`, etc.).
	 */

	export type LocalReviewAssignmentEntry = {
		reviewer_principal: string;
		state: string;
		assigned_at?: string;
	};

	export type LocalReviewCommentEntry = {
		id: string;
		thread_id: string;
		file: string | null;
		line: number | null;
		resolved: boolean;
		author_principal: string;
		body: string;
		created_at: string;
	};

	export type LocalReviewStatusPayload = {
		lifecycle: string;
		agent_authored: boolean;
		approved: boolean;
		target: string;
		source_branch?: string;
		sha?: string;
		author?: string;
		title?: string;
		created_at?: string;
		assignments: LocalReviewAssignmentEntry[];
	};

	export type LocalReviewService = {
		reviewStatus: (projectId: string, branch: string) => Promise<LocalReviewStatusPayload | null>;
		listComments: (projectId: string, branch: string) => Promise<LocalReviewCommentEntry[]>;
	};

	export const AGENT_AUTHORED_TOOLTIP =
		"This PR was opened by a principal declared as an agent in .gitbutler/permissions.toml. This is a metadata tag — it does not affect merge decisions.";

	export const MERGE_GATE_NOTE =
		"Merge decisions are made by the merge gate, not this view. A status of Approved or Mergeable here reflects the derived state — the gate re-derives verdict at merge time.";

	type BadgeColor = "gray" | "pop" | "safe" | "warning";

	const LIFECYCLE_STYLES: Record<string, BadgeColor> = {
		Draft: "gray",
		Open: "gray",
		AwaitingReview: "pop",
		ChangesRequested: "warning",
		Approved: "safe",
		Mergeable: "safe",
	};

	const LIFECYCLE_LABELS: Record<string, string> = {
		Draft: "Draft",
		Open: "Draft",
		AwaitingReview: "Awaiting Review",
		ChangesRequested: "Changes Requested",
		Approved: "Approved",
		Mergeable: "Mergeable",
	};

	const LIFECYCLE_CAPTIONS: Record<string, string> = {
		Draft: "This review is in draft. Request a reviewer via `but review assign`.",
		Open: "This review is in draft. Request a reviewer via `but review assign`.",
		AwaitingReview: "Awaiting review. Reviewers can approve via `but review approve`.",
		ChangesRequested:
			"Changes requested. Address comments and push; reviewers can re-review via `but review approve`.",
		Approved: "Approved. The branch is ready for the merge gate.",
		Mergeable: "Mergeable. All conditions are met; attempt merge via `but merge`.",
	};

	export function lifecycleBadgeStyle(lifecycle: string): BadgeColor {
		return LIFECYCLE_STYLES[lifecycle] ?? "gray";
	}

	export function lifecycleLabel(lifecycle: string): string {
		return LIFECYCLE_LABELS[lifecycle] ?? lifecycle;
	}

	export function lifecycleCaption(lifecycle: string): string {
		return LIFECYCLE_CAPTIONS[lifecycle] ?? "";
	}

	export function shortRef(ref: string | undefined): string {
		if (!ref) return "—";
		return ref.replace(/^refs\/heads\//, "");
	}

	type ReviewStatusRaw = {
		lifecycle: string;
		agent_authored: boolean;
		approved: boolean;
		target: string;
		source_branch?: string;
		sha?: string;
		author?: string;
		title?: string;
		created_at?: string;
		assignments: unknown;
	};

	type BackendLike = { invoke: <T>(command: string, ...args: unknown[]) => Promise<T> };

	/**
	 * Factory that wires the LPR-015 Tauri commands (`review_status` +
	 * `list_comments`) into the LocalReviewService shape. Used by the
	 * production mount path; tests inject data directly via props.
	 */
	export function createLocalReviewService(backend: BackendLike): LocalReviewService {
		return {
			async reviewStatus(projectId, branch) {
				const raw = await backend.invoke<ReviewStatusRaw>("review_status", {
					projectId,
					branch,
				});
				const assignments = Array.isArray(raw.assignments)
					? (raw.assignments as LocalReviewAssignmentEntry[])
					: raw.assignments
						? [raw.assignments as LocalReviewAssignmentEntry]
						: [];
				return {
					lifecycle: raw.lifecycle,
					agent_authored: raw.agent_authored,
					approved: raw.approved,
					target: raw.target,
					source_branch: raw.source_branch,
					sha: raw.sha,
					author: raw.author,
					title: raw.title,
					created_at: raw.created_at,
					assignments,
				};
			},
			async listComments(projectId, branch) {
				return await backend.invoke<LocalReviewCommentEntry[]>("list_comments", {
					projectId,
					branch,
				});
			},
		};
	}
</script>

<script lang="ts">
	import LocalReviewAssignments from "$components/governance/LocalReviewAssignments.svelte";
	import LocalReviewThreads from "$components/governance/LocalReviewThreads.svelte";
	import { Badge, EmptyStatePlaceholder, InfoMessage, SkeletonBone, Tooltip } from "@gitbutler/ui";
	import { untrack } from "svelte";

	type Props = {
		projectId: string;
		branch: string;
		/** Pre-loaded review status — when provided, skips the loading state. */
		review?: LocalReviewStatusPayload | null;
		/** Pre-loaded comments — when provided, skips the loading state. */
		comments?: LocalReviewCommentEntry[];
		/** Service for the production load path (not used when review is provided). */
		service?: LocalReviewService;
		/**
		 * Forces the skeleton to render and blocks the load transition. Used by
		 * CT tests to simulate the in-flight state (function-valued service props
		 * do not survive the Playwright CT mount boundary, so the skeleton cannot
		 * be exercised via a never-resolving-promise service in CT).
		 */
		loading?: boolean;
		/** Accepted for mount-site parity; the view is read-only by design. */
		isReadOnly?: boolean;
	};

	const {
		projectId,
		branch,
		review: providedReview,
		comments: providedComments,
		service: providedService,
		loading: forceLoading = false,
		isReadOnly: _isReadOnly,
	}: Props = $props();

	// Resolve the service at init (GroupsList pattern) so it is captured before
	// the $effect fires. This is important in the CT harness where prop
	// reactivity across the mount boundary can lag.
	const resolvedService = untrack(() => providedService);

	let review = $state<LocalReviewStatusPayload | null | undefined>(untrack(() => providedReview));
	let comments = $state<LocalReviewCommentEntry[]>(untrack(() => providedComments ?? []));
	let isLoading = $state(untrack(() => forceLoading || providedReview === undefined));
	let loadError = $state<string | undefined>();

	$effect(() => {
		if (forceLoading) return;
		if (providedReview !== undefined) {
			review = providedReview;
			comments = providedComments ?? [];
			isLoading = false;
			return;
		}
		if (!resolvedService || !projectId || !branch) {
			isLoading = false;
			review = null;
			return;
		}
		untrack(() => {
			void loadReview();
		});
	});

	async function loadReview() {
		isLoading = true;
		loadError = undefined;
		try {
			const [statusResult, allComments] = await Promise.all([
				resolvedService!.reviewStatus(projectId, branch),
				resolvedService!.listComments(projectId, branch),
			]);
			review = statusResult;
			comments = allComments;
			isLoading = false;
		} catch (error) {
			loadError = error instanceof Error ? error.message : "local_review.load_failed";
			isLoading = false;
		}
	}

	const lifecycleStyle = $derived(
		review ? lifecycleBadgeStyle(review.lifecycle) : ("gray" as BadgeColor),
	);
	const lifecycleText = $derived(review ? lifecycleLabel(review.lifecycle) : "");
	const caption = $derived(review ? lifecycleCaption(review.lifecycle) : "");
</script>

<section class="local-review-view" data-testid="local-review-view">
	{#if isLoading}
		<div class="local-review-skeleton" data-testid="local-review-loading">
			<SkeletonBone height="2rem" />
			<SkeletonBone height="1rem" width="60%" />
			<SkeletonBone height="4rem" />
			<SkeletonBone height="3rem" />
		</div>
	{:else if loadError}
		<InfoMessage testId="local-review-error" style="danger" outlined>
			{#snippet title()}Could not load local review{/snippet}
			{#snippet content()}{loadError}{/snippet}
		</InfoMessage>
	{:else if !review}
		<div data-testid="local-review-empty">
			<EmptyStatePlaceholder gap={12} topBottomPadding={24}>
				{#snippet title()}No local review open for this branch.{/snippet}
				{#snippet caption()}Open one with `but review request &lt;branch&gt;` to start the review
					loop.{/snippet}
			</EmptyStatePlaceholder>
		</div>
	{:else}
		<header class="local-review-section" data-testid="local-review-header">
			<div class="local-review-header__badges">
				<Badge testId="local-review-lifecycle-badge" style={lifecycleStyle} kind="soft" size="tag">
					{lifecycleText}
				</Badge>
				{#if review.agent_authored}
					<Tooltip text={AGENT_AUTHORED_TOOLTIP} delay={0}>
						<Badge testId="local-review-agent-authored" style="gray" kind="soft" size="tag">
							agent-authored
						</Badge>
					</Tooltip>
				{/if}
			</div>
			<div class="local-review-header__branch">
				{shortRef(review.source_branch)} &rarr; {shortRef(review.target)}
			</div>
			<dl class="local-review-header__meta">
				{#if review.sha}<div>
						<dt>SHA</dt>
						<dd>{review.sha}</dd>
					</div>{/if}
				{#if review.author}<div>
						<dt>Author</dt>
						<dd>{review.author}</dd>
					</div>{/if}
				{#if review.title}<div>
						<dt>Title</dt>
						<dd>{review.title}</dd>
					</div>{/if}
				{#if review.created_at}<div>
						<dt>Created</dt>
						<dd>{review.created_at}</dd>
					</div>{/if}
			</dl>
		</header>

		<section class="local-review-section" data-testid="local-review-assignments">
			<h3>Reviewer Assignments</h3>
			<LocalReviewAssignments assignments={review.assignments} />
		</section>

		<section class="local-review-section" data-testid="local-review-threads">
			<h3>Comment Threads</h3>
			<LocalReviewThreads {comments} />
		</section>

		<p
			class="local-review-section local-review-caption"
			data-testid="local-review-lifecycle-caption"
		>
			{caption}
		</p>

		<div class="local-review-section" data-testid="local-review-merge-gate-note">
			<InfoMessage testId="local-review-merge-gate" style="info" outlined>
				{#snippet content()}{MERGE_GATE_NOTE}{/snippet}
			</InfoMessage>
		</div>
	{/if}
</section>

<style>
	.local-review-view {
		display: flex;
		flex-direction: column;
		gap: var(--clr-space-12, 16px);
	}

	.local-review-skeleton {
		display: flex;
		flex-direction: column;
		padding: var(--clr-space-12, 16px) 0;
		gap: var(--clr-space-8, 12px);
	}

	.local-review-section {
		display: flex;
		flex-direction: column;
		gap: var(--clr-space-4, 8px);
	}

	.local-review-section h3 {
		margin: 0;
	}

	.local-review-header__badges {
		display: flex;
		align-items: center;
		gap: var(--clr-space-4, 8px);
	}

	.local-review-header__branch {
		color: var(--clr-text-2, var(--text-2));
	}

	.local-review-header__meta {
		display: flex;
		flex-wrap: wrap;
		margin: 0;
		gap: var(--clr-space-8, 12px);
	}

	.local-review-header__meta div {
		display: flex;
		gap: var(--clr-space-2, 4px);
	}

	.local-review-header__meta dt {
		color: var(--clr-text-2, var(--text-2));
	}

	.local-review-header__meta dd {
		margin: 0;
	}

	.local-review-caption {
		margin: 0;
		color: var(--clr-text-2, var(--text-2));
	}
</style>
