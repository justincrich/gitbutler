// Single source of truth for governance capability (authority) metadata used by the
// Permissions & Governance UI. The canonical vocabulary lives in
// `crates/but-authz/src/authority.rs` (the `Authority` enum, 12 variants). This table
// mirrors that vocabulary and attaches human-readable labels, descriptions, a display
// category, and whether the capability is actually enforced by a gate in v1 — so the
// Principals matrix and Groups tab can explain what each permission means instead of
// printing raw `resource:action` tokens.

export type CapabilityCategoryId =
	| "repository"
	| "pull-requests"
	| "review"
	| "merge"
	| "statuses"
	| "administration";

export type Capability = {
	/** Canonical authority token, e.g. `contents:write`. Matches the backend exactly. */
	authority: string;
	/** Short column header used inside a category group, e.g. "Write". */
	short: string;
	/** Full human label, e.g. "Write contents". */
	label: string;
	/** One-sentence plain-language meaning. */
	description: string;
	/** Display grouping for the matrix columns and the glossary. */
	category: CapabilityCategoryId;
	/** True when a gate actually enforces this capability in v1; false = catalog-only. */
	enforced: boolean;
};

export type CapabilityCategory = {
	id: CapabilityCategoryId;
	label: string;
	capabilities: Capability[];
};

// Ordered to match `Authority` in crates/but-authz/src/authority.rs.
export const CAPABILITY_CATALOG: Capability[] = [
	{
		authority: "metadata:read",
		short: "Metadata",
		label: "Read metadata",
		description: "Read repository metadata — branches, tags, and refs.",
		category: "repository",
		enforced: false
	},
	{
		authority: "contents:read",
		short: "Read",
		label: "Read contents",
		description: "Read repository files, branches, and commits.",
		category: "repository",
		enforced: false
	},
	{
		authority: "contents:write",
		short: "Write",
		label: "Write contents",
		description: "Create commits and push changes — required to pass the commit gate.",
		category: "repository",
		enforced: true
	},
	{
		authority: "pull_requests:read",
		short: "Read",
		label: "Read pull requests",
		description: "View pull requests and their state.",
		category: "pull-requests",
		enforced: false
	},
	{
		authority: "pull_requests:write",
		short: "Write",
		label: "Write pull requests",
		description: "Open, edit, and publish pull requests.",
		category: "pull-requests",
		enforced: true
	},
	{
		authority: "reviews:write",
		short: "Reviews",
		label: "Write reviews",
		description: "Submit or update reviews — approve or request changes.",
		category: "review",
		enforced: true
	},
	{
		authority: "comments:write",
		short: "Comments",
		label: "Write comments",
		description: "Post and resolve review and pull-request comments.",
		category: "review",
		enforced: true
	},
	{
		authority: "merge",
		short: "Merge",
		label: "Merge",
		description: "Merge reviewed changes into a protected branch — required to pass the merge gate.",
		category: "merge",
		enforced: true
	},
	{
		authority: "statuses:read",
		short: "Read",
		label: "Read status checks",
		description: "Read CI and build status checks.",
		category: "statuses",
		enforced: false
	},
	{
		authority: "statuses:write",
		short: "Write",
		label: "Write status checks",
		description: "Set or update CI and build status checks.",
		category: "statuses",
		enforced: false
	},
	{
		authority: "administration:read",
		short: "Read",
		label: "Read administration",
		description: "Read the governance configuration.",
		category: "administration",
		enforced: false
	},
	{
		authority: "administration:write",
		short: "Write",
		label: "Administration",
		description: "Edit governance config and manage permissions — required to pass the admin gate.",
		category: "administration",
		enforced: true
	}
];

const CATEGORY_LABELS: Record<CapabilityCategoryId, string> = {
	repository: "Repository",
	"pull-requests": "Pull requests",
	review: "Review",
	merge: "Merge",
	statuses: "Status checks",
	administration: "Administration"
};

// Column/glossary order, left to right.
const CATEGORY_ORDER: CapabilityCategoryId[] = [
	"repository",
	"pull-requests",
	"review",
	"merge",
	"statuses",
	"administration"
];

const CAPABILITY_BY_AUTHORITY = new Map(CAPABILITY_CATALOG.map((cap) => [cap.authority, cap]));

/** Capabilities grouped by category, in display order — drives the matrix columns. */
export const CAPABILITY_CATEGORIES: CapabilityCategory[] = CATEGORY_ORDER.map((id) => ({
	id,
	label: CATEGORY_LABELS[id],
	capabilities: CAPABILITY_CATALOG.filter((cap) => cap.category === id)
}));

/** All known authority tokens, in catalog order. */
export const CAPABILITY_AUTHORITIES: string[] = CAPABILITY_CATALOG.map((cap) => cap.authority);

/**
 * Resolve metadata for an authority token. Unknown tokens (e.g. a future capability not
 * yet in this table) degrade gracefully to a row that still renders the raw token rather
 * than dropping it — important for a security surface where silently hiding a grant lies.
 */
export function describeAuthority(authority: string): Capability {
	const known = CAPABILITY_BY_AUTHORITY.get(authority);
	if (known) return known;
	return {
		authority,
		short: authority,
		label: authority,
		description: "Custom capability not in the standard catalog.",
		category: "repository",
		enforced: false
	};
}

/** Human label for an authority token, falling back to the raw token. */
export function authorityLabel(authority: string): string {
	return describeAuthority(authority).label;
}
