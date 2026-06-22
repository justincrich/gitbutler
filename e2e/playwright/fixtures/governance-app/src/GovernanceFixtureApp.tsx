import { useMemo, useState } from "react";

type Persona = "admin" | "member" | "read-only-admin";
type PageId = "project" | "git" | "ai" | "experimental" | "governance";
type TabId = "principals" | "groups" | "branch-gates" | "rules";
type PrincipalId = "settings-agent" | "claude-agent";
type GroupId = "eng" | "reviewers";

type Principal = {
	id: PrincipalId;
	ownGrants: string[];
	inheritedGrants: string[];
	groups: GroupId[];
	pending: boolean;
};

type Group = {
	id: GroupId;
	label: string;
	members: string[];
	grants: string[];
	pending: boolean;
};

const permissionRows = [
	"contents:read",
	"contents:write",
	"reviews:write",
	"merge",
	"administration:write",
];

const pageItems: Array<{ id: PageId; label: string; adminOnly?: boolean }> = [
	{ id: "project", label: "Project" },
	{ id: "git", label: "Git stuff" },
	{ id: "ai", label: "AI options" },
	{ id: "experimental", label: "Experimental" },
	{ id: "governance", label: "Permissions & Governance", adminOnly: true },
];

const fixtureEvidenceLabels = [
	"admin-visible.fixture-evidence",
	"non-admin-hidden.fixture-evidence",
	"four-tabs.fixture-evidence",
	"read-only-admin.fixture-evidence",
	"principal-pending.fixture-evidence",
	"group-pending.fixture-evidence",
	"cross-tab-pending.fixture-evidence",
	"post-commit-clean.fixture-evidence",
];

function initialPrincipals(): Principal[] {
	return [
		{
			id: "settings-agent",
			ownGrants: ["contents:read"],
			inheritedGrants: [],
			groups: [],
			pending: false,
		},
		{
			id: "claude-agent",
			ownGrants: ["contents:read"],
			inheritedGrants: ["contents:write"],
			groups: ["eng"],
			pending: false,
		},
	];
}

function initialGroups(): Group[] {
	return [
		{
			id: "eng",
			label: "eng",
			members: ["claude-agent", "codex-agent"],
			grants: ["contents:read", "contents:write"],
			pending: false,
		},
		{
			id: "reviewers",
			label: "reviewers",
			members: ["claude-agent"],
			grants: ["contents:read", "reviews:write"],
			pending: false,
		},
	];
}

function slug(value: string) {
	return value.replace(/[^a-z0-9]+/gi, "-");
}

export function GovernanceFixtureApp() {
	const [persona, setPersona] = useState<Persona>("admin");
	const [selectedPage, setSelectedPage] = useState<PageId>("governance");
	const [selectedTab, setSelectedTab] = useState<TabId>("principals");
	const [principals, setPrincipals] = useState<Principal[]>(initialPrincipals);
	const [groups, setGroups] = useState<Group[]>(initialGroups);
	const [expandedGroup, setExpandedGroup] = useState<GroupId | undefined>();
	const [selectedPrincipal, setSelectedPrincipal] = useState<PrincipalId | undefined>();
	const [draftReviewGrant, setDraftReviewGrant] = useState(false);
	const [commitMessage, setCommitMessage] = useState("");

	const isAdmin = persona !== "member";
	const canWrite = persona === "admin";
	const pendingCount =
		principals.filter((principal) => principal.pending).length +
		groups.filter((group) => group.pending).length;

	const selectedPrincipalRecord = principals.find((principal) => principal.id === selectedPrincipal);
	const visiblePages = pageItems.filter((page) => !page.adminOnly || isAdmin);

	function resetGovernanceState() {
		setPrincipals(initialPrincipals());
		setGroups(initialGroups());
		setExpandedGroup(undefined);
		setSelectedPrincipal(undefined);
		setDraftReviewGrant(false);
		setCommitMessage("");
		setSelectedTab("principals");
	}

	function switchPersona(nextPersona: Persona) {
		setPersona(nextPersona);
		setSelectedPage(nextPersona === "member" ? "project" : "governance");
		resetGovernanceState();
	}

	function selectPrincipal(principalId: PrincipalId) {
		const nextPrincipal = principals.find((principal) => principal.id === principalId);
		setSelectedPrincipal(principalId);
		setDraftReviewGrant(Boolean(nextPrincipal?.ownGrants.includes("reviews:write")));
	}

	function savePrincipal() {
		if (!selectedPrincipal || !canWrite) return;
		setPrincipals((current) =>
			current.map((principal) => {
				if (principal.id !== selectedPrincipal) return principal;
				const ownGrants = draftReviewGrant
					? [...new Set([...principal.ownGrants, "reviews:write"])]
					: principal.ownGrants.filter((grant) => grant !== "reviews:write");
				return { ...principal, ownGrants, pending: true };
			}),
		);
	}

	function toggleGroupGrant(groupId: GroupId, grant: string, checked: boolean) {
		if (!canWrite) return;
		setGroups((current) =>
			current.map((group) => {
				if (group.id !== groupId) return group;
				const grants = checked
					? [...new Set([...group.grants, grant])]
					: group.grants.filter((value) => value !== grant);
				return { ...group, grants, pending: true };
			}),
		);
	}

	function commitChanges() {
		if (!canWrite || pendingCount === 0) return;
		setPrincipals((current) => current.map((principal) => ({ ...principal, pending: false })));
		setGroups((current) => current.map((group) => ({ ...group, pending: false })));
		setCommitMessage("Committed: chore: update governance config");
	}

	const personaLabel = useMemo(() => {
		if (persona === "admin") return "Admin";
		if (persona === "member") return "Member";
		return "Read-only admin";
	}, [persona]);

	return (
		<div className="app-shell">
			<header className="fixture-header">
				<div>
					<strong>Fixture governance harness</strong>
					<span>Not product E2E evidence</span>
					<span aria-label="Fixture evidence labels">
						{fixtureEvidenceLabels.join(" ")}
					</span>
				</div>
				<div className="persona-switcher" aria-label="Fixture user type">
					{(["admin", "member", "read-only-admin"] as Persona[]).map((value) => (
						<button
							type="button"
							key={value}
							className={persona === value ? "active" : ""}
							onClick={() => switchPersona(value)}
						>
							{value === "read-only-admin" ? "Read-only admin" : value}
						</button>
					))}
				</div>
			</header>

			<main className="settings-modal" aria-label="Project settings">
				<aside className="settings-sidebar">
					<h1>Project settings</h1>
					<p className="persona-label" data-testid="fixture-persona">
						{personaLabel}
					</p>
					<nav aria-label="Project settings pages">
						{visiblePages.map((page) => (
							<button
								type="button"
								key={page.id}
								className={selectedPage === page.id ? "selected" : ""}
								onClick={() => setSelectedPage(page.id)}
							>
								{page.label}
							</button>
						))}
					</nav>
				</aside>

				<section className="settings-content">
					{selectedPage !== "governance" && (
						<div className="standard-page">
							<h2>{pageItems.find((page) => page.id === selectedPage)?.label}</h2>
							<p>Standard project settings remain visible for this fixture user.</p>
						</div>
					)}

					{selectedPage === "governance" && isAdmin && (
						<section className="governance-settings" data-testid="governance-settings">
							<h2>Permissions & Governance</h2>

							{!canWrite && (
								<div className="info-message" data-testid="governance-read-only-message">
									<strong>Read-only governance settings</strong>
									<span>You need administration:write authority to edit governance settings.</span>
								</div>
							)}

							{pendingCount > 0 && canWrite && (
								<div className="pending-banner" data-testid="governance-pending-banner">
									<div>
										<strong>{pendingCount} pending changes</strong>
										<span>Commit governance changes to the configured target branch.</span>
									</div>
									<button
										type="button"
										className="primary"
										data-testid="governance-commit-button"
										onClick={commitChanges}
									>
										Commit changes
									</button>
								</div>
							)}

							{commitMessage && (
								<p className="commit-result" data-testid="governance-commit-result">
									{commitMessage}
								</p>
							)}

							<div className="tabs" role="tablist" aria-label="Governance sections">
								{[
									["principals", "Principals"],
									["groups", "Groups"],
									["branch-gates", "Branch Gates"],
									["rules", "Rules"],
								].map(([id, label]) => (
									<button
										type="button"
										role="tab"
										aria-selected={selectedTab === id}
										key={id}
										className={selectedTab === id ? "active" : ""}
										onClick={() => setSelectedTab(id as TabId)}
									>
										{label}
									</button>
								))}
							</div>

							{selectedTab === "principals" && (
								<section className="panel" data-testid="governance-principals-panel">
									<div className="panel-heading">
										<h3>Principals</h3>
										<button type="button" disabled={!canWrite}>
											Add
										</button>
									</div>
									<div className="list">
										{principals.map((principal) => (
											<div className="list-row-wrap" key={principal.id}>
												<button
													type="button"
													className="list-row"
													data-testid={`principals-list-row-${slug(principal.id)}`}
													aria-expanded={selectedPrincipal === principal.id}
													onClick={() => selectPrincipal(principal.id)}
												>
													<span className="subject">
														{principal.pending && (
															<span
																className="pending-dot"
																aria-label="pending"
																data-testid={`principals-list-pending-${slug(principal.id)}`}
															/>
														)}
														<strong>{principal.id}</strong>
													</span>
													<span className="chips">
														{principal.ownGrants.map((grant) => (
															<span className="chip" key={grant}>
																{grant}
																<small>own grant</small>
															</span>
														))}
														{principal.inheritedGrants.map((grant) => (
															<span className="chip inherited" key={grant}>
																{grant}
																<small>from group eng</small>
															</span>
														))}
													</span>
													<span className="chips">
														{principal.groups.map((group) => (
															<span className="tag" key={group}>
																group: {group}
															</span>
														))}
													</span>
												</button>

												{selectedPrincipalRecord?.id === principal.id && (
													<div className="editor" data-testid="principal-editor">
														<h4>{principal.id}</h4>
														<label className="checkbox-row">
															<input
																type="checkbox"
																checked={draftReviewGrant}
																disabled={!canWrite}
																onChange={(event) => setDraftReviewGrant(event.target.checked)}
															/>
															<span>reviews:write own grant</span>
														</label>
														<label className="checkbox-row muted">
															<input type="checkbox" checked disabled />
															<span>contents:write inherited from group eng</span>
														</label>
														<button
															type="button"
															data-testid="principal-editor-save"
															disabled={!canWrite}
															onClick={savePrincipal}
														>
															Save changes
														</button>
													</div>
												)}
											</div>
										))}
									</div>
								</section>
							)}

							{selectedTab === "groups" && (
								<section className="panel" data-testid="governance-groups-panel">
									<div className="panel-heading">
										<h3>Groups</h3>
										<button type="button" disabled={!canWrite}>
											New group
										</button>
									</div>
									<div className="list">
										{groups.map((group) => (
											<div className="group" key={group.id} data-testid={`groups-list-row-${group.id}`}>
												<button
													type="button"
													className="list-row"
													aria-expanded={expandedGroup === group.id}
													onClick={() => setExpandedGroup(expandedGroup === group.id ? undefined : group.id)}
												>
													<span className="subject">
														{group.pending && (
															<span
																className="pending-dot"
																aria-label="pending"
																data-testid={`groups-list-pending-${group.id}`}
															/>
														)}
														<strong>{group.label}</strong>
													</span>
													<span className="tag">{group.members.length} members</span>
													<span className="tag">{group.grants.length} grants</span>
												</button>

												{expandedGroup === group.id && (
													<div className="editor" data-testid={`groups-list-editor-${group.id}`}>
														<div className="permission-grid">
															{permissionRows.map((grant) => (
																<label className="checkbox-row" key={grant}>
																	<input
																		type="checkbox"
																		checked={group.grants.includes(grant)}
																		disabled={!canWrite}
																		onChange={(event) =>
																			toggleGroupGrant(group.id, grant, event.target.checked)
																		}
																	/>
																	<span>{grant}</span>
																</label>
															))}
														</div>
													</div>
												)}
											</div>
										))}
									</div>
								</section>
							)}

							{selectedTab === "branch-gates" && (
								<section className="panel split" data-testid="governance-branch-gates-panel">
									<div>
										<h3>Branch Gates</h3>
										<p>main requires reviewers before merge.</p>
									</div>
									<button type="button" data-testid="governance-branch-gates-control" disabled={!canWrite}>
										Add gate
									</button>
								</section>
							)}

							{selectedTab === "rules" && (
								<section className="panel split" data-testid="governance-rules-panel">
									<div>
										<h3>Rules</h3>
										<p>Require review before merge.</p>
									</div>
									<button type="button" data-testid="governance-rules-control" disabled={!canWrite}>
										Add rule
									</button>
								</section>
							)}
						</section>
					)}
				</section>
			</main>
		</div>
	);
}
