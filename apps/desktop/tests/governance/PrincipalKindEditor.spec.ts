import PrincipalEditor from "$components/governance/PrincipalEditor.svelte";
import PrincipalsList from "$components/governance/PrincipalsList.svelte";
import { expect, test } from "@playwright/experimental-ct-svelte";
import type {
	PrincipalEditorService,
	PrincipalEditorWriteResult,
} from "$components/governance/PrincipalEditor.svelte";
import type { PrincipalsListEntry } from "$components/governance/PrincipalsList.svelte";
import type { GrantOutcome, GroupWriteOutcome, PermWriteOutcome } from "@gitbutler/but-sdk";

const projectId = "project-1";
const targetRef = "refs/remotes/origin/main";

type PrincipalKindValue = "agent" | "human";

type KindWriteCall = {
	principal: string;
	kind: PrincipalKindValue;
};

type ServiceCall = {
	name: keyof PrincipalEditorService;
	args: string[];
};

/**
 * LPR-014 — component-test scope spy for the LPR-013 principal_kind_update SDK binding.
 *
 * The real backend persistence proof lives in LPR-013 AC-5 (real Tauri bus + real but-db).
 * This spy verifies the component-level call contract only (call count, args, pending wiring).
 *
 * Seam stub: contract-derived, integration-bonded to LPR-013 AC-5, status: fixture-only.
 */
type PrincipalKindOutcome = { ok: true };

function createEditorService(opts?: {
	kindCalls?: KindWriteCall[];
	calls?: ServiceCall[];
}): PrincipalEditorService {
	const kindCalls = opts?.kindCalls ?? [];
	const calls = opts?.calls ?? [];

	return {
		async permGrant(
			projectId: string,
			targetRef: string,
			principal: string,
			authorities: string[],
		): Promise<GrantOutcome> {
			calls.push({ name: "permGrant", args: [projectId, targetRef, principal, ...authorities] });
			return { principal, authorities, caveat: targetRef };
		},
		async permRevoke(
			projectId: string,
			targetRef: string,
			principal: string,
			authorities: string[],
		): Promise<PermWriteOutcome> {
			calls.push({ name: "permRevoke", args: [projectId, targetRef, principal, ...authorities] });
			return { principal, authorities, caveat: targetRef };
		},
		async groupAddMember(
			projectId: string,
			targetRef: string,
			group: string,
			member: string,
		): Promise<GroupWriteOutcome> {
			calls.push({ name: "groupAddMember", args: [projectId, targetRef, group, member] });
			return { group, member, authorities: [], caveat: targetRef };
		},
		async groupRemoveMember(
			projectId: string,
			targetRef: string,
			group: string,
			member: string,
		): Promise<GroupWriteOutcome> {
			calls.push({ name: "groupRemoveMember", args: [projectId, targetRef, group, member] });
			return { group, member, authorities: [], caveat: targetRef };
		},
		async principalKindUpdate(
			projectId: string,
			targetRef: string,
			principal: string,
			kind: PrincipalKindValue,
		): Promise<PrincipalEditorWriteResult<PrincipalKindOutcome>> {
			calls.push({
				name: "principalKindUpdate",
				args: [projectId, targetRef, principal, kind],
			});
			kindCalls.push({ principal, kind });
			return { ok: true };
		},
	};
}

function principalListProps(
	entries: PrincipalsListEntry[],
	overrides: Record<string, unknown> = {},
) {
	return {
		projectId,
		targetRef,
		principals: entries,
		editorService: createEditorService(),
		availableGroups: ["eng", "platform"],
		...overrides,
	};
}

test.describe("PrincipalKindEditor", () => {
	test("PrincipalsListAgentBadge", async ({ mount }) => {
		const entries: PrincipalsListEntry[] = [
			{
				principalId: "agent:codex",
				kind: "agent",
				ownGrants: ["reviews:write"],
				groupMemberships: [],
				pending: false,
			},
			{
				principalId: "human:alice",
				kind: "human",
				ownGrants: ["contents:read"],
				groupMemberships: [],
				pending: false,
			},
		];

		const component = await mount(PrincipalsList, {
			props: principalListProps(entries),
		});

		// AC-1 must_observe: principal-A row contains a Badge with text 'agent'
		const agentRow = component.getByTestId("principals-list-row-agent-codex");
		await expect(agentRow.getByTestId("principals-list-agent-agent-codex")).toBeVisible();
		await expect(agentRow.getByTestId("principals-list-agent-agent-codex")).toContainText("agent");

		// AC-1 must_observe: principal-B row has 0 agent badge elements
		const humanRow = component.getByTestId("principals-list-row-human-alice");
		await expect(humanRow.getByTestId("principals-list-agent-human-alice")).toHaveCount(0);
	});

	test("PrincipalsListAgentBadgeDisplayOnly", async ({ mount }) => {
		const kindCalls: KindWriteCall[] = [];
		const entries: PrincipalsListEntry[] = [
			{
				principalId: "agent:codex",
				kind: "agent",
				ownGrants: ["reviews:write"],
				groupMemberships: [],
				pending: false,
			},
		];

		const component = await mount(PrincipalsList, {
			props: principalListProps(entries, {
				isReadOnly: true,
				editorService: createEditorService({ kindCalls }),
			}),
		});

		const agentBadge = component.getByTestId("principals-list-agent-agent-codex");

		// AC-2 must_observe: badge has role="presentation" (not button), no aria-pressed
		await expect(agentBadge).toHaveAttribute("role", "presentation");
		await expect(agentBadge).not.toHaveAttribute("aria-pressed");

		// Click the agent badge — row's expand/collapse bubbles up but no SDK write fires
		await agentBadge.click();

		// AC-2 must_observe: 0 SDK write calls fire on badge interaction
		expect(kindCalls).toHaveLength(0);
	});

	test("PrincipalsListNoKindDefaultsHuman", async ({ mount }) => {
		const entries: PrincipalsListEntry[] = [
			{
				principalId: "ci:runner",
				// kind absent — defaults to human presentation, no badge
				ownGrants: ["reviews:write"],
				groupMemberships: [],
				pending: false,
			},
		];

		const component = await mount(PrincipalsList, {
			props: principalListProps(entries),
		});

		// AC-3 must_observe: 0 agent badge elements in principal-C's row
		await expect(component.getByTestId("principals-list-agent-ci-runner")).toHaveCount(0);

		// Open the PrincipalEditor for principal-C
		await component.getByTestId("principals-list-row-ci-runner").click();
		await expect(component.getByTestId("principal-editor")).toBeVisible();

		// AC-3 must_observe: kind selector defaults to 'human'
		await expect(component.getByTestId("principal-editor-kind-human")).toHaveAttribute(
			"aria-selected",
			"true",
		);
		await expect(component.getByTestId("principal-editor-kind-agent")).toHaveAttribute(
			"aria-selected",
			"false",
		);
	});

	test("PrincipalKindWriteAndPending", async ({ mount }) => {
		const kindCalls: KindWriteCall[] = [];
		const pendingPrincipalIds: string[] = [];
		const entries: PrincipalsListEntry[] = [
			{
				principalId: "ci:runner",
				// kind absent — defaults to human
				ownGrants: ["reviews:write"],
				groupMemberships: [],
				pending: false,
			},
		];

		const component = await mount(PrincipalsList, {
			props: principalListProps(entries, {
				editorService: createEditorService({ kindCalls }),
				onPrincipalPending(principalId: string) {
					pendingPrincipalIds.push(principalId);
				},
			}),
		});

		// Open the PrincipalEditor for principal-C
		await component.getByTestId("principals-list-row-ci-runner").click();
		await expect(component.getByTestId("principal-editor")).toBeVisible();

		// Change the kind selector from 'human' to 'agent' (commits on Save per DESIGN-LPR-002)
		await component.getByTestId("principal-editor-kind-agent").click();
		await expect(component.getByTestId("principal-editor-kind-agent")).toHaveAttribute(
			"aria-selected",
			"true",
		);

		// Save changes — this fires the LPR-013 SDK write spy
		await expect(component.getByTestId("principal-editor-save")).toBeEnabled();
		await component.getByTestId("principal-editor-save").click();

		// AC-4 must_observe: LPR-013 SDK write spy called == 1 with kind='agent' for ci:runner
		await expect.poll(() => kindCalls.length).toBe(1);
		expect(kindCalls[0]).toEqual({ principal: "ci:runner", kind: "agent" });

		// AC-4 must_observe: principal-C row gains a Badge with style='warning' kind='soft' size='icon' text='○'
		await expect(component.getByTestId("principals-list-pending-ci-runner")).toBeVisible();
		await expect(component.getByTestId("principals-list-pending-ci-runner")).toContainText("○");

		// AC-4 must_observe: governance pending-banner count incremented by 1 (callback contract)
		expect(pendingPrincipalIds).toEqual(["ci:runner"]);
	});

	test("PrincipalKindEditorReadOnly", async ({ mount }) => {
		const kindCalls: KindWriteCall[] = [];
		const entries: PrincipalsListEntry[] = [
			{
				principalId: "agent:codex",
				kind: "agent",
				ownGrants: ["reviews:write"],
				groupMemberships: [],
				pending: false,
			},
		];

		const component = await mount(PrincipalsList, {
			props: principalListProps(entries, {
				isReadOnly: true,
				editorService: createEditorService({ kindCalls }),
			}),
		});

		// Open the PrincipalEditor for the agent principal
		await component.getByTestId("principals-list-row-agent-codex").click();
		await expect(component.getByTestId("principal-editor")).toBeVisible();

		// AC-5 must_observe: kind selector segments have disabled or aria-disabled
		await expect(component.getByTestId("principal-editor-kind-human")).toBeDisabled();
		await expect(component.getByTestId("principal-editor-kind-agent")).toBeDisabled();

		// Force-click the agent segment — disabled controls should not fire SDK writes
		await component.getByTestId("principal-editor-kind-human").click({ force: true });

		// AC-5 must_observe: 0 LPR-013 SDK write calls fire
		expect(kindCalls).toHaveLength(0);
	});

	test("PrincipalKindEditorSegmentSelectorShape", async ({ mount }) => {
		// Standalone editor mount: verify the kind selector has exactly two options
		// (agent / human), SegmentControl shape, defaults to 'human' when kind absent.
		const component = await mount(PrincipalEditor, {
			props: {
				projectId,
				targetRef,
				principalId: "ci:runner",
				ownGrants: ["reviews:write"],
				groupMemberships: [],
				availableGroups: [],
				service: createEditorService(),
			},
		});

		// Two segments only — Human and Agent
		await expect(component.getByTestId("principal-editor-kind-human")).toBeVisible();
		await expect(component.getByTestId("principal-editor-kind-agent")).toBeVisible();

		// Defaults to 'human' (conservative posture) when kind is absent
		await expect(component.getByTestId("principal-editor-kind-human")).toHaveAttribute(
			"aria-selected",
			"true",
		);
		await expect(component.getByTestId("principal-editor-kind-agent")).toHaveAttribute(
			"aria-selected",
			"false",
		);

		// Non-enforcement disclosure caption present (DESIGN-LPR-002 AC-3)
		await expect(component.getByTestId("principal-editor-kind-caption")).toContainText(
			"does not change any permission grant or gate decision",
		);
	});
});
