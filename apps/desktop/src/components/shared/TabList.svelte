<script lang="ts">
	import { getContext, type Snippet } from "svelte";
	import type { TabContext } from "$lib/utils/tabs";

	interface Props {
		children: Snippet;
		ariaLabel?: string;
	}

	const { ariaLabel, children }: Props = $props();
	const tabStore = getContext<TabContext>("tab");

	function enabledTabs(tabList: EventTarget | null): HTMLButtonElement[] {
		if (!(tabList instanceof HTMLElement)) return [];

		return Array.from(tabList.querySelectorAll<HTMLButtonElement>('[role="tab"]')).filter(
			(tab) => !tab.disabled,
		);
	}

	function activateTab(tabs: HTMLButtonElement[], index: number) {
		const tab = tabs[index];
		if (!tab) return;

		tabStore?.setSelected(tab.value);
		tab.focus();
	}

	function handleKeydown(event: KeyboardEvent) {
		const tabs = enabledTabs(event.currentTarget);
		const currentIndex = tabs.findIndex((tab) => tab === document.activeElement);
		if (currentIndex === -1) return;

		if (event.key === "ArrowRight") {
			event.preventDefault();
			activateTab(tabs, (currentIndex + 1) % tabs.length);
		} else if (event.key === "ArrowLeft") {
			event.preventDefault();
			activateTab(tabs, (currentIndex - 1 + tabs.length) % tabs.length);
		} else if (event.key === "Home") {
			event.preventDefault();
			activateTab(tabs, 0);
		} else if (event.key === "End") {
			event.preventDefault();
			activateTab(tabs, tabs.length - 1);
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
