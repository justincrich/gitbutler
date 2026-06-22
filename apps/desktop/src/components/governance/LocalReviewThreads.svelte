<script lang="ts" module>
	import type { LocalReviewCommentEntry } from "$components/governance/LocalReviewView.svelte";

	export type LocalReviewThreadView = {
		thread_id: string;
		file: string | null;
		line: number | null;
		resolved: boolean;
		comments: LocalReviewCommentEntry[];
	};

	/**
	 * Group flat comments into threads by thread_id. A thread is resolved
	 * only when ALL its comments carry resolved=true (per ReviewStatus
	 * semantics). Comments within a thread preserve created_at ASC order.
	 */
	export function groupThreads(comments: LocalReviewCommentEntry[]): LocalReviewThreadView[] {
		const map = new Map<string, LocalReviewCommentEntry[]>();
		for (const comment of comments) {
			const group = map.get(comment.thread_id) ?? [];
			group.push(comment);
			map.set(comment.thread_id, group);
		}
		return [...map.entries()]
			.map(([thread_id, group]) => {
				const first = group[0];
				return {
					thread_id,
					file: first ? first.file : null,
					line: first ? first.line : null,
					resolved: group.every((c) => c.resolved),
					comments: [...group].sort((a, b) => a.created_at.localeCompare(b.created_at)),
				};
			})
			.sort((a, b) => a.thread_id.localeCompare(b.thread_id));
	}

	export function threadLocationLabel(thread: LocalReviewThreadView): string {
		if (thread.file === null) return "PR-level";
		return thread.line !== null ? `${thread.file}:${thread.line}` : thread.file;
	}
</script>

<script lang="ts">
	import { Badge } from "@gitbutler/ui";

	type Props = {
		comments: LocalReviewCommentEntry[];
	};

	let { comments }: Props = $props();

	const threads = $derived(groupThreads(comments));
</script>

{#if threads.length === 0}
	<p class="local-review-threads-empty" data-testid="local-review-threads-empty">
		No comment threads yet.
	</p>
{:else}
	<ul class="local-review-threads" data-testid="local-review-threads-list">
		{#each threads as thread (thread.thread_id)}
			<li
				class="local-review-thread"
				class:thread--resolved={thread.resolved}
				data-testid={`local-review-thread-${thread.thread_id}`}
			>
				<div class="local-review-thread__header">
					<span class="local-review-thread__location">
						{threadLocationLabel(thread)}
					</span>
					{#if thread.resolved}
						<Badge
							testId={`local-review-thread-resolved-indicator-${thread.thread_id}`}
							style="safe"
							kind="soft"
							size="tag"
						>
							Resolved
						</Badge>
					{/if}
				</div>
				<ul class="local-review-thread__comments">
					{#each thread.comments as comment (comment.id)}
						<li class="local-review-comment">
							<span class="local-review-comment__author">{comment.author_principal}</span>
							<span class="local-review-comment__body">{comment.body}</span>
						</li>
					{/each}
				</ul>
			</li>
		{/each}
	</ul>
{/if}

<style>
	.local-review-threads {
		display: flex;
		flex-direction: column;
		margin: 0;
		padding: 0;
		gap: var(--clr-space-8, 12px);
		list-style: none;
	}

	.local-review-thread {
		display: flex;
		flex-direction: column;
		padding: var(--clr-space-8, 12px);
		gap: var(--clr-space-4, 8px);
		border: 1px solid var(--clr-border-2, var(--border-2));
		border-radius: var(--radius-s, 4px);
		background: var(--clr-bg-1, var(--bg-1));
		color: var(--clr-text-1, var(--text-1));
	}

	.thread--resolved {
		opacity: 0.5;
	}

	.local-review-thread__header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--clr-space-4, 8px);
	}

	.local-review-thread__location {
		font-weight: 600;
		font-family: var(--font-mono, monospace);
	}

	.local-review-thread__comments {
		display: flex;
		flex-direction: column;
		margin: 0;
		padding: 0;
		gap: var(--clr-space-4, 8px);
		list-style: none;
	}

	.local-review-comment {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.local-review-comment__author {
		font-weight: 600;
		font-size: 0.85em;
	}

	.local-review-comment__body {
		color: var(--clr-text-1, var(--text-1));
	}

	.local-review-threads-empty {
		margin: 0;
		color: var(--clr-text-2, var(--text-2));
	}
</style>
