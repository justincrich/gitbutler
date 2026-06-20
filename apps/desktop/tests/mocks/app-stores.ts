import { readable } from "svelte/store";
import { page as pageState } from "./app-state";

export const navigating = readable(null);
export const page = readable(pageState);
export const updated = {
	check: async () => false,
	subscribe: readable(false).subscribe,
};
