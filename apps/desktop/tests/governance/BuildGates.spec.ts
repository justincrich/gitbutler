import { expect, test } from "@playwright/experimental-ct-svelte";
import { spawnSync } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";

type GateResult = {
	stdout: string;
	stderr: string;
};

function findRepoRoot(startPath: string): string {
	let currentPath = startPath;

	while (currentPath !== path.dirname(currentPath)) {
		if (fs.existsSync(path.join(currentPath, "pnpm-workspace.yaml"))) return currentPath;
		currentPath = path.dirname(currentPath);
	}

	throw new Error(`Could not find repo root from ${startPath}`);
}

const repoRoot = findRepoRoot(process.cwd());
const governanceComponentPaths = [
	"apps/desktop/src/components/governance",
	"apps/desktop/src/components/settings/GovernanceSettings.svelte",
	"apps/desktop/src/components/rules/RulesList.svelte",
];

function walkFiles(targetPath: string): string[] {
	const stat = fs.statSync(targetPath);
	if (stat.isFile()) return [targetPath];

	return fs.readdirSync(targetPath, { withFileTypes: true }).flatMap((entry) => {
		const entryPath = path.join(targetPath, entry.name);
		if (entry.isDirectory()) return walkFiles(entryPath);
		if (entry.isFile()) return [entryPath];
		return [];
	});
}

function readScopedLines(relativePaths: string[]): string[] {
	return relativePaths.flatMap((relativePath) =>
		fs.existsSync(path.join(repoRoot, relativePath))
			? walkFiles(path.join(repoRoot, relativePath)).flatMap((filePath) =>
					fs
						.readFileSync(filePath, "utf8")
						.split(/\r?\n/)
						.map((line, index) => `${path.relative(repoRoot, filePath)}:${index + 1}:${line}`),
				)
			: [],
	);
}

function countMatchingFiles(root: string, fileName: string): number {
	return walkFiles(path.join(repoRoot, root)).filter(
		(filePath) => path.basename(filePath) === fileName,
	).length;
}

function runGateCommand(command: string, args: string[]): GateResult {
	const result = spawnSync(command, args, {
		cwd: repoRoot,
		encoding: "utf8",
		stdio: ["ignore", "pipe", "pipe"],
	});

	if (result.status !== 0) {
		throw new Error(
			[
				`Command failed (${result.status}): ${command} ${args.join(" ")}`,
				"stdout:",
				result.stdout,
				"stderr:",
				result.stderr,
			].join("\n"),
		);
	}

	return { stdout: result.stdout, stderr: result.stderr };
}

test.describe("governance build gates", () => {
	test.describe.configure({ mode: "serial" });
	test.setTimeout(180_000);

	test("blocks direct governance file writes from governed desktop components", () => {
		const prohibitedPattern =
			/gitbutler.*\.toml|writeFile|fs\.write|writeTextFile|writeBinaryFile|plugin-fs/;
		const ignoredPattern = /but-sdk|import|\/\/|warn|error|log/;
		const prohibitedLines = readScopedLines(governanceComponentPaths).filter(
			(line) => prohibitedPattern.test(line) && !ignoredPattern.test(line),
		);

		expect(prohibitedLines).toEqual([]);
	});

	test("keeps the desktop source tree static-rendering only", () => {
		expect(countMatchingFiles("apps/desktop/src", "+page.server.ts")).toBe(0);
	});

	test("typechecks the desktop app against the regenerated SDK", () => {
		const result = runGateCommand("pnpm", ["-F", "@gitbutler/desktop", "check"]);

		expect(`${result.stdout}${result.stderr}`).toContain("@gitbutler/desktop");
	});

	test("keeps governance Tauri command wiring on the fleet-owner identity shim", () => {
		const tauriLines = readScopedLines(["crates/gitbutler-tauri/src"]);
		const governanceCommandPattern = /governance|perm_|group_|branch_gates/i;
		const commentPattern = /\/\//;
		const fleetOwnerLines = tauriLines.filter(
			(line) =>
				/fleet_owner|with_fleet_owner_identity|UserService|forge_session/.test(line) &&
				governanceCommandPattern.test(line) &&
				!commentPattern.test(line),
		);
		const envPrincipalLines = tauriLines.filter(
			(line) =>
				line.includes("resolve_principal_from_env") &&
				governanceCommandPattern.test(line) &&
				!commentPattern.test(line),
		);

		expect(fleetOwnerLines.length).toBeGreaterThan(0);
		expect(envPrincipalLines).toEqual([]);
	});

	test("does not add a governance-specific error boundary component", () => {
		expect(countMatchingFiles("apps/desktop/src", "GovernanceErrorBoundary.svelte")).toBe(0);
	});

	test("blocks direct Tauri fs imports from governed desktop components", () => {
		const fsImportLines = readScopedLines(governanceComponentPaths).filter((line) =>
			line.includes("@tauri-apps/plugin-fs"),
		);

		expect(fsImportLines).toEqual([]);
	});

	test("passes the repository lint gate", () => {
		const result = runGateCommand("pnpm", ["lint"]);

		expect(`${result.stdout}${result.stderr}`).toContain("lint");
	});
});
