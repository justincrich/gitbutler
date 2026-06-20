type NavigationCallback = (navigation: unknown) => void;
type NavigationTarget = string | URL;
type InvalidationTarget = NavigationTarget | ((url: URL) => boolean);

export async function goto(_url: NavigationTarget): Promise<void> {
	return;
}

export async function invalidate(_resource: InvalidationTarget): Promise<void> {
	return;
}

export async function invalidateAll(): Promise<void> {
	return;
}

export async function preloadData(_href: string): Promise<unknown> {
	return { type: "loaded", status: 200, data: {} };
}

export async function preloadCode(..._hrefs: string[]): Promise<void> {
	return;
}

export function beforeNavigate(_callback: NavigationCallback): void {
	return;
}

export function afterNavigate(_callback: NavigationCallback): void {
	return;
}
