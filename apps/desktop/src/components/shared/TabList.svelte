<script lang="ts">
	import { type Snippet } from "svelte";

	interface Props {
		children: Snippet;
		ariaLabel?: string;
	}

	const { ariaLabel, children }: Props = $props();

	function enabledTabs(tabList: EventTarget | null): HTMLButtonElement[] {
		if (!(tabList instanceof HTMLElement)) return [];

		return Array.from(tabList.querySelectorAll<HTMLButtonElement>('[role="tab"]')).filter(
			(tab) => !tab.disabled,
		);
	}

	function focusTab(tabs: HTMLButtonElement[], index: number) {
		tabs[index]?.focus();
	}

	function handleKeydown(event: KeyboardEvent) {
		const tabs = enabledTabs(event.currentTarget);
		const currentIndex = tabs.findIndex((tab) => tab === document.activeElement);
		if (currentIndex === -1) return;

		if (event.key === "ArrowRight") {
			event.preventDefault();
			focusTab(tabs, (currentIndex + 1) % tabs.length);
		} else if (event.key === "ArrowLeft") {
			event.preventDefault();
			focusTab(tabs, (currentIndex - 1 + tabs.length) % tabs.length);
		} else if (event.key === "Home") {
			event.preventDefault();
			focusTab(tabs, 0);
		} else if (event.key === "End") {
			event.preventDefault();
			focusTab(tabs, tabs.length - 1);
		}
	}
</script>

<ul
	class="segment-control-container"
	role="tablist"
	aria-label={ariaLabel}
	onkeydown={handleKeydown}
>
	{@render children()}
</ul>
