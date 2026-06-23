<script lang="ts">
	import { getContext, type Snippet } from "svelte";
	import type { TabContext } from "$lib/utils/tabs";

	interface Props {
		children: Snippet;
		value: string;
		disabled?: boolean;
	}

	const { value, children, disabled }: Props = $props();

	const tabStore = getContext<TabContext>("tab");
	const selectedIndex = $derived(tabStore.selectedIndex);
	const isActive = $derived($selectedIndex === value);

	function setActive() {
		tabStore?.setSelected(value);
	}
</script>

<button
	type="button"
	role="tab"
	tabindex={isActive ? 0 : -1}
	aria-selected={isActive}
	aria-controls={`${value}-panel`}
	id={value}
	{value}
	{disabled}
	onclick={setActive}
	class="segment-control-item"
	class:disabled
	class:active={isActive}
>
	<span class="text-12 text-semibold segment-control-item__label">
		{@render children()}
	</span>
</button>
