import { defineConfig, devices } from "@playwright/experimental-ct-svelte";
import { resolve } from "path";

export default defineConfig({
	testDir: "./tests",
	timeout: 10 * 1000,
	fullyParallel: true,
	forbidOnly: !!process.env.CI,
	retries: process.env.CI ? 2 : 0,
	workers: process.env.CI ? 1 : undefined,
	reporter: "list",
	use: {
		ctPort: 3101,
		ctTemplateDir: "../../packages/ui/tests",
		ctViteConfig: {
			resolve: {
				alias: {
					$components: resolve("./src/components"),
					$lib: resolve("./src/lib"),
				},
			},
		},
	},
	projects: [
		{
			name: "chromium",
			use: { ...devices["Desktop Chrome"] },
		},
	],
});
