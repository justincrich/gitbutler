import { expect, test } from "@playwright/experimental-ct-svelte";
import { spawnSync } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";

type GateResult = {
	stdout: string;
	stderr: string;
};

type SourceLine = {
	filePath: string;
	lineNumber: number;
	text: string;
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

function readScopedLines(relativePaths: string[]): SourceLine[] {
	return relativePaths.flatMap((relativePath) =>
		fs.existsSync(path.join(repoRoot, relativePath))
			? walkFiles(path.join(repoRoot, relativePath)).flatMap((filePath) =>
					fs
						.readFileSync(filePath, "utf8")
						.split(/\r?\n/)
						.map((line, index) => ({
							filePath: path.relative(repoRoot, filePath),
							lineNumber: index + 1,
							text: line,
						})),
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

function formatSourceLine(line: SourceLine): string {
	return `${line.filePath}:${line.lineNumber}:${line.text}`;
}

function isCommentOnlyLine(line: SourceLine): boolean {
	const text = line.text.trim();
	return (
		text.startsWith("//") ||
		text.startsWith("/*") ||
		text.startsWith("*") ||
		text.startsWith("<!--")
	);
}

function findDirectGovernanceWrites(lines: SourceLine[]): string[] {
	const directWritePattern =
		/@tauri-apps\/plugin-fs|\bplugin-fs\b|\bwriteFile\s*\(|\bwriteTextFile\s*\(|\bwriteBinaryFile\s*\(|\bfs\s*\.\s*write\b/;
	const gitbutlerConfigWriteCallPattern =
		/\b(write|save|persist|commit|update|mutate|create|delete)[A-Za-z0-9_]*\s*\(/i;

	return lines
		.filter((line) => {
			if (isCommentOnlyLine(line)) return false;
			if (directWritePattern.test(line.text)) return true;
			if (/\.gitbutler\/.*\.toml|gitbutler.*\.toml/i.test(line.text)) {
				return gitbutlerConfigWriteCallPattern.test(line.text);
			}
			return false;
		})
		.map(formatSourceLine);
}

function extractRustFunctionBody(source: string, functionName: string): string {
	const declaration = `pub fn ${functionName}`;
	const declarationIndex = source.indexOf(declaration);
	if (declarationIndex < 0) throw new Error(`Missing Rust function ${functionName}`);

	const bodyStart = source.indexOf("{", declarationIndex);
	if (bodyStart < 0) throw new Error(`Missing body for Rust function ${functionName}`);

	let depth = 0;
	for (let index = bodyStart; index < source.length; index++) {
		const char = source[index];
		if (char === "{") depth += 1;
		if (char === "}") depth -= 1;
		if (depth === 0) return source.slice(bodyStart, index + 1);
	}

	throw new Error(`Unterminated Rust function ${functionName}`);
}

test.describe("governance build gates", () => {
	test.describe.configure({ mode: "serial" });
	test.setTimeout(180_000);

	test("blocks direct governance file writes from governed desktop components", () => {
		const prohibitedLines = findDirectGovernanceWrites(readScopedLines(governanceComponentPaths));

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
		const governanceSource = fs.readFileSync(
			path.join(repoRoot, "crates/gitbutler-tauri/src/governance.rs"),
			"utf8",
		);
		const desktopWriteFunctions = [
			"perm_grant_for_desktop_session",
			"perm_revoke_for_desktop_session",
			"group_create_for_desktop_session",
			"group_grant_for_desktop_session",
			"group_revoke_for_desktop_session",
			"group_add_member_for_desktop_session",
			"group_remove_member_for_desktop_session",
			"group_delete_for_desktop_session",
			"branch_gates_read_for_desktop_session",
			"branch_gates_update_for_desktop_session",
			"governance_commit_for_desktop_session",
		];
		const functionsWithoutFleetOwnerContext = desktopWriteFunctions.filter(
			(functionName) =>
				!extractRustFunctionBody(governanceSource, functionName)
					.split(/\r?\n/)
					.some((line) => !line.trim().startsWith("//") && line.includes("fleet_owner_context(")),
		);

		expect(governanceSource).not.toContain("resolve_principal_from_env");
		expect(functionsWithoutFleetOwnerContext).toEqual([]);
	});

	test("does not add a governance-specific error boundary component", () => {
		expect(countMatchingFiles("apps/desktop/src", "GovernanceErrorBoundary.svelte")).toBe(0);
	});

	test("blocks direct Tauri fs imports from governed desktop components", () => {
		const fsImportLines = readScopedLines(governanceComponentPaths)
			.filter((line) => line.text.includes("@tauri-apps/plugin-fs"))
			.map(formatSourceLine);

		expect(fsImportLines).toEqual([]);
	});

	test("passes the repository lint gate", () => {
		// Invoke spawnSync directly so we can assert on the exit code explicitly;
		// runGateCommand swallows the status (it throws on non-zero), which would
		// leave the exit-code contract implicit rather than verified.
		const result = spawnSync("pnpm", ["lint"], {
			cwd: repoRoot,
			encoding: "utf8",
			stdio: ["ignore", "pipe", "pipe"],
		});
		const combined = `${result.stdout}${result.stderr}`;

		// A green lint run MUST exit 0 — a non-zero code means a linter failed.
		expect(result.status, `lint exited ${result.status}:\n${combined}`).toBe(0);

		// And the output must carry no failure markers. Each tool in the `pnpm lint`
		// chain (prettier, eslint, oxlint, knip) emits one of these on failure:
		//   - "✖ N problem" / "✖ N error"   (ESLint)
		//   - "[warn] Code style issue ..." (Prettier --check)
		//   - "N warning(s)" / "N error(s)" (oxlint)
		// None of these appear in a clean run, so their absence confirms success
		// rather than merely that the word "lint" was printed.
		expect(combined).not.toMatch(
			/✖|\b\d+\s+errors?\b|\b\d+\s+warnings?\b|\b\d+\s+problems?\b|\[warn\]|Code style issue/i,
		);
	});
});
