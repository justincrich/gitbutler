import { defineConfig, devices } from "@playwright/test";
import path from "node:path";

const FIXTURE_PORT = process.env.GOVERNANCE_FIXTURE_PORT || "4173";
const FIXTURE_URL = `http://127.0.0.1:${FIXTURE_PORT}`;
const E2E_ROOT = path.resolve(import.meta.dirname, "..");

export default defineConfig({
	testDir: "./tests/governance-fixture",
	fullyParallel: true,
	forbidOnly: !!process.env.CI,
	retries: process.env.CI ? 2 : 0,
	workers: process.env.CI ? 2 : 4,
	reporter: process.env.CI ? [["github"], ["buildkite-test-collector/playwright/reporter"]] : "dot",
	timeout: 60_000,
	expect: { timeout: 10_000 },
	use: {
		baseURL: FIXTURE_URL,
		actionTimeout: 10_000,
		trace: "retain-on-failure",
		screenshot: "only-on-failure",
		video: { mode: "retain-on-failure", size: { width: 1440, height: 1000 } },
	},
	projects: [
		{
			name: "chromium",
			use: {
				...devices["Desktop Chrome"],
				viewport: { width: 1440, height: 1000 },
				deviceScaleFactor: 1,
				headless: process.env.PLAYWRIGHT_UI === "1" ? false : undefined,
			},
		},
	],
	webServer: {
		cwd: E2E_ROOT,
		command: `pnpm exec vite --host 127.0.0.1 --port ${FIXTURE_PORT} --strictPort playwright/fixtures/governance-app`,
		url: FIXTURE_URL,
		reuseExistingServer: true,
		stdout: "pipe",
	},
});
