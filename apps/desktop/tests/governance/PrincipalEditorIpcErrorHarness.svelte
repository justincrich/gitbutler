<script lang="ts">
	import PrincipalEditor from "$components/governance/PrincipalEditor.svelte";
	import { IpcError } from "$lib/error/normalizedError";
	import type { PrincipalEditorService } from "$components/governance/PrincipalEditor.svelte";

	type Props = {
		failureMode?: "ipc" | "result";
	};

	const { failureMode = "ipc" }: Props = $props();
	const targetRef = "refs/remotes/origin/main";
	const projectId = "ct-project";

	let calls = $state(0);

	function structuredDenial() {
		return {
			code: "perm.denied",
			message: "Governance write denied by branch protection policy",
			remediation_hint: "Ask an administrator with administration:write to approve this change.",
		};
	}

	const service: PrincipalEditorService = {
		async permGrant(_projectId, _targetRef, _principal, _authorities) {
			calls += 1;
			if (failureMode === "result") return structuredDenial();
			throw new IpcError(structuredDenial(), "perm_grant");
		},
		async permRevoke(_projectId, _targetRef, principal, authorities) {
			return { authorities, caveat: "", principal };
		},
		async groupAddMember(_projectId, _targetRef, group, member) {
			return { authorities: [], caveat: "", group, member };
		},
		async groupRemoveMember(_projectId, _targetRef, group, member) {
			return { authorities: [], caveat: "", group, member };
		},
	};
</script>

<PrincipalEditor
	{projectId}
	{targetRef}
	principalId="settings-agent"
	ownGrants={["contents:read"]}
	groupMemberships={[]}
	inheritedGrants={[]}
	{service}
/>

<output data-testid="principal-editor-ipc-error-calls">{calls}</output>
