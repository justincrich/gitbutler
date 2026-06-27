<script lang="ts">
	import PrincipalsList from "$components/governance/PrincipalsList.svelte";
	import type { PrincipalsListEntry } from "$components/governance/PrincipalsList.svelte";

	const WRITE = [
		"metadata:read",
		"contents:read",
		"contents:write",
		"pull_requests:read",
		"pull_requests:write",
		"reviews:write",
		"comments:write",
		"statuses:read",
		"statuses:write",
	];
	const principals: PrincipalsListEntry[] = [
		{ principalId: "rust-implementer", ownGrants: WRITE, inheritedGrants: [], groupMemberships: [] },
		{
			principalId: "operator",
			ownGrants: [...WRITE, "merge", "administration:read", "administration:write"],
			inheritedGrants: [],
			groupMemberships: [],
		},
	];
</script>

<!-- 420px-wide flex column with align-items: flex-start — the exact shape of the real
     `.governance-panel--principals` that previously made the matrix overflow instead of scroll. -->
<div style="width: 420px; display: flex; flex-direction: column; align-items: flex-start;">
	<PrincipalsList projectId="p" targetRef="refs/remotes/origin/master" isReadOnly {principals} />
</div>
